// SPDX-License-Identifier: MPL-2.0
#include "OgnPublishLidarPointCloudToRustDatabase.h"
#include "isaacsimrs/forward.hpp"
#include <carb/logging/Log.h>
#include <cuda_runtime.h>
#include <cstddef>
#include <cstdint>

namespace
{

// Per-thread (and per-device-index) pinned host staging buffer for
// GPU→host copies. Pinned memory hits a faster DMA path than paged —
// roughly 2× the host-side throughput for multi-MB buffers, which
// matters once camera frames (8+ MB at 60 Hz) flow through this path.
//
// Allocated on first use, grown on demand (cudaFreeHost + realloc).
// Process-lifetime — explicit cleanup is unnecessary given the OGN
// node's thread-local context.
struct GpuStaging
{
    int cuda_idx{ -1 };
    void* host_ptr{ nullptr };
    std::size_t host_capacity{ 0 };
    cudaStream_t stream{ nullptr };
    cudaEvent_t event{ nullptr };
    bool initialised{ false };

    bool ensure_initialised(int idx)
    {
        if (initialised && cuda_idx == idx) {
            return true;
        }
        // Device or first-use change — recreate stream + event on the
        // active context.
        if (auto rc = cudaSetDevice(idx); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishLidarPointCloudToRust] cudaSetDevice(%d) failed: %s",
                idx, cudaGetErrorString(rc));
            return false;
        }
        if (initialised) {
            // Releasing on the previous device.
            if (host_ptr) {
                cudaFreeHost(host_ptr);
                host_ptr = nullptr;
                host_capacity = 0;
            }
            if (stream) {
                cudaStreamDestroy(stream);
                stream = nullptr;
            }
            if (event) {
                cudaEventDestroy(event);
                event = nullptr;
            }
        }
        // See OgnPublishCameraRgbToRust for the rationale: cudaStreamCreate
        // without cudaStreamNonBlocking creates a stream that implicitly
        // syncs with the default stream, defeating the async memcpy intent.
        // Harmless today because the LiDAR PointCloud annotator publishes
        // host-resident buffers (cudaDeviceIndex == -1), so the CUDA branch
        // never runs — but the bug would surface immediately if a future
        // annotator ships GPU-resident points. Fix proactively.
        if (auto rc = cudaStreamCreateWithFlags(&stream, cudaStreamNonBlocking); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishLidarPointCloudToRust] cudaStreamCreateWithFlags failed: %s",
                cudaGetErrorString(rc));
            return false;
        }
        if (auto rc = cudaEventCreateWithFlags(&event, cudaEventDisableTiming); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishLidarPointCloudToRust] cudaEventCreate failed: %s",
                cudaGetErrorString(rc));
            return false;
        }
        cuda_idx = idx;
        initialised = true;
        return true;
    }

    bool ensure_capacity(std::size_t bytes)
    {
        if (host_capacity >= bytes) {
            return true;
        }
        if (host_ptr) {
            cudaFreeHost(host_ptr);
            host_ptr = nullptr;
            host_capacity = 0;
        }
        // Grow geometrically to avoid per-frame realloc on small
        // fluctuations.
        std::size_t new_cap = bytes * 2;
        if (auto rc = cudaHostAlloc(&host_ptr, new_cap, cudaHostAllocPortable);
            rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishLidarPointCloudToRust] cudaHostAlloc(%zu) failed: %s",
                new_cap, cudaGetErrorString(rc));
            host_ptr = nullptr;
            host_capacity = 0;
            return false;
        }
        host_capacity = new_cap;
        return true;
    }
};

}

class OgnPublishLidarPointCloudToRust
{
public:
    static bool compute(OgnPublishLidarPointCloudToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;
        const auto cuda_idx = db.inputs.cudaDeviceIndex();
        const auto data_ptr = db.inputs.dataPtr();
        const auto buffer_size = db.inputs.bufferSize();
        if (data_ptr == 0 || buffer_size == 0) {
            return false;
        }

        constexpr std::size_t kStride = 3 * sizeof(float);
        if (buffer_size % kStride != 0) {
            return false;
        }
        const auto num_points = static_cast<std::size_t>(buffer_size / kStride);
        const auto num_floats = num_points * 3;

        const float* host_ptr = nullptr;

        if (cuda_idx == -1) {
            host_ptr = reinterpret_cast<const float*>(data_ptr);
        } else {
            static thread_local GpuStaging staging;
            if (!staging.ensure_initialised(cuda_idx)) {
                return false;
            }
            if (!staging.ensure_capacity(buffer_size)) {
                return false;
            }
            if (auto rc = cudaMemcpyAsync(
                    staging.host_ptr,
                    reinterpret_cast<const void*>(data_ptr),
                    buffer_size,
                    cudaMemcpyDeviceToHost,
                    staging.stream);
                rc != cudaSuccess) {
                CARB_LOG_ERROR(
                    "[OgnPublishLidarPointCloudToRust] cudaMemcpyAsync DtoH failed (cuda=%d size=%zu): %s",
                    cuda_idx, static_cast<std::size_t>(buffer_size), cudaGetErrorString(rc));
                return false;
            }
            // Record + sync on the dedicated stream rather than blocking
            // the entire device. Lets unrelated GPU work continue
            // while this copy completes.
            cudaEventRecord(staging.event, staging.stream);
            if (auto rc = cudaEventSynchronize(staging.event); rc != cudaSuccess) {
                CARB_LOG_ERROR(
                    "[OgnPublishLidarPointCloudToRust] cudaEventSynchronize failed: %s",
                    cudaGetErrorString(rc));
                return false;
            }
            host_ptr = static_cast<const float*>(staging.host_ptr);
        }

        rust::Slice<const float> points{ host_ptr, num_floats };

        isaacsimrs::LidarPointCloudMeta meta{
            static_cast<std::int32_t>(num_points),
            static_cast<std::int32_t>(db.inputs.width()),
            static_cast<std::int32_t>(db.inputs.height()),
        };

        isaacsimrs::forward_lidar_pointcloud(str_from(db.inputs.sourceId()), points, meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

#include "OgnPublishCameraRgbToRustDatabase.h"
#include "isaacsimrs/forward.hpp"
#include <carb/logging/Log.h>
#include <cuda_runtime.h>
#include <cstddef>
#include <cstdint>

namespace
{

// Mirrors the staging struct in OgnPublishLidarPointCloudToRust.cpp. RTX
// renderer outputs (RGBA) and the IsaacConvertRGBAToRGB output ride
// through the same GPU memory; an RGB frame at 1080p is ~6 MB so the
// async + pinned path matters at 60 Hz.
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
        if (auto rc = cudaSetDevice(idx); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishCameraRgbToRust] cudaSetDevice(%d) failed: %s",
                idx, cudaGetErrorString(rc));
            return false;
        }
        if (initialised) {
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
        if (auto rc = cudaStreamCreate(&stream); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishCameraRgbToRust] cudaStreamCreate failed: %s",
                cudaGetErrorString(rc));
            return false;
        }
        if (auto rc = cudaEventCreateWithFlags(&event, cudaEventDisableTiming); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishCameraRgbToRust] cudaEventCreate failed: %s",
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
        std::size_t new_cap = bytes * 2;
        if (auto rc = cudaHostAlloc(&host_ptr, new_cap, cudaHostAllocPortable);
            rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishCameraRgbToRust] cudaHostAlloc(%zu) failed: %s",
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

class OgnPublishCameraRgbToRust
{
public:
    static bool compute(OgnPublishCameraRgbToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;
        const auto cuda_idx = db.inputs.cudaDeviceIndex();
        const auto data_ptr = db.inputs.dataPtr();
        const auto buffer_size = static_cast<std::size_t>(db.inputs.bufferSize());
        const auto width = static_cast<std::size_t>(db.inputs.width());
        const auto height = static_cast<std::size_t>(db.inputs.height());

        if (data_ptr == 0 || buffer_size == 0 || width == 0 || height == 0) {
            return false;
        }

        // RGB8 — three bytes per pixel. Drop frames whose payload doesn't
        // line up with the declared resolution rather than dispatching a
        // mis-sized buffer adapters would render as garbage.
        if (buffer_size != width * height * 3) {
            CARB_LOG_WARN(
                "[OgnPublishCameraRgbToRust] buffer size mismatch: w=%zu h=%zu bufferSize=%zu (expected %zu)",
                width, height, buffer_size, width * height * 3);
            return false;
        }

        const std::uint8_t* host_ptr = nullptr;

        if (cuda_idx == -1) {
            host_ptr = reinterpret_cast<const std::uint8_t*>(data_ptr);
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
                    "[OgnPublishCameraRgbToRust] cudaMemcpyAsync DtoH failed (cuda=%d size=%zu): %s",
                    cuda_idx, buffer_size, cudaGetErrorString(rc));
                return false;
            }
            cudaEventRecord(staging.event, staging.stream);
            if (auto rc = cudaEventSynchronize(staging.event); rc != cudaSuccess) {
                CARB_LOG_ERROR(
                    "[OgnPublishCameraRgbToRust] cudaEventSynchronize failed: %s",
                    cudaGetErrorString(rc));
                return false;
            }
            host_ptr = static_cast<const std::uint8_t*>(staging.host_ptr);
        }

        rust::Slice<const std::uint8_t> pixels{ host_ptr, buffer_size };

        // Intrinsics are not on this annotator chain; downstream wiring
        // via IsaacReadCameraInfo can populate fx/fy/cx/cy in a later
        // pass. Adapters that only need the image (rerun::Image) can
        // ignore them.
        isaacsimrs::CameraRgbMeta meta{
            static_cast<std::int32_t>(width),
            static_cast<std::int32_t>(height),
            0.0f,
            0.0f,
            0.0f,
            0.0f,
            0,
        };

        isaacsimrs::forward_camera_rgb(str_from(db.inputs.sourceId()), pixels, meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

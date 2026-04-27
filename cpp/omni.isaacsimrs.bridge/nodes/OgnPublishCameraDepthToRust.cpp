#include "OgnPublishCameraDepthToRustDatabase.h"
#include "isaacsimrs/forward.hpp"
#include <carb/RenderingTypes.h>
#include <carb/logging/Log.h>
#include <cuda_runtime.h>
#include <cstddef>
#include <cstdint>

namespace
{

// Mirrors the staging struct in OgnPublishCameraRgbToRust.cpp /
// OgnPublishLidarPointCloudToRust.cpp. The IsaacPassthroughImagePtr
// annotator forwards GPU-resident render-product output, so a depth
// frame at 640x480 = ~1.2 MB / frame is staged via async DMA on a
// non-blocking stream (avoids implicit sync with the default RTX
// stream — see commit 0ad451b for the rationale).
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
                "[OgnPublishCameraDepthToRust] cudaSetDevice(%d) failed: %s",
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
        if (auto rc = cudaStreamCreateWithFlags(&stream, cudaStreamNonBlocking); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishCameraDepthToRust] cudaStreamCreateWithFlags failed: %s",
                cudaGetErrorString(rc));
            return false;
        }
        if (auto rc = cudaEventCreateWithFlags(&event, cudaEventDisableTiming); rc != cudaSuccess) {
            CARB_LOG_ERROR(
                "[OgnPublishCameraDepthToRust] cudaEventCreate failed: %s",
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
                "[OgnPublishCameraDepthToRust] cudaHostAlloc(%zu) failed: %s",
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

class OgnPublishCameraDepthToRust
{
public:
    static bool compute(OgnPublishCameraDepthToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;
        const auto cuda_idx = db.inputs.cudaDeviceIndex();
        const auto data_ptr = db.inputs.dataPtr();
        const auto reported_size = static_cast<std::size_t>(db.inputs.bufferSize());
        const auto width = static_cast<std::size_t>(db.inputs.width());
        const auto height = static_cast<std::size_t>(db.inputs.height());
        const auto pixel_format = static_cast<carb::Format>(db.inputs.format());

        if (data_ptr == 0 || width == 0 || height == 0) {
            return false;
        }

        // Float32 per pixel (carb::Format::eR32_SFLOAT). Linear byte
        // count derives from dims; the annotator's reported bufferSize
        // is 0 in texture mode and = width*height*4 in linear-buffer
        // mode.
        constexpr std::size_t kBytesPerPixel = sizeof(float);
        const std::size_t expected_bytes = width * height * kBytesPerPixel;
        if (reported_size != 0 && reported_size != expected_bytes) {
            CARB_LOG_WARN(
                "[OgnPublishCameraDepthToRust] buffer size mismatch: w=%zu h=%zu bufferSize=%zu (expected %zu)",
                width, height, reported_size, expected_bytes);
            return false;
        }

        const float* host_ptr = nullptr;

        if (cuda_idx == -1) {
            // Host-resident path — annotator already gave us a CPU
            // pointer to a linear buffer.
            host_ptr = reinterpret_cast<const float*>(data_ptr);
        } else {
            static thread_local GpuStaging staging;
            if (!staging.ensure_initialised(cuda_idx)) {
                return false;
            }
            if (!staging.ensure_capacity(expected_bytes)) {
                return false;
            }

            if (reported_size == 0) {
                // Texture-mode path. The rendervar's
                // DistanceToImagePlaneSDPtr output lands here as a
                // `cudaMipmappedArray_t` handle (NOT a linear device
                // pointer), with bufferSize == 0. Mirror NVIDIA's
                // OgnROS2PublishImage cpp: extract level-0 cudaArray,
                // then cudaMemcpy2DFromArrayAsync with row pitch =
                // width * sizeof(float) into a linear host staging
                // buffer.
                if (pixel_format != carb::Format::eR32_SFLOAT) {
                    CARB_LOG_ERROR(
                        "[OgnPublishCameraDepthToRust] texture-mode requires eR32_SFLOAT, got format=%llu",
                        static_cast<unsigned long long>(pixel_format));
                    return false;
                }
                cudaArray_t level_array = nullptr;
                if (auto rc = cudaGetMipmappedArrayLevel(
                        &level_array,
                        reinterpret_cast<cudaMipmappedArray_t>(data_ptr),
                        0);
                    rc != cudaSuccess) {
                    CARB_LOG_ERROR(
                        "[OgnPublishCameraDepthToRust] cudaGetMipmappedArrayLevel failed: %s",
                        cudaGetErrorString(rc));
                    return false;
                }
                const std::size_t row_pitch = width * kBytesPerPixel;
                if (auto rc = cudaMemcpy2DFromArrayAsync(
                        staging.host_ptr, row_pitch,
                        level_array, 0, 0,
                        row_pitch, height,
                        cudaMemcpyDeviceToHost,
                        staging.stream);
                    rc != cudaSuccess) {
                    CARB_LOG_ERROR(
                        "[OgnPublishCameraDepthToRust] cudaMemcpy2DFromArrayAsync failed (cuda=%d w=%zu h=%zu): %s",
                        cuda_idx, width, height, cudaGetErrorString(rc));
                    return false;
                }
            } else {
                // Linear-buffer path — straightforward DtoH copy.
                if (auto rc = cudaMemcpyAsync(
                        staging.host_ptr,
                        reinterpret_cast<const void*>(data_ptr),
                        expected_bytes,
                        cudaMemcpyDeviceToHost,
                        staging.stream);
                    rc != cudaSuccess) {
                    CARB_LOG_ERROR(
                        "[OgnPublishCameraDepthToRust] cudaMemcpyAsync DtoH failed (cuda=%d size=%zu): %s",
                        cuda_idx, expected_bytes, cudaGetErrorString(rc));
                    return false;
                }
            }

            cudaEventRecord(staging.event, staging.stream);
            if (auto rc = cudaEventSynchronize(staging.event); rc != cudaSuccess) {
                CARB_LOG_ERROR(
                    "[OgnPublishCameraDepthToRust] cudaEventSynchronize failed: %s",
                    cudaGetErrorString(rc));
                return false;
            }
            host_ptr = static_cast<const float*>(staging.host_ptr);
        }

        const std::size_t num_pixels = width * height;
        rust::Slice<const float> depths{ host_ptr, num_pixels };

        // Intrinsics not on this annotator chain; downstream wiring via
        // IsaacReadCameraInfo can populate fx/fy/cx/cy in a later pass.
        isaacsimrs::CameraDepthMeta meta{
            static_cast<std::int32_t>(width),
            static_cast<std::int32_t>(height),
            0.0f,
            0.0f,
            0.0f,
            0.0f,
            0,
        };

        isaacsimrs::forward_camera_depth(str_from(db.inputs.sourceId()), depths, meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

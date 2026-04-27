#include "OgnPublishLidarPointCloudToRustDatabase.h"
#include "isaacsimrs/forward.hpp"
#include <carb/logging/Log.h>
#include <cuda_runtime.h>
#include <cstddef>
#include <cstdint>
#include <vector>

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
        static thread_local std::vector<float> staging;


        if (cuda_idx == -1) {
            host_ptr = reinterpret_cast<const float*>(data_ptr);
        } else {
            // Multi-GPU rigs: the buffer lives on cuda_idx, not necessarily
            // device 0. Setting the current device before cudaMemcpy makes
            // the copy resolve against the correct context.
            if (auto rc = cudaSetDevice(cuda_idx); rc != cudaSuccess) {
                CARB_LOG_ERROR(
                    "[OgnPublishLidarPointCloudToRust] cudaSetDevice(%d) failed: %s",
                    cuda_idx, cudaGetErrorString(rc));
                return false;
            }
            staging.resize(num_floats);
            if (auto rc = cudaMemcpy(
                    staging.data(),
                    reinterpret_cast<const void*>(data_ptr),
                    buffer_size,
                    cudaMemcpyDeviceToHost);
                rc != cudaSuccess) {
                CARB_LOG_ERROR(
                    "[OgnPublishLidarPointCloudToRust] cudaMemcpy DtoH failed (cuda=%d size=%zu): %s",
                    cuda_idx, static_cast<std::size_t>(buffer_size), cudaGetErrorString(rc));
                return false;
            }
            host_ptr = staging.data();
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

#include "OgnPublishLidarPointCloudToRustDatabase.h"
#include "isaac-sim-bridge/src/lib.rs.h"
#include <cuda_runtime.h>
#include <cstddef>
#include <cstdint>
#include <vector>

class OgnPublishLidarPointCloudToRust
{
public:
    static bool compute(OgnPublishLidarPointCloudToRustDatabase& db)
    {
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
            staging.resize(num_floats);
            const auto rc = cudaMemcpy(
                staging.data(),
                reinterpret_cast<const void*>(data_ptr),
                buffer_size,
                cudaMemcpyDeviceToHost);
            if (rc != cudaSuccess) {
                return false;
            }
            host_ptr = staging.data();
        }

        rust::Slice<const float> points{ host_ptr, num_floats };

        const std::string& source = db.inputs.sourceId();
        rust::Str source_id{ source.data(), source.size() };

        isaacsimrs::LidarPointCloudMeta meta{
            static_cast<std::int32_t>(num_points),
            static_cast<std::int32_t>(db.inputs.width()),
            static_cast<std::int32_t>(db.inputs.height()),
        };

        isaacsimrs::forward_lidar_pointcloud(source_id, points, meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

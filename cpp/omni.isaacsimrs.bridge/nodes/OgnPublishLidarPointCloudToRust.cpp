#include "OgnPublishLidarPointCloudToRustDatabase.h"
#include "isaac-sim-bridge/src/lib.rs.h"
#include <cstddef>
#include <cstdint>

class OgnPublishLidarPointCloudToRust
{
public:
    static bool compute(OgnPublishLidarPointCloudToRustDatabase& db)
    {
        if (db.inputs.cudaDeviceIndex() != -1) {
            // GPU-resident buffers not yet supported; would need cudaMemcpy.
            return false;
        }

        const auto azimuth_ptr = db.inputs.azimuthPtr();
        const auto azimuth_size = db.inputs.azimuthBufferSize();
        if (azimuth_ptr == 0 || azimuth_size == 0) {
            return false;
        }

        const auto num_points = static_cast<std::size_t>(azimuth_size / sizeof(float));

        auto make_slice = [num_points](std::uint64_t ptr, std::uint64_t size) {
            const auto n = static_cast<std::size_t>(size / sizeof(float));
            if (ptr == 0 || n == 0 || n != num_points) {
                return rust::Slice<const float>{};
            }
            return rust::Slice<const float>{ reinterpret_cast<const float*>(ptr), n };
        };

        auto az_slice = make_slice(azimuth_ptr, azimuth_size);
        auto el_slice = make_slice(db.inputs.elevationPtr(), db.inputs.elevationBufferSize());
        auto dist_slice = make_slice(db.inputs.distancePtr(), db.inputs.distanceBufferSize());
        auto intens_slice = make_slice(db.inputs.intensityPtr(), db.inputs.intensityBufferSize());

        isaacsimrs::LidarPointCloudMeta meta{
            static_cast<std::int32_t>(num_points),
        };
        isaacsimrs::forward_lidar_pointcloud(az_slice, el_slice, dist_slice, intens_slice, meta);

        db.outputs.exec() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

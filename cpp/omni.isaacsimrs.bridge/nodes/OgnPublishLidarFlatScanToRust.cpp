#include "OgnPublishLidarFlatScanToRustDatabase.h"
#include "isaac-sim-bridge/src/lib.rs.h"

class OgnPublishLidarFlatScanToRust
{
public:
    static bool compute(OgnPublishLidarFlatScanToRustDatabase& db)
    {
        const auto& depths = db.inputs.linearDepthData();
        const auto& intensities = db.inputs.intensitiesData();
        const auto& azimuth = db.inputs.azimuthRange();
        const auto& depth = db.inputs.depthRange();

        isaacsimrs::LidarFlatScanMeta meta{
            db.inputs.horizontalFov(),
            db.inputs.horizontalResolution(),
            azimuth[0],
            azimuth[1],
            depth[0],
            depth[1],
            db.inputs.numRows(),
            db.inputs.numCols(),
            db.inputs.rotationRate(),
        };

        const std::string& source = db.inputs.sourceId();
        rust::Str source_id{ source.data(), source.size() };
        rust::Slice<const float> scan_slice{ depths.data(), depths.size() };
        rust::Slice<const std::uint8_t> intensity_slice{ intensities.data(), intensities.size() };

        isaacsimrs::forward_lidar_flatscan(source_id, scan_slice, intensity_slice, meta);

        db.outputs.exec() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

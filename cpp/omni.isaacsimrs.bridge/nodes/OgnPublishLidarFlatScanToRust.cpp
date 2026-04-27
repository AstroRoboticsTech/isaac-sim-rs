#include "OgnPublishLidarFlatScanToRustDatabase.h"
#include "isaacsimrs/forward.hpp"

class OgnPublishLidarFlatScanToRust
{
public:
    static bool compute(OgnPublishLidarFlatScanToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;
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

        isaacsimrs::forward_lidar_flatscan(
            str_from(db.inputs.sourceId()),
            slice_from<float>(db.inputs.linearDepthData()),
            slice_from<std::uint8_t>(db.inputs.intensitiesData()),
            meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

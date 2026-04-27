#include "OgnPublishCameraInfoToRustDatabase.h"
#include "isaacsimrs/forward.hpp"

class OgnPublishCameraInfoToRust
{
public:
    static bool compute(OgnPublishCameraInfoToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;

        isaacsimrs::CameraInfoMeta meta{
            static_cast<std::int32_t>(db.inputs.width()),
            static_cast<std::int32_t>(db.inputs.height()),
            // timeStamp arrives in seconds (NVIDIA convention); pack
            // as nanoseconds to match the rgb/depth meta types.
            static_cast<std::int64_t>(db.inputs.timeStamp() * 1.0e9),
        };

        isaacsimrs::forward_camera_info(
            str_from(db.inputs.sourceId()),
            str_from(db.inputs.frameId()),
            str_from(db.inputs.physicalDistortionModel()),
            str_from(db.inputs.projectionType()),
            slice_from<double>(db.inputs.k()),
            slice_from<double>(db.inputs.r()),
            slice_from<double>(db.inputs.p()),
            slice_from<float>(db.inputs.physicalDistortionCoefficients()),
            meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

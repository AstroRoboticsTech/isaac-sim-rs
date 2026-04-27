#include "OgnPublishImuToRustDatabase.h"
#include "isaacsimrs/forward.hpp"
#include <pxr/base/gf/quatd.h>
#include <pxr/base/gf/vec3d.h>

class OgnPublishImuToRust
{
public:
    static bool compute(OgnPublishImuToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;

        const pxr::GfVec3d& lin_acc = db.inputs.linearAcceleration();
        const pxr::GfVec3d& ang_vel = db.inputs.angularVelocity();
        const pxr::GfQuatd& orientation = db.inputs.orientation();
        const pxr::GfVec3d imag = orientation.GetImaginary();

        isaacsimrs::ImuMeta meta{
            lin_acc[0],
            lin_acc[1],
            lin_acc[2],
            ang_vel[0],
            ang_vel[1],
            ang_vel[2],
            orientation.GetReal(),
            imag[0],
            imag[1],
            imag[2],
            // timeStamp arrives in seconds; pack as nanoseconds so the
            // meta type stays consistent with rgb/depth/info.
            static_cast<std::int64_t>(db.inputs.timeStamp() * 1.0e9),
        };

        isaacsimrs::forward_imu(
            str_from(db.inputs.sourceId()),
            str_from(db.inputs.frameId()),
            meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

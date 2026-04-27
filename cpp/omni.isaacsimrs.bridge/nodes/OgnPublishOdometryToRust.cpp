#include "OgnPublishOdometryToRustDatabase.h"
#include "isaacsimrs/forward.hpp"
#include <pxr/base/gf/quatd.h>
#include <pxr/base/gf/vec3d.h>

class OgnPublishOdometryToRust
{
public:
    static bool compute(OgnPublishOdometryToRustDatabase& db)
    {
        using namespace isaacsimrs::detail;

        const pxr::GfVec3d& pos = db.inputs.position();
        const pxr::GfVec3d& lin_vel = db.inputs.linearVelocity();
        const pxr::GfVec3d& ang_vel = db.inputs.angularVelocity();
        const pxr::GfQuatd& orientation = db.inputs.orientation();
        const pxr::GfVec3d imag = orientation.GetImaginary();

        isaacsimrs::OdometryMeta meta{
            pos[0],
            pos[1],
            pos[2],
            orientation.GetReal(),
            imag[0],
            imag[1],
            imag[2],
            lin_vel[0],
            lin_vel[1],
            lin_vel[2],
            ang_vel[0],
            ang_vel[1],
            ang_vel[2],
            static_cast<std::int64_t>(db.inputs.timeStamp() * 1.0e9),
        };

        isaacsimrs::forward_odometry(
            str_from(db.inputs.sourceId()),
            str_from(db.inputs.chassisFrameId()),
            str_from(db.inputs.odomFrameId()),
            meta);

        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

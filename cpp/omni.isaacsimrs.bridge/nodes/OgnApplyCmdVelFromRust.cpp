#include "OgnApplyCmdVelFromRustDatabase.h"
#include "isaacsimrs/forward.hpp"

class OgnApplyCmdVelFromRust
{
public:
    static bool compute(OgnApplyCmdVelFromRustDatabase& db)
    {
        using namespace isaacsimrs::detail;

        isaacsimrs::CmdVel out{};
        const bool hit = isaacsimrs::poll_cmd_vel(str_from(db.inputs.targetId()), out);

        // Zero on miss (already zeroed by `out{}`); on hit we forward
        // body-frame linear x and yaw rate. Differential drive only
        // consumes those two components, and ApplyCmdVel stays generic
        // enough to feed any downstream Twist-shaped controller.
        db.outputs.linearVelocity() = hit ? static_cast<double>(out.linear_x) : 0.0;
        db.outputs.angularVelocity() = hit ? static_cast<double>(out.angular_z) : 0.0;
        db.outputs.execOut() = kExecutionAttributeStateEnabled;
        return true;
    }
};

REGISTER_OGN_NODE()

// SPDX-License-Identifier: MPL-2.0
#include "OgnEchoDatabase.h"
#include "isaac-sim-bridge/src/lib.rs.h"

class OgnEcho
{
public:
    static bool compute(OgnEchoDatabase& db)
    {
        db.outputs.doubled() = isaacsimrs::double_value(db.inputs.value());
        return true;
    }
};

REGISTER_OGN_NODE()

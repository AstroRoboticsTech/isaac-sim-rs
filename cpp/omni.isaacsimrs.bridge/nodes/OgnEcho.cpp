#include "OgnEchoDatabase.h"

class OgnEcho
{
public:
    static bool compute(OgnEchoDatabase& db)
    {
        db.outputs.doubled() = db.inputs.value() * 2;
        return true;
    }
};

REGISTER_OGN_NODE()

#define CARB_EXPORTS
#include <carb/PluginUtils.h>
#include <carb/logging/Log.h>
#include <omni/graph/core/ogn/Registration.h>
#include "isaac-sim-bridge/src/lib.rs.h"

namespace isaacsimrs
{
struct IBridge
{
    CARB_PLUGIN_INTERFACE("isaacsimrs.bridge.IBridge", 0, 1)
};
}

const struct carb::PluginImplDesc kPluginImpl = {
    "omni.isaacsimrs.bridge",
    "Rust FFI host extension",
    "Astro Robotics",
    carb::PluginHotReload::eDisabled,
    "dev"
};

CARB_PLUGIN_IMPL(kPluginImpl, isaacsimrs::IBridge)
CARB_PLUGIN_IMPL_NO_DEPS()
DECLARE_OGN_NODES()

void fillInterface(isaacsimrs::IBridge& iface)
{
    (void)iface;
    INITIALIZE_OGN_NODES();
}

namespace
{
struct EagerInit
{
    EagerInit()
    {
        isaacsimrs::init();
        INITIALIZE_OGN_NODES();
    }
};

static EagerInit g_eager_init;
}

void carbOnPluginStartup()
{
    CARB_LOG_INFO("[omni.isaacsimrs.bridge] hello from carbOnPluginStartup");
    INITIALIZE_OGN_NODES();
}

void carbOnPluginShutdown()
{
    RELEASE_OGN_NODES();
    CARB_LOG_INFO("[omni.isaacsimrs.bridge] shutting down");
}

#define CARB_EXPORTS
#include <carb/PluginUtils.h>
#include <carb/logging/Log.h>

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

void fillInterface(isaacsimrs::IBridge& iface)
{
    (void)iface;
}

void carbOnPluginStartup()
{
    CARB_LOG_INFO("[omni.isaacsimrs.bridge] hello from carbOnPluginStartup");
}

void carbOnPluginShutdown()
{
    CARB_LOG_INFO("[omni.isaacsimrs.bridge] shutting down");
}

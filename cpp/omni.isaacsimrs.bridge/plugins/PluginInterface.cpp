#define CARB_EXPORTS
#include <carb/PluginUtils.h>
#include <carb/logging/Log.h>
#include <dlfcn.h>
#include <cstdlib>

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

namespace
{
struct RustCdylibLoader
{
    RustCdylibLoader()
    {
        const char* path = std::getenv("ISAAC_SIM_RS_CDYLIB");
        if (!path) { return; }
        void* handle = dlopen(path, RTLD_NOW | RTLD_GLOBAL);
        if (!handle) { return; }
        auto init_fn = reinterpret_cast<void (*)()>(dlsym(handle, "isaac_sim_rs_init"));
        if (init_fn) { init_fn(); }
    }
};

static RustCdylibLoader g_loader;
}

void carbOnPluginStartup()
{
    CARB_LOG_INFO("[omni.isaacsimrs.bridge] hello from carbOnPluginStartup");
}

void carbOnPluginShutdown()
{
    CARB_LOG_INFO("[omni.isaacsimrs.bridge] shutting down");
}

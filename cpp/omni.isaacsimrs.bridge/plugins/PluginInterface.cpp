#define CARB_EXPORTS
#include <carb/PluginUtils.h>
#include <carb/logging/Log.h>
#include <omni/graph/core/ogn/Registration.h>
#include "isaac-sim-bridge/src/lib.rs.h"
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
DECLARE_OGN_NODES()

void fillInterface(isaacsimrs::IBridge& iface)
{
    (void)iface;
    INITIALIZE_OGN_NODES();
}

namespace
{
void load_optional_runner(const char* env_var, const char* init_symbol)
{
    const char* path = std::getenv(env_var);
    if (!path) { return; }
    void* handle = dlopen(path, RTLD_NOW | RTLD_GLOBAL);
    if (!handle)
    {
        CARB_LOG_WARN("[omni.isaacsimrs.bridge] dlopen %s failed: %s", path, dlerror());
        return;
    }
    auto init_fn = reinterpret_cast<int (*)()>(dlsym(handle, init_symbol));
    if (!init_fn)
    {
        CARB_LOG_WARN("[omni.isaacsimrs.bridge] dlsym %s failed: %s", init_symbol, dlerror());
        return;
    }
    if (int rc = init_fn(); rc != 0)
    {
        CARB_LOG_WARN("[omni.isaacsimrs.bridge] %s returned %d", init_symbol, rc);
    }
}

struct EagerInit
{
    EagerInit()
    {
        isaacsimrs::init();
        INITIALIZE_OGN_NODES();
        load_optional_runner("ISAAC_SIM_RS_DORA_RUNNER", "isaac_sim_dora_init");
        load_optional_runner("ISAAC_SIM_RS_RERUN_RUNNER", "isaac_sim_rerun_init");
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

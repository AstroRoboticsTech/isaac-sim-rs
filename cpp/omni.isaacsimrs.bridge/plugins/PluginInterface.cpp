// SPDX-License-Identifier: MPL-2.0
#define CARB_EXPORTS
#include <carb/PluginUtils.h>
#include <carb/logging/Log.h>
#include <carb/settings/ISettings.h>
#include <omni/graph/core/ogn/Registration.h>
#include "isaac-sim-bridge/src/lib.rs.h"
#include <dlfcn.h>
#include <cstdlib>
#include <cstring>
#include <string>

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

// INITIALIZE_OGN_NODES() is called from THREE Kit lifecycle hooks
// (static-init, fillInterface, carbOnPluginStartup). This is intentional —
// Kit's extension manager loads the plugin .so lazily and the timing of
// each hook varies depending on whether something requests the IBridge
// interface and whether `--exec drive.py` runs before plugins finish
// startup. Calling INITIALIZE_OGN_NODES from multiple hooks is the
// empirically reliable way to ensure OG node types are registered before
// any code tries to instantiate them. The macro is idempotent.
void fillInterface(isaacsimrs::IBridge& iface)
{
    (void)iface;
    INITIALIZE_OGN_NODES();
}

namespace
{

// Resolve the directory that contains this plugin .so at runtime.
// dladdr on a local symbol gives us the .so path; stripping the filename
// gives bin/linux-x86_64/, whose parent is the extension root.
std::string plugin_so_dir()
{
    Dl_info info{};
    if (dladdr(reinterpret_cast<void*>(&plugin_so_dir), &info) && info.dli_fname)
    {
        std::string path(info.dli_fname);
        auto slash = path.rfind('/');
        if (slash != std::string::npos)
            return path.substr(0, slash);
    }
    return {};
}

void load_adapter(const std::string& name, const std::string& adapter_dir)
{
    const std::string lib_name = "libisaac_sim_" + name + ".so";
    const std::string init_sym = "isaac_sim_" + name + "_init";

    // Legacy source-build override: if the env var is set, use it and warn.
    std::string legacy_env = "ISAAC_SIM_RS_" + name + "_RUNNER";
    for (auto& c : legacy_env)
        c = static_cast<char>(std::toupper(static_cast<unsigned char>(c)));

    const char* legacy_path = std::getenv(legacy_env.c_str());
    std::string resolved_path;
    if (legacy_path && legacy_path[0] != '\0')
    {
        CARB_LOG_WARN(
            "[omni.isaacsimrs.bridge] %s is set — using legacy path '%s'. "
            "Set adapters + adapter_path in extension.toml instead.",
            legacy_env.c_str(), legacy_path);
        resolved_path = legacy_path;
    }
    else if (!adapter_dir.empty())
    {
        resolved_path = adapter_dir + "/" + lib_name;
    }
    else
    {
        CARB_LOG_ERROR(
            "[omni.isaacsimrs.bridge] cannot resolve path for adapter '%s': "
            "adapter_path is empty and %s is not set",
            name.c_str(), legacy_env.c_str());
        return;
    }

    void* handle = dlopen(resolved_path.c_str(), RTLD_NOW | RTLD_GLOBAL);
    if (!handle)
    {
        CARB_LOG_ERROR(
            "[omni.isaacsimrs.bridge] dlopen '%s' failed: %s",
            resolved_path.c_str(), dlerror());
        return;
    }

    auto init_fn = reinterpret_cast<int (*)()>(dlsym(handle, init_sym.c_str()));
    if (!init_fn)
    {
        CARB_LOG_ERROR(
            "[omni.isaacsimrs.bridge] dlsym '%s' in '%s' failed: %s",
            init_sym.c_str(), resolved_path.c_str(), dlerror());
        return;
    }

    if (int rc = init_fn(); rc != 0)
    {
        CARB_LOG_ERROR(
            "[omni.isaacsimrs.bridge] adapter '%s' init returned %d",
            name.c_str(), rc);
        return;
    }

    CARB_LOG_INFO(
        "[omni.isaacsimrs.bridge] adapter '%s' loaded from '%s'",
        name.c_str(), resolved_path.c_str());
}

void load_adapters_from_settings()
{
    auto* settings = carb::getCachedInterface<carb::settings::ISettings>();
    if (!settings)
    {
        CARB_LOG_WARN("[omni.isaacsimrs.bridge] ISettings unavailable; no adapters loaded");
        return;
    }

    // Resolve the directory to search for adapter cdylibs.
    // Order: adapter_path setting (if non-empty) → bin/<platform>/ next to this .so.
    constexpr const char* kAdapterPathKey = "/settings/omni/isaacsimrs/bridge/adapter_path";
    const char* cfg_adapter_path = settings->getStringBuffer(kAdapterPathKey);

    std::string adapter_dir;
    if (cfg_adapter_path && cfg_adapter_path[0] != '\0')
    {
        adapter_dir = cfg_adapter_path;
    }
    else
    {
        adapter_dir = plugin_so_dir();
    }

    // Read the adapters list. Kit serialises TOML arrays as comma-separated
    // strings when accessed via getStringBuffer on the array key.
    constexpr const char* kAdaptersKey = "/settings/omni/isaacsimrs/bridge/adapters";
    const char* adapters_raw = settings->getStringBuffer(kAdaptersKey);
    std::string adapters_str;
    if (adapters_raw && adapters_raw[0] != '\0')
    {
        adapters_str = adapters_raw;
    }
    else
    {
        adapters_str = "dora,rerun";
    }
    std::string token;
    std::size_t start = 0;
    while (start <= adapters_str.size())
    {
        auto comma = adapters_str.find(',', start);
        if (comma == std::string::npos) comma = adapters_str.size();
        token = adapters_str.substr(start, comma - start);
        // Trim whitespace.
        auto b = token.find_first_not_of(" \t");
        auto e = token.find_last_not_of(" \t");
        if (b != std::string::npos)
            load_adapter(token.substr(b, e - b + 1), adapter_dir);
        start = comma + 1;
    }
}

struct EagerInit
{
    EagerInit()
    {
        isaacsimrs::init();
        INITIALIZE_OGN_NODES();
        load_adapters_from_settings();
    }
};

static EagerInit g_eager_init;
}

void carbOnPluginStartup()
{
    INITIALIZE_OGN_NODES();
}

void carbOnPluginShutdown()
{
    RELEASE_OGN_NODES();
}

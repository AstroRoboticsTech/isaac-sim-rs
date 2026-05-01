# Environment variables

`isaac-sim-rs` reads several environment variables across its source-build, runtime adapter, and per-dataflow routing surfaces. This document is the canonical reference.

End-user crates.io consumers (`cargo add isaac-sim-rs`) do not need any of these.

## Source-build (developer-only)

| Variable           | Required when                                    | Effect                                          | Where read                                          |
| ------------------ | ------------------------------------------------ | ----------------------------------------------- | --------------------------------------------------- |
| `ISAAC_SIM`        | Building the C++ extension from source           | Path to the Isaac Sim install root              | `examples/*/launch*.sh`, `justfile`                 |
| `ISAAC_SIM_PATH`   | Same as `ISAAC_SIM`; CMake uses this name        | Path to the Isaac Sim install root              | `cpp/omni.isaacsimrs.bridge/CMakeLists.txt`         |
| `ISAAC_SIM_RS`     | Running the in-tree examples                     | Path to this repo's checkout                    | `examples/*/launch*.sh`                             |
| `CARB_INCLUDE_DIR` | Generating `carb-sys` bindings                   | Override for Carb headers location              | `crates/carb-sys/build.rs`                          |

`ISAAC_SIM` and `ISAAC_SIM_PATH` are both used in the source-build surface: shell scripts resolve `ISAAC_SIM`; CMake reads `ISAAC_SIM_PATH`. Set both to the same value or let the justfile propagate them.

## Runtime adapter discovery (deprecated; prefer extension settings)

The C++ extension formerly used these env vars to dlopen Rust adapter cdylibs. Prefer the `[settings.omni.isaacsimrs.bridge].adapters` block in `extension.toml` — these are kept only as source-build overrides:

| Variable                    | Effect                                                         | Status     |
| --------------------------- | -------------------------------------------------------------- | ---------- |
| `ISAAC_SIM_RS_DORA_RUNNER`  | Absolute path to `libisaac_sim_dora.so` to dlopen              | Deprecated |
| `ISAAC_SIM_RS_RERUN_RUNNER` | Absolute path to `libisaac_sim_rerun.so` to dlopen             | Deprecated |

When set, the extension logs a deprecation warning and uses the env-var path. When unset, the extension reads `[settings.omni.isaacsimrs.bridge].adapters`, resolves each name against `adapter_path` (or the plugin's own directory if empty), and dlopens `libisaac_sim_<name>.so`.

Preferred replacement for source builds:

```bash
$ISAAC_SIM/isaac-sim.sh \
    --enable omni.isaacsimrs.bridge \
    --/settings/omni.isaacsimrs.bridge/adapters="dora,rerun"
```

Or edit `cpp/omni.isaacsimrs.bridge/config/extension.toml`:

```toml
[settings.omni.isaacsimrs.bridge]
adapters = ["dora"]
adapter_path = ""   # empty = directory next to the plugin .so
```

## Per-dataflow routing (Kit + dora users)

These are runtime config for individual dora dataflows. They can be set by the launching script, the `dataflow.yml`, or the user's shell.

| Variable                                    | Default              | Effect                                                              |
| ------------------------------------------- | -------------------- | ------------------------------------------------------------------- |
| `ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_SOURCE`   | (empty = any source) | Prim-path filter for the FlatScan publisher                         |
| `ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT`   | `lidar_flatscan`     | dora node output id for the FlatScan record-batch                   |
| `ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_SOURCE` | (empty = any source) | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_OUTPUT` | `lidar_pointcloud`   | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_CAMERA_RGB_SOURCE`       | (empty = any source) | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_CAMERA_RGB_OUTPUT`       | `camera_rgb`         | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_CAMERA_DEPTH_*`          |                      | (same shape — `_SOURCE` / `_OUTPUT`)                                |
| `ISAAC_SIM_RS_DORA_CAMERA_INFO_*`           |                      | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_IMU_*`                   |                      | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_ODOMETRY_*`              |                      | (same shape)                                                        |
| `ISAAC_SIM_RS_DORA_CMD_VEL_OUTPUT`          | `cmd_vel_observed`   | dora output id for the cmd_vel observer (publisher direction)        |
| `ISAAC_SIM_RS_DORA_CMD_VEL_INPUT`           | (unset)              | dora input id the bridge subscribes to (subscriber direction)        |
| `ISAAC_SIM_RS_DORA_CMD_VEL_TARGET`          | (unset)              | Producer-slot key the C++ apply node polls                           |

**cmd_vel asymmetry note**: `CMD_VEL_OUTPUT` uses a default of `cmd_vel_observed` (not `cmd_vel`) to avoid dataflow self-loops when both the publisher and subscriber are active in the same node. `CMD_VEL_INPUT` and `CMD_VEL_TARGET` have no default — if either is unset or empty the subscriber does not start. This asymmetry is a known rough edge tracked in the runtime audit.

For the exact env-var name per channel, search the source:

```bash
git grep ISAAC_SIM_RS_DORA crates/isaac-sim-dora/src/ffi.rs
```

The `examples/lidar-receiver/launch-kit.sh` and `examples/nova-carter/launch.sh` scripts set these for those specific dataflows; treat them as templates.

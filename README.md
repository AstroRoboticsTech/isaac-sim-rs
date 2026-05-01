# isaac-sim-rs

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/MPL/2.0/)
[![Isaac Sim](https://img.shields.io/badge/Isaac%20Sim-5.1-76B900.svg)](https://developer.nvidia.com/isaac/sim)
[![dora-rs](https://img.shields.io/badge/dora--rs-0.5-blue.svg)](https://github.com/dora-rs/dora)
[![Rust](https://img.shields.io/badge/rustc-1.85+-orange.svg)](https://www.rust-lang.org/)

Unofficial Rust SDK for [NVIDIA Isaac Sim](https://developer.nvidia.com/isaac/sim) & [Omniverse](https://developer.nvidia.com/omniverse). Bridges Isaac Sim's C++ Carbonite plugin surface and OmniGraph runtime into safe Rust callbacks, with **transport-agnostic** consumer adapters — your data goes to [`dora-rs`](https://github.com/dora-rs/dora), [`rerun`](https://rerun.io), a file logger, or any custom bus you wire up, **without** a Python or ROS hop in the hot path.

Maintained by [Astro Robotics](https://github.com/AstroRoboticsTech). MPL-2.0.

## Why

Isaac Sim ships a Python API and a ROS 2 bridge. Both add a serialization hop on every sensor frame and constrain the runtime your robotics stack can use. `isaac-sim-rs` skips both: a custom Carb C++ extension `dlopen`s a Rust `cdylib`, sensor buffers cross via [`cxx::bridge`](https://cxx.rs), and downstream consumers register Rust callbacks. Native C++ ↔ Rust handoff, no language jump, no IPC marshalling tax.

## Architecture

```mermaid
flowchart LR
    Kit["Isaac Sim<br/>+ omni.isaacsimrs.bridge<br/>(C++ Carb plugin)"]
    Bridge["isaac-sim-bridge<br/>(Rust cdylib)<br/>consumer registry"]
    Arrow["isaac-sim-arrow<br/>(Apache Arrow batches)"]
    Dora["isaac-sim-dora<br/>(dora-rs adapter)"]
    Other["isaac-sim-rerun, ...<br/>(future adapters)"]
    Down["downstream consumers<br/>(dora dataflows, rerun viewer, ...)"]

    Kit -->|cxx::bridge| Bridge
    Bridge -->|register_*_consumer| Arrow
    Bridge -->|register_*_consumer| Dora
    Bridge -.->|register_*_consumer| Other
    Dora -->|Arrow RecordBatch| Down
    Other -.-> Down
```

The core (`isaac-sim-bridge`) has zero adapter dependencies. Adapters (`isaac-sim-dora`, future `isaac-sim-rerun`, etc.) depend on the core, never the other way around.

## What works today

- `omni.isaacsimrs.bridge` Carb extension loads in Isaac Sim 5.1 with full OmniGraph + USD + GSL toolchain (CMake build, packman-fetched USD).
- Custom OmniGraph nodes authored in C++ via the standard `.ogn` codegen, registered eagerly so they're visible in OG graphs the moment the extension loads.
- Per-sensor `OgnPublish*ToRust` nodes accept the matching NVIDIA RTX / Replicator / Isaac annotator output and forward payload + metadata to Rust via `cxx::bridge`. The reverse-direction `OgnApplyCmdVelFromRust` polls a Rust producer slot each tick and emits scalar lin/ang velocities into the Isaac differential controller.
- `isaac-sim-bridge` exposes a thread-safe consumer registry plus a producer registry (cmd_vel): any Rust closure for a sensor can be registered and gets dispatched on every frame; any actuation source can call `register_cmd_vel_producer(target).publish(...)` and the C++ tick reads it.
- `isaac-sim-arrow` converts every bridged sensor (LiDAR FlatScan + PointCloud, Camera RGB + Depth + Info, IMU, Odometry, cmd_vel) to an Apache Arrow `RecordBatch` with a stable schema per sensor.
- `isaac-sim-dora` ships publisher adapters for every sensor and a bidirectional cmd_vel adapter, plus convenience subscribe decoders (`subscribe::*`) so downstream dora algorithm nodes can pull native owned structs out of inbound `ArrayRef`s without touching `arrow` directly. As a `cdylib` the bridge `dlopen`s it via `ISAAC_SIM_RS_DORA_RUNNER` so Kit becomes a dora node with no extra extension code; as an `rlib` downstream Rust crates use the helper API directly.
- `isaac-sim-rerun` wraps the same consumer registry behind a `Viewer` builder: one `with_source(SensorMarker, prim, entity_path)` call per stream, optional cross-host gRPC.
- `examples/lidar-receiver/` ships a synthesized-input dora pipeline. `examples/nova-carter/` is the full Nova Carter showcase: 2D + 3D LiDAR, RGB + depth + camera-info, IMU, chassis odometry, plus a synthetic cmd_vel publisher driving the robot through a warehouse.

## Adapter coverage

Every bridged sensor currently has Arrow + dora + rerun coverage. Adding a new sensor is two files plus the trait impls, so the matrix is one source of truth:

| Channel             | Bridge | Arrow | Dora pub | Dora sub | Rerun |
| ------------------- | :----: | :---: | :------: | :------: | :---: |
| LiDAR FlatScan      | x      | x     | x        | x        | x     |
| LiDAR PointCloud    | x      | x     | x        | x        | x     |
| Camera RGB          | x      | x     | x        | x        | x     |
| Camera Depth        | x      | x     | x        | x        | x     |
| Camera Info         | x      | x     | x        | x        | x     |
| IMU                 | x      | x     | x        | x        | x     |
| Chassis Odometry    | x      | x     | x        | x        | x     |
| cmd_vel (Twist)     | x      | x     | x        | x        | x     |

**Dora pub** — bridge fans out the Arrow `RecordBatch` on a dora node output. **Dora sub** — convenience decoders (`isaac_sim_dora::subscribe::*`) on the consumer side: a downstream dora algorithm node wires its input to the bridge's output and calls `subscribe::lidar_pointcloud(&data.0)?` (etc.) to get an owned native struct it can run perception / state estimation / control over. **cmd_vel sub** in addition closes the loop into the bridge: an upstream dora node publishes a `Twist`, the bridge decodes it and republishes into the producer slot the C++ apply node polls. **Rerun** opens a `RecordingStream` per channel and pushes payloads as the rerun-native primitive (`Points3D`, `Image`, `Pinhole`, `Scalars`, `Transform3D`).

## Quick start

```bash
# 1. clone
git clone https://github.com/AstroRoboticsTech/isaac-sim-rs.git
cd isaac-sim-rs

# 2. point at your Isaac Sim install
export ISAAC_SIM=/path/to/isaac-sim
export ISAAC_SIM_RS=$(pwd)

# 3. build (cmake drives cargo for every workspace cdylib via a
#    custom target; CMake fetches USD via NVIDIA packman the first
#    time, ~3.8 GB)
ISAAC_SIM_PATH=$ISAAC_SIM CARGO_PROFILE=release just build

# 4. run the example dora pipeline
cd $ISAAC_SIM_RS/examples/lidar-receiver
dora up
dora build dataflow.yml
dora start dataflow.yml --detach

# 5. watch the receiver
RUN=$(dora list | awk '/Running/ {print $1}')
dora logs $RUN receiver
# [receiver] scan: n=360 fov=360.0° rate=10.0Hz depth=[3.000,7.000]m
# (repeating at 10 Hz)
```

See [`examples/lidar-receiver/README.md`](examples/lidar-receiver/README.md) for the full walkthrough.

The full set of public recipes is `just --list` (workspace tests, clippy, fmt, link-smoke, kit-smoke, clean). Per-developer cross-host helpers go in `justfile.local` (gitignored).

## Crates

| Crate                                          | Purpose                                                                                                       |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| [`carb-sys`](crates/carb-sys/)                 | Raw FFI bindings to the NVIDIA Carbonite SDK (bindgen, env-driven build)                                      |
| [`isaac-sim-bridge`](crates/isaac-sim-bridge/) | C++ ↔ Rust bridge cdylib + consumer registry + cmd_vel producer registry. The hub everything else plugs into. |
| [`isaac-sim-arrow`](crates/isaac-sim-arrow/)   | Apache Arrow conversion utilities for every bridged sensor + cmd_vel. Consumer-agnostic.                      |
| [`isaac-sim-dora`](crates/isaac-sim-dora/)     | dora-rs publisher adapters for every sensor + cmd_vel subscriber; rlib + cdylib                               |
| [`isaac-sim-rerun`](crates/isaac-sim-rerun/)   | rerun viewer adapter: per-sensor `RecordingStream`, gRPC client, builder API                                  |

## Examples

| Example                                      | Demonstrates                                                                                                                                                  |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`lidar-receiver`](examples/lidar-receiver/) | Kit-as-dora-source + receiver dora node; full end-to-end pipeline                                                                                             |
| [`nova-carter`](examples/nova-carter/)       | Nova Carter in a warehouse: 2D + 3D LiDAR, RGB + depth + camera-info, IMU, chassis odometry, cmd_vel apply chain; streams to a rerun viewer over gRPC        |

Each example lives in its own self-contained `examples/<name>/` directory.

## Compatibility

|              | Tested on                                             |
| ------------ | ----------------------------------------------------- |
| Isaac Sim    | 5.1.0-rc.19 (Linux x86_64)                            |
| GPU / CUDA   | NVIDIA RTX 4090, CUDA 12.6, driver 550.x              |
| OS           | Ubuntu 24.04 (other modern Linux distros should work) |
| Compiler     | gcc 13.3, CMake 3.28                                  |
| Rust         | rustc 1.85+ (workspace `rust-version = "1.85"`)       |
| dora-rs      | 0.5 (dora-cli, dora-node-api 0.5)                     |
| Apache Arrow | 54 (workspace-pinned to match dora)                   |

The C++ plugin is Linux-only (Isaac Sim runs only on Linux/Windows). The Rust crates compile on macOS for development — `cargo check`, `cargo test` for the pure-Rust crates work locally; the bridge cdylib needs the Carb headers from a real Isaac Sim install.

**One-time build cost (Linux): the first `just build` fetches USD via NVIDIA's packman (~3.8 GB extracted into `cpp/omni.isaacsimrs.bridge/build/deps/`).** USD is needed only for headers — OmniGraph's `omni/graph/core/StringUtils.h` includes `pxr/base/tf/token.h` transitively, so any cpp file pulled into the plugin via the OGN-generated `*Database.h` chain ends up parsing pxr headers at compile time. The plugin's `.so` does **not** link against any `libusd_*` / `libpxr_*` (verified via `readelf -d`). The fetch is cached after the first run; subsequent builds skip it.

## License

[Mozilla Public License 2.0](LICENSE). File-level copyleft: use this SDK in any project (commercial, proprietary, open-source). Modifications to source files in this repository must be released under MPL-2.0. See [LICENSE](LICENSE) for full terms.

## Prior art

This SDK builds on patterns first explored by:

- [`AndrejOrsula/omniverse_rs`](https://github.com/AndrejOrsula/omniverse_rs) — autocxx-based Omniverse interface (dormant since 2024)
- [`AndrejOrsula/isaac_sim_rs`](https://github.com/AndrejOrsula/isaac_sim_rs) — Rust interface for Isaac Sim (dormant since 2024)
- [`AndrejOrsula/pxr_rs`](https://github.com/AndrejOrsula/pxr_rs) — autocxx-based OpenUSD bindings

We may vendor `pxr_rs` for USD support rather than reimplement.

## Related

- [NVIDIA Isaac Sim](https://github.com/isaac-sim/IsaacSim) — open-source RTX sensor sources we wire into via sibling OmniGraph nodes
- [dora-rs](https://github.com/dora-rs/dora) — low-latency dataflow runtime; first-class consumer
- [rerun](https://github.com/rerun-io/rerun) — interactive 3D viewer; first-class consumer adapter (`crates/isaac-sim-rerun`)
- [Apache Arrow](https://arrow.apache.org) — universal columnar interchange format used by all our consumer adapters

## Contributing

Pull requests welcome. The repo is in early development and APIs may change. CONTRIBUTING.md and a CLA setup are coming.

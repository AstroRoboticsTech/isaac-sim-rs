# isaac-sim-bridge

[![Crates.io](https://img.shields.io/crates/v/isaac-sim-bridge.svg)](https://crates.io/crates/isaac-sim-bridge)
[![Documentation](https://docs.rs/isaac-sim-bridge/badge.svg)](https://docs.rs/isaac-sim-bridge)
[![License](https://img.shields.io/badge/license-MPL--2.0-brightgreen.svg)](https://www.mozilla.org/MPL/2.0/)

The `cxx::bridge` core of the Isaac Sim Rust SDK. A Carb C++ plugin (`omni.isaacsimrs.bridge`) `dlopen`s this crate's cdylib and calls exported bridge entry points that forward sensor buffers into a process-wide consumer registry. Any number of Rust adapters (dora-rs, rerun, custom loggers) register callbacks and receive the forwarded frames on the bridge thread.

**rlib (default)**: links into any downstream without requiring Isaac Sim at compile time. The consumer registration API, channel markers, producer registry, and `SourceFilter` are all accessible in pure Rust without a C++ toolchain.

**cdylib (opt-in feature `cdylib`)**: adds the `cxx::bridge` shim that the C++ extension calls at runtime. Requires `ISAAC_SIM` pointing at a real Isaac Sim install and a compatible C++ toolchain at link time.

## Usage

```toml
[dependencies]
# rlib path — no Isaac Sim required
isaac-sim-bridge = "0.1"

# cdylib path — only for the Kit extension build
isaac-sim-bridge = { version = "0.1", features = ["cdylib"] }
```

```rust
use isaac_sim_bridge::{register_lidar_flatscan_consumer, LidarFlatScan};

// Register a callback — runs on the bridge thread each time a FlatScan
// frame arrives from the C++ extension. Keep the closure bounded.
register_lidar_flatscan_consumer(|source_id, depths, intensities, meta| {
    println!(
        "{}: {} rays, fov={:.1}°",
        source_id,
        depths.len(),
        meta.horizontal_fov
    );
});
```

Part of the [`isaac-sim-rs`](https://github.com/AstroRoboticsTech/isaac-sim-rs) SDK. The facade crate re-exports this as the `bridge` namespace when the `bridge` or `dora` or `rerun` feature is enabled.

## Compatibility

| Dependency  | Version        |
| ----------- | -------------- |
| Isaac Sim   | 5.1            |
| cxx         | 1              |
| Rust        | 1.85+          |

The rlib compiles on any platform. The cdylib requires Linux x86_64 with Isaac Sim 5.1 installed and a C++17 toolchain (gcc 13.3 tested).

## License notice for re-publishers

MPL-2.0 is per-file copyleft. If you bundle this crate's binary into your own crate or extension, retain the SPDX header on every source file you include. The full license text is in [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE) at the repo root.

## License

MPL-2.0 (see [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE)).

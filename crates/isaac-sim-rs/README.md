# isaac-sim-rs

[![Crates.io](https://img.shields.io/crates/v/isaac-sim-rs.svg)](https://crates.io/crates/isaac-sim-rs)
[![Documentation](https://docs.rs/isaac-sim-rs/badge.svg)](https://docs.rs/isaac-sim-rs)
[![License](https://img.shields.io/badge/license-MPL--2.0-brightgreen.svg)](https://www.mozilla.org/MPL/2.0/)

Unofficial Rust SDK for [NVIDIA Isaac Sim](https://developer.nvidia.com/isaac/sim) & Omniverse. Bridges Isaac Sim's C++ Carbonite plugin surface into safe Rust callbacks, with transport-agnostic consumer adapters — your data goes to dora-rs, rerun, a file logger, or any custom bus you wire up, without a Python or ROS hop in the hot path.

Adapter selection happens at `cargo add` time via feature flags. The default feature (`arrow`) is pure Rust and compiles on any machine without Isaac Sim installed.

## Feature matrix

| Feature   | Pulls in                                        | Requires Isaac Sim at link time |
| --------- | ----------------------------------------------- | :-----------------------------: |
| `arrow`   | pure-Rust Arrow schema + decoders (default)     | No                              |
| `bridge`  | bridge rlib + Arrow                             | No (rlib only)                  |
| `dora`    | bridge + dora publisher + subscriber decoders   | No (rlib only)                  |
| `rerun`   | bridge + rerun Viewer builder                   | No (rlib only)                  |
| `full`    | bridge + dora + rerun                           | No (rlib only)                  |

The C++ extension build passes `--features isaac-sim-bridge/cdylib` separately to produce the `.so` the Kit extension `dlopen`s.

## Usage

```bash
# schema + decoders only (no Isaac Sim required)
cargo add isaac-sim-rs

# dora subscriber decoders
cargo add isaac-sim-rs -F dora

# rerun viewer adapter
cargo add isaac-sim-rs -F rerun

# both adapters in one build
cargo add isaac-sim-rs -F full
```

```rust
// default features: Arrow schema + decoders, pure Rust
use isaac_sim_rs::arrow::lidar::flatscan::{LidarFlatScan, to_record_batch};

let depths = [0.5_f32, 1.0, 1.5, 2.0];
let intensities = [10_u8, 50, 200, 100];
let scan = LidarFlatScan {
    depths: &depths,
    intensities: &intensities,
    horizontal_fov: 270.0,
    horizontal_resolution: 0.25,
    azimuth_min: -135.0,
    azimuth_max: 135.0,
    depth_min: 0.1,
    depth_max: 30.0,
    num_rows: 1,
    num_cols: 4,
    rotation_rate: 10.0,
};
let batch = to_record_batch(&scan)?;
assert_eq!(batch.num_rows(), 1);
# Ok::<(), arrow::error::ArrowError>(())
```

See the [workspace README](https://github.com/AstroRoboticsTech/isaac-sim-rs) for the full architecture, examples, and source-build instructions.

## Compatibility

| Dependency    | Version |
| ------------- | ------- |
| Isaac Sim     | 5.1     |
| Apache Arrow  | 54      |
| dora-node-api | 0.5 (when `dora` feature enabled) |
| rerun         | 0.31 (when `rerun` feature enabled) |
| Rust          | 1.85+   |

## License notice for re-publishers

MPL-2.0 is per-file copyleft. If you bundle this crate's binary into your own crate or extension, retain the SPDX header on every source file you include. The full license text is in [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE) at the repo root.

## License

MPL-2.0 (see [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE)).

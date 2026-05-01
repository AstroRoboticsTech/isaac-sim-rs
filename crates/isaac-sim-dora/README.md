# isaac-sim-dora

[![Crates.io](https://img.shields.io/crates/v/isaac-sim-dora.svg)](https://crates.io/crates/isaac-sim-dora)
[![Documentation](https://docs.rs/isaac-sim-dora/badge.svg)](https://docs.rs/isaac-sim-dora)
[![License](https://img.shields.io/badge/license-MPL--2.0-brightgreen.svg)](https://www.mozilla.org/MPL/2.0/)

Dora-rs publisher and subscriber adapter for the Isaac Sim Rust SDK. On the Kit side, this crate's cdylib registers bridge consumers that convert every sensor frame to an Apache Arrow `RecordBatch` and publish it on a dora node output. On the receiver side, `subscribe::*` convenience decoders turn inbound `ArrayRef` values back into owned native structs — no direct arrow dependency needed in algorithm nodes.

## Usage

```toml
[dependencies]
isaac-sim-dora = "0.1"
```

```rust
use dora_node_api::{DoraNode, Event};
use isaac_sim_dora::subscribe;

let (_node, mut events) = DoraNode::init_from_env()?;
while let Some(event) = events.recv() {
    match event {
        Event::Input { id, data, .. } if id.as_str() == "lidar_flatscan" => {
            let scan = subscribe::lidar_flatscan(&data.0)?;
            // scan.depths: Vec<f32>, scan.horizontal_fov: f32, etc.
        }
        Event::Input { id, data, .. } if id.as_str() == "imu" => {
            let imu = subscribe::imu(&data.0)?;
        }
        Event::Stop(_) => break,
        _ => {}
    }
}
# Ok::<(), eyre::Report>(())
```

Part of the [`isaac-sim-rs`](https://github.com/AstroRoboticsTech/isaac-sim-rs) SDK. The facade crate re-exports this as the `dora` namespace when the `dora` feature is enabled.

## Compatibility

| isaac-sim-dora | dora-node-api | Rust MSRV |
| -------------- | ------------- | --------- |
| 0.1            | 0.5           | 1.85      |

dora-node-api is pre-1.0; major bump in lockstep.

dora-node-api 0.5 is pre-1.0; minor bumps may be breaking. The Arrow major is pinned to match dora's internal requirement.

## License notice for re-publishers

MPL-2.0 is per-file copyleft. If you bundle this crate's binary into your own crate or extension, retain the SPDX header on every source file you include. The full license text is in [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE) at the repo root.

## License

MPL-2.0 (see [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE)).

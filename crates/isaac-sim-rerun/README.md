# isaac-sim-rerun

[![Crates.io](https://img.shields.io/crates/v/isaac-sim-rerun.svg)](https://crates.io/crates/isaac-sim-rerun)
[![Documentation](https://docs.rs/isaac-sim-rerun/badge.svg)](https://docs.rs/isaac-sim-rerun)
[![License](https://img.shields.io/badge/license-MPL--2.0-brightgreen.svg)](https://www.mozilla.org/MPL/2.0/)

Rerun viewer adapter for the Isaac Sim Rust SDK. Exposes a `Viewer` builder that registers bridge consumers forwarding sensor frames to a rerun `RecordingStream` over gRPC. Each sensor stream opens its own gRPC connection so a high-bandwidth camera channel cannot backpressure a low-bandwidth LiDAR stream. All streams share one `recording_id` so the rerun viewer renders them on a single timeline.

## Usage

```toml
[dependencies]
isaac-sim-rerun = "0.1"
```

```rust,no_run
use isaac_sim_rerun::Viewer;
use isaac_sim_bridge::{LidarFlatScan, LidarPointCloud};

Viewer::new()
    .with_grpc_addr("192.168.1.10:9876")
    .with_source(LidarFlatScan, "/World/Carter/lidar_2d", "scene/lidar/flatscan")
    .with_source(LidarPointCloud, "/World/Carter/lidar_3d", "scene/lidar/pointcloud")
    .run()?;
# Ok::<(), eyre::Report>(())
```

`Viewer::run()` blocks until the process exits. The default gRPC address is `127.0.0.1:9876`; override via `with_grpc_addr` or the `ISAAC_SIM_RS_RERUN_GRPC_ADDR` env var.

Part of the [`isaac-sim-rs`](https://github.com/AstroRoboticsTech/isaac-sim-rs) SDK. The facade crate re-exports this as the `rerun` namespace when the `rerun` feature is enabled.

## Compatibility

| isaac-sim-rerun | rerun | Rust MSRV |
| --------------- | ----- | --------- |
| 0.1             | 0.31  | 1.85      |

rerun is pre-1.0; every minor bump is a breaking change. Major bump in lockstep.

rerun 0.31 is pre-1.0; minor bumps are breaking. Pin your `rerun` dependency to the same major to ensure `RecordingStream` compatibility.

## License notice for re-publishers

MPL-2.0 is per-file copyleft. If you bundle this crate's binary into your own crate or extension, retain the SPDX header on every source file you include. The full license text is in [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE) at the repo root.

## License

MPL-2.0 (see [`LICENSE`](https://github.com/AstroRoboticsTech/isaac-sim-rs/blob/main/LICENSE)).

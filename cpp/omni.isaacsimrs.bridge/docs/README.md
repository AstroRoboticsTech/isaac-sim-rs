# omni.isaacsimrs.bridge

Carb extension that forwards Isaac Sim sensor and actuator pipelines into a Rust runtime, exposed to dora-rs, rerun, or custom adapters via cargo features.

## What it does

Wires NVIDIA RTX sensors (LiDAR FlatScan + PointCloud, Camera RGB + Depth + Info, IMU, Odometry) and a `cmd_vel` actuation chain through OmniGraph nodes that publish into a Rust cdylib. Adapters dispatch the data downstream without a Python or ROS hop in the hot path.

## Installation

1. Install via the Isaac Sim extension manager OR drop the prebuilt tarball into `~/Documents/Kit/.../exts/omni.isaacsimrs.bridge/`.
2. Enable in the extension manager.
3. Configure adapters (default `["dora"]`):

   ```toml
   [settings.omni.isaacsimrs.bridge]
   adapters = ["dora", "rerun"]
   ```

## OmniGraph nodes

The extension registers eight `OgnPublish*ToRust` nodes plus `OgnApplyCmdVelFromRust`. Wire them to NVIDIA RTX sensor outputs in your OG graph; the bridge forwards each tick.

## Compatibility

See `docs/COMPAT.md` for the supported Isaac Sim / Kit version matrix.

## License

MPL-2.0. See [`LICENSE.md`](LICENSE.md).

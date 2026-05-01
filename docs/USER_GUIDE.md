# User guide — Isaac Sim user (extension manager)

You have Isaac Sim installed. You want to plug a Rust dataflow (dora) or remote viewer (rerun) into your simulation. This is the path.

> Writing a downstream Rust crate that depends on `isaac-sim-rs`? See [`INTEGRATING.md`](INTEGRATING.md) for the cargo consumer track.

## 1. Install the extension

Option A: extension manager — search "isaacsimrs" once the extension is published to the NVIDIA registry and click Install.

Option B: tarball — extract the prebuilt package into your Kit extensions folder:

```bash
tar -xzf omni.isaacsimrs.bridge-0.1.0-linux-x86_64.tar.gz \
    -C ~/Documents/Kit/<version>/exts/
```

## 2. Enable the extension

In Isaac Sim: Window → Extensions → search "isaacsimrs" → Enable.

Verify in the Console tab that the adapter loaded:

```
[omni.isaacsimrs.bridge] adapter "dora" loaded from ${EXT_PATH}/bin/linux-x86_64/libisaac_sim_dora.so
```

## 3. Configure adapters (optional)

The default adapter set is `["dora"]`. To enable both dora and rerun, add to your `user.toml` (or any Kit config layer):

```toml
[settings.omni.isaacsimrs.bridge]
adapters = ["dora", "rerun"]
```

Override at launch without editing any file:

```bash
$ISAAC_SIM/isaac-sim.sh \
    --enable omni.isaacsimrs.bridge \
    --/settings/omni.isaacsimrs.bridge/adapters="dora,rerun"
```

## 4. Author your OmniGraph

In the Action Graph editor, search for "isaacsimrs" to find the published node types:

- `PublishLidarFlatScanToRust` — 2D RTX LiDAR flat scan
- `PublishLidarPointCloudToRust` — 3D point cloud
- `PublishCameraRgbToRust` — RGB image
- `PublishCameraDepthToRust` — depth image
- `PublishCameraInfoToRust` — camera intrinsics
- `PublishImuToRust` — IMU linear acceleration + angular velocity
- `PublishOdometryToRust` — odometry pose + twist
- `ApplyCmdVelFromRust` — reverse direction: polls a Rust producer slot and emits lin/ang velocity

Wire each `PublishXxx` node's inputs to the matching NVIDIA RTX sensor output (e.g., `IsaacComputeRTXLidarFlatScan` for the flat-scan path, `IsaacConvertRGBAToRGB` annotator for RGB). Set the `sourceId` attribute on each node to a stable identifier (typically the sensor prim path); downstream dora or rerun consumers use this to route data by source.

## 5. Run a dora dataflow

```bash
cd /path/to/your/dataflow
dora up
dora build dataflow.yml
dora start dataflow.yml --detach
```

The Kit extension acts as an upstream dora node. Downstream dora nodes subscribe to its outputs (e.g., `lidar_flatscan`, `camera_rgb`). The `sourceId` you set on each OG node appears as a field in the Arrow record batch so subscribers can filter by sensor origin.

## 6. Reverse-direction actuation (cmd_vel)

For closed-loop control: a downstream dora node publishes a Twist on the `cmd_vel` topic; the bridge subscribes and writes into the producer slot. The `ApplyCmdVelFromRust` OG node polls the slot each tick and emits scalar linear and angular velocities. Wire those outputs to `IsaacDifferentialController.inputs:linearVelocity` and `IsaacDifferentialController.inputs:angularVelocity`, then connect the controller to `IsaacArticulationController` targeting your robot's articulation root.

On a poll miss (no producer registered yet, or no value published) the node emits zero on both outputs, so the robot stops cleanly rather than holding the last command.

## Troubleshooting

- **"Adapter not found"** in the Console: check that `${EXT_PATH}/bin/linux-x86_64/` contains `libisaac_sim_<adapter>.so`. If you placed the adapter library at a custom path, set `adapter_path` in the settings block.
- **Sensors silent**: confirm the `sourceId` attribute on the OG node matches what your dora subscriber's `SourceFilter` expects.
- **Tarball RPATH issues**: run `just verify-rpath` after `just package-extension` to confirm all bundled libraries resolve via `$ORIGIN`.
- See [`docs/ENV_VARS.md`](ENV_VARS.md) for the full env-var reference.

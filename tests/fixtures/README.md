# Annotator Fixtures

Captured snapshots of NVIDIA RTX-sensor annotator outputs for tier-(a)
schema-conformance tests. Each fixture is a pair:

- `<annotator>_<isaac_version>.bin` — raw little-endian dump of the
  annotator's output buffer (e.g. packed XYZ for the PointCloud
  variants, or per-channel pointers + sizes for the FlatScan buffer).
- `<annotator>_<isaac_version>.meta.json` — recording-time schema:
  field names, dtypes, byte stride, host vs CUDA flag, and the Isaac
  Sim version that produced it.

The matching `crates/isaac-sim-bridge/tests/fixture_replay.rs` test
walks this directory, parses each `.meta.json`, mmaps the `.bin`, and
replays through `forward_lidar_pointcloud` / `forward_lidar_flatscan`
asserting the dispatch chain accepts the captured shape. This catches
the d7a102a bug class — schema drift between what NVIDIA's annotator
emits and what our publish nodes expect — without needing a Kit run
in CI.

When no fixtures are present the replay test simply passes (the
schema validator in `tests/ogn_schema.rs` covers the static side).

## Refreshing

On the Linux workstation with Isaac Sim installed:

```bash
$ISAAC_SIM/kit/kit \
    $ISAAC_SIM/apps/isaacsim.exp.full.kit \
    --no-window --no-ros-env \
    --ext-folder $ISAAC_SIM_RS/cpp \
    --enable omni.isaacsimrs.bridge \
    --enable isaacsim.sensors.rtx \
    --exec scripts/capture_fixture.py
```

`scripts/capture_fixture.py` writes a fresh pair into this directory
named with the running Isaac Sim version. Commit the resulting files.

## Schema drift

When the Isaac Sim version field in `.meta.json` differs from the
running install, `tests/fixture_drift.rs` warns rather than fails:
the fixture is still useful (replay should still pass for any
schema-compatible version) but a re-capture is recommended.

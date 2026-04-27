"""Capture a single RTX-LiDAR annotator output to a fixture pair on
disk for replay in tier-(a) schema-conformance tests.

Run from the workstation with Isaac Sim installed:

    $ISAAC_SIM/kit/kit \\
        $ISAAC_SIM/apps/isaacsim.exp.full.kit \\
        --no-window --no-ros-env \\
        --ext-folder $ISAAC_SIM_RS/cpp \\
        --enable omni.isaacsimrs.bridge \\
        --enable isaacsim.sensors.rtx \\
        --exec scripts/capture_fixture.py

Produces under `tests/fixtures/`:

    lidar_pointcloud_isaac_<version>.bin
    lidar_pointcloud_isaac_<version>.meta.json

Repeat for FlatScan after wiring the corresponding annotator-attach
path. The schema-conformance test then replays the bin through
`forward_lidar_pointcloud` and verifies the dispatch chain accepts
this exact shape.
"""

import json
import os
import struct
from pathlib import Path

import omni.graph.core as og
import omni.replicator.core as rep
import omni.timeline
import omni.usd
from isaacsim.core.api import World
from isaacsim.core.nodes.scripts.utils import register_node_writer_with_telemetry
from isaacsim.core.utils.stage import add_reference_to_stage, open_stage
from isaacsim.sensors.rtx import LidarRtx

ASSETS_ROOT = "https://omniverse-content-production.s3-us-west-2.amazonaws.com/Assets/Isaac/5.1"
WAREHOUSE_USD = f"{ASSETS_ROOT}/Isaac/Environments/Simple_Warehouse/warehouse.usd"
CARTER_USD = f"{ASSETS_ROOT}/Isaac/Robots/NVIDIA/NovaCarter/nova_carter.usd"

CARTER_PRIM = "/Root/World/Carter"
LIDAR_3D_PRIM = f"{CARTER_PRIM}/chassis_link/sensors/XT_32/PandarXT_32_10hz"

REPO_ROOT = Path(__file__).resolve().parent.parent
FIXTURE_DIR = REPO_ROOT / "tests" / "fixtures"


def isaac_version() -> str:
    try:
        from isaacsim.core.version import get_version

        version = get_version()
        return f"{version.major}.{version.minor}.{version.patch}"
    except Exception:
        return "unknown"


def capture_pointcloud():
    open_stage(WAREHOUSE_USD)
    add_reference_to_stage(CARTER_USD, CARTER_PRIM)

    world = World(stage_units_in_meters=1.0)
    lidar_3d = world.scene.add(
        LidarRtx(prim_path=LIDAR_3D_PRIM, name="fixture_lidar_3d")
    )
    world.reset()

    # The IsaacCreateRTXLidarScanBuffer annotator (defaults — no
    # enablePerFrameOutput, so it accumulates a full rotation) emits
    # the same packed XYZ shape the bridge consumes today.
    annotator = rep.AnnotatorRegistry.get_annotator("IsaacCreateRTXLidarScanBuffer")
    annotator.attach([lidar_3d.get_render_product_path()])

    omni.timeline.get_timeline_interface().play()

    captured = {"data": None, "meta": None}

    def on_frame(_e=None):
        if captured["data"] is not None:
            return
        out = annotator.get_data()
        if out is None:
            return
        # The annotator returns numpy on host. Capture the raw bytes
        # plus enough schema to replay through `forward_lidar_pointcloud`.
        if hasattr(out, "tobytes"):
            captured["data"] = out.tobytes()
            captured["meta"] = {
                "annotator": "IsaacCreateRTXLidarScanBuffer",
                "isaac_version": isaac_version(),
                "dtype": str(out.dtype),
                "shape": list(out.shape),
                "byte_stride": int(out.dtype.itemsize),
                "host_resident": True,
                "channels": 3,
                "captured_at": "drive",
            }

    # Run a few simulation ticks to let the annotator produce.
    import time

    for _ in range(120):
        if captured["data"] is not None:
            break
        og.Controller.evaluate_sync()
        time.sleep(0.05)

    omni.timeline.get_timeline_interface().stop()

    if captured["data"] is None:
        print("[capture_fixture] failed to capture annotator output")
        return False

    FIXTURE_DIR.mkdir(parents=True, exist_ok=True)
    name = f"lidar_pointcloud_isaac_{captured['meta']['isaac_version']}"
    bin_path = FIXTURE_DIR / f"{name}.bin"
    meta_path = FIXTURE_DIR / f"{name}.meta.json"

    bin_path.write_bytes(captured["data"])
    meta_path.write_text(json.dumps(captured["meta"], indent=2) + "\n")
    print(f"[capture_fixture] wrote {bin_path} ({len(captured['data'])} bytes)")
    print(f"[capture_fixture] wrote {meta_path}")
    return True


if __name__ == "__main__":
    if not capture_pointcloud():
        os._exit(1)

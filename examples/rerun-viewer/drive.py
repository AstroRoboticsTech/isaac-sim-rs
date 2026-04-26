"""Drive PublishLidarFlatScanToRust from a real RTX LiDAR ray-casting against
the Simple_Warehouse scene with a NovaCarter robot. Both assets are
fetched at runtime from the public Isaac Sim S3 bucket (the same URL
Kit's `/persistent/isaac/asset_root/default` setting points at).

Replaces the synthetic drive: the bridge now sees real depths that
vary with azimuth as the warehouse geometry occludes rays.
"""

import omni.graph.core as og
import omni.timeline
import omni.usd
from isaacsim.core.api import World
from isaacsim.core.utils.stage import add_reference_to_stage, open_stage
from isaacsim.sensors.rtx import LidarRtx

ASSETS_ROOT = "https://omniverse-content-production.s3-us-west-2.amazonaws.com/Assets/Isaac/5.1"
WAREHOUSE_USD = f"{ASSETS_ROOT}/Isaac/Environments/Simple_Warehouse/warehouse.usd"
CARTER_USD = f"{ASSETS_ROOT}/Isaac/Robots/NVIDIA/NovaCarter/nova_carter.usd"
SCENE_ROOT = "/Root/World"
CARTER_PRIM = f"{SCENE_ROOT}/Carter"
LIDAR_PRIM = f"{CARTER_PRIM}/chassis_link/lidar_2d"
LIDAR_CONFIG = "Example_Rotary_2D"

FLATSCAN_TO_PUBLISH_PORTS = (
    "exec",
    "linearDepthData",
    "intensitiesData",
    "horizontalFov",
    "horizontalResolution",
    "azimuthRange",
    "depthRange",
    "numRows",
    "numCols",
    "rotationRate",
)

open_stage(WAREHOUSE_USD)
add_reference_to_stage(CARTER_USD, CARTER_PRIM)

world = World(stage_units_in_meters=1.0)
lidar = world.scene.add(
    LidarRtx(
        prim_path=LIDAR_PRIM,
        name="rerun_demo_lidar",
        config_file_name=LIDAR_CONFIG,
    )
)
world.reset()
lidar.attach_annotator("IsaacComputeRTXLidarFlatScan")

flat_scan = next(
    (
        n
        for g in og.get_all_graphs()
        for n in g.get_nodes()
        if n.get_type_name() == "isaacsim.sensors.rtx.IsaacComputeRTXLidarFlatScan"
    ),
    None,
)
if flat_scan is None:
    raise RuntimeError(
        "FlatScan OG node not found after attach_annotator; check Isaac Sim version"
    )

publish_path = f"{flat_scan.get_graph().get_path_to_graph()}/PublishLidarFlatScanToRust"
og.Controller.create_node(publish_path, "omni.isaacsimrs.bridge.PublishLidarFlatScanToRust")
for port in FLATSCAN_TO_PUBLISH_PORTS:
    og.Controller.connect(
        flat_scan.get_attribute(f"outputs:{port}"),
        f"{publish_path}.inputs:{port}",
    )

omni.timeline.get_timeline_interface().play()
print(
    "[og_lidar_drive] RTX LiDAR -> PublishLidarFlatScanToRust on Simple_Warehouse + NovaCarter, timeline playing"
)

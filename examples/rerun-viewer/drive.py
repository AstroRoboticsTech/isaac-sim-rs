"""Drive both 2D and 3D RTX LiDAR streams through the bridge.

2D FlatScan uses og.Controller.connect because the FlatScan
annotator's on_attach_callback wires SdOnNewRenderProductFrame
into the FlatScan node — exec propagates without a NodeWriter.

3D PointCloud uses register_node_writer_with_telemetry against the
accumulating IsaacCreateRTXLidarScanBuffer annotator (full rotation
per output). The NoAccumulator variant emits a per-frame wedge
instead and visually spins.
"""

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
SCENE_ROOT = "/Root/World"
CARTER_PRIM = f"{SCENE_ROOT}/Carter"

LIDAR_2D_PRIM = f"{CARTER_PRIM}/chassis_link/lidar_2d"
LIDAR_2D_CONFIG = "Example_Rotary_2D"
LIDAR_3D_PRIM = f"{CARTER_PRIM}/chassis_link/sensors/XT_32/PandarXT_32_10hz"

FLATSCAN_PORTS = (
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


def _find_node(node_type_id):
    return next(
        (
            n
            for g in og.get_all_graphs()
            for n in g.get_nodes()
            if n.get_type_name() == node_type_id
        ),
        None,
    )


open_stage(WAREHOUSE_USD)
add_reference_to_stage(CARTER_USD, CARTER_PRIM)

world = World(stage_units_in_meters=1.0)
lidar_2d = world.scene.add(
    LidarRtx(
        prim_path=LIDAR_2D_PRIM,
        name="rerun_demo_lidar_2d",
        config_file_name=LIDAR_2D_CONFIG,
    )
)
lidar_3d = world.scene.add(
    LidarRtx(prim_path=LIDAR_3D_PRIM, name="rerun_demo_lidar_3d")
)
world.reset()

lidar_2d.attach_annotator("IsaacComputeRTXLidarFlatScan")

flat_scan = _find_node("isaacsim.sensors.rtx.IsaacComputeRTXLidarFlatScan")
if flat_scan is None:
    raise RuntimeError("FlatScan OG node not found after attach_annotator")
flatscan_publish_path = (
    f"{flat_scan.get_graph().get_path_to_graph()}/PublishLidarFlatScanToRust"
)
og.Controller.create_node(
    flatscan_publish_path, "omni.isaacsimrs.bridge.PublishLidarFlatScanToRust"
)
for port in FLATSCAN_PORTS:
    og.Controller.connect(
        flat_scan.get_attribute(f"outputs:{port}"),
        f"{flatscan_publish_path}.inputs:{port}",
    )
og.Controller.set(
    og.Controller.attribute(f"{flatscan_publish_path}.inputs:sourceId"), LIDAR_2D_PRIM
)

register_node_writer_with_telemetry(
    name="PublishLidarPointCloudToRust",
    node_type_id="omni.isaacsimrs.bridge.PublishLidarPointCloudToRust",
    annotators=[
        "IsaacCreateRTXLidarScanBuffer",
        "PostProcessDispatchIsaacSimulationGate",
    ],
    category="omni.isaacsimrs.bridge",
)
pointcloud_writer = rep.WriterRegistry.get("PublishLidarPointCloudToRust")
pointcloud_writer.initialize(sourceId=LIDAR_3D_PRIM)
pointcloud_writer.attach([lidar_3d.get_render_product_path()])

omni.timeline.get_timeline_interface().play()
print("[og_lidar_drive] 2D FlatScan + 3D PointCloud wired; timeline playing")

"""Drive 2D + 3D RTX LiDAR, RGBD camera + camera info, and IMU through the bridge.

2D FlatScan uses og.Controller.connect because the FlatScan
annotator's on_attach_callback wires SdOnNewRenderProductFrame
into the FlatScan node — exec propagates without a NodeWriter.

3D PointCloud uses register_node_writer_with_telemetry against the
accumulating IsaacCreateRTXLidarScanBuffer annotator (full rotation
per output). The NoAccumulator variant emits a per-frame wedge
instead and visually spins.

Camera RGB and Depth share one Camera prim and one render product but
use independent annotator chains (LdrColorSDIsaacConvertRGBAToRGB and
DistanceToImagePlaneSDIsaacPassthroughImagePtr respectively). They
dispatch independently at the camera's render rate, mirroring NVIDIA's
ROS2 image bridge.

Camera info rides the PostProcessDispatchIsaacSimulationGate annotator
chain — the same one NVIDIA's ROS2PublishCameraInfo writer uses. K is
computed from the camera prim's focalLength / aperture / resolution
once at startup and pinned as static inputs on the writer; timeStamp
is auto-wired from IsaacReadSimulationTime via a NodeConnectionTemplate.

IMU is sampled per physics step, not per render frame — so it doesn't
fit the NodeWriter pattern. We create an IMUSensor prim, then manually
chain IsaacReadIMU and PublishImuToRust into the same push graph as the
LiDAR flat-scan, ticking IsaacReadIMU.execIn off the FlatScan node's
outputs:exec (10 Hz lidar cadence — enough for a visualization demo).
"""

import omni.graph.core as og
import omni.replicator.core as rep
import omni.syntheticdata._syntheticdata as sd
import omni.timeline
import omni.usd
from isaacsim.core.api import World
from isaacsim.core.nodes.scripts.utils import register_node_writer_with_telemetry
from isaacsim.core.utils.stage import add_reference_to_stage, open_stage
from isaacsim.sensors.camera import Camera
from isaacsim.sensors.physics import IMUSensor
from isaacsim.sensors.rtx import LidarRtx
from omni.syntheticdata import SyntheticData

ASSETS_ROOT = "https://omniverse-content-production.s3-us-west-2.amazonaws.com/Assets/Isaac/5.1"
WAREHOUSE_USD = f"{ASSETS_ROOT}/Isaac/Environments/Simple_Warehouse/warehouse.usd"
CARTER_USD = f"{ASSETS_ROOT}/Isaac/Robots/NVIDIA/NovaCarter/nova_carter.usd"
SCENE_ROOT = "/Root/World"
CARTER_PRIM = f"{SCENE_ROOT}/Carter"

LIDAR_2D_PRIM = f"{CARTER_PRIM}/chassis_link/lidar_2d"
LIDAR_2D_CONFIG = "Example_Rotary_2D"
LIDAR_3D_PRIM = f"{CARTER_PRIM}/chassis_link/sensors/XT_32/PandarXT_32_10hz"
CAMERA_RGB_PRIM = f"{CARTER_PRIM}/chassis_link/camera_rgb"
CAMERA_RGB_RESOLUTION = (640, 480)
IMU_PRIM = f"{CARTER_PRIM}/chassis_link/imu"

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
camera_rgb = Camera(
    prim_path=CAMERA_RGB_PRIM,
    name="rerun_demo_camera_rgb",
    resolution=CAMERA_RGB_RESOLUTION,
    translation=[0.3, 0.0, 0.2],
    orientation=[1.0, 0.0, 0.0, 0.0],
)
imu_sensor = IMUSensor(
    prim_path=IMU_PRIM,
    name="rerun_demo_imu",
    translation=[0.0, 0.0, 0.1],
    orientation=[1.0, 0.0, 0.0, 0.0],
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

# Camera RGB. The Camera object creates and attaches a render product;
# initialize() also attaches the standard `rgb` annotator which the
# IsaacConvertRGBAToRGB downstream annotator (LdrColorSD prefix) reads.
camera_rgb.initialize()
camera_rgb_rendervar = SyntheticData.convert_sensor_type_to_rendervar(sd.SensorType.Rgb.name)
camera_rgb_writer_name = f"{camera_rgb_rendervar}PublishCameraRgbToRust"
register_node_writer_with_telemetry(
    name=camera_rgb_writer_name,
    node_type_id="omni.isaacsimrs.bridge.PublishCameraRgbToRust",
    annotators=[f"{camera_rgb_rendervar}IsaacConvertRGBAToRGB"],
    category="omni.isaacsimrs.bridge",
)
camera_rgb_writer = rep.WriterRegistry.get(camera_rgb_writer_name)
camera_rgb_writer.initialize(sourceId=CAMERA_RGB_PRIM)
camera_rgb_writer.attach([camera_rgb.get_render_product_path()])

# Depth comes from the same Camera prim — attach the distance annotator
# and a second NodeWriter against IsaacPassthroughImagePtr for the
# DistanceToImagePlaneSD rendervar (float32 metres per pixel).
camera_rgb.attach_annotator("distance_to_image_plane")
camera_depth_rendervar = SyntheticData.convert_sensor_type_to_rendervar(
    sd.SensorType.DistanceToImagePlane.name
)
camera_depth_writer_name = f"{camera_depth_rendervar}PublishCameraDepthToRust"
register_node_writer_with_telemetry(
    name=camera_depth_writer_name,
    node_type_id="omni.isaacsimrs.bridge.PublishCameraDepthToRust",
    annotators=[f"{camera_depth_rendervar}IsaacPassthroughImagePtr"],
    category="omni.isaacsimrs.bridge",
)
camera_depth_writer = rep.WriterRegistry.get(camera_depth_writer_name)
camera_depth_writer.initialize(sourceId=CAMERA_RGB_PRIM)
camera_depth_writer.attach([camera_rgb.get_render_product_path()])

# Camera info. Compute K once from the camera prim's pinhole intrinsics
# (focalLength / aperture / resolution are all in mm and pixels;
# fx_pixels = focalLength_mm / horizontalAperture_mm * width). We
# hardcode r as identity and p as [K | 0] — monocular, no rectification.
focal_length_mm = camera_rgb.get_focal_length()
horizontal_aperture_mm = camera_rgb.get_horizontal_aperture()
vertical_aperture_mm = camera_rgb.get_vertical_aperture()
img_width, img_height = CAMERA_RGB_RESOLUTION
fx = (focal_length_mm / horizontal_aperture_mm) * img_width
fy = (focal_length_mm / vertical_aperture_mm) * img_height
cx = img_width / 2.0
cy = img_height / 2.0
camera_info_k = [fx, 0.0, cx, 0.0, fy, cy, 0.0, 0.0, 1.0]
camera_info_r = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
camera_info_p = [fx, 0.0, cx, 0.0, 0.0, fy, cy, 0.0, 0.0, 0.0, 1.0, 0.0]

register_node_writer_with_telemetry(
    name="PublishCameraInfoToRust",
    node_type_id="omni.isaacsimrs.bridge.PublishCameraInfoToRust",
    annotators=[
        "PostProcessDispatchIsaacSimulationGate",
        SyntheticData.NodeConnectionTemplate(
            "IsaacReadSimulationTime",
            attributes_mapping={"outputs:simulationTime": "inputs:timeStamp"},
        ),
    ],
    category="omni.isaacsimrs.bridge",
)
camera_info_writer = rep.WriterRegistry.get("PublishCameraInfoToRust")
camera_info_writer.initialize(
    sourceId=CAMERA_RGB_PRIM,
    frameId="sim_camera",
    width=img_width,
    height=img_height,
    k=camera_info_k,
    r=camera_info_r,
    p=camera_info_p,
    physicalDistortionCoefficients=[],
    physicalDistortionModel="",
    projectionType="pinhole",
)
camera_info_writer.attach([camera_rgb.get_render_product_path()])

# IMU. Manual og.Controller wiring inside the same push graph as the
# FlatScan publisher: chain IsaacReadIMU.execIn off the FlatScan's
# outputs:exec, then chain PublishImuToRust.execIn off
# IsaacReadIMU.outputs:execOut. Sample data flows on the same connect
# pattern. imuPrim is a USD relationship; set via og.Controller.
imu_graph_path = flat_scan.get_graph().get_path_to_graph()
read_imu_path = f"{imu_graph_path}/IsaacReadIMU"
publish_imu_path = f"{imu_graph_path}/PublishImuToRust"
og.Controller.create_node(read_imu_path, "isaacsim.sensors.physics.IsaacReadIMU")
og.Controller.create_node(
    publish_imu_path, "omni.isaacsimrs.bridge.PublishImuToRust"
)
og.Controller.set(
    og.Controller.attribute(f"{read_imu_path}.inputs:imuPrim"),
    [IMU_PRIM],
)
og.Controller.connect(
    flat_scan.get_attribute("outputs:exec"),
    f"{read_imu_path}.inputs:execIn",
)
for src, dst in (
    ("execOut", "execIn"),
    ("linAcc", "linearAcceleration"),
    ("angVel", "angularVelocity"),
    ("orientation", "orientation"),
    ("sensorTime", "timeStamp"),
):
    og.Controller.connect(
        f"{read_imu_path}.outputs:{src}",
        f"{publish_imu_path}.inputs:{dst}",
    )
og.Controller.set(
    og.Controller.attribute(f"{publish_imu_path}.inputs:sourceId"), IMU_PRIM
)
og.Controller.set(
    og.Controller.attribute(f"{publish_imu_path}.inputs:frameId"), "sim_imu"
)

omni.timeline.get_timeline_interface().play()
print(
    "[og_drive] 2D FlatScan + 3D PointCloud + RGB + Depth + CameraInfo + IMU "
    "wired; timeline playing"
)

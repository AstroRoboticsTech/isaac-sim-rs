//! Bridge runner the Carb plugin dlopens via
//! `ISAAC_SIM_RS_RERUN_RUNNER`. Sensor entities are organized as
//! children of a single `chassis` entity so a Transform3D logged at
//! `chassis` (from the odometry publisher) moves all sensors with the
//! robot — Carter visibly drives through the warehouse with its
//! sensor frustums and scans attached.
//!
//! Camera info is logged to the same entity as the RGB image
//! (`chassis/camera_rgb`) so the rerun viewer projects the RGB frame
//! inside the pinhole frustum. Depth is intentionally on a sibling
//! entity and gets its own panel without a frustum.

use isaac_sim_bridge::{
    CameraDepth, CameraInfo, CameraRgb, Imu, LidarFlatScan, LidarPointCloud, Odometry,
};
use isaac_sim_rerun::Viewer;

// Carter's articulation root sits at the chassis_link rigid body —
// not the top-level Carter prim. IsaacComputeOdometry rejects the
// latter ("not a valid rigid body or articulation root").
const CHASSIS_PRIM: &str = "/Root/World/Carter/chassis_link";
const LIDAR_2D_PRIM: &str = "/Root/World/Carter/chassis_link/lidar_2d";
const LIDAR_3D_PRIM: &str = "/Root/World/Carter/chassis_link/sensors/XT_32/PandarXT_32_10hz";
const CAMERA_PRIM: &str = "/Root/World/Carter/chassis_link/camera_rgb";
const IMU_PRIM: &str = "/Root/World/Carter/chassis_link/imu";

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_rerun_init() -> i32 {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    match try_init() {
        Ok(()) => 0,
        Err(e) => {
            log::error!("[example-rerun-viewer] init failed: {e}");
            -1
        }
    }
}

fn try_init() -> eyre::Result<()> {
    Viewer::new()
        .with_source(Odometry, CHASSIS_PRIM, "chassis")
        .with_source(LidarFlatScan, LIDAR_2D_PRIM, "chassis/lidar_flatscan")
        .with_source(LidarPointCloud, LIDAR_3D_PRIM, "chassis/lidar_pointcloud")
        .with_source(CameraRgb, CAMERA_PRIM, "chassis/camera_rgb")
        .with_source(CameraInfo, CAMERA_PRIM, "chassis/camera_rgb")
        .with_source(CameraDepth, CAMERA_PRIM, "chassis/camera_depth")
        .with_source(Imu, IMU_PRIM, "chassis/imu")
        .run()
}

//! Bridge runner the Carb plugin dlopens via
//! `ISAAC_SIM_RS_RERUN_RUNNER`. Pure wiring: each sensor goes to its
//! own top-level entity tree so the rerun viewer's default blueprint
//! gives each its own panel — RGB and depth do not share a 2D view.

use isaac_sim_bridge::{CameraDepth, CameraRgb, LidarFlatScan, LidarPointCloud};
use isaac_sim_rerun::Viewer;

const LIDAR_2D_PRIM: &str = "/Root/World/Carter/chassis_link/lidar_2d";
const LIDAR_3D_PRIM: &str = "/Root/World/Carter/chassis_link/sensors/XT_32/PandarXT_32_10hz";
const CAMERA_PRIM: &str = "/Root/World/Carter/chassis_link/camera_rgb";

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
        .with_source(LidarFlatScan, LIDAR_2D_PRIM, "lidar_flatscan")
        .with_source(LidarPointCloud, LIDAR_3D_PRIM, "lidar_pointcloud")
        .with_source(CameraRgb, CAMERA_PRIM, "camera_rgb")
        .with_source(CameraDepth, CAMERA_PRIM, "camera_depth")
        .run()
}

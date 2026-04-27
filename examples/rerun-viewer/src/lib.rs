use isaac_sim_bridge::{LidarFlatScan, LidarPointCloud};
use isaac_sim_rerun::Viewer;

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
        .with_source(
            LidarFlatScan,
            "/Root/World/Carter/chassis_link/lidar_2d",
            "scene/lidar/flatscan",
        )
        .with_source(
            LidarPointCloud,
            "/Root/World/Carter/chassis_link/sensors/XT_32/PandarXT_32_10hz",
            "scene/lidar/pointcloud",
        )
        .with_blueprint(|rec| {
            rec.log_static(
                "scene/lidar/flatscan",
                &rerun::TextDocument::new(
                    "2D RTX LiDAR (Example_Rotary_2D) mounted on Carter chassis.",
                ),
            )?;
            rec.log_static(
                "scene/lidar/pointcloud",
                &rerun::TextDocument::new(
                    "3D RTX LiDAR (PandarXT_32_10hz) — Carter's built-in sensor, 32-channel.",
                ),
            )?;
            Ok(())
        })
        .run()
}

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
        .with_lidar_flatscan("/World/LidarGraph/LidarFwd", "scene/lidar/flatscan")
        .with_blueprint(|rec| {
            rec.log_static(
                "scene/lidar/flatscan",
                &rerun::TextDocument::new(
                    "RTX LiDAR flat scan from Isaac Sim, projected planar (single row).",
                ),
            )?;
            Ok(())
        })
        .run()
}

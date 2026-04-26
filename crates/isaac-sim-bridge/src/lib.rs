use std::sync::Once;

static INIT: Once = Once::new();

#[cxx::bridge(namespace = "isaacsimrs")]
mod ffi {
    struct ScanMeta {
        horizontal_fov: f32,
        horizontal_resolution: f32,
        azimuth_min: f32,
        azimuth_max: f32,
        depth_min: f32,
        depth_max: f32,
        num_rows: i32,
        num_cols: i32,
        rotation_rate: f32,
    }

    extern "Rust" {
        fn init();
        fn double_value(x: i32) -> i32;
        fn forward_lidar_scan(scan: &[f32], intensities: &[u8], meta: &ScanMeta);
    }
}

fn init() {
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .try_init();
        log::info!("[isaac-sim-rs] init: env_logger up");
    });
}

fn double_value(x: i32) -> i32 {
    log::info!("[isaac-sim-rs] double_value({}) called from C++", x);
    x * 2
}

fn forward_lidar_scan(scan: &[f32], intensities: &[u8], meta: &ffi::ScanMeta) {
    let depth_min = scan.iter().cloned().fold(f32::INFINITY, f32::min);
    let depth_max = scan.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    log::info!(
        "[isaac-sim-rs] forward_lidar_scan: scan_n={}, intensity_n={}, fov={:.1}°, res={:.3}°, az=[{:.2},{:.2}]°, depth=[{:.2},{:.2}]m, rows={}, cols={}, rate={:.1}Hz, observed_depth=[{:.3},{:.3}]m",
        scan.len(),
        intensities.len(),
        meta.horizontal_fov,
        meta.horizontal_resolution,
        meta.azimuth_min,
        meta.azimuth_max,
        meta.depth_min,
        meta.depth_max,
        meta.num_rows,
        meta.num_cols,
        meta.rotation_rate,
        depth_min,
        depth_max
    );
}

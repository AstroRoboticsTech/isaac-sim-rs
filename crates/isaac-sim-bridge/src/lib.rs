mod demo;
mod lidar;
mod lifecycle;

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

use demo::double_value;
use lidar::forward_lidar_scan;
use lifecycle::init;

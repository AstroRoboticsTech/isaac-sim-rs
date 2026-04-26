mod consumers;
mod demo;
mod lidar_flatscan;
mod lifecycle;

pub use consumers::{
    dispatch_lidar_flatscan, lidar_flatscan_consumer_count, register_lidar_flatscan_consumer,
};

#[cxx::bridge(namespace = "isaacsimrs")]
mod ffi {
    struct LidarFlatScanMeta {
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
        fn forward_lidar_flatscan(scan: &[f32], intensities: &[u8], meta: &LidarFlatScanMeta);
    }
}

pub use ffi::LidarFlatScanMeta;

use demo::double_value;
use lidar_flatscan::forward_lidar_flatscan;
use lifecycle::init;

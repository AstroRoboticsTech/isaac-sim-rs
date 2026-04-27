mod channel;
mod demo;
mod lidar;
mod lifecycle;

pub use lidar::flatscan::{
    dispatch_lidar_flatscan, lidar_flatscan_consumer_count, register_lidar_flatscan_consumer,
};
pub use lidar::pointcloud::{
    dispatch_lidar_pointcloud, lidar_pointcloud_consumer_count, register_lidar_pointcloud_consumer,
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

    struct LidarPointCloudMeta {
        num_points: i32,
        width: i32,
        height: i32,
    }

    extern "Rust" {
        fn init();
        fn double_value(x: i32) -> i32;
        fn forward_lidar_flatscan(
            source_id: &str,
            scan: &[f32],
            intensities: &[u8],
            meta: &LidarFlatScanMeta,
        );
        fn forward_lidar_pointcloud(source_id: &str, points: &[f32], meta: &LidarPointCloudMeta);
    }
}

pub use ffi::{LidarFlatScanMeta, LidarPointCloudMeta};

use demo::double_value;
use lidar::flatscan::forward_lidar_flatscan;
use lidar::pointcloud::forward_lidar_pointcloud;
use lifecycle::init;

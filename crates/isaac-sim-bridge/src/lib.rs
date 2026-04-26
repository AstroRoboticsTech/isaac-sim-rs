mod consumers;
mod demo;
mod lidar_flatscan;
mod lidar_pointcloud;
mod lifecycle;

pub use consumers::{
    dispatch_lidar_flatscan, dispatch_lidar_pointcloud, lidar_flatscan_consumer_count,
    lidar_pointcloud_consumer_count, register_lidar_flatscan_consumer,
    register_lidar_pointcloud_consumer,
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
    }

    extern "Rust" {
        fn init();
        fn double_value(x: i32) -> i32;
        fn forward_lidar_flatscan(scan: &[f32], intensities: &[u8], meta: &LidarFlatScanMeta);
        fn forward_lidar_pointcloud(
            azimuth: &[f32],
            elevation: &[f32],
            distance: &[f32],
            intensity: &[f32],
            meta: &LidarPointCloudMeta,
        );
    }
}

pub use ffi::{LidarFlatScanMeta, LidarPointCloudMeta};

use demo::double_value;
use lidar_flatscan::forward_lidar_flatscan;
use lidar_pointcloud::forward_lidar_pointcloud;
use lifecycle::init;

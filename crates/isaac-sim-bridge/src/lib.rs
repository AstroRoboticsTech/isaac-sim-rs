mod articulation;
mod camera;
mod channel;
mod demo;
mod lidar;
mod lifecycle;
mod producer;
mod sensor;
mod source;

pub use articulation::cmd_vel::{cmd_vel_producer_count, register_cmd_vel_producer, CmdVelChannel};
pub use camera::rgb::{
    camera_rgb_consumer_count, dispatch_camera_rgb, register_camera_rgb_consumer, CameraRgb,
};
pub use lidar::flatscan::{
    dispatch_lidar_flatscan, lidar_flatscan_consumer_count, register_lidar_flatscan_consumer,
    LidarFlatScan,
};
pub use lidar::pointcloud::{
    dispatch_lidar_pointcloud, lidar_pointcloud_consumer_count, register_lidar_pointcloud_consumer,
    LidarPointCloud,
};
pub use producer::{ProducerRegistry, ProducerSlot};
pub use sensor::Sensor;
pub use source::SourceFilter;

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

    struct CameraRgbMeta {
        width: i32,
        height: i32,
        fx: f32,
        fy: f32,
        cx: f32,
        cy: f32,
        timestamp_ns: i64,
    }

    #[derive(Default, Clone, Copy)]
    struct CmdVel {
        linear_x: f32,
        linear_y: f32,
        linear_z: f32,
        angular_x: f32,
        angular_y: f32,
        angular_z: f32,
        timestamp_ns: i64,
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
        fn forward_camera_rgb(source_id: &str, pixels: &[u8], meta: &CameraRgbMeta);
        fn poll_cmd_vel(target_id: &str, out: &mut CmdVel) -> bool;
    }
}

pub use ffi::{CameraRgbMeta, CmdVel, LidarFlatScanMeta, LidarPointCloudMeta};

use articulation::cmd_vel::poll_cmd_vel;
use camera::rgb::forward_camera_rgb;
use demo::double_value;
use lidar::flatscan::forward_lidar_flatscan;
use lidar::pointcloud::forward_lidar_pointcloud;
use lifecycle::init;

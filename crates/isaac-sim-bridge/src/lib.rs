// SPDX-License-Identifier: MPL-2.0
#![cfg_attr(docsrs, feature(doc_cfg))]
//! `cxx::bridge` core for the Isaac Sim Rust SDK.
//!
//! Exposes a process-wide consumer registry and producer registry that
//! adapters (dora, rerun, custom) plug into. The bridge cdylib entry points
//! (`forward_*`, `poll_cmd_vel`) are gated behind the `cdylib` feature so
//! the rlib path compiles on any machine without Isaac Sim installed.

#![warn(missing_docs)]

mod articulation;
mod camera;
mod channel;
mod demo;
mod imu;
mod lidar;
mod lifecycle;
mod odometry;
mod producer;
mod sensor;
mod source;

pub use articulation::cmd_vel::{
    cmd_vel_consumer_count, cmd_vel_producer_count, peek_cmd_vel, register_cmd_vel_consumer,
    register_cmd_vel_producer, CmdVelChannel,
};
pub use camera::depth::{
    camera_depth_consumer_count, dispatch_camera_depth, register_camera_depth_consumer, CameraDepth,
};
pub use camera::info::{
    camera_info_consumer_count, dispatch_camera_info, register_camera_info_consumer, CameraInfo,
    CameraInfoFrame,
};
pub use camera::rgb::{
    camera_rgb_consumer_count, dispatch_camera_rgb, register_camera_rgb_consumer, CameraRgb,
};
pub use imu::{dispatch_imu, imu_consumer_count, register_imu_consumer, Imu};
pub use lidar::flatscan::{
    dispatch_lidar_flatscan, lidar_flatscan_consumer_count, register_lidar_flatscan_consumer,
    LidarFlatScan,
};
pub use lidar::pointcloud::{
    dispatch_lidar_pointcloud, lidar_pointcloud_consumer_count, register_lidar_pointcloud_consumer,
    LidarPointCloud,
};
pub use odometry::{
    dispatch_odometry, odometry_consumer_count, register_odometry_consumer, Odometry,
};
pub use producer::{ProducerRegistry, ProducerSlot};
pub use sensor::Sensor;
pub use source::SourceFilter;

#[allow(clippy::too_many_arguments)]
#[allow(missing_docs)]
#[cxx::bridge(namespace = "isaacsimrs")]
mod ffi {
    #[derive(Clone, Copy)]
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

    #[derive(Clone, Copy)]
    struct LidarPointCloudMeta {
        num_points: i32,
        width: i32,
        height: i32,
    }

    #[derive(Clone, Copy)]
    struct CameraRgbMeta {
        width: i32,
        height: i32,
        fx: f32,
        fy: f32,
        cx: f32,
        cy: f32,
        timestamp_ns: i64,
    }

    #[derive(Clone, Copy)]
    struct CameraDepthMeta {
        width: i32,
        height: i32,
        fx: f32,
        fy: f32,
        cx: f32,
        cy: f32,
        timestamp_ns: i64,
    }

    #[derive(Clone, Copy)]
    struct CameraInfoMeta {
        width: i32,
        height: i32,
        timestamp_ns: i64,
    }

    #[derive(Clone, Copy)]
    struct ImuMeta {
        lin_acc_x: f64,
        lin_acc_y: f64,
        lin_acc_z: f64,
        ang_vel_x: f64,
        ang_vel_y: f64,
        ang_vel_z: f64,
        orientation_w: f64,
        orientation_x: f64,
        orientation_y: f64,
        orientation_z: f64,
        timestamp_ns: i64,
    }

    #[derive(Clone, Copy)]
    struct OdometryMeta {
        position_x: f64,
        position_y: f64,
        position_z: f64,
        orientation_w: f64,
        orientation_x: f64,
        orientation_y: f64,
        orientation_z: f64,
        lin_vel_x: f64,
        lin_vel_y: f64,
        lin_vel_z: f64,
        ang_vel_x: f64,
        ang_vel_y: f64,
        ang_vel_z: f64,
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

    #[cfg(feature = "cdylib")]
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
        fn forward_camera_depth(source_id: &str, depths: &[f32], meta: &CameraDepthMeta);
        fn forward_camera_info(
            source_id: &str,
            frame_id: &str,
            distortion_model: &str,
            projection_type: &str,
            k: &[f64],
            r: &[f64],
            p: &[f64],
            distortion: &[f32],
            meta: &CameraInfoMeta,
        );
        fn forward_imu(source_id: &str, frame_id: &str, meta: &ImuMeta);
        fn forward_odometry(
            source_id: &str,
            chassis_frame_id: &str,
            odom_frame_id: &str,
            meta: &OdometryMeta,
        );
        fn poll_cmd_vel(target_id: &str, out: &mut CmdVel) -> bool;
    }
}

/// C++-compatible metadata structs shared across the cxx::bridge boundary.
/// Fields are named after physical quantities and need no further prose.
#[allow(missing_docs)]
pub use ffi::{
    CameraDepthMeta, CameraInfoMeta, CameraRgbMeta, CmdVel, ImuMeta, LidarFlatScanMeta,
    LidarPointCloudMeta, OdometryMeta,
};

#[cfg(feature = "cdylib")]
use articulation::cmd_vel::poll_cmd_vel;
#[cfg(feature = "cdylib")]
use camera::depth::forward_camera_depth;
#[cfg(feature = "cdylib")]
use camera::info::forward_camera_info;
#[cfg(feature = "cdylib")]
use camera::rgb::forward_camera_rgb;
#[cfg(feature = "cdylib")]
use demo::double_value;
#[cfg(feature = "cdylib")]
use imu::forward_imu;
#[cfg(feature = "cdylib")]
use lidar::flatscan::forward_lidar_flatscan;
#[cfg(feature = "cdylib")]
use lidar::pointcloud::forward_lidar_pointcloud;
#[cfg(feature = "cdylib")]
use lifecycle::init;
#[cfg(feature = "cdylib")]
use odometry::forward_odometry;

// SPDX-License-Identifier: MPL-2.0
//! Convenience decoders for downstream dora nodes that consume the
//! sensor outputs this crate publishes.
//!
//! Pattern from a downstream node's event loop:
//!
//! ```ignore
//! use dora_node_api::{DoraNode, Event};
//! use isaac_sim_dora::subscribe;
//!
//! let (_node, mut events) = DoraNode::init_from_env()?;
//! while let Some(event) = events.recv() {
//!     match event {
//!         Event::Input { id, data, .. } if id.as_str() == "lidar_pointcloud" => {
//!             let cloud = subscribe::lidar_pointcloud(&data.0)?;
//!             // run perception against `cloud.points`
//!         }
//!         Event::Input { id, data, .. } if id.as_str() == "imu" => {
//!             let imu = subscribe::imu(&data.0)?;
//!             // run state-estimation against `imu`
//!         }
//!         Event::Stop(_) => break,
//!         _ => {}
//!     }
//! }
//! # Ok::<(), arrow::error::ArrowError>(())
//! ```
//!
//! Each helper accepts `&ArrayRef` (so callers don't need to import
//! `arrow` directly) and returns `Result<T, ArrowError>`. The error
//! variant (`InvalidArgumentError` if the payload is not a StructArray,
//! `SchemaError` / `CastError` etc. from the inner decoder) is directly
//! accessible without downcasting. Re-export `ArrowError` so callers do
//! not need to add an `arrow` dependency solely for the error type.

pub use arrow::error::ArrowError;

use arrow::array::{ArrayRef, StructArray};
use isaac_sim_arrow::camera::depth::{from_struct_array as decode_camera_depth, CameraDepthOwned};
use isaac_sim_arrow::camera::info::{from_struct_array as decode_camera_info, CameraInfoOwned};
use isaac_sim_arrow::camera::rgb::{from_struct_array as decode_camera_rgb, CameraRgbOwned};
use isaac_sim_arrow::cmd_vel::{from_struct_array as decode_cmd_vel, CmdVel};
use isaac_sim_arrow::imu::{from_struct_array as decode_imu, ImuOwned};
use isaac_sim_arrow::lidar::flatscan::{
    from_struct_array as decode_lidar_flatscan, LidarFlatScanOwned,
};
use isaac_sim_arrow::lidar::pointcloud::{
    from_struct_array as decode_lidar_pointcloud, LidarPointCloudOwned,
};
use isaac_sim_arrow::odometry::{from_struct_array as decode_odometry, OdometryOwned};

fn cast<'a>(data: &'a ArrayRef, sensor: &str) -> Result<&'a StructArray, ArrowError> {
    data.as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| ArrowError::SchemaError(format!("{sensor} payload is not a StructArray")))
}

/// Decode an inbound dora `ArrayRef` as a `LidarFlatScan` sample. Errors if the
/// payload is not a `StructArray` or its fields do not match the expected schema.
pub fn lidar_flatscan(data: &ArrayRef) -> Result<LidarFlatScanOwned, ArrowError> {
    decode_lidar_flatscan(cast(data, "lidar_flatscan")?)
}

/// Decode an inbound dora `ArrayRef` as a `LidarPointCloud` sample.
pub fn lidar_pointcloud(data: &ArrayRef) -> Result<LidarPointCloudOwned, ArrowError> {
    decode_lidar_pointcloud(cast(data, "lidar_pointcloud")?)
}

/// Decode an inbound dora `ArrayRef` as an RGB camera frame.
pub fn camera_rgb(data: &ArrayRef) -> Result<CameraRgbOwned, ArrowError> {
    decode_camera_rgb(cast(data, "camera_rgb")?)
}

/// Decode an inbound dora `ArrayRef` as a depth camera frame.
pub fn camera_depth(data: &ArrayRef) -> Result<CameraDepthOwned, ArrowError> {
    decode_camera_depth(cast(data, "camera_depth")?)
}

/// Decode an inbound dora `ArrayRef` as camera calibration metadata.
pub fn camera_info(data: &ArrayRef) -> Result<CameraInfoOwned, ArrowError> {
    decode_camera_info(cast(data, "camera_info")?)
}

/// Decode an inbound dora `ArrayRef` as an IMU sample.
pub fn imu(data: &ArrayRef) -> Result<ImuOwned, ArrowError> {
    decode_imu(cast(data, "imu")?)
}

/// Decode an inbound dora `ArrayRef` as a chassis odometry sample.
pub fn odometry(data: &ArrayRef) -> Result<OdometryOwned, ArrowError> {
    decode_odometry(cast(data, "odometry")?)
}

/// Decode an inbound dora `ArrayRef` as a cmd_vel Twist command.
pub fn cmd_vel(data: &ArrayRef) -> Result<CmdVel, ArrowError> {
    decode_cmd_vel(cast(data, "cmd_vel")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use arrow::record_batch::RecordBatch;
    use isaac_sim_arrow::camera::depth::{to_record_batch as depth_batch, CameraDepth};
    use isaac_sim_arrow::camera::info::{to_record_batch as info_batch, CameraInfo};
    use isaac_sim_arrow::camera::rgb::{to_record_batch as rgb_batch, CameraRgb};
    use isaac_sim_arrow::cmd_vel::{to_record_batch as cmd_vel_batch, CmdVel};
    use isaac_sim_arrow::imu::{to_record_batch, Imu};
    use isaac_sim_arrow::lidar::flatscan::{to_record_batch as flatscan_batch, LidarFlatScan};
    use isaac_sim_arrow::lidar::pointcloud::{
        to_record_batch as pointcloud_batch, LidarPointCloud,
    };
    use isaac_sim_arrow::odometry::{to_record_batch as odometry_batch, Odometry};

    fn to_array_ref(batch: RecordBatch) -> ArrayRef {
        Arc::new(StructArray::from(batch))
    }

    #[test]
    fn imu_decode_round_trips_through_arrayref() {
        let sample = Imu {
            frame_id: "sim_imu",
            lin_acc_x: 0.1,
            lin_acc_y: 0.2,
            lin_acc_z: 9.81,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.5,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            timestamp_ns: 17,
        };
        let array = to_array_ref(to_record_batch(&sample).expect("convert"));
        let decoded = imu(&array).expect("decode");
        assert_eq!(decoded.frame_id, "sim_imu");
        assert!((decoded.lin_acc_z - 9.81).abs() < 1e-9);
        assert_eq!(decoded.timestamp_ns, 17);
    }

    #[test]
    fn lidar_flatscan_decode_round_trips_through_arrayref() {
        let depths = [0.5_f32, 1.0, 1.5];
        let intensities = [10_u8, 100, 200];
        let scan = LidarFlatScan {
            depths: &depths,
            intensities: &intensities,
            horizontal_fov: 270.0,
            horizontal_resolution: 0.25,
            azimuth_min: -135.0,
            azimuth_max: 135.0,
            depth_min: 0.1,
            depth_max: 30.0,
            num_rows: 1,
            num_cols: 3,
            rotation_rate: 10.0,
        };
        let array = to_array_ref(flatscan_batch(&scan).expect("convert"));
        let decoded = lidar_flatscan(&array).expect("decode");
        assert_eq!(decoded.depths, depths);
        assert_eq!(decoded.intensities, intensities);
        assert_eq!(decoded.horizontal_fov, 270.0);
        assert_eq!(decoded.num_cols, 3);
    }

    #[test]
    fn lidar_pointcloud_decode_round_trips_through_arrayref() {
        let points = [1.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0];
        let pc = LidarPointCloud {
            points: &points,
            num_points: 2,
            width: 2,
            height: 1,
        };
        let array = to_array_ref(pointcloud_batch(&pc).expect("convert"));
        let decoded = lidar_pointcloud(&array).expect("decode");
        assert_eq!(decoded.points, points);
        assert_eq!(decoded.num_points, 2);
    }

    #[test]
    fn camera_rgb_decode_round_trips_through_arrayref() {
        let pixels = vec![0_u8, 64, 128, 255, 1, 2];
        let img = CameraRgb {
            pixels: &pixels,
            width: 1,
            height: 2,
            fx: 100.0,
            fy: 110.0,
            cx: 0.5,
            cy: 1.0,
            timestamp_ns: 42,
        };
        let array = to_array_ref(rgb_batch(&img).expect("convert"));
        let decoded = camera_rgb(&array).expect("decode");
        assert_eq!(decoded.pixels, pixels);
        assert_eq!(decoded.width, 1);
        assert_eq!(decoded.timestamp_ns, 42);
    }

    #[test]
    fn camera_depth_decode_round_trips_through_arrayref() {
        let depths = vec![0.5_f32, 1.0, 1.5, 2.0];
        let img = CameraDepth {
            depths: &depths,
            width: 2,
            height: 2,
            fx: 100.0,
            fy: 100.0,
            cx: 1.0,
            cy: 1.0,
            timestamp_ns: 99,
        };
        let array = to_array_ref(depth_batch(&img).expect("convert"));
        let decoded = camera_depth(&array).expect("decode");
        assert_eq!(decoded.depths, depths);
        assert_eq!(decoded.timestamp_ns, 99);
    }

    #[test]
    fn camera_info_decode_round_trips_through_arrayref() {
        let k = [500.0_f64, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0];
        let r = [1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let p = [
            500.0_f64, 0.0, 320.0, 0.0, 0.0, 500.0, 240.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ];
        let d = [0.0_f32; 5];
        let info = CameraInfo {
            frame_id: "sim_camera",
            distortion_model: "plumb_bob",
            projection_type: "pinhole",
            k: &k,
            r: &r,
            p: &p,
            distortion: &d,
            width: 640,
            height: 480,
            timestamp_ns: 7,
        };
        let array = to_array_ref(info_batch(&info).expect("convert"));
        let decoded = camera_info(&array).expect("decode");
        assert_eq!(decoded.frame_id, "sim_camera");
        assert_eq!(decoded.k, k);
        assert_eq!(decoded.width, 640);
        assert_eq!(decoded.timestamp_ns, 7);
    }

    #[test]
    fn odometry_decode_round_trips_through_arrayref() {
        let odom = Odometry {
            chassis_frame_id: "base_link",
            odom_frame_id: "odom",
            position_x: 1.0,
            position_y: 2.0,
            position_z: 0.0,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            lin_vel_x: 0.4,
            lin_vel_y: 0.0,
            lin_vel_z: 0.0,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.3,
            timestamp_ns: 55,
        };
        let array = to_array_ref(odometry_batch(&odom).expect("convert"));
        let decoded = odometry(&array).expect("decode");
        assert_eq!(decoded.chassis_frame_id, "base_link");
        assert!((decoded.lin_vel_x - 0.4).abs() < 1e-9);
        assert_eq!(decoded.timestamp_ns, 55);
    }

    #[test]
    fn cmd_vel_decode_round_trips_through_arrayref() {
        let twist = CmdVel {
            linear_x: 0.4,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.3,
            timestamp_ns: 999,
        };
        let array = to_array_ref(cmd_vel_batch(&twist).expect("convert"));
        let decoded = cmd_vel(&array).expect("decode");
        assert_eq!(decoded, twist);
    }
}

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
//! # Ok::<(), eyre::Report>(())
//! ```
//!
//! Each helper accepts `&ArrayRef` (so callers don't need to import
//! `arrow` directly) and returns the matching `*Owned` type from
//! `isaac_sim_arrow`.

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

fn cast<'a>(data: &'a ArrayRef, sensor: &str) -> eyre::Result<&'a StructArray> {
    data.as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| eyre::eyre!("{sensor} payload is not a StructArray"))
}

pub fn lidar_flatscan(data: &ArrayRef) -> eyre::Result<LidarFlatScanOwned> {
    Ok(decode_lidar_flatscan(cast(data, "lidar_flatscan")?)?)
}

pub fn lidar_pointcloud(data: &ArrayRef) -> eyre::Result<LidarPointCloudOwned> {
    Ok(decode_lidar_pointcloud(cast(data, "lidar_pointcloud")?)?)
}

pub fn camera_rgb(data: &ArrayRef) -> eyre::Result<CameraRgbOwned> {
    Ok(decode_camera_rgb(cast(data, "camera_rgb")?)?)
}

pub fn camera_depth(data: &ArrayRef) -> eyre::Result<CameraDepthOwned> {
    Ok(decode_camera_depth(cast(data, "camera_depth")?)?)
}

pub fn camera_info(data: &ArrayRef) -> eyre::Result<CameraInfoOwned> {
    Ok(decode_camera_info(cast(data, "camera_info")?)?)
}

pub fn imu(data: &ArrayRef) -> eyre::Result<ImuOwned> {
    Ok(decode_imu(cast(data, "imu")?)?)
}

pub fn odometry(data: &ArrayRef) -> eyre::Result<OdometryOwned> {
    Ok(decode_odometry(cast(data, "odometry")?)?)
}

pub fn cmd_vel(data: &ArrayRef) -> eyre::Result<CmdVel> {
    Ok(decode_cmd_vel(cast(data, "cmd_vel")?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use isaac_sim_arrow::imu::{to_record_batch, Imu};

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
        let batch = to_record_batch(&sample).expect("convert");
        let array: ArrayRef = Arc::new(StructArray::from(batch));
        let decoded = imu(&array).expect("decode");
        assert_eq!(decoded.frame_id, "sim_imu");
        assert!((decoded.lin_acc_z - 9.81).abs() < 1e-9);
        assert_eq!(decoded.timestamp_ns, 17);
    }
}

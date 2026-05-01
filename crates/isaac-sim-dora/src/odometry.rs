use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::odometry::{to_record_batch, Odometry as ArrowOdometry};
use isaac_sim_bridge::{register_odometry_consumer, Odometry, OdometryMeta, SourceFilter};
use parking_lot::Mutex;

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::DoraPublish;

struct Frame {
    chassis_frame_id: String,
    odom_frame_id: String,
    meta: OdometryMeta,
}

impl DoraPublish for Odometry {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_odometry_publisher(node, source, output_id);
    }
}

pub fn register_dora_odometry_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    let (slot, wake) = LatestSlot::<Frame>::new();
    let source_for_drain = source.clone();
    let drain_name = format!("dora-drain-odometry:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = publish(
            &node,
            &output,
            &frame.chassis_frame_id,
            &frame.odom_frame_id,
            &frame.meta,
        ) {
            log::warn!("[isaac-sim-dora] odometry publish failed for '{source_for_drain}': {e}");
        }
    });

    register_odometry_consumer(move |src, chassis, odom, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame {
            chassis_frame_id: chassis.to_string(),
            odom_frame_id: odom.to_string(),
            meta: *meta,
        });
    });
}

pub fn build_struct_array(
    chassis_frame_id: &str,
    odom_frame_id: &str,
    meta: &OdometryMeta,
) -> eyre::Result<StructArray> {
    let odom = ArrowOdometry {
        chassis_frame_id,
        odom_frame_id,
        position_x: meta.position_x,
        position_y: meta.position_y,
        position_z: meta.position_z,
        orientation_w: meta.orientation_w,
        orientation_x: meta.orientation_x,
        orientation_y: meta.orientation_y,
        orientation_z: meta.orientation_z,
        lin_vel_x: meta.lin_vel_x,
        lin_vel_y: meta.lin_vel_y,
        lin_vel_z: meta.lin_vel_z,
        ang_vel_x: meta.ang_vel_x,
        ang_vel_y: meta.ang_vel_y,
        ang_vel_z: meta.ang_vel_z,
        timestamp_ns: meta.timestamp_ns,
    };
    let batch = to_record_batch(&odom)?;
    Ok(StructArray::from(batch))
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    chassis_frame_id: &str,
    odom_frame_id: &str,
    meta: &OdometryMeta,
) -> eyre::Result<()> {
    let array = build_struct_array(chassis_frame_id, odom_frame_id, meta)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn build_struct_array_round_trips_odometry() {
        let meta = OdometryMeta {
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
            timestamp_ns: 17,
        };
        let array = build_struct_array("base_link", "odom", &meta).expect("build");
        assert_eq!(array.num_columns(), 16);
        assert_eq!(array.len(), 1);
    }
}

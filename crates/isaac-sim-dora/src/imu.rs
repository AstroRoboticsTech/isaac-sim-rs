// SPDX-License-Identifier: MPL-2.0
use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::imu::{to_record_batch, Imu as ArrowImu};
use isaac_sim_bridge::{register_imu_consumer, Imu, ImuMeta, SourceFilter};
use parking_lot::Mutex;

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::DoraPublish;

struct Frame {
    frame_id: String,
    meta: ImuMeta,
}

impl DoraPublish for Imu {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_imu_publisher(node, source, output_id);
    }
}

pub fn register_dora_imu_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    let (slot, wake) = LatestSlot::<Frame>::new();
    let source_for_drain = source.clone();
    let drain_name = format!("dora-drain-imu:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = publish(&node, &output, &frame.frame_id, &frame.meta) {
            log::warn!("[isaac-sim-dora] imu publish failed for '{source_for_drain}': {e}");
        }
    });

    register_imu_consumer(move |src, frame_id, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame {
            frame_id: frame_id.to_string(),
            meta: *meta,
        });
    });
}

pub fn build_struct_array(frame_id: &str, meta: &ImuMeta) -> eyre::Result<StructArray> {
    let imu = ArrowImu {
        frame_id,
        lin_acc_x: meta.lin_acc_x,
        lin_acc_y: meta.lin_acc_y,
        lin_acc_z: meta.lin_acc_z,
        ang_vel_x: meta.ang_vel_x,
        ang_vel_y: meta.ang_vel_y,
        ang_vel_z: meta.ang_vel_z,
        orientation_w: meta.orientation_w,
        orientation_x: meta.orientation_x,
        orientation_y: meta.orientation_y,
        orientation_z: meta.orientation_z,
        timestamp_ns: meta.timestamp_ns,
    };
    let batch = to_record_batch(&imu)?;
    Ok(StructArray::from(batch))
}

fn publish(
    node: &Mutex<DoraNode>,
    output: &DataId,
    frame_id: &str,
    meta: &ImuMeta,
) -> eyre::Result<()> {
    let array = build_struct_array(frame_id, meta)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn build_struct_array_round_trips_imu() {
        let meta = ImuMeta {
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
            timestamp_ns: 13,
        };
        let array = build_struct_array("sim_imu", &meta).expect("build");
        assert_eq!(array.num_columns(), 12);
        assert_eq!(array.len(), 1);
    }
}

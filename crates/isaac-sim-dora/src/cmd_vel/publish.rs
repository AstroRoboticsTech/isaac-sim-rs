//! Bridge→dora cmd_vel publisher.
//!
//! Mirrors the sensor publisher pattern: register a consumer (here
//! `register_cmd_vel_consumer`), filter by target_id, drain through a
//! latest-wins slot, emit on a dora node output. The observer side of
//! this story lives in `isaac-sim-bridge::ProducerRegistry::add_observer`,
//! which fires for every cmd_vel `publish` from any Rust source.

use std::sync::Arc;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, MetadataParameters};
use isaac_sim_arrow::cmd_vel::{to_record_batch, CmdVel as ArrowCmdVel};
use isaac_sim_bridge::{
    register_cmd_vel_consumer, CmdVel as BridgeCmdVel, CmdVelChannel, SourceFilter,
};
use parking_lot::Mutex;

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::DoraPublish;

struct Frame {
    twist: BridgeCmdVel,
}

impl DoraPublish for CmdVelChannel {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String) {
        register_dora_cmd_vel_publisher(node, source, output_id);
    }
}

pub fn register_dora_cmd_vel_publisher(
    node: Arc<Mutex<DoraNode>>,
    source: String,
    output_id: impl Into<String>,
) {
    let output: DataId = output_id.into().into();
    let filter = SourceFilter::exact(source.clone());

    let (slot, wake) = LatestSlot::<Frame>::new();
    let source_for_drain = source.clone();
    let drain_name = format!("dora-drain-cmd_vel:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = publish(&node, &output, &frame.twist) {
            log::warn!("[isaac-sim-dora] cmd_vel publish failed for '{source_for_drain}': {e}");
        }
    });

    register_cmd_vel_consumer(move |target, twist| {
        if !filter.matches(target) {
            return;
        }
        slot.publish(Frame { twist: *twist });
    });
}

pub fn build_struct_array(twist: &BridgeCmdVel) -> eyre::Result<StructArray> {
    let arrow_twist = ArrowCmdVel {
        linear_x: twist.linear_x,
        linear_y: twist.linear_y,
        linear_z: twist.linear_z,
        angular_x: twist.angular_x,
        angular_y: twist.angular_y,
        angular_z: twist.angular_z,
        timestamp_ns: twist.timestamp_ns,
    };
    let batch = to_record_batch(&arrow_twist)?;
    Ok(StructArray::from(batch))
}

fn publish(node: &Mutex<DoraNode>, output: &DataId, twist: &BridgeCmdVel) -> eyre::Result<()> {
    let array = build_struct_array(twist)?;
    let mut guard = node.lock();
    guard.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn build_struct_array_round_trips_twist() {
        let twist = BridgeCmdVel {
            linear_x: 0.4,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.3,
            timestamp_ns: 11,
        };
        let array = build_struct_array(&twist).expect("build");
        assert_eq!(array.num_columns(), 7);
        assert_eq!(array.len(), 1);
    }
}

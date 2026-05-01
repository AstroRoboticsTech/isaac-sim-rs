//! Reverse-direction adapter: dora → bridge.
//!
//! Whereas every other module in this crate forwards bridge data onto
//! a dora output, this one consumes a dora `Twist`-shaped input and
//! republishes it into the bridge's cmd_vel producer slot. The C++
//! `OgnApplyCmdVelFromRust` node polls that slot every OG tick.
//!
//! Pair with the `DataId` declared as an input in the dora dataflow
//! YAML; whichever upstream node sends a `StructArray` matching
//! `isaac_sim_arrow::cmd_vel::schema()` drives the articulation.

use std::sync::Arc;
use std::thread::{self, JoinHandle};

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{Event, EventStream};
use isaac_sim_arrow::cmd_vel::from_struct_array;
use isaac_sim_bridge::{register_cmd_vel_producer, CmdVel as BridgeCmdVel, CmdVelChannel, Sensor};

/// Spawn a thread that drains `events`, decodes `Twist` Arrow batches
/// arriving on `input_id`, and republishes them into the bridge's
/// cmd_vel producer slot keyed by `target_id`.
///
/// The thread exits when the dora event stream closes (i.e. on
/// `Event::Stop`).
pub fn start_cmd_vel_subscriber(
    events: EventStream,
    input_id: DataId,
    target_id: impl Into<String>,
) -> JoinHandle<()> {
    let target_id = target_id.into();
    let slot = register_cmd_vel_producer(target_id.clone());
    let name = format!("dora-sub-{}:{target_id}", CmdVelChannel::NAME);
    thread::Builder::new()
        .name(name)
        .spawn(move || event_loop(events, input_id, target_id, slot))
        .expect("spawn dora cmd_vel subscriber thread")
}

fn event_loop(
    mut events: EventStream,
    input_id: DataId,
    target_id: String,
    slot: Arc<isaac_sim_bridge::ProducerSlot<BridgeCmdVel>>,
) {
    log::info!(
        "[isaac-sim-dora] cmd_vel subscriber: input='{input_id}' target='{target_id}' (poll-via-blocking-recv)"
    );
    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } if id == input_id => {
                if let Err(e) = handle_input(&data.0, &slot) {
                    log::warn!(
                        "[isaac-sim-dora] cmd_vel decode failed for target='{target_id}': {e}"
                    );
                }
            }
            Event::Stop(_) => break,
            _ => {}
        }
    }
    log::info!("[isaac-sim-dora] cmd_vel subscriber for target='{target_id}' exiting");
}

fn handle_input(
    data: &arrow::array::ArrayRef,
    slot: &isaac_sim_bridge::ProducerSlot<BridgeCmdVel>,
) -> eyre::Result<()> {
    let array = data
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| eyre::eyre!("cmd_vel input is not a StructArray"))?;
    let twist = from_struct_array(array)?;
    slot.publish(BridgeCmdVel {
        linear_x: twist.linear_x,
        linear_y: twist.linear_y,
        linear_z: twist.linear_z,
        angular_x: twist.angular_x,
        angular_y: twist.angular_y,
        angular_z: twist.angular_z,
        timestamp_ns: twist.timestamp_ns,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::StructArray;
    use isaac_sim_arrow::cmd_vel::{to_record_batch, CmdVel as ArrowCmdVel};
    use isaac_sim_bridge::peek_cmd_vel;

    #[test]
    fn handle_input_publishes_to_slot() {
        let target = "/test/articulation/handle_input_publishes";
        let slot = register_cmd_vel_producer(target);

        let twist = ArrowCmdVel {
            linear_x: 0.42,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: -0.17,
            timestamp_ns: 1_234,
        };
        let batch = to_record_batch(&twist).expect("convert");
        let array: arrow::array::ArrayRef = Arc::new(StructArray::from(batch));
        handle_input(&array, &slot).expect("handle");

        let polled = peek_cmd_vel(target).expect("slot has value");
        assert!((polled.linear_x - 0.42).abs() < 1e-6);
        assert!((polled.angular_z + 0.17).abs() < 1e-6);
        assert_eq!(polled.timestamp_ns, 1_234);
    }
}

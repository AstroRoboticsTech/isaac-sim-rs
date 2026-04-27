use std::sync::Arc;

use dora_node_api::DoraNode;
use isaac_sim_bridge::Sensor;
use parking_lot::Mutex;

/// Per-sensor trait that knows how to register a dora publisher for
/// that sensor type. Each sensor module provides one impl, so generic
/// init code can iterate sensors uniformly without each new sensor
/// adding env-var consts and a register call.
pub trait DoraPublish: Sensor {
    fn register(node: Arc<Mutex<DoraNode>>, source: String, output_id: String);
}

use std::env;
use std::sync::{Arc, Mutex};

use dora_node_api::DoraNode;
use isaac_sim_bridge::{LidarFlatScan, LidarPointCloud};

use crate::sensor::DoraPublish;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_dora_init() -> i32 {
    match try_init() {
        Ok(()) => 0,
        Err(e) => {
            log::error!("[isaac-sim-dora] init failed: {e}");
            -1
        }
    }
}

fn try_init() -> eyre::Result<()> {
    let (node, _events) = DoraNode::init_from_env()?;
    let node = Arc::new(Mutex::new(node));

    register_publisher::<LidarFlatScan>(Arc::clone(&node));
    register_publisher::<LidarPointCloud>(Arc::clone(&node));

    Ok(())
}

/// Look up `ISAAC_SIM_RS_DORA_<NAME>_SOURCE` and `_OUTPUT` for sensor
/// `S`, defaulting OUTPUT to `S::NAME`. Adding a new sensor is just
/// another `register_publisher::<NewSensor>(node)` call here.
fn register_publisher<S: DoraPublish>(node: Arc<Mutex<DoraNode>>) {
    let name_upper = S::NAME.to_uppercase();
    let source_env = format!("ISAAC_SIM_RS_DORA_{name_upper}_SOURCE");
    let output_env = format!("ISAAC_SIM_RS_DORA_{name_upper}_OUTPUT");

    let source = env::var(&source_env).unwrap_or_default();
    let output = env::var(&output_env).unwrap_or_else(|_| S::NAME.to_string());
    log::info!(
        "[isaac-sim-dora] {} publisher: source='{source}' output='{output}'",
        S::NAME
    );
    S::register(node, source, output);
}

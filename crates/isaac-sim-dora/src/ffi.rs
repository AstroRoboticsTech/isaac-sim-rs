use std::env;

use dora_node_api::DoraNode;

use crate::lidar_flatscan::register_dora_lidar_flatscan_publisher;

const LIDAR_FLATSCAN_OUTPUT_ENV: &str = "ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT";
const DEFAULT_LIDAR_FLATSCAN_OUTPUT: &str = "lidar_flatscan";

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
    let output = env::var(LIDAR_FLATSCAN_OUTPUT_ENV)
        .unwrap_or_else(|_| DEFAULT_LIDAR_FLATSCAN_OUTPUT.to_string());
    log::info!("[isaac-sim-dora] registering lidar_flatscan publisher on output '{output}'");
    register_dora_lidar_flatscan_publisher(node, output);
    Ok(())
}

use std::env;

use dora_node_api::DoraNode;

use crate::lidar::register_dora_lidar_publisher;

const LIDAR_OUTPUT_ENV: &str = "ISAAC_SIM_RS_DORA_LIDAR_OUTPUT";
const DEFAULT_LIDAR_OUTPUT: &str = "scan";

/// C ABI entry point. Called by the omni.isaacsimrs.bridge Carb plugin's
/// EagerInit ctor when ISAAC_SIM_RS_DORA_RUNNER points at this cdylib.
///
/// Reads dora environment variables (set by the dora coordinator), creates
/// a DoraNode, and registers it as a LiDAR consumer on the bridge.
///
/// Returns 0 on success, non-zero on failure (failure is logged and the
/// bridge continues without dora).
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
    let output = env::var(LIDAR_OUTPUT_ENV).unwrap_or_else(|_| DEFAULT_LIDAR_OUTPUT.to_string());
    log::info!("[isaac-sim-dora] registering lidar publisher on output '{output}'");
    register_dora_lidar_publisher(node, output);
    Ok(())
}

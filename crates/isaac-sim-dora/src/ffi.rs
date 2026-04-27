use std::env;
use std::sync::{Arc, Mutex};

use dora_node_api::DoraNode;

use crate::lidar::flatscan::register_dora_lidar_flatscan_publisher;
use crate::lidar::pointcloud::register_dora_lidar_pointcloud_publisher;

const LIDAR_FLATSCAN_SOURCE_ENV: &str = "ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_SOURCE";
const LIDAR_FLATSCAN_OUTPUT_ENV: &str = "ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT";
const DEFAULT_LIDAR_FLATSCAN_OUTPUT: &str = "lidar_flatscan";
const LIDAR_POINTCLOUD_SOURCE_ENV: &str = "ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_SOURCE";
const LIDAR_POINTCLOUD_OUTPUT_ENV: &str = "ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_OUTPUT";
const DEFAULT_LIDAR_POINTCLOUD_OUTPUT: &str = "lidar_pointcloud";

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

    let flatscan_source = env::var(LIDAR_FLATSCAN_SOURCE_ENV).unwrap_or_default();
    let flatscan_output = env::var(LIDAR_FLATSCAN_OUTPUT_ENV)
        .unwrap_or_else(|_| DEFAULT_LIDAR_FLATSCAN_OUTPUT.to_string());
    log::info!(
        "[isaac-sim-dora] lidar_flatscan publisher: source='{flatscan_source}' output='{flatscan_output}'"
    );
    register_dora_lidar_flatscan_publisher(Arc::clone(&node), flatscan_source, flatscan_output);

    let pointcloud_source = env::var(LIDAR_POINTCLOUD_SOURCE_ENV).unwrap_or_default();
    let pointcloud_output = env::var(LIDAR_POINTCLOUD_OUTPUT_ENV)
        .unwrap_or_else(|_| DEFAULT_LIDAR_POINTCLOUD_OUTPUT.to_string());
    log::info!(
        "[isaac-sim-dora] lidar_pointcloud publisher: source='{pointcloud_source}' output='{pointcloud_output}'"
    );
    register_dora_lidar_pointcloud_publisher(
        Arc::clone(&node),
        pointcloud_source,
        pointcloud_output,
    );

    Ok(())
}

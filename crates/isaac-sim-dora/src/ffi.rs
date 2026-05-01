// SPDX-License-Identifier: MPL-2.0
use std::env;
use std::sync::Arc;

use dora_node_api::dora_core::config::DataId;
use dora_node_api::DoraNode;
use isaac_sim_bridge::{
    CameraDepth, CameraInfo, CameraRgb, CmdVelChannel, Imu, LidarFlatScan, LidarPointCloud,
    Odometry,
};
use parking_lot::Mutex;

use crate::cmd_vel::start_cmd_vel_subscriber;
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
    let (node, events) = DoraNode::init_from_env()?;
    let node = Arc::new(Mutex::new(node));

    register_publisher::<LidarFlatScan>(Arc::clone(&node));
    register_publisher::<LidarPointCloud>(Arc::clone(&node));
    register_publisher::<CameraRgb>(Arc::clone(&node));
    register_publisher::<CameraDepth>(Arc::clone(&node));
    register_publisher::<CameraInfo>(Arc::clone(&node));
    register_publisher::<Imu>(Arc::clone(&node));
    register_publisher::<Odometry>(Arc::clone(&node));
    register_publisher_with_default::<CmdVelChannel>(Arc::clone(&node), "cmd_vel_observed");

    if let Some((input_id, target_id)) = lookup_cmd_vel_subscriber_config() {
        log::info!(
            "[isaac-sim-dora] starting cmd_vel subscriber: input='{input_id}' target='{target_id}'"
        );
        start_cmd_vel_subscriber(events, DataId::from(input_id), target_id);
    } else {
        log::info!(
            "[isaac-sim-dora] no cmd_vel subscriber configured (set ISAAC_SIM_RS_DORA_CMD_VEL_INPUT + _TARGET to enable)"
        );
    }

    Ok(())
}

/// Env-var convention for publisher direction (bridge → dora):
///
///   `ISAAC_SIM_RS_DORA_<NAME>_SOURCE`  — prim-path filter; empty string
///                                        means "accept any source".
///   `ISAAC_SIM_RS_DORA_<NAME>_OUTPUT`  — dora output id declared in the
///                                        dataflow; defaults to `S::NAME`.
///
/// The naming encodes direction: OUTPUT because data flows *out* of the
/// bridge into the dora graph. SOURCE because the filter selects which
/// simulator prim the data originates from.
fn register_publisher<S: DoraPublish>(node: Arc<Mutex<DoraNode>>) {
    register_publisher_with_default::<S>(node, S::NAME);
}

/// Same as `register_publisher` but lets the caller override the default
/// output id. Used for cmd_vel whose default must be `"cmd_vel_observed"`
/// (not `S::NAME = "cmd_vel"`) to avoid a dataflow self-loop when both the
/// publisher and subscriber are active.
fn register_publisher_with_default<S: DoraPublish>(
    node: Arc<Mutex<DoraNode>>,
    default_output: &str,
) {
    let name_upper = S::NAME.to_uppercase();
    let source_env = format!("ISAAC_SIM_RS_DORA_{name_upper}_SOURCE");
    let output_env = format!("ISAAC_SIM_RS_DORA_{name_upper}_OUTPUT");

    let source = env::var(&source_env).unwrap_or_default();
    let output = env::var(&output_env).unwrap_or_else(|_| default_output.to_string());
    log::info!(
        "[isaac-sim-dora] {} publisher: source='{source}' output='{output}'",
        S::NAME
    );
    S::register(node, source, output);
}

/// Env-var convention for subscriber direction (dora → bridge):
///
///   `ISAAC_SIM_RS_DORA_CMD_VEL_INPUT`  — dora input id the node listens on;
///                                        matches the upstream node's output
///                                        id in the dataflow.
///   `ISAAC_SIM_RS_DORA_CMD_VEL_TARGET` — producer-slot key; the prim path
///                                        the C++ ApplyCmdVel node polls.
///
/// The naming is intentionally asymmetric with the publisher: INPUT because
/// data flows *into* the bridge from the dora graph; TARGET because the value
/// is written to a producer slot keyed by articulation prim path, which is
/// the "target" of the actuation command.
///
/// Returns `Some((input_id, target_id))` when both vars are set and non-empty.
/// Either missing → no subscriber spawns and the EventStream is dropped
/// silently (publishers don't need it).
fn lookup_cmd_vel_subscriber_config() -> Option<(String, String)> {
    let input = env::var("ISAAC_SIM_RS_DORA_CMD_VEL_INPUT").ok()?;
    let target = env::var("ISAAC_SIM_RS_DORA_CMD_VEL_TARGET").ok()?;
    if input.is_empty() || target.is_empty() {
        return None;
    }
    Some((input, target))
}

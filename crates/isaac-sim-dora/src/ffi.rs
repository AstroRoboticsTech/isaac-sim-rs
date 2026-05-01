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
    register_publisher::<CmdVelChannel>(Arc::clone(&node));

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

/// Returns `Some((input_id, target_id))` when both env vars are set.
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

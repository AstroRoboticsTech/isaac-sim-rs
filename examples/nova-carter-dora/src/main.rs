//! Nova Carter dora receiver.
//!
//! Subscribes to every sensor channel the Kit-side dora source emits,
//! decodes via `isaac_sim_dora::subscribe::*`, prints a one-line
//! summary per input, and emits a constant-arc Twist on `cmd_vel` so
//! the bridge's cmd_vel subscriber drives Carter through the
//! warehouse.
//!
//! Optional rerun viz: set `RERUN_GRPC_ADDR` (e.g. `127.0.0.1:9876`)
//! and the receiver opens a `RecordingStream` and forwards IMU /
//! odometry / cmd_vel scalars + LiDAR pointcloud as the
//! "subscription-only viz" pipeline (rerun consumes data via dora,
//! not via direct bridge consumers).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use arrow::array::StructArray;
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, Event, MetadataParameters};
use isaac_sim_arrow::cmd_vel::{to_record_batch, CmdVel as ArrowCmdVel};
use isaac_sim_dora::subscribe;
use rerun::{
    datatypes::ChannelDatatype, DepthImage, Image, Points3D, RecordingStream,
    RecordingStreamBuilder, Scalars,
};

const CMD_VEL_OUTPUT: &str = "cmd_vel";
const CMD_VEL_LINEAR_X: f32 = 0.4;
const CMD_VEL_ANGULAR_Z: f32 = 0.0;
const RERUN_APP_ID: &str = "isaac-sim-rs/nova-carter-dora";
const RERUN_GRPC_ADDR_ENV: &str = "RERUN_GRPC_ADDR";

fn main() -> eyre::Result<()> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    let (mut node, mut events) = DoraNode::init_from_env()?;
    let cmd_vel_id: DataId = CMD_VEL_OUTPUT.into();
    let started = Instant::now();
    let viewer = open_rerun_sink()?;
    let counts: [AtomicU64; 8] = Default::default();
    log::info!("[receiver] started; rerun sink={}", viewer.is_some());

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } => {
                if let Err(e) = handle_input(&id, &data.0, &counts, viewer.as_ref()) {
                    log::warn!("[receiver] {id}: handle failed: {e}");
                }
                if id.as_str() == "odometry" {
                    publish_cmd_vel(&mut node, &cmd_vel_id, started)?;
                }
            }
            Event::Stop(_) => {
                log::info!("[receiver] stop event; exiting");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_input(
    id: &DataId,
    data: &arrow::array::ArrayRef,
    counts: &[AtomicU64; 8],
    viewer: Option<&RecordingStream>,
) -> eyre::Result<()> {
    match id.as_str() {
        "lidar_flatscan" => {
            let n = bump(counts, 0);
            let scan = subscribe::lidar_flatscan(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] lidar_flatscan: n={} fov={:.1}° rate={:.1}Hz depth=[{:.3},{:.3}]m",
                    scan.depths.len(),
                    scan.horizontal_fov,
                    scan.rotation_rate,
                    scan.depth_min,
                    scan.depth_max,
                );
            }
        }
        "lidar_pointcloud" => {
            let n = bump(counts, 1);
            let cloud = subscribe::lidar_pointcloud(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] lidar_pointcloud: n={} (floats={})",
                    cloud.num_points,
                    cloud.points.len()
                );
            }
            // Decimate pointcloud → rerun. 44k points × 10 Hz × full buffer = ~17 MB/s
            // pre-arrow encode; the gRPC client backpressures fast on a single
            // Linux-host viewer. Every 3rd scan ≈ 3 Hz keeps motion smooth.
            if let Some(rec) = viewer {
                if n.is_multiple_of(3) {
                    log_pointcloud_to_rerun(rec, &cloud)?;
                }
            }
        }
        "camera_rgb" => {
            let n = bump(counts, 2);
            let rgb = subscribe::camera_rgb(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] camera_rgb: {}x{} bytes={}",
                    rgb.width,
                    rgb.height,
                    rgb.pixels.len()
                );
            }
            // Decimate RGB to ~6 Hz (raw is 30 Hz × 921 600 B = 28 MB/s).
            if let Some(rec) = viewer {
                if n.is_multiple_of(5) {
                    log_rgb_to_rerun(rec, &rgb)?;
                }
            }
        }
        "camera_depth" => {
            let n = bump(counts, 3);
            let depth = subscribe::camera_depth(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] camera_depth: {}x{} samples={}",
                    depth.width,
                    depth.height,
                    depth.depths.len()
                );
            }
            if let Some(rec) = viewer {
                if n.is_multiple_of(5) {
                    log_depth_to_rerun(rec, &depth)?;
                }
            }
        }
        "camera_info" => {
            let n = bump(counts, 4);
            let info = subscribe::camera_info(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] camera_info: {}x{} k=[{:.1},{:.1},{:.1}] proj='{}' frame='{}'",
                    info.width,
                    info.height,
                    info.k.first().copied().unwrap_or_default(),
                    info.k.get(4).copied().unwrap_or_default(),
                    info.k.get(2).copied().unwrap_or_default(),
                    info.projection_type,
                    info.frame_id,
                );
            }
        }
        "imu" => {
            let n = bump(counts, 5);
            let imu = subscribe::imu(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] imu: lin_acc=[{:.2},{:.2},{:.2}] ang_vel=[{:.2},{:.2},{:.2}] frame='{}'",
                    imu.lin_acc_x,
                    imu.lin_acc_y,
                    imu.lin_acc_z,
                    imu.ang_vel_x,
                    imu.ang_vel_y,
                    imu.ang_vel_z,
                    imu.frame_id,
                );
            }
            if let Some(rec) = viewer {
                rec.log("dora/imu/lin_acc/z", &Scalars::single(imu.lin_acc_z))?;
                rec.log("dora/imu/ang_vel/z", &Scalars::single(imu.ang_vel_z))?;
            }
        }
        "odometry" => {
            let n = bump(counts, 6);
            let odom = subscribe::odometry(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] odometry: pos=[{:.3},{:.3},{:.3}] lin=[{:.3},{:.3},{:.3}] ang=[{:.3},{:.3},{:.3}] chassis='{}'",
                    odom.position_x,
                    odom.position_y,
                    odom.position_z,
                    odom.lin_vel_x,
                    odom.lin_vel_y,
                    odom.lin_vel_z,
                    odom.ang_vel_x,
                    odom.ang_vel_y,
                    odom.ang_vel_z,
                    odom.chassis_frame_id,
                );
            }
            if let Some(rec) = viewer {
                rec.log("dora/odom/pos/x", &Scalars::single(odom.position_x))?;
                rec.log("dora/odom/pos/y", &Scalars::single(odom.position_y))?;
                rec.log("dora/odom/lin/x", &Scalars::single(odom.lin_vel_x))?;
                rec.log("dora/odom/ang/z", &Scalars::single(odom.ang_vel_z))?;
            }
        }
        "cmd_vel_observed" => {
            let n = bump(counts, 7);
            let twist = subscribe::cmd_vel(data)?;
            if first_or_periodic(n) {
                log::info!(
                    "[receiver] cmd_vel_observed: linear=[{:.3},{:.3},{:.3}] angular=[{:.3},{:.3},{:.3}]",
                    twist.linear_x,
                    twist.linear_y,
                    twist.linear_z,
                    twist.angular_x,
                    twist.angular_y,
                    twist.angular_z,
                );
            }
            if let Some(rec) = viewer {
                rec.log("dora/cmd_vel/linear/x", &Scalars::single(twist.linear_x as f64))?;
                rec.log("dora/cmd_vel/angular/z", &Scalars::single(twist.angular_z as f64))?;
            }
        }
        other => {
            log::debug!("[receiver] ignoring unknown input '{other}'");
        }
    }
    Ok(())
}

fn publish_cmd_vel(
    node: &mut DoraNode,
    output: &DataId,
    started: Instant,
) -> eyre::Result<()> {
    let timestamp_ns = started.elapsed().as_nanos() as i64;
    let twist = ArrowCmdVel {
        linear_x: CMD_VEL_LINEAR_X,
        linear_y: 0.0,
        linear_z: 0.0,
        angular_x: 0.0,
        angular_y: 0.0,
        angular_z: CMD_VEL_ANGULAR_Z,
        timestamp_ns,
    };
    let batch = to_record_batch(&twist)?;
    let array = StructArray::from(batch);
    node.send_output(output.clone(), MetadataParameters::default(), array)?;
    Ok(())
}

fn open_rerun_sink() -> eyre::Result<Option<RecordingStream>> {
    let Ok(addr) = std::env::var(RERUN_GRPC_ADDR_ENV) else {
        return Ok(None);
    };
    if addr.is_empty() {
        return Ok(None);
    }
    log::info!("[receiver] forwarding scalars + pointcloud to rerun at {addr}");
    let url = format!("rerun+http://{addr}/proxy");
    let rec = RecordingStreamBuilder::new(RERUN_APP_ID).connect_grpc_opts(url)?;
    Ok(Some(rec))
}

fn log_pointcloud_to_rerun(
    rec: &RecordingStream,
    cloud: &isaac_sim_arrow::lidar::pointcloud::LidarPointCloudOwned,
) -> eyre::Result<()> {
    let n = cloud.points.len() / 3;
    let positions: Vec<[f32; 3]> = (0..n)
        .map(|i| {
            [
                cloud.points[3 * i],
                cloud.points[3 * i + 1],
                cloud.points[3 * i + 2],
            ]
        })
        .collect();
    rec.log("dora/lidar_pointcloud", &Points3D::new(positions))?;
    Ok(())
}

fn log_rgb_to_rerun(
    rec: &RecordingStream,
    rgb: &isaac_sim_arrow::camera::rgb::CameraRgbOwned,
) -> eyre::Result<()> {
    if rgb.width <= 0 || rgb.height <= 0 {
        return Ok(());
    }
    let expected = (rgb.width as usize) * (rgb.height as usize) * 3;
    if rgb.pixels.len() != expected {
        return Ok(());
    }
    let img = Image::from_rgb24(rgb.pixels.clone(), [rgb.width as u32, rgb.height as u32]);
    rec.log("dora/camera_rgb", &img)?;
    Ok(())
}

fn log_depth_to_rerun(
    rec: &RecordingStream,
    depth: &isaac_sim_arrow::camera::depth::CameraDepthOwned,
) -> eyre::Result<()> {
    if depth.width <= 0 || depth.height <= 0 {
        return Ok(());
    }
    let expected = (depth.width as usize) * (depth.height as usize);
    if depth.depths.len() != expected {
        return Ok(());
    }
    let bytes: &[u8] = bytemuck::cast_slice(&depth.depths);
    let img = DepthImage::from_data_type_and_bytes(
        bytes,
        [depth.width as u32, depth.height as u32],
        ChannelDatatype::F32,
    )
    .with_meter(1.0);
    rec.log("dora/camera_depth", &img)?;
    Ok(())
}

fn bump(counts: &[AtomicU64; 8], idx: usize) -> u64 {
    counts[idx].fetch_add(1, Ordering::Relaxed)
}

fn first_or_periodic(seen: u64) -> bool {
    seen == 0 || seen.is_multiple_of(50)
}

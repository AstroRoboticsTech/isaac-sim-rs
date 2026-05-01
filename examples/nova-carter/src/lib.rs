// SPDX-License-Identifier: MPL-2.0
//! Bridge runner the Carb plugin dlopens via
//! `ISAAC_SIM_RS_RERUN_RUNNER`. Sensor entities are organized as
//! children of a single `chassis` entity so a Transform3D logged at
//! `chassis` (from the odometry publisher) moves all sensors with the
//! robot — Carter visibly drives through the warehouse with its
//! sensor frustums and scans attached.
//!
//! Camera info is logged to the same entity as the RGB image
//! (`chassis/camera_rgb`) so the rerun viewer projects the RGB frame
//! inside the pinhole frustum. Depth is intentionally on a sibling
//! entity and gets its own panel without a frustum.
//!
//! As a Phase-4 demo this runner also drives Carter: it spawns a
//! background thread that publishes a synthetic Twist into the
//! cmd_vel producer registry (constant linear + small yaw rate so the
//! robot crawls in a wide arc). The C++ ApplyCmdVelFromRust node polls
//! the latest value each OG tick and feeds it into the differential
//! controller wired up in `drive.py`.

use std::thread;
use std::time::{Duration, Instant};

use isaac_sim_bridge::{
    register_cmd_vel_producer, CameraDepth, CameraInfo, CameraRgb, CmdVel, Imu, LidarFlatScan,
    LidarPointCloud, Odometry,
};
use isaac_sim_rerun::Viewer;

// Carter's articulation root sits at the chassis_link rigid body —
// not the top-level Carter prim. IsaacComputeOdometry rejects the
// latter ("not a valid rigid body or articulation root").
const CHASSIS_PRIM: &str = "/Root/World/Carter/chassis_link";
const LIDAR_2D_PRIM: &str = "/Root/World/Carter/chassis_link/lidar_2d";
const LIDAR_3D_PRIM: &str = "/Root/World/Carter/chassis_link/sensors/XT_32/PandarXT_32_10hz";
const CAMERA_PRIM: &str = "/Root/World/Carter/chassis_link/camera_rgb";
const IMU_PRIM: &str = "/Root/World/Carter/chassis_link/imu";
// Producer registry key for cmd_vel — must match `CMD_VEL_TARGET` in
// drive.py (the Carter root prim).
const CMD_VEL_TARGET: &str = "/Root/World/Carter";

const CMD_VEL_LINEAR_X: f32 = 0.4;
const CMD_VEL_ANGULAR_Z: f32 = 0.3;
const CMD_VEL_PUBLISH_HZ: f32 = 50.0;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_rerun_init() -> i32 {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    match try_init() {
        Ok(()) => 0,
        Err(e) => {
            log::error!("[example-nova-carter] init failed: {e}");
            -1
        }
    }
}

fn try_init() -> eyre::Result<()> {
    spawn_cmd_vel_demo();
    Viewer::new()
        .with_source(Odometry, CHASSIS_PRIM, "chassis")
        .with_source(LidarFlatScan, LIDAR_2D_PRIM, "chassis/lidar_flatscan")
        .with_source(LidarPointCloud, LIDAR_3D_PRIM, "chassis/lidar_pointcloud")
        .with_source(CameraRgb, CAMERA_PRIM, "chassis/camera_rgb")
        .with_source(CameraInfo, CAMERA_PRIM, "chassis/camera_rgb")
        .with_source(CameraDepth, CAMERA_PRIM, "chassis/camera_depth")
        .with_source(Imu, IMU_PRIM, "chassis/imu")
        .run()
}

fn spawn_cmd_vel_demo() {
    let slot = register_cmd_vel_producer(CMD_VEL_TARGET);
    let period = Duration::from_secs_f32(1.0 / CMD_VEL_PUBLISH_HZ);
    let started = Instant::now();
    log::info!(
        "[example-nova-carter] cmd_vel demo: target='{CMD_VEL_TARGET}' \
         linear_x={CMD_VEL_LINEAR_X} angular_z={CMD_VEL_ANGULAR_Z} \
         @ {CMD_VEL_PUBLISH_HZ:.0} Hz"
    );
    thread::Builder::new()
        .name("cmd-vel-demo".into())
        .spawn(move || loop {
            let timestamp_ns = started.elapsed().as_nanos() as i64;
            slot.publish(CmdVel {
                linear_x: CMD_VEL_LINEAR_X,
                linear_y: 0.0,
                linear_z: 0.0,
                angular_x: 0.0,
                angular_y: 0.0,
                angular_z: CMD_VEL_ANGULAR_Z,
                timestamp_ns,
            });
            thread::sleep(period);
        })
        .expect("spawn cmd-vel demo thread");
}

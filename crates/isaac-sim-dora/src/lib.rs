//! dora-rs adapter for the isaac-sim-rs SDK.
//!
//! Generic transport adapter, not a single-sensor binding. Each sensor
//! domain the bridge exposes lives in its own sub-module and registers
//! a consumer that converts the data to an Apache Arrow `RecordBatch`
//! (via `isaac-sim-arrow`) and emits it on a dora node output.

pub mod camera;
pub mod cmd_vel;
pub mod dispatch;
pub mod ffi;
pub mod imu;
pub mod lidar;
pub mod odometry;
pub mod sensor;
pub mod subscribe;

pub use sensor::DoraPublish;

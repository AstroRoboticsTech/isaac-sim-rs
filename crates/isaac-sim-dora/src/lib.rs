// SPDX-License-Identifier: MPL-2.0
#![cfg_attr(docsrs, feature(doc_cfg))]
//! dora-rs adapter for the isaac-sim-rs SDK.
//!
//! Generic transport adapter, not a single-sensor binding. Each sensor
//! domain the bridge exposes lives in its own sub-module and registers
//! a consumer that converts the data to an Apache Arrow `RecordBatch`
//! (via `isaac-sim-arrow`) and emits it on a dora node output.

#![warn(missing_docs)]

mod camera;
#[doc(hidden)]
pub mod cmd_vel;
mod dispatch;
mod imu;
mod lidar;
mod odometry;
mod sensor;
pub mod subscribe;

#[doc(hidden)]
pub mod ffi;

pub use sensor::DoraPublish;

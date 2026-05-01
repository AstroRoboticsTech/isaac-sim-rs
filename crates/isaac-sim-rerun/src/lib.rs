// SPDX-License-Identifier: MPL-2.0
#![cfg_attr(docsrs, feature(doc_cfg))]
//! Rerun viewer adapter for the Isaac Sim Rust SDK.
//!
//! The `Viewer` builder registers bridge consumers that forward sensor frames
//! to a rerun `RecordingStream` over gRPC. Sensor selection happens at build
//! time via `with_source::<S>()`.

#![warn(missing_docs)]

mod camera;
mod cmd_vel;
mod dispatch;
mod imu;
mod lidar;
mod odometry;
mod sensor;
pub mod viewer;

pub use sensor::RerunRender;
pub use viewer::Viewer;

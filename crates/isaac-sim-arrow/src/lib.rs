// SPDX-License-Identifier: MPL-2.0
#![cfg_attr(docsrs, feature(doc_cfg))]
//! Apache Arrow conversion utilities for Isaac Sim sensor data.
//!
//! Consumer-agnostic: any downstream (dora, rerun, file logger, custom bus)
//! can take the resulting `RecordBatch`. One sub-module per sensor domain.

#![warn(missing_docs)]

pub mod camera;
pub mod cmd_vel;
pub mod imu;
pub mod lidar;
pub mod odometry;

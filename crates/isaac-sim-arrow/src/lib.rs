//! Apache Arrow conversion utilities for Isaac Sim sensor data.
//!
//! Consumer-agnostic: any downstream (dora, rerun, file logger, custom bus)
//! can take the resulting `RecordBatch`. One sub-module per sensor domain.

pub mod camera;
pub mod lidar;

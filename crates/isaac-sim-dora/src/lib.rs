//! dora-rs adapter for the isaac-sim-rs SDK.
//!
//! Generic transport adapter, not a single-sensor binding. Each
//! sensor + annotator the bridge exposes lives in its own sub-module
//! and registers a consumer that converts the data to an Apache
//! Arrow `RecordBatch` (via `isaac-sim-arrow`) and emits it on a dora
//! node output. LiDAR FlatScan is the first; pointcloud, scan
//! buffer, camera, IMU, and others follow the same shape.

pub mod ffi;
pub mod lidar_flatscan;

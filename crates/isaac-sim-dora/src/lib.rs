//! dora-rs adapter for the isaac-sim-rs SDK.
//!
//! Generic transport adapter, not a single-sensor binding. Each
//! sensor + annotator the bridge exposes lives in its own sub-module
//! and registers a consumer that converts the data to an Apache
//! Arrow `RecordBatch` (via `isaac-sim-arrow`) and emits it on a dora
//! node output. LiDAR FlatScan + LiDAR PointCloud share a single
//! `DoraNode` (Arc<Mutex>) so both can co-exist on the same dora
//! source. Camera, IMU, and others follow the same shape.

pub mod ffi;
pub mod lidar_flatscan;
pub mod lidar_pointcloud;

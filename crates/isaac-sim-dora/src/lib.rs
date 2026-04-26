//! dora-rs adapter for the isaac-sim-rs SDK.
//!
//! Generic transport adapter, not a single-sensor binding. Each
//! sensor family the bridge exposes lives in its own sub-module
//! and registers a consumer that converts the sensor data to an
//! Apache Arrow `RecordBatch` (via `isaac-sim-arrow`) and emits it
//! on a dora node output. LiDAR is the first; camera, IMU,
//! articulation feedback, and others follow.

pub mod lidar;

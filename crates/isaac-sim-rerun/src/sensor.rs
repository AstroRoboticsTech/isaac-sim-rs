// SPDX-License-Identifier: MPL-2.0
use isaac_sim_bridge::Sensor;
use rerun::RecordingStream;

/// Per-sensor trait that knows how to register a rerun publisher
/// for that sensor type.
///
/// Each sensor module provides one impl, so the `Viewer` builder can
/// stay generic over `S: RerunRender` instead of growing a per-sensor
/// `with_<sensor>` method on every new sensor.
pub trait RerunRender: Sensor {
    /// Register a bridge consumer that logs sensor frames as rerun primitives
    /// on `entity_path`, filtered to frames arriving from `source`.
    fn register(rec: RecordingStream, source: String, entity_path: String);
}

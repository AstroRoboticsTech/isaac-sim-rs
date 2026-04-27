/// Type-level identifier for a sensor domain.
///
/// Adapters use this trait to derive per-sensor wiring (env-var names,
/// registry keys, log labels) from one canonical name without each new
/// sensor adding a string constant to every adapter.
///
/// Implemented by ZST markers in each sensor module (e.g. `LidarFlatScan`,
/// `LidarPointCloud`). Adapters layer their own per-sensor trait on top
/// (`RerunRender`, `DoraPublish`) keyed on these markers.
pub trait Sensor: 'static {
    const NAME: &'static str;
}

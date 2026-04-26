use crate::consumers::dispatch_lidar_pointcloud;
use crate::ffi::LidarPointCloudMeta;

pub fn forward_lidar_pointcloud(
    azimuth: &[f32],
    elevation: &[f32],
    distance: &[f32],
    intensity: &[f32],
    meta: &LidarPointCloudMeta,
) {
    log::debug!(
        "[isaac-sim-rs] forward_lidar_pointcloud: n={} (az={}, el={}, dist={}, intens={})",
        meta.num_points,
        azimuth.len(),
        elevation.len(),
        distance.len(),
        intensity.len()
    );
    dispatch_lidar_pointcloud(azimuth, elevation, distance, intensity, meta);
}

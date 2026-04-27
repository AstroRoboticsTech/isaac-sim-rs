use crate::consumers::dispatch_lidar_pointcloud;
use crate::ffi::LidarPointCloudMeta;

pub fn forward_lidar_pointcloud(source_id: &str, points: &[f32], meta: &LidarPointCloudMeta) {
    log::debug!(
        "[isaac-sim-rs] forward_lidar_pointcloud: source='{}' n={} (floats={}, width={}, height={})",
        source_id,
        meta.num_points,
        points.len(),
        meta.width,
        meta.height
    );
    dispatch_lidar_pointcloud(source_id, points, meta);
}

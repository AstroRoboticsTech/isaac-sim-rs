// SPDX-License-Identifier: MPL-2.0
use std::sync::OnceLock;

use crate::channel::{channel_singleton, Channel};
use crate::ffi::LidarPointCloudMeta;
use crate::sensor::Sensor;

/// Type-level marker for the 3D RTX LiDAR PointCloud sensor channel.
pub struct LidarPointCloud;

impl Sensor for LidarPointCloud {
    const NAME: &'static str = "lidar_pointcloud";
}

pub type Callback = Box<dyn Fn(&str, &[f32], &LidarPointCloudMeta) + Send + Sync + 'static>;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_channel_lidar_pointcloud() -> *const Channel<Callback> {
    static SLOT: OnceLock<Box<Channel<Callback>>> = OnceLock::new();
    channel_singleton(&SLOT)
}

fn channel() -> &'static Channel<Callback> {
    unsafe { &*isaac_sim_bridge_channel_lidar_pointcloud() }
}

/// Register a callback to receive every `LidarPointCloud` frame the bridge
/// dispatches. The closure runs on the bridge thread; keep it bounded.
pub fn register_lidar_pointcloud_consumer<F>(cb: F)
where
    F: Fn(&str, &[f32], &LidarPointCloudMeta) + Send + Sync + 'static,
{
    channel().register(Box::new(cb));
}

/// Fan out a single `LidarPointCloud` frame to all registered consumers.
pub fn dispatch_lidar_pointcloud(source_id: &str, points: &[f32], meta: &LidarPointCloudMeta) {
    channel().for_each(|cb| cb(source_id, points, meta));
}

/// Number of currently registered `LidarPointCloud` consumers.
pub fn lidar_pointcloud_consumer_count() -> usize {
    channel().count()
}

/// Entry point called by the C++ bridge on each OmniGraph tick.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn registered_consumer_receives_dispatch_with_source() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = lidar_pointcloud_consumer_count();

        register_lidar_pointcloud_consumer(move |src, points, meta| {
            assert_eq!(src, "/World/Carter/Lidar3D");
            assert_eq!(points.len(), 9);
            assert_eq!(meta.num_points, 3);
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(lidar_pointcloud_consumer_count(), n_baseline + 1);

        let points = [
            1.0_f32, 0.0, 0.0, //
            0.0, 1.0, 0.0, //
            0.0, 0.0, 1.0,
        ];
        let meta = LidarPointCloudMeta {
            num_points: 3,
            width: 3,
            height: 1,
        };
        dispatch_lidar_pointcloud("/World/Carter/Lidar3D", &points, &meta);

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

use crate::channel::Channel;
use crate::ffi::LidarPointCloudMeta;

pub type Callback = Box<dyn Fn(&str, &[f32], &LidarPointCloudMeta) + Send + Sync + 'static>;

static CHANNEL: Channel<Callback> = Channel::new();

pub fn register_lidar_pointcloud_consumer<F>(cb: F)
where
    F: Fn(&str, &[f32], &LidarPointCloudMeta) + Send + Sync + 'static,
{
    CHANNEL.register(Box::new(cb));
}

pub fn dispatch_lidar_pointcloud(source_id: &str, points: &[f32], meta: &LidarPointCloudMeta) {
    CHANNEL.for_each(|cb| cb(source_id, points, meta));
}

pub fn lidar_pointcloud_consumer_count() -> usize {
    CHANNEL.count()
}

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

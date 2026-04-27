use crate::channel::Channel;
use crate::ffi::LidarFlatScanMeta;
use crate::sensor::Sensor;

/// Type-level marker for the 2D RTX LiDAR FlatScan sensor channel.
pub struct LidarFlatScan;

impl Sensor for LidarFlatScan {
    const NAME: &'static str = "lidar_flatscan";
}

pub type Callback = Box<dyn Fn(&str, &[f32], &[u8], &LidarFlatScanMeta) + Send + Sync + 'static>;

static CHANNEL: Channel<Callback> = Channel::new();

pub fn register_lidar_flatscan_consumer<F>(cb: F)
where
    F: Fn(&str, &[f32], &[u8], &LidarFlatScanMeta) + Send + Sync + 'static,
{
    CHANNEL.register(Box::new(cb));
}

pub fn dispatch_lidar_flatscan(
    source_id: &str,
    scan: &[f32],
    intensities: &[u8],
    meta: &LidarFlatScanMeta,
) {
    CHANNEL.for_each(|cb| cb(source_id, scan, intensities, meta));
}

pub fn lidar_flatscan_consumer_count() -> usize {
    CHANNEL.count()
}

pub fn forward_lidar_flatscan(
    source_id: &str,
    scan: &[f32],
    intensities: &[u8],
    meta: &LidarFlatScanMeta,
) {
    let depth_min = scan.iter().copied().fold(f32::INFINITY, f32::min);
    let depth_max = scan.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    log::debug!(
        "[isaac-sim-rs] forward_lidar_flatscan: source='{}' scan_n={}, intensity_n={}, fov={:.1}°, observed_depth=[{:.3},{:.3}]m",
        source_id,
        scan.len(),
        intensities.len(),
        meta.horizontal_fov,
        depth_min,
        depth_max
    );

    dispatch_lidar_flatscan(source_id, scan, intensities, meta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fake_meta() -> LidarFlatScanMeta {
        LidarFlatScanMeta {
            horizontal_fov: 270.0,
            horizontal_resolution: 0.25,
            azimuth_min: -135.0,
            azimuth_max: 135.0,
            depth_min: 0.1,
            depth_max: 30.0,
            num_rows: 1,
            num_cols: 4,
            rotation_rate: 10.0,
        }
    }

    #[test]
    fn registered_consumer_receives_dispatch_with_source() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = lidar_flatscan_consumer_count();

        register_lidar_flatscan_consumer(move |src, scan, _intens, meta| {
            assert_eq!(src, "/World/Lidar2D");
            assert_eq!(scan.len(), 4);
            assert!((meta.horizontal_fov - 270.0).abs() < 1e-6);
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(lidar_flatscan_consumer_count(), n_baseline + 1);

        let scan = [0.5_f32, 1.2, 2.7, 3.0];
        let intensities = [10_u8, 50, 200, 100];
        dispatch_lidar_flatscan("/World/Lidar2D", &scan, &intensities, &fake_meta());

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

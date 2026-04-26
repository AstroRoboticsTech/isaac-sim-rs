use std::sync::{Mutex, OnceLock};

use crate::ffi::LidarFlatScanMeta;

type LidarFlatScanCallback = Box<dyn Fn(&[f32], &[u8], &LidarFlatScanMeta) + Send + Sync + 'static>;

static LIDAR_FLATSCAN_CONSUMERS: OnceLock<Mutex<Vec<LidarFlatScanCallback>>> = OnceLock::new();

fn lidar_flatscan_registry() -> &'static Mutex<Vec<LidarFlatScanCallback>> {
    LIDAR_FLATSCAN_CONSUMERS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn register_lidar_flatscan_consumer<F>(cb: F)
where
    F: Fn(&[f32], &[u8], &LidarFlatScanMeta) + Send + Sync + 'static,
{
    lidar_flatscan_registry().lock().unwrap().push(Box::new(cb));
}

pub fn dispatch_lidar_flatscan(scan: &[f32], intensities: &[u8], meta: &LidarFlatScanMeta) {
    let consumers = lidar_flatscan_registry().lock().unwrap();
    for cb in consumers.iter() {
        cb(scan, intensities, meta);
    }
}

pub fn lidar_flatscan_consumer_count() -> usize {
    lidar_flatscan_registry().lock().unwrap().len()
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
    fn registered_consumer_receives_dispatched_scan() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = lidar_flatscan_consumer_count();

        register_lidar_flatscan_consumer(move |scan, _intensities, meta| {
            assert_eq!(scan.len(), 4);
            assert!((meta.horizontal_fov - 270.0).abs() < 1e-6);
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(lidar_flatscan_consumer_count(), n_baseline + 1);

        let scan = [0.5_f32, 1.2, 2.7, 3.0];
        let intensities = [10_u8, 50, 200, 100];
        dispatch_lidar_flatscan(&scan, &intensities, &fake_meta());

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

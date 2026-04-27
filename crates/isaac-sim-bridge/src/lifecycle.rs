use std::sync::Once;

use crate::consumers::{register_lidar_flatscan_consumer, register_lidar_pointcloud_consumer};

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .try_init();
        log::info!("[isaac-sim-rs] init: env_logger up");
        register_default_consumers();
    });
}

fn register_default_consumers() {
    register_lidar_flatscan_consumer(|src, scan, intensities, meta| {
        let depth_min = scan.iter().copied().fold(f32::INFINITY, f32::min);
        let depth_max = scan.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        log::info!(
            "[isaac-sim-rs:default] lidar_flatscan src='{}' scan_n={} intensity_n={} fov={:.1}° rate={:.1}Hz observed_depth=[{:.3},{:.3}]m",
            src,
            scan.len(),
            intensities.len(),
            meta.horizontal_fov,
            meta.rotation_rate,
            depth_min,
            depth_max
        );
    });

    register_lidar_pointcloud_consumer(|src, points, meta| {
        let mut zmin = f32::INFINITY;
        let mut zmax = f32::NEG_INFINITY;
        for chunk in points.chunks_exact(3) {
            let z = chunk[2];
            if z < zmin {
                zmin = z;
            }
            if z > zmax {
                zmax = z;
            }
        }
        log::info!(
            "[isaac-sim-rs:default] lidar_pointcloud src='{}' n={} floats={} width={} height={} observed_z=[{:.3},{:.3}]m",
            src,
            meta.num_points,
            points.len(),
            meta.width,
            meta.height,
            zmin,
            zmax
        );
    });
}

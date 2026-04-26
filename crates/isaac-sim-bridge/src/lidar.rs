use crate::consumers::dispatch_lidar_scan;
use crate::ffi::ScanMeta;

pub fn forward_lidar_scan(scan: &[f32], intensities: &[u8], meta: &ScanMeta) {
    let depth_min = scan.iter().copied().fold(f32::INFINITY, f32::min);
    let depth_max = scan.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    log::debug!(
        "[isaac-sim-rs] forward_lidar_scan: scan_n={}, intensity_n={}, fov={:.1}°, res={:.3}°, az=[{:.2},{:.2}]°, depth=[{:.2},{:.2}]m, rows={}, cols={}, rate={:.1}Hz, observed_depth=[{:.3},{:.3}]m",
        scan.len(),
        intensities.len(),
        meta.horizontal_fov,
        meta.horizontal_resolution,
        meta.azimuth_min,
        meta.azimuth_max,
        meta.depth_min,
        meta.depth_max,
        meta.num_rows,
        meta.num_cols,
        meta.rotation_rate,
        depth_min,
        depth_max
    );

    dispatch_lidar_scan(scan, intensities, meta);
}

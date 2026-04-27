use crate::consumers::dispatch_lidar_flatscan;
use crate::ffi::LidarFlatScanMeta;

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

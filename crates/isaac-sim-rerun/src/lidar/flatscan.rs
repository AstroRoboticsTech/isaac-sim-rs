use isaac_sim_bridge::{register_lidar_flatscan_consumer, LidarFlatScanMeta};
use rerun::{Color, Points3D, RecordingStream};

pub fn flatscan_to_points(meta: &LidarFlatScanMeta, depths: &[f32]) -> Vec<[f32; 3]> {
    let n = depths.len();
    let mut positions = Vec::with_capacity(n);
    for (i, &r) in depths.iter().enumerate() {
        let t = if n > 1 {
            i as f32 / (n - 1) as f32
        } else {
            0.0
        };
        let az_deg = meta.azimuth_min + t * (meta.azimuth_max - meta.azimuth_min);
        let az = az_deg.to_radians();
        positions.push([r * az.cos(), r * az.sin(), 0.0]);
    }
    positions
}

pub fn log_lidar_flatscan(
    rec: &RecordingStream,
    entity_path: &str,
    depths: &[f32],
    intensities: &[u8],
    meta: &LidarFlatScanMeta,
) -> eyre::Result<()> {
    if depths.is_empty() {
        return Ok(());
    }
    let positions = flatscan_to_points(meta, depths);
    let mut archetype = Points3D::new(positions);
    if intensities.len() == depths.len() {
        let colors: Vec<Color> = intensities
            .iter()
            .map(|&v| Color::from_rgb(v, v, v))
            .collect();
        archetype = archetype.with_colors(colors);
    }
    rec.log(entity_path, &archetype)?;
    Ok(())
}

pub fn register_rerun_lidar_flatscan_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    register_lidar_flatscan_consumer(move |src, scan, intensities, meta| {
        if src != source {
            return;
        }
        if let Err(e) = log_lidar_flatscan(&rec, &entity_path, scan, intensities, meta) {
            log::warn!("[isaac-sim-rerun] log failed for '{source}' -> '{entity_path}': {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_270deg() -> LidarFlatScanMeta {
        LidarFlatScanMeta {
            horizontal_fov: 270.0,
            horizontal_resolution: 90.0,
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
    fn flatscan_to_points_places_beams_on_unit_circle() {
        use std::f32::consts::FRAC_1_SQRT_2;

        let depths = [1.0_f32, 1.0, 1.0, 1.0];
        let positions = flatscan_to_points(&meta_270deg(), &depths);
        assert_eq!(positions.len(), 4);

        let p0 = positions[0];
        assert!((p0[0] - (-FRAC_1_SQRT_2)).abs() < 1e-5, "p0.x={}", p0[0]);
        assert!((p0[1] - (-FRAC_1_SQRT_2)).abs() < 1e-5, "p0.y={}", p0[1]);
        assert_eq!(p0[2], 0.0);

        let p1 = positions[1];
        assert!((p1[0] - FRAC_1_SQRT_2).abs() < 1e-5, "p1.x={}", p1[0]);
        assert!((p1[1] - (-FRAC_1_SQRT_2)).abs() < 1e-5, "p1.y={}", p1[1]);

        let p3 = positions[3];
        assert!((p3[0] - (-FRAC_1_SQRT_2)).abs() < 1e-5, "p3.x={}", p3[0]);
        assert!((p3[1] - FRAC_1_SQRT_2).abs() < 1e-5, "p3.y={}", p3[1]);
    }

    #[test]
    fn log_lidar_flatscan_writes_to_memory_sink() {
        let depths = [1.0_f32, 1.0, 1.0, 1.0];
        let intensities = [10_u8, 80, 160, 240];
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_lidar_flatscan(&rec, "lidar/scan", &depths, &intensities, &meta_270deg()).expect("log");
        rec.flush_blocking().expect("flush");
        let msgs = storage.take();
        assert!(!msgs.is_empty());
    }
}

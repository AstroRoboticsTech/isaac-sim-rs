use isaac_sim_bridge::{register_lidar_pointcloud_consumer, LidarPointCloudMeta};
use rerun::{Color, Points3D, RecordingStream};

pub fn pointcloud_to_xyz(azimuth: &[f32], elevation: &[f32], distance: &[f32]) -> Vec<[f32; 3]> {
    let n = azimuth.len().min(elevation.len()).min(distance.len());
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let az = azimuth[i];
        let el = elevation[i];
        let d = distance[i];
        let cos_el = el.cos();
        positions.push([d * cos_el * az.cos(), d * cos_el * az.sin(), d * el.sin()]);
    }
    positions
}

pub fn log_lidar_pointcloud(
    rec: &RecordingStream,
    entity_path: &str,
    azimuth: &[f32],
    elevation: &[f32],
    distance: &[f32],
    intensity: &[f32],
    _meta: &LidarPointCloudMeta,
) -> eyre::Result<()> {
    let positions = pointcloud_to_xyz(azimuth, elevation, distance);
    if positions.is_empty() {
        return Ok(());
    }
    let mut archetype = Points3D::new(positions.clone());
    if intensity.len() == positions.len() {
        let colors: Vec<Color> = intensity
            .iter()
            .map(|&v| {
                let g = (v.clamp(0.0, 1.0) * 255.0) as u8;
                Color::from_rgb(g, g, g)
            })
            .collect();
        archetype = archetype.with_colors(colors);
    }
    rec.log(entity_path, &archetype)?;
    Ok(())
}

pub fn register_rerun_lidar_pointcloud_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    register_lidar_pointcloud_consumer(move |az, el, dist, intens, meta| {
        if let Err(e) = log_lidar_pointcloud(&rec, &entity_path, az, el, dist, intens, meta) {
            log::warn!("[isaac-sim-rerun] log failed for '{source}' -> '{entity_path}': {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointcloud_to_xyz_zero_elevation_lies_on_xy_plane() {
        let azimuth = [0.0_f32, std::f32::consts::FRAC_PI_2, std::f32::consts::PI];
        let elevation = [0.0_f32, 0.0, 0.0];
        let distance = [1.0_f32, 2.0, 3.0];
        let positions = pointcloud_to_xyz(&azimuth, &elevation, &distance);
        assert_eq!(positions.len(), 3);
        // (az=0, d=1) → (1, 0, 0)
        assert!((positions[0][0] - 1.0).abs() < 1e-5);
        assert!(positions[0][1].abs() < 1e-5);
        assert!(positions[0][2].abs() < 1e-5);
        // (az=π/2, d=2) → (0, 2, 0)
        assert!(positions[1][0].abs() < 1e-5);
        assert!((positions[1][1] - 2.0).abs() < 1e-5);
        // (az=π, d=3) → (-3, 0, 0)
        assert!((positions[2][0] - (-3.0)).abs() < 1e-5);
        assert!(positions[2][1].abs() < 1e-5);
    }

    #[test]
    fn log_lidar_pointcloud_writes_to_memory_sink() {
        let azimuth = [0.0_f32, 1.0, 2.0];
        let elevation = [0.1_f32, 0.2, 0.3];
        let distance = [5.0_f32, 6.0, 7.0];
        let intensity = [0.1_f32, 0.5, 0.9];
        let meta = LidarPointCloudMeta { num_points: 3 };
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_lidar_pointcloud(
            &rec,
            "lidar/pointcloud",
            &azimuth,
            &elevation,
            &distance,
            &intensity,
            &meta,
        )
        .expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }
}

use std::sync::Arc;

use isaac_sim_bridge::{register_lidar_pointcloud_consumer, LidarPointCloud, LidarPointCloudMeta};
use rerun::{Points3D, RecordingStream};

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::RerunRender;

struct Frame {
    points: Arc<[f32]>,
    meta: LidarPointCloudMeta,
}

impl RerunRender for LidarPointCloud {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_lidar_pointcloud_publisher(rec, source, entity_path);
    }
}

pub fn log_lidar_pointcloud(
    rec: &RecordingStream,
    entity_path: &str,
    points: &[f32],
    _meta: &LidarPointCloudMeta,
) -> eyre::Result<()> {
    let n = points.len() / 3;
    if n == 0 {
        return Ok(());
    }
    let positions: &[[f32; 3]] = bytemuck::cast_slice(&points[..n * 3]);
    rec.log(entity_path, &Points3D::new(positions.iter().copied()))?;
    Ok(())
}

pub fn register_rerun_lidar_pointcloud_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    let (slot, wake) = LatestSlot::<Frame>::new();
    let entity_path_for_drain = entity_path.clone();
    let source_for_drain = source.clone();
    let drain_name = format!("rerun-drain-lidar_pointcloud:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) =
            log_lidar_pointcloud(&rec, &entity_path_for_drain, &frame.points, &frame.meta)
        {
            log::warn!(
                "[isaac-sim-rerun] log failed for '{source_for_drain}' -> '{entity_path_for_drain}': {e}"
            );
        }
    });
    register_lidar_pointcloud_consumer(move |src, points, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame {
            points: Arc::from(points),
            meta: *meta,
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_lidar_pointcloud_writes_to_memory_sink() {
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
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_lidar_pointcloud(&rec, "lidar/pointcloud", &points, &meta).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }
}

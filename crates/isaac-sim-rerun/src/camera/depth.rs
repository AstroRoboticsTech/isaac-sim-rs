use std::sync::Arc;

use isaac_sim_bridge::{register_camera_depth_consumer, CameraDepth, CameraDepthMeta};
use rerun::{datatypes::ChannelDatatype, DepthImage, RecordingStream};

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::RerunRender;

struct Frame {
    depths: Arc<[f32]>,
    meta: CameraDepthMeta,
}

impl RerunRender for CameraDepth {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_camera_depth_publisher(rec, source, entity_path);
    }
}

pub fn log_camera_depth(
    rec: &RecordingStream,
    entity_path: &str,
    depths: &[f32],
    meta: &CameraDepthMeta,
) -> eyre::Result<()> {
    if meta.width <= 0 || meta.height <= 0 {
        return Ok(());
    }
    let expected = (meta.width as usize) * (meta.height as usize);
    if depths.len() != expected {
        log::warn!(
            "[isaac-sim-rerun] camera_depth pixel/dimension mismatch for '{entity_path}': \
             samples={} width={} height={} (expected {expected})",
            depths.len(),
            meta.width,
            meta.height
        );
        return Ok(());
    }
    // The cpp side properly extracts the texture into a linear f32
    // buffer (cudaMemcpy2DFromArrayAsync), so we forward raw metres
    // here. Isaac uses +inf for "no hit" / past-far-clip; rerun's
    // DepthImage handles non-finite values transparently.
    let bytes: &[u8] = bytemuck::cast_slice(depths);
    let img = DepthImage::from_data_type_and_bytes(
        bytes,
        [meta.width as u32, meta.height as u32],
        ChannelDatatype::F32,
    )
    .with_meter(1.0);
    rec.log(entity_path, &img)?;
    Ok(())
}

pub fn register_rerun_camera_depth_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    let (slot, wake) = LatestSlot::<Frame>::new();
    let entity_path_for_drain = entity_path.clone();
    let source_for_drain = source.clone();
    let drain_name = format!("rerun-drain-camera_depth:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if let Err(e) = log_camera_depth(&rec, &entity_path_for_drain, &frame.depths, &frame.meta) {
            log::warn!(
                "[isaac-sim-rerun] log failed for '{source_for_drain}' -> '{entity_path_for_drain}': {e}"
            );
        }
    });
    register_camera_depth_consumer(move |src, depths, meta| {
        if !filter.matches(src) {
            return;
        }
        slot.publish(Frame {
            depths: Arc::from(depths),
            meta: *meta,
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_camera_depth_writes_to_memory_sink() {
        let depths = vec![1.5_f32; 4 * 4];
        let meta = CameraDepthMeta {
            width: 4,
            height: 4,
            fx: 0.0,
            fy: 0.0,
            cx: 0.0,
            cy: 0.0,
            timestamp_ns: 0,
        };
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_camera_depth(&rec, "scene/camera/depth", &depths, &meta).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }

    #[test]
    fn log_camera_depth_skips_mismatched_buffer() {
        let depths = vec![0.0_f32; 5]; // wrong length for 4x4
        let meta = CameraDepthMeta {
            width: 4,
            height: 4,
            fx: 0.0,
            fy: 0.0,
            cx: 0.0,
            cy: 0.0,
            timestamp_ns: 0,
        };
        let (rec, _storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        log_camera_depth(&rec, "scene/camera/depth", &depths, &meta)
            .expect("log returns Ok on mismatch");
    }
}

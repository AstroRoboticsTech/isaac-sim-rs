use isaac_sim_bridge::{register_camera_rgb_consumer, CameraRgb, CameraRgbMeta};
use rerun::{Image, RecordingStream};

use crate::sensor::RerunRender;

impl RerunRender for CameraRgb {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_camera_rgb_publisher(rec, source, entity_path);
    }
}

pub fn log_camera_rgb(
    rec: &RecordingStream,
    entity_path: &str,
    pixels: &[u8],
    meta: &CameraRgbMeta,
) -> eyre::Result<()> {
    if meta.width <= 0 || meta.height <= 0 {
        return Ok(());
    }
    let expected = (meta.width as usize) * (meta.height as usize) * 3;
    if pixels.len() != expected {
        log::warn!(
            "[isaac-sim-rerun] camera_rgb pixel/dimension mismatch for '{entity_path}': \
             pixels={} width={} height={} (expected {expected} bytes)",
            pixels.len(),
            meta.width,
            meta.height
        );
        return Ok(());
    }
    let img = Image::from_rgb24(pixels, [meta.width as u32, meta.height as u32]);
    rec.log(entity_path, &img)?;
    Ok(())
}

pub fn register_rerun_camera_rgb_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    register_camera_rgb_consumer(move |src, pixels, meta| {
        if !filter.matches(src) {
            return;
        }
        if let Err(e) = log_camera_rgb(&rec, &entity_path, pixels, meta) {
            log::warn!("[isaac-sim-rerun] log failed for '{source}' -> '{entity_path}': {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_camera_rgb_writes_to_memory_sink() {
        let pixels = vec![128_u8; 4 * 4 * 3];
        let meta = CameraRgbMeta {
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
        log_camera_rgb(&rec, "scene/camera/rgb", &pixels, &meta).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }

    #[test]
    fn log_camera_rgb_skips_mismatched_buffer() {
        let pixels = vec![0_u8; 5]; // wrong size for 4x4 RGB
        let meta = CameraRgbMeta {
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
        // The mismatched buffer must not panic and must return Ok — it's
        // dropped with a warn log so a downstream consumer can never see
        // a wrong-sized image.
        log_camera_rgb(&rec, "scene/camera/rgb", &pixels, &meta)
            .expect("log returns Ok on mismatch");
    }
}

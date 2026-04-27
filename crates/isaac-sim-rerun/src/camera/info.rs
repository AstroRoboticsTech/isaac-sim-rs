use isaac_sim_bridge::{register_camera_info_consumer, CameraInfo, CameraInfoMeta};
use rerun::{Pinhole, RecordingStream};

use crate::dispatch::{spawn_drain, LatestSlot};
use crate::sensor::RerunRender;

/// One CameraInfo snapshot owned by the drain thread. Allocates per
/// dispatch so the OG-thread callback returns immediately.
struct Frame {
    k: [f64; 9],
    width: u32,
    height: u32,
    has_k: bool,
    _meta: CameraInfoMeta,
}

impl RerunRender for CameraInfo {
    fn register(rec: RecordingStream, source: String, entity_path: String) {
        register_rerun_camera_info_publisher(rec, source, entity_path);
    }
}

/// Convert a row-major 3x3 K matrix into rerun's column-major
/// `[[f32; 3]; 3]` form. ROS / OpenCV K is `[fx, 0, cx, 0, fy, cy, 0, 0, 1]`
/// (row-major); rerun's `Pinhole::new` expects each inner array to be
/// a column, so cx/cy land in the last inner array as `[cx, cy, 1]`.
fn k_row_major_to_pinhole_columns(k: &[f64; 9]) -> [[f32; 3]; 3] {
    [
        [k[0] as f32, k[3] as f32, k[6] as f32],
        [k[1] as f32, k[4] as f32, k[7] as f32],
        [k[2] as f32, k[5] as f32, k[8] as f32],
    ]
}

pub fn log_camera_info(
    rec: &RecordingStream,
    entity_path: &str,
    k: &[f64; 9],
    width: u32,
    height: u32,
) -> eyre::Result<()> {
    if width == 0 || height == 0 {
        return Ok(());
    }
    let pinhole = Pinhole::new(k_row_major_to_pinhole_columns(k))
        .with_resolution([width as f32, height as f32]);
    rec.log(entity_path, &pinhole)?;
    Ok(())
}

pub fn register_rerun_camera_info_publisher(
    rec: RecordingStream,
    source: String,
    entity_path: String,
) {
    let filter = isaac_sim_bridge::SourceFilter::exact(source.clone());
    let (slot, wake) = LatestSlot::<Frame>::new();
    let entity_path_for_drain = entity_path.clone();
    let source_for_drain = source.clone();
    let drain_name = format!("rerun-drain-camera_info:{source}");
    let _ = spawn_drain(&drain_name, slot.clone(), wake, move |frame| {
        if !frame.has_k {
            return;
        }
        if let Err(e) = log_camera_info(
            &rec,
            &entity_path_for_drain,
            &frame.k,
            frame.width,
            frame.height,
        ) {
            log::warn!(
                "[isaac-sim-rerun] log failed for '{source_for_drain}' -> '{entity_path_for_drain}': {e}"
            );
        }
    });
    register_camera_info_consumer(move |src, frame| {
        if !filter.matches(src) {
            return;
        }
        let mut k_arr = [0.0_f64; 9];
        let has_k = frame.k.len() == 9;
        if has_k {
            k_arr.copy_from_slice(frame.k);
        }
        slot.publish(Frame {
            k: k_arr,
            width: frame.meta.width.max(0) as u32,
            height: frame.meta.height.max(0) as u32,
            has_k,
            _meta: *frame.meta,
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_k() -> [f64; 9] {
        [500.0, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0]
    }

    #[test]
    fn log_camera_info_writes_to_memory_sink() {
        let (rec, storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        let k = fake_k();
        log_camera_info(&rec, "scene/camera", &k, 640, 480).expect("log");
        rec.flush_blocking().expect("flush");
        assert!(!storage.take().is_empty());
    }

    #[test]
    fn log_camera_info_skips_zero_resolution() {
        let (rec, _storage) = rerun::RecordingStreamBuilder::new("isaac-sim-rerun-test")
            .memory()
            .expect("memory recording");
        let k = fake_k();
        // Zero resolution is the "no useful data yet" case — must not
        // panic, must not error. Rerun's memory sink has unconditional
        // setup events so storage non-emptiness is not a useful signal.
        log_camera_info(&rec, "scene/camera", &k, 0, 0).expect("log returns Ok");
    }

    #[test]
    fn k_to_pinhole_columns_matches_focal_resolution_helper() {
        // Pinhole::from_focal_length_and_resolution([fx,fy], [w,h])
        // builds a column-major matrix with cx=w/2, cy=h/2. Feeding
        // an equivalent row-major K through our converter should produce
        // the same inner arrays.
        let fx = 500.0;
        let fy = 500.0;
        let cx = 320.0;
        let cy = 240.0;
        let k = [fx, 0.0, cx, 0.0, fy, cy, 0.0, 0.0, 1.0];
        let cols = k_row_major_to_pinhole_columns(&k);
        assert_eq!(cols[0], [fx as f32, 0.0, 0.0]); // first column
        assert_eq!(cols[1], [0.0, fy as f32, 0.0]); // second column
        assert_eq!(cols[2], [cx as f32, cy as f32, 1.0]); // third column
    }
}

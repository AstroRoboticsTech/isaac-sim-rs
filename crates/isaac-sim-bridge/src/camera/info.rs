// SPDX-License-Identifier: MPL-2.0
use std::sync::OnceLock;

use crate::channel::{channel_singleton, Channel};
use crate::ffi::CameraInfoMeta;
use crate::sensor::Sensor;

/// Type-level marker for the camera-info (calibration metadata) channel.
pub struct CameraInfo;

impl Sensor for CameraInfo {
    const NAME: &'static str = "camera_info";
}

/// One camera-info dispatch. Bundles the matrix and distortion slices
/// alongside the small `CameraInfoMeta` so consumer callbacks don't
/// have to thread eight separate parameters.
#[allow(missing_docs)]
pub struct CameraInfoFrame<'a> {
    pub frame_id: &'a str,
    pub distortion_model: &'a str,
    pub projection_type: &'a str,
    pub k: &'a [f64],
    pub r: &'a [f64],
    pub p: &'a [f64],
    pub distortion: &'a [f32],
    pub meta: &'a CameraInfoMeta,
}

pub type Callback = Box<dyn Fn(&str, &CameraInfoFrame<'_>) + Send + Sync + 'static>;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_channel_camera_info() -> *const Channel<Callback> {
    static SLOT: OnceLock<Box<Channel<Callback>>> = OnceLock::new();
    channel_singleton(&SLOT)
}

fn channel() -> &'static Channel<Callback> {
    unsafe { &*isaac_sim_bridge_channel_camera_info() }
}

/// Register a callback to receive every camera-info frame the bridge dispatches.
/// The closure runs on the bridge thread; keep it bounded.
pub fn register_camera_info_consumer<F>(cb: F)
where
    F: Fn(&str, &CameraInfoFrame<'_>) + Send + Sync + 'static,
{
    channel().register(Box::new(cb));
}

/// Fan out a single camera-info frame to all registered consumers.
pub fn dispatch_camera_info(source_id: &str, frame: &CameraInfoFrame<'_>) {
    channel().for_each(|cb| cb(source_id, frame));
}

/// Number of currently registered camera-info consumers.
pub fn camera_info_consumer_count() -> usize {
    channel().count()
}

/// Entry point called by the C++ bridge on each OmniGraph tick. Bundles the
/// matrix slices into a `CameraInfoFrame` and calls `dispatch_camera_info`.
#[allow(clippy::too_many_arguments)]
pub fn forward_camera_info(
    source_id: &str,
    frame_id: &str,
    distortion_model: &str,
    projection_type: &str,
    k: &[f64],
    r: &[f64],
    p: &[f64],
    distortion: &[f32],
    meta: &CameraInfoMeta,
) {
    log::debug!(
        "[isaac-sim-rs] forward_camera_info: source='{}' frame='{}' wxh={}x{} k={} r={} p={} d={} model='{}' proj='{}'",
        source_id,
        frame_id,
        meta.width,
        meta.height,
        k.len(),
        r.len(),
        p.len(),
        distortion.len(),
        distortion_model,
        projection_type,
    );
    let frame = CameraInfoFrame {
        frame_id,
        distortion_model,
        projection_type,
        k,
        r,
        p,
        distortion,
        meta,
    };
    dispatch_camera_info(source_id, &frame);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fake_meta(w: i32, h: i32) -> CameraInfoMeta {
        CameraInfoMeta {
            width: w,
            height: h,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn registered_consumer_receives_dispatch_with_source() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = camera_info_consumer_count();

        register_camera_info_consumer(move |src, frame| {
            assert_eq!(src, "/World/Camera");
            assert_eq!(frame.frame_id, "sim_camera");
            assert_eq!(frame.k.len(), 9);
            assert_eq!(frame.p.len(), 12);
            assert_eq!(frame.meta.width, 640);
            assert_eq!(frame.meta.height, 480);
            assert_eq!(frame.distortion_model, "plumb_bob");
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(camera_info_consumer_count(), n_baseline + 1);

        let k = [500.0, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0];
        let r = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let p = [
            500.0, 0.0, 320.0, 0.0, 0.0, 500.0, 240.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ];
        let d = [0.0_f32, 0.0, 0.0, 0.0, 0.0];
        let meta = fake_meta(640, 480);
        forward_camera_info(
            "/World/Camera",
            "sim_camera",
            "plumb_bob",
            "pinhole",
            &k,
            &r,
            &p,
            &d,
            &meta,
        );

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

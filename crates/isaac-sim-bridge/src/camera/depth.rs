// SPDX-License-Identifier: MPL-2.0
use std::sync::OnceLock;

use crate::channel::{channel_singleton, Channel};
use crate::ffi::CameraDepthMeta;
use crate::sensor::Sensor;

/// Type-level marker for the per-pixel depth (metres, float32) camera channel.
pub struct CameraDepth;

impl Sensor for CameraDepth {
    const NAME: &'static str = "camera_depth";
}

pub type Callback = Box<dyn Fn(&str, &[f32], &CameraDepthMeta) + Send + Sync + 'static>;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_channel_camera_depth() -> *const Channel<Callback> {
    static SLOT: OnceLock<Box<Channel<Callback>>> = OnceLock::new();
    channel_singleton(&SLOT)
}

fn channel() -> &'static Channel<Callback> {
    unsafe { &*isaac_sim_bridge_channel_camera_depth() }
}

/// Register a callback to receive every depth camera frame the bridge dispatches.
/// The closure runs on the bridge thread; keep it bounded.
pub fn register_camera_depth_consumer<F>(cb: F)
where
    F: Fn(&str, &[f32], &CameraDepthMeta) + Send + Sync + 'static,
{
    channel().register(Box::new(cb));
}

/// Fan out a single depth camera frame to all registered consumers.
pub fn dispatch_camera_depth(source_id: &str, depths: &[f32], meta: &CameraDepthMeta) {
    channel().for_each(|cb| cb(source_id, depths, meta));
}

/// Number of currently registered depth camera consumers.
pub fn camera_depth_consumer_count() -> usize {
    channel().count()
}

/// Entry point called by the C++ bridge on each OmniGraph tick.
pub fn forward_camera_depth(source_id: &str, depths: &[f32], meta: &CameraDepthMeta) {
    log::debug!(
        "[isaac-sim-rs] forward_camera_depth: source='{}' wxh={}x{} samples={}",
        source_id,
        meta.width,
        meta.height,
        depths.len()
    );
    dispatch_camera_depth(source_id, depths, meta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fake_meta(w: i32, h: i32) -> CameraDepthMeta {
        CameraDepthMeta {
            width: w,
            height: h,
            fx: 0.0,
            fy: 0.0,
            cx: 0.0,
            cy: 0.0,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn registered_consumer_receives_dispatch_with_source() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = camera_depth_consumer_count();

        register_camera_depth_consumer(move |src, depths, meta| {
            assert_eq!(src, "/World/Camera/depth");
            assert_eq!(meta.width, 2);
            assert_eq!(meta.height, 2);
            assert_eq!(depths.len(), 4);
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(camera_depth_consumer_count(), n_baseline + 1);

        let depths = [0.5_f32, 1.0, 1.5, 2.0];
        dispatch_camera_depth("/World/Camera/depth", &depths, &fake_meta(2, 2));

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

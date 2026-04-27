use std::sync::OnceLock;

use crate::channel::{channel_singleton, Channel};
use crate::ffi::CameraRgbMeta;
use crate::sensor::Sensor;

/// Type-level marker for the RGB camera sensor channel.
pub struct CameraRgb;

impl Sensor for CameraRgb {
    const NAME: &'static str = "camera_rgb";
}

pub type Callback = Box<dyn Fn(&str, &[u8], &CameraRgbMeta) + Send + Sync + 'static>;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_channel_camera_rgb() -> *const Channel<Callback> {
    static SLOT: OnceLock<Box<Channel<Callback>>> = OnceLock::new();
    channel_singleton(&SLOT)
}

fn channel() -> &'static Channel<Callback> {
    unsafe { &*isaac_sim_bridge_channel_camera_rgb() }
}

pub fn register_camera_rgb_consumer<F>(cb: F)
where
    F: Fn(&str, &[u8], &CameraRgbMeta) + Send + Sync + 'static,
{
    channel().register(Box::new(cb));
}

pub fn dispatch_camera_rgb(source_id: &str, pixels: &[u8], meta: &CameraRgbMeta) {
    channel().for_each(|cb| cb(source_id, pixels, meta));
}

pub fn camera_rgb_consumer_count() -> usize {
    channel().count()
}

pub fn forward_camera_rgb(source_id: &str, pixels: &[u8], meta: &CameraRgbMeta) {
    log::debug!(
        "[isaac-sim-rs] forward_camera_rgb: source='{}' wxh={}x{} bytes={}",
        source_id,
        meta.width,
        meta.height,
        pixels.len()
    );
    dispatch_camera_rgb(source_id, pixels, meta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fake_meta(w: i32, h: i32) -> CameraRgbMeta {
        CameraRgbMeta {
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
        let n_baseline = camera_rgb_consumer_count();

        register_camera_rgb_consumer(move |src, pixels, meta| {
            assert_eq!(src, "/World/Camera/rgb");
            assert_eq!(meta.width, 2);
            assert_eq!(meta.height, 2);
            assert_eq!(pixels.len(), 12); // 2*2*3
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(camera_rgb_consumer_count(), n_baseline + 1);

        let pixels = [
            255_u8, 0, 0, // px(0,0)
            0, 255, 0, // px(1,0)
            0, 0, 255, // px(0,1)
            255, 255, 255, // px(1,1)
        ];
        dispatch_camera_rgb("/World/Camera/rgb", &pixels, &fake_meta(2, 2));

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

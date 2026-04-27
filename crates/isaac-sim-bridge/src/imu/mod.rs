use std::sync::OnceLock;

use crate::channel::{channel_singleton, Channel};
use crate::ffi::ImuMeta;
use crate::sensor::Sensor;

/// Type-level marker for the IMU sensor channel. One sample per
/// dispatch carries linear acceleration, angular velocity, and
/// orientation packed into [`ImuMeta`] — no variable-sized payload.
pub struct Imu;

impl Sensor for Imu {
    const NAME: &'static str = "imu";
}

pub type Callback = Box<dyn Fn(&str, &str, &ImuMeta) + Send + Sync + 'static>;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_channel_imu() -> *const Channel<Callback> {
    static SLOT: OnceLock<Box<Channel<Callback>>> = OnceLock::new();
    channel_singleton(&SLOT)
}

fn channel() -> &'static Channel<Callback> {
    unsafe { &*isaac_sim_bridge_channel_imu() }
}

pub fn register_imu_consumer<F>(cb: F)
where
    F: Fn(&str, &str, &ImuMeta) + Send + Sync + 'static,
{
    channel().register(Box::new(cb));
}

pub fn dispatch_imu(source_id: &str, frame_id: &str, meta: &ImuMeta) {
    channel().for_each(|cb| cb(source_id, frame_id, meta));
}

pub fn imu_consumer_count() -> usize {
    channel().count()
}

pub fn forward_imu(source_id: &str, frame_id: &str, meta: &ImuMeta) {
    log::debug!(
        "[isaac-sim-rs] forward_imu: source='{}' frame='{}' lin_acc=[{:.3},{:.3},{:.3}] ang_vel=[{:.3},{:.3},{:.3}] q=[{:.3},{:.3},{:.3},{:.3}]",
        source_id,
        frame_id,
        meta.lin_acc_x,
        meta.lin_acc_y,
        meta.lin_acc_z,
        meta.ang_vel_x,
        meta.ang_vel_y,
        meta.ang_vel_z,
        meta.orientation_w,
        meta.orientation_x,
        meta.orientation_y,
        meta.orientation_z,
    );
    dispatch_imu(source_id, frame_id, meta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fake_meta() -> ImuMeta {
        ImuMeta {
            lin_acc_x: 0.1,
            lin_acc_y: 0.2,
            lin_acc_z: 9.81,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.5,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn registered_consumer_receives_dispatch_with_source() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = imu_consumer_count();

        register_imu_consumer(move |src, frame, meta| {
            assert_eq!(src, "/World/Carter/imu");
            assert_eq!(frame, "sim_imu");
            assert!((meta.lin_acc_z - 9.81).abs() < 1e-9);
            assert_eq!(meta.orientation_w, 1.0);
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(imu_consumer_count(), n_baseline + 1);

        forward_imu("/World/Carter/imu", "sim_imu", &fake_meta());

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

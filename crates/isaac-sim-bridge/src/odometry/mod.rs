use std::sync::OnceLock;

use crate::channel::{channel_singleton, Channel};
use crate::ffi::OdometryMeta;
use crate::sensor::Sensor;

/// Type-level marker for the chassis odometry channel. One sample per
/// dispatch carries position, orientation, and body-frame velocities
/// packed into [`OdometryMeta`] — no variable-sized payload.
pub struct Odometry;

impl Sensor for Odometry {
    const NAME: &'static str = "odometry";
}

pub type Callback = Box<dyn Fn(&str, &str, &str, &OdometryMeta) + Send + Sync + 'static>;

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_channel_odometry() -> *const Channel<Callback> {
    static SLOT: OnceLock<Box<Channel<Callback>>> = OnceLock::new();
    channel_singleton(&SLOT)
}

fn channel() -> &'static Channel<Callback> {
    unsafe { &*isaac_sim_bridge_channel_odometry() }
}

pub fn register_odometry_consumer<F>(cb: F)
where
    F: Fn(&str, &str, &str, &OdometryMeta) + Send + Sync + 'static,
{
    channel().register(Box::new(cb));
}

pub fn dispatch_odometry(
    source_id: &str,
    chassis_frame_id: &str,
    odom_frame_id: &str,
    meta: &OdometryMeta,
) {
    channel().for_each(|cb| cb(source_id, chassis_frame_id, odom_frame_id, meta));
}

pub fn odometry_consumer_count() -> usize {
    channel().count()
}

pub fn forward_odometry(
    source_id: &str,
    chassis_frame_id: &str,
    odom_frame_id: &str,
    meta: &OdometryMeta,
) {
    log::debug!(
        "[isaac-sim-rs] forward_odometry: source='{}' chassis='{}' odom='{}' pos=[{:.3},{:.3},{:.3}] q=[{:.3},{:.3},{:.3},{:.3}] lin=[{:.3},{:.3},{:.3}] ang=[{:.3},{:.3},{:.3}]",
        source_id,
        chassis_frame_id,
        odom_frame_id,
        meta.position_x,
        meta.position_y,
        meta.position_z,
        meta.orientation_w,
        meta.orientation_x,
        meta.orientation_y,
        meta.orientation_z,
        meta.lin_vel_x,
        meta.lin_vel_y,
        meta.lin_vel_z,
        meta.ang_vel_x,
        meta.ang_vel_y,
        meta.ang_vel_z,
    );
    dispatch_odometry(source_id, chassis_frame_id, odom_frame_id, meta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fake_meta() -> OdometryMeta {
        OdometryMeta {
            position_x: 1.0,
            position_y: 2.0,
            position_z: 0.5,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            lin_vel_x: 0.3,
            lin_vel_y: 0.0,
            lin_vel_z: 0.0,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.1,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn registered_consumer_receives_dispatch_with_source() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let n_baseline = odometry_consumer_count();

        register_odometry_consumer(move |src, chassis, odom, meta| {
            assert_eq!(src, "/World/Carter");
            assert_eq!(chassis, "base_link");
            assert_eq!(odom, "odom");
            assert_eq!(meta.position_x, 1.0);
            assert_eq!(meta.lin_vel_x, 0.3);
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(odometry_consumer_count(), n_baseline + 1);

        forward_odometry("/World/Carter", "base_link", "odom", &fake_meta());

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

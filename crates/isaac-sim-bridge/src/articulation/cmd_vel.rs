use std::sync::{Arc, OnceLock};

use crate::ffi::CmdVel;
use crate::producer::{ProducerRegistry, ProducerSlot};
use crate::sensor::Sensor;

/// Type-level marker for the cmd_vel articulation channel.
///
/// Articulation reverses the data direction (Rust→C++): downstream
/// adapters publish CmdVel values into a per-target slot; the C++ tick
/// reads the latest via `poll_cmd_vel`. The `Sensor` impl gives us
/// uniform NAME-derived env var / log labels even though the data
/// direction is opposite to the sensor consumers.
pub struct CmdVelChannel;

impl Sensor for CmdVelChannel {
    const NAME: &'static str = "cmd_vel";
}

#[unsafe(no_mangle)]
pub extern "C" fn isaac_sim_bridge_registry_cmd_vel() -> *const ProducerRegistry<CmdVel> {
    static SLOT: OnceLock<Box<ProducerRegistry<CmdVel>>> = OnceLock::new();
    let r = SLOT.get_or_init(|| Box::new(ProducerRegistry::new()));
    Box::as_ref(r) as *const ProducerRegistry<CmdVel>
}

fn registry() -> &'static ProducerRegistry<CmdVel> {
    unsafe { &*isaac_sim_bridge_registry_cmd_vel() }
}

/// Register (or fetch) a cmd_vel producer for `target_id` (typically
/// the articulation prim path). Multiple writers can hold their own
/// Arc to the slot — last `publish` wins.
pub fn register_cmd_vel_producer(target_id: impl Into<String>) -> Arc<ProducerSlot<CmdVel>> {
    registry().register(target_id)
}

pub fn cmd_vel_producer_count() -> usize {
    registry().count()
}

/// Register a consumer that observes every Twist published into any
/// cmd_vel producer slot. The closure receives `(target_id, &CmdVel)`
/// — filter on `target_id` if you only care about one articulation.
///
/// This is the symmetric "publisher" direction: anything that wants
/// to watch the actuation stream (dora downstream output, telemetry
/// logger, second simulator) hooks in here without coupling to the
/// upstream source.
pub fn register_cmd_vel_consumer<F>(cb: F)
where
    F: Fn(&str, &CmdVel) + Send + Sync + 'static,
{
    registry().add_observer(cb);
}

pub fn cmd_vel_consumer_count() -> usize {
    registry().observer_count()
}

/// C++-facing poll. Looks up the slot for `target_id`, copies the
/// latest published value into `out`, and returns true on hit.
/// Returns false if no producer is registered for that target or if
/// no value has been published yet.
pub fn poll_cmd_vel(target_id: &str, out: &mut CmdVel) -> bool {
    if let Some(slot) = registry().lookup(target_id) {
        if let Some(v) = slot.latest() {
            *out = *v;
            return true;
        }
    }
    false
}

/// Rust-friendly variant of [`poll_cmd_vel`] for tests + adapter
/// inspection. Returns `Some(CmdVel)` when a producer is registered
/// and has published at least once; `None` otherwise.
pub fn peek_cmd_vel(target_id: &str) -> Option<CmdVel> {
    let mut out = CmdVel::default();
    poll_cmd_vel(target_id, &mut out).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cmd_vel(linear_x: f32) -> CmdVel {
        CmdVel {
            linear_x,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.1,
            timestamp_ns: 1_000_000,
        }
    }

    #[test]
    fn poll_returns_published_value() {
        let target = "/test/articulation/poll_returns";
        let slot = register_cmd_vel_producer(target);
        slot.publish(make_cmd_vel(0.5));

        let mut out = CmdVel::default();
        assert!(poll_cmd_vel(target, &mut out));
        assert_eq!(out.linear_x, 0.5);
    }

    #[test]
    fn poll_misses_when_no_producer() {
        let mut out = CmdVel::default();
        assert!(!poll_cmd_vel(
            "/test/articulation/never_registered",
            &mut out
        ));
    }

    #[test]
    fn poll_misses_until_first_publish() {
        let target = "/test/articulation/no_publish_yet";
        let _slot = register_cmd_vel_producer(target);
        let mut out = CmdVel::default();
        assert!(!poll_cmd_vel(target, &mut out));
    }
}

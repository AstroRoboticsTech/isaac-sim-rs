use std::sync::{Mutex, OnceLock};

use arc_swap::ArcSwapOption;

/// Lock-free single-slot store for a "latest-wins" producer.
///
/// Multiple Rust sources (dora subscriber, joystick driver, RL policy)
/// can call `publish` concurrently; the C++ consumer reads via `latest`
/// from any thread without blocking writers. ArcSwap gives us atomic
/// pointer swap with no torn reads — `publish` cost is one Arc clone +
/// atomic store, `latest` is one atomic load + clone.
pub struct ProducerSlot<T> {
    slot: ArcSwapOption<T>,
}

impl<T> ProducerSlot<T> {
    pub fn new() -> Self {
        Self {
            slot: ArcSwapOption::const_empty(),
        }
    }

    pub fn publish(&self, value: T) {
        self.slot.store(Some(std::sync::Arc::new(value)));
    }

    pub fn latest(&self) -> Option<std::sync::Arc<T>> {
        self.slot.load_full()
    }

    pub fn clear(&self) {
        self.slot.store(None);
    }
}

impl<T> Default for ProducerSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-target_id registry of producer slots. Each cmd_vel-shaped sensor
/// (or controller target) gets its own keyed slot so multiple targets
/// can co-exist and the C++ poll path looks up by target_id.
pub struct ProducerRegistry<T: 'static> {
    inner: OnceLock<Mutex<Vec<(String, std::sync::Arc<ProducerSlot<T>>)>>>,
}

impl<T: 'static> ProducerRegistry<T> {
    pub const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    fn slots(&self) -> &Mutex<Vec<(String, std::sync::Arc<ProducerSlot<T>>)>> {
        self.inner.get_or_init(|| Mutex::new(Vec::new()))
    }

    /// Register (or fetch) the producer slot for `target_id`. Returns
    /// the Arc'd handle so the caller can `publish` from any thread.
    pub fn register(&self, target_id: impl Into<String>) -> std::sync::Arc<ProducerSlot<T>> {
        let target_id = target_id.into();
        let mut guard = self.slots().lock().unwrap();
        if let Some((_, slot)) = guard.iter().find(|(t, _)| t == &target_id) {
            return std::sync::Arc::clone(slot);
        }
        let slot = std::sync::Arc::new(ProducerSlot::new());
        guard.push((target_id, std::sync::Arc::clone(&slot)));
        slot
    }

    /// Look up the producer slot for `target_id` without registering one.
    /// Used by the C++ poll path to get the latest published value.
    pub fn lookup(&self, target_id: &str) -> Option<std::sync::Arc<ProducerSlot<T>>> {
        let guard = self.slots().lock().unwrap();
        guard
            .iter()
            .find(|(t, _)| t == target_id)
            .map(|(_, s)| std::sync::Arc::clone(s))
    }

    pub fn count(&self) -> usize {
        self.slots().lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_value_round_trips_through_slot() {
        let slot: ProducerSlot<i32> = ProducerSlot::new();
        assert!(slot.latest().is_none());
        slot.publish(42);
        assert_eq!(*slot.latest().expect("published"), 42);
        slot.publish(7);
        assert_eq!(*slot.latest().expect("overwrite"), 7);
        slot.clear();
        assert!(slot.latest().is_none());
    }

    #[test]
    fn registry_is_keyed_by_target_id() {
        let reg: ProducerRegistry<i32> = ProducerRegistry::new();
        let a = reg.register("/Robot/A");
        let b = reg.register("/Robot/B");
        let a_again = reg.register("/Robot/A");
        // Same target → same slot (Arc-equal).
        assert!(std::sync::Arc::ptr_eq(&a, &a_again));
        // Different target → different slot.
        assert!(!std::sync::Arc::ptr_eq(&a, &b));

        a.publish(1);
        b.publish(2);
        assert_eq!(*reg.lookup("/Robot/A").unwrap().latest().unwrap(), 1);
        assert_eq!(*reg.lookup("/Robot/B").unwrap().latest().unwrap(), 2);
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn lookup_unregistered_target_returns_none() {
        let reg: ProducerRegistry<i32> = ProducerRegistry::new();
        assert!(reg.lookup("/Robot/Unregistered").is_none());
    }
}

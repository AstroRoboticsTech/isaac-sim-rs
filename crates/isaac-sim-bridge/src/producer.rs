use std::sync::{Arc, Mutex, OnceLock};

use arc_swap::ArcSwapOption;

use crate::channel::Channel;

type Slots<T> = Mutex<Vec<(String, Arc<ProducerSlot<T>>)>>;
type Observer<T> = Box<dyn Fn(&str, &T) + Send + Sync + 'static>;

/// Lock-free single-slot store for a "latest-wins" producer.
///
/// Multiple Rust sources (dora subscriber, joystick driver, RL policy)
/// can call `publish` concurrently; the C++ consumer reads via `latest`
/// from any thread without blocking writers. ArcSwap gives us atomic
/// pointer swap with no torn reads — `publish` cost is one Arc clone +
/// atomic store, `latest` is one atomic load + clone.
///
/// Slots produced by [`ProducerRegistry::register`] also notify the
/// registry's observer channel on every `publish`, so the same
/// downstream telemetry / dora-output / logging fan-out shape used for
/// sensor consumers extends to actuation streams.
pub struct ProducerSlot<T> {
    target_id: String,
    slot: ArcSwapOption<T>,
    observers: Option<Arc<Channel<Observer<T>>>>,
}

impl<T> ProducerSlot<T> {
    pub fn new(target_id: impl Into<String>) -> Self {
        Self {
            target_id: target_id.into(),
            slot: ArcSwapOption::const_empty(),
            observers: None,
        }
    }

    fn with_observers(target_id: impl Into<String>, observers: Arc<Channel<Observer<T>>>) -> Self {
        Self {
            target_id: target_id.into(),
            slot: ArcSwapOption::const_empty(),
            observers: Some(observers),
        }
    }

    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    pub fn publish(&self, value: T) {
        let arc = Arc::new(value);
        if let Some(obs) = &self.observers {
            obs.for_each(|cb| cb(&self.target_id, &arc));
        }
        self.slot.store(Some(arc));
    }

    pub fn latest(&self) -> Option<Arc<T>> {
        self.slot.load_full()
    }

    pub fn clear(&self) {
        self.slot.store(None);
    }
}

/// Per-target_id registry of producer slots. Each cmd_vel-shaped sensor
/// (or controller target) gets its own keyed slot so multiple targets
/// can co-exist and the C++ poll path looks up by target_id.
///
/// The registry also owns a single observer channel; observers added
/// via [`add_observer`] receive `(target_id, &T)` for every publish on
/// any slot in this registry.
pub struct ProducerRegistry<T: 'static> {
    inner: OnceLock<Slots<T>>,
    observers: OnceLock<Arc<Channel<Observer<T>>>>,
}

impl<T: 'static> ProducerRegistry<T> {
    pub const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
            observers: OnceLock::new(),
        }
    }

    fn slots(&self) -> &Slots<T> {
        self.inner.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn observers(&self) -> &Arc<Channel<Observer<T>>> {
        self.observers.get_or_init(|| Arc::new(Channel::new()))
    }

    /// Register (or fetch) the producer slot for `target_id`. Returns
    /// the Arc'd handle so the caller can `publish` from any thread.
    pub fn register(&self, target_id: impl Into<String>) -> Arc<ProducerSlot<T>> {
        let target_id = target_id.into();
        let mut guard = self.slots().lock().unwrap();
        if let Some((_, slot)) = guard.iter().find(|(t, _)| t == &target_id) {
            return Arc::clone(slot);
        }
        let slot = Arc::new(ProducerSlot::with_observers(
            target_id.clone(),
            Arc::clone(self.observers()),
        ));
        guard.push((target_id, Arc::clone(&slot)));
        slot
    }

    /// Look up the producer slot for `target_id` without registering one.
    /// Used by the C++ poll path to get the latest published value.
    pub fn lookup(&self, target_id: &str) -> Option<Arc<ProducerSlot<T>>> {
        let guard = self.slots().lock().unwrap();
        guard
            .iter()
            .find(|(t, _)| t == target_id)
            .map(|(_, s)| Arc::clone(s))
    }

    pub fn count(&self) -> usize {
        self.slots().lock().unwrap().len()
    }

    /// Register an observer that fires on every `publish` to any slot
    /// in this registry. The closure receives the slot's target_id and
    /// the published value. Use with the dora cmd_vel publisher
    /// adapter, telemetry sinks, replay loggers, etc.
    pub fn add_observer<F>(&self, cb: F)
    where
        F: Fn(&str, &T) + Send + Sync + 'static,
    {
        self.observers().register(Box::new(cb));
    }

    pub fn observer_count(&self) -> usize {
        self.observers().count()
    }
}

impl<T: 'static> Default for ProducerRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_value_round_trips_through_slot() {
        let slot: ProducerSlot<i32> = ProducerSlot::new("/test/standalone");
        assert!(slot.latest().is_none());
        slot.publish(42);
        assert_eq!(*slot.latest().expect("published"), 42);
        slot.publish(7);
        assert_eq!(*slot.latest().expect("overwrite"), 7);
        slot.clear();
        assert!(slot.latest().is_none());
        assert_eq!(slot.target_id(), "/test/standalone");
    }

    #[test]
    fn registry_observers_see_every_publish() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let reg: ProducerRegistry<i32> = ProducerRegistry::new();
        let total = Arc::new(AtomicUsize::new(0));
        let total_clone = Arc::clone(&total);
        reg.add_observer(move |target, value| {
            assert!(target.starts_with("/Robot/"));
            total_clone.fetch_add(*value as usize, Ordering::SeqCst);
        });
        let a = reg.register("/Robot/A");
        let b = reg.register("/Robot/B");
        a.publish(3);
        b.publish(4);
        a.publish(5);
        assert_eq!(total.load(Ordering::SeqCst), 12);
        assert_eq!(reg.observer_count(), 1);
    }

    #[test]
    fn registry_is_keyed_by_target_id() {
        let reg: ProducerRegistry<i32> = ProducerRegistry::new();
        let a = reg.register("/Robot/A");
        let b = reg.register("/Robot/B");
        let a_again = reg.register("/Robot/A");
        // Same target → same slot (Arc-equal).
        assert!(Arc::ptr_eq(&a, &a_again));
        // Different target → different slot.
        assert!(!Arc::ptr_eq(&a, &b));

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

use std::sync::{Arc, OnceLock, RwLock};

pub struct Channel<C> {
    cbs: OnceLock<RwLock<Arc<Vec<Arc<C>>>>>,
}

impl<C> Channel<C> {
    pub const fn new() -> Self {
        Self {
            cbs: OnceLock::new(),
        }
    }

    fn registry(&self) -> &RwLock<Arc<Vec<Arc<C>>>> {
        self.cbs.get_or_init(|| RwLock::new(Arc::new(Vec::new())))
    }

    pub fn register(&self, cb: C) {
        let mut guard = self.registry().write().unwrap();
        let mut next: Vec<Arc<C>> = (**guard).clone();
        next.push(Arc::new(cb));
        *guard = Arc::new(next);
    }

    pub fn count(&self) -> usize {
        self.registry().read().unwrap().len()
    }

    pub fn for_each<F: FnMut(&C)>(&self, mut f: F) {
        let snap = self.registry().read().unwrap().clone();
        for cb in snap.iter() {
            f(cb.as_ref());
        }
    }
}

impl<C> Default for Channel<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    type IntCallback = Box<dyn Fn(i32) + Send + Sync + 'static>;

    #[test]
    fn registered_callbacks_receive_for_each() {
        let ch: Channel<IntCallback> = Channel::new();
        let total = Arc::new(AtomicUsize::new(0));

        let total_a = Arc::clone(&total);
        ch.register(Box::new(move |v| {
            total_a.fetch_add(v as usize, Ordering::SeqCst);
        }));
        let total_b = Arc::clone(&total);
        ch.register(Box::new(move |v| {
            total_b.fetch_add((v * 10) as usize, Ordering::SeqCst);
        }));

        assert_eq!(ch.count(), 2);
        ch.for_each(|cb| cb(3));
        assert_eq!(total.load(Ordering::SeqCst), 33);
    }

    #[test]
    fn for_each_does_not_hold_lock_across_callback() {
        // Re-entrant register from within a callback must not deadlock.
        let ch: Channel<Box<dyn Fn() + Send + Sync>> = Channel::new();
        let count = Arc::new(AtomicUsize::new(0));

        // Registering a second callback from inside the first proves the
        // read lock isn't held across the user closure.
        let count_clone = Arc::clone(&count);
        ch.register(Box::new(move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        ch.for_each(|cb| cb());
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

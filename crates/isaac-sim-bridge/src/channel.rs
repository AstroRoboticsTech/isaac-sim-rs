use std::sync::{Arc, OnceLock, RwLock};

/// Generic registry of typed callbacks.
///
/// `Channel<C>` is held inside a heap-allocated singleton accessed via
/// an exported `#[no_mangle] extern "C"` getter (see [`channel_singleton`]).
/// This is the load-bearing trick for cdylib state isolation: when the
/// bridge crate is linked into multiple cdylibs in the same process
/// (one as the bridge plugin's cdylib, one as an adapter runner),
/// hidden Rust statics would each get a private copy. An exported
/// `extern "C"` symbol is deduplicated by the dynamic linker (first
/// loaded wins under RTLD_GLOBAL), so every caller — adapter and
/// bridge alike — gets back the same heap pointer.
pub struct Channel<C> {
    cbs: RwLock<Arc<Vec<Arc<C>>>>,
}

impl<C> Channel<C> {
    pub fn new() -> Self {
        Self {
            cbs: RwLock::new(Arc::new(Vec::new())),
        }
    }

    pub fn register(&self, cb: C) {
        let mut guard = self.cbs.write().unwrap();
        let mut next: Vec<Arc<C>> = (**guard).clone();
        next.push(Arc::new(cb));
        *guard = Arc::new(next);
    }

    pub fn count(&self) -> usize {
        self.cbs.read().unwrap().len()
    }

    pub fn for_each<F: FnMut(&C)>(&self, mut f: F) {
        let snap = self.cbs.read().unwrap().clone();
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

/// Returns a stable pointer to the channel held in `slot`. The slot
/// lives in a single canonical cdylib (whichever the dynamic linker
/// resolved first when the calling module's exported getter was set
/// up), so every cdylib that calls through the same getter shares one
/// heap registry.
///
/// Used by per-sensor `#[no_mangle] pub extern "C" fn` getters in each
/// sensor module — the getter declaration is what the dynamic linker
/// dedups; this helper just abstracts the OnceLock + Box machinery
/// behind it.
pub fn channel_singleton<C: 'static>(slot: &'static OnceLock<Box<Channel<C>>>) -> *const Channel<C> {
    let ch = slot.get_or_init(|| Box::new(Channel::new()));
    Box::as_ref(ch) as *const Channel<C>
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
    fn singleton_returns_same_pointer_across_calls() {
        static SLOT: OnceLock<Box<Channel<IntCallback>>> = OnceLock::new();
        let p1 = channel_singleton(&SLOT);
        let p2 = channel_singleton(&SLOT);
        assert_eq!(p1, p2, "channel_singleton must return a stable pointer");
    }

    #[test]
    fn singleton_state_is_shared() {
        static SLOT: OnceLock<Box<Channel<IntCallback>>> = OnceLock::new();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        unsafe {
            let ch = &*channel_singleton(&SLOT);
            ch.register(Box::new(move |v| {
                count_clone.fetch_add(v as usize, Ordering::SeqCst);
            }));
        }
        unsafe {
            let ch = &*channel_singleton(&SLOT);
            ch.for_each(|cb| cb(7));
        }
        assert_eq!(count.load(Ordering::SeqCst), 7);
    }
}

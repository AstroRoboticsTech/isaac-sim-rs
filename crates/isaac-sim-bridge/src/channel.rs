use std::sync::{Mutex, OnceLock};

pub struct Channel<C> {
    cbs: OnceLock<Mutex<Vec<C>>>,
}

impl<C> Channel<C> {
    pub const fn new() -> Self {
        Self {
            cbs: OnceLock::new(),
        }
    }

    fn registry(&self) -> &Mutex<Vec<C>> {
        self.cbs.get_or_init(|| Mutex::new(Vec::new()))
    }

    pub fn register(&self, cb: C) {
        self.registry().lock().unwrap().push(cb);
    }

    pub fn count(&self) -> usize {
        self.registry().lock().unwrap().len()
    }

    pub fn for_each<F: FnMut(&C)>(&self, mut f: F) {
        let cbs = self.registry().lock().unwrap();
        for cb in cbs.iter() {
            f(cb);
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
    use std::sync::Arc;

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
}

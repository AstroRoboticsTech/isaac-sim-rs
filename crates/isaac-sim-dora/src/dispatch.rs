// SPDX-License-Identifier: MPL-2.0
//! Latest-wins per-output pump for the dora adapter.
//!
//! Same shape as the rerun adapter's `dispatch`: publisher's
//! `publish` is non-blocking, drain thread does the actual
//! `node.send_output(...)`. This keeps the OG render thread
//! independent of dora's serialise + Zenoh write path so a slow peer
//! can never backpressure the simulator.
//!
//! The shared `Arc<Mutex<DoraNode>>` is held only inside the drain
//! thread, where one drain per output serialises sends naturally;
//! there is no contention with other sensors.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;

use arc_swap::ArcSwapOption;

pub struct LatestSlot<T: Send + Sync + 'static> {
    slot: ArcSwapOption<T>,
    wake: SyncSender<()>,
}

impl<T: Send + Sync + 'static> LatestSlot<T> {
    pub fn new() -> (Arc<Self>, Receiver<()>) {
        let (tx, rx) = sync_channel::<()>(1);
        let slot = Arc::new(Self {
            slot: ArcSwapOption::const_empty(),
            wake: tx,
        });
        (slot, rx)
    }

    pub fn publish(&self, value: T) {
        self.slot.store(Some(Arc::new(value)));
        let _ = self.wake.try_send(());
    }

    fn take(&self) -> Option<Arc<T>> {
        self.slot.swap(None)
    }
}

/// Spawn a drain thread that forwards the latest slot value to `sink`.
///
/// Ordering: `publish` atomically stores the new value then sends a wake.
/// The drain unblocks on that wake, drains any additional wakes via
/// `try_recv` (coalescing burst publishes into one read), then calls
/// `slot.take()` which atomically swaps the slot to `None`. If a new
/// `publish` races in during that window, the bounded wake channel already
/// has capacity and the next `recv()` fires immediately, guaranteeing no
/// value is silently dropped — worst case is one extra round-trip of
/// latency. The `arc_swap::ArcSwapOption::swap(None)` inside `take` is
/// the load-bearing atomic.
///
/// The thread exits when all `Arc` refs to the `LatestSlot` are dropped:
/// dropping the slot closes the `SyncSender` wake half, causing
/// `wake.recv()` inside the drain to return `Err`. Production callers
/// (cdylib publishers) intentionally leak the returned `JoinHandle`
/// because the drain is process-lifetime; the shutdown path is exercised
/// in tests.
pub fn spawn_drain<T, F>(
    name: &str,
    slot: Arc<LatestSlot<T>>,
    wake: Receiver<()>,
    mut sink: F,
) -> thread::JoinHandle<()>
where
    T: Send + Sync + 'static,
    F: FnMut(Arc<T>) + Send + 'static,
{
    let weak = Arc::downgrade(&slot);
    drop(slot);
    let name = name.to_string();
    thread::Builder::new()
        .name(name)
        .spawn(move || {
            while wake.recv().is_ok() {
                while wake.try_recv().is_ok() {}
                if let Some(slot) = weak.upgrade() {
                    if let Some(v) = slot.take() {
                        sink(v);
                    }
                }
            }
        })
        .expect("spawn dora drain thread")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn drain_observes_published_values() {
        let (slot, wake) = LatestSlot::<i32>::new();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        let _h = spawn_drain("test-drain", Arc::clone(&slot), wake, move |_v| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });
        slot.publish(1);
        slot.publish(2);
        for _ in 0..50 {
            if count.load(Ordering::SeqCst) > 0 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(count.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn drain_thread_exits_when_slot_is_dropped() {
        let (slot, wake) = LatestSlot::<i32>::new();
        let handle = spawn_drain("test-shutdown", Arc::clone(&slot), wake, move |_| {});
        slot.publish(1);
        thread::sleep(Duration::from_millis(20));
        drop(slot);
        let deadline = std::time::Instant::now() + Duration::from_millis(500);
        loop {
            if handle.is_finished() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "drain thread did not exit within 500 ms after slot drop"
            );
            thread::sleep(Duration::from_millis(10));
        }
        handle.join().expect("drain exits cleanly");
    }
}

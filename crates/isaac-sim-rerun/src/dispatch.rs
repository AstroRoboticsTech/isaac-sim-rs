//! Latest-wins per-output pump.
//!
//! Decouples the OG render thread (synchronous publisher inside
//! `forward_<sensor>`) from the rerun gRPC sink (potentially-slow
//! consumer). The OG side calls [`LatestSlot::publish`] which is
//! non-blocking and replaces any pending value; a dedicated drain
//! thread pulls the latest frame and calls `rec.log(...)`.
//!
//! Why this matters: a single shared `RecordingStream` was the
//! contention point that made LiDAR slow as soon as a 60 Hz camera
//! came online — every `rec.log` from every sensor went through one
//! gRPC channel, and when the camera saturated it the LiDAR's
//! `rec.log` blocked the OG thread on the next compute(). With this
//! pump, each sensor publishes into its own slot at OG-thread speed
//! and a per-output drain thread feeds the (still-shared) gRPC
//! channel at network speed, dropping older frames on overflow.
//!
//! Mirrors audit P6's "per-output bounded mpsc with single drain
//! task — drops oldest on overflow so a slow consumer can't
//! backpressure the render thread." Rerun is the first
//! manifestation; dora gets the same treatment via
//! `isaac-sim-dora::dispatch`.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;

use arc_swap::ArcSwapOption;

/// Latest-wins one-slot pipeline. `publish` is non-blocking and
/// replaces any pending value; the drain thread blocks on the wake
/// channel and reads the slot when it fires.
pub struct LatestSlot<T: Send + Sync + 'static> {
    slot: ArcSwapOption<T>,
    wake: SyncSender<()>,
}

impl<T: Send + Sync + 'static> LatestSlot<T> {
    /// Returns the slot plus the wake-receiver to hand to a drain
    /// thread. The wake channel is bounded at 1: a queued wake
    /// already pending guarantees the drain will observe the latest
    /// value once it reads, so further wakes are redundant.
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
        // try_send: if the wake slot is already armed, a wake is
        // already in flight; the drain will read our newly-stored
        // value when it processes that wake. If the drain has
        // disconnected (joined / dropped), nothing to do.
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
/// the load-bearing atomic. The thread exits when all `Arc` refs to the
/// slot are dropped (wake channel disconnects, `wake.recv()` returns `Err`).
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
    let name = name.to_string();
    thread::Builder::new()
        .name(name)
        .spawn(move || {
            while wake.recv().is_ok() {
                while wake.try_recv().is_ok() {}
                if let Some(v) = slot.take() {
                    sink(v);
                }
            }
        })
        .expect("spawn rerun drain thread")
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
        slot.publish(3);
        // Drain may coalesce; just wait long enough for at least one
        // observation.
        for _ in 0..50 {
            if count.load(Ordering::SeqCst) > 0 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(count.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn drain_drops_intermediate_values_under_slow_sink() {
        let (slot, wake) = LatestSlot::<i32>::new();
        let observed = Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
        let observed_clone = Arc::clone(&observed);
        let _h = spawn_drain("test-drain", Arc::clone(&slot), wake, move |v| {
            // Slow sink: 50 ms per call. Producer pushes faster than this.
            thread::sleep(Duration::from_millis(50));
            observed_clone.lock().unwrap().push(*v);
        });
        for i in 1..=10_i32 {
            slot.publish(i);
            thread::sleep(Duration::from_millis(5));
        }
        thread::sleep(Duration::from_millis(200));
        let seen = observed.lock().unwrap().clone();
        // We never see all 10; the slot coalesces. We must always see
        // a non-empty subset whose last value is the most recent
        // produced (or close to it — drain may snap on the second-to-last
        // wake too).
        assert!(!seen.is_empty(), "drain saw nothing");
        assert!(seen.len() < 10, "drain saw all values; coalescing failed");
        assert_eq!(
            *seen.last().unwrap(),
            10,
            "last observed must be the most recent"
        );
    }
}

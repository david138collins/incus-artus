//! The loading lane (the regroup, docs/regroup.md, lane 0.2): worker threads that make things in the
//! background (chunks now; the card's readback and the cache's strips later), so the frame never
//! waits on them.
//!
//! - **Every job has a tier** [David 2026-11-05: "we should definitely be sequencing with priorities as
//!   we establish our grip on the hardware"]: `Soon` (what's in view, nearest first) before `Whenever`
//!   (far fills, saves, editor pictures), and within a tier the smallest `key` first (its distance, say).
//!   A worker always takes the most important job waiting; a job already running isn't stopped.
//! - **The workers stand below the frame:** their threads run at a lower priority than the frame's, so a
//!   loading job can never take the frame's core (`runner::hardware::background_thread`).
//! - **A job nobody wants any more is dropped** before it runs: dropping its `Job` handle says so.
//! - **Results come back to whoever asked**, read without waiting (`Job::ready`); taking them in within
//!   a time budget is the asker's part (streaming's `insert_budget_ms`), so too much arriving at once
//!   waits for the next frame instead of stalling this one.
//!
//! Our own, not Bevy's (the ejection seat: one less thing reached through the doorway).

use std::cmp::Ordering as Order;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

/// How soon a job is wanted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// What's in view, or about to be: chunks and strips round the player.
    Soon,
    /// Whatever can wait: far fills, saves, editor pictures. (Nothing uses it yet: the cache's far
    /// strips, phase 3.1, are its first.)
    #[allow(dead_code)]
    Whenever,
}

/// A job asked for: its result, once it's made. Dropping it drops the job if it hasn't started.
pub struct Job<T> {
    slot: Arc<Slot<T>>,
}

/// Where a job's result is left: shared by the job and whoever asked for it.
struct Slot<T> {
    made: Mutex<Option<T>>,
    /// Still wanted (cleared when the `Job` is dropped), and failed (the job panicked).
    wanted: AtomicBool,
    failed: AtomicBool,
}

impl<T> Job<T> {
    /// The result if it's made, without waiting. `Err` if the job failed (a panic in it is caught,
    /// and reported in the crash log).
    pub fn ready(&self) -> Result<Option<T>, ()> {
        if self.slot.failed.load(Ordering::Acquire) {
            return Err(());
        }
        Ok(self.slot.made.lock().ok().and_then(|mut made| made.take()))
    }
}

impl<T> Drop for Job<T> {
    fn drop(&mut self) {
        self.slot.wanted.store(false, Ordering::Relaxed);
    }
}

/// Marks the job failed if it ends without leaving its result (it panicked).
struct Failing<'a>(&'a AtomicBool, bool);

impl Drop for Failing<'_> {
    fn drop(&mut self) {
        if !self.1 {
            self.0.store(true, Ordering::Release);
        }
    }
}

/// Puts a job on the loading lane.
pub fn submit<T: Send + 'static>(tier: Tier, key: f32, work: impl FnOnce() -> T + Send + 'static) -> Job<T> {
    let slot = Arc::new(Slot { made: Mutex::new(None), wanted: AtomicBool::new(true), failed: AtomicBool::new(false) });
    let shared = slot.clone();
    let lane = lane();
    let order = lane.submitted.fetch_add(1, Ordering::Relaxed);
    let run: Box<dyn FnOnce() + Send> = Box::new(move || {
        if !shared.wanted.load(Ordering::Relaxed) {
            return;
        }
        let mut failing = Failing(&shared.failed, false);
        let made = work();
        if let Ok(mut slot) = shared.made.lock() {
            *slot = Some(made);
        }
        failing.1 = true;
    });
    if let Ok(mut waiting) = lane.waiting.lock() {
        waiting.push(Waiting { tier, key, order, run });
    }
    lane.woken.notify_one();
    Job { slot }
}

/// How many jobs are waiting for a worker now.
pub fn waiting() -> usize {
    lane().waiting.lock().map_or(0, |waiting| waiting.len())
}

/// How many workers the lane has: all the processor's threads but two (the frame's, and one for
/// Windows and everything else), at least one.
pub fn workers() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get()).saturating_sub(2).max(1)
}

struct Lane {
    waiting: Mutex<BinaryHeap<Waiting>>,
    woken: Condvar,
    submitted: AtomicU64,
}

/// A job waiting for a worker: the most important first (the heap's top is the greatest).
struct Waiting {
    tier: Tier,
    key: f32,
    order: u64,
    run: Box<dyn FnOnce() + Send>,
}

impl Waiting {
    /// Less is more important: the tier, then the key, then first come.
    fn rank(&self, other: &Self) -> Order {
        self.tier.cmp(&other.tier).then(self.key.total_cmp(&other.key)).then(self.order.cmp(&other.order))
    }
}

impl PartialEq for Waiting {
    fn eq(&self, other: &Self) -> bool {
        self.rank(other) == Order::Equal
    }
}

impl Eq for Waiting {}

impl PartialOrd for Waiting {
    fn partial_cmp(&self, other: &Self) -> Option<Order> {
        Some(self.cmp(other))
    }
}

impl Ord for Waiting {
    fn cmp(&self, other: &Self) -> Order {
        // Reversed: the heap gives the greatest, and the most important ranks least.
        other.rank(self)
    }
}

fn lane() -> &'static Lane {
    static LANE: OnceLock<&'static Lane> = OnceLock::new();
    LANE.get_or_init(|| {
        let lane: &'static Lane = Box::leak(Box::new(Lane { waiting: Mutex::new(BinaryHeap::new()), woken: Condvar::new(), submitted: AtomicU64::new(0) }));
        for k in 0..workers() {
            std::thread::Builder::new().name(format!("loading {k}")).spawn(move || work(lane)).ok();
        }
        lane
    })
}

/// A worker: takes the most important job waiting, runs it, and sleeps when there's none.
fn work(lane: &'static Lane) {
    crate::runner::hardware::background_thread();
    loop {
        let job = {
            let Ok(mut waiting) = lane.waiting.lock() else { return };
            loop {
                if let Some(job) = waiting.pop() {
                    break job;
                }
                waiting = match lane.woken.wait(waiting) {
                    Ok(waiting) => waiting,
                    Err(_) => return,
                };
            }
        };
        // A panic in a job is the job's, not the worker's: it's reported (the crash report) and the
        // worker goes on.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job.run));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_for<T>(job: &Job<T>) -> T {
        let started = Instant::now();
        loop {
            if let Ok(Some(made)) = job.ready() {
                return made;
            }
            assert!(started.elapsed() < Duration::from_secs(10), "the job never came back");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn a_job_comes_back_with_its_result() {
        let job = submit(Tier::Soon, 0.0, || 6 * 7);
        assert_eq!(wait_for(&job), 42);
    }

    #[test]
    fn soon_comes_before_whenever_and_near_before_far() {
        let mut queue = BinaryHeap::new();
        let job = |tier, key, order| Waiting { tier, key, order, run: Box::new(|| {}) };
        queue.push(job(Tier::Whenever, 0.0, 0));
        queue.push(job(Tier::Soon, 5.0, 1));
        queue.push(job(Tier::Soon, 1.0, 2));
        queue.push(job(Tier::Soon, 1.0, 3));
        let order: Vec<(Tier, f32, u64)> = std::iter::from_fn(|| queue.pop().map(|w| (w.tier, w.key, w.order))).collect();
        assert_eq!(order, vec![(Tier::Soon, 1.0, 2), (Tier::Soon, 1.0, 3), (Tier::Soon, 5.0, 1), (Tier::Whenever, 0.0, 0)]);
    }

    #[test]
    fn a_dropped_job_is_not_run() {
        let ran = Arc::new(AtomicBool::new(false));
        // Every worker kept busy for a moment, so the job below waits in the queue.
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let busy: Vec<Job<()>> = (0..workers())
            .map(|_| {
                let gate = gate.clone();
                submit(Tier::Soon, 0.0, move || {
                    let (open, opened) = &*gate;
                    let mut open = open.lock().unwrap();
                    while !*open {
                        open = opened.wait(open).unwrap();
                    }
                })
            })
            .collect();
        let flag = ran.clone();
        let dropped = submit(Tier::Whenever, 0.0, move || flag.store(true, Ordering::SeqCst));
        drop(dropped);
        {
            let (open, opened) = &*gate;
            *open.lock().unwrap() = true;
            opened.notify_all();
        }
        for job in &busy {
            wait_for(job);
        }
        // Once the queue has drained, the dropped job hasn't run.
        let after = submit(Tier::Whenever, f32::MAX, || ());
        wait_for(&after);
        assert!(!ran.load(Ordering::SeqCst));
    }
}

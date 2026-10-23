//! The runner — runs steps on a background thread, so the suite's screen never freezes, and
//! reports back after each step so you can watch the world being made one step at a time.

use super::map::Map;
use super::step::{LogLine, Step, Work};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub enum Progress {
    /// Step `index` finished; here's the map as it stands after it.
    Done { index: usize, map: Map, millis: f64, lines: Vec<LogLine> },
    /// The run was stopped part-way.
    Stopped,
}

/// One run in progress.
pub struct Job {
    /// The step running now.
    pub current: usize,
    // A Mutex only because Bevy resources must be shareable between threads; one side reads it.
    receiver: Mutex<Receiver<Progress>>,
    stop: Arc<AtomicBool>,
}

/// What `Job::poll` found.
pub enum Poll {
    Progress(Progress),
    /// Nothing new yet.
    Waiting,
    /// The run is over (finished, stopped, or a step crashed: the console says why).
    Over,
}

impl Job {
    /// Starts running `steps[first..=last]` on a copy of `map` (the map as it was before `first`).
    pub fn start(steps: Vec<Box<dyn Step>>, first: usize, last: usize, mut map: Map, seed: u64) -> Self {
        let (sender, receiver) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        std::thread::spawn(move || {
            for (index, step) in steps.iter().enumerate().take(last + 1).skip(first) {
                let started = Instant::now();
                let mut work = Work::new(seed, step.name(), stop_flag.clone());
                if step.run(&mut map, &mut work).is_err() || stop_flag.load(Ordering::Relaxed) {
                    let _ = sender.send(Progress::Stopped);
                    return;
                }
                let millis = started.elapsed().as_secs_f64() * 1000.0;
                let done = Progress::Done { index, map: map.clone(), millis, lines: work.lines };
                if sender.send(done).is_err() {
                    return; // nobody's listening any more
                }
            }
        });
        Self { current: first, receiver: Mutex::new(receiver), stop }
    }

    /// Asks the run to stop as soon as it can.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn poll(&mut self) -> Poll {
        let received = self.receiver.lock().map(|receiver| receiver.try_recv());
        match received {
            Ok(Ok(progress)) => {
                if let Progress::Done { index, .. } = &progress {
                    self.current = index + 1;
                }
                Poll::Progress(progress)
            }
            Ok(Err(TryRecvError::Empty)) => Poll::Waiting,
            _ => Poll::Over,
        }
    }
}

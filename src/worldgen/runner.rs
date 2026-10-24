//! The runner — makes a layer (or anything else slow, like the World Gen plates) on a background
//! thread, so the screen never freezes while you drag a slider.

use super::made::Made;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// What one piece of work took to make, alongside what it made (a layer, unless said otherwise).
pub struct Finished<T = Made> {
    pub made: T,
    pub millis: f64,
}

pub struct Job<T = Made> {
    pub started: Instant,
    // A Mutex only because Bevy resources must be shareable between threads; one side reads it.
    receiver: Mutex<Receiver<Finished<T>>>,
    stop: Arc<AtomicBool>,
}

pub enum Poll<T = Made> {
    Done(Box<Finished<T>>),
    Waiting,
    /// Stopped, or it crashed (the console says why).
    Over,
}

impl<T: Send + 'static> Job<T> {
    /// Runs `work` in the background. It's handed the stop signal, and gives back `None` if it
    /// noticed the signal and gave up.
    pub fn start(work: impl FnOnce(&AtomicBool) -> Option<T> + Send + 'static) -> Self {
        let (sender, receiver) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        std::thread::spawn(move || {
            let started = Instant::now();
            if let Some(made) = work(&stop_flag) {
                let millis = started.elapsed().as_secs_f64() * 1000.0;
                let _ = sender.send(Finished { made, millis });
            }
        });
        Self { started: Instant::now(), receiver: Mutex::new(receiver), stop }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn poll(&self) -> Poll<T> {
        match self.receiver.lock().map(|receiver| receiver.try_recv()) {
            Ok(Ok(finished)) => Poll::Done(Box::new(finished)),
            Ok(Err(TryRecvError::Empty)) => Poll::Waiting,
            _ => Poll::Over,
        }
    }
}

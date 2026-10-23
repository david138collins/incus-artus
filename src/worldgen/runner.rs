//! The runner — makes a field on a background thread, so the screen never freezes while you drag
//! a slider.

use super::noise::{Made, PerlinSettings, make};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// What one field took to make, alongside the field itself.
pub struct Finished {
    pub made: Made,
    pub millis: f64,
}

pub struct Job {
    pub started: Instant,
    // A Mutex only because Bevy resources must be shareable between threads; one side reads it.
    receiver: Mutex<Receiver<Finished>>,
    stop: Arc<AtomicBool>,
}

pub enum Poll {
    Done(Box<Finished>),
    Waiting,
    /// Stopped, or it crashed (the console says why).
    Over,
}

impl Job {
    pub fn start(settings: PerlinSettings, seed: u64, size_blocks: f32, blocks_per_pixel: f32) -> Self {
        let (sender, receiver) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        std::thread::spawn(move || {
            let started = Instant::now();
            if let Some(made) = make(&settings, seed, size_blocks, blocks_per_pixel, &stop_flag) {
                let millis = started.elapsed().as_secs_f64() * 1000.0;
                let _ = sender.send(Finished { made, millis });
            }
        });
        Self { started: Instant::now(), receiver: Mutex::new(receiver), stop }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn poll(&self) -> Poll {
        match self.receiver.lock().map(|receiver| receiver.try_recv()) {
            Ok(Ok(finished)) => Poll::Done(Box::new(finished)),
            Ok(Err(TryRecvError::Empty)) => Poll::Waiting,
            _ => Poll::Over,
        }
    }
}

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
            // Below the frame's priority, like the loading lane's workers.
            crate::runner::hardware::background_thread();
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

/// `f` for every pixel of a `width` × `width` map, on all the computer's cores. `None` if `stop`
/// was set part-way. (For the editor's tools, which you wait on. Work done while you play uses
/// `every_pixel_on`, with a few cores, so the game's own threads are never crowded out.)
pub fn every_pixel<T: Copy + Default + Send>(width: usize, stop: &AtomicBool, f: impl Fn(usize) -> T + Sync) -> Option<Vec<T>> {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    every_pixel_on(cores, width, stop, f)
}

/// The same on `threads` threads. [2026-10-28: the near light window used every core while you
/// flew; Windows then paused the game's own threads to make room, and frames took 50-250 ms.]
pub fn every_pixel_on<T: Copy + Default + Send>(threads: usize, width: usize, stop: &AtomicBool, f: impl Fn(usize) -> T + Sync) -> Option<Vec<T>> {
    let rows_each = width.div_ceil(threads.max(1)).max(1);
    let mut out = vec![T::default(); width * width];
    std::thread::scope(|scope| {
        for (band, part) in out.chunks_mut(rows_each * width).enumerate() {
            let f = &f;
            scope.spawn(move || {
                for (k, value) in part.iter_mut().enumerate() {
                    if k % width == 0 && stop.load(Ordering::Relaxed) {
                        return;
                    }
                    *value = f(band * rows_each * width + k);
                }
            });
        }
    });
    (!stop.load(Ordering::Relaxed)).then_some(out)
}

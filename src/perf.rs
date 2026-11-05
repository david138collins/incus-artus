//! Performance — the gauge for "are we hitting the wall?" [David: "I need to know when we start
//! hitting that wall so I can start hitting the brakes."], and the **slow-frame log** [the regroup,
//! docs/regroup.md: "we need to isolate the offender and not repeat it"].
//!
//! - **Steps time themselves:** the frame's parts (the runner's: the game's update, the panel,
//!   handing the card its work, showing it) and the game's own heavy steps (`Timed`), so a slow
//!   frame says where its time went. Background jobs on the loading lane time themselves too
//!   (`Background`), logged every 10 s.
//! - **Every slow frame is logged** with what ran in it: uncapped, any frame over `SLOW_MS` (8.3 ms:
//!   what an unbreakable 120 allows); under a cap, any frame more than `LATE_MS` past the cap's own
//!   frame (a refresh missed shows as a whole refresh late; a frame a fraction of a millisecond off
//!   the cap is only the clock's wobble). Nothing is written for frames that keep to it.
//! - The "Performance" debug page shows the frame rate (average, 1% low, worst), a graph of the last
//!   few seconds, the slowest time each step took in the last second, and the recent slow frames.
//!
//! All of it is a few clock readings a frame; nothing per frame is drawn or written unless the panel
//! is open or a frame was slow.

use crate::world::{Streaming, VoxelWorld};
use crate::ecs::prelude::*;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

pub struct PerfPlugin;

impl Plugin for PerfPlugin {
    fn build(&self, app: &mut App) {
        use crate::debug::DebugAppExt;
        app.init_resource::<FrameTimes>().init_resource::<StepTimes>().add_systems(Last, log_background).add_debug_page("Performance", performance_debug_page);
    }
}

/// How many recent frames are kept (about five seconds at 240 FPS).
const KEPT: usize = 1200;
/// The target: 240 frames a second.
const BUDGET_MS: f32 = 1000.0 / 240.0;
/// A frame slower than this is logged with what ran in it (ms): what an unbreakable 120 allows.
pub const SLOW_MS: f32 = 1000.0 / 120.0;
/// Under a frame cap, how far past the cap's frame a frame may run before it's logged (ms).
pub const LATE_MS: f32 = 1.0;

/// Recent frame times, in milliseconds, newest last.
#[derive(Resource, Default)]
pub struct FrameTimes(pub VecDeque<f32>);

/// The steps timed in the frame so far (`Timed`).
static STEPS: Mutex<Vec<(&'static str, f32)>> = Mutex::new(Vec::new());

/// Times a step from here to the end of the block it's made in: `let _timed = Timed::new("name");`.
pub struct Timed {
    name: &'static str,
    started: Instant,
}

impl Timed {
    pub fn new(name: &'static str) -> Self {
        Self { name, started: Instant::now() }
    }
}

impl Drop for Timed {
    fn drop(&mut self) {
        let ms = self.started.elapsed().as_secs_f32() * 1000.0;
        if let Ok(mut steps) = STEPS.lock() {
            steps.push((self.name, ms));
        }
    }
}

/// The background jobs' time (off the main thread), by kind: milliseconds and how many, since the
/// last count. Tells which job keeps the worker threads busy, which the frame's steps can't.
static BACKGROUND: Mutex<Vec<(&'static str, f32, u32)>> = Mutex::new(Vec::new());

/// Times a background job from here to the end of the block it's made in, inside the job:
/// `let _timed = Background::new("name");`.
pub struct Background {
    name: &'static str,
    started: Instant,
}

impl Background {
    pub fn new(name: &'static str) -> Self {
        Self { name, started: Instant::now() }
    }
}

impl Drop for Background {
    fn drop(&mut self) {
        let ms = self.started.elapsed().as_secs_f32() * 1000.0;
        if let Ok(mut jobs) = BACKGROUND.lock() {
            match jobs.iter_mut().find(|(n, ..)| *n == self.name) {
                Some(job) => {
                    job.1 += ms;
                    job.2 += 1;
                }
                None => jobs.push((self.name, ms, 1)),
            }
        }
    }
}

/// How often the background jobs' time is logged.
const BACKGROUND_EVERY: f32 = 10.0;

/// Every `BACKGROUND_EVERY` seconds, the log says what the background jobs took, busiest first.
pub fn log_background(time: Res<Time<Real>>, mut last: Local<f32>) {
    let now = time.elapsed_secs();
    if now - *last < BACKGROUND_EVERY {
        return;
    }
    let seconds = now - *last;
    *last = now;
    let Ok(mut jobs) = BACKGROUND.lock() else { return };
    let mut taken = std::mem::take(&mut *jobs);
    drop(jobs);
    if taken.is_empty() {
        return;
    }
    taken.sort_by(|a, b| b.1.total_cmp(&a.1));
    let listed: Vec<String> = taken.iter().map(|(name, ms, count)| format!("{name} {:.2} s ({count})", ms / 1000.0)).collect();
    info!("Background work in the last {seconds:.0} s, worker time (how many): {}", listed.join(", "));
}

/// The steps' times: the slowest of each in the second before last, and the recent slow frames.
#[derive(Resource, Default)]
pub struct StepTimes {
    this_second: Vec<(&'static str, f32)>,
    second_started: Option<Instant>,
    /// The slowest time each step took in the last full second, slowest first.
    pub last_second: Vec<(&'static str, f32)>,
    /// Recent slow frames: their length and their steps, slowest first.
    pub slow: VecDeque<(f32, Vec<(&'static str, f32)>)>,
    /// Slow frames since the start.
    pub slow_count: u64,
    /// The length a frame is slow past, now (ms).
    pub limit: f32,
}

/// At the end of a frame (the runner, once it's shown): its length, and what its steps took. A slow
/// one is logged with them. `capped`: the cap's frame (ms), if there is a cap.
pub fn frame_done(world: &mut World, ms: f32, capped: Option<f32>) {
    let frame: Vec<(&'static str, f32)> = {
        let Ok(mut steps) = STEPS.lock() else { return };
        // The same step can run more than once a frame: added up.
        let mut frame: Vec<(&'static str, f32)> = Vec::new();
        for (name, step) in steps.drain(..) {
            match frame.iter_mut().find(|(n, _)| *n == name) {
                Some(found) => found.1 += step,
                None => frame.push((name, step)),
            }
        }
        frame
    };
    if ms <= 0.0 {
        return;
    }
    let mut times = world.resource_mut::<FrameTimes>();
    times.0.push_back(ms);
    while times.0.len() > KEPT {
        times.0.pop_front();
    }
    let mut steps = world.resource_mut::<StepTimes>();
    steps.limit = capped.map_or(SLOW_MS, |frame| frame + LATE_MS);
    for &(name, step) in &frame {
        match steps.this_second.iter_mut().find(|(n, _)| *n == name) {
            Some(slowest) => slowest.1 = slowest.1.max(step),
            None => steps.this_second.push((name, step)),
        }
    }
    if ms > steps.limit {
        let mut listed = frame;
        listed.sort_by(|a, b| b.1.total_cmp(&a.1));
        let accounted: f32 = listed.iter().map(|(_, step)| step).sum();
        let shown: Vec<String> = listed.iter().filter(|&&(_, step)| step >= 0.05).map(|(name, step)| format!("{name} {step:.2}")).collect();
        info!("Slow frame: {ms:.1} ms; timed {accounted:.1} ms: {}", if shown.is_empty() { "nothing timed".to_string() } else { shown.join(", ") });
        steps.slow.push_back((ms, listed));
        steps.slow_count += 1;
        while steps.slow.len() > 8 {
            steps.slow.pop_front();
        }
    }
    let started = *steps.second_started.get_or_insert_with(Instant::now);
    if started.elapsed().as_secs_f32() >= 1.0 {
        let mut second = std::mem::take(&mut steps.this_second);
        second.sort_by(|a, b| b.1.total_cmp(&a.1));
        steps.last_second = second;
        steps.second_started = Some(Instant::now());
    }
}

/// Average FPS, 1%-low FPS and the worst frame (ms) over the frames kept.
pub fn summary(times: &VecDeque<f32>) -> Option<(f32, f32, f32)> {
    if times.is_empty() {
        return None;
    }
    let total: f32 = times.iter().sum();
    let average = 1000.0 * times.len() as f32 / total;
    let mut sorted: Vec<f32> = times.iter().copied().collect();
    sorted.sort_by(|a, b| b.total_cmp(a));
    // The slowest 1% of frames, averaged.
    let slowest = (sorted.len() / 100).max(1);
    let low = 1000.0 / (sorted[..slowest].iter().sum::<f32>() / slowest as f32);
    Some((average, low, sorted[0]))
}

/// The frame rate in a line: average, 1% low and the worst frame.
pub fn fps_line(world: &World) -> String {
    summary(&world.resource::<FrameTimes>().0).map_or("no frames".to_string(), |(average, low, worst)| format!("{average:.0} fps, 1% low {low:.0}, worst {worst:.1} ms"))
}

pub fn performance_debug_page(ui: &mut egui::Ui, world: &mut World) {
    ui.small("Against the budget: 240 FPS (4.2 ms a frame), and never a frame over 8.3 ms at 120. The 1% low is what you feel as stutter.");
    let times = world.resource::<FrameTimes>().0.clone();
    if let Some((average, low, worst)) = summary(&times) {
        let colour = |fps: f32| if fps >= 240.0 { egui::Color32::from_rgb(120, 220, 120) } else if fps >= 120.0 { egui::Color32::from_rgb(230, 200, 90) } else { egui::Color32::from_rgb(235, 90, 80) };
        ui.horizontal(|ui| {
            ui.colored_label(colour(average), format!("{average:.0} FPS average"));
            ui.colored_label(colour(low), format!("{low:.0} FPS 1% low"));
            ui.colored_label(colour(1000.0 / worst), format!("worst frame {worst:.1} ms"));
        });
        // The graph: one bar per frame, newest on the right.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(420.0), 90.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(20));
        let top_ms = 25.0;
        let y_of = |ms: f32| rect.bottom() - (ms / top_ms).min(1.0) * rect.height();
        for (ms, label) in [(BUDGET_MS, "240"), (SLOW_MS, "120"), (1000.0 / 60.0, "60")] {
            painter.line_segment([egui::pos2(rect.left(), y_of(ms)), egui::pos2(rect.right(), y_of(ms))], egui::Stroke::new(1.0, egui::Color32::from_gray(90)));
            painter.text(egui::pos2(rect.left() + 2.0, y_of(ms) - 1.0), egui::Align2::LEFT_BOTTOM, label, egui::FontId::proportional(9.0), egui::Color32::from_gray(140));
        }
        let shown = times.len().min(rect.width() as usize);
        for (k, &ms) in times.iter().skip(times.len() - shown).enumerate() {
            let x = rect.right() - (shown - k) as f32;
            let colour = if ms <= BUDGET_MS { egui::Color32::from_rgb(90, 180, 90) } else if ms <= SLOW_MS { egui::Color32::from_rgb(220, 190, 80) } else { egui::Color32::from_rgb(230, 80, 70) };
            painter.line_segment([egui::pos2(x, rect.bottom()), egui::pos2(x, y_of(ms))], egui::Stroke::new(1.0, colour));
        }
    }

    ui.separator();
    ui.strong("Where the time goes");
    ui.small("Every step of the frame, the slowest each took in the last second (ms). A slow frame whose steps don't add up to it lost its time outside the game: in Windows, or waiting on the screen.");
    let steps = world.resource::<StepTimes>();
    if steps.last_second.is_empty() {
        ui.small("Nothing timed yet.");
    }
    for (name, ms) in steps.last_second.iter().take(12) {
        ui.label(format!("{ms:6.2}  {name}"));
    }
    ui.small(format!("Slow frames (over {:.1} ms now) since the start: {}. The latest (also in the session log):", steps.limit, steps.slow_count));
    for (ms, listed) in steps.slow.iter().rev() {
        let shown: Vec<String> = listed.iter().filter(|&&(_, step)| step >= 0.05).take(6).map(|(name, step)| format!("{name} {step:.1}")).collect();
        ui.small(format!("{ms:.1} ms: {}", if shown.is_empty() { "nothing timed".to_string() } else { shown.join(", ") }));
    }

    ui.separator();
    ui.strong("Work in flight");
    let streaming = world.resource::<Streaming>();
    ui.label(format!("Chunks waiting: {}   being made: {}", streaming.queued(), streaming.being_made()));
    ui.label(format!("Chunks in the world: {}", world.resource::<VoxelWorld>().chunk_count()));

    ui.separator();
    ui.strong("Temperatures");
    crate::temperatures::temperatures_settings(ui, world);
}

/// The frame rate and how many of each kind of thing the game is holding (a count that keeps
/// climbing on a long flight is a leak), in a line: for the soak test's log.
pub fn soak_line(world: &mut World) -> String {
    let fps = fps_line(world);
    let streaming = world.resource::<Streaming>();
    let (queued, making, made, dropped) = (streaming.queued(), streaming.being_made(), streaming.made, streaming.dropped);
    let chunks = world.resource::<VoxelWorld>().chunk_count();
    let slow = world.resource::<StepTimes>().slow_count;
    let entities = world.query::<Entity>().iter(world).count();
    format!("{fps} | slow frames so far {slow} | chunks {chunks} (queued {queued}, making {making}, made {made}, dropped {dropped}) | entities {entities}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_one_percent_low_is_the_slowest_frames() {
        // 99 frames at 4 ms and one at 20 ms.
        let mut times: VecDeque<f32> = std::iter::repeat_n(4.0, 99).collect();
        times.push_back(20.0);
        let (average, low, worst) = summary(&times).unwrap();
        assert!((average - 1000.0 / 4.16).abs() < 1.0, "{average}");
        assert!((low - 50.0).abs() < 0.01, "{low}");
        assert_eq!(worst, 20.0);
    }
}

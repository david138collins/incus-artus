//! The terrain lab — the World Compiler's view: a live view of the ground block by block, at any
//! zoom, exactly as the Interpreter will make it in the world. Park it over any patch and move the
//! Interpreter's sliders: it follows them as they move.
//!
//! - **The close-up** is the whole view [David, 2026-10-30: the old map view, a pixel every 32
//!   blocks, retired]. It works out every column in view (one sample per block when zoomed right in; every 2nd, 4th… block
//!   further out, so it stays quick), on all the computer's cores, and shows the blocks the world will
//!   have there (mineralogy's choice, in the Climate tab's live climate), in the distant terrain's
//!   colours, lit as it lights them or flat (compiler_view.rs, `block_picture`). Sliders that only
//!   choose blocks repaint it from the heights it has; the rest work the heights out again.
//!   Changing a slider reads the same maps with the new recipe (nothing is copied), so it can redraw
//!   while the slider moves; the whole-map compile catches up behind it.
//! - **Layer switches** leave parts of the recipe out, to see what each does.
//! - **A cross-section:** Shift + drag across the map draws a line; its side view (the ground's Z
//!   along it, with sea level) shows under the map.
//! - **Cost:** how long the last close-up took, and what that comes to per chunk.
//! - **Sharpness:** ½ to 16 samples per screen point (1 to start; ½ for zoomed right out, 16 for
//!   screenshots).

use super::runner::{Job, Poll};
use crate::world::{CompiledMap, InterpreterSettings, Mutes};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The most samples across or down a close-up at sharpness 1× (more and the step between samples
/// grows); sharper close-ups may take up to 16 times as many, up to `LARGEST`.
const MOST_SAMPLES: f32 = 900.0;
/// The largest picture the graphics card is sure to take, across or down.
const LARGEST: f32 = 8192.0;
/// Close-ups are laid on a grid of this many samples, so small pans don't redraw.
const SNAP: f32 = 64.0;

/// What a close-up shows: which recipe, and which blocks.
#[derive(Clone, PartialEq)]
pub struct Request {
    maps: usize,
    rivers: usize,
    settings: InterpreterSettings,
    mutes: Mutes,
    /// The north-west corner (blocks), the blocks between samples, and how many samples.
    pub north: f32,
    pub east: f32,
    pub step: f32,
    pub cols: usize,
    pub rows: usize,
}

/// A finished close-up: the ground's Z at each sample, row by row from the north-west (`NaN` past
/// the map).
pub struct Frame {
    pub request: Request,
    pub heights: Vec<f32>,
    pub millis: f64,
}

#[derive(Default)]
pub struct Lab {
    pub mutes: Mutes,
    /// Close-up samples per screen point: ½, 1, 2, 4, 8 or 16 (as −1 to 4; ½ for zoomed right out).
    pub sharper: i32,
    /// The close-up being made, and what it shows.
    job: Option<(Request, Job<Frame>)>,
    /// The compiler's maps drawn over the map and the close-up.
    pub overlays: super::overlays::Overlays,
    pub frame: Option<Frame>,
    /// Goes up with each new frame (so the picture knows to repaint).
    pub version: u64,
    /// The recipe being shown, for readouts and the cross-section.
    pub map: Option<Arc<CompiledMap>>,
    /// The cross-section's ends (north, east, in blocks), and where a Shift-drag started.
    pub section: Option<((f32, f32), (f32, f32))>,
    pub drawing_from: Option<(f32, f32)>,
}

impl Lab {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    /// The recipe as it is now: the compiled maps, read with these settings and switches.
    pub fn recipe(&mut self, compiled: &Arc<CompiledMap>, settings: &InterpreterSettings) -> Arc<CompiledMap> {
        let current = self.map.as_ref().is_some_and(|map| {
            Arc::ptr_eq(&map.maps, &compiled.maps) && &map.settings == settings && map.mutes == self.mutes
        });
        if !current {
            self.map = Some(Arc::new(compiled.with(settings.clone(), self.mutes)));
        }
        self.map.clone().unwrap()
    }

    /// Asks for the close-up of `wanted` (blocks: north-west corner, south-east corner, and blocks
    /// per screen point), collects finished frames, and starts the next one when the view or the
    /// recipe has changed.
    pub fn drive(&mut self, map: &Arc<CompiledMap>, view: Option<((f32, f32), (f32, f32), f32)>) {
        if let Some((_, job)) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.frame = Some(finished.made);
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting => {}
            }
        }
        let Some(((north, west), (south, east), blocks_per_point)) = view else { return };
        // One sample per screen point (½ to 16 by sharpness), in whole powers of two blocks (so
        // zooming a little doesn't redraw), never finer than a block, and never more than
        // `MOST_SAMPLES` across (more when sharper, up to `LARGEST`).
        let sharper = 2f32.powi(self.sharper.clamp(-1, 4));
        let mut step = (2f32.powi(blocks_per_point.max(1.0).log2().ceil() as i32) / sharper).max(1.0);
        let most = (MOST_SAMPLES * sharper).min(LARGEST - 2.0 * SNAP);
        while ((east - west) / step).max((north - south) / step) > most {
            step *= 2.0;
        }
        let grid = step * SNAP;
        let (top, left) = ((north / grid).ceil() * grid, (west / grid).floor() * grid);
        let (bottom, right) = ((south / grid).floor() * grid, (east / grid).ceil() * grid);
        let request = Request {
            maps: Arc::as_ptr(&map.maps) as usize,
            rivers: Arc::as_ptr(&map.rivers) as usize,
            settings: shape_only(&map.settings),
            mutes: map.mutes,
            north: top,
            east: left,
            step,
            cols: ((right - left) / step).round() as usize,
            rows: ((top - bottom) / step).round() as usize,
        };
        let shown = self.frame.as_ref().is_some_and(|frame| frame.request == request);
        if shown {
            return;
        }
        if let Some((making, job)) = &self.job {
            // Already making this one; or let a quick one finish (the next starts straight after, with
            // the newest view). A slow one for an old view is abandoned.
            if *making == request || job.started.elapsed().as_secs_f32() < 0.15 {
                return;
            }
            job.stop();
            self.job = None;
        }
        let map = map.clone();
        self.job = Some((request.clone(), Job::start(move |stop| close_up(&map, request, stop))));
    }
}

/// The recipe without what only decides which blocks cover the ground (mineralogy, and the heights
/// stone and gabbro start at): moving those only repaints the close-up, its heights kept [David and
/// Counsel, 2026-10-30: move a slider and see the world change at once].
fn shape_only(settings: &InterpreterSettings) -> InterpreterSettings {
    InterpreterSettings { mineralogy: Default::default(), stone_from: 0, gabbro_from: 0, ..settings.clone() }
}

/// Works out a close-up on all the computer's cores.
fn close_up(map: &CompiledMap, request: Request, stop: &AtomicBool) -> Option<Frame> {
    let started = std::time::Instant::now();
    let (cols, rows) = (request.cols, request.rows);
    if cols == 0 || rows == 0 {
        return Some(Frame { request, heights: Vec::new(), millis: 0.0 });
    }
    let mut heights = vec![f32::NAN; cols * rows];
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = rows.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, part) in heights.chunks_mut(rows_each * cols.max(1)).enumerate() {
            let request = &request;
            scope.spawn(move || {
                for (k, value) in part.iter_mut().enumerate() {
                    if k % cols == 0 && stop.load(Ordering::Relaxed) {
                        return;
                    }
                    let (col, row) = (k % cols, band * rows_each + k / cols);
                    let north = request.north - (row as f32 + 0.5) * request.step;
                    let east = request.east + (col as f32 + 0.5) * request.step;
                    if let Some(z) = map.height(north, east) {
                        // Whole blocks, as the world has them, at every zoom: drawn smooth further out,
                        // zooming in one click lost the 1-block terrace lines the map shows (David,
                        // 2026-10-27: "the detail goes *down* when i zoom in").
                        *value = z.round();
                    }
                }
            });
        }
    });
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    Some(Frame { request, heights, millis: started.elapsed().as_secs_f64() * 1000.0 })
}

/// The ground's Z along the cross-section, every `step` blocks or so (at most `most` samples): pairs
/// of (blocks along the line, Z), leaving out what's past the map.
pub fn profile(map: &CompiledMap, from: (f32, f32), to: (f32, f32), most: usize) -> Vec<(f32, f32)> {
    let length = (to.0 - from.0).hypot(to.1 - from.1);
    let samples = (length.ceil() as usize).clamp(2, most);
    (0..samples)
        .filter_map(|k| {
            let t = k as f32 / (samples - 1) as f32;
            let (north, east) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            map.height(north, east).map(|z| (length * t, z))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_close_up_matches_the_interpreter_column_for_column() {
        let width = 16;
        let z: Vec<f32> = (0..width * width).map(|i| 100.0 + ((i % width) * 9 + (i / width) * 3) as f32).collect();
        let map = Arc::new(CompiledMap::of_heights(width, 32.0, &z));
        let mut lab = Lab::default();
        let recipe = lab.recipe(&map, &map.settings.clone());
        // One block per screen point: the close-up samples every block.
        lab.drive(&recipe, Some(((100.0, -100.0), (-100.0, 100.0), 1.0)));
        let started = std::time::Instant::now();
        while lab.frame.is_none() && started.elapsed().as_secs() < 10 {
            lab.drive(&recipe, Some(((100.0, -100.0), (-100.0, 100.0), 1.0)));
        }
        let frame = lab.frame.as_ref().expect("no close-up made");
        assert_eq!(frame.request.step, 1.0);
        // It covers the view, snapped outward to its grid.
        assert!(frame.request.north >= 100.0 && frame.request.east <= -100.0);
        for (row, col) in [(0, 0), (5, 17), (60, 99)] {
            let (north, east) = (frame.request.north - row as f32 - 0.5, frame.request.east + col as f32 + 0.5);
            assert_eq!(frame.heights[row * frame.request.cols + col], map.height(north, east).unwrap().round());
        }
    }

    #[test]
    fn switching_a_layer_off_gives_a_new_recipe_over_the_same_maps() {
        let map = Arc::new(CompiledMap::of_heights(8, 32.0, &[140.0; 64]));
        let mut lab = Lab::default();
        let first = lab.recipe(&map, &map.settings.clone());
        assert!(Arc::ptr_eq(&first, &lab.recipe(&map, &map.settings.clone())), "remade for nothing");
        lab.mutes.torn_coast = true;
        let second = lab.recipe(&map, &map.settings.clone());
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&first.maps, &second.maps), "the maps were copied");
        assert!(second.mutes.torn_coast);
    }

    #[test]
    fn a_profile_runs_along_the_line_at_the_grounds_heights() {
        let width = 8;
        let z: Vec<f32> = (0..width * width).map(|i| 100.0 + (i % width) as f32 * 20.0).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let line = profile(&map, (0.0, -100.0), (0.0, 100.0), 500);
        assert!(line.len() > 150);
        assert!((line.last().unwrap().0 - 200.0).abs() < 1e-3);
        // West to east the ground climbs.
        assert!(line.first().unwrap().1 < line.last().unwrap().1);
    }
}

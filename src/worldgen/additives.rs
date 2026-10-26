//! Additives — the Terrain Compiler's second panel: features added onto the landmass, before
//! erosion and the water (which come later). Dots are scattered from what the maps already know,
//! and will drive terrain features (ridge stamps and the like) once they look right.
//!
//! **Ridge streams**, the first scatterer: dots placed along each range's spine (the crest line the
//! "range spines" overlay shows) stream off it to one or both sides. Each spine pixel knows, from how
//! the mountain map bends round it, which way the ridge runs, which way is off it, and which way and
//! how sharply the spine itself curves; a stream turns toward the inside of that curve as it goes (or
//! away from it), so neighbouring streams curve together and follow the range.
//!
//! Finding the spines needs the whole map, so it's done in the background, once per map and
//! mountain curve; the streams are quick and follow the sliders as they move.

use super::compiler::blur;
use super::overlays::on_spine;
use super::runner::{Job, Poll, every_pixel};
use crate::fate::planet_hash;
use crate::world::{CompiledMap, Spline};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Which sides of the spine the streams leave from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sides {
    #[default]
    Both,
    /// The side the spine curves toward.
    Inside,
    /// The side it curves away from.
    Outside,
}

impl Sides {
    pub const ALL: [Sides; 3] = [Sides::Both, Sides::Inside, Sides::Outside];

    pub fn name(self) -> &'static str {
        match self {
            Sides::Both => "both",
            Sides::Inside => "inside of the bend",
            Sides::Outside => "outside of the bend",
        }
    }
}

/// The Additives panel's settings (saved in the compiler's presets).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdditiveSettings {
    /// Ridge streams: blocks between the streams' starts along a spine.
    pub spacing: f32,
    /// How far each stream runs from the spine, in blocks.
    pub distance: f32,
    /// Dots along each stream.
    pub dots: u32,
    /// How much the spine's curve steers the streams round to sweep along it: 1 = turning as sharply
    /// as the spine curves, 0 = straight off it, negative = sweeping the other way.
    pub bend: f32,
    pub sides: Sides,
    /// Spines only where the mountains are at least this strong (0 to 1).
    pub least_mountains: f32,
}

impl Default for AdditiveSettings {
    fn default() -> Self {
        Self { spacing: 400.0, distance: 1500.0, dots: 16, bend: 1.0, sides: Sides::Both, least_mountains: 0.12 }
    }
}

/// A point on a range's spine, in map pixels (x east, y south, from the map's north-west corner).
#[derive(Clone, Copy, Debug)]
pub struct SpinePoint {
    pub pixel: usize,
    pub at: (f32, f32),
    /// Straight off the ridge (one way; the other side is the opposite).
    pub across: (f32, f32),
    /// Toward the inside of the spine's curve, as long as the curve is sharp (1 / its radius, in
    /// pixels).
    pub bend: (f32, f32),
    /// How strong the mountains are here (0 to 1).
    pub share: f32,
}

/// A dot: where it is (map pixels) and how far along its stream (0 at the spine, 1 at the end).
#[derive(Clone, Copy, Debug)]
pub struct Dot {
    pub at: (f32, f32),
    pub along: f32,
}

/// What the spines were found from: which maps, and the mountain curve.
#[derive(Clone, PartialEq)]
struct Key {
    maps: usize,
    mountains: Spline,
}

pub struct Spines {
    key: Key,
    pub points: Vec<SpinePoint>,
}

pub struct Additives {
    /// Whether the dots are drawn over the map.
    pub show: bool,
    job: Option<(Key, Job<Spines>)>,
    pub spines: Option<Spines>,
    /// The dots, and the settings and spines (by version) they were made from.
    pub dots: Vec<Dot>,
    pub streams: usize,
    made_from: Option<(u64, u64, AdditiveSettings, f32)>,
    version: u64,
}

impl Default for Additives {
    fn default() -> Self {
        Self { show: true, job: None, spines: None, dots: Vec::new(), streams: 0, made_from: None, version: 0 }
    }
}

impl Additives {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    /// Collects finished spines, starts finding them again when the maps or the mountain curve have
    /// changed, and makes the dots again when the settings have.
    pub fn drive(&mut self, map: &Arc<CompiledMap>, settings: &AdditiveSettings, blocks_per_pixel: f32) {
        if let Some((_, job)) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.spines = Some(finished.made);
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting => {}
            }
        }
        if !self.show {
            return;
        }
        let key = Key { maps: Arc::as_ptr(&map.maps) as usize, mountains: map.settings.mountains.clone() };
        let current = self.spines.as_ref().is_some_and(|spines| spines.key == key);
        if !current {
            let making = self.job.as_ref().is_some_and(|(making, _)| *making == key);
            let quick = self.job.as_ref().is_some_and(|(_, job)| job.started.elapsed().as_secs_f32() < 0.15);
            if !making && !quick {
                if let Some((_, job)) = self.job.take() {
                    job.stop();
                }
                let (map, making) = (map.clone(), key.clone());
                self.job = Some((key, Job::start(move |stop| find_spines(&map, making, stop))));
            }
        }
        let Some(spines) = &self.spines else { return };
        let wanted = (self.version, map.seed, settings.clone(), blocks_per_pixel);
        if self.made_from.as_ref() != Some(&wanted) {
            (self.dots, self.streams) = ridge_streams(&spines.points, settings, map.seed, blocks_per_pixel);
            self.made_from = Some(wanted);
        }
    }
}

/// Finds every spine pixel on the map, on all the computer's cores. `None` if `stop` was set.
fn find_spines(map: &CompiledMap, key: Key, stop: &AtomicBool) -> Option<Spines> {
    let width = map.maps.width;
    let share = every_pixel(width, stop, |i| map.mountain_share(&map.pixel(i)))?;
    Some(Spines { key, points: spine_points(&share, width) })
}

/// The spine points of a mountain-share map (lightly smoothed first, as the overlay does, so
/// single-pixel wobbles don't break the spines into dashes).
pub fn spine_points(share: &[f32], width: usize) -> Vec<SpinePoint> {
    let mut smooth = share.to_vec();
    blur(&mut smooth, width, 1);
    let at = |col: usize, row: usize| smooth[row.min(width - 1) * width + col.min(width - 1)];
    // The way the ridge runs at a pixel: the direction the share bends down least (the other way,
    // across the ridge, it bends down most).
    let along = |col: usize, row: usize| {
        let (col, row) = (col.clamp(1, width - 2), row.clamp(1, width - 2));
        let here = at(col, row);
        let xx = at(col + 1, row) - 2.0 * here + at(col - 1, row);
        let yy = at(col, row + 1) - 2.0 * here + at(col, row - 1);
        let xy = (at(col + 1, row + 1) - at(col + 1, row - 1) - at(col - 1, row + 1) + at(col - 1, row - 1)) / 4.0;
        let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
        (angle.cos(), angle.sin())
    };
    // How far along the spine (pixels) the curve is measured over, each way.
    const REACH: f32 = 2.0;
    let pixel_at = |(x, y): (f32, f32)| ((x.max(0.0) as usize).min(width - 1), (y.max(0.0) as usize).min(width - 1));
    let mut points = Vec::new();
    for row in 1..width.saturating_sub(1) {
        for col in 1..width - 1 {
            if !on_spine(&smooth, width, col, row) {
                continue;
            }
            let t = along(col, row);
            let at_pixel = (col as f32 + 0.5, row as f32 + 0.5);
            // On the outside slope of a curving range, the line straight down the slope is the
            // highest across too, but only just: a spine bends down across far more sharply than
            // the ground there is steep.
            let here = at(col, row);
            let (east, south) = ((at(col + 1, row) - at(col - 1, row)) / 2.0, (at(col, row + 1) - at(col, row - 1)) / 2.0);
            let steepness = (east * east + south * south).sqrt();
            let across_bend = [1.0f32, -1.0].map(|way| {
                let (c, r) = pixel_at((at_pixel.0 - way * t.1, at_pixel.1 + way * t.0));
                here - at(c, r)
            });
            if across_bend[0] + across_bend[1] < 0.5 * steepness {
                continue;
            }
            // The ridge's direction a little way ahead and behind, turned to point the same way.
            let [ahead, behind] = [1.0f32, -1.0].map(|way| {
                let (c, r) = pixel_at((at_pixel.0 + way * REACH * t.0, at_pixel.1 + way * REACH * t.1));
                let d = along(c, r);
                if d.0 * t.0 + d.1 * t.1 < 0.0 { (-d.0, -d.1) } else { d }
            });
            let bend = ((ahead.0 - behind.0) / (2.0 * REACH), (ahead.1 - behind.1) / (2.0 * REACH));
            points.push(SpinePoint { pixel: row * width + col, at: at_pixel, across: (-t.1, t.0), bend, share: share[row * width + col] });
        }
    }
    points
}

/// A number from 0 to 1, the same each time for the same seed and spine pixel.
fn unit(seed: u64, pixel: usize) -> f32 {
    (planet_hash(seed, 0xadd17e, pixel as i64, 0, 0) >> 40) as f32 / (1u64 << 24) as f32
}

/// The ridge streams' dots, and how many streams.
pub fn ridge_streams(points: &[SpinePoint], settings: &AdditiveSettings, seed: u64, blocks_per_pixel: f32) -> (Vec<Dot>, usize) {
    // A spine pixel is about a pixel of spine long, so keeping this share of them leaves a start
    // every `spacing` blocks, scattered.
    let keep = blocks_per_pixel / settings.spacing.max(1.0);
    let dots = settings.dots.max(1);
    let step = settings.distance.max(0.0) / blocks_per_pixel / dots as f32;
    let mut out = Vec::new();
    let mut streams = 0;
    for point in points.iter().filter(|p| p.share >= settings.least_mountains && unit(seed, p.pixel) < keep) {
        // Which way along the ridge the streams sweep: the bend turned a quarter turn, so it's the
        // same way round for every stream off a curve (a pinwheel), on both sides; as long as the
        // curve is sharp, times `bend`.
        let sweep = (-point.bend.1 * settings.bend, point.bend.0 * settings.bend);
        for way in [1.0f32, -1.0] {
            let mut heading = (point.across.0 * way, point.across.1 * way);
            let inside = heading.0 * point.bend.0 + heading.1 * point.bend.1 >= 0.0;
            match settings.sides {
                Sides::Inside if !inside => continue,
                Sides::Outside if inside => continue,
                _ => {}
            }
            streams += 1;
            let mut at = point.at;
            for k in 0..dots {
                // Turn toward the sweep, as sharply as the spine curves (times `bend`), then step.
                heading = (heading.0 + sweep.0 * step, heading.1 + sweep.1 * step);
                let length = (heading.0 * heading.0 + heading.1 * heading.1).sqrt().max(1e-6);
                heading = (heading.0 / length, heading.1 / length);
                at = (at.0 + heading.0 * step, at.1 + heading.1 * step);
                out.push(Dot { at, along: (k + 1) as f32 / dots as f32 });
            }
        }
    }
    (out, streams)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring of mountains, `radius` pixels round the middle of a `width` map.
    fn ring(width: usize, radius: f32) -> Vec<f32> {
        let middle = width as f32 / 2.0;
        (0..width * width)
            .map(|i| {
                let (x, y) = ((i % width) as f32 + 0.5 - middle, (i / width) as f32 + 0.5 - middle);
                let off = (x * x + y * y).sqrt() - radius;
                (-(off * off) / 18.0).exp()
            })
            .collect()
    }

    #[test]
    fn a_ring_shaped_range_bends_toward_its_middle_as_sharply_as_it_curves() {
        let (width, radius) = (200, 60.0);
        let points = spine_points(&ring(width, radius), width);
        assert!(points.len() > 300, "only {} spine points round the ring", points.len());
        let middle = width as f32 / 2.0;
        let mut sharpness = Vec::new();
        for p in &points {
            let inward = (middle - p.at.0, middle - p.at.1);
            assert!(p.bend.0 * inward.0 + p.bend.1 * inward.1 > 0.0, "a bend points away from the middle at {:?}", p.at);
            // Across the ridge is along the radius.
            let length = (inward.0 * inward.0 + inward.1 * inward.1).sqrt();
            assert!((p.across.0 * inward.0 + p.across.1 * inward.1).abs() / length > 0.9);
            sharpness.push((p.bend.0 * p.bend.0 + p.bend.1 * p.bend.1).sqrt());
        }
        sharpness.sort_by(f32::total_cmp);
        let typical = sharpness[sharpness.len() / 2];
        assert!((typical * radius - 1.0).abs() < 0.3, "curve {typical} for a ring of radius {radius}");
    }

    #[test]
    fn streams_off_a_ring_curve_inward_and_go_as_far_as_asked() {
        let (width, radius, blocks) = (200, 60.0, 32.0);
        let points = spine_points(&ring(width, radius), width);
        let settings = AdditiveSettings { spacing: 32.0 * 8.0, distance: 32.0 * 20.0, dots: 10, bend: 0.0, sides: Sides::Inside, least_mountains: 0.1 };
        let middle = width as f32 / 2.0;
        let from_middle = |(x, y): (f32, f32)| ((x - middle).powi(2) + (y - middle).powi(2)).sqrt();
        // Straight, inward: each stream's last dot is 20 pixels in.
        let (dots, streams) = ridge_streams(&points, &settings, 7, blocks);
        assert!(streams > 20 && dots.len() == streams * 10);
        for end in dots.iter().filter(|d| d.along == 1.0) {
            assert!((from_middle(end.at) - (radius - 20.0)).abs() < 2.0, "ended {} from the middle", from_middle(end.at));
        }
        // Bent, every stream (either side) sweeps round the ring the same way, and the other way
        // when the bend is negative.
        let turns = |bend: f32| {
            let (dots, _) = ridge_streams(&points, &AdditiveSettings { bend, sides: Sides::Both, ..settings.clone() }, 7, blocks);
            dots.chunks(10)
                .map(|stream| {
                    let (first, last) = (stream[0].at, stream[9].at);
                    let (x, y) = (first.0 - middle, first.1 - middle);
                    (x * (last.1 - first.1) - y * (last.0 - first.0)).signum()
                })
                .collect::<Vec<f32>>()
        };
        let (one_way, other_way) = (turns(3.0), turns(-3.0));
        assert!(one_way.iter().all(|&s| s == one_way[0]), "streams sweep both ways round");
        assert!(other_way.iter().all(|&s| s == -one_way[0]));
    }
}

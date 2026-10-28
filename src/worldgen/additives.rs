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
//! **Grain drift**, the second: seeds scattered over the mountains (more where they're stronger)
//! send dots drifting along the mountains' grain, the way the range runs, spreading sideways and
//! thinning out as they go. Where the mountains are a knot rather than a string, the dots just
//! scatter round their seed instead; in between, a blend of the two.
//!
//! Finding the spines and the grain needs the whole map, so it's done in the background, once per
//! map, mountain curve and grain setting; the dots are quick and follow the sliders as they move.

use super::compiler::blur;
use super::overlays::on_spine;
use super::runner::{Job, Poll, every_pixel};
use crate::fate::planet_hash;
use crate::world::{CompiledMap, Maps, Spline};
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

/// Which way along the grain the grain drift's dots go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftWay {
    /// Toward the tip of the range the seed is in: its thin end, the way it reaches furthest from
    /// its middle.
    #[default]
    Tip,
    /// The way nearer a compass bearing, the same for every range.
    Bearing,
    Both,
}

impl DriftWay {
    pub const ALL: [DriftWay; 3] = [DriftWay::Tip, DriftWay::Bearing, DriftWay::Both];

    pub fn name(self) -> &'static str {
        match self {
            DriftWay::Tip => "to the range's tip",
            DriftWay::Bearing => "to a bearing",
            DriftWay::Both => "both ways",
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
    /// The mountains' grain: how far round each spot (blocks) is looked at to tell which way the
    /// range runs, and how much that comes from the squeeze (1) rather than the mountains' shape (0).
    pub grain_reach: f32,
    pub grain_from_squeeze: f32,
    /// Whether each scatterer makes its dots.
    pub streams_on: bool,
    pub drift_on: bool,
    /// Grain drift: about how many blocks apart the seeds are where the mountains are strongest
    /// (fewer where they're weaker), and none where they're below `drift_least`.
    pub drift_spacing: f32,
    pub drift_least: f32,
    /// And none where they're stronger than this.
    pub drift_most: f32,
    /// How far the dots drift from their seed along the grain, in blocks, and how many per seed.
    pub drift_distance: f32,
    pub drift_dots: u32,
    /// How far they spread sideways by the far end, as a share of the distance.
    pub drift_spread: f32,
    /// Which side they spread to: -1 all to the left (of the way they're drifting), 0 both, 1 all
    /// to the right.
    pub drift_lean: f32,
    /// How many fewer dots there are toward the far end (0 = as many, 1 = none at the very end).
    pub drift_thin: f32,
    /// Which way along the grain the dots drift: toward their range's tip, the way nearer
    /// `drift_toward` (a compass bearing, degrees, 0 = north, 90 = east), or both ways.
    pub drift_way: DriftWay,
    pub drift_toward: f32,
    /// Where the mountains are weaker than this (0 to 1), they're cut into separate ranges.
    pub range_cut: f32,
    /// How hard each range pulls its dots toward its tip as they drift (0 = not at all, following
    /// the grain only; 1 = straight at it), as much as the range is lopsided.
    pub tip_pull: f32,
    /// How much each range's dots come together into one stream (0 = as scattered as the other
    /// settings make them; 1 = drawn right in to the range's middle line, with no spread or knot
    /// scatter).
    pub coherence: f32,
    /// In knots, how far round the seed the dots scatter, as a share of the distance.
    pub knot_scatter: f32,
    /// The spiral: every dot turns to swirl in round the map's middle, harder the further it has
    /// gone. `spiral` is how hard (positive clockwise, negative anticlockwise), `spiral_exponent`
    /// how the turning grows along the way (1 = steadily; above 1 = little at first, then hard;
    /// below 1 = early, then easing).
    pub spiral: f32,
    pub spiral_exponent: f32,
}

impl Default for AdditiveSettings {
    fn default() -> Self {
        Self {
            streams_on: false,
            drift_on: true,
            drift_spacing: 1_000.0,
            drift_least: 0.25,
            drift_distance: 2_000.0,
            drift_dots: 12,
            drift_spread: 0.35,
            drift_lean: 0.0,
            drift_thin: 0.5,
            drift_way: DriftWay::Tip,
            drift_toward: 180.0,
            range_cut: 0.2,
            tip_pull: 0.4,
            coherence: 0.0,
            spiral: 0.0,
            spiral_exponent: 1.0,
            drift_most: 1.0,
            knot_scatter: 0.5,
            spacing: 400.0,
            distance: 1500.0,
            dots: 16,
            bend: 1.0,
            sides: Sides::Both,
            least_mountains: 0.12,
            grain_reach: 1_500.0,
            grain_from_squeeze: 0.5,
        }
    }
}

/// The mountains' grain at each map pixel: which way the range runs (an angle, radians, east = 0,
/// turning toward south), and how string-like it is there (1 = a clear range running that way, 0 =
/// a knot or bundle with no one direction).
pub struct Grain {
    pub angle: Vec<f32>,
    pub string: Vec<f32>,
}

/// Works out the grain from the mountain share. The shape's opinion: round each spot (`reach`
/// pixels), do the slopes all face across one line (a string, running along that line) or every
/// which way (a knot)? (The "structure tensor".) The squeeze's opinion: ranges run across the way the
/// rock was pushed. `from_squeeze` blends the two directions; how string-like is the shape's alone.
pub fn grain(share: &[f32], maps: &Maps, reach: usize, from_squeeze: f32) -> Grain {
    let width = maps.width;
    let mut smooth = share.to_vec();
    blur(&mut smooth, width, 1);
    let at = |col: usize, row: usize| smooth[row.min(width - 1) * width + col.min(width - 1)];
    // Each pixel's slope, as the three products the tensor is made of, gathered over the reach.
    let (mut xx, mut xy, mut yy) = (vec![0.0f32; width * width], vec![0.0f32; width * width], vec![0.0f32; width * width]);
    for row in 0..width {
        for col in 0..width {
            let east = (at(col + 1, row) - at(col.saturating_sub(1), row)) / 2.0;
            let south = (at(col, row + 1) - at(col, row.saturating_sub(1))) / 2.0;
            let i = row * width + col;
            (xx[i], xy[i], yy[i]) = (east * east, east * south, south * south);
        }
    }
    for field in [&mut xx, &mut xy, &mut yy] {
        blur(field, width, reach.max(1));
    }
    let mut angle = vec![0.0f32; width * width];
    let mut string = vec![0.0f32; width * width];
    for i in 0..width * width {
        // Directions as doubled angles (half a turn apart is the same direction). The slopes face
        // across the range, so the range runs a quarter turn round: the doubled arrow reversed.
        let total = xx[i] + yy[i];
        let shape = if total > 1e-12 { ((yy[i] - xx[i]) / total, -2.0 * xy[i] / total) } else { (0.0, 0.0) };
        string[i] = (shape.0 * shape.0 + shape.1 * shape.1).sqrt().min(1.0);
        // The push, from north-up to south-down, then turned a quarter turn.
        let push = (maps.push_x[i], -maps.push_y[i]);
        let length = (push.0 * push.0 + push.1 * push.1).sqrt();
        let squeeze = if length > 1e-9 { (-push.0 / length, -push.1 / length) } else { (0.0, 0.0) };
        let blend = (shape.0 * (1.0 - from_squeeze) + squeeze.0 * from_squeeze, shape.1 * (1.0 - from_squeeze) + squeeze.1 * from_squeeze);
        angle[i] = blend.1.atan2(blend.0) / 2.0;
    }
    Grain { angle, string }
}

/// One range: a blob of mountains, cut off from the others where they're weaker than the range cut.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    /// Its middle (weighted by how strong the mountains are), in map pixels.
    pub centre: (f32, f32),
    /// Toward its tip: along its length, the way it reaches furthest from its middle (a carrot's
    /// thin end).
    pub tip: (f32, f32),
    /// How lopsided it is that way: 0 = the same both ends (an egg, a sausage), 1 = a carrot or more.
    pub lopsided: f32,
    /// How many pixels it covers.
    pub size: usize,
    /// About how long it is, end to end along its main axis (pixels).
    pub length: f32,
    /// Its strongest spot (0 to 1).
    pub peak: f32,
}

/// The ranges, and which one each map pixel belongs to (`NONE` where none).
pub struct Ranges {
    pub of_pixel: Vec<u32>,
    pub list: Vec<Range>,
}

impl Ranges {
    pub const NONE: u32 = u32::MAX;
}

/// Cuts the mountains into ranges where the share is below `cut`, and finds each one's middle and
/// tip. The tip: the range's length runs along its main axis (the way its mountains spread out
/// most). Round the middle, the mountains always balance (that's what the middle is), but how far
/// they *reach* doesn't: a carrot's thin end trails further out than its fat end. The skew (the
/// cubes of the distances along the axis, which count far-out mountains most) says which way.
pub fn find_ranges(share: &[f32], width: usize, cut: f32) -> Ranges {
    let mut of_pixel = vec![Ranges::NONE; share.len()];
    let mut list = Vec::new();
    let mut stack = Vec::new();
    let mut members = Vec::new();
    for start in 0..share.len() {
        if of_pixel[start] != Ranges::NONE || share[start] < cut {
            continue;
        }
        // Flood out from here through every mountain pixel touching it.
        let label = list.len() as u32;
        of_pixel[start] = label;
        stack.push(start);
        members.clear();
        while let Some(i) = stack.pop() {
            members.push(i);
            let (col, row) = (i % width, i / width);
            let neighbours = [(col > 0).then(|| i - 1), (col + 1 < width).then(|| i + 1), (row > 0).then(|| i - width), (row + 1 < width).then(|| i + width)];
            for n in neighbours.into_iter().flatten() {
                if of_pixel[n] == Ranges::NONE && share[n] >= cut {
                    of_pixel[n] = label;
                    stack.push(n);
                }
            }
        }
        let place = |i: usize| ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
        let weight: f32 = members.iter().map(|&i| share[i]).sum();
        let (mut cx, mut cy) = (0.0, 0.0);
        for &i in &members {
            let (x, y) = place(i);
            (cx, cy) = (cx + x * share[i], cy + y * share[i]);
        }
        let centre = (cx / weight, cy / weight);
        // The main axis, from how the mountains spread round the middle.
        let (mut xx, mut xy, mut yy) = (0.0f32, 0.0f32, 0.0f32);
        for &i in &members {
            let (x, y) = place(i);
            let (dx, dy) = (x - centre.0, y - centre.1);
            (xx, xy, yy) = (xx + dx * dx * share[i], xy + dx * dy * share[i], yy + dy * dy * share[i]);
        }
        let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
        let axis = (angle.cos(), angle.sin());
        // Along the axis: the spread (squares) and the skew (cubes).
        let (mut spread, mut skew) = (0.0f32, 0.0f32);
        for &i in &members {
            let (x, y) = place(i);
            let along = (x - centre.0) * axis.0 + (y - centre.1) * axis.1;
            spread += along * along * share[i];
            skew += along * along * along * share[i];
        }
        let (spread, skew) = (spread / weight, skew / weight);
        let skewness = if spread > 1e-6 { skew / spread.powf(1.5) } else { 0.0 };
        let way = if skewness >= 0.0 { 1.0 } else { -1.0 };
        // A carrot (thickness falling off evenly to a point) has a skewness of about 0.57.
        // (And an even bar's spread is its length / 3.5.)
        list.push(Range {
            centre,
            tip: (axis.0 * way, axis.1 * way),
            lopsided: (skewness.abs() / 0.57).min(1.0),
            size: members.len(),
            length: 3.5 * spread.sqrt(),
            peak: members.iter().map(|&i| share[i]).fold(0.0, f32::max),
        });
    }
    Ranges { of_pixel, list }
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

/// What the spines and grain were found from: which maps, the mountain curve, and the grain's reach
/// and share from the squeeze (as bits).
#[derive(Clone, PartialEq)]
struct Key {
    maps: usize,
    mountains: Spline,
    grain: (u32, u32),
}

/// What's found over the whole map for the dots: the mountain share, its spines and its grain.
pub struct Found {
    key: Key,
    pub points: Vec<SpinePoint>,
    pub share: Vec<f32>,
    pub grain: Grain,
}

pub struct Additives {
    /// Whether the dots are drawn over the map, and how many times their usual size.
    pub show: bool,
    pub dot_size: f32,
    job: Option<(Key, Job<Found>)>,
    pub found: Option<Found>,
    /// The ridge streams' dots, and how many streams.
    pub dots: Vec<Dot>,
    pub streams: usize,
    /// The grain drift's seeds and dots, and the ranges (cut where, from which find, by version).
    pub seeds: Vec<(f32, f32)>,
    pub drift: Vec<Dot>,
    pub ranges: Option<(u64, u32, Ranges)>,
    /// The settings and what was found (by version) the dots were made from.
    made_from: Option<(u64, u64, AdditiveSettings, f32)>,
    version: u64,
}

impl Default for Additives {
    fn default() -> Self {
        Self { show: true, dot_size: 1.0, job: None, found: None, dots: Vec::new(), streams: 0, seeds: Vec::new(), drift: Vec::new(), ranges: None, made_from: None, version: 0 }
    }
}

impl Additives {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    /// Collects finished spines and grain, starts finding them again when the maps, the mountain
    /// curve or the grain settings have changed, and makes the dots again when the settings have.
    pub fn drive(&mut self, map: &Arc<CompiledMap>, settings: &AdditiveSettings, blocks_per_pixel: f32) {
        if let Some((_, job)) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.found = Some(finished.made);
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
        let key = Key {
            maps: Arc::as_ptr(&map.maps) as usize,
            mountains: map.settings.mountains.clone(),
            grain: (settings.grain_reach.to_bits(), settings.grain_from_squeeze.to_bits()),
        };
        let current = self.found.as_ref().is_some_and(|found| found.key == key);
        if !current {
            let making = self.job.as_ref().is_some_and(|(making, _)| *making == key);
            let quick = self.job.as_ref().is_some_and(|(_, job)| job.started.elapsed().as_secs_f32() < 0.15);
            if !making && !quick {
                if let Some((_, job)) = self.job.take() {
                    job.stop();
                }
                let (map, making) = (map.clone(), key.clone());
                self.job = Some((key, Job::start(move |stop| find(&map, making, stop))));
            }
        }
        let Some(found) = &self.found else { return };
        let wanted = (self.version, map.seed, settings.clone(), blocks_per_pixel);
        if self.made_from.as_ref() != Some(&wanted) {
            (self.dots, self.streams) =
                if settings.streams_on { ridge_streams(&found.points, settings, map.seed, blocks_per_pixel) } else { (Vec::new(), 0) };
            let cut = settings.range_cut.to_bits();
            if !self.ranges.as_ref().is_some_and(|(version, at, _)| *version == self.version && *at == cut) {
                self.ranges = Some((self.version, cut, find_ranges(&found.share, map.maps.width, settings.range_cut)));
            }
            let ranges = &self.ranges.as_ref().expect("just made").2;
            (self.seeds, self.drift) = if settings.drift_on {
                grain_drift(&found.share, &found.grain, ranges, map.maps.width, settings, map.seed, blocks_per_pixel)
            } else {
                (Vec::new(), Vec::new())
            };
            self.made_from = Some(wanted);
        }
    }
}

/// Finds the mountain share, every spine pixel and the grain, on all the computer's cores. `None`
/// if `stop` was set.
fn find(map: &CompiledMap, key: Key, stop: &AtomicBool) -> Option<Found> {
    let width = map.maps.width;
    let share = every_pixel(width, stop, |i| map.mountain_share(&map.pixel(i)))?;
    let (reach, from_squeeze) = (f32::from_bits(key.grain.0), f32::from_bits(key.grain.1));
    let grain = grain(&share, &map.maps, (reach / map.maps.blocks_per_pixel).round() as usize, from_squeeze);
    Some(Found { key, points: spine_points(&share, width), share, grain })
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

/// The `n`th number from 0 to 1 for dot `k` of the seed at `pixel`, the same each time.
fn dice(seed: u64, pixel: usize, k: u32, n: u32) -> f32 {
    (planet_hash(seed, 0xd21f7, pixel as i64, k as i64, n as i64) >> 40) as f32 / (1u64 << 24) as f32
}

/// Scatters points (pixel indices) over a 0-to-1 map such as the mountain share: about one every
/// `spacing` blocks each way where it's 1, fewer as it weakens toward `least`, none below `least`
/// or above `most`. The same each time for the same `seed`. (The grain drift's seeds and the
/// lift's points both come from here.)
pub fn pick_seeds(share: &[f32], spacing: f32, least: f32, most: f32, seed: u64, blocks_per_pixel: f32) -> Vec<usize> {
    let per_pixel = (blocks_per_pixel / spacing.max(1.0)).powi(2);
    let least = least.min(0.99);
    (0..share.len())
        .filter(|&i| {
            let strength = ((share[i] - least) / (1.0 - least)).clamp(0.0, 1.0);
            strength > 0.0 && share[i] <= most && dice(seed, i, 0, 0) < per_pixel * strength
        })
        .collect()
}

/// The grain drift's seeds and dots (map pixels, x east and y south).
pub fn grain_drift(share: &[f32], grain: &Grain, ranges: &Ranges, width: usize, settings: &AdditiveSettings, seed: u64, blocks_per_pixel: f32) -> (Vec<(f32, f32)>, Vec<Dot>) {
    let length = settings.drift_distance.max(0.0) / blocks_per_pixel;
    // The path along the grain is followed in steps of about a pixel and a half (32 at least, 1024
    // at most).
    let steps = ((length / 1.5).ceil() as usize).clamp(32, 1024);
    let step = length / steps as f32;
    let coherence = settings.coherence.clamp(0.0, 1.0);
    // The spiral swirls in round the map's middle: its dots head this far round from straight in
    // (60 degrees, so mostly round, a little in), turning toward that by up to this much a pixel.
    let middle = (width as f32 / 2.0, width as f32 / 2.0);
    let swirl = 60f32.to_radians() * -settings.spiral.signum();
    let (swirl_cos, swirl_sin) = (swirl.cos(), swirl.sin());
    const SPIRAL_TURN: f32 = 0.05;
    let bearing = settings.drift_toward.to_radians();
    let toward = (bearing.sin(), -bearing.cos());
    let pixel_at = |(x, y): (f32, f32)| (y.clamp(0.0, width as f32 - 1.0) as usize) * width + x.clamp(0.0, width as f32 - 1.0) as usize;
    let (mut seeds, mut dots) = (Vec::new(), Vec::new());
    for i in pick_seeds(share, settings.drift_spacing, settings.drift_least, settings.drift_most, seed, blocks_per_pixel) {
        let start = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
        seeds.push(start);
        // The range this seed is in: its tip, and how hard it pulls toward it (none if the seed is
        // in no range, or a range that's the same both ends).
        let range = ranges.list.get(ranges.of_pixel[i] as usize);
        let tip = range.map_or((0.0, 0.0), |r| r.tip);
        let pull = range.map_or(0.0, |r| r.lopsided) * settings.tip_pull.clamp(0.0, 1.0);
        // The way the range runs here, pointing toward the tip (or the bearing, outside any range),
        // or nearer the bearing, or both ways.
        let runs = (grain.angle[i].cos(), grain.angle[i].sin());
        let goal = match (settings.drift_way, range) {
            (DriftWay::Tip, Some(_)) => tip,
            _ => toward,
        };
        let forward = if runs.0 * goal.0 + runs.1 * goal.1 >= 0.0 { runs } else { (-runs.0, -runs.1) };
        let ways: &[f32] = if settings.drift_way == DriftWay::Both { &[1.0, -1.0] } else { &[1.0] };
        // How string-like the range is at the seed: 1 drifts along it, 0 just scatters round it
        // (and coherence makes knots drift too).
        let string = grain.string[i] + (1.0 - grain.string[i]) * coherence;
        // Coherence draws the dots in toward the range's middle line (through its middle, toward its
        // tip) as they travel: by this much each step, so over one range's length they close in
        // most of the way at full coherence.
        let close_in = range.map_or(0.0, |r| 1.0 - (-3.0 * coherence * step / r.length.max(1.0)).exp());
        let line = range.map_or(((0.0, 0.0), (0.0, 0.0)), |r| (r.centre, (-r.tip.1, r.tip.0)));
        for (w, &way) in ways.iter().enumerate() {
            // The path: a step at a time, turning to follow the grain wherever there are mountains
            // (and keeping on straight where there aren't), and pulled toward the range's tip all
            // the way, so past the range's end it carries on out beyond the tip; and drawn in toward
            // the range's middle line, so the dots from all over a range come together.
            let mut heading = (forward.0 * way, forward.1 * way);
            let mut path = vec![(start, heading)];
            let mut at = start;
            for s in 0..steps {
                let p = pixel_at(at);
                if share[p] > 0.02 {
                    let g = (grain.angle[p].cos(), grain.angle[p].sin());
                    heading = if g.0 * heading.0 + g.1 * heading.1 >= 0.0 { g } else { (-g.0, -g.1) };
                }
                if pull > 0.0 {
                    let pulled = (heading.0 * (1.0 - pull) + tip.0 * pull, heading.1 * (1.0 - pull) + tip.1 * pull);
                    let length = (pulled.0 * pulled.0 + pulled.1 * pulled.1).sqrt();
                    if length > 1e-6 {
                        heading = (pulled.0 / length, pulled.1 / length);
                    }
                }
                if settings.spiral != 0.0 {
                    let gone = s as f32 / steps as f32;
                    let turn = (settings.spiral.abs() * gone.powf(settings.spiral_exponent.max(0.01)) * SPIRAL_TURN * step).min(1.0);
                    let inward = (middle.0 - at.0, middle.1 - at.1);
                    let far = (inward.0 * inward.0 + inward.1 * inward.1).sqrt();
                    if far > 1e-3 {
                        let inward = (inward.0 / far, inward.1 / far);
                        let target = (inward.0 * swirl_cos - inward.1 * swirl_sin, inward.0 * swirl_sin + inward.1 * swirl_cos);
                        let turned = (heading.0 + (target.0 - heading.0) * turn, heading.1 + (target.1 - heading.1) * turn);
                        let length = (turned.0 * turned.0 + turned.1 * turned.1).sqrt();
                        if length > 1e-6 {
                            heading = (turned.0 / length, turned.1 / length);
                        }
                    }
                }
                at = (at.0 + heading.0 * step, at.1 + heading.1 * step);
                if close_in > 0.0 {
                    let ((cx, cy), across) = line;
                    let off = (at.0 - cx) * across.0 + (at.1 - cy) * across.1;
                    at = (at.0 - across.0 * off * close_in, at.1 - across.1 * off * close_in);
                }
                path.push((at, heading));
            }
            for k in 0..settings.drift_dots {
                let k = k + w as u32 * 1000;
                // How far along (scattered, not in rows), and dropped more often the further out.
                let along = dice(seed, i, k, 1).max(0.01);
                if dice(seed, i, k, 2) < settings.drift_thin * along {
                    continue;
                }
                let place = along * steps as f32;
                let (a, b) = (path[(place as usize).min(steps)], path[(place as usize + 1).min(steps)]);
                let f = place.fract();
                let on_path = (a.0.0 + (b.0.0 - a.0.0) * f, a.0.1 + (b.0.1 - a.0.1) * f);
                // Off to one side, further the further along (less, the more coherent);
                // `drift_lean` picks the side.
                let left = (a.1.1, -a.1.0);
                let side = if dice(seed, i, k, 3) < (1.0 - settings.drift_lean) / 2.0 { 1.0 } else { -1.0 };
                let off = side * settings.drift_spread * (1.0 - coherence) * length * along * dice(seed, i, k, 4);
                let drifted = (on_path.0 + left.0 * off, on_path.1 + left.1 * off);
                // In a knot: anywhere round the seed, out to the knot scatter.
                let turn = dice(seed, i, k, 5) * std::f32::consts::TAU;
                let reach = settings.knot_scatter * (1.0 - coherence) * length * dice(seed, i, k, 6).sqrt();
                let scattered = (start.0 + turn.cos() * reach, start.1 + turn.sin() * reach);
                let at = (scattered.0 + (drifted.0 - scattered.0) * string, scattered.1 + (drifted.1 - scattered.1) * string);
                dots.push(Dot { at, along });
            }
        }
    }
    (seeds, dots)
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

    /// Flat maps `width` across, with the rock pushed `push` (doubled-angle arrow, north up).
    fn maps(width: usize, push: (f32, f32)) -> Maps {
        let cells = width * width;
        Maps {
            width,
            blocks_per_pixel: 32.0,
            coast: vec![1.0; cells],
            height: vec![0.5; cells],
            tilt: vec![0.0; cells],
            squeeze: vec![0.0; cells],
            push_x: vec![push.0; cells],
            push_y: vec![push.1; cells],
            push_strength: vec![1.0; cells],
            coast_distance: vec![1_000.0; cells],
            lift_distance: vec![32.0 * width as f32; cells],
            mountain_points: Vec::new(),
            water: 0.0,
            lowest: -1.0,
            highest: 1.0,
            most_crushed: 1.0,
            most_stretched: -1.0,
        }
    }

    /// How far apart two grain angles are (radians, 0 to a quarter turn; half a turn apart is the
    /// same direction).
    fn apart(a: f32, b: f32) -> f32 {
        let d = (a - b).rem_euclid(std::f32::consts::PI);
        d.min(std::f32::consts::PI - d)
    }

    #[test]
    fn a_straight_range_is_a_string_running_its_own_way_and_a_round_massif_is_a_knot() {
        use std::f32::consts::FRAC_PI_2;
        let width = 120;
        let middle = width as f32 / 2.0;
        // A range running north to south down the middle, the rock pushed from the north (so the
        // squeeze says the range runs east to west).
        let range: Vec<f32> = (0..width * width).map(|i| (-((i % width) as f32 - middle).powi(2) / 50.0).exp()).collect();
        let pushed_from_north = maps(width, (-1.0, 0.0));
        let i = 60 * width + 64;
        let shape = grain(&range, &pushed_from_north, 8, 0.0);
        assert!(apart(shape.angle[i], FRAC_PI_2) < 0.05, "runs at {} radians", shape.angle[i]);
        assert!(shape.string[i] > 0.9, "only {} a string", shape.string[i]);
        let squeeze = grain(&range, &pushed_from_north, 8, 1.0);
        assert!(apart(squeeze.angle[i], 0.0) < 0.05, "the squeeze says {} radians", squeeze.angle[i]);
        // A round massif: slopes face every way round its middle.
        let massif: Vec<f32> = (0..width * width)
            .map(|i| (-(((i % width) as f32 - middle).powi(2) + ((i / width) as f32 - middle).powi(2)) / 200.0).exp())
            .collect();
        let knot = grain(&massif, &pushed_from_north, 12, 0.0);
        assert!(knot.string[60 * width + 60] < 0.2, "the massif's middle is {} a string", knot.string[60 * width + 60]);
    }

    #[test]
    fn drifting_dots_follow_a_range_the_way_asked_and_scatter_in_a_knot() {
        let (width, blocks) = (120, 32.0);
        let middle = width as f32 / 2.0;
        let none = Ranges { of_pixel: vec![Ranges::NONE; width * width], list: Vec::new() };
        // A range running north to south down the middle.
        let range: Vec<f32> = (0..width * width).map(|i| (-((i % width) as f32 - middle).powi(2) / 50.0).exp()).collect();
        let found = grain(&range, &maps(width, (1.0, 0.0)), 8, 0.0);
        let settings = AdditiveSettings {
            drift_spacing: 32.0 * 6.0,
            drift_least: 0.5,
            drift_distance: 32.0 * 20.0,
            drift_spread: 0.0,
            drift_thin: 0.0,
            drift_toward: 180.0,
            drift_way: DriftWay::Bearing,
            ..AdditiveSettings::default()
        };
        let (seeds, dots) = grain_drift(&range, &found, &none, width, &settings, 7, blocks);
        assert!(seeds.len() > 5, "only {} seeds", seeds.len());
        // Away from the map's ends (where the grain gets muddled), every dot is on the range and
        // south of a seed.
        let inside = |y: f32| y > 20.0 && y < width as f32 - 20.0;
        let southmost_seed = seeds.iter().map(|s| s.1).fold(f32::MIN, f32::max);
        for dot in dots.iter().filter(|d| inside(d.at.1)) {
            assert!((dot.at.0 - middle).abs() < 4.0, "a dot strayed off the range to {:?}", dot.at);
        }
        assert!(dots.iter().any(|d| d.at.1 > southmost_seed + 5.0), "nothing drifted south");
        // Drifting north instead: the dots go the other way.
        let north = grain_drift(&range, &found, &none, width, &AdditiveSettings { drift_toward: 0.0, ..settings.clone() }, 7, blocks).1;
        let northmost_seed = seeds.iter().map(|s| s.1).fold(f32::MAX, f32::min);
        assert!(north.iter().any(|d| d.at.1 < northmost_seed - 5.0), "nothing drifted north");
        // A knot everywhere: the dots stay round their seeds, as far as the knot scatter.
        let knot = Grain { angle: found.angle.clone(), string: vec![0.0; width * width] };
        let (seeds, dots) = grain_drift(&range, &knot, &none, width, &settings, 7, blocks);
        for dot in &dots {
            let nearest = seeds.iter().map(|s| ((s.0 - dot.at.0).powi(2) + (s.1 - dot.at.1).powi(2)).sqrt()).fold(f32::MAX, f32::min);
            assert!(nearest <= 20.0 * settings.knot_scatter + 0.01, "a knot dot {nearest} from its seed");
        }
    }

    #[test]
    fn a_carrot_shaped_range_points_to_its_thin_end_and_pulls_its_dots_out_past_it() {
        let (width, blocks) = (200, 32.0);
        // A carrot lying east to west: fat at the west end (x 40), thinning to a point at x 160.
        let carrot: Vec<f32> = (0..width * width)
            .map(|i| {
                let (x, y) = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
                let half_width = if (40.0..160.0).contains(&x) { 20.0 * (160.0 - x) / 120.0 } else { 0.0 };
                if (y - 100.0).abs() < half_width { 1.0 } else { 0.0 }
            })
            .collect();
        // And a carrot the other way round, far off to the south, as a range of its own.
        let both: Vec<f32> = (0..width * width).map(|i| if i / width < 140 { carrot[i] } else { carrot[(i / width - 80) * width + (width - 1 - i % width)] }).collect();
        let ranges = find_ranges(&both, width, 0.5);
        assert_eq!(ranges.list.len(), 2);
        let east = &ranges.list[ranges.of_pixel[100 * width + 60] as usize];
        assert!(east.tip.0 > 0.95, "the first carrot points {:?}", east.tip);
        assert!(east.lopsided > 0.8, "only {} lopsided", east.lopsided);
        // Its middle is nearer the fat end.
        assert!(east.centre.0 < 100.0);
        let west = &ranges.list[ranges.of_pixel[180 * width + 140] as usize];
        assert!(west.tip.0 < -0.95, "the second carrot points {:?}", west.tip);
        // Dots drift off the first carrot's thin end, out past it.
        let found = grain(&both, &maps(width, (1.0, 0.0)), 6, 0.0);
        let settings = AdditiveSettings { drift_spacing: 32.0 * 5.0, drift_least: 0.5, drift_distance: 32.0 * 80.0, drift_thin: 0.0, drift_spread: 0.0, tip_pull: 0.5, ..AdditiveSettings::default() };
        let (_, dots) = grain_drift(&both, &found, &ranges, width, &settings, 7, blocks);
        assert!(dots.iter().any(|d| d.at.0 > 175.0 && (d.at.1 - 100.0).abs() < 15.0), "nothing went out past the first carrot's tip");
        assert!(dots.iter().any(|d| d.at.0 < 25.0 && (d.at.1 - 180.0).abs() < 15.0), "nothing went out past the second carrot's tip");
        // Fully coherent, with lots of spread: past the first carrot's tip, the dots are all in one
        // tight stream along its middle line.
        let coherent = AdditiveSettings { coherence: 1.0, drift_spread: 1.0, knot_scatter: 1.0, ..settings };
        let (_, dots) = grain_drift(&both, &found, &ranges, width, &coherent, 7, blocks);
        let past: Vec<&Dot> = dots.iter().filter(|d| d.at.0 > 170.0).collect();
        assert!(past.len() > 5, "only {} dots past the tip", past.len());
        for dot in past {
            assert!((dot.at.1 - east.centre.1).abs() < 3.0, "a dot {} off the stream", dot.at.1 - east.centre.1);
        }
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
        let settings = AdditiveSettings { spacing: 32.0 * 8.0, distance: 32.0 * 20.0, dots: 10, bend: 0.0, sides: Sides::Inside, least_mountains: 0.1, ..AdditiveSettings::default() };
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

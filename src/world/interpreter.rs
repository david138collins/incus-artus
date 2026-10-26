//! The World Interpreter — the end of the world-making chain: the recipe that gives every column of
//! blocks its ground height, worked out **at that block**, as the world is made.
//!
//! ```text
//! Tectonic Sim ──► bake (32-block pixels: height, squeeze, tilt…)
//!     │
//! Terrain Compiler — whole-map jobs, done once:
//!     sea level · soften the coastline · foothill skirts
//!     │
//!     ▼  smooth maps (coast, height, tilt, squeeze and its direction, skirt)
//! World Interpreter — at every block, as chunks are made (this file):
//!     blend the maps smoothly to this spot, then:
//!     land & sea curves · grading · mountains · ridges · passes · rifts · hills · detail
//!     │
//!     ▼
//! the ground you walk on, and the distant terrain (the same recipe, sampled coarsely)
//! ```
//!
//! **Why the recipe is here and not in the compiler** [found 2026-10-24]: the compiler used to work
//! the recipe out once per 32-block pixel and hand over finished heights, which were then smoothed
//! between pixels. But the recipe has sharp curves in it (a coast can drop 90 blocks within one
//! pixel; mountain curves can be steep), and smoothing between finished pixel heights swings past
//! every sharp step: one hump per pixel, in rows along every coast and cliff. So now the smooth
//! *inputs* are blended to each block first, and the sharp curves applied after, at the block.
//!
//! **Blending between pixels:** a Catmull–Rom curve through the 4 × 4 pixels round the spot (smooth:
//! no creases at pixel edges), held within the four nearest pixels' values so it can never swing
//! past them.
//!
//! **The recipe** at each spot:
//! 1. **Land or sea** from the *coast* map (the bake's height near sea level, softened by the
//!    compiler so the coastline is a curve, not a staircase of pixels).
//! 2. **Land and sea curves:** land climbs from the coast (Z 128) to the **highlands**, the sea floor
//!    falls to the **deepest**, along curves through dots (`Spline`).
//! 3. **Grade:** each plate's tilt steepens its high side and softens its low side.
//! 4. **Mountains** from the squeeze: the **mountain curve** says how tall for how much squeeze, as a
//!    share of the room up to Z 333; carved into **ridges** across the squeeze direction, whose
//!    crests dip into **passes**; never below the compiler's **foothill skirt**.
//! 5. **Rifts:** stretched rock sinks.
//! 6. **Hills** on land, and a little **detail** everywhere: each is a Perlin field, the same as a
//!    Graphics Kit Perlin layer, laid on the ground this many blocks up and down, and faded in and
//!    out by a **mask** that follows one of the maps (distance from the coast, height, mountains,
//!    foothills or squeeze) through a curve.

use super::terrain::SEA_LEVEL;
use crate::worldgen::noise::{Fade, PerlinSettings, perlin};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// No ground goes higher than this [David].
pub const HIGHEST: f32 = 333.0;

// ── Curves ──────────────────────────────────────────────────────────────────

/// A curve through dots, from 0 to 1 across. The first dot is always at the left edge and the last
/// at the right; the line flows smoothly through all of them, without overshooting (a flat stretch
/// stays flat, and it never bulges past a dot).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Spline {
    /// (across, up), left to right.
    pub dots: Vec<(f32, f32)>,
}

impl Default for Spline {
    fn default() -> Self {
        Self { dots: vec![(0.0, 0.0), (1.0, 1.0)] }
    }
}

impl Spline {
    pub fn through(dots: &[(f32, f32)]) -> Self {
        Self { dots: dots.to_vec() }
    }

    /// The curve's height at `x` (0 to 1 across).
    pub fn at(&self, x: f32) -> f32 {
        self.curve().at(x)
    }

    /// The curve ready to be read many times (its steepness at each dot worked out once).
    pub fn curve(&self) -> Curve {
        Curve { dots: self.dots.clone(), slopes: self.slopes() }
    }

    /// How steep the curve is at each dot: the average of the slopes either side, flattened where
    /// the line turns (so it never overshoots), and kept gentle enough not to bulge (Fritsch and
    /// Carlson's rule).
    fn slopes(&self) -> Vec<f32> {
        let dots = &self.dots;
        let n = dots.len();
        if n < 2 {
            return vec![0.0; n];
        }
        let between: Vec<f32> = dots.windows(2).map(|p| (p[1].1 - p[0].1) / (p[1].0 - p[0].0).max(1e-6)).collect();
        let mut slopes = vec![0.0; n];
        slopes[0] = between[0];
        slopes[n - 1] = between[n - 2];
        for k in 1..n - 1 {
            slopes[k] = if between[k - 1] * between[k] <= 0.0 { 0.0 } else { (between[k - 1] + between[k]) / 2.0 };
        }
        for k in 0..n - 1 {
            if between[k].abs() < 1e-9 {
                (slopes[k], slopes[k + 1]) = (0.0, 0.0);
                continue;
            }
            let (a, b) = (slopes[k] / between[k], slopes[k + 1] / between[k]);
            let size = a * a + b * b;
            if size > 9.0 {
                let shrink = 3.0 / size.sqrt();
                (slopes[k], slopes[k + 1]) = (shrink * a * between[k], shrink * b * between[k]);
            }
        }
        slopes
    }
}

/// A `Spline` ready to read.
#[derive(Clone, Debug, Default)]
pub struct Curve {
    dots: Vec<(f32, f32)>,
    slopes: Vec<f32>,
}

impl Curve {
    pub fn at(&self, x: f32) -> f32 {
        let dots = &self.dots;
        match dots.len() {
            0 => return 0.0,
            1 => return dots[0].1,
            _ => {}
        }
        let x = x.clamp(dots[0].0, dots[dots.len() - 1].0);
        let k = dots.windows(2).position(|pair| x <= pair[1].0).unwrap_or(dots.len() - 2);
        let ((x0, y0), (x1, y1)) = (dots[k], dots[k + 1]);
        let h = (x1 - x0).max(1e-6);
        let t = (x - x0) / h;
        // Hermite: the two dots and the curve's steepness at each.
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * y0 + (t3 - 2.0 * t2 + t) * h * self.slopes[k] + (-2.0 * t3 + 3.0 * t2) * y1 + (t3 - t2) * h * self.slopes[k + 1]
    }
}

// ── The recipe's settings ───────────────────────────────────────────────────

/// The Interpreter's recipe: how the compiler's maps become the ground, and which blocks cover it.
/// Set on the Interpreter panel, next to the Terrain Compiler; copied into each world.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InterpreterSettings {
    /// The Z of the deepest sea floor, and of the highest land (before mountains).
    pub deepest: f32,
    pub highlands: f32,
    /// How land climbs from the coast (left) to the bake's highest ground (right): up = a share of
    /// the way from Z 128 to the highlands.
    pub land: Spline,
    /// How the sea floor falls from the coast (left) to the bake's deepest point (right): up = a
    /// share of the way from just below Z 128 down to the deepest.
    pub sea: Spline,
    /// How strongly each plate's tilt grades it (0 = no grading).
    pub grade: f32,
    /// The lift: land raised toward the mountains, like a tablecloth draped over points floating
    /// above them, its edges tapering off just before the coast.
    pub lift: Lift,
    /// How tall mountains are for each amount of squeeze, from none (left) to the most crushed rock
    /// in the bake (right): up = a share of the room up to Z 333.
    pub mountains: Spline,
    /// How much the ridge pattern carves into the mountains: 0 = smooth domes, 1 = all ridges; past
    /// 1, the valleys between crests reach right down to the ground over wider stretches.
    pub ridges: f32,
    /// How many layers of ridges are stacked (each finer than the last), and how strong each finer
    /// layer is next to the one before (higher = rougher, more broken crests).
    pub ridge_layers: u32,
    pub ridge_roughness: f32,
    /// How readily ridges line up across the squeeze direction: 0 = wandering every way, 1 = lined up
    /// wherever the squeeze is clear, 2 = lined up even where it's faint.
    pub ridge_alignment: f32,
    /// Steady ridges: laid out in fixed directions, so their spacing holds where the squeeze bends
    /// (off = the old way, which crushes them into grooves there).
    pub steady_ridges: bool,
    /// How far (blocks) ridges are nudged sideways by a slow noise, and how big its swirls are.
    pub ridge_wander: f32,
    pub ridge_wander_size: f32,
    /// How far apart ridges are (across them), and how long they run (along them), in blocks.
    pub ridge_spacing: f32,
    pub ridge_length: f32,
    /// Higher = narrower, sharper ridge crests.
    pub sharpness: f32,
    /// How deeply the crest height dips into passes along each ridge (0 = even crests; past 1 the
    /// deepest passes cut right through to the ground, leaving separate peaks), and how far apart the
    /// dips are, in blocks.
    pub passes: f32,
    pub pass_spacing: f32,
    /// How deep the most stretched rock sinks, in blocks.
    pub rift_depth: f32,
    /// Rolling hills on land: how high (blocks either way), the Perlin field that shapes them, and
    /// where they show.
    pub hills: f32,
    pub hill_noise: PerlinSettings,
    pub hill_mask: Mask,
    /// Small bumps on the ground: how high (blocks either way), their Perlin field, and where they
    /// show.
    pub detail: f32,
    pub detail_noise: PerlinSettings,
    pub detail_mask: Mask,
    /// The surface is bare stone from this Z up, and gabbro (the peaks) from this Z up.
    pub stone_from: i32,
    pub gabbro_from: i32,
}

impl Default for InterpreterSettings {
    fn default() -> Self {
        Self {
            deepest: 30.0,
            highlands: 200.0,
            land: Spline::through(&[(0.0, 0.0), (0.4, 0.2), (0.75, 0.6), (1.0, 1.0)]),
            sea: Spline::through(&[(0.0, 0.0), (0.4, 0.25), (1.0, 1.0)]),
            grade: 0.4,
            lift: Lift::default(),
            mountains: Spline::through(&[(0.0, 0.0), (0.08, 0.0), (0.5, 0.35), (1.0, 0.9)]),
            ridges: 0.5,
            ridge_layers: 3,
            ridge_roughness: 0.45,
            ridge_alignment: 1.0,
            steady_ridges: true,
            ridge_wander: 0.0,
            ridge_wander_size: 2_000.0,
            ridge_spacing: 3_000.0,
            ridge_length: 15_000.0,
            sharpness: 1.2,
            passes: 0.4,
            pass_spacing: 12_000.0,
            rift_depth: 30.0,
            hills: 6.0,
            // Two octaves, the second 2.3 times finer and 0.43 as strong (as hills always were).
            hill_noise: PerlinSettings { scale: 1_500.0, octaves: 2, persistence: 0.43, lacunarity: 2.3, ..PerlinSettings::default() },
            hill_mask: Mask::default(),
            detail: 3.0,
            detail_noise: PerlinSettings { scale: 96.0, octaves: 1, ..PerlinSettings::default() },
            detail_mask: Mask::default(),
            stone_from: 215,
            gabbro_from: 285,
        }
    }
}

/// Where the lift's points are: scattered over the mountains, or one in the middle of each range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiftFrom {
    #[default]
    Points,
    Ranges,
}

/// The lift's settings. The points are found from the bake's own crush (how squeezed the rock is,
/// against the most crushed in the bake, 0 to 1), not the mountains curve, so they hold still as
/// the mountains are reshaped.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lift {
    /// How high (blocks) the land is raised right under a point, at the curve's top (0 = no lift).
    pub height: f32,
    /// How far (blocks) the cloth spreads from its points before it's down to the curve's left end.
    pub reach: f32,
    /// How the cloth falls away from its points: across, from `reach` away (left) to right under a
    /// point (right); up, the share of `height`.
    pub curve: Spline,
    /// The coast pad: within this many blocks of the coast, the lift is braked down to nothing at
    /// the coastline (so the coast never moves); `pad_falloff` is how hard (1 = evenly, above 1 =
    /// braking from further in, below 1 = only close to the coast).
    pub pad: f32,
    pub pad_falloff: f32,
    pub from: LiftFrom,
    /// Points only where the crush is between these (0 to 1); ranges, only those whose most crushed
    /// spot is.
    pub least: f32,
    pub most: f32,
    /// About how many blocks apart the points are, where the rock is most crushed.
    pub spacing: f32,
    /// Ranges: cut apart where the crush is weaker than this, and only those at least this many
    /// blocks across.
    pub range_cut: f32,
    pub smallest_range: f32,
}

impl Default for Lift {
    fn default() -> Self {
        Self {
            height: 0.0,
            reach: 8_000.0,
            curve: Spline::default(),
            pad: 1_500.0,
            pad_falloff: 1.0,
            from: LiftFrom::Points,
            least: 0.25,
            most: 1.0,
            spacing: 3_000.0,
            range_cut: 0.2,
            smallest_range: 2_000.0,
        }
    }
}

/// Which map a mask follows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskSource {
    /// No mask: the same everywhere.
    #[default]
    Everywhere,
    /// Distance from the coast: far out to sea (left), the coastline (middle), far inland (right).
    Coast,
    /// The ground's height: Z 0 (left) to Z 333 (right).
    Height,
    /// How much of the room above the ground mountains take: none (left) to all of it (right).
    Mountains,
    /// The foothill skirt: none (left) to the highest skirt on the map (right).
    Foothills,
    /// The squeeze: most stretched (left), neither (middle), most crushed (right).
    Squeeze,
}

impl MaskSource {
    pub const ALL: [MaskSource; 6] = [MaskSource::Everywhere, MaskSource::Coast, MaskSource::Height, MaskSource::Mountains, MaskSource::Foothills, MaskSource::Squeeze];

    pub fn name(self) -> &'static str {
        match self {
            MaskSource::Everywhere => "everywhere",
            MaskSource::Coast => "coast distance",
            MaskSource::Height => "height",
            MaskSource::Mountains => "mountains",
            MaskSource::Foothills => "foothills",
            MaskSource::Squeeze => "squeeze",
        }
    }

    /// What the curve's left and right edges stand for.
    pub fn ends(self) -> (&'static str, &'static str) {
        match self {
            MaskSource::Everywhere => ("", ""),
            MaskSource::Coast => ("out to sea · coast · inland", "inland"),
            MaskSource::Height => ("Z 0", "Z 333"),
            MaskSource::Mountains => ("no mountains", "tallest"),
            MaskSource::Foothills => ("no skirt", "highest skirt"),
            MaskSource::Squeeze => ("stretched · neither · crushed", "crushed"),
        }
    }
}

/// Fades a layer (hills, detail) in and out across the world: it follows one of the maps, read
/// through a curve (up = how much of the layer shows, 0 to 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mask {
    pub source: MaskSource,
    /// Read the map the other way round (left becomes right).
    pub invert: bool,
    pub curve: Spline,
    /// For coast distance: how many blocks from the coastline the curve's edges stand for.
    pub reach: f32,
}

impl Default for Mask {
    fn default() -> Self {
        Self { source: MaskSource::Everywhere, invert: false, curve: Spline::default(), reach: 4_000.0 }
    }
}

// ── The compiler's maps ─────────────────────────────────────────────────────

/// What the Terrain Compiler hands over: smooth maps, one value per pixel, and the bake's ranges the
/// recipe measures against.
#[derive(Clone, Debug, Default)]
pub struct Maps {
    /// Pixels across (the maps are square), and blocks per pixel.
    pub width: usize,
    pub blocks_per_pixel: f32,
    /// Land (+) or sea (−): the bake's height near sea level, softened so the coast is a curve.
    pub coast: Vec<f32>,
    /// The bake's height, softened as much as the coast.
    pub height: Vec<f32>,
    /// How far up (+) or down (−) its plate's tilt each spot is (softened the same).
    pub tilt: Vec<f32>,
    /// How crushed (+) or stretched (−) the rock is.
    pub squeeze: Vec<f32>,
    /// Which way it was squeezed, as a doubled-angle arrow (so directions half a turn apart, which
    /// are the same squeeze, blend instead of cancelling), and how hard.
    pub push_x: Vec<f32>,
    pub push_y: Vec<f32>,
    pub push_strength: Vec<f32>,
    /// The foothill skirt: mountains are never lower than this (blocks above the ground).
    pub skirt: Vec<f32>,
    /// Blocks to the coastline: + on land, − at sea.
    pub coast_distance: Vec<f32>,
    /// Blocks to the nearest of the lift's points (the map's width in blocks where there are none).
    pub lift_distance: Vec<f32>,
    /// The highest foothill skirt on the map (blocks).
    pub highest_skirt: f32,
    /// The bake's height at sea level, its lowest and highest heights, and its most crushed and
    /// most stretched rock.
    pub water: f32,
    pub lowest: f32,
    pub highest: f32,
    pub most_crushed: f32,
    pub most_stretched: f32,
}

/// Blocks to the coastline from every pixel of a coast map (+ on land, − at sea), measured pixel to
/// pixel (straight steps 1, diagonal steps √2: within a few percent of the true distance).
pub fn coast_distance(coast: &[f32], width: usize, blocks_per_pixel: f32) -> Vec<f32> {
    let w = width;
    let land = |i: usize| coast[i] >= 0.0;
    let mut d = vec![f32::MAX; w * w];
    // The coast itself: pixels with the other side next door, half a pixel from the line.
    for row in 0..w {
        for col in 0..w {
            let i = row * w + col;
            let other = (col + 1 < w && land(i + 1) != land(i)) || (col > 0 && land(i - 1) != land(i)) || (row + 1 < w && land(i + w) != land(i)) || (row > 0 && land(i - w) != land(i));
            if other {
                d[i] = 0.5;
            }
        }
    }
    let diagonal = std::f32::consts::SQRT_2;
    // Down and across, then back up.
    for row in 0..w {
        for col in 0..w {
            let i = row * w + col;
            let mut best = d[i];
            if col > 0 {
                best = best.min(d[i - 1] + 1.0);
            }
            if row > 0 {
                best = best.min(d[i - w] + 1.0);
                if col > 0 {
                    best = best.min(d[i - w - 1] + diagonal);
                }
                if col + 1 < w {
                    best = best.min(d[i - w + 1] + diagonal);
                }
            }
            d[i] = best;
        }
    }
    for row in (0..w).rev() {
        for col in (0..w).rev() {
            let i = row * w + col;
            let mut best = d[i];
            if col + 1 < w {
                best = best.min(d[i + 1] + 1.0);
            }
            if row + 1 < w {
                best = best.min(d[i + w] + 1.0);
                if col + 1 < w {
                    best = best.min(d[i + w + 1] + diagonal);
                }
                if col > 0 {
                    best = best.min(d[i + w - 1] + diagonal);
                }
            }
            d[i] = best;
        }
    }
    let far = w as f32;
    (0..w * w).map(|i| if land(i) { 1.0 } else { -1.0 } * d[i].min(far) * blocks_per_pixel).collect()
}

/// Blocks from every pixel's centre to the nearest of `points` (pixel indices), or the map's width
/// in blocks if there are none. Each pixel passes on which point is nearest to it, and the distance
/// is measured straight to that point, so it's round, not the octagons step counting gives.
pub fn point_distance(points: &[usize], width: usize, blocks_per_pixel: f32) -> Vec<f32> {
    let w = width;
    let far = w as f32 * blocks_per_pixel;
    if points.is_empty() {
        return vec![far; w * w];
    }
    let mut nearest = vec![usize::MAX; w * w];
    for &p in points {
        nearest[p] = p;
    }
    let apart = |i: usize, p: usize| {
        let (dx, dy) = ((i % w) as f32 - (p % w) as f32, (i / w) as f32 - (p / w) as f32);
        dx * dx + dy * dy
    };
    let consider = |nearest: &mut [usize], i: usize, n: usize| {
        let p = nearest[n];
        if p != usize::MAX && (nearest[i] == usize::MAX || apart(i, p) < apart(i, nearest[i])) {
            nearest[i] = p;
        }
    };
    // Down and across, then back up (twice, to settle the few the first round misses).
    for _ in 0..2 {
        for row in 0..w {
            for col in 0..w {
                let i = row * w + col;
                if col > 0 {
                    consider(&mut nearest, i, i - 1);
                }
                if row > 0 {
                    consider(&mut nearest, i, i - w);
                    if col > 0 {
                        consider(&mut nearest, i, i - w - 1);
                    }
                    if col + 1 < w {
                        consider(&mut nearest, i, i - w + 1);
                    }
                }
            }
        }
        for row in (0..w).rev() {
            for col in (0..w).rev() {
                let i = row * w + col;
                if col + 1 < w {
                    consider(&mut nearest, i, i + 1);
                }
                if row + 1 < w {
                    consider(&mut nearest, i, i + w);
                    if col + 1 < w {
                        consider(&mut nearest, i, i + w + 1);
                    }
                    if col > 0 {
                        consider(&mut nearest, i, i + w - 1);
                    }
                }
            }
        }
    }
    (0..w * w).map(|i| (apart(i, nearest[i]).sqrt() * blocks_per_pixel).min(far)).collect()
}

/// The maps' values at one spot.
#[derive(Clone, Copy, Debug, Default)]
pub struct Here {
    pub coast: f32,
    pub height: f32,
    pub tilt: f32,
    pub squeeze: f32,
    pub push_x: f32,
    pub push_y: f32,
    pub push_strength: f32,
    pub skirt: f32,
    pub coast_distance: f32,
    pub lift_distance: f32,
}

/// The recipe's pieces at one spot, before they're put together.
#[derive(Clone, Copy, Debug)]
pub struct Parts {
    pub ground: f32,
    pub mountains: f32,
    pub rift: f32,
}

/// Parts of the recipe the terrain lab can leave out, to see what each one does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mutes {
    pub lift: bool,
    pub mountains: bool,
    pub ridges: bool,
    pub passes: bool,
    pub rifts: bool,
    pub hills: bool,
    pub detail: bool,
}

/// A compiled world: the compiler's maps, and the recipe that reads them.
#[derive(Clone, Debug)]
pub struct CompiledMap {
    /// Shared: a new recipe over the same maps (as the Interpreter panel's sliders move) costs nothing.
    pub maps: Arc<Maps>,
    /// The bake's seed (the detail noise uses it; the recipe's noise a seed made from it).
    pub seed: u64,
    pub settings: InterpreterSettings,
    /// Parts of the recipe left out (the terrain lab's layer switches; a world never leaves any out).
    pub mutes: Mutes,
    noise_seed: u64,
    land: Curve,
    sea: Curve,
    mountains: Curve,
    lift: Curve,
    hill_mask: Curve,
    detail_mask: Curve,
}

/// A Perlin field (as a Graphics Kit layer makes it, output shaping and all) at a spot, about −1
/// to 1. (Filling black to white needs the whole field at once, so it has no say here.)
fn noise_at(noise: &PerlinSettings, seed: u64, north: f32, east: f32) -> f32 {
    noise.output.shape(noise.raw(seed, north, east) * 0.5 + 0.5) * 2.0 - 1.0
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Catmull–Rom: the smooth curve through four evenly spaced values, at `t` (0 to 1) between the
/// middle two.
fn catmull_rom(p: [f32; 4], t: f32) -> f32 {
    let (t2, t3) = (t * t, t * t * t);
    0.5 * (2.0 * p[1] + (p[2] - p[0]) * t + (2.0 * p[0] - 5.0 * p[1] + 4.0 * p[2] - p[3]) * t2 + (3.0 * (p[1] - p[2]) + p[3] - p[0]) * t3)
}

/// How many fixed directions steady ridges are laid out in (across half a turn: every 15°).
const RIDGE_DIRECTIONS: usize = 12;

/// A ridge pattern, 0 to 1 (1 = a crest): `layers` of "folded" noise stacked, each 2.1 times finer
/// and `roughness` times as strong as the last, whose creases are the crests.
fn ridges(seed: u64, u: f32, v: f32, sharpness: f32, layers: u32, roughness: f32) -> f32 {
    let (mut total, mut strength, mut strengths, mut scale) = (0.0, 1.0, 0.0, 1.0);
    for octave in 0..layers.max(1) as u64 {
        let crest = 1.0 - perlin(seed, 0xa11 + octave, Fade::Smooth, u * scale, v * scale).abs().min(1.0);
        total += crest.powf(sharpness.max(0.1)) * strength;
        strengths += strength;
        strength *= roughness;
        scale *= 2.1;
    }
    total / strengths
}

impl CompiledMap {
    /// The same maps read with another recipe, leaving out `mutes`: no copying, so the terrain lab
    /// can follow a slider as it moves.
    pub fn with(&self, settings: InterpreterSettings, mutes: Mutes) -> Self {
        Self { mutes, ..Self::new(self.maps.clone(), self.seed, settings) }
    }

    pub fn new(maps: Arc<Maps>, seed: u64, settings: InterpreterSettings) -> Self {
        Self {
            noise_seed: crate::fate::derive_seed(seed, "world compiler"),
            land: settings.land.curve(),
            sea: settings.sea.curve(),
            mountains: settings.mountains.curve(),
            lift: settings.lift.curve.curve(),
            hill_mask: settings.hill_mask.curve.curve(),
            detail_mask: settings.detail_mask.curve.curve(),
            mutes: Mutes::default(),
            maps,
            seed,
            settings,
        }
    }

    /// The maps' values at pixel `i`, exactly.
    pub fn pixel(&self, i: usize) -> Here {
        let m = &self.maps;
        Here {
            coast: m.coast[i],
            height: m.height[i],
            tilt: m.tilt[i],
            squeeze: m.squeeze[i],
            push_x: m.push_x[i],
            push_y: m.push_y[i],
            push_strength: m.push_strength[i],
            skirt: m.skirt[i],
            coast_distance: m.coast_distance[i],
            lift_distance: m.lift_distance[i],
        }
    }

    /// The maps' values blended to a spot between pixels (`col`, `row` count pixel centres).
    fn blended(&self, col: f32, row: f32) -> Here {
        let (c0, r0) = (col.floor() as i64, row.floor() as i64);
        let (tc, tr) = (col - c0 as f32, row - r0 as f32);
        let w = self.maps.width as i64;
        let index = |c: i64, r: i64| (r.clamp(0, w - 1) * w + c.clamp(0, w - 1)) as usize;
        let blend = |field: &[f32]| {
            let mut rows = [0.0; 4];
            for (k, row) in rows.iter_mut().enumerate() {
                let r = r0 - 1 + k as i64;
                *row = catmull_rom([field[index(c0 - 1, r)], field[index(c0, r)], field[index(c0 + 1, r)], field[index(c0 + 2, r)]], tc);
            }
            // Held within the four nearest pixels' values: the curve never swings past them.
            let near = [field[index(c0, r0)], field[index(c0 + 1, r0)], field[index(c0, r0 + 1)], field[index(c0 + 1, r0 + 1)]];
            let (low, high) = near.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            catmull_rom(rows, tr).clamp(low, high)
        };
        let m = &self.maps;
        Here {
            coast: blend(&m.coast),
            height: blend(&m.height),
            tilt: blend(&m.tilt),
            squeeze: blend(&m.squeeze),
            push_x: blend(&m.push_x),
            push_y: blend(&m.push_y),
            push_strength: blend(&m.push_strength),
            skirt: blend(&m.skirt),
            coast_distance: blend(&m.coast_distance),
            lift_distance: blend(&m.lift_distance),
        }
    }

    /// How much of the room above the ground mountains take at a spot, 0 to 1, before ridges and
    /// passes carve them (the mountains curve read at its squeeze; 0 where the rock isn't crushed).
    pub fn mountain_share(&self, here: &Here) -> f32 {
        if here.squeeze <= 0.0 {
            return 0.0;
        }
        self.mountains.at(here.squeeze / self.maps.most_crushed.max(1e-6)).clamp(0.0, 1.0)
    }

    /// The recipe's pieces at a spot with these map values; `x` east and `y` south are blocks from
    /// the map's north-west corner (where the noise patterns are laid).
    pub fn parts(&self, here: &Here, x: f32, y: f32) -> Parts {
        let (s, m) = (&self.settings, &self.maps);
        let sea = SEA_LEVEL as f32;

        // 1 to 3. Land or sea, the curves, and grading by the plate's tilt.
        let steep = (1.0 + s.grade * here.tilt).max(0.15);
        let ground = if here.coast >= 0.0 {
            let t = (here.height - m.water).max(0.0) / (m.highest - m.water).max(1e-6);
            sea + (s.highlands - sea).max(0.0) * self.land.at(t).clamp(0.0, 1.0) * steep
        } else {
            let t = (m.water - here.height).max(0.0) / (m.water - m.lowest).max(1e-6);
            (sea - 1.0) - (sea - 1.0 - s.deepest).max(0.0) * self.sea.at(t).clamp(0.0, 1.0) * steep
        };
        let ground = ground + self.lift(here);

        // 4. Mountains.
        let mut mountains = 0.0;
        let tall = self.mountain_share(here);
        if tall > 0.0 && !self.mutes.mountains {
            // The push's direction on the map (north is −y), from the doubled-angle arrow, and the
            // ridges lined up across it, blended into wandering ridges where the direction is
            // unclear.
            let angle = here.push_y.atan2(here.push_x) / 2.0;
            let spacing = s.ridge_spacing.max(1.0);
            let seed = self.noise_seed;
            let (layers, roughness) = (s.ridge_layers.clamp(1, 8), s.ridge_roughness.clamp(0.0, 1.0));
            let pass_size = s.pass_spacing.max(1.0);
            // Wander: the ridges' spot nudged sideways by a slow noise, so they bend and their
            // bases shift instead of running dead straight side by side.
            let (x, y) = if s.ridge_wander > 0.0 {
                let size = s.ridge_wander_size.max(1.0);
                let nudge = |k| perlin(seed, k, Fade::Smooth, x / size, y / size) * s.ridge_wander;
                (x + nudge(0x3a1), y + nudge(0x3a2))
            } else {
                (x, y)
            };
            // The ridges (and the passes along them) lined up across one direction.
            let lined = |direction: f32, k: u64| {
                let push = (direction.cos(), -direction.sin());
                let across = x * push.0 + y * push.1;
                let along = -x * push.1 + y * push.0;
                let crest = ridges(seed ^ k.wrapping_mul(0x9e37_79b9), across / spacing, along / s.ridge_length.max(1.0), s.sharpness, layers, roughness);
                (crest, perlin(seed, 0x9a55 + k, Fade::Smooth, along / pass_size, across / (pass_size * 3.0)))
            };
            let (lined_up, along_ridge) = if s.steady_ridges {
                // Steady: each of a few fixed directions has its own pattern laid across the whole
                // map, and a spot blends the two nearest its push. Turning the pattern to follow
                // the push itself (the old way, below) swings it round the map's corner, which
                // crushes ridges together wherever the push bends, worse the further out: rows of
                // grooves a few blocks wide (found 2026-10-25, David's "giant fingerprint").
                let step = std::f32::consts::PI / RIDGE_DIRECTIONS as f32;
                let k = (angle + std::f32::consts::FRAC_PI_2) / step;
                let first = k.floor();
                let t = smooth(k - first);
                let [a, b] = [first as usize % RIDGE_DIRECTIONS, (first as usize + 1) % RIDGE_DIRECTIONS];
                let toward = |i: usize| lined(-std::f32::consts::FRAC_PI_2 + i as f32 * step, i as u64 + 1);
                let ((crest_a, pass_a), (crest_b, pass_b)) = (toward(a), toward(b));
                (crest_a + (crest_b - crest_a) * t, pass_a + (pass_b - pass_a) * t)
            } else {
                lined(angle, 0)
            };
            let wandering = ridges(seed ^ 0x5eed, x / spacing, y / spacing, s.sharpness, layers, roughness);
            let clear = smooth(here.push_strength * s.ridge_alignment.max(0.0) / 0.1);
            let crest = wandering + (lined_up - wandering) * clear;
            let carved = if self.mutes.ridges { 1.0 } else { (1.0 - s.ridges.max(0.0) * (1.0 - crest)).max(0.0) };
            // Passes: the crest height rises and dips along the ridge.
            let wandering_passes = perlin(seed, 0x9a56, Fade::Smooth, x / pass_size, y / pass_size);
            let wave = wandering_passes + (along_ridge - wandering_passes) * clear;
            let dipped = if self.mutes.passes { 1.0 } else { (1.0 - s.passes.max(0.0) * (0.5 - 0.5 * wave)).max(0.0) };
            mountains = (HIGHEST - ground).max(0.0) * tall * carved * dipped;
        }
        let mountains = if self.mutes.mountains { 0.0 } else { mountains.max(here.skirt) };

        // 5. Rifts.
        let rift = if here.squeeze < 0.0 && !self.mutes.rifts { s.rift_depth * smooth(here.squeeze / m.most_stretched.min(-1e-6)) } else { 0.0 };
        Parts { ground, mountains, rift }
    }

    /// How much the lift raises the land at a spot (blocks; none at sea): the curve read at how near
    /// the nearest lift point is, braked by the coast pad down to nothing at the coastline.
    pub fn lift(&self, here: &Here) -> f32 {
        let lift = &self.settings.lift;
        if lift.height <= 0.0 || here.coast < 0.0 || self.mutes.lift {
            return 0.0;
        }
        let near = 1.0 - (here.lift_distance / lift.reach.max(1.0)).clamp(0.0, 1.0);
        let pad = smooth(here.coast_distance / lift.pad.max(1.0)).powf(lift.pad_falloff.max(0.01));
        lift.height * self.lift.at(near).clamp(0.0, 1.0) * pad
    }

    /// The pieces put together, with hills on land (6), before detail.
    pub fn put_together(&self, parts: Parts, here: &Here, x: f32, y: f32) -> f32 {
        let s = &self.settings;
        let sea = SEA_LEVEL as f32;
        let mut z = (parts.ground + parts.mountains).min(HIGHEST) - parts.rift;
        if z >= sea && !self.mutes.hills && s.hills != 0.0 {
            let shown = self.mask(&s.hill_mask, &self.hill_mask, here, z);
            if shown > 0.0 {
                let rolling = noise_at(&s.hill_noise, self.noise_seed ^ 0x4111, -y, x);
                // Never drowns land.
                z = (z + rolling * s.hills * shown).max(sea);
            }
        }
        z.clamp(1.0, HIGHEST)
    }

    /// How much of a masked layer shows at a spot (0 to 1), with the ground at `z` there.
    fn mask(&self, mask: &Mask, curve: &Curve, here: &Here, z: f32) -> f32 {
        let m = &self.maps;
        let t = match mask.source {
            MaskSource::Everywhere => return 1.0,
            MaskSource::Coast => 0.5 + 0.5 * (here.coast_distance / mask.reach.max(1.0)).clamp(-1.0, 1.0),
            MaskSource::Height => z / HIGHEST,
            MaskSource::Mountains => self.mountain_share(here),
            MaskSource::Foothills => here.skirt / m.highest_skirt.max(1e-6),
            MaskSource::Squeeze if here.squeeze >= 0.0 => 0.5 + 0.5 * here.squeeze / m.most_crushed.max(1e-6),
            MaskSource::Squeeze => 0.5 - 0.5 * here.squeeze / m.most_stretched.min(-1e-6),
        };
        let t = t.clamp(0.0, 1.0);
        curve.at(if mask.invert { 1.0 - t } else { t }).clamp(0.0, 1.0)
    }

    /// The ground's height at pixel `i`'s centre, without the block-sized detail: what the World
    /// Compiler shows.
    pub fn at_pixel(&self, i: usize) -> f32 {
        let (x, y) = self.pixel_spot(i);
        let here = self.pixel(i);
        self.put_together(self.parts(&here, x, y), &here, x, y)
    }

    /// Pixel `i`'s centre, in blocks east and south of the map's north-west corner.
    pub fn pixel_spot(&self, i: usize) -> (f32, f32) {
        let (w, b) = (self.maps.width, self.maps.blocks_per_pixel);
        (((i % w) as f32 + 0.5) * b, ((i / w) as f32 + 0.5) * b)
    }

    /// The ground's height (a Z, with fractions) at a spot `north` and `east` blocks from the
    /// middle of the world, or `None` past the edge of the map.
    pub fn height(&self, north: f32, east: f32) -> Option<f32> {
        let (width, blocks) = (self.maps.width as f32, self.maps.blocks_per_pixel);
        let half = width / 2.0;
        // Which pixel, counting pixel centres (columns run east, rows south).
        let (col, row) = (east / blocks + half - 0.5, half - north / blocks - 0.5);
        let last = width - 0.5;
        if col < -0.5 || row < -0.5 || col > last || row > last {
            return None;
        }
        let (x, y) = (east + half * blocks, half * blocks - north);
        let here = self.blended(col, row);
        let ground = self.put_together(self.parts(&here, x, y), &here, x, y);
        let s = &self.settings;
        if self.mutes.detail || s.detail == 0.0 {
            return Some(ground);
        }
        let shown = self.mask(&s.detail_mask, &self.detail_mask, &here, ground);
        let detail = if shown > 0.0 { noise_at(&s.detail_noise, self.seed ^ 0xde7a11, north, east) * s.detail * shown } else { 0.0 };
        Some(ground + detail)
    }

    /// A map whose ground is simply the given Z at each pixel (with plain straight curves, and no
    /// mountains, hills or detail): for tests.
    #[cfg(test)]
    pub fn of_heights(width: usize, blocks_per_pixel: f32, z: &[f32]) -> Self {
        let sea = SEA_LEVEL as f32;
        // Land: Z 128 + 205 × height (height 0 to 1). Sea: Z 127 − 126 × depth (height 0 to −1).
        let height: Vec<f32> = z.iter().map(|&z| if z >= sea { (z - sea) / (HIGHEST - sea) } else { -((sea - 1.0) - z) / (sea - 2.0) }).collect();
        let coast: Vec<f32> = height.iter().map(|&h| if h >= 0.0 { h.max(1e-6) } else { h }).collect();
        let cells = width * width;
        let coast_distance = coast_distance(&coast, width, blocks_per_pixel);
        let maps = Maps {
            width,
            blocks_per_pixel,
            coast,
            height,
            tilt: vec![0.0; cells],
            squeeze: vec![0.0; cells],
            push_x: vec![0.0; cells],
            push_y: vec![0.0; cells],
            push_strength: vec![0.0; cells],
            skirt: vec![0.0; cells],
            coast_distance,
            lift_distance: point_distance(&[], width, blocks_per_pixel),
            highest_skirt: 0.0,
            water: 0.0,
            lowest: -1.0,
            highest: 1.0,
            most_crushed: 1.0,
            most_stretched: -1.0,
        };
        let straight = Spline::through(&[(0.0, 0.0), (1.0, 1.0)]);
        let settings = InterpreterSettings {
            deepest: 1.0,
            highlands: HIGHEST,
            land: straight.clone(),
            sea: straight,
            grade: 0.0,
            hills: 0.0,
            detail: 0.0,
            ..InterpreterSettings::default()
        };
        Self::new(Arc::new(maps), 1, settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_distance_to_the_nearest_point_is_measured_straight() {
        let width = 128;
        let points = [64 * width + 64, 10 * width + 10];
        let d = point_distance(&points, width, 32.0);
        // 30 east and 40 south of the middle point: 50 pixels.
        assert!((d[104 * width + 94] - 50.0 * 32.0).abs() < 1.0, "{} blocks", d[104 * width + 94]);
        // Diagonally from the corner point, 45 degrees: as round as straight across.
        assert!((d[30 * width + 30] - 20.0 * std::f32::consts::SQRT_2 * 32.0).abs() < 1.0, "{} blocks", d[30 * width + 30]);
        assert_eq!(d[10 * width + 10], 0.0);
        assert_eq!(point_distance(&[], width, 32.0)[0], 128.0 * 32.0);
    }

    #[test]
    fn the_lift_raises_land_toward_its_points_and_leaves_the_coast_and_sea_alone() {
        // An island (Z 150) of radius 28 pixels in a sea (Z 100), with a lift point in its middle.
        let width = 64;
        let island: Vec<f32> = (0..width * width)
            .map(|i| {
                let (x, y) = ((i % width) as f32 + 0.5 - 32.0, (i / width) as f32 + 0.5 - 32.0);
                if x * x + y * y < 28.0 * 28.0 { 150.0 } else { 100.0 }
            })
            .collect();
        let mut map = CompiledMap::of_heights(width, 32.0, &island);
        let middle = 32 * width + 32;
        Arc::make_mut(&mut map.maps).lift_distance = point_distance(&[middle], width, 32.0);
        let before: Vec<f32> = (0..width * width).map(|i| map.at_pixel(i)).collect();
        let mut settings = map.settings.clone();
        settings.lift = Lift { height: 50.0, reach: 30.0 * 32.0, pad: 200.0, ..Lift::default() };
        let lifted = map.with(settings, Mutes::default());
        // Right under the point: the full lift.
        assert!((lifted.at_pixel(middle) - before[middle] - 50.0).abs() < 0.5, "{} lifted to {}", before[middle], lifted.at_pixel(middle));
        // Halfway out (the curve is straight): half, as the coast is far enough off.
        let halfway = 32 * width + 32 + 15;
        assert!((lifted.at_pixel(halfway) - before[halfway] - 25.0).abs() < 2.0);
        // At the coast and out at sea: not raised (so the coastline stays put).
        for i in 0..width * width {
            let shore = map.maps.coast[i] >= 0.0 && map.maps.coast_distance[i] < 40.0;
            if map.maps.coast[i] < 0.0 || shore {
                assert!((lifted.at_pixel(i) - before[i]).abs() < 0.5, "pixel {i} went from {} to {}", before[i], lifted.at_pixel(i));
            }
        }
    }

    #[test]
    fn a_spline_passes_through_its_dots_without_overshooting() {
        let spline = Spline::through(&[(0.0, 0.0), (0.3, 0.2), (0.6, 0.2), (1.0, 1.0)]);
        for &(x, y) in &spline.dots {
            assert!((spline.at(x) - y).abs() < 1e-5);
        }
        // Flat between two equal dots, and never below the lower or above the higher dot of a stretch.
        for k in 0..=20 {
            let x = 0.3 + 0.3 * k as f32 / 20.0;
            assert!((spline.at(x) - 0.2).abs() < 1e-5, "{x}: {}", spline.at(x));
        }
        for k in 0..=100 {
            let x = k as f32 / 100.0;
            assert!((-1e-5..=1.0 + 1e-5).contains(&spline.at(x)));
        }
        // Smooth: no sudden jumps.
        for k in 0..100 {
            let (a, b) = (spline.at(k as f32 / 100.0), spline.at((k + 1) as f32 / 100.0));
            assert!((a - b).abs() < 0.05);
        }
    }

    /// Ridge crests counted along 4,000 blocks of a line running east from `east` blocks.
    fn crests_along(map: &CompiledMap, east: f32, south: f32) -> usize {
        let at = |x: f32| map.parts(&map.blended(x / map.maps.blocks_per_pixel - 0.5, south / map.maps.blocks_per_pixel - 0.5), x, south).mountains;
        let samples: Vec<f32> = (0..1000).map(|k| at(east + k as f32 * 4.0)).collect();
        samples.windows(3).filter(|w| w[1] > w[0] && w[1] >= w[2]).count()
    }

    #[test]
    fn steady_ridges_keep_their_spacing_where_the_squeeze_bends_far_from_the_corner() {
        // Squeezed everywhere, the push turning slowly from west to east across the map.
        let width = 64;
        let mut map = CompiledMap::of_heights(width, 1024.0, &vec![150.0; width * width]);
        let maps = Arc::make_mut(&mut map.maps);
        maps.most_crushed = 1.0;
        for i in 0..width * width {
            let turn = (i % width) as f32 * 0.04;
            maps.squeeze[i] = 1.0;
            maps.push_strength[i] = 1.0;
            (maps.push_x[i], maps.push_y[i]) = ((2.0 * turn).cos(), (2.0 * turn).sin());
        }
        let recipe = |steady| InterpreterSettings {
            mountains: Spline::through(&[(0.0, 1.0), (1.0, 1.0)]),
            ridges: 1.0,
            ridge_layers: 1,
            sharpness: 1.0,
            passes: 0.0,
            ridge_spacing: 400.0,
            ridge_length: 2_000.0,
            steady_ridges: steady,
            ..InterpreterSettings::default()
        };
        let (steady, old) = (map.with(recipe(true), Mutes::default()), map.with(recipe(false), Mutes::default()));
        let (near, far) = ((2_000.0, 30_000.0), (55_000.0, 30_000.0));
        let count = |map: &CompiledMap, (east, south): (f32, f32)| crests_along(map, east, south);
        // Near the corner both are about one crest per few hundred blocks; far out, the old way's
        // are crushed together and the steady ones aren't.
        assert!(count(&steady, far) < count(&steady, near) * 2 + 4, "steady: {} near, {} far", count(&steady, near), count(&steady, far));
        assert!(count(&old, far) > count(&steady, far) * 2, "old {} vs steady {} far out", count(&old, far), count(&steady, far));
    }

    #[test]
    fn coast_distance_grows_away_from_the_coast_on_both_sides() {
        // Land on the left half, sea on the right.
        let width = 20;
        let coast: Vec<f32> = (0..width * width).map(|i| if i % width < 10 { 0.01 } else { -0.01 }).collect();
        let d = coast_distance(&coast, width, 32.0);
        let at = |col: usize| d[5 * width + col];
        assert_eq!(at(9), 16.0);
        assert_eq!(at(10), -16.0);
        assert!((at(4) - (5.5 * 32.0)).abs() < 0.01, "{}", at(4));
        assert!((at(15) + 5.5 * 32.0).abs() < 0.01, "{}", at(15));
    }

    #[test]
    fn a_mask_fades_hills_out_by_its_curve() {
        // Land at Z 200 everywhere, hills 20 blocks high.
        let width = 16;
        let flat = CompiledMap::of_heights(width, 32.0, &vec![200.0; width * width]);
        let hilly = |mask: Mask| {
            let settings = InterpreterSettings { hills: 20.0, hill_mask: mask, ..flat.settings.clone() };
            let map = flat.with(settings, Mutes::default());
            // How far the ground strays from Z 200 over a stretch.
            (0..200).map(|k| (map.height(0.0, k as f32 * 2.0 - 200.0).unwrap() - 200.0).abs()).fold(0.0, f32::max)
        };
        assert!(hilly(Mask::default()) > 2.0, "hills everywhere show");
        // Following height (Z 200 is 0.6 across), through a curve that's 0 there: no hills.
        let none_here = Spline::through(&[(0.0, 1.0), (0.5, 0.0), (1.0, 0.0)]);
        assert_eq!(hilly(Mask { source: MaskSource::Height, curve: none_here.clone(), ..Mask::default() }), 0.0);
        // Inverted, the same curve reads 0.4 across: 0.2 of the hills show.
        let some = hilly(Mask { source: MaskSource::Height, curve: none_here, invert: true, ..Mask::default() });
        assert!(some > 0.0 && some < hilly(Mask::default()), "{some}");
    }

    #[test]
    fn blending_never_swings_past_the_pixels_round_a_spot() {
        // A cliff: Z 60 in the west half, Z 200 in the east. Crossing it, the ground stays between
        // the two (the old smoothing made a hump either side of every such step).
        let width = 8;
        let z: Vec<f32> = (0..width * width).map(|i| if i % width < 4 { 60.0 } else { 200.0 }).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        for k in 0..256 {
            let east = -128.0 + k as f32;
            let h = map.height(-16.0, east).unwrap();
            assert!((59.9..=200.1).contains(&h), "{h} at {east}");
        }
    }

    #[test]
    fn the_coast_follows_a_smooth_coast_map_between_pixels() {
        // Land and sea split along a diagonal line, given as a smooth coast map (as the compiler
        // softens it): the coastline the interpreter finds runs straight, not in pixel steps.
        let width = 16;
        let mut map = CompiledMap::of_heights(width, 32.0, &vec![140.0; width * width]);
        for i in 0..width * width {
            let (col, row) = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
            Arc::make_mut(&mut map.maps).coast[i] = (row - col * 0.5 - 4.0) * 0.01;
        }
        // For each block row (pixel rows 6 to 10), the first sea column east of the land: it should
        // move steadily, 2 blocks east per block south, never in 32-block jumps.
        let mut firsts = Vec::new();
        for r in 0..128 {
            let north = 48.0 - r as f32;
            let first = (0..512).find(|&c| map.height(north, -256.0 + c as f32).unwrap() < SEA_LEVEL as f32);
            firsts.push(first);
        }
        let firsts: Vec<i32> = firsts.into_iter().flatten().map(|c| c as i32).collect();
        assert!(firsts.len() > 100, "{firsts:?}");
        for pair in firsts.windows(2) {
            assert!((pair[1] - pair[0]).abs() <= 4, "the coast jumps: {firsts:?}");
        }
    }
}

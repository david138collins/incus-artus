//! The Interpreter — the end of the world-making chain: the recipe that gives every column of
//! blocks its ground height, worked out **at that block**, as the world is made.
//!
//! ```text
//! Tectonic Sim ──► bake (32-block pixels: height, squeeze, tilt…)
//!     │
//! World Compiler — whole-map jobs, done once:
//!     sea level · soften the coastline · foothill skirts
//!     │
//!     ▼  smooth maps (coast, height, tilt, squeeze and its direction, skirt)
//! Interpreter — at every block, as chunks are made (this file):
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
//! 6. **Hills** on land, and a little **detail** everywhere.

use super::terrain::SEA_LEVEL;
use crate::worldgen::noise::{Fade, perlin};
use serde::{Deserialize, Serialize};

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
/// Set on the Interpreter panel, next to the World Compiler; copied into each world.
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
    /// How tall mountains are for each amount of squeeze, from none (left) to the most crushed rock
    /// in the bake (right): up = a share of the room up to Z 333.
    pub mountains: Spline,
    /// How much the ridge pattern carves into the mountains: 0 = smooth domes, 1 = all ridges.
    pub ridges: f32,
    /// How far apart ridges are (across them), and how long they run (along them), in blocks.
    pub ridge_spacing: f32,
    pub ridge_length: f32,
    /// Higher = narrower, sharper ridge crests.
    pub sharpness: f32,
    /// How deeply the crest height dips into passes along each ridge (0 = even crests), and how far
    /// apart the dips are, in blocks.
    pub passes: f32,
    pub pass_spacing: f32,
    /// How deep the most stretched rock sinks, in blocks.
    pub rift_depth: f32,
    /// Rolling hills on land: how high (blocks either way) and how big (blocks).
    pub hills: f32,
    pub hill_size: f32,
    /// Small bumps on the ground: how high (blocks either way) and how wide (blocks).
    pub detail: f32,
    pub detail_size: f32,
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
            mountains: Spline::through(&[(0.0, 0.0), (0.08, 0.0), (0.5, 0.35), (1.0, 0.9)]),
            ridges: 0.5,
            ridge_spacing: 3_000.0,
            ridge_length: 15_000.0,
            sharpness: 1.2,
            passes: 0.4,
            pass_spacing: 12_000.0,
            rift_depth: 30.0,
            hills: 6.0,
            hill_size: 1_500.0,
            detail: 3.0,
            detail_size: 96.0,
            stone_from: 215,
            gabbro_from: 285,
        }
    }
}

// ── The compiler's maps ─────────────────────────────────────────────────────

/// What the World Compiler hands over: smooth maps, one value per pixel, and the bake's ranges the
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
    /// The bake's height at sea level, its lowest and highest heights, and its most crushed and
    /// most stretched rock.
    pub water: f32,
    pub lowest: f32,
    pub highest: f32,
    pub most_crushed: f32,
    pub most_stretched: f32,
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
}

/// The recipe's pieces at one spot, before they're put together.
#[derive(Clone, Copy, Debug)]
pub struct Parts {
    pub ground: f32,
    pub mountains: f32,
    pub rift: f32,
}

/// A compiled world: the compiler's maps, and the recipe that reads them.
#[derive(Clone, Debug)]
pub struct CompiledMap {
    pub maps: Maps,
    /// The bake's seed (the detail noise uses it; the recipe's noise a seed made from it).
    pub seed: u64,
    pub settings: InterpreterSettings,
    noise_seed: u64,
    land: Curve,
    sea: Curve,
    mountains: Curve,
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

/// A ridge pattern, 0 to 1 (1 = a crest): stacked "folded" noise, whose creases are the crests.
fn ridges(seed: u64, u: f32, v: f32, sharpness: f32) -> f32 {
    let (mut total, mut strength, mut strengths, mut scale) = (0.0, 1.0, 0.0, 1.0);
    for octave in 0..3 {
        let crest = 1.0 - perlin(seed, 0xa11 + octave, Fade::Smooth, u * scale, v * scale).abs().min(1.0);
        total += crest.powf(sharpness.max(0.1)) * strength;
        strengths += strength;
        strength *= 0.45;
        scale *= 2.1;
    }
    total / strengths
}

impl CompiledMap {
    pub fn new(maps: Maps, seed: u64, settings: InterpreterSettings) -> Self {
        Self {
            noise_seed: crate::fate::derive_seed(seed, "world compiler"),
            land: settings.land.curve(),
            sea: settings.sea.curve(),
            mountains: settings.mountains.curve(),
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
        }
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

        // 4. Mountains.
        let mut mountains = 0.0;
        let tall = self.mountains.at(here.squeeze / m.most_crushed.max(1e-6)).clamp(0.0, 1.0);
        if here.squeeze > 0.0 && tall > 0.0 {
            // The push's direction on the map (north is −y), from the doubled-angle arrow, and the
            // ridges lined up across it, blended into wandering ridges where the direction is
            // unclear.
            let angle = here.push_y.atan2(here.push_x) / 2.0;
            let push = (angle.cos(), -angle.sin());
            let across = x * push.0 + y * push.1;
            let along = -x * push.1 + y * push.0;
            let spacing = s.ridge_spacing.max(1.0);
            let seed = self.noise_seed;
            let lined_up = ridges(seed, across / spacing, along / s.ridge_length.max(1.0), s.sharpness);
            let wandering = ridges(seed ^ 0x5eed, x / spacing, y / spacing, s.sharpness);
            let clear = smooth(here.push_strength / 0.1);
            let crest = wandering + (lined_up - wandering) * clear;
            let carved = 1.0 - s.ridges.clamp(0.0, 1.0) * (1.0 - crest);
            // Passes: the crest height rises and dips along the ridge.
            let pass_size = s.pass_spacing.max(1.0);
            let along_ridge = perlin(seed, 0x9a55, Fade::Smooth, along / pass_size, across / (pass_size * 3.0));
            let wandering_passes = perlin(seed, 0x9a56, Fade::Smooth, x / pass_size, y / pass_size);
            let wave = wandering_passes + (along_ridge - wandering_passes) * clear;
            let dipped = 1.0 - s.passes.clamp(0.0, 1.0) * (0.5 - 0.5 * wave);
            mountains = (HIGHEST - ground).max(0.0) * tall * carved * dipped;
        }
        let mountains = mountains.max(here.skirt);

        // 5. Rifts.
        let rift = if here.squeeze < 0.0 { s.rift_depth * smooth(here.squeeze / m.most_stretched.min(-1e-6)) } else { 0.0 };
        Parts { ground, mountains, rift }
    }

    /// The pieces put together, with hills on land (6), before detail.
    pub fn put_together(&self, parts: Parts, x: f32, y: f32) -> f32 {
        let s = &self.settings;
        let sea = SEA_LEVEL as f32;
        let mut z = (parts.ground + parts.mountains).min(HIGHEST) - parts.rift;
        if z >= sea {
            let size = s.hill_size.max(1.0);
            let rolling = perlin(self.noise_seed, 0x4111, Fade::Smooth, x / size, y / size) * 0.7
                + perlin(self.noise_seed, 0x4112, Fade::Smooth, x * 2.3 / size, y * 2.3 / size) * 0.3;
            // Never drowns land.
            z = (z + rolling * s.hills).max(sea);
        }
        z.clamp(1.0, HIGHEST)
    }

    /// The ground's height at pixel `i`'s centre, without the block-sized detail: what the World
    /// Compiler shows.
    pub fn at_pixel(&self, i: usize) -> f32 {
        let (x, y) = self.pixel_spot(i);
        self.put_together(self.parts(&self.pixel(i), x, y), x, y)
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
        let ground = self.put_together(self.parts(&self.blended(col, row), x, y), x, y);
        let size = self.settings.detail_size.max(1.0);
        let detail = perlin(self.seed, 0xde7a11, Fade::Smooth, north / size, east / size) * self.settings.detail;
        Some(ground + detail)
    }

    /// A map whose ground is simply the given Z at each pixel (with plain straight curves, and no
    /// mountains, hills or detail): for tests.
    #[cfg(test)]
    pub fn of_heights(width: usize, blocks_per_pixel: f32, z: &[f32]) -> Self {
        let sea = SEA_LEVEL as f32;
        // Land: Z 128 + 205 × height (height 0 to 1). Sea: Z 127 − 126 × depth (height 0 to −1).
        let height: Vec<f32> = z.iter().map(|&z| if z >= sea { (z - sea) / (HIGHEST - sea) } else { -((sea - 1.0) - z) / (sea - 2.0) }).collect();
        let coast = height.iter().map(|&h| if h >= 0.0 { h.max(1e-6) } else { h }).collect();
        let cells = width * width;
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
        Self::new(maps, 1, settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            map.maps.coast[i] = (row - col * 0.5 - 4.0) * 0.01;
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

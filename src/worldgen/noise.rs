//! Perlin noise — the first kind of noise in the suite.
//!
//! **How it works.** Imagine a grid of posts, one unit apart. Each post gets a random arrow lying
//! flat (from the seed, so the same seed always gives the same arrows). The value at any point is
//! worked out from the four posts around it: how far the point lies along each post's arrow,
//! blended together with a smooth curve (the **fade**). The result is soft hills and hollows,
//! about one per grid square, from about −1 to 1.
//!
//! **Octaves.** One layer of that looks like blobs. Real landscapes have detail at every size, so
//! several layers are stacked: each one finer than the last (by the **lacunarity**: 2 = twice as
//! fine) and fainter (by the **persistence**: 0.5 = half as strong). This stack is called
//! "fractal Brownian motion" (fBm).
//!
//! **Then the output is shaped** (stretched to fill black to white, contrast, brightness, curve,
//! invert), so a field can be made to fit a purpose without changing the pattern underneath.
//!
//! The randomness comes from Fate's `planet_hash`.

use super::field::Field;
use super::made::{Made, Shaping};
use crate::fate::planet_hash;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// The curve used to blend between grid posts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Fade {
    /// 6t⁵ − 15t⁴ + 10t³ (Perlin's improved noise, 2002). Smooth all the way through.
    #[default]
    Smooth,
    /// 3t² − 2t³ (Perlin's first noise, 1985). Slightly creased where grid squares meet.
    Soft,
    /// Straight lines. Shows the grid clearly; useful for seeing what the fade does.
    Straight,
}

impl Fade {
    pub const ALL: [Fade; 3] = [Fade::Smooth, Fade::Soft, Fade::Straight];

    pub fn name(self) -> &'static str {
        match self {
            Fade::Smooth => "smooth (2002)",
            Fade::Soft => "soft (1985)",
            Fade::Straight => "straight",
        }
    }

    fn apply(self, t: f32) -> f32 {
        match self {
            Fade::Smooth => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
            Fade::Soft => t * t * (3.0 - 2.0 * t),
            Fade::Straight => t,
        }
    }
}

/// Everything that makes one kind of Perlin field. This is what a noise preset saves. The seed
/// isn't part of it, so a preset is a *kind* of noise that works with any seed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PerlinSettings {
    // ── Pattern ──
    /// The size of the biggest features, in blocks (one grid square of the first octave).
    pub scale: f32,
    pub octaves: u32,
    /// How strong each octave is compared with the one before.
    pub persistence: f32,
    /// How much finer each octave is than the one before.
    pub lacunarity: f32,

    // ── Placement ──
    /// Slide the whole field, in blocks.
    pub offset_north: f32,
    pub offset_east: f32,
    /// Stretch the features: 1 = round, 3 = three times as long as they are wide.
    pub stretch: f32,
    /// Which way the stretch runs, in degrees: 0 = east–west, 90 = north–south.
    pub stretch_angle: f32,
    /// Turn each octave this many degrees from the one before, so the grids don't line up.
    pub octave_turn: f32,

    // ── Maths ──
    pub fade: Fade,
    /// Hear one octave on its own (0 = all of them), like soloing a channel on a mixing desk.
    pub solo_octave: u32,

    // ── Output ── (written alongside the other settings in presets, as before)
    #[serde(flatten)]
    pub output: Shaping,
}

impl Default for PerlinSettings {
    fn default() -> Self {
        Self {
            scale: 15_000.0,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
            offset_north: 0.0,
            offset_east: 0.0,
            stretch: 1.0,
            stretch_angle: 0.0,
            octave_turn: 0.0,
            fade: Fade::Smooth,
            solo_octave: 0,
            output: Shaping::default(),
        }
    }
}

/// Perlin noise at a point, about −1 to 1, one feature per unit.
pub fn perlin(seed: u64, salt: u64, fade: Fade, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (ix, iy) = (x0 as i64, y0 as i64);
    // Each grid post's arrow: a random direction. The value is how far along it the point lies.
    let post = |px: i64, py: i64, dx: f32, dy: f32| {
        let angle = (planet_hash(seed, salt, px, py, 0) >> 40) as f32 / (1u64 << 24) as f32 * std::f32::consts::TAU;
        angle.cos() * dx + angle.sin() * dy
    };
    let (u, v) = (fade.apply(fx), fade.apply(fy));
    let a = post(ix, iy, fx, fy);
    let b = post(ix + 1, iy, fx - 1.0, fy);
    let c = post(ix, iy + 1, fx, fy - 1.0);
    let d = post(ix + 1, iy + 1, fx - 1.0, fy - 1.0);
    let top = a + (b - a) * u;
    let bottom = c + (d - c) * u;
    // Perlin noise rarely goes past ±0.7; stretch it to about ±1.
    (top + (bottom - top) * v) * 1.4
}

/// Smooth Perlin noise at a point, as `perlin` gives it, with how steep it is there: (value,
/// slope along x, slope along y), the slopes per unit, worked out exactly alongside the value at
/// almost no cost [Inigo Quilez, "More Noise"; for the peaks' crags, 2026-10-27].
pub fn perlin_with_slope(seed: u64, salt: u64, x: f32, y: f32) -> (f32, f32, f32) {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (ix, iy) = (x0 as i64, y0 as i64);
    // Each grid post's arrow; the value is how far along it the point lies, which changes by the
    // arrow itself as the point moves.
    let arrow = |px: i64, py: i64| {
        let angle = (planet_hash(seed, salt, px, py, 0) >> 40) as f32 / (1u64 << 24) as f32 * std::f32::consts::TAU;
        (angle.cos(), angle.sin())
    };
    let (ga, gb, gc, gd) = (arrow(ix, iy), arrow(ix + 1, iy), arrow(ix, iy + 1), arrow(ix + 1, iy + 1));
    let a = ga.0 * fx + ga.1 * fy;
    let b = gb.0 * (fx - 1.0) + gb.1 * fy;
    let c = gc.0 * fx + gc.1 * (fy - 1.0);
    let d = gd.0 * (fx - 1.0) + gd.1 * (fy - 1.0);
    // The smooth fade, and how steep it is.
    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let fade_slope = |t: f32| 30.0 * t * t * (t - 1.0) * (t - 1.0);
    let (u, v, du, dv) = (fade(fx), fade(fy), fade_slope(fx), fade_slope(fy));
    // value = a + u(b − a) + v(c − a) + uv(a − b − c + d), and its slope along each way.
    let k = a - b - c + d;
    let value = a + u * (b - a) + v * (c - a) + u * v * k;
    let along_x = ga.0 + u * (gb.0 - ga.0) + v * (gc.0 - ga.0) + u * v * (ga.0 - gb.0 - gc.0 + gd.0) + du * ((b - a) + v * k);
    let along_y = ga.1 + u * (gb.1 - ga.1) + v * (gc.1 - ga.1) + u * v * (ga.1 - gb.1 - gc.1 + gd.1) + dv * ((c - a) + u * k);
    // Stretched as `perlin` is.
    (value * 1.4, along_x * 1.4, along_y * 1.4)
}

/// Which maths the noise is worked out in, in the recipe that makes blocks as the game is played
/// (saved with it, so a world always comes back with its own) [David, 2026-11-01: "projected code is
/// GPU-safe"; a switch, to compare worlds both ways]. Whole numbers by default since David's look
/// ("i'd like for whole math to be the default setting now"); the fractions stay until he says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseMaths {
    /// Fractions, with sines and cosines: as the noise was until 2026-11-01.
    Fractions,
    /// Whole numbers only (`whole_perlin`): every chip, the CPU's or the GPU's, gets the same answer
    /// exactly.
    #[default]
    WholeNumbers,
}

/// Perlin noise in the maths given.
pub fn perlin_in(maths: NoiseMaths, seed: u64, salt: u64, fade: Fade, x: f32, y: f32) -> f32 {
    match maths {
        NoiseMaths::Fractions => perlin(seed, salt, fade, x, y),
        NoiseMaths::WholeNumbers => whole_perlin(seed, salt, fade, x, y),
    }
}

/// The same, with its slope (`perlin_with_slope`).
pub fn perlin_with_slope_in(maths: NoiseMaths, seed: u64, salt: u64, x: f32, y: f32) -> (f32, f32, f32) {
    match maths {
        NoiseMaths::Fractions => perlin_with_slope(seed, salt, x, y),
        NoiseMaths::WholeNumbers => whole_perlin_with_slope(seed, salt, x, y),
    }
}

// ── Whole-number noise ──────────────────────────────────────────────────────────────────────────
//
// The same idea as `perlin` (posts one unit apart, an arrow at each, blended by the fade), in whole
// numbers only, in 32 bits, so a GPU shader can do exactly the same:
// - **the barcodes** are a 32-bit hash (shaders have no 64-bit numbers), so the patterns are new;
// - **the arrows** are 64 fixed directions from a table, not sines and cosines;
// - **positions and values** are in 65536ths of a unit, **the arrows and fades** in 4096ths. Every
//   step is chosen so nothing can overflow 32 bits. [At first the positions and values were in
//   4096ths too: that grain was coarse enough to make tiny wiggles, false crests along the ridges,
//   found by a ridges test the day whole numbers became the default.]
// Only the way in (a position given as a fraction) and the way out (back to a fraction, by a power
// of two, which is exact) aren't whole numbers.

/// 4096ths: a unit, in the arrows and the fades.
const ONE: i32 = 4096;
/// 65536ths: a unit, in the positions and the values.
const FINE: i32 = 65_536;

/// 64 arrows, evenly round a turn, each 4096 long.
#[rustfmt::skip]
const ARROWS: [(i32, i32); 64] = [
    (4096, 0), (4076, 401), (4017, 799), (3920, 1189), (3784, 1567), (3612, 1931), (3406, 2276), (3166, 2598),
    (2896, 2896), (2598, 3166), (2276, 3406), (1931, 3612), (1567, 3784), (1189, 3920), (799, 4017), (401, 4076),
    (0, 4096), (-401, 4076), (-799, 4017), (-1189, 3920), (-1567, 3784), (-1931, 3612), (-2276, 3406), (-2598, 3166),
    (-2896, 2896), (-3166, 2598), (-3406, 2276), (-3612, 1931), (-3784, 1567), (-3920, 1189), (-4017, 799), (-4076, 401),
    (-4096, 0), (-4076, -401), (-4017, -799), (-3920, -1189), (-3784, -1567), (-3612, -1931), (-3406, -2276), (-3166, -2598),
    (-2896, -2896), (-2598, -3166), (-2276, -3406), (-1931, -3612), (-1567, -3784), (-1189, -3920), (-799, -4017), (-401, -4076),
    (0, -4096), (401, -4076), (799, -4017), (1189, -3920), (1567, -3784), (1931, -3612), (2276, -3406), (2598, -3166),
    (2896, -2896), (3166, -2598), (3406, -2276), (3612, -1931), (3784, -1567), (3920, -1189), (4017, -799), (4076, -401),
];

/// A noise's key: its seed and what it's for, folded into 32 bits (once, on the processor: the card is
/// handed the key, since shaders have no 64-bit numbers).
pub fn noise_key(seed: u64, salt: u64) -> u32 {
    let fold = |v: u64| (v ^ (v >> 32)) as u32;
    fold(seed) ^ fold(salt).wrapping_mul(0x9e37_79b9)
}

/// A grid post's 32-bit barcode (murmur3's mixing), from its noise's key and where it is.
fn post_hash(key: u32, x: i32, y: i32) -> u32 {
    let mut h = key;
    for k in [x as u32, y as u32] {
        h ^= k.wrapping_mul(0xcc9e_2d51).rotate_left(15).wrapping_mul(0x1b87_3593);
        h = h.rotate_left(13).wrapping_mul(5).wrapping_add(0xe654_6b64);
    }
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// A post's arrow.
fn arrow(key: u32, x: i32, y: i32) -> (i32, i32) {
    ARROWS[(post_hash(key, x, y) >> 26) as usize]
}

/// A fade, in 65536ths, of `t` (0 to 65535). In unsigned 32 bits, each product split so it fits: a
/// fade only in 4096ths stepped, and the value ticked back a little at each step, a faint sawtooth
/// that made false crests.
fn whole_fade(fade: Fade, t: i32) -> i32 {
    let t = t as u32;
    let t2 = (t * t) >> 16;
    match fade {
        // 6t⁵ − 15t⁴ + 10t³, as t³ (6t² − 15t + 10): the second factor 1 to 10, in 65536ths.
        Fade::Smooth => {
            let t3 = (t2 * t) >> 16;
            let inner = 10 * FINE as u32 + 6 * t2 - 15 * t;
            ((t3 * (inner >> 4)) >> 12) as i32
        }
        // 3t² − 2t³, as t² (3 − 2t).
        Fade::Soft => ((t2 * ((3 * FINE as u32 - 2 * t) >> 2)) >> 14) as i32,
        Fade::Straight => t as i32,
    }
}

/// A position as a whole number of 65536ths: its grid post, and how far past it (0 to 65535). (The
/// post wraps round far out, past 32,768 units, so the pattern repeats there rather than failing.)
fn fixed(x: f32) -> (i32, i32) {
    let q = (x as f64 * FINE as f64).floor() as i64;
    ((q >> 16) as i32, (q & (FINE as i64 - 1)) as i32)
}

/// The four posts' arrows round a point, and how far along each the point lies (in 65536ths).
struct Corners {
    arrows: [(i32, i32); 4],
    dots: [i32; 4],
}

fn corners(key: u32, (ix, fx): (i32, i32), (iy, fy): (i32, i32)) -> Corners {
    let arrows = [arrow(key, ix, iy), arrow(key, ix + 1, iy), arrow(key, ix, iy + 1), arrow(key, ix + 1, iy + 1)];
    let offsets = [(fx, fy), (fx - FINE, fy), (fx, fy - FINE), (fx - FINE, fy - FINE)];
    // Each at most 4096 × 65536 × 2 (2²⁹) before it's brought back to 65536ths.
    let dots = [0, 1, 2, 3].map(|k| (arrows[k].0 * offsets[k].0 + arrows[k].1 * offsets[k].1) >> 12);
    Corners { arrows, dots }
}

/// `a` to `b` by `t` (in 65536ths). `b − a` is at most 2¹⁸, so `t` goes in two parts: its 4096ths,
/// then the rest, each product under 2³⁰.
fn mix(a: i32, b: i32, t: i32) -> i32 {
    let gap = b - a;
    a + ((gap * (t >> 4)) >> 12) + ((gap * (t & 15)) >> 16)
}

/// Stretched to about ±1 as `perlin` is (× 1.4, as 1434/1024), then back to a fraction: a value in
/// 65536ths, or a slope in 4096ths.
fn out(value: i32, unit: i32) -> f32 {
    ((value * 1434) >> 10) as f32 / unit as f32
}

/// Perlin noise at a point, about −1 to 1, one feature per unit, in whole numbers.
pub fn whole_perlin(seed: u64, salt: u64, fade: Fade, x: f32, y: f32) -> f32 {
    out(whole_value(noise_key(seed, salt), fade, fixed(x), fixed(y)), FINE)
}

/// The noise's value in 65536ths, before it's stretched, at a post and how far past it.
fn whole_value(key: u32, fade: Fade, px: (i32, i32), py: (i32, i32)) -> i32 {
    let c = corners(key, px, py);
    let (u, v) = (whole_fade(fade, px.1), whole_fade(fade, py.1));
    let [a, b, cc, d] = c.dots;
    mix(mix(a, b, u), mix(cc, d, u), v)
}

/// Whole numbers in, whole numbers out (the recipe in play: `world/whole_recipe.rs`): the noise at a
/// point given in its own units as a `Fix` (its grid wraps round every 65,536 units, as a `Fix`
/// does), about −1 to 1.
pub fn whole_perlin_at(key: u32, fade: Fade, x: crate::whole::Fix, y: crate::whole::Fix) -> crate::whole::Fix {
    let post = |f: crate::whole::Fix| (f.0 >> 16, f.0 & (FINE - 1));
    crate::whole::Fix((whole_value(key, fade, post(x), post(y)) * 1434) >> 10)
}

/// The same, with its slope along x and y (per unit), all `Fix`.
pub fn whole_perlin_with_slope_at(key: u32, x: crate::whole::Fix, y: crate::whole::Fix) -> (crate::whole::Fix, crate::whole::Fix, crate::whole::Fix) {
    let post = |f: crate::whole::Fix| (f.0 >> 16, f.0 & (FINE - 1));
    let (value, along_x, along_y) = whole_slopes(key, post(x), post(y));
    // The value in 65536ths; the slopes in 4096ths, brought to 65536ths (×16, exact).
    let stretched = |v: i32| (v * 1434) >> 10;
    (crate::whole::Fix(stretched(value)), crate::whole::Fix(stretched(along_x) << 4), crate::whole::Fix(stretched(along_y) << 4))
}

/// The same with its slope along x and y (per unit), as `perlin_with_slope` gives them, in whole
/// numbers (the smooth fade).
pub fn whole_perlin_with_slope(seed: u64, salt: u64, x: f32, y: f32) -> (f32, f32, f32) {
    let (value, along_x, along_y) = whole_slopes(noise_key(seed, salt), fixed(x), fixed(y));
    (out(value, FINE), out(along_x, ONE), out(along_y, ONE))
}

/// The noise's value (in 65536ths) and its slopes (in 4096ths), before they're stretched.
fn whole_slopes(key: u32, px: (i32, i32), py: (i32, i32)) -> (i32, i32, i32) {
    let c = corners(key, px, py);
    let (fine_u, fine_v) = (whole_fade(Fade::Smooth, px.1), whole_fade(Fade::Smooth, py.1));
    let value = mix(mix(c.dots[0], c.dots[1], fine_u), mix(c.dots[2], c.dots[3], fine_u), fine_v);
    // The slopes in 4096ths (they're finer than the blocks need).
    let (fx, fy) = (px.1 >> 4, py.1 >> 4);
    let (u, v) = (fine_u >> 4, fine_v >> 4);
    // The smooth fade's steepness, 30 t² (t − 1)², in 4096ths.
    let steep = |t: i32| 30 * ((((t * t) >> 12) * (((t - ONE) * (t - ONE)) >> 12)) >> 12);
    let (du, dv) = (steep(fx), steep(fy));
    let [a, b, cc, d] = c.dots;
    let [ga, gb, gc, gd] = c.arrows;
    let k = a - b - cc + d;
    let uv = (u * v) >> 12;
    // In 4096ths: the arrows blended, and the fade's steepness times the values' change across
    // (65536ths brought to 4096ths).
    let along = |g: [i32; 4], fade_steep: i32, across: i32| {
        g[0] + ((u * (g[1] - g[0])) >> 12) + ((v * (g[2] - g[0])) >> 12) + ((uv * (g[0] - g[1] - g[2] + g[3])) >> 12) + ((fade_steep * (across >> 4)) >> 12)
    };
    let along_x = along([ga.0, gb.0, gc.0, gd.0], du, (b - a) + ((v * k) >> 12));
    let along_y = along([ga.1, gb.1, gc.1, gd.1], dv, (cc - a) + ((u * k) >> 12));
    (value, along_x, along_y)
}

// ── Whole-number noise in 3D, for cave walls ────────────────────────────────
//
// Value noise (a number at each post, blended by the smooth fade), given whole blocks and a feature
// size in whole blocks, so nothing at all is a fraction: a GPU shader can do exactly the same, in 32
// bits [GPU-safe: docs/caves.md, stop 1]. Lumps rather than Perlin's swirls, which is what a wall
// pushed in and out needs.

/// A 32-bit barcode for a post in 3D (murmur3's mixing, as `post_hash`).
fn hash32_3(seed: u32, x: i32, y: i32, z: i32) -> u32 {
    let mut h = seed ^ 0x2545_f491;
    for k in [x as u32, y as u32, z as u32] {
        h ^= k.wrapping_mul(0xcc9e_2d51).rotate_left(15).wrapping_mul(0x1b87_3593);
        h = h.rotate_left(13).wrapping_mul(5).wrapping_add(0xe654_6b64);
    }
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// Value noise at a block, one lump every `size` blocks (at least 1): −65536 to 65535, in 65536ths.
pub fn whole_value3(seed: u32, block: [i32; 3], size: i32) -> i32 {
    let size = size.max(1);
    // Each axis: its post, and how far past it in 65536ths (the remainder is under `size`, so this
    // stays under 2³¹ for lumps up to 32,768 blocks).
    let [(ix, fx), (iy, fy), (iz, fz)] = block.map(|b| (b.div_euclid(size), b.rem_euclid(size) * FINE / size));
    let value = |x: i32, y: i32, z: i32| (hash32_3(seed, x, y, z) >> 15) as i32 - FINE;
    let (u, v, w) = (whole_fade(Fade::Smooth, fx), whole_fade(Fade::Smooth, fy), whole_fade(Fade::Smooth, fz));
    let plane = |z: i32| mix(mix(value(ix, iy, z), value(ix + 1, iy, z), u), mix(value(ix, iy + 1, z), value(ix + 1, iy + 1, z), u), v);
    mix(plane(iz), plane(iz + 1), w)
}

impl PerlinSettings {
    /// The raw stacked value at a position in blocks (about −1 to 1), before the output shaping.
    pub fn raw(&self, seed: u64, north: f32, east: f32) -> f32 {
        self.raw_in(NoiseMaths::Fractions, seed, north, east)
    }

    /// The same, its noise in the maths given.
    pub fn raw_in(&self, maths: NoiseMaths, seed: u64, north: f32, east: f32) -> f32 {
        let (mut x, mut y) = (east + self.offset_east, north + self.offset_north);
        // Stretch: turn into the stretch's direction, then squash along it.
        let (sin, cos) = self.stretch_angle.to_radians().sin_cos();
        (x, y) = ((x * cos + y * sin) / self.stretch.max(0.01), -x * sin + y * cos);

        let (turn_sin, turn_cos) = self.octave_turn.to_radians().sin_cos();
        let (mut total, mut strength, mut strengths) = (0.0, 1.0, 0.0);
        let mut size = self.scale.max(1.0);
        for octave in 1..=self.octaves.max(1) {
            let value = perlin_in(maths, seed, octave as u64, self.fade, x / size, y / size);
            if self.solo_octave == octave {
                return value;
            }
            total += value * strength;
            strengths += strength;
            strength *= self.persistence;
            size /= self.lacunarity.max(1.01);
            (x, y) = (x * turn_cos - y * turn_sin, x * turn_sin + y * turn_cos);
        }
        total / strengths
    }
}

/// Makes the field. Returns `None` if `stop` was set part-way (a newer version is wanted).
pub fn make(settings: &PerlinSettings, seed: u64, size_blocks: f32, blocks_per_pixel: f32, stop: &AtomicBool) -> Option<Made> {
    let mut raw = Field::new(size_blocks, blocks_per_pixel);
    raw.fill(|north, east| settings.raw(seed, north, east));
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    // Unfilled, raw −1 is black and 1 is white.
    Some(Made::finish(raw, None, None, &settings.output, |v| v * 0.5 + 0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_noise_with_its_slope_matches_the_noise_and_its_measured_slope() {
        for k in 0..200 {
            let (x, y) = (k as f32 * 0.731 - 40.0, k as f32 * 0.377 + 13.1);
            let (value, along_x, along_y) = perlin_with_slope(9, 3, x, y);
            assert!((value - perlin(9, 3, Fade::Smooth, x, y)).abs() < 1e-5, "value at {x}, {y}");
            // The slope, measured: a tiny step each way.
            let h = 1e-3;
            let measured_x = (perlin(9, 3, Fade::Smooth, x + h, y) - perlin(9, 3, Fade::Smooth, x - h, y)) / (2.0 * h);
            let measured_y = (perlin(9, 3, Fade::Smooth, x, y + h) - perlin(9, 3, Fade::Smooth, x, y - h)) / (2.0 * h);
            assert!((along_x - measured_x).abs() < 0.02, "along x at {x}, {y}: {along_x} vs {measured_x}");
            assert!((along_y - measured_y).abs() < 0.02, "along y at {x}, {y}: {along_y} vs {measured_y}");
        }
    }

    /// Given whole numbers, the noise is the very same as given the same point as a fraction.
    #[test]
    fn the_noise_in_whole_numbers_is_the_noise() {
        use crate::whole::Fix;
        for k in 0..500 {
            // Points a fraction holds exactly (in 256ths), so both ways start from the same spot.
            let (x, y) = ((k * 37 % 9000) as f32 / 256.0 - 17.0, (k * 91 % 7000) as f32 / 256.0 + 3.0);
            let key = noise_key(7, 0x51);
            let whole = whole_perlin_at(key, Fade::Smooth, Fix::from_f32(x), Fix::from_f32(y));
            assert_eq!(whole.to_f32(), whole_perlin(7, 0x51, Fade::Smooth, x, y), "at {x}, {y}");
            let (v, sx, sy) = whole_perlin_with_slope_at(key, Fix::from_f32(x), Fix::from_f32(y));
            assert_eq!((v.to_f32(), sx.to_f32(), sy.to_f32()), whole_perlin_with_slope(7, 0x51, x, y), "with its slope at {x}, {y}");
        }
    }

    #[test]
    fn whole_number_noise_is_smooth_zero_on_its_posts_and_like_the_old_in_character() {
        // Zero on every post, as Perlin noise is.
        assert_eq!(whole_perlin(3, 1, Fade::Smooth, 4.0, -2.0), 0.0);
        // Smooth: a 4096th of a unit moves it hardly at all, anywhere.
        for k in 0..2_000 {
            let (x, y) = (k as f32 * 0.6173 - 500.0, k as f32 * 0.4219 + 77.0);
            let step = whole_perlin(1, 2, Fade::Smooth, x + 1.0 / 4096.0, y) - whole_perlin(1, 2, Fade::Smooth, x, y);
            assert!(step.abs() < 0.01, "a jump of {step} at {x}, {y}");
        }
        // Its spread of values: as wide, and as often near the middle, as the old noise's.
        let stats = |noise: &dyn Fn(f32, f32) -> f32| {
            let values: Vec<f32> = (0..40_000).map(|k| noise((k % 200) as f32 * 0.137, (k / 200) as f32 * 0.137)).collect();
            let mean = values.iter().sum::<f32>() / values.len() as f32;
            let spread = (values.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / values.len() as f32).sqrt();
            let (low, high) = values.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            (mean, spread, low, high)
        };
        let (old_mean, old_spread, _, _) = stats(&|x, y| perlin(11, 4, Fade::Smooth, x, y));
        let (mean, spread, low, high) = stats(&|x, y| whole_perlin(11, 4, Fade::Smooth, x, y));
        assert!(mean.abs() < 0.05 && old_mean.abs() < 0.05, "{mean} {old_mean}");
        assert!((spread / old_spread - 1.0).abs() < 0.15, "spread {spread} against {old_spread}");
        assert!(low > -1.5 && high < 1.5 && low < -0.6 && high > 0.6, "{low}..{high}");
        // Every fade, far out (no number too big for 32 bits).
        for fade in Fade::ALL {
            for k in 0..500 {
                let v = whole_perlin(u64::MAX - k, k, fade, 40_000.0 - k as f32 * 77.7, -39_000.0 + k as f32 * 81.1);
                assert!(v.abs() < 1.5, "{fade:?}: {v}");
            }
        }
    }

    #[test]
    fn whole_number_noise_is_fine_enough_to_make_no_false_crests() {
        // Crests (as the ridges fold the noise) along lines: as many as the old noise's, give or take.
        let (mut old, mut new) = (0, 0);
        for seed in 0..30u64 {
            let count = |f: &dyn Fn(f32) -> f32| {
                let s: Vec<f32> = (0..1000).map(|k| 1.0 - f(k as f32 * 0.01 - 5.0).abs()).collect();
                s.windows(3).filter(|w| w[1] > w[0] && w[1] >= w[2]).count()
            };
            old += count(&|x| perlin(seed, 0xa11, Fade::Smooth, x, 3.3));
            new += count(&|x| whole_perlin(seed, 0xa11, Fade::Smooth, x, 3.3));
        }
        assert!((new as f32 / old as f32 - 1.0).abs() < 0.2, "{new} crests against {old}");
        // And it never ticks back against its way by more than a thousandth (a fifth of a block on
        // a 200-block ridge, far less in practice).
        let mut most = 0.0f32;
        for seed in 0..20u64 {
            let s: Vec<f32> = (0..5_000).map(|k| whole_perlin(seed, 0xa11, Fade::Smooth, k as f32 * 0.001 - 3.0, 3.3)).collect();
            for k in 5..s.len() - 6 {
                let (trend, step) = (s[k + 5] - s[k - 5], s[k + 1] - s[k]);
                if trend.abs() > 0.002 && step * trend < 0.0 {
                    most = most.max(step.abs());
                }
            }
        }
        assert!(most < 0.001, "it ticked back {most}");
    }

    #[test]
    fn whole_number_noise_with_its_slope_matches_it_and_its_measured_slope() {
        for k in 0..300 {
            let (x, y) = (k as f32 * 0.731 - 40.0, k as f32 * 0.377 + 13.1);
            let (value, along_x, along_y) = whole_perlin_with_slope(9, 3, x, y);
            assert_eq!(value, whole_perlin(9, 3, Fade::Smooth, x, y), "the value at {x}, {y}");
            let h = 1.0 / 64.0;
            let measured_x = (whole_perlin(9, 3, Fade::Smooth, x + h, y) - whole_perlin(9, 3, Fade::Smooth, x - h, y)) / (2.0 * h);
            let measured_y = (whole_perlin(9, 3, Fade::Smooth, x, y + h) - whole_perlin(9, 3, Fade::Smooth, x, y - h)) / (2.0 * h);
            assert!((along_x - measured_x).abs() < 0.08, "along x at {x}, {y}: {along_x} vs {measured_x}");
            assert!((along_y - measured_y).abs() < 0.08, "along y at {x}, {y}: {along_y} vs {measured_y}");
        }
    }

    #[test]
    fn same_seed_same_noise() {
        assert_eq!(perlin(5, 1, Fade::Smooth, 3.3, 7.7), perlin(5, 1, Fade::Smooth, 3.3, 7.7));
        assert_ne!(perlin(5, 1, Fade::Smooth, 3.3, 7.7), perlin(6, 1, Fade::Smooth, 3.3, 7.7));
    }

    #[test]
    fn noise_stays_roughly_between_minus_one_and_one() {
        let settings = PerlinSettings { scale: 10.0, octaves: 5, ..PerlinSettings::default() };
        let (mut low, mut high) = (f32::MAX, f32::MIN);
        for i in 0..20_000 {
            let value = settings.raw(9, (i % 200) as f32 * 0.37, (i / 200) as f32 * 0.37);
            low = low.min(value);
            high = high.max(value);
        }
        assert!(low > -1.3 && high < 1.3, "{low}..{high}");
        assert!(low < -0.4 && high > 0.4, "it should actually use the range: {low}..{high}");
    }

    #[test]
    fn noise_is_smooth() {
        let step = perlin(1, 2, Fade::Smooth, 0.501, 0.5) - perlin(1, 2, Fade::Smooth, 0.5, 0.5);
        assert!(step.abs() < 0.01);
    }

    #[test]
    fn noise_is_zero_on_the_grid_posts() {
        // A defining property of Perlin noise: every post sits at exactly middle grey.
        assert_eq!(perlin(3, 1, Fade::Smooth, 4.0, -2.0), 0.0);
    }

    #[test]
    fn filling_the_range_reaches_black_and_white() {
        let made = make(&PerlinSettings::default(), 1, 72_000.0, 512.0, &AtomicBool::new(false)).unwrap();
        let (low, high) = made.shaped.values.iter().fold((1.0f32, 0.0f32), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        assert_eq!((low, high), (0.0, 1.0));
        assert_eq!(made.stats.histogram.iter().sum::<u32>() as usize, made.shaped.values.len());
    }

    #[test]
    fn a_preset_survives_being_written_and_read() {
        let output = Shaping { invert: true, ..Shaping::default() };
        let settings = PerlinSettings { octaves: 3, fade: Fade::Soft, output, ..PerlinSettings::default() };
        let text = toml::to_string(&settings).unwrap();
        assert_eq!(toml::from_str::<PerlinSettings>(&text).unwrap(), settings);
    }

    #[test]
    fn presets_saved_before_the_output_settings_moved_still_load() {
        // Written the way presets were saved before image layers (output settings side by side).
        let old = "scale = 9000.0\noctaves = 4\ninvert = true\ncontrast = 2.0\nfill_range = false\n";
        let settings: PerlinSettings = toml::from_str(old).unwrap();
        assert_eq!((settings.scale, settings.octaves), (9000.0, 4));
        assert!(settings.output.invert && !settings.output.fill_range);
        assert_eq!(settings.output.contrast, 2.0);
        assert_eq!(settings.output.curve, 1.0, "settings not in the file keep their defaults");
        // And they're still written side by side.
        assert!(toml::to_string(&settings).unwrap().contains("\ninvert = true"));
    }
}

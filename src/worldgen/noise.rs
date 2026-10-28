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

impl PerlinSettings {
    /// The raw stacked value at a position in blocks (about −1 to 1), before the output shaping.
    pub fn raw(&self, seed: u64, north: f32, east: f32) -> f32 {
        let (mut x, mut y) = (east + self.offset_east, north + self.offset_north);
        // Stretch: turn into the stretch's direction, then squash along it.
        let (sin, cos) = self.stretch_angle.to_radians().sin_cos();
        (x, y) = ((x * cos + y * sin) / self.stretch.max(0.01), -x * sin + y * cos);

        let (turn_sin, turn_cos) = self.octave_turn.to_radians().sin_cos();
        let (mut total, mut strength, mut strengths) = (0.0, 1.0, 0.0);
        let mut size = self.scale.max(1.0);
        for octave in 1..=self.octaves.max(1) {
            let value = perlin(seed, octave as u64, self.fade, x / size, y / size);
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

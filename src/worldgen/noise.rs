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

    // ── Output ──
    /// Stretch the lowest value to black and the highest to white.
    pub fill_range: bool,
    /// Above 1 pushes values away from middle grey; below 1 pulls them in.
    pub contrast: f32,
    /// Added after contrast (−1 to 1).
    pub brightness: f32,
    /// Bends the values: above 1 makes most of the field darker, below 1 lighter.
    pub curve: f32,
    pub invert: bool,
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
            fill_range: true,
            contrast: 1.0,
            brightness: 0.0,
            curve: 1.0,
            invert: false,
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

    /// Contrast, brightness, curve and invert, on a value already between 0 and 1.
    pub fn shape(&self, t: f32) -> f32 {
        let t = ((t - 0.5) * self.contrast + 0.5 + self.brightness).clamp(0.0, 1.0);
        let t = t.powf(self.curve.max(0.01));
        if self.invert { 1.0 - t } else { t }
    }
}

/// Numbers about a finished field, for judging it at a glance.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub raw_low: f32,
    pub raw_high: f32,
    pub mean: f32,
    /// Share of pixels pushed to pure black or pure white by the shaping (detail lost there).
    pub clipped_black: f32,
    pub clipped_white: f32,
    /// How many pixels fall in each of 64 equal steps from black to white.
    pub histogram: Vec<u32>,
}

/// A finished field: the raw noise, the shaped result, and its numbers.
pub struct Made {
    pub raw: Field,
    pub shaped: Field,
    pub stats: Stats,
}

/// Makes the field. Returns `None` if `stop` was set part-way (a newer version is wanted).
pub fn make(settings: &PerlinSettings, seed: u64, size_blocks: f32, blocks_per_pixel: f32, stop: &AtomicBool) -> Option<Made> {
    let mut raw = Field::new(size_blocks, blocks_per_pixel);
    raw.fill(|north, east| settings.raw(seed, north, east));
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    let (low, high) = raw.values.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let span = (high - low).max(1e-6);

    let mut shaped = raw.clone();
    let mut histogram = vec![0u32; 64];
    let (mut sum, mut black, mut white) = (0.0f64, 0usize, 0usize);
    for value in &mut shaped.values {
        let t = if settings.fill_range { (*value - low) / span } else { (*value * 0.5 + 0.5).clamp(0.0, 1.0) };
        *value = settings.shape(t);
        sum += *value as f64;
        black += (*value <= 0.0) as usize;
        white += (*value >= 1.0) as usize;
        histogram[((*value * 64.0) as usize).min(63)] += 1;
    }
    let count = shaped.values.len().max(1) as f32;
    let stats = Stats {
        raw_low: low,
        raw_high: high,
        mean: (sum / count as f64) as f32,
        clipped_black: black as f32 / count,
        clipped_white: white as f32 / count,
        histogram,
    };
    Some(Made { raw, shaped, stats })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let settings = PerlinSettings { octaves: 3, fade: Fade::Soft, invert: true, ..PerlinSettings::default() };
        let text = toml::to_string(&settings).unwrap();
        assert_eq!(toml::from_str::<PerlinSettings>(&text).unwrap(), settings);
    }
}

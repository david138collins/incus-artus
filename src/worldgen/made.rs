//! What every kind of layer makes, whatever it's made from: values from 0 to 1 for each pixel,
//! which pixels it covers, its numbers (range, average, histogram), and the shaping that turned
//! its raw values into those 0-to-1 values.

use super::field::Field;
use serde::{Deserialize, Serialize};

/// Shaping the values without changing the pattern underneath. Shared by every kind of layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shaping {
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

impl Default for Shaping {
    fn default() -> Self {
        Self { fill_range: true, contrast: 1.0, brightness: 0.0, curve: 1.0, invert: false }
    }
}

impl Shaping {
    /// Contrast, brightness, curve and invert, on a value already between 0 and 1.
    pub fn shape(&self, t: f32) -> f32 {
        let t = ((t - 0.5) * self.contrast + 0.5 + self.brightness).clamp(0.0, 1.0);
        let t = t.powf(self.curve.max(0.01));
        if self.invert { 1.0 - t } else { t }
    }
}

/// Numbers about a finished layer, for judging it at a glance (covered pixels only).
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
    pub covered: usize,
}

/// A finished layer.
pub struct Made {
    /// The values before shaping (Perlin: about −1 to 1; images: brightness 0 to 1).
    pub raw: Field,
    /// The values after shaping, 0 to 1.
    pub shaped: Field,
    /// Which pixels the layer covers. `None` = all of them. Elsewhere the layer below shows.
    pub covered: Option<Vec<bool>>,
    /// An image's own colours, one per pixel (kept so it can be shown as it was).
    pub original: Option<Vec<[u8; 3]>>,
    /// The covered shaped values in order, lowest first (for spreading a palette in equal shares).
    pub sorted: Vec<f32>,
    pub stats: Stats,
}

impl Made {
    /// `raw_to_unit` turns a raw value into 0–1 when "fill black to white" is off.
    pub fn finish(
        raw: Field,
        covered: Option<Vec<bool>>,
        original: Option<Vec<[u8; 3]>>,
        shaping: &Shaping,
        raw_to_unit: impl Fn(f32) -> f32,
    ) -> Self {
        let is_covered = |i: usize| covered.as_ref().is_none_or(|c| c[i]);
        let (mut low, mut high) = (f32::MAX, f32::MIN);
        for (i, &v) in raw.values.iter().enumerate() {
            if is_covered(i) {
                (low, high) = (low.min(v), high.max(v));
            }
        }
        if low > high {
            (low, high) = (0.0, 0.0);
        }
        let span = (high - low).max(1e-6);

        let mut shaped = raw.clone();
        let mut histogram = vec![0u32; 64];
        let (mut sum, mut black, mut white, mut count) = (0.0f64, 0usize, 0usize, 0usize);
        let mut sorted = Vec::with_capacity(shaped.values.len());
        for (i, value) in shaped.values.iter_mut().enumerate() {
            let t = if shaping.fill_range { (*value - low) / span } else { raw_to_unit(*value).clamp(0.0, 1.0) };
            *value = shaping.shape(t);
            if is_covered(i) {
                sum += *value as f64;
                black += (*value <= 0.0) as usize;
                white += (*value >= 1.0) as usize;
                count += 1;
                histogram[((*value * 64.0) as usize).min(63)] += 1;
                sorted.push(*value);
            }
        }
        sorted.sort_unstable_by(f32::total_cmp);
        let share = |n: usize| n as f32 / count.max(1) as f32;
        let stats = Stats {
            raw_low: low,
            raw_high: high,
            mean: (sum / count.max(1) as f64) as f32,
            clipped_black: share(black),
            clipped_white: share(white),
            histogram,
            covered: count,
        };
        Made { raw, shaped, covered, original, sorted, stats }
    }

    pub fn covers(&self, i: usize) -> bool {
        self.covered.as_ref().is_none_or(|c| c[i])
    }
}

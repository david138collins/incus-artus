//! The recipe in whole numbers (the regroup, phase 2.2): the same rules as `interpreter.rs`, `carving.rs`,
//! mineralogy and the cave carve, worked in the whole-number kit (`crate::whole`), so the processor and
//! the card make the same world bit for bit [David 2026-11-05: "move to whole numbers. otherwise the
//! hologram breaks. as long as the behavior is qualitatively identical in its procession, it doesnt
//! matter if worlds come out different, so long as the rules never changed"].
//!
//! Everything a fraction is still allowed for (the settings, the maps made at world creation) is turned
//! into whole numbers once, here, as a world loads, and those same numbers go to the card. Reading the
//! recipe at a spot is whole numbers only, in the kit's steps, in the order the card's copy does them.
//!
//! Built a piece at a time beside the fractions, which stay until David has compared them.

use crate::whole::{self, Fix, Pos};
use crate::worldgen::Shaping;
use crate::worldgen::noise::{Fade, PerlinSettings, noise_key, whole_perlin_at};

/// One stretch of a curve, between two dots: where it runs, and its Hermite pieces made ready.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stretch {
    pub x0: Fix,
    pub x1: Fix,
    pub y0: Fix,
    pub y1: Fix,
    /// How wide it is, and its steepness at each end times that.
    pub h: Fix,
    pub hm0: Fix,
    pub hm1: Fix,
    /// The bow's power (`interpreter::bowed`), if it's bowed.
    pub power: Option<Fix>,
}

/// A curve (`interpreter::Curve`), read in whole numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WholeCurve {
    pub stretches: Vec<Stretch>,
    /// A curve of one dot is that dot's height everywhere.
    pub single: Fix,
}

impl WholeCurve {
    /// From a curve's dots, their steepness and bends (worked out once, from the settings).
    pub fn new(dots: &[(f32, f32)], slopes: &[f32], bends: &[f32]) -> Self {
        let single = dots.first().map_or(Fix::ZERO, |dot| Fix::from_f32(dot.1));
        let stretches = (0..dots.len().saturating_sub(1))
            .map(|k| {
                let ((x0, y0), (x1, y1)) = (dots[k], dots[k + 1]);
                let h = (x1 - x0).max(1e-6);
                let bend = bends.get(k).copied().unwrap_or(0.0).clamp(-1.0, 1.0) * (y1 - y0).signum();
                // 8 to the power −bend, as 2 to the power −3 × bend, in the kit (no chip's own powf).
                let power = (bend != 0.0).then(|| whole::exp2(Fix::from_f32(-3.0 * bend)));
                Stretch { x0: Fix::from_f32(x0), x1: Fix::from_f32(x1), y0: Fix::from_f32(y0), y1: Fix::from_f32(y1), h: Fix::from_f32(h), hm0: Fix::from_f32(h * slopes[k]), hm1: Fix::from_f32(h * slopes[k + 1]), power }
            })
            .collect();
        Self { stretches, single }
    }

    pub fn at(&self, x: Fix) -> Fix {
        let Some(last) = self.stretches.last() else { return self.single };
        let x = x.clamp(self.stretches[0].x0, last.x1);
        let s = self.stretches.iter().find(|s| x <= s.x1).unwrap_or(last);
        let mut t = ((x - s.x0) / s.h).clamp(Fix::ZERO, Fix::ONE);
        if let Some(power) = s.power {
            t = whole::pow(t, power);
        }
        let t2 = t * t;
        let t3 = t2 * t;
        // Hermite: the two dots, and the steepness at each.
        let h00 = Fix(2 * t3.0 - 3 * t2.0 + whole::ONE);
        let h10 = Fix(t3.0 - 2 * t2.0 + t.0);
        let h01 = Fix(3 * t2.0 - 2 * t3.0);
        let h11 = t3 - t2;
        h00 * s.y0 + h10 * s.hm0 + h01 * s.y1 + h11 * s.hm1
    }
}

/// A noise layer's shaping (`Shaping::shape`: contrast, brightness, curve, invert), in whole numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WholeShaping {
    pub contrast: Fix,
    pub brightness: Fix,
    /// The curve's power, if it isn't 1.
    pub power: Option<Fix>,
    pub invert: bool,
}

impl WholeShaping {
    pub fn new(shaping: &Shaping) -> Self {
        let curve = shaping.curve.max(0.01);
        Self { contrast: Fix::from_f32(shaping.contrast), brightness: Fix::from_f32(shaping.brightness), power: (curve != 1.0).then(|| Fix::from_f32(curve)), invert: shaping.invert }
    }

    /// On a value already from 0 to 1.
    pub fn shape(&self, t: Fix) -> Fix {
        let half = Fix(whole::ONE / 2);
        let mut t = ((t - half) * self.contrast + half + self.brightness).clamp(Fix::ZERO, Fix::ONE);
        if let Some(power) = self.power {
            t = whole::pow(t, power);
        }
        if self.invert { Fix::ONE - t } else { t }
    }
}

/// One octave of a noise layer: its key, its feature size, and its share of the total.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Octave {
    pub key: u32,
    pub size: Pos,
    pub weight: Fix,
}

/// A noise layer (`PerlinSettings::raw_in`, and `interpreter::noise_at` with its shaping), read in whole
/// numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct WholeNoise {
    pub offset_north: Pos,
    pub offset_east: Pos,
    /// The stretch: its direction, and one over how much.
    pub cos: Fix,
    pub sin: Fix,
    pub over_stretch: Fix,
    /// Each octave turned this much from the last.
    pub turn_cos: Fix,
    pub turn_sin: Fix,
    pub octaves: Vec<Octave>,
    /// Only this octave (counting from 0), for looking at one alone.
    pub solo: Option<usize>,
    pub fade: Fade,
    pub shaping: WholeShaping,
}

/// An angle in degrees, in the kit's 65536ths of a turn (a division, which a processor rounds exactly).
fn turns(degrees: f32) -> i32 {
    Fix::from_f32(degrees / 360.0).0
}

impl WholeNoise {
    pub fn new(noise: &PerlinSettings, seed: u64) -> Self {
        let octaves = noise.octaves.max(1);
        let (mut strength, mut strengths, mut size) = (1.0f32, 0.0f32, noise.scale.max(1.0));
        let mut made = Vec::new();
        for octave in 1..=octaves {
            made.push((noise_key(seed, octave as u64), size, strength));
            strengths += strength;
            strength *= noise.persistence;
            size /= noise.lacunarity.max(1.01);
        }
        let (angle, turn) = (turns(noise.stretch_angle), turns(noise.octave_turn));
        Self {
            offset_north: Pos::from_f32(noise.offset_north),
            offset_east: Pos::from_f32(noise.offset_east),
            cos: whole::cos(angle),
            sin: whole::sin(angle),
            over_stretch: Fix::from_f32(1.0 / noise.stretch.max(0.01)),
            turn_cos: whole::cos(turn),
            turn_sin: whole::sin(turn),
            octaves: made.into_iter().map(|(key, size, strength)| Octave { key, size: Pos::from_f32(size), weight: Fix::from_f32(strength / strengths) }).collect(),
            solo: (noise.solo_octave >= 1 && noise.solo_octave <= octaves).then(|| noise.solo_octave as usize - 1),
            fade: noise.fade,
            shaping: WholeShaping::new(&noise.output),
        }
    }

    /// The octaves summed at a spot, about −1 to 1 (`PerlinSettings::raw_in`).
    pub fn raw(&self, north: Pos, east: Pos) -> Fix {
        let (x, y) = (east + self.offset_east, north + self.offset_north);
        // Stretch: turned into the stretch's direction, then squashed along it.
        let (mut x, mut y) = ((x.scaled(self.cos) + y.scaled(self.sin)).scaled(self.over_stretch), y.scaled(self.cos) - x.scaled(self.sin));
        let mut total = Fix::ZERO;
        for (k, octave) in self.octaves.iter().enumerate() {
            let value = whole_perlin_at(octave.key, self.fade, x.per(octave.size), y.per(octave.size));
            if self.solo == Some(k) {
                return value;
            }
            total = total + value * octave.weight;
            (x, y) = (x.scaled(self.turn_cos) - y.scaled(self.turn_sin), x.scaled(self.turn_sin) + y.scaled(self.turn_cos));
        }
        total
    }

    /// Shaped (`interpreter::noise_at`), about −1 to 1.
    pub fn at(&self, north: Pos, east: Pos) -> Fix {
        let half = Fix(whole::ONE / 2);
        let shaped = self.shaping.shape(self.raw(north, east) * half + half);
        Fix(shaped.0.wrapping_mul(2)) - Fix::ONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Spline;
    use crate::worldgen::noise::NoiseMaths;

    #[test]
    fn a_curve_in_whole_numbers_follows_the_curve() {
        let mut spline = Spline::through(&[(0.0, 0.0), (0.3, 0.6), (0.7, 0.65), (1.0, 1.0)]);
        spline.bends = vec![0.0, 0.5, -0.4];
        let (curve, whole) = (spline.curve(), spline.curve().whole());
        for k in 0..=200 {
            let x = k as f32 / 200.0;
            let (want, got) = (curve.at(x), whole.at(Fix::from_f32(x)).to_f32());
            assert!((want - got).abs() < 0.002, "at {x}: {got} against {want}");
        }
    }

    #[test]
    fn a_noise_layer_in_whole_numbers_follows_the_layer() {
        let mut noise = PerlinSettings { scale: 900.0, octaves: 4, stretch: 2.0, stretch_angle: 30.0, octave_turn: 17.0, offset_north: 50.0, ..PerlinSettings::default() };
        noise.output.contrast = 1.3;
        noise.output.curve = 1.4;
        let whole = WholeNoise::new(&noise, 99);
        let (mut worst, mut total) = (0.0f32, 0.0f32);
        for k in 0..400 {
            // Column middles across a 72,000-block world.
            let (north, east) = ((k * 7919 % 72_000) as f32 - 36_000.0 + 0.5, (k * 104_729 % 72_000) as f32 - 36_000.0 + 0.5);
            let want = noise.output.shape(noise.raw_in(NoiseMaths::WholeNumbers, 99, north, east) * 0.5 + 0.5) * 2.0 - 1.0;
            let got = whole.at(Pos::from_f32(north), Pos::from_f32(east)).to_f32();
            worst = worst.max((want - got).abs());
            total += (want - got).abs();
        }
        // The same field: the same posts and arrows, only the steps rounded differently on the way. Far
        // out, the turns (the sine and cosine tables, good to about 1/50,000) put a spot up to a block or
        // so from where fractions do, which shows where the field is steep.
        assert!(worst < 0.02 && total / 400.0 < 0.002, "the whole-number layer strays {worst} at worst, {} on average, from the fractions'", total / 400.0);
    }
}

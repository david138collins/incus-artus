//! Noise — smooth randomness, the raw clay most generators start from.
//!
//! `smooth` is gradient noise ("Perlin-style"): hills and hollows about one unit apart, from −1
//! to 1. `layered` stacks several copies, each finer and fainter than the last ("octaves"), which
//! gives coastlines and ridges detail at every scale, like real landscapes.
//!
//! The randomness comes from Fate's `planet_hash`, so the same seed and salt always give the same
//! hills in the same places.

use crate::fate::planet_hash;

/// The settings for layered noise, shared by any step that uses it.
#[derive(Clone, Copy, Debug)]
pub struct Layered {
    /// How far apart the largest hills are, in blocks.
    pub scale: f32,
    /// How many layers of finer detail.
    pub octaves: u32,
    /// How strong each finer layer is compared with the one before (0.5 = half).
    pub roughness: f32,
}

/// Gradient noise at a point, from about −1 to 1. One "hill" per unit.
pub fn smooth(seed: u64, salt: u64, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (ix, iy) = (x0 as i64, y0 as i64);
    // Each lattice corner gets a random direction; a point's value is how far it lies along
    // each corner's direction, blended smoothly between the four corners.
    let corner = |cx: i64, cy: i64, dx: f32, dy: f32| {
        let angle = (planet_hash(seed, salt, cx, cy, 0) >> 40) as f32 / (1u64 << 24) as f32 * std::f32::consts::TAU;
        angle.cos() * dx + angle.sin() * dy
    };
    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (u, v) = (fade(fx), fade(fy));
    let a = corner(ix, iy, fx, fy);
    let b = corner(ix + 1, iy, fx - 1.0, fy);
    let c = corner(ix, iy + 1, fx, fy - 1.0);
    let d = corner(ix + 1, iy + 1, fx - 1.0, fy - 1.0);
    let top = a + (b - a) * u;
    let bottom = c + (d - c) * u;
    // Gradient noise peaks at about ±0.7; stretch it to about ±1.
    (top + (bottom - top) * v) * 1.4
}

/// Several octaves of `smooth` stacked up, still from about −1 to 1. `x` and `y` are in blocks.
pub fn layered(seed: u64, salt: u64, settings: Layered, x: f32, y: f32) -> f32 {
    let (mut total, mut strength, mut size, mut strengths) = (0.0, 1.0, settings.scale.max(1.0), 0.0);
    for octave in 0..settings.octaves.max(1) {
        total += smooth(seed, salt.wrapping_add(octave as u64), x / size, y / size) * strength;
        strengths += strength;
        strength *= settings.roughness;
        size /= 2.0;
    }
    total / strengths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_noise() {
        assert_eq!(smooth(5, 1, 3.3, 7.7), smooth(5, 1, 3.3, 7.7));
        assert_ne!(smooth(5, 1, 3.3, 7.7), smooth(6, 1, 3.3, 7.7));
    }

    #[test]
    fn noise_stays_roughly_between_minus_one_and_one() {
        let settings = Layered { scale: 10.0, octaves: 5, roughness: 0.5 };
        let (mut low, mut high) = (f32::MAX, f32::MIN);
        for i in 0..20_000 {
            let value = layered(9, 3, settings, (i % 200) as f32 * 0.37, (i / 200) as f32 * 0.37);
            low = low.min(value);
            high = high.max(value);
        }
        assert!(low > -1.3 && high < 1.3, "{low}..{high}");
        assert!(low < -0.4 && high > 0.4, "it should actually use the range: {low}..{high}");
    }

    #[test]
    fn noise_is_smooth() {
        let step = smooth(1, 2, 0.501, 0.5) - smooth(1, 2, 0.5, 0.5);
        assert!(step.abs() < 0.01);
    }
}

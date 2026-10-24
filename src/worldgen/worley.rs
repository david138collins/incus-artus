//! Worley noise (also called cellular or Voronoi noise) — the second kind of noise in the suite.
//!
//! **How it works.** Scatter points across the world: one in each square of a grid, nudged a
//! random amount inside its square (the **jitter**: 0 = a neat grid, 1 = anywhere in the square).
//! Every spot in the world then belongs to its **nearest point**. That splits the world into cells,
//! one around each point: the Voronoi diagram.
//!
//! **What the value is** (the **feature**):
//! - **nearest:** how far it is to the nearest point (F1). Dark in the middle of each cell,
//!   bright at the edges: bubbles, scales, pebbles.
//! - **second nearest:** how far it is to the second-nearest point (F2). Softer, puffier cells.
//! - **edges:** second nearest minus nearest (F2 − F1). Zero exactly on the line between two cells:
//!   cracks, dried mud, crazing, a web of borders.
//! - **cells:** a random value for each cell, flat all over it. The classic Voronoi look: patches,
//!   a map of territories.
//!
//! **Distance** can be measured three ways: straight line (round cells), along the grid (diamond-
//! shaped cells, like a city's blocks), or by the longer side (square cells).
//!
//! Octaves stack finer and fainter copies, as with Perlin noise, and the output is shaped the same
//! way. The randomness comes from Fate's `planet_hash`.

use super::field::Field;
use super::made::{Made, Shaping};
use crate::fate::planet_hash;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// What a Worley field shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Feature {
    /// Distance to the nearest point (F1).
    #[default]
    Nearest,
    /// Distance to the second-nearest point (F2).
    Second,
    /// F2 − F1: zero on the borders between cells.
    Edges,
    /// One random value per cell.
    Cells,
}

impl Feature {
    pub const ALL: [Feature; 4] = [Feature::Nearest, Feature::Second, Feature::Edges, Feature::Cells];

    pub fn name(self) -> &'static str {
        match self {
            Feature::Nearest => "nearest",
            Feature::Second => "second nearest",
            Feature::Edges => "edges",
            Feature::Cells => "cells",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Feature::Nearest => "How far to the nearest point. Dark cell middles, bright edges: bubbles, scales, pebbles.",
            Feature::Second => "How far to the second-nearest point. Softer, puffier cells.",
            Feature::Edges => "Second nearest minus nearest. Black exactly on the borders between cells: cracks, dried mud, a web of borders.",
            Feature::Cells => "A random value for each cell, flat all over it. Patches, territories: the classic Voronoi look.",
        }
    }
}

/// How distance is measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Distance {
    /// Straight line: round cells.
    #[default]
    Straight,
    /// Along the grid (east–west plus north–south): diamond-shaped cells.
    Grid,
    /// The longer of east–west and north–south: square cells.
    Longest,
}

impl Distance {
    pub const ALL: [Distance; 3] = [Distance::Straight, Distance::Grid, Distance::Longest];

    pub fn name(self) -> &'static str {
        match self {
            Distance::Straight => "straight (round)",
            Distance::Grid => "grid (diamond)",
            Distance::Longest => "longest side (square)",
        }
    }

    fn measure(self, dx: f32, dy: f32) -> f32 {
        match self {
            Distance::Straight => (dx * dx + dy * dy).sqrt(),
            Distance::Grid => dx.abs() + dy.abs(),
            Distance::Longest => dx.abs().max(dy.abs()),
        }
    }
}

/// Everything that makes one kind of Worley field. This is what a Worley preset saves (not the
/// seed, so a preset is a *kind* of noise that works with any seed).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorleySettings {
    // ── Cells ──
    /// About how far apart the points are, in blocks (one grid square of the first octave).
    pub scale: f32,
    /// How far each point may wander inside its grid square: 0 = a neat grid, 1 = anywhere.
    pub jitter: f32,
    pub feature: Feature,
    pub distance: Distance,

    // ── Octaves ──
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,

    // ── Placement ──
    pub offset_north: f32,
    pub offset_east: f32,
    pub stretch: f32,
    pub stretch_angle: f32,
    pub octave_turn: f32,

    // ── Output ──
    #[serde(flatten)]
    pub output: Shaping,
}

impl Default for WorleySettings {
    fn default() -> Self {
        Self {
            scale: 6_000.0,
            jitter: 1.0,
            feature: Feature::Nearest,
            distance: Distance::Straight,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
            offset_north: 0.0,
            offset_east: 0.0,
            stretch: 1.0,
            stretch_angle: 0.0,
            octave_turn: 0.0,
            output: Shaping::default(),
        }
    }
}

/// A random number from 0 to 1 for grid square (i, j).
fn unit(seed: u64, salt: u64, i: i64, j: i64, which: i64) -> f32 {
    (planet_hash(seed, salt, i, j, which) >> 40) as f32 / (1u64 << 24) as f32
}

/// One octave of Worley noise at a point, one cell per unit. About 0 to 1 for most features.
pub fn worley(seed: u64, salt: u64, jitter: f32, feature: Feature, distance: Distance, x: f32, y: f32) -> f32 {
    let (cx, cy) = (x.floor() as i64, y.floor() as i64);
    let jitter = jitter.clamp(0.0, 1.0);
    // The nearest and second-nearest points, and the nearest one's cell.
    let (mut first, mut second, mut cell) = (f32::MAX, f32::MAX, (cx, cy));
    // Points stay inside their own grid square, so the two nearest are always within two
    // squares of this one.
    for j in cy - 2..=cy + 2 {
        for i in cx - 2..=cx + 2 {
            let px = i as f32 + 0.5 + (unit(seed, salt, i, j, 0) - 0.5) * jitter;
            let py = j as f32 + 0.5 + (unit(seed, salt, i, j, 1) - 0.5) * jitter;
            let d = distance.measure(px - x, py - y);
            if d < first {
                (second, first, cell) = (first, d, (i, j));
            } else if d < second {
                second = d;
            }
        }
    }
    match feature {
        Feature::Nearest => first,
        Feature::Second => second,
        Feature::Edges => second - first,
        Feature::Cells => unit(seed, salt, cell.0, cell.1, 2),
    }
}

impl WorleySettings {
    /// The raw stacked value at a position in blocks, before the output shaping.
    pub fn raw(&self, seed: u64, north: f32, east: f32) -> f32 {
        let (mut x, mut y) = (east + self.offset_east, north + self.offset_north);
        let (sin, cos) = self.stretch_angle.to_radians().sin_cos();
        (x, y) = ((x * cos + y * sin) / self.stretch.max(0.01), -x * sin + y * cos);

        let (turn_sin, turn_cos) = self.octave_turn.to_radians().sin_cos();
        let (mut total, mut strength, mut strengths) = (0.0, 1.0, 0.0);
        let mut size = self.scale.max(1.0);
        for octave in 1..=self.octaves.max(1) {
            total += worley(seed, octave as u64, self.jitter, self.feature, self.distance, x / size, y / size) * strength;
            strengths += strength;
            strength *= self.persistence;
            size /= self.lacunarity.max(1.01);
            (x, y) = (x * turn_cos - y * turn_sin, x * turn_sin + y * turn_cos);
        }
        total / strengths
    }
}

/// Makes the field. Returns `None` if `stop` was set part-way (a newer version is wanted).
pub fn make(settings: &WorleySettings, seed: u64, size_blocks: f32, blocks_per_pixel: f32, stop: &AtomicBool) -> Option<Made> {
    let mut raw = Field::new(size_blocks, blocks_per_pixel);
    raw.fill(|north, east| settings.raw(seed, north, east));
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    // Unfilled, raw values are used as they are (0 = black, 1 = white).
    Some(Made::finish(raw, None, None, &settings.output, |v| v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_cells() {
        let f = |seed| worley(seed, 1, 1.0, Feature::Nearest, Distance::Straight, 3.3, 7.7);
        assert_eq!(f(5), f(5));
        assert_ne!(f(5), f(6));
    }

    #[test]
    fn with_no_jitter_the_points_sit_in_the_middle_of_each_square() {
        let at_point = worley(1, 1, 0.0, Feature::Nearest, Distance::Straight, 4.5, 2.5);
        assert!(at_point.abs() < 1e-6);
        let at_corner = worley(1, 1, 0.0, Feature::Nearest, Distance::Straight, 4.0, 2.0);
        assert!((at_corner - 0.5f32.hypot(0.5)).abs() < 1e-5);
    }

    #[test]
    fn edges_are_never_negative_and_cells_are_flat() {
        for i in 0..2_000 {
            let (x, y) = ((i % 50) as f32 * 0.13, (i / 50) as f32 * 0.17);
            assert!(worley(3, 1, 1.0, Feature::Edges, Distance::Straight, x, y) >= 0.0);
        }
        // Next to its point, a cell's value is the same as at the point.
        let a = worley(3, 1, 0.0, Feature::Cells, Distance::Straight, 2.5, 2.5);
        let b = worley(3, 1, 0.0, Feature::Cells, Distance::Straight, 2.6, 2.4);
        assert_eq!(a, b);
    }

    #[test]
    fn a_worley_field_fills_black_to_white() {
        let made = make(&WorleySettings::default(), 1, 72_000.0, 512.0, &AtomicBool::new(false)).unwrap();
        let (low, high) = made.shaped.values.iter().fold((1.0f32, 0.0f32), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        assert_eq!((low, high), (0.0, 1.0));
    }

    #[test]
    fn a_preset_survives_being_written_and_read() {
        let settings = WorleySettings { feature: Feature::Edges, distance: Distance::Grid, jitter: 0.6, ..WorleySettings::default() };
        let text = toml::to_string(&settings).unwrap();
        assert_eq!(toml::from_str::<WorleySettings>(&text).unwrap(), settings);
    }
}

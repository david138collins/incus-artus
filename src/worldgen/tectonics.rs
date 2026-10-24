//! Tectonics — World Gen tool 2: the plates move, collide, and wrinkle like rags on a table.
//!
//! **Step 1 (this much is built): what each plate is doing, and what that would do at the
//! boundaries.** Nothing moves yet; this is for judging whether a setup is interesting before the
//! simulation runs.
//!
//! **Each plate's motion**, random from the seed:
//! - a **speed** and **direction**, in blocks per step;
//! - a **spin**, in degrees per step (+ = clockwise as seen on the map).
//!
//! A spinning plate moves differently at different places: its far side swings one way and its
//! near side the other. So the speed of a plate *at a spot* is its own speed plus the swing from
//! its spin there.
//!
//! **Boundaries.** Wherever two plates touch, compare how each is moving at that spot:
//! - **colliding** (convergent): moving toward each other → mountains, later;
//! - **pulling apart** (divergent): moving away → rifts, and in the ocean, V-shaped valleys;
//! - **sliding** (transform): moving past each other → friction, which grips and turns them.
//!
//! Most boundaries do a bit of everything; each spot is named after what it does most of. "Toward"
//! is measured across the line between the two plates' middles.
//!
//! The plates then move in the simulation (`rags.rs`), starting with these motions.

use super::plates::Plates;
use crate::fate::planet_hash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MotionSettings {
    /// Each plate's speed is picked between these, in blocks per step.
    pub slowest: f32,
    pub fastest: f32,
    /// Ocean plates' speed is multiplied by this (real ones are often faster).
    pub ocean_speed: f32,
    /// The fastest spin either way, in degrees per step.
    pub spin: f32,
    /// How much the directions lean toward the middle of the world: 0 = any way at all, 1 = all
    /// straight at the middle (lots of collisions).
    pub inward: f32,
}

impl Default for MotionSettings {
    fn default() -> Self {
        Self { slowest: 20.0, fastest: 60.0, ocean_speed: 1.0, spin: 0.1, inward: 0.3 }
    }
}

/// How one plate moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// Blocks per step, east and south (the way the field's pixels count).
    pub velocity: (f32, f32),
    /// Radians per step, + = clockwise on the map.
    pub spin: f32,
}

impl Motion {
    /// How fast the plate moves at `at` (field pixels) when its middle is at `middle`, in blocks
    /// per step: its own speed plus the swing from its spin.
    pub fn at(&self, middle: (f32, f32), at: (f32, f32), blocks_per_pixel: f32) -> (f32, f32) {
        let (rx, ry) = ((at.0 - middle.0) * blocks_per_pixel, (at.1 - middle.1) * blocks_per_pixel);
        // South is down, so turning (east → south) is clockwise on the map.
        (self.velocity.0 - self.spin * ry, self.velocity.1 + self.spin * rx)
    }

    /// Blocks per step.
    pub fn speed(&self) -> f32 {
        self.velocity.0.hypot(self.velocity.1)
    }

    /// Which way it's heading, in degrees clockwise from north.
    pub fn heading(&self) -> f32 {
        self.velocity.0.atan2(-self.velocity.1).to_degrees().rem_euclid(360.0)
    }
}

/// A random number from 0 to 1.
fn unit(seed: u64, plate: usize, which: i64) -> f32 {
    (planet_hash(seed, 0x7ec7_0215, plate as i64, which, 0) >> 40) as f32 / (1u64 << 24) as f32
}

/// Every plate's motion, random from `seed`.
pub fn motions(settings: &MotionSettings, seed: u64, plates: &Plates) -> Vec<Motion> {
    let centre = plates.width as f32 / 2.0;
    plates
        .kept
        .iter()
        .enumerate()
        .map(|(p, plate)| {
            let angle = unit(seed, p, 0) * std::f32::consts::TAU;
            let random = (angle.cos(), angle.sin());
            // Lean toward the middle (not at all for a plate sitting right on it).
            let (ix, iy) = (centre - plate.middle.0, centre - plate.middle.1);
            let length = ix.hypot(iy);
            let inward = if length > 1e-3 { (ix / length, iy / length) } else { random };
            let lean = settings.inward.clamp(0.0, 1.0);
            let (dx, dy) = (random.0 * (1.0 - lean) + inward.0 * lean, random.1 * (1.0 - lean) + inward.1 * lean);
            let length = dx.hypot(dy);
            let direction = if length > 1e-3 { (dx / length, dy / length) } else { inward };

            let (slowest, fastest) = (settings.slowest.min(settings.fastest), settings.slowest.max(settings.fastest));
            let mut speed = slowest + unit(seed, p, 1) * (fastest - slowest);
            if !plate.land {
                speed *= settings.ocean_speed;
            }
            let spin = (unit(seed, p, 2) * 2.0 - 1.0) * settings.spin.to_radians();
            Motion { velocity: (direction.0 * speed, direction.1 * speed), spin }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    Colliding,
    PullingApart,
    Sliding,
}

/// What's happening at one spot on a boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    pub kind: Boundary,
    /// The two plates.
    pub plates: (u8, u8),
    /// How fast they're closing (+) or opening (−), and sliding past, in blocks per step.
    pub closing: f32,
    pub sliding: f32,
}

/// What's happening at each pixel on a boundary between two plates. (Plates against empty ocean
/// aren't boundaries.)
pub fn boundaries(plates: &Plates, motions: &[Motion]) -> Vec<Option<Contact>> {
    let (width, blocks_per_pixel) = (plates.width, plates.blocks_per_pixel);
    let mut contacts = vec![None; width * width];
    for row in 0..width {
        for col in 0..width {
            let i = row * width + col;
            let Some(a) = plates.plate[i] else { continue };
            for (c, r) in [(col + 1, row), (col, row + 1)] {
                if c >= width || r >= width {
                    continue;
                }
                let j = r * width + c;
                let Some(b) = plates.plate[j].filter(|&b| b != a) else { continue };
                let (pa, pb) = (&plates.kept[a as usize], &plates.kept[b as usize]);
                // Across: from a's middle toward b's.
                let (nx, ny) = (pb.middle.0 - pa.middle.0, pb.middle.1 - pa.middle.1);
                let length = nx.hypot(ny).max(1e-6);
                let (nx, ny) = (nx / length, ny / length);
                // Between the two pixels' centres.
                let at = ((col + c) as f32 / 2.0 + 0.5, (row + r) as f32 / 2.0 + 0.5);
                let va = motions[a as usize].at(pa.middle, at, blocks_per_pixel);
                let vb = motions[b as usize].at(pb.middle, at, blocks_per_pixel);
                // How a moves, seen from b.
                let (rx, ry) = (va.0 - vb.0, va.1 - vb.1);
                let closing = rx * nx + ry * ny;
                let sliding = (rx * ny - ry * nx).abs();
                let kind = if closing.abs() < sliding {
                    Boundary::Sliding
                } else if closing > 0.0 {
                    Boundary::Colliding
                } else {
                    Boundary::PullingApart
                };
                let contact = Some(Contact { kind, plates: (a.min(b), a.max(b)), closing, sliding });
                contacts[i] = contact;
                contacts[j] = contact;
            }
        }
    }
    contacts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::plates::{self, Plate, PlateSettings};
    use std::sync::atomic::AtomicBool;

    /// Two plates side by side: plate 0 on the left half, plate 1 on the right.
    fn two_plates() -> Plates {
        let width = 8;
        let plate = (0..width * width).map(|i| Some(if i % width < width / 2 { 0 } else { 1 })).collect();
        let half = |middle| Plate { cell: 0, land: true, pixels: 32, share: 0.5, middle };
        Plates {
            width,
            blocks_per_pixel: 100.0,
            vertices: Vec::new(),
            edges: Vec::new(),
            cell: vec![0; width * width],
            plate,
            inside: vec![true; width * width],
            kept: vec![half((2.0, 4.0)), half((6.0, 4.0))],
            cells: 2,
            past_the_edge: 0,
            coverage: 1.0,
        }
    }

    fn kinds(motions: &[Motion]) -> Vec<Boundary> {
        boundaries(&two_plates(), motions).into_iter().flatten().map(|contact| contact.kind).collect()
    }

    #[test]
    fn head_on_collides_apart_pulls_and_past_slides() {
        let moving = |x, y| Motion { velocity: (x, y), spin: 0.0 };
        let all = |kinds: Vec<Boundary>, kind| !kinds.is_empty() && kinds.iter().all(|&k| k == kind);
        assert!(all(kinds(&[moving(10.0, 0.0), moving(-10.0, 0.0)]), Boundary::Colliding));
        assert!(all(kinds(&[moving(-10.0, 0.0), moving(10.0, 0.0)]), Boundary::PullingApart));
        assert!(all(kinds(&[moving(0.0, 10.0), moving(0.0, -10.0)]), Boundary::Sliding));
        // Only the pixels along the line between them are on the boundary.
        assert_eq!(kinds(&[moving(10.0, 0.0), moving(-10.0, 0.0)]).len(), 16);
    }

    #[test]
    fn spin_swings_the_far_side_one_way_and_the_near_side_the_other() {
        let spinning = Motion { velocity: (0.0, 0.0), spin: 0.01 };
        // Clockwise: east of the middle moves south, west of it moves north.
        assert!(spinning.at((0.0, 0.0), (10.0, 0.0), 1.0).1 > 0.0);
        assert!(spinning.at((0.0, 0.0), (-10.0, 0.0), 1.0).1 < 0.0);
        // North of the middle (up the map) moves east.
        assert!(spinning.at((0.0, 0.0), (0.0, -10.0), 1.0).0 > 0.0);
    }

    #[test]
    fn headings_are_clockwise_from_north() {
        let heading = |x, y| Motion { velocity: (x, y), spin: 0.0 }.heading().round();
        assert_eq!(heading(0.0, -1.0), 0.0);
        assert_eq!(heading(1.0, 0.0), 90.0);
        assert_eq!(heading(0.0, 1.0), 180.0);
        assert_eq!(heading(-1.0, 0.0), 270.0);
    }

    #[test]
    fn motions_follow_the_settings_and_the_seed() {
        let made = plates::make(&PlateSettings::default(), 5, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap();
        let settings = MotionSettings { ocean_speed: 1.0, ..MotionSettings::default() };
        let first = motions(&settings, 1, &made);
        assert_eq!(first.len(), made.kept.len());
        assert_eq!(first, motions(&settings, 1, &made), "same seed, same motions");
        assert_ne!(first, motions(&settings, 2, &made));
        for motion in &first {
            assert!((settings.slowest - 0.01..=settings.fastest + 0.01).contains(&motion.speed()));
            assert!(motion.spin.abs() <= settings.spin.to_radians());
        }
        // All the way inward: every plate heads at the middle.
        let inward = motions(&MotionSettings { inward: 1.0, ..settings }, 1, &made);
        let centre = made.width as f32 / 2.0;
        for (motion, plate) in inward.iter().zip(&made.kept) {
            let (ix, iy) = (centre - plate.middle.0, centre - plate.middle.1);
            assert!(motion.velocity.0 * ix + motion.velocity.1 * iy > 0.0);
        }
    }
}

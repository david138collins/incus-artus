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
//! **Step 2 (built): the plates move, as rigid pieces** (`Sim`). Each is given its push at the start
//! and then coasts. Every step:
//! 1. Every plate's pixels are laid onto the map where the plate is now; where two plates land on
//!    the same pixel, they **overlap**.
//! 2. For each pair that overlaps, at the middle of their overlap:
//!    - **push back** shoves them apart;
//!    - **give** soaks up the speed they're meeting with (rock doesn't bounce);
//!    - **friction** drags them along each other. It acts at the contact, off to one side of each
//!      plate's middle, so it turns them: plates grip and rotate one another.
//! 3. **The bowl:** any part of a plate further from the middle of the world than the bowl's start
//!    is pushed back inward, harder the further out it is.
//! 4. **Drag** takes a little of every plate's speed and spin, so they coast to a stop.
//! 5. Crude height, for now: where plates crush into each other, the ground on both is raised (by
//!    how fast they're closing). The height belongs to the plate's pixels, so it travels with them.
//!
//! The steps are always the same size and taken in the same order, so the same seed and settings
//! give the same result, however fast it's played.

use super::plates::Plates;
use crate::fate::planet_hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

/// The simulation's settings. These can be changed while it runs (press Reset to replay from the
/// start with them).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SimSettings {
    /// How much of its speed and spin each plate loses every step (0.01 = 1%).
    pub drag: f32,
    /// How hard overlapping plates are shoved apart, in blocks per step, for an overlap the size
    /// of the smaller plate.
    pub push: f32,
    /// How quickly the speed two plates meet with is soaked up (higher = they stop dead sooner).
    pub give: f32,
    /// How strongly plates sliding past each other grip and drag (and so turn) one another.
    pub friction: f32,
    /// How heavy ocean plates are for their size, compared with land plates.
    pub ocean_weight: f32,
    /// The bowl starts this far out (a share of the world's radius); past it, plates are pushed
    /// back inward.
    pub bowl_start: f32,
    /// How hard the bowl pushes, in blocks per step, for a plate entirely at the world's edge.
    pub bowl: f32,
    /// How much the ground rises where plates crush into each other, per step, for a closing
    /// speed of 100 blocks per step.
    pub uplift: f32,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self { drag: 0.01, push: 15.0, give: 20.0, friction: 10.0, ocean_weight: 0.4, bowl_start: 0.8, bowl: 5.0, uplift: 0.25 }
    }
}

/// One plate in the simulation: a rigid piece made of its pixels.
struct Body {
    land: bool,
    /// Its pixels, relative to its middle as it started (before turning), and the ground's height
    /// on each.
    pixels: Vec<(f32, f32)>,
    height: Vec<f32>,
    /// Its starting shape as a little map, for painting: the pixel at each spot of its box (or
    /// `NONE`).
    corner: (i32, i32),
    size: (usize, usize),
    map: Vec<u32>,
    /// Where its middle started, and the furthest any of its pixels is from it.
    start: (f32, f32),
    reach: f32,
    /// How hard it is to turn, for a weight of 1 per pixel.
    turning: f32,

    /// Where its middle is now and how far it has turned (radians, + = clockwise), in field pixels.
    at: (f32, f32),
    angle: f32,
    /// Pixels per step, and radians per step.
    velocity: (f32, f32),
    spin: f32,
}

const NONE: u32 = u32::MAX;

impl Body {
    /// How heavy it is per pixel.
    fn weight_each(&self, settings: &SimSettings) -> f32 {
        if self.land { 1.0 } else { settings.ocean_weight.max(0.01) }
    }

    fn weight(&self, settings: &SimSettings) -> f32 {
        self.pixels.len() as f32 * self.weight_each(settings)
    }

    fn inertia(&self, settings: &SimSettings) -> f32 {
        (self.turning * self.weight_each(settings)).max(1e-3)
    }

    /// How fast its ground at `p` is moving (pixels per step): its own speed plus the swing from
    /// its spin.
    fn velocity_at(&self, p: (f32, f32)) -> (f32, f32) {
        (self.velocity.0 - self.spin * (p.1 - self.at.1), self.velocity.1 + self.spin * (p.0 - self.at.0))
    }

    /// Gives it a shove `j` (a change of momentum) at `p`: off its middle, a shove also turns it.
    fn shove(&mut self, p: (f32, f32), j: (f32, f32), settings: &SimSettings) {
        let weight = self.weight(settings);
        self.velocity.0 += j.0 / weight;
        self.velocity.1 += j.1 / weight;
        self.spin += cross((p.0 - self.at.0, p.1 - self.at.1), j) / self.inertia(settings);
    }
}

/// The turning part of a push `b` at offset `a`.
fn cross(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.1 - a.1 * b.0
}

/// Pixels overlapping between two plates in one step.
#[derive(Default)]
struct Overlap {
    count: usize,
    sum: (f32, f32),
}

pub struct Sim {
    width: usize,
    blocks_per_pixel: f32,
    bodies: Vec<Body>,
    /// Steps taken so far.
    pub steps: u64,
    /// Whether everything has coasted to a stop.
    pub settled: bool,
    /// Which plate's pixel landed on each map pixel this step (plate × 2²⁴ + pixel).
    owner: Vec<u32>,
}

impl Sim {
    /// The plates as they are, with the motions they're given at the start.
    pub fn new(plates: &Plates, motions: &[Motion]) -> Self {
        let (width, blocks_per_pixel) = (plates.width, plates.blocks_per_pixel);
        let mut pixels: Vec<Vec<(usize, usize)>> = vec![Vec::new(); plates.kept.len()];
        for (i, plate) in plates.plate.iter().enumerate() {
            if let Some(p) = plate {
                pixels[*p as usize].push((i % width, i / width));
            }
        }
        let bodies = plates
            .kept
            .iter()
            .zip(pixels)
            .zip(motions)
            .map(|((plate, own), motion)| {
                let start = plate.middle;
                let (mut low, mut high) = ((usize::MAX, usize::MAX), (0, 0));
                for &(col, row) in &own {
                    low = (low.0.min(col), low.1.min(row));
                    high = (high.0.max(col), high.1.max(row));
                }
                let size = if own.is_empty() { (0, 0) } else { (high.0 - low.0 + 1, high.1 - low.1 + 1) };
                let mut map = vec![NONE; size.0 * size.1];
                let relative: Vec<(f32, f32)> = own.iter().map(|&(col, row)| (col as f32 + 0.5 - start.0, row as f32 + 0.5 - start.1)).collect();
                for (k, &(col, row)) in own.iter().enumerate() {
                    map[(row - low.1) * size.0 + (col - low.0)] = k as u32;
                }
                let reach = relative.iter().map(|r| r.0.hypot(r.1)).fold(0.0, f32::max) + 1.0;
                let turning = relative.iter().map(|r| r.0 * r.0 + r.1 * r.1).sum();
                Body {
                    land: plate.land,
                    height: vec![0.0; relative.len()],
                    pixels: relative,
                    corner: (low.0 as i32, low.1 as i32),
                    size,
                    map,
                    start,
                    reach,
                    turning,
                    at: start,
                    angle: 0.0,
                    velocity: (motion.velocity.0 / blocks_per_pixel, motion.velocity.1 / blocks_per_pixel),
                    spin: motion.spin,
                }
            })
            .collect();
        Self { width, blocks_per_pixel, bodies, steps: 0, settled: false, owner: vec![NONE; width * width] }
    }

    /// How plate `p` is moving now (blocks per step, radians per step).
    pub fn motion(&self, p: usize) -> Motion {
        let body = &self.bodies[p];
        Motion { velocity: (body.velocity.0 * self.blocks_per_pixel, body.velocity.1 * self.blocks_per_pixel), spin: body.spin }
    }

    /// Where plate `p`'s middle is now, in field pixels.
    pub fn middle(&self, p: usize) -> (f32, f32) {
        self.bodies[p].at
    }

    /// One step.
    pub fn step(&mut self, settings: &SimSettings) {
        let width = self.width;
        let field = width as f32;
        let (centre, radius) = (field / 2.0, field / 2.0);
        let bowl_start = radius * settings.bowl_start.clamp(0.0, 0.999);
        let bowl_each = settings.bowl / self.blocks_per_pixel;

        // 1. Lay every plate's pixels onto the map, noting overlaps and the bowl's push.
        self.owner.fill(NONE);
        let mut overlaps: Vec<(usize, usize, usize, usize, (f32, f32))> = Vec::new();
        let mut bowl_pushes: Vec<((f32, f32), f32)> = Vec::with_capacity(self.bodies.len());
        for (p, body) in self.bodies.iter().enumerate() {
            let (cos, sin) = (body.angle.cos(), body.angle.sin());
            let (mut push, mut turn) = ((0.0, 0.0), 0.0);
            // Each pixel's share of the push (a whole plate at the edge gets the full push).
            let per_pixel = bowl_each * body.weight_each(settings);
            for (k, rel) in body.pixels.iter().enumerate() {
                let (x, y) = (body.at.0 + cos * rel.0 - sin * rel.1, body.at.1 + sin * rel.0 + cos * rel.1);
                // The bowl: past its start, pushed back toward the middle.
                let (dx, dy) = (x - centre, y - centre);
                let out = dx.hypot(dy);
                if out > bowl_start && out > 1e-3 {
                    let depth = (out - bowl_start) / (radius - bowl_start);
                    let j = (-dx / out * per_pixel * depth, -dy / out * per_pixel * depth);
                    push = (push.0 + j.0, push.1 + j.1);
                    turn += cross((x - body.at.0, y - body.at.1), j);
                }
                if x < 0.0 || y < 0.0 || x >= field || y >= field {
                    continue;
                }
                let i = y as usize * width + x as usize;
                let there = self.owner[i];
                if there == NONE {
                    self.owner[i] = (p as u32) << 24 | k as u32;
                } else if (there >> 24) as usize != p {
                    overlaps.push(((there >> 24) as usize, (there & 0xFF_FFFF) as usize, p, k, (x, y)));
                }
            }
            bowl_pushes.push((push, turn));
        }
        // The bowl's pushes, added up over each plate's pixels.
        for (body, (push, turn)) in self.bodies.iter_mut().zip(bowl_pushes) {
            let weight = body.weight(settings).max(1e-3);
            body.velocity = (body.velocity.0 + push.0 / weight, body.velocity.1 + push.1 / weight);
            body.spin += turn / body.inertia(settings);
        }

        // 2. Each pair that overlaps: push back, give and friction, at the middle of the overlap.
        let mut pairs: BTreeMap<(usize, usize), Overlap> = BTreeMap::new();
        for &(a, _, b, _, at) in &overlaps {
            let pair = pairs.entry((a, b)).or_default();
            pair.count += 1;
            pair.sum = (pair.sum.0 + at.0, pair.sum.1 + at.1);
        }
        let mut closing_speed: BTreeMap<(usize, usize), f32> = BTreeMap::new();
        for (&(a, b), overlap) in &pairs {
            let c = (overlap.sum.0 / overlap.count as f32, overlap.sum.1 / overlap.count as f32);
            let (body_a, body_b) = (&self.bodies[a], &self.bodies[b]);
            // Across: from b's middle toward a's.
            let (nx, ny) = (body_a.at.0 - body_b.at.0, body_a.at.1 - body_b.at.1);
            let length = nx.hypot(ny);
            let n = if length > 1e-3 { (nx / length, ny / length) } else { (1.0, 0.0) };
            let (va, vb) = (body_a.velocity_at(c), body_b.velocity_at(c));
            let relative = (va.0 - vb.0, va.1 - vb.1);
            let across = relative.0 * n.0 + relative.1 * n.1;
            let along = (relative.0 - across * n.0, relative.1 - across * n.1);
            // How big the overlap is, compared with the smaller plate.
            let share = overlap.count as f32 / body_a.pixels.len().min(body_b.pixels.len()).max(1) as f32;
            // How much shove it takes to change their speed against each other by 1 along `d`
            // (their weights, and how easily the shove turns them instead).
            let (ra, rb) = ((c.0 - body_a.at.0, c.1 - body_a.at.1), (c.0 - body_b.at.0, c.1 - body_b.at.1));
            let heft = |d: (f32, f32)| {
                let ease = 1.0 / body_a.weight(settings)
                    + 1.0 / body_b.weight(settings)
                    + cross(ra, d).powi(2) / body_a.inertia(settings)
                    + cross(rb, d).powi(2) / body_b.inertia(settings);
                1.0 / ease.max(1e-9)
            };
            let mut j = 0.0;
            // Push back, and give (soaking up the speed they meet with).
            j += settings.push / self.blocks_per_pixel * share * heft(n);
            j -= (settings.give * share).min(1.0) * across * heft(n);
            let mut shove = (n.0 * j, n.1 * j);
            // Friction, along the boundary.
            let sliding = along.0.hypot(along.1);
            if sliding > 1e-6 {
                let t = (along.0 / sliding, along.1 / sliding);
                let f = (settings.friction * share).min(1.0) * sliding * heft(t);
                shove = (shove.0 - t.0 * f, shove.1 - t.1 * f);
            }
            self.bodies[a].shove(c, shove, settings);
            self.bodies[b].shove(c, (-shove.0, -shove.1), settings);
            closing_speed.insert((a, b), (-across).max(0.0) * self.blocks_per_pixel);
        }

        // 3. Crude height: where they crush together, both plates' ground rises.
        for &(a, ia, b, ib, _) in &overlaps {
            let rise = settings.uplift * closing_speed[&(a, b)] / 100.0;
            self.bodies[a].height[ia] += rise;
            self.bodies[b].height[ib] += rise;
        }

        // 4. Drag, then move.
        let keep = 1.0 - settings.drag.clamp(0.0, 1.0);
        let mut fastest: f32 = 0.0;
        for body in &mut self.bodies {
            body.velocity = (body.velocity.0 * keep, body.velocity.1 * keep);
            body.spin *= keep;
            body.at = (body.at.0 + body.velocity.0, body.at.1 + body.velocity.1);
            body.angle += body.spin;
            fastest = fastest.max((body.velocity.0.hypot(body.velocity.1) + body.spin.abs() * body.reach) * self.blocks_per_pixel);
        }
        self.steps += 1;
        // Coasted to a stop: nothing moves even 1 block per step.
        self.settled = fastest < 1.0;
    }

    /// The plates where they are now: for each map pixel, the plate on it and the ground's height
    /// there. (Where two overlap, the later plate shows.)
    pub fn paint(&self) -> Vec<Option<(u8, f32)>> {
        let width = self.width;
        let mut painted = vec![None; width * width];
        for (p, body) in self.bodies.iter().enumerate() {
            let (cos, sin) = (body.angle.cos(), body.angle.sin());
            let rows = ((body.at.1 - body.reach).floor().max(0.0) as usize)..((body.at.1 + body.reach).ceil().clamp(0.0, width as f32) as usize);
            let cols = ((body.at.0 - body.reach).floor().max(0.0) as usize)..((body.at.0 + body.reach).ceil().clamp(0.0, width as f32) as usize);
            for row in rows {
                for col in cols.clone() {
                    // Turn back to where this spot was at the start, and look it up there.
                    let (dx, dy) = (col as f32 + 0.5 - body.at.0, row as f32 + 0.5 - body.at.1);
                    let (x, y) = (body.start.0 + cos * dx + sin * dy, body.start.1 - sin * dx + cos * dy);
                    let (c, r) = (x.floor() as i32 - body.corner.0, y.floor() as i32 - body.corner.1);
                    if c < 0 || r < 0 || c as usize >= body.size.0 || r as usize >= body.size.1 {
                        continue;
                    }
                    let k = body.map[r as usize * body.size.0 + c as usize];
                    if k != NONE {
                        painted[row * width + col] = Some((p as u8, body.height[k as usize]));
                    }
                }
            }
        }
        painted
    }
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

    /// Two square plates in a 60-pixel world, 4 pixels apart, moving straight at each other.
    fn head_on() -> (Plates, Vec<Motion>) {
        let width = 60;
        let mut plate = vec![None; width * width];
        for row in 20..40 {
            for col in 8..28 {
                plate[row * width + col] = Some(0);
            }
            for col in 32..52 {
                plate[row * width + col] = Some(1);
            }
        }
        let square = |middle| Plate { cell: 0, land: true, pixels: 400, share: 0.1, middle };
        let plates = Plates {
            width,
            blocks_per_pixel: 100.0,
            vertices: Vec::new(),
            edges: Vec::new(),
            cell: vec![0; width * width],
            plate,
            inside: vec![true; width * width],
            kept: vec![square((18.0, 30.0)), square((42.0, 30.0))],
            cells: 2,
            past_the_edge: 0,
            coverage: 0.2,
        };
        let motions = vec![Motion { velocity: (60.0, 0.0), spin: 0.0 }, Motion { velocity: (-60.0, 0.0), spin: 0.0 }];
        (plates, motions)
    }

    #[test]
    fn plates_that_meet_head_on_stop_each_other_and_raise_the_ground() {
        let (plates, motions) = head_on();
        let settings = SimSettings { bowl: 0.0, ..SimSettings::default() };
        let mut sim = Sim::new(&plates, &motions);
        for _ in 0..60 {
            sim.step(&settings);
        }
        // They didn't pass through each other.
        assert!(sim.middle(0).0 < sim.middle(1).0);
        // They're no longer closing at anything like their starting speed.
        assert!(sim.motion(0).velocity.0 < 20.0, "{:?}", sim.motion(0));
        // Ground rose where they met.
        assert!(sim.paint().iter().flatten().any(|&(_, height)| height > 0.0));
        // Head on and the same size: neither is turned.
        assert!(sim.motion(0).spin.abs() < 1e-4);
    }

    #[test]
    fn friction_turns_plates_that_slide_past_each_other() {
        let (plates, _) = head_on();
        // Overlapping a little, sliding past each other.
        let motions = vec![Motion { velocity: (300.0, 80.0), spin: 0.0 }, Motion { velocity: (-300.0, -80.0), spin: 0.0 }];
        let settings = SimSettings { bowl: 0.0, push: 0.0, ..SimSettings::default() };
        let mut sim = Sim::new(&plates, &motions);
        for _ in 0..3 {
            sim.step(&settings);
        }
        assert!(sim.motion(0).spin.abs() > 1e-5 && sim.motion(1).spin.abs() > 1e-5, "{:?} {:?}", sim.motion(0), sim.motion(1));
    }

    #[test]
    fn drag_brings_a_lone_plate_to_a_stop_and_the_bowl_keeps_it_in() {
        let made = plates::make(&PlateSettings { plates: 1, coverage: 0.05, ocean_rings: 0, ..PlateSettings::default() }, 3, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap();
        let motions = vec![Motion { velocity: (400.0, 0.0), spin: 0.0 }];
        let mut sim = Sim::new(&made, &motions);
        let centre = made.width as f32 / 2.0;
        for _ in 0..3_000 {
            sim.step(&SimSettings::default());
            assert!((sim.middle(0).0 - centre).abs() < centre, "the plate's middle left the world");
            if sim.settled {
                break;
            }
        }
        assert!(sim.settled, "still moving: {:?}", sim.motion(0));
    }

    #[test]
    fn same_seed_same_simulation() {
        let made = plates::make(&PlateSettings::default(), 11, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap();
        let motions = motions(&MotionSettings::default(), 4, &made);
        let run = || {
            let mut sim = Sim::new(&made, &motions);
            for _ in 0..200 {
                sim.step(&SimSettings::default());
            }
            sim.paint()
        };
        assert_eq!(run(), run());
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

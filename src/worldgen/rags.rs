//! Rags — the tectonics simulation (World Gen tool 2, step 3). The plates are **large, heavy rags**
//! laid on a table: they squash, bend, wrinkle and tear where they're pushed, instead of bouncing
//! off each other like billiard balls.
//!
//! **What a plate is made of.** A grid of points (`grain` blocks apart), each joined to its
//! neighbours by springs: across, down, and both diagonals (so it resists shearing as well as
//! squashing). A plate is a sheet of these. Points weigh what their patch of rock weighs; ocean
//! plates' rock is lighter for its size.
//!
//! **Why a hit doesn't just spin a plate.** Each point only feels its neighbours. When a plate is
//! hit, the points at the hit are moved first, and the springs pass it on, row by row, so the far
//! side feels less of it and later. The rock near the hit takes the punishment: it squashes, bends,
//! or tears.
//!
//! **Rock gives way and stays given.** A rag springs back; rock doesn't. Each spring has a
//! **give-way** point: squeezed or stretched further than that, its resting length creeps toward
//! its new length (by **creep** each step) and stays there. That's what keeps wrinkles and squashed
//! ground in place after the push ends. Stretched past **crack**, it snaps: the plate tears there.
//!
//! **Irregular stiffness.** A noise map sets how stiff and how strong the rock is from place to
//! place. Soft patches give way deep inside a plate, not just at its edges.
//!
//! **No favourite fold** (both can be switched off, to compare). An even grid of even rock crushes
//! into the same bump over and over, at the grid's spacing and lined up with its rows, which drew
//! as robotic, repeating ridges and coasts. *Shake the grid* nudges every point off its spot and
//! splits each square along its shorter diagonal (then a random one); *uneven rock* layers the soft
//! and stiff patches at three sizes and gives every spring its own breaking point.
//!
//! **Height comes from the squeeze.** The grid's small triangles each keep their amount of rock.
//! Squeeze one to half its area and the rock there is twice as thick: **brighter**. Stretch it and
//! it thins: **darker**. Nobody places mountains; they're where rock was crowded together.
//!
//! **Each step:**
//! 1. Drag slows every point a little, and the bowl pushes back any point past its start.
//! 2. Every point moves on by its speed.
//! 3. Several **passes**, each: every spring pulls its two points back toward its resting length
//!    (by **stiffness**); every two points of different plates that are closer than the grain are
//!    pushed apart (by **push back**). Lighter points are moved more.
//! 4. **Friction:** points of different plates that are touching are dragged along with each other.
//! 5. Each point's new speed is how far it actually moved.
//! 6. Springs past give-way creep; springs past crack snap.
//!
//! The steps are always the same size and taken in the same order, so the same seed and settings
//! give the same result, however fast it's played.

use super::noise::{Fade, perlin};
use crate::fate::planet_hash;
use super::plates::Plates;
use super::tectonics::Motion;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RagSettings {
    // How the rags are made (changing these starts the run again).
    /// Blocks between the points of a plate's grid. Smaller = finer wrinkles, slower.
    pub grain: f32,
    /// How much stiffness and strength vary from place to place (0 = all the same).
    pub patchiness: f32,
    /// How big the soft and stiff patches are, in blocks.
    pub patch_size: f32,
    /// Shake the grid: nudge each point a random way off its spot (up to `shake` × the grain, at
    /// most 0.3: past about 0.35 some squares would turn inside out), and split each grid square
    /// along its shorter diagonal, which is then a random one. A perfectly regular grid crushes into the
    /// same bump over and over, at its own spacing and lined up with its own rows; a shaken one has
    /// no favourite spacing or direction to fold at.
    pub shake_grid: bool,
    pub shake: f32,
    /// Uneven rock: stiffness varies at three sizes (patch size, a third of it, a ninth), not just
    /// one; and each spring gives way and cracks at its own point (up to ± `unevenness` of the
    /// usual), so a crush front doesn't give way all along its length at once.
    pub uneven_rock: bool,
    pub unevenness: f32,

    // How they behave (can change while it runs).
    /// How firmly each spring pulls back, each pass (0 to 1).
    pub stiffness: f32,
    /// Passes per step. More = the push travels further through a plate each step (stiffer,
    /// slower to work out).
    pub passes: u32,
    /// How far a spring can be squeezed or stretched before the rock gives way for good (0.02 =
    /// 2%).
    pub give_way: f32,
    /// How much of the stretch past give-way becomes permanent each step (0 to 1).
    pub creep: f32,
    /// How far rock can be stretched in all, from how it started, before it snaps (0.4 = 40%
    /// longer).
    pub crack: f32,
    /// How firmly points of different plates are pushed apart, each pass (0 to 1).
    pub push: f32,
    /// How much plates touching each other drag each other along (0 to 1).
    pub friction: f32,
    /// How much of its speed each point loses every step (0.01 = 1%).
    pub drag: f32,
    /// How heavy ocean rock is for its size, compared with land rock.
    pub ocean_weight: f32,
    /// How strongly plates pressed together weld (0 = they never stick). Welds hold like the rock's
    /// own springs, and crack if pulled hard enough.
    pub stickiness: f32,
    /// The most rock can thicken: crushed to 1 ÷ this of its area, it pushes back instead of
    /// folding over itself.
    pub thickest: f32,
    /// The bowl starts this far out (a share of the world's radius).
    pub bowl_start: f32,
    /// How hard the bowl pushes back, in blocks per step each step, at the world's edge.
    pub bowl: f32,
}

impl Default for RagSettings {
    fn default() -> Self {
        Self {
            grain: 750.0,
            patchiness: 0.6,
            patch_size: 8_000.0,
            // Off by default, so presets saved before these existed come out as they did.
            shake_grid: false,
            shake: 0.25,
            uneven_rock: false,
            unevenness: 0.4,
            stiffness: 0.3,
            passes: 3,
            give_way: 0.02,
            creep: 0.3,
            crack: 0.4,
            push: 0.3,
            friction: 0.3,
            drag: 0.01,
            ocean_weight: 0.4,
            stickiness: 0.6,
            thickest: 4.0,
            bowl_start: 0.8,
            bowl: 2.0,
        }
    }
}

impl RagSettings {
    /// The settings the rags are made with: when these change, the run starts again.
    pub fn making(&self) -> [u32; 7] {
        [
            self.grain.to_bits(),
            self.patchiness.to_bits(),
            self.patch_size.to_bits(),
            self.shake_grid as u32,
            self.shake.to_bits(),
            self.uneven_rock as u32,
            self.unevenness.to_bits(),
        ]
    }
}

/// What's on one map pixel as the simulation runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub plate: u8,
    /// How much thicker (+) or thinner (−) the rock is than at the start: 0.5 = half as thick
    /// again.
    pub squeeze: f32,
    /// Which pixel of the plates as made this rock started on (so its terrain can be looked up).
    pub from: u32,
    /// Which way, and how hard, the rock here was squeezed, as a "doubled-angle" arrow: its
    /// direction is twice the squeeze's angle, so squeezed east–west and west–east (the same
    /// thing) add up instead of cancelling. Its length is how hard (0 = not squeezed).
    pub push: (f32, f32),
}

impl Spot {
    /// Which way the rock was squeezed, in degrees from 0 to 180, anticlockwise from east as on
    /// the map: 0 = east–west, 45 = northeast–southwest, 90 = north–south.
    pub fn push_angle(&self) -> f32 {
        // The field's rows run south, so north is −y.
        ((-self.push.1).atan2(self.push.0).to_degrees() / 2.0).rem_euclid(180.0)
    }

    /// How hard (0 = not at all; 0.3 = squashed to about 70% along that direction).
    pub fn push_strength(&self) -> f32 {
        self.push.0.hypot(self.push.1)
    }
}

/// How a triangle has been squashed: its doubled-angle arrow (see `Spot::push`), from its corners
/// at the start (`was`) and now (`now`).
///
/// The triangle's change of shape is measured as a stretch in every direction: the direction it
/// was stretched least (squashed most) is the squeeze, and how far below its old length that
/// direction now is, is how hard.
fn squash(was: [(f32, f32); 3], now: [(f32, f32); 3]) -> (f32, f32) {
    // The two sides from the first corner, at the start and now.
    let (a, b) = ((was[1].0 - was[0].0, was[1].1 - was[0].1), (was[2].0 - was[0].0, was[2].1 - was[0].1));
    let (c, d) = ((now[1].0 - now[0].0, now[1].1 - now[0].1), (now[2].0 - now[0].0, now[2].1 - now[0].1));
    let det = a.0 * b.1 - b.0 * a.1;
    if det.abs() < 1e-9 {
        return (0.0, 0.0);
    }
    // The change of shape (how start positions map to positions now): F = [c d] × [a b]⁻¹.
    let inv = [[b.1 / det, -b.0 / det], [-a.1 / det, a.0 / det]];
    let f = [[c.0 * inv[0][0] + d.0 * inv[1][0], c.0 * inv[0][1] + d.0 * inv[1][1]], [c.1 * inv[0][0] + d.1 * inv[1][0], c.1 * inv[0][1] + d.1 * inv[1][1]]];
    // How much each direction (as it lies now) has been stretched, squared: F × Fᵀ.
    let (p, q, r) = (f[0][0] * f[0][0] + f[0][1] * f[0][1], f[0][0] * f[1][0] + f[0][1] * f[1][1], f[1][0] * f[1][0] + f[1][1] * f[1][1]);
    let middle = (p + r) / 2.0;
    let spread = (((p - r) / 2.0).powi(2) + q * q).sqrt();
    let least = (middle - spread).max(0.0);
    let hard = (1.0 - least.sqrt()).max(0.0);
    // The most-stretched direction's doubled angle is atan2(2q, p − r); the least-stretched one is
    // at right angles to it, which doubled is the opposite way.
    let (x, y) = (p - r, 2.0 * q);
    let length = x.hypot(y);
    if length < 1e-9 || hard <= 0.0 {
        return (0.0, 0.0);
    }
    (-x / length * hard, -y / length * hard)
}

struct Spring {
    a: usize,
    b: usize,
    /// Its resting length now, and when it was made (field pixels).
    rest: f32,
    made: f32,
    /// How stiff and strong the rock is here (1 = ordinary).
    strength: f32,
    /// How far past the usual this spring gives way and cracks (1 = the usual; see `uneven_rock`).
    toughness: f32,
    broken: bool,
}

/// One of the grid's small triangles: its corners, its sides (springs), and its area at the start
/// (signed: the sign says which way round the corners go).
struct Triangle {
    corners: [usize; 3],
    sides: [usize; 3],
    area: f32,
}

fn signed_area(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    ((b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1)) / 2.0
}

pub struct Sim {
    width: usize,
    blocks_per_pixel: f32,
    /// Field pixels between grid points.
    grain: f32,
    /// How close points of different plates come before they touch (and push apart): the grain,
    /// less twice the shake (shaken, neighbouring plates' edge points can start that close).
    contact: f32,
    /// Each point: where it is, how fast it's moving (pixels per step), and whose it is.
    at: Vec<(f32, f32)>,
    velocity: Vec<(f32, f32)>,
    plate: Vec<u8>,
    land: Vec<bool>,
    /// Where each point started.
    home: Vec<(f32, f32)>,
    /// Whether it's on its plate (and so collides). The others are **outline points** just past
    /// the plate's edge: they move with it, so its true outline can be carried along, but they
    /// don't collide.
    real: Vec<bool>,
    /// Which plate each pixel was on at the start: the plates' true shapes, carried along by the
    /// grid when painting.
    shapes: Vec<Option<u8>>,
    springs: Vec<Spring>,
    triangles: Vec<Triangle>,
    /// Where each plate's middle was at the start (for plates too small to have any points).
    middles: Vec<(f32, f32)>,
    /// Pairs of points (from different plates) already welded together.
    welded: std::collections::HashSet<(usize, usize)>,
    /// Steps taken so far.
    pub steps: u64,
    /// Whether everything has coasted to a stop.
    pub settled: bool,
}

impl Sim {
    /// The plates as they are, as rags, with the motions they're given at the start.
    pub fn new(plates: &Plates, motions: &[Motion], settings: &RagSettings, seed: u64) -> Self {
        let (width, blocks_per_pixel) = (plates.width, plates.blocks_per_pixel);
        let grain = (settings.grain / blocks_per_pixel).max(1.0);

        // Which grid spots each plate needs: the four corners of the grid square around each of
        // its pixels, so every pixel of it sits inside its grid. (Sorted, so the same seed always
        // gives the same run.) Spot (i, j) is at ((i + ½) × grain, (j + ½) × grain), or near it
        // when the grid is shaken; then a pixel can end up in a neighbouring square's triangles, so
        // one more ring of spots is needed all round.
        let shake = if settings.shake_grid { settings.shake.clamp(0.0, 0.3) * grain } else { 0.0 };
        let reach = if shake > 0.0 { -1..=2 } else { 0..=1 };
        let mut needed: BTreeSet<(u8, i64, i64)> = BTreeSet::new();
        for (index, here) in plates.plate.iter().enumerate() {
            let Some(p) = *here else { continue };
            let (x, y) = ((index % width) as f32 + 0.5, (index / width) as f32 + 0.5);
            let (i, j) = ((x / grain - 0.5).floor() as i64, (y / grain - 0.5).floor() as i64);
            for di in reach.clone() {
                for dj in reach.clone() {
                    needed.insert((p, i + di, j + dj));
                }
            }
        }
        // A random number from 0 to 1 for a grid spot (the same for every plate there).
        let dice = |salt: u64, i: i64, j: i64| (planet_hash(seed, salt, i, j, 0) >> 40) as f32 / (1u64 << 24) as f32;

        // The points.
        let (mut at, mut velocity, mut plate, mut land, mut real, mut strength) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut point_at: HashMap<(u8, i64, i64), usize> = HashMap::new();
        let patch = (settings.patch_size / blocks_per_pixel).max(1.0);
        for &(p, i, j) in &needed {
            let Some(motion) = motions.get(p as usize) else { continue };
            let mut spot = ((i as f32 + 0.5) * grain, (j as f32 + 0.5) * grain);
            if shake > 0.0 {
                // Anywhere in a circle around its spot (square root: spread evenly over it).
                let (angle, distance) = (dice(0x5a4e_0001, i, j) * std::f32::consts::TAU, dice(0x5a4e_0002, i, j).sqrt() * shake);
                spot = (spot.0 + angle.cos() * distance, spot.1 + angle.sin() * distance);
            }
            let (col, row) = (spot.0.floor() as i64, spot.1.floor() as i64);
            let on_it = col >= 0 && row >= 0 && (col as usize) < width && (row as usize) < width && plates.plate[row as usize * width + col as usize] == Some(p);
            let v = motion.at(plates.kept[p as usize].middle, spot, blocks_per_pixel);
            point_at.insert((p, i, j), at.len());
            at.push(spot);
            velocity.push((v.0 / blocks_per_pixel, v.1 / blocks_per_pixel));
            plate.push(p);
            land.push(plates.kept[p as usize].land);
            real.push(on_it);
            let mut noise = perlin(seed, 0x5715_f0e5, Fade::Smooth, spot.0 / patch, spot.1 / patch) * 1.4;
            if settings.uneven_rock {
                // Smaller patches inside the big ones: a third and a ninth the size, each half as
                // strong as the one before.
                let third = perlin(seed, 0x5715_f0e6, Fade::Smooth, spot.0 * 3.0 / patch, spot.1 * 3.0 / patch) * 1.4;
                let ninth = perlin(seed, 0x5715_f0e7, Fade::Smooth, spot.0 * 9.0 / patch, spot.1 * 9.0 / patch) * 1.4;
                noise = (noise + third * 0.5 + ninth * 0.25) / 1.75 * 1.3;
            }
            strength.push((1.0 + settings.patchiness * noise).clamp(0.1, 2.0));
        }

        // Springs to the neighbours on the same plate: across, down, and both diagonals.
        let mut springs = Vec::new();
        let mut spring_between: HashMap<(usize, usize), usize> = HashMap::new();
        for &(p, i, j) in &needed {
            let a = point_at[&(p, i, j)];
            for (di, dj) in [(1, 0), (0, 1), (1, 1), (-1, 1)] {
                let Some(&b) = point_at.get(&(p, i + di, j + dj)) else { continue };
                let length = (at[b].0 - at[a].0).hypot(at[b].1 - at[a].1);
                let toughness = if settings.uneven_rock {
                    let roll = planet_hash(seed, 0x7006_4e55, a.min(b) as i64, a.max(b) as i64, 0) >> 40;
                    (1.0 + settings.unevenness * (roll as f32 / (1u64 << 24) as f32 * 2.0 - 1.0)).max(0.1)
                } else {
                    1.0
                };
                spring_between.insert((a.min(b), a.max(b)), springs.len());
                springs.push(Spring { a, b, rest: length, made: length, strength: (strength[a] + strength[b]) / 2.0, toughness, broken: false });
            }
        }

        // The triangles: each grid square makes two (or one, if a corner is missing), split along
        // the same diagonal every time, or a random one when the grid is shaken.
        let mut triangles = Vec::new();
        let side = |a: usize, b: usize| spring_between.get(&(a.min(b), a.max(b))).copied();
        for &(p, i, j) in &needed {
            {
                let corner = |di: i64, dj: i64| point_at.get(&(p, i + di, j + dj)).copied();
                let (c00, c10, c01, c11) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
                // Along the shorter diagonal: the healthier pair of triangles. (On an even grid both are
                // the same length and it takes the usual one; on a shaken grid it varies at random.)
                let length = |a: Option<usize>, b: Option<usize>| match (a, b) { (Some(a), Some(b)) => (at[b].0 - at[a].0).hypot(at[b].1 - at[a].1), _ => f32::MAX };
                let other_diagonal = length(c10, c01) < length(c00, c11) * 0.999;
                let threes: Vec<[usize; 3]> = match (c00, c10, c01, c11) {
                    (Some(a), Some(b), Some(c), Some(d)) if other_diagonal => vec![[a, b, c], [b, d, c]],
                    (Some(a), Some(b), Some(c), Some(d)) => vec![[a, b, d], [a, d, c]],
                    (None, Some(b), Some(c), Some(d)) => vec![[b, d, c]],
                    (Some(a), None, Some(c), Some(d)) => vec![[a, d, c]],
                    (Some(a), Some(b), None, Some(d)) => vec![[a, b, d]],
                    (Some(a), Some(b), Some(c), None) => vec![[a, b, c]],
                    _ => Vec::new(),
                };
                for corners in threes {
                    let [a, b, c] = corners;
                    if let (Some(ab), Some(bc), Some(ca)) = (side(a, b), side(b, c), side(c, a)) {
                        triangles.push(Triangle { corners, sides: [ab, bc, ca], area: signed_area(at[a], at[b], at[c]) });
                    }
                }
            }
        }

        Self {
            width,
            blocks_per_pixel,
            grain,
            contact: grain - 2.0 * shake,
            home: at.clone(),
            at,
            velocity,
            plate,
            land,
            real,
            shapes: plates.plate.clone(),
            springs,
            triangles,
            middles: plates.kept.iter().map(|plate| plate.middle).collect(),
            welded: std::collections::HashSet::new(),
            steps: 0,
            settled: false,
        }
    }

    /// How easily point `k` is moved (1 ÷ its weight).
    fn lightness(&self, k: usize, settings: &RagSettings) -> f32 {
        if self.land[k] { 1.0 } else { 1.0 / settings.ocean_weight.max(0.01) }
    }

    /// How plate `p` is moving now as a whole (blocks per step, radians per step).
    pub fn motion(&self, p: usize) -> Motion {
        let middle = self.middle(p);
        let (mut velocity, mut turn, mut spread, mut count) = ((0.0, 0.0), 0.0, 0.0, 0usize);
        for k in (0..self.at.len()).filter(|&k| self.real[k] && self.plate[k] as usize == p) {
            let v = self.velocity[k];
            let r = (self.at[k].0 - middle.0, self.at[k].1 - middle.1);
            velocity = (velocity.0 + v.0, velocity.1 + v.1);
            turn += r.0 * v.1 - r.1 * v.0;
            spread += r.0 * r.0 + r.1 * r.1;
            count += 1;
        }
        let n = count.max(1) as f32;
        let average = (velocity.0 / n, velocity.1 / n);
        Motion { velocity: (average.0 * self.blocks_per_pixel, average.1 * self.blocks_per_pixel), spin: if spread > 0.0 { turn / spread } else { 0.0 } }
    }

    /// Where plate `p`'s middle is now (the average of its points), in field pixels.
    pub fn middle(&self, p: usize) -> (f32, f32) {
        let (mut sum, mut count) = ((0.0, 0.0), 0usize);
        for k in (0..self.at.len()).filter(|&k| self.real[k] && self.plate[k] as usize == p) {
            sum = (sum.0 + self.at[k].0, sum.1 + self.at[k].1);
            count += 1;
        }
        if count == 0 { self.middles[p] } else { (sum.0 / count as f32, sum.1 / count as f32) }
    }

    /// Pairs of points from different plates near enough to touch.
    fn near_pairs(&self) -> Vec<(usize, usize)> {
        let cells = (self.width as f32 / self.grain).ceil() as usize + 1;
        let cell_of = |p: (f32, f32)| ((p.0 / self.grain).clamp(0.0, (cells - 1) as f32) as usize, (p.1 / self.grain).clamp(0.0, (cells - 1) as f32) as usize);
        let mut grid: Vec<Vec<usize>> = vec![Vec::new(); cells * cells];
        // Only the real points collide (outline points just carry the outline).
        for (k, &p) in self.at.iter().enumerate().filter(|&(k, _)| self.real[k]) {
            let (cx, cy) = cell_of(p);
            grid[cy * cells + cx].push(k);
        }
        let reach = self.grain * 1.5;
        let mut pairs = Vec::new();
        for (k, &p) in self.at.iter().enumerate().filter(|&(k, _)| self.real[k]) {
            let (cx, cy) = cell_of(p);
            for ny in cy.saturating_sub(1)..=(cy + 1).min(cells - 1) {
                for nx in cx.saturating_sub(1)..=(cx + 1).min(cells - 1) {
                    for &other in &grid[ny * cells + nx] {
                        if other > k && self.plate[other] != self.plate[k] {
                            let q = self.at[other];
                            if (q.0 - p.0).hypot(q.1 - p.1) < reach {
                                pairs.push((k, other));
                            }
                        }
                    }
                }
            }
        }
        pairs
    }

    /// One step.
    pub fn step(&mut self, settings: &RagSettings) {
        let field = self.width as f32;
        let (centre, radius) = (field / 2.0, field / 2.0);
        let bowl_start = radius * settings.bowl_start.clamp(0.0, 0.999);
        let bowl = settings.bowl / self.blocks_per_pixel;
        let keep = 1.0 - settings.drag.clamp(0.0, 1.0);

        // 1 and 2. Drag and the bowl, then move on.
        let before = self.at.clone();
        for k in 0..self.at.len() {
            let (dx, dy) = (self.at[k].0 - centre, self.at[k].1 - centre);
            let out = dx.hypot(dy);
            let v = &mut self.velocity[k];
            if out > bowl_start && out > 1e-3 {
                let depth = (out - bowl_start) / (radius - bowl_start);
                *v = (v.0 - dx / out * bowl * depth, v.1 - dy / out * bowl * depth);
            }
            *v = (v.0 * keep, v.1 * keep);
            self.at[k] = (self.at[k].0 + v.0, self.at[k].1 + v.1);
        }

        // 3. Passes: springs pull back, touching plates push apart.
        let pairs = self.near_pairs();
        let lightness: Vec<f32> = (0..self.at.len()).map(|k| self.lightness(k, settings)).collect();
        let stiffness = settings.stiffness.clamp(0.0, 1.0);
        let push = settings.push.clamp(0.0, 1.0);
        for _ in 0..settings.passes.max(1) {
            for spring in self.springs.iter().filter(|s| !s.broken) {
                let (a, b) = (self.at[spring.a], self.at[spring.b]);
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let length = dx.hypot(dy).max(1e-6);
                let (wa, wb) = (lightness[spring.a], lightness[spring.b]);
                let pull = (stiffness * spring.strength).min(1.0) * (length - spring.rest) / length / (wa + wb);
                self.at[spring.a] = (a.0 + dx * pull * wa, a.1 + dy * pull * wa);
                self.at[spring.b] = (b.0 - dx * pull * wb, b.1 - dy * pull * wb);
            }
            for &(i, j) in &pairs {
                let (a, b) = (self.at[i], self.at[j]);
                let (dx, dy) = (a.0 - b.0, a.1 - b.1);
                let length = dx.hypot(dy).max(1e-6);
                if length >= self.contact {
                    continue;
                }
                let (wi, wj) = (lightness[i], lightness[j]);
                let apart = push * (self.contact - length) / length / (wi + wj);
                self.at[i] = (a.0 + dx * apart * wi, a.1 + dy * apart * wi);
                self.at[j] = (b.0 - dx * apart * wj, b.1 - dy * apart * wj);
            }
            self.keep_from_folding(&lightness, settings.thickest);
        }

        // 4. Friction: touching points drag each other along (the part of their movement past
        // each other is shared out).
        let friction = settings.friction.clamp(0.0, 1.0);
        for &(i, j) in &pairs {
            let (a, b) = (self.at[i], self.at[j]);
            let (dx, dy) = (a.0 - b.0, a.1 - b.1);
            let length = dx.hypot(dy).max(1e-6);
            if length > self.contact * 1.05 {
                continue;
            }
            let n = (dx / length, dy / length);
            let moved = ((a.0 - before[i].0) - (b.0 - before[j].0), (a.1 - before[i].1) - (b.1 - before[j].1));
            let across = moved.0 * n.0 + moved.1 * n.1;
            let past = (moved.0 - across * n.0, moved.1 - across * n.1);
            let (wi, wj) = (lightness[i], lightness[j]);
            let share = friction / (wi + wj);
            self.at[i] = (a.0 - past.0 * share * wi, a.1 - past.1 * share * wi);
            self.at[j] = (b.0 + past.0 * share * wj, b.1 + past.1 * share * wj);
        }

        // Welding: points of different plates pressed into contact are joined by a new spring at
        // that distance, so the plates stick instead of drifting back apart. (In the order the
        // pairs were found, so the same run always welds the same way.)
        if settings.stickiness > 0.0 {
            for &(i, j) in &pairs {
                let (a, b) = (self.at[i], self.at[j]);
                let length = (a.0 - b.0).hypot(a.1 - b.1);
                if length <= self.contact * 1.02 && self.welded.insert((i, j)) {
                    self.springs.push(Spring { a: i, b: j, rest: self.contact, made: self.contact, strength: settings.stickiness, toughness: 1.0, broken: false });
                }
            }
        }

        // 5. The new speeds: how far each point actually moved.
        let mut fastest: f32 = 0.0;
        for k in 0..self.at.len() {
            self.velocity[k] = (self.at[k].0 - before[k].0, self.at[k].1 - before[k].1);
            fastest = fastest.max(self.velocity[k].0.hypot(self.velocity[k].1));
        }

        // 6. Give way and crack.
        let creep = settings.creep.clamp(0.0, 1.0);
        let real = &self.real;
        for spring in self.springs.iter_mut().filter(|s| !s.broken) {
            let (a, b) = (self.at[spring.a], self.at[spring.b]);
            let length = (b.0 - a.0).hypot(b.1 - a.1);
            // Cracking is measured from the length it started with: rock can stretch and give way
            // only so far in all before it snaps. (Springs to outline points never snap, so the
            // outline can't break loose and drift off.)
            let between_real = real[spring.a] && real[spring.b];
            if between_real && length / spring.made - 1.0 > settings.crack * spring.strength * spring.toughness {
                spring.broken = true;
                continue;
            }
            let strain = length / spring.rest - 1.0;
            let give = settings.give_way * spring.strength * spring.toughness;
            if strain.abs() > give {
                // Creep toward the length that would be exactly at give-way.
                let settled_at = length / (1.0 + give * strain.signum());
                spring.rest += (settled_at - spring.rest) * creep;
                // Never squashed to nothing.
                spring.rest = spring.rest.max(spring.made * 0.2);
            }
        }

        self.steps += 1;
        // Coasted to a stop: no point moves even 1 block per step.
        self.settled = fastest * self.blocks_per_pixel < 1.0;
    }

    /// Rock can't be crushed thinner than a share of its area, nor fold over onto itself: any
    /// triangle squashed to less than 1 ÷ `thickest` of its area at the start is pushed back open
    /// to that (its corners moved straight away from the side opposite them, lighter rock moving
    /// more). So crushed rock thickens up to `thickest` times and then pushes back, spreading the
    /// crush further out, instead of folding into overlapping layers (which drew as sawtooth
    /// "zipper teeth" on the squeeze map).
    fn keep_from_folding(&mut self, lightness: &[f32], thickest: f32) {
        let least = 1.0 / thickest.max(1.01);
        for triangle in &self.triangles {
            let [a, b, c] = triangle.corners;
            // Area measured the same way round as at the start, so a fold reads as negative.
            let sign = triangle.area.signum();
            let (pa, pb, pc) = (self.at[a], self.at[b], self.at[c]);
            let now = signed_area(pa, pb, pc) * sign;
            let want = triangle.area.abs() * least;
            if now >= want {
                continue;
            }
            // How each corner moving changes the area (half the opposite side, turned outward).
            let outward = |from: (f32, f32), to: (f32, f32)| ((from.1 - to.1) * 0.5 * sign, (to.0 - from.0) * 0.5 * sign);
            let gradients = [outward(pb, pc), outward(pc, pa), outward(pa, pb)];
            let weights = [lightness[a], lightness[b], lightness[c]];
            let total: f32 = gradients.iter().zip(&weights).map(|(g, w)| w * (g.0 * g.0 + g.1 * g.1)).sum();
            if total < 1e-9 {
                continue;
            }
            let scale = (want - now) / total;
            for ((k, g), w) in [a, b, c].into_iter().zip(gradients).zip(weights) {
                self.at[k] = (self.at[k].0 + g.0 * scale * w, self.at[k].1 + g.1 * scale * w);
            }
        }
    }

    /// Whether a triangle has been torn wide open: one of its sides is cracked and stretched to
    /// more than `tear_open` times its length at the start. (Only cracked sides tear: an unbroken
    /// spring is still holding the rock together.)
    fn torn(&self, triangle: &Triangle, tear_open: f32) -> bool {
        triangle.sides.iter().any(|&s| {
            let spring = &self.springs[s];
            let (a, b) = (self.at[spring.a], self.at[spring.b]);
            spring.broken && (b.0 - a.0).hypot(b.1 - a.1) > spring.made * tear_open
        })
    }

    /// How thick the rock is at each point, compared with at the start (1 = unchanged): the
    /// average over its triangles of start area ÷ area now. Cracked triangles count too (a crack
    /// pulled a little open is thin, stretched rock), until they're torn wide open.
    ///
    /// Then blended `smoothing` times with each point's neighbours, so the shading reads as smooth
    /// fields rather than the grid's triangles.
    fn thickness(&self, smoothing: u32, tear_open: f32) -> Vec<f32> {
        let mut sum = vec![(0.0f32, 0u32); self.at.len()];
        for triangle in self.triangles.iter().filter(|t| !self.torn(t, tear_open)) {
            let [a, b, c] = triangle.corners;
            // Folded over counts as crushed (its area, whichever way round).
            let now = signed_area(self.at[a], self.at[b], self.at[c]).abs().max(1e-6);
            let ratio = (triangle.area.abs() / now).clamp(0.25, 10.0);
            for k in triangle.corners {
                sum[k] = (sum[k].0 + ratio, sum[k].1 + 1);
            }
        }
        let mut thickness: Vec<f32> = sum.into_iter().map(|(total, n)| if n == 0 { 1.0 } else { total / n as f32 }).collect();
        for _ in 0..smoothing {
            let mut blended: Vec<(f32, u32)> = thickness.iter().map(|&t| (t, 1)).collect();
            for spring in &self.springs {
                blended[spring.a] = (blended[spring.a].0 + thickness[spring.b], blended[spring.a].1 + 1);
                blended[spring.b] = (blended[spring.b].0 + thickness[spring.a], blended[spring.b].1 + 1);
            }
            thickness = blended.into_iter().map(|(total, n)| total / n as f32).collect();
        }
        thickness
    }

    /// Which way, and how hard, the rock at each point was squeezed (see `Spot::push`): the
    /// average over its triangles (until they're torn open), blended `smoothing` times with its
    /// neighbours like the thickness.
    fn pushes(&self, smoothing: u32, tear_open: f32) -> Vec<(f32, f32)> {
        let mut sum = vec![((0.0f32, 0.0f32), 0u32); self.at.len()];
        for triangle in self.triangles.iter().filter(|t| !self.torn(t, tear_open)) {
            let [a, b, c] = triangle.corners;
            let push = squash([self.home[a], self.home[b], self.home[c]], [self.at[a], self.at[b], self.at[c]]);
            for k in triangle.corners {
                sum[k] = ((sum[k].0.0 + push.0, sum[k].0.1 + push.1), sum[k].1 + 1);
            }
        }
        let mut pushes: Vec<(f32, f32)> = sum.into_iter().map(|(total, n)| if n == 0 { (0.0, 0.0) } else { (total.0 / n as f32, total.1 / n as f32) }).collect();
        for _ in 0..smoothing {
            let mut blended: Vec<((f32, f32), u32)> = pushes.iter().map(|&p| (p, 1)).collect();
            for spring in &self.springs {
                let (pa, pb) = (pushes[spring.a], pushes[spring.b]);
                blended[spring.a] = ((blended[spring.a].0.0 + pb.0, blended[spring.a].0.1 + pb.1), blended[spring.a].1 + 1);
                blended[spring.b] = ((blended[spring.b].0.0 + pa.0, blended[spring.b].0.1 + pa.1), blended[spring.b].1 + 1);
            }
            pushes = blended.into_iter().map(|(total, n)| (total.0 / n as f32, total.1 / n as f32)).collect();
        }
        pushes
    }

    /// The plates where they are now: for each map pixel, what's there (see `Spot`).
    ///
    /// The plates keep their true outlines: each spot inside a grid triangle now is traced back to
    /// the same place in that triangle as it was at the start, and painted only if that pixel was
    /// on the plate then. So the grid carries the outline along, bending it as the rock bends.
    ///
    /// Triangles are drawn cracked or not, so small cracks don't punch holes in the fill; but one
    /// torn wide open (a cracked side stretched past `tear_open` times its length) isn't drawn: it
    /// would smear the plate across the gap. That gap is a real rift, and shows the sea. The
    /// shading (thickness, blended `smoothing` times) goes on top.
    ///
    /// Then any sea narrower than `seal` pixels with rock on both sides is filled from the nearest
    /// rock (see `seal_gaps`).
    ///
    /// Last, any hole no bigger than a circle `fill` pixels across, with rock all round it, is
    /// filled from the nearest rock (see `fill_holes`).
    pub fn paint(&self, smoothing: u32, tear_open: f32, seal: f32, fill: f32) -> Vec<Option<Spot>> {
        let mut painted = self.paint_plates(smoothing, tear_open);
        if seal > 0.0 {
            seal_gaps(&mut painted, self.width, seal / 2.0);
        }
        if fill > 0.0 {
            fill_holes(&mut painted, self.width, std::f32::consts::FRAC_PI_4 * fill * fill);
        }
        painted
    }

    fn paint_plates(&self, smoothing: u32, tear_open: f32) -> Vec<Option<Spot>> {
        let width = self.width;
        let mut painted = vec![None; width * width];
        let thickness = self.thickness(smoothing, tear_open);
        let pushes = self.pushes(smoothing, tear_open);
        for triangle in self.triangles.iter().filter(|t| !self.torn(t, tear_open)) {
            let [a, b, c] = triangle.corners;
            let (pa, pb, pc) = (self.at[a], self.at[b], self.at[c]);
            let area = signed_area(pa, pb, pc);
            if area.abs() < 1e-6 {
                continue;
            }
            let left = pa.0.min(pb.0).min(pc.0).floor().max(0.0) as usize;
            let right = (pa.0.max(pb.0).max(pc.0).ceil().max(0.0) as usize).min(width);
            let top = pa.1.min(pb.1).min(pc.1).floor().max(0.0) as usize;
            let bottom = (pa.1.max(pb.1).max(pc.1).ceil().max(0.0) as usize).min(width);
            for row in top..bottom {
                for col in left..right {
                    let p = (col as f32 + 0.5, row as f32 + 0.5);
                    // How much of each corner this spot is (all three ≥ 0 means inside).
                    let (wa, wb, wc) = (signed_area(p, pb, pc) / area, signed_area(pa, p, pc) / area, signed_area(pa, pb, p) / area);
                    if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                        continue;
                    }
                    // The same place in the triangle as it was at the start: was it on this plate?
                    let (ha, hb, hc) = (self.home[a], self.home[b], self.home[c]);
                    let was = (wa * ha.0 + wb * hb.0 + wc * hc.0, wa * ha.1 + wb * hb.1 + wc * hc.1);
                    let (wcol, wrow) = (was.0.floor(), was.1.floor());
                    if wcol < 0.0 || wrow < 0.0 || wcol >= width as f32 || wrow >= width as f32 {
                        continue;
                    }
                    let from = wrow as usize * width + wcol as usize;
                    if self.shapes[from] != Some(self.plate[a]) {
                        continue;
                    }
                    let squeeze = wa * thickness[a] + wb * thickness[b] + wc * thickness[c] - 1.0;
                    let push = (wa * pushes[a].0 + wb * pushes[b].0 + wc * pushes[c].0, wa * pushes[a].1 + wb * pushes[b].1 + wc * pushes[c].1);
                    painted[row * width + col] = Some(Spot { plate: self.plate[a], squeeze, from: from as u32, push });
                }
            }
        }
        painted
    }
}

/// For every pixel, how far it is (in pixels) from the nearest pixel where `from` is true, and
/// which pixel that is. Two sweeps, each passing distances on to neighbours (1 across, 1.41
/// diagonally).
fn nearest(from: &[bool], width: usize) -> (Vec<f32>, Vec<usize>) {
    let mut distance: Vec<f32> = from.iter().map(|&f| if f { 0.0 } else { f32::MAX }).collect();
    let mut source: Vec<usize> = (0..from.len()).collect();
    const DIAGONAL: f32 = std::f32::consts::SQRT_2;
    let mut sweep = |rows: &mut dyn Iterator<Item = usize>, steps: [(i64, i64, f32); 4]| {
        for row in rows {
            let cols: Box<dyn Iterator<Item = usize>> = if steps[0].0 < 0 { Box::new(0..width) } else { Box::new((0..width).rev()) };
            for col in cols {
                let i = row * width + col;
                for (dc, dr, step) in steps {
                    let (c, r) = (col as i64 + dc, row as i64 + dr);
                    if c < 0 || r < 0 || c as usize >= width || r as usize >= width {
                        continue;
                    }
                    let j = r as usize * width + c as usize;
                    if distance[j] + step < distance[i] {
                        distance[i] = distance[j] + step;
                        source[i] = source[j];
                    }
                }
            }
        }
    };
    sweep(&mut (0..width), [(-1, 0, 1.0), (0, -1, 1.0), (-1, -1, DIAGONAL), (1, -1, DIAGONAL)]);
    sweep(&mut (0..width).rev(), [(1, 0, 1.0), (0, 1, 1.0), (1, 1, DIAGONAL), (-1, 1, DIAGONAL)]);
    (distance, source)
}

/// Seals thin gaps of sea between rock: grows the rock outward by `reach` pixels, then shrinks it
/// back by as much. Gaps narrower than twice `reach` stay filled; wider seas open again. Filled
/// pixels take the nearest rock's values (so a crushed range runs across where the gap was).
fn seal_gaps(painted: &mut [Option<Spot>], width: usize, reach: f32) {
    let rock: Vec<bool> = painted.iter().map(Option::is_some).collect();
    let (to_rock, nearest_rock) = nearest(&rock, width);
    let grown: Vec<bool> = to_rock.iter().map(|&d| d <= reach).collect();
    let outside: Vec<bool> = grown.iter().map(|&g| !g).collect();
    let (to_outside, _) = nearest(&outside, width);
    for i in 0..painted.len() {
        if painted[i].is_none() && grown[i] && to_outside[i] > reach {
            painted[i] = painted[nearest_rock[i]];
        }
    }
}

/// The crust has no holes. Each point of a plate keeps other plates' points at arm's length, so
/// where three plates meet, or one plate's corner is jammed into another's notch, the points can't
/// pack the space and a pocket of nothing opens between them: drawn as deep-ocean "bubbles" drifting
/// round collision zones. Real plates leave no such space. So any hole with rock all round it (not
/// joined to the open sea round the plates) and no bigger than `most` pixels is filled from the
/// nearest rock, as `seal_gaps` fills thin gaps. Bigger ones (a rift torn wide open) stay.
fn fill_holes(painted: &mut [Option<Spot>], width: usize, most: f32) {
    // Which empty pixels are joined to the field's edge (the open sea): flood in from it.
    let mut open = vec![false; painted.len()];
    let mut stack: Vec<usize> = (0..painted.len())
        .filter(|&i| {
            let (col, row) = (i % width, i / width);
            (col == 0 || row == 0 || col == width - 1 || row == width - 1) && painted[i].is_none()
        })
        .collect();
    for &i in &stack {
        open[i] = true;
    }
    let neighbours = |i: usize| {
        let (col, row) = (i % width, i / width);
        [(col > 0).then(|| i - 1), (col + 1 < width).then(|| i + 1), (row > 0).then(|| i - width), (row + 1 < width).then(|| i + width)]
    };
    while let Some(i) = stack.pop() {
        for j in neighbours(i).into_iter().flatten() {
            if !open[j] && painted[j].is_none() {
                open[j] = true;
                stack.push(j);
            }
        }
    }
    // Each hole: its pixels; small ones are filled.
    let rock: Vec<bool> = painted.iter().map(Option::is_some).collect();
    let (_, nearest_rock) = nearest(&rock, width);
    let mut seen = open;
    for start in 0..painted.len() {
        if seen[start] || painted[start].is_some() {
            continue;
        }
        seen[start] = true;
        let mut hole = vec![start];
        let mut next = 0;
        while next < hole.len() {
            for j in neighbours(hole[next]).into_iter().flatten() {
                if !seen[j] && painted[j].is_none() {
                    seen[j] = true;
                    hole.push(j);
                }
            }
            next += 1;
        }
        if hole.len() as f32 <= most {
            for i in hole {
                painted[i] = painted[nearest_rock[i]];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::plates::{self, Plate, PlateSettings};
    use crate::worldgen::tectonics::{MotionSettings, motions};
    use std::sync::atomic::AtomicBool;

    /// Two square plates in a 120-pixel world, side by side, moving straight at each other. One
    /// point per 2 pixels.
    fn head_on(speed: f32) -> (Plates, Vec<Motion>, RagSettings) {
        let width = 120;
        let mut plate = vec![None; width * width];
        for row in 40..80 {
            for col in 10..60 {
                plate[row * width + col] = Some(0);
            }
            for col in 60..110 {
                plate[row * width + col] = Some(1);
            }
        }
        let square = |middle| Plate { cell: 0, land: true, pixels: 2_000, share: 0.1, middle };
        let plates = Plates {
            width,
            blocks_per_pixel: 100.0,
            vertices: Vec::new(),
            edges: Vec::new(),
            cell: vec![0; width * width],
            plate,
            inside: vec![true; width * width],
            kept: vec![square((35.0, 60.0)), square((85.0, 60.0))],
            cells: 2,
            past_the_edge: 0,
            coverage: 0.2,
        };
        let motions = vec![Motion { velocity: (speed, 0.0), spin: 0.0 }, Motion { velocity: (-speed, 0.0), spin: 0.0 }];
        let settings = RagSettings { grain: 200.0, patchiness: 0.0, bowl: 0.0, ..RagSettings::default() };
        (plates, motions, settings)
    }

    #[test]
    fn a_hit_is_felt_first_where_it_lands_and_the_far_side_keeps_coming() {
        let (plates, motions, settings) = head_on(100.0);
        let mut sim = Sim::new(&plates, &motions, &settings, 1);
        for _ in 0..4 {
            sim.step(&settings);
        }
        // The left plate's points: the ones at the hit (right edge) have slowed; the far side
        // (left edge) hasn't yet.
        let speed_at = |x0: f32, x1: f32| {
            let near: Vec<f32> = (0..sim.at.len()).filter(|&k| sim.plate[k] == 0 && (x0..x1).contains(&sim.at[k].0)).map(|k| sim.velocity[k].0).collect();
            near.iter().sum::<f32>() / near.len().max(1) as f32
        };
        let (hit, far) = (speed_at(52.0, 70.0), speed_at(0.0, 20.0));
        assert!(hit < far * 0.8, "hit side {hit}, far side {far}");
    }

    #[test]
    fn squeezed_ground_rises_and_plates_dont_pass_through() {
        let (plates, motions, settings) = head_on(100.0);
        let mut sim = Sim::new(&plates, &motions, &settings, 1);
        for _ in 0..80 {
            sim.step(&settings);
        }
        assert!(sim.middle(0).0 < sim.middle(1).0, "they passed through each other");
        let highest = sim.paint(0, f32::MAX, 0.0, 0.0).iter().flatten().map(|spot| spot.squeeze).fold(0.0, f32::max);
        assert!(highest > 0.1, "nothing rose: highest {highest}");
        // Given way: some springs are now shorter at rest than when made.
        assert!(sim.springs.iter().any(|s| s.rest < s.made * 0.97));
    }

    #[test]
    fn pulled_hard_enough_a_plate_tears() {
        let (plates, _, settings) = head_on(0.0);
        let still = vec![Motion { velocity: (0.0, 0.0), spin: 0.0 }; 2];
        let mut sim = Sim::new(&plates, &still, &settings, 1);
        // Pull the left plate's two halves apart.
        for k in 0..sim.at.len() {
            if sim.plate[k] == 0 {
                sim.velocity[k] = (if sim.at[k].0 < 35.0 { -1.0 } else { 1.0 }, 0.0);
            }
        }
        for _ in 0..10 {
            sim.step(&settings);
        }
        assert!(sim.springs.iter().any(|s| s.broken), "nothing tore");
        // The tear shows as stretched, thin rock, not a hole: the plate covers at least as much
        // ground as before, and the middle of the tear is thinner than it started.
        let painted = sim.paint(0, f32::MAX, 0.0, 0.0);
        let before = plates.plate.iter().filter(|&&p| p == Some(0)).count();
        let after = painted.iter().flatten().filter(|spot| spot.plate == 0).count();
        assert!(after >= before, "the plate lost fill: {before} pixels before, {after} after");
        let middle = painted[60 * plates.width + 35];
        assert!(matches!(middle, Some(Spot { plate: 0, squeeze, .. }) if squeeze < 0.0), "the tear's middle: {middle:?}");
        // The outline never breaks loose: springs to outline points don't snap.
        assert!(sim.springs.iter().filter(|s| !sim.real[s.a] || !sim.real[s.b]).all(|s| !s.broken));

        // Pulled much further, the crack is torn wide open: it shows as a gap (sea), not the
        // plate smeared across it.
        for _ in 0..30 {
            sim.step(&RagSettings { drag: 0.0, ..settings.clone() });
        }
        let painted = sim.paint(0, 2.0, 0.0, 0.0);
        let gap = (0..plates.width).filter(|&col| painted[60 * plates.width + col].is_none()).count();
        let row: Vec<bool> = (0..plates.width).map(|col| painted[60 * plates.width + col].is_some()).collect();
        let first = row.iter().position(|&on| on).unwrap();
        let last = row.iter().rposition(|&on| on).unwrap();
        assert!(row[first..=last].iter().any(|&on| !on), "no gap opened across the tear ({gap} empty pixels in the row)");
    }

    #[test]
    fn a_lone_plate_coasts_to_a_stop_inside_the_bowl() {
        let made = plates::make(&PlateSettings { plates: 1, coverage: 0.05, ocean_rings: 0, ..PlateSettings::default() }, 3, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap();
        let motions = vec![Motion { velocity: (300.0, 0.0), spin: 0.0 }];
        let settings = RagSettings::default();
        let mut sim = Sim::new(&made, &motions, &settings, 1);
        let centre = made.width as f32 / 2.0;
        for _ in 0..3_000 {
            sim.step(&settings);
            assert!((sim.middle(0).0 - centre).abs() < centre, "the plate's middle left the world");
            if sim.settled {
                break;
            }
        }
        assert!(sim.settled, "still moving: {:?}", sim.motion(0));
    }

    #[test]
    fn plates_keep_their_true_outlines_however_coarse_the_grain() {
        // Fine pixels (64 blocks) and a coarse grain (1,500 blocks: one point every 23 pixels).
        let made = plates::make(&PlateSettings::default(), 5, 72_000.0, 64.0, &AtomicBool::new(false)).unwrap();
        let still = vec![Motion { velocity: (0.0, 0.0), spin: 0.0 }; made.kept.len()];
        let settings = RagSettings { grain: 1_500.0, bowl: 0.0, ..RagSettings::default() };
        let mut sim = Sim::new(&made, &still, &settings, 1);
        sim.step(&settings);
        let painted: Vec<Option<u8>> = sim.paint(0, f32::MAX, 0.0, 0.0).iter().map(|here| here.map(|spot| spot.plate)).collect();
        let wrong = painted.iter().zip(&made.plate).filter(|(a, b)| a != b).count();
        assert_eq!(wrong, 0, "{wrong} pixels differ from the plates as made");
    }

    #[test]
    fn a_shaken_grid_keeps_the_true_outlines_too() {
        let made = plates::make(&PlateSettings::default(), 5, 72_000.0, 64.0, &AtomicBool::new(false)).unwrap();
        let still = vec![Motion { velocity: (0.0, 0.0), spin: 0.0 }; made.kept.len()];
        let settings = RagSettings { grain: 1_500.0, bowl: 0.0, shake_grid: true, shake: 0.3, uneven_rock: true, ..RagSettings::default() };
        let mut sim = Sim::new(&made, &still, &settings, 1);
        // Really shaken: points are off their even spots.
        let grain = 1_500.0 / 64.0;
        let off = sim.home.iter().filter(|p| ((p.0 / grain - 0.5) - (p.0 / grain - 0.5).round()).abs() > 0.05).count();
        assert!(off > sim.home.len() / 2, "only {off} of {} points moved", sim.home.len());
        sim.step(&settings);
        let painted: Vec<Option<u8>> = sim.paint(0, f32::MAX, 0.0, 0.0).iter().map(|here| here.map(|spot| spot.plate)).collect();
        let wrong = painted.iter().zip(&made.plate).filter(|(a, b)| a != b).count();
        assert_eq!(wrong, 0, "{wrong} pixels differ from the plates as made");
    }

    #[test]
    fn shaken_uneven_rock_still_never_folds_over_itself() {
        let (plates, motions, settings) = head_on(100.0);
        let settings = RagSettings { stiffness: 0.05, passes: 2, push: 0.1, shake_grid: true, uneven_rock: true, patchiness: 0.6, ..settings };
        let mut sim = Sim::new(&plates, &motions, &settings, 1);
        for _ in 0..60 {
            sim.step(&settings);
        }
        for triangle in &sim.triangles {
            let [a, b, c] = triangle.corners;
            let now = signed_area(sim.at[a], sim.at[b], sim.at[c]) * triangle.area.signum();
            assert!(now > 0.0, "a triangle folded over");
        }
        assert!(sim.paint(0, f32::MAX, 0.0, 0.0).iter().flatten().any(|spot| spot.squeeze > 0.5));
    }

    #[test]
    fn rock_thickens_but_never_folds_over_itself() {
        // A hard head-on smash with soft rock (as fast as the real sim allows: about half a grid
        // cell a step): plenty to fold, without the guard.
        let (plates, motions, settings) = head_on(100.0);
        let settings = RagSettings { stiffness: 0.05, passes: 2, push: 0.1, ..settings };
        let mut sim = Sim::new(&plates, &motions, &settings, 1);
        for _ in 0..60 {
            sim.step(&settings);
        }
        let mut thinnest: f32 = f32::MAX;
        for triangle in &sim.triangles {
            let [a, b, c] = triangle.corners;
            let now = signed_area(sim.at[a], sim.at[b], sim.at[c]) * triangle.area.signum();
            thinnest = thinnest.min(now / triangle.area.abs());
        }
        // Never folded: no triangle turned over onto itself.
        assert!(thinnest > 0.0, "a triangle folded over: {thinnest} of its area");
        // And the crush is still there: some rock did thicken.
        assert!(sim.paint(0, f32::MAX, 0.0, 0.0).iter().flatten().any(|spot| spot.squeeze > 0.5));
    }

    #[test]
    fn small_holes_in_the_rock_are_filled_and_big_ones_and_open_sea_stay() {
        // A 40-pixel field: sea along the left edge (cols 0..4), rock elsewhere, with a 3 × 3 hole
        // and a 10 × 10 hole in it.
        let width = 40;
        let spot = Spot { plate: 0, squeeze: 0.2, from: 0, push: (0.0, 0.0) };
        let mut painted: Vec<Option<Spot>> = (0..width * width)
            .map(|i| {
                let (col, row) = (i % width, i / width);
                let small = (10..13).contains(&col) && (10..13).contains(&row);
                let big = (20..30).contains(&col) && (20..30).contains(&row);
                (col >= 4 && !small && !big).then_some(spot)
            })
            .collect();
        // Up to a circle 6 pixels across (about 28 pixels): the 9-pixel hole, not the 100-pixel one.
        fill_holes(&mut painted, width, std::f32::consts::FRAC_PI_4 * 36.0);
        assert_eq!(painted[11 * width + 11], Some(spot), "the small hole wasn't filled");
        assert!(painted[25 * width + 25].is_none(), "the big hole was filled");
        assert!(painted[20 * width + 2].is_none(), "the open sea was filled");
    }

    #[test]
    fn thin_gaps_between_rock_are_sealed_and_wide_ones_stay_open() {
        // Three strips of rock down a 40-pixel field: a 3-pixel gap, then a 12-pixel one.
        let width = 40;
        let rock = |col: usize| (2..10).contains(&col) || (13..20).contains(&col) || (32..38).contains(&col);
        let mut painted: Vec<Option<Spot>> = (0..width * width)
            .map(|i| rock(i % width).then_some(Spot { plate: (i % width / 13) as u8, squeeze: 0.5, from: 0, push: (0.0, 0.0) }))
            .collect();
        seal_gaps(&mut painted, width, 2.5);
        let row = |col: usize| painted[20 * width + col].is_some();
        assert!((10..13).all(row), "the thin gap wasn't sealed");
        assert!((21..31).all(|col| !row(col)), "the wide gap was sealed");
        // Filled from the nearest rock: its squeeze carries across.
        assert_eq!(painted[20 * width + 11].map(|spot| spot.squeeze), Some(0.5));
    }

    #[test]
    fn welded_plates_stay_together_when_pulled_back_apart() {
        let apart = |stickiness: f32| {
            let (plates, motions, settings) = head_on(100.0);
            let settings = RagSettings { stickiness, ..settings };
            let mut sim = Sim::new(&plates, &motions, &settings, 1);
            for _ in 0..40 {
                sim.step(&settings);
            }
            // Now pull them back apart, gently.
            for k in 0..sim.at.len() {
                sim.velocity[k] = (if sim.plate[k] == 0 { -1.0 } else { 1.0 }, 0.0);
            }
            for _ in 0..40 {
                sim.step(&RagSettings { drag: 0.0, ..settings.clone() });
            }
            // How much sea opened between them: empty pixels between the two plates, along the rows
            // where they met.
            let painted = sim.paint(0, f32::MAX, 0.0, 0.0);
            let width = plates.width;
            let mut gap = 0;
            for row in 44..76 {
                let line: Vec<bool> = (0..width).map(|col| painted[row * width + col].is_some()).collect();
                if let (Some(first), Some(last)) = (line.iter().position(|&on| on), line.iter().rposition(|&on| on)) {
                    gap += line[first..=last].iter().filter(|&&on| !on).count();
                }
            }
            gap
        };
        let (welded, loose) = (apart(1.0), apart(0.0));
        assert!(welded * 2 < loose, "sea between them: welded {welded} pixels, loose {loose}");
    }

    #[test]
    fn squeeze_directions_point_the_way_the_rock_was_squashed() {
        let was = [(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)];
        let spot = |push| Spot { plate: 0, squeeze: 0.0, from: 0, push };
        // Squashed east–west to 70%: 0 degrees, strength 0.3.
        let east_west = spot(squash(was, [(0.0, 0.0), (7.0, 0.0), (0.0, 10.0)]));
        assert!(east_west.push_angle().min(180.0 - east_west.push_angle()) < 0.5, "{}", east_west.push_angle());
        assert!((east_west.push_strength() - 0.3).abs() < 0.01);
        // Squashed north–south: 90 degrees.
        let north_south = spot(squash(was, [(0.0, 0.0), (10.0, 0.0), (0.0, 6.0)]));
        assert!((north_south.push_angle() - 90.0).abs() < 0.5, "{}", north_south.push_angle());
        // Squashed along the northeast–southwest line (rows run south, so north-east is +x, −y).
        let (c, s) = (std::f32::consts::FRAC_1_SQRT_2, -std::f32::consts::FRAC_1_SQRT_2);
        let squashed = |(x, y): (f32, f32)| {
            let along = (x * c + y * s) * 0.5;
            let across = -x * s + y * c;
            (along * c - across * s, along * s + across * c)
        };
        let diagonal = spot(squash(was, was.map(squashed)));
        assert!((diagonal.push_angle() - 45.0).abs() < 0.5, "{}", diagonal.push_angle());
        // Moved and turned, but not squashed: no squeeze.
        let turned = squash(was, [(5.0, 5.0), (5.0, 15.0), (-5.0, 5.0)]);
        assert!(spot(turned).push_strength() < 1e-4);
    }

    #[test]
    fn same_seed_same_run() {
        let made = plates::make(&PlateSettings::default(), 11, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap();
        let motions = motions(&MotionSettings::default(), 4, &made);
        let settings = RagSettings::default();
        let run = || {
            let mut sim = Sim::new(&made, &motions, &settings, 9);
            for _ in 0..100 {
                sim.step(&settings);
            }
            sim.paint(0, f32::MAX, 0.0, 0.0)
        };
        assert_eq!(run(), run());
    }
}

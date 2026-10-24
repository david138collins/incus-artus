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
            stiffness: 0.3,
            passes: 3,
            give_way: 0.02,
            creep: 0.3,
            crack: 0.4,
            push: 0.3,
            friction: 0.3,
            drag: 0.01,
            ocean_weight: 0.4,
            bowl_start: 0.8,
            bowl: 2.0,
        }
    }
}

impl RagSettings {
    /// The settings the rags are made with: when these change, the run starts again.
    pub fn making(&self) -> [u32; 3] {
        [self.grain.to_bits(), self.patchiness.to_bits(), self.patch_size.to_bits()]
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
}

struct Spring {
    a: usize,
    b: usize,
    /// Its resting length now, and when it was made (field pixels).
    rest: f32,
    made: f32,
    /// How stiff and strong the rock is here (1 = ordinary).
    strength: f32,
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
        // gives the same run.) Spot (i, j) is at ((i + ½) × grain, (j + ½) × grain).
        let mut needed: BTreeSet<(u8, i64, i64)> = BTreeSet::new();
        for (index, here) in plates.plate.iter().enumerate() {
            let Some(p) = *here else { continue };
            let (x, y) = ((index % width) as f32 + 0.5, (index / width) as f32 + 0.5);
            let (i, j) = ((x / grain - 0.5).floor() as i64, (y / grain - 0.5).floor() as i64);
            for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                needed.insert((p, i + di, j + dj));
            }
        }

        // The points.
        let (mut at, mut velocity, mut plate, mut land, mut real, mut strength) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut point_at: HashMap<(u8, i64, i64), usize> = HashMap::new();
        let patch = (settings.patch_size / blocks_per_pixel).max(1.0);
        for &(p, i, j) in &needed {
            let Some(motion) = motions.get(p as usize) else { continue };
            let spot = ((i as f32 + 0.5) * grain, (j as f32 + 0.5) * grain);
            let (col, row) = (spot.0.floor() as i64, spot.1.floor() as i64);
            let on_it = col >= 0 && row >= 0 && (col as usize) < width && (row as usize) < width && plates.plate[row as usize * width + col as usize] == Some(p);
            let v = motion.at(plates.kept[p as usize].middle, spot, blocks_per_pixel);
            point_at.insert((p, i, j), at.len());
            at.push(spot);
            velocity.push((v.0 / blocks_per_pixel, v.1 / blocks_per_pixel));
            plate.push(p);
            land.push(plates.kept[p as usize].land);
            real.push(on_it);
            let noise = perlin(seed, 0x5715_f0e5, Fade::Smooth, spot.0 / patch, spot.1 / patch) * 1.4;
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
                spring_between.insert((a.min(b), a.max(b)), springs.len());
                springs.push(Spring { a, b, rest: length, made: length, strength: (strength[a] + strength[b]) / 2.0, broken: false });
            }
        }

        // The triangles: each grid square makes two (or one, if a corner is missing).
        let mut triangles = Vec::new();
        let side = |a: usize, b: usize| spring_between.get(&(a.min(b), a.max(b))).copied();
        for &(p, i, j) in &needed {
            {
                let corner = |di: i64, dj: i64| point_at.get(&(p, i + di, j + dj)).copied();
                let (c00, c10, c01, c11) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
                let threes: Vec<[usize; 3]> = match (c00, c10, c01, c11) {
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
                if length >= self.grain {
                    continue;
                }
                let (wi, wj) = (lightness[i], lightness[j]);
                let apart = push * (self.grain - length) / length / (wi + wj);
                self.at[i] = (a.0 + dx * apart * wi, a.1 + dy * apart * wi);
                self.at[j] = (b.0 - dx * apart * wj, b.1 - dy * apart * wj);
            }
        }

        // 4. Friction: touching points drag each other along (the part of their movement past
        // each other is shared out).
        let friction = settings.friction.clamp(0.0, 1.0);
        for &(i, j) in &pairs {
            let (a, b) = (self.at[i], self.at[j]);
            let (dx, dy) = (a.0 - b.0, a.1 - b.1);
            let length = dx.hypot(dy).max(1e-6);
            if length > self.grain * 1.05 {
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
            if between_real && length / spring.made - 1.0 > settings.crack * spring.strength {
                spring.broken = true;
                continue;
            }
            let strain = length / spring.rest - 1.0;
            let give = settings.give_way * spring.strength;
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
            let ratio = (triangle.area.abs() / now).clamp(0.25, 4.0);
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
    pub fn paint(&self, smoothing: u32, tear_open: f32) -> Vec<Option<Spot>> {
        let width = self.width;
        let mut painted = vec![None; width * width];
        let thickness = self.thickness(smoothing, tear_open);
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
                    painted[row * width + col] = Some(Spot { plate: self.plate[a], squeeze, from: from as u32 });
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
        let highest = sim.paint(0, f32::MAX).iter().flatten().map(|spot| spot.squeeze).fold(0.0, f32::max);
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
        let painted = sim.paint(0, f32::MAX);
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
        let painted = sim.paint(0, 2.0);
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
        let painted: Vec<Option<u8>> = sim.paint(0, f32::MAX).iter().map(|here| here.map(|spot| spot.plate)).collect();
        let wrong = painted.iter().zip(&made.plate).filter(|(a, b)| a != b).count();
        assert_eq!(wrong, 0, "{wrong} pixels differ from the plates as made");
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
            sim.paint(0, f32::MAX)
        };
        assert_eq!(run(), run());
    }
}

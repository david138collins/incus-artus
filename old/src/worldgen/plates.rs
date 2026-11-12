//! Plates — the first World Gen tool: the big pieces of land the continents will grow from.
//!
//! **The world is the circle** inside the square field (the square is only there because pictures
//! are square). Past the circle is the edge of the world.
//!
//! **How the plates are found:**
//! 1. Scatter points over the field, and split it into Voronoi cells: every pixel belongs to its
//!    nearest point.
//! 2. **Spread them evenly** (optional, a few rounds): move each point to the middle of its cell
//!    and split again. Cells get rounder and more even in size, more like plates. (This is Lloyd's
//!    relaxation.)
//! 3. **Delete** every cell that reaches past the circle: the ones outside and the ones touching
//!    the edge.
//! 4. **Keep the cells nearest the middle** of the world, as many as there should be plates.
//!
//! How many points are scattered comes from the number of plates and how much of the world they
//! should cover: 7 plates covering 30% of the world's circle means each cell is about 30% ÷ 7 of
//! it, which sets how many fit across the field. The coverage you get is close to the target but
//! not exact (cells vary in size); the readout shows what came out.
//!
//! **Wobbly edges.** Straight Voronoi edges look like cut glass. So, before deleting and keeping:
//! - **Find the vertices**, where three cells meet. They show up in the pixel map as a 2 × 2
//!   square holding three different cells; the exact spot is the one point the same distance from
//!   all three cells' points (the centre of the circle through them).
//! - **Find the edges**: each line between two cells runs from one vertex to another.
//! - **Wobble each edge**: push the line sideways (perpendicular to itself). The push is nothing at
//!   both ends, so the vertices stay put and the edges still meet. Pixels near a line go to
//!   whichever side of the wobbly line they're on.
//!
//! There are **two line mutators**, added together: **large waves** and **small roughness**. Each
//! works one of two ways:
//! - **Perlin noise** running along the line (faded to nothing at the ends): smooth, rolling.
//! - **Midpoint displacement**: split the line in half and nudge the middle sideways by a random
//!   amount; split each half and nudge their middles by less (times the roughness); and so on down
//!   to the finest step. Jagged and coastline-like, and the ends never move.
//!
//! **Ocean plates.** The cells touching the land plates become ocean plates (a ring around the
//! land, or more rings). They're plates too: they move, and where they meet land they make the coast
//! types. Cells past them stay plain, empty ocean. **Ocean plates among the land** [David,
//! 2026-10-25]: some of the cells kept nearest the middle are made ocean instead, the ones most
//! surrounded by the others, so they get caught in the land as it closes in and bite bays and
//! inland seas out of it: a continent shaped less like a blob, more like a letter. Picked from the
//! land's edge instead (past the middle of "inside ← from → edge"), they're always touching one
//! another, so they bite one gulf (a C, H or E) rather than ringing a round middle (an O).
//!
//! Only the vertices and lines that touch a plate are kept (the ones out in the empty ocean are
//! culled).

use super::noise::{Fade, perlin};
use crate::fate::planet_hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlateSettings {
    /// How many plates to keep.
    pub plates: u32,
    /// How much of the world's circle they should cover (0 to 1).
    pub coverage: f32,
    /// Rounds of spreading the points evenly (0 = as scattered).
    pub spread: u32,
    /// Rings of ocean plates around the land plates (0 = none).
    pub ocean_rings: u32,
    /// Ocean plates in among the land plates, caught in the middle of them (0 = none).
    pub ocean_among_land: u32,
    /// Where those come from: 0 = the cells most surrounded by land (inland seas), 1 = cells on the
    /// outside edge of the land (bites into the coast, bays).
    pub ocean_edge: f32,

    /// The two line mutators, added together.
    pub waves: LineNoise,
    pub rough: LineNoise,
}

impl Default for PlateSettings {
    fn default() -> Self {
        Self {
            plates: 7,
            coverage: 0.30,
            spread: 2,
            ocean_rings: 1,
            ocean_among_land: 0,
            ocean_edge: 0.0,
            waves: LineNoise { on: true, style: LineStyle::Perlin, amount: 3_000.0, size: 15_000.0, octaves: 2, roughness: 0.5 },
            rough: LineNoise { on: true, style: LineStyle::Midpoint, amount: 800.0, size: 1_000.0, octaves: 3, roughness: 0.55 },
        }
    }
}

/// How a line mutator pushes a line sideways.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyle {
    /// Perlin noise running along the line: smooth, rolling.
    #[default]
    Perlin,
    /// Halve the line again and again, nudging each new middle sideways: jagged.
    Midpoint,
}

/// One line mutator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LineNoise {
    pub on: bool,
    pub style: LineStyle,
    /// How far the line may be pushed sideways, in blocks. (Midpoint: the first nudge, at the
    /// line's middle.)
    pub amount: f32,
    /// Perlin: how long the biggest wobbles are, along the line. Midpoint: the finest step, where
    /// the halving stops. In blocks.
    pub size: f32,
    /// Perlin only: finer, fainter wobbles stacked on top.
    pub octaves: u32,
    /// How strong each finer wobble (Perlin) or each smaller nudge (midpoint) is compared with the
    /// one before.
    pub roughness: f32,
}

impl Default for LineNoise {
    fn default() -> Self {
        Self { on: true, style: LineStyle::Perlin, amount: 1_500.0, size: 6_000.0, octaves: 3, roughness: 0.5 }
    }
}

impl PlateSettings {
    /// How many points are scattered over the whole square field. The circle is π/4 of the
    /// square, so each cell should be `coverage × π/4 ÷ plates` of the square.
    pub fn points(&self) -> usize {
        let plates = self.plates.max(1) as f32;
        let each = self.coverage.clamp(0.01, 1.0) * std::f32::consts::FRAC_PI_4 / plates;
        ((1.0 / each).round() as usize).max(self.plates as usize + 1)
    }
}

pub struct Plate {
    /// Which cell it is.
    pub cell: usize,
    /// A land plate, or an ocean plate (in the rings around them, or among them).
    pub land: bool,
    pub pixels: usize,
    /// Its share of the world's circle.
    pub share: f32,
    /// The middle of its cell, in field pixels.
    pub middle: (f32, f32),
}

/// Where three cells meet, in field pixels, and which three.
pub struct Vertex {
    pub at: (f32, f32),
    pub cells: [usize; 3],
}

/// A line between two cells, from one vertex to another (straight, before wobbling).
pub struct Edge {
    /// The two cells, lower number first. A push is measured toward the first.
    pub cells: (usize, usize),
    pub from: (f32, f32),
    pub to: (f32, f32),
}

pub struct Plates {
    pub width: usize,
    /// The pixel size they were made at.
    pub blocks_per_pixel: f32,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    /// Which cell each pixel is in.
    pub cell: Vec<u16>,
    /// Which plate each pixel is on (an index into `kept`), if any.
    pub plate: Vec<Option<u8>>,
    /// Whether each pixel is inside the world's circle.
    pub inside: Vec<bool>,
    /// The land plates first, then the ocean plates.
    pub kept: Vec<Plate>,
    /// How many cells there were, and how many reached past the circle (and were deleted).
    pub cells: usize,
    pub past_the_edge: usize,
    /// How much of the world's circle the kept plates cover.
    pub coverage: f32,
}

/// A random number from 0 to 1.
fn unit(seed: u64, k: usize, which: i64) -> f32 {
    (planet_hash(seed, 0x91a7e5, k as i64, which, 0) >> 40) as f32 / (1u64 << 24) as f32
}

/// Plate `p`'s own random number for one spread setting (`which`), from −1 to 1. Each setting has
/// its own, so a plate that's high isn't always stiff too.
pub fn plate_random(seed: u64, p: usize, which: i64) -> f32 {
    (planet_hash(seed, 0x5b7ead, p as i64, which, 0) >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
}

/// A value spread per plate: `spread` 1 gives each plate between half and twice the value, 2
/// between a quarter and four times (evenly in between, on a doubling scale); 0 = all the same.
pub fn spread_by(value: f32, spread: f32, random: f32) -> f32 {
    value * (spread * random).exp2()
}

/// For each pixel, its nearest point, using all the computer's cores.
fn split(points: &[(f32, f32)], width: usize, cell: &mut [u16]) {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in cell.chunks_mut(rows_each * width).enumerate() {
            scope.spawn(move || {
                for (i, nearest) in chunk.iter_mut().enumerate() {
                    let (x, y) = ((i % width) as f32 + 0.5, (band * rows_each + i / width) as f32 + 0.5);
                    let mut best = f32::MAX;
                    for (k, &(px, py)) in points.iter().enumerate() {
                        let d = (px - x) * (px - x) + (py - y) * (py - y);
                        if d < best {
                            best = d;
                            *nearest = k as u16;
                        }
                    }
                }
            });
        }
    });
}

/// The centre of the circle through three points (the one spot the same distance from all
/// three), or `None` if they're in a line.
fn circle_centre(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> Option<(f32, f32)> {
    let d = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
    if d.abs() < 1e-6 {
        return None;
    }
    let (a2, b2, c2) = (a.0 * a.0 + a.1 * a.1, b.0 * b.0 + b.1 * b.1, c.0 * c.0 + c.1 * c.1);
    Some(((a2 * (b.1 - c.1) + b2 * (c.1 - a.1) + c2 * (a.1 - b.1)) / d, (a2 * (c.0 - b.0) + b2 * (a.0 - c.0) + c2 * (b.0 - a.0)) / d))
}

/// The vertices: wherever a 2 × 2 square of pixels holds three different cells, the exact spot
/// where those three meet. (A spot only counts if no other cell's point is nearer to it.)
pub fn find_vertices(points: &[(f32, f32)], cell: &[u16], width: usize) -> Vec<Vertex> {
    let mut found: HashSet<[usize; 3]> = HashSet::new();
    let mut vertices = Vec::new();
    for row in 0..width.saturating_sub(1) {
        for col in 0..width - 1 {
            let mut here: Vec<usize> =
                [(col, row), (col + 1, row), (col, row + 1), (col + 1, row + 1)].iter().map(|&(c, r)| cell[r * width + c] as usize).collect();
            here.sort_unstable();
            here.dedup();
            if here.len() < 3 {
                continue;
            }
            // Usually three cells; where four meet, try each three of them.
            for skip in 0..here.len() {
                let three: Vec<usize> = here.iter().enumerate().filter(|&(i, _)| i != skip || here.len() == 3).map(|(_, &c)| c).collect();
                let Ok(three) = <[usize; 3]>::try_from(&three[..3]) else { continue };
                if !found.insert(three) {
                    continue;
                }
                let Some(at) = circle_centre(points[three[0]], points[three[1]], points[three[2]]) else { continue };
                let reach = (points[three[0]].0 - at.0).hypot(points[three[0]].1 - at.1);
                let nearer = points
                    .iter()
                    .enumerate()
                    .any(|(k, p)| !three.contains(&k) && (p.0 - at.0).hypot(p.1 - at.1) < reach * 0.999);
                if !nearer {
                    vertices.push(Vertex { at, cells: three });
                }
                if here.len() == 3 {
                    break;
                }
            }
        }
    }
    vertices
}

/// The edges: each pair of cells that shares two vertices has a line between them.
pub fn find_edges(vertices: &[Vertex]) -> Vec<Edge> {
    let mut ends: HashMap<(usize, usize), Vec<(f32, f32)>> = HashMap::new();
    for vertex in vertices {
        let [a, b, c] = vertex.cells;
        for pair in [(a, b), (a, c), (b, c)] {
            ends.entry((pair.0.min(pair.1), pair.0.max(pair.1))).or_default().push(vertex.at);
        }
    }
    let mut edges: Vec<Edge> = ends
        .into_iter()
        .filter_map(|(cells, ends)| {
            // Normally two ends. (Rarely more, where four cells meet almost at one spot: take the
            // two furthest apart.)
            let mut best: Option<((f32, f32), (f32, f32), f32)> = None;
            for (i, &from) in ends.iter().enumerate() {
                for &to in &ends[i + 1..] {
                    let length = (to.0 - from.0).hypot(to.1 - from.1);
                    if best.is_none_or(|(_, _, longest)| length > longest) {
                        best = Some((from, to, length));
                    }
                }
            }
            best.filter(|&(_, _, length)| length > 1e-3).map(|(from, to, _)| Edge { cells, from, to })
        })
        .collect();
    edges.sort_by_key(|edge| edge.cells);
    edges
}

/// Each edge gets its own noise (its own salt, from its two cells), so no two edges wobble alike.
fn edge_salt(edge: &Edge) -> u64 {
    (edge.cells.0 as u64) << 24 | (edge.cells.1 as u64) << 4
}

/// Midpoint displacement along one edge `length` blocks long: the push (in blocks) at evenly
/// spaced steps from one end (0) to the other (0).
fn midpoint_profile(noise: &LineNoise, seed: u64, salt: u64, length: f32) -> Vec<f32> {
    let levels = ((length / noise.size.max(1.0)).log2().ceil() as i32).clamp(1, 12) as u32;
    let n = 1usize << levels;
    let mut push = vec![0.0f32; n + 1];
    let (mut step, mut nudge) = (n, noise.amount);
    let mut level = 0;
    while step > 1 {
        let half = step / 2;
        for i in (half..n).step_by(step) {
            let random = (planet_hash(seed, salt, i as i64, level, 0) >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0;
            push[i] = (push[i - half] + push[i + half]) / 2.0 + random * nudge;
        }
        nudge *= noise.roughness;
        step = half;
        level += 1;
    }
    push
}

/// One mutator, ready to use on every edge (midpoint profiles are worked out once per edge).
struct Mutator<'a> {
    noise: &'a LineNoise,
    seed: u64,
    profiles: Vec<Vec<f32>>,
}

impl<'a> Mutator<'a> {
    fn new(noise: &'a LineNoise, seed: u64, edges: &[Edge], blocks_per_pixel: f32) -> Self {
        let profiles = match noise.style {
            LineStyle::Midpoint => edges.iter().map(|edge| midpoint_profile(noise, seed, edge_salt(edge), edge.length() * blocks_per_pixel)).collect(),
            LineStyle::Perlin => Vec::new(),
        };
        Self { noise, seed, profiles }
    }

    /// The push in blocks at `t`, from 0 at one vertex to 1 at the other.
    fn push(&self, edge: &Edge, index: usize, t: f32, blocks_per_pixel: f32) -> f32 {
        match self.noise.style {
            LineStyle::Perlin => {
                let along = t * edge.length() * blocks_per_pixel / self.noise.size.max(1.0);
                let (mut total, mut strength, mut strengths, mut scale) = (0.0, 1.0, 0.0, 1.0);
                for octave in 0..self.noise.octaves.max(1) {
                    total += perlin(self.seed, edge_salt(edge) | octave as u64, Fade::Smooth, along * scale, 0.37) * strength;
                    strengths += strength;
                    strength *= self.noise.roughness;
                    scale *= 2.0;
                }
                // Fade to nothing at both vertices, so the edges still meet there.
                total / strengths * (t * std::f32::consts::PI).sin() * self.noise.amount
            }
            LineStyle::Midpoint => {
                let profile = &self.profiles[index];
                let at = t * (profile.len() - 1) as f32;
                let i = (at.floor() as usize).min(profile.len() - 2);
                profile[i] + (profile[i + 1] - profile[i]) * (at - i as f32)
            }
        }
    }
}

impl Edge {
    /// In field pixels.
    fn length(&self) -> f32 {
        (self.to.0 - self.from.0).hypot(self.to.1 - self.from.1)
    }
}

/// Moves pixels near each edge to whichever side of the wobbly line they're on.
fn wobble(points: &[(f32, f32)], width: usize, cell: &mut [u16], edges: &[Edge], seed: u64, settings: &PlateSettings, blocks_per_pixel: f32) {
    let mutators: Vec<Mutator> = [(&settings.waves, "waves"), (&settings.rough, "roughness")]
        .into_iter()
        .filter(|(noise, _)| noise.on && noise.amount > 0.0)
        .map(|(noise, name)| Mutator::new(noise, crate::fate::derive_seed(seed, name), edges, blocks_per_pixel))
        .collect();
    if mutators.is_empty() {
        return;
    }
    let edge_of: HashMap<(usize, usize), usize> = edges.iter().enumerate().map(|(i, edge)| (edge.cells, i)).collect();
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in cell.chunks_mut(rows_each * width).enumerate() {
            let (edge_of, mutators) = (&edge_of, &mutators);
            scope.spawn(move || {
                for (i, nearest) in chunk.iter_mut().enumerate() {
                    let p = ((i % width) as f32 + 0.5, (band * rows_each + i / width) as f32 + 0.5);
                    // The nearest and second-nearest points: the pixel sits by the edge between them.
                    let (mut first, mut second) = ((f32::MAX, 0usize), (f32::MAX, 0usize));
                    for (k, &(px, py)) in points.iter().enumerate() {
                        let d = (px - p.0) * (px - p.0) + (py - p.1) * (py - p.1);
                        if d < first.0 {
                            (second, first) = (first, (d, k));
                        } else if d < second.0 {
                            second = (d, k);
                        }
                    }
                    let (a, b) = (first.1, second.1);
                    let Some(&index) = edge_of.get(&(a.min(b), a.max(b))) else { continue };
                    let edge = &edges[index];
                    // How far the pixel is from the straight edge, into its own cell's side.
                    let gap = (points[b].0 - points[a].0).hypot(points[b].1 - points[a].1).max(1e-6);
                    let into_own = (second.0 - first.0) / (2.0 * gap);
                    // How far along the edge it is.
                    let (dx, dy) = (edge.to.0 - edge.from.0, edge.to.1 - edge.from.1);
                    let t = (((p.0 - edge.from.0) * dx + (p.1 - edge.from.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
                    let pushed: f32 = mutators.iter().map(|m| m.push(edge, index, t, blocks_per_pixel)).sum::<f32>() / blocks_per_pixel;
                    // A push toward the edge's first cell takes ground from it, and gives ground to
                    // it when negative.
                    let toward_own = if a == edge.cells.0 { pushed } else { -pushed };
                    if into_own < toward_own {
                        *nearest = b as u16;
                    }
                }
            });
        }
    });
}

/// The middle of each cell and how many pixels it has.
fn middles(cell: &[u16], width: usize, count: usize) -> Vec<(f32, f32, usize)> {
    let mut sums = vec![(0.0f64, 0.0f64, 0usize); count];
    for (i, &c) in cell.iter().enumerate() {
        let sum = &mut sums[c as usize];
        sum.0 += (i % width) as f64 + 0.5;
        sum.1 += (i / width) as f64 + 0.5;
        sum.2 += 1;
    }
    sums.into_iter().map(|(x, y, n)| (x as f32 / n.max(1) as f32, y as f32 / n.max(1) as f32, n)).collect()
}

/// Which cells touch which, from the pixel map (pixels side by side in different cells).
fn neighbours(cell: &[u16], width: usize, count: usize) -> Vec<Vec<usize>> {
    let mut touching: HashSet<(usize, usize)> = HashSet::new();
    for row in 0..width {
        for col in 0..width {
            let here = cell[row * width + col] as usize;
            for (c, r) in [(col + 1, row), (col, row + 1)] {
                if c < width && r < width {
                    let there = cell[r * width + c] as usize;
                    if there != here {
                        touching.insert((here.min(there), here.max(there)));
                    }
                }
            }
        }
    }
    let mut neighbours = vec![Vec::new(); count];
    for (a, b) in touching {
        neighbours[a].push(b);
        neighbours[b].push(a);
    }
    neighbours
}

/// Finds the plates. Returns `None` if `stop` was set part-way (a newer version is wanted).
pub fn make(settings: &PlateSettings, seed: u64, size_blocks: f32, blocks_per_pixel: f32, stop: &AtomicBool) -> Option<Plates> {
    let width = (size_blocks / blocks_per_pixel).ceil().max(1.0) as usize;
    // The field in pixels (the last pixel may run past the edge) and the world's circle.
    let across = size_blocks / blocks_per_pixel;
    let (centre, radius) = (across / 2.0, across / 2.0);

    let count = settings.points().min(u16::MAX as usize);
    let mut points: Vec<(f32, f32)> = (0..count).map(|k| (unit(seed, k, 0) * across, unit(seed, k, 1) * across)).collect();
    let mut cell = vec![0u16; width * width];
    split(&points, width, &mut cell);
    for _ in 0..settings.spread {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        for (point, (x, y, n)) in points.iter_mut().zip(middles(&cell, width, count)) {
            if n > 0 {
                *point = (x, y);
            }
        }
        split(&points, width, &mut cell);
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // The vertices and edges of the straight cells, then the edges wobbled.
    let mut vertices = find_vertices(&points, &cell, width);
    let mut edges = find_edges(&vertices);
    let edge_seed = crate::fate::derive_seed(seed, "plate edges");
    wobble(&points, width, &mut cell, &edges, edge_seed, settings, blocks_per_pixel);

    // Which pixels are in the world, and which cells reach past it.
    let inside: Vec<bool> = (0..width * width)
        .map(|i| {
            let (x, y) = ((i % width) as f32 + 0.5 - centre, (i / width) as f32 + 0.5 - centre);
            x * x + y * y <= radius * radius
        })
        .collect();
    let mut past_edge = vec![false; count];
    for (i, &c) in cell.iter().enumerate() {
        if !inside[i] {
            past_edge[c as usize] = true;
        }
    }

    // Keep the cells nearest the middle.
    let middles = middles(&cell, width, count);
    let mut candidates: Vec<usize> = (0..count).filter(|&c| !past_edge[c] && middles[c].2 > 0).collect();
    let from_middle = |c: usize| (middles[c].0 - centre).hypot(middles[c].1 - centre);
    candidates.sort_by(|&a, &b| from_middle(a).total_cmp(&from_middle(b)));
    candidates.truncate((settings.plates + settings.ocean_among_land) as usize);
    let neighbours = neighbours(&cell, width, count);

    // Ocean plates among the land: of the cells kept, those most surrounded by the others (with a
    // little chance mixed in) become ocean, so they're caught in the middle of the land and bite
    // into it instead of rounding it off.
    let among = (settings.ocean_among_land as usize).min(candidates.len().saturating_sub(1));
    if among > 0 {
        let cluster: HashSet<usize> = candidates.iter().copied().collect();
        // Toward the edge (David, 2026-10-27), the least surrounded are picked instead: cells on the
        // land group's outside, biting bays into its coast.
        let toward_inside = 1.0 - 2.0 * settings.ocean_edge.clamp(0.0, 1.0);
        let surrounded = |c: usize| neighbours[c].iter().filter(|n| cluster.contains(n)).count() as f32 * toward_inside + unit(seed ^ 0x0cea, c, 0) * 2.0;
        let mut by_surrounded = candidates.clone();
        by_surrounded.sort_by(|&a, &b| surrounded(b).total_cmp(&surrounded(a)));
        // Toward the edge, each one after the first is the best of those touching the ones already
        // picked, so together they make one bite: a C, H or E, never a ring of bites round a round
        // middle (an O) [David, 2026-10-27: "they should always be touching one another"].
        let picked = if settings.ocean_edge > 0.5 {
            let mut left = by_surrounded.clone();
            let mut picked = vec![left.remove(0)];
            while picked.len() < among {
                let next = left.iter().position(|c| picked.iter().any(|p| neighbours[*p].contains(c))).unwrap_or(0);
                picked.push(left.remove(next));
            }
            picked
        } else {
            by_surrounded[..among].to_vec()
        };
        let sea: HashSet<usize> = picked.into_iter().collect();
        // The land plates first, then the ocean ones (in the order they were kept).
        let (land_cells, sea_cells): (Vec<usize>, Vec<usize>) = candidates.iter().partition(|c| !sea.contains(c));
        candidates = land_cells.into_iter().chain(sea_cells).collect();
    }
    let land = candidates.len() - among;

    // The rings of ocean plates: each ring is the cells touching the ones before (and not
    // reaching past the edge).
    let mut taken = vec![false; count];
    for &c in &candidates {
        taken[c] = true;
    }
    let mut ring_start = 0;
    for _ in 0..settings.ocean_rings {
        let mut ring: Vec<usize> = candidates[ring_start..]
            .iter()
            .flat_map(|&c| neighbours[c].iter().copied())
            .filter(|&n| !past_edge[n] && middles[n].2 > 0)
            .collect();
        ring.sort_by(|&a, &b| from_middle(a).total_cmp(&from_middle(b)));
        ring.retain(|&n| !std::mem::replace(&mut taken[n], true));
        ring_start = candidates.len();
        candidates.extend(ring);
    }
    // A plate number has to fit in a byte.
    candidates.truncate(u8::MAX as usize);

    let world = inside.iter().filter(|&&i| i).count().max(1);
    let kept: Vec<Plate> = candidates
        .iter()
        .enumerate()
        .map(|(k, &c)| Plate {
            cell: c,
            land: k < land,
            pixels: middles[c].2,
            share: middles[c].2 as f32 / world as f32,
            middle: (middles[c].0, middles[c].1),
        })
        .collect();
    let mut plate_of_cell = vec![None; count];
    for (p, plate) in kept.iter().enumerate() {
        plate_of_cell[plate.cell] = Some(p as u8);
    }
    let plate = cell.iter().map(|&c| plate_of_cell[c as usize]).collect();
    // Cull the vertices and lines out in the empty ocean: keep only those touching a plate.
    let on_a_plate = |c: &usize| plate_of_cell[*c].is_some();
    vertices.retain(|vertex| vertex.cells.iter().any(on_a_plate));
    edges.retain(|edge| on_a_plate(&edge.cells.0) || on_a_plate(&edge.cells.1));
    // Coverage counts the land plates only.
    let coverage = kept.iter().filter(|plate| plate.land).map(|plate| plate.share).sum();

    Some(Plates {
        width,
        blocks_per_pixel,
        vertices,
        edges,
        cell,
        plate,
        inside,
        kept,
        cells: count,
        past_the_edge: past_edge.iter().filter(|&&p| p).count(),
        coverage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plates(settings: &PlateSettings, seed: u64) -> Plates {
        make(settings, seed, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap()
    }

    #[test]
    fn no_plate_reaches_past_the_edge_of_the_world() {
        for seed in 0..5 {
            let made = plates(&PlateSettings::default(), seed);
            for (i, plate) in made.plate.iter().enumerate() {
                if plate.is_some() {
                    assert!(made.inside[i], "seed {seed}: a plate pixel is outside the world");
                }
            }
        }
    }

    #[test]
    fn it_keeps_as_many_plates_as_asked_and_roughly_the_coverage() {
        let settings = PlateSettings::default();
        let made = plates(&settings, 3);
        assert_eq!(made.kept.iter().filter(|plate| plate.land).count(), 7);
        assert!((0.15..0.45).contains(&made.coverage), "coverage {}", made.coverage);
    }

    #[test]
    fn same_seed_same_plates() {
        let settings = PlateSettings::default();
        assert_eq!(plates(&settings, 8).plate, plates(&settings, 8).plate);
        assert_ne!(plates(&settings, 8).plate, plates(&settings, 9).plate);
    }

    #[test]
    fn each_vertex_is_the_same_distance_from_its_three_cells_points() {
        let points = [(10.0, 10.0), (30.0, 12.0), (20.0, 30.0), (45.0, 35.0), (5.0, 40.0)];
        let width = 50;
        let mut cell = vec![0u16; width * width];
        split(&points, width, &mut cell);
        let vertices = find_vertices(&points, &cell, width);
        assert!(!vertices.is_empty());
        for vertex in &vertices {
            let d = vertex.cells.map(|c| (points[c].0 - vertex.at.0).hypot(points[c].1 - vertex.at.1));
            assert!((d[0] - d[1]).abs() < 1e-3 && (d[1] - d[2]).abs() < 1e-3, "{d:?}");
        }
        // Every edge runs between two of the vertices.
        for edge in find_edges(&vertices) {
            assert!(edge.cells.0 < edge.cells.1);
        }
    }

    #[test]
    fn wobbling_moves_the_borders_but_keeps_the_rules() {
        let off = |noise: LineNoise| LineNoise { on: false, ..noise };
        let defaults = PlateSettings::default();
        let straight = plates(&PlateSettings { waves: off(defaults.waves.clone()), rough: off(defaults.rough.clone()), ..defaults.clone() }, 4);
        for style in [LineStyle::Perlin, LineStyle::Midpoint] {
            let waves = LineNoise { style, amount: 3_000.0, ..defaults.waves.clone() };
            let wobbly = plates(&PlateSettings { waves, ..defaults.clone() }, 4);
            assert_ne!(straight.cell, wobbly.cell, "the wobble should move some pixels");
            for (i, plate) in wobbly.plate.iter().enumerate() {
                if plate.is_some() {
                    assert!(wobbly.inside[i]);
                }
            }
        }
    }

    #[test]
    fn only_vertices_and_lines_touching_a_plate_are_kept() {
        let made = plates(&PlateSettings::default(), 6);
        assert!(!made.vertices.is_empty());
        let on_a_plate = |c: usize| made.kept.iter().any(|plate| plate.cell == c);
        for vertex in &made.vertices {
            assert!(vertex.cells.iter().any(|&c| on_a_plate(c)));
        }
        for edge in &made.edges {
            assert!(on_a_plate(edge.cells.0) || on_a_plate(edge.cells.1));
        }
    }

    #[test]
    fn midpoint_displacement_never_moves_the_ends() {
        let noise = LineNoise { style: LineStyle::Midpoint, amount: 500.0, size: 100.0, roughness: 0.6, ..LineNoise::default() };
        let profile = midpoint_profile(&noise, 7, 3, 10_000.0);
        assert_eq!((profile[0], profile[profile.len() - 1]), (0.0, 0.0));
        assert!(profile.iter().any(|&p| p != 0.0));
        assert_eq!(profile, midpoint_profile(&noise, 7, 3, 10_000.0), "same seed, same line");
        // Halved down to the finest step: 10,000 blocks in steps of about 100.
        assert_eq!(profile.len(), 129);
    }

    #[test]
    fn the_ocean_ring_touches_the_land_and_stays_inside_the_world() {
        let made = plates(&PlateSettings::default(), 2);
        let land: Vec<usize> = made.kept.iter().filter(|plate| plate.land).map(|plate| plate.cell).collect();
        let ocean: Vec<usize> = made.kept.iter().filter(|plate| !plate.land).map(|plate| plate.cell).collect();
        assert!(!ocean.is_empty());
        // The land plates come first.
        assert!(made.kept[..land.len()].iter().all(|plate| plate.land));
        let touching = neighbours(&made.cell, made.width, made.cells);
        for &c in &ocean {
            assert!(touching[c].iter().any(|n| land.contains(n)), "ocean plate cell {c} doesn't touch land");
        }
        for (i, plate) in made.plate.iter().enumerate() {
            if plate.is_some() {
                assert!(made.inside[i]);
            }
        }
        // No ring, no ocean plates.
        let none = plates(&PlateSettings { ocean_rings: 0, ..PlateSettings::default() }, 2);
        assert!(none.kept.iter().all(|plate| plate.land));
    }

    #[test]
    fn ocean_plates_among_the_land_are_caught_between_land_plates() {
        for seed in 1..=4 {
            let settings = PlateSettings { plates: 14, ocean_rings: 0, ocean_among_land: 3, ..PlateSettings::default() };
            let made = plates(&settings, seed);
            let land: Vec<usize> = made.kept.iter().filter(|plate| plate.land).map(|plate| plate.cell).collect();
            let ocean: Vec<usize> = made.kept.iter().filter(|plate| !plate.land).map(|plate| plate.cell).collect();
            // As many land plates as asked for, the land first, and three ocean plates.
            assert_eq!((land.len(), ocean.len()), (14, 3), "seed {seed}");
            assert!(made.kept[..land.len()].iter().all(|plate| plate.land));
            // Each is in among the land: touching at least three land plates.
            let touching = neighbours(&made.cell, made.width, made.cells);
            for &c in &ocean {
                let around = touching[c].iter().filter(|n| land.contains(n)).count();
                assert!(around >= 3, "seed {seed}: ocean plate cell {c} touches only {around} land plates");
            }
        }
    }

    #[test]
    fn ocean_plates_toward_the_edge_touch_fewer_land_plates() {
        // Over a few seeds, ocean plates picked from the land group's edge are less surrounded by
        // land than ones picked from its inside.
        let touching_land = |edge: f32| -> usize {
            (1..=6)
                .map(|seed| {
                    let made = plates(&PlateSettings { plates: 14, ocean_rings: 0, ocean_among_land: 3, ocean_edge: edge, ..PlateSettings::default() }, seed);
                    let land: Vec<usize> = made.kept.iter().filter(|plate| plate.land).map(|plate| plate.cell).collect();
                    let touching = neighbours(&made.cell, made.width, made.cells);
                    made.kept.iter().filter(|plate| !plate.land).map(|plate| touching[plate.cell].iter().filter(|n| land.contains(n)).count()).sum::<usize>()
                })
                .sum()
        };
        let (inside, edge) = (touching_land(0.0), touching_land(1.0));
        assert!(edge < inside, "edge {edge}, inside {inside}");
    }

    #[test]
    fn ocean_plates_toward_the_edge_touch_one_another() {
        for seed in 1..=8 {
            let made = plates(&PlateSettings { plates: 14, ocean_rings: 0, ocean_among_land: 4, ocean_edge: 1.0, ..PlateSettings::default() }, seed);
            let ocean: Vec<usize> = made.kept.iter().filter(|plate| !plate.land).map(|plate| plate.cell).collect();
            assert_eq!(ocean.len(), 4, "seed {seed}");
            // One group: every ocean plate can be reached from the first through ocean plates.
            let touching = neighbours(&made.cell, made.width, made.cells);
            let mut reached = vec![ocean[0]];
            let mut k = 0;
            while k < reached.len() {
                let c = reached[k];
                for &n in &touching[c] {
                    if ocean.contains(&n) && !reached.contains(&n) {
                        reached.push(n);
                    }
                }
                k += 1;
            }
            assert_eq!(reached.len(), ocean.len(), "seed {seed}: ocean plates {ocean:?} aren't one group");
        }
    }

    #[test]
    fn more_coverage_means_bigger_cells() {
        let small = PlateSettings { coverage: 0.2, ..PlateSettings::default() };
        let big = PlateSettings { coverage: 0.5, ..PlateSettings::default() };
        assert!(small.points() > big.points());
    }
}

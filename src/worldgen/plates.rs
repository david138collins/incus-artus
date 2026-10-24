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
//! - **Wobble each edge**: push the line sideways (perpendicular to itself) by Perlin noise running
//!   along it. The push fades to nothing at both ends, so the vertices stay put and the edges still
//!   meet. Pixels near a line go to whichever side of the wobbly line they're on.

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

    // ── Wobbly edges ──
    /// How far an edge may be pushed sideways, in blocks (0 = straight edges).
    pub wobble: f32,
    /// The size of the biggest wobbles along an edge, in blocks.
    pub wobble_size: f32,
    /// Finer, fainter wobbles stacked on top (as noise octaves).
    pub wobble_octaves: u32,
    /// How strong each finer wobble is compared with the one before.
    pub wobble_roughness: f32,
}

impl Default for PlateSettings {
    fn default() -> Self {
        Self { plates: 7, coverage: 0.30, spread: 2, wobble: 1_500.0, wobble_size: 6_000.0, wobble_octaves: 4, wobble_roughness: 0.5 }
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
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    /// Which cell each pixel is in.
    pub cell: Vec<u16>,
    /// Which plate each pixel is on (an index into `kept`), if any.
    pub plate: Vec<Option<u8>>,
    /// Whether each pixel is inside the world's circle.
    pub inside: Vec<bool>,
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

/// How far the wobbly edge is pushed sideways (in pixels, toward the edge's first cell) at `t`,
/// from 0 at one vertex to 1 at the other.
fn push(edge: &Edge, index: usize, t: f32, seed: u64, settings: &PlateSettings, blocks_per_pixel: f32) -> f32 {
    let length = (edge.to.0 - edge.from.0).hypot(edge.to.1 - edge.from.1) * blocks_per_pixel;
    let along = t * length / settings.wobble_size.max(1.0);
    let (mut total, mut strength, mut strengths, mut scale) = (0.0, 1.0, 0.0, 1.0);
    for octave in 0..settings.wobble_octaves.max(1) {
        // Each edge gets its own noise (its own salt), so no two edges wobble alike.
        let salt = (index as u64) << 8 | octave as u64;
        total += perlin(seed, salt, Fade::Smooth, along * scale, 0.37) * strength;
        strengths += strength;
        strength *= settings.wobble_roughness;
        scale *= 2.0;
    }
    // Fade to nothing at both vertices, so the edges still meet there.
    let fade = (t * std::f32::consts::PI).sin();
    total / strengths * fade * settings.wobble / blocks_per_pixel
}

/// Moves pixels near each edge to whichever side of the wobbly line they're on.
fn wobble(points: &[(f32, f32)], width: usize, cell: &mut [u16], edges: &[Edge], seed: u64, settings: &PlateSettings, blocks_per_pixel: f32) {
    if settings.wobble <= 0.0 {
        return;
    }
    let edge_of: HashMap<(usize, usize), usize> = edges.iter().enumerate().map(|(i, edge)| (edge.cells, i)).collect();
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in cell.chunks_mut(rows_each * width).enumerate() {
            let edge_of = &edge_of;
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
                    let pushed = push(edge, index, t, seed, settings, blocks_per_pixel);
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
    let vertices = find_vertices(&points, &cell, width);
    let edges = find_edges(&vertices);
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
    candidates.truncate(settings.plates as usize);

    let world = inside.iter().filter(|&&i| i).count().max(1);
    let kept: Vec<Plate> = candidates
        .iter()
        .map(|&c| Plate { cell: c, pixels: middles[c].2, share: middles[c].2 as f32 / world as f32, middle: (middles[c].0, middles[c].1) })
        .collect();
    let mut plate_of_cell = vec![None; count];
    for (p, plate) in kept.iter().enumerate() {
        plate_of_cell[plate.cell] = Some(p as u8);
    }
    let plate = cell.iter().map(|&c| plate_of_cell[c as usize]).collect();
    let coverage = kept.iter().map(|plate| plate.share).sum();

    Some(Plates {
        width,
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
        assert_eq!(made.kept.len(), 7);
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
        let straight = plates(&PlateSettings { wobble: 0.0, ..PlateSettings::default() }, 4);
        let wobbly = plates(&PlateSettings { wobble: 3_000.0, ..PlateSettings::default() }, 4);
        assert_ne!(straight.cell, wobbly.cell, "the wobble should move some pixels");
        // About two vertices per cell, as in any Voronoi diagram.
        assert!(wobbly.vertices.len() > wobbly.cells, "{} vertices for {} cells", wobbly.vertices.len(), wobbly.cells);
        for (i, plate) in wobbly.plate.iter().enumerate() {
            if plate.is_some() {
                assert!(wobbly.inside[i]);
            }
        }
    }

    #[test]
    fn more_coverage_means_bigger_cells() {
        let small = PlateSettings { coverage: 0.2, ..PlateSettings::default() };
        let big = PlateSettings { coverage: 0.5, ..PlateSettings::default() };
        assert!(small.points() > big.points());
    }
}

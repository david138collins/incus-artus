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

use crate::fate::planet_hash;
use serde::{Deserialize, Serialize};
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
}

impl Default for PlateSettings {
    fn default() -> Self {
        Self { plates: 7, coverage: 0.30, spread: 2 }
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

pub struct Plates {
    pub width: usize,
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
    fn more_coverage_means_bigger_cells() {
        let small = PlateSettings { coverage: 0.2, ..PlateSettings::default() };
        let big = PlateSettings { coverage: 0.5, ..PlateSettings::default() };
        assert!(small.points() > big.points());
    }
}

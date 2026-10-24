//! Heights — every spot on a plate has a height, and the sea is a level you slide.
//!
//! A real plate isn't all land: it's rock at some height, and the coast is wherever that height
//! crosses sea level. So each spot's height is three things added together:
//! 1. **the plate's base height**: land plates sit around sea level, ocean plates deep;
//! 2. **its shape**, a blend of two things (the **blend** slider mixes them):
//!    - **tilt** [David]: each plate balances on its middle and tips a little, a random direction
//!      and amount, so one side rises and the other sinks: a plain slope through the middle;
//!    - **terrain noise**: rolling highs and lows, so a land plate is partly above water and partly
//!      sunk. **One noise for the whole world**, so at the start it flows straight across plate
//!      boundaries (a continent can straddle two plates).
//!
//!    Both are read where each pixel *started*, so they ride, bend and squash with the rock;
//! 3. **sunken edges** (continental shelves): each plate slopes down toward its edges, so coasts
//!    fall inside plates rather than along their outlines, and plates touching at the start don't
//!    automatically make one landmass;
//!
//! — that's the **terrain**, worked out once per pixel of the plates as made — and then, while the
//! simulation runs, 4. **squeeze**: thickened rock rises, so crumple belts become mountains (and
//! squeezed ocean floor can break the surface as islands).
//!
//! The **water level** only decides what's drawn as land: it can be changed at any time without
//! changing the simulation.

use super::noise::PerlinSettings;
use super::plates::Plates;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HeightSettings {
    /// Base heights of land plates and ocean plates (0 = the usual sea level).
    pub land: f32,
    pub ocean: f32,
    /// How high and low the terrain noise goes either way.
    pub terrain: f32,
    /// The size of its biggest features, in blocks.
    pub terrain_size: f32,
    /// Finer, fainter layers on top, and how strong each is compared with the one before.
    pub octaves: u32,
    pub roughness: f32,
    /// The most a plate tips: how far its highest edge rises (and its lowest sinks), roughly.
    /// Each plate tips a random direction, by a random share of this.
    pub tilt: f32,
    /// Mixes the tilt and the terrain noise: 0 = only tilt, 1 = only noise.
    pub blend: f32,
    /// How far in from a plate's edge its ground starts sinking (blocks), and how far it sinks at
    /// the very edge.
    pub shelf_width: f32,
    pub shelf_depth: f32,
    /// How much height squeezing adds: rock twice as thick rises by this much.
    pub squeeze: f32,
    /// The water level: below it is sea.
    pub water: f32,
}

impl Default for HeightSettings {
    fn default() -> Self {
        Self {
            land: 0.05,
            ocean: -0.6,
            terrain: 0.8,
            terrain_size: 15_000.0,
            octaves: 5,
            roughness: 0.5,
            tilt: 0.8,
            blend: 0.7,
            shelf_width: 2_500.0,
            shelf_depth: 0.2,
            squeeze: 0.6,
            water: 0.0,
        }
    }
}

impl HeightSettings {
    /// The settings the terrain is worked out from (not the squeeze or the water level, which
    /// are applied as it's drawn).
    pub fn shaping(&self) -> [u32; 10] {
        [
            self.land.to_bits(),
            self.ocean.to_bits(),
            self.terrain.to_bits(),
            self.terrain_size.to_bits(),
            self.octaves,
            self.roughness.to_bits(),
            self.tilt.to_bits(),
            self.blend.to_bits(),
            self.shelf_width.to_bits(),
            self.shelf_depth.to_bits(),
        ]
    }
}

/// One plate's tilt: which way its rising side faces (a unit direction, in field pixels), how far
/// that side rises (height), and the plate's rough radius (pixels) that measures "a side".
struct Tilt {
    toward: (f32, f32),
    rise: f32,
    radius: f32,
}

/// Each plate's tilt: a random direction, and a random share (from a third to all) of the most.
fn tilts(plates: &Plates, settings: &HeightSettings, seed: u64) -> Vec<Tilt> {
    let random = |p: usize, which: i64| (crate::fate::planet_hash(seed, 0x7117, p as i64, which, 0) >> 40) as f32 / (1u64 << 24) as f32;
    plates
        .kept
        .iter()
        .enumerate()
        .map(|(p, plate)| {
            let angle = random(p, 0) * std::f32::consts::TAU;
            Tilt {
                toward: (angle.cos(), angle.sin()),
                rise: settings.tilt * (0.33 + 0.67 * random(p, 1)),
                radius: (plate.pixels as f32 / std::f32::consts::PI).sqrt().max(1.0),
            }
        })
        .collect()
}

/// For each pixel of a plate, how many pixels it is from the nearest pixel that isn't on the same
/// plate (its edge). Two sweeps, each passing distances on to neighbours (1 across, 1.41
/// diagonally).
fn edge_distance(plates: &Plates) -> Vec<f32> {
    let width = plates.width;
    let mut distance: Vec<f32> = plates.plate.iter().map(|p| if p.is_some() { f32::MAX } else { 0.0 }).collect();
    let same = |a: usize, b: usize| plates.plate[a] == plates.plate[b];
    const DIAGONAL: f32 = std::f32::consts::SQRT_2;
    // Pixels at the field's edge, or next to another plate or the ocean, are at distance 1.
    for row in 0..width {
        for col in 0..width {
            let i = row * width + col;
            if plates.plate[i].is_none() {
                continue;
            }
            let at_edge = col == 0
                || row == 0
                || col + 1 == width
                || row + 1 == width
                || !same(i, i - 1)
                || !same(i, i + 1)
                || !same(i, i - width)
                || !same(i, i + width);
            if at_edge {
                distance[i] = 1.0;
            }
        }
    }
    // Down and right, then up and left.
    for row in 0..width {
        for col in 0..width {
            let i = row * width + col;
            if plates.plate[i].is_none() {
                continue;
            }
            let mut best = distance[i];
            for (dc, dr, step) in [(-1i64, 0i64, 1.0), (0, -1, 1.0), (-1, -1, DIAGONAL), (1, -1, DIAGONAL)] {
                let (c, r) = (col as i64 + dc, row as i64 + dr);
                if c >= 0 && r >= 0 && (c as usize) < width {
                    let j = r as usize * width + c as usize;
                    if same(i, j) {
                        best = best.min(distance[j] + step);
                    }
                }
            }
            distance[i] = best;
        }
    }
    for row in (0..width).rev() {
        for col in (0..width).rev() {
            let i = row * width + col;
            if plates.plate[i].is_none() {
                continue;
            }
            let mut best = distance[i];
            for (dc, dr, step) in [(1i64, 0i64, 1.0), (0, 1, 1.0), (1, 1, DIAGONAL), (-1, 1, DIAGONAL)] {
                let (c, r) = (col as i64 + dc, row as i64 + dr);
                if c >= 0 && (c as usize) < width && (r as usize) < width {
                    let j = r as usize * width + c as usize;
                    if same(i, j) {
                        best = best.min(distance[j] + step);
                    }
                }
            }
            distance[i] = best;
        }
    }
    distance
}

/// Each pixel's tilt, as made: how far up (+) or down (−) its plate's slope it is, about −1.5 to
/// 1.5 (0 at the plate's middle, ±1 about a plate's radius out, times that plate's random share of
/// the most tilt). The same tilts as `terrain` uses (given the same seed), without the "most
/// tilt" or the blend, so the World Compiler can grade land with it. 0 off the plates.
pub fn tilt_field(plates: &Plates, settings: &HeightSettings, seed: u64) -> Vec<f32> {
    let unit = HeightSettings { tilt: 1.0, ..settings.clone() };
    let tilts = tilts(plates, &unit, crate::fate::derive_seed(seed, "tilt"));
    let width = plates.width;
    (0..width * width)
        .map(|i| {
            let Some(p) = plates.plate[i] else { return 0.0 };
            let (plate, tilt) = (&plates.kept[p as usize], &tilts[p as usize]);
            let (x, y) = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
            let across = ((x - plate.middle.0) * tilt.toward.0 + (y - plate.middle.1) * tilt.toward.1) / tilt.radius;
            tilt.rise * across.clamp(-1.5, 1.5)
        })
        .collect()
}

/// The terrain of every pixel of the plates as made: base height + noise − sunken edges. (Pixels
/// on no plate get the ocean plates' base height.) Uses all the computer's cores.
pub fn terrain(plates: &Plates, settings: &HeightSettings, seed: u64) -> Vec<f32> {
    let width = plates.width;
    let blocks = plates.blocks_per_pixel;
    let edges = edge_distance(plates);
    let noise = PerlinSettings { scale: settings.terrain_size, octaves: settings.octaves, persistence: settings.roughness, ..PerlinSettings::default() };
    let shelf = (settings.shelf_width / blocks).max(1e-3);
    let tilts = tilts(plates, settings, crate::fate::derive_seed(seed, "tilt"));
    let blend = settings.blend.clamp(0.0, 1.0);
    let mut heights = vec![settings.ocean; width * width];
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in heights.chunks_mut(rows_each * width).enumerate() {
            let (noise, edges, tilts) = (&noise, &edges, &tilts);
            scope.spawn(move || {
                for (k, height) in chunk.iter_mut().enumerate() {
                    let i = band * rows_each * width + k;
                    let Some(p) = plates.plate[i] else { continue };
                    let plate = &plates.kept[p as usize];
                    let base = if plate.land { settings.land } else { settings.ocean };
                    let (x, y) = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
                    let rolling = noise.raw(seed, y * blocks, x * blocks) * settings.terrain;
                    // Tipped: up on the side the tilt faces, down on the other, nothing at the
                    // middle.
                    let tilt = &tilts[p as usize];
                    let across = ((x - plate.middle.0) * tilt.toward.0 + (y - plate.middle.1) * tilt.toward.1) / tilt.radius;
                    let tipped = tilt.rise * across.clamp(-1.5, 1.5);
                    let shape = tipped * (1.0 - blend) + rolling * blend;
                    // Sinking toward the edge: smoothly from nothing at the shelf's width in, to
                    // the full depth at the edge.
                    let inward = (edges[i] / shelf).clamp(0.0, 1.0);
                    let sunk = settings.shelf_depth * (1.0 - inward * inward * (3.0 - 2.0 * inward));
                    *height = base + shape - sunk;
                }
            });
        }
    });
    heights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::plates::{self, PlateSettings};
    use std::sync::atomic::AtomicBool;

    fn made() -> Plates {
        plates::make(&PlateSettings::default(), 7, 72_000.0, 256.0, &AtomicBool::new(false)).unwrap()
    }

    #[test]
    fn edges_are_close_and_middles_are_far() {
        let made = made();
        let distance = edge_distance(&made);
        for plate in &made.kept {
            let (col, row) = (plate.middle.0 as usize, plate.middle.1 as usize);
            let i = row * made.width + col;
            if made.plate[i].is_some() {
                assert!(distance[i] > 5.0, "a plate's middle is only {} from its edge", distance[i]);
            }
        }
        // Some pixel on a plate is right at its edge.
        assert!(made.plate.iter().zip(&distance).any(|(p, &d)| p.is_some() && d == 1.0));
    }

    #[test]
    fn land_plates_are_partly_under_water_and_ocean_plates_mostly_are() {
        let made = made();
        let heights = terrain(&made, &HeightSettings::default(), 3);
        let share_above = |land: bool| {
            let mine: Vec<f32> = (0..heights.len()).filter(|&i| made.plate[i].is_some_and(|p| made.kept[p as usize].land == land)).map(|i| heights[i]).collect();
            mine.iter().filter(|&&h| h > 0.0).count() as f32 / mine.len().max(1) as f32
        };
        let (land, ocean) = (share_above(true), share_above(false));
        assert!((0.2..0.9).contains(&land), "land plates above water: {land}");
        assert!(ocean < 0.05, "ocean plates above water: {ocean}");
    }

    #[test]
    fn a_tilted_plate_rises_on_one_side_and_sinks_on_the_other() {
        let made = made();
        let settings = HeightSettings { blend: 0.0, shelf_depth: 0.0, ..HeightSettings::default() };
        let heights = terrain(&made, &settings, 3);
        let tilt = &tilts(&made, &settings, crate::fate::derive_seed(3, "tilt"))[0];
        let plate = &made.kept[0];
        // Step from the middle toward the rising side, and the same distance away from it.
        let at = |sign: f32| {
            let (x, y) = (plate.middle.0 + sign * tilt.toward.0 * tilt.radius * 0.6, plate.middle.1 + sign * tilt.toward.1 * tilt.radius * 0.6);
            heights[y as usize * made.width + x as usize]
        };
        let middle = heights[plate.middle.1 as usize * made.width + plate.middle.0 as usize];
        assert!(at(1.0) > middle && middle > at(-1.0), "uphill {}, middle {middle}, downhill {}", at(1.0), at(-1.0));
        // Only noise: no tilt at all.
        let flat = terrain(&made, &HeightSettings { blend: 1.0, terrain: 0.0, shelf_depth: 0.0, ..HeightSettings::default() }, 3);
        assert_eq!(flat[plate.middle.1 as usize * made.width + plate.middle.0 as usize], settings.land);
    }

    #[test]
    fn sunken_edges_are_lower_than_middles() {
        let made = made();
        let flat = HeightSettings { terrain: 0.0, shelf_depth: 0.3, ..HeightSettings::default() };
        let heights = terrain(&made, &flat, 3);
        let distance = edge_distance(&made);
        let land = |i: usize| made.plate[i].is_some_and(|p| made.kept[p as usize].land);
        let edge = (0..heights.len()).find(|&i| land(i) && distance[i] == 1.0).unwrap();
        let middle = (0..heights.len()).find(|&i| land(i) && distance[i] > 12.0).unwrap();
        assert!(heights[edge] < heights[middle] - 0.2);
    }
}

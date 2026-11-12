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
use super::plates::{self, Plates};
use serde::{Deserialize, Serialize};

/// One kind of plate's ground: land plates have one, ocean plates another [David, 2026-10-27].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ground {
    /// Where these plates sit (0 = the usual sea level).
    pub base: f32,
    /// How high and low the terrain noise goes either way on them.
    pub terrain: f32,
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

    /// Spread per plate [David]: each plate's own value scattered around the slider's. The base
    /// height moves up or down by up to this much; the others are multiplied (1 = between half
    /// and twice; see `plates::spread_by`). 0 = every plate the same.
    pub base_spread: f32,
    pub terrain_spread: f32,
    pub shelf_width_spread: f32,
    pub shelf_depth_spread: f32,
    pub squeeze_spread: f32,
}

impl Default for Ground {
    fn default() -> Self {
        Self {
            base: 0.05,
            terrain: 0.8,
            tilt: 0.8,
            blend: 0.7,
            shelf_width: 2_500.0,
            shelf_depth: 0.2,
            squeeze: 0.6,
            base_spread: 0.0,
            terrain_spread: 0.0,
            shelf_width_spread: 0.0,
            shelf_depth_spread: 0.0,
            squeeze_spread: 0.0,
        }
    }
}

impl Ground {
    /// The settings the terrain is worked out from (not the squeeze, applied as it's drawn).
    fn shaping(&self) -> [u32; 10] {
        [
            self.base.to_bits(),
            self.terrain.to_bits(),
            self.tilt.to_bits(),
            self.blend.to_bits(),
            self.shelf_width.to_bits(),
            self.shelf_depth.to_bits(),
            self.base_spread.to_bits(),
            self.terrain_spread.to_bits(),
            self.shelf_width_spread.to_bits(),
            self.shelf_depth_spread.to_bits(),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, from = "HeightSettingsFile")]
pub struct HeightSettings {
    /// Each kind of plate's ground.
    pub land: Ground,
    pub ocean: Ground,
    /// The terrain noise is one for the whole world: the size of its biggest features, in blocks;
    /// finer, fainter layers on top, and how strong each is compared with the one before.
    pub terrain_size: f32,
    pub octaves: u32,
    pub roughness: f32,
    /// The water level: below it is sea.
    pub water: f32,
}

impl Default for HeightSettings {
    fn default() -> Self {
        Self {
            land: Ground::default(),
            ocean: Ground { base: -0.6, ..Ground::default() },
            terrain_size: 15_000.0,
            octaves: 5,
            roughness: 0.5,
            water: 0.0,
        }
    }
}

impl HeightSettings {
    /// The settings the terrain is worked out from (not the squeeze or the water level, which
    /// are applied as it's drawn).
    pub fn shaping(&self) -> Vec<u32> {
        let mut all = vec![self.terrain_size.to_bits(), self.octaves, self.roughness.to_bits()];
        all.extend(self.land.shaping());
        all.extend(self.ocean.shaping());
        all
    }

    /// Land plates' ground, or ocean plates'.
    pub fn ground(&self, land: bool) -> &Ground {
        if land { &self.land } else { &self.ocean }
    }

    /// Plate `plate`'s own settings, with the spreads applied. `seed` is the terrain's seed.
    pub fn for_plate(&self, plate: usize, land: bool, seed: u64) -> PlateHeights {
        let random = |which: i64| plates::plate_random(crate::fate::derive_seed(seed, "spread"), plate, which);
        let ground = self.ground(land);
        PlateHeights {
            base: ground.base + ground.base_spread * random(if land { 0 } else { 1 }),
            terrain: plates::spread_by(ground.terrain, ground.terrain_spread, random(2)),
            blend: ground.blend.clamp(0.0, 1.0),
            shelf_width: plates::spread_by(ground.shelf_width, ground.shelf_width_spread, random(3)),
            shelf_depth: plates::spread_by(ground.shelf_depth, ground.shelf_depth_spread, random(4)),
        }
    }

    /// How much squeezing lifts rock on plate `plate` (its kind's squeeze, spread). `seed` is the
    /// terrain's seed. (Asked for every pixel as the map is drawn, so it works out only this.)
    pub fn squeeze_on(&self, plate: u8, land: bool, seed: u64) -> f32 {
        let ground = self.ground(land);
        if ground.squeeze_spread == 0.0 {
            return ground.squeeze;
        }
        plates::spread_by(ground.squeeze, ground.squeeze_spread, plates::plate_random(crate::fate::derive_seed(seed, "spread"), plate as usize, 5))
    }
}

/// One plate's own height settings, after the spreads. (Its squeeze: `squeeze_on`; its tilt:
/// `tilts`.)
pub struct PlateHeights {
    pub base: f32,
    pub terrain: f32,
    pub blend: f32,
    pub shelf_width: f32,
    pub shelf_depth: f32,
}

/// How height settings are read from a file: as they are now, or as they were before land and
/// ocean were separated (one set for both, and a base height each), when both kinds get the old
/// set.
#[derive(Deserialize, Default)]
#[serde(default)]
struct HeightSettingsFile {
    land: Option<OldOrNew>,
    ocean: Option<OldOrNew>,
    terrain_size: Option<f32>,
    octaves: Option<u32>,
    roughness: Option<f32>,
    water: Option<f32>,
    // Before the separation.
    terrain: Option<f32>,
    tilt: Option<f32>,
    blend: Option<f32>,
    shelf_width: Option<f32>,
    shelf_depth: Option<f32>,
    squeeze: Option<f32>,
    land_spread: Option<f32>,
    ocean_spread: Option<f32>,
    terrain_spread: Option<f32>,
    shelf_width_spread: Option<f32>,
    shelf_depth_spread: Option<f32>,
    squeeze_spread: Option<f32>,
}

/// `land` and `ocean` were base heights; now they're each a whole kind's ground.
#[derive(Deserialize)]
#[serde(untagged)]
enum OldOrNew {
    Old(f32),
    New(Ground),
}

impl From<HeightSettingsFile> for HeightSettings {
    fn from(file: HeightSettingsFile) -> Self {
        let now = HeightSettings::default();
        let kind = |read: Option<OldOrNew>, default: &Ground, base_spread: Option<f32>| match read {
            Some(OldOrNew::New(ground)) => ground,
            old => Ground {
                base: match old {
                    Some(OldOrNew::Old(base)) => base,
                    _ => default.base,
                },
                terrain: file.terrain.unwrap_or(default.terrain),
                tilt: file.tilt.unwrap_or(default.tilt),
                blend: file.blend.unwrap_or(default.blend),
                shelf_width: file.shelf_width.unwrap_or(default.shelf_width),
                shelf_depth: file.shelf_depth.unwrap_or(default.shelf_depth),
                squeeze: file.squeeze.unwrap_or(default.squeeze),
                base_spread: base_spread.unwrap_or(default.base_spread),
                terrain_spread: file.terrain_spread.unwrap_or(default.terrain_spread),
                shelf_width_spread: file.shelf_width_spread.unwrap_or(default.shelf_width_spread),
                shelf_depth_spread: file.shelf_depth_spread.unwrap_or(default.shelf_depth_spread),
                squeeze_spread: file.squeeze_spread.unwrap_or(default.squeeze_spread),
            },
        };
        Self {
            land: kind(file.land, &now.land, file.land_spread),
            ocean: kind(file.ocean, &now.ocean, file.ocean_spread),
            terrain_size: file.terrain_size.unwrap_or(now.terrain_size),
            octaves: file.octaves.unwrap_or(now.octaves),
            roughness: file.roughness.unwrap_or(now.roughness),
            water: file.water.unwrap_or(now.water),
        }
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
                rise: settings.ground(plate.land).tilt * (0.33 + 0.67 * random(p, 1)),
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
    let mut unit = settings.clone();
    (unit.land.tilt, unit.ocean.tilt) = (1.0, 1.0);
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
    let own: Vec<PlateHeights> = plates.kept.iter().enumerate().map(|(p, plate)| settings.for_plate(p, plate.land, seed)).collect();
    let tilts = tilts(plates, settings, crate::fate::derive_seed(seed, "tilt"));
    let mut heights = vec![settings.ocean.base; width * width];
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in heights.chunks_mut(rows_each * width).enumerate() {
            let (noise, edges, tilts, own) = (&noise, &edges, &tilts, &own);
            scope.spawn(move || {
                for (k, height) in chunk.iter_mut().enumerate() {
                    let i = band * rows_each * width + k;
                    let Some(p) = plates.plate[i] else { continue };
                    let plate = &plates.kept[p as usize];
                    let mine = &own[p as usize];
                    let base = mine.base;
                    let (x, y) = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
                    let rolling = noise.raw(seed, y * blocks, x * blocks) * mine.terrain;
                    // Tipped: up on the side the tilt faces, down on the other, nothing at the
                    // middle.
                    let tilt = &tilts[p as usize];
                    let across = ((x - plate.middle.0) * tilt.toward.0 + (y - plate.middle.1) * tilt.toward.1) / tilt.radius;
                    let tipped = tilt.rise * across.clamp(-1.5, 1.5);
                    let shape = tipped * (1.0 - mine.blend) + rolling * mine.blend;
                    // Sinking toward the edge: smoothly from nothing at the shelf's width in, to
                    // the full depth at the edge.
                    let shelf = (mine.shelf_width / blocks).max(1e-3);
                    let inward = (edges[i] / shelf).clamp(0.0, 1.0);
                    let sunk = mine.shelf_depth * (1.0 - inward * inward * (3.0 - 2.0 * inward));
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

    /// The defaults with the same change made to both kinds' ground.
    fn both(change: impl Fn(&mut Ground)) -> HeightSettings {
        let mut settings = HeightSettings::default();
        change(&mut settings.land);
        change(&mut settings.ocean);
        settings
    }

    #[test]
    fn land_settings_change_only_land_plates() {
        let made = made();
        let before = terrain(&made, &HeightSettings::default(), 3);
        let mut changed = HeightSettings::default();
        (changed.land.base, changed.land.terrain, changed.land.shelf_depth) = (0.5, 0.2, 0.6);
        let after = terrain(&made, &changed, 3);
        for i in 0..before.len() {
            match made.plate[i].map(|p| made.kept[p as usize].land) {
                Some(true) => {}
                _ => assert_eq!(before[i], after[i], "pixel {i}, not on a land plate, changed"),
            }
        }
        assert!(before.iter().zip(&after).any(|(a, b)| a != b));
    }

    #[test]
    fn settings_saved_before_the_split_read_as_both_kinds() {
        // As a preset saved them before land and ocean were separated.
        let old = "land = 0.1\nocean = -0.9\nterrain = 0.4\ntilt = 1.2\nsqueeze = 2.0\nland_spread = 0.3\nterrain_size = 9000.0\nwater = 0.05\n";
        let read: HeightSettings = toml::from_str(old).unwrap();
        assert_eq!((read.land.base, read.ocean.base, read.land.base_spread, read.ocean.base_spread), (0.1, -0.9, 0.3, 0.0));
        for ground in [&read.land, &read.ocean] {
            assert_eq!((ground.terrain, ground.tilt, ground.squeeze), (0.4, 1.2, 2.0));
            assert_eq!(ground.shelf_width, Ground::default().shelf_width);
        }
        assert_eq!((read.terrain_size, read.water), (9000.0, 0.05));
        // And as they're saved now, read back exactly.
        let now = toml::to_string(&read).unwrap();
        assert_eq!(toml::from_str::<HeightSettings>(&now).unwrap(), read);
        assert_eq!(toml::from_str::<HeightSettings>("").unwrap(), HeightSettings::default());
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
        let settings = both(|g| (g.blend, g.shelf_depth) = (0.0, 0.0));
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
        let flat = terrain(&made, &both(|g| (g.blend, g.terrain, g.shelf_depth) = (1.0, 0.0, 0.0)), 3);
        assert_eq!(flat[plate.middle.1 as usize * made.width + plate.middle.0 as usize], settings.land.base);
    }

    #[test]
    fn a_spread_makes_plates_differ_and_none_leaves_them_alike() {
        let made = made();
        // Flat plates: only the base height, so each plate's middle shows its own.
        let flat = both(|g| (g.terrain, g.tilt, g.shelf_depth) = (0.0, 0.0, 0.0));
        let middles = |settings: &HeightSettings| {
            let heights = terrain(&made, settings, 3);
            made.kept.iter().filter(|plate| plate.land).map(|plate| heights[plate.middle.1 as usize * made.width + plate.middle.0 as usize]).collect::<Vec<f32>>()
        };
        assert!(middles(&flat).iter().all(|&h| h == flat.land.base));
        let mut spread = flat.clone();
        spread.land.base_spread = 0.5;
        let spread = middles(&spread);
        let (low, high) = spread.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)));
        assert!(high - low > 0.2, "land plates from {low} to {high}");
        assert!(spread.iter().all(|&h| (h - flat.land.base).abs() <= 0.5));
        // The squeeze: the slider's own at no spread, each plate's own with one, within half to twice.
        assert_eq!(flat.squeeze_on(0, true, 3), flat.land.squeeze);
        let mut squeezed = flat.clone();
        squeezed.land.squeeze_spread = 1.0;
        let lifts: Vec<f32> = (0..made.kept.len() as u8).map(|p| squeezed.squeeze_on(p, true, 3)).collect();
        assert!(lifts.iter().all(|&s| (flat.land.squeeze * 0.5..=flat.land.squeeze * 2.0).contains(&s)));
        assert!(lifts.iter().any(|&s| s != lifts[0]));
    }

    #[test]
    fn sunken_edges_are_lower_than_middles() {
        let made = made();
        let flat = both(|g| (g.terrain, g.shelf_depth) = (0.0, 0.3));
        let heights = terrain(&made, &flat, 3);
        let distance = edge_distance(&made);
        let land = |i: usize| made.plate[i].is_some_and(|p| made.kept[p as usize].land);
        let edge = (0..heights.len()).find(|&i| land(i) && distance[i] == 1.0).unwrap();
        let middle = (0..heights.len()).find(|&i| land(i) && distance[i] > 12.0).unwrap();
        assert!(heights[edge] < heights[middle] - 0.2);
    }
}

//! Sample steps — TEMPORARY. Four simple steps so the suite has something to run, tune and look
//! at. They are not the world generator; the real steps get designed one by one, in the suite.
//!
//! 1. **Height**: layered noise, sinking toward the world's edge so the circle ends in ocean.
//! 2. **Moisture**: another layer of noise.
//! 3. **Land kinds**: ocean, plains, forest, mountains, from height and moisture.
//! 4. **Regions**: David's region idea, roughly. Every patch of one kind of land is split into
//!    regions of about the same size (Voronoi cells), and named from the patch: a forest named
//!    "Ravenwood" that splits in three gives "North Ravenwood", "South Ravenwood", "East Ravenwood".

use super::map::{LayerData, Map, Mark};
use super::noise::{Layered, layered};
use super::step::{Rules, Stopped, Work, slider};
use crate::fate::planet_hash;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

// ── 1. Height ──────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Height {
    pub scale: f32,
    pub octaves: u32,
    pub roughness: f32,
    /// How far out (0 = centre, 1 = edge) the land starts sinking.
    pub edge_start: f32,
    /// How much it has sunk by the edge.
    pub edge_drop: f32,
}

impl Default for Height {
    fn default() -> Self {
        Self { scale: 14_000.0, octaves: 8, roughness: 0.5, edge_start: 0.5, edge_drop: 0.8 }
    }
}

impl Rules for Height {
    const NAME: &'static str = "Height (sample)";
    const ABOUT: &'static str = "Layered noise that sinks toward the world's edge.";

    fn tune(&mut self, ui: &mut egui::Ui) {
        slider(ui, &mut self.scale, 1_000.0..=40_000.0, "size of the biggest hills (blocks)");
        slider(ui, &mut self.octaves, 1..=12, "layers of detail");
        slider(ui, &mut self.roughness, 0.1..=0.9, "roughness");
        slider(ui, &mut self.edge_start, 0.0..=1.0, "edge starts sinking at");
        slider(ui, &mut self.edge_drop, 0.0..=2.0, "edge sinks by");
    }

    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped> {
        let noise = Layered { scale: self.scale, octaves: self.octaves, roughness: self.roughness };
        let (seed, radius) = (work.seed, map.world_radius);
        let values = map.fill(|_, _, north, east| {
            let out = (north * north + east * east).sqrt() / radius;
            let t = ((out - self.edge_start) / (1.0 - self.edge_start).max(1e-3)).clamp(0.0, 1.0);
            let sink = t * t * (3.0 - 2.0 * t);
            layered(seed, 1, noise, north, east) * 0.5 + 0.5 - self.edge_drop * sink
        });
        work.check()?;
        let highest = (0..values.len()).max_by(|&a, &b| values[a].total_cmp(&values[b])).unwrap_or(0);
        let (col, row) = (highest % map.width, highest / map.width);
        work.log_at(col as f32, row as f32, format!("The highest point is {:.3}.", values[highest]));
        map.put("height", LayerData::Numbers(values));
        Ok(())
    }
}

// ── 2. Moisture ────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Moisture {
    pub scale: f32,
    pub octaves: u32,
    pub roughness: f32,
}

impl Default for Moisture {
    fn default() -> Self {
        Self { scale: 8_000.0, octaves: 6, roughness: 0.55 }
    }
}

impl Rules for Moisture {
    const NAME: &'static str = "Moisture (sample)";
    const ABOUT: &'static str = "Another layer of noise: wet and dry.";

    fn tune(&mut self, ui: &mut egui::Ui) {
        slider(ui, &mut self.scale, 500.0..=30_000.0, "size of wet and dry patches (blocks)");
        slider(ui, &mut self.octaves, 1..=12, "layers of detail");
        slider(ui, &mut self.roughness, 0.1..=0.9, "roughness");
    }

    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped> {
        let noise = Layered { scale: self.scale, octaves: self.octaves, roughness: self.roughness };
        let seed = work.seed;
        let values = map.fill(|_, _, north, east| layered(seed, 2, noise, north, east) * 0.5 + 0.5);
        work.check()?;
        map.put("moisture", LayerData::Numbers(values));
        Ok(())
    }
}

// ── 3. Land kinds ──────────────────────────────────────────────────────────

pub const OFF_WORLD: u8 = 0;
pub const OCEAN: u8 = 1;
const PLAINS: u8 = 2;
const FOREST: u8 = 3;
const MOUNTAINS: u8 = 4;
const KIND_NAMES: [&str; 5] = ["off the world", "ocean", "plains", "forest", "mountains"];

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LandKinds {
    /// How much of the world is water. The sea level is set to give exactly this.
    pub water_percent: f32,
    /// How much of the land is mountains (the highest of it).
    pub mountain_percent: f32,
    /// Land wetter than this is forest.
    pub forest_moisture: f32,
}

impl Default for LandKinds {
    fn default() -> Self {
        Self { water_percent: 70.0, mountain_percent: 8.0, forest_moisture: 0.55 }
    }
}

impl Rules for LandKinds {
    const NAME: &'static str = "Land kinds (sample)";
    const ABOUT: &'static str = "Ocean, plains, forest, mountains, from height and moisture.";

    fn tune(&mut self, ui: &mut egui::Ui) {
        slider(ui, &mut self.water_percent, 0.0..=100.0, "water (% of the world)");
        slider(ui, &mut self.mountain_percent, 0.0..=50.0, "mountains (% of the land)");
        slider(ui, &mut self.forest_moisture, 0.0..=1.0, "forest above this moisture");
    }

    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped> {
        let (Some(height), Some(moisture)) = (map.numbers("height"), map.numbers("moisture")) else {
            work.log("Needs the height and moisture layers first.");
            return Ok(());
        };
        // Sort the heights of the world's cells to find the levels that give the wanted amounts.
        let mut sorted: Vec<f32> = (0..map.cells())
            .filter(|&i| map.in_world(i % map.width, i / map.width))
            .map(|i| height[i])
            .collect();
        sorted.sort_by(f32::total_cmp);
        let level_at = |fraction: f32| sorted[((sorted.len() as f32 * fraction) as usize).min(sorted.len() - 1)];
        let water = (self.water_percent / 100.0).clamp(0.0, 1.0);
        let sea_level = level_at(water);
        let mountain_level = level_at(water + (1.0 - water) * (1.0 - self.mountain_percent / 100.0));
        work.check()?;

        let mut counts = [0usize; 5];
        let values: Vec<u8> = (0..map.cells())
            .map(|i| {
                let kind = if !map.in_world(i % map.width, i / map.width) {
                    OFF_WORLD
                } else if height[i] < sea_level {
                    OCEAN
                } else if height[i] >= mountain_level {
                    MOUNTAINS
                } else if moisture[i] > self.forest_moisture {
                    FOREST
                } else {
                    PLAINS
                };
                counts[kind as usize] += 1;
                kind
            })
            .collect();
        let world_cells = (sorted.len()).max(1) as f32;
        work.log(format!("Sea level {sea_level:.3}, mountains from {mountain_level:.3}."));
        for kind in [OCEAN, PLAINS, FOREST, MOUNTAINS] {
            let share = counts[kind as usize] as f32 / world_cells * 100.0;
            work.log(format!("{}: {share:.1}% of the world.", KIND_NAMES[kind as usize]));
        }
        let names = KIND_NAMES.iter().map(|name| name.to_string()).collect();
        map.put("land kinds", LayerData::Kinds { values, names });
        Ok(())
    }
}

// ── 4. Regions ─────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Regions {
    /// About how wide a region is, in blocks.
    pub region_size: f32,
    /// Patches smaller than this share of a region join the neighbour they touch most.
    pub smallest_patch_percent: f32,
    /// Split the oceans into regions too.
    pub include_water: bool,
    /// Rounds of evening out the regions' shapes (Lloyd relaxation).
    pub even_out: u32,
}

impl Default for Regions {
    fn default() -> Self {
        Self { region_size: 3_000.0, smallest_patch_percent: 30.0, include_water: false, even_out: 2 }
    }
}

/// TEMPORARY name parts, until David's naming conventions.
const NAME_STARTS: [&str; 16] = [
    "Dark", "Ash", "Elder", "Grey", "Raven", "Thorn", "Mist", "Stone", "Gold", "Wolf", "Iron", "Bright", "Hollow",
    "Frost", "Amber", "Willow",
];

fn name_ending(kind: &str) -> &'static str {
    match kind {
        "forest" => "wood",
        "plains" => "fields",
        "mountains" => "peaks",
        "ocean" => " Sea",
        _ => "land",
    }
}

const DIRECTIONS: [&str; 8] = ["East", "North-East", "North", "North-West", "West", "South-West", "South", "South-East"];

struct Patch {
    kind: u8,
    cells: Vec<usize>,
}

impl Rules for Regions {
    const NAME: &'static str = "Regions (sample)";
    const ABOUT: &'static str = "Splits each patch of land into named regions of about the same size.";

    fn tune(&mut self, ui: &mut egui::Ui) {
        slider(ui, &mut self.region_size, 250.0..=20_000.0, "region size (blocks across)");
        slider(ui, &mut self.smallest_patch_percent, 0.0..=100.0, "smallest patch (% of a region)");
        slider(ui, &mut self.even_out, 0..=8, "rounds of evening out");
        ui.checkbox(&mut self.include_water, "split the oceans too");
    }

    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped> {
        let Some((kinds, kind_names)) = map.kinds("land kinds") else {
            work.log("Needs the land kinds layer first.");
            return Ok(());
        };
        let (kinds, kind_names) = (kinds.to_vec(), kind_names.to_vec());
        let counts_as_land = |kind: u8| kind != OFF_WORLD && (kind != OCEAN || self.include_water);

        // 1. Patches: connected cells of the same kind.
        let mut patch_of = vec![usize::MAX; map.cells()];
        let mut patches: Vec<Patch> = Vec::new();
        for start in 0..map.cells() {
            if patch_of[start] != usize::MAX || !counts_as_land(kinds[start]) {
                continue;
            }
            let (id, kind) = (patches.len(), kinds[start]);
            let mut cells = vec![start];
            patch_of[start] = id;
            let mut next = 0;
            while next < cells.len() {
                let i = cells[next];
                next += 1;
                for (c, r) in map.neighbours(i % map.width, i / map.width) {
                    let j = map.index(c, r);
                    if patch_of[j] == usize::MAX && kinds[j] == kind {
                        patch_of[j] = id;
                        cells.push(j);
                    }
                }
            }
            patches.push(Patch { kind, cells });
        }
        work.check()?;

        // 2. Small patches join the neighbouring patch they share the most edge with.
        let region_cells = (self.region_size / map.blocks_per_cell).powi(2).max(1.0);
        let smallest = (region_cells * self.smallest_patch_percent / 100.0) as usize;
        let mut order: Vec<usize> = (0..patches.len()).collect();
        order.sort_by_key(|&p| patches[p].cells.len());
        let mut folded = 0;
        for p in order {
            if patches[p].cells.is_empty() || patches[p].cells.len() >= smallest {
                continue;
            }
            let mut touching: Vec<(usize, usize)> = Vec::new();
            for &i in &patches[p].cells {
                for (c, r) in map.neighbours(i % map.width, i / map.width) {
                    let other = patch_of[map.index(c, r)];
                    if other != usize::MAX && other != p {
                        match touching.iter_mut().find(|(q, _)| *q == other) {
                            Some((_, n)) => *n += 1,
                            None => touching.push((other, 1)),
                        }
                    }
                }
            }
            // An island on its own stays as it is.
            let Some(&(into, _)) = touching.iter().max_by_key(|(_, n)| *n) else { continue };
            let cells = std::mem::take(&mut patches[p].cells);
            for &i in &cells {
                patch_of[i] = into;
            }
            patches[into].cells.extend(cells);
            folded += 1;
        }
        work.check()?;

        // 3. Split each patch into regions, and name them.
        let mut ids = vec![0u32; map.cells()];
        let mut names: Vec<String> = Vec::new();
        let mut marks: Vec<Mark> = Vec::new();
        let mut used_names: Vec<String> = Vec::new();
        let mut patch_count = 0;
        for (p, patch) in patches.iter().enumerate() {
            if patch.cells.is_empty() {
                continue;
            }
            work.check()?;
            patch_count += 1;
            let kind_name = kind_names.get(patch.kind as usize).map_or("land", |n| n.as_str());
            let base = {
                // Try every start (beginning at a random one) before falling back to a number.
                let first = planet_hash(work.seed, 7, p as i64, 0, 0) as usize;
                let ending = name_ending(kind_name);
                let mut name = (0..NAME_STARTS.len())
                    .map(|k| format!("{}{ending}", NAME_STARTS[(first + k) % NAME_STARTS.len()]))
                    .find(|name| !used_names.contains(name))
                    .unwrap_or_default();
                let mut number = 2;
                while name.is_empty() || used_names.contains(&name) {
                    name = format!("{}{ending} {number}", NAME_STARTS[first % NAME_STARTS.len()]);
                    number += 1;
                }
                used_names.push(name.clone());
                name
            };

            let count = ((patch.cells.len() as f32 / region_cells).round() as usize).max(1);
            let owner = self.split(map, &patch.cells, count, work.seed ^ p as u64);
            let centre = centroid(map, &patch.cells);
            let spread = (patch.cells.len() as f32 / std::f32::consts::PI).sqrt();

            let mut region_names = Vec::new();
            for part in 0..count {
                let cells: Vec<usize> = patch.cells.iter().zip(&owner).filter(|(_, o)| **o == part).map(|(i, _)| *i).collect();
                if cells.is_empty() {
                    continue;
                }
                let at = centroid(map, &cells);
                let name = if count == 1 {
                    base.clone()
                } else {
                    let (east, north) = (at.0 - centre.0, centre.1 - at.1);
                    let direction = if (east * east + north * north).sqrt() < spread * 0.25 {
                        "Central"
                    } else {
                        let turn = (north.atan2(east) / std::f32::consts::TAU * 8.0).round().rem_euclid(8.0);
                        DIRECTIONS[turn as usize]
                    };
                    let mut name = format!("{direction} {base}");
                    let mut number = 2;
                    while region_names.contains(&name) {
                        name = format!("{direction} {base} {number}");
                        number += 1;
                    }
                    name
                };
                names.push(name.clone());
                let id = names.len() as u32;
                for &i in &cells {
                    ids[i] = id;
                }
                marks.push(Mark { col: at.0, row: at.1, label: name.clone(), group: "region names".into() });
                region_names.push(name);
            }
            if count > 1 {
                let square_blocks = patch.cells.len() as f32 * map.blocks_per_cell.powi(2) / 1_000_000.0;
                work.log_at(
                    centre.0,
                    centre.1,
                    format!(
                        "{base} ({kind_name}, {:.0} million square blocks) splits into {count}: {}.",
                        square_blocks,
                        region_names.join(", ")
                    ),
                );
            }
        }
        work.log(format!(
            "{} regions from {patch_count} patches ({folded} small patches joined their neighbours).",
            names.len()
        ));
        map.marks.retain(|mark| mark.group != "region names");
        map.marks.extend(marks);
        map.put("regions", LayerData::Regions { ids, names });
        Ok(())
    }
}

impl Regions {
    /// Splits `cells` into `count` parts around scattered centres, evened out by moving each centre
    /// to the middle of its part a few times. Returns which part each cell went to.
    fn split(&self, map: &Map, cells: &[usize], count: usize, seed: u64) -> Vec<usize> {
        let place = |i: usize| ((i % map.width) as f32, (i / map.width) as f32);
        let mut centres: Vec<(f32, f32)> = (0..count)
            .map(|k| place(cells[(planet_hash(seed, 8, k as i64, 0, 0) % cells.len() as u64) as usize]))
            .collect();
        let mut owner = vec![0usize; cells.len()];
        for round in 0..=self.even_out {
            for (o, &i) in owner.iter_mut().zip(cells) {
                let (x, y) = place(i);
                *o = (0..count)
                    .min_by(|&a, &b| {
                        let da = (centres[a].0 - x).powi(2) + (centres[a].1 - y).powi(2);
                        let db = (centres[b].0 - x).powi(2) + (centres[b].1 - y).powi(2);
                        da.total_cmp(&db)
                    })
                    .unwrap_or(0);
            }
            if round == self.even_out {
                break;
            }
            let mut sums = vec![(0.0f32, 0.0f32, 0usize); count];
            for (&o, &i) in owner.iter().zip(cells) {
                let (x, y) = place(i);
                sums[o].0 += x;
                sums[o].1 += y;
                sums[o].2 += 1;
            }
            for (centre, (x, y, n)) in centres.iter_mut().zip(sums) {
                if n > 0 {
                    *centre = (x / n as f32, y / n as f32);
                }
            }
        }
        owner
    }
}

/// The middle of some cells, as (col, row).
fn centroid(map: &Map, cells: &[usize]) -> (f32, f32) {
    let (mut x, mut y) = (0.0, 0.0);
    for &i in cells {
        x += (i % map.width) as f32 + 0.5;
        y += (i / map.width) as f32 + 0.5;
    }
    let n = cells.len().max(1) as f32;
    (x / n, y / n)
}

#[cfg(test)]
mod tests {
    use super::super::step::{Stage, Step};
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    fn run_all(seed: u64) -> Map {
        let steps: Vec<Box<dyn Step>> = super::super::sample_steps();
        let mut map = Map::new(20_000.0, 100.0);
        let stop = Arc::new(AtomicBool::new(false));
        for step in &steps {
            let mut work = Work::new(seed, step.name(), stop.clone());
            step.run(&mut map, &mut work).unwrap();
        }
        map
    }

    #[test]
    fn the_same_seed_makes_the_same_world() {
        let (a, b) = (run_all(11), run_all(11));
        assert_eq!(a.numbers("height"), b.numbers("height"));
        assert_eq!(a.kinds("land kinds").map(|k| k.0.to_vec()), b.kinds("land kinds").map(|k| k.0.to_vec()));
        let c = run_all(12);
        assert_ne!(a.numbers("height"), c.numbers("height"));
    }

    #[test]
    fn the_water_comes_out_at_the_asked_for_amount() {
        let map = run_all(3);
        let (kinds, _) = map.kinds("land kinds").unwrap();
        let world = kinds.iter().filter(|&&k| k != OFF_WORLD).count() as f32;
        let water = kinds.iter().filter(|&&k| k == OCEAN).count() as f32;
        assert!((water / world - 0.7).abs() < 0.01, "{}", water / world);
    }

    #[test]
    fn every_bit_of_land_is_in_a_named_region() {
        let map = run_all(5);
        let (kinds, _) = map.kinds("land kinds").unwrap();
        let Some(LayerData::Regions { ids, names }) = map.layer("regions").map(|l| &l.data) else { panic!() };
        for (kind, id) in kinds.iter().zip(ids) {
            assert_eq!(*kind != OFF_WORLD && *kind != OCEAN, *id != 0);
            assert!(*id as usize <= names.len());
        }
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "region names should all be different");
    }

    #[test]
    fn settings_go_into_a_preset_and_back() {
        let mut step = Stage(Height { octaves: 3, ..Height::default() });
        let saved = step.save();
        step.reset();
        assert_eq!(step.0.octaves, Height::default().octaves);
        step.load(&saved).unwrap();
        assert_eq!(step.0.octaves, 3);
    }
}

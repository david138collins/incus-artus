//! The placer's fields (`docs/placer.md` section 2): every number a world has at a spot, in one table
//! that describes itself (its name, the stage that makes it, its unit and its range), read by number.
//!
//! - **A field is named in data by its key** ("climate/rain"), so a made field or a distributor is
//!   data, not code. A key the table doesn't have is reported, never crashed on.
//! - **Adding a field** is a row in `FIELDS` and a line in `Reading::column_value`.
//! - **Most fields are the same all the way down a column** (worked out once per column and kept);
//!   a few change with the block's Z (`per_block`): its Z, its depth below the surface, and that depth
//!   as a share of the crust.
//! - **What the caller knows already** (the ground's height, its slope, the climate and the water
//!   over it) comes in with the column: the chunk maker, the far view and the World Compiler's
//!   close-up each have it to hand, and the close-up's climate is the Climate tab's live one.

use super::super::block::BlockId;
use super::super::interpreter::{CompiledMap, Here, Parts};
use super::super::mineralogy::NO_CLIMATE;
use super::super::position::{WORLD_BOTTOM, WORLD_HEIGHT};
use super::super::terrain::SEA_LEVEL;
use crate::worldgen::noise::{Fade, perlin};

/// The stages that make fields, in the order things happen. A field may be read only by a stage
/// after its own: the linear rule. (The ground at each block is last before Mineralogy: it's the
/// whole recipe, carved by the rivers.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Tectonics,
    Landmass,
    Topography,
    Rivers,
    Climate,
    Ground,
    Mineralogy,
}

impl Stage {
    pub const ALL: [Stage; 7] = [Stage::Tectonics, Stage::Landmass, Stage::Topography, Stage::Rivers, Stage::Climate, Stage::Ground, Stage::Mineralogy];

    pub fn name(self) -> &'static str {
        match self {
            Stage::Tectonics => "Tectonics (the bake)",
            Stage::Landmass => "Landmass",
            Stage::Topography => "Topography",
            Stage::Rivers => "Climate: rivers",
            Stage::Climate => "Climate: weather",
            Stage::Ground => "The ground (at each block)",
            Stage::Mineralogy => "Mineralogy",
        }
    }
}

/// Every field in the table, in the table's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    BakeHeight,
    Tilt,
    Squeeze,
    Push,
    PushWay,
    CoastDistance,
    MountainDistance,
    Mountains,
    Rise,
    EdgeRise,
    Ridgelines,
    Peaks,
    RiverDistance,
    WaterAcross,
    WaterDepth,
    UnderWater,
    Temperature,
    Rain,
    Height,
    Slope,
    Z,
    Depth,
    CrustShare,
}

/// A row of the table.
pub struct Info {
    pub field: Field,
    /// Its name in data (stage/what), and the name David sees.
    pub key: &'static str,
    pub name: &'static str,
    pub stage: Stage,
    pub unit: &'static str,
    /// Where its curves' ends are, left to right (its values may go past them).
    pub range: (f32, f32),
    /// Whether it changes down a column, block by block.
    pub per_block: bool,
    pub about: &'static str,
}

/// How far "distance to a river" looks (blocks): further than this is this far.
pub const RIVER_REACH: f32 = 2_000.0;

/// The highest Z, the end of heights' ranges.
const TOP: f32 = (WORLD_HEIGHT - 1) as f32;

const fn row(field: Field, key: &'static str, name: &'static str, stage: Stage, unit: &'static str, range: (f32, f32), about: &'static str) -> Info {
    Info { field, key, name, stage, unit, range, per_block: false, about }
}

const fn per_block(info: Info) -> Info {
    Info { per_block: true, ..info }
}

pub const FIELDS: [Info; 23] = [
    row(Field::BakeHeight, "tectonics/height", "the bake's height", Stage::Tectonics, "", (-1.5, 5.0), "The Tectonic Sim's height map, before the World Compiler's curves: below 0 is sea (about), the higher the more the plates piled up."),
    row(Field::Tilt, "tectonics/tilt", "tilt", Stage::Tectonics, "", (-1.5, 1.5), "How far up (+) or down (−) its plate's tilt each spot is."),
    row(Field::Squeeze, "tectonics/squeeze", "squeeze", Stage::Tectonics, "", (-1.0, 1.0), "How crushed (+, up to the map's most crushed at 1) or stretched (−, its most stretched at −1) the rock is."),
    row(Field::Push, "tectonics/push", "push strength", Stage::Tectonics, "", (0.0, 1.0), "How hard the rock was pushed (squeezed)."),
    row(Field::PushWay, "tectonics/push_way", "push way", Stage::Tectonics, "°", (0.0, 180.0), "Which way the rock was pushed: 0 east-west, 90 north-south (as the bake has it). Ridges run across it."),
    row(Field::CoastDistance, "landmass/coast_distance", "distance to the coast", Stage::Landmass, "blocks", (-2_000.0, 6_000.0), "Blocks to the coastline: + inland, − out to sea."),
    row(Field::MountainDistance, "topography/mountain_distance", "distance to the mountains' points", Stage::Topography, "blocks", (0.0, 10_000.0), "Blocks to the nearest of the mountains' points (where the rise is measured from, and the peaks stand)."),
    row(Field::Mountains, "topography/mountains", "mountains", Stage::Topography, "share", (0.0, 1.0), "How much of the room above the ground the mountains take here (the mountain curve read at the squeeze): 0 none, 1 all."),
    row(Field::Rise, "topography/rise", "the rise", Stage::Topography, "blocks", (0.0, 200.0), "How many blocks the rise lifts the land here (Topography, Mountains)."),
    row(Field::EdgeRise, "topography/edge_rise", "the edge rise", Stage::Topography, "blocks", (0.0, 100.0), "How many blocks the edge rise lifts the land here (on a plate's high side)."),
    row(Field::Ridgelines, "topography/ridgelines", "ridgelines", Stage::Topography, "blocks", (0.0, 200.0), "How many blocks the ridgelines stand here."),
    row(Field::Peaks, "topography/peaks", "peaks", Stage::Topography, "blocks", (0.0, 200.0), "How many blocks the peaks stand here, crags and all."),
    row(Field::RiverDistance, "rivers/distance", "distance to a river", Stage::Rivers, "blocks", (0.0, RIVER_REACH), "Blocks to the nearest river's water (0 in it), up to 2,000. Lakes aren't counted."),
    row(Field::WaterAcross, "rivers/water_across", "a river's water across", Stage::Rivers, "blocks", (0.0, 256.0), "About how many blocks of water the nearest river has across (its width times its depth), within its banks' reach; 0 further away."),
    row(Field::WaterDepth, "rivers/water_depth", "water over the ground", Stage::Rivers, "blocks", (0.0, 40.0), "How many blocks of water stand over the ground here: the sea's, a river's or a lake's (0 on dry land)."),
    row(Field::UnderWater, "rivers/under_water", "under a river's or lake's water", Stage::Rivers, "", (0.0, 1.0), "1 where a river's or a lake's water stands over the ground (its bed), 0 elsewhere (the sea's floor is 0)."),
    row(Field::Temperature, "climate/temperature", "temperature", Stage::Climate, "", (-1.0, 1.0), "−1 frozen to +1 the hottest, colder the higher the ground stands above the sea (the Climate tab's live one here)."),
    row(Field::Rain, "climate/rain", "rain", Stage::Climate, "", (0.0, 1.0), "0 dry to 1 the wettest."),
    row(Field::Height, "ground/height", "the ground's height", Stage::Ground, "Z", (0.0, TOP), "The ground's Z, as the world has it (the sea's surface is at 128, the peaks reach about 900)."),
    row(Field::Slope, "ground/slope", "slope", Stage::Ground, "blocks up per block", (0.0, 3.0), "How steep the ground is: blocks up per block across (1 is 45°), measured across 4 blocks of the ground (2 either side), the same near, far and in the World Compiler."),
    per_block(row(Field::Z, "ground/z", "the block's Z", Stage::Ground, "Z", (WORLD_BOTTOM as f32, TOP), "The block's own height, at each block down the column.")),
    per_block(row(Field::Depth, "ground/depth", "depth below the surface", Stage::Ground, "blocks", (0.0, 130.0), "How far a block is below the ground's surface (0 is the surface block).")),
    per_block(row(Field::CrustShare, "ground/crust_share", "depth, as a share of the crust", Stage::Ground, "share", (0.0, 1.0), "Depth below the surface over the crust's depth (Mineralogy's setting): 0 at the surface, 1 at the crust's bottom. Things placed by it keep their places whatever depth the world ends up with.")),
];

impl Field {
    pub fn info(self) -> &'static Info {
        &FIELDS[self as usize]
    }

    /// The field with this key, if the table has it.
    pub fn by_key(key: &str) -> Option<Field> {
        FIELDS.iter().find(|info| info.key == key).map(|info| info.field)
    }
}

/// What the caller knows about a column before any field is read.
#[derive(Clone, Copy, Debug)]
pub struct Column {
    /// Blocks from the world's middle.
    pub north: f32,
    pub east: f32,
    /// The ground's Z (whole blocks, as the world has it).
    pub ground: f32,
    /// Blocks up per block across.
    pub slope: f32,
    /// The temperature at the ground and the rain, or none (a world from before climate).
    pub climate: Option<(f32, f32)>,
    /// The top of a river's or lake's water over the ground (the blocks below it are water), if any.
    pub water: Option<i32>,
}

/// The fields at one column, each read once and kept, however many layers read it.
pub struct Reading<'a> {
    pub map: &'a CompiledMap,
    pub column: Column,
    /// The crust's depth (blocks), for depth as a share of it.
    crust: f32,
    maps: Option<Option<(Here, f32, f32)>>,
    parts: Option<Parts>,
    values: [f32; FIELDS.len()],
    /// The noise read here so far (its salt, its patches' size, its value): a few kept, as the same
    /// wander is often read by several candidates.
    noises: [(u64, f32, f32); 4],
    noises_kept: usize,
    /// What each distributor so far chose here (`AIR`: nothing), for the candidates after them.
    pub chosen: Vec<BlockId>,
}

impl<'a> Reading<'a> {
    pub fn new(map: &'a CompiledMap, column: Column, crust: f32) -> Self {
        Self { map, column, crust: crust.max(1.0), maps: None, parts: None, values: [f32::NAN; FIELDS.len()], noises: [(0, 0.0, 0.0); 4], noises_kept: 0, chosen: Vec::new() }
    }

    /// Seeded noise here, −1 to 1, in patches about `size` blocks across, its pattern chosen by `salt`.
    pub fn noise(&mut self, salt: u64, size: f32) -> f32 {
        if let Some(&(_, _, value)) = self.noises[..self.noises_kept].iter().find(|(s, z, _)| *s == salt && *z == size) {
            return value;
        }
        let value = perlin(self.map.seed, salt, Fade::Smooth, self.column.north / size, self.column.east / size);
        if self.noises_kept < self.noises.len() {
            self.noises[self.noises_kept] = (salt, size, value);
            self.noises_kept += 1;
        }
        value
    }

    /// A field's value at the block `z` high in this column.
    pub fn read(&mut self, field: Field, z: f32) -> f32 {
        let depth = self.column.ground - z;
        match field {
            Field::Z => z,
            Field::Depth => depth,
            Field::CrustShare => depth / self.crust,
            _ => {
                let kept = self.values[field as usize];
                if !kept.is_nan() {
                    return kept;
                }
                let value = self.column_value(field);
                self.values[field as usize] = value;
                value
            }
        }
    }

    /// The maps blended to this column (and where it is from the map's north-west corner).
    fn maps(&mut self) -> Option<(Here, f32, f32)> {
        let Column { north, east, .. } = self.column;
        *self.maps.get_or_insert_with(|| self.map.maps_at(north, east))
    }

    fn column_value(&mut self, field: Field) -> f32 {
        let Column { north, east, ground, slope, climate, water } = self.column;
        let (temperature, rain) = climate.unwrap_or(NO_CLIMATE);
        let map = self.map;
        let from_maps = |reading: &mut Self, value: &dyn Fn(&Here) -> f32| reading.maps().map_or(0.0, |(here, _, _)| value(&here));
        match field {
            Field::BakeHeight => from_maps(self, &|here| here.height),
            Field::Tilt => from_maps(self, &|here| here.tilt),
            Field::Squeeze => from_maps(self, &|here| {
                let (crushed, stretched) = (map.maps.most_crushed.max(1e-6), (-map.maps.most_stretched).max(1e-6));
                if here.squeeze >= 0.0 { here.squeeze / crushed } else { here.squeeze / stretched }
            }),
            Field::Push => from_maps(self, &|here| here.push_strength),
            Field::PushWay => from_maps(self, &|here| (here.push_y.atan2(here.push_x) / 2.0).to_degrees().rem_euclid(180.0)),
            Field::CoastDistance => from_maps(self, &|here| here.coast_distance),
            Field::MountainDistance => from_maps(self, &|here| here.lift_distance),
            Field::Mountains => from_maps(self, &|here| map.mountain_share(here)),
            Field::Rise => from_maps(self, &|here| map.lift(here)),
            Field::EdgeRise => from_maps(self, &|here| map.raised_coast(here)),
            Field::Ridgelines | Field::Peaks => {
                let Some((here, x, y)) = self.maps() else { return 0.0 };
                let parts = *self.parts.get_or_insert_with(|| map.parts(&here, x, y));
                if field == Field::Peaks { parts.peaks } else { parts.mountains }
            }
            Field::RiverDistance => map.rivers.distance_to_river(north, east, RIVER_REACH),
            Field::WaterAcross => map.rivers.water_near(north, east).map_or(0.0, |(_, _, across)| across),
            Field::WaterDepth => {
                let sea = if ground < SEA_LEVEL as f32 { SEA_LEVEL as f32 - 1.0 - ground } else { 0.0 };
                let river = water.map_or(0.0, |top| top as f32 - 1.0 - ground);
                sea.max(river).max(0.0)
            }
            Field::UnderWater => water.is_some() as u8 as f32,
            Field::Temperature => temperature,
            Field::Rain => rain,
            Field::Height => ground,
            Field::Slope => slope,
            Field::Z | Field::Depth | Field::CrustShare => unreachable!("worked out at each block"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_in_order_with_one_key_each() {
        for (k, info) in FIELDS.iter().enumerate() {
            assert_eq!(info.field as usize, k, "{} is out of place", info.key);
            assert_eq!(Field::by_key(info.key), Some(info.field));
            assert!(info.range.0 < info.range.1, "{}", info.key);
        }
        assert_eq!(Field::by_key("climate/snowfall"), None);
    }

    #[test]
    fn fields_read_what_the_world_has_and_depth_goes_down_the_column() {
        // Land rising from the sea in the west to Z 250 in the east (as terrain.rs's tests).
        let width = 8;
        let z: Vec<f32> = (0..width * width).map(|i| 100.0 + (i % width) as f32 * 22.0).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let ground = map.height(-16.0, 80.0).unwrap().round();
        let column = Column { north: -16.0, east: 80.0, ground, slope: 0.5, climate: Some((0.1, 0.7)), water: None };
        let mut reading = Reading::new(&map, column, 100.0);
        assert_eq!(reading.read(Field::Height, ground), ground);
        assert_eq!(reading.read(Field::Slope, ground), 0.5);
        assert_eq!((reading.read(Field::Temperature, ground), reading.read(Field::Rain, ground)), (0.1, 0.7));
        assert_eq!(reading.read(Field::Depth, ground - 30.0), 30.0);
        assert_eq!(reading.read(Field::CrustShare, ground - 30.0), 0.3);
        assert_eq!(reading.read(Field::Z, 77.0), 77.0);
        assert!(reading.read(Field::CoastDistance, ground) > 0.0, "inland");
        assert_eq!(reading.read(Field::WaterDepth, ground), 0.0);
        // No rivers: as far as it looks.
        assert_eq!(reading.read(Field::RiverDistance, ground), RIVER_REACH);
        // Out at sea: the sea's water over the floor, and the coast behind.
        let floor = map.height(-16.0, -112.0).unwrap().round();
        let mut sea = Reading::new(&map, Column { east: -112.0, ground: floor, ..column }, 100.0);
        assert_eq!(sea.read(Field::WaterDepth, floor), 127.0 - floor);
        assert!(sea.read(Field::CoastDistance, floor) < 0.0);
        // A river's water over the ground counts too.
        let mut river = Reading::new(&map, Column { water: Some(ground as i32 + 4), ..column }, 100.0);
        assert_eq!(river.read(Field::WaterDepth, ground), 3.0);
    }
}

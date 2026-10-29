//! Mineralogy — which blocks the ground is made of: the last section of the World Compiler's Climate
//! tab (after Rivers), read by the Interpreter at every column [Counsel and David, 2026-10-29].
//!
//! A rough first version, refined later: it reads only the climate (temperature and rain, handed to
//! the world at "Render World and Spawn") and the height. When rivers exist it gains its river parts
//! (gravel in fast water, sand where it slows, mud where it pools, shores); being after Rivers in the
//! pipeline, those are additions, not a redo.
//!
//! The surface, first rule that applies:
//! 1. **Snowy ground** where it's cold enough (a block's temperature falls with its height), peaks
//!    included.
//! 2. **Gabbro** on the peaks and **stone** on the mountainsides, by height (`gabbro_from`,
//!    `stone_from`).
//! 3. **Sand** on beaches just above the sea, and in hot, dry deserts.
//! 4. **Dry grass** where it's dry.
//! 5. **Grass** everywhere else.
//!
//! Under grass, dry grass and snowy ground: a few layers of **dirt** (so a step's riser shows earth,
//! not grey stone), under sand more sand, then stone. The sea floor: sand in the shallows, gravel
//! deeper. A little noise makes the lines between zones wander, so they don't follow the climate's
//! 128-block pixels.
//!
//! **A rule whose block isn't registered yet is skipped** (no sand block: no beaches), so each new
//! block David paints lands in the world as soon as it's registered, and until then the ground is as
//! it was.

use super::block::BlockId;
use super::interpreter::CompiledMap;
use super::terrain::{SEA_LEVEL, TerrainBlocks};
use crate::worldgen::noise::{Fade, perlin};
use serde::{Deserialize, Serialize};

/// Mineralogy's settings (in the Interpreter's recipe, so every world keeps its own; set in the
/// Climate tab's Mineralogy section).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MineralogySettings {
    /// Snowy ground where the temperature is below this (−1 frozen to +1 hottest).
    pub snow_below: f32,
    /// Dry grass where the rain is below this (0 dry to 1 wettest).
    pub dry_below: f32,
    /// Desert sand where the rain is below this and the temperature above `desert_above`.
    pub desert_below: f32,
    pub desert_above: f32,
    /// Beaches: sand up to this many blocks above the sea.
    pub beach: i32,
    /// How many blocks of dirt under grass, dry grass and snowy ground, and of sand under sand.
    pub dirt_depth: i32,
    pub sand_depth: i32,
    /// The sea floor: sand where the water is no deeper than this, gravel deeper.
    pub shallows: i32,
    /// How far the lines between zones wander (in temperature and rain), and how big the wandering
    /// is (blocks).
    pub wander: f32,
    pub wander_size: f32,
}

impl Default for MineralogySettings {
    fn default() -> Self {
        Self {
            snow_below: -0.45,
            dry_below: 0.3,
            desert_below: 0.12,
            desert_above: 0.25,
            beach: 2,
            dirt_depth: 3,
            sand_depth: 3,
            shallows: 8,
            wander: 0.06,
            wander_size: 48.0,
        }
    }
}

/// The climate where there is none (a world from before climate, the test world): mild and
/// middling wet, which is grass, as the ground always was.
pub const NO_CLIMATE: (f32, f32) = (0.2, 0.6);

/// What a column is made of at its top: the surface block, the block under it and how many blocks
/// of it, then stone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layers {
    pub top: BlockId,
    pub under: BlockId,
    pub depth: i32,
}

impl CompiledMap {
    /// The layers of the column at (`x` north, `y` east) whose ground is at `z`.
    pub fn layers(&self, x: i32, y: i32, z: i32, blocks: &TerrainBlocks) -> Layers {
        let s = &self.settings;
        let m = &s.mineralogy;
        let stone = Layers { top: blocks.stone, under: blocks.stone, depth: 0 };
        // The sea floor: sand in the shallows, gravel deeper.
        if z < SEA_LEVEL {
            let deep = SEA_LEVEL - z;
            let floor = if deep <= m.shallows { blocks.sand } else { blocks.gravel };
            return floor.map_or(stone, |floor| Layers { top: floor, under: floor, depth: m.sand_depth });
        }
        let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
        let (temperature, rain) = self.climate(north, east, z as f32).unwrap_or(NO_CLIMATE);
        // The lines between zones wander a little.
        let size = m.wander_size.max(1.0);
        let wobble = |salt: u64| perlin(self.seed, salt, Fade::Smooth, north / size, east / size) * m.wander;
        let (temperature, rain) = (temperature + wobble(0x5e0f), rain + wobble(0x7a1d));
        let earth = |top: BlockId| Layers { top, under: blocks.dirt.unwrap_or(blocks.stone), depth: if blocks.dirt.is_some() { m.dirt_depth } else { 0 } };
        let sand = |sand: BlockId| Layers { top: sand, under: sand, depth: m.sand_depth };
        if let Some(snow) = blocks.snow.filter(|_| temperature < m.snow_below) {
            return earth(snow);
        }
        if z >= s.gabbro_from {
            return Layers { top: blocks.gabbro, ..stone };
        }
        if z >= s.stone_from {
            return stone;
        }
        if let Some(beach) = blocks.sand.filter(|_| z < SEA_LEVEL + m.beach) {
            return sand(beach);
        }
        if let Some(desert) = blocks.sand.filter(|_| rain < m.desert_below && temperature > m.desert_above) {
            return sand(desert);
        }
        if let Some(dry) = blocks.dry_grass.filter(|_| rain < m.dry_below) {
            return earth(dry);
        }
        earth(blocks.grass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::interpreter::{InterpreterSettings, Maps};
    use std::sync::Arc;

    /// Every block registered, numbered by hand.
    fn all_blocks() -> TerrainBlocks {
        TerrainBlocks {
            gabbro: BlockId(1),
            stone: BlockId(2),
            grass: BlockId(3),
            water: BlockId(4),
            dirt: Some(BlockId(5)),
            sand: Some(BlockId(6)),
            dry_grass: Some(BlockId(7)),
            snow: Some(BlockId(8)),
            gravel: Some(BlockId(9)),
        }
    }

    /// A flat map with one temperature and one rain everywhere (or no climate), no wandering, cooling
    /// `cooling` for every 100 blocks up.
    fn cooled(climate: Option<(f32, f32)>, cooling: f32) -> CompiledMap {
        let width = 8;
        let (temperature, rain) = climate.map_or((Vec::new(), Vec::new()), |(t, r)| (vec![t; width * width], vec![r; width * width]));
        let maps = Maps { width, blocks_per_pixel: 32.0, temperature, rain, cooling, coast: vec![1.0; width * width], height: vec![0.0; width * width], tilt: vec![0.0; width * width], squeeze: vec![0.0; width * width], push_x: vec![0.0; width * width], push_y: vec![0.0; width * width], push_strength: vec![0.0; width * width], coast_distance: vec![100.0; width * width], lift_distance: vec![1_000.0; width * width], ..Maps::default() };
        let mut settings = InterpreterSettings::default();
        settings.mineralogy.wander = 0.0;
        CompiledMap::new(Arc::new(maps), 1, settings)
    }

    /// The same, not cooling with height.
    fn map(climate: Option<(f32, f32)>) -> CompiledMap {
        cooled(climate, 0.0)
    }

    #[test]
    fn the_climate_picks_the_surface_and_dirt_lies_under_grass() {
        let blocks = all_blocks();
        let top = |climate, z| map(climate).layers(0, 0, z, &blocks).top;
        // Mild and wet: grass, with dirt under it.
        let grass = map(Some((0.2, 0.6))).layers(0, 0, 150, &blocks);
        assert_eq!(grass, Layers { top: blocks.grass, under: BlockId(5), depth: 3 });
        // Cold: snowy ground, even on the peaks; dry: dry grass; hot and very dry: desert sand.
        assert_eq!(top(Some((-0.8, 0.6)), 150), BlockId(8));
        assert_eq!(top(Some((-0.8, 0.6)), 400), BlockId(8));
        assert_eq!(top(Some((0.2, 0.2)), 150), BlockId(7));
        assert_eq!(top(Some((0.5, 0.05)), 150), BlockId(6));
        // Peaks and mountainsides by height; beaches; the sea floor, sand then gravel.
        assert_eq!(top(Some((0.2, 0.6)), 400), blocks.gabbro);
        assert_eq!(top(Some((0.2, 0.6)), 250), blocks.stone);
        assert_eq!(top(Some((0.2, 0.6)), 129), BlockId(6));
        assert_eq!(top(Some((0.2, 0.6)), 124), BlockId(6));
        assert_eq!(top(Some((0.2, 0.6)), 100), BlockId(9));
    }

    #[test]
    fn higher_is_colder_so_snow_starts_up_a_mountain() {
        // Mild at sea level, cooling 0.3 per 100 blocks: snowy above about 340 (with the defaults).
        let map = cooled(Some((0.2, 0.6)), 0.3);
        let blocks = all_blocks();
        let mut settings = map.settings.clone();
        settings.gabbro_from = 1_000;
        settings.stone_from = 1_000;
        let map = map.with(settings, Default::default());
        assert_eq!(map.layers(0, 0, 300, &blocks).top, blocks.grass);
        assert_eq!(map.layers(0, 0, 400, &blocks).top, BlockId(8));
    }

    #[test]
    fn a_block_not_registered_yet_leaves_the_ground_as_it_was() {
        // Only the first blocks: no snow, sand, dry grass, dirt or gravel.
        let blocks = TerrainBlocks { dirt: None, sand: None, dry_grass: None, snow: None, gravel: None, ..all_blocks() };
        for climate in [Some((-0.8, 0.6)), Some((0.5, 0.05)), Some((0.2, 0.2)), None] {
            let map = map(climate);
            assert_eq!(map.layers(0, 0, 150, &blocks), Layers { top: blocks.grass, under: blocks.stone, depth: 0 });
            assert_eq!(map.layers(0, 0, 100, &blocks).top, blocks.stone);
        }
        // No climate at all, every block registered: grass, as before climate.
        assert_eq!(map(None).layers(0, 0, 150, &all_blocks()).top, all_blocks().grass);
    }
}

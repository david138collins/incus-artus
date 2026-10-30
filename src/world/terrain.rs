//! Terrain from a compiled world — chunks you can walk on.
//!
//! Each column of blocks gets its ground height from the Interpreter (`interpreter.rs`: the World
//! Compiler's smooth maps, with the recipe worked out at that column), and is filled:
//! - **stone** below the surface, under a few layers of what the surface is made of (dirt under
//!   grass, sand under sand);
//! - the **surface** block, by climate and height: snowy ground, gabbro on the peaks, stone on the
//!   mountainsides, sand, dry grass, grass; sand or gravel on the sea floor (`mineralogy.rs`);
//! - **water** from the sea floor up to sea level (Z 127, so the water's surface is at Z 128);
//! - air above.
//!
//! Past the edge of the compiled map there are no chunks at all.
//!
//! Every chunk can be made on its own from the map, in any order, on any thread: that's what lets
//! chunks stream in around you, and distant terrain sample the same function more coarsely.
//! Only chunks you change are saved; the rest are made again from the map.

use super::block::BlockId;
use super::chunk::Chunk;
use super::position::{CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTION_VOLUME, SECTIONS_PER_CHUNK, WORLD_HEIGHT};
use super::section::Section;
use super::VoxelWorld;
use crate::data::BlockRegistry;
use super::interpreter::CompiledMap;
use bevy::prelude::*;
use std::sync::Arc;

/// Ground at this Z or above is dry land; water fills everything below it down to the sea floor.
pub const SEA_LEVEL: i32 = 128;

/// Is ground at this height (a Z, with fractions) under the sea? Where it rounds below sea level: the
/// one rule the near ground (`CompiledMap::ground`) and the distant terrain both go by, so they always
/// agree where the water is. [2026-10-30: the distant terrain took anything below 128 as sea, so
/// ground at 127.5 to 128, land up close, was water far away; flats right at sea level showed it.]
pub fn under_sea(height: f32) -> bool {
    (height.round() as i32) < SEA_LEVEL
}

/// The blocks the interpreter uses: the four it always has, and the ones mineralogy uses when they're
/// registered (none: its rules for them are skipped).
#[derive(Clone, Copy)]
pub struct TerrainBlocks {
    pub gabbro: BlockId,
    pub stone: BlockId,
    pub grass: BlockId,
    pub water: BlockId,
    pub dirt: Option<BlockId>,
    pub sand: Option<BlockId>,
    pub dry_grass: Option<BlockId>,
    pub snow: Option<BlockId>,
    pub gravel: Option<BlockId>,
}

impl TerrainBlocks {
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        let get = |name: &str| registry.id(name).unwrap_or_else(|| panic!("the terrain needs block '{name}'"));
        Self {
            gabbro: get("incus:gabbro"),
            stone: get("incus:stone"),
            grass: get("incus:grass"),
            water: get("incus:water"),
            dirt: registry.id("incus:dirt"),
            sand: registry.id("incus:sand"),
            dry_grass: registry.id("incus:dry_grass"),
            snow: registry.id("incus:snowy_ground"),
            gravel: registry.id("incus:gravel"),
        }
    }
}

/// Filling columns with blocks (the ground's height itself is the interpreter's: see
/// `interpreter.rs`).
impl CompiledMap {
    /// The ground's Z in the column of blocks at (`x` north, `y` east), or `None` past the map.
    /// Sea floor is at least one block under the water's surface: ground at Z 127 would leave no
    /// room for water above it, a stone floor flush with the sea that the distant terrain and the
    /// terrain lab both draw as water.
    pub fn ground(&self, x: i32, y: i32) -> Option<i32> {
        self.height(x as f32 + 0.5, y as f32 + 0.5).map(|z| {
            let z = (z.round() as i32).clamp(1, WORLD_HEIGHT - 1);
            if under_sea(z as f32) { z.min(SEA_LEVEL - 2) } else { z }
        })
    }

    /// One chunk, or `None` if it's wholly past the edge of the map.
    pub fn chunk(&self, pos: ChunkPos, blocks: &TerrainBlocks) -> Option<Chunk> {
        let origin = pos.min_block();
        let size = CHUNK_SIZE as usize;
        // Each column's ground (−1 = past the map) and what it's made of (mineralogy).
        let mut ground = vec![-1i32; size * size];
        let mut layers = vec![super::Layers { top: BlockId::AIR, under: blocks.stone, depth: 0 }; size * size];
        for local_y in 0..size {
            for local_x in 0..size {
                let column = local_x + local_y * size;
                if let Some(z) = self.ground(origin.x + local_x as i32, origin.y + local_y as i32) {
                    ground[column] = z;
                    layers[column] = self.layers(origin.x + local_x as i32, origin.y + local_y as i32, z, blocks);
                }
            }
        }
        let inside: Vec<&i32> = ground.iter().filter(|&&z| z >= 0).collect();
        if inside.is_empty() {
            return None;
        }
        let all_inside = inside.len() == size * size;
        // How deep the surface's layers go anywhere in the chunk (stone below that).
        let deepest_layer = layers.iter().map(|layer| layer.depth).max().unwrap_or(0);
        let lowest = inside.iter().copied().copied().min().unwrap_or(0);
        // The highest thing in the chunk: ground, or the sea's surface.
        let highest = inside.iter().copied().copied().max().unwrap_or(0).max(SEA_LEVEL - 1);

        let mut sections = Vec::with_capacity(SECTIONS_PER_CHUNK);
        let mut buffer = vec![BlockId::AIR; SECTION_VOLUME];
        for level in 0..SECTIONS_PER_CHUNK {
            let bottom = level as i32 * SECTION_SIZE;
            let top_z = bottom + SECTION_SIZE - 1;
            // Shortcuts: whole sections of sky, or of solid stone, need no per-block work.
            if bottom > highest {
                sections.push(Section::Uniform(BlockId::AIR));
                continue;
            }
            if all_inside && top_z < lowest - deepest_layer {
                sections.push(Section::Uniform(blocks.stone));
                continue;
            }
            for local_z in 0..SECTION_SIZE as usize {
                let z = bottom + local_z as i32;
                let layer = &mut buffer[local_z * size * size..(local_z + 1) * size * size];
                for column in 0..size * size {
                    let g = ground[column];
                    layer[column] = if g < 0 || z >= WORLD_HEIGHT {
                        BlockId::AIR
                    } else if z < g - layers[column].depth {
                        blocks.stone
                    } else if z < g {
                        layers[column].under
                    } else if z == g {
                        layers[column].top
                    } else if z < SEA_LEVEL {
                        blocks.water
                    } else {
                        BlockId::AIR
                    };
                }
            }
            sections.push(Section::from_blocks(&buffer));
        }
        Some(Chunk::from_sections(pos, sections))
    }
}

/// What the terrain is made from: the test cylinder, or a compiled world.
#[derive(Resource, Clone, Default)]
pub enum Terrain {
    #[default]
    Test,
    Compiled(Arc<CompiledMap>),
}

impl Terrain {
    pub fn compiled(&self) -> Option<&Arc<CompiledMap>> {
        match self {
            Terrain::Compiled(map) => Some(map),
            Terrain::Test => None,
        }
    }
}

/// The compiled world the next new world is made from, and where to spawn in it (north, east): set
/// by the World Compiler's "Render World and Spawn"; used once.
#[derive(Resource, Default)]
pub struct NextTerrain(pub Option<(Arc<CompiledMap>, IVec2)>);

/// Makes every chunk within `radius` chunks of `centre` (a circle), on all the computer's cores,
/// and puts them in the world. Returns how many were made.
pub fn generate_around(world: &mut VoxelWorld, map: &CompiledMap, blocks: &TerrainBlocks, centre: ChunkPos, radius: i32) -> usize {
    let mut wanted = Vec::new();
    for x in -radius..=radius {
        for y in -radius..=radius {
            if x * x + y * y <= radius * radius {
                wanted.push(ChunkPos::new(centre.x + x, centre.y + y));
            }
        }
    }
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let each = wanted.len().div_ceil(cores).max(1);
    let made: Vec<Chunk> = std::thread::scope(|scope| {
        let jobs: Vec<_> = wanted.chunks(each).map(|part| scope.spawn(move || part.iter().filter_map(|&pos| map.chunk(pos, blocks)).collect::<Vec<_>>())).collect();
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_default()).collect()
    });
    let count = made.len();
    for chunk in made {
        world.insert_chunk(chunk);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny map: 8 × 8 pixels of 32 blocks; land rising from the sea in the west to Z 250 in the
    /// east.
    fn map() -> CompiledMap {
        let width = 8;
        let z: Vec<f32> = (0..width * width).map(|i| 100.0 + (i % width) as f32 * 22.0).collect();
        CompiledMap::of_heights(width, 32.0, &z)
    }

    fn blocks() -> TerrainBlocks {
        TerrainBlocks::from_registry(&BlockRegistry::default())
    }

    #[test]
    fn the_ground_follows_the_map_smoothly() {
        let map = map();
        // At a pixel's centre, the map's own value.
        let east_of_pixel = |col: f32| (col + 0.5 - 4.0) * 32.0;
        assert!((map.height(0.0 - 16.0, east_of_pixel(2.0)).unwrap() - 144.0).abs() < 0.01);
        // Between two pixels, between their values, with no steps from one block to the next.
        let mut last = map.height(-16.0, east_of_pixel(2.0)).unwrap();
        for k in 1..=32 {
            let now = map.height(-16.0, east_of_pixel(2.0) + k as f32).unwrap();
            assert!(now >= last && now - last < 1.5, "{last} → {now}");
            last = now;
        }
        // Past the map, nothing.
        assert!(map.height(0.0, 200.0).is_none());
    }

    #[test]
    fn columns_fill_with_stone_surface_water_and_air() {
        let map = map();
        let blocks = blocks();
        // A column in the sea (the west) and one on land (the east).
        for (x, y) in [(-16, -112), (-16, 80)] {
            let g = map.ground(x, y).unwrap();
            let chunk = map.chunk(ChunkPos::of_block(IVec3::new(x, y, 0)), &blocks).unwrap();
            let at = |z: i32| chunk.get(IVec3::new(x, y, z));
            // The surface, its layers, then stone (mineralogy).
            let layers = map.layers(x, y, g, &blocks);
            assert_eq!(at(g), layers.top);
            if layers.depth > 0 {
                assert_eq!(at(g - 1), layers.under);
            }
            assert_eq!(at(g - layers.depth - 1), blocks.stone);
            if g < SEA_LEVEL {
                assert!(g < SEA_LEVEL - 1, "a sea floor flush with the surface");
                assert_eq!(at(g + 1), blocks.water);
                assert_eq!(at(SEA_LEVEL - 1), blocks.water);
                assert_eq!(at(SEA_LEVEL), BlockId::AIR);
            } else {
                assert_eq!(at(g + 1), BlockId::AIR);
            }
        }
    }

    #[test]
    fn peaks_are_gabbro_mountainsides_stone_and_land_grass() {
        let map = map();
        // Without mineralogy's blocks (whatever's registered): the ground as it always was.
        let blocks = TerrainBlocks { dirt: None, sand: None, dry_grass: None, snow: None, gravel: None, ..blocks() };
        let top = |z: i32| map.layers(0, 0, z, &blocks).top;
        assert_eq!(top(300), blocks.gabbro);
        assert_eq!(top(230), blocks.stone);
        assert_eq!(top(140), blocks.grass);
        // The sea floor, with no sand or gravel registered: stone.
        assert_eq!(top(90), blocks.stone);
    }


    #[test]
    fn land_has_its_dirt_layers_between_the_surface_and_the_stone() {
        let map = map();
        let blocks = TerrainBlocks { dirt: Some(BlockId(50)), ..blocks() };
        // A low column on land (the map rises 22 blocks a pixel from Z 100 in the west).
        let (x, y) = (-16, -48);
        let g = map.ground(x, y).unwrap();
        assert!((SEA_LEVEL + 3..map.settings.stone_from).contains(&g), "a grassy column ({g})");
        let chunk = map.chunk(ChunkPos::of_block(IVec3::new(x, y, 0)), &blocks).unwrap();
        let at = |z: i32| chunk.get(IVec3::new(x, y, z));
        let depth = map.settings.mineralogy.dirt_depth;
        assert_eq!(at(g), blocks.grass);
        assert!((1..=depth).all(|k| at(g - k) == BlockId(50)));
        assert_eq!(at(g - depth - 1), blocks.stone);
    }

    #[test]
    fn ground_just_under_the_sea_s_surface_is_land_where_it_rounds_up() {
        // Flats right at sea level (a crag kept from drowning land stops at 127.99, say) are land up
        // close, a block above the water; the distant terrain asks the same question.
        assert!(!under_sea(127.99) && !under_sea(127.5) && !under_sea(128.0));
        assert!(under_sea(127.49) && under_sea(126.0));
        let map = CompiledMap::of_heights(8, 32.0, &[0.0; 64]);
        for x in -100..100 {
            let h = map.height(x as f32 + 0.5, 0.5).unwrap();
            assert_eq!(map.ground(x, 0).unwrap() < SEA_LEVEL, under_sea(h), "at {h}");
        }
    }
    #[test]
    fn chunks_past_the_map_are_nothing_and_generation_fills_a_circle() {
        let map = map();
        let blocks = blocks();
        assert!(map.chunk(ChunkPos::new(50, 50), &blocks).is_none());
        let mut world = VoxelWorld::default();
        let made = generate_around(&mut world, &map, &blocks, ChunkPos::new(0, 0), 3);
        assert!(made > 0 && made == world.chunk_count());
        // The same chunk made twice is the same.
        let (a, b) = (map.chunk(ChunkPos::new(0, 0), &blocks).unwrap(), map.chunk(ChunkPos::new(0, 0), &blocks).unwrap());
        for z in [100, 128, 140, 200] {
            assert_eq!(a.get(IVec3::new(3, 5, z)), b.get(IVec3::new(3, 5, z)));
        }
    }
}

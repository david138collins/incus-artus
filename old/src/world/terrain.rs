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
use super::caves::CavePlan;
use crate::data::BlockRegistry;
use super::interpreter::CompiledMap;
use crate::ecs::prelude::*;
use std::collections::HashMap;
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

/// The blocks the terrain always has (the rock, water, and gabbro and grass for the far view and the
/// tests), and every block by its name, for mineralogy's distributors (a block not in it isn't
/// registered: its candidates are skipped).
#[derive(Clone, PartialEq)]
pub struct TerrainBlocks {
    pub gabbro: BlockId,
    pub stone: BlockId,
    pub grass: BlockId,
    pub water: BlockId,
    pub names: Arc<HashMap<String, BlockId>>,
}

impl TerrainBlocks {
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        let get = |name: &str| registry.id(name).unwrap_or_else(|| panic!("the terrain needs block '{name}'"));
        let names = registry.defs().iter().filter_map(|def| Some((def.name.clone(), registry.id(&def.name)?))).collect();
        Self { gabbro: get("incus:gabbro"), stone: get("incus:stone"), grass: get("incus:grass"), water: get("incus:water"), names: Arc::new(names) }
    }
}

/// Filling columns with blocks (the ground's height itself is the interpreter's: see
/// `interpreter.rs`).
impl CompiledMap {
    /// The ground's Z in the column of blocks at (`x` north, `y` east), or `None` past the map.
    /// Sea floor is at least one block under the water's surface: ground at Z 127 would leave no
    /// room for water above it, a stone floor flush with the sea that the distant terrain and the
    /// terrain lab both draw as water.
    #[cfg(test)]
    pub fn ground(&self, x: i32, y: i32) -> Option<i32> {
        self.column(x, y).map(|(z, _)| z)
    }

    /// The column's ground (as `ground`), and the Z below which a river's or lake's water fills it
    /// (`carving.rs`), if any.
    pub fn column(&self, x: i32, y: i32) -> Option<(i32, Option<i32>)> {
        if let Some(whole) = self.whole() {
            // Whole numbers all the way to the block: a fraction can't hold a height's 65536ths above Z
            // 256, and a height a hair under a half could round the other way from the card's.
            use crate::whole::Pos;
            return whole.map.ground_and_water(&whole.carving, Pos::middle_of(x), Pos::middle_of(y)).map(|(z, water)| whole_block_column(z, water));
        }
        self.ground_and_water(x as f32 + 0.5, y as f32 + 0.5).map(|(z, water)| whole_column(z, water))
    }

    /// The slope at the column of blocks at (`x` north, `y` east), whose ground is at `ground`: the
    /// one measure every view uses (the near chunks, the distant terrain, the World Compiler), so a
    /// layer that reads it gets the same answer near and far [the Metaphysics Engine, 2026-11-01: "1
    /// always equals 1"]. It's measured across the whole-block ground `SLOPE_REACH` either side (a
    /// neighbour past the map counts as level), whole numbers until the last step.
    pub fn slope(&self, x: i32, y: i32, ground: i32) -> f32 {
        let at = |dx: i32, dy: i32| self.column(x + dx, y + dy).map_or(ground, |(z, _)| z);
        slope_across(at(SLOPE_REACH, 0) - at(-SLOPE_REACH, 0), at(0, SLOPE_REACH) - at(0, -SLOPE_REACH))
    }

    /// One chunk, or `None` if it's wholly past the edge of the map, with the caves in `caves` taken
    /// out of it (`caves.rs`: the column first, then every solid block inside a cave becomes air).
    pub fn chunk(&self, pos: ChunkPos, blocks: &TerrainBlocks, caves: &CavePlan) -> Option<Chunk> {
        let origin = pos.min_block();
        let size = CHUNK_SIZE as usize;
        // Each column's ground (−1 = past the map), what it's made of (mineralogy), and where a river's or
        // lake's water over it stops (0 = none).
        let mut ground = vec![-1i32; size * size];
        let mut layers = vec![super::Layers::rock(blocks.stone); size * size];
        let mut water = vec![0i32; size * size];
        for local_y in 0..size {
            for local_x in 0..size {
                if let Some((z, over)) = self.column(origin.x + local_x as i32, origin.y + local_y as i32) {
                    ground[local_x + local_y * size] = z;
                    water[local_x + local_y * size] = over.unwrap_or(0);
                }
            }
        }
        // What each column is made of (mineralogy), with its slope if anything reads it (as
        // `CompiledMap::slope` measures it, the neighbours past the chunk's edge asked for).
        let mineralogy = self.mineralogy(blocks);
        let ground_at = |x: i32, y: i32| -> Option<i32> {
            if (0..size as i32).contains(&x) && (0..size as i32).contains(&y) {
                Some(ground[x as usize + y as usize * size]).filter(|&z| z >= 0)
            } else {
                self.column(origin.x + x, origin.y + y).map(|(z, _)| z)
            }
        };
        for local_y in 0..size {
            for local_x in 0..size {
                let column = local_x + local_y * size;
                let z = ground[column];
                if z < 0 {
                    continue;
                }
                let (x, y) = (local_x as i32, local_y as i32);
                let slope = if mineralogy.reads_slope {
                    let at = |dx: i32, dy: i32| ground_at(x + dx, y + dy).unwrap_or(z);
                    slope_across(at(SLOPE_REACH, 0) - at(-SLOPE_REACH, 0), at(0, SLOPE_REACH) - at(0, -SLOPE_REACH))
                } else {
                    0.0
                };
                let (north, east) = ((origin.x + x) as f32 + 0.5, (origin.y + y) as f32 + 0.5);
                let over = Some(water[column]).filter(|&top| top > 0);
                layers[column] = mineralogy.layers(self, super::placer::Column { north, east, ground: z as f32, slope, climate: self.climate(north, east, z as f32), water: over });
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
        // The highest thing in the chunk: ground, the sea's surface, or a river's or lake's.
        let highest = inside.iter().copied().copied().max().unwrap_or(0).max(SEA_LEVEL - 1).max(water.iter().copied().max().unwrap_or(0) - 1);

        let mut sections = Vec::with_capacity(SECTIONS_PER_CHUNK);
        let mut buffer = vec![BlockId::AIR; SECTION_VOLUME];
        for level in 0..SECTIONS_PER_CHUNK {
            let bottom = level as i32 * SECTION_SIZE;
            let top_z = bottom + SECTION_SIZE - 1;
            // Shortcuts: whole sections of sky, or of solid stone with no cave near, need no
            // per-block work.
            if bottom > highest {
                sections.push(Section::Uniform(BlockId::AIR));
                continue;
            }
            let caved = caves.touches(pos, level);
            if all_inside && top_z < lowest - deepest_layer && !caved {
                sections.push(Section::Uniform(blocks.stone));
                continue;
            }
            for local_z in 0..SECTION_SIZE as usize {
                let z = bottom + local_z as i32;
                let layer = &mut buffer[local_z * size * size..(local_z + 1) * size * size];
                for column in 0..size * size {
                    let g = ground[column];
                    let block = if g < 0 || z >= WORLD_HEIGHT { BlockId::AIR } else { layers[column].block_at(z, g, water[column], blocks) };
                    // The caves, taken out (water never is).
                    let carved = caved && !block.is_air() && block != blocks.water && caves.inside(origin + IVec3::new((column % size) as i32, (column / size) as i32, z));
                    layer[column] = if carved { BlockId::AIR } else { block };
                }
            }
            sections.push(Section::from_blocks(&buffer));
        }
        Some(Chunk::from_sections(pos, sections))
    }
}

/// How far either side of a column its slope is measured (blocks).
pub const SLOPE_REACH: i32 = 2;

/// Blocks up per block across, from how much the ground rises across `2 × SLOPE_REACH` blocks north
/// and east. All whole numbers, the square root too (in 16ths of a block, within 32 bits), so every
/// chip gets the same answer exactly [GPU-safe: David, 2026-11-01]; then two divisions by powers of
/// two, which are exact.
pub(super) fn slope_across(north_rise: i32, east_rise: i32) -> f32 {
    let squared = (north_rise * north_rise + east_rise * east_rise).unsigned_abs().min(u32::MAX / 256);
    let sixteenths = (squared * 256).isqrt();
    sixteenths as f32 / 16.0 / (2 * SLOPE_REACH) as f32
}

/// The same as `whole_column` from a height in whole numbers (`whole_recipe`), rounded as the card does:
/// halves up.
pub fn whole_block_column(height: crate::whole::Fix, water: Option<i32>) -> (i32, Option<i32>) {
    let z = ((height.0 + crate::whole::ONE / 2) >> 16).clamp(1, WORLD_HEIGHT - 1);
    (if z < SEA_LEVEL { z.min(SEA_LEVEL - 2) } else { z }, water.filter(|&w| w > z + 1).map(|w| w.min(WORLD_HEIGHT)))
}

/// A column's ground and its river's or lake's water as the world fills it, from the ground's height
/// and the water's top: whole blocks, the sea floor at least a block under the sea's surface, and the
/// water only where it stands over the ground.
pub fn whole_column(height: f32, water: Option<i32>) -> (i32, Option<i32>) {
    let z = (height.round() as i32).clamp(1, WORLD_HEIGHT - 1);
    (if under_sea(z as f32) { z.min(SEA_LEVEL - 2) } else { z }, water.filter(|&w| w > z + 1).map(|w| w.min(WORLD_HEIGHT)))
}

impl super::Layers {
    /// The block at `z` in a column whose ground is at `ground`, made of these layers, with water up to
    /// the sea's surface and up to below `water` (a river's or lake's top; 0 for none): the one rule
    /// every column is filled by (the world's chunks, and the World Compiler's depth sweep).
    #[inline]
    pub fn block_at(&self, z: i32, ground: i32, water: i32, blocks: &TerrainBlocks) -> BlockId {
        if z < ground - self.depth {
            blocks.stone
        } else if z < ground {
            self.under_at(ground - z)
        } else if z == ground {
            self.top
        } else if z < SEA_LEVEL || z < water {
            blocks.water
        } else {
            BlockId::AIR
        }
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
pub fn generate_around(world: &mut VoxelWorld, map: &CompiledMap, blocks: &TerrainBlocks, caves: &CavePlan, centre: ChunkPos, radius: i32) -> usize {
    let mut wanted = Vec::new();
    for x in -radius..=radius {
        for y in -radius..=radius {
            if x * x + y * y <= radius * radius {
                wanted.push(ChunkPos::new(centre.x + x, centre.y + y));
            }
        }
    }
    let made = make_chunks(map, blocks, caves, &wanted);
    let count = made.len();
    for chunk in made {
        world.insert_chunk(chunk);
    }
    count
}

/// Makes the chunks at `places` on all the computer's cores (those past the map are left out).
pub fn make_chunks(map: &CompiledMap, blocks: &TerrainBlocks, caves: &CavePlan, places: &[ChunkPos]) -> Vec<Chunk> {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let each = places.len().div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        let jobs: Vec<_> = places.chunks(each).map(|part| scope.spawn(move || part.iter().filter_map(|&pos| map.chunk(pos, blocks, caves)).collect::<Vec<_>>())).collect();
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_default()).collect()
    })
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

    /// The registry's blocks, but only the four the terrain always had by name (whatever else this
    /// computer has registered).
    fn only_the_first_blocks() -> TerrainBlocks {
        let blocks = blocks();
        let names = blocks.names.iter().filter(|(name, _)| ["incus:gabbro", "incus:stone", "incus:grass", "incus:water"].contains(&name.as_str())).map(|(name, &id)| (name.clone(), id)).collect();
        TerrainBlocks { names: Arc::new(names), ..blocks }
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
            let chunk = map.chunk(ChunkPos::of_block(IVec3::new(x, y, 0)), &blocks, &CavePlan::default()).unwrap();
            let at = |z: i32| chunk.get(IVec3::new(x, y, z));
            // The surface, its layers, then stone (mineralogy).
            let layers = map.layers(x, y, g, &blocks);
            assert_eq!(at(g), layers.top);
            if layers.depth > 0 {
                assert_eq!(at(g - 1), layers.under_at(1));
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
        // Without mineralogy's other blocks (whatever's registered): the ground as it always was.
        let blocks = only_the_first_blocks();
        let top = |z: i32| map.layers(0, 0, z, &blocks).top;
        assert_eq!(top(340), blocks.gabbro);
        assert_eq!(top(300), blocks.stone);
        assert_eq!(top(140), blocks.grass);
        // The sea floor, with no sand or gravel registered: stone.
        assert_eq!(top(90), blocks.stone);
    }


    #[test]
    fn land_has_its_dirt_layers_between_the_surface_and_the_stone() {
        let map = map();
        let mut blocks = only_the_first_blocks();
        Arc::make_mut(&mut blocks.names).insert("incus:dirt".into(), BlockId(50));
        // A low column on land (the map rises 22 blocks a pixel from Z 100 in the west).
        let (x, y) = (-16, -48);
        let g = map.ground(x, y).unwrap();
        assert!((SEA_LEVEL + 3..256).contains(&g), "a grassy column ({g})");
        let chunk = map.chunk(ChunkPos::of_block(IVec3::new(x, y, 0)), &blocks, &CavePlan::default()).unwrap();
        let at = |z: i32| chunk.get(IVec3::new(x, y, z));
        // The surface's dirt goes 2 down (David's).
        let depth = 2;
        assert_eq!(at(g), blocks.grass);
        assert!((1..=depth).all(|k| at(g - k) == BlockId(50)));
        assert_eq!(at(g - depth - 1), blocks.stone);
    }

    #[test]
    fn a_layer_reading_the_slope_gets_the_same_answer_in_the_chunks_as_everywhere_else() {
        // Sand where steeper than 0.6 (the map rises about 0.7 a block, so its whole blocks are
        // sometimes steeper and sometimes not), grass elsewhere.
        use crate::world::placer::{Candidate, Column, Curve, Distributor, Layer, Source};
        let mut map = map();
        let steep = Layer { source: Source::Field { key: "ground/slope".into() }, curve: Curve::Step { at: 0.6, above: true, soft: 0.0 }, ..Layer::default() };
        let candidates = vec![Candidate { block: "incus:sand".into(), layers: vec![steep], ..Candidate::default() }, Candidate { block: "incus:grass".into(), ..Candidate::default() }];
        map.settings.mineralogy.distributors = vec![Distributor { candidates, ..Distributor::default() }];
        let mut blocks = only_the_first_blocks();
        Arc::make_mut(&mut blocks.names).insert("incus:sand".into(), BlockId(60));
        let mineralogy = map.mineralogy(&blocks);
        assert!(mineralogy.reads_slope);
        let pos = ChunkPos::of_block(IVec3::new(-16, -40, 0));
        let chunk = map.chunk(pos, &blocks, &CavePlan::default()).unwrap();
        let (mut sand, mut grass) = (0, 0);
        for x in pos.min_block().x..pos.min_block().x + CHUNK_SIZE {
            for y in pos.min_block().y..pos.min_block().y + CHUNK_SIZE {
                let Some((g, _)) = map.column(x, y) else { continue };
                if g < SEA_LEVEL {
                    continue;
                }
                // As the distant terrain and the World Compiler ask it.
                let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
                let column = Column { north, east, ground: g as f32, slope: map.slope(x, y, g), climate: None, water: None };
                let top = mineralogy.layers(&map, column).top;
                assert_eq!(chunk.get(IVec3::new(x, y, g)), top, "at ({x}, {y})");
                if top == BlockId(60) { sand += 1 } else { grass += 1 }
            }
        }
        assert!(sand > 50 && grass > 50, "{sand} steep, {grass} not");
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
        assert!(map.chunk(ChunkPos::new(50, 50), &blocks, &CavePlan::default()).is_none());
        let mut world = VoxelWorld::default();
        let made = generate_around(&mut world, &map, &blocks, &CavePlan::default(), ChunkPos::new(0, 0), 3);
        assert!(made > 0 && made == world.chunk_count());
        // The same chunk made twice is the same.
        let (a, b) = (map.chunk(ChunkPos::new(0, 0), &blocks, &CavePlan::default()).unwrap(), map.chunk(ChunkPos::new(0, 0), &blocks, &CavePlan::default()).unwrap());
        for z in [100, 128, 140, 200] {
            assert_eq!(a.get(IVec3::new(3, 5, z)), b.get(IVec3::new(3, 5, z)));
        }
    }
}

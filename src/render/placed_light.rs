//! Placed light — the light things give, spreading through the air [`docs/objects.md`, stop 3;
//! `docs/caves.md` section 11, the second kind of light].
//!
//! - **From each light:** an object with a light (a brazier, later a torch, glow lichen) floods it out
//!   from its spot through cells that aren't solid, round corners and down tunnels, an even step
//!   darker each block to black at its reach. It stops at walls: rock between you and a brazier keeps
//!   its light from you.
//! - **In colour:** each cell keeps red, green and blue, 16 levels each (its colour, times its
//!   brightness, times how near). Where two lights meet, each colour is the brighter of the two.
//! - **Stored in the faces:** each face takes the light of the cell in front of it, as daylight is,
//!   in the vertex's third word (`mesher.rs`). So there's no limit to how many lights there are; a
//!   light costs only the faces near it.
//! - **Worked out when it changes:** when a light is placed or taken (its chunk's objects drawn again,
//!   `objects.rs`), when blocks change near it, or when the chunks round it come in. The sections it
//!   reaches are made again.
//! - **Whole numbers** throughout: a walk of at most `reach` steps (15 at most) from each light.

use crate::world::{BlockId, CHUNK_SIZE, ChunkPos, SECTION_SIZE, SectionKey, VoxelWorld, WORLD_HEIGHT};
use bevy::math::IVec3;
use bevy::prelude::Resource;
use std::collections::{HashMap, HashSet, VecDeque};

/// The farthest a placed light reaches (blocks).
pub const MOST_REACH: i32 = 15;

/// A light's colour at full, 0 to 15 for each of red, green and blue, packed (red in the low 4 bits).
pub fn pack(rgb: [u32; 3]) -> u16 {
    (rgb[0].min(15) | rgb[1].min(15) << 4 | rgb[2].min(15) << 8) as u16
}

fn unpack(packed: u16) -> [u32; 3] {
    [packed as u32 & 15, (packed as u32 >> 4) & 15, (packed as u32 >> 8) & 15]
}

/// The brighter of two packed lights, colour by colour.
pub fn brighter(a: u16, b: u16) -> u16 {
    let (a, b) = (unpack(a), unpack(b));
    pack([a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])])
}

/// One light: where it is, its colour at full, how far it reaches, and the cells it lights.
#[derive(Clone, PartialEq)]
struct Source {
    cell: IVec3,
    colour: [u32; 3],
    reach: i32,
    cells: HashMap<IVec3, u16>,
}

/// A light as an object gives it: which object (its feet, 16ths), its cell, its colour at full (0 to
/// 15 each) and its reach (blocks).
#[derive(Clone, Copy, PartialEq)]
pub struct LightFrom {
    pub object: IVec3,
    pub cell: IVec3,
    pub colour: [u32; 3],
    pub reach: i32,
}

/// Every placed light, and what it lights.
#[derive(Resource, Default)]
pub struct PlacedLights {
    sources: HashMap<IVec3, Source>,
    /// The lights each chunk's objects give (the objects' feet).
    by_chunk: HashMap<ChunkPos, Vec<IVec3>>,
}

/// A light flooded out from `cell` through cells that aren't solid: full (its colour) at its cell,
/// an even step darker each block on, none at `reach`. Unloaded chunks count as solid.
fn flood(world: &VoxelWorld, is_opaque: &dyn Fn(BlockId) -> bool, cell: IVec3, colour: [u32; 3], reach: i32) -> HashMap<IVec3, u16> {
    let reach = reach.clamp(1, MOST_REACH);
    let solid = |at: IVec3| at.z < 0 || (at.z < WORLD_HEIGHT && (world.chunk(ChunkPos::of_block(at)).is_none() || is_opaque(world.get_block(at))));
    let mut cells = HashMap::new();
    let mut queue = VecDeque::from([(cell, 0)]);
    let mut seen = HashSet::from([cell]);
    while let Some((at, steps)) = queue.pop_front() {
        let level = colour.map(|full| full * (reach - steps) as u32 / reach as u32);
        if level == [0; 3] {
            continue;
        }
        cells.insert(at, pack(level));
        for step in [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z] {
            let next = at + step;
            if steps + 1 < reach && !solid(next) && seen.insert(next) {
                queue.push_back((next, steps + 1));
            }
        }
    }
    cells
}

/// The sections whose faces may look into these cells (each cell's own, and those beside it).
fn sections_of<'a>(cells: impl Iterator<Item = &'a IVec3>) -> HashSet<SectionKey> {
    let mut sections = HashSet::new();
    for cell in cells {
        for step in [IVec3::ZERO, IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z] {
            let at = *cell + step;
            if (0..WORLD_HEIGHT).contains(&at.z) {
                sections.insert(SectionKey { chunk: ChunkPos::of_block(at), level: (at.z / SECTION_SIZE) as u8 });
            }
        }
    }
    sections
}

impl PlacedLights {
    /// A chunk's lights as its objects give them now: those gone are taken away, new ones flooded
    /// out, the same ones kept. The sections whose light changed.
    pub fn set_chunk(&mut self, world: &VoxelWorld, is_opaque: &dyn Fn(BlockId) -> bool, chunk: ChunkPos, lights: &[LightFrom]) -> HashSet<SectionKey> {
        let mut changed = HashSet::new();
        let old = self.by_chunk.remove(&chunk).unwrap_or_default();
        for object in &old {
            let same = lights.iter().any(|light| light.object == *object && self.sources.get(object).is_some_and(|source| source.cell == light.cell && source.colour == light.colour && source.reach == light.reach));
            if !same && let Some(source) = self.sources.remove(object) {
                changed.extend(sections_of(source.cells.keys()));
            }
        }
        for light in lights {
            if let std::collections::hash_map::Entry::Vacant(slot) = self.sources.entry(light.object) {
                let cells = flood(world, is_opaque, light.cell, light.colour, light.reach);
                changed.extend(sections_of(cells.keys()));
                slot.insert(Source { cell: light.cell, colour: light.colour, reach: light.reach, cells });
            }
        }
        if !lights.is_empty() {
            self.by_chunk.insert(chunk, lights.iter().map(|light| light.object).collect());
        }
        changed
    }

    /// Lights reaching any of these cells flooded out again (blocks there changed, or came in). The
    /// sections whose light changed.
    pub fn reflood_near(&mut self, world: &VoxelWorld, is_opaque: &dyn Fn(BlockId) -> bool, low: IVec3, high: IVec3) -> HashSet<SectionKey> {
        let mut changed = HashSet::new();
        for source in self.sources.values_mut() {
            let reach = IVec3::splat(source.reach + 1);
            if (source.cell - reach).cmpgt(high).any() || (source.cell + reach).cmplt(low).any() {
                continue;
            }
            let cells = flood(world, is_opaque, source.cell, source.colour, source.reach);
            if cells != source.cells {
                changed.extend(sections_of(source.cells.keys().chain(cells.keys())));
                source.cells = cells;
            }
        }
        changed
    }

    /// The light of the cells round a section (its cells and the one block round it, from its
    /// lowest corner `origin`), by their place relative to it; cells with none are left out.
    pub fn around_section(&self, origin: IVec3) -> HashMap<[i32; 3], u16> {
        let (low, high) = (origin - IVec3::ONE, origin + IVec3::splat(SECTION_SIZE));
        let mut found: HashMap<[i32; 3], u16> = HashMap::new();
        for source in self.sources.values() {
            let reach = IVec3::splat(source.reach);
            if (source.cell - reach).cmpgt(high).any() || (source.cell + reach).cmplt(low).any() {
                continue;
            }
            for (cell, &light) in &source.cells {
                if cell.cmpge(low).all() && cell.cmple(high).all() {
                    let at = (*cell - origin).to_array();
                    found.entry(at).and_modify(|was| *was = brighter(*was, light)).or_insert(light);
                }
            }
        }
        found
    }

    /// The chunks whose objects give light.
    pub fn chunks(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.by_chunk.keys().copied()
    }

    /// The light at a cell (packed), from every light reaching it.
    pub fn at(&self, cell: IVec3) -> u16 {
        self.sources.values().filter_map(|source| source.cells.get(&cell)).fold(0, |a, &b| brighter(a, b))
    }

    /// Is any light within reach of this chunk column (so its sections may need lighting)?
    pub fn near_chunk(&self, chunk: ChunkPos) -> bool {
        let low = IVec3::new(chunk.x * CHUNK_SIZE, chunk.y * CHUNK_SIZE, 0);
        let high = low + IVec3::new(CHUNK_SIZE - 1, CHUNK_SIZE - 1, WORLD_HEIGHT);
        self.sources.values().any(|source| {
            let reach = IVec3::splat(source.reach + 1);
            !((source.cell - reach).cmpgt(high).any() || (source.cell + reach).cmplt(low).any())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{TestBlocks, TestWorldSettings, generate_test_world};

    #[test]
    fn a_light_spreads_through_the_air_fading_and_stops_at_walls() {
        let registry = crate::data::BlockRegistry::default();
        let blocks = TestBlocks::from_registry(&registry);
        let mut world = VoxelWorld::default();
        generate_test_world(&mut world, &TestWorldSettings::default(), &blocks);
        let stone = registry.id("incus:stone").unwrap();
        let opaque = |id: BlockId| !id.is_air();
        // On the test world's ground (grass at Z 144): a green light at (0, 0, 145), reaching 5.
        let cell = IVec3::new(0, 0, 145);
        // A wall of stone at x = 2, from y = -3 to 3, three high.
        for y in -3..=3 {
            for z in 145..=147 {
                world.set_block(IVec3::new(2, y, z), stone);
            }
        }
        let mut lights = PlacedLights::default();
        let light = LightFrom { object: cell * 16, cell, colour: [0, 15, 0], reach: 5 };
        let changed = lights.set_chunk(&world, &opaque, ChunkPos::of_block(cell), &[light]);
        assert!(changed.contains(&SectionKey { chunk: ChunkPos::of_block(cell), level: (145 / SECTION_SIZE) as u8 }));
        let green = |lights: &PlacedLights, at: IVec3| unpack(lights.at(at))[1];
        // Full at its cell, a step darker each block, none at its reach; none in the ground.
        assert_eq!((green(&lights, cell), green(&lights, cell + IVec3::NEG_X), green(&lights, cell + IVec3::NEG_X * 2)), (15, 12, 9));
        assert_eq!(green(&lights, cell + IVec3::NEG_X * 5), 0);
        assert_eq!(green(&lights, cell + IVec3::NEG_Z), 0, "the ground is solid");
        // Just behind the wall (x = 3, level with the light): the way round is long, so it's darker
        // than it would be through the wall (2 steps: 9); over the top it's 5 steps away: none.
        assert!(green(&lights, IVec3::new(3, 0, 145)) < 9, "{}", green(&lights, IVec3::new(3, 0, 145)));
        // The wall taken down: the light floods through, and the sections there change.
        for y in -3..=3 {
            for z in 145..=147 {
                world.set_block(IVec3::new(2, y, z), BlockId::AIR);
            }
        }
        let changed = lights.reflood_near(&world, &opaque, IVec3::new(2, -3, 145), IVec3::new(2, 3, 147));
        assert!(!changed.is_empty());
        assert_eq!(green(&lights, IVec3::new(3, 0, 145)), 6);
        // The section's cells, relative to it.
        let origin = IVec3::new(0, 0, (145 / SECTION_SIZE) * SECTION_SIZE);
        assert_eq!(lights.around_section(origin).get(&(cell - origin).to_array()), Some(&pack([0, 15, 0])));
        // Taken away: gone, and the section changes again.
        let changed = lights.set_chunk(&world, &opaque, ChunkPos::of_block(cell), &[]);
        assert!(!changed.is_empty() && lights.at(cell) == 0);
    }
}

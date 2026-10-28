//! World sizes and coordinate maths.
//!
//! A block position is an `IVec3` (three whole numbers) in game axes: X north, Y east, Z up.
//!
//! The world is split into **chunks**: columns 32 × 32 blocks wide and the full 1,024 tall.
//! Each chunk is split vertically into 32 **sections** of 32 × 32 × 32. [Was 400 tall (13 sections)
//! until 2026-10-27: raised for mountains that tower over the land (David).]

use bevy::math::IVec3;

/// Width of a chunk along X and Y, in blocks.
pub const CHUNK_SIZE: i32 = 32;
/// Edge length of a cube-shaped section, in blocks.
pub const SECTION_SIZE: i32 = 32;
/// Blocks in one section (32 × 32 × 32 = 32,768).
pub const SECTION_VOLUME: usize = (SECTION_SIZE * SECTION_SIZE * SECTION_SIZE) as usize;
/// Valid Z values are 0 ..= 1,023.
pub const WORLD_HEIGHT: i32 = 1_024;
/// Sections stacked in one chunk (1,024 / 32 = 32).
pub const SECTIONS_PER_CHUNK: usize = ((WORLD_HEIGHT + SECTION_SIZE - 1) / SECTION_SIZE) as usize;

/// Which chunk column, counted in chunks (chunk (1, 0) starts at block X = 32).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
}

impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// The chunk that contains a block. `div_euclid` rounds toward negative infinity,
    /// so block X = −1 correctly lands in chunk −1, not chunk 0.
    pub fn of_block(block: IVec3) -> Self {
        Self::new(block.x.div_euclid(CHUNK_SIZE), block.y.div_euclid(CHUNK_SIZE))
    }

    /// The block at this chunk's south-west bottom corner.
    pub fn min_block(self) -> IVec3 {
        IVec3::new(self.x * CHUNK_SIZE, self.y * CHUNK_SIZE, 0)
    }
}

/// Identifies one section in the world: which chunk, and how high (0 = bottom).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectionKey {
    pub chunk: ChunkPos,
    pub level: u8,
}

/// True if Z is inside the world (0 ..= 1,023).
pub fn z_in_world(z: i32) -> bool {
    (0..WORLD_HEIGHT).contains(&z)
}

/// Which section level holds this Z (assumes Z is in the world).
pub fn section_level(z: i32) -> usize {
    (z / SECTION_SIZE) as usize
}

/// Where a block sits inside its section, as a single number 0 ..= 32,767.
/// Layout: X varies fastest, then Y, then Z.
pub fn index_in_section(block: IVec3) -> usize {
    let x = block.x.rem_euclid(SECTION_SIZE) as usize;
    let y = block.y.rem_euclid(SECTION_SIZE) as usize;
    let z = block.z.rem_euclid(SECTION_SIZE) as usize;
    let size = SECTION_SIZE as usize;
    x + y * size + z * size * size
}

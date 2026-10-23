//! Test world generator — a flat cylinder for building and testing systems.
//!
//! David's spec: 100 blocks across, centred on (0, 0). Gabbro bedrock at Z = 0,
//! stone from Z = 1 up to the grass, one layer of grass at Z = 144. Nothing outside.
//!
//! The real world generator ("planet forming", geology, biomes...) replaces this later.

use super::block::BlockId;
use super::chunk::Chunk;
use super::position::{
    CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTION_VOLUME, SECTIONS_PER_CHUNK, WORLD_HEIGHT,
};
use super::section::Section;
use super::VoxelWorld;
use crate::data::BlockRegistry;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Tunables for the test world (shown on the "World" debug page, and saved with each world).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TestWorldSettings {
    /// Cylinder radius in blocks (50 = 100 blocks across).
    pub radius: f32,
    /// Z of the single grass layer.
    pub grass_z: i32,
}

impl Default for TestWorldSettings {
    fn default() -> Self {
        Self {
            radius: 50.0,
            grass_z: 144,
        }
    }
}

/// The block types the test world uses, looked up by name.
pub struct TestBlocks {
    pub bedrock: BlockId,
    pub stone: BlockId,
    pub grass: BlockId,
}

impl TestBlocks {
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        let get = |name: &str| {
            registry
                .id(name)
                .unwrap_or_else(|| panic!("test world needs block '{name}'"))
        };
        Self {
            bedrock: get("incus:gabbro"),
            stone: get("incus:stone"),
            grass: get("incus:grass"),
        }
    }
}

/// Clear the world and build the test cylinder. Returns how many chunks were made.
pub fn generate_test_world(
    world: &mut VoxelWorld,
    settings: &TestWorldSettings,
    blocks: &TestBlocks,
) -> usize {
    world.clear();
    let reach = settings.radius.ceil() as i32;
    let min = (-reach).div_euclid(CHUNK_SIZE);
    let max = (reach - 1).div_euclid(CHUNK_SIZE);
    for x in min..=max {
        for y in min..=max {
            if let Some(chunk) = generate_chunk(ChunkPos::new(x, y), settings, blocks) {
                world.insert_chunk(chunk);
            }
        }
    }
    world.chunk_count()
}

/// One chunk of the cylinder, or `None` if no part of the cylinder touches it.
fn generate_chunk(pos: ChunkPos, settings: &TestWorldSettings, blocks: &TestBlocks) -> Option<Chunk> {
    let origin = pos.min_block();
    let size = CHUNK_SIZE as usize;

    // Which of the 32 × 32 columns fall inside the circle (measured at block centres)?
    let radius_squared = settings.radius * settings.radius;
    let mut inside = vec![false; size * size];
    for local_y in 0..size {
        for local_x in 0..size {
            let x = (origin.x + local_x as i32) as f32 + 0.5;
            let y = (origin.y + local_y as i32) as f32 + 0.5;
            inside[local_x + local_y * size] = x * x + y * y <= radius_squared;
        }
    }
    let inside_count = inside.iter().filter(|&&i| i).count();
    if inside_count == 0 {
        return None;
    }
    let fully_inside = inside_count == size * size;

    let mut sections = Vec::with_capacity(SECTIONS_PER_CHUNK);
    let mut buffer = vec![BlockId::AIR; SECTION_VOLUME];
    for level in 0..SECTIONS_PER_CHUNK {
        let bottom = level as i32 * SECTION_SIZE;
        let top = bottom + SECTION_SIZE - 1;

        // Shortcuts: whole sections of sky, or of solid stone, need no per-block work.
        if bottom > settings.grass_z {
            sections.push(Section::Uniform(BlockId::AIR));
            continue;
        }
        if fully_inside && bottom >= 1 && top < settings.grass_z {
            sections.push(Section::Uniform(blocks.stone));
            continue;
        }

        for local_z in 0..SECTION_SIZE as usize {
            let z = bottom + local_z as i32;
            let layer = if z >= WORLD_HEIGHT || z > settings.grass_z {
                BlockId::AIR
            } else if z == 0 {
                blocks.bedrock
            } else if z < settings.grass_z {
                blocks.stone
            } else {
                blocks.grass
            };
            for column in 0..size * size {
                buffer[column + local_z * size * size] =
                    if inside[column] { layer } else { BlockId::AIR };
            }
        }
        sections.push(Section::from_blocks(&buffer));
    }
    Some(Chunk::from_sections(pos, sections))
}

//! Chunks — one 32 × 32 × 400 column of the world.
//!
//! A chunk holds:
//! - 13 stacked **sections** with the block IDs (the compact, every-block data).
//! - **Extra state** for the few blocks that need bulky data (container contents,
//!   mining progress, quarter-block layouts...). Stored in a lookup table keyed by
//!   position, so a block with no extra state costs nothing.
//!
//! It can also produce a **column summary** (the top block of each of its 1,024 columns).
//! That is the seed of the distant-terrain system: far away, a chunk can be drawn from
//! its summary alone, after the full block data is unloaded.

use super::block::BlockId;
use super::position::{
    CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTIONS_PER_CHUNK, WORLD_HEIGHT, index_in_section,
    section_level,
};
use super::pieces::Pieces;
use super::section::Section;
use bevy::math::IVec3;
use std::collections::HashMap;

/// Bulky per-block state. Each future system adds its own kind here
/// (e.g. `Container(...)`, `MiningProgress(...)`, `QuarterBlocks(...)`).
#[derive(Debug, Clone)]
pub enum BlockExtra {
    /// A text note attached from the debug panel, to test this storage works.
    DebugNote(String),
}

pub struct Chunk {
    pos: ChunkPos,
    sections: Vec<Section>,
    extra: HashMap<IVec3, BlockExtra>,
    /// The cells built of pieces (`pieces.rs`; their sections hold `BlockId::PIECES`).
    pieces: HashMap<IVec3, Pieces>,
}

impl Chunk {
    pub fn from_sections(pos: ChunkPos, sections: Vec<Section>) -> Self {
        assert_eq!(sections.len(), SECTIONS_PER_CHUNK);
        Self {
            pos,
            sections,
            extra: HashMap::new(),
            pieces: HashMap::new(),
        }
    }

    /// The pieces in a cell built of them.
    pub fn pieces(&self, block: IVec3) -> Option<&Pieces> {
        self.pieces.get(&block)
    }

    /// Makes a cell one built of these pieces (or, if there are none, air). Returns the block that
    /// was there before.
    pub fn set_pieces(&mut self, block: IVec3, pieces: Pieces) -> BlockId {
        if pieces.is_empty() {
            return self.set(block, BlockId::AIR);
        }
        let old = self.set(block, BlockId::PIECES);
        self.pieces.insert(block, pieces);
        old
    }

    /// Every cell built of pieces, with its pieces (in no order).
    pub fn all_pieces(&self) -> impl Iterator<Item = (&IVec3, &Pieces)> {
        self.pieces.iter()
    }

    /// Every cell built of pieces in section `level`, with its pieces.
    pub fn pieces_in(&self, level: usize) -> impl Iterator<Item = (&IVec3, &Pieces)> {
        self.pieces.iter().filter(move |(block, _)| section_level(block.z) == level)
    }

    #[allow(dead_code)] // used by saving (step 8)
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn sections_mut(&mut self) -> &mut [Section] {
        &mut self.sections
    }

    /// Block at a world position inside this chunk (Z must be 0 ..= 399).
    pub fn get(&self, block: IVec3) -> BlockId {
        self.sections[section_level(block.z)].get(index_in_section(block))
    }

    /// Replace a block. Any extra state the old block had is removed with it, and its pieces.
    /// Returns the block that was there before.
    pub fn set(&mut self, block: IVec3, id: BlockId) -> BlockId {
        let section = &mut self.sections[section_level(block.z)];
        let index = index_in_section(block);
        let old = section.get(index);
        section.set(index, id);
        self.extra.remove(&block);
        self.pieces.remove(&block);
        old
    }

    pub fn extra(&self, block: IVec3) -> Option<&BlockExtra> {
        self.extra.get(&block)
    }

    pub fn set_extra(&mut self, block: IVec3, extra: BlockExtra) {
        self.extra.insert(block, extra);
    }

    pub fn extra_count(&self) -> usize {
        self.extra.len()
    }

    /// Highest non-air block in one column, or `None` if the column is empty.
    /// `local_x` and `local_y` are 0 ..= 31 within this chunk.
    pub fn surface_at(&self, local_x: i32, local_y: i32) -> Option<(i32, BlockId)> {
        let origin = self.pos.min_block();
        for level in (0..SECTIONS_PER_CHUNK).rev() {
            let section = &self.sections[level];
            if let Section::Uniform(id) = section {
                if id.is_air() {
                    continue; // skip a whole empty section at once
                }
            }
            let bottom = level as i32 * SECTION_SIZE;
            let top = (bottom + SECTION_SIZE - 1).min(WORLD_HEIGHT - 1);
            for z in (bottom..=top).rev() {
                let block = IVec3::new(origin.x + local_x, origin.y + local_y, z);
                let id = section.get(index_in_section(block));
                if !id.is_air() {
                    return Some((z, id));
                }
            }
        }
        None
    }

    /// The column summary: surface height and block for all 1,024 columns.
    #[allow(dead_code)] // first used by distant terrain (after step 3)
    pub fn summary(&self) -> ColumnSummary {
        let mut summary = ColumnSummary {
            surface: Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize),
        };
        for local_y in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                summary.surface.push(self.surface_at(local_x, local_y));
            }
        }
        summary
    }
}

/// A lightweight stand-in for a whole chunk: just the top of each column.
#[allow(dead_code)]
pub struct ColumnSummary {
    /// Indexed `local_x + local_y * 32`. `None` = empty column.
    pub surface: Vec<Option<(i32, BlockId)>>,
}

//! Sections — how 32,768 blocks are stored compactly.
//!
//! A section is stored one of two ways:
//!
//! - **Uniform**: every block is the same (all air, or all stone). Stored as ONE number.
//!   Most of the sky is like this, so the sky is nearly free.
//!
//! - **Packed**: a *palette* lists the block types present in this section, and each block
//!   stores only its position in that list, using as few bits as the list needs:
//!     2 types → 1 bit each,  ≤4 → 2 bits,  ≤16 → 4 bits,  ≤256 → 8 bits,  more → 16 bits.
//!   A section holding granite, gabbro and two ores uses 2 bits per block instead of 16.
//!
//! The bits are packed into 64-bit "words". Bit counts are chosen so they always divide 64
//! evenly, which means a block never straddles two words (simpler and faster).

use super::block::BlockId;
use super::position::SECTION_VOLUME;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Section {
    Uniform(BlockId),
    Packed(PackedSection),
}

impl Section {
    pub fn get(&self, index: usize) -> BlockId {
        match self {
            Section::Uniform(id) => *id,
            Section::Packed(packed) => packed.get(index),
        }
    }

    pub fn set(&mut self, index: usize, id: BlockId) {
        match self {
            Section::Uniform(current) => {
                if *current == id {
                    return;
                }
                // The section is no longer uniform: switch to packed storage.
                let mut packed = PackedSection::filled_with(*current);
                packed.set(index, id);
                *self = Section::Packed(packed);
            }
            Section::Packed(packed) => packed.set(index, id),
        }
    }

    /// Build the most compact section for a full list of 32,768 blocks.
    pub fn from_blocks(blocks: &[BlockId]) -> Self {
        assert_eq!(blocks.len(), SECTION_VOLUME);

        let mut palette = Vec::new();
        let mut palette_index = HashMap::new();
        for &block in blocks {
            palette_index.entry(block).or_insert_with(|| {
                palette.push(block);
                palette.len() - 1
            });
        }

        if palette.len() == 1 {
            return Section::Uniform(palette[0]);
        }

        let bits = bits_needed(palette.len());
        let mut packed = PackedSection::with_palette(palette, bits);
        for (index, block) in blocks.iter().enumerate() {
            packed.write(index, palette_index[block]);
        }
        Section::Packed(packed)
    }

    /// Re-pack as tightly as possible. Edits only ever grow a palette (shrinking on every
    /// edit would be slow), so this tidies up afterwards: types no longer present are
    /// dropped, and a section that has become all one type turns back into Uniform.
    pub fn compact(&mut self) {
        if let Section::Packed(packed) = self {
            let blocks: Vec<BlockId> = (0..SECTION_VOLUME).map(|i| packed.get(i)).collect();
            *self = Section::from_blocks(&blocks);
        }
    }

    /// Memory used by the block data, in bytes.
    pub fn memory_bytes(&self) -> usize {
        match self {
            Section::Uniform(_) => size_of::<BlockId>(),
            Section::Packed(packed) => {
                packed.words.len() * size_of::<u64>() + packed.palette.len() * size_of::<BlockId>()
            }
        }
    }

    /// Short description for the debug panel, e.g. "uniform" or "packed: 3 types, 2 bits".
    pub fn describe(&self) -> String {
        match self {
            Section::Uniform(_) => "uniform".to_string(),
            Section::Packed(packed) => format!(
                "packed: {} types, {} bit{}",
                packed.palette.len(),
                packed.bits,
                if packed.bits == 1 { "" } else { "s" }
            ),
        }
    }

    /// Bits per block (0 for uniform sections).
    pub fn bits_per_block(&self) -> u32 {
        match self {
            Section::Uniform(_) => 0,
            Section::Packed(packed) => packed.bits,
        }
    }
}

/// Palette + bit-packed indices. See the module notes above.
#[derive(Clone)]
pub struct PackedSection {
    palette: Vec<BlockId>,
    bits: u32,
    words: Vec<u64>,
}

/// Smallest allowed bit count that can number `palette_len` different types.
fn bits_needed(palette_len: usize) -> u32 {
    match palette_len {
        0..=2 => 1,
        3..=4 => 2,
        5..=16 => 4,
        17..=256 => 8,
        _ => 16,
    }
}

impl PackedSection {
    fn with_palette(palette: Vec<BlockId>, bits: u32) -> Self {
        let per_word = 64 / bits as usize;
        Self {
            palette,
            bits,
            words: vec![0; SECTION_VOLUME / per_word],
        }
    }

    /// Every block is `id` (palette entry 0, and all the bits start at 0).
    fn filled_with(id: BlockId) -> Self {
        Self::with_palette(vec![id], 1)
    }

    /// Read the palette position stored for one block.
    fn read(&self, index: usize) -> usize {
        let bits = self.bits as usize;
        let per_word = 64 / bits;
        let word = self.words[index / per_word];
        let shift = (index % per_word) * bits;
        ((word >> shift) & ((1u64 << bits) - 1)) as usize
    }

    /// Store a palette position for one block.
    fn write(&mut self, index: usize, value: usize) {
        let bits = self.bits as usize;
        let per_word = 64 / bits;
        let shift = (index % per_word) * bits;
        let mask = ((1u64 << bits) - 1) << shift;
        let word = &mut self.words[index / per_word];
        *word = (*word & !mask) | ((value as u64) << shift);
    }

    fn get(&self, index: usize) -> BlockId {
        self.palette[self.read(index)]
    }

    fn set(&mut self, index: usize, id: BlockId) {
        let position = match self.palette.iter().position(|&p| p == id) {
            Some(position) => position,
            None => {
                self.palette.push(id);
                if self.palette.len() > (1 << self.bits) {
                    self.widen();
                }
                self.palette.len() - 1
            }
        };
        self.write(index, position);
    }

    /// The palette outgrew the bit count: re-pack every block with more bits.
    fn widen(&mut self) {
        let values: Vec<usize> = (0..SECTION_VOLUME).map(|i| self.read(i)).collect();
        let bits = bits_needed(self.palette.len());
        *self = Self::with_palette(std::mem::take(&mut self.palette), bits);
        for (index, value) in values.into_iter().enumerate() {
            self.write(index, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_becomes_packed_on_edit() {
        let mut section = Section::Uniform(BlockId(5));
        section.set(100, BlockId(7));
        assert_eq!(section.get(100), BlockId(7));
        assert_eq!(section.get(99), BlockId(5));
        assert_eq!(section.bits_per_block(), 1);
    }

    #[test]
    fn palette_widens_and_keeps_every_block() {
        let mut section = Section::Uniform(BlockId::AIR);
        // 300 different types forces 1 → 2 → 4 → 8 → 16 bits.
        for i in 0..300u16 {
            section.set(i as usize * 7, BlockId(i + 1));
        }
        for i in 0..300u16 {
            assert_eq!(section.get(i as usize * 7), BlockId(i + 1));
        }
        assert_eq!(section.get(1), BlockId::AIR);
        assert_eq!(section.bits_per_block(), 16);
    }

    #[test]
    fn compact_returns_to_uniform() {
        let mut section = Section::Uniform(BlockId(3));
        section.set(10, BlockId(4));
        section.set(10, BlockId(3));
        section.compact();
        assert!(matches!(section, Section::Uniform(BlockId(3))));
    }
}

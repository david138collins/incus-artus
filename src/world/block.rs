//! Block IDs — the compact number stored for every block in the world.
//!
//! In data files and saves, blocks are named with text IDs like "incus:granite".
//! At runtime, each name is given a small number (a `BlockId`) because numbers are
//! far cheaper to store and compare. `data::BlockRegistry` keeps the name ↔ number list.

/// A block type, as a 16-bit number (up to 65,536 types).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u16);

impl BlockId {
    /// Empty space. Always number 0.
    pub const AIR: BlockId = BlockId(0);

    pub fn is_air(self) -> bool {
        self == Self::AIR
    }
}

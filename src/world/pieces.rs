//! Pieces — a cell built of parts smaller than a block (`docs/quartile.md`, stop 1).
//!
//! Every cell is 8 **octants** (½ × ½ × ½). A **bar** is two of them in a row, 1 × ½ × ½, lying along
//! any of the 3 axes in any of 4 places: 12 bar slots. The slots go on to the 8 octants (12 to 19), so
//! a slot is one byte, 0 to 19 [David, 2026-11-01: "lets make sure we are careful it works for octant
//! items too"].
//!
//! - **Bars are the pieces** of a material. An **octant item** is an object resting in an octant ("a
//!   cup or a candle", David), not a material filling one: objects come later, on their own. The
//!   octant slots are kept for them, so an object can claim its octant (for collision, and so no bar
//!   is placed through it).
//! - **A cell with pieces** holds the marker `BlockId::PIECES` in its section (one more kind to the
//!   palette, so packing is unchanged), and its pieces in its chunk's side table, in the order placed.
//! - **The first piece keeps a shared octant:** a bar placed across one already there fills only the
//!   octants still free, which makes corners and L shapes ("that needs to be implemented seamlessly",
//!   David: no faces are drawn where bars touch), and a cell can mix materials.
//! - A piece is only placed if it fills at least one free octant.
//!
//! Octant `o` is `x + 2y + 4z`, each 0 (the south, west or lower half) or 1 (north, east, upper).

use super::block::BlockId;

/// Which part of a cell a piece fills: a bar (0 to 11: its axis × 4 + its place across), or an
/// octant (12 to 19).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot(pub u8);

impl Slot {
    pub const COUNT: u8 = 20;

    /// The bar lying along `axis` (0 north–south, 1 east–west, 2 up–down), in the halves `u` and `v`
    /// (0 or 1) of the two axes across it, in the mesher's order (`plane_axes`).
    #[allow(dead_code)] // placing by aim (quartile stop 2)
    pub fn bar(axis: u8, u: u8, v: u8) -> Self {
        Slot(axis * 4 + (u & 1) + 2 * (v & 1))
    }

    /// The octant in the halves `x`, `y`, `z` (0 or 1).
    #[allow(dead_code)] // objects resting in an octant (later)
    pub fn octant(x: u8, y: u8, z: u8) -> Self {
        Slot(12 + (x & 1) + 2 * (y & 1) + 4 * (z & 1))
    }

    pub fn is_bar(self) -> bool {
        self.0 < 12
    }

    /// The octants it fills, one bit each.
    pub fn octants(self) -> u8 {
        if self.0 >= Self::COUNT {
            return 0;
        }
        if !self.is_bar() {
            return 1 << (self.0 - 12);
        }
        let axis = (self.0 / 4) as usize;
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let mut half = [0u8; 3];
        half[u] = self.0 & 1;
        half[v] = (self.0 >> 1) & 1;
        (0..2).fold(0, |mask, along| {
            half[axis] = along;
            mask | 1 << (half[0] + 2 * half[1] + 4 * half[2])
        })
    }

    /// Its name, as the debug panel shows it.
    pub fn name(self) -> String {
        let half = |axis: usize, h: u8| [["south", "north"], ["west", "east"], ["low", "high"]][axis][h as usize];
        if self.is_bar() {
            let axis = (self.0 / 4) as usize;
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let along = ["north–south", "east–west", "upright"][axis];
            // Name the across halves in the order: north/south, east/west, high/low.
            let mut parts = [(u, self.0 & 1), (v, (self.0 >> 1) & 1)];
            parts.sort_by_key(|&(axis, _)| axis);
            format!("bar {along}, {} {}", half(parts[0].0, parts[0].1), half(parts[1].0, parts[1].1))
        } else {
            let o = self.0 - 12;
            format!("octant {} {} {}", half(0, o & 1), half(1, (o >> 1) & 1), half(2, (o >> 2) & 1))
        }
    }
}

/// The piece filling each of a cell's 8 octants: its slot and its block.
pub type Owners = [Option<(Slot, BlockId)>; 8];

/// A cell's pieces, in the order placed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pieces {
    list: Vec<(Slot, BlockId)>,
}

impl Pieces {
    /// Adds a bar of `block` in `slot`, if it fills at least one free octant (true if it did). (The
    /// octant slots are for objects, which come later.)
    pub fn add(&mut self, slot: Slot, block: BlockId) -> bool {
        let taken = self.list.iter().fold(0, |mask, (slot, _)| mask | slot.octants());
        if !slot.is_bar() || slot.octants() & !taken == 0 || block.is_air() || block == BlockId::PIECES {
            return false;
        }
        self.list.push((slot, block));
        true
    }

    /// The pieces, in the order placed.
    #[allow(dead_code)] // saving pieces (quartile stop 3)
    pub fn list(&self) -> &[(Slot, BlockId)] {
        &self.list
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// What fills each octant (`AIR`: nothing), the first piece keeping a shared one.
    pub fn octants(&self) -> [BlockId; 8] {
        self.owners().map(|owner| owner.map_or(BlockId::AIR, |(_, block)| block))
    }

    /// The piece filling each octant, its slot and block (the first piece keeping a shared one): a
    /// bar's own textures are measured from its own corner, even where another took part of it.
    pub fn owners(&self) -> Owners {
        let mut owners = [None; 8];
        for &(slot, block) in &self.list {
            let mask = slot.octants();
            for (o, owner) in owners.iter_mut().enumerate() {
                if mask >> o & 1 == 1 && owner.is_none() {
                    *owner = Some((slot, block));
                }
            }
        }
        owners
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_slots_fill_the_octants_they_name() {
        // Each bar fills two octants in a row along its axis, each octant slot one.
        let mut covered = [0u32; 8];
        for s in 0..Slot::COUNT {
            let mask = Slot(s).octants();
            assert_eq!(mask.count_ones(), if s < 12 { 2 } else { 1 }, "{}", Slot(s).name());
            for o in 0..8 {
                covered[o] += (mask >> o & 1) as u32;
            }
        }
        // Every octant lies in three bars (one along each axis) and is one octant slot.
        assert_eq!(covered, [4; 8]);
        // The upright bar in the north-east: the two octants north, east, low and high.
        let (north, east) = (1, 2);
        assert_eq!(Slot::bar(2, 1, 1).octants(), 1 << (north + east) | 1 << (north + east + 4));
        assert_eq!(Slot::bar(2, 1, 1).name(), "bar upright, north east");
        assert_eq!(Slot::octant(0, 1, 1).name(), "octant south east high");
        assert_eq!(Slot(20).octants(), 0);
    }

    #[test]
    fn the_first_piece_keeps_a_shared_octant_and_a_piece_needs_a_free_one() {
        let (oak, stone) = (BlockId(5), BlockId(6));
        let mut pieces = Pieces::default();
        // A low bar north–south on the west, then a low bar east–west on the south: they share
        // the south-west low octant, which stays oak.
        assert!(pieces.add(Slot::bar(0, 0, 0), oak));
        assert!(pieces.add(Slot::bar(1, 0, 0), stone));
        let octants = pieces.octants();
        assert_eq!(octants[0], oak, "the shared octant");
        assert_eq!(octants[1], oak);
        assert_eq!(octants[2], stone);
        assert_eq!(octants.iter().filter(|block| !block.is_air()).count(), 3);
        // A bar whose octants are both taken: not placed. An upright one in the north-west: one free.
        assert!(!pieces.add(Slot::bar(0, 0, 0), stone));
        assert!(pieces.add(Slot::bar(2, 1, 0), stone), "the north-west upright: its high octant is free");
        assert_eq!(pieces.list().len(), 3);
        // An octant slot isn't a material's piece (it's for objects); air and the marker never are.
        assert!(!pieces.add(Slot::octant(1, 1, 1), stone));
        assert!(!pieces.add(Slot::bar(2, 1, 1), BlockId::AIR));
    }
}

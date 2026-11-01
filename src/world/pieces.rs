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
use bevy::math::{IVec3, Vec3};

/// Which part of a cell a piece fills, in one byte: the low 5 bits its place (a bar, 0 to 11: its axis
/// × 4 + its place across; or an octant, 12 to 19), the top 3 its **link**. An octant of material is
/// always half of a bar across two cells (`docs/quartile.md` section 10, "linked half-bars"), and its
/// link says which way its other half lies: 1 north, 2 south, 3 east, 4 west, 5 up, 6 down (0 none).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot(pub u8);

impl Slot {
    pub const COUNT: u8 = 20;

    /// The bar lying along `axis` (0 north–south, 1 east–west, 2 up–down), in the halves `u` and `v`
    /// (0 or 1) of the two axes across it, in the mesher's order (`plane_axes`).
    pub fn bar(axis: u8, u: u8, v: u8) -> Self {
        Slot(axis * 4 + (u & 1) + 2 * (v & 1))
    }

    /// The octant in the halves `x`, `y`, `z` (0 or 1).
    pub fn octant(x: u8, y: u8, z: u8) -> Self {
        Slot(12 + (x & 1) + 2 * (y & 1) + 4 * (z & 1))
    }

    /// Half of a bar across two cells: the octant in the halves `x`, `y`, `z`, its other half the way
    /// `toward` (one step along an axis) in the next cell.
    pub fn half(x: u8, y: u8, z: u8, toward: IVec3) -> Self {
        let axis = toward.abs().max_position() as u8;
        let link = 1 + axis * 2 + u8::from(toward[axis as usize] < 0);
        Slot(Self::octant(x, y, z).0 | link << 5)
    }

    /// Its place, 0 to 19 (without its link).
    pub fn index(self) -> u8 {
        self.0 & 31
    }

    /// Which way its other half lies, if it's half of a bar across two cells.
    pub fn link(self) -> Option<IVec3> {
        let link = self.0 >> 5;
        (1..=6).contains(&link).then(|| {
            let mut toward = IVec3::ZERO;
            toward[((link - 1) / 2) as usize] = if link % 2 == 1 { 1 } else { -1 };
            toward
        })
    }

    pub fn is_bar(self) -> bool {
        self.index() < 12
    }

    /// The axis its bar lies along: a bar's own, or for half a bar, the way to its other half.
    pub fn bar_axis(self) -> Option<usize> {
        if self.is_bar() { Some((self.index() / 4) as usize) } else { self.link().map(|toward| toward.abs().max_position()) }
    }

    /// The octants it fills, one bit each.
    pub fn octants(self) -> u8 {
        let index = self.index();
        if index >= Self::COUNT {
            return 0;
        }
        if !self.is_bar() {
            return 1 << (index - 12);
        }
        let axis = (index / 4) as usize;
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let mut half = [0u8; 3];
        half[u] = index & 1;
        half[v] = (index >> 1) & 1;
        (0..2).fold(0, |mask, along| {
            half[axis] = along;
            mask | 1 << (half[0] + 2 * half[1] + 4 * half[2])
        })
    }

    /// Its name, as the debug panel and the log show it.
    #[allow(dead_code)] // for the log and the panel; used by the tests so far
    pub fn name(self) -> String {
        let half = |axis: usize, h: u8| [["south", "north"], ["west", "east"], ["low", "high"]][axis][h as usize];
        let index = self.index();
        if self.is_bar() {
            let axis = (index / 4) as usize;
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let along = ["north–south", "east–west", "upright"][axis];
            // Name the across halves in the order: north/south, east/west, high/low.
            let mut parts = [(u, index & 1), (v, (index >> 1) & 1)];
            parts.sort_by_key(|&(axis, _)| axis);
            format!("bar {along}, {} {}", half(parts[0].0, parts[0].1), half(parts[1].0, parts[1].1))
        } else {
            let o = index - 12;
            let at = format!("{} {} {}", half(0, o & 1), half(1, (o >> 1) & 1), half(2, (o >> 2) & 1));
            match self.0 >> 5 {
                link @ 1..=6 => format!("half a bar, {at}, going on {}", ["north", "south", "east", "west", "up", "down"][link as usize - 1]),
                _ => format!("octant {at}"),
            }
        }
    }
}

/// Where a bar would go, aimed at a face (David's control, 2026-11-01: "the quarter blocks either snap
/// to the face you're aiming at sideways, or longways, and you toggle with a button"): lying flat
/// against the face, in the half-block layer in front of it, in the half of the face the crosshair is
/// in, and along its length centred on the crosshair, snapped to half blocks (so it can run across two
/// cells). Its axis: on a floor or a ceiling, longways points the way you're `looking` and sideways
/// across it; on a wall, sideways lies flat and longways stands upright.
///
/// `half` is the half-block hit, `normal` the face's way, `point` where the ray met it. Returns the
/// bar's corner (in half blocks, its lowest) and its axis, for `EditKind::PlaceBar`.
pub fn bar_placement(half: IVec3, normal: IVec3, point: Vec3, looking: Vec3, longways: bool) -> Option<(IVec3, u8)> {
    if normal == IVec3::ZERO {
        return None;
    }
    let facing = normal.abs().max_position();
    let axis = if facing == 2 {
        let ahead = if looking.x.abs() >= looking.y.abs() { 0 } else { 1 };
        if longways { ahead } else { 1 - ahead }
    } else {
        let flat = if facing == 0 { 1 } else { 0 };
        if longways { 2 } else { flat }
    };
    let across = 3 - facing - axis;
    let mut at = IVec3::ZERO;
    at[facing] = half[facing] + normal[facing];
    at[across] = (point[across] * 2.0).floor() as i32;
    at[axis] = (point[axis] * 2.0 - 1.0).round() as i32;
    Some((at, axis as u8))
}

/// A bar's box (its two corners, blocks), from its corner in half blocks and its axis.
pub fn bar_box(at: IVec3, axis: u8) -> (Vec3, Vec3) {
    let low = at.as_vec3() / 2.0;
    let mut size = Vec3::splat(0.5);
    size[axis as usize] = 1.0;
    (low, low + size)
}

/// The piece filling each of a cell's 8 octants: its slot and its block.
pub type Owners = [Option<(Slot, BlockId)>; 8];

/// A cell's pieces, in the order placed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pieces {
    list: Vec<(Slot, BlockId)>,
}

impl Pieces {
    /// Adds a piece of `block` in `slot`, if it fills at least one free octant (true if it did): a
    /// bar, or half of one across two cells (an octant with a link). (Octants without a link are for
    /// objects, which come later.)
    pub fn add(&mut self, slot: Slot, block: BlockId) -> bool {
        if !self.has_room(slot) || block.is_air() || block == BlockId::PIECES {
            return false;
        }
        self.list.push((slot, block));
        true
    }

    /// Whether a piece could go in `slot`: a bar or a linked half, with a free octant.
    pub fn has_room(&self, slot: Slot) -> bool {
        let taken = self.list.iter().fold(0, |mask, (slot, _)| mask | slot.octants());
        (slot.is_bar() || slot.link().is_some()) && slot.octants() & !taken != 0
    }

    /// Takes out the half of a bar in this octant (`octant`, 0 to 7) whose other half lies `toward`;
    /// true if there was one.
    pub fn remove_half(&mut self, octant: u8, toward: IVec3) -> bool {
        let before = self.list.len();
        self.list.retain(|(slot, _)| !(slot.index() == 12 + octant && slot.link() == Some(toward)));
        self.list.len() != before
    }

    /// The pieces, in the order placed.
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
    fn a_bar_snaps_flat_to_the_face_aimed_at_longways_or_sideways() {
        // The top of the block at (0, 0, 10), aimed at in its north-east quarter, looking north.
        let half = IVec3::new(1, 1, 21);
        let point = Vec3::new(0.7, 0.8, 11.0);
        let north = Vec3::new(1.0, 0.2, -0.5);
        // Longways: north–south, on top (Z half 22), in the east half, centred on the crosshair
        // (0.7 → from 0.2, snapped to 0): one cell.
        assert_eq!(bar_placement(half, IVec3::Z, point, north, true), Some((IVec3::new(0, 1, 22), 0)));
        // Sideways: east–west, in the north half, from 0.3 east snapped to 0.5: across into the next cell.
        assert_eq!(bar_placement(half, IVec3::Z, point, north, false), Some((IVec3::new(1, 1, 22), 1)));
        assert_eq!(bar_box(IVec3::new(1, 1, 22), 1), (Vec3::new(0.5, 0.5, 11.0), Vec3::new(1.0, 1.5, 11.5)));
        // A wall facing north (its north face, X = 1): sideways lies flat (east–west), longways stands.
        let wall = IVec3::new(1, 0, 20);
        let on_wall = Vec3::new(1.0, 0.3, 10.8);
        assert_eq!(bar_placement(wall, IVec3::X, on_wall, -Vec3::X, false).map(|(at, axis)| (at.x, axis)), Some((2, 1)));
        assert_eq!(bar_placement(wall, IVec3::X, on_wall, -Vec3::X, true), Some((IVec3::new(2, 0, 21), 2)));
        // Nothing when the ray started inside a block.
        assert_eq!(bar_placement(half, IVec3::ZERO, point, north, true), None);
    }

    #[test]
    fn half_a_bar_knows_which_way_its_other_half_lies() {
        for toward in [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z] {
            let slot = Slot::half(1, 0, 1, toward);
            assert_eq!((slot.link(), slot.index(), slot.octants()), (Some(toward), Slot::octant(1, 0, 1).index(), 1 << 5));
            assert!(!slot.is_bar());
        }
        assert_eq!(Slot::half(0, 1, 0, IVec3::Z).name(), "half a bar, south east low, going on up");
        assert_eq!((Slot::bar(2, 1, 1).link(), Slot::octant(1, 1, 1).link()), (None, None));
        // Taken out by its octant and link: only that one.
        let mut pieces = Pieces::default();
        pieces.add(Slot::half(1, 0, 0, IVec3::X), BlockId(5));
        pieces.add(Slot::bar(2, 0, 1), BlockId(5));
        assert!(!pieces.remove_half(1, IVec3::NEG_X));
        assert!(pieces.remove_half(1, IVec3::X));
        assert_eq!(pieces.list().len(), 1);
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
        // An octant slot isn't a material's piece (it's for objects) unless it's half of a bar across
        // two cells; air and the marker never are.
        assert!(!pieces.add(Slot::octant(1, 1, 1), stone));
        assert!(pieces.add(Slot::half(1, 1, 1, IVec3::Z), stone));
        assert!(!pieces.add(Slot::half(1, 1, 1, IVec3::NEG_X), stone), "its octant is taken");
        assert!(!pieces.add(Slot::bar(2, 1, 1), BlockId::AIR));
    }
}

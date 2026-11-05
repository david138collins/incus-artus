//! Pieces — a cell built of parts smaller than a block (`docs/quartile.md`).
//!
//! Every cell is 8 **octants** (½ × ½ × ½), and the octant is the only unit [David, 2026-11-01: "any oct
//! can just be a stickon"; section 12]. Each octant of a cell of pieces holds its material and a
//! **part** byte saying what it's part of:
//! - **a bar** (1 × ½ × ½, a quarter block) is two octants in a row, each knowing its bar's axis, which
//!   end it is, and that its other half is whole. Its other half is in the same cell, or across the
//!   cell's side (a bar placed across two cells): the same either way.
//! - **a cut piece** is an octant whose other half was already taken when it was placed (the first
//!   piece keeps an octant). It's "stuck to" whatever holds that octant, and goes with it when that's
//!   mined. A cut piece can't be got back (David: "if you have 'cut' a block, its no longer
//!   retrievable").
//! - **a lone octant** belongs to no bar (an "octblock"), mined on its own.
//!
//! Single bars are aimed at and mined one at a time, like slabs (David: "you can mine individual
//! slabs in minecraft theres no reason you shouldnt be able to target and mine quarter blocks"):
//! `VoxelWorld::piece_halves` finds a piece from any octant of it. Objects resting in an octant (a cup,
//! a candle) are things of their own, for later.
//!
//! - **A cell of pieces** holds the marker `BlockId::PIECES` in its section (one more kind to the
//!   palette, so packing is unchanged), and its octants in its chunk's side table.
//! - Octant `o` is `x + 2y + 4z`, each 0 (the south, west or lower half) or 1 (north, east, upper);
//!   a place counted in half blocks (`half`) is in the cell `half / 2`, its octant `half % 2`.

use super::block::BlockId;
use bevy::math::{IVec3, Vec3};

/// What an octant of material is part of, in one byte: its bar's axis (2 bits: 0 none, a lone
/// octant; 1 north–south, 2 east–west, 3 upright), which end of the bar it is (1 bit: 0 the low end,
/// its other half the positive way), and whether its other half is there (1 bit; not: it was cut).
/// Or, with bit 4 set, **a corner of a whole block on the octant grid** (David, 2026-11-01: "we
/// should be able to place whole blocks on the oct grid, as well"): a block half a block off the
/// cells along one, two or three axes, its 8 octants across 2, 4 or 8 cells, each knowing which of
/// the block's corners it is (3 bits, `x + 2y + 4z`), so it's mined whole from any of them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Part(pub u8);

impl Part {
    /// A lone octant, part of no bar.
    pub const LONE: Part = Part(0);

    /// Corner `corner` (`x + 2y + 4z`, each 0 low or 1 high) of a whole block on the octant grid.
    pub fn of_block(corner: usize) -> Self {
        Part(16 | (corner as u8 & 7) << 5)
    }

    /// Which corner of a whole block on the octant grid this is, if it's one.
    pub fn corner(self) -> Option<usize> {
        (self.0 & 16 != 0).then_some((self.0 >> 5) as usize)
    }

    /// Half of a bar along `axis`: its `end` (0 low, 1 high), and whether the other half is `whole`.
    pub fn of_bar(axis: usize, end: u8, whole: bool) -> Self {
        Part((axis as u8 % 3 + 1) | (end & 1) << 2 | u8::from(whole) << 3)
    }

    /// The axis its bar lies along (none for a lone octant).
    pub fn axis(self) -> Option<usize> {
        match self.0 & 3 {
            0 => None,
            axis => Some(axis as usize - 1),
        }
    }

    pub fn end(self) -> u8 {
        (self.0 >> 2) & 1
    }

    /// Its bar's other half is there: a bar, not a cut piece.
    pub fn whole(self) -> bool {
        self.axis().is_some() && (self.0 >> 3) & 1 == 1
    }

    /// Cut when placed: half of a bar whose other half was taken. It's stuck to what holds that octant.
    pub fn is_cut(self) -> bool {
        self.axis().is_some() && !self.whole()
    }

    /// Where its bar's other half is (or would have been), one octant along, in half blocks.
    pub fn other_half(self) -> Option<IVec3> {
        let axis = self.axis()?;
        let mut toward = IVec3::ZERO;
        toward[axis] = if self.end() == 0 { 1 } else { -1 };
        Some(toward)
    }

    /// Its name, as the log shows it.
    #[allow(dead_code)] // for the log and the panel; used by the tests so far
    pub fn name(self) -> String {
        if let Some(corner) = self.corner() {
            return format!("corner {corner} of a whole block on the octant grid");
        }
        match self.axis() {
            None => "an octant".into(),
            Some(axis) => {
                let along = ["north–south", "east–west", "upright"][axis];
                let end = ["its low end", "its high end"][self.end() as usize];
                if self.whole() { format!("half of a {along} bar, {end}") } else { format!("a cut {along} bar, {end}") }
            }
        }
    }
}

/// An octant's place in its cell, from a place counted in half blocks.
pub fn octant_of(half: IVec3) -> usize {
    let o = half.rem_euclid(IVec3::splat(2));
    (o.x + 2 * o.y + 4 * o.z) as usize
}

/// A cell's octants of material.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pieces {
    octants: [Option<(BlockId, Part)>; 8],
}

impl Pieces {
    /// What fills octant `o` (0 to 7), and what it's part of.
    pub fn get(&self, o: usize) -> Option<(BlockId, Part)> {
        self.octants.get(o).copied().flatten()
    }

    /// Fills octant `o` with `block`, part of `part`, if it's free (true if it was).
    pub fn set(&mut self, o: usize, block: BlockId, part: Part) -> bool {
        if block.is_air() || block == BlockId::PIECES || self.get(o).is_some() || o >= 8 {
            return false;
        }
        self.octants[o] = Some((block, part));
        true
    }

    /// Empties octant `o`.
    pub fn clear(&mut self, o: usize) {
        if let Some(octant) = self.octants.get_mut(o) {
            *octant = None;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.octants.iter().all(Option::is_none)
    }
}

/// Which way a bar runs from where it starts, as B turns it [David, 2026-11-01: "B 3 ways. sideways,
/// longways, upways"].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Turn {
    /// To your right (your left is its "bottom" end).
    Sideways,
    /// Away from you, the way you're looking.
    #[default]
    Longways,
    /// Up.
    Upways,
}

impl Turn {
    /// The next, as B turns it.
    pub fn next(self) -> Self {
        match self {
            Turn::Sideways => Turn::Longways,
            Turn::Longways => Turn::Upways,
            Turn::Upways => Turn::Sideways,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Turn::Sideways => "sideways",
            Turn::Longways => "longways",
            Turn::Upways => "upways",
        }
    }
}

/// Where a piece starts, aimed at a face: the octant in front of the face at the crosshair (so the
/// crosshair always knows which octant it starts from). `half` is the half-block hit, `normal` the
/// face's way.
pub fn piece_start(half: IVec3, normal: IVec3) -> Option<IVec3> {
    (normal != IVec3::ZERO).then_some(half + normal)
}

/// Where a bar would go, aimed at a face (David's control, 2026-11-01): from the octant in front of
/// the face at the crosshair, one octant on along the line B has turned it, as Minecraft places by the
/// way you face. Upways is upright, longways the way you're `looking`, sideways across that. Which way
/// along the line is the half of the octant the crosshair is on, at the `point` it met the face (David:
/// "can we have the octfaces split so the left or right side flips the quarterblocks orientation?"):
/// its right half runs right, its left half left, so you needn't turn round. A line straight through
/// the face goes out of it (longways on a wall sticks straight out, upways under a ceiling hangs
/// down). It can run across two cells. Returns the bar's corner (in half blocks, its lowest) and its
/// axis, for `EditKind::PlaceBar`.
pub fn bar_placement(half: IVec3, normal: IVec3, point: Vec3, looking: Vec3, turn: Turn) -> Option<(IVec3, u8)> {
    let start = piece_start(half, normal)?;
    // The way you face, on the ground, and across it.
    let ahead = if looking.x.abs() >= looking.y.abs() { IVec3::X } else { IVec3::Y };
    let axis = match turn {
        Turn::Sideways => IVec3::new(ahead.y, ahead.x, 0),
        Turn::Longways => ahead,
        Turn::Upways => IVec3::Z,
    }
    .max_position();
    let step = if normal[axis] != 0 {
        normal
    } else {
        let mut step = IVec3::ZERO;
        step[axis] = toward(half, point, axis);
        step
    };
    Some((start.min(start + step), axis as u8))
}

/// Which way along `axis` (lying in the face) the crosshair's half of the octant `half` is: where in
/// the octant's face the `point` is, 0 to 1 along it, past the middle the positive way.
fn toward(half: IVec3, point: Vec3, axis: usize) -> i32 {
    if point[axis] * 2.0 - half[axis] as f32 >= 0.5 { 1 } else { -1 }
}

/// Where a whole block on the octant grid would go, aimed at a face: its lowest corner (in half
/// blocks), from the octant in front of the face at the crosshair, one octant on along every axis:
/// straight out of the face, and across it toward the half of the octant the crosshair is on, as a
/// bar goes (`bar_placement`). Its 8 octants are `block_halves`. Where it lines up with the cells
/// (every corner even), it's an ordinary block in the cell in front of the face.
pub fn block_placement(half: IVec3, normal: IVec3, point: Vec3) -> Option<IVec3> {
    let start = piece_start(half, normal)?;
    let mut corner = start;
    for axis in 0..3 {
        let step = if normal[axis] != 0 { normal[axis] } else { toward(half, point, axis) };
        corner[axis] = start[axis].min(start[axis] + step);
    }
    Some(corner)
}

/// A whole block's 8 octants (in half blocks) from its lowest corner, in corner order (`x + 2y + 4z`).
pub fn block_halves(at: IVec3) -> [IVec3; 8] {
    std::array::from_fn(|c| at + IVec3::new(c as i32 & 1, (c as i32 >> 1) & 1, (c as i32 >> 2) & 1))
}

/// The two octants a bar fills (in half blocks), from its corner and its axis.
pub fn bar_halves(at: IVec3, axis: u8) -> [IVec3; 2] {
    let mut step = IVec3::ZERO;
    step[axis as usize % 3] = 1;
    [at, at + step]
}

/// An octant's box (its two corners, blocks), from its place in half blocks.
pub fn half_box(half: IVec3) -> (Vec3, Vec3) {
    let low = half.as_vec3() / 2.0;
    (low, low + Vec3::splat(0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_knows_its_bar_its_end_and_whether_it_was_cut() {
        for axis in 0..3 {
            for end in 0..2 {
                for whole in [true, false] {
                    let part = Part::of_bar(axis, end, whole);
                    assert_eq!((part.axis(), part.end(), part.whole(), part.is_cut()), (Some(axis), end, whole, !whole));
                    let mut toward = IVec3::ZERO;
                    toward[axis] = if end == 0 { 1 } else { -1 };
                    assert_eq!(part.other_half(), Some(toward));
                }
            }
        }
        assert_eq!((Part::LONE.axis(), Part::LONE.whole(), Part::LONE.is_cut(), Part::LONE.other_half()), (None, false, false, None));
        assert_eq!(Part::of_bar(2, 1, false).name(), "a cut upright bar, its high end");
    }

    #[test]
    fn the_first_piece_keeps_an_octant() {
        let (oak, stone) = (BlockId(5), BlockId(6));
        let mut pieces = Pieces::default();
        assert!(pieces.set(3, oak, Part::of_bar(0, 0, true)));
        assert!(!pieces.set(3, stone, Part::LONE), "taken");
        assert!(!pieces.set(4, BlockId::AIR, Part::LONE) && !pieces.set(4, BlockId::PIECES, Part::LONE));
        assert_eq!(pieces.get(3), Some((oak, Part::of_bar(0, 0, true))));
        pieces.clear(3);
        assert!(pieces.is_empty());
        assert_eq!(octant_of(IVec3::new(-1, 2, 3)), 1 + 4);
    }

    #[test]
    fn a_bar_starts_at_the_crosshair_and_runs_the_way_b_turned_it() {
        // The top of the block at (0, 0, 10), aimed at in its north-east-high octant (half (1, 1, 21)),
        // looking north: it starts in the octant above, (1, 1, 22).
        let half = IVec3::new(1, 1, 21);
        let north = Vec3::new(1.0, 0.2, -0.5);
        assert_eq!(piece_start(half, IVec3::Z), Some(IVec3::new(1, 1, 22)));
        // Aimed at the octant's north-east quarter: longways runs north, on into the next cell;
        // sideways east, on into the next; upways up.
        let north_east = Vec3::new(0.8, 0.8, 11.0);
        assert_eq!(bar_placement(half, IVec3::Z, north_east, north, Turn::Longways), Some((IVec3::new(1, 1, 22), 0)));
        assert_eq!(bar_placement(half, IVec3::Z, north_east, north, Turn::Sideways), Some((IVec3::new(1, 1, 22), 1)));
        assert_eq!(bar_placement(half, IVec3::Z, north_east, north, Turn::Upways), Some((IVec3::new(1, 1, 22), 2)));
        // The other half of the octant flips it, whichever way you face: on its west half, sideways
        // runs west, so the corner is one octant west; on its south half, longways runs south.
        let west = Vec3::new(0.8, 0.6, 11.0);
        assert_eq!(bar_placement(half, IVec3::Z, west, north, Turn::Sideways), Some((IVec3::new(1, 0, 22), 1)));
        assert_eq!(bar_placement(half, IVec3::Z, west, -north, Turn::Sideways), Some((IVec3::new(1, 0, 22), 1)));
        assert_eq!(bar_placement(half, IVec3::Z, north_east, -north, Turn::Sideways), Some((IVec3::new(1, 1, 22), 1)));
        assert_eq!(bar_placement(half, IVec3::Z, Vec3::new(0.6, 0.8, 11.0), north, Turn::Longways), Some((IVec3::new(0, 1, 22), 0)));
        // A wall facing north (its north face), looking south at it: longways would go into it, so it
        // sticks out north; upways stands on its top half and hangs on its bottom half; sideways lies
        // along it, west on its west half.
        let wall = IVec3::new(1, 0, 20);
        let south = Vec3::new(-1.0, 0.1, 0.0);
        let (top_west, bottom_east) = (Vec3::new(1.0, 0.1, 10.4), Vec3::new(1.0, 0.4, 10.1));
        assert_eq!(bar_placement(wall, IVec3::X, top_west, south, Turn::Longways), Some((IVec3::new(2, 0, 20), 0)));
        assert_eq!(bar_placement(wall, IVec3::X, top_west, south, Turn::Upways), Some((IVec3::new(2, 0, 20), 2)));
        assert_eq!(bar_placement(wall, IVec3::X, bottom_east, south, Turn::Upways), Some((IVec3::new(2, 0, 19), 2)));
        assert_eq!(bar_placement(wall, IVec3::X, top_west, south, Turn::Sideways), Some((IVec3::new(2, -1, 20), 1)));
        assert_eq!(bar_placement(wall, IVec3::X, bottom_east, south, Turn::Sideways), Some((IVec3::new(2, 0, 20), 1)));
        // A ceiling: upways would go into it, so it hangs down.
        assert_eq!(bar_placement(IVec3::new(0, 0, 30), IVec3::NEG_Z, Vec3::new(0.2, 0.2, 15.0), north, Turn::Upways), Some((IVec3::new(0, 0, 28), 2)));
        // B goes round all three; nothing when the ray started inside a block.
        assert_eq!(Turn::Sideways.next().next().next(), Turn::Sideways);
        assert_eq!(bar_placement(half, IVec3::ZERO, north_east, north, Turn::Upways), None);
        assert_eq!(bar_halves(IVec3::new(1, 1, 22), 1), [IVec3::new(1, 1, 22), IVec3::new(1, 2, 22)]);
        assert_eq!(half_box(IVec3::new(1, 1, 22)), (Vec3::new(0.5, 0.5, 11.0), Vec3::new(1.0, 1.0, 11.5)));
    }

    #[test]
    fn a_whole_block_on_the_octant_grid_goes_toward_the_crosshair_s_half_and_knows_its_corners() {
        // The top of the block at (0, 0, 10), aimed at in its north-east-high octant: from the octant
        // above, north and east of it on the north-east quarter (half a block off the cells both
        // ways), and lined up with the cells on the south-west quarter (an ordinary block above).
        let half = IVec3::new(1, 1, 21);
        assert_eq!(block_placement(half, IVec3::Z, Vec3::new(0.8, 0.8, 11.0)), Some(IVec3::new(1, 1, 22)));
        assert_eq!(block_placement(half, IVec3::Z, Vec3::new(0.6, 0.6, 11.0)), Some(IVec3::new(0, 0, 22)));
        // A wall's north face: straight out north, and along it by the crosshair's half.
        assert_eq!(block_placement(IVec3::new(1, 0, 20), IVec3::X, Vec3::new(1.0, 0.1, 10.4)), Some(IVec3::new(2, -1, 20)));
        assert_eq!(block_placement(half, IVec3::ZERO, Vec3::ZERO), None);
        let halves = block_halves(IVec3::new(1, 1, 22));
        assert_eq!((halves[0], halves[7]), (IVec3::new(1, 1, 22), IVec3::new(2, 2, 23)));
        // Each of its octants knows which corner it is, and is no bar.
        for corner in 0..8 {
            let part = Part::of_block(corner);
            assert_eq!((part.corner(), part.axis(), part.whole(), part.is_cut(), part.other_half()), (Some(corner), None, false, false, None));
        }
        assert_eq!((Part::LONE.corner(), Part::of_bar(1, 1, true).corner()), (None, None));
    }
}

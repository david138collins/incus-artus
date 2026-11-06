//! A cell of the world as sight sees it (the regroup, phase 3.1a): a cube of blocks of any size, 1 block
//! to a whole mountain, and what's in it.
//!
//! **Kept as sums, in whole numbers** (`Sums`): how many of its blocks are solid and how many water, the solid
//! faces showing (a face against air or water, counted once, in the cell of the block it belongs to, by the
//! way it looks), their colours added, and their brightness and height added and squared for the spreads. A
//! cell's sums are exactly its children's added, so every coarser level is exactly the average of the one
//! below by construction, not by care.
//!
//! **What the march reads is the compact form** (`Cell`, 16 bytes), worked out from the sums by one rule
//! (`Sums::compact`): the shares, the average colour and facing, and the spreads. The sums are only held
//! while building; the compact form is what's kept.

use crate::world::BlockId;

/// The ways a face looks, in the order `Sums::faces` keeps them: up, down, north, south, east, west (game
/// axes: x north, y east, z up).
pub const DIRECTIONS: [[i32; 3]; 6] = [[0, 0, 1], [0, 0, -1], [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0]];
pub const UP: usize = 0;

/// Colours and brightness are in 1,024ths of linear light.
pub const FULL: u64 = 1023;

/// What a block is, to sight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Solid,
    Water,
    Air,
}

/// Every block's far colours in whole numbers (`look::Palette`'s, top and sides, in 1,024ths of linear
/// light), and which block is water.
#[derive(Clone, Debug, Default)]
pub struct FarColours {
    /// By block number: its top's colour and its sides' (the bottom shows the sides').
    pub blocks: Vec<([u64; 3], [u64; 3])>,
    pub water: BlockId,
}

impl FarColours {
    pub fn new(palette: &crate::look::Palette, water: BlockId) -> Self {
        let whole = |c: [f32; 3]| c.map(|v| (v.clamp(0.0, 1.0) * FULL as f32).round() as u64);
        Self { blocks: palette.blocks.iter().map(|&(top, side)| (whole(top), whole(side))).collect(), water }
    }

    pub fn kind(&self, block: BlockId) -> Kind {
        if block == BlockId::AIR {
            Kind::Air
        } else if block == self.water {
            Kind::Water
        } else {
            Kind::Solid
        }
    }

    /// A face's colour: its block's top's looking up, its sides' every other way.
    pub fn colour(&self, block: BlockId, direction: usize) -> [u64; 3] {
        self.blocks.get(block.0 as usize).map_or([FULL / 2; 3], |&(top, side)| if direction == UP { top } else { side })
    }
}

/// A colour's brightness (Rec. 709's weights, in whole numbers, rounded).
pub fn light(colour: [u64; 3]) -> u64 {
    (colour[0] * 2126 + colour[1] * 7152 + colour[2] * 722 + 5000) / 10_000
}

/// A cell's contents as exact sums.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sums {
    pub solid: u64,
    pub water: u64,
    /// Solid faces showing, by the way they look (`DIRECTIONS`).
    pub faces: [u64; 6],
    /// Their colours, added.
    pub colour: [u64; 3],
    /// Their brightness, added and squared.
    pub light: u64,
    pub light_squared: u64,
    /// Their heights (the block's Z), added and squared.
    pub height: u64,
    pub height_squared: u64,
}

impl Sums {
    /// Another cell's sums added in.
    pub fn add(&mut self, other: &Sums) {
        self.solid += other.solid;
        self.water += other.water;
        for d in 0..6 {
            self.faces[d] += other.faces[d];
        }
        for c in 0..3 {
            self.colour[c] += other.colour[c];
        }
        self.light += other.light;
        self.light_squared += other.light_squared;
        self.height += other.height;
        self.height_squared += other.height_squared;
    }

    /// How many solid faces show.
    pub fn showing(&self) -> u64 {
        self.faces.iter().sum()
    }

    /// One block at `at`, given what's at the six blocks beside it (`beside`, in `DIRECTIONS`' order).
    pub fn add_block(&mut self, at: [i32; 3], block: BlockId, beside: [BlockId; 6], colours: &FarColours) {
        match colours.kind(block) {
            Kind::Air => {}
            Kind::Water => self.water += 1,
            Kind::Solid => {
                self.solid += 1;
                for (d, &next) in beside.iter().enumerate() {
                    if colours.kind(next) != Kind::Solid {
                        let colour = colours.colour(block, d);
                        let (light, z) = (light(colour), at[2].max(0) as u64);
                        self.faces[d] += 1;
                        for c in 0..3 {
                            self.colour[c] += colour[c];
                        }
                        self.light += light;
                        self.light_squared += light * light;
                        self.height += z;
                        self.height_squared += z * z;
                    }
                }
            }
        }
    }

    /// The compact form the march reads, for a cell `size` blocks a side (`Cell`).
    pub fn compact(&self, size: u32) -> Cell {
        let volume = (size as u64).pow(3);
        let showing = self.showing();
        let mut cell = Cell { solid: share(self.solid, volume, 255) as u8, water: share(self.water, volume, 255) as u8, ..Cell::default() };
        if showing == 0 {
            return cell;
        }
        let colour = self.colour.map(|c| share(c, showing, 1));
        cell.colour = (colour[0] << 20 | colour[1] << 10 | colour[2]) as u32;
        // The average facing: each face's way, added, over how many (127 a whole way).
        let way = |plus: usize, minus: usize| self.faces[plus] as i64 - self.faces[minus] as i64;
        let facing = [way(2, 3), way(4, 5), way(0, 1)];
        cell.facing = facing.map(|f| signed_share(f, showing, 127) as i8);
        // How spread the facings are: none when they all look one way, all when they cancel out.
        let length = isqrt((facing[0] * facing[0] + facing[1] * facing[1] + facing[2] * facing[2]) as u128) as u64;
        cell.facing_spread = (255 - share(length, showing, 255)) as u8;
        cell.light_spread = spread(self.light, self.light_squared, showing).min(255) as u8;
        // The heights' spread in 16ths of a block.
        cell.height_spread = (spread(self.height * 16, self.height_squared * 256, showing)).min(u16::MAX as u64) as u16;
        cell.showing = share(showing, (size as u64).pow(2), 256).min(u16::MAX as u64) as u16;
        cell
    }
}

/// `part` of `whole` in `unit`s, rounded (halves up).
fn share(part: u64, whole: u64, unit: u64) -> u64 {
    ((part as u128 * unit as u128 * 2 + whole as u128) / (2 * whole as u128)) as u64
}

/// The same for a signed part, rounded away from zero at halves.
fn signed_share(part: i64, whole: u64, unit: u64) -> i64 {
    let size = share(part.unsigned_abs(), whole, unit) as i64;
    if part < 0 { -size } else { size }
}

/// The standard deviation of `n` values from their sum and their squares' sum, rounded down.
fn spread(sum: u64, squares: u64, n: u64) -> u64 {
    let (sum, squares, n) = (sum as u128, squares as u128, n as u128);
    isqrt((n * squares).saturating_sub(sum * sum)) as u64 / n as u64
}

/// The whole square root, rounded down.
fn isqrt(v: u128) -> u128 {
    if v < 2 {
        return v;
    }
    let mut x = (v as f64).sqrt() as u128;
    while x * x > v {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= v {
        x += 1;
    }
    x
}

impl Sums {
    /// Its 15 numbers in order, for keeping (`from_numbers` reads them back).
    pub fn to_numbers(&self) -> [u64; 15] {
        let f = self.faces;
        let c = self.colour;
        [self.solid, self.water, f[0], f[1], f[2], f[3], f[4], f[5], c[0], c[1], c[2], self.light, self.light_squared, self.height, self.height_squared]
    }

    pub fn from_numbers(n: [u64; 15]) -> Self {
        Self { solid: n[0], water: n[1], faces: [n[2], n[3], n[4], n[5], n[6], n[7]], colour: [n[8], n[9], n[10]], light: n[11], light_squared: n[12], height: n[13], height_squared: n[14] }
    }
}

impl Cell {
    /// Its 16 bytes, for keeping (`from_bytes` reads them back).
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = self.solid;
        b[1] = self.water;
        b[2] = self.facing[0] as u8;
        b[3] = self.facing[1] as u8;
        b[4] = self.facing[2] as u8;
        b[5] = self.facing_spread;
        b[6] = self.light_spread;
        b[7] = self.spare;
        b[8..12].copy_from_slice(&self.colour.to_le_bytes());
        b[12..14].copy_from_slice(&self.height_spread.to_le_bytes());
        b[14..16].copy_from_slice(&self.showing.to_le_bytes());
        b
    }

    pub fn from_bytes(b: [u8; 16]) -> Self {
        Self {
            solid: b[0],
            water: b[1],
            facing: [b[2] as i8, b[3] as i8, b[4] as i8],
            facing_spread: b[5],
            light_spread: b[6],
            spare: b[7],
            colour: u32::from_le_bytes([b[8], b[9], b[10], b[11]]),
            height_spread: u16::from_le_bytes([b[12], b[13]]),
            showing: u16::from_le_bytes([b[14], b[15]]),
        }
    }
}

/// A cell as the march reads it: 16 bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Cell {
    /// The share of its blocks that are solid, and water, in 255ths.
    pub solid: u8,
    pub water: u8,
    /// The faces' average facing (x north, y east, z up), 127 a whole way.
    pub facing: [i8; 3],
    /// How much the facings disagree (255ths: 0 all one way).
    pub facing_spread: u8,
    /// How much the faces' brightness spreads (its standard deviation, 1,024ths of linear light, at most 255).
    pub light_spread: u8,
    pub spare: u8,
    /// The faces' average colour, 1,024ths of linear light, red, green and blue in ten bits each.
    pub colour: u32,
    /// How much the faces' heights spread (standard deviation, 16ths of a block).
    pub height_spread: u16,
    /// How much surface shows: faces over the cell's floor area, in 256ths.
    pub showing: u16,
}

/// The sums of a box of blocks `size` a side, its least corner at `at`, every block read through `block`
/// (the world's, the faces at the box's edge looking at the blocks just past it). The reference the card's
/// version (3.1b) is tested against.
pub fn sums_of_box(at: [i32; 3], size: i32, block: &impl Fn([i32; 3]) -> BlockId, colours: &FarColours) -> Sums {
    let mut sums = Sums::default();
    for x in at[0]..at[0] + size {
        for y in at[1]..at[1] + size {
            for z in at[2]..at[2] + size {
                let here = [x, y, z];
                let beside = DIRECTIONS.map(|d| block([x + d[0], y + d[1], z + d[2]]));
                sums.add_block(here, block(here), beside, colours);
            }
        }
    }
    sums
}

#[cfg(test)]
mod tests {
    use super::*;

    const STONE: BlockId = BlockId(1);
    const GRASS: BlockId = BlockId(2);
    const WATER: BlockId = BlockId(3);

    fn colours() -> FarColours {
        FarColours { blocks: vec![([0; 3], [0; 3]), ([300, 300, 310], [250, 250, 260]), ([90, 400, 60], [200, 150, 90]), ([20, 60, 200], [20, 60, 200])], water: WATER }
    }

    /// A small world with everything a cell can meet: rolling ground, a pond, an overhang, a cave.
    fn block(at: [i32; 3]) -> BlockId {
        let [x, y, z] = at;
        let mix = (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)) as u32;
        let ground = 20 + ((x / 3 + y / 5).rem_euclid(7)) + (mix % 3) as i32;
        let pond = (x - 10).pow(2) + (y - 12).pow(2) < 30;
        let ledge = (0..6).contains(&x) && z == 30 && y > 3;
        let cave = (z - 14).pow(2) + (y - 8).pow(2) < 12;
        if ledge {
            STONE
        } else if z < ground && !cave && !(pond && z > 17) {
            if z == ground - 1 { GRASS } else { STONE }
        } else if pond && z <= 22 && z > 17 {
            WATER
        } else {
            BlockId::AIR
        }
    }

    /// A cell's sums are exactly its children's added, at every size, wherever it sits: every level is
    /// exactly the sum of the one below.
    #[test]
    fn a_cell_is_its_children_added() {
        let colours = colours();
        for size in [2, 4, 8, 16] {
            for corner in [[0, 0, 16], [-8, 4, 8], [4, -16, 24], [8, 8, 0]] {
                let whole = sums_of_box(corner, size, &block, &colours);
                let half = size / 2;
                let mut children = Sums::default();
                for d in 0..8 {
                    let child = [corner[0] + (d & 1) * half, corner[1] + (d >> 1 & 1) * half, corner[2] + (d >> 2) * half];
                    children.add(&sums_of_box(child, half, &block, &colours));
                }
                assert_eq!(whole, children, "a cell of {size} at {corner:?}");
                if size == 16 {
                    assert!(whole.solid > 0 && whole.showing() > 0, "the test cell at {corner:?} should hold ground");
                }
            }
        }
        // The test world holds what it should: water, faces looking down (the ledge, the cave's roof).
        let all = sums_of_box([-8, -8, 0], 32, &block, &colours);
        assert!(all.water > 0 && all.faces[1] > 0, "{all:?}");
    }

    /// The compact form follows its rule: a cell of 2 with its bottom half grass and its top air.
    #[test]
    fn the_compact_form_follows_its_rule() {
        let colours = colours();
        let flat = |at: [i32; 3]| if at[2] < 1 { GRASS } else { BlockId::AIR };
        let sums = sums_of_box([0, 0, 0], 2, &flat, &colours);
        assert_eq!(sums.faces, [4, 0, 0, 0, 0, 0]);
        let cell = sums.compact(2);
        assert_eq!(cell.solid, 128, "4 of 8 is 127.5, halves up");
        assert_eq!(cell.water, 0);
        assert_eq!(cell.facing, [0, 0, 127]);
        assert_eq!(cell.facing_spread, 0);
        assert_eq!(cell.colour, 90 << 20 | 400 << 10 | 60);
        assert_eq!(cell.light_spread, 0);
        assert_eq!(cell.height_spread, 0);
        assert_eq!(cell.showing, 256, "a whole floor's worth of surface");
        // Facings that cancel spread fully; heights spread by their standard deviation.
        let sums = Sums { faces: [1, 1, 0, 0, 0, 0], colour: [200, 200, 200], height: 10, height_squared: 4 * 4 + 6 * 6, ..Sums::default() };
        let cell = sums.compact(8);
        assert_eq!((cell.facing, cell.facing_spread), ([0, 0, 0], 255));
        assert_eq!(cell.height_spread, 16, "heights 4 and 6: one block either side");
        assert_eq!(Sums::default().compact(4), Cell::default(), "an empty cell");
    }

    #[test]
    fn the_compact_form_is_16_bytes() {
        assert_eq!(std::mem::size_of::<Cell>(), 16);
    }
}

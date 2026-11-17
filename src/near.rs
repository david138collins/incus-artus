//! The near eye's blocks and their faces (plan 2.3): the world close to the eye as triangles, by David's word of
//! 2026-11-16 ("I believe we should try to accomplish the rest of the graphics engine with triangles"; "lets just
//! make a radius slider and get started. the render radius set there will determine where the triangles start and
//! rays fade"). A chunk is 32 x 32 columns, the world's whole height (old's chunk: his "64 chunk render radius" is
//! 2,048 blocks). Its blocks are the doorway's answers; its faces are every side of a solid block that looks into
//! air or water, merged where they can be (greedy meshing), and the water's top.
//!
//! This file, so far: a chunk's blocks and the mesher, checked block by block. The lane that keeps the chunks round
//! the eye made, and the pass that draws them, come with the rest of build 195 (its pass mark:
//! `journal/docs/experiments.md`). Pieces (build 204; David, 2026-11-16: "builder can just scatter random quarters
//! and octs to test it"): a chunk with any quarter block or eighth in it is meshed in half blocks, the whole of it,
//! every octant's open sides merged as whole blocks' are.

// (Nothing outside its tests uses it yet: the lane and the pass are the rest of build 195.)
#![cfg_attr(not(test), allow(dead_code))]

use crate::doorway::{Block, Change, Column, Doorway, Tile, WATER};
use std::collections::HashMap;

/// Columns a side of a chunk.
pub const CHUNK: i32 = 32;

/// The way a face looks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Facing {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Facing {
    pub const ALL: [Facing; 6] = [Facing::North, Facing::South, Facing::East, Facing::West, Facing::Up, Facing::Down];

    /// The step from a cell to the one its face of this facing looks into (north, east, up).
    pub fn step(self) -> [i32; 3] {
        match self {
            Facing::North => [1, 0, 0],
            Facing::South => [-1, 0, 0],
            Facing::East => [0, 1, 0],
            Facing::West => [0, -1, 0],
            Facing::Up => [0, 0, 1],
            Facing::Down => [0, 0, -1],
        }
    }

    /// The two axes a face of this facing spreads along (0 north, 1 east, 2 up), and the axis it looks along: a face
    /// that looks up or down spreads north and east; one that looks north or south, east and up; east or west, north
    /// and up.
    pub fn axes(self) -> ([usize; 2], usize) {
        match self {
            Facing::North | Facing::South => ([1, 2], 0),
            Facing::East | Facing::West => ([0, 2], 1),
            Facing::Up | Facing::Down => ([0, 1], 2),
        }
    }
}

/// The order a chunk's faces are kept in on the card (build 209): in six groups by the way they look, so a group no
/// face of which could look at the eye is left out of a frame's draws. In this order groups that show together mostly
/// lie together (up with north and east, or with west and south), and go as one draw.
pub const GROUPS: [Facing; 6] = [Facing::North, Facing::East, Facing::Up, Facing::West, Facing::South, Facing::Down];

/// Which of a chunk's groups (in `GROUPS`' order) could have a face that looks at `eye` (north, east, up; blocks): a
/// face shows only from the side it looks out of, so one that looks north shows only to an eye north of it, and none
/// of a chunk's does to an eye south of the chunk's south edge. `low` and `high`: the lowest and highest block the
/// chunk's faces touch. Exact the safe way: a group left out has no face the eye could see.
pub fn groups_shown(chunk: [i32; 2], low: i32, high: i32, eye: [f32; 3]) -> [bool; 6] {
    let (north, east) = ((chunk[0] * CHUNK) as f32, (chunk[1] * CHUNK) as f32);
    [eye[0] > north, eye[1] > east, eye[2] > low as f32, eye[1] < east + CHUNK as f32, eye[0] < north + CHUNK as f32, eye[2] < high as f32]
}

/// A chunk's faces in the order the card keeps them: the blocks' sides in their groups (`GROUPS`' order), then the
/// water's tops; with how many each group has, how many water's tops, and the lowest and highest block any face
/// touches (a face's own numbers are in half blocks where the chunk is).
pub fn grouped(blocks: &Blocks) -> (Vec<Face>, [u32; 6], u32, i32, i32) {
    let group_of = |facing: Facing| GROUPS.iter().position(|g| *g == facing).unwrap();
    let mut faces = blocks.faces();
    // (A sort that keeps each kind's own order.)
    faces.sort_by_key(|face| (face.water, group_of(face.facing)));
    let mut groups = [0u32; 6];
    for face in faces.iter().filter(|face| !face.water) {
        groups[group_of(face.facing)] += 1;
    }
    let water = faces.iter().filter(|face| face.water).count() as u32;
    let unit = blocks.unit;
    let (low, high) = faces.iter().fold((blocks.high, blocks.low), |(low, high), face| {
        let top = face.at[2] + if matches!(face.facing, Facing::Up | Facing::Down) { 1 } else { face.high };
        (low.min(face.at[2].div_euclid(unit)), high.max((top + unit - 1).div_euclid(unit)))
    });
    (faces, groups, water, low, high)
}

/// A face: a rectangle of one block's sides that look one way, all in one plane. `at` is its least cell in the chunk
/// (north and east from the chunk's least corner, 0 to 31; up from bedrock); it's `wide` cells along the first of
/// its facing's axes and `high` along the second. `water`: the water's top, not a block's side. In a chunk meshed in
/// half blocks (`Blocks::unit` 2) all of these are in half blocks (north and east 0 to 63).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Face {
    pub at: [i32; 3],
    pub facing: Facing,
    pub wide: i32,
    pub high: i32,
    pub block: Block,
    pub water: bool,
    /// The sky's light in the cell it looks into, 0 to 15 (build 208; 15 with `INCUS_NEAR_LIGHT=0`).
    pub light: u8,
}

/// A chunk's blocks with a column of its neighbours all round (so a side at the chunk's edge knows what it looks
/// into), from `low` up to `high`: under `low` every one of these columns is solid, over `high` all are air, and
/// neither has a face. A cell is 0 for air, or its block's number and one. Where a piece (a quarter block or an
/// eighth) lies in these columns, everything is kept in half blocks: `unit` is 2, a block is eight cells, and the
/// cells are octants.
pub struct Blocks {
    /// The chunk (its place in chunks, north and east: the chunk of column `c` is `c.div_euclid(32)`).
    pub chunk: [i32; 2],
    pub low: i32,
    pub high: i32,
    /// How many cells a block is across: 1, or 2 where there are pieces.
    pub unit: i32,
    /// Columns of its neighbours read on each side: `MARGIN`, or `LAMP_MARGIN` for a placed light's spread.
    margin: i32,
    cells: Vec<u16>,
}

/// Columns read round a chunk on each side: enough for its faces (one) and for the sky's light, which spreads into
/// openings past a chunk's edge (build 208).
const MARGIN: i32 = 8;
const WIDE: usize = (CHUNK + 2 * MARGIN) as usize;

/// A placed light (build 216): how far (blocks) the brightest reaches, a step dimmer a block from 15; and the columns
/// read round a chunk to spread it, so a lamp that far outside the chunk lights it as it would from inside.
const LAMP_REACH: i32 = 14;
const LAMP_MARGIN: i32 = 16;

/// The sky's light (build 208), spread into what's covered by averaging: rounds of it; under `CUT` a cell is black;
/// over it, never darker than `FLOOR` of 15 (David, 2026-11-16: "outdoors should have very very dark as the floor,
/// but caves should be pitch black past a distance into their mouth that would equal i think the diameter of their
/// mouth"). Averaging with the walls counting dark, light in a tube falls about a hundredfold over its width: the cut.
const LIGHT_ROUNDS: usize = 48;
const LIGHT_CUT: f32 = 0.012;
const LIGHT_FLOOR: u8 = 1;

/// How the near eye is lit (`INCUS_NEAR_LIGHT`): not by a light of its own (`0`: every face as to build 207); each
/// face carrying its light, and split from its neighbours by it (`faces`: build 208's way, to measure against); or
/// its light kept apart from the faces, only where it isn't plain daylight (unset: build 210, David's choice of
/// 2026-11-17, "proceed with C").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lit {
    Off,
    Faces,
    Kept,
}

pub fn lit() -> Lit {
    static LIT: std::sync::OnceLock<Lit> = std::sync::OnceLock::new();
    *LIT.get_or_init(|| match std::env::var("INCUS_NEAR_LIGHT").as_deref() {
        Ok("0") => Lit::Off,
        Ok("faces") => Lit::Faces,
        _ => Lit::Kept,
    })
}

/// Whether the near eye works out its own sky light at all.
pub fn light_on() -> bool {
    lit() != Lit::Off
}

/// A side of a brick of light, cells; its words on the card (512 cells of 4 bits); and no brick (plain daylight).
pub const LIGHT_BRICK: i32 = 8;
pub const LIGHT_BRICK_WORDS: usize = 64;
pub const NO_BRICK: u32 = u32::MAX;

/// A chunk's light as the card keeps it (build 210): only where it isn't plain daylight. Bricks of 8 by 8 by 8 cells
/// (whole blocks), 4 bits a cell (the sky's light, 0 to 15; a solid cell 0), a brick only where some cell of it that
/// isn't solid isn't in plain daylight. `words`: first a list, for each layer of bricks from `low` (a layer is 8
/// blocks of height; `layers` of them) its 4 by 4 bricks (north fastest), each where its brick begins in `words` or
/// `NO_BRICK`; then the bricks, a cell at north + east * 8 + up * 64 within its brick, eight cells a word from the
/// low bits. And placed light (build 216): after the sky's list a second like it, each brick's colour brick or
/// `NO_BRICK` (no placed light reaches it); a colour brick is 256 words, two cells a word from the low bits, a cell's
/// red, green and blue in 4 bits each from the low bits.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LightBlock {
    pub low: u32,
    pub layers: u32,
    pub words: Vec<u32>,
}

impl LightBlock {
    /// The sky's light in a cell of the chunk (north and east 0 to 31 from its least corner, up from bedrock), read
    /// as the card reads it (`near.wgsl`, `kept_at`): 15 where there's no brick.
    pub fn at(&self, cell: [i32; 3]) -> u8 {
        let layer = (cell[2] >> 3) - self.low as i32;
        if layer < 0 || layer >= self.layers as i32 {
            return 15;
        }
        let brick = self.words[(layer * 16 + (cell[0] >> 3) + (cell[1] >> 3) * 4) as usize];
        if brick == NO_BRICK {
            return 15;
        }
        let i = ((cell[0] & 7) + (cell[1] & 7) * 8 + (cell[2] & 7) * 64) as usize;
        (self.words[brick as usize + (i >> 3)] >> ((i & 7) * 4) & 15) as u8
    }

    /// The placed light in a cell of the chunk (red, green, blue, each 0 to 15), read as the card reads it: none
    /// where there's no colour brick.
    pub fn placed_at(&self, cell: [i32; 3]) -> [u8; 3] {
        let layer = (cell[2] >> 3) - self.low as i32;
        if layer < 0 || layer >= self.layers as i32 {
            return [0; 3];
        }
        let brick = self.words[(self.layers as i32 * 16 + layer * 16 + (cell[0] >> 3) + (cell[1] >> 3) * 4) as usize];
        if brick == NO_BRICK {
            return [0; 3];
        }
        let i = ((cell[0] & 7) + (cell[1] & 7) * 8 + (cell[2] & 7) * 64) as usize;
        let colour = self.words[brick as usize + (i >> 1)] >> ((i & 1) * 16);
        [(colour & 15) as u8, (colour >> 4 & 15) as u8, (colour >> 8 & 15) as u8]
    }
}

/// A colour brick's words on the card (512 cells of 16 bits).
pub const GLOW_BRICK_WORDS: usize = 256;

/// Whether there are placed lights at all (`INCUS_NEAR_LAMPS=0`: no block gives light, and the scatter stage stands
/// no lamps: build 215's world, to measure against).
pub fn lamps_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("INCUS_NEAR_LAMPS").as_deref() != Ok("0"))
}

/// Which blocks give light (build 216; `data/lights.toml`): for each block by its number, its light's red, green and
/// blue, each 0 to 15 [David, 2026-11-17: "just use red and green blocks for the glowing blocks"; 2026-11-06: "the
/// light from the brazieres, where its very clear steps in brightness"].
#[derive(Clone, Default, Debug)]
pub struct Glows {
    blocks: Vec<[u8; 3]>,
}

#[derive(serde::Deserialize)]
struct GlowsFile {
    #[serde(default)]
    light: Vec<GlowFile>,
}

#[derive(serde::Deserialize)]
struct GlowFile {
    block: String,
    light: [u8; 3],
}

impl Glows {
    /// `data/lights.toml`, each block found by its name through the doorway. None where placed lights are off.
    pub fn load(doorway: &mut Doorway) -> Result<Self, String> {
        if !lamps_on() {
            return Ok(Self::default());
        }
        let path = crate::session::project_folder().join("data").join("lights.toml");
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let file: GlowsFile = toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut blocks = Vec::new();
        for glow in file.light {
            let block = doorway.block(&glow.block) as usize;
            if blocks.len() <= block {
                blocks.resize(block + 1, [0; 3]);
            }
            blocks[block] = glow.light.map(|level| level.min(15));
        }
        Ok(Self { blocks })
    }

    /// A block's light, if it gives any.
    pub fn of(&self, block: Block) -> Option<[u8; 3]> {
        self.blocks.get(block as usize).copied().filter(|light| *light != [0; 3])
    }

    /// Whether no block gives light.
    pub fn none(&self) -> bool {
        self.blocks.iter().all(|light| *light == [0; 3])
    }
}

/// The placed light in a chunk's own columns (build 216), a cell a whole block, from `low` up to `high`: its red,
/// green and blue in 4 bits each from the low bits, at north + east x 32 + (up - low) x 1,024; 0 in a cell no light
/// reaches, or that a whole block fills.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Placed {
    pub low: i32,
    pub high: i32,
    pub cells: Vec<u16>,
}

impl Placed {
    pub fn at(&self, north: i32, east: i32, up: i32) -> u16 {
        if up < self.low || up >= self.high {
            return 0;
        }
        self.cells[(north + east * CHUNK + (up - self.low) * CHUNK * CHUNK) as usize]
    }
}

/// A chunk's placed light (build 216), if a glowing block is near enough to reach it: the glowing blocks the scatter
/// stage stood in the tiles round it (each tile's found once and kept in `lamps`) and those placed through the
/// doorway; if any is within a light's reach of the chunk's columns, its blocks are read again 16 columns out and the
/// light spread through them (`Blocks::placed_light`). A chunk with none near pays only this asking.
pub fn placed_for(doorway: &Doorway, tiles: &mut HashMap<[i32; 2], Tile>, lamps: &mut HashMap<[i32; 2], Vec<[i32; 3]>>, glows: &Glows, chunk: [i32; 2]) -> Result<Option<Placed>, String> {
    if glows.none() {
        return Ok(None);
    }
    let lo = [chunk[0] * CHUNK - LAMP_REACH, chunk[1] * CHUNK - LAMP_REACH];
    let hi = [chunk[0] * CHUNK + CHUNK - 1 + LAMP_REACH, chunk[1] * CHUNK + CHUNK - 1 + LAMP_REACH];
    let near = |cell: &[i32; 3]| cell[0] >= lo[0] && cell[0] <= hi[0] && cell[1] >= lo[1] && cell[1] <= hi[1];
    let mut found: Vec<[i32; 3]> = Vec::new();
    let mut asked: Vec<[i32; 2]> = Vec::new();
    for corner in [[lo[0], lo[1]], [hi[0], lo[1]], [lo[0], hi[1]], [hi[0], hi[1]]] {
        let (tile, _) = doorway.place(corner[0], corner[1]);
        if asked.contains(&tile) {
            continue;
        }
        asked.push(tile);
        if !lamps.contains_key(&tile) {
            if !tiles.contains_key(&tile) {
                tiles.insert(tile, doorway.tile(tile[0], tile[1])?);
            }
            lamps.insert(tile, tiles[&tile].glowing(doorway.tile_least(tile), |block| glows.of(block).is_some()));
        }
        found.extend(lamps[&tile].iter().filter(|cell| near(cell)));
        for change in doorway.changes_in(tile[0], tile[1]) {
            if let Change::Block { cell, block } = change {
                if glows.of(*block).is_some() && near(cell) {
                    found.push(*cell);
                }
            }
        }
    }
    if found.is_empty() {
        return Ok(None);
    }
    Ok(Blocks::read_out(doorway, tiles, chunk, LAMP_MARGIN)?.placed_light(&found, glows))
}

impl Blocks {
    /// A chunk's blocks, read through the doorway (the tiles it needs are read once each into `tiles` and kept
    /// there for the next chunk), with the whole blocks placed since (`Change::Block`; bars aren't the near eye's
    /// yet).
    pub fn read(doorway: &Doorway, tiles: &mut HashMap<[i32; 2], Tile>, chunk: [i32; 2]) -> Result<Self, String> {
        Self::read_out(doorway, tiles, chunk, MARGIN)
    }

    /// The same, with `margin` columns of its neighbours all round (`LAMP_MARGIN` for a placed light's spread).
    pub fn read_out(doorway: &Doorway, tiles: &mut HashMap<[i32; 2], Tile>, chunk: [i32; 2], margin: i32) -> Result<Self, String> {
        let across = (CHUNK + 2 * margin) as usize;
        let least = [chunk[0] * CHUNK - margin, chunk[1] * CHUNK - margin];
        let height = doorway.height();
        let mut columns: Vec<Column> = Vec::with_capacity(across * across);
        // (East slowest, north fastest: a column's place here is north + east x 34.)
        for east in 0..across as i32 {
            for north in 0..across as i32 {
                let (tile, [i, j]) = doorway.place(least[0] + north, least[1] + east);
                let inside = tile[0] >= 0 && tile[1] >= 0 && tile[0] * doorway.grid().1 < doorway.across() && tile[1] * doorway.grid().1 < doorway.across();
                if !inside {
                    columns.push(Column::Past);
                    continue;
                }
                if !tiles.contains_key(&tile) {
                    tiles.insert(tile, doorway.tile(tile[0], tile[1])?);
                }
                columns.push(tiles[&tile].column(doorway, i, j)?);
            }
        }
        // What was placed in these columns, in the order it was made: whole blocks, and pieces by their octants.
        let mut placed: Vec<Change> = Vec::new();
        let inside = |cell: [i32; 3]| {
            let local = [cell[0] - least[0], cell[1] - least[1]];
            local[0] >= 0 && local[1] >= 0 && local[0] < across as i32 && local[1] < across as i32 && cell[2] >= 0 && cell[2] < height
        };
        let mut asked: Vec<[i32; 2]> = Vec::new();
        for corner in [[0, 0], [across as i32 - 1, 0], [0, across as i32 - 1], [across as i32 - 1, across as i32 - 1]] {
            let (tile, _) = doorway.place(least[0] + corner[0], least[1] + corner[1]);
            if !asked.contains(&tile) {
                asked.push(tile);
                for change in doorway.changes_in(tile[0], tile[1]) {
                    let here = match change.octants() {
                        None => inside(change.cell()),
                        Some((octants, _)) => octants.iter().any(|octant| inside(octant.map(|half| half.div_euclid(2)))),
                    };
                    if here {
                        placed.push(*change);
                    }
                }
            }
        }
        // (A change is kept with its cell's tile. A bar whose cell is outside these columns can reach half a block
        // into their outermost ring, the neighbours' column: only its far half, which no side of this chunk's own
        // blocks looks into. So the tiles of these columns are all that's asked.)
        let unit: i32 = if placed.iter().any(|change| change.octants().is_some()) { 2 } else { 1 };
        // From the lowest cell that isn't solid in any of the columns (one under it, so its floor's face is in) to the
        // highest that is anything at all.
        let mut low = height;
        let mut high = 0;
        for column in &columns {
            match column {
                Column::Past => low = 0,
                Column::Here { runs, .. } => {
                    let mut open = 0;
                    for run in runs {
                        if run.from > open || run.block == WATER {
                            break;
                        }
                        open = run.to;
                    }
                    low = low.min(open);
                    high = high.max(runs.last().map_or(0, |run| run.to));
                }
            }
        }
        for change in &placed {
            let cells = match change.octants() {
                None => vec![change.cell()],
                Some((octants, _)) => octants.iter().map(|octant| octant.map(|half| half.div_euclid(2))).collect(),
            };
            for cell in cells.into_iter().filter(|cell| inside(*cell)) {
                low = low.min(cell[2]);
                high = high.max(cell[2] + 1);
            }
        }
        let low = (low - 1).max(0);
        let high = (high + 1).min(height).max(low + 1);
        let (u, wide, tall) = (unit as usize, across * unit as usize, ((high - low) * unit) as usize);
        let mut cells = vec![0u16; wide * wide * tall];
        for (k, column) in columns.iter().enumerate() {
            if let Column::Here { runs, .. } = column {
                let (north, east) = (k % across, k / across);
                for run in runs {
                    for z in (run.from.max(low) - low) as usize * u..(run.to.min(high) - low).max(0) as usize * u {
                        for de in 0..u {
                            for dn in 0..u {
                                cells[(north * u + dn + (east * u + de) * wide) * tall + z] = run.block + 1;
                            }
                        }
                    }
                }
            }
        }
        for change in placed {
            match change.octants() {
                // A whole block: its cell, all of it.
                None => {
                    let Change::Block { cell, block } = change else { continue };
                    let (north, east, z) = ((cell[0] - least[0]) as usize, (cell[1] - least[1]) as usize, (cell[2] - low) as usize);
                    for dz in 0..u {
                        for de in 0..u {
                            for dn in 0..u {
                                cells[(north * u + dn + (east * u + de) * wide) * tall + z * u + dz] = block + 1;
                            }
                        }
                    }
                }
                // A piece: each of its octants that's empty (air or water): it doesn't displace ground, and where two
                // pieces would share an octant the first placed keeps it.
                Some((octants, block)) => {
                    for octant in octants {
                        let local = [octant[0] - least[0] * 2, octant[1] - least[1] * 2, octant[2] - low * 2];
                        if local[0] < 0 || local[1] < 0 || local[2] < 0 || local[0] >= wide as i32 || local[1] >= wide as i32 || local[2] >= tall as i32 {
                            continue;
                        }
                        let at = (local[0] as usize + local[1] as usize * wide) * tall + local[2] as usize;
                        if cells[at] == 0 || cells[at] == WATER + 1 {
                            cells[at] = block + 1;
                        }
                    }
                }
            }
        }
        Ok(Self { chunk, low, high, unit, margin, cells })
    }

    /// Columns a side of what's read: the chunk's and its margin's.
    fn wide(&self) -> usize {
        (CHUNK + 2 * self.margin) as usize
    }

    /// The cell at `north`, `east` from the chunk's least corner (-8 to 39) and `up` from bedrock: 0 air, or its
    /// block's number and one. Under what's kept, solid stone; over it, air. (In half blocks where `unit` is 2: -16 to
    /// 79, and `up` in half blocks.)
    pub fn cell(&self, north: i32, east: i32, up: i32) -> u16 {
        let u = self.unit;
        if up < self.low * u {
            return crate::doorway::STONE + 1;
        }
        if up >= self.high * u {
            return 0;
        }
        let tall = ((self.high - self.low) * u) as usize;
        self.cells[((north + self.margin * u) as usize + (east + self.margin * u) as usize * self.wide() * u as usize) * tall + (up - self.low * u) as usize]
    }

    /// Whether a cell stops the eye: a block, not air and not water.
    fn solid(cell: u16) -> bool {
        cell != 0 && cell != WATER + 1
    }

    /// The sky's light in every cell (build 208), 0 to 15, laid out as the cells are: a cell nothing solid stands over
    /// is 15; every other air or water cell takes, round after round, the average of its six neighbours (a solid one
    /// counting dark; over what's kept, the open sky; past the margin, itself), so light spreads into an opening and
    /// fades with the walls round it, reaching about as far in as the opening is wide. Then in 16 steps: under
    /// `LIGHT_CUT` black, over it never darker than `LIGHT_FLOOR`. Solid cells are 0.
    pub fn sky_light(&self) -> Vec<u8> {
        let u = self.unit as usize;
        let wide = self.wide() * u;
        let tall = ((self.high - self.low) * self.unit) as usize;
        let mut light = vec![0f32; self.cells.len()];
        // Each column, top down to its first solid cell: open; under it, the air and water covered.
        let mut covered: Vec<usize> = Vec::new();
        for column in 0..wide * wide {
            let base = column * tall;
            let mut z = tall;
            while z > 0 && !Self::solid(self.cells[base + z - 1]) {
                z -= 1;
                light[base + z] = 1.0;
            }
            covered.extend((0..z).map(|k| base + k).filter(|&i| !Self::solid(self.cells[i])));
        }
        if !covered.is_empty() {
            // Each covered cell's six neighbours: a cell's place; or the open sky (above what's kept); or itself (past
            // the margin). Below what's kept is solid, and solid cells stay dark.
            const SKY: u32 = u32::MAX;
            let near: Vec<[u32; 6]> = covered
                .iter()
                .map(|&i| {
                    let (z, column) = (i % tall, i / tall);
                    let (north, east) = (column % wide, column / wide);
                    let at = |n: usize, e: usize, z: usize| ((n + e * wide) * tall + z) as u32;
                    [
                        if north + 1 < wide { at(north + 1, east, z) } else { i as u32 },
                        if north > 0 { at(north - 1, east, z) } else { i as u32 },
                        if east + 1 < wide { at(north, east + 1, z) } else { i as u32 },
                        if east > 0 { at(north, east - 1, z) } else { i as u32 },
                        if z + 1 < tall { at(north, east, z + 1) } else { SKY },
                        if z > 0 { at(north, east, z - 1) } else { i as u32 },
                    ]
                })
                .collect();
            for _ in 0..LIGHT_ROUNDS {
                for (k, &i) in covered.iter().enumerate() {
                    let sum: f32 = near[k].iter().map(|&j| if j == SKY { 1.0 } else { light[j as usize] }).sum();
                    light[i] = sum / 6.0;
                }
            }
        }
        light
            .iter()
            .zip(&self.cells)
            .map(|(&v, &cell)| {
                if Self::solid(cell) || v < LIGHT_CUT {
                    0
                } else if v >= 1.0 {
                    15
                } else {
                    ((v * 15.0).ceil() as u8).clamp(LIGHT_FLOOR, 15)
                }
            })
            .collect()
    }

    /// The sky's light in a block of the chunk's own columns (north and east 0 to 31, up from bedrock) from
    /// `sky_light`'s cells: its cell's; where the chunk is in half blocks, the brightest of its octants that aren't
    /// solid. None where it's solid through. (Under what's kept, solid; over it, plain daylight.)
    pub fn light_of(&self, lights: &[u8], north: i32, east: i32, up: i32) -> Option<u8> {
        if up < self.low {
            return None;
        }
        if up >= self.high {
            return Some(15);
        }
        let u = self.unit;
        let wide = self.wide() * u as usize;
        let tall = ((self.high - self.low) * u) as usize;
        let mut most: Option<u8> = None;
        for k in 0..u * u * u {
            let (n, e, z) = (north * u + k % u, east * u + k / u % u, (up - self.low) * u + k / (u * u));
            let at = ((n + self.margin * u) as usize + (e + self.margin * u) as usize * wide) * tall + z as usize;
            if !Self::solid(self.cells[at]) {
                most = Some(most.map_or(lights[at], |m| m.max(lights[at])));
            }
        }
        most
    }

    /// Whether a whole block (north and east from the chunk's least corner, up from bedrock) stops a placed light:
    /// solid through (a piece in it lets light by).
    fn stops(&self, north: i32, east: i32, up: i32) -> bool {
        let u = self.unit;
        (0..u * u * u).all(|k| Self::solid(self.cell(north * u + k % u, east * u + k / u % u, up * u + k / (u * u))))
    }

    /// The placed light in the chunk's own columns (build 216) from the glowing blocks at `lamps` (cells of the
    /// world; one that isn't a glowing block now, or lies outside what's read, gives none): from each, its light a
    /// step dimmer each block, six ways, through every block that isn't solid through (air, water, pieces), red,
    /// green and blue each on its own, a cell keeping the brightest that reaches it. None if nothing reaches the
    /// chunk. (For blocks read `LAMP_MARGIN` out, so a lamp outside the chunk lights it as it would from inside.)
    pub fn placed_light(&self, lamps: &[[i32; 3]], glows: &Glows) -> Option<Placed> {
        let least = [self.chunk[0] * CHUNK, self.chunk[1] * CHUNK];
        let (from, to) = (-self.margin, CHUNK + self.margin);
        let seeds: Vec<([i32; 3], [u8; 3])> = lamps
            .iter()
            .filter_map(|lamp| {
                let local = [lamp[0] - least[0], lamp[1] - least[1], lamp[2]];
                if local[0] < from || local[0] >= to || local[1] < from || local[1] >= to || local[2] < self.low || local[2] >= self.high {
                    return None;
                }
                let cell = self.cell(local[0] * self.unit, local[1] * self.unit, local[2] * self.unit);
                if cell == 0 {
                    return None;
                }
                glows.of(cell - 1).map(|light| (local, light))
            })
            .collect();
        let low = (seeds.iter().map(|seed| seed.0[2]).min()? - LAMP_REACH).max(self.low);
        let high = (seeds.iter().map(|seed| seed.0[2]).max()? + LAMP_REACH + 1).min(self.high);
        let wide = self.wide() as i32;
        let at = |cell: [i32; 3]| ((cell[0] - from) + (cell[1] - from) * wide + (cell[2] - low) * wide * wide) as usize;
        let mut open = vec![false; (wide * wide * (high - low)) as usize];
        for up in low..high {
            for east in from..to {
                for north in from..to {
                    open[at([north, east, up])] = !self.stops(north, east, up);
                }
            }
        }
        let mut levels = vec![[0u8; 3]; open.len()];
        let mut queue: std::collections::VecDeque<[i32; 3]> = Default::default();
        for channel in 0..3 {
            for (cell, light) in &seeds {
                if light[channel] > levels[at(*cell)][channel] {
                    levels[at(*cell)][channel] = light[channel];
                    queue.push_back(*cell);
                }
            }
            while let Some(cell) = queue.pop_front() {
                let level = levels[at(cell)][channel];
                if level <= 1 {
                    continue;
                }
                for step in Facing::ALL.map(Facing::step) {
                    let next = [cell[0] + step[0], cell[1] + step[1], cell[2] + step[2]];
                    if next[0] < from || next[0] >= to || next[1] < from || next[1] >= to || next[2] < low || next[2] >= high {
                        continue;
                    }
                    let k = at(next);
                    if open[k] && levels[k][channel] < level - 1 {
                        levels[k][channel] = level - 1;
                        queue.push_back(next);
                    }
                }
            }
        }
        let mut cells = vec![0u16; (CHUNK * CHUNK * (high - low)) as usize];
        let mut any = false;
        for up in low..high {
            for east in 0..CHUNK {
                for north in 0..CHUNK {
                    let k = at([north, east, up]);
                    if open[k] && levels[k] != [0; 3] {
                        cells[(north + east * CHUNK + (up - low) * CHUNK * CHUNK) as usize] = levels[k][0] as u16 | (levels[k][1] as u16) << 4 | (levels[k][2] as u16) << 8;
                        any = true;
                    }
                }
            }
        }
        any.then_some(Placed { low, high, cells })
    }

    /// The chunk's light as the card keeps it (build 210), from `sky_light`'s cells, and its placed light if any
    /// reaches it (build 216): none if every cell of its own columns that isn't solid is in plain daylight and no
    /// placed light reaches it.
    pub fn light_block(&self, lights: &[u8], placed: Option<&Placed>) -> Option<LightBlock> {
        // The bricks with a cell that isn't in plain daylight, and those a placed light reaches: for each layer from
        // the lowest kept, which of its 16.
        let base = self.low >> 3;
        let mut wanted = vec![false; (((self.high - 1) >> 3) - base + 1) as usize * 16];
        let mut tinted = vec![false; wanted.len()];
        if self.unit == 1 {
            // (Whole blocks: straight down each column's cells.)
            let tall = (self.high - self.low) as usize;
            for east in 0..CHUNK {
                for north in 0..CHUNK {
                    let column = ((north + self.margin) as usize + (east + self.margin) as usize * self.wide()) * tall;
                    let brick = ((north >> 3) + (east >> 3) * 4) as usize;
                    for z in 0..tall {
                        if lights[column + z] < 15 && !Self::solid(self.cells[column + z]) {
                            wanted[(((self.low + z as i32) >> 3) - base) as usize * 16 + brick] = true;
                        }
                    }
                }
            }
        } else {
            for east in 0..CHUNK {
                for north in 0..CHUNK {
                    for up in self.low..self.high {
                        if self.light_of(lights, north, east, up).is_some_and(|light| light < 15) {
                            wanted[((up >> 3) - base) as usize * 16 + ((north >> 3) + (east >> 3) * 4) as usize] = true;
                        }
                    }
                }
            }
        }
        // (No face looks into a cell over what's kept or under it: placed light there isn't kept.)
        if let Some(placed) = placed {
            for up in placed.low.max(self.low)..placed.high.min(self.high) {
                for east in 0..CHUNK {
                    for north in 0..CHUNK {
                        if placed.at(north, east, up) != 0 {
                            tinted[((up >> 3) - base) as usize * 16 + ((north >> 3) + (east >> 3) * 4) as usize] = true;
                        }
                    }
                }
            }
        }
        let low = base + ((0..wanted.len()).position(|k| wanted[k] || tinted[k])? / 16) as i32;
        let high = base + ((0..wanted.len()).rposition(|k| wanted[k] || tinted[k])? / 16) as i32;
        let layers = (high - low + 1) as usize;
        // The two lists (the sky's bricks, then the colour bricks), then the bricks.
        let mut words = vec![NO_BRICK; layers * 32];
        for k in (0..wanted.len()).filter(|&k| wanted[k]) {
            let (layer, brick) = (base + (k / 16) as i32, (k % 16) as i32);
            let first = words.len();
            words[(layer - low) as usize * 16 + brick as usize] = first as u32;
            words.resize(first + LIGHT_BRICK_WORDS, 0);
            let (north, east, up) = ((brick % 4) * LIGHT_BRICK, (brick / 4) * LIGHT_BRICK, layer * LIGHT_BRICK);
            for i in 0..512 {
                let light = self.light_of(lights, north + i % 8, east + i / 8 % 8, up + i / 64).unwrap_or(0) as u32;
                words[first + (i >> 3) as usize] |= light << ((i & 7) * 4);
            }
        }
        if let Some(placed) = placed {
            for k in (0..tinted.len()).filter(|&k| tinted[k]) {
                let (layer, brick) = (base + (k / 16) as i32, (k % 16) as i32);
                let first = words.len();
                words[layers * 16 + (layer - low) as usize * 16 + brick as usize] = first as u32;
                words.resize(first + GLOW_BRICK_WORDS, 0);
                let (north, east, up) = ((brick % 4) * LIGHT_BRICK, (brick / 4) * LIGHT_BRICK, layer * LIGHT_BRICK);
                for i in 0..512 {
                    let colour = if up + i / 64 >= self.low && up + i / 64 < self.high { placed.at(north + i % 8, east + i / 8 % 8, up + i / 64) as u32 } else { 0 };
                    words[first + (i >> 1) as usize] |= colour << ((i & 1) * 16);
                }
            }
        }
        Some(LightBlock { low: low as u32, layers: layers as u32, words })
    }

    /// The chunk's faces, merged: for each facing and each plane across it, the open sides of one block that make a
    /// rectangle become one face (the rectangle grown along its first axis, then its second: greedy meshing). A
    /// block's side is open where the cell it looks into isn't solid (air or water); the water's top is where water
    /// has air over it.
    pub fn faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();
        let unit = self.unit;
        let size = [CHUNK * unit, CHUNK * unit, (self.high - self.low) * unit];
        // The sky's light in the cell each face looks into (build 208): faces of one block merge only where it's the
        // same. (Over what's kept, open sky; off, every face as if open.)
        let lights = if lit() == Lit::Faces { self.sky_light() } else { Vec::new() };
        let tall = ((self.high - self.low) * unit) as usize;
        let light_at = |north: i32, east: i32, up: i32| -> u32 {
            if lights.is_empty() || up >= self.high * unit {
                return 15;
            }
            if up < self.low * unit {
                return 0;
            }
            lights[((north + self.margin * unit) as usize + (east + self.margin * unit) as usize * self.wide() * unit as usize) * tall + (up - self.low * unit) as usize] as u32
        };
        for facing in Facing::ALL {
            let ([a, b], along) = facing.axes();
            let step = facing.step();
            let (wide, high) = (size[a] as usize, size[b] as usize);
            // What shows at each place of the plane: 0 nothing; a block's number and one; water's top marked by the
            // highest bit.
            let mut mask = vec![0u32; wide * high];
            for plane in 0..size[along] {
                let mut any = false;
                for v in 0..high {
                    for u in 0..wide {
                        let mut at = [0i32; 3];
                        at[a] = u as i32;
                        at[b] = v as i32;
                        at[along] = plane;
                        let (north, east, up) = (at[0], at[1], at[2] + self.low * unit);
                        let cell = self.cell(north, east, up);
                        let beyond = self.cell(north + step[0], east + step[1], up + step[2]);
                        // (The world's floor shows no underside.)
                        let lit = || light_at(north + step[0], east + step[1], up + step[2]) << 20;
                        let shows = if Self::solid(cell) {
                            if !Self::solid(beyond) && !(facing == Facing::Down && up == 0) { cell as u32 | lit() } else { 0 }
                        } else if cell == WATER + 1 && facing == Facing::Up && beyond == 0 {
                            1 << 31 | cell as u32 | lit()
                        } else {
                            0
                        };
                        mask[u + v * wide] = shows;
                        any |= shows != 0;
                    }
                }
                if !any {
                    continue;
                }
                for v in 0..high {
                    let mut u = 0;
                    while u < wide {
                        let shows = mask[u + v * wide];
                        if shows == 0 {
                            u += 1;
                            continue;
                        }
                        // (No wider than 32 nor higher than 1,024, as a face is packed: whole blocks never are; in
                        // half blocks a face that would be is two.)
                        let mut across = 1;
                        while u + across < wide && across < 32 && mask[u + across + v * wide] == shows {
                            across += 1;
                        }
                        let mut down = 1;
                        while v + down < high && down < 1024 && (0..across).all(|k| mask[u + k + (v + down) * wide] == shows) {
                            down += 1;
                        }
                        for dv in 0..down {
                            for du in 0..across {
                                mask[u + du + (v + dv) * wide] = 0;
                            }
                        }
                        let mut at = [0i32; 3];
                        at[a] = u as i32;
                        at[b] = v as i32;
                        at[along] = plane;
                        at[2] += self.low * unit;
                        faces.push(Face { at, facing, wide: across as i32, high: down as i32, block: (shows & 0xFFFF) as Block - 1, water: shows >> 31 == 1, light: ((shows >> 20) & 15) as u8 });
                        u += across;
                    }
                }
            }
        }
        faces
    }
}

/// The chunks whose faces a change in `cell` (north, east, up) can alter: its own, and the one across each edge of
/// its chunk it lies on (a face depends only on the cell it's on and the one it looks into).
pub fn touched(cell: [i32; 3]) -> Vec<[i32; 2]> {
    touched_cell(cell)
}

/// The chunks a change can alter the faces or the light of (build 208): those `touched_by` has, and every chunk whose
/// margin the light is read over reaches its cells (the placed light's, 16 columns, since build 216: a wall put up or
/// a lamp set down changes the light that far. The whole chunk is made again, where only its light need be).
pub fn touched_by_light(change: &Change) -> Vec<[i32; 2]> {
    let mut chunks = touched_by(change);
    let cells = match change.octants() {
        None => vec![change.cell()],
        Some((octants, _)) => octants.iter().map(|octant| octant.map(|half| half.div_euclid(2))).collect(),
    };
    for cell in cells {
        for dn in [-LAMP_MARGIN, 0, LAMP_MARGIN] {
            for de in [-LAMP_MARGIN, 0, LAMP_MARGIN] {
                let chunk = [(cell[0] + dn).div_euclid(CHUNK), (cell[1] + de).div_euclid(CHUNK)];
                if !chunks.contains(&chunk) {
                    chunks.push(chunk);
                }
            }
        }
    }
    chunks
}

/// The chunks a change can alter the faces of: a whole block's cell's, or each of a piece's octants' cells'.
pub fn touched_by(change: &Change) -> Vec<[i32; 2]> {
    let mut chunks = Vec::new();
    let cells = match change.octants() {
        None => vec![change.cell()],
        Some((octants, _)) => octants.iter().map(|octant| octant.map(|half| half.div_euclid(2))).collect(),
    };
    for cell in cells {
        for chunk in touched_cell(cell) {
            if !chunks.contains(&chunk) {
                chunks.push(chunk);
            }
        }
    }
    chunks
}

fn touched_cell(cell: [i32; 3]) -> Vec<[i32; 2]> {
    let chunk = [cell[0].div_euclid(CHUNK), cell[1].div_euclid(CHUNK)];
    let local = [cell[0].rem_euclid(CHUNK), cell[1].rem_euclid(CHUNK)];
    let mut chunks = vec![chunk];
    for axis in 0..2 {
        if local[axis] == 0 {
            let mut other = chunk;
            other[axis] -= 1;
            chunks.push(other);
        }
        if local[axis] == CHUNK - 1 {
            let mut other = chunk;
            other[axis] += 1;
            chunks.push(other);
        }
    }
    chunks
}

/// A face as the card is given it, three words: its place in its chunk, its facing and its width; its height and
/// its block; its chunk. (North and east in 6 bits each, up in 10, the facing in 3, the width less one in 5, water's
/// top marked, and the top bit set where it's all in half blocks (`unit` 2: north and east then 0 to 63, and up to
/// 2,047, its eleventh bit kept in the second word); the height less one in 10, the block in 16, up's eleventh bit;
/// the chunk's north and east, each with 32,768 added, in 16 bits each.)
pub fn packed(face: &Face, chunk: [i32; 2], unit: i32) -> [u32; 3] {
    let facing = Facing::ALL.iter().position(|f| *f == face.facing).unwrap() as u32;
    let up = face.at[2] as u32;
    let a = face.at[0] as u32 | (face.at[1] as u32) << 6 | (up & 1023) << 12 | facing << 22 | ((face.wide - 1) as u32) << 25 | (face.water as u32) << 30 | ((unit == 2) as u32) << 31;
    let b = (face.high - 1) as u32 | (face.block as u32) << 10 | (up >> 10 & 1) << 26 | (face.light as u32 & 15) << 27;
    let c = (chunk[0] + 32768) as u32 | ((chunk[1] + 32768) as u32) << 16;
    [a, b, c]
}

/// A chunk as the lane hands it over: its packed faces, the blocks' sides first (in their groups by the way they look,
/// `groups` of each in `GROUPS`' order) and then the water's tops (`water` of them, so each kind is drawn by its own
/// pass with no word of the other's), the lowest and highest block any of them touches (for leaving it out when it's
/// out of view), how long it took to make (ms), and which telling of the changes it was made from (`Lane::changed`'s
/// count; 0 before any). And its light, where some of it isn't plain daylight (build 210).
pub struct Made {
    pub chunk: [i32; 2],
    pub light: Option<LightBlock>,
    pub faces: Vec<[u32; 3]>,
    pub groups: [u32; 6],
    pub water: u32,
    pub low: i32,
    pub high: i32,
    pub ms: f32,
    pub told: u64,
}

/// The chunks within `radius` chunks of `eye` (north, east, blocks): every chunk any part of which is that near, so
/// the whole circle is covered. Nearest first.
pub fn wanted(eye: [f32; 2], radius: f32) -> Vec<[i32; 2]> {
    let reach = radius * CHUNK as f32;
    let middle = [(eye[0] / CHUNK as f32).floor() as i32, (eye[1] / CHUNK as f32).floor() as i32];
    let span = radius.ceil() as i32 + 1;
    let mut chunks: Vec<(f32, [i32; 2])> = Vec::new();
    for north in middle[0] - span..=middle[0] + span {
        for east in middle[1] - span..=middle[1] + span {
            let off = nearest_part([north, east], eye);
            if off <= reach {
                chunks.push((off, [north, east]));
            }
        }
    }
    chunks.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    chunks.into_iter().map(|(_, chunk)| chunk).collect()
}

/// How far (blocks, level) the nearest part of a chunk is from `eye`.
pub fn nearest_part(chunk: [i32; 2], eye: [f32; 2]) -> f32 {
    let side = CHUNK as f32;
    let d: Vec<f32> = (0..2).map(|k| (chunk[k] as f32 * side - eye[k]).max(eye[k] - (chunk[k] + 1) as f32 * side).max(0.0)).collect();
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// What the lane's threads share: the chunks asked for, nearest first; the changes made through the game's doorway
/// as the lane was last told them, and how many times it's been told; and whether to stop.
struct Asked {
    queue: std::collections::VecDeque<[i32; 2]>,
    changes: std::sync::Arc<HashMap<[i32; 2], Vec<Change>>>,
    told: u64,
    stop: bool,
}

/// The near lane: threads of its own that make chunks' faces as they're asked for, each through a doorway of its
/// own, handing each over as it's done. The asking is the frame's: `ask` replaces what's waiting (so a change of
/// place or radius is heard at the next chunk), and what a thread already has in hand is finished and handed over.
pub struct Lane {
    asked: std::sync::Arc<(std::sync::Mutex<Asked>, std::sync::Condvar)>,
    made: std::sync::mpsc::Receiver<Result<Made, String>>,
    pub threads: usize,
}

impl Lane {
    /// Starts the lane on the map behind the doorway in `folder` (with the scatter stage if `staged`), on `threads`.
    pub fn start(folder: std::path::PathBuf, staged: bool, threads: usize) -> Result<Self, String> {
        // (Opened once here so a world that can't be opened is said at once, not by a thread later.)
        Doorway::open_with(&folder, staged)?;
        let asked = std::sync::Arc::new((std::sync::Mutex::new(Asked { queue: Default::default(), changes: Default::default(), told: 0, stop: false }), std::sync::Condvar::new()));
        let (sender, made) = std::sync::mpsc::channel();
        for k in 0..threads.max(1) {
            let (asked, sender, folder) = (asked.clone(), sender.clone(), folder.clone());
            std::thread::Builder::new()
                .name(format!("the near lane {k}"))
                .spawn(move || {
                    let mut doorway = match Doorway::open_with(&folder, staged) {
                        Ok(doorway) => doorway,
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            return;
                        }
                    };
                    // The telling of the changes this thread's doorway holds.
                    let mut told = 0u64;
                    // A few tiles kept (a chunk's columns come from one tile, or two or four at a tile's edge).
                    let mut tiles: HashMap<[i32; 2], Tile> = HashMap::new();
                    // Which blocks give light (build 216), and each tile's glowing blocks, kept with the tiles.
                    let glows = match Glows::load(&mut doorway) {
                        Ok(glows) => glows,
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            return;
                        }
                    };
                    let mut lamps: HashMap<[i32; 2], Vec<[i32; 3]>> = HashMap::new();
                    loop {
                        let (chunk, news) = {
                            let (lock, heard) = &*asked;
                            let mut asked = lock.lock().unwrap();
                            loop {
                                if asked.stop {
                                    return;
                                }
                                if let Some(chunk) = asked.queue.pop_front() {
                                    break (chunk, (asked.told != told).then(|| (asked.told, asked.changes.clone())));
                                }
                                asked = heard.wait(asked).unwrap();
                            }
                        };
                        // The changes, if the lane's been told of them since this thread's last chunk (copied here,
                        // outside the lock).
                        if let Some((now, changes)) = news {
                            doorway.set_changes((*changes).clone());
                            told = now;
                        }
                        let began = std::time::Instant::now();
                        if tiles.len() > 6 {
                            tiles.clear();
                            lamps.clear();
                        }
                        let made = Blocks::read(&doorway, &mut tiles, chunk).and_then(|blocks| {
                            let (faces, groups, water, low, high) = grouped(&blocks);
                            let unit = blocks.unit;
                            let light = if lit() == Lit::Kept {
                                let placed = placed_for(&doorway, &mut tiles, &mut lamps, &glows, chunk)?;
                                blocks.light_block(&blocks.sky_light(), placed.as_ref())
                            } else {
                                None
                            };
                            Ok(Made { chunk, light, faces: faces.iter().map(|face| packed(face, chunk, unit)).collect(), groups, water, low, high, ms: began.elapsed().as_secs_f32() * 1000.0, told })
                        });
                        if sender.send(made).is_err() {
                            return;
                        }
                    }
                })
                .map_err(|error| format!("the near lane's thread couldn't start: {error}"))?;
        }
        Ok(Self { asked, made, threads: threads.max(1) })
    }

    /// What's wanted now, nearest first: replaces what was waiting.
    pub fn ask(&self, chunks: impl IntoIterator<Item = [i32; 2]>) {
        let (lock, heard) = &*self.asked;
        let mut asked = lock.lock().unwrap();
        asked.queue.clear();
        asked.queue.extend(chunks);
        heard.notify_all();
    }

    /// The changes made through the game's doorway, as they are now: every chunk taken up from here on is made with
    /// them. Says the count of tellings, which each chunk made carries (`Made::told`).
    pub fn changed(&self, changes: HashMap<[i32; 2], Vec<Change>>) -> u64 {
        let (lock, _) = &*self.asked;
        let mut asked = lock.lock().unwrap();
        asked.changes = std::sync::Arc::new(changes);
        asked.told += 1;
        asked.told
    }

    /// The chunks made since the last taking, as many as `most`.
    pub fn take(&self, most: usize) -> Result<Vec<Made>, String> {
        let mut taken = Vec::new();
        while taken.len() < most {
            match self.made.try_recv() {
                Ok(made) => taken.push(made?),
                Err(_) => break,
            }
        }
        Ok(taken)
    }
}

impl Drop for Lane {
    fn drop(&mut self) {
        let (lock, heard) = &*self.asked;
        lock.lock().unwrap().stop = true;
        heard.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doorway::Axis;
    use std::collections::HashSet;

    /// Every open side of every solid block of a chunk, and every cell of water's top, one at a time from the
    /// chunk's own cells: the count the merged faces have to cover, each once.
    fn open_sides(blocks: &Blocks) -> HashSet<([i32; 3], Facing, Block, bool)> {
        let mut open = HashSet::new();
        let unit = blocks.unit;
        for north in 0..CHUNK * unit {
            for east in 0..CHUNK * unit {
                for up in blocks.low * unit..blocks.high * unit {
                    let cell = blocks.cell(north, east, up);
                    if cell == 0 {
                        continue;
                    }
                    for facing in Facing::ALL {
                        let step = facing.step();
                        let beyond = blocks.cell(north + step[0], east + step[1], up + step[2]);
                        if Blocks::solid(cell) {
                            if !Blocks::solid(beyond) && !(facing == Facing::Down && up == 0) {
                                open.insert(([north, east, up], facing, cell - 1, false));
                            }
                        } else if facing == Facing::Up && beyond == 0 {
                            open.insert(([north, east, up], facing, cell - 1, true));
                        }
                    }
                }
            }
        }
        open
    }

    /// The cells a face covers, each with its facing, block and kind.
    fn covered(face: &Face) -> Vec<([i32; 3], Facing, Block, bool)> {
        let ([a, b], _) = face.facing.axes();
        let mut cells = Vec::new();
        for dv in 0..face.high {
            for du in 0..face.wide {
                let mut at = face.at;
                at[a] += du;
                at[b] += dv;
                cells.push((at, face.facing, face.block, face.water));
            }
        }
        cells
    }

    /// A chunk's merged faces cover every open side once and nothing else. How many sides, and how many faces.
    fn exact(blocks: &Blocks) -> (usize, usize) {
        let open = open_sides(blocks);
        let faces = blocks.faces();
        let mut seen = HashSet::new();
        for face in &faces {
            assert!(face.wide >= 1 && face.high >= 1, "a face has a size: {face:?}");
            for side in covered(face) {
                assert!(open.contains(&side), "a face covers a side that isn't open: {side:?} of {face:?} in chunk {:?}", blocks.chunk);
                assert!(seen.insert(side), "a side is covered twice: {side:?} in chunk {:?}", blocks.chunk);
            }
        }
        assert_eq!(seen.len(), open.len(), "every open side is covered, in chunk {:?}", blocks.chunk);
        (open.len(), faces.len())
    }

    fn chunk_of(column: [i32; 2]) -> [i32; 2] {
        [column[0].div_euclid(CHUNK), column[1].div_euclid(CHUNK)]
    }

    /// Bare ground: chunks round the spawn, the mountain top's and the sea's marks, and one at the map's corner (where
    /// columns past the map are air, and the world's edge shows its side).
    #[test]
    fn bare_ground_s_faces_are_every_open_side_once() {
        let doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let mut tiles = HashMap::new();
        let ([least_north, least_east], _) = doorway.grid();
        let mut report = Vec::new();
        for (name, column) in [("the spawn", [0, 0]), ("the mountain top", [-605, 78]), ("the plain", [-3975, -1192]), ("the map's corner", [least_north, least_east])] {
            let (mut sides, mut faces, mut chunks) = (0, 0, 0);
            for dn in -1..=1 {
                for de in -1..=1 {
                    let chunk = chunk_of(column);
                    let chunk = [chunk[0] + dn, chunk[1] + de];
                    let blocks = Blocks::read(&doorway, &mut tiles, chunk).unwrap();
                    // The cells are the doorway's own answers, column by column (the chunk's middle row and its edge).
                    for (north, east) in [(0, 0), (15, 16), (31, 31), (-1, 7), (32, 20)] {
                        let column = doorway.column(chunk[0] * CHUNK + north, chunk[1] * CHUNK + east).unwrap();
                        for up in blocks.low..blocks.high {
                            let block = match &column {
                                Column::Past => None,
                                Column::Here { runs, .. } => runs.iter().find(|run| run.from <= up && up < run.to).map(|run| run.block),
                            };
                            assert_eq!(blocks.cell(north, east, up), block.map_or(0, |block| block + 1), "{name}: the cell at {north}, {east}, {up} of chunk {chunk:?} is the doorway's");
                        }
                    }
                    let (s, f) = exact(&blocks);
                    sides += s;
                    faces += f;
                    chunks += 1;
                }
            }
            assert!(sides > 0, "{name} has ground");
            report.push(format!("{name}: {chunks} chunks, {sides} open sides in {faces} faces"));
        }
        eprintln!("Bare ground: {}", report.join("; "));
    }

    /// What's built: the scatter stage's things (a tower, the castle, a house, the hall in the mountain, a cave, a
    /// wood), each chunk its place lies in and those round it.
    #[test]
    fn the_stage_s_faces_are_every_open_side_once() {
        let doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        // Its places (each kind of thing: the caves are many, so every fifth of them), and a wood near the spawn.
        let mut places: Vec<[i32; 2]> = doorway.stage().expect("the stage is on").sites().iter().enumerate().filter(|(k, site)| !matches!(site.kind, crate::scatter_stage::Kind::Cave { .. }) || k % 5 == 0).map(|(_, site)| site.at).collect();
        places.push([192, 0]);
        let mut tiles = HashMap::new();
        let mut done = HashSet::new();
        let (mut sides, mut faces) = (0, 0);
        for at in places {
            let chunk = chunk_of(at);
            for dn in -1..=1 {
                for de in -1..=1 {
                    let chunk = [chunk[0] + dn, chunk[1] + de];
                    if done.insert(chunk) {
                        let (s, f) = exact(&Blocks::read(&doorway, &mut tiles, chunk).unwrap());
                        sides += s;
                        faces += f;
                    }
                }
            }
        }
        assert!(done.len() >= 9, "the stage has places");
        eprintln!("The stage: {} chunks, {sides} open sides in {faces} faces", done.len());
    }

    /// A block placed: the chunks it touches are made again and are what's made from scratch; no other chunk
    /// changes.
    #[test]
    fn a_placed_block_changes_only_the_chunks_it_touches() {
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let stone = doorway.block("incus:stone");
        let around: Vec<[i32; 2]> = (-1..=1).flat_map(|dn| (-1..=1).map(move |de| [dn, de])).collect();
        let before: Vec<Vec<Face>> = around.iter().map(|chunk| Blocks::read(&doorway, &mut HashMap::new(), *chunk).unwrap().faces()).collect();
        // On the edge of chunk (0, 0) toward the north, well over the ground: a block in the air.
        let ground = match doorway.column(CHUNK - 1, 5).unwrap() {
            Column::Here { ground, .. } => ground,
            Column::Past => panic!("the spawn's chunk is on the map"),
        };
        let cell = [CHUNK - 1, 5, ground + 12];
        doorway.make(Change::Block { cell, block: stone }).unwrap();
        let touched = touched(cell);
        assert_eq!(touched, vec![[0, 0], [1, 0]], "a cell on a chunk's north edge touches that chunk and the one north of it");
        for (chunk, before) in around.iter().zip(&before) {
            let blocks = Blocks::read(&doorway, &mut HashMap::new(), *chunk).unwrap();
            exact(&blocks);
            let after = blocks.faces();
            if *chunk == [0, 0] {
                // (Its six sides; and, the sky's light under it falling, the ground's faces there split from the rest.)
                assert!(after.len() >= before.len() + 6, "a block in the air shows six sides in its own chunk");
                assert_eq!(after.iter().filter(|face| face.at[2] >= cell[2] && face.block == stone).count(), 6, "the block's six sides");
            } else {
                assert_eq!(&after, before, "chunk {chunk:?} doesn't change (a block in the air hides no side of its neighbour's)");
            }
        }
    }

    /// Pieces: a quarter block (a bar) and an eighth in the air over the spawn's chunk, then a bar across the chunk's
    /// north edge. The chunk is meshed in half blocks; its merged faces are every open side of every octant once; a
    /// bar is six faces (two ends half by half, four sides one by a half) and an eighth six; a bar across two chunks
    /// is in both, each with its half; and a piece put where one already is leaves the first's octant as it was.
    #[test]
    fn pieces_are_meshed_in_half_blocks() {
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let (stone, sand) = (doorway.block("incus:stone"), doorway.block("incus:sand"));
        // (Over the highest ground of the chunk and the columns round it, so every piece here hangs in the air.)
        let ground = (-1..=CHUNK)
            .flat_map(|north| (-1..=CHUNK).map(move |east| (north, east)))
            .map(|(north, east)| match doorway.column(north, east).unwrap() {
                Column::Here { ground, .. } => ground,
                Column::Past => panic!("the spawn's chunk is on the map"),
            })
            .max()
            .unwrap();
        let before = Blocks::read(&doorway, &mut HashMap::new(), [0, 0]).unwrap();
        assert_eq!(before.unit, 1, "no piece: whole blocks");
        let (sides_before, _) = exact(&before);
        // A north–south bar well over the ground, in one cell; and an eighth beside it, apart.
        let z = ground + 12;
        doorway.make(Change::Bar { cell: [16, 16, z], from: [32, 33, z * 2], axis: Axis::NorthSouth, block: stone }).unwrap();
        doorway.make(Change::Oct { cell: [20, 16, z], from: [41, 32, z * 2 + 1], block: sand }).unwrap();
        let blocks = Blocks::read(&doorway, &mut HashMap::new(), [0, 0]).unwrap();
        assert_eq!(blocks.unit, 2, "a piece: half blocks");
        // What half blocks cost the lane, for the record: the same chunk read and meshed each way (its tiles kept).
        let mut tiles = HashMap::new();
        let mut timed = |doorway: &Doorway| {
            Blocks::read(doorway, &mut tiles, [0, 0]).unwrap();
            let began = std::time::Instant::now();
            let faces = (0..5).map(|_| Blocks::read(doorway, &mut tiles, [0, 0]).unwrap().faces().len()).last().unwrap();
            (began.elapsed().as_secs_f32() * 200.0, faces)
        };
        let plain = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let ((whole_ms, whole_faces), (half_ms, half_faces)) = (timed(&plain), timed(&doorway));
        eprintln!("The spawn's chunk read and meshed: in whole blocks {whole_ms:.1} ms, {whole_faces} faces; with two pieces in it, in half blocks, {half_ms:.1} ms, {half_faces} faces");
        let (sides, _) = exact(&blocks);
        // (Every whole block's side is four half sides; the bar shows 2 ends + 4 sides of 2 = 10, the eighth 6.)
        assert_eq!(sides, sides_before * 4 + 10 + 6, "the ground's sides in half blocks, and the pieces'");
        let faces = blocks.faces();
        // (Nothing else is that high: every face from there up is a piece's.)
        let of = |block: Block| faces.iter().filter(|face| face.block == block && face.at[2] >= z * 2).collect::<Vec<_>>();
        let bar = of(stone);
        assert_eq!(bar.len(), 6, "a bar in the air is six faces: {bar:?}");
        assert_eq!(bar.iter().filter(|face| face.wide * face.high == 1).count(), 2, "two ends, half by half");
        assert_eq!(bar.iter().filter(|face| face.wide * face.high == 2).count(), 4, "four sides, one by a half");
        assert_eq!(of(sand).len(), 6, "an eighth is six faces");
        // A second bar put across the first's north octant: the first keeps it, the second shows only its other half.
        doorway.make(Change::Bar { cell: [16, 16, z], from: [33, 33, z * 2], axis: Axis::Upright, block: sand }).unwrap();
        let blocks = Blocks::read(&doorway, &mut HashMap::new(), [0, 0]).unwrap();
        exact(&blocks);
        assert_eq!(blocks.cell(33, 33, z * 2), stone + 1, "the first bar keeps the octant they'd share");
        assert_eq!(blocks.cell(33, 33, z * 2 + 1), sand + 1, "the second fills its other octant");
        // A bar across the chunk's north edge (its second octant in the chunk to the north): both chunks have it.
        doorway.forget_changes();
        let edge = Change::Bar { cell: [CHUNK - 1, 5, z], from: [CHUNK * 2 - 1, 10, z * 2], axis: Axis::NorthSouth, block: stone };
        doorway.make(edge).unwrap();
        assert_eq!(touched_by(&edge), vec![[0, 0], [1, 0]], "a bar across a chunk's edge touches both chunks");
        let south = Blocks::read(&doorway, &mut HashMap::new(), [0, 0]).unwrap();
        let north = Blocks::read(&doorway, &mut HashMap::new(), [1, 0]).unwrap();
        exact(&south);
        exact(&north);
        assert_eq!(south.cell(CHUNK * 2 - 1, 10, z * 2), stone + 1, "its south half is the south chunk's");
        assert_eq!(north.cell(0, 10, z * 2), stone + 1, "its north half is the north chunk's");
        assert_eq!(south.cell(CHUNK * 2, 10, z * 2), stone + 1, "and each chunk knows the other's half beside its own (no face between them)");
        assert_eq!(Blocks::read(&doorway, &mut HashMap::new(), [0, 1]).unwrap().unit, 1, "a chunk the bar doesn't touch stays in whole blocks");
    }

    /// Counts for choosing how light is kept (2026-11-17), only if asked (`NEAR_TEST_COUNT`: a radius in chunks;
    /// `NEAR_TEST_AT`: north, east; `NEAR_TEST_STAGE=1`: with the scatter stage): over every chunk of the radius, the
    /// merged faces, the open sides they cover (one per block side: what a light value a side would take), the sides
    /// not in plain daylight (sky light under 15), the air cells those faces look into and those not in plain
    /// daylight; and of the faces in a view 100 degrees across facing north from a player's eye there, those turned
    /// away from the eye, with the sides they cover. Run it with `INCUS_NEAR_LIGHT=0` too for the faces merged
    /// whatever the light.
    #[test]
    fn counts_for_keeping_light() {
        let Some(radius) = std::env::var("NEAR_TEST_COUNT").ok().and_then(|r| r.parse::<f32>().ok()) else { return };
        let staged = std::env::var("NEAR_TEST_STAGE").as_deref() == Ok("1");
        let at: Vec<f32> = std::env::var("NEAR_TEST_AT").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        let place = if at.len() == 2 { [at[0], at[1]] } else { [0.5, 0.5] };
        let doorway = Doorway::open_with(&crate::doorway::test_world(), staged).unwrap();
        let column = doorway.column(place[0].floor() as i32, place[1].floor() as i32).unwrap();
        let eye = [place[0], place[1], doorway.surface(&column).unwrap() as f32 + crate::marks::PLAYER_EYE];
        let chunks = wanted(place, radius);
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(2).max(1);
        let next = std::sync::atomic::AtomicUsize::new(0);
        // faces, sides, sides shaded, air cells, air cells shaded; in view: faces, faces turned away, their sides, and
        // the faces in groups the frame's listing leaves out (`groups_shown`); the light kept: chunks with any, its
        // bricks, its words; and what bricks of 4 a side would be.
        let totals = std::sync::Mutex::new([0u64; 13]);
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    let doorway = Doorway::open_with(&crate::doorway::test_world(), staged).unwrap();
                    let mut tiles = HashMap::new();
                    let mut mine = [0u64; 13];
                    loop {
                        let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&chunk) = chunks.get(k) else { break };
                        if tiles.len() > 6 {
                            tiles.clear();
                        }
                        let blocks = Blocks::read(&doorway, &mut tiles, chunk).unwrap();
                        let lights = blocks.sky_light();
                        let u = blocks.unit;
                        let tall = ((blocks.high - blocks.low) * u) as usize;
                        let light = |n: i32, e: i32, z: i32| -> u8 {
                            if z >= blocks.high * u {
                                return 15;
                            }
                            if z < blocks.low * u {
                                return 0;
                            }
                            lights[((n + MARGIN * u) as usize + (e + MARGIN * u) as usize * WIDE * u as usize) * tall + (z - blocks.low * u) as usize]
                        };
                        let mut air = HashSet::new();
                        // Is the chunk in the view (its middle within 50 degrees either side of north)?
                        let middle = [(chunk[0] as f32 + 0.5) * CHUNK as f32 - eye[0], (chunk[1] as f32 + 0.5) * CHUNK as f32 - eye[1]];
                        let in_view = middle[0] > 0.0 && middle[1].abs() <= middle[0] * 50f32.to_radians().tan() + CHUNK as f32;
                        if let Some(block) = blocks.light_block(&lights, None) {
                            mine[9] += 1;
                            mine[10] += ((block.words.len() - block.layers as usize * 32) / LIGHT_BRICK_WORDS) as u64;
                            mine[11] += block.words.len() as u64;
                            let mut small = HashSet::new();
                            for east in 0..CHUNK {
                                for north in 0..CHUNK {
                                    for up in blocks.low..blocks.high {
                                        if blocks.light_of(&lights, north, east, up).is_some_and(|light| light < 15) {
                                            small.insert([north >> 2, east >> 2, up >> 2]);
                                        }
                                    }
                                }
                            }
                            mine[12] += small.len() as u64;
                        }
                        let (faces, _, _, low, high) = grouped(&blocks);
                        let shown = groups_shown(chunk, low, high, eye);
                        for face in faces {
                            if in_view && !face.water && !shown[GROUPS.iter().position(|g| *g == face.facing).unwrap()] {
                                mine[8] += 1;
                            }
                            let ([a, b], along) = face.facing.axes();
                            let step = face.facing.step();
                            let sides = (face.wide * face.high) as u64;
                            mine[0] += 1;
                            mine[1] += sides;
                            for dv in 0..face.high {
                                for du in 0..face.wide {
                                    let mut cell = face.at;
                                    cell[a] += du;
                                    cell[b] += dv;
                                    let beyond = [cell[0] + step[0], cell[1] + step[1], cell[2] + step[2]];
                                    let l = light(beyond[0], beyond[1], beyond[2]);
                                    if l < 15 {
                                        mine[2] += 1;
                                    }
                                    if air.insert(beyond) {
                                        mine[3] += 1;
                                        if l < 15 {
                                            mine[4] += 1;
                                        }
                                    }
                                }
                            }
                            if in_view {
                                mine[5] += 1;
                                // The face's plane, in blocks from the world's origin, and whether the eye is on the
                                // side it looks out of.
                                let greater = matches!(face.facing, Facing::North | Facing::East | Facing::Up);
                                let origin = [chunk[0] * CHUNK, chunk[1] * CHUNK, 0];
                                let plane = (origin[along] as f32 * u as f32 + (face.at[along] + greater as i32) as f32) / u as f32;
                                let sign = step[along] as f32;
                                if (eye[along] - plane) * sign <= 0.0 {
                                    mine[6] += 1;
                                    mine[7] += sides;
                                }
                            }
                        }
                    }
                    let mut totals = totals.lock().unwrap();
                    for k in 0..13 {
                        totals[k] += mine[k];
                    }
                });
            }
        });
        let t = totals.into_inner().unwrap();
        eprintln!(
            "Light counts, radius {radius} round {place:?}{} (light {}): {} chunks; {} faces covering {} sides ({:.1} a face), {} sides not in plain daylight ({:.1}%); {} air cells the faces look into, {} not in plain daylight ({:.1}%); in a view north, {} faces, {} turned away ({:.1}%), covering {} sides; {} in groups left out ({:.1}%); the light kept: {} chunks have some, {} bricks, {:.2} MB (bricks of 4 a side would be {}: {:.2} MB at 4 bits a cell)",
            if staged { ", the stage on" } else { "" },
            if light_on() { "on" } else { "off" },
            chunks.len(),
            t[0],
            t[1],
            t[1] as f64 / t[0] as f64,
            t[2],
            100.0 * t[2] as f64 / t[1] as f64,
            t[3],
            t[4],
            100.0 * t[4] as f64 / t[3] as f64,
            t[5],
            t[6],
            100.0 * t[6] as f64 / t[5] as f64,
            t[7],
            t[8],
            100.0 * t[8] as f64 / t[5] as f64,
            t[9],
            t[10],
            t[11] as f64 * 4.0 / 1048576.0,
            t[12],
            t[12] as f64 * 32.0 / 1048576.0
        );
    }

    /// The lane makes every chunk of a radius, each the mesh made directly; and what a radius comes to (`NEAR_TEST_RADII`:
    /// radii in chunks, for the numbers; `NEAR_TEST_AT`: north, east; `NEAR_TEST_STAGE=1`: with the scatter stage).
    #[test]
    fn the_lane_fills_a_radius() {
        let folder = crate::doorway::test_world();
        let staged = std::env::var("NEAR_TEST_STAGE").as_deref() == Ok("1");
        let at: Vec<f32> = std::env::var("NEAR_TEST_AT").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        let eye = if at.len() == 2 { [at[0], at[1]] } else { [0.5, 0.5] };
        let radii: Vec<f32> = std::env::var("NEAR_TEST_RADII").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_else(|| vec![2.0]);
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(1).max(1);
        for radius in radii {
            let lane = Lane::start(folder.clone(), staged, threads).unwrap();
            let want = wanted(eye, radius);
            // Only chunks in reach are wanted, and every chunk in reach is.
            assert!(want.iter().all(|chunk| nearest_part(*chunk, eye) <= radius * CHUNK as f32), "only chunks in reach are wanted");
            let began = std::time::Instant::now();
            lane.ask(want.iter().copied());
            let mut made: HashMap<[i32; 2], Made> = HashMap::new();
            while made.len() < want.len() {
                for chunk in lane.take(4096).unwrap() {
                    made.insert(chunk.chunk, chunk);
                }
                assert!(began.elapsed().as_secs() < 900, "the lane fills radius {radius} in a quarter of an hour");
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let seconds = began.elapsed().as_secs_f32();
            let faces: usize = made.values().map(|chunk| chunk.faces.len()).sum();
            let work: f32 = made.values().map(|chunk| chunk.ms).sum();
            eprintln!(
                "The near lane, radius {radius} chunks ({:.0} blocks) round {eye:?}{}: {} chunks, {faces} faces ({} triangles), {:.1} MB of faces; filled in {seconds:.2} s on {} threads, {:.2} ms a chunk",
                radius * CHUNK as f32,
                if staged { ", the stage on" } else { "" },
                made.len(),
                faces * 2,
                faces as f32 * 12.0 / 1048576.0,
                lane.threads,
                work / made.len() as f32
            );
            // A few of them against the mesh made directly, and their heights.
            let doorway = Doorway::open_with(&folder, staged).unwrap();
            let mut tiles = HashMap::new();
            for chunk in want.iter().step_by((want.len() / 7).max(1)) {
                let blocks = Blocks::read(&doorway, &mut tiles, *chunk).unwrap();
                let (direct, groups, water, _, _) = grouped(&blocks);
                let direct: Vec<[u32; 3]> = direct.iter().map(|face| packed(face, *chunk, blocks.unit)).collect();
                assert_eq!(made[chunk].faces, direct, "chunk {chunk:?} from the lane is the mesh made directly, in its groups, its water's tops last");
                assert_eq!(made[chunk].water, water, "and it says how many of them are water's");
                assert_eq!(made[chunk].groups, groups, "and how many each group has");
                assert!(made[chunk].faces.is_empty() || made[chunk].low < made[chunk].high, "a chunk with faces has a height");
            }
        }
    }

    /// The lane told of a block placed hands its chunk over with it, as the mesh made directly; told it's taken back,
    /// as it was before.
    #[test]
    fn the_lane_hears_of_changes() {
        let folder = crate::doorway::test_world();
        let mut doorway = Doorway::open_with(&folder, false).unwrap();
        let stone = doorway.block("incus:stone");
        let lane = Lane::start(folder.clone(), false, 2).unwrap();
        let from_lane = |told: u64| {
            lane.ask([[0, 0]]);
            let began = std::time::Instant::now();
            loop {
                if let Some(made) = lane.take(16).unwrap().into_iter().find(|made| made.told == told) {
                    assert_eq!(made.chunk, [0, 0]);
                    break made.faces;
                }
                assert!(began.elapsed().as_secs() < 60, "the lane hands the chunk over, made from telling {told}");
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        };
        let direct = |doorway: &Doorway| {
            let (faces, ..) = grouped(&Blocks::read(doorway, &mut HashMap::new(), [0, 0]).unwrap());
            faces.iter().map(|face| packed(face, [0, 0], 1)).collect::<Vec<_>>()
        };
        let before = from_lane(0);
        assert_eq!(before, direct(&doorway), "before any change, the mesh made directly");
        // A block in the air over the chunk's middle.
        let ground = match doorway.column(16, 16).unwrap() {
            Column::Here { ground, .. } => ground,
            Column::Past => panic!("the spawn's chunk is on the map"),
        };
        doorway.make(Change::Block { cell: [16, 16, ground + 12], block: stone }).unwrap();
        assert_eq!(lane.changed(doorway.changes().clone()), 1);
        let with = from_lane(1);
        assert_eq!(with.len(), before.len() + 6, "a block in the air shows six sides");
        assert_eq!(with, direct(&doorway), "told of the block, the mesh made directly with it");
        doorway.forget_changes();
        assert_eq!(lane.changed(doorway.changes().clone()), 2);
        assert_eq!(from_lane(2), before, "told it's taken back, as it was before");
    }

    /// The sky's light (build 208): open ground is 15; a sealed room black; a tunnel into a hillside lit from its mouth
    /// falls to black past about its width in, and a tunnel twice as wide reaches about twice as far; shade under a slab
    /// outdoors is never under the floor.
    #[test]
    fn the_sky_s_light_reaches_in_as_far_as_an_opening_is_wide() {
        // A made-up chunk: solid stone to `ground`, a hill over the middle with tunnels into it from the south.
        let (ground, top) = (10, 40);
        let blocks_with = |carve: &dyn Fn(i32, i32, i32) -> bool| {
            let (low, high) = (ground - 1, top + 2);
            let tall = (high - low) as usize;
            let mut cells = vec![0u16; WIDE * WIDE * tall];
            for e in 0..WIDE as i32 {
                for n in 0..WIDE as i32 {
                    let (north, east) = (n - MARGIN, e - MARGIN);
                    for up in low..high {
                        let hill = (4..28).contains(&north) && up < top;
                        if (up < ground || hill) && !carve(north, east, up) {
                            cells[(n as usize + e as usize * WIDE) * tall + (up - low) as usize] = crate::doorway::STONE + 1;
                        }
                    }
                }
            }
            Blocks { chunk: [0, 0], low, high, unit: 1, margin: MARGIN, cells }
        };
        let at = |blocks: &Blocks, light: &[u8], north: i32, east: i32, up: i32| {
            let tall = (blocks.high - blocks.low) as usize;
            light[((north + MARGIN) as usize + (east + MARGIN) as usize * WIDE) * tall + (up - blocks.low) as usize]
        };
        // How far into a tunnel `w` wide and high, from the hill's south face, the light reaches.
        let reach = |w: i32| {
            let blocks = blocks_with(&|north, east, up| north >= 4 && (16 - w / 2..16 - w / 2 + w).contains(&east) && (ground..ground + w).contains(&up));
            let light = blocks.sky_light();
            assert_eq!(at(&blocks, &light, 0, 16, ground), 15, "open ground in front of the hill is fully lit");
            (4..28).take_while(|&north| at(&blocks, &light, north, 16, ground) > 0).count() as i32
        };
        let (narrow, wide) = (reach(2), reach(4));
        eprintln!("The sky's light reaches {narrow} blocks into a tunnel 2 wide, {wide} into one 4 wide");
        assert!((1..=4).contains(&narrow), "light reaches into a tunnel about as far as it's wide: {narrow} for 2");
        assert!(wide > narrow && wide <= 8, "twice as wide, about twice as far: {wide} for 4, {narrow} for 2");
        // A sealed room inside the hill: black.
        let blocks = blocks_with(&|north, east, up| (10..14).contains(&north) && (10..14).contains(&east) && (20..24).contains(&up));
        let light = blocks.sky_light();
        assert_eq!(at(&blocks, &light, 12, 12, 22), 0, "a sealed room is black");
        // Shade under a roof outdoors (8 by 8, 3 over open ground, open on every side): dim, never under the floor.
        let blocks = blocks_with(&|north, east, up| (4..28).contains(&north) && up >= ground && !((12..20).contains(&north) && (12..20).contains(&east) && up == ground + 3));
        let light = blocks.sky_light();
        let under = at(&blocks, &light, 16, 16, ground);
        assert!(under >= LIGHT_FLOOR, "outdoors under a roof is never darker than the floor: {under}");
    }

    /// The light kept (build 210): for chunks of the stage (the town, a wood, the cave's), every cell any face looks
    /// into inside the chunk, looked up as the card looks it up, is the sky's light worked out, exactly; some of it
    /// is shade; and a chunk of bare land hands over no light at all.
    #[test]
    fn the_light_kept_is_the_sky_s_light() {
        let doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        let mut tiles = HashMap::new();
        let (mut looked, mut shaded, mut words) = (0u64, 0u64, 0usize);
        for chunk in [[-4, -12], [12, 8], [14, 4], [15, 4], [16, 4]] {
            let blocks = Blocks::read(&doorway, &mut tiles, chunk).unwrap();
            let lights = blocks.sky_light();
            let block = blocks.light_block(&lights, None);
            words += block.as_ref().map_or(0, |block| block.words.len());
            let u = blocks.unit;
            for face in blocks.faces() {
                let ([a, b], _) = face.facing.axes();
                let step = face.facing.step();
                for dv in 0..face.high {
                    for du in 0..face.wide {
                        let mut cell = face.at;
                        cell[a] += du;
                        cell[b] += dv;
                        let into: [i32; 3] = std::array::from_fn(|k| (cell[k] + step[k]).div_euclid(u));
                        if into[0] < 0 || into[1] < 0 || into[0] >= CHUNK || into[1] >= CHUNK {
                            continue;
                        }
                        let worked = blocks.light_of(&lights, into[0], into[1], into[2]).expect("a face looks into a cell that isn't solid");
                        let kept = block.as_ref().map_or(15, |block| block.at(into));
                        assert_eq!(kept, worked, "the light kept in {into:?} of chunk {chunk:?}");
                        looked += 1;
                        shaded += (worked < 15) as u64;
                    }
                }
            }
        }
        eprintln!("The light kept: {looked} cells looked into over five chunks of the stage, {shaded} of them not in plain daylight, in {} KB", words * 4 / 1024);
        // What each part of a chunk's making takes (the five chunks, twenty times over; one thread).
        let chunks: Vec<Blocks> = [[-4, -12], [12, 8], [14, 4], [15, 4], [16, 4]].iter().map(|chunk| Blocks::read(&doorway, &mut tiles, *chunk).unwrap()).collect();
        let timed = |work: &dyn Fn(&Blocks)| {
            let began = std::time::Instant::now();
            for _ in 0..20 {
                chunks.iter().for_each(work);
            }
            began.elapsed().as_secs_f64() * 1000.0 / 100.0
        };
        let lights: Vec<Vec<u8>> = chunks.iter().map(Blocks::sky_light).collect();
        let (faces, sky) = (timed(&|blocks| drop(blocks.faces())), timed(&|blocks| drop(blocks.sky_light())));
        let began = std::time::Instant::now();
        for _ in 0..20 {
            for (blocks, lights) in chunks.iter().zip(&lights) {
                drop(blocks.light_block(lights, None));
            }
        }
        eprintln!("A chunk of these, ms: its faces {faces:.2}, its sky light {sky:.2}, its light put in bricks {:.2}", began.elapsed().as_secs_f64() * 1000.0 / 100.0);
        assert!(shaded > 1000, "the stage's town, wood and cave have shade: {shaded}");
        let bare = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let blocks = Blocks::read(&bare, &mut HashMap::new(), [0, 0]).unwrap();
        assert_eq!(blocks.light_block(&blocks.sky_light(), None), None, "bare land is all in plain daylight: no light kept");
    }

    /// Faces turned away (build 209): a chunk's faces come in the groups' order, their counts adding up; and for eyes
    /// inside, beside, over and under chunks of the stage (its town, a wood, the spawn), every face of a group left
    /// out has the eye behind it.
    #[test]
    fn groups_left_out_are_turned_away() {
        let doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        let mut tiles = HashMap::new();
        let (mut out, mut all) = (0u64, 0u64);
        for chunk in [[0, 0], [-4, -12], [12, 8], [14, 4]] {
            let blocks = Blocks::read(&doorway, &mut tiles, chunk).unwrap();
            let (faces, groups, water, low, high) = grouped(&blocks);
            assert_eq!(groups.iter().sum::<u32>() + water, faces.len() as u32, "the groups and the water's tops are every face");
            let mut k = 0;
            for (g, &count) in groups.iter().enumerate() {
                assert!(faces[k..k + count as usize].iter().all(|face| face.facing == GROUPS[g] && !face.water), "group {g} holds only faces that look its way");
                k += count as usize;
            }
            assert!(faces[k..].iter().all(|face| face.water), "the water's tops come last");
            let (u, origin) = (blocks.unit, [chunk[0] * CHUNK, chunk[1] * CHUNK, 0]);
            let middle = [(chunk[0] as f32 + 0.5) * CHUNK as f32, (chunk[1] as f32 + 0.5) * CHUNK as f32];
            for dn in [-40.0, -16.0, 0.0, 16.0, 40.0] {
                for de in [-40.0, -16.0, 0.3, 16.0, 40.0] {
                    for up in [low as f32 - 5.0, low as f32, (low + high) as f32 * 0.5 + 0.4, high as f32, high as f32 + 5.0] {
                        let eye = [middle[0] + dn, middle[1] + de, up];
                        let shown = groups_shown(chunk, low, high, eye);
                        for face in faces.iter().filter(|face| !face.water) {
                            all += 1;
                            let g = GROUPS.iter().position(|g| *g == face.facing).unwrap();
                            if shown[g] {
                                continue;
                            }
                            out += 1;
                            let (_, along) = face.facing.axes();
                            let greater = matches!(face.facing, Facing::North | Facing::East | Facing::Up);
                            let plane = (origin[along] as f32 * u as f32 + (face.at[along] + greater as i32) as f32) / u as f32;
                            assert!((eye[along] - plane) * face.facing.step()[along] as f32 <= 0.0, "a face left out is turned away: {face:?} of chunk {chunk:?}, the eye at {eye:?}");
                        }
                    }
                }
            }
        }
        eprintln!("Groups left out: {out} of {all} faces over the eyes tried, every one turned away");
        assert!(out > all / 4, "the rule leaves out a good share");
    }

    /// A face's three words hold what it is.
    #[test]
    fn a_packed_face_unpacks() {
        let face = Face { at: [31, 7, 1000], facing: Facing::West, wide: 32, high: 1024, block: 513, water: false, light: 9 };
        let [a, b, c] = packed(&face, [-1125, 990], 1);
        assert_eq!([a & 63, (a >> 6) & 63, (a >> 12) & 1023, (a >> 22) & 7, ((a >> 25) & 31) + 1, (a >> 30) & 1, a >> 31], [31, 7, 1000, 3, 32, 0, 0]);
        assert_eq!([(b & 1023) + 1, (b >> 10) & 0xFFFF, (b >> 26) & 1, b >> 27], [1024, 513, 0, 9]);
        assert_eq!([(c & 0xFFFF) as i32 - 32768, (c >> 16) as i32 - 32768], [-1125, 990]);
        // A face in half blocks: north and east to 63, up to 2,047 (its eleventh bit in the second word).
        let half = Face { at: [63, 40, 2047], facing: Facing::Up, wide: 2, high: 1, block: 7, water: false, light: 15 };
        let [a, b, _] = packed(&half, [3, -4], 2);
        assert_eq!([a & 63, (a >> 6) & 63, ((a >> 12) & 1023) | ((b >> 26) & 1) << 10, (a >> 22) & 7, ((a >> 25) & 31) + 1, a >> 31], [63, 40, 2047, 4, 2, 1]);
        assert_eq!([(b & 1023) + 1, (b >> 10) & 0xFFFF], [1, 7]);
    }

    /// Placed light (build 216): a step dimmer each block from a lamp, round a wall's end and not through it, two
    /// colours mixing where they meet; and what's kept for the card reads back as it was spread.
    #[test]
    fn placed_light_falls_a_step_a_block_and_walls_stop_it() {
        // A made-up chunk read 16 columns out: a stone floor; a red lamp on it at north 10, east 10 and a green one
        // at 20, 20; and a wall along east 13 from the south edge to north 12, to the top of what's kept.
        let (ground, margin) = (10, LAMP_MARGIN);
        let (low, high) = (ground - 1, ground + 20);
        let wide = (CHUNK + 2 * margin) as usize;
        let tall = (high - low) as usize;
        let (red, green): (Block, Block) = (7, 8);
        let mut cells = vec![0u16; wide * wide * tall];
        let mut set = |north: i32, east: i32, up: i32, block: Block| cells[((north + margin) as usize + (east + margin) as usize * wide) * tall + (up - low) as usize] = block + 1;
        for east in -margin..CHUNK + margin {
            for north in -margin..CHUNK + margin {
                set(north, east, ground - 1, crate::doorway::STONE);
                if east == 13 && north <= 12 {
                    for up in ground..high {
                        set(north, east, up, crate::doorway::STONE);
                    }
                }
            }
        }
        set(10, 10, ground, red);
        set(20, 20, ground, green);
        let blocks = Blocks { chunk: [0, 0], low, high, unit: 1, margin, cells };
        let mut glows = Glows { blocks: vec![[0; 3]; 9] };
        glows.blocks[red as usize] = [15, 0, 0];
        glows.blocks[green as usize] = [0, 15, 0];
        let placed = blocks.placed_light(&[[10, 10, ground], [20, 20, ground]], &glows).expect("the lamps light the chunk");
        let at = |north: i32, east: i32, up: i32| {
            let colour = placed.at(north, east, up);
            [(colour & 15) as u8, (colour >> 4 & 15) as u8, (colour >> 8 & 15) as u8]
        };
        for d in 1..=14 {
            assert_eq!(at(10 + d, 10, ground)[0], 15 - d as u8, "{d} blocks north of the red lamp");
        }
        assert_eq!(at(25, 10, ground)[0], 0, "15 blocks off, no light");
        assert_eq!(at(10, 10, ground + 3)[0], 12, "three blocks over the lamp");
        assert_eq!(at(10, 10, ground), [0; 3], "the lamp's own cell is a block: none kept");
        // Behind the wall, 4 blocks off through it: the light goes round its end (3 north, 4 east, 3 back south).
        assert_eq!(at(10, 14, ground)[0], 5, "round the wall's end, not through it");
        // Ten steps from each lamp: both colours.
        assert_eq!(at(15, 15, ground), [5, 5, 0], "red and green mix");
        // As the card keeps it.
        let block = blocks.light_block(&blocks.sky_light(), Some(&placed)).expect("light kept");
        for up in low..high {
            for east in 0..CHUNK {
                for north in 0..CHUNK {
                    assert_eq!(block.placed_at([north, east, up]), at(north, east, up), "at {north} {east} {up}");
                }
            }
        }
        assert_eq!(block.at([5, 5, ground]), 15, "open ground is still in plain daylight");
        // With no lamp near, none; and a lamp that's been built over gives none.
        assert_eq!(blocks.placed_light(&[[10, 10, ground + 1]], &glows), None, "air where a lamp was listed: no light");
    }

    /// Counts for placed light (build 216), asked for by `NEAR_TEST_LAMPS` (a radius in chunks; `NEAR_TEST_AT` as the
    /// other counts), the scatter stage on: the chunks a lamp's light reaches, their colour bricks and words, and a
    /// chunk's making time (its faces and its light) with a lamp near and without.
    #[test]
    fn counts_for_placed_light() {
        let Some(radius) = std::env::var("NEAR_TEST_LAMPS").ok().and_then(|v| v.parse::<f32>().ok()) else { return };
        let at: Vec<f32> = std::env::var("NEAR_TEST_AT").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        let place = if at.len() == 2 { [at[0], at[1]] } else { [0.5, 0.5] };
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        let glows = Glows::load(&mut doorway).unwrap();
        let (mut tiles, mut lamps) = (HashMap::new(), HashMap::new());
        let (mut lit_chunks, mut plain, mut lit_ms, mut plain_ms, mut bricks, mut words, mut all_words) = (0u32, 0u32, 0f64, 0f64, 0usize, 0usize, 0usize);
        for chunk in wanted(place, radius) {
            if tiles.len() > 6 {
                tiles.clear();
                lamps.clear();
            }
            let began = std::time::Instant::now();
            let blocks = Blocks::read(&doorway, &mut tiles, chunk).unwrap();
            drop(grouped(&blocks));
            let sky = blocks.sky_light();
            let placed = placed_for(&doorway, &mut tiles, &mut lamps, &glows, chunk).unwrap();
            let block = blocks.light_block(&sky, placed.as_ref());
            let ms = began.elapsed().as_secs_f64() * 1000.0;
            if placed.is_some() {
                lit_chunks += 1;
                lit_ms += ms;
            } else {
                plain += 1;
                plain_ms += ms;
            }
            if let Some(block) = block {
                let layers = block.layers as usize;
                let tinted = block.words[layers * 16..layers * 32].iter().filter(|brick| **brick != NO_BRICK).count();
                bricks += tinted;
                words += tinted * GLOW_BRICK_WORDS;
                all_words += block.words.len();
            }
        }
        eprintln!(
            "Placed light within {radius} chunks of {place:?}: {lit_chunks} chunks a lamp lights of {}, {bricks} colour bricks ({:.2} MB of the light's {:.2}); a chunk made in {:.1} ms with a lamp near, {:.1} ms without (one thread)",
            lit_chunks + plain,
            words as f64 * 4.0 / 1048576.0,
            all_words as f64 * 4.0 / 1048576.0,
            lit_ms / lit_chunks.max(1) as f64,
            plain_ms / plain.max(1) as f64
        );
    }
}

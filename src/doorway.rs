//! The doorway: the one piece that answers "what's here?", a tile at a time (plan 2.1). The engine never opens a world
//! file itself; everything it knows of the world comes through here.
//!
//! Behind it now: an export, a world written out as columns by old Incus Artus (`exports/<name>/`, its `README.txt` the
//! rule). Read-only. The tablet takes the export's place at 3.1.
//!
//! What it answers today: each column's ground, its river's or lake's water, and its blocks from bedrock up as runs;
//! and what changed, a tile at a time (a mark's changes: David's build, the scatter). Its other answers (pieces,
//! objects and lights; named fields over the land) come with what first feeds them. Going to a mark asks it for the
//! ground under the eye; the far cells' lane will ask for tiles (2.2).

use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A block, by its place in the world's list of blocks (`Doorway::block_name`): stone first, then water.
pub type Block = u16;
pub const STONE: Block = 0;
pub const WATER: Block = 1;

/// A run of one block in a column, from `from` up to `to` (not including it).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Run {
    pub block: Block,
    pub from: i32,
    pub to: i32,
}

/// Which way a bar lies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    NorthSouth,
    EastWest,
    Upright,
}

/// A change made through the doorway: a whole block placed in a cell, or a bar of a block (half a block across, a whole
/// block long) from its corner, in half blocks, along an axis, in a cell. As old's event log tells its placings.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    Block { cell: [i32; 3], block: Block },
    Bar { cell: [i32; 3], from: [i32; 3], axis: Axis, block: Block },
}

impl Change {
    pub fn cell(&self) -> [i32; 3] {
        match self {
            Change::Block { cell, .. } | Change::Bar { cell, .. } => *cell,
        }
    }
}

/// A column: past the map, or its ground (the top block's Z), its river's or lake's water top (none for the sea alone),
/// and its blocks from bedrock up, air above the last run.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Column {
    Past,
    Here { ground: i32, water: Option<i32>, runs: Vec<Run> },
}

/// The test world: old's export of "nice25 test".
pub fn test_world() -> PathBuf {
    crate::session::project_folder().join("exports").join("nice25 test")
}

/// `world.toml`, as the export writes it.
#[derive(Deserialize)]
struct WorldFile {
    least: [i32; 2],
    tile: u32,
    tiles: u32,
    sea_level: i32,
    world_height: i32,
    stone: String,
    water: String,
    profile: Vec<ProfileFile>,
}

#[derive(Deserialize)]
struct ProfileFile {
    number: u32,
    top: String,
    under: Vec<(String, i32)>,
}

/// A profile: the top block, and each block under it with the depth below the ground it reaches, deepest last.
struct Profile {
    top: Block,
    under: Vec<(Block, i32)>,
}

/// The doorway, open on an export.
pub struct Doorway {
    folder: PathBuf,
    /// The least corner's column, north and east.
    least: [i32; 2],
    /// Columns a side of a tile, and tiles a side of the world.
    side: i32,
    tiles: i32,
    pub sea_level: i32,
    world_height: i32,
    blocks: Vec<String>,
    /// The profiles, by number.
    profiles: Vec<Option<Profile>>,
    /// What changed, by tile.
    changes: HashMap<[i32; 2], Vec<Change>>,
    /// How long each tile asked for took to answer (ms), since they were last taken.
    times: RefCell<Vec<f32>>,
}

/// A tile's columns as the export holds them, north fastest (column i north, j east at i + j × side): the ground (−1
/// past the map), the water (0 none) and the profile's number. Each column's runs are made from these when asked.
pub struct Tile {
    side: usize,
    grounds: Vec<i32>,
    waters: Vec<i32>,
    profiles: Vec<u32>,
}

impl Doorway {
    /// The doorway, opened on the export in `folder` (its `world.toml` read; its tiles when asked for).
    pub fn open(folder: &Path) -> Result<Self, String> {
        let path = folder.join("world.toml");
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let world: WorldFile = toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut blocks = vec![world.stone.clone(), world.water.clone()];
        let mut block = |name: &str| -> Block {
            match blocks.iter().position(|known| known == name) {
                Some(k) => k as Block,
                None => {
                    blocks.push(name.to_string());
                    (blocks.len() - 1) as Block
                }
            }
        };
        let mut profiles: Vec<Option<Profile>> = Vec::new();
        for profile in &world.profile {
            let number = profile.number as usize;
            if profiles.len() <= number {
                profiles.resize_with(number + 1, || None);
            }
            profiles[number] = Some(Profile { top: block(&profile.top), under: profile.under.iter().map(|(name, depth)| (block(name), *depth)).collect() });
        }
        Ok(Self {
            folder: folder.to_path_buf(),
            least: world.least,
            side: world.tile as i32,
            tiles: world.tiles as i32,
            sea_level: world.sea_level,
            world_height: world.world_height,
            blocks,
            profiles,
            changes: HashMap::new(),
            times: RefCell::new(Vec::new()),
        })
    }

    /// A block by its name, added to the world's list if it isn't there yet (a change can bring blocks the export
    /// doesn't use).
    pub fn block(&mut self, name: &str) -> Block {
        match self.blocks.iter().position(|known| known == name) {
            Some(k) => k as Block,
            None => {
                self.blocks.push(name.to_string());
                (self.blocks.len() - 1) as Block
            }
        }
    }

    /// A change made, kept with its tile. Refused if its cell is past the map or outside the world's height.
    pub fn make(&mut self, change: Change) -> Result<(), String> {
        let [north, east, z] = change.cell();
        let (tile, _) = self.place(north, east);
        if tile[0] < 0 || tile[1] < 0 || tile[0] >= self.tiles || tile[1] >= self.tiles || z < 0 || z >= self.world_height {
            return Err(format!("{change:?} is outside the world"));
        }
        self.changes.entry(tile).or_default().push(change);
        Ok(())
    }

    /// Every change forgotten (a mark's changes are taken back when the flight moves on).
    pub fn forget_changes(&mut self) {
        self.changes.clear();
    }

    /// What changed in a tile, in the order it was made. Asked for by the near eye's change path (2.3); today by the tests.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn changes_in(&self, tx: i32, ty: i32) -> &[Change] {
        self.changes.get(&[tx, ty]).map_or(&[], Vec::as_slice)
    }

    /// How long each tile asked for since the last call took to answer (ms).
    pub fn take_times(&self) -> Vec<f32> {
        std::mem::take(&mut *self.times.borrow_mut())
    }

    /// A block's name ("incus:stone"). Asked for by the eye (2.2, 2.3); today by the tests.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn block_name(&self, block: Block) -> &str {
        self.blocks.get(block as usize).map_or("unknown", String::as_str)
    }

    /// The tile a column is in (north and east of the least corner, in tiles), and the column's place in it.
    pub fn place(&self, north: i32, east: i32) -> ([i32; 2], [usize; 2]) {
        let (n, e) = (north - self.least[0], east - self.least[1]);
        let tile = [n.div_euclid(self.side), e.div_euclid(self.side)];
        (tile, [n.rem_euclid(self.side) as usize, e.rem_euclid(self.side) as usize])
    }

    /// The tile `tx` tiles north and `ty` east of the least corner. Past the map (outside it, or a tile with no file:
    /// the export writes none where no column is on the map), every column is past.
    pub fn tile(&self, tx: i32, ty: i32) -> Result<Tile, String> {
        let started = std::time::Instant::now();
        let tile = self.read_tile(tx, ty);
        self.times.borrow_mut().push(started.elapsed().as_secs_f32() * 1000.0);
        tile
    }

    fn read_tile(&self, tx: i32, ty: i32) -> Result<Tile, String> {
        let side = self.side as usize;
        let n = side * side;
        let past = || Tile { side, grounds: vec![-1; n], waters: vec![0; n], profiles: vec![0; n] };
        if tx < 0 || ty < 0 || tx >= self.tiles || ty >= self.tiles {
            return Ok(past());
        }
        let path = self.folder.join("tiles").join(format!("{tx} {ty}.bin"));
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(past()),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        let raw = miniz_oxide::inflate::decompress_to_vec(&bytes).map_err(|error| format!("{}: inflate: {error:?}", path.display()))?;
        let mut at = 0;
        let mut leb = || -> Result<u64, String> {
            let (mut value, mut shift) = (0u64, 0);
            loop {
                let byte = *raw.get(at).ok_or_else(|| format!("{}: ends early", path.display()))?;
                at += 1;
                value |= ((byte & 0x7F) as u64) << shift;
                if byte & 0x80 == 0 {
                    return Ok(value);
                }
                shift += 7;
            }
        };
        // The ground: each the change from the column before (zigzag), the first from 0.
        let mut last = 0i64;
        let mut grounds = Vec::with_capacity(n);
        for _ in 0..n {
            let value = leb()?;
            last += ((value >> 1) as i64) ^ -((value & 1) as i64);
            grounds.push(last as i32);
        }
        let waters = (0..n).map(|_| leb().map(|value| value as i32)).collect::<Result<Vec<_>, _>>()?;
        let profiles = (0..n).map(|_| leb().map(|value| value as u32)).collect::<Result<Vec<_>, _>>()?;
        Ok(Tile { side, grounds, waters, profiles })
    }

    /// One column (its whole tile is read for it).
    pub fn column(&self, north: i32, east: i32) -> Result<Column, String> {
        let (tile, [i, j]) = self.place(north, east);
        self.tile(tile[0], tile[1])?.column(self, i, j)
    }

    /// The top of a column's ground or water, whichever is higher: what a player stands on, or floats at (none past the
    /// map).
    pub fn surface(&self, column: &Column) -> Option<i32> {
        match column {
            Column::Past => None,
            Column::Here { ground, water, .. } => Some((ground + 1).max(self.sea_level).max(water.unwrap_or(0))),
        }
    }

    /// A column's blocks from bedrock up, by the export's rule (its README): below the ground's deepest layer stone,
    /// then each layer under the top to the depth it reaches, the top block at the ground, then water up to the sea or
    /// the river's or lake's top, whichever is higher. The same block side by side is one run.
    fn runs(&self, ground: i32, water: i32, profile: u32) -> Result<Vec<Run>, String> {
        let profile = self.profiles.get(profile as usize).and_then(Option::as_ref).ok_or_else(|| format!("profile {profile} isn't in world.toml"))?;
        let mut runs: Vec<Run> = Vec::new();
        let mut push = |block: Block, from: i32, to: i32| {
            let from = from.max(0);
            if to <= from {
                return;
            }
            match runs.last_mut() {
                Some(last) if last.block == block && last.to == from => last.to = to,
                _ => runs.push(Run { block, from, to }),
            }
        };
        let deepest = profile.under.last().map_or(0, |(_, depth)| *depth);
        push(STONE, 0, ground - deepest);
        // Each layer from the depth the one above it reaches down to its own, pushed deepest first (bedrock up).
        for (k, &(block, depth)) in profile.under.iter().enumerate().rev() {
            let above = if k == 0 { 0 } else { profile.under[k - 1].1 };
            push(block, ground - depth, ground - above);
        }
        push(profile.top, ground, ground + 1);
        let top = if water > 0 { water.max(self.sea_level) } else { self.sea_level };
        push(WATER, ground + 1, top);
        Ok(runs)
    }

    /// The least corner's column (north, east), and columns a side of a tile.
    pub fn grid(&self) -> ([i32; 2], i32) {
        (self.least, self.side)
    }

    /// Columns a side of the whole map (its tiles a side, each `grid`'s columns a side).
    pub fn across(&self) -> i32 {
        self.tiles * self.side
    }

    /// The world's height in blocks (nothing at or above it).
    pub fn height(&self) -> i32 {
        self.world_height
    }

    /// A profile by its number: its top block, and each block under it with the depth it reaches, deepest last.
    pub fn profile(&self, number: u32) -> Option<(Block, &[(Block, i32)])> {
        self.profiles.get(number as usize).and_then(Option::as_ref).map(|profile| (profile.top, profile.under.as_slice()))
    }
}

impl Tile {
    /// The column `i` north and `j` east of the tile's least corner as the export holds it: its ground (−1 past the
    /// map), its river's or lake's water (0 none), and its profile's number. For readers of many columns at once (the
    /// far cells), where making each column's runs would cost more than the reading.
    pub fn raw(&self, i: usize, j: usize) -> (i32, i32, u32) {
        let k = i + j * self.side;
        (self.grounds[k], self.waters[k], self.profiles[k])
    }

    /// The column `i` north and `j` east of the tile's least corner.
    pub fn column(&self, doorway: &Doorway, i: usize, j: usize) -> Result<Column, String> {
        let k = i + j * self.side;
        let (ground, water, profile) = (self.grounds[k], self.waters[k], self.profiles[k]);
        if ground < 0 {
            return Ok(Column::Past);
        }
        Ok(Column::Here { ground, water: (water > 0).then_some(water), runs: doorway.runs(ground, water, profile)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// The gate (plan 2.1): every sample column of old (`references/columns.txt`) answered to the block, and its runs
    /// keeping the export's rule. The export is on this laptop and the USB, not in git: without it, this fails and says so.
    #[test]
    fn the_doorway_answers_old_s_sample_columns() {
        let folder = test_world();
        assert!(folder.join("world.toml").is_file(), "the test world's export isn't at {} (it's on the USB: exports/nice25 test)", folder.display());
        let doorway = Doorway::open(&folder).unwrap();
        let samples = std::fs::read_to_string(crate::session::project_folder().join("references").join("columns.txt")).unwrap();
        let started = Instant::now();
        let (mut checked, mut past, mut wet, mut tiles_read) = (0, 0, 0, 0);
        let mut tile_ms = Vec::new();
        let mut kept: Option<([i32; 2], Tile)> = None;
        for line in samples.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()) {
            let words: Vec<&str> = line.split_whitespace().collect();
            let (north, east): (i32, i32) = (words[0].parse().unwrap(), words[1].parse().unwrap());
            let (at, [i, j]) = doorway.place(north, east);
            if kept.as_ref().is_none_or(|(kept_at, _)| *kept_at != at) {
                let read = Instant::now();
                kept = Some((at, doorway.tile(at[0], at[1]).unwrap()));
                tile_ms.push(read.elapsed().as_secs_f64() * 1000.0);
                tiles_read += 1;
            }
            let column = kept.as_ref().unwrap().1.column(&doorway, i, j).unwrap();
            checked += 1;
            if words[2] == "past" {
                assert_eq!(column, Column::Past, "{line}: old says past the map");
                past += 1;
                continue;
            }
            let (ground, water): (i32, Option<i32>) = (words[2].parse().unwrap(), if words[3] == "-" { None } else { Some(words[3].parse().unwrap()) });
            let Column::Here { ground: answered, water: answered_water, runs } = column else { panic!("{line}: the doorway says past the map") };
            assert_eq!((answered, answered_water), (ground, water), "{line}: the doorway's ground and water");
            wet += water.is_some() as usize;
            // The export's rule: from bedrock up with no gap, the top block at the ground, water to the sea or the
            // river's top, air above.
            assert_eq!(runs.first().map(|run| run.from), Some(0), "{line}: the runs start at bedrock");
            assert!(runs.windows(2).all(|pair| pair[0].to == pair[1].from && pair[0].block != pair[1].block), "{line}: the runs touch and differ: {runs:?}");
            let top = runs.iter().find(|run| run.from <= ground && ground < run.to).unwrap();
            assert_ne!(doorway.block_name(top.block), "incus:water", "{line}: the ground isn't water");
            let wanted = water.map_or(doorway.sea_level, |water| water.max(doorway.sea_level)).max(ground + 1);
            assert_eq!(runs.last().unwrap().to, wanted, "{line}: the column's top");
        }
        let seconds = started.elapsed().as_secs_f64();
        tile_ms.sort_by(f64::total_cmp);
        eprintln!(
            "The doorway: {checked} sample columns answered as old's ({past} past the map, {wet} with a river's or lake's water); {tiles_read} tiles read in {seconds:.1} s; a tile median {:.2} ms, slowest {:.2} ms",
            tile_ms[tile_ms.len() / 2],
            tile_ms[tile_ms.len() - 1]
        );
        assert_eq!(checked, 5047, "references/columns.txt holds 5,047 sample columns");
    }

    /// A column's runs by the export's rule, with layers under the top (the test world's profiles have none).
    #[test]
    fn layers_under_the_top_go_where_their_depths_say() {
        let doorway = Doorway {
            folder: PathBuf::new(),
            least: [0, 0],
            side: 1,
            tiles: 1,
            sea_level: 128,
            world_height: 1024,
            blocks: vec!["stone".into(), "water".into(), "grass".into(), "dirt".into(), "clay".into()],
            profiles: vec![Some(Profile { top: 2, under: vec![(3, 3), (4, 5)] })],
            changes: HashMap::new(),
            times: RefCell::new(Vec::new()),
        };
        // Ground 200: stone below 195, clay 195 to 197, dirt 197 to 200, grass at 200, nothing over it.
        assert_eq!(doorway.runs(200, 0, 0).unwrap(), vec![Run { block: STONE, from: 0, to: 195 }, Run { block: 4, from: 195, to: 197 }, Run { block: 3, from: 197, to: 200 }, Run { block: 2, from: 200, to: 201 }]);
        // Ground 100, under the sea: water from 101 up to the sea (128).
        assert_eq!(doorway.runs(100, 0, 0).unwrap().last(), Some(&Run { block: WATER, from: 101, to: 128 }));
        // Ground 150 with a lake's top at 155: water from 151 to 155.
        assert_eq!(doorway.runs(150, 155, 0).unwrap().last(), Some(&Run { block: WATER, from: 151, to: 155 }));
        // Ground 2, its layers reaching below bedrock: cut at 0, so only the dirt (the first 3 down) is left under it.
        assert_eq!(doorway.runs(2, 0, 0).unwrap().first(), Some(&Run { block: 3, from: 0, to: 2 }));
    }
}

//! Sight's live levels (the regroup, phase 3.1c): cells of 1, 2, 4, 8 and 16 blocks kept round the camera,
//! 128 cells a side at each level, filled from the recipe on the card a tile at a time on the loading lane,
//! nearest first, and let go as the camera leaves them.
//!
//! **Only each cell column's surface band is kept** (`CellColumn`): below it every cell is solid with no face
//! showing, above it every cell is empty. The band runs from the cell holding the lowest ground of the
//! column's blocks and their neighbours (or lower, where a cave box comes near; from the bottom where a
//! layer is air or water, or the map ends) to the cell holding its highest ground, water or sea.
//!
//! **The standing test:** every cell is exactly its eight children at the level below added, the cells left
//! out of the bands included, which checks the bands as well as the sums.
//!
//! Every level is made from the recipe (exact either way); building a level from the one below where they
//! overlap is the saving kept for when the card's time is short.

use super::cell::Sums;
use crate::loading::{Job, Tier};
use crate::world::CavePlan;
use crate::world::card_recipe::WorldOnCard;
use std::collections::HashMap;
use std::sync::Arc;

/// The live levels: cells of 1, 2, 4, 8 and 16 blocks.
pub const LIVE_LEVELS: usize = 5;
/// Cell columns a side in a tile, the unit filled at once.
pub const TILE: i32 = 16;
/// Tiles being made at once, the fine levels' (cells of 1, 2 and 4) and the coarse ones' (8 and 16) apart. Every
/// tile is 16 × 16 cell columns, but each cell of 16 adds up 4,096 blocks to a cell of 8's 512 and a block's 1: a
/// tile of 16s is about 8 times the card's work of a tile of 8s, and thousands of times a tile of blocks. Taken
/// together, the coarse tiles held up the near blocks behind them [David 2026-11-07, build 76: "it takes forever
/// to load chunks around me... hopefully we can speed it up or make it go to the closest chunk first"]; the fine
/// ones mostly wait on the card's round trips, so more of them at once costs it little. (Measured, flying the soak
/// at 150 a second at 237: 3 tiles at once of any level cost about 1.6 ms a frame; 8 at once, 3.5.)
/// `INCUS_SIGHT_FILLING=<fine>,<coarse>` other numbers (measuring).
pub const MOST_FILLING: (usize, usize) = (8, 2);
/// The coarsest fine level.
const FINE_UP_TO: usize = 2;

fn most_filling() -> (usize, usize) {
    std::env::var("INCUS_SIGHT_FILLING").ok().and_then(|v| v.split_once(',').and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))).unwrap_or(MOST_FILLING)
}

/// A tile: which level, and where (in tiles).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub level: usize,
    pub x: i32,
    pub y: i32,
}

impl TileKey {
    /// Blocks a cell at this level.
    pub fn size(&self) -> i32 {
        1 << self.level
    }

    /// The tile's least corner, in blocks, and how many blocks a side.
    pub fn blocks(&self) -> (i32, i32, i32) {
        let side = TILE * self.size();
        (self.x * side, self.y * side, side)
    }
}

/// A cell column's kept band: its lowest kept cell (cell index up: `z / size`), and every cell from there
/// up. Below `low`, solid all round if `low > 0`; above the band, empty.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CellColumn {
    pub low: i32,
    pub sums: Vec<Sums>,
}

impl CellColumn {
    /// The cell `index` up, the ones left out included.
    /// Its highest kept cell (`low − 1` if none).
    pub fn high(&self) -> i32 {
        self.low + self.sums.len() as i32 - 1
    }

    pub fn at(&self, index: i32, size: i32) -> Sums {
        if index >= self.low && index < self.low + self.sums.len() as i32 {
            self.sums[(index - self.low) as usize]
        } else if index < self.low && index >= 0 {
            Sums { solid: (size as u64).pow(3), ..Sums::default() }
        } else {
            Sums::default()
        }
    }
}

/// A tile's cell columns, x fastest, and what making it took (ms: the columns round it, then its cells).
#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub columns: Vec<CellColumn>,
    /// Each cell's block, in the columns' order (only a cell of one block's means anything: `block_of`).
    pub blocks: Vec<u16>,
    pub took: (f32, f32),
}

impl Tile {
    pub fn column(&self, x: i32, y: i32) -> &CellColumn {
        &self.columns[(y * TILE + x) as usize]
    }

    pub fn cells(&self) -> usize {
        self.columns.iter().map(|c| c.sums.len()).sum()
    }
}

/// Makes a tile: the columns round it on the card, each cell column's band worked out from them, then the
/// band's cells on the card. `odd`: a layer somewhere is air or water, so bands start at the bottom. Waits on
/// the card: for the loading lane.
pub fn fill_tile(device: &wgpu::Device, queue: &wgpu::Queue, world: &WorldOnCard, caves: &CavePlan, odd: bool, key: TileKey) -> Tile {
    let size = key.size();
    let (x0, y0, side) = key.blocks();
    let started = std::time::Instant::now();
    let mut columns_ms = 0.0;
    let mut bands = Vec::with_capacity((TILE * TILE) as usize);
    let made = world.cells_by(device, queue, [x0, y0, side, side], |grounds| {
        columns_ms = started.elapsed().as_secs_f32() * 1000.0;
        // The region's grounds and three columns round them.
        let across = side + 6;
        let column = |x: i32, y: i32| grounds[((y - y0 + 3) * across + (x - x0 + 3)) as usize];
        bands_of(column, x0, y0, size, TILE, caves, odd, &mut bands)
    }, LIVE_PIECES);
    let cells_ms = started.elapsed().as_secs_f32() * 1000.0 - columns_ms;
    let blocks = made.iter().map(block_of).collect();
    let mut made = made.iter().map(sums_of);
    Tile { columns: bands.into_iter().map(|(low, count)| CellColumn { low, sums: (0..count).filter_map(|_| made.next()).collect() }).collect(), blocks, took: (columns_ms, cells_ms) }
}

/// A tile's cell columns in the form the card keeps (each its lowest cell and its cells), a cell of one block
/// with its block (3.6).
pub fn compact_tile(tile: &Tile, size: u32) -> Vec<(i32, Vec<super::cell::Cell>)> {
    let mut blocks = tile.blocks.iter();
    tile.columns
        .iter()
        .map(|c| {
            let cells = c.sums.iter().map(|s| {
                let block = blocks.next().copied().unwrap_or(0);
                let cell = s.compact(size);
                if size == 1 { cell.with_block(block) } else { cell }
            });
            (c.low, cells.collect())
        })
        .collect()
}

/// How the live levels' card work is cut into pieces: columns, and cells, a piece.
pub const LIVE_PIECES: (u32, u32) = (8192, 512);

/// Each cell column's band (its lowest kept cell and how many), for `across` × `across` cell columns of `size`
/// from (`x0`, `y0`), and the cells to make for them. `column` gives a column's ground (−1 past the map) and
/// water, for those columns and one round them.
#[allow(clippy::too_many_arguments)]
pub fn bands_of(column: impl Fn(i32, i32) -> [i32; 2], x0: i32, y0: i32, size: i32, across: i32, caves: &CavePlan, odd: bool, bands: &mut Vec<(i32, i32)>) -> Vec<[i32; 4]> {
    // The cave boxes (32 blocks a side) and their levels, for the bands they reach down into.
    let boxes: Vec<(i32, i32, i32)> = caves.boxes_with_pieces().map(|(&b, _)| b).collect();
    let sea = crate::world::SEA_LEVEL;
    let mut cells = Vec::new();
    for cy in 0..across {
        for cx in 0..across {
            let (bx, by) = (x0 + cx * size, y0 + cy * size);
            let (mut lowest, mut highest, mut outside) = (i32::MAX, i32::MIN, false);
            for y in by - 1..=by + size {
                for x in bx - 1..=bx + size {
                    let [ground, water] = column(x, y);
                    if ground < 0 {
                        outside = true;
                        continue;
                    }
                    lowest = lowest.min(ground);
                    let own = (bx..bx + size).contains(&x) && (by..by + size).contains(&y);
                    if own {
                        highest = highest.max(ground).max(water - 1).max(sea - 1);
                    }
                }
            }
            if highest == i32::MIN {
                // Past the map: nothing.
                bands.push((0, 0));
                continue;
            }
            let mut low = if outside || odd { 0 } else { lowest.div_euclid(size) };
            // A cave box over the cell column or the blocks just round it: the band reaches down to the
            // block just under the box (its top face may show into the cave).
            for &(cave_x, cave_y, level) in &boxes {
                let (from_x, from_y) = (cave_x * 32, cave_y * 32);
                if bx - 1 <= from_x + 31 && bx + size >= from_x && by - 1 <= from_y + 31 && by + size >= from_y {
                    low = low.min((level * 32 - 1).max(0).div_euclid(size));
                }
            }
            let high = highest.min(crate::world::WORLD_HEIGHT - 1).div_euclid(size).max(low - 1);
            bands.push((low, high - low + 1));
            for z in low..=high {
                cells.push([bx, by, z * size, size]);
            }
        }
    }
    cells
}

/// The card's 17 numbers as `Sums`.
/// Which block a cell of one block is (the card's sum of its solid blocks' numbers; 0 for none).
pub fn block_of(c: &[u32; super::cell::CARD_NUMBERS]) -> u16 {
    c[17].min(u16::MAX as u32) as u16
}

pub fn sums_of(c: &[u32; super::cell::CARD_NUMBERS]) -> Sums {
    let wide = |k: usize| c[k] as u64 | (c[k + 1] as u64) << 32;
    Sums {
        solid: c[0] as u64,
        water: c[1] as u64,
        faces: std::array::from_fn(|d| c[2 + d] as u64),
        colour: std::array::from_fn(|k| c[8 + k] as u64),
        light: c[11] as u64,
        light_squared: wide(12),
        height: c[14] as u64,
        height_squared: wide(15),
    }
}

/// What a live store needs to make tiles: the card, the world on it, the caves, and the store on the card the
/// tiles go to (none in tests).
#[derive(Clone)]
pub struct Maker {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub world: Arc<WorldOnCard>,
    pub caves: Arc<CavePlan>,
    pub card: Option<Arc<super::card_store::CardStore>>,
}

/// The cells a side the live levels want for a coarse grid of `factor` (David 2026-11-06, the sizing rule):
/// each level reaching out to where one of its cells is one coarse pixel, 1,342 ÷ the factor at 100° on 1,600
/// rows, in whole tiles.
pub fn side_for(factor: u32) -> i32 {
    let cells = (1342.0 / factor.max(1) as f32).ceil() as i32;
    (cells + TILE - 1) / TILE * TILE
}

/// What a tile took: its cells, and the milliseconds for its columns and its cells.
struct Made {
    cells: usize,
    took: (f32, f32),
}

/// The live levels round the camera: which tiles are made (their cells are on the card), and which are being.
pub struct LiveStore {
    /// The levels kept live, and their cell columns a side (a whole number of tiles).
    pub levels: Vec<usize>,
    pub side: i32,
    tiles: HashMap<TileKey, usize>,
    filling: HashMap<TileKey, Job<Made>>,
    /// Tiles an edit reached while they were being made (with the blocks as they were): made again after.
    stale: std::collections::HashSet<TileKey>,
    /// Tiles made since it started (for the log), and by level what they took (columns ms, cells ms, how many).
    pub made: usize,
    took: [(f32, f32, usize); LIVE_LEVELS],
}

impl LiveStore {
    pub fn new(levels: Vec<usize>, side: i32) -> Self {
        Self { levels, side, tiles: HashMap::new(), filling: HashMap::new(), stale: Default::default(), made: 0, took: [(0.0, 0.0, 0); LIVE_LEVELS] }
    }

    /// Edits at `spots` (blocks): the tiles they reach, at every level, made again (build 88). A tile's cells
    /// stay on the card till the new ones replace them; one being made is made again once it's done.
    pub fn touch(&mut self, spots: impl IntoIterator<Item = [i32; 3]>) {
        let mut keys = std::collections::HashSet::new();
        for [x, y, _] in spots {
            for &level in &self.levels {
                let tile_blocks = TILE << level;
                // The block and the ones beside it (whose faces it may bare), across a tile's edge too.
                for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                    keys.insert(TileKey { level, x: (x + dx).div_euclid(tile_blocks), y: (y + dy).div_euclid(tile_blocks) });
                }
            }
        }
        for key in keys {
            self.tiles.remove(&key);
            if self.filling.contains_key(&key) {
                self.stale.insert(key);
            }
        }
    }

    /// The tiles the levels want round `camera` (x and y, blocks), nearest first: exactly `side / TILE` a
    /// side, so in the card's wrapping window no two of them share a place. Near is measured in each level's own
    /// cells, so the levels fill side by side (by blocks, the cells of 1 round the eye always came first and the
    /// coarser levels behind them starved [2026-11-06, building the cells of 1: 14 of 1,764 tiles of 8 made]),
    /// and in those units the loading lane weighs them against the chunks.
    pub fn wanted(&self, camera: (f32, f32)) -> Vec<(f32, TileKey)> {
        let n = self.side / TILE;
        let mut wanted = Vec::new();
        for &level in &self.levels {
            let tile_blocks = (TILE << level) as f32;
            let (centre_x, centre_y) = ((camera.0 / tile_blocks).floor() as i32, (camera.1 / tile_blocks).floor() as i32);
            for x in centre_x - n / 2..centre_x - n / 2 + n {
                for y in centre_y - n / 2..centre_y - n / 2 + n {
                    let centre = ((x as f32 + 0.5) * tile_blocks, (y as f32 + 0.5) * tile_blocks);
                    let distance = ((centre.0 - camera.0).powi(2) + (centre.1 - camera.1).powi(2)).sqrt();
                    wanted.push((distance / (1 << level) as f32, TileKey { level, x, y }));
                }
            }
        }
        wanted.sort_by(|a, b| a.0.total_cmp(&b.0));
        wanted
    }

    /// Keeps the levels round `camera`: tiles left behind let go (and their making dropped), made ones
    /// taken in, and more asked for, nearest first, a few at a time (fine and coarse apart).
    pub fn update(&mut self, camera: (f32, f32), maker: &Maker, odd: bool, edits: Option<&Arc<super::edits::Edits>>) {
        let wanted = self.wanted(camera);
        let keep: std::collections::HashSet<TileKey> = wanted.iter().map(|&(_, key)| key).collect();
        self.tiles.retain(|key, _| keep.contains(key));
        self.filling.retain(|key, _| keep.contains(key));
        self.stale.retain(|key| self.filling.contains_key(key));
        let mut done = Vec::new();
        self.filling.retain(|&key, job| match job.ready() {
            Ok(Some(made)) => {
                done.push((key, made));
                false
            }
            Ok(None) => true,
            Err(()) => false,
        });
        self.made += done.len();
        for (key, made) in done {
            let took = &mut self.took[key.level];
            *took = (took.0 + made.took.0, took.1 + made.took.1, took.2 + 1);
            // Edited while it was being made: made again.
            if !self.stale.remove(&key) {
                self.tiles.insert(key, made.cells);
            }
        }
        let most = most_filling();
        let fine = |key: &TileKey| key.level <= FINE_UP_TO;
        let mut filling = (self.filling.keys().filter(|k| fine(k)).count(), self.filling.keys().filter(|k| !fine(k)).count());
        for (distance, key) in wanted {
            if filling.0 >= most.0 && filling.1 >= most.1 {
                break;
            }
            let (n, cap) = if fine(&key) { (&mut filling.0, most.0) } else { (&mut filling.1, most.1) };
            if *n >= cap || self.tiles.contains_key(&key) || self.filling.contains_key(&key) {
                continue;
            }
            *n += 1;
            let maker = maker.clone();
            let edits = edits.filter(|edits| edits.any()).cloned();
            // Keyed in chunks' units (32 cells), so sight and the chunks round the camera share the lane fairly.
            let job = crate::loading::submit(Tier::Soon, distance / 32.0, move || {
                let mut tile = fill_tile(&maker.device, &maker.queue, &maker.world, &maker.caves, odd, key);
                if let Some(edits) = &edits {
                    super::edits::patch_tile(&mut tile, key, edits);
                }
                let size = key.size() as u32;
                let columns = compact_tile(&tile, size);
                if let Some(card) = &maker.card {
                    card.put_tile(key.level, key.x, key.y, TILE as u32, &columns);
                }
                Made { cells: tile.cells(), took: tile.took }
            });
            self.filling.insert(key, job);
        }
    }

    /// For the log: tiles made and wanted at each level, and the cells kept.
    pub fn report(&self, camera: (f32, f32)) -> String {
        let wanted = self.wanted(camera);
        let levels: Vec<String> = self
            .levels
            .iter()
            .map(|&level| {
                let want = wanted.iter().filter(|(_, k)| k.level == level).count();
                let have = self.tiles.keys().filter(|k| k.level == level).count();
                let (columns, cells, n) = self.took[level];
                format!("cells of {}: {have}/{want} tiles, a tile {:.1}+{:.1} ms", 1 << level, columns / n.max(1) as f32, cells / n.max(1) as f32)
            })
            .collect();
        let cells: usize = self.tiles.values().sum();
        format!("Sight's live levels ({} a side): {}; {cells} cells on the card, {} filling, {} made", self.side, levels.join("; "), self.filling.len(), self.made)
    }
}

//! Sight's bake (the regroup, phases 3.1d and 3.1e): the cells of 32 blocks and up, for the whole world,
//! made once and kept.
//!
//! - **Only the cells of 32 run the recipe**, on the card, a region at a time (one cell column of 1,024 blocks:
//!   32 × 32 cell columns of 32). Every level above is its children added, exactly (`parent_of`): within a
//!   region up to 1,024, and above that as soon as all four children of a cell column are baked, up to the
//!   level where four cells cover the whole map.
//! - **Nearest the spawn first**, ring by ring (square rings, so the eighth of the world round the spawn, the
//!   start bake [David 2026-11-06: "an eighth of the world being immediately present as the rest fill in"],
//!   is done before anything past it), on the loading lane at its lowest priority, the card's work in pieces
//!   waited for gently.
//! - **Kept compact** (`Cell`, 16 bytes, by level in flat arrays); the exact sums are kept only from 1,024 up,
//!   where the next levels are built from them.
//! - **Edits** (3.1e): the recipe doesn't know them, so a cell column of 32 whose chunk, or a chunk beside it
//!   (whose faces an edit may bare), has changed is made from the blocks themselves (`column_from_blocks`),
//!   the changed chunks as they are and the rest as the recipe makes them; its region's levels are built up
//!   again from there.
//! - **Kept with the world** (3.1e, `sight-bake.bin`): the rules' stamp (`WorldOnCard::stamp`: the world's
//!   pack, the cells' code), then a record for each region as it's baked or patched, added to the end at each
//!   save; read back at load if the stamp is the same, a later record for a region over an earlier one.

use super::cell::{Cell, FarColours, Sums, sums_of_box};
use super::store::{CellColumn, Maker, bands_of, sums_of};
use crate::world::{BlockId, Chunk, ChunkPos, CompiledMap, TerrainBlocks};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// The first baked level: cells of 32 blocks (a chunk's width: a changed chunk is one cell column).
pub const FIRST: usize = 5;
/// A region is one cell column of this level (1,024 blocks).
pub const REGION: usize = 10;
/// How the bake's card work is cut into pieces (columns, and cells, a piece): bigger than the live levels',
/// about a millisecond of the card each.
pub const BAKE_PIECES: (u32, u32) = (32768, 1024);
/// Regions being baked at once.
pub const BAKERS: usize = 4;
/// The bake's file in the world's folder.
pub const FILE: &str = "sight-bake.bin";

/// The file's first bytes, and its layout's version (2: the cells keep six ways' shares of their faces, 3.4a).
const MAGIC: &[u8; 8] = b"INCUSBK2";
/// Every version's first seven bytes: a file of an older layout is baked again, not refused.
const KIND: &[u8; 7] = b"INCUSBK";

/// A baked level's bounds: its cell size, its least cell column (absolute: cell `n` runs from `n × size`), and
/// how many across each way.
#[derive(Clone, Copy, Debug)]
pub struct Extent {
    pub size: i32,
    pub from: (i32, i32),
    pub across: i32,
}

impl Extent {
    pub fn of(level: usize, half: i32) -> Self {
        let size = 1 << level;
        let from = (-half).div_euclid(size);
        Self { size, from: (from, from), across: (half - 1).div_euclid(size) - from + 1 }
    }

    fn inside(&self, cx: i32, cy: i32) -> bool {
        (0..self.across).contains(&(cx - self.from.0)) && (0..self.across).contains(&(cy - self.from.1))
    }
}

/// A cell column made from its four children (each `child_size` a cell), exactly: each cell the sum of the
/// eight under it, the cells left out of the children's bands included.
pub fn parent_of(children: [&CellColumn; 4], child_size: i32) -> CellColumn {
    let low = children.iter().map(|c| c.low).min().unwrap_or(0).div_euclid(2);
    let Some(high) = children.iter().filter(|c| !c.sums.is_empty()).map(|c| c.high()).max() else {
        // Nothing kept in any (all past the map, or all solid up to their bands and empty above).
        return CellColumn { low, sums: Vec::new() };
    };
    let sums = (low..=high.div_euclid(2))
        .map(|z| {
            let mut sums = Sums::default();
            for child in &children {
                sums.add(&child.at(2 * z, child_size));
                sums.add(&child.at(2 * z + 1, child_size));
            }
            sums
        })
        .collect();
    CellColumn { low, sums }
}

/// A region's cell columns, level by level from 32 to 1,024 (x fastest at each), from its cells of 32.
fn build_up(first: Vec<CellColumn>) -> Vec<Vec<CellColumn>> {
    let mut levels = vec![first];
    for level in FIRST + 1..=REGION {
        let below = levels.last().map(Vec::as_slice).unwrap_or_default();
        let child_across = 1 << (REGION - level + 1);
        let across = child_across / 2;
        let columns = (0..across * across)
            .map(|i| {
                let (x, y) = (i % across, i / across);
                let child = |a: i32, b: i32| &below[((y * 2 + b) * child_across + x * 2 + a) as usize];
                parent_of([child(0, 0), child(1, 0), child(0, 1), child(1, 1)], 1 << (level - 1))
            })
            .collect();
        levels.push(columns);
    }
    levels
}

/// One region as kept: its compact cell columns level by level (32 to 1,024), and its 1,024's exact sums.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub region: (i32, i32),
    pub levels: Vec<Vec<(i32, Vec<Cell>)>>,
    pub top: CellColumn,
}

impl Record {
    fn of(region: (i32, i32), made: &[Vec<CellColumn>]) -> Self {
        let levels = made.iter().enumerate().map(|(k, columns)| columns.iter().map(|c| (c.low, c.sums.iter().map(|s| s.compact(1 << (FIRST + k))).collect())).collect()).collect();
        Self { region, levels, top: made.last().and_then(|c| c.first()).cloned().unwrap_or_default() }
    }
}

/// The whole bake.
pub struct Bake {
    /// Each level's bounds, from cells of 32 up.
    pub extents: Vec<Extent>,
    /// Every region's record as last made (its cell columns from 32 to 1,024): kept once, shared with a save
    /// under way; made again, replaced whole (so nothing is left behind).
    records: HashMap<(i32, i32), Arc<Record>>,
    /// Above 1,024: each cell column's compact cells, and the exact sums from 1,024 up (the levels above are
    /// made from them).
    upper: HashMap<(usize, i32, i32), (i32, Vec<Cell>)>,
    sums: HashMap<(usize, i32, i32), CellColumn>,
    pub regions: usize,
    pub start_regions: usize,
    pub regions_done: usize,
    pub started: Instant,
    /// Seconds to the start bake, and to all of it.
    pub start_done: Option<f32>,
    pub all_done: Option<f32>,
    /// The order regions were made in (for the file), and the file it's kept in with how many of `order` are
    /// in it already.
    order: Vec<(i32, i32)>,
    kept_in: Option<(PathBuf, usize)>,
    pub stamp: u64,
    pub half: i32,
}

impl Bake {
    /// The top level: four cells or fewer cover the map.
    pub fn top(half: i32) -> usize {
        let mut level = REGION;
        while (1 << level) < half {
            level += 1;
        }
        level
    }

    pub fn extent(&self, level: usize) -> Extent {
        self.extents[level - FIRST]
    }

    pub fn is_baked(&self, region: (i32, i32)) -> bool {
        self.records.contains_key(&region)
    }

    /// A cell column's band at `level` (its lowest cell, and its cells), if it's baked; `Some((0, []))` past
    /// the map.
    pub fn column(&self, level: usize, cx: i32, cy: i32) -> Option<(i32, &[Cell])> {
        if !self.extent(level).inside(cx, cy) {
            return Some((0, &[]));
        }
        if level <= REGION {
            let per = 1 << (REGION - level);
            let record = self.records.get(&(cx.div_euclid(per), cy.div_euclid(per)))?;
            let (low, cells) = &record.levels[level - FIRST][(cy.rem_euclid(per) * per + cx.rem_euclid(per)) as usize];
            return Some((*low, cells.as_slice()));
        }
        self.upper.get(&(level, cx, cy)).map(|(low, cells)| (*low, cells.as_slice()))
    }

    /// A region's record put in (made, patched or read back); then every level above built where all its
    /// children are baked.
    fn take(&mut self, record: Record, card: Option<&super::card_store::CardStore>) {
        let region = record.region;
        self.sums.insert((REGION, region.0, region.1), record.top.clone());
        let record = Arc::new(record);
        if let Some(card) = card {
            card.put_region(&record);
        }
        if self.records.insert(region, record).is_none() {
            self.regions_done += 1;
        }
        self.order.push(region);
        let seconds = self.started.elapsed().as_secs_f32();
        if self.start_done.is_none() && self.regions_done >= self.start_regions {
            self.start_done = Some(seconds);
        }
        if self.all_done.is_none() && self.regions_done == self.regions {
            self.all_done = Some(seconds);
        }
        // Up the levels while all four children are there.
        let (mut cx, mut cy) = region;
        for level in REGION + 1..FIRST + self.extents.len() {
            let (px, py) = (cx.div_euclid(2), cy.div_euclid(2));
            let below = self.extent(level - 1);
            let mut children = Vec::with_capacity(4);
            for k in 0..4 {
                let (x, y) = (px * 2 + k % 2, py * 2 + k / 2);
                match self.sums.get(&(level - 1, x, y)) {
                    Some(column) => children.push(column.clone()),
                    None if !below.inside(x, y) => children.push(CellColumn::default()),
                    None => return,
                }
            }
            let parent = parent_of([&children[0], &children[1], &children[2], &children[3]], 1 << (level - 1));
            let size = 1 << level;
            let cells: Vec<Cell> = parent.sums.iter().map(|s| s.compact(size)).collect();
            if let Some(card) = card {
                card.put_upper(level, px, py, parent.low, &cells);
            }
            self.upper.insert((level, px, py), (parent.low, cells));
            self.sums.insert((level, px, py), parent);
            (cx, cy) = (px, py);
        }
    }

    /// How many compact cells it holds.
    pub fn cells(&self) -> usize {
        self.records.values().map(|r| r.levels.iter().flatten().map(|(_, c)| c.len()).sum::<usize>()).sum::<usize>() + self.upper.values().map(|(_, c)| c.len()).sum::<usize>()
    }

    /// For the log.
    pub fn report(&self) -> String {
        let cells = self.cells();
        let at = |seconds: Option<f32>| seconds.map_or("not yet".to_string(), |s| format!("{s:.0} s"));
        format!(
            "Sight's bake (cells of 32 up): regions {}/{} (the start's {} in {}; all in {}), {cells} cells kept ({:.0} MB)",
            self.regions_done,
            self.regions,
            self.start_regions,
            at(self.start_done),
            at(self.all_done),
            cells as f64 * 16.0 / 1e6
        )
    }
}

/// What a save writes, taken from the bake in a moment (the records are shared, not copied): written later,
/// on the loading lane.
struct Writing {
    path: PathBuf,
    fresh: bool,
    header: Vec<u8>,
    records: Vec<Arc<Record>>,
    /// How many of the bake's `order` the file holds once it's written.
    then: usize,
}

impl Bake {
    /// What keeping it in `folder` takes: the records made since it was last written there, to go on the end
    /// of its file, or the whole of it afresh if it's a new place (or the file's gone, or was cut short).
    fn to_write(&self, folder: &Path) -> Writing {
        let path = folder.join(FILE);
        let (fresh, from) = match &self.kept_in {
            Some((kept, done)) if *kept == path && path.exists() => (false, *done),
            _ => (true, 0),
        };
        let mut header = Vec::new();
        if fresh {
            header.extend_from_slice(MAGIC);
            header.extend_from_slice(&self.stamp.to_le_bytes());
            header.extend_from_slice(&self.half.to_le_bytes());
        }
        // Fresh, every region once (its latest record); else the ones made since, in order.
        let regions: Vec<(i32, i32)> = if fresh {
            let mut seen = HashSet::new();
            self.order.iter().rev().filter(|r| seen.insert(**r)).copied().collect::<Vec<_>>().into_iter().rev().collect()
        } else {
            self.order[from.min(self.order.len())..].to_vec()
        };
        Writing { path, fresh, header, records: regions.iter().filter_map(|r| self.records.get(r).cloned()).collect(), then: self.order.len() }
    }

    /// Kept in `folder`'s file now, waiting till it's written (tests; the game keeps it on the lane:
    /// `keep_later`).
    pub fn keep(&mut self, folder: &Path) -> Result<String, String> {
        let writing = self.to_write(folder);
        let note = write(&writing)?;
        self.kept_in = Some((writing.path, writing.then));
        Ok(note)
    }
}

/// Writes what a save took: on the end of the file, or the whole file afresh.
fn write(writing: &Writing) -> Result<String, String> {
    let mut bytes = writing.header.clone();
    for record in &writing.records {
        write_record(&mut bytes, record);
    }
    if writing.fresh {
        crate::save::write_file(&writing.path, &bytes)?;
    } else {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().append(true).open(&writing.path).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    Ok(format!("sight's bake: {} regions{}, {:.1} MB", writing.records.len(), if writing.fresh { " (written afresh)" } else { " added" }, bytes.len() as f64 / 1e6))
}

fn write_record(bytes: &mut Vec<u8>, record: &Record) {
    bytes.extend_from_slice(&record.region.0.to_le_bytes());
    bytes.extend_from_slice(&record.region.1.to_le_bytes());
    for columns in &record.levels {
        bytes.extend_from_slice(&(columns.len() as u32).to_le_bytes());
        for (low, cells) in columns {
            bytes.extend_from_slice(&(*low as i16).to_le_bytes());
            bytes.extend_from_slice(&(cells.len() as u16).to_le_bytes());
            for cell in cells {
                bytes.extend_from_slice(&cell.to_bytes());
            }
        }
    }
    bytes.extend_from_slice(&record.top.low.to_le_bytes());
    bytes.extend_from_slice(&(record.top.sums.len() as u32).to_le_bytes());
    for sums in &record.top.sums {
        for v in sums.to_numbers() {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
}

/// Reads a file's bytes in order.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let slice = self.bytes.get(self.at..self.at + n).ok_or("the bake's file ends short")?;
        self.at += n;
        Ok(slice)
    }
}

/// A bake's file read back: its stamp, its map's half width, its records in order, and whether it was whole
/// (a record cut short, the game closed while writing, ends it there).
pub struct Kept {
    pub stamp: u64,
    pub half: i32,
    pub records: Vec<Record>,
    pub path: PathBuf,
    pub whole: bool,
}

/// Reads a bake's file (`None` if there's none).
pub fn read(folder: &Path) -> Result<Option<Kept>, String> {
    let path = folder.join(FILE);
    let Ok(bytes) = std::fs::read(&path) else { return Ok(None) };
    let mut r = Reader { bytes: &bytes, at: 0 };
    let first = r.take(8)?;
    if first != MAGIC {
        if first.starts_with(KIND) {
            // An older layout: baked again (the new bake writes its file afresh).
            return Ok(None);
        }
        return Err("the bake's file is of another kind".into());
    }
    let u16_of = |b: &[u8]| u16::from_le_bytes(b.try_into().unwrap_or_default());
    let i16_of = |b: &[u8]| i16::from_le_bytes(b.try_into().unwrap_or_default());
    let u32_of = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap_or_default());
    let i32_of = |b: &[u8]| i32::from_le_bytes(b.try_into().unwrap_or_default());
    let u64_of = |b: &[u8]| u64::from_le_bytes(b.try_into().unwrap_or_default());
    let stamp = u64_of(r.take(8)?);
    let half = i32_of(r.take(4)?);
    let mut records = Vec::new();
    let one = |r: &mut Reader| -> Result<Record, String> {
        let region = (i32_of(r.take(4)?), i32_of(r.take(4)?));
        let mut levels = Vec::new();
        for _ in FIRST..=REGION {
            let n = u32_of(r.take(4)?) as usize;
            let mut columns = Vec::with_capacity(n);
            for _ in 0..n {
                let low = i16_of(r.take(2)?) as i32;
                let count = u16_of(r.take(2)?) as usize;
                let mut cells = Vec::with_capacity(count);
                for _ in 0..count {
                    cells.push(Cell::from_bytes(r.take(16)?.try_into().unwrap_or_default()));
                }
                columns.push((low, cells));
            }
            levels.push(columns);
        }
        let low = i32_of(r.take(4)?);
        let n = u32_of(r.take(4)?) as usize;
        let mut sums = Vec::with_capacity(n);
        for _ in 0..n {
            let mut numbers = [0u64; 15];
            for v in &mut numbers {
                *v = u64_of(r.take(8)?);
            }
            sums.push(Sums::from_numbers(numbers));
        }
        Ok(Record { region, levels, top: CellColumn { low, sums } })
    };
    let mut whole = true;
    while r.at < bytes.len() {
        match one(&mut r) {
            Ok(record) => records.push(record),
            Err(_) => {
                whole = false;
                break;
            }
        }
    }
    Ok(Some(Kept { stamp, half, records, path, whole }))
}

/// What the bake needs besides the card: the world's map, blocks and far colours (for cell columns made
/// from the blocks), and the changed chunks as they were when it started (or when an edit was patched).
#[derive(Clone)]
pub struct Blocks {
    pub map: Arc<CompiledMap>,
    pub blocks: TerrainBlocks,
    pub colours: FarColours,
    /// Shared, so an edit copies only its own chunk (build 88).
    pub changed: Arc<HashMap<ChunkPos, Arc<Chunk>>>,
}

/// A cell column of 32 made from the blocks themselves (every one of its cells, then trimmed to its band:
/// solid all round below, empty above), the changed chunks as they are and the rest as the recipe makes
/// them. For edits: the recipe on the card doesn't know them.
pub fn column_from_blocks(cx: i32, cy: i32, with: &Blocks, caves: &crate::world::CavePlan) -> CellColumn {
    let size = 1 << FIRST;
    let made: RefCell<HashMap<ChunkPos, Option<Chunk>>> = RefCell::new(HashMap::new());
    let block = |[x, y, z]: [i32; 3]| -> BlockId {
        if z < 0 {
            return with.blocks.stone;
        }
        if z >= crate::world::WORLD_HEIGHT {
            return BlockId::AIR;
        }
        let pos = ChunkPos::new(x.div_euclid(32), y.div_euclid(32));
        if let Some(chunk) = with.changed.get(&pos) {
            // A cell of pieces as its main material (build 90, `edits::main_material`).
            let block = chunk.block_at(x, y, z);
            return if block == BlockId::PIECES { super::edits::main_material(chunk, [x, y, z]) } else { block };
        }
        let mut made = made.borrow_mut();
        let chunk = made.entry(pos).or_insert_with(|| with.map.chunk(pos, &with.blocks, caves));
        chunk.as_ref().map_or(BlockId::AIR, |chunk| chunk.block_at(x, y, z))
    };
    let all: Vec<Sums> = (0..crate::world::WORLD_HEIGHT / size).map(|z| sums_of_box([cx * size, cy * size, z * size], size, &block, &with.colours)).collect();
    let full = |s: &Sums| s.solid == (size as u64).pow(3) && s.showing() == 0;
    let low = all.iter().position(|s| !full(s)).unwrap_or(all.len()) as i32;
    let high = all.iter().rposition(|s| s.solid > 0 || s.water > 0).map_or(-1, |h| h as i32);
    CellColumn { low, sums: if high >= low { all[low as usize..=high as usize].to_vec() } else { Vec::new() } }
}

/// A region baked: its cells of 32 made on the card, those by changed chunks made from the blocks, and
/// the levels up to 1,024 added up from them.
pub fn bake_region(maker: &Maker, with: &Blocks, odd: bool, region: (i32, i32)) -> Record {
    let size = 1 << FIRST;
    let across = 1 << (REGION - FIRST);
    let side = 1 << REGION;
    let (x0, y0) = (region.0 * side, region.1 * side);
    let mut bands = Vec::with_capacity((across * across) as usize);
    let made = maker.world.cells_by(
        &maker.device,
        &maker.queue,
        [x0, y0, side, side],
        |grounds| {
            let column = |x: i32, y: i32| grounds[((y - y0 + 3) * (side + 6) + (x - x0 + 3)) as usize];
            bands_of(column, x0, y0, size, across, &maker.caves, odd, &mut bands)
        },
        BAKE_PIECES,
    );
    let mut made = made.iter().map(sums_of);
    let mut first: Vec<CellColumn> = bands.into_iter().map(|(low, count)| CellColumn { low, sums: (0..count).filter_map(|_| made.next()).collect() }).collect();
    // A changed chunk, or one beside it, and its cell column is made from the blocks.
    for (i, column) in first.iter_mut().enumerate() {
        let (cx, cy) = (region.0 * across + i as i32 % across, region.1 * across + i as i32 / across);
        if [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| with.changed.contains_key(&ChunkPos::new(cx + dx, cy + dy))) {
            *column = column_from_blocks(cx, cy, with, &maker.caves);
        }
    }
    Record::of(region, &build_up(first))
}

/// The regions an edit to `chunk` reaches (its own and those of the chunks beside it).
pub fn regions_of(chunk: ChunkPos) -> Vec<(i32, i32)> {
    let per = 1 << (REGION - FIRST);
    let mut regions: Vec<(i32, i32)> = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].iter().map(|(dx, dy)| ((chunk.x + dx).div_euclid(per), (chunk.y + dy).div_euclid(per))).collect();
    regions.sort();
    regions.dedup();
    regions
}

/// A bake under way: the bake, the regions still to do (nearest first), the switch to stop it, and its saves
/// being written (one at a time, in turn).
pub struct Baking {
    pub bake: Mutex<Bake>,
    queue: Mutex<VecDeque<(i32, i32)>>,
    /// What was read back with the world, still to go on the card (`take_kept`, a few regions a frame).
    kept: Mutex<VecDeque<Record>>,
    pub stop: AtomicBool,
    writing: Mutex<()>,
    writes: Mutex<Vec<crate::loading::Job<()>>>,
}

impl Baking {
    /// Whether a save is still being written.
    pub fn writing(&self) -> bool {
        self.writes.lock().map(|writes| writes.iter().any(|job| matches!(job.ready(), Ok(None)))).unwrap_or(false)
    }
}

/// Keeps the bake in `folder` on the loading lane: what's to be written is taken in a moment (the records
/// shared, not copied), written in turn after any save before it [Counsel 2026-11-06: a whole world's bake,
/// some 300 MB, written on the frame would freeze it]. A save the game closes in the middle of leaves the
/// file cut short; it's read back as far as it's whole, and written afresh at the next save.
pub fn keep_later(baking: &Arc<Baking>, folder: PathBuf) -> String {
    let baking_for_job = baking.clone();
    let job = crate::loading::submit(crate::loading::Tier::Soon, 0.0, move || {
        let Ok(_turn) = baking_for_job.writing.lock() else { return };
        let Some(writing) = baking_for_job.bake.lock().ok().map(|bake| bake.to_write(&folder)) else { return };
        match write(&writing) {
            Ok(_) => {
                if let Ok(mut bake) = baking_for_job.bake.lock() {
                    bake.kept_in = Some((writing.path, writing.then));
                }
            }
            Err(_) => {
                if let Ok(mut bake) = baking_for_job.bake.lock() {
                    bake.kept_in = None;
                }
            }
        }
    });
    if let Ok(mut writes) = baking.writes.lock() {
        writes.retain(|job| matches!(job.ready(), Ok(None)));
        writes.push(job);
    }
    "sight's bake: being written on the loading lane".into()
}

/// Starts baking the map (`half` blocks either side of the middle) outward from `spawn`: what `kept` holds
/// (read back, if its stamp is `stamp`) taken first, then `BAKERS` loops on the loading lane, each taking
/// the next region not yet baked till there are none or it's told to stop.
#[allow(clippy::too_many_arguments)]
pub fn start(maker: Maker, with: Blocks, odd: bool, half: i32, spawn: (i32, i32), stamp: u64, kept: Option<Kept>) -> (Arc<Baking>, Vec<crate::loading::Job<()>>) {
    let regions_from = (-half).div_euclid(1 << REGION);
    let regions_to = (half - 1).div_euclid(1 << REGION);
    let centre = (spawn.0.div_euclid(1 << REGION), spawn.1.div_euclid(1 << REGION));
    let mut all: Vec<(i32, (i32, i32))> = Vec::new();
    for ry in regions_from..=regions_to {
        for rx in regions_from..=regions_to {
            all.push(((rx - centre.0).abs().max((ry - centre.1).abs()), (rx, ry)));
        }
    }
    all.sort_by_key(|&(ring, (rx, ry))| (ring, ry, rx));
    // The start bake: the smallest square of rings round the spawn holding an eighth of the regions.
    let eighth = all.len().div_ceil(8);
    let start_ring = all.get(eighth.saturating_sub(1)).map_or(0, |&(ring, _)| ring);
    let start_regions = all.iter().filter(|&&(ring, _)| ring <= start_ring).count();
    let extents = (FIRST..=Bake::top(half)).map(|level| Extent::of(level, half)).collect();
    let mut bake = Bake {
        extents,
        upper: HashMap::new(),
        sums: HashMap::new(),
        regions: all.len(),
        start_regions,
        regions_done: 0,
        started: Instant::now(),
        start_done: None,
        all_done: None,
        records: HashMap::new(),
        order: Vec::new(),
        kept_in: None,
        stamp,
        half,
    };
    // What was read back goes on the card a few regions a frame (`take_kept`), not all at once [2026-11-09, the
    // dyno's test world read back whole: 5,184 regions put at once, some 64 writes each, all waiting for the frame's
    // hand-over, and the card ran out of memory for them].
    let mut read_back: VecDeque<Record> = VecDeque::new();
    if let Some(kept) = kept.filter(|kept| kept.stamp == stamp && kept.half == half) {
        let count = kept.records.len();
        read_back = kept.records.into_iter().collect();
        // What was read back is in the file already (unless it was cut short: then it's written afresh).
        bake.kept_in = kept.whole.then_some((kept.path, count));
    }
    let reading: HashSet<(i32, i32)> = read_back.iter().map(|record| record.region).collect();
    // Regions not baked yet nor read back, and those holding changed chunks (made again from their blocks, whatever
    // the file had: a save may have caught a patch still under way).
    let touched: HashSet<(i32, i32)> = with.changed.keys().flat_map(|&chunk| regions_of(chunk)).collect();
    let queue: VecDeque<(i32, i32)> = all.into_iter().map(|(_, r)| r).filter(|r| (!bake.is_baked(*r) && !reading.contains(r)) || touched.contains(r)).collect();
    let baking = Arc::new(Baking { bake: Mutex::new(bake), queue: Mutex::new(queue), kept: Mutex::new(read_back), stop: AtomicBool::new(false), writing: Mutex::new(()), writes: Mutex::new(Vec::new()) });
    let jobs = (0..BAKERS)
        .map(|_| {
            let (baking, maker, with) = (baking.clone(), maker.clone(), with.clone());
            crate::loading::submit(crate::loading::Tier::Whenever, 0.0, move || {
                while !baking.stop.load(Ordering::Relaxed) {
                    let Some(region) = baking.queue.lock().ok().and_then(|mut queue| queue.pop_front()) else { break };
                    // Its card work counts as filling (the frame leaves it its share of the card).
                    let record = crate::runner::baking(|| bake_region(&maker, &with, odd, region));
                    let Ok(mut bake) = baking.bake.lock() else { break };
                    bake.take(record, maker.card.as_deref());
                }
            })
        })
        .collect();
    (baking, jobs)
}

/// Bakes `regions` again with the changed chunks as they are now (an edit): on the loading lane.
pub fn patch(baking: Arc<Baking>, maker: Maker, with: Blocks, odd: bool, regions: Vec<(i32, i32)>) -> crate::loading::Job<()> {
    crate::loading::submit(crate::loading::Tier::Whenever, 0.0, move || {
        for region in regions {
            if baking.stop.load(Ordering::Relaxed) {
                return;
            }
            // Only regions already baked: the others will be, with the changes, when their turn comes.
            if !baking.bake.lock().map(|bake| bake.is_baked(region)).unwrap_or(false) {
                continue;
            }
            let record = bake_region(&maker, &with, odd, region);
            if let Ok(mut bake) = baking.bake.lock() {
                bake.take(record, maker.card.as_deref());
            }
        }
    })
}

/// Up to `most` of the regions read back with the world put on the card (each frame, by the frame's thread, so the
/// card's writes are handed over in between). One baked meanwhile from changed chunks is newer: kept, not the file's.
pub fn take_kept(baking: &Baking, card: Option<&super::card_store::CardStore>, most: usize) {
    for _ in 0..most {
        let Some(record) = baking.kept.lock().ok().and_then(|mut kept| kept.pop_front()) else { return };
        let Ok(mut bake) = baking.bake.lock() else { return };
        if !bake.is_baked(record.region) {
            bake.take(record, card);
        }
    }
}

/// Every bake under way stops (a new world, or the game closing).
pub fn stop(baking: &Baking) {
    baking.stop.store(true, Ordering::Relaxed);
}

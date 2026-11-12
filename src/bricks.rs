//! The far bricks (plan 2.2, step 2): the far cells on the card, for the far eye to read.
//!
//! Each level (cells of 32 up to 65,536) is laid over the whole map:
//! - **Each cell column's band** (`entries`): its lowest kept cell and how many, as old kept it, or "not summed yet".
//!   Below the band every cell is solid with no face, above it empty (`far::CellColumn`).
//! - **The cells of the bands, in bricks** (`pool`): a brick is 4 × 4 cell columns, 4 cells tall, where the level is
//!   that big (or 8 or 2 a side, chosen at launch: plan 2.2, step 3, `brick_side`). A brick stays within its region,
//!   so a region's arrival makes its bricks whole: at 4, 2 × 2 at 512, 1 at 1,024. A brick is found by
//!   arithmetic, its place in the level's index (`index`), and a cell by its place in the brick. Only bricks holding
//!   some band are made.
//!
//! The regions are summed on the loading lane (`far::Lane`), each brought here as it comes and put on the card at once,
//! so the far view sharpens in live. The levels over 1,024 are made as soon as the regions under a cell are all in.
//! The pool grows as it fills (copied on the card into one twice the size): the whole world stays on the card, as
//! old's bake did.

use crate::far::{Cell, CellColumn, FIRST, REGION};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// The coarsest level: four cells cover the whole map (cells of 65,536).
pub const TOP: usize = 16;
/// The levels, from cells of 32 to 65,536.
pub const LEVELS: usize = TOP - FIRST + 1;
/// Numbers a level takes in the card's table (`far.wgsl`'s `level_*`).
pub const TABLE: usize = 12;
/// A column not summed yet.
pub const NOT_SUMMED: u32 = 0xFFFF_0000;
/// No brick at a place in the index.
pub const NO_BRICK: u32 = u32::MAX;

/// A brick's side in cells: 4, or 8 or 2 at launch for measuring (`INCUS_FAR_BRICK`). Plan 2.2, step 3: smaller bricks
/// hold less beside the bands, so the card holds less; build 131 flew them, 4 taking the whole game from 2,656 to 1,555
/// MB for 5 to 13% more far time, and David: "4 sounds good". The last pick waits on the window round the eye (step 4),
/// flown at 4 and 8.
pub fn brick_side() -> i32 {
    match std::env::var("INCUS_FAR_BRICK").ok().and_then(|s| s.parse().ok()) {
        Some(side @ (2 | 4 | 8)) => side,
        _ => 4,
    }
}

/// The levels kept only in a window round the eye (plan 2.2, step 4): the 32s and the 64s, which hold 93% of the far
/// bricks and are read only near the eye.
pub const WINDOWED: usize = 2;

/// The window's radius at the 32s, in regions (the 64s' is twice it). At detail 0 on David's 2560 × 1600 the 8 × 8 layer
/// reads 32s out to about 9 km and 64s to about 18.
pub const WINDOW: i32 = 10;

/// The window's radius at the 32s in regions: `WINDOW`, or as `INCUS_FAR_WINDOW` says; 0 the whole world on the card,
/// as before.
pub fn window_regions() -> i32 {
    std::env::var("INCUS_FAR_WINDOW").ok().and_then(|s| s.parse().ok()).filter(|r: &i32| *r >= 0).unwrap_or(WINDOW)
}

/// A level laid over the map: its columns, how tall it is, its bricks' shape, and where its columns and bricks are kept.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub level: usize,
    /// Its least column, north and east, and how many across each way.
    pub from: [i32; 2],
    pub across: i32,
    /// Cells up a column (the world's height over its cells, at least one).
    pub up: i32,
    /// A brick's columns a side, and its cells tall.
    pub wide: i32,
    pub tall: i32,
    /// Bricks a side, and bricks up.
    pub bricks: i32,
    pub bricks_up: i32,
    /// Where its columns start in `entries`, and its bricks' places in `index`.
    pub entries_at: u32,
    pub index_at: u32,
}

impl Level {
    /// Cells a brick holds.
    pub fn brick_cells(&self) -> u32 {
        (self.wide * self.wide * self.tall) as u32
    }

    /// Where a column is in `entries` (none outside the level).
    pub fn entry(&self, cx: i32, cy: i32) -> Option<u32> {
        let (x, y) = (cx - self.from[0], cy - self.from[1]);
        ((0..self.across).contains(&x) && (0..self.across).contains(&y)).then(|| self.entries_at + (y * self.across + x) as u32)
    }

    /// A brick's place in `index`, by the brick's coordinates in the level (from its least corner).
    pub fn brick(&self, bx: i32, by: i32, bz: i32) -> u32 {
        self.index_at + ((by * self.bricks + bx) * self.bricks_up + bz) as u32
    }
}

/// The levels laid over a map whose regions run `regions` (least region, north and east, and how many across), in bricks
/// `side` cells a side.
pub fn levels(regions: ([i32; 2], i32), height: i32, side: i32) -> Vec<Level> {
    let (from, across) = regions;
    let (mut entries_at, mut index_at) = (0u32, 0u32);
    (FIRST..=TOP)
        .map(|level| {
            let (from, across) = if level <= REGION {
                let per = 1 << (REGION - level);
                ([from[0] * per, from[1] * per], across * per)
            } else {
                let per = 1 << (level - REGION);
                let least = [from[0].div_euclid(per), from[1].div_euclid(per)];
                (least, (from[0] + across - 1).div_euclid(per) - least[0] + 1)
            };
            let size = 1 << level;
            let up = (height + size - 1) / size;
            let wide = if level <= REGION { side.min(1 << (REGION - level)) } else { side };
            let tall = side.min(up);
            let bricks = (across + wide - 1) / wide;
            let bricks_up = (up + tall - 1) / tall;
            let made = Level { level, from, across, up, wide, tall, bricks, bricks_up, entries_at, index_at };
            entries_at += (across * across) as u32;
            index_at += (bricks * bricks * bricks_up) as u32;
            made
        })
        .collect()
}

/// The card's table of the levels: `TABLE` numbers each (`far.wgsl` reads them by these places; a brick's width and
/// height as shifts, both powers of two).
pub fn table(levels: &[Level]) -> Vec<i32> {
    let table: Vec<i32> = levels
        .iter()
        .flat_map(|l| [l.level as i32, l.from[0], l.from[1], l.across, l.entries_at as i32, l.index_at as i32, l.wide.trailing_zeros() as i32, l.tall.trailing_zeros() as i32, l.bricks, l.bricks_up, l.up, 0])
        .collect();
    debug_assert_eq!(table.len(), levels.len() * TABLE);
    table
}

/// A band's entry: its lowest cell (16 bits, signed) and how many (16 bits).
pub fn entry_of(column: &CellColumn) -> u32 {
    (column.low as i16 as u16 as u32) | ((column.sums.len() as u32) << 16)
}


/// What some columns of a level put on the card: their entries, and the bricks holding their bands.
#[derive(Default)]
pub struct Part {
    /// Runs of entries: where the run starts in `entries`, and its words.
    pub entries: Vec<(u32, Vec<u32>)>,
    /// Bricks: its place in `index`, and its cells (a whole brick's, empty where no band is).
    pub bricks: Vec<(u32, Vec<Cell>)>,
}

/// The entries and bricks (if `bricks`) of the columns `cx0..cx0 + n`, `cy0..cy0 + n` of `level` (they must cover whole
/// bricks), each column found by `column(cx, cy)` (none: not summed yet).
pub fn part<'a>(level: &Level, cx0: i32, cy0: i32, n: i32, column: impl Fn(i32, i32) -> Option<&'a CellColumn>, bricks: bool) -> Part {
    let mut part = Part::default();
    let size = 1u32 << level.level;
    // The entries, a row at a time, only as far as the level's columns go (a brick at its edge reaches past them: build
    // 121 wrote those into the next row, and at the last row off the end of the bands, and the card refused it).
    let last = (cx0 + n).min(level.from[0] + level.across);
    for cy in cy0..cy0 + n {
        let Some(start) = level.entry(cx0, cy) else { continue };
        let row = (cx0..last).map(|cx| column(cx, cy).map_or(NOT_SUMMED, entry_of)).collect();
        part.entries.push((start, row));
    }
    if !bricks {
        return part;
    }
    let cells = level.brick_cells() as usize;
    let first = |c0: i32, from: i32| (c0 - from).div_euclid(level.wide);
    for by in first(cy0, level.from[1])..=first(cy0 + n - 1, level.from[1]) {
        for bx in first(cx0, level.from[0])..=first(cx0 + n - 1, level.from[0]) {
            for bz in 0..level.bricks_up {
                let (z0, z1) = (bz * level.tall, ((bz + 1) * level.tall).min(level.up));
                let mut brick: Option<Vec<Cell>> = None;
                for j in 0..level.wide {
                    for i in 0..level.wide {
                        let Some(c) = column(level.from[0] + bx * level.wide + i, level.from[1] + by * level.wide + j) else { continue };
                        for cz in c.low.max(z0)..(c.high() + 1).min(z1) {
                            let at = (i + j * level.wide + (cz - z0) * level.wide * level.wide) as usize;
                            brick.get_or_insert_with(|| vec![Cell::default(); cells])[at] = c.at(cz, size as i32).compact(size);
                        }
                    }
                }
                if let Some(brick) = brick {
                    part.bricks.push((level.brick(bx, by, bz), brick));
                }
            }
        }
    }
    part
}

/// A region's parts, level by level from 32 to 1,024 (its columns from `far::region`), the first `windowed` levels' bands
/// only.
pub fn region_parts(levels: &[Level], region: [i32; 2], columns: &[Vec<CellColumn>], windowed: usize) -> Vec<Part> {
    (FIRST..=REGION)
        .map(|level| {
            let n = 1 << (REGION - level);
            let these = &columns[level - FIRST];
            part(&levels[level - FIRST], region[0] * n, region[1] * n, n, |cx, cy| these.get(((cy - region[1] * n) * n + (cx - region[0] * n)) as usize), level >= FIRST + windowed)
        })
        .collect()
}

/// Bytes a cell takes on the card.
const CELL_BYTES: u64 = 16;
/// The pool's first size: 128 MB.
const FIRST_POOL: u64 = (128 << 20) / CELL_BYTES;

/// Blocks a region a side.
const REGION_BLOCKS: f32 = (1 << REGION) as f32;

/// How far `eye` is from a region's middle (blocks).
fn region_distance(region: [i32; 2], eye: [f32; 2]) -> f32 {
    let middle = [(region[0] as f32 + 0.5) * REGION_BLOCKS, (region[1] as f32 + 0.5) * REGION_BLOCKS];
    ((middle[0] - eye[0]).powi(2) + (middle[1] - eye[1]).powi(2)).sqrt()
}

/// What the window's move put on the card and gave back, for the atlas to follow: the bricks that came, and the places
/// of those that left.
#[derive(Default)]
pub struct Moved {
    pub came: Vec<crate::atlas::Brick>,
    pub left: Vec<u32>,
}

/// The far bricks on the card, and what's kept here: to make the levels over 1,024 as their regions come in, and to
/// bring the window's regions onto the card as the eye moves.
pub struct Store {
    /// A brick's side in cells.
    pub side: i32,
    pub levels: Vec<Level>,
    pub table: wgpu::Buffer,
    pub entries: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub pool: wgpu::Buffer,
    /// Cells the pool holds, and those used (its places given back included).
    capacity: u64,
    used: u64,
    /// Bumped whenever the pool is made again (bind groups made with the old one are stale).
    pub generation: u32,
    /// Each region's 1,024 as it comes in, and each level over 1,024's columns as they're made.
    tops: HashMap<[i32; 2], CellColumn>,
    upper: Vec<HashMap<[i32; 2], CellColumn>>,
    /// The far cells kept as they come: the window's regions are made from them as the eye moves, and the atlas
    /// (change 1). Shared only once every region is in.
    pub kept: Arc<crate::atlas::Kept>,
    /// Regions in, the highest anything stands (blocks), and the bricks on the card.
    pub regions_in: usize,
    pub highest: f32,
    pub bricks: usize,
    /// The map's regions: the least, north and east, and how many across.
    pub regions: ([i32; 2], i32),
    /// The window round the eye (step 4): its radius in regions at the 32s and the 64s (none: the whole world on the
    /// card); each windowed level's regions on the card with their bricks (each its place in the index and in the
    /// pool); each level's places in the pool given back; the regions summed; and the regions wanted but not yet on the
    /// card at the last move.
    window: Option<[i32; WINDOWED]>,
    on_card: Vec<HashMap<[i32; 2], Vec<(u32, u64)>>>,
    free: Vec<Vec<u64>>,
    summed: HashSet<[i32; 2]>,
    pub behind: usize,
    /// Each windowed level's regions, usable or not (it and its eight neighbours on the card), as the card reads them
    /// (`usable`: a word a region, the 32s' then the 64s').
    flags: Vec<u32>,
    pub usable: wgpu::Buffer,
}

impl Store {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, regions: ([i32; 2], i32), height: i32, side: i32, window: i32) -> Self {
        let levels = levels(regions, height, side);
        let table = table(&levels);
        let entries_count = levels.last().map_or(0, |l| l.entries_at + (l.across * l.across) as u32) as u64;
        let index_count = levels.last().map_or(0, |l| l.index_at + (l.bricks * l.bricks * l.bricks_up) as u32) as u64;
        let storage = |label: &str, size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: size.max(16), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let table_buffer = storage("the far levels", table.len() as u64 * 4);
        queue.write_buffer(&table_buffer, 0, &table.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let entries = storage("the far bands", entries_count * 4);
        queue.write_buffer(&entries, 0, &NOT_SUMMED.to_le_bytes().repeat(entries_count as usize));
        let index = storage("the far bricks' index", index_count * 4);
        queue.write_buffer(&index, 0, &NO_BRICK.to_le_bytes().repeat(index_count as usize));
        let pool = storage("the far bricks", FIRST_POOL * CELL_BYTES);
        let window = (window > 0).then_some([window, window * 2]);
        let flags = vec![u32::from(window.is_none()); WINDOWED * (regions.1 * regions.1) as usize];
        let usable = storage("the far window's usable regions", flags.len() as u64 * 4);
        queue.write_buffer(&usable, 0, &flags.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        log::info!(
            "The far bricks: {} levels, bricks of {side}, {:.1} MB of bands and {:.1} MB of index on the card; the pool {} MB to start; {}",
            levels.len(),
            (entries_count * 4) as f64 / 1e6,
            (index_count * 4) as f64 / 1e6,
            (FIRST_POOL * CELL_BYTES) >> 20,
            window.map_or("the whole world on the card".to_string(), |w| format!("the 32s within {} regions of the eye, the 64s within {}", w[0], w[1]))
        );
        let kept = Arc::new(crate::atlas::Kept::new(&levels));
        Self {
            side,
            kept,
            levels,
            table: table_buffer,
            entries,
            index,
            pool,
            capacity: FIRST_POOL,
            used: 0,
            generation: 0,
            tops: HashMap::new(),
            upper: vec![HashMap::new(); TOP - REGION],
            regions_in: 0,
            highest: 0.0,
            bricks: 0,
            regions,
            window,
            on_card: vec![HashMap::new(); WINDOWED],
            free: vec![Vec::new(); WINDOWED],
            summed: HashSet::new(),
            behind: 0,
            flags,
            usable,
        }
    }

    /// The window's radius in regions at the 32s and the 64s (none: the whole world on the card).
    pub fn window(&self) -> Option<[i32; WINDOWED]> {
        self.window
    }

    /// The pool's bytes in use (its places given back not counted), and its size.
    pub fn pool_bytes(&self) -> (u64, u64) {
        let given_back: u64 = self.free.iter().enumerate().map(|(k, free)| free.len() as u64 * self.levels[k].brick_cells() as u64).sum();
        ((self.used - given_back) * CELL_BYTES, self.capacity * CELL_BYTES)
    }

    /// Room for `cells` more at the pool's end. If it's full it's grown: the old pool copied into the new and handed to
    /// the card at once, so the writes made before land in the old and are carried over, and those after land in the new.
    fn reserve(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, cells: u64) {
        if self.used + cells <= self.capacity {
            return;
        }
        let limit = device.limits().max_storage_buffer_binding_size.min(device.limits().max_buffer_size) / CELL_BYTES;
        let mut capacity = self.capacity;
        while self.used + cells > capacity && capacity < limit {
            capacity = (capacity * 2).min(limit);
        }
        assert!(self.used + cells <= capacity, "the far bricks don't fit in the card's largest buffer ({} MB)", (limit * CELL_BYTES) >> 20);
        let pool = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the far bricks"), size: capacity * CELL_BYTES, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far bricks, grown") });
        encoder.copy_buffer_to_buffer(&self.pool, 0, &pool, 0, self.used * CELL_BYTES);
        queue.submit([encoder.finish()]);
        log::info!("The far bricks' pool grown to {} MB ({} MB used)", (capacity * CELL_BYTES) >> 20, (self.used * CELL_BYTES) >> 20);
        self.pool = pool;
        self.capacity = capacity;
        self.generation += 1;
    }

    /// A part put on the card: its entries written, its bricks given places at the pool's end and written there, and
    /// their places written in the index.
    fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, part: &Part) {
        for (start, words) in &part.entries {
            queue.write_buffer(&self.entries, *start as u64 * 4, &words.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        }
        let cells: u64 = part.bricks.iter().map(|(_, cells)| cells.len() as u64).sum();
        if cells == 0 {
            return;
        }
        self.reserve(device, queue, cells);
        let mut bytes = Vec::with_capacity((cells * CELL_BYTES) as usize);
        let start = self.used;
        for (place, brick) in &part.bricks {
            let at = start + bytes.len() as u64 / CELL_BYTES;
            queue.write_buffer(&self.index, *place as u64 * 4, &(at as u32).to_le_bytes());
            for cell in brick {
                bytes.extend_from_slice(&cell_bytes(cell));
            }
        }
        queue.write_buffer(&self.pool, start * CELL_BYTES, &bytes);
        self.used += cells;
        self.bricks += part.bricks.len();
    }

    /// A region's columns (`far::region`'s, 32 to 1,024) put on the card, and the levels over 1,024 made where all the
    /// regions under them are in. With a window, the 32s' and 64s' bands are written but not their bricks: those come
    /// with the window (`follow`).
    pub fn put_region(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, region: [i32; 2], columns: &[Vec<CellColumn>]) {
        let windowed = if self.window.is_some() { WINDOWED } else { 0 };
        for part in region_parts(&self.levels, region, columns, windowed) {
            self.put(device, queue, &part);
        }
        let kept = Arc::get_mut(&mut self.kept).expect("the far cells kept are shared only once every region is in");
        for level in FIRST..=REGION {
            let n = 1 << (REGION - level);
            let size = 1u32 << level;
            for (k, column) in columns[level - FIRST].iter().enumerate() {
                let (cx, cy) = (region[0] * n + k as i32 % n, region[1] * n + k as i32 / n);
                kept.keep(level, cx, cy, entry_of(column), column.sums.iter().map(|s| s.compact(size)).collect());
            }
        }
        // The highest anything stands: the top of the highest band among its cells of 32.
        let top = columns[0].iter().map(|c| (c.high() + 1) * (1 << FIRST)).max().unwrap_or(0);
        self.highest = self.highest.max(top as f32);
        self.regions_in += 1;
        self.summed.insert(region);
        self.tops.insert(region, columns[REGION - FIRST][0].clone());
        let mut place = region;
        for level in REGION + 1..=TOP {
            let parent = [place[0].div_euclid(2), place[1].div_euclid(2)];
            let children: Option<Vec<CellColumn>> = (0..4).map(|k| self.child(level - 1, [parent[0] * 2 + (k & 1), parent[1] * 2 + (k >> 1)])).collect();
            let Some(children) = children else { break };
            let made = crate::far::parent_of([&children[0], &children[1], &children[2], &children[3]], 1 << (level - 1));
            self.upper[level - REGION - 1].insert(parent, made);
            self.put_upper(device, queue, level, parent);
            place = parent;
        }
    }

    /// A column at `level` (1,024 or over), for the level over it: as it is if it's in, empty if it's outside the
    /// regions, none if it isn't summed yet.
    fn child(&self, level: usize, at: [i32; 2]) -> Option<CellColumn> {
        if self.levels[level - FIRST].entry(at[0], at[1]).is_none() {
            return Some(CellColumn::default());
        }
        if level == REGION { self.tops.get(&at).cloned() } else { self.upper[level - REGION - 1].get(&at).cloned() }
    }

    /// The brick holding `place` at a level over 1,024 made again from its columns as they are, and put on the card. A
    /// brick made again takes a new place in the pool (its old one is left: a few small bricks at most).
    fn put_upper(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, level: usize, place: [i32; 2]) {
        let l = self.levels[level - FIRST];
        let (bx, by) = ((place[0] - l.from[0]).div_euclid(l.wide), (place[1] - l.from[1]).div_euclid(l.wide));
        let columns = std::mem::take(&mut self.upper[level - REGION - 1]);
        let part = part(&l, l.from[0] + bx * l.wide, l.from[1] + by * l.wide, l.wide, |cx, cy| columns.get(&[cx, cy]), true);
        let size = 1u32 << level;
        let kept = Arc::get_mut(&mut self.kept).expect("the far cells kept are shared only once every region is in");
        for (at, column) in columns.iter().filter(|(at, _)| (at[0] - l.from[0]).div_euclid(l.wide) == bx && (at[1] - l.from[1]).div_euclid(l.wide) == by) {
            kept.keep(level, at[0], at[1], entry_of(column), column.sums.iter().map(|s| s.compact(size)).collect());
        }
        self.upper[level - REGION - 1] = columns;
        self.put(device, queue, &part);
    }

    /// The window brought to the eye (step 4), for at most `ms`. First the regions past it are given back (one region
    /// past its radius, so its edge doesn't flicker in and out): their places in the index emptied, their places in the
    /// pool kept for the next to come. Then the summed regions within it not on the card are made from the cells kept
    /// and put there, nearest first, the 32s and the 64s together. The usable flags are written after. What came and
    /// what left, for the atlas to follow.
    pub fn follow(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, eye: [f32; 2], ms: f32) -> Moved {
        let mut moved = Moved::default();
        let Some(window) = self.window else { return moved };
        let started = Instant::now();
        let mut changed = false;
        let mut wanted: Vec<(f32, usize, [i32; 2])> = Vec::new();
        let middle = [(eye[0] / REGION_BLOCKS).floor() as i32, (eye[1] / REGION_BLOCKS).floor() as i32];
        for k in 0..WINDOWED {
            let reach = window[k] as f32 * REGION_BLOCKS;
            let leaving: Vec<[i32; 2]> = self.on_card[k].keys().filter(|g| region_distance(**g, eye) > reach + REGION_BLOCKS).copied().collect();
            for g in leaving {
                for (place, start) in self.on_card[k].remove(&g).unwrap_or_default() {
                    queue.write_buffer(&self.index, place as u64 * 4, &NO_BRICK.to_le_bytes());
                    self.free[k].push(start);
                    self.bricks -= 1;
                    moved.left.push(place);
                }
                changed = true;
            }
            let r = window[k] + 1;
            for gy in middle[1] - r..=middle[1] + r {
                for gx in middle[0] - r..=middle[0] + r {
                    let d = region_distance([gx, gy], eye);
                    if d <= reach && self.summed.contains(&[gx, gy]) && !self.on_card[k].contains_key(&[gx, gy]) {
                        wanted.push((d, k, [gx, gy]));
                    }
                }
            }
        }
        wanted.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut brought = 0;
        for &(_, k, g) in &wanted {
            if started.elapsed().as_secs_f32() * 1000.0 >= ms {
                break;
            }
            let level = FIRST + k;
            let cells = self.levels[k].brick_cells() as u64;
            let mut placed = Vec::new();
            for (place, origin, brick) in self.region_bricks(level, g) {
                let start = match self.free[k].pop() {
                    Some(start) => start,
                    None => {
                        self.reserve(device, queue, cells);
                        self.used += cells;
                        self.used - cells
                    }
                };
                let bytes: Vec<u8> = brick.iter().flat_map(cell_bytes).collect();
                queue.write_buffer(&self.pool, start * CELL_BYTES, &bytes);
                queue.write_buffer(&self.index, place as u64 * 4, &(start as u32).to_le_bytes());
                placed.push((place, start));
                moved.came.push(crate::atlas::Brick { level, place, origin });
                self.bricks += 1;
            }
            self.on_card[k].insert(g, placed);
            changed = true;
            brought += 1;
        }
        self.behind = wanted.len() - brought;
        if changed {
            self.write_flags(queue);
        }
        moved
    }

    /// A windowed region's bricks at `level`, made from the cells kept: each brick's place in the index, its least
    /// cell, and its cells (only bricks holding some band, as `part` makes them).
    fn region_bricks(&self, level: usize, region: [i32; 2]) -> Vec<(u32, [i32; 3], Vec<Cell>)> {
        let l = &self.levels[level - FIRST];
        let n = 1 << (REGION - level);
        let (cx0, cy0) = (region[0] * n, region[1] * n);
        let cells = l.brick_cells() as usize;
        let columns = &self.kept.columns[level - FIRST];
        let mut out = Vec::new();
        for by in 0..n / l.wide {
            for bx in 0..n / l.wide {
                for bz in 0..l.bricks_up {
                    let (z0, z1) = (bz * l.tall, ((bz + 1) * l.tall).min(l.up));
                    let mut brick: Option<Vec<Cell>> = None;
                    for j in 0..l.wide {
                        for i in 0..l.wide {
                            let Some(at) = l.entry(cx0 + bx * l.wide + i, cy0 + by * l.wide + j) else { continue };
                            let (entry, band) = &columns[(at - l.entries_at) as usize];
                            if *entry == NOT_SUMMED {
                                continue;
                            }
                            let low = (*entry & 0xFFFF) as u16 as i16 as i32;
                            for cz in low.max(z0)..(low + band.len() as i32).min(z1) {
                                let at = (i + j * l.wide + (cz - z0) * l.wide * l.wide) as usize;
                                brick.get_or_insert_with(|| vec![Cell::default(); cells])[at] = band[(cz - low) as usize];
                            }
                        }
                    }
                    if let Some(brick) = brick {
                        let (gx, gy) = ((cx0 - l.from[0]) / l.wide + bx, (cy0 - l.from[1]) / l.wide + by);
                        out.push((l.brick(gx, gy, bz), [l.from[0] + gx * l.wide, l.from[1] + gy * l.wide, bz * l.tall], brick));
                    }
                }
            }
        }
        out
    }

    /// The usable flags worked out again and written: a windowed level's region is usable where it and its eight
    /// neighbours (those on the map) are on the card, since a blend reaches a cell past its own region.
    fn write_flags(&mut self, queue: &wgpu::Queue) {
        let (from, across) = self.regions;
        let on_map = |g: [i32; 2]| (from[0]..from[0] + across).contains(&g[0]) && (from[1]..from[1] + across).contains(&g[1]);
        let mut flags = vec![0u32; self.flags.len()];
        for (k, on_card) in self.on_card.iter().enumerate() {
            for g in on_card.keys() {
                let usable = (-1..=1).all(|dy| (-1..=1).all(|dx| {
                    let n = [g[0] + dx, g[1] + dy];
                    !on_map(n) || on_card.contains_key(&n)
                }));
                if usable && on_map(*g) {
                    flags[k * (across * across) as usize + ((g[1] - from[1]) * across + (g[0] - from[0])) as usize] = 1;
                }
            }
        }
        self.flags = flags;
        queue.write_buffer(&self.usable, 0, &self.flags.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    }

    /// Whether a windowed level's region is usable (always, without a window).
    fn usable_at(&self, k: usize, g: [i32; 2]) -> bool {
        let (from, across) = self.regions;
        let (x, y) = (g[0] - from[0], g[1] - from[1]);
        self.window.is_none() || ((0..across).contains(&x) && (0..across).contains(&y) && self.flags[k * (across * across) as usize + (y * across + x) as usize] != 0)
    }

    /// The windowed levels' bricks on the card (their places in the index), for the atlas made from the window as it
    /// is; none without a window (every brick is on the card).
    pub fn window_places(&self) -> Option<HashSet<u32>> {
        self.window.map(|_| self.on_card.iter().flat_map(|level| level.values().flatten().map(|(place, _)| *place)).collect())
    }

    /// How far round the eye the usable regions reach at the 32s and the 64s (blocks): to the nearest edge of a region
    /// on the map that isn't usable, looked for out to two regions past the window (none without a window).
    pub fn usable_reach(&self, eye: [f32; 2]) -> Option<[f32; WINDOWED]> {
        let window = self.window?;
        let (from, across) = self.regions;
        let middle = [(eye[0] / REGION_BLOCKS).floor() as i32, (eye[1] / REGION_BLOCKS).floor() as i32];
        Some(std::array::from_fn(|k| {
            let r = window[k] + 2;
            let mut least = r as f32 * REGION_BLOCKS;
            for gy in middle[1] - r..=middle[1] + r {
                for gx in middle[0] - r..=middle[0] + r {
                    let on_map = (from[0]..from[0] + across).contains(&gx) && (from[1]..from[1] + across).contains(&gy);
                    if !on_map || self.usable_at(k, [gx, gy]) {
                        continue;
                    }
                    let edge = |g: i32, e: f32| (g as f32 * REGION_BLOCKS - e).max(e - (g + 1) as f32 * REGION_BLOCKS).max(0.0);
                    least = least.min((edge(gx, eye[0]).powi(2) + edge(gy, eye[1]).powi(2)).sqrt());
                }
            }
            least
        }))
    }
}

/// A cell's 16 bytes, as the card reads them (`far.wgsl`'s `cell_of`).
pub fn cell_bytes(cell: &Cell) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0] = cell.solid;
    b[1] = cell.water;
    b[2..7].copy_from_slice(&cell.faces);
    b[7] = cell.light_spread;
    b[8..12].copy_from_slice(&cell.colour.to_le_bytes());
    b[12..14].copy_from_slice(&cell.height_spread.to_le_bytes());
    b[14..16].copy_from_slice(&cell.showing.to_le_bytes());
    b
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::far::{FarColours, region, regions_of};
    use crate::doorway::Doorway;

    /// The card for a test: the high-performance one, with the far bricks' buffer limit raised as the game raises it.
    fn card() -> (wgpu::Device, wgpu::Queue) {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).expect("a graphics card");
        let largest = adapter.limits().max_buffer_size.min(adapter.limits().max_storage_buffer_binding_size).min((2 << 30) - 1);
        let required_limits = wgpu::Limits { max_buffer_size: largest, max_storage_buffer_binding_size: largest, ..wgpu::Limits::default() };
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("a test"), required_limits, ..Default::default() })).expect("the card's device")
    }

    /// A cell's 16 bytes as the card's four words.
    fn words(cell: &Cell) -> [u32; 4] {
        let b = cell_bytes(cell);
        [0, 1, 2, 3].map(|k| u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]]))
    }

    /// Every level's bricks at its edges write their bands only within their own rows and the level's columns (build 121
    /// wrote past them at the levels over 1,024, whose last brick a row reaches past the map's columns).
    #[test]
    fn edge_bricks_stay_in_their_rows() {
        let doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let levels: Vec<Level> = [2, 4, 8].into_iter().flat_map(|side| levels(regions_of(&doorway), doorway.height(), side)).collect();
        let empty = CellColumn::default();
        for l in &levels {
            let end = l.entries_at + (l.across * l.across) as u32;
            for (bx, by) in [(0, 0), (l.bricks - 1, 0), (0, l.bricks - 1), (l.bricks - 1, l.bricks - 1)] {
                let (cx0, cy0) = (l.from[0] + bx * l.wide, l.from[1] + by * l.wide);
                let part = part(l, cx0, cy0, l.wide, |cx, cy| l.entry(cx, cy).map(|_| &empty), true);
                for (start, row) in &part.entries {
                    let row_end = l.entries_at + (((start - l.entries_at) / l.across as u32) + 1) * l.across as u32;
                    assert!(start + row.len() as u32 <= row_end.min(end), "cells of {}: a brick at {bx} {by} writes {} bands from {start}, past its row's end {row_end}", 1 << l.level, row.len());
                }
            }
        }
    }

    const UNKNOWN: [u32; 4] = [0xFFFF_FFFF, 0, 0, 0];
    const PAST: [u32; 4] = [0xFFFF_FFFE, 0, 0, 0];

    /// The pass mark (plan 2.2, step 2a): cells read through the bricks on the card, at every level, are the far cells
    /// as summed (step 1's), in their compact form: in their bands, under them (solid), over them (empty), in regions not
    /// summed (unknown), and past the levels (past the map). Four regions round the spawn put on the card, so the first
    /// level over 1,024 is made for them and the one over that isn't. At each brick side (plan 2.2, step 3).
    #[test]
    fn the_far_bricks_read_back_as_the_far_cells() {
        for side in [2, 4, 8] {
            reads_back(side);
        }
    }

    fn reads_back(side: i32) {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (device, queue) = card();
        let mut store = Store::new(&device, &queue, regions_of(&doorway), doorway.height(), side, 10);
        let summed: Vec<([i32; 2], Vec<Vec<CellColumn>>)> = [[0, 0], [1, 0], [0, 1], [1, 1]].iter().map(|r| (*r, region(&doorway, &colours, *r).unwrap())).collect();
        for (r, columns) in &summed {
            store.put_region(&device, &queue, *r, columns);
        }
        // The window round the spawn brings the 32s and 64s on (all four regions are within it).
        store.follow(&device, &queue, [1024.0, 1024.0], f32::INFINITY);
        // What's asked (level, column north, column east, cell up), and what's wanted.
        let mut asks: Vec<[i32; 4]> = Vec::new();
        let mut wanted: Vec<[u32; 4]> = Vec::new();
        for (r, columns) in &summed {
            for level in FIRST..=REGION {
                let n = 1 << (REGION - level);
                let size = 1u32 << level;
                let up = store.levels[level - FIRST].up;
                for (k, column) in columns[level - FIRST].iter().enumerate() {
                    let (cx, cy) = (r[0] * n + (k as i32 % n), r[1] * n + (k as i32 / n));
                    for cz in 0..up {
                        asks.push([level as i32, cx, cy, cz]);
                        wanted.push(words(&column.at(cz, size as i32).compact(size)));
                    }
                }
            }
        }
        // The level over 1,024 made from the four, and the one over it not yet.
        let parent = crate::far::parent_of([&summed[0].1[REGION - FIRST][0], &summed[1].1[REGION - FIRST][0], &summed[2].1[REGION - FIRST][0], &summed[3].1[REGION - FIRST][0]], 1 << REGION);
        asks.push([REGION as i32 + 1, 0, 0, 0]);
        wanted.push(words(&parent.at(0, 1 << (REGION + 1)).compact(1 << (REGION + 1))));
        asks.push([REGION as i32 + 2, 0, 0, 0]);
        wanted.push(UNKNOWN);
        // A region not summed, and past a level's columns.
        asks.push([FIRST as i32, 2 * 32 + 5, 7, 4]);
        wanted.push(UNKNOWN);
        asks.push([FIRST as i32, 100_000, 7, 4]);
        wanted.push(PAST);
        let got = read_through_card(&device, &queue, &store, &asks);
        let differ: Vec<usize> = (0..asks.len()).filter(|&k| got[k] != wanted[k]).collect();
        eprintln!("The far bricks of {side}: {} cells read back through the bricks on the card ({} bricks, {:.1} MB of cells for four regions), {} not as summed", asks.len(), store.bricks, store.pool_bytes().0 as f64 / 1e6, differ.len());
        if let Some(&k) = differ.first() {
            panic!("{} cells differ; the first, {:?} (level, north, east, up): the card read {:x?}, the far cells hold {:x?}", differ.len(), asks[k], got[k], wanted[k]);
        }
    }
    /// Cells read through the bricks on the card as the far eye reads them (`far.wgsl`'s `entry_at` and `cell_word`):
    /// each ask (level, column north, column east, cell up) answered with its four words, or unknown, or past the map.
    fn read_through_card(device: &wgpu::Device, queue: &wgpu::Queue, store: &Store, asks: &[[i32; 4]]) -> Vec<[u32; 4]> {
        let source = format!(
            "{}\n{}",
            include_str!("far.wgsl"),
            r#"
@group(0) @binding(11) var<storage, read> asks: array<vec4<i32>>;
@group(0) @binding(12) var<storage, read_write> answers: array<vec4<u32>>;
@compute @workgroup_size(64)
fn test_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = id.x;
    if (n >= arrayLength(&asks)) {
        return;
    }
    let a = asks[n];
    let e = entry_at(a.x, a.y, a.z);
    if (e == NOT_SUMMED) {
        answers[n] = vec4<u32>(0xFFFFFFFFu, 0u, 0u, 0u);
        return;
    }
    if (e == PAST_MAP) {
        answers[n] = vec4<u32>(0xFFFFFFFEu, 0u, 0u, 0u);
        return;
    }
    let low = bitcast<i32>(e << 16u) >> 16u;
    let count = i32(e >> 16u);
    if (a.w < 0 || (a.w < low && low > 0)) {
        answers[n] = vec4<u32>(255u, 0u, 0u, 0u);
        return;
    }
    if (a.w < low || a.w >= low + count) {
        answers[n] = vec4<u32>(0u);
        return;
    }
    answers[n] = cell_word(a.x, a.y, a.z, a.w);
}
"#
        );
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far bricks' test"), source: wgpu::ShaderSource::Wgsl(source.into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("the far bricks' test"), layout: None, module: &module, entry_point: Some("test_cells"), compilation_options: Default::default(), cache: None });
        let bytes = |words: &[[i32; 4]]| words.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>();
        let asks_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("asks"), size: (asks.len() * 16) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&asks_buffer, 0, &bytes(asks));
        let size = (asks.len() * 16) as u64;
        let answers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("answers"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("answers read back"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("the far bricks' test"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 1, resource: store.table.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: store.entries.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: store.index.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: store.pool.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 11, resource: asks_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 12, resource: answers.as_entire_binding() },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far bricks' test") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the far bricks' test"), timestamp_writes: None });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group, &[]);
            let groups = (asks.len() as u32).div_ceil(64);
            pass.dispatch_workgroups(groups.min(65535), groups.div_ceil(65535), 1);
        }
        encoder.copy_buffer_to_buffer(&answers, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let got: Vec<[u32; 4]> = readback.slice(..).get_mapped_range().unwrap().as_chunks::<16>().0.iter().map(|b| [0, 1, 2, 3].map(|k| u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]]))).collect();
        got
    }

    /// The window round the eye (step 4): moved away and back, it gives back the regions it leaves and takes in those
    /// it comes to; once it has been both ways, more moves leave the pool's use and its room as they were (nothing grows
    /// with play); and in the usable regions the 32s' and 64s' cells read back through the card as summed.
    #[test]
    fn the_window_follows_the_eye_and_gives_back_its_room() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (device, queue) = card();
        // The 32s within 2 regions of the eye, the 64s within 4; the regions 3 either way of the spawn summed.
        let mut store = Store::new(&device, &queue, regions_of(&doorway), doorway.height(), 4, 2);
        let mut summed = Vec::new();
        for ry in -3..=3 {
            for rx in -3..=3 {
                let columns = region(&doorway, &colours, [rx, ry]).unwrap();
                store.put_region(&device, &queue, [rx, ry], &columns);
                summed.push(([rx, ry], columns));
            }
        }
        let (spawn, east) = ([512.0, 512.0], [512.0 + 2048.0, 512.0]);
        store.follow(&device, &queue, spawn, f32::INFINITY);
        assert!(store.on_card[0].contains_key(&[0, 0]) && store.on_card[0].contains_key(&[2, 0]) && !store.on_card[0].contains_key(&[3, 0]), "the 32s within two regions of the spawn on the card, none three off");
        assert_eq!(store.behind, 0, "every region wanted brought");
        // On a first visit: 13 regions on at the 32s, and only the middle has all eight neighbours, so the usable 32s
        // reach to its edges, half a region from the eye.
        assert!(store.usable_at(0, [0, 0]) && !store.usable_at(0, [1, 0]) && !store.usable_at(0, [1, 1]), "only the spawn's region usable at the 32s");
        let first_reach = store.usable_reach(spawn).unwrap();
        assert!((first_reach[0] - 512.0).abs() < 1.0, "the usable 32s reach half a region: {}", first_reach[0]);
        store.follow(&device, &queue, east, f32::INFINITY);
        assert!(!store.on_card[0].contains_key(&[-2, 0]) && store.on_card[0].contains_key(&[-1, 0]) && store.on_card[0].contains_key(&[4 - 1, 0]), "the 32s four off given back, three off kept (the edge's region of grace), the new ones in");
        store.follow(&device, &queue, spawn, f32::INFINITY);
        let at_spawn = store.pool_bytes().0;
        store.follow(&device, &queue, east, f32::INFINITY);
        let (at_east, room) = store.pool_bytes();
        for _ in 0..3 {
            store.follow(&device, &queue, spawn, f32::INFINITY);
            assert_eq!(store.pool_bytes().0, at_spawn, "back at the spawn, the pool's use as before");
            store.follow(&device, &queue, east, f32::INFINITY);
            assert_eq!(store.pool_bytes(), (at_east, room), "east again, the pool's use and room as before");
        }
        store.follow(&device, &queue, spawn, f32::INFINITY);
        // Every cell of the usable regions' 32s and 64s, read through the card.
        let mut asks: Vec<[i32; 4]> = Vec::new();
        let mut wanted: Vec<[u32; 4]> = Vec::new();
        let mut usable = [0; WINDOWED];
        for (r, columns) in &summed {
            for k in 0..WINDOWED {
                if !store.usable_at(k, *r) {
                    continue;
                }
                usable[k] += 1;
                let level = FIRST + k;
                let n = 1 << (REGION - level);
                let size = 1u32 << level;
                for (c, column) in columns[level - FIRST].iter().enumerate() {
                    let (cx, cy) = (r[0] * n + (c as i32 % n), r[1] * n + (c as i32 / n));
                    for cz in 0..store.levels[k].up {
                        asks.push([level as i32, cx, cy, cz]);
                        wanted.push(words(&column.at(cz, size as i32).compact(size)));
                    }
                }
            }
        }
        assert!(usable[0] >= 1 && usable[1] >= 9, "the spawn's 32s and the 64s round it usable: {usable:?}");
        let got = read_through_card(&device, &queue, &store, &asks);
        let differ: Vec<usize> = (0..asks.len()).filter(|&k| got[k] != wanted[k]).collect();
        let reach = store.usable_reach(spawn).unwrap();
        eprintln!(
            "The window: {} cells of the usable regions read back after {} moves ({} at the 32s, {} at the 64s), {} not as summed; the pool {:.1} MB used of {:.1} MB; the usable reach round the spawn {:.0} and {:.0} blocks",
            asks.len(),
            11,
            usable[0],
            usable[1],
            differ.len(),
            at_spawn as f64 / 1e6,
            room as f64 / 1e6,
            reach[0],
            reach[1]
        );
        if let Some(&k) = differ.first() {
            panic!("{} cells differ; the first, {:?}: the card read {:x?}, the far cells hold {:x?}", differ.len(), asks[k], got[k], wanted[k]);
        }
    }

}

#[cfg(test)]
mod whole_world {
    use super::*;
    use crate::doorway::Doorway;
    use crate::far::{FarColours, region, regions_of};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// How big the whole world's far bricks are (no card: the parts counted as the store would put them). A measure,
    /// run when asked: `cargo test --release whole_world -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn the_whole_world_s_bricks() {
        let doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let (from, across) = regions_of(&doorway);
        let levels = levels((from, across), doorway.height(), 8);
        let next = AtomicUsize::new(0);
        let totals = Mutex::new(vec![(0usize, 0u64); levels.len()]);
        std::thread::scope(|scope| {
            for _ in 0..std::thread::available_parallelism().map_or(4, |n| n.get()) {
                scope.spawn(|| {
                    let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
                    let colours = FarColours::load(&mut doorway).unwrap();
                    loop {
                        let k = next.fetch_add(1, Ordering::Relaxed);
                        if k >= (across * across) as usize {
                            break;
                        }
                        let r = [from[0] + k as i32 % across, from[1] + k as i32 / across];
                        let columns = region(&doorway, &colours, r).unwrap();
                        let parts = region_parts(&levels, r, &columns, 0);
                        let mut totals = totals.lock().unwrap();
                        for (l, part) in parts.iter().enumerate() {
                            totals[l].0 += part.bricks.len();
                            totals[l].1 += part.bricks.iter().map(|(_, cells)| cells.len() as u64 * 16).sum::<u64>();
                        }
                    }
                });
            }
        });
        let totals = totals.into_inner().unwrap();
        for (l, (bricks, bytes)) in totals.iter().enumerate().take(REGION - FIRST + 1) {
            eprintln!("cells of {:>5}: {bricks:>7} bricks, {:>7.1} MB", 1 << (FIRST + l), *bytes as f64 / 1e6);
        }
        eprintln!("all, 32 to 1,024: {} bricks, {:.1} MB", totals.iter().map(|t| t.0).sum::<usize>(), totals.iter().map(|t| t.1).sum::<u64>() as f64 / 1e6);
    }

    /// Bricks of other sizes counted over the whole world, levels 32 to 1,024 (where nearly all of it is), before any are
    /// built (plan 2.2, step 3): for each side, the bricks the bands touch, the pool's cells (16 bytes, a whole brick
    /// each), the atlas's texels (its side and a border, 16 bytes), and the index (4 bytes a brick place). A measure, run
    /// when asked: `cargo test --release bricks_of_each_size -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn bricks_of_each_size() {
        use std::collections::HashSet;
        const SIDES: [i32; 4] = [2, 4, 8, 16];
        let doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let (from, across) = regions_of(&doorway);
        let levels = levels((from, across), doorway.height(), 8);
        let next = AtomicUsize::new(0);
        let marked: Mutex<Vec<Vec<HashSet<(i32, i32, i32)>>>> = Mutex::new(vec![vec![HashSet::new(); REGION - FIRST + 1]; SIDES.len()]);
        let band_cells = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..std::thread::available_parallelism().map_or(4, |n| n.get()) {
                scope.spawn(|| {
                    let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
                    let colours = FarColours::load(&mut doorway).unwrap();
                    loop {
                        let k = next.fetch_add(1, Ordering::Relaxed);
                        if k >= (across * across) as usize {
                            break;
                        }
                        let r = [from[0] + k as i32 % across, from[1] + k as i32 / across];
                        let columns = region(&doorway, &colours, r).unwrap();
                        let mut mine: Vec<Vec<HashSet<(i32, i32, i32)>>> = vec![vec![HashSet::new(); REGION - FIRST + 1]; SIDES.len()];
                        let mut cells = 0;
                        for level in FIRST..=REGION {
                            let l = &levels[level - FIRST];
                            let n = 1i32 << (REGION - level);
                            for (k, column) in columns[level - FIRST].iter().enumerate() {
                                let (x, y) = (r[0] * n + k as i32 % n - l.from[0], r[1] * n + k as i32 / n - l.from[1]);
                                let (low, high) = (column.low.max(0), (column.low + column.sums.len() as i32).min(l.up));
                                cells += (high - low).max(0) as usize;
                                for (s, side) in SIDES.iter().enumerate() {
                                    for bz in (low / side)..=((high - 1).max(low) / side) {
                                        if high > low {
                                            mine[s][level - FIRST].insert((x / side, y / side, bz));
                                        }
                                    }
                                }
                            }
                        }
                        band_cells.fetch_add(cells, Ordering::Relaxed);
                        let mut marked = marked.lock().unwrap();
                        for (s, by_level) in mine.into_iter().enumerate() {
                            for (l, set) in by_level.into_iter().enumerate() {
                                marked[s][l].extend(set);
                            }
                        }
                    }
                });
            }
        });
        let marked = marked.into_inner().unwrap();
        let cells = band_cells.into_inner();
        eprintln!("Levels 32 to 1,024: {cells} cells in the bands ({:.1} MB at 16 bytes)", cells as f64 * 16.0 / 1e6);
        for (s, side) in SIDES.iter().enumerate() {
            let bricks: usize = marked[s].iter().map(HashSet::len).sum();
            let index: i64 = levels[..=REGION - FIRST].iter().map(|l| ((l.across + side - 1) / side) as i64 * ((l.across + side - 1) / side) as i64 * ((l.up + side - 1) / side) as i64).sum();
            let pool = bricks as f64 * (side * side * side) as f64 * 16.0;
            let atlas = bricks as f64 * ((side + 1) * (side + 1) * (side + 1)) as f64 * 16.0;
            eprintln!(
                "bricks of {side:>2}: {bricks:>8} bricks, the pool {:>7.1} MB ({:.0}% of it band), the atlas {:>7.1} MB, the index {:>6.1} MB; by level {:?}",
                pool / 1e6,
                cells as f64 * 16.0 / pool * 100.0,
                atlas / 1e6,
                index as f64 * 4.0 / 1e6,
                marked[s].iter().map(HashSet::len).collect::<Vec<_>>()
            );
        }
    }
}

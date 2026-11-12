//! The far bricks (plan 2.2, step 2): the far cells on the card, for the far eye to read.
//!
//! Each level (cells of 32 up to 65,536) is laid over the whole map:
//! - **Each cell column's band** (`entries`): its lowest kept cell and how many, as old kept it, or "not summed yet".
//!   Below the band every cell is solid with no face, above it empty (`far::CellColumn`).
//! - **The cells of the bands, in bricks** (`pool`): a brick is 8 × 8 cell columns, 8 cells tall, where the level is
//!   that big. From cells of 256 up to 1,024 a brick stays within its region, so a region's arrival makes its bricks
//!   whole: 4 × 4 at 256, 2 × 2 at 512, 1 at 1,024. A brick is found by arithmetic, its place in the level's index
//!   (`index`), and a cell by its place in the brick. Only bricks holding some band are made.
//!
//! The regions are summed on the loading lane (`far::Lane`), each brought here as it comes and put on the card at once,
//! so the far view sharpens in live. The levels over 1,024 are made as soon as the regions under a cell are all in.
//! The pool grows as it fills (copied on the card into one twice the size): the whole world stays on the card, as
//! old's bake did.

use crate::far::{Cell, CellColumn, FIRST, REGION};
use std::collections::HashMap;

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

/// The levels laid over a map whose regions run `regions` (least region, north and east, and how many across).
pub fn levels(regions: ([i32; 2], i32), height: i32) -> Vec<Level> {
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
            let wide = if level <= REGION { 8.min(1 << (REGION - level)) } else { 8 };
            let tall = 8.min(up);
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

/// The entries and bricks of the columns `cx0..cx0 + n`, `cy0..cy0 + n` of `level` (they must cover whole bricks), each
/// column found by `column(cx, cy)` (none: not summed yet).
pub fn part<'a>(level: &Level, cx0: i32, cy0: i32, n: i32, column: impl Fn(i32, i32) -> Option<&'a CellColumn>) -> Part {
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

/// A region's parts, level by level from 32 to 1,024 (its columns from `far::region`).
pub fn region_parts(levels: &[Level], region: [i32; 2], columns: &[Vec<CellColumn>]) -> Vec<Part> {
    (FIRST..=REGION)
        .map(|level| {
            let n = 1 << (REGION - level);
            let these = &columns[level - FIRST];
            part(&levels[level - FIRST], region[0] * n, region[1] * n, n, |cx, cy| these.get(((cy - region[1] * n) * n + (cx - region[0] * n)) as usize))
        })
        .collect()
}

/// Bytes a cell takes on the card.
const CELL_BYTES: u64 = 16;
/// The pool's first size: 128 MB.
const FIRST_POOL: u64 = (128 << 20) / CELL_BYTES;

/// The far bricks on the card, and what's kept here to make the levels over 1,024 as their regions come in.
pub struct Store {
    pub levels: Vec<Level>,
    pub table: wgpu::Buffer,
    pub entries: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub pool: wgpu::Buffer,
    /// Cells the pool holds, and those used.
    capacity: u64,
    used: u64,
    /// Bumped whenever the pool is made again (bind groups made with the old one are stale).
    pub generation: u32,
    /// Each region's 1,024 as it comes in, and each level over 1,024's columns as they're made.
    tops: HashMap<[i32; 2], CellColumn>,
    upper: Vec<HashMap<[i32; 2], CellColumn>>,
    /// Regions in, the highest anything stands (blocks), and the bricks made.
    pub regions_in: usize,
    pub highest: f32,
    pub bricks: usize,
}

impl Store {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, regions: ([i32; 2], i32), height: i32) -> Self {
        let levels = levels(regions, height);
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
        log::info!(
            "The far bricks: {} levels, {:.1} MB of bands and {:.1} MB of index on the card; the pool {} MB to start",
            levels.len(),
            (entries_count * 4) as f64 / 1e6,
            (index_count * 4) as f64 / 1e6,
            (FIRST_POOL * CELL_BYTES) >> 20
        );
        Self { levels, table: table_buffer, entries, index, pool, capacity: FIRST_POOL, used: 0, generation: 0, tops: HashMap::new(), upper: vec![HashMap::new(); TOP - REGION], regions_in: 0, highest: 0.0, bricks: 0 }
    }

    /// The pool's bytes in use, and its size.
    pub fn pool_bytes(&self) -> (u64, u64) {
        (self.used * CELL_BYTES, self.capacity * CELL_BYTES)
    }

    /// A part put on the card: its entries written, its bricks given places in the pool (grown first if it's full: the
    /// old pool copied into the new in `encoder`, which is handed over after the queue's writes, so the copy carries
    /// them too) and written there, and their places written in the index.
    fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, part: &Part) {
        for (start, words) in &part.entries {
            queue.write_buffer(&self.entries, *start as u64 * 4, &words.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        }
        let cells: u64 = part.bricks.iter().map(|(_, cells)| cells.len() as u64).sum();
        if cells == 0 {
            return;
        }
        if self.used + cells > self.capacity {
            let limit = device.limits().max_storage_buffer_binding_size.min(device.limits().max_buffer_size) / CELL_BYTES;
            let mut capacity = self.capacity;
            while self.used + cells > capacity && capacity < limit {
                capacity = (capacity * 2).min(limit);
            }
            assert!(self.used + cells <= capacity, "the far bricks don't fit in the card's largest buffer ({} MB)", (limit * CELL_BYTES) >> 20);
            let pool = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the far bricks"), size: capacity * CELL_BYTES, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
            // The old pool's writes not yet made go in first (they're the queue's, made before the encoder's copy).
            encoder.copy_buffer_to_buffer(&self.pool, 0, &pool, 0, self.used * CELL_BYTES);
            log::info!("The far bricks' pool grown to {} MB ({} MB used)", (capacity * CELL_BYTES) >> 20, (self.used * CELL_BYTES) >> 20);
            self.pool = pool;
            self.capacity = capacity;
            self.generation += 1;
        }
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
    /// regions under them are in.
    pub fn put_region(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, region: [i32; 2], columns: &[Vec<CellColumn>]) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far bricks, grown") });
        for part in region_parts(&self.levels, region, columns) {
            self.put(device, queue, &mut encoder, &part);
        }
        // The highest anything stands: the top of the highest band among its cells of 32.
        let top = columns[0].iter().map(|c| (c.high() + 1) * (1 << FIRST)).max().unwrap_or(0);
        self.highest = self.highest.max(top as f32);
        self.regions_in += 1;
        self.tops.insert(region, columns[REGION - FIRST][0].clone());
        let mut place = region;
        for level in REGION + 1..=TOP {
            let parent = [place[0].div_euclid(2), place[1].div_euclid(2)];
            let children: Option<Vec<CellColumn>> = (0..4).map(|k| self.child(level - 1, [parent[0] * 2 + (k & 1), parent[1] * 2 + (k >> 1)])).collect();
            let Some(children) = children else { break };
            let made = crate::far::parent_of([&children[0], &children[1], &children[2], &children[3]], 1 << (level - 1));
            self.upper[level - REGION - 1].insert(parent, made);
            self.put_upper(device, queue, &mut encoder, level, parent);
            place = parent;
        }
        queue.submit([encoder.finish()]);
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
    fn put_upper(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, level: usize, place: [i32; 2]) {
        let l = self.levels[level - FIRST];
        let (bx, by) = ((place[0] - l.from[0]).div_euclid(l.wide), (place[1] - l.from[1]).div_euclid(l.wide));
        let columns = std::mem::take(&mut self.upper[level - REGION - 1]);
        let part = part(&l, l.from[0] + bx * l.wide, l.from[1] + by * l.wide, l.wide, |cx, cy| columns.get(&[cx, cy]));
        self.upper[level - REGION - 1] = columns;
        self.put(device, queue, encoder, &part);
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
        let levels = levels(regions_of(&doorway), doorway.height());
        let empty = CellColumn::default();
        for l in &levels {
            let end = l.entries_at + (l.across * l.across) as u32;
            for (bx, by) in [(0, 0), (l.bricks - 1, 0), (0, l.bricks - 1), (l.bricks - 1, l.bricks - 1)] {
                let (cx0, cy0) = (l.from[0] + bx * l.wide, l.from[1] + by * l.wide);
                let part = part(l, cx0, cy0, l.wide, |cx, cy| l.entry(cx, cy).map(|_| &empty));
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
    /// level over 1,024 is made for them and the one over that isn't.
    #[test]
    fn the_far_bricks_read_back_as_the_far_cells() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (device, queue) = card();
        let mut store = Store::new(&device, &queue, regions_of(&doorway), doorway.height());
        let summed: Vec<([i32; 2], Vec<Vec<CellColumn>>)> = [[0, 0], [1, 0], [0, 1], [1, 1]].iter().map(|r| (*r, region(&doorway, &colours, *r).unwrap())).collect();
        for (r, columns) in &summed {
            store.put_region(&device, &queue, *r, columns);
        }
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
        // The card reads them as the far eye does (`far.wgsl`'s `entry_at` and `cell_word`).
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
        queue.write_buffer(&asks_buffer, 0, &bytes(&asks));
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
        let differ: Vec<usize> = (0..asks.len()).filter(|&k| got[k] != wanted[k]).collect();
        eprintln!("The far bricks: {} cells read back through the bricks on the card ({} bricks, {:.1} MB of cells for four regions), {} not as summed", asks.len(), store.bricks, store.pool_bytes().0 as f64 / 1e6, differ.len());
        if let Some(&k) = differ.first() {
            panic!("{} cells differ; the first, {:?} (level, north, east, up): the card read {:x?}, the far cells hold {:x?}", differ.len(), asks[k], got[k], wanted[k]);
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
        let levels = levels((from, across), doorway.height());
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
                        let parts = region_parts(&levels, r, &columns);
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
}

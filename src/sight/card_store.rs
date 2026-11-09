//! Sight's store on the card (the regroup, phase 3.2): the cells the march reads, where it reads them.
//!
//! - **The pool:** every kept cell (16 bytes: `Cell::to_bytes`, four 32-bit words), in one buffer. Places in
//!   it are handed out and taken back on the processor (`Pool`: first fit, neighbours merged), so a region or a
//!   tile made again replaces its cells and frees the old ones.
//! - **The baked levels' index:** for each level from 32 up, a cell column's entry over the whole map, row by
//!   row (where its band starts in the pool, its lowest cell, and how many; `NOT_BAKED` till it's baked).
//! - **The live levels' index:** for each live level from the finest kept up to 16, a window of cell columns
//!   round the camera, wrapping round (a column at `(x mod side, y mod side)`), each entry also saying which
//!   column it is, so one left from a tile the camera has moved off is never read for the column that's taken
//!   its place. Every live level's window is the same cell columns a side (each reaches out to where one of its
//!   cells is one finest pixel: `store::side_for`), one after the other in the index. A level between them
//!   not kept has its window all "not there", so the march reads the next coarser one.
//! - **The header:** the levels' numbers (`header` in `march.wgsl`).
//! - **The pieces** (build 90): each cell built of pieces (half blocks) near enough to have been edited, its
//!   eight octants' blocks, in a table the near walk looks a cell up in by its place (`put_pieces`).
//!
//! The processor writes it all in small pieces (a region's or a tile's worth), queued with the card's own
//! work in order: a column's cells always go before the entry that points to them.

use super::bake::{Bake, Extent, FIRST, REGION, Record};
use super::cell::Cell;
use std::sync::Mutex;

/// The pool's size, in cells (16 bytes each: 384 MB). A whole default world's bake is about 11 million.
pub const POOL_CELLS: u32 = 24 << 20;
/// An entry not baked yet.
pub const NOT_BAKED: u32 = 0xFFFF_0000;
/// The pieces table's places (32 bytes each, 512 KB); at most half are used, so a look-up stays short.
pub const PIECES_SLOTS: u32 = 16_384;
/// The most places a cell of pieces is looked for past its own (`march.wgsl`'s `pieces_slot`).
pub const PIECES_PROBES: u32 = 32;

/// The placed lights' table's places (16 bytes each, 1 MB): each a lit cell of air (build 95); at most half are
/// used, so a look-up stays short.
pub const LIGHT_SLOTS: u32 = 65_536;
/// The most places a lit cell is looked for past its own (`march.wgsl`'s `placed_light`).
pub const LIGHT_PROBES: u32 = 32;

/// Where a lit cell at `c` of `level` (0 for blocks, the wash's coarser ones: `lights::wash`) is first looked for
/// (the same in `march.wgsl`).
pub fn light_hash(c: [i32; 3], level: u32) -> u32 {
    ((c[0] as u32).wrapping_mul(73_856_093) ^ (c[1] as u32).wrapping_mul(19_349_663) ^ (c[2] as u32).wrapping_mul(83_492_791) ^ level.wrapping_mul(2_654_435_761)) & (LIGHT_SLOTS - 1)
}

/// Where a cell of pieces at `c` is first looked for (the same in `march.wgsl`).
pub fn pieces_hash(c: [i32; 3]) -> u32 {
    ((c[0] as u32).wrapping_mul(73_856_093) ^ (c[1] as u32).wrapping_mul(19_349_663) ^ (c[2] as u32).wrapping_mul(83_492_791)) & (PIECES_SLOTS - 1)
}

/// Places in the pool: first fit, neighbours merged when given back.
#[derive(Debug)]
pub struct Pool {
    free: Vec<(u32, u32)>,
}

impl Pool {
    pub fn new(cells: u32) -> Self {
        Self { free: vec![(0, cells)] }
    }

    /// `n` cells' place, if there's room.
    pub fn take(&mut self, n: u32) -> Option<u32> {
        if n == 0 {
            return Some(0);
        }
        let k = self.free.iter().position(|&(_, len)| len >= n)?;
        let (start, len) = self.free[k];
        if len == n {
            self.free.remove(k);
        } else {
            self.free[k] = (start + n, len - n);
        }
        Some(start)
    }

    /// A place given back.
    pub fn give(&mut self, start: u32, n: u32) {
        if n == 0 {
            return;
        }
        let k = self.free.partition_point(|&(s, _)| s < start);
        self.free.insert(k, (start, n));
        // Merged with the place after it, then the one before.
        if k + 1 < self.free.len() && self.free[k].0 + self.free[k].1 == self.free[k + 1].0 {
            self.free[k].1 += self.free[k + 1].1;
            self.free.remove(k + 1);
        }
        if k > 0 && self.free[k - 1].0 + self.free[k - 1].1 == self.free[k].0 {
            self.free[k - 1].1 += self.free[k].1;
            self.free.remove(k);
        }
    }

    pub fn free_cells(&self) -> u64 {
        self.free.iter().map(|&(_, len)| len as u64).sum()
    }
}

/// What's in the pool and where, by owner (a region, a column above the regions, a live tile).
#[derive(Default)]
struct Owners {
    regions: std::collections::HashMap<(i32, i32), (u32, u32)>,
    upper: std::collections::HashMap<(usize, i32, i32), (u32, u32)>,
    tiles: std::collections::HashMap<(usize, i32, i32), (u32, u32)>,
}

/// What the store holds, read back (`CardStore::read_back`, tests).
#[cfg(test)]
pub struct StoreRead {
    pub live: Vec<[u32; 4]>,
    pub baked: Vec<[u32; 2]>,
    pub pool: Vec<[u32; 4]>,
    pub finest_live: usize,
    pub most_side: usize,
    /// Each baked level's (from, across, where its entries start).
    pub extents: Vec<((i32, i32), i32, u32)>,
}

#[cfg(test)]
impl StoreRead {
    /// A live slot's or baked entry's cells (its lowest, and the cells themselves), or none if empty or not there.
    pub fn cells(&self, start: u32, word: u32) -> (i32, Vec<[u32; 4]>) {
        let low = (word & 0xFFFF) as u16 as i16 as i32;
        let count = (word >> 16) as usize;
        if word == super::card_store::NOT_BAKED || count == 0 {
            return (low, Vec::new());
        }
        let start = start as usize;
        (low, self.pool.get(start..start + count).map(|c| c.to_vec()).unwrap_or_default())
    }
}

/// The store on the card.
pub struct CardStore {
    pub pool: wgpu::Buffer,
    pub baked: wgpu::Buffer,
    pub live: wgpu::Buffer,
    pub header: wgpu::Buffer,
    /// The baked levels' bounds, and where each one's entries start in `baked`.
    pub extents: Vec<(Extent, u32)>,
    /// The finest live level, and which levels from it up to 16 (`FIRST - 1`) are kept (a bit each).
    pub finest_live: usize,
    pub live_kept: u32,
    /// The window's cell columns a side now (at most `most_side`, what the live index was made for).
    pub side: std::sync::atomic::AtomicI32,
    pub most_side: i32,
    /// The highest anything stands in what's on the card so far (blocks, to a cell): over it a ray that isn't
    /// coming down meets nothing (the march's ceiling, from the first region, not only once the bake is done).
    pub highest: std::sync::atomic::AtomicI32,
    /// The cells of pieces (`put_pieces`).
    pub pieces: wgpu::Buffer,
    /// The lit cells of air (`put_lights`).
    pub lights: wgpu::Buffer,
    queue: wgpu::Queue,
    places: Mutex<(Pool, Owners)>,
}

impl CardStore {
    /// A store for a map `half` blocks either side of the middle, its live levels `live` (each under 32), each
    /// with at most `side` cell columns a side. Its indexes start "not baked".
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, half: i32, live: &[usize], side: i32) -> Self {
        let finest_live = live.iter().copied().min().unwrap_or(FIRST - 1);
        let live_kept = live.iter().filter(|&&level| level < FIRST).fold(0u32, |kept, &level| kept | 1 << level);
        let mut extents = Vec::new();
        let mut entries = 0u32;
        for level in FIRST..=Bake::top(half) {
            let extent = Extent::of(level, half);
            extents.push((extent, entries));
            entries += (extent.across * extent.across) as u32;
        }
        // Each baked entry: its start, then its lowest cell (low half) and count (high half).
        let baked = {
            let mut words = Vec::with_capacity(entries as usize * 2);
            for _ in 0..entries {
                words.extend([0, NOT_BAKED]);
            }
            use wgpu::util::DeviceExt;
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's baked index"), contents: &words.iter().flat_map(|w: &u32| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC })
        };
        // Each live entry: start, lowest cell and count, and which column it is (x, y): "none" to begin with.
        let windows = (FIRST - finest_live) as i32;
        let live = {
            let mut words: Vec<u32> = Vec::with_capacity((windows * side * side * 4) as usize);
            for _ in 0..windows * side * side {
                words.extend([0, NOT_BAKED, i32::MIN as u32, i32::MIN as u32]);
            }
            use wgpu::util::DeviceExt;
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's live index"), contents: &words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC })
        };
        let pool = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's cells"), size: POOL_CELLS as u64 * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        // The header: the first baked level, the top, the finest live level, the live windows' side now, the
        // most (each level's window starts that squared on from the last), and which live levels are kept (a
        // bit each); then each baked level's from x, from y, across, and where its entries start.
        let mut header = vec![FIRST as i32, Bake::top(half) as i32, finest_live as i32, side, side, live_kept as i32];
        // (Each level's part of the live index is made for `side`; a smaller window uses the start of it.)
        for (extent, at) in &extents {
            header.extend([extent.from.0, extent.from.1, extent.across, *at as i32]);
        }
        let header = {
            use wgpu::util::DeviceExt;
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's header"), contents: &header.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST })
        };
        let pieces = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's pieces"), size: PIECES_SLOTS as u64 * 32, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let lights = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's placed lights"), size: LIGHT_SLOTS as u64 * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Self { pool, baked, live, header, extents, finest_live, live_kept, side: std::sync::atomic::AtomicI32::new(side), most_side: side, highest: std::sync::atomic::AtomicI32::new(0), pieces, lights, queue: queue.clone(), places: Mutex::new((Pool::new(POOL_CELLS), Owners::default())) }
    }

    /// The lit cells of air on the card, all of them at once (each its place, and its light as
    /// `lights::PlacedLights::lit` gives it), the table written whole. Each cell goes in its first free place from
    /// its hash on; past half the table, or past `LIGHT_PROBES` places on, it's left out (dark). How many went in.
    pub fn put_lights(&self, cells: &[([i32; 3], u32)]) -> usize {
        let mut table = vec![[0u32; 4]; LIGHT_SLOTS as usize];
        let mut put = 0;
        for &(c, light) in cells.iter().take(LIGHT_SLOTS as usize / 2) {
            let start = light_hash(c, (light >> 20) & 31);
            for k in 0..LIGHT_PROBES {
                let slot = &mut table[((start + k) & (LIGHT_SLOTS - 1)) as usize];
                if slot[3] == 0 {
                    // The top bit: a place in use.
                    *slot = [c[0] as u32, c[1] as u32, c[2] as u32, light | 1 << 31];
                    put += 1;
                    break;
                }
            }
        }
        self.queue.write_buffer(&self.lights, 0, &table.iter().flatten().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>());
        put
    }

    /// The cells of pieces on the card, all of them at once (each its place and its eight octants' blocks, 0 an
    /// empty octant), the table written whole. Each cell goes in its first free place from its hash on; past half
    /// the table, or past `PIECES_PROBES` places on, it's left out (it shows as a block of its main material).
    /// How many went in.
    pub fn put_pieces(&self, cells: &[([i32; 3], [u16; 8])]) -> usize {
        let mut table = vec![[0u32; 8]; PIECES_SLOTS as usize];
        let mut put = 0;
        for &(c, octants) in cells.iter().take(PIECES_SLOTS as usize / 2) {
            let start = pieces_hash(c);
            for k in 0..PIECES_PROBES {
                let slot = &mut table[((start + k) & (PIECES_SLOTS - 1)) as usize];
                if slot[3] == 0 {
                    let pair = |i: usize| octants[i] as u32 | (octants[i + 1] as u32) << 16;
                    *slot = [c[0] as u32, c[1] as u32, c[2] as u32, 1, pair(0), pair(2), pair(4), pair(6)];
                    put += 1;
                    break;
                }
            }
        }
        self.queue.write_buffer(&self.pieces, 0, &table.iter().flatten().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>());
        put
    }

    pub fn side(&self) -> i32 {
        self.side.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Whether `level` is kept live.
    pub fn is_live(&self, level: usize) -> bool {
        level < FIRST && self.live_kept & 1 << level != 0
    }

    /// How far the live level `level` reaches from the camera (blocks: half its window).
    pub fn reach(&self, level: usize) -> f32 {
        (self.side() << level) as f32 / 2.0
    }

    /// The live windows made `side` a side (the grid changed): their tiles given back and entries cleared,
    /// so nothing left from the old window is ever read.
    pub fn reset_live(&self, side: i32) {
        let side = side.min(self.most_side);
        let Ok(mut places) = self.places.lock() else { return };
        let (pool, owners) = &mut *places;
        for (_, (start, n)) in owners.tiles.drain() {
            pool.give(start, n);
        }
        let windows = (FIRST - self.finest_live) as i32;
        let none: Vec<u8> = (0..windows * self.most_side * self.most_side).flat_map(|_| [0u32, NOT_BAKED, i32::MIN as u32, i32::MIN as u32]).flat_map(|w| w.to_le_bytes()).collect();
        self.queue.write_buffer(&self.live, 0, &none);
        self.queue.write_buffer(&self.header, 12, &side.to_le_bytes());
        self.side.store(side, std::sync::atomic::Ordering::Relaxed);
    }

    fn cells_bytes(cells: &[Cell]) -> Vec<u8> {
        cells.iter().flat_map(|c| c.to_bytes()).collect()
    }

    /// A band's entry: its start, then its lowest cell (low 16 bits) and count (high 16).
    fn entry(start: u32, low: i32, count: usize) -> [u32; 2] {
        [start, (low as u16 as u32) | ((count as u32) << 16)]
    }

    /// A region's cells (from 32 to 1,024) put on the card, in one place in the pool, and its entries
    /// pointed at them; the place it had before given back.
    pub fn put_region(&self, record: &Record) {
        let total: usize = record.levels.iter().flatten().map(|(_, cells)| cells.len()).sum();
        let top = record.levels.first().map_or(0, |columns| columns.iter().filter(|(_, c)| !c.is_empty()).map(|(low, cells)| (low + cells.len() as i32) << FIRST).max().unwrap_or(0));
        self.highest.fetch_max(top, std::sync::atomic::Ordering::Relaxed);
        let Ok(mut places) = self.places.lock() else { return };
        let (pool, owners) = &mut *places;
        if let Some((start, n)) = owners.regions.remove(&record.region) {
            pool.give(start, n);
        }
        let Some(start) = pool.take(total as u32) else { return };
        owners.regions.insert(record.region, (start, total as u32));
        // The cells first, all levels in one place, then the entries pointing at them (queued in order).
        let mut bytes = Vec::with_capacity(total * 16);
        let mut entries = Vec::new();
        let mut at = start;
        for (k, columns) in record.levels.iter().enumerate() {
            let level = FIRST + k;
            let across = 1 << (REGION - level);
            let mut rows = vec![Vec::with_capacity(across as usize * 8); across as usize];
            for (i, (low, cells)) in columns.iter().enumerate() {
                bytes.extend(Self::cells_bytes(cells));
                let entry = Self::entry(at, *low, cells.len());
                at += cells.len() as u32;
                rows[i / across as usize].extend(entry.iter().flat_map(|w| w.to_le_bytes()));
            }
            // Each row's columns inside the level's bounds only [2026-11-09: a map needn't be whole regions, and a
            // region past its edge wrote its row on past the end of the map's row, over the start of the next (the
            // columns there "empty", or not, as the bakers happened to finish), and one starting outside it wasn't
            // written at all; near the eye the walk then skipped into hills it took for empty air].
            let Some((extent, offset)) = self.extents.get(k) else { continue };
            for (row, data) in rows.into_iter().enumerate() {
                let (cx, cy) = (record.region.0 * across, record.region.1 * across + row as i32);
                let (first, last) = (cx.max(extent.from.0), (cx + across).min(extent.from.0 + extent.across));
                if first >= last {
                    continue;
                }
                let Some(index) = self.index(level, first, cy) else { continue };
                let (skip, take) = (((first - cx) * 8) as usize, ((last - first) * 8) as usize);
                entries.push(((*offset as u64 + index as u64) * 8, data[skip..skip + take].to_vec()));
            }
        }
        self.queue.write_buffer(&self.pool, start as u64 * 16, &bytes);
        for (at, data) in entries {
            self.queue.write_buffer(&self.baked, at, &data);
        }
    }

    /// Where a baked cell column's entry is within its level (`None` past the map).
    fn index(&self, level: usize, cx: i32, cy: i32) -> Option<u32> {
        let (extent, _) = self.extents.get(level - FIRST)?;
        let (x, y) = (cx - extent.from.0, cy - extent.from.1);
        ((0..extent.across).contains(&x) && (0..extent.across).contains(&y)).then(|| (y * extent.across + x) as u32)
    }

    /// A cell column above the regions (1,024 up) put on the card.
    pub fn put_upper(&self, level: usize, cx: i32, cy: i32, low: i32, cells: &[Cell]) {
        let Ok(mut places) = self.places.lock() else { return };
        let (pool, owners) = &mut *places;
        if let Some((start, n)) = owners.upper.remove(&(level, cx, cy)) {
            pool.give(start, n);
        }
        let Some(start) = pool.take(cells.len() as u32) else { return };
        owners.upper.insert((level, cx, cy), (start, cells.len() as u32));
        self.queue.write_buffer(&self.pool, start as u64 * 16, &Self::cells_bytes(cells));
        if let (Some(index), Some((_, offset))) = (self.index(level, cx, cy), self.extents.get(level - FIRST)) {
            let entry = Self::entry(start, low, cells.len());
            self.queue.write_buffer(&self.baked, (*offset as u64 + index as u64) * 8, &entry.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>());
        }
    }

    /// A live tile's cell columns (`TILE` × `TILE`, x fastest, each its lowest cell and its cells) put on the
    /// card at tile (`tx`, `ty`) of the live level `level`; the place a tile had there before given back.
    pub fn put_tile(&self, level: usize, tx: i32, ty: i32, tile: u32, columns: &[(i32, Vec<Cell>)]) {
        if !self.is_live(level) {
            return;
        }
        let total: usize = columns.iter().map(|(_, c)| c.len()).sum();
        let top = columns.iter().filter(|(_, c)| !c.is_empty()).map(|(low, cells)| (low + cells.len() as i32) << level).max().unwrap_or(0);
        self.highest.fetch_max(top, std::sync::atomic::Ordering::Relaxed);
        let Ok(mut places) = self.places.lock() else { return };
        let (pool, owners) = &mut *places;
        // The slot the tile takes in the wrapping window: whatever tile was there is given back.
        let side = self.side();
        let slot = (level, tx.rem_euclid(side / tile as i32), ty.rem_euclid(side / tile as i32));
        if let Some((start, n)) = owners.tiles.remove(&slot) {
            pool.give(start, n);
        }
        let Some(start) = pool.take(total as u32) else { return };
        owners.tiles.insert(slot, (start, total as u32));
        let mut bytes = Vec::with_capacity(total * 16);
        let mut rows = vec![Vec::with_capacity(tile as usize * 16); tile as usize];
        let mut at = start;
        for (i, (low, cells)) in columns.iter().enumerate() {
            bytes.extend(Self::cells_bytes(cells));
            let (cx, cy) = (tx * tile as i32 + i as i32 % tile as i32, ty * tile as i32 + i as i32 / tile as i32);
            let entry = Self::entry(at, *low, cells.len());
            at += cells.len() as u32;
            rows[i / tile as usize].extend([entry[0], entry[1], cx as u32, cy as u32].iter().flat_map(|w| w.to_le_bytes()));
        }
        self.queue.write_buffer(&self.pool, start as u64 * 16, &bytes);
        let window = (level - self.finest_live) as u64 * (self.most_side * self.most_side) as u64;
        for (row, data) in rows.iter().enumerate() {
            let (x, y) = ((tx * tile as i32).rem_euclid(side), (ty * tile as i32 + row as i32).rem_euclid(side));
            self.queue.write_buffer(&self.live, (window + (y * side + x) as u64) * 16, data);
        }
    }

    /// What the store holds, read back from the card (tests: two makings of a scene set side by side): the live
    /// index (each slot's start, lowest cell and count, and column), the baked index (each entry's start, lowest
    /// cell and count), and the pool's cells.
    #[cfg(test)]
    pub fn read_back(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> StoreRead {
        let read = |buffer: &wgpu::Buffer| -> Vec<u32> {
            let size = buffer.size();
            let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("read back"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_buffer_to_buffer(buffer, 0, &readback, 0, size);
            queue.submit([encoder.finish()]);
            readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            let words = readback.slice(..).get_mapped_range().map(|bytes| bytes.chunks_exact(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()).unwrap_or_default();
            readback.unmap();
            words
        };
        let live = read(&self.live).chunks_exact(4).map(|w| [w[0], w[1], w[2], w[3]]).collect();
        let baked = read(&self.baked).chunks_exact(2).map(|w| [w[0], w[1]]).collect();
        let pool = read(&self.pool).chunks_exact(4).map(|w| [w[0], w[1], w[2], w[3]]).collect();
        StoreRead { live, baked, pool, finest_live: self.finest_live, most_side: self.most_side as usize, extents: self.extents.iter().map(|(e, at)| (e.from, e.across, *at)).collect() }
    }

    /// For the log: the pool's cells in use.
    pub fn used(&self) -> u64 {
        self.places.lock().map_or(0, |places| POOL_CELLS as u64 - places.0.free_cells())
    }
}

#[cfg(test)]
mod tests {
    use super::Pool;

    /// Places are handed out first fit and merged with their neighbours when given back, so a pool taken
    /// whole and given back whole is one place again.
    #[test]
    fn the_pool_hands_out_and_takes_back() {
        let mut pool = Pool::new(100);
        let a = pool.take(10).unwrap();
        let b = pool.take(20).unwrap();
        let c = pool.take(30).unwrap();
        assert_eq!((a, b, c), (0, 10, 30));
        pool.give(b, 20);
        assert_eq!(pool.take(15), Some(10), "first fit, in the gap");
        pool.give(10, 15);
        pool.give(a, 10);
        pool.give(c, 30);
        assert_eq!(pool.free_cells(), 100);
        assert_eq!(pool.take(100), Some(0), "all of it one place again");
        assert_eq!(pool.take(1), None);
    }
}

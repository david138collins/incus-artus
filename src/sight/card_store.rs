//! Sight's store on the card (the regroup, phase 3.2): the cells the march reads, where it reads them.
//!
//! - **The pool:** every kept cell (16 bytes: `Cell::to_bytes`, four 32-bit words), in one buffer. Places in
//!   it are handed out and taken back on the processor (`Pool`: first fit, neighbours merged), so a region or a
//!   tile made again replaces its cells and frees the old ones.
//! - **The baked levels' index:** for each level from 32 up, a cell column's entry over the whole map, row by
//!   row (where its band starts in the pool, its lowest cell, and how many; `NOT_BAKED` till it's baked).
//! - **The live level's index:** for the live level, a window of cell columns round the camera, wrapping
//!   round (a column at `(x mod side, y mod side)`), each entry also saying which column it is, so one left
//!   from a tile the camera has moved off is never read for the column that's taken its place.
//! - **The header:** the levels' numbers (`header` in `march.wgsl`).
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
    tiles: std::collections::HashMap<(i32, i32), (u32, u32)>,
}

/// The store on the card.
pub struct CardStore {
    pub pool: wgpu::Buffer,
    pub baked: wgpu::Buffer,
    pub live: wgpu::Buffer,
    pub header: wgpu::Buffer,
    /// The baked levels' bounds, and where each one's entries start in `baked`.
    pub extents: Vec<(Extent, u32)>,
    /// The live level, and its window's cell columns a side.
    pub live_level: usize,
    /// The window's cell columns a side now (at most `most_side`, what the live index was made for).
    pub side: std::sync::atomic::AtomicI32,
    pub most_side: i32,
    /// The highest anything stands in what's on the card so far (blocks, to a cell): over it a ray that isn't
    /// coming down meets nothing (the march's ceiling, from the first region, not only once the bake is done).
    pub highest: std::sync::atomic::AtomicI32,
    queue: wgpu::Queue,
    places: Mutex<(Pool, Owners)>,
}

impl CardStore {
    /// A store for a map `half` blocks either side of the middle, its live level `live_level` with at most
    /// `side` cell columns a side. Its indexes start "not baked".
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, half: i32, live_level: usize, side: i32) -> Self {
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
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's baked index"), contents: &words.iter().flat_map(|w: &u32| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST })
        };
        // Each live entry: start, lowest cell and count, and which column it is (x, y): "none" to begin with.
        let live = {
            let mut words: Vec<u32> = Vec::with_capacity((side * side * 4) as usize);
            for _ in 0..side * side {
                words.extend([0, NOT_BAKED, i32::MIN as u32, i32::MIN as u32]);
            }
            use wgpu::util::DeviceExt;
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's live index"), contents: &words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST })
        };
        let pool = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's cells"), size: POOL_CELLS as u64 * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        // The header: the first baked level, the top, the live level, its side; then each baked level's
        // from x, from y, across, and where its entries start.
        let mut header = vec![FIRST as i32, Bake::top(half) as i32, live_level as i32, side];
        // (The live index is made for `side`; a smaller window uses the start of it.)
        for (extent, at) in &extents {
            header.extend([extent.from.0, extent.from.1, extent.across, *at as i32]);
        }
        let header = {
            use wgpu::util::DeviceExt;
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's header"), contents: &header.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST })
        };
        Self { pool, baked, live, header, extents, live_level, side: std::sync::atomic::AtomicI32::new(side), most_side: side, highest: std::sync::atomic::AtomicI32::new(0), queue: queue.clone(), places: Mutex::new((Pool::new(POOL_CELLS), Owners::default())) }
    }

    pub fn side(&self) -> i32 {
        self.side.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// The live window made `side` a side (the grid changed): its tiles given back and its entries cleared,
    /// so nothing left from the old window is ever read.
    pub fn reset_live(&self, side: i32) {
        let side = side.min(self.most_side);
        let Ok(mut places) = self.places.lock() else { return };
        let (pool, owners) = &mut *places;
        for (_, (start, n)) in owners.tiles.drain() {
            pool.give(start, n);
        }
        let none: Vec<u8> = (0..self.most_side * self.most_side).flat_map(|_| [0u32, NOT_BAKED, i32::MIN as u32, i32::MIN as u32]).flat_map(|w| w.to_le_bytes()).collect();
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
            for (row, data) in rows.into_iter().enumerate() {
                let (cx, cy) = (record.region.0 * across, record.region.1 * across + row as i32);
                if let (Some(index), Some((_, offset))) = (self.index(level, cx, cy), self.extents.get(k)) {
                    entries.push(((*offset as u64 + index as u64) * 8, data));
                }
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
    /// card at tile (`tx`, `ty`) of the live level; the place a tile had there before given back.
    pub fn put_tile(&self, tx: i32, ty: i32, tile: u32, columns: &[(i32, Vec<Cell>)]) {
        let total: usize = columns.iter().map(|(_, c)| c.len()).sum();
        let top = columns.iter().filter(|(_, c)| !c.is_empty()).map(|(low, cells)| (low + cells.len() as i32) << self.live_level).max().unwrap_or(0);
        self.highest.fetch_max(top, std::sync::atomic::Ordering::Relaxed);
        let Ok(mut places) = self.places.lock() else { return };
        let (pool, owners) = &mut *places;
        // The slot the tile takes in the wrapping window: whatever tile was there is given back.
        let side = self.side();
        let slot = (tx.rem_euclid(side / tile as i32), ty.rem_euclid(side / tile as i32));
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
        for (row, data) in rows.iter().enumerate() {
            let (x, y) = ((tx * tile as i32).rem_euclid(side), (ty * tile as i32 + row as i32).rem_euclid(side));
            self.queue.write_buffer(&self.live, ((y * side + x) as u64) * 16, data);
        }
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

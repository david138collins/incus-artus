//! Sight's bake (the regroup, phase 3.1d): the cells of 32 blocks and up, for the whole world, made once.
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
//!
//! Saving it with the world (stamped with the rules' version) and patching it for edits is 3.1e.

use super::cell::{Cell, Sums};
use super::store::{CellColumn, Maker, bands_of, sums_of};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// The first baked level: cells of 32 blocks.
pub const FIRST: usize = 5;
/// A region is one cell column of this level (1,024 blocks).
pub const REGION: usize = 10;
/// How the bake's card work is cut into pieces (columns, and cells, a piece): bigger than the live levels',
/// about a millisecond of the card each.
pub const BAKE_PIECES: (u32, u32) = (32768, 1024);
/// Regions being baked at once.
pub const BAKERS: usize = 4;

/// Not baked yet.
const NOT_YET: u16 = u16::MAX;

/// One baked level: every cell column over the map, its band's lowest cell and its compact cells.
pub struct Level {
    pub size: i32,
    /// The least cell column (absolute: cell `n` runs from `n × size`), and how many across each way.
    pub from: (i32, i32),
    pub across: i32,
    lows: Vec<i16>,
    counts: Vec<u16>,
    starts: Vec<u32>,
    cells: Vec<Cell>,
}

impl Level {
    fn new(level: usize, half: i32) -> Self {
        let size = 1 << level;
        let from = (-half).div_euclid(size);
        let across = (half - 1).div_euclid(size) - from + 1;
        let n = (across * across) as usize;
        Self { size, from: (from, from), across, lows: vec![0; n], counts: vec![NOT_YET; n], starts: vec![0; n], cells: Vec::new() }
    }

    fn index(&self, cx: i32, cy: i32) -> Option<usize> {
        let (x, y) = (cx - self.from.0, cy - self.from.1);
        ((0..self.across).contains(&x) && (0..self.across).contains(&y)).then(|| (y * self.across + x) as usize)
    }

    /// A cell column's band (its lowest cell, and its cells), if it's baked; `Some((0, []))` past the map.
    pub fn column(&self, cx: i32, cy: i32) -> Option<(i32, &[Cell])> {
        let Some(k) = self.index(cx, cy) else { return Some((0, &[])) };
        (self.counts[k] != NOT_YET).then(|| (self.lows[k] as i32, &self.cells[self.starts[k] as usize..self.starts[k] as usize + self.counts[k] as usize]))
    }

    fn put(&mut self, cx: i32, cy: i32, column: &CellColumn) {
        let Some(k) = self.index(cx, cy) else { return };
        self.lows[k] = column.low as i16;
        self.counts[k] = column.sums.len() as u16;
        self.starts[k] = self.cells.len() as u32;
        self.cells.extend(column.sums.iter().map(|sums| sums.compact(self.size as u32)));
    }

    pub fn cells(&self) -> usize {
        self.cells.len()
    }
}

/// A cell column made from its four children (each `child_size` a cell), exactly: each cell the sum of the
/// eight under it, the cells left out of the children's bands included.
pub fn parent_of(children: [&CellColumn; 4], child_size: i32) -> CellColumn {
    let present: Vec<&&CellColumn> = children.iter().filter(|c| !c.sums.is_empty()).collect();
    if present.is_empty() {
        // All past the map, or nothing kept below (and all solid under it): kept as its lowest child.
        let low = children.iter().map(|c| c.low).min().unwrap_or(0).div_euclid(2);
        return CellColumn { low, sums: Vec::new() };
    }
    let low = children.iter().map(|c| c.low).min().unwrap_or(0).div_euclid(2);
    let high = present.iter().map(|c| c.high()).max().unwrap_or(0).div_euclid(2);
    let sums = (low..=high)
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

/// The whole bake.
pub struct Bake {
    /// The levels from cells of 32 up.
    pub levels: Vec<Level>,
    /// The exact sums from 1,024 up, by level and cell column (the levels above are made from them).
    sums: HashMap<(usize, i32, i32), CellColumn>,
    pub regions: usize,
    pub start_regions: usize,
    pub regions_done: usize,
    pub started: Instant,
    /// Seconds to the start bake, and to all of it.
    pub start_done: Option<f32>,
    pub all_done: Option<f32>,
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

    pub fn level(&self, level: usize) -> &Level {
        &self.levels[level - FIRST]
    }

    /// A region's cell columns, level by level from 32 to 1,024, put in; then every level above built where
    /// all its children are baked.
    fn take(&mut self, region: (i32, i32), made: Vec<Vec<CellColumn>>) {
        for (k, columns) in made.iter().enumerate() {
            let level = FIRST + k;
            let across = 1 << (REGION - level);
            let (from_x, from_y) = (region.0 * across, region.1 * across);
            for (i, column) in columns.iter().enumerate() {
                self.levels[k].put(from_x + i as i32 % across, from_y + i as i32 / across, column);
            }
        }
        let top_column = made.last().and_then(|columns| columns.first()).cloned().unwrap_or_default();
        self.sums.insert((REGION, region.0, region.1), top_column);
        // Up the levels while all four children are there.
        let (mut cx, mut cy) = region;
        for level in REGION + 1..FIRST + self.levels.len() {
            let (px, py) = (cx.div_euclid(2), cy.div_euclid(2));
            let below = &self.levels[level - 1 - FIRST];
            let mut children = Vec::with_capacity(4);
            for k in 0..4 {
                let (x, y) = (px * 2 + k % 2, py * 2 + k / 2);
                match self.sums.get(&(level - 1, x, y)) {
                    Some(column) => children.push(column.clone()),
                    None if below.index(x, y).is_none() => children.push(CellColumn::default()),
                    None => return,
                }
            }
            let parent = parent_of([&children[0], &children[1], &children[2], &children[3]], 1 << (level - 1));
            self.levels[level - FIRST].put(px, py, &parent);
            self.sums.insert((level, px, py), parent);
            (cx, cy) = (px, py);
        }
    }

    /// For the log.
    pub fn report(&self) -> String {
        let cells: usize = self.levels.iter().map(Level::cells).sum();
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

/// A bake under way: the bake, the regions still to do (nearest first), and the switch to stop it.
pub struct Baking {
    pub bake: Mutex<Bake>,
    queue: Mutex<VecDeque<(i32, i32)>>,
    pub stop: AtomicBool,
}

/// Starts baking the map (`half` blocks either side of the middle) outward from `spawn`: `BAKERS` loops on
/// the loading lane, each taking the next region till there are none or it's told to stop.
pub fn start(maker: Maker, odd: bool, half: i32, spawn: (i32, i32)) -> (Arc<Baking>, Vec<crate::loading::Job<()>>) {
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
    let levels = (FIRST..=Bake::top(half)).map(|level| Level::new(level, half)).collect();
    let bake = Bake { levels, sums: HashMap::new(), regions: all.len(), start_regions, regions_done: 0, started: Instant::now(), start_done: None, all_done: None };
    let baking = Arc::new(Baking { bake: Mutex::new(bake), queue: Mutex::new(all.into_iter().map(|(_, r)| r).collect()), stop: AtomicBool::new(false) });
    let jobs = (0..BAKERS)
        .map(|_| {
            let (baking, maker) = (baking.clone(), maker.clone());
            crate::loading::submit(crate::loading::Tier::Whenever, 0.0, move || {
                while !baking.stop.load(Ordering::Relaxed) {
                    let Some(region) = baking.queue.lock().ok().and_then(|mut queue| queue.pop_front()) else { break };
                    let made = bake_region(&maker, odd, region);
                    let Ok(mut bake) = baking.bake.lock() else { break };
                    bake.take(region, made);
                    bake.regions_done += 1;
                    let seconds = bake.started.elapsed().as_secs_f32();
                    if bake.start_done.is_none() && bake.regions_done >= bake.start_regions {
                        bake.start_done = Some(seconds);
                    }
                    if bake.regions_done == bake.regions {
                        bake.all_done = Some(seconds);
                    }
                }
            })
        })
        .collect();
    (baking, jobs)
}

/// A region's cell columns, level by level from 32 to 1,024 (x fastest at each): the cells of 32 made on the
/// card, the rest added up from them.
pub fn bake_region(maker: &Maker, odd: bool, region: (i32, i32)) -> Vec<Vec<CellColumn>> {
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
    let first: Vec<CellColumn> = bands.into_iter().map(|(low, count)| CellColumn { low, sums: (0..count).filter_map(|_| made.next()).collect() }).collect();
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

/// Every bake under way stops (a new world, or the game closing).
pub fn stop(baking: &Baking) {
    baking.stop.store(true, Ordering::Relaxed);
}

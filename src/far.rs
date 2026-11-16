//! The far cells (plan 2.2, step 1): the world as the far eye sees it, cubes of 32 blocks and every level above
//! (64, 128, … up to four cells covering the whole map), summed from what the doorway answers.
//!
//! **Old's cell, carried as a result** (`old/src/sight/cell.rs`): each cell keeps exact sums in whole numbers (how many
//! of its blocks are solid and how many water; the solid faces showing, counted by the six ways they look; their
//! colours added; their brightness and height added and squared, for the spreads). Every level is exactly the one below
//! added, so a coarse cell is the exact average of its children by construction. What's kept is the compact form (16
//! bytes), worked out from the sums by old's one rule, and only each cell column's surface band: below it every cell is
//! solid with no face showing, above it empty.
//!
//! **Old's rule for a block's faces** (`old/src/sight/cell.wgsl`): a solid block's face shows where the block beside it
//! isn't solid (air or water); below the world counts as stone, so nothing shows underneath; past the map counts as air,
//! so the map's edge shows its sides. A face's colour is its block's top colour looking up, its sides' every other way
//! (`data/far colours.toml`, old's colours, carried until the Registrar).
//!
//! **Summed on the processor, a column at a time in runs** [David, 2026-11-12: "yes, i think we'd be right to use the
//! processor for summing the far cells. its a 1 time event, and it's quick"]: the loading lane, not the frame; the
//! doorway's columns are already on the processor; and the card stays free for drawing. Old summed on the card, block
//! by block. Here a column's solid blocks are runs (stone, the layers under the top, the top), so its faces against
//! each neighbour are one range, added in one step.
//!
//! A region is one cell column of 1,024 (32 × 32 cell columns of 32), made whole at once, as old's bake made it. The
//! loading lane (`Lane`) sums them on threads of their own, nearest the eye first, and hands each over as it's done:
//! the far eye puts it on the card (`bricks.rs`).

use crate::doorway::{Block, Change, Doorway, Run, WATER};
use std::collections::HashMap;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Instant;

/// The first level: cells of 32 blocks.
pub const FIRST: usize = 5;
/// A region is one cell column of this level (1,024 blocks).
pub const REGION: usize = 10;
/// Colours and brightness are in 1,024ths of linear light.
pub const FULL: u64 = 1023;

/// The ways a face looks, in the order the sums keep them: up, down, north, south, east, west (x north, y east, z up).
pub const UP: usize = 0;

/// Every block's colour from far: its top's and its sides', by block.
pub struct FarColours {
    blocks: Vec<Option<([u64; 3], [u64; 3])>>,
}

#[derive(Deserialize)]
struct ColoursFile {
    block: Vec<ColourFile>,
}

#[derive(Deserialize)]
struct ColourFile {
    name: String,
    top: [u64; 3],
    side: [u64; 3],
}

impl FarColours {
    /// `data/far colours.toml`, each block found by its name through the doorway.
    pub fn load(doorway: &mut Doorway) -> Result<Self, String> {
        let path = crate::session::project_folder().join("data").join("far colours.toml");
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let file: ColoursFile = toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut blocks = Vec::new();
        for colour in file.block {
            let block = doorway.block(&colour.name) as usize;
            if blocks.len() <= block {
                blocks.resize(block + 1, None);
            }
            blocks[block] = Some((colour.top, colour.side));
        }
        Ok(Self { blocks })
    }

    /// Every block's colour as light (0 to 1), its top's and then its sides', by the block's number: for the near eye,
    /// whose faces are the colour the far cells are summed from.
    pub fn as_light(&self) -> Vec<[f32; 4]> {
        (0..self.blocks.len().max(1) as Block)
            .flat_map(|block| [true, false].map(|up| {
                let colour = self.colour(block, up);
                [colour[0] as f32 / FULL as f32, colour[1] as f32 / FULL as f32, colour[2] as f32 / FULL as f32, 1.0]
            }))
            .collect()
    }

    /// A face's colour: its block's top looking up, its sides' every other way (a block with none: mid grey, as old).
    fn colour(&self, block: Block, up: bool) -> [u64; 3] {
        self.blocks.get(block as usize).copied().flatten().map_or([FULL / 2; 3], |(top, side)| if up { top } else { side })
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
    /// Solid faces showing, by the way they look.
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

    /// `count` faces looking way `d`, of one colour, at the heights `from` to `to` (one each).
    fn add_faces(&mut self, d: usize, from: i32, to: i32, colour: [u64; 3]) {
        let count = (to - from + 1) as u64;
        let light = light(colour);
        let squares = |m: i64| if m < 0 { 0 } else { (m * (m + 1) * (2 * m + 1) / 6) as u64 };
        self.faces[d] += count;
        for (sum, c) in self.colour.iter_mut().zip(colour) {
            *sum += count * c;
        }
        self.light += count * light;
        self.light_squared += count * light * light;
        self.height += (from as u64 + to as u64) * count / 2;
        self.height_squared += squares(to as i64) - squares(from as i64 - 1);
    }

    /// The compact form the far eye reads, for a cell `size` blocks a side (old's one rule).
    pub fn compact(&self, size: u32) -> Cell {
        let volume = (size as u64).pow(3);
        let showing = self.showing();
        let mut cell = Cell { solid: share(self.solid, volume, 255) as u8, water: share(self.water, volume, 255) as u8, ..Cell::default() };
        if showing == 0 {
            return cell;
        }
        let colour = self.colour.map(|c| share(c, showing, 1));
        cell.colour = (colour[0] << 20 | colour[1] << 10 | colour[2]) as u32;
        // Each way's share in 255ths, rounded as running totals so the six add to 255 exactly; the first five kept.
        let (mut before, mut running) = (0, 0);
        for (d, &n) in self.faces.iter().enumerate().take(5) {
            running += n;
            let to = share(running, showing, 255);
            cell.faces[d] = (to - before) as u8;
            before = to;
        }
        cell.light_spread = spread(self.light, self.light_squared, showing).min(255) as u8;
        // The heights' spread in 16ths of a block.
        cell.height_spread = spread(self.height * 16, self.height_squared * 256, showing).min(u16::MAX as u64) as u16;
        cell.showing = share(showing, (size as u64).pow(2), 256).min(u16::MAX as u64) as u16;
        cell
    }
}

/// `part` of `whole` in `unit`s, rounded (halves up).
fn share(part: u64, whole: u64, unit: u64) -> u64 {
    ((part as u128 * unit as u128 * 2 + whole as u128) / (2 * whole as u128)) as u64
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

/// A cell as the far eye reads it: 16 bytes (old's).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Cell {
    /// The share of its blocks that are solid, and water, in 255ths.
    pub solid: u8,
    pub water: u8,
    /// Each way's share of its faces (255ths): up, down, north, south, east. West's is what's left.
    pub faces: [u8; 5],
    /// How much the faces' brightness spreads (standard deviation, 1,024ths of linear light, at most 255).
    pub light_spread: u8,
    /// The faces' average colour, 1,024ths of linear light, red, green and blue in ten bits each.
    pub colour: u32,
    /// How much the faces' heights spread (standard deviation, 16ths of a block).
    pub height_spread: u16,
    /// How much surface shows: faces over the cell's floor area, in 256ths.
    pub showing: u16,
}

/// A cell column's surface band: its cells from `low` up. Below it every cell is solid with no face showing; above it
/// every cell is empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellColumn {
    pub low: i32,
    pub sums: Vec<Sums>,
    /// The height in blocks of the highest thing in the column, ground or water: the top of its highest block (0 if
    /// there's nothing). True to the block, where the band's top is rounded up to the cell: the height table's number
    /// (`bricks.rs`), which lets a ray skimming low over flat ground clear a coarse column in one hop.
    pub highest: i32,
}

impl CellColumn {
    /// Its highest kept cell (`low − 1` if none).
    pub fn high(&self) -> i32 {
        self.low + self.sums.len() as i32 - 1
    }

    /// The cell `index` up, the ones left out of the band included.
    pub fn at(&self, index: i32, size: i32) -> Sums {
        if index >= self.low && index <= self.high() {
            self.sums[(index - self.low) as usize]
        } else if index < self.low && index >= 0 {
            Sums { solid: (size as u64).pow(3), ..Sums::default() }
        } else {
            Sums::default()
        }
    }

    /// A column's cells, every one, trimmed to its band.
    fn trimmed(cells: Vec<Sums>, size: i32, highest: i32) -> Self {
        let full = Sums { solid: (size as u64).pow(3), ..Sums::default() };
        let low = cells.iter().position(|sums| *sums != full).unwrap_or(cells.len());
        let Some(high) = cells.iter().rposition(|sums| *sums != Sums::default()) else { return Self { highest, ..Self::default() } };
        if high < low {
            return Self { low: low as i32, sums: Vec::new(), highest };
        }
        Self { low: low as i32, sums: cells[low..=high].to_vec(), highest }
    }
}

/// A cell column made from its four children (each `child_size` a cell), exactly: each cell the sum of the eight under
/// it, the cells left out of the children's bands included (old's `parent_of`).
pub fn parent_of(children: [&CellColumn; 4], child_size: i32) -> CellColumn {
    let low = children.iter().map(|c| c.low).min().unwrap_or(0).div_euclid(2);
    let highest = children.iter().map(|c| c.highest).max().unwrap_or(0);
    let Some(high) = children.iter().filter(|c| !c.sums.is_empty()).map(|c| c.high()).max() else {
        return CellColumn { low, sums: Vec::new(), highest };
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
    CellColumn { low, sums, highest }
}

/// The regions a map covers: the least (north and east), and how many across.
pub fn regions_of(doorway: &Doorway) -> ([i32; 2], i32) {
    let (least, _) = doorway.grid();
    let size = 1 << REGION;
    let from = [least[0].div_euclid(size), least[1].div_euclid(size)];
    let to = [(least[0] + doorway.across() - 1).div_euclid(size), (least[1] + doorway.across() - 1).div_euclid(size)];
    (from, (to[0] - from[0] + 1).max(to[1] - from[1] + 1))
}

/// A region summed: its place, and its cell columns from 32 to 1,024 (or why it couldn't be).
pub type Summed = ([i32; 2], Result<Vec<Vec<CellColumn>>, String>);

/// The loading lane's summing: every region, on threads of its own, nearest a place first (square rings, as old's bake
/// went), each handed over as it's done.
pub struct Lane {
    receiver: mpsc::Receiver<Summed>,
    /// The regions to sum, those handed over, and when it started.
    pub total: usize,
    pub done: usize,
    pub started: Instant,
    /// When the eighth nearest were all in (seconds from the start), once they are.
    pub eighth: Option<f32>,
    order: Arc<Vec<[i32; 2]>>,
    arrived: Vec<bool>,
}

impl Lane {
    /// Starts summing the map behind the doorway in `folder`, nearest `near` (north, east) first, on `threads`.
    pub fn start(folder: PathBuf, near: [f32; 2], threads: usize) -> Result<Self, String> {
        let doorway = Doorway::open(&folder)?;
        let (from, across) = regions_of(&doorway);
        let size = (1 << REGION) as f32;
        let middle = [(near[0] / size).floor() as i32, (near[1] / size).floor() as i32];
        let mut order: Vec<[i32; 2]> = (0..across * across).map(|k| [from[0] + k % across, from[1] + k / across]).collect();
        order.sort_by_key(|r| ((r[0] - middle[0]).abs().max((r[1] - middle[1]).abs()), r[0], r[1]));
        let order = Arc::new(order);
        let next = Arc::new(AtomicUsize::new(0));
        let (sender, receiver) = mpsc::channel();
        for _ in 0..threads.max(1) {
            let (order, next, sender, folder) = (order.clone(), next.clone(), sender.clone(), folder.clone());
            std::thread::Builder::new()
                .name("summing the far cells".into())
                .spawn(move || {
                    let made = Doorway::open(&folder).and_then(|mut doorway| FarColours::load(&mut doorway).map(|colours| (doorway, colours)));
                    let (doorway, colours) = match made {
                        Ok(made) => made,
                        Err(error) => {
                            let _ = sender.send(([0, 0], Err(error)));
                            return;
                        }
                    };
                    loop {
                        let k = next.fetch_add(1, Ordering::Relaxed);
                        let Some(&r) = order.get(k) else { break };
                        if sender.send((r, region(&doorway, &colours, r))).is_err() {
                            break;
                        }
                    }
                })
                .map_err(|error| format!("a thread for summing the far cells: {error}"))?;
        }
        log::info!("Summing the far cells: {} regions on {} threads, nearest {:.0} {:.0} first", order.len(), threads.max(1), near[0], near[1]);
        Ok(Self { receiver, total: order.len(), done: 0, started: Instant::now(), eighth: None, arrived: vec![false; order.len()], order })
    }

    /// Up to `most` regions summed since last asked.
    pub fn take(&mut self, most: usize) -> Vec<([i32; 2], Vec<Vec<CellColumn>>)> {
        let mut taken = Vec::new();
        while taken.len() < most {
            let Ok((r, summed)) = self.receiver.try_recv() else { break };
            match summed {
                Ok(columns) => {
                    self.done += 1;
                    if let Some(k) = self.order.iter().position(|o| *o == r) {
                        self.arrived[k] = true;
                    }
                    taken.push((r, columns));
                }
                Err(error) => log::error!("A far region ({r:?}) couldn't be summed: {error}"),
            }
        }
        if self.eighth.is_none() && self.arrived[..self.total / 8].iter().all(|a| *a) {
            let seconds = self.started.elapsed().as_secs_f32();
            self.eighth = Some(seconds);
            log::info!("The far cells: the eighth nearest ({} regions) in {seconds:.1} s", self.total / 8);
        }
        if !taken.is_empty() && self.done == self.total {
            log::info!("The far cells: all {} regions in {:.1} s", self.total, self.started.elapsed().as_secs_f32());
        }
        taken
    }
}

/// A region's cell columns, level by level from 32 to 1,024 (north fastest at each): its 32 × 32 cell columns of 32
/// summed from the doorway's columns (and those one round it, for the faces at its edge), then each level above added.
pub fn region(doorway: &Doorway, colours: &FarColours, region: [i32; 2]) -> Result<Vec<Vec<CellColumn>>, String> {
    let first = cells_of_32(doorway, colours, region)?;
    let mut levels = vec![first];
    for level in FIRST + 1..=REGION {
        let below = levels.last().map(Vec::as_slice).unwrap_or_default();
        let child_across = 1 << (REGION - level + 1);
        let across = child_across / 2;
        let columns = (0..across * across)
            .map(|i| {
                let (x, y) = (i % across, i / across);
                let child = |a: usize, b: usize| &below[(y * 2 + b) * child_across + x * 2 + a];
                parent_of([child(0, 0), child(1, 0), child(0, 1), child(1, 1)], 1 << (level - 1))
            })
            .collect();
        levels.push(columns);
    }
    Ok(levels)
}

/// A region's cell columns of 32.
fn cells_of_32(doorway: &Doorway, colours: &FarColours, region: [i32; 2]) -> Result<Vec<CellColumn>, String> {
    let size = 1 << REGION;
    cells(doorway, colours, 1 << FIRST, [region[0] * size, region[1] * size], (size >> FIRST) as usize, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new))
}

/// The cell columns of `cell` blocks a side over a square of the map, `across` columns each way from the block `at`
/// (north fastest): summed from the doorway's columns (and those one round the square, for the faces at its edge),
/// each tile asked of `tile` (the doorway's own reading, or a reader that keeps the last few: the ring's maker asks
/// for the same tiles again and again).
pub fn cells(doorway: &Doorway, colours: &FarColours, cell: i32, at: [i32; 2], across: usize, tile_of: impl FnMut(i32, i32) -> Result<std::rc::Rc<crate::doorway::Tile>, String>) -> Result<Vec<CellColumn>, String> {
    cells_by(doorway, colours, cell, at, across, tile_of, false)
}

/// `cells`, with every column summed from its runs if told (`all_by_runs`: for a test; plain ground then comes out as
/// its own quick road makes it). A column the scatter stage changed, or one beside such a column, always is: its
/// blocks can stand over air, and a face shows wherever the block beside it isn't solid.
fn cells_by(doorway: &Doorway, colours: &FarColours, cell: i32, at: [i32; 2], across: usize, mut tile_of: impl FnMut(i32, i32) -> Result<std::rc::Rc<crate::doorway::Tile>, String>, all_by_runs: bool) -> Result<Vec<CellColumn>, String> {
    let size = cell * across as i32;
    // The square's columns and one round it: ground (−1 past the map), water, profile.
    let side = (size + 2) as usize;
    let mut grounds = vec![-1i32; side * side];
    let mut waters = vec![0i32; side * side];
    let mut profiles = vec![0u32; side * side];
    // The columns the scatter stage changed, each as its solid runs: where they are in `staged_runs` (none: unchanged).
    let mut staged: Vec<(u32, u32)> = Vec::new();
    let mut staged_runs: Vec<Run> = Vec::new();
    // What was placed through the doorway in the tiles read (build 207).
    let mut changes: Vec<Change> = Vec::new();
    let (least, tile) = doorway.grid();
    let tiles = |from: i32, to: i32, axis: usize| (from - least[axis]).div_euclid(tile)..=(to - least[axis]).div_euclid(tile);
    for tx in tiles(at[0] - 1, at[0] + size, 0) {
        for ty in tiles(at[1] - 1, at[1] + size, 1) {
            let answer = tile_of(tx, ty)?;
            changes.extend_from_slice(doorway.changes_in(tx, ty));
            let base = [least[0] + tx * tile, least[1] + ty * tile];
            let (n0, n1) = (base[0].max(at[0] - 1), (base[0] + tile).min(at[0] + size + 1));
            let (e0, e1) = (base[1].max(at[1] - 1), (base[1] + tile).min(at[1] + size + 1));
            for e in e0..e1 {
                for n in n0..n1 {
                    let (ground, water, profile) = answer.raw((n - base[0]) as usize, (e - base[1]) as usize);
                    let k = (n - at[0] + 1) as usize + (e - at[1] + 1) as usize * side;
                    grounds[k] = ground;
                    waters[k] = water;
                    profiles[k] = profile;
                    if answer.any_staged() && let Some(runs) = answer.staged((n - base[0]) as usize, (e - base[1]) as usize) {
                        if staged.is_empty() {
                            staged = vec![(0, 0); side * side];
                        }
                        staged[k] = (staged_runs.len() as u32, (staged_runs.len() + runs.len()) as u32);
                        staged_runs.extend_from_slice(runs);
                    }
                }
            }
        }
    }
    // What was placed through the doorway (build 207): each column of the square, and round it, that has some is summed
    // from its runs as they are now (a whole block placed its block; a cell of pieces its main material, where it isn't
    // solid already), as the scatter stage's columns are. An export column keeps its water where nothing solid stands
    // (`wet`); a stage column has none, as before.
    let mut wet: Vec<bool> = Vec::new();
    let placed = placed_cells(&changes, at, size);
    if !placed.is_empty() {
        if staged.is_empty() {
            staged = vec![(0, 0); side * side];
        }
        wet = vec![false; side * side];
        for (column, cells) in placed {
            let k = (column[0] - at[0] + 1) as usize + (column[1] - at[1] + 1) as usize * side;
            if grounds[k] < 0 {
                continue;
            }
            let (from, to) = staged[k];
            let was_staged = to > from;
            let base = if was_staged { staged_runs[from as usize..to as usize].to_vec() } else { doorway.solid_runs(grounds[k], profiles[k])? };
            let runs = with_placed(base, &cells);
            staged[k] = (staged_runs.len() as u32, (staged_runs.len() + runs.len()) as u32);
            staged_runs.extend_from_slice(&runs);
            wet[k] = !was_staged;
        }
    }
    // A column's solid runs: the stage's, or the export's own (none past the map).
    let is_staged = |k: usize| staged.get(k).is_some_and(|(from, to)| to > from);
    let runs_of = |k: usize| -> Result<Vec<Run>, String> {
        if is_staged(k) {
            Ok(staged_runs[staged[k].0 as usize..staged[k].1 as usize].to_vec())
        } else if grounds[k] < 0 {
            Ok(Vec::new())
        } else {
            doorway.solid_runs(grounds[k], profiles[k])
        }
    };
    let height = doorway.height();
    let up = (height / cell) as usize;
    let mut columns = Vec::with_capacity(across * across);
    for cy in 0..across {
        for cx in 0..across {
            let mut sums = vec![Sums::default(); up];
            let mut highest = 0;
            for j in 0..cell as usize {
                for i in 0..cell as usize {
                    let k = (cx * cell as usize + i + 1) + (cy * cell as usize + j + 1) * side;
                    if grounds[k] < 0 {
                        continue;
                    }
                    // A column the scatter stage changed, or one beside it: summed from its runs.
                    let around = [k + 1, k - 1, k + side, k - side];
                    if all_by_runs || (!staged.is_empty() && (is_staged(k) || around.iter().any(|&other| is_staged(other)))) {
                        let own = runs_of(k)?;
                        let beside = [runs_of(around[0])?, runs_of(around[1])?, runs_of(around[2])?, runs_of(around[3])?];
                        // The export's water, in what's open over its ground (a column the stage changed has none).
                        let water = if is_staged(k) && !wet.get(k).copied().unwrap_or(false) { (1, 0) } else { (grounds[k] + 1, waters[k].max(doorway.sea_level).min(height) - 1) };
                        highest = highest.max(own.last().map_or(0, |run| run.to)).max(water.1 + 1);
                        add_column_runs(&mut sums, cell, height, colours, &own, water, [&beside[0], &beside[1], &beside[2], &beside[3]]);
                        continue;
                    }
                    // The top of its highest block: the ground's, or its water's if that stands over it (as `add_column`
                    // fills it: to the sea or the river's or lake's top).
                    highest = highest.max(grounds[k] + 1).max(waters[k].max(doorway.sea_level).min(height));
                    let beside = [grounds[k + 1], grounds[k - 1], grounds[k + side], grounds[k - side]];
                    add_column(&mut sums, cell, doorway, colours, (grounds[k], waters[k], profiles[k]), beside)?;
                }
            }
            columns.push(CellColumn::trimmed(sums, cell, highest));
        }
    }
    Ok(columns)
}

/// One column's blocks added to its cells of `cell` blocks (`cells`, bottom up): its ground, water and profile, and
/// the grounds beside it (north, south, east, west; −1 past the map).
fn add_column(cells: &mut [Sums], cell: i32, doorway: &Doorway, colours: &FarColours, (ground, water, number): (i32, i32, u32), beside: [i32; 4]) -> Result<(), String> {
    let height = doorway.height();
    if ground >= height {
        return Err(format!("a column's ground ({ground}) is above the world's height"));
    }
    let (top, under) = doorway.profile(number).ok_or_else(|| format!("profile {number} isn't in the world"))?;
    if top == WATER || under.iter().any(|(block, _)| *block == WATER) {
        return Err(format!("profile {number} has water among its solid blocks, which the far cells don't count"));
    }
    // Solid from bedrock to the ground; water above it to the sea or the river's or lake's top.
    add_range(cells, cell, 0, ground, |sums, from, to| sums.solid += (to - from + 1) as u64);
    let water_top = water.max(doorway.sea_level).min(height) - 1;
    add_range(cells, cell, ground + 1, water_top, |sums, from, to| sums.water += (to - from + 1) as u64);
    // The top block's face looking up (above it is water or air).
    let up = colours.colour(top, true);
    add_range(cells, cell, ground, ground, |sums, from, to| sums.add_faces(UP, from, to, up));
    // The column's solid blocks bottom up, as runs: stone, each layer under the top (deepest first), the top.
    let deepest = under.last().map_or(0, |(_, depth)| *depth);
    let mut runs: [(i32, i32, Block); 6] = [(0, -1, 0); 6];
    let mut count = 0;
    let mut push = |from: i32, to: i32, block: Block| {
        if to >= from.max(0) && count < runs.len() {
            runs[count] = (from.max(0), to, block);
            count += 1;
        }
    };
    push(0, ground - deepest - 1, crate::doorway::STONE);
    for (k, &(block, depth)) in under.iter().enumerate().rev() {
        let above = if k == 0 { 0 } else { under[k - 1].1 };
        push(ground - depth, ground - above - 1, block);
    }
    push(ground, ground, top);
    if under.len() + 2 > runs.len() {
        return Err(format!("profile {number} has more layers than the far cells read"));
    }
    // Each side: the faces from just above the neighbour's ground (bedrock if it's past the map) up to the ground.
    for (d, &next) in beside.iter().enumerate() {
        let from = (next + 1).max(0);
        if from > ground {
            continue;
        }
        for &(run_from, run_to, block) in &runs[..count] {
            let (a, b) = (run_from.max(from), run_to.min(ground));
            if a <= b {
                let side = colours.colour(block, false);
                add_range(cells, cell, a, b, |sums, from, to| sums.add_faces(2 + d, from, to, side));
            }
        }
    }
    Ok(())
}

/// One column's blocks added to its cells of `cell` blocks, from its solid runs (bedrock up, air between them) and the
/// runs of the columns beside it (north, south, east, west; none past the map): for a column the scatter stage
/// changed, whose blocks can stand over air. Old's rule for faces, as `add_column` keeps it for plain ground: a solid
/// block's face shows where the block beside it isn't solid; below the world counts as stone; past the map as air.
/// `water`: the heights (from, to, both counted) the export's water fills where nothing solid stands.
fn add_column_runs(cells: &mut [Sums], cell: i32, height: i32, colours: &FarColours, runs: &[Run], water: (i32, i32), beside: [&[Run]; 4]) {
    for run in runs {
        add_range(cells, cell, run.from, run.to - 1, |sums, from, to| sums.solid += (to - from + 1) as u64);
    }
    // The heights of `from` up to `to` (not including it) that `others` leave open, each stretch given to `open`.
    fn gaps(from: i32, to: i32, others: &[Run], mut open: impl FnMut(i32, i32)) {
        let mut z = from;
        for other in others {
            if other.to <= z {
                continue;
            }
            if other.from >= to {
                break;
            }
            if other.from > z {
                open(z, other.from - 1);
            }
            z = z.max(other.to);
            if z >= to {
                return;
            }
        }
        if z < to {
            open(z, to - 1);
        }
    }
    gaps(water.0, water.1 + 1, runs, |from, to| add_range(cells, cell, from, to, |sums, from, to| sums.water += (to - from + 1) as u64));
    for (i, run) in runs.iter().enumerate() {
        // Its top block's face looking up, if nothing solid stands on it; its bottom block's looking down, if it
        // stands over air (bedrock's never shows).
        if run.to < height && runs.get(i + 1).is_none_or(|next| next.from > run.to) || run.to >= height {
            let up = colours.colour(run.block, true);
            add_range(cells, cell, run.to - 1, run.to - 1, |sums, from, to| sums.add_faces(UP, from, to, up));
        }
        if run.from > 0 && (i == 0 || runs[i - 1].to < run.from) {
            let down = colours.colour(run.block, false);
            add_range(cells, cell, run.from, run.from, |sums, from, to| sums.add_faces(1, from, to, down));
        }
        // Each side: its faces wherever the column beside it isn't solid.
        let side = colours.colour(run.block, false);
        for (d, other) in beside.iter().enumerate() {
            gaps(run.from, run.to, other, |from, to| add_range(cells, cell, from, to, |sums, from, to| sums.add_faces(2 + d, from, to, side)));
        }
    }
}

/// What was placed through the doorway over the square of `size` columns from `at` and the columns round it (build
/// 207), by column: each cell touched, its block as the far cells count it, and whether it's a whole block (which takes
/// its cell whatever was there) or pieces (which fill it only where it's empty). A cell of pieces counts as their main
/// material, the block with the most of its octants (the first placed of those that tie), never as air: old's sight
/// counted a cell of pieces so. Where two pieces would share an octant the first placed keeps it; a whole block placed
/// takes its cell, and pieces after it find no room (as the near eye has them).
fn placed_cells(changes: &[Change], at: [i32; 2], size: i32) -> HashMap<[i32; 2], Vec<(i32, Block, bool)>> {
    struct Placed {
        whole: Option<Block>,
        octants: [Option<Block>; 8],
        order: Vec<Block>,
    }
    let inside = |cell: [i32; 3]| cell[0] >= at[0] - 1 && cell[0] <= at[0] + size && cell[1] >= at[1] - 1 && cell[1] <= at[1] + size;
    let mut cells: HashMap<[i32; 3], Placed> = HashMap::new();
    for change in changes {
        match (change.octants(), *change) {
            (None, Change::Block { cell, block }) => {
                if inside(cell) {
                    cells.insert(cell, Placed { whole: Some(block), octants: [None; 8], order: Vec::new() });
                }
            }
            (Some((octants, block)), _) => {
                for octant in octants {
                    let cell = octant.map(|half| half.div_euclid(2));
                    if !inside(cell) {
                        continue;
                    }
                    let placed = cells.entry(cell).or_insert_with(|| Placed { whole: None, octants: [None; 8], order: Vec::new() });
                    if placed.whole.is_some() {
                        continue;
                    }
                    let k = (octant[0].rem_euclid(2) + octant[1].rem_euclid(2) * 2 + octant[2].rem_euclid(2) * 4) as usize;
                    if placed.octants[k].is_none() {
                        placed.octants[k] = Some(block);
                        if !placed.order.contains(&block) {
                            placed.order.push(block);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let mut columns: HashMap<[i32; 2], Vec<(i32, Block, bool)>> = HashMap::new();
    for (cell, placed) in cells {
        let counted = match placed.whole {
            Some(block) => Some((block, true)),
            None => {
                let mut best: Option<(Block, usize)> = None;
                for &block in &placed.order {
                    let n = placed.octants.iter().filter(|o| **o == Some(block)).count();
                    if best.is_none_or(|(_, most)| n > most) {
                        best = Some((block, n));
                    }
                }
                best.map(|(block, _)| (block, false))
            }
        };
        if let Some((block, whole)) = counted {
            columns.entry([cell[0], cell[1]]).or_default().push((cell[2], block, whole));
        }
    }
    columns
}

/// A column's solid runs (bedrock up, air between) with what was placed in it (`placed_cells`' for the column): a
/// whole block takes its cell; pieces fill a cell only where nothing solid is. Runs of one block that meet are joined.
fn with_placed(mut runs: Vec<Run>, placed: &[(i32, Block, bool)]) -> Vec<Run> {
    for &(z, block, whole) in placed {
        match runs.iter().position(|run| run.from <= z && z < run.to) {
            Some(i) => {
                if !whole || runs[i].block == block {
                    continue;
                }
                let run = runs[i];
                let mut split = Vec::new();
                if run.from < z {
                    split.push(Run { block: run.block, from: run.from, to: z });
                }
                split.push(Run { block, from: z, to: z + 1 });
                if z + 1 < run.to {
                    split.push(Run { block: run.block, from: z + 1, to: run.to });
                }
                runs.splice(i..=i, split);
            }
            None => {
                let i = runs.iter().position(|run| run.from > z).unwrap_or(runs.len());
                runs.insert(i, Run { block, from: z, to: z + 1 });
            }
        }
    }
    let mut joined: Vec<Run> = Vec::with_capacity(runs.len());
    for run in runs {
        match joined.last_mut() {
            Some(last) if last.to == run.from && last.block == run.block => last.to = run.to,
            _ => joined.push(run),
        }
    }
    joined
}

/// The heights `from` to `to` (from bedrock up), split at the edges of the cells of `cell` blocks, each piece added
/// to its cell.
fn add_range(cells: &mut [Sums], cell: i32, from: i32, to: i32, add: impl Fn(&mut Sums, i32, i32)) {
    let mut z = from.max(0);
    while z <= to {
        let end = to.min((z / cell + 1) * cell - 1);
        add(&mut cells[(z / cell) as usize], z, end);
        z = end + 1;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// What's placed in the far cells (build 207): a block placed in the air over the spawn changes its cell column of
    /// 32 by exactly one solid block and its six faces (and the columns beside it not at all); a cell of pieces counts
    /// as its main material; a piece where ground is changes nothing; taken back, the cells are as they were.
    #[test]
    fn what_is_placed_is_summed() {
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (stone, sand, gabbro) = (doorway.block("incus:stone"), doorway.block("incus:sand"), doorway.block("incus:gabbro"));
        let summed = |doorway: &Doorway| cells(doorway, &colours, 32, [0, 0], 2, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
        let before = summed(&doorway);
        let ground = match doorway.column(5, 5).unwrap() {
            crate::doorway::Column::Here { ground, .. } => ground,
            crate::doorway::Column::Past => panic!("the spawn is on the map"),
        };
        let z = ground + 40;
        doorway.make(Change::Block { cell: [5, 5, z], block: stone }).unwrap();
        let after = summed(&doorway);
        let (one, two) = (before[0].at(z / 32, 32), after[0].at(z / 32, 32));
        assert_eq!(two.solid, one.solid + 1, "one solid block more in its cell");
        assert_eq!(two.showing(), one.showing() + 6, "its six faces showing");
        assert_eq!(after[1..], before[1..], "the cell columns beside it as they were");
        // Pieces: three octants of sand and two of gabbro in one cell: a cell of sand; in the air, six faces of sand.
        doorway.forget_changes();
        let cell = [9, 9, z];
        for (octant, block) in [([18, 18, z * 2], sand), ([19, 18, z * 2], sand), ([18, 19, z * 2], gabbro), ([19, 19, z * 2], gabbro), ([18, 18, z * 2 + 1], sand)] {
            doorway.make(Change::Oct { cell, from: octant, block }).unwrap();
        }
        let pieces = summed(&doorway);
        let mut whole = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        whole.block("incus:stone");
        whole.block("incus:sand");
        whole.block("incus:gabbro");
        whole.make(Change::Block { cell, block: sand }).unwrap();
        assert_eq!(pieces, summed(&whole), "a cell of pieces counts as its main material, whole");
        // A piece in the ground: nothing changes.
        doorway.forget_changes();
        doorway.make(Change::Oct { cell: [5, 5, ground - 3], from: [10, 10, (ground - 3) * 2], block: sand }).unwrap();
        assert_eq!(summed(&doorway), before, "a piece where ground is changes nothing");
        // Taken back: as before, exactly.
        doorway.forget_changes();
        assert_eq!(summed(&doorway), before, "taken back, the cells are as they were");
    }
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    /// One region of old's bake as its file keeps it (`old/src/sight/bake.rs`, `write_record`): its compact cell columns
    /// level by level (32 to 1,024, north fastest), then its 1,024's exact sums.
    struct OldRecord {
        levels: Vec<Vec<(i32, Vec<Cell>)>>,
        top: CellColumn,
    }

    /// Reads a file's bytes in order.
    struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
    }

    impl Reader<'_> {
        fn take<const N: usize>(&mut self) -> [u8; N] {
            let taken = self.bytes[self.at..self.at + N].try_into().unwrap();
            self.at += N;
            taken
        }
    }

    /// Old's bake read back (only this test opens it): its map's half width, and each region's last record (a later one
    /// for a region replaces an earlier, as old read it).
    fn old_bake(bytes: &[u8]) -> (i32, HashMap<[i32; 2], OldRecord>) {
        let mut r = Reader { bytes, at: 0 };
        assert_eq!(&r.take::<8>(), b"INCUSBK2", "old's bake is in the layout this reads");
        let _stamp = r.take::<8>();
        let half = i32::from_le_bytes(r.take());
        let mut records = HashMap::new();
        while r.at < bytes.len() {
            let region = [i32::from_le_bytes(r.take()), i32::from_le_bytes(r.take())];
            let mut levels = Vec::new();
            for _ in FIRST..=REGION {
                let n = u32::from_le_bytes(r.take()) as usize;
                let mut columns = Vec::with_capacity(n);
                for _ in 0..n {
                    let low = i16::from_le_bytes(r.take()) as i32;
                    let count = u16::from_le_bytes(r.take()) as usize;
                    let cells = (0..count)
                        .map(|_| {
                            let b: [u8; 16] = r.take();
                            Cell {
                                solid: b[0],
                                water: b[1],
                                faces: [b[2], b[3], b[4], b[5], b[6]],
                                light_spread: b[7],
                                colour: u32::from_le_bytes([b[8], b[9], b[10], b[11]]),
                                height_spread: u16::from_le_bytes([b[12], b[13]]),
                                showing: u16::from_le_bytes([b[14], b[15]]),
                            }
                        })
                        .collect();
                    columns.push((low, cells));
                }
                levels.push(columns);
            }
            let low = i32::from_le_bytes(r.take());
            let n = u32::from_le_bytes(r.take()) as usize;
            let sums = (0..n)
                .map(|_| {
                    let v: Vec<u64> = (0..15).map(|_| u64::from_le_bytes(r.take())).collect();
                    Sums { solid: v[0], water: v[1], faces: [v[2], v[3], v[4], v[5], v[6], v[7]], colour: [v[8], v[9], v[10]], light: v[11], light_squared: v[12], height: v[13], height_squared: v[14] }
                })
                .collect();
            records.insert(region, OldRecord { levels, top: CellColumn { low, sums, highest: 0 } });
        }
        (half, records)
    }

    /// A region against old's record: every cell at every level (the compact form), the cells left out of either's bands
    /// included, and the 1,024's exact sums. How many cells differ, and the first.
    fn compare(region: [i32; 2], mine: &[Vec<CellColumn>], old: &OldRecord) -> (usize, Option<String>) {
        let (mut differ, mut first) = (0, None);
        for (l, (columns, old_columns)) in mine.iter().zip(&old.levels).enumerate() {
            let size = 1 << (FIRST + l);
            if columns.len() != old_columns.len() {
                return (1, Some(format!("region {region:?}, cells of {size}: {} cell columns, old's {}", columns.len(), old_columns.len())));
            }
            for (c, (column, (old_low, old_cells))) in columns.iter().zip(old_columns).enumerate() {
                let old_band = CellColumn { low: *old_low, sums: Vec::new(), highest: 0 };
                let old_high = old_low + old_cells.len() as i32 - 1;
                for z in column.low.min(*old_low).max(0)..=column.high().max(old_high) {
                    let theirs = if (*old_low..=old_high).contains(&z) { old_cells[(z - old_low) as usize] } else { old_band.at(z, size).compact(size as u32) };
                    let ours = column.at(z, size).compact(size as u32);
                    if ours != theirs {
                        differ += 1;
                        first.get_or_insert_with(|| format!("region {region:?}, cells of {size}, column {c} (north {}, east {}), cell {z} up: ours {ours:?}, old's {theirs:?}", c % (1024 / size as usize), c / (1024 / size as usize)));
                    }
                }
            }
        }
        let (ours, theirs) = (&mine[REGION - FIRST][0], &old.top);
        for z in ours.low.min(theirs.low).max(0)..=ours.high().max(theirs.high()) {
            if ours.at(z, 1024) != theirs.at(z, 1024) {
                differ += 1;
                first.get_or_insert_with(|| format!("region {region:?}, the 1,024's exact sums, cell {z} up: ours {:?}, old's {:?}", ours.at(z, 1024), theirs.at(z, 1024)));
            }
        }
        (differ, first)
    }

    /// The pass mark (plan 2.2, step 1): every far cell, at every level, the same as old's bake byte for byte in its
    /// compact form, over the whole world, and each region's 1,024 its exact sums; the first sum's time measured. On all
    /// the processor's threads, the regions nearest the spawn first (square rings, as old's bake went).
    #[test]
    fn the_far_cells_are_old_s_bake() {
        let path = crate::session::project_folder().join("old").join("worlds").join("nice25 test").join("sight-bake.bin");
        assert!(path.is_file(), "old's bake isn't at {}", path.display());
        let (half, records) = old_bake(&std::fs::read(&path).unwrap());
        // The regions: old's extent of cells of 1,024 for a map of that half width (`Extent::of`).
        let size = 1 << REGION;
        let from = (-half).div_euclid(size);
        let across = (half - 1).div_euclid(size) - from + 1;
        let mut order: Vec<[i32; 2]> = (0..across * across).map(|k| [from + k % across, from + k / across]).collect();
        order.sort_by_key(|r| (r[0].abs().max(r[1].abs()), r[0], r[1]));
        assert_eq!(records.len(), order.len(), "old's bake holds a record for every region of its map");
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let started = Instant::now();
        let next = AtomicUsize::new(0);
        let done = Mutex::new(vec![0.0f64; order.len()]);
        let differ = AtomicUsize::new(0);
        let first = Mutex::new(Vec::new());
        let tops = Mutex::new(HashMap::new());
        let cells = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
                    let colours = FarColours::load(&mut doorway).unwrap();
                    loop {
                        let k = next.fetch_add(1, Ordering::Relaxed);
                        let Some(&r) = order.get(k) else { break };
                        let levels = region(&doorway, &colours, r).unwrap();
                        let (n, problem) = compare(r, &levels, &records[&r]);
                        differ.fetch_add(n, Ordering::Relaxed);
                        if let Some(problem) = problem {
                            first.lock().unwrap().push(problem);
                        }
                        cells.fetch_add(levels.iter().flatten().map(|column| column.sums.len()).sum(), Ordering::Relaxed);
                        tops.lock().unwrap().insert(r, levels[REGION - FIRST][0].clone());
                        done.lock().unwrap()[k] = started.elapsed().as_secs_f64();
                    }
                });
            }
        });
        let whole = started.elapsed().as_secs_f64();
        let done = done.into_inner().unwrap();
        let eighth = done[..order.len() / 8].iter().copied().fold(0.0, f64::max);
        // The levels above 1,024, each cell its four children added, up to four cells covering the map.
        let upper = Instant::now();
        let mut columns = tops.into_inner().unwrap();
        let mut level = REGION;
        loop {
            let size = 1 << level;
            let across = (half - 1).div_euclid(size) - (-half).div_euclid(size) + 1;
            if across <= 2 {
                break;
            }
            let empty = CellColumn::default();
            let mut parents = HashMap::new();
            let places: std::collections::HashSet<[i32; 2]> = columns.keys().map(|r| [r[0].div_euclid(2), r[1].div_euclid(2)]).collect();
            for p in places {
                let child = |a: i32, b: i32| columns.get(&[p[0] * 2 + a, p[1] * 2 + b]).unwrap_or(&empty);
                parents.insert(p, parent_of([child(0, 0), child(1, 0), child(0, 1), child(1, 1)], size));
            }
            columns = parents;
            level += 1;
        }
        let upper = upper.elapsed().as_secs_f64();
        let mut first = first.into_inner().unwrap();
        first.sort();
        eprintln!(
            "The far cells: {} regions on {threads} threads, {} cells kept from 32 to 1,024; the eighth round the spawn in {eighth:.1} s, the whole world in {whole:.1} s, the levels up to cells of {} in {upper:.2} s more; {} cells not old's{}",
            order.len(),
            cells.load(Ordering::Relaxed),
            1 << level,
            differ.load(Ordering::Relaxed),
            first.first().map_or(String::new(), |problem| format!(" (the first: {problem})"))
        );
        assert_eq!(differ.load(Ordering::Relaxed), 0, "{} regions differ from old's bake, the first: {:?}", first.len(), first.first());
    }

    /// The compact form follows old's rule (old's own cases, `old/src/sight/cell.rs`).
    /// The cells of 16 add up to the cells of 32: over whole regions (the spawn's, a mountain's, the sea's, one at the
    /// map's edge), every cell of 32 is exactly its eight cells of 16 added, the cells left out of the bands included.
    /// The ring's cells are summed by the same reading as the regions', a cell half the size.
    #[test]
    fn the_cells_of_16_add_up_to_the_cells_of_32() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (from, across) = regions_of(&doorway);
        let size = 1 << REGION;
        let mut cells_checked = 0;
        for r in [[0, 0], [-1, 0], [-2, -3], [from[0], from[1]], [from[0] + across - 1, from[1] + across / 2]] {
            let thirty_twos = &region(&doorway, &colours, r).unwrap()[0];
            let sixteens = cells(&doorway, &colours, 16, [r[0] * size, r[1] * size], 64, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            for (k, parent) in thirty_twos.iter().enumerate() {
                let (x, y) = (k % 32, k / 32);
                let child = |a: usize, b: usize| &sixteens[(y * 2 + b) * 64 + x * 2 + a];
                let made = parent_of([child(0, 0), child(1, 0), child(0, 1), child(1, 1)], 16);
                for z in 0..(doorway.height() / 32) {
                    assert_eq!(made.at(z, 32), parent.at(z, 32), "region {r:?}, the cell column {x} {y} of 32, cell {z} up");
                    cells_checked += 1;
                }
            }
        }
        eprintln!("The cells of 16: {cells_checked} cells of 32 are exactly their eight cells of 16 added");
    }

    /// And the cells of 8 add up to the cells of 16, the same way (the 8s' ring): over squares of the map at the spawn,
    /// on a mountain, by the sea and at the map's edge.
    #[test]
    fn the_cells_of_8_add_up_to_the_cells_of_16() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (from, _) = regions_of(&doorway);
        let size = 1 << REGION;
        let mut cells_checked = 0;
        for at in [[0, 0], [-640, 64], [-2048 - 512, -3072], [from[0] * size, from[1] * size]] {
            let sixteens = cells(&doorway, &colours, 16, at, 32, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            let eights = cells(&doorway, &colours, 8, at, 64, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            for (k, parent) in sixteens.iter().enumerate() {
                let (x, y) = (k % 32, k / 32);
                let child = |a: usize, b: usize| &eights[(y * 2 + b) * 64 + x * 2 + a];
                let made = parent_of([child(0, 0), child(1, 0), child(0, 1), child(1, 1)], 8);
                for z in 0..(doorway.height() / 16) {
                    assert_eq!(made.at(z, 16), parent.at(z, 16), "the square at {at:?}, the cell column {x} {y} of 16, cell {z} up");
                    cells_checked += 1;
                }
            }
        }
        eprintln!("The cells of 8: {cells_checked} cells of 16 are exactly their eight cells of 8 added");
    }

    /// Plain ground summed from its runs is plain ground summed its own quick way, to the number (the road a column
    /// the scatter stage changed goes, given the export's own columns): cells of 8, 16 and 32 at the spawn, on a
    /// mountain, by the sea and at the map's edge.
    #[test]
    fn plain_ground_summed_from_its_runs_is_the_same() {
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let (from, _) = regions_of(&doorway);
        let size = 1 << REGION;
        let mut columns_checked = 0;
        for at in [[0, 0], [-640, 64], [-2048 - 512, -3072], [from[0] * size, from[1] * size]] {
            for (cell, across) in [(8, 32), (16, 16), (32, 8)] {
                let quick = cells(&doorway, &colours, cell, at, across, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
                let by_runs = cells_by(&doorway, &colours, cell, at, across, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new), true).unwrap();
                assert_eq!(quick, by_runs, "the square at {at:?}, cells of {cell}");
                columns_checked += quick.len();
            }
        }
        eprintln!("Plain ground from its runs: {columns_checked} cell columns the same as the quick way's");
    }

    /// With the scatter stage on, the cells are what counting says: over squares that hold a tower (its ring stands
    /// out over air), the free-standing arch, the slab in the air, the castle's gate, a town's roofs, the hall cut
    /// into the hillside, a cave and a wood, the cells of 8 summed by the summer are the cells counted one block and
    /// one face at a time from the doorway's own answer for each column.
    #[test]
    fn the_stage_s_cells_are_counted_block_by_block() {
        use crate::doorway::Column;
        use crate::scatter_stage::Kind;
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let height = doorway.height();
        let sites = doorway.stage().unwrap().sites().to_vec();
        for site in &sites {
            eprintln!("The stage's places: {:?} at north {}, east {}, its floor {}", site.kind, site.at[0], site.at[1], site.floor);
        }
        let first = |wanted: fn(&Kind) -> bool| sites.iter().find(|site| wanted(&site.kind)).map(|site| site.at);
        let mut squares: Vec<(&str, [i32; 2])> = Vec::new();
        for (name, at) in [
            ("a tower", first(|kind| matches!(kind, Kind::Tower { .. }))),
            ("the arch", first(|kind| matches!(kind, Kind::Arch { .. }))),
            ("the slab", first(|kind| matches!(kind, Kind::Slab { .. }))),
            ("the castle's gate", first(|kind| matches!(kind, Kind::Castle { .. })).map(|at| [at[0] + 50, at[1]])),
            ("a town", first(|kind| matches!(kind, Kind::House { make: 2, .. }))),
            ("the hall", first(|kind| matches!(kind, Kind::Hall { .. })).map(|at| [at[0] - 20, at[1]])),
            ("a cave", first(|kind| matches!(kind, Kind::Cave { .. }))),
        ] {
            squares.push((name, at.unwrap_or_else(|| panic!("the stage has no {name}"))));
        }
        // A wood: the first place north of the spawn where a crown of leaves stands.
        let leaves = doorway.block("incus:leaves");
        let wood = (0..4000).map(|k| [k * 8, 300]).find(|at| matches!(doorway.column(at[0], at[1]), Ok(Column::Here { runs, .. }) if runs.iter().any(|run| run.block == leaves))).expect("a wood north of the spawn");
        squares.push(("a wood", wood));
        let (cell, across) = (8, 8usize);
        let side = cell as usize * across;
        let (mut blocks_counted, mut faces_counted) = (0u64, 0u64);
        for (name, middle) in squares {
            let at = [(middle[0] - side as i32 / 2).div_euclid(cell) * cell, (middle[1] - side as i32 / 2).div_euclid(cell) * cell];
            let summed = cells(&doorway, &colours, cell, at, across, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            // Every block of the square and one column round it: 0 air, 1 water, else its block's number + 2.
            let wide = side + 2;
            let mut blocks = vec![0u16; wide * wide * height as usize];
            let mut tiles: HashMap<[i32; 2], crate::doorway::Tile> = HashMap::new();
            for j in 0..wide {
                for i in 0..wide {
                    let (tile, [ti, tj]) = doorway.place(at[0] + i as i32 - 1, at[1] + j as i32 - 1);
                    let answer = tiles.entry(tile).or_insert_with(|| doorway.tile(tile[0], tile[1]).unwrap());
                    if let Column::Here { runs, .. } = answer.column(&doorway, ti, tj).unwrap() {
                        for run in runs {
                            for z in run.from..run.to.min(height) {
                                blocks[(i + j * wide) * height as usize + z as usize] = if run.block == WATER { 1 } else { run.block + 2 };
                            }
                        }
                    }
                }
            }
            let block = |i: usize, j: usize, z: i32| if z < 0 { 2 } else if z >= height { 0 } else { blocks[(i + j * wide) * height as usize + z as usize] };
            let mut looking_down = 0;
            for cy in 0..across {
                for cx in 0..across {
                    let mut counted = vec![Sums::default(); (height / cell) as usize];
                    for j in 0..cell as usize {
                        for i in 0..cell as usize {
                            let (i, j) = (cx * cell as usize + i + 1, cy * cell as usize + j + 1);
                            for z in 0..height {
                                let here = block(i, j, z);
                                let sums = &mut counted[(z / cell) as usize];
                                if here == 1 {
                                    sums.water += 1;
                                }
                                if here < 2 {
                                    continue;
                                }
                                sums.solid += 1;
                                blocks_counted += 1;
                                let beside = [block(i, j, z + 1), block(i, j, z - 1), block(i + 1, j, z), block(i - 1, j, z), block(i, j + 1, z), block(i, j - 1, z)];
                                for (d, other) in beside.into_iter().enumerate() {
                                    if other < 2 {
                                        sums.add_faces(d, z, z, colours.colour(here - 2, d == UP));
                                        faces_counted += 1;
                                    }
                                }
                            }
                        }
                    }
                    let column = &summed[cx + cy * across];
                    for z in 0..height / cell {
                        assert_eq!(column.at(z, cell), counted[z as usize], "{name}: the cell column {cx} {cy} of the square at {at:?}, cell {z} up");
                    }
                    looking_down += counted.iter().filter(|sums| sums.faces[1] > 0).count();
                }
            }
            eprintln!("The stage's cells, {name}: the square at {at:?} counted block by block ({looking_down} cells hold a face looking down)");
        }
        eprintln!("The stage's cells: {blocks_counted} blocks and {faces_counted} faces counted one at a time, every cell the summer's");
    }

    /// And with the scatter stage on, every level is still its children added: a 32 its eight 16s, a 16 its eight 8s,
    /// over the squares that hold the castle, the towns and the hall.
    #[test]
    fn the_stage_s_cells_add_up() {
        use crate::scatter_stage::Kind;
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let sites = doorway.stage().unwrap().sites().to_vec();
        let mut cells_checked = 0;
        for site in sites.iter().filter(|site| matches!(site.kind, Kind::Castle { .. } | Kind::Hall { .. } | Kind::House { make: 2, .. })) {
            let at = [(site.at[0] - 128).div_euclid(32) * 32, (site.at[1] - 128).div_euclid(32) * 32];
            let thirty_twos = cells(&doorway, &colours, 32, at, 8, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            let sixteens = cells(&doorway, &colours, 16, at, 16, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            let eights = cells(&doorway, &colours, 8, at, 32, |tx, ty| doorway.tile(tx, ty).map(std::rc::Rc::new)).unwrap();
            for (children, parents, child_size, across) in [(&sixteens, &thirty_twos, 16, 8usize), (&eights, &sixteens, 8, 16)] {
                for (k, parent) in parents.iter().enumerate() {
                    let (x, y) = (k % across, k / across);
                    let child = |a: usize, b: usize| &children[(y * 2 + b) * across * 2 + x * 2 + a];
                    let made = parent_of([child(0, 0), child(1, 0), child(0, 1), child(1, 1)], child_size);
                    for z in 0..(doorway.height() / (child_size * 2)) {
                        assert_eq!(made.at(z, child_size * 2), parent.at(z, child_size * 2), "{:?} at {:?}: the cell column {x} {y} of {}, cell {z} up", site.kind, site.at, child_size * 2);
                        cells_checked += 1;
                    }
                }
            }
        }
        eprintln!("The stage's cells add up: {cells_checked} cells are exactly their eight children added");
    }

    /// The height table's numbers are true: over whole regions (a mountain's, the spawn's, the sea's), each cell column
    /// of 32's `highest` is the top of the highest block in it, ground or water, read here straight from the doorway's
    /// columns one by one; each coarser column's is the highest of its four children's; and the numbers the card gets
    /// (`bricks::tops_round`) are never under anything in a column or in the eight round it.
    #[test]
    fn the_height_table_is_never_under_what_stands_there() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let size = 1 << REGION;
        let mut tiles: HashMap<[i32; 2], crate::doorway::Tile> = HashMap::new();
        let mut checked = 0;
        for r in [[-1, 0], [0, 0], [-2, -3]] {
            let levels = region(&doorway, &colours, r).unwrap();
            // Every block column's top, read one by one.
            let mut block_tops = vec![0i32; (size * size) as usize];
            for e in 0..size {
                for n in 0..size {
                    let (tile, within) = doorway.place(r[0] * size + n, r[1] * size + e);
                    let answer = tiles.entry(tile).or_insert_with(|| doorway.tile(tile[0], tile[1]).unwrap());
                    let (ground, water, _) = answer.raw(within[0], within[1]);
                    if ground >= 0 {
                        block_tops[(e * size + n) as usize] = (ground + 1).max(water.max(doorway.sea_level).min(doorway.height()));
                    }
                }
            }
            for (l, columns) in levels.iter().enumerate() {
                let cell = 32 << l;
                let across = size / cell;
                for (k, column) in columns.iter().enumerate() {
                    let (cx, cy) = (k as i32 % across, k as i32 / across);
                    let mut most = 0;
                    for e in cy * cell..(cy + 1) * cell {
                        for n in cx * cell..(cx + 1) * cell {
                            most = most.max(block_tops[(e * size + n) as usize]);
                        }
                    }
                    assert_eq!(column.highest, most, "region {r:?}, cells of {cell}, column {cx} {cy}: the highest block's top");
                    checked += 1;
                }
            }
            // The card's numbers for the 32s: over everything in the column and the eight round it.
            let tops: Vec<u16> = levels[0].iter().map(|c| c.highest as u16).collect();
            let round = crate::bricks::tops_round(&tops, 32);
            for cy in 0..32i32 {
                for cx in 0..32i32 {
                    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                        let (a, b) = (cx + dx, cy + dy);
                        if (0..32).contains(&a) && (0..32).contains(&b) {
                            assert!(round[(cy * 32 + cx) as usize] >= tops[(b * 32 + a) as usize] as u32, "region {r:?}, column {cx} {cy}: under its neighbour {a} {b}");
                        }
                    }
                }
            }
        }
        // A column not known makes its neighbours' numbers unknown too.
        let mut some = vec![5u16; 9];
        some[4] = u16::MAX;
        assert!(crate::bricks::tops_round(&some, 3).iter().all(|top| *top == u32::MAX), "a column not known must hold its neighbours back");
        eprintln!("The height table: {checked} cell columns' highest things are the doorway's own, block for block");
    }

    #[test]
    fn the_compact_form_follows_old_s_rule() {
        let sums = Sums { faces: [1, 1, 0, 0, 0, 0], colour: [200, 200, 200], height: 10, height_squared: 4 * 4 + 6 * 6, ..Sums::default() };
        let cell = sums.compact(8);
        assert_eq!((cell.faces[0], cell.faces[1]), (128, 127), "half up, half down: 127.5 rounds up, the rest the other");
        assert_eq!(cell.height_spread, 16, "heights 4 and 6: one block either side");
        let sums = Sums { faces: [3, 1, 1, 1, 0, 1], colour: [700, 700, 700], ..Sums::default() };
        assert_eq!(sums.compact(4).faces, [109, 37, 36, 37, 0], "3/7, then 1/7 each, the running totals rounded");
        assert_eq!(Sums::default().compact(4), Cell::default(), "an empty cell");
        assert_eq!(std::mem::size_of::<Cell>(), 16);
    }

    /// A face range's heights add as each face's would, one by one.
    #[test]
    fn a_range_of_faces_adds_as_its_faces_one_by_one() {
        let mut together = Sums::default();
        together.add_faces(3, 5, 40, [100, 200, 300]);
        let mut one_by_one = Sums::default();
        for z in 5..=40 {
            one_by_one.add_faces(3, z, z, [100, 200, 300]);
        }
        assert_eq!(together, one_by_one);
    }
}

//! The far eye's ring (plan 2.2, the cells of 2 to 16; first of them, the 16s): cells too fine to lay over the whole
//! map, kept round the eye and moved with it.
//!
//! Old kept its live levels 128 cells a side round the camera (`old/src/sight/store.rs`); the ring is that, carried as
//! a result: 128 × 128 cell columns of 16, so the 16s reach about 1,024 blocks from the eye. Past its edge the march
//! reads the 32s as before, and over its outermost cells the two are blended, so the ring's edge is no seam
//! (`far.wgsl`, `spot_kept` and the march's hand-over).
//!
//! **Made off the frame.** A thread of its own (the ring's maker) sums the ring's cells from the doorway, a square of
//! columns at a time, kept while the ring is over them, and makes each new brick's cells and textures. The frame only
//! sends what it's handed, a little each frame, to places nothing reads yet; and when every brick of the new ring is on
//! the card it moves the ring over all at once (its level's least column, its bands, its bricks' places). So there's
//! never a hole, and the frame makes nothing [David, 2026-11-14: "hopefully all this work, being much shorter than the
//! longest rays, goes on pretty much alongside everything else and doesn't cost much in time"].

use crate::atlas::{Atlas, Bench, Brick, Kept, NO_SLOT};
use crate::bricks::{Level, NO_BRICK, RING, RING_LEVEL, Store, row};
use crate::doorway::{Doorway, Tile};
use crate::far::{Cell, FarColours};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Instant;

/// The ring moves this many cell columns at a time (a brick's side at bricks of 8: 128 blocks at the 16s), and its
/// maker sums squares of them.
const STEP: i32 = 8;
/// Cell columns the maker keeps round the ring, for its bricks' textures: a brick's border and its borrowing reach
/// past the brick (`atlas.rs`).
const MARGIN: i32 = 8;
/// The ring's level fades into the 32s over this many of its outermost cells.
pub const FADE: i32 = 8;

/// A brick the maker made: where it is in the world (in bricks of its level: north, east, up), its cells for the pool
/// and its four textures' bytes.
pub struct Made {
    at: [i32; 3],
    cells: Vec<Cell>,
    textures: [Vec<u8>; 4],
}

/// A ring the maker has ready: its least column, its bands (north fastest from there), every brick it holds, those of
/// them not handed over with the ring before, and how long it took to make.
pub struct Ready {
    from: [i32; 2],
    entries: Vec<u32>,
    holds: Vec<[i32; 3]>,
    new: Vec<Made>,
    seconds: f32,
}

/// A brick on the card: where its cells start in the pool, its slot in the atlas (`NO_SLOT` till the atlas is in), and
/// its textures while they wait for the atlas.
struct Held {
    start: u64,
    slot: u32,
    textures: Option<[Vec<u8>; 4]>,
}

/// The ring as the frame keeps it.
pub struct Ring {
    asks: mpsc::Sender<[i32; 2]>,
    readies: mpsc::Receiver<Result<Ready, String>>,
    /// The ring last asked of the maker (its least column).
    asked: Option<[i32; 2]>,
    /// The ring handed over and not yet moved to; its new bricks still to send; and every brick sent.
    coming: Option<Ready>,
    waiting: HashMap<[i32; 3], Made>,
    held: HashMap<[i32; 3], Held>,
    /// The ring on the card: its least column (none yet), and its bricks.
    pub from: Option<[i32; 2]>,
    on_card: Vec<[i32; 3]>,
    /// Bricks whose textures wait for the atlas, and whether the atlas's slots for the ring on the card want writing
    /// again (a brick of it got its textures since).
    unsent: usize,
    stale: bool,
    /// Times the ring has moved, and how long the maker took over the last one (seconds).
    pub moves: u32,
    pub made_in: f32,
    /// Whether the maker has failed (it says why in the log): nothing waits on a ring that won't come.
    pub failed: bool,
}

impl Ring {
    /// Starts the ring's maker for the world in `folder`: `level` the store's ring level (its bricks' shape), `side` a
    /// brick's texels a side in the atlas.
    pub fn start(folder: PathBuf, level: Level, side: u32) -> Result<Self, String> {
        let (asks, asked) = mpsc::channel();
        let (made, readies) = mpsc::channel();
        std::thread::Builder::new().name("making the far ring".into()).spawn(move || maker(folder, level, side, asked, made)).map_err(|error| format!("a thread for the far ring: {error}"))?;
        Ok(Self { asks, readies, asked: None, coming: None, waiting: HashMap::new(), held: HashMap::new(), from: None, on_card: Vec::new(), unsent: 0, stale: false, moves: 0, made_in: 0.0, failed: false })
    }

    /// The ring's least column for an eye at `eye` (north, east): the eye's column in the middle, to a step.
    fn wanted(level: &Level, eye: [f32; 2]) -> [i32; 2] {
        let cell = (1 << level.level) as f32;
        let snap = |e: f32| ((e / cell).floor() as i32 - RING / 2).div_euclid(STEP) * STEP;
        [snap(eye[0]), snap(eye[1])]
    }

    /// Whether the ring round `eye` is on the card whole, its textures too if the atlas is in (for a picture).
    #[cfg(test)]
    pub fn settled(&self, store: &Store, eye: [f32; 2], atlas_in: bool) -> bool {
        self.from == Some(Self::wanted(&store.levels[row(RING_LEVEL)], eye)) && self.coming.is_none() && (!atlas_in || self.unsent == 0)
    }

    /// The ring brought to the eye, for at most `ms` of the frame: the maker asked for the ring round `eye` if that
    /// isn't the one last asked for; what it has handed over sent to the card; and the ring moved over once its
    /// bricks are all there.
    pub fn follow(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, store: &mut Store, mut atlas: Option<&mut Atlas>, eye: [f32; 2], ms: f32) {
        let started = Instant::now();
        let in_time = || started.elapsed().as_secs_f32() * 1000.0 < ms;
        let level = store.levels[row(RING_LEVEL)];
        let wanted = Self::wanted(&level, eye);
        if self.asked != Some(wanted) && self.asks.send(wanted).is_ok() {
            self.asked = Some(wanted);
        }
        // What the maker has handed over: the latest ring counts, and only its bricks are kept waiting.
        while let Ok(ready) = self.readies.try_recv() {
            match ready {
                Ok(mut ready) => {
                    for made in ready.new.drain(..) {
                        self.waiting.insert(made.at, made);
                    }
                    let holds: HashSet<[i32; 3]> = ready.holds.iter().copied().collect();
                    self.waiting.retain(|at, _| holds.contains(at));
                    self.made_in = ready.seconds;
                    self.coming = Some(ready);
                }
                Err(error) => {
                    if !self.failed {
                        log::error!("The far ring couldn't be made: {error}");
                    }
                    self.failed = true;
                }
            }
        }
        // The bricks waiting, sent a little a frame: cells into the pool, textures into the atlas if it's in (kept
        // here till it is).
        while in_time() {
            let Some(&at) = self.waiting.keys().next() else { break };
            let Some(made) = self.waiting.remove(&at) else { break };
            if self.held.contains_key(&at) {
                continue;
            }
            let start = store.ring_cells(device, queue, &made.cells);
            let held = match atlas.as_deref_mut() {
                Some(atlas) => Held { start, slot: atlas.prepare(device, queue, &made.textures), textures: None },
                None => {
                    self.unsent += 1;
                    Held { start, slot: NO_SLOT, textures: Some(made.textures) }
                }
            };
            self.held.insert(at, held);
        }
        // The textures made before the atlas was in.
        if self.unsent > 0
            && let Some(atlas) = atlas.as_deref_mut()
        {
            for held in self.held.values_mut() {
                if !in_time() {
                    break;
                }
                if let Some(textures) = held.textures.take() {
                    held.slot = atlas.prepare(device, queue, &textures);
                    self.unsent -= 1;
                    self.stale = true;
                }
            }
        }
        // The ring moved over, once every brick of the new one is on the card: its level's least column, its bands
        // and its bricks' places written together, then the bricks it left given back.
        let places = (level.bricks * level.bricks * level.bricks_up) as usize;
        let place_of = |from: [i32; 2], at: &[i32; 3]| (((at[1] - from[1].div_euclid(level.wide)) * level.bricks + (at[0] - from[0].div_euclid(level.wide))) * level.bricks_up + at[2]) as usize;
        if self.coming.as_ref().is_some_and(|ready| ready.holds.iter().all(|at| self.held.contains_key(at)))
            && let Some(ready) = self.coming.take()
        {
            let mut index = vec![NO_BRICK; places];
            let mut slots = vec![NO_SLOT; places];
            for at in &ready.holds {
                let held = &self.held[at];
                let place = place_of(ready.from, at);
                index[place] = held.start as u32;
                slots[place] = held.slot;
            }
            store.ring_switch(queue, ready.from, &ready.entries, &index);
            if let Some(atlas) = atlas.as_deref_mut() {
                atlas.place(queue, level.index_at, &slots);
            }
            let keep: HashSet<[i32; 3]> = ready.holds.iter().copied().collect();
            let gone: Vec<[i32; 3]> = self.held.keys().filter(|at| !keep.contains(*at)).copied().collect();
            for at in gone {
                let Some(held) = self.held.remove(&at) else { continue };
                store.ring_release(held.start);
                if held.textures.is_some() {
                    self.unsent -= 1;
                } else if held.slot != NO_SLOT
                    && let Some(atlas) = atlas.as_deref_mut()
                {
                    atlas.release(held.slot);
                }
            }
            if self.from.is_none() {
                log::info!("The far ring: the 16s round the eye on the card, {} bricks, made in {:.2} s", ready.holds.len(), ready.seconds);
            }
            self.from = Some(ready.from);
            self.on_card = ready.holds;
            self.moves += 1;
            self.stale = false;
        } else if self.stale
            && let (Some(from), Some(atlas)) = (self.from, atlas.as_deref_mut())
        {
            // A brick of the ring on the card got its textures: the atlas's slots for the ring written again.
            let mut slots = vec![NO_SLOT; places];
            for at in &self.on_card {
                if let Some(held) = self.held.get(at) {
                    slots[place_of(from, at)] = held.slot;
                }
            }
            atlas.place(queue, level.index_at, &slots);
            self.stale = false;
        }
    }

    /// The ring's box on the card, in blocks: least and most, north and east (none yet: an empty box), a cell in from
    /// its edge all round (a blend reads the cell past its own).
    pub fn reach(&self, level: &Level) -> [f32; 4] {
        let cell = (1 << level.level) as f32;
        match self.from {
            Some(from) => [(from[0] + 1) as f32 * cell, (from[1] + 1) as f32 * cell, (from[0] + RING - 1) as f32 * cell, (from[1] + RING - 1) as f32 * cell],
            None => [1.0, 1.0, 0.0, 0.0],
        }
    }

    /// The ring in words, for a report.
    pub fn state(&self) -> String {
        match self.from {
            Some(_) => format!("the 16s in a ring of {RING} round the eye ({} bricks, moved {} times, the last made in {:.3} s)", self.on_card.len(), self.moves, self.made_in),
            None => "the 16s' ring not on the card yet".to_string(),
        }
    }
}

/// The ring's maker, on a thread of its own: each ring asked for (the latest, if several wait) made and handed over.
fn maker(folder: PathBuf, level: Level, side: u32, asks: mpsc::Receiver<[i32; 2]>, readies: mpsc::Sender<Result<Ready, String>>) {
    let opened = Doorway::open(&folder).and_then(|mut doorway| FarColours::load(&mut doorway).map(|colours| (doorway, colours)));
    let (doorway, colours) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            let _ = readies.send(Err(error));
            return;
        }
    };
    let mut making = Making { doorway, colours, level, side, squares: HashMap::new(), read: HashMap::new(), handed: HashSet::new(), bench: Bench::default() };
    while let Ok(mut from) = asks.recv() {
        while let Ok(later) = asks.try_recv() {
            from = later;
        }
        let ready = making.ring(from);
        if let Ok(ready) = &ready {
            making.handed = ready.holds.iter().copied().collect();
        }
        if readies.send(ready).is_err() {
            break;
        }
    }
}

/// A cell column as the maker keeps it: its band's entry (`bricks::entry_of`) and its band's cells, compact.
type Column = (u32, Vec<Cell>);

/// What the maker keeps between rings.
struct Making {
    doorway: Doorway,
    colours: FarColours,
    level: Level,
    side: u32,
    /// The squares of columns summed (`STEP` a side, by their place in squares; north fastest in each), kept while the
    /// ring is over or near them.
    squares: HashMap<[i32; 2], Vec<Column>>,
    /// The doorway's tiles read last (the squares of a ring ask for the same few).
    read: HashMap<[i32; 2], Rc<Tile>>,
    /// The bricks of the ring handed over last: the frame has those.
    handed: HashSet<[i32; 3]>,
    bench: Bench,
}

impl Making {
    /// The ring whose least column is `from`, made: the squares it and its margin lie over summed (those not kept
    /// already), its bands, and the cells and textures of each of its bricks the frame doesn't have.
    fn ring(&mut self, from: [i32; 2]) -> Result<Ready, String> {
        let started = Instant::now();
        let level = self.level;
        let cell = 1i32 << level.level;
        let (lo, hi) = ([from[0] - MARGIN, from[1] - MARGIN], [from[0] + RING + MARGIN - 1, from[1] + RING + MARGIN - 1]);
        let (s0, s1) = ([lo[0].div_euclid(STEP), lo[1].div_euclid(STEP)], [hi[0].div_euclid(STEP), hi[1].div_euclid(STEP)]);
        self.squares.retain(|s, _| s[0] >= s0[0] - 2 && s[0] <= s1[0] + 2 && s[1] >= s0[1] - 2 && s[1] <= s1[1] + 2);
        for sy in s0[1]..=s1[1] {
            for sx in s0[0]..=s1[0] {
                if self.squares.contains_key(&[sx, sy]) {
                    continue;
                }
                if self.read.len() > 16 {
                    self.read.clear();
                }
                let (doorway, read) = (&self.doorway, &mut self.read);
                let columns = crate::far::cells(doorway, &self.colours, cell, [sx * STEP * cell, sy * STEP * cell], STEP as usize, |tx, ty| {
                    if let Some(tile) = read.get(&[tx, ty]) {
                        return Ok(tile.clone());
                    }
                    let tile = Rc::new(doorway.tile(tx, ty)?);
                    read.insert([tx, ty], tile.clone());
                    Ok(tile)
                })?;
                self.squares.insert([sx, sy], columns.iter().map(|c| (crate::bricks::entry_of(c), c.sums.iter().map(|s| s.compact(cell as u32)).collect())).collect());
            }
        }
        let squares = &self.squares;
        let column = |cx: i32, cy: i32| -> &Column { &squares[&[cx.div_euclid(STEP), cy.div_euclid(STEP)]][(cy.rem_euclid(STEP) * STEP + cx.rem_euclid(STEP)) as usize] };
        let entries: Vec<u32> = (0..RING * RING).map(|k| column(from[0] + k % RING, from[1] + k / RING).0).collect();
        // The ring's bricks: one wherever some column of it has band in its heights (as the store's own are made).
        let band = |c: &Column| {
            let low = (c.0 & 0xFFFF) as u16 as i16 as i32;
            (low, low + (c.0 >> 16) as i32)
        };
        let first = [from[0].div_euclid(level.wide), from[1].div_euclid(level.wide)];
        let mut holds = Vec::new();
        let mut wanted = Vec::new();
        for by in 0..level.bricks {
            for bx in 0..level.bricks {
                for bz in 0..level.bricks_up {
                    let (z0, z1) = (bz * level.tall, ((bz + 1) * level.tall).min(level.up));
                    let (cx0, cy0) = ((first[0] + bx) * level.wide, (first[1] + by) * level.wide);
                    let any = (0..level.wide * level.wide).any(|k| {
                        let (low, top) = band(column(cx0 + k % level.wide, cy0 + k / level.wide));
                        low.max(z0) < top.min(z1)
                    });
                    if !any {
                        continue;
                    }
                    let at = [first[0] + bx, first[1] + by, bz];
                    holds.push(at);
                    if !self.handed.contains(&at) {
                        wanted.push(at);
                    }
                }
            }
        }
        // The new bricks' cells and textures. For the textures, the cells kept as the atlas reads them: the ring's
        // level laid over the ring and its margin.
        let mut new = Vec::with_capacity(wanted.len());
        if !wanted.is_empty() {
            let across = RING + 2 * MARGIN;
            let laid = Level { from: lo, across, bricks: across / level.wide, entries_at: 0, index_at: 0, ..level };
            let kept = Kept { levels: vec![laid], columns: vec![(0..across * across).map(|k| column(lo[0] + k % across, lo[1] + k / across).clone()).collect()] };
            for at in wanted {
                let (z0, z1) = (at[2] * level.tall, ((at[2] + 1) * level.tall).min(level.up));
                let mut cells = vec![Cell::default(); level.brick_cells() as usize];
                for j in 0..level.wide {
                    for i in 0..level.wide {
                        let c = column(at[0] * level.wide + i, at[1] * level.wide + j);
                        let (low, top) = band(c);
                        for cz in low.max(z0)..top.min(z1) {
                            cells[(i + j * level.wide + (cz - z0) * level.wide * level.wide) as usize] = c.1[(cz - low) as usize];
                        }
                    }
                }
                let brick = Brick { level: level.level, place: 0, origin: [at[0] * level.wide, at[1] * level.wide, at[2] * level.tall] };
                let textures = crate::atlas::brick_bytes(&kept, &brick, self.side, &mut self.bench);
                new.push(Made { at, cells, textures });
            }
        }
        Ok(Ready { from, entries, holds, new, seconds: started.elapsed().as_secs_f32() })
    }
}

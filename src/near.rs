//! The near eye's blocks and their faces (plan 2.3): the world close to the eye as triangles, by David's word of
//! 2026-11-16 ("I believe we should try to accomplish the rest of the graphics engine with triangles"; "lets just
//! make a radius slider and get started. the render radius set there will determine where the triangles start and
//! rays fade"). A chunk is 32 x 32 columns, the world's whole height (old's chunk: his "64 chunk render radius" is
//! 2,048 blocks). Its blocks are the doorway's answers; its faces are every side of a solid block that looks into
//! air or water, merged where they can be (greedy meshing), and the water's top.
//!
//! This file, so far: a chunk's blocks and the mesher, checked block by block. The lane that keeps the chunks round
//! the eye made, and the pass that draws them, come with the rest of build 195 (its pass mark:
//! `journal/docs/experiments.md`). Pieces (bars, quarter blocks) aren't meshed yet: a later step of 2.3.

// (Nothing outside its tests uses it yet: the lane and the pass are the rest of build 195.)
#![cfg_attr(not(test), allow(dead_code))]

use crate::doorway::{Block, Change, Column, Doorway, Tile, WATER};
use std::collections::HashMap;

/// Columns a side of a chunk.
pub const CHUNK: i32 = 32;

/// The way a face looks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Facing {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Facing {
    pub const ALL: [Facing; 6] = [Facing::North, Facing::South, Facing::East, Facing::West, Facing::Up, Facing::Down];

    /// The step from a cell to the one its face of this facing looks into (north, east, up).
    pub fn step(self) -> [i32; 3] {
        match self {
            Facing::North => [1, 0, 0],
            Facing::South => [-1, 0, 0],
            Facing::East => [0, 1, 0],
            Facing::West => [0, -1, 0],
            Facing::Up => [0, 0, 1],
            Facing::Down => [0, 0, -1],
        }
    }

    /// The two axes a face of this facing spreads along (0 north, 1 east, 2 up), and the axis it looks along: a face
    /// that looks up or down spreads north and east; one that looks north or south, east and up; east or west, north
    /// and up.
    pub fn axes(self) -> ([usize; 2], usize) {
        match self {
            Facing::North | Facing::South => ([1, 2], 0),
            Facing::East | Facing::West => ([0, 2], 1),
            Facing::Up | Facing::Down => ([0, 1], 2),
        }
    }
}

/// A face: a rectangle of one block's sides that look one way, all in one plane. `at` is its least cell in the chunk
/// (north and east from the chunk's least corner, 0 to 31; up from bedrock); it's `wide` cells along the first of
/// its facing's axes and `high` along the second. `water`: the water's top, not a block's side.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Face {
    pub at: [i32; 3],
    pub facing: Facing,
    pub wide: i32,
    pub high: i32,
    pub block: Block,
    pub water: bool,
}

/// A chunk's blocks with a column of its neighbours all round (so a side at the chunk's edge knows what it looks
/// into), from `low` up to `high`: under `low` every one of these columns is solid, over `high` all are air, and
/// neither has a face. A cell is 0 for air, or its block's number and one.
pub struct Blocks {
    /// The chunk (its place in chunks, north and east: the chunk of column `c` is `c.div_euclid(32)`).
    pub chunk: [i32; 2],
    pub low: i32,
    pub high: i32,
    cells: Vec<u16>,
}

const WIDE: usize = CHUNK as usize + 2;

impl Blocks {
    /// A chunk's blocks, read through the doorway (the tiles it needs are read once each into `tiles` and kept
    /// there for the next chunk), with the whole blocks placed since (`Change::Block`; bars aren't the near eye's
    /// yet).
    pub fn read(doorway: &Doorway, tiles: &mut HashMap<[i32; 2], Tile>, chunk: [i32; 2]) -> Result<Self, String> {
        let least = [chunk[0] * CHUNK - 1, chunk[1] * CHUNK - 1];
        let height = doorway.height();
        let mut columns: Vec<Column> = Vec::with_capacity(WIDE * WIDE);
        // (East slowest, north fastest: a column's place here is north + east x 34.)
        for east in 0..WIDE as i32 {
            for north in 0..WIDE as i32 {
                let (tile, [i, j]) = doorway.place(least[0] + north, least[1] + east);
                let inside = tile[0] >= 0 && tile[1] >= 0 && tile[0] * doorway.grid().1 < doorway.across() && tile[1] * doorway.grid().1 < doorway.across();
                if !inside {
                    columns.push(Column::Past);
                    continue;
                }
                if !tiles.contains_key(&tile) {
                    tiles.insert(tile, doorway.tile(tile[0], tile[1])?);
                }
                columns.push(tiles[&tile].column(doorway, i, j)?);
            }
        }
        // The whole blocks placed in these columns.
        let mut placed: Vec<([i32; 3], Block)> = Vec::new();
        let mut asked: Vec<[i32; 2]> = Vec::new();
        for corner in [[0, 0], [WIDE as i32 - 1, 0], [0, WIDE as i32 - 1], [WIDE as i32 - 1, WIDE as i32 - 1]] {
            let (tile, _) = doorway.place(least[0] + corner[0], least[1] + corner[1]);
            if !asked.contains(&tile) {
                asked.push(tile);
                for change in doorway.changes_in(tile[0], tile[1]) {
                    if let Change::Block { cell, block } = *change {
                        let local = [cell[0] - least[0], cell[1] - least[1]];
                        if local[0] >= 0 && local[1] >= 0 && local[0] < WIDE as i32 && local[1] < WIDE as i32 && cell[2] >= 0 && cell[2] < height {
                            placed.push(([local[0], local[1], cell[2]], block));
                        }
                    }
                }
            }
        }
        // From the lowest cell that isn't solid in any of the columns (one under it, so its floor's face is in) to the
        // highest that is anything at all.
        let mut low = height;
        let mut high = 0;
        for column in &columns {
            match column {
                Column::Past => low = 0,
                Column::Here { runs, .. } => {
                    let mut open = 0;
                    for run in runs {
                        if run.from > open || run.block == WATER {
                            break;
                        }
                        open = run.to;
                    }
                    low = low.min(open);
                    high = high.max(runs.last().map_or(0, |run| run.to));
                }
            }
        }
        for (cell, _) in &placed {
            low = low.min(cell[2]);
            high = high.max(cell[2] + 1);
        }
        let low = (low - 1).max(0);
        let high = (high + 1).min(height).max(low + 1);
        let tall = (high - low) as usize;
        let mut cells = vec![0u16; WIDE * WIDE * tall];
        for (k, column) in columns.iter().enumerate() {
            if let Column::Here { runs, .. } = column {
                for run in runs {
                    for z in run.from.max(low)..run.to.min(high) {
                        cells[k * tall + (z - low) as usize] = run.block + 1;
                    }
                }
            }
        }
        for (cell, block) in placed {
            cells[(cell[0] as usize + cell[1] as usize * WIDE) * tall + (cell[2] - low) as usize] = block + 1;
        }
        Ok(Self { chunk, low, high, cells })
    }

    /// The cell at `north`, `east` from the chunk's least corner (-1 to 32) and `up` from bedrock: 0 air, or its
    /// block's number and one. Under what's kept, solid stone; over it, air.
    pub fn cell(&self, north: i32, east: i32, up: i32) -> u16 {
        if up < self.low {
            return crate::doorway::STONE + 1;
        }
        if up >= self.high {
            return 0;
        }
        let tall = (self.high - self.low) as usize;
        self.cells[((north + 1) as usize + (east + 1) as usize * WIDE) * tall + (up - self.low) as usize]
    }

    /// Whether a cell stops the eye: a block, not air and not water.
    fn solid(cell: u16) -> bool {
        cell != 0 && cell != WATER + 1
    }

    /// The chunk's faces, merged: for each facing and each plane across it, the open sides of one block that make a
    /// rectangle become one face (the rectangle grown along its first axis, then its second: greedy meshing). A
    /// block's side is open where the cell it looks into isn't solid (air or water); the water's top is where water
    /// has air over it.
    pub fn faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();
        let size = [CHUNK, CHUNK, self.high - self.low];
        for facing in Facing::ALL {
            let ([a, b], along) = facing.axes();
            let step = facing.step();
            let (wide, high) = (size[a] as usize, size[b] as usize);
            // What shows at each place of the plane: 0 nothing; a block's number and one; water's top marked by the
            // highest bit.
            let mut mask = vec![0u32; wide * high];
            for plane in 0..size[along] {
                let mut any = false;
                for v in 0..high {
                    for u in 0..wide {
                        let mut at = [0i32; 3];
                        at[a] = u as i32;
                        at[b] = v as i32;
                        at[along] = plane;
                        let (north, east, up) = (at[0], at[1], at[2] + self.low);
                        let cell = self.cell(north, east, up);
                        let beyond = self.cell(north + step[0], east + step[1], up + step[2]);
                        // (The world's floor shows no underside.)
                        let shows = if Self::solid(cell) {
                            if !Self::solid(beyond) && !(facing == Facing::Down && up == 0) { cell as u32 } else { 0 }
                        } else if cell == WATER + 1 && facing == Facing::Up && beyond == 0 {
                            1 << 31 | cell as u32
                        } else {
                            0
                        };
                        mask[u + v * wide] = shows;
                        any |= shows != 0;
                    }
                }
                if !any {
                    continue;
                }
                for v in 0..high {
                    let mut u = 0;
                    while u < wide {
                        let shows = mask[u + v * wide];
                        if shows == 0 {
                            u += 1;
                            continue;
                        }
                        let mut across = 1;
                        while u + across < wide && mask[u + across + v * wide] == shows {
                            across += 1;
                        }
                        let mut down = 1;
                        while v + down < high && (0..across).all(|k| mask[u + k + (v + down) * wide] == shows) {
                            down += 1;
                        }
                        for dv in 0..down {
                            for du in 0..across {
                                mask[u + du + (v + dv) * wide] = 0;
                            }
                        }
                        let mut at = [0i32; 3];
                        at[a] = u as i32;
                        at[b] = v as i32;
                        at[along] = plane;
                        at[2] += self.low;
                        faces.push(Face { at, facing, wide: across as i32, high: down as i32, block: (shows & 0xFFFF) as Block - 1, water: shows >> 31 == 1 });
                        u += across;
                    }
                }
            }
        }
        faces
    }
}

/// The chunks whose faces a change in `cell` (north, east, up) can alter: its own, and the one across each edge of
/// its chunk it lies on (a face depends only on the cell it's on and the one it looks into).
pub fn touched(cell: [i32; 3]) -> Vec<[i32; 2]> {
    let chunk = [cell[0].div_euclid(CHUNK), cell[1].div_euclid(CHUNK)];
    let local = [cell[0].rem_euclid(CHUNK), cell[1].rem_euclid(CHUNK)];
    let mut chunks = vec![chunk];
    for axis in 0..2 {
        if local[axis] == 0 {
            let mut other = chunk;
            other[axis] -= 1;
            chunks.push(other);
        }
        if local[axis] == CHUNK - 1 {
            let mut other = chunk;
            other[axis] += 1;
            chunks.push(other);
        }
    }
    chunks
}

/// A face as the card is given it, three words: its place in its chunk, its facing and its width; its height and
/// its block; its chunk. (North and east 0 to 31 in 6 bits each, up in 10, the facing in 3, the width less one in 5,
/// water's top marked; the height less one in 10, the block in 16; the chunk's north and east, each with 32,768
/// added, in 16 bits each.)
pub fn packed(face: &Face, chunk: [i32; 2]) -> [u32; 3] {
    let facing = Facing::ALL.iter().position(|f| *f == face.facing).unwrap() as u32;
    let a = face.at[0] as u32 | (face.at[1] as u32) << 6 | (face.at[2] as u32) << 12 | facing << 22 | ((face.wide - 1) as u32) << 25 | (face.water as u32) << 30;
    let b = (face.high - 1) as u32 | (face.block as u32) << 10;
    let c = (chunk[0] + 32768) as u32 | ((chunk[1] + 32768) as u32) << 16;
    [a, b, c]
}

/// A chunk as the lane hands it over: its packed faces, the lowest and highest block any of them touches (for
/// leaving it out when it's out of view), and how long it took to make (ms).
pub struct Made {
    pub chunk: [i32; 2],
    pub faces: Vec<[u32; 3]>,
    pub low: i32,
    pub high: i32,
    pub ms: f32,
}

/// The chunks within `radius` chunks of `eye` (north, east, blocks): every chunk any part of which is that near, so
/// the whole circle is covered. Nearest first.
pub fn wanted(eye: [f32; 2], radius: f32) -> Vec<[i32; 2]> {
    let reach = radius * CHUNK as f32;
    let middle = [(eye[0] / CHUNK as f32).floor() as i32, (eye[1] / CHUNK as f32).floor() as i32];
    let span = radius.ceil() as i32 + 1;
    let mut chunks: Vec<(f32, [i32; 2])> = Vec::new();
    for north in middle[0] - span..=middle[0] + span {
        for east in middle[1] - span..=middle[1] + span {
            let off = nearest_part([north, east], eye);
            if off <= reach {
                chunks.push((off, [north, east]));
            }
        }
    }
    chunks.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    chunks.into_iter().map(|(_, chunk)| chunk).collect()
}

/// How far (blocks, level) the nearest part of a chunk is from `eye`.
pub fn nearest_part(chunk: [i32; 2], eye: [f32; 2]) -> f32 {
    let side = CHUNK as f32;
    let d: Vec<f32> = (0..2).map(|k| (chunk[k] as f32 * side - eye[k]).max(eye[k] - (chunk[k] + 1) as f32 * side).max(0.0)).collect();
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// What the lane's threads share: the chunks asked for, nearest first, and whether to stop.
struct Asked {
    queue: std::collections::VecDeque<[i32; 2]>,
    stop: bool,
}

/// The near lane: threads of its own that make chunks' faces as they're asked for, each through a doorway of its
/// own, handing each over as it's done. The asking is the frame's: `ask` replaces what's waiting (so a change of
/// place or radius is heard at the next chunk), and what a thread already has in hand is finished and handed over.
pub struct Lane {
    asked: std::sync::Arc<(std::sync::Mutex<Asked>, std::sync::Condvar)>,
    made: std::sync::mpsc::Receiver<Result<Made, String>>,
    pub threads: usize,
}

impl Lane {
    /// Starts the lane on the map behind the doorway in `folder` (with the scatter stage if `staged`), on `threads`.
    pub fn start(folder: std::path::PathBuf, staged: bool, threads: usize) -> Result<Self, String> {
        // (Opened once here so a world that can't be opened is said at once, not by a thread later.)
        Doorway::open_with(&folder, staged)?;
        let asked = std::sync::Arc::new((std::sync::Mutex::new(Asked { queue: Default::default(), stop: false }), std::sync::Condvar::new()));
        let (sender, made) = std::sync::mpsc::channel();
        for k in 0..threads.max(1) {
            let (asked, sender, folder) = (asked.clone(), sender.clone(), folder.clone());
            std::thread::Builder::new()
                .name(format!("the near lane {k}"))
                .spawn(move || {
                    let doorway = match Doorway::open_with(&folder, staged) {
                        Ok(doorway) => doorway,
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            return;
                        }
                    };
                    // A few tiles kept (a chunk's columns come from one tile, or two or four at a tile's edge).
                    let mut tiles: HashMap<[i32; 2], Tile> = HashMap::new();
                    loop {
                        let chunk = {
                            let (lock, heard) = &*asked;
                            let mut asked = lock.lock().unwrap();
                            loop {
                                if asked.stop {
                                    return;
                                }
                                if let Some(chunk) = asked.queue.pop_front() {
                                    break chunk;
                                }
                                asked = heard.wait(asked).unwrap();
                            }
                        };
                        let began = std::time::Instant::now();
                        if tiles.len() > 6 {
                            tiles.clear();
                        }
                        let made = Blocks::read(&doorway, &mut tiles, chunk).map(|blocks| {
                            let faces = blocks.faces();
                            let (low, high) = faces.iter().fold((blocks.high, blocks.low), |(low, high), face| {
                                let top = face.at[2] + if matches!(face.facing, Facing::Up | Facing::Down) { 1 } else { face.high };
                                (low.min(face.at[2]), high.max(top))
                            });
                            Made { chunk, faces: faces.iter().map(|face| packed(face, chunk)).collect(), low, high, ms: began.elapsed().as_secs_f32() * 1000.0 }
                        });
                        if sender.send(made).is_err() {
                            return;
                        }
                    }
                })
                .map_err(|error| format!("the near lane's thread couldn't start: {error}"))?;
        }
        Ok(Self { asked, made, threads: threads.max(1) })
    }

    /// What's wanted now, nearest first: replaces what was waiting.
    pub fn ask(&self, chunks: impl IntoIterator<Item = [i32; 2]>) {
        let (lock, heard) = &*self.asked;
        let mut asked = lock.lock().unwrap();
        asked.queue.clear();
        asked.queue.extend(chunks);
        heard.notify_all();
    }

    /// The chunks made since the last taking, as many as `most`.
    pub fn take(&self, most: usize) -> Result<Vec<Made>, String> {
        let mut taken = Vec::new();
        while taken.len() < most {
            match self.made.try_recv() {
                Ok(made) => taken.push(made?),
                Err(_) => break,
            }
        }
        Ok(taken)
    }
}

impl Drop for Lane {
    fn drop(&mut self) {
        let (lock, heard) = &*self.asked;
        lock.lock().unwrap().stop = true;
        heard.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Every open side of every solid block of a chunk, and every cell of water's top, one at a time from the
    /// chunk's own cells: the count the merged faces have to cover, each once.
    fn open_sides(blocks: &Blocks) -> HashSet<([i32; 3], Facing, Block, bool)> {
        let mut open = HashSet::new();
        for north in 0..CHUNK {
            for east in 0..CHUNK {
                for up in blocks.low..blocks.high {
                    let cell = blocks.cell(north, east, up);
                    if cell == 0 {
                        continue;
                    }
                    for facing in Facing::ALL {
                        let step = facing.step();
                        let beyond = blocks.cell(north + step[0], east + step[1], up + step[2]);
                        if Blocks::solid(cell) {
                            if !Blocks::solid(beyond) && !(facing == Facing::Down && up == 0) {
                                open.insert(([north, east, up], facing, cell - 1, false));
                            }
                        } else if facing == Facing::Up && beyond == 0 {
                            open.insert(([north, east, up], facing, cell - 1, true));
                        }
                    }
                }
            }
        }
        open
    }

    /// The cells a face covers, each with its facing, block and kind.
    fn covered(face: &Face) -> Vec<([i32; 3], Facing, Block, bool)> {
        let ([a, b], _) = face.facing.axes();
        let mut cells = Vec::new();
        for dv in 0..face.high {
            for du in 0..face.wide {
                let mut at = face.at;
                at[a] += du;
                at[b] += dv;
                cells.push((at, face.facing, face.block, face.water));
            }
        }
        cells
    }

    /// A chunk's merged faces cover every open side once and nothing else. How many sides, and how many faces.
    fn exact(blocks: &Blocks) -> (usize, usize) {
        let open = open_sides(blocks);
        let faces = blocks.faces();
        let mut seen = HashSet::new();
        for face in &faces {
            assert!(face.wide >= 1 && face.high >= 1, "a face has a size: {face:?}");
            for side in covered(face) {
                assert!(open.contains(&side), "a face covers a side that isn't open: {side:?} of {face:?} in chunk {:?}", blocks.chunk);
                assert!(seen.insert(side), "a side is covered twice: {side:?} in chunk {:?}", blocks.chunk);
            }
        }
        assert_eq!(seen.len(), open.len(), "every open side is covered, in chunk {:?}", blocks.chunk);
        (open.len(), faces.len())
    }

    fn chunk_of(column: [i32; 2]) -> [i32; 2] {
        [column[0].div_euclid(CHUNK), column[1].div_euclid(CHUNK)]
    }

    /// Bare ground: chunks round the spawn, the mountain top's and the sea's marks, and one at the map's corner (where
    /// columns past the map are air, and the world's edge shows its side).
    #[test]
    fn bare_ground_s_faces_are_every_open_side_once() {
        let doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let mut tiles = HashMap::new();
        let ([least_north, least_east], _) = doorway.grid();
        let mut report = Vec::new();
        for (name, column) in [("the spawn", [0, 0]), ("the mountain top", [-605, 78]), ("the plain", [-3975, -1192]), ("the map's corner", [least_north, least_east])] {
            let (mut sides, mut faces, mut chunks) = (0, 0, 0);
            for dn in -1..=1 {
                for de in -1..=1 {
                    let chunk = chunk_of(column);
                    let chunk = [chunk[0] + dn, chunk[1] + de];
                    let blocks = Blocks::read(&doorway, &mut tiles, chunk).unwrap();
                    // The cells are the doorway's own answers, column by column (the chunk's middle row and its edge).
                    for (north, east) in [(0, 0), (15, 16), (31, 31), (-1, 7), (32, 20)] {
                        let column = doorway.column(chunk[0] * CHUNK + north, chunk[1] * CHUNK + east).unwrap();
                        for up in blocks.low..blocks.high {
                            let block = match &column {
                                Column::Past => None,
                                Column::Here { runs, .. } => runs.iter().find(|run| run.from <= up && up < run.to).map(|run| run.block),
                            };
                            assert_eq!(blocks.cell(north, east, up), block.map_or(0, |block| block + 1), "{name}: the cell at {north}, {east}, {up} of chunk {chunk:?} is the doorway's");
                        }
                    }
                    let (s, f) = exact(&blocks);
                    sides += s;
                    faces += f;
                    chunks += 1;
                }
            }
            assert!(sides > 0, "{name} has ground");
            report.push(format!("{name}: {chunks} chunks, {sides} open sides in {faces} faces"));
        }
        eprintln!("Bare ground: {}", report.join("; "));
    }

    /// What's built: the scatter stage's things (a tower, the castle, a house, the hall in the mountain, a cave, a
    /// wood), each chunk its place lies in and those round it.
    #[test]
    fn the_stage_s_faces_are_every_open_side_once() {
        let doorway = Doorway::open_with(&crate::doorway::test_world(), true).unwrap();
        // Its places (each kind of thing: the caves are many, so every fifth of them), and a wood near the spawn.
        let mut places: Vec<[i32; 2]> = doorway.stage().expect("the stage is on").sites().iter().enumerate().filter(|(k, site)| !matches!(site.kind, crate::scatter_stage::Kind::Cave { .. }) || k % 5 == 0).map(|(_, site)| site.at).collect();
        places.push([192, 0]);
        let mut tiles = HashMap::new();
        let mut done = HashSet::new();
        let (mut sides, mut faces) = (0, 0);
        for at in places {
            let chunk = chunk_of(at);
            for dn in -1..=1 {
                for de in -1..=1 {
                    let chunk = [chunk[0] + dn, chunk[1] + de];
                    if done.insert(chunk) {
                        let (s, f) = exact(&Blocks::read(&doorway, &mut tiles, chunk).unwrap());
                        sides += s;
                        faces += f;
                    }
                }
            }
        }
        assert!(done.len() >= 9, "the stage has places");
        eprintln!("The stage: {} chunks, {sides} open sides in {faces} faces", done.len());
    }

    /// A block placed: the chunks it touches are made again and are what's made from scratch; no other chunk
    /// changes.
    #[test]
    fn a_placed_block_changes_only_the_chunks_it_touches() {
        let mut doorway = Doorway::open_with(&crate::doorway::test_world(), false).unwrap();
        let stone = doorway.block("incus:stone");
        let around: Vec<[i32; 2]> = (-1..=1).flat_map(|dn| (-1..=1).map(move |de| [dn, de])).collect();
        let before: Vec<Vec<Face>> = around.iter().map(|chunk| Blocks::read(&doorway, &mut HashMap::new(), *chunk).unwrap().faces()).collect();
        // On the edge of chunk (0, 0) toward the north, well over the ground: a block in the air.
        let ground = match doorway.column(CHUNK - 1, 5).unwrap() {
            Column::Here { ground, .. } => ground,
            Column::Past => panic!("the spawn's chunk is on the map"),
        };
        let cell = [CHUNK - 1, 5, ground + 12];
        doorway.make(Change::Block { cell, block: stone }).unwrap();
        let touched = touched(cell);
        assert_eq!(touched, vec![[0, 0], [1, 0]], "a cell on a chunk's north edge touches that chunk and the one north of it");
        for (chunk, before) in around.iter().zip(&before) {
            let blocks = Blocks::read(&doorway, &mut HashMap::new(), *chunk).unwrap();
            exact(&blocks);
            let after = blocks.faces();
            if *chunk == [0, 0] {
                assert_eq!(after.len(), before.len() + 6, "a block in the air shows six sides in its own chunk");
            } else {
                assert_eq!(&after, before, "chunk {chunk:?} doesn't change (a block in the air hides no side of its neighbour's)");
            }
        }
    }

    /// The lane makes every chunk of a radius, each the mesh made directly; and what a radius comes to (`NEAR_TEST_RADII`:
    /// radii in chunks, for the numbers; `NEAR_TEST_AT`: north, east; `NEAR_TEST_STAGE=1`: with the scatter stage).
    #[test]
    fn the_lane_fills_a_radius() {
        let folder = crate::doorway::test_world();
        let staged = std::env::var("NEAR_TEST_STAGE").as_deref() == Ok("1");
        let at: Vec<f32> = std::env::var("NEAR_TEST_AT").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        let eye = if at.len() == 2 { [at[0], at[1]] } else { [0.5, 0.5] };
        let radii: Vec<f32> = std::env::var("NEAR_TEST_RADII").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_else(|| vec![2.0]);
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(1).max(1);
        for radius in radii {
            let lane = Lane::start(folder.clone(), staged, threads).unwrap();
            let want = wanted(eye, radius);
            // Only chunks in reach are wanted, and every chunk in reach is.
            assert!(want.iter().all(|chunk| nearest_part(*chunk, eye) <= radius * CHUNK as f32), "only chunks in reach are wanted");
            let began = std::time::Instant::now();
            lane.ask(want.iter().copied());
            let mut made: HashMap<[i32; 2], Made> = HashMap::new();
            while made.len() < want.len() {
                for chunk in lane.take(4096).unwrap() {
                    made.insert(chunk.chunk, chunk);
                }
                assert!(began.elapsed().as_secs() < 900, "the lane fills radius {radius} in a quarter of an hour");
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let seconds = began.elapsed().as_secs_f32();
            let faces: usize = made.values().map(|chunk| chunk.faces.len()).sum();
            let work: f32 = made.values().map(|chunk| chunk.ms).sum();
            eprintln!(
                "The near lane, radius {radius} chunks ({:.0} blocks) round {eye:?}{}: {} chunks, {faces} faces ({} triangles), {:.1} MB of faces; filled in {seconds:.2} s on {} threads, {:.2} ms a chunk",
                radius * CHUNK as f32,
                if staged { ", the stage on" } else { "" },
                made.len(),
                faces * 2,
                faces as f32 * 12.0 / 1048576.0,
                lane.threads,
                work / made.len() as f32
            );
            // A few of them against the mesh made directly, and their heights.
            let doorway = Doorway::open_with(&folder, staged).unwrap();
            let mut tiles = HashMap::new();
            for chunk in want.iter().step_by((want.len() / 7).max(1)) {
                let blocks = Blocks::read(&doorway, &mut tiles, *chunk).unwrap();
                let direct: Vec<[u32; 3]> = blocks.faces().iter().map(|face| packed(face, *chunk)).collect();
                assert_eq!(made[chunk].faces, direct, "chunk {chunk:?} from the lane is the mesh made directly");
                assert!(made[chunk].faces.is_empty() || made[chunk].low < made[chunk].high, "a chunk with faces has a height");
            }
        }
    }

    /// A face's three words hold what it is.
    #[test]
    fn a_packed_face_unpacks() {
        let face = Face { at: [31, 7, 1000], facing: Facing::West, wide: 32, high: 1024, block: 513, water: false };
        let [a, b, c] = packed(&face, [-1125, 990]);
        assert_eq!([a & 63, (a >> 6) & 63, (a >> 12) & 1023, (a >> 22) & 7, ((a >> 25) & 31) + 1, (a >> 30) & 1], [31, 7, 1000, 3, 32, 0]);
        assert_eq!([(b & 1023) + 1, b >> 10], [1024, 513]);
        assert_eq!([(c & 0xFFFF) as i32 - 32768, (c >> 16) as i32 - 32768], [-1125, 990]);
    }
}

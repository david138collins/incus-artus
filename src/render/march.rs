//! The far view by ray marching (`docs/far-view.md`, step 1 part 3) [David, 2026-11-02: "its the hard
//! part, we might as well do it now"].
//!
//! **What it draws:** the far land the sheet draws (`distant.rs`), past the real chunks, the same
//! land: every pixel sends a ray out from the camera and finds where it first meets the ground.
//! The switch "drawn by the march" (Render page, Distant terrain) flips between the two, to compare.
//!
//! **What it reads:** the sheet's own tiles. As each tile is made, its corners (height, the coarser
//! ring's height for easing, slope, colours) are also copied into one long list on the graphics
//! card: every ring has a slot for each of its 16 × 16 tiles, the tile at (x, y) in the slot at (x, y)
//! counted round 16. A second list says which tile each slot holds, so a ray never reads a tile that
//! isn't in yet: it reads the ring outside instead. A third keeps each tile's highest land, and each
//! block of 4 × 4 cells' (part 4).
//!
//! **The shader** (`assets/shaders/march.wesl`) draws one triangle over the whole screen, and for
//! each pixel: starts the ray where it leaves the square of real chunks round you; then each step
//! goes past a tile the ray is wholly above, past a block of cells likewise, or across one cell,
//! meeting its two triangles exactly. The first it comes down onto outside the real chunks (as the
//! sheet's `shown` texture marks them) is where it meets the land; there, it lights the land with
//! the sheet's own sums (`lighting.wesl`) and writes how far away it is, so the near world draws in
//! front. The land between corners is the sheet's two triangles a cell, eased into the next ring out
//! the same way, so the two draw the same shape. `MarchState::march_ray` is the shader's steps in
//! Rust, which the tests check against a slow search in small steps.
//!
//! **The far copies** (part 5): their cells (`far_copies::CopyCells`) go to the graphics card in pages,
//! with a record of each copy and a map of which chunks each draws; a ray in one of those chunks steps
//! through its cells one at a time, and meets it coming from the air into anything solid (or water),
//! whichever side it comes from. Plain cells take the far land's look; the rest their block's colour,
//! shaded by the face and darkened by the daylight of the air the ray came through.
//!
//! **Not yet:** night light (the braziers'), its
//! own line in "Graphics card time".

use super::distant::{CELLS, DistantMaterial, DistantSettings, DistantShaderSettings, DistantState, DistantTile, Palette, RING_TILES, TileKey, TileMesh};
use super::far_copies::{CopyCells, FarCopies, FarCopy, FarCopySettings, TileKey as CopyKey};
use std::collections::HashMap;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError};
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::RenderQueue;
use bevy::render::storage::{GpuShaderBuffer, ShaderBuffer};
use bevy::shader::ShaderRef;
use std::sync::{Arc, Mutex};

/// Corners along a tile's side.
const SIDE: usize = (CELLS + 1) as usize;
/// A ring's slots: one for each of its tiles.
const SLOTS: usize = (RING_TILES * RING_TILES) as usize;
/// Numbers kept for each corner: height, the coarser ring's height, the slope north and east; the
/// top's colour and a 1 on land (0 on the sea); the side's colour and a 0.
const CORNER: usize = 12;
/// A corner past the map: no land there.
const MISSING: f32 = -1.0e30;
/// The highest land kept for each tile, for long steps over plain land: for each block of
/// `BLOCK` × `BLOCK` cells (16 of them, row by row east, north first), then the tile's; and 3 spare,
/// so each slot's is 5 × 4 numbers. Each is the highest its corners reach either way their easing
/// goes, so the eased land is never higher.
const TOPS: usize = 20;
const TILE_TOP: usize = 16;
const BLOCK: i32 = 4;

/// Cells across a far copy (one section of the mesher's, `far_copies.rs`).
const COPY_CELLS: i32 = 32;
/// A page of the copies' cells on the graphics card: 32 levels of one copy (128 KB).
const PAGE: usize = (COPY_CELLS * COPY_CELLS * COPY_CELLS) as usize;
/// How many pages there are (64 MB), how many copies at once, and the most pages one copy takes
/// (512 cells tall). A copy that doesn't fit is left to the land.
const PAGES: usize = 512;
const MOST_COPIES: usize = 1024;
const COPY_PAGES: usize = 16;
/// Numbers kept for each copy: its first cell's corner (x, y, z, blocks) and its cell size; how many
/// cells tall it is and how many pages it has; 2 spare; then its pages.
const COPY_RECORD: usize = 8 + COPY_PAGES;
/// The copies' map of chunks (which copy, if any, draws each), as wide as the sheet's `shown` texture.
const MAP: i32 = super::distant::SHOWN_SIZE;

/// The far copies the march draws (step 1 part 5): each copy's cells in pages on the graphics card,
/// a record of each, and a map of which chunks each covers, round the camera's chunk.
#[derive(Default)]
struct Copies {
    cells_buffer: Option<Handle<ShaderBuffer>>,
    records_buffer: Option<Handle<ShaderBuffer>>,
    map_buffer: Option<Handle<ShaderBuffer>>,
    colours_buffer: Option<Handle<ShaderBuffer>>,
    drawn: HashMap<CopyKey, MarchedCopy>,
    free_pages: Vec<u32>,
    free_records: Vec<u32>,
    /// Whether the map needs making again.
    map_stale: bool,
    /// The colours the cells are drawn in were sent for this palette.
    palette: Option<Arc<Palette>>,
    /// The highest any copy reaches (blocks).
    top: f32,
    /// Copies that didn't fit (for the log, once each).
    left_out: usize,
}

struct MarchedCopy {
    cells: Arc<CopyCells>,
    record: u32,
    pages: Vec<u32>,
    /// Its least and greatest corner (blocks, north and east) and its top.
    footprint: (IVec2, IVec2, f32),
}

/// Parts of the lists to write on the graphics card, handed from the game to Bevy's drawing side
/// (`write_march`), which writes each straight into the list already there. Only the tiles that came
/// in go, a few kilobytes each. [Build 13 sent both lists whole, about 25 MB, ten times a second
/// while tiles came in, and each time took 8 to 12 ms of the frame.]
#[derive(Resource, Clone, Default)]
pub struct MarchWrites(Arc<Mutex<Vec<Write>>>);

struct Write {
    buffer: AssetId<ShaderBuffer>,
    /// Where in the list (bytes) and what goes there.
    offset: u64,
    bytes: Vec<u8>,
}

/// Numbers as the graphics card reads them.
fn bytes_of<T: Copy>(values: &[T], to_bytes: impl Fn(T) -> [u8; 4]) -> Vec<u8> {
    values.iter().flat_map(|&v| to_bytes(v)).collect()
}

/// The march's copy of the far land, and what it's drawn with.
#[derive(Resource, Default)]
pub struct MarchState {
    material: Option<Handle<MarchMaterial>>,
    screen: Option<Entity>,
    corners_buffer: Option<Handle<ShaderBuffer>>,
    slots_buffer: Option<Handle<ShaderBuffer>>,
    tops_buffer: Option<Handle<ShaderBuffer>>,
    writes: MarchWrites,
    /// Every ring's slots' corners, `CORNER` numbers each.
    corners: Vec<f32>,
    /// Every ring's slots' highest land, `TOPS` numbers each.
    tops: Vec<f32>,
    /// Every ring's slots: the tile's x and y, then 0 nothing yet, 1 land, 2 wholly past the map; and a 0.
    slots: Vec<i32>,
    rings: u32,
    /// The finest ring's cell (blocks).
    finest: f32,
    /// The distant terrain's `cleared` count this copy follows.
    cleared: u64,
    /// The highest land in it (a ray going up past it has missed).
    highest: f32,
    /// Started again: the lists go to the graphics card whole, once (made again if their size changed).
    whole: bool,
    /// The far copies the march draws (step 1 part 5).
    copies: Copies,
}

impl MarchState {
    /// Starts again, empty, when the sheet threw its tiles away or the number of rings or the
    /// finest cell changed.
    pub(super) fn follow(&mut self, cleared: u64, rings: u32, finest: i32) {
        if self.cleared == cleared && self.rings == rings && self.finest == finest as f32 && !self.slots.is_empty() {
            return;
        }
        self.cleared = cleared;
        self.rings = rings;
        self.finest = finest as f32;
        self.corners = vec![MISSING; rings as usize * SLOTS * SIDE * SIDE * CORNER];
        self.tops = vec![MISSING; rings as usize * SLOTS * TOPS];
        self.slots = vec![0; rings as usize * SLOTS * 4];
        self.highest = f32::MIN;
        self.whole = true;
    }

    fn queue(&self, buffer: &Option<Handle<ShaderBuffer>>, offset: usize, bytes: Vec<u8>) {
        if let (Some(buffer), Ok(mut writes)) = (buffer, self.writes.0.lock()) {
            writes.push(Write { buffer: buffer.id(), offset: offset as u64, bytes });
        }
    }

    /// Where a tile goes: its ring's slot at (x, y) counted round the ring's width.
    fn slot(key: TileKey) -> usize {
        key.ring as usize * SLOTS + (key.y.rem_euclid(RING_TILES) * RING_TILES + key.x.rem_euclid(RING_TILES)) as usize
    }

    /// Copies a finished tile in (`None`: wholly past the map).
    pub(super) fn store(&mut self, key: TileKey, tile: Option<&TileMesh>) {
        if key.ring as u32 >= self.rings {
            return;
        }
        let slot = Self::slot(key);
        if let Some(tile) = tile {
            let base = slot * SIDE * SIDE * CORNER;
            for (k, &index) in tile.grid.iter().enumerate() {
                let at = base + k * CORNER;
                if index == u32::MAX {
                    self.corners[at] = MISSING;
                    continue;
                }
                let i = index as usize;
                // Bevy's up is the game's height; the tile's corner sits at height 0.
                let (height, target) = (tile.positions[i][1] + tile.corner.z, tile.morph[i][0]);
                let (slope, top, side) = (tile.slopes[i], tile.colours[i], tile.sides[i]);
                self.corners[at..at + CORNER].copy_from_slice(&[height, target, slope[0], slope[1], top[0], top[1], top[2], top[3], side[0], side[1], side[2], 0.0]);
                self.highest = self.highest.max(height).max(target);
            }
            let span = base..base + SIDE * SIDE * CORNER;
            self.queue(&self.corners_buffer, span.start * 4, bytes_of(&self.corners[span], f32::to_le_bytes));
            // The highest land in each block of cells (its corners round it too) and in the tile.
            let blocks = (CELLS / BLOCK) as usize;
            let mut tops = [MISSING; TOPS];
            for (k, &index) in tile.grid.iter().enumerate() {
                if index == u32::MAX {
                    continue;
                }
                let top = tile.positions[index as usize][1].max(tile.morph[index as usize][0]);
                let (i, j) = (k % SIDE, k / SIDE);
                // A corner on a block's edge is in the blocks either side.
                for bj in (j.saturating_sub(1) / BLOCK as usize)..=(j / BLOCK as usize).min(blocks - 1) {
                    for bi in (i.saturating_sub(1) / BLOCK as usize)..=(i / BLOCK as usize).min(blocks - 1) {
                        tops[bj * blocks + bi] = tops[bj * blocks + bi].max(top);
                    }
                }
                tops[TILE_TOP] = tops[TILE_TOP].max(top);
            }
            self.tops[slot * TOPS..slot * TOPS + TOPS].copy_from_slice(&tops);
            // Anything a far copy holds higher than the land (a tower), so no long step goes over it.
            let raises: Vec<(IVec2, IVec2, f32)> = self.copies.drawn.values().map(|copy| copy.footprint).collect();
            for footprint in raises {
                self.raise_tops(slot, key, footprint);
            }
            self.queue(&self.tops_buffer, slot * TOPS * 4, bytes_of(&self.tops[slot * TOPS..slot * TOPS + TOPS], f32::to_le_bytes));
        }
        // The slot after its corners, so a ray never reads it before they're there.
        self.slots[slot * 4..slot * 4 + 4].copy_from_slice(&[key.x, key.y, if tile.is_some() { 1 } else { 2 }, 0]);
        self.queue(&self.slots_buffer, slot * 16, bytes_of(&self.slots[slot * 4..slot * 4 + 4], i32::to_le_bytes));
    }

    /// Raises a ring tile's highest land (in `self.tops`, not sent) over a far copy's footprint (its
    /// least and greatest corner, blocks, and its top): its blocks of cells that overlap, and the tile.
    /// Returns whether anything rose.
    fn raise_tops(&mut self, slot: usize, key: TileKey, (low, high, top): (IVec2, IVec2, f32)) -> bool {
        let size = self.finest * CELLS as f32 * 2f32.powi(key.ring as i32);
        let origin = Vec2::new(key.x as f32, key.y as f32) * size;
        let block = size / (CELLS / BLOCK) as f32;
        let (from, to) = (((low.as_vec2() - origin) / block).floor(), ((high.as_vec2() - origin) / block).ceil());
        let blocks = CELLS / BLOCK;
        let (from, to) = (from.max(Vec2::ZERO).as_ivec2(), to.min(Vec2::splat(blocks as f32)).as_ivec2());
        let mut rose = false;
        for by in from.y..to.y {
            for bx in from.x..to.x {
                let at = slot * TOPS + (by * blocks + bx) as usize;
                if self.tops[at] < top {
                    self.tops[at] = top;
                    self.tops[slot * TOPS + TILE_TOP] = self.tops[slot * TOPS + TILE_TOP].max(top);
                    rose = true;
                }
            }
        }
        rose
    }

    /// Keeps the copies the march draws the same as the far copies drawn (`tiles`, those drawn first),
    /// with their map; none while `on` is false.
    fn follow_copies(&mut self, tiles: Vec<(CopyKey, Arc<CopyCells>)>, on: bool) {
        if self.copies.free_pages.is_empty() && self.copies.drawn.is_empty() {
            self.copies.free_pages = (0..PAGES as u32).rev().collect();
            self.copies.free_records = (0..MOST_COPIES as u32).rev().collect();
            self.copies.map_stale = true;
        }
        let mut wanted: Vec<(CopyKey, Arc<CopyCells>)> = Vec::new();
        if on {
            for (key, cells) in tiles {
                if !wanted.iter().any(|(k, _)| *k == key) {
                    wanted.push((key, cells));
                }
            }
        }
        // Gone, or made again: let go.
        let gone: Vec<CopyKey> =
            self.copies.drawn.iter().filter(|(key, copy)| !wanted.iter().any(|(k, cells)| k == *key && Arc::ptr_eq(cells, &copy.cells))).map(|(key, _)| *key).collect();
        for key in gone {
            if let Some(copy) = self.copies.drawn.remove(&key) {
                self.copies.free_pages.extend(copy.pages);
                self.copies.free_records.push(copy.record);
                self.copies.map_stale = true;
            }
        }
        // New: into pages, with a record.
        for (key, cells) in &wanted {
            if self.copies.drawn.contains_key(key) {
                continue;
            }
            let pages = (cells.tall as usize).div_ceil(COPY_CELLS as usize);
            if pages > COPY_PAGES || pages > self.copies.free_pages.len() || self.copies.free_records.is_empty() {
                self.copies.left_out += 1;
                if self.copies.left_out.is_power_of_two() {
                    info!("March: a far copy at ({}, {}) didn't fit ({} pages; {} free); {} left out so far", cells.corner.x, cells.corner.y, pages, self.copies.free_pages.len(), self.copies.left_out);
                }
                continue;
            }
            let taken: Vec<u32> = (0..pages).filter_map(|_| self.copies.free_pages.pop()).collect();
            let record = self.copies.free_records.pop().unwrap_or(0);
            for (k, &page) in taken.iter().enumerate() {
                let span = k * PAGE..((k + 1) * PAGE).min(cells.cells.len());
                self.queue(&self.copies.cells_buffer, page as usize * PAGE * 4, bytes_of(&cells.cells[span], u32::to_le_bytes));
            }
            let mut numbers = [0u32; COPY_RECORD];
            numbers[..6].copy_from_slice(&[cells.corner.x as u32, cells.corner.y as u32, cells.corner.z as u32, cells.scale as u32, cells.tall as u32, pages as u32]);
            numbers[8..8 + pages].copy_from_slice(&taken);
            self.queue(&self.copies.records_buffer, record as usize * COPY_RECORD * 4, bytes_of(&numbers, u32::to_le_bytes));
            let width = COPY_CELLS * cells.scale;
            let footprint = (cells.corner.truncate(), cells.corner.truncate() + IVec2::splat(width), (cells.corner.z + cells.tall * cells.scale) as f32);
            self.copies.drawn.insert(*key, MarchedCopy { cells: cells.clone(), record, pages: taken, footprint });
            self.copies.map_stale = true;
            // The ring tiles under it: their highest land raised over it.
            for slot in 0..self.slots.len() / 4 {
                let info = &self.slots[slot * 4..slot * 4 + 4];
                if info[2] != 1 {
                    continue;
                }
                let ring_key = TileKey { ring: (slot / SLOTS) as u8, x: info[0], y: info[1] };
                if self.raise_tops(slot, ring_key, footprint) {
                    self.queue(&self.tops_buffer, slot * TOPS * 4, bytes_of(&self.tops[slot * TOPS..slot * TOPS + TOPS], f32::to_le_bytes));
                }
            }
        }
        self.copies.top = self.copies.drawn.values().map(|copy| copy.footprint.2).fold(f32::MIN, f32::max);
        // The map: which copy draws each chunk (one drawn now wins over one retiring). It's laid out
        // by the world, each chunk at its place counted round the map's width, not round you: one laid
        // out round you was made again at every chunk you crossed, and for a frame the graphics card
        // could read it against the last one's middle, so a mountain showed sky through it while you
        // moved [David, build 19: "shards of sky appeared in several layers, stopping when i stopped
        // moving"]. The copies all lie within the hand-off mask (`far_copies::MASK_REACH`, less than
        // half the map's width from you), so no two share a place; a ray also checks its copy covers it.
        if self.copies.map_stale {
            self.copies.map_stale = false;
            let mut map = vec![0u32; (MAP * MAP) as usize];
            for (key, _) in &wanted {
                let Some(copy) = self.copies.drawn.get(key) else { continue };
                let (low, high, _) = copy.footprint;
                let chunk = crate::world::CHUNK_SIZE;
                for cy in low.y.div_euclid(chunk)..high.y.div_euclid(chunk) {
                    for cx in low.x.div_euclid(chunk)..high.x.div_euclid(chunk) {
                        let at = (cy.rem_euclid(MAP) * MAP + cx.rem_euclid(MAP)) as usize;
                        if map[at] == 0 {
                            map[at] = copy.record + 1;
                        }
                    }
                }
            }
            self.queue(&self.copies.map_buffer, 0, bytes_of(&map, u32::to_le_bytes));
        }
    }

    /// The CPU's twins of the shader's `locate`, `eased_height` and march, for the tests (the
    /// shader's are written to follow these line by line).
    ///
    /// Where the land at `at` (game axes) is kept, with the camera at `camera`: the finest ring whose
    /// tile there is in.
    #[cfg(test)]
    fn locate(&self, finest: f32, camera: Vec2, at: Vec2) -> Located {
        let mut beyond = true;
        for ring in 0..self.rings {
            let size = finest * CELLS as f32 * 2f32.powi(ring as i32);
            let tile = (at / size).floor().as_ivec2();
            let start = 2 * (camera / (2.0 * size)).floor().as_ivec2() - IVec2::splat(RING_TILES / 2);
            if tile.cmplt(start).any() || tile.cmpge(start + RING_TILES).any() {
                continue;
            }
            beyond = false;
            let slot = Self::slot(TileKey { ring: ring as u8, x: tile.x, y: tile.y });
            let info = &self.slots[slot * 4..slot * 4 + 4];
            if info[2] == 0 || info[0] != tile.x || info[1] != tile.y {
                continue;
            }
            if info[2] == 2 {
                return Located::Beyond;
            }
            return Located::Land(Place { slot, origin: tile.as_vec2() * size, size });
        }
        if beyond { Located::Beyond } else { Located::NotIn }
    }

    /// Corner (i, j)'s height, eased toward the coarser ring's toward its ring's outer edge; `None`
    /// past the map.
    #[cfg(test)]
    fn eased(&self, place: Place, i: usize, j: usize, camera: Vec2) -> Option<f32> {
        let at = (place.slot * SIDE * SIDE + j * SIDE + i) * CORNER;
        let (height, target) = (self.corners[at], self.corners[at + 1]);
        let spot = place.origin + Vec2::new(i as f32, j as f32) * (place.size / CELLS as f32);
        let away = (spot - camera).abs().max_element();
        let eased = ((away - super::distant::EASE_FROM * place.size) / place.size.max(1.0)).clamp(0.0, 1.0);
        (height > MISSING / 2.0).then_some(height + (target - height) * eased)
    }

    /// The land's height at `at`; `None` where there's none.
    #[cfg(test)]
    fn height_at(&self, finest: f32, camera: Vec2, at: Vec2) -> Option<f32> {
        let Located::Land(place) = self.locate(finest, camera, at) else { return None };
        let local = (at - place.origin) / (place.size / CELLS as f32);
        let (i, j) = ((local.x.floor() as usize).min(CELLS as usize - 1), (local.y.floor() as usize).min(CELLS as usize - 1));
        let (u, v) = (local.x - i as f32, local.y - j as f32);
        let corner = |di: usize, dj: usize| self.eased(place, i + di, j + dj, camera);
        let (a, c) = (corner(0, 0)?, corner(1, 1)?);
        Some(if u >= v { a * (1.0 - u) + corner(1, 0)? * (u - v) + c * v } else { a * (1.0 - v) + c * u + corner(0, 1)? * (v - u) })
    }

    /// Where a ray from `origin` along `ray` (game axes, `ray` of length 1) first comes down onto the
    /// land, from `start` on, outside what `drawn_near` says the near world draws: the distance, or
    /// `None` if it misses (or goes deep under the land first). Each step is one of: past a tile the
    /// ray is wholly above (by its highest), past a block of 4 × 4 cells likewise, or across one cell,
    /// meeting its two triangles exactly.
    ///
    /// **Never slipping through** [build 15: "a field of sky speckles"]: a ray meeting the land right
    /// on the line between a cell's two triangles, or right at a cell's edge, could be turned away by
    /// both triangles, or by both cells, by a rounding hair. So a triangle takes a meeting a hair past
    /// its edge, and a ray that was above the land leaving one step and is under it entering the next
    /// met it there, between them.
    #[cfg(test)]
    fn march_ray(&self, finest: f32, origin: Vec3, ray: Vec3, start: f32, most: u32, drawn_near: impl Fn(Vec2) -> bool) -> Option<f32> {
        let camera = origin.truncate();
        let mut t = start;
        // Whether the ray was above the land where the last step left it (not known at the start:
        // a ray starting under the land went in under the near world).
        let mut was_above = false;
        for _ in 0..most {
            let p = origin + ray * t;
            if ray.z >= 0.0 && p.z > self.highest {
                return None;
            }
            let place = match self.locate(finest, camera, p.truncate()) {
                Located::Beyond => return None,
                Located::NotIn => {
                    t += (t * 0.05).max(16.0);
                    was_above = false;
                    continue;
                }
                Located::Land(place) => place,
            };
            let cell = place.size / CELLS as f32;
            // In cells across the tile from its corner, and how fast that changes along the ray.
            let local = (p.truncate() - place.origin) / cell;
            let rate = ray.truncate() / cell;
            // Where the ray leaves the square from `low` to `high` (cells), as a distance along it.
            let leave = |low: Vec2, high: Vec2| {
                let axis = |at: f32, rate: f32, low: f32, high: f32| if rate > 1e-9 { (high - at) / rate } else if rate < -1e-9 { (low - at) / rate } else { f32::MAX };
                t + axis(local.x, rate.x, low.x, high.x).min(axis(local.y, rate.y, low.y, high.y)).max(0.0)
            };
            let nudge = cell * 1e-3;
            let tops = &self.tops[place.slot * TOPS..place.slot * TOPS + TOPS];
            let above_all = |exit: f32, top: f32| p.z.min(origin.z + ray.z * exit) > top;
            let out_of_tile = leave(Vec2::ZERO, Vec2::splat(CELLS as f32));
            if above_all(out_of_tile, tops[TILE_TOP]) {
                t = out_of_tile + nudge;
                was_above = true;
                continue;
            }
            let block = (local / BLOCK as f32).floor().clamp(Vec2::ZERO, Vec2::splat((CELLS / BLOCK - 1) as f32));
            let out_of_block = leave(block * BLOCK as f32, (block + 1.0) * BLOCK as f32);
            if above_all(out_of_block, tops[(block.y as usize) * (CELLS / BLOCK) as usize + block.x as usize]) {
                t = out_of_block + nudge;
                was_above = true;
                continue;
            }
            let ij = local.floor().clamp(Vec2::ZERO, Vec2::splat((CELLS - 1) as f32));
            let out_of_cell = leave(ij, ij + 1.0);
            let (i, j) = (ij.x as usize, ij.y as usize);
            let heights = [(0, 0), (1, 0), (1, 1), (0, 1)].map(|(di, dj)| self.eased(place, i + di, j + dj, camera));
            let [Some(a), Some(b), Some(c), Some(d)] = heights else {
                t = out_of_cell + nudge;
                was_above = false;
                continue;
            };
            // The land's height at a spot across the cell (u, v from 0 to 1).
            let surface = |u: f32, v: f32| if u >= v { a * (1.0 - u) + b * (u - v) + c * v } else { a * (1.0 - v) + c * u + d * (v - u) };
            let (u, v) = (local.x - ij.x, local.y - ij.y);
            let under = surface(u, v);
            // Above the land leaving the last step and under it here: it met the land in between.
            if was_above && p.z < under && !drawn_near(p.truncate()) {
                return Some(t);
            }
            // Each triangle's plane: height = h0 + gu u + gv v; and which half of the cell it covers
            // (with a hair either side, where the two planes are the same height anyway).
            let mut hit = f32::MAX;
            for (h0, gu, gv, first_half) in [(a, b - a, c - b, true), (a, c - d, d - a, false)] {
                let gap = p.z - (h0 + gu * u + gv * v);
                let closing = ray.z - gu * rate.x - gv * rate.y;
                if gap >= 0.0 && closing < 0.0 {
                    let at = t + gap / -closing;
                    let across = (u + rate.x * (at - t)) - (v + rate.y * (at - t));
                    let in_half = if first_half { across >= -1e-3 } else { across <= 1e-3 };
                    if at <= out_of_cell + nudge && in_half {
                        hit = hit.min(at);
                    }
                }
            }
            if hit < f32::MAX && !drawn_near((origin + ray * hit).truncate()) {
                return Some(hit);
            }
            // Deep under the land: gone in under the near world.
            if p.z < under - (2.0 * cell).max(16.0) {
                return None;
            }
            let leaving = ((local + rate * (out_of_cell - t)) - ij).clamp(Vec2::ZERO, Vec2::ONE);
            was_above = origin.z + ray.z * out_of_cell >= surface(leaving.x, leaving.y);
            t = out_of_cell + nudge;
        }
        None
    }
}

/// Where a spot's land is kept: its tile's slot, the tile's corner (game axes) and its size.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct Place {
    slot: usize,
    origin: Vec2,
    size: f32,
}

#[cfg(test)]
enum Located {
    Land(Place),
    /// In a ring, but its tile there isn't in yet (nor any ring's outside it).
    NotIn,
    /// Past every ring, or wholly past the map: no land here or further on.
    Beyond,
}

// ── The material ────────────────────────────────────────────────────────────

/// The far land's settings and textures (copied from the sheet's material whenever it changes), the
/// march's own, and its two lists.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct MarchMaterial {
    #[uniform(0)]
    pub settings: DistantShaderSettings,
    #[texture(1)]
    pub shown: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub light: Handle<Image>,
    #[uniform(4)]
    pub march: MarchShaderSettings,
    #[storage(5, read_only)]
    pub corners: Handle<ShaderBuffer>,
    #[storage(6, read_only)]
    pub slots: Handle<ShaderBuffer>,
    #[storage(7, read_only)]
    pub tops: Handle<ShaderBuffer>,
    /// The far copies' cells, records, map of chunks, and the blocks' colours (top, side) to draw them in.
    #[storage(8, read_only)]
    pub copy_cells: Handle<ShaderBuffer>,
    #[storage(9, read_only)]
    pub copy_records: Handle<ShaderBuffer>,
    #[storage(10, read_only)]
    pub copy_map: Handle<ShaderBuffer>,
    #[storage(11, read_only)]
    pub block_colours: Handle<ShaderBuffer>,
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct MarchShaderSettings {
    /// x rings, y the finest cell (blocks), z the highest land (or far copy), w the most steps a pixel takes.
    pub grid: Vec4,
    /// The far copies: x 1 if the march draws them.
    pub copies: Vec4,
    /// The square of real chunks the rays start at the edge of (game axes, blocks): its least x and
    /// y, its greatest x and y; all 0 if there's none.
    pub inner: Vec4,
}

impl Material for MarchMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/march.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/march.wesl".into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// The screen's triangle: made once, drawn only while the march is on.
#[derive(Component)]
pub struct MarchScreen;

pub fn setup_march(
    mut commands: Commands,
    mut state: ResMut<MarchState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MarchMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut images: ResMut<Assets<Image>>,
    writes: Res<MarchWrites>,
) {
    state.writes = writes.clone();
    // Bound before any tile is in, so each list needs something in it.
    let corners = buffers.add(ShaderBuffer::new(vec![MISSING; CORNER], RenderAssetUsages::RENDER_WORLD));
    let slots = buffers.add(ShaderBuffer::new(vec![0i32; 4], RenderAssetUsages::RENDER_WORLD));
    let tops = buffers.add(ShaderBuffer::new(vec![MISSING; TOPS], RenderAssetUsages::RENDER_WORLD));
    // The copies' lists at their full size from the start (written into a piece at a time).
    let copy_cells = buffers.add(ShaderBuffer::new(vec![0u32; PAGES * PAGE], RenderAssetUsages::RENDER_WORLD));
    let copy_records = buffers.add(ShaderBuffer::new(vec![0u32; MOST_COPIES * COPY_RECORD], RenderAssetUsages::RENDER_WORLD));
    let copy_map = buffers.add(ShaderBuffer::new(vec![0u32; (MAP * MAP) as usize], RenderAssetUsages::RENDER_WORLD));
    let block_colours = buffers.add(ShaderBuffer::new(vec![0.5f32; 8], RenderAssetUsages::RENDER_WORLD));
    let material = materials.add(MarchMaterial {
        settings: DistantShaderSettings::default(),
        shown: images.add(super::light::placeholder()),
        light: images.add(super::light::placeholder()),
        march: MarchShaderSettings::default(),
        corners: corners.clone(),
        slots: slots.clone(),
        tops: tops.clone(),
        copy_cells: copy_cells.clone(),
        copy_records: copy_records.clone(),
        copy_map: copy_map.clone(),
        block_colours: block_colours.clone(),
    });
    state.copies.cells_buffer = Some(copy_cells);
    state.copies.records_buffer = Some(copy_records);
    state.copies.map_buffer = Some(copy_map);
    state.copies.colours_buffer = Some(block_colours);
    // One triangle past the screen's corners: the screen is wholly inside it.
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[-1.0f32, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]]);
    let screen = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden, NoFrustumCulling, MarchScreen)).id();
    state.material = Some(material);
    state.screen = Some(screen);
    state.corners_buffer = Some(corners);
    state.slots_buffer = Some(slots);
    state.tops_buffer = Some(tops);
}

/// After starting again, makes the lists on the graphics card the right size and sends them whole
/// (tiles coming in go a piece at a time, `MarchState::store`); and keeps the material's settings in
/// step with the sheet's and the march's own.
#[allow(clippy::too_many_arguments)]
pub fn update_march(
    mut state: ResMut<MarchState>,
    (distant, settings): (Res<DistantState>, Res<DistantSettings>),
    (far, far_settings): (Res<FarCopies>, Res<FarCopySettings>),
    mut changed_sheet: MessageReader<AssetEvent<DistantMaterial>>,
    sheets: Res<Assets<DistantMaterial>>,
    mut materials: ResMut<Assets<MarchMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let _timed = crate::render::Timed::new("march");
    let Some(handle) = state.material.clone() else { return };
    if state.whole && !state.slots.is_empty() {
        state.whole = false;
        // What's waiting for the land's lists is in them now (the copies' still goes).
        let land: Vec<AssetId<ShaderBuffer>> = [&state.corners_buffer, &state.tops_buffer, &state.slots_buffer].iter().filter_map(|buffer| buffer.as_ref().map(|b| b.id())).collect();
        if let Ok(mut writes) = state.writes.0.lock() {
            writes.retain(|write| !land.contains(&write.buffer));
        }
        // The lists whole, at their size (once: tiles coming in after go a piece at a time).
        if let Some(buffer) = &state.corners_buffer {
            let _ = buffers.insert(buffer, ShaderBuffer::new(state.corners.clone(), RenderAssetUsages::RENDER_WORLD));
        }
        if let Some(buffer) = &state.tops_buffer {
            let _ = buffers.insert(buffer, ShaderBuffer::new(state.tops.clone(), RenderAssetUsages::RENDER_WORLD));
        }
        if let Some(buffer) = &state.slots_buffer {
            let _ = buffers.insert(buffer, ShaderBuffer::new(state.slots.clone(), RenderAssetUsages::RENDER_WORLD));
        }
    }
    // The far copies, if the march draws them, and the colours their cells are drawn in.
    let marched = far_settings.marched(&settings);
    state.follow_copies(if marched { far.march_tiles() } else { Vec::new() }, marched);
    if let Some(palette) = distant.palette().cloned()
        && state.copies.palette.as_ref().is_none_or(|old| !Arc::ptr_eq(old, &palette))
    {
        let colours: Vec<f32> = palette.blocks.iter().flat_map(|(top, side)| [top[0], top[1], top[2], 1.0, side[0], side[1], side[2], 1.0]).collect();
        if let Some(buffer) = &state.copies.colours_buffer
            && !colours.is_empty()
        {
            let _ = buffers.insert(buffer, ShaderBuffer::new(colours, RenderAssetUsages::RENDER_WORLD));
        }
        state.copies.palette = Some(palette);
    }
    let sheet_changed = changed_sheet.read().any(|event| matches!(event, AssetEvent::Modified { .. }));
    let inner = distant.inner.map_or(Vec4::ZERO, |(centre, reach)| {
        let size = crate::world::CHUNK_SIZE as f32;
        Vec4::new((centre.x - reach) as f32 * size, (centre.y - reach) as f32 * size, (centre.x + reach + 1) as f32 * size, (centre.y + reach + 1) as f32 * size)
    });
    let march = MarchShaderSettings {
        grid: Vec4::new(state.rings as f32, settings.finest_cell as f32, state.highest.max(state.copies.top), settings.march_most as f32),
        copies: Vec4::new(if marched { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0),
        inner,
    };
    let unchanged = materials.get(&handle).is_some_and(|material| material.march == march);
    if unchanged && !sheet_changed {
        return;
    }
    let sheet = distant.material.as_ref().and_then(|sheet| sheets.get(sheet)).cloned();
    if let Some(mut material) = materials.get_mut(&handle) {
        material.march = march;
        if let Some(sheet) = sheet {
            material.settings = sheet.settings;
            material.shown = sheet.shown;
            material.light = sheet.light;
        }
    }
}

/// On Bevy's drawing side, once the lists are on the graphics card: writes the pieces the game
/// handed over into them. A piece for a list not there yet, or not yet made again at its new size,
/// waits (the game throws away what's waiting whenever it starts again).
pub fn write_march(writes: Res<MarchWrites>, gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>, queue: Res<RenderQueue>) {
    let Ok(mut pending) = writes.0.lock() else { return };
    let mut waiting = Vec::new();
    for write in pending.drain(..) {
        match gpu_buffers.get(write.buffer) {
            Some(gpu) if write.offset + write.bytes.len() as u64 <= gpu.buffer.size() => queue.write_buffer(&gpu.buffer, write.offset, &write.bytes),
            _ => waiting.push(write),
        }
    }
    *pending = waiting;
}

/// One of the two draws the far land: the march's triangle or the sheet's tiles, by the switch
/// (neither while the deep screenshot is being taken: it draws its own); and the far copies' own
/// blocks are hidden while the march draws them.
#[allow(clippy::type_complexity)]
pub fn show_one_drawer(
    state: Res<MarchState>,
    settings: Res<DistantSettings>,
    far_settings: Res<FarCopySettings>,
    shot: Res<super::deep_shot::DeepShot>,
    mut screens: Query<&mut Visibility, (With<MarchScreen>, Without<DistantTile>, Without<FarCopy>)>,
    mut tiles: Query<&mut Visibility, (With<DistantTile>, Without<MarchScreen>, Without<FarCopy>)>,
    mut copies: Query<&mut Visibility, (With<FarCopy>, Without<MarchScreen>, Without<DistantTile>)>,
) {
    // New copies come in all the time: every frame.
    let marched = far_settings.marched(&settings) && !shot.busy();
    for mut visibility in &mut copies {
        visibility.set_if_neq(if marched { Visibility::Hidden } else { Visibility::Inherited });
    }
    if shot.busy() {
        if let Some(mut visibility) = state.screen.and_then(|screen| screens.get_mut(screen).ok()) {
            visibility.set_if_neq(Visibility::Hidden);
        }
        return;
    }
    let march = settings.march && settings.enabled;
    if let Some(mut visibility) = state.screen.and_then(|screen| screens.get_mut(screen).ok()) {
        visibility.set_if_neq(if march { Visibility::Inherited } else { Visibility::Hidden });
    }
    if settings.is_changed() {
        for mut visibility in &mut tiles {
            visibility.set_if_neq(if march { Visibility::Hidden } else { Visibility::Inherited });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CompiledMap, TerrainBlocks};

    #[test]
    fn the_march_finds_the_land_the_sheet_draws() {
        // The sheet's tiles round a spot, copied in; at every corner and between corners the
        // march's land is the sheet's triangles' height.
        let width = 64;
        // Land rising east and, more gently, north, with a ripple: no two cells alike.
        let z: Vec<f32> = (0..width * width).map(|i| 150.0 + (i % width) as f32 * 1.5 + (i / width) as f32 * 0.5 + ((i * 7) % 5) as f32).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let registry = crate::data::BlockRegistry::default();
        let blocks = TerrainBlocks::from_registry(&registry);
        let palette = super::super::distant::Palette::from_textures(&registry, &blocks);
        let finest = 16;
        let camera = Vec3::new(40.0, -70.0, 200.0);
        let mut state = MarchState::default();
        state.follow(1, 1, 16);
        let mut meshes = Vec::new();
        for key in super::super::distant::wanted_tiles(camera, finest, 1) {
            let tile = super::super::distant::tile_mesh(&map, &blocks, &palette, key, finest);
            state.store(key, tile.as_ref());
            meshes.extend(tile.map(|tile| (key, tile)));
        }
        assert!(state.highest > f32::MIN, "some land came in");
        let mut checked = 0;
        for (key, tile) in &meshes {
            if key.ring != 0 {
                continue;
            }
            let cell = finest as f32;
            for k in [0usize, 5, 17 * 3 + 4] {
                let index = tile.grid[k];
                if index == u32::MAX {
                    continue;
                }
                let (i, j) = (k % SIDE, k / SIDE);
                // The cell north-east of it, which the spot just inside is in, must be wholly on the map.
                if [k + 1, k + SIDE + 1].iter().any(|&n| tile.grid[n] == u32::MAX) {
                    continue;
                }
                let at = Vec2::new(tile.corner.x + i as f32 * cell, tile.corner.y + j as f32 * cell);
                // In ring 0 within its easing's start, nothing is eased: the corner's own height.
                if (at - camera.truncate()).abs().max_element() >= super::super::distant::EASE_FROM * cell * CELLS as f32 {
                    continue;
                }
                let found = state.height_at(cell, camera.truncate(), at + Vec2::splat(0.001)).expect("land at a corner");
                assert!((found - tile.positions[index as usize][1]).abs() < 0.05, "at {at}: {found} against {}", tile.positions[index as usize][1]);
                checked += 1;
            }
        }
        assert!(checked > 10, "checked {checked} corners");
        // Between corners: the height of the sheet's own triangle over the spot.
        let mut between = 0;
        for (key, tile) in meshes.iter().filter(|(key, _)| key.ring == 0) {
            let size = (finest * CELLS) as f32;
            for spot in [Vec2::new(0.37, 0.61), Vec2::new(0.83, 0.12), Vec2::new(0.5, 0.25)] {
                let at = Vec2::new(key.x as f32, key.y as f32) * size + spot * size;
                if (at - camera.truncate()).abs().max_element() >= super::super::distant::EASE_FROM * size {
                    continue;
                }
                let corner = |v: u32| crate::coords::bevy_to_game(Vec3::from_array(tile.positions[v as usize])) + tile.corner;
                let sheet = tile.indices.chunks(3).find_map(|t| {
                    let (a, b, c) = (corner(t[0]), corner(t[1]), corner(t[2]));
                    let area = (b - a).truncate().perp_dot((c - a).truncate());
                    let wb = (at - a.truncate()).perp_dot((c - a).truncate()) / area;
                    let wc = (b - a).truncate().perp_dot(at - a.truncate()) / area;
                    (wb >= -1e-4 && wc >= -1e-4 && wb + wc <= 1.0 + 1e-4).then_some(a.z + (b.z - a.z) * wb + (c.z - a.z) * wc)
                });
                let Some(sheet) = sheet else { continue };
                let found = state.height_at(finest as f32, camera.truncate(), at).expect("land between corners");
                assert!((found - sheet).abs() < 0.05, "at {at}: march {found}, sheet {sheet}");
                between += 1;
            }
        }
        assert!(between > 3, "checked {between} spots between corners");
        // Past every ring: nothing.
        assert_eq!(state.height_at(finest as f32, camera.truncate(), Vec2::new(1.0e6, 1.0e6)), None);
    }

    #[test]
    fn long_steps_find_the_same_land_as_small_ones() {
        // Hilly land round a camera, two rings in: rays every way, downward and level, each marched
        // with long steps (past tiles and blocks it's above, across cells exactly) and by a slow
        // search in quarter-block steps; both must meet the land in the same place, or both miss.
        let width = 64;
        let z: Vec<f32> = (0..width * width)
            .map(|i| {
                let (x, y) = ((i % width) as f32, (i / width) as f32);
                150.0 + 40.0 * (x * 0.4).sin() * (y * 0.3).cos() + ((i * 7) % 5) as f32 * 3.0
            })
            .collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let registry = crate::data::BlockRegistry::default();
        let blocks = TerrainBlocks::from_registry(&registry);
        let palette = super::super::distant::Palette::from_textures(&registry, &blocks);
        let finest = 16;
        let camera = Vec3::new(100.0, -150.0, 260.0);
        let mut state = MarchState::default();
        state.follow(1, 2, 16);
        for key in super::super::distant::wanted_tiles(camera, finest, 2) {
            state.store(key, super::super::distant::tile_mesh(&map, &blocks, &palette, key, finest).as_ref());
        }
        let near = |_: Vec2| false;
        let slow = |ray: Vec3| {
            let mut above = state.height_at(finest as f32, camera.truncate(), camera.truncate()).is_none_or(|h| camera.z >= h);
            let mut t = 0.0f32;
            while t < 9000.0 {
                let p = camera + ray * t;
                if ray.z >= 0.0 && p.z > state.highest {
                    return None;
                }
                let now = state.height_at(finest as f32, camera.truncate(), p.truncate()).is_none_or(|h| p.z >= h);
                if above && !now {
                    return Some(t);
                }
                above = now;
                t += 0.25;
            }
            None
        };
        // Where the long steps meet the land, the ray is on it.
        let on_land = |ray: Vec3, t: f32| {
            let p = camera + ray * t;
            state.height_at(finest as f32, camera.truncate(), p.truncate()).is_some_and(|h| (p.z - h).abs() < 0.01)
        };
        let (mut hits, mut misses) = (0, 0);
        for k in 0..72 {
            let heading = k as f32 * 0.37;
            for dip in [-0.6f32, -0.25, -0.08, -0.03, 0.0] {
                let ray = Vec3::new(heading.cos(), heading.sin(), dip).normalize();
                let fast = state.march_ray(finest as f32, camera, ray, 0.0, 2000, near);
                let careful = slow(ray);
                // The small steps can step over a ray grazing the land for a few hundredths of a
                // block (heading 0.74, dip -0.08 does, at 770.6); the long steps meet each triangle
                // exactly, so they may find land first, never later and never not at all.
                match (fast, careful) {
                    (Some(a), Some(b)) => {
                        assert!(on_land(ray, a) && a <= b + 0.3, "heading {heading}, dip {dip}: long steps {a}, small {b}");
                        hits += 1;
                    }
                    (Some(a), None) => assert!(on_land(ray, a), "heading {heading}, dip {dip}: long steps found land at {a}, not on it"),
                    (None, None) => misses += 1,
                    (None, Some(b)) => panic!("heading {heading}, dip {dip}: long steps missed the land small steps found at {b}"),
                }
            }
        }
        assert!(hits > 100 && misses > 5, "{hits} hits, {misses} misses");
    }

    #[test]
    fn no_ray_slips_through_the_land() {
        // Rays every way and steeply down from above hilly land, all inside the map: every one meets
        // the land. [Build 15: "a field of sky speckles", rays slipping through between a cell's two
        // triangles or across a cell's edge.]
        let width = 64;
        let z: Vec<f32> = (0..width * width)
            .map(|i| {
                let (x, y) = ((i % width) as f32, (i / width) as f32);
                150.0 + 40.0 * (x * 0.4).sin() * (y * 0.3).cos() + ((i * 7) % 5) as f32 * 3.0
            })
            .collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let registry = crate::data::BlockRegistry::default();
        let blocks = TerrainBlocks::from_registry(&registry);
        let palette = super::super::distant::Palette::from_textures(&registry, &blocks);
        let finest = 16;
        let camera = Vec3::new(100.0, -150.0, 300.0);
        let mut state = MarchState::default();
        state.follow(1, 2, 16);
        for key in super::super::distant::wanted_tiles(camera, finest, 2) {
            state.store(key, super::super::distant::tile_mesh(&map, &blocks, &palette, key, finest).as_ref());
        }
        let mut slipped = Vec::new();
        for k in 0..20_000 {
            let heading = k as f32 * 0.618_034 * std::f32::consts::TAU;
            let dip = -0.3 - 0.5 * ((k * 7919) % 1000) as f32 / 1000.0;
            let ray = Vec3::new(heading.cos(), heading.sin(), dip).normalize();
            if state.march_ray(finest as f32, camera, ray, 0.0, 2000, |_| false).is_none() {
                slipped.push((heading, dip));
            }
        }
        assert!(slipped.is_empty(), "{} of 20,000 rays slipped through, first {:?}", slipped.len(), slipped.first());
    }

    #[test]
    fn a_far_copy_is_taken_in_raises_the_land_s_highest_under_it_and_is_let_go() {
        let mut state = MarchState::default();
        state.follow(1, 1, 16);
        // A flat ring tile at 50 high, from (0, 0) to (256, 256).
        let tile = TileMesh { positions: vec![[0.0, 50.0, 0.0]], colours: vec![[0.5; 4]], sides: vec![[0.5; 4]], slopes: vec![[0.0; 4]], morph: vec![[50.0, 0.0, 0.0]], indices: Vec::new(), corner: Vec3::ZERO, grid: vec![0; SIDE * SIDE] };
        state.store(TileKey { ring: 0, x: 0, y: 0 }, Some(&tile));
        let slot = MarchState::slot(TileKey { ring: 0, x: 0, y: 0 });
        assert_eq!(state.tops[slot * TOPS + TILE_TOP], 50.0);
        // A copy of one chunk at (64, 64), 40 cells of a block from 40 high: its top is 80.
        let cells = Arc::new(CopyCells { corner: IVec3::new(64, 64, 40), scale: 1, tall: 40, cells: vec![0; 32 * 32 * 40] });
        let key = CopyKey { scale: 1, x: 2, y: 2 };
        state.follow_copies(vec![(key, cells.clone())], true);
        assert_eq!(state.copies.drawn.len(), 1);
        assert_eq!(state.copies.drawn[&key].pages.len(), 2);
        assert_eq!(state.copies.top, 80.0);
        // The blocks of cells under it (64 blocks each: the one from 64 to 128) and the tile rose.
        assert_eq!(state.tops[slot * TOPS + TILE_TOP], 80.0);
        assert_eq!(state.tops[slot * TOPS + 4 + 1], 80.0);
        assert_eq!(state.tops[slot * TOPS], 50.0);
        // A tile coming in again under it keeps the copy's height.
        state.store(TileKey { ring: 0, x: 0, y: 0 }, Some(&tile));
        assert_eq!(state.tops[slot * TOPS + 4 + 1], 80.0);
        // One far too tall is left to the land.
        let tall = Arc::new(CopyCells { corner: IVec3::ZERO, scale: 1, tall: 600, cells: vec![0; 32 * 32 * 600] });
        state.follow_copies(vec![(key, cells), (CopyKey { scale: 1, x: 0, y: 0 }, tall)], true);
        assert_eq!(state.copies.drawn.len(), 1);
        // Off: all let go, and their pages free again.
        state.follow_copies(Vec::new(), false);
        assert!(state.copies.drawn.is_empty());
        assert_eq!(state.copies.free_pages.len(), PAGES);
    }

    #[test]
    fn a_ray_never_reads_a_tile_that_isnt_in_yet() {
        let mut state = MarchState::default();
        state.follow(1, 1, 16);
        let key = TileKey { ring: 0, x: 3, y: -2 };
        // A tile 16 along shares its slot; until it's in, the old one's there and isn't read for it.
        let mut tile = TileMesh { positions: vec![[0.0, 50.0, 0.0]], colours: vec![[0.5; 4]], sides: vec![[0.5; 4]], slopes: vec![[0.0; 4]], morph: vec![[50.0, 0.0, 0.0]], indices: Vec::new(), corner: Vec3::ZERO, grid: vec![0; SIDE * SIDE] };
        state.store(key, Some(&tile));
        let size = 16.0 * CELLS as f32;
        let camera = Vec2::new(3.5, -1.5) * size;
        assert_eq!(state.height_at(16.0, camera, Vec2::new(3.5, -1.5) * size), Some(50.0));
        let moved = Vec2::new(19.5, -1.5) * size;
        assert_eq!(state.height_at(16.0, moved, Vec2::new(19.5, -1.5) * size), None);
        tile.positions[0][1] = 80.0;
        tile.morph[0][0] = 80.0;
        state.store(TileKey { ring: 0, x: 19, y: -2 }, Some(&tile));
        assert_eq!(state.height_at(16.0, moved, Vec2::new(19.5, -1.5) * size), Some(80.0));
    }
}

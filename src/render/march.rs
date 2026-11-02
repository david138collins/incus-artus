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
//! **Not yet:** caves, changes and objects (the far copies still draw them, as with the sheet), its
//! own line in "Graphics card time".

use super::distant::{CELLS, DistantMaterial, DistantSettings, DistantShaderSettings, DistantState, DistantTile, RING_TILES, TileKey, TileMesh};
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
    /// The distant terrain's `cleared` count this copy follows.
    cleared: u64,
    /// The highest land in it (a ray going up past it has missed).
    highest: f32,
    /// Started again: the lists go to the graphics card whole, once (made again if their size changed).
    whole: bool,
}

impl MarchState {
    /// Starts again, empty, when the sheet threw its tiles away or the number of rings changed.
    pub(super) fn follow(&mut self, cleared: u64, rings: u32) {
        if self.cleared == cleared && self.rings == rings && !self.slots.is_empty() {
            return;
        }
        self.cleared = cleared;
        self.rings = rings;
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
            self.queue(&self.tops_buffer, slot * TOPS * 4, bytes_of(&tops, f32::to_le_bytes));
        }
        // The slot after its corners, so a ray never reads it before they're there.
        self.slots[slot * 4..slot * 4 + 4].copy_from_slice(&[key.x, key.y, if tile.is_some() { 1 } else { 2 }, 0]);
        self.queue(&self.slots_buffer, slot * 16, bytes_of(&self.slots[slot * 4..slot * 4 + 4], i32::to_le_bytes));
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
    #[cfg(test)]
    fn march_ray(&self, finest: f32, origin: Vec3, ray: Vec3, start: f32, most: u32, drawn_near: impl Fn(Vec2) -> bool) -> Option<f32> {
        let camera = origin.truncate();
        let mut t = start;
        for _ in 0..most {
            let p = origin + ray * t;
            if ray.z >= 0.0 && p.z > self.highest {
                return None;
            }
            let place = match self.locate(finest, camera, p.truncate()) {
                Located::Beyond => return None,
                Located::NotIn => {
                    t += (t * 0.05).max(16.0);
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
                continue;
            }
            let block = (local / BLOCK as f32).floor().clamp(Vec2::ZERO, Vec2::splat((CELLS / BLOCK - 1) as f32));
            let out_of_block = leave(block * BLOCK as f32, (block + 1.0) * BLOCK as f32);
            if above_all(out_of_block, tops[(block.y as usize) * (CELLS / BLOCK) as usize + block.x as usize]) {
                t = out_of_block + nudge;
                continue;
            }
            let ij = local.floor().clamp(Vec2::ZERO, Vec2::splat((CELLS - 1) as f32));
            let out_of_cell = leave(ij, ij + 1.0);
            let (i, j) = (ij.x as usize, ij.y as usize);
            let heights = [(0, 0), (1, 0), (1, 1), (0, 1)].map(|(di, dj)| self.eased(place, i + di, j + dj, camera));
            if let [Some(a), Some(b), Some(c), Some(d)] = heights {
                let (u, v) = (local.x - ij.x, local.y - ij.y);
                // Each triangle's plane: height = h0 + gu u + gv v; and which half of the cell it covers.
                let mut hit = f32::MAX;
                for (h0, gu, gv, first_half) in [(a, b - a, c - b, true), (a, c - d, d - a, false)] {
                    let gap = p.z - (h0 + gu * u + gv * v);
                    let closing = ray.z - gu * rate.x - gv * rate.y;
                    if gap >= 0.0 && closing < 0.0 {
                        let at = t + gap / -closing;
                        let (ua, va) = (u + rate.x * (at - t), v + rate.y * (at - t));
                        if at <= out_of_cell && (ua >= va) == first_half {
                            hit = hit.min(at);
                        }
                    }
                }
                if hit < f32::MAX && !drawn_near((origin + ray * hit).truncate()) {
                    return Some(hit);
                }
                // Deep under the land: gone in under the near world.
                let under = if u >= v { a * (1.0 - u) + b * (u - v) + c * v } else { a * (1.0 - v) + c * u + d * (v - u) };
                if p.z < under - (2.0 * cell).max(16.0) {
                    return None;
                }
            }
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
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct MarchShaderSettings {
    /// x rings, y the finest cell (blocks), z the highest land, w the most steps a pixel takes.
    pub grid: Vec4,
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
    let material = materials.add(MarchMaterial {
        settings: DistantShaderSettings::default(),
        shown: images.add(super::light::placeholder()),
        light: images.add(super::light::placeholder()),
        march: MarchShaderSettings::default(),
        corners: corners.clone(),
        slots: slots.clone(),
        tops: tops.clone(),
    });
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
pub fn update_march(
    mut state: ResMut<MarchState>,
    distant: Res<DistantState>,
    settings: Res<DistantSettings>,
    mut changed_sheet: MessageReader<AssetEvent<DistantMaterial>>,
    sheets: Res<Assets<DistantMaterial>>,
    mut materials: ResMut<Assets<MarchMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let _timed = crate::render::Timed::new("march");
    let Some(handle) = state.material.clone() else { return };
    if state.whole && !state.slots.is_empty() {
        state.whole = false;
        if let Ok(mut writes) = state.writes.0.lock() {
            writes.clear();
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
    let sheet_changed = changed_sheet.read().any(|event| matches!(event, AssetEvent::Modified { .. }));
    let inner = distant.inner.map_or(Vec4::ZERO, |(centre, reach)| {
        let size = crate::world::CHUNK_SIZE as f32;
        Vec4::new((centre.x - reach) as f32 * size, (centre.y - reach) as f32 * size, (centre.x + reach + 1) as f32 * size, (centre.y + reach + 1) as f32 * size)
    });
    let march = MarchShaderSettings {
        grid: Vec4::new(state.rings as f32, settings.finest_cell as f32, state.highest, settings.march_most as f32),
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
/// (neither while the deep screenshot is being taken: it draws its own).
pub fn show_one_drawer(
    state: Res<MarchState>,
    settings: Res<DistantSettings>,
    shot: Res<super::deep_shot::DeepShot>,
    mut screens: Query<&mut Visibility, (With<MarchScreen>, Without<DistantTile>)>,
    mut tiles: Query<&mut Visibility, (With<DistantTile>, Without<MarchScreen>)>,
) {
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
        state.follow(1, 1);
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
        state.follow(1, 2);
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
    fn a_ray_never_reads_a_tile_that_isnt_in_yet() {
        let mut state = MarchState::default();
        state.follow(1, 1);
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

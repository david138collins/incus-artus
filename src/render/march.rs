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
//! isn't in yet: it reads the ring outside instead.
//!
//! **The shader** (`assets/shaders/march.wesl`) draws one triangle over the whole screen, and for
//! each pixel: starts the ray where it leaves the square of real chunks round you; steps along it,
//! further the higher above the land it is; where it goes under the land outside the real chunks
//! (as the sheet's `shown` texture marks them), it narrows in on the spot; there, it lights the land
//! with the sheet's own sums (`lighting.wesl`) and writes how far away it is, so the near world draws
//! in front. The land between corners is the sheet's two triangles a cell, eased into the next ring
//! out the same way, so the two draw the same shape.
//!
//! **Not yet:** caves, changes and objects (the far copies still draw them, as with the sheet), long
//! steps over plain land (part 4), its own line in "Graphics card time".

use super::distant::{CELLS, DistantMaterial, DistantSettings, DistantShaderSettings, DistantState, DistantTile, RING_TILES, TileKey, TileMesh};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;

/// Corners along a tile's side.
const SIDE: usize = (CELLS + 1) as usize;
/// A ring's slots: one for each of its tiles.
const SLOTS: usize = (RING_TILES * RING_TILES) as usize;
/// Numbers kept for each corner: height, the coarser ring's height, the slope north and east; the
/// top's colour and a 1 on land (0 on the sea); the side's colour and a 0.
const CORNER: usize = 12;
/// A corner past the map: no land there.
const MISSING: f32 = -1.0e30;
/// How often, at most, the lists go to the graphics card again while tiles come in (seconds): each
/// time the whole of it goes (about 25 MB with 7 rings).
const UPLOAD_EVERY: f32 = 0.1;

/// The march's copy of the far land, and what it's drawn with.
#[derive(Resource, Default)]
pub struct MarchState {
    material: Option<Handle<MarchMaterial>>,
    screen: Option<Entity>,
    corners_buffer: Option<Handle<ShaderBuffer>>,
    slots_buffer: Option<Handle<ShaderBuffer>>,
    /// Every ring's slots' corners, `CORNER` numbers each.
    corners: Vec<f32>,
    /// Every ring's slots: the tile's x and y, then 0 nothing yet, 1 land, 2 wholly past the map; and a 0.
    slots: Vec<i32>,
    rings: u32,
    /// The distant terrain's `cleared` count this copy follows.
    cleared: u64,
    /// The highest land in it (a ray going up past it has missed).
    highest: f32,
    changed: bool,
    uploaded_at: f32,
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
        self.slots = vec![0; rings as usize * SLOTS * 4];
        self.highest = f32::MIN;
        self.changed = true;
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
        self.slots[slot * 4..slot * 4 + 4].copy_from_slice(&[key.x, key.y, if tile.is_some() { 1 } else { 2 }, 0]);
        self.changed = true;
        let Some(tile) = tile else { return };
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
    }

    /// The land's height where a ray at `at` (game axes) would find it, with the camera at `camera`:
    /// the CPU's twin of the shader's `height_at`, for the tests. `None` where there's no land.
    #[cfg(test)]
    fn height_at(&self, finest: f32, camera: Vec2, at: Vec2) -> Option<f32> {
        for ring in 0..self.rings {
            let size = finest * CELLS as f32 * 2f32.powi(ring as i32);
            let tile = (at / size).floor().as_ivec2();
            let start = 2 * (camera / (2.0 * size)).floor().as_ivec2() - IVec2::splat(RING_TILES / 2);
            if tile.cmplt(start).any() || tile.cmpge(start + RING_TILES).any() {
                continue;
            }
            let slot = Self::slot(TileKey { ring: ring as u8, x: tile.x, y: tile.y });
            let info = &self.slots[slot * 4..slot * 4 + 4];
            if info[2] == 0 || info[0] != tile.x || info[1] != tile.y {
                continue;
            }
            if info[2] == 2 {
                return None;
            }
            let cell = size / CELLS as f32;
            let origin = tile.as_vec2() * size;
            let local = (at - origin) / cell;
            let (i, j) = ((local.x.floor() as usize).min(CELLS as usize - 1), (local.y.floor() as usize).min(CELLS as usize - 1));
            let (u, v) = (local.x - i as f32, local.y - j as f32);
            let ease_from = super::distant::EASE_FROM * size;
            let corner = |di: usize, dj: usize| {
                let at = (slot * SIDE * SIDE + (j + dj) * SIDE + i + di) * CORNER;
                let (height, target) = (self.corners[at], self.corners[at + 1]);
                let place = origin + Vec2::new((i + di) as f32, (j + dj) as f32) * cell;
                let away = (place - camera).abs().max_element();
                let eased = ((away - ease_from) / size.max(1.0)).clamp(0.0, 1.0);
                (height > MISSING / 2.0).then_some(height + (target - height) * eased)
            };
            let (a, c) = (corner(0, 0)?, corner(1, 1)?);
            return Some(if u >= v { a * (1.0 - u) + corner(1, 0)? * (u - v) + c * v } else { a * (1.0 - v) + c * u + corner(0, 1)? * (v - u) });
        }
        None
    }
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
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct MarchShaderSettings {
    /// x rings, y the finest cell (blocks), z the highest land, w the most steps a pixel takes.
    pub grid: Vec4,
    /// x a step's share of the height above the land, y the shortest step's share of the distance.
    pub steps: Vec4,
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
) {
    // Bound before any tile is in, so each list needs something in it.
    let corners = buffers.add(ShaderBuffer::new(vec![MISSING; CORNER], RenderAssetUsages::RENDER_WORLD));
    let slots = buffers.add(ShaderBuffer::new(vec![0i32; 4], RenderAssetUsages::RENDER_WORLD));
    let material = materials.add(MarchMaterial {
        settings: DistantShaderSettings::default(),
        shown: images.add(super::light::placeholder()),
        light: images.add(super::light::placeholder()),
        march: MarchShaderSettings::default(),
        corners: corners.clone(),
        slots: slots.clone(),
    });
    // One triangle past the screen's corners: the screen is wholly inside it.
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[-1.0f32, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]]);
    let screen = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden, NoFrustumCulling, MarchScreen)).id();
    state.material = Some(material);
    state.screen = Some(screen);
    state.corners_buffer = Some(corners);
    state.slots_buffer = Some(slots);
}

/// Sends the lists to the graphics card when they've changed (at most every `UPLOAD_EVERY`), and
/// keeps the material's settings in step with the sheet's and the march's own.
#[allow(clippy::too_many_arguments)]
pub fn update_march(
    mut state: ResMut<MarchState>,
    distant: Res<DistantState>,
    settings: Res<DistantSettings>,
    time: Res<Time<Real>>,
    mut changed_sheet: MessageReader<AssetEvent<DistantMaterial>>,
    sheets: Res<Assets<DistantMaterial>>,
    mut materials: ResMut<Assets<MarchMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let _timed = crate::render::Timed::new("march");
    let Some(handle) = state.material.clone() else { return };
    let now = time.elapsed_secs();
    if state.changed && settings.march && now - state.uploaded_at >= UPLOAD_EVERY && !state.slots.is_empty() {
        if let Some(buffer) = &state.corners_buffer {
            let _ = buffers.insert(buffer, ShaderBuffer::new(state.corners.clone(), RenderAssetUsages::RENDER_WORLD));
        }
        if let Some(buffer) = &state.slots_buffer {
            let _ = buffers.insert(buffer, ShaderBuffer::new(state.slots.clone(), RenderAssetUsages::RENDER_WORLD));
        }
        state.changed = false;
        state.uploaded_at = now;
    }
    let sheet_changed = changed_sheet.read().any(|event| matches!(event, AssetEvent::Modified { .. }));
    let inner = distant.inner.map_or(Vec4::ZERO, |(centre, reach)| {
        let size = crate::world::CHUNK_SIZE as f32;
        Vec4::new((centre.x - reach) as f32 * size, (centre.y - reach) as f32 * size, (centre.x + reach + 1) as f32 * size, (centre.y + reach + 1) as f32 * size)
    });
    let march = MarchShaderSettings {
        grid: Vec4::new(state.rings as f32, settings.finest_cell as f32, state.highest, settings.march_most as f32),
        steps: Vec4::new(settings.march_step, settings.march_shortest, 0.0, 0.0),
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

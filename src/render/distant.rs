//! Distant terrain — the landscape out to the edge of the world, beyond the full-detail chunks
//! (like Distant Horizons).
//!
//! **Rings of tiles.** Around you are rings, each made of blocks twice as big as the ring inside
//! it: 16-block cells just past the full-detail chunks, then 32, 64 … up to 1,024-block cells far
//! away. Seven rings reach the edge of the world from anywhere. Each ring is 8 × 8 tiles of 32 × 32
//! cells, with a hole in the middle where the next ring in sits (the finest ring has no hole: it
//! lies under the real chunks).
//!
//! **The hand-off.** The shader draws no distant terrain over a real chunk that's on screen. A
//! small texture marks those chunks, one pixel each, centred on the camera's chunk. Only chunks
//! actually meshed are marked, so flying faster than chunks load shows distant terrain ahead of
//! you, never a hole.
//!
//! **Each ring snaps to its own grid** (twice its tile size), so its hole lines up exactly with the
//! ring inside, and as you move only the tiles at its edges change; the rest stay put.
//!
//! **A tile** is a blocky height field: each cell is one column, its top at the ground's height in
//! the middle of the cell (the same column function the real chunks use), coloured by its surface
//! block (the block texture's average colour), with sides down to lower neighbours. Water fills to
//! sea level. Neighbouring cells on the same row with the same height and colour share one top.
//!
//! Tiles are made in the background, coarsest rings first (so the whole world appears quickly),
//! then nearest first.

use super::RenderSettings;
use super::light::LightSettings;
use crate::coords::{bevy_to_game, game_to_bevy};
use crate::data::{BlockRegistry, FACE_NAMES};
use super::meshing::MeshingState;
use crate::world::{BlockId, CHUNK_SIZE, ChunkPos, CompiledMap, Streaming, Terrain, TerrainBlocks, VoxelWorld};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Cells along a tile's side.
const CELLS: i32 = 32;
/// Tiles along a ring's side.
const RING_TILES: i32 = 8;
/// The side of the texture marking the chunks on screen, in chunks (centred on the camera's chunk:
/// 128 each way, far past where real chunks ever reach).
const SHOWN_SIZE: i32 = 256;

/// Tunables (on the "Render" debug page, saved in "everything" presets).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DistantSettings {
    pub enabled: bool,
    /// The finest ring's cell size, in blocks (each ring out doubles it).
    pub finest_cell: i32,
    /// How many rings.
    pub rings: u32,
    /// The most tiles being made at once (0 = one per CPU core).
    pub most_at_once: usize,
    /// Hill shading: each cell's top lit by its slope toward the sun, so hills and valleys read from
    /// afar (0 = flat colour). Measured per cell, so far rings (bigger cells) shade as strongly on
    /// screen as near ones.
    pub hill_shading: f32,
    /// Land a little lighter the higher it is (0 = none).
    pub height_tint: f32,
}

impl Default for DistantSettings {
    fn default() -> Self {
        Self { enabled: true, finest_cell: 16, rings: 7, most_at_once: 0, hill_shading: 0.3, height_tint: 0.15 }
    }
}

/// One tile: which ring (0 = finest), and where in that ring's grid of tiles (north, east).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub ring: u8,
    pub x: i32,
    pub y: i32,
}

/// The tiles wanted around a spot (game axes, blocks): every ring's 8 × 8 tiles, less the hole
/// where the ring inside sits.
pub fn wanted_tiles(at: Vec3, finest_cell: i32, rings: u32) -> Vec<TileKey> {
    let mut tiles = Vec::new();
    for ring in 0..rings {
        let size = (finest_cell * CELLS) as f32 * 2f32.powi(ring as i32);
        // Snapped to twice the tile size, so the hole lines up with the ring inside.
        let start = |v: f32| 2 * (v / (2.0 * size)).floor() as i32 - RING_TILES / 2;
        let (sx, sy) = (start(at.x), start(at.y));
        // The ring inside covers 4 × 4 of this ring's tiles, around the tile you're in.
        let hole = |v: f32| (v / size).floor() as i32 - 2;
        let (hx, hy) = (hole(at.x), hole(at.y));
        for x in sx..sx + RING_TILES {
            for y in sy..sy + RING_TILES {
                let in_hole = ring > 0 && (hx..hx + 4).contains(&x) && (hy..hy + 4).contains(&y);
                if !in_hole {
                    tiles.push(TileKey { ring: ring as u8, x, y });
                }
            }
        }
    }
    tiles
}

/// The colours distant terrain uses, from the block textures' average colours (linear RGB).
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub grass_top: [f32; 3],
    pub grass_side: [f32; 3],
    pub stone: [f32; 3],
    pub gabbro: [f32; 3],
    pub water: [f32; 3],
}

impl Palette {
    /// Averages each block's top and side textures from `assets/textures/blocks/`.
    pub fn from_textures(registry: &BlockRegistry, blocks: &TerrainBlocks) -> Self {
        let face = |id: BlockId, which: &str| {
            let index = FACE_NAMES.iter().position(|&name| name == which).unwrap_or(0);
            registry.def(id).map(|def| def.faces[index].clone()).unwrap_or_default()
        };
        let average = |texture: String| average_colour(&texture).unwrap_or([0.5, 0.5, 0.5]);
        Self {
            grass_top: average(face(blocks.grass, "top")),
            grass_side: average(face(blocks.grass, "north")),
            stone: average(face(blocks.stone, "top")),
            gabbro: average(face(blocks.gabbro, "top")),
            water: average(face(blocks.water, "top")),
        }
    }

    fn top(&self, block: BlockId, blocks: &TerrainBlocks) -> [f32; 3] {
        if block == blocks.grass {
            self.grass_top
        } else if block == blocks.gabbro {
            self.gabbro
        } else if block == blocks.water {
            self.water
        } else {
            self.stone
        }
    }

    fn side(&self, block: BlockId, blocks: &TerrainBlocks) -> [f32; 3] {
        if block == blocks.grass { self.grass_side } else { self.top(block, blocks) }
    }
}

/// The average colour of `assets/textures/blocks/<name>.png`, in linear RGB (as the shaders work).
fn average_colour(name: &str) -> Option<[f32; 3]> {
    let path = crate::data::assets_folder().join("textures").join("blocks").join(format!("{name}.png"));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).ok()?));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    let channels = info.color_type.samples();
    let (mut sum, mut count) = ([0.0f32; 3], 0.0);
    for pixel in buffer[..info.buffer_size()].chunks_exact(channels) {
        if channels == 4 && pixel[3] < 128 {
            continue;
        }
        for k in 0..3 {
            let v = pixel[k.min(channels - 1)] as f32 / 255.0;
            // sRGB to linear, so the colours mix like the textures do on the GPU.
            sum[k] += if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        }
        count += 1.0;
    }
    (count > 0.0).then(|| sum.map(|s| s / count))
}

/// A tile's mesh, in Bevy's axes, relative to the tile's corner.
pub struct TileMesh {
    pub positions: Vec<[f32; 3]>,
    pub colours: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    /// The tile's north-west-bottom corner... the corner it's placed at (game axes).
    pub corner: Vec3,
}

/// Face brightness: top, north/south sides, east/west sides; and the hill shading on tops (how
/// strong, the sun's direction in game axes, and the height tint).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    pub top: f32,
    pub north_south: f32,
    pub east_west: f32,
    pub hills: f32,
    pub sun: Vec3,
    pub height_tint: f32,
}

impl Shade {
    /// The face brightnesses from the render settings, the hill shading from the distant
    /// terrain's, and the sun from the light map's (one sun for everything).
    pub fn new(render: &RenderSettings, settings: &DistantSettings, light: &LightSettings) -> Self {
        Self {
            top: render.shade_top,
            north_south: render.shade_north_south,
            east_west: render.shade_east_west,
            hills: settings.hill_shading,
            sun: light.sun(),
            height_tint: settings.height_tint,
        }
    }

    /// How brightly a top is lit, with the ground rising `north` and `east` blocks per cell across
    /// it: 1 on flat ground, more facing the sun, less facing away.
    fn hill(&self, north: f32, east: f32) -> f32 {
        let normal = Vec3::new(-north * self.hills, -east * self.hills, 1.0).normalize();
        (normal.dot(self.sun) / self.sun.z).clamp(0.35, 1.6)
    }
}

impl TileMesh {
    /// Adds a quad with corners `a b c d` (game axes, going round), facing `normal` (game axes).
    fn quad(&mut self, corners: [Vec3; 4], normal: Vec3, colour: [f32; 3], shade: f32) {
        let start = self.positions.len() as u32;
        let points: Vec<Vec3> = corners.iter().map(|&p| game_to_bevy(p - self.corner)).collect();
        for p in &points {
            self.positions.push(p.to_array());
            self.colours.push([colour[0] * shade, colour[1] * shade, colour[2] * shade, 1.0]);
        }
        // Wound so the front faces `normal` (Bevy draws counter-clockwise fronts).
        let facing = (points[1] - points[0]).cross(points[2] - points[0]).dot(game_to_bevy(normal));
        let order: [u32; 6] = if facing > 0.0 { [0, 1, 2, 0, 2, 3] } else { [0, 2, 1, 0, 3, 2] };
        self.indices.extend(order.iter().map(|k| start + k));
    }
}

/// Makes one tile's mesh, or `None` if it's wholly past the map.
pub fn tile_mesh(map: &CompiledMap, blocks: &TerrainBlocks, palette: &Palette, shade: Shade, key: TileKey, finest_cell: i32) -> Option<TileMesh> {
    let cell = (finest_cell << key.ring) as f32;
    let size = cell * CELLS as f32;
    let corner = Vec3::new(key.x as f32 * size, key.y as f32 * size, 0.0);
    // Each cell's top (a Z, the top of the block) and surface block, with a one-cell border for
    // the sides. `None` = past the map.
    let n = (CELLS + 2) as usize;
    let mut tops: Vec<Option<(i32, BlockId)>> = vec![None; n * n];
    for j in 0..n {
        for i in 0..n {
            let x = corner.x + (i as f32 - 1.0 + 0.5) * cell;
            let y = corner.y + (j as f32 - 1.0 + 0.5) * cell;
            tops[j * n + i] = map.height(x, y).map(|h| {
                let ground = (h.round() as i32).clamp(1, crate::world::WORLD_HEIGHT - 1);
                if ground < crate::world::SEA_LEVEL { (crate::world::SEA_LEVEL, blocks.water) } else { (ground + 1, map.surface(ground, blocks)) }
            });
        }
    }
    let at = |i: i32, j: i32| tops[(j + 1) as usize * n + (i + 1) as usize];
    if (0..CELLS).all(|j| (0..CELLS).all(|i| at(i, j).is_none())) {
        return None;
    }
    let mut mesh = TileMesh { positions: Vec::new(), colours: Vec::new(), indices: Vec::new(), corner };
    let cell_corner = |i: i32, j: i32| corner + Vec3::new(i as f32 * cell, j as f32 * cell, 0.0);
    // Each top's brightness: hill shading from its neighbours' heights (the slope across it, in
    // blocks per cell), and the height tint; in 32 steps, so neighbours alike still share a quad.
    // Water stays flat.
    let lit = |i: i32, j: i32| -> i32 {
        let Some((h, block)) = at(i, j) else { return 32 };
        if block == blocks.water {
            return 32;
        }
        let z = |di: i32, dj: i32| at(i + di, j + dj).map_or(h, |(z, _)| z) as f32;
        let hill = shade.hill((z(1, 0) - z(-1, 0)) / 2.0, (z(0, 1) - z(0, -1)) / 2.0);
        let tint = 1.0 + shade.height_tint * (h - crate::world::SEA_LEVEL) as f32 / (crate::world::HIGHEST - crate::world::SEA_LEVEL as f32);
        (hill * tint * 32.0).round() as i32
    };
    for j in 0..CELLS {
        // Tops: runs along the row with the same height, block and brightness share one quad.
        let mut i = 0;
        while i < CELLS {
            let Some((h, block)) = at(i, j) else {
                i += 1;
                continue;
            };
            let light = lit(i, j);
            let mut end = i + 1;
            while end < CELLS && at(end, j) == Some((h, block)) && lit(end, j) == light {
                end += 1;
            }
            let (a, b) = (cell_corner(i, j), cell_corner(end, j + 1));
            let z = h as f32;
            mesh.quad(
                [Vec3::new(a.x, a.y, z), Vec3::new(b.x, a.y, z), Vec3::new(b.x, b.y, z), Vec3::new(a.x, b.y, z)],
                Vec3::Z,
                palette.top(block, blocks),
                shade.top * light as f32 / 32.0,
            );
            i = end;
        }
        // Sides, down to each lower neighbour.
        for i in 0..CELLS {
            let Some((h, block)) = at(i, j) else { continue };
            let colour = palette.side(block, blocks);
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let Some((low, _)) = at(i + di, j + dj) else { continue };
                if low >= h {
                    continue;
                }
                let (z0, z1) = (low as f32, h as f32);
                let (p, q) = match (di, dj) {
                    (1, 0) => (cell_corner(i + 1, j), cell_corner(i + 1, j + 1)),
                    (-1, 0) => (cell_corner(i, j), cell_corner(i, j + 1)),
                    (0, 1) => (cell_corner(i, j + 1), cell_corner(i + 1, j + 1)),
                    _ => (cell_corner(i, j), cell_corner(i + 1, j)),
                };
                let side_shade = if di != 0 { shade.north_south } else { shade.east_west };
                mesh.quad(
                    [Vec3::new(p.x, p.y, z0), Vec3::new(q.x, q.y, z0), Vec3::new(q.x, q.y, z1), Vec3::new(p.x, p.y, z1)],
                    Vec3::new(di as f32, dj as f32, 0.0),
                    colour,
                    side_shade,
                );
            }
        }
    }
    Some(mesh)
}

// ── The material ────────────────────────────────────────────────────────────

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct DistantMaterial {
    #[uniform(0)]
    pub settings: DistantShaderSettings,
    /// The chunks on screen (see the module notes).
    #[texture(1)]
    pub shown: Handle<Image>,
    /// How lit the ground is (see `light.rs`).
    #[texture(2)]
    #[sampler(3)]
    pub light: Handle<Image>,
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub struct DistantShaderSettings {
    pub fog_color: Vec4,
    /// x fog start, y fog end, z fog on.
    pub fog: Vec4,
    /// xy = the chunk the `shown` texture is centred on.
    pub shown_centre: Vec4,
    /// Survey view: x on, y every how many levels, z darkness, w sea level.
    pub survey: Vec4,
    /// The light map: x on, y shadows, z sky, w relief (strengths); and its area: x the map's half
    /// width (blocks), y the light map's width (blocks).
    pub light: Vec4,
    pub light_area: Vec4,
}

impl Material for DistantMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/distant.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/distant.wesl".into()
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
        let vertex_layout = layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0), Mesh::ATTRIBUTE_COLOR.at_shader_location(1)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

// ── Keeping the tiles up to date ────────────────────────────────────────────

/// A drawn tile.
#[derive(Component)]
pub struct DistantTile {
    pub quads: u32,
}

#[derive(Resource, Default)]
pub struct DistantState {
    pub(super) material: Option<Handle<DistantMaterial>>,
    /// The map the tiles were made from.
    map: Option<Arc<CompiledMap>>,
    palette: Option<Palette>,
    /// Tiles drawn (an entity), or known empty (`None`).
    tiles: HashMap<TileKey, Option<Entity>>,
    making: HashMap<TileKey, Task<Option<TileMesh>>>,
    /// Tiles no longer wanted, still drawn until the tiles taking their place are in.
    retiring: Vec<(TileKey, Entity)>,
    /// The settings and the tile the wanted list was worked out for.
    wanted_for: Option<(DistantSettings, Shade, IVec2)>,
    wanted: Vec<TileKey>,
    /// The texture marking chunks on screen, the chunks it marks, and the centre and render
    /// distance it was made for.
    shown_texture: Option<Handle<Image>>,
    shown: HashSet<ChunkPos>,
    shown_for: Option<(IVec2, i32)>,
    /// What the hand-off was last looked over for (see `update_shown_chunks`).
    looked_over: Option<(u64, u64, usize, IVec2, i32)>,
    // ── For the Performance page ──
    pub quads: u64,
}

impl DistantState {
    pub fn tiles_drawn(&self) -> usize {
        self.tiles.values().filter(|tile| tile.is_some()).count()
    }

    pub fn being_made(&self) -> usize {
        self.making.len()
    }

    fn clear(&mut self, commands: &mut Commands) {
        for (_, entity) in self.tiles.drain() {
            if let Some(entity) = entity {
                commands.entity(entity).despawn();
            }
        }
        for (_, entity) in self.retiring.drain(..) {
            commands.entity(entity).despawn();
        }
        self.making.clear();
        self.wanted_for = None;
        self.quads = 0;
    }
}

pub fn setup_distant(mut state: ResMut<DistantState>, mut materials: ResMut<Assets<DistantMaterial>>, mut images: ResMut<Assets<Image>>) {
    let side = SHOWN_SIZE as u32;
    let texture = Image::new_fill(
        Extent3d { width: side, height: side, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0],
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    let shown = images.add(texture);
    state.shown_texture = Some(shown.clone());
    let light = images.add(super::light::placeholder());
    state.material = Some(materials.add(DistantMaterial { settings: DistantShaderSettings::default(), shown, light }));
}

#[allow(clippy::too_many_arguments)]
pub fn update_distant(
    mut commands: Commands,
    terrain: Res<Terrain>,
    settings: Res<DistantSettings>,
    render: Res<RenderSettings>,
    light: Res<LightSettings>,
    registry: Res<BlockRegistry>,
    camera: Query<&Transform, With<Camera3d>>,
    mut state: ResMut<DistantState>,
    mut meshes: ResMut<Assets<Mesh>>,
    tiles: Query<&DistantTile>,
) {
    let map = match terrain.compiled() {
        Some(map) if settings.enabled => map.clone(),
        _ => {
            if !state.tiles.is_empty() || !state.making.is_empty() {
                state.clear(&mut commands);
            }
            return;
        }
    };
    let blocks = TerrainBlocks::from_registry(&registry);
    if state.map.as_ref().is_none_or(|old| !Arc::ptr_eq(old, &map)) {
        state.clear(&mut commands);
        state.map = Some(map.clone());
        state.palette = Some(Palette::from_textures(&registry, &blocks));
    }
    let Some(camera) = camera.iter().next() else { return };
    let at = bevy_to_game(camera.translation);
    let shade = Shade::new(&render, &settings, &light);

    // Which tiles are wanted: worked out again when you move into another finest tile (or the
    // settings change).
    let finest_tile = (settings.finest_cell * CELLS) as f32;
    let here = IVec2::new((at.x / finest_tile).floor() as i32, (at.y / finest_tile).floor() as i32);
    let look = shade;
    let now = (settings.clone(), look, here);
    if state.wanted_for.as_ref() != Some(&now) {
        // Shading changed: every tile has to be made again.
        if state.wanted_for.as_ref().is_some_and(|(_, old, _)| *old != look) {
            state.clear(&mut commands);
        }
        let wanted = wanted_tiles(at, settings.finest_cell, settings.rings);
        let keep: HashSet<TileKey> = wanted.iter().copied().collect();
        let gone: Vec<TileKey> = state.tiles.keys().copied().filter(|key| !keep.contains(key)).collect();
        for key in gone {
            if let Some(Some(entity)) = state.tiles.remove(&key) {
                state.retiring.push((key, entity));
            }
        }
        state.making.retain(|key, _| keep.contains(key));
        // Best last: coarsest rings first (the whole world shows quickly), then nearest.
        let mut wanted: Vec<TileKey> = wanted.into_iter().filter(|key| !state.tiles.contains_key(key)).collect();
        let distance = |key: &TileKey| {
            let size = finest_tile * 2f32.powi(key.ring as i32);
            let middle = Vec2::new((key.x as f32 + 0.5) * size, (key.y as f32 + 0.5) * size);
            middle.distance(Vec2::new(at.x, at.y))
        };
        wanted.sort_by(|a, b| a.ring.cmp(&b.ring).then(distance(b).total_cmp(&distance(a))));
        state.wanted = wanted;
        state.wanted_for = Some(now);
    }

    // Finished tiles become meshes.
    let mut done = Vec::new();
    state.making.retain(|&key, task| match check_ready(task) {
        Some(mesh) => {
            done.push((key, mesh));
            false
        }
        None => true,
    });
    let material = state.material.clone();
    for (key, mesh) in done {
        let Some(tile) = mesh else {
            state.tiles.insert(key, None);
            continue;
        };
        let quads = (tile.indices.len() / 6) as u32;
        let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tile.positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, tile.colours);
        bevy_mesh.insert_indices(Indices::U32(tile.indices));
        let Some(material) = material.clone() else { continue };
        let entity = commands
            .spawn((Mesh3d(meshes.add(bevy_mesh)), MeshMaterial3d(material), Transform::from_translation(game_to_bevy(tile.corner)), DistantTile { quads }))
            .id();
        state.quads += quads as u64;
        state.tiles.insert(key, Some(entity));
    }

    // Tiles no longer wanted stay until every tile taking their place is in, so moving never opens
    // a hole (sky showing through for a frame or two).
    if !state.retiring.is_empty() {
        let area = |key: &TileKey| {
            let size = finest_tile * 2f32.powi(key.ring as i32);
            (key.x as f32 * size, key.y as f32 * size, size)
        };
        let overlap = |a: (f32, f32, f32), b: (f32, f32, f32)| a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.2 && b.1 < a.1 + a.2;
        let waiting: Vec<(f32, f32, f32)> = state.wanted.iter().chain(state.making.keys()).map(area).collect();
        let mut done_with = Vec::new();
        state.retiring.retain(|(key, entity)| {
            let still_needed = waiting.iter().any(|&w| overlap(area(key), w));
            if !still_needed {
                done_with.push(*entity);
            }
            still_needed
        });
        for entity in done_with {
            if let Ok(tile) = tiles.get(entity) {
                state.quads = state.quads.saturating_sub(tile.quads as u64);
            }
            commands.entity(entity).despawn();
        }
    }

    // Start more.
    let most = if settings.most_at_once == 0 { std::thread::available_parallelism().map_or(4, |n| n.get()) } else { settings.most_at_once };
    let pool = AsyncComputeTaskPool::get();
    let palette = state.palette.unwrap_or_else(|| Palette::from_textures(&registry, &blocks));
    let finest = settings.finest_cell;
    while state.making.len() < most {
        let Some(key) = state.wanted.pop() else { break };
        if state.tiles.contains_key(&key) || state.making.contains_key(&key) {
            continue;
        }
        let map = map.clone();
        let task = pool.spawn(async move { tile_mesh(&map, &blocks, &palette, shade, key, finest) });
        state.making.insert(key, task);
    }
}

/// Keeps the distant terrain's fog and survey view the same as the near terrain's.
pub fn update_distant_material(
    render: Res<RenderSettings>,
    clear: Res<ClearColor>,
    state: Res<DistantState>,
    mut materials: ResMut<Assets<DistantMaterial>>,
) {
    let Some(handle) = &state.material else { return };
    if !render.is_changed() && !clear.is_changed() {
        return;
    }
    if let Some(mut material) = materials.get_mut(handle) {
        let fog = clear.0.to_linear();
        material.settings.fog_color = Vec4::new(fog.red, fog.green, fog.blue, 1.0);
        material.settings.fog = Vec4::new(render.fog_start, render.fog_end, if render.fog_enabled { 1.0 } else { 0.0 }, 0.0);
        material.settings.survey = render.survey_for_shader();
    }
}

/// Marks which real chunks are on screen, for the hand-off (see the module notes): chunks handed
/// to the renderer whose sections have all been meshed once, within the render distance. A chunk
/// being meshed again after an edit stays marked (its old mesh is still showing).
#[allow(clippy::too_many_arguments)]
pub fn update_shown_chunks(
    streaming: Res<Streaming>,
    world: Res<VoxelWorld>,
    meshing: Res<MeshingState>,
    render: Res<RenderSettings>,
    camera: Query<&Transform, With<Camera3d>>,
    mut state: ResMut<DistantState>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<DistantMaterial>>,
) {
    let (Some(material), Some(texture)) = (state.material.clone(), state.shown_texture.clone()) else { return };
    let Some(camera) = camera.iter().next() else { return };
    let at = bevy_to_game(camera.translation);
    let centre = IVec2::new((at.x / CHUNK_SIZE as f32).floor() as i32, (at.y / CHUNK_SIZE as f32).floor() as i32);
    // Nothing to look over again unless the meshing, the chunks handed over, or the view moved on.
    let looked = (meshing.changes, streaming.drawn_changes(), streaming.drawn().len(), centre, render.render_distance_chunks);
    if state.looked_over == Some(looked) {
        return;
    }
    state.looked_over = Some(looked);

    // Chunks with sections not meshed yet.
    let pending: HashSet<ChunkPos> =
        world.dirty_sections().chain(meshing.waiting.iter()).chain(meshing.running.keys()).map(|key| key.chunk).collect();
    let drawn = streaming.drawn();
    let before = state.shown.len();
    state.shown.retain(|pos| drawn.contains(pos));
    let mut changed = state.shown.len() != before;
    for &pos in drawn {
        if !pending.contains(&pos) && state.shown.insert(pos) {
            changed = true;
        }
    }
    let now = (centre, render.render_distance_chunks);
    if !changed && state.shown_for == Some(now) {
        return;
    }
    state.shown_for = Some(now);

    // A chunk is marked if it and all 8 chunks around it are on screen. So along the edge of the
    // real chunks, distant terrain is drawn under them too, one chunk deep: where the real ground
    // is a little lower than the distant ground beside it, no sliver of sky shows through the seam.
    let half = SHOWN_SIZE / 2;
    let limit = render.render_distance_chunks.min(half - 2);
    let on_screen = |pos: ChunkPos| {
        let offset = IVec2::new(pos.x, pos.y) - centre;
        // The same square the render distance hides sections outside of.
        offset.x.abs().max(offset.y.abs()) <= limit && state.shown.contains(&pos)
    };
    let mut pixels = vec![0u8; (SHOWN_SIZE * SHOWN_SIZE) as usize];
    for &pos in &state.shown {
        let around = (-1..=1).flat_map(|x| (-1..=1).map(move |y| ChunkPos::new(pos.x + x, pos.y + y)));
        if around.into_iter().all(on_screen) {
            let offset = IVec2::new(pos.x, pos.y) - centre;
            pixels[((offset.y + half) * SHOWN_SIZE + offset.x + half) as usize] = 255;
        }
    }
    if let Some(mut image) = images.get_mut(&texture) {
        image.data = Some(pixels);
    }
    // Touching the material also makes sure it picks up the new texture.
    if let Some(mut material) = materials.get_mut(&material) {
        material.settings.shown_centre = Vec4::new(centre.x as f32, centre.y as f32, 0.0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rings_fit_inside_each_other_with_no_gaps_or_overlaps() {
        // Cover a patch of the ground and check every spot is covered by exactly one ring's tile
        // (outside the finest ring's middle, which the full-detail chunks cover).
        for at in [Vec3::new(0.0, 0.0, 150.0), Vec3::new(1234.0, -5678.0, 150.0), Vec3::new(-20_000.0, 17_000.0, 300.0)] {
            let tiles = wanted_tiles(at, 16, 4);
            let cover = |x: f32, y: f32| {
                tiles
                    .iter()
                    .filter(|key| {
                        let size = 512.0 * 2f32.powi(key.ring as i32);
                        (key.x as f32 * size..(key.x + 1) as f32 * size).contains(&x) && (key.y as f32 * size..(key.y + 1) as f32 * size).contains(&y)
                    })
                    .count()
            };
            for dx in (-14_000..14_000).step_by(997) {
                for dy in (-14_000..14_000).step_by(1009) {
                    let (x, y) = (at.x + dx as f32, at.y + dy as f32);
                    let count = cover(x, y);
                    assert!(count <= 1, "{count} rings overlap at {x}, {y}");
                    // Within the finest ring's reach the spot must be covered.
                    if (dx as f32).abs() < 2_000.0 && (dy as f32).abs() < 2_000.0 {
                        assert_eq!(count, 1, "nothing covers {x}, {y}");
                    }
                }
            }
        }
    }

    #[test]
    fn slopes_facing_the_sun_are_lighter_and_flat_ground_is_as_it_was() {
        let light = LightSettings { sun_from: 315.0, sun_height: 40.0, ..LightSettings::default() };
        let shade = Shade::new(&RenderSettings::default(), &DistantSettings { hill_shading: 0.3, ..DistantSettings::default() }, &light);
        assert!((shade.hill(0.0, 0.0) - 1.0).abs() < 1e-5);
        // Ground falling away to the north-west faces the sun; rising toward it, faces away.
        let (facing, away) = (shade.hill(-4.0, 4.0), shade.hill(4.0, -4.0));
        assert!(facing > 1.05 && away < 0.95, "facing {facing}, away {away}");
    }

    #[test]
    fn a_tile_is_a_blocky_height_field_and_past_the_map_is_nothing() {
        let width = 64;
        // Sea in the west, land from 256 blocks east of the middle (inside the tile tested).
        let z: Vec<f32> = (0..width * width).map(|i| if i % width < 40 { 100.0 } else { 180.0 }).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let palette = Palette { grass_top: [0.0, 1.0, 0.0], grass_side: [0.5, 0.3, 0.1], stone: [0.5; 3], gabbro: [0.2; 3], water: [0.0, 0.0, 1.0] };
        let shade = Shade { top: 1.0, north_south: 0.8, east_west: 0.6, hills: 0.0, sun: Vec3::Z, height_tint: 0.0 };
        let mesh = tile_mesh(&map, &blocks, &palette, shade, TileKey { ring: 0, x: -1, y: 0 }, 16).unwrap();
        assert!(!mesh.indices.is_empty() && mesh.indices.len() % 6 == 0);
        assert_eq!(mesh.positions.len(), mesh.colours.len());
        // Water and grass both show.
        assert!(mesh.colours.iter().any(|c| c[2] > 0.9) && mesh.colours.iter().any(|c| c[1] > 0.9));
        // Far past the map: nothing.
        assert!(tile_mesh(&map, &blocks, &palette, shade, TileKey { ring: 0, x: 100, y: 100 }, 16).is_none());
    }
}

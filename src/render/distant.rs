//! Distant terrain — the landscape out to the edge of the world, beyond the full-detail chunks
//! (like Distant Horizons).
//!
//! **Rings of tiles.** Around you are rings, each with cells twice as big as the ring inside it:
//! 16-block cells just past the full-detail chunks, then 32, 64 … up to 1,024-block cells far away.
//! Seven rings reach the edge of the world from anywhere. Each ring is 16 × 16 tiles of 16 × 16
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
//! **A tile is a smooth sheet** [2026-10-28; it was a blocky height field, cells as columns with
//! sides, and far mountains read as ziggurats]: a corner every cell, at the ground's height there
//! (the same column recipe the real chunks use, unrounded), two triangles a cell, coloured by the
//! surface block (its texture's average colour; steep ground toward its sides' colour) and lit by
//! the slope round each corner. The sea is its surface.
//!
//! **Easing between rings** (as CDLOD does): each corner also carries the coarser ring's height at
//! that spot, and toward a ring's outer edge (5 to 6 of its tiles from you, the larger of
//! north-south and east-west) its corners slide there, in the shader, as you move. A ring's hole is
//! always nearer than 5 of its tiles and its outer edge never nearer than 6, so where two rings meet
//! the inner one is fully eased and the outer one not at all: they match exactly, with no crack, and
//! nothing pops as tiles change. (That room is why the rings are 16 tiles across.)
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
use bevy::mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexFormat};
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
pub const CELLS: i32 = 16;
/// Tiles along a ring's side.
const RING_TILES: i32 = 16;
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
    /// afar (0 = flat colour). The slope is measured per finest cell in every ring (as the near
    /// chunks' lighting measures it), so the rings shade alike and meet without a seam. [It was
    /// measured per cell, so each ring out shaded twice as strongly, and the rings' square edges
    /// showed as straight bands across the land (David's screenshot, 2026-10-27).]
    pub hill_shading: f32,
    /// How hill shading fades toward its darkest and brightest: low = a long, gentle fade, high =
    /// close to a hard stop (it used to stop dead, so strong shading went from shaded to fully dark
    /// over a short stretch). [David, 2026-10-27]
    pub shading_fade: f32,
    /// Land a little lighter the higher it is (0 = none).
    pub height_tint: f32,
}

impl Default for DistantSettings {
    fn default() -> Self {
        Self { enabled: true, finest_cell: 16, rings: 7, most_at_once: 0, hill_shading: 0.3, shading_fade: 2.0, height_tint: 0.15 }
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
    wanted_tiles_across(at, finest_cell, rings, RING_TILES)
}

/// The same with `across` tiles along each ring's side (a multiple of 4; the deep screenshot's
/// rings are wider, so each ring's cells stay about a pixel across all the way out).
pub fn wanted_tiles_across(at: Vec3, finest_cell: i32, rings: u32, across: i32) -> Vec<TileKey> {
    let mut tiles = Vec::new();
    let inner = across / 2;
    for ring in 0..rings {
        let size = (finest_cell * CELLS) as f32 * 2f32.powi(ring as i32);
        // Snapped to twice the tile size, so the hole lines up with the ring inside.
        let start = |v: f32| 2 * (v / (2.0 * size)).floor() as i32 - across / 2;
        let (sx, sy) = (start(at.x), start(at.y));
        // The ring inside covers half as many of this ring's tiles along a side, around the tile you're in.
        let hole = |v: f32| (v / size).floor() as i32 - across / 4;
        let (hx, hy) = (hole(at.x), hole(at.y));
        for x in sx..sx + across {
            for y in sy..sy + across {
                let in_hole = ring > 0 && (hx..hx + inner).contains(&x) && (hy..hy + inner).contains(&y);
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
    /// Per corner, for easing into the next ring out (see the module notes): the height the coarser
    /// ring's surface has here (game Z), and the distances (blocks, the larger of north-south and
    /// east-west from the camera) where easing starts and is complete.
    pub morph: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    /// The tile's north-west-bottom corner... the corner it's placed at (game axes).
    pub corner: Vec3,
}

/// The tops' brightness: the face brightness for tops, the hill shading (how strong, how it fades
/// toward its limits, the sun's direction in game axes) and the height tint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    pub top: f32,
    pub hills: f32,
    pub fade: f32,
    pub sun: Vec3,
    pub height_tint: f32,
}

impl Shade {
    /// The face brightness from the render settings, the hill shading from the distant terrain's,
    /// and the sun from the light map's (one sun for everything).
    pub fn new(render: &RenderSettings, settings: &DistantSettings, light: &LightSettings) -> Self {
        Self { top: render.shade_top, hills: settings.hill_shading, fade: settings.shading_fade, sun: light.sun(), height_tint: settings.height_tint }
    }

    /// How brightly the ground is lit, rising `north` and `east` blocks per finest cell across it
    /// (in any ring): 1 on flat ground, more facing the sun, less facing away.
    fn hill(&self, north: f32, east: f32) -> f32 {
        let normal = Vec3::new(-north * self.hills, -east * self.hills, 1.0).normalize();
        soft_limit(normal.dot(self.sun) / self.sun.z, self.fade)
    }
}

/// Hill shading's darkest and brightest (1 = as painted).
pub const DARKEST: f32 = 0.35;
pub const BRIGHTEST: f32 = 1.6;

/// Hill shading's light, `raw` (1 = flat ground), eased toward its darkest and brightest instead of
/// stopped dead: gentle slopes are lit as they are, steep ones closer and closer to the limit,
/// more gradually the lower `fade` is (at 8 and up, much like a hard stop). The terrain shader
/// does the same sums (assets/shaders/terrain.wesl, `soft_limit`), so near and far match.
pub fn soft_limit(raw: f32, fade: f32) -> f32 {
    let d = raw - 1.0;
    let limit = if d < 0.0 { 1.0 - DARKEST } else { BRIGHTEST - 1.0 };
    let x = d.abs() / limit;
    let p = fade.clamp(0.5, 16.0);
    1.0 + d.signum() * limit * x / (1.0 + x.powf(p)).powf(1.0 / p)
}

/// Each corner's easing (see `TileMesh::morph`): the coarser ring's height there, and where easing
/// starts and is complete.
pub const ATTRIBUTE_EASE: MeshVertexAttribute = MeshVertexAttribute::new("DistantEase", 0x1a5_e4e5, VertexFormat::Float32x3);

/// Where a ring's tiles ease into the next ring out, in its own tiles from the camera (the larger of
/// north-south and east-west): from here, complete there. Its hole is always nearer than the start
/// and its outer edge never nearer than the end (see `wanted_tiles_across`), so it meets the ring
/// inside it uneased and the ring outside it fully eased, exactly, with no crack between.
pub const EASE_FROM: f32 = (RING_TILES / 2 - 3) as f32;
pub const EASE_TO: f32 = (RING_TILES / 2 - 2) as f32;

/// Makes one tile's mesh, or `None` if it's wholly past the map.
pub fn tile_mesh(map: &CompiledMap, blocks: &TerrainBlocks, palette: &Palette, shade: Shade, key: TileKey, finest_cell: i32) -> Option<TileMesh> {
    tile_mesh_measured(map, blocks, palette, shade, key, finest_cell, finest_cell, true)
}

/// The same, with the hill shading's slopes measured over `slope_cell` blocks (the usual rings'
/// finest cell), so cells finer than that (the deep screenshot's) are shaded as the rings are; and
/// `eases`: whether it eases into the next ring out (the deep screenshot's tiles stand still).
///
/// **A tile is a smooth sheet** [2026-10-28, replacing the blocky one that made far mountains into
/// ziggurats]: a corner every cell, at the ground's height there (the same column recipe the real
/// chunks use, not rounded to blocks; the sea's surface at sea), two triangles a cell, coloured by
/// the surface block and lit by the slope from the corners round each one.
#[allow(clippy::too_many_arguments)]
pub fn tile_mesh_measured(map: &CompiledMap, blocks: &TerrainBlocks, palette: &Palette, shade: Shade, key: TileKey, finest_cell: i32, slope_cell: i32, eases: bool) -> Option<TileMesh> {
    let cell = (finest_cell << key.ring) as f32;
    let size = cell * CELLS as f32;
    let corner = Vec3::new(key.x as f32 * size, key.y as f32 * size, 0.0);
    let sea = crate::world::SEA_LEVEL as f32;
    // The ground at every corner, with a corner more round the edge for the slopes. `None` = past
    // the map.
    let n = (CELLS + 3) as usize;
    let ground: Vec<Option<f32>> = (0..n * n)
        .map(|k| {
            let (i, j) = ((k % n) as f32 - 1.0, (k / n) as f32 - 1.0);
            map.height(corner.x + i * cell, corner.y + j * cell)
        })
        .collect();
    let at = |i: i32, j: i32| ground[(j + 1) as usize * n + (i + 1) as usize];
    // The surface: the ground, or the sea's over it.
    let surface = |i: i32, j: i32| at(i, j).map(|h| h.max(sea));
    if (0..=CELLS).all(|j| (0..=CELLS).all(|i| at(i, j).is_none())) {
        return None;
    }
    let corners = (CELLS + 1) as usize;
    let mut mesh = TileMesh { positions: Vec::new(), colours: Vec::new(), morph: Vec::new(), indices: Vec::new(), corner };
    let slope = slope_cell as f32;
    let per_finest = slope / cell;
    let (ease_from, ease_to) = if eases { (EASE_FROM * size, EASE_TO * size) } else { (f32::MAX, f32::MAX) };
    // Each corner's place in the list (`u32::MAX` past the map).
    let mut index = vec![u32::MAX; corners * corners];
    for j in 0..=CELLS {
        for i in 0..=CELLS {
            let (Some(h), Some(top)) = (at(i, j), surface(i, j)) else { continue };
            let (x, y) = (corner.x + i as f32 * cell, corner.y + j as f32 * cell);
            let water = h < sea;
            let block = map.surface((h.round() as i32).clamp(1, crate::world::WORLD_HEIGHT - 1), blocks);
            let mut colour = if water { palette.water } else { palette.top(block, blocks) };
            let light = if water {
                shade.top
            } else {
                let (north, east) = if cell < slope {
                    // Finer than the slope is measured over: the ground's own height half that either side.
                    let z = |dx: f32, dy: f32| map.height(x + dx, y + dy).unwrap_or(h).max(sea);
                    (z(slope / 2.0, 0.0) - z(-slope / 2.0, 0.0), z(0.0, slope / 2.0) - z(0.0, -slope / 2.0))
                } else {
                    let z = |di: i32, dj: i32| surface(i + di, j + dj).unwrap_or(top);
                    ((z(1, 0) - z(-1, 0)) / 2.0 * per_finest, (z(0, 1) - z(0, -1)) / 2.0 * per_finest)
                };
                // Steep ground shows its sides' colour, as the blocks' sides do up close (grass's earth):
                // from half a block up per block across, all of it at more than one.
                let steep = ((north.hypot(east) / slope - 0.5) / 0.6).clamp(0.0, 1.0);
                let side = palette.side(block, blocks);
                colour = [0, 1, 2].map(|k| colour[k] + (side[k] - colour[k]) * steep);
                let tint = 1.0 + shade.height_tint * (top - sea) / (crate::world::HIGHEST - sea);
                shade.top * shade.hill(north, east) * tint
            };
            // The coarser ring's surface here: its corners are this ring's even ones, its triangles
            // split the same way, so between them it's the average of the two along the line.
            let coarse = |di: i32, dj: i32| surface(i + di, j + dj).unwrap_or(top);
            let target = match (i % 2 != 0, j % 2 != 0) {
                (false, false) => top,
                (true, false) => (coarse(-1, 0) + coarse(1, 0)) / 2.0,
                (false, true) => (coarse(0, -1) + coarse(0, 1)) / 2.0,
                (true, true) => (coarse(-1, -1) + coarse(1, 1)) / 2.0,
            };
            index[j as usize * corners + i as usize] = mesh.positions.len() as u32;
            mesh.positions.push(game_to_bevy(Vec3::new(x, y, top) - corner).to_array());
            mesh.colours.push([colour[0] * light, colour[1] * light, colour[2] * light, 1.0]);
            mesh.morph.push([target, ease_from, ease_to]);
        }
    }
    // Two triangles a cell, split from its north-west-most corner to the opposite one (as every
    // ring splits them, so a coarser ring's surface is known between its corners), facing up.
    for j in 0..CELLS as usize {
        for i in 0..CELLS as usize {
            let k = |di: usize, dj: usize| index[(j + dj) * corners + i + di];
            let (a, b, c, d) = (k(0, 0), k(1, 0), k(1, 1), k(0, 1));
            for [p, q, r] in [[a, b, c], [a, c, d]] {
                if p == u32::MAX || q == u32::MAX || r == u32::MAX {
                    continue;
                }
                let at = |v: u32| Vec3::from_array(mesh.positions[v as usize]);
                let up = (at(q) - at(p)).cross(at(r) - at(p)).y;
                mesh.indices.extend(if up > 0.0 { [p, q, r] } else { [p, r, q] });
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
        let vertex_layout = layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0), Mesh::ATTRIBUTE_COLOR.at_shader_location(1), ATTRIBUTE_EASE.at_shader_location(2)])?;
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
    let _timed = crate::render::Timed::new("distant tiles");
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
        bevy_mesh.insert_attribute(ATTRIBUTE_EASE, tile.morph);
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
    let _timed = crate::render::Timed::new("mark chunks shown");
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
        for (at, across) in [(Vec3::new(0.0, 0.0, 150.0), 8), (Vec3::new(1234.0, -5678.0, 150.0), 8), (Vec3::new(-20_000.0, 17_000.0, 300.0), 8), (Vec3::new(1234.0, -5678.0, 150.0), 16)] {
            let tiles = wanted_tiles_across(at, 16, 4, across);
            let cover = |x: f32, y: f32| {
                tiles
                    .iter()
                    .filter(|key| {
                        let size = (16 * CELLS) as f32 * 2f32.powi(key.ring as i32);
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
    fn shading_eases_toward_its_limits_more_gently_the_lower_the_fade() {
        for fade in [0.5, 2.0, 8.0] {
            // Flat ground as painted, gentle slopes almost as they are, and never past the limits.
            assert!((soft_limit(1.0, fade) - 1.0).abs() < 1e-6);
            assert!((soft_limit(0.95, fade) - 0.95).abs() < 0.02 || fade < 1.0);
            let mut last = f32::MIN;
            for k in 0..=100 {
                let raw = k as f32 * 0.03;
                let lit = soft_limit(raw, fade);
                assert!(lit > DARKEST - 1e-4 && lit < BRIGHTEST + 1e-4 && lit >= last, "fade {fade}, raw {raw}: {lit}");
                last = lit;
            }
        }
        // 90% of the way to the darkest, a low fade is still well short of it; a high one is about there.
        let raw = 1.0 - (1.0 - DARKEST) * 0.9;
        assert!(soft_limit(raw, 0.5) > soft_limit(raw, 8.0) + 0.1);
        assert!((soft_limit(raw, 8.0) - raw).abs() < 0.03);
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
    fn the_same_slope_is_lit_the_same_in_every_ring() {
        // An even, gentle slope rising east from the middle (2 blocks per 32-block pixel), all grass, under
        // Z 333 to the map's edge.
        let width: usize = 64;
        let z: Vec<f32> = (0..width * width).map(|i| 130.0 + (i % width).saturating_sub(32) as f32 * 2.0).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let palette = Palette { grass_top: [0.0, 1.0, 0.0], grass_side: [0.5, 0.3, 0.1], stone: [0.5; 3], gabbro: [0.2; 3], water: [0.0, 0.0, 1.0] };
        let light = LightSettings { sun_from: 270.0, sun_height: 40.0, ..LightSettings::default() };
        let shade = Shade::new(&RenderSettings::default(), &DistantSettings { hill_shading: 0.3, height_tint: 0.0, ..DistantSettings::default() }, &light);
        // The tops' brightness most of a tile shows (grass tops are pure green).
        let lit = |ring: u8| {
            let mesh = tile_mesh(&map, &blocks, &palette, shade, TileKey { ring, x: 0, y: 0 }, 16).unwrap();
            let mut counts: HashMap<u32, usize> = HashMap::new();
            for c in mesh.colours.iter().filter(|c| c[0] == 0.0 && c[2] == 0.0 && c[1] > 0.0) {
                *counts.entry((c[1] * 1000.0).round() as u32).or_default() += 1;
            }
            counts.into_iter().max_by_key(|&(_, n)| n).unwrap().0
        };
        let (near, further) = (lit(0), lit(1));
        assert_ne!(near, 1000, "the slope isn't shaded at all");
        // Alike (measured per cell, ring 1 would be shaded about twice as strongly)...
        assert!(near.abs_diff(further) <= 5, "ring 0 lit {near}, ring 1 lit {further} (per mille)");
    }

    #[test]
    fn a_tile_is_a_smooth_sheet_on_the_ground_and_past_the_map_is_nothing() {
        let width = 64;
        // Sea in the west, land from 256 blocks east of the middle (inside the tiles tested), rising.
        let z: Vec<f32> = (0..width * width).map(|i| if i % width < 40 { 100.0 } else { 160.0 + (i % width) as f32 }).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let palette = Palette { grass_top: [0.0, 1.0, 0.0], grass_side: [0.5, 0.3, 0.1], stone: [0.5; 3], gabbro: [0.2; 3], water: [0.0, 0.0, 1.0] };
        let shade = Shade { top: 1.0, hills: 0.0, fade: 2.0, sun: Vec3::Z, height_tint: 0.0 };
        let key = TileKey { ring: 1, x: 0, y: 0 };
        let mesh = tile_mesh(&map, &blocks, &palette, shade, key, 16).unwrap();
        // A corner every cell, two triangles a cell, and the easing for each corner.
        assert_eq!(mesh.positions.len(), ((CELLS + 1) * (CELLS + 1)) as usize);
        assert_eq!(mesh.indices.len(), (CELLS * CELLS * 6) as usize);
        assert_eq!(mesh.morph.len(), mesh.positions.len());
        // Every corner on the ground (or the sea's surface), not rounded to blocks.
        let sea = crate::world::SEA_LEVEL as f32;
        for (p, morph) in mesh.positions.iter().zip(&mesh.morph) {
            let game = bevy_to_game(Vec3::from_array(*p)) + mesh.corner;
            let ground = map.height(game.x, game.y).unwrap().max(sea);
            assert!((game.z - ground).abs() < 1e-3, "corner at {game}, ground {ground}");
            // It eases between 5 and 6 of its tiles out (a ring-1 tile is 512 blocks).
            assert_eq!((morph[1], morph[2]), (EASE_FROM * 512.0, EASE_TO * 512.0));
        }
        // Every triangle faces up (Bevy's y).
        for t in mesh.indices.chunks_exact(3) {
            let at = |k: usize| Vec3::from_array(mesh.positions[t[k] as usize]);
            assert!((at(1) - at(0)).cross(at(2) - at(0)).y > 0.0);
        }
        // An odd corner eases to the average of its even neighbours (the coarser ring's surface).
        let corners = (CELLS + 1) as usize;
        let z_of = |i: usize, j: usize| bevy_to_game(Vec3::from_array(mesh.positions[j * corners + i])).z;
        let odd = mesh.morph[2 * corners + 3][0];
        assert!((odd - (z_of(2, 2) + z_of(4, 2)) / 2.0).abs() < 1e-3);
        assert_eq!(mesh.morph[2 * corners + 2][0], z_of(2, 2));
        // Water and grass both show, near the coast.
        let coast = tile_mesh(&map, &blocks, &palette, shade, TileKey { ring: 1, x: -1, y: 0 }, 16).unwrap();
        assert!(coast.colours.iter().any(|c| c[2] > 0.9) && coast.colours.iter().any(|c| c[1] > 0.9));
        // Far past the map: nothing.
        assert!(tile_mesh(&map, &blocks, &palette, shade, TileKey { ring: 0, x: 100, y: 100 }, 16).is_none());
    }

    #[test]
    fn every_ring_eases_fully_before_its_outer_edge_and_not_at_all_at_its_hole() {
        // Wherever you stand, a ring's hole is nearer than where it starts easing and its outer edge
        // no nearer than where it's eased completely (in its own tiles, the larger of the two ways).
        for k in 0..400 {
            let at = Vec3::new(k as f32 * 37.3 - 7_000.0, k as f32 * -91.7 + 3_000.0, 200.0);
            for ring in 1..5u8 {
                let size = (16 * CELLS) as f32 * 2f32.powi(ring as i32);
                let tiles: Vec<TileKey> = wanted_tiles_across(at, 16, 5, RING_TILES).into_iter().filter(|t| t.ring == ring).collect();
                let away = |x: f32, y: f32| (x - at.x).abs().max((y - at.y).abs()) / size;
                let (mut nearest_edge, mut farthest_hole) = (f32::MAX, 0.0f32);
                for t in &tiles {
                    let (x0, y0) = (t.x as f32 * size, t.y as f32 * size);
                    for (x, y) in [(x0, y0), (x0 + size, y0), (x0, y0 + size), (x0 + size, y0 + size)] {
                        let d = away(x, y);
                        // A tile corner with no tile of this ring beyond it is on the outer edge; one
                        // with no tile toward you, on the hole.
                        let has = |cx: i32, cy: i32| tiles.iter().any(|o| o.x == cx && o.y == cy);
                        let (cx, cy) = ((x / size).round() as i32, (y / size).round() as i32);
                        let all_round = has(cx, cy) && has(cx - 1, cy) && has(cx, cy - 1) && has(cx - 1, cy - 1);
                        if !all_round {
                            if d > (EASE_FROM + EASE_TO) / 2.0 {
                                nearest_edge = nearest_edge.min(d);
                            } else {
                                farthest_hole = farthest_hole.max(d);
                            }
                        }
                    }
                }
                assert!(farthest_hole <= EASE_FROM + 1e-3, "ring {ring} at {at}: hole as far as {farthest_hole}");
                assert!(nearest_edge >= EASE_TO - 1e-3, "ring {ring} at {at}: outer edge as near as {nearest_edge}");
            }
        }
    }
}

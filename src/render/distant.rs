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
use crate::world::{BlockId, CHUNK_SIZE, ChunkPos, CompiledMap, Streaming, Terrain, TerrainBlocks, VoxelWorld, under_sea};
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
pub(super) const RING_TILES: i32 = 16;
/// The side of the texture marking the chunks on screen, in chunks (centred on the camera's chunk:
/// 128 each way, far past where real chunks ever reach).
pub(super) const SHOWN_SIZE: i32 = 256;
/// How a chunk the far copies draw is marked in that texture (a real chunk is 255; both are more
/// than half, which is all the sheet asks).
const COPY_MARK: u8 = 160;

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
    /// How much the far slopes are lit as the near ground's block steps light them on average (their
    /// feet's shade and their sides), so the two match where they meet (1 = as worked out, 0 = not at
    /// all: smooth slopes, brighter). [2026-10-28]
    pub slope_match: f32,
    /// Drawn by the march instead of the sheet (`march.rs`, `docs/far-view.md`): each pixel's ray finds
    /// the same land the sheet is made of. [David, 2026-11-02]
    pub march: bool,
    /// The most steps a pixel's ray takes (each past a tile, a block of cells, or one cell).
    pub march_most: u32,
}

impl Default for DistantSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            finest_cell: 16,
            rings: 7,
            most_at_once: 0,
            hill_shading: 0.3,
            shading_fade: 2.0,
            height_tint: 0.15,
            slope_match: 1.0,
            march: true,
            march_most: 200,
        }
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

/// The colours distant terrain uses (linear RGB): every block's top and side, as the near ground
/// shows them once a block is under a pixel (`textures::far_colour`: the textures' own smallest
/// mipmaps), so near and far agree, whichever blocks mineralogy puts down.
#[derive(Clone, Debug)]
pub struct Palette {
    /// By block number: its top's colour and its sides'.
    pub blocks: Vec<([f32; 3], [f32; 3])>,
    pub water: [f32; 3],
}

impl Palette {
    pub fn from_textures(registry: &BlockRegistry, blocks: &TerrainBlocks) -> Self {
        let average = |texture: &str| super::textures::far_colour(texture).unwrap_or([0.5, 0.5, 0.5]);
        let face = |name: &str| FACE_NAMES.iter().position(|&face| face == name).unwrap_or(0);
        let colours = registry.defs().iter().map(|def| (average(&def.faces[face("top")]), average(&def.faces[face("north")]))).collect();
        let water = registry.def(blocks.water).map_or([0.5; 3], |def| average(&def.faces[face("top")]));
        Self { blocks: colours, water }
    }

    pub fn top(&self, block: BlockId) -> [f32; 3] {
        self.blocks.get(block.0 as usize).map_or([0.5; 3], |colours| colours.0)
    }

    pub fn side(&self, block: BlockId) -> [f32; 3] {
        self.blocks.get(block.0 as usize).map_or([0.5; 3], |colours| colours.1)
    }
}

/// A tile's mesh, in Bevy's axes, relative to the tile's corner.
///
/// It carries only what's true of the ground [2026-10-29, Counsel's seam plan]: the surface block's
/// colours and the slope. The shader lights it every frame with the same sums the near ground uses
/// (assets/shaders/lighting.wesl), so changing the light never makes a tile again, and near and far
/// can't drift apart.
pub struct TileMesh {
    pub positions: Vec<[f32; 3]>,
    /// The tops' colour (rgb; alpha 1 on land, 0 for the sea's surface, which isn't lit by slope).
    pub colours: Vec<[f32; 4]>,
    /// The steps' sides' colour (rgb).
    pub sides: Vec<[f32; 4]>,
    /// The slope: blocks up per block across, north (x) and east (y).
    pub slopes: Vec<[f32; 4]>,
    /// Per corner, for easing into the next ring out (see the module notes): the height the coarser
    /// ring's surface has here (game Z), and the distances (blocks, the larger of north-south and
    /// east-west from the camera) where easing starts and is complete.
    pub morph: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    /// The tile's north-west-bottom corner... the corner it's placed at (game axes).
    pub corner: Vec3,
    /// Each corner's place in the lists above: corner `i` cells north and `j` east of the tile's
    /// corner at `grid[j * (CELLS + 1) + i]`; `u32::MAX` past the map. The march reads them by it.
    pub grid: Vec<u32>,
}

/// How the far terrain is lit, from the same settings as the near ground (handed to its shader:
/// `Shade::uniforms`): the face brightness for tops and sides, the hill shading (how strong, how it
/// fades toward its limits, the sun's direction in game axes), the height tint, and what the near
/// ground's block steps do to a slope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    pub top: f32,
    pub north_south: f32,
    pub east_west: f32,
    pub hills: f32,
    pub fade: f32,
    pub sun: Vec3,
    pub height_tint: f32,
    /// The steps' feet on a slope, averaged across a step (1 = no shade), for slopes from flat to
    /// `STEEPEST_STEPS` blocks up per block across: all round (`foot`), and from the sun's side
    /// (`step`); and how much the far slopes take it on (`slope_match`, 0 = none, 1 = all).
    pub foot: [f32; SLOPE_STEPS + 1],
    pub step: [f32; SLOPE_STEPS + 1],
    pub slope_match: f32,
    /// How much of the slope's sun and shade the steps' sides take (as the near shader's `sides`).
    pub sides: f32,
    /// Light only (the seam probe): every colour plain grey, the lighting kept.
    pub grey: bool,
}

/// The steepest slope the steps' shade is worked out for (blocks up per block across), and in how many
/// steps from flat (the shader reads the tables at these, distant.wesl).
const STEEPEST_STEPS: f32 = 4.0;
const SLOPE_STEPS: usize = 32;
/// How far apart the spots a far corner's colours are averaged over (blocks; `tile_mesh_measured`).
const COLOUR_SPACING: f32 = 32.0;

/// How dark the foot of block steps is on average, across one step, on ground rising `rise` blocks
/// per block (so a step every 1/rise blocks), with the near shader's sums: each step's top casts a
/// cone falling a block every `reach` blocks, a spot is `below` the highest cone over it (counting
/// every step up the slope, never more than `cap` blocks: a step's shade reaches only so far), eased
/// toward the darkest by `fade`, and darkened by `strength` of that
/// (assets/shaders/lighting.wesl, `eased`, and the feet in terrain.wesl). 1 = no shade.
fn steps_shade(rise: f32, strength: f32, reach: f32, cap: f32, fade: f32) -> f32 {
    if rise <= 1e-3 || strength <= 0.0 {
        return 1.0;
    }
    let (width, reach) = (1.0 / rise, reach.max(0.1));
    let eased = |below: f32| {
        if below <= 0.0 {
            return 0.0;
        }
        let p = fade.clamp(0.5, 16.0);
        below / (1.0 + below.powf(p)).powf(1.0 / p)
    };
    const ACROSS: usize = 16;
    let mut total = 0.0;
    for s in 0..ACROSS {
        // A spot this far from the foot of the step above it.
        let x = (s as f32 + 0.5) / ACROSS as f32 * width;
        let below = (1..=64).map(|k| k as f32 - (x + (k - 1) as f32 * width) / reach).fold(0.0f32, f32::max).min(cap);
        total += 1.0 - strength * eased(below);
    }
    total / ACROSS as f32
}

impl Shade {
    /// The face brightness from the render settings, the hill shading from the distant terrain's,
    /// the sun and the steps' shade from the light map's (one sun for everything).
    pub fn new(render: &RenderSettings, settings: &DistantSettings, light: &LightSettings) -> Self {
        let fade = settings.shading_fade;
        let table = |strength: f32| std::array::from_fn(|k| steps_shade(k as f32 / SLOPE_STEPS as f32 * STEEPEST_STEPS, strength, light.shade_reach, light.step_reach / light.shade_reach.max(0.1), fade));
        Self {
            top: render.shade_top,
            north_south: render.shade_north_south,
            east_west: render.shade_east_west,
            hills: settings.hill_shading,
            fade,
            sun: light.sun(),
            height_tint: settings.height_tint,
            foot: table(if light.near { light.foot_shading } else { 0.0 }),
            step: table(if light.near { light.step_shadows } else { 0.0 }),
            slope_match: settings.slope_match,
            sides: if light.near { light.side_shading } else { 0.0 },
            grey: render.light_only,
        }
    }

    /// Into the far terrain shader's settings (`finest_cell`: the distance the hill shading's slope is
    /// measured over, as the near ground's; see distant.wesl).
    fn uniforms(&self, settings: &mut DistantShaderSettings, finest_cell: f32) {
        let sea = crate::world::SEA_LEVEL as f32;
        settings.sun = self.sun.extend(self.hills);
        settings.faces = Vec4::new(self.top, self.north_south, self.east_west, self.sides);
        settings.hill = Vec4::new(self.height_tint, finest_cell, sea, crate::world::HIGHEST);
        settings.shade = Vec4::new(self.fade, self.slope_match, if self.grey { 1.0 } else { 0.0 }, STEEPEST_STEPS);
        (settings.foot, settings.step) = self.tables();
    }

    /// The steps' feet's shade tables, packed four to a `Vec4` for the shaders.
    pub fn tables(&self) -> ([Vec4; TABLE_VEC4S], [Vec4; TABLE_VEC4S]) {
        let pack = |table: &[f32; SLOPE_STEPS + 1]| std::array::from_fn(|k| Vec4::from_array(std::array::from_fn(|j| table.get(k * 4 + j).copied().unwrap_or(1.0))));
        (pack(&self.foot), pack(&self.step))
    }

    /// For the near shader's far copies (`TerrainLight::far`): how much the far slopes take on the
    /// steps' shade, and the steepest slope the tables reach.
    pub fn far(&self) -> Vec4 {
        Vec4::new(self.slope_match, STEEPEST_STEPS, 0.0, 0.0)
    }

    /// How brightly the ground is lit, rising `north` and `east` blocks per finest cell across it
    /// (in any ring): 1 on flat ground, more facing the sun, less facing away (lighting.wesl,
    /// `hill_light`: the same sums, for the World Compiler's viewer and the tests).
    pub fn hill(&self, north: f32, east: f32) -> f32 {
        let normal = Vec3::new(-north * self.hills, -east * self.hills, 1.0).normalize();
        soft_limit(normal.dot(self.sun) / self.sun.z, self.fade)
    }
}

/// The steps' tables packed four to a `Vec4` for the shader (33 values, so 9).
pub(super) const TABLE_VEC4S: usize = SLOPE_STEPS.div_ceil(4) + 1;

/// Hill shading's darkest and brightest (1 = as painted), as lighting.wesl has them.
pub const DARKEST: f32 = 0.35;
pub const BRIGHTEST: f32 = 1.6;

/// Hill shading's light, `raw` (1 = flat ground), eased toward its darkest and brightest instead of
/// stopped dead: gentle slopes are lit as they are, steep ones closer and closer to the limit,
/// more gradually the lower `fade` is (at 8 and up, much like a hard stop). The shaders do the same
/// sums (assets/shaders/lighting.wesl, `soft_limit`).
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

/// Each corner's steps' sides' colour, and its slope (see `TileMesh::sides` and `slopes`).
pub const ATTRIBUTE_SIDE: MeshVertexAttribute = MeshVertexAttribute::new("DistantSide", 0x1a5_e4e6, VertexFormat::Float32x4);
pub const ATTRIBUTE_SLOPE: MeshVertexAttribute = MeshVertexAttribute::new("DistantSlope", 0x1a5_e4e7, VertexFormat::Float32x4);

/// Where a ring's tiles ease into the next ring out, in its own tiles from the camera (the larger of
/// north-south and east-west): from here, complete there. Its hole is always nearer than the start
/// and its outer edge never nearer than the end (see `wanted_tiles_across`), so it meets the ring
/// inside it uneased and the ring outside it fully eased, exactly, with no crack between.
pub const EASE_FROM: f32 = (RING_TILES / 2 - 3) as f32;
pub const EASE_TO: f32 = (RING_TILES / 2 - 2) as f32;

/// Makes one tile's mesh, or `None` if it's wholly past the map.
pub fn tile_mesh(map: &CompiledMap, blocks: &TerrainBlocks, palette: &Palette, key: TileKey, finest_cell: i32) -> Option<TileMesh> {
    tile_mesh_measured(map, blocks, palette, key, finest_cell, finest_cell, true)
}

/// The same, with the slopes measured over `slope_cell` blocks (the usual rings' finest cell), so
/// cells finer than that (the deep screenshot's) are shaded as the rings are; and `eases`: whether it
/// eases into the next ring out (the deep screenshot's tiles stand still).
///
/// **A tile is a smooth sheet** [2026-10-28, replacing the blocky one that made far mountains into
/// ziggurats]: a corner every cell, at the ground's height there (the same column recipe the real
/// chunks use, not rounded to blocks; the sea's surface at sea), two triangles a cell, with the
/// surface block's colours and the slope round each corner.
pub fn tile_mesh_measured(map: &CompiledMap, blocks: &TerrainBlocks, palette: &Palette, key: TileKey, finest_cell: i32, slope_cell: i32, eases: bool) -> Option<TileMesh> {
    let cell = (finest_cell << key.ring) as f32;
    let size = cell * CELLS as f32;
    let corner = Vec3::new(key.x as f32 * size, key.y as f32 * size, 0.0);
    let sea = crate::world::SEA_LEVEL as f32;
    // The ground at every corner, with a corner more round the edge for the slopes. `None` = past
    // the map.
    let n = (CELLS + 3) as usize;
    let ground: Vec<Option<(f32, Option<i32>)>> = (0..n * n)
        .map(|k| {
            let (i, j) = ((k % n) as f32 - 1.0, (k / n) as f32 - 1.0);
            map.ground_and_water(corner.x + i * cell, corner.y + j * cell)
        })
        .collect();
    let at = |i: i32, j: i32| ground[(j + 1) as usize * n + (i + 1) as usize].map(|(h, _)| h);
    // A river's or lake's water standing over the ground (its surface).
    let river = |i: i32, j: i32| ground[(j + 1) as usize * n + (i + 1) as usize].and_then(|(_, water)| water).map(|top| top as f32);
    // The surface: the ground, or the sea's over it wherever the near ground has water (`under_sea`),
    // or a river's or lake's.
    let surface = |i: i32, j: i32| at(i, j).map(|h| if under_sea(h) { sea } else { river(i, j).unwrap_or(h) });
    if (0..=CELLS).all(|j| (0..=CELLS).all(|i| at(i, j).is_none())) {
        return None;
    }
    let corners = (CELLS + 1) as usize;
    let mut mesh = TileMesh { positions: Vec::new(), colours: Vec::new(), sides: Vec::new(), slopes: Vec::new(), morph: Vec::new(), indices: Vec::new(), corner, grid: Vec::new() };
    let slope = slope_cell as f32;
    let (ease_from, ease_to) = if eases { (EASE_FROM * size, EASE_TO * size) } else { (f32::MAX, f32::MAX) };
    // Each corner's place in the list (`u32::MAX` past the map).
    let mut index = vec![u32::MAX; corners * corners];
    let mineralogy = map.mineralogy(blocks);
    for j in 0..=CELLS {
        for i in 0..=CELLS {
            let (Some(h), Some(top)) = (at(i, j), surface(i, j)) else { continue };
            let (x, y) = (corner.x + i as f32 * cell, corner.y + j as f32 * cell);
            let water = under_sea(h) || river(i, j).is_some();
            // The slope, blocks up per block across.
            let (north, east) = if water {
                (0.0, 0.0)
            } else if cell < slope {
                // Finer than the slope is measured over: the ground's own height half that either side.
                let z = |dx: f32, dy: f32| map.height(x + dx, y + dy).map_or(top, |h| if under_sea(h) { sea } else { h });
                ((z(slope / 2.0, 0.0) - z(-slope / 2.0, 0.0)) / slope, (z(0.0, slope / 2.0) - z(0.0, -slope / 2.0)) / slope)
            } else {
                let z = |di: i32, dj: i32| surface(i + di, j + dj).unwrap_or(top);
                ((z(1, 0) - z(-1, 0)) / (2.0 * cell), (z(0, 1) - z(0, -1)) / (2.0 * cell))
            };
            // What the ground is made of here (mineralogy, as the near ground has it): the surface
            // block's colours at a spot, and those averaged over spots across the cell round the
            // corner, one every `COLOUR_SPACING` blocks or so (up to 3 × 3). One spot a corner, in the
            // coarse rings, painted a hillside with whatever that spot happened to be (dry grass on a
            // hill mostly of stone and gabbro), and a far copy there showed as a patch of another
            // colour [David, build 7: "it's kind of obvious"].
            let colours_at = |north: f32, east: f32, h: f32, water: Option<i32>| {
                let (ground, _) = crate::world::whole_column(h, None);
                let (block_north, block_east) = (north.floor() + 0.5, east.floor() + 0.5);
                // The slope as every view measures it (not the shading's, which is across the cell).
                let ground_slope = if mineralogy.reads_slope { map.slope(north.floor() as i32, east.floor() as i32, ground) } else { 0.0 };
                let column = crate::world::placer::Column { north: block_north, east: block_east, ground: ground as f32, slope: ground_slope, climate: map.climate(block_north, block_east, ground as f32), water };
                let block = mineralogy.layers(map, column).top;
                (palette.top(block), palette.side(block))
            };
            let (mut top_colour, mut side_colour) = if water {
                (palette.water, palette.water)
            } else {
                let spots = ((cell / COLOUR_SPACING).round() as i32).clamp(1, 3);
                let mut sum = ([0.0f32; 3], [0.0f32; 3], 0.0f32);
                for b in 0..spots {
                    for a in 0..spots {
                        let (north, east) = (x + ((a as f32 + 0.5) / spots as f32 - 0.5) * cell, y + ((b as f32 + 0.5) / spots as f32 - 0.5) * cell);
                        // Spots under the sea or a river, or past the map, leave the land's colour to
                        // the rest.
                        if spots > 1
                            && let Some((h, None)) = map.ground_and_water(north, east)
                            && !under_sea(h)
                        {
                            let (top, side) = colours_at(north, east, h, None);
                            sum = ([0, 1, 2].map(|k| sum.0[k] + top[k]), [0, 1, 2].map(|k| sum.1[k] + side[k]), sum.2 + 1.0);
                        }
                    }
                }
                if sum.2 > 0.0 {
                    (sum.0.map(|v| v / sum.2), sum.1.map(|v| v / sum.2))
                } else {
                    colours_at(x, y, h, river(i, j).map(|top| top as i32))
                }
            };
            // A river narrower than a cell: its water blended in by the share of the cell it covers,
            // so a creek fades in from afar as a texture's mipmaps do, rather than appearing where the
            // near ground begins.
            if !water
                && !map.mutes.rivers
                && let Some((distance, width)) = map.rivers.nearest(x, y)
                && distance < cell / 2.0
            {
                let share = (width / cell).min(1.0) * (1.0 - distance / (cell / 2.0)).max(0.0) * 2.0;
                let blend = |c: [f32; 3]| [0, 1, 2].map(|k| c[k] + (palette.water[k] - c[k]) * share.min(1.0));
                top_colour = blend(top_colour);
                side_colour = blend(side_colour);
            }
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
            mesh.colours.push([top_colour[0], top_colour[1], top_colour[2], if water { 0.0 } else { 1.0 }]);
            mesh.sides.push([side_colour[0], side_colour[1], side_colour[2], 0.0]);
            mesh.slopes.push([north, east, 0.0, 0.0]);
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
    mesh.grid = index;
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
    /// The lighting, from the same settings as the near ground (`Shade::uniforms`): xyz toward the
    /// sun (game axes), w hill shading strength; the face brightness (x tops, y north/south sides, z
    /// east/west sides, w how much of the slope's light the sides take); x height tint, y the finest
    /// cell (the slope's measure), z sea level, w the tint's top; x shading fade, y far slopes match
    /// the near, z light only, w the steepest slope the tables reach.
    pub sun: Vec4,
    pub faces: Vec4,
    pub hill: Vec4,
    pub shade: Vec4,
    /// The steps' feet's shade by steepness, all round and from the sun's side (`Shade::foot`, `step`).
    pub foot: [Vec4; TABLE_VEC4S],
    pub step: [Vec4; TABLE_VEC4S],
    /// x the "who drew this" view on (`docs/caves.md` 13.7), y where a ring's easing starts, in its
    /// own tiles (`EASE_FROM`: the shader tells a corner's ring by it), z how bright the sky is (the
    /// near terrain's stand-in for night: `RenderSettings::sky_brightness`).
    pub view: Vec4,
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
        let vertex_layout = layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0), Mesh::ATTRIBUTE_COLOR.at_shader_location(1), ATTRIBUTE_EASE.at_shader_location(2), ATTRIBUTE_SIDE.at_shader_location(3), ATTRIBUTE_SLOPE.at_shader_location(4)])?;
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
    palette: Option<Arc<Palette>>,
    /// Tiles drawn (an entity), or known empty (`None`).
    tiles: HashMap<TileKey, Option<Entity>>,
    making: HashMap<TileKey, Task<Option<TileMesh>>>,
    /// Tiles no longer wanted, still drawn until the tiles taking their place are in.
    retiring: Vec<(TileKey, Entity)>,
    /// The settings and the tile the wanted list was worked out for.
    wanted_for: Option<(i32, u32, IVec2)>,
    wanted: Vec<TileKey>,
    /// The texture marking chunks on screen, the chunks it marks, and the centre and render
    /// distance it was made for.
    shown_texture: Option<Handle<Image>>,
    shown: HashSet<ChunkPos>,
    shown_for: Option<(IVec2, i32, u64)>,
    /// What the hand-off was last looked over for (see `update_shown_chunks`).
    looked_over: Option<(u64, u64, usize, IVec2, i32, u64)>,
    /// How many times every tile was thrown away (the march throws its copy away too).
    pub(super) cleared: u64,
    /// The square of chunks round the camera's chunk wholly marked in `shown`: its centre chunk and
    /// how many chunks it reaches each way (`None` if not even the camera's chunk is). No far land
    /// can show inside it, so the march's rays start where they leave it.
    pub(super) inner: Option<(IVec2, i32)>,
    // ── For the Performance page ──
    pub quads: u64,
}

impl DistantState {
    /// The real chunks handed over and wholly meshed: on screen (the far copies wait for them before
    /// going, `far_copies.rs`).
    pub fn shown(&self) -> &HashSet<ChunkPos> {
        &self.shown
    }

    /// The blocks' far colours the tiles are made with (the march colours the far copies' cells by them).
    pub(super) fn palette(&self) -> Option<&Arc<Palette>> {
        self.palette.as_ref()
    }

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
        self.cleared += 1;
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
    registry: Res<BlockRegistry>,
    camera: Query<&Transform, With<Camera3d>>,
    mut state: ResMut<DistantState>,
    mut meshes: ResMut<Assets<Mesh>>,
    tiles: Query<&DistantTile>,
    mut march: ResMut<super::march::MarchState>,
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
        state.palette = Some(Arc::new(Palette::from_textures(&registry, &blocks)));
    }
    let Some(camera) = camera.iter().next() else { return };
    let at = bevy_to_game(camera.translation);

    // Which tiles are wanted: worked out again when you move into another finest tile (or the
    // rings change). The light is the shader's alone now, so changing it makes no tile again.
    let finest_tile = (settings.finest_cell * CELLS) as f32;
    let here = IVec2::new((at.x / finest_tile).floor() as i32, (at.y / finest_tile).floor() as i32);
    let now = (settings.finest_cell, settings.rings, here);
    if state.wanted_for.as_ref() != Some(&now) {
        // Cells of another size: every tile has to be made again.
        if state.wanted_for.as_ref().is_some_and(|(finest, ..)| *finest != settings.finest_cell) {
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
    march.follow(state.cleared, settings.rings, settings.finest_cell);
    for (key, mesh) in done {
        // The march reads the same land (`march.rs`).
        march.store(key, mesh.as_ref());
        let Some(tile) = mesh else {
            state.tiles.insert(key, None);
            continue;
        };
        let quads = (tile.indices.len() / 6) as u32;
        let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tile.positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, tile.colours);
        bevy_mesh.insert_attribute(ATTRIBUTE_EASE, tile.morph);
        bevy_mesh.insert_attribute(ATTRIBUTE_SIDE, tile.sides);
        bevy_mesh.insert_attribute(ATTRIBUTE_SLOPE, tile.slopes);
        bevy_mesh.insert_indices(Indices::U32(tile.indices));
        let Some(material) = material.clone() else { continue };
        // Hidden while the march draws the far land (`march::show_one_drawer`).
        let visibility = if settings.march { Visibility::Hidden } else { Visibility::Inherited };
        let entity = commands
            .spawn((Mesh3d(meshes.add(bevy_mesh)), MeshMaterial3d(material), Transform::from_translation(game_to_bevy(tile.corner)), visibility, DistantTile { quads }))
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
    let palette = state.palette.clone().unwrap_or_else(|| Arc::new(Palette::from_textures(&registry, &blocks)));
    let finest = settings.finest_cell;
    while state.making.len() < most {
        let Some(key) = state.wanted.pop() else { break };
        if state.tiles.contains_key(&key) || state.making.contains_key(&key) {
            continue;
        }
        let map = map.clone();
        let (palette, blocks) = (palette.clone(), blocks.clone());
        let task = pool.spawn(async move { tile_mesh(&map, &blocks, &palette, key, finest) });
        state.making.insert(key, task);
    }
}

/// Keeps the distant terrain's fog and survey view the same as the near terrain's.
pub fn update_distant_material(
    render: Res<RenderSettings>,
    distant: Res<DistantSettings>,
    light: Res<LightSettings>,
    clear: Res<ClearColor>,
    state: Res<DistantState>,
    mut materials: ResMut<Assets<DistantMaterial>>,
) {
    let Some(handle) = &state.material else { return };
    if !render.is_changed() && !distant.is_changed() && !light.is_changed() && !clear.is_changed() {
        return;
    }
    if let Some(mut material) = materials.get_mut(handle) {
        let fog = clear.0.to_linear();
        material.settings.fog_color = Vec4::new(fog.red, fog.green, fog.blue, 1.0);
        material.settings.fog = Vec4::new(render.fog_start, render.fog_end, if render.fog_enabled { 1.0 } else { 0.0 }, if render.seam_probe { 1.0 } else { 0.0 });
        material.settings.survey = render.survey_for_shader();
        material.settings.view = Vec4::new(if render.debug_view == super::DebugView::WhoDrew { 1.0 } else { 0.0 }, EASE_FROM, render.sky_brightness.clamp(0.0, 1.0), 0.0);
        // The lighting, from the same settings as the near ground's (lit in the shader).
        Shade::new(&render, &distant, &light).uniforms(&mut material.settings, distant.finest_cell as f32);
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
    copies: Res<super::far_copies::FarCopies>,
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
    let looked = (meshing.changes, streaming.drawn_changes(), streaming.drawn().len(), centre, render.render_distance_chunks * 1000 + render.march_from, copies.changes);
    if state.looked_over == Some(looked) {
        return;
    }
    state.looked_over = Some(looked);

    // Chunks with sections not meshed yet.
    let pending: HashSet<ChunkPos> =
        world.dirty_sections().chain(meshing.waiting.iter()).chain(meshing.running.keys()).chain(meshing.ready.iter().map(|(key, _)| key)).map(|key| key.chunk).collect();
    let drawn = streaming.drawn();
    let before = state.shown.len();
    state.shown.retain(|pos| drawn.contains(pos));
    let mut changed = state.shown.len() != before;
    for &pos in drawn {
        if !pending.contains(&pos) && state.shown.insert(pos) {
            changed = true;
        }
    }
    let now = (centre, render.render_distance_chunks * 1000 + render.march_from, copies.changes);
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
        // Where the near world draws its sections (`apply_render_distance`).
        offset.x.abs().max(offset.y.abs()) <= limit && render.drawn_near(offset) && state.shown.contains(&pos)
    };
    // But not under a chunk holding what the distant terrain can't show (a cave mouth, anything
    // changed): its smooth ground would cover the mouth, or fill the dip of a cave just under the
    // surface [David, build 7: the mouth "stops appearing ... a visible solid color amongst the
    // other blocks"].
    let differs = |pos: ChunkPos| world.changed_chunks().contains(&pos) || copies.differs(pos);
    let mut pixels = vec![0u8; (SHOWN_SIZE * SHOWN_SIZE) as usize];
    for &pos in &state.shown {
        let around = (-1..=1).flat_map(|x| (-1..=1).map(move |y| ChunkPos::new(pos.x + x, pos.y + y)));
        if (on_screen(pos) && differs(pos)) || around.into_iter().all(on_screen) {
            let offset = IVec2::new(pos.x, pos.y) - centre;
            pixels[((offset.y + half) * SHOWN_SIZE + offset.x + half) as usize] = 255;
        }
    }
    // The chunks drawn by far copies (`far_copies.rs`), each on its own: a copy's skirt hides its
    // edge, so it needs no sheet under its neighbours. Marked a little less (`COPY_MARK`): the sheet
    // isn't drawn under them either way, but the march, drawing them itself, tells them apart from
    // the real chunks.
    // Always the copies' mark, even where the real chunk is still in: the near world hides its
    // sections under a copy (`apply_render_distance`), so marked real, nobody drew it [David, build 20:
    // the peak by a cave just past the real chunks lost its top, sky in its place, once he'd moved back
    // across their edge].
    for pos in copies.covered() {
        let offset = IVec2::new(pos.x, pos.y) - centre;
        if offset.x.abs() < half && offset.y.abs() < half {
            pixels[((offset.y + half) * SHOWN_SIZE + offset.x + half) as usize] = COPY_MARK;
        }
    }
    // The square round the camera's chunk wholly marked as real: the march starts its rays at its edge.
    let marked = |offset: IVec2| offset.x.abs() < half && offset.y.abs() < half && pixels[((offset.y + half) * SHOWN_SIZE + offset.x + half) as usize] == 255;
    let mut reach = -1;
    while reach < half - 2 && (-(reach + 1)..=reach + 1).all(|a| [IVec2::new(a, reach + 1), IVec2::new(a, -(reach + 1)), IVec2::new(reach + 1, a), IVec2::new(-(reach + 1), a)].into_iter().all(marked)) {
        reach += 1;
    }
    state.inner = (reach >= 0).then_some((centre, reach));
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
    fn far_slopes_are_lit_as_the_near_ground_s_steps_light_them() {
        // No steps on flat ground; steeper ground, more and closer steps, more of the tops in shade.
        assert_eq!(steps_shade(0.0, 0.45, 4.0, 4.0, 2.0), 1.0);
        let (gentle, steep) = (steps_shade(0.2, 0.45, 4.0, 4.0, 2.0), steps_shade(1.0, 0.45, 4.0, 4.0, 2.0));
        assert!(gentle < 1.0 && steep < gentle && steep > 0.5, "gentle {gentle}, steep {steep}");
        // The shader's tables: the steps' shade from flat to the steepest, four to a vec4, as the
        // shader reads them (distant.wesl, `table_at`).
        let light = LightSettings { sun_from: 0.0, sun_height: 40.0, ..LightSettings::default() };
        let shade = Shade::new(&RenderSettings::default(), &DistantSettings::default(), &light);
        assert_eq!((shade.foot[0], shade.step[0]), (1.0, 1.0));
        assert!(shade.foot[SLOPE_STEPS / 2] < 1.0 && shade.step[SLOPE_STEPS / 2] < 1.0);
        let mut settings = DistantShaderSettings::default();
        shade.uniforms(&mut settings, 16.0);
        for k in 0..=SLOPE_STEPS {
            assert_eq!(settings.foot[k / 4][k % 4], shade.foot[k]);
            assert_eq!(settings.step[k / 4][k % 4], shade.step[k]);
        }
        assert_eq!(settings.shade.w, STEEPEST_STEPS);
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
    fn the_same_slope_reaches_the_shader_alike_from_every_ring() {
        // An even, gentle slope rising east from the middle (2 blocks per 32-block pixel), all grass, under
        // Z 333 to the map's edge.
        let width: usize = 64;
        let z: Vec<f32> = (0..width * width).map(|i| 130.0 + (i % width).saturating_sub(32) as f32 * 2.0).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let palette = Palette::from_textures(&BlockRegistry::default(), &blocks);
        // The slope east most of a tile carries (blocks up per block across, per mille): the shader
        // lights it, so the same slope must reach it alike from every ring.
        let rise = |ring: u8| {
            let mesh = tile_mesh(&map, &blocks, &palette, TileKey { ring, x: 0, y: 0 }, 16).unwrap();
            let mut counts: HashMap<u32, usize> = HashMap::new();
            for s in mesh.slopes.iter().filter(|s| s[1] > 0.0) {
                *counts.entry((s[1] * 1000.0).round() as u32).or_default() += 1;
            }
            counts.into_iter().max_by_key(|&(_, n)| n).unwrap().0
        };
        let (near, further) = (rise(0), rise(1));
        // 2 blocks per 32 across, in both (measured per cell, ring 1's would be about twice as steep).
        assert_eq!(near, 63, "ring 0's slope {near} (per mille)");
        assert!(near.abs_diff(further) <= 1, "ring 0's slope {near}, ring 1's {further} (per mille)");
    }

    #[test]
    fn a_tile_is_a_smooth_sheet_on_the_ground_and_past_the_map_is_nothing() {
        let width = 64;
        // Sea in the west, land from 256 blocks east of the middle (inside the tiles tested), rising.
        let z: Vec<f32> = (0..width * width).map(|i| if i % width < 40 { 100.0 } else { 160.0 + (i % width) as f32 }).collect();
        let map = CompiledMap::of_heights(width, 32.0, &z);
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let palette = Palette::from_textures(&BlockRegistry::default(), &blocks);
        let key = TileKey { ring: 1, x: 0, y: 0 };
        let mesh = tile_mesh(&map, &blocks, &palette, key, 16).unwrap();
        // A corner every cell, two triangles a cell, and the easing for each corner.
        assert_eq!(mesh.positions.len(), ((CELLS + 1) * (CELLS + 1)) as usize);
        assert_eq!(mesh.indices.len(), (CELLS * CELLS * 6) as usize);
        assert_eq!(mesh.morph.len(), mesh.positions.len());
        // Every corner on the ground (or the sea's surface), not rounded to blocks.
        let sea = crate::world::SEA_LEVEL as f32;
        for (p, morph) in mesh.positions.iter().zip(&mesh.morph) {
            let game = bevy_to_game(Vec3::from_array(*p)) + mesh.corner;
            let ground = map.height(game.x, game.y).map(|h| if under_sea(h) { sea } else { h }).unwrap();
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
        let coast = tile_mesh(&map, &blocks, &palette, TileKey { ring: 1, x: -1, y: 0 }, 16).unwrap();
        let shows = |colour: [f32; 3]| coast.colours.iter().any(|c| c[..3] == colour);
        assert!(shows(palette.water) && shows(palette.top(blocks.grass)));
        // Far past the map: nothing.
        assert!(tile_mesh(&map, &blocks, &palette, TileKey { ring: 0, x: 100, y: 100 }, 16).is_none());
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

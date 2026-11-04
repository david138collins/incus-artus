//! Far copies — the world as it is, coarsened, past the real chunks, wherever it differs from the
//! plain land the far sheet draws [`docs/caves.md` section 13; David, 2026-11-02: "builder may
//! proceed with the present plan"].
//!
//! **One path for everything that isn't the plain recipe:** a cave's mouth, a pit, a wall, a
//! build, an object. Nothing gets an ID. The layer here asks the chunks' question, "what's here?",
//! coarsely, and only where the answer can differ from the sheet:
//! - **changed chunks** (the save's own list);
//! - **chunk columns where a cave comes near the ground** (the cave plan's boxes, checked against
//!   the ground's height there);
//! - **and round each one found to show** (its 8 neighbours), so a copy's edge meets plain ground.
//!
//! A chunk "shows" if it's changed, or if daylight reaches covered air in it (a mouth, an open shaft):
//! the test needs no IDs. A deep tunnel shows nothing from outside, so it's left to the sheet.
//!
//! **A copy is a tile** of 2 × 2, 4 × 4 or 8 × 8 chunks drawn in cells of 2, 4 or 8 blocks (or one
//! chunk at full detail), so every tile is 32 cells across, one section of the near mesher's. Which
//! size depends on the distance and the sharpness: a cell about 4 ÷ sharpness pixels on screen. A tile
//! is only ever wholly outside the real chunks; near their edge the finer tiles fill in.
//!
//! **Making one** (in the background, nearest first):
//! 1. its chunks at full detail, as streaming makes them (the recipe, carved), or as changed;
//! 2. daylight at full detail, as the near chunks have it (`sky_light.rs`): full in the open, a step
//!    darker each block into covered air, black at its reach;
//! 3. coarsened: a cell is solid if at least "how much must be solid" of it is (objects counted by
//!    their shapes), looking like the highest solid block in it; else water if mostly water; else
//!    air. An air cell's daylight is the average of its blocks' and of the air hidden in the solid
//!    cells beside it, so an opening smaller than a cell still darkens the faces round it;
//! 4. meshed by the near mesher and drawn by the near shader, scaled up (`terrain.wesl`).
//!
//! **Its edges:** the ground beside a tile counts as open for 2 cells down from its surface, so a
//! tile's sides show there: a skirt hiding any sliver of sky where its steps meet the smooth sheet.
//!
//! **The hand-off:** a drawn tile cuts its chunks out of the sheet with the mask the real chunks use
//! (`distant.rs`). A tile no longer wanted stays until the tiles taking its place are in.

use super::material::{ATTRIBUTE_PACKED_VOXEL, TerrainMaterial};
use super::mesher::{MeshInput, MeshOutput, face_index, mesh_section_lit, plane_axes};
use super::sky_light::{Daylight, FULL, sky_heights};
use super::{RenderSettings, TerrainAssets};
use crate::data::{BlockRegistry, ObjectRegistry};
use crate::world::{BlockId, CHUNK_SIZE, CavePlan, Caves, Chunk, ChunkPos, CompiledMap, StreamSettings, Terrain, TerrainBlocks, VoxelWorld, WORLD_BOTTOM, real_chunk, shape_boxes};
use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use bevy::window::PrimaryWindow;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

/// Cells across a tile (one section of the mesher's).
const CELLS: i32 = 32;
/// The coarsest cell (blocks).
const COARSEST: i32 = 8;
/// The furthest a tile may be, in chunks from yours (the larger of north-south and east-west):
/// inside the hand-off mask (`distant.rs`).
const MASK_REACH: i32 = super::distant::SHOWN_SIZE / 2 - 2;
/// How far down from the ground beside a tile its sides still show (cells): the skirt.
const SKIRT: i32 = 2;
/// How far below the lowest ground in a tile it's copied from (blocks): pits and mouths.
const BELOW: i32 = 24;
/// The most tiles being made at once (the processor runs hot; streaming comes first).
const MOST_AT_ONCE: usize = 2;
/// Marks a face of plain ground in its texture layer, which then carries its block's top layer (the
/// low 7 bits) and side layer (the 7 above): terrain.wesl draws it as the far terrain draws the land.
const PLAIN: u32 = 0x8000;
/// How far past the line between two sizes a tile has to be before it changes size (a share of the
/// distance), so going back and forth on the line doesn't flip it.
const SWITCH_PAST: f32 = 1.25;
/// How wide the band of whole copies is past the far view's line, when it's pulled in (chunks).
const FULL_BAND: i32 = 6;

/// Tunables (the Render page, "Far copies"; saved in "everything" presets).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FarCopySettings {
    pub on: bool,
    /// How crisp: a copy's cell is about 4 ÷ this pixels on screen (1 soft, 4 a pixel a cell).
    pub sharpness: f32,
    /// How much of a cell must be solid for it to be solid (a share): lower keeps thin walls and
    /// pillars, higher keeps small holes open.
    pub solid_share: f32,
    /// How far copies reach (blocks, the larger of north-south and east-west).
    pub reach: f32,
    /// Drawn by the march from their cells, not as meshes (while the march draws the far land:
    /// `march.rs`, `docs/far-view.md` step 1 part 5).
    pub march: bool,
    /// With the line pulled in (`RenderSettings::march_from`), the real chunks past it copied straight
    /// from themselves and drawn by the march, no longer meshed (R2 in `docs/far-view.md`). Off: a
    /// band of whole copies made from the recipe, the real chunks meshed underneath (build 28's way).
    pub real: bool,
    /// The march's whole copies shaded as the meshes are: the same face placement, random turn,
    /// ground light and underground light (one shading recipe, `assets/shaders/lighting.wesl`), and
    /// the texture's size by what each pixel covers. Off: build 35's own shading, to compare.
    pub shaded: bool,
    /// Sizes per ray: each pixel of the march's whole copies drawn from the level of the copy's cells
    /// (`copy_levels`) about `pixels` pixels across where it is, the change from one level to the next
    /// spread over the cells it passes (David: "sizing rays fits our philosophy"). Off: every cell
    /// at the copy's own size, as build 37.
    pub sized: bool,
    /// About how many pixels across a far cell is drawn, before the next coarser level is (David's
    /// "pixelization rate of terrain as it becomes more distant"): higher is blockier further off.
    /// Never finer than the copy keeps (a cell about 4 ÷ `sharpness` pixels).
    pub pixels: f32,
    /// Contour bands (round 2): the far land, and the march's cells of 2 blocks or more on the open
    /// ground, drawn with the steps' sides in bands at each whole block of height, the copies' from
    /// their own columns' tops, a band thinner than a pixel a faint line, several steps to a pixel
    /// their plain mix (David: "contour bands are how we see shape and depth in land, we just need
    /// them to all look the same"). Off: build 42's look, the far land in its plain mix and the
    /// coarser cells as blocks.
    pub bands: bool,
    /// The reference view (for looking, not playing: slow): 16 rays a pixel through the copies' cells
    /// at their own size, textured as the meshes, no bands, so a pixel shows what the blocks come to
    /// from where you are: what the far land's look is to match. Not saved on.
    #[serde(skip)]
    pub reference: bool,
}

impl Default for FarCopySettings {
    fn default() -> Self {
        Self { on: true, sharpness: 2.0, solid_share: 0.5, reach: 4_000.0, march: true, real: true, shaded: true, sized: true, pixels: 2.0, bands: true, reference: false }
    }
}

impl FarCopySettings {
    /// Whether the march draws the copies (it has to be drawing the far land too).
    pub fn marched(&self, distant: &super::distant::DistantSettings) -> bool {
        self.on && self.march && distant.march && distant.enabled
    }
}

/// Whether the march draws the real chunks past the line (`FarCopySettings::real`): only while it
/// draws the copies, with the line inside where the real chunks end.
pub fn marches_real(settings: &FarCopySettings, distant: &super::distant::DistantSettings, render: &RenderSettings, streaming: &StreamSettings) -> bool {
    settings.real && settings.marched(distant) && render.march_from < streaming.radius
}

/// One tile: its cell size (blocks) and where it is in the grid of tiles that size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub scale: i32,
    pub x: i32,
    pub y: i32,
}

impl TileKey {
    /// The tile of this size a chunk is in.
    fn of(chunk: ChunkPos, scale: i32) -> Self {
        Self { scale, x: chunk.x.div_euclid(scale), y: chunk.y.div_euclid(scale) }
    }

    fn chunks(self) -> impl Iterator<Item = ChunkPos> {
        let s = self.scale;
        (0..s).flat_map(move |j| (0..s).map(move |i| ChunkPos::new(self.x * s + i, self.y * s + j)))
    }

    /// Its middle (blocks, north and east).
    fn middle(self) -> Vec2 {
        let width = (CHUNK_SIZE * self.scale) as f32;
        Vec2::new((self.x as f32 + 0.5) * width, (self.y as f32 + 0.5) * width)
    }

    /// Its chunks' range from `home`: its nearest chunk's offset north-south and east-west (each
    /// the nearest along that way), and its furthest, the larger of north-south and east-west.
    fn chunk_range(self, home: IVec2) -> (IVec2, i32) {
        let s = self.scale;
        let span = |low: i32, at: i32| {
            let (a, b) = (low * s - at, low * s + s - 1 - at);
            (if a <= 0 && b >= 0 { 0 } else { a.abs().min(b.abs()) }, a.abs().max(b.abs()))
        };
        let (near_x, far_x) = span(self.x, home.x);
        let (near_y, far_y) = span(self.y, home.y);
        (IVec2::new(near_x, near_y), far_x.max(far_y))
    }
}

/// A tile made: its sections (where each starts, blocks; and its mesh), its cells for the march, the
/// chunks in it that show, and how long it took.
struct Made {
    sections: Vec<(IVec3, MeshOutput)>,
    cells: Option<Arc<CopyCells>>,
    hot: Vec<ChunkPos>,
    ms: f32,
}

/// A tile's cells as the march reads them (`march.rs`, step 1 part 5): where its first cell's corner
/// is (blocks), its cell size, how many cells tall it is, and each cell (`CELLS` × `CELLS` a level,
/// row by row east, north first, from the bottom), packed: its look's block number in the low 12
/// bits; 1 solid or 2 water in the 2 above; 1 << 14 if plain ground; an air cell's daylight (0 to
/// `FULL`) in bits 16 to 19, and its placed light (a brazier's: red, green, blue, 0 to 15 each, as
/// `placed_light::pack` packs it) in bits 20 to 31. Below its bottom is solid ground; above its top, open air.
pub struct CopyCells {
    pub corner: IVec3,
    pub scale: i32,
    pub tall: i32,
    /// Copied whole (the near-march experiment): the march draws its blocks with their textures.
    pub full: bool,
    pub cells: Vec<u32>,
    /// A whole copy's coarser levels (`copy_levels`): its cells 2, 4 and 8 to a side made one, for
    /// the march to draw a cell about as big as a few pixels wherever it is (sizes per ray,
    /// `docs/far-view.md`). Empty for the rest.
    pub levels: Vec<u32>,
}

pub const CELL_SOLID: u32 = 1 << 12;
pub const CELL_WATER: u32 = 2 << 12;
pub const CELL_PLAIN: u32 = 1 << 14;

/// How many coarser levels a whole copy keeps (`copy_levels`): 2, 4 and 8 of its cells to a side.
pub const LEVELS: u32 = 3;

/// A copy's coarser levels, each made from the one finer (`cells`, 32 × 32 × `tall`, packed as
/// `CopyCells`), one after another: level 1 is 16 × 16 × `tall`/2 (rounded up), level 2 8 × 8 ×
/// `tall`/4 and level 3 4 × 4 × `tall`/8, each row by row east, north first, then up. Each cell is
/// one of the eight it covers:
/// - solid if at least `share` of them are (as a copy's cells are made from blocks), the block of
///   its highest solid one (the surface, as the far land shows it);
/// - else water if as much is water as air;
/// - else air: its daylight as bright as its air's on average (the average of the eased light, so
///   squaring it again on the graphics card doesn't darken it), and the brightest placed light of
///   any of it, each colour (a far brazier doesn't go out by being averaged away: Counsel).
///
/// Above the top it's open air in full daylight.
pub fn copy_levels(cells: &[u32], tall: i32, share: f32) -> Vec<u32> {
    let mut levels = Vec::new();
    let (mut finer, mut side, mut high) = (cells.to_vec(), 32i32, tall);
    for _ in 0..LEVELS {
        let (coarse_side, coarse_high) = (side / 2, (high + 1) / 2);
        let at = |x: i32, y: i32, z: i32| if z < high { finer[(x + y * side + z * side * side) as usize] } else { 15 << 16 };
        let mut coarse = Vec::with_capacity((coarse_side * coarse_side * coarse_high) as usize);
        // Each cell's highest solid block of the eight, solid or not (for the cell under it).
        let mut tops = Vec::with_capacity(coarse.capacity());
        for z in 0..coarse_high {
            for y in 0..coarse_side {
                for x in 0..coarse_side {
                    let (mut solid, mut water, mut air) = (0u32, 0u32, 0u32);
                    let (mut top, mut water_block, mut light, mut glow) = (None, 0u32, 0.0f32, 0u16);
                    for (dx, dy, dz) in (0..8).map(|k| (k & 1, (k >> 1) & 1, k >> 2)) {
                        let (cz, value) = (z * 2 + dz, at(x * 2 + dx, y * 2 + dy, z * 2 + dz));
                        match (value >> 12) & 3 {
                            1 => {
                                solid += 1;
                                if top.is_none_or(|(at, _)| cz > at) {
                                    top = Some((cz, value & 0xFFF));
                                }
                            }
                            2 => {
                                water += 1;
                                water_block = value & 0xFFF;
                            }
                            _ => {
                                air += 1;
                                let level = ((value >> 16) & 15) as f32 / 15.0;
                                light += level * level;
                                glow = super::placed_light::brighter(glow, (value >> 20) as u16);
                            }
                        }
                    }
                    tops.push(top.map(|(_, block)| block));
                    coarse.push(if solid as f32 >= share * 8.0 - 1e-3 {
                        top.map_or(0, |(_, block)| block) | CELL_SOLID
                    } else if water > 0 && water >= air {
                        water_block | CELL_WATER
                    } else {
                        let daylight = ((light / air.max(1) as f32).sqrt() * 15.0).round() as u32;
                        daylight.min(15) << 16 | (glow as u32) << 20
                    });
                }
            }
        }
        // A solid cell under one that isn't, but holds some of the ground's top: it's that top that
        // shows, so it looks like it (the surface, not the stone or dirt under it). [Grey and brown
        // patches by the ground's height within a cell, at the coarser cells' distances: David,
        // build 43, "the grey is still there".]
        let layer = (coarse_side * coarse_side) as usize;
        for k in 0..coarse.len().saturating_sub(layer) {
            if (coarse[k] >> 12) & 3 == 1 && (coarse[k + layer] >> 12) & 3 != 1
                && let Some(block) = tops[k + layer]
            {
                coarse[k] = (coarse[k] & !0xFFF) | block;
            }
        }
        levels.extend_from_slice(&coarse);
        (finer, side, high) = (coarse, coarse_side, coarse_high);
    }
    levels
}

enum Tile {
    Drawn { entities: Vec<Entity>, faces: u32, bytes: usize, cells: Option<Arc<CopyCells>> },
    /// Nothing in it shows: the sheet draws it.
    Empty,
}

/// A drawn copy's section (for the Performance page and the who-drew view's counts).
#[derive(Component)]
pub struct FarCopy;

/// What the wanted tiles were worked out for: your chunk, the render distance, the streaming radius,
/// the sharpness and reach, how many chunks are changed and show, and the view.
type WantedFor = (IVec2, i32, i32, [u32; 2], usize, usize, u32);

/// The camera's place and lens.
type CameraLens<'w, 's> = Query<'w, 's, (&'static Transform, &'static Projection), With<Camera3d>>;

#[derive(Resource, Default)]
pub struct FarCopies {
    /// The terrain material, marked as a copy's (the shader colours copies in the who-drew view).
    material: Option<Handle<TerrainMaterial>>,
    /// What the tiles were made from.
    map: Option<Arc<CompiledMap>>,
    plan: Option<Arc<CavePlan>>,
    revision: u64,
    share: f32,
    /// The tiles copied whole (the near-march experiment: their plain ground as blocks too).
    full: HashSet<TileKey>,
    /// Whether the real chunks past the line are copied from themselves (`marches_real`): then the
    /// whole tiles are theirs.
    real: bool,
    /// Whether the tiles were meshed (drawn as their own blocks), or made for the march alone.
    meshed: Option<bool>,
    /// Chunk columns where a cave comes near the ground.
    near_ground: HashSet<ChunkPos>,
    /// Each cave's chunks: once any of them shows, all are copied, so the cave is closed wherever it
    /// runs (a tunnel running on into a chunk with no copy was open at the tile's edge, and the sky
    /// showed through it: David, build 9).
    caves: Vec<HashSet<ChunkPos>>,
    /// Changed chunks known of, and chunks found to show.
    known_changed: HashSet<ChunkPos>,
    hot: HashSet<ChunkPos>,
    tiles: HashMap<TileKey, Tile>,
    making: HashMap<TileKey, Task<Made>>,
    /// Tiles wanted (and whether each is made even if nothing in it shows: a neighbour's), and those
    /// still to start, nearest last.
    wanted: HashMap<TileKey, bool>,
    queue: Vec<TileKey>,
    wanted_for: Option<WantedFor>,
    retiring: Vec<(TileKey, Vec<Entity>, Option<Arc<CopyCells>>)>,
    /// The chunks drawn by copies (cut out of the sheet), and a count of its changes.
    covered: HashSet<ChunkPos>,
    pub changes: u64,
    /// How long the last tiles took to make (ms).
    times: VecDeque<f32>,
}

impl FarCopies {
    /// The chunks the copies draw, for the hand-off mask.
    pub fn covered(&self) -> &HashSet<ChunkPos> {
        &self.covered
    }

    /// The tiles drawn, with their cells, for the march (`march.rs`): those drawn first, then those
    /// still waiting to retire (which the march draws only where no tile drawn covers).
    pub fn march_tiles(&self) -> Vec<(TileKey, Arc<CopyCells>)> {
        let drawn = self.tiles.iter().filter_map(|(key, tile)| match tile {
            Tile::Drawn { cells: Some(cells), .. } => Some((*key, cells.clone())),
            _ => None,
        });
        let retiring = self.retiring.iter().filter_map(|(key, _, cells)| cells.clone().map(|cells| (*key, cells)));
        drawn.chain(retiring).collect()
    }

    /// Might this chunk show something the distant terrain can't (a cave near the ground, or one
    /// found to show)? The distant terrain isn't drawn under it at the real chunks' edge
    /// (`distant.rs`, `update_shown_chunks`).
    pub fn differs(&self, pos: ChunkPos) -> bool {
        self.hot.contains(&pos) || self.near_ground.contains(&pos)
    }

    fn clear(&mut self, commands: &mut Commands) {
        for (_, tile) in self.tiles.drain() {
            if let Tile::Drawn { entities, .. } = tile {
                entities.into_iter().for_each(|entity| commands.entity(entity).despawn());
            }
        }
        for (_, entities, _) in self.retiring.drain(..) {
            entities.into_iter().for_each(|entity| commands.entity(entity).despawn());
        }
        self.making.clear();
        self.wanted.clear();
        self.queue.clear();
        self.wanted_for = None;
        self.hot.clear();
        self.update_covered();
    }

    fn retire(&mut self, key: TileKey) {
        if let Some(Tile::Drawn { entities, cells, .. }) = self.tiles.remove(&key) {
            self.retiring.push((key, entities, cells));
        }
        self.making.remove(&key);
    }

    fn update_covered(&mut self) {
        let drawn = self.tiles.iter().filter(|(_, tile)| matches!(tile, Tile::Drawn { .. })).map(|(key, _)| *key);
        self.covered = drawn.chain(self.retiring.iter().map(|(key, ..)| *key)).flat_map(TileKey::chunks).collect();
        self.changes += 1;
    }

    /// One line for the Render page: tiles drawn by size, being made, faces, memory, and how long
    /// making one takes.
    pub fn summary(&self) -> String {
        let mut by_size = [0; 4];
        let (mut faces, mut bytes, mut empty) = (0u64, 0usize, 0);
        for (key, tile) in &self.tiles {
            match tile {
                Tile::Drawn { faces: f, bytes: b, .. } => {
                    by_size[key.scale.trailing_zeros().min(3) as usize] += 1;
                    faces += *f as u64;
                    bytes += b;
                }
                Tile::Empty => empty += 1,
            }
        }
        let times = if self.times.is_empty() {
            "none made yet".to_string()
        } else {
            let (low, high) = self.times.iter().fold((f32::MAX, 0.0f32), |(low, high), &t| (low.min(t), high.max(t)));
            format!("making one {low:.0} / {:.0} / {high:.0} ms (least / average / most)", self.times.iter().sum::<f32>() / self.times.len() as f32)
        };
        let real = if self.real { format!(" {} from the real chunks past the line.", self.full.iter().filter(|key| self.tiles.contains_key(key)).count()) } else { String::new() };
        format!(
            "Copies drawn: {} at full detail, {} at 2, {} at 4, {} at 8 blocks a cell; {empty} with nothing to show; {} being made.{real}\n{faces} faces, {:.1} MB; {times}.",
            by_size[0],
            by_size[1],
            by_size[2],
            by_size[3],
            self.making.len(),
            bytes as f64 / 1_048_576.0
        )
    }
}

/// The cell size wanted `distance` blocks away: about 4 ÷ sharpness pixels a cell, with the camera's
/// field of view (radians) and the window's height (pixels). 1, 2, 4 or 8.
fn scale_for(distance: f32, sharpness: f32, fov: f32, height: f32) -> i32 {
    let pixels_per_block = (height / 2.0) / (distance.max(1.0) * (fov / 2.0).tan().max(1e-3));
    let blocks = 4.0 / sharpness.clamp(0.5, 8.0) / pixels_per_block;
    let mut scale = 1;
    while scale < COARSEST && (scale * 2) as f32 <= blocks {
        scale *= 2;
    }
    scale
}

/// The chunk columns where a cave comes within 8 blocks of the ground's height there (its lowest and
/// highest at the chunk's corners and middle).
fn near_ground(plan: &CavePlan, map: &CompiledMap) -> HashSet<ChunkPos> {
    let mut ground: HashMap<ChunkPos, (i32, i32)> = HashMap::new();
    let mut near = HashSet::new();
    for (pos, level) in plan.boxes() {
        let (low, high) = *ground.entry(pos).or_insert_with(|| {
            let corner = pos.min_block();
            [(0, 0), (31, 0), (0, 31), (31, 31), (16, 16)]
                .iter()
                .filter_map(|&(x, y)| map.column(corner.x + x, corner.y + y).map(|(z, _)| z))
                .fold((i32::MAX, i32::MIN), |(low, high), z| (low.min(z), high.max(z)))
        });
        let (bottom, top) = (level * CHUNK_SIZE, level * CHUNK_SIZE + CHUNK_SIZE - 1);
        if top >= low - 8 && bottom <= high + 8 {
            near.insert(pos);
        }
    }
    near
}

/// Everything a tile is made from, copied out so it can be made on another thread.
struct CopyJob {
    key: TileKey,
    map: Arc<CompiledMap>,
    plan: Arc<CavePlan>,
    blocks: TerrainBlocks,
    /// Its changed chunks, as they are, and its objects' shapes (blocks, in the world).
    changed: HashMap<ChunkPos, Chunk>,
    objects: Vec<(Vec3, Vec3)>,
    /// The lights its changed chunks' objects give (a brazier's), flooded through its cells for the march.
    lights: Vec<super::placed_light::LightFrom>,
    opaque: Arc<Vec<bool>>,
    see_through: Arc<Vec<bool>>,
    inner_faces: Arc<Vec<bool>>,
    face_layers: Arc<Vec<[u32; 6]>>,
    face_random_rotation: Arc<Vec<[bool; 6]>>,
    bar_layers: Arc<Vec<Option<[u32; 2]>>>,
    /// How far daylight reaches in (blocks), how much of a cell must be solid, and whether it's made
    /// even if nothing in it shows (a neighbour of one that does).
    reach: i32,
    share: f32,
    force: bool,
    /// Whether it's meshed: only when the copies are drawn as their own blocks, not by the march.
    mesh: bool,
    /// Copied whole (the near-march experiment): nothing in it is plain ground.
    full: bool,
    /// Made from the real chunks past the line (R2): its chunks are in `changed`, as they are; its
    /// see-through blocks and pieces left to the near meshes (air here), its objects to their
    /// models; its placed light as the near world's (`glows`, each lit block's), not flooded again.
    real: bool,
    glows: Vec<(IVec3, u16)>,
}

/// What's added up in each cell while coarsening.
#[derive(Clone, Copy, Default)]
struct Tally {
    air: u32,
    light: u32,
    water: u32,
    water_block: BlockId,
    /// Its highest solid block, and its highest block of the surface (a column's top, open to the
    /// sky): with both, a cell is the surface's.
    top: Option<(i32, BlockId)>,
    surface: Option<(i32, BlockId)>,
    objects: f32,
}

/// Makes a tile (see the module notes).
fn make_copy(mut job: CopyJob) -> Made {
    let started = Instant::now();
    let s = job.key.scale;
    let width = CHUNK_SIZE * s;
    let corner = IVec2::new(job.key.x, job.key.y) * width;
    // Its changed chunks, taken out of the job (they go into the copy as they are).
    let mut changed = std::mem::take(&mut job.changed);
    let changed_here: HashSet<ChunkPos> = changed.keys().copied().collect();
    let is_opaque = |id: BlockId| id != BlockId::PIECES && job.opaque.get(id.0 as usize).copied().unwrap_or(true);
    let is_see_through = |id: BlockId| job.see_through.get(id.0 as usize).copied().unwrap_or(false);

    // 1. Its chunks at full detail, and each column's sky height (its highest solid block).
    let mut chunks: HashMap<ChunkPos, Chunk> = HashMap::new();
    for pos in job.key.chunks() {
        if let Some(chunk) = changed.remove(&pos).or_else(|| job.map.chunk(pos, &job.blocks, &job.plan)) {
            chunks.insert(pos, chunk);
        }
    }
    let mut sky = vec![i32::MIN; (width * width) as usize];
    for (pos, chunk) in &chunks {
        let heights = sky_heights(chunk, &is_opaque);
        let (ox, oy) = ((pos.x * CHUNK_SIZE - corner.x), (pos.y * CHUNK_SIZE - corner.y));
        for (k, &height) in heights.iter().enumerate() {
            let (x, y) = (ox + k as i32 % CHUNK_SIZE, oy + k as i32 / CHUNK_SIZE);
            sky[(x + y * width) as usize] = height;
        }
    }
    let present: Vec<i32> = sky.iter().copied().filter(|&h| h != i32::MIN).collect();
    let (Some(&lowest), Some(&highest)) = (present.iter().min(), present.iter().max()) else {
        return Made { sections: Vec::new(), cells: None, hot: Vec::new(), ms: 0.0 };
    };
    let reach = job.reach.clamp(1, super::sky_light::MOST_REACH);
    // Made from the real chunks, it's copied only as deep as daylight reaches under the lowest ground
    // (below that it's looked into nowhere, so solid either way): less room on the graphics card.
    let (bottom, top) = ((lowest - if job.real { reach + 1 } else { BELOW }).max(WORLD_BOTTOM), highest + 1);
    let sky_at = |x: i32, y: i32| if (0..width).contains(&x) && (0..width).contains(&y) { sky[(x + y * width) as usize] } else { i32::MIN };
    // Each column is only looked into down to daylight's reach below the lowest open air beside it:
    // deeper than that nothing is lit, and from outside it's solid ground. (Looking through every
    // block from the lowest ground up made a mountain tile take seconds.)
    let lowest_beside = |x: i32, y: i32| {
        let here = sky_at(x, y);
        [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().map(|&(dx, dy)| sky_at(x + dx, y + dy)).filter(|&h| h != i32::MIN).fold(here, i32::min)
    };
    let from: Vec<i32> = (0..width * width).map(|k| (lowest_beside(k % width, k / width) - reach - 1).max(bottom)).collect();
    let from_at = |x: i32, y: i32| from[(x + y * width) as usize];
    let block_at = |x: i32, y: i32, z: i32| {
        let at = IVec3::new(corner.x + x, corner.y + y, z);
        chunks.get(&ChunkPos::of_block(at)).map_or(BlockId::AIR, |chunk| chunk.get(at))
    };
    // Covered: below its column's sky height, in the range looked into, and not solid.
    let covered = |p: IVec3| {
        let height = sky_at(p.x, p.y);
        height != i32::MIN && p.z >= from_at(p.x, p.y) && p.z <= height && !is_opaque(block_at(p.x, p.y, p.z))
    };
    let open = |p: IVec3| {
        let height = sky_at(p.x, p.y);
        height != i32::MIN && p.z > height
    };

    // 2. Daylight in covered air: one step in from the open, a step darker each block on. Only air
    // with open air beside it starts: lower than the lowest open air beside its column, there's none.
    let sides = [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z];
    let mut steps: HashMap<IVec3, i32> = HashMap::new();
    let mut queue = VecDeque::new();
    for y in 0..width {
        for x in 0..width {
            let height = sky_at(x, y);
            for z in (lowest_beside(x, y) + 1).max(from_at(x, y))..height {
                let p = IVec3::new(x, y, z);
                if covered(p) && sides.iter().any(|&side| open(p + side)) {
                    steps.insert(p, 1);
                    queue.push_back(p);
                }
            }
        }
    }
    while let Some(p) = queue.pop_front() {
        let here = steps[&p];
        if here >= reach {
            continue;
        }
        for side in sides {
            let next = p + side;
            if !steps.contains_key(&next) && covered(next) {
                steps.insert(next, here + 1);
                queue.push_back(next);
            }
        }
    }
    let lit = |p: IVec3| steps.get(&p).map_or(0, |&s| if s >= reach { 0 } else { FULL as i32 * (reach - s) / reach }) as u32;
    // Changed chunks show; so do those daylight reaches covered air in.
    let was_changed = changed_here.clone();
    let mut hot = changed_here;
    for (p, _) in steps.iter().filter(|(p, _)| lit(**p) > 0) {
        hot.insert(ChunkPos::of_block(IVec3::new(corner.x + p.x, corner.y + p.y, 0)));
    }

    // 3. Coarsened.
    // A whole copy starts on a whole number of its coarsest level's cells up (`copy_levels`), so its
    // coarser cells line up with its neighbours' (the cells below where it was looked into are solid).
    let low_cell = if job.full { bottom.div_euclid(s).div_euclid(1 << LEVELS) << LEVELS } else { bottom.div_euclid(s) };
    let high_cell = top.div_euclid(s);
    let tall = high_cell - low_cell + 1;
    let index = |x: i32, y: i32, z: i32| (x + y * CELLS + z * CELLS * CELLS) as usize;
    let mut tallies = vec![Tally::default(); (CELLS * CELLS * tall) as usize];
    for y in 0..width {
        for x in 0..width {
            let height = sky_at(x, y);
            if height == i32::MIN {
                continue;
            }
            // Below where the column is looked into, it counts as solid; above its sky height it's open
            // air, counted a cell at a time.
            let last = (high_cell + 1) * s - 1;
            for z in from_at(x, y)..=height.min(last) {
                let tally = &mut tallies[index(x / s, y / s, z.div_euclid(s) - low_cell)];
                let id = block_at(x, y, z);
                if is_opaque(id) {
                    if tally.top.is_none_or(|(at, _)| z > at) {
                        tally.top = Some((z, id));
                    }
                    if z == height && tally.surface.is_none_or(|(at, _)| z > at) {
                        tally.surface = Some((z, id));
                    }
                } else if is_see_through(id) && !job.real {
                    tally.water += 1;
                    tally.water_block = id;
                } else {
                    tally.air += 1;
                    tally.light += lit(IVec3::new(x, y, z));
                }
            }
            for level in (height + 1).max(bottom).div_euclid(s)..=high_cell {
                let open = ((level + 1) * s).min(last + 1) - (level * s).max(height + 1).max(bottom);
                let tally = &mut tallies[index(x / s, y / s, level - low_cell)];
                tally.air += open as u32;
                tally.light += open as u32 * FULL as u32;
            }
        }
    }
    // Objects, by their shapes: the share of each block they fill.
    for &(low, high) in &job.objects {
        let (low, high) = (low - Vec3::new(corner.x as f32, corner.y as f32, 0.0), high - Vec3::new(corner.x as f32, corner.y as f32, 0.0));
        for z in low.z.floor() as i32..high.z.ceil() as i32 {
            for y in low.y.floor() as i32..high.y.ceil() as i32 {
                for x in low.x.floor() as i32..high.x.ceil() as i32 {
                    if !(0..width).contains(&x) || !(0..width).contains(&y) || z < low_cell * s || z > high_cell * s + s - 1 {
                        continue;
                    }
                    let overlap = |a: f32, b: f32, at: i32| (b.min(at as f32 + 1.0) - a.max(at as f32)).max(0.0);
                    let fill = overlap(low.x, high.x, x) * overlap(low.y, high.y, y) * overlap(low.z, high.z, z);
                    tallies[index(x / s, y / s, z.div_euclid(s) - low_cell)].objects += fill;
                }
            }
        }
    }
    let whole = (s * s * s) as f32;
    let solid_enough = |tally: &Tally| whole - tally.air as f32 - tally.water as f32 + tally.objects >= job.share * whole - 1e-3;
    let cells: Vec<BlockId> = tallies
        .iter()
        .enumerate()
        .map(|(k, tally)| {
            if solid_enough(tally) {
                // The surface in the cell over it, if that cell isn't solid: it's what shows on top
                // [where the ground's top fell low in a cell, that cell was air and the one under it
                // showed the stone or dirt below: grey and brown patches, David, build 43, "the grey
                // is still there"]. Else its highest block of the surface, or its highest solid block,
                // or the one below's, or stone. (Its highest solid block alone, on a steep slope, was
                // often the rock under an uphill column's surface, so coarse copies came out darker
                // and blotchy: David, build 8, "the texture breaks out" backing away.)
                let below = (k as i32 - CELLS * CELLS).max(0) as usize;
                let over = tallies.get(k + (CELLS * CELLS) as usize).filter(|over| !solid_enough(over)).and_then(|over| over.surface);
                over.or(tally.surface).or(tally.top).or(tallies[below].surface).or(tallies[below].top).map_or(job.blocks.stone, |(_, id)| id)
            } else if tally.water > 0 && tally.water >= tally.air {
                tally.water_block
            } else {
                BlockId::AIR
            }
        })
        .collect();
    let cell = |x: i32, y: i32, z: i32| cells[index(x, y, z)];
    // Plain ground: solid cells touching only open air (air with nothing solid over it in its
    // column), with no objects, in chunks nobody changed. They're drawn as the far terrain draws the
    // land (terrain.wesl, `far_ground`), so the copy doesn't show as a patch round what's different
    // in it [David, build 5]; only cells touching covered air (a cave mouth's rim, an overhang) and
    // what you've changed keep the blocks' look. (Build 6 asked it of whole columns, so the hillside
    // over a whole test cave kept the blocks' look: David's paler square.) Each plain cell is a
    // stand-in block, its number past the real ones', whose faces all carry `PLAIN` and its top's and
    // side's layers.
    let real_count = job.face_layers.len();
    let stand_ins = real_count * 2 < BlockId::PIECES.0 as usize && job.face_layers.iter().all(|layers| layers[4] < 128 && layers[0] < 128);
    // Each column's highest solid cell (cells above it are open air).
    let roof: Vec<i32> = (0..CELLS * CELLS).map(|k| (0..tall).rev().find(|&z| is_opaque(cell(k % CELLS, k / CELLS, z))).unwrap_or(-1)).collect();
    let covered_air = |p: IVec3| {
        (0..CELLS).contains(&p.x) && (0..CELLS).contains(&p.y) && (0..tall).contains(&p.z) && !is_opaque(cell(p.x, p.y, p.z)) && p.z < roof[(p.x + p.y * CELLS) as usize]
    };
    let plain_ground = |x: i32, y: i32, z: i32| {
        // Copied whole (the near-march experiment): no plain ground, every block drawn as one.
        !job.full
            && tallies[index(x, y, z)].objects <= 0.0
            && !was_changed.contains(&ChunkPos::of_block(IVec3::new(corner.x + x * s, corner.y + y * s, 0)))
            && !sides.iter().any(|&side| covered_air(IVec3::new(x, y, z) + side))
    };
    let plain = |x: i32, y: i32, z: i32| stand_ins && plain_ground(x, y, z);
    let drawn_as = |x: i32, y: i32, z: i32| {
        let id = cell(x, y, z);
        if is_opaque(id) && plain(x, y, z) { BlockId(id.0 + real_count as u16) } else { id }
    };
    let doubled = |table: &Arc<Vec<bool>>| Arc::new(table.iter().chain(table.iter()).copied().collect::<Vec<bool>>());
    let (opaque, see_through, inner_faces) = (doubled(&job.opaque), doubled(&job.see_through), doubled(&job.inner_faces));
    let face_layers = Arc::new(job.face_layers.iter().copied().chain(job.face_layers.iter().map(|layers| [PLAIN | layers[4] | layers[0] << 7; 6])).collect::<Vec<_>>());
    let face_random_rotation = Arc::new(job.face_random_rotation.iter().copied().chain(std::iter::repeat_n([false; 6], real_count)).collect::<Vec<_>>());
    let bar_layers = Arc::new(job.bar_layers.iter().copied().chain(std::iter::repeat_n(None, real_count)).collect::<Vec<_>>());
    // An air cell's daylight: its own air's, and the air hidden in the solid cells beside it.
    let around = [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z];
    let light_of = |x: i32, y: i32, z: i32| -> u8 {
        let (mut light, mut air) = (tallies[index(x, y, z)].light, tallies[index(x, y, z)].air);
        for side in around {
            let p = IVec3::new(x, y, z) + side;
            if (0..CELLS).contains(&p.x) && (0..CELLS).contains(&p.y) && (0..tall).contains(&p.z) && is_opaque(cell(p.x, p.y, p.z)) {
                light += tallies[index(p.x, p.y, p.z)].light;
                air += tallies[index(p.x, p.y, p.z)].air;
            }
        }
        light.checked_div(air).map_or(FULL, |level| level as u8)
    };
    // Placed light (a brazier's: step 1 part 6), flooded through the tile's own blocks as
    // `placed_light.rs` floods it through the near world's (a step dimmer each block, stopped by
    // walls), each cell keeping the brightest that reaches its air.
    let mut glow = vec![0u16; (CELLS * CELLS * tall) as usize];
    let in_tile = |p: IVec3| (0..width).contains(&p.x) && (0..width).contains(&p.y) && (low_cell * s..(high_cell + 1) * s).contains(&p.z);
    // Made from the real chunks: their light as the near world has it, lights beyond the tile's
    // edge too.
    for &(at, light) in &job.glows {
        let at = at - IVec3::new(corner.x, corner.y, 0);
        if in_tile(at) {
            let k = index(at.x / s, at.y / s, at.z.div_euclid(s) - low_cell);
            glow[k] = super::placed_light::brighter(glow[k], light);
        }
    }
    for light in &job.lights {
        let reach = light.reach.clamp(1, super::placed_light::MOST_REACH);
        let start = light.cell - IVec3::new(corner.x, corner.y, 0);
        if !in_tile(start) {
            continue;
        }
        let mut queue = VecDeque::from([(start, 0)]);
        let mut seen = HashSet::from([start]);
        while let Some((at, steps)) = queue.pop_front() {
            let level = light.colour.map(|full| full * (reach - steps) as u32 / reach as u32);
            if level == [0; 3] {
                continue;
            }
            let k = index(at.x / s, at.y / s, at.z.div_euclid(s) - low_cell);
            glow[k] = super::placed_light::brighter(glow[k], super::placed_light::pack(level));
            for side in sides {
                let next = at + side;
                if steps + 1 < reach && in_tile(next) && !is_opaque(block_at(next.x, next.y, next.z)) && seen.insert(next) {
                    queue.push_back((next, steps + 1));
                }
            }
        }
    }
    // An air cell's glow: its own air's, and that hidden in the solid cells beside it (as its daylight).
    let glow_of = |x: i32, y: i32, z: i32| -> u16 {
        let mut light = glow[index(x, y, z)];
        for side in around {
            let p = IVec3::new(x, y, z) + side;
            if (0..CELLS).contains(&p.x) && (0..CELLS).contains(&p.y) && (0..tall).contains(&p.z) && is_opaque(cell(p.x, p.y, p.z)) {
                light = super::placed_light::brighter(light, glow[index(p.x, p.y, p.z)]);
            }
        }
        light
    };
    // The cells as the march reads them (`CopyCells`).
    let packed: Vec<u32> = (0..CELLS * CELLS * tall)
        .map(|k| {
            let (x, y, z) = (k % CELLS, (k / CELLS) % CELLS, k / (CELLS * CELLS));
            let id = cell(x, y, z);
            if is_opaque(id) {
                id.0 as u32 & 0xFFF | CELL_SOLID | if plain_ground(x, y, z) { CELL_PLAIN } else { 0 }
            } else if id != BlockId::AIR {
                id.0 as u32 & 0xFFF | CELL_WATER
            } else {
                (light_of(x, y, z) as u32) << 16 | (glow_of(x, y, z) as u32) << 20
            }
        })
        .collect();
    let levels = if job.full { copy_levels(&packed, tall, job.share) } else { Vec::new() };
    let copy_cells = Arc::new(CopyCells { corner: IVec3::new(corner.x, corner.y, low_cell * s), scale: s, tall, full: job.full, cells: packed, levels });
    // Drawn by the march, the cells are all a copy needs (step 1 part 7): meshing it was most of the
    // time a copy took (200 to 320 ms at 8 blocks a cell), and frames hitched while copies were made
    // after a flight (`docs/caves.md` 13.24).
    if !job.mesh {
        let shows = !hot.is_empty() || job.force;
        // Made from the real chunks, it's there for being past the line, not for differing from the
        // land: none of its chunks is counted as showing (kept, they'd pile up as you flew).
        if job.real {
            hot.clear();
        }
        return Made { sections: Vec::new(), cells: shows.then_some(copy_cells), hot: hot.into_iter().collect(), ms: started.elapsed().as_secs_f32() * 1000.0 };
    }

    // 4. Meshed, a section of cells at a time, from the bottom.
    // The plain ground beside the tile: the lowest of a cell's corners and middle, and of the cell
    // inside beside it, so on a steep slope its skirt reaches down past the coarser or finer copy or
    // the far terrain next to it (from its middle alone, sky showed through the seam between two
    // copies: David, build 8).
    // Worked out once a tile, for the ring of cells round it (asked for every level of every
    // section, it made copies five times slower: build 9).
    let around_tile = CELLS + 2;
    let beside: Vec<i32> = (0..around_tile * around_tile)
        .map(|k| {
            let (x, y) = (k % around_tile - 1, k / around_tile - 1);
            if (0..CELLS).contains(&x) && (0..CELLS).contains(&y) {
                return i32::MIN;
            }
            let inside = IVec2::new(x.clamp(0, CELLS - 1), y.clamp(0, CELLS - 1));
            [IVec2::new(x, y), inside]
                .iter()
                .flat_map(|&c| [(0, 0), (s - 1, 0), (0, s - 1), (s - 1, s - 1), (s / 2, s / 2)].map(|(dx, dy)| corner + c * s + IVec2::new(dx, dy)))
                .filter_map(|at| job.map.column(at.x, at.y).map(|(z, _)| z))
                .min()
                .unwrap_or(i32::MIN)
        })
        .collect();
    let ground_beside = |x: i32, y: i32| beside[((x + 1) + (y + 1) * around_tile) as usize];
    let mut sections = Vec::new();
    for first in (0..tall).step_by(CELLS as usize) {
        let blocks: Vec<BlockId> = (0..CELLS * CELLS * CELLS)
            .map(|k| {
                let z = first + k / (CELLS * CELLS);
                if z < tall { drawn_as(k % CELLS, (k / CELLS) % CELLS, z) } else { BlockId::AIR }
            })
            .collect();
        // The cells just outside: below the copied range solid, above it open; beside the tile, the
        // plain ground there, open for `SKIRT` cells down from its surface.
        let solid_outside = |p: IVec3| -> bool {
            let z = first + p.z;
            if z < 0 {
                return true;
            }
            if z >= tall {
                return false;
            }
            if (0..CELLS).contains(&p.x) && (0..CELLS).contains(&p.y) {
                return is_opaque(cell(p.x, p.y, z));
            }
            (low_cell + z + 1) * s - 1 <= ground_beside(p.x, p.y) - SKIRT * s
        };
        let mut outside = [[0u32; 32]; 6];
        for axis in 0..3 {
            let (u, v) = plane_axes(axis);
            for positive in [false, true] {
                for b in 0..CELLS {
                    for a in 0..CELLS {
                        let mut p = [0; 3];
                        p[axis] = if positive { CELLS } else { -1 };
                        p[u] = a;
                        p[v] = b;
                        if solid_outside(IVec3::from_array(p)) {
                            outside[face_index(axis, positive)][b as usize] |= 1 << a;
                        }
                    }
                }
            }
        }
        // Daylight of the cells and the one round them: open beside the tile and above, black below.
        let around = CELLS + 2;
        let mut levels = vec![0u8; (around * around * around) as usize];
        for z in -1..=CELLS {
            for y in -1..=CELLS {
                for x in -1..=CELLS {
                    let at = first + z;
                    let level = if at < 0 {
                        0
                    } else if at >= tall || !(0..CELLS).contains(&x) || !(0..CELLS).contains(&y) || is_opaque(cell(x, y, at)) {
                        FULL
                    } else {
                        light_of(x, y, at)
                    };
                    levels[((x + 1) + (y + 1) * around + (z + 1) * around * around) as usize] = level;
                }
            }
        }
        let input = MeshInput {
            blocks,
            outside,
            outside_outline: [[0; 32]; 6],
            outside_grid: [[0; 32]; 6],
            opaque: opaque.clone(),
            see_through: see_through.clone(),
            inner_faces: inner_faces.clone(),
            face_layers: face_layers.clone(),
            face_random_rotation: face_random_rotation.clone(),
            pieces: Vec::new(),
            bar_layers: bar_layers.clone(),
            beyond_pieces: HashMap::new(),
            light: None,
            reach,
            placed: HashMap::new(),
            only_see_through: false,
        };
        let output = mesh_section_lit(&input, Daylight(Some(levels)));
        if !output.indices.is_empty() {
            sections.push((IVec3::new(corner.x, corner.y, (low_cell + first) * s), output));
        }
    }
    if hot.is_empty() && !job.force {
        sections.clear();
    }
    let cells = (!sections.is_empty()).then_some(copy_cells);
    Made { sections, cells, hot: hot.into_iter().collect(), ms: started.elapsed().as_secs_f32() * 1000.0 }
}

/// Keeps the copies up to date round you: which tiles are wanted, making them in the background,
/// drawing those made, and retiring those no longer wanted once their replacements are in.
#[allow(clippy::too_many_arguments)]
pub fn update_far_copies(
    mut commands: Commands,
    (settings, distant_settings): (Res<FarCopySettings>, Res<super::distant::DistantSettings>),
    render: Res<RenderSettings>,
    (streaming, placed, mut meshing): (Res<StreamSettings>, Res<super::placed_light::PlacedLights>, ResMut<super::meshing::MeshingState>),
    distant: Res<super::distant::DistantState>,
    terrain: Res<Terrain>,
    caves: Res<Caves>,
    world: Res<VoxelWorld>,
    assets: Option<Res<TerrainAssets>>,
    objects: Res<ObjectRegistry>,
    registry: Res<BlockRegistry>,
    (camera, render_origin): (CameraLens, Res<crate::coords::RenderOrigin>),
    window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<FarCopies>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let _timed = super::Timed::new("far copies");
    let (Some(map), Some(assets)) = (terrain.compiled().cloned(), assets) else { return };
    if !settings.on {
        if !state.tiles.is_empty() || !state.retiring.is_empty() || !state.making.is_empty() {
            state.clear(&mut commands);
        }
        return;
    }
    let plan = caves.0.clone();
    let real = marches_real(&settings, &distant_settings, &render, &streaming);
    // Made from another map, plan, set of blocks or share, or the real chunks past the line taken
    // up or let go: every tile again.
    let same = state.map.as_ref().is_some_and(|old| Arc::ptr_eq(old, &map))
        && state.plan.as_ref().is_some_and(|old| Arc::ptr_eq(old, &plan))
        && state.revision == assets.revision
        && state.share == settings.solid_share
        && state.meshed == Some(!settings.marched(&distant_settings))
        && state.real == real;
    if !same {
        state.clear(&mut commands);
        state.near_ground = near_ground(&plan, &map);
        state.caves = plan.tunnels.iter().map(CavePlan::chunks_near).collect();
        state.map = Some(map.clone());
        state.plan = Some(plan.clone());
        state.revision = assets.revision;
        state.share = settings.solid_share;
        state.meshed = Some(!settings.marched(&distant_settings));
        state.real = real;
        state.known_changed = world.changed_chunks().clone();
    }
    // Real chunks whose blocks changed, came or went: the tiles made from them are made again.
    let touched = std::mem::take(&mut meshing.touched);
    if real {
        let again: Vec<TileKey> = state.full.iter().copied().filter(|key| (state.tiles.contains_key(key) || state.making.contains_key(key)) && key.chunks().any(|pos| touched.contains(&pos))).collect();
        for key in again {
            state.retire(key);
            state.queue.push(key);
        }
    }
    // Chunks changed since: the tiles holding them are made again.
    let fresh: Vec<ChunkPos> = world.changed_chunks().difference(&state.known_changed).copied().collect();
    for &pos in &fresh {
        state.known_changed.insert(pos);
        for scale in [1, 2, 4, 8] {
            let key = TileKey::of(pos, scale);
            state.retire(key);
            state.tiles.remove(&key);
        }
    }

    // The copies' material: the terrain's, marked as a copy's.
    if let Some(main) = materials.get(&assets.material).cloned() {
        let mut copy = main;
        copy.settings.options.z = 1.0;
        match state.material.clone() {
            None => state.material = Some(materials.add(copy)),
            Some(handle) => {
                let differs = materials.get(&handle).is_none_or(|now| {
                    now.settings != copy.settings || now.light != copy.light || now.textures != copy.textures || now.light_map != copy.light_map || now.near_map != copy.near_map || now.near_cones != copy.near_cones
                });
                if differs && let Some(mut now) = materials.get_mut(&handle) {
                    *now = copy;
                }
            }
        }
    }
    let Some(material) = state.material.clone() else { return };

    let Some((transform, projection)) = camera.iter().next() else { return };
    let at = render_origin.to_game(transform.translation);
    let fov = match projection {
        Projection::Perspective(perspective) => perspective.fov,
        _ => 1.2,
    };
    let height = window.iter().next().map_or(1080.0, |window| window.resolution.height());
    let home = IVec2::new((at.x / CHUNK_SIZE as f32).floor() as i32, (at.y / CHUNK_SIZE as f32).floor() as i32);
    // Where the real chunks end: the render distance, the streaming circle, and where the far view
    // starts (`RenderSettings::march_from`), whichever is nearest.
    let (edge, radius) = (render.render_distance_chunks, streaming.radius.min(render.march_from));
    let limit = MASK_REACH.min((settings.reach / CHUNK_SIZE as f32).ceil() as i32);

    // Which tiles are wanted: worked out again when you move into another chunk, or what's known
    // changes.
    let now = (home, edge, radius, [settings.sharpness.to_bits(), settings.reach.to_bits()], world.changed_chunks().len(), state.hot.len(), (fov * 1000.0) as u32 ^ (height as u32) << 16);
    if state.wanted_for != Some(now) {
        state.wanted_for = Some(now);
        let offset = |pos: ChunkPos| IVec2::new(pos.x - home.x, pos.y - home.y);
        let mut candidates: HashMap<ChunkPos, bool> = HashMap::new();
        for &pos in world.changed_chunks().iter().chain(state.near_ground.iter()) {
            candidates.insert(pos, false);
        }
        for pos in &state.hot {
            for (x, y) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                candidates.entry(ChunkPos::new(pos.x + x, pos.y + y)).or_insert(true);
            }
        }
        // A cave that shows anywhere is copied all along.
        for cave in state.caves.iter().filter(|cave| cave.iter().any(|pos| state.hot.contains(pos))) {
            for &pos in cave {
                candidates.insert(pos, true);
            }
        }
        // The near-march experiment (`docs/far-view.md`): with the far view's line pulled in past where
        // the real chunks would end, every chunk in a band `FULL_BAND` chunks wide just past it is
        // copied whole, its plain ground as blocks too, so the march draws the blocks there. With the
        // real chunks drawn by the march (R2), the band reaches where they end, and they're copied
        // from themselves.
        let mut full_chunks = HashSet::new();
        if settings.marched(&distant_settings) && render.march_from < streaming.radius {
            let out = if real { streaming.radius } else { render.march_from + FULL_BAND };
            for y in -out..=out {
                for x in -out..=out {
                    let reach = IVec2::new(x, y);
                    if reach.length_squared() <= out * out && !real_chunk(reach, edge, radius) && real_chunk(reach, edge, streaming.radius) {
                        let pos = ChunkPos::new(home.x + x, home.y + y);
                        candidates.insert(pos, true);
                        full_chunks.insert(pos);
                    }
                }
            }
        }
        let mut wanted: HashMap<TileKey, bool> = HashMap::new();
        let mut full = HashSet::new();
        for (pos, force) in candidates {
            let reach = offset(pos);
            if real_chunk(reach, edge, radius) || reach.x.abs().max(reach.y.abs()) > limit {
                continue;
            }
            // The coarsest tile that fits wholly outside the real chunks, inside the mask, and is
            // coarse enough for its distance (a drawn tile keeps its size until well past the line).
            for scale in [8, 4, 2, 1] {
                let key = TileKey::of(pos, scale);
                let (nearest, furthest) = key.chunk_range(home);
                if real_chunk(nearest, edge, radius) || furthest > limit {
                    continue;
                }
                // Drawn or being made: kept until well past the line, so moving doesn't throw it away.
                let drawn = state.tiles.contains_key(&key) || state.making.contains_key(&key);
                let distance = key.middle().distance(at.truncate()) * if drawn { SWITCH_PAST } else { 1.0 / SWITCH_PAST };
                if scale == 1 || scale <= scale_for(distance, settings.sharpness, fov, height) {
                    *wanted.entry(key).or_insert(false) |= force;
                    if full_chunks.contains(&pos) {
                        full.insert(key);
                    }
                    break;
                }
            }
        }
        let gone: Vec<TileKey> = state.tiles.keys().chain(state.making.keys()).copied().filter(|key| !wanted.contains_key(key)).collect();
        for key in gone {
            state.retire(key);
        }
        let mut queue: Vec<TileKey> = wanted.keys().copied().filter(|key| !state.tiles.contains_key(key) && !state.making.contains_key(key)).collect();
        queue.sort_by(|a, b| b.middle().distance(at.truncate()).total_cmp(&a.middle().distance(at.truncate())));
        if wanted.len() != state.wanted.len() {
            info!("Far copies: {} tiles wanted, {} to make", wanted.len(), queue.len());
        }
        // A tile becoming whole, or no longer, is made again.
        let switched: Vec<TileKey> = full.symmetric_difference(&state.full).copied().filter(|key| state.tiles.contains_key(key)).collect();
        for key in switched {
            state.retire(key);
            queue.push(key);
        }
        state.full = full;
        state.queue = queue;
        state.wanted = wanted;
        state.update_covered();
    }

    // Finished tiles are drawn.
    let mut done = Vec::new();
    state.making.retain(|&key, task| match check_ready(task) {
        Some(made) => {
            done.push((key, made));
            false
        }
        None => true,
    });
    let finished = !done.is_empty();
    for (key, made) in done {
        state.times.push_back(made.ms);
        if state.times.len() > 64 {
            state.times.pop_front();
        }
        info!("Far copy {} blocks a cell at ({}, {}): {:.0} ms, {} sections, {} chunks showing", key.scale, key.x * key.scale * CHUNK_SIZE, key.y * key.scale * CHUNK_SIZE, made.ms, made.sections.len(), made.hot.len());
        state.hot.extend(made.hot);
        if made.sections.is_empty() && made.cells.is_none() {
            state.tiles.insert(key, Tile::Empty);
            continue;
        }
        let (mut entities, mut faces, mut bytes) = (Vec::new(), 0, 0);
        for (origin, output) in made.sections {
            faces += output.quads;
            bytes += output.vertices.len() * 12 + output.indices.len() * 4;
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
            mesh.insert_attribute(ATTRIBUTE_PACKED_VOXEL, output.vertices);
            mesh.insert_indices(Indices::U32(output.indices));
            let size = CELLS as f32;
            entities.push(
                commands
                    .spawn((
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d::<TerrainMaterial>(material.clone()),
                        Transform::from_translation(render_origin.to_bevy(origin.as_vec3())).with_scale(Vec3::splat(key.scale as f32)),
                        Aabb::from_min_max(Vec3::new(0.0, 0.0, -size), Vec3::new(size, size, 0.0)),
                        FarCopy,
                    ))
                    .id(),
            );
        }
        state.tiles.insert(key, Tile::Drawn { entities, faces, bytes, cells: made.cells });
    }

    // Tiles no longer wanted stay until every wanted tile over them is in, and the real chunks over
    // them are drawn and meshed (coming close, the cave vanished for a second while they were made: David,
    // build 5).
    let mut retired = false;
    if !state.retiring.is_empty() {
        let drawn = distant.shown();
        let unready = |key: &TileKey| key.chunks().any(|pos| real_chunk(IVec2::new(pos.x - home.x, pos.y - home.y), edge, radius) && !drawn.contains(&pos));
        let area = |key: &TileKey| {
            let width = CHUNK_SIZE * key.scale;
            (key.x * width, key.y * width, width)
        };
        let overlap = |a: (i32, i32, i32), b: (i32, i32, i32)| a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.2 && b.1 < a.1 + a.2;
        let waiting: Vec<(i32, i32, i32)> = state.wanted.keys().filter(|key| !state.tiles.contains_key(key)).map(area).collect();
        let mut done_with = Vec::new();
        state.retiring.retain(|(key, entities, _)| {
            let still = waiting.iter().any(|&w| overlap(area(key), w)) || unready(key);
            if !still {
                done_with.extend(entities.iter().copied());
            }
            still
        });
        retired = !done_with.is_empty();
        for entity in done_with {
            commands.entity(entity).despawn();
        }
    }
    if finished || retired {
        state.update_covered();
    }

    // Start more, nearest first.
    let pool = AsyncComputeTaskPool::get();
    let mut not_yet = Vec::new();
    // Twice as many while the real chunks past the line are copied: their meshing, no longer done,
    // left the processor that time.
    let most = if real { MOST_AT_ONCE * 2 } else { MOST_AT_ONCE };
    while state.making.len() < most {
        let Some(key) = state.queue.pop() else { break };
        if state.tiles.contains_key(&key) || state.making.contains_key(&key) || !state.wanted.contains_key(&key) {
            continue;
        }
        // Made from the real chunks (R2): each as it is, once streaming has brought it in (one past
        // where the real chunks end, at the edge of a coarser tile, from the recipe as before).
        let from_real = real && state.full.contains(&key);
        let streamed = |pos: ChunkPos| real_chunk(IVec2::new(pos.x - home.x, pos.y - home.y), edge, streaming.radius);
        if from_real && key.chunks().any(|pos| streamed(pos) && world.chunk(pos).is_none()) {
            not_yet.push(key);
            continue;
        }
        let mut changed = HashMap::new();
        let (mut shapes, mut lights, mut glows) = (Vec::new(), Vec::new(), Vec::new());
        if from_real {
            for pos in key.chunks() {
                if let Some(chunk) = world.chunk(pos) {
                    changed.insert(pos, chunk.clone());
                }
            }
            let width = CHUNK_SIZE * key.scale;
            let low = IVec3::new(key.x * width, key.y * width, WORLD_BOTTOM);
            glows = placed.lit_in(low, low + IVec3::new(width - 1, width - 1, crate::world::WORLD_HEIGHT));
        }
        for pos in key.chunks().filter(|_| !from_real) {
            if world.changed_chunks().contains(&pos)
                && let Some(chunk) = world.changed_chunk(pos)
            {
                for object in chunk.objects() {
                    if let Some(def) = objects.by_name(&object.name) {
                        shapes.extend(shape_boxes(object, def));
                    }
                }
                lights.extend(super::objects::lights_here(&objects, chunk.objects()));
                changed.insert(pos, chunk.clone());
            }
        }
        let job = CopyJob {
            key,
            map: map.clone(),
            plan: plan.clone(),
            blocks: TerrainBlocks::from_registry(&registry),
            changed,
            objects: shapes,
            lights,
            opaque: assets.opaque.clone(),
            see_through: assets.see_through.clone(),
            inner_faces: assets.inner_faces.clone(),
            face_layers: assets.face_layers.clone(),
            face_random_rotation: assets.face_random_rotation.clone(),
            bar_layers: assets.bar_layers.clone(),
            reach: render.sky_reach,
            share: settings.solid_share,
            force: state.wanted[&key],
            mesh: !settings.marched(&distant_settings),
            full: state.full.contains(&key),
            real: from_real,
            glows,
        };
        state.making.insert(key, pool.spawn(async move { make_copy(job) }));
    }
    // Those waiting on streaming go back, nearest last as before.
    state.queue.extend(not_yet.into_iter().rev());
}

/// The Render page's "Far copies" controls (greyed while copies are off), and what's drawn.
pub fn controls(ui: &mut bevy_egui::egui::Ui, world: &mut World) {
    {
        let mut settings = world.resource_mut::<FarCopySettings>();
        ui.checkbox(&mut settings.on, "far copies").on_hover_text("Off: past the real chunks there's only the smooth far surface, as before (to compare).");
        let on = settings.on;
        ui.add_enabled_ui(on, |ui| {
            ui.checkbox(&mut settings.march, "drawn by the march (new)").on_hover_text(
                "The copies' cells drawn by the march, with the far land, instead of as blocks of their own (docs/far-view.md).\n\
                 Only while the far land is drawn by the march too. Then they're made without their blocks, which is much\n\
                 quicker; flipping it makes them all again (to compare).",
            );
            let marched = settings.march;
            ui.add_enabled_ui(marched, |ui| {
                ui.checkbox(&mut settings.real, "the real chunks past the line too (new)").on_hover_text(
                    "With \"the march starts at\" pulled in, the real chunks past it are copied straight from themselves and drawn by the\n\
                     march, no longer meshed: only their see-through blocks and half blocks still are (docs/far-view.md, R2).\n\
                     Off: as build 34, a band of copies made from the recipe, the real chunks meshed underneath (to compare).",
                );
                ui.checkbox(&mut settings.shaded, "the march shaded as the meshes (new)").on_hover_text(
                    "The blocks the march draws (past the line) lit and textured as the meshes are: the same textures the same way\n\
                     round, the same random turn, the ground light (the lie of the land, the height tint, the feet of steps) and the\n\
                     light underground, from one recipe both share. Off: as build 35, the march's own shading (to compare).",
                );
                ui.checkbox(&mut settings.sized, "far cells sized per pixel (new)").on_hover_text(
                    "The blocks the march draws, drawn at the size each pixel can show: further off, 2, 4 or 8 to a side as one cell\n\
                     (textured as a copy's own cells of that size), changing a cell at a time, so no squares where one size meets the next.\n\
                     \"Who drew this\" shows the sizes: green a block a cell, then yellow, orange and red. Off: as build 37.",
                );
                ui.add_enabled_ui(settings.sized, |ui| {
                    ui.add(crate::debug::Slider::new(&mut settings.pixels, 1.0..=8.0).text("pixelization (pixels a far cell)")).on_hover_text(
                        "About how many pixels across a far cell grows before the next coarser size takes over: higher is blockier\n\
                         further off (and quicker to draw). It never gets finer than the copies keep (\"sharpness\").",
                    );
                });
                ui.checkbox(&mut settings.bands, "contour bands (new)").on_hover_text(
                    "The far land, and the march's cells of 2 blocks or more on the open ground, drawn with the steps' sides in bands at\n\
                     each whole block of height (the copies' from their own blocks), so the steps look alike near and far: a band\n\
                     thinner than a pixel a faint line, several steps to a pixel their plain mix. Off: as build 42.",
                );
                ui.checkbox(&mut settings.reference, "reference (slow, for looking)").on_hover_text(
                    "For looking, not playing: 16 rays a pixel through the copies' blocks at their own size (no coarser cells,\n\
                     no bands), so each pixel shows what the blocks really come to from here. What the far land's look should\n\
                     match. Slow: play with it off. \"sharpness\" at 4 copies more of it a block a cell. Not saved on.",
                );
            });
            ui.add(crate::debug::Slider::new(&mut settings.sharpness, 1.0..=4.0).text("sharpness")).on_hover_text(
                "How crisp the copies are: a copy's cell is about 4 ÷ this pixels on screen (1: 4 pixels, soft and cheap; 4: a pixel\n\
                 a cell, crisp and dearer). Finer than a pixel only shimmers, so it stops there.",
            );
            ui.add(crate::debug::Slider::new(&mut settings.solid_share, 0.125..=0.875).text("how much must be solid")).on_hover_text(
                "How much of a copy's cell must be solid for it to be drawn solid. Lower keeps thin walls and pillars; higher keeps\n\
                 small holes (cave mouths) open. Changing it makes every copy again.",
            );
            ui.add(crate::debug::Slider::new(&mut settings.reach, 1_024.0..=4_000.0).text("how far copies reach (blocks)")).on_hover_text(
                "Past this the smooth far surface draws everything, as before. A cave mouth is about a pixel by 4,000 blocks.",
            );
        });
        if ui.button("Reset far copy settings").clicked() {
            *settings = crate::debug::page_default("render", "far_copies");
        }
    }
    ui.small(world.resource::<FarCopies>().summary());
    ui.small(world.resource::<super::march::MarchState>().summary());
    // Stop 4's measurement (`docs/caves.md` 13.23): one line in the log for each setting tried.
    if ui.button("Write the frame rate to the log").on_hover_text("The frame rate over the last few seconds, with these settings and the copies drawn: one line in the log for each setting tried. Turn slowly through a full circle first.").clicked() {
        let settings = world.resource::<FarCopySettings>().clone();
        let copies = if settings.on { format!("on, sharpness {:.1}, must be solid {:.2}, reach {:.0}", settings.sharpness, settings.solid_share, settings.reach) } else { "off".to_string() };
        let line = format!("Far copies measured ({copies}): {}. {}", super::drawing_line(world), world.resource::<FarCopies>().summary());
        info!("{line}");
    }
}

/// A copy's cells made from `map` and `plan`, for the march's tests (`march.rs`).
#[cfg(test)]
pub(super) fn cells_for_tests(key: TileKey, map: CompiledMap, plan: CavePlan, full: bool) -> Option<Arc<CopyCells>> {
    let mut job = tests::job(key, map, plan, HashMap::new(), 0.5);
    (job.force, job.full) = (true, full);
    make_copy(job).cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::BlockRegistry;
    use crate::world::TestCaveSettings;

    /// A job for the tile `key` over `map` with `plan`, its changed chunks given.
    pub(super) fn job(key: TileKey, map: CompiledMap, plan: CavePlan, changed: HashMap<ChunkPos, Chunk>, share: f32) -> CopyJob {
        let registry = BlockRegistry::default();
        let blocks = TerrainBlocks::from_registry(&registry);
        let count = registry.defs().len().max(64);
        let opaque: Vec<bool> = (0..count).map(|id| id != BlockId::AIR.0 as usize && id != blocks.water.0 as usize).collect();
        let see_through: Vec<bool> = (0..count).map(|id| id == blocks.water.0 as usize).collect();
        CopyJob {
            key,
            map: Arc::new(map),
            plan: Arc::new(plan),
            blocks,
            changed,
            objects: Vec::new(),
            lights: Vec::new(),
            opaque: Arc::new(opaque),
            see_through: Arc::new(see_through),
            inner_faces: Arc::new(vec![false; count]),
            face_layers: Arc::new(vec![[0; 6]; count]),
            face_random_rotation: Arc::new(vec![[false; 6]; count]),
            bar_layers: Arc::new(vec![None; count]),
            reach: 8,
            share,
            force: false,
            mesh: true,
            full: false,
            real: false,
            glows: Vec::new(),
        }
    }

    /// The highest point any face of a copy reaches (blocks), and its darkest face's daylight.
    fn highest_and_darkest(made: &Made, scale: i32) -> (f32, u32) {
        let mut highest = f32::MIN;
        let mut darkest = 15;
        for (origin, output) in &made.sections {
            for vertex in &output.vertices {
                let z = ((vertex[0] >> 14) & 127) as f32 / 2.0;
                highest = highest.max(origin.z as f32 + z * scale as f32);
                darkest = darkest.min((vertex[0] >> 28) & 15);
            }
        }
        (highest, darkest)
    }

    #[test]
    fn a_mouth_in_a_hillside_is_copied_darker_round_it_and_plain_ground_is_left_to_the_sheet() {
        // Ground rising steeply one way (12 blocks a pixel of 32).
        let width = 64;
        let heights: Vec<f32> = (0..width * width).map(|k| (230.0 + ((k / width) as f32 - 32.0) * 3.0 + ((k % width) as f32 - 32.0) * 12.0).clamp(140.0, 330.0)).collect();
        let map = CompiledMap::of_heights(width, 32.0, &heights);
        let key = TileKey { scale: 2, x: 0, y: 0 };
        // Plain: nothing shows, so nothing is drawn.
        let plain = make_copy(job(key, CompiledMap::of_heights(width, 32.0, &heights), CavePlan::default(), HashMap::new(), 0.5));
        assert!(plain.sections.is_empty() && plain.hot.is_empty());
        // A test cave worming uphill into the slope from (20, 20).
        let ground = |x: i32, y: i32| map.column(x, y).unwrap().0 as f32;
        let uphill = Vec2::new(ground(23, 20) - ground(17, 20), ground(20, 23) - ground(20, 17)).normalize();
        let mouth = Vec3::new(20.5 - uphill.x, 20.5 - uphill.y, ground(20, 20) + 1.0 + 2.0 * 0.6);
        let tunnel = TestCaveSettings::default().tunnel(mouth, uphill, 3);
        let made = make_copy(job(key, map, CavePlan::new(vec![tunnel]), HashMap::new(), 0.5));
        assert!(made.hot.contains(&ChunkPos::new(0, 0)), "{:?}", made.hot);
        assert!(!made.sections.is_empty());
        // Daylight fades into it: some faces are darker than full.
        let (_, darkest) = highest_and_darkest(&made, 2);
        assert!(darkest < 15, "{darkest}");
        // The open hillside round it is plain ground (drawn as the far terrain draws the land); the
        // mouth's rim, touching the cave's air, isn't.
        let plain = |made: &Made| made.sections.iter().flat_map(|(_, output)| output.vertices.iter()).map(|vertex| vertex[1] & PLAIN != 0).collect::<Vec<_>>();
        let faces = plain(&made);
        assert!(faces.iter().any(|&p| p) && faces.iter().any(|&p| !p));
        // The cells the march reads: plain ground and not, and air in the cave darker than full.
        let cells = made.cells.as_ref().expect("cells for the march");
        assert_eq!(cells.cells.len(), (CELLS * CELLS * cells.tall) as usize);
        assert!(cells.cells.iter().any(|&c| c & CELL_SOLID != 0 && c & CELL_PLAIN != 0));
        assert!(cells.cells.iter().any(|&c| c & CELL_SOLID != 0 && c & CELL_PLAIN == 0));
        assert!(cells.cells.iter().any(|&c| (c >> 12) & 3 == 0 && (c >> 16) & 15 < 15));
        assert!(cells.cells.iter().all(|&c| (c >> 12) & 3 != 0 || c >> 20 == 0), "no glow without a light");
        // Made for the march alone: the same cells, and no blocks.
        let mut for_march = job(key, CompiledMap::of_heights(width, 32.0, &heights), CavePlan::new(vec![TestCaveSettings::default().tunnel(mouth, uphill, 3)]), HashMap::new(), 0.5);
        for_march.mesh = false;
        let marched = make_copy(for_march);
        assert!(marched.sections.is_empty());
        assert_eq!(marched.cells.expect("cells for the march").cells, cells.cells);
        // Copied whole (the near-march experiment): the same solid cells, none plain ground.
        let mut whole = job(key, CompiledMap::of_heights(width, 32.0, &heights), CavePlan::new(vec![TestCaveSettings::default().tunnel(mouth, uphill, 3)]), HashMap::new(), 0.5);
        (whole.mesh, whole.full) = (false, true);
        let whole = make_copy(whole).cells.expect("cells");
        assert!(whole.full && whole.cells.iter().all(|&c| c & CELL_PLAIN == 0));
        assert_eq!(whole.cells.iter().filter(|&&c| c & CELL_SOLID != 0).count(), cells.cells.iter().filter(|&&c| c & CELL_SOLID != 0).count());
        // A green light just inside the mouth: the cave's air round it glows green, fading, and none red.
        let mut lit = job(key, CompiledMap::of_heights(width, 32.0, &heights), CavePlan::new(vec![TestCaveSettings::default().tunnel(mouth, uphill, 3)]), HashMap::new(), 0.5);
        let inside = (mouth + uphill.extend(0.0) * 2.0).floor().as_ivec3();
        lit.lights = vec![super::super::placed_light::LightFrom { object: inside * 16, cell: inside, colour: [0, 15, 0], reach: 6 }];
        let glows: Vec<u32> = make_copy(lit).cells.expect("cells").cells.iter().filter(|&&c| (c >> 12) & 3 == 0).map(|&c| c >> 20).filter(|&g| g != 0).collect();
        assert!(!glows.is_empty(), "the air round the light glows");
        assert!(glows.iter().all(|&g| g & 15 == 0 && (g >> 4) & 15 > 0), "green only: {glows:?}");
        assert!(glows.iter().any(|&g| (g >> 4) & 15 < 15), "fading: {glows:?}");
        // In a changed chunk nothing is plain ground.
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let changed = CompiledMap::of_heights(width, 32.0, &heights).chunk(ChunkPos::new(0, 0), &blocks, &CavePlan::default()).unwrap();
        let made = make_copy(job(key, CompiledMap::of_heights(width, 32.0, &heights), CavePlan::default(), HashMap::from([(ChunkPos::new(0, 0), changed)]), 0.5));
        let ground_of_changed: Vec<bool> = made.sections.iter().flat_map(|(_, output)| output.vertices.iter()).filter(|vertex| {
            // Faces in chunk (0, 0): the tile's first 16 cells each way (2 blocks a cell).
            let (x, y) = ((vertex[0] & 127) / 2, ((vertex[0] >> 7) & 127) / 2);
            x < 16 && y < 16
        }).map(|vertex| vertex[1] & PLAIN != 0).collect();
        assert!(!ground_of_changed.is_empty() && ground_of_changed.iter().all(|&p| !p));
    }

    #[test]
    fn a_copy_of_the_real_chunks_leaves_see_through_blocks_to_the_meshes_and_takes_their_light() {
        let width = 16;
        let map = || CompiledMap::of_heights(width, 32.0, &vec![200.0; width * width]);
        let ground = map().column(10, 10).unwrap().0;
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        // A see-through block (the test's water) on the ground under a roof (open to the sky, a
        // copy counts it as air either way), and a lit cell beside it.
        let mut chunk = map().chunk(ChunkPos::new(0, 0), &blocks, &CavePlan::default()).unwrap();
        let glass = IVec3::new(10, 10, ground + 1);
        chunk.set(glass, blocks.water);
        chunk.set(glass + IVec3::Z * 2, blocks.stone);
        let key = TileKey { scale: 1, x: 0, y: 0 };
        let made = |real: bool| {
            let mut job = job(key, map(), CavePlan::default(), HashMap::from([(ChunkPos::new(0, 0), chunk.clone())]), 0.5);
            (job.mesh, job.full, job.force, job.real) = (false, true, true, real);
            job.glows = if real { vec![(glass + IVec3::X, 12 << 4)] } else { Vec::new() };
            make_copy(job).cells.expect("cells")
        };
        let (recipe, real) = (made(false), made(true));
        let at = |cells: &CopyCells, p: IVec3| cells.cells[(p.x + p.y * CELLS + (p.z - cells.corner.z) * CELLS * CELLS) as usize];
        // From the recipe it's a see-through cell; from the real chunks, air (the near meshes draw it).
        assert_eq!(at(&recipe, glass) & CELL_WATER, CELL_WATER);
        assert_eq!((at(&real, glass) >> 12) & 3, 0);
        // The light beside it as the near world has it.
        assert_eq!(at(&real, glass + IVec3::X) >> 20, 12 << 4);
        // Copied less deep: only as far as daylight reaches under the ground.
        assert!(real.corner.z > recipe.corner.z, "{} over {}", real.corner.z, recipe.corner.z);
    }

    #[test]
    fn a_thin_pillar_shows_when_little_must_be_solid_and_not_when_half_must() {
        let width = 16;
        let map = || CompiledMap::of_heights(width, 32.0, &vec![200.0; width * width]);
        let ground = map().column(10, 10).unwrap().0;
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let mut chunk = map().chunk(ChunkPos::new(0, 0), &blocks, &CavePlan::default()).unwrap();
        for z in ground + 1..=ground + 20 {
            chunk.set(IVec3::new(10, 10, z), blocks.stone);
        }
        let key = TileKey { scale: 2, x: 0, y: 0 };
        let copy = |share: f32| make_copy(job(key, map(), CavePlan::default(), HashMap::from([(ChunkPos::new(0, 0), chunk.clone())]), share));
        // A changed chunk always shows. A 1 × 1 pillar fills a quarter of each 2-block cell it's in.
        let (half, eighth) = (copy(0.5), copy(0.125));
        assert!(half.hot.contains(&ChunkPos::new(0, 0)));
        let (top_half, _) = highest_and_darkest(&half, 2);
        let (top_eighth, _) = highest_and_darkest(&eighth, 2);
        assert!(top_half <= ground as f32 + 2.0, "half: {top_half} over ground {ground}");
        assert!(top_eighth >= ground as f32 + 19.0, "an eighth: {top_eighth} over ground {ground}");
    }

    #[test]
    fn cells_are_coarser_further_away_and_finer_the_sharper() {
        let fov = 70f32.to_radians();
        let at = |distance: f32, sharpness: f32| scale_for(distance, sharpness, fov, 1080.0);
        assert!(at(1_000.0, 2.0) <= at(2_000.0, 2.0) && at(2_000.0, 2.0) <= at(4_000.0, 2.0));
        assert!(at(3_000.0, 4.0) <= at(3_000.0, 1.0));
        assert_eq!(at(10.0, 2.0), 1);
        assert_eq!(at(1_000_000.0, 2.0), COARSEST);
        // A tile's chunks, nearest and furthest, the larger of north-south and east-west.
        assert_eq!(TileKey { scale: 4, x: 2, y: -1 }.chunk_range(IVec2::ZERO), (IVec2::new(8, 1), 11));
        assert_eq!(TileKey { scale: 4, x: 0, y: 0 }.chunk_range(IVec2::new(1, 1)), (IVec2::ZERO, 2));
        // Real chunks reach the streaming circle, inside the render distance's square.
        assert!(real_chunk(IVec2::new(30, 10), 64, 32) && !real_chunk(IVec2::new(30, 12), 64, 32));
        assert!(real_chunk(IVec2::new(20, 20), 64, 32) && !real_chunk(IVec2::new(20, 20), 19, 32));
    }
}

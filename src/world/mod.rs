//! World — block storage and queries.
//!
//! `VoxelWorld` holds every loaded chunk and answers two questions for the rest of the game:
//! "what block is at X, Y, Z?" and "change the block at X, Y, Z".
//!
//! It also remembers which sections changed ("dirty" sections), so the renderer (step 3)
//! knows which meshes to rebuild. A change on a section's edge also dirties the neighbour,
//! because the neighbour's visible faces may have changed too.
//!
//! Separately, it remembers which *chunks* have been changed since the terrain was generated:
//! only those are saved (see `storage.rs`).
//!
//! This module does not decide *why* blocks change. Mining, building and world generation
//! are separate systems that call `set_block` and announce what they did on the event bus.

mod block;
mod chunk;
mod edits;
mod generator;
mod position;
mod pieces;
mod raycast;
mod section;
mod storage;
mod streaming;
mod carving;
mod caves;
mod interpreter;
mod mineralogy;
pub mod placer;
mod terrain;

pub use block::BlockId;
pub use caves::{CavePlan, Caves, TestCaveSettings};
pub use pieces::{Owners, Part, Pieces, Turn, bar_halves, bar_placement, block_halves, block_placement, half_box, octant_of, piece_start};
pub use edits::{
    BlockChanged, BlockEditRefused, BlockEditRequest, EditKind, EditRules, EditSource, RecentBlockChanges,
    RefuseReason,
};
pub use raycast::{RayHit, cast_ray_halves};
#[allow(unused_imports)] // re-exported for later modules (rendering, saving)
pub use chunk::{BlockExtra, Chunk, ColumnSummary};
pub use generator::{TestBlocks, TestWorldSettings, generate_test_world};
#[allow(unused_imports)]
pub use position::{
    CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTION_VOLUME, SECTIONS_PER_CHUNK, SectionKey,
    WORLD_BOTTOM, WORLD_HEIGHT, z_in_world,
};
pub use section::Section;
pub use streaming::{StreamSettings, Streaming};
pub use interpreter::{CompiledMap, HIGHEST, InterpreterSettings, Lift, LiftFrom, Mask, MaskSource, Maps, Mutes, Peaks, Spline, coast_distance, point_distance};
pub use carving::{Banks, Carving, FadeWay, Lake, Lakes, Point as RiverPoint, RIVER_FROM, Reach, STREAM_FROM, Shores, Sizes, depth_across, fullness};
pub use mineralogy::{Layers, MOST_UNDER, Mineralogy, MineralogySettings, Why, starting_distributors};
pub use terrain::{NextTerrain, SEA_LEVEL, Terrain, TerrainBlocks, under_sea, whole_column};
#[cfg(test)]
pub use storage::{fresh_terrain, load_terrain, save_terrain};
pub use storage::as_saved;

use crate::data::BlockRegistry;
use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::events::EventLogAppExt;
use crate::save::WorldStateAppExt;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

/// All loaded terrain.
#[derive(Resource, Default)]
pub struct VoxelWorld {
    chunks: HashMap<ChunkPos, Chunk>,
    dirty: HashSet<SectionKey>,
    /// Chunks changed since the terrain was generated (the ones that get saved).
    changed: HashSet<ChunkPos>,
    /// Changed chunks put away while you're far from them: kept (they can't be made again from the
    /// map), but not drawn or counted as loaded. Streaming brings them back as you come near.
    put_away: HashMap<ChunkPos, Chunk>,
    /// Where new arrivals start (north, east): the middle of the test cylinder, or the spot chosen
    /// on a compiled world's map.
    spawn: IVec2,
}

impl VoxelWorld {
    /// Block at a position. Outside the world or in an unloaded chunk → air.
    pub fn get_block(&self, block: IVec3) -> BlockId {
        if !z_in_world(block.z) {
            return BlockId::AIR;
        }
        self.chunks
            .get(&ChunkPos::of_block(block))
            .map_or(BlockId::AIR, |chunk| chunk.get(block))
    }

    /// Change a block. Returns the old block, or `None` if the position isn't loaded
    /// (or is above/below the world), in which case nothing happens.
    pub fn set_block(&mut self, block: IVec3, id: BlockId) -> Option<BlockId> {
        if !z_in_world(block.z) {
            return None;
        }
        let chunk_pos = ChunkPos::of_block(block);
        let old = self.chunks.get_mut(&chunk_pos)?.set(block, id);
        if old != id {
            self.mark_dirty_around(block);
            self.changed.insert(chunk_pos);
        }
        Some(old)
    }

    /// Is the chunk holding this block loaded (and the block inside the world's height)?
    pub fn is_loaded(&self, block: IVec3) -> bool {
        z_in_world(block.z) && self.chunks.contains_key(&ChunkPos::of_block(block))
    }

    /// Is the half-block at `half` (counted in half blocks: its cell is `half / 2`, its octant
    /// `half % 2`) filled by something `filled` says yes to? A whole block fills all eight of its
    /// halves, a cell of pieces only its octants. Collision and aiming step through the world on this
    /// finer grid, so they meet bars as they are (quartile stop 2).
    pub fn half_filled(&self, half: IVec3, filled: impl Fn(BlockId) -> bool) -> bool {
        let cell = half.div_euclid(IVec3::splat(2));
        let block = self.get_block(cell);
        if block != BlockId::PIECES {
            return filled(block);
        }
        self.octant(half).is_some_and(|(block, _)| filled(block))
    }

    /// The octant of material at `half` (in half blocks), and what it's part of, if it's in a cell of
    /// pieces.
    pub fn octant(&self, half: IVec3) -> Option<(BlockId, Part)> {
        self.get_pieces(half.div_euclid(IVec3::splat(2)))?.get(octant_of(half))
    }

    /// The octants (in half blocks) of the piece at `half`: that octant, its bar's other half if it's
    /// whole, and any cut pieces stuck to them (whose other half would have been one of these). What's
    /// aimed at and mined at once [David, 2026-11-01: "the target box would target the whole
    /// quarterblock plus any octblocks that are stuck to it"]. Empty if there's no octant there.
    pub fn piece_halves(&self, half: IVec3) -> Vec<IVec3> {
        let Some((block, part)) = self.octant(half) else { return Vec::new() };
        let mut halves = vec![half];
        if part.whole()
            && let Some(other) = part.other_half()
        {
            halves.push(half + other);
        }
        // A whole block on the octant grid: all 8 of its octants, found from its corner.
        if let Some(corner) = part.corner() {
            let low = half - IVec3::new(corner as i32 & 1, (corner as i32 >> 1) & 1, (corner as i32 >> 2) & 1);
            halves = block_halves(low).into_iter().enumerate().filter(|&(c, other)| self.octant(other) == Some((block, Part::of_block(c)))).map(|(_, other)| other).collect();
        }
        for core in halves.clone() {
            for axis in 0..3 {
                for way in [-1, 1] {
                    let mut step = IVec3::ZERO;
                    step[axis] = way;
                    let next = core + step;
                    if !halves.contains(&next)
                        && let Some((_, stuck)) = self.octant(next)
                        && stuck.is_cut()
                        && stuck.other_half() == Some(-step)
                    {
                        halves.push(next);
                    }
                }
            }
        }
        halves
    }

    /// The pieces of a cell built of them, if it is one (and loaded).
    pub fn get_pieces(&self, block: IVec3) -> Option<&Pieces> {
        if !z_in_world(block.z) {
            return None;
        }
        self.chunks.get(&ChunkPos::of_block(block))?.pieces(block)
    }

    /// Makes a cell one built of these pieces. Returns the old block, or `None` if the position isn't
    /// loaded (nothing happens then).
    pub fn set_pieces(&mut self, block: IVec3, pieces: Pieces) -> Option<BlockId> {
        if !z_in_world(block.z) {
            return None;
        }
        let chunk_pos = ChunkPos::of_block(block);
        let old = self.chunks.get_mut(&chunk_pos)?.set_pieces(block, pieces);
        // Even from pieces to pieces: they're drawn again.
        self.mark_dirty_around(block);
        self.changed.insert(chunk_pos);
        Some(old)
    }

    /// Where new arrivals start (north, east).
    pub fn spawn(&self) -> IVec2 {
        self.spawn
    }

    pub fn set_spawn(&mut self, spawn: IVec2) {
        self.spawn = spawn;
    }

    /// Chunks changed since the terrain was generated.
    pub fn changed_chunks(&self) -> &HashSet<ChunkPos> {
        &self.changed
    }

    /// Count a chunk as changed (e.g. one loaded from a save over the generated terrain).
    pub fn mark_changed(&mut self, pos: ChunkPos) {
        self.changed.insert(pos);
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    /// Add (or replace) a chunk. All its sections become dirty so they get drawn.
    pub fn insert_chunk(&mut self, chunk: Chunk) {
        let pos = chunk.pos();
        for level in 0..SECTIONS_PER_CHUNK {
            self.dirty.insert(SectionKey { chunk: pos, level: level as u8 });
        }
        self.chunks.insert(pos, chunk);
    }

    /// Add a chunk without marking it for drawing (streaming draws it once its neighbours are in,
    /// so its edges are right the first time; see `streaming.rs`).
    pub fn insert_chunk_quiet(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.pos(), chunk);
    }

    /// Mark all of a chunk's sections for (re)drawing.
    pub fn mark_chunk_dirty(&mut self, pos: ChunkPos) {
        for level in 0..SECTIONS_PER_CHUNK {
            self.dirty.insert(SectionKey { chunk: pos, level: level as u8 });
        }
    }

    /// Take a chunk out of the world (its meshes are removed). Changed chunks should be kept:
    /// the caller decides.
    pub fn remove_chunk(&mut self, pos: ChunkPos) -> Option<Chunk> {
        let removed = self.chunks.remove(&pos);
        if removed.is_some() {
            self.mark_chunk_dirty(pos);
        }
        removed
    }

    /// Take a changed chunk out of the world but keep it (its meshes are removed), to come back
    /// with `take_put_away` when it's wanted again.
    pub fn put_away(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.remove_chunk(pos) {
            self.put_away.insert(pos, chunk);
        }
    }

    /// A changed chunk that was put away, taken back out to be loaded again.
    pub fn take_put_away(&mut self, pos: ChunkPos) -> Option<Chunk> {
        self.put_away.remove(&pos)
    }

    /// A changed chunk, loaded or put away (for saving).
    pub fn changed_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos).or_else(|| self.put_away.get(&pos))
    }

    /// Where every loaded chunk is.
    pub fn chunk_positions(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunks.keys().copied()
    }

    /// Remove every chunk. Their sections are marked dirty so the renderer removes their meshes.
    pub fn clear(&mut self) {
        self.put_away.clear();
        for &pos in self.chunks.keys() {
            for level in 0..SECTIONS_PER_CHUNK {
                self.dirty.insert(SectionKey { chunk: pos, level: level as u8 });
            }
        }
        self.chunks.clear();
        self.changed.clear();
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Sections changed but not yet handed to the renderer.
    pub fn dirty_sections(&self) -> impl Iterator<Item = &SectionKey> {
        self.dirty.iter()
    }

    /// Hand over the list of changed sections and start a fresh one (used by the renderer).
    pub fn take_dirty(&mut self) -> HashSet<SectionKey> {
        std::mem::take(&mut self.dirty)
    }

    /// Re-pack every section as tightly as possible.
    pub fn compact_all(&mut self) {
        for chunk in self.chunks.values_mut() {
            for section in chunk.sections_mut() {
                section.compact();
            }
        }
    }

    /// Mark the block's section dirty, plus any neighbouring section it touches.
    fn mark_dirty_around(&mut self, block: IVec3) {
        let last = SECTION_SIZE - 1;
        let local = block.rem_euclid(IVec3::splat(SECTION_SIZE));
        let mut offsets = vec![IVec3::ZERO];
        for axis in 0..3 {
            if local[axis] == 0 {
                let mut offset = IVec3::ZERO;
                offset[axis] = -1;
                offsets.push(offset);
            } else if local[axis] == last {
                let mut offset = IVec3::ZERO;
                offset[axis] = 1;
                offsets.push(offset);
            }
        }
        for offset in offsets {
            let neighbour = block + offset;
            if z_in_world(neighbour.z) {
                self.dirty.insert(SectionKey {
                    chunk: ChunkPos::of_block(neighbour),
                    level: position::section_level(neighbour.z) as u8,
                });
            }
        }
    }

    /// Memory and storage statistics for the debug panel.
    pub fn stats(&self) -> WorldStats {
        let mut stats = WorldStats::default();
        for chunk in self.chunks.values() {
            stats.extra_states += chunk.extra_count();
            for section in chunk.sections() {
                stats.sections += 1;
                stats.memory_bytes += section.memory_bytes();
                *stats.sections_by_bits.entry(section.bits_per_block()).or_default() += 1;
            }
        }
        stats.raw_bytes = stats.sections * position::SECTION_VOLUME * size_of::<BlockId>();
        stats
    }
}

#[derive(Default)]
pub struct WorldStats {
    pub sections: usize,
    /// Bits per block → how many sections use that many (0 = uniform).
    pub sections_by_bits: BTreeMap<u32, usize>,
    pub memory_bytes: usize,
    /// What the same blocks would take stored plainly at 16 bits each.
    pub raw_bytes: usize,
    pub extra_states: usize,
}

/// The system set (on ticks, in `FixedUpdate`) where block edit requests are applied.
/// Simulation systems that should see this tick's edits run `.after(WorldEdits)`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldEdits;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<TestWorldSettings>()
            .init_resource::<Terrain>()
            .init_resource::<NextTerrain>()
            .init_resource::<Streaming>()
            .init_resource::<StreamSettings>()
            .add_settings::<StreamSettings>("streaming")
            .init_resource::<Caves>()
            .init_resource::<TestCaveSettings>()
            .add_settings::<TestCaveSettings>("test_cave")
            .add_systems(Update, streaming::stream_chunks)
            .init_resource::<WorldDebugState>()
            .init_resource::<EditRules>()
            .add_settings::<EditRules>("edit_rules")
            .init_resource::<RecentBlockChanges>()
            .add_message::<BlockEditRequest>()
            .add_message::<BlockChanged>()
            .add_message::<BlockEditRefused>()
            .log_event::<BlockEditRequest>()
            .log_event::<BlockChanged>()
            .log_event::<BlockEditRefused>()
            // The terrain is made when a world starts (see the save module), and saved as "terrain".
            .add_world_state_with("terrain", storage::save_terrain, storage::load_terrain, storage::fresh_terrain)
            .add_systems(FixedUpdate, edits::apply_block_edits.in_set(WorldEdits))
            .add_systems(bevy_egui::EguiPrimaryContextPass, show_left_out.run_if(in_state(crate::screen::Screen::Play)))
            .add_debug_page("World", world_debug_page);
    }
}

/// Along the top of the screen, while a world spawned with some of the recipe switched off is
/// showing: what's left out, so a stripped world is never mistaken for the real one.
fn show_left_out(mut contexts: bevy_egui::EguiContexts, terrain: Res<Terrain>) {
    let Some(left_out) = terrain.compiled().map(|map| map.mutes.left_out()).filter(|names| !names.is_empty()) else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Area::new(egui::Id::new("recipe left out")).anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 8.0)).order(egui::Order::Foreground).interactable(false).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.colored_label(egui::Color32::from_rgb(255, 0, 255), format!("Not the whole recipe: {} left out", left_out.join(", ")));
        });
    });
}

/// How far around the middle of a compiled world chunks are made at once when it starts, so
/// there's ground under you straight away. The rest stream in (see `streaming.rs`).
const COMPILED_RADIUS_CHUNKS: i32 = 2;

/// Make the terrain of a compiled world around its spawn spot (north, east), timing how long it
/// takes.
fn generate_compiled_world(world: &mut World, map: &CompiledMap, spawn: IVec2) {
    let blocks = TerrainBlocks::from_registry(world.resource::<BlockRegistry>());
    let caves = world.get_resource::<Caves>().map(|caves| caves.0.clone()).unwrap_or_default();
    let started = Instant::now();
    let mut voxel = world.resource_mut::<VoxelWorld>();
    voxel.clear();
    voxel.set_spawn(spawn);
    let centre = ChunkPos::of_block(IVec3::new(spawn.x, spawn.y, 0));
    let chunks = terrain::generate_around(&mut voxel, map, &blocks, &caves, centre, COMPILED_RADIUS_CHUNKS);
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    info!("Generated compiled world: {chunks} chunks in {millis:.1} ms");
    // Streaming starts over too, even for the same map (spawning again somewhere else).
    if let Some(mut streaming) = world.get_resource_mut::<Streaming>() {
        *streaming = Streaming::default();
    }
    if let Some(mut debug) = world.get_resource_mut::<WorldDebugState>() {
        debug.last_generation_ms = millis;
    }
}

/// Rebuild the test world from the current settings, timing how long it takes.
fn regenerate_test_world(world: &mut World) {
    let settings = world.resource::<TestWorldSettings>().clone();
    let blocks = TestBlocks::from_registry(world.resource::<BlockRegistry>());
    let started = Instant::now();
    let chunks = generate_test_world(&mut world.resource_mut::<VoxelWorld>(), &settings, &blocks);
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    info!("Generated test world: {chunks} chunks in {millis:.1} ms");
    if let Some(mut debug) = world.get_resource_mut::<WorldDebugState>() {
        debug.last_generation_ms = millis;
    }
}

// ── The test cave (docs/caves.md, stop 1) ───────────────────────────────────

/// Places a test cave at the crosshair: a dead end worming into the face aimed at (into a wall the
/// way it faces, or, aimed at the ground, the way you're looking). Its chunks are made again with it.
/// Says what happened.
fn place_test_cave(world: &mut World) -> String {
    let Some(map) = world.resource::<Terrain>().compiled().cloned() else { return "A test cave needs a compiled world.".to_string() };
    let target = world.resource::<crate::camera::LookTarget>();
    let (Some(hit), Some(point)) = (target.hit, target.point) else { return "Aim at a hillside first (within reach).".to_string() };
    let looking = world.query::<&crate::camera::FlyCamera>().iter(world).next().map_or(Vec3::X, |camera| camera.looking());
    let heading = if hit.normal.z == 0 && hit.normal != IVec3::ZERO { -hit.normal.truncate().as_vec2() } else { looking.truncate().normalize_or(Vec2::X) };
    let old = world.resource::<Caves>().0.clone();
    let seed = (hit.block.x.wrapping_mul(73_856_093) ^ hit.block.y.wrapping_mul(19_349_663) ^ hit.block.z.wrapping_mul(83_492_791)) as u32 ^ old.tunnels.len() as u32;
    let settings = world.resource::<TestCaveSettings>().clone();
    let tunnel = settings.tunnel(point + hit.normal.as_vec3(), heading, seed);
    let places = CavePlan::chunks_near(&tunnel);
    let plan = std::sync::Arc::new(CavePlan::new(old.tunnels.iter().cloned().chain([tunnel]).collect()));
    world.insert_resource(Caves(plan.clone()));
    let (with, without, kept) = remake_chunks(world, &map, &places, &plan, &old);
    let mut note = format!("A test cave, {:.0} blocks in. Its {} chunks made again in {with:.1} ms ({without:.1} ms without it).", settings.length, places.len());
    if kept > 0 {
        note += &format!(" {kept} you'd changed were carved, keeping your changes.");
    }
    note
}

/// Takes every test cave out. Chunks you've changed keep their holes (they're saved as they are).
fn remove_test_caves(world: &mut World) -> String {
    let Some(map) = world.resource::<Terrain>().compiled().cloned() else { return String::new() };
    let old = world.resource::<Caves>().0.clone();
    let places: HashSet<ChunkPos> = old.tunnels.iter().flat_map(CavePlan::chunks_near).collect();
    let plan = std::sync::Arc::new(CavePlan::default());
    world.insert_resource(Caves(plan.clone()));
    let (_, _, kept) = remake_chunks(world, &map, &places, &plan, &old);
    let mut note = format!("{} test caves removed.", old.tunnels.len());
    if kept > 0 {
        note += &format!(" {kept} chunks you'd changed keep their holes.");
    }
    note
}

/// Makes the loaded chunks at `places` again from the recipe with the caves in `plan`, timing it
/// (with them, and with the `old` plan); chunks you've changed are carved instead, so your changes
/// stay. Returns the two times (ms) and how many changed chunks there were.
fn remake_chunks(world: &mut World, map: &CompiledMap, places: &HashSet<ChunkPos>, plan: &CavePlan, old: &CavePlan) -> (f64, f64, usize) {
    let blocks = TerrainBlocks::from_registry(world.resource::<BlockRegistry>());
    world.resource_mut::<Streaming>().forget(places);
    let ms = |started: Instant| started.elapsed().as_secs_f64() * 1000.0;
    let (mut with, mut without, mut kept) = (0.0, 0.0, 0);
    let mut voxel = world.resource_mut::<VoxelWorld>();
    for &pos in places {
        if voxel.chunk(pos).is_none() {
            continue;
        }
        if voxel.changed_chunks().contains(&pos) {
            if let Some(chunk) = voxel.chunk_mut(pos) {
                caves::carve_chunk(chunk, plan, blocks.water);
            }
            voxel.mark_chunk_dirty(pos);
            kept += 1;
            continue;
        }
        let started = Instant::now();
        let _ = map.chunk(pos, &blocks, old);
        without += ms(started);
        let started = Instant::now();
        if let Some(chunk) = map.chunk(pos, &blocks, plan) {
            with += ms(started);
            voxel.insert_chunk(chunk);
        }
    }
    (with, without, kept)
}

// ── Debug page ──────────────────────────────────────────────────────────────

#[derive(Resource)]
struct WorldDebugState {
    /// Block position being inspected.
    lookup: IVec3,
    last_generation_ms: f64,
    note_text: String,
    /// What placing or removing a test cave did.
    cave_note: String,
}

impl Default for WorldDebugState {
    fn default() -> Self {
        Self {
            lookup: IVec3::new(0, 0, 144),
            last_generation_ms: 0.0,
            note_text: "hello".to_string(),
            cave_note: String::new(),
        }
    }
}

enum LookupAction {
    SetBlock(&'static str),
    AttachNote,
}

fn world_debug_page(ui: &mut egui::Ui, world: &mut World) {
    // ── Test world settings ──
    ui.strong("Test world");
    let mut regenerate = false;
    {
        let mut settings = world.resource_mut::<TestWorldSettings>();
        ui.add(crate::debug::Slider::new(&mut settings.radius, 1.0..=120.0).text("radius (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.grass_z, 1..=WORLD_HEIGHT - 2).text("grass layer Z"));
        ui.horizontal(|ui| {
            if ui.button("Regenerate").clicked() {
                regenerate = true;
            }
            if ui.button("Reset to defaults").clicked() {
                *settings = TestWorldSettings::default();
            }
        });
    }
    if regenerate {
        regenerate_test_world(world);
    }
    ui.label(format!(
        "Last generation: {:.1} ms",
        world.resource::<WorldDebugState>().last_generation_ms
    ));

    // ── Streaming (compiled worlds) ──
    ui.separator();
    ui.strong("Streaming (compiled worlds)");
    {
        let mut settings = world.resource_mut::<StreamSettings>();
        ui.add(crate::debug::Slider::new(&mut settings.radius, 2..=64).text("radius (chunks)"))
            .on_hover_text("How far around you chunks are made and drawn. 32 chunks = 1,024 blocks.");
        ui.add(crate::debug::Slider::new(&mut settings.ahead, 0.0..=0.9).text("ahead first"))
            .on_hover_text("How much nearer chunks in front of you count. 0 = all directions alike.");
        ui.add(crate::debug::Slider::new(&mut settings.most_at_once, 0..=32).text("most made at once (0 = one per core)"))
            .on_hover_text("Fewer = less CPU at once (smoother), more = the world fills in faster.");
        ui.add(crate::debug::Slider::new(&mut settings.insert_budget_ms, 0.1..=8.0).logarithmic(true).text("insert budget (ms per frame)"))
            .on_hover_text("The most time each frame spent putting finished chunks into the world.");
    }
    {
        let streaming = world.resource::<Streaming>();
        ui.label(format!("Waiting: {}   being made: {}   made: {}   dropped: {}", streaming.queued(), streaming.being_made(), streaming.made, streaming.dropped));
    }

    // ── The test cave (docs/caves.md, stop 1) ──
    ui.separator();
    ui.strong("Test cave");
    ui.small("A dead-end tunnel worming into the hillside at your crosshair, taken out of the ground as its chunks are made, and saved with the world. To compare settings: remove the test caves, change them, and place again.");
    {
        let mut settings = world.resource_mut::<TestCaveSettings>();
        ui.add(crate::debug::Slider::new(&mut settings.length, 6.0..=120.0).text("how far in (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.width, 0.5..=8.0).text("half its width (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.height, 0.5..=8.0).text("half its height (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.wander, 0.0..=60.0).text("how much it worms (degrees every 3 blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.rise_and_fall, 0.0..=30.0).text("rise and fall as it worms (degrees)"));
        ui.add(crate::debug::Slider::new(&mut settings.dip, -20.0..=30.0).text("heading down (degrees)")).on_hover_text("Below 0 it heads up into the hill");
        ui.add(crate::debug::Slider::new(&mut settings.end_size, 0.1..=1.0).text("size at its end (share)")).on_hover_text("How narrow its last third gets: 1 = no narrower");
        ui.add(crate::debug::Slider::new(&mut settings.rough, 0.0..=4.0).text("rough walls (blocks)")).on_hover_text("How far the walls are pushed in and out, so it isn't a pipe");
        ui.add(crate::debug::Slider::new(&mut settings.lump, 1..=16).text("lumps (blocks)")).on_hover_text("How big the walls' lumps are");
        ui.add(crate::debug::Slider::new(&mut settings.floor, 0.0..=1.0).text("flat floor (share of its height down)")).on_hover_text("1 = round all the way down; lower = a flatter floor, higher up");
    }
    let compiled = world.resource::<Terrain>().compiled().is_some();
    let count = world.resource::<Caves>().0.tunnels.len();
    let (mut place, mut remove, mut reset) = (false, false, false);
    ui.horizontal_wrapped(|ui| {
        place = ui.add_enabled(compiled, egui::Button::new("Place a test cave at the crosshair")).on_disabled_hover_text("Needs a compiled world (World Compiler, Render World and Spawn)").clicked();
        remove = ui.add_enabled(count > 0, egui::Button::new(format!("Remove the test caves ({count})"))).clicked();
        reset = ui.button("Reset to defaults").clicked();
    });
    if reset {
        *world.resource_mut::<TestCaveSettings>() = TestCaveSettings::default();
    }
    if place {
        let note = place_test_cave(world);
        world.resource_mut::<WorldDebugState>().cave_note = note;
    }
    if remove {
        let note = remove_test_caves(world);
        world.resource_mut::<WorldDebugState>().cave_note = note;
    }
    let note = world.resource::<WorldDebugState>().cave_note.clone();
    if !note.is_empty() {
        ui.small(note);
    }

    // ── Storage statistics ──
    ui.separator();
    ui.strong("Storage");
    let voxel = world.resource::<VoxelWorld>();
    let stats = voxel.stats();
    ui.label(format!("Chunks: {}   Sections: {}", voxel.chunk_count(), stats.sections));
    for (bits, count) in &stats.sections_by_bits {
        let kind = if *bits == 0 {
            "uniform".to_string()
        } else {
            format!("{bits}-bit packed")
        };
        ui.label(format!("   {kind}: {count}"));
    }
    let ratio = stats.raw_bytes as f64 / stats.memory_bytes.max(1) as f64;
    ui.label(format!(
        "Block memory: {}   (plain 16-bit would be {}, {ratio:.0}× more)",
        format_bytes(stats.memory_bytes),
        format_bytes(stats.raw_bytes)
    ));
    ui.label(format!(
        "Dirty sections (waiting to redraw): {}   Extra states: {}",
        voxel.dirty_count(),
        stats.extra_states
    ));
    ui.label(format!(
        "Changed chunks (saved with the world): {}",
        voxel.changed_chunks().len()
    ));
    let mut compact = false;
    if ui.button("Compact all sections").clicked() {
        compact = true;
    }
    if compact {
        world.resource_mut::<VoxelWorld>().compact_all();
    }

    // ── Block inspector ──
    ui.separator();
    ui.strong("Block inspector");
    let mut lookup = world.resource::<WorldDebugState>().lookup;
    ui.horizontal(|ui| {
        ui.label("X (north)");
        ui.add(egui::DragValue::new(&mut lookup.x));
        ui.label("Y (east)");
        ui.add(egui::DragValue::new(&mut lookup.y));
        ui.label("Z (up)");
        ui.add(egui::DragValue::new(&mut lookup.z).range(0..=WORLD_HEIGHT - 1));
    });
    world.resource_mut::<WorldDebugState>().lookup = lookup;

    let voxel = world.resource::<VoxelWorld>();
    let registry = world.resource::<BlockRegistry>();
    let chunk_pos = ChunkPos::of_block(lookup);
    let level = position::section_level(lookup.z.clamp(0, WORLD_HEIGHT - 1));
    let block = voxel.get_block(lookup);
    ui.label(format!("Block: {}  (id {})", registry.name(block), block.0));
    ui.label(format!("Chunk: ({}, {})   Section level: {level}", chunk_pos.x, chunk_pos.y));

    let mut action = None;
    match voxel.chunk(chunk_pos) {
        None => {
            ui.label("Chunk not loaded.");
        }
        Some(chunk) => {
            ui.label(format!("Section storage: {}", chunk.sections()[level].describe()));
            let local = lookup.rem_euclid(IVec3::splat(CHUNK_SIZE));
            match chunk.surface_at(local.x, local.y) {
                Some((z, id)) => ui.label(format!("Column surface: Z {z} ({})", registry.name(id))),
                None => ui.label("Column surface: (empty column)"),
            };
            match chunk.extra(lookup) {
                Some(BlockExtra::DebugNote(text)) => ui.label(format!("Extra state: note \"{text}\"")),
                None => ui.label("Extra state: none"),
            };

            ui.horizontal(|ui| {
                if ui.button("Set to air").clicked() {
                    action = Some(LookupAction::SetBlock("incus:air"));
                }
                if ui.button("Set to stone").clicked() {
                    action = Some(LookupAction::SetBlock("incus:stone"));
                }
                if ui.button("Set to gabbro").clicked() {
                    action = Some(LookupAction::SetBlock("incus:gabbro"));
                }
            });
        }
    }
    {
        let mut state = world.resource_mut::<WorldDebugState>();
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut state.note_text).desired_width(120.0));
            if ui.button("Attach note").clicked() {
                action = Some(LookupAction::AttachNote);
            }
        });
    }

    match action {
        Some(LookupAction::SetBlock(name)) => {
            if let Some(id) = world.resource::<BlockRegistry>().id(name) {
                world.resource_mut::<VoxelWorld>().set_block(lookup, id);
            }
        }
        Some(LookupAction::AttachNote) => {
            let text = world.resource::<WorldDebugState>().note_text.clone();
            if let Some(chunk) = world.resource_mut::<VoxelWorld>().chunk_mut(chunk_pos) {
                chunk.set_extra(lookup, BlockExtra::DebugNote(text));
            }
        }
        None => {}
    }
}

fn format_bytes(bytes: usize) -> String {
    match bytes {
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1 << 10) as f64),
        b => format!("{b} B"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world() -> (VoxelWorld, TestBlocks) {
        let registry = BlockRegistry::default();
        let blocks = TestBlocks::from_registry(&registry);
        let mut world = VoxelWorld::default();
        generate_test_world(&mut world, &TestWorldSettings::default(), &blocks);
        (world, blocks)
    }

    #[test]
    fn test_world_layers() {
        let (world, blocks) = test_world();
        assert_eq!(world.get_block(IVec3::new(0, 0, 0)), blocks.bedrock);
        assert_eq!(world.get_block(IVec3::new(0, 0, 1)), blocks.stone);
        assert_eq!(world.get_block(IVec3::new(0, 0, 143)), blocks.stone);
        assert_eq!(world.get_block(IVec3::new(0, 0, 144)), blocks.grass);
        assert_eq!(world.get_block(IVec3::new(0, 0, 145)), BlockId::AIR);
    }

    #[test]
    fn cylinder_edges() {
        let (world, blocks) = test_world();
        // 100 blocks across: X from −50 to 49 along the centre line.
        assert_eq!(world.get_block(IVec3::new(49, 0, 144)), blocks.grass);
        assert_eq!(world.get_block(IVec3::new(-50, 0, 144)), blocks.grass);
        assert_eq!(world.get_block(IVec3::new(50, 0, 144)), BlockId::AIR);
        assert_eq!(world.get_block(IVec3::new(-51, 0, 144)), BlockId::AIR);
        // Corners of the bounding square are outside the circle.
        assert_eq!(world.get_block(IVec3::new(45, 45, 144)), BlockId::AIR);
    }

    #[test]
    fn negative_coordinates_land_in_the_right_chunk() {
        assert_eq!(ChunkPos::of_block(IVec3::new(-1, -1, 0)), ChunkPos::new(-1, -1));
        assert_eq!(ChunkPos::of_block(IVec3::new(-32, 31, 0)), ChunkPos::new(-1, 0));
        assert_eq!(ChunkPos::of_block(IVec3::new(-33, 32, 0)), ChunkPos::new(-2, 1));
    }

    #[test]
    fn set_block_and_dirty_neighbours() {
        let (mut world, blocks) = test_world();
        world.take_dirty();
        // X = 0 is on a section's west edge, so its neighbour at X = −1 must redraw too.
        let old = world.set_block(IVec3::new(0, 5, 100), BlockId::AIR);
        assert_eq!(old, Some(blocks.stone));
        assert_eq!(world.get_block(IVec3::new(0, 5, 100)), BlockId::AIR);
        let dirty = world.take_dirty();
        assert!(dirty.contains(&SectionKey { chunk: ChunkPos::new(0, 0), level: 3 }));
        assert!(dirty.contains(&SectionKey { chunk: ChunkPos::new(-1, 0), level: 3 }));
        assert_eq!(dirty.len(), 2);
    }

    #[test]
    fn outside_world_is_air_and_unchangeable() {
        let (mut world, blocks) = test_world();
        assert_eq!(world.get_block(IVec3::new(0, 0, WORLD_HEIGHT)), BlockId::AIR);
        assert_eq!(world.set_block(IVec3::new(0, 0, -1), blocks.stone), None);
        assert_eq!(world.set_block(IVec3::new(5000, 0, 10), blocks.stone), None);
    }
}

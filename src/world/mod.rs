//! World — block storage and queries.
//!
//! `VoxelWorld` holds every loaded chunk and answers two questions for the rest of the game:
//! "what block is at X, Y, Z?" and "change the block at X, Y, Z".
//!
//! It also remembers which sections changed ("dirty" sections), so the renderer (step 3)
//! knows which meshes to rebuild. A change on a section's edge also dirties the neighbour,
//! because the neighbour's visible faces may have changed too.
//!
//! This module does not decide *why* blocks change. Mining, building and world generation
//! are separate systems that call `set_block` and announce what they did on the event bus.

mod block;
mod chunk;
mod edits;
mod generator;
mod position;
mod raycast;
mod section;

pub use block::BlockId;
pub use edits::{
    BlockChanged, BlockEditRefused, BlockEditRequest, EditKind, EditRules, EditSource, RecentBlockChanges,
    RefuseReason,
};
pub use raycast::{RayHit, cast_ray};
#[allow(unused_imports)] // re-exported for later modules (rendering, saving)
pub use chunk::{BlockExtra, Chunk, ColumnSummary};
pub use generator::{TestBlocks, TestWorldSettings, generate_test_world};
#[allow(unused_imports)]
pub use position::{
    CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTION_VOLUME, SECTIONS_PER_CHUNK, SectionKey,
    WORLD_HEIGHT, z_in_world,
};
pub use section::Section;

use crate::data::BlockRegistry;
use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::events::EventLogAppExt;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

/// All loaded terrain.
#[derive(Resource, Default)]
pub struct VoxelWorld {
    chunks: HashMap<ChunkPos, Chunk>,
    dirty: HashSet<SectionKey>,
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
        }
        Some(old)
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

    /// Remove every chunk. Their sections are marked dirty so the renderer removes their meshes.
    pub fn clear(&mut self) {
        for &pos in self.chunks.keys() {
            for level in 0..SECTIONS_PER_CHUNK {
                self.dirty.insert(SectionKey { chunk: pos, level: level as u8 });
            }
        }
        self.chunks.clear();
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
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
            .add_systems(Startup, generate_on_startup)
            .add_systems(FixedUpdate, edits::apply_block_edits.in_set(WorldEdits))
            .add_debug_page("World", world_debug_page);
    }
}

fn generate_on_startup(world: &mut World) {
    regenerate_test_world(world);
}

/// Rebuild the test world from the current settings, timing how long it takes.
fn regenerate_test_world(world: &mut World) {
    let settings = world.resource::<TestWorldSettings>().clone();
    let blocks = TestBlocks::from_registry(world.resource::<BlockRegistry>());
    let started = Instant::now();
    let chunks = generate_test_world(&mut world.resource_mut::<VoxelWorld>(), &settings, &blocks);
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    info!("Generated test world: {chunks} chunks in {millis:.1} ms");
    world.resource_mut::<WorldDebugState>().last_generation_ms = millis;
}

// ── Debug page ──────────────────────────────────────────────────────────────

#[derive(Resource)]
struct WorldDebugState {
    /// Block position being inspected.
    lookup: IVec3,
    last_generation_ms: f64,
    note_text: String,
}

impl Default for WorldDebugState {
    fn default() -> Self {
        Self {
            lookup: IVec3::new(0, 0, 144),
            last_generation_ms: 0.0,
            note_text: "hello".to_string(),
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
        ui.add(egui::Slider::new(&mut settings.radius, 1.0..=120.0).text("radius (blocks)"));
        ui.add(egui::Slider::new(&mut settings.grass_z, 1..=WORLD_HEIGHT - 2).text("grass layer Z"));
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
        assert_eq!(world.get_block(IVec3::new(0, 0, 400)), BlockId::AIR);
        assert_eq!(world.set_block(IVec3::new(0, 0, -1), blocks.stone), None);
        assert_eq!(world.set_block(IVec3::new(5000, 0, 10), blocks.stone), None);
    }
}

//! Render — drawing the block world.
//!
//! - `textures` — loads block face images into one texture array
//! - `mesher`   — turns a section of blocks into merged rectangles (binary greedy meshing)
//! - `meshing`  — runs the mesher on background threads and keeps meshes up to date
//! - `material` — connects our block shader (`assets/shaders/terrain.wesl`) to Bevy
//!
//! Culling (not drawing what can't be seen):
//! - **Frustum culling** is done by Bevy: sections outside the camera's view are skipped,
//!   using the bounding box each section mesh carries.
//! - **Distance culling** is done here: sections beyond the render distance are hidden.
//!   (Simulation distance is a separate setting, added when there's simulation to limit.)

mod distant;
mod material;
mod mesher;
mod meshing;
mod performance;
mod textures;

use crate::coords::bevy_to_game;
use crate::data::{BlockRegistry, FACE_NAMES, TextureRotation};
use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::world::{CHUNK_SIZE, SectionKey, VoxelWorld};
use bevy::prelude::*;
use bevy::render::error_handler::{ErrorType, RenderError, RenderErrorHandler, RenderErrorPolicy};
use bevy_egui::egui;
use material::{TerrainMaterial, TerrainShaderSettings};
use meshing::{MeshingState, SectionMesh};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct RenderPlugin;

/// What to do when the graphics card reports an error. Bevy's own answer is to quit the game for
/// any of them, but minimizing the window on DirectX 12 always reports one, harmlessly: the window's
/// picture can't be resized to nothing ("window is in use"), nothing is drawn while it's minimized,
/// and the picture is resized again when it comes back. So an error while the window is minimized
/// is let past; any other still quits, as before.
fn on_render_error(error: &RenderError, main_world: &mut World, _render_world: &mut World) -> RenderErrorPolicy {
    let mut windows = main_world.query::<&Window>();
    let minimized = windows.iter(main_world).any(|window| window.physical_width() == 0 || window.physical_height() == 0);
    if minimized && error.ty == ErrorType::Validation {
        warn!("Graphics error while the window is minimized; carrying on (it's harmless)");
        return RenderErrorPolicy::Ignore;
    }
    error!("Quitting the game because of a {:?} graphics error", error.ty);
    main_world.write_message(AppExit::error());
    RenderErrorPolicy::StopRendering
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderErrorHandler(on_render_error))
            .add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .add_plugins(MaterialPlugin::<distant::DistantMaterial>::default())
            .init_resource::<distant::DistantState>()
            .init_resource::<distant::DistantSettings>()
            .add_settings::<distant::DistantSettings>("distant")
            .add_systems(Startup, distant::setup_distant)
            .add_systems(Update, (distant::update_distant, distant::update_distant_material))
            .init_resource::<RenderSettings>()
            .init_resource::<MeshingState>()
            .init_resource::<performance::FrameTimes>()
            .add_systems(Update, performance::record_frame_time)
            .insert_resource(ClearColor(SKY_COLOUR))
            .add_systems(Startup, setup_terrain)
            .add_systems(
                Update,
                (
                    rebuild_terrain_after_reload,
                    follow_block_rotations,
                    meshing::collect_dirty_sections,
                    meshing::start_jobs,
                    meshing::finish_jobs,
                    distant::update_shown_chunks,
                    apply_render_distance,
                    update_shader_settings,
                )
                    .chain(),
            )
            .add_debug_page("Render", render_debug_page)
            .add_debug_page("Performance", performance::performance_debug_page)
            .add_settings::<RenderSettings>("render");
    }
}

/// Placeholder sky (also the fog colour).
const SKY_COLOUR: Color = Color::srgb(0.53, 0.62, 0.75);

/// Tunables for drawing (shown on the "Render" debug page, and saved in "everything" presets).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderSettings {
    /// How far away sections are drawn, in chunks (1 chunk = 32 blocks).
    pub render_distance_chunks: i32,
    /// How many new mesh jobs may start per frame (spreads big rebuilds over several frames).
    pub max_jobs_per_frame: usize,
    /// Face brightness: top, bottom, north/south, east/west.
    pub shade_top: f32,
    pub shade_bottom: f32,
    pub shade_north_south: f32,
    pub shade_east_west: f32,
    pub fog_enabled: bool,
    /// Fog distances in blocks.
    pub fog_start: f32,
    pub fog_end: f32,
    pub debug_view: DebugView,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DebugView {
    Normal,
    /// Each merged rectangle a different colour: shows greedy meshing at work.
    Quads,
    /// Each face direction a different colour.
    Faces,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            render_distance_chunks: 32,
            max_jobs_per_frame: 64,
            shade_top: 1.0,
            shade_bottom: 0.5,
            shade_north_south: 0.8,
            shade_east_west: 0.65,
            fog_enabled: true,
            fog_start: 6_000.0,
            fog_end: 60_000.0,
            debug_view: DebugView::Normal,
        }
    }
}

impl RenderSettings {
    fn shader_settings(&self, fog_colour: Color) -> TerrainShaderSettings {
        let fog = fog_colour.to_linear();
        TerrainShaderSettings {
            face_shade: Vec4::new(
                self.shade_top,
                self.shade_bottom,
                self.shade_north_south,
                self.shade_east_west,
            ),
            fog_color: Vec4::new(fog.red, fog.green, fog.blue, 1.0),
            fog_and_debug: Vec4::new(
                self.fog_start,
                self.fog_end.max(self.fog_start + 1.0),
                if self.fog_enabled { 1.0 } else { 0.0 },
                match self.debug_view {
                    DebugView::Normal => 0.0,
                    DebugView::Quads => 1.0,
                    DebugView::Faces => 2.0,
                },
            ),
            // x: random rotation on. Always on now; it's switched per block on the Blocks page.
            options: Vec4::new(1.0, 0.0, 0.0, 0.0),
        }
    }
}

/// What the terrain renderer built from the block registry (at startup, and again whenever
/// the data files or textures change).
#[derive(Resource)]
pub struct TerrainAssets {
    pub material: Handle<TerrainMaterial>,
    /// The registry revision this was built from.
    pub revision: u64,
    /// Per block number: opaque?
    pub opaque: Arc<Vec<bool>>,
    /// Per block number: see-through (drawn both sides, with holes)?
    pub see_through: Arc<Vec<bool>>,
    /// Per block number: if see-through, are faces between neighbours drawn (grid) or not (outline)?
    pub inner_faces: Arc<Vec<bool>>,
    /// Per block number: texture layer for each face.
    pub face_layers: Arc<Vec<[u32; 6]>>,
    /// Per block number: is each face's texture randomly rotated?
    pub face_random_rotation: Arc<Vec<[bool; 6]>>,
    pub layer_names: Vec<String>,
    pub texture_problems: Vec<String>,
}

fn setup_terrain(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    settings: Res<RenderSettings>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let (image, terrain) = build_terrain(&registry, Handle::default());
    let material = materials.add(TerrainMaterial {
        textures: images.add(image),
        settings: settings.shader_settings(SKY_COLOUR),
    });
    commands.insert_resource(TerrainAssets { material, ..terrain });
}

/// Load the block textures and build the per-block tables the mesher uses.
fn build_terrain(registry: &BlockRegistry, material: Handle<TerrainMaterial>) -> (Image, TerrainAssets) {
    let textures = textures::build_block_textures(registry);
    info!(
        "Block textures: {} layers, {} problem(s)",
        textures.layer_names.len(),
        textures.problems.len()
    );
    let terrain = TerrainAssets {
        material,
        revision: registry.revision(),
        opaque: Arc::new(registry.defs().iter().map(|def| def.opaque).collect()),
        see_through: Arc::new(registry.defs().iter().map(|def| def.see_through).collect()),
        inner_faces: Arc::new(registry.defs().iter().map(|def| def.inner_faces).collect()),
        face_layers: Arc::new(textures.face_layers),
        face_random_rotation: Arc::new(rotation_table(registry)),
        layer_names: textures.layer_names,
        texture_problems: textures.problems,
    };
    (textures.image, terrain)
}

/// Live reload: when the registry's data or textures changed, rebuild the texture array and
/// tables, and redraw every section.
fn rebuild_terrain_after_reload(
    registry: Res<BlockRegistry>,
    terrain: Option<ResMut<TerrainAssets>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<MeshingState>,
) {
    let Some(mut terrain) = terrain else {
        return;
    };
    if terrain.revision == registry.revision() {
        return;
    }
    let (image, rebuilt) = build_terrain(&registry, terrain.material.clone());
    // A brand-new texture for the material (the old one is dropped). Changing the old
    // texture in place doesn't re-upload its layers and mip levels correctly.
    if let Some(mut material) = materials.get_mut(&terrain.material) {
        material.textures = images.add(image);
    }
    *terrain = rebuilt;
    let keys: Vec<SectionKey> = state.entities.keys().copied().collect();
    state.waiting.extend(keys);
}

/// Per block number: is each face's texture randomly rotated?
fn rotation_table(registry: &BlockRegistry) -> Vec<[bool; 6]> {
    registry
        .defs()
        .iter()
        .map(|def| def.rotations.map(|rotation| rotation == TextureRotation::Random))
        .collect()
}

/// When a block's rotation switches change (Blocks page), rebuild every mesh, since the
/// rotation is stored in the mesh.
fn follow_block_rotations(
    registry: Res<BlockRegistry>,
    terrain: Option<ResMut<TerrainAssets>>,
    mut state: ResMut<MeshingState>,
) {
    let Some(mut terrain) = terrain else {
        return;
    };
    if !registry.is_changed() {
        return;
    }
    let wanted = rotation_table(&registry);
    if *terrain.face_random_rotation == wanted {
        return;
    }
    terrain.face_random_rotation = Arc::new(wanted);
    let keys: Vec<SectionKey> = state.entities.keys().copied().collect();
    state.waiting.extend(keys);
}

/// Hide sections beyond the render distance (measured horizontally, in chunks).
fn apply_render_distance(
    settings: Res<RenderSettings>,
    camera: Query<&Transform, With<Camera3d>>,
    mut sections: Query<(&SectionMesh, &mut Visibility)>,
) {
    let Some(camera) = camera.iter().next() else {
        return;
    };
    let position = bevy_to_game(camera.translation);
    let camera_chunk = IVec2::new(
        (position.x / CHUNK_SIZE as f32).floor() as i32,
        (position.y / CHUNK_SIZE as f32).floor() as i32,
    );
    let limit = settings.render_distance_chunks;
    for (section, mut visibility) in &mut sections {
        let offset = IVec2::new(section.key.chunk.x, section.key.chunk.y) - camera_chunk;
        let wanted = if offset.x.abs().max(offset.y.abs()) <= limit {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(wanted);
    }
}

fn update_shader_settings(
    settings: Res<RenderSettings>,
    clear: Res<ClearColor>,
    terrain: Option<Res<TerrainAssets>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let Some(terrain) = terrain else {
        return;
    };
    if !settings.is_changed() && !clear.is_changed() {
        return;
    }
    if let Some(mut material) = materials.get_mut(&terrain.material) {
        material.settings = settings.shader_settings(clear.0);
    }
}

// ── Debug page ──────────────────────────────────────────────────────────────

fn render_debug_page(ui: &mut egui::Ui, world: &mut World) {
    // ── Statistics ──
    let mut sections = 0usize;
    let mut drawn = 0usize;
    let mut quads = 0u64;
    let mut vertices = 0u64;
    let mut query = world.query::<(&SectionMesh, &ViewVisibility)>();
    for (mesh, visibility) in query.iter(world) {
        sections += 1;
        quads += mesh.quads as u64;
        vertices += mesh.vertices as u64;
        if visibility.get() {
            drawn += 1;
        }
    }
    let gpu_bytes = vertices * 8 + quads * 6 * 4;
    ui.strong("Meshes");
    ui.label(format!("Section meshes: {sections}   drawn this frame: {drawn}"));
    ui.label(format!(
        "Rectangles: {quads}   triangles: {}   vertices: {vertices}",
        quads * 2
    ));
    ui.label(format!("Mesh memory on GPU: {:.1} KB", gpu_bytes as f64 / 1024.0));
    let state = world.resource::<MeshingState>();
    let average = if state.jobs_finished > 0 {
        state.total_job_micros as f64 / state.jobs_finished as f64 / 1000.0
    } else {
        0.0
    };
    ui.label(format!(
        "Jobs: {} waiting, {} running, {} done   avg {average:.2} ms, slowest {:.2} ms",
        state.waiting.len(),
        state.running.len(),
        state.jobs_finished,
        state.slowest_job_micros as f64 / 1000.0
    ));
    let mut remesh = false;
    if ui.button("Rebuild every mesh").clicked() {
        remesh = true;
    }
    if remesh {
        let keys: Vec<SectionKey> = world.resource::<MeshingState>().entities.keys().copied().collect();
        world.resource_mut::<MeshingState>().waiting.extend(keys);
    }

    // ── Settings ──
    ui.separator();
    ui.strong("Settings");
    {
        let mut settings = world.resource_mut::<RenderSettings>();
        ui.add(crate::debug::Slider::new(&mut settings.render_distance_chunks, 1..=64).text("render distance (chunks)"));
        ui.add(crate::debug::Slider::new(&mut settings.max_jobs_per_frame, 1..=512).text("mesh jobs started / frame"));
        ui.add(crate::debug::Slider::new(&mut settings.shade_top, 0.0..=1.5).text("shade: top"));
        ui.add(crate::debug::Slider::new(&mut settings.shade_bottom, 0.0..=1.5).text("shade: bottom"));
        ui.add(crate::debug::Slider::new(&mut settings.shade_north_south, 0.0..=1.5).text("shade: north/south"));
        ui.add(crate::debug::Slider::new(&mut settings.shade_east_west, 0.0..=1.5).text("shade: east/west"));
        ui.checkbox(&mut settings.fog_enabled, "fog");
        ui.add(crate::debug::Slider::new(&mut settings.fog_start, 0.0..=12_000.0).text("fog start (blocks)"))
            .on_hover_text("Where the haze begins. Closer haze makes far things look farther: a strong depth cue from the air.");
        let fog_start = settings.fog_start;
        // Fog end can't be closer than fog start (the fog gets thicker from start to end).
        ui.add(crate::debug::Slider::new(&mut settings.fog_end, fog_start..=120_000.0).text("fog end (blocks)"))
            .on_hover_text("Where the haze is total (the sky's colour).");
        if settings.fog_end < settings.fog_start {
            settings.fog_end = settings.fog_start;
        }
        ui.horizontal(|ui| {
            ui.label("View:");
            ui.selectable_value(&mut settings.debug_view, DebugView::Normal, "normal");
            ui.selectable_value(&mut settings.debug_view, DebugView::Quads, "rectangles");
            ui.selectable_value(&mut settings.debug_view, DebugView::Faces, "face directions");
        });
        ui.small("Random texture rotation is switched per block, on the Blocks page.");
        if ui.button("Reset render settings").clicked() {
            *settings = RenderSettings::default();
        }
    }

    // ── Distant terrain ──
    ui.separator();
    ui.strong("Distant terrain");
    ui.small("The landscape past the real chunks, in rings of coarser and coarser cells.");
    {
        let mut settings = world.resource_mut::<distant::DistantSettings>();
        ui.checkbox(&mut settings.enabled, "show distant terrain");
        ui.add(crate::debug::Slider::new(&mut settings.rings, 1..=10).text("rings")).on_hover_text("Each ring out has cells twice the size: more rings, further you see.");
        ui.add(crate::debug::Slider::new(&mut settings.finest_cell, 4..=64).text("finest cell (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.hill_shading, 0.0..=0.6).text("hill shading")).on_hover_text(
            "Lights each patch by its slope toward the sun, so hills and valleys read from afar. 0 = flat colour.\n\
             (Only the distant terrain: the real chunks show their slopes as block steps.)",
        );
        ui.add(crate::debug::Slider::new(&mut settings.sun_from, 0.0..=360.0).suffix("°").text("sun from")).on_hover_text("The compass direction the light comes from: 0° north, 90° east, 315° north-west.");
        ui.add(crate::debug::Slider::new(&mut settings.sun_height, 5.0..=90.0).suffix("°").text("sun height")).on_hover_text("Lower = longer, stronger shading on slopes.");
        ui.add(crate::debug::Slider::new(&mut settings.height_tint, 0.0..=0.3).text("height tint")).on_hover_text("Land a little lighter the higher it is.");
        if ui.button("Reset distant terrain settings").clicked() {
            *settings = distant::DistantSettings::default();
        }
    }

    // ── Textures ──
    ui.separator();
    ui.strong("Block textures");
    if let Some(terrain) = world.get_resource::<TerrainAssets>() {
        for (layer, name) in terrain.layer_names.iter().enumerate() {
            ui.monospace(format!("layer {layer:>3}: {name}"));
        }
        if terrain.texture_problems.is_empty() {
            ui.label("No texture problems.");
        } else {
            for problem in &terrain.texture_problems {
                ui.colored_label(egui::Color32::from_rgb(255, 80, 200), format!("Problem: {problem}"));
            }
        }
        ui.small(format!("Face order: {}", FACE_NAMES.join(", ")));
    }

    ui.separator();
    ui.small(format!("Chunks loaded: {}", world.resource::<VoxelWorld>().chunk_count()));
}

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

mod material;
mod mesher;
mod meshing;
mod textures;

use crate::coords::bevy_to_game;
use crate::data::{BlockRegistry, FACE_NAMES};
use crate::debug::DebugAppExt;
use crate::world::{CHUNK_SIZE, SectionKey, VoxelWorld};
use bevy::prelude::*;
use bevy_egui::egui;
use material::{TerrainMaterial, TerrainShaderSettings};
use meshing::{MeshingState, SectionMesh};
use std::sync::Arc;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .init_resource::<RenderSettings>()
            .init_resource::<MeshingState>()
            .insert_resource(ClearColor(SKY_COLOUR))
            .add_systems(Startup, setup_terrain)
            .add_systems(
                Update,
                (
                    meshing::collect_dirty_sections,
                    meshing::start_jobs,
                    meshing::finish_jobs,
                    apply_render_distance,
                    update_shader_settings,
                )
                    .chain(),
            )
            .add_debug_page("Render", render_debug_page);
    }
}

/// Placeholder sky (also the fog colour).
const SKY_COLOUR: Color = Color::srgb(0.53, 0.62, 0.75);

/// Tunables for drawing (shown on the "Render" debug page).
#[derive(Resource, Clone)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
            render_distance_chunks: 12,
            max_jobs_per_frame: 64,
            shade_top: 1.0,
            shade_bottom: 0.5,
            shade_north_south: 0.8,
            shade_east_west: 0.65,
            fog_enabled: true,
            fog_start: 150.0,
            fog_end: 380.0,
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
        }
    }
}

/// What the terrain renderer built at startup.
#[derive(Resource)]
pub struct TerrainAssets {
    pub material: Handle<TerrainMaterial>,
    /// Per block number: opaque?
    pub opaque: Arc<Vec<bool>>,
    /// Per block number: texture layer for each face.
    pub face_layers: Arc<Vec<[u32; 6]>>,
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
    let textures = textures::build_block_textures(&registry);
    info!(
        "Block textures: {} layers, {} problem(s)",
        textures.layer_names.len(),
        textures.problems.len()
    );
    let material = materials.add(TerrainMaterial {
        textures: images.add(textures.image),
        settings: settings.shader_settings(SKY_COLOUR),
    });
    commands.insert_resource(TerrainAssets {
        material,
        opaque: Arc::new(registry.defs().iter().map(|def| def.opaque).collect()),
        face_layers: Arc::new(textures.face_layers),
        layer_names: textures.layer_names,
        texture_problems: textures.problems,
    });
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
        ui.add(egui::Slider::new(&mut settings.render_distance_chunks, 1..=64).text("render distance (chunks)"));
        ui.add(egui::Slider::new(&mut settings.max_jobs_per_frame, 1..=512).text("mesh jobs started / frame"));
        ui.add(egui::Slider::new(&mut settings.shade_top, 0.0..=1.5).text("shade: top"));
        ui.add(egui::Slider::new(&mut settings.shade_bottom, 0.0..=1.5).text("shade: bottom"));
        ui.add(egui::Slider::new(&mut settings.shade_north_south, 0.0..=1.5).text("shade: north/south"));
        ui.add(egui::Slider::new(&mut settings.shade_east_west, 0.0..=1.5).text("shade: east/west"));
        ui.checkbox(&mut settings.fog_enabled, "fog");
        ui.add(egui::Slider::new(&mut settings.fog_start, 0.0..=2000.0).text("fog start (blocks)"));
        let fog_start = settings.fog_start;
        // Fog end can't be closer than fog start (the fog gets thicker from start to end).
        ui.add(egui::Slider::new(&mut settings.fog_end, fog_start..=2000.0).text("fog end (blocks)"));
        if settings.fog_end < settings.fog_start {
            settings.fog_end = settings.fog_start;
        }
        ui.horizontal(|ui| {
            ui.label("View:");
            ui.selectable_value(&mut settings.debug_view, DebugView::Normal, "normal");
            ui.selectable_value(&mut settings.debug_view, DebugView::Quads, "rectangles");
            ui.selectable_value(&mut settings.debug_view, DebugView::Faces, "face directions");
        });
        if ui.button("Reset render settings").clicked() {
            *settings = RenderSettings::default();
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

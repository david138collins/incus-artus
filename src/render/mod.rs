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

mod deep_shot;
mod distant;
mod frame_cap;
mod light;
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
pub(crate) use light::LightSettings;
pub(crate) use performance::{Timed, soak_line};
pub use deep_shot::deep_shot_section;
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
            .init_resource::<deep_shot::DeepShot>()
            .init_resource::<deep_shot::DeepShotSettings>()
            .add_systems(Update, deep_shot::run_deep_shot.after(distant::update_distant))
            .add_systems(bevy_egui::EguiPrimaryContextPass, deep_shot::draw_deep_shot_bar)
            .add_systems(PostUpdate, deep_shot::leave_out_panels.after(bevy_egui::EguiPostUpdateSet::ProcessOutput).before(bevy_egui::EguiPostUpdateSet::PostProcessOutput))
            .init_resource::<light::LightSettings>()
            .init_resource::<light::LightState>()
            .add_settings::<light::LightSettings>("light")
            .init_resource::<light::NearLightState>()
            .add_systems(Update, (light::update_light, light::update_near_light))
            .init_resource::<RenderSettings>()
            .init_resource::<MeshingState>()
            .init_resource::<performance::FrameTimes>()
            .init_resource::<performance::StepTimes>()
            .add_systems(Last, performance::collect_steps)
            .add_systems(Update, performance::record_frame_time)
            .add_systems(Last, frame_cap::cap_frames)
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
            .add_page_preset("render", &["render", "distant", "light"])
            .add_debug_page("Performance", performance::performance_debug_page)
            .add_settings::<RenderSettings>("render");
        // A developer tool, as F12's screenshot is.
        #[cfg(feature = "dev")]
        app.add_systems(Update, deep_shot::deep_shot_key);
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
    /// How many new mesh jobs may start per 240th of a second (spreads big rebuilds over several
    /// frames; a slower frame starts more, see `meshing.rs`).
    pub max_jobs_per_frame: usize,
    /// The most new section meshes handed to the graphics side each frame (the rest wait their turn).
    pub meshes_per_frame: usize,
    /// The seam probe [2026-10-29, Counsel's plan for the near/far seam]: the near blocks hidden and the
    /// far surface drawn everywhere, to flick between them on the same ground; and light only: near
    /// and far painted plain grey, their lighting kept, to tell colour from light.
    pub seam_probe: bool,
    pub light_only: bool,
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
    /// Survey view: every `survey_every`th level of blocks above sea level darkened by
    /// `survey_darkness`, near and far, so the land reads in bands like a contour map.
    pub survey: bool,
    pub survey_every: i32,
    pub survey_darkness: f32,
    /// Draw a frame every this many refreshes of the screen (1 = every one; 2 = half as many…),
    /// or 0 = as fast as possible, vsync off (see `frame_cap.rs`).
    pub frame_cap: u32,
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
            meshes_per_frame: 48,
            seam_probe: false,
            light_only: false,
            shade_top: 1.0,
            shade_bottom: 0.5,
            shade_north_south: 0.8,
            shade_east_west: 0.65,
            fog_enabled: true,
            fog_start: 6_000.0,
            fog_end: 60_000.0,
            debug_view: DebugView::Normal,
            survey: false,
            survey_every: 4,
            survey_darkness: 0.35,
            // Every other refresh: 120 on a 240 screen, smooth, and much cooler than 240.
            frame_cap: 2,
        }
    }
}

impl RenderSettings {
    /// The survey view for both shaders: x on, y every how many levels, z darkness, w sea level.
    pub fn survey_for_shader(&self) -> Vec4 {
        Vec4::new(
            if self.survey { 1.0 } else { 0.0 },
            self.survey_every.max(1) as f32,
            self.survey_darkness,
            crate::world::SEA_LEVEL as f32,
        )
    }

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
            // x: random rotation on (always on now; it's switched per block on the Blocks page), y:
            // light only (the seam probe's plain grey).
            options: Vec4::new(1.0, if self.light_only { 1.0 } else { 0.0 }, 0.0, 0.0),
            survey: self.survey_for_shader(),
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
        light_map: images.add(light::placeholder()),
        near_map: images.add(light::placeholder()),
        near_cones: images.add(light::cones_placeholder()),
        light: default(),
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
    let _timed = crate::render::Timed::new("render distance");
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
        // The seam probe hides them all, leaving the far surface on the same ground.
        let wanted = if offset.x.abs().max(offset.y.abs()) <= limit && !settings.seam_probe {
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
    let _timed = crate::render::Timed::new("shader settings");
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
    crate::debug::page_presets(ui, world, "render");
    ui.separator();
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
        let refresh = frame_cap::refresh_rate(world.query_filtered::<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>().iter(world).map(|monitor| monitor.refresh_rate_millihertz));
        let mut settings = world.resource_mut::<RenderSettings>();
        ui.horizontal_wrapped(|ui| {
            ui.label("Frame cap:");
            for (cap, name) in [(1, "every refresh"), (2, "1/2"), (3, "1/3"), (4, "1/4")] {
                ui.selectable_value(&mut settings.frame_cap, cap, format!("{name} ({:.0})", refresh / cap as f32));
            }
            ui.selectable_value(&mut settings.frame_cap, 0, "uncapped").on_hover_text("Vsync off, as fast as it can: for measuring top speed. Hot, and the picture can tear.");
        })
        .response
        .on_hover_text(format!(
            "Frames a second, as a share of the screen's {refresh:.0} refreshes a second: each frame shows for the same number of refreshes, so motion stays even. Fewer frames = less heat and power."
        ));
        ui.add(crate::debug::Slider::new(&mut settings.render_distance_chunks, 1..=64).text("render distance (chunks)"));
        ui.add(crate::debug::Slider::new(&mut settings.max_jobs_per_frame, 1..=512).text("mesh jobs started / 240th of a second"))
            .on_hover_text("Per 240th of a second, not per frame: at 60 FPS each frame starts four times as many, so terrain fills in as fast on a slower machine.");
        ui.add(crate::debug::Slider::new(&mut settings.meshes_per_frame, 1..=512).text("new meshes shown per frame")).on_hover_text(
            "The most newly made sections handed to the graphics card each frame; the rest wait their turn (the distant\n\
             terrain shows meanwhile). Higher = the ground fills in faster; lower = smoother frames when lots arrive at once.",
        );
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
        ui.horizontal(|ui| {
            ui.checkbox(&mut settings.seam_probe, "seam probe").on_hover_text(
                "Hides the near blocks and draws the far surface everywhere, so you can flick between near and far on the\n\
                 same ground, seen from the same spot, and see what changes (for matching them).",
            );
            ui.checkbox(&mut settings.light_only, "light only").on_hover_text(
                "Paints near and far plain grey, keeping all their lighting: tells a colour difference from a light one.",
            );
        });
        ui.checkbox(&mut settings.survey, "survey view").on_hover_text(
            "Darkens every few levels of blocks, counted up from sea level, near and far: the land reads in bands like a contour map.",
        );
        ui.add_enabled_ui(settings.survey, |ui| {
            ui.add(crate::debug::Slider::new(&mut settings.survey_every, 1..=64).text("every (levels)"))
                .on_hover_text("How many block levels from one dark band to the next. The first dark band is the first level above the sea.");
            ui.add(crate::debug::Slider::new(&mut settings.survey_darkness, 0.0..=1.0).text("band darkness"));
        });
        ui.small("Random texture rotation is switched per block, on the Blocks page.");
        if ui.button("Reset render settings").clicked() {
            *settings = crate::debug::page_default("render", "render");
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
             (The near chunks too, when they're lit like the distant terrain: see Light.)",
        );
        ui.add(crate::debug::Slider::new(&mut settings.shading_fade, 0.5..=8.0).logarithmic(true).text("shading fade")).on_hover_text(
            "How hill shading fades toward its darkest and brightest. Low = a long, gentle fade, so strong shading still\n\
             eases in; high = close to a hard stop (strong shading goes from shaded to fully dark quickly). Near and far alike.",
        );
        ui.add(crate::debug::Slider::new(&mut settings.height_tint, 0.0..=0.3).text("height tint")).on_hover_text("Land a little lighter the higher it is.");
        ui.add(crate::debug::Slider::new(&mut settings.slope_match, 0.0..=2.0).text("far slopes match the near")).on_hover_text(
            "The far terrain's slopes lit as the near ground's block steps light them on average: the shade at the steps'\n\
             feet (Light: foot shading, step shadows) and the block sides they show. 1 = as worked out; nudge it until\n\
             the two meet without a line. 0 = smooth slopes (brighter).",
        );
        if ui.button("Reset distant terrain settings").clicked() {
            *settings = crate::debug::page_default("render", "distant");
        }
    }

    // ── Light ──
    ui.separator();
    ui.strong("Light");
    ui.small("One sun for everything. The light map adds mountains' shadows and darker valleys to the distant terrain, worked out from the ground's heights in the background.");
    {
        let mut s = world.resource_mut::<light::LightSettings>();
        ui.add(crate::debug::Slider::new(&mut s.sun_from, 0.0..=360.0).suffix("°").text("sun from")).on_hover_text("The compass direction the light comes from: 0° north, 90° east, 315° north-west.");
        ui.add(crate::debug::Slider::new(&mut s.sun_height, 5.0..=90.0).suffix("°").text("sun height")).on_hover_text("Lower = longer shadows, and stronger shading on slopes.");
        ui.checkbox(&mut s.on, "light map");
        ui.add_enabled_ui(s.on, |ui| {
            ui.add(crate::debug::Slider::new(&mut s.shadows, 0.0..=1.0).text("shadows")).on_hover_text("How dark a mountain's shadow is. 0 = none, 1 = black.");
            ui.add(crate::debug::Slider::new(&mut s.softness, 0.0..=15.0).suffix("°").text("shadow softness"))
                .on_hover_text("How soft the shadows' edges are, in degrees of the sun's height. The real sun's is about half a degree.");
            ui.add(crate::debug::Slider::new(&mut s.sky, 0.0..=10.0).text("valley darkening")).on_hover_text("How much darker a spot gets for the sky it can't see: valleys, gorges, the feet of cliffs.");
            ui.add(crate::debug::Slider::new(&mut s.sky_reach, 200.0..=20_000.0).logarithmic(true).text("valley reach (blocks)"))
                .on_hover_text("How far away a rise still hides sky. Short: only tight gorges darken. Long: whole basins between ranges.");
            ui.add(crate::debug::Slider::new(&mut s.relief, 0.0..=4.0).text("relief")).on_hover_text("Darkens what sits below the ground around it, and lightens what stands above it.");
            ui.add(crate::debug::Slider::new(&mut s.relief_reach, 200.0..=20_000.0).logarithmic(true).text("relief reach (blocks)"))
                .on_hover_text("How wide \"the ground around it\" is. Short picks out hills and dips; long picks out whole ranges and basins.");
            ui.add(crate::debug::Slider::new(&mut s.detail, 16.0..=128.0).logarithmic(true).text("light map detail (blocks a pixel)"))
                .on_hover_text("Finer is sharper but slower to work out, and takes more graphics memory (16 is four times 32).");
        });
        ui.checkbox(&mut s.near, "light the near chunks like the distant terrain").on_hover_text(
            "Hill shading on the real chunks' tops, from the slope of the ground under them, with the same sun and strength as the distant terrain; \
             and the light map's shadows and valleys. Faces well under the ground (caves) keep their plain light.",
        );
        ui.add_enabled_ui(s.near, |ui| {
            ui.add(crate::debug::Slider::new(&mut s.near_detail, 1.0..=8.0).logarithmic(true).text("near detail (blocks a pixel)"))
                .on_hover_text("How finely the slope under the near chunks is measured. Coarser is smoother and quicker to make as you move. (A long render distance may coarsen it.)");
            ui.add(crate::debug::Slider::new(&mut s.broad_reach, 8.0..=256.0).logarithmic(true).text("broad shading reach (blocks)")).on_hover_text(
                "The broad lie of the land is the smooth ground's slope averaged over this far, so wobbles too small for the\n\
                 blocks to show shade nothing. Longer = only big landforms shade.",
            );
            ui.label("The steps you can see (a step's top is never darkened, only its foot):");
            ui.add(crate::debug::Slider::new(&mut s.foot_shading, 0.0..=1.0).text("foot shading")).on_hover_text(
                "How dark the foot of a step gets, on every side of it: darkest against the face, fading out.",
            );
            ui.add(crate::debug::Slider::new(&mut s.step_shadows, 0.0..=1.0).text("step shadows")).on_hover_text(
                "How much darker the foot of a step gets on its side away from the sun, roughly where its shadow falls.",
            );
            ui.add(crate::debug::Slider::new(&mut s.step_reach, 2.0..=128.0).logarithmic(true).text("step shade reaches at most (blocks)")).on_hover_text(
                "The farthest a step's shade reaches from its foot, however tall the step. Steps are shaded; whole mountains\n\
                 cast the light map's shadows (a mountain's step shade used to reach a thousand blocks, painting a huge\n\
                 shadow the far terrain doesn't have).",
            );
            ui.add(crate::debug::Slider::new(&mut s.side_shading, 0.0..=1.0).text("sides take the slope's light")).on_hover_text(
                "How much of a slope's sun and shade the steps' sides take, as the tops do. 0 = only their fixed face shade\n\
                 (near mountains look flat beside the far ones); 1 = all of it (every step on a shaded slope stripes dark).\n\
                 The far terrain follows it.",
            );
            ui.add(crate::debug::Slider::new(&mut s.shade_reach, 0.5..=32.0).logarithmic(true).text("shade reach (blocks per block of height)")).on_hover_text(
                "How far a step's shade reaches out from its foot, for each block of its height: at 4, a 1-block step shades\n\
                 4 blocks out and a 5-block cliff 20. (The shading fade, under distant terrain, sets how it eases in.)",
            );
        });
        if ui.button("Reset light settings").clicked() {
            *s = crate::debug::page_default("render", "light");
        }
    }
    ui.small(world.resource::<light::LightState>().status.clone());

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

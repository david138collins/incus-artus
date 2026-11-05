//! Camera — the free-fly developer camera, and "what am I looking at?" (raycast debug).
//!
//! Controls (Minecraft creative-style flying, no collision):
//!   Tab           capture / release the mouse (released = you can use the debug panel)
//!   mouse         look around (only while captured)
//!   W A S D       fly forward / left / back / right (level, whatever the camera's tilt)
//!   Space / Ctrl  fly up / down
//!   Shift         fly faster
//!   F             switch to walking (see the `player` module), where the camera rides
//!                 at the walking body's eyes instead
//!
//! The camera runs on real time, not simulation time, so it still works while the game
//! is paused: it's a tool for looking at the world, not part of the world.
//!
//! Every frame a ray is cast from the camera through the crosshair. The block it hits is
//! outlined in the world, and its details are shown on the "Camera" debug page. Step 5
//! uses the same ray to break and place blocks.
//!
//! (The Panini projection, which re-mapped the picture for wide fields of view, went with the old
//! renderer in the regroup; its preset values are kept, `look.rs`.)


use crate::data::{BlockRegistry, FACE_NAMES};
use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::player::{ControlMode, PlayerSettings};
use crate::screen::Screen;
use crate::world::{ChunkPos, RayHit, VoxelWorld, cast_ray_halves};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use crate::runner::WindowState;
use crate::ui::EguiWantsInput;
use crate::ui::{EguiContexts, EguiPrimaryContextPass};
use serde::{Deserialize, Serialize};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .init_resource::<CameraView>()
            .init_resource::<MouseCapture>()
            .init_resource::<LookTarget>()
            .init_resource::<FieldOfViewBoost>()
            .init_resource::<CameraHold>()
            .init_resource::<Wobble>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    toggle_mouse_capture,
                    look_around.run_if(not_held),
                    fly.run_if(not_held),
                    place_camera,
                    watch_for_wobble,
                    find_look_target,
                )
                    .chain()
                    .run_if(in_state(Screen::Play)),
            )
            .add_systems(EguiPrimaryContextPass, draw_crosshair.run_if(in_state(Screen::Play)))
            .add_debug_page("Camera", camera_debug_page)
            .add_page_preset("camera", &["camera", "panini"])
            .add_settings::<CameraSettings>("camera");
    }
}

/// Tunables (shown on the "Camera" debug page, and saved in "everything" presets).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraSettings {
    /// Flying speed in blocks per second.
    pub fly_speed: f32,
    /// Speed multiplier while Shift is held.
    pub fast_multiplier: f32,
    /// Degrees turned per pixel of mouse movement.
    pub mouse_sensitivity: f32,
    pub invert_mouse_y: bool,
    /// Vertical field of view in degrees.
    pub field_of_view: f32,
    /// How far the debug ray reaches, in blocks.
    pub ray_length: f32,
    pub show_target_outline: bool,
    /// How bright the outlines and placing helpers are drawn, 0 to 1 (David, 2026-11-01: "helpers on
    /// screen that can be dimmed will probably stay in some form in the end game").
    pub helper_brightness: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            fly_speed: 20.0,
            fast_multiplier: 5.0,
            mouse_sensitivity: 0.1,
            invert_mouse_y: false,
            // David's choice, 22 Oct 2026, with Panini on (distance 0.3, crop 1.0).
            field_of_view: 100.0,
            ray_length: 64.0,
            show_target_outline: true,
            helper_brightness: 1.0,
        }
    }
}

/// Where the camera starts (and where "Go to start" returns it): above the test world's
/// south edge, facing north toward the centre and looking slightly down.
const START_POSITION: Vec3 = Vec3::new(-45.0, 0.0, 160.0);
const START_HEADING: f32 = 0.0;
const START_PITCH: f32 = -20.0;

/// The fly camera's state, in game terms.
#[derive(Component)]
pub struct FlyCamera {
    /// Eye position (game axes).
    pub position: Vec3,
    /// Compass direction faced, in degrees: 0 = north, 90 = east, 180 = south, 270 = west.
    pub heading: f32,
    /// Up/down tilt in degrees: 0 = level, +89 = straight up, −89 = straight down.
    pub pitch: f32,
}

impl FlyCamera {
    fn at_start() -> Self {
        Self {
            position: START_POSITION,
            heading: START_HEADING,
            pitch: START_PITCH,
        }
    }

    /// The direction the camera looks (game axes, length 1).
    pub fn looking(&self) -> Vec3 {
        let (heading, pitch) = (self.heading.to_radians(), self.pitch.to_radians());
        Vec3::new(
            heading.cos() * pitch.cos(),
            heading.sin() * pitch.cos(),
            pitch.sin(),
        )
    }
}

/// Is the mouse captured for looking around?
#[derive(Resource, Default)]
pub struct MouseCapture {
    pub captured: bool,
}

/// Holds the camera still (no looking round or flying) while something needs the view to stay put:
/// the deep screenshot.
#[derive(Resource, Default)]
pub struct CameraHold {
    pub held: bool,
}

fn not_held(hold: Res<CameraHold>) -> bool {
    !hold.held
}

/// The block under the crosshair this frame (if any).
#[derive(Resource, Default)]
pub struct LookTarget {
    pub hit: Option<RayHit>,
    /// The half-block hit (counted in half blocks), and the point the ray met it (blocks): which
    /// bar of a cell, and which part of a face, the crosshair is on.
    pub half: Option<IVec3>,
    pub point: Option<Vec3>,
    /// Where what's being placed would go, when that isn't simply the cell in front of the face: the
    /// octants a bar would fill (each box's two corners, blocks; set by the player's placing,
    /// `player/mod.rs`), exactly, a cut one's single octant included. Outlined in magenta.
    pub ghost: Vec<(Vec3, Vec3)>,
    /// The piece aimed at, in a cell of pieces: its octants (a bar, and the cut pieces stuck to it),
    /// outlined instead of the whole cell.
    pub piece: Vec<(Vec3, Vec3)>,
    /// An object being placed: an arrow (from, to: blocks) out of its front, the way it would face.
    pub arrow: Option<(Vec3, Vec3)>,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(FlyCamera::at_start());
}

/// Tab captures / releases the mouse. Losing window focus (e.g. Alt-Tab) also releases it.
fn toggle_mouse_capture(keys: Res<ButtonInput<KeyCode>>, mut capture: ResMut<MouseCapture>, mut window: ResMut<WindowState>) {
    if keys.just_pressed(KeyCode::Tab) {
        capture.captured = !capture.captured;
    }
    if !window.focused {
        capture.captured = false;
    }
    if window.capture_mouse != capture.captured {
        window.capture_mouse = capture.captured;
    }
}

fn look_around(
    capture: Res<MouseCapture>,
    settings: Res<CameraSettings>,
    motion: Res<AccumulatedMouseMotion>,
    mut cameras: Query<&mut FlyCamera>,
) {
    if !capture.captured || motion.delta == Vec2::ZERO {
        return;
    }
    let vertical = if settings.invert_mouse_y { 1.0 } else { -1.0 };
    for mut camera in &mut cameras {
        // Mouse right → turn toward east (heading goes up). Mouse up → look up.
        camera.heading = (camera.heading + motion.delta.x * settings.mouse_sensitivity).rem_euclid(360.0);
        camera.pitch = (camera.pitch + vertical * motion.delta.y * settings.mouse_sensitivity).clamp(-89.0, 89.0);
    }
}

fn fly(
    keys: Res<ButtonInput<KeyCode>>,
    egui_input: Res<EguiWantsInput>,
    capture: Res<MouseCapture>,
    mode: Res<ControlMode>,
    settings: Res<CameraSettings>,
    time: Res<Time<Real>>,
    mut cameras: Query<&mut FlyCamera>,
) {
    // While walking, the player module moves the camera with the body instead.
    if *mode != ControlMode::Fly {
        return;
    }
    // With the mouse free, typing in a debug panel text box shouldn't also fly the camera.
    // While flying, the keys always belong to the camera (`draw_crosshair` makes the panel
    // let go of keyboard focus).
    if !capture.captured && egui_input.wants_any_keyboard_input() {
        return;
    }
    let held = |key| keys.pressed(key);
    for mut camera in &mut cameras {
        let heading = camera.heading.to_radians();
        // Level directions, so looking down doesn't make W dive into the ground.
        let forward = Vec3::new(heading.cos(), heading.sin(), 0.0);
        let right = Vec3::new(-heading.sin(), heading.cos(), 0.0);

        let mut movement = Vec3::ZERO;
        if held(KeyCode::KeyW) {
            movement += forward;
        }
        if held(KeyCode::KeyS) {
            movement -= forward;
        }
        if held(KeyCode::KeyD) {
            movement += right;
        }
        if held(KeyCode::KeyA) {
            movement -= right;
        }
        if held(KeyCode::Space) {
            movement += Vec3::Z;
        }
        if held(KeyCode::ControlLeft) || held(KeyCode::ControlRight) {
            movement -= Vec3::Z;
        }

        let mut speed = settings.fly_speed;
        if held(KeyCode::ShiftLeft) || held(KeyCode::ShiftRight) {
            speed *= settings.fast_multiplier;
        }
        // normalize: flying diagonally isn't faster than flying straight.
        camera.position += movement.normalize_or_zero() * speed * time.delta_secs();
    }
}

/// What the picture is drawn from: the eye (game axes), the way it looks, and the field of view
/// (degrees, up and down; the setting plus any boost). Set once a frame from the fly camera, for
/// whatever draws (the card's work, from phase 1 of the regroup).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct CameraView {
    pub position: Vec3,
    pub heading: f32,
    pub pitch: f32,
    pub looking: Vec3,
    pub field_of_view: f32,
}

impl Default for CameraView {
    fn default() -> Self {
        let camera = FlyCamera::at_start();
        Self { position: camera.position, heading: camera.heading, pitch: camera.pitch, looking: camera.looking(), field_of_view: CameraSettings::default().field_of_view }
    }
}

impl CameraView {
    /// Where a spot in the world (game axes) is on the window, in egui's points from its top left, or
    /// `None` behind the eye.
    pub fn to_screen(&self, window: &WindowState, at: Vec3) -> Option<Vec2> {
        let forward = self.looking.normalize_or_zero();
        // Game axes: X north, Y east, Z up; looking north, east is to the right.
        let right = Vec3::Z.cross(forward).normalize_or_zero();
        let up = forward.cross(right);
        let d = at - self.position;
        let ahead = d.dot(forward);
        if ahead <= 1e-3 || right == Vec3::ZERO {
            return None;
        }
        let (width, height) = (window.size.0 as f32 / window.scale, window.size.1 as f32 / window.scale);
        let tan_half = (self.field_of_view.to_radians() / 2.0).tan();
        let x = d.dot(right) / ahead / (tan_half * width / height.max(1.0));
        let y = d.dot(up) / ahead / tan_half;
        Some(Vec2::new((x + 1.0) / 2.0 * width, (1.0 - y) / 2.0 * height))
    }
}

/// The fly camera's state, and the field of view with its boost, into `CameraView`.
pub fn place_camera(settings: Res<CameraSettings>, boost: Res<FieldOfViewBoost>, cameras: Query<&FlyCamera>, mut view: ResMut<CameraView>) {
    let Ok(camera) = cameras.single() else { return };
    let placed = CameraView {
        position: camera.position,
        heading: camera.heading,
        pitch: camera.pitch,
        looking: camera.looking(),
        field_of_view: (settings.field_of_view + boost.0).clamp(10.0, 170.0),
    };
    if *view != placed {
        *view = placed;
    }
}

/// The camera's smallest movements, for tracking down a quiver [David, 2026-10-28: blocks and the
/// selection box seemed to shake while standing still]: how far it moved, turned and changed its
/// field of view from one frame to the next, the largest of each in the last second.
#[derive(Resource, Default)]
pub struct Wobble {
    last: Option<(Vec3, Vec3, f32)>,
    this_second: (f32, f32, f32, u32),
    second_started: f32,
    /// Moved (blocks), turned (degrees), field of view (degrees), and in how many frames anything
    /// changed at all, in the last full second.
    pub last_second: (f32, f32, f32, u32),
}

fn watch_for_wobble(time: Res<Time<Real>>, mut wobble: ResMut<Wobble>, view: Res<CameraView>) {
    let now = (view.position, view.looking, view.field_of_view);
    if let Some((position, looking, last_fov)) = wobble.last {
        let (moved, turned, widened) = (now.0.distance(position), now.1.angle_between(looking).to_degrees(), (now.2 - last_fov).abs());
        let second = &mut wobble.this_second;
        second.0 = second.0.max(moved);
        second.1 = second.1.max(turned);
        second.2 = second.2.max(widened);
        if moved > 0.0 || turned > 0.0 || widened > 0.0 {
            second.3 += 1;
        }
    }
    wobble.last = Some(now);
    let t = time.elapsed_secs();
    if t - wobble.second_started >= 1.0 {
        wobble.last_second = std::mem::take(&mut wobble.this_second);
        wobble.second_started = t;
    }
}

/// Extra field of view, in degrees, added on top of the setting by other modules
/// (the sprint's FOV boost). Each frame's value replaces the last.
#[derive(Resource, Default, PartialEq)]
pub struct FieldOfViewBoost(pub f32);

/// While walking the ray only reaches as far as your arm; while flying, the debug ray length.
fn find_look_target(
    world: Res<VoxelWorld>,
    registry: Res<BlockRegistry>,
    settings: Res<CameraSettings>,
    mode: Res<ControlMode>,
    player: Res<PlayerSettings>,
    cameras: Query<&FlyCamera>,
    mut target: ResMut<LookTarget>,
) {
    let length = match *mode {
        ControlMode::Fly => settings.ray_length,
        ControlMode::Walk => player.reach,
    };
    // The ray passes through anything the crosshair can't target (air, and the grid if
    // its "crosshair can target it" switch is off), on the half-block grid, so it meets the bars of
    // a cell as they are.
    let stops = |half: IVec3| world.half_filled(half, |block| registry.is_targetable(block));
    let found = cameras.iter().next().and_then(|camera| {
        let (hit, half) = cast_ray_halves(camera.position, camera.looking(), length, stops)?;
        Some((hit, half, camera.position + camera.looking().normalize_or_zero() * hit.distance))
    });
    target.hit = found.map(|(hit, _, _)| hit);
    target.half = found.map(|(_, half, _)| half);
    target.point = found.map(|(_, _, point)| point);
}

// (The targeted block's outline, the face's cross and a placing object's arrow were drawn by Bevy's
// gizmos; they come back with the new renderer, from `LookTarget`.)

/// A small crosshair at the screen centre while flying; a hint while the mouse is free.
/// Placeholder look: magenta lines, pink text.
///
/// Also, while flying: egui (the panel's UI library) uses Tab to move keyboard focus
/// between its buttons, so pressing Tab to fly would hand focus to a panel button,
/// and Space would then click it. So while flying, the panel is made to let go of focus.
fn draw_crosshair(mut contexts: EguiContexts, capture: Res<MouseCapture>) -> Result {
    let ctx = contexts.ctx_mut()?;
    if capture.captured {
        ctx.memory_mut(|memory| {
            if let Some(focused) = memory.focused() {
                memory.surrender_focus(focused);
            }
        });
    }
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("crosshair"),
    ));
    let centre = ctx.content_rect().center();
    if capture.captured {
        let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 255));
        let size = 8.0;
        painter.line_segment([centre - egui::vec2(size, 0.0), centre + egui::vec2(size, 0.0)], stroke);
        painter.line_segment([centre - egui::vec2(0.0, size), centre + egui::vec2(0.0, size)], stroke);
    } else {
        let rect = egui::Rect::from_center_size(centre + egui::vec2(0.0, 40.0), egui::vec2(260.0, 26.0));
        painter.rect(
            rect,
            0.0,
            egui::Color32::BLACK,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 255)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Press Tab to fly",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(255, 140, 220),
        );
    }
    Ok(())
}

// ── Debug page ──────────────────────────────────────────────────────────────

fn camera_debug_page(ui: &mut egui::Ui, world: &mut World) {
    crate::debug::page_presets(ui, world, "camera");
    ui.separator();
    let captured = world.resource::<MouseCapture>().captured;
    ui.label(if captured {
        "Mouse: captured (Tab to release)"
    } else {
        "Mouse: free (Tab to fly)"
    });

    // ── Where the camera is ──
    let mut go_to_start = false;
    {
        let mut query = world.query::<&mut FlyCamera>();
        if let Ok(mut camera) = query.single_mut(world) {
            ui.strong("Position");
            ui.horizontal(|ui| {
                ui.label("X (north)");
                ui.add(egui::DragValue::new(&mut camera.position.x).speed(0.5));
                ui.label("Y (east)");
                ui.add(egui::DragValue::new(&mut camera.position.y).speed(0.5));
                ui.label("Z (up)");
                ui.add(egui::DragValue::new(&mut camera.position.z).speed(0.5));
            });
            let block = camera.position.floor().as_ivec3();
            let chunk = ChunkPos::of_block(block);
            ui.label(format!(
                "In block {}, {}, {}   chunk {}, {}",
                block.x, block.y, block.z, chunk.x, chunk.y
            ));
            ui.label(format!(
                "Facing {:.0}° ({})   tilt {:.0}°",
                camera.heading,
                compass_name(camera.heading),
                camera.pitch
            ));
            if ui.button("Go to start").clicked() {
                go_to_start = true;
            }
            if go_to_start {
                *camera = FlyCamera::at_start();
            }
        }
    }

    // ── The smallest movements (tracking down a quiver) ──
    {
        let (moved, turned, widened, frames) = world.resource::<Wobble>().last_second;
        ui.label(format!("Last second: moved up to {moved:.6} blocks, turned {turned:.5}°, view widened {widened:.5}°, in {frames} frames"))
            .on_hover_text("Standing still with hands off, all of these should be 0. Anything else is the camera quivering.");
    }

    // ── What the crosshair points at ──
    ui.separator();
    ui.strong("Looking at");
    let hit = world.resource::<LookTarget>().hit;
    match hit {
        None => {
            ui.label("Nothing within reach.");
        }
        Some(hit) => {
            let name = world
                .resource::<BlockRegistry>()
                .name(world.resource::<VoxelWorld>().get_block(hit.block))
                .to_string();
            ui.label(format!("Block: {name}"));
            ui.label(format!("At: {}, {}, {}", hit.block.x, hit.block.y, hit.block.z));
            match hit.face() {
                Some(face) => ui.label(format!("Face: {}", FACE_NAMES[face])),
                None => ui.label("Face: none (camera is inside this block)"),
            };
            ui.label(format!("Distance: {:.2} blocks", hit.distance));
            match hit.in_front() {
                Some(cell) => ui.label(format!("A placed block would go at: {}, {}, {}", cell.x, cell.y, cell.z)),
                None => ui.label("A placed block would go at: nowhere"),
            };
        }
    }

    // ── Settings ──
    ui.separator();
    ui.strong("Settings");
    let mut settings = world.resource_mut::<CameraSettings>();
    ui.add(crate::debug::Slider::new(&mut settings.fly_speed, 1.0..=200.0).text("fly speed (blocks/s)"));
    ui.add(crate::debug::Slider::new(&mut settings.fast_multiplier, 1.0..=20.0).text("Shift speed ×"));
    ui.add(crate::debug::Slider::new(&mut settings.mouse_sensitivity, 0.01..=0.5).text("mouse sensitivity (°/pixel)"));
    ui.checkbox(&mut settings.invert_mouse_y, "invert mouse up/down");
    ui.add(crate::debug::Slider::new(&mut settings.field_of_view, 30.0..=150.0).text("field of view (°, up-down)"));
    ui.add(crate::debug::Slider::new(&mut settings.ray_length, 1.0..=256.0).text("ray length while flying (blocks)"));
    ui.checkbox(&mut settings.show_target_outline, "outline the block being looked at");
    let outlined = settings.show_target_outline;
    ui.add_enabled(outlined, crate::debug::Slider::new(&mut settings.helper_brightness, 0.0..=1.0).text("how bright the outlines and placing helpers are")).on_hover_text("The block outline, the magenta room a block, bar or object would take, and an object's facing arrow");
    if ui.button("Reset camera settings").clicked() {
        *settings = crate::debug::page_default("camera", "camera");
    }

    ui.separator();

}

/// 0° → "north", 90° → "east", etc. (to the nearest of 8 directions).
fn compass_name(heading: f32) -> &'static str {
    const NAMES: [&str; 8] = [
        "north",
        "north-east",
        "east",
        "south-east",
        "south",
        "south-west",
        "west",
        "north-west",
    ];
    NAMES[((heading.rem_euclid(360.0) + 22.5) / 45.0) as usize % 8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_zero_looks_north_and_ninety_looks_east() {
        let mut camera = FlyCamera::at_start();
        camera.pitch = 0.0;
        camera.heading = 0.0;
        assert!(camera.looking().abs_diff_eq(Vec3::X, 1e-5));
        camera.heading = 90.0;
        assert!(camera.looking().abs_diff_eq(Vec3::Y, 1e-5));
        assert_eq!(compass_name(90.0), "east");
        assert_eq!(compass_name(350.0), "north");
    }
}

//! Player — the walking body ("pawn"), switching between flying and walking, and
//! breaking / placing blocks.
//!
//! Controls:
//!   F             switch between flying (no collision) and walking (collision + gravity)
//!   W A S D       walk (in walking mode)
//!   Space         jump (hold to keep jumping)
//!   Ctrl          sprint
//!   left click    break the block under the crosshair (while the mouse is captured)
//!   right click   place the selected block against the face under the crosshair
//!   1 2 3         choose the block to place: stone, gabbro, grass (until there's an inventory)
//!
//! The body moves on simulation ticks (`FixedUpdate`), like everything in the simulation, so
//! it stops when the game is paused. The camera is drawn smoothly between ticks.
//!
//! Breaking and placing don't change the world directly: they send a request to the world
//! (see `world/edits.rs`), which checks it, makes the change, and announces it.
//!
//! The test character is a dwarf: 1.5 blocks tall. Speeds and jump match Minecraft's defaults.

mod collision;

use crate::camera::{FlyCamera, LookTarget, MouseCapture};
use crate::data::BlockRegistry;
use crate::debug::DebugAppExt;
use crate::time::{InterpolationSettings, SimPosition};
use crate::world::{
    BlockEditRequest, BlockId, ChunkPos, EditKind, EditRules, EditSource, RecentBlockChanges,
    VoxelWorld, WORLD_HEIGHT, WorldEdits,
};
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use collision::{BodySize, body_bounds, move_body, overlaps_solid};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlMode>()
            .init_resource::<PlayerSettings>()
            .init_resource::<PawnInput>()
            .init_resource::<Hotbar>()
            .init_resource::<ActionCooldowns>()
            // PostStartup: after the world has been generated, so there's ground to stand on.
            .add_systems(PostStartup, spawn_pawn)
            .add_systems(
                Update,
                (
                    toggle_mode,
                    choose_block,
                    read_walk_input,
                    follow_pawn.before(crate::camera::place_camera),
                    block_actions.before(WorldEdits),
                ),
            )
            .add_systems(FixedUpdate, move_pawn)
            .add_systems(EguiPrimaryContextPass, draw_hotbar)
            .add_debug_page("Player", player_debug_page);
    }
}

/// Flying (the developer camera) or walking (the pawn). The game starts walking, on the
/// grass at the world's centre.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ControlMode {
    Fly,
    #[default]
    Walk,
}

/// Tunables (shown on the "Player" debug page).
#[derive(Resource, Clone)]
pub struct PlayerSettings {
    /// Blocks per second.
    pub walk_speed: f32,
    pub sprint_speed: f32,
    /// How high a jump rises, in blocks.
    pub jump_height: f32,
    /// Blocks per second, per second.
    pub gravity: f32,
    /// Fastest possible fall, blocks per second.
    pub max_fall_speed: f32,
    /// How quickly you can change direction in the air (higher = more control).
    pub air_control: f32,
    pub body_height: f32,
    pub body_width: f32,
    pub eye_height: f32,
    /// How far away you can break / place while walking, in blocks.
    pub reach: f32,
    /// Seconds between breaks (and between placements) while the button is held.
    pub break_cooldown: f32,
    pub place_cooldown: f32,
    /// Falling below this Z sends you back to the spawn point.
    pub respawn_below_z: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            walk_speed: 4.317,
            sprint_speed: 5.612,
            jump_height: 1.25,
            gravity: 32.0,
            max_fall_speed: 78.4,
            air_control: 1.9,
            body_height: 1.5,
            body_width: 0.6,
            eye_height: 1.35,
            reach: 5.0,
            break_cooldown: 0.25,
            place_cooldown: 0.25,
            respawn_below_z: -64.0,
        }
    }
}

impl PlayerSettings {
    fn body(&self) -> BodySize {
        BodySize {
            width: self.body_width,
            height: self.body_height,
        }
    }
}

/// The walking body. Its position (feet: the middle of the bottom of the body) is a
/// `SimPosition`, because it moves on ticks.
#[derive(Component, Default)]
pub struct Pawn {
    /// Blocks per second, game axes.
    pub velocity: Vec3,
    pub on_ground: bool,
}

/// What the keys say, gathered every frame and used on the next tick.
#[derive(Resource, Default)]
struct PawnInput {
    /// Wanted walking direction (level, length 0 or 1, game axes).
    direction: Vec3,
    sprint: bool,
    jump_held: bool,
    /// A quick tap of Space between two ticks still counts.
    jump_tapped: bool,
}

/// The blocks keys 1, 2, 3 choose between.
#[derive(Resource)]
pub struct Hotbar {
    pub slots: Vec<&'static str>,
    pub selected: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            slots: vec!["incus:stone", "incus:gabbro", "incus:grass"],
            selected: 0,
        }
    }
}

#[derive(Resource, Default)]
struct ActionCooldowns {
    last_break: f32,
    last_place: f32,
}

/// Where a respawn puts your feet: on top of the highest block at the world's centre.
fn spawn_point(world: &VoxelWorld) -> Vec3 {
    let top = world
        .chunk(ChunkPos::new(0, 0))
        .and_then(|chunk| chunk.surface_at(0, 0))
        .map_or(WORLD_HEIGHT, |(z, _)| z + 1);
    Vec3::new(0.5, 0.5, top as f32)
}

fn spawn_pawn(mut commands: Commands, world: Res<VoxelWorld>) {
    commands.spawn((Pawn::default(), SimPosition::new(spawn_point(&world))));
}

/// Are the keys free for the game (not being typed into a debug panel box)?
fn keys_are_ours(capture: &MouseCapture, egui_input: &EguiWantsInput) -> bool {
    capture.captured || !egui_input.wants_any_keyboard_input()
}

/// Put the pawn at `feet` with no speed; if that's inside blocks, move it up until it isn't.
fn place_pawn(pawn: &mut Pawn, position: &mut SimPosition, feet: Vec3, settings: &PlayerSettings, world: &VoxelWorld) {
    let mut feet = feet;
    let solid = |cell: IVec3| !world.get_block(cell).is_air();
    while overlaps_solid(feet, settings.body(), solid) && feet.z < WORLD_HEIGHT as f32 {
        feet.z = feet.z.floor() + 1.0;
    }
    position.teleport(feet);
    pawn.velocity = Vec3::ZERO;
    pawn.on_ground = false;
}

/// F switches mode. Walking starts where the flying camera is; flying starts where your eyes were.
fn toggle_mode(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<MouseCapture>,
    egui_input: Res<EguiWantsInput>,
    mut mode: ResMut<ControlMode>,
    settings: Res<PlayerSettings>,
    world: Res<VoxelWorld>,
    camera: Query<&FlyCamera>,
    mut pawns: Query<(&mut Pawn, &mut SimPosition)>,
) {
    if !keys.just_pressed(KeyCode::KeyF) || !keys_are_ours(&capture, &egui_input) {
        return;
    }
    *mode = match *mode {
        ControlMode::Fly => {
            if let (Ok(camera), Ok((mut pawn, mut position))) = (camera.single(), pawns.single_mut()) {
                let feet = camera.position - Vec3::Z * settings.eye_height;
                place_pawn(&mut pawn, &mut position, feet, &settings, &world);
            }
            ControlMode::Walk
        }
        // The camera simply stays where it is and starts flying from there.
        ControlMode::Walk => ControlMode::Fly,
    };
}

fn choose_block(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<MouseCapture>,
    egui_input: Res<EguiWantsInput>,
    mut hotbar: ResMut<Hotbar>,
) {
    if !keys_are_ours(&capture, &egui_input) {
        return;
    }
    for (slot, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3].into_iter().enumerate() {
        if keys.just_pressed(key) && slot < hotbar.slots.len() {
            hotbar.selected = slot;
        }
    }
}

fn read_walk_input(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<MouseCapture>,
    egui_input: Res<EguiWantsInput>,
    mode: Res<ControlMode>,
    camera: Query<&FlyCamera>,
    mut input: ResMut<PawnInput>,
) {
    if *mode != ControlMode::Walk || !keys_are_ours(&capture, &egui_input) {
        input.direction = Vec3::ZERO;
        input.jump_held = false;
        input.sprint = false;
        return;
    }
    let Ok(camera) = camera.single() else {
        return;
    };
    let heading = camera.heading.to_radians();
    let forward = Vec3::new(heading.cos(), heading.sin(), 0.0);
    let right = Vec3::new(-heading.sin(), heading.cos(), 0.0);
    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    input.direction = direction.normalize_or_zero();
    input.sprint = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    input.jump_held = keys.pressed(KeyCode::Space);
    if keys.just_pressed(KeyCode::Space) {
        input.jump_tapped = true;
    }
}

/// One simulation tick of walking, jumping, falling and bumping into blocks.
fn move_pawn(
    time: Res<Time>,
    mode: Res<ControlMode>,
    settings: Res<PlayerSettings>,
    world: Res<VoxelWorld>,
    mut input: ResMut<PawnInput>,
    mut pawns: Query<(&mut Pawn, &mut SimPosition)>,
) {
    let jump = input.jump_held || input.jump_tapped;
    input.jump_tapped = false;
    if *mode != ControlMode::Walk {
        return;
    }
    let dt = time.delta_secs();
    for (mut pawn, mut position) in &mut pawns {
        // Walking: on the ground you go exactly where the keys say; in the air you keep
        // most of your momentum and can only steer gradually.
        let speed = if input.sprint {
            settings.sprint_speed
        } else {
            settings.walk_speed
        };
        let target = input.direction * speed;
        let horizontal = Vec3::new(pawn.velocity.x, pawn.velocity.y, 0.0);
        let horizontal = if pawn.on_ground {
            target
        } else {
            horizontal.lerp(target, 1.0 - (-settings.air_control * dt).exp())
        };
        pawn.velocity.x = horizontal.x;
        pawn.velocity.y = horizontal.y;

        // Jumping: the launch speed that rises exactly `jump_height` under this gravity.
        if pawn.on_ground && jump {
            pawn.velocity.z = (2.0 * settings.gravity * settings.jump_height).sqrt();
        }
        pawn.velocity.z = (pawn.velocity.z - settings.gravity * dt).max(-settings.max_fall_speed);

        let solid = |cell: IVec3| !world.get_block(cell).is_air();
        let (feet, blocked) = move_body(position.current, settings.body(), pawn.velocity * dt, solid);
        position.current = feet;
        if blocked.x {
            pawn.velocity.x = 0.0;
        }
        if blocked.y {
            pawn.velocity.y = 0.0;
        }
        if blocked.down || blocked.up {
            pawn.velocity.z = 0.0;
        }
        pawn.on_ground = blocked.down;

        if feet.z < settings.respawn_below_z {
            info!("Fell out of the world: back to the spawn point");
            let spawn = spawn_point(&world);
            place_pawn(&mut pawn, &mut position, spawn, &settings, &world);
        }
    }
}

/// While walking, the camera sits at the pawn's eyes (drawn smoothly between ticks).
fn follow_pawn(
    mode: Res<ControlMode>,
    settings: Res<PlayerSettings>,
    fixed_time: Res<Time<Fixed>>,
    smoothing: Res<InterpolationSettings>,
    pawns: Query<&SimPosition, With<Pawn>>,
    mut cameras: Query<&mut FlyCamera>,
) {
    if *mode != ControlMode::Walk {
        return;
    }
    let (Ok(position), Ok(mut camera)) = (pawns.single(), cameras.single_mut()) else {
        return;
    };
    camera.position = position.display(&fixed_time, &smoothing) + Vec3::Z * settings.eye_height;
}

/// Left click breaks, right click places. Held buttons repeat after the cooldown.
fn block_actions(
    mouse: Res<ButtonInput<MouseButton>>,
    capture: Res<MouseCapture>,
    mode: Res<ControlMode>,
    time: Res<Time<Real>>,
    settings: Res<PlayerSettings>,
    target: Res<LookTarget>,
    hotbar: Res<Hotbar>,
    registry: Res<BlockRegistry>,
    pawns: Query<&SimPosition, With<Pawn>>,
    mut cooldowns: ResMut<ActionCooldowns>,
    mut requests: MessageWriter<BlockEditRequest>,
) {
    // With the mouse free, clicks belong to the debug panel.
    if !capture.captured {
        return;
    }
    let Some(hit) = target.hit else {
        return;
    };
    let now = time.elapsed_secs();

    if mouse.pressed(MouseButton::Left) && now - cooldowns.last_break >= settings.break_cooldown {
        cooldowns.last_break = now;
        requests.write(BlockEditRequest {
            position: hit.block,
            kind: EditKind::Break,
            source: EditSource::Player,
        });
    }

    if mouse.pressed(MouseButton::Right) && now - cooldowns.last_place >= settings.place_cooldown {
        let Some(cell) = hit.in_front() else {
            return;
        };
        // Don't place a block inside your own body while walking.
        if *mode == ControlMode::Walk {
            if let Ok(position) = pawns.single() {
                let (min, max) = body_bounds(position.current, settings.body());
                let cell_min = cell.as_vec3();
                let cell_max = cell_min + Vec3::ONE;
                if min.cmplt(cell_max).all() && max.cmpgt(cell_min).all() {
                    return;
                }
            }
        }
        let Some(block) = hotbar.slots.get(hotbar.selected).and_then(|name| registry.id(name)) else {
            return;
        };
        cooldowns.last_place = now;
        requests.write(BlockEditRequest {
            position: cell,
            kind: EditKind::Place(block),
            source: EditSource::Player,
        });
    }
}

/// Placeholder block bar at the bottom of the screen: black boxes, magenta outlines, pink text.
fn draw_hotbar(
    mut contexts: EguiContexts,
    hotbar: Res<Hotbar>,
    mode: Res<ControlMode>,
    registry: Res<BlockRegistry>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("hotbar")));
    let screen = ctx.content_rect();
    let pink = egui::Color32::from_rgb(255, 140, 220);
    let magenta = egui::Color32::from_rgb(255, 0, 255);
    let slot_size = egui::vec2(110.0, 30.0);
    let gap = 6.0;
    let total_width = hotbar.slots.len() as f32 * (slot_size.x + gap) - gap;
    let mut left = screen.center().x - total_width / 2.0;
    let top = screen.bottom() - slot_size.y - 16.0;

    for (slot, name) in hotbar.slots.iter().enumerate() {
        let rect = egui::Rect::from_min_size(egui::pos2(left, top), slot_size);
        let selected = slot == hotbar.selected;
        let stroke = if selected {
            egui::Stroke::new(3.0, egui::Color32::WHITE)
        } else {
            egui::Stroke::new(1.0, magenta)
        };
        painter.rect(rect, 0.0, egui::Color32::BLACK, stroke, egui::StrokeKind::Inside);
        let label = registry
            .id(name)
            .map_or(*name, |id| registry.name(id))
            .trim_start_matches("incus:");
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{} {label}", slot + 1),
            egui::FontId::proportional(14.0),
            pink,
        );
        left += slot_size.x + gap;
    }

    let mode_text = match *mode {
        ControlMode::Fly => "FLYING  (F to walk)",
        ControlMode::Walk => "WALKING  (F to fly)",
    };
    painter.text(
        egui::pos2(screen.center().x, top - 12.0),
        egui::Align2::CENTER_BOTTOM,
        mode_text,
        egui::FontId::proportional(13.0),
        pink,
    );
    Ok(())
}

// ── Debug page ──────────────────────────────────────────────────────────────

fn player_debug_page(ui: &mut egui::Ui, world: &mut World) {
    // ── Mode ──
    let mode = *world.resource::<ControlMode>();
    ui.horizontal(|ui| {
        ui.label(format!("Mode: {mode:?}"));
        ui.small("(F switches)");
    });

    // ── The pawn ──
    let mut respawn = false;
    {
        let mut query = world.query::<(&Pawn, &SimPosition)>();
        if let Ok((pawn, position)) = query.single(world) {
            ui.strong("Body");
            let feet = position.current;
            ui.label(format!("Feet at: {:.2}, {:.2}, {:.2}", feet.x, feet.y, feet.z));
            let horizontal = Vec2::new(pawn.velocity.x, pawn.velocity.y).length();
            ui.label(format!(
                "Speed: {horizontal:.2} blocks/s across, {:.2} up/down",
                pawn.velocity.z
            ));
            ui.label(if pawn.on_ground { "On the ground" } else { "In the air" });
        }
    }
    if ui.button("Respawn at the centre (and walk)").clicked() {
        respawn = true;
    }
    if respawn {
        let settings = world.resource::<PlayerSettings>().clone();
        let spawn = spawn_point(world.resource::<VoxelWorld>());
        world.resource_scope(|world, voxel: Mut<VoxelWorld>| {
            let mut query = world.query::<(&mut Pawn, &mut SimPosition)>();
            if let Ok((mut pawn, mut position)) = query.single_mut(world) {
                place_pawn(&mut pawn, &mut position, spawn, &settings, &voxel);
            }
        });
        *world.resource_mut::<ControlMode>() = ControlMode::Walk;
    }

    // ── Settings ──
    ui.separator();
    ui.strong("Movement (Minecraft defaults)");
    {
        let mut settings = world.resource_mut::<PlayerSettings>();
        ui.add(egui::Slider::new(&mut settings.walk_speed, 0.5..=20.0).text("walk speed (blocks/s)"));
        ui.add(egui::Slider::new(&mut settings.sprint_speed, 0.5..=30.0).text("sprint speed (Ctrl)"));
        ui.add(egui::Slider::new(&mut settings.jump_height, 0.1..=5.0).text("jump height (blocks)"));
        ui.add(egui::Slider::new(&mut settings.gravity, 1.0..=100.0).text("gravity (blocks/s²)"));
        ui.add(egui::Slider::new(&mut settings.max_fall_speed, 1.0..=200.0).text("max fall speed"));
        ui.add(egui::Slider::new(&mut settings.air_control, 0.0..=20.0).text("air control"));

        ui.strong("Body (dwarf)");
        ui.add(egui::Slider::new(&mut settings.body_height, 0.5..=3.0).text("height"));
        ui.add(egui::Slider::new(&mut settings.body_width, 0.2..=0.98).text("width"));
        let height = settings.body_height;
        ui.add(egui::Slider::new(&mut settings.eye_height, 0.1..=height).text("eye height"));
        if settings.eye_height > settings.body_height {
            settings.eye_height = settings.body_height;
        }

        ui.strong("Breaking & placing");
        ui.add(egui::Slider::new(&mut settings.reach, 1.0..=20.0).text("reach while walking (blocks)"));
        ui.add(egui::Slider::new(&mut settings.break_cooldown, 0.0..=2.0).text("break repeat (s)"));
        ui.add(egui::Slider::new(&mut settings.place_cooldown, 0.0..=2.0).text("place repeat (s)"));
        ui.add(egui::Slider::new(&mut settings.respawn_below_z, -400.0..=0.0).text("respawn when below Z"));
        if ui.button("Reset player settings").clicked() {
            *settings = PlayerSettings::default();
        }
    }
    {
        let mut rules = world.resource_mut::<EditRules>();
        ui.checkbox(&mut rules.protect_bottom_layer, "bottom layer (Z 0) can't be broken");
    }

    // ── Recent block changes (the messages the world sent) ──
    ui.separator();
    ui.strong("Recent block changes");
    let registry = world.resource::<BlockRegistry>();
    let recent = world.resource::<RecentBlockChanges>();
    ui.small(format!("Requests refused so far: {}", recent.refused));
    if recent.changes.is_empty() {
        ui.label("None yet.");
    }
    for change in &recent.changes {
        let name = |id: BlockId| registry.name(id).trim_start_matches("incus:").to_string();
        ui.monospace(format!(
            "{:>4} {:>4} {:>3}  {} → {}  ({:?})",
            change.position.x,
            change.position.y,
            change.position.z,
            name(change.old),
            name(change.new),
            change.source
        ));
    }
}

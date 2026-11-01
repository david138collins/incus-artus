//! Player — the walking body ("pawn"), switching between flying and walking, and
//! breaking / placing blocks.
//!
//! Controls:
//!   F             switch between flying (no collision) and walking (collision + gravity)
//!   W A S D       walk (in walking mode)
//!   Space         jump (hold to keep jumping)
//!   Shift         run (hold; builds up gradually through the run envelope)
//!   Shift Shift   sprint (double-tap and hold, moving forwards only; widens the view)
//!   Ctrl          crouch (hold; shorter, slower, and you won't step off edges)
//!   left click    break the block under the crosshair (while the mouse is captured)
//!   right click   place the selected block against the face under the crosshair
//!   1 2 3 4 5     choose the block to place: stone, gabbro, grass, grid, outline (until there's an inventory)
//!
//! The body moves on simulation ticks (`FixedUpdate`), like everything in the simulation, so
//! it stops when the game is paused. The camera is drawn smoothly between ticks.
//!
//! Breaking and placing don't change the world directly: they send a request to the world
//! (see `world/edits.rs`), which checks it, makes the change, and announces it.
//!
//! The test character is a dwarf: 1.5 blocks tall. Speeds and jump match Minecraft's defaults.
//!
//! Speeds don't switch on instantly: envelopes (attack / decay / sustain / release, like a
//! synth) blend between them. The walk envelope is the legs pushing at all; the run envelope
//! blends walk → run; the sprint envelope blends run → sprint. They only move while your
//! feet are on the ground, since you can't push off the air.
//!
//! Each time the walk envelope fires (a movement key pressed from standing or slowing down),
//! a `Footstep` message is sent: a hook for step sounds, noise and stealth later.

mod collision;

use crate::camera::{FieldOfViewBoost, FlyCamera, LookTarget, MouseCapture};
use crate::data::{BlockRegistry, ObjectId, ObjectRegistry};
use crate::debug::{DebugAppExt, SettingsAppExt, preset_bar};
use crate::kinematics::{Envelope, EnvelopeState, envelope_editor};
use crate::events::{EventLogAppExt, LocalPlayer, LoggedEvent, PlayerId};
use crate::save::{WorldFolder, WorldStateAppExt};
use crate::screen::Screen;
use crate::sim::Presence;
use crate::time::{InterpolationSettings, SimPosition, TimeControl, blend_fraction};
use crate::world::{
    BlockEditRefused, BlockEditRequest, BlockId, CHUNK_SIZE, ChunkPos, EditKind, EditRules, EditSource, RecentBlockChanges,
    RefuseReason, Turn, VoxelWorld, WORLD_HEIGHT, WorldEdits, bar_halves, bar_placement, block_halves, block_placement, half_box, object_box, piece_start, PlacedObject, solid_object_half,
};
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use collision::{BodySize, body_bounds, has_support_halves, move_body_stepping, overlaps_solid_halves};
use serde::{Deserialize, Serialize};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlMode>()
            .init_resource::<PlayerSettings>()
            .init_resource::<Hotbar>()
            .init_resource::<ActionCooldowns>()
            .init_resource::<PieceChoice>()
            .init_resource::<SprintView>()
            .init_resource::<FootstepFlash>()
            .add_message::<Footstep>()
            .add_message::<Landed>()
            // PostStartup: after the world has been generated, so there's ground to stand on.
            .add_systems(PostStartup, spawn_pawn)
            .add_systems(
                Update,
                (
                    toggle_mode,
                    choose_block,
                    read_walk_input,
                    follow_pawn.before(crate::camera::place_camera),
                    widen_view_when_sprinting.before(crate::camera::place_camera),
                    aim_bar.before(block_actions),
                    block_actions,
                    notice_footsteps,
                    update_presence,
                )
                    .run_if(in_state(Screen::Play)),
            )
            // Saved with each world: where you are and what you're looking at.
            .add_world_state_with("player", save_player, load_player, fresh_player)
            // After this tick's block edits, so the body bumps into the world as it now is.
            .add_systems(FixedUpdate, move_pawn.after(WorldEdits))
            .log_event::<Footstep>()
            .log_event::<Landed>()
            .add_systems(EguiPrimaryContextPass, draw_hotbar.run_if(in_state(Screen::Play)))
            .add_debug_page("Player", player_debug_page)
            .add_settings::<PlayerSettings>("player");
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

/// Tunables (shown on the "Player" debug page, and saved in player presets).
/// `serde(default)`: a preset missing some settings fills them in from the defaults.
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerSettings {
    /// Blocks per second.
    pub walk_speed: f32,
    /// Full running speed (Ctrl held), reached when the run envelope is at 1.
    pub run_speed: f32,
    /// Full sprint speed (Ctrl double-tapped), reached when the sprint envelope is at 1.
    pub sprint_speed: f32,
    /// The second Ctrl press must come within this many seconds of the first to sprint.
    /// (Later: modulated by the Athletics skill and the Agility and Speed attributes.)
    pub sprint_double_tap: f32,
    /// How many degrees the view widens at full sprint.
    pub sprint_fov_boost: f32,
    /// How the view widens when a sprint starts, and narrows when it ends.
    pub sprint_fov_envelope: Envelope,
    /// Sideways speed as a fraction of forwards speed (1 = as fast as forwards).
    pub strafe_factor: f32,
    /// Backwards speed as a fraction of forwards speed.
    pub backward_factor: f32,
    /// How your legs build up to walking speed (attack) and stop pushing (release).
    pub walk_envelope: Envelope,
    /// How running builds up (attack), holds (sustain) and fades back to walking (release).
    pub run_envelope: Envelope,
    /// How sprinting builds up from running speed, and fades back to it.
    pub sprint_envelope: Envelope,
    /// Off: the envelopes freeze while your feet are off the ground.
    pub envelopes_in_air: bool,
    /// How quickly your real speed catches up with what your legs want, on ground of
    /// friction 1. Multiplied by the friction of the block you stand on. Lower = more sliding.
    pub ground_grip: f32,
    /// Momentum charge: builds while running or sprinting (attack, decay, sustain = how much
    /// charge). It's spent by sliding when you stop; if you just slow to a walk it fades (release).
    pub momentum_envelope: Envelope,
    /// How far a full charge carries you when you stop dead from a full sprint, in blocks
    /// (on ground of friction 1). Slower speeds and smaller charges carry you less.
    pub slide_distance: f32,
    /// Body height while crouched (Ctrl).
    pub crouch_height: f32,
    pub crouch_eye_height: f32,
    /// Speed while crouched, as a fraction of normal.
    pub crouch_speed: f32,
    /// How quickly you crouch down (attack) and stand back up (release).
    pub crouch_envelope: Envelope,
    /// While crouched on the ground, you won't step off an edge.
    pub crouch_edge_guard: bool,
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
            // David's tuning, 22 Oct 2026.
            walk_speed: 3.2,
            run_speed: 5.7,
            sprint_speed: 7.2,
            sprint_double_tap: 0.3,
            sprint_fov_boost: 12.0,
            sprint_fov_envelope: Envelope {
                attack: 0.15,
                attack_curve: 0.9,
                decay: 0.0,
                decay_curve: 0.0,
                sustain: 1.0,
                release: 0.4,
                release_curve: 0.0,
            },
            strafe_factor: 0.75,
            backward_factor: 0.5,
            walk_envelope: Envelope {
                attack: 0.15,
                attack_curve: -0.5,
                decay: 0.0,
                decay_curve: 0.0,
                sustain: 1.0,
                release: 0.3,
                release_curve: 1.0,
            },
            run_envelope: Envelope {
                attack: 0.75,
                attack_curve: 0.75,
                decay: 0.0,
                decay_curve: 0.0,
                sustain: 1.0,
                release: 0.6,
                release_curve: 0.0,
            },
            sprint_envelope: Envelope {
                attack: 0.3,
                attack_curve: 0.5,
                decay: 0.0,
                decay_curve: 0.0,
                sustain: 1.0,
                release: 1.5,
                release_curve: -0.15,
            },
            envelopes_in_air: false,
            ground_grip: 10.0,
            momentum_envelope: Envelope {
                attack: 1.0,
                attack_curve: 0.0,
                decay: 0.0,
                decay_curve: 0.0,
                sustain: 1.0,
                release: 0.6,
                release_curve: -0.5,
            },
            slide_distance: 3.0,
            crouch_height: 1.2,
            crouch_eye_height: 1.05,
            crouch_speed: 0.3,
            crouch_envelope: Envelope {
                attack: 0.12,
                decay: 0.0,
                sustain: 1.0,
                release: 0.12,
                ..Envelope::default()
            },
            crouch_edge_guard: true,
            jump_height: 1.25,
            gravity: 32.0,
            max_fall_speed: 78.4,
            air_control: 1.9,
            body_height: 1.5,
            body_width: 0.75,
            eye_height: 1.38,
            reach: 5.0,
            break_cooldown: 0.25,
            place_cooldown: 0.25,
            respawn_below_z: -64.0,
        }
    }
}

impl PlayerSettings {
    /// The standing body.
    fn body(&self) -> BodySize {
        BodySize {
            width: self.body_width,
            height: self.body_height,
        }
    }

    /// The body as it is now: shorter while crouched.
    fn body_for(&self, pawn: &Pawn) -> BodySize {
        BodySize {
            width: self.body_width,
            height: if pawn.crouched { self.crouch_height.min(self.body_height) } else { self.body_height },
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
    /// The running walk envelope: how hard the legs are pushing (0 = not at all, 1 = fully).
    pub walk: EnvelopeState,
    /// The running run envelope: 0 = walking speed, 1 = full running speed.
    pub run: EnvelopeState,
    /// The running sprint envelope: 0 = running speed, 1 = full sprint speed.
    pub sprint: EnvelopeState,
    /// The momentum charge: built up by running, let go of after you stop.
    pub momentum: EnvelopeState,
    /// Is the sprint gate on (double-tapped Shift held, moving forwards)?
    pub sprinting: bool,
    /// Crouched (Ctrl held, or no room to stand up)?
    pub crouched: bool,
    /// 0 = standing, 1 = fully crouched (blends eye height and speed).
    pub crouch: EnvelopeState,
    /// The crouch level one tick earlier, so the eyes can be drawn smoothly between ticks.
    pub crouch_previous: f32,
    /// The last direction the keys asked for. The legs keep pushing this way while the walk
    /// envelope releases after you let go.
    pub last_direction: Vec3,
    /// Friction of the ground under the feet (for the debug page).
    pub ground_friction: f32,
}

/// What a pawn's controls say, gathered every frame and used on the next tick. Each pawn has
/// its own: the local player's comes from this keyboard; another player's could come from the
/// network, or a recording.
#[derive(Component, Default)]
struct PawnInput {
    /// Wanted walking direction (level, game axes). Length 1 straight forwards; shorter
    /// sideways and backwards (the strafe and backward factors); 0 with no keys held.
    direction: Vec3,
    /// Shift held.
    run: bool,
    /// Shift double-tapped and still held, while moving forwards.
    sprint: bool,
    /// When Shift was last pressed (real seconds), to spot a double tap.
    last_run_press: f32,
    /// Ctrl held.
    crouch: bool,
    jump_held: bool,
    /// A quick tap of Space between two ticks still counts.
    jump_tapped: bool,
}

/// Sent each time the walk envelope fires (a movement key pressed from standing, or again
/// while slowing down). A hook for step sounds, noise and stealth.
#[derive(Message, Debug, Clone, Copy)]
pub struct Footstep {
    pub player: PlayerId,
    /// Where the feet were, game coordinates.
    pub position: Vec3,
    pub tick: u64,
}

impl LoggedEvent for Footstep {
    const KIND: &'static str = "footstep";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        let p = self.position;
        format!("{} at {:.1} {:.1} {:.1}", self.player, p.x, p.y, p.z)
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// Sent when the body touches down after being in the air. A hook for landing sounds,
/// fall damage, stumbles and so on.
#[derive(Message, Debug, Clone, Copy)]
pub struct Landed {
    pub player: PlayerId,
    /// Where the feet landed, game coordinates.
    pub position: Vec3,
    /// How fast it was falling at touchdown, blocks per second.
    pub speed: f32,
    pub tick: u64,
}

impl LoggedEvent for Landed {
    const KIND: &'static str = "landed";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        let p = self.position;
        format!("{} at {:.1} {:.1} {:.1}, falling at {:.1} blocks/s", self.player, p.x, p.y, p.z, self.speed)
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// For the on-screen "STEP" light and the debug page.
#[derive(Resource, Default)]
struct FootstepFlash {
    /// Real seconds when the last footstep happened.
    last: Option<f32>,
    last_position: Vec3,
    count: u64,
    landings: u64,
    last_landing_speed: f32,
    last_landing_position: Vec3,
}

/// The running sprint FOV envelope (it runs every frame, for smoothness, not on ticks).
#[derive(Resource, Default)]
struct SprintView {
    envelope: EnvelopeState,
}

/// The blocks keys 1 to 5 choose between.
#[derive(Resource)]
pub struct Hotbar {
    pub slots: Vec<String>,
    pub selected: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            // Stone brick first, chosen to start with (David: "make stone brick the default").
            slots: ["incus:stone_brick", "incus:stone", "incus:gabbro", "incus:grass", "incus:grid", "incus:outline"].map(String::from).to_vec(),
            selected: 0,
        }
    }
}

#[derive(Resource, Default)]
struct ActionCooldowns {
    last_break: f32,
    last_place: f32,
}

/// Placing bars, David's control (`docs/quartile.md`, sections 11 to 13): with a bar block in the
/// hotbar, the selector shows exactly the octants a piece would fill, from the octant in front of the
/// face at the crosshair; B turns the bar three ways (sideways, longways, upways: "B 3 ways"), right
/// click places it, and Alt + right click places a single octant (an octblock: a stand-in until
/// there's a chisel). With any block in hand, holding Shift places it whole on the octant grid (David:
/// "shift should free you from the main grid"). Kept apart from storage and aiming, so the control
/// can change without touching them.
#[derive(Resource, Default)]
struct PieceChoice {
    turn: Turn,
    /// What would be placed this frame, if anything: a bar (its corner in half blocks, and its axis),
    /// one octant (where), a whole block on the octant grid (its lowest corner, in half blocks), or
    /// an object (its foot, in 16ths of a block).
    aim: Option<Aim>,
    /// The object under the crosshair, nearer than any block (its foot, 16ths): what left click takes.
    aimed_object: Option<IVec3>,
}

#[derive(Clone, Copy)]
enum Aim {
    Bar(IVec3, u8),
    Octant(IVec3),
    Shifted(IVec3),
    Object(IVec3, ObjectId),
}

/// The nearest object a ray meets (its foot, in 16ths, and how far along), within `reach` blocks:
/// each object's box (`object_box`) in the chunks round where the ray starts.
fn object_on_ray(world: &VoxelWorld, objects: &ObjectRegistry, from: Vec3, toward: Vec3, reach: f32) -> Option<(IVec3, f32)> {
    let toward = toward.normalize_or_zero();
    let start = ChunkPos::of_block(from.floor().as_ivec3());
    let mut nearest: Option<(IVec3, f32)> = None;
    for x in -1..=1 {
        for y in -1..=1 {
            let cell = IVec3::new((start.x + x) * CHUNK_SIZE, (start.y + y) * CHUNK_SIZE, 0);
            for object in world.objects_near(cell) {
                let (low, high) = object_box(object, objects);
                // Where the ray enters and leaves the box, along each axis in turn.
                let (mut enter, mut leave) = (0.0f32, reach);
                for axis in 0..3 {
                    if toward[axis].abs() < 1e-6 {
                        if from[axis] < low[axis] || from[axis] > high[axis] {
                            leave = -1.0;
                        }
                        continue;
                    }
                    let (a, b) = ((low[axis] - from[axis]) / toward[axis], (high[axis] - from[axis]) / toward[axis]);
                    enter = enter.max(a.min(b));
                    leave = leave.min(a.max(b));
                }
                if enter <= leave && nearest.is_none_or(|(_, best)| enter < best) {
                    nearest = Some((object.at, enter));
                }
            }
        }
    }
    nearest
}

/// With a bar block selected: B turns the bar, and the selector snaps to where it would go. With
/// Shift held and any block in hand, where it would go whole on the octant grid.
fn aim_bar(
    (keys, capture): (Res<ButtonInput<KeyCode>>, Res<MouseCapture>),
    hotbar: Res<Hotbar>,
    (registry, objects): (Res<BlockRegistry>, Res<ObjectRegistry>),
    cameras: Query<&FlyCamera>,
    world: Res<VoxelWorld>,
    mut target: ResMut<LookTarget>,
    mut piece: ResMut<PieceChoice>,
) {
    let bar_selected = hotbar.slots.get(hotbar.selected).and_then(|name| registry.id(name)).and_then(|id| registry.def(id)).is_some_and(|def| def.bar.is_some());
    if bar_selected && capture.captured && keys.just_pressed(KeyCode::KeyB) {
        piece.turn = piece.turn.next();
    }
    let looking = cameras.iter().next().map_or(Vec3::X, |camera| camera.looking());
    let block_in_hand = hotbar.slots.get(hotbar.selected).and_then(|name| registry.id(name)).is_some_and(|id| !id.is_air());
    let off_the_grid = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let octant = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    // An object in hand stands on the floor of the cell in front of the face aimed at, in its middle.
    let object_in_hand = hotbar.slots.get(hotbar.selected).and_then(|name| objects.id(name));
    // The object under the crosshair, if it's nearer than the block.
    piece.aimed_object = cameras.iter().next().and_then(|camera| {
        let reach = target.hit.map_or(8.0, |hit| hit.distance);
        object_on_ray(&world, &objects, camera.position, camera.looking(), reach).map(|(at, _)| at)
    });
    piece.aim = match (target.hit, target.half, target.point) {
        (Some(hit), _, _) if object_in_hand.is_some() => hit.in_front().map(|cell| Aim::Object(cell * 16 + IVec3::new(8, 8, 0), object_in_hand.unwrap())),
        (Some(hit), Some(half), Some(point)) if off_the_grid && block_in_hand => block_placement(half, hit.normal, point).map(Aim::Shifted),
        (Some(hit), Some(half), _) if octant && bar_selected => piece_start(half, hit.normal).map(Aim::Octant),
        (Some(hit), Some(half), Some(point)) if bar_selected => bar_placement(half, hit.normal, point, looking, piece.turn).map(|(at, axis)| Aim::Bar(at, axis)),
        _ => None,
    };
    // Exactly the octants it would fill: those free (David: "it should highlight exactly the oct itd
    // fill"); a bar crossing one already there is cut to its other octant.
    let free = |half: &IVec3| match world.get_block(half.div_euclid(IVec3::splat(2))) {
        BlockId::PIECES => world.octant(*half).is_none(),
        block => registry.is_replaceable(block),
    };
    let halves = match piece.aim {
        Some(Aim::Bar(at, axis)) => bar_halves(at, axis).to_vec(),
        Some(Aim::Octant(half)) => vec![half],
        Some(Aim::Shifted(at)) => block_halves(at).to_vec(),
        Some(Aim::Object(..)) | None => Vec::new(),
    };
    target.ghost = halves.into_iter().filter(free).map(half_box).collect();
    // An object in hand: the box it would stand in.
    if let Some(Aim::Object(at, object)) = piece.aim
        && let Some(def) = objects.def(object)
    {
        let placed = PlacedObject { name: def.name.clone(), at, facing: 0, stage: 0, since: 0 };
        target.ghost = vec![object_box(&placed, &objects)];
    }
    // The piece aimed at in a cell of pieces: a bar and what's stuck to it, or a lone octant. Or the
    // object aimed at, its box.
    target.piece = target.half.map_or_else(Vec::new, |half| world.piece_halves(half).into_iter().map(half_box).collect());
    if let Some(at) = piece.aimed_object
        && let Some(object) = world.objects_near(at.div_euclid(IVec3::splat(16))).iter().find(|object| object.at == at)
    {
        target.piece = vec![object_box(object, &objects)];
    }
}

/// Where a respawn puts your feet: on top of the highest block at the world's spawn spot (the
/// middle of the test cylinder, or the spot chosen on a compiled world's map).
fn spawn_point(world: &VoxelWorld) -> Vec3 {
    let spawn = world.spawn();
    let chunk = ChunkPos::of_block(IVec3::new(spawn.x, spawn.y, 0));
    let local = spawn.rem_euclid(IVec2::splat(CHUNK_SIZE));
    let top = world
        .chunk(chunk)
        .and_then(|chunk| chunk.surface_at(local.x, local.y))
        .map_or(WORLD_HEIGHT, |(z, _)| z + 1);
    Vec3::new(spawn.x as f32 + 0.5, spawn.y as f32 + 0.5, top as f32)
}

fn spawn_pawn(mut commands: Commands, world: Res<VoxelWorld>) {
    let feet = spawn_point(&world);
    commands.spawn((
        Pawn::default(),
        SimPosition::new(feet),
        PawnInput::default(),
        PlayerId(0),
        LocalPlayer,
        Presence(feet),
    ));
}

/// The player's presence (what keeps the world around them local) is wherever they are: the
/// flying camera while flying, the body while walking.
fn update_presence(
    mode: Res<ControlMode>,
    camera: Query<&FlyCamera>,
    mut pawns: Query<(&SimPosition, &mut Presence), With<LocalPlayer>>,
) {
    for (position, mut presence) in &mut pawns {
        presence.0 = match (*mode, camera.single()) {
            (ControlMode::Fly, Ok(camera)) => camera.position,
            _ => position.current,
        };
    }
}

// ── Saved with the world ────────────────────────────────────────────────────

/// What `player.toml` holds.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct PlayerSave {
    /// Where the feet are (game axes).
    feet: [f32; 3],
    /// Where the flying camera is.
    camera: [f32; 3],
    heading: f32,
    pitch: f32,
    flying: bool,
    selected_block: usize,
}

fn save_player(world: &World, folder: &WorldFolder) -> Result<String, String> {
    let mut save = PlayerSave {
        flying: *world.resource::<ControlMode>() == ControlMode::Fly,
        selected_block: world.resource::<Hotbar>().selected,
        ..default()
    };
    if let Some(mut query) = world.try_query_filtered::<&SimPosition, With<LocalPlayer>>() {
        if let Some(position) = query.iter(world).next() {
            save.feet = position.current.to_array();
        }
    }
    if let Some(mut query) = world.try_query::<&FlyCamera>() {
        if let Some(camera) = query.iter(world).next() {
            save.camera = camera.position.to_array();
            save.heading = camera.heading;
            save.pitch = camera.pitch;
        }
    }
    folder.write_toml(&save)?;
    Ok(String::new())
}

fn load_player(world: &mut World, folder: &WorldFolder) -> Result<String, String> {
    let Some(save) = folder.read_toml::<PlayerSave>()? else {
        fresh_player(world, folder.seed);
        return Ok("not in this world, put at the spawn point".to_string());
    };
    *world.resource_mut::<ControlMode>() = if save.flying { ControlMode::Fly } else { ControlMode::Walk };
    world.resource_mut::<Hotbar>().selected = save.selected_block;
    for mut camera in world.query::<&mut FlyCamera>().iter_mut(world) {
        camera.position = Vec3::from_array(save.camera);
        camera.heading = save.heading;
        camera.pitch = save.pitch;
    }
    put_pawn_at(world, Vec3::from_array(save.feet));
    Ok(String::new())
}

/// How far above the ground a fresh world drops you [David, 2026-10-27].
const DROPPED_FROM: f32 = 3.0;

/// Slower than this (blocks a second), with no wish to move, is standing still (see the feet).
const STOPPED: f32 = 0.05;

/// A fresh world: walking, dropped from a little above the spawn point, facing north (level).
fn fresh_player(world: &mut World, _seed: u64) {
    *world.resource_mut::<ControlMode>() = ControlMode::Walk;
    let feet = spawn_point(world.resource::<VoxelWorld>()) + Vec3::Z * DROPPED_FROM;
    let mut cameras = world.query::<&mut FlyCamera>();
    for mut camera in cameras.iter_mut(world) {
        (camera.heading, camera.pitch) = (0.0, 0.0);
    }
    put_pawn_at(world, feet);
}

/// Move the local player's body to `feet` (nothing happens before the body exists).
fn put_pawn_at(world: &mut World, feet: Vec3) {
    world.resource_scope(|world, settings: Mut<PlayerSettings>| {
        world.resource_scope(|world, voxel: Mut<VoxelWorld>| {
            world.resource_scope(|world, registry: Mut<BlockRegistry>| {
                let mut pawns = world.query_filtered::<(&mut Pawn, &mut SimPosition, &mut Presence), With<LocalPlayer>>();
                for (mut pawn, mut position, mut presence) in pawns.iter_mut(world) {
                    place_pawn(&mut pawn, &mut position, feet, &settings, &voxel, &registry);
                    presence.0 = position.current;
                }
            });
        });
    });
}

/// Are the keys free for the game (not being typed into a debug panel box)?
fn keys_are_ours(capture: &MouseCapture, egui_input: &EguiWantsInput) -> bool {
    capture.captured || !egui_input.wants_any_keyboard_input()
}

/// Put the pawn at `feet` with no speed; if that's inside blocks, move it up until it isn't.
fn place_pawn(
    pawn: &mut Pawn,
    position: &mut SimPosition,
    feet: Vec3,
    settings: &PlayerSettings,
    world: &VoxelWorld,
    registry: &BlockRegistry,
) {
    let mut feet = feet;
    let solid = |half: IVec3| world.half_filled(half, |block| registry.is_solid(block));
    while overlaps_solid_halves(feet, settings.body(), solid) && feet.z < WORLD_HEIGHT as f32 {
        feet.z = feet.z.floor() + 1.0;
    }
    position.teleport(feet);
    pawn.velocity = Vec3::ZERO;
    pawn.on_ground = false;
    pawn.walk = EnvelopeState::default();
    pawn.run = EnvelopeState::default();
    pawn.sprint = EnvelopeState::default();
    pawn.momentum = EnvelopeState::default();
    pawn.sprinting = false;
    pawn.crouched = false;
    pawn.crouch = EnvelopeState::default();
    pawn.crouch_previous = 0.0;
}

/// The friction of the ground under a body: the average of the solid blocks just below its
/// centre and four corners (so standing half over an edge blends the two).
fn ground_friction(feet: Vec3, size: BodySize, world: &VoxelWorld, registry: &BlockRegistry) -> f32 {
    let half = size.width / 2.0;
    let below = feet.z - 0.01;
    let mut total = 0.0;
    let mut count = 0;
    for (dx, dy) in [(0.0, 0.0), (-half, -half), (half, -half), (-half, half), (half, half)] {
        let cell = Vec3::new(feet.x + dx, feet.y + dy, below).floor().as_ivec3();
        let block = world.get_block(cell);
        if registry.is_solid(block) {
            total += registry.def(block).map_or(1.0, |def| def.friction);
            count += 1;
        }
    }
    if count == 0 { 1.0 } else { total / count as f32 }
}

/// F switches mode. Walking starts where the flying camera is; flying starts where your eyes were.
fn toggle_mode(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<MouseCapture>,
    egui_input: Res<EguiWantsInput>,
    mut mode: ResMut<ControlMode>,
    settings: Res<PlayerSettings>,
    world: Res<VoxelWorld>,
    registry: Res<BlockRegistry>,
    camera: Query<&FlyCamera>,
    mut pawns: Query<(&mut Pawn, &mut SimPosition), With<LocalPlayer>>,
) {
    if !keys.just_pressed(KeyCode::KeyF) || !keys_are_ours(&capture, &egui_input) {
        return;
    }
    *mode = match *mode {
        ControlMode::Fly => {
            if let (Ok(camera), Ok((mut pawn, mut position))) = (camera.single(), pawns.single_mut()) {
                let feet = camera.position - Vec3::Z * settings.eye_height;
                place_pawn(&mut pawn, &mut position, feet, &settings, &world, &registry);
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
    let keys_in_order = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];
    for (slot, key) in keys_in_order.into_iter().enumerate() {
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
    settings: Res<PlayerSettings>,
    time: Res<Time<Real>>,
    camera: Query<&FlyCamera>,
    mut inputs: Query<&mut PawnInput, With<LocalPlayer>>,
) {
    let Ok(mut input) = inputs.single_mut() else {
        return;
    };
    if *mode != ControlMode::Walk || !keys_are_ours(&capture, &egui_input) {
        input.direction = Vec3::ZERO;
        input.jump_held = false;
        input.run = false;
        input.sprint = false;
        input.crouch = false;
        return;
    }
    let Ok(camera) = camera.single() else {
        return;
    };
    let heading = camera.heading.to_radians();
    let forward = Vec3::new(heading.cos(), heading.sin(), 0.0);
    let right = Vec3::new(-heading.sin(), heading.cos(), 0.0);
    // First in "body" terms (x = forwards, y = to the right), so sideways and backwards
    // can be slowed down; then turned to face where the camera looks.
    let mut wanted = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        wanted.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        wanted.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        wanted.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        wanted.y -= 1.0;
    }
    let mut wanted = wanted.normalize_or_zero();
    if wanted.x < 0.0 {
        wanted.x *= settings.backward_factor;
    }
    wanted.y *= settings.strafe_factor;
    input.direction = forward * wanted.x + right * wanted.y;

    // Shift held = run. A second press soon after the first = sprint, for as long as Shift
    // stays held and you keep moving forwards. Ctrl held = crouch.
    let shift = [KeyCode::ShiftLeft, KeyCode::ShiftRight];
    input.run = keys.any_pressed(shift);
    input.crouch = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if keys.any_just_pressed(shift) {
        let now = time.elapsed_secs();
        if now - input.last_run_press <= settings.sprint_double_tap && wanted.x > 0.0 {
            input.sprint = true;
        }
        input.last_run_press = now;
    }
    if !input.run || wanted.x <= 0.0 {
        input.sprint = false;
    }

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
    (registry, objects): (Res<BlockRegistry>, Res<ObjectRegistry>),
    time_control: Res<TimeControl>,
    mut footsteps: MessageWriter<Footstep>,
    mut landings: MessageWriter<Landed>,
    mut pawns: Query<(&mut Pawn, &mut SimPosition, &mut PawnInput, &PlayerId, Has<LocalPlayer>)>,
) {
    let dt = time.delta_secs();
    let tick = time_control.tick();
    // Blocks, and solid objects (a brazier), are bumped into.
    let solid = |half: IVec3| world.half_filled(half, |block| registry.is_solid(block)) || solid_object_half(&world, &objects, half);
    for (mut pawn, mut position, mut input, &player, local) in &mut pawns {
        let jump = input.jump_held || input.jump_tapped;
        input.jump_tapped = false;
        // While this keyboard's player is flying the developer camera, its body waits.
        if local && *mode != ControlMode::Walk {
            continue;
        }
        // Crouching: Ctrl held, or no room to stand up (you stay crouched under a low ceiling).
        let room_to_stand = !overlaps_solid_halves(position.current, settings.body(), solid);
        pawn.crouched = input.crouch || (pawn.crouched && !room_to_stand);
        let crouched = pawn.crouched;
        pawn.crouch_previous = pawn.crouch.level;
        pawn.crouch.update(&settings.crouch_envelope, crouched, dt);
        let body = settings.body_for(&pawn);

        // The legs. Gates: walk = any movement key held; run = Shift held while moving;
        // sprint = Shift double-tapped and held while moving forwards. With feet off the
        // ground the envelopes stay where they are (unless the debug setting says otherwise),
        // since you can't push off the air.
        let moving = input.direction != Vec3::ZERO;
        if moving {
            pawn.last_direction = input.direction;
        }
        pawn.sprinting = input.sprint && moving;
        if pawn.on_ground || settings.envelopes_in_air {
            let sprinting = pawn.sprinting;
            pawn.walk.update(&settings.walk_envelope, moving, dt);
            pawn.run.update(&settings.run_envelope, input.run && moving, dt);
            pawn.sprint.update(&settings.sprint_envelope, sprinting, dt);
            // Momentum charges while you're pushing on at a run. While you're sliding to a stop
            // the charge is held (the slide is what spends it); otherwise it fades. In the air
            // it stays as it was, so it carries through a jump.
            let speed = Vec2::new(pawn.velocity.x, pawn.velocity.y).length();
            let sliding = !moving && speed > 0.05;
            if !sliding {
                pawn.momentum.update(&settings.momentum_envelope, input.run && moving, dt);
            }
            if pawn.walk.just_triggered() {
                footsteps.write(Footstep {
                    player,
                    position: position.current,
                    tick,
                });
            }
        }
        // Walk speed, plus the run envelope's share of the step up to running, plus the sprint
        // envelope's share of the step up to sprinting.
        let full_speed = settings.walk_speed
            + (settings.run_speed - settings.walk_speed) * pawn.run.level
            + (settings.sprint_speed - settings.run_speed) * pawn.sprint.level;
        // Crouching slows you down, blended by the crouch envelope.
        let crouch_factor = 1.0 + (settings.crouch_speed - 1.0) * pawn.crouch.level;
        let target = pawn.last_direction * full_speed * pawn.walk.level * crouch_factor;

        // The feet: your real speed eases towards what the legs want. On the ground, how
        // quickly depends on grip × the block's friction (so you slide a little, more on
        // slippery blocks). In the air you can only steer gradually (air control).
        let horizontal = Vec3::new(pawn.velocity.x, pawn.velocity.y, 0.0);
        let rate = if pawn.on_ground {
            pawn.ground_friction = ground_friction(position.current, body, &world, &registry);
            settings.ground_grip * pawn.ground_friction
        } else {
            settings.air_control
        };
        let mut new_horizontal = horizontal.lerp(target, 1.0 - (-rate * dt).exp());

        // Momentum: when the legs want less speed than you have, a charged body brakes evenly,
        // hard enough that a full charge stops a full sprint in exactly `slide_distance` blocks.
        // Slower speeds slide less (distance grows with speed squared, like a real body), and
        // slippery ground slides further. It only ever makes stopping gentler, never harsher.
        let speed = horizontal.length();
        let slowing = target.length() < speed - 0.01 || target.dot(horizontal) < 0.0;
        let charge = pawn.momentum.level;
        if pawn.on_ground && slowing && charge > 0.001 && settings.slide_distance > 0.0 {
            let distance = settings.slide_distance * charge;
            let braking = settings.sprint_speed.powi(2) / (2.0 * distance) * pawn.ground_friction;
            let change = target - horizontal;
            let gentle_step = (braking * dt).min((new_horizontal - horizontal).length());
            new_horizontal = horizontal + change.clamp_length_max(gentle_step);
        }
        // Easing toward a stop never quite gets there: a sliver of speed was left, and far from the
        // world's middle (where a position can only change in steps of about 0.002 blocks) it
        // tipped you to the next step and back again, a dozen times a second, so the view quivered
        // while you stood still [2026-10-28]. Not trying to move, and barely moving: stopped.
        let mut horizontal = new_horizontal;
        if target.length_squared() == 0.0 && horizontal.length() < STOPPED {
            horizontal = Vec3::ZERO;
        }
        pawn.velocity.x = horizontal.x;
        pawn.velocity.y = horizontal.y;

        // Jumping: the launch speed that rises exactly `jump_height` under this gravity.
        if pawn.on_ground && jump {
            pawn.velocity.z = (2.0 * settings.gravity * settings.jump_height).sqrt();
        }
        pawn.velocity.z = (pawn.velocity.z - settings.gravity * dt).max(-settings.max_fall_speed);

        // Edge guard: crouched on the ground (and not jumping), a step that would leave
        // nothing under you is cancelled, one direction at a time, so you can still edge along.
        let mut motion = pawn.velocity * dt;
        if crouched && settings.crouch_edge_guard && pawn.on_ground && pawn.velocity.z <= 0.0 {
            let feet = position.current;
            let supported = |offset: Vec3| has_support_halves(feet + offset, body, solid);
            if !supported(Vec3::new(motion.x, 0.0, 0.0)) {
                motion.x = 0.0;
                pawn.velocity.x = 0.0;
            }
            if !supported(Vec3::new(0.0, motion.y, 0.0)) {
                motion.y = 0.0;
                pawn.velocity.y = 0.0;
            }
            if !supported(Vec3::new(motion.x, motion.y, 0.0)) {
                motion.x = 0.0;
                motion.y = 0.0;
                pawn.velocity.x = 0.0;
                pawn.velocity.y = 0.0;
            }
        }

        let falling_speed = -pawn.velocity.z;
        let (feet, blocked) = move_body_stepping(position.current, body, motion, pawn.on_ground, solid);
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
        // Touchdown: announce it (a hook for landing sounds, fall damage, stumbles…).
        if blocked.down && !pawn.on_ground {
            landings.write(Landed {
                player,
                position: feet,
                speed: falling_speed.max(0.0),
                tick,
            });
        }
        pawn.on_ground = blocked.down;

        if feet.z < settings.respawn_below_z {
            info!("Fell out of the world: back to the spawn point");
            let spawn = spawn_point(&world);
            place_pawn(&mut pawn, &mut position, spawn, &settings, &world, &registry);
        }
    }
}

/// While walking, the camera sits at the pawn's eyes (drawn smoothly between ticks).
fn follow_pawn(
    mode: Res<ControlMode>,
    settings: Res<PlayerSettings>,
    fixed_time: Res<Time<Fixed>>,
    smoothing: Res<InterpolationSettings>,
    pawns: Query<(&Pawn, &SimPosition), With<LocalPlayer>>,
    mut cameras: Query<&mut FlyCamera>,
) {
    if *mode != ControlMode::Walk {
        return;
    }
    let (Ok((pawn, position)), Ok(mut camera)) = (pawns.single(), cameras.single_mut()) else {
        return;
    };
    // Eyes drop while crouching, blended by the crouch envelope. The envelope moves on ticks,
    // so it's drawn part-way between its last two values, like the position.
    let crouch = pawn.crouch_previous + (pawn.crouch.level - pawn.crouch_previous) * blend_fraction(&fixed_time, &smoothing);
    let eyes = settings.eye_height + (settings.crouch_eye_height - settings.eye_height) * crouch;
    camera.position = position.display(&fixed_time, &smoothing) + Vec3::Z * eyes;
}

/// Sprinting widens the field of view, shaped by its own envelope. This runs every frame
/// (not on ticks) so the view changes smoothly; it still stops when time is paused.
fn widen_view_when_sprinting(
    time: Res<Time>,
    mode: Res<ControlMode>,
    settings: Res<PlayerSettings>,
    pawns: Query<&Pawn, With<LocalPlayer>>,
    mut view: ResMut<SprintView>,
    mut boost: ResMut<FieldOfViewBoost>,
) {
    let sprinting = *mode == ControlMode::Walk && pawns.single().is_ok_and(|pawn| pawn.sprinting);
    let level = view.envelope.update(&settings.sprint_fov_envelope, sprinting, time.delta_secs());
    boost.set_if_neq(FieldOfViewBoost(settings.sprint_fov_boost * level));
}

/// Remember the last footstep (for the "STEP" light) and the last landing (for the debug page).
fn notice_footsteps(
    time: Res<Time<Real>>,
    mut footsteps: MessageReader<Footstep>,
    mut landings: MessageReader<Landed>,
    mut flash: ResMut<FootstepFlash>,
) {
    for step in footsteps.read() {
        flash.last = Some(time.elapsed_secs());
        flash.last_position = step.position;
        flash.count += 1;
    }
    for landing in landings.read() {
        flash.landings += 1;
        flash.last_landing_speed = landing.speed;
        flash.last_landing_position = landing.position;
    }
}

/// Left click breaks, right click places. Held buttons repeat after the cooldown.
/// These send requests; the world carries them out on the next tick. While time is paused
/// your hands are frozen with everything else, so the request is refused straight away.
fn block_actions(
    mouse: Res<ButtonInput<MouseButton>>,
    capture: Res<MouseCapture>,
    mode: Res<ControlMode>,
    time: Res<Time<Real>>,
    time_control: Res<TimeControl>,
    settings: Res<PlayerSettings>,
    target: Res<LookTarget>,
    hotbar: Res<Hotbar>,
    registry: Res<BlockRegistry>,
    pawns: Query<(&SimPosition, &PlayerId), With<LocalPlayer>>,
    mut cooldowns: ResMut<ActionCooldowns>,
    mut requests: MessageWriter<BlockEditRequest>,
    mut refused: MessageWriter<BlockEditRefused>,
    (piece, world): (Res<PieceChoice>, Res<VoxelWorld>),
) {
    // With the mouse free, clicks belong to the debug panel.
    if !capture.captured {
        return;
    }
    let Some(hit) = target.hit else {
        return;
    };
    let Ok((position, &player)) = pawns.single() else {
        return;
    };
    let now = time.elapsed_secs();
    let mut send = |request: BlockEditRequest| {
        if time_control.is_running() {
            requests.write(request);
        } else {
            refused.write(BlockEditRefused {
                request,
                reason: RefuseReason::TimePaused,
                tick: time_control.tick(),
            });
        }
    };

    if mouse.pressed(MouseButton::Left) && now - cooldowns.last_break >= settings.break_cooldown {
        cooldowns.last_break = now;
        // An object aimed at (nearer than the block) is taken away. In a cell of pieces, just the
        // piece aimed at (a bar and what's stuck to it, or an octant).
        let kind = match (piece.aimed_object, target.half.filter(|&half| world.octant(half).is_some())) {
            (Some(at), _) => EditKind::BreakObject { at },
            (None, Some(half)) => EditKind::BreakPiece { half },
            (None, None) => EditKind::Break,
        };
        send(BlockEditRequest {
            position: hit.block,
            kind,
            source: EditSource::Player(player),
        });
    }

    if mouse.pressed(MouseButton::Right) && now - cooldowns.last_place >= settings.place_cooldown {
        // An object: standing where the selector shows it.
        if let Some(Aim::Object(at, object)) = piece.aim {
            cooldowns.last_place = now;
            send(BlockEditRequest { position: at.div_euclid(IVec3::splat(16)), kind: EditKind::PlaceObject { at, object, facing: 0 }, source: EditSource::Player(player) });
            return;
        }
        let Some(block) = hotbar.slots.get(hotbar.selected).and_then(|name| registry.id(name)) else {
            return;
        };
        // A bar (a bar block selected, aimed at a face): where the selector shows it. Else a whole
        // block, in the cell in front of the face.
        let (place, (low, high)) = match piece.aim {
            Some(Aim::Bar(at, axis)) => {
                let [first, last] = bar_halves(at, axis);
                (BlockEditRequest { position: at.div_euclid(IVec3::splat(2)), kind: EditKind::PlaceBar { at, axis, block }, source: EditSource::Player(player) }, (half_box(first).0, half_box(last).1))
            }
            Some(Aim::Octant(half)) => (BlockEditRequest { position: half.div_euclid(IVec3::splat(2)), kind: EditKind::PlaceOctant { half, block }, source: EditSource::Player(player) }, half_box(half)),
            Some(Aim::Shifted(at)) => (BlockEditRequest { position: at.div_euclid(IVec3::splat(2)), kind: EditKind::PlaceShifted { at, block }, source: EditSource::Player(player) }, (half_box(at).0, half_box(at).0 + Vec3::ONE)),
            // (Placed above.)
            Some(Aim::Object(..)) => return,
            None => {
                let Some(cell) = hit.in_front() else {
                    return;
                };
                (BlockEditRequest { position: cell, kind: EditKind::Place(block), source: EditSource::Player(player) }, (cell.as_vec3(), cell.as_vec3() + Vec3::ONE))
            }
        };
        // Don't place something solid inside your own body while walking.
        if *mode == ControlMode::Walk && registry.is_solid(block) {
            let (min, max) = body_bounds(position.current, settings.body());
            if min.cmplt(high).all() && max.cmpgt(low).all() {
                return;
            }
        }
        cooldowns.last_place = now;
        send(place);
    }
}

/// Placeholder block bar at the bottom of the screen: black boxes, magenta outlines, pink text.
fn draw_hotbar(
    mut contexts: EguiContexts,
    hotbar: Res<Hotbar>,
    mode: Res<ControlMode>,
    (registry, objects): (Res<BlockRegistry>, Res<ObjectRegistry>),
    time: Res<Time<Real>>,
    (flash, piece): (Res<FootstepFlash>, Res<PieceChoice>),
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
            .and_then(|id| registry.def(id))
            .map(|def| def.display_name.as_str())
            .or_else(|| objects.by_name(name).map(|def| def.display_name.as_str()))
            .unwrap_or(name.as_str());
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{} {label}", slot + 1),
            egui::FontId::proportional(14.0),
            pink,
        );
        left += slot_size.x + gap;
    }

    // With a bar block in hand: the quarter-block controls, in small magenta along the top (David:
    // "include a very small line of magenta text at top with the controls").
    let bar_in_hand = hotbar.slots.get(hotbar.selected).and_then(|name| registry.id(name)).and_then(|id| registry.def(id)).is_some_and(|def| def.bar.is_some());
    if bar_in_hand {
        painter.text(
            egui::pos2(screen.center().x, screen.top() + 4.0),
            egui::Align2::CENTER_TOP,
            format!("B turns it ({}: sideways · longways · upways)   right click places   Alt + right click an octblock   Shift + right click a whole block off the grid   left click mines a piece", piece.turn.name()),
            egui::FontId::proportional(11.0),
            magenta,
        );
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

    // The footstep light, to the left of the bar: lit pink for a moment after each footstep.
    let lit = flash.last.is_some_and(|last| time.elapsed_secs() - last < 0.12);
    let light = egui::Rect::from_min_size(
        egui::pos2(screen.center().x - total_width / 2.0 - 56.0, top),
        egui::vec2(48.0, slot_size.y),
    );
    let (fill, text) = if lit { (pink, egui::Color32::BLACK) } else { (egui::Color32::BLACK, pink) };
    painter.rect(light, 0.0, fill, egui::Stroke::new(1.0, magenta), egui::StrokeKind::Inside);
    painter.text(light.center(), egui::Align2::CENTER_CENTER, "STEP", egui::FontId::proportional(12.0), text);
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
    let mut live_walk = None;
    let mut live_run = None;
    let mut live_sprint = None;
    let mut live_crouch = None;
    let mut live_momentum = None;
    let live_view = world.resource::<SprintView>().envelope;
    {
        let mut query = world.query_filtered::<(&Pawn, &SimPosition), With<LocalPlayer>>();
        if let Ok((pawn, position)) = query.single(world) {
            live_walk = Some(pawn.walk);
            live_run = Some(pawn.run);
            live_sprint = Some(pawn.sprint);
            live_crouch = Some(pawn.crouch);
            live_momentum = Some(pawn.momentum);
            ui.strong("Body");
            let feet = position.current;
            ui.label(format!("Feet at: {:.2}, {:.2}, {:.2}", feet.x, feet.y, feet.z));
            let horizontal = Vec2::new(pawn.velocity.x, pawn.velocity.y).length();
            ui.label(format!(
                "Speed: {horizontal:.2} blocks/s across, {:.2} up/down",
                pawn.velocity.z
            ));
            ui.label(format!(
                "{}{}",
                if pawn.on_ground { "On the ground" } else { "In the air" },
                if pawn.crouched { ", crouched" } else { "" }
            ));
            ui.label(format!(
                "Legs: walk {:.0}% ({:?})  run {:.0}% ({:?})  sprint {:.0}% ({:?})",
                pawn.walk.level * 100.0,
                pawn.walk.stage,
                pawn.run.level * 100.0,
                pawn.run.stage,
                pawn.sprint.level * 100.0,
                pawn.sprint.stage
            ));
            ui.label(format!("Momentum charge: {:.0}% ({:?})", pawn.momentum.level * 100.0, pawn.momentum.stage));
            ui.label(format!("Ground friction under feet: {:.2}", pawn.ground_friction));
            let flash = world.resource::<FootstepFlash>();
            let at = flash.last_position;
            ui.label(format!("Footsteps so far: {}   last at {:.1}, {:.1}, {:.1}", flash.count, at.x, at.y, at.z));
            let landed = flash.last_landing_position;
            ui.label(format!(
                "Landings so far: {}   last at {:.1}, {:.1}, {:.1}, hitting the ground at {:.1} blocks/s",
                flash.landings, landed.x, landed.y, landed.z, flash.last_landing_speed
            ));
        }
    }
    if ui.button("Respawn at the spawn point (and walk)").clicked() {
        respawn = true;
    }
    if respawn {
        let settings = world.resource::<PlayerSettings>().clone();
        let spawn = spawn_point(world.resource::<VoxelWorld>());
        world.resource_scope(|world, voxel: Mut<VoxelWorld>| {
            world.resource_scope(|world, registry: Mut<BlockRegistry>| {
                let mut query = world.query_filtered::<(&mut Pawn, &mut SimPosition), With<LocalPlayer>>();
                if let Ok((mut pawn, mut position)) = query.single_mut(world) {
                    place_pawn(&mut pawn, &mut position, spawn, &settings, &voxel, &registry);
                }
            });
        });
        *world.resource_mut::<ControlMode>() = ControlMode::Walk;
    }

    // ── Settings ──
    ui.separator();
    {
        let mut settings = world.resource_mut::<PlayerSettings>();
        // Saves / loads everything in this section (not the bottom-layer rule below).
        preset_bar(ui, "player", &mut *settings);

        ui.strong("Movement (Minecraft defaults)");
        ui.add(crate::debug::Slider::new(&mut settings.walk_speed, 0.5..=20.0).text("walk speed (blocks/s)"));
        ui.add(crate::debug::Slider::new(&mut settings.run_speed, 0.5..=30.0).text("run speed (Shift)"));
        ui.add(crate::debug::Slider::new(&mut settings.sprint_speed, 0.5..=40.0).text("sprint speed (Shift Shift)"));
        ui.add(crate::debug::Slider::new(&mut settings.sprint_double_tap, 0.05..=1.0).text("sprint double-tap window (s)"));
        ui.add(crate::debug::Slider::new(&mut settings.strafe_factor, 0.0..=1.5).text("sideways speed × forwards"));
        ui.add(crate::debug::Slider::new(&mut settings.backward_factor, 0.0..=1.5).text("backwards speed × forwards"));
        ui.add(crate::debug::Slider::new(&mut settings.ground_grip, 0.5..=60.0).logarithmic(true).text("ground grip (× block friction)"));
        ui.small("Lower grip = more slide when stopping, turning and landing.");

        ui.small("Envelopes: square handles set times and sustain; round ones bend curves (double-click straightens).");
        ui.strong("Walk envelope (legs pushing)");
        envelope_editor(ui, "walk", &mut settings.walk_envelope, live_walk.as_ref());
        ui.small("The walk envelope firing is a footstep (the STEP light by the block bar).");
        ui.strong("Run envelope (walk → run)");
        envelope_editor(ui, "run", &mut settings.run_envelope, live_run.as_ref());
        ui.strong("Sprint envelope (run → sprint)");
        envelope_editor(ui, "sprint", &mut settings.sprint_envelope, live_sprint.as_ref());
        ui.checkbox(&mut settings.envelopes_in_air, "envelopes keep moving in the air");

        ui.strong("Momentum (charged by running, spent sliding when you stop)");
        ui.small("Attack = how long running takes to charge; sustain = how much charge. The charge is held while you slide; release = how fast it fades if you just slow to a walk.");
        envelope_editor(ui, "momentum", &mut settings.momentum_envelope, live_momentum.as_ref());
        ui.add(crate::debug::Slider::new(&mut settings.slide_distance, 0.0..=20.0).text("slide from a full sprint at full charge (blocks)"));

        ui.strong("Sprint view widening");
        ui.add(crate::debug::Slider::new(&mut settings.sprint_fov_boost, -30.0..=40.0).text("extra field of view at full sprint (°)"));
        envelope_editor(ui, "sprint_fov", &mut settings.sprint_fov_envelope, Some(&live_view));

        ui.strong("Crouch (Ctrl)");
        let standing = settings.body_height;
        ui.add(crate::debug::Slider::new(&mut settings.crouch_height, 0.3..=standing).text("crouched height"));
        let crouched_height = settings.crouch_height;
        ui.add(crate::debug::Slider::new(&mut settings.crouch_eye_height, 0.1..=crouched_height).text("crouched eye height"));
        ui.add(crate::debug::Slider::new(&mut settings.crouch_speed, 0.0..=1.0).text("crouched speed (× normal)"));
        ui.checkbox(&mut settings.crouch_edge_guard, "won't step off edges while crouched");
        envelope_editor(ui, "crouch", &mut settings.crouch_envelope, live_crouch.as_ref());

        ui.strong("Jumping & falling");
        ui.add(crate::debug::Slider::new(&mut settings.jump_height, 0.1..=5.0).text("jump height (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.gravity, 1.0..=100.0).text("gravity (blocks/s²)"));
        ui.add(crate::debug::Slider::new(&mut settings.max_fall_speed, 1.0..=200.0).text("max fall speed"));
        ui.add(crate::debug::Slider::new(&mut settings.air_control, 0.0..=20.0).text("air control"));

        ui.strong("Body (dwarf)");
        ui.add(crate::debug::Slider::new(&mut settings.body_height, 0.5..=3.0).text("height"));
        ui.add(crate::debug::Slider::new(&mut settings.body_width, 0.2..=0.98).text("width"));
        let height = settings.body_height;
        ui.add(crate::debug::Slider::new(&mut settings.eye_height, 0.1..=height).text("eye height"));
        if settings.eye_height > settings.body_height {
            settings.eye_height = settings.body_height;
        }

        ui.strong("Breaking & placing");
        ui.add(crate::debug::Slider::new(&mut settings.reach, 1.0..=20.0).text("reach while walking (blocks)"));
        ui.add(crate::debug::Slider::new(&mut settings.break_cooldown, 0.0..=2.0).text("break repeat (s)"));
        ui.add(crate::debug::Slider::new(&mut settings.place_cooldown, 0.0..=2.0).text("place repeat (s)"));
        ui.add(crate::debug::Slider::new(&mut settings.respawn_below_z, -400.0..=0.0).text("respawn when below Z"));
        if ui.button("Reset player settings").clicked() {
            *settings = PlayerSettings::default();
        }
    }
    {
        let mut rules = world.resource_mut::<EditRules>();
        ui.checkbox(&mut rules.protect_bottom_layer, "bottom layer (Z 0) can't be broken");
    }
    {
        // Placing bars (David's control, docs/quartile.md sections 11 to 13).
        let mut piece = world.resource_mut::<PieceChoice>();
        ui.strong("Bars");
        ui.small("With a bar block in the hotbar (pick one below), the selector shows exactly the octants a bar would fill: from the octant in front of the face at your crosshair, along the line it's turned, toward the half of the octant your crosshair is on. B turns it (sideways, longways, upways); right click places it; Alt + right click places a single octant. With any block in hand, Shift + right click places it whole on the octant grid, half a block off the cells, toward the half of the octant your crosshair is on (refused if any of its octants is taken). A bar crossing one already there is cut to its other octant. Left click mines one piece: a bar and what's stuck to it.");
        ui.horizontal(|ui| {
            ui.label("turned:");
            ui.selectable_value(&mut piece.turn, Turn::Sideways, Turn::Sideways.name()).on_hover_text("Across the way you face: right on the right half of the octant, left on its left half.");
            ui.selectable_value(&mut piece.turn, Turn::Longways, Turn::Longways.name()).on_hover_text("The way you face: away on the far half of the octant, toward you on its near half (straight out of a wall).");
            ui.selectable_value(&mut piece.turn, Turn::Upways, Turn::Upways.name()).on_hover_text("Upright: up on the top half of the octant, down on its bottom half (straight out of a floor or ceiling).");
        });
    }
    {
        // Any block or object onto the selected hotbar slot (registered blocks, bar blocks among them;
        // then the objects).
        let blocks: Vec<(String, String, bool)> = {
            let registry = world.resource::<BlockRegistry>();
            let objects = world.resource::<ObjectRegistry>();
            let blocks = registry.defs().iter().skip(1).map(|def| (def.name.clone(), def.display_name.clone(), def.bar.is_some()));
            blocks.chain(objects.defs().iter().map(|def| (def.name.clone(), format!("{} (object)", def.display_name), false))).collect()
        };
        let mut hotbar = world.resource_mut::<Hotbar>();
        let selected = hotbar.selected;
        if let Some(slot) = hotbar.slots.get_mut(selected) {
            ui.horizontal(|ui| {
                ui.label(format!("Hotbar slot {}:", selected + 1));
                let shown = blocks.iter().find(|(name, _, _)| name == slot).map_or(slot.clone(), |(_, display, _)| display.clone());
                egui::ComboBox::from_id_salt("hotbar slot block").selected_text(shown).show_ui(ui, |ui| {
                    for (name, display, bar) in &blocks {
                        let text = if *bar { format!("{display} (can be a bar)") } else { display.clone() };
                        if ui.selectable_label(slot == name, text).on_hover_text(name).clicked() {
                            *slot = name.clone();
                        }
                    }
                });
            });
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_survive_a_preset_round_trip() {
        let mut settings = PlayerSettings::default();
        settings.sprint_envelope.attack = 2.25;
        settings.walk_speed = 3.0;
        let text = toml::to_string_pretty(&settings).expect("saves");
        let loaded: PlayerSettings = toml::from_str(&text).expect("loads");
        assert_eq!(loaded.sprint_envelope, settings.sprint_envelope);
        assert_eq!(loaded.walk_speed, 3.0);
    }

    /// Sprint north on the test world for a few seconds, let go of everything, and measure how
    /// far the body carries on before it stops.
    fn slide_after_sprinting(slide_distance: f32) -> f32 {
        use bevy::ecs::system::RunSystemOnce;
        use crate::world::{TestBlocks, TestWorldSettings, generate_test_world};

        let mut world = World::new();
        let registry = BlockRegistry::default();
        let mut voxel = VoxelWorld::default();
        let terrain = TestWorldSettings { radius: 120.0, ..default() };
        generate_test_world(&mut voxel, &terrain, &TestBlocks::from_registry(&registry));
        let settings = PlayerSettings { slide_distance, ..default() };
        let start = Vec3::new(-100.0, 0.5, terrain.grass_z as f32 + 1.0);
        world.insert_resource(registry);
        world.insert_resource(ObjectRegistry::default());
        world.insert_resource(voxel);
        world.insert_resource(settings);
        world.insert_resource(ControlMode::Walk);
        world.insert_resource(TimeControl::default());
        world.insert_resource(Time::<()>::default());
        world.init_resource::<Messages<Footstep>>();
        world.init_resource::<Messages<Landed>>();
        let pawn = world
            .spawn((
                Pawn { on_ground: true, ..default() },
                SimPosition::new(start),
                PawnInput { direction: Vec3::X, run: true, sprint: true, ..default() },
                PlayerId(0),
            ))
            .id();

        let tick = std::time::Duration::from_secs_f64(1.0 / 30.0);
        let step = |world: &mut World| {
            world.resource_mut::<Time>().advance_by(tick);
            world.run_system_once(move_pawn).expect("runs");
        };
        for _ in 0..(30 * 4) {
            step(&mut world);
        }
        let speed = world.get::<Pawn>(pawn).unwrap().velocity.length();
        assert!(speed > 7.0, "should be sprinting, going {speed}");
        let released_at = world.get::<SimPosition>(pawn).unwrap().current;
        *world.get_mut::<PawnInput>(pawn).unwrap() = PawnInput::default();
        for _ in 0..(30 * 10) {
            step(&mut world);
        }
        let stopped_at = world.get::<SimPosition>(pawn).unwrap().current;
        assert!(world.get::<Pawn>(pawn).unwrap().velocity.length() < 0.01, "should have stopped");
        stopped_at.x - released_at.x
    }

    #[test]
    fn a_full_sprint_slides_about_the_set_distance() {
        let none = slide_after_sprinting(0.0);
        let ten = slide_after_sprinting(10.0);
        assert!(none < 2.0, "no momentum slid {none}");
        assert!((9.5..11.0).contains(&ten), "momentum 10 slid {ten}");
    }

    #[test]
    fn old_presets_with_missing_settings_still_load() {
        let loaded: PlayerSettings = toml::from_str("walk_speed = 2.0").expect("loads");
        assert_eq!(loaded.walk_speed, 2.0);
        assert_eq!(loaded.run_speed, PlayerSettings::default().run_speed);
    }
}

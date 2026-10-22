//! Sandbox — TEMPORARY test helpers, removed piece by piece as real systems arrive.
//!
//! What's here now:
//! - A **viewer camera** you steer with sliders on the "Sandbox" page (until the free-fly
//!   camera arrives in step 4). It circles a target point, looking at it.
//! - Compass markers on the grass: BLUE cube = north (+X), RED cube = east (+Y).
//! - A MAGENTA cube orbiting the centre. It moves only on simulation ticks, so it reacts
//!   to Pause, Speed, Step and the tick rate on the "Time" debug page.

use crate::coords::game_to_bevy;
use crate::debug::DebugAppExt;
use crate::time::SimPosition;
use bevy::prelude::*;
use bevy::render::view::{DebandDither, Tonemapping};
use bevy_egui::egui;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SandboxSettings>()
            .add_systems(Startup, spawn_scene)
            .add_systems(FixedUpdate, orbit)
            .add_systems(Update, place_viewer_camera)
            .add_debug_page("Sandbox", sandbox_debug_page);
    }
}

/// Height of the test world's grass surface (top face of the Z = 144 layer).
const GROUND: f32 = 145.0;

/// Tunables for the test scene (shown on the "Sandbox" debug page).
#[derive(Resource)]
struct SandboxSettings {
    /// Orbit speed in turns per simulated second.
    orbit_turns_per_second: f32,
    /// Orbit radius in blocks.
    orbit_radius: f32,
    /// Viewer camera: compass direction it looks toward (0° = north, 90° = east).
    view_heading_degrees: f32,
    /// Viewer camera: how steeply it looks down (0° = level, 90° = straight down).
    view_tilt_degrees: f32,
    /// Viewer camera: distance from the point it looks at.
    view_distance: f32,
    /// Viewer camera: the point it looks at (game coordinates).
    view_target: Vec3,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            orbit_turns_per_second: 0.25,
            orbit_radius: 3.0,
            view_heading_degrees: 30.0,
            view_tilt_degrees: 30.0,
            view_distance: 130.0,
            view_target: Vec3::new(0.0, 0.0, GROUND),
        }
    }
}

/// Marks the orbiting cube and stores its angle (in turns: 1.0 = a full circle).
#[derive(Component)]
struct Orbiter {
    angle_turns: f32,
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        // Show colours exactly as painted: no film-style colour grading, no dithering noise.
        Tonemapping::None,
        DebandDither::Disabled,
        Transform::default(),
    ));

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let solid = |color: Color| StandardMaterial {
        base_color: color,
        unlit: true, // flat colour, no shading (placeholder look)
        ..default()
    };

    // Compass markers, sitting on the grass.
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(materials.add(solid(Color::srgb(0.1, 0.3, 1.0)))),
        Transform::from_translation(game_to_bevy(Vec3::new(5.0, 0.0, GROUND + 0.5))),
    ));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(materials.add(solid(Color::srgb(1.0, 0.15, 0.1)))),
        Transform::from_translation(game_to_bevy(Vec3::new(0.0, 5.0, GROUND + 0.5))),
    ));

    // The orbiting cube (magenta = placeholder colour).
    let start = Vec3::new(3.0, 0.0, GROUND + 0.5);
    commands.spawn((
        Mesh3d(cube),
        MeshMaterial3d(materials.add(solid(Color::srgb(1.0, 0.0, 1.0)))),
        Transform::from_translation(game_to_bevy(start)),
        SimPosition::new(start),
        Orbiter { angle_turns: 0.0 },
    ));
}

/// Runs once per tick: advance the orbit angle and set the new simulation position.
fn orbit(
    time: Res<Time>,
    settings: Res<SandboxSettings>,
    mut query: Query<(&mut Orbiter, &mut SimPosition)>,
) {
    for (mut orbiter, mut position) in &mut query {
        orbiter.angle_turns =
            (orbiter.angle_turns + settings.orbit_turns_per_second * time.delta_secs()).fract();
        let radians = orbiter.angle_turns * std::f32::consts::TAU;
        position.current = Vec3::new(
            radians.cos() * settings.orbit_radius,
            radians.sin() * settings.orbit_radius,
            GROUND + 0.5,
        );
    }
}

/// Every frame: put the camera where the sliders say.
fn place_viewer_camera(
    settings: Res<SandboxSettings>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let heading = settings.view_heading_degrees.to_radians();
    let tilt = settings.view_tilt_degrees.to_radians();
    // Direction the camera looks, in game axes (heading 0 = north = +X, 90 = east = +Y).
    let looking = Vec3::new(
        heading.cos() * tilt.cos(),
        heading.sin() * tilt.cos(),
        -tilt.sin(),
    );
    let eye = settings.view_target - looking * settings.view_distance;
    for mut transform in &mut camera {
        *transform = Transform::from_translation(game_to_bevy(eye))
            .looking_at(game_to_bevy(settings.view_target), Vec3::Y);
    }
}

fn sandbox_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let mut settings = world.resource_mut::<SandboxSettings>();
    ui.strong("Viewer camera (until free-fly arrives in step 4)");
    ui.add(
        egui::Slider::new(&mut settings.view_heading_degrees, 0.0..=360.0)
            .text("looking toward (° from north)"),
    );
    ui.add(egui::Slider::new(&mut settings.view_tilt_degrees, -10.0..=89.0).text("looking down (°)"));
    ui.add(egui::Slider::new(&mut settings.view_distance, 2.0..=600.0).text("distance"));
    ui.horizontal(|ui| {
        ui.label("Target X");
        ui.add(egui::DragValue::new(&mut settings.view_target.x));
        ui.label("Y");
        ui.add(egui::DragValue::new(&mut settings.view_target.y));
        ui.label("Z");
        ui.add(egui::DragValue::new(&mut settings.view_target.z));
    });
    if ui.button("Reset view").clicked() {
        let defaults = SandboxSettings::default();
        settings.view_heading_degrees = defaults.view_heading_degrees;
        settings.view_tilt_degrees = defaults.view_tilt_degrees;
        settings.view_distance = defaults.view_distance;
        settings.view_target = defaults.view_target;
    }

    ui.separator();
    ui.strong("Test objects");
    ui.label("Blue = north (+X). Red = east (+Y).");
    ui.add(
        egui::Slider::new(&mut settings.orbit_turns_per_second, 0.0..=2.0)
            .text("orbit turns / sec"),
    );
    ui.add(egui::Slider::new(&mut settings.orbit_radius, 0.5..=5.0).text("orbit radius"));
}

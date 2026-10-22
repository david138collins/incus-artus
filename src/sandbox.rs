//! Sandbox — a TEMPORARY test scene for step 1. It is removed once the real world exists.
//!
//! What you see:
//! - A grey floor at the origin.
//! - Compass markers: BLUE cube = north (+X), RED cube = east (+Y). Facing north,
//!   east should appear on your right; this is how we check the coordinate mapping.
//! - A MAGENTA cube orbiting the origin. It moves only on simulation ticks, so it
//!   reacts to Pause, Speed, Step and the tick rate on the "Time" debug page.

use crate::coords::game_to_bevy;
use crate::debug::DebugAppExt;
use crate::time::SimPosition;
use bevy::prelude::*;
use bevy_egui::egui;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SandboxSettings>()
            .add_systems(Startup, spawn_scene)
            .add_systems(FixedUpdate, orbit)
            .add_debug_page("Sandbox", sandbox_debug_page);
    }
}

/// Tunables for the test scene (shown on the "Sandbox" debug page).
#[derive(Resource)]
struct SandboxSettings {
    /// Orbit speed in turns per simulated second.
    orbit_turns_per_second: f32,
    /// Orbit radius in blocks.
    orbit_radius: f32,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            orbit_turns_per_second: 0.25,
            orbit_radius: 3.0,
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
    // Camera: south of the origin and above it, looking north at the origin.
    let eye = game_to_bevy(Vec3::new(-9.0, 0.0, 6.0));
    let target = game_to_bevy(Vec3::ZERO);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(eye).looking_at(target, Vec3::Y),
    ));

    // Sun: shining down at an angle.
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            ..default()
        },
        Transform::from_translation(game_to_bevy(Vec3::new(-4.0, -2.0, 10.0)))
            .looking_at(target, Vec3::Y),
    ));

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let solid = |color: Color| StandardMaterial {
        base_color: color,
        unlit: true, // flat colour, no shading (placeholder look)
        ..default()
    };

    // Floor: 10 × 10 blocks, its top surface at Z = 0.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 0.2, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.25),
            ..default()
        })),
        Transform::from_translation(game_to_bevy(Vec3::new(0.0, 0.0, -0.1))),
    ));

    // Compass markers.
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(materials.add(solid(Color::srgb(0.1, 0.3, 1.0)))),
        Transform::from_translation(game_to_bevy(Vec3::new(5.0, 0.0, 0.5))),
    ));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(materials.add(solid(Color::srgb(1.0, 0.15, 0.1)))),
        Transform::from_translation(game_to_bevy(Vec3::new(0.0, 5.0, 0.5))),
    ));

    // The orbiting cube (magenta = placeholder colour).
    let start = Vec3::new(3.0, 0.0, 0.5);
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
            0.5,
        );
    }
}

fn sandbox_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let mut settings = world.resource_mut::<SandboxSettings>();
    ui.label("Temporary test scene.");
    ui.label("Blue = north (+X). Red = east (+Y).");
    ui.add(
        egui::Slider::new(&mut settings.orbit_turns_per_second, 0.0..=2.0)
            .text("orbit turns / sec"),
    );
    ui.add(egui::Slider::new(&mut settings.orbit_radius, 0.5..=5.0).text("orbit radius"));
}

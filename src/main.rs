//! Incus Artus — program entry point.
//!
//! Bevy games are assembled from *plugins*. A plugin is a self-contained module
//! that adds its own data (resources, components) and behaviour (systems) to the app.
//! `main` lists the plugins and starts the loop; it contains no game logic itself.
//!
//! Modules (one folder or file each):
//! - `debug`   — the developer debug panel; other modules add pages to it
//! - `time`    — TimeControl: the single simulation clock (pause / speed / tick rate)
//! - `coords`  — conversion between game axes (X north, Y east, Z up) and Bevy's axes
//! - `events`  — the event bus: requests, announcements, player IDs, the event log
//! - `data`    — registries of game content (block names ↔ numbers)
//! - `kinematics` — the shapes of motion (ADSR envelopes) and their editor
//! - `save`    — worlds as folders: save, load, new world (each module saves its own part)
//! - `world`   — chunks, sections, block storage and queries
//! - `render`  — meshing, textures, the block shader, culling
//! - `fate`    — the Goddess of Fate: one probability engine for every random roll
//! - `sim`     — the world simulation: local / rich / thin levels, places, journeys, hooks
//! - `people`  — named people: urges that charge up, pressures that lower the line, meetings
//! - `camera`  — the free-fly developer camera and "what am I looking at?" raycast
//! - `player`  — the walking body, fly/walk switching, breaking and placing blocks
//! - `screen`  — which screen is showing: the world, or the World Gen suite (F2)
//! - `worldgen` — the World Gen suite: the workbench the world generator is built in, one tool at a time
//! - `sandbox` — TEMPORARY test helpers (compass markers, orbiting cube)

mod camera;
mod coords;
mod data;
mod debug;
mod events;
mod fate;
mod kinematics;
mod people;
mod player;
mod render;
mod sandbox;
mod save;
mod screen;
mod sim;
mod time;
mod world;
mod worldgen;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

/// How the game talks to the graphics card. On Windows: **DirectX 12**. On 2026-10-24 we found
/// that the other choice, Vulkan, sometimes gets stuck on this kind of machine (an NVIDIA laptop):
/// handing each finished frame to the screen stalls, and the game drops to about 6 frames a second
/// while DirectX 12 runs at 240, back to back. Setting `WGPU_BACKEND` (for example to `vulkan`)
/// still chooses, for testing.
fn graphics_settings() -> bevy::render::settings::WgpuSettings {
    let mut settings = bevy::render::settings::WgpuSettings::default();
    if cfg!(windows) && std::env::var("WGPU_BACKEND").is_err() {
        settings.backends = Some(bevy::render::settings::Backends::DX12);
    }
    settings
}

fn main() {
    App::new()
        // Bevy's standard set: window, input, rendering, assets, audio, etc.
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Incus Artus".into(),
                        resolution: (1280, 960).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin { render_creation: graphics_settings().into(), ..default() }),
        )
        // egui draws the debug panel.
        .add_plugins(EguiPlugin::default())
        // Our modules. Debug goes first so its "Overview" page is the first tab.
        // The order also decides the order parts of a world are saved and loaded in
        // (clock, terrain, fate, sim, people, player): terrain must be back before the player
        // stands on it, and the sim's places before the people who live in them.
        .add_plugins((
            debug::DebugPlugin,
            time::TimeControlPlugin,
            events::EventsPlugin,
            data::DataPlugin,
            save::SavePlugin,
            world::WorldPlugin,
            render::RenderPlugin,
            fate::FatePlugin,
            sim::SimPlugin,
            people::PeoplePlugin,
            camera::CameraPlugin,
            player::PlayerPlugin,
            screen::ScreenPlugin,
            worldgen::WorldGenPlugin,
            sandbox::SandboxPlugin,
        ))
        .add_systems(Startup, fill_the_screen)
        .run();
}

/// Starts the window maximised, so it fills the screen above the taskbar (1280 × 960 is only
/// its size when un-maximised).
fn fill_the_screen(mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    for mut window in &mut windows {
        window.set_maximized(true);
    }
}

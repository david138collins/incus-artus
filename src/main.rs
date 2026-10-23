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
//! - `camera`  — the free-fly developer camera and "what am I looking at?" raycast
//! - `player`  — the walking body, fly/walk switching, breaking and placing blocks
//! - `sandbox` — TEMPORARY test helpers (compass markers, orbiting cube)

mod camera;
mod coords;
mod data;
mod debug;
mod events;
mod fate;
mod kinematics;
mod player;
mod render;
mod sandbox;
mod save;
mod sim;
mod time;
mod world;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        // Bevy's standard set: window, input, rendering, assets, audio, etc.
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Incus Artus".into(),
                resolution: (1280, 960).into(),
                ..default()
            }),
            ..default()
        }))
        // egui draws the debug panel.
        .add_plugins(EguiPlugin::default())
        // Our modules. Debug goes first so its "Overview" page is the first tab.
        // The order also decides the order parts of a world are saved and loaded in
        // (clock, terrain, fate, sim, player): terrain must be back before the player stands on it.
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
            camera::CameraPlugin,
            player::PlayerPlugin,
            sandbox::SandboxPlugin,
        ))
        .run();
}

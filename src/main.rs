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
//! - `data`    — registries of game content (block names ↔ numbers)
//! - `world`   — chunks, sections, block storage and queries
//! - `render`  — meshing, textures, the block shader, culling
//! - `sandbox` — TEMPORARY test helpers (viewer camera, compass markers, orbiting cube)

mod coords;
mod data;
mod debug;
mod render;
mod sandbox;
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
        .add_plugins((
            debug::DebugPlugin,
            time::TimeControlPlugin,
            data::DataPlugin,
            world::WorldPlugin,
            render::RenderPlugin,
            sandbox::SandboxPlugin,
        ))
        .run();
}

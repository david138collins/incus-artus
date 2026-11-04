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
//! - `soak` — the soak test (does the game run worse over time?); `temperatures` — CPU / GPU readout
//! - `diagnostics` — the session log, crash and freeze reports, and whether the last session ended
//!   normally (all in `logs/`)
//!
//! The developer tools (the debug panel, the World Gen suite, sandbox, soak test, temperatures)
//! are switched on by the `dev` feature (Cargo.toml), on by default. Built without it, the game is
//! a player's copy: their code is still compiled, just never switched on, so it's left unused.
#![cfg_attr(not(feature = "dev"), allow(dead_code, unused_imports))]

mod camera;
mod coords;
mod data;
mod debug;
mod diagnostics;
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
mod soak;
mod temperatures;
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
///
/// Its shaders are compiled with **DXC** where it can be found: the old compiler (FXC), Bevy's
/// choice without `dxcompiler.dll` beside the game, took some 50 s over the far view's march at every
/// start, a core flat out and the march not drawn till then; DXC takes under 10 s [found 2026-11-04].
/// `WGPU_DX12_COMPILER` (`fxc`, `dxc`) still chooses, for testing.
fn graphics_settings() -> bevy::render::settings::WgpuSettings {
    let mut settings = bevy::render::settings::WgpuSettings::default();
    if cfg!(windows) && std::env::var("WGPU_BACKEND").is_err() {
        settings.backends = Some(bevy::render::settings::Backends::DX12);
    }
    if cfg!(windows)
        && std::env::var("WGPU_DX12_COMPILER").is_err()
        && matches!(settings.dx12_shader_compiler, bevy::render::settings::Dx12Compiler::Fxc)
        && let Some(dxc) = windows_sdk_dxc()
    {
        settings.dx12_shader_compiler = bevy::render::settings::Dx12Compiler::DynamicDxc { dxc_path: dxc };
    }
    settings
}

/// The Windows SDK's `dxcompiler.dll`, if it is installed (with the `dxil.dll` beside it).
fn windows_sdk_dxc() -> Option<String> {
    let folder = std::path::Path::new(r"C:\Program Files (x86)\Windows Kits\10\Redist\D3D\x64");
    let dxc = folder.join("dxcompiler.dll");
    (dxc.is_file() && folder.join("dxil.dll").is_file()).then(|| dxc.to_string_lossy().into_owned())
}

fn main() {
    // First, so even a crash while the game is being put together leaves a report.
    diagnostics::start();
    let mut app = App::new();
    app
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
                .set(bevy::render::RenderPlugin { render_creation: graphics_settings().into(), ..default() })
                // Everything logged also goes to the session log (see `diagnostics`).
                .set(bevy::log::LogPlugin { custom_layer: diagnostics::log_layer, ..default() }),
        )
        // egui draws the debug panel.
        .add_plugins(EguiPlugin::default())
        // Our modules. Debug goes first so its "Overview" page is the first tab.
        // The order also decides the order parts of a world are saved and loaded in
        // (clock, terrain, fate, sim, people, player): terrain must be back before the player
        // stands on it, and the sim's places before the people who live in them.
        .add_plugins(diagnostics::DiagnosticsPlugin)
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
        ))
        .add_systems(Startup, fill_the_screen);
    // The developer tools: everything a player's copy of the game leaves out (see `dev` in
    // Cargo.toml). Their code is still built either way; only switching them on is skipped.
    #[cfg(feature = "dev")]
    app.add_plugins((worldgen::WorldGenPlugin, sandbox::SandboxPlugin, soak::SoakPlugin, temperatures::TemperaturesPlugin));
    app.run();
    diagnostics::ended_normally();
}

/// Starts the window maximised, so it fills the screen above the taskbar (1280 × 960 is only
/// its size when un-maximised).
fn fill_the_screen(mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    for mut window in &mut windows {
        window.set_maximized(true);
    }
}

//! Incus Artus — program entry point.
//!
//! The game is assembled from *plugins*: each module adds its own data (resources, components) and
//! behaviour (systems). Since the regroup (docs/regroup.md) only Bevy's core is used (the ECS, the
//! scheduler, states, time, input types, logging); the window, the card and the frame loop are ours
//! (`runner`). `main` lists the plugins and starts the loop; it contains no game logic itself.
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
//! - `runner`  — the frame lane: the window, the card, the frame loop and the frame cap
//! - `ui`      — the debug panel's egui, as the game's systems see it
//! - `look`    — the World Compiler's block colours and shading, and the old renderer's kept settings
//! - `perf`    — the frame rate, the steps' times and the slow-frame log
//! - `fate`    — the Goddess of Fate: one probability engine for every random roll
//! - `sim`     — the world simulation: local / rich / thin levels, places, journeys, hooks
//! - `people`  — named people: urges that charge up, pressures that lower the line, meetings
//! - `camera`  — the free-fly developer camera and "what am I looking at?" raycast
//! - `player`  — the walking body, fly/walk switching, breaking and placing blocks
//! - `screen`  — which screen is showing: the world, or the World Gen suite (F2)
//! - `worldgen` — the World Gen suite: the workbench the world generator is built in, one tool at a time
//! - `soak` — the soak test (does the game run worse over time?); `temperatures` — CPU / GPU readout
//! - `diagnostics` — the session log, crash and freeze reports, and whether the last session ended
//!   normally (all in `logs/`)
//!
//! The developer tools (the debug panel, the World Gen suite, soak test, temperatures)
//! are switched on by the `dev` feature (Cargo.toml), on by default. Built without it, the game is
//! a player's copy: their code is still compiled, just never switched on, so it's left unused.
#![cfg_attr(not(feature = "dev"), allow(dead_code, unused_imports))]

mod camera;
mod coords;
mod data;
mod debug;
mod diagnostics;
mod ecs;
mod events;
mod fate;
mod kinematics;
mod look;
mod people;
mod perf;
mod player;
mod runner;
mod save;
mod screen;
mod sim;
mod soak;
mod temperatures;
mod time;
mod ui;
mod world;
mod worldgen;

use crate::ecs::prelude::*;

/// Bevy's worker threads [2026-11-05: its default woke 16 workers every frame, about 47,000 wake-ups a
/// second at the menu]: the game's systems run on one thread (each schedule single-threaded, `main`),
/// so the compute pool keeps one; the async pool (the loading lane's jobs: chunks, the World Compiler)
/// keeps Bevy's own size and sleeps when there's nothing to do. `INCUS_COMPUTE_THREADS=<n>` still sets
/// the compute pool (for measuring).
fn task_pools() -> TaskPoolOptions {
    let mut options = TaskPoolOptions::default();
    let threads = std::env::var("INCUS_COMPUTE_THREADS").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(1).max(1);
    options.compute.min_threads = threads;
    options.compute.max_threads = threads;
    options
}

fn main() {
    // First, so even a crash while the game is being put together leaves a report.
    diagnostics::start();
    let mut app = App::new();
    app
        // Bevy's core only (the regroup): threads, the frame counter, time, input types, states,
        // and logging (which also goes to the session log, see `diagnostics`).
        .add_plugins((
            TaskPoolPlugin { task_pool_options: task_pools() },
            crate::ecs::diagnostic::FrameCountPlugin,
            crate::ecs::time::TimePlugin,
            crate::ecs::input::InputPlugin,
            crate::ecs::state::app::StatesPlugin,
            crate::ecs::log::LogPlugin { custom_layer: diagnostics::log_layer, ..default() },
        ))
        // The debug panel's egui (drawn by the runner).
        .add_plugins((ui::UiPlugin, look::LookPlugin, perf::PerfPlugin))
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
            fate::FatePlugin,
            sim::SimPlugin,
            people::PeoplePlugin,
            camera::CameraPlugin,
            player::PlayerPlugin,
            screen::ScreenPlugin,
        ));
    // The developer tools: everything a player's copy of the game leaves out (see `dev` in
    // Cargo.toml). Their code is still built either way; only switching them on is skipped.
    #[cfg(feature = "dev")]
    app.add_plugins((worldgen::WorldGenPlugin, soak::SoakPlugin, temperatures::TemperaturesPlugin));
    // Every schedule on one thread: the game's systems are small, and waking workers to share them
    // out cost more than it saved [chain 7, 2026-11-05].
    single_threaded(&mut app);
    runner::run(app);
    diagnostics::ended_normally();
}

/// Sets every schedule the app has to run its systems on one thread.
fn single_threaded(app: &mut App) {
    for (_, schedule) in app.world_mut().resource_mut::<Schedules>().iter_mut() {
        schedule.set_executor(crate::ecs::ecs::schedule::SingleThreadedExecutor::new());
    }
}

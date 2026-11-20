//! Incus Artus: the rebuild's first build (plan 2.1, the skeleton).
//!
//! A window that runs, its clocks, a flying camera and the instruments, with nothing in the world yet: the empty
//! frame, measured. Everything after this is built on it, one change at a time, each measured from its first build.
//! What's learnt in old Incus Artus (`old/`) is carried as results, not as code.
//!
//! The parts:
//! - `session`: the build's stamp, the project's folder, and the session's log.
//! - `card`: choosing the graphics card and setting it up to draw to the window.
//! - `clocks`: the frame's clock (how long each frame takes) and the card's (how long each of its passes takes).
//! - `camera`: the flying camera, in the game's axes.
//! - `gui`: the 95 drawing: the editor's and the game's windows drawn by us in Windows 95's look, as sheets of pixels
//!   the card shows enlarged a whole number of times.
//! - `panel`: the debug panel (F3), a floating window over the game.
//! - `presets`: a page of the panel's settings saved under a name and loaded back; "default" is what the game starts with.
//! - `editor`: the editor (F2), a window of its own, its pages reached by the drop-down in its top bar.
//! - `doorway`: the one piece that answers "what's here?" a tile at a time (the export behind it now).
//! - `far`: the far cells, summed from the doorway's columns (cubes of 32 and every level above), on the loading lane.
//! - `bricks`: the far cells on the card, as bricks.
//! - `atlas`: the far bricks as textures, blended by the card's texture hardware (change 1).
//! - `far_eye`: the far eye, old's far march on bricks (`far.wgsl`, `far_up.wgsl`).
//! - `marks`: the places every flight stands at, as data (`data/marks.toml`); the game starts at the first.
//! - `flight`: a measured flight (`INCUS_FLIGHT=1`) at every mark, written to a report in `logs/`.
//! - `wiring`: every lane, pass and store, declared in one place; the frame runs in its order, and the software map and
//!   `master/map.md` are drawn from it.
//! - `runner`: the window and the frame, which use all of the above.

mod atlas;
mod blue_noise;
mod bricks;
mod camera;
mod card;
mod clocks;
mod doorway;
mod editor;
mod far;
mod far_eye;
mod far_light;
mod far_ring;
mod flight;
mod gui;
mod marks;
mod memory;
mod panel;
mod presets;
mod runner;
mod near;
mod near_eye;
mod scatter_stage;
mod session;
mod temperatures;
mod textures;
mod wiring;

fn main() {
    session::start_log();
    runner::run();
}

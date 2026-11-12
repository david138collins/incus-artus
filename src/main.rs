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
//! - `panel`: the developer panel (F3), drawn over the picture.
//! - `flight`: a measured flight (`INCUS_FLIGHT=1`), written to a report in `logs/`.
//! - `runner`: the window and the frame, which use all of the above.

mod camera;
mod card;
mod clocks;
mod flight;
mod panel;
mod runner;
mod session;

fn main() {
    session::start_log();
    runner::run();
}

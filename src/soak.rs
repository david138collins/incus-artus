//! The soak test — does the game run worse the longer it runs? For automatic testing only.
//!
//! Start the game with `INCUS_SOAK=<seconds>` (and if wanted `INCUS_SOAK_SPEED=<blocks a second>`,
//! `INCUS_SOAK_DISTANCE=<render distance in chunks>`, `INCUS_SOAK_CAP=<frames a second, or uncapped>`
//! for the frame limit,
//! `INCUS_SOAK_WINDOW=<width>x<height>` for the window),
//! together with `INCUS_START_SCREEN=worldgen` and
//! `INCUS_SELFTEST_SUITE=compiler`. It then:
//! 1. renders the newest bake with the World Compiler's recipe as it starts (the defaults), and
//!    spawns in the middle (see `worldgen::run_suite`);
//! 2. flies a wide circle over the world at a steady speed, a little above the ground, so chunks
//!    and distant tiles are made and dropped all the way round, and the same ground comes round
//!    again each lap;
//! 3. writes a line to the log every 10 seconds: the frame rate, and how many chunks, meshes,
//!    pictures and so on the game is holding (`perf::soak_line`);
//! 4. quits when the time is up.
//!
//! Healthy: after the first lap the counts level off and the frame rate holds. A count that keeps
//! climbing lap after lap is a leak.

use crate::camera::FlyCamera;
use crate::player::ControlMode;
use crate::screen::Screen;
use crate::world::{SEA_LEVEL, Terrain};
use crate::ecs::prelude::*;

/// The circle's radius, and how fast it's flown (blocks a second): a lap takes about four minutes.
/// `INCUS_SOAK_SPEED` flies it faster (David flies at 2,000 [2026-10-28]); the circle then widens, so
/// a lap still takes about a minute or more.
const RADIUS: f32 = 6000.0;
const SPEED: f32 = 150.0;


fn speed() -> f32 {
    std::env::var("INCUS_SOAK_SPEED").ok().and_then(|v| v.parse().ok()).unwrap_or(SPEED)
}

fn radius() -> f32 {
    // `INCUS_SOAK_RADIUS`: another circle (out over the sea, say, for sight's checks).
    std::env::var("INCUS_SOAK_RADIUS").ok().and_then(|v| v.parse().ok()).unwrap_or(RADIUS.max(speed() * 9.5))
}
/// How high above the ground (or the sea).
const ABOVE: f32 = 120.0;
/// Seconds between log lines.
const EVERY: f32 = 10.0;

/// Whether the game was started for a soak test.
pub fn wanted() -> bool {
    std::env::var("INCUS_SOAK").is_ok()
}

/// Whether the game is measuring itself: a soak test or a test window.
pub fn measuring() -> bool {
    wanted() || test_window_seconds().is_some()
}

// ── The test window ──────────────────────────────────────────────────────────
//
// The frame loop's torque spec [the regroup, docs/regroup.md, Fork A]: after any change to the frame
// loop, the same runs at the screen, with their numbers written down. Started with
// `INCUS_TEST_WINDOW=<seconds>` (120 if no number), the game runs them itself, so nobody has to watch
// for a cue: the panel closed, `WARM_UP` seconds to settle, then that long uncapped, `SETTLE` seconds,
// then that long just under the screen's top rate (237 on a 240 screen), then it quits. With
// `INCUS_TEST_BALLAST`, the run under the top is repeated with each of `BALLAST_RUNS` of made-up work
// on the card (`runner/ballast.rs`; the warm-up, uncapped, calibrates it). It runs on
// whether it's in front or not (it never freezes). Each run's
// start and end go in the log with the time (seconds since 1970), so PresentMon's record, started
// beside it, can be cut into the runs. The settings' frame cap is left as it was.

const WARM_UP: f32 = 15.0;
const SETTLE: f32 = 5.0;

/// How long each run of the test window lasts, if the game was started for one.
pub fn test_window_seconds() -> Option<f32> {
    let value = std::env::var("INCUS_TEST_WINDOW").ok()?;
    Some(value.parse().unwrap_or(120.0))
}

#[derive(Resource)]
struct TestWindow {
    seconds: f32,
    /// The phase now (`phases`); `usize::MAX` before the first.
    phase: usize,
}

/// The ballast runs' sizes (ms at full clock), with `INCUS_TEST_BALLAST` [David 2026-11-05: "good
/// deal, install ballast"]: none, then up to build 60's whole scene standing (3.7 ms).
const BALLAST_RUNS: [f32; 4] = [0.0, 1.0, 2.0, 3.7];

/// One phase of the test window: the frame limit's name (`look.rs`), the run's name (none for the
/// gaps between runs), how long, and the ballast (ms at full clock).
struct Phase {
    limit: &'static str,
    name: Option<String>,
    seconds: f32,
    ballast: f32,
}

/// The test window's phases: settling, uncapped, then just under the screen's top rate; with
/// `INCUS_TEST_BALLAST`, that last once for each ballast.
fn phases(seconds: f32) -> Vec<Phase> {
    use crate::look::{UNCAPPED, UNDER_THE_TOP};
    let phase = |limit, name: Option<String>, seconds, ballast| Phase { limit, name, seconds, ballast };
    let mut phases = vec![phase(UNCAPPED, None, WARM_UP, 0.0), phase(UNCAPPED, Some("uncapped".into()), seconds, 0.0)];
    let ballasts: &[f32] = if std::env::var("INCUS_TEST_BALLAST").is_ok() { &BALLAST_RUNS } else { &[0.0] };
    for &ballast in ballasts {
        let name = if ballast > 0.0 { format!("just under the screen's top rate, with {ballast} ms of ballast") } else { "just under the screen's top rate".to_string() };
        phases.push(phase(UNDER_THE_TOP, None, SETTLE, ballast));
        phases.push(phase(UNDER_THE_TOP, Some(name), seconds, ballast));
    }
    phases
}

fn run_test_window(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed_secs();
    let mut test = world.resource_mut::<TestWindow>();
    let phases = phases(test.seconds);
    let mut end = 0.0;
    let phase = phases.iter().position(|phase| {
        end += phase.seconds;
        now < end
    });
    let phase = phase.unwrap_or(phases.len());
    let previous = std::mem::replace(&mut test.phase, phase);
    if phase != previous {
        let at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64());
        if let Some(Phase { name: Some(name), .. }) = phases.get(previous) {
            info!("Test window: the run {name} ended at {at:.3} (seconds since 1970)");
        }
        match phases.get(phase) {
            Some(Phase { name: Some(name), seconds, .. }) => info!("Test window: the run {name} began at {at:.3} (seconds since 1970), for {seconds:.0} s"),
            Some(_) if previous == usize::MAX => info!("Test window: {WARM_UP:.0} s to settle, then the runs"),
            Some(_) => {}
            None => {
                info!("Test window: done");
                world.write_message(AppExit::Success);
            }
        }
    }
    let refresh = world.resource::<crate::runner::WindowState>().refresh;
    world.resource_mut::<crate::runner::TestCap>().0 = phases.get(phase).map(|phase| crate::look::frame_limit(phase.limit, refresh));
    if let Some(phase) = phases.get(phase) {
        world.resource_mut::<crate::runner::BallastTarget>().0 = phase.ballast;
    }
    world.resource_mut::<crate::debug::DebugPanel>().visible = false;
}

pub struct SoakPlugin;

impl Plugin for SoakPlugin {
    fn build(&self, app: &mut App) {
        if let Some(seconds) = test_window_seconds() {
            app.insert_resource(TestWindow { seconds, phase: usize::MAX }).add_systems(Last, run_test_window);
        }
        let Some(seconds) = std::env::var("INCUS_SOAK").ok().and_then(|v| v.parse().ok()) else { return };
        // (Since the regroup the game draws flat out with or without the focus: the old switch for it,
        // `INCUS_SOAK_CONTINUOUS`, isn't needed. The old renderer's switches went with it.)
        app.insert_resource(Soak { seconds, started: None, next_line: 0.0, frames: 0, focused: 0 })
            .add_systems(Update, fly_round.before(crate::camera::place_camera).run_if(in_state(Screen::Play)))
            .add_systems(Last, report);
    }
}

#[derive(Resource)]
struct Soak {
    seconds: f32,
    /// When the flight began (real seconds since the game started), once there's a world.
    started: Option<f32>,
    next_line: f32,
    /// Frames since the last line, and how many of them the window had the focus (as Bevy sees it).
    frames: u32,
    focused: u32,
}

fn fly_round(
    mut soak: ResMut<Soak>,
    terrain: Res<Terrain>,
    time: Res<Time<Real>>,
    mut mode: ResMut<ControlMode>,
    mut cameras: Query<&mut FlyCamera>,
    mut kept: ResMut<crate::look::KeptSettings>,
    mut window: ResMut<crate::runner::WindowState>,
    mut resized: Local<bool>,
) {
    let Some(map) = terrain.compiled() else { return };
    let now = time.elapsed_secs();
    let started = *soak.started.get_or_insert_with(|| {
        info!("Soak: flying a circle {} blocks round at {} blocks a second", radius(), speed());
        now
    });
    *mode = ControlMode::Fly;
    // `INCUS_SOAK_DISTANCE`: the render distance (chunks) to fly at, instead of the preset's.
    if let Some(distance) = std::env::var("INCUS_SOAK_DISTANCE").ok().and_then(|v| v.parse::<i32>().ok())
        && kept.render_distance_chunks() != distance
    {
        kept.render.set_number("render_distance_chunks", distance as f32);
    }
    // `INCUS_SOAK_CAP=<frames a second, or "uncapped">`: the frame limit.
    if let Ok(name) = std::env::var("INCUS_SOAK_CAP")
        && kept.frame_limit_name() != name
    {
        kept.set_frame_limit_name(name);
    }
    // `INCUS_SOAK_WINDOW=<width>x<height>`: the window's size (does the card's work follow the pixels?).
    if !*resized
        && let Some((width, height)) = std::env::var("INCUS_SOAK_WINDOW").ok().and_then(|v| v.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))))
    {
        window.resize_to = Some((width, height));
        *resized = true;
    }
    // `INCUS_SOAK_RESIZE_EVERY=<seconds>`: the window made bigger and smaller in turn (the screen's picture set
    // up again while the card is busy, as F11 does).
    if let Some(every) = std::env::var("INCUS_SOAK_RESIZE_EVERY").ok().and_then(|v| v.parse::<f32>().ok()) {
        let (before, after) = (((now - started - time.delta_secs()) / every).floor(), ((now - started) / every).floor());
        if after > before {
            window.resize_to = Some(if after as i32 % 2 == 0 { (1280, 720) } else { (1600, 900) });
            info!("Soak: the window resized ({})", after);
        }
    }
    // Round the circle (north of the middle to start with), facing the way it's going.
    let (radius, speed) = (radius(), speed());
    let angle = (now - started) * speed / radius;
    let (north, east) = (radius * angle.cos(), radius * angle.sin());
    let ground = map.height(north, east).unwrap_or(0.0).max(SEA_LEVEL as f32);
    for mut camera in &mut cameras {
        // Eased up and down, so a cliff doesn't jerk the view.
        // `INCUS_SOAK_ABOVE` and `INCUS_SOAK_PITCH`: higher, and looking further down (for sight's checks).
        let above = std::env::var("INCUS_SOAK_ABOVE").ok().and_then(|v| v.parse().ok()).unwrap_or(ABOVE);
        let target = ground + above;
        let z = camera.position.z + (target - camera.position.z) * (2.0 * time.delta_secs()).min(1.0);
        camera.position = Vec3::new(north, east, if (camera.position.z - target).abs() > 500.0 { target } else { z });
        camera.heading = (angle.to_degrees() + 90.0).rem_euclid(360.0);
        camera.pitch = std::env::var("INCUS_SOAK_PITCH").ok().and_then(|v| v.parse().ok()).unwrap_or(-12.0);
    }
}

fn report(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed_secs();
    let soak = world.resource::<Soak>();
    let Some(started) = soak.started else { return };
    let (flown, seconds, next_line) = (now - started, soak.seconds, soak.next_line);
    let focused = world.resource::<crate::runner::WindowState>().focused;
    let mut soak = world.resource_mut::<Soak>();
    soak.frames += 1;
    soak.focused += focused as u32;
    if flown >= next_line {
        let share = 100 * soak.focused / soak.frames.max(1);
        (soak.frames, soak.focused) = (0, 0);
        let line = crate::perf::soak_line(world);
        info!("Soak {flown:>5.0} s: {line} | focused {share}%");
        world.resource_mut::<Soak>().next_line = next_line + EVERY;
    }
    if flown >= seconds {
        info!("Soak: done after {flown:.0} s");
        world.write_message(AppExit::Success);
    }
}

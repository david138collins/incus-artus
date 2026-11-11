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

/// Whether the game was started for a soak test, or the dyno's flight (`INCUS_DYNO`: the World Compiler renders for
/// it as for the soak, and the game never freezes away from the front).
pub fn wanted() -> bool {
    std::env::var("INCUS_SOAK").is_ok() || std::env::var("INCUS_DYNO").is_ok()
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
        // The dyno's flight (`dyno.rs`), instead of the soak's circle.
        if std::env::var("INCUS_DYNO").is_ok() {
            let laps = std::env::var("INCUS_DYNO_LAPS").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
            app.insert_resource(Dyno { laps, baking_run: dyno_baking_run(), ..Dyno::default() })
                .add_systems(Update, (fly_dyno, fly_marks).chain().before(crate::camera::place_camera).run_if(in_state(Screen::Play)))
                .add_systems(Last, record_dyno);
            return;
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

// ── The dyno's flight (`docs/dyno.md`, the instruments; `dyno.rs` has the route and the sums) ──
//
// `INCUS_DYNO=baked` (or `baking`), with `INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno` for David's
// settings kept still (copies of his defaults, 2026-11-09). Uncapped, the panel shut; the report goes in
// `logs/dyno-<time>.txt` and the game quits. The world: the test world, `INCUS_LOAD_WORLD="nice25 test"` (David's
// nice25 rendered and saved with its whole bake: `docs/dyno.md`); or nice25 rendered afresh, with
// `INCUS_START_SCREEN=worldgen INCUS_SELFTEST_SUITE=compiler` and `compiler/dyno,worldgen/dyno` added to the presets.
// - **baked:** the laps wait for the whole bake (read back with the test world, it's there in moments; rendered
//   afresh, about a quarter of an hour, the route flown over and over meanwhile, measured as baking). Then the route
//   from the start, `INCUS_DYNO_LAPS` times (1 if not said), measured as baked and leg by leg. With
//   `INCUS_DYNO_SAVE_AS=<name>`, the world is saved under that name once its bake is whole (how the test world was
//   made, and how it's made again after the cells' code changes, `SIGHT_RULES`, and its bake no longer reads back).
// - **baking:** the route flown `INCUS_DYNO_LAPS` times from the start while the bake runs (David's "creaking when
//   you move too quick"), measured as baking: on nice25 rendered afresh.
// - **The standing marks** (`dyno::marks`, the Oracle's experiment 1), after the baked laps: the camera held still at
//   each, waited on till the live levels round it are whole (at most `MARK_SETTLE_MOST`), the build replayed where
//   there is one, then measured `MARK_MEASURE` seconds. `INCUS_DYNO_LAPS=0` flies the marks alone (for the small card);
//   `INCUS_DYNO_MARKS=0` leaves them out; `INCUS_DYNO_BUILD=<event log>` replays another build.
// - **The header** says exactly what flew: the build and whether its code was committed (the difference saved beside
//   the report, `dyno-<time>.patch`), every `INCUS_*` and `WGPU_*` switch, the card and how it was chosen, the window
//   and how the picture was handed to the screen as granted, the presets, what ran beside it.

/// The longest a mark is waited on for its live levels to be whole (seconds), and how long it's measured.
const MARK_SETTLE_MOST: f32 = 180.0;
const MARK_MEASURE: f32 = 20.0;
/// How long the build's placings are given to be applied, once sent (seconds).
const BUILD_TAKES: f32 = 3.0;
/// The build replayed at its mark, if none is named: David's quarter-block build of 2026-11-10 (build 108).
const THE_BUILD: &str = "kept logs/events-1794341728.log";

/// Seconds between the heat watch's readings.
const HEAT_EVERY: f32 = 5.0;
/// The longest the bake is waited for (seconds): past it, the baked laps start anyway (and the report says so).
const BAKING_MOST: f32 = 1800.0;
/// The heat a flight stops at (the CPU area, °C: agreed with David for measuring runs, 2026-11-05), read twice in a
/// row (the CPU area's reading jumps 20 degrees between readings: one spike isn't the laptop's heat).
const TOO_HOT: f32 = 105.0;

/// Whether the dyno's flight is the baking one (`INCUS_DYNO=baking`).
fn dyno_baking_run() -> bool {
    std::env::var("INCUS_DYNO").is_ok_and(|v| v == "baking")
}



#[derive(Resource, Default)]
struct Dyno {
    laps: u32,
    /// The baking flight (no bake read back, every lap measured as baking).
    baking_run: bool,
    /// The world saved (`INCUS_DYNO_SAVE_AS`), once its bake was whole.
    saved: bool,
    /// Stopped for the heat (`TOO_HOT`), at this CPU area's temperature; and whether the last reading was over it.
    too_hot: Option<f32>,
    hot_once: bool,
    /// What else was running as the flight began (`background_programs`).
    background: String,
    baking_laps: u32,
    legs: Vec<crate::dyno::Leg>,
    /// When the flight began, and the lap now.
    started: Option<f32>,
    lap_started: f32,
    /// When the bake was done (the measured laps began), and whether it really was (not just waited for too long).
    baked_at: Option<f32>,
    bake_finished: bool,
    baked_laps: u32,
    /// The leg flown now.
    leg: Option<usize>,
    /// Skip measuring this frame (the lap just began again: the camera jumped).
    jumped: bool,
    /// Baked laps still to fly unmeasured first (one: the near levels never finish while the camera moves, some
    /// 8,800 tiles round it, so a lap measured from the start began however full they were; after one lap, each
    /// measured lap begins as the last ended).
    warming: u32,
    baking: crate::dyno::Samples,
    baked: crate::dyno::Samples,
    by_leg: Vec<(String, crate::dyno::Samples)>,
    /// The twin's readings (`INCUS_SIGHT_TWIN`: the frames it drew, baked), whole and by leg; the march's are the
    /// rest.
    baked_twin: crate::dyno::Samples,
    by_leg_twin: Vec<(String, crate::dyno::Samples)>,
    heat: Vec<String>,
    next_heat: f32,
    done: bool,
    /// The route's laps are done: the marks now (`fly_marks`), made the first time; the one flown and how far it's
    /// got, since when; and what each measured.
    marks_due: bool,
    marks: Option<Vec<crate::dyno::Mark>>,
    mark: Option<(usize, MarkState)>,
    mark_since: f32,
    by_mark: Vec<MarkRecord>,
    /// The fog as it was before a clear day's mark turned it off.
    fog_was: Option<bool>,
    /// The lanes' work read since sight's last reading (ms), for the next reading.
    lanes_pending: f32,
}

/// How far a mark has got.
#[derive(Clone, Copy, Debug, PartialEq)]
enum MarkState {
    /// The camera there, the live levels filling round it.
    Arriving,
    /// The build's placings sent (or about to be), being applied.
    Building { sent: bool },
    /// The build in; the levels and the bake made again round it.
    Rebuilding,
    /// Measured, standing still.
    Standing,
}

/// What a mark measured: where it stood, how it settled, the build's placings, and its frames while settling and
/// standing.
#[derive(Default)]
struct MarkRecord {
    name: String,
    place: String,
    settled: Vec<String>,
    build: Option<(usize, usize, usize)>,
    settling: crate::dyno::Samples,
    standing: crate::dyno::Samples,
}

#[allow(clippy::too_many_arguments)]
fn fly_dyno(
    mut dyno: ResMut<Dyno>,
    terrain: Res<Terrain>,
    caves: Res<crate::world::Caves>,
    sight: Res<crate::world::SightStore>,
    time: Res<Time<Real>>,
    mut mode: ResMut<ControlMode>,
    mut cameras: Query<&mut FlyCamera>,
    mut kept: ResMut<crate::look::KeptSettings>,
    mut current: ResMut<crate::save::CurrentWorld>,
    mut requests: MessageWriter<crate::save::WorldFileRequest>,
) {
    let Some(map) = terrain.compiled() else { return };
    if dyno.done || dyno.marks_due {
        return;
    }
    let now = time.elapsed_secs();
    // A saved world asked for (`INCUS_LOAD_WORLD`): the route waits till it's loaded (till then the world is the one
    // the game starts with, without the saved one's caves).
    if dyno.legs.is_empty() && std::env::var("INCUS_LOAD_WORLD").is_ok() && current.saved.is_none() {
        return;
    }
    // And till the world's on the card and drawn (its bake begun): before it, the frames draw next to nothing, at
    // some 2,000 a second [2026-11-09, the first baking flight: 20 s of them were most of its frames].
    if dyno.legs.is_empty() && sight.bake.is_none() {
        return;
    }
    if dyno.legs.is_empty() {
        // The cave mouths: each tunnel's ends that come up to the ground (its points are in 16ths of a block).
        let ground = |north: f32, east: f32| map.height(north, east);
        let mut mouths = Vec::new();
        for tunnel in &caves.0.tunnels {
            for point in [tunnel.points.first(), tunnel.points.last()].into_iter().flatten() {
                let [north, east, z] = point.at.map(|v| v as f32 / 16.0);
                if ground(north, east).is_some_and(|top| z >= top - 8.0) {
                    mouths.push([north, east, z]);
                }
            }
        }
        dyno.legs = crate::dyno::route(&ground, SEA_LEVEL as f32, &mouths);
        let names: Vec<&str> = dyno.legs.iter().map(|leg| leg.name).collect();
        info!("Dyno: the route, {:.0} s a lap: {}", crate::dyno::duration(&dyno.legs), names.join(", "));
        dyno.started = Some(now);
        dyno.lap_started = now;
        dyno.next_heat = now;
        dyno.background = background_programs();
        info!("Dyno: running beside it: {}", dyno.background);
    }
    *mode = ControlMode::Fly;
    if kept.frame_limit_name() != crate::look::UNCAPPED {
        kept.set_frame_limit_name(crate::look::UNCAPPED.to_string());
    }
    // Baked yet? Then the measured laps begin, from the start of the route (and the world's saved, if asked).
    let started = dyno.started.unwrap_or(now);
    let finished = sight.bake.as_ref().is_some_and(|baking| baking.bake.lock().is_ok_and(|bake| bake.all_done.is_some()));
    if finished && !dyno.saved {
        // Saved as asked (`INCUS_DYNO_SAVE_AS`); or the world loaded (`INCUS_LOAD_WORLD`) had to be baked again (its
        // bake didn't read back: the cells' code, or the world, changed), saved again so the next flight needn't.
        let rebaked = std::env::var("INCUS_LOAD_WORLD").is_ok() && now - started > 60.0;
        if let Ok(name) = std::env::var("INCUS_DYNO_SAVE_AS") {
            info!("Dyno: the bake is whole; the world saved as worlds/{name}");
            current.name = name;
            requests.write(crate::save::WorldFileRequest::Save);
        } else if rebaked {
            info!("Dyno: the world's bake was made again ({:.0} s); saved again with it", now - started);
            requests.write(crate::save::WorldFileRequest::Save);
        }
        dyno.saved = true;
    }
    if dyno.baked_at.is_none() && !dyno.baking_run {
        if finished || now - started > BAKING_MOST {
            dyno.baked_at = Some(now);
            dyno.bake_finished = finished;
            // `INCUS_DYNO_LAPS=0`: no laps, the marks at once.
            if dyno.laps == 0 {
                info!("Dyno: {} after {:.0} s; no laps, the standing marks", if finished { "baked" } else { "still baking, waited long enough" }, now - started);
                end_of_route(&mut dyno);
                return;
            }
            info!("Dyno: {} after {:.0} s; a lap to warm up, then the measured laps", if finished { "baked" } else { "still baking, waited long enough" }, now - started);
            dyno.lap_started = now;
            dyno.jumped = true;
            dyno.warming = 1;
        }
    }
    let flown = now - dyno.lap_started;
    let Some((across, k, _)) = crate::dyno::along(&dyno.legs, flown) else {
        // The lap's done: another, or the end.
        if dyno.warming > 0 {
            dyno.warming -= 1;
            info!("Dyno: warmed up; the measured laps begin");
        } else if dyno.baked_at.is_some() {
            dyno.baked_laps += 1;
            if dyno.baked_laps >= dyno.laps.max(1) {
                end_of_route(&mut dyno);
                return;
            }
        } else if dyno.baking_run {
            dyno.baking_laps += 1;
            if dyno.baking_laps >= dyno.laps.max(1) {
                dyno.done = true;
                return;
            }
        }
        dyno.lap_started = now;
        dyno.jumped = true;
        return;
    };
    dyno.leg = Some(k);
    let leg = dyno.legs[k].clone();
    let from = if k == 0 { [0.0, 0.0] } else { dyno.legs[k - 1].to };
    let heading = (leg.to[1] - from[1]).atan2(leg.to[0] - from[0]).to_degrees().rem_euclid(360.0);
    let target = match leg.height {
        crate::dyno::Height::Above(above) => map.height(across[0], across[1]).unwrap_or(0.0).max(SEA_LEVEL as f32) + above,
        crate::dyno::Height::At(z) => z,
    };
    let ease = (2.0 * time.delta_secs()).min(1.0);
    let jumped = dyno.jumped;
    for mut camera in &mut cameras {
        let z = camera.position.z + (target - camera.position.z) * ease;
        camera.position = Vec3::new(across[0], across[1], if jumped || (camera.position.z - target).abs() > 500.0 { target } else { z });
        // Turned the short way round, eased.
        let turn = (heading - camera.heading + 540.0).rem_euclid(360.0) - 180.0;
        camera.heading = if jumped { heading } else { (camera.heading + turn * ease).rem_euclid(360.0) };
        camera.pitch = if jumped { leg.pitch } else { camera.pitch + (leg.pitch - camera.pitch) * ease };
    }
}

/// The route's laps done: the standing marks next, unless they're left out (`INCUS_DYNO_MARKS=0`).
fn end_of_route(dyno: &mut Dyno) {
    if std::env::var("INCUS_DYNO_MARKS").is_ok_and(|v| v == "0") {
        dyno.done = true;
    } else {
        dyno.marks_due = true;
    }
}

/// The build replayed at its mark: the placings in `INCUS_DYNO_BUILD`'s event log, or `THE_BUILD`'s (none if it can't
/// be read, said in the log).
fn the_build() -> Vec<crate::dyno::Placing> {
    let path = std::env::var("INCUS_DYNO_BUILD").map(std::path::PathBuf::from).unwrap_or_else(|_| crate::debug::project_folder().join(THE_BUILD));
    match std::fs::read_to_string(&path) {
        Ok(log) => crate::dyno::placings(&log),
        Err(error) => {
            warn!("Dyno: the build to replay ({}) couldn't be read ({error}); no build mark", path.display());
            Vec::new()
        }
    }
}

/// The highest solid block in a column (pieces count), once its chunk is here.
fn top_solid(voxel: &crate::world::VoxelWorld, registry: &crate::data::BlockRegistry, column: IVec2) -> Option<i32> {
    use crate::world::{BlockId, WORLD_BOTTOM, WORLD_HEIGHT};
    if !voxel.is_loaded(IVec3::new(column.x, column.y, WORLD_BOTTOM)) {
        return None;
    }
    (WORLD_BOTTOM..WORLD_HEIGHT).rev().find(|&z| {
        let block = voxel.get_block(IVec3::new(column.x, column.y, z));
        block == BlockId::PIECES || registry.is_solid(block)
    })
}

/// The standing marks, one after another (`dyno::marks`): the camera held at each with a player's eye over the ground,
/// waited on till the live levels round it are whole, the build replayed where there is one, then measured.
#[allow(clippy::too_many_arguments)]
fn fly_marks(
    mut dyno: ResMut<Dyno>,
    terrain: Res<Terrain>,
    sight: Res<crate::world::SightStore>,
    (voxel, registry, player): (Res<crate::world::VoxelWorld>, Res<crate::data::BlockRegistry>, Res<crate::player::PlayerSettings>),
    time: Res<Time<Real>>,
    mut mode: ResMut<ControlMode>,
    mut cameras: Query<&mut FlyCamera>,
    mut kept: ResMut<crate::look::KeptSettings>,
    mut requests: MessageWriter<crate::world::BlockEditRequest>,
    (mut changed, mut refused): (MessageReader<crate::world::BlockChanged>, MessageReader<crate::world::BlockEditRefused>),
) {
    let (changes, refusals) = (changed.read().count(), refused.read().count());
    if !dyno.marks_due || dyno.done {
        return;
    }
    let Some(map) = terrain.compiled() else { return };
    let now = time.elapsed_secs();
    if dyno.marks.is_none() {
        let ground = |north: f32, east: f32| map.height(north, east);
        let marks = crate::dyno::marks(&ground, voxel.spawn().to_array(), the_build());
        let names: Vec<&str> = marks.iter().map(|mark| mark.name).collect();
        info!("Dyno: the standing marks: {}", names.join(", "));
        dyno.by_mark = marks.iter().map(|mark| MarkRecord { name: mark.name.to_string(), ..MarkRecord::default() }).collect();
        dyno.marks = Some(marks);
        dyno.mark = Some((0, MarkState::Arriving));
        dyno.mark_since = now;
        dyno.jumped = true;
    }
    let Some((k, state)) = dyno.mark else { return };
    let Some(mark) = dyno.marks.as_ref().and_then(|marks| marks.get(k)).cloned() else { return };
    *mode = ControlMode::Fly;
    if kept.frame_limit_name() != crate::look::UNCAPPED {
        kept.set_frame_limit_name(crate::look::UNCAPPED.to_string());
    }
    // The clear day: fog off while it stands (put back after).
    if mark.clear && dyno.fog_was.is_none() {
        dyno.fog_was = Some(kept.sight_fog());
        kept.render.set_flag("sight_fog", false);
    }
    // A player's eye over the column's top block once its chunk is here (over the map's ground till then).
    let column = IVec2::new(mark.at[0].floor() as i32, mark.at[1].floor() as i32);
    let top = top_solid(&voxel, &registry, column);
    let feet = top.map_or_else(|| map.height(mark.at[0], mark.at[1]).unwrap_or(SEA_LEVEL as f32).max(SEA_LEVEL as f32) + 2.0, |z| z as f32 + 1.0);
    let eye = feet + player.eye_height;
    let pitch = mark.look_at.map_or(mark.pitch, |[north, east, z]| (z - eye).atan2((north - mark.at[0]).hypot(east - mark.at[1])).to_degrees());
    for mut camera in &mut cameras {
        camera.position = Vec3::new(mark.at[0], mark.at[1], eye);
        camera.heading = mark.heading;
        camera.pitch = pitch;
    }
    let (have, want, filling) = sight.live.as_ref().map_or((0, 1, 0), |live| live.fullness((mark.at[0], mark.at[1])));
    let whole = have == want && filling == 0 && top.is_some();
    let waited = now - dyno.mark_since;
    let how = |waited: f32| if whole { format!("whole in {waited:.0} s") } else { format!("NOT whole after {waited:.0} s: {have} of {want} tiles, {filling} being made{}", if top.is_none() { ", the ground's chunk not here" } else { "" }) };
    let next = match state {
        MarkState::Arriving if whole || waited > MARK_SETTLE_MOST => {
            let record = &mut dyno.by_mark[k];
            record.place = format!("north {:.1}, east {:.1}, the eye at {eye:.1} ({:.2} over the ground), facing {:.0}°, pitch {pitch:.1}°{}", mark.at[0], mark.at[1], player.eye_height, mark.heading, if mark.clear { ", fog off" } else { "" });
            record.settled.push(format!("the live levels {}", how(waited)));
            Some(if mark.build.is_empty() { MarkState::Standing } else { MarkState::Building { sent: false } })
        }
        MarkState::Building { sent: false } => {
            for placing in &mark.build {
                let block = |name: &str| registry.id(name);
                let (position, kind) = match placing {
                    crate::dyno::Placing::Block { cell, block: name } => (IVec3::from_array(*cell), block(name).map(crate::world::EditKind::Place)),
                    crate::dyno::Placing::Bar { cell, at, axis, block: name } => (IVec3::from_array(*cell), block(name).map(|block| crate::world::EditKind::PlaceBar { at: IVec3::from_array(*at), axis: *axis, block })),
                };
                match kind {
                    Some(kind) => {
                        requests.write(crate::world::BlockEditRequest { position, kind, source: crate::world::EditSource::Player(crate::events::PlayerId(0)) });
                    }
                    None => warn!("Dyno: the build names a block that isn't registered: {placing:?}"),
                }
            }
            info!("Dyno: the build's {} placings sent", mark.build.len());
            dyno.by_mark[k].build = Some((mark.build.len(), 0, 0));
            dyno.mark_since = now;
            dyno.mark = Some((k, MarkState::Building { sent: true }));
            None
        }
        MarkState::Building { sent: true } => {
            if let Some((_, cells, refused)) = &mut dyno.by_mark[k].build {
                (*cells, *refused) = (*cells + changes, *refused + refusals);
            }
            (waited > BUILD_TAKES).then_some(MarkState::Rebuilding)
        }
        MarkState::Rebuilding => {
            if let Some((_, cells, refused)) = &mut dyno.by_mark[k].build {
                (*cells, *refused) = (*cells + changes, *refused + refusals);
            }
            let rebuilt = whole && !sight.edits_waiting();
            (rebuilt || waited > MARK_SETTLE_MOST).then(|| {
                let bake = if sight.edits_waiting() { "; the bake NOT yet patched" } else { "; the bake patched" };
                dyno.by_mark[k].settled.push(format!("after the build, the live levels {}{bake}", how(waited)));
                MarkState::Standing
            })
        }
        MarkState::Standing if waited >= MARK_MEASURE => {
            // The next mark, or the end; the fog put back after a clear day.
            if mark.clear
                && let Some(fog) = dyno.fog_was.take()
            {
                kept.render.set_flag("sight_fog", fog);
            }
            if k + 1 < dyno.marks.as_ref().map_or(0, Vec::len) {
                dyno.mark = Some((k + 1, MarkState::Arriving));
                dyno.mark_since = now;
                dyno.jumped = true;
            } else {
                dyno.done = true;
            }
            None
        }
        _ => None,
    };
    if let Some(state) = next {
        if state == MarkState::Standing {
            info!("Dyno: {}: standing, measured for {MARK_MEASURE:.0} s", mark.name);
        }
        dyno.mark = Some((k, state));
        dyno.mark_since = now;
    }
}

fn record_dyno(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed_secs();
    let ms = world.resource::<Time<Real>>().delta_secs() * 1000.0;
    world.resource_mut::<crate::debug::DebugPanel>().visible = false;
    let readings = crate::sight::draw::take_card_readings();
    // The lanes' work on the card read this frame (taken every frame, so none is left over from frames not measured).
    let (lane_kinds, lane_pieces) = crate::world::card_recipe::take_lane_times();
    let heat = (world.resource::<Dyno>().started.is_some() && now >= world.resource::<Dyno>().next_heat).then(|| crate::temperatures::latest(world));
    let window = world.resource::<crate::runner::WindowState>();
    let screen = Shown { size: window.size, refresh: window.refresh, card: window.card.clone(), present: window.present.clone() };
    let mut dyno = world.resource_mut::<Dyno>();
    if dyno.started.is_none() {
        return;
    }
    let baked = dyno.baked_at.is_some();
    if dyno.warming > 0 {
        // The lap to warm up: not measured.
        dyno.lanes_pending = 0.0;
        return;
    }
    let leg_name = dyno.leg.and_then(|k| dyno.legs.get(k)).map(|leg| leg.name.to_string());
    // The lanes' work since sight's last reading goes beside the next.
    let lanes = if std::mem::take(&mut dyno.jumped) {
        dyno.lanes_pending = 0.0;
        None
    } else {
        dyno.lanes_pending += lane_kinds.iter().sum::<f32>();
        Some(if readings.iter().any(|reading| !reading.twin) { std::mem::take(&mut dyno.lanes_pending) } else { 0.0 })
    };
    let take = |samples: &mut crate::dyno::Samples, lanes: f32| {
        samples.frame(ms);
        let mut lanes = Some(lanes);
        for reading in readings.iter().filter(|reading| !reading.twin) {
            samples.card(reading.sight, &reading.passes);
            samples.with_lanes(reading.sight, lanes.take().unwrap_or(0.0));
        }
        samples.lane_work(lane_kinds, lane_pieces);
    };
    if let Some((k, state)) = dyno.mark {
        // At a mark: settling, or standing.
        if let (Some(lanes), Some(record)) = (lanes, dyno.by_mark.get_mut(k)) {
            take(if state == MarkState::Standing { &mut record.standing } else { &mut record.settling }, lanes);
        }
    } else if let Some(lanes) = lanes {
        let dyno = &mut *dyno;
        let phase = if baked { &mut dyno.baked } else { &mut dyno.baking };
        take(phase, lanes);
        // The twin's frames (`INCUS_SIGHT_TWIN`) kept apart, baked only.
        if baked {
            for reading in readings.iter().filter(|reading| reading.twin) {
                dyno.baked_twin.card(reading.sight, &reading.passes);
            }
        }
        if (baked || dyno.baking_run) && let Some(name) = leg_name {
            let place = |legs: &mut Vec<(String, crate::dyno::Samples)>| match legs.iter().position(|(have, _)| *have == name) {
                Some(at) => at,
                None => {
                    legs.push((name.clone(), crate::dyno::Samples::default()));
                    legs.len() - 1
                }
            };
            let at = place(&mut dyno.by_leg);
            take(&mut dyno.by_leg[at].1, lanes);
            if baked && readings.iter().any(|reading| reading.twin) {
                let at = place(&mut dyno.by_leg_twin);
                for reading in readings.iter().filter(|reading| reading.twin) {
                    dyno.by_leg_twin[at].1.card(reading.sight, &reading.passes);
                }
            }
        }
    }
    if let Some(heat) = heat {
        let at = now - dyno.started.unwrap_or(now);
        let over = heat.cpu_area.filter(|&celsius| celsius >= TOO_HOT);
        if let Some(celsius) = over
            && dyno.hot_once
        {
            warn!("Dyno: the CPU area at {celsius:.0} °C twice running; the flight stops (the cutoff is {TOO_HOT:.0})");
            dyno.too_hot = Some(celsius);
            dyno.done = true;
        }
        dyno.hot_once = over.is_some();
        let show = |value: Option<f32>, unit: &str| value.map_or("-".to_string(), |v| format!("{v:.0}{unit}"));
        let line = format!("{at:>4.0} s ({}): GPU {}, clock {}, CPU area {}", if baked { "baked" } else { "baking" }, show(heat.gpu, " °C"), show(heat.gpu_clock, " MHz"), show(heat.cpu_area, " °C"));
        info!("Dyno heat: {line}");
        dyno.heat.push(line);
        dyno.next_heat = now + HEAT_EVERY;
    }
    if dyno.done {
        let folder = crate::debug::project_folder().join("logs");
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let path = folder.join(format!("dyno-{stamp}.txt"));
        let _ = std::fs::create_dir_all(&folder);
        // The code's difference from its commit, beside the report, so what flew can be had again.
        let patch = (!crate::diagnostics::BUILD_CHANGED.is_empty()).then(|| {
            let patch = folder.join(format!("dyno-{stamp}.patch"));
            match std::fs::write(&patch, crate::diagnostics::BUILD_PATCH) {
                Ok(()) => patch.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
                Err(error) => format!("NOT saved ({error})"),
            }
        });
        let report = dyno_report(&dyno, screen, patch);
        match std::fs::write(&path, &report) {
            Ok(()) => info!("Dyno: the report is in {}", path.display()),
            Err(error) => warn!("Dyno: the report couldn't be written ({error})"),
        }
        for line in report.lines() {
            info!("Dyno: {line}");
        }
        world.write_message(AppExit::Success);
    }
}

/// The window and the card a flight was drawn on, for its header.
struct Shown {
    size: (u32, u32),
    refresh: f32,
    card: String,
    present: String,
}

/// Every `INCUS_*` and `WGPU_*` switch set for the game, sorted, as `NAME=value` (not the build's own stamp, which
/// `cargo run` hands the game too: the header gives it already).
fn switches() -> String {
    let mut set: Vec<String> = std::env::vars()
        .filter(|(name, _)| (name.starts_with("INCUS_") && !name.starts_with("INCUS_BUILD")) || name.starts_with("WGPU_"))
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    set.sort();
    if set.is_empty() { "none".into() } else { set.join(", ") }
}

fn dyno_report(dyno: &Dyno, shown: Shown, patch: Option<String>) -> String {
    use crate::diagnostics::{BUILD, BUILD_CHANGED, BUILD_NUMBER};
    let code = match &patch {
        None => "committed (the code as its commit has it)".to_string(),
        Some(patch) => format!("NOT committed: {BUILD_CHANGED}; the difference beside this report: {patch}"),
    };
    let lane_clock = if std::env::var("INCUS_LANE_CLOCK").is_ok_and(|v| v == "off") { "off (INCUS_LANE_CLOCK=off): the lanes' work isn't counted" } else { "on (where the card can time itself)" };
    let mut text = format!(
        "The dyno's flight: build {BUILD_NUMBER} ({BUILD}); the code {code}\nThe card: {}\nWindow {} x {}, the screen's refresh {:.0}, uncapped; the picture handed to the screen: {} (as granted); presets: {}\nSwitches: {}\nThe lanes' clock: {lane_clock}\n",
        if shown.card.is_empty() { "unknown" } else { &shown.card },
        shown.size.0,
        shown.size.1,
        shown.refresh,
        if shown.present.is_empty() { "unknown" } else { &shown.present },
        std::env::var("INCUS_START_PRESET").unwrap_or_else(|_| "the defaults".into()),
        switches()
    );
    let names: Vec<&str> = dyno.legs.iter().map(|leg| leg.name).collect();
    text += &format!("The route, {:.0} s a lap: {}\n", crate::dyno::duration(&dyno.legs), names.join(", "));
    text += &format!("Running beside it: {}\n", dyno.background);
    if let Some(celsius) = dyno.too_hot {
        text += &format!("STOPPED for the heat: the CPU area reached {celsius:.0} °C (the cutoff is {TOO_HOT:.0}); what's below is as far as it got\n");
    }
    let (started, baked_at) = (dyno.started.unwrap_or(0.0), dyno.baked_at.unwrap_or(0.0));
    if dyno.baking_run {
        text += &format!("The baking flight: a fresh bake, {} laps flown while it baked\n\n", dyno.baking_laps);
    } else {
        text += &format!("The bake: {} {:.0} s in; baked laps {}\n\n", if dyno.bake_finished { "done" } else { "NOT done, waited" }, baked_at - started, dyno.baked_laps);
    }
    if dyno.baking_run {
        text += &format!("Baking: {}\n\nBaking, by leg:\n", crate::dyno::line(&dyno.baking));
    } else {
        text += &format!("Loading (the bake read back, or baked whole): {}\n", crate::dyno::line(&dyno.baking));
        text += &format!("Baked: {}\n\nBaked, by leg:\n", crate::dyno::line(&dyno.baked));
    }
    for (name, samples) in &dyno.by_leg {
        text += &format!("  {name}: {}\n", crate::dyno::line(samples));
    }
    if !dyno.baked_twin.sight.is_empty() {
        text += &format!(
            "\nThe twin ({}), drawing every other timed frame in place of the march: sight on the card (ms), the march's frames against the twin's (the frames' times above are both's, and the card's above the march's alone):\n",
            std::env::var("INCUS_SIGHT_TWIN").unwrap_or_default()
        );
        text += &format!("  baked, whole: {}\n", crate::dyno::twin_line(&dyno.baked, &dyno.baked_twin));
        for (name, samples) in &dyno.by_leg {
            if let Some((_, twin)) = dyno.by_leg_twin.iter().find(|(have, _)| have == name) {
                text += &format!("  {name}: {}\n", crate::dyno::twin_line(samples, twin));
            }
        }
    }
    if !dyno.by_mark.is_empty() {
        text += &format!(
            "\nThe standing marks (uncapped, the camera still; each waited on till the live levels round it were whole, at most {MARK_SETTLE_MOST:.0} s, then measured {MARK_MEASURE:.0} s):\n"
        );
        for record in &dyno.by_mark {
            text += &format!("  {}: {}\n", record.name, if record.place.is_empty() { "not reached" } else { &record.place });
            for settled in &record.settled {
                text += &format!("    {settled}\n");
            }
            if let Some((asked, cells, refused)) = record.build {
                text += &format!("    the build: {asked} placings replayed, {cells} cells changed, {refused} refused (David's session: 1 refused)\n");
            }
            text += &format!("    standing: {}\n", crate::dyno::line(&record.standing));
            text += &format!("    settling: {}\n", crate::dyno::line(&record.settling));
        }
    }
    text += "\nHeat:\n";
    for line in &dyno.heat {
        text += &format!("  {line}\n");
    }
    text
}


/// The programs running beside a flight that could take the processor or the card (browsers, Aseprite, and the
/// like), by name, from Windows' task list; "none of note" if none, "unknown" where there's no task list.
fn background_programs() -> String {
    const NOTABLE: [(&str, &str); 12] = [
        ("chrome.exe", "Chrome"),
        ("msedge.exe", "Edge"),
        ("firefox.exe", "Firefox"),
        ("brave.exe", "Brave"),
        ("opera.exe", "Opera"),
        ("aseprite.exe", "Aseprite"),
        ("blender.exe", "Blender"),
        ("blockbench.exe", "Blockbench"),
        ("obs64.exe", "OBS"),
        ("discord.exe", "Discord"),
        ("steam.exe", "Steam"),
        ("spotify.exe", "Spotify"),
    ];
    let mut command = std::process::Command::new("tasklist");
    command.args(["/FO", "CSV", "/NH"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let Ok(output) = command.output() else { return "unknown".into() };
    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    let running: Vec<&str> = NOTABLE.iter().filter(|(exe, _)| text.lines().any(|line| line.starts_with(&format!("\"{exe}\"")))).map(|(_, name)| *name).collect();
    if running.is_empty() { "none of note".into() } else { running.join(", ") }
}

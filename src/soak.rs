//! The soak test — does the game run worse the longer it runs? For automatic testing only.
//!
//! Start the game with `INCUS_SOAK=<seconds>` (and if wanted `INCUS_SOAK_SPEED=<blocks a second>`,
//! `INCUS_SOAK_DISTANCE=<render distance in chunks>`, `INCUS_SOAK_NEAR=0` for no near light window),
//! together with `INCUS_START_SCREEN=worldgen` and
//! `INCUS_SELFTEST_SUITE=compiler`. It then:
//! 1. renders the newest bake with the World Compiler's recipe as it starts (the defaults), and
//!    spawns in the middle (see `worldgen::run_suite`);
//! 2. flies a wide circle over the world at a steady speed, a little above the ground, so chunks
//!    and distant tiles are made and dropped all the way round, and the same ground comes round
//!    again each lap;
//! 3. writes a line to the log every 10 seconds: the frame rate, and how many chunks, meshes,
//!    pictures and so on the game is holding (`render::soak_line`);
//! 4. quits when the time is up.
//!
//! Healthy: after the first lap the counts level off and the frame rate holds. A count that keeps
//! climbing lap after lap is a leak.

use crate::camera::FlyCamera;
use crate::player::ControlMode;
use crate::screen::Screen;
use crate::world::{SEA_LEVEL, Terrain};
use bevy::prelude::*;

/// The circle's radius, and how fast it's flown (blocks a second): a lap takes about four minutes.
/// `INCUS_SOAK_SPEED` flies it faster (David flies at 2,000 [2026-10-28]); the circle then widens, so
/// a lap still takes about a minute or more.
const RADIUS: f32 = 6000.0;
const SPEED: f32 = 150.0;


fn speed() -> f32 {
    std::env::var("INCUS_SOAK_SPEED").ok().and_then(|v| v.parse().ok()).unwrap_or(SPEED)
}

fn radius() -> f32 {
    RADIUS.max(speed() * 9.5)
}
/// How high above the ground (or the sea).
const ABOVE: f32 = 120.0;
/// Seconds between log lines.
const EVERY: f32 = 10.0;

/// Whether the game was started for a soak test.
pub fn wanted() -> bool {
    std::env::var("INCUS_SOAK").is_ok()
}

pub struct SoakPlugin;

impl Plugin for SoakPlugin {
    fn build(&self, app: &mut App) {
        let Some(seconds) = std::env::var("INCUS_SOAK").ok().and_then(|v| v.parse().ok()) else { return };
        app.insert_resource(Soak { seconds, started: None, next_line: 0.0 })
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
}

fn fly_round(
    mut soak: ResMut<Soak>,
    terrain: Res<Terrain>,
    time: Res<Time<Real>>,
    mut mode: ResMut<ControlMode>,
    mut cameras: Query<&mut FlyCamera>,
    mut render: ResMut<crate::render::RenderSettings>,
    mut light: ResMut<crate::render::LightSettings>,
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
        && render.render_distance_chunks != distance
    {
        render.render_distance_chunks = distance;
    }
    // `INCUS_SOAK_NEAR=0`: without the near chunks' light window (to measure what it costs).
    if std::env::var("INCUS_SOAK_NEAR").is_ok_and(|v| v == "0") && light.near {
        light.near = false;
    }
    // Round the circle (north of the middle to start with), facing the way it's going.
    let (radius, speed) = (radius(), speed());
    let angle = (now - started) * speed / radius;
    let (north, east) = (radius * angle.cos(), radius * angle.sin());
    let ground = map.height(north, east).unwrap_or(0.0).max(SEA_LEVEL as f32);
    for mut camera in &mut cameras {
        // Eased up and down, so a cliff doesn't jerk the view.
        let target = ground + ABOVE;
        let z = camera.position.z + (target - camera.position.z) * (2.0 * time.delta_secs()).min(1.0);
        camera.position = Vec3::new(north, east, if (camera.position.z - target).abs() > 500.0 { target } else { z });
        camera.heading = (angle.to_degrees() + 90.0).rem_euclid(360.0);
        camera.pitch = -12.0;
    }
}

fn report(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed_secs();
    let soak = world.resource::<Soak>();
    let Some(started) = soak.started else { return };
    let (flown, seconds, next_line) = (now - started, soak.seconds, soak.next_line);
    if flown >= next_line {
        let line = crate::render::soak_line(world);
        info!("Soak {flown:>5.0} s: {line}");
        world.resource_mut::<Soak>().next_line = next_line + EVERY;
    }
    if flown >= seconds {
        info!("Soak: done after {flown:.0} s");
        world.write_message(AppExit::Success);
    }
}

//! Time — the single clock for the whole simulation.
//!
//! The game has two kinds of "time":
//! - **Frames** (variable): the screen redraws as fast as it can, 60, 144, or whatever.
//!   Rendering, UI and input polling happen every frame.
//! - **Ticks** (fixed): the simulation advances in equal steps, 30 per second by default.
//!   Movement, mining, NPCs, growth — anything that *changes the world* — happens on ticks.
//!
//! Fixed ticks make the simulation consistent: a falling tree behaves the same at 30 fps
//! or 300 fps, and at 4× speed the game simply runs 4× as many ticks per second.
//!
//! `TimeControl` is the one place to pause, speed up, or change the tick rate. Every frame,
//! it is copied into Bevy's own clocks (`Time<Virtual>` and `Time<Fixed>`), which then run
//! the `FixedUpdate` schedule the right number of times. Simulation systems go in
//! `FixedUpdate`; they never need to know about pausing or speed.
//!
//! `WorldClock` is the *world's* own time (the calendar), saved with the world. The tick counter
//! in `TimeControl` only counts since the program started.

mod interpolation;

pub use interpolation::{InterpolationSettings, SimPosition, blend_fraction};

use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::save::WorldStateAppExt;
use crate::screen::Screen;
use bevy::prelude::*;
use bevy::time::TimeSystems;
use crate::ui::{EguiContexts, EguiPrimaryContextPass};
use serde::{Deserialize, Serialize};

/// Default simulation rate (ticks per second). See docs/decisions.md.
pub const DEFAULT_TICK_RATE: f64 = 30.0;
/// Fastest allowed speed multiplier (dev tool).
pub const MAX_SPEED: f64 = 4.0;

/// The simulation clock's controls and live statistics.
#[derive(Resource, Debug)]
pub struct TimeControl {
    /// When paused, no ticks run (rendering continues).
    pub paused: bool,
    /// Speed multiplier: 1.0 = normal, 4.0 = four times as fast. 0 also stops time.
    pub speed: f64,
    /// Ticks per second at speed 1.0.
    pub tick_rate: f64,
    /// Single ticks requested while paused (the "Step" button).
    pending_steps: u32,

    // ── Live statistics (read-only from outside) ──
    tick: u64,
    sim_seconds: f64,
    ticks_this_frame: u32,
}

impl Default for TimeControl {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1.0,
            tick_rate: DEFAULT_TICK_RATE,
            pending_steps: 0,
            tick: 0,
            sim_seconds: 0.0,
            ticks_this_frame: 0,
        }
    }
}

#[allow(dead_code)] // some of these are for systems that don't exist yet
impl TimeControl {
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.clamp(0.0, MAX_SPEED);
    }

    /// Advance exactly one tick (only works while paused).
    pub fn request_step(&mut self) {
        self.pending_steps += 1;
    }

    /// True when ticks are actually advancing.
    pub fn is_running(&self) -> bool {
        !self.paused && self.speed > 0.0
    }

    /// How many ticks have run since the program started.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Simulated seconds since the program started (affected by pause and speed).
    pub fn sim_seconds(&self) -> f64 {
        self.sim_seconds
    }

    /// How many ticks ran during the most recent frame (usually 0, 1 or 2; more when sped up).
    pub fn ticks_this_frame(&self) -> u32 {
        self.ticks_this_frame
    }
}

/// Simulated seconds in one in-game day (15 minutes light + 15 minutes dark).
pub const DAY_SECONDS: f64 = 1800.0;
pub const HOUR_SECONDS: f64 = DAY_SECONDS / 24.0;
pub const DAYS_PER_SEASON: u64 = 7;
const SEASONS: [&str; 4] = ["Spring", "Summer", "Autumn", "Winter"];

/// The world's calendar: simulated seconds since the world began. Saved with the world, and
/// it only moves on ticks, so pausing stops it and "TIME ×4" speeds it up.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldClock {
    pub seconds: f64,
}

impl WorldClock {
    /// "Year 1, Spring day 3, 14:05"
    pub fn describe(&self) -> String {
        describe_moment(self.seconds)
    }
}

/// A moment in world time, as a calendar date: "Year 1, Spring day 3, 14:05".
pub fn describe_moment(seconds: f64) -> String {
    let day = (seconds / DAY_SECONDS).floor().max(0.0) as u64;
    let days_per_year = DAYS_PER_SEASON * SEASONS.len() as u64;
    let year = day / days_per_year + 1;
    let season = SEASONS[((day % days_per_year) / DAYS_PER_SEASON) as usize];
    let day_of_season = day % DAYS_PER_SEASON + 1;
    let minutes_into_day = ((seconds.max(0.0) % DAY_SECONDS) / HOUR_SECONDS * 60.0) as u64;
    format!(
        "Year {year}, {season} day {day_of_season}, {:02}:{:02}",
        minutes_into_day / 60,
        minutes_into_day % 60
    )
}

/// A length of world time in plain words: "3 days 4 hours", "40 minutes".
pub fn describe_span(seconds: f64) -> String {
    let minutes = (seconds.max(0.0) / HOUR_SECONDS * 60.0).round() as u64;
    let (days, hours, minutes) = (minutes / (24 * 60), (minutes / 60) % 24, minutes % 60);
    match (days, hours) {
        (0, 0) => format!("{minutes} minutes"),
        (0, _) => format!("{hours} hours {minutes} minutes"),
        _ => format!("{days} days {hours} hours"),
    }
}

pub struct TimeControlPlugin;

impl Plugin for TimeControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeControl>()
            .init_resource::<WorldClock>()
            .add_world_state::<WorldClock>("clock")
            .init_resource::<InterpolationSettings>()
            // Before Bevy updates its clocks each frame, hand it our settings.
            .add_systems(First, apply_time_control.before(TimeSystems))
            // Just before the fixed ticks run: reset the per-frame counter, queue single steps.
            .add_systems(
                RunFixedMainLoop,
                prepare_ticks.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            // At the very start of every tick: count it.
            .add_systems(FixedFirst, count_tick)
            .add_plugins(interpolation::InterpolationPlugin)
            .add_systems(EguiPrimaryContextPass, draw_time_notice.run_if(in_state(Screen::Play)))
            .add_debug_page("Time", time_debug_page)
            .add_settings::<InterpolationSettings>("smooth_motion");
    }
}

/// A placeholder notice at the top of the screen whenever time isn't running at normal speed,
/// so a paused world never looks like a bug.
fn draw_time_notice(mut contexts: EguiContexts, control: Res<TimeControl>) -> Result {
    let text = if !control.is_running() {
        "TIME PAUSED  (Time tab to resume)".to_string()
    } else if control.speed != 1.0 {
        format!("TIME ×{:.2}", control.speed)
    } else {
        return Ok(());
    };
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("time_notice")));
    let screen = ctx.content_rect();
    let rect = egui::Rect::from_center_size(egui::pos2(screen.center().x, screen.top() + 24.0), egui::vec2(260.0, 26.0));
    painter.rect(
        rect,
        0.0,
        egui::Color32::BLACK,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 255)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(255, 140, 220),
    );
    Ok(())
}

/// Copies TimeControl into Bevy's clocks.
fn apply_time_control(
    control: Res<TimeControl>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    if control.is_running() {
        virtual_time.unpause();
        virtual_time.set_relative_speed_f64(control.speed);
    } else {
        virtual_time.pause();
    }

    let tick_rate = control.tick_rate.max(1.0);
    if (fixed_time.timestep().as_secs_f64() - 1.0 / tick_rate).abs() > 1e-9 {
        fixed_time.set_timestep_hz(tick_rate);
    }
}

fn prepare_ticks(mut control: ResMut<TimeControl>, mut fixed_time: ResMut<Time<Fixed>>) {
    control.ticks_this_frame = 0;
    if control.pending_steps > 0 {
        if !control.is_running() {
            // Pretend exactly one tick's worth of time has passed.
            let one_tick = fixed_time.timestep();
            fixed_time.accumulate_overstep(one_tick);
        }
        control.pending_steps -= 1;
    }
}

fn count_tick(mut control: ResMut<TimeControl>, mut clock: ResMut<WorldClock>, time: Res<Time>) {
    // Inside a tick, `Time` is the fixed clock, so `delta` is exactly one tick long.
    control.tick += 1;
    control.sim_seconds += time.delta_secs_f64();
    control.ticks_this_frame += 1;
    clock.seconds += time.delta_secs_f64();
}

// ── Debug page ──────────────────────────────────────────────────────────────

fn time_debug_page(ui: &mut egui::Ui, world: &mut World) {
    // Read Bevy's clock first (a separate borrow from TimeControl).
    let effective_speed = world.resource::<Time<Virtual>>().effective_speed_f64();
    let overstep = world.resource::<Time<Fixed>>().overstep_fraction();

    let mut control = world.resource_mut::<TimeControl>();

    ui.horizontal(|ui| {
        let label = if control.paused { "Resume" } else { "Pause" };
        if ui.button(label).clicked() {
            control.toggle_pause();
        }
        if ui
            .add_enabled(!control.is_running(), egui::Button::new("Step 1 tick"))
            .clicked()
        {
            control.request_step();
        }
    });

    ui.horizontal(|ui| {
        ui.label("Speed:");
        for preset in [0.25, 0.5, 1.0, 2.0, 4.0] {
            if ui
                .selectable_label(control.speed == preset, format!("{preset}×"))
                .clicked()
            {
                control.set_speed(preset);
            }
        }
    });
    ui.add(crate::debug::Slider::new(&mut control.speed, 0.0..=MAX_SPEED).text("speed ×"));
    ui.add(
        crate::debug::Slider::new(&mut control.tick_rate, 1.0..=120.0)
            .text("ticks / second")
            .integer(),
    );
    if ui.small_button("Reset tick rate to 30").clicked() {
        control.tick_rate = DEFAULT_TICK_RATE;
    }

    ui.separator();
    let clock = world.resource::<WorldClock>().describe();
    let control = world.resource::<TimeControl>();
    ui.label(format!("World clock: {clock}"));
    ui.label(format!("Tick: {}", control.tick()));
    ui.label(format!("Sim time: {}", format_duration(control.sim_seconds())));
    ui.label(format!("Ticks this frame: {}", control.ticks_this_frame()));
    ui.label(format!("Actual speed: {effective_speed:.2}×"));
    ui.label(format!("Between ticks: {:.0}%", overstep * 100.0));

    ui.separator();
    let mut interpolation = world.resource_mut::<InterpolationSettings>();
    ui.checkbox(&mut interpolation.enabled, "Smooth motion between ticks");
}

/// 3725.4 → "1:02:05.4"
fn format_duration(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u64;
    let minutes = ((seconds % 3600.0) / 60.0) as u64;
    let secs = seconds % 60.0;
    format!("{hours}:{minutes:02}:{secs:04.1}")
}

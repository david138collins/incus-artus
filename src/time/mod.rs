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

mod interpolation;

pub use interpolation::{InterpolationSettings, SimPosition};

use crate::debug::DebugAppExt;
use bevy::prelude::*;
use bevy::time::TimeSystems;
use bevy_egui::egui;

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

pub struct TimeControlPlugin;

impl Plugin for TimeControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeControl>()
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
            .add_debug_page("Time", time_debug_page);
    }
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

fn count_tick(mut control: ResMut<TimeControl>, time: Res<Time>) {
    // Inside a tick, `Time` is the fixed clock, so `delta` is exactly one tick long.
    control.tick += 1;
    control.sim_seconds += time.delta_secs_f64();
    control.ticks_this_frame += 1;
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
    ui.add(egui::Slider::new(&mut control.speed, 0.0..=MAX_SPEED).text("speed ×"));
    ui.add(
        egui::Slider::new(&mut control.tick_rate, 1.0..=120.0)
            .text("ticks / second")
            .integer(),
    );
    if ui.small_button("Reset tick rate to 30").clicked() {
        control.tick_rate = DEFAULT_TICK_RATE;
    }

    ui.separator();
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

//! World Gen — the suite we build the world generator with.
//!
//! This is not the generator. It's the workbench: a separate screen (F2) where a stack of
//! **steps** turns a seed into maps, one layer at a time, and every step's settings are sliders.
//! The real steps (plates, rivers, climate, biomes, regions, sites, history...) get designed one
//! by one inside it. Later the same screen grows into the player's world-creation map and the
//! manager map.
//!
//! What the suite does for every step (see `step.rs` for how a step is written):
//! - runs the stack on a background thread and shows each step's map as soon as it's done;
//! - notices when a setting changes and re-runs from that step down (earlier steps are kept);
//! - "one step at a time" mode, to watch the world being made step by step;
//! - keeps the map as it was after each step, so you can look back at any of them;
//! - times every step, and collects every step's log lines (click one to see where);
//! - an inspector showing every layer's value under the mouse;
//! - presets of the whole stack (seed, map size, colours, every step's settings).
//!
//! Files:
//! - `map.rs`     the map: cells, layers (numbers, kinds, regions) and marks
//! - `step.rs`    what a step is
//! - `runner.rs`  running steps in the background
//! - `noise.rs`   seeded smooth noise
//! - `colours.rs` the palette and colour ramps
//! - `view.rs`    the screen: step list, map, inspector, log
//! - `samples.rs` TEMPORARY sample steps, to have something to run

mod colours;
mod map;
mod noise;
mod runner;
mod samples;
mod step;
mod view;

use crate::debug::DebugAppExt;
use crate::screen::Screen;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use colours::{KindColour, default_palette};
use map::Map;
use runner::{Job, Poll, Progress};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;
use step::{LogLine, Stage, Step};

/// The steps, in order. New steps go here.
fn sample_steps() -> Vec<Box<dyn Step>> {
    vec![
        Box::new(Stage(samples::Height::default())),
        Box::new(Stage(samples::Moisture::default())),
        Box::new(Stage(samples::LandKinds::default())),
        Box::new(Stage(samples::Regions::default())),
    ]
}

/// The settings that belong to the whole stack rather than one step.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StackSettings {
    #[serde(with = "crate::save::hex_u64")]
    pub seed: u64,
    /// The world circle's width in blocks.
    pub world_diameter: f32,
    /// How many blocks wide one cell (one pixel of the map) is.
    pub blocks_per_cell: f32,
    pub palette: Vec<KindColour>,
}

impl Default for StackSettings {
    fn default() -> Self {
        Self { seed: 1, world_diameter: 72_000.0, blocks_per_cell: 64.0, palette: default_palette() }
    }
}

/// What a preset file holds: the stack's settings and every step's settings by name.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StackPreset {
    pub stack: StackSettings,
    pub steps: BTreeMap<String, toml::Table>,
}

/// One step's latest result.
pub struct StepResult {
    pub map: Map,
    pub millis: f64,
    pub lines: Vec<LogLine>,
    /// Settings have changed since this was made; a newer one is on its way (or waiting for
    /// "Next step").
    pub stale: bool,
}

#[derive(Resource)]
pub struct Suite {
    pub stack: StackSettings,
    pub steps: Vec<Box<dyn Step>>,
    pub results: Vec<Option<StepResult>>,
    /// Run the rest of the stack by itself whenever something changes. Off = one step at a time.
    pub auto_run: bool,
    /// The step whose settings are open.
    pub selected: usize,
    pub view: view::ViewState,

    job: Option<Job>,
    /// Steps from here down need running again.
    pending_from: Option<usize>,
    /// Run only as far as this step (for "Next step"); `None` = to the end.
    pending_to: Option<usize>,
    /// What everything looked like last frame, to notice changes.
    seen_stack: StackSettings,
    seen_steps: Vec<toml::Table>,
    last_change: Instant,
}

impl Default for Suite {
    fn default() -> Self {
        let steps = sample_steps();
        let seen_steps = steps.iter().map(|step| step.save()).collect();
        Self {
            stack: StackSettings::default(),
            results: (0..steps.len()).map(|_| None).collect(),
            steps,
            auto_run: true,
            selected: 0,
            view: view::ViewState::default(),
            job: None,
            pending_from: Some(0),
            pending_to: None,
            seen_stack: StackSettings::default(),
            seen_steps,
            last_change: Instant::now(),
        }
    }
}

impl Suite {
    pub fn is_running(&self) -> bool {
        self.job.is_some()
    }

    /// Which step is running now, if any.
    pub fn running_step(&self) -> Option<usize> {
        self.job.as_ref().map(|job| job.current)
    }

    /// The first step that has no up-to-date result.
    pub fn first_unfinished(&self) -> Option<usize> {
        self.results.iter().position(|result| result.as_ref().is_none_or(|r| r.stale))
    }

    /// Run steps `from` onward (to `to`, or to the end).
    pub fn request_run(&mut self, from: usize, to: Option<usize>) {
        self.mark_stale(from);
        self.pending_from = Some(self.pending_from.map_or(from, |p| p.min(from)));
        self.pending_to = to;
    }

    /// "Next step": run the first step that isn't up to date, and only that one.
    pub fn next_step(&mut self) {
        if let Some(step) = self.first_unfinished() {
            self.request_run(step, Some(step));
        }
    }

    pub fn stop(&mut self) {
        if let Some(job) = self.job.take() {
            job.stop();
        }
        self.pending_from = None;
    }

    fn mark_stale(&mut self, from: usize) {
        for result in self.results.iter_mut().skip(from).flatten() {
            result.stale = true;
        }
    }

    pub fn preset(&self) -> StackPreset {
        StackPreset {
            stack: self.stack.clone(),
            steps: self.steps.iter().map(|step| (step.name().to_string(), step.save())).collect(),
        }
    }

    /// Steps the preset doesn't mention go back to their defaults.
    pub fn apply_preset(&mut self, preset: &StackPreset) {
        self.stack = preset.stack.clone();
        for step in &mut self.steps {
            match preset.steps.get(step.name()) {
                Some(settings) => {
                    if let Err(error) = step.load(settings) {
                        warn!("World gen preset: couldn't read the settings of '{}': {error}", step.name());
                        step.reset();
                    }
                }
                None => step.reset(),
            }
        }
    }

    /// Compares everything with last frame. A changed step means re-running from it; a changed
    /// seed or map size means re-running everything. Colours only change the picture.
    fn notice_changes(&mut self) {
        let shape_changed = self.stack.seed != self.seen_stack.seed
            || self.stack.world_diameter != self.seen_stack.world_diameter
            || self.stack.blocks_per_cell != self.seen_stack.blocks_per_cell;
        let mut changed_from = shape_changed.then_some(0);
        for (index, step) in self.steps.iter().enumerate() {
            let now = step.save();
            if now != self.seen_steps[index] {
                changed_from = Some(changed_from.map_or(index, |c: usize| c.min(index)));
                self.seen_steps[index] = now;
            }
        }
        self.seen_stack = self.stack.clone();
        if let Some(from) = changed_from {
            self.last_change = Instant::now();
            self.mark_stale(from);
            if self.auto_run {
                self.request_run(from, None);
            }
        }
    }

    /// Collects finished steps from the running job, and starts the next run when one is wanted.
    fn drive(&mut self) {
        if let Some(job) = &mut self.job {
            loop {
                match job.poll() {
                    Poll::Progress(Progress::Done { index, map, millis, lines }) => {
                        // A result can arrive already out of date, if a setting moved while it ran.
                        let stale = self.pending_from.is_some_and(|from| from <= index);
                        self.results[index] = Some(StepResult { map, millis, lines, stale });
                        self.view.picture_changed();
                    }
                    Poll::Progress(Progress::Stopped) | Poll::Over => {
                        self.job = None;
                        break;
                    }
                    Poll::Waiting => break,
                }
            }
        }

        let Some(mut from) = self.pending_from else { return };
        if let Some(job) = &self.job {
            // The job is working on something already out of date. Let it finish if the sliders
            // are still moving (so fast steps update live while dragging); once they've been
            // still for a moment, stop it and start the fresh run.
            if job.current >= from && self.last_change.elapsed().as_secs_f32() > 0.15 {
                self.stop();
                self.pending_from = Some(from);
            } else {
                return;
            }
        }
        // Start from the earliest step without an up-to-date result before `from`, if any.
        if let Some(unfinished) = self.first_unfinished() {
            from = from.min(unfinished);
        }
        let last = self.pending_to.unwrap_or(self.steps.len() - 1).max(from).min(self.steps.len() - 1);
        let start_map = match from {
            0 => Map::new(self.stack.world_diameter, self.stack.blocks_per_cell),
            _ => match &self.results[from - 1] {
                Some(result) => result.map.clone(),
                None => return,
            },
        };
        self.mark_stale(from);
        let steps = self.steps.iter().map(|step| step.copy()).collect();
        self.job = Some(Job::start(steps, from, last, start_map, self.stack.seed));
        self.pending_from = None;
        self.pending_to = None;
    }
}

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Suite>()
            .add_systems(Update, run_suite.run_if(in_state(Screen::WorldGen)))
            .add_systems(EguiPrimaryContextPass, draw_suite.run_if(in_state(Screen::WorldGen)))
            .add_debug_page("World Gen", world_gen_debug_page);
    }
}

fn run_suite(mut suite: ResMut<Suite>) {
    suite.notice_changes();
    suite.drive();
}

fn draw_suite(mut contexts: EguiContexts, mut suite: ResMut<Suite>, mut next: ResMut<NextState<Screen>>) -> Result {
    let ctx = contexts.ctx_mut()?.clone();
    if view::draw(&ctx, &mut suite) {
        next.set(Screen::Play);
    }
    // Keep redrawing while a run is going, so finished steps show up straight away.
    if suite.is_running() {
        ctx.request_repaint();
    }
    Ok(())
}

fn world_gen_debug_page(ui: &mut egui::Ui, world: &mut World) {
    ui.label("The World Gen suite is a separate screen. The world is switched off while it's open.");
    let in_suite = *world.resource::<State<Screen>>().get() == Screen::WorldGen;
    let label = if in_suite { "Back to the world  [F2]" } else { "Open the World Gen suite  [F2]" };
    if ui.button(label).clicked() {
        world.resource_mut::<NextState<Screen>>().set(if in_suite { Screen::Play } else { Screen::WorldGen });
    }
    let suite = world.resource::<Suite>();
    let done = suite.results.iter().filter(|r| r.as_ref().is_some_and(|r| !r.stale)).count();
    ui.label(format!("{done} of {} steps up to date.", suite.steps.len()));
}

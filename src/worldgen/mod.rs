//! World Gen — the suite we build the world generator with.
//!
//! Rebuilt from the bottom, one tool at a time: each tool is tried, tuned and kept (or changed)
//! before the next one is added. See `docs/worldgen.md` for the order so far.
//!
//! **Tool 1: the Perlin field.** One greyscale field of Perlin noise, every mathematical setting
//! on a slider, and noise presets: when a useful kind of noise turns up, save it by name.
//!
//! The suite is its own screen (F2), with the physical world switched off. The field is made on
//! a background thread and remade whenever a setting changes.
//!
//! For the art workflow, a field can be coloured with a GIMP palette (`.gpl`), filtered by hue,
//! saturation and lightness, and the picture in view (or the whole field) exported as a PNG.
//!
//! Files:
//! - `field.rs`   a field: one value per pixel
//! - `noise.rs`   Perlin noise and its settings
//! - `runner.rs`  making a field in the background
//! - `palette.rs` colouring with a palette, the HSL filter, reading and writing `.gpl` files
//! - `export.rs`  saving pictures as PNG
//! - `view.rs`    the screen

mod export;
mod field;
mod noise;
mod palette;
mod runner;
mod view;

use crate::debug::DebugAppExt;
use crate::screen::Screen;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use noise::PerlinSettings;
use runner::{Finished, Job, Poll};

#[derive(Resource)]
pub struct Suite {
    /// The field's seed. Kept apart from the noise settings, so a noise preset is a *kind* of
    /// noise that can be tried with any seed.
    pub seed: u64,
    /// The field is a square this many blocks across (the world is 72,000).
    pub size_blocks: f32,
    pub blocks_per_pixel: f32,
    pub noise: PerlinSettings,
    /// How the field is coloured on screen and in exports. Only changes the picture, never the field.
    pub colouring: palette::Colouring,
    /// The latest field made.
    pub result: Option<Finished>,
    pub view: view::ViewState,

    job: Option<Job>,
    /// Settings changed since the job (or the result) started: make it again.
    wanted: bool,
    /// What the settings were last frame, to notice changes.
    seen: (u64, f32, f32, PerlinSettings),
}

impl Default for Suite {
    fn default() -> Self {
        let noise = PerlinSettings::default();
        Self {
            seed: 1,
            size_blocks: 72_000.0,
            blocks_per_pixel: 128.0,
            seen: (0, 0.0, 0.0, noise.clone()),
            noise,
            colouring: palette::Colouring::default(),
            result: None,
            view: view::ViewState::default(),
            job: None,
            wanted: true,
        }
    }
}

impl Suite {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    fn settings_now(&self) -> (u64, f32, f32, PerlinSettings) {
        (self.seed, self.size_blocks, self.blocks_per_pixel, self.noise.clone())
    }

    /// Notices changes, collects the finished field, and starts the next one when wanted.
    ///
    /// While a slider is being dragged, the running field is left to finish (so the picture keeps
    /// up as fast as fields can be made) and the next one starts straight after with the newest
    /// settings. A slow one (over a quarter of a second) is abandoned instead.
    fn drive(&mut self) {
        let now = self.settings_now();
        if now != self.seen {
            self.seen = now;
            self.wanted = true;
        }
        if let Some(job) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.result = Some(*finished);
                    self.view.picture_changed();
                    self.job = None;
                }
                Poll::Over => self.job = None,
                // A slow field that's already out of date isn't worth waiting for.
                Poll::Waiting if self.wanted && job.started.elapsed().as_secs_f32() > 0.25 => {
                    job.stop();
                    self.job = None;
                }
                Poll::Waiting => return,
            }
        }
        if self.wanted {
            self.wanted = false;
            self.job = Some(Job::start(self.noise.clone(), self.seed, self.size_blocks, self.blocks_per_pixel));
        }
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
    suite.drive();
}

fn draw_suite(mut contexts: EguiContexts, mut suite: ResMut<Suite>, mut next: ResMut<NextState<Screen>>) -> Result {
    let ctx = contexts.ctx_mut()?.clone();
    if view::draw(&ctx, &mut suite) {
        next.set(Screen::Play);
    }
    // Keep redrawing while a field is being made, so it shows up the moment it's done.
    if suite.is_making() {
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
}

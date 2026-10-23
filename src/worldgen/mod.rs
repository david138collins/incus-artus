//! World Gen — the suite we build the world generator with.
//!
//! Rebuilt from the bottom, one tool at a time: each tool is tried, tuned and kept (or changed)
//! before the next one is added. See `docs/worldgen.md` for the order so far.
//!
//! **A stack of layers, like a graphics editor.** It starts blank. Add a layer of a chosen type
//! (a Perlin noise field, or an image from a PNG), name it, tune it, and choose how it mixes with
//! the layers below (multiply, add…). Layers mix their numbers; the result is coloured with a
//! palette. Numbers become colours, and colours (pictures) become numbers.
//! Noise presets save useful kinds of noise by name. What's on screen (or the whole field) can be
//! exported as a PNG, for the art workflow.
//!
//! The suite is its own screen (F2), with the physical world switched off. Each layer is made on a
//! background thread and remade whenever its settings change.
//!
//! Files:
//! - `field.rs`   a field: one value per pixel
//! - `made.rs`    what every kind of layer makes, and the output shaping they share
//! - `noise.rs`   Perlin noise and its settings
//! - `image.rs`   image layers: PNG pictures laid onto the field
//! - `layers.rs`  the layer stack
//! - `blend.rs`   how a layer mixes with the ones below it
//! - `runner.rs`  making a layer in the background
//! - `palette.rs` colouring with a palette, the HSL filter, reading and writing `.gpl` files
//! - `export.rs`  saving pictures as PNG
//! - `view.rs`    the screen

mod blend;
mod export;
mod field;
mod image;
mod layers;
mod made;
mod noise;
mod palette;
mod runner;
mod view;

use crate::debug::DebugAppExt;
use crate::screen::Screen;
use bevy::prelude::*;
use bevy::window::FileDragAndDrop;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use layers::{Layer, LayerKind};
use palette::Colouring;
use std::path::Path;
use std::sync::Arc;

/// Blocks along one side of a chunk (see `docs/decisions.md`).
pub const CHUNK_BLOCKS: f32 = 32.0;

/// `Suite::selected` for "the result" (all the layers mixed) rather than one layer. Layer ids
/// start at 1.
pub const RESULT: u32 = 0;

#[derive(Resource)]
pub struct Suite {
    /// The field is a square this many blocks across (the world is 72,000).
    pub size_blocks: f32,
    /// How many chunks wide one pixel is.
    pub chunks_per_pixel: u32,
    /// The layers, bottom first.
    pub layers: Vec<Layer>,
    /// The layer whose settings are open, or `RESULT`.
    pub selected: Option<u32>,
    /// How the result (all the layers mixed) is coloured.
    pub colouring: Colouring,
    /// Show only the open layer, in its own colours, instead of the result.
    pub show_alone: bool,
    pub view: view::ViewState,
    next_id: u32,
}

impl Default for Suite {
    fn default() -> Self {
        let mut suite = Self {
            size_blocks: 72_000.0,
            chunks_per_pixel: 8,
            layers: Vec::new(),
            selected: None,
            colouring: Colouring::default(),
            show_alone: false,
            view: view::ViewState::default(),
            next_id: 1,
        };
        // For automatic testing: `INCUS_SELFTEST_SUITE=<a PNG>` starts with a Perlin layer and that
        // picture on top of it, multiplied in, instead of a blank page.
        if let Ok(path) = std::env::var("INCUS_SELFTEST_SUITE") {
            suite.add_perlin();
            match suite.add_image(Path::new(&path)) {
                Ok(()) => {
                    if let Some(layer) = suite.layers.last_mut() {
                        layer.mix = blend::Mix::Multiply;
                    }
                }
                Err(error) => warn!("Self-test picture: {error}"),
            }
        }
        suite
    }
}

impl Suite {
    pub fn blocks_per_pixel(&self) -> f32 {
        self.chunks_per_pixel as f32 * CHUNK_BLOCKS
    }

    pub fn is_making(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_making())
    }

    pub fn selected_layer(&mut self) -> Option<&mut Layer> {
        let id = self.selected?;
        self.layers.iter_mut().find(|layer| layer.id == id)
    }

    /// Adds a layer on top of the stack and opens its settings.
    pub fn add_layer(&mut self, kind: LayerKind) {
        let name = layers::next_name(&self.layers, kind.type_name());
        let id = self.next_id;
        self.next_id += 1;
        self.layers.push(Layer::new(id, name, kind));
        self.selected = Some(id);
    }

    pub fn add_perlin(&mut self) {
        let seed = crate::fate::derive_seed(fresh_number(), "perlin layer");
        self.add_layer(LayerKind::Perlin { seed, settings: noise::PerlinSettings::default() });
    }

    /// Loads a PNG as a new image layer, shown at one picture pixel per field pixel, in the middle.
    pub fn add_image(&mut self, path: &Path) -> Result<(), String> {
        let picture = image::load_png(path)?;
        let file = path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        let placement = image::Placement { blocks_per_pixel: self.blocks_per_pixel(), ..image::Placement::default() };
        let stem = path.file_stem().map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
        self.add_layer(LayerKind::Image { file, picture: Arc::new(picture), placement });
        // Image layers are named after their file.
        if let Some(layer) = self.layers.last_mut() {
            layer.name = stem;
        }
        Ok(())
    }

    pub fn remove_layer(&mut self, id: u32) {
        self.layers.retain(|layer| layer.id != id);
        if self.selected == Some(id) {
            self.selected = self.layers.last().map(|layer| layer.id);
        }
    }

    /// Moves a layer up (toward the top) or down the stack.
    pub fn move_layer(&mut self, id: u32, up: bool) {
        let Some(at) = self.layers.iter().position(|layer| layer.id == id) else { return };
        let to = if up { at + 1 } else { at.wrapping_sub(1) };
        if to < self.layers.len() {
            self.layers.swap(at, to);
        }
    }
}

/// A number that's different every time (for new seeds).
pub fn fresh_number() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64)
}

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Suite>()
            .add_systems(Update, (run_suite, take_dropped_files).run_if(in_state(Screen::WorldGen)))
            .add_systems(EguiPrimaryContextPass, draw_suite.run_if(in_state(Screen::WorldGen)))
            .add_debug_page("World Gen", world_gen_debug_page);
    }
}

fn run_suite(mut suite: ResMut<Suite>) {
    let (size, blocks) = (suite.size_blocks, suite.blocks_per_pixel());
    for layer in &mut suite.layers {
        layer.drive(size, blocks);
    }
}

/// PNG files dragged onto the window become image layers.
fn take_dropped_files(mut drops: MessageReader<FileDragAndDrop>, mut suite: ResMut<Suite>) {
    for drop in drops.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = drop {
            let message = match suite.add_image(path_buf) {
                Ok(()) => format!("Added {}", path_buf.display()),
                Err(error) => format!("Couldn't use {}: {error}", path_buf.display()),
            };
            suite.view.say(message);
        }
    }
}

fn draw_suite(mut contexts: EguiContexts, mut suite: ResMut<Suite>, mut next: ResMut<NextState<Screen>>) -> Result {
    let ctx = contexts.ctx_mut()?.clone();
    if view::draw(&ctx, &mut suite) {
        next.set(Screen::Play);
    }
    // Keep redrawing while a layer is being made, so it shows up the moment it's done.
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

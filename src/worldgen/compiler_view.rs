//! The Terrain Compiler's screen: pick a bake, set the compiler's whole-map jobs and the
//! Interpreter's recipe, and see the result as shaded relief. (The work itself is in `compiler.rs`
//! and `crate::world::interpreter`.)
//!
//! ```text
//! ┌──── top: which bake, land share ───────────────────────────────────────────────┐
//! │ Terrain Compiler │ World Interpreter │                          │ under the    │
//! │ sea level        │ the chain         │   the world, in relief   │ mouse        │
//! │ coastline        │ land and sea      │                          │              │
//! │ foothills        │ mountains         │                          │              │
//! │ save, render     │ hills, blocks     │                          │              │
//! └──────────────────┴───────────────────┴──────────────────────────┴──────────────┘
//! ```

use super::Suite;
use super::bake::{self, Baked};
use super::canvas::Camera;
use super::compiler::{self, Bake, CompileSettings, Compiled, HIGHEST, SEA_LEVEL, Spline, TOP_OF_THE_WORLD};
use crate::world::InterpreterSettings;
use super::image::{Picture, Placement};
use super::layers::LayerKind;
use super::runner::{Finished, Job, Poll};
use super::lab::{self, Lab};
use super::view::{MAGENTA, PINK, compass};
use super::world::defaults_row;
use crate::debug::preset_bar;
use bevy::math::IVec2;
use bevy_egui::egui::{self, Color32, Vec2};
use std::sync::Arc;

/// Past the edge of the world.
const BEYOND: [u8; 3] = [10, 10, 14];
/// The scratch bake EZ-Bake writes (and overwrites) each time.
const EZ_BAKE: &str = "_ez-bake";

pub struct Compiler {
    pub settings: CompileSettings,
    /// Which bake is chosen, and the bake itself once read.
    chosen: Option<String>,
    bake: Option<Arc<Bake>>,
    note: String,

    result: Option<Finished<Compiled>>,
    job: Option<Job<Compiled>>,
    seen: Option<(String, CompileSettings)>,
    wanted: bool,
    version: u64,

    /// How strongly slopes are shaded (only changes the look).
    pub relief: f32,
    /// Your own settings, kept aside while the A/B button shows the defaults.
    ab: Option<CompileSettings>,
    /// EZ-Bake: a scratch world being baked from the Tectonic Sim's settings; and whether the next
    /// bake read should keep the compiler's settings (an EZ-Bake reload) rather than take the
    /// bake's own water level.
    ez: Option<Job<Result<Baked, String>>>,
    keep_settings: bool,
    /// The name to keep the scratch world under.
    keep_name: String,
    /// A picture waiting to be sent to the Graphics Kit as a layer (name, picture, blocks per
    /// pixel).
    send: Option<(String, Picture, f32)>,
    /// "Render World and Spawn": the world name, seed and map waiting to be started.
    pub render: Option<(String, u64, Arc<crate::world::CompiledMap>, IVec2)>,
    /// The spawn picker is open (after "Render World and Spawn").
    picking: bool,
    /// The name to give a rendered world (blank = the bake's name).
    world_name: String,
    camera: Camera,
    texture: Option<egui::TextureHandle>,
    painted_for: Option<(u64, u32)>,
    /// The terrain lab (the close-up, its layer switches and cross-section), its picture, and the
    /// block under the mouse in it (north, east, Z).
    pub lab: Lab,
    lab_texture: Option<egui::TextureHandle>,
    lab_painted_for: Option<(u64, u32)>,
    lab_hover: Option<(f32, f32, f32)>,
    /// The overlays' picture, and which of their pictures it holds.
    overlay_texture: Option<egui::TextureHandle>,
    overlay_painted_for: Option<u64>,
    hover: Option<(usize, usize)>,
    /// The Terrain Compiler and World Interpreter panels, folded away to thin strips.
    pub folded: [bool; 2],
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            settings: CompileSettings::default(),
            chosen: None,
            bake: None,
            note: String::new(),
            result: None,
            job: None,
            seen: None,
            wanted: false,
            version: 0,
            relief: 5.0,
            ab: None,
            ez: None,
            keep_settings: false,
            keep_name: String::new(),
            send: None,
            render: None,
            picking: false,
            world_name: String::new(),
            camera: Camera::default(),
            texture: None,
            painted_for: None,
            lab: Lab::default(),
            lab_texture: None,
            lab_painted_for: None,
            lab_hover: None,
            overlay_texture: None,
            overlay_painted_for: None,
            hover: None,
            folded: [false; 2],
        }
    }
}

impl Compiler {
    pub fn is_making(&self) -> bool {
        self.job.is_some() || self.ez.is_some()
    }

    /// What it's busy with, if anything (for the suite's processing meter).
    pub fn working(&self) -> Option<&'static str> {
        if self.ez.is_some() {
            Some("EZ-baking")
        } else if self.job.is_some() {
            Some("compiling")
        } else if self.lab.is_making() {
            Some("drawing the close-up")
        } else if self.lab.overlays.is_making() {
            Some("drawing overlays")
        } else {
            None
        }
    }

    /// Starts an EZ-Bake: the world `order` describes, baked in the background into the scratch
    /// bake.
    fn start_ez(&mut self, mut order: bake::Order) {
        order.name = EZ_BAKE.to_string();
        self.ez = Some(Job::start(move |stop| bake::bake(&order, stop)));
        self.note = "EZ-Baking a new world from the Tectonic Sim's settings…".to_string();
    }

    /// Picks up a finished EZ-Bake and reads it in, keeping the compiler's settings.
    fn collect_ez(&mut self) {
        let Some(job) = &self.ez else { return };
        match job.poll() {
            Poll::Done(finished) => {
                self.ez = None;
                match finished.made {
                    Ok(baked) => {
                        let ending = if baked.settled { "everything stopped" } else { "step limit" };
                        self.note = format!("EZ-Baked in {:.1} s ({} steps, {ending}).", finished.millis / 1000.0, baked.steps);
                        self.chosen = Some(EZ_BAKE.to_string());
                        // Read it again even though the name is the same, keeping these settings.
                        self.bake = None;
                        self.keep_settings = true;
                    }
                    Err(error) => self.note = format!("EZ-Bake failed: {error}"),
                }
            }
            Poll::Over => {
                self.ez = None;
                self.note = "EZ-Bake stopped.".to_string();
            }
            Poll::Waiting => {}
        }
    }

    /// Reads the chosen bake if it isn't read yet (the newest bake, if none is chosen), compiles
    /// again when the bake or the settings change, and collects finished work.
    pub fn drive(&mut self) {
        self.collect_ez();
        if self.chosen.is_none() {
            self.chosen = newest_bake();
        }
        let Some(chosen) = self.chosen.clone() else { return };
        if self.bake.as_ref().is_none_or(|bake| bake.name != chosen) {
            match compiler::load(&bake::bakes_folder().join(&chosen)) {
                Ok(bake) => {
                    // Start from the water level it was baked with (unless this is an EZ-Bake
                    // being read again: then keep everything as it is).
                    if !std::mem::take(&mut self.keep_settings) {
                        self.settings.water = bake.info.water_level;
                    }
                    let tilt = if bake.tilt.iter().any(|&t| t != 0.0) { "" } else { " (made before tilt was saved: no grading)" };
                    self.note = format!("Read \"{chosen}\": {} × {} pixels, {} blocks each{tilt}.", bake.width, bake.width, bake.info.blocks_per_pixel);
                    self.bake = Some(Arc::new(bake));
                    self.result = None;
                    // A new bake always needs compiling, even under the same name with the same
                    // settings (every EZ-Bake is called "_ez-bake").
                    self.seen = None;
                }
                Err(error) => {
                    self.note = format!("Couldn't read \"{chosen}\": {error}");
                    self.chosen = None;
                    self.bake = None;
                    return;
                }
            }
        }
        let now = (chosen, self.settings.clone());
        if self.seen.as_ref() != Some(&now) {
            self.seen = Some(now);
            self.wanted = true;
        }
        if let Some(job) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.result = Some(*finished);
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting if self.wanted && job.started.elapsed().as_secs_f32() > 0.25 => {
                    job.stop();
                    self.job = None;
                }
                Poll::Waiting => return,
            }
        }
        if self.wanted
            && let Some(bake) = self.bake.clone()
        {
            self.wanted = false;
            let settings = self.settings.clone();
            self.job = Some(Job::start(move |stop| compiler::compile(&bake, &settings, stop)));
        }
    }
}

/// Copies a bake's folder (its files; bakes have no folders inside) to `to`, replacing what's
/// there.
fn copy_folder(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|error| error.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|error| error.to_string())?.flatten() {
        if entry.path().is_file() {
            std::fs::copy(entry.path(), to.join(entry.file_name())).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

/// The most recently made bake.
fn newest_bake() -> Option<String> {
    bake::saved_bakes().into_iter().max_by_key(|name| {
        bake::bakes_folder().join(name).join("map.toml").metadata().and_then(|meta| meta.modified()).unwrap_or(std::time::UNIX_EPOCH)
    })
}

/// The colour of ground at height `z`: blues under the sea (darker deeper), then green lowland,
/// olive, tan, grey rock and white peaks.
fn colour_at(z: f32) -> [u8; 3] {
    const SEA: [(f32, [u8; 3]); 4] = [(1.0, [12, 28, 70]), (60.0, [25, 60, 115]), (100.0, [40, 95, 150]), (127.0, [70, 140, 180])];
    const LAND: [(f32, [u8; 3]); 7] = [
        (128.0, [96, 140, 74]),
        (150.0, [120, 160, 84]),
        (185.0, [170, 170, 110]),
        (220.0, [160, 130, 95]),
        (260.0, [130, 120, 115]),
        (300.0, [185, 185, 185]),
        (333.0, [245, 245, 250]),
    ];
    let stops: &[(f32, [u8; 3])] = if z < SEA_LEVEL as f32 { &SEA } else { &LAND };
    let z = z.clamp(stops[0].0, stops[stops.len() - 1].0);
    for pair in stops.windows(2) {
        let ((z0, c0), (z1, c1)) = (pair[0], pair[1]);
        if z <= z1 {
            let t = (z - z0) / (z1 - z0).max(1e-6);
            return [0, 1, 2].map(|k| (c0[k] as f32 + (c1[k] as f32 - c0[k] as f32) * t) as u8);
        }
    }
    stops[stops.len() - 1].1
}

/// The world as shaded relief: coloured by height, with slopes lit from the northwest (land only;
/// the sea is flat colour). `relief` exaggerates the slopes.
fn relief_picture(compiled: &Compiled, blocks_per_pixel: f32, relief: f32) -> Vec<[u8; 3]> {
    let width = compiled.width;
    let z = |col: usize, row: usize| compiled.z[row.min(width - 1) * width + col.min(width - 1)] as f32;
    let half = width as f32 / 2.0;
    // Light from the northwest, fairly high up (east, south, up).
    let light = {
        let (x, y, up) = (-1.0f32, -1.0f32, 1.4f32);
        let length = (x * x + y * y + up * up).sqrt();
        (x / length, y / length, up / length)
    };
    (0..width * width)
        .map(|i| {
            let (col, row) = (i % width, i / width);
            let (dx, dy) = (col as f32 + 0.5 - half, row as f32 + 0.5 - half);
            if dx * dx + dy * dy > half * half {
                return BEYOND;
            }
            let here = z(col, row);
            let colour = colour_at(here);
            if here < SEA_LEVEL as f32 {
                return colour;
            }
            let east = (z(col + 1, row) - z(col.saturating_sub(1), row)) / (2.0 * blocks_per_pixel) * relief;
            let south = (z(col, row + 1) - z(col, row.saturating_sub(1))) / (2.0 * blocks_per_pixel) * relief;
            let length = (east * east + south * south + 1.0).sqrt();
            let lit = (-east * light.0 - south * light.1 + light.2) / length;
            let shade = (lit / light.2).clamp(0.35, 1.5);
            colour.map(|c| (c as f32 * shade).min(255.0) as u8)
        })
        .collect()
}

pub fn draw(root: &mut egui::Ui, suite: &mut Suite) {
    // Space: EZ-Bake this world again (the same seed) with the Tectonic Sim's settings as they are
    // now. Shift + Space: EZ-Bake a new world (a new seed). Not while typing, say a preset's name.
    let typing = root.ctx().egui_wants_keyboard_input();
    let mut new_world = !typing && root.input_mut(|input| input.consume_key(egui::Modifiers::SHIFT, egui::Key::Space));
    let mut same_world = !typing && root.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Space));
    let compiler = &mut suite.compiler;
    egui::Panel::top("world compiler bar").show(root, |ui| {
        ui.horizontal_wrapped(|ui| {
            if compiler.ez.is_some() {
                ui.spinner();
                ui.label("EZ-Baking…");
            } else {
                if ui
                    .button("EZ-Bake  [Space]")
                    .on_hover_text(
                        "This world again, with the Tectonic Sim's settings as they are now (the same seed), baked\n\
                         (into \"_ez-bake\", replaced each time) and read straight in here, keeping these compiler\n\
                         settings. For seeing what a tectonics change does to the finished world.",
                    )
                    .clicked()
                {
                    same_world = true;
                }
                if ui
                    .button("EZ-Bake a new world  [Shift+Space]")
                    .on_hover_text(
                        "The same, with a new world seed: for seeing how these settings do across many worlds.\n\
                         The new seed becomes the Tectonic Sim's too, so you can go and watch that world run.",
                    )
                    .clicked()
                {
                    new_world = true;
                }
            }
            ui.separator();
            ui.label("Bake:");
            let bakes = bake::saved_bakes();
            egui::ComboBox::from_id_salt("world compiler bake").selected_text(compiler.chosen.clone().unwrap_or_else(|| "none yet".to_string())).show_ui(ui, |ui| {
                if bakes.is_empty() {
                    ui.label("(no bakes yet: bake one in the Tectonic Sim)");
                }
                for name in &bakes {
                    ui.selectable_value(&mut compiler.chosen, Some(name.clone()), name);
                }
            });
            if ui.small_button("Bakes folder").clicked()
                && let Err(error) = super::export::open_folder(&bake::bakes_folder())
            {
                compiler.note = format!("Couldn't open the folder: {error}");
            }
            // An EZ-Baked world worth keeping: copy it to a bake of its own.
            if compiler.chosen.as_deref() == Some(EZ_BAKE) && compiler.ez.is_none() {
                ui.separator();
                ui.add(egui::TextEdit::singleline(&mut compiler.keep_name).desired_width(110.0).hint_text("name"));
                let name = bake::clean_name(&compiler.keep_name);
                if ui
                    .add_enabled(!name.is_empty() && name != EZ_BAKE, egui::Button::new("Keep this world"))
                    .on_hover_text("Copies this EZ-Baked world to a bake of its own, under this name, so the next EZ-Bake doesn't replace it.")
                    .clicked()
                {
                    let folder = bake::bakes_folder();
                    compiler.note = match copy_folder(&folder.join(EZ_BAKE), &folder.join(&name)) {
                        Ok(()) => format!("Kept this world as \"{name}\"."),
                        Err(error) => format!("Couldn't keep it: {error}"),
                    };
                }
            }
            ui.separator();
            if let Some(result) = &compiler.result {
                ui.label(format!("land: {:.0}% of the world", result.made.land_share * 100.0));
                ui.small(format!("compiled in {:.0} ms", result.millis));
            } else if compiler.is_making() {
                ui.spinner();
            }
        });
    });
    let mut folded = compiler.folded;
    super::view::foldable_left(root, "world compiler settings", "Terrain Compiler", &mut folded[0], 320.0, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| settings(ui, compiler));
    });
    super::view::foldable_left(root, "world interpreter settings", "World Interpreter", &mut folded[1], 320.0, |ui| {
        egui::ScrollArea::vertical().id_salt("interpreter").show(ui, |ui| interpreter_panel(ui, compiler));
    });
    compiler.folded = folded;
    egui::Panel::right("world compiler readouts").resizable(true).default_size(270.0).size_range(180.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| readouts(ui, compiler));
    });
    let send = compiler.send.take();
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, compiler));
    if compiler.picking {
        spawn_picker(root.ctx(), compiler);
    }

    // EZ-Bake: the Tectonic Sim's world (a new one for Shift + Space) baked and read in here.
    if (new_world || same_world) && suite.compiler.ez.is_none() {
        if new_world {
            suite.world.seed = crate::fate::derive_seed(super::fresh_number(), "world seed");
        }
        let order = suite.world.bake_order(EZ_BAKE);
        suite.compiler.start_ez(order);
    }

    if let Some((name, picture, blocks_per_pixel)) = send {
        suite.add_layer(LayerKind::Image { file: format!("{name} (from the Terrain Compiler)"), picture: Arc::new(picture), placement: Placement { blocks_per_pixel, ..Placement::default() } });
        if let Some(layer) = suite.layers.last_mut() {
            layer.name = name;
        }
    }
}

fn settings(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("Terrain Compiler");
    ui.small(
        "Turns a baked world into smooth maps, doing the jobs that need the whole map at once. The Interpreter (the \
         next panel) makes the ground from them. Bake worlds in the Tectonic Sim, then pick one in the bar along the top.",
    );
    if !compiler.note.is_empty() {
        ui.small(&compiler.note);
    }
    ui.add_space(4.0);
    ui.strong("Presets");
    ui.small("Save every setting on both panels (the curves too) as a preset, to try on any bake.");
    preset_bar(ui, "compiler", &mut compiler.settings);
    let Some(bake) = compiler.bake.clone() else {
        ui.colored_label(PINK, "No bake read yet.");
        return;
    };
    let s = &mut compiler.settings;

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Sea level and the coastline");
    let (low, high) = (bake.info.height.black, bake.info.height.white);
    ui.add(crate::debug::Slider::new(&mut s.water, low..=high).text("bake height at sea level")).on_hover_text(
        "Which of the bake's heights becomes sea level (Z 128). Higher = more sea. Watch the land share in the bar along the top.",
    );
    ui.add(crate::debug::Slider::new(&mut s.soften, 0.0..=512.0).text("soften the coastline (blocks)")).on_hover_text(
        "The bake's coastline is the edge of a plate traced on a grid of pixels: a staircase of 32-block steps.\n\
         Softened a little, the Interpreter finds a smooth curve between the pixels instead, at every block.\n\
         The heights and tilt near the coast are softened as much, so nothing there keeps the steps.\n\
         About 64 blocks (two pixels) hides the steps; more rounds off small capes and inlets.\n\
         0 = none: the staircase shows (as rows of humps or steps along every coast).",
    );
    ui.add(crate::debug::Slider::new(&mut s.foothills, 0.0..=12_000.0).text("foothills (blocks)")).on_hover_text(
        "How far each range spreads a skirt of lower slopes around it, so it rises from the plains instead of\n\
         jumping up. Peaks aren't lowered. 0 = no foothills.\n\
         (Here rather than on the Interpreter: a skirt is the mountains blurred, which needs the whole map at once.)",
    );
    ui.add_space(4.0);
    defaults_row(ui, "Both panels' settings", &mut compiler.settings, &mut compiler.ab);

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Look");
    ui.add(crate::debug::Slider::new(&mut compiler.relief, 0.0..=10.0).text("relief shading")).on_hover_text(
        "Only changes the look: how strongly slopes are shaded (lit from the northwest).",
    );

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Save");
    let Some(result) = &compiler.result else { return };
    if ui
        .button("Save elevation into the bake")
        .on_hover_text(
            "Writes elevation.png (16-bit: the grey is the ground's Z, at each pixel's centre) and compile.toml\n\
             (these settings) into the bake's folder.",
        )
        .clicked()
    {
        compiler.note = match compiler::save(&bake::bakes_folder().join(&bake.name), &result.made, &compiler.settings) {
            Ok(()) => format!("Saved elevation.png into \"{}\".", bake.name),
            Err(error) => format!("Couldn't save: {error}"),
        };
    }
    if ui
        .button("Send elevation to the Graphics Kit")
        .on_hover_text(
            "The finished height map, as compiled with these settings, as a grey picture layer in the Graphics Kit:\n\
             black = Z 0, white = Z 399 (sea level Z 128 is a dark mid-grey).",
        )
        .clicked()
    {
        compiler.send = Some(("elevation".to_string(), elevation_picture(&result.made), bake.info.blocks_per_pixel));
        compiler.note = "Sent the elevation to the Graphics Kit as a layer.".to_string();
    }
    if ui.button("Send relief to the Graphics Kit").on_hover_text("The coloured, shaded map as you see it here.").clicked() {
        let width = result.made.width;
        let pixels = relief_picture(&result.made, bake.info.blocks_per_pixel, compiler.relief).into_iter().map(|[r, g, b]| [r, g, b, 255]).collect();
        compiler.send = Some(("relief".to_string(), Picture { width, height: width, pixels }, bake.info.blocks_per_pixel));
        compiler.note = "Sent the relief map to the Graphics Kit as a layer.".to_string();
    }

    ui.add_space(6.0);
    ui.separator();
    ui.heading("Render World and Spawn");
    ui.small(
        "Makes a real world from these maps and the Interpreter's recipe, and puts you where you pick. The world keeps \
         its own copy of both, so changing things here afterwards never changes it.",
    );
    ui.horizontal_wrapped(|ui| {
        ui.label("World name:");
        ui.add(egui::TextEdit::singleline(&mut compiler.world_name).desired_width(140.0).hint_text(bake.name.clone()));
    });
    if ui
        .button("Render World and Spawn")
        .on_hover_text("Opens a map of the world: click where you want to start. The world you're in now is replaced
(save it first if you want it).")
        .clicked()
    {
        compiler.picking = true;
    }
}

/// The signal chain, for the top of the Interpreter panel.
const CHAIN: &str = "\
Tectonic Sim -> bake (32-block pixels)
      |
Terrain Compiler · whole map, once:
  sea level · soften coast · foothills
      |  smooth maps
World Interpreter · every block, as
  the world is made (this panel):
  blend the maps to the block, then
  curves · grade · mountains · ridges
  passes · rifts · hills · detail
      |
the ground (and the distant terrain)";

/// The Interpreter's recipe: how the compiler's maps become the ground, worked out at every block.
fn interpreter_panel(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("World Interpreter");
    ui.small(
        "The end of the chain: the recipe that gives every column of blocks its height, worked out at that block as the \
         world is made. The map here shows it worked out at each pixel's centre.",
    );
    ui.add(egui::Label::new(egui::RichText::new(CHAIN).monospace().small())).on_hover_text(
        "Moved here from the Terrain Compiler: the compiler used to work this recipe out once per 32-block\n\
         pixel and hand over finished heights, which were then smoothed between pixels. The recipe has\n\
         sharp curves in it (a coast can drop 90 blocks within one pixel), and smoothing between finished\n\
         heights swings past every sharp step: one hump per pixel, in rows along every coast and cliff.\n\
         Now the smooth maps are blended to each block first, and the sharp curves applied after.",
    );
    if compiler.bake.is_none() {
        return;
    }
    let defaults = InterpreterSettings::default();
    let s = &mut compiler.settings.interpreter;

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Land and sea");
    ui.add(crate::debug::Slider::new(&mut s.deepest, 1.0..=127.0).text("deepest sea floor (Z)")).on_hover_text("The Z the deepest ocean reaches.");
    ui.add(crate::debug::Slider::new(&mut s.highlands, 129.0..=HIGHEST as f32).text("highlands (Z)"))
        .on_hover_text("The Z the highest land reaches before mountains are added (the top of the land curve).");
    ui.add_space(4.0);
    ui.label("Land: from the coast (left) to the bake's highest ground (right).");
    spline_box(ui, "land spline", &mut s.land, &defaults.land, "toward the highest ground", "up to the highlands");
    ui.label("Sea: from the coast (left) to the bake's deepest point (right).");
    spline_box(ui, "sea spline", &mut s.sea, &defaults.sea, "toward the deepest point", "down to the deepest floor");
    ui.small("A steep start to the sea curve makes cliffs at the coast; a gentle one, shallow shelves.");
    ui.add(crate::debug::Slider::new(&mut s.grade, 0.0..=1.0).text("grade by tilt")).on_hover_text(
        "Each plate's tilt steepens its high side (land climbs faster, the sea drops off close in) and softens\nits low side (wide plains, shallow shelves). 0 = no grading. (Bakes made before tilt was saved can't grade.)",
    );

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Mountains (from the squeeze map)");
    ui.small(
        "Across: how crushed the rock is, from none (left) to the most crushed rock in this bake (right). Up: how \
         tall the mountains are, as a share of the room up to Z 333. Keep the left part flat at the bottom to keep \
         lightly squeezed land as plains.",
    );
    spline_box(ui, "mountain spline", &mut s.mountains, &defaults.mountains, "more squeeze", "taller (top = Z 333)");
    // The ranges are centred on David's "hope" preset (2026-10-25), which had every one of these
    // pinned at the end of its old range.
    ui.add(crate::debug::Slider::new(&mut s.ridges, 0.0..=2.0).text("ridges")).on_hover_text(
        "How much the ridge pattern carves into the mountains. 0 = smooth domes and spines, 1 = all ridges and valleys;\n\
         past 1, the valleys between crests reach right down to the ground over wider stretches.\n\
         (Worked out at every block, so ridges are sharp down to single blocks.)",
    );
    ui.add(crate::debug::Slider::new(&mut s.ridge_spacing, 50.0..=3_200.0).logarithmic(true).text("ridge spacing (blocks)"))
        .on_hover_text("How far apart the ridges are, across them.");
    ui.add(crate::debug::Slider::new(&mut s.ridge_length, 250.0..=16_000.0).logarithmic(true).text("ridge length (blocks)")).on_hover_text(
        "How far ridges run along themselves. Ridges are lined up across the squeeze direction;\nlonger = long parallel ranges, shorter = broken, lumpy ones.\n\
         (Shorter than the spacing, they turn to run across the range instead of along it.)",
    );
    ui.add(crate::debug::Slider::new(&mut s.sharpness, 1.0..=16.0).logarithmic(true).text("ridge sharpness")).on_hover_text("Higher = narrower, sharper crests. Lower = broad, rounded ones.");
    ui.add(crate::debug::Slider::new(&mut s.ridge_layers, 1..=8).text("ridge layers")).on_hover_text(
        "How many layers of ridges are stacked, each about twice as fine as the last: 1 = one clean set of ridges,\n\
         more = smaller ridges branching off the big ones (spurs, gullies).",
    );
    ui.add(crate::debug::Slider::new(&mut s.ridge_roughness, 0.0..=1.0).text("ridge roughness")).on_hover_text(
        "How strong each finer layer of ridges is next to the one before. Low = clean, smooth-sided ridges; high = rough, broken ones.",
    );
    ui.add(crate::debug::Slider::new(&mut s.ridge_alignment, 0.0..=2.0).text("ridge alignment")).on_hover_text(
        "How readily ridges line up across the squeeze direction (see the 'squeeze direction' overlay).\n\
         0 = wandering every way, 1 = lined up wherever the squeeze is clear, 2 = lined up even where it's faint.",
    );
    ui.checkbox(&mut s.steady_ridges, "steady ridges").on_hover_text(
        "On: ridges keep their spacing where the squeeze direction bends (laid out in 12 fixed directions,\n\
         each spot blending the two nearest its squeeze).\n\
         Off: the old way, which turns the whole pattern to follow the squeeze and crushes ridges into\n\
         rows of thin grooves wherever it bends (the 'giant fingerprint'). For A/B.",
    );
    ui.add(crate::debug::Slider::new(&mut s.ridge_wander, 0.0..=2_000.0).text("ridge wander (blocks)")).on_hover_text(
        "How far ridges are nudged sideways by a slow swirl: they bend, change direction, and stop lining up with their\n\
         neighbours. 0 = straight. Around the ridge spacing and up bends them strongly.",
    );
    ui.add(crate::debug::Slider::new(&mut s.ridge_wander_size, 250.0..=16_000.0).logarithmic(true).text("wander size (blocks)"))
        .on_hover_text("How big the swirls that nudge the ridges are. Smaller = ridges kink more often.");
    ui.add(crate::debug::Slider::new(&mut s.passes, 0.0..=2.0).text("passes")).on_hover_text(
        "How deeply the crest height dips along each ridge, into passes between higher peaks. 0 = even crests;\n\
         past 1, the deepest passes cut right through to the ground, leaving separate peaks.",
    );
    ui.add(crate::debug::Slider::new(&mut s.pass_spacing, 375.0..=24_000.0).logarithmic(true).text("pass spacing (blocks)"))
        .on_hover_text("How far apart the dips are along a ridge.");

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Rifts, hills and detail");
    ui.add(crate::debug::Slider::new(&mut s.rift_depth, 0.0..=80.0).text("rift depth (blocks)"))
        .on_hover_text("How far the most stretched rock in this bake sinks (less stretched rock, less).");
    ui.add(crate::debug::Slider::new(&mut s.hills, 0.0..=25.0).text("hills (blocks)")).on_hover_text("Gentle rolling bumps on land, this high either way.");
    ui.add(crate::debug::Slider::new(&mut s.hill_size, 300.0..=8_000.0).logarithmic(true).text("hill size (blocks)"));
    ui.add(crate::debug::Slider::new(&mut s.detail, 0.0..=12.0).text("ground detail (blocks)")).on_hover_text(
        "Small bumps on the ground, this high either way, so it isn't glassy smooth. Only in the world:\nthey're smaller than one of this map's pixels.",
    );
    ui.add(crate::debug::Slider::new(&mut s.detail_size, 16.0..=512.0).logarithmic(true).text("detail size (blocks)"));

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Surface blocks");
    ui.add(crate::debug::Slider::new(&mut s.stone_from, SEA_LEVEL as i32..=HIGHEST as i32).text("bare stone from Z"))
        .on_hover_text("The surface is stone from this height up (mountainsides).");
    ui.add(crate::debug::Slider::new(&mut s.gabbro_from, SEA_LEVEL as i32..=HIGHEST as i32).text("gabbro from Z"))
        .on_hover_text("The surface is gabbro from this height up, so peaks stand out from a distance.");
    ui.add_space(4.0);
    ui.small("Presets, and back-to-defaults, are on the Terrain Compiler panel: they cover both.");
}

impl Compiler {
    /// "Render World and Spawn": the compiled maps, with the Interpreter's recipe, handed over to
    /// start a new world.
    pub fn press_render(&mut self, spawn: IVec2) {
        let (Some(bake), Some(result)) = (&self.bake, &self.result) else { return };
        let name = if self.world_name.trim().is_empty() { bake.name.clone() } else { self.world_name.clone() };
        let seed = u64::from_str_radix(&bake.info.seed, 16).unwrap_or(0);
        self.render = Some((name, seed, result.made.map.clone(), spawn));
        self.picking = false;
    }
}

/// The spawn picker: the world's relief map in a window; click where you want to start, or pick
/// the middle. Shows where the mouse is and how high the ground is there.
fn spawn_picker(ctx: &egui::Context, compiler: &mut Compiler) {
    let (Some(bake), Some(result), Some(texture)) = (compiler.bake.clone(), compiler.result.as_ref(), compiler.texture.clone()) else {
        compiler.picking = false;
        return;
    };
    let width = result.made.width;
    let blocks = bake.info.blocks_per_pixel;
    // The spot (north, east, in blocks) at a point on the map, from 0 to 1 across and down.
    let spot = |u: f32, v: f32| {
        let half = width as f32 / 2.0;
        IVec2::new(((half - v * width as f32) * blocks).round() as i32, ((u * width as f32 - half) * blocks).round() as i32)
    };
    let mut chosen = None;
    let mut open = true;
    egui::Window::new("Where to spawn")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("Click where you want to start.");
            let size = (ctx.content_rect().height() - 220.0).clamp(240.0, 720.0);
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click());
            let painter = ui.painter_at(rect);
            painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
            painter.circle_stroke(rect.center(), rect.width() / 2.0, egui::Stroke::new(1.0, MAGENTA));
            if let Some(pointer) = response.hover_pos() {
                let (u, v) = ((pointer.x - rect.left()) / rect.width(), (pointer.y - rect.top()) / rect.height());
                let at = spot(u, v);
                let (col, row) = ((u * width as f32) as usize, (v * width as f32) as usize);
                let z = result.made.z[row.min(width - 1) * width + col.min(width - 1)];
                let what = if z >= SEA_LEVEL { format!("land, Z {z}") } else { format!("sea, {} deep", SEA_LEVEL - z) };
                ui.label(format!("{}, {}: {what}", compass(at.x as f32, "N", "S"), compass(at.y as f32, "E", "W")));
                painter.circle_stroke(pointer, 6.0, egui::Stroke::new(2.0, Color32::WHITE));
                if response.clicked() {
                    chosen = Some(at);
                }
            } else {
                ui.label(" ");
            }
            ui.horizontal(|ui| {
                if ui.button("The middle").clicked() {
                    chosen = Some(IVec2::ZERO);
                }
                if ui.button("Cancel").clicked() {
                    compiler.picking = false;
                }
            });
        });
    if !open {
        compiler.picking = false;
    }
    if let Some(spawn) = chosen {
        compiler.press_render(spawn);
    }
}

/// The compiled world as a grey picture: black = Z 0, white = Z 399.
fn elevation_picture(compiled: &Compiled) -> Picture {
    let pixels = compiled
        .z
        .iter()
        .map(|&z| {
            let v = (z as f32 / TOP_OF_THE_WORLD as f32 * 255.0).round() as u8;
            [v, v, v, 255]
        })
        .collect();
    Picture { width: compiled.width, height: compiled.width, pixels }
}

/// A spline box: the curve over a grid, with its dots. Drag a dot to move it (the end dots stay
/// at the edges and only move up and down); double-click to add a dot; right-click a middle dot to
/// take it away.
fn spline_box(ui: &mut egui::Ui, name: &str, spline: &mut Spline, default: &Spline, across: &str, up: &str) {
    let size = Vec2::new(ui.available_width().clamp(160.0, 320.0), 140.0);
    let (rect, background) = ui.allocate_exact_size(size, egui::Sense::click());
    let inner = rect.shrink(10.0);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_gray(22));
    for k in 0..=4 {
        let t = k as f32 / 4.0;
        let stroke = egui::Stroke::new(1.0, Color32::from_gray(if k == 0 || k == 4 { 70 } else { 40 }));
        painter.line_segment([egui::pos2(inner.left() + t * inner.width(), inner.top()), egui::pos2(inner.left() + t * inner.width(), inner.bottom())], stroke);
        painter.line_segment([egui::pos2(inner.left(), inner.top() + t * inner.height()), egui::pos2(inner.right(), inner.top() + t * inner.height())], stroke);
    }
    let to_screen = |x: f32, y: f32| egui::pos2(inner.left() + x * inner.width(), inner.bottom() - y.clamp(0.0, 1.0) * inner.height());
    let from_screen = |p: egui::Pos2| (((p.x - inner.left()) / inner.width()).clamp(0.0, 1.0), ((inner.bottom() - p.y) / inner.height()).clamp(0.0, 1.0));
    let line: Vec<egui::Pos2> = (0..=64).map(|k| k as f32 / 64.0).map(|x| to_screen(x, spline.at(x))).collect();
    painter.add(egui::Shape::line(line, egui::Stroke::new(2.0, Color32::from_rgb(255, 230, 120))));
    painter.text(rect.left_bottom() + Vec2::new(12.0, -12.0), egui::Align2::LEFT_BOTTOM, across, egui::FontId::proportional(10.0), Color32::from_gray(140));
    painter.text(rect.left_top() + Vec2::new(12.0, 12.0), egui::Align2::LEFT_TOP, up, egui::FontId::proportional(10.0), Color32::from_gray(140));

    // The dots.
    let last = spline.dots.len() - 1;
    let mut remove = None;
    for k in 0..spline.dots.len() {
        let (x, y) = spline.dots[k];
        let at = to_screen(x, y);
        let response = ui.interact(egui::Rect::from_center_size(at, Vec2::splat(16.0)), ui.id().with((name, k)), egui::Sense::click_and_drag());
        let lit = response.hovered() || response.dragged();
        painter.circle(at, if lit { 6.5 } else { 5.0 }, if lit { Color32::WHITE } else { Color32::from_rgb(255, 230, 120) }, egui::Stroke::new(1.5, Color32::BLACK));
        if response.dragged()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let (px, py) = from_screen(pointer);
            // The end dots stay at the edges; a middle dot stays between its neighbours.
            let x = if k == 0 || k == last { x } else { px.clamp(spline.dots[k - 1].0 + 0.01, spline.dots[k + 1].0 - 0.01) };
            spline.dots[k] = (x, py);
        }
        if response.secondary_clicked() && k != 0 && k != last {
            remove = Some(k);
        }
        if lit {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
    }
    if let Some(k) = remove {
        spline.dots.remove(k);
    }
    // Double-click to add a dot where you clicked (not right on top of another).
    if background.double_clicked()
        && let Some(pointer) = background.interact_pointer_pos()
    {
        let (x, y) = from_screen(pointer);
        if spline.dots.iter().all(|&(dx, _)| (dx - x).abs() > 0.02) {
            let k = spline.dots.iter().position(|&(dx, _)| dx > x).unwrap_or(spline.dots.len());
            spline.dots.insert(k, (x, y));
        }
    }
    ui.horizontal_wrapped(|ui| {
        ui.small(format!("{} dots: double-click to add, drag to move, right-click to remove", spline.dots.len()));
        if ui.small_button("reset").clicked() {
            *spline = default.clone();
        }
    });
}

fn readouts(ui: &mut egui::Ui, compiler: &mut Compiler) {
    lab_readouts(ui, compiler);
    ui.separator();
    ui.heading("Under the mouse");
    let (Some(bake), Some(result)) = (&compiler.bake, &compiler.result) else {
        ui.small("Nothing compiled yet.");
        return;
    };
    let width = result.made.width;
    if let Some((north, east, z)) = compiler.lab_hover {
        // In the close-up: the very block.
        ui.label(format!("{}, {}", compass(north, "N", "S"), compass(east, "E", "W")));
        ui.heading(format!("Z {z:.0}"));
        if z >= SEA_LEVEL as f32 {
            ui.label(format!("land, {:.0} blocks above the sea", z.round() - SEA_LEVEL as f32));
        } else {
            ui.label(format!("sea, {:.0} blocks deep", SEA_LEVEL as f32 - z.round()));
        }
        ui.small("(this block, as the Interpreter makes it)");
    }
    match compiler.hover {
        Some((col, row)) if col < width && row < width => {
            let i = row * width + col;
            if compiler.lab_hover.is_none() {
                let blocks = bake.info.blocks_per_pixel;
                let half = width as f32 / 2.0;
                ui.label(format!("{}, {}", compass((half - row as f32 - 0.5) * blocks, "N", "S"), compass((col as f32 + 0.5 - half) * blocks, "E", "W")));
                let z = result.made.z[i];
                ui.heading(format!("Z {z}"));
                if z >= SEA_LEVEL {
                    ui.label(format!("land, {} blocks above the sea", z - SEA_LEVEL));
                } else {
                    ui.label(format!("sea, {} blocks deep", SEA_LEVEL - z));
                }
            }
            ui.separator();
            ui.small("From the bake (this pixel):");
            ui.small(format!("height {:.3}", bake.height[i]));
            ui.small(format!("squeeze {:+.2}", bake.squeeze[i]));
            if bake.push_strength[i] > 0.02 {
                ui.small(format!("squeezed at {:.0}°, strength {:.2}", bake.push_angle[i], bake.push_strength[i]));
            }
            ui.small(if bake.plate[i] == 0 { "no plate".to_string() } else { format!("plate grey {}", bake.plate[i]) });
            if let Some(painted) = &compiler.lab.overlays.painted
                && painted.width == width
            {
                ui.small("From the overlays (this pixel):");
                let d = painted.fields.coast_distance[i];
                ui.small(if d >= 0.0 { format!("{d:.0} blocks inland") } else { format!("{:.0} blocks out to sea", -d) });
                ui.small(format!("mountain share {:.0}%", painted.share[i] * 100.0))
                    .on_hover_text("How much of the room above the ground the mountains take here, before ridges and passes carve them.");
                let skirt = result.made.map.maps.skirt[i];
                if skirt > 0.0 {
                    ui.small(format!("foothill skirt {skirt:.0} blocks"));
                }
            }
        }
        _ => {
            ui.small("Point at the world.");
        }
    }
    ui.separator();
    ui.heading("The world");
    let highest = result.made.z.iter().copied().max().unwrap_or(0);
    let lowest = result.made.z.iter().copied().min().unwrap_or(0);
    ui.label(format!("land: {:.1}% of the world", result.made.land_share * 100.0));
    ui.label(format!("highest ground Z {highest}, deepest sea floor Z {lowest}"));
    ui.small(format!("Sea level Z {SEA_LEVEL}; nothing past Z {HIGHEST}; the world's top is Z {TOP_OF_THE_WORLD}."));
}

/// The terrain lab's part of the right-hand panel: what the close-up is showing, what it cost, the
/// layer switches, and the cross-section.
fn lab_readouts(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("Terrain lab");
    let close = compiler.camera.zoom >= lab::TAKES_OVER_AT;
    if !close {
        ui.small("Zoom in on the map (scroll) and it becomes a live close-up of the ground, block by block, as the Interpreter makes it. Move the Interpreter's sliders while you watch.");
    } else if let Some(frame) = &compiler.lab.frame {
        let r = &frame.request;
        let columns = r.cols * r.rows;
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        let per_chunk = frame.millis * cores as f64 / columns.max(1) as f64 * 1024.0;
        let every = if r.step <= 1.0 { "every block".to_string() } else { format!("every {} blocks", r.step) };
        ui.label(format!("Close-up: {every}, {} × {} blocks", r.cols as f32 * r.step, r.rows as f32 * r.step));
        ui.small(format!("drawn in {:.0} ms: {columns} columns on {cores} cores", frame.millis))
            .on_hover_text(format!(
                "About {per_chunk:.2} ms per chunk (1,024 columns) on one core: what making a chunk's ground costs\n\
                 with this recipe. Streaming makes chunks in the background, on all cores."
            ));
    } else {
        ui.small("Drawing the close-up…");
    }
    ui.add_space(4.0);
    ui.label("Show:").on_hover_text("Leave parts of the Interpreter's recipe out, to see what each one does. Only here: worlds always use the whole recipe.");
    let m = &mut compiler.lab.mutes;
    ui.horizontal_wrapped(|ui| {
        for (name, muted, about) in [
            ("mountains", &mut m.mountains, "The mountains from the squeeze map (with their ridges, passes and foothill skirts)."),
            ("ridges", &mut m.ridges, "The ridge pattern carved into the mountains (off: smooth domes)."),
            ("passes", &mut m.passes, "The dips along each ridge's crest (off: even crests)."),
            ("rifts", &mut m.rifts, "Stretched rock sinking."),
            ("hills", &mut m.hills, "The rolling hills on land."),
            ("detail", &mut m.detail, "The small block-sized bumps."),
        ] {
            let mut shown = !*muted;
            if ui.checkbox(&mut shown, name).on_hover_text(about).changed() {
                *muted = !shown;
            }
        }
    });
    ui.add_space(4.0);
    ui.label("Overlays:").on_hover_text(
        "The compiler's maps drawn over the map and the close-up, one value per map pixel (32 blocks): the areas\n\
         and lines hidden in them. Later, these are what the scatterers (second-layer mountains) will be driven by.",
    );
    let o = &mut compiler.lab.overlays;
    ui.horizontal_wrapped(|ui| {
        let s = &mut o.shown;
        for (name, shown, about) in [
            ("plates", &mut s.plates, "Each plate in its own colour, with its outline."),
            ("squeeze", &mut s.squeeze, "Crushed rock in red, stretched rock in blue: the stronger, the deeper the colour."),
            ("mountains", &mut s.mountains, "Where mountains rise (orange: the deeper, the taller the mountains curve makes them), and their foothill skirts (yellow)."),
            ("range spines", &mut s.spines, "The crest line of each mountain range: where its mountains are strongest, across the range."),
            ("coast distance", &mut s.coast, "The coastline (cyan), and lines every so many blocks inland (white) and out to sea (pale blue)."),
            ("squeeze direction", &mut s.direction, "Which way the rock was squeezed: longer strokes, harder squeezing. Ridges line up across the strokes."),
        ] {
            ui.checkbox(shown, name).on_hover_text(about);
        }
    });
    let any = o.shown != super::overlays::Shown::default();
    if any {
        ui.add(crate::debug::Slider::new(&mut o.opacity, 0.1..=1.0).text("overlay strength")).on_hover_text("How strongly the overlays cover the map.");
    }
    if o.shown.coast {
        ui.add(crate::debug::Slider::new(&mut o.band, 250.0..=8_000.0).text("coast lines every (blocks)")).on_hover_text("The blocks between the coast-distance lines.");
    }
    ui.add_space(4.0);
    ui.label("Cross-section:");
    if compiler.lab.section.is_some() {
        if ui.small_button("Clear the cross-section").on_hover_text("Or right-click the map.").clicked() {
            compiler.lab.section = None;
        }
    } else {
        ui.small("Shift + drag across the map to draw one: its side view shows under the map, live.");
    }
}

/// A close-up's picture: coloured by height, slopes lit from the northwest; past the map is
/// `BEYOND`.
fn close_up_picture(frame: &lab::Frame, relief: f32) -> Vec<Color32> {
    let (cols, rows, step) = (frame.request.cols, frame.request.rows, frame.request.step);
    let z = |col: usize, row: usize| frame.heights[row.min(rows - 1) * cols + col.min(cols - 1)];
    let light = {
        let (x, y, up) = (-1.0f32, -1.0f32, 1.4f32);
        let length = (x * x + y * y + up * up).sqrt();
        (x / length, y / length, up / length)
    };
    // Slopes are real slopes here (blocks up per block across); the relief slider exaggerates them
    // as it does the map's (5 = as they are).
    let exaggerate = relief / 5.0;
    (0..cols * rows)
        .map(|i| {
            let (col, row) = (i % cols, i / cols);
            let here = z(col, row);
            if here.is_nan() {
                let [r, g, b] = BEYOND;
                return Color32::from_rgb(r, g, b);
            }
            let colour = colour_at(here);
            if here < SEA_LEVEL as f32 {
                return Color32::from_rgb(colour[0], colour[1], colour[2]);
            }
            let or_here = |v: f32| if v.is_nan() { here } else { v };
            let east = (or_here(z(col + 1, row)) - or_here(z(col.saturating_sub(1), row))) / (2.0 * step) * exaggerate;
            let south = (or_here(z(col, row + 1)) - or_here(z(col, row.saturating_sub(1)))) / (2.0 * step) * exaggerate;
            let length = (east * east + south * south + 1.0).sqrt();
            let lit = (-east * light.0 - south * light.1 + light.2) / length;
            let shade = (lit / light.2).clamp(0.35, 1.5);
            let [r, g, b] = colour.map(|c| (c as f32 * shade).min(255.0) as u8);
            Color32::from_rgb(r, g, b)
        })
        .collect()
}

fn canvas(ui: &mut egui::Ui, compiler: &mut Compiler) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    compiler.lab_hover = None;
    let (Some(bake), Some(result)) = (compiler.bake.clone(), &compiler.result) else {
        let message = if compiler.ez.is_some() {
            "EZ-Baking a new world…"
        } else if compiler.is_making() || compiler.bake.is_some() {
            "Compiling…"
        } else {
            "No bake yet. Press EZ-Bake (Space), or bake a world in the Tectonic Sim."
        };
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, message, egui::FontId::proportional(14.0), PINK);
        return;
    };
    let width = result.made.width;
    let compiled_map = result.made.map.clone();
    let wanted = (compiler.version, compiler.relief.to_bits());
    if compiler.painted_for != Some(wanted) || compiler.texture.is_none() {
        let pixels: Vec<Color32> = relief_picture(&result.made, bake.info.blocks_per_pixel, compiler.relief).into_iter().map(|[r, g, b]| Color32::from_rgb(r, g, b)).collect();
        let image = egui::ColorImage::new([width, width], pixels);
        // Made tiny first, then given the picture (egui's first-make size check is too cautious).
        let texture = compiler.texture.get_or_insert_with(|| ui.ctx().load_texture("world compiler", egui::ColorImage::new([1, 1], vec![Color32::BLACK]), egui::TextureOptions::NEAREST));
        texture.set(image, egui::TextureOptions::NEAREST);
        compiler.painted_for = Some(wanted);
    }
    // Shift + drag draws a cross-section instead of moving the map.
    let shift = ui.input(|input| input.modifiers.shift);
    compiler.camera.hold_pan = shift;
    compiler.hover = compiler.camera.update(ui, rect, &response, (width, width));
    if let Some(texture) = &compiler.texture {
        let field = compiler.camera.field_rect(rect, (width, width));
        painter.image(texture.id(), field, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
        painter.circle_stroke(field.center(), field.width() / 2.0, egui::Stroke::new(1.0, MAGENTA));
    }

    // Blocks (north, east) and the screen, both ways.
    let (centre, zoom, blocks, half) = (compiler.camera.centre, compiler.camera.zoom, bake.info.blocks_per_pixel, width as f32 / 2.0);
    let to_blocks = |pos: egui::Pos2| {
        let p = centre + (pos - rect.center()) / zoom;
        ((half - p.y) * blocks, (p.x - half) * blocks)
    };
    let to_screen = |(north, east): (f32, f32)| rect.center() + (Vec2::new(east / blocks + half, half - north / blocks) - centre) * zoom;

    // The terrain lab's close-up, once zoomed in far enough.
    let map = compiler.lab.recipe(&compiled_map, &compiler.settings.interpreter);
    let close = zoom >= lab::TAKES_OVER_AT;
    let view = close.then(|| (to_blocks(rect.left_top()), to_blocks(rect.right_bottom()), blocks / zoom));
    compiler.lab.drive(&map, view);
    if close {
        if let Some(frame) = &compiler.lab.frame
            && !frame.heights.is_empty()
        {
            let wanted = (compiler.lab.version, compiler.relief.to_bits());
            if compiler.lab_painted_for != Some(wanted) || compiler.lab_texture.is_none() {
                let image = egui::ColorImage::new([frame.request.cols, frame.request.rows], close_up_picture(frame, compiler.relief));
                let texture = compiler.lab_texture.get_or_insert_with(|| ui.ctx().load_texture("terrain lab", egui::ColorImage::new([1, 1], vec![Color32::BLACK]), egui::TextureOptions::NEAREST));
                texture.set(image, egui::TextureOptions::NEAREST);
                compiler.lab_painted_for = Some(wanted);
            }
            if let Some(texture) = &compiler.lab_texture {
                let r = &frame.request;
                let corner = to_screen((r.north, r.east));
                let far = to_screen((r.north - r.rows as f32 * r.step, r.east + r.cols as f32 * r.step));
                painter.image(texture.id(), egui::Rect::from_min_max(corner, far), egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
            }
        }
        if let Some(mouse) = response.hover_pos() {
            let (north, east) = to_blocks(mouse);
            if let Some(z) = map.height(north, east) {
                compiler.lab_hover = Some((north, east, z));
            }
        }
    } else {
        compiler.camera.outline(&painter, rect, compiler.hover, MAGENTA);
    }

    // The overlays, over the map and the close-up alike.
    compiler.lab.overlays.drive(&map, &bake);
    let overlays = &compiler.lab.overlays;
    if overlays.shown != super::overlays::Shown::default()
        && let Some(painted) = &overlays.painted
        && painted.width == width
    {
        let tint = Color32::WHITE.gamma_multiply(overlays.opacity);
        if compiler.overlay_painted_for != Some(overlays.version) || compiler.overlay_texture.is_none() {
            let image = egui::ColorImage::new([width, width], painted.pixels.clone());
            let texture = compiler.overlay_texture.get_or_insert_with(|| ui.ctx().load_texture("terrain lab overlays", egui::ColorImage::new([1, 1], vec![Color32::TRANSPARENT]), egui::TextureOptions::LINEAR));
            texture.set(image, egui::TextureOptions::LINEAR);
            compiler.overlay_painted_for = Some(overlays.version);
        }
        if let Some(texture) = &compiler.overlay_texture {
            let field = compiler.camera.field_rect(rect, (width, width));
            painter.image(texture.id(), field, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), tint);
        }
        if overlays.shown.direction {
            squeeze_strokes(&painter, rect, &compiler.camera, &compiled_map.maps, painted.strongest_push, tint);
        }
    }

    // The cross-section: Shift + drag to draw, right-click to clear.
    if let Some(mouse) = response.interact_pointer_pos().or(response.hover_pos()) {
        if shift && response.drag_started() {
            compiler.lab.drawing_from = Some(to_blocks(mouse));
        }
        if let Some(from) = compiler.lab.drawing_from {
            compiler.lab.section = Some((from, to_blocks(mouse)));
        }
    }
    if response.drag_stopped() || !shift {
        compiler.lab.drawing_from = None;
    }
    if response.secondary_clicked() {
        compiler.lab.section = None;
    }
    if let Some((from, to)) = compiler.lab.section {
        let (a, b) = (to_screen(from), to_screen(to));
        painter.line_segment([a, b], egui::Stroke::new(2.0, PINK));
        painter.circle_filled(a, 4.0, PINK);
        painter.circle_filled(b, 4.0, MAGENTA);
        side_view(&painter, rect, &map, from, to);
    }

    let help = if close {
        "close-up: every column as the Interpreter makes it   drag = move   scroll = zoom   Shift + drag = cross-section   double-click = fit"
    } else {
        "drag = move   scroll = zoom (in close for the live close-up)   Shift + drag = cross-section   double-click = fit   north is up"
    };
    painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), egui::Align2::LEFT_BOTTOM, help, egui::FontId::proportional(12.0), PINK);
}

/// The squeeze direction overlay: a short stroke every so many screen points, along the way the rock
/// was squeezed there, longer where it was squeezed harder.
fn squeeze_strokes(painter: &egui::Painter, rect: egui::Rect, camera: &Camera, maps: &crate::world::Maps, strongest: f32, tint: Color32) {
    const EVERY: f32 = 28.0;
    let width = maps.width;
    let stroke = egui::Stroke::new(1.5, Color32::WHITE.gamma_multiply(tint.a() as f32 / 255.0));
    let (across, down) = ((rect.width() / EVERY) as usize + 1, (rect.height() / EVERY) as usize + 1);
    for row in 0..down {
        for col in 0..across {
            let at = rect.left_top() + Vec2::new((col as f32 + 0.5) * EVERY, (row as f32 + 0.5) * EVERY);
            let pixel = camera.centre + (at - rect.center()) / camera.zoom;
            if pixel.x < 0.0 || pixel.y < 0.0 || pixel.x >= width as f32 || pixel.y >= width as f32 {
                continue;
            }
            let i = pixel.y as usize * width + pixel.x as usize;
            let strength = maps.push_strength[i] / strongest.max(1e-6);
            if strength < 0.05 {
                continue;
            }
            // The doubled-angle arrow back to a direction (on screen, down is south).
            let angle = maps.push_y[i].atan2(maps.push_x[i]) / 2.0;
            let half = Vec2::new(angle.cos(), -angle.sin()) * (3.0 + 9.0 * strength.sqrt());
            painter.line_segment([at - half, at + half], stroke);
        }
    }
}

/// The cross-section's side view, along the bottom of the map: the ground's Z from the pink end to
/// the magenta one, with sea level.
fn side_view(painter: &egui::Painter, rect: egui::Rect, map: &crate::world::CompiledMap, from: (f32, f32), to: (f32, f32)) {
    let line = lab::profile(map, from, to, 1_000);
    if line.len() < 2 {
        return;
    }
    let area = egui::Rect::from_min_max(egui::Pos2::new(rect.left() + 8.0, rect.bottom() - 200.0), egui::Pos2::new(rect.right() - 8.0, rect.bottom() - 28.0));
    painter.rect_filled(area, 4.0, Color32::from_black_alpha(215));
    let length = line.last().map_or(1.0, |&(along, _)| along).max(1.0);
    let sea = SEA_LEVEL as f32;
    let (mut low, mut high) = line.iter().fold((sea, sea), |(lo, hi), &(_, z)| (lo.min(z), hi.max(z)));
    let pad = ((high - low) * 0.08).max(4.0);
    (low, high) = (low - pad, high + pad);
    let inner = area.shrink2(Vec2::new(8.0, 18.0));
    let at = |along: f32, z: f32| egui::Pos2::new(inner.left() + along / length * inner.width(), inner.bottom() - (z - low) / (high - low) * inner.height());
    // Sea level, then the ground (filled below, so its shape reads at a glance).
    let sea_y = at(0.0, sea).y;
    painter.line_segment([egui::Pos2::new(inner.left(), sea_y), egui::Pos2::new(inner.right(), sea_y)], egui::Stroke::new(1.0, Color32::from_rgb(70, 140, 220)));
    for pair in line.windows(2) {
        let (a, b) = (at(pair[0].0, pair[0].1), at(pair[1].0, pair[1].1));
        let base = inner.bottom();
        let fill = if pair[0].1 < sea { Color32::from_rgb(25, 60, 115) } else { Color32::from_rgb(70, 90, 60) };
        painter.add(egui::Shape::convex_polygon(vec![a, b, egui::Pos2::new(b.x, base), egui::Pos2::new(a.x, base)], fill, egui::Stroke::NONE));
    }
    let points: Vec<egui::Pos2> = line.iter().map(|&(along, z)| at(along, z)).collect();
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, Color32::WHITE)));
    let font = egui::FontId::proportional(11.0);
    painter.text(area.left_top() + Vec2::new(8.0, 3.0), egui::Align2::LEFT_TOP, format!("cross-section: {length:.0} blocks, Z {:.0} to {:.0}  (sea level Z {SEA_LEVEL})", low + pad, high - pad), font.clone(), PINK);
    painter.text(area.right_bottom() + Vec2::new(-8.0, -3.0), egui::Align2::RIGHT_BOTTOM, "right-click to clear", font, PINK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_get_lighter_from_the_deep_to_the_peaks_and_jump_at_the_coast() {
        let brightness = |z: f32| colour_at(z).iter().map(|&c| c as u32).sum::<u32>();
        assert!(brightness(10.0) < brightness(120.0), "deep sea darker than shallow");
        assert!(brightness(300.0) < brightness(333.0), "peaks lightest");
        // The coast is a clear change: blue below, green above.
        let (sea, land) = (colour_at(127.0), colour_at(128.0));
        assert!(sea[2] > sea[1] && land[1] > land[2]);
    }
}

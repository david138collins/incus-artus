//! The World Compiler's screen: pick a bake, set the compiler's whole-map jobs, the topography (the
//! additives, in code), the climate, the mineralogy and the Interpreter's recipe, and see the result
//! as the world's blocks. (The work itself is in `compiler.rs`, `additives.rs`, `climate.rs` and
//! `crate::world::interpreter`.)
//!
//! ```text
//! ┌──── top: panels shown, which bake, land share ──────────────────────────────────────────────────────────┐
//! │ Landmass     │ Topography   │ Climate      │ Mineralogy │ World Interpreter │                │ under the │
//! │ sea level    │ land and sea │ rivers       │ the blocks │ the chain, and    │ the world, in  │ mouse     │
//! │ coastline    │ ridgelines   │ temperature  │ underfoot  │ where each part   │ blocks         │           │
//! │ soften       │ mountains    │ wind, rain   │ (caves     │ is set            │                │           │
//! │ save, render │ hills, torn  │              │  to come)  │                   │                │           │
//! │ presets      │ coast        │              │            │                   │                │           │
//! └──────────────┴──────────────┴──────────────┴────────────┴───────────────────┴────────────────┴───────────┘
//! ```

use super::Suite;
use super::additives::{Additives, DriftWay, Sides};
use super::bake::{self, Baked};
use super::canvas::Camera;
use super::compiler::{self, Bake, CompileSettings, Compiled, HIGHEST, SEA_LEVEL, Spline, TOP_OF_THE_WORLD};
use crate::world::{InterpreterSettings, Lift, LiftFrom, Mask, MaskSource, Mutes, Peaks};
use super::noise::PerlinSettings;
use super::image::{Picture, Placement};
use super::layers::LayerKind;
use super::runner::{Finished, Job, Poll};
use super::lab::{self, Lab};
use super::view::{MAGENTA, PINK, compass, fold_section, switched_section};
use super::world::defaults_row;
use crate::debug::{default_light, default_preset, preset_bar, save_shortcut};
use bevy::math::IVec2;
use bevy_egui::egui::{self, Color32, Vec2};
use std::sync::Arc;

/// Past the edge of the world.
const BEYOND: [u8; 3] = [10, 10, 14];
/// The scratch bake EZ-Bake writes (and overwrites) each time.
const EZ_BAKE: &str = "_ez-bake";
/// The left-hand panels, in the order things happen [David, 2026-10-31: "Tabs by stage",
/// docs/worldgen.md; the Topography panel was the Additives (its settings keep that name in code and
/// presets), and Mineralogy moved out of Climate: "minerology is gonna need a lot of space"].
const PANELS: [&str; 5] = ["Landmass", "Topography", "Climate", "Mineralogy", "World Interpreter"];

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
    /// Picking where to start, on the map (after "Render World and Spawn"): the next click there
    /// starts the world; and Space was pressed again while picking (it picks the spot under the mouse).
    picking: bool,
    space_pick: bool,
    /// The name to give a rendered world (blank = the bake's name).
    world_name: String,
    camera: Camera,
    /// The terrain lab (the close-up, its layer switches and cross-section), its picture, and the
    /// block under the mouse in it (north, east, Z).
    pub lab: Lab,
    lab_texture: Option<egui::TextureHandle>,
    lab_painted_for: Option<PaintKey>,
    /// The close-up shown unlit (flat colours), and the far colours of every block (rebuilt when the
    /// blocks change).
    pub lab_unlit: bool,
    /// The close-up's shores coloured by how flush they are, and its rivers by their water across (the
    /// Carving section).
    pub lab_shores: bool,
    pub lab_sizes: bool,
    /// The placer's field view (the Mineralogy panel's Fields section).
    pub fields: super::placer_view::FieldView,
    lab_palette: Option<(u64, Arc<crate::render::distant::Palette>)>,
    /// The noise maths' timings, once asked for (the World Interpreter panel).
    noise_timing: Option<String>,
    /// The view's map with the rivers carved in, and what it was made from (the recipe's map and the
    /// carving, and whether the Show switch leaves them out).
    rivers_view: Option<((usize, usize, bool), Arc<crate::world::CompiledMap>)>,
    lab_hover: Option<(f32, f32, f32)>,
    /// The overlays' picture, and which of their pictures it holds.
    overlay_texture: Option<egui::TextureHandle>,
    overlay_painted_for: Option<u64>,
    hover: Option<(usize, usize)>,
    /// The Climate stage's work (in the background), and its picture over the map.
    pub climate: super::climate::Climate,
    climate_texture: Option<egui::TextureHandle>,
    climate_painted_for: Option<u64>,
    /// How many map pixels across the climate picture covers.
    climate_painted_size: usize,
    /// The Topography panel's dots.
    pub additives: Additives,
    /// Whether the lift's points are drawn over the map.
    pub show_lift_points: bool,
    /// Where you were in the world when you opened the Editor Suite (F2): north, east (blocks) and
    /// the compass bearing you faced (degrees), drawn on the map. [David, 2026-10-27]
    pub came_from: Option<(f32, f32, f32)>,
    /// The Landmass, Topography, Climate, Mineralogy and World Interpreter panels: shown (the buttons
    /// along the top), and folded away to thin strips (they start folded, so the map has the room).
    pub shown: [bool; 5],
    pub folded: [bool; 5],
    /// The readouts panel on the right, folded away to a thin strip.
    pub readouts_folded: bool,
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
            space_pick: false,
            world_name: String::new(),
            camera: Camera::default(),
            lab: Lab::default(),
            lab_texture: None,
            lab_painted_for: None,
            lab_unlit: false,
            lab_shores: false,
            lab_sizes: false,
            fields: Default::default(),
            lab_palette: None,
            noise_timing: None,
            rivers_view: None,
            lab_hover: None,
            overlay_texture: None,
            overlay_painted_for: None,
            climate: Default::default(),
            climate_texture: None,
            climate_painted_for: None,
            climate_painted_size: 0,
            hover: None,
            additives: Additives::default(),
            show_lift_points: false,
            came_from: None,
            shown: [true; 5],
            folded: [true; 5],
            readouts_folded: true,
        }
    }
}

impl Compiler {
    /// Starts from the "default" preset, if one has been saved, and keeps it when the first bake is
    /// read: a bake otherwise sets the sea level to the one it was baked with, which at start-up
    /// quietly undid the default's (found 2026-10-27: David's coasts and lift came out different until
    /// he loaded "default" by hand).
    pub(super) fn use_default_preset(&mut self) {
        if let Some(settings) = default_preset("compiler") {
            self.settings = settings;
            self.keep_settings = true;
        }
    }

    pub fn is_making(&self) -> bool {
        self.job.is_some() || self.ez.is_some()
    }

    /// A compiled world is ready to render.
    pub fn is_compiled(&self) -> bool {
        self.result.is_some() && self.job.is_none()
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
        } else if self.climate.is_making() {
            Some("reading the ground for the climate")
        } else if self.additives.is_making() {
            Some("finding range spines")
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
        // The additives and climate aren't compiled (moving their sliders compiles nothing again).
        let now = (chosen, CompileSettings { additives: Default::default(), climate: Default::default(), ..self.settings.clone() });
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

pub fn draw(root: &mut egui::Ui, suite: &mut Suite, look: &Look) {
    // Space: Render World and Spawn (opens the spawn picker) [David, 2026-10-27: EZ-Bake is pressed
    // once after changing pages, so it lost the space bar]. Shift + Space: EZ-Bake a new world (a
    // new seed). Not while typing, say a preset's name.
    let typing = root.ctx().egui_wants_keyboard_input();
    let mut new_world = !typing && root.input_mut(|input| input.consume_key(egui::Modifiers::SHIFT, egui::Key::Space));
    let mut same_world = false;
    let compiler = &mut suite.compiler;
    if !typing && compiler.result.is_some() && root.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Space)) {
        // The first press starts picking on the map; a second picks the spot under the mouse.
        if compiler.picking {
            compiler.space_pick = true;
        } else {
            compiler.picking = true;
        }
    }
    save_shortcut(root.ctx(), "compiler", || compiler.settings.clone());
    egui::Panel::top("world compiler bar").show(root, |ui| {
        ui.horizontal_wrapped(|ui| {
            default_light(ui, "compiler", &compiler.settings);
            ui.separator();
            for (shown, name) in compiler.shown.iter_mut().zip(PANELS) {
                ui.toggle_value(shown, name).on_hover_text(format!("Show or hide the {name} panel"));
            }
            ui.separator();
            if compiler.ez.is_some() {
                ui.spinner();
                ui.label("EZ-Baking…");
            } else {
                if ui
                    .button("EZ-Bake")
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
    let (mut folded, shown) = (compiler.folded, compiler.shown);
    if shown[0] {
        super::view::foldable_left(root, "world compiler settings", PANELS[0], &mut folded[0], 320.0, |ui| {
            super::notes::on_page(Some(PANELS[0]));
            egui::ScrollArea::vertical().show(ui, |ui| settings(ui, compiler));
        });
    }
    if shown[1] {
        super::view::foldable_left(root, "additives settings", PANELS[1], &mut folded[1], 300.0, |ui| {
            super::notes::on_page(Some(PANELS[1]));
            egui::ScrollArea::vertical().id_salt("additives").show(ui, |ui| additives_panel(ui, compiler));
        });
    }
    if shown[2] {
        super::view::foldable_left(root, "climate settings", PANELS[2], &mut folded[2], 300.0, |ui| {
            super::notes::on_page(Some(PANELS[2]));
            egui::ScrollArea::vertical().id_salt("climate").show(ui, |ui| climate_panel(ui, compiler));
        });
    }
    if shown[3] {
        super::view::foldable_left(root, "mineralogy settings", PANELS[3], &mut folded[3], 300.0, |ui| {
            super::notes::on_page(Some(PANELS[3]));
            egui::ScrollArea::vertical().id_salt("mineralogy").show(ui, |ui| mineralogy_panel(ui, compiler, look));
        });
    }
    if shown[4] {
        super::view::foldable_left(root, "world interpreter settings", PANELS[4], &mut folded[4], 320.0, |ui| {
            super::notes::on_page(Some(PANELS[4]));
            egui::ScrollArea::vertical().id_salt("interpreter").show(ui, |ui| interpreter_panel(ui, compiler));
        });
    }
    compiler.folded = folded;
    let mut readouts_folded = compiler.readouts_folded;
    super::view::foldable_right(root, "world compiler readouts", "readouts", &mut readouts_folded, 270.0, |ui| {
        super::notes::on_page(Some("World Compiler readouts"));
        egui::ScrollArea::vertical().show(ui, |ui| readouts(ui, compiler));
    });
    compiler.readouts_folded = readouts_folded;
    let send = compiler.send.take();
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, compiler, look));

    // EZ-Bake: the Tectonic Sim's world (a new one for Shift + Space) baked and read in here.
    if (new_world || same_world) && suite.compiler.ez.is_none() {
        if new_world {
            suite.world.seed = crate::fate::derive_seed(super::fresh_number(), "world seed");
        }
        let order = suite.world.bake_order(EZ_BAKE);
        suite.compiler.start_ez(order);
    }

    if let Some((name, picture, blocks_per_pixel)) = send {
        suite.add_layer(LayerKind::Image { file: format!("{name} (from the World Compiler)"), picture: Arc::new(picture), placement: Placement { blocks_per_pixel, ..Placement::default() } });
        if let Some(layer) = suite.layers.last_mut() {
            layer.name = name;
        }
    }
}

fn settings(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("Landmass");
    ui.small(
        "Turns a baked world into smooth maps, doing the jobs that need the whole map at once. The World Interpreter \
         makes the ground from them. Bake worlds in the Tectonic Sim, then pick one in the bar along the top.",
    );
    if !compiler.note.is_empty() {
        ui.small(&compiler.note);
    }
    fold_section(ui, "Presets", |ui| {
        ui.small("Save every setting on both panels (the curves too) as a preset, to try on any bake.");
        preset_bar(ui, "compiler", &mut compiler.settings);
    });
    let Some(bake) = compiler.bake.clone() else {
        ui.colored_label(PINK, "No bake read yet.");
        return;
    };

    fold_section(ui, "Sea level and the coastline", |ui| sea_and_coast(ui, compiler, &bake));
    if compiler.result.is_none() {
        return;
    }
    fold_section(ui, "Save", |ui| save_buttons(ui, compiler, &bake));
    fold_section(ui, "Render World and Spawn", |ui| {
        ui.small(
            "Makes a real world from these maps and the Interpreter's recipe, and puts you where you pick. The world keeps \
             its own copy of both, so changing things here afterwards never changes it.",
        );
        ui.horizontal_wrapped(|ui| {
            ui.label("World name:");
            ui.add(egui::TextEdit::singleline(&mut compiler.world_name).desired_width(140.0).hint_text(bake.name.clone()));
        });
        if ui
            .button("Render World and Spawn  [Space]")
            .on_hover_text(
                "Then click on the map where you want to start (or press Space again over the spot); Esc or right-click\n\
                 cancels. You start 3 blocks above the ground, facing north. The world you're in now is replaced\n\
                 (save it first if you want it).",
            )
            .clicked()
        {
            compiler.picking = true;
        }
    });
}

/// The Landmass panel's sea level and coastline sliders (and soften the mountains).
fn sea_and_coast(ui: &mut egui::Ui, compiler: &mut Compiler, bake: &Bake) {
    let s = &mut compiler.settings;
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
    ui.add(crate::debug::Slider::new(&mut s.soften_mountains, 0.0..=512.0).text("soften the mountains (blocks)")).on_hover_text(
        "The bake's squeeze, and which way and how hard the rock was pushed, are traced on the same grid of 32-block pixels.\n\
         Unsoftened, the mountains and the ridges laid across the push show those steps: rows of notches along a range's flanks.\n\
         About 64 blocks (two pixels) hides them; more spreads ranges a little wider and rounds off their small shapes.\n\
         The tallest peaks stay as tall (the most crushed rock is measured again after softening). 0 = as baked.",
    );
    ui.add_space(4.0);
    defaults_row(ui, "Both panels' settings", &mut compiler.settings, &mut compiler.ab, || default_preset("compiler").unwrap_or_default());
}

/// The Landmass panel's buttons that save or send the compiled elevation.
fn save_buttons(ui: &mut egui::Ui, compiler: &mut Compiler, bake: &Bake) {
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
    if ui.button("Send relief to the Graphics Kit").on_hover_text("The compiler's map (a pixel every 32 blocks), coloured by height and shaded, as a picture layer.").clicked() {
        let width = result.made.width;
        let pixels = relief_picture(&result.made, bake.info.blocks_per_pixel, compiler.relief).into_iter().map(|[r, g, b]| [r, g, b, 255]).collect();
        compiler.send = Some(("relief".to_string(), Picture { width, height: width, pixels }, bake.info.blocks_per_pixel));
        compiler.note = "Sent the relief map to the Graphics Kit as a layer.".to_string();
    }
}

/// The signal chain, for the top of the Interpreter panel: the recipe's order, and where each part is
/// set (the Topography panel's sections in [ ]).
const CHAIN: &str = "\
Tectonic Sim -> bake (32-block pixels)
      |
World Compiler · whole map, once:
  sea level · soften coast  [Landmass]
  soften mountains          [Landmass]
  where the mountains stand [Mountains]
      |  smooth maps
World Interpreter · every block, as
  the world is made. In this order:
  blend the maps to the block, then
  curves · grade · edge rise [Land and sea]
  the mountains' rise        [Mountains]
  ridgelines                 [Ridgelines]
  the peaks                  [Mountains]
  torn coast                 [Torn coast]
  hills                      [Hills]
      |  the ground's height
Climate reads it (without the hills,
  torn coast and crags), and carves
  its rivers and lakes into it
      |
Mineralogy: the blocks on it
      |
the world (and the distant terrain)";

/// The Topography panel (the Additives until 2026-10-31): the ground's shape, in David's order [David,
/// 2026-10-31: "move the features where they belong. it should be land and sea, ridgelines, mountains,
/// hills, torn coast"]. Land and sea, Ridgelines, Hills and Torn coast were the Interpreter panel's
/// until then; their settings are still the Interpreter's recipe's, so presets load as they were. The
/// recipe works them out in its own order (the chain on the Interpreter panel).
fn additives_panel(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("Topography");
    ui.small(
        "The ground's shape, before the water: the land and the sea, the ridgelines, the mountains, the hills and the torn \
         coast, worked out at every block as the world is made (the chain on the World Interpreter panel shows in what \
         order). Saved in the presets with the other panels.",
    );
    ui.add_space(4.0);
    let defaults = InterpreterSettings::default();
    land_and_sea(ui, &mut compiler.settings.interpreter, &defaults);
    ridgelines(ui, &mut compiler.settings.interpreter, &defaults);
    let points = compiler.result.as_ref().map(|r| r.made.lift_points.len());
    let show_points = &mut compiler.show_lift_points;
    let (lift, peaks) = (&mut compiler.settings.interpreter.lift, &mut compiler.settings.interpreter.peaks);
    let a = &mut compiler.settings.additives;
    let dots = &mut compiler.additives;

    // Everything here is about mountains, so it's under one heading, with Hydrology to come beside it
    // [David, 2026-10-27]. Where and Rise were the Interpreter's Lift: its point picker gives the
    // peaks to come their points, and its cloth is the bump they're cut down out of. (Still kept in
    // the Interpreter's recipe, as `lift`: the rise is worked out at every block, from points the
    // compiler finds.)
    fold_section(ui, "Mountains", |ui| {
        ui.small(
            "Where the mountains stand, the rise they stand on, and how the mountain belts run. Peaks are next: \
             they'll stand at the points, cut down out of the rise.",
        );
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut dots.show, "show the dots").on_hover_text("Draws the grain drift's and the ridge streams' dots over the map and the close-up.");
            ui.label("size:");
            for size in [1.0, 2.0, 4.0, 8.0] {
                ui.selectable_value(&mut dots.dot_size, size, format!("{size}x"))
                    .on_hover_text("Draws the dots this many times bigger (the seeds stay as they are), to see them from further out.");
            }
        });
        fold_section(ui, "Where", |ui| mountain_points(ui, lift, show_points, points));
        fold_section(ui, "Rise", |ui| mountain_rise(ui, lift));
        let mut on = peaks.on;
        switched_section(ui, "Peaks", &mut on, |ui| mountain_peaks(ui, peaks));
        peaks.on = on;

        fold_section(ui, "Mountain grain", |ui| {
            ui.small("Which way the mountains run at each spot, and whether they're a string (a range running one way) or a knot. See it with the \"range grain\" and \"string or knot\" overlays.");
            ui.add(crate::debug::Slider::new(&mut a.grain_reach, 250.0..=8_000.0).logarithmic(true).text("grain reach (blocks)")).on_hover_text(
                "How far round each spot is looked at to tell which way the mountains run. Small: every spur and bump has its\n\
                 own direction. Large: only whole ranges do (and small knots blend into the range around them).",
            );
            ui.add(crate::debug::Slider::new(&mut a.grain_from_squeeze, 0.0..=1.0).text("from squeeze")).on_hover_text(
                "Where the direction comes from. 0 = the mountains' own shape (which way their slopes face); 1 = the squeeze\n\
                 (ranges run across the way the rock was pushed, as real fold belts do); between = a blend.\n\
                 String or knot always comes from the shape.",
            );
        });

        switched_section(ui, "Grain drift", &mut a.drift_on, |ui| {
            ui.small("Seeds (blue) scattered over the mountains send dots (yellow) drifting the way the range runs, spreading sideways and thinning out. In knots they just scatter round the seed.");
            ui.add(crate::debug::Slider::new(&mut a.drift_spacing, 200.0..=5_000.0).logarithmic(true).text("seed spacing (blocks)"))
                .on_hover_text("About how far apart the seeds are where the mountains are strongest (scattered, not even); further apart where they're weaker.");
            ui.add(crate::debug::Slider::new(&mut a.drift_least, 0.0..=1.0).text("least mountains")).on_hover_text(
                "No seeds where the mountains are weaker than this (0 to 1, as the mountain overlay shows).",
            );
            ui.add(crate::debug::Slider::new(&mut a.drift_most, 0.0..=1.0).text("most mountains")).on_hover_text(
                "No seeds where the mountains are stronger than this: trims off the strongest, as least mountains trims off the weakest.\n\
                 (Between the two, seeds come thicker the stronger the mountains.)",
            );
            ui.add(crate::debug::Slider::new(&mut a.drift_distance, 500.0..=100_000.0).logarithmic(true).text("distance (blocks)"))
                .on_hover_text("How far the dots drift from their seed along the range.");
            ui.add(crate::debug::Slider::new(&mut a.drift_dots, 1..=64).text("dots per seed"));
            ui.add(crate::debug::Slider::new(&mut a.drift_spread, 0.0..=1.0).text("spread")).on_hover_text(
                "How far the dots spread sideways off the range by the far end, as a share of the distance. 0 = all in a line along it.",
            );
            ui.add(crate::debug::Slider::new(&mut a.drift_lean, -1.0..=1.0).text("lean")).on_hover_text(
                "Which side they spread to, looking the way they drift: -1 all to the left, 0 both sides, 1 all to the right.",
            );
            ui.add(crate::debug::Slider::new(&mut a.drift_thin, 0.0..=1.0).text("thin out")).on_hover_text(
                "How many fewer dots there are toward the far end. 0 = as many all the way, 1 = none at the very end.",
            );
            ui.horizontal_wrapped(|ui| {
                ui.label("drift:");
                for way in DriftWay::ALL {
                    ui.selectable_value(&mut a.drift_way, way, way.name());
                }
            })
            .response
            .on_hover_text(
                "A range runs two ways. To the range's tip: each range's own thin end (a carrot's point; the white arrows),\n\
                 the way it reaches furthest from its middle. To a bearing: the same way for every range. Or both ways.",
            );
            ui.add_enabled(
                a.drift_way != DriftWay::Both,
                crate::debug::Slider::new(&mut a.drift_toward, 0.0..=360.0).text("bearing"),
            )
            .on_hover_text(
                "The compass bearing (0 north, 90 east, 180 south, 270 west) dots drift nearer to, like a prevailing wind:\n\
                 for every range when drifting to a bearing, and for seeds outside any range when drifting to the tip.",
            );
            ui.add(crate::debug::Slider::new(&mut a.range_cut, 0.0..=1.0).text("range cut")).on_hover_text(
                "Where the mountains are weaker than this, they're cut into separate ranges, each with its own middle (white\n\
                 ring) and tip (white arrow, longer the more lopsided). Lower: ranges join up into bigger ones.",
            );
            ui.add(crate::debug::Slider::new(&mut a.tip_pull, 0.0..=1.0).text("tip pull")).on_hover_text(
                "How hard each range pulls its dots toward its tip as they drift, so they carry on out past the range's end.\n\
                 0 = follow the grain only; 1 = straight at the tip. Weaker for ranges that are less lopsided (none for an egg).",
            );
            ui.add(crate::debug::Slider::new(&mut a.coherence, 0.0..=1.0).text("coherence")).on_hover_text(
                "How much each range's dots come together into one stream: they're drawn in toward the range's middle line\n\
                 (through its white ring, along its arrow) as they travel, and the spread and knot scatter are turned down.\n\
                 0 = as scattered as the sliders above make them; 1 = one tight stream out of each range.",
            );
            ui.add(crate::debug::Slider::new(&mut a.spiral, -1.0..=1.0).text("spiral")).on_hover_text(
                "Every dot turns to swirl in round the map's middle as it goes: positive clockwise, negative anticlockwise,\n\
                 0 = no spiral. Further out, it bends more.",
            );
            ui.add_enabled(a.spiral != 0.0, crate::debug::Slider::new(&mut a.spiral_exponent, 0.25..=4.0).logarithmic(true).text("spiral exponent"))
                .on_hover_text(
                    "How the spiral's turning grows along each dot's way: 1 = steadily; above 1 = hardly at first, then hard\n\
                     near the end; below 1 = early on, then easing off.",
                );
            ui.add(crate::debug::Slider::new(&mut a.knot_scatter, 0.0..=1.0).text("knot scatter")).on_hover_text(
                "In knots (yellow on the \"string or knot\" overlay), how far round the seed the dots scatter, as a share of the\n\
                 distance. Partly-knotted spots blend the scatter with the drift.",
            );
        });

        switched_section(ui, "Ridge streams", &mut a.streams_on, |ui| {
            ui.small("Dots placed along each range's spine (the \"range spines\" overlay) stream off it, curving with it.");
            ui.add(crate::debug::Slider::new(&mut a.spacing, 50.0..=4_000.0).logarithmic(true).text("spacing (blocks)"))
                .on_hover_text("Blocks between the streams' starts along a spine (scattered, not even).");
            ui.add(crate::debug::Slider::new(&mut a.distance, 0.0..=6_000.0).text("distance (blocks)"))
                .on_hover_text("How far each stream runs from the spine.");
            ui.add(crate::debug::Slider::new(&mut a.dots, 1..=64).text("dots per stream"));
            ui.add(crate::debug::Slider::new(&mut a.bend, -4.0..=4.0).text("bend")).on_hover_text(
                "How much the spine's curve steers the streams as they go: they turn to sweep along the ridge, all the same way\n\
                 round a curve on both sides (like a pinwheel), so neighbours curve together. The sharper the spine curves,\n\
                 the more they turn. 0 = straight off the spine; negative = sweeping the other way.",
            );
            ui.horizontal_wrapped(|ui| {
                ui.label("sides:");
                for sides in Sides::ALL {
                    ui.selectable_value(&mut a.sides, sides, sides.name());
                }
            })
            .response
            .on_hover_text("Which side of the spine streams leave from: both, the side it curves toward, or the side it curves away from.");
            ui.add(crate::debug::Slider::new(&mut a.least_mountains, 0.0..=1.0).text("least mountains")).on_hover_text(
                "Streams start only where the mountains are at least this strong (0 to 1, as the mountain overlay shows),\n\
                 to keep them to the main ranges.",
            );
        });
        ui.add_space(6.0);
        // (It resets only these three, as it always did: Where, Rise and Peaks are the Interpreter's.)
        if ui
            .small_button("Reset the grain, drift and streams")
            .on_hover_text("Mountain grain, Grain drift and Ridge streams back to their defaults (called \"Reset the Topography\" until 2026-10-31).")
            .clicked()
        {
            *a = Default::default();
        }
        if !dots.show {
            ui.small("The dots are hidden.");
        } else if dots.found.is_none() || dots.is_making() {
            ui.small("Finding the range spines and grain…");
        } else {
            ui.small(format!("grain drift: {} seeds, {} dots · ridge streams: {} streams, {} dots", dots.seeds.len(), dots.drift.len(), dots.streams, dots.dots.len()));
        }
    });
    hills(ui, &mut compiler.settings.interpreter, &defaults);
    torn_coast(ui, &mut compiler.settings.interpreter, &defaults);
}

/// Mountains, Where: the points the mountains stand at (what was the Lift's point picker).
fn mountain_points(ui: &mut egui::Ui, lift: &mut Lift, show_points: &mut bool, points: Option<usize>) {
    ui.small("The points come from the bake's own crush, so the ridgelines' curve doesn't move them. Shown in cyan.");
    ui.horizontal(|ui| {
        ui.label("points:");
        ui.selectable_value(&mut lift.from, LiftFrom::Points, "scattered").on_hover_text("Scattered over the crushed rock, thicker where it's more crushed.");
        ui.selectable_value(&mut lift.from, LiftFrom::Ranges, "range middles").on_hover_text("One in the middle of each range of crushed rock.");
    });
    ui.add(crate::debug::Slider::new(&mut lift.least, 0.0..=1.0).text("least crush"))
        .on_hover_text("No points where the rock is less crushed than this (0 to 1, against the bake's most crushed); ranges: none whose most crushed spot is.");
    ui.add(crate::debug::Slider::new(&mut lift.most, 0.0..=1.0).text("most crush"))
        .on_hover_text("No points where the rock is more crushed than this; ranges: none whose most crushed spot is.");
    // Both kinds' settings show, the one not chosen greyed out.
    let scattered = lift.from == LiftFrom::Points;
    ui.add_enabled(scattered, crate::debug::Slider::new(&mut lift.spacing, 300.0..=20_000.0).logarithmic(true).text("spacing (blocks)"))
        .on_hover_text("Scattered points: about how far apart they are where the rock is most crushed (scattered, not even).");
    ui.add_enabled(!scattered, crate::debug::Slider::new(&mut lift.range_cut, 0.0..=1.0).text("range cut"))
        .on_hover_text("Range middles: where the crush is weaker than this, it's cut into separate ranges. Lower: ranges join into bigger ones.");
    ui.add_enabled(!scattered, crate::debug::Slider::new(&mut lift.smallest_range, 250.0..=50_000.0).logarithmic(true).text("smallest range (blocks)"))
        .on_hover_text("Range middles: only ranges at least about this many blocks across get a point.");
    ui.horizontal(|ui| {
        ui.checkbox(show_points, "show the points");
        if let Some(count) = points {
            ui.small(format!("{count} points"));
        }
    });
}

/// Mountains, Rise: the land raised round the points (what was the Lift's cloth).
fn mountain_rise(ui: &mut egui::Ui, lift: &mut Lift) {
    ui.small(
        "Land raised round the points, like a tablecloth draped over them floating above the ground, its edges tapering \
         off just before the coast: the bump the peaks will be cut down out of. Roll the mouse wheel over a slider to \
         nudge it; hold Shift for finer steps.",
    );
    ui.add(crate::debug::Slider::new(&mut lift.height, 0.0..=200.0).text("rise (blocks)"))
        .on_hover_text("How high the land is raised right under a point, at the top of the curve. 0 = no rise.");
    ui.add(crate::debug::Slider::new(&mut lift.reach, 500.0..=60_000.0).logarithmic(true).text("reach (blocks)"))
        .on_hover_text("How far the cloth spreads out from its points: the curve's left end is this far from the nearest point.");
    ui.label("The cloth: from reach away (left) to right under a point (right).");
    spline_box(ui, "lift spline", &mut lift.curve, &Lift::default().curve, "toward the nearest point", "share of the rise");
    ui.add(crate::debug::Slider::new(&mut lift.pad, 32.0..=20_000.0).logarithmic(true).text("coast pad (blocks)")).on_hover_text(
        "Within this far of the coast, the rise is braked, down to nothing at the coastline, so the coast never moves.\n\
         The rise's only: the peaks have their own coast fade.",
    );
    ui.add(crate::debug::Slider::new(&mut lift.pad_falloff, 0.1..=10.0).logarithmic(true).text("pad falloff")).on_hover_text(
        "How the pad brakes: 1 = evenly across it; above 1 = braking hard from further in; below 1 = only right at the\n\
         coast, so the rise runs nearly to the shore.",
    );
}

/// Mountains, Peaks: a mountain at each point, on top of everything else.
fn mountain_peaks(ui: &mut egui::Ui, peaks: &mut Peaks) {
    ui.small(
        "A peak stands at each of the points, on top of the rise and the ridgelines, and its crags are carved into it. \
         Each peak takes its own height and width from where it stands, within the ± spreads. Past the ground's \
         cut-off it's flat: see \"ground never above\" (World Interpreter, Land and sea).",
    );
    let default = Peaks::default();
    spread_row(ui, &mut peaks.height_spread, crate::debug::Slider::new(&mut peaks.height, 0.0..=800.0).text("height (blocks)"))
        .on_hover_text("How high a peak stands above the ground under it, at its top.");
    spread_row(ui, &mut peaks.width_spread, crate::debug::Slider::new(&mut peaks.width, 50.0..=10_000.0).logarithmic(true).text("width (blocks)"))
        .on_hover_text("How far from its point a peak's foot reaches. Narrower than its height = a towering spire.");
    spread_row(ui, &mut peaks.wobble_spread, crate::debug::Slider::new(&mut peaks.wobble, 0.0..=2.0).text("outline wobble")).on_hover_text(
        "How far a peak's outline wanders in and out round it, as a share of its width: fatter on one side, pinched on\n\
         another. 0 = round. Past 1 the pinches close right up, splitting a peak's foot into lobes.",
    );
    ui.horizontal(|ui| {
        ui.label("where peaks overlap:");
        ui.selectable_value(&mut peaks.add, false, "higher wins").on_hover_text("Separate peaks: where two overlap, the ground follows the higher.");
        ui.selectable_value(&mut peaks.add, true, "add up").on_hover_text("They add up where they overlap, merging into massifs.");
    });
    ui.label("The peak: from its foot (left) to its top (right).");
    spline_box(ui, "peak rise", &mut peaks.rise, &default.rise, "toward the point", "share of the height");

    ui.add_space(4.0);
    ui.label("Crags: layers of noise, each finer and weaker, kept or smoothed by how steep the peak is there.");
    ui.add(crate::debug::Slider::new(&mut peaks.crags, 0.0..=1.0).text("crags")).on_hover_text(
        "How big the crags are, as a share of the peak's height (so a small peak's crags don't swamp it). 0 = smooth.",
    );
    ui.add(crate::debug::Slider::new(&mut peaks.crag_size, 16.0..=4_000.0).logarithmic(true).text("biggest crag (blocks)"))
        .on_hover_text("How far across the biggest layer's lumps are. Each finer layer's are smaller, down to a block.");
    ui.add(crate::debug::Slider::new(&mut peaks.crag_layers, 1..=8).text("layers")).on_hover_text("How many layers, each finer than the last (any finer than a block are left out).");
    ui.add(crate::debug::Slider::new(&mut peaks.crag_finer, 1.1..=6.0).text("each layer finer by ×")).on_hover_text(
        "How much smaller each layer's lumps are than the last's (at the top of its curve below).",
    );
    ui.label("How much of that each step takes: from the first step (left) to the last (right).");
    spline_box(ui, "finer by layer", &mut peaks.finer_by_layer, &default.finer_by_layer, "later steps", "share of finer");
    ui.add(crate::debug::Slider::new(&mut peaks.crag_weaker, 0.2..=1.0).text("each layer as strong ×")).on_hover_text(
        "How strong each layer is next to the last (at the top of its curve below). Higher = rougher, more broken;\n\
         lower = smoother, with only the big shapes. 1 = every layer as strong as the first.",
    );
    ui.label("How much of that each step takes: from the first step (left) to the last (right).");
    spline_box(ui, "strength by layer", &mut peaks.weaker_by_layer, &default.weaker_by_layer, "later steps", "share of strength");
    ui.label("Slope response: how much of each layer is kept (up), by how steep it is there (across: flat to steepest).");
    spline_box(ui, "slope response", &mut peaks.slope_response, &default.slope_response, "steeper", "kept");
    ui.add(crate::debug::Slider::new(&mut peaks.steepest, 0.1..=8.0).logarithmic(true).text("steepest (up per across)")).on_hover_text(
        "How steep the right-hand end of the slope response is: blocks up for each block across (1 = 45°). Steeper than\n\
         this counts as this. Falling to the right = the crags smoothed off where it's steep (worn, the article's\n\
         look); rising = kept or sharpened there (towering, broken faces); a bump = crags only at one steepness.",
    );
    ui.checkbox(&mut peaks.carve_below, "crags carve below the peaks").on_hover_text(
        "On: the crags' dips can cut into the ground the peak stands on (the ridgelines and rise), never below the sea.\n\
         Off: they only carve the peak itself.",
    );
    ui.add(crate::debug::Slider::new(&mut peaks.foot_fade, 0.0..=1.0).text("foot fade")).on_hover_text(
        "How far up a peak, from its foot, its crags take to come to full strength, as a share of its width.\n\
         0 = full right at the foot (craggy all the way down); higher = smoother lower slopes. The crags always\n\
         stop at the foot.",
    );

    ui.add_space(4.0);
    ui.label("Summit crags: a finer field pressed into all the ground high up, stronger the higher it is.");
    ui.add(crate::debug::Slider::new(&mut peaks.summit, 0.0..=120.0).text("summit crags (blocks)")).on_hover_text(
        "How far the summit crags push the ground up and down, at full strength. 0 = none. They work on everything above\n\
         the start height (peaks and the tops of tall ridgelines alike), braked by the coast fade below.",
    );
    let summits = peaks.summit > 0.0;
    ui.add_enabled_ui(summits, |ui| {
        ui.add(crate::debug::Slider::new(&mut peaks.summit_from, 128.0..=1_023.0).text("start height (Z)"))
            .on_hover_text("Below this, no summit crags.");
        ui.add(crate::debug::Slider::new(&mut peaks.summit_full, 128.0..=1_023.0).text("full height (Z)"))
            .on_hover_text("From here up, the summit crags are at full strength (as the curve's right end says).");
        ui.label("How they fade in: from the start height (left) to the full height (right).");
        spline_box(ui, "summit fade", &mut peaks.summit_fade, &default.summit_fade, "higher", "share of the summit crags");
        noise_box(ui, "summit crags", &mut peaks.summit_noise, &default.summit_noise, 8.0..=1_000.0);
    });

    ui.add_space(4.0);
    ui.label("Coast fade: the peaks braked toward the coast so it never moves.");
    ui.add(crate::debug::Slider::new(&mut peaks.coast_fade, 32.0..=20_000.0).logarithmic(true).text("coast fade (blocks)")).on_hover_text(
        "Within this far of the coast, the peaks are braked, down to nothing at the coastline. The peaks'\n\
         own (they used to borrow the Rise's coast pad, so the Rise's settings changed them even with \"rise\" off).",
    );
    ui.add(crate::debug::Slider::new(&mut peaks.coast_falloff, 0.1..=10.0).logarithmic(true).text("coast fade falloff")).on_hover_text(
        "How it brakes: 1 = evenly across it; above 1 = braking hard from further in; below 1 = only right at the coast.",
    );
}

/// A slider with a ± spread box before it, as the Tectonic Sim's (1 = half to twice).
fn spread_row(ui: &mut egui::Ui, spread: &mut f32, slider: crate::debug::Slider<f32>) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(spread).range(0.0..=2.0).speed(0.01).prefix("± ")).on_hover_text(
            "Spread: each peak takes its own value, from its own seed. 1 = anywhere from half to twice; 2 = a quarter to\n\
             four times. 0 = all alike.",
        );
        ui.add(slider)
    })
    .inner
}

/// The Climate panel: the weather the world is made with, worked out once over the whole map, and
/// the water it leaves (see `climate.rs`). Built a stage at a time; the stages still to come are
/// shown greyed, in the order they'll happen.
fn climate_panel(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("Climate");
    ui.small(
        "Between Topography and Mineralogy: the rivers and lakes first (they read the ground alone), then temperature, \
         wind and rain, worked out once over the whole map. It reads the ground the recipe makes, and the World \
         Interpreter carves what it chooses. Nothing here changes the ground before it.",
    );
    let climate = &mut compiler.settings.climate;
    ui.horizontal_wrapped(|ui| {
        use super::climate::Show;
        ui.label("Show over the map:");
        let show = &mut compiler.climate.show;
        ui.selectable_value(show, Show::Nothing, "nothing");
        ui.selectable_value(show, Show::Ground, "the ground it reads").on_hover_text("The broad ground: sea blue, land grey (lighter higher), lit from the north-west.");
        ui.selectable_value(show, Show::Water, "water").on_hover_text("Where the water flows (brighter and bluer, the more it carries) and the lakes.");
        ui.selectable_value(show, Show::Rivers, "rivers").on_hover_text("The river network: rivers bluer and more solid by rank, waterfalls white, rapids pale; springs green (a river's head), yellow (joined one), cyan (into a lake or the sea), red (fizzled out); mouths magenta.");
        ui.selectable_value(show, Show::Temperature, "temperature").on_hover_text("Blue frozen, through green, to red hottest; the sea dimmed; the three lines in white.");
        ui.selectable_value(show, Show::Wind, "wind").on_hover_text("Arrows the way the wind blows, longer and brighter where it's stronger, over the ground it reads.");
        ui.selectable_value(show, Show::Rain, "rain").on_hover_text("The rain, or one of its parts (chosen in the Rain section).");
    });
    fold_section(ui, "The ground it reads", |ui| {
        ui.small(
            "The World Interpreter's own recipe, asked for the ground's height in the middle of each climate pixel, with its \
             fine layers left out (crags, summit crags, hills, torn coast): the broad shape of the land, peaks and \
             ridgelines and all. Weather and water follow that; crag noise would only make false pits for water to pool in.",
        );
        let blocks = climate.pixels * MAP_PIXEL_BLOCKS;
        ui.add(crate::debug::Slider::new(&mut climate.pixels, 1..=16).text(format!("climate pixel ({blocks} blocks)")))
            .on_hover_text("How far across one climate pixel is, in the map's 32-block pixels, for the temperature, wind and rain. Smaller = finer, slower. (The rivers read the same broad ground on the map's own pixels.)");
    });
    fold_section(ui, "Rivers", |ui| {
        let rivers = &mut climate.rivers;
        ui.small(
            "Where the water goes, on the map's own grid (32 blocks a pixel). Every dip with no way out fills to the lowest \
             point of its rim, a lake with a flat surface, and spills on downhill; every patch drains somewhere, always to the \
             sea. Each patch of land gives a unit of water (not its rain), and it adds up downhill: the more it carries, the \
             brighter and bluer it's drawn. (Snowmelt and slope-foot springs, which added to it, were retired on 2026-10-31: only \
             a spring's own water read them, and nothing it decided showed. Their code is kept.)",
        );
        if let Some(flow) = compiler.climate.flow() {
            ui.label(format!("Most flow: {:.0} units   land under lakes: {:.1}%", flow.most, flow.lake_share * 100.0));
        }
        if ui.small_button("show the water on the map").on_hover_text("Shows the water over the map (the Show row at the top).").clicked() {
            compiler.climate.show = super::climate::Show::Water;
        }
        ui.add(crate::debug::Slider::new(&mut rivers.shown_from, 1.0..=5_000.0).logarithmic(true).text("the picture: water drawn from (units)"))
            .on_hover_text("The picture only (the Show row's \"water\"): how much water has to flow through a patch for it to be drawn. Lower = every trickle; higher = only the big rivers. It changes no river.");
        ui.add(crate::debug::Slider::new(&mut rivers.lake_depth, 0.5..=40.0).logarithmic(true).text("a lake from (blocks deep)"))
            .on_hover_text("How deep a basin's water has to stand to count as a lake (shallower is wet ground, later perhaps marsh).");
    });
    fold_section(ui, "River network", |ui| {
        let by_area = climate.carving.by_area;
        let n = &mut climate.network;
        ui.small(
            "Which of the water's paths are rivers, grown from the mouths up to the springs along the way the water drains \
             (so every river runs downhill to the sea and stays in its own valleys). Springs first, spaced out; each spring's \
             path ends at a mouth; rivers grow up from each mouth toward its springs, splitting where they part. Springs no \
             river took run down as short streams: they join a river, end in a lake or the sea, or fizzle out. Rank comes \
             from branches joining (stream order). The bed is laid in flat runs and steps: waterfalls and rapids.",
        );
        if let Some(network) = compiler.climate.network() {
            let [river, joined, ended, fizzled] = network.fates();
            ui.label(format!("{} springs: {river} rivers' heads, {joined} joined one, {ended} into a lake or the sea, {fizzled} fizzled out; {} mouths", network.springs.len(), network.mouths.len()));
        }
        if ui.small_button("show the rivers on the map").on_hover_text("Shows the river network over the map (the Show row at the top).").clicked() {
            compiler.climate.show = super::climate::Show::Rivers;
        }
        ui.add_space(4.0);
        ui.strong("Springs");
        ui.horizontal(|ui| {
            ui.label("by:");
            ui.selectable_value(&mut n.relative, false, "plain height").on_hover_text("Springs where the ground is between two heights.");
            ui.selectable_value(&mut n.relative, true, "relative height").on_hover_text("Springs where the ground stands high for its surroundings, so lesser hills get some too.");
        });
        ui.add_enabled_ui(!n.relative, |ui| {
            ui.add(crate::debug::Slider::new(&mut n.spring_from, 128.0..=1_023.0).text("from Z"));
            ui.add(crate::debug::Slider::new(&mut n.spring_to, 128.0..=1_023.0).text("up to Z"));
        });
        ui.add_enabled_ui(n.relative, |ui| {
            ui.add(crate::debug::Slider::new(&mut n.rise, 1.0..=400.0).logarithmic(true).text("higher than its surroundings by (blocks)"));
            ui.add(crate::debug::Slider::new(&mut n.around, 200.0..=20_000.0).logarithmic(true).text("surroundings within (blocks)"))
                .on_hover_text("How far round a spot its surroundings' average height is taken from.");
        });
        ui.add(crate::debug::Slider::new(&mut n.spring_water, 1.0..=5.0).text("water it needs (units)"))
            .on_hover_text("Its own water: 1 unit for a patch of land, plus snowmelt and slope-foot springs (not rain). 1 = any high land; more = only where snow or a slope foot adds water.");
        ui.add(crate::debug::Slider::new(&mut n.spacing, 200.0..=20_000.0).logarithmic(true).text("spaced at least (blocks)"))
            .on_hover_text("The least distance between springs: what spreads the rivers evenly round the world.");
        ui.add(crate::debug::Slider::new(&mut n.jitter, 0.0..=1.0).text("spacing varies")).on_hover_text("0 = as even as can be; 1 = some springs as close as you like.");
        ui.add(crate::debug::Slider::new(&mut n.lake_size, 1.0..=400.0).logarithmic(true).text("a lake feeds a river from (patches)"))
            .on_hover_text("A lake this big or bigger (in 32-block patches) spills a river from its outlet; smaller is a pond, which only takes them in.");
        ui.add_space(4.0);
        ui.strong("Mouths and branches");
        ui.add(crate::debug::Slider::new(&mut n.mouth_spacing, 0.0..=40_000.0).text("mouths apart at least (blocks)"))
            .on_hover_text("Rivers reaching the sea nearer than this: the one with fewer springs loses them (they run as short streams).");
        ui.add(crate::debug::Slider::new(&mut n.branching, 0.0..=1.0).text("how readily rivers split"))
            .on_hover_text("0 = one trunk to each mouth; 1 = a branch up to every spring. Between: a branch is kept if it carries enough of the springs.");
        ui.add(crate::debug::Slider::new(&mut n.reach, 0.0..=20_000.0).text("a lone stream runs (blocks)"))
            .on_hover_text("How far a spring no river took runs before it dries away (springs with more water run further).");
        ui.add_enabled(!by_area, crate::debug::Slider::new(&mut n.orders_per_rank, 1..=4).text("stream orders per rank")).on_hover_text(
            "Where two branches of the same order meet, the order goes up one. This many orders make one rank (creek, stream, river,\n\
             great river). Greyed out: Carving's \"sizes by area\" is ticked, so ranks don't size the rivers.",
        );
        ui.add_space(4.0);
        ui.strong("The bed: flat runs and steps");
        ui.add(crate::debug::Slider::new(&mut n.step, 0.0..=40.0).text("drop gathered into one step (blocks)"))
            .on_hover_text("0 = the bed follows the ground; higher = longer flat runs, then bigger steps.");
        ui.add(crate::debug::Slider::new(&mut n.fall_from, 1.0..=40.0).text("a waterfall from (blocks)")).on_hover_text("A step this tall or more is a waterfall; smaller ones are rapids.");
    });
    fold_section(ui, "Carving", |ui| {
        let c = &mut climate.carving;
        ui.small(
            "The rivers as smooth lines, carved into the ground with water in them (no flow, nothing moves), and the lakes \
             filled to their surface. Each river finds its way on the real ground, round the hills and through the lowest \
             gaps, in a corridor round the network's plan; then its corners are cut, with a gentle wander, leaning to the \
             outside of its bends, and meandering on flat valley floors; its ends stay put so rivers always meet, and its \
             water only falls downstream. Each block measures its distance to the line; a river's odd block goes to the outside \
             of the bend. The carve lowers the ground itself, so the far terrain and this view show the rivers too. Climate \
             reads the ground without them.",
        );
        if let Some(carving) = compiler.climate.carving() {
            let points: usize = carving.reaches.iter().map(|reach| reach.points.len()).sum();
            ui.label(format!("{} stretches of river, {points} points, made in {:.1} s", carving.reaches.len(), carving.took));
            ui.label(format!("deepest cut below its banks: {:.0} blocks; {} points cut deeper than 6", carving.deepest_cut, carving.deep_cuts))
                .on_hover_text("How far the water passes below the lowest ground across a river. Following the valleys, this should stay small; deep cuts are trenches through rises it couldn't find a way round.");
            ui.label(format!("{} lakes and ponds", carving.pools));
            ui.label(format!("{} ponds behind rises; {} rises cut through instead ({} would drain into a river)", carving.rise_ponds, carving.cut_instead, carving.into_water)).on_hover_text(
                "Rises cut through instead: deeper than the cut, but their pond would have spread too far (Lakes: \"a pond spreads at most\"), \
                 or drained into a river below its water: another river's channel in it (not one flowing into it), its own river below the \
                 rise, or a river's notch through its rim. It would have stood above that water, in walls. (A lake below a pond is taken in.)",
            );
            let sh = &carving.shores;
            let share = |n: usize| 100.0 * n as f32 / sh.shores().max(1) as f32;
            ui.label(format!(
                "shores: {:.0}% flush, {:.0}% a 1-block ledge, {:.0}% taller banks, {:.0}% below the water",
                share(sh.flush),
                share(sh.ledge),
                share(sh.bank),
                share(sh.below)
            ))
            .on_hover_text(format!(
                "Sampled at every 8th point of every river above the sea, both sides ({} shores): the first dry block beside the water.\n\
                 Flush: its top is level with the water, so you can walk out. A ledge: a block up, which you can't climb out onto.\n\
                 Below the water: water standing against air there (there should be none).",
                sh.shores()
            ));
            let columns = sh.columns.max(1) as f64;
            ui.label(format!(
                "land beside rivers: lowered {:.1} blocks on average (most {:.0}), raised {:.1} (most {:.0}); steepest pull-down {:.1} up a block across",
                sh.lowered / columns,
                sh.most_lowered,
                sh.raised / columns,
                sh.most_raised,
                sh.steepest
            ))
            .on_hover_text(format!(
                "Across the same banks, out to as far as each bank's fade could widen ({} blocks sampled): how much the carve lowered\n\
                 or raised the land, on average and at most. The steepest pull-down is the steepest slope next to land the carve changed.",
                sh.columns
            ));
        }
        ui.checkbox(&mut compiler.lab_shores, "shores in the close-up").on_hover_text(
            "Colours the dry block beside every river's and lake's water in the close-up: green flush (you can walk out), amber a\n\
             1-block ledge, red a taller bank, magenta below the water (water standing against air). Closest in, every block.",
        );
        ui.checkbox(&mut compiler.lab_sizes, "sizes in the close-up").on_hover_text(
            "Colours every river's water in the close-up by its water across (its blocks of water, width by depth by how full its\n\
             bowl is): yellow 1 block, green 4, sky blue 16, violet 64, magenta 256 and more. Lakes and the sea are left as they are.",
        );
        let most = std::thread::available_parallelism().map_or(4, |n| n.get());
        ui.add(crate::debug::Slider::new(&mut c.cores, 1..=most).text(format!("cores used (of {most})"))).on_hover_text(
            "How many of the CPU's cores making the rivers shares its work over (it starts at half). Fewer runs cooler and takes \
             longer (\"made in\" above); the rivers come out the same either way.",
        );
        ui.add_space(4.0);
        ui.strong("The route");
        ui.horizontal_wrapped(|ui| {
            use super::carving::Route;
            ui.label("found:");
            ui.selectable_value(&mut c.route, Route::Everywhere, "everywhere").on_hover_text("In the corridor round every river's plan (the way before stage 4).");
            ui.selectable_value(&mut c.route, Route::WhereNeeded, "only where needed").on_hover_text(
                "Stage 4 of the water rework: each river follows its plan, and only where its water would cut deeper than the\n\
                 deep cut below does it look for a way round, in a box round that stretch, out to where the cut is small again\n\
                 (half the stretch's length or 4 times its cut across, 32 to 256 blocks, at an 8-block grain). It only looks nearby: a\n\
                 river crossing a whole range with no gap near it still cuts through.",
            );
            ui.selectable_value(&mut c.route, Route::Off, "off").on_hover_text("Every river follows its plan, smoothed.");
        });
        let everywhere = c.route == super::carving::Route::Everywhere;
        ui.add_enabled(everywhere, crate::debug::Slider::new(&mut c.corridor, 8.0..=320.0).logarithmic(true).text("corridor (blocks either side)"))
            .on_hover_text("How far either side of the network's plan a river may find its way round hills. Wider follows the land more freely, and takes longer.");
        ui.add_enabled(everywhere, crate::debug::Slider::new(&mut c.grain, 1.0..=16.0).text("grain (blocks)")).on_hover_text("How finely the way is found: smaller follows the land more closely, and takes longer.");
        let needed = c.route == super::carving::Route::WhereNeeded;
        ui.add_enabled(needed, crate::debug::Slider::new(&mut c.deep_cut[0], 2.0..=40.0).text("a way round where the cut is deeper than (near the sea)")).on_hover_text(
            "How far below the ground a river's water may cut before it looks for a way round, at the sea's level. Small here\n\
             means rivers wind round the land down low (David: \"better for the rivers to meander around the terrain down low\").",
        );
        ui.add_enabled(needed, crate::debug::Slider::new(&mut c.deep_cut[1], 2.0..=40.0).text("…and high up")).on_hover_text(
            "The same at the highest river's level; between, it's blended by the water's height. Large here lets rivers cut\n\
             gorges high up (David: gorges are \"better up high\").",
        );
        if let Some(carving) = compiler.climate.carving() {
            let found = if needed { format!("{} of {} stretches cutting too deep found a better way round; ", carving.rerouted, carving.deep_stretches) } else { String::new() };
            ui.label(format!("{found}deepest cut on the plan {:.0}, on the way found {:.0}; {:.1} s", carving.plan_cut, carving.line_cut, carving.route_took)).on_hover_text(
                "The deepest the water would cut below the ground under each river's line (read every 8 blocks along it, the water\n\
                 only falling), on the network's plan and on the way found; and how long finding the ways (and the curves) took.",
            );
        }
        ui.add_space(4.0);
        ui.strong("The line");
        ui.add(crate::debug::Slider::new(&mut c.smoothing, 0..=5).text("corner cutting (passes for a creek)")).on_hover_text("Each pass slices the corners off the route, over points about a river's width apart; bigger rivers get one more pass a rank (by area: a stream one more, a river two), so they bend wider.");
        ui.checkbox(&mut c.merge_alongside, "rivers alongside each other merge").on_hover_text(
            "David: two rivers side by side \"should cause a waterfall to merge\". Where a river's water comes within a block of\n\
             another river's lower water (away from its own ends), it ends there, falling to that river's level, and the rest of\n\
             its way alongside is dropped.",
        );
        if let Some(carving) = compiler.climate.carving() {
            ui.small(format!("{} rivers merged into one alongside", carving.merged));
        }
        ui.add(crate::debug::Slider::new(&mut c.join_fade, 0.0..=10.0).text("joins ease over (the joined river's widths)")).on_hover_text(
            "David's joint fade: where a river joins another, its water eases down toward the joined river's over its last\n\
             stretch, this many of the joined river's widths long, as steeply as rapids go; what's left is a fall at the shore.\n\
             0 = a step at the shore.",
        );
        ui.checkbox(&mut c.anti_kitty_corner, "anti-kitty-cornerer").on_hover_text(
            "David's rule: the water's blocks always join by a face, never only at a corner. On a diagonal, a river too narrow\n\
             to cover its blocks' faces (under about 1.4 blocks) steps along in a 1-block zigzag: every block the line passes\n\
             through is water. Wider rivers are unchanged.",
        );
        ui.add_space(4.0);
        ui.strong("The water's level");
        ui.horizontal_wrapped(|ui| {
            use super::carving::Level;
            ui.label("after a hollow:");
            ui.selectable_value(&mut c.level, Level::Cut, "cut through").on_hover_text(
                "The water only falls, from the lowest ground so far: after a hollow it stays at the hollow's level and cuts\n\
                 through every rise after it (the way before).",
            );
            ui.selectable_value(&mut c.level, Level::LeastChange, "least change").on_hover_text(
                "The water only falls, at the level that changes the land least along each stretch: rises cut and hollows filled\n\
                 by turns. In a filled hollow the water is deeper and its banks are raised to the shore, easing back down to the\n\
                 land over their fade.",
            );
        });
        let least = c.level == super::carving::Level::LeastChange;
        ui.add_enabled(least, crate::debug::Slider::new(&mut c.fill_up_to, 0.0..=20.0).text("fill hollows up to (blocks)"))
            .on_hover_text(
                "The most the water may stand above the lowest ground across a hollow. A deeper hollow is filled this much, and the\n\
                 rise after it cut for the rest. And never above the lower bank, 3 blocks out either side: only where the land rises\n\
                 on both sides is a hollow filled (no causeways over valleys, no embankments along hillsides). Steps work the same\n\
                 way: each flat run at the level that changes its land least.",
            );
        ui.add_enabled(least, crate::debug::Slider::new(&mut c.fill_over_banks, 0.0..=10.0).text("…and above the lower bank up to (blocks)")).on_hover_text(
            "How far the water may stand above the lower bank beside it (3 blocks out from the water). 0 = only where the land\n\
             rises on both sides (no embankments at all, but rivers along a slope can't fill, so more is cut). More lets a filled\n\
             hollow raise its low side's bank, easing down at \"raised banks no steeper than\".",
        );
        ui.add_enabled(least, crate::debug::Slider::new(&mut c.cut_most, 0.0..=40.0).text("cut rises up to (blocks)"))
            .on_hover_text("Before cutting a rise deeper than this, the hollow before it is filled more (but never past \"fill hollows up to\", which wins).");
        if let Some(carving) = compiler.climate.carving() {
            ui.label(format!("water above its ground: at {} points, at most {:.0} blocks", carving.filled, carving.deepest_fill))
                .on_hover_text("Where the water stands above the lowest ground across a river by half a block or more: a hollow filled (or a pond).");
        }
        ui.add_space(4.0);
        ui.strong("Sizes");
        ui.checkbox(&mut c.by_area, "sizes by area").on_hover_text(
            "Stage 5 of the water rework, an experiment beside the ranks' sizes (David: \"make sure it can be disable with a\n\
             checkbox\"). A river's size is the blocks of water across it, gathered on its way down: a spring's to start, most of\n\
             both where waters join (a lake passes on the water of the rivers into it), and a little more as it runs. Its slope\n\
             lays it out: broad and shallow on flat ground, deep and narrow on steep. Unticked, the ranks below size the rivers.",
        );
        let by_area = c.by_area;
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.spring_area, 0.5..=16.0).logarithmic(true).text("a spring's water (blocks across)"))
            .on_hover_text("1 = a block of water: a spring a block wide and a block deep (David: \"a spring is 1 block\").");
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.join_keeps, 0.5..=1.0).text("where waters join, they keep (share of both)"))
            .on_hover_text("David: two runs of water joining \"should probably add most of their area together\". 1 = all of both.");
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.grows_every, 50.0..=20_000.0).logarithmic(true).text("…and grow a block of water every (blocks run)"))
            .on_hover_text("How fast water grows just by running (David: it \"grows slowly just by running, and quickly by joining\").");
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.broadest, 2.0..=32.0).logarithmic(true).text("on flat ground, blocks wide for each block deep"))
            .on_hover_text("Flat, gentle land spreads the water broad and shallow.");
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.deepest, 0.25..=4.0).logarithmic(true).text("on steep ground, blocks wide for each block deep")).on_hover_text(
            "Steep falls and drops stack the water deep and narrow. 0.5 = twice as deep as wide (David's \"2 deep spring with steep\n\
             falls and drops\"). Never under a block wide or deep.",
        );
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.flat_below, 0.01..=5.0).logarithmic(true).text("flat ground falls at most (blocks in 100)")).on_hover_text(
            "Where the broad ground falls this little or less along a river (over up to 4 patches down, 128 blocks), the water is at\n\
             its broadest. On David's world, a quarter of the rivers' patches are flat, and half fall under 0.5 blocks in 100.",
        );
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.steep_from, 0.5..=50.0).logarithmic(true).text("steep ground falls at least (blocks in 100)")).on_hover_text(
            "Where it falls this much or more, the water is at its deepest; between the two, evenly by ratio. On David's world, a\n\
             tenth of the rivers' patches fall 7 blocks in 100 or more.",
        );
        if let Some(carving) = compiler.climate.carving() {
            let sizes = &carving.sizes;
            let all = sizes.lengths.iter().sum::<f32>().max(1.0);
            let share = |k: usize| 100.0 * sizes.lengths[k] / all;
            let (stream, river) = (crate::world::STREAM_FROM, crate::world::RIVER_FROM);
            ui.label(format!(
                "water across: under {stream:.0} blocks for {:.0}% of the rivers' length, {stream:.0} to {river:.0} for {:.0}%, more for {:.0}%; widest {:.0}, deepest {:.0}, most water across {:.0}",
                share(0),
                share(1),
                share(2),
                sizes.widest,
                sizes.deepest,
                sizes.most
            ))
            .on_hover_text(
                "However the rivers are sized (by area or by rank): a river's water across is its width by its depth by how full its\n\
                 bowl is, every column at least a block. Under 3 is a spring, 3 to 16 a stream, more a river (names only).",
            );
        }
        ui.add_space(2.0);
        ui.add_enabled(
            !by_area,
            egui::Label::new(
                egui::RichText::new(
                    "By rank, from David's drawing: each rank so many times wider and deeper than the one below. A rank's sizes are \
                     a band meeting the ranks either side, and a river grows down its band as it goes (a creek from a spring's size), \
                     so it's nearly the next rank's size where it joins one.",
                )
                .small(),
            ),
        );
        ui.add_enabled(!by_area, crate::debug::Slider::new(&mut c.spring_width, 1.0..=4.0).text("a spring's width (blocks)"));
        ui.add_enabled(!by_area, crate::debug::Slider::new(&mut c.spring_depth, 1.0..=3.0).text("…its depth (blocks)"));
        // Which ranks any river reaches, and so which settings act: a rank's band runs from below its
        // own size up toward the next rank's (`carving::size_in_band`), so a creek uses the first two
        // steps up and the first two shapes. (All act until the network's made.)
        let reached: [bool; 4] = compiler.climate.network().map_or([true; 4], |network| [1, 2, 3, 4].map(|rank| network.rank.contains(&rank)));
        let any = |from: usize, to: usize| (from.max(1)..=to.min(4)).any(|rank| reached[rank - 1]);
        if reached != [true; 4] {
            ui.small("Settings only ranks no river reaches would use are greyed out (fewer stream orders per rank lets rivers reach them).");
        }
        for (k, step) in ["spring to creek", "creek to stream", "stream to river", "river to great river"].iter().enumerate() {
            let used = !by_area && any(k, 4);
            ui.add_enabled(used, egui::Label::new(*step));
            ui.add_enabled(used, crate::debug::Slider::new(&mut c.width_ratio[k], 1.5..=5.0).text("times wider"));
            ui.add_enabled(used, crate::debug::Slider::new(&mut c.depth_ratio[k], 1.0..=3.0).text("times deeper"));
        }
        ui.add(crate::debug::Slider::new(&mut c.size_varies, 0.0..=0.5).text("size varies")).on_hover_text("How much a river's size wanders along it, slowly (0.15 = up to 15% either way).");
        for (k, rank) in super::network::RANKS.iter().enumerate() {
            let ((low_w, low_d, _), (high_w, high_d, _)) = (super::carving::size_in_band(c, k + 1, 0.0), super::carving::size_in_band(c, k + 1, 1.0));
            ui.small(format!("{rank}: {low_w:.1} to {high_w:.1} wide, {low_d:.1} to {high_d:.1} deep"));
        }
        ui.add_space(4.0);
        ui.strong("The shape across");
        ui.small(
            "By area: the water's on the flattest ground and on the steepest, mixed by the slope between. By rank: for each rank, \
             in the middle of its band (halfway to its neighbours' at its ends). The preview below draws them like David's drawing.",
        );
        for (name, shape) in [("by area, on flat ground", &mut c.flat_shape), ("by area, on steep ground", &mut c.steep_shape)] {
            ui.add_enabled(by_area, egui::Label::new(name));
            ui.add_enabled(by_area, crate::debug::Slider::new(&mut shape.edge, 0.0..=1.0).text("water at the bank (share of the deepest)")).on_hover_text("0.3 = a V; 0.5 = steep sides; 1 = a box.");
            ui.add_enabled(by_area, crate::debug::Slider::new(&mut shape.flat, 0.0..=0.9).text("flat floor (share of the width)"));
            ui.add_enabled(by_area, crate::debug::Slider::new(&mut shape.bowl, 0.2..=5.0).logarithmic(true).text("bowl")).on_hover_text("Below 1: deep quickly, then flat (steep sides). 1: even steps. Above 1: shallow shelves, then dropping.");
        }
        for (k, rank) in super::network::RANKS.iter().enumerate() {
            let shape = &mut c.shapes[k];
            let used = !by_area && any(k, k + 2);
            ui.add_enabled(used, egui::Label::new(*rank));
            ui.add_enabled(used, crate::debug::Slider::new(&mut shape.edge, 0.0..=1.0).text("water at the bank (share of the deepest)")).on_hover_text("0.3 = a V; 0.5 = steep sides; 1 = a box.");
            ui.add_enabled(used, crate::debug::Slider::new(&mut shape.flat, 0.0..=0.9).text("flat floor (share of the width)"));
            ui.add_enabled(used, crate::debug::Slider::new(&mut shape.bowl, 0.2..=5.0).logarithmic(true).text("bowl")).on_hover_text("Below 1: deep quickly, then flat (steep sides). 1: even steps. Above 1: shallow shelves, then dropping.");
        }
        ui.add(crate::debug::Slider::new(&mut c.shape_varies, 0.0..=1.0).text("shape varies")).on_hover_text("How much the shape wanders along a river (David's drawing varies as much within a rank as between them).");
        profiles_preview(ui, c);
        ui.add_space(4.0);
        ui.strong("Banks");
        ui.small(
            "Beside the water the ground is pulled to the shore (its top flush with the water) and fades back to its own \
             height: lower where the land is higher, raised as a levee where it's lower, so water never stands against air. \
             Where the fade would be steeper than the steepest bank it widens, up to a limit; past that it stays steep (a gorge).",
        );
        ui.add_enabled(by_area, crate::debug::Slider::new(&mut c.fade_over, 0.0..=4.0).text("by area, all water fades over (widths)"))
            .on_hover_text("With sizes by area there are no ranks: every river's banks fade over this many of its widths.");
        for (k, rank) in super::network::RANKS.iter().enumerate() {
            ui.add_enabled(!by_area && reached[k], crate::debug::Slider::new(&mut c.fade_widths[k], 0.0..=4.0).text(format!("{rank}s fade over (widths)")));
        }
        ui.add(crate::debug::Slider::new(&mut c.fade_least, 0.0..=16.0).text("…at least (blocks)"));
        let banks = &mut c.banks;
        ui.horizontal_wrapped(|ui| {
            ui.label("banks fade:").on_hover_text("The same for every bank (the inside and outside of bends were cut on 2026-10-31).");
            for way in crate::world::FadeWay::ALL {
                ui.selectable_value(&mut banks.way, way, way.name());
            }
        });
        ui.add_enabled(banks.widest > 1.0, crate::debug::Slider::new(&mut banks.steepest, 0.2..=4.0).logarithmic(true).text("steepest bank (blocks up a block across)"))
            .on_hover_text("Where a bank would be steeper than this, its fade widens (up to \"widens up to\"). Greyed out: \"widens up to\" is 1, so it never widens.");
        ui.add(crate::debug::Slider::new(&mut banks.widest, 1.0..=6.0).text("widens up to (times)"));
        ui.add_enabled(banks.way == crate::world::FadeWay::Terraces, crate::debug::Slider::new(&mut banks.terrace, 1.0..=12.0).text("terrace steps (blocks)"));
        ui.checkbox(&mut banks.compressor, "compressor banks").on_hover_text(
            "Stage 1 of the water rework (docs/water-rework.md). Every river near a spot has its say: the land comes down to the\n\
             lowest of their banks, so two rivers' banks meet with no seam. The land beside any river's water stays at least flush\n\
             with it, coming down from there no steeper than the steepest bank, so no water stands against air where a river\n\
             steps down at a bend. The first dry block is the shore itself. Off: each spot takes its nearest river's bank alone.",
        );
        ui.add_enabled(banks.compressor, crate::debug::Slider::new(&mut banks.raised_steepest, 0.05..=4.0).logarithmic(true).text("raised banks no steeper than (blocks a block)"))
            .on_hover_text("Where the land beside a river is lower than its water, the bank is raised to the shore and eases back down to the land: no steeper than this, its fade widening with its height.");
        ui.add_enabled(banks.compressor, crate::debug::Slider::new(&mut banks.raised_widest, 1.0..=6.0).text("raised banks widen up to (times)"))
            .on_hover_text("How far a raised bank's fade may widen to stay that gentle (times its fade). Wider looks further for banks, so the world takes a little longer to make.");
        ui.add_space(4.0);
        ui.strong("Lakes");
        ui.small(
            "Round each lake the Rivers section found (on the broad ground, 32-block patches), the real ground is filled the \
             way water fills it: each dip to its lowest gap, whatever its shape, at the route's grain. Dips just beside the \
             water are filled to just under it, so water never stands against air.",
        );
        ui.add(crate::debug::Slider::new(&mut c.lake_reach, 0.0..=256.0).text("look round each lake (blocks)"))
            .on_hover_text("How far round each planned lake its real basin is looked for. Ponds in dips within this reach are found too.");
        ui.add(crate::debug::Slider::new(&mut c.pond_depth, 0.5..=10.0).text("a pond from (blocks deep)")).on_hover_text("A pool this deep at its deepest or more is kept; shallower dips stay dry.");
        ui.checkbox(&mut c.rise_pools, "ponds behind rises").on_hover_text(
            "Where a rise crosses a river's way, the river cuts through it up to the depth below; a deeper rise holds the water \
             back in a pond up to its lowest gap, and the river spills on over the top (David's marble). Off: every rise is cut \
             through, however deep.",
        );
        ui.add_enabled_ui(c.rise_pools, |ui| {
            ui.add(crate::debug::Slider::new(&mut c.cut_up_to, 0.0..=40.0).text("cut through rises up to (blocks)"))
                .on_hover_text("A rise this much higher than the water, or less, is cut through; a higher one makes a pond behind it.");
            ui.add(crate::debug::Slider::new(&mut c.pool_reach, 32.0..=2048.0).logarithmic(true).text("a pond spreads at most (blocks)"))
                .on_hover_text("How far a pond may spread from its river. One that would spread further (its water spilling off into the next valley, say) is cut through instead. Further takes longer to make.");
        });
        ui.add_space(4.0);
        ui.strong("Falls and deltas");
        ui.checkbox(&mut c.curtains, "waterfalls get a curtain of water").on_hover_text("A one-block sheet of water blocks down each waterfall's face, so it reads as falling water (nothing moves).");
        ui.add(crate::debug::Slider::new(&mut c.rapids_length, 1.0..=10.0).text("rapids: blocks of river per block of drop")).on_hover_text("Smaller steps than a waterfall are spread out as rapids, stepping down a block at a time.");
        ui.checkbox(&mut c.deltas, "deltas").on_hover_text("Where a river or great river meets the sea over flat ground, its last stretch fans into three channels.");
        ui.add_enabled_ui(c.deltas, |ui| {
            ui.add(crate::debug::Slider::new(&mut c.delta_flat, 0.001..=0.1).logarithmic(true).text("only where flatter than (slope)"));
            ui.add(crate::debug::Slider::new(&mut c.delta_width[0], 16.0..=256.0).text("a river's delta (blocks wide)"));
            ui.add(crate::debug::Slider::new(&mut c.delta_width[1], 16.0..=512.0).text("a great river's delta (blocks wide)"));
        });
    });
    fold_section(ui, "Temperature", |ui| climate_temperature(ui, &mut climate.temperature, compiler.climate.temperature().map(|t| (t.frozen_land, t.hottest_land, t.mean))));
    fold_section(ui, "Wind", |ui| {
        let wind = &mut climate.wind;
        ui.small(
            "From the temperature at sea level (so mountains don't push air away as if they were cold), blurred: it blows from \
             cold toward hot, turned to the right north of the equator and to the left south of it, so it swirls round hot \
             and cold spots and the two halves mirror each other. Mountains don't bend it; they act on the rain.",
        );
        ui.add(crate::debug::Slider::new(&mut wind.blur, 0.0..=60_000.0).text("blur (blocks)"))
            .on_hover_text("How far the temperature is blurred before the wind reads it: higher = broad, steady winds; lower = winds bending round every bay.");
        ui.add(crate::debug::Slider::new(&mut wind.turn, 0.0..=90.0).text("turned sideways (degrees)")).on_hover_text(
            "How far the wind is turned from straight cold-to-hot: 0 = straight at the warmth, 90 = running along the temperature bands.",
        );
        ui.add(crate::debug::Slider::new(&mut wind.even, 0.0..=1.0).text("even strength")).on_hover_text(
            "0 = as strong as the temperature's slope says (calm where it's even, as inside the ice caps); 1 = the same strength everywhere.",
        );
    });
    fold_section(ui, "Rain", |ui| {
        use super::rain::View;
        let rain = &mut climate.rain;
        ui.small(
            "Each spot looks upwind, following the wind, and reads the ground on the way: high ground upwind casts a rain \
             shadow, ground rising toward it from upwind (facing the wind) is wetter, and the further upwind the sea is, \
             the drier. Mountains act here, not on the wind.",
        );
        ui.horizontal_wrapped(|ui| {
            ui.label("Show:");
            let view = &mut compiler.climate.rain_view;
            ui.selectable_value(view, View::Rain, "rain").on_hover_text("Dry tan, through green, to wet blue.");
            ui.selectable_value(view, View::Shadow, "shadow").on_hover_text("How much the rain shadow dries each spot (white = all of it).");
            ui.selectable_value(view, View::Windward, "facing the wind").on_hover_text("How much facing the wind wets each spot (white = all of it).");
            ui.selectable_value(view, View::Inland, "far from the sea").on_hover_text("How far upwind the sea is, as a share of its reach (white = all the way or further).");
            if ui.small_button("on the map").on_hover_text("Shows rain over the map (the Show row at the top).").clicked() {
                compiler.climate.show = super::climate::Show::Rain;
            }
        });
        ui.add(crate::debug::Slider::new(&mut rain.reach, 1_000.0..=80_000.0).logarithmic(true).text("looks upwind (blocks)"))
            .on_hover_text("How far back along the wind each spot looks for high ground and the sea.");
        ui.add_space(4.0);
        ui.strong("Rain shadow");
        ui.add(crate::debug::Slider::new(&mut rain.shadow, 0.0..=1.0).text("how much it dries")).on_hover_text("0 = no rain shadows; 1 = a full shadow leaves no rain at all.");
        ui.add(crate::debug::Slider::new(&mut rain.shadow_height, 20.0..=800.0).logarithmic(true).text("full shadow (blocks higher)"))
            .on_hover_text("How much higher than here the ground upwind has to stand (nearer counting more) for the full shadow.");
        ui.label("The shadow: from none (left) to full (right), how much of it dries (up).");
        spline_box(ui, "rain shadow", &mut rain.shadow_curve, &super::rain::RainSettings::default().shadow_curve, "higher ground upwind", "share of the drying");
        ui.add_space(4.0);
        ui.strong("Facing the wind");
        ui.add(crate::debug::Slider::new(&mut rain.windward, 0.0..=1.0).text("how much wetter")).on_hover_text("0 = slopes facing the wind are no wetter; 1 = they're as wet as can be.");
        ui.add(crate::debug::Slider::new(&mut rain.windward_rise, 5.0..=500.0).logarithmic(true).text("full rise (blocks)"))
            .on_hover_text("How much the ground has to rise toward here, over the distance below, for all of it.");
        ui.add(crate::debug::Slider::new(&mut rain.windward_reach, 200.0..=10_000.0).logarithmic(true).text("over (blocks upwind)"))
            .on_hover_text("How far upwind the rise is measured from.");
        ui.add_space(4.0);
        ui.strong("Far from the sea");
        ui.add(crate::debug::Slider::new(&mut rain.inland, 0.0..=1.0).text("how much it dries")).on_hover_text("0 = no drier inland; 1 = no rain at all that far from the sea.");
        ui.add(crate::debug::Slider::new(&mut rain.inland_reach, 2_000.0..=100_000.0).logarithmic(true).text("all of it at (blocks upwind)"))
            .on_hover_text("How far back along the wind the sea has to be for all of it (nearer, less).");
        ui.add_space(4.0);
        ui.add(crate::debug::Slider::new(&mut rain.warmth, 0.0..=1.0).text("warm air carries more")).on_hover_text("How much drier the cold side of the world is (warm air holds more water). 0 = not at all.");
        ui.label("The look: the rain worked out (left, dry to wet) to the rain shown (up).");
        spline_box(ui, "rain look", &mut rain.look, &super::rain::RainSettings::default().look, "wetter", "rain");
    });
}

/// The Mineralogy panel [David, 2026-10-31: "minerology is gonna need a lot of space"]: which blocks
/// the ground is made of, after Climate (it reads the temperature and the rain) and before the World
/// Interpreter; then caves, still to come (David: they "might go in there after or have its own
/// page because it might be a pain").
fn mineralogy_panel(ui: &mut egui::Ui, compiler: &mut Compiler, look: &Look) {
    ui.heading("Mineralogy");
    // The placer's made fields, and the field view (stop 1 of docs/placer.md).
    super::placer_view::fields_section(ui, &mut compiler.settings.interpreter.mineralogy, &mut compiler.fields);
    // What's placed: the distributors (stop 3 of docs/placer.md), with the blocks' far colours.
    let palette = compiler.lab_palette.as_ref().map(|(_, palette)| palette.clone());
    let swatch = |block: crate::world::BlockId| {
        palette.as_ref().map(|palette| {
            let [r, g, b] = palette.top(block).map(|c| to_srgb(c * look.shade.top));
            Color32::from_rgb(r, g, b)
        })
    };
    super::placer_view::distributors_section(ui, &mut compiler.settings.interpreter.mineralogy, &mut compiler.fields, look.registry, &swatch);
    ui.add_space(6.0);
    ui.separator();
    ui.add_enabled_ui(false, |ui| {
        ui.strong("Caves");
        ui.small("Still to come, after forests: caves, cliffs and overhangs, all visible from a distance (docs/worldgen.md, \"Caves, cliffs and carving\").");
    });
}

/// The map's pixels are this many blocks across (the climate pixel is a whole number of them).
const MAP_PIXEL_BLOCKS: usize = 32;

/// The Climate panel's Temperature section; `shares` = the land frozen and hottest, and the world's average, once made.
fn climate_temperature(ui: &mut egui::Ui, t: &mut super::temperature::TemperatureSettings, shares: Option<(f32, f32, f32)>) {
    use super::temperature::{FROZEN, HOTTEST, Line, TemperatureSettings};
    ui.small(
        "Three lines across the world, each squiggled by its own seed: north of the north ice line and south of the south one \
         it's evenly cold; along the equator, hot. Between them the fade settles like a soap film stretched between two \
         lines, so the bands bend with both. -1 is frozen, +1 the hottest.",
    );
    if let Some((frozen, hottest, mean)) = shares {
        ui.label(format!("Land frozen: {:.0}%   hottest: {:.0}%   world average {mean:+.2}", frozen * 100.0, hottest * 100.0)).on_hover_text(format!(
            "How much of the land is frozen (below {FROZEN}) and how much the hottest (above {HOTTEST}), after the coast and\n\
             height have had their say. Tune for some ice land in every world, and some hottest land for the drylands."
        ));
    }
    let default = TemperatureSettings::default();
    let line = |ui: &mut egui::Ui, name: &str, line: &mut Line, (at, about): (&str, &str)| {
        ui.add_space(4.0);
        ui.strong(name);
        ui.add(crate::debug::Slider::new(&mut line.at, 0.0..=1.0).text(at)).on_hover_text(about);
        ui.add(crate::debug::Slider::new(&mut line.squiggle, 0.0..=20_000.0).text("squiggle (blocks)"))
            .on_hover_text("How far the line wanders north and south: the longer the string between its ends. 0 = straight.");
        ui.add(crate::debug::Slider::new(&mut line.wave, 1_000.0..=80_000.0).logarithmic(true).text("wave length (blocks)")).on_hover_text("How long its biggest waves are (finer ones ride on them).");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut line.seed).speed(0.2).prefix("seed "));
            ui.small("its own squiggle (mixed with the world's seed)");
        });
    };
    line(ui, "Ice lines (north and south alike)", &mut t.ice, (
        "from the pole (share of the map)",
        "How far each ice line is from its own edge of the map: the north line from the north edge, the south line from\n\
         the south. Each squiggles by its own seed (this one, and the next), so the caps aren't mirror images.",
    ));
    line(ui, "Equator", &mut t.equator, ("where (north edge to south)", "Where the equator runs, as a share of the way from the map's north edge to its south edge."));
    ui.add(crate::debug::Slider::new(&mut t.band, 0.0..=30_000.0).text("equator band (blocks)")).on_hover_text("How wide the evenly hot band along the equator is. 0 = just the line.");

    ui.add_space(4.0);
    ui.strong("How cold and hot");
    ui.add(crate::debug::Slider::new(&mut t.cold, -1.0..=1.0).text("ice caps")).on_hover_text("How cold it is past the ice lines, north and south alike.");
    ui.add(crate::debug::Slider::new(&mut t.hot, -1.0..=1.0).text("equator")).on_hover_text("How hot the equator (and its band) is.");
    ui.label("The fade: from an ice line (left) to the equator (right).");
    spline_box(ui, "temperature fade", &mut t.fade, &default.fade, "toward the equator", "share of the way to hot");

    ui.add_space(4.0);
    ui.strong("Sea and height");
    ui.add(crate::debug::Slider::new(&mut t.sea_pull, 0.0..=1.0).text("milder by the sea")).on_hover_text(
        "How much the sea pulls temperatures toward the world's average: coasts and sea neither as cold nor as hot. 0 = not at all.",
    );
    ui.add(crate::debug::Slider::new(&mut t.sea_reach, 200.0..=40_000.0).logarithmic(true).text("how far inland (blocks)"))
        .on_hover_text("How far inland the sea's mildness reaches, fading away.");
    ui.add(crate::debug::Slider::new(&mut t.cooling, 0.0..=0.5).text("colder per 100 blocks up")).on_hover_text("How much colder the ground is for every 100 blocks above the sea.");
}

/// The Interpreter's recipe: how the compiler's maps become the ground, worked out at every block. Its
/// settings are set where each part belongs in the chain: the ground's shape on the Topography panel,
/// its blocks on the Mineralogy panel [David, 2026-10-31: "move the features where they belong"]. This
/// panel shows the chain.
fn interpreter_panel(ui: &mut egui::Ui, compiler: &mut Compiler) {
    ui.heading("World Interpreter");
    ui.small(
        "The end of the chain: the recipe that gives every column of blocks its height, worked out at that block as the \
         world is made. The map shows it worked out at every block (or every few, zoomed out).",
    );
    ui.add(egui::Label::new(egui::RichText::new(CHAIN).monospace().small())).on_hover_text(
        "Moved here from the World Compiler: the compiler used to work this recipe out once per 32-block\n\
         pixel and hand over finished heights, which were then smoothed between pixels. The recipe has\n\
         sharp curves in it (a coast can drop 90 blocks within one pixel), and smoothing between finished\n\
         heights swings past every sharp step: one hump per pixel, in rows along every coast and cliff.\n\
         Now the smooth maps are blended to each block first, and the sharp curves applied after.",
    );
    ui.add_space(4.0);
    ui.small(
        "Its parts are set where they come in the chain: the ground's shape on the Topography panel (Land and sea, Ridgelines, \
         Mountains, Hills, Torn coast), its blocks on the Mineralogy panel. What it makes is seen on the map, with the readouts \
         on the right (Under the mouse, The world, Terrain lab, Show, Overlays, Cross-section).",
    );
    ui.add_space(4.0);
    ui.small("Presets, and back-to-defaults, are on the Landmass panel: they cover every panel.");
    ui.add_space(6.0);
    noise_maths_section(ui, compiler);
}

/// The A/B switch for the recipe's noise [David, 2026-11-01: whole numbers, so a GPU can project the
/// world exactly as the CPU does; "if i cant look at ten worlds and tell you which one was 64 bit
/// seeds and which was 32 bit, it doesnt matter"], and how long each takes.
fn noise_maths_section(ui: &mut egui::Ui, compiler: &mut Compiler) {
    use crate::worldgen::noise::NoiseMaths;
    fold_section(ui, "Noise maths (for now)", |ui| {
        ui.small(
            "Which maths the recipe's noise is worked out in, as the world is played: the hills, the crags, the ridges' wander, the \
             torn coast, mineralogy's wander. Whole numbers give the same answer on every chip, so a graphics card could make \
             the world exactly as the processor does. Their barcodes are new, so the patterns are new: compare a few worlds of \
             each, and see whether you can tell them apart. Saved with the preset and the world.",
        );
        let maths = &mut compiler.settings.interpreter.noise_maths;
        ui.horizontal(|ui| {
            ui.selectable_value(maths, NoiseMaths::Fractions, "fractions (as always)").on_hover_text("With sines and cosines, as the noise has always been.");
            ui.selectable_value(maths, NoiseMaths::WholeNumbers, "whole numbers").on_hover_text("32-bit whole numbers only: the same on every chip, CPU or GPU.");
        });
        let base = compiler.result.as_ref().map(|result| result.made.map.clone());
        ui.horizontal(|ui| {
            if ui.add_enabled(base.is_some(), egui::Button::new("Time both")).on_hover_text("Times the noise alone, and the recipe at a few thousand columns of this world, both ways (a second or so).").on_disabled_hover_text("Compile a world first.").clicked()
                && let Some(base) = &base
            {
                compiler.noise_timing = Some(time_noise_maths(base, &compiler.settings.interpreter, compiler.lab.mutes));
            }
        });
        if let Some(timing) = &compiler.noise_timing {
            ui.small(timing);
        }
    });
}

/// How long the noise takes, alone and in the recipe at a column of this world, in each maths.
fn time_noise_maths(base: &crate::world::CompiledMap, settings: &InterpreterSettings, mutes: crate::world::Mutes) -> String {
    use crate::worldgen::noise::{Fade, NoiseMaths, perlin_in};
    use std::hint::black_box;
    let calls = 200_000;
    let spots: Vec<(f32, f32)> = (0..calls).map(|k| ((k * 37 % 9_973) as f32 * 0.731, (k * 91 % 7_919) as f32 * 0.377)).collect();
    let alone = |maths: NoiseMaths| {
        let started = std::time::Instant::now();
        let sum: f32 = spots.iter().map(|&(x, y)| perlin_in(maths, 7, 3, Fade::Smooth, x, y)).sum();
        black_box(sum);
        started.elapsed().as_nanos() as f64 / calls as f64
    };
    // Columns over the land and sea, on a spiral out from the middle.
    let reach = base.maps.width as f32 * base.maps.blocks_per_pixel / 2.0 * 0.7;
    let columns: Vec<(f32, f32)> = (0..5_000).map(|k| {
        let (turn, out) = (k as f32 * 2.399_963, (k as f32 / 5_000.0).sqrt() * reach);
        (out * turn.cos(), out * turn.sin())
    }).collect();
    let recipe = |maths: NoiseMaths| {
        let map = base.with(InterpreterSettings { noise_maths: maths, ..settings.clone() }, mutes);
        let started = std::time::Instant::now();
        let sum: f32 = columns.iter().filter_map(|&(north, east)| map.height(north, east)).sum();
        black_box(sum);
        started.elapsed().as_nanos() as f64 / 1_000.0 / columns.len() as f64
    };
    let (fractions, whole) = (alone(NoiseMaths::Fractions), alone(NoiseMaths::WholeNumbers));
    let (in_fractions, in_whole) = (recipe(NoiseMaths::Fractions), recipe(NoiseMaths::WholeNumbers));
    format!(
        "The noise alone: fractions {fractions:.1} ns a call, whole numbers {whole:.1} ns.\nThe recipe at a column (the ground's height, before the rivers' carving, which has no noise): fractions {in_fractions:.2} µs, whole numbers {in_whole:.2} µs."
    )
}

/// Topography, Land and sea: the curves from the coast up the land and down the sea, the grade by
/// tilt, and the edge rise (the Interpreter panel's until 2026-10-31).
fn land_and_sea(ui: &mut egui::Ui, s: &mut InterpreterSettings, defaults: &InterpreterSettings) {
    fold_section(ui, "Land and sea", |ui| {
        ui.add(crate::debug::Slider::new(&mut s.deepest, 1.0..=127.0).text("deepest sea floor (Z)")).on_hover_text("The Z the deepest ocean reaches.");
        ui.add(crate::debug::Slider::new(&mut s.highlands, 129.0..=HIGHEST as f32).text("highlands (Z)"))
            .on_hover_text("The Z the highest land reaches before mountains are added (the top of the land curve).");
        ui.add(crate::debug::Slider::new(&mut s.ceiling, HIGHEST as f32..=TOP_OF_THE_WORLD as f32).text("ground never above (Z)")).on_hover_text(
            "The ground is cut off flat at this height. The ridgelines still measure their height up to Z 333,\n\
             so raising this changes nothing until something (the mountains' rise, peaks to come) goes past 333.\n\
             The world itself goes up to Z 1,023.",
        );
        ui.add_space(4.0);
        ui.label("Land: from the coast (left) to the bake's highest ground (right).");
        spline_box(ui, "land spline", &mut s.land, &defaults.land, "toward the highest ground", "up to the highlands");
        ui.label("Sea: from the coast (left) to the bake's deepest point (right).");
        spline_box(ui, "sea spline", &mut s.sea, &defaults.sea, "toward the deepest point", "down to the deepest floor");
        ui.small("A steep start to the sea curve makes cliffs at the coast; a gentle one, shallow shelves.");
        ui.add(crate::debug::Slider::new(&mut s.grade, 0.0..=1.0).text("grade by tilt")).on_hover_text(
            "Each plate's tilt steepens its high side (land climbs faster, the sea drops off close in) and softens\nits low side (wide plains, shallow shelves). 0 = no grading. (Bakes made before tilt was saved can't grade.)",
        );
        ui.add_space(4.0);
        ui.add(crate::debug::Slider::new(&mut s.coast_rise, 0.0..=80.0).text("edge rise (blocks)")).on_hover_text(
            "On a plate's high side, the land rises inland from the coast: this high where the plate is tilted up by 1\n\
             (more where it's tilted further; none on its low side). With a wide rise width and a fall-back, a gentle\n\
             hill following the plate's edge. 0 = off.",
        );
        ui.add(crate::debug::Slider::new(&mut s.coast_edge_width, 1.0..=20_000.0).logarithmic(true).text("rise width (blocks)"))
            .on_hover_text("How far inland from the coast it climbs to its full height. Narrow = a steep wall just inland; wide = a gentle rise.");
        ui.label("The climb: from the coast (left) to the rise width inland (right).");
        spline_box(ui, "coast edge spline", &mut s.coast_edge, &defaults.coast_edge, "inland", "share of the edge rise");
        ui.add(crate::debug::Slider::new(&mut s.coast_fall_back, 0.0..=40_000.0).logarithmic(true).text("falls back over (blocks)")).on_hover_text(
            "Past the rise width, how far further inland it eases back down to nothing: a hill along the plate's edge, with\n\
             the interior and the highest ground left as they were. 0 = it stays up all the way inland (the plate's whole\n\
             high side raised).",
        );
    });
}

/// Topography, Ridgelines: today's ridged "mountains", which David treats as hill country, the nest
/// the peaks stand in [David, 2026-10-27: renamed Ridgelines; "the ridges raise it to the highlands, the
/// additive mountains put the peaks across the same area"] (the Interpreter panel's until 2026-10-31).
fn ridgelines(ui: &mut egui::Ui, s: &mut InterpreterSettings, defaults: &InterpreterSettings) {
    fold_section(ui, "Ridgelines", |ui| {
        ui.small(
            "Ridged high ground from the squeeze map, which the peaks (Topography, Mountains) will stand in. Across: how \
             crushed the rock is, from none (left) to the most crushed rock in this bake (right). Up: how tall the \
             ridgelines are, as a share of the room up to Z 333. Keep the left part flat at the bottom to keep lightly \
             squeezed land as plains.",
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
    });
}

/// Topography, Torn coast (the Interpreter panel's until 2026-10-31).
fn torn_coast(ui: &mut egui::Ui, s: &mut InterpreterSettings, defaults: &InterpreterSettings) {
    fold_section(ui, "Torn coast", |ui| {
        ui.small(
            "Ponds, tide pools and inlets on land, islets and shoals at sea: in a band along the coastline, a pattern pushes \
             the ground back and forth across the waterline, in patches (some stretches of coast torn up, others clean). \
             The one part of the recipe meant to move the coast.",
        );
        let t = &mut s.torn_coast;
        let d = &defaults.torn_coast;
        ui.add(crate::debug::Slider::new(&mut t.amount, 0.0..=24.0).text("amount (blocks)")).on_hover_text(
            "How far the ground is pushed up and down at the coastline. Land a few blocks above the sea is torn by a little;\n\
             higher ground (an edge rise, cliffs) needs more to reach the water. 0 = off.",
        );
        ui.add(crate::debug::Slider::new(&mut t.inland, 0.0..=4_000.0).text("reach inland (blocks)"))
            .on_hover_text("How far in from the coast the band reaches (it fades out toward its edge): how far inland the ponds go.");
        ui.add(crate::debug::Slider::new(&mut t.out_to_sea, 0.0..=4_000.0).text("reach out to sea (blocks)"))
            .on_hover_text("How far out from the coast the band reaches: how far out the islets and shoals go.");
        ui.add(crate::debug::Slider::new(&mut t.balance, -1.0..=1.0).text("water ← balance → land")).on_hover_text(
            "-1 = mostly pushed down (ponds, inlets, a drowned, broken shore); 1 = mostly pushed up (islets, shoals,\n\
             land reaching out into the sea); 0 = even.",
        );
        ui.add(crate::debug::Slider::new(&mut t.patches, 0.0..=1.0).text("how much is torn")).on_hover_text(
            "How much of the coast is torn up: 1 = all of it, 0 = none. Between, a slow pattern picks the stretches,\n\
             and the rest of the coast stays clean.",
        );
        ui.add(crate::debug::Slider::new(&mut t.patch_size, 500.0..=40_000.0).logarithmic(true).text("stretch size (blocks)"))
            .on_hover_text("About how long the torn and clean stretches of coast are.");
        noise_box(ui, "torn coast", &mut t.noise, &d.noise, 16.0..=2_000.0);
    });
}

/// Topography, Hills (ground detail until 2026-10-28; the Interpreter panel's until 2026-10-31).
fn hills(ui: &mut egui::Ui, s: &mut InterpreterSettings, defaults: &InterpreterSettings) {
    fold_section(ui, "Hills", |ui| {
        ui.add(crate::debug::Slider::new(&mut s.detail, 0.0..=40.0).text("hills (blocks)")).on_hover_text(
            "Hills and bumps on the ground: a Perlin field laid on the ground, this many blocks up and down (called\n\
             ground detail until 2026-10-28). Only in the world (and the close-up): finer than one of this map's pixels.",
        );
        ui.checkbox(&mut s.detail_keeps_coast, "keeps off the waterline").on_hover_text(
            "On: near the waterline the hills are held back so they never cross it, and moving the hills never\n\
             changes the coast (that's the torn coast's job). Off: the old way, where they tore coasts up. For A/B.",
        );
        masked_noise(ui, "hills", &mut s.detail_noise, &defaults.detail_noise, 16.0..=2_000.0, &mut s.detail_mask);
    });
}

impl Compiler {
    /// The recipe's map with the river stage's rivers and lakes carved in (made again only when
    /// either changes), or as it is before there are any; left out if the Show switch says so.
    fn with_rivers(&mut self, map: &Arc<crate::world::CompiledMap>) -> Arc<crate::world::CompiledMap> {
        let Some(carving) = self.climate.carving().cloned() else { return map.clone() };
        let hidden = self.lab.mutes.rivers;
        let key = (Arc::as_ptr(map) as usize, Arc::as_ptr(&carving) as usize, hidden);
        if let Some((made_for, carved)) = &self.rivers_view
            && *made_for == key
        {
            return carved.clone();
        }
        let mut carved = map.with_rivers(carving);
        carved.mutes.rivers = hidden;
        let carved = Arc::new(carved);
        self.rivers_view = Some((key, carved.clone()));
        carved
    }

    /// The map with the Climate tab's temperature and rain in it, for the world to keep (mineralogy
    /// reads them at every column), blended from the climate pixels to the map's own: the temperature
    /// as at sea level (each block cools for its own height), the rain as it is. Unchanged if the
    /// climate isn't worked out yet: the world then has none (mild and middling wet everywhere).
    fn with_climate(&self, map: Arc<crate::world::CompiledMap>) -> Arc<crate::world::CompiledMap> {
        let (Some(ground), Some(temperature), Some(rain)) = (self.climate.ground(), self.climate.temperature(), self.climate.rain()) else { return map };
        let (cw, w) = (ground.width, map.maps.width);
        if temperature.width != cw || rain.width != cw || cw == 0 {
            return map;
        }
        // A climate field at the centre of the map's pixel `i`, blended between climate pixels.
        let per = ground.pixels as f32;
        let blend = |field: &dyn Fn(usize) -> f32, i: usize| {
            let (x, y) = (((i % w) as f32 + 0.5) / per - 0.5, ((i / w) as f32 + 0.5) / per - 0.5);
            let (x0, y0) = (x.floor(), y.floor());
            let (tx, ty) = (x - x0, y - y0);
            let at = |c: f32, r: f32| field(r.clamp(0.0, cw as f32 - 1.0) as usize * cw + c.clamp(0.0, cw as f32 - 1.0) as usize);
            (at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx) * (1.0 - ty) + (at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx) * ty
        };
        let mut maps = (*map.maps).clone();
        maps.temperature = (0..w * w).map(|i| blend(&|k| temperature.at_sea[k], i)).collect();
        maps.rain = (0..w * w).map(|i| blend(&|k| rain.rain[k].unwrap_or(0.5), i)).collect();
        maps.cooling = self.settings.climate.temperature.cooling;
        let mut with = crate::world::CompiledMap::new(Arc::new(maps), map.seed, map.settings.clone());
        // And the rivers and lakes carved in.
        if let Some(carving) = self.climate.carving() {
            with.rivers = carving.clone();
        }
        with.mutes = map.mutes;
        Arc::new(with)
    }

    /// "Render World and Spawn": the compiled maps, with the Interpreter's recipe, handed over to
    /// start a new world, leaving out what the Show switches leave out [David, 2026-10-27: "lets
    /// have the spawning follow the show switches"].
    pub fn press_render(&mut self, spawn: IVec2) {
        let (Some(bake), Some(result)) = (&self.bake, &self.result) else { return };
        let name = if self.world_name.trim().is_empty() { bake.name.clone() } else { self.world_name.clone() };
        let seed = u64::from_str_radix(&bake.info.seed, 16).unwrap_or(0);
        let map = &result.made.map;
        let map = if self.lab.mutes == Mutes::default() { map.clone() } else { Arc::new(map.with(map.settings.clone(), self.lab.mutes)) };
        let map = self.with_climate(map);
        self.render = Some((name, seed, map, spawn));
        self.picking = false;
        self.space_pick = false;
    }
}

/// White in the elevation picture: the world's old top, kept when it was raised to 1,024
/// (2026-10-27) so the picture looks as before.
const PICTURE_WHITE: f32 = 399.0;

/// The compiled world as a grey picture: black = Z 0, white = Z 399 (higher is white too).
fn elevation_picture(compiled: &Compiled) -> Picture {
    let pixels = compiled
        .z
        .iter()
        .map(|&z| {
            let v = (z as f32 / PICTURE_WHITE * 255.0).min(255.0).round() as u8;
            [v, v, v, 255]
        })
        .collect();
    Picture { width: compiled.width, height: compiled.width, pixels }
}

/// A spline box: the curve over a grid, with its dots. Drag a dot to move it (the end dots stay
/// at the edges and only move up and down); double-click to add a dot; right-click a middle dot to
/// take it away.
/// A layer of the ground that's a Perlin field (hills): its noise, as a Graphics Kit
/// Perlin layer has it, and its mask, each folded away until opened.
/// A recipe part's Perlin noise settings, folded away under one heading.
fn noise_box(ui: &mut egui::Ui, name: &str, noise: &mut PerlinSettings, default: &PerlinSettings, sizes: std::ops::RangeInclusive<f32>) {
    egui::CollapsingHeader::new(format!("{name}: Perlin noise")).id_salt((name, "noise")).show(ui, |ui| {
        ui.small(format!("The same Perlin noise as a Graphics Kit Perlin layer, laid on the ground as {name}."));
        super::view::perlin_rows(ui, noise, sizes, None);
        if ui.small_button("This noise back to defaults").clicked() {
            *noise = default.clone();
        }
    });
}

fn masked_noise(ui: &mut egui::Ui, name: &str, noise: &mut PerlinSettings, default: &PerlinSettings, sizes: std::ops::RangeInclusive<f32>, mask: &mut Mask) {
    noise_box(ui, name, noise, default, sizes);
    // The mask in the open, not folded under a heading of its own [David, 2026-10-27: it looked
    // lost two folds down].
    ui.add_space(4.0);
    ui.label(format!("Where {name} show (the mask):")).on_hover_text(format!(
        "Fades {name} in and out across the world: pick a map to follow (the overlays show them), and the curve\n\
         says how much shows (up) for each value of it (across)."
    ));
    ui.horizontal_wrapped(|ui| {
        ui.label("follows:");
        for source in MaskSource::ALL {
            ui.selectable_value(&mut mask.source, source, source.name());
        }
    });
    // Everything below is always there, greyed out until it applies (David's convention).
    let everywhere = mask.source == MaskSource::Everywhere;
    if everywhere {
        ui.small(format!("Everywhere: {name} show the same all over. Pick a map to follow to fade them by the curve below."));
    }
    ui.add_enabled_ui(!everywhere, |ui| {
        ui.checkbox(&mut mask.invert, "invert").on_hover_text("Read the map the other way round: what was on the left is on the right.");
        ui.add_enabled(mask.source == MaskSource::Coast, crate::debug::Slider::new(&mut mask.reach, 250.0..=16_000.0).logarithmic(true).text("reach from the coast (blocks)"))
            .on_hover_text("When following coast distance: how far from the coastline the curve's edges are. The left edge is this far\nout to sea, the right this far inland.");
        let (left, right) = mask.source.ends();
        let (left, right) = if mask.invert { (right, left) } else { (left, right) };
        ui.label("The falloff: how much shows (up) along the map it follows (across).");
        spline_box(ui, &format!("{name} mask"), &mut mask.curve, &Spline::default(), &format!("{left}  →  {right}"), &format!("how much {name} shows"));
    });
}

/// The rivers' profiles, drawn like David's drawing (a square a block; tan ground, blue water),
/// running straight, their size and shape not wandering: by area, 1, 4, 16, 64 and 256 blocks of water
/// on flat ground, halfway and on steep; by rank, each rank at the start, middle and end of its band.
fn profiles_preview(ui: &mut egui::Ui, c: &super::carving::CarvingSettings) {
    let (tan, blue) = (Color32::from_rgb(168, 148, 44), Color32::from_rgb(20, 40, 235));
    // Each row: its label, and three sizes (width, depth, shape).
    type Size = (f32, f32, super::carving::Shape);
    let rows: Vec<(String, [Size; 3])> = if c.by_area {
        [1.0f32, 4.0, 16.0, 64.0, 256.0].iter().map(|&water| (format!("{water:.0} blocks of water on flat ground, halfway and on steep"), [0.0, 0.5, 1.0].map(|steep| super::carving::size_by_area(c, water, steep)))).collect()
    } else {
        super::network::RANKS.iter().enumerate().map(|(k, rank)| (format!("{rank}s, as they start, halfway and as they join a bigger one"), [0.0, 0.5, 1.0].map(|share| super::carving::size_in_band(c, k + 1, share)))).collect()
    };
    for (label, sizes) in rows {
        // Two blocks of bank each side, and a block's gap between.
        let blocks: f32 = sizes.iter().map(|s| s.0.round().max(1.0) + 5.0).sum();
        let pixel = (ui.available_width() / blocks).floor().clamp(1.0, 8.0);
        let rows = sizes.iter().map(|s| s.1.round()).fold(1.0, f32::max) + 2.0;
        ui.small(format!("{label} ({pixel:.0} pixels a block)"));
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), rows * pixel), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let mut x = rect.left();
        for (width, depth, shape) in sizes {
            let across = width.round().max(1.0);
            painter.rect_filled(egui::Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new((across + 4.0) * pixel, rows * pixel)), 0.0, tan);
            for column in 0..across as usize {
                let off = (column as f32 + 0.5 - across / 2.0) / (across / 2.0);
                let deep = crate::world::depth_across(depth, shape.edge, shape.flat, shape.bowl, off);
                painter.rect_filled(egui::Rect::from_min_size(egui::pos2(x + (2.0 + column as f32) * pixel, rect.top()), Vec2::new(pixel, deep * pixel)), 0.0, blue);
            }
            x += (across + 5.0) * pixel;
        }
        ui.add_space(4.0);
    }
}

pub(super) fn spline_box(ui: &mut egui::Ui, name: &str, spline: &mut Spline, default: &Spline, across: &str, up: &str) {
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
    // The hollow handles, one on each stretch between two dots: drag up or down to bow it,
    // right-click to straighten it.
    for k in 0..spline.dots.len() - 1 {
        let x = (spline.dots[k].0 + spline.dots[k + 1].0) / 2.0;
        let at = to_screen(x, spline.at(x));
        let response = ui.interact(egui::Rect::from_center_size(at, Vec2::splat(14.0)), ui.id().with((name, "bend", k)), egui::Sense::click_and_drag());
        let lit = response.hovered() || response.dragged();
        painter.circle(at, if lit { 5.0 } else { 3.5 }, Color32::from_gray(22), egui::Stroke::new(1.5, if lit { Color32::WHITE } else { Color32::from_rgb(255, 230, 120) }));
        let bend = spline.bends.get(k).copied().unwrap_or(0.0);
        let mut wanted = bend;
        if response.dragged() {
            wanted = (bend - response.drag_delta().y / inner.height() * 2.0).clamp(-1.0, 1.0);
        }
        if response.secondary_clicked() {
            wanted = 0.0;
        }
        if wanted != bend {
            spline.bends.resize(spline.dots.len() - 1, 0.0);
            spline.bends[k] = wanted;
        }
        if lit {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
    }
    if let Some(k) = remove {
        spline.dots.remove(k);
        // Its two stretches become one, keeping the first's bend.
        if k < spline.bends.len() {
            spline.bends.remove(k);
        }
    }
    // Double-click to add a dot where you clicked (not right on top of another).
    if background.double_clicked()
        && let Some(pointer) = background.interact_pointer_pos()
    {
        let (x, y) = from_screen(pointer);
        if spline.dots.iter().all(|&(dx, _)| (dx - x).abs() > 0.02) {
            let k = spline.dots.iter().position(|&(dx, _)| dx > x).unwrap_or(spline.dots.len());
            spline.dots.insert(k, (x, y));
            // The stretch it splits becomes two, both as it was bowed.
            if k > 0 && k - 1 < spline.bends.len() {
                spline.bends.insert(k, spline.bends[k - 1]);
            }
        }
    }
    // All straight is kept as none, so an untouched curve still matches its saved preset.
    if spline.bends.iter().all(|&b| b == 0.0) {
        spline.bends.clear();
    }
    ui.horizontal_wrapped(|ui| {
        ui.small(format!(
            "{} dots: double-click to add, drag to move, right-click to remove. Hollow handles: drag up or down to bow, right-click to straighten",
            spline.dots.len()
        ));
        if ui.small_button("reset").clicked() {
            *spline = default.clone();
        }
    });
}

fn readouts(ui: &mut egui::Ui, compiler: &mut Compiler) {
    lab_readouts(ui, compiler);
    fold_section(ui, "Under the mouse", |ui| {
        under_the_mouse(ui, compiler);
        super::placer_view::readout(ui, &compiler.settings.interpreter.mineralogy, &compiler.fields);
    });
    let Some(result) = &compiler.result else { return };
    fold_section(ui, "The world", |ui| {
        let highest = result.made.z.iter().copied().max().unwrap_or(0);
        let lowest = result.made.z.iter().copied().min().unwrap_or(0);
        ui.label(format!("land: {:.1}% of the world", result.made.land_share * 100.0));
        ui.label(format!("highest ground Z {highest}, deepest sea floor Z {lowest}"));
        ui.small(format!("Sea level Z {SEA_LEVEL}; ground never above Z {:.0}; the world's top is Z {TOP_OF_THE_WORLD}.", compiler.settings.interpreter.ceiling));
    });
}

/// What's under the mouse: the block in the close-up, or the map's pixel, and what the maps say there.
fn under_the_mouse(ui: &mut egui::Ui, compiler: &mut Compiler) {
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
            if let Some(ground) = compiler.climate.ground()
                && let Some(z) = ground.at_map_pixel(col, row)
            {
                ui.small("From the climate (its pixel):");
                ui.small(format!("broad ground Z {z:.0}")).on_hover_text("The ground the Climate reads: the recipe without crags, summit crags, hills or torn coast.");
                if let Some(t) = compiler.climate.temperature().and_then(|temperature| temperature.at_map_pixel(ground, col, row)) {
                    ui.small(format!("temperature {t:+.2} ({})", super::temperature::describe(t))).on_hover_text("-1 frozen to +1 the hottest.");
                }
                if let Some(wind) = compiler.climate.wind()
                    && wind.width == ground.width
                {
                    let (c, r) = (col / ground.pixels, row / ground.pixels);
                    if c < wind.width && r < wind.width {
                        let i = r * wind.width + c;
                        let [east, south] = wind.way[i];
                        // Named by where it comes from, as winds are: blowing south is a north wind.
                        let from = compass_point(-east, -south);
                        ui.small(format!("wind from the {from}, strength {:.2}", wind.strength[i])).on_hover_text("Winds are named for where they blow from.");
                    }
                }
                if let Some(rain) = compiler.climate.rain()
                    && rain.width == ground.width
                {
                    let (c, r) = (col / ground.pixels, row / ground.pixels);
                    if c < rain.width && r < rain.width
                        && let Some(wet) = rain.rain[r * rain.width + c]
                    {
                        let i = r * rain.width + c;
                        ui.small(format!("rain {wet:.2}")).on_hover_text("0 dry to 1 the wettest.");
                        ui.small(format!(
                            "because: shadow {:.0}%, facing the wind {:.0}%, far from the sea {:.0}%",
                            rain.shadow[i] * 100.0,
                            rain.windward[i] * 100.0,
                            rain.inland[i] * 100.0
                        ))
                        .on_hover_text("Each part's say here, before its strength slider: how much the rain shadow dries, facing the wind wets, and distance from the sea dries.");
                    }
                }
                if let Some(flow) = compiler.climate.flow()
                    && flow.width == width
                {
                    if flow.lake[i] > 0.0 {
                        ui.small(format!("a planned lake (on the broad ground) {:.0} deep, its surface at Z {:.0}", flow.lake[i], flow.surface[i]));
                    }
                    ui.small(format!("water flowing through: {:.1} units", flow.flow[i])).on_hover_text("1 unit = a patch of land (more with snowmelt and slope-foot springs); it adds up downhill.");
                }
                // The river here, if any: its rank (from branches joining), its bed and the step below.
                if let Some(network) = compiler.climate.network()
                    && network.width == width
                    && network.rank[i] > 0
                {
                    let rank = network.rank[i] as usize;
                    let carving = &compiler.settings.climate.carving;
                    let (narrowest, widest) = (super::carving::size_in_band(carving, rank, 0.0).0, super::carving::size_in_band(carving, rank, 1.0).0);
                    if carving.by_area {
                        ui.small(format!("a river (sized by area: see \"sizes in the close-up\"), stream order {}, its bed at Z {:.0}", network.order[i], network.bed[i]));
                    } else {
                        ui.small(format!("a {} ({narrowest:.0}-{widest:.0} blocks wide), stream order {}, its bed at Z {:.0}", super::network::RANKS[rank - 1], network.order[i], network.bed[i]));
                    }
                    let step = network.drop[i];
                    if step > 0.0 {
                        let kind = if step >= compiler.settings.climate.network.fall_from { "a waterfall" } else { "rapids" };
                        ui.small(format!("{kind} just below: {step:.0} blocks down"));
                    }
                }
            }
            if let Some(painted) = &compiler.lab.overlays.painted
                && painted.width == width
            {
                ui.small("From the overlays (this pixel):");
                let d = result.made.map.maps.coast_distance[i];
                ui.small(if d >= 0.0 { format!("{d:.0} blocks inland") } else { format!("{:.0} blocks out to sea", -d) });
                ui.small(format!("mountain share {:.0}%", painted.share[i] * 100.0))
                    .on_hover_text("How much of the room above the ground the mountains take here, before ridges carve them.");
                if let Some(grain) = &painted.grain
                    && painted.share[i] > 0.02
                {
                    ui.small(format!("string {:.0}% (knot {:.0}%)", grain.string[i] * 100.0, (1.0 - grain.string[i]) * 100.0))
                        .on_hover_text("How clearly the mountains round here run one way (string) rather than every which way (knot).");
                }
            }
        }
        _ => {
            ui.small("Point at the world.");
        }
    }
}

/// The terrain lab's part of the right-hand panel: what the close-up is showing, what it cost, the
/// layer switches, and the cross-section.
fn lab_readouts(ui: &mut egui::Ui, compiler: &mut Compiler) {
    fold_section(ui, "Terrain lab", |ui| {
        ui.horizontal(|ui| {
            ui.label("Sharpness:");
            for (sharper, name) in [(-1, "½×"), (0, "1×"), (1, "2×"), (2, "4×"), (3, "8×"), (4, "16×")] {
                ui.selectable_value(&mut compiler.lab.sharper, sharper, name).on_hover_text(
                    "Samples per screen point. Sharper shows finer ground detail from further out; each step up takes about 4 times as long; 16× is for screenshots, ½× for zoomed right out.",
                );
            }
        });
        // The close-up shows the world's blocks (mineralogy's choice, in the live climate) in the
        // distant terrain's colours; lit as it lights them, or flat.
        let mut lit = !compiler.lab_unlit;
        ui.checkbox(&mut lit, "lit").on_hover_text(
            "The close-up shows the blocks the world will have (in the Climate tab's live climate), in the colours the distant terrain gives them.\n\
             Lit: with its hill shading and height tint, from the same settings (Render page, Distant terrain, Light), so hills read as they will from the air.\n\
             Unlit: each block's plain colour, to judge where the blocks fall.",
        );
        compiler.lab_unlit = !lit;
        if let Some(frame) = &compiler.lab.frame {
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
    });
    fold_section(ui, "Show", |ui| {
        let m = &mut compiler.lab.mutes;
        ui.horizontal_wrapped(|ui| {
            for (name, muted, about) in [
                ("edge rise", &mut m.raised_coast, "The land rising inland from the coast on a plate's high side."),
                ("rise", &mut m.lift, "The mountains' rise (Topography, Mountains): land raised round their points, like a tablecloth draped over them."),
                ("ridgelines", &mut m.mountains, "The ridgelines from the squeeze map (with their ridges)."),
                ("ridge carving", &mut m.ridges, "The ridge pattern carved into the ridgelines. Off: the ridgelines stay, as smooth domes (switch ridgelines off to take the high ground away altogether)."),
                ("torn coast", &mut m.torn_coast, "Ponds, tide pools, inlets, islets and shoals along the coast."),
                ("hills", &mut m.detail, "The hills laid on the ground (World Interpreter, Hills)."),
                ("peaks", &mut m.peaks, "The peaks standing at the mountains' points (Topography, Mountains, Peaks)."),
                ("crags", &mut m.crags, "The crags carved into the peaks (off: smooth peaks)."),
                ("summit crags", &mut m.summit_crags, "The finer crags pressed into the ground high up (Topography, Mountains, Peaks)."),
                (
                    "rivers and lakes",
                    &mut m.rivers,
                    "The Climate tab's water: the rivers, lakes and ponds, with their channels and banks (Carving). Off: the ground\n\
                     as it was before they were carved, and only the sea's water, to tell the two apart. (The rivers aren't made\n\
                     again: Climate reads the ground without them either way.)",
                ),
            ] {
                let mut shown = !*muted;
                if ui.checkbox(&mut shown, name).on_hover_text(about).changed() {
                    *muted = !shown;
                }
            }
        });
    })
    .on_hover_text(
        "Leave parts of the Interpreter's recipe out, to see what each one does.\nRender World and Spawn leaves out the same: the world says which along the top of the screen,\nand remembers them when saved.",
    );
    fold_section(ui, "Overlays", |ui| {
        let o = &mut compiler.lab.overlays;
        ui.horizontal_wrapped(|ui| {
            let s = &mut o.shown;
            for (name, shown, about) in [
                ("plates", &mut s.plates, "Each plate in its own colour, with its outline."),
                ("squeeze", &mut s.squeeze, "Crushed rock in red, stretched rock in blue: the stronger, the deeper the colour."),
                ("ridgelines", &mut s.mountains, "Where the ridgelines rise (orange: the deeper, the taller their curve makes them)."),
                ("range spines", &mut s.spines, "The crest line of each mountain range: where its mountains are strongest, across the range."),
                ("coast distance", &mut s.coast, "The coastline (cyan), and lines every so many blocks inland (white) and out to sea (pale blue)."),
                ("squeeze direction", &mut s.direction, "Which way the rock was squeezed: longer strokes, harder squeezing. Ridges line up across the strokes."),
                ("range grain", &mut s.grain, "Which way the mountains run (the Topography panel's mountain grain): longer strokes where it's clearer."),
                ("string or knot", &mut s.knots, "Mountains that clearly run one way, strings, in blue; knots and bundles with no one direction in yellow."),
            ] {
                ui.checkbox(shown, name).on_hover_text(about);
            }
        });
        let any = o.shown != super::overlays::Shown::default();
        ui.add_enabled(any, crate::debug::Slider::new(&mut o.opacity, 0.1..=1.0).text("overlay strength")).on_hover_text("How strongly the overlays cover the map.");
        ui.add_enabled(o.shown.coast, crate::debug::Slider::new(&mut o.band, 250.0..=8_000.0).text("coast lines every (blocks)"))
            .on_hover_text("With the coast distance overlay: the blocks between its lines.");
    })
    .on_hover_text(
        "The compiler's maps drawn over the map and the close-up, one value per map pixel (32 blocks): the areas\n\
         and lines hidden in them. Later, these are what the scatterers (second-layer mountains) will be driven by.",
    );
    fold_section(ui, "Cross-section", |ui| {
        if compiler.lab.section.is_some() {
            if ui.small_button("Clear the cross-section").on_hover_text("Or right-click the map.").clicked() {
                compiler.lab.section = None;
            }
        } else {
            ui.small("Shift + drag across the map to draw one: its side view shows under the map, live.");
        }
    });
}

/// What the close-up needs from the game to show the world as it will be: the blocks (and their far
/// colours), and how the distant terrain lights the ground (the same settings: `distant::Shade`).
pub struct Look<'a> {
    pub registry: &'a crate::data::BlockRegistry,
    pub shade: crate::render::distant::Shade,
    /// The distance the hill shading's slope is measured over (the distant terrain's finest cell).
    pub finest_cell: f32,
}

/// The Climate tab's live temperature and rain at any spot, as the world would get them at spawn:
/// read straight from the climate pixels (blended), so a climate slider shows in the close-up at
/// once, without making a copy of the maps.
struct LiveClimate {
    temperature: Arc<super::temperature::Temperature>,
    rain: Arc<super::rain::Rain>,
    /// Climate pixels across, the map's pixels across, blocks per map pixel, map pixels per climate
    /// pixel, and how much colder per 100 blocks above the sea.
    width: usize,
    map_width: f32,
    blocks: f32,
    per: f32,
    cooling: f32,
}

impl LiveClimate {
    fn new(compiler: &Compiler, map: &crate::world::CompiledMap) -> Option<Self> {
        let (ground, temperature, rain) = (compiler.climate.ground()?, compiler.climate.temperature()?, compiler.climate.rain()?);
        let width = ground.width;
        (width > 0 && temperature.width == width && rain.width == width).then(|| Self {
            temperature: temperature.clone(),
            rain: rain.clone(),
            width,
            map_width: map.maps.width as f32,
            blocks: map.maps.blocks_per_pixel,
            per: ground.pixels as f32,
            cooling: compiler.settings.climate.temperature.cooling,
        })
    }

    /// Temperature (colder the higher `z` stands above the sea) and rain at a spot.
    fn at(&self, north: f32, east: f32, z: f32) -> (f32, f32) {
        let half = self.map_width / 2.0;
        // In climate pixels, counting their centres.
        let (x, y) = ((east / self.blocks + half) / self.per - 0.5, (half - north / self.blocks) / self.per - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (tx, ty) = (x - x0, y - y0);
        let last = self.width as f32 - 1.0;
        let index = |c: f32, r: f32| r.clamp(0.0, last) as usize * self.width + c.clamp(0.0, last) as usize;
        let blend = |field: &dyn Fn(usize) -> f32| {
            (field(index(x0, y0)) * (1.0 - tx) + field(index(x0 + 1.0, y0)) * tx) * (1.0 - ty) + (field(index(x0, y0 + 1.0)) * (1.0 - tx) + field(index(x0 + 1.0, y0 + 1.0)) * tx) * ty
        };
        let above_sea = (z - SEA_LEVEL as f32).max(0.0);
        let temperature = blend(&|i| self.temperature.at_sea[i]) - self.cooling * above_sea / 100.0;
        (temperature, blend(&|i| self.rain.rain[i].unwrap_or(0.5)))
    }
}

/// What the close-up's colours were worked out for: when any of it changes, it's painted again (the
/// heights are kept).
#[derive(Clone, PartialEq)]
struct PaintKey {
    frame: u64,
    mineralogy: crate::world::MineralogySettings,
    climate: u64,
    lit: bool,
    shores: bool,
    sizes: bool,
    blocks: u64,
    shade: crate::render::distant::Shade,
    /// What the map shows, and the field view's or the depth sweep's look, when it shows one
    /// (`placer_view.rs`).
    field: Option<(super::placer_view::Shows, super::placer_view::FieldLook)>,
}

/// Linear light (0 to 1) as a screen's 0 to 255.
fn to_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let v = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (v * 255.0).round() as u8
}

/// A close-up's picture as the world will look [David and Counsel, 2026-10-30]: each sample the
/// block on top (mineralogy's choice, in the live climate) in its far colour, the palette the distant
/// terrain uses, water where the near ground has it; lit, if `lit`, with the distant terrain's hill
/// shading and height tint (the same sums and settings as lighting.wesl). Past the map is `BEYOND`.
/// With `shores`, the dry block beside a river's or lake's water is coloured by how far it stands
/// above the water (stage 0 of `docs/water-rework.md`).
#[allow(clippy::too_many_arguments)]
/// The close-up's colour for a river's water across (`world::water_across`), evenly by ratio: yellow
/// at a block, green at 4, sky blue at 16, violet at 64, magenta at 256 and more.
fn water_tint(across: f32) -> [u8; 3] {
    const STOPS: [[f32; 3]; 5] = [[250.0, 230.0, 60.0], [60.0, 220.0, 120.0], [40.0, 170.0, 240.0], [110.0, 80.0, 240.0], [235.0, 60.0, 200.0]];
    let t = (across.max(1.0).log2() / 2.0).clamp(0.0, 4.0);
    let k = (t.floor() as usize).min(3);
    let f = t - k as f32;
    let (a, b) = (STOPS[k], STOPS[k + 1]);
    [0, 1, 2].map(|c| (a[c] + (b[c] - a[c]) * f) as u8)
}

fn block_picture(frame: &lab::Frame, map: &crate::world::CompiledMap, climate: Option<&LiveClimate>, look: &Look, palette: &crate::render::distant::Palette, wanted: &PaintKey) -> Vec<Color32> {
    let (lit, shores, sizes) = (wanted.lit, wanted.shores, wanted.sizes);
    let request = &frame.request;
    let (cols, rows, step) = (request.cols, request.rows, request.step);
    let blocks = crate::world::TerrainBlocks::from_registry(look.registry);
    let mineralogy = map.mineralogy(&blocks);
    let shade = &look.shade;
    let sea = SEA_LEVEL as f32;
    let z = |col: usize, row: usize| frame.heights[row.min(rows - 1) * cols + col.min(cols - 1)];
    let mut pixels = vec![Color32::BLACK; cols * rows];
    // Each sample's river or lake water (its top; 0 for none).
    let mut tops = vec![0i32; cols * rows];
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = rows.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        let each = rows_each * cols.max(1);
        for (band, (part, tops)) in pixels.chunks_mut(each).zip(tops.chunks_mut(each)).enumerate() {
            let (z, mineralogy) = (&z, &mineralogy);
            scope.spawn(move || {
                for (k, pixel) in part.iter_mut().enumerate() {
                    let (col, row) = (k % cols, band * rows_each + k / cols);
                    let here = z(col, row);
                    if here.is_nan() {
                        let [r, g, b] = BEYOND;
                        *pixel = Color32::from_rgb(r, g, b);
                        continue;
                    }
                    let north = request.north - (row as f32 + 0.5) * step;
                    let east = request.east + (col as f32 + 0.5) * step;
                    // The sample as the world fills it: water (the sea's, or a river's or lake's
                    // standing over the ground here), and what the ground is made of.
                    let at = |north: f32, east: f32, z: f32| climate.map(|climate| climate.at(north, east, z));
                    let column = super::placer_view::world_spot(map, &at, mineralogy, north, east, here);
                    tops[k] = column.water.unwrap_or(0);
                    let colour = if crate::world::under_sea(here) || column.water.is_some() {
                        palette.water.map(|c| c * shade.top)
                    } else {
                        let top = mineralogy.layers(map, column).top;
                        let mut light = shade.top;
                        if lit {
                            // The slope, blocks up per block across, as the distant terrain measures it
                            // (per finest cell), and the height tint.
                            let or_here = |v: f32| if v.is_nan() { here } else { v };
                            let east_rise = (or_here(z(col + 1, row)) - or_here(z(col.saturating_sub(1), row))) / (2.0 * step);
                            let north_rise = (or_here(z(col, row.saturating_sub(1))) - or_here(z(col, row + 1))) / (2.0 * step);
                            let tint = 1.0 + shade.height_tint * (here - sea) / (crate::world::HIGHEST - sea);
                            light *= shade.hill(north_rise * look.finest_cell, east_rise * look.finest_cell) * tint;
                        }
                        palette.top(top).map(|c| c * light)
                    };
                    *pixel = Color32::from_rgb(to_srgb(colour[0]), to_srgb(colour[1]), to_srgb(colour[2]));
                    // A river's water by its water across (a lake's and the sea's left as they are).
                    if sizes
                        && column.water.is_some()
                        && let Some((distance, width, across)) = map.rivers.water_near(north, east)
                        && distance <= width / 2.0 + 1.0
                    {
                        let tint = water_tint(across);
                        let mix = |a: u8, b: u8| ((a as f32) * 0.3 + (b as f32) * 0.7) as u8;
                        *pixel = Color32::from_rgb(mix(pixel.r(), tint[0]), mix(pixel.g(), tint[1]), mix(pixel.b(), tint[2]));
                    }
                }
            });
        }
    });
    if shores {
        // A dry sample beside water: green flush (its top level with the water), amber a 1-block
        // ledge, red a taller bank, magenta below the water (water standing against air).
        for row in 0..rows {
            for col in 0..cols {
                let here = z(col, row);
                if here.is_nan() || tops[row * cols + col] != 0 || crate::world::under_sea(here) {
                    continue;
                }
                let beside = [(col.wrapping_sub(1), row), (col + 1, row), (col, row.wrapping_sub(1)), (col, row + 1)]
                    .into_iter()
                    .filter(|&(c, r)| c < cols && r < rows)
                    .map(|(c, r)| tops[r * cols + c])
                    .max()
                    .unwrap_or(0);
                if beside == 0 {
                    continue;
                }
                let tint = match here as i32 + 1 - beside {
                    ..=-1 => [235, 40, 235],
                    0 => [70, 200, 110],
                    1 => [245, 175, 40],
                    _ => [230, 70, 60],
                };
                let pixel = &mut pixels[row * cols + col];
                let mix = |a: u8, b: u8| ((a as f32) * 0.3 + (b as f32) * 0.7) as u8;
                *pixel = Color32::from_rgb(mix(pixel.r(), tint[0]), mix(pixel.g(), tint[1]), mix(pixel.b(), tint[2]));
            }
        }
    }
    pixels
}

fn canvas(ui: &mut egui::Ui, compiler: &mut Compiler, look: &Look) {
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
            "No bake yet. Press EZ-Bake, or bake a world in the Tectonic Sim."
        };
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, message, egui::FontId::proportional(14.0), PINK);
        return;
    };
    let width = result.made.width;
    let compiled_map = result.made.map.clone();
    // Shift + drag draws a cross-section instead of moving the view.
    let shift = ui.input(|input| input.modifiers.shift);
    compiler.camera.hold_pan = shift;
    // Ctrl + the wheel steps the field view's level (the wheel alone zooms, as ever).
    compiler.fields.wheel(ui, response.hovered());
    compiler.hover = compiler.camera.update(ui, rect, &response, (width, width));
    // The world's edge (the view itself is the close-up, below).
    let field = compiler.camera.field_rect(rect, (width, width));
    painter.circle_stroke(field.center(), field.width() / 2.0, egui::Stroke::new(1.0, MAGENTA));

    // Blocks (north, east) and the screen, both ways.
    let (centre, zoom, blocks, half) = (compiler.camera.centre, compiler.camera.zoom, bake.info.blocks_per_pixel, width as f32 / 2.0);
    let to_blocks = |pos: egui::Pos2| {
        let p = centre + (pos - rect.center()) / zoom;
        ((half - p.y) * blocks, (p.x - half) * blocks)
    };
    let to_screen = |(north, east): (f32, f32)| rect.center() + (Vec2::new(east / blocks + half, half - north / blocks) - centre) * zoom;

    // The view: the terrain lab's close-up, the ground as the Interpreter makes it, at any zoom.
    let map = compiler.lab.recipe(&compiled_map, &compiler.settings.interpreter);
    // The Climate stage: kept up to date with the recipe (it reads the ground without the rivers,
    // which are made from it), but not made again when only mineralogy changes (it comes after).
    let shape = compiler.lab.shape().cloned().unwrap_or_else(|| map.clone());
    compiler.climate.drive(&shape, &compiler.settings.climate);
    // The view: the same map with the rivers and lakes carved in, as the world will have them.
    let map = compiler.with_rivers(&map);
    let view = Some((to_blocks(rect.left_top()), to_blocks(rect.right_bottom()), blocks / zoom));
    compiler.lab.drive(&map, view);
    {
        if let Some(frame) = &compiler.lab.frame
            && !frame.heights.is_empty()
        {
            // The blocks' far colours, made again when the blocks change.
            let revision = look.registry.revision();
            if compiler.lab_palette.as_ref().is_none_or(|(made_for, _)| *made_for != revision) {
                let blocks = crate::world::TerrainBlocks::from_registry(look.registry);
                compiler.lab_palette = Some((revision, Arc::new(crate::render::distant::Palette::from_textures(look.registry, &blocks))));
            }
            let s = &map.settings;
            let wanted = PaintKey {
                frame: compiler.lab.version,
                mineralogy: s.mineralogy.clone(),
                climate: compiler.climate.version,
                lit: !compiler.lab_unlit,
                shores: compiler.lab_shores,
                sizes: compiler.lab_sizes,
                blocks: revision,
                shade: look.shade.clone(),
                field: (compiler.fields.shows != super::placer_view::Shows::World).then(|| (compiler.fields.shows, compiler.fields.look.clone())),
            };
            if compiler.lab_painted_for.as_ref() != Some(&wanted) || compiler.lab_texture.is_none() {
                let climate = LiveClimate::new(compiler, &map);
                let palette = compiler.lab_palette.as_ref().map(|(_, palette)| palette.clone()).unwrap();
                let blocks = crate::world::TerrainBlocks::from_registry(look.registry);
                let at = |north: f32, east: f32, z: f32| climate.as_ref().map(|climate| climate.at(north, east, z));
                let pixels = match compiler.fields.shown(&s.mineralogy) {
                    // The field view: the field being made (or a candidate's suitability), on black.
                    Some(k) => super::placer_view::field_picture(frame, &map, &at, &map.mineralogy(&blocks), k, &compiler.fields.look),
                    // The depth sweep: the blocks at the level, in their far colours, unlit.
                    None if compiler.fields.shows == super::placer_view::Shows::Blocks => {
                        let colour = |block: crate::world::BlockId| {
                            let [r, g, b] = palette.top(block).map(|c| to_srgb(c * look.shade.top));
                            Color32::from_rgb(r, g, b)
                        };
                        super::placer_view::sweep_picture(frame, &map, &at, &blocks, &colour, &compiler.fields.look)
                    }
                    None => block_picture(frame, &map, climate.as_ref(), look, &palette, &wanted),
                };
                let image = egui::ColorImage::new([frame.request.cols, frame.request.rows], pixels);
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
        // Under the mouse (shown in the right-hand panel): the field layer by layer (in the field
        // view), the column block by block (in the depth sweep), and why it's made of what it is.
        (compiler.fields.hover, compiler.fields.column, compiler.fields.why) = (None, None, Vec::new());
        if let Some((north, east, z)) = compiler.lab_hover {
            let climate = LiveClimate::new(compiler, &map);
            let at = |north: f32, east: f32, z: f32| climate.as_ref().map(|climate| climate.at(north, east, z));
            let blocks = crate::world::TerrainBlocks::from_registry(look.registry);
            let mineralogy = map.mineralogy(&blocks);
            if let Some(k) = compiler.fields.shown(&map.settings.mineralogy) {
                compiler.fields.hover = super::placer_view::hover(&map, &at, &mineralogy, k, &compiler.fields.look, (north, east, z));
            }
            let name = |block: crate::world::BlockId| look.registry.def(block).map_or_else(|| look.registry.name(block).to_string(), |def| def.display_name.clone());
            if compiler.fields.shows == super::placer_view::Shows::Blocks {
                compiler.fields.column = Some(super::placer_view::column_hover(&map, &at, &blocks, &name, &compiler.fields.look, (north, east, z)));
            }
            let named = |block: &str| look.registry.id(block).map_or_else(|| format!("{block} (not registered)"), name);
            compiler.fields.why = super::placer_view::why(&map, &at, &mineralogy, &named, (north, east, z));
        }
    }

    // What the Climate stage is showing, drawn over the view.
    let show = compiler.climate.show;
    if show != super::climate::Show::Nothing
        && let Some(ground) = compiler.climate.ground()
    {
        let view = compiler.climate.rain_view;
        let shown_from = compiler.settings.climate.rivers.shown_from;
        let wanted = compiler.climate.version ^ ((show as u64) << 48) ^ ((view as u64) << 40) ^ ((shown_from.to_bits() as u64) << 8);
        // The picture, how many pixels across, and how many map pixels each of them is.
        let picture = (compiler.climate_painted_for != Some(wanted) || compiler.climate_texture.is_none())
            .then(|| match show {
                super::climate::Show::Temperature => compiler.climate.temperature().filter(|t| t.width == ground.width).map(|t| (t.picture(ground), ground.width, ground.pixels)),
                super::climate::Show::Rain => compiler.climate.rain().filter(|r| r.width == ground.width).map(|r| (r.picture(ground, view), ground.width, ground.pixels)),
                super::climate::Show::Water => compiler.climate.flow().map(|f| (f.picture(shown_from), f.width, 1)),
                super::climate::Show::Rivers => compiler.climate.network().map(|n| (n.picture(compiler.settings.climate.network.fall_from), n.width, 1)),
                _ => Some((ground.picture(), ground.width, ground.pixels)),
            })
            .flatten();
        if let Some((pixels, across, scale)) = picture {
            let image = egui::ColorImage::new([across, across], pixels);
            let texture = compiler.climate_texture.get_or_insert_with(|| ui.ctx().load_texture("climate", egui::ColorImage::new([1, 1], vec![Color32::TRANSPARENT]), egui::TextureOptions::NEAREST));
            texture.set(image, egui::TextureOptions::NEAREST);
            compiler.climate_painted_for = Some(wanted);
            compiler.climate_painted_size = across * scale;
        }
        if let Some(texture) = &compiler.climate_texture {
            // A climate pixel is a whole number of map pixels; the last row and column may reach past the map.
            let field = compiler.camera.field_rect(rect, (width, width));
            let across = compiler.climate_painted_size as f32 / width as f32;
            painter.image(texture.id(), egui::Rect::from_min_size(field.min, field.size() * across), egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
        }
        // The rivers' curves at their real width (at least a line), over the network's grid path, so
        // the two can be compared; waterfalls' curtains marked white.
        if show == super::climate::Show::Rivers
            && let Some(carving) = compiler.climate.carving()
        {
            let visible = rect.expand(64.0);
            for reach in &carving.reaches {
                let shown: Vec<(egui::Pos2, f32, bool)> = reach.points.iter().map(|p| (to_screen((p.north, p.east)), p.width, p.curtain.is_finite())).collect();
                if !shown.iter().any(|(at, ..)| visible.contains(*at)) {
                    continue;
                }
                for pair in shown.windows(2) {
                    let width = (pair[0].1 * zoom / blocks).max(1.0);
                    let colour = if pair[0].2 { Color32::WHITE } else { Color32::from_rgba_unmultiplied(40, 120, 255, 200) };
                    painter.line_segment([pair[0].0, pair[1].0], egui::Stroke::new(width, colour));
                }
            }
        }
        if show == super::climate::Show::Wind
            && let Some(wind) = compiler.climate.wind()
            && wind.width == ground.width
        {
            wind_arrows(&painter, rect, &compiler.camera, wind, ground.pixels);
        }
    }

    // The overlays, over the map and the close-up alike.
    let a = &compiler.settings.additives;
    compiler.lab.overlays.drive(&map, &bake, (a.grain_reach, a.grain_from_squeeze));
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
        if overlays.shown.grain
            && let Some(grain) = &painted.grain
        {
            grain_strokes(&painter, rect, &compiler.camera, grain, &painted.share, width, tint);
        }
    }

    // The lift's points (the World Interpreter panel's), as cyan rings.
    if compiler.show_lift_points
        && let Some(result) = &compiler.result
    {
        let w = result.made.width;
        for &i in &result.made.lift_points {
            let at = compiler.camera.to_screen(rect, Vec2::new((i % w) as f32 + 0.5, (i / w) as f32 + 0.5));
            if rect.contains(at) {
                painter.circle_stroke(at, 4.0, egui::Stroke::new(2.0, Color32::from_rgb(0, 230, 255)));
            }
        }
    }

    // Where you were in the world when you pressed F2: a ring, and a line the way you faced.
    if let Some((north, east, facing)) = compiler.came_from {
        let (half, blocks) = (width as f32 / 2.0, bake.info.blocks_per_pixel);
        let at = compiler.camera.to_screen(rect, Vec2::new(east / blocks + half, half - north / blocks));
        if rect.expand(20.0).contains(at) {
            // North is up the map; the bearing turns clockwise from it.
            let (sin, cos) = facing.to_radians().sin_cos();
            let ahead = at + Vec2::new(sin, -cos) * 16.0;
            let stroke = egui::Stroke::new(2.0, Color32::from_rgb(255, 60, 220));
            painter.circle_stroke(at, 6.0, egui::Stroke::new(4.0, Color32::BLACK));
            painter.circle_stroke(at, 6.0, stroke);
            painter.line_segment([at + Vec2::new(sin, -cos) * 6.0, ahead], stroke);
            painter.text(at + Vec2::new(9.0, 9.0), egui::Align2::LEFT_TOP, "you were here", egui::FontId::proportional(11.0), Color32::from_rgb(255, 60, 220));
        }
    }

    // The Topography panel's dots, over the map and the close-up alike: bright at the spine, fading
    // along each stream.
    compiler.additives.drive(&map, &compiler.settings.additives, blocks);
    if compiler.additives.show {
        let size = (zoom * 0.75).clamp(1.5, 5.0);
        for dot in &compiler.additives.dots {
            let at = compiler.camera.to_screen(rect, Vec2::new(dot.at.0, dot.at.1));
            if rect.contains(at) {
                let colour = Color32::WHITE.gamma_multiply(1.0 - 0.7 * dot.along);
                painter.rect_filled(egui::Rect::from_center_size(at, Vec2::splat(size * compiler.additives.dot_size)), 0.0, colour);
            }
        }
        // Each range's middle (a ring) and tip (an arrow, as long as the range is lopsided, up to
        // half its size), for the bigger ranges.
        if compiler.settings.additives.drift_on
            && let Some((_, _, ranges)) = &compiler.additives.ranges
        {
            let stroke = egui::Stroke::new(2.0, Color32::WHITE);
            for range in ranges.list.iter().filter(|r| r.size >= 40) {
                let centre = compiler.camera.to_screen(rect, Vec2::new(range.centre.0, range.centre.1));
                if !rect.contains(centre) {
                    continue;
                }
                painter.circle_stroke(centre, 5.0, stroke);
                let reach = 0.5 * (range.size as f32).sqrt() * range.lopsided;
                let tip = compiler.camera.to_screen(rect, Vec2::new(range.centre.0 + range.tip.0 * reach, range.centre.1 + range.tip.1 * reach));
                if (tip - centre).length() > 8.0 {
                    painter.arrow(centre, tip - centre, stroke);
                }
            }
        }
        // The grain drift: yellow dots fading as they go, blue seeds on top.
        for dot in &compiler.additives.drift {
            let at = compiler.camera.to_screen(rect, Vec2::new(dot.at.0, dot.at.1));
            if rect.contains(at) {
                let colour = Color32::from_rgb(230, 220, 0).gamma_multiply(1.0 - 0.6 * dot.along);
                painter.circle_filled(at, size * 0.6 * compiler.additives.dot_size, colour);
            }
        }
        for &(x, y) in &compiler.additives.seeds {
            let at = compiler.camera.to_screen(rect, Vec2::new(x, y));
            if rect.contains(at) {
                painter.rect_filled(egui::Rect::from_center_size(at, Vec2::splat(size + 1.0)), 0.0, Color32::from_rgb(40, 80, 255));
            }
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

    // Picking where to start (after "Render World and Spawn"): a click on the map, or Space again
    // over the spot, starts the world there; Esc or right-click cancels.
    if compiler.picking {
        let half = width as f32 / 2.0;
        let blocks = bake.info.blocks_per_pixel;
        let spot = response.hover_pos().and_then(|mouse| {
            let pixel = compiler.camera.centre + (mouse - rect.center()) / compiler.camera.zoom;
            (pixel.x >= 0.0 && pixel.y >= 0.0 && pixel.x < width as f32 && pixel.y < width as f32)
                .then(|| (mouse, IVec2::new(((half - pixel.y) * blocks).round() as i32, ((pixel.x - half) * blocks).round() as i32)))
        });
        painter.text(
            rect.center_top() + Vec2::new(0.0, 12.0),
            egui::Align2::CENTER_TOP,
            "Click where you want to start (or press Space over the spot).   Esc or right-click cancels.",
            egui::FontId::proportional(16.0),
            Color32::WHITE,
        );
        if let Some((mouse, _)) = spot {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            painter.circle_stroke(mouse, 8.0, egui::Stroke::new(4.0, Color32::BLACK));
            painter.circle_stroke(mouse, 8.0, egui::Stroke::new(2.0, Color32::WHITE));
        }
        let space = std::mem::take(&mut compiler.space_pick);
        if let Some((_, at)) = spot
            && (response.clicked() || space)
        {
            compiler.press_render(at);
        } else if response.secondary_clicked() || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            compiler.picking = false;
        }
    }

    // The level bar, for the field view and the depth sweep (`placer_view.rs`).
    super::placer_view::level_bar(ui, rect, &mut compiler.fields);

    let help = "every column as the Interpreter makes it   drag = move   scroll = zoom   Shift + drag = cross-section   double-click = fit   north is up";
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

/// The nearest of the 8 compass points to a direction (east, south).
fn compass_point(east: f32, south: f32) -> &'static str {
    const NAMES: [&str; 8] = ["east", "south-east", "south", "south-west", "west", "north-west", "north", "north-east"];
    let turn = south.atan2(east).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU * 8.0;
    NAMES[(turn.round() as usize) % 8]
}

/// The wind over the map: an arrow every so many screen points, the way the wind blows there,
/// longer and brighter where it's stronger. `pixels` = map pixels per climate pixel.
fn wind_arrows(painter: &egui::Painter, rect: egui::Rect, camera: &Camera, wind: &super::wind::Wind, pixels: usize) {
    const EVERY: f32 = 30.0;
    let w = wind.width;
    let (across, down) = ((rect.width() / EVERY) as usize + 1, (rect.height() / EVERY) as usize + 1);
    for row in 0..down {
        for col in 0..across {
            let at = rect.left_top() + Vec2::new((col as f32 + 0.5) * EVERY, (row as f32 + 0.5) * EVERY);
            // The map pixel under this spot, then its climate pixel.
            let pixel = (camera.centre + (at - rect.center()) / camera.zoom) / pixels as f32;
            if pixel.x < 0.0 || pixel.y < 0.0 || pixel.x >= w as f32 || pixel.y >= w as f32 {
                continue;
            }
            let i = pixel.y as usize * w + pixel.x as usize;
            let ([east, south], strength) = (wind.way[i], wind.strength[i]);
            if strength < 0.02 || (east == 0.0 && south == 0.0) {
                continue;
            }
            let colour = Color32::from_rgb(170, 230, 255).gamma_multiply(0.35 + 0.65 * strength);
            let stroke = egui::Stroke::new(1.5, colour);
            // On screen, east is right and south is down, as on the grid.
            let way = Vec2::new(east, south) * (5.0 + 9.0 * strength);
            let (tail, head) = (at - way, at + way);
            painter.line_segment([tail, head], stroke);
            let back = -way.normalized() * 5.0;
            let side = Vec2::new(-back.y, back.x) * 0.6;
            painter.line_segment([head, head + back + side], stroke);
            painter.line_segment([head, head + back - side], stroke);
        }
    }
}

/// The range grain overlay: a short stroke every so many screen points over the mountains, along
/// the way the range runs there, longer where it's more string-like.
fn grain_strokes(painter: &egui::Painter, rect: egui::Rect, camera: &Camera, grain: &super::additives::Grain, share: &[f32], width: usize, tint: Color32) {
    const EVERY: f32 = 24.0;
    let stroke = egui::Stroke::new(1.5, Color32::from_rgb(150, 255, 190).gamma_multiply(tint.a() as f32 / 255.0));
    let (across, down) = ((rect.width() / EVERY) as usize + 1, (rect.height() / EVERY) as usize + 1);
    for row in 0..down {
        for col in 0..across {
            let at = rect.left_top() + Vec2::new((col as f32 + 0.5) * EVERY, (row as f32 + 0.5) * EVERY);
            let pixel = camera.centre + (at - rect.center()) / camera.zoom;
            if pixel.x < 0.0 || pixel.y < 0.0 || pixel.x >= width as f32 || pixel.y >= width as f32 {
                continue;
            }
            let i = pixel.y as usize * width + pixel.x as usize;
            if share[i] < 0.05 {
                continue;
            }
            // East = 0, turning toward south: the same way round as the screen.
            let angle = grain.angle[i];
            let half = Vec2::new(angle.cos(), angle.sin()) * (3.0 + 8.0 * grain.string[i]);
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

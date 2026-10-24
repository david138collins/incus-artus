//! The World Compiler's screen: pick a bake, shape how its heights become blocks, and see the
//! result as shaded relief. (The work itself is in `compiler.rs`.)
//!
//! ```text
//! ┌──── top: which bake, land share ─────────────────────────────┐
//! │ sea level      │                          │ under the mouse │
//! │ mountains      │   the world, in relief   │                 │
//! │ rifts, hills   │                          │                 │
//! │ save, send     │                          │                 │
//! └────────────────┴──────────────────────────┴─────────────────┘
//! ```

use super::Suite;
use super::bake::{self, Baked};
use super::canvas::Camera;
use super::compiler::{self, Bake, CompileSettings, Compiled, HIGHEST, SEA_LEVEL, Spline, TOP_OF_THE_WORLD};
use super::image::{Picture, Placement};
use super::layers::LayerKind;
use super::runner::{Finished, Job, Poll};
use super::view::{MAGENTA, PINK, compass};
use super::world::defaults_row;
use crate::debug::preset_bar;
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
    pub render: Option<(String, u64, Arc<crate::world::CompiledMap>)>,
    /// The name to give a rendered world (blank = the bake's name).
    world_name: String,
    camera: Camera,
    texture: Option<egui::TextureHandle>,
    painted_for: Option<(u64, u32)>,
    hover: Option<(usize, usize)>,
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
            world_name: String::new(),
            camera: Camera::default(),
            texture: None,
            painted_for: None,
            hover: None,
        }
    }
}

impl Compiler {
    pub fn is_making(&self) -> bool {
        self.job.is_some() || self.ez.is_some()
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
    egui::Panel::left("world compiler settings").resizable(true).default_size(340.0).size_range(220.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| settings(ui, compiler));
    });
    egui::Panel::right("world compiler readouts").resizable(true).default_size(270.0).size_range(180.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| readouts(ui, compiler));
    });
    let send = compiler.send.take();
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, compiler));

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
    ui.heading("World Compiler");
    ui.small(
        "Turns a baked world into the ground's height in blocks. Sea level is Z 128; nothing goes past Z 333 (the \
         world's top is Z 399). Bake worlds in the Tectonic Sim, then pick one in the bar along the top.",
    );
    if !compiler.note.is_empty() {
        ui.small(&compiler.note);
    }
    ui.add_space(4.0);
    ui.strong("Presets");
    ui.small("Save every compiler setting (the splines too) as a preset, to try on any bake.");
    preset_bar(ui, "compiler", &mut compiler.settings);
    let Some(bake) = compiler.bake.clone() else {
        ui.colored_label(PINK, "No bake read yet.");
        return;
    };
    let s = &mut compiler.settings;

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Sea level and the curves");
    let (low, high) = (bake.info.height.black, bake.info.height.white);
    ui.add(crate::debug::Slider::new(&mut s.water, low..=high).text("bake height at sea level")).on_hover_text(
        "Which of the bake's heights becomes sea level (Z 128). Higher = more sea. Watch the land share in the bar along the top.",
    );
    ui.add(crate::debug::Slider::new(&mut s.deepest, 1.0..=127.0).text("deepest sea floor (Z)")).on_hover_text("The Z the deepest ocean reaches.");
    ui.add(crate::debug::Slider::new(&mut s.highlands, 129.0..=HIGHEST as f32).text("highlands (Z)"))
        .on_hover_text("The Z the highest land reaches before mountains are added (the top of the land spline).");
    ui.add_space(4.0);
    ui.label("Land: from the coast (left) to the bake's highest ground (right).");
    let defaults = CompileSettings::default();
    spline_box(ui, "land spline", &mut s.land, &defaults.land, "toward the highest ground", "up to the highlands");
    ui.label("Sea: from the coast (left) to the bake's deepest point (right).");
    spline_box(ui, "sea spline", &mut s.sea, &defaults.sea, "toward the deepest point", "down to the deepest floor");
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
    ui.add(crate::debug::Slider::new(&mut s.ridges, 0.0..=1.0).text("ridges")).on_hover_text(
        "How much the ridge pattern carves into the mountains. 0 = smooth domes and spines, 1 = all ridges and valleys.",
    );
    ui.add(crate::debug::Slider::new(&mut s.ridge_spacing, 400.0..=20_000.0).logarithmic(true).text("ridge spacing (blocks)"))
        .on_hover_text("How far apart the ridges are, across them.");
    ui.add(crate::debug::Slider::new(&mut s.ridge_length, 2_000.0..=100_000.0).logarithmic(true).text("ridge length (blocks)")).on_hover_text(
        "How far ridges run along themselves. Ridges are lined up across the squeeze direction;\nlonger = long parallel ranges, shorter = broken, lumpy ones.",
    );
    ui.add(crate::debug::Slider::new(&mut s.sharpness, 0.3..=4.0).text("ridge sharpness")).on_hover_text("Higher = narrower, sharper crests. Lower = broad, rounded ones.");
    ui.add(crate::debug::Slider::new(&mut s.passes, 0.0..=1.0).text("passes")).on_hover_text(
        "How deeply the crest height dips along each ridge, into passes between higher peaks. 0 = even crests.",
    );
    ui.add(crate::debug::Slider::new(&mut s.pass_spacing, 3_000.0..=60_000.0).logarithmic(true).text("pass spacing (blocks)"))
        .on_hover_text("How far apart the dips are along a ridge.");
    ui.add(crate::debug::Slider::new(&mut s.foothills, 0.0..=12_000.0).text("foothills (blocks)")).on_hover_text(
        "How far each range spreads a skirt of lower slopes around it, so it rises from the plains instead of\njumping up. Peaks aren't lowered. 0 = no foothills.",
    );

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Rifts and hills");
    ui.add(crate::debug::Slider::new(&mut s.rift_depth, 0.0..=80.0).text("rift depth (blocks)"))
        .on_hover_text("How far the most stretched rock in this bake sinks (less stretched rock, less).");
    ui.add(crate::debug::Slider::new(&mut s.hills, 0.0..=25.0).text("hills (blocks)")).on_hover_text("Gentle rolling bumps on land, this high either way.");
    ui.add(crate::debug::Slider::new(&mut s.hill_size, 300.0..=8_000.0).logarithmic(true).text("hill size (blocks)"));
    ui.add_space(4.0);
    defaults_row(ui, "Compiler settings", &mut compiler.settings, &mut compiler.ab);

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
        .on_hover_text("Writes elevation.png (16-bit: the grey is the ground's Z) and compile.toml (these settings) into the bake's folder.")
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
        "Makes a real world from this compiled map and puts you in its middle: the ground follows the map smoothly, \
         with grass on land, stone on mountainsides, gabbro on the peaks, and water filling the sea to Z 128. The \
         world keeps its own copy of the map, so changing things here afterwards never changes it.",
    );
    let i = &mut compiler.settings.interpreter;
    ui.add(crate::debug::Slider::new(&mut i.detail, 0.0..=12.0).text("ground detail (blocks)"))
        .on_hover_text("Small bumps on the ground, this high either way, so it isn't glassy smooth between the map's points.");
    ui.add(crate::debug::Slider::new(&mut i.detail_size, 16.0..=512.0).logarithmic(true).text("detail size (blocks)"));
    ui.add(crate::debug::Slider::new(&mut i.stone_from, SEA_LEVEL as i32..=HIGHEST as i32).text("bare stone from Z"))
        .on_hover_text("The surface is stone from this height up (mountainsides).");
    ui.add(crate::debug::Slider::new(&mut i.gabbro_from, SEA_LEVEL as i32..=HIGHEST as i32).text("gabbro from Z"))
        .on_hover_text("The surface is gabbro from this height up, so peaks stand out from a distance.");
    ui.horizontal_wrapped(|ui| {
        ui.label("World name:");
        ui.add(egui::TextEdit::singleline(&mut compiler.world_name).desired_width(140.0).hint_text(bake.name.clone()));
    });
    if ui
        .button("Render World and Spawn")
        .on_hover_text("Starts a new world from this map (the world you're in now is replaced; save it first if you want it).")
        .clicked()
    {
        compiler.press_render();
    }
}

impl Compiler {
    /// "Render World and Spawn": the compiled map, with the interpreter settings, handed over to
    /// start a new world.
    pub fn press_render(&mut self) {
        let (Some(bake), Some(result)) = (&self.bake, &self.result) else { return };
        let name = if self.world_name.trim().is_empty() { bake.name.clone() } else { self.world_name.clone() };
        let seed = u64::from_str_radix(&bake.info.seed, 16).unwrap_or(0);
        let map = crate::world::CompiledMap {
            width: result.made.width,
            blocks_per_pixel: bake.info.blocks_per_pixel,
            z: result.made.z.clone(),
            seed,
            settings: self.settings.interpreter.clone(),
        };
        self.render = Some((name, seed, Arc::new(map)));
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

fn readouts(ui: &mut egui::Ui, compiler: &Compiler) {
    ui.heading("Under the mouse");
    let (Some(bake), Some(result)) = (&compiler.bake, &compiler.result) else {
        ui.small("Nothing compiled yet.");
        return;
    };
    let width = result.made.width;
    match compiler.hover {
        Some((col, row)) if col < width && row < width => {
            let i = row * width + col;
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
            ui.separator();
            ui.small("From the bake:");
            ui.small(format!("height {:.3}", bake.height[i]));
            ui.small(format!("squeeze {:+.2}", bake.squeeze[i]));
            if bake.push_strength[i] > 0.02 {
                ui.small(format!("squeezed at {:.0}°, strength {:.2}", bake.push_angle[i], bake.push_strength[i]));
            }
            ui.small(if bake.plate[i] == 0 { "no plate".to_string() } else { format!("plate grey {}", bake.plate[i]) });
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

fn canvas(ui: &mut egui::Ui, compiler: &mut Compiler) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
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
    let wanted = (compiler.version, compiler.relief.to_bits());
    if compiler.painted_for != Some(wanted) || compiler.texture.is_none() {
        let pixels: Vec<Color32> = relief_picture(&result.made, bake.info.blocks_per_pixel, compiler.relief).into_iter().map(|[r, g, b]| Color32::from_rgb(r, g, b)).collect();
        let image = egui::ColorImage::new([width, width], pixels);
        // Made tiny first, then given the picture (egui's first-make size check is too cautious).
        let texture = compiler.texture.get_or_insert_with(|| ui.ctx().load_texture("world compiler", egui::ColorImage::new([1, 1], vec![Color32::BLACK]), egui::TextureOptions::NEAREST));
        texture.set(image, egui::TextureOptions::NEAREST);
        compiler.painted_for = Some(wanted);
    }
    compiler.hover = compiler.camera.update(ui, rect, &response, (width, width));
    if let Some(texture) = &compiler.texture {
        let field = compiler.camera.field_rect(rect, (width, width));
        painter.image(texture.id(), field, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
        painter.circle_stroke(field.center(), field.width() / 2.0, egui::Stroke::new(1.0, MAGENTA));
    }
    compiler.camera.outline(&painter, rect, compiler.hover, MAGENTA);
    painter.text(
        rect.left_bottom() + Vec2::new(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "drag = move   scroll = zoom   double-click = fit   north is up",
        egui::FontId::proportional(12.0),
        PINK,
    );
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

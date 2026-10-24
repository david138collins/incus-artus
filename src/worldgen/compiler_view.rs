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
use super::bake;
use super::canvas::Camera;
use super::compiler::{self, Bake, CompileSettings, Compiled, HIGHEST, SEA_LEVEL, TOP_OF_THE_WORLD};
use super::image::{Picture, Placement};
use super::layers::LayerKind;
use super::runner::{Finished, Job, Poll};
use super::view::{MAGENTA, PINK, compass};
use super::world::defaults_row;
use bevy_egui::egui::{self, Color32, Vec2};
use std::sync::Arc;

/// Past the edge of the world.
const BEYOND: [u8; 3] = [10, 10, 14];

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
            camera: Camera::default(),
            texture: None,
            painted_for: None,
            hover: None,
        }
    }
}

impl Compiler {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    /// Reads the chosen bake if it isn't read yet (the newest bake, if none is chosen), compiles
    /// again when the bake or the settings change, and collects finished work.
    pub fn drive(&mut self) {
        if self.chosen.is_none() {
            self.chosen = newest_bake();
        }
        let Some(chosen) = self.chosen.clone() else { return };
        if self.bake.as_ref().is_none_or(|bake| bake.name != chosen) {
            match compiler::load(&bake::bakes_folder().join(&chosen)) {
                Ok(bake) => {
                    // Start from the water level it was baked with.
                    self.settings.water = bake.info.water_level;
                    self.note = format!("Read \"{chosen}\": {} × {} pixels, {} blocks each.", bake.width, bake.width, bake.info.blocks_per_pixel);
                    self.bake = Some(Arc::new(bake));
                    self.result = None;
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
    let compiler = &mut suite.compiler;
    egui::Panel::top("world compiler bar").show(root, |ui| {
        ui.horizontal_wrapped(|ui| {
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
    let mut send = None;
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, compiler, &mut send));

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
    let Some(bake) = compiler.bake.clone() else {
        ui.colored_label(PINK, "No bake read yet.");
        return;
    };
    let s = &mut compiler.settings;

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Sea level and the curves");
    let (low, high) = (bake.info.height.black, bake.info.height.white);
    ui.add(egui::Slider::new(&mut s.water, low..=high).text("bake height at sea level")).on_hover_text(
        "Which of the bake's heights becomes sea level (Z 128). Higher = more sea. Watch the land share in the bar along the top.",
    );
    ui.add(egui::Slider::new(&mut s.deepest, 1.0..=127.0).text("deepest sea floor (Z)")).on_hover_text("The Z the deepest ocean reaches.");
    ui.add(egui::Slider::new(&mut s.highlands, 129.0..=HIGHEST as f32).text("highlands (Z)"))
        .on_hover_text("The Z the highest land reaches before mountains are added.");
    ui.add(egui::Slider::new(&mut s.land_curve, 0.3..=4.0).logarithmic(true).text("land curve")).on_hover_text(
        "How the land climbs from the coast to the highlands. 1 = evenly. Above 1 = mostly lowland,\nonly the highest parts climbing. Below 1 = most land high, falling steeply to the coast.",
    );
    ui.add(egui::Slider::new(&mut s.sea_curve, 0.3..=4.0).logarithmic(true).text("sea curve")).on_hover_text(
        "How the sea floor falls from the coast to the deepest. Above 1 = shallow shelves near the coast,\nthen a deep drop. Below 1 = deep water right off the coast.",
    );

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Mountains (from the squeeze map)");
    ui.add(egui::Slider::new(&mut s.mountains, 0.0..=1.0).text("how tall")).on_hover_text(
        "How much of the room up to Z 333 the most crushed rock fills. 1 = right up to it.",
    );
    ui.add(egui::Slider::new(&mut s.mountains_start, 0.0..=1.0).text("start at squeeze"))
        .on_hover_text("How crushed the rock must be (thickness gained) before mountains begin.");
    ui.add(egui::Slider::new(&mut s.mountains_full, 0.05..=2.0).text("fullest at squeeze"))
        .on_hover_text("How crushed the rock must be for the mountains to be at their tallest.");
    ui.add(egui::Slider::new(&mut s.ridge_spacing, 200.0..=10_000.0).logarithmic(true).text("ridge spacing (blocks)"))
        .on_hover_text("How far apart the ridges are, across them.");
    ui.add(egui::Slider::new(&mut s.ridge_length, 1_000.0..=50_000.0).logarithmic(true).text("ridge length (blocks)")).on_hover_text(
        "How far ridges run along themselves. Ridges are lined up across the squeeze direction;\nlonger = long parallel ranges, shorter = broken, lumpy ones.",
    );
    ui.add(egui::Slider::new(&mut s.sharpness, 0.5..=6.0).text("ridge sharpness")).on_hover_text("Higher = narrower, sharper crests.");

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Rifts and hills");
    ui.add(egui::Slider::new(&mut s.rift_depth, 0.0..=100.0).text("rift depth (blocks)")).on_hover_text("How far the most stretched rock sinks.");
    ui.add(egui::Slider::new(&mut s.hills, 0.0..=40.0).text("hills (blocks)")).on_hover_text("Gentle rolling bumps on land, this high either way.");
    ui.add(egui::Slider::new(&mut s.hill_size, 200.0..=10_000.0).logarithmic(true).text("hill size (blocks)"));
    ui.add_space(4.0);
    defaults_row(ui, "Compiler settings", &mut compiler.settings, &mut compiler.ab);

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Look");
    ui.add(egui::Slider::new(&mut compiler.relief, 0.0..=10.0).text("relief shading")).on_hover_text(
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

fn canvas(ui: &mut egui::Ui, compiler: &mut Compiler, send: &mut Option<(String, Picture, f32)>) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let (Some(bake), Some(result)) = (compiler.bake.clone(), &compiler.result) else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "No bake compiled yet. Bake a world in the Tectonic Sim.", egui::FontId::proportional(14.0), PINK);
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

    // Sending to the Graphics Kit, from the bottom right.
    let corner = rect.right_bottom() + Vec2::new(-8.0, -8.0);
    let area = egui::Area::new(egui::Id::new("world compiler send")).fixed_pos(corner).pivot(egui::Align2::RIGHT_BOTTOM);
    area.show(ui.ctx(), |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.small("to the Graphics Kit:");
                if ui.small_button("elevation").on_hover_text("Grey: black = Z 0, white = Z 399.").clicked() {
                    let pixels = result.made.z.iter().map(|&z| {
                        let v = (z as f32 / TOP_OF_THE_WORLD as f32 * 255.0).round() as u8;
                        [v, v, v, 255]
                    });
                    *send = Some(("elevation".to_string(), Picture { width, height: width, pixels: pixels.collect() }, bake.info.blocks_per_pixel));
                }
                if ui.small_button("relief").on_hover_text("The coloured, shaded map as you see it.").clicked() {
                    let pixels = relief_picture(&result.made, bake.info.blocks_per_pixel, compiler.relief).into_iter().map(|[r, g, b]| [r, g, b, 255]);
                    *send = Some(("relief".to_string(), Picture { width, height: width, pixels: pixels.collect() }, bake.info.blocks_per_pixel));
                }
            });
        });
    });
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

//! The Editor Suite's screen. Placeholder looks: plain panels and the field.
//!
//! The bar along the top is shared: the switch between the **Graphics Kit** and **World Gen**,
//! and the field and pixel size both use. Below it, the Graphics Kit (this file) looks like this;
//! World Gen draws its own (`world.rs`).
//!
//! ```text
//! ┌──────────── top: export ─────────────────────────────────────┐
//! │ + Add layer    │                          │ Layers         │
//! │                │                          │  (top first)   │
//! │ the open       │        the field         │                │
//! │ layer's        │  drag = pan, scroll =    │ under the      │
//! │ settings       │  zoom                    │ mouse          │
//! │                │                          │                │
//! │ its colour     │                          │ histogram      │
//! └────────────────┴──────────────────────────┴────────────────┘
//! ```

use super::blend::Mix;
use super::canvas::Camera;
use super::export;
use super::image::{self, Placement};
use super::layers::{self, LayerKind};
use super::made::{Made, Shaping};
use super::{Kit, RESULT, Suite, Watch};
use super::noise::{Fade, PerlinSettings};
use super::warp::{WarpMode, WarpSettings};
use super::worley::{Distance, Feature, WorleySettings};
use super::palette::{self, Channel, Colouring, Spread};
use crate::debug::preset_bar;
use bevy_egui::egui::{self, Color32, Pos2, Rect, Vec2};

pub const PINK: Color32 = Color32::from_rgb(255, 140, 220);
pub const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);
/// The checkerboard where no layer covers the field (as in image editors, but dark).
const EMPTY: [[u8; 3]; 2] = [[24, 24, 28], [40, 40, 46]];

pub struct ViewState {
    camera: Camera,
    texture: Option<egui::TextureHandle>,
    /// The result: every layer mixed (its values and numbers).
    stacked: Option<Made>,
    /// The picture on screen (one colour per field pixel, `None` = no layer), and what it was
    /// made from: exports are cut from it.
    composite: Vec<Option<[u8; 3]>>,
    composite_for: Option<PictureKey>,
    size: (usize, usize),
    canvas: Rect,
    hover: Option<(usize, usize)>,
    seed_text: String,
    /// The last message (exports, loading pictures), shown in the top bar.
    status: Option<String>,
    palette_name: String,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            texture: None,
            stacked: None,
            composite: Vec::new(),
            composite_for: None,
            size: (0, 0),
            canvas: Rect::NOTHING,
            hover: None,
            seed_text: String::new(),
            status: None,
            palette_name: String::new(),
        }
    }
}

impl ViewState {
    pub fn say(&mut self, message: String) {
        self.status = Some(message);
    }
}

/// Everything the picture on screen is made from, to notice when it changes: each layer (id,
/// shown, version, mix, opacity), the result's colouring, what's watched, and the colouring of
/// a layer watched alone.
type PictureKey = (Vec<(u32, bool, u64, Mix, f32)>, Colouring, Watch, Option<Colouring>);

/// The layer watched on its own, if any.
fn alone(suite: &Suite) -> Option<usize> {
    let Watch::Alone(id) = suite.watch else { return None };
    suite.layers.iter().position(|layer| layer.id == id)
}

fn picture_key(suite: &Suite) -> PictureKey {
    let stack = suite.layers.iter().map(|l| (l.id, l.visible, l.version, l.mix, l.opacity)).collect();
    let alone = alone(suite).map(|i| suite.layers[i].colouring.clone());
    (stack, suite.colouring.clone(), suite.watch, alone)
}

/// "the result", "Perlin noise field 1 alone", "everything up to Perlin noise field 1".
fn watch_name(suite: &Suite, watch: Watch) -> String {
    let name = |id: u32| suite.layers.iter().find(|layer| layer.id == id).map_or("?".into(), |layer| short(&layer.name, 24));
    match watch {
        Watch::Result => "the result".into(),
        Watch::Alone(id) => format!("{} alone", name(id)),
        Watch::UpTo(id) => format!("everything up to {}", name(id)),
    }
}

/// "a-very-long-file-name" → "a-very-long-fi…" (for tight spots; hover shows the whole name).
/// A left-hand side panel that can be folded away to a thin strip, for more room for the map. Open,
/// it has a small `<` in its top right corner; folded, the strip shows a `>` and the panel's name
/// down its length. `id` names the panel (it remembers its width), `title` is shown on the strip.
pub fn foldable_left(root: &mut egui::Ui, id: &str, title: &str, folded: &mut bool, default_size: f32, add_contents: impl FnOnce(&mut egui::Ui)) {
    if *folded {
        egui::Panel::left(format!("{id} folded")).resizable(false).exact_size(26.0).show(root, |ui| {
            ui.vertical_centered(|ui| {
                if ui.small_button(">").on_hover_text(format!("Open the {title} panel")).clicked() {
                    *folded = false;
                }
                ui.add_space(6.0);
                for letter in title.chars() {
                    if letter == ' ' {
                        ui.add_space(6.0);
                    } else {
                        ui.label(egui::RichText::new(letter.to_string()).small().color(PINK));
                    }
                }
            });
            // Anywhere on the strip opens it too.
            if ui.interact(ui.max_rect(), egui::Id::new((id, "strip")), egui::Sense::click()).clicked() {
                *folded = false;
            }
        });
        return;
    }
    egui::Panel::left(id.to_string()).resizable(true).default_size(default_size).size_range(220.0..=900.0).show(root, |ui| {
        // The fold button, drawn over the top right corner afterwards (so it takes no room from the
        // panel, and sits on top of it).
        let corner = Rect::from_min_size(Pos2::new(ui.max_rect().right() - 20.0, ui.max_rect().top()), Vec2::splat(20.0));
        add_contents(ui);
        let button = ui.interact(corner, egui::Id::new((id, "fold")), egui::Sense::click()).on_hover_text(format!("Fold the {title} panel away (more room for the map)"));
        let painter = ui.painter();
        if button.hovered() {
            painter.rect_filled(corner, 3.0, ui.visuals().widgets.hovered.bg_fill);
        }
        painter.text(corner.center(), egui::Align2::CENTER_CENTER, "<", egui::FontId::proportional(14.0), PINK);
        if button.clicked() {
            *folded = true;
        }
    });
}

pub fn short(name: &str, most: usize) -> String {
    if name.chars().count() <= most { name.to_string() } else { name.chars().take(most - 1).chain(['…']).collect() }
}

/// Draws the whole screen. Returns true when "Back to the world" was pressed.
pub fn draw(ctx: &egui::Context, suite: &mut Suite) -> bool {
    let mut root = egui::Ui::new(
        ctx.clone(),
        "editor suite".into(),
        egui::UiBuilder::new().layer_id(egui::LayerId::background()).max_rect(ctx.viewport_rect()),
    );
    let mut close = false;
    egui::Panel::top("editor suite bar").show(&mut root, |ui| close = suite_bar(ui, suite));
    match suite.kit {
        Kit::Graphics => graphics_kit(&mut root, suite),
        Kit::WorldGen => super::world::draw(&mut root, suite),
        Kit::Compiler => super::compiler_view::draw(&mut root, suite),
    }
    close
}

fn graphics_kit(root: &mut egui::Ui, suite: &mut Suite) {
    egui::Panel::top("graphics kit bar").show(root, |ui| top_bar(ui, suite));
    // Both side bars can be dragged wider or narrower by their inner edge. Layer names (a picture's
    // long file name, say) are cut short with "…" instead of pushing the bar wider; hover shows
    // them in full.
    egui::Panel::left("worldgen settings").resizable(true).default_size(360.0).size_range(220.0..=900.0).show(
        root,
        |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| settings_panel(ui, suite));
        },
    );
    egui::Panel::right("worldgen layers").resizable(true).default_size(270.0).size_range(180.0..=900.0).show(
        root,
        |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| layers_panel(ui, suite));
        },
    );
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, suite));
}

// ── The suite's bar: which kit, the field and pixel size ──────────────────

/// The processing meter: whenever the suite is working something out (compiling, baking, drawing
/// the close-up, making layers…), a spinner, what it's doing and for how long; otherwise "ready".
/// It has a slot of its own that's always there, so it never pushes the bar around.
fn processing_meter(ui: &mut egui::Ui, suite: &mut Suite) {
    let working = suite.compiler.working().or(suite.world.working()).or(suite.is_making().then_some("making layers"));
    suite.busy = match (working, suite.busy) {
        // Still busy (perhaps with the next step): the clock keeps running from the start.
        (Some(what), Some((_, since))) => Some((what, since)),
        (Some(what), None) => Some((what, std::time::Instant::now())),
        (None, _) => None,
    };
    let height = ui.spacing().interact_size.y;
    let (slot, response) = ui.allocate_exact_size(egui::vec2(230.0, height), egui::Sense::hover());
    let font = egui::FontId::proportional(12.0);
    match suite.busy {
        Some((what, since)) => {
            let spinner = egui::Rect::from_center_size(slot.left_center() + egui::vec2(8.0, 0.0), egui::Vec2::splat(14.0));
            egui::Spinner::new().size(14.0).color(PINK).paint_at(ui, spinner);
            let seconds = since.elapsed().as_secs_f32();
            let text = if seconds >= 0.5 { format!("{what}  {seconds:.1} s") } else { what.to_string() };
            ui.painter().text(slot.left_center() + egui::vec2(22.0, 0.0), egui::Align2::LEFT_CENTER, text, font, PINK);
            ui.ctx().request_repaint();
        }
        None => {
            ui.painter().circle_filled(slot.left_center() + egui::vec2(8.0, 0.0), 4.0, Color32::from_gray(90));
            ui.painter().text(slot.left_center() + egui::vec2(22.0, 0.0), egui::Align2::LEFT_CENTER, "ready", font, Color32::from_gray(120));
        }
    }
    response.on_hover_text(
        "The processing meter. While the suite is working something out (compiling, baking, simulating, drawing\n\
         the close-up, making layers), it spins and says what, and for how long. Settings that ask a lot can\n\
         take a moment: everything keeps working meanwhile, and the result appears when it's ready.",
    );
}

fn suite_bar(ui: &mut egui::Ui, suite: &mut Suite) -> bool {
    let mut close = false;
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(PINK, "EDITOR SUITE");
        processing_meter(ui, suite);
        if ui.button("Back to the world  [F2]").clicked() {
            close = true;
        }
        ui.separator();
        ui.selectable_value(&mut suite.kit, Kit::Graphics, "Graphics Kit")
            .on_hover_text("Noise, pictures, mixing and palettes: numbers into colours and colours into numbers.\nThe art pipeline.");
        ui.selectable_value(&mut suite.kit, Kit::WorldGen, "Tectonic Sim").on_hover_text(
            "The tectonic simulation: plates, their motion and collisions, heights and the water level.\nIts product, a bake, feeds the Terrain Compiler.",
        );
        ui.selectable_value(&mut suite.kit, Kit::Compiler, "Terrain Compiler")
            .on_hover_text("Turns a baked world into the ground's height in blocks: sea, lowland, mountains.");
        ui.separator();

        ui.label("Field:");
        ui.add(
            egui::DragValue::new(&mut suite.size_blocks)
                .range(1_000.0..=200_000.0)
                .speed(500.0)
                .suffix(" blocks across"),
        );
        ui.separator();
        ui.label("1 pixel =");
        ui.add(crate::debug::Slider::new(&mut suite.chunks_per_pixel, 1..=45).suffix(" chunks"))
            .on_hover_text("A chunk is 32 × 32 blocks. Bigger = chunkier, fewer pixels.");
        let blocks = suite.blocks_per_pixel();
        let across = (suite.size_blocks / blocks).ceil();
        let even = (suite.size_blocks / blocks).fract() == 0.0;
        ui.label(format!("= {blocks} blocks, {across} × {across} pixels"));
        if even {
            ui.small("(fits the field exactly)");
        } else {
            ui.small("(the last pixel runs past the edge)").on_hover_text(
                "The field isn't a whole number of pixels this size.\nFor 72,000 blocks, these fit exactly: 1, 2, 3, 5, 6, 9, 10, 15, 18, 25, 30 and 45 chunks.",
            );
        }
    });
    close
}

// ── The Graphics Kit's bar: export ─────────────────────────────────────────

fn top_bar(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.horizontal_wrapped(|ui| {
        let ready = !suite.view.composite.is_empty();
        let view = &mut suite.view;
        if ui
            .add_enabled(ready, egui::Button::new("Export what's in view"))
            .on_hover_text("Saves the pixels on screen as a PNG, one field pixel per picture pixel,\ncoloured as you see them. Where no layer covers, it's see-through.")
            .clicked()
        {
            view.status = Some(export_picture(view, false));
        }
        if ui.add_enabled(ready, egui::Button::new("Export the whole field")).clicked() {
            view.status = Some(export_picture(view, true));
        }
        if ui.small_button("Exports folder").clicked()
            && let Err(error) = export::open_folder(&export::export_folder())
        {
            view.status = Some(format!("Couldn't open the folder: {error}"));
        }
        if let Some(status) = &view.status {
            ui.small(status);
        }
    });
}

fn export_picture(view: &ViewState, whole: bool) -> String {
    let (width, height) = view.size;
    let (left, top, w, h) = if whole {
        (0, 0, width, height)
    } else {
        match view.camera.visible_pixels(view.canvas, view.size) {
            Some(area) => area,
            None => return "Nothing in view to export.".into(),
        }
    };
    let mut colours = Vec::with_capacity(w * h);
    for row in top..top + h {
        colours.extend_from_slice(&view.composite[row * width + left..row * width + left + w]);
    }
    match export::save_png(&export::fresh_name("field"), w, h, &colours) {
        Ok(path) => format!("Saved {w} × {h}: {}", path.display()),
        Err(error) => format!("Couldn't export: {error}"),
    }
}

// ── Upper left: add a layer, and the open layer's settings ─────────────────

fn settings_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.horizontal(|ui| {
        ui.menu_button("+ Add layer", |ui| {
            if ui.button("Perlin noise field").on_hover_text("Soft hills and hollows.").clicked() {
                suite.add_perlin();
                ui.close();
            }
            if ui.button("Worley noise field").on_hover_text("Cells: bubbles, cracks, patches (Voronoi).").clicked() {
                suite.add_worley();
                ui.close();
            }
            if ui.button("Warp").on_hover_text("A filter: bends the layer below it (domain warping).\nStraight things meander, round things swirl.").clicked() {
                suite.add_warp();
                ui.close();
            }
            ui.menu_button("Image from a file", |ui| {
                let names = image::saved_images();
                if names.is_empty() {
                    ui.label("(no PNG files in the images folder yet)");
                }
                for name in names {
                    if ui.button(&name).clicked() {
                        if let Err(error) = suite.add_image(&image::image_folder().join(&name)) {
                            suite.view.say(error);
                        }
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Open the images folder").clicked()
                    && let Err(error) = export::open_folder(&image::image_folder())
                {
                    suite.view.say(format!("Couldn't open the folder: {error}"));
                }
                ui.small("Or drag a PNG onto the window.");
            });
        });
    });
    ui.separator();

    let blocks_per_pixel = suite.blocks_per_pixel();
    let size_blocks = suite.size_blocks;
    let Suite { layers, selected, view, colouring, watch, .. } = suite;
    if *selected == Some(RESULT) {
        ui.heading("The result");
        ui.small(
            "All the visible layers mixed together, bottom first, each by its own mix and opacity. \
             Their numbers are mixed; the palette here colours what comes out.",
        );
        ui.add_space(10.0);
        ui.separator();
        ui.heading("Colour");
        colour_panel(ui, colouring, false, view);
        return;
    }
    let Some(layer) = selected.and_then(|id| layers.iter_mut().find(|layer| layer.id == id)) else {
        ui.add_space(20.0);
        if layers.is_empty() {
            ui.label("A blank page.");
            ui.small("Add a layer to start: a Perlin or Worley noise field, or an image (drag a PNG onto the window).");
        } else {
            ui.small("Pick a layer on the right to see its settings.");
        }
        return;
    };

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.add(egui::TextEdit::singleline(&mut layer.name).desired_width(f32::INFINITY));
    });
    ui.small(layer.kind.type_name());

    if let LayerKind::Warp { warp, .. } = &mut layer.kind {
        warp_settings(ui, warp, &mut layer.opacity);
    } else {
        mix_settings(ui, layer);
    }

    match &mut layer.kind {
        LayerKind::Perlin { seed, settings } => perlin_settings(ui, layer.id, seed, settings, view),
        LayerKind::Worley { seed, settings } => worley_settings(ui, layer.id, seed, settings, view),
        LayerKind::Image { file, picture, placement } => {
            image_settings(ui, file, (picture.width, picture.height), placement, blocks_per_pixel, size_blocks)
        }
        LayerKind::Warp { seed, settings, .. } => {
            ui.add_space(6.0);
            ui.heading("The warp's own noise");
            ui.small("Where the push is strong and which way it goes, read from this field. Big, soft noise (few octaves) bends smoothly; fine detail just jitters.");
            perlin_settings(ui, layer.id, seed, settings, view)
        }
    }

    ui.add_space(10.0);
    ui.separator();
    ui.heading("Colour when shown alone");
    ui.small(
        "How this layer looks on its own. In the result, the layers' numbers are mixed and the \
         result's palette colours them, so these colours only show when the layer is shown alone.",
    );
    let alone = Watch::Alone(layer.id);
    if *watch == alone {
        if ui.button("Back to watching the result").clicked() {
            *watch = Watch::Result;
        }
    } else if ui.button("Watch this layer alone").clicked() {
        *watch = alone;
    }
    let is_image = matches!(layer.kind, LayerKind::Image { .. });
    colour_panel(ui, &mut layer.colouring, is_image, view);
}

/// How a warp bends the layer below it (see `warp.rs`).
fn warp_settings(ui: &mut egui::Ui, warp: &mut WarpSettings, opacity: &mut f32) {
    section(
        ui,
        "Warp",
        "Bends the layer below: each pixel shows what that layer has a little way off, by an amount read from this layer's own noise. Nothing is added; the same shapes, bent.",
    );
    ui.horizontal(|ui| {
        ui.label("mode:");
        for mode in WarpMode::ALL {
            ui.selectable_value(&mut warp.mode, mode, mode.name()).on_hover_text(mode.about());
        }
    });
    ui.small(warp.mode.about());
    ui.add(crate::debug::Slider::new(&mut warp.amount, 50.0..=20_000.0).logarithmic(true).text("amount (blocks)"))
        .on_hover_text("How far the push reaches. About the size of the shapes below = strong bending; much more = smeared.");
    match warp.mode {
        WarpMode::Swirl => {
            ui.add(crate::debug::Slider::new(&mut warp.turns, 0.1..=4.0).text("turns")).on_hover_text(
                "How many times round the compass the push turns, from the warp's darkest value to its brightest.\nMore = tighter curls.",
            );
            ui.add(crate::debug::Slider::new(&mut warp.direction, 0.0..=360.0).suffix("°").text("start"))
                .on_hover_text("Which way the darkest value pushes (0 = east, 90 = north). Turns the whole swirl.");
        }
        WarpMode::Push => {
            ui.add(crate::debug::Slider::new(&mut warp.direction, 0.0..=360.0).suffix("°").text("direction"))
                .on_hover_text("Which way bright values push (0 = east, 90 = north); dark values push the opposite way.");
        }
        WarpMode::Flow => {}
    }
    let mut percent = *opacity * 100.0;
    if ui
        .add(crate::debug::Slider::new(&mut percent, 0.0..=100.0).suffix("%").text("opacity"))
        .on_hover_text("0% = not bent at all. 100% = fully bent.")
        .changed()
    {
        *opacity = percent / 100.0;
    }
    ui.checkbox(&mut warp.everything_below, "bend everything below").on_hover_text(
        "Off: bends only the layer just below, before it mixes in.\nOn: bends the whole stack beneath it, as it stands there.",
    );
}

/// How a layer mixes with the layers below it.
fn mix_settings(ui: &mut egui::Ui, layer: &mut layers::Layer) {
    section(ui, "Mix", "How this layer combines with the layers below it. Their numbers mix, not their colours.");
    ui.horizontal(|ui| {
        ui.label("mode:");
        egui::ComboBox::from_id_salt("worldgen mix").selected_text(layer.mix.name()).show_ui(ui, |ui| {
            for mix in Mix::ALL {
                ui.selectable_value(&mut layer.mix, mix, mix.name()).on_hover_text(mix.about());
            }
        });
    });
    ui.small(layer.mix.about());
    let mut percent = layer.opacity * 100.0;
    if ui
        .add(crate::debug::Slider::new(&mut percent, 0.0..=100.0).suffix("%").text("opacity"))
        .on_hover_text("0% = just the layers below. 100% = the full mix.")
        .changed()
    {
        layer.opacity = percent / 100.0;
    }
    ui.small("(The bottom layer has nothing below, so its own values are used as they are.)");
}

/// The seed, typed in hex or freshly rolled. `text` holds what's being typed.
pub fn seed_row(ui: &mut egui::Ui, id: u32, seed: &mut u64, text: &mut String) {
    ui.horizontal(|ui| {
        ui.label("Seed:");
        let seed_id = egui::Id::new(("worldgen seed", id));
        if ui.ctx().memory(|m| !m.has_focus(seed_id)) {
            *text = format!("{seed:016X}");
        }
        let edit = ui.add(egui::TextEdit::singleline(text).id(seed_id).desired_width(140.0));
        if edit.changed()
            && let Ok(typed) = u64::from_str_radix(text.trim().trim_start_matches("0x"), 16)
        {
            *seed = typed;
        }
        if ui.button("New seed").clicked() {
            *seed = crate::fate::derive_seed(super::fresh_number(), "noise layer");
        }
    });
}

fn worley_settings(ui: &mut egui::Ui, id: u32, seed: &mut u64, noise: &mut WorleySettings, view: &mut ViewState) {
    ui.small("Points scattered over a grid. Every spot belongs to its nearest point, which splits the world into cells (Voronoi).");
    seed_row(ui, id, seed, &mut view.seed_text);

    section(ui, "Cells", "The points, and what's measured from them.");
    ui.add(crate::debug::Slider::new(&mut noise.scale, 100.0..=72_000.0).logarithmic(true).text("cell size (blocks)"))
        .on_hover_text("About how far apart the points are: one grid square of the first octave.");
    ui.add(crate::debug::Slider::new(&mut noise.jitter, 0.0..=1.0).text("jitter"))
        .on_hover_text("How far each point may wander inside its grid square.\n0 = a neat grid of identical cells. 1 = fully random.");
    ui.horizontal_wrapped(|ui| {
        ui.label("shows:");
        for feature in Feature::ALL {
            ui.selectable_value(&mut noise.feature, feature, feature.name()).on_hover_text(feature.about());
        }
    });
    ui.small(noise.feature.about());
    ui.horizontal_wrapped(|ui| {
        ui.label("distance:");
        for distance in Distance::ALL {
            ui.selectable_value(&mut noise.distance, distance, distance.name());
        }
    })
    .response
    .on_hover_text("How distance to a point is measured, which sets the cells' shape.");

    section(ui, "Octaves", "Finer, fainter copies stacked on top. 1 = plain cells.");
    ui.add(crate::debug::Slider::new(&mut noise.octaves, 1..=8).text("octaves"));
    ui.add(crate::debug::Slider::new(&mut noise.persistence, 0.0..=1.0).text("persistence"))
        .on_hover_text("How strong each octave is compared with the one before.");
    ui.add(crate::debug::Slider::new(&mut noise.lacunarity, 1.1..=4.0).text("lacunarity"))
        .on_hover_text("How much finer each octave is than the one before.");

    placement_rows(
        ui,
        [&mut noise.offset_north, &mut noise.offset_east, &mut noise.stretch, &mut noise.stretch_angle, &mut noise.octave_turn],
    );

    output_settings(ui, &mut noise.output, Some("Stretch the lowest value to black and the highest to white.\nOff: the raw values (0 = black, 1 = white)."));

    ui.separator();
    if ui.button("All settings back to defaults").clicked() {
        *noise = WorleySettings::default();
    }
    ui.separator();
    ui.strong("Worley presets");
    ui.small("Save a useful kind of cells by name. The seed isn't saved with it.");
    preset_bar(ui, "worley", noise);
}

/// Slide, stretch, stretch direction, turn each octave (shared by the noise layers).
fn placement_rows(ui: &mut egui::Ui, [north, east, stretch, angle, turn]: [&mut f32; 5]) {
    section(ui, "Placement", "Where the pattern sits and which way it leans.");
    ui.horizontal(|ui| {
        ui.label("slide");
        ui.add(egui::DragValue::new(north).speed(50.0).suffix(" N"));
        ui.add(egui::DragValue::new(east).speed(50.0).suffix(" E"));
    })
    .response
    .on_hover_text("Moves the whole field, in blocks. Drag the numbers.");
    ui.add(crate::debug::Slider::new(stretch, 0.1..=10.0).logarithmic(true).text("stretch"))
        .on_hover_text("1 = round features. 3 = three times as long as they are wide.");
    ui.add(crate::debug::Slider::new(angle, 0.0..=180.0).suffix("°").text("stretch direction"))
        .on_hover_text("0° = stretched east–west, 90° = north–south.");
    ui.add(crate::debug::Slider::new(turn, 0.0..=90.0).suffix("°").text("turn each octave"))
        .on_hover_text("Turns each octave against the one before, so the grids don't line up\n(hides faint straight lines in the pattern).");
}

fn perlin_settings(ui: &mut egui::Ui, id: u32, seed: &mut u64, noise: &mut PerlinSettings, view: &mut ViewState) {
    ui.small("Soft hills and hollows from a grid of random arrows, stacked in finer and fainter layers (octaves).");
    seed_row(ui, id, seed, &mut view.seed_text);
    perlin_rows(ui, noise, 100.0..=72_000.0, Some("Stretch the lowest value to black and the highest to white.\nOff: the raw values, −1 = black, 1 = white."));

    ui.separator();
    if ui.button("All settings back to defaults").clicked() {
        *noise = PerlinSettings::default();
    }
    ui.separator();
    ui.strong("Noise presets");
    ui.small("Save a useful kind of noise by name. The seed isn't saved with it, so a preset works with any seed.");
    preset_bar(ui, "noise", noise);
}

/// A Perlin field's settings: pattern, placement, maths and output. Shared by Graphics Kit Perlin
/// layers and the World Interpreter's hills and ground detail. `fill_help` shows the "fill black
/// to white" switch (which needs the whole field at once, so the Interpreter leaves it out).
pub fn perlin_rows(ui: &mut egui::Ui, noise: &mut PerlinSettings, sizes: std::ops::RangeInclusive<f32>, fill_help: Option<&str>) {
    section(ui, "Pattern", "What the noise is made of.");
    ui.add(crate::debug::Slider::new(&mut noise.scale, sizes).logarithmic(true).text("size (blocks)"))
        .on_hover_text("The size of the biggest features: one grid square of the first octave.");
    ui.add(crate::debug::Slider::new(&mut noise.octaves, 1..=16).text("octaves"))
        .on_hover_text("How many layers are stacked. Each one adds finer detail.");
    ui.add(crate::debug::Slider::new(&mut noise.persistence, 0.0..=1.0).text("persistence"))
        .on_hover_text("How strong each octave is compared with the one before.\n0.5 = half as strong. Higher = rougher.");
    ui.add(crate::debug::Slider::new(&mut noise.lacunarity, 1.1..=4.0).text("lacunarity"))
        .on_hover_text("How much finer each octave is than the one before.\n2 = twice as fine.");

    placement_rows(
        ui,
        [&mut noise.offset_north, &mut noise.offset_east, &mut noise.stretch, &mut noise.stretch_angle, &mut noise.octave_turn],
    );

    section(ui, "Maths", "How each octave is worked out.");
    ui.horizontal(|ui| {
        ui.label("fade:");
        for fade in Fade::ALL {
            ui.selectable_value(&mut noise.fade, fade, fade.name());
        }
    })
    .response
    .on_hover_text("The curve that blends between grid posts.\nStraight shows the grid; smooth hides it.");
    ui.horizontal_wrapped(|ui| {
        ui.label("solo octave:");
        if ui.selectable_label(noise.solo_octave == 0, "all").clicked() {
            noise.solo_octave = 0;
        }
        for octave in 1..=noise.octaves {
            if ui.selectable_label(noise.solo_octave == octave, octave.to_string()).clicked() {
                noise.solo_octave = octave;
            }
        }
    })
    .response
    .on_hover_text("See one octave on its own, like soloing a channel on a mixing desk.");
    if noise.solo_octave > noise.octaves {
        noise.solo_octave = 0;
    }

    output_settings(ui, &mut noise.output, fill_help);
}

fn image_settings(
    ui: &mut egui::Ui,
    file: &str,
    (width, height): (usize, usize),
    placement: &mut Placement,
    blocks_per_pixel: f32,
    size_blocks: f32,
) {
    ui.small(format!("{file}: {width} × {height} pixels. Its brightness is the layer's values; its own colours are kept too."));

    section(ui, "Placement", "Where the picture sits in the world, and how big.");
    ui.horizontal(|ui| {
        ui.label("centre");
        ui.add(egui::DragValue::new(&mut placement.centre_north).speed(50.0).suffix(" N"));
        ui.add(egui::DragValue::new(&mut placement.centre_east).speed(50.0).suffix(" E"));
    })
    .response
    .on_hover_text("Where the picture's middle is, in blocks from the field's centre. Drag the numbers.");
    ui.add(
        crate::debug::Slider::new(&mut placement.blocks_per_pixel, 1.0..=4096.0)
            .logarithmic(true)
            .text("blocks per picture pixel"),
    );
    ui.small(format!(
        "The picture covers {:.0} × {:.0} blocks.",
        width as f32 * placement.blocks_per_pixel,
        height as f32 * placement.blocks_per_pixel
    ));
    ui.horizontal(|ui| {
        if ui.button("1 picture pixel = 1 field pixel").clicked() {
            placement.blocks_per_pixel = blocks_per_pixel;
        }
        if ui.button("Fit the field").clicked() {
            placement.blocks_per_pixel = size_blocks / width.max(height).max(1) as f32;
            (placement.centre_north, placement.centre_east) = (0.0, 0.0);
        }
    });
    ui.checkbox(&mut placement.repeat, "repeat (tile it across the whole field)");

    output_settings(ui, &mut placement.output, Some("Stretch the picture's darkest pixel to black and its brightest to white."));
    if ui.button("Placement and output back to defaults").clicked() {
        *placement = Placement { blocks_per_pixel: placement.blocks_per_pixel, ..Placement::default() };
    }
}

fn output_settings(ui: &mut egui::Ui, output: &mut Shaping, fill_help: Option<&str>) {
    section(ui, "Output", "Shaping the values, without changing the pattern underneath.");
    if let Some(fill_help) = fill_help {
        ui.checkbox(&mut output.fill_range, "fill black to white").on_hover_text(fill_help);
    }
    ui.add(crate::debug::Slider::new(&mut output.contrast, 0.0..=5.0).text("contrast"));
    ui.add(crate::debug::Slider::new(&mut output.brightness, -1.0..=1.0).text("brightness"));
    ui.add(crate::debug::Slider::new(&mut output.curve, 0.1..=5.0).logarithmic(true).text("curve"))
        .on_hover_text("Above 1: most of the field gets darker, peaks stay bright.\nBelow 1: most of it gets lighter.");
    ui.checkbox(&mut output.invert, "invert");
}

fn section(ui: &mut egui::Ui, title: &str, about: &str) {
    ui.add_space(6.0);
    ui.separator();
    ui.strong(title);
    ui.small(about);
}

fn colour_panel(ui: &mut egui::Ui, colouring: &mut Colouring, is_image: bool, view: &mut ViewState) {
    if is_image {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut colouring.original, true, "the picture's own colours");
            ui.selectable_value(&mut colouring.original, false, "a palette");
        });
    }

    if !colouring.original || !is_image {
        ui.horizontal(|ui| {
            let shown = if colouring.name.is_empty() { format!("default ({} greys)", colouring.shades) } else { colouring.name.clone() };
            egui::ComboBox::from_id_salt("worldgen palette").selected_text(shown).show_ui(ui, |ui| {
                if ui.selectable_label(colouring.name.is_empty(), "default (greys)").clicked() {
                    colouring.name.clear();
                    colouring.colours.clear();
                }
                for name in palette::saved_palettes() {
                    if ui.selectable_label(colouring.name == name, &name).clicked() {
                        match palette::load_palette(&name) {
                            Ok(colours) => {
                                colouring.colours = colours;
                                colouring.name = name;
                            }
                            Err(error) => view.status = Some(error),
                        }
                    }
                }
            });
            if ui.small_button("Palettes folder").on_hover_text("Put GIMP palette (.gpl) files here.\nAseprite: Palette menu → Save Palette, as .gpl.").clicked()
                && let Err(error) = export::open_folder(&palette::palette_folder())
            {
                view.status = Some(format!("Couldn't open the folder: {error}"));
            }
        });
        if colouring.name.is_empty() {
            ui.add(crate::debug::Slider::new(&mut colouring.shades, 2..=256).text("shades of grey"))
                .on_hover_text("The default palette: this many greys, evenly spaced from black to white.");
        } else {
            ui.small("From the palette file:");
            swatches(ui, &colouring.palette());
        }
        ui.small("After the filter (valleys on the left, peaks on the right):");
        let mut filtered = colouring.filtered();
        if colouring.reverse {
            filtered.reverse();
        }
        swatches(ui, &filtered);

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("spread:");
            ui.selectable_value(&mut colouring.spread, Spread::EqualValues, "equal steps of value")
                .on_hover_text("Every colour gets the same slice of black-to-white, like contour bands.");
            ui.selectable_value(&mut colouring.spread, Spread::EqualShares, "equal shares of the picture")
                .on_hover_text("Every colour covers the same amount of the picture.");
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut colouring.blend, "blend between colours")
                .on_hover_text("Off: hard bands (posterised). On: smooth gradients.");
            ui.checkbox(&mut colouring.reverse, "reverse")
                .on_hover_text("First colour on the peaks instead of in the valleys.");
        });
    }

    ui.add_space(4.0);
    ui.strong("HSL filter");
    channel_row(ui, "hue", &mut colouring.hue, 0.0..=360.0, 0.0);
    channel_row(ui, "saturation", &mut colouring.saturation, 0.0..=1.0, 0.0);
    channel_row(ui, "lightness", &mut colouring.lightness, 0.0..=1.0, 0.5);

    if !colouring.original || !is_image {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Save the filtered palette:");
            ui.add(egui::TextEdit::singleline(&mut view.palette_name).desired_width(110.0).hint_text("name"));
            let name: String = view
                .palette_name
                .trim()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
                .collect();
            if ui.add_enabled(!name.is_empty(), egui::Button::new("Save .gpl")).clicked() {
                view.status = Some(match palette::save_palette(&palette::palette_folder(), &name, &colouring.filtered()) {
                    Ok(path) => format!("Saved {}", path.display()),
                    Err(error) => format!("Couldn't save the palette: {error}"),
                });
            }
        });
        ui.small("(\"From the field\" can't be saved: it's different for every pixel.)");
    }
}

/// One HSL channel: keep it, set every colour to one value, or take it from the field.
fn channel_row(ui: &mut egui::Ui, label: &str, channel: &mut Channel, range: std::ops::RangeInclusive<f32>, start: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        if ui.selectable_label(*channel == Channel::Keep, "keep").clicked() {
            *channel = Channel::Keep;
        }
        let is_set = matches!(channel, Channel::Set(_));
        if ui.selectable_label(is_set, "set to").clicked() && !is_set {
            *channel = Channel::Set(start);
        }
        if ui
            .selectable_label(*channel == Channel::FromField, "from the field")
            .on_hover_text("Take it from the layer's value under each pixel.")
            .clicked()
        {
            *channel = Channel::FromField;
        }
        if let Channel::Set(value) = channel {
            ui.add(crate::debug::Slider::new(value, range));
        }
    });
}

fn swatches(ui: &mut egui::Ui, colours: &[[u8; 3]]) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let each = rect.width() / colours.len().max(1) as f32;
    for (i, [r, g, b]) in colours.iter().enumerate() {
        let x = rect.left() + i as f32 * each;
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(x, rect.top()), Pos2::new(x + each + 0.5, rect.bottom())),
            0.0,
            Color32::from_rgb(*r, *g, *b),
        );
    }
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Inside);
}

// ── Right: layers, under the mouse, histogram ──────────────────────────────

/// A button pressed under the Layers list (done after the list is drawn).
enum LayerAction {
    Move(u32, bool),
    Duplicate(u32),
    Delete(u32),
}

fn layers_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.heading("Layers");
    if suite.layers.is_empty() {
        ui.small("None yet. \"+ Add layer\" is in the top left.");
    } else {
        // What's on screen, chosen separately from the open layer: watch one thing while
        // adjusting another.
        ui.horizontal(|ui| {
            ui.label("watching:");
            let mut watch = suite.watch;
            egui::ComboBox::from_id_salt("worldgen watch")
                .selected_text(watch_name(suite, watch))
                .truncate()
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut watch, Watch::Result, "the result")
                        .on_hover_text("All the visible layers mixed, in the result's colours.");
                    for layer in suite.layers.iter().rev() {
                        ui.separator();
                        let up_to = Watch::UpTo(layer.id);
                        ui.selectable_value(&mut watch, up_to, watch_name(suite, up_to))
                            .on_hover_text("The layers from the bottom up to this one, mixed,\nbefore the ones above it mix in. In the result's colours.");
                        let alone = Watch::Alone(layer.id);
                        ui.selectable_value(&mut watch, alone, watch_name(suite, alone))
                            .on_hover_text("Just this layer, in its own colours.");
                    }
                });
            suite.watch = watch;
        })
        .response
        .on_hover_text("What's on screen. It stays put while you open and adjust other layers.");
        let picked = suite.selected == Some(RESULT);
        if ui
            .add(egui::Button::selectable(picked, "▣ The result (all layers mixed)").truncate())
            .on_hover_text("Open the result's colouring and numbers.")
            .clicked()
        {
            suite.selected = Some(RESULT);
        }
    }
    // Top of the stack first, as in image editors.
    let mut action = None;
    for layer in suite.layers.iter_mut().rev() {
        ui.horizontal(|ui| {
            ui.checkbox(&mut layer.visible, "").on_hover_text("Show or hide this layer");
            let picked = suite.selected == Some(layer.id);
            let making = if layer.is_making() { "  …" } else { "" };
            let watched = if suite.watch.layer() == Some(layer.id) { "» " } else { "" };
            let mix = match (&layer.kind, layer.mix, layer.opacity) {
                (LayerKind::Warp { warp, .. }, _, opacity) => {
                    let what = if warp.everything_below { "everything below" } else { "the layer below" };
                    format!("  [{} {what} {:.0}%]", warp.mode.name(), opacity * 100.0)
                }
                (_, Mix::Normal, 1.0) => String::new(),
                (_, mix, opacity) => format!("  [{} {:.0}%]", mix.name(), opacity * 100.0),
            };
            let label = format!("{watched}{}{making}{mix}", layer.name);
            let hover = format!("{}\n{}\nmix: {} at {:.0}%", layer.name, layer.kind.type_name(), layer.mix.name(), layer.opacity * 100.0);
            if ui.add(egui::Button::selectable(picked, label).truncate()).on_hover_text(hover).clicked() {
                suite.selected = Some(layer.id);
            }
        });
    }
    if let Some(id) = suite.selected.filter(|&id| id != RESULT) {
        ui.horizontal(|ui| {
            if ui.small_button("Up").on_hover_text("Move the open layer up the stack").clicked() {
                action = Some(LayerAction::Move(id, true));
            }
            if ui.small_button("Down").clicked() {
                action = Some(LayerAction::Move(id, false));
            }
            if ui.small_button("Duplicate").on_hover_text("Copy the open layer exactly as it is (same seed too), just above it").clicked() {
                action = Some(LayerAction::Duplicate(id));
            }
            if ui.small_button("Delete").clicked() {
                action = Some(LayerAction::Delete(id));
            }
        });
    }
    match action {
        Some(LayerAction::Move(id, up)) => suite.move_layer(id, up),
        Some(LayerAction::Duplicate(id)) => suite.duplicate_layer(id),
        Some(LayerAction::Delete(id)) => suite.remove_layer(id),
        None => {}
    }

    ui.separator();
    ui.heading("Under the mouse");
    let (width, height) = suite.view.size;
    match suite.view.hover {
        Some((col, row)) if col < width && row < height => {
            let i = row * width + col;
            let blocks = suite.blocks_per_pixel();
            let north = (height as f32 / 2.0 - row as f32 - 0.5) * blocks;
            let east = (col as f32 + 0.5 - width as f32 / 2.0) * blocks;
            ui.label(format!("Pixel {col}, {row}"));
            ui.label(format!("{}, {}", compass(north, "N", "S"), compass(east, "E", "W")));
            match suite.view.composite.get(i).copied().flatten() {
                Some([r, g, b]) => {
                    ui.horizontal(|ui| {
                        let (swatch, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
                        ui.painter().rect_filled(swatch, 0.0, Color32::from_rgb(r, g, b));
                        ui.label(format!("colour #{r:02X}{g:02X}{b:02X}"));
                    });
                }
                None => {
                    ui.small("No layer covers this pixel.");
                }
            }
            egui::Grid::new("worldgen inspector").striped(true).show(ui, |ui| {
                if let Some(stacked) = &suite.view.stacked
                    && stacked.shaped.values.len() == width * height
                {
                    ui.strong("result");
                    if stacked.covers(i) {
                        ui.strong(format!("{:.4}", stacked.shaped.values[i]));
                    } else {
                        ui.small("(nothing)");
                    }
                    ui.end_row();
                }
                for layer in suite.layers.iter().rev() {
                    let Some(result) = &layer.result else { continue };
                    if result.made.shaped.values.len() != width * height {
                        continue;
                    }
                    ui.label(short(&layer.name, 18)).on_hover_text(format!("{}\nmix: {}", layer.name, layer.mix.name()));
                    if result.made.covers(i) {
                        ui.label(format!("{:.4}", result.made.shaped.values[i]));
                        ui.small(format!("raw {:+.4}", result.made.raw.values[i]));
                    } else {
                        ui.small("(not here)");
                    }
                    ui.end_row();
                }
            });
        }
        _ => {
            ui.small("Point at the field.");
        }
    }

    ui.separator();
    let stats = if suite.selected == Some(RESULT) {
        ui.heading("The result");
        let Some(stacked) = &suite.view.stacked else { return };
        ui.label(format!("covers {} pixels", stacked.stats.covered));
        &stacked.stats
    } else {
        let Some(layer) = suite.selected_layer() else { return };
        ui.add(egui::Label::new(egui::RichText::new(&layer.name).heading()).truncate()).on_hover_text(&layer.name);
        let Some(result) = &layer.result else {
            ui.small("Being made…");
            return;
        };
        ui.label(format!("made in {:.0} ms, covers {} pixels", result.millis, result.made.stats.covered));
        &result.made.stats
    };
    ui.label(format!("raw from {:+.3} to {:+.3}", stats.raw_low, stats.raw_high));
    ui.label(format!("average value {:.3}", stats.mean));
    let clipped = |share: f32| format!("{:.1}%", share * 100.0);
    ui.label(format!("pure black {}   pure white {}", clipped(stats.clipped_black), clipped(stats.clipped_white)))
        .on_hover_text("Pixels pushed all the way to black or white by the output shaping.\nDetail is lost there.");

    ui.add_space(6.0);
    ui.strong("Histogram");
    ui.small("How many pixels have each value, black (left) to white (right).");
    histogram(ui, &stats.histogram, stats.mean);
}

fn histogram(ui: &mut egui::Ui, counts: &[u32], mean: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 90.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let highest = counts.iter().copied().max().unwrap_or(1).max(1) as f32;
    let bar = rect.width() / counts.len() as f32;
    for (i, &count) in counts.iter().enumerate() {
        let height = count as f32 / highest * (rect.height() - 4.0);
        let shade = (40.0 + 215.0 * i as f32 / (counts.len() - 1) as f32) as u8;
        let x = rect.left() + i as f32 * bar;
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(x, rect.bottom() - height), Pos2::new(x + bar - 0.5, rect.bottom())),
            0.0,
            Color32::from_gray(shade),
        );
    }
    let x = rect.left() + mean * rect.width();
    painter.line_segment([Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())], egui::Stroke::new(1.0, MAGENTA));
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Inside);
    ui.small("magenta line = average");
}

/// 1250.0, "N", "S" → "1,250 N"
pub fn compass(blocks: f32, positive: &str, negative: &str) -> String {
    let text = (blocks.abs().round() as u64).to_string();
    let mut grouped = String::new();
    for (i, digit) in text.chars().enumerate() {
        if i > 0 && (text.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    format!("{grouped} {}", if blocks >= 0.0 { positive } else { negative })
}

// ── The field ──────────────────────────────────────────────────────────────

fn canvas(ui: &mut egui::Ui, suite: &mut Suite) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    suite.view.canvas = rect;

    // The field's size in pixels, from the settings (layers may still be catching up).
    let across = (suite.size_blocks / suite.blocks_per_pixel()).ceil().max(1.0) as usize;
    let (width, height) = (across, across);
    let cells = width * height;

    // Remix the stack and repaint the picture only when something it's made from changed.
    let wanted = picture_key(suite);
    let alone = alone(suite);
    let watching = watch_name(suite, suite.watch);
    let blocks = suite.blocks_per_pixel();
    let Suite { layers: all, colouring, view, watch, .. } = suite;
    if view.composite_for.as_ref() != Some(&wanted) || view.size != (width, height) || view.texture.is_none() {
        let stacked = layers::stack(all, width, blocks, None);
        view.composite = match (alone.map(|i| &all[i]), *watch) {
            (Some(layer), _) => match &layer.result {
                Some(result) if result.made.shaped.values.len() == cells => layers::colour(&result.made, &layer.colouring),
                _ => vec![None; cells],
            },
            (None, Watch::UpTo(id)) => layers::colour(&layers::stack(all, width, blocks, Some(id)), colouring),
            (None, _) => layers::colour(&stacked, colouring),
        };
        view.stacked = Some(stacked);
        view.size = (width, height);
        let pixels: Vec<Color32> = view
            .composite
            .iter()
            .enumerate()
            .map(|(i, colour)| {
                let [r, g, b] = colour.unwrap_or(EMPTY[((i % width) / 4 + (i / width) / 4) % 2]);
                Color32::from_rgb(r, g, b)
            })
            .collect();
        let image = egui::ColorImage::new([width, height], pixels);
        match &mut view.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => view.texture = Some(ui.ctx().load_texture("worldgen field", image, egui::TextureOptions::NEAREST)),
        }
        view.composite_for = Some(wanted);
    }

    view.hover = view.camera.update(ui, rect, &response, (width, height));
    if let Some(texture) = &view.texture {
        let image_rect = view.camera.field_rect(rect, (width, height));
        painter.image(texture.id(), image_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        painter.rect_stroke(image_rect, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Outside);
    }
    if all.is_empty() {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "A blank page: \"+ Add layer\", top left", egui::FontId::proportional(18.0), PINK);
    } else {
        painter.text(
            rect.left_top() + Vec2::new(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            format!("watching: {watching}"),
            egui::FontId::proportional(14.0),
            PINK,
        );
    }

    view.camera.outline(&painter, rect, view.hover, MAGENTA);
    painter.text(
        rect.left_bottom() + Vec2::new(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "drag = move   scroll = zoom   double-click = fit   north is up",
        egui::FontId::proportional(12.0),
        PINK,
    );
}

//! The suite's screen. Placeholder looks: plain panels and the field.
//!
//! ```text
//! ┌──────────── top: field size, pixel size, export ─────────────┐
//! │ + Add layer    │                          │ Layers         │
//! │                │                          │  (top first)   │
//! │ the open       │        the field         │                │
//! │ layer's        │  drag = pan, scroll =    │ under the      │
//! │ settings       │  zoom                    │ mouse          │
//! │                │                          │                │
//! │ its colour     │                          │ histogram      │
//! └────────────────┴──────────────────────────┴────────────────┘
//! ```

use super::export;
use super::image::{self, Placement};
use super::Suite;
use super::layers::{self, LayerKind};
use super::made::Shaping;
use super::noise::{Fade, PerlinSettings};
use super::palette::{self, Channel, Colouring, Spread};
use crate::debug::preset_bar;
use bevy_egui::egui::{self, Color32, Pos2, Rect, Vec2};

const PINK: Color32 = Color32::from_rgb(255, 140, 220);
const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);
/// The checkerboard where no layer covers the field (as in image editors, but dark).
const EMPTY: [[u8; 3]; 2] = [[24, 24, 28], [40, 40, 46]];

pub struct ViewState {
    /// Field position (in pixels) at the middle of the screen, and screen points per pixel.
    centre: Vec2,
    zoom: f32,
    fitted_for: (usize, usize),
    /// Frames left in which to keep fitting the field to the screen (the panels take a few frames
    /// to settle into their sizes when the suite opens).
    settling: u32,
    texture: Option<egui::TextureHandle>,
    /// The stacked picture (one colour per field pixel, `None` = no layer), and what it was
    /// stacked from: exports are cut from it.
    composite: Vec<Option<[u8; 3]>>,
    composite_for: Vec<(u32, bool, u64)>,
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
            centre: Vec2::ZERO,
            zoom: 1.0,
            fitted_for: (0, 0),
            settling: 10,
            texture: None,
            composite: Vec::new(),
            composite_for: Vec::new(),
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

/// Draws the whole screen. Returns true when "Back to the world" was pressed.
pub fn draw(ctx: &egui::Context, suite: &mut Suite) -> bool {
    for layer in &mut suite.layers {
        layer.repaint();
    }
    let mut root = egui::Ui::new(
        ctx.clone(),
        "world gen suite".into(),
        egui::UiBuilder::new().layer_id(egui::LayerId::background()).max_rect(ctx.viewport_rect()),
    );
    let mut close = false;
    egui::Panel::top("worldgen top").show(&mut root, |ui| close = top_bar(ui, suite));
    egui::Panel::left("worldgen settings").resizable(true).default_size(360.0).show(&mut root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| settings_panel(ui, suite));
    });
    egui::Panel::right("worldgen layers").resizable(true).default_size(270.0).show(&mut root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| layers_panel(ui, suite));
    });
    egui::CentralPanel::no_frame().show(&mut root, |ui| canvas(ui, suite));
    close
}

// ── Top bar ────────────────────────────────────────────────────────────────

fn top_bar(ui: &mut egui::Ui, suite: &mut Suite) -> bool {
    let mut close = false;
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(PINK, "WORLD GEN SUITE");
        if ui.button("Back to the world  [F2]").clicked() {
            close = true;
        }
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
        ui.add(egui::Slider::new(&mut suite.chunks_per_pixel, 1..=45).suffix(" chunks"))
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
        ui.separator();
        if suite.is_making() {
            ui.label("Making…");
        }
    });

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
    close
}

/// The field pixels showing on screen: (first column, first row, width, height).
fn visible_pixels(view: &ViewState) -> Option<(usize, usize, usize, usize)> {
    let (width, height) = view.size;
    let half = view.canvas.size() / 2.0 / view.zoom;
    let (left, top) = ((view.centre.x - half.x).floor().max(0.0), (view.centre.y - half.y).floor().max(0.0));
    let (right, bottom) = ((view.centre.x + half.x).ceil().min(width as f32), (view.centre.y + half.y).ceil().min(height as f32));
    (right > left && bottom > top).then(|| (left as usize, top as usize, (right - left) as usize, (bottom - top) as usize))
}

fn export_picture(view: &ViewState, whole: bool) -> String {
    let (width, height) = view.size;
    let (left, top, w, h) = if whole {
        (0, 0, width, height)
    } else {
        match visible_pixels(view) {
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
            if ui.button("Perlin noise field").clicked() {
                suite.add_perlin();
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
    let Suite { layers, selected, view, .. } = suite;
    let Some(layer) = selected.and_then(|id| layers.iter_mut().find(|layer| layer.id == id)) else {
        ui.add_space(20.0);
        if layers.is_empty() {
            ui.label("A blank page.");
            ui.small("Add a layer to start: a Perlin noise field, or an image (drag a PNG onto the window).");
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

    match &mut layer.kind {
        LayerKind::Perlin { seed, settings } => perlin_settings(ui, layer.id, seed, settings, view),
        LayerKind::Image { file, picture, placement } => {
            image_settings(ui, file, (picture.width, picture.height), placement, blocks_per_pixel, size_blocks)
        }
    }

    ui.add_space(10.0);
    ui.separator();
    ui.heading("Colour");
    ui.small("Colours this layer's picture (on screen and in exports). Its values stay as they are.");
    let is_image = matches!(layer.kind, LayerKind::Image { .. });
    colour_panel(ui, &mut layer.colouring, is_image, view);
}

fn perlin_settings(ui: &mut egui::Ui, id: u32, seed: &mut u64, noise: &mut PerlinSettings, view: &mut ViewState) {
    ui.small("Soft hills and hollows from a grid of random arrows, stacked in finer and fainter layers (octaves).");
    ui.horizontal(|ui| {
        ui.label("Seed:");
        let seed_id = egui::Id::new(("worldgen seed", id));
        if ui.ctx().memory(|m| !m.has_focus(seed_id)) {
            view.seed_text = format!("{seed:016X}");
        }
        let edit = ui.add(egui::TextEdit::singleline(&mut view.seed_text).id(seed_id).desired_width(140.0));
        if edit.changed()
            && let Ok(typed) = u64::from_str_radix(view.seed_text.trim().trim_start_matches("0x"), 16)
        {
            *seed = typed;
        }
        if ui.button("New seed").clicked() {
            *seed = crate::fate::derive_seed(super::fresh_number(), "perlin layer");
        }
    });

    section(ui, "Pattern", "What the noise is made of.");
    ui.add(egui::Slider::new(&mut noise.scale, 100.0..=72_000.0).logarithmic(true).text("size (blocks)"))
        .on_hover_text("The size of the biggest features: one grid square of the first octave.");
    ui.add(egui::Slider::new(&mut noise.octaves, 1..=16).text("octaves"))
        .on_hover_text("How many layers are stacked. Each one adds finer detail.");
    ui.add(egui::Slider::new(&mut noise.persistence, 0.0..=1.0).text("persistence"))
        .on_hover_text("How strong each octave is compared with the one before.\n0.5 = half as strong. Higher = rougher.");
    ui.add(egui::Slider::new(&mut noise.lacunarity, 1.1..=4.0).text("lacunarity"))
        .on_hover_text("How much finer each octave is than the one before.\n2 = twice as fine.");

    section(ui, "Placement", "Where the pattern sits and which way it leans.");
    ui.horizontal(|ui| {
        ui.label("slide");
        ui.add(egui::DragValue::new(&mut noise.offset_north).speed(50.0).suffix(" N"));
        ui.add(egui::DragValue::new(&mut noise.offset_east).speed(50.0).suffix(" E"));
    })
    .response
    .on_hover_text("Moves the whole field, in blocks. Drag the numbers.");
    ui.add(egui::Slider::new(&mut noise.stretch, 0.1..=10.0).logarithmic(true).text("stretch"))
        .on_hover_text("1 = round features. 3 = three times as long as they are wide.");
    ui.add(egui::Slider::new(&mut noise.stretch_angle, 0.0..=180.0).suffix("°").text("stretch direction"))
        .on_hover_text("0° = stretched east–west, 90° = north–south.");
    ui.add(egui::Slider::new(&mut noise.octave_turn, 0.0..=90.0).suffix("°").text("turn each octave"))
        .on_hover_text("Turns each octave against the one before, so the grids don't line up\n(hides faint straight lines in the pattern).");

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

    output_settings(ui, &mut noise.output, "Stretch the lowest value to black and the highest to white.\nOff: the raw values, −1 = black, 1 = white.");

    ui.separator();
    if ui.button("All settings back to defaults").clicked() {
        *noise = PerlinSettings::default();
    }
    ui.separator();
    ui.strong("Noise presets");
    ui.small("Save a useful kind of noise by name. The seed isn't saved with it, so a preset works with any seed.");
    preset_bar(ui, "noise", noise);
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
        egui::Slider::new(&mut placement.blocks_per_pixel, 1.0..=4096.0)
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

    output_settings(ui, &mut placement.output, "Stretch the picture's darkest pixel to black and its brightest to white.");
    if ui.button("Placement and output back to defaults").clicked() {
        *placement = Placement { blocks_per_pixel: placement.blocks_per_pixel, ..Placement::default() };
    }
}

fn output_settings(ui: &mut egui::Ui, output: &mut Shaping, fill_help: &str) {
    section(ui, "Output", "Shaping the values, without changing the pattern underneath.");
    ui.checkbox(&mut output.fill_range, "fill black to white").on_hover_text(fill_help);
    ui.add(egui::Slider::new(&mut output.contrast, 0.0..=5.0).text("contrast"));
    ui.add(egui::Slider::new(&mut output.brightness, -1.0..=1.0).text("brightness"));
    ui.add(egui::Slider::new(&mut output.curve, 0.1..=5.0).logarithmic(true).text("curve"))
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
            ui.add(egui::Slider::new(&mut colouring.shades, 2..=256).text("shades of grey"))
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
            ui.add(egui::Slider::new(value, range));
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

fn layers_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.heading("Layers");
    if suite.layers.is_empty() {
        ui.small("None yet. \"+ Add layer\" is in the top left.");
    }
    // Top of the stack first, as in image editors.
    let mut action = None;
    for layer in suite.layers.iter_mut().rev() {
        ui.horizontal(|ui| {
            ui.checkbox(&mut layer.visible, "").on_hover_text("Show or hide this layer");
            let picked = suite.selected == Some(layer.id);
            let label = if layer.is_making() { format!("{}  …", layer.name) } else { layer.name.clone() };
            if ui.selectable_label(picked, label).on_hover_text(layer.kind.type_name()).clicked() {
                suite.selected = Some(layer.id);
            }
        });
    }
    if let Some(id) = suite.selected {
        ui.horizontal(|ui| {
            if ui.small_button("Up").on_hover_text("Move the open layer up the stack").clicked() {
                action = Some((id, Some(true)));
            }
            if ui.small_button("Down").clicked() {
                action = Some((id, Some(false)));
            }
            if ui.small_button("Delete").clicked() {
                action = Some((id, None));
            }
        });
    }
    match action {
        Some((id, Some(up))) => suite.move_layer(id, up),
        Some((id, None)) => suite.remove_layer(id),
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
                for layer in suite.layers.iter().rev() {
                    let Some(result) = &layer.result else { continue };
                    if result.made.shaped.values.len() != width * height {
                        continue;
                    }
                    ui.label(&layer.name);
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
    let Some(layer) = suite.selected_layer() else { return };
    ui.heading(format!("{}", layer.name));
    let Some(result) = &layer.result else {
        ui.small("Being made…");
        return;
    };
    let stats = &result.made.stats;
    ui.label(format!("made in {:.0} ms, covers {} pixels", result.millis, stats.covered));
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
fn compass(blocks: f32, positive: &str, negative: &str) -> String {
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

    // Restack the picture only when a layer's picture, order, or visibility changed.
    let wanted: Vec<(u32, bool, u64)> =
        suite.layers.iter().map(|layer| (layer.id, layer.visible, layer.paint_version)).collect();
    let view = &mut suite.view;
    if wanted != view.composite_for || view.size != (width, height) || view.texture.is_none() {
        view.composite = layers::composite(&suite.layers, cells);
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
        view.composite_for = wanted;
    }

    // Fit the whole field on screen when its size changes (and on double-click).
    let size = Vec2::new(width as f32, height as f32);
    if view.settling > 0 {
        view.settling -= 1;
        view.fitted_for = (0, 0);
        ui.ctx().request_repaint();
    }
    if response.double_clicked() || view.fitted_for != (width, height) {
        if view.fitted_for != (0, 0) && !response.double_clicked() {
            // Same view, new pixel size: keep looking at the same place at the same size.
            let ratio = width as f32 / view.fitted_for.0 as f32;
            view.centre *= ratio;
            view.zoom /= ratio;
        } else {
            view.zoom = (rect.width() / size.x).min(rect.height() / size.y) * 0.95;
            view.centre = size / 2.0;
        }
        view.fitted_for = (width, height);
    }

    // Drag to pan, scroll to zoom (keeping the point under the mouse still).
    if response.dragged() {
        view.centre -= response.drag_delta() / view.zoom;
    }
    let hover = response.hover_pos();
    if let Some(mouse) = hover {
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let under = view.centre + (mouse - rect.center()) / view.zoom;
            view.zoom = (view.zoom * (scroll * 0.003).exp()).clamp(0.1, 256.0);
            view.centre = under - (mouse - rect.center()) / view.zoom;
        }
    }
    let (centre, zoom) = (view.centre, view.zoom);
    let to_screen = |pixel: Vec2| -> Pos2 { rect.center() + (pixel - centre) * zoom };

    if let Some(texture) = &view.texture {
        let image_rect = Rect::from_min_size(to_screen(Vec2::ZERO), size * zoom);
        painter.image(texture.id(), image_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        painter.rect_stroke(image_rect, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Outside);
    }
    if suite.layers.is_empty() {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "A blank page: \"+ Add layer\", top left", egui::FontId::proportional(18.0), PINK);
    }

    view.hover = hover.map(|mouse| centre + (mouse - rect.center()) / zoom).and_then(|pixel| {
        (pixel.x >= 0.0 && pixel.y >= 0.0 && pixel.x < size.x && pixel.y < size.y).then_some((pixel.x as usize, pixel.y as usize))
    });
    if let Some((col, row)) = view.hover
        && zoom >= 4.0
    {
        let cell = Rect::from_min_size(to_screen(Vec2::new(col as f32, row as f32)), Vec2::splat(zoom));
        painter.rect_stroke(cell, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Outside);
    }
    painter.text(
        rect.left_bottom() + Vec2::new(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "drag = move   scroll = zoom   double-click = fit   north is up",
        egui::FontId::proportional(12.0),
        PINK,
    );
}

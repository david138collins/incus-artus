//! The suite's screen. Placeholder looks: plain panels and a greyscale field.
//!
//! ```text
//! ┌──────────── top: seed, field size, resolution ────────────┐
//! │ Perlin field   │                          │ under the     │
//! │ settings       │        the field         │ mouse         │
//! │                │  drag = pan, scroll =    │               │
//! │ noise presets  │  zoom                    │ the numbers,  │
//! │                │                          │ histogram     │
//! └────────────────┴──────────────────────────┴───────────────┘
//! ```

use super::Suite;
use super::export;
use super::noise::{Fade, PerlinSettings};
use super::palette::{self, Channel, Colouring, Spread};
use crate::debug::preset_bar;
use bevy_egui::egui::{self, Color32, Pos2, Rect, Vec2};

const PINK: Color32 = Color32::from_rgb(255, 140, 220);
const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);

pub struct ViewState {
    /// Field position (in pixels) at the middle of the screen, and screen points per pixel.
    centre: Vec2,
    zoom: f32,
    fitted_for: (usize, usize),
    texture: Option<egui::TextureHandle>,
    picture_version: u64,
    /// The picture as last painted (one colour per field pixel), what it was painted from, and
    /// where on screen the field is drawn: exports are cut from these.
    painted: Vec<[u8; 3]>,
    painted_for: Option<(u64, Colouring)>,
    canvas: Rect,
    hover: Option<(usize, usize)>,
    seed_text: String,
    /// The last export's result, shown under the buttons.
    status: Option<String>,
    palette_name: String,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            centre: Vec2::ZERO,
            zoom: 1.0,
            fitted_for: (0, 0),
            texture: None,
            picture_version: 0,
            painted: Vec::new(),
            painted_for: None,
            canvas: Rect::NOTHING,
            hover: None,
            seed_text: String::new(),
            status: None,
            palette_name: String::new(),
        }
    }
}

/// The field pixels showing on screen: (first column, first row, width, height).
fn visible_pixels(view: &ViewState, width: usize, height: usize) -> Option<(usize, usize, usize, usize)> {
    let half = view.canvas.size() / 2.0 / view.zoom;
    let (left, top) = ((view.centre.x - half.x).floor().max(0.0), (view.centre.y - half.y).floor().max(0.0));
    let (right, bottom) = ((view.centre.x + half.x).ceil().min(width as f32), (view.centre.y + half.y).ceil().min(height as f32));
    (right > left && bottom > top).then(|| (left as usize, top as usize, (right - left) as usize, (bottom - top) as usize))
}

fn export(view: &ViewState, width: usize, height: usize, whole: bool) -> String {
    let (left, top, w, h) = if whole {
        (0, 0, width, height)
    } else {
        match visible_pixels(view, width, height) {
            Some(area) => area,
            None => return "Nothing in view to export.".into(),
        }
    };
    let mut colours = Vec::with_capacity(w * h);
    for row in top..top + h {
        colours.extend_from_slice(&view.painted[row * width + left..row * width + left + w]);
    }
    match export::save_png(&export::fresh_name("noise"), w, h, &colours) {
        Ok(path) => format!("Saved {w} × {h}: {}", path.display()),
        Err(error) => format!("Couldn't export: {error}"),
    }
}

impl ViewState {
    pub fn picture_changed(&mut self) {
        self.picture_version += 1;
    }
}

/// Draws the whole screen. Returns true when "Back to the world" was pressed.
pub fn draw(ctx: &egui::Context, suite: &mut Suite) -> bool {
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
    egui::Panel::right("worldgen numbers").resizable(true).default_size(260.0).show(&mut root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| numbers_panel(ui, suite));
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

        ui.label("Seed:");
        let seed_id = egui::Id::new("worldgen seed");
        let view = &mut suite.view;
        if ui.ctx().memory(|m| !m.has_focus(seed_id)) {
            view.seed_text = format!("{:016X}", suite.seed);
        }
        let edit = ui.add(egui::TextEdit::singleline(&mut view.seed_text).id(seed_id).desired_width(140.0));
        if edit.changed()
            && let Ok(seed) = u64::from_str_radix(view.seed_text.trim().trim_start_matches("0x"), 16)
        {
            suite.seed = seed;
        }
        if ui.button("New seed").clicked() {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
            suite.seed = crate::fate::derive_seed(now as u64, "world gen seed");
        }
        ui.separator();

        ui.label("Field:");
        ui.add(
            egui::DragValue::new(&mut suite.size_blocks)
                .range(1_000.0..=200_000.0)
                .speed(500.0)
                .suffix(" blocks across"),
        );
        ui.label("1 pixel =");
        for blocks in [256.0, 128.0, 64.0, 32.0] {
            if ui.selectable_label(suite.blocks_per_pixel == blocks, format!("{blocks}")).clicked() {
                suite.blocks_per_pixel = blocks;
            }
        }
        ui.label("blocks");
        ui.separator();
        match (&suite.result, suite.is_making()) {
            (_, true) => ui.label("Making…"),
            (Some(result), false) => ui.label(format!("Made in {:.0} ms", result.millis)),
            (None, false) => ui.label(""),
        };
    });

    ui.horizontal_wrapped(|ui| {
        let size = suite.result.as_ref().map(|r| (r.made.shaped.width, r.made.shaped.height));
        let ready = size.is_some() && !suite.view.painted.is_empty();
        let view = &mut suite.view;
        if ui
            .add_enabled(ready, egui::Button::new("Export what's in view"))
            .on_hover_text("Saves the pixels on screen as a PNG, one field pixel per picture pixel,\ncoloured as you see them.")
            .clicked()
            && let Some((width, height)) = size
        {
            view.status = Some(export(view, width, height, false));
        }
        if ui.add_enabled(ready, egui::Button::new("Export the whole field")).clicked()
            && let Some((width, height)) = size
        {
            view.status = Some(export(view, width, height, true));
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

// ── The Perlin field's settings ────────────────────────────────────────────

fn settings_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.heading("Perlin field");
    ui.small("Soft hills and hollows from a grid of random arrows, stacked in finer and fainter layers (octaves).");

    let noise = &mut suite.noise;
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

    section(ui, "Output", "Shaping the values, without changing the pattern underneath.");
    ui.checkbox(&mut noise.fill_range, "fill black to white")
        .on_hover_text("Stretch the lowest value to black and the highest to white.\nOff: the raw values, −1 = black, 1 = white.");
    ui.add(egui::Slider::new(&mut noise.contrast, 0.0..=5.0).text("contrast"));
    ui.add(egui::Slider::new(&mut noise.brightness, -1.0..=1.0).text("brightness"));
    ui.add(egui::Slider::new(&mut noise.curve, 0.1..=5.0).logarithmic(true).text("curve"))
        .on_hover_text("Above 1: most of the field gets darker, peaks stay bright.\nBelow 1: most of it gets lighter.");
    ui.checkbox(&mut noise.invert, "invert");

    ui.separator();
    if ui.button("All settings back to defaults").clicked() {
        *noise = PerlinSettings::default();
    }

    ui.separator();
    ui.strong("Noise presets");
    ui.small("Save a useful kind of noise by name. The seed isn't saved with it, so a preset works with any seed.");
    preset_bar(ui, "noise", &mut suite.noise);

    ui.add_space(10.0);
    ui.separator();
    ui.heading("Colour");
    ui.small("Colours the picture (on screen and in exports). The field itself stays greyscale.");
    colour_panel(ui, &mut suite.colouring, &mut suite.view);
}

fn colour_panel(ui: &mut egui::Ui, colouring: &mut Colouring, view: &mut ViewState) {
    ui.checkbox(&mut colouring.on, "colour with a palette");

    ui.horizontal(|ui| {
        let shown = if colouring.name.is_empty() { "choose…".to_string() } else { colouring.name.clone() };
        egui::ComboBox::from_id_salt("worldgen palette").selected_text(shown).show_ui(ui, |ui| {
            let names = palette::saved_palettes();
            if names.is_empty() {
                ui.label("(no .gpl files in the palettes folder yet)");
            }
            for name in names {
                if ui.selectable_label(colouring.name == name, &name).clicked() {
                    match palette::load_palette(&name) {
                        Ok(colours) => {
                            colouring.colours = colours;
                            colouring.name = name;
                            colouring.on = true;
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
    if colouring.colours.is_empty() {
        ui.small("No palette loaded.");
        return;
    }

    ui.small("From the palette file:");
    swatches(ui, &colouring.colours);
    ui.small("After the filter (valleys on the left, peaks on the right):");
    let mut filtered = colouring.filtered();
    if colouring.reverse {
        filtered.reverse();
    }
    swatches(ui, &filtered);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("spread:");
        ui.selectable_value(&mut colouring.spread, Spread::EqualShares, "equal shares of the picture")
            .on_hover_text("Every colour covers the same amount of the picture.");
        ui.selectable_value(&mut colouring.spread, Spread::EqualValues, "equal steps of value")
            .on_hover_text("Every colour gets the same slice of black-to-white, like contour bands.");
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut colouring.blend, "blend between colours")
            .on_hover_text("Off: hard bands (posterised). On: smooth gradients.");
        ui.checkbox(&mut colouring.reverse, "reverse")
            .on_hover_text("First colour on the peaks instead of in the valleys.");
    });

    ui.add_space(4.0);
    ui.strong("HSL filter");
    channel_row(ui, "hue", &mut colouring.hue, 0.0..=360.0, 0.0);
    channel_row(ui, "saturation", &mut colouring.saturation, 0.0..=1.0, 0.0);
    channel_row(ui, "lightness", &mut colouring.lightness, 0.0..=1.0, 0.5);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Save the filtered palette:");
        ui.add(egui::TextEdit::singleline(&mut view.palette_name).desired_width(110.0).hint_text("name"));
        let name: String =
            view.palette_name.trim().chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == ' ').collect();
        if ui.add_enabled(!name.is_empty(), egui::Button::new("Save .gpl")).clicked() {
            view.status = Some(match palette::save_palette(&palette::palette_folder(), &name, &filtered) {
                Ok(path) => format!("Saved {}", path.display()),
                Err(error) => format!("Couldn't save the palette: {error}"),
            });
        }
    });
    ui.small("(\"From the field\" can't be saved: it's different for every pixel.)");
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
            .on_hover_text("Take it from the noise under each pixel.")
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
            Rect::from_min_max(Pos2::new(x, rect.top()), Pos2::new(x + each, rect.bottom())),
            0.0,
            Color32::from_rgb(*r, *g, *b),
        );
    }
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Inside);
}

fn section(ui: &mut egui::Ui, title: &str, about: &str) {
    ui.add_space(6.0);
    ui.separator();
    ui.strong(title);
    ui.small(about);
}

// ── Under the mouse, and the numbers ───────────────────────────────────────

fn numbers_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    let Some(result) = &suite.result else {
        ui.label("Making the first field…");
        return;
    };
    let (raw, shaped, stats) = (&result.made.raw, &result.made.shaped, &result.made.stats);

    ui.heading("Under the mouse");
    match suite.view.hover {
        Some((col, row)) if col < shaped.width && row < shaped.height => {
            let (north, east) = shaped.blocks_at(col, row);
            let i = shaped.index(col, row);
            ui.label(format!("Pixel {col}, {row}"));
            ui.label(format!("{}, {}", compass(north, "N", "S"), compass(east, "E", "W")));
            ui.label(format!("raw noise   {:+.4}", raw.values[i]));
            ui.label(format!("value       {:.4}", shaped.values[i]));
            if let Some([r, g, b]) = suite.view.painted.get(i).copied() {
                ui.horizontal(|ui| {
                    let (swatch, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
                    ui.painter().rect_filled(swatch, 0.0, Color32::from_rgb(r, g, b));
                    ui.label(format!("colour #{r:02X}{g:02X}{b:02X}"));
                });
            }
        }
        _ => {
            ui.small("Point at the field.");
        }
    }

    ui.separator();
    ui.heading("The field");
    ui.label(format!("{} × {} pixels", shaped.width, shaped.height));
    ui.label(format!("raw noise from {:+.3} to {:+.3}", stats.raw_low, stats.raw_high));
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
    let Suite { result, view, colouring, .. } = suite;
    view.canvas = rect;
    let Some(result) = result else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "Making the first field…", egui::FontId::proportional(18.0), PINK);
        return;
    };
    let field = &result.made.shaped;

    // Repaint the picture only when a new field has arrived or the colouring changed.
    let wanted = (view.picture_version, colouring.clone());
    if view.painted_for.as_ref() != Some(&wanted) || view.texture.is_none() {
        view.painted = colouring.paint(&field.values, &result.made.sorted);
        let pixels: Vec<Color32> = view.painted.iter().map(|[r, g, b]| Color32::from_rgb(*r, *g, *b)).collect();
        let image = egui::ColorImage::new([field.width, field.height], pixels);
        match &mut view.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => view.texture = Some(ui.ctx().load_texture("worldgen field", image, egui::TextureOptions::NEAREST)),
        }
        view.painted_for = Some(wanted);
    }

    // Fit the whole field on screen when its size changes (and on double-click).
    let size = Vec2::new(field.width as f32, field.height as f32);
    if response.double_clicked() || view.fitted_for != (field.width, field.height) {
        if view.fitted_for != (0, 0) && view.fitted_for != (field.width, field.height) && !response.double_clicked() {
            // Same view, new resolution: keep looking at the same place at the same size.
            let ratio = field.width as f32 / view.fitted_for.0 as f32;
            view.centre *= ratio;
            view.zoom /= ratio;
        } else {
            view.zoom = (rect.width() / size.x).min(rect.height() / size.y) * 0.95;
            view.centre = size / 2.0;
        }
        view.fitted_for = (field.width, field.height);
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
            view.zoom = (view.zoom * (scroll * 0.003).exp()).clamp(0.1, 64.0);
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

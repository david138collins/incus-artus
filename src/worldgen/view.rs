//! The suite's screen. Placeholder looks: plain panels and a pixel map, high contrast.
//!
//! ```text
//! ┌──────────────── top: seed, map size, run controls ────────────────┐
//! │ steps       │                                       │ layers      │
//! │ (settings   │              the map                  │ inspector   │
//! │  of the     │   drag = pan, scroll = zoom           │ (every value│
//! │  open step) │                                       │  under the  │
//! │ colours     │                                       │  mouse)     │
//! │ presets     │                                       │             │
//! ├──────────────── bottom: the generation log ───────────────────────┤
//! ```

use super::colours::{KindColour, Look, Ramp, paint};
use super::map::{LayerData, Map};
use super::Suite;
use crate::debug::preset_bar;
use bevy_egui::egui::{self, Color32, Pos2, Rect, Vec2};
use std::collections::BTreeMap;
use std::time::Instant;

const PINK: Color32 = Color32::from_rgb(255, 140, 220);
const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);

pub struct ViewState {
    /// The layer on show; `None` = the newest one.
    layer: Option<String>,
    ramps: BTreeMap<String, Ramp>,
    borders: bool,
    labels: bool,
    /// Whose map is on show; `None` = the latest step that has one.
    looking_at: Option<usize>,
    /// Map position (in cells) at the middle of the screen, and pixels per cell.
    centre: Vec2,
    zoom: f32,
    fitted_for: (usize, usize),
    texture: Option<egui::TextureHandle>,
    texture_key: String,
    range: Option<(f32, f32)>,
    picture_version: u64,
    hover: Option<(usize, usize)>,
    highlight: Option<(Vec2, Instant)>,
    seed_text: String,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            layer: None,
            ramps: BTreeMap::new(),
            borders: true,
            labels: true,
            looking_at: None,
            centre: Vec2::ZERO,
            zoom: 1.0,
            fitted_for: (0, 0),
            texture: None,
            texture_key: String::new(),
            range: None,
            picture_version: 0,
            hover: None,
            highlight: None,
            seed_text: String::new(),
        }
    }
}

impl ViewState {
    pub fn picture_changed(&mut self) {
        self.picture_version += 1;
    }
}

/// Which step's map is on show.
fn shown_step(suite: &Suite) -> Option<usize> {
    if let Some(index) = suite.view.looking_at
        && suite.results.get(index).is_some_and(|r| r.is_some())
    {
        return Some(index);
    }
    suite.results.iter().rposition(|result| result.is_some())
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
    egui::Panel::bottom("worldgen log").resizable(true).default_size(150.0).show(&mut root, |ui| log(ui, suite));
    egui::Panel::left("worldgen steps").resizable(true).default_size(340.0).show(&mut root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| steps_panel(ui, suite));
    });
    egui::Panel::right("worldgen layers").resizable(true).default_size(250.0).show(&mut root, |ui| {
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

        ui.label("Seed:");
        let view = &mut suite.view;
        if view.seed_text.is_empty() || ui.ctx().memory(|m| !m.has_focus(egui::Id::new("worldgen seed"))) {
            view.seed_text = format!("{:016X}", suite.stack.seed);
        }
        let edit = ui.add(egui::TextEdit::singleline(&mut view.seed_text).id(egui::Id::new("worldgen seed")).desired_width(140.0));
        if edit.changed()
            && let Ok(seed) = u64::from_str_radix(view.seed_text.trim().trim_start_matches("0x"), 16)
        {
            suite.stack.seed = seed;
        }
        if ui.button("New seed").clicked() {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
            suite.stack.seed = crate::fate::derive_seed(now as u64, "world gen seed");
        }
        ui.separator();

        ui.label("World:");
        ui.add(egui::DragValue::new(&mut suite.stack.world_diameter).range(4_000.0..=200_000.0).speed(500.0).suffix(" blocks across"));
        ui.label("1 pixel =");
        for blocks in [256.0, 128.0, 64.0, 32.0] {
            if ui.selectable_label(suite.stack.blocks_per_cell == blocks, format!("{blocks}")).clicked() {
                suite.stack.blocks_per_cell = blocks;
            }
        }
        ui.label("blocks");
    });

    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut suite.auto_run, "Run by itself").on_hover_text(
            "On: any change re-runs the stack from that step down.\nOff: one step at a time, with Next step.",
        );
        if ui.button("Next step").clicked() {
            suite.next_step();
        }
        if ui.button("Run all").clicked() {
            let from = suite.first_unfinished().unwrap_or(suite.steps.len());
            if from < suite.steps.len() {
                suite.request_run(from, None);
            }
        }
        if ui.button("Start over").on_hover_text("Clear every result and run again").clicked() {
            for result in &mut suite.results {
                *result = None;
            }
            suite.view.picture_changed();
            if suite.auto_run {
                suite.request_run(0, None);
            }
        }
        if ui.add_enabled(suite.is_running(), egui::Button::new("Stop")).clicked() {
            suite.stop();
        }
        ui.separator();
        let status = match (suite.running_step(), suite.first_unfinished()) {
            (Some(step), _) => format!("Running {}…", suite.steps.get(step).map_or("", |s| s.name())),
            (None, None) => "Up to date.".to_string(),
            (None, Some(step)) => format!("Waiting at {} (Next step or Run all).", suite.steps[step].name()),
        };
        ui.label(status);
        let total: f64 = suite.results.iter().flatten().map(|r| r.millis).sum();
        ui.label(format!("Total {total:.0} ms"));
    });
    close
}

// ── Steps, colours, presets ────────────────────────────────────────────────

fn steps_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.heading("Steps");
    let running = suite.running_step();
    let shown = shown_step(suite);
    for index in 0..suite.steps.len() {
        ui.horizontal(|ui| {
            let name = suite.steps[index].name();
            let state = match (&suite.results[index], running == Some(index)) {
                (_, true) => "running…".to_string(),
                (Some(result), _) if result.stale => format!("{:.0} ms (old)", result.millis),
                (Some(result), _) => format!("{:.0} ms", result.millis),
                (None, _) => "not run".to_string(),
            };
            if ui.selectable_label(suite.selected == index, format!("{}. {name}", index + 1)).clicked() {
                suite.selected = index;
            }
            ui.small(state);
            let showing = shown == Some(index) && suite.view.looking_at.is_some();
            if ui
                .add_enabled(suite.results[index].is_some(), egui::Button::selectable(showing, "map").small())
                .on_hover_text("Show the map as it was after this step")
                .clicked()
            {
                suite.view.looking_at = if showing { None } else { Some(index) };
            }
        });
    }
    if suite.view.looking_at.is_some() && ui.small_button("Show the latest map again").clicked() {
        suite.view.looking_at = None;
    }

    ui.separator();
    let index = suite.selected.min(suite.steps.len() - 1);
    let step = &mut suite.steps[index];
    ui.strong(step.name());
    ui.small(step.about());
    step.tune(ui);
    ui.horizontal(|ui| {
        if ui.button("Defaults").clicked() {
            suite.steps[index].reset();
        }
        if ui.button("Run from here").clicked() {
            suite.request_run(index, if suite.auto_run { None } else { Some(index) });
        }
    });

    ui.separator();
    egui::CollapsingHeader::new("Colours").show(ui, |ui| colours(ui, &mut suite.stack.palette));

    ui.separator();
    ui.label("The whole stack (seed, map size, colours, every step):");
    let mut preset = suite.preset();
    let before = toml::to_string(&preset).unwrap_or_default();
    preset_bar(ui, "worldgen", &mut preset);
    if toml::to_string(&preset).unwrap_or_default() != before {
        suite.apply_preset(&preset);
    }
}

fn colours(ui: &mut egui::Ui, palette: &mut Vec<KindColour>) {
    ui.small("One colour per kind of land, for every step. Magenta = no colour chosen yet.");
    for colour in palette.iter_mut() {
        ui.horizontal(|ui| {
            ui.color_edit_button_srgb(&mut colour.rgb);
            ui.label(&colour.kind);
        });
    }
    if ui.small_button("Back to the starting colours").clicked() {
        *palette = super::colours::default_palette();
    }
}

// ── Layers and inspector ───────────────────────────────────────────────────

fn layers_panel(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.heading("Layers");
    let Some(index) = shown_step(suite) else {
        ui.label("Nothing made yet.");
        return;
    };
    let Some(result) = &suite.results[index] else { return };
    let map = &result.map;
    ui.small(format!("After step {}: {}", index + 1, suite.steps[index].name()));
    let view = &mut suite.view;
    if ui.selectable_label(view.layer.is_none(), "(the newest layer)").clicked() {
        view.layer = None;
    }
    for layer in &map.layers {
        let sort = match layer.data {
            LayerData::Numbers(_) => "numbers",
            LayerData::Kinds { .. } => "kinds",
            LayerData::Regions { .. } => "regions",
        };
        let picked = view.layer.as_deref() == Some(layer.name.as_str());
        if ui.selectable_label(picked, format!("{}   ({sort})", layer.name)).clicked() {
            view.layer = Some(layer.name.clone());
        }
    }

    let showing = shown_layer(map, view);
    if let Some(layer) = showing.and_then(|name| map.layer(name))
        && matches!(layer.data, LayerData::Numbers(_))
    {
        let ramp = view.ramps.entry(layer.name.clone()).or_default();
        ui.horizontal(|ui| {
            ui.label("Ramp:");
            for choice in Ramp::ALL {
                ui.selectable_value(ramp, choice, choice.name());
            }
        });
        if let Some((low, high)) = view.range {
            ui.small(format!("black = {low:.3}, brightest = {high:.3}"));
        }
    }
    ui.checkbox(&mut view.borders, "Region borders");
    ui.checkbox(&mut view.labels, "Names and marks");

    ui.separator();
    ui.heading("Under the mouse");
    match view.hover {
        Some((col, row)) if col < map.width && row < map.height => {
            let (north, east) = map.blocks_at(col, row);
            ui.label(format!("Cell {col}, {row}"));
            ui.label(format!("{}, {}", compass(north, "N", "S"), compass(east, "E", "W")));
            if !map.in_world(col, row) {
                ui.colored_label(PINK, "Off the edge of the world");
            }
            egui::Grid::new("inspector").striped(true).show(ui, |ui| {
                for (name, value) in map.describe_cell(col, row) {
                    ui.label(name);
                    ui.label(value);
                    ui.end_row();
                }
            });
        }
        _ => {
            ui.small("Point at the map.");
        }
    }
}

/// 1250.0, "N", "S" → "1,250 N"
fn compass(blocks: f32, positive: &str, negative: &str) -> String {
    let whole = blocks.abs().round() as u64;
    let text = whole.to_string();
    let mut grouped = String::new();
    for (i, digit) in text.chars().enumerate() {
        if i > 0 && (text.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    format!("{grouped} {}", if blocks >= 0.0 { positive } else { negative })
}

fn shown_layer<'a>(map: &'a Map, view: &'a ViewState) -> Option<&'a str> {
    match &view.layer {
        Some(name) if map.layer(name).is_some() => Some(name.as_str()),
        _ => map.layers.last().map(|layer| layer.name.as_str()),
    }
}

// ── Log ────────────────────────────────────────────────────────────────────

fn log(ui: &mut egui::Ui, suite: &mut Suite) {
    ui.horizontal(|ui| {
        ui.strong("Log");
        ui.small("(lines with ⌖ show where on the map when clicked)");
    });
    let mut jump = None;
    egui::ScrollArea::vertical().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
        for result in suite.results.iter().flatten() {
            for line in &result.lines {
                let text = format!("{}: {}{}", line.step, if line.at.is_some() { "⌖ " } else { "" }, line.text);
                let text = if result.stale { egui::RichText::new(text).weak() } else { egui::RichText::new(text) };
                let response = ui.add(egui::Label::new(text).sense(egui::Sense::click()));
                if let Some(at) = line.at
                    && response.clicked()
                {
                    jump = Some(Vec2::new(at.0, at.1));
                }
            }
        }
    });
    if let Some(at) = jump {
        suite.view.centre = at;
        suite.view.zoom = suite.view.zoom.max(4.0);
        suite.view.highlight = Some((at, Instant::now()));
    }
}

// ── The map ────────────────────────────────────────────────────────────────

fn canvas(ui: &mut egui::Ui, suite: &mut Suite) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let Some(index) = shown_step(suite) else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "Nothing made yet", egui::FontId::proportional(18.0), PINK);
        return;
    };
    let Suite { results, stack, view, .. } = suite;
    let Some(result) = &results[index] else { return };
    let map = &result.map;

    // Kinds without a colour join the palette (as magenta), so they can be given one.
    if let Some(name) = shown_layer(map, view)
        && let Some((_, names)) = map.kinds(name)
    {
        for kind in names {
            if !stack.palette.iter().any(|c| &c.kind == kind) {
                stack.palette.push(KindColour { kind: kind.clone(), rgb: [255, 0, 255] });
            }
        }
    }

    // Repaint the picture only when something it depends on changed.
    let layer = shown_layer(map, view).unwrap_or("").to_string();
    let ramp = view.ramps.get(&layer).copied().unwrap_or_default();
    let key = format!("{index}:{}:{layer}:{ramp:?}:{}:{:?}", view.picture_version, view.borders, stack.palette);
    if key != view.texture_key || view.texture.is_none() {
        let look = Look { layer: &layer, ramp, palette: &stack.palette, borders: view.borders };
        let (pixels, range) = paint(map, &look);
        let image = egui::ColorImage::from_rgba_unmultiplied([map.width, map.height], &pixels);
        match &mut view.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => view.texture = Some(ui.ctx().load_texture("worldgen map", image, egui::TextureOptions::NEAREST)),
        }
        view.range = range;
        view.texture_key = key;
    }

    // Fit the whole map on screen when its size changes.
    let size = Vec2::new(map.width as f32, map.height as f32);
    if view.fitted_for != (map.width, map.height) {
        view.zoom = (rect.width() / size.x).min(rect.height() / size.y) * 0.95;
        view.centre = size / 2.0;
        view.fitted_for = (map.width, map.height);
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
    if response.double_clicked() {
        view.fitted_for = (0, 0);
    }
    let (centre, zoom) = (view.centre, view.zoom);
    let to_screen = |cell: Vec2| -> Pos2 { rect.center() + (cell - centre) * zoom };

    if let Some(texture) = &view.texture {
        let image_rect = Rect::from_min_size(to_screen(Vec2::ZERO), size * zoom);
        painter.image(texture.id(), image_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        painter.rect_stroke(image_rect, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Outside);
    }

    // Names and marks, skipping any that would overlap one already drawn.
    if view.labels {
        let font = egui::FontId::proportional(12.0);
        let mut taken: Vec<Rect> = Vec::new();
        for mark in &map.marks {
            let at = to_screen(Vec2::new(mark.col, mark.row));
            if !rect.contains(at) {
                continue;
            }
            let galley = painter.layout_no_wrap(mark.label.clone(), font.clone(), PINK);
            let label = Rect::from_center_size(at + Vec2::new(0.0, -11.0), galley.size() + Vec2::new(6.0, 2.0));
            if taken.iter().any(|other| other.intersects(label)) {
                continue;
            }
            taken.push(label);
            painter.circle_filled(at, 2.5, PINK);
            painter.rect_filled(label, 0.0, Color32::from_black_alpha(200));
            painter.galley(label.min + Vec2::new(3.0, 1.0), galley, PINK);
        }
    }

    // The cell under the mouse, and a flash where a clicked log line points.
    view.hover = hover.map(|mouse| view.centre + (mouse - rect.center()) / view.zoom).and_then(|cell| {
        (cell.x >= 0.0 && cell.y >= 0.0 && cell.x < size.x && cell.y < size.y).then_some((cell.x as usize, cell.y as usize))
    });
    if let Some((col, row)) = view.hover
        && zoom >= 4.0
    {
        let cell = Rect::from_min_size(to_screen(Vec2::new(col as f32, row as f32)), Vec2::splat(zoom));
        painter.rect_stroke(cell, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Outside);
    }
    if let Some((at, when)) = view.highlight {
        let age = when.elapsed().as_secs_f32();
        if age < 3.0 {
            painter.circle_stroke(to_screen(at), 12.0 + age * 6.0, egui::Stroke::new(2.0, MAGENTA));
            ui.ctx().request_repaint();
        } else {
            view.highlight = None;
        }
    }
    painter.text(
        rect.left_bottom() + Vec2::new(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "drag = move   scroll = zoom   double-click = fit   north is up",
        egui::FontId::proportional(12.0),
        PINK,
    );
}

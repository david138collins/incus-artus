//! The software map: the game's wiring, live [David, 2026-11-12: "the software map should just be a black screen with
//! magenta boxes that show a tooltip when you hover, connected by lines, and inside of larger boxes for sections if need
//! be"].
//!
//! Drawn from `wiring.rs`, the same list the game runs from, so it can't disagree with the code. Each lane is a larger
//! box with its passes inside, in the order they run; the stores sit below, each joined by a line to every pass that
//! writes or reads it. Hovering a box shows its card, with its times now. (The lanes as the larger boxes and the stores
//! below them are Builder's reading of "sections", David's to change.)

use crate::clocks::{PassTimes, Spread};
use crate::wiring::{self, LANES, PASSES, STORES, Where};
use egui::{Color32, FontId, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};

const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);
/// Around a box's name.
const PAD: Vec2 = vec2(10.0, 6.0);
/// Between boxes side by side.
const GAP: f32 = 24.0;
/// Inside a lane's box, around its passes.
const INSIDE: f32 = 14.0;
/// Between the lanes and the stores below them.
const BELOW: f32 = 100.0;
/// Around the whole map.
const MARGIN: f32 = 40.0;

pub fn draw(ui: &mut egui::Ui, times: &[PassTimes]) {
    let area = ui.max_rect();
    let painter = ui.painter().clone();
    painter.rect_filled(area, 0.0, Color32::BLACK);
    let font = FontId::proportional(15.0);
    let size_of = |text: &str| painter.layout_no_wrap(text.to_string(), font.clone(), MAGENTA).size();
    let line = Stroke::new(1.0, MAGENTA);

    // Laid out from (0, 0), then moved to the middle of the screen. The lanes in a row, each with its passes in a row.
    let title = size_of("Ag").y + 2.0 * PAD.y;
    let mut passes = vec![Rect::NOTHING; PASSES.len()];
    let mut lanes = Vec::new();
    let mut x = 0.0;
    for lane in &LANES {
        let left = x;
        let mut right = left + INSIDE;
        let mut bottom = title;
        for pass in wiring::passes_in(lane.id) {
            let size = size_of(pass.name) + 2.0 * PAD;
            let rect = Rect::from_min_size(pos2(right, title), size);
            passes[pass.id as usize] = rect;
            right = rect.right() + GAP;
            bottom = bottom.max(rect.bottom());
        }
        let width = (right - GAP + INSIDE - left).max(size_of(lane.name).x + 2.0 * PAD.x);
        lanes.push(Rect::from_min_max(pos2(left, 0.0), pos2(left + width, bottom + INSIDE)));
        x = left + width + 2.0 * GAP;
    }
    // The stores below, each under the middle of the passes it's joined to, kept apart.
    let lanes_bottom = lanes.iter().map(|rect| rect.bottom()).fold(0.0, f32::max);
    let mut order: Vec<(f32, usize)> = STORES
        .iter()
        .enumerate()
        .map(|(k, store)| {
            let joined: Vec<f32> = wiring::writers(store.id).chain(wiring::readers(store.id)).map(|pass| passes[pass.id as usize].center().x).collect();
            (if joined.is_empty() { f32::MAX } else { joined.iter().sum::<f32>() / joined.len() as f32 }, k)
        })
        .collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut stores = vec![Rect::NOTHING; STORES.len()];
    let mut next = 0.0;
    for (wanted, k) in order {
        let size = size_of(STORES[k].name) + 2.0 * PAD;
        let left = if wanted == f32::MAX { next } else { (wanted - size.x / 2.0).max(next) };
        stores[k] = Rect::from_min_size(pos2(left, lanes_bottom + BELOW), size);
        next = stores[k].right() + GAP;
    }
    let whole = lanes.iter().chain(&stores).fold(Rect::NOTHING, |all, rect| all.union(*rect));
    let left = (area.center().x - whole.width() / 2.0).max(area.left() + MARGIN);
    let shift = vec2(left - whole.left(), area.top() + MARGIN - whole.top());
    let lanes: Vec<Rect> = lanes.iter().map(|rect| rect.translate(shift)).collect();
    let passes: Vec<Rect> = passes.iter().map(|rect| rect.translate(shift)).collect();
    let stores: Vec<Rect> = stores.iter().map(|rect| rect.translate(shift)).collect();

    // The lines first, so the boxes sit over them.
    for pass in &PASSES {
        for store in pass.reads.iter().chain(pass.writes) {
            painter.line_segment([passes[pass.id as usize].center_bottom(), stores[*store as usize].center_top()], line);
        }
    }
    for (lane, rect) in LANES.iter().zip(&lanes) {
        painter.rect_stroke(*rect, 0.0, line, StrokeKind::Inside);
        painter.text(rect.left_top() + PAD, egui::Align2::LEFT_TOP, lane.name, font.clone(), MAGENTA);
        let strip = Rect::from_min_size(rect.left_top(), vec2(rect.width(), title));
        ui.interact(strip, ui.id().with(("lane", lane.id as usize)), egui::Sense::hover()).on_hover_ui(|ui| lane_card(ui, lane));
    }
    for (pass, rect) in PASSES.iter().zip(&passes) {
        a_box(&painter, *rect, pass.name, &font);
        ui.interact(*rect, ui.id().with(("pass", pass.id as usize)), egui::Sense::hover()).on_hover_ui(|ui| pass_card(ui, pass, &times[pass.id as usize]));
    }
    for (store, rect) in STORES.iter().zip(&stores) {
        a_box(&painter, *rect, store.name, &font);
        ui.interact(*rect, ui.id().with(("store", store.id as usize)), egui::Sense::hover()).on_hover_ui(|ui| store_card(ui, store));
    }
}

/// A box: black inside, a magenta edge, its name in the middle.
fn a_box(painter: &egui::Painter, rect: Rect, name: &str, font: &FontId) {
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, MAGENTA), StrokeKind::Inside);
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, name, font.clone(), MAGENTA);
}

fn switches(ui: &mut egui::Ui, switches: &[wiring::Switch]) {
    for (name, what) in switches {
        ui.label(format!("Switch {name}: {what}."));
    }
}

fn files(ui: &mut egui::Ui, files: &[&str]) {
    ui.label(format!("Files: {}.", wiring::joined(files.iter().copied())));
}

fn lane_card(ui: &mut egui::Ui, lane: &wiring::Lane) {
    ui.strong(lane.name);
    ui.label(format!("A lane: it runs {}.", lane.runs));
    ui.label(lane.what);
    ui.label(format!("Its passes, in order: {}.", wiring::joined(wiring::passes_in(lane.id).map(|pass| pass.name))));
    switches(ui, lane.switches);
    files(ui, lane.files);
}

fn pass_card(ui: &mut egui::Ui, pass: &wiring::Pass, times: &PassTimes) {
    ui.strong(pass.name);
    ui.label(format!("A pass in {}, on {}.", wiring::lane(pass.lane).name, pass.on.name()));
    ui.label(pass.what);
    ui.label(format!("Reads: {}. Writes: {}.", wiring::joined(pass.reads.iter().map(|store| wiring::store(*store).name)), wiring::joined(pass.writes.iter().map(|store| wiring::store(*store).name))));
    switches(ui, pass.switches);
    let now = |spread: &Spread| if spread.count == 0 { "not yet run".to_string() } else { format!("median {:.3} ms, 99th {:.3}", spread.median, spread.p99) };
    let on_processor = if pass.on == Where::Card { "Recording it, on the processor" } else { "On the processor" };
    ui.label(format!("Now: {on_processor}, {}.", now(&times.processor)));
    if let Some(card) = &times.card {
        ui.label(format!("Now: on the card, {}.", now(card)));
    }
    files(ui, pass.files);
}

fn store_card(ui: &mut egui::Ui, store: &wiring::Store) {
    ui.strong(store.name);
    ui.label(format!("A store on {}.", store.on.name()));
    ui.label(store.what);
    ui.label(format!("Written by: {}. Read by: {}.", wiring::joined(wiring::writers(store.id).map(|pass| pass.name)), wiring::joined(wiring::readers(store.id).map(|pass| pass.name))));
    for (name, value) in store.limits {
        ui.label(format!("Limit: {name}, {value}."));
    }
    files(ui, store.files);
}

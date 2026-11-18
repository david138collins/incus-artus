//! The software map: the game's wiring, live [David, 2026-11-12: "the software map should just be a black screen with
//! magenta boxes that show a tooltip when you hover, connected by lines, and inside of larger boxes for sections if need
//! be"].
//!
//! Drawn from `wiring.rs`, the same list the game runs from, so it can't disagree with the code. Each lane is a larger
//! box with its passes inside, in the order they run; the stores sit below, each joined by a line to every pass that
//! writes or reads it. Hovering a box shows its card, with its times now. (The lanes as the larger boxes and the stores
//! below them are Builder's reading of "sections", David's to change.)
//!
//! Wrapped to the screen's width, and scrolled up and down where it's taller than the screen (build 220) [David,
//! 2026-11-18: "we need to give some love to the software map; it needs to smart-wrap and let me scroll downwards and
//! upwards"]. The lanes flow like words: side by side while they fit, then onto the next row; a lane wider than the
//! screen wraps its passes onto rows inside its box; the stores flow the same way under the lanes, in the order of the
//! passes they're joined to (`layout`; the reading of "smart-wrap" is Builder's, his to change). Wrapped, a line can
//! pass behind a box that isn't its own: hovering a pass or a store draws its lines over the boxes, thicker, and dims
//! the rest.

use crate::clocks::{PassTimes, Spread};
use crate::wiring::{self, LANES, PASSES, STORES, Where};
use egui::{Color32, FontId, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};

const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);
/// The lines that aren't the hovered box's.
const DIM: Color32 = Color32::from_rgb(95, 0, 95);
/// Around a box's name.
const PAD: Vec2 = vec2(10.0, 6.0);
/// Between boxes side by side.
const GAP: f32 = 24.0;
/// Between a lane's rows of passes, between the rows of lanes, and between the rows of stores.
const ROW: f32 = 12.0;
const LANE_ROW: f32 = 28.0;
const STORE_ROW: f32 = 36.0;
/// Inside a lane's box, around its passes.
const INSIDE: f32 = 14.0;
/// Between the lanes and the stores below them.
const BELOW: f32 = 100.0;
/// Around the whole map.
const MARGIN: f32 = 24.0;

/// Where every box sits, from (0, 0) at the map's top left: the lanes', the passes' and the stores', each by its id;
/// the whole map's size; and how tall a lane's title is.
pub struct Layout {
    pub lanes: Vec<Rect>,
    pub passes: Vec<Rect>,
    pub stores: Vec<Rect>,
    pub size: Vec2,
    pub title: f32,
}

/// The map laid out no wider than `width` (a single box wider than that can't be helped); `size_of` a name's size as
/// it's written.
pub fn layout(width: f32, size_of: &dyn Fn(&str) -> Vec2) -> Layout {
    let title = size_of("Ag").y + 2.0 * PAD.y;
    let mut passes = vec![Rect::NOTHING; PASSES.len()];
    let mut lanes = Vec::with_capacity(LANES.len());
    let (mut x, mut y, mut lanes_bottom) = (0.0f32, 0.0f32, 0.0f32);
    for lane in &LANES {
        // The lane by itself first, from (0, 0): its passes in rows no wider than the screen.
        let inner = width - 2.0 * INSIDE;
        let mut inside: Vec<(usize, Rect)> = Vec::new();
        let (mut along, mut down, mut bottom, mut widest) = (0.0f32, title, title, 0.0f32);
        for pass in wiring::passes_in(lane.id) {
            let size = size_of(pass.name) + 2.0 * PAD;
            if along > 0.0 && along + size.x > inner {
                along = 0.0;
                down = bottom + ROW;
            }
            let rect = Rect::from_min_size(pos2(INSIDE + along, down), size);
            inside.push((pass.id as usize, rect));
            along += size.x + GAP;
            bottom = bottom.max(rect.bottom());
            widest = widest.max(rect.right());
        }
        let size = vec2((widest + INSIDE).max(size_of(lane.name).x + 2.0 * PAD.x), bottom + INSIDE);
        // Then among the lanes: on this row if it fits, else at the start of the next.
        if x > 0.0 && x + size.x > width {
            x = 0.0;
            y = lanes_bottom + LANE_ROW;
        }
        for (k, rect) in inside {
            passes[k] = rect.translate(vec2(x, y));
        }
        let rect = Rect::from_min_size(pos2(x, y), size);
        lanes.push(rect);
        x = rect.right() + 2.0 * GAP;
        lanes_bottom = lanes_bottom.max(rect.bottom());
    }
    // The stores below, in the order of the passes they're joined to (the middle of their places in the frame), flowing
    // the same way.
    let mut order: Vec<(f32, usize)> = STORES
        .iter()
        .enumerate()
        .map(|(k, store)| {
            let joined: Vec<f32> = wiring::writers(store.id).chain(wiring::readers(store.id)).map(|pass| pass.id as usize as f32).collect();
            (if joined.is_empty() { f32::MAX } else { joined.iter().sum::<f32>() / joined.len() as f32 }, k)
        })
        .collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut stores = vec![Rect::NOTHING; STORES.len()];
    let (mut x, mut y) = (0.0f32, lanes_bottom + BELOW);
    let mut bottom = y;
    for (_, k) in order {
        let size = size_of(STORES[k].name) + 2.0 * PAD;
        if x > 0.0 && x + size.x > width {
            x = 0.0;
            y = bottom + STORE_ROW;
        }
        stores[k] = Rect::from_min_size(pos2(x, y), size);
        x += size.x + GAP;
        bottom = bottom.max(stores[k].bottom());
    }
    let whole = lanes.iter().chain(&stores).fold(Rect::NOTHING, |all, rect| all.union(*rect));
    Layout { lanes, passes, stores, size: whole.max.to_vec2(), title }
}

pub fn draw(ui: &mut egui::Ui, times: &[PassTimes]) {
    ui.painter().rect_filled(ui.max_rect(), 0.0, Color32::BLACK);
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let painter = ui.painter().clone();
        let font = FontId::proportional(15.0);
        let size_of = |text: &str| painter.layout_no_wrap(text.to_string(), font.clone(), MAGENTA).size();
        // Laid out from (0, 0) to the room there is, then moved into the scrolled space, in its middle where it's
        // narrower than the screen.
        let width = (ui.available_width() - 2.0 * MARGIN).max(200.0);
        let map = layout(width, &size_of);
        let (space, _) = ui.allocate_exact_size(vec2(ui.available_width(), map.size.y + 2.0 * MARGIN), egui::Sense::hover());
        let shift = vec2(space.left() + MARGIN + ((width - map.size.x) / 2.0).max(0.0), space.top() + MARGIN);
        let moved = |rects: &[Rect]| -> Vec<Rect> { rects.iter().map(|rect| rect.translate(shift)).collect() };
        let (lanes, passes, stores) = (moved(&map.lanes), moved(&map.passes), moved(&map.stores));

        // What the mouse is over, before anything is drawn: a hovered pass's or store's lines are drawn last.
        let over_lanes: Vec<egui::Response> = LANES.iter().zip(&lanes).map(|(lane, rect)| ui.interact(Rect::from_min_size(rect.left_top(), vec2(rect.width(), map.title)), ui.id().with(("lane", lane.id as usize)), egui::Sense::hover())).collect();
        let over_passes: Vec<egui::Response> = PASSES.iter().zip(&passes).map(|(pass, rect)| ui.interact(*rect, ui.id().with(("pass", pass.id as usize)), egui::Sense::hover())).collect();
        let over_stores: Vec<egui::Response> = STORES.iter().zip(&stores).map(|(store, rect)| ui.interact(*rect, ui.id().with(("store", store.id as usize)), egui::Sense::hover())).collect();
        let pass_over = over_passes.iter().position(|over| over.hovered());
        let store_over = over_stores.iter().position(|over| over.hovered());
        let any_over = pass_over.is_some() || store_over.is_some();

        // The lines first, so the boxes sit over them; the hovered box's kept for last.
        let line = Stroke::new(1.0, MAGENTA);
        let mut lit = Vec::new();
        for pass in &PASSES {
            for store in pass.reads.iter().chain(pass.writes) {
                let ends = [passes[pass.id as usize].center_bottom(), stores[*store as usize].center_top()];
                if pass_over == Some(pass.id as usize) || store_over == Some(*store as usize) {
                    lit.push(ends);
                } else {
                    painter.line_segment(ends, if any_over { Stroke::new(1.0, DIM) } else { line });
                }
            }
        }
        for (lane, rect) in LANES.iter().zip(&lanes) {
            painter.rect_stroke(*rect, 0.0, line, StrokeKind::Inside);
            painter.text(rect.left_top() + PAD, egui::Align2::LEFT_TOP, lane.name, font.clone(), MAGENTA);
        }
        for (pass, rect) in PASSES.iter().zip(&passes) {
            a_box(&painter, *rect, pass.name, &font);
        }
        for (store, rect) in STORES.iter().zip(&stores) {
            a_box(&painter, *rect, store.name, &font);
        }
        for ends in lit {
            painter.line_segment(ends, Stroke::new(2.0, MAGENTA));
        }

        // The cards.
        for (lane, over) in LANES.iter().zip(over_lanes) {
            over.on_hover_ui(|ui| lane_card(ui, lane));
        }
        for (pass, over) in PASSES.iter().zip(over_passes) {
            over.on_hover_ui(|ui| pass_card(ui, pass, &times[pass.id as usize]));
        }
        for (store, over) in STORES.iter().zip(over_stores) {
            over.on_hover_ui(|ui| store_card(ui, store));
        }
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A name's size at about the map's own letters, without a card to lay them out.
    fn size_of(text: &str) -> Vec2 {
        vec2(text.chars().count() as f32 * 7.5, 18.0)
    }

    /// The map at four widths of window (build 220): every box inside the width, no two passes or stores or lanes
    /// overlapping, every pass inside its lane's box, the stores under the lanes, and the lanes and each lane's passes
    /// still in their order, read as a page is. Wide enough for it all, it's the one row it was.
    #[test]
    fn the_map_wraps_to_the_width() {
        let apart = |a: &Rect, b: &Rect| !a.intersects(*b) || a.intersect(*b).area() < 0.01;
        let reads_after = |a: &Rect, b: &Rect| b.top() > a.top() + 0.5 || ((b.top() - a.top()).abs() <= 0.5 && b.left() >= a.right());
        for width in [800.0f32, 1280.0, 1920.0, 2560.0, 100_000.0] {
            let map = layout(width, &size_of);
            for (what, rects) in [("lane", &map.lanes), ("pass", &map.passes), ("store", &map.stores)] {
                for (k, rect) in rects.iter().enumerate() {
                    assert!(rect.is_positive() && rect.left() >= 0.0 && rect.right() <= width + 0.5, "at {width}, {what} {k} runs off: {rect:?}");
                    for (j, other) in rects.iter().enumerate().skip(k + 1) {
                        assert!(apart(rect, other), "at {width}, {what}s {k} and {j} overlap: {rect:?} {other:?}");
                    }
                }
            }
            for pass in &PASSES {
                let (lane, rect) = (map.lanes[pass.lane as usize], map.passes[pass.id as usize]);
                assert!(lane.contains_rect(rect) && rect.top() >= lane.top() + map.title - 0.5, "at {width}, {} isn't inside its lane", pass.name);
            }
            for lane in &LANES {
                let inside: Vec<Rect> = wiring::passes_in(lane.id).map(|pass| map.passes[pass.id as usize]).collect();
                assert!(inside.windows(2).all(|pair| reads_after(&pair[0], &pair[1])), "at {width}, {}'s passes are out of order", lane.name);
            }
            assert!(map.lanes.windows(2).all(|pair| reads_after(&pair[0], &pair[1])), "at {width}, the lanes are out of order");
            let lanes_bottom = map.lanes.iter().map(|rect| rect.bottom()).fold(0.0, f32::max);
            assert!(map.stores.iter().all(|rect| rect.top() >= lanes_bottom + BELOW - 0.5), "at {width}, a store isn't under the lanes");
            assert!(map.size.x <= width + 0.5);
            if width == 100_000.0 {
                assert!(map.lanes.iter().all(|rect| rect.top() == 0.0) && map.stores.windows(2).all(|pair| pair[0].top() == pair[1].top()), "with room for it all, one row of lanes and one of stores");
            } else if width <= 1280.0 {
                assert!(map.lanes.iter().any(|rect| rect.top() > 0.0), "at {width}, the lanes wrap");
            }
        }
    }
}

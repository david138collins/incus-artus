//! The envelope editor: a draggable ADSR graph for the debug panel, like a synth's.
//!
//! - Square handles are the corners of the envelope. Drag them sideways to change a stage's
//!   time; drag the sustain corners up and down to change the sustain level.
//! - Round handles sit in the middle of each moving stage. Drag them up or down to bend that
//!   stage's curve; double-click one to straighten it again.
//! - The pink dot is the live envelope, if something is using it right now.
//!
//! The sustain stretch has no length of its own (it lasts as long as the gate is held), so the
//! graph draws it at a fixed width.

use super::envelope::{Envelope, EnvelopeState, Stage, shape};
use bevy_egui::egui;

const PINK: egui::Color32 = egui::Color32::from_rgb(255, 140, 220);
const MAGENTA: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);
const GRID: egui::Color32 = egui::Color32::from_rgb(60, 20, 60);
const LETTERS: egui::Color32 = egui::Color32::from_rgb(150, 70, 150);

/// Draws the editor for `envelope`. `live` is a running copy to show as a moving dot.
pub fn envelope_editor(ui: &mut egui::Ui, id_salt: &str, envelope: &mut Envelope, live: Option<&EnvelopeState>) {
    let id = ui.make_persistent_id(("envelope", id_salt));

    // How many seconds the graph's width shows (kept per editor, between frames).
    let view_id = id.with("view");
    // The first time, zoom to fit this envelope.
    let fit = ((envelope.attack + envelope.decay + envelope.release) * 1.6).clamp(1.0, 30.0);
    let mut view_seconds: f32 = ui.data(|data| data.get_temp(view_id)).unwrap_or(fit);

    let size = egui::vec2(ui.available_width().max(200.0), 140.0);
    let (outer, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let area = outer.shrink(10.0);
    let painter = ui.painter_at(outer);
    painter.rect_filled(outer, 0.0, egui::Color32::BLACK);
    painter.rect_stroke(outer, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Inside);

    let pixels_per_second = area.width() / view_seconds;
    let to_screen = |seconds: f32, level: f32| {
        egui::pos2(
            area.left() + seconds * pixels_per_second,
            area.bottom() - level.clamp(0.0, 1.0) * area.height(),
        )
    };

    // One faint line per second, and the 0 / ½ / 1 levels.
    let mut second = 0.0;
    while second <= view_seconds {
        let x = area.left() + second * pixels_per_second;
        painter.line_segment([egui::pos2(x, area.top()), egui::pos2(x, area.bottom())], (1.0, GRID));
        second += 1.0;
    }
    for level in [0.0, 0.5, 1.0] {
        let y = to_screen(0.0, level).y;
        painter.line_segment([egui::pos2(area.left(), y), egui::pos2(area.right(), y)], (1.0, GRID));
    }

    // Where each stage starts and ends, in seconds along the graph.
    let hold = view_seconds * 0.15;
    let sustain = envelope.sustain.clamp(0.0, 1.0);
    let attack_end = envelope.attack;
    let decay_end = attack_end + envelope.decay;
    let release_start = decay_end + hold;
    let release_end = release_start + envelope.release;

    // The curve itself, sampled finely.
    let mut points = vec![to_screen(0.0, 0.0)];
    let steps = 32;
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        points.push(to_screen(attack_end * t, shape(t, envelope.attack_curve)));
    }
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let level = 1.0 + (sustain - 1.0) * shape(t, envelope.decay_curve);
        points.push(to_screen(attack_end + envelope.decay * t, level));
    }
    points.push(to_screen(release_start, sustain));
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let level = sustain * (1.0 - shape(t, envelope.release_curve));
        points.push(to_screen(release_start + envelope.release * t, level));
    }
    painter.line(points, egui::Stroke::new(2.0, PINK));

    // Stage letters along the top.
    let font = egui::FontId::monospace(11.0);
    for (letter, from, to) in [
        ("A", 0.0, attack_end),
        ("D", attack_end, decay_end),
        ("S", decay_end, release_start),
        ("R", release_start, release_end),
    ] {
        if to > from {
            let x = to_screen((from + to) / 2.0, 1.0).x;
            painter.text(egui::pos2(x, outer.top() + 1.0), egui::Align2::CENTER_TOP, letter, font.clone(), LETTERS);
        }
    }

    // ── Corner handles (square) ──
    let handle = |ui: &mut egui::Ui, name: &str, position: egui::Pos2| {
        let response = ui.interact(
            egui::Rect::from_center_size(position, egui::vec2(14.0, 14.0)),
            id.with(name),
            egui::Sense::click_and_drag(),
        );
        let color = if response.hovered() || response.dragged() { egui::Color32::WHITE } else { MAGENTA };
        painter.rect_filled(egui::Rect::from_center_size(position, egui::vec2(8.0, 8.0)), 0.0, color);
        response
    };
    let seconds = |pixels: f32| pixels / pixels_per_second;
    let levels = |pixels: f32| -pixels / area.height();

    // The decay corner goes first: when decay is 0 and sustain is 1 it sits under the peak,
    // and the handle made last is the one that gets grabbed.
    let decay_corner = handle(ui, "decay", to_screen(decay_end, sustain));
    envelope.decay = (envelope.decay + seconds(decay_corner.drag_delta().x)).max(0.0);
    envelope.sustain = (envelope.sustain + levels(decay_corner.drag_delta().y)).clamp(0.0, 1.0);

    let peak = handle(ui, "peak", to_screen(attack_end, 1.0));
    envelope.attack = (envelope.attack + seconds(peak.drag_delta().x)).max(0.0);

    let release_corner = handle(ui, "release_start", to_screen(release_start, sustain));
    envelope.sustain = (envelope.sustain + levels(release_corner.drag_delta().y)).clamp(0.0, 1.0);

    let end = handle(ui, "end", to_screen(release_end, 0.0));
    envelope.release = (envelope.release + seconds(end.drag_delta().x)).max(0.0);

    // ── Curve handles (round), half-way along each moving stage ──
    // Dragging "up" always bulges the line upwards, whichever way the stage goes.
    let curve_handle = |ui: &mut egui::Ui, name: &str, position: egui::Pos2, curve: &mut f32, rising: bool| {
        let response = ui.interact(
            egui::Rect::from_center_size(position, egui::vec2(14.0, 14.0)),
            id.with(name),
            egui::Sense::click_and_drag(),
        );
        let direction = if rising { 1.0 } else { -1.0 };
        *curve = (*curve + levels(response.drag_delta().y) * 2.0 * direction).clamp(-1.0, 1.0);
        if response.double_clicked() {
            *curve = 0.0;
        }
        let color = if response.hovered() || response.dragged() { egui::Color32::WHITE } else { MAGENTA };
        painter.circle_stroke(position, 4.0, egui::Stroke::new(1.5, color));
    };
    if envelope.attack > 0.0 {
        let position = to_screen(attack_end * 0.5, shape(0.5, envelope.attack_curve));
        curve_handle(ui, "attack_curve", position, &mut envelope.attack_curve, true);
    }
    if envelope.decay > 0.0 && sustain < 1.0 {
        let level = 1.0 + (sustain - 1.0) * shape(0.5, envelope.decay_curve);
        let position = to_screen(attack_end + envelope.decay * 0.5, level);
        curve_handle(ui, "decay_curve", position, &mut envelope.decay_curve, false);
    }
    if envelope.release > 0.0 && sustain > 0.0 {
        let level = sustain * (1.0 - shape(0.5, envelope.release_curve));
        let position = to_screen(release_start + envelope.release * 0.5, level);
        curve_handle(ui, "release_curve", position, &mut envelope.release_curve, false);
    }

    // ── The live dot ──
    if let Some(state) = live {
        let length = state.stage_length(envelope);
        let progress = if length > 0.0 { (state.elapsed() / length).min(1.0) } else { 1.0 };
        let at = match state.stage {
            Stage::Idle => 0.0,
            Stage::Attack => attack_end * progress,
            Stage::Decay => attack_end + envelope.decay * progress,
            Stage::Sustain => decay_end + hold * 0.5,
            Stage::Release => release_start + envelope.release * progress,
        };
        painter.circle_filled(to_screen(at, state.level), 5.0, PINK);
    }

    // ── Exact numbers, for typing in or fine nudges ──
    egui::Grid::new(id.with("numbers")).num_columns(4).show(ui, |ui| {
        fn time(value: &mut f32) -> egui::DragValue<'_> {
            egui::DragValue::new(value).speed(0.01).range(0.0..=60.0).suffix(" s")
        }
        fn curve(value: &mut f32) -> egui::DragValue<'_> {
            egui::DragValue::new(value).speed(0.01).range(-1.0..=1.0)
        }
        ui.label("attack");
        ui.add(time(&mut envelope.attack));
        ui.label("curve");
        ui.add(curve(&mut envelope.attack_curve));
        ui.end_row();
        ui.label("decay");
        ui.add(time(&mut envelope.decay));
        ui.label("curve");
        ui.add(curve(&mut envelope.decay_curve));
        ui.end_row();
        ui.label("sustain");
        ui.add(egui::DragValue::new(&mut envelope.sustain).speed(0.01).range(0.0..=1.0));
        ui.end_row();
        ui.label("release");
        ui.add(time(&mut envelope.release));
        ui.label("curve");
        ui.add(curve(&mut envelope.release_curve));
        ui.end_row();
    });
    ui.add(egui::Slider::new(&mut view_seconds, 0.25..=30.0).logarithmic(true).text("graph shows (s)"));
    ui.data_mut(|data| data.insert_temp(view_id, view_seconds));
}

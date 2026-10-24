//! Performance — the gauge for "are we hitting the wall?" [David: "I need to know when we start
//! hitting that wall so I can start hitting the brakes."]
//!
//! The "Performance" debug page shows:
//! - **frame rate**: the average, the **1% low** (the slowest 1 frame in 100, which is what you
//!   feel as stutter), and the **worst frame** of the last few seconds;
//! - a **graph** of the last few seconds of frame times, with lines at 240 and 60 frames a second;
//! - **the work in flight**: chunks waiting and being made, sections waiting to be meshed;
//! - **what's being drawn**: sections, rectangles ("quads") and corners ("vertices").
//!
//! The budget it reads against: 240 FPS (4.2 ms a frame) on David's laptop, as a stand-in for "plays
//! on a GTX 980 Ti at 1080p on low settings".

use super::meshing::{MeshingState, SectionMesh};
use crate::world::{Streaming, VoxelWorld};
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::VecDeque;

/// How many recent frames are kept (about five seconds at 240 FPS).
const KEPT: usize = 1200;
/// The target: 240 frames a second.
const BUDGET_MS: f32 = 1000.0 / 240.0;

/// Recent frame times, in milliseconds, newest last.
#[derive(Resource, Default)]
pub struct FrameTimes(VecDeque<f32>);

pub fn record_frame_time(time: Res<Time<Real>>, mut times: ResMut<FrameTimes>) {
    let ms = time.delta_secs() * 1000.0;
    if ms <= 0.0 {
        return;
    }
    times.0.push_back(ms);
    while times.0.len() > KEPT {
        times.0.pop_front();
    }
}

/// Average FPS, 1%-low FPS and the worst frame (ms) over the frames kept.
fn summary(times: &VecDeque<f32>) -> Option<(f32, f32, f32)> {
    if times.is_empty() {
        return None;
    }
    let total: f32 = times.iter().sum();
    let average = 1000.0 * times.len() as f32 / total;
    let mut sorted: Vec<f32> = times.iter().copied().collect();
    sorted.sort_by(|a, b| b.total_cmp(a));
    // The slowest 1% of frames, averaged.
    let slowest = (sorted.len() / 100).max(1);
    let low = 1000.0 / (sorted[..slowest].iter().sum::<f32>() / slowest as f32);
    Some((average, low, sorted[0]))
}

pub fn performance_debug_page(ui: &mut egui::Ui, world: &mut World) {
    ui.small("Against the budget: 240 FPS (4.2 ms a frame). The 1% low is what you feel as stutter.");
    let times = world.resource::<FrameTimes>().0.clone();
    if let Some((average, low, worst)) = summary(&times) {
        let colour = |fps: f32| if fps >= 240.0 { egui::Color32::from_rgb(120, 220, 120) } else if fps >= 120.0 { egui::Color32::from_rgb(230, 200, 90) } else { egui::Color32::from_rgb(235, 90, 80) };
        ui.horizontal(|ui| {
            ui.colored_label(colour(average), format!("{average:.0} FPS average"));
            ui.colored_label(colour(low), format!("{low:.0} FPS 1% low"));
            ui.colored_label(colour(1000.0 / worst), format!("worst frame {worst:.1} ms"));
        });
        // The graph: one bar per frame, newest on the right.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(420.0), 90.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(20));
        let top_ms = 25.0;
        let y_of = |ms: f32| rect.bottom() - (ms / top_ms).min(1.0) * rect.height();
        for (ms, label) in [(BUDGET_MS, "240"), (1000.0 / 60.0, "60")] {
            painter.line_segment([egui::pos2(rect.left(), y_of(ms)), egui::pos2(rect.right(), y_of(ms))], egui::Stroke::new(1.0, egui::Color32::from_gray(90)));
            painter.text(egui::pos2(rect.left() + 2.0, y_of(ms) - 1.0), egui::Align2::LEFT_BOTTOM, label, egui::FontId::proportional(9.0), egui::Color32::from_gray(140));
        }
        let shown = times.len().min(rect.width() as usize);
        for (k, &ms) in times.iter().skip(times.len() - shown).enumerate() {
            let x = rect.right() - (shown - k) as f32;
            let colour = if ms <= BUDGET_MS { egui::Color32::from_rgb(90, 180, 90) } else if ms <= 1000.0 / 60.0 { egui::Color32::from_rgb(220, 190, 80) } else { egui::Color32::from_rgb(230, 80, 70) };
            painter.line_segment([egui::pos2(x, rect.bottom()), egui::pos2(x, y_of(ms))], egui::Stroke::new(1.0, colour));
        }
    }

    ui.separator();
    ui.strong("Work in flight");
    let streaming = world.resource::<Streaming>();
    ui.label(format!("Chunks waiting: {}   being made: {}", streaming.queued(), streaming.being_made()));
    ui.label(format!("Chunks in the world: {}", world.resource::<VoxelWorld>().chunk_count()));
    let meshing = world.resource::<MeshingState>();
    let average_job = if meshing.jobs_finished > 0 { meshing.total_job_micros as f64 / meshing.jobs_finished as f64 / 1000.0 } else { 0.0 };
    ui.label(format!("Sections waiting to mesh: {}   meshing: {}   (a mesh takes {average_job:.2} ms on average)", meshing.waiting.len(), meshing.running.len()));

    ui.separator();
    ui.strong("Being drawn");
    let (mut sections, mut quads, mut vertices) = (0u64, 0u64, 0u64);
    let mut query = world.query::<(&SectionMesh, &Visibility)>();
    for (mesh, visibility) in query.iter(world) {
        if *visibility != Visibility::Hidden {
            sections += 1;
            quads += mesh.quads as u64;
            vertices += mesh.vertices as u64;
        }
    }
    ui.label(format!("Sections: {sections}   quads: {quads}   vertices: {vertices}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_one_percent_low_is_the_slowest_frames() {
        // 99 frames at 4 ms and one at 20 ms.
        let mut times: VecDeque<f32> = std::iter::repeat_n(4.0, 99).collect();
        times.push_back(20.0);
        let (average, low, worst) = summary(&times).unwrap();
        assert!((average - 1000.0 / 4.16).abs() < 1.0, "{average}");
        assert!((low - 50.0).abs() < 0.01, "{low}");
        assert_eq!(worst, 20.0);
    }
}

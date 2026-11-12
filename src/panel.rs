//! The developer panel (F3), drawn over the picture: the build, the card, the clocks and the camera.
//!
//! Its name and its pages' names are old's, so they're the ones David knows; new pages are named with him.

use crate::camera::Camera;
use crate::clocks::{PassTimes, Spread};

/// The pages, in order.
const PAGES: [&str; 2] = ["Overview", "Camera"];

pub struct Panel {
    pub visible: bool,
    page: usize,
}

/// What the panel shows this frame.
pub struct Readings<'a> {
    pub card: &'a str,
    pub picture: (u32, u32),
    pub present: wgpu::PresentMode,
    /// The screen's refresh rate, a second.
    pub refresh: f32,
    /// The recent frames.
    pub frame: Spread,
    /// Each pass's recent times, by its id (`wiring.rs`).
    pub passes: &'a [PassTimes],
    /// The card's whole frame, first pass's start to last pass's end (none: the card can't time its own work).
    pub card_whole: Option<Spread>,
    pub camera: &'a Camera,
    pub captured: bool,
}

impl Panel {
    pub fn new() -> Self {
        Self { visible: true, page: 0 }
    }

    pub fn show(&mut self, ctx: &egui::Context, readings: &Readings) {
        if !self.visible {
            return;
        }
        egui::Window::new("Debug  [F3]").default_pos([10.0, 10.0]).default_width(320.0).show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (index, name) in PAGES.iter().enumerate() {
                    if ui.selectable_label(self.page == index, *name).clicked() {
                        self.page = index;
                    }
                }
            });
            ui.separator();
            match self.page {
                0 => overview(ui, readings),
                _ => camera(ui, readings),
            }
        });
    }
}

fn overview(ui: &mut egui::Ui, readings: &Readings) {
    ui.label(format!("Incus Artus, {}", crate::session::build_line()));
    if !crate::session::BUILD_CHANGED.is_empty() {
        ui.label(format!("Changed from the commit: {}", crate::session::BUILD_CHANGED));
    }
    ui.label(format!("The card: {}", readings.card));
    ui.label(format!("The picture: {} x {}, {:?}; the screen {:.0} a second", readings.picture.0, readings.picture.1, readings.present, readings.refresh));
    ui.separator();
    let frame = readings.frame;
    let fps = if frame.median > 0.0 { 1000.0 / frame.median } else { 0.0 };
    ui.strong(format!("The frame: {fps:.0} a second"));
    ui.label(format!("ms: median {:.2}, 95th {:.2}, 99th {:.2} (the last {} frames)", frame.median, frame.p95, frame.p99, frame.count));
    ui.separator();
    ui.strong("Each pass (ms, median and 99th)");
    egui::Grid::new("the passes").striped(true).show(ui, |ui| {
        ui.label("");
        ui.label("on the processor");
        ui.label("on the card");
        ui.end_row();
        let both = |spread: &Spread| if spread.count == 0 { "-".to_string() } else { format!("{:.3}  {:.3}", spread.median, spread.p99) };
        for pass in &crate::wiring::PASSES {
            let times = &readings.passes[pass.id as usize];
            ui.label(pass.name);
            ui.label(both(&times.processor));
            ui.label(times.card.as_ref().map_or(String::new(), both));
            ui.end_row();
        }
        if let Some(whole) = &readings.card_whole {
            ui.label("the card's whole");
            ui.label("");
            ui.label(both(whole));
            ui.end_row();
        }
    });
    if readings.card_whole.is_none() {
        ui.label("This card can't time its own work.");
    }
}

fn camera(ui: &mut egui::Ui, readings: &Readings) {
    let camera = readings.camera;
    ui.label(format!("north {:.1}, east {:.1}, up {:.1}", camera.position.x, camera.position.y, camera.position.z));
    ui.label(format!("facing {:.0}°, pitch {:.1}°", camera.facing, camera.pitch));
    ui.label(if readings.captured { "Mouse: captured (Tab to release)" } else { "Mouse: free (Tab to fly)" });
}

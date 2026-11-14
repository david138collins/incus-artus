//! The developer panel (F3), drawn over the picture: the build, the card, the clocks and the camera.
//!
//! Its name and its pages' names are old's, so they're the ones David knows; new pages are named with him.

use crate::camera::{Camera, Pace};
use crate::clocks::{PassTimes, Spread};
use crate::far_eye::FarLook;
use crate::wiring::PassId;

/// The pages, in order (old's names; the Render page holds the far eye's look, David 2026-11-12).
const PAGES: [&str; 3] = ["Overview", "Camera", "Render"];

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
    pub camera: Camera,
    pub captured: bool,
    /// The card's memory the game holds, MB: in use, and held (none where the card can't say).
    pub memory: Option<(f64, f64)>,
    /// Which far blends can be chosen now (the code's, the hardware's), and how the far cells are coming in.
    pub blends: [bool; 2],
    pub far: String,
}

impl Panel {
    pub fn new() -> Self {
        Self { visible: true, page: 0 }
    }

    /// The panel, its Camera page changing the camera's speed and its Render page the far eye's look (`look`: none
    /// without a far eye).
    pub fn show(&mut self, ctx: &egui::Context, readings: &Readings, look: Option<&mut FarLook>, pace: &mut Pace) {
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
                1 => camera(ui, readings, pace),
                _ => render(ui, readings, look),
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
    if let Some((used, held)) = readings.memory {
        ui.label(format!("The card's memory: {used:.0} MB in use, {held:.0} MB held"));
    }
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

/// The far eye's passes on the card, in the frame's order.
const FAR_PASSES: [PassId; 5] = [PassId::TheFarStart, PassId::TheFar8, PassId::TheFar4, PassId::TheFar2, PassId::BringingItUp];

/// A section's Default: greyed while its settings are the defaults; pressed, they go back.
fn section_default(ui: &mut egui::Ui, look: &mut FarLook, differs: bool, reset: impl FnOnce(&mut FarLook)) {
    if ui.add_enabled(differs, egui::Button::new("Default").small()).clicked() {
        reset(look);
    }
}

/// The Render page: the far eye's card time at the top (its gauge, to watch as the settings move), then its look's
/// settings in sections, each with its own Default, and all of them back at once at the bottom [David, 2026-11-12: "a
/// very simple crossover slider set for the transitions ... i can watch the card time as i adjust it ... and include a
/// default button"; "please expose any interesting or valuable parameters that may be worth adjusting live, with
/// default switches so i can get back to normal"].
fn render(ui: &mut egui::Ui, readings: &Readings, look: Option<&mut FarLook>) {
    let card = |id: PassId| readings.passes[id as usize].card.unwrap_or_default();
    let median: f32 = FAR_PASSES.iter().map(|id| card(*id).median).sum();
    let p99: f32 = FAR_PASSES.iter().map(|id| card(*id).p99).sum();
    let frame = readings.frame;
    let fps = if frame.median > 0.0 { 1000.0 / frame.median } else { 0.0 };
    ui.strong(format!("The far eye on the card: {median:.2} ms (99th {p99:.2})"));
    if let Some((used, held)) = readings.memory {
        ui.label(format!("The card's memory: {used:.0} MB in use, {held:.0} MB held (the bar: well under 4,000 at worst)"));
    }
    ui.label(format!("The far cells: {}", readings.far));
    ui.label(format!("The frame: {:.2} ms, {fps:.0} a second", frame.median));
    ui.label(FAR_PASSES.iter().map(|id| format!("{} {:.2}", crate::wiring::pass(*id).name.trim_start_matches("the far "), card(*id).median)).collect::<Vec<_>>().join(" · "));
    ui.separator();
    let Some(look) = look else {
        ui.label("No far eye (the doorway couldn't open the world).");
        return;
    };
    let d = FarLook::DEFAULT;
    egui::ScrollArea::vertical().max_height(560.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong("The picture");
            let differs = (look.blend, look.detail, look.bring_up, look.view, look.sky_weight) != (d.blend, d.detail, d.bring_up, d.view, d.sky_weight);
            section_default(ui, look, differs, |l| (l.blend, l.detail, l.bring_up, l.view, l.sky_weight) = (d.blend, d.detail, d.bring_up, d.view, d.sky_weight));
        });
        // Change 1: the far cells blended by the code (old's) or the texture hardware (greyed till its textures are in).
        ui.horizontal(|ui| {
            ui.label("far blend");
            for (k, name) in ["old's (code)", "the hardware's"].iter().enumerate() {
                ui.add_enabled_ui(readings.blends[k], |ui| {
                    ui.selectable_value(&mut look.blend, k as u32, *name);
                });
            }
        });
        ui.add(egui::Slider::new(&mut look.detail, -3..=4).text("detail (levels finer)"));
        ui.horizontal(|ui| {
            ui.label("brought up");
            ui.selectable_value(&mut look.bring_up, 0, "crisp");
            ui.selectable_value(&mut look.bring_up, 1, "smooth");
            ui.selectable_value(&mut look.bring_up, 2, "sharp skylines");
        });
        ui.horizontal(|ui| {
            ui.label("show");
            ui.selectable_value(&mut look.view, 0, "the picture");
            ui.selectable_value(&mut look.view, 1, "steps");
            ui.selectable_value(&mut look.view, 2, "levels");
        });
        ui.add(egui::Slider::new(&mut look.sky_weight, 0.0..=8.0).text("sky weight"));
        ui.separator();
        ui.horizontal(|ui| {
            ui.strong("Crossovers");
            let differs = (look.hand_over, look.fade_4, look.fade_2) != (d.hand_over, d.fade_4, d.fade_2);
            section_default(ui, look, differs, |l| (l.hand_over, l.fade_4, l.fade_2) = (d.hand_over, d.fade_4, d.fade_2));
        });
        ui.add(egui::Slider::new(&mut look.hand_over, 0.3..=0.95).text("levels hand over from"));
        ui.add(egui::Slider::new(&mut look.fade_4[0], 0.1..=4.0).text("4 × 4 fades in from"));
        ui.add(egui::Slider::new(&mut look.fade_4[1], 0.05..=3.0).text("4 × 4 fades in over"));
        ui.add(egui::Slider::new(&mut look.fade_2[0], 0.1..=4.0).text("2 × 2 fades in from"));
        ui.add(egui::Slider::new(&mut look.fade_2[1], 0.05..=3.0).text("2 × 2 fades in over"));
        ui.separator();
        ui.horizontal(|ui| {
            ui.strong("Light");
            let differs = (look.sun_from, look.sun_height, look.ambient, look.shadows, look.sky_light, look.sky_strength, look.sky_tint, look.light_rays) != (d.sun_from, d.sun_height, d.ambient, d.shadows, d.sky_light, d.sky_strength, d.sky_tint, d.light_rays);
            section_default(ui, look, differs, |l| {
                (l.sun_from, l.sun_height, l.ambient, l.shadows, l.sky_light, l.sky_strength, l.sky_tint, l.light_rays) = (d.sun_from, d.sun_height, d.ambient, d.shadows, d.sky_light, d.sky_strength, d.sky_tint, d.light_rays);
            });
        });
        ui.add(egui::Slider::new(&mut look.sun_from, 0.0..=360.0).text("sun from (degrees, north to east)"));
        ui.add(egui::Slider::new(&mut look.sun_height, -10.0..=90.0).text("sun height (degrees)"));
        ui.add(egui::Slider::new(&mut look.ambient, 0.0..=1.0).text("sky's share of the light"));
        ui.checkbox(&mut look.shadows, "sun shadows");
        ui.checkbox(&mut look.sky_light, "sky light");
        // David's switch for looking at the rays against the light read from the cell (build 161; placed and named
        // with him). It goes when the rays' light is cut from the march.
        ui.checkbox(&mut look.light_rays, "light by rays");
        ui.add_enabled(look.sky_light, egui::Slider::new(&mut look.sky_strength, 0.0..=1.0).text("sky light strength"));
        ui.add(egui::Slider::new(&mut look.sky_tint, 0.0..=1.0).text("sky tint"));
        ui.separator();
        ui.horizontal(|ui| {
            ui.strong("Fog");
            let differs = (look.fog, look.fog_reach, look.fog_height, look.fog_warmth) != (d.fog, d.fog_reach, d.fog_height, d.fog_warmth);
            section_default(ui, look, differs, |l| (l.fog, l.fog_reach, l.fog_height, l.fog_warmth) = (d.fog, d.fog_reach, d.fog_height, d.fog_warmth));
        });
        ui.checkbox(&mut look.fog, "fog");
        ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_reach, 500.0..=20_000.0).logarithmic(true).text("fog reach (blocks)"));
        ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_height, 25.0..=1_000.0).text("fog thins every (blocks up)"));
        ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_warmth, 0.0..=1.0).text("fog warmth towards the sun"));
    });
    ui.separator();
    if ui.add_enabled(*look != FarLook::DEFAULT, egui::Button::new("All to default")).clicked() {
        *look = FarLook::DEFAULT;
    }
}

/// The Camera page: where the camera is, and how fast it flies (its speed and Shift's multiplier, with a Default).
fn camera(ui: &mut egui::Ui, readings: &Readings, pace: &mut Pace) {
    let camera = &readings.camera;
    ui.label(format!("north {:.1}, east {:.1}, up {:.1}", camera.position.x, camera.position.y, camera.position.z));
    ui.label(format!("facing {:.0}°, pitch {:.1}°", camera.facing, camera.pitch));
    ui.label(if readings.captured { "Mouse: captured (Tab to release)" } else { "Mouse: free (Tab to fly)" });
    ui.separator();
    ui.add(egui::Slider::new(&mut pace.speed, 1.0..=2000.0).logarithmic(true).text("speed (blocks a second)"));
    ui.add(egui::Slider::new(&mut pace.multiplier, 1.0..=20.0).text("multiplier (Shift held)"));
    if ui.add_enabled(*pace != Pace::DEFAULT, egui::Button::new("Default").small()).clicked() {
        *pace = Pace::DEFAULT;
    }
}

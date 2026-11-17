//! The developer panel (F3), drawn over the picture: the build, the card, the clocks and the camera.
//!
//! Its name and its pages' names are old's, so they're the ones David knows; new pages are named with him.

use crate::camera::{Camera, Pace};
use crate::clocks::{PassTimes, Spread};
use crate::far_eye::FarLook;
use crate::near_eye::NearLook;
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
    /// Whether the cover-by-faces switch can do anything now (`FarEye::faces_switch`).
    pub faces_switch: bool,
    /// The near eye's counts (none: no near eye).
    pub near: Option<crate::near_eye::Counts>,
    pub far: String,
}

impl Panel {
    pub fn new() -> Self {
        Self { visible: true, page: 0 }
    }

    /// The panel, its Camera page changing the camera's speed and its Render page the far eye's look (`look`: none
    /// without a far eye).
    pub fn show(&mut self, ctx: &egui::Context, readings: &Readings, look: Option<&mut FarLook>, near_look: Option<&mut NearLook>, pace: &mut Pace, fov: &mut f32) {
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
            // The page scrolls under its tabs, and is never taller than the room from here to the screen's bottom
            // [David, 2026-11-16: "we gotta do something about that debug panel, it runs off the screen its so long.
            // we should add collapsing sections to it and a scrollbar"].
            let room = (ui.ctx().content_rect().bottom() - ui.cursor().top() - 24.0).max(120.0);
            egui::ScrollArea::vertical().max_height(room).show(ui, |ui| match self.page {
                0 => overview(ui, readings),
                1 => camera(ui, readings, pace, fov),
                _ => render(ui, readings, look, near_look),
            });
        });
    }
}

/// A section of a page that folds [David, 2026-11-16: "we should add collapsing sections to it and a scrollbar"; of the
/// sections and which start open: "your way of folding the content of the debug pages is fine"]. Its heading: the
/// arrow, its title (a click on either folds or opens it), and whatever `beside` puts there (a section's Default);
/// `body` under it while it's open. Folded or open is kept while the game runs, by `name`. `on`: what both work on.
fn section<T: ?Sized>(ui: &mut egui::Ui, name: &str, title: &str, open: bool, on: &mut T, beside: impl FnOnce(&mut egui::Ui, &mut T), body: impl FnOnce(&mut egui::Ui, &mut T)) {
    let id = ui.make_persistent_id(name);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open);
    let head = ui.horizontal(|ui| {
        state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
        let clicked = ui.add(egui::Label::new(egui::RichText::new(title).strong()).selectable(false).sense(egui::Sense::click())).clicked();
        beside(ui, &mut *on);
        clicked
    });
    if head.inner {
        state.toggle(ui);
    }
    state.show_body_unindented(ui, |ui| body(ui, on));
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
    section(ui, "overview: the frame", &format!("The frame: {fps:.0} a second"), false, &mut (), |_, _| {}, |ui, _| {
        ui.label(format!("ms: median {:.2}, 95th {:.2}, 99th {:.2} (the last {} frames)", frame.median, frame.p95, frame.p99, frame.count));
    });
    ui.separator();
    section(ui, "overview: each pass", "Each pass (ms, median and 99th)", false, &mut (), |_, _| {}, |ui, _| {
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
    });
}

/// The far eye's passes on the card, in the frame's order.
const FAR_PASSES: [PassId; 5] = [PassId::TheFarStart, PassId::TheFar8, PassId::TheFar4, PassId::TheFar2, PassId::BringingItUp];

/// A section's Default: greyed while its settings are the defaults; pressed, they go back.
fn section_default(ui: &mut egui::Ui, look: &mut FarLook, differs: bool, reset: impl FnOnce(&mut FarLook)) {
    if ui.add_enabled(differs, egui::Button::new("Default").small()).clicked() {
        reset(look);
    }
}

/// The Render page: the far eye's card time at the top (its gauge, to watch as the settings move), the near eye's and
/// its radius, then the far eye's look's settings in sections, each with its own Default, and all of them back at once
/// at the bottom [David, 2026-11-12: "a very simple crossover slider set for the transitions ... i can watch the card
/// time as i adjust it ... and include a default button"; "please expose any interesting or valuable parameters that
/// may be worth adjusting live, with default switches so i can get back to normal"]. Every section folds (`section`);
/// the far eye, the near eye and the picture are open at the start.
fn render(ui: &mut egui::Ui, readings: &Readings, look: Option<&mut FarLook>, near_look: Option<&mut NearLook>) {
    let card = |id: PassId| readings.passes[id as usize].card.unwrap_or_default();
    let median: f32 = FAR_PASSES.iter().map(|id| card(*id).median).sum();
    let p99: f32 = FAR_PASSES.iter().map(|id| card(*id).p99).sum();
    let frame = readings.frame;
    let fps = if frame.median > 0.0 { 1000.0 / frame.median } else { 0.0 };
    section(ui, "render: the far eye", &format!("The far eye on the card: {median:.2} ms (99th {p99:.2})"), true, &mut (), |_, _| {}, |ui, _| {
        if let Some((used, held)) = readings.memory {
            ui.label(format!("The card's memory: {used:.0} MB in use, {held:.0} MB held (the bar: well under 4,000 at worst)"));
        }
        ui.label(format!("The far cells: {}", readings.far));
        ui.label(format!("The frame: {:.2} ms, {fps:.0} a second", frame.median));
        ui.label(FAR_PASSES.iter().map(|id| format!("{} {:.2}", crate::wiring::pass(*id).name.trim_start_matches("the far "), card(*id).median)).collect::<Vec<_>>().join(" · "));
    });
    // The near eye: its time on the card, what it holds and draws, and its radius [David, 2026-11-16: "lets just make
    // a radius slider and get started. the render radius set there will determine where the triangles start and rays
    // fade"; its place and name here: "yes thats fine"]. And the x-fader, how blocks hand over to rays [David,
    // 2026-11-16: "i think a x-fader with curves could hide it almost completely so blocks smoothly fade into rays";
    // its two sliders' place and names were put to him with the step, and his answer was "go ahead with the next
    // step"]. Greyed where there's no near eye. The section's Default puts all three back, like the far eye's.
    let near_ms: f32 = [PassId::TheNearFaces, PassId::TheNearWater, PassId::TheNearCover, PassId::TheXFader].iter().map(|id| card(*id).median).sum();
    let near_title = if readings.near.is_some() { format!("The near eye on the card: {near_ms:.2} ms") } else { "The near eye".to_string() };
    let mut none = NearLook::DEFAULT;
    let there = near_look.is_some();
    let beside = |ui: &mut egui::Ui, look: &mut NearLook| {
        if ui.add_enabled(there && *look != NearLook::DEFAULT, egui::Button::new("Default").small()).clicked() {
            *look = NearLook::DEFAULT;
        }
    };
    section(ui, "render: the near eye", &near_title, true, near_look.unwrap_or(&mut none), beside, |ui, look| {
        match &readings.near {
            Some(near) => {
                ui.label(format!(
                    "{} of {} chunks in, {} drawn; {:.2} million triangles drawn of {:.2} kept, {:.0} MB",
                    near.kept,
                    near.wanted,
                    near.drawn,
                    near.faces_drawn as f64 * 2.0 / 1e6,
                    near.faces as f64 * 2.0 / 1e6,
                    near.megabytes
                ));
                ui.label(format!("its faces reach {:.0} blocks; the far rays begin at {:.0}; a chunk meshed in {:.1} ms on {} threads", near.reach, near.far_from, near.chunk_ms, near.threads));
            }
            None => {
                ui.label("No near eye (INCUS_NEAR=0, or no far eye).");
            }
        }
        ui.add_enabled(there, egui::Slider::new(&mut look.radius, 1.0..=crate::near_eye::MOST_RADIUS).step_by(1.0).text("Render radius (chunks)"));
        ui.add_enabled(there, egui::Slider::new(&mut look.fader_width, 0.0..=crate::near_eye::MOST_FADER_WIDTH).step_by(1.0).text("x-fader width (blocks)"));
        ui.add_enabled(there, egui::Slider::new(&mut look.fader_curve, 1.0..=crate::near_eye::MOST_FADER_CURVE).text("x-fader curve"));
        // The detail curve's near half [David, 2026-11-17; the places and names his choices]: the largest squares the
        // near eye draws in, 1 (the curve off), 2 or 4 screen pixels a side (the slider moves the power of two); and
        // the curve's near number, the size in the world of the smallest thing drawn up close.
        let mut power = look.pixel.max(1).ilog2().min(2);
        ui.add_enabled(
            there,
            egui::Slider::new(&mut power, 0..=2)
                .custom_formatter(|v, _| format!("{}", 1u32 << (v.round().clamp(0.0, 2.0) as u32)))
                .custom_parser(|text| text.trim().parse::<f64>().ok().map(|size| size.max(1.0).log2().round()))
                .text("Largest near squares"),
        );
        look.pixel = 1 << power;
        let details = crate::near_eye::DETAILS;
        ui.add_enabled(there && look.pixel > 1, egui::Slider::new(&mut look.detail, details.0..=details.1).logarithmic(true).text("Detail up close (blocks)"));
        ui.add_enabled(there && look.pixel > 1, egui::Slider::new(&mut look.stipple, 0.0..=crate::near_eye::MOST_STIPPLE).text("Stipple width (share of the band)"));
        ui.add_enabled(there && look.pixel > 1, egui::Slider::new(&mut look.sharp, 0.0..=1.0).text("Picture sharpness in the squares"));
    });
    ui.separator();
    let Some(look) = look else {
        ui.label("No far eye (the doorway couldn't open the world).");
        return;
    };
    let d = FarLook::DEFAULT;
    section(
        ui,
        "render: the picture",
        "The picture",
        true,
        &mut *look,
        |ui, look| {
            let differs = (look.blend, look.detail, look.bring_up, look.view, look.sky_weight, look.faces) != (d.blend, d.detail, d.bring_up, d.view, d.sky_weight, d.faces);
            section_default(ui, look, differs, |l| (l.blend, l.detail, l.bring_up, l.view, l.sky_weight, l.faces) = (d.blend, d.detail, d.bring_up, d.view, d.sky_weight, d.faces));
        },
        |ui, look| {
            // Change 1: the far cells blended by the code (old's: the "initial blend", David's name for it, 2026-11-14,
            // as it's what draws till the textures are in and wherever a brick isn't in them) or the texture hardware
            // (greyed till its textures are in).
            ui.horizontal(|ui| {
                ui.label("far blend");
                for (k, name) in ["initial blend", "the hardware's"].iter().enumerate() {
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
            // Cover by faces [David, 2026-11-16: "do i have an a/b switch?"; its place and name: "yes, that place is
            // fine for the switch"]: greyed till the world is loaded (the march's other program takes over only
            // then), and where there's no other program.
            ui.add_enabled(readings.faces_switch, egui::Checkbox::new(&mut look.faces, "Cover by faces"));
        },
    );
    ui.separator();
    // Where each ray's steps begin within a step [David, 2026-11-14: "you could put them under a ray start section or
    // something"; "if they have parameters worth exposing i'd like to play with those"].
    section(
        ui,
        "render: ray start",
        "Ray start",
        false,
        &mut *look,
        |ui, look| {
            let differs = (look.start_pattern, look.weave_bits, look.start_blue, look.start_spread) != (d.start_pattern, d.weave_bits, d.start_blue, d.start_spread);
            section_default(ui, look, differs, |l| (l.start_pattern, l.weave_bits, l.start_blue, l.start_spread) = (d.start_pattern, d.weave_bits, d.start_blue, d.start_spread));
        },
        |ui, look| {
            ui.horizontal(|ui| {
                ui.label("pattern");
                for (k, name) in ["scattered", "woven", "blue noise", "woven + blue noise"].iter().enumerate() {
                    ui.selectable_value(&mut look.start_pattern, k as u32, *name);
                }
            });
            let woven = look.start_pattern == 1 || look.start_pattern == 3;
            ui.add_enabled_ui(woven, |ui| {
                ui.horizontal(|ui| {
                    ui.label("weave");
                    for bits in 1..=4u32 {
                        ui.selectable_value(&mut look.weave_bits, bits, format!("{0} × {0}", 1 << bits));
                    }
                });
            });
            ui.add_enabled(look.start_pattern == 3, egui::Slider::new(&mut look.start_blue, 0.0..=1.0).text("blue noise in the weave"));
            ui.add(egui::Slider::new(&mut look.start_spread, 0.0..=1.0).text("spread (of a step)"));
        },
    );
    ui.separator();
    section(
        ui,
        "render: crossovers",
        "Crossovers",
        false,
        &mut *look,
        |ui, look| {
            let differs = (look.hand_over, look.fade_4, look.fade_2) != (d.hand_over, d.fade_4, d.fade_2);
            section_default(ui, look, differs, |l| (l.hand_over, l.fade_4, l.fade_2) = (d.hand_over, d.fade_4, d.fade_2));
        },
        |ui, look| {
            ui.add(egui::Slider::new(&mut look.hand_over, 0.3..=0.95).text("levels hand over from"));
            ui.add(egui::Slider::new(&mut look.fade_4[0], 0.1..=4.0).text("4 × 4 fades in from"));
            ui.add(egui::Slider::new(&mut look.fade_4[1], 0.05..=3.0).text("4 × 4 fades in over"));
            ui.add(egui::Slider::new(&mut look.fade_2[0], 0.1..=4.0).text("2 × 2 fades in from"));
            ui.add(egui::Slider::new(&mut look.fade_2[1], 0.05..=3.0).text("2 × 2 fades in over"));
        },
    );
    ui.separator();
    section(
        ui,
        "render: light",
        "Light",
        false,
        &mut *look,
        |ui, look| {
            let differs = (look.sun_from, look.sun_height, look.ambient, look.shadows, look.sky_light, look.sky_strength, look.sky_tint) != (d.sun_from, d.sun_height, d.ambient, d.shadows, d.sky_light, d.sky_strength, d.sky_tint);
            section_default(ui, look, differs, |l| {
                (l.sun_from, l.sun_height, l.ambient, l.shadows, l.sky_light, l.sky_strength, l.sky_tint) = (d.sun_from, d.sun_height, d.ambient, d.shadows, d.sky_light, d.sky_strength, d.sky_tint);
            });
        },
        |ui, look| {
            ui.add(egui::Slider::new(&mut look.sun_from, 0.0..=360.0).text("sun from (degrees, north to east)"));
            ui.add(egui::Slider::new(&mut look.sun_height, -10.0..=90.0).text("sun height (degrees)"));
            ui.add(egui::Slider::new(&mut look.ambient, 0.0..=1.0).text("sky's share of the light"));
            ui.checkbox(&mut look.shadows, "sun shadows");
            ui.checkbox(&mut look.sky_light, "sky light");
            ui.add_enabled(look.sky_light, egui::Slider::new(&mut look.sky_strength, 0.0..=1.0).text("sky light strength"));
            ui.add(egui::Slider::new(&mut look.sky_tint, 0.0..=1.0).text("sky tint"));
        },
    );
    ui.separator();
    section(
        ui,
        "render: fog",
        "Fog",
        false,
        &mut *look,
        |ui, look| {
            let differs = (look.fog, look.fog_reach, look.fog_height, look.fog_warmth) != (d.fog, d.fog_reach, d.fog_height, d.fog_warmth);
            section_default(ui, look, differs, |l| (l.fog, l.fog_reach, l.fog_height, l.fog_warmth) = (d.fog, d.fog_reach, d.fog_height, d.fog_warmth));
        },
        |ui, look| {
            ui.checkbox(&mut look.fog, "fog");
            ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_reach, 500.0..=20_000.0).logarithmic(true).text("fog reach (blocks)"));
            ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_height, 25.0..=1_000.0).text("fog thins every (blocks up)"));
            ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_warmth, 0.0..=1.0).text("fog warmth towards the sun"));
        },
    );
    ui.separator();
    if ui.add_enabled(*look != FarLook::DEFAULT, egui::Button::new("All to default")).clicked() {
        *look = FarLook::DEFAULT;
    }
}

/// The Camera page: where the camera is, how fast it flies (its speed and Shift's multiplier) and its field of view
/// [David, 2026-11-17: "i think we need to add FOV and Panini under camera again. it helps for me to be able to set
/// low FOV and zoom into seams to inspect"; "fov for now"], with a Default for all three.
fn camera(ui: &mut egui::Ui, readings: &Readings, pace: &mut Pace, fov: &mut f32) {
    let camera = &readings.camera;
    ui.label(format!("north {:.1}, east {:.1}, up {:.1}", camera.position.x, camera.position.y, camera.position.z));
    ui.label(format!("facing {:.0}°, pitch {:.1}°", camera.facing, camera.pitch));
    ui.label(if readings.captured { "Mouse: captured (Tab to release)" } else { "Mouse: free (Tab to fly)" });
    ui.separator();
    ui.add(egui::Slider::new(&mut pace.speed, 1.0..=2000.0).logarithmic(true).text("speed (blocks a second)"));
    ui.add(egui::Slider::new(&mut pace.multiplier, 1.0..=20.0).text("multiplier (Shift held)"));
    let fields = crate::camera::FIELDS_OF_VIEW;
    ui.add(egui::Slider::new(fov, fields.0..=fields.1).step_by(1.0).text("Field of view (degrees, up and down)"));
    if ui.add_enabled(*pace != Pace::DEFAULT || *fov != crate::camera::FIELD_OF_VIEW, egui::Button::new("Default").small()).clicked() {
        *pace = Pace::DEFAULT;
        *fov = crate::camera::FIELD_OF_VIEW;
    }
}

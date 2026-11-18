//! The developer panel (F3), drawn over the picture: the build, the card, the clocks and the camera.
//!
//! Its name and its pages' names are old's, so they're the ones David knows; new pages are named with him.

use crate::camera::{Camera, Pace};
use crate::clocks::{PassTimes, Spread};
use crate::far_eye::FarLook;
use crate::near_eye::NearLook;
use crate::presets::{Bar, CameraPreset, RenderPreset};
use crate::wiring::PassId;

/// The pages, in order (old's names; the Render page holds the far eye's look, David 2026-11-12).
const PAGES: [&str; 3] = ["Overview", "Camera", "Render"];

pub struct Panel {
    pub visible: bool,
    page: usize,
    /// The Camera page's and the Render page's preset bars (`presets.rs`) [David, 2026-11-18, their place: "On each
    /// page"].
    camera: Bar<CameraPreset>,
    render: Bar<RenderPreset>,
    /// The near eye's settings where there's no near eye: what the game started with or a preset last loaded, so a
    /// preset saved without one keeps them.
    pub near_spare: NearLook,
    /// What Ctrl+S last came to, and when (egui's clock): shown at the bottom of the screen for three seconds.
    said: Option<(String, f64)>,
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
        Self::with_presets_in(&crate::presets::folder())
    }

    /// The panel, its pages' presets kept in `root`.
    pub fn with_presets_in(root: &std::path::Path) -> Self {
        Self { visible: true, page: 0, camera: Bar::new(root, "camera"), render: Bar::new(root, "render"), near_spare: NearLook::DEFAULT, said: None }
    }

    /// The panel, its Camera page changing the camera's speed and its Render page the near and far eyes' looks (`look`:
    /// none without a far eye). Each of those two pages has its preset bar under the tabs, in view however far the page
    /// is scrolled; Ctrl+S saves the showing page's settings under the name in its box, while the mouse is free.
    pub fn show(&mut self, ctx: &egui::Context, readings: &Readings, mut look: Option<&mut FarLook>, mut near_look: Option<&mut NearLook>, pace: &mut Pace, fov: &mut f32) {
        if !self.visible {
            return;
        }
        let camera_now = |pace: &Pace, fov: f32| CameraPreset { speed: pace.speed, multiplier: pace.multiplier, field_of_view: fov };
        let render_now = |near: Option<&NearLook>, far: Option<&FarLook>, spare: &NearLook| RenderPreset::of(near.unwrap_or(spare), far.unwrap_or(&FarLook::DEFAULT));
        let now = ctx.input(|input| input.time);
        if !readings.captured && ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            let said = match self.page {
                1 => self.camera.save_named(&camera_now(pace, *fov)),
                2 if look.is_some() => self.render.save_named(&render_now(near_look.as_deref(), look.as_deref(), &self.near_spare)),
                2 => "Ctrl+S: there's no far eye, so nothing to save.".to_string(),
                _ => "Ctrl+S saves the Camera page's or the Render page's settings: open one of them.".to_string(),
            };
            self.said = Some((said, now));
        }
        if let Some((said, when)) = &self.said {
            if now - when < 3.0 {
                egui::Area::new("preset saved".into()).anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -40.0)).order(egui::Order::Foreground).show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(said));
                });
            } else {
                self.said = None;
            }
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
            // The page's preset bar: what's loaded goes straight into the settings (the lantern left lit or out as it
            // is: that's the L key's).
            match self.page {
                1 => {
                    let mut preset = camera_now(pace, *fov);
                    if self.camera.show(ui, &mut preset, true) {
                        (pace.speed, pace.multiplier, *fov) = (preset.speed, preset.multiplier, preset.field_of_view);
                    }
                    ui.separator();
                }
                2 => {
                    let mut preset = render_now(near_look.as_deref(), look.as_deref(), &self.near_spare);
                    if self.render.show(ui, &mut preset, look.is_some()) {
                        if let Some(far) = look.as_deref_mut() {
                            *far = preset.far;
                        }
                        match near_look.as_deref_mut() {
                            Some(near) => *near = NearLook { lantern: near.lantern, ..preset.near },
                            None => self.near_spare = preset.near,
                        }
                    }
                    ui.separator();
                }
                _ => {}
            }
            let spare = self.near_spare;
            // The page scrolls under its tabs, and is never taller than the room from here to the screen's bottom
            // [David, 2026-11-16: "we gotta do something about that debug panel, it runs off the screen its so long.
            // we should add collapsing sections to it and a scrollbar"].
            let room = (ui.ctx().content_rect().bottom() - ui.cursor().top() - 24.0).max(120.0);
            egui::ScrollArea::vertical().max_height(room).show(ui, |ui| match self.page {
                0 => overview(ui, readings),
                1 => camera(ui, readings, pace, fov),
                _ => render(ui, readings, look, near_look, spare),
            });
        });
    }
}

/// A section of a page that folds [David, 2026-11-16: "we should add collapsing sections to it and a scrollbar"; of the
/// sections and which start open: "your way of folding the content of the debug pages is fine"]. Its heading: the
/// arrow and its title (a click on either folds or opens it); `body` under it while it's open. Folded or open is kept
/// while the game runs, by `name`. `on`: what the body works on. (A section's Default sat beside its title till build
/// 219: the page's presets took their place [David, 2026-11-18: "this can replace the defaulter buttons inside the
/// panel"].)
fn section<T: ?Sized>(ui: &mut egui::Ui, name: &str, title: &str, open: bool, on: &mut T, body: impl FnOnce(&mut egui::Ui, &mut T)) {
    let id = ui.make_persistent_id(name);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open);
    let head = ui.horizontal(|ui| {
        state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
        ui.add(egui::Label::new(egui::RichText::new(title).strong()).selectable(false).sense(egui::Sense::click())).clicked()
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
    section(ui, "overview: the frame", &format!("The frame: {fps:.0} a second"), false, &mut (), |ui, _| {
        ui.label(format!("ms: median {:.2}, 95th {:.2}, 99th {:.2} (the last {} frames)", frame.median, frame.p95, frame.p99, frame.count));
    });
    ui.separator();
    section(ui, "overview: each pass", "Each pass (ms, median and 99th)", false, &mut (), |ui, _| {
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

/// The Render page: the far eye's card time at the top (its gauge, to watch as the settings move), the near eye's and
/// its radius, then the far eye's look's settings in sections [David, 2026-11-12: "a very simple crossover slider set
/// for the transitions ... i can watch the card time as i adjust it ... and include a default button"; "please expose
/// any interesting or valuable parameters that may be worth adjusting live, with default switches so i can get back to
/// normal"; the way back is the page's "default" preset since build 219]. Every section folds (`section`); the far
/// eye, the near eye and the picture are open at the start. `spare`: the near eye's settings to show, greyed, where
/// there's no near eye.
fn render(ui: &mut egui::Ui, readings: &Readings, look: Option<&mut FarLook>, near_look: Option<&mut NearLook>, spare: NearLook) {
    let card = |id: PassId| readings.passes[id as usize].card.unwrap_or_default();
    let median: f32 = FAR_PASSES.iter().map(|id| card(*id).median).sum();
    let p99: f32 = FAR_PASSES.iter().map(|id| card(*id).p99).sum();
    let frame = readings.frame;
    let fps = if frame.median > 0.0 { 1000.0 / frame.median } else { 0.0 };
    section(ui, "render: the far eye", &format!("The far eye on the card: {median:.2} ms (99th {p99:.2})"), true, &mut (), |ui, _| {
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
    // step"]. Greyed where there's no near eye.
    let near_ms: f32 = [PassId::TheNearFaces, PassId::TheNearWater, PassId::TheNearCover, PassId::TheXFader, PassId::SeeingTheFarLamps, PassId::TheFarLamps].iter().map(|id| card(*id).median).sum();
    let near_title = if readings.near.is_some() { format!("The near eye on the card: {near_ms:.2} ms") } else { "The near eye".to_string() };
    let mut none = spare;
    let there = near_look.is_some();
    let near_look = near_look.unwrap_or(&mut none);
    section(ui, "render: the near eye", &near_title, true, &mut *near_look, |ui, look| {
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
    // Lights (build 216's "Placed lights", a section of their own by his word; renamed and the lantern put in it at
    // build 217) [David, 2026-11-17, asked where the lantern's controls go: "In \"Placed lights\", renamed
    // \"Lights\""]: the flicker, a wobble where the light is read; and the lantern, a light at the eye (the L
    // key). (A preset leaves the lantern lit or out as it is.)
    section(ui, "render: lights", "Lights", true, &mut *near_look, |ui, look| {
        let steps = crate::near_eye::STEPS;
        ui.add_enabled(there, egui::Slider::new(&mut look.step, steps.0..=steps.1).text("Light step (share dimmer each block)"));
        ui.add_enabled(there, egui::Slider::new(&mut look.flicker, 0.0..=1.0).text("Flicker (share of brightness)"));
        ui.add_enabled(there, egui::Slider::new(&mut look.flicker_speed, 0.25..=8.0).logarithmic(true).text("Flicker speed (a second)"));
        ui.add_enabled(there, egui::Checkbox::new(&mut look.lantern, "Lantern")).on_hover_text("The L key while playing");
        ui.add_enabled(there, egui::Slider::new(&mut look.lantern_level, 1.0..=15.0).step_by(1.0).text("Lantern brightness"));
        ui.horizontal(|ui| {
            ui.add_enabled_ui(there, |ui| ui.color_edit_button_rgb(&mut look.lantern_colour));
            ui.label("Lantern colour");
        });
    });
    ui.separator();
    let Some(look) = look else {
        ui.label("No far eye (the doorway couldn't open the world).");
        return;
    };
    section(
        ui,
        "render: the picture",
        "The picture",
        true,
        &mut *look,
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
            ui.add(egui::Slider::new(&mut look.sun_from, 0.0..=360.0).text("sun from (degrees, north to east)"));
            ui.add(egui::Slider::new(&mut look.sun_height, -90.0..=90.0).text("sun height (degrees)"));
            // Night (build 225) [David, 2026-11-18: "Night by the sun slider", "Moonlit, with a slider"; "In Light,
            // \"night brightness\""]: the sun under the horizon, the land lit by this share of the day's sky light.
            ui.add(egui::Slider::new(&mut look.night_brightness, 0.0..=crate::far_eye::MOST_NIGHT).text("night brightness"));
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
            ui.checkbox(&mut look.fog, "fog");
            ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_reach, 500.0..=20_000.0).logarithmic(true).text("fog reach (blocks)"));
            ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_height, 25.0..=1_000.0).text("fog thins every (blocks up)"));
            ui.add_enabled(look.fog, egui::Slider::new(&mut look.fog_warmth, 0.0..=1.0).text("fog warmth towards the sun"));
        },
    );
}

/// The Camera page: where the camera is, how fast it flies (its speed and Shift's multiplier) and its field of view
/// [David, 2026-11-17: "i think we need to add FOV and Panini under camera again. it helps for me to be able to set
/// low FOV and zoom into seams to inspect"; "fov for now"]. All three are kept in the page's presets.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Camera and Render pages laid out off the screen with their preset bars, a few frames each, Ctrl+S among
    /// them: nothing faults, and Ctrl+S saves the showing page's settings under the name in its box.
    #[test]
    fn the_pages_lay_out_with_their_preset_bars_and_ctrl_s_saves() {
        let root = std::env::temp_dir().join("incus artus preset tests").join("the panel");
        let _ = std::fs::remove_dir_all(&root);
        let mut panel = Panel::with_presets_in(&root);
        let ctx = egui::Context::default();
        let passes = vec![PassTimes::default(); crate::wiring::PASSES.len()];
        let readings = Readings {
            card: "no card",
            picture: (1600, 1000),
            present: wgpu::PresentMode::Fifo,
            refresh: 60.0,
            frame: Spread::default(),
            passes: &passes,
            card_whole: None,
            camera: Camera::new(),
            captured: false,
            memory: None,
            blends: [true, true],
            faces_switch: true,
            near: None,
            far: "none".to_string(),
        };
        let (mut far, mut near, mut pace, mut fov) = (FarLook::DEFAULT, NearLook::DEFAULT, Pace::DEFAULT, 70.0);
        (far.fog, near.radius, near.lantern, pace.speed) = (true, 12.0, true, 333.0);
        let ctrl_s = egui::Event::Key { key: egui::Key::S, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::COMMAND };
        for page in 0..PAGES.len() {
            panel.page = page;
            (panel.camera.name, panel.render.name) = ("try".to_string(), "try".to_string());
            for frame in 0..4 {
                let events = if frame == 2 { vec![ctrl_s.clone()] } else { Vec::new() };
                ctx.begin_pass(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 1000.0))), events, ..Default::default() });
                panel.show(&ctx, &readings, Some(&mut far), Some(&mut near), &mut pace, &mut fov);
                // (The font pictures egui made are let go of: there's no card here to hand them to.)
                ctx.end_pass().textures_delta.clear();
            }
        }
        let render: RenderPreset = toml::from_str(&std::fs::read_to_string(root.join("render").join("try.toml")).unwrap()).unwrap();
        assert_eq!(render, RenderPreset::of(&near, &far));
        assert!(render.far.fog && render.near.radius == 12.0 && !render.near.lantern && near.lantern, "the settings as they stood, the lantern left lit and not kept");
        let camera: CameraPreset = toml::from_str(&std::fs::read_to_string(root.join("camera").join("try.toml")).unwrap()).unwrap();
        assert_eq!(camera, CameraPreset { speed: 333.0, multiplier: Pace::DEFAULT.multiplier, field_of_view: 70.0 });
        let _ = std::fs::remove_dir_all(&root);
    }
}

//! The look the World Compiler's pictures still use, and the old renderer's settings that are kept
//! (the regroup, docs/regroup.md: the renderer was cut, the World Gen suite wasn't).
//!
//! - **The block colours** (`Palette`): each block's top and side as the ground shows them once a
//!   block is smaller than a pixel (each texture's smallest shrunk version, as light).
//! - **The shading** (`Shade`): face brightness, hill shading, the sun and the steps' shade, worked out
//!   as the old far view's were, so the World Compiler's block picture looks as it did.
//! - **The settings groups `render`, `distant`, `light`, `far_copies` and `panini`** are kept as they were loaded, whole
//!   (`KeptSettings`): presets saved now still hold every value David tuned for the old renderer, and
//!   the few this file reads are written back with them. The new renderer gets its own settings.
//! - **Reading block textures** (`read_png`, `load_texture`, `far_colour`).

use crate::data::{BlockRegistry, FACE_NAMES, block_texture_folder};
use crate::debug::DebugAppExt;
use crate::debug::SettingsAppExt;
use crate::world::{BlockId, TerrainBlocks};
use crate::ecs::prelude::*;


/// The seams' names in the settings, by their finer layer (`draw::SEAMS`).
const SEAM_NAMES: [&str; 4] = ["4", "2", "1_5", "1"];

pub struct LookPlugin;

impl Plugin for LookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeptSettings>()
            .add_settings_with("render", |world| Ok(world.resource::<KeptSettings>().render.saved()), |world, value| world.resource_mut::<KeptSettings>().render.load(value))
            .add_settings_with("distant", |world| Ok(world.resource::<KeptSettings>().distant.saved()), |world, value| world.resource_mut::<KeptSettings>().distant.load(value))
            .add_settings_with("light", |world| Ok(world.resource::<KeptSettings>().light.saved()), |world, value| world.resource_mut::<KeptSettings>().light.load(value))
            .add_settings_with("far_copies", |world| Ok(world.resource::<KeptSettings>().far_copies.saved()), |world, value| world.resource_mut::<KeptSettings>().far_copies.load(value))
            .add_settings_with("panini", |world| Ok(world.resource::<KeptSettings>().panini.saved()), |world, value| world.resource_mut::<KeptSettings>().panini.load(value))
            .add_debug_page("Render", render_debug_page)
            .add_page_preset("render", &["render", "distant", "far_copies", "light"])
            .add_systems(Last, hand_over_the_frame_cap);
    }
}

/// The "Render" page: its presets (the old renderer's, kept whole) and the frame cap. The rest of the
/// old page went with the old renderer; the new one brings its own settings.
fn render_debug_page(ui: &mut egui::Ui, world: &mut World) {
    crate::debug::page_presets(ui, world, "render");
    ui.separator();
    ui.small("The world is drawn by sight's first form (the regroup, 3.2): the coarse layer alone. These presets still hold every value tuned for the old renderer.");
    ui.strong("Frame limit");
    let refresh = world.resource::<crate::runner::WindowState>().refresh;
    // How quick sight is (shown with its settings below): the frames, and the card's clock on sight alone.
    let speed = format!(
        "{} | sight on the card: {}",
        crate::perf::fps_line(world),
        crate::sight::draw::card_ms().map_or("(the card can't time itself)".to_string(), |ms| format!("{ms:.2} ms"))
    );
    let mut kept = world.resource_mut::<KeptSettings>();
    let mut chosen = kept.frame_limit_name();
    ui.vertical(|ui| {
        for (name, label) in [
            (UNDER_THE_TOP, format!("just under the screen's top rate ({:.0}): for a screen with G-SYNC or FreeSync", under_the_top(refresh))),
            (SCREEN_RATE, format!("the screen's rate ({refresh:.0}), by vsync: for a screen without them")),
            ("120", "120".to_string()),
            (UNCAPPED, "uncapped: as fast as it can, for measuring the room left".to_string()),
        ] {
            ui.radio_value(&mut chosen, name.to_string(), label);
        }
    });
    if chosen != kept.frame_limit_name() {
        kept.set_frame_limit_name(chosen);
    }
    let mut freeze = kept.freeze_when_away();
    if ui.checkbox(&mut freeze, "Freeze when the game isn't in front").changed() {
        kept.render.table.insert("freeze_when_away".to_string(), toml::Value::Boolean(freeze));
    }
    ui.small("Frozen, the game stays on the screen under a grey veil, the world stops and nothing is drawn till it's clicked again. F11: fullscreen or a window.");
    ui.separator();
    ui.strong("Sight (the screen pyramid: the coarsest layer everywhere, finer where it's worth it)");
    // How quick it is [David 2026-11-08: "worth adding more render settings to check out what different combos
    // look like and how quick they are"]: the frames, and the card's own clock on sight's work.
    ui.label(egui::RichText::new(speed).monospace());
    // Where the pyramid starts [David 2026-11-08: "a set of selections for coarsest rays, so i can experiment with
    // nixing the 8x8 layer"]: everything is marched there.
    let mut coarsest = kept.sight_coarsest();
    ui.horizontal(|ui| {
        ui.label("Coarsest layer:");
        for factor in [1, 2, 4, 8] {
            ui.radio_value(&mut coarsest, factor, format!("{factor} × {factor}"));
        }
    });
    if coarsest != kept.sight_coarsest() {
        kept.render.set_number("sight_coarsest", coarsest as f32);
    }
    let mut grid = kept.sight_grid();
    ui.horizontal(|ui| {
        ui.label("Finest layer:");
        for factor in [1.0f32, 1.5, 2.0, 4.0, 8.0] {
            // Coarser than the coarsest: greyed (the finest is never coarser than where it starts).
            ui.add_enabled_ui(factor <= coarsest as f32, |ui| {
                ui.radio_value(&mut grid, factor, format!("{factor} × {factor}"));
            });
        }
    });
    if grid != kept.sight_grid() {
        kept.render.set_number("sight_grid", grid);
    }
    // 1.5 between 2 × 2 and 1 × 1 [David 2026-11-08: "perhaps to help blend the 1x1 to 2x2"]: only with the
    // finest at 1 × 1 and the coarsest at 2 × 2 or more.
    let mut between = kept.sight_between();
    ui.add_enabled_ui(grid < 1.25 && coarsest >= 2, |ui| {
        if ui.checkbox(&mut between, "with a 1.5 layer between 2 × 2 and 1 × 1").changed() {
            kept.render.table.insert("sight_between".to_string(), toml::Value::Boolean(between));
        }
    });
    // Where each change between layers falls [David 2026-11-08: "do you believe we'll be able to tune the ways
    // that it pumps between scales?"]: a strip for each step of the pyramid, each with its own sliders [David
    // 2026-11-09, after build 85's crossover: "i think it'd be better if we went back to a single strip for each
    // successive stage of the pyramid enabled to the next with their own sliders"]. A step not in the pyramid now is
    // greyed. The same keys as the crossover's, so nothing set is lost.
    egui::CollapsingHeader::new("Fades between layers").default_open(true).show(ui, |ui| {
        use crate::sight::draw::SEAMS;
        let layers = crate::sight::draw::factors(crate::sight::draw::Eye { grid, coarsest, between, ..Default::default() });
        // Each step by its finer layer: in use when that layer is in the pyramid under another.
        let coarser = |finer: f32| layers.iter().position(|&f| (f - finer).abs() < 0.01).filter(|&k| k > 0).map(|k| layers[k - 1]);
        let name = |f: f32| if (f - 1.5).abs() < 0.01 { "1.5".to_string() } else { format!("{f} × {f}") };
        ui.small("Near you, among the blocks: where the finer layer fades out, as shares of the blocks' reach (about 336 blocks), from where it starts fading to where it's gone. Past the blocks: where it fades back in, when its pixel is so many times the cells it reads, over so many doublings of distance.");
        let near = kept.sight_blocks_fades();
        let far = kept.sight_cells_fades();
        for (k, &finer) in SEAMS.iter().enumerate() {
            let above = coarser(finer);
            ui.add_enabled_ui(above.is_some(), |ui| {
                let title = match above {
                    Some(above) => format!("{} into {}", name(above), name(finer)),
                    None => format!("into {} (not in use)", name(finer)),
                };
                ui.strong(title);
                let [mut from, mut to] = near[k];
                ui.horizontal(|ui| {
                    let a = ui.add(egui::Slider::new(&mut from, 0.0..=1.0).step_by(0.01).text("from"));
                    let b = ui.add(egui::Slider::new(&mut to, 0.0..=1.0).step_by(0.01).text("to (among the blocks)"));
                    if a.changed() || b.changed() {
                        // Never ending before it starts.
                        if a.changed() {
                            to = to.max(from);
                        } else {
                            from = from.min(to);
                        }
                        kept.render.set_number(&format!("sight_fade_{}_from", SEAM_NAMES[k]), from);
                        kept.render.set_number(&format!("sight_fade_{}_to", SEAM_NAMES[k]), to);
                    }
                });
                let [mut start, mut doublings] = far[k];
                ui.horizontal(|ui| {
                    if ui.add(egui::Slider::new(&mut start, 0.25..=4.0).logarithmic(true).text("starts at (pixel / cell)")).changed() {
                        kept.render.set_number(&format!("sight_fade_{}_cells_start", SEAM_NAMES[k]), start);
                    }
                    if ui.add(egui::Slider::new(&mut doublings, 0.05..=4.0).text("over (doublings, past the blocks)")).changed() {
                        kept.render.set_number(&format!("sight_fade_{}_cells_doublings", SEAM_NAMES[k]), doublings);
                    }
                });
            });
        }
        if ui.button("Fades back to as they were").clicked() {
            for name in SEAM_NAMES {
                for part in ["from", "to", "cells_start", "cells_doublings"] {
                    kept.render.table.remove(&format!("sight_fade_{name}_{part}"));
                }
            }
            kept.render.table.remove("sight_fade_cells_start");
            kept.render.table.remove("sight_fade_cells_doublings");
        }
    });
    let mut earned = kept.sight_earned();
    ui.horizontal(|ui| {
        ui.label("Finer layers:");
        ui.radio_value(&mut earned, true, "only where they're worth it");
        ui.radio_value(&mut earned, false, "everywhere");
    });
    if earned != kept.sight_earned() {
        kept.render.table.insert("sight_earned".to_string(), toml::Value::Boolean(earned));
    }
    // The far march's hand-over between levels [the dyno, 2026-11-10: at 0.8 the whole flight's 95th 6.7% faster, out
    // at sea the median 7%; for David's eye on rings over flat water, build 69's].
    let mut blend_from = kept.sight_blend_from();
    if ui.add(egui::Slider::new(&mut blend_from, 0.3..=0.95).step_by(0.05).text("level hand-over starts at (later: narrower and faster; too late and rings show over flat water)")).changed() {
        kept.render.set_number("sight_blend_from", blend_from);
    }
    // The Oracle's experiment 2 [2026-11-11]: the far cells read crisp, one at a time, against today's blend of eight,
    // for David's eye (it decides the rebuild's bricks: crisp, plain and smaller; soft, textures the card blends).
    let mut crisp = kept.sight_crisp();
    if ui.checkbox(&mut crisp, format!("Far cells crisp: each read alone, not blended with its neighbours (experiment 2; the crisp build {})", crate::sight::draw::crisp_state())).changed() {
        kept.render.set_flag("sight_crisp", crisp);
    }
    // Block textures (3.6) [David 2026-11-08: "so the blocks are mipmapped, right? go ahead"]: on the blocks near
    // the eye; a texture changed on disk shows at once (the data watch).
    let mut textures = kept.sight_textures();
    if ui.checkbox(&mut textures, "Block textures (on the blocks near you; a texture changed on disk shows at once)").changed() {
        kept.render.table.insert("sight_textures".to_string(), toml::Value::Boolean(textures));
    }
    ui.add_enabled_ui(textures, |ui| {
        let mut sharpness = kept.sight_texture_sharpness();
        if ui.add(egui::Slider::new(&mut sharpness, -2.0..=2.0).step_by(0.1).text("texture sharpness (higher crisper, lower smoother as faces shrink)")).changed() {
            kept.render.set_number("sight_texture_sharpness", sharpness);
        }
    });
    // The bake's share of the card [the start bake 46 s before shadows, 281 s with sky light: richer frames left it no
    // room].
    let mut share = kept.sight_fill_share();
    if ui.add(egui::Slider::new(&mut share, 0.0..=0.9).step_by(0.05).text("of the card left to the far view's bake, while it bakes")).changed() {
        kept.render.set_number("sight_fill_share", share);
    }
    ui.small(if crate::runner::filling_now() { "Baking now: the frames are paced to leave it its share (more: it's in sooner, the frames slower)." } else { "Nothing baking now: the frames go as fast as the limit lets them." });
    let mut rays = kept.sight_rays();
    ui.horizontal(|ui| {
        ui.label("Rays a coarse pixel:");
        for n in [1, 4] {
            ui.radio_value(&mut rays, n, n.to_string());
        }
    });
    if rays != kept.sight_rays() {
        kept.render.set_number("sight_rays", rays as f32);
    }
    let mut upscale = kept.sight_upscale();
    ui.horizontal(|ui| {
        ui.label("Upscale:");
        for (mode, name) in [(0, "crisp"), (1, "smooth"), (2, "smooth, sharp at skylines")] {
            ui.radio_value(&mut upscale, mode, name);
        }
    });
    if upscale != kept.sight_upscale() {
        kept.render.set_number("sight_upscale", upscale as f32);
    }
    ui.small("A new grid sizes the store round you again: it fills in again, raw, for some seconds. Rays and the upscale change at once.");
    ui.separator();
    // The one sun (3.4a, David 2026-11-08: "sun sliders are a good idea"): the world's light, and the World
    // Compiler's pictures' too (the `light` group's `sun_from` and `sun_height`, kept in the Render presets).
    ui.strong("Sun");
    let mut from = kept.light.number("sun_from", 315.0).rem_euclid(360.0);
    if ui.add(egui::Slider::new(&mut from, 0.0..=360.0).step_by(1.0).suffix("°").text("from (0 north, 90 east, 180 south, 270 west)")).changed() {
        kept.light.set_number("sun_from", from);
    }
    // Under the horizon for night [David 2026-11-09: darkness first, "yes"]: the sun's light gone 2° under, the sky's
    // dusk over by 18° under, leaving night's light.
    let mut height = kept.light.number("sun_height", 40.0).clamp(-30.0, 90.0);
    if ui.add(egui::Slider::new(&mut height, -30.0..=90.0).step_by(1.0).suffix("°").text("height over the horizon (under 0: dusk, then night)")).changed() {
        kept.light.set_number("sun_height", height);
    }
    let mut night = kept.sight_night_light();
    if ui.add(egui::Slider::new(&mut night, 0.0..=0.5).step_by(0.005).text("night light (what's left of the sky's light, the sun well down)")).changed() {
        kept.render.set_number("sight_night_light", night);
    }
    let mut shadows = kept.sight_shadows();
    if ui.checkbox(&mut shadows, "Shadows").changed() {
        kept.render.table.insert("sight_shadows".to_string(), toml::Value::Boolean(shadows));
    }
    // Sky light (3.4b): how much of the sky a place sees [David 2026-11-08: "tint slider is fine, i can't decide what
    // color the light should be at this stage anyways"].
    let mut sky = kept.sight_sky();
    if ui.checkbox(&mut sky, "Sky light (valleys and hollows see less of the sky)").changed() {
        kept.render.table.insert("sight_sky".to_string(), toml::Value::Boolean(sky));
    }
    let mut strength = kept.sight_sky_strength();
    ui.add_enabled_ui(sky, |ui| {
        if ui.add(egui::Slider::new(&mut strength, 0.0..=1.0).step_by(0.01).text("its strength (how dark a place seeing no sky goes)")).changed() {
            kept.render.set_number("sight_sky_strength", strength);
        }
    });
    let mut tint = kept.sight_sky_tint();
    if ui.add(egui::Slider::new(&mut tint, 0.0..=1.0).step_by(0.01).text("the sky's tint (0 white, 1 sky blue)")).changed() {
        kept.render.set_number("sight_sky_tint", tint);
    }
    ui.small("A low sun brings the land's shape out; overhead, it flattens it. The World Compiler's pictures take the same sun (not its shadows or sky light), never lower than 5°.");
    // Placed lights (build 95): the light objects give (a brazier's), through the air, round corners, not through
    // walls; the old renderer's keys (`placed_light`, `flicker`, `flicker_speed`), so its presets keep their say.
    ui.separator();
    ui.strong("Placed lights");
    let mut placed = kept.sight_placed_lights();
    if ui.checkbox(&mut placed, "Placed lights (what objects like the brazier light round them)").changed() {
        kept.render.table.insert("placed_light".to_string(), toml::Value::Boolean(placed));
    }
    ui.add_enabled_ui(placed, |ui| {
        let mut flicker = kept.sight_flicker();
        if ui.add(egui::Slider::new(&mut flicker, 0.0..=1.0).step_by(0.01).text("flicker (0 steady, 1 down to dark)")).changed() {
            kept.render.set_number("flicker", flicker);
        }
        let mut speed = kept.sight_flicker_speed();
        if ui.add(egui::Slider::new(&mut speed, 0.0..=5.0).step_by(0.05).text("how fast it flickers")).changed() {
            kept.render.set_number("flicker_speed", speed);
        }
        // A block's step at a time, hard-edged, unless chosen [David 2026-11-09: "i just love the quantized per block
        // lighting with the hard edges"].
        let mut smooth = kept.sight_smooth_light();
        if ui.checkbox(&mut smooth, "blended across faces (off: a block's step at a time, hard-edged)").changed() {
            kept.render.table.insert("placed_light_smooth".to_string(), toml::Value::Boolean(smooth));
        }
    });
    // The near walk's truth (the dyno, 2026-11-09; Counsel: so David can check a place himself): every block near the
    // eye walked, no air skipped over. Slow; if the picture changes when it's flipped, the skip is wrong there.
    ui.separator();
    ui.strong("Checks");
    let mut every = kept.sight_walk_every_block();
    if ui.checkbox(&mut every, "Walk every block near the eye (slow: no air skipped; if the picture changes, the skip is wrong there)").changed() {
        kept.render.table.insert("sight_walk_every_block".to_string(), toml::Value::Boolean(every));
    }
    // The march in stages and light per face near the eye (the dyno, 2026-11-09), the game's drawing since build 104
    // [David 2026-11-10, asked whether they become the normal drawing: "yes"; the switches stay here to flip back].
    let mut stages = kept.sight_stages();
    if ui.checkbox(&mut stages, "Draw in stages (the march, then the shading, each a pass of its own; off: all in one pass)").changed() {
        kept.render.table.insert("sight_stages".to_string(), toml::Value::Boolean(stages));
    }
    let mut per_face = kept.sight_face_light();
    if ui.add_enabled(stages, egui::Checkbox::new(&mut per_face, "Light per face near the eye (each block face's shadow and sky light worked out once, at its middle; in stages)")).changed() {
        kept.render.table.insert("sight_face_light".to_string(), toml::Value::Boolean(per_face));
    }
    // Fog (3.4c) [David 2026-10-28: fog out to about 2 miles; 2026-11-08: "that sounds alright"].
    ui.separator();
    ui.strong("Fog");
    let mut fog = kept.sight_fog();
    if ui.checkbox(&mut fog, "Fog (far land fades into the sky; valleys hazier than peaks)").changed() {
        kept.render.table.insert("sight_fog".to_string(), toml::Value::Boolean(fog));
    }
    ui.add_enabled_ui(fog, |ui| {
        let mut reach = kept.sight_fog_reach();
        if ui.add(egui::Slider::new(&mut reach, 300.0..=50_000.0).logarithmic(true).step_by(10.0).suffix(" blocks").text("how far you see at sea level (land there mostly fog; 3,200 about 2 miles)")).changed() {
            kept.render.set_number("sight_fog_reach", reach);
        }
        let mut height = kept.sight_fog_height();
        if ui.add(egui::Slider::new(&mut height, 20.0..=2000.0).logarithmic(true).step_by(1.0).suffix(" blocks").text("how high the haze lies (it halves each so many blocks up)")).changed() {
            kept.render.set_number("sight_fog_height", height);
        }
        let mut warmth = kept.sight_fog_warmth();
        if ui.add(egui::Slider::new(&mut warmth, 0.0..=1.0).step_by(0.01).text("warmer looking towards the sun")).changed() {
            kept.render.set_number("sight_fog_warmth", warmth);
        }
    });
}

/// The frame limits' names in the settings.
pub const UNDER_THE_TOP: &str = "just under the top";
pub const SCREEN_RATE: &str = "the screen's rate";
pub const UNCAPPED: &str = "uncapped";

/// Just under the screen's top rate: 3 frames a second under it, so a screen with a variable refresh
/// rate shows every frame the moment it's ready and never makes one wait [David 2026-11-05: "setting
/// our frames at 237 or 240 if its a 250hz"].
pub fn under_the_top(refresh: f32) -> f32 {
    (refresh - 3.0).max(30.0)
}

/// The frame limit a setting's name stands for, on a screen of this refresh rate (a number is so many
/// frames a second).
pub fn frame_limit(name: &str, refresh: f32) -> crate::runner::FrameLimit {
    use crate::runner::FrameLimit;
    match name {
        UNCAPPED => FrameLimit::Uncapped,
        SCREEN_RATE => FrameLimit::ScreenRate,
        _ => name.parse::<f32>().ok().filter(|fps| *fps >= 10.0).map_or(FrameLimit::Steady(under_the_top(refresh)), FrameLimit::Steady),
    }
}

/// The frame limit and the freeze from the `render` group, for the runner. While the game measures
/// itself (a soak test or test window) it never freezes.
fn hand_over_the_frame_cap(kept: Res<KeptSettings>, mut cap: ResMut<crate::runner::FrameCap>, mut window: ResMut<crate::runner::WindowState>) {
    let wanted = frame_limit(&kept.frame_limit_name(), window.refresh);
    if cap.0 != wanted {
        cap.0 = wanted;
    }
    let freeze = kept.freeze_when_away() && !crate::soak::measuring();
    if window.freeze_when_away != freeze {
        window.freeze_when_away = freeze;
    }
}

/// A settings group kept whole as it was loaded (a TOML table), read by name.
#[derive(Clone, Default)]
pub struct KeptGroup {
    table: toml::Table,
}

impl KeptGroup {
    fn load(&mut self, value: toml::Value) -> Result<(), String> {
        self.table = value.try_into().map_err(|error: toml::de::Error| error.to_string())?;
        Ok(())
    }

    fn saved(&self) -> toml::Value {
        toml::Value::Table(self.table.clone())
    }

    /// A number in it, or `default` if it isn't there.
    pub fn number(&self, name: &str, default: f32) -> f32 {
        match self.table.get(name) {
            Some(toml::Value::Float(value)) => *value as f32,
            Some(toml::Value::Integer(value)) => *value as f32,
            _ => default,
        }
    }

    pub fn flag(&self, name: &str, default: bool) -> bool {
        self.table.get(name).and_then(toml::Value::as_bool).unwrap_or(default)
    }

    pub fn set_number(&mut self, name: &str, value: f32) {
        self.table.insert(name.to_string(), toml::Value::Float(value as f64));
    }

    pub fn set_flag(&mut self, name: &str, value: bool) {
        self.table.insert(name.to_string(), toml::Value::Boolean(value));
    }
}

/// The old renderer's settings groups, kept whole (see the module notes). The defaults are the old
/// renderer's own.
#[derive(Resource, Clone, Default)]
pub struct KeptSettings {
    pub render: KeptGroup,
    pub distant: KeptGroup,
    pub light: KeptGroup,
    pub far_copies: KeptGroup,
    /// The camera page's Panini projection (cut with the old renderer).
    pub panini: KeptGroup,
}

impl KeptSettings {
    /// How far the real chunks are made round you, in chunks.
    pub fn render_distance_chunks(&self) -> i32 {
        self.render.number("render_distance_chunks", 32.0) as i32
    }

    /// The frame limit's name (`frame_limit`): just under the screen's top rate unless chosen. (The
    /// old `frame_cap`, a frame every so many refreshes, is left in old presets and not read.)
    pub fn frame_limit_name(&self) -> String {
        match self.render.table.get("frame_limit") {
            Some(toml::Value::String(name)) => name.clone(),
            Some(toml::Value::Integer(fps)) => fps.to_string(),
            _ => UNDER_THE_TOP.to_string(),
        }
    }

    pub fn set_frame_limit_name(&mut self, name: String) {
        self.render.table.insert("frame_limit".to_string(), toml::Value::String(name));
    }

    /// Sight's finest layer: how many screen pixels a side its pixels are (1, 1.5, 2, 4 or 8; 2 unless chosen).
    pub fn sight_grid(&self) -> f32 {
        // For measuring: `INCUS_SIGHT_GRID` chooses it.
        let chosen: f32 = std::env::var("INCUS_SIGHT_GRID").ok().and_then(|g| g.parse().ok()).unwrap_or(self.render.number("sight_grid", 2.0));
        [1.0, 1.5, 2.0, 4.0, 8.0].into_iter().find(|&f| (f - chosen).abs() < 0.01).unwrap_or(2.0)
    }

    /// With the finest at 1 × 1, a layer of 1.5 between it and 2 × 2 (off unless chosen).
    pub fn sight_between(&self) -> bool {
        self.render.flag("sight_between", false)
    }

    /// Each seam's fade among the blocks, by its finer layer (`draw::SEAMS`; shares of their reach, from and to;
    /// `draw::BLOCKS_FADE` unless chosen). Till build 85 the 4 × 4 seam took the 2 × 2's: a preset from then
    /// keeps its look.
    pub fn sight_blocks_fades(&self) -> [[f32; 2]; 4] {
        let d = crate::sight::draw::BLOCKS_FADE;
        let two = [self.render.number("sight_fade_2_from", d[0]), self.render.number("sight_fade_2_to", d[1])];
        SEAM_NAMES.map(|name| {
            let d = if name == "4" { two } else { d };
            [self.render.number(&format!("sight_fade_{name}_from"), d[0]), self.render.number(&format!("sight_fade_{name}_to"), d[1])]
        })
    }

    /// Each seam's fade in past the blocks, by its finer layer: where it starts (its pixel so many times its
    /// cells) and over how many doublings (`draw::CELLS_FADE` unless chosen). Till build 85 one for all: a
    /// preset from then gives each seam that one.
    pub fn sight_cells_fades(&self) -> [[f32; 2]; 4] {
        let d = crate::sight::draw::CELLS_FADE;
        let all = [self.render.number("sight_fade_cells_start", d[0]), self.render.number("sight_fade_cells_doublings", d[1])];
        SEAM_NAMES.map(|name| [self.render.number(&format!("sight_fade_{name}_cells_start"), all[0]), self.render.number(&format!("sight_fade_{name}_cells_doublings"), all[1])])
    }

    /// Sight's coarsest layer, where the pyramid starts (1, 2, 4 or 8; 4 unless chosen [David 2026-11-08: "i
    /// liked the looks of 2x finest, 4x coarsest"]).
    pub fn sight_coarsest(&self) -> u32 {
        // For measuring: `INCUS_SIGHT_COARSEST` chooses it.
        let chosen = std::env::var("INCUS_SIGHT_COARSEST").ok().and_then(|g| g.parse().ok()).unwrap_or(self.render.number("sight_coarsest", 4.0));
        match chosen as u32 {
            1 => 1,
            2 => 2,
            8 => 8,
            _ => 4,
        }
    }

    /// Whether the pyramid's finer layers are marched only where they're worth it (a pixel no finer than the
    /// cells it reads), or everywhere (for comparing). Only where they're worth it unless chosen.
    pub fn sight_earned(&self) -> bool {
        // For measuring: `INCUS_SIGHT_EVERYWHERE` refines everywhere.
        !std::env::var("INCUS_SIGHT_EVERYWHERE").is_ok() && self.render.flag("sight_earned", true)
    }

    /// Rays a coarse pixel (1 unless chosen).
    pub fn sight_rays(&self) -> u32 {
        if self.render.number("sight_rays", 1.0) as u32 >= 4 { 4 } else { 1 }
    }

    /// How the coarse layer is brought up to the screen: 0 crisp, 1 smooth, 2 smooth within a surface but
    /// sharp at skylines [David 2026-11-06: "can we have a switch?"; opens on crisp].
    pub fn sight_upscale(&self) -> u32 {
        (self.render.number("sight_upscale", 0.0) as u32).min(2)
    }

    /// While the far view bakes, the share of the card left to the bake (0 to 0.9; the frame paced so drawing takes
    /// the rest; 0.5 unless chosen; `INCUS_SIGHT_FILL_SHARE` for measuring, 0 for none).
    pub fn sight_fill_share(&self) -> f32 {
        let chosen = std::env::var("INCUS_SIGHT_FILL_SHARE").ok().and_then(|v| v.parse().ok()).unwrap_or(self.render.number("sight_fill_share", 0.5));
        chosen.clamp(0.0, 0.9)
    }

    /// Whether the sun casts shadows in sight (3.4b; on unless chosen).
    pub fn sight_shadows(&self) -> bool {
        self.render.flag("sight_shadows", true)
    }

    /// Sky light (3.4b): on unless chosen; its strength and tint (`draw::SKY_STRENGTH`, `draw::SKY_TINT` unless
    /// chosen).
    pub fn sight_sky(&self) -> bool {
        self.render.flag("sight_sky", true)
    }

    pub fn sight_sky_strength(&self) -> f32 {
        self.render.number("sight_sky_strength", crate::sight::draw::SKY_STRENGTH)
    }

    pub fn sight_sky_tint(&self) -> f32 {
        self.render.number("sight_sky_tint", crate::sight::draw::SKY_TINT)
    }

    /// Placed lights (build 95): on unless chosen; how much and how fast they flicker (`draw::FLICKER`,
    /// `FLICKER_SPEED` unless chosen). The old renderer's keys.
    pub fn sight_placed_lights(&self) -> bool {
        self.render.flag("placed_light", true)
    }

    pub fn sight_flicker(&self) -> f32 {
        self.render.number("flicker", crate::sight::draw::FLICKER)
    }

    pub fn sight_flicker_speed(&self) -> f32 {
        self.render.number("flicker_speed", crate::sight::draw::FLICKER_SPEED)
    }

    /// Placed light blended across faces (off unless chosen: a block's step at a time).
    pub fn sight_smooth_light(&self) -> bool {
        self.render.flag("placed_light_smooth", false)
    }

    /// Every block near the eye walked, no air skipped (a check, off unless chosen).
    pub fn sight_walk_every_block(&self) -> bool {
        self.render.flag("sight_walk_every_block", false)
    }

    /// The march drawn in stages (the dyno's; on unless chosen, since build 104: David 2026-11-10, "yes").
    pub fn sight_stages(&self) -> bool {
        self.render.flag("sight_stages", true)
    }

    /// Light per face near the eye, in stages (on unless chosen, since build 104).
    pub fn sight_face_light(&self) -> bool {
        self.render.flag("sight_face_light", true)
    }

    /// Where in a level's stretch the far march's hand-over to the next begins (`draw::BLEND_FROM` unless chosen).
    pub fn sight_blend_from(&self) -> f32 {
        self.render.number("sight_blend_from", crate::sight::draw::BLEND_FROM)
    }

    /// The far cells read crisp, one at a time (experiment 2; off unless chosen).
    pub fn sight_crisp(&self) -> bool {
        self.render.flag("sight_crisp", false)
    }

    /// Night's light (what's left of the sky's light with the sun well down; `draw::NIGHT_LIGHT` unless chosen).
    pub fn sight_night_light(&self) -> f32 {
        self.render.number("sight_night_light", crate::sight::draw::NIGHT_LIGHT)
    }

    /// Block textures on the near faces (3.6): on unless chosen; how sharp (a shift in the shrunk versions taken:
    /// above 0 crisper, below smoother; 0 unless chosen).
    pub fn sight_textures(&self) -> bool {
        self.render.flag("sight_textures", true)
    }

    pub fn sight_texture_sharpness(&self) -> f32 {
        self.render.number("sight_texture_sharpness", 0.0)
    }

    /// Fog (3.4c): on unless chosen; how far one sees, how high the air thins, its warmth towards the sun
    /// (`draw::FOG_REACH`, `FOG_HEIGHT`, `FOG_WARMTH` unless chosen). Sight's own: the old renderer's `fog_*`
    /// stay as they were.
    pub fn sight_fog(&self) -> bool {
        self.render.flag("sight_fog", true)
    }

    pub fn sight_fog_reach(&self) -> f32 {
        self.render.number("sight_fog_reach", crate::sight::draw::FOG_REACH)
    }

    pub fn sight_fog_height(&self) -> f32 {
        self.render.number("sight_fog_height", crate::sight::draw::FOG_HEIGHT)
    }

    pub fn sight_fog_warmth(&self) -> f32 {
        self.render.number("sight_fog_warmth", crate::sight::draw::FOG_WARMTH)
    }

    /// Whether the game freezes when it isn't in front (`freeze_when_away`, on unless chosen).
    pub fn freeze_when_away(&self) -> bool {
        self.render.flag("freeze_when_away", true)
    }

    /// The finest ring's cell size the old far view used (blocks): the distance the World Compiler's
    /// hill shading measures its slope over.
    pub fn finest_cell(&self) -> f32 {
        self.distant.number("finest_cell", 16.0)
    }

    /// Where the sun is, in game axes (from its compass direction and height), for the World Compiler's
    /// pictures: never lower than 5° (they're for seeing the land's shape, night or not).
    pub fn sun(&self) -> Vec3 {
        self.sun_at(self.light.number("sun_height", 40.0).clamp(5.0, 90.0))
    }

    /// Where the sun is for sight: as the Render page has it, under the horizon at night.
    pub fn sight_sun(&self) -> Vec3 {
        self.sun_at(self.light.number("sun_height", 40.0).clamp(-30.0, 90.0))
    }

    fn sun_at(&self, height: f32) -> Vec3 {
        let from = self.light.number("sun_from", 315.0).to_radians();
        let up = height.to_radians();
        Vec3::new(up.cos() * from.cos(), up.cos() * from.sin(), up.sin())
    }
}

/// The colours the World Compiler's block picture uses (linear RGB): every block's top and side, as
/// the ground shows them once a block is under a pixel (`far_colour`).
#[derive(Clone, Debug)]
pub struct Palette {
    /// By block number: its top's colour and its sides'.
    pub blocks: Vec<([f32; 3], [f32; 3])>,
    pub water: [f32; 3],
}

impl Palette {
    pub fn from_textures(registry: &BlockRegistry, blocks: &TerrainBlocks) -> Self {
        let (faces, water) = Self::faces(registry, blocks);
        Self::from_faces(&faces, water.as_deref())
    }

    /// The textures it reads: every block's top and side, and water's top (no files read, so it can be
    /// taken on the frame and read elsewhere: `from_faces`).
    pub fn faces(registry: &BlockRegistry, blocks: &TerrainBlocks) -> (Vec<(String, String)>, Option<String>) {
        let face = |name: &str| FACE_NAMES.iter().position(|&face| face == name).unwrap_or(0);
        let faces = registry.defs().iter().map(|def| (def.faces[face("top")].clone(), def.faces[face("north")].clone())).collect();
        (faces, registry.def(blocks.water).map(|def| def.faces[face("top")].clone()))
    }

    /// Made from those textures.
    pub fn from_faces(faces: &[(String, String)], water: Option<&str>) -> Self {
        let average = |texture: &str| far_colour(texture).unwrap_or([0.5, 0.5, 0.5]);
        Self { blocks: faces.iter().map(|(top, side)| (average(top), average(side))).collect(), water: water.map_or([0.5; 3], average) }
    }

    pub fn top(&self, block: BlockId) -> [f32; 3] {
        self.blocks.get(block.0 as usize).map_or([0.5; 3], |colours| colours.0)
    }
}

/// How the World Compiler's block picture is lit, from the kept settings (as the old far view was):
/// the face brightness for tops and sides, the hill shading (how strong, how it fades toward its
/// limits, the sun's direction in game axes), the height tint, and what block steps do to a slope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    pub top: f32,
    pub north_south: f32,
    pub east_west: f32,
    pub hills: f32,
    pub fade: f32,
    pub sun: Vec3,
    pub height_tint: f32,
    /// The steps' feet on a slope, averaged across a step (1 = no shade), for slopes from flat to
    /// `STEEPEST_STEPS` blocks up per block across: all round (`foot`), and from the sun's side
    /// (`step`); and how much the far slopes take it on (`slope_match`, 0 = none, 1 = all).
    pub foot: [f32; SLOPE_STEPS + 1],
    pub step: [f32; SLOPE_STEPS + 1],
    pub slope_match: f32,
    /// How much of the slope's sun and shade the steps' sides take.
    pub sides: f32,
    /// Light only (the old seam probe): every colour plain grey, the lighting kept.
    pub grey: bool,
}

/// The steepest slope the steps' shade is worked out for (blocks up per block across), and in how many
/// steps from flat.
pub const STEEPEST_STEPS: f32 = 4.0;
pub const SLOPE_STEPS: usize = 32;

/// How dark the foot of block steps is on average, across one step, on ground rising `rise` blocks
/// per block (so a step every 1/rise blocks): each step's top casts a cone falling a block every
/// `reach` blocks, a spot is `below` the highest cone over it (counting every step up the slope, never
/// more than `cap` blocks: a step's shade reaches only so far), eased toward the darkest by `fade`, and
/// darkened by `strength` of that. 1 = no shade.
fn steps_shade(rise: f32, strength: f32, reach: f32, cap: f32, fade: f32) -> f32 {
    if rise <= 1e-3 || strength <= 0.0 {
        return 1.0;
    }
    let (width, reach) = (1.0 / rise, reach.max(0.1));
    let eased = |below: f32| {
        if below <= 0.0 {
            return 0.0;
        }
        let p = fade.clamp(0.5, 16.0);
        below / (1.0 + below.powf(p)).powf(1.0 / p)
    };
    const ACROSS: usize = 16;
    let mut total = 0.0;
    for s in 0..ACROSS {
        // A spot this far from the foot of the step above it.
        let x = (s as f32 + 0.5) / ACROSS as f32 * width;
        let below = (1..=64).map(|k| k as f32 - (x + (k - 1) as f32 * width) / reach).fold(0.0f32, f32::max).min(cap);
        total += 1.0 - strength * eased(below);
    }
    total / ACROSS as f32
}

impl Shade {
    /// The face brightness from the `render` group, the hill shading from `distant`, the sun and the
    /// steps' shade from `light` (one sun for everything).
    pub fn new(kept: &KeptSettings) -> Self {
        let (render, distant, light) = (&kept.render, &kept.distant, &kept.light);
        let fade = distant.number("shading_fade", 2.0);
        let near = light.flag("near", true);
        let (shade_reach, step_reach) = (light.number("shade_reach", 0.5), light.number("step_reach", 17.0));
        let table = |strength: f32| std::array::from_fn(|k| steps_shade(k as f32 / SLOPE_STEPS as f32 * STEEPEST_STEPS, strength, shade_reach, step_reach / shade_reach.max(0.1), fade));
        Self {
            top: render.number("shade_top", 1.0),
            north_south: render.number("shade_north_south", 0.8),
            east_west: render.number("shade_east_west", 0.65),
            hills: distant.number("hill_shading", 0.3),
            fade,
            sun: kept.sun(),
            height_tint: distant.number("height_tint", 0.15),
            foot: table(if near { light.number("foot_shading", 0.22) } else { 0.0 }),
            step: table(if near { light.number("step_shadows", 0.1) } else { 0.0 }),
            slope_match: distant.number("slope_match", 1.0),
            sides: if near { light.number("side_shading", 0.75) } else { 0.0 },
            grey: render.flag("light_only", false),
        }
    }

    /// How brightly the ground is lit, rising `north` and `east` blocks per finest cell across it: 1
    /// on flat ground, more facing the sun, less facing away.
    pub fn hill(&self, north: f32, east: f32) -> f32 {
        let normal = Vec3::new(-north * self.hills, -east * self.hills, 1.0).normalize();
        soft_limit(normal.dot(self.sun) / self.sun.z, self.fade)
    }
}

/// Hill shading's darkest and brightest (1 = as painted).
pub const DARKEST: f32 = 0.35;
pub const BRIGHTEST: f32 = 1.6;

/// Hill shading's light, `raw` (1 = flat ground), eased toward its darkest and brightest instead of
/// stopped dead: gentle slopes are lit as they are, steep ones closer and closer to the limit,
/// more gradually the lower `fade` is (at 8 and up, much like a hard stop).
pub fn soft_limit(raw: f32, fade: f32) -> f32 {
    let d = raw - 1.0;
    let limit = if d < 0.0 { 1.0 - DARKEST } else { BRIGHTEST - 1.0 };
    let x = d.abs() / limit;
    let p = fade.clamp(0.5, 16.0);
    1.0 + d.signum() * limit * x / (1.0 + x.powf(p)).powf(1.0 / p)
}

/// Block face textures must be this many pixels square.
pub const TEXTURE_SIZE: u32 = 32;

/// A PNG's size and RGBA pixels, or why not.
pub fn read_png(path: &std::path::Path) -> Result<(u32, u32, Vec<u8>), String> {
    let bytes = std::fs::read(path).map_err(|_| format!("file not found ({})", path.display()))?;
    crate::data::objects::png_rgba(&bytes).map_err(|error| format!("not a readable PNG ({error})"))
}

/// Load a PNG as 32 × 32 RGBA pixels, or explain why not.
pub fn load_texture(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let (width, height, rgba) = read_png(path)?;
    if width != TEXTURE_SIZE || height != TEXTURE_SIZE {
        return Err(format!("is {width}×{height}, must be {TEXTURE_SIZE}×{TEXTURE_SIZE}"));
    }
    Ok(rgba)
}

/// Halve an RGBA image by averaging each 2 × 2 block of pixels, colours weighted by opacity (so a
/// see-through texture's clear pixels don't darken its coloured ones), opacity the most opaque of the
/// four (as the old mipmaps were made).
pub fn shrink(pixels: &[u8], size: u32) -> Vec<u8> {
    let half = size / 2;
    let mut out = Vec::with_capacity((half * half * 4) as usize);
    for y in 0..half {
        for x in 0..half {
            let sample = |dx: u32, dy: u32, channel: u32| pixels[(((y * 2 + dy) * size + (x * 2 + dx)) * 4 + channel) as usize] as u32;
            let corners = [(0, 0), (1, 0), (0, 1), (1, 1)];
            let alpha_sum: u32 = corners.iter().map(|&(dx, dy)| sample(dx, dy, 3)).sum();
            for channel in 0..3 {
                let value = if alpha_sum == 0 {
                    0
                } else {
                    let weighted: u32 = corners.iter().map(|&(dx, dy)| sample(dx, dy, channel) * sample(dx, dy, 3)).sum();
                    (weighted + alpha_sum / 2) / alpha_sum
                };
                out.push(value as u8);
            }
            let alpha_max = corners.iter().map(|&(dx, dy)| sample(dx, dy, 3)).max().unwrap_or(0);
            out.push(alpha_max as u8);
        }
    }
    out
}

/// A block texture's colour once a block is smaller than a pixel: its smallest shrunk version, read
/// as light (linear RGB). [2026-10-29: averaging the texture as light itself, instead of the stored
/// values, came out 18% darker on gabbro's strong speckle.]
pub fn far_colour(name: &str) -> Option<[f32; 3]> {
    let pixels = load_texture(&block_texture_folder().join(format!("{name}.png"))).ok()?;
    Some(smallest_mip_colour(pixels, TEXTURE_SIZE))
}

/// The 1 × 1 shrunk version of a `size` × `size` RGBA texture, as light.
fn smallest_mip_colour(mut pixels: Vec<u8>, mut size: u32) -> [f32; 3] {
    while size > 1 {
        pixels = shrink(&pixels, size);
        size /= 2;
    }
    let linear = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    [linear(pixels[0]), linear(pixels[1]), linear(pixels[2])]
}

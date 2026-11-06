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
    ui.strong("Sight (the screen pyramid: 8 × 8 everywhere, finer where it's worth it)");
    let mut grid = kept.sight_grid();
    ui.horizontal(|ui| {
        ui.label("Finest layer:");
        for factor in [2, 4, 8] {
            ui.radio_value(&mut grid, factor, format!("{factor} × {factor}"));
        }
    });
    if grid != kept.sight_grid() {
        kept.render.set_number("sight_grid", grid as f32);
    }
    let mut earned = kept.sight_earned();
    ui.horizontal(|ui| {
        ui.label("Finer layers:");
        ui.radio_value(&mut earned, true, "only where they're worth it");
        ui.radio_value(&mut earned, false, "everywhere");
    });
    if earned != kept.sight_earned() {
        kept.render.table.insert("sight_earned".to_string(), toml::Value::Boolean(earned));
    }
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

    /// Sight's coarse grid: how many screen pixels a side a coarse pixel is (2, 4 or 8; 2 unless chosen).
    pub fn sight_grid(&self) -> u32 {
        // For measuring: `INCUS_SIGHT_GRID` chooses it.
        let chosen = std::env::var("INCUS_SIGHT_GRID").ok().and_then(|g| g.parse().ok()).unwrap_or(self.render.number("sight_grid", 2.0));
        match chosen as u32 {
            4 => 4,
            8 => 8,
            _ => 2,
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

    /// Whether the game freezes when it isn't in front (`freeze_when_away`, on unless chosen).
    pub fn freeze_when_away(&self) -> bool {
        self.render.flag("freeze_when_away", true)
    }

    /// The finest ring's cell size the old far view used (blocks): the distance the World Compiler's
    /// hill shading measures its slope over.
    pub fn finest_cell(&self) -> f32 {
        self.distant.number("finest_cell", 16.0)
    }

    /// Where the sun is, in game axes (from its compass direction and height).
    pub fn sun(&self) -> Vec3 {
        let from = self.light.number("sun_from", 315.0).to_radians();
        let up = self.light.number("sun_height", 40.0).clamp(5.0, 90.0).to_radians();
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
fn shrink(pixels: &[u8], size: u32) -> Vec<u8> {
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

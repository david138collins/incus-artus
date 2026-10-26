//! Light map — how lit the ground is, worked out ahead of time from its heights, for the distant
//! terrain to draw with.
//!
//! The ground is a height field (one height per spot), so its light can be worked out in 2D, in
//! the background, rather than every frame. The result is a picture over the whole world, one
//! pixel per `detail` blocks, which the shader reads once per screen pixel and multiplies its
//! colour by. It only ever lightens and darkens: a colour stays the colour it was painted.
//!
//! The picture's four channels:
//! - **sun** (red): 1 in sunlight, 0 in a mountain's shadow, with a soft edge `softness` degrees
//!   wide (the sun isn't a point). Found by looking from each spot toward the sun for ground that
//!   rises above the line to it, until that line is higher than the highest ground.
//! - **sky** (green): how much of the sky a spot can't see (0 on open ground, more in valleys and
//!   gorges), from the horizon's height in 8 directions out to `sky_reach`. Stored as its square
//!   root, so the small values (most of them) keep their precision.
//! - **relief** (blue): how far a spot is above or below the ground around it (the heights minus a
//!   blurred copy, `relief_reach` wide): 0.5 = level with its surroundings, 0 = 128 or more blocks
//!   below, 1 = 128 or more above (square-rooted the same way).
//! - **land** (alpha): 1 on land, 0 on the sea. The sea only takes the shadows.
//!
//! The strengths are applied in the shader, so those sliders cost nothing. The sun and the reaches
//! are worked out again in the background. The heights, sky and relief don't depend on the sun and
//! are kept, so moving the sun only redoes the shadows (a fraction of a second).
//!
//! **Left open for later** (see docs/worldgen.md, "Graphics: the light map"): the near chunks can
//! read a finer window of the same thing (step D), the distant tiles' hill shading can move in
//! here, and the day/night cycle only has to turn the sun.

use super::distant::{DistantMaterial, DistantState};
use crate::world::{CompiledMap, SEA_LEVEL, Terrain};
use crate::worldgen::runner::every_pixel;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// How many blocks above or below its surroundings counts as full relief.
const RELIEF_SCALE: f32 = 128.0;
/// Directions the sky is looked for in.
const SKY_DIRECTIONS: usize = 8;

/// Tunables (on the "Render" debug page, saved in "everything" presets).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LightSettings {
    pub on: bool,
    /// Where the sun is: the compass direction it shines from (degrees: 0 north, 90 east, 315
    /// north-west) and how high it stands (degrees above the horizon). The distant terrain's hill
    /// shading uses the same sun.
    pub sun_from: f32,
    pub sun_height: f32,
    /// How wide the shadows' soft edge is, in degrees of the sun's height.
    pub softness: f32,
    /// How dark a shadow is (0 = no shadows, 1 = black).
    pub shadows: f32,
    /// How much the valleys darken by the sky they can't see, and how far away a rise still
    /// hides sky (blocks).
    pub sky: f32,
    pub sky_reach: f32,
    /// How much being below your surroundings darkens (and above them lightens), and how wide
    /// "your surroundings" are (blocks).
    pub relief: f32,
    pub relief_reach: f32,
    /// Blocks per light map pixel.
    pub detail: f32,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            on: true,
            sun_from: 315.0,
            sun_height: 40.0,
            softness: 3.0,
            shadows: 0.45,
            sky: 2.0,
            sky_reach: 3_000.0,
            relief: 0.5,
            relief_reach: 2_000.0,
            detail: 32.0,
        }
    }
}

impl LightSettings {
    /// The sun's direction, game axes (x north, y east, z up), pointing toward it.
    pub fn sun(&self) -> Vec3 {
        let (from, up) = (self.sun_from.to_radians(), self.sun_height.clamp(5.0, 90.0).to_radians());
        Vec3::new(up.cos() * from.cos(), up.cos() * from.sin(), up.sin())
    }

    /// What the shader gets: x on, y shadows, z sky, w relief.
    fn for_shader(&self, ready: bool) -> Vec4 {
        Vec4::new(if self.on && ready { 1.0 } else { 0.0 }, self.shadows, self.sky, self.relief)
    }
}

/// The ground's heights and the parts of its light that don't depend on the sun.
struct Ground {
    width: usize,
    /// Blocks per pixel, and the map's half width in blocks (where pixel 0's corner is).
    detail: f32,
    half: f32,
    /// The surface's height (the sea's surface at sea), land or not, and the highest of them.
    heights: Vec<f32>,
    land: Vec<bool>,
    highest: f32,
    /// How much sky can't be seen (0 to 1), and the relief (blocks above the surroundings).
    sky: Vec<f32>,
    relief: Vec<f32>,
}

/// What a `Ground` was made from.
#[derive(Clone)]
struct GroundKey {
    map: Arc<CompiledMap>,
    detail: f32,
    sky_reach: f32,
    relief_reach: f32,
}

impl GroundKey {
    fn new(map: &Arc<CompiledMap>, settings: &LightSettings) -> Self {
        Self { map: map.clone(), detail: settings.detail, sky_reach: settings.sky_reach, relief_reach: settings.relief_reach }
    }

    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.map, &other.map) && self.detail == other.detail && self.sky_reach == other.sky_reach && self.relief_reach == other.relief_reach
    }
}

/// What the shadows were worked out for: sun from, sun height, softness.
type SunKey = [f32; 3];

fn sun_key(settings: &LightSettings) -> SunKey {
    [settings.sun_from, settings.sun_height, settings.softness]
}

/// A light map picture ready to show: its side, mip levels and pixels (every level, largest first).
struct Picture {
    width: u32,
    levels: u32,
    data: Vec<u8>,
}

#[derive(Resource, Default)]
pub struct LightState {
    ground: Option<(GroundKey, Arc<Ground>)>,
    ground_job: Option<(GroundKey, Arc<AtomicBool>, Task<Option<Ground>>, Instant)>,
    /// The sun and the ground the shadows being made are for.
    sun_job: Option<(SunKey, Arc<Ground>, Task<Picture>, Instant)>,
    /// The sun the shown picture was made for (with the ground in `ground`).
    shown: Option<SunKey>,
    /// For the Render page.
    pub status: String,
}

/// A 1 × 1 light map that changes nothing (for the material before the real one is made).
pub fn placeholder() -> Image {
    picture_image(Picture { width: 1, levels: 1, data: vec![255, 0, 128, 255] })
}

fn picture_image(picture: Picture) -> Image {
    let size = Extent3d { width: picture.width, height: picture.width, depth_or_array_layers: 1 };
    let first = (picture.width * picture.width * 4) as usize;
    // Only the full-size level is given here; the mip chain is attached just below.
    let mut image = Image::new(size, TextureDimension::D2, picture.data[..first].to_vec(), TextureFormat::Rgba8Unorm, RenderAssetUsages::RENDER_WORLD);
    image.data = Some(picture.data);
    image.texture_descriptor.mip_level_count = picture.levels;
    // Smooth: the light map is far coarser than the blocks it lights.
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

/// Keeps the light map up to date with the world and the settings, and hands it to the distant
/// terrain's material.
pub fn update_light(
    terrain: Res<Terrain>,
    settings: Res<LightSettings>,
    distant: Res<DistantState>,
    mut state: ResMut<LightState>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<DistantMaterial>>,
) {
    let Some(handle) = distant.material.clone() else { return };
    let map = match terrain.compiled() {
        Some(map) if settings.on => map.clone(),
        _ => {
            // Nothing to light (or switched off): the shader leaves the colours as they are.
            set_uniforms(&mut materials, &handle, settings.for_shader(false), Vec4::ZERO);
            return;
        }
    };
    let pool = AsyncComputeTaskPool::get();

    // The heights, sky and relief: made again (dropping any half-made ones) when the map or the
    // reaches change.
    let wanted = GroundKey::new(&map, &settings);
    if state.ground.as_ref().is_none_or(|(key, _)| !key.same(&wanted)) {
        if state.ground_job.as_ref().is_some_and(|(key, ..)| !key.same(&wanted)) {
            if let Some((_, stop, ..)) = state.ground_job.take() {
                stop.store(true, Ordering::Relaxed);
            }
        }
        if state.ground_job.is_none() {
            let stop = Arc::new(AtomicBool::new(false));
            let (key, flag) = (wanted.clone(), stop.clone());
            let task = pool.spawn(async move { make_ground(&key.map, key.detail, key.sky_reach, key.relief_reach, &flag) });
            state.ground_job = Some((wanted.clone(), stop, task, Instant::now()));
            state.status = "Light map: measuring the ground…".into();
        }
    }
    if let Some((key, _, task, started)) = state.ground_job.as_mut() {
        if let Some(made) = check_ready(task) {
            let (key, ms) = (key.clone(), started.elapsed().as_millis());
            state.ground_job = None;
            if let Some(ground) = made {
                state.status = format!("Light map: ground measured in {ms} ms ({0} × {0} pixels)", ground.width);
                state.ground = Some((key, Arc::new(ground)));
                state.shown = None;
            }
        }
    }
    let Some((_, ground)) = state.ground.clone() else { return };

    // The shadows: a running job is let finish (so dragging the sun shows something), then the
    // next starts with the newest sun.
    let sun = sun_key(&settings);
    if let Some((made_for, made_on, task, started)) = state.sun_job.as_mut() {
        if let Some(picture) = check_ready(task) {
            let (made_for, ms, current) = (*made_for, started.elapsed().as_millis(), Arc::ptr_eq(made_on, &ground));
            state.sun_job = None;
            // Shadows on a ground that's since been measured again are thrown away.
            if !current {
                return;
            }
            let image = images.add(picture_image(picture));
            if let Some(mut material) = materials.get_mut(&handle) {
                material.light = image;
            }
            state.shown = Some(made_for);
            state.status = format!("Light map: {0} × {0} pixels, shadows in {ms} ms", ground.width);
        }
    }
    if state.shown != Some(sun) && state.sun_job.is_none() {
        let (on, made) = (ground.clone(), settings.clone());
        let task = pool.spawn(async move { make_picture(&on, &made) });
        state.sun_job = Some((sun, ground.clone(), task, Instant::now()));
    }

    let area = Vec4::new(ground.half, ground.width as f32 * ground.detail, 0.0, 0.0);
    set_uniforms(&mut materials, &handle, settings.for_shader(state.shown.is_some()), area);
}

/// Sets the material's light uniforms, only when they change (touching a material re-sends it).
fn set_uniforms(materials: &mut Assets<DistantMaterial>, handle: &Handle<DistantMaterial>, light: Vec4, area: Vec4) {
    let Some(material) = materials.get(handle) else { return };
    if material.settings.light == light && (light.x == 0.0 || material.settings.light_area == area) {
        return;
    }
    if let Some(mut material) = materials.get_mut(handle) {
        material.settings.light = light;
        material.settings.light_area = area;
    }
}

// ── Working it out ──────────────────────────────────────────────────────────

/// The heights at each pixel's middle, then the sky and the relief. `None` if stopped.
fn make_ground(map: &CompiledMap, detail: f32, sky_reach: f32, relief_reach: f32, stop: &AtomicBool) -> Option<Ground> {
    let detail = detail.max(1.0);
    let half = map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0;
    let width = (2.0 * half / detail).ceil() as usize;
    let sea = SEA_LEVEL as f32;
    let raw = every_pixel(width, stop, |i| {
        let (east, north) = (((i % width) as f32 + 0.5) * detail - half, half - ((i / width) as f32 + 0.5) * detail);
        map.height(north, east).unwrap_or(0.0)
    })?;
    // Land where the distant terrain puts land (the ground rounds to sea level or above).
    let land: Vec<bool> = raw.iter().map(|&h| h >= sea - 0.5).collect();
    let heights: Vec<f32> = raw.iter().map(|&h| h.max(sea)).collect();
    let highest = heights.iter().copied().fold(sea, f32::max);
    let sky = every_pixel(width, stop, |i| sky_hidden(&heights, width, detail, sky_reach, i))?;
    let relief = relief(&heights, width, (relief_reach / detail).round().max(1.0) as usize);
    Some(Ground { width, detail, half, heights, land, highest, sky, relief })
}

/// The height between pixel middles (`c` columns east, `r` rows south), or `None` off the map.
fn sample(heights: &[f32], width: usize, c: f32, r: f32) -> Option<f32> {
    let last = (width - 1) as f32;
    if !(0.0..=last).contains(&c) || !(0.0..=last).contains(&r) {
        return None;
    }
    let (c0, r0) = (c.floor() as usize, r.floor() as usize);
    let (c1, r1) = ((c0 + 1).min(width - 1), (r0 + 1).min(width - 1));
    let (fc, fr) = (c - c0 as f32, r - r0 as f32);
    let at = |c: usize, r: usize| heights[r * width + c];
    let top = at(c0, r0) + (at(c1, r0) - at(c0, r0)) * fc;
    let bottom = at(c0, r1) + (at(c1, r1) - at(c0, r1)) * fc;
    Some(top + (bottom - top) * fr)
}

/// How much of the sky pixel `i` can't see: the average, over 8 directions, of how high the
/// horizon stands (the sine of its angle), looking out to `reach` blocks.
fn sky_hidden(heights: &[f32], width: usize, detail: f32, reach: f32, i: usize) -> f32 {
    let (c, r) = ((i % width) as f32, (i / width) as f32);
    let h0 = heights[i];
    let mut hidden = 0.0;
    for k in 0..SKY_DIRECTIONS {
        let angle = (k as f32 + 0.5) * TAU / SKY_DIRECTIONS as f32;
        let (dc, dr) = (angle.cos(), angle.sin());
        // The steepest rise seen (blocks up per block across); steps grow further out.
        let mut steepest = 0.0f32;
        let mut d = 1.0f32;
        while d * detail <= reach {
            let Some(h) = sample(heights, width, c + dc * d, r + dr * d) else { break };
            steepest = steepest.max((h - h0) / (d * detail));
            d = (d * 1.3).max(d + 1.0);
        }
        hidden += steepest / (1.0 + steepest * steepest).sqrt();
    }
    hidden / SKY_DIRECTIONS as f32
}

/// How far each pixel is above the ground around it: the heights minus a blur `radius` pixels
/// wide (two box blurs, which make a smooth tent).
fn relief(heights: &[f32], width: usize, radius: usize) -> Vec<f32> {
    let blurred = box_blur(&box_blur(heights, width, radius), width, radius);
    heights.iter().zip(&blurred).map(|(h, b)| h - b).collect()
}

/// The average over a square `radius` pixels each way (fewer pixels at the edges), along rows
/// then columns, from running sums.
fn box_blur(values: &[f32], width: usize, radius: usize) -> Vec<f32> {
    let pass = |values: &[f32], step: usize, along: usize| -> Vec<f32> {
        let mut out = vec![0.0; values.len()];
        let mut sums = vec![0.0f64; width + 1];
        for line in 0..width {
            let at = |k: usize| line * along + k * step;
            for k in 0..width {
                sums[k + 1] = sums[k] + values[at(k)] as f64;
            }
            for k in 0..width {
                let (a, b) = (k.saturating_sub(radius), (k + radius).min(width - 1));
                out[at(k)] = ((sums[b + 1] - sums[a]) / (b + 1 - a) as f64) as f32;
            }
        }
        out
    };
    let rows = pass(values, 1, width);
    pass(&rows, width, 1)
}

/// How much sun pixel `i` gets (0 to 1): the steepest rise toward the sun, against the sun's
/// height, with a soft edge `soft` radians wide.
fn sunlight(ground: &Ground, toward: (f32, f32), up: f32, soft: f32, i: usize) -> f32 {
    let width = ground.width;
    let (c, r) = ((i % width) as f32, (i / width) as f32);
    let h0 = ground.heights[i];
    // Past where the line to the bottom of the soft edge clears the highest ground, nothing can
    // shade this spot.
    let low = (up - soft / 2.0).max(0.5f32.to_radians()).tan();
    let mut steepest = f32::MIN;
    let mut d = 1.0f32;
    loop {
        let run = d * ground.detail;
        if h0 + run * low > ground.highest {
            break;
        }
        let Some(h) = sample(&ground.heights, width, c + toward.0 * d, r + toward.1 * d) else { break };
        steepest = steepest.max((h - h0) / run);
        d += (d / 64.0).max(1.0);
    }
    if steepest == f32::MIN {
        return 1.0;
    }
    let clear = up - steepest.atan();
    if soft <= 0.0 { if clear > 0.0 { 1.0 } else { 0.0 } } else { smoothstep(-soft / 2.0, soft / 2.0, clear) }
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The shadows for the settings' sun, packed with the rest into the picture (every mip level).
fn make_picture(ground: &Ground, settings: &LightSettings) -> Picture {
    let from = settings.sun_from.to_radians();
    // Toward the sun in pixels: columns run east, rows south.
    let toward = (from.sin(), -from.cos());
    let up = settings.sun_height.clamp(5.0, 90.0).to_radians();
    let soft = settings.softness.max(0.0).to_radians();
    let never = AtomicBool::new(false);
    let lit = every_pixel(ground.width, &never, |i| sunlight(ground, toward, up, soft, i)).unwrap_or_default();
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut data = Vec::with_capacity(ground.width * ground.width * 4 * 4 / 3 + 4);
    for i in 0..ground.width * ground.width {
        let r = (ground.relief[i] / RELIEF_SCALE).clamp(-1.0, 1.0);
        data.extend_from_slice(&[
            byte(lit[i]),
            byte(ground.sky[i].sqrt()),
            byte(0.5 + 0.5 * r.signum() * r.abs().sqrt()),
            if ground.land[i] { 255 } else { 0 },
        ]);
    }
    // The mip chain: each level the average of 2 × 2 pixels of the one before.
    let mut level = data.clone();
    let mut size = ground.width;
    let mut levels = 1;
    while size > 1 {
        level = shrink(&level, size);
        size = (size / 2).max(1);
        data.extend_from_slice(&level);
        levels += 1;
    }
    Picture { width: ground.width as u32, levels, data }
}

/// Halves an RGBA picture `size` pixels square (an odd last row or column is left out, as the
/// graphics card's own mip sizes do).
fn shrink(pixels: &[u8], size: usize) -> Vec<u8> {
    let half = (size / 2).max(1);
    let mut out = Vec::with_capacity(half * half * 4);
    for y in 0..half {
        for x in 0..half {
            for k in 0..4 {
                let at = |dx: usize, dy: usize| pixels[(((2 * y + dy).min(size - 1)) * size + (2 * x + dx).min(size - 1)) * 4 + k] as u32;
                out.push(((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1) + 2) / 4) as u8);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flat ground at sea level with a ridge running north–south down the middle, `tall` blocks
    /// high, 4 pixels wide.
    fn ridge(width: usize, tall: f32) -> Ground {
        let sea = SEA_LEVEL as f32;
        let heights: Vec<f32> = (0..width * width).map(|i| if (i % width).abs_diff(width / 2) < 2 { sea + tall } else { sea + 10.0 }).collect();
        let sky = (0..width * width).map(|i| sky_hidden(&heights, width, 32.0, 3_000.0, i)).collect();
        let relief = relief(&heights, width, 8);
        Ground { width, detail: 32.0, half: width as f32 * 16.0, land: vec![true; width * width], highest: sea + tall, heights, sky, relief }
    }

    #[test]
    fn a_ridge_shades_the_side_away_from_the_sun() {
        let ground = ridge(128, 200.0);
        let row = 64 * 128;
        // Sun low in the west (shining from 270°): the ground just east of the ridge is in shadow,
        // the ground just west of it and far east of it is lit.
        let settings = LightSettings { sun_from: 270.0, sun_height: 10.0, softness: 0.0, ..LightSettings::default() };
        let picture = make_picture(&ground, &settings);
        let sun = |c: usize| picture.data[(row + c) * 4];
        assert_eq!(sun(60), 255, "west of the ridge");
        assert_eq!(sun(68), 0, "just east of the ridge");
        // 190 blocks above the ground at a 10° sun casts about 1,080 blocks (34 pixels) of shadow.
        assert_eq!(sun(96), 0, "in the far end of the shadow");
        assert_eq!(sun(105), 255, "past the shadow");
        // A high sun casts hardly any.
        let noon = make_picture(&ground, &LightSettings { sun_from: 270.0, sun_height: 85.0, softness: 0.0, ..LightSettings::default() });
        assert_eq!(noon.data[(row + 68) * 4], 255);
    }

    #[test]
    fn valleys_see_less_sky_and_sit_below_their_surroundings() {
        let ground = ridge(64, 200.0);
        let row = 32 * 64;
        // Beside the ridge, the sky is half hidden one way; far from it, nothing is hidden.
        assert!(ground.sky[row + 34] > 0.1, "beside the ridge {}", ground.sky[row + 34]);
        assert!(ground.sky[row + 5] < ground.sky[row + 34] / 4.0, "far away {}", ground.sky[row + 5]);
        // The ridge stands above its surroundings, the ground beside it below them.
        assert!(ground.relief[row + 32] > 50.0 && ground.relief[row + 35] < 0.0);
    }

    #[test]
    fn the_picture_carries_every_mip_level() {
        let ground = ridge(64, 50.0);
        let picture = make_picture(&ground, &LightSettings::default());
        // 64² + 32² + … + 1² pixels, 4 bytes each.
        assert_eq!(picture.levels, 7);
        assert_eq!(picture.data.len(), (4096 + 1024 + 256 + 64 + 16 + 4 + 1) * 4);
    }
}

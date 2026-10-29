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
//! **The near chunks** read the same light map, plus a window of the ground around them for hill
//! shading (see "The near window" below), so near and far are lit alike, and its cone map, which
//! tells the shader which side of a step each block top is on (see "The cone map").
//!
//! **Left open for later** (see docs/worldgen.md, "Graphics: the light map"): the distant tiles'
//! hill shading can move in here, and the day/night cycle only has to turn the sun.

use super::distant::{DistantMaterial, DistantSettings, DistantState};
use super::material::{TerrainLight, TerrainMaterial};
use super::{RenderSettings, TerrainAssets};
use crate::coords::bevy_to_game;
use crate::world::{CHUNK_SIZE, CompiledMap, SEA_LEVEL, Terrain};
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
    /// Light the near chunks like the distant terrain (hill shading from a window of the ground
    /// around you, and the light map), and blocks per pixel of that window.
    pub near: bool,
    pub near_detail: f32,
    /// The broad lie of the land: the near chunks' hill shading reads the smooth ground's slope averaged
    /// over this many blocks, so wobbles too small for the blocks to show shade nothing.
    pub broad_reach: f32,
    /// The steps you can see are shaded from the cone map ("The cone map" below) [David,
    /// 2026-10-27]: how dark the foot of a step gets, all round it (`foot_shading`) and more on the
    /// side away from the sun (`step_shadows`), and how far the shade reaches out from a step for
    /// each block of its height (`shade_reach`, blocks). A step's top is never darkened.
    pub foot_shading: f32,
    pub step_shadows: f32,
    pub shade_reach: f32,
    /// How much of the lie of the land's sun and shade the steps' sides take, as the tops do (0 = only
    /// their fixed face shade, 1 = all of it, which stripes every step on a shaded slope). [2026-10-28]
    pub side_shading: f32,
    /// The farthest a step's shade reaches from its foot, however tall the step (blocks): steps are
    /// shaded, mountains cast the light map's shadows. [2026-10-28]
    pub step_reach: f32,
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
            near: true,
            near_detail: 2.0,
            broad_reach: 32.0,
            foot_shading: 0.2,
            step_shadows: 0.45,
            shade_reach: 4.0,
            side_shading: 0.5,
            step_reach: 16.0,
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
/// terrain's material and the near chunks'.
#[allow(clippy::too_many_arguments)]
pub fn update_light(
    terrain: Res<Terrain>,
    settings: Res<LightSettings>,
    distant: Res<DistantState>,
    near: Option<Res<TerrainAssets>>,
    mut state: ResMut<LightState>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<DistantMaterial>>,
    mut near_materials: ResMut<Assets<TerrainMaterial>>,
) {
    let _timed = crate::render::Timed::new("light map");
    let Some(handle) = distant.material.clone() else { return };
    let near_handle = near.map(|near| near.material.clone());
    let map = match terrain.compiled() {
        Some(map) if settings.on => map.clone(),
        _ => {
            // Nothing to light (or switched off): the shaders leave the colours as they are.
            set_uniforms(&mut materials, &handle, settings.for_shader(false), Vec4::ZERO);
            set_near_uniforms(&mut near_materials, near_handle.as_ref(), settings.for_shader(false), Vec4::ZERO);
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
                material.light = image.clone();
            }
            if let Some(mut material) = near_handle.as_ref().and_then(|near| near_materials.get_mut(near)) {
                material.light_map = image;
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
    let near_light = if settings.near { settings.for_shader(state.shown.is_some()) } else { Vec4::ZERO };
    set_near_uniforms(&mut near_materials, near_handle.as_ref(), near_light, area);
}

/// The same for the near chunks' material.
fn set_near_uniforms(materials: &mut Assets<TerrainMaterial>, handle: Option<&Handle<TerrainMaterial>>, light: Vec4, area: Vec4) {
    let Some(handle) = handle else { return };
    let Some(material) = materials.get(handle) else { return };
    if material.light.light == light && (light.x == 0.0 || material.light.light_area == area) {
        return;
    }
    if let Some(mut material) = materials.get_mut(handle) {
        material.light.light = light;
        material.light.light_area = area;
    }
}

// ── The near window ─────────────────────────────────────────────────────────
//
// The near chunks' tops are all flat (it's a block world), so lighting them by which way each face
// points can't show the lie of the land. Like the distant terrain, they're lit by the slope of the
// ground under them instead: a window of the ground around you, `near_detail` blocks a pixel, as
// wide as the render distance, holding each spot's slope and height. The shader works out the hill
// shading from the slope, with the same sun and the same sums as the distant tiles, so the two match
// at the hand-off and moving the sun costs nothing. The window is made again in the background when
// you've moved a quarter of the way to its edge (the old one shows until then).

/// Slopes up to this (blocks up per block across) are kept; steeper ones count as this.
const STEEPEST: f32 = 4.0;
/// The most pixels along the window's side.
const NEAR_MOST: f32 = 2048.0;
/// The height the near window's ground byte counts up to, from sea level: the top of the world (it
/// stopped at the old top, Z 400, so peaks above it read as Z 400; 2026-10-28).
const NEAR_TOP: f32 = crate::world::WORLD_HEIGHT as f32;
/// The height the height tint counts up to, near and far alike (the distant tiles measure against
/// `HIGHEST`; the near chunks measured against Z 400, so the peaks changed brightness where the two
/// met: David's shading mismatch, 2026-10-28).
const TINT_TOP: f32 = crate::world::HIGHEST;

/// What a near window was made for: the map, its north-west corner (north, east), its side
/// (blocks) and blocks per pixel.
#[derive(Clone)]
struct NearKey {
    map: Arc<CompiledMap>,
    corner: Vec2,
    side: f32,
    detail: f32,
    /// How its slopes and cones are made (see `LightSettings`): broad reach and shade reach (blocks),
    /// and toward the sun in pixels (columns east, rows south).
    broad_reach: f32,
    shade_reach: f32,
    step_reach: f32,
    toward_sun: (f32, f32),
}

#[derive(Resource, Default)]
pub struct NearLightState {
    /// The window showing, and the one being made (with its stop switch).
    shown: Option<NearKey>,
    job: Option<(NearKey, Arc<AtomicBool>, Task<Option<NearMade>>)>,
    /// The ground the last window read, for the next one to slide over (see `make_near`).
    heights: Option<(NearKey, Arc<Vec<f32>>)>,
}

/// Keeps the window of the ground around you up to date, and the near chunks' sun and hill shading
/// the same as the distant terrain's.
#[allow(clippy::too_many_arguments)]
pub fn update_near_light(
    terrain: Res<Terrain>,
    settings: Res<LightSettings>,
    distant: Res<DistantSettings>,
    render: Res<RenderSettings>,
    near: Option<Res<TerrainAssets>>,
    camera: Query<&Transform, With<Camera3d>>,
    mut state: ResMut<NearLightState>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let _timed = crate::render::Timed::new("near light");
    let Some(handle) = near.map(|near| near.material.clone()) else { return };
    let sea = SEA_LEVEL as f32;
    let mut wanted = TerrainLight {
        sun: settings.sun().extend(distant.hill_shading),
        hill: Vec4::new(distant.height_tint, distant.finest_cell as f32, sea, NEAR_TOP),
        shade: Vec4::new(distant.shading_fade, settings.foot_shading, settings.step_shadows, TINT_TOP),
        sides: Vec4::new(settings.side_shading, 0.0, 0.0, 0.0),
        ..default()
    };
    let map = terrain.compiled().filter(|_| settings.near).cloned();
    if let (Some(map), Some(camera)) = (map, camera.iter().next()) {
        // Wide enough for the render distance (and a chunk more), pixels permitting.
        let side = ((render.render_distance_chunks * 2 + 2) * CHUNK_SIZE) as f32;
        let detail = settings.near_detail.max(side / NEAR_MOST).max(0.5);
        let at = bevy_to_game(camera.translation).truncate();
        // Toward the sun in the window's pixels (columns run east, rows south), to a hundredth, so the
        // sun-side cones are made again only when the sun really moves.
        let from = settings.sun_from.to_radians();
        let toward_sun = ((from.sin() * 100.0).round() / 100.0, (-from.cos() * 100.0).round() / 100.0);
        let fits = |key: &NearKey| {
            let middle = key.corner + Vec2::new(-key.side, key.side) / 2.0;
            Arc::ptr_eq(&key.map, &map) && key.side == side && key.detail == detail && key.broad_reach == settings.broad_reach && key.shade_reach == settings.shade_reach && key.step_reach == settings.step_reach && key.toward_sun == toward_sun && (at - middle).abs().max_element() < side / 8.0
        };
        if !state.shown.as_ref().is_some_and(fits) && !state.job.as_ref().is_some_and(|(key, ..)| fits(key)) {
            if let Some((_, stop, _)) = state.job.take() {
                stop.store(true, Ordering::Relaxed);
            }
            // Centred on you, snapped to whole pixels.
            let snap = |v: f32| (v / detail).round() * detail;
            let key = NearKey { map: map.clone(), corner: Vec2::new(snap(at.x + side / 2.0), snap(at.y - side / 2.0)), side, detail, broad_reach: settings.broad_reach, shade_reach: settings.shade_reach, step_reach: settings.step_reach, toward_sun };
            let (made, stop) = (key.clone(), Arc::new(AtomicBool::new(false)));
            let flag = stop.clone();
            let previous = state.heights.clone();
            let task = AsyncComputeTaskPool::get().spawn(async move { make_near(&made, previous, &flag) });
            state.job = Some((key, stop, task));
        }
        if let Some((key, _, task)) = state.job.as_mut() {
            if let Some(made) = check_ready(task) {
                let key = key.clone();
                state.job = None;
                if let Some((picture, cones, heights)) = made {
                    state.heights = Some((key.clone(), heights));
                    let (image, cones) = (images.add(picture_image(picture)), images.add(cones_image(cones)));
                    if let Some(mut material) = materials.get_mut(&handle) {
                        material.near_map = image;
                        material.near_cones = cones;
                    }
                    state.shown = Some(key);
                }
            }
        }
        if let Some(key) = &state.shown {
            wanted.near_area = Vec4::new(key.corner.x, key.corner.y, key.side, 1.0);
        }
    }
    let Some(material) = materials.get(&handle) else { return };
    let now = material.light;
    if (now.sun, now.hill, now.shade, now.sides, now.near_area) != (wanted.sun, wanted.hill, wanted.shade, wanted.sides, wanted.near_area) {
        if let Some(mut material) = materials.get_mut(&handle) {
            material.light.sun = wanted.sun;
            material.light.hill = wanted.hill;
            material.light.shade = wanted.shade;
            material.light.sides = wanted.sides;
            material.light.near_area = wanted.near_area;
        }
    }
}

/// The window's picture: red and green the slope rising north and east (blocks up per block
/// across, 0.5 = flat, square-rooted each way out to `STEEPEST`): the broad lie of the land, the
/// smooth ground's slope averaged over the broad reach; blue the surface's height (sea level to
/// the top of the world); alpha 1 on the map, 0 off it. The sea's surface is flat. And its cone
/// map. `None` if stopped.
/// The window round you: its picture and cone maps, and the ground it read (kept for the next one).
///
/// **It slides** [2026-10-28]: the ground the last window read (`previous`) is copied across where
/// the two overlap (same map and pixel size; windows are snapped to whole pixels, so they line up),
/// and only the new strip is read from the recipe. It used to read all of it, about 4 million
/// readings at render distance 64, twice a second while flying fast, on every core: Windows paused
/// the game's own threads to make room, and frames took 50-250 ms. It also keeps to a quarter of
/// the cores now.
type NearMade = (Picture, Cones, Arc<Vec<f32>>);

fn make_near(key: &NearKey, previous: Option<(NearKey, Arc<Vec<f32>>)>, stop: &AtomicBool) -> Option<NearMade> {
    let width = (key.side / key.detail).round() as usize;
    let sea = SEA_LEVEL as f32;
    // Where the last window's pixels are in this one's (rows run south, columns east).
    let reuse = previous.filter(|(old, heights)| Arc::ptr_eq(&old.map, &key.map) && old.detail == key.detail && heights.len() == width * width).map(|(old, heights)| {
        let rows = ((old.corner.x - key.corner.x) / key.detail).round() as i64;
        let columns = ((key.corner.y - old.corner.y) / key.detail).round() as i64;
        (heights, rows, columns)
    });
    let threads = (std::thread::available_parallelism().map_or(4, |n| n.get()) / 4).max(2);
    let raw = crate::worldgen::runner::every_pixel_on(threads, width, stop, |i| {
        let (r, c) = ((i / width) as i64, (i % width) as i64);
        if let Some((heights, rows, columns)) = &reuse {
            let (or, oc) = (r + rows, c + columns);
            if (0..width as i64).contains(&or) && (0..width as i64).contains(&oc) {
                return heights[or as usize * width + oc as usize];
            }
        }
        let (north, east) = (key.corner.x - (r as f32 + 0.5) * key.detail, key.corner.y + (c as f32 + 0.5) * key.detail);
        key.map.height(north, east).map_or(f32::NAN, |h| h.max(sea))
    })?;
    let at = |c: usize, r: usize| {
        let h = raw[r.min(width - 1) * width + c.min(width - 1)];
        if h.is_nan() { sea } else { h }
    };
    let slope = |g: f32| {
        let g = (g / STEEPEST).clamp(-1.0, 1.0);
        0.5 + 0.5 * g.signum() * g.abs().sqrt()
    };
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    // The broad lie of the land (rows run south, so the row above is to the north).
    let reach = (key.broad_reach / key.detail / 2.0).round() as usize;
    let (mut north, mut east) = (vec![0.0f32; width * width], vec![0.0f32; width * width]);
    for r in 0..width {
        for c in 0..width {
            north[r * width + c] = (at(c, r.saturating_sub(1)) - at(c, r + 1)) / (2.0 * key.detail);
            east[r * width + c] = (at(c + 1, r) - at(c.saturating_sub(1), r)) / (2.0 * key.detail);
        }
    }
    crate::worldgen::compiler::blur(&mut north, width, reach);
    crate::worldgen::compiler::blur(&mut east, width, reach);
    let mut data = Vec::with_capacity(width * width * 4);
    for r in 0..width {
        for c in 0..width {
            let i = r * width + c;
            data.extend_from_slice(&[byte(slope(north[i])), byte(slope(east[i])), byte((at(c, r) - sea) / (NEAR_TOP - sea)), if raw[i].is_nan() { 0 } else { 255 }]);
        }
    }
    // The tops of the blocks, as the world has them: a land block's top face is one above its Z;
    // the sea's surface is at sea level.
    let tops: Vec<f32> = raw.iter().map(|&h| if h.is_nan() || h < sea - 0.5 { sea } else { h.round() + 1.0 }).collect();
    let fall = key.detail / key.shade_reach.max(0.1);
    // A step's shade reaches at most `step_reach` blocks from its foot, however tall the step is.
    let cap = key.step_reach / key.shade_reach.max(0.1);
    let cones = Cones { width: width as u32, all_round: cones(&tops, width, fall, None, cap), from_sun: cones(&tops, width, fall, Some(key.toward_sun), cap) };
    Some((Picture { width: width as u32, levels: 1, data }, cones, Arc::new(raw)))
}

// ── The cone map ────────────────────────────────────────────────────────────
//
// How the shading knows which side of a step it's on [David, 2026-10-27]. Every block top casts a
// cone down round itself: its height, less `fall` for each pixel of distance. A pixel of the cone
// map holds the highest cone over it (its own block counts, at no distance). The shader draws a
// block top at an exact height, and compares: at or above the cone map, nothing near stands above
// it (a top, a rim, open ground), so it isn't darkened; below it, higher ground is near, and the
// amount below says how much higher and how close, so the foot of a step darkens most against the
// face and fades out, further from a taller step. The map is blended between pixels, which only
// softens how strongly a foot is darkened: a top's own height is at least the blend round it, so
// it is never darkened, right up to its edge.
//
// Two maps: `all_round` (the foot of every step), and `from_sun`, built only from ground on the
// sun's side (the feet in a step's sun shadow, roughly). Heights are kept as plain numbers (32-bit),
// exact to far under a block, so the shader blends them itself.

/// The cone map for the near window: per pixel, the highest cone over it, all round and from the
/// sun's side (blocks, top-face heights).
struct Cones {
    width: u32,
    all_round: Vec<f32>,
    from_sun: Vec<f32>,
}

/// The highest cone over each pixel of `tops`: each top less `fall` per pixel of distance (√2 on
/// the diagonal), from every side, or with `toward_sun`, only from ground on the sun's side, and never
/// more than `cap` above a pixel's own top, so a step's shade reaches at most `cap / fall` pixels from its
/// foot however tall it is [2026-10-28: a mountain's cone reached a thousand blocks, painting a huge
/// "step shadow" across its lee that the far terrain doesn't have; mountain shadows are the light map's]
/// (columns run east, rows south).
fn cones(tops: &[f32], width: usize, fall: f32, toward_sun: Option<(f32, f32)>, cap: f32) -> Vec<f32> {
    let mut cone = tops.to_vec();
    let diagonal = fall * std::f32::consts::SQRT_2;
    let step = |cone: &mut Vec<f32>, c: usize, r: usize, offsets: &[(isize, isize)]| {
        let i = r * width + c;
        for &(dc, dr) in offsets {
            let (c2, r2) = (c as isize + dc, r as isize + dr);
            if c2 < 0 || r2 < 0 || c2 as usize >= width || r2 as usize >= width {
                continue;
            }
            let drop = if dc != 0 && dr != 0 { diagonal } else { fall };
            cone[i] = cone[i].max(cone[r2 as usize * width + c2 as usize] - drop).min(tops[i] + cap);
        }
    };
    match toward_sun {
        None => {
            // Two sweeps: down and across from the top left, then back from the bottom right, each
            // pixel taking the cones of the neighbours already done.
            for r in 0..width {
                for c in 0..width {
                    step(&mut cone, c, r, &[(-1, 0), (-1, -1), (0, -1), (1, -1)]);
                }
            }
            for r in (0..width).rev() {
                for c in (0..width).rev() {
                    step(&mut cone, c, r, &[(1, 0), (1, 1), (0, 1), (-1, 1)]);
                }
            }
        }
        Some(toward) => {
            // Only the neighbours on the sun's side; pixels taken sun side first, so those are
            // always done already.
            let offsets: Vec<(isize, isize)> = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
                .into_iter()
                .filter(|&(dc, dr): &(isize, isize)| {
                    let length = ((dc * dc + dr * dr) as f32).sqrt();
                    (dc as f32 * toward.0 + dr as f32 * toward.1) / length > 0.3
                })
                .collect();
            let mut order: Vec<usize> = (0..width * width).collect();
            let along = |i: usize| (i % width) as f32 * toward.0 + (i / width) as f32 * toward.1;
            order.sort_by(|&a, &b| along(b).total_cmp(&along(a)));
            for i in order {
                step(&mut cone, i % width, i / width, &offsets);
            }
        }
    }
    cone
}

/// The cone map as an image: two 32-bit numbers a pixel (all round, from the sun), read unblended.
fn cones_image(cones: Cones) -> Image {
    let size = Extent3d { width: cones.width, height: cones.width, depth_or_array_layers: 1 };
    let data: Vec<u8> = cones.all_round.iter().zip(&cones.from_sun).flat_map(|(a, s)| a.to_le_bytes().into_iter().chain(s.to_le_bytes())).collect();
    Image::new(size, TextureDimension::D2, data, TextureFormat::Rg32Float, RenderAssetUsages::RENDER_WORLD)
}

/// An empty cone map (nothing near stands above anything), until the first window is made.
pub fn cones_placeholder() -> Image {
    cones_image(Cones { width: 1, all_round: vec![0.0], from_sun: vec![0.0] })
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
    fn a_window_slid_over_the_last_one_is_the_same_as_one_read_afresh() {
        // Bumpy ground; a window, then one moved 40 blocks north and 24 west (whole pixels).
        let width = 64;
        let z: Vec<f32> = (0..width * width).map(|i| 150.0 + ((i * 37) % 23) as f32 + (i / width) as f32).collect();
        let map = Arc::new(CompiledMap::of_heights(width, 32.0, &z));
        let first = NearKey { map: map.clone(), corner: Vec2::new(256.0, -256.0), side: 512.0, detail: 4.0, broad_reach: 32.0, shade_reach: 4.0, step_reach: 1_000.0, toward_sun: (1.0, 0.0) };
        let moved = NearKey { corner: Vec2::new(296.0, -280.0), ..first.clone() };
        let never = AtomicBool::new(false);
        let (_, _, heights) = make_near(&first, None, &never).unwrap();
        let (slid, slid_cones, slid_heights) = make_near(&moved, Some((first, heights)), &never).unwrap();
        let (fresh, fresh_cones, fresh_heights) = make_near(&moved, None, &never).unwrap();
        assert!(slid_heights.iter().zip(fresh_heights.iter()).all(|(a, b)| a == b || (a.is_nan() && b.is_nan())));
        assert_eq!(slid.data, fresh.data);
        assert_eq!((slid_cones.all_round, slid_cones.from_sun), (fresh_cones.all_round, fresh_cones.from_sun));
    }

    #[test]
    fn the_near_window_holds_the_slope_and_height() {
        // Ground rising 2 blocks every 32 east, level north to south.
        let width = 64;
        let z: Vec<f32> = (0..width * width).map(|i| 150.0 + (i % width) as f32 * 2.0).collect();
        let map = Arc::new(CompiledMap::of_heights(width, 32.0, &z));
        let key = NearKey { map, corner: Vec2::new(256.0, -256.0), side: 512.0, detail: 4.0, broad_reach: 32.0, shade_reach: 4.0, step_reach: 1_000.0, toward_sun: (1.0, 0.0) };
        let (picture, _, _) = make_near(&key, None, &AtomicBool::new(false)).unwrap();
        assert_eq!(picture.width, 128);
        let pixel = |c: usize, r: usize| &picture.data[(r * 128 + c) * 4..(r * 128 + c) * 4 + 4];
        let middle = pixel(64, 64);
        // Rising east 1/16 of a block per block: 0.5 + 0.5 × √(0.0625 / 4) ≈ 0.56.
        assert!((138..150).contains(&middle[1]), "east {}", middle[1]);
        assert!((middle[0] as i32 - 128).abs() <= 2, "north {}", middle[0]);
        assert_eq!(middle[3], 255);
        // Higher to the east.
        assert!(pixel(120, 64)[2] > pixel(8, 64)[2]);
    }

    /// Tops in two halves: `low` west of column 32, `high` from it east (a step running north to
    /// south), and their cone maps, all round and from the sun toward `toward`.
    fn step_cones(low: f32, high: f32, fall: f32, toward: (f32, f32)) -> (Vec<f32>, Vec<f32>) {
        let width = 64;
        let tops: Vec<f32> = (0..width * width).map(|i| if i % width < 32 { low } else { high }).collect();
        (cones(&tops, width, fall, None, f32::MAX), cones(&tops, width, fall, Some(toward), f32::MAX))
    }

    #[test]
    fn a_tall_step_s_shade_reaches_no_further_than_a_short_one_s_cap() {
        // A 250-block wall at column 10 on flat ground at 150, a block a pixel, falling a quarter of a
        // block a pixel, capped at 4 blocks above the ground: its shade is gone 16 pixels out, not
        // the thousand it would reach uncapped.
        let width = 64;
        let tops: Vec<f32> = (0..width * width).map(|i| if i % width == 10 { 400.0 } else { 150.0 }).collect();
        let capped = cones(&tops, width, 0.25, None, 4.0);
        let uncapped = cones(&tops, width, 0.25, None, f32::MAX);
        let at = |cone: &[f32], c: usize| cone[30 * width + c];
        assert!(at(&capped, 12) > 153.0, "{}", at(&capped, 12));
        assert_eq!(at(&capped, 40), 150.0);
        assert!(at(&uncapped, 40) > 390.0);
        // The wall's own top is untouched.
        assert_eq!(at(&capped, 10), 400.0);
    }

    #[test]
    fn a_step_shades_its_foot_fading_out_and_never_its_top() {
        // A 1-block step, a cone falling half a block a pixel: the foot is shaded 2 pixels out.
        let (all_round, _) = step_cones(10.0, 11.0, 0.5, (1.0, 0.0));
        let row = |c: usize| all_round[20 * 64 + c];
        // The top: exactly its own height, right up to the edge, so it's never below its cones.
        for c in 32..64 {
            assert_eq!(row(c), 11.0, "top at column {c}");
        }
        // The foot: below the cones by less and less, then not at all.
        let below: Vec<f32> = (26..32).map(|c| row(c) - 10.0).collect();
        assert_eq!(below, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.5], "foot, columns 26 to 31");
        // Blended between pixels, the top still never comes out below: a top's own pixel holds its
        // height, and its neighbours never hold more.
        let blend = (row(31) + row(32)) / 2.0;
        assert!(blend <= 11.0);
        // A taller step shades further: 4 blocks, 8 pixels.
        let (tall, _) = step_cones(10.0, 14.0, 0.5, (1.0, 0.0));
        assert!(tall[20 * 64 + 25] > 10.0 && tall[20 * 64 + 23] == 10.0);
    }

    #[test]
    fn step_shadows_fall_only_on_the_side_away_from_the_sun() {
        // The step rises to the east. With the sun in the east, its foot (to the west) is in its
        // shadow; with the sun in the west, nothing is.
        let (_, sun_east) = step_cones(10.0, 11.0, 0.5, (1.0, 0.0));
        let (_, sun_west) = step_cones(10.0, 11.0, 0.5, (-1.0, 0.0));
        assert!(sun_east[20 * 64 + 31] > 10.0);
        assert!((0..64 * 64).all(|i| sun_west[i] == if i % 64 < 32 { 10.0 } else { 11.0 }));
    }

    #[test]
    fn the_window_makes_its_cone_map_from_the_block_tops() {
        // Level ground, Z 150 (blocks' tops at 151): the cone map is that everywhere.
        let width = 64;
        let key = NearKey { map: Arc::new(CompiledMap::of_heights(width, 32.0, &vec![150.0; width * width])), corner: Vec2::new(256.0, -256.0), side: 512.0, detail: 4.0, broad_reach: 32.0, shade_reach: 4.0, step_reach: 1_000.0, toward_sun: (1.0, 0.0) };
        let (_, cones, _) = make_near(&key, None, &AtomicBool::new(false)).unwrap();
        assert_eq!(cones.width, 128);
        assert!(cones.all_round.iter().chain(&cones.from_sun).all(|&c| c == 151.0));
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

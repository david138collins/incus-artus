//! The far eye (plan 2.2, step 2a): old's far march on bricks, drawn each frame under the panels.
//!
//! The frame's card passes, each timed on its own: the start pass (one cone a tile of 8 × 8 screen pixels, finding
//! where its rays can start), the pyramid's layers (8 × 8, 4 × 4 and 2 × 2: each finer one only where the one above
//! says it's worth it, never for sky), then bringing it up to the screen. Old's settings for the dyno, carried as they
//! were: a field of view of 100° (up and down), the sun from 315° at 40° (noon), fog on, the hand-over from 0.8, the
//! finer layers' fades as old's preset had them, the layers brought up crisp; the sun's shadows and the sky light read from the cell (`far_light.rs`).
//!
//! The far cells come from the loading lane (`far::Lane`), nearest the eye first, and go on the card as they come
//! (`bricks::Store`), so the view sharpens in live, raw, with no loading screen [David, 2026-11-06: "id like to see the
//! sharpening myself, realtime and raw"].

use crate::atlas::Atlas;
use crate::bricks::Store;
use crate::camera::Camera;
use crate::far::Lane;
use std::sync::mpsc;
use std::time::Instant;

/// The pyramid's layers, coarsest first: each layer's pixel is this many screen pixels a side.
pub const LAYERS: [f32; 3] = [8.0, 4.0, 2.0];
/// A start tile's side, in screen pixels.
const TILE: u32 = 8;
/// The farthest a ray goes (blocks).
const REACH: f32 = 150_000.0;
/// The field of view, up and down (degrees): old's dyno camera's.
pub const FIELD_OF_VIEW: f32 = 100.0;
/// The sky's share of a face's light by day (old's).
const AMBIENT: f32 = 0.35;
/// Sky light: strength and tint (old's).
const SKY_STRENGTH: f32 = 0.8;
const SKY_TINT: f32 = 0.3;
/// Fog: land this far off at sea level is nineteen parts in twenty fog; the air's thickness halves every this many blocks
/// up; a little warmer towards the sun (old's).
const FOG_REACH: f32 = 3200.0;
const FOG_HEIGHT: f32 = 200.0;
const FOG_WARMTH: f32 = 0.3;
/// The sky's weight at a skyline: David's [build 124, the slider at its top, 4: "that actually works great", "i maxed it"].
const SKY_WEIGHT: f32 = 4.0;
/// Where in a level's stretch its blend with the next begins (old's since build 109).
const BLEND_FROM: f32 = 0.8;
/// Night's light (old's).
const NIGHT_LIGHT: f32 = 0.04;
/// The sun: from 315° (north toward east), 40° up (old's dyno light).
const SUN_FROM: f32 = 315.0;
const SUN_HEIGHT: f32 = 40.0;
/// The finer layer's fade in past the blocks, by the seam it makes: 4 × 4 into 8 × 8, then 2 × 2 into 4 × 4 (where it
/// starts, over how many doublings: old's dyno preset).
const CELLS_FADES: [[f32; 2]; 2] = [[0.25, 0.6], [1.1, 1.15]];
/// The world's water as a medium (old's: its colour, seen through 16 blocks).
const WATER: [u8; 3] = [63, 111, 149];
const WATER_CLARITY: f32 = 16.0;
/// The most a frame spends putting regions on the card (ms), so the view sharpens in without holding the frame up.
const PUTTING_MS: f32 = 3.0;
/// Numbers in the march's camera (`far.wgsl`'s `Camera`): rows of four.
const CAMERA_ROWS: usize = 23;
/// The most of a frame the ring's sending takes (ms): its maker works off the frame, the frame only hands its bricks on.
const RING_MS: f32 = 0.3;

/// The far eye's look as David tunes it on the Render page (F3), each with its default.
#[derive(Clone, Copy, Debug, PartialEq)]
///
/// David, 2026-11-12: "please expose any interesting or valuable parameters that may be worth adjusting live, with
/// default switches so i can get back to normal."
pub struct FarLook {
    /// Where in a level's stretch its blend with the next begins (old's 0.8: later is a narrower hand-over).
    pub hand_over: f32,
    /// Where the 4 x 4 fades in over the 8 x 8 (its pixel so many times the cells it reads), and over how many
    /// doublings of distance; the same for the 2 x 2 over the 4 x 4.
    pub fade_4: [f32; 2],
    pub fade_2: [f32; 2],
    /// The sky's weight over land at a skyline (0: as old's; 4 David's, build 124: "i maxed it").
    pub sky_weight: f32,
    /// How the far cells are blended (change 1): 0 by the code (old's), 1 by the texture hardware (the spread way: David
    /// saw no difference from exact, build 128, "as far as the far blend, they all look the same").
    pub blend: u32,
    /// Detail: the level read at each distance, so many levels finer (more) or coarser (less) than its pixel's size
    /// (old's level bias, turned round). 0, old's: +2 cost up to 77% more looking down from height (build 129, the 2 x 2
    /// earned), and David, looking from up high at build 130: "yeah 0 is fine".
    pub detail: i32,
    /// How the pyramid is brought up to the screen: 0 crisp, 1 smooth, 2 smooth with sharp skylines.
    pub bring_up: u32,
    /// What's shown: 0 the picture, 1 the steps' view, 2 the levels' view.
    pub view: u32,
    /// The sun: from (degrees from north toward east) and how high (degrees); the sky's share of a face's light.
    pub sun_from: f32,
    pub sun_height: f32,
    pub ambient: f32,
    /// The sun's shadows and the sky light, on or off, and the sky light's strength and tint.
    pub shadows: bool,
    pub sky_light: bool,
    pub sky_strength: f32,
    pub sky_tint: f32,
    /// Where each ray's steps begin within a step (the page's "Ray start"; David, 2026-11-14): the pattern (0
    /// scattered by chance, as it was; 1 woven, an even pattern; 2 blue noise; 3 woven with blue noise in it), the
    /// weave's side as a power of two, how much blue noise goes into the weave, and how far the starts spread (of a
    /// step: at 0 every ray begins alike, and the rings the scatter was for come back).
    pub start_pattern: u32,
    pub weave_bits: u32,
    pub start_blue: f32,
    pub start_spread: f32,
    /// Fog: on or off, how far off land at sea level is nineteen parts in twenty fog, how fast it thins going up (its
    /// thickness halves every so many blocks), and how much warmer it is towards the sun.
    pub fog: bool,
    pub fog_reach: f32,
    pub fog_height: f32,
    pub fog_warmth: f32,
}

impl FarLook {
    /// Old's settings as the flights flew them, and David's sky weight.
    pub const DEFAULT: FarLook = FarLook {
        hand_over: BLEND_FROM,
        fade_4: CELLS_FADES[0],
        fade_2: CELLS_FADES[1],
        sky_weight: SKY_WEIGHT,
        blend: 1,
        detail: 0,
        bring_up: 0,
        view: 0,
        sun_from: SUN_FROM,
        sun_height: SUN_HEIGHT,
        ambient: AMBIENT,
        shadows: true,
        sky_light: true,
        sky_strength: SKY_STRENGTH,
        sky_tint: SKY_TINT,
        start_pattern: 0,
        weave_bits: 2,
        start_blue: 0.25,
        start_spread: 1.0,
        // Off by default [David, 2026-11-13: "off by default would be better. it looks nice but is most likely reserved
        // for final touches"; "we are not using fog or haze or any sort of cover as a performance tool. it is just for
        // looks"]: the far eye is judged and measured without it.
        fog: false,
        fog_reach: FOG_REACH,
        fog_height: FOG_HEIGHT,
        fog_warmth: FOG_WARMTH,
    };
}

/// A layer of the pyramid on the card.
struct Layer {
    factor: f32,
    size: (u32, u32),
    picture: wgpu::TextureView,
    /// Its distances: its own texture, twice the same one; or by the one dispatch, the two textures all the layers'
    /// distances are kept in side by side (each frame writes one, led by the other, the frame before's), and where
    /// its part starts in them.
    far: [wgpu::TextureView; 2],
    far_at: u32,
    camera: wgpu::Buffer,
    /// Its bind group marched alone (the layers one after another; and the start pass's, the base's).
    group: wgpu::BindGroup,
}

/// What's sized to the screen (made again when the screen's size changes, or the bricks' pool grows).
struct Targets {
    screen: (u32, u32),
    generation: u32,
    /// Whether the atlas was in when they were made, and its generation (bumped when it grows).
    atlas_in: bool,
    atlas_generation: u32,
    /// The far light's shadow shown when they were made (none: not made yet).
    light: Option<u32>,
    tiles: (u32, u32),
    /// The longest ray's counting (step 0): a bind group a layer, its steps and its counts.
    tally_groups: Vec<[wgpu::BindGroup; 2]>,
    /// Each layer's part of its distances' texture, for the counting.
    tally_parts: Vec<wgpu::Buffer>,
    layers: Vec<Layer>,
    up_numbers: wgpu::Buffer,
    up_group: [wgpu::BindGroup; 2],
    /// The one-dispatch form's: every layer's numbers in one buffer, and its bind group for each of the two copies.
    one: Option<(wgpu::Buffer, [wgpu::BindGroup; 2])>,
    /// The audit's own bind group (`FarEye::audit`): the far eye's, with scratch in place of everything it writes.
    audit: Option<wgpu::BindGroup>,
}

pub struct FarEye {
    pub store: Store,
    lane: Option<Lane>,
    /// The rings round the eye (`far_ring.rs`), the 16s' first, then the 8s' (as many as `INCUS_FAR_RING` says: 0
    /// none), and where the eye last was.
    rings: Vec<crate::far_ring::Ring>,
    eye: [f32; 2],
    start: wgpu::ComputePipeline,
    march: wgpu::ComputePipeline,
    up: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    up_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    targets: Option<Targets>,
    /// Its look, as tuned (the Render page).
    pub look: FarLook,
    /// Change 1: whether the far textures are made (`INCUS_FAR_BLEND`), the atlas once it's in, and its builder's answer
    /// on the way; textures of one texel standing in till then.
    textures: bool,
    atlas: Option<Atlas>,
    building: Option<mpsc::Receiver<Atlas>>,
    stand_ins: [wgpu::TextureView; 2],
    /// The far light read from the cell (`far_light.rs`): made on a thread of its own once every region is in; a shadow
    /// swept on another when the sun's setting has changed and stood still a moment, and shown whole when it's in
    /// (David's snap). Stand-ins of one texel till it's made.
    light: Option<crate::far_light::Light>,
    lighting: Option<mpsc::Receiver<crate::far_light::Light>>,
    sweeping: Option<mpsc::Receiver<([f32; 2], f32)>>,
    sun_asked: ([f32; 2], Instant),
    light_stand_ins: [wgpu::TextureView; 2],
    /// The blue noise (`blue_noise.rs`) on the card, and its numbers till they're sent (at the first putting).
    blue: (wgpu::Texture, wgpu::TextureView),
    blue_bytes: Option<Vec<u8>>,
    /// Whether rays end at the map's edge (`INCUS_FAR_EDGE=0`: marched on toward the reach, as before), and the Oracle's
    /// test (2026-11-12), for measuring: the most steps a ray takes (`INCUS_TEST_STEPS`; 768).
    edge: bool,
    /// The layers' sizes in screen pixels, each half the one above (`INCUS_FAR_BASE`: the first, 8 as old's or 4:
    /// step 2's probe).
    layers: [f32; 3],
    /// The one dispatch (step 2; how the far eye marches unless `INCUS_FAR_SIDE=0`): all three layers' pixels marched
    /// at once, a finer layer led by the coarser layer of the frame before. Its pipeline and its bind group's layout;
    /// which of the two copies this frame writes; and whether the layers were remade this frame (its lead is then
    /// made in the frame itself: the dispatch run three times).
    one: Option<(wgpu::ComputePipeline, wgpu::BindGroupLayout)>,
    /// The march once the textures are in (`far.wgsl`'s `LOADED`, true): the one dispatch's second program, without
    /// the code's blend or the views, which the far eye marches by when `loaded` says so; `one`'s is then the whole
    /// march (`march_loading`). `INCUS_FAR_LOADED=0` keeps the whole march on (for measuring one against the other).
    /// And whether the log has said the second has taken over.
    one_loaded: Option<wgpu::ComputePipeline>,
    loaded_off: bool,
    loaded_said: std::sync::atomic::AtomicBool,
    /// The march's audit (`INCUS_FAR_AUDIT`, for measuring): cut-down versions of the march (`audit_variants`), each
    /// compiled beside it under its own name and marched over the base layer into scratch after the far eye, so one
    /// Nsight trace names each one's registers, how many groups of rays fit and its time. Empty unless asked: the
    /// game never builds them [David, 2026-11-14: "start with trying to break it into smaller passes to fit more"; the
    /// march keeps 168 registers and 12 groups fit of 48].
    audit: Vec<(&'static str, wgpu::ComputePipeline)>,
    parity: usize,
    fresh: bool,
    /// A ray's step through thin haze, in cells (the lean step, 1d; `INCUS_FAR_HAZE_STEP`): a whole cell (20% off the
    /// sea, 13% off a player's run; David, looking at build 151: "i don't see anything bad about it. Seems like a good
    /// thing"), or old's half (0.5).
    haze_step: f32,
    most_steps: f32,
    /// The height table's true numbers (`bricks::tops_whole`): worked out on a thread of its own once every region is in
    /// and sent to the card whole (till then it holds each band's rounded top, and the skip is as it was); how long that
    /// took, once it's in.
    tops_making: Option<mpsc::Receiver<f32>>,
    tops_made: Option<f32>,
    tops_off: bool,
    /// For measuring a ray's two lengths apart (the deep dive, 2026-11-14): the farthest a ray goes
    /// (`INCUS_TEST_REACH`, blocks: the near length alone) and the nearest it begins (`INCUS_TEST_BEGIN`: the far
    /// length alone). The reach as it is, and 0, unset.
    reach: f32,
    begin: f32,
    /// The longest ray (the Oracle's step 0): the pass that counts a layer's rays' steps, each layer's four counts on the
    /// card, where they're read back, and whether a reading is free (0), on its way (1) or ready (2).
    tally: wgpu::ComputePipeline,
    tally_counts: Vec<wgpu::Buffer>,
    tally_back: wgpu::Buffer,
    tally_state: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

impl FarEye {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, store: Store, lane: Option<Lane>, rings: Vec<crate::far_ring::Ring>) -> Self {
        // The march's source: as it's written, or for a picture test asked to count a ray's steps by kind, a variant
        // that counts them (`kinds_source`; the game never builds it, so it costs the game nothing).
        #[cfg(not(test))]
        let far_source = include_str!("far.wgsl").to_string();
        #[cfg(test)]
        let far_source = kinds_source(include_str!("far.wgsl"));
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye"), source: wgpu::ShaderSource::Wgsl(far_source.as_str().into()) });
        let compute = wgpu::ShaderStages::COMPUTE;
        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry { binding, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let written = |binding: u32, format: wgpu::TextureFormat| wgpu::BindGroupLayoutEntry { binding, visibility: compute, ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::WriteOnly, format, view_dimension: wgpu::TextureViewDimension::D2 }, count: None };
        let read = |binding: u32, visibility: wgpu::ShaderStages, filterable: bool| wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let uniform = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let volume = |binding: u32| wgpu::BindGroupLayoutEntry { binding, visibility: compute, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D3, multisampled: false }, count: None };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the far eye"),
            entries: &[
                uniform(0, compute),
                storage(1, true),
                storage(2, true),
                storage(3, true),
                storage(4, true),
                written(5, wgpu::TextureFormat::Rgba8Unorm),
                written(6, wgpu::TextureFormat::Rgba32Float),
                storage(7, false),
                storage(8, true),
                storage(9, false),
                read(10, compute, false),
                storage(11, true),
                wgpu::BindGroupLayoutEntry { binding: 12, visibility: compute, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                volume(13),
                volume(14),
                volume(15),
                volume(16),
                read(22, compute, true),
                read(23, compute, true),
                read(24, compute, false),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the far eye"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some(entry), layout: Some(&pipeline_layout), module: &module, entry_point: Some(entry), compilation_options: Default::default(), cache: None })
        };
        let start = pipeline("start_pass");
        let march = pipeline("march_pass");
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let up_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bringing it up"),
            entries: &[
                read(0, fragment, true),
                read(1, fragment, true),
                read(2, fragment, true),
                read(3, fragment, false),
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: fragment, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                uniform(5, fragment),
            ],
        });
        let up_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("bringing it up"), source: wgpu::ShaderSource::Wgsl(include_str!("far_up.wgsl").into()) });
        let up_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("bringing it up"), bind_group_layouts: &[Some(&up_layout)], immediate_size: 0 });
        let up = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bringing it up"),
            layout: Some(&up_pipeline_layout),
            vertex: wgpu::VertexState { module: &up_module, entry_point: Some("whole_screen"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &up_module, entry_point: Some("brought_up"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("bringing it up"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        // The look starts at its defaults, with old's switches for measuring set at launch: the views
        // (`INCUS_SIGHT_STEPS`, `INCUS_SIGHT_LEVELS`) and the light left out (`INCUS_SIGHT_NO_SHADOWS`, `INCUS_SIGHT_NO_SKY`).
        let mut look = FarLook::DEFAULT;
        let set = |name: &str| std::env::var_os(name).is_some();
        look.view = if set("INCUS_SIGHT_LEVELS") {
            2
        } else if set("INCUS_SIGHT_STEPS") {
            1
        } else {
            0
        };
        look.shadows = !set("INCUS_SIGHT_NO_SHADOWS");
        look.sky_light = !set("INCUS_SIGHT_NO_SKY");
        // Change 1 (`INCUS_FAR_BLEND`): `code` makes no far textures and blends by the code (old's); otherwise they're made
        // and blend once they're in. The detail at launch (`INCUS_FAR_DETAIL`), for measuring.
        let textures = std::env::var("INCUS_FAR_BLEND").as_deref() != Ok("code");
        if !textures {
            look.blend = 0;
        }
        if let Some(detail) = std::env::var("INCUS_FAR_DETAIL").ok().and_then(|d| d.parse().ok()) {
            look.detail = detail;
        }
        let stand_in = |label: &str, format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D3,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        let stand_ins = [stand_in("no far atlas yet", wgpu::TextureFormat::Rg16Float), stand_in("no far atlas yet", wgpu::TextureFormat::Rgba8Unorm)];
        let light_stand_ins = [crate::far_light::stand_in(device, wgpu::TextureFormat::R16Float), crate::far_light::stand_in(device, wgpu::TextureFormat::R8Unorm)];
        let sun_asked = ([look.sun_from, look.sun_height], Instant::now());
        let blue_side = crate::blue_noise::SIDE as u32;
        let blue = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("the blue noise"),
            size: wgpu::Extent3d { width: blue_side, height: blue_side, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let blue = (blue.clone(), blue.create_view(&wgpu::TextureViewDescriptor::default()));
        let blue_made = Instant::now();
        let blue_bytes = Some(crate::blue_noise::bytes());
        log::info!("The blue noise: {blue_side} x {blue_side} made in {:.3} s", blue_made.elapsed().as_secs_f32());
        let edge = std::env::var("INCUS_FAR_EDGE").as_deref() != Ok("0");
        let haze_step = std::env::var("INCUS_FAR_HAZE_STEP").ok().and_then(|s| s.parse::<f32>().ok()).filter(|s| (0.25..=2.0).contains(s)).unwrap_or(1.0);
        let base = if std::env::var("INCUS_FAR_BASE").as_deref() == Ok("4") { 4.0 } else { LAYERS[0] };
        let layers = [base, base / 2.0, base / 4.0];
        // The one dispatch: `far.wgsl` with one layer's outputs changed for every layer's (`far_all.wgsl`), and its
        // camera read in place from its layer's row.
        let mut audit = Vec::new();
        let mut one_loaded = None;
        let one = (std::env::var("INCUS_FAR_SIDE").as_deref() != Ok("0")).then(|| {
            let source = one_source(far_source.as_str());
            // The two programs from the one text: the whole march under its own name, and the march once the textures
            // are in, which keeps the name the far eye's march has always had.
            const WHICH: &str = "const LOADED: bool = false;";
            assert_eq!(source.matches(WHICH).count(), 1, "far.wgsl says which program it is in one place");
            let loaded_source = source.replace(WHICH, "const LOADED: bool = true;");
            let source = source.replace("fn march_all(", "fn march_loading(");
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye, one dispatch"), source: wgpu::ShaderSource::Wgsl(source.into()) });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("the far eye, one dispatch"),
                entries: &[
                    uniform(0, compute),
                    storage(1, true),
                    storage(2, true),
                    storage(3, true),
                    storage(4, true),
                    written(5, wgpu::TextureFormat::Rgba8Unorm),
                    written(6, wgpu::TextureFormat::Rgba32Float),
                    storage(7, false),
                    storage(8, true),
                    storage(9, false),
                    read(10, compute, false),
                    storage(11, true),
                    wgpu::BindGroupLayoutEntry { binding: 12, visibility: compute, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                    volume(13),
                    volume(14),
                    volume(15),
                    volume(16),
                    read(22, compute, true),
                    read(23, compute, true),
                read(24, compute, false),
                    written(17, wgpu::TextureFormat::Rgba8Unorm),
                    written(19, wgpu::TextureFormat::Rgba8Unorm),
                ],
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the far eye, one dispatch"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("march_loading"), layout: Some(&pipeline_layout), module: &module, entry_point: Some("march_loading"), compilation_options: Default::default(), cache: None });
            let loaded_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye, once the textures are in"), source: wgpu::ShaderSource::Wgsl(loaded_source.into()) });
            one_loaded = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("march_all"), layout: Some(&pipeline_layout), module: &loaded_module, entry_point: Some("march_all"), compilation_options: Default::default(), cache: None }));
            // The audit's variants, if asked: the same layout, each its own module and its own name.
            if std::env::var("INCUS_FAR_AUDIT").is_ok_and(|asked| asked != "0") {
                for (name, text) in audit_variants(far_source.as_str()) {
                    let source = one_source(&text).replace("fn march_all(", &format!("fn {name}("));
                    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(name), source: wgpu::ShaderSource::Wgsl(source.into()) });
                    audit.push((name, device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some(name), layout: Some(&pipeline_layout), module: &module, entry_point: Some(name), compilation_options: Default::default(), cache: None })));
                }
                log::info!("The march's audit: {} cut-down versions compiled; each is marched over the base layer into scratch after the far eye (for measuring: the frame is that much longer)", audit.len());
            }
            (pipeline, layout)
        });
        let blocks = |name: &str| std::env::var(name).ok().and_then(|n| n.parse::<f32>().ok()).filter(|n| *n > 0.0);
        let (reach, begin) = (blocks("INCUS_TEST_REACH").unwrap_or(REACH), blocks("INCUS_TEST_BEGIN").unwrap_or(0.0));
        let most_steps = std::env::var("INCUS_TEST_STEPS").ok().and_then(|n| n.parse::<f32>().ok()).filter(|n| *n >= 1.0).unwrap_or(768.0);
        log::info!("The far rays: {}, {most_steps:.0} steps at most", if edge { "ended at the map's edge" } else { "marched on past the map's edge (INCUS_FAR_EDGE=0)" });
        let tally_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the longest ray"), source: wgpu::ShaderSource::Wgsl(include_str!("far_tally.wgsl").into()) });
        let tally = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("the longest ray"), layout: None, module: &tally_module, entry_point: Some("tally_pass"), compilation_options: Default::default(), cache: None });
        let tally_counts = LAYERS.iter().map(|_| device.create_buffer(&wgpu::BufferDescriptor { label: Some("a far layer's longest ray"), size: 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })).collect();
        let tally_back = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the longest rays read back"), size: 16 * LAYERS.len() as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let tally_state = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));
        Self { store, lane, rings, eye: [0.0; 2], start, march, up, layout, up_layout, sampler, targets: None, look, textures, atlas: None, building: None, stand_ins, light: None, lighting: None, sweeping: None, sun_asked, light_stand_ins, blue, blue_bytes, edge, layers, one, one_loaded, loaded_off: std::env::var("INCUS_FAR_LOADED").as_deref() == Ok("0"), loaded_said: std::sync::atomic::AtomicBool::new(false), audit, parity: 0, fresh: false, haze_step, most_steps, tops_making: None, tops_made: None, tops_off: false, reach, begin, tally, tally_counts, tally_back, tally_state }
    }

    /// Whether every region is on the card, the window's regions round the eye too, and the far textures if they're made.
    pub fn whole(&self) -> bool {
        // (The rings: once each has been on the card, with its textures if the atlas is in. Not held to the eye's place: a
        // moving mark outruns it, as it does the window.)
        self.lane.as_ref().is_none_or(|lane| lane.done == lane.total) && (self.atlas.is_some() || !self.textures) && self.store.behind == 0 && self.lit() && self.rings.iter().all(|ring| ring.from.is_some() || ring.failed) && self.tops_made.is_some()
    }

    /// Whether the far light is in.
    pub fn lit(&self) -> bool {
        self.light.is_some()
    }

    /// How far round the eye the usable 32s and 64s reach (blocks; none without a window), for a flight.
    pub fn usable_reach(&self, eye: [f32; 2]) -> Option<[f32; crate::bricks::WINDOWED]> {
        self.store.usable_reach(eye)
    }

    /// Which blends can be chosen now: the code's, the hardware's (once the far textures are in).
    pub fn blends(&self) -> [bool; 2] {
        [true, self.atlas.is_some()]
    }

    /// Whether the far eye marches by its second program, the march once the textures are in: the hardware's blend
    /// chosen and its textures in, every ring's bricks' too (a ring's come a little after the atlas, and till then
    /// only the code's blend can answer for them), and no view on. Otherwise by the whole march.
    pub fn loaded(&self) -> bool {
        let loaded = !self.loaded_off && self.one_loaded.is_some() && self.atlas.is_some() && self.look.blend >= 1 && self.look.view == 0 && self.rings.iter().all(|ring| ring.textured());
        if loaded && !self.loaded_said.swap(true, std::sync::atomic::Ordering::Relaxed) {
            log::info!("The far eye: marched from here by the march once the textures are in (no code's blend, no views)");
        }
        loaded
    }

    /// The look's settings a flight's numbers depend on, for its report.
    pub fn settings(&self) -> String {
        let window = self.store.window().map_or("the whole world on the card".to_string(), |w| format!("the 32s within {} regions of the eye, the 64s within {}", w[0], w[1]));
        let mut rays = if self.edge { "rays ended at the map's edge".to_string() } else { "rays marched past the map's edge".to_string() };
        rays += &format!(", thin haze stepped by {} of a cell; the layers {} x {}, {} x {} and {} x {} screen pixels", self.haze_step, self.layers[0], self.layers[0], self.layers[1], self.layers[1], self.layers[2], self.layers[2]);
        rays += ", light read from the cell";
        if !self.rings.is_empty() {
            let sizes: Vec<String> = self.rings.iter().map(|ring| format!("{}s", 1 << ring.level)).collect();
            rays += &format!(", the {} in rings round the eye", sizes.join(" and the "));
        }
        rays += if self.one.is_some() { ", marched by one dispatch (a finer layer led by the frame before's coarser)" } else { ", marched one after another" };
        if self.most_steps != 768.0 {
            rays += &format!(", {:.0} steps at most", self.most_steps);
        }
        if self.reach != REACH {
            rays += &format!(", FOR MEASURING rays ended {:.0} blocks out (the near length alone)", self.reach);
        }
        if self.begin != 0.0 {
            rays += &format!(", FOR MEASURING rays begun {:.0} blocks out (the far length alone)", self.begin);
        }
        // Where the rays' steps begin, if not as it was (scattered, over a whole step).
        let look = &self.look;
        if look.start_pattern != 0 || look.start_spread != 1.0 {
            let side = 1 << look.weave_bits.clamp(1, 4);
            rays += &match look.start_pattern {
                1 => format!(", rays' starts woven {side} x {side}"),
                2 => ", rays' starts by blue noise".to_string(),
                3 => format!(", rays' starts woven {side} x {side} with {} of blue noise", look.start_blue),
                _ => ", rays' starts scattered".to_string(),
            };
            if look.start_spread != 1.0 {
                rays += &format!(" over {} of a step", look.start_spread);
            }
        }
        format!("bricks of {}, {window}, {rays}, blended by {}, detail {:+}", self.store.side, if self.look.blend == 1 && self.atlas.is_some() { "the hardware (spread)" } else { "the initial blend (the code's, old's)" }, self.look.detail)
    }

    /// How the far cells are coming in, for the panel and the flight's report.
    pub fn state(&self) -> String {
        let (used, size) = self.store.pool_bytes();
        let lane = self.lane.as_ref().map_or(String::new(), |lane| {
            let eighth = lane.eighth.map_or(String::new(), |s| format!(", the eighth nearest in {s:.1} s"));
            format!("{} of {} regions in{eighth}; ", lane.done, lane.total)
        });
        let atlas = match (&self.atlas, &self.building) {
            (Some(a), _) => format!("; the atlas {} bricks, {:.0} MB of texels, built and sent in {:.1} s", a.bricks, a.bytes as f64 / 1e6, a.seconds),
            (None, Some(_)) => "; the atlas being built".to_string(),
            (None, None) => String::new(),
        };
        let ring: String = self.rings.iter().map(|ring| format!("; {}", ring.state())).collect();
        let light = self.light.as_ref().map_or(String::new(), |l| format!("; the far light {:.0} MB, its tops and sky made in {:.2} s, its last shadow swept and sent in {:.3} s", l.bytes as f64 / 1e6, l.made_in, l.swept_in));
        let tops = match self.tops_made {
            Some(_) if self.tops_off => "; the height table left as the bands' rounded tops (FOR MEASURING)".to_string(),
            Some(seconds) => format!("; the height table {:.0} MB, made and sent in {seconds:.2} s", self.store.tops_at as f64 * 4.0 / 1e6),
            None => String::new(),
        };
        format!("{lane}{} bricks, {:.0} MB of {:.0} MB{atlas}{ring}{tops}{light}", self.store.bricks, used as f64 / 1e6, size as f64 / 1e6)
    }

    /// The regions summed since the last frame put on the card, and the window brought to the eye, for at most
    /// `PUTTING_MS` together (the pass "putting the far cells on the card").
    pub fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, eye: [f32; 2]) {
        let started = Instant::now();
        self.eye = eye;
        if let Some(bytes) = self.blue_bytes.take() {
            let side = crate::blue_noise::SIDE as u32;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &self.blue.0, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                &bytes,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(side), rows_per_image: Some(side) },
                wgpu::Extent3d { width: side, height: side, depth_or_array_layers: 1 },
            );
        }
        // The atlas, once every region is in: built on a thread of its own from the cells kept and the window as it is
        // (held still till it's in), then taken in here.
        if self.lane.is_none() && self.atlas.is_none() && self.building.is_none() && self.textures {
            let kept = self.store.kept.clone();
            let window = self.store.window_regions();
            let windowed = if window.is_some() { crate::bricks::WINDOWED } else { 0 };
            let places = self.store.index_count as usize;
            let (device, queue) = (device.clone(), queue.clone());
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("making the far textures".into()).spawn(move || {
                let keep = |b: &crate::atlas::Brick| window.as_ref().is_none_or(|regions| b.level >= crate::far::FIRST + crate::bricks::WINDOWED || regions[b.level - crate::far::FIRST].contains(&crate::atlas::region_of(b)));
                let _ = sender.send(crate::atlas::build(&device, &queue, &kept, places, windowed, keep));
            });
            log::info!("The far textures: being made");
            self.building = Some(receiver);
        }
        if let Some(receiver) = &self.building
            && let Ok(atlas) = receiver.try_recv()
        {
            log::info!("The far textures: {} bricks, {:.0} MB of texels on the card, made and sent in {:.1} s", atlas.bricks, atlas.bytes as f64 / 1e6, atlas.seconds);
            self.atlas = Some(atlas);
            self.building = None;
        }
        self.light(device, queue);
        // The height table's true numbers, once every region is in: worked out on a thread of its own and sent whole.
        if self.lane.is_none() && self.tops_made.is_none() && self.tops_making.is_none() && std::env::var("INCUS_FAR_TOPS").as_deref() == Ok("0") {
            // (For flying against: the true numbers left unmade.)
            self.tops_made = Some(0.0);
            self.tops_off = true;
        }
        if self.lane.is_none() && self.tops_made.is_none() && self.tops_making.is_none() {
            let kept = self.store.kept.clone();
            let (queue, entries, at) = (queue.clone(), self.store.entries.clone(), self.store.tops_at);
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("making the height table".into()).spawn(move || {
                let started = Instant::now();
                for (start, tops) in crate::bricks::tops_whole(&kept) {
                    queue.write_buffer(&entries, (at + start as u64) * 4, &tops.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
                }
                let _ = sender.send(started.elapsed().as_secs_f32());
            });
            self.tops_making = Some(receiver);
        }
        if let Some(receiver) = &self.tops_making
            && let Ok(seconds) = receiver.try_recv()
        {
            log::info!("The height table: every column's highest thing worked out and sent in {seconds:.2} s");
            self.tops_made = Some(seconds);
            self.tops_making = None;
        }
        if let Some(lane) = &mut self.lane {
            while started.elapsed().as_secs_f32() * 1000.0 < PUTTING_MS {
                let taken = lane.take(1);
                let Some((region, columns)) = taken.into_iter().next() else { break };
                self.store.put_region(device, queue, region, &columns);
            }
            if lane.done == lane.total && self.store.regions_in == lane.total {
                let (used, _) = self.store.pool_bytes();
                log::info!("The far bricks: all {} regions in, {} bricks on the card, {:.0} MB of cells", lane.total, self.store.bricks, used as f64 / 1e6);
                self.lane = None;
            }
        }
        // The window brought to the eye with what's left of the pass's time (a millisecond at least), half of it when
        // the atlas follows too; held still while the atlas is being made from it.
        if self.building.is_none() {
            let left = (PUTTING_MS - started.elapsed().as_secs_f32() * 1000.0).max(1.0);
            let moved = self.store.follow(device, queue, eye, if self.atlas.is_some() { left / 2.0 } else { left });
            if let Some(atlas) = &mut self.atlas {
                for &(level, region) in &moved.left {
                    atlas.remove_region(level, region);
                }
                for &(level, region) in &moved.came {
                    atlas.add_region(device, queue, &self.store.kept, level, region);
                }
            }
        }
        // The rings brought to the eye: what their makers have handed over sent on, a little a frame each.
        for ring in &mut self.rings {
            ring.follow(device, queue, &mut self.store, self.atlas.as_mut(), eye, RING_MS);
        }
    }

    /// Whether the rings round the eye are on the card whole (true with none), for a picture.
    #[cfg(test)]
    pub fn ring_settled(&self) -> bool {
        self.rings.iter().all(|ring| ring.settled(&self.store, self.eye, self.atlas.is_some()))
    }

    /// The far light kept up (read from the cell): made once every region is in, on a thread of its own; and a new
    /// shadow swept, on a thread too, when the sun's setting has changed and stood still a quarter of a second, shown
    /// whole when it's in.
    fn light(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let sun = [self.look.sun_from, self.look.sun_height];
        if sun != self.sun_asked.0 {
            self.sun_asked = (sun, Instant::now());
        }
        if self.lane.is_none() && self.light.is_none() && self.lighting.is_none() {
            let kept = self.store.kept.clone();
            let (device, queue) = (device.clone(), queue.clone());
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("making the far light".into()).spawn(move || {
                let _ = sender.send(crate::far_light::make(&device, &queue, &kept, sun));
            });
            log::info!("The far light: being made");
            self.lighting = Some(receiver);
        }
        if let Some(receiver) = &self.lighting
            && let Ok(light) = receiver.try_recv()
        {
            log::info!("The far light: {:.0} MB on the card ({} x {}), its tops and sky made in {:.2} s, its shadow swept and sent in {:.3} s", light.bytes as f64 / 1e6, light.tops.size[0], light.tops.size[1], light.made_in, light.swept_in);
            self.light = Some(light);
            self.lighting = None;
        }
        if let Some(receiver) = &self.sweeping
            && let Ok((swept, seconds)) = receiver.try_recv()
        {
            if let Some(light) = &mut self.light {
                light.swept(swept, seconds);
                log::info!("The far light: the shadow swept for the sun from {:.0} at {:.0} and sent in {:.3} s, shown", swept[0], swept[1], seconds);
            }
            self.sweeping = None;
        }
        if let Some(light) = &self.light
            && self.sweeping.is_none()
            && light.sun != sun
            && self.sun_asked.1.elapsed().as_secs_f32() >= 0.25
        {
            let (tops, into) = light.sweeper();
            let queue = queue.clone();
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("sweeping the far shadow".into()).spawn(move || {
                let _ = sender.send((sun, crate::far_light::sweep_into(&queue, &tops, &into, sun)));
            });
            self.sweeping = Some(receiver);
        }
    }

    /// The layers sized to the screen, and their bind groups made with the bricks as they are.
    fn targets(&mut self, device: &wgpu::Device, screen: (u32, u32)) -> bool {
        let atlas_generation = self.atlas.as_ref().map_or(0, |a| a.generation);
        let light_now = self.light.as_ref().map(|l| l.generation);
        let (shadow, sky) = self.light.as_ref().map_or((&self.light_stand_ins[0], &self.light_stand_ins[1]), |l| (&l.shadow_views[l.shown], &l.sky));
        if self.targets.as_ref().is_some_and(|t| t.screen == screen && t.generation == self.store.generation && t.atlas_in == self.atlas.is_some() && t.atlas_generation == atlas_generation && t.light == light_now) {
            return false;
        }
        let texture = |label: &str, size: (u32, u32), format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        let buffer = |label: &str, size: u64, usage: wgpu::BufferUsages| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: size.max(16), usage, mapped_at_creation: false });
        let storage = wgpu::BufferUsages::STORAGE;
        let tiles = (screen.0.div_ceil(TILE), screen.1.div_ceil(TILE));
        let starts = buffer("the far starts", (tiles.0 * tiles.1) as u64 * 4, storage);
        let nothing_above = buffer("the far layers' nothing above", 16, storage);
        let no_far = texture("the far layers' no distances above", (1, 1), wgpu::TextureFormat::Rgba32Float);
        let mut layers: Vec<Layer> = Vec::new();
        let mut worth_above: Option<wgpu::Buffer> = None;
        let sizes = self.layers.map(|factor| ((screen.0 as f32 / factor).ceil().max(1.0) as u32, (screen.1 as f32 / factor).ceil().max(1.0) as u32));
        // By the one dispatch: all the layers' distances in one texture, side by side from its left, kept twice.
        let one_way = self.one.is_some();
        let packed = one_way.then(|| {
            let size = (sizes.iter().map(|s| s.0).sum(), sizes.iter().map(|s| s.1).max().unwrap_or(1));
            [0, 1].map(|_| texture("the far layers' distances", size, wgpu::TextureFormat::Rgba32Float))
        });
        let mut far_at = 0;
        for (factor, size) in self.layers.into_iter().zip(sizes) {
            let picture = texture("a far layer", size, wgpu::TextureFormat::Rgba8Unorm);
            // One after another, a layer has its own distances and its own words for the layer below (by the one
            // dispatch these are never marched with: the base's group is only the start pass's).
            let far = packed.clone().unwrap_or_else(|| {
                let own = texture("a far layer's distances", size, wgpu::TextureFormat::Rgba32Float);
                [own.clone(), own]
            });
            let worth = buffer("a far layer's worth it", if one_way { 16 } else { (size.0 * size.1) as u64 * 4 }, storage);
            let camera = buffer("a far layer's camera", CAMERA_ROWS as u64 * 16, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
            let far_above = if one_way { &far[1] } else { layers.last().map_or(&no_far, |above: &Layer| &above.far[0]) };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("a far layer"),
                layout: &self.layout,
                entries: &[
                    entry(0, camera.as_entire_binding()),
                    entry(1, self.store.table.as_entire_binding()),
                    entry(2, self.store.entries.as_entire_binding()),
                    entry(3, self.store.index.as_entire_binding()),
                    entry(4, self.store.pool.as_entire_binding()),
                    entry(5, wgpu::BindingResource::TextureView(&picture)),
                    entry(6, wgpu::BindingResource::TextureView(&far[0])),
                    entry(7, starts.as_entire_binding()),
                    entry(8, worth_above.as_ref().unwrap_or(&nothing_above).as_entire_binding()),
                    entry(9, worth.as_entire_binding()),
                    entry(10, wgpu::BindingResource::TextureView(far_above)),
                    entry(11, self.atlas.as_ref().map_or(&self.store.index, |a| &a.slots).as_entire_binding()),
                    entry(12, wgpu::BindingResource::Sampler(&self.sampler)),
                    entry(13, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[0], |a| &a.solid))),
                    entry(14, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[0]))),
                    entry(15, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[1]))),
                    entry(16, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[2]))),
                    entry(22, wgpu::BindingResource::TextureView(shadow)),
                    entry(23, wgpu::BindingResource::TextureView(sky)),
                    entry(24, wgpu::BindingResource::TextureView(&self.blue.1)),
                ],
            });
            worth_above = Some(worth);
            layers.push(Layer { factor, size, picture, far, far_at: if one_way { far_at } else { 0 }, camera, group });
            far_at += size.0;
        }
        // The one-dispatch form's: every layer's numbers in one buffer; the base's and the middle layer's words in one
        // run, kept twice; and a bind group for each of the two copies (writing one, led by the other).
        let one = self.one.as_ref().map(|(_, layout)| {
            let cameras = buffer("the far layers' cameras", (LAYERS.len() * CAMERA_ROWS) as u64 * 16, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
            let words = (layers[0].size.0 * layers[0].size.1 + layers[1].size.0 * layers[1].size.1) as u64 * 4;
            let worth = [buffer("the far layers' worth it", words, storage), buffer("the far layers' worth it", words, storage)];
            let groups = [0usize, 1].map(|p| {
                let q = 1 - p;
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("the far eye, one dispatch"),
                    layout,
                    entries: &[
                        entry(0, cameras.as_entire_binding()),
                        entry(1, self.store.table.as_entire_binding()),
                        entry(2, self.store.entries.as_entire_binding()),
                        entry(3, self.store.index.as_entire_binding()),
                        entry(4, self.store.pool.as_entire_binding()),
                        entry(5, wgpu::BindingResource::TextureView(&layers[0].picture)),
                        entry(6, wgpu::BindingResource::TextureView(&layers[0].far[p])),
                        entry(7, starts.as_entire_binding()),
                        entry(8, worth[q].as_entire_binding()),
                        entry(9, worth[p].as_entire_binding()),
                        entry(10, wgpu::BindingResource::TextureView(&layers[0].far[q])),
                        entry(11, self.atlas.as_ref().map_or(&self.store.index, |a| &a.slots).as_entire_binding()),
                        entry(12, wgpu::BindingResource::Sampler(&self.sampler)),
                        entry(13, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[0], |a| &a.solid))),
                        entry(14, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[0]))),
                        entry(15, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[1]))),
                        entry(16, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[2]))),
                    entry(22, wgpu::BindingResource::TextureView(shadow)),
                    entry(23, wgpu::BindingResource::TextureView(sky)),
                    entry(24, wgpu::BindingResource::TextureView(&self.blue.1)),
                        entry(17, wgpu::BindingResource::TextureView(&layers[1].picture)),
                        entry(19, wgpu::BindingResource::TextureView(&layers[2].picture)),
                    ],
                })
            });
            (cameras, groups)
        });
        // The audit's bind group: the far eye's own, with scratch in place of all it writes (its pictures, its
        // distances, its words) and of the words it reads, so a variant's march touches nothing that's shown or kept.
        // (Scratch a few pixels across: a write outside a texture isn't made, and a word past a buffer's end lands on
        // its last.)
        let audit = (!self.audit.is_empty()).then(|| self.one.as_ref().zip(one.as_ref())).flatten().map(|((_, layout), (cameras, _))| {
            let pictures = [0, 1, 2].map(|_| texture("the audit's scratch", (8, 8), wgpu::TextureFormat::Rgba8Unorm));
            let distances = texture("the audit's scratch", (8, 8), wgpu::TextureFormat::Rgba32Float);
            let words = [buffer("the audit's scratch", 16, storage), buffer("the audit's scratch", 16, storage)];
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the march's audit"),
                layout,
                entries: &[
                    entry(0, cameras.as_entire_binding()),
                    entry(1, self.store.table.as_entire_binding()),
                    entry(2, self.store.entries.as_entire_binding()),
                    entry(3, self.store.index.as_entire_binding()),
                    entry(4, self.store.pool.as_entire_binding()),
                    entry(5, wgpu::BindingResource::TextureView(&pictures[0])),
                    entry(6, wgpu::BindingResource::TextureView(&distances)),
                    entry(7, starts.as_entire_binding()),
                    entry(8, words[0].as_entire_binding()),
                    entry(9, words[1].as_entire_binding()),
                    entry(10, wgpu::BindingResource::TextureView(&layers[0].far[0])),
                    entry(11, self.atlas.as_ref().map_or(&self.store.index, |a| &a.slots).as_entire_binding()),
                    entry(12, wgpu::BindingResource::Sampler(&self.sampler)),
                    entry(13, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[0], |a| &a.solid))),
                    entry(14, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[0]))),
                    entry(15, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[1]))),
                    entry(16, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[2]))),
                    entry(22, wgpu::BindingResource::TextureView(shadow)),
                    entry(23, wgpu::BindingResource::TextureView(sky)),
                    entry(24, wgpu::BindingResource::TextureView(&self.blue.1)),
                    entry(17, wgpu::BindingResource::TextureView(&pictures[1])),
                    entry(19, wgpu::BindingResource::TextureView(&pictures[2])),
                ],
            })
        });
        let up_numbers = buffer("bringing it up's numbers", 32, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let view = |k: usize| wgpu::BindingResource::TextureView(&layers[k].picture);
        let up_group = [0usize, 1].map(|p| device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bringing it up"),
            layout: &self.up_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: view(0) },
                wgpu::BindGroupEntry { binding: 1, resource: view(1) },
                wgpu::BindGroupEntry { binding: 2, resource: view(2) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&layers[0].far[p]) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 5, resource: up_numbers.as_entire_binding() },
            ],
        }));
        let tally_parts: Vec<wgpu::Buffer> = layers.iter().map(|_| buffer("a far layer's part of its distances", 16, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)).collect();
        let tally_groups = layers
            .iter()
            .zip(&self.tally_counts)
            .zip(&tally_parts)
            .map(|((layer, counts), part)| [0usize, 1].map(|p| device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("the longest ray"), layout: &self.tally.get_bind_group_layout(0), entries: &[entry(0, wgpu::BindingResource::TextureView(&layer.far[p])), entry(1, counts.as_entire_binding()), entry(2, part.as_entire_binding())] })))
            .collect();
        self.targets = Some(Targets { screen, generation: self.store.generation, atlas_in: self.atlas.is_some(), atlas_generation, light: light_now, tiles, tally_groups, tally_parts, layers, up_numbers, up_group, one, audit });
        true
    }

    /// The frame's numbers for each layer and for bringing it up, from the camera (done before the card's passes are
    /// recorded).
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, screen: (u32, u32), camera: &Camera, sea_level: f32) {
        self.fresh = self.targets(device, screen);
        if self.one.is_some() {
            self.parity ^= 1;
        }
        let Some(targets) = &self.targets else { return };
        let (facing, pitch) = (camera.facing.to_radians(), camera.pitch.to_radians());
        let forward = [pitch.cos() * facing.cos(), pitch.cos() * facing.sin(), pitch.sin()];
        let right = [-facing.sin(), facing.cos(), 0.0];
        let up = [-pitch.sin() * facing.cos(), -pitch.sin() * facing.sin(), pitch.cos()];
        let tan_half = (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let screen_pixel = 2.0 * tan_half / screen.1.max(1) as f32;
        let look = self.look;
        // The blend as chosen, if its way is built and in; the code's till then.
        let blend = if self.blends()[look.blend.min(1) as usize] { look.blend.min(1) } else { 0 };
        let atlas_size = self.atlas.as_ref().map_or([1.0; 3], |a| a.size.map(|v| v as f32));
        let (regions, window_levels) = (self.store.regions, self.store.window().map_or(0.0, |_| crate::bricks::WINDOWED as f32));
        let (from, height) = (look.sun_from.to_radians(), look.sun_height.to_radians());
        let sun = [height.cos() * from.cos(), height.cos() * from.sin(), height.sin()];
        let (direct, glow) = daylight(sun[2]);
        let water = WATER.map(|c| {
            let v = c as f32 / 255.0;
            if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        });
        // The rings as they are on the card: each one's box (an empty one while it isn't there), and the finest level
        // the march asks for (the finest ring's on the card).
        let mut ring_boxes = [[1.0f32, 1.0, 0.0, 0.0]; 4];
        let mut finest_level = crate::far::FIRST;
        for ring in &self.rings {
            if ring.from.is_some() {
                ring_boxes[crate::far::FIRST - 1 - ring.level] = ring.reach(&self.store.levels[crate::bricks::row(ring.level)]);
                finest_level = finest_level.min(ring.level);
            }
        }
        for (k, layer) in targets.layers.iter().enumerate() {
            let pixel = 2.0 * tan_half / layer.size.1 as f32;
            let above = (k > 0).then(|| &targets.layers[k - 1]);
            let below = targets.layers.get(k + 1);
            let fade = match (below.is_some(), k) {
                (false, _) => [1.0, 1.0],
                (true, 0) => look.fade_4,
                (true, _) => look.fade_2,
            };
            let numbers: [[f32; 4]; CAMERA_ROWS] = [
                [camera.position.x, camera.position.y, camera.position.z, self.haze_step],
                [forward[0], forward[1], forward[2], 0.0],
                [right[0] * tan_half * aspect, right[1] * tan_half * aspect, right[2] * tan_half * aspect, 0.0],
                [up[0] * tan_half, up[1] * tan_half, up[2] * tan_half, 0.0],
                [layer.size.0 as f32, layer.size.1 as f32, pixel, if self.light.is_some() { 1.0 } else { 0.0 }],
                [sun[0], sun[1], sun[2], look.ambient.clamp(0.0, 1.0)],
                [self.reach, -look.detail as f32, look.view as f32, self.store.highest],
                [targets.tiles.0 as f32, targets.tiles.1 as f32, (TILE as f32 / layer.factor).max(1.0), screen_pixel * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
                [above.map_or(0.0, |a| a.size.0 as f32), if below.is_some() { 1.0 } else { 0.0 }, 1.0, blend as f32],
                [1.0, if look.shadows && direct > 0.0 { 1.0 } else { 0.0 }, finest_level as f32, if self.edge { 1.0 } else { 0.0 }],
                [above.map_or(0.0, |a| layer.factor / a.factor), below.map_or(0.0, |b| b.factor / layer.factor), self.most_steps, self.store.tops_at as f32],
                [fade[0].max(0.1), fade[1].max(0.05), look.hand_over.clamp(0.3, 0.95), look.sky_weight.max(0.0)],
                [if look.sky_light { 1.0 } else { 0.0 }, look.sky_strength.clamp(0.0, 1.0), look.sky_tint.clamp(0.0, 1.0), self.begin],
                [if look.fog { 3.0 / look.fog_reach.max(10.0) } else { 0.0 }, std::f32::consts::LN_2 / look.fog_height.max(1.0), sea_level, look.fog_warmth.clamp(0.0, 1.0)],
                [direct, NIGHT_LIGHT + (1.0 - NIGHT_LIGHT) * glow, glow, NIGHT_LIGHT],
                [water[0], water[1], water[2], 20f32.ln() / WATER_CLARITY],
                [atlas_size[0], atlas_size[1], atlas_size[2], self.atlas.as_ref().map_or(1.0, |a| a.side as f32)],
                [window_levels, regions.1 as f32, regions.0[0] as f32, regions.0[1] as f32],
                [look.start_pattern.min(3) as f32, look.weave_bits.clamp(1, 4) as f32, look.start_blue.clamp(0.0, 1.0), look.start_spread.clamp(0.0, 1.0)],
                ring_boxes[0],
                ring_boxes[1],
                ring_boxes[2],
                ring_boxes[3],
            ];
            let bytes = numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>();
            queue.write_buffer(&layer.camera, 0, &bytes);
            if let Some((cameras, _)) = &targets.one {
                queue.write_buffer(cameras, (k * CAMERA_ROWS) as u64 * 16, &bytes);
            }
        }
        let finest = self.layers[LAYERS.len() - 1];
        let up_numbers: [f32; 8] = [screen.0 as f32, screen.1 as f32, finest, self.layers[0], look.bring_up.min(2) as f32, targets.layers[0].size.0 as f32, targets.layers[0].size.1 as f32, 0.0];
        queue.write_buffer(&targets.up_numbers, 0, &up_numbers.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    }

    /// The start pass: a cone a tile, where its rays can start.
    pub fn start(&self, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) {
        let Some(targets) = &self.targets else { return };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the far start"), timestamp_writes: timestamps });
        pass.set_pipeline(&self.start);
        pass.set_bind_group(0, &targets.layers[0].group, &[]);
        pass.dispatch_workgroups(targets.tiles.0.div_ceil(8), targets.tiles.1.div_ceil(8), 1);
    }

    /// Layer `k` of the pyramid (0 the base, 8 × 8). By the one dispatch, all the layers are marched in the first's pass
    /// and the others' passes are empty, kept for their place in the timing.
    pub fn layer(&self, k: usize, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) {
        let Some(targets) = &self.targets else { return };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("a far layer"), timestamp_writes: timestamps });
        // By the one dispatch: the base's groups, the middle layer's and the finest's side by side across the grid
        // (`far_all.wgsl`). In the frame the layers were remade there's no frame before to lead it: it's run three
        // times, each led by the last (the third's lead is then this frame's own), ending on this frame's copy.
        if let (Some((pipeline, _)), Some((_, groups))) = (&self.one, &targets.one) {
            if k == 0 {
                let groups_of = |layer: &Layer| (layer.size.0.div_ceil(8), layer.size.1.div_ceil(8));
                let (base, middle, fine) = (groups_of(&targets.layers[0]), groups_of(&targets.layers[1]), groups_of(&targets.layers[2]));
                pass.set_pipeline(if self.loaded() { self.one_loaded.as_ref().unwrap_or(pipeline) } else { pipeline });
                let runs = if self.fresh { 3 } else { 1 };
                for run in 0..runs {
                    let copy = if (runs - 1 - run) % 2 == 0 { self.parity } else { 1 - self.parity };
                    pass.set_bind_group(0, &groups[copy], &[]);
                    pass.dispatch_workgroups(base.0 + middle.0 + fine.0, fine.1.max(middle.1).max(base.1), 1);
                }
                // The audit's variants, each in a pass of its own under its own name (so a trace gives each its
                // time): the base layer's groups, into scratch.
                if let Some(group) = &targets.audit {
                    drop(pass);
                    for (name, pipeline) in &self.audit {
                        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(name), timestamp_writes: None });
                        pass.set_pipeline(pipeline);
                        pass.set_bind_group(0, group, &[]);
                        pass.dispatch_workgroups(base.0, base.1, 1);
                    }
                }
            }
            return;
        }
        let layer = &targets.layers[k];
        pass.set_pipeline(&self.march);
        pass.set_bind_group(0, &layer.group, &[]);
        pass.dispatch_workgroups(layer.size.0.div_ceil(8), layer.size.1.div_ceil(8), 1);
    }

    /// The longest ray counted (step 0), after the layers are marched: each layer's rays' steps into its four counts,
    /// copied to where they're read back. Whether it was recorded (not while a reading is still on its way).
    pub fn tally(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) -> bool {
        let Some(targets) = &self.targets else { return false };
        if self.tally_state.load(std::sync::atomic::Ordering::Acquire) != 0 {
            return false;
        }
        for (k, layer) in targets.layers.iter().enumerate() {
            queue.write_buffer(&self.tally_counts[k], 0, &[0u8; 16]);
            queue.write_buffer(&targets.tally_parts[k], 0, &[layer.far_at, 0, layer.size.0, layer.size.1].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the longest ray"), timestamp_writes: None });
                pass.set_pipeline(&self.tally);
                pass.set_bind_group(0, &targets.tally_groups[k][self.parity], &[]);
                pass.dispatch_workgroups(layer.size.0.div_ceil(8), layer.size.1.div_ceil(8), 1);
            }
            encoder.copy_buffer_to_buffer(&self.tally_counts[k], 0, &self.tally_back, 16 * k as u64, 16);
        }
        true
    }

    /// The frame that counted the longest rays is handed to the card: their reading asked for, never waited on.
    pub fn tally_sent(&self) {
        use std::sync::atomic::Ordering;
        self.tally_state.store(1, Ordering::Release);
        let state = self.tally_state.clone();
        self.tally_back.slice(..).map_async(wgpu::MapMode::Read, move |result| state.store(if result.is_ok() { 2 } else { 0 }, Ordering::Release));
    }

    /// The longest rays, if a reading has come back: for each layer, the most steps any ray took and how many rays
    /// took over 256, 384 and 512.
    pub fn tally_read(&self) -> Option<Vec<[u32; 4]>> {
        use std::sync::atomic::Ordering;
        if self.tally_state.load(Ordering::Acquire) != 2 {
            return None;
        }
        let counts = self.tally_back.slice(..).get_mapped_range().unwrap().as_chunks::<16>().0.iter().map(|b| [0, 1, 2, 3].map(|k| u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]]))).collect();
        self.tally_back.unmap();
        self.tally_state.store(0, Ordering::Release);
        Some(counts)
    }

    /// The pyramid brought up onto the screen's picture.
    pub fn bring_up(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let Some(targets) = &self.targets else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("bringing it up"), color_attachments: &[Some(attachment)], timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.up);
        pass.set_bind_group(0, &targets.up_group[self.parity], &[]);
        pass.draw(0..3, 0..1);
    }
}

/// A bind group's entry.
fn entry(binding: u32, resource: wgpu::BindingResource<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource }
}

/// The time of day by the sun's height (its up part): the sun's own light, and the sky's glow (old's).
fn daylight(sun_up: f32) -> (f32, f32) {
    let height = sun_up.clamp(-1.0, 1.0).asin().to_degrees();
    let smooth = |from: f32, to: f32| {
        let k = ((height - from) / (to - from)).clamp(0.0, 1.0);
        k * k * (3.0 - 2.0 * k)
    };
    let glow = smooth(-18.0, 4.0);
    (smooth(-2.0, 4.0), glow * glow)
}

/// The one dispatch's source from the march's (`far.wgsl`): `far_all.wgsl` put in the place of one layer's outputs,
/// and every use of the camera made a read of this invocation's layer's row.
fn one_source(whole: &str) -> String {
    let (begin, end) = ("// ── One layer's outputs: begin ──", "// ── One layer's outputs: end ──");
    let (from, to) = (whole.find(begin).expect("the outputs' first mark"), whole.find(end).expect("the outputs' last mark"));
    format!("{}{}{}", &whole[..from], include_str!("far_all.wgsl"), &whole[to..]).replace("camera.", "cameras[layer_now].").replace("@group(0) @binding(0) var<uniform> camera: Camera;", "@group(0) @binding(0) var<uniform> cameras: array<Camera, 3>;")
}

/// The march's audit (`FarEye::audit`): cut-down versions of `far.wgsl`, each with one part taken out, by name. What
/// each says, read off Nsight's Shader Pipelines rows (# Reg, # Warp) and the trace's passes: what that part costs
/// the march in registers, and what a pass holding only the rest would fit. They draw wrong pictures on purpose
/// (into scratch). Each change must find its one place in the source: a variant that can't is a panic at start,
/// under the switch only, and `the_audit_s_variants_are_sound` says so first.
fn audit_variants(whole: &str) -> Vec<(&'static str, String)> {
    let whole = whole.replace("\r\n", "\n");
    // One place changed.
    fn once(text: &str, old: &str, new: &str) -> String {
        assert_eq!(text.matches(old).count(), 1, "the audit can't find its place in far.wgsl: {old:?}");
        text.replace(old, new)
    }
    // From one place up to another, changed.
    fn cut(text: &str, from: &str, to: &str, with: &str) -> String {
        assert_eq!(text.matches(from).count(), 1, "the audit can't find its place in far.wgsl: {from:?}");
        let a = text.find(from).unwrap();
        let b = a + text[a..].find(to).unwrap_or_else(|| panic!("the audit can't find its place in far.wgsl after {from:?}: {to:?}"));
        format!("{}{}{}", &text[..a], with, &text[b..])
    }
    const SKIP: &str = "        // Space with nothing in it is skipped whole.";
    const SAMPLE: &str = "        // Over the last stretch before the next level takes over";
    const AFTER: &str = "    // Down through the world's floor having gathered";
    const FINISH: &str = "    // Its shadow and sky light only where it shows";
    const VIEWS: &str = "    if (!LOADED && camera.reach.z > 1.5 && seen.level >= 0) {\n";
    const PUT: &str = "    // Kept with its alpha (premultiplied)";
    // A ray stops where it would first sample: the hops alone (what a pass that only crosses clear air would be).
    let hops = |t: &str| cut(t, SAMPLE, AFTER, "        first = t;\n        first_level = level;\n        break;\n    }\n");
    // No skipping: every step samples (what a pass that only samples would be).
    let samples = |t: &str| cut(t, SKIP, SAMPLE, "");
    // Over a hand-over, the level's own blend alone: no second level read and mixed.
    let no_handover = |t: &str| once(t, "        if (mixed > 0.0) {\n", "        if (false) {\n");
    // The ray's answer written as it is: no light, fog, sky's weight or views after the march.
    let no_finish = |t: &str| cut(t, FINISH, PUT, "    if (camera.layer.y > 0.5) {\n        put_worth(id.y * size.x + id.x, u32(round(worth_of(seen.distance, seen.level) * 255.0)));\n    }\n    var colour = seen.colour;\n");
    // Nothing kept of the cells over the world's floor.
    let no_floor = |t: &str| once(t, "                if (p.z <= size * 1.5) {\n                    bottom = true;\n                    bottom_here = here;\n                    bottom_sun = here_sun;\n                    bottom_t = t;\n                    bottom_level = level;\n                }\n", "");
    // The skip with two columns, its own and the one two levels coarser: no third row kept.
    let two_rows = |t: &str| {
        let t = once(t, "        for (var k = 0; k < 3 && in_air && !walking; k++) {\n", "        for (var k = 1; k < 3 && in_air && !walking; k++) {\n");
        let t = once(&t, "            keep_row(&row_far, far_level);\n", "");
        once(&t, "            high_far = top_in(row_far, i32(cc_far.x), i32(cc_far.y));\n", "")
    };
    // No rings: a level is its distance's, and nothing fades at a ring's edge.
    let no_rings = |t: &str| {
        let t = once(t, "        level = ring_level(level, p);\n", "");
        let t = once(&t, "    l = ring_level(l, p);\n", "");
        once(&t, "        if (level < FIRST) {\n            mixed = max(mixed, 1.0 - in_ring(level, p) / (RING_FADE * f32(1 << u32(level))));\n        }\n", "")
    };
    // Neither of the views (the steps', the levels' with its table of six tints).
    let no_views = |t: &str| cut(t, VIEWS, PUT, "");
    // Where the bricks can't answer, nothing: no blend of eight cells by the code.
    let no_code_blend = |t: &str| once(t, "    if (!LOADED && spot.kind == 1) {\n        return blend(spot.level, p);\n    }\n", "    if (spot.kind == 1) {\n        return empty_look(true);\n    }\n");
    let lean = |t: &str| no_code_blend(&no_views(&no_rings(&two_rows(&no_floor(&no_handover(t))))));
    vec![
        ("audit_whole", whole.clone()),
        ("audit_hops", hops(&whole)),
        ("audit_samples", samples(&whole)),
        ("audit_no_handover", no_handover(&whole)),
        ("audit_no_finish", no_finish(&whole)),
        ("audit_no_floor", no_floor(&whole)),
        ("audit_two_rows", two_rows(&whole)),
        ("audit_no_rings", no_rings(&whole)),
        ("audit_no_views", no_views(&whole)),
        ("audit_no_code_blend", no_code_blend(&whole)),
        ("audit_lean", lean(&whole)),
        ("audit_samples_lean", samples(&no_code_blend(&no_views(&no_floor(&no_handover(&whole)))))),
        ("audit_hops_lean", hops(&no_views(&no_floor(&whole)))),
    ]
}

/// For the Oracle's question of 2026-11-14 ("the longest ray's steps by kind"): the march's source with a ray's steps
/// counted by kind, if a picture test asks (`FAR_TEST_KINDS`: three kinds' numbers, drawn as the picture's red, green
/// and blue, a 1,024th a step). The kinds: 0 a region not summed passed whole; 1 a skip over a coarser column
/// (four levels up, or two), 2 over its own level's; 3 a back-up after a first touch; 4 a stride through air; 5 a step walking back in
/// through air; 6 a step in thin haze; 7 a step at a surface; 8 a sample mixed from two levels; 9 a blend fallen back
/// to the code; 10 a sample read from the 8s, 11 from the 16s; 12 every step. Each change must find its one place in
/// the source, or the variant isn't made and the test says so.
#[cfg(test)]
fn kinds_source(source: &str) -> String {
    // Or one of the audit's cut-down versions drawn as the march itself (`FAR_TEST_VARIANT`, its name): to see what a
    // part taken out does to the picture, against the whole march's.
    if let Ok(asked) = std::env::var("FAR_TEST_VARIANT") {
        return audit_variants(source).into_iter().find(|(name, _)| *name == asked).unwrap_or_else(|| panic!("FAR_TEST_VARIANT={asked} is none of the audit's versions")).1;
    }
    // Or its steps counted by the level they were taken at (`FAR_TEST_LEVELS`: three levels' numbers, drawn the same
    // way): a level is a doubling of distance, so this is where along a ray its steps lie (for sizing a long ray cut
    // into lengths, or spread over frames).
    if let Ok(asked) = std::env::var("FAR_TEST_LEVELS") {
        let show: Vec<usize> = asked.split(',').filter_map(|n| n.trim().parse().ok()).collect();
        assert_eq!(show.len(), 3, "FAR_TEST_LEVELS wants three levels");
        let changes: [(&str, String); 5] = [
            ("    sky_share: f32,\n}", "    sky_share: f32,\n    kinds: array<f32, 17>,\n}".into()),
            ("    var steps = 0;\n", "    var steps = 0;\n    var kinds: array<f32, 17>;\n".into()),
            ("        var skipped = false;\n", "        kinds[level] += 1.0;\n        var skipped = false;\n".into()),
            ("gathered - sunny, touched, through);", "gathered - sunny, touched, through, kinds);".into()),
            ("    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", format!("    colour = vec3<f32>(seen.kinds[{}], seen.kinds[{}], seen.kinds[{}]) / 1024.0;\n    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", show[0], show[1], show[2])),
        ];
        let mut made = source.replace("\r\n", "\n");
        for (old, new) in changes {
            assert_eq!(made.matches(old).count(), 1, "the steps-by-level variant can't find its place in far.wgsl: {old:?}");
            made = made.replace(old, &new);
        }
        return made;
    }
    let Ok(asked) = std::env::var("FAR_TEST_KINDS") else { return source.to_string() };
    let show: Vec<usize> = asked.split(',').filter_map(|n| n.trim().parse().ok()).collect();
    assert_eq!(show.len(), 3, "FAR_TEST_KINDS wants three kinds");
    let kind = |k: usize| if k == 12 { "seen.steps".to_string() } else { format!("seen.kinds[{k}]") };
    let changes: [(&str, String); 14] = [
        ("    sky_share: f32,\n}", "    sky_share: f32,\n    kinds: array<f32, 12>,\n}".into()),
        ("    var steps = 0;\n", "    var steps = 0;\n    var kinds: array<f32, 12>;\n".into()),
        ("                t = max(leave2(o.xy, d.xy, vec2<f32>(place.region), f32(1 << u32(REGION))), t) + 0.01;\n", "                t = max(leave2(o.xy, d.xy, vec2<f32>(place.region), f32(1 << u32(REGION))), t) + 0.01;\n                kinds[0] += 1.0;\n".into()),
        ("                skipped = true;\n", "                skipped = true;\n                kinds[1 + k / 2] += 1.0;\n".into()),
        ("        var over = 0.0;\n", "        if (level == 3) { kinds[10] += 1.0; } else if (level == 4) { kinds[11] += 1.0; }\n        var over = 0.0;\n".into()),
        ("        if (mixed > 0.0) {\n", "        if (mixed > 0.0) {\n            kinds[8] += 1.0;\n".into()),
        ("            } else {\n                look = look_at(here, p);\n", "            } else {\n                kinds[9] += 1.0;\n                look = look_at(here, p);\n".into()),
        ("            look = far_blend_kept(&kept, place, level, p);\n", "            let spot_k = spot_kept(&kept, place, level, p);\n            if (spot_k.kind == 1) { kinds[9] += 1.0; }\n            look = look_at(spot_k, p);\n".into()),
        ("            walk_to = t;\n", "            walk_to = t;\n            kinds[3] += 1.0;\n".into()),
        ("        if (thick > 0.0) {\n            in_air = false;\n            walking = true;\n", "        if (thick > 0.0) {\n            in_air = false;\n            walking = true;\n            if (thick > 0.02) { kinds[7] += 1.0; } else { kinds[6] += 1.0; }\n".into()),
        ("        } else if (walking && t < walk_to) {\n            in_air = false;\n", "        } else if (walking && t < walk_to) {\n            in_air = false;\n            kinds[5] += 1.0;\n".into()),
        ("        } else {\n            last_air = t;\n            walking = false;\n", "        } else {\n            last_air = t;\n            walking = false;\n            kinds[4] += 1.0;\n".into()),
        ("gathered - sunny, touched, through);", "gathered - sunny, touched, through, kinds);".into()),
        ("    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", format!("    colour = vec3<f32>({}, {}, {}) / 1024.0;\n    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", kind(show[0]), kind(show[1]), kind(show[2]))),
    ];
    let mut made = source.replace("\r\n", "\n");
    for (old, new) in changes {
        assert_eq!(made.matches(old).count(), 1, "the steps-by-kind variant can't find its place in far.wgsl: {old:?}");
        made = made.replace(old, &new);
    }
    made
}

#[cfg(test)]
mod tests {
    /// The audit's variants each find their places in the march and are sound shaders (parsed and checked without the
    /// card), so `INCUS_FAR_AUDIT` can't fail at the game's start.
    #[test]
    fn the_audit_s_variants_are_sound() {
        let variants = super::audit_variants(include_str!("far.wgsl"));
        assert!(variants.len() >= 10);
        for (name, text) in variants {
            let source = super::one_source(&text).replace("fn march_all(", &format!("fn {name}("));
            let module = wgpu::naga::front::wgsl::parse_str(&source).unwrap_or_else(|error| panic!("{name} doesn't parse: {}", error.emit_to_string(&source)));
            wgpu::naga::valid::Validator::new(wgpu::naga::valid::ValidationFlags::all(), wgpu::naga::valid::Capabilities::all()).validate(&module).unwrap_or_else(|error| panic!("{name} isn't sound: {error:?}"));
            assert!(module.entry_points.iter().any(|entry| entry.name == name), "{name} has no entry of its name");
        }
    }

    use super::*;
    use crate::doorway::Doorway;
    use crate::far::{FarColours, region, regions_of};

    /// The far eye draws (plan 2.2, step 2a): its passes made and run with no error from the card, off the screen, from
    /// the spawn looking north, with the regions round the spawn on the card: the picture has ground below and sky above.
    #[test]
    fn the_far_eye_draws_ground_and_sky() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).expect("a graphics card");
        let largest = adapter.limits().max_buffer_size.min(adapter.limits().max_storage_buffer_binding_size).min((2 << 30) - 1);
        let required_limits = wgpu::Limits { max_buffer_size: largest, max_storage_buffer_binding_size: largest, ..wgpu::Limits::default() };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("a test"), required_limits, ..Default::default() })).expect("the card's device");
        let errors = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let marks = crate::marks::load().unwrap();
        // From the spawn, or another mark if asked (`FAR_TEST_MARK`, its name, for a look).
        let mark = std::env::var("FAR_TEST_MARK").ok().and_then(|name| marks.iter().find(|m| m.name == name)).unwrap_or(&marks[0]);
        let mut camera = Camera::new();
        camera.position = crate::marks::eye(&doorway, mark).unwrap();
        camera.facing = mark.facing;
        camera.pitch = mark.pitch;
        // Or from anywhere, if asked (`FAR_TEST_EYE`: north, east, up, facing, pitch, as the Camera page shows them;
        // the mark still says which regions are loaded), for a look at the very view David sent.
        let eye_asked: Option<Vec<f32>> = std::env::var("FAR_TEST_EYE").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect());
        if let Some(v) = eye_asked.as_ref().filter(|v| v.len() == 5) {
            (camera.position.x, camera.position.y, camera.position.z, camera.facing, camera.pitch) = (v[0], v[1], v[2], v[3], v[4]);
        }
        let at = [camera.position.x, camera.position.y];
        // The regions round the mark's (one each way, or as many as `FAR_TEST_REGIONS` says: for a look at the window
        // against the whole world, step 4; `all`: the whole world, summed on the loading lane as the game sums it), the
        // window brought to the eye.
        let mut store = Store::new(&device, &queue, regions_of(&doorway), doorway.height(), crate::bricks::brick_side(), crate::bricks::window_regions());
        let asked = std::env::var("FAR_TEST_REGIONS").ok();
        let lane = if asked.as_deref() == Some("all") {
            let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(1).max(1);
            Some(crate::far::Lane::start(crate::doorway::test_world(), at, threads).unwrap())
        } else {
            let around = asked.and_then(|r| r.parse::<i32>().ok()).unwrap_or(1);
            let middle = [(at[0] / 1024.0).floor() as i32, (at[1] / 1024.0).floor() as i32];
            for ry in middle[1] - around..=middle[1] + around {
                for rx in middle[0] - around..=middle[0] + around {
                    store.put_region(&device, &queue, [rx, ry], &region(&doorway, &colours, [rx, ry]).unwrap());
                }
            }
            store.follow(&device, &queue, at, f32::INFINITY);
            None
        };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        // The rings, as the game starts them (as many as `INCUS_FAR_RING` says: 0 none).
        let rings = crate::far_ring::rings_asked().into_iter().map(|level| crate::far_ring::Ring::start(crate::doorway::test_world(), store.levels[crate::bricks::row(level)], store.kept.side()).unwrap()).collect();
        let mut eye = FarEye::new(&device, format, store, lane, rings);
        // The whole world: put on the card as it's summed, till every region and the window round the eye are in, the
        // writes handed to the card each time round as a frame would (held, their copies pile up till it runs out).
        while eye.lane.is_some() || eye.store.behind > 0 {
            eye.put(&device, &queue, at);
            queue.submit([]);
            let _ = device.poll(wgpu::PollType::Poll);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        // The blend to draw with, if asked (`FAR_TEST_BLEND`: 0 the code's, 1 the hardware's): the atlas made first.
        // The detail is the default's, or `INCUS_FAR_DETAIL`'s.
        if let Some(blend) = std::env::var("FAR_TEST_BLEND").ok().and_then(|b| b.parse::<u32>().ok()) {
            while !eye.blends()[blend as usize] {
                eye.put(&device, &queue, at);
                queue.submit([]);
                let _ = device.poll(wgpu::PollType::Poll);
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            eye.look.blend = blend;
        }
        // The far light made first too (read from the cell), the ring round the eye whole on the card, and the height
        // table's true numbers in.
        while !eye.lit() || !eye.ring_settled() || eye.tops_made.is_none() {
            eye.put(&device, &queue, at);
            queue.submit([]);
            let _ = device.poll(wgpu::PollType::Poll);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        // The ring moved first, if asked (`FAR_TEST_MOVE`: blocks north, east): brought to a place that far from the
        // eye, then back, so the picture is drawn from a ring that has moved. It must be the picture of a ring made here.
        let moved: Vec<f32> = std::env::var("FAR_TEST_MOVE").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        if moved.len() == 2 {
            for place in [[at[0] - moved[0], at[1] - moved[1]], at] {
                loop {
                    eye.put(&device, &queue, place);
                    queue.submit([]);
                    let _ = device.poll(wgpu::PollType::Poll);
                    if eye.ring_settled() && eye.store.behind == 0 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
        // The sky's weight to draw with, if asked (`FAR_TEST_SKY_WEIGHT`, for a look at the skyline).
        if let Some(weight) = std::env::var("FAR_TEST_SKY_WEIGHT").ok().and_then(|w| w.parse().ok()) {
            eye.look.sky_weight = weight;
        }
        // Where the rays' steps begin, if asked (`FAR_TEST_START`: the pattern, then the weave's bits, the blue noise
        // in the weave and the spread, as many as given).
        let start_asked: Vec<f32> = std::env::var("FAR_TEST_START").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        if let Some(&v) = start_asked.first() {
            eye.look.start_pattern = v as u32;
        }
        if let Some(&v) = start_asked.get(1) {
            eye.look.weave_bits = v as u32;
        }
        if let Some(&v) = start_asked.get(2) {
            eye.look.start_blue = v;
        }
        if let Some(&v) = start_asked.get(3) {
            eye.look.start_spread = v;
        }
        // The picture's size (640 x 400, or `FAR_TEST_SCREEN`: across, down).
        let screen_asked: Vec<u32> = std::env::var("FAR_TEST_SCREEN").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        let screen = if screen_asked.len() == 2 { (screen_asked[0], screen_asked[1]) } else { (640u32, 400u32) };
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("the far eye's test picture"),
            size: wgpu::Extent3d { width: screen.0, height: screen.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let row = (screen.0 * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the picture read back"), size: (row * screen.1) as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        // A frame, or as many as asked (`FAR_TEST_FRAMES`; by the one dispatch a finer layer's lead is the frame
        // before's, and the first frame makes its own); the last one's picture read back.
        let frames = std::env::var("FAR_TEST_FRAMES").ok().and_then(|n| n.parse::<u32>().ok()).unwrap_or(1).max(1);
        for _ in 1..frames {
            eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
            eye.start(&mut encoder, None);
            for k in 0..LAYERS.len() {
                eye.layer(k, &mut encoder, None);
            }
            queue.submit([encoder.finish()]);
        }
        eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
        eye.start(&mut encoder, None);
        for k in 0..LAYERS.len() {
            eye.layer(k, &mut encoder, None);
        }
        eye.bring_up(&mut encoder, &view, None);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: None } },
            wgpu::Extent3d { width: screen.0, height: screen.1, depth_or_array_layers: 1 },
        );
        queue.submit([encoder.finish()]);
        let error = pollster::block_on(errors.pop());
        assert!(error.is_none(), "the card refused the far eye's work: {error:?}");
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let pixels = readback.slice(..).get_mapped_range().unwrap().to_vec();
        // The average colour of a band of rows.
        let average = |rows: std::ops::Range<u32>| {
            let mut sum = [0u64; 3];
            for y in rows.clone() {
                for x in 0..screen.0 {
                    let at = (y * row + x * 4) as usize;
                    for c in 0..3 {
                        sum[c] += pixels[at + c] as u64;
                    }
                }
            }
            let n = (rows.len() as u32 * screen.0) as u64;
            sum.map(|s| s / n)
        };
        // Kept as raw pixels for a look, if asked (`FAR_TEST_PICTURE`: a folder).
        if let Some(folder) = std::env::var_os("FAR_TEST_PICTURE") {
            let mut raw = Vec::new();
            for y in 0..screen.1 {
                raw.extend_from_slice(&pixels[(y * row) as usize..(y * row + screen.0 * 4) as usize]);
            }
            let _ = std::fs::write(std::path::Path::new(&folder).join(format!("far eye {} {}x{} weight {}{} blend {} detail {}", mark.name, screen.0, screen.1, eye.look.sky_weight, if std::env::var_os("INCUS_SIGHT_NO_SHADOWS").is_some() { " unlit" } else { "" }, eye.look.blend, eye.look.detail) + &std::env::var("FAR_TEST_START").map_or(String::new(), |s| format!(" start {s}")) + &std::env::var("FAR_TEST_KINDS").map_or(String::new(), |s| format!(" kinds {s}")) + &std::env::var("FAR_TEST_LEVELS").map_or(String::new(), |s| format!(" levels {s}")) + &std::env::var("FAR_TEST_VARIANT").map_or(String::new(), |s| format!(" variant {s}")) + if eye_asked.is_some() { " eye" } else { "" } + ".rgba"), raw);
        }
        let (top, bottom) = (average(0..20), average(screen.1 - 20..screen.1));
        eprintln!("The far eye's test picture: the top rows {top:?}, the bottom rows {bottom:?} (red, green, blue of 255)");
        eprintln!("The far eye's state: {}", eye.state());
        let memory = device.generate_allocator_report().map(|r| (r.total_allocated_bytes >> 20, r.total_reserved_bytes >> 20));
        eprintln!("The card's memory for nine regions' far eye: {memory:?} MB (in use, held)");
        assert!(memory.is_some(), "the card says what memory the game holds");
        if mark.name == marks[0].name {
            assert!(top[2] > top[0] && top[2] > 100, "the top should be sky, blue: {top:?}");
        }
        assert_ne!(top, bottom, "the bottom should be ground, not the same as the sky");
    }
}

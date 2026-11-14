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
const CAMERA_ROWS: usize = 18;

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
}

pub struct FarEye {
    pub store: Store,
    lane: Option<Lane>,
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
    parity: usize,
    fresh: bool,
    /// A ray's step through thin haze, in cells (the lean step, 1d; `INCUS_FAR_HAZE_STEP`): a whole cell (20% off the
    /// sea, 13% off a player's run; David, looking at build 151: "i don't see anything bad about it. Seems like a good
    /// thing"), or old's half (0.5).
    haze_step: f32,
    most_steps: f32,
    /// The longest ray (the Oracle's step 0): the pass that counts a layer's rays' steps, each layer's four counts on the
    /// card, where they're read back, and whether a reading is free (0), on its way (1) or ready (2).
    tally: wgpu::ComputePipeline,
    tally_counts: Vec<wgpu::Buffer>,
    tally_back: wgpu::Buffer,
    tally_state: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

impl FarEye {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, store: Store, lane: Option<Lane>) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye"), source: wgpu::ShaderSource::Wgsl(include_str!("far.wgsl").into()) });
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
        let edge = std::env::var("INCUS_FAR_EDGE").as_deref() != Ok("0");
        let haze_step = std::env::var("INCUS_FAR_HAZE_STEP").ok().and_then(|s| s.parse::<f32>().ok()).filter(|s| (0.25..=2.0).contains(s)).unwrap_or(1.0);
        let base = if std::env::var("INCUS_FAR_BASE").as_deref() == Ok("4") { 4.0 } else { LAYERS[0] };
        let layers = [base, base / 2.0, base / 4.0];
        // The one dispatch: `far.wgsl` with one layer's outputs changed for every layer's (`far_all.wgsl`), and its
        // camera read in place from its layer's row.
        let one = (std::env::var("INCUS_FAR_SIDE").as_deref() != Ok("0")).then(|| {
            let whole = include_str!("far.wgsl");
            let (begin, end) = ("// ── One layer's outputs: begin ──", "// ── One layer's outputs: end ──");
            let (from, to) = (whole.find(begin).expect("the outputs' first mark"), whole.find(end).expect("the outputs' last mark"));
            let source = format!("{}{}{}", &whole[..from], include_str!("far_all.wgsl"), &whole[to..]).replace("camera.", "cameras[layer_now].").replace("@group(0) @binding(0) var<uniform> camera: Camera;", "@group(0) @binding(0) var<uniform> cameras: array<Camera, 3>;");
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
                    written(17, wgpu::TextureFormat::Rgba8Unorm),
                    written(19, wgpu::TextureFormat::Rgba8Unorm),
                ],
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the far eye, one dispatch"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("march_all"), layout: Some(&pipeline_layout), module: &module, entry_point: Some("march_all"), compilation_options: Default::default(), cache: None });
            (pipeline, layout)
        });
        let most_steps = std::env::var("INCUS_TEST_STEPS").ok().and_then(|n| n.parse::<f32>().ok()).filter(|n| *n >= 1.0).unwrap_or(768.0);
        log::info!("The far rays: {}, {most_steps:.0} steps at most", if edge { "ended at the map's edge" } else { "marched on past the map's edge (INCUS_FAR_EDGE=0)" });
        let tally_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the longest ray"), source: wgpu::ShaderSource::Wgsl(include_str!("far_tally.wgsl").into()) });
        let tally = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("the longest ray"), layout: None, module: &tally_module, entry_point: Some("tally_pass"), compilation_options: Default::default(), cache: None });
        let tally_counts = LAYERS.iter().map(|_| device.create_buffer(&wgpu::BufferDescriptor { label: Some("a far layer's longest ray"), size: 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })).collect();
        let tally_back = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the longest rays read back"), size: 16 * LAYERS.len() as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let tally_state = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));
        Self { store, lane, start, march, up, layout, up_layout, sampler, targets: None, look, textures, atlas: None, building: None, stand_ins, light: None, lighting: None, sweeping: None, sun_asked, light_stand_ins, edge, layers, one, parity: 0, fresh: false, haze_step, most_steps, tally, tally_counts, tally_back, tally_state }
    }

    /// Whether every region is on the card, the window's regions round the eye too, and the far textures if they're made.
    pub fn whole(&self) -> bool {
        self.lane.as_ref().is_none_or(|lane| lane.done == lane.total) && (self.atlas.is_some() || !self.textures) && self.store.behind == 0 && self.lit()
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

    /// The look's settings a flight's numbers depend on, for its report.
    pub fn settings(&self) -> String {
        let window = self.store.window().map_or("the whole world on the card".to_string(), |w| format!("the 32s within {} regions of the eye, the 64s within {}", w[0], w[1]));
        let mut rays = if self.edge { "rays ended at the map's edge".to_string() } else { "rays marched past the map's edge".to_string() };
        rays += &format!(", thin haze stepped by {} of a cell; the layers {} x {}, {} x {} and {} x {} screen pixels", self.haze_step, self.layers[0], self.layers[0], self.layers[1], self.layers[1], self.layers[2], self.layers[2]);
        rays += ", light read from the cell";
        rays += if self.one.is_some() { ", marched by one dispatch (a finer layer led by the frame before's coarser)" } else { ", marched one after another" };
        if self.most_steps != 768.0 {
            rays += &format!(", {:.0} steps at most", self.most_steps);
        }
        format!("bricks of {}, {window}, {rays}, blended by {}, detail {:+}", self.store.side, if self.look.blend == 1 && self.atlas.is_some() { "the hardware (spread)" } else { "the code (old's)" }, self.look.detail)
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
        let light = self.light.as_ref().map_or(String::new(), |l| format!("; the far light {:.0} MB, its tops and sky made in {:.2} s, its last shadow swept and sent in {:.3} s", l.bytes as f64 / 1e6, l.made_in, l.swept_in));
        format!("{lane}{} bricks, {:.0} MB of {:.0} MB{atlas}{light}", self.store.bricks, used as f64 / 1e6, size as f64 / 1e6)
    }

    /// The regions summed since the last frame put on the card, and the window brought to the eye, for at most
    /// `PUTTING_MS` together (the pass "putting the far cells on the card").
    pub fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, eye: [f32; 2]) {
        let started = Instant::now();
        // The atlas, once every region is in: built on a thread of its own from the cells kept and the window as it is
        // (held still till it's in), then taken in here.
        if self.lane.is_none() && self.atlas.is_none() && self.building.is_none() && self.textures {
            let kept = self.store.kept.clone();
            let window = self.store.window_places();
            let (device, queue) = (device.clone(), queue.clone());
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("making the far textures".into()).spawn(move || {
                let keep = |b: &crate::atlas::Brick| window.as_ref().is_none_or(|places| b.level >= crate::far::FIRST + crate::bricks::WINDOWED || places.contains(&b.place));
                let _ = sender.send(crate::atlas::build(&device, &queue, &kept, keep));
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
                atlas.remove(&moved.left);
                atlas.add(device, queue, &self.store.kept, &moved.came);
            }
        }
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
                        entry(17, wgpu::BindingResource::TextureView(&layers[1].picture)),
                        entry(19, wgpu::BindingResource::TextureView(&layers[2].picture)),
                    ],
                })
            });
            (cameras, groups)
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
        self.targets = Some(Targets { screen, generation: self.store.generation, atlas_in: self.atlas.is_some(), atlas_generation, light: light_now, tiles, tally_groups, tally_parts, layers, up_numbers, up_group, one });
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
                [REACH, -look.detail as f32, look.view as f32, self.store.highest],
                [targets.tiles.0 as f32, targets.tiles.1 as f32, (TILE as f32 / layer.factor).max(1.0), screen_pixel * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
                [above.map_or(0.0, |a| a.size.0 as f32), if below.is_some() { 1.0 } else { 0.0 }, 1.0, blend as f32],
                [1.0, if look.shadows && direct > 0.0 { 1.0 } else { 0.0 }, 0.0, if self.edge { 1.0 } else { 0.0 }],
                [above.map_or(0.0, |a| layer.factor / a.factor), below.map_or(0.0, |b| b.factor / layer.factor), self.most_steps, 0.0],
                [fade[0].max(0.1), fade[1].max(0.05), look.hand_over.clamp(0.3, 0.95), look.sky_weight.max(0.0)],
                [if look.sky_light { 1.0 } else { 0.0 }, look.sky_strength.clamp(0.0, 1.0), look.sky_tint.clamp(0.0, 1.0), 0.0],
                [if look.fog { 3.0 / look.fog_reach.max(10.0) } else { 0.0 }, std::f32::consts::LN_2 / look.fog_height.max(1.0), sea_level, look.fog_warmth.clamp(0.0, 1.0)],
                [direct, NIGHT_LIGHT + (1.0 - NIGHT_LIGHT) * glow, glow, NIGHT_LIGHT],
                [water[0], water[1], water[2], 20f32.ln() / WATER_CLARITY],
                [atlas_size[0], atlas_size[1], atlas_size[2], self.atlas.as_ref().map_or(1.0, |a| a.side as f32)],
                [window_levels, regions.1 as f32, regions.0[0] as f32, regions.0[1] as f32],
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
                pass.set_pipeline(pipeline);
                let runs = if self.fresh { 3 } else { 1 };
                for run in 0..runs {
                    let copy = if (runs - 1 - run) % 2 == 0 { self.parity } else { 1 - self.parity };
                    pass.set_bind_group(0, &groups[copy], &[]);
                    pass.dispatch_workgroups(base.0 + middle.0 + fine.0, fine.1.max(middle.1).max(base.1), 1);
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

#[cfg(test)]
mod tests {
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
        let mut eye = FarEye::new(&device, format, store, lane);
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
        // The far light made first too (read from the cell).
        while !eye.lit() {
            eye.put(&device, &queue, at);
            queue.submit([]);
            let _ = device.poll(wgpu::PollType::Poll);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        // The sky's weight to draw with, if asked (`FAR_TEST_SKY_WEIGHT`, for a look at the skyline).
        if let Some(weight) = std::env::var("FAR_TEST_SKY_WEIGHT").ok().and_then(|w| w.parse().ok()) {
            eye.look.sky_weight = weight;
        }
        let screen = (640u32, 400u32);
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
            let _ = std::fs::write(std::path::Path::new(&folder).join(format!("far eye {} {}x{} weight {}{} blend {} detail {}.rgba", mark.name, screen.0, screen.1, eye.look.sky_weight, if std::env::var_os("INCUS_SIGHT_NO_SHADOWS").is_some() { " unlit" } else { "" }, eye.look.blend, eye.look.detail)), raw);
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

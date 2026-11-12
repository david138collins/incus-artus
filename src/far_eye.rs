//! The far eye (plan 2.2, step 2a): old's far march on bricks, drawn each frame under the panels.
//!
//! The frame's card passes, each timed on its own: the start pass (one cone a tile of 8 × 8 screen pixels, finding
//! where its rays can start), the pyramid's layers (8 × 8, 4 × 4 and 2 × 2: each finer one only where the one above
//! says it's worth it, never for sky), then bringing it up to the screen. Old's settings for the dyno, carried as they
//! were: a field of view of 100° (up and down), the sun from 315° at 40° (noon), fog on, the hand-over from 0.8, the
//! finer layers' fades as old's preset had them, the layers brought up crisp.
//!
//! The far cells come from the loading lane (`far::Lane`), nearest the eye first, and go on the card as they come
//! (`bricks::Store`), so the view sharpens in live, raw, with no loading screen [David, 2026-11-06: "id like to see the
//! sharpening myself, realtime and raw"].

use crate::bricks::Store;
use crate::camera::Camera;
use crate::far::Lane;
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
/// Sky light (2b; its tint is used in 2a too, as old's was with sky light off): strength, tint, its cones' width.
const SKY_STRENGTH: f32 = 0.8;
const SKY_TINT: f32 = 0.3;
const SKY_SPREAD: f32 = 0.6;
/// How fast a shadow ray widens (2b).
const SHADOW_SPREAD: f32 = 0.04;
/// Fog: land this far off at sea level is nineteen parts in twenty fog; the air's thickness halves every this many blocks
/// up; a little warmer towards the sun (old's).
const FOG_REACH: f32 = 3200.0;
const FOG_HEIGHT: f32 = 200.0;
const FOG_WARMTH: f32 = 0.3;
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
const CAMERA_ROWS: usize = 16;

/// A layer of the pyramid on the card.
struct Layer {
    factor: f32,
    size: (u32, u32),
    picture: wgpu::TextureView,
    far: wgpu::TextureView,
    camera: wgpu::Buffer,
    group: wgpu::BindGroup,
}

/// What's sized to the screen (made again when the screen's size changes, or the bricks' pool grows).
struct Targets {
    screen: (u32, u32),
    generation: u32,
    tiles: (u32, u32),
    layers: Vec<Layer>,
    up_numbers: wgpu::Buffer,
    up_group: wgpu::BindGroup,
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
    /// 1 the steps' view, 2 the levels' view (`INCUS_SIGHT_STEPS`, `INCUS_SIGHT_LEVELS`), 0 neither.
    view_mode: f32,
}

impl FarEye {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, store: Store, lane: Option<Lane>) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye"), source: wgpu::ShaderSource::Wgsl(include_str!("far.wgsl").into()) });
        let compute = wgpu::ShaderStages::COMPUTE;
        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry { binding, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let written = |binding: u32, format: wgpu::TextureFormat| wgpu::BindGroupLayoutEntry { binding, visibility: compute, ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::WriteOnly, format, view_dimension: wgpu::TextureViewDimension::D2 }, count: None };
        let read = |binding: u32, visibility: wgpu::ShaderStages, filterable: bool| wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let uniform = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the far eye"),
            entries: &[
                uniform(0, compute),
                storage(1, true),
                storage(2, true),
                storage(3, true),
                storage(4, true),
                written(5, wgpu::TextureFormat::Rgba8Unorm),
                written(6, wgpu::TextureFormat::R32Float),
                storage(7, false),
                storage(8, true),
                storage(9, false),
                read(10, compute, false),
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
        let view_mode = if std::env::var_os("INCUS_SIGHT_LEVELS").is_some() {
            2.0
        } else if std::env::var_os("INCUS_SIGHT_STEPS").is_some() {
            1.0
        } else {
            0.0
        };
        Self { store, lane, start, march, up, layout, up_layout, sampler, targets: None, view_mode }
    }

    /// Whether every region is on the card.
    pub fn whole(&self) -> bool {
        self.lane.as_ref().is_none_or(|lane| lane.done == lane.total)
    }

    /// How the far cells are coming in, for the panel and the flight's report.
    pub fn state(&self) -> String {
        let (used, size) = self.store.pool_bytes();
        let lane = self.lane.as_ref().map_or(String::new(), |lane| {
            let eighth = lane.eighth.map_or(String::new(), |s| format!(", the eighth nearest in {s:.1} s"));
            format!("{} of {} regions in{eighth}; ", lane.done, lane.total)
        });
        format!("{lane}{} bricks, {:.0} MB of {:.0} MB", self.store.bricks, used as f64 / 1e6, size as f64 / 1e6)
    }

    /// The regions summed since the last frame put on the card, for at most `PUTTING_MS` (the pass "putting the far
    /// cells on the card").
    pub fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let Some(lane) = &mut self.lane else { return };
        let started = Instant::now();
        while started.elapsed().as_secs_f32() * 1000.0 < PUTTING_MS {
            let taken = lane.take(1);
            let Some((region, columns)) = taken.into_iter().next() else { break };
            self.store.put_region(device, queue, region, &columns);
        }
        if lane.done == lane.total && self.store.regions_in == lane.total {
            let (used, _) = self.store.pool_bytes();
            log::info!("The far bricks: all {} regions on the card, {} bricks, {:.0} MB of cells", lane.total, self.store.bricks, used as f64 / 1e6);
            self.lane = None;
        }
    }

    /// The layers sized to the screen, and their bind groups made with the bricks as they are.
    fn targets(&mut self, device: &wgpu::Device, screen: (u32, u32)) {
        if self.targets.as_ref().is_some_and(|t| t.screen == screen && t.generation == self.store.generation) {
            return;
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
        let no_far = texture("the far layers' no distances above", (1, 1), wgpu::TextureFormat::R32Float);
        let mut layers: Vec<Layer> = Vec::new();
        let mut worth_above: Option<wgpu::Buffer> = None;
        for factor in LAYERS {
            let size = ((screen.0 as f32 / factor).ceil().max(1.0) as u32, (screen.1 as f32 / factor).ceil().max(1.0) as u32);
            let picture = texture("a far layer", size, wgpu::TextureFormat::Rgba8Unorm);
            let far = texture("a far layer's distances", size, wgpu::TextureFormat::R32Float);
            let worth = buffer("a far layer's worth it", (size.0 * size.1) as u64 * 4, storage);
            let camera = buffer("a far layer's camera", CAMERA_ROWS as u64 * 16, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
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
                    entry(6, wgpu::BindingResource::TextureView(&far)),
                    entry(7, starts.as_entire_binding()),
                    entry(8, worth_above.as_ref().unwrap_or(&nothing_above).as_entire_binding()),
                    entry(9, worth.as_entire_binding()),
                    entry(10, wgpu::BindingResource::TextureView(layers.last().map_or(&no_far, |above| &above.far))),
                ],
            });
            worth_above = Some(worth);
            layers.push(Layer { factor, size, picture, far, camera, group });
        }
        let up_numbers = buffer("bringing it up's numbers", 32, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let view = |k: usize| wgpu::BindingResource::TextureView(&layers[k].picture);
        let up_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bringing it up"),
            layout: &self.up_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: view(0) },
                wgpu::BindGroupEntry { binding: 1, resource: view(1) },
                wgpu::BindGroupEntry { binding: 2, resource: view(2) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&layers[0].far) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 5, resource: up_numbers.as_entire_binding() },
            ],
        });
        self.targets = Some(Targets { screen, generation: self.store.generation, tiles, layers, up_numbers, up_group });
    }

    /// The frame's numbers for each layer and for bringing it up, from the camera (done before the card's passes are
    /// recorded).
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, screen: (u32, u32), camera: &Camera, sea_level: f32) {
        self.targets(device, screen);
        let Some(targets) = &self.targets else { return };
        let (facing, pitch) = (camera.facing.to_radians(), camera.pitch.to_radians());
        let forward = [pitch.cos() * facing.cos(), pitch.cos() * facing.sin(), pitch.sin()];
        let right = [-facing.sin(), facing.cos(), 0.0];
        let up = [-pitch.sin() * facing.cos(), -pitch.sin() * facing.sin(), pitch.cos()];
        let tan_half = (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let screen_pixel = 2.0 * tan_half / screen.1.max(1) as f32;
        let (from, height) = (SUN_FROM.to_radians(), SUN_HEIGHT.to_radians());
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
            let fade = below.map_or([1.0, 1.0], |_| CELLS_FADES[k.min(CELLS_FADES.len() - 1)]);
            let numbers: [[f32; 4]; CAMERA_ROWS] = [
                [camera.position.x, camera.position.y, camera.position.z, 0.0],
                [forward[0], forward[1], forward[2], 0.0],
                [right[0] * tan_half * aspect, right[1] * tan_half * aspect, right[2] * tan_half * aspect, 0.0],
                [up[0] * tan_half, up[1] * tan_half, up[2] * tan_half, 0.0],
                [layer.size.0 as f32, layer.size.1 as f32, pixel, 1.0],
                [sun[0], sun[1], sun[2], AMBIENT],
                [REACH, 0.0, self.view_mode, self.store.highest],
                [targets.tiles.0 as f32, targets.tiles.1 as f32, (TILE as f32 / layer.factor).max(1.0), screen_pixel * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
                [above.map_or(0.0, |a| a.size.0 as f32), if below.is_some() { 1.0 } else { 0.0 }, 1.0, 0.0],
                [1.0, 0.0, SHADOW_SPREAD, 0.0],
                [above.map_or(0.0, |a| layer.factor / a.factor), below.map_or(0.0, |b| b.factor / layer.factor), 0.0, 0.0],
                [fade[0].max(0.1), fade[1].max(0.05), BLEND_FROM, 0.0],
                [0.0, SKY_STRENGTH, SKY_TINT, SKY_SPREAD],
                [3.0 / FOG_REACH, std::f32::consts::LN_2 / FOG_HEIGHT, sea_level, FOG_WARMTH],
                [direct, NIGHT_LIGHT + (1.0 - NIGHT_LIGHT) * glow, glow, NIGHT_LIGHT],
                [water[0], water[1], water[2], 20f32.ln() / WATER_CLARITY],
            ];
            queue.write_buffer(&layer.camera, 0, &numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        }
        let finest = LAYERS[LAYERS.len() - 1];
        let up_numbers: [f32; 8] = [screen.0 as f32, screen.1 as f32, finest, LAYERS[0], 0.0, 0.0, 0.0, 0.0];
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

    /// Layer `k` of the pyramid (0 the base, 8 × 8).
    pub fn layer(&self, k: usize, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) {
        let Some(targets) = &self.targets else { return };
        let layer = &targets.layers[k];
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("a far layer"), timestamp_writes: timestamps });
        pass.set_pipeline(&self.march);
        pass.set_bind_group(0, &layer.group, &[]);
        pass.dispatch_workgroups(layer.size.0.div_ceil(8), layer.size.1.div_ceil(8), 1);
    }

    /// The pyramid brought up onto the screen's picture.
    pub fn bring_up(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let Some(targets) = &self.targets else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("bringing it up"), color_attachments: &[Some(attachment)], timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.up);
        pass.set_bind_group(0, &targets.up_group, &[]);
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
        let mut store = Store::new(&device, &queue, regions_of(&doorway), doorway.height());
        for ry in -1..=1 {
            for rx in -1..=1 {
                store.put_region(&device, &queue, [rx, ry], &region(&doorway, &colours, [rx, ry]).unwrap());
            }
        }
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let mut eye = FarEye::new(&device, format, store, None);
        let marks = crate::marks::load().unwrap();
        let mut camera = Camera::new();
        camera.position = crate::marks::eye(&doorway, &marks[0]).unwrap();
        let screen = (640u32, 400u32);
        eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
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
            let _ = std::fs::write(std::path::Path::new(&folder).join(format!("far eye {} {}x{}.rgba", marks[0].name, screen.0, screen.1)), raw);
        }
        let (top, bottom) = (average(0..20), average(screen.1 - 20..screen.1));
        eprintln!("The far eye's test picture: the top rows {top:?}, the bottom rows {bottom:?} (red, green, blue of 255)");
        assert!(top[2] > top[0] && top[2] > 100, "the top should be sky, blue: {top:?}");
        assert_ne!(top, bottom, "the bottom should be ground, not the same as the sky");
    }
}

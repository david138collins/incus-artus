//! Sight drawn (the regroup, phase 3.2, the first sight; the screen pyramid, David 2026-11-06): the world
//! marched through the store on the card (`march.wgsl`) in a pyramid of layers, 8 × 8 screen pixels to a pixel
//! everywhere, then 4 × 4, 2 × 2 and 1 × 1 only where the layer above says they're worth it, each finer ray starting at
//! its tile's start (the start pass), then brought up to the screen (`upscale.wgsl`), each screen pixel showing
//! the finest layer marched there. The runner calls `frame` before the panel is drawn over it.

use super::card_store::CardStore;
use std::sync::Arc;

/// What the frame hands sight: the eye and where it looks, and the settings.
#[derive(Clone, Copy, Debug)]
pub struct Eye {
    pub position: [f32; 3],
    pub looking: [f32; 3],
    /// The field of view, up and down (degrees).
    pub field_of_view: f32,
    /// The finest layer the pyramid may go to (screen pixels a side of its pixels: 8, 4, 2 or 1); rays a pixel;
    /// the upscale's mode; and whether the finer layers are marched only where they're worth it (else
    /// everywhere).
    pub grid: u32,
    /// The coarsest layer, where the pyramid starts (8, 4, 2 or 1; everything is marched there) [David
    /// 2026-11-08: "a set of selections for coarsest rays, so i can experiment with nixing the 8x8 layer"].
    pub coarsest: u32,
    pub rays: u32,
    pub upscale: u32,
    pub earned: bool,
    /// Towards the sun (game axes, a whole way long): the Render page's sun, the one the World Compiler's
    /// pictures are lit by too (`KeptSettings::sun`).
    pub sun: [f32; 3],
    /// Sun shadows (3.4b; the Render page's switch).
    pub shadows: bool,
}

/// How fast a shadow ray widens (blocks across a block of distance, 3.4b): wider than the sun itself (about
/// 0.01), so a shadow ray's steps grow with the distance and a far ridge's shadow comes soft.
const SHADOW_SPREAD: f32 = 0.04;

/// The first sight's noon sun, high in the south, a little east ("a fixed noon sun"): the tests' sun.
pub const NOON: [f32; 3] = [-0.35, 0.2, 0.91];
const AMBIENT: f32 = 0.35;
/// The start pass's tile, and the pyramid's base: screen pixels a side.
const TILE: u32 = 8;
/// The farthest a ray goes (blocks): past any map's far corner from anywhere in it.
const REACH: f32 = 150_000.0;

pub struct SightDraw {
    march: wgpu::ComputePipeline,
    start: wgpu::ComputePipeline,
    upscale: wgpu::RenderPipeline,
    up: wgpu::Buffer,
    sampler: wgpu::Sampler,
    targets: Option<Targets>,
}

/// One layer of the pyramid: its factor (screen pixels a side of a pixel), its size, its picture and distances,
/// its camera numbers, and its bindings.
struct Layer {
    factor: u32,
    size: (u32, u32),
    picture: wgpu::TextureView,
    far: wgpu::TextureView,
    camera: wgpu::Buffer,
    group: wgpu::BindGroup,
}

struct Targets {
    screen: (u32, u32),
    finest: u32,
    coarsest: u32,
    store: Arc<CardStore>,
    /// The start pass's tiles across and down.
    tiles: (u32, u32),
    layers: Vec<Layer>,
    start_group: wgpu::BindGroup,
    upscale_group: wgpu::BindGroup,
}

fn buffer(b: &wgpu::Buffer) -> wgpu::BindingResource<'_> {
    b.as_entire_binding()
}

impl SightDraw {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let march_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("sight's march"), source: wgpu::ShaderSource::Wgsl(include_str!("march.wgsl").into()) });
        let march = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("sight's march"), layout: None, module: &march_module, entry_point: Some("march_pass"), compilation_options: Default::default(), cache: None });
        let start = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("sight's start pass"), layout: None, module: &march_module, entry_point: Some("start_pass"), compilation_options: Default::default(), cache: None });
        let up_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("sight's upscale"), source: wgpu::ShaderSource::Wgsl(include_str!("upscale.wgsl").into()) });
        let upscale = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sight's upscale"),
            layout: None,
            vertex: wgpu::VertexState { module: &up_module, entry_point: Some("whole_screen"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &up_module, entry_point: Some("brought_up"), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let up = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's upscale numbers"), size: 2 * 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("sight's blend"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });
        Self { march, start, upscale, up, sampler, targets: None }
    }

    /// The pyramid's layers for a screen of `screen`, from `coarsest` down to `finest`, and their bindings to
    /// `store`, made again when any of them changes (a moment's work: the store is the world's, not the screen's).
    fn targets(&mut self, device: &wgpu::Device, screen: (u32, u32), finest: u32, coarsest: u32, store: &Arc<CardStore>) -> &Targets {
        let stale = self.targets.as_ref().is_none_or(|t| t.screen != screen || t.finest != finest || t.coarsest != coarsest || !Arc::ptr_eq(&t.store, store));
        if stale {
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
                    .create_view(&Default::default())
            };
            let storage = |label: &str, words: u32| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: words.max(1) as u64 * 4, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false });
            let tiles = (screen.0.div_ceil(TILE), screen.1.div_ceil(TILE));
            let starts = storage("sight's starts", tiles.0 * tiles.1);
            // The base has no layer above: its "worth it" in is a placeholder never read.
            let nothing_above = storage("sight's nothing above", 1);
            let no_distances_above = texture("sight's no distances above", (1, 1), wgpu::TextureFormat::R32Float);
            let mut factors = vec![coarsest];
            while factors.last().is_some_and(|&f| f > finest.max(1)) {
                let next = factors.last().copied().unwrap_or(coarsest) / 2;
                factors.push(next);
            }
            let mut layers: Vec<Layer> = Vec::new();
            let mut worth_above: Option<wgpu::Buffer> = None;
            let mut far_above: Option<wgpu::TextureView> = None;
            for &factor in &factors {
                let size = (screen.0.div_ceil(factor).max(1), screen.1.div_ceil(factor).max(1));
                let picture = texture("sight's layer", size, wgpu::TextureFormat::Rgba8Unorm);
                let far = texture("sight's layer's distances", size, wgpu::TextureFormat::R32Float);
                let worth = storage("sight's worth it", size.0 * size.1);
                let camera = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's camera"), size: 10 * 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("sight's march"),
                    layout: &self.march.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: buffer(&camera) },
                        wgpu::BindGroupEntry { binding: 1, resource: buffer(&store.header) },
                        wgpu::BindGroupEntry { binding: 2, resource: buffer(&store.baked) },
                        wgpu::BindGroupEntry { binding: 3, resource: buffer(&store.live) },
                        wgpu::BindGroupEntry { binding: 4, resource: buffer(&store.pool) },
                        wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&picture) },
                        wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&far) },
                        wgpu::BindGroupEntry { binding: 7, resource: buffer(&starts) },
                        wgpu::BindGroupEntry { binding: 8, resource: buffer(worth_above.as_ref().unwrap_or(&nothing_above)) },
                        wgpu::BindGroupEntry { binding: 9, resource: buffer(&worth) },
                        wgpu::BindGroupEntry { binding: 10, resource: wgpu::BindingResource::TextureView(far_above.as_ref().unwrap_or(&no_distances_above)) },
                    ],
                });
                far_above = Some(far.clone());
                layers.push(Layer { factor, size, picture, far, camera, group });
                worth_above = Some(worth);
            }
            let start_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sight's start pass"),
                layout: &self.start.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buffer(&layers[0].camera) },
                    wgpu::BindGroupEntry { binding: 1, resource: buffer(&store.header) },
                    wgpu::BindGroupEntry { binding: 2, resource: buffer(&store.baked) },
                    wgpu::BindGroupEntry { binding: 3, resource: buffer(&store.live) },
                    wgpu::BindGroupEntry { binding: 7, resource: buffer(&starts) },
                ],
            });
            // A layer the pyramid doesn't go to is never read; the base stands in for it.
            let picture_of = |factor: u32| &layers.iter().find(|l| l.factor == factor).unwrap_or(&layers[0]).picture;
            let upscale_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sight's upscale"),
                layout: &self.upscale.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(picture_of(8)) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(picture_of(4)) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(picture_of(2)) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&layers[0].far) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 5, resource: buffer(&self.up) },
                    wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(picture_of(1)) },
                ],
            });
            self.targets = Some(Targets { screen, finest, coarsest, store: store.clone(), tiles, layers, start_group, upscale_group });
        }
        self.targets.as_ref().expect("made just now")
    }

    /// The pyramid marched and brought up onto `view` (the screen's picture, `screen` pixels).
    pub fn frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, screen: (u32, u32), eye: Eye, store: &Arc<CardStore>) {
        let factor = |n: u32| match n {
            0..=1 => 1,
            2 => 2,
            3..=4 => 4,
            _ => 8,
        };
        let coarsest = factor(eye.coarsest);
        // Never coarser than the coarsest.
        let finest = factor(eye.grid).min(coarsest);
        let tiles = self.targets(device, screen, finest, coarsest, store).tiles;
        // The camera: the eye, and the directions of the screen's edges (game axes: looking north, east is right).
        let forward = normalise(eye.looking);
        let right = normalise(cross([0.0, 0.0, 1.0], forward));
        let up = cross(forward, right);
        let tan_half = (eye.field_of_view.to_radians() / 2.0).tan();
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let no_start = std::env::var("INCUS_SIGHT_NO_START").is_ok();
        let no_follow = std::env::var("INCUS_SIGHT_NO_FOLLOW").is_ok();
        let no_shadows = std::env::var("INCUS_SIGHT_NO_SHADOWS").is_ok();
        let measuring = if std::env::var("INCUS_SIGHT_STEPS").is_ok() {
            1.0
        } else if std::env::var("INCUS_SIGHT_LEVELS").is_ok() {
            2.0
        } else {
            0.0
        };
        // How far the live level reaches (blocks): half its window.
        let live_reach = store.reach(store.finest_live);
        let screen_pixel = 2.0 * tan_half / screen.1.max(1) as f32;
        let targets = self.targets.as_ref().expect("made above");
        for (k, layer) in targets.layers.iter().enumerate() {
            let pixel = 2.0 * tan_half / layer.size.1 as f32;
            let above_width = if k == 0 { 0.0 } else { targets.layers[k - 1].size.0 as f32 };
            let below = if k + 1 < targets.layers.len() { 1.0 } else { 0.0 };
            let numbers: [[f32; 4]; 10] = [
                [eye.position[0], eye.position[1], eye.position[2], 0.0],
                [forward[0], forward[1], forward[2], 0.0],
                [right[0] * tan_half * aspect, right[1] * tan_half * aspect, right[2] * tan_half * aspect, 0.0],
                [up[0] * tan_half, up[1] * tan_half, up[2] * tan_half, 0.0],
                [layer.size.0 as f32, layer.size.1 as f32, pixel, eye.rays.max(1) as f32],
                {
                    let sun = normalise(eye.sun);
                    [sun[0], sun[1], sun[2], AMBIENT]
                },
                [REACH, 0.0, measuring, store.highest.load(std::sync::atomic::Ordering::Relaxed) as f32],
                // The start pass (`INCUS_SIGHT_NO_START` leaves it out, for measuring): tiles, a tile's pixels in
                // this layer, and the cone's spread (a tile's half diagonal, and a screen pixel more).
                [tiles.0 as f32, tiles.1 as f32, if no_start { 0.0 } else { (TILE / layer.factor).max(1) as f32 }, screen_pixel * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
                [above_width, below, if eye.earned { 1.0 } else { 0.0 }, live_reach],
                // Each finer ray from short of its parent's nearest touch (`INCUS_SIGHT_NO_FOLLOW` leaves it out);
                // sun shadows (`INCUS_SIGHT_NO_SHADOWS` leaves them out, for measuring), and their widening.
                [if no_follow { 0.0 } else { 1.0 }, if eye.shadows && !no_shadows { 1.0 } else { 0.0 }, SHADOW_SPREAD, 0.0],
            ];
            queue.write_buffer(&layer.camera, 0, &numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        }
        let up_numbers: [[f32; 4]; 2] = [[screen.0 as f32, screen.1 as f32, finest as f32, coarsest as f32], [eye.upscale as f32, 0.0, 0.0, 0.0]];
        queue.write_buffer(&self.up, 0, &up_numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        if !no_start {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("sight's start pass"), timestamp_writes: None });
            pass.set_pipeline(&self.start);
            pass.set_bind_group(0, &targets.start_group, &[]);
            pass.dispatch_workgroups(tiles.0.div_ceil(8), tiles.1.div_ceil(8), 1);
        }
        for layer in &targets.layers {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("sight's march"), timestamp_writes: None });
            pass.set_pipeline(&self.march);
            pass.set_bind_group(0, &layer.group, &[]);
            pass.dispatch_workgroups(layer.size.0.div_ceil(8), layer.size.1.div_ceil(8), 1);
        }
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("sight's upscale"), color_attachments: &[Some(attachment)], ..Default::default() });
        pass.set_pipeline(&self.upscale);
        pass.set_bind_group(0, &targets.upscale_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9);
    [v[0] / length, v[1] / length, v[2] / length]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

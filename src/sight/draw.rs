//! Sight drawn (the regroup, phase 3.2, the first sight): the coarse layer marched through the store on the
//! card (`march.wgsl`) at a fraction of the screen, then brought up to it (`upscale.wgsl`). The runner calls
//! `frame` before the panel is drawn over it.

use super::card_store::CardStore;
use std::sync::Arc;

/// What the frame hands sight: the eye and where it looks, and the settings.
#[derive(Clone, Copy, Debug)]
pub struct Eye {
    pub position: [f32; 3],
    pub looking: [f32; 3],
    /// The field of view, up and down (degrees).
    pub field_of_view: f32,
    /// How many screen pixels a side a coarse pixel is; rays a coarse pixel; the upscale's mode.
    pub grid: u32,
    pub rays: u32,
    pub upscale: u32,
}

/// The noon sun: high in the south, a little east [the first sight's default: "a fixed noon sun"].
const SUN: [f32; 3] = [-0.35, 0.2, 0.91];
const AMBIENT: f32 = 0.35;
/// The start pass's tile: screen pixels a side.
const TILE: u32 = 8;
/// The farthest a ray goes (blocks): past any map's far corner from anywhere in it.
const REACH: f32 = 150_000.0;

pub struct SightDraw {
    march: wgpu::ComputePipeline,
    start: wgpu::ComputePipeline,
    upscale: wgpu::RenderPipeline,
    camera: wgpu::Buffer,
    up: wgpu::Buffer,
    sampler: wgpu::Sampler,
    /// The coarse layer's picture and distance, their size, and what was bound to them.
    targets: Option<Targets>,
}

struct Targets {
    size: (u32, u32),
    grid: u32,
    store: Arc<CardStore>,
    /// The start pass's tiles across and down.
    tiles: (u32, u32),
    march_group: wgpu::BindGroup,
    start_group: wgpu::BindGroup,
    upscale_group: wgpu::BindGroup,
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
        let uniform = |label: &str, size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("sight's blend"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });
        Self { march, start, upscale, camera: uniform("sight's camera", 8 * 16), up: uniform("sight's upscale numbers", 2 * 16), sampler, targets: None }
    }

    /// The coarse targets for a screen of `screen` at `grid`, and their bindings to `store`, made again when
    /// any of them changes (a new grid is a moment's work: the store is the world's, not the screen's).
    fn targets(&mut self, device: &wgpu::Device, screen: (u32, u32), grid: u32, store: &Arc<CardStore>) -> &Targets {
        let size = (screen.0.div_ceil(grid).max(1), screen.1.div_ceil(grid).max(1));
        let stale = self.targets.as_ref().is_none_or(|t| t.size != size || t.grid != grid || !Arc::ptr_eq(&t.store, store));
        if stale {
            let texture = |label: &str, format: wgpu::TextureFormat| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
            };
            let picture = texture("sight's coarse picture", wgpu::TextureFormat::Rgba8Unorm).create_view(&Default::default());
            // The start pass: a tile of 8 × 8 screen pixels (at least a coarse pixel).
            let tile = (TILE / grid).max(1);
            let tiles = (size.0.div_ceil(tile), size.1.div_ceil(tile));
            let starts = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's starts"), size: (tiles.0 * tiles.1).max(1) as u64 * 4, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false });
            let far = texture("sight's coarse distances", wgpu::TextureFormat::R32Float).create_view(&Default::default());
            fn buffer(b: &wgpu::Buffer) -> wgpu::BindingResource<'_> {
                b.as_entire_binding()
            }
            let march_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sight's march"),
                layout: &self.march.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buffer(&self.camera) },
                    wgpu::BindGroupEntry { binding: 1, resource: buffer(&store.header) },
                    wgpu::BindGroupEntry { binding: 2, resource: buffer(&store.baked) },
                    wgpu::BindGroupEntry { binding: 3, resource: buffer(&store.live) },
                    wgpu::BindGroupEntry { binding: 4, resource: buffer(&store.pool) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&picture) },
                    wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&far) },
                    wgpu::BindGroupEntry { binding: 7, resource: buffer(&starts) },
                ],
            });
            let start_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sight's start pass"),
                layout: &self.start.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buffer(&self.camera) },
                    wgpu::BindGroupEntry { binding: 1, resource: buffer(&store.header) },
                    wgpu::BindGroupEntry { binding: 2, resource: buffer(&store.baked) },
                    wgpu::BindGroupEntry { binding: 3, resource: buffer(&store.live) },
                    wgpu::BindGroupEntry { binding: 7, resource: buffer(&starts) },
                ],
            });
            let upscale_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sight's upscale"),
                layout: &self.upscale.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&picture) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&far) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: buffer(&self.up) },
                ],
            });
            self.targets = Some(Targets { size, grid, store: store.clone(), tiles, march_group, start_group, upscale_group });
        }
        self.targets.as_ref().expect("made just now")
    }

    /// The coarse layer marched and brought up onto `view` (the screen's picture, `screen` pixels).
    pub fn frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, screen: (u32, u32), eye: Eye, store: &Arc<CardStore>) {
        let grid = eye.grid.max(1);
        let (size, tiles) = {
            let targets = self.targets(device, screen, grid, store);
            (targets.size, targets.tiles)
        };
        // The camera: the eye, and the directions of the screen's edges (game axes: looking north, east is right).
        let forward = normalise(eye.looking);
        let right = normalise(cross([0.0, 0.0, 1.0], forward));
        let up = cross(forward, right);
        let tan_half = (eye.field_of_view.to_radians() / 2.0).tan();
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let pixel = 2.0 * tan_half / size.1 as f32;
        let no_start = std::env::var("INCUS_SIGHT_NO_START").is_ok();
        let numbers: [[f32; 4]; 8] = [
            [eye.position[0], eye.position[1], eye.position[2], 0.0],
            [forward[0], forward[1], forward[2], 0.0],
            [right[0] * tan_half * aspect, right[1] * tan_half * aspect, right[2] * tan_half * aspect, 0.0],
            [up[0] * tan_half, up[1] * tan_half, up[2] * tan_half, 0.0],
            [size.0 as f32, size.1 as f32, pixel, eye.rays.max(1) as f32],
            [SUN[0], SUN[1], SUN[2], AMBIENT],
            [REACH, 0.0, if std::env::var("INCUS_SIGHT_STEPS").is_ok() { 1.0 } else if std::env::var("INCUS_SIGHT_LEVELS").is_ok() { 2.0 } else { 0.0 }, store.highest.load(std::sync::atomic::Ordering::Relaxed) as f32],
            // The start pass (`INCUS_SIGHT_NO_START` leaves it out, for measuring): tiles, a tile's coarse pixels,
            // and the cone's spread (a tile's half diagonal, in screen pixels at full resolution, and a pixel more).
            [tiles.0 as f32, tiles.1 as f32, if no_start { 0.0 } else { (TILE / grid).max(1) as f32 }, pixel / grid as f32 * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
        ];
        queue.write_buffer(&self.camera, 0, &numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let up_numbers: [[f32; 4]; 2] = [[screen.0 as f32, screen.1 as f32, size.0 as f32, size.1 as f32], [eye.upscale as f32, grid as f32, 0.0, 0.0]];
        queue.write_buffer(&self.up, 0, &up_numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let targets = self.targets.as_ref().expect("made above");
        if !no_start {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("sight's start pass"), timestamp_writes: None });
            pass.set_pipeline(&self.start);
            pass.set_bind_group(0, &targets.start_group, &[]);
            pass.dispatch_workgroups(tiles.0.div_ceil(8), tiles.1.div_ceil(8), 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("sight's march"), timestamp_writes: None });
            pass.set_pipeline(&self.march);
            pass.set_bind_group(0, &targets.march_group, &[]);
            pass.dispatch_workgroups(size.0.div_ceil(8), size.1.div_ceil(8), 1);
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

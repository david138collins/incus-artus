//! Sheets shown on the card: each held there as a picture of its own size, sent again only when it has changed, and
//! drawn over the screen's picture enlarged a whole number of times, every pixel of it a square of the screen's
//! (`sheets.wgsl`). The screen's picture isn't sRGB (`card.rs`), so the 95 scheme's colours reach it as they are.

use super::Sheet;

/// Where a sheet goes: its top left on the screen (the screen's pixels) and how many times over.
pub struct Placed<'a> {
    pub sheet: &'a Sheet,
    pub at: (i32, i32),
    pub scale: u32,
}

/// What the card holds for one place in the list of sheets drawn.
struct Held {
    texture: wgpu::Texture,
    size: (u32, u32),
    place: wgpu::Buffer,
    group: wgpu::BindGroup,
    /// The sheet it holds, as it was when sent (its number and its version).
    sheet: (u64, u64),
}

pub struct OnCard {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    held: Vec<Held>,
}

impl OnCard {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the sheets"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the sheets"), source: wgpu::ShaderSource::Wgsl(include_str!("sheets.wgsl").into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the sheets"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the sheets"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("corners"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("pixel"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self { pipeline, layout, held: Vec::new() }
    }

    /// The sheets sent to the card where they've changed, then drawn over `view` (the screen's picture, `screen` its
    /// size) in their order, in one pass named `label` (as the card's debuggers show it): `load` says whether the picture
    /// is kept or cleared first.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, screen: (u32, u32), sheets: &[Placed], load: wgpu::LoadOp<wgpu::Color>, timestamps: Option<wgpu::RenderPassTimestampWrites>, label: &str) {
        for (k, placed) in sheets.iter().enumerate() {
            let size = placed.sheet.size();
            if self.held.get(k).is_none_or(|held| held.size != size) {
                let held = self.hold(device, size);
                if k < self.held.len() {
                    self.held[k] = held;
                } else {
                    self.held.push(held);
                }
            }
            let held = &mut self.held[k];
            let now = (placed.sheet.id(), placed.sheet.version());
            if held.sheet != now {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: &held.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    placed.sheet.bytes(),
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size.0 * 4), rows_per_image: Some(size.1) },
                    wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                );
                held.sheet = now;
            }
            let numbers = [placed.at.0 as f32, placed.at.1 as f32, size.0 as f32, size.1 as f32, screen.0 as f32, screen.1 as f32, placed.scale as f32, 0.0];
            let bytes: Vec<u8> = numbers.iter().flat_map(|number| number.to_le_bytes()).collect();
            queue.write_buffer(&held.place, 0, &bytes);
        }
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some(label), color_attachments: &[Some(attachment)], timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.pipeline);
        for held in &self.held[..sheets.len()] {
            pass.set_bind_group(0, &held.group, &[]);
            pass.draw(0..6, 0..1);
        }
    }

    /// A picture `size` on the card, and its place's numbers.
    fn hold(&self, device: &wgpu::Device, size: (u32, u32)) -> Held {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("a sheet"),
            size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let place = device.create_buffer(&wgpu::BufferDescriptor { label: Some("a sheet's place"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("a sheet"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry { binding: 1, resource: place.as_entire_binding() }],
        });
        Held { texture, size, place, group, sheet: (0, 0) }
    }
}

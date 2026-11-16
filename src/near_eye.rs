//! The near eye on the card (plan 2.3, build 195): the chunks' faces kept in pages of one buffer each, made by the
//! near lane (`near.rs`) and drawn as triangles after the far picture is brought up, with depth (`near.wgsl`). The
//! render radius (the Render page's slider, David's word of 2026-11-16: "the render radius set there will determine
//! where the triangles start and rays fade") says how far out chunks are kept; the faces reach as far as every chunk
//! is in, fade out over the last stretch before that, and the far rays begin a little inside it, so there's never a
//! gap between the two eyes.

use crate::far_eye::FarEye;
use crate::near::{CHUNK, Lane, Made, nearest_part, wanted};
use std::collections::HashMap;

/// The radius the game starts with, chunks (128 blocks), and the most the slider goes to (David's goal: 64).
pub const RADIUS: f32 = 4.0;
pub const MOST_RADIUS: f32 = 64.0;
/// A page's faces (12 bytes each: a page is 12 MB).
const PAGE: u32 = 1 << 20;
/// A face on the card: three words.
const FACE: u64 = 12;
/// The most chunks drawn in a frame (a 64-chunk radius is 13,150).
const MOST_DRAWN: u64 = 16_384;
/// The nearest and farthest depth kept (blocks ahead of the eye).
const DEPTHS: [f32; 2] = [0.2, 8192.0];
/// The most faces put on the card in a frame (4 MB), so a radius filling doesn't hold a frame.
const FACES_A_FRAME: usize = 350_000;

/// A chunk on the card: its page, where its faces begin there and how many (the last `water` of them the water's
/// tops), and the lowest and highest block they touch.
struct Kept {
    page: usize,
    first: u32,
    count: u32,
    water: u32,
    low: i32,
    high: i32,
}

/// A page: a buffer of faces, and its free stretches (where they begin, how many faces), in order.
struct Page {
    buffer: wgpu::Buffer,
    free: Vec<(u32, u32)>,
}

/// What the near eye says of itself, for the panel and a flight's report.
#[derive(Clone, Copy, Default, Debug)]
pub struct Counts {
    /// Chunks wanted inside the radius, kept on the card, and drawn this frame.
    pub wanted: usize,
    pub kept: usize,
    pub drawn: usize,
    /// Faces kept on the card, and drawn this frame (two triangles each).
    pub faces: u64,
    pub faces_drawn: u64,
    /// The pages' memory on the card, MB.
    pub megabytes: f32,
    /// How far out the faces reach now (blocks), and where the far rays begin.
    pub reach: f32,
    pub far_from: f32,
    /// A chunk's making on the lane, ms (the last hundred's mean), and the lane's threads.
    pub chunk_ms: f32,
    pub threads: usize,
}

pub struct NearEye {
    lane: Lane,
    /// The render radius, chunks (the Render page's slider).
    pub radius: f32,
    chunks: HashMap<[i32; 2], Kept>,
    pages: Vec<Page>,
    /// What was last asked of the lane: the eye's chunk and the radius; and the chunks wanted then, nearest first.
    asked: Option<([i32; 2], f32)>,
    want: Vec<[i32; 2]>,
    /// Made and waiting for room in a frame.
    waiting: std::collections::VecDeque<Made>,
    recent_ms: std::collections::VecDeque<f32>,
    faces: wgpu::RenderPipeline,
    water: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    depth_layout: wgpu::BindGroupLayout,
    /// The bind group, and what it was made from (`FarEye::near_reads`' number).
    group: Option<(wgpu::BindGroup, u64)>,
    numbers: wgpu::Buffer,
    colours: wgpu::Buffer,
    /// The depth, its view and the water's group for it, at the picture's size.
    depth: Option<(wgpu::TextureView, wgpu::BindGroup, (u32, u32))>,
    /// The frame's draws, the blocks' sides' and then the water's tops', each a page after another: for each page
    /// where its draws begin in `draws` and how many.
    draws: wgpu::Buffer,
    drawn: Vec<(usize, u64, u32)>,
    drawn_water: Vec<(usize, u64, u32)>,
    pub counts: Counts,
}

/// A piece of `far.wgsl`: from `from` to the first line that is only a closing brace after it (a struct or a
/// function), or to the end of its own line (a `const`).
fn far_piece<'a>(far: &'a str, from: &str) -> &'a str {
    let start = far.find(from).unwrap_or_else(|| panic!("far.wgsl has no {from:?} for the near eye"));
    let rest = &far[start..];
    let end = if from.starts_with("const") { rest.find('\n').map(|at| at + 1) } else { rest.find("\n}").map(|at| at + 3) };
    &rest[..end.unwrap_or(rest.len()).min(rest.len())]
}

/// The near eye's program: the far eye's `Camera`, its levels' table and its light's text, then `near.wgsl`.
pub fn source() -> String {
    let far = include_str!("far.wgsl").replace("\r\n", "\n");
    let mut text = String::new();
    for piece in ["struct Camera {", "const FIRST: i32", "const TOP: i32", "const TABLE: i32", "const SKY_TINT", "fn table(", "fn lit(", "fn sunlit(", "fn light_of(", "fn cell_light(", "fn to_srgb("] {
        text += far_piece(&far, piece);
        text += "\n";
    }
    text + include_str!("near.wgsl")
}

impl NearEye {
    /// The near eye, its lane started on the map behind the doorway in `folder`. `colours`: each block's top's and
    /// sides' colour as light (`FarColours::as_light`). `format`: the picture's.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, folder: std::path::PathBuf, colours: &[[f32; 4]]) -> Result<Self, String> {
        // The lane's threads: a quarter of the processor's, 2 at least and 6 at most, unless told (`INCUS_NEAR_THREADS`).
        // [Build 195 took 12 of 24: the whole of 64 chunks in 5 s, and the processor's area at 102 degrees in David's
        // first look. Six fill it in about twice the time and leave the frame and the far eye's lanes their room.]
        let threads = std::env::var("INCUS_NEAR_THREADS").ok().and_then(|n| n.parse::<usize>().ok()).map_or_else(|| (std::thread::available_parallelism().map_or(2, |n| n.get()) / 4).clamp(2, 6), |n| n.clamp(1, 32));
        let lane = Lane::start(folder, crate::scatter_stage::on(), threads)?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the near eye"), source: wgpu::ShaderSource::Wgsl(source().into()) });
        let both = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let uniform = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry { binding, visibility, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let picture = |binding: u32| wgpu::BindGroupLayoutEntry { binding, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the near eye"),
            entries: &[
                uniform(0, both),
                uniform(1, fragment),
                picture(2),
                picture(3),
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: fragment, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: fragment, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                uniform(6, both),
            ],
        });
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the near eye's depth, for its water"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None }],
        });
        // (Each pipeline its own layout: the water's reads the depth as a picture, in a second group; the faces'
        // writes that depth and so can't be given it.)
        let faces_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the near faces"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let water_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the near water"), bind_group_layouts: &[Some(&layout), Some(&depth_layout)], immediate_size: 0 });
        let face_words = [wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32x3, offset: 0, shader_location: 0 }];
        let buffers = [Some(wgpu::VertexBufferLayout { array_stride: FACE, step_mode: wgpu::VertexStepMode::Instance, attributes: &face_words })];
        let fading = Some(wgpu::BlendState::ALPHA_BLENDING);
        let strip = wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() };
        let faces = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the near faces"),
            layout: Some(&faces_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("near_corner"), compilation_options: Default::default(), buffers: &buffers },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("near_face"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: fading, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: strip,
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Less), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let water = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the near water"),
            layout: Some(&water_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("near_corner"), compilation_options: Default::default(), buffers: &buffers },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("near_water"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: fading, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: strip,
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near eye's numbers"), size: 48, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let colour_bytes: Vec<u8> = colours.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
        let colours = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the blocks' colours from far"), size: colour_bytes.len().max(32) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&colours, 0, &colour_bytes);
        let draws = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near eye's draws"), size: MOST_DRAWN * 2 * 16, usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let radius = std::env::var("INCUS_NEAR_RADIUS").ok().and_then(|r| r.parse::<f32>().ok()).map_or(RADIUS, |r| r.clamp(1.0, MOST_RADIUS));
        log::info!("The near eye: faces as triangles within a render radius of {radius} chunks ({} blocks), its lane on {threads} threads", radius * CHUNK as f32);
        Ok(Self { lane, radius, chunks: HashMap::new(), pages: Vec::new(), asked: None, want: Vec::new(), waiting: Default::default(), recent_ms: Default::default(), faces, water, layout, depth_layout, group: None, numbers, colours, depth: None, draws, drawn: Vec::new(), drawn_water: Vec::new(), counts: Counts { threads, ..Default::default() } })
    }

    /// Room for `count` faces: a page and where in it (a free stretch that holds them, the first found; a new page
    /// if none does).
    fn room(&mut self, device: &wgpu::Device, count: u32) -> (usize, u32) {
        for (k, page) in self.pages.iter_mut().enumerate() {
            if let Some(at) = page.free.iter().position(|(_, free)| *free >= count) {
                let (first, free) = page.free[at];
                if free == count {
                    page.free.remove(at);
                } else {
                    page.free[at] = (first + count, free - count);
                }
                return (k, first);
            }
        }
        let size = PAGE.max(count);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("a page of near faces"), size: size as u64 * FACE, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        self.pages.push(Page { buffer, free: if size > count { vec![(count, size - count)] } else { Vec::new() } });
        (self.pages.len() - 1, 0)
    }

    /// A chunk's faces given back to its page.
    fn give_back(&mut self, kept: &Kept) {
        if kept.count == 0 {
            return;
        }
        let free = &mut self.pages[kept.page].free;
        let at = free.partition_point(|(first, _)| *first < kept.first);
        free.insert(at, (kept.first, kept.count));
        // Joined with the stretch after it, and the one before, where they touch.
        if at + 1 < free.len() && free[at].0 + free[at].1 == free[at + 1].0 {
            free[at].1 += free[at + 1].1;
            free.remove(at + 1);
        }
        if at > 0 && free[at - 1].0 + free[at - 1].1 == free[at].0 {
            free[at - 1].1 += free[at].1;
            free.remove(at);
        }
    }

    /// Each frame, before the far eye's numbers are made: the lane asked for what the eye's place and the radius want
    /// (when either changed); the chunks it made put on the card, as many as a frame may; those outside the radius
    /// given back; how far the faces reach now; and the far eye told where its rays may begin.
    pub fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, eye: [f32; 2], far: &mut FarEye) -> Result<(), String> {
        let radius = self.radius.clamp(1.0, MOST_RADIUS);
        let here = [(eye[0] / CHUNK as f32).floor() as i32, (eye[1] / CHUNK as f32).floor() as i32];
        if self.asked != Some((here, radius)) {
            self.asked = Some((here, radius));
            self.want = wanted(eye, radius);
            // Out of reach by a chunk and more: given back.
            let reach = (radius + 2.0) * CHUNK as f32;
            let gone: Vec<[i32; 2]> = self.chunks.keys().filter(|chunk| nearest_part(**chunk, eye) > reach).copied().collect();
            for chunk in gone {
                if let Some(kept) = self.chunks.remove(&chunk) {
                    self.give_back(&kept);
                }
            }
            self.waiting.retain(|made| nearest_part(made.chunk, eye) <= reach);
            // (What's made and waiting for its frame isn't asked for again. Looked up in a set: by walking the waiting
            // ones for each wanted chunk, a crossing into a new chunk while a radius filled was thousands by thousands.)
            let waiting: std::collections::HashSet<[i32; 2]> = self.waiting.iter().map(|made| made.chunk).collect();
            let chunks = &self.chunks;
            self.lane.ask(self.want.iter().filter(|chunk| !chunks.contains_key(*chunk) && !waiting.contains(*chunk)).copied());
        }
        self.waiting.extend(self.lane.take(4096)?);
        // Onto the card, nearest the eye first, as many faces as a frame may (one chunk at least).
        let mut put = 0usize;
        while let Some(made) = self.waiting.front() {
            if put > 0 && put + made.faces.len() > FACES_A_FRAME {
                break;
            }
            let made = self.waiting.pop_front().unwrap();
            put += made.faces.len();
            self.recent_ms.push_back(made.ms);
            if self.recent_ms.len() > 100 {
                self.recent_ms.pop_front();
            }
            if let Some(old) = self.chunks.remove(&made.chunk) {
                self.give_back(&old);
            }
            let count = made.faces.len() as u32;
            let (page, first) = if count > 0 { self.room(device, count) } else { (0, 0) };
            if count > 0 {
                let bytes: Vec<u8> = made.faces.iter().flatten().flat_map(|word| word.to_le_bytes()).collect();
                queue.write_buffer(&self.pages[page].buffer, first as u64 * FACE, &bytes);
            }
            self.chunks.insert(made.chunk, Kept { page, first, count, water: made.water, low: made.low, high: made.high });
        }
        // How far the faces reach: to the nearest chunk that's wanted and not yet in, or the whole radius.
        let mut reach = radius * CHUNK as f32;
        let mut missing = 0;
        for chunk in &self.want {
            if !self.chunks.contains_key(chunk) {
                reach = reach.min(nearest_part(*chunk, eye));
                missing += 1;
                if missing >= 64 {
                    break;
                }
            }
        }
        // The far rays begin inside the stretch the faces fade over, and a far cell's width more (the 16s there).
        let fade = fade_of(reach);
        let far_from = (reach - fade - 24.0).max(0.0);
        far.near_from = far_from;
        self.counts = Counts {
            wanted: self.want.len(),
            kept: self.chunks.len(),
            faces: self.chunks.values().map(|kept| kept.count as u64).sum(),
            megabytes: self.pages.iter().map(|page| page.buffer.size()).sum::<u64>() as f32 / 1048576.0,
            reach,
            far_from,
            chunk_ms: if self.recent_ms.is_empty() { 0.0 } else { self.recent_ms.iter().sum::<f32>() / self.recent_ms.len() as f32 },
            threads: self.counts.threads,
            drawn: self.counts.drawn,
            faces_drawn: self.counts.faces_drawn,
        };
        Ok(())
    }

    /// The frame's numbers and draws made ready (before its passes are recorded): the depth at the picture's size, the
    /// bind group from what the far eye has now, and the chunks in view listed for each page. `eye`, `forward`, and
    /// the view's half-widths (the tangent of half the angle across and up) say what's in view.
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, screen: (u32, u32), far: &FarEye, camera: &crate::camera::Camera) -> bool {
        let Some((far_camera, table, shadow, sky, sampler, made_from)) = far.near_reads() else { return false };
        if self.group.as_ref().is_none_or(|(_, from)| *from != made_from) {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the near eye"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: far_camera.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: table.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(shadow) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(sky) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(sampler) },
                    wgpu::BindGroupEntry { binding: 5, resource: self.colours.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 6, resource: self.numbers.as_entire_binding() },
                ],
            });
            self.group = Some((group, made_from));
        }
        if self.depth.as_ref().is_none_or(|(_, _, size)| *size != screen) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("the near eye's depth"),
                size: wgpu::Extent3d { width: screen.0.max(1), height: screen.1.max(1), depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("the near eye's depth, for its water"), layout: &self.depth_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }] });
            self.depth = Some((view, group, screen));
        }
        // The numbers: the eye's block and how far into it the eye is; the reach, the fade and the depths.
        let eye = [camera.position.x, camera.position.y, camera.position.z];
        let cell = eye.map(|v| v.floor());
        let mut bytes: Vec<u8> = Vec::with_capacity(48);
        bytes.extend(cell.iter().map(|v| *v as i32).chain([0]).flat_map(|v| v.to_le_bytes()));
        bytes.extend((0..3).map(|k| eye[k] - cell[k]).chain([0.0]).flat_map(|v| v.to_le_bytes()));
        bytes.extend([self.counts.reach, fade_of(self.counts.reach), DEPTHS[0], DEPTHS[1]].iter().flat_map(|v| v.to_le_bytes()));
        queue.write_buffer(&self.numbers, 0, &bytes);
        // The chunks in view, a page after another: each a draw of its faces.
        let view = View::of(camera, screen);
        let reach = self.counts.reach;
        let mut by_page: Vec<Vec<[u32; 4]>> = vec![Vec::new(); self.pages.len()];
        let mut water_by_page: Vec<Vec<[u32; 4]>> = vec![Vec::new(); self.pages.len()];
        let (mut drawn, mut faces_drawn) = (0usize, 0u64);
        for (chunk, kept) in &self.chunks {
            if kept.count == 0 || nearest_part(*chunk, [eye[0], eye[1]]) > reach || !view.sees(*chunk, kept.low, kept.high) {
                continue;
            }
            if drawn as u64 >= MOST_DRAWN {
                break;
            }
            // A face is four corners (a strip), once for each of the chunk's faces of the kind.
            let sides = kept.count - kept.water;
            if sides > 0 {
                by_page[kept.page].push([4, sides, 0, kept.first]);
            }
            if kept.water > 0 {
                water_by_page[kept.page].push([4, kept.water, 0, kept.first + sides]);
            }
            drawn += 1;
            faces_drawn += kept.count as u64;
        }
        self.drawn.clear();
        self.drawn_water.clear();
        let mut words: Vec<u8> = Vec::with_capacity(drawn * 32);
        for (lists, said) in [(&by_page, &mut self.drawn), (&water_by_page, &mut self.drawn_water)] {
            for (page, draws) in lists.iter().enumerate() {
                if !draws.is_empty() {
                    said.push((page, words.len() as u64, draws.len() as u32));
                    words.extend(draws.iter().flatten().flat_map(|word| word.to_le_bytes()));
                }
            }
        }
        if !words.is_empty() {
            queue.write_buffer(&self.draws, 0, &words);
        }
        self.counts.drawn = drawn;
        self.counts.faces_drawn = faces_drawn;
        true
    }

    /// The faces drawn over the picture (`view`), with depth.
    pub fn draw_faces(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let (Some((group, _)), Some((depth, _, _))) = (&self.group, &self.depth) else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let depth = wgpu::RenderPassDepthStencilAttachment { view: depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the near faces"), color_attachments: &[Some(attachment)], depth_stencil_attachment: Some(depth), timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.faces);
        pass.set_bind_group(0, group, &[]);
        for (page, at, count) in &self.drawn {
            pass.set_vertex_buffer(0, self.pages[*page].buffer.slice(..));
            pass.multi_draw_indirect(&self.draws, *at, *count);
        }
    }

    /// The water's top drawn over the faces, by how much water the eye looks through.
    pub fn draw_water(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let (Some((group, _)), Some((_, depth_group, _))) = (&self.group, &self.depth) else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the near water"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.water);
        pass.set_bind_group(0, group, &[]);
        pass.set_bind_group(1, depth_group, &[]);
        for (page, at, count) in &self.drawn_water {
            pass.set_vertex_buffer(0, self.pages[*page].buffer.slice(..));
            pass.multi_draw_indirect(&self.draws, *at, *count);
        }
    }
}

/// Over how many blocks before their reach the near faces fade out: an eighth of it, 8 at least and 48 at most.
pub fn fade_of(reach: f32) -> f32 {
    (reach / 8.0).clamp(8.0, 48.0)
}

/// What's in view: the eye, its way, and its four sides, for leaving out a chunk none of which is.
struct View {
    eye: [f32; 3],
    /// Each side's way out (a place is out of view if it's on the outer side of any).
    sides: [[f32; 3]; 5],
}

impl View {
    fn of(camera: &crate::camera::Camera, screen: (u32, u32)) -> Self {
        let (facing, pitch) = (camera.facing.to_radians(), camera.pitch.to_radians());
        let forward = [pitch.cos() * facing.cos(), pitch.cos() * facing.sin(), pitch.sin()];
        let right = [-facing.sin(), facing.cos(), 0.0];
        let up = [-pitch.sin() * facing.cos(), -pitch.sin() * facing.sin(), pitch.cos()];
        let tan_up = (crate::far_eye::FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let tan_across = tan_up * screen.0 as f32 / screen.1.max(1) as f32;
        let side = |way: [f32; 3], sign: f32, tan: f32| -> [f32; 3] { std::array::from_fn(|k| sign * way[k] - tan * forward[k]) };
        Self { eye: [camera.position.x, camera.position.y, camera.position.z], sides: [side(right, 1.0, tan_across), side(right, -1.0, tan_across), side(up, 1.0, tan_up), side(up, -1.0, tan_up), forward.map(|v| -v)] }
    }

    /// Whether any of a chunk's box, from `low` to `high`, may be in view.
    fn sees(&self, chunk: [i32; 2], low: i32, high: i32) -> bool {
        let side = CHUNK as f32;
        let middle = [(chunk[0] as f32 + 0.5) * side - self.eye[0], (chunk[1] as f32 + 0.5) * side - self.eye[1], (low + high) as f32 * 0.5 - self.eye[2]];
        let half = [side * 0.5, side * 0.5, (high - low) as f32 * 0.5];
        self.sides.iter().all(|out| (0..3).map(|k| middle[k] * out[k] - half[k] * out[k].abs()).sum::<f32>() <= 0.0)
    }
}

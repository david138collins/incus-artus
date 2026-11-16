//! The near eye on the card (plan 2.3, build 195): the chunks' faces kept in pages of one buffer each, made by the
//! near lane (`near.rs`) and drawn as triangles after the far picture is brought up, with depth (`near.wgsl`). The
//! render radius (the Render page's slider, David's word of 2026-11-16: "the render radius set there will determine
//! where the triangles start and rays fade") says how far out chunks are kept; the faces reach as far as every chunk
//! is in, fade out over the last stretch before that, and the far rays begin a little inside it, so there's never a
//! gap between the two eyes.

use crate::doorway::Change;
use crate::far_eye::FarEye;
use crate::near::{CHUNK, Lane, Made, nearest_part, wanted};
use std::collections::{HashMap, HashSet};

/// The radius the game starts with, chunks (1,024 blocks; David's word of 2026-11-16, with the x-fader's starting
/// numbers: `NearLook::DEFAULT`), and the most the slider goes to (his goal: 64).
pub const RADIUS: f32 = 32.0;
pub const MOST_RADIUS: f32 = 64.0;
/// A page's faces (12 bytes each: a page is 12 MB).
const PAGE: u32 = 1 << 20;
/// A face on the card: three words.
const FACE: u64 = 12;
/// The most chunks drawn in a frame (a 64-chunk radius is 13,150).
const MOST_DRAWN: u64 = 16_384;
/// The nearest kept (blocks ahead of the eye), and how far ahead "nothing behind" is taken to be for the water. The
/// depth is kept turned round (the nearest 1, falling towards 0 far out: `near.wgsl`), so nothing is cut off far out.
const DEPTHS: [f32; 2] = [0.2, 8192.0];
/// The most faces put on the card in a frame (4 MB), so a radius filling doesn't hold a frame.
const FACES_A_FRAME: usize = 350_000;
/// The most the x-fader's sliders go to: its band's width (blocks) and its curve.
pub const MOST_FADER_WIDTH: f32 = 512.0;
pub const MOST_FADER_CURVE: f32 = 8.0;

/// The near eye's settings (the Render page's "The near eye"): the render radius, chunks; and the x-fader, how blocks
/// hand over to rays [David, 2026-11-16: "i think a x-fader with curves could hide it almost completely so blocks
/// smoothly fade into rays"]: its band's width before the faces' reach (blocks), and its curve (1 a straight fade;
/// higher, an S that holds each side longer and crosses quicker in the middle).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NearLook {
    pub radius: f32,
    pub fader_width: f32,
    pub fader_curve: f32,
}

impl NearLook {
    /// [David, 2026-11-16, asked by Counsel "Make your crossfade settings, 128 wide and curve 1, the defaults?":
    /// "yes, and 32 chunk radius."]
    pub const DEFAULT: Self = Self { radius: RADIUS, fader_width: 128.0, fader_curve: 1.0 };
}

/// The near eye's own pictures, the screen's size: its colour (each pixel multiplied by how much of it shows) and its
/// depth, the water's group for the depth, and the x-fader's group for the colour.
struct Pictures {
    colour: wgpu::TextureView,
    depth: wgpu::TextureView,
    depth_group: wgpu::BindGroup,
    join_group: wgpu::BindGroup,
    size: (u32, u32),
}

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
    /// The render radius these are for (chunks), and whether every wanted chunk is in, as its changes have it now.
    pub radius: f32,
    pub whole: bool,
}

pub struct NearEye {
    lane: Lane,
    /// The render radius and the x-fader (the Render page's sliders).
    pub look: NearLook,
    chunks: HashMap<[i32; 2], Kept>,
    pages: Vec<Page>,
    /// What was last asked of the lane: the eye's chunk and the radius; and the chunks wanted then, nearest first.
    asked: Option<([i32; 2], f32)>,
    want: Vec<[i32; 2]>,
    /// How far into `want` every chunk is in (all before this are): the faces' reach is looked for from here on.
    /// [Build 196 walked every wanted chunk each frame for it: 13,100 look-ups at 64 chunks.]
    first_to_come: usize,
    /// The changes as the lane was last told them: the chunks they touch; for each chunk a telling has touched, the
    /// telling its faces must be made from at the least; and the chunks kept whose faces are older than that (made
    /// again, and drawn as they were until then).
    touched: HashSet<[i32; 2]>,
    need: HashMap<[i32; 2], u64>,
    remake: HashSet<[i32; 2]>,
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
    /// The blocks' pictures on the card (a layer a picture), how they're read, and whether the faces wear them.
    blocks_view: wgpu::TextureView,
    crisp: wgpu::Sampler,
    textured: bool,
    /// The x-fader's pass, and what its group is made by; the picture's format; the near eye's own pictures.
    join: wgpu::RenderPipeline,
    join_layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    pictures: Option<Pictures>,
    /// The cover pass (build 201): its program, what its group is made by, its numbers (the far layers' sizes), and
    /// its group with what it was made from (the far eye's targets' number, and the near picture's size).
    cover_pass: wgpu::ComputePipeline,
    cover_layout: wgpu::BindGroupLayout,
    cover_sizes: wgpu::Buffer,
    cover_sampler: wgpu::Sampler,
    cover_group: Option<(wgpu::BindGroup, u64, (u32, u32))>,
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
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, folder: std::path::PathBuf, colours: &[[f32; 4]], textures: &crate::textures::Textures) -> Result<Self, String> {
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
                wgpu::BindGroupLayoutEntry { binding: 7, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 8, visibility: fragment, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
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
        // (The near picture is kept multiplied by how much of it shows: a face is written whole, the nearest winning
        // a pixel; the water, and then the whole picture over the far one, laid over what's there.)
        let over = Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        let strip = wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() };
        let faces = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the near faces"),
            layout: Some(&faces_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("near_corner"), compilation_options: Default::default(), buffers: &buffers },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("near_face"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: strip,
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Greater), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let water = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the near water"),
            layout: Some(&water_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("near_corner"), compilation_options: Default::default(), buffers: &buffers },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("near_water"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: over, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: strip,
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        // The x-fader: the near picture laid over the far picture.
        let join_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the x-fader"), source: wgpu::ShaderSource::Wgsl(include_str!("near_join.wgsl").into()) });
        let join_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the x-fader"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None }],
        });
        let join_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the x-fader"), bind_group_layouts: &[Some(&join_layout)], immediate_size: 0 });
        let join = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the x-fader"),
            layout: Some(&join_pipeline_layout),
            vertex: wgpu::VertexState { module: &join_module, entry_point: Some("near_whole"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &join_module, entry_point: Some("near_over"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: over, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        // The cover: which of the far eye's pixels the near picture wholly covers.
        let cover_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the near cover"), source: wgpu::ShaderSource::Wgsl(include_str!("near_cover.wgsl").into()) });
        let compute = wgpu::ShaderStages::COMPUTE;
        let cover_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the near cover"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: compute, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: compute, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: compute, ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::WriteOnly, format: wgpu::TextureFormat::Rgba8Unorm, view_dimension: wgpu::TextureViewDimension::D2 }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: compute, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let cover_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the near cover"), bind_group_layouts: &[Some(&cover_layout)], immediate_size: 0 });
        let cover_pass = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("near_cover"), layout: Some(&cover_pipeline_layout), module: &cover_module, entry_point: Some("near_cover"), compilation_options: Default::default(), cache: None });
        let cover_sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("the near cover's four at a time"), ..Default::default() });
        let cover_sizes = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near cover's numbers"), size: 64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near eye's numbers"), size: 48, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let colour_bytes: Vec<u8> = colours.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
        let colours = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the blocks' colours from far"), size: colour_bytes.len().max(32) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&colours, 0, &colour_bytes);
        let draws = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near eye's draws"), size: MOST_DRAWN * 2 * 16, usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        // The blocks' pictures (build 205): one texture, a layer a picture (a block's top, then its sides), each size
        // of it as `textures.rs` shrank it; read with its own pixels crisp up close and its shrunk sizes blended as
        // it gets small, a block at a time across a merged face. Kept as stored, so the card reads them as light.
        // (`INCUS_NEAR_TEXTURES=0`: the faces flat in their far colours, as to build 204, to measure against. Then no
        // picture is put on the card either, only a one-pixel stand-in: so the card's library has no more to keep
        // account of than it had before there were pictures.)
        let textured = std::env::var("INCUS_NEAR_TEXTURES").as_deref() != Ok("0");
        log::info!("The near eye's faces: {}", if textured { "wearing their blocks' pictures" } else { "flat in their far colours (INCUS_NEAR_TEXTURES=0)" });
        let (picture_side, layers, sizes): (u32, u32, &[Vec<u8>]) = if textured { (crate::textures::SIDE, textures.pictures.max(1), &textures.sizes) } else { (1, 1, &[]) };
        let blocks_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("the blocks' pictures"),
            size: wgpu::Extent3d { width: picture_side, height: picture_side, depth_or_array_layers: layers },
            mip_level_count: sizes.len().max(1) as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, pixels) in sizes.iter().enumerate() {
            let side = picture_side >> level;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &blocks_texture, mip_level: level as u32, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                pixels,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(side * 4), rows_per_image: Some(side) },
                wgpu::Extent3d { width: side, height: side, depth_or_array_layers: layers },
            );
        }
        let blocks_view = blocks_texture.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() });
        let crisp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("the blocks' pictures: crisp near, shrunk far"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let radius = std::env::var("INCUS_NEAR_RADIUS").ok().and_then(|r| r.parse::<f32>().ok()).map_or(RADIUS, |r| r.clamp(1.0, MOST_RADIUS));
        log::info!("The near eye: faces as triangles within a render radius of {radius} chunks ({} blocks), its lane on {threads} threads", radius * CHUNK as f32);
        Ok(Self { lane, look: NearLook { radius, ..NearLook::DEFAULT }, chunks: HashMap::new(), pages: Vec::new(), asked: None, want: Vec::new(), first_to_come: 0, touched: HashSet::new(), need: HashMap::new(), remake: HashSet::new(), waiting: Default::default(), recent_ms: Default::default(), faces, water, layout, depth_layout, group: None, numbers, colours, blocks_view, crisp, textured, join, join_layout, format, pictures: None, cover_pass, cover_layout, cover_sizes, cover_sampler, cover_group: None, draws, drawn: Vec::new(), drawn_water: Vec::new(), counts: Counts { threads, ..Default::default() } })
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
        let radius = self.look.radius.clamp(1.0, MOST_RADIUS);
        let here = [(eye[0] / CHUNK as f32).floor() as i32, (eye[1] / CHUNK as f32).floor() as i32];
        if self.asked != Some((here, radius)) {
            self.asked = Some((here, radius));
            self.want = wanted(eye, radius);
            self.first_to_come = 0;
            // Out of reach by a chunk and more: given back. So is a chunk to be made again that isn't wanted now (it's
            // past the radius, and is made new when it's wanted).
            let reach = (radius + 2.0) * CHUNK as f32;
            let mut gone: Vec<[i32; 2]> = self.chunks.keys().filter(|chunk| nearest_part(**chunk, eye) > reach).copied().collect();
            if !self.remake.is_empty() {
                let want: HashSet<[i32; 2]> = self.want.iter().copied().collect();
                gone.extend(self.remake.iter().filter(|chunk| !want.contains(*chunk)));
            }
            for chunk in gone {
                if let Some(kept) = self.chunks.remove(&chunk) {
                    self.give_back(&kept);
                }
                self.remake.remove(&chunk);
            }
            self.waiting.retain(|made| nearest_part(made.chunk, eye) <= reach);
            // (What's made and waiting for its frame isn't asked for again. Looked up in a set: by walking the waiting
            // ones for each wanted chunk, a crossing into a new chunk while a radius filled was thousands by thousands.)
            let waiting: HashSet<[i32; 2]> = self.waiting.iter().map(|made| made.chunk).collect();
            let (chunks, remake) = (&self.chunks, &self.remake);
            self.lane.ask(self.want.iter().filter(|chunk| (!chunks.contains_key(*chunk) || remake.contains(*chunk)) && !waiting.contains(*chunk)).copied());
        }
        // (A chunk made from changes older than its own is let go: it's been asked for again.)
        let need = &self.need;
        self.waiting.extend(self.lane.take(4096)?.into_iter().filter(|made| made.told >= need.get(&made.chunk).copied().unwrap_or(0)));
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
            self.remake.remove(&made.chunk);
        }
        while self.first_to_come < self.want.len() && self.chunks.contains_key(&self.want[self.first_to_come]) {
            self.first_to_come += 1;
        }
        // How far the faces reach: to the nearest chunk that's wanted and not yet in, or the whole radius. (A chunk
        // being made again counts as in: its old faces are drawn.)
        let mut reach = radius * CHUNK as f32;
        let mut missing = 0;
        for chunk in &self.want[self.first_to_come..] {
            if !self.chunks.contains_key(chunk) {
                reach = reach.min(nearest_part(*chunk, eye));
                missing += 1;
                if missing >= 64 {
                    break;
                }
            }
        }
        // The far rays begin before the x-fader's band, and a far cell's width more (the 16s there).
        let far_from = (reach - band_of(self.look.fader_width, reach) - 24.0).max(0.0);
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
            radius,
            whole: self.first_to_come >= self.want.len() && self.remake.is_empty(),
        };
        Ok(())
    }

    /// Whether every chunk wanted round `eye` at the radius is in, as its changes have it now (a flight's standing
    /// waits for this).
    pub fn whole(&self, eye: [f32; 2]) -> bool {
        let here = [(eye[0] / CHUNK as f32).floor() as i32, (eye[1] / CHUNK as f32).floor() as i32];
        self.asked == Some((here, self.look.radius.clamp(1.0, MOST_RADIUS))) && self.counts.whole
    }

    /// The changes made through the game's doorway, as they are now (a mark's are made, and the mark before's taken
    /// back, as the camera goes to it): the lane told, and every chunk these or the ones before them touch made
    /// again. A chunk's old faces are drawn until its new ones come. Whole blocks and pieces (build 204).
    pub fn changed(&mut self, changes: &HashMap<[i32; 2], Vec<Change>>) {
        let now: HashSet<[i32; 2]> = changes
            .values()
            .flatten()
            .flat_map(crate::near::touched_by)
            .collect();
        if now.is_empty() && self.touched.is_empty() {
            return;
        }
        let told = self.lane.changed(changes.clone());
        for chunk in now.union(&self.touched) {
            self.need.insert(*chunk, told);
            if self.chunks.contains_key(chunk) {
                self.remake.insert(*chunk);
            }
        }
        let need = &self.need;
        self.waiting.retain(|made| made.told >= need.get(&made.chunk).copied().unwrap_or(0));
        self.touched = now;
        self.asked = None;
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
                    wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&self.blocks_view) },
                    wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::Sampler(&self.crisp) },
                ],
            });
            self.group = Some((group, made_from));
        }
        if self.pictures.as_ref().is_none_or(|pictures| pictures.size != screen) {
            let made = |label: &'static str, format: wgpu::TextureFormat| {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: screen.0.max(1), height: screen.1.max(1), depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                texture.create_view(&wgpu::TextureViewDescriptor::default())
            };
            let colour = made("the near picture", self.format);
            let depth = made("the near eye's depth", wgpu::TextureFormat::Depth32Float);
            let depth_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("the near eye's depth, for its water"), layout: &self.depth_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&depth) }] });
            let join_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("the x-fader"), layout: &self.join_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&colour) }] });
            self.pictures = Some(Pictures { colour, depth, depth_group, join_group, size: screen });
        }
        // The numbers: the eye's block and how far into it the eye is (and the x-fader's curve); the reach, the
        // x-fader's band and the depths.
        let eye = [camera.position.x, camera.position.y, camera.position.z];
        let cell = eye.map(|v| v.floor());
        let mut bytes: Vec<u8> = Vec::with_capacity(48);
        bytes.extend(cell.iter().map(|v| *v as i32).chain([self.textured as i32]).flat_map(|v| v.to_le_bytes()));
        bytes.extend((0..3).map(|k| eye[k] - cell[k]).chain([self.look.fader_curve.clamp(1.0, MOST_FADER_CURVE)]).flat_map(|v| v.to_le_bytes()));
        bytes.extend([self.counts.reach, band_of(self.look.fader_width, self.counts.reach), DEPTHS[0], DEPTHS[1]].iter().flat_map(|v| v.to_le_bytes()));
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

    /// The faces drawn into the near eye's own picture, with depth: every face whole, the nearest winning a pixel.
    pub fn draw_faces(&self, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let (Some((group, _)), Some(pictures)) = (&self.group, &self.pictures) else { return };
        let attachment = wgpu::RenderPassColorAttachment { view: &pictures.colour, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store } };
        let depth = wgpu::RenderPassDepthStencilAttachment { view: &pictures.depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Store }), stencil_ops: None };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the near faces"), color_attachments: &[Some(attachment)], depth_stencil_attachment: Some(depth), timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.faces);
        pass.set_bind_group(0, group, &[]);
        for (page, at, count) in &self.drawn {
            pass.set_vertex_buffer(0, self.pages[*page].buffer.slice(..));
            pass.multi_draw_indirect(&self.draws, *at, *count);
        }
    }

    /// The water's top laid over the faces in the near eye's picture, by how much water the eye looks through.
    pub fn draw_water(&self, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let (Some((group, _)), Some(pictures)) = (&self.group, &self.pictures) else { return };
        let attachment = wgpu::RenderPassColorAttachment { view: &pictures.colour, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the near water"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.water);
        pass.set_bind_group(0, group, &[]);
        pass.set_bind_group(1, &pictures.depth_group, &[]);
        for (page, at, count) in &self.drawn_water {
            pass.set_vertex_buffer(0, self.pages[*page].buffer.slice(..));
            pass.multi_draw_indirect(&self.draws, *at, *count);
        }
    }

    /// The cover pass (build 201), recorded after the faces and the water: which of the far eye's pixels the near
    /// picture wholly covers, written into the far eye's own picture for it, so the march needn't march them. False,
    /// and nothing recorded, where the far eye takes no cover or there's no near picture.
    pub fn cover(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, far: &FarEye, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) -> bool {
        let (Some(pictures), Some((words, sizes, side, made))) = (&self.pictures, far.cover_writes()) else { return false };
        if self.cover_group.as_ref().is_none_or(|(_, from, size)| *from != made || *size != pictures.size) {
            let numbers: Vec<u8> = [[sizes[0].0, sizes[0].1, side, 0], [sizes[1].0, sizes[1].1, 0, 0], [sizes[2].0, sizes[2].1, 0, 0], [pictures.size.0, pictures.size.1, 0, 0]].iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
            queue.write_buffer(&self.cover_sizes, 0, &numbers);
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the near cover"),
                layout: &self.cover_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&pictures.colour) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(words) },
                    wgpu::BindGroupEntry { binding: 2, resource: self.cover_sizes.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.cover_sampler) },
                ],
            });
            self.cover_group = Some((group, made, pictures.size));
        }
        let Some((group, _, _)) = &self.cover_group else { return false };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the near cover"), timestamp_writes: timestamps });
        pass.set_pipeline(&self.cover_pass);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(sizes[0].0.div_ceil(8), sizes[0].1.div_ceil(8), 1);
        true
    }

    /// The x-fader: the near eye's picture laid over the far picture (`view`).
    pub fn join(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let Some(pictures) = &self.pictures else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the x-fader"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.join);
        pass.set_bind_group(0, &pictures.join_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// The x-fader's band: over how many blocks before their reach the near faces hand over to the far picture. As wide
/// as its slider says, and never more than half the reach (while a radius fills, or at a small one).
pub fn band_of(width: f32, reach: f32) -> f32 {
    width.clamp(0.0, MOST_FADER_WIDTH).min(reach * 0.5)
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

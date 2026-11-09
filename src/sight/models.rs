//! Objects drawn in sight (build 96; David through Counsel, 2026-11-09: "yeah, lets put the object in at the same
//! time"): each placed object's model and its sprite, drawn after the march into the picture it made, in front of
//! and behind the world as the depth says (`upscale.wgsl` writes it from the march's touch). As the old renderer
//! drew them (`render/objects.rs` at the tag `before-regroup`):
//!
//! - **The model** is the object's glTF (Blockbench's export, `assets/models/`): its meshes, each with its texture
//!   (pixel art: nearest, as painted; a texel see-through or solid), placed as its nodes place them, in the game's
//!   axes as its shape was read (`data::read_shape`), scaled to its height and stood on its feet, turned to face its
//!   way. A model or a strip changed on disk is read again.
//! - **Lit as the land round it is:** the sky's light and the sun's on its faces (none of either under cover), and
//!   the placed light where it stands (`lights.rs`).
//! - **Its sprite** (a brazier's flame): a strip of frames (`data::read_strip`), on its spot, played at its
//!   milliseconds a frame, each object on its own frame (from its place), turning about the upright to face you,
//!   full bright.
//! - **Far lights' glows** (build 100): past the loaded chunks, each light a square of its colour a pixel or so
//!   wide, added to the picture: whole pixels, smaller (and dimmer as it's under a pixel) with distance, twinkling
//!   and through the fog as `lights::far_glow` and `draw::fog_left` say, seen or not as a ray marched to it says
//!   (`march.wgsl`'s `glow_pass`, so not by the depth).

use crate::data::{Anchor, Strip, models_folder, png_rgba, read_strip, sprites_folder};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// One placed object's model to draw this frame.
#[derive(Clone, Debug)]
pub struct ModelDrawn {
    /// Its model's file (in `assets/models/`).
    pub model: String,
    /// Its feet (blocks), the model's own feet (its shape's, in the game's axes, model units), and the model's units
    /// to blocks.
    pub feet: [f32; 3],
    pub model_feet: [f32; 3],
    pub scale: f32,
    /// Its turn: quarter turns, anticlockwise seen from above.
    pub facing: u8,
    /// How much of the sky's light reaches it (0 under cover, 1 in the open), and the placed light where it stands.
    pub sky: f32,
    pub glow: [f32; 3],
}

/// One sprite to draw this frame.
#[derive(Clone, Debug)]
pub struct SpriteDrawn {
    /// Its strip's file (in `assets/textures/sprites/`).
    pub strip: String,
    /// Where it hangs (blocks), its size (a strip pixel to a block pixel times this), how it hangs from its spot, its
    /// milliseconds a frame, and its own start (from its object's place).
    pub spot: [f32; 3],
    pub scale: f32,
    pub anchor: Anchor,
    pub frame_ms: u32,
    pub phase: u32,
}

/// A far light's glow to draw this frame (build 100).
#[derive(Clone, Copy, Debug)]
pub struct GlowDrawn {
    /// Its flame's place (blocks).
    pub at: [f32; 3],
    /// Its light now (linear, 0 to 1 each): its colour as it twinkles, through the fog.
    pub colour: [f32; 3],
    /// How big its flame is (blocks).
    pub size: f32,
}

/// What the objects need drawn this frame.
#[derive(Clone, Debug, Default)]
pub struct ObjectsFrame {
    pub models: Vec<ModelDrawn>,
    pub sprites: Vec<SpriteDrawn>,
    /// Far lights' glows, nearest first (`lights::GLOWS_MOST` at most are drawn).
    pub glows: Vec<GlowDrawn>,
    /// Milliseconds since the game started (the sprites' frames).
    pub millis: u64,
}

/// The view the objects are drawn from, as the march's.
#[derive(Clone, Copy, Debug)]
pub struct ObjectsView {
    pub eye: [f32; 3],
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    /// The screen's half width and half height at a block's distance.
    pub half: [f32; 2],
    pub sun: [f32; 3],
    pub ambient: f32,
    /// The sun's own light and the sky's, by the time of day.
    pub daylight: [f32; 2],
    /// The screen's size (pixels).
    pub screen: [f32; 2],
}

/// The nearest anything is drawn (blocks): the depth's scale (`upscale.wgsl` the same).
pub const NEAREST: f32 = 0.05;

/// A mesh of a model: its corners (three a triangle) and which of the model's textures it wears.
struct Part {
    corners: wgpu::Buffer,
    count: u32,
    texture: usize,
}

struct Model {
    parts: Vec<Part>,
    textures: Vec<wgpu::BindGroup>,
}

struct StripOnCard {
    strip: Strip,
    group: wgpu::BindGroup,
}

/// A file read once, kept with when it was changed (read again when that changes); none if it couldn't be read.
struct Kept<T> {
    modified: Option<SystemTime>,
    value: Option<T>,
}

pub struct ObjectsDraw {
    models_pipeline: wgpu::RenderPipeline,
    sprites_pipeline: wgpu::RenderPipeline,
    glows_pipeline: wgpu::RenderPipeline,
    view: wgpu::Buffer,
    view_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    models: HashMap<String, Kept<Model>>,
    strips: HashMap<String, Kept<StripOnCard>>,
    frame: ObjectsFrame,
    /// What couldn't be drawn, for the log (`take_problems`).
    problems: Vec<String>,
}

/// The depth picture's format.
pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Floats as the card reads them.
fn bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// A data URI's bytes (glTF's embedded buffers and pictures: base64 after the comma).
fn data_uri(uri: &str) -> Option<Vec<u8>> {
    let (head, text) = uri.split_once(',')?;
    if !head.starts_with("data:") || !head.ends_with(";base64") {
        return None;
    }
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    };
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let (mut bits, mut held) = (0u32, 0u32);
    for c in text.bytes().filter_map(value) {
        bits = bits << 6 | c as u32;
        held += 6;
        if held >= 8 {
            held -= 8;
            out.push((bits >> held) as u8);
        }
    }
    Some(out)
}

/// A model's meshes from its glTF's bytes: each primitive's triangles (corners in blocks' worth of model units,
/// the game's axes: glTF's Y up made Z up as `read_shape` makes it), and which picture it wears; and its pictures
/// (RGBA, their sizes).
pub fn read_model(bytes: &[u8]) -> Result<(Vec<(Vec<f32>, usize)>, Vec<(u32, u32, Vec<u8>)>), String> {
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|error| error.to_string())?;
    let buffers: Vec<Vec<u8>> = gltf
        .buffers()
        .map(|buffer| match buffer.source() {
            gltf::buffer::Source::Uri(uri) => data_uri(uri).ok_or_else(|| format!("its buffer {uri:.40} isn't embedded")),
            gltf::buffer::Source::Bin => gltf.blob.clone().ok_or_else(|| "its binary chunk is missing".to_string()),
        })
        .collect::<Result<_, _>>()?;
    let pictures: Vec<(u32, u32, Vec<u8>)> = gltf
        .images()
        .map(|image| match image.source() {
            gltf::image::Source::Uri { uri, .. } => data_uri(uri).ok_or_else(|| "a picture isn't embedded".to_string()).and_then(|png| png_rgba(&png)),
            gltf::image::Source::View { view, .. } => {
                let data = &buffers[view.buffer().index()][view.offset()..view.offset() + view.length()];
                png_rgba(data)
            }
        })
        .collect::<Result<_, _>>()?;
    let to_game = |v: [f32; 3]| [-v[2], v[0], v[1]];
    let scene = gltf.default_scene().or_else(|| gltf.scenes().next()).ok_or("it has no scene")?;
    let mut parts = Vec::new();
    let mut stack: Vec<(gltf::Node, [[f32; 4]; 4])> = scene.nodes().map(|node| (node, identity())).collect();
    while let Some((node, parent)) = stack.pop() {
        let place = multiply(parent, node.transform().matrix());
        for child in node.children() {
            stack.push((child, place));
        }
        for primitive in node.mesh().iter().flat_map(|mesh| mesh.primitives()) {
            let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(Vec::as_slice));
            let Some(positions) = reader.read_positions() else { continue };
            let positions: Vec<[f32; 3]> = positions.collect();
            let normals: Vec<[f32; 3]> = reader.read_normals().map(|n| n.collect()).unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
            let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|uv| uv.into_f32().collect()).unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
            let indices: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..positions.len() as u32).collect());
            let picture = primitive.material().pbr_metallic_roughness().base_color_texture().map(|info| info.texture().source().index()).unwrap_or(usize::MAX);
            let mut corners = Vec::with_capacity(indices.len() * 8);
            for &i in &indices {
                let i = i as usize;
                let (Some(p), Some(n), Some(uv)) = (positions.get(i), normals.get(i), uvs.get(i)) else { continue };
                let at = to_game(transform_point(place, *p));
                let normal = to_game(transform_vector(place, *n));
                corners.extend_from_slice(&[at[0], at[1], at[2], normal[0], normal[1], normal[2], uv[0], uv[1]]);
            }
            parts.push((corners, picture));
        }
    }
    if parts.is_empty() {
        return Err("it has no meshes".to_string());
    }
    Ok((parts, pictures))
}

fn identity() -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
}

/// Two column-major matrices (glTF's) multiplied: `a` then `b` within it.
fn multiply(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0; 4]; 4];
    for (column, out_column) in out.iter_mut().enumerate() {
        for (row, value) in out_column.iter_mut().enumerate() {
            *value = (0..4).map(|k| a[k][row] * b[column][k]).sum();
        }
    }
    out
}

fn transform_point(m: [[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|row| m[0][row] * p[0] + m[1][row] * p[1] + m[2][row] * p[2] + m[3][row])
}

fn transform_vector(m: [[f32; 4]; 4], v: [f32; 3]) -> [f32; 3] {
    let t = [0, 1, 2].map(|row| m[0][row] * v[0] + m[1][row] * v[1] + m[2][row] * v[2]);
    let length = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-6);
    t.map(|c| c / length)
}

fn modified(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|meta| meta.modified()).ok()
}

impl ObjectsDraw {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("sight's objects"), source: wgpu::ShaderSource::Wgsl(include_str!("models.wgsl").into()) });
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sight's objects' view"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }],
        });
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sight's objects' pictures"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("sight's objects"), bind_group_layouts: &[Some(&view_layout), Some(&texture_layout)], immediate_size: 0 });
        let depth = Some(wgpu::DepthStencilState { format: DEPTH, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Greater), stencil: Default::default(), bias: Default::default() });
        let target = [Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })];
        let float = |offset: u64, location: u32, format: wgpu::VertexFormat| wgpu::VertexAttribute { offset, shader_location: location, format };
        let corner_attributes = [float(0, 0, wgpu::VertexFormat::Float32x3), float(12, 1, wgpu::VertexFormat::Float32x3), float(24, 2, wgpu::VertexFormat::Float32x2)];
        let placed_attributes = [float(0, 3, wgpu::VertexFormat::Float32x3), float(12, 4, wgpu::VertexFormat::Float32x3), float(24, 5, wgpu::VertexFormat::Float32x4), float(40, 6, wgpu::VertexFormat::Float32x3)];
        let models_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sight's models"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("model_corner"),
                buffers: &[
                    Some(wgpu::VertexBufferLayout { array_stride: 32, step_mode: wgpu::VertexStepMode::Vertex, attributes: &corner_attributes }),
                    Some(wgpu::VertexBufferLayout { array_stride: PLACED_FLOATS as u64 * 4, step_mode: wgpu::VertexStepMode::Instance, attributes: &placed_attributes }),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("model_pixel"), targets: &target, compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: depth.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sprite_attributes = [float(0, 0, wgpu::VertexFormat::Float32x3), float(12, 1, wgpu::VertexFormat::Float32x4), float(28, 2, wgpu::VertexFormat::Float32x4)];
        let sprites_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sight's sprites"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("sprite_corner"),
                buffers: &[Some(wgpu::VertexBufferLayout { array_stride: SPRITE_FLOATS as u64 * 4, step_mode: wgpu::VertexStepMode::Instance, attributes: &sprite_attributes })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("sprite_pixel"), targets: &target, compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: depth,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        // The glows: added to the picture, by no depth (the glows' pass says which are seen).
        let glow_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("sight's far glows"), bind_group_layouts: &[Some(&view_layout)], immediate_size: 0 });
        let place_attributes = [float(0, 0, wgpu::VertexFormat::Float32x4)];
        let glow_attributes = [float(0, 1, wgpu::VertexFormat::Float32x3), float(12, 2, wgpu::VertexFormat::Float32)];
        let add = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
        let glows_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sight's far glows"),
            layout: Some(&glow_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("glow_corner"),
                buffers: &[
                    Some(wgpu::VertexBufferLayout { array_stride: 16, step_mode: wgpu::VertexStepMode::Instance, attributes: &place_attributes }),
                    Some(wgpu::VertexBufferLayout { array_stride: GLOW_FLOATS as u64 * 4, step_mode: wgpu::VertexStepMode::Instance, attributes: &glow_attributes }),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("glow_pixel"),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState { color: add, alpha: add }), write_mask: wgpu::ColorWrites::COLOR })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: DEPTH, depth_write_enabled: Some(false), depth_compare: Some(wgpu::CompareFunction::Always), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let view = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's objects' view"), size: VIEW_ROWS as u64 * 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let view_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("sight's objects' view"), layout: &view_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: view.as_entire_binding() }] });
        // Pixel art: each texel a square as painted.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("sight's objects' pictures"), mag_filter: wgpu::FilterMode::Nearest, min_filter: wgpu::FilterMode::Nearest, ..Default::default() });
        Self { models_pipeline, sprites_pipeline, glows_pipeline, view, view_group, texture_layout, sampler, models: HashMap::new(), strips: HashMap::new(), frame: ObjectsFrame::default(), problems: Vec::new() }
    }

    /// The far glows' places this frame (as many as are drawn), for the glows' pass to say which are seen.
    pub fn glow_places(&self) -> Vec<[f32; 3]> {
        self.frame.glows.iter().take(super::lights::GLOWS_MOST).map(|glow| glow.at).collect()
    }

    /// Whether there's anything to draw this frame.
    pub fn has_work(&self) -> bool {
        !(self.frame.models.is_empty() && self.frame.sprites.is_empty() && self.frame.glows.is_empty())
    }

    /// What to draw this frame.
    pub fn set(&mut self, frame: ObjectsFrame) {
        self.frame = frame;
    }

    /// What couldn't be drawn since last asked (a model or strip that couldn't be read), for the log.
    pub fn take_problems(&mut self) -> Vec<String> {
        std::mem::take(&mut self.problems)
    }

    /// A picture on the card, as painted (read as light), with its sampler.
    fn picture(&self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, rgba: &[u8]) -> wgpu::BindGroup {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("an object's picture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            rgba,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(width * 4), rows_per_image: Some(height) },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        let view = texture.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("an object's picture"),
            layout: &self.texture_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) }],
        })
    }

    /// A model on the card, read again if its file changed.
    fn ensure_model(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, name: &str) {
        let path: PathBuf = models_folder().join(name);
        let when = modified(&path);
        if self.models.get(name).is_some_and(|kept| kept.modified == when) {
            return;
        }
        let read = std::fs::read(&path).map_err(|error| error.to_string()).and_then(|bytes| read_model(&bytes));
        let value = match read {
            Ok((parts, pictures)) => {
                let mut textures: Vec<wgpu::BindGroup> = pictures.iter().map(|(w, h, rgba)| self.picture(device, queue, *w, *h, rgba)).collect();
                // A part with no picture wears white.
                let white = textures.len();
                textures.push(self.picture(device, queue, 1, 1, &[255, 255, 255, 255]));
                let parts = parts
                    .into_iter()
                    .filter(|(corners, _)| !corners.is_empty())
                    .map(|(corners, picture)| {
                        use wgpu::util::DeviceExt;
                        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("a model's part"), contents: &bytes(&corners), usage: wgpu::BufferUsages::VERTEX });
                        Part { corners: buffer, count: (corners.len() / 8) as u32, texture: if picture < white { picture } else { white } }
                    })
                    .collect();
                Some(Model { parts, textures })
            }
            Err(error) => {
                self.problems.push(format!("The model {name} couldn't be drawn: {error}"));
                None
            }
        };
        self.models.insert(name.to_string(), Kept { modified: when, value });
    }

    /// A sprite strip on the card, read again if its file changed.
    fn ensure_strip(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, name: &str) {
        let path: PathBuf = sprites_folder().join(name);
        let when = modified(&path);
        if self.strips.get(name).is_some_and(|kept| kept.modified == when) {
            return;
        }
        let value = match std::fs::read(&path).map_err(|error| error.to_string()).and_then(|bytes| read_strip(&bytes)) {
            Ok(strip) => {
                let group = self.picture(device, queue, strip.width, strip.height, &strip.rgba);
                Some(StripOnCard { strip, group })
            }
            Err(error) => {
                self.problems.push(format!("The sprite strip {name} couldn't be drawn: {error}"));
                None
            }
        };
        self.strips.insert(name.to_string(), Kept { modified: when, value });
    }

    /// The objects drawn onto `target`, into `depth` (the march's, from the upscale), from `view`.
    /// Drawn into `target` and `depth`; the glows' places (and whether each is seen) in `glow_places`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, depth: &wgpu::TextureView, view: ObjectsView, glow_places: &wgpu::Buffer, timestamp_writes: Option<wgpu::RenderPassTimestampWrites>) {
        if !self.has_work() {
            return;
        }
        let rows: [[f32; 4]; VIEW_ROWS] = [
            [view.eye[0], view.eye[1], view.eye[2], 0.0],
            [view.forward[0], view.forward[1], view.forward[2], 0.0],
            [view.right[0], view.right[1], view.right[2], 0.0],
            [view.up[0], view.up[1], view.up[2], 0.0],
            [view.half[0], view.half[1], NEAREST, 0.0],
            [view.sun[0], view.sun[1], view.sun[2], view.ambient],
            [view.daylight[0], view.daylight[1], view.screen[0], view.screen[1]],
        ];
        queue.write_buffer(&self.view, 0, &bytes(rows.as_flattened()));
        // Each model's placings, together.
        let frame = std::mem::take(&mut self.frame);
        let mut by_model: Vec<(String, Vec<f32>)> = Vec::new();
        for model in &frame.models {
            self.ensure_model(device, queue, &model.model);
            let turn = (model.facing % 4) as f32 * std::f32::consts::FRAC_PI_2;
            let placed = [
                model.feet[0], model.feet[1], model.feet[2],
                model.model_feet[0], model.model_feet[1], model.model_feet[2],
                turn.cos(), turn.sin(), model.scale, model.sky,
                model.glow[0], model.glow[1], model.glow[2],
            ];
            match by_model.iter_mut().find(|(name, _)| *name == model.model) {
                Some((_, list)) => list.extend_from_slice(&placed),
                None => by_model.push((model.model.clone(), placed.to_vec())),
            }
        }
        let mut by_strip: Vec<(String, Vec<f32>)> = Vec::new();
        for sprite in &frame.sprites {
            self.ensure_strip(device, queue, &sprite.strip);
            let Some(Some(on_card)) = self.strips.get(&sprite.strip).map(|kept| kept.value.as_ref()) else { continue };
            let strip = &on_card.strip;
            let Some(&(_, first_width)) = strip.frames.first() else { continue };
            let k = ((frame.millis / sprite.frame_ms.max(1) as u64 + sprite.phase as u64) % strip.frames.len() as u64) as usize;
            let (left, width) = strip.frames[k];
            let (w, h) = (first_width as f32 * sprite.scale / 16.0, strip.height as f32 * sprite.scale / 16.0);
            let lift = match sprite.anchor {
                Anchor::BottomCentre => h / 2.0,
                Anchor::Centre => 0.0,
                Anchor::TopCentre => -h / 2.0,
            };
            // The frame across the strip, its whole height.
            let sw = strip.width as f32;
            let placed = [sprite.spot[0], sprite.spot[1], sprite.spot[2], w, h, lift, 0.0, left as f32 / sw, 0.0, (left + width) as f32 / sw, 1.0];
            match by_strip.iter_mut().find(|(name, _)| *name == sprite.strip) {
                Some((_, list)) => list.extend_from_slice(&placed),
                None => by_strip.push((sprite.strip.clone(), placed.to_vec())),
            }
        }
        let glows: Vec<f32> = frame.glows.iter().take(super::lights::GLOWS_MOST).flat_map(|glow| [glow.colour[0], glow.colour[1], glow.colour[2], glow.size]).collect();
        use wgpu::util::DeviceExt;
        let glow_buffer = (!glows.is_empty()).then(|| device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's far glows' lights"), contents: &bytes(&glows), usage: wgpu::BufferUsages::VERTEX }));
        let model_buffers: Vec<(String, wgpu::Buffer, u32)> = by_model
            .into_iter()
            .map(|(name, list)| {
                let count = (list.len() / PLACED_FLOATS) as u32;
                (name, device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("a model's placings"), contents: &bytes(&list), usage: wgpu::BufferUsages::VERTEX }), count)
            })
            .collect();
        let strip_buffers: Vec<(String, wgpu::Buffer, u32)> = by_strip
            .into_iter()
            .map(|(name, list)| {
                let count = (list.len() / SPRITE_FLOATS) as u32;
                (name, device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("a strip's sprites"), contents: &bytes(&list), usage: wgpu::BufferUsages::VERTEX }), count)
            })
            .collect();
        let colour = wgpu::RenderPassColorAttachment { view: target, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let depth_attachment = wgpu::RenderPassDepthStencilAttachment { view: depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }), stencil_ops: None };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("sight's objects"), color_attachments: &[Some(colour)], depth_stencil_attachment: Some(depth_attachment), timestamp_writes, ..Default::default() });
        pass.set_bind_group(0, &self.view_group, &[]);
        pass.set_pipeline(&self.models_pipeline);
        for (name, placings, count) in &model_buffers {
            let Some(Some(model)) = self.models.get(name).map(|kept| kept.value.as_ref()) else { continue };
            pass.set_vertex_buffer(1, placings.slice(..));
            for part in &model.parts {
                pass.set_bind_group(1, &model.textures[part.texture], &[]);
                pass.set_vertex_buffer(0, part.corners.slice(..));
                pass.draw(0..part.count, 0..*count);
            }
        }
        pass.set_pipeline(&self.sprites_pipeline);
        for (name, sprites, count) in &strip_buffers {
            let Some(Some(strip)) = self.strips.get(name).map(|kept| kept.value.as_ref()) else { continue };
            pass.set_bind_group(1, &strip.group, &[]);
            pass.set_vertex_buffer(0, sprites.slice(..));
            pass.draw(0..6, 0..*count);
        }
        if let Some(lights) = &glow_buffer {
            pass.set_pipeline(&self.glows_pipeline);
            pass.set_bind_group(0, &self.view_group, &[]);
            pass.set_vertex_buffer(0, glow_places.slice(..));
            pass.set_vertex_buffer(1, lights.slice(..));
            pass.draw(0..6, 0..(glows.len() / GLOW_FLOATS) as u32);
        }
    }
}

/// Floats in a model's placing, a sprite's, and the view's rows.
const PLACED_FLOATS: usize = 13;
const SPRITE_FLOATS: usize = 11;
const GLOW_FLOATS: usize = 4;
const VIEW_ROWS: usize = 7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_uris_are_read() {
        assert_eq!(data_uri("data:application/octet-stream;base64,TWFu").unwrap(), b"Man");
        assert_eq!(data_uri("data:image/png;base64,TWE=").unwrap(), b"Ma");
        assert!(data_uri("braziere.bin").is_none());
    }

    /// David's brazier, as its glTF has it: every mesh read, a picture for each, standing about a block and a half
    /// tall in the game's axes (Z up) once scaled as its shape is.
    #[test]
    fn the_brazier_is_read() {
        let bytes = std::fs::read(models_folder().join("braziere.gltf")).expect("the brazier's model");
        let (parts, pictures) = read_model(&bytes).expect("read");
        assert!(parts.len() >= 9, "{} parts", parts.len());
        assert!(!pictures.is_empty());
        assert!(parts.iter().all(|(corners, picture)| corners.len() % 24 == 0 && *picture < pictures.len()));
        let shape = crate::data::read_shape(&bytes, Some(1.5)).expect("its shape");
        let heights: Vec<f32> = parts.iter().flat_map(|(corners, _)| corners.chunks(8).map(|c| c[2])).collect();
        let (low, high) = heights.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &z| (lo.min(z), hi.max(z)));
        let tall = (high - low) * shape.scale;
        assert!((tall - 1.5).abs() < 0.05, "{tall} blocks tall");
        assert!((low - shape.feet[1]).abs() < 1e-3, "its feet's height is its lowest corner (glTF Y: {} against {low})", shape.feet[1]);
    }
}

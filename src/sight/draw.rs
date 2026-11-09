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
    /// The finest layer the pyramid may go to (screen pixels a side of its pixels: 8, 4, 2, 1.5 or 1); rays a
    /// pixel; the upscale's mode; and whether the finer layers are marched only where they're worth it (else
    /// everywhere).
    pub grid: f32,
    /// The coarsest layer, where the pyramid starts (8, 4, 2 or 1; everything is marched there) [David
    /// 2026-11-08: "a set of selections for coarsest rays, so i can experiment with nixing the 8x8 layer"].
    pub coarsest: u32,
    /// With the finest at 1 × 1, a layer of 1.5 between it and 2 × 2 [David 2026-11-08: "perhaps to help blend
    /// the 1x1 to 2x2, which is a little obvious sometimes when you can look up a big slope"].
    pub between: bool,
    /// Each seam's fades, by its finer layer (`SEAMS`: 4 × 4, 2 × 2, 1.5, 1 × 1) [David 2026-11-08: "i'd like to get
    /// at least 2x2 to 8x8 working to where i can 'valve' them together"]: among the blocks near the eye, where
    /// the finer layer fades out (shares of the blocks' reach, from and to); past the blocks, its fade in (where it
    /// starts, its pixel this many times its cells, and over how many doublings of distance).
    pub blocks_fades: [[f32; 2]; 4],
    pub cells_fades: [[f32; 2]; 4],
    pub rays: u32,
    pub upscale: u32,
    pub earned: bool,
    /// Towards the sun (game axes, a whole way long): the Render page's sun (`KeptSettings::sight_sun`), below
    /// the horizon at night (the World Compiler's pictures keep theirs over it).
    pub sun: [f32; 3],
    /// What light is left when the sun is well down (0 to 1 of the sky's by day: starlight, moonlight) [David
    /// 2026-11-09, darkness before lights; "Light is precious": nights very dark].
    pub night_light: f32,
    /// Sun shadows (3.4b; the Render page's switch).
    pub shadows: bool,
    /// Sky light (3.4b): on or off, its strength (0 to 1: how dark a place seeing no sky goes), and its tint (0
    /// white, 1 a sky blue) [David 2026-11-08: "tint slider is fine, i can't decide what color the light should be
    /// at this stage anyways"].
    pub sky: bool,
    pub sky_strength: f32,
    pub sky_tint: f32,
    /// Fog (3.4c): on or off; how far one sees at sea level (blocks: land there is mostly fog); how high the air
    /// thins (blocks: its thickness halves each so many up); and how much warmer it is towards the sun (0 to 1).
    pub fog: bool,
    pub fog_reach: f32,
    pub fog_height: f32,
    pub fog_warmth: f32,
    /// Block textures on the near faces (3.6), and how sharp: a shift in the shrunk versions taken (above 0
    /// crisper, below smoother).
    pub textures: bool,
    pub texture_sharpness: f32,
    /// Placed lights (build 95): on or off; how much they flicker (0 none, 1 down to dark); and the time they
    /// flicker by (seconds, sped up as chosen).
    pub placed_lights: bool,
    pub flicker: f32,
    pub flicker_time: f32,
    /// Placed light blended across a face, not a block's step at a time (off unless chosen: David wants the steps).
    pub smooth_light: bool,
}

impl Default for Eye {
    /// The settings' defaults, looking north from over the middle (the tests start from it).
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 200.0],
            looking: [1.0, 0.0, 0.0],
            field_of_view: 70.0,
            grid: 2.0,
            coarsest: 4,
            between: false,
            blocks_fades: [BLOCKS_FADE; 4],
            cells_fades: [CELLS_FADE; 4],
            rays: 1,
            upscale: 0,
            earned: true,
            sun: NOON,
            night_light: NIGHT_LIGHT,
            shadows: true,
            sky: true,
            sky_strength: SKY_STRENGTH,
            sky_tint: SKY_TINT,
            fog: true,
            fog_reach: FOG_REACH,
            fog_height: FOG_HEIGHT,
            fog_warmth: FOG_WARMTH,
            textures: true,
            texture_sharpness: 0.0,
            placed_lights: true,
            flicker: FLICKER,
            flicker_time: 0.0,
            smooth_light: false,
        }
    }
}

/// The fades' defaults: among the blocks, a finer layer out over the last quarter of their reach; past them, in
/// from where its pixel is as big as its cells, over one doubling (what builds 74 to 81 did).
pub const BLOCKS_FADE: [f32; 2] = [0.75, 1.0];
pub const CELLS_FADE: [f32; 2] = [1.0, 1.0];

/// The seams between layers, each by its finer layer: 4 × 4 (into 8 × 8), 2 × 2, 1.5 and 1 × 1.
pub const SEAMS: [f32; 4] = [4.0, 2.0, 1.5, 1.0];

/// Which seam a finer layer of this factor makes with the one above it (its place in `SEAMS`).
pub fn seam(finer: f32) -> usize {
    if finer >= 3.0 {
        0
    } else if finer >= 1.9 {
        1
    } else if finer >= 1.4 {
        2
    } else {
        3
    }
}

/// Sky light's defaults: a place seeing no sky keeps a fifth of the sky's light; the tint a little towards blue.
pub const SKY_STRENGTH: f32 = 0.8;
pub const SKY_TINT: f32 = 0.3;
/// How fast a sky cone widens (blocks across a block of distance): wide, so it takes few steps and sees the land
/// round about softly.
const SKY_SPREAD: f32 = 0.6;

/// Fog's defaults (3.4c): at sea level, land 3,200 blocks off (about 2 miles: David 2026-10-28) is nineteen
/// parts in twenty fog; the air's thickness halves every 200 blocks up, so valleys fill with haze and peaks stand
/// clear; a little warmer towards the sun.
pub const FOG_REACH: f32 = 3200.0;
pub const FOG_HEIGHT: f32 = 200.0;
pub const FOG_WARMTH: f32 = 0.3;

/// How fast a shadow ray widens (blocks across a block of distance, 3.4b): wider than the sun itself (about
/// 0.01), so a shadow ray's steps grow with the distance and a far ridge's shadow comes soft.
const SHADOW_SPREAD: f32 = 0.04;

/// The first sight's noon sun, high in the south, a little east ("a fixed noon sun"): the tests' sun.
pub const NOON: [f32; 3] = [-0.35, 0.2, 0.91];
const AMBIENT: f32 = 0.35;

/// Night's light by default: a twenty-fifth of the sky's by day, so a night is very dark but the land's shape
/// still shows where it sees the sky (and none where it doesn't: caves pitch black).
pub const NIGHT_LIGHT: f32 = 0.04;

/// How much placed lights flicker by default, and how fast (the old renderer's: `flicker`, `flicker_speed`).
pub const FLICKER: f32 = 0.15;
pub const FLICKER_SPEED: f32 = 1.5;

/// The light by the sun's height (`sun_up`: the sine of its height over the horizon): how strong its own light
/// is (full from 4° up, gone 2° under, as its disc sinks behind the horizon), and how bright the sky is (full from
/// 4° up, fading through the dusk to nothing 18° under, when the last of the glow has gone: a dusk's first part
/// bright, its last dim).
pub fn daylight(sun_up: f32) -> (f32, f32) {
    let height = sun_up.clamp(-1.0, 1.0).asin().to_degrees();
    let smooth = |from: f32, to: f32| {
        let k = ((height - from) / (to - from)).clamp(0.0, 1.0);
        k * k * (3.0 - 2.0 * k)
    };
    let glow = smooth(-18.0, 4.0);
    (smooth(-2.0, 4.0), glow * glow)
}
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
    /// The card's clock on sight's work (none if the card can't time itself).
    timer: Option<CardTimer>,
    /// The block textures on the card (3.6): the stack, how it's read, the table of each block's faces, the
    /// registry's revision they were made from, and how many times they've been made (the layers' bindings are
    /// made again when it changes).
    textures: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,
    texture_table: wgpu::Buffer,
    textures_revision: Option<u64>,
    textures_made: u64,
    /// The media table (build 98, `BlockTextures::media_table`), sent with the textures.
    media: wgpu::Buffer,
    /// The objects' models and sprites (build 96), drawn after the upscale into its depth.
    pub objects: super::models::ObjectsDraw,
}

/// One layer of the pyramid: its factor (screen pixels a side of a pixel), its size, its picture and distances,
/// its camera numbers, and its bindings.
struct Layer {
    factor: f32,
    size: (u32, u32),
    picture: wgpu::TextureView,
    far: wgpu::TextureView,
    camera: wgpu::Buffer,
    group: wgpu::BindGroup,
}

struct Targets {
    screen: (u32, u32),
    /// The layers' factors, coarsest first.
    factors: Vec<f32>,
    store: Arc<CardStore>,
    /// The textures they were bound with (`SightDraw::textures_made`).
    textures_made: u64,
    /// The start pass's tiles across and down.
    tiles: (u32, u32),
    layers: Vec<Layer>,
    start_group: wgpu::BindGroup,
    upscale_group: wgpu::BindGroup,
    /// The screen's depth (build 96): the upscale's, from the march's distances, that the objects are drawn into.
    depth: wgpu::TextureView,
}

fn buffer(b: &wgpu::Buffer) -> wgpu::BindingResource<'_> {
    b.as_entire_binding()
}

impl SightDraw {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
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
            // The depth the objects are drawn into (build 96): written whole, every pixel.
            depth_stencil: Some(wgpu::DepthStencilState { format: super::models::DEPTH, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Always), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let up = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's upscale numbers"), size: 3 * 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("sight's blend"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });
        // Until the registry's textures come: one texture of one texel, and a table with none.
        let textures = Self::texture_stack(device, queue, &crate::sight::textures::BlockTextures::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sight's block textures"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            // Up close a texel is a square, as painted; shrinking, the versions blend so far faces don't shimmer.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let texture_table = {
            use wgpu::util::DeviceExt;
            let table = crate::sight::textures::BlockTextures::default().table();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's block textures' table"), contents: &table.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST })
        };
        let objects = super::models::ObjectsDraw::new(device, format);
        let media = {
            use wgpu::util::DeviceExt;
            let table = crate::sight::textures::BlockTextures::default().media_table();
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sight's media"), contents: &table.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST })
        };
        Self { march, start, upscale, up, sampler, targets: None, timer: CardTimer::new(device, queue), textures, texture_sampler, texture_table, textures_revision: None, textures_made: 0, media, objects }
    }

    /// The registry revision the block textures on the card were made from (none yet).
    pub fn textures_revision(&self) -> Option<u64> {
        self.textures_revision
    }

    /// The block textures put on the card, made from the registry's `revision`.
    pub fn set_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, textures: &crate::sight::textures::BlockTextures, revision: u64) {
        self.textures = Self::texture_stack(device, queue, textures);
        let table = textures.table();
        queue.write_buffer(&self.texture_table, 0, &table.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>());
        queue.write_buffer(&self.media, 0, &textures.media_table().iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        self.textures_revision = Some(revision);
        self.textures_made += 1;
    }

    /// The textures as one stack on the card, each with its shrunk versions (stored as painted, read as light).
    fn texture_stack(device: &wgpu::Device, queue: &wgpu::Queue, textures: &crate::sight::textures::BlockTextures) -> wgpu::TextureView {
        use crate::sight::textures::MIPS;
        let (side, mips, layers) = if textures.layers.is_empty() { (1, 1, 1) } else { (crate::look::TEXTURE_SIZE, MIPS, textures.layers.len() as u32) };
        let stack = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sight's block textures"),
            size: wgpu::Extent3d { width: side, height: side, depth_or_array_layers: layers },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let empty = vec![vec![vec![255u8, 0, 255, 255]]];
        let all = if textures.layers.is_empty() { &empty } else { &textures.layers };
        for (layer, versions) in all.iter().enumerate() {
            for (mip, pixels) in versions.iter().enumerate().take(mips as usize) {
                let size = (side >> mip).max(1);
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: &stack, mip_level: mip as u32, origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 }, aspect: wgpu::TextureAspect::All },
                    pixels,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size * 4), rows_per_image: Some(size) },
                    wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
                );
            }
        }
        stack.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() })
    }

    /// The pyramid's layers for a screen of `screen`, one a factor (coarsest first), and their bindings to
    /// `store`, made again when any of them changes (a moment's work: the store is the world's, not the screen's).
    fn targets(&mut self, device: &wgpu::Device, screen: (u32, u32), factors: &[f32], store: &Arc<CardStore>) -> &Targets {
        let stale = self.targets.as_ref().is_none_or(|t| t.screen != screen || t.factors != factors || !Arc::ptr_eq(&t.store, store) || t.textures_made != self.textures_made);
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
            let mut layers: Vec<Layer> = Vec::new();
            let mut worth_above: Option<wgpu::Buffer> = None;
            let mut far_above: Option<wgpu::TextureView> = None;
            for &factor in factors {
                let size = ((screen.0 as f32 / factor).ceil().max(1.0) as u32, (screen.1 as f32 / factor).ceil().max(1.0) as u32);
                let picture = texture("sight's layer", size, wgpu::TextureFormat::Rgba8Unorm);
                let far = texture("sight's layer's distances", size, wgpu::TextureFormat::R32Float);
                let worth = storage("sight's worth it", size.0 * size.1);
                let camera = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's camera"), size: CAMERA_ROWS as u64 * 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
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
                        wgpu::BindGroupEntry { binding: 11, resource: wgpu::BindingResource::TextureView(&self.textures) },
                        wgpu::BindGroupEntry { binding: 12, resource: wgpu::BindingResource::Sampler(&self.texture_sampler) },
                        wgpu::BindGroupEntry { binding: 13, resource: buffer(&self.texture_table) },
                        wgpu::BindGroupEntry { binding: 14, resource: buffer(&store.pieces) },
                        wgpu::BindGroupEntry { binding: 15, resource: buffer(&store.lights) },
                        wgpu::BindGroupEntry { binding: 16, resource: buffer(&self.media) },
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
            let picture_of = |factor: f32| &layers.iter().find(|l| (l.factor - factor).abs() < 0.01).unwrap_or(&layers[0]).picture;
            let far_of = |factor: f32| &layers.iter().find(|l| (l.factor - factor).abs() < 0.01).unwrap_or(&layers[0]).far;
            let upscale_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sight's upscale"),
                layout: &self.upscale.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(picture_of(8.0)) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(picture_of(4.0)) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(picture_of(2.0)) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&layers[0].far) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 5, resource: buffer(&self.up) },
                    wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(picture_of(1.0)) },
                    wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(picture_of(1.5)) },
                    wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(far_of(4.0)) },
                    wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::TextureView(far_of(2.0)) },
                    wgpu::BindGroupEntry { binding: 10, resource: wgpu::BindingResource::TextureView(far_of(1.0)) },
                    wgpu::BindGroupEntry { binding: 11, resource: wgpu::BindingResource::TextureView(far_of(1.5)) },
                ],
            });
            let depth = device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("sight's depth"),
                    size: wgpu::Extent3d { width: screen.0, height: screen.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: super::models::DEPTH,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&Default::default());
            self.targets = Some(Targets { screen, factors: factors.to_vec(), store: store.clone(), textures_made: self.textures_made, tiles, layers, start_group, upscale_group, depth });
        }
        self.targets.as_ref().expect("made just now")
    }

    /// The pyramid marched and brought up onto `view` (the screen's picture, `screen` pixels).
    pub fn frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, screen: (u32, u32), eye: Eye, store: &Arc<CardStore>) {
        let factors = factors(eye);
        let (coarsest, finest) = (factors[0], factors[factors.len() - 1]);
        let has_between = factors.iter().any(|&f| (f - 1.5).abs() < 0.01);
        let tiles = self.targets(device, screen, &factors, store).tiles;
        // The camera: the eye, and the directions of the screen's edges (game axes: looking north, east is right).
        let forward = normalise(eye.looking);
        let right = normalise(cross([0.0, 0.0, 1.0], forward));
        let up = cross(forward, right);
        let tan_half = (eye.field_of_view.to_radians() / 2.0).tan();
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let no_start = std::env::var("INCUS_SIGHT_NO_START").is_ok();
        let no_follow = std::env::var("INCUS_SIGHT_NO_FOLLOW").is_ok();
        let no_shadows = std::env::var("INCUS_SIGHT_NO_SHADOWS").is_ok();
        let no_sky = std::env::var("INCUS_SIGHT_NO_SKY").is_ok();
        let no_fog = std::env::var("INCUS_SIGHT_NO_FOG").is_ok();
        let no_textures = std::env::var("INCUS_SIGHT_NO_TEXTURES").is_ok();
        let no_lights = std::env::var("INCUS_SIGHT_NO_LIGHTS").is_ok();
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
        // The time of day (the sun's height): its own light and the sky's; with the sun down, no shadow rays.
        let (direct, glow) = daylight(normalise(eye.sun)[2]);
        let night = eye.night_light.clamp(0.0, 1.0);
        for (k, layer) in targets.layers.iter().enumerate() {
            let pixel = 2.0 * tan_half / layer.size.1 as f32;
            let above_width = if k == 0 { 0.0 } else { targets.layers[k - 1].size.0 as f32 };
            let below = if k + 1 < targets.layers.len() { 1.0 } else { 0.0 };
            // The layer above's factor and the one below's, as shares of this one's (0: none).
            let to_above = if k == 0 { 0.0 } else { layer.factor / targets.layers[k - 1].factor };
            let child = targets.layers.get(k + 1).map_or(0.0, |below| below.factor);
            let to_child = child / layer.factor;
            // The seam with the next finer layer: its fades, by that layer's factor.
            let fade = eye.blocks_fades[seam(child)];
            let cells_fade = eye.cells_fades[seam(child)];
            let numbers: [[f32; 4]; CAMERA_ROWS] = [
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
                [tiles.0 as f32, tiles.1 as f32, if no_start { 0.0 } else { (TILE as f32 / layer.factor).max(1.0) }, screen_pixel * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
                [above_width, below, if eye.earned { 1.0 } else { 0.0 }, live_reach],
                // Each finer ray from short of its parent's nearest touch (`INCUS_SIGHT_NO_FOLLOW` leaves it out);
                // sun shadows (`INCUS_SIGHT_NO_SHADOWS` leaves them out, for measuring), and their widening.
                [if no_follow { 0.0 } else { 1.0 }, if eye.shadows && !no_shadows && direct > 0.0 { 1.0 } else { 0.0 }, SHADOW_SPREAD, 0.0],
                // The pyramid's steps: this layer to the one above, the one below to this, and where the one below
                // fades out among the blocks (shares of their reach).
                [to_above, to_child, fade[0].min(fade[1]), fade[1]],
                // Past the blocks, the one below's fade in: where it starts and over how many doublings.
                [cells_fade[0].max(0.1), cells_fade[1].max(0.05), 0.0, 0.0],
                // Sky light (`INCUS_SIGHT_NO_SKY` leaves it out, for measuring): on, strength, tint, its cones' width.
                [if eye.sky && !no_sky { 1.0 } else { 0.0 }, eye.sky_strength.clamp(0.0, 1.0), eye.sky_tint.clamp(0.0, 1.0), SKY_SPREAD],
                // Fog (`INCUS_SIGHT_NO_FOG` leaves it out, for measuring): its thickness at sea level (a block's
                // worth: three over the reach, so land there is nineteen parts in twenty fog; 0 none), how fast it
                // thins going up (a block's worth), sea level, and its warmth towards the sun.
                [
                    if eye.fog && !no_fog { 3.0 / eye.fog_reach.max(10.0) } else { 0.0 },
                    std::f32::consts::LN_2 / eye.fog_height.max(1.0),
                    crate::world::SEA_LEVEL as f32,
                    eye.fog_warmth.clamp(0.0, 1.0),
                ],
                // Block textures (3.6; `INCUS_SIGHT_NO_TEXTURES` leaves them out, for measuring): on, and how sharp.
                [if eye.textures && !no_textures { 1.0 } else { 0.0 }, eye.texture_sharpness.clamp(-4.0, 4.0), 0.0, 0.0],
                // The time of day: the sun's own light, the sky's light (night's at the least), the sky's
                // brightness, and night's light.
                [direct, night + (1.0 - night) * glow, glow, night],
                // Placed lights (build 95; `INCUS_SIGHT_NO_LIGHTS` leaves them out, for measuring): on, how much they
                // flicker, the time they flicker by (kept under an hour's worth, so the card's sums stay fine).
                [if eye.placed_lights && !no_lights { 1.0 } else { 0.0 }, eye.flicker.clamp(0.0, 1.0), eye.flicker_time.rem_euclid(3600.0), if eye.smooth_light { 1.0 } else { 0.0 }],
            ];
            queue.write_buffer(&layer.camera, 0, &numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        }
        let up_numbers: [[f32; 4]; 3] = [[screen.0 as f32, screen.1 as f32, finest, coarsest], [eye.upscale as f32, if has_between { 1.0 } else { 0.0 }, 0.0, 0.0], [tan_half * aspect, tan_half, super::models::NEAREST, 0.0]];
        queue.write_buffer(&self.up, 0, &up_numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        // The card's own clock on sight's work, from its first pass's start to the upscale's end, when it can.
        let timed = self.timer.as_mut().and_then(|timer| timer.begin());
        let queries = self.timer.as_ref().map(|timer| &timer.queries);
        let mut first = timed.and(queries);
        if !no_start {
            let timestamp_writes = first.take().map(|query_set| wgpu::ComputePassTimestampWrites { query_set, beginning_of_pass_write_index: Some(0), end_of_pass_write_index: None });
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("sight's start pass"), timestamp_writes });
            pass.set_pipeline(&self.start);
            pass.set_bind_group(0, &targets.start_group, &[]);
            pass.dispatch_workgroups(tiles.0.div_ceil(8), tiles.1.div_ceil(8), 1);
        }
        for layer in &targets.layers {
            let timestamp_writes = first.take().map(|query_set| wgpu::ComputePassTimestampWrites { query_set, beginning_of_pass_write_index: Some(0), end_of_pass_write_index: None });
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("sight's march"), timestamp_writes });
            pass.set_pipeline(&self.march);
            pass.set_bind_group(0, &layer.group, &[]);
            pass.dispatch_workgroups(layer.size.0.div_ceil(8), layer.size.1.div_ceil(8), 1);
        }
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } };
        let timestamp_writes = timed.and(queries).map(|query_set| wgpu::RenderPassTimestampWrites { query_set, beginning_of_pass_write_index: None, end_of_pass_write_index: Some(1) });
        let depth = wgpu::RenderPassDepthStencilAttachment { view: &targets.depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Store }), stencil_ops: None };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("sight's upscale"), color_attachments: &[Some(attachment)], depth_stencil_attachment: Some(depth), timestamp_writes, ..Default::default() });
        pass.set_pipeline(&self.upscale);
        pass.set_bind_group(0, &targets.upscale_group, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        // The objects, into the depth the upscale left (build 96).
        let objects_view = super::models::ObjectsView {
            eye: eye.position,
            forward,
            right,
            up,
            half: [tan_half * aspect, tan_half],
            sun: normalise(eye.sun),
            ambient: AMBIENT,
            daylight: [direct, night + (1.0 - night) * glow],
        };
        let depth_view = targets.depth.clone();
        self.objects.draw(device, queue, encoder, view, &depth_view, objects_view);
        if let (Some(k), Some(timer)) = (timed, &self.timer) {
            timer.resolve(encoder, k);
        }
    }

    /// After the frame's work is handed to the card: this frame's clock asked for, and any come back read
    /// (never waiting).
    pub fn after_submit(&mut self, device: &wgpu::Device) {
        if let Some(timer) = &mut self.timer {
            timer.after_submit(device);
        }
    }
}

/// The pyramid's layers for `eye`, coarsest first: halving from the coarsest to the finest (never coarser than
/// the coarsest), with a layer of 1.5 at the end if the finest is 1.5, or between 2 × 2 and 1 × 1 if asked.
pub fn factors(eye: Eye) -> Vec<f32> {
    let coarsest = match eye.coarsest {
        0..=1 => 1.0,
        2 => 2.0,
        3..=4 => 4.0,
        _ => 8.0,
    };
    let finest = if eye.grid < 1.25 {
        1.0
    } else if eye.grid < 1.75 {
        1.5
    } else if eye.grid < 3.0 {
        2.0
    } else if eye.grid < 6.0 {
        4.0
    } else {
        8.0
    };
    let finest = if finest > coarsest { coarsest } else { finest };
    let mut factors = vec![coarsest];
    let mut f = coarsest;
    while f > finest + 0.01 {
        if f == 2.0 && finest < 1.9 && (finest > 1.25 || eye.between) {
            factors.push(1.5);
            if finest < 1.25 {
                factors.push(1.0);
            }
            break;
        }
        f /= 2.0;
        factors.push(f);
    }
    factors
}

/// The camera numbers' rows (`Camera` in `march.wgsl`).
const CAMERA_ROWS: usize = 17;

/// What the card took on sight's last frames (milliseconds, smoothed), if it can time itself: the Render page's.
static CARD_MS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn card_ms() -> Option<f32> {
    let ms = f32::from_bits(CARD_MS.load(std::sync::atomic::Ordering::Relaxed));
    (ms > 0.0).then_some(ms)
}

/// The card's clock on sight's work: two timestamps a frame, copied out to one of a few readbacks taken in
/// turn, read when they come back (a frame or two late), never waited for.
struct CardTimer {
    queries: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readbacks: Vec<(wgpu::Buffer, Arc<std::sync::atomic::AtomicU8>)>,
    pending: Option<usize>,
    period_ns: f32,
}

const FREE: u8 = 0;
const MAPPING: u8 = 1;
const READY: u8 = 2;

impl CardTimer {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let queries = device.create_query_set(&wgpu::QuerySetDescriptor { label: Some("sight's clock"), ty: wgpu::QueryType::Timestamp, count: 2 });
        let resolved = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's clock"), size: 16, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readbacks = (0..4).map(|_| (device.create_buffer(&wgpu::BufferDescriptor { label: Some("sight's clock read"), size: 16, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }), Arc::new(std::sync::atomic::AtomicU8::new(FREE)))).collect();
        Some(Self { queries, resolved, readbacks, pending: None, period_ns: queue.get_timestamp_period() })
    }

    /// A free readback for this frame (none free: this frame isn't timed).
    fn begin(&mut self) -> Option<usize> {
        self.pending = self.readbacks.iter().position(|(_, state)| state.load(std::sync::atomic::Ordering::Acquire) == FREE);
        self.pending
    }

    fn resolve(&self, encoder: &mut wgpu::CommandEncoder, k: usize) {
        encoder.resolve_query_set(&self.queries, 0..2, &self.resolved, 0);
        encoder.copy_buffer_to_buffer(&self.resolved, 0, &self.readbacks[k].0, 0, 16);
    }

    fn after_submit(&mut self, device: &wgpu::Device) {
        use std::sync::atomic::Ordering;
        if let Some(k) = self.pending.take() {
            let state = self.readbacks[k].1.clone();
            state.store(MAPPING, Ordering::Release);
            self.readbacks[k].0.slice(..).map_async(wgpu::MapMode::Read, move |result| state.store(if result.is_ok() { READY } else { FREE }, Ordering::Release));
        }
        let _ = device.poll(wgpu::PollType::Poll);
        for (buffer, state) in &self.readbacks {
            if state.load(Ordering::Acquire) != READY {
                continue;
            }
            let stamps = buffer.slice(..).get_mapped_range().ok().map(|bytes| {
                let at = |i: usize| u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap_or_default());
                (at(0), at(1))
            });
            buffer.unmap();
            state.store(FREE, Ordering::Release);
            if let Some((from, to)) = stamps.filter(|(from, to)| to > from) {
                let ms = (to - from) as f32 * self.period_ns / 1e6;
                // Smoothed over about the last twenty frames.
                let before = card_ms().unwrap_or(ms);
                CARD_MS.store((before + (ms - before) * 0.05).to_bits(), Ordering::Relaxed);
            }
        }
    }
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9);
    [v[0] / length, v[1] / length, v[2] / length]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pyramid's layers follow the choices: halving from the coarsest, never coarser than it, 1.5 at the end
    /// or between 2 × 2 and 1 × 1.
    #[test]
    fn each_seam_has_its_own_fades() {
        assert_eq!(SEAMS.map(seam), [0, 1, 2, 3]);
    }

    #[test]
    fn the_layers_follow_the_choices() {
        let layers = |coarsest: u32, grid: f32, between: bool| factors(Eye { coarsest, grid, between, ..Eye::default() });
        assert_eq!(layers(8, 2.0, false), vec![8.0, 4.0, 2.0]);
        assert_eq!(layers(4, 2.0, false), vec![4.0, 2.0], "David's choice");
        assert_eq!(layers(4, 1.0, false), vec![4.0, 2.0, 1.0]);
        assert_eq!(layers(4, 1.5, false), vec![4.0, 2.0, 1.5]);
        assert_eq!(layers(4, 1.0, true), vec![4.0, 2.0, 1.5, 1.0]);
        assert_eq!(layers(2, 1.0, true), vec![2.0, 1.5, 1.0]);
        assert_eq!(layers(1, 1.0, true), vec![1.0], "1 × 1 everywhere: nothing between");
        assert_eq!(layers(2, 8.0, false), vec![2.0], "the finest never coarser than the coarsest");
        assert_eq!(layers(8, 1.5, true), vec![8.0, 4.0, 2.0, 1.5], "between only means anything with 1 × 1");
    }

    /// The time of day: a sun 4° up or higher lights as it always has (the presets' suns from 5° up look as they
    /// did); at the horizon the sun's light is going and the sky's dimming; 18° under, both gone.
    #[test]
    fn the_light_goes_with_the_sun() {
        let at = |degrees: f32| daylight(degrees.to_radians().sin());
        assert_eq!(at(5.0), (1.0, 1.0));
        assert_eq!(at(90.0), (1.0, 1.0));
        let (direct, glow) = at(0.0);
        assert!(direct > 0.2 && direct < 0.6, "the sun half down behind the horizon: {direct}");
        assert!(glow > 0.5 && glow < 0.9, "the sky still bright at sunset: {glow}");
        let (direct, glow) = at(-6.0);
        assert_eq!(direct, 0.0, "the sun's own light gone");
        assert!(glow > 0.1 && glow < 0.5, "the dusk dimming: {glow}");
        assert_eq!(at(-18.0), (0.0, 0.0));
        assert_eq!(at(-30.0), (0.0, 0.0));
        // Steady as the sun sinks.
        let mut last = at(10.0);
        for k in 0..80 {
            let now = at(10.0 - k as f32 * 0.5);
            assert!(now.0 <= last.0 && now.1 <= last.1, "never brighter as it sinks");
            last = now;
        }
    }
}

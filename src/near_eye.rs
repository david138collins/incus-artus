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
/// The near light's directory (build 210): chunks a side (a chunk's entry is at its place wrapped; the chunks kept
/// are never 256 apart), two words a chunk; and the buffer's first size, words (4 MB; doubled when it's full).
const LIGHT_SIDE: i32 = 256;
const LIGHT_DIRECTORY: u64 = (LIGHT_SIDE * LIGHT_SIDE * 2) as u64;
const LIGHT_FIRST: u64 = 1 << 20;
/// The least and most of the detail curve's near number, blocks; and the most of a band's reach its stipple takes.
/// (The least was 1/16 to build 214: a block's picture is 32 squares a side, and a 4-pixel square is no bigger than
/// one of them only to about 5 blocks, which is the 4s' reach at 1/48; so the slider goes under that, build 215.)
pub const DETAILS: (f32, f32) = (0.015625, 1.0);
pub const MOST_STIPPLE: f32 = 0.5;
/// The least and most of a light's step: the share dimmer each block from a lamp is than the last.
pub const STEPS: (f32, f32) = (0.05, 0.5);
/// The far lamps' dots (build 222) [David, 2026-11-18: "About a pixel"]: a lamp's block across (screen pixels) at
/// which its dot begins to show, and at which it's whole; the bar its light must be over to show (a share of its
/// swing) where it's whole, how much that rises, and over how many blocks past there (old's far glows' numbers, build
/// 100: a third, rising to nearly the top 1,500 blocks on); and how far behind a lamp's middle something at its pixel
/// may be and not hide it (its own block's faces are within half a block and a little).
const DOT_FROM: f32 = 2.0;
const DOT_WHOLE: f32 = 1.0;
const DOT_BAR: f32 = 0.3;
const DOT_BAR_RISES: f32 = 0.62;
const DOT_BAR_OVER: f32 = 1500.0;
const DOT_SLACK: f32 = 1.5;
/// Past the near reach (build 223): how many far cells (at the size the far eye reads at a lamp's distance) short of
/// a lamp the ray seeing it stops (the hill a cave mouth is cut into, the lamp's own cell).
const DOT_FAR_SLACK: f32 = 2.0;
/// Seeing the far lamps in turns (build 226): the list is seen to over so many frames, a share of it each (lamp k
/// on frame k mod this), a lamp's answer kept on the card till its turn; the whole list in the frame a new one is put
/// on the card. [David, 2026-11-18, of assess's checks: "make a note of those to add to the things we need to
/// check"; seeing every lamp every frame was the dearest new thing, 0.10 to 0.20 ms on the card.]
const SIGHT_TURNS: u32 = 4;

/// The near eye's bands by the detail curve (build 213), the nearest first: each its squares' side in screen pixels,
/// the distance from the eye (true, not level) nearer than which it draws nothing, and the distance it draws to.
/// The detail curve [`master/plan.md`]: the smallest thing drawn at a distance d is sqrt(a² + (g d / P)²) blocks,
/// which on the glass is a square of sqrt((a P / d)² + g²) pixels; for triangles g is 1, a pixel. A band's squares
/// are the power of two nearest that (4 down to where it's 2.83, 2 to 1.41, then 1): so 4 x 4 squares reach to
/// a P / sqrt(7), 2 x 2 to a P. `largest`: 1 (one band, as before the curve), 2 or 4; `a`: the curve's near number,
/// blocks; `p`: the pixels a block one block away is tall (the picture's height over twice the tangent of half the
/// field of view). `stipple`: the share of a coarse band's reach, at its far end, it thins out over (build 214). A
/// band begins at half of where the stipple of the band nearer than it begins, so the two overlap and the farther
/// one can leave out what the nearer one's picture covers without a gap.
pub fn bands_of(largest: u32, a: f32, p: f32, stipple: f32) -> Vec<(u32, f32, f32)> {
    let a = a.clamp(DETAILS.0, DETAILS.1);
    let reach = |square: f32| a * p / (square * square / 2.0 - 1.0).sqrt();
    let behind = |square: f32| reach(square) * (1.0 - stipple.clamp(0.0, MOST_STIPPLE)) / 2.0;
    match largest {
        0 | 1 => vec![(1, 0.0, f32::MAX)],
        2 | 3 => vec![(2, 0.0, reach(2.0)), (1, behind(2.0), f32::MAX)],
        _ => vec![(4, 0.0, reach(4.0)), (2, behind(4.0), reach(2.0)), (1, behind(2.0), f32::MAX)],
    }
}

/// The most the x-fader's sliders go to: its band's width (blocks) and its curve.
pub const MOST_FADER_WIDTH: f32 = 512.0;
pub const MOST_FADER_CURVE: f32 = 8.0;

/// The near eye's settings (the Render page's "The near eye"): the render radius, chunks; and the x-fader, how blocks
/// hand over to rays [David, 2026-11-16: "i think a x-fader with curves could hide it almost completely so blocks
/// smoothly fade into rays"]: its band's width before the faces' reach (blocks), and its curve (1 a straight fade;
/// higher, an S that holds each side longer and crosses quicker in the middle).
///
/// Kept in the Render page's presets (`presets.rs`), all but the lantern lit or out.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct NearLook {
    pub radius: f32,
    pub fader_width: f32,
    pub fader_curve: f32,
    /// The largest squares the near eye draws in, screen pixels a side (the Render page's "Largest near squares";
    /// "Near pixel size" at build 212): 1, every near pixel the screen's own; 2 or 4, the nearest band of distance
    /// drawn in squares that size, the next in squares half that, down to single pixels, by the detail curve
    /// (`bands_of`) [David, 2026-11-17: "i almost want 4x4 *up close*, fading to 2x2, fading to 1x1"; the control's
    /// place and names his choices].
    pub pixel: u32,
    /// The detail curve's near number, a (the Render page's "Detail up close (blocks)"): the size in the world of
    /// the smallest thing drawn up close, a fraction of a block [David, 2026-11-17: "a sort of linear equivalence of
    /// detail from the back if your hand towards infinity ... a formula with a single flexture"].
    pub detail: f32,
    /// How much of a coarse band's reach, at its far end, it thins out over by a stipple, so it fades into the band
    /// behind (the Render page's "Stipple width (share of the band)", build 214; 0 a hard edge, to a half) [David,
    /// 2026-11-17, of a dissolve or a stipple: "lets try stippling"; the slider's name his choice].
    pub stipple: f32,
    /// How crisp a block's picture stays in the coarse squares (the Render page's "Picture sharpness in the
    /// squares", build 215): 0, the card blends the picture once its squares are smaller than the band's (as to
    /// build 214); 1, a coarse square shows one square of the picture until they're smaller than a screen pixel
    /// [David, 2026-11-17, of build 214 at 4: "the blocks become blurry after like 5 block radius around me"; both
    /// this and a lower detail by his choice, the slider's name his].
    pub sharp: f32,
    /// Placed lights' flicker (the Render page's "Placed lights", build 216): how much of a light's brightness wavers
    /// ("Flicker (share of brightness)"), and how many times a second ("Flicker speed (a second)"). A wobble where
    /// the light is read, so nothing kept changes [the section's place David's choice, 2026-11-17].
    pub flicker: f32,
    pub flicker_speed: f32,
    /// The lantern (build 217): a light at the eye, lit or not (the L key; the Render page's "Lantern" in "Lights"),
    /// its brightness (1 to 15: "Lantern brightness") and its colour ("Lantern colour") [David, 2026-11-17: the
    /// first moving light, the key and the controls' place his choices].
    #[serde(skip)]
    pub lantern: bool,
    pub lantern_level: f32,
    pub lantern_colour: [f32; 3],
    /// How much dimmer each block from a lamp or the lantern is than the last, a share (the Render page's "Light
    /// step (share dimmer each block)" in "Lights", build 218): every step the same size to the eye [David,
    /// 2026-11-17, of build 217: "i couldn't really see the steps in the red and green light levels"; 2026-11-06:
    /// "very clear steps in brightness"; the slider's name his choice].
    pub step: f32,
}

impl Default for NearLook {
    /// The built-in settings (what a preset's file leaves out keeps these).
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl NearLook {
    /// [David, 2026-11-16, asked by Counsel "Make your crossfade settings, 128 wide and curve 1, the defaults?":
    /// "yes, and 32 chunk radius."]
    pub const DEFAULT: Self = Self { radius: RADIUS, fader_width: 128.0, fader_curve: 1.0, pixel: 1, detail: 0.25, stipple: 0.25, sharp: 0.0, flicker: 0.05, flicker_speed: 2.0, lantern: false, lantern_level: 15.0, lantern_colour: [1.0, 0.6, 0.25], step: 0.25 };

    /// The lantern's light: red, green and blue, each 0 to 15: its colour, the strongest of the three at its
    /// brightness, each of the others as many steps under that as its share of the strongest comes to (a step is a
    /// fixed share dimmer where a level is read, `near.wgsl`'s `placed`: so the three fall together and the light
    /// keeps its colour as it fades, till the weakest runs out).
    pub fn lantern_light(&self) -> [u8; 3] {
        let most = self.lantern_colour.iter().fold(1e-3f32, |most, v| most.max(*v));
        let level = self.lantern_level.clamp(1.0, 15.0);
        let kept = (1.0 - self.step.clamp(STEPS.0, STEPS.1)).ln();
        self.lantern_colour.map(|v| (level - (v.max(0.0) / most).max(1e-6).ln() / kept).round().clamp(0.0, 15.0) as u8)
    }
}

/// One of the near eye's pictures (build 213): a band of distance from the eye, drawn in squares of `pixel` screen
/// pixels a side: its colour (each pixel multiplied by how much of it shows) and its depth, `size` in its own pixels.
/// `numbers`: `near.wgsl`'s `Band`. `group`: those numbers and the pictures of the bands nearer than it (which it
/// leaves out what they cover: its faces by the mask laid in its depth before they're drawn, its water by reading
/// them). `depths`: for its water, its own depth and the depths of the bands behind it.
/// `drawn`, `drawn_water`: the frame's draws for it, the blocks' sides' and the water's tops', a page after another:
/// for each page where its draws begin in the near eye's `draws` and how many.
struct Band {
    pixel: u32,
    size: (u32, u32),
    colour: wgpu::TextureView,
    depth: wgpu::TextureView,
    numbers: wgpu::Buffer,
    group: wgpu::BindGroup,
    depths: wgpu::BindGroup,
    drawn: Vec<(usize, u64, u32)>,
    drawn_water: Vec<(usize, u64, u32)>,
}

/// A chunk on the card: its page, where its faces begin there and how many (the blocks' sides in their groups by the
/// way they look, `groups` of each in `near::GROUPS`' order; the last `water` of them the water's tops), and the
/// lowest and highest block they touch.
struct Kept {
    page: usize,
    first: u32,
    count: u32,
    /// Its light on the card, if some of it isn't plain daylight: where it begins in the light's buffer and how many
    /// words.
    light: Option<(u64, u64)>,
    groups: [u32; 6],
    water: u32,
    low: i32,
    high: i32,
}

/// The near light on the card (build 210, David's choice of 2026-11-17): one buffer of words. First the directory, an
/// entry a chunk at (north wrapped) + (east wrapped) * 256: where the chunk's light begins in the buffer (or
/// `NO_BRICK`: all plain daylight, or no chunk), and its first layer of bricks with how many layers (`<< 16`). Then
/// the chunks' light (`near::LightBlock`'s words), each in a stretch given back with its chunk. `free`: the free
/// stretches after the directory (where they begin, how many words), in order; `used`: words in chunks' light.
struct Light {
    buffer: wgpu::Buffer,
    free: Vec<(u64, u64)>,
    used: u64,
    /// Whether the buffer couldn't be grown when it was full (said once; chunks after are lit as plain daylight).
    full: bool,
}

/// A page: a buffer of faces, and its free stretches (where they begin, how many faces), in order.
struct Page {
    buffer: wgpu::Buffer,
    free: Vec<(u32, u32)>,
}

/// What the near eye says of itself, for the panel and a flight's report.
#[derive(Clone, Copy, Default, Debug)]
pub struct Counts {
    /// Chunks wanted inside the radius, kept on the card, and drawn this frame; and the frame's draws (a chunk's
    /// groups that show and lie together are one; its water's tops another).
    pub wanted: usize,
    pub kept: usize,
    pub drawn: usize,
    pub draws: usize,
    /// Faces kept on the card, and sent to it this frame (two triangles each).
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
    /// The near light (build 210): the chunks kept that have some, the memory their light takes, and the whole
    /// buffer's with its directory, MB.
    pub lit_chunks: usize,
    pub light_kept: f32,
    pub light_megabytes: f32,
}

pub struct NearEye {
    lane: Lane,
    /// The lantern (build 217): its thread; its box on the card (32 x 32 x 32 cells of 16 bits); what was last
    /// asked of the thread (the light's cell and its light; none, the lantern out); and what the box on the card
    /// was made for (none: no box).
    lantern_lane: crate::near::LanternLane,
    lantern: wgpu::Buffer,
    lantern_asked: Option<([i32; 3], [u8; 3])>,
    lantern_shown: Option<([i32; 3], [u8; 3])>,
    /// How long the last box's blocks took to read and its light to spread, on its thread (ms).
    pub lantern_ms: (f32, f32),
    /// The far lamps (build 222): their thread; what was last asked of it (the eye's column and the reach) and
    /// whether a list has come since; the list on the card (`near::MOST_LAMPS` of four words) and how many are in it;
    /// the dots' pass, what its group is made by, its numbers and its group (made again with the bands); and whether
    /// they're drawn at all (`INCUS_FAR_LAMPS=0`: not, as to build 221).
    lamps_lane: crate::near::LampsLane,
    /// The placed lights' clock, seconds, if it's set (the tests' pictures at chosen moments); none, the time since
    /// the game began.
    pub clock: Option<f32>,
    lamps_asked: Option<([i32; 2], i32)>,
    lamps_come: bool,
    lamps: wgpu::Buffer,
    pub lamps_count: u32,
    /// The list as it was put on the card (for the tests, which say where the far lamps are).
    #[cfg_attr(not(test), allow(dead_code))]
    pub lamps_listed: Vec<crate::near::Lamp>,
    dots: wgpu::RenderPipeline,
    dots_layout: wgpu::BindGroupLayout,
    dots_numbers: wgpu::Buffer,
    dots_group: Option<(wgpu::BindGroup, u64)>,
    /// Seeing the far lamps (build 223): its numbers, and its group (made by the far eye's layout, once). Whether
    /// each lamp is seen is written into the lamps' own words.
    sight_numbers: wgpu::Buffer,
    sight_group: Option<wgpu::BindGroup>,
    /// In turns (build 226): how many turns the list is seen in (`SIGHT_TURNS`; `INCUS_FAR_LAMP_TURNS` for
    /// another, 1 as to build 225); the frame's count, which gives its turn; whether the next pass recorded sees to
    /// the whole list (a new list is on the card; put out when that pass is recorded); and the turns and the
    /// threads of the pass the numbers were last written for.
    sight_turns: u32,
    sight_frame: u32,
    sight_whole: std::cell::Cell<bool>,
    sight_threads: u32,
    pub far_lamps: bool,
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
    /// What a band's group and its water's depths are made by; and stand-ins where a band has no nearer band's
    /// picture or no farther band's depth (nothing drawn: clear, and nothing behind).
    band_layout: wgpu::BindGroupLayout,
    depths_layout: wgpu::BindGroupLayout,
    blank: wgpu::TextureView,
    blank_depth: wgpu::TextureView,
    /// The bind group, and what it was made from (`FarEye::near_reads`' number).
    group: Option<(wgpu::BindGroup, u64)>,
    numbers: wgpu::Buffer,
    colours: wgpu::Buffer,
    /// The blocks' pictures on the card (a layer a picture), how they're read, and whether the faces wear them.
    blocks_view: wgpu::TextureView,
    crisp: wgpu::Sampler,
    textured: bool,
    light: Light,
    /// Whether a chunk's groups of faces that all look away from the eye are left out of the frame's draws (build
    /// 209; `INCUS_NEAR_TURNED=0`: not, and the card doesn't drop faces turned away either, as to build 208).
    pub turned: bool,
    /// The mask's pass (build 213): before a band's faces, its depth is filled where a nearer band's picture covers
    /// the pixel, so the card drops the faces there before their pixels' work.
    mask: wgpu::RenderPipeline,
    /// The x-fader's pass, what its group is made by, its numbers (each band's squares' side) and its group (the
    /// bands' pictures, the full-size one's first); the picture's format; the near eye's bands, the nearest first
    /// (the last is the full-size picture), with what they were made for (their squares' sizes and the screen's).
    join: wgpu::RenderPipeline,
    join_layout: wgpu::BindGroupLayout,
    join_numbers: wgpu::Buffer,
    join_group: Option<wgpu::BindGroup>,
    format: wgpu::TextureFormat,
    bands: Vec<Band>,
    bands_for: Option<(Vec<u32>, (u32, u32))>,
    /// The cover pass (build 201): its program, what its group is made by, its numbers (the far layers' sizes), and
    /// its group with what it was made from (the far eye's targets' number, and the near picture's size).
    cover_pass: wgpu::ComputePipeline,
    cover_layout: wgpu::BindGroupLayout,
    cover_sizes: wgpu::Buffer,
    cover_sampler: wgpu::Sampler,
    cover_group: Option<(wgpu::BindGroup, u64, (u32, u32))>,
    /// The frame's draws, each band's after another's (`Band::drawn`).
    draws: wgpu::Buffer,
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
        let lantern_lane = crate::near::LanternLane::start(folder.clone(), crate::scatter_stage::on())?;
        let lamps_lane = crate::near::LampsLane::start(folder.clone(), crate::scatter_stage::on())?;
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
                uniform(1, both),
                picture(2),
                picture(3),
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: fragment, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: fragment, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                uniform(6, both),
                wgpu::BindGroupLayoutEntry { binding: 7, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 8, visibility: fragment, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 9, visibility: fragment, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 10, visibility: fragment, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        // The lantern's box: 32 x 32 x 32 cells of 16 bits, nothing in it till its thread hands one over.
        let lantern = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the lantern's light"), size: (crate::near::CARRIED_BOX as u64).pow(3) * 2, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        // The near light: its directory all "no chunk", and the rest free.
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near light"), size: LIGHT_FIRST * 4, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        queue.write_buffer(&light_buffer, 0, &vec![0xFFu8; LIGHT_DIRECTORY as usize * 4]);
        let light = Light { buffer: light_buffer, free: vec![(LIGHT_DIRECTORY, LIGHT_FIRST - LIGHT_DIRECTORY)], used: 0, full: false };
        log::info!("The near eye's light: {:?}", crate::near::lit());
        // A band's own group (build 213): its numbers, and the pictures of the two bands nearer than it. And its
        // water's: its own depth and the depths of the two bands behind it.
        let plain = |binding: u32| wgpu::BindGroupLayoutEntry { binding, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let deep = |binding: u32| wgpu::BindGroupLayoutEntry { binding, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let band_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("a near band"), entries: &[uniform(0, both), plain(1), plain(2)] });
        let depths_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("a near band's depths, for its water"), entries: &[deep(0), deep(1), deep(2)] });
        let stand_in = |label: &'static str, format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor { label: Some(label), size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        let (blank, blank_depth) = (stand_in("no nearer band's picture", format), stand_in("no farther band's depth", wgpu::TextureFormat::Depth32Float));
        // (Each pipeline its own layout: the water's reads depths as pictures, in a third group; the faces' writes
        // its band's depth and so can't be given it.)
        let faces_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the near faces"), bind_group_layouts: &[Some(&layout), Some(&band_layout)], immediate_size: 0 });
        // The mask: a band's depth filled where a nearer band's picture covers the pixel (the nearest there is).
        let mask_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("a near band's mask"), source: wgpu::ShaderSource::Wgsl(include_str!("near_mask.wgsl").into()) });
        let mask_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("a near band's mask"), bind_group_layouts: &[Some(&band_layout)], immediate_size: 0 });
        let mask = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("a near band's mask"),
            layout: Some(&mask_layout),
            vertex: wgpu::VertexState { module: &mask_module, entry_point: Some("mask_whole"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &mask_module, entry_point: Some("mask_covered"), compilation_options: Default::default(), targets: &[] }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Always), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let water_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the near water"), bind_group_layouts: &[Some(&layout), Some(&band_layout), Some(&depths_layout)], immediate_size: 0 });
        let face_words = [wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32x3, offset: 0, shader_location: 0 }];
        let buffers = [Some(wgpu::VertexBufferLayout { array_stride: FACE, step_mode: wgpu::VertexStepMode::Instance, attributes: &face_words })];
        // (The near picture is kept multiplied by how much of it shows: a face is written whole, the nearest winning
        // a pixel; the water, and then the whole picture over the far one, laid over what's there.)
        let over = Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        let strip = wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() };
        // Faces turned away (build 209): the card drops them before filling them (every face is wound one way seen from
        // the side it looks out of: clockwise), and a chunk's groups of them aren't listed at all (`prepare`). Not the
        // water's tops: they're seen from under too. (`INCUS_NEAR_TURNED=0`: neither, as to build 208.)
        let turned = std::env::var("INCUS_NEAR_TURNED").as_deref() != Ok("0");
        let sides = wgpu::PrimitiveState { front_face: wgpu::FrontFace::Cw, cull_mode: turned.then_some(wgpu::Face::Back), ..strip };
        log::info!("The near eye's faces turned away: {}", if turned { "dropped by the card, their groups not listed" } else { "drawn like the rest (INCUS_NEAR_TURNED=0)" });
        let faces = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the near faces"),
            layout: Some(&faces_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("near_corner"), compilation_options: Default::default(), buffers: &buffers },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("near_face"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: sides,
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
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: fragment, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None }, uniform(1, fragment), plain(2), plain(3)],
        });
        let join_numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the x-fader's numbers"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
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
        // The far lamps (build 222): a dot a lamp, added to the picture after the x-fader. Its own group, the fourth
        // (`near.wgsl`'s `Twinkle`, the lamps with whether each is seen past the near reach in them, build 223; the
        // full-size band's depth), read where each dot's corners are placed.
        let vertex = wgpu::ShaderStages::VERTEX;
        let dots_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("the far lamps"),
            entries: &[
                uniform(0, vertex),
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: vertex, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: vertex, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
            ],
        });
        let dots_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the far lamps"), bind_group_layouts: &[Some(&layout), None, None, Some(&dots_layout)], immediate_size: 0 });
        let added = Some(wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add } });
        let dots = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the far lamps"),
            layout: Some(&dots_pipeline_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("lamp_dot"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("lamp_glow"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: added, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sight_numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("seeing the far lamps' numbers"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sight_turns = std::env::var("INCUS_FAR_LAMP_TURNS").ok().and_then(|turns| turns.parse::<u32>().ok()).filter(|turns| (1..=64).contains(turns)).unwrap_or(SIGHT_TURNS);
        log::info!("Seeing the far lamps: {}", if sight_turns == 1 { "every lamp every frame".to_string() } else { format!("in {sight_turns} turns, a share of the list a frame") });
        let dots_numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the far lamps' numbers"), size: 48, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let lamps = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the far lamps' list"), size: crate::near::MOST_LAMPS as u64 * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let far_lamps = std::env::var("INCUS_FAR_LAMPS").as_deref() != Ok("0");
        log::info!("The far lamps: {}", if far_lamps { "a dot a lamp from where it's about a pixel, twinkling" } else { "not drawn (INCUS_FAR_LAMPS=0)" });
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
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: compute, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: compute, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
            ],
        });
        let cover_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the near cover"), bind_group_layouts: &[Some(&cover_layout)], immediate_size: 0 });
        let cover_pass = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("near_cover"), layout: Some(&cover_pipeline_layout), module: &cover_module, entry_point: Some("near_cover"), compilation_options: Default::default(), cache: None });
        let cover_sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("the near cover's four at a time"), ..Default::default() });
        let cover_sizes = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near cover's numbers"), size: 80, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near eye's numbers"), size: 80, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let colour_bytes: Vec<u8> = colours.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
        let colours = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the blocks' colours from far"), size: colour_bytes.len().max(32) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&colours, 0, &colour_bytes);
        let draws = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near eye's draws"), size: MOST_DRAWN * 8 * 16, usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
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
        Ok(Self { lane, lantern_lane, lantern, lantern_asked: None, lantern_shown: None, lantern_ms: (0.0, 0.0), lamps_lane, clock: None, lamps_asked: None, lamps_come: false, lamps, lamps_count: 0, lamps_listed: Vec::new(), dots, dots_layout, dots_numbers, sight_numbers, sight_group: None, sight_turns, sight_frame: 0, sight_whole: std::cell::Cell::new(true), sight_threads: 0, dots_group: None, far_lamps, look: NearLook { radius, ..NearLook::DEFAULT }, chunks: HashMap::new(), pages: Vec::new(), asked: None, want: Vec::new(), first_to_come: 0, touched: HashSet::new(), need: HashMap::new(), remake: HashSet::new(), waiting: Default::default(), recent_ms: Default::default(), faces, water, layout, band_layout, depths_layout, blank, blank_depth, group: None, numbers, colours, blocks_view, crisp, textured, light, turned, mask, join, join_layout, join_numbers, join_group: None, format, bands: Vec::new(), bands_for: None, cover_pass, cover_layout, cover_sizes, cover_sampler, cover_group: None, draws, counts: Counts { threads, ..Default::default() } })
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

    /// Room for `words` of a chunk's light: where in the light's buffer (a free stretch that holds them, the first
    /// found). None left: the buffer made twice the size (and what it held copied over), unless the card won't have
    /// one so big: then none, said once.
    fn light_room(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, words: u64) -> Option<u64> {
        loop {
            let free = &mut self.light.free;
            if let Some(at) = free.iter().position(|(_, room)| *room >= words) {
                let (first, room) = free[at];
                if room == words {
                    free.remove(at);
                } else {
                    free[at] = (first + words, room - words);
                }
                self.light.used += words;
                return Some(first);
            }
            let old = self.light.buffer.size() / 4;
            let size = (old * 2).max(old + words);
            if size * 4 > device.limits().max_storage_buffer_binding_size as u64 {
                if !self.light.full {
                    log::error!("The near light is full at {} MB and the card won't have a buffer twice the size: chunks from here on are lit as plain daylight", old * 4 >> 20);
                    self.light.full = true;
                }
                return None;
            }
            let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the near light"), size: size * 4, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
            // (What's been written to the old buffer this frame goes with this handing over, before the copy.)
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the near light grown") });
            encoder.copy_buffer_to_buffer(&self.light.buffer, 0, &buffer, 0, old * 4);
            queue.submit([encoder.finish()]);
            match self.light.free.last_mut() {
                Some((first, room)) if *first + *room == old => *room += size - old,
                _ => self.light.free.push((old, size - old)),
            }
            self.light.buffer = buffer;
            self.group = None;
            log::info!("The near light: its buffer grown to {} MB", size * 4 >> 20);
        }
    }

    /// Where a chunk's entry is in the light's directory, bytes.
    fn light_entry(chunk: [i32; 2]) -> u64 {
        (chunk[0].rem_euclid(LIGHT_SIDE) + chunk[1].rem_euclid(LIGHT_SIDE) * LIGHT_SIDE) as u64 * 8
    }

    /// A chunk's light given back, and its entry in the directory made "no chunk".
    fn give_light_back(&mut self, queue: &wgpu::Queue, chunk: [i32; 2], kept: &Kept) {
        let Some((first, words)) = kept.light else { return };
        queue.write_buffer(&self.light.buffer, Self::light_entry(chunk), &[0xFFu8; 8]);
        self.light.used -= words;
        let free = &mut self.light.free;
        let at = free.partition_point(|(from, _)| *from < first);
        free.insert(at, (first, words));
        if at + 1 < free.len() && free[at].0 + free[at].1 == free[at + 1].0 {
            free[at].1 += free[at + 1].1;
            free.remove(at + 1);
        }
        if at > 0 && free[at - 1].0 + free[at - 1].1 == free[at].0 {
            free[at - 1].1 += free[at].1;
            free.remove(at);
        }
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
                    self.give_light_back(queue, chunk, &kept);
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
                self.give_light_back(queue, made.chunk, &old);
            }
            // Its light, where some of it isn't plain daylight: put in the light's buffer, and its entry written.
            let light = made.light.as_ref().and_then(|block| {
                let words = block.words.len() as u64;
                let first = self.light_room(device, queue, words)?;
                let bytes: Vec<u8> = block.words.iter().flat_map(|word| word.to_le_bytes()).collect();
                queue.write_buffer(&self.light.buffer, first * 4, &bytes);
                let entry: Vec<u8> = [first as u32, block.low | block.layers << 16].iter().flat_map(|word| word.to_le_bytes()).collect();
                queue.write_buffer(&self.light.buffer, Self::light_entry(made.chunk), &entry);
                Some((first, words))
            });
            let count = made.faces.len() as u32;
            let (page, first) = if count > 0 { self.room(device, count) } else { (0, 0) };
            if count > 0 {
                let bytes: Vec<u8> = made.faces.iter().flatten().flat_map(|word| word.to_le_bytes()).collect();
                queue.write_buffer(&self.pages[page].buffer, first as u64 * FACE, &bytes);
            }
            self.chunks.insert(made.chunk, Kept { page, first, count, light, groups: made.groups, water: made.water, low: made.low, high: made.high });
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
            draws: self.counts.draws,
            faces_drawn: self.counts.faces_drawn,
            radius,
            whole: self.first_to_come >= self.want.len() && self.remake.is_empty(),
            lit_chunks: self.chunks.values().filter(|kept| kept.light.is_some()).count(),
            light_kept: self.light.used as f32 * 4.0 / 1048576.0,
            light_megabytes: self.light.buffer.size() as f32 / 1048576.0,
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
            .flat_map(crate::near::touched_by_light)
            .collect();
        if now.is_empty() && self.touched.is_empty() {
            return;
        }
        let told = self.lane.changed(changes.clone());
        self.lantern_lane.changed(changes.clone());
        self.lamps_lane.changed(changes.clone());
        self.lamps_asked = None;
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

    /// Whether the lantern's box on the card is the one last asked for (or there's none and none is wanted): for
    /// the tests, which wait for its thread.
    pub fn lantern_settled(&self) -> bool {
        self.lantern_asked == self.lantern_shown
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
                    wgpu::BindGroupEntry { binding: 9, resource: self.light.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 10, resource: self.lantern.as_entire_binding() },
                ],
            });
            self.group = Some((group, made_from));
        }
        // The bands by the detail curve: their squares' sizes, and where each begins and ends this frame (from the
        // slider's a, the picture's height and the field of view).
        let per_block = screen.1.max(1) as f32 / (2.0 * camera.tan_half());
        let spans = bands_of(self.look.pixel, self.look.detail, per_block, self.look.stipple);
        let pixels: Vec<u32> = spans.iter().map(|span| span.0).collect();
        if self.bands_for.as_ref().is_none_or(|(made, size)| *made != pixels || *size != screen) {
            // Each band's picture and depth, a half or a quarter of the screen each way where its squares are 2 or 4
            // (rounded up, so the picture covers the screen; its pixels are squares of screen pixels counted from
            // the screen's top left corner).
            let made = |label: &'static str, format: wgpu::TextureFormat, size: (u32, u32)| {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                texture.create_view(&wgpu::TextureViewDescriptor::default())
            };
            let pictures: Vec<((u32, u32), wgpu::TextureView, wgpu::TextureView)> = pixels
                .iter()
                .map(|pixel| {
                    let size = (screen.0.max(1).div_ceil(*pixel), screen.1.max(1).div_ceil(*pixel));
                    (size, made("a near band's picture", self.format, size), made("a near band's depth", wgpu::TextureFormat::Depth32Float, size))
                })
                .collect();
            let mut bands = Vec::with_capacity(pixels.len());
            for (k, pixel) in pixels.iter().enumerate() {
                let numbers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("a near band's numbers"), size: 48, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
                // (The bands nearer than it: the one just nearer, then the one before that. The bands behind it:
                // the next, then the one after.)
                let nearer = |back: usize| k.checked_sub(back).map_or(&self.blank, |at| &pictures[at].1);
                let behind = |on: usize| pictures.get(k + on).map_or(&self.blank_depth, |picture| &picture.2);
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("a near band"),
                    layout: &self.band_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: numbers.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(nearer(1)) },
                        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(nearer(2)) },
                    ],
                });
                let depths = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("a near band's depths, for its water"),
                    layout: &self.depths_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&pictures[k].2) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(behind(1)) },
                        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(behind(2)) },
                    ],
                });
                bands.push(Band { pixel: *pixel, size: pictures[k].0, colour: pictures[k].1.clone(), depth: pictures[k].2.clone(), numbers, group, depths, drawn: Vec::new(), drawn_water: Vec::new() });
            }
            // The x-fader's group: the full-size picture, then the band just nearer than it and the one before that
            // (clear stand-ins where there are none), with each one's squares' side.
            let last = pictures.len() - 1;
            let before = |back: usize| last.checked_sub(back).map_or((&self.blank, 0), |at| (&pictures[at].1, pixels[at]));
            queue.write_buffer(&self.join_numbers, 0, &[pixels[last], before(1).1, before(2).1, 0].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
            self.join_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the x-fader"),
                layout: &self.join_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&pictures[last].1) },
                    wgpu::BindGroupEntry { binding: 1, resource: self.join_numbers.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(before(1).0) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(before(2).0) },
                ],
            }));
            // (The far lamps' group reads the full-size band's depth: it's made again.)
            self.dots_group = None;
            self.bands = bands;
            self.bands_for = Some((pixels.clone(), screen));
            // (The cover reads the full-size picture: its group is made again.)
            self.cover_group = None;
        }
        // Each band's numbers (`near.wgsl`'s `Band`): how much of its picture the screen takes, across and down (all
        // of it but for a rounding up), its pixel's side, and how much a face's hair is grown by (as much as its
        // pixel is bigger, and as the field of view is wider than the 100 degrees the hair was sized at, so it
        // stays the same share of a pixel); where it begins and ends, and how many of its pixels a side a pixel of
        // each of the two bands nearer than it is; and how many of each of the two bands behind it's a pixel of
        // its own is, with the share of its reach its stipple takes (none for the farthest band, which the x-fader
        // hands over to the far picture), and how many sizes finer its blocks' pictures are read than the card
        // would choose for its pixels (the sharpness: up to all the sizes its pixel is bigger than the screen's).
        let wide = camera.tan_half() / (crate::camera::FIELD_OF_VIEW.to_radians() / 2.0).tan();
        for (k, band) in self.bands.iter().enumerate() {
            let pixel = band.pixel as f32;
            let ratio = |other: Option<&u32>, over: bool| other.map_or(0.0, |other| if over { *other as f32 / pixel } else { pixel / *other as f32 });
            let numbers = [
                [screen.0.max(1) as f32 / (band.size.0 * band.pixel) as f32, screen.1.max(1) as f32 / (band.size.1 * band.pixel) as f32, pixel, pixel * wide],
                [spans[k].1, spans[k].2.min(1e30), ratio(k.checked_sub(1).and_then(|at| pixels.get(at)), true), ratio(k.checked_sub(2).and_then(|at| pixels.get(at)), true)],
                [ratio(pixels.get(k + 1), false), ratio(pixels.get(k + 2), false), if k + 1 < pixels.len() { self.look.stipple.clamp(0.0, MOST_STIPPLE) } else { 0.0 }, -self.look.sharp.clamp(0.0, 1.0) * pixel.log2()],
            ];
            queue.write_buffer(&band.numbers, 0, &numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        }
        // The lantern (build 217): asked of its thread when the eye's block, its light or its switch changes; its
        // box put on the card as it comes (the frame never waits: the light follows a frame or two behind).
        let carried_at = [camera.position.x, camera.position.y, camera.position.z].map(|v| v.floor() as i32);
        let wanted = self.look.lantern.then(|| (carried_at, self.look.lantern_light()));
        if wanted != self.lantern_asked {
            self.lantern_asked = wanted;
            match wanted {
                Some((at, light)) => self.lantern_lane.ask(at, light),
                None => {
                    self.lantern_lane.rest();
                    self.lantern_shown = None;
                }
            }
        }
        while let Some(carried) = self.lantern_lane.take() {
            match carried {
                Ok(carried) if self.look.lantern => {
                    queue.write_buffer(&self.lantern, 0, &carried.cells.iter().flat_map(|cell| cell.to_le_bytes()).collect::<Vec<u8>>());
                    self.lantern_shown = Some((carried.at, carried.light));
                    self.lantern_ms = (carried.read_ms, carried.spread_ms);
                }
                Ok(_) => {}
                Err(error) => log::error!("The lantern's light couldn't be spread: {error}"),
            }
        }
        // The far lamps (build 222): the list asked of its thread when the eye has moved `LAMPS_STEP` blocks (out to
        // `LAMPS_REACH`, build 223), put on the card as it comes; their group (their numbers, the list, the full-size
        // band's depth), made again with the bands; seeing the far
        // lamps' group (once: by the far eye's layout); and the numbers of both.
        if let Some(whole) = self.bands.last()
            && self.dots_group.is_none()
        {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the far lamps"),
                layout: &self.dots_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.dots_numbers.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.lamps.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&whole.depth) },
                ],
            });
            self.dots_group = Some((group, made_from));
        }
        if self.sight_group.is_none() {
            self.sight_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("seeing the far lamps"),
                layout: &far.sight_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.sight_numbers.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.lamps.as_entire_binding() },
                ],
            }));
        }
        let column = [camera.position.x.floor() as i32, camera.position.y.floor() as i32];
        let lamps_reach = crate::near::LAMPS_REACH;
        if self.lamps_asked.is_none_or(|(at, reach)| reach != lamps_reach || (at[0] - column[0]).abs() >= crate::near::LAMPS_STEP || (at[1] - column[1]).abs() >= crate::near::LAMPS_STEP) {
            self.lamps_asked = Some((column, lamps_reach));
            self.lamps_lane.ask(column, lamps_reach);
        }
        for made in self.lamps_lane.take() {
            match made {
                Ok(list) => {
                    let words: Vec<u8> = list.iter().flat_map(|lamp| [lamp.cell[0], lamp.cell[1], lamp.cell[2], lamp.light[0] as i32 | (lamp.light[1] as i32) << 4 | (lamp.light[2] as i32) << 8 | (lamp.voice as i32) << 12]).flat_map(|v| v.to_le_bytes()).collect();
                    if !words.is_empty() {
                        queue.write_buffer(&self.lamps, 0, &words);
                    }
                    self.lamps_count = list.len() as u32;
                    self.lamps_listed = list;
                    self.lamps_come = true;
                    self.sight_whole.set(true);
                }
                Err(error) => log::error!("The far lamps couldn't be listed: {error}"),
            }
        }
        // Seeing them, in turns (build 226): this frame's turn of the list, or the whole of a list new on the card
        // (after the list is taken, so the numbers are of the list the pass reads).
        self.sight_frame = self.sight_frame.wrapping_add(1);
        let turns = if self.sight_whole.get() { 1 } else { self.sight_turns };
        self.sight_threads = self.lamps_count.div_ceil(turns);
        queue.write_buffer(&self.sight_numbers, 0, &[self.lamps_count as f32, far.near_from, DOT_FAR_SLACK, 0.0, (self.sight_frame % turns) as f32, turns as f32, 0.0, 0.0].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let whole_pixel = self.bands.last().map_or(1, |band| band.pixel) as f32;
        let dot_numbers = [[screen.0.max(1) as f32, screen.1.max(1) as f32, whole_pixel, per_block], [DOT_FROM, DOT_WHOLE, DOT_BAR, DOT_BAR_RISES], [DOT_BAR_OVER, DOT_SLACK, 0.0, 0.0]];
        queue.write_buffer(&self.dots_numbers, 0, &dot_numbers.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        // The numbers: the eye's block and how far into it the eye is (and the x-fader's curve); the reach, the
        // x-fader's band and the depths; and the placed lights' flicker and step, with the time (seconds, turned over each
        // hour so it stays fine); and the lantern's box's least corner, with whether there's a box at all.
        let eye = [camera.position.x, camera.position.y, camera.position.z];
        let cell = eye.map(|v| v.floor());
        let mut bytes: Vec<u8> = Vec::with_capacity(80);
        bytes.extend(cell.iter().map(|v| *v as i32).chain([self.textured as i32 | (crate::near::light_on() as i32) << 1 | ((crate::near::lit() == crate::near::Lit::Kept) as i32) << 2]).flat_map(|v| v.to_le_bytes()));
        bytes.extend((0..3).map(|k| eye[k] - cell[k]).chain([self.look.fader_curve.clamp(1.0, MOST_FADER_CURVE)]).flat_map(|v| v.to_le_bytes()));
        bytes.extend([self.counts.reach, band_of(self.look.fader_width, self.counts.reach), DEPTHS[0], DEPTHS[1]].iter().flat_map(|v| v.to_le_bytes()));
        static BEGAN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let seconds = self.clock.unwrap_or_else(|| BEGAN.get_or_init(std::time::Instant::now).elapsed().as_secs_f32() % 3600.0);
        bytes.extend([self.look.flicker.clamp(0.0, 1.0), self.look.flicker_speed.max(0.0), seconds, self.look.step.clamp(STEPS.0, STEPS.1)].iter().flat_map(|v| v.to_le_bytes()));
        let corner = self.lantern_shown.map(|(at, _)| at.map(|v| v - crate::near::CARRIED_BOX / 2));
        bytes.extend(corner.unwrap_or([0; 3]).iter().copied().chain([corner.is_some() as i32]).flat_map(|v| v.to_le_bytes()));
        queue.write_buffer(&self.numbers, 0, &bytes);
        // The chunks in view, for each band it lies in, a page after another: each a draw of its faces.
        let view = View::of(camera, screen);
        let reach = self.counts.reach;
        let count = self.bands.len();
        let mut by_page: Vec<Vec<Vec<[u32; 4]>>> = vec![vec![Vec::new(); self.pages.len()]; count];
        let mut water_by_page: Vec<Vec<Vec<[u32; 4]>>> = vec![vec![Vec::new(); self.pages.len()]; count];
        let (mut drawn, mut faces_drawn) = (0usize, 0u64);
        let mut runs: Vec<[u32; 4]> = Vec::with_capacity(4);
        for (chunk, kept) in &self.chunks {
            if kept.count == 0 || nearest_part(*chunk, [eye[0], eye[1]]) > reach || !view.sees(*chunk, kept.low, kept.high) {
                continue;
            }
            if drawn as u64 >= MOST_DRAWN {
                break;
            }
            // A face is four corners (a strip), once for each of the chunk's faces of the kind. Of the blocks' sides,
            // only the groups a face of which could look at the eye (build 209), those that lie together as one draw.
            runs.clear();
            let mut sent = 0u64;
            let sides = kept.count - kept.water;
            if sides > 0 && self.turned {
                let shown = crate::near::groups_shown(*chunk, kept.low, kept.high, eye);
                let mut first = kept.first;
                let mut run: Option<(u32, u32)> = None;
                for (k, &count) in kept.groups.iter().enumerate() {
                    if count == 0 {
                        continue;
                    }
                    if shown[k] {
                        run = match run {
                            Some((from, n)) if from + n == first => Some((from, n + count)),
                            Some((from, n)) => {
                                runs.push([4, n, 0, from]);
                                Some((first, count))
                            }
                            None => Some((first, count)),
                        };
                        sent += count as u64;
                    }
                    first += count;
                }
                if let Some((from, n)) = run {
                    runs.push([4, n, 0, from]);
                }
            } else if sides > 0 {
                runs.push([4, sides, 0, kept.first]);
                sent += sides as u64;
            }
            // The bands it lies in: those whose stretch of distance from the eye its box reaches into (its nearest
            // and its farthest point, true distance).
            let (lo, hi) = ([(chunk[0] * CHUNK) as f32, (chunk[1] * CHUNK) as f32, kept.low as f32], [((chunk[0] + 1) * CHUNK) as f32, ((chunk[1] + 1) * CHUNK) as f32, kept.high as f32]);
            let nearest = (0..3).map(|k| (lo[k] - eye[k]).max(eye[k] - hi[k]).max(0.0).powi(2)).sum::<f32>().sqrt();
            let farthest = (0..3).map(|k| (eye[k] - lo[k]).abs().max((eye[k] - hi[k]).abs()).powi(2)).sum::<f32>().sqrt();
            for (k, span) in spans.iter().enumerate() {
                if nearest > span.2 || farthest < span.1 {
                    continue;
                }
                by_page[k][kept.page].extend_from_slice(&runs);
                faces_drawn += sent;
                if kept.water > 0 {
                    water_by_page[k][kept.page].push([4, kept.water, 0, kept.first + sides]);
                    faces_drawn += kept.water as u64;
                }
            }
            drawn += 1;
        }
        let room = self.draws.size() as usize;
        let mut words: Vec<u8> = Vec::with_capacity(drawn * 32);
        let mut listed = 0usize;
        for (k, band) in self.bands.iter_mut().enumerate() {
            band.drawn.clear();
            band.drawn_water.clear();
            for (lists, said) in [(&by_page[k], &mut band.drawn), (&water_by_page[k], &mut band.drawn_water)] {
                for (page, draws) in lists.iter().enumerate() {
                    // (More draws than the buffer holds: the rest left out, said in the count.)
                    if !draws.is_empty() && words.len() + draws.len() * 16 <= room {
                        said.push((page, words.len() as u64, draws.len() as u32));
                        words.extend(draws.iter().flatten().flat_map(|word| word.to_le_bytes()));
                        listed += draws.len();
                    }
                }
            }
        }
        if !words.is_empty() {
            queue.write_buffer(&self.draws, 0, &words);
        }
        self.counts.drawn = drawn;
        self.counts.draws = listed;
        self.counts.faces_drawn = faces_drawn;
        true
    }

    /// A pass's timestamps shared out over several passes recorded for it: the first takes its beginning, the last
    /// its end.
    fn stamps<'a>(timestamps: &Option<wgpu::RenderPassTimestampWrites<'a>>, first: bool, last: bool) -> Option<wgpu::RenderPassTimestampWrites<'a>> {
        let whole = timestamps.as_ref()?;
        let (beginning, end) = (if first { whole.beginning_of_pass_write_index } else { None }, if last { whole.end_of_pass_write_index } else { None });
        (beginning.is_some() || end.is_some()).then(|| wgpu::RenderPassTimestampWrites { query_set: whole.query_set, beginning_of_pass_write_index: beginning, end_of_pass_write_index: end })
    }

    /// The faces drawn into each band's own picture, with depth, the nearest band first: every face whole, the nearest
    /// winning a pixel. Before a farther band's faces its depth is filled, as the nearest there is, wherever a nearer
    /// band's picture covers the pixel (the mask), so the card drops its faces there before their pixels' work.
    pub fn draw_faces(&self, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let Some((group, _)) = &self.group else { return };
        for (k, band) in self.bands.iter().enumerate() {
            if k > 0 {
                let depth = wgpu::RenderPassDepthStencilAttachment { view: &band.depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Store }), stencil_ops: None };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("a near band's mask"), color_attachments: &[], depth_stencil_attachment: Some(depth), timestamp_writes: None, ..Default::default() });
                pass.set_pipeline(&self.mask);
                pass.set_bind_group(0, &band.group, &[]);
                pass.draw(0..3, 0..1);
            }
            let attachment = wgpu::RenderPassColorAttachment { view: &band.colour, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store } };
            let depth = wgpu::RenderPassDepthStencilAttachment { view: &band.depth, depth_ops: Some(wgpu::Operations { load: if k > 0 { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(0.0) }, store: wgpu::StoreOp::Store }), stencil_ops: None };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the near faces"), color_attachments: &[Some(attachment)], depth_stencil_attachment: Some(depth), timestamp_writes: Self::stamps(&timestamps, k == 0, k + 1 == self.bands.len()), ..Default::default() });
            pass.set_pipeline(&self.faces);
            pass.set_bind_group(0, group, &[]);
            pass.set_bind_group(1, &band.group, &[]);
            for (page, at, count) in &band.drawn {
                pass.set_vertex_buffer(0, self.pages[*page].buffer.slice(..));
                pass.multi_draw_indirect(&self.draws, *at, *count);
            }
        }
    }

    /// The water's top laid over the faces in each band's picture, by how much water the eye looks through (to the
    /// faces of its own band, or of the bands behind it).
    pub fn draw_water(&self, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let Some((group, _)) = &self.group else { return };
        for (k, band) in self.bands.iter().enumerate() {
            let attachment = wgpu::RenderPassColorAttachment { view: &band.colour, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the near water"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, timestamp_writes: Self::stamps(&timestamps, k == 0, k + 1 == self.bands.len()), ..Default::default() });
            pass.set_pipeline(&self.water);
            pass.set_bind_group(0, group, &[]);
            pass.set_bind_group(1, &band.group, &[]);
            pass.set_bind_group(2, &band.depths, &[]);
            for (page, at, count) in &band.drawn_water {
                pass.set_vertex_buffer(0, self.pages[*page].buffer.slice(..));
                pass.multi_draw_indirect(&self.draws, *at, *count);
            }
        }
    }

    /// The cover pass (build 201), recorded after the faces and the water: which of the far eye's pixels the near
    /// picture wholly covers, written into the far eye's own picture for it, so the march needn't march them. False,
    /// and nothing recorded, where the far eye takes no cover or there's no near picture.
    pub fn cover(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, far: &FarEye, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) -> bool {
        let (Some(whole), Some((words, sizes, side, made))) = (self.bands.last(), far.cover_writes()) else { return false };
        // (The band just nearer than the full-size picture, and the one before that: a pixel either covers is covered.)
        let last = self.bands.len() - 1;
        let before = |back: usize| last.checked_sub(back).map_or((&self.blank, 0), |at| (&self.bands[at].colour, self.bands[at].pixel));
        if self.cover_group.as_ref().is_none_or(|(_, from, size)| *from != made || *size != whole.size) {
            let numbers: Vec<u8> = [[sizes[0].0, sizes[0].1, side, 0], [sizes[1].0, sizes[1].1, 0, 0], [sizes[2].0, sizes[2].1, 0, 0], [whole.size.0, whole.size.1, whole.pixel, 0], [before(1).1, before(2).1, 0, 0]].iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
            queue.write_buffer(&self.cover_sizes, 0, &numbers);
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the near cover"),
                layout: &self.cover_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&whole.colour) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(words) },
                    wgpu::BindGroupEntry { binding: 2, resource: self.cover_sizes.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.cover_sampler) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(before(1).0) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(before(2).0) },
                ],
            });
            self.cover_group = Some((group, made, whole.size));
        }
        let Some((group, _, _)) = &self.cover_group else { return false };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the near cover"), timestamp_writes: timestamps });
        pass.set_pipeline(&self.cover_pass);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(sizes[0].0.div_ceil(8), sizes[0].1.div_ceil(8), 1);
        true
    }

    /// Whether the far lamps' list on the card is the one last asked for (a list has come since the asking): for the
    /// tests, which wait for its thread.
    pub fn lamps_settled(&self) -> bool {
        self.lamps_come && self.lamps_asked.is_some()
    }

    /// Seeing the far lamps (build 223): whether each lamp past where the far rays begin is seen, by a ray marched to
    /// it through the far cells (the far eye's pass), for the dots after the x-fader; this frame's turn of the list
    /// (build 226), as `prepare` wrote its numbers.
    pub fn see_lamps(&self, encoder: &mut wgpu::CommandEncoder, far: &FarEye, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) {
        let Some(group) = &self.sight_group else { return };
        if far.see_lamps(encoder, group, if self.far_lamps { self.sight_threads } else { 0 }, timestamps) {
            self.sight_whole.set(false);
        }
    }

    /// The far lamps (build 222): a dot a lamp added to the picture (`view`), after the x-fader.
    pub fn draw_lamps(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let (Some((group, _)), Some((dots, _))) = (&self.group, &self.dots_group) else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the far lamps"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, timestamp_writes: timestamps, ..Default::default() });
        if !self.far_lamps || self.lamps_count == 0 {
            return;
        }
        pass.set_pipeline(&self.dots);
        pass.set_bind_group(0, group, &[]);
        pass.set_bind_group(3, dots, &[]);
        pass.draw(0..6, 0..self.lamps_count);
    }

    /// The x-fader: the near eye's pictures laid over the far picture (`view`): the full-size one, then each nearer
    /// band's over it in its squares, the nearest last.
    pub fn join(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, timestamps: Option<wgpu::RenderPassTimestampWrites<'_>>) {
        let Some(joined) = &self.join_group else { return };
        let attachment = wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the x-fader"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, timestamp_writes: timestamps, ..Default::default() });
        pass.set_pipeline(&self.join);
        pass.set_bind_group(0, joined, &[]);
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
        let tan_up = camera.tan_half();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The lantern's colour as levels (build 218): the strongest at its brightness, the others as many steps under
    /// it as their shares come to at the step, so the three fall together.
    #[test]
    fn the_lantern_s_colour_keeps_its_shares_as_it_fades() {
        let look = NearLook::DEFAULT;
        assert_eq!(look.lantern_light(), [15, 13, 10], "warm orange at 15, a quarter a step: 0.6 is two steps under, 0.25 five");
        assert_eq!(NearLook { lantern_colour: [0.0, 1.0, 0.0], lantern_level: 9.0, ..look }.lantern_light(), [0, 9, 0], "a colour with none of it gives none");
        assert_eq!(NearLook { lantern_colour: [1.0, 1.0, 1.0], ..look }.lantern_light(), [15, 15, 15]);
    }
}

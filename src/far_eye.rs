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
const CAMERA_ROWS: usize = 25;
/// The cut (the long rays cut into lengths): a ray is long at this many steps or more, and a frame's list of long
/// rays holds this many (the plain has about 850 of 128 steps or more, the mountain top about 1,000, by the count
/// off the screen at build 176).
const LONG_STEPS: u32 = 128;
/// (64 x 64, the join's threads: `far_all.wgsl`'s `join_pass`. The most long rays at once over build 181's flight:
/// 2,419, on the run.)
const LONG_LIST: u32 = 4096;
/// The cut's store, words (`far_all.wgsl`'s `cut`, which has the same layout): the two lists of long rays, three
/// words a place; how each listed ray stood at each of the sixteen places it may be cut at (17 numbers a place in
/// each list); the two lists of lengths (`PIECES` places each: the strip's threads); a stack of `STACK` words for
/// each place in a list of rays (its ray's first length), then for each place in a list of lengths.
const PIECES: u32 = 16384;
/// Where the first place a ray may be cut at is, blocks out, unless `INCUS_FAR_CUT` says (as build 186 flew).
const FIRST_CUT: f32 = 384.0;
const STACK: u32 = 13;
const BEFORES_AT: u32 = 2 * 3 * LONG_LIST;
const PIECES_AT: u32 = BEFORES_AT + 2 * 17 * LONG_LIST;
const OWN_STACKS_AT: u32 = PIECES_AT + 2 * PIECES;
const PIECE_STACKS_AT: u32 = OWN_STACKS_AT + LONG_LIST * STACK;
const CUT_WORDS: u32 = PIECE_STACKS_AT + PIECES * STACK;
/// How many numbers the count of a layer's rays holds (`far_tally.wgsl`).
pub const TALLY: usize = 12;
/// The most of a frame the ring's sending takes (ms): its maker works off the frame, the frame only hands its bricks on.
const RING_MS: f32 = 0.3;

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
    /// Where each ray's steps begin within a step (the page's "Ray start"; David, 2026-11-14): the pattern (0
    /// scattered by chance, as it was; 1 woven, an even pattern; 2 blue noise; 3 woven with blue noise in it), the
    /// weave's side as a power of two, how much blue noise goes into the weave, and how far the starts spread (of a
    /// step: at 0 every ray begins alike, and the rings the scatter was for come back).
    pub start_pattern: u32,
    pub weave_bits: u32,
    pub start_blue: f32,
    pub start_spread: f32,
    /// Fog: on or off, how far off land at sea level is nineteen parts in twenty fog, how fast it thins going up (its
    /// thickness halves every so many blocks), and how much warmer it is towards the sun.
    pub fog: bool,
    pub fog_reach: f32,
    pub fog_height: f32,
    pub fog_warmth: f32,
    /// Cover by faces (the Render page's tick box; David, 2026-11-16: "do i have an a/b switch?"): a far cell stops
    /// a ray by the face it shows it as well as by its bulk. It picks which of the march's two programs marches once
    /// the world is loaded (`FarEye::one_loaded` and `one_loaded_other`): nothing is switched inside either.
    pub faces: bool,
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
        start_pattern: 0,
        weave_bits: 2,
        start_blue: 0.25,
        start_spread: 1.0,
        // Off by default [David, 2026-11-13: "off by default would be better. it looks nice but is most likely reserved
        // for final touches"; "we are not using fog or haze or any sort of cover as a performance tool. it is just for
        // looks"]: the far eye is judged and measured without it.
        fog: false,
        fog_reach: FOG_REACH,
        fog_height: FOG_HEIGHT,
        fog_warmth: FOG_WARMTH,
        faces: true,
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
    /// The count of threads kept waiting, by the one dispatch: its numbers, and its bind group for each of the two
    /// copies of the distances (this frame's, with the frame before's beside it).
    waiting_numbers: wgpu::Buffer,
    waiting_groups: Option<[wgpu::BindGroup; 2]>,
    layers: Vec<Layer>,
    up_numbers: wgpu::Buffer,
    up_group: [wgpu::BindGroup; 2],
    /// The one-dispatch form's: every layer's numbers in one buffer, and its bind group for each of the two copies.
    one: Option<(wgpu::Buffer, [wgpu::BindGroup; 2])>,
    /// The start pass's two groups by the one dispatch (each writes the tiles' starts into the copy of the distances
    /// the march reads that frame); the long rays' two counts; the cut's store (the two lists, then the listed rays'
    /// lengths' stacks: `far_all.wgsl`); and how many threads across the dispatch's strip for the lengths is.
    one_starts: Option<[wgpu::BindGroup; 2]>,
    long_rays: wgpu::Buffer,
    /// (Read here only by the test's looks at the list and the lengths; the card reads it through the bind groups.)
    #[cfg_attr(not(test), allow(dead_code))]
    cut: wgpu::Buffer,
    strip: u32,
    /// The near eye's cover: a pixel a pixel of each layer, the layers side by side as their distances are (written
    /// by the near eye's cover pass, read by the march: `far_all.wgsl`'s `cover`).
    cover: wgpu::TextureView,
    /// The audit's own bind group (`FarEye::audit`): the far eye's, with scratch in place of everything it writes.
    audit: Option<wgpu::BindGroup>,
}

pub struct FarEye {
    /// Where the far rays begin while the near eye draws what's nearer (blocks from the eye; 0: at the eye). Set by
    /// the near eye each frame (`near_eye.rs`: a little inside where its faces fade out).
    pub near_from: f32,
    /// How many times the targets have been made (the near eye's bind group is made again when they are).
    targets_made: u64,
    /// Whether the march skips what the near eye's blocks cover (build 201; `INCUS_FAR_COVER=0`: compiled out).
    cover_on: bool,
    pub store: Store,
    lane: Option<Lane>,
    /// What's placed through the doorway (build 207): the regions being summed again with it (its answer on the way,
    /// the regions it sums, those the changes touch), what's come back and waits for its moment, the regions on the
    /// card with something placed in them, whether the light is to be made again, and when it was last told.
    placed_job: Option<(mpsc::Receiver<Result<Vec<([i32; 2], Vec<Vec<crate::far::CellColumn>>)>, String>>, std::collections::HashSet<[i32; 2]>, std::collections::HashSet<[i32; 2]>)>,
    placed_ready: Option<(Vec<([i32; 2], Vec<Vec<crate::far::CellColumn>>)>, std::collections::HashSet<[i32; 2]>)>,
    placed_regions: std::collections::HashSet<[i32; 2]>,
    relight: bool,
    placed_asked: Option<Instant>,
    /// The rings round the eye (`far_ring.rs`), the 16s' first, then the 8s' (as many as `INCUS_FAR_RING` says: 0
    /// none), and where the eye last was.
    rings: Vec<crate::far_ring::Ring>,
    eye: [f32; 2],
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
    /// The blue noise (`blue_noise.rs`) on the card, and its numbers till they're sent (at the first putting).
    blue: (wgpu::Texture, wgpu::TextureView),
    blue_bytes: Option<Vec<u8>>,
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
    /// The march once the textures are in (`far.wgsl`'s `LOADED`, true): the one dispatch's second program, without
    /// the code's blend or the views, which the far eye marches by when `loaded` says so; `one`'s is then the whole
    /// march (`march_loading`). `INCUS_FAR_LOADED=0` keeps the whole march on (for measuring one against the other).
    /// And whether the log has said the second has taken over.
    one_loaded: Option<wgpu::ComputePipeline>,
    /// The march once the textures are in with cover by faces the other way from the launch's (`faces_asked`), for
    /// the Render page's switch: the same text but for `far.wgsl`'s `FACES`, so each of the two is the program the
    /// launch's choice alone would compile. None where there's no one dispatch, or an audit's version marches in
    /// the loaded march's place.
    one_loaded_other: Option<wgpu::ComputePipeline>,
    /// Whether the launch asked for cover by faces (`INCUS_FAR_FACES`, on unless 0): what every program but
    /// `one_loaded_other` is compiled with.
    faces_asked: bool,
    /// The start pass by the one dispatch: its program from the one dispatch's text, which writes the tiles' starts
    /// into the frame before's distances (`far_all.wgsl`'s `put_start`).
    one_start: Option<wgpu::ComputePipeline>,
    /// The cut's join (`far_all.wgsl`'s `join_pass`), from the whole march's text and from the march's once the
    /// textures are in; and where a cut ray's first cut is, blocks out (0: no ray is cut; `INCUS_FAR_CUT`).
    one_join: Option<[wgpu::ComputePipeline; 2]>,
    cut_at: f32,
    /// How many steps make a ray long (`LONG_STEPS`, or `INCUS_FAR_LONG`'s for measuring).
    long: u32,
    loaded_off: bool,
    loaded_said: std::sync::atomic::AtomicBool,
    /// The march's audit (`INCUS_FAR_AUDIT`, for measuring): cut-down versions of the march (`audit_variants`), each
    /// compiled beside it under its own name and marched over the base layer into scratch after the far eye, so one
    /// Nsight trace names each one's registers, how many groups of rays fit and its time. Empty unless asked: the
    /// game never builds them [David, 2026-11-14: "start with trying to break it into smaller passes to fit more"; the
    /// march keeps 168 registers and 12 groups fit of 48].
    audit: Vec<(&'static str, wgpu::ComputePipeline)>,
    parity: usize,
    fresh: bool,
    /// A ray's step through thin haze, in cells (the lean step, 1d; `INCUS_FAR_HAZE_STEP`): a whole cell (20% off the
    /// sea, 13% off a player's run; David, looking at build 151: "i don't see anything bad about it. Seems like a good
    /// thing"), or old's half (0.5).
    haze_step: f32,
    most_steps: f32,
    /// The height table's true numbers (`bricks::tops_whole`): worked out on a thread of its own once every region is in
    /// and sent to the card whole (till then it holds each band's rounded top, and the skip is as it was); how long that
    /// took, once it's in.
    tops_making: Option<mpsc::Receiver<f32>>,
    tops_made: Option<f32>,
    tops_off: bool,
    /// For measuring a ray's two lengths apart (the deep dive, 2026-11-14): the farthest a ray goes
    /// (`INCUS_TEST_REACH`, blocks: the near length alone) and the nearest it begins (`INCUS_TEST_BEGIN`: the far
    /// length alone). The reach as it is, and 0, unset.
    reach: f32,
    begin: f32,
    /// The longest ray (the Oracle's step 0): the pass that counts a layer's rays' steps, each layer's four counts on the
    /// card, where they're read back, and whether a reading is free (0), on its way (1) or ready (2).
    tally: wgpu::ComputePipeline,
    tally_counts: Vec<wgpu::Buffer>,
    tally_back: wgpu::Buffer,
    tally_state: std::sync::Arc<std::sync::atomic::AtomicU8>,
    /// Threads kept waiting (`far_waiting.wgsl`; build 184): the march's grid counted group by group with the rays'
    /// count, by the one dispatch; its counts are read back after the layers'.
    waiting: wgpu::ComputePipeline,
    waiting_counts: wgpu::Buffer,
}

impl FarEye {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, store: Store, lane: Option<Lane>, rings: Vec<crate::far_ring::Ring>) -> Self {
        // The march's source: as it's written, or for a picture test asked to count a ray's steps by kind, a variant
        // that counts them (`kinds_source`; the game never builds it, so it costs the game nothing).
        #[cfg(not(test))]
        let far_source = include_str!("far.wgsl").to_string();
        #[cfg(test)]
        let far_source = kinds_source(include_str!("far.wgsl"));
        // Cover by faces compiled out, if asked (`INCUS_FAR_FACES=0`): build 190's program, to measure against.
        const FACES: &str = "const FACES: bool = true;";
        assert_eq!(far_source.matches(FACES).count(), 1, "far.wgsl says whether faces stop a ray in one place");
        const NO_FACES: &str = "const FACES: bool = false;";
        let faces_asked = std::env::var("INCUS_FAR_FACES").as_deref() != Ok("0");
        // The text with the rule the other way, for the Render page's switch.
        let (far_source, far_source_other) = if faces_asked {
            let other = far_source.replace(FACES, NO_FACES);
            (far_source, other)
        } else {
            log::info!("Cover by faces: off, a far cell stops a ray by its bulk alone (INCUS_FAR_FACES=0)");
            (far_source.replace(FACES, NO_FACES), far_source)
        };
        // Far rays skipped under the near eye's blocks (build 201), compiled out if asked (`INCUS_FAR_COVER=0`): build
        // 200's march, to measure against.
        const COVER: &str = "const COVER: bool = true;";
        assert_eq!(far_source.matches(COVER).count(), 1, "far.wgsl says whether the near eye's cover skips rays in one place");
        const NO_COVER: &str = "const COVER: bool = false;";
        let cover_on = std::env::var("INCUS_FAR_COVER").as_deref() != Ok("0");
        let (far_source, far_source_other) = if cover_on {
            (far_source, far_source_other)
        } else {
            log::info!("The near eye's cover: off, the far eye marches under blocks too (INCUS_FAR_COVER=0)");
            (far_source.replace(COVER, NO_COVER), far_source_other.replace(COVER, NO_COVER))
        };
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye"), source: wgpu::ShaderSource::Wgsl(far_source.as_str().into()) });
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
                uniform(1, compute),
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
                read(24, compute, false),
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
        look.faces = std::env::var("INCUS_FAR_FACES").as_deref() != Ok("0");
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
        let blue_side = crate::blue_noise::SIDE as u32;
        let blue = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("the blue noise"),
            size: wgpu::Extent3d { width: blue_side, height: blue_side, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let blue = (blue.clone(), blue.create_view(&wgpu::TextureViewDescriptor::default()));
        let blue_made = Instant::now();
        let blue_bytes = Some(crate::blue_noise::bytes());
        log::info!("The blue noise: {blue_side} x {blue_side} made in {:.3} s", blue_made.elapsed().as_secs_f32());
        let edge = std::env::var("INCUS_FAR_EDGE").as_deref() != Ok("0");
        let haze_step = std::env::var("INCUS_FAR_HAZE_STEP").ok().and_then(|s| s.parse::<f32>().ok()).filter(|s| (0.25..=2.0).contains(s)).unwrap_or(1.0);
        let base = if std::env::var("INCUS_FAR_BASE").as_deref() == Ok("4") { 4.0 } else { LAYERS[0] };
        let layers = [base, base / 2.0, base / 4.0];
        // The one dispatch: `far.wgsl` with one layer's outputs changed for every layer's (`far_all.wgsl`), and its
        // camera read in place from its layer's row.
        let mut audit = Vec::new();
        let mut one_loaded = None;
        let mut one_loaded_other = None;
        let mut one_start = None;
        let mut one_join = None;
        let one = (std::env::var("INCUS_FAR_SIDE").as_deref() != Ok("0")).then(|| {
            let source = one_source(far_source.as_str());
            // The two programs from the one text: the whole march under its own name, and the march once the textures
            // are in, which keeps the name the far eye's march has always had.
            const WHICH: &str = "const LOADED: bool = false;";
            assert_eq!(source.matches(WHICH).count(), 1, "far.wgsl says which program it is in one place");
            let mut loaded_source = source.replace(WHICH, "const LOADED: bool = true;");
            // The same program with cover by faces the other way (the Render page's switch), unless an audit's
            // version is to march in its place.
            let mut loaded_source_other = Some(one_source(far_source_other.as_str()).replace(WHICH, "const LOADED: bool = true;"));
            // For measuring (`INCUS_FAR_AS`, the name of one of the audit's second batch): that version marched as
            // the far eye's own program once the textures are in, its picture wrong on purpose, so a flight says what
            // its room and its shorter step are worth with every layer marching (the audit marches the base alone).
            if let Ok(asked) = std::env::var("INCUS_FAR_AS") {
                let text = audit_variants(far_source.as_str(), 2).into_iter().find(|(name, _)| *name == asked).unwrap_or_else(|| panic!("INCUS_FAR_AS={asked} is none of the audit's second batch")).1;
                loaded_source = one_source(&text);
                loaded_source_other = None;
                log::warn!("The far eye: once the textures are in, marched by the audit's {asked} (for measuring: the picture is wrong on purpose)");
            }
            let source = source.replace("fn march_all(", "fn march_loading(");
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye, one dispatch"), source: wgpu::ShaderSource::Wgsl(source.into()) });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("the far eye, one dispatch"),
                entries: &[
                    uniform(0, compute),
                    uniform(1, compute),
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
                read(24, compute, false),
                    written(17, wgpu::TextureFormat::Rgba8Unorm),
                    written(19, wgpu::TextureFormat::Rgba8Unorm),
                    storage(25, false),
                    read(26, compute, false),
                ],
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("the far eye, one dispatch"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("march_loading"), layout: Some(&pipeline_layout), module: &module, entry_point: Some("march_loading"), compilation_options: Default::default(), cache: None });
            one_start = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("start_pass"), layout: Some(&pipeline_layout), module: &module, entry_point: Some("start_pass"), compilation_options: Default::default(), cache: None }));
            let loaded_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye, once the textures are in"), source: wgpu::ShaderSource::Wgsl(loaded_source.into()) });
            one_loaded = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("march_all"), layout: Some(&pipeline_layout), module: &loaded_module, entry_point: Some("march_all"), compilation_options: Default::default(), cache: None }));
            if let Some(other) = loaded_source_other {
                let began = std::time::Instant::now();
                let other_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the far eye, once the textures are in, cover by faces the other way"), source: wgpu::ShaderSource::Wgsl(other.into()) });
                one_loaded_other = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("march_all"), layout: Some(&pipeline_layout), module: &other_module, entry_point: Some("march_all"), compilation_options: Default::default(), cache: None }));
                log::info!("Cover by faces: the march's other program ({}) made in {:.1} s, for the Render page's switch", if faces_asked { "without it" } else { "with it" }, began.elapsed().as_secs_f32());
                #[cfg(test)]
                eprintln!("Cover by faces: the march's other program made in {:.1} s", began.elapsed().as_secs_f32());
            }
            // The join (the cut), from each of the two programs' text: a cut ray is finished as its program finishes
            // any ray.
            one_join = Some([&module, &loaded_module].map(|module| device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("join_pass"), layout: Some(&pipeline_layout), module, entry_point: Some("join_pass"), compilation_options: Default::default(), cache: None })));
            // The audit's variants, if asked: the same layout, each its own module and its own name.
            if let Some(batch) = std::env::var("INCUS_FAR_AUDIT").ok().and_then(|asked| asked.parse::<u32>().ok()).filter(|batch| *batch >= 1) {
                for (name, text) in audit_variants(far_source.as_str(), batch) {
                    let source = one_source(&text).replace("fn march_all(", &format!("fn {name}("));
                    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(name), source: wgpu::ShaderSource::Wgsl(source.into()) });
                    audit.push((name, device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some(name), layout: Some(&pipeline_layout), module: &module, entry_point: Some(name), compilation_options: Default::default(), cache: None })));
                }
                log::info!("The march's audit: {} cut-down versions compiled; each is marched over the base layer into scratch after the far eye (for measuring: the frame is that much longer)", audit.len());
            }
            (pipeline, layout)
        });
        let blocks = |name: &str| std::env::var(name).ok().and_then(|n| n.parse::<f32>().ok()).filter(|n| *n > 0.0);
        let (reach, begin) = (blocks("INCUS_TEST_REACH").unwrap_or(REACH), blocks("INCUS_TEST_BEGIN").unwrap_or(0.0));
        let most_steps = std::env::var("INCUS_TEST_STEPS").ok().and_then(|n| n.parse::<f32>().ok()).filter(|n| *n >= 1.0).unwrap_or(768.0);
        log::info!("The far rays: {}, {most_steps:.0} steps at most", if edge { "ended at the map's edge" } else { "marched on past the map's edge (INCUS_FAR_EDGE=0)" });
        // The cut, on unless told (`INCUS_FAR_CUT`: 0, no ray is cut; or where the first place a ray may be cut at
        // is, blocks out; the places after it each half a doubling of distance on, `far.wgsl`'s `cut_place`). [Flown
        // packed and trimmed, build 186: the far eye's time a quarter to two fifths less at five of nine views (the
        // plain 0.872 to 0.568 ms, the sea 1.071 to 0.651), 1 to 4% more at the other four; two views over 1 ms for
        // four. David, 2026-11-15: "we can move on, if it gets a little better overall than it is now", which it
        // did. Build 183's first form, sixteen threads a ray, was a sixth to a third dearer where the card works.]
        let cut_at = match std::env::var("INCUS_FAR_CUT").ok().and_then(|n| n.parse::<f32>().ok()) {
            Some(asked) if asked <= 0.0 => 0.0,
            Some(asked) => asked.clamp(64.0, 65536.0),
            None => FIRST_CUT,
        };
        // How many steps make a ray long (`INCUS_FAR_LONG`, for measuring).
        let long = std::env::var("INCUS_FAR_LONG").ok().and_then(|n| n.parse::<u32>().ok()).map_or(LONG_STEPS, |n| n.clamp(16, 768));
        if one.is_some() && long != LONG_STEPS {
            log::info!("The cut: a ray is long at {long} steps or more (INCUS_FAR_LONG)");
        }
        if one.is_some() && cut_at != FIRST_CUT {
            log::info!("The cut: {}", if cut_at > 0.0 { format!("the first place a ray may be cut at is {cut_at:.0} blocks out (INCUS_FAR_CUT)") } else { "off, every ray marched whole (INCUS_FAR_CUT=0)".to_string() });
        }
        let tally_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the longest ray"), source: wgpu::ShaderSource::Wgsl(include_str!("far_tally.wgsl").into()) });
        let tally = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("the longest ray"), layout: None, module: &tally_module, entry_point: Some("tally_pass"), compilation_options: Default::default(), cache: None });
        let waiting_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("threads kept waiting"), source: wgpu::ShaderSource::Wgsl(include_str!("far_waiting.wgsl").into()) });
        let waiting = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("threads kept waiting"), layout: None, module: &waiting_module, entry_point: Some("waiting_pass"), compilation_options: Default::default(), cache: None });
        let waiting_counts = device.create_buffer(&wgpu::BufferDescriptor { label: Some("threads kept waiting"), size: 4 * TALLY as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let tally_counts = LAYERS.iter().map(|_| device.create_buffer(&wgpu::BufferDescriptor { label: Some("a far layer's longest ray"), size: 4 * TALLY as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })).collect();
        let tally_back = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the longest rays read back"), size: 4 * (TALLY * (LAYERS.len() + 1)) as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let tally_state = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));
        Self { near_from: 0.0, targets_made: 0, cover_on, placed_job: None, placed_ready: None, placed_regions: std::collections::HashSet::new(), relight: false, placed_asked: None, store, lane, rings, eye: [0.0; 2], start, march, up, layout, up_layout, sampler, targets: None, look, textures, atlas: None, building: None, stand_ins, light: None, lighting: None, sweeping: None, sun_asked, light_stand_ins, blue, blue_bytes, edge, layers, one, one_loaded, one_loaded_other, faces_asked, one_start, one_join, cut_at, long, loaded_off: std::env::var("INCUS_FAR_LOADED").as_deref() == Ok("0"), loaded_said: std::sync::atomic::AtomicBool::new(false), audit, parity: 0, fresh: false, haze_step, most_steps, tops_making: None, tops_made: None, tops_off: false, reach, begin, tally, tally_counts, tally_back, tally_state, waiting, waiting_counts }
    }

    /// Whether every region is on the card, the window's regions round the eye too, and the far textures if they're made.
    pub fn whole(&self) -> bool {
        // (The rings: once each has been on the card, with its textures if the atlas is in. Not held to the eye's place: a
        // moving mark outruns it, as it does the window.)
        self.lane.as_ref().is_none_or(|lane| lane.done == lane.total) && (self.atlas.is_some() || !self.textures) && self.store.behind == 0 && self.lit() && self.rings.iter().all(|ring| ring.from.is_some() || ring.failed) && self.tops_made.is_some()
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

    /// What the near eye reads of the far eye's (`near.wgsl`): the base layer's camera, the levels' table, the light's
    /// two pictures (the stand-ins till the light is in) and the sampler; and a number that changes when any of them
    /// is another buffer or picture. None till the targets are made.
    pub fn near_reads(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, &wgpu::TextureView, &wgpu::TextureView, &wgpu::Sampler, u64)> {
        let targets = self.targets.as_ref()?;
        let (shadow, sky) = self.light.as_ref().map_or((&self.light_stand_ins[0], &self.light_stand_ins[1]), |l| (&l.shadow_views[l.shown], &l.sky));
        let light = self.light.as_ref().map_or(0, |l| 1 + l.generation as u64 * 2 + l.shown as u64);
        Some((&targets.layers.first()?.camera, &self.store.table, shadow, sky, &self.sampler, self.targets_made << 32 | light << 8 | self.store.generation as u64 & 0xFF))
    }

    /// Whether the Render page's cover-by-faces switch can do anything now: there's a second program, and the march
    /// once the textures are in is what's marching (the whole march draws by the launch's choice).
    pub fn faces_switch(&self) -> bool {
        self.one_loaded_other.is_some() && !self.loaded_off && self.atlas.is_some() && self.look.blend >= 1 && self.look.view == 0 && self.rings.iter().all(|ring| ring.textured())
    }

    /// Whether the far eye marches by its second program, the march once the textures are in: the hardware's blend
    /// chosen and its textures in, every ring's bricks' too (a ring's come a little after the atlas, and till then
    /// only the code's blend can answer for them), and no view on. Otherwise by the whole march.
    pub fn loaded(&self) -> bool {
        let loaded = !self.loaded_off && self.one_loaded.is_some() && self.atlas.is_some() && self.look.blend >= 1 && self.look.view == 0 && self.rings.iter().all(|ring| ring.textured());
        if loaded && !self.loaded_said.swap(true, std::sync::atomic::Ordering::Relaxed) {
            log::info!("The far eye: marched from here by the march once the textures are in (no code's blend, no views)");
        }
        loaded
    }

    /// The look's settings a flight's numbers depend on, for its report.
    pub fn settings(&self) -> String {
        let window = self.store.window().map_or("the whole world on the card".to_string(), |w| format!("the 32s within {} regions of the eye, the 64s within {}", w[0], w[1]));
        let mut rays = if self.edge { "rays ended at the map's edge".to_string() } else { "rays marched past the map's edge".to_string() };
        rays += &format!(", thin haze stepped by {} of a cell; the layers {} x {}, {} x {} and {} x {} screen pixels", self.haze_step, self.layers[0], self.layers[0], self.layers[1], self.layers[1], self.layers[2], self.layers[2]);
        rays += ", light read from the cell";
        if !self.rings.is_empty() {
            let sizes: Vec<String> = self.rings.iter().map(|ring| format!("{}s", 1 << ring.level)).collect();
            rays += &format!(", the {} in rings round the eye", sizes.join(" and the "));
        }
        rays += if self.one.is_some() { ", marched by one dispatch (a finer layer led by the frame before's coarser)" } else { ", marched one after another" };
        if self.most_steps != 768.0 {
            rays += &format!(", {:.0} steps at most", self.most_steps);
        }
        if self.reach != REACH {
            rays += &format!(", FOR MEASURING rays ended {:.0} blocks out (the near length alone)", self.reach);
        }
        if self.begin != 0.0 {
            rays += &format!(", FOR MEASURING rays begun {:.0} blocks out (the far length alone)", self.begin);
        }
        // Where the rays' steps begin, if not as it was (scattered, over a whole step).
        let look = &self.look;
        if look.start_pattern != 0 || look.start_spread != 1.0 {
            let side = 1 << look.weave_bits.clamp(1, 4);
            rays += &match look.start_pattern {
                1 => format!(", rays' starts woven {side} x {side}"),
                2 => ", rays' starts by blue noise".to_string(),
                3 => format!(", rays' starts woven {side} x {side} with {} of blue noise", look.start_blue),
                _ => ", rays' starts scattered".to_string(),
            };
            if look.start_spread != 1.0 {
                rays += &format!(" over {} of a step", look.start_spread);
            }
        }
        format!("bricks of {}, {window}, {rays}, blended by {}, detail {:+}", self.store.side, if self.look.blend == 1 && self.atlas.is_some() { "the hardware (spread)" } else { "the initial blend (the code's, old's)" }, self.look.detail)
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
        let ring: String = self.rings.iter().map(|ring| format!("; {}", ring.state())).collect();
        let light = self.light.as_ref().map_or(String::new(), |l| format!("; the far light {:.0} MB, its tops and sky made in {:.2} s, its last shadow swept and sent in {:.3} s", l.bytes as f64 / 1e6, l.made_in, l.swept_in));
        let tops = match self.tops_made {
            Some(_) if self.tops_off => "; the height table left as the bands' rounded tops (FOR MEASURING)".to_string(),
            Some(seconds) => format!("; the height table {:.0} MB, made and sent in {seconds:.2} s", self.store.tops_at as f64 * 4.0 / 1e6),
            None => String::new(),
        };
        format!("{lane}{} bricks, {:.0} MB of {:.0} MB{atlas}{ring}{tops}{light}", self.store.bricks, used as f64 / 1e6, size as f64 / 1e6)
    }

    /// The regions summed since the last frame put on the card, and the window brought to the eye, for at most
    /// `PUTTING_MS` together (the pass "putting the far cells on the card").
    pub fn put(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, eye: [f32; 2]) {
        let started = Instant::now();
        self.eye = eye;
        if let Some(bytes) = self.blue_bytes.take() {
            let side = crate::blue_noise::SIDE as u32;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &self.blue.0, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                &bytes,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(side), rows_per_image: Some(side) },
                wgpu::Extent3d { width: side, height: side, depth_or_array_layers: 1 },
            );
        }
        // The atlas, once every region is in: built on a thread of its own from the cells kept and the window as it is
        // (held still till it's in), then taken in here.
        if self.lane.is_none() && self.atlas.is_none() && self.building.is_none() && self.textures {
            let kept = self.store.kept.clone();
            let window = self.store.window_regions();
            let windowed = if window.is_some() { crate::bricks::WINDOWED } else { 0 };
            let places = self.store.index_count as usize;
            let (device, queue) = (device.clone(), queue.clone());
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("making the far textures".into()).spawn(move || {
                let keep = |b: &crate::atlas::Brick| window.as_ref().is_none_or(|regions| b.level >= crate::far::FIRST + crate::bricks::WINDOWED || regions[b.level - crate::far::FIRST].contains(&crate::atlas::region_of(b)));
                let _ = sender.send(crate::atlas::build(&device, &queue, &kept, places, windowed, keep));
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
        // The height table's true numbers, once every region is in: worked out on a thread of its own and sent whole.
        if self.lane.is_none() && self.tops_made.is_none() && self.tops_making.is_none() && std::env::var("INCUS_FAR_TOPS").as_deref() == Ok("0") {
            // (For flying against: the true numbers left unmade.)
            self.tops_made = Some(0.0);
            self.tops_off = true;
        }
        if self.lane.is_none() && self.tops_made.is_none() && self.tops_making.is_none() {
            let kept = self.store.kept.clone();
            let (queue, entries, at) = (queue.clone(), self.store.entries.clone(), self.store.tops_at);
            let (sender, receiver) = mpsc::channel();
            let _ = std::thread::Builder::new().name("making the height table".into()).spawn(move || {
                let started = Instant::now();
                for (start, tops) in crate::bricks::tops_whole(&kept) {
                    queue.write_buffer(&entries, (at + start as u64) * 4, &tops.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
                }
                let _ = sender.send(started.elapsed().as_secs_f32());
            });
            self.tops_making = Some(receiver);
        }
        if let Some(receiver) = &self.tops_making
            && let Ok(seconds) = receiver.try_recv()
        {
            log::info!("The height table: every column's highest thing worked out and sent in {seconds:.2} s");
            self.tops_made = Some(seconds);
            self.tops_making = None;
        }
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
        self.placed(device, queue);
        // The window brought to the eye with what's left of the pass's time (a millisecond at least), half of it when
        // the atlas follows too; held still while the atlas is being made from it.
        if self.building.is_none() {
            let left = (PUTTING_MS - started.elapsed().as_secs_f32() * 1000.0).max(1.0);
            let moved = self.store.follow(device, queue, eye, if self.atlas.is_some() { left / 2.0 } else { left });
            if let Some(atlas) = &mut self.atlas {
                for &(level, region) in &moved.left {
                    atlas.remove_region(level, region);
                }
                for &(level, region) in &moved.came {
                    atlas.add_region(device, queue, &self.store.kept, level, region);
                }
            }
        }
        // The rings brought to the eye: what their makers have handed over sent on, a little a frame each.
        for ring in &mut self.rings {
            ring.follow(device, queue, &mut self.store, self.atlas.as_mut(), eye, RING_MS);
        }
    }

    /// What's placed through the doorway, as it is now (build 207; a mark's changes are made, and the mark before's taken
    /// back, as the camera goes to it): the far regions these changes touch, and those the last ones did, summed again
    /// with them on a thread of their own; the rings told too.
    pub fn changed(&mut self, changes: &std::collections::HashMap<[i32; 2], Vec<crate::doorway::Change>>) {
        let region = 1i32 << crate::far::REGION;
        let mut touched = std::collections::HashSet::new();
        for change in changes.values().flatten() {
            let cells = match change.octants() {
                None => vec![change.cell()],
                Some((octants, _)) => octants.iter().map(|octant| octant.map(|half| half.div_euclid(2))).collect(),
            };
            for c in cells {
                for dn in -1..=1 {
                    for de in -1..=1 {
                        touched.insert([(c[0] + dn).div_euclid(region), (c[1] + de).div_euclid(region)]);
                    }
                }
            }
        }
        let (from, across) = self.store.regions;
        touched.retain(|r| (from[0]..from[0] + across).contains(&r[0]) && (from[1]..from[1] + across).contains(&r[1]));
        let mut regions = touched.clone();
        regions.extend(self.placed_regions.iter().copied());
        if let Some((_, pending, _)) = self.placed_job.take() {
            regions.extend(pending);
        }
        if let Some((ready, _)) = self.placed_ready.take() {
            regions.extend(ready.iter().map(|(r, _)| *r));
        }
        for ring in &mut self.rings {
            ring.changed(changes.clone());
        }
        if regions.is_empty() {
            return;
        }
        let list: Vec<[i32; 2]> = regions.iter().copied().collect();
        let changes = changes.clone();
        let (sender, receiver) = mpsc::channel();
        let spawned = std::thread::Builder::new().name("summing what's placed".into()).spawn(move || {
            let made = (|| {
                let mut doorway = crate::doorway::Doorway::open(&crate::doorway::test_world())?;
                let colours = crate::far::FarColours::load(&mut doorway)?;
                doorway.set_changes(changes);
                list.iter().map(|r| crate::far::region(&doorway, &colours, *r).map(|columns| (*r, columns))).collect::<Result<Vec<_>, String>>()
            })();
            let _ = sender.send(made);
        });
        if let Err(error) = spawned {
            log::error!("What's placed couldn't be summed for the far eye: {error}");
            return;
        }
        self.placed_job = Some((receiver, regions, touched));
        self.placed_asked = Some(Instant::now());
    }

    /// What's placed put on the card (build 207), once it's summed, every region is in and nothing else holds the cells
    /// kept: each region summed again put over what was there, the far textures of its bricks (and of those beside it
    /// that read its cells) made again, and the height table and the light worked out again on their threads.
    fn placed(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some((receiver, _, _)) = &self.placed_job
            && let Ok(made) = receiver.try_recv()
        {
            let (_, _, touched) = self.placed_job.take().unwrap();
            match made {
                Ok(columns) => self.placed_ready = Some((columns, touched)),
                Err(error) => log::error!("What's placed couldn't be summed for the far eye: {error}"),
            }
        }
        if self.lane.is_some() || self.placed_ready.is_none() || std::sync::Arc::strong_count(&self.store.kept) > 1 {
            return;
        }
        let Some((columns, touched)) = self.placed_ready.take() else { return };
        let started = Instant::now();
        let windowed = self.store.window().map_or(0, |_| crate::bricks::WINDOWED);
        for (region, levels) in &columns {
            self.store.resum_region(device, queue, *region, levels);
        }
        if let Some(atlas) = &mut self.atlas {
            let kept = &self.store.kept;
            let mut done = std::collections::HashSet::new();
            for (region, _) in &columns {
                for level in crate::far::FIRST..=crate::far::REGION {
                    for (dx, dy) in [(0, 0), (-1, 0), (0, -1), (-1, -1)] {
                        let r = [region[0] + dx, region[1] + dy];
                        if done.insert((level, r)) {
                            atlas.redo_region(device, queue, kept, level, r, level < crate::far::FIRST + windowed);
                        }
                    }
                }
                let mut place = *region;
                for level in crate::far::REGION + 1..=crate::bricks::TOP {
                    place = [place[0].div_euclid(2), place[1].div_euclid(2)];
                    if done.insert((level, place)) {
                        atlas.redo_upper(device, queue, kept, level, place);
                    }
                }
            }
        }
        self.placed_regions = touched;
        if !self.tops_off {
            self.tops_made = None;
        }
        self.relight = true;
        log::info!(
            "What's placed: {} far regions summed again and put on the card in {:.1} ms ({:.2} s after it was made)",
            columns.len(),
            started.elapsed().as_secs_f32() * 1000.0,
            self.placed_asked.map_or(0.0, |at| at.elapsed().as_secs_f32())
        );
    }

    /// Whether the far eye has what's placed as last told: summed, on the card, its light made again, and the rings
    /// made with it (a flight's standing waits for this).
    pub fn changes_settled(&self) -> bool {
        self.placed_job.is_none() && self.placed_ready.is_none() && !self.relight && self.lighting.is_none() && self.rings.iter().all(|ring| ring.changes_settled())
    }

    /// Whether the rings round the eye are on the card whole (true with none), for a picture.
    #[cfg(test)]
    pub fn ring_settled(&self) -> bool {
        self.rings.iter().all(|ring| ring.settled(&self.store, self.eye, self.atlas.is_some()))
    }

    /// The far light kept up (read from the cell): made once every region is in, on a thread of its own; and a new
    /// shadow swept, on a thread too, when the sun's setting has changed and stood still a quarter of a second, shown
    /// whole when it's in.
    fn light(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let sun = [self.look.sun_from, self.look.sun_height];
        if sun != self.sun_asked.0 {
            self.sun_asked = (sun, Instant::now());
        }
        if self.lane.is_none() && (self.light.is_none() || self.relight) && self.lighting.is_none() {
            self.relight = false;
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
        let long_rays = buffer("the long rays' counts", 16, storage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC);
        let cut_bytes = CUT_WORDS as u64 * 4;
        let cut = buffer("the cut's lists and lengths", cut_bytes, storage | wgpu::BufferUsages::COPY_SRC);
        let nothing_above = buffer("the far layers' nothing above", 16, storage);
        let no_far = texture("the far layers' no distances above", (1, 1), wgpu::TextureFormat::Rgba32Float);
        let mut layers: Vec<Layer> = Vec::new();
        let mut worth_above: Option<wgpu::Buffer> = None;
        let sizes = self.layers.map(|factor| ((screen.0 as f32 / factor).ceil().max(1.0) as u32, (screen.1 as f32 / factor).ceil().max(1.0) as u32));
        // The near eye's cover: a pixel a pixel of each layer, side by side (nothing covered till the near eye's pass
        // writes it).
        let cover = texture("the near eye's cover", (sizes.iter().map(|size| size.0).sum(), sizes.iter().map(|size| size.1).max().unwrap_or(1)), wgpu::TextureFormat::Rgba8Unorm);
        // By the one dispatch: all the layers' distances in one texture, side by side from its left, kept twice.
        let one_way = self.one.is_some();
        // The strip of the dispatch's grid for the cut rays' lengths: a thread a place in the list of lengths, row
        // after row down the grid's rows, in whole groups of 8 across.
        let strip = PIECES.div_ceil(sizes.iter().map(|s| s.1.div_ceil(8)).max().unwrap_or(1) * 8).div_ceil(8) * 8;
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
                    entry(24, wgpu::BindingResource::TextureView(&self.blue.1)),
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
            // A group for the march (`start` false: this frame's distances written, the frame before's read) or for
            // the start pass (true: the frame before's distances written, where the tiles' starts go, and nothing
            // of them read), for each of the two copies.
            let group = |p: usize, start: bool| {
                let q = 1 - p;
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(if start { "the far start, one dispatch" } else { "the far eye, one dispatch" }),
                    layout,
                    entries: &[
                        entry(0, cameras.as_entire_binding()),
                        entry(1, self.store.table.as_entire_binding()),
                        entry(2, self.store.entries.as_entire_binding()),
                        entry(3, self.store.index.as_entire_binding()),
                        entry(4, self.store.pool.as_entire_binding()),
                        entry(5, wgpu::BindingResource::TextureView(&layers[0].picture)),
                        entry(6, wgpu::BindingResource::TextureView(&layers[0].far[if start { q } else { p }])),
                        entry(7, long_rays.as_entire_binding()),
                        entry(8, worth[q].as_entire_binding()),
                        entry(9, worth[p].as_entire_binding()),
                        entry(10, wgpu::BindingResource::TextureView(if start { &no_far } else { &layers[0].far[q] })),
                        entry(11, self.atlas.as_ref().map_or(&self.store.index, |a| &a.slots).as_entire_binding()),
                        entry(12, wgpu::BindingResource::Sampler(&self.sampler)),
                        entry(13, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[0], |a| &a.solid))),
                        entry(14, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[0]))),
                        entry(15, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[1]))),
                        entry(16, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[2]))),
                    entry(22, wgpu::BindingResource::TextureView(shadow)),
                    entry(23, wgpu::BindingResource::TextureView(sky)),
                    entry(24, wgpu::BindingResource::TextureView(&self.blue.1)),
                        entry(17, wgpu::BindingResource::TextureView(&layers[1].picture)),
                        entry(19, wgpu::BindingResource::TextureView(&layers[2].picture)),
                        entry(25, cut.as_entire_binding()),
                        entry(26, wgpu::BindingResource::TextureView(&cover)),
                    ],
                })
            };
            let (groups, starts) = ([group(0, false), group(1, false)], [group(0, true), group(1, true)]);
            (cameras, groups, starts)
        });
        let (one, one_starts) = match one {
            Some((cameras, groups, starts)) => (Some((cameras, groups)), Some(starts)),
            None => (None, None),
        };
        // The audit's bind group: the far eye's own, with scratch in place of all it writes (its pictures, its
        // distances, its words) and of the words it reads, so a variant's march touches nothing that's shown or kept.
        // (Scratch a few pixels across: a write outside a texture isn't made, and a word past a buffer's end lands on
        // its last.)
        let audit = (!self.audit.is_empty()).then(|| self.one.as_ref().zip(one.as_ref())).flatten().map(|((_, layout), (cameras, _))| {
            let pictures = [0, 1, 2].map(|_| texture("the audit's scratch", (8, 8), wgpu::TextureFormat::Rgba8Unorm));
            let distances = texture("the audit's scratch", (8, 8), wgpu::TextureFormat::Rgba32Float);
            let words = [buffer("the audit's scratch", 16, storage), buffer("the audit's scratch", 16, storage)];
            let lists = buffer("the audit's scratch", 8, storage);
            let lengths = buffer("the audit's scratch", cut_bytes, storage);
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("the march's audit"),
                layout,
                entries: &[
                    entry(0, cameras.as_entire_binding()),
                    entry(1, self.store.table.as_entire_binding()),
                    entry(2, self.store.entries.as_entire_binding()),
                    entry(3, self.store.index.as_entire_binding()),
                    entry(4, self.store.pool.as_entire_binding()),
                    entry(5, wgpu::BindingResource::TextureView(&pictures[0])),
                    entry(6, wgpu::BindingResource::TextureView(&distances)),
                    entry(7, lists.as_entire_binding()),
                    entry(8, words[0].as_entire_binding()),
                    entry(9, words[1].as_entire_binding()),
                    entry(10, wgpu::BindingResource::TextureView(&layers[0].far[0])),
                    entry(11, self.atlas.as_ref().map_or(&self.store.index, |a| &a.slots).as_entire_binding()),
                    entry(12, wgpu::BindingResource::Sampler(&self.sampler)),
                    entry(13, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[0], |a| &a.solid))),
                    entry(14, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[0]))),
                    entry(15, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[1]))),
                    entry(16, wgpu::BindingResource::TextureView(self.atlas.as_ref().map_or(&self.stand_ins[1], |a| &a.spread[2]))),
                    entry(22, wgpu::BindingResource::TextureView(shadow)),
                    entry(23, wgpu::BindingResource::TextureView(sky)),
                    entry(24, wgpu::BindingResource::TextureView(&self.blue.1)),
                    entry(17, wgpu::BindingResource::TextureView(&pictures[1])),
                    entry(19, wgpu::BindingResource::TextureView(&pictures[2])),
                    entry(25, lengths.as_entire_binding()),
                    entry(26, wgpu::BindingResource::TextureView(&cover)),
                ],
            })
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
        let tally_parts: Vec<wgpu::Buffer> = layers.iter().map(|_| buffer("a far layer's part of its distances", 32, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)).collect();
        let tally_groups = layers
            .iter()
            .zip(&self.tally_counts)
            .zip(&tally_parts)
            .map(|((layer, counts), part)| [0usize, 1].map(|p| device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("the longest ray"), layout: &self.tally.get_bind_group_layout(0), entries: &[entry(0, wgpu::BindingResource::TextureView(&layer.far[p])), entry(1, counts.as_entire_binding()), entry(2, part.as_entire_binding())] })))
            .collect();
        let waiting_numbers = buffer("threads kept waiting, its numbers", 96, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let waiting_groups = one_way.then(|| {
            [0usize, 1].map(|p| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("threads kept waiting"),
                    layout: &self.waiting.get_bind_group_layout(0),
                    entries: &[
                        entry(0, wgpu::BindingResource::TextureView(&layers[0].far[p])),
                        entry(1, wgpu::BindingResource::TextureView(&layers[0].far[1 - p])),
                        entry(2, cut.as_entire_binding()),
                        entry(3, waiting_numbers.as_entire_binding()),
                        entry(4, self.waiting_counts.as_entire_binding()),
                    ],
                })
            })
        });
        self.targets_made += 1;
        self.targets = Some(Targets { screen, generation: self.store.generation, atlas_in: self.atlas.is_some(), atlas_generation, light: light_now, tiles, tally_groups, tally_parts, waiting_numbers, waiting_groups, layers, up_numbers, up_group, one, one_starts, long_rays, cut, strip, cover, audit });
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
        // This frame's list of long rays begins empty, and its list of lengths.
        queue.write_buffer(&targets.long_rays, self.parity as u64 * 4, &0u32.to_le_bytes());
        queue.write_buffer(&targets.long_rays, (2 + self.parity as u64) * 4, &0u32.to_le_bytes());
        let cutting = self.cutting();
        let (facing, pitch) = (camera.facing.to_radians(), camera.pitch.to_radians());
        let forward = [pitch.cos() * facing.cos(), pitch.cos() * facing.sin(), pitch.sin()];
        let right = [-facing.sin(), facing.cos(), 0.0];
        let up = [-pitch.sin() * facing.cos(), -pitch.sin() * facing.sin(), pitch.cos()];
        let tan_half = camera.tan_half();
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
        // The rings as they are on the card: each one's box (an empty one while it isn't there), and the finest level
        // the march asks for (the finest ring's on the card).
        let mut ring_boxes = [[1.0f32, 1.0, 0.0, 0.0]; 4];
        let mut finest_level = crate::far::FIRST;
        for ring in &self.rings {
            if ring.from.is_some() {
                ring_boxes[crate::far::FIRST - 1 - ring.level] = ring.reach(&self.store.levels[crate::bricks::row(ring.level)]);
                finest_level = finest_level.min(ring.level);
            }
        }
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
                [self.reach, -look.detail as f32, look.view as f32, self.store.highest],
                [targets.tiles.0 as f32, targets.tiles.1 as f32, (TILE as f32 / layer.factor).max(1.0), screen_pixel * (TILE as f32 * 0.5 * std::f32::consts::SQRT_2 + 1.0)],
                [above.map_or(0.0, |a| a.size.0 as f32), if below.is_some() { 1.0 } else { 0.0 }, 1.0, blend as f32],
                [1.0, if look.shadows && direct > 0.0 { 1.0 } else { 0.0 }, finest_level as f32, if self.edge { 1.0 } else { 0.0 }],
                [above.map_or(0.0, |a| layer.factor / a.factor), below.map_or(0.0, |b| b.factor / layer.factor), self.most_steps, self.store.tops_at as f32],
                [fade[0].max(0.1), fade[1].max(0.05), look.hand_over.clamp(0.3, 0.95), look.sky_weight.max(0.0)],
                [if look.sky_light { 1.0 } else { 0.0 }, look.sky_strength.clamp(0.0, 1.0), look.sky_tint.clamp(0.0, 1.0), self.begin.max(self.near_from)],
                [if look.fog { 3.0 / look.fog_reach.max(10.0) } else { 0.0 }, std::f32::consts::LN_2 / look.fog_height.max(1.0), sea_level, look.fog_warmth.clamp(0.0, 1.0)],
                [direct, NIGHT_LIGHT + (1.0 - NIGHT_LIGHT) * glow, glow, NIGHT_LIGHT],
                [water[0], water[1], water[2], 20f32.ln() / WATER_CLARITY],
                [atlas_size[0], atlas_size[1], atlas_size[2], self.atlas.as_ref().map_or(1.0, |a| a.side as f32)],
                [window_levels, regions.1 as f32, regions.0[0] as f32, regions.0[1] as f32],
                [look.start_pattern.min(3) as f32, look.weave_bits.clamp(1, 4) as f32, look.start_blue.clamp(0.0, 1.0), look.start_spread.clamp(0.0, 1.0)],
                ring_boxes[0],
                ring_boxes[1],
                ring_boxes[2],
                ring_boxes[3],
                // The long rays' lists: which is this frame's; none listed while no ray is cut (the cut off, or a
                // frame the layers were remade: its dispatch runs three times over); and the cut: where the first
                // place a ray may be cut at is, and how many lengths a list of lengths holds.
                [self.parity as f32, if cutting { self.long as f32 } else { 1e9 }, LONG_LIST as f32, targets.strip as f32],
                [if cutting { self.cut_at } else { 0.0 }, PIECES as f32, 0.0, 0.0],
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

    /// Whether rays long the frame before are cut into lengths this frame (the one dispatch's; never in a frame the
    /// layers were remade).
    fn cutting(&self) -> bool {
        self.one_join.is_some() && self.cut_at > 0.0 && !self.fresh
    }

    /// What the near eye's cover pass writes (`near_eye.rs`): the cover's picture, each layer's size in its pixels, the
    /// base's pixel's side in screen pixels, and a number that changes when they're made again. None where the
    /// layers aren't marched by the one dispatch, each half the one above down to whole screen pixels, or the rule
    /// is compiled out (`INCUS_FAR_COVER=0`).
    pub fn cover_writes(&self) -> Option<(&wgpu::TextureView, [(u32, u32); 3], u32, u64)> {
        let targets = self.targets.as_ref()?;
        targets.one.as_ref()?;
        let [a, b, c] = targets.layers.as_slice() else { return None };
        let side = a.factor as u32;
        let halved = side >= 4 && side % 4 == 0 && a.factor == side as f32 && b.factor * 2.0 == a.factor && c.factor * 4.0 == a.factor;
        (self.cover_on && halved).then(|| (&targets.cover, [a.size, b.size, c.size], side, self.targets_made))
    }

    /// The near eye's cover is this frame's: said once its pass is recorded, each frame it is, and the march skips
    /// what it covers. (`prepare` writes "not this frame's" with the frame's numbers, before.)
    pub fn covered(&self, queue: &wgpu::Queue) {
        if let Some((cameras, _)) = self.targets.as_ref().and_then(|targets| targets.one.as_ref()) {
            // (The base layer's row of "forward", its fourth number: `far.wgsl`'s `Camera`.)
            queue.write_buffer(cameras, 16 + 12, &1.0f32.to_le_bytes());
        }
    }

    /// The start pass: a cone a tile, where its rays can start.
    pub fn start(&self, encoder: &mut wgpu::CommandEncoder, timestamps: Option<wgpu::ComputePassTimestampWrites<'_>>) {
        let Some(targets) = &self.targets else { return };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the far start"), timestamp_writes: timestamps });
        // By the one dispatch the tiles' starts go into the copy of the distances the march reads this frame; in a
        // frame the layers were remade its march runs three times, reading each copy in turn, so into both.
        if let (Some(pipeline), Some(groups)) = (&self.one_start, &targets.one_starts) {
            pass.set_pipeline(pipeline);
            for copy in [self.parity, 1 - self.parity].into_iter().take(if self.fresh { 2 } else { 1 }) {
                pass.set_bind_group(0, &groups[copy], &[]);
                pass.dispatch_workgroups(targets.tiles.0.div_ceil(8), targets.tiles.1.div_ceil(8), 1);
            }
            return;
        }
        pass.set_pipeline(&self.start);
        pass.set_bind_group(0, &targets.layers[0].group, &[]);
        pass.dispatch_workgroups(targets.tiles.0.div_ceil(8), targets.tiles.1.div_ceil(8), 1);
    }

    /// Layer `k` of the pyramid (0 the base, 8 × 8). By the one dispatch, all the layers are marched in the first's
    /// pass, with the cut rays' lengths in a strip beside them; the second's pass is the cut's join; the third's is
    /// empty, kept for its place in the timing.
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
                // Once the textures are in: the loaded march, the one with cover by faces as the look asks (the
                // launch's, or the other).
                let loaded_march = if self.look.faces == self.faces_asked { self.one_loaded.as_ref() } else { self.one_loaded_other.as_ref().or(self.one_loaded.as_ref()) };
                pass.set_pipeline(if self.loaded() { loaded_march.unwrap_or(pipeline) } else { pipeline });
                let runs = if self.fresh { 3 } else { 1 };
                for run in 0..runs {
                    let copy = if (runs - 1 - run) % 2 == 0 { self.parity } else { 1 - self.parity };
                    pass.set_bind_group(0, &groups[copy], &[]);
                    pass.dispatch_workgroups(base.0 + middle.0 + fine.0 + if self.cutting() { targets.strip / 8 } else { 0 }, fine.1.max(middle.1).max(base.1), 1);
                }
                // The audit's variants, each in a pass of its own under its own name (so a trace gives each its
                // time): the base layer's groups, into scratch.
                if let Some(group) = &targets.audit {
                    drop(pass);
                    for (name, pipeline) in &self.audit {
                        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(name), timestamp_writes: None });
                        pass.set_pipeline(pipeline);
                        pass.set_bind_group(0, group, &[]);
                        pass.dispatch_workgroups(base.0, base.1, 1);
                    }
                }
            } else if k == 1 && self.cutting() && let Some(joins) = &self.one_join {
                // The join: a thread a place in the list, each cut ray's lengths made into its pixel.
                pass.set_pipeline(&joins[if self.loaded() { 1 } else { 0 }]);
                pass.set_bind_group(0, &groups[self.parity], &[]);
                pass.dispatch_workgroups(8, LONG_LIST / 64 / 8, 1);
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
            queue.write_buffer(&self.tally_counts[k], 0, &[0u8; 4 * TALLY]);
            queue.write_buffer(&targets.tally_parts[k], 0, &[layer.far_at, 0, layer.size.0, layer.size.1, self.long, 0, 0, 0].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("the longest ray"), timestamp_writes: None });
                pass.set_pipeline(&self.tally);
                pass.set_bind_group(0, &targets.tally_groups[k][self.parity], &[]);
                pass.dispatch_workgroups(layer.size.0.div_ceil(8), layer.size.1.div_ceil(8), 1);
            }
            encoder.copy_buffer_to_buffer(&self.tally_counts[k], 0, &self.tally_back, (4 * TALLY * k) as u64, 4 * TALLY as u64);
        }
        // Threads kept waiting, by the one dispatch: the march's grid as `layer` dispatched it, group by group.
        queue.write_buffer(&self.waiting_counts, 0, &[0u8; 4 * TALLY]);
        if let Some(groups) = &targets.waiting_groups {
            let across: Vec<u32> = targets.layers.iter().map(|layer| layer.size.0.div_ceil(8) * 8).collect();
            let down = targets.layers.iter().map(|layer| layer.size.1.div_ceil(8)).max().unwrap_or(1);
            let strip = if self.cutting() { targets.strip } else { 0 };
            let mut numbers: Vec<u32> = Vec::new();
            let mut starts = 0;
            for (layer, across) in targets.layers.iter().zip(&across) {
                numbers.extend([layer.size.0, layer.size.1, layer.far_at, starts]);
                starts += across;
            }
            let before = 1 - self.parity as u32;
            numbers.extend([starts, strip, LONG_LIST, PIECES]);
            numbers.extend([STACK, 0, (starts + strip) / 8, down]);
            numbers.extend([before * LONG_LIST * 3, PIECES_AT + before * PIECES, OWN_STACKS_AT, PIECE_STACKS_AT]);
            queue.write_buffer(&targets.waiting_numbers, 0, &numbers.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("threads kept waiting"), timestamp_writes: None });
            pass.set_pipeline(&self.waiting);
            pass.set_bind_group(0, &groups[self.parity], &[]);
            pass.dispatch_workgroups(((starts + strip) / 8).div_ceil(8), down.div_ceil(8), 1);
        }
        encoder.copy_buffer_to_buffer(&self.waiting_counts, 0, &self.tally_back, (4 * TALLY * targets.layers.len()) as u64, 4 * TALLY as u64);
        true
    }

    /// The frame that counted the longest rays is handed to the card: their reading asked for, never waited on.
    pub fn tally_sent(&self) {
        use std::sync::atomic::Ordering;
        self.tally_state.store(1, Ordering::Release);
        let state = self.tally_state.clone();
        self.tally_back.slice(..).map_async(wgpu::MapMode::Read, move |result| state.store(if result.is_ok() { 2 } else { 0 }, Ordering::Release));
    }

    /// The cut's store and its counts read back from the card, for a test.
    #[cfg(test)]
    fn cut_words(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (Vec<u32>, [u32; 4]) {
        let targets = self.targets.as_ref().expect("the far eye's targets");
        let size = CUT_WORDS as u64 * 4;
        let back = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the cut read back"), size: size + 16, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the cut read back") });
        encoder.copy_buffer_to_buffer(&targets.cut, 0, &back, 0, size);
        encoder.copy_buffer_to_buffer(&targets.long_rays, 0, &back, size, 16);
        queue.submit([encoder.finish()]);
        back.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let mut words: Vec<u32> = back.slice(..).get_mapped_range().unwrap().as_chunks::<4>().0.iter().map(|b| u32::from_le_bytes(*b)).collect();
        let counts = words.split_off(CUT_WORDS as usize);
        (words, [counts[0], counts[1], counts[2], counts[3]])
    }

    /// This frame's list of long rays read back, for a test: its count, then its pixels.
    #[cfg(test)]
    fn long_list(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u32> {
        let (words, counts) = self.cut_words(device, queue);
        let now = self.parity as u32;
        std::iter::once(counts[now as usize]).chain((0..LONG_LIST).map(|slot| words[((now * LONG_LIST + slot) * 3) as usize])).collect()
    }

    /// The rays marched as lengths this frame read back, for a test's look: each one's layer, its pixel, the steps
    /// each of its lengths took (its own thread's first), and each length's stack as numbers.
    #[cfg(test)]
    fn cut_lengths(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<(u32, (u32, u32), Vec<u32>, Vec<[f32; STACK as usize]>)> {
        let (words, counts) = self.cut_words(device, queue);
        let before = 1 - self.parity as u32;
        let stack = |at: u32| -> (u32, [f32; STACK as usize]) { ((words[at as usize + 9] >> 5) & 0x3FF_FFFF, std::array::from_fn(|n| f32::from_bits(words[at as usize + n]))) };
        (0..counts[before as usize].min(LONG_LIST))
            .map(|slot| {
                let at = ((before * LONG_LIST + slot) * 3) as usize;
                let (entry, cuts, first) = (words[at], words[at + 1], words[at + 2]);
                let stacks: Vec<_> = std::iter::once(stack(OWN_STACKS_AT + slot * STACK)).chain((0..cuts.count_ones()).map(|i| stack(PIECE_STACKS_AT + (first + i) * STACK))).collect();
                (entry >> 30, (entry & 0x7FFF, (entry >> 15) & 0x7FFF), stacks.iter().map(|s| s.0).collect(), stacks.iter().map(|s| s.1).collect())
            })
            .collect()
    }

    /// The longest rays, if a reading has come back: for each layer, the most steps any ray took and how many rays
    /// took over 256, 384 and 512; then how many took `LONG_STEPS` or more, how many of those are in the long rays'
    /// list, how many rays marched at all and their steps summed; then the longest chain (the most steps any one
    /// thread took: a cut ray's longest length), how many rays were cut, how many are in the list at all, and last
    /// how many steps made a ray long when they were counted. After the layers', a row for the threads kept waiting
    /// (`far_waiting.wgsl`): the layers' threads' steps taken, held by 32s, held by 64s and how many marched; then
    /// the strip's, the same four.
    pub fn tally_read(&self) -> Option<Vec<[u32; TALLY]>> {
        use std::sync::atomic::Ordering;
        if self.tally_state.load(Ordering::Acquire) != 2 {
            return None;
        }
        let counts = self.tally_back.slice(..).get_mapped_range().unwrap().as_chunks::<{ 4 * TALLY }>().0.iter().map(|b| std::array::from_fn(|k| if k == TALLY - 1 { self.long } else { u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]]) })).collect();
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

/// The one dispatch's source from the march's (`far.wgsl`): `far_all.wgsl` put in the place of one layer's outputs,
/// and every use of the camera made a read of this invocation's layer's row.
fn one_source(whole: &str) -> String {
    let (begin, end) = ("// ── One layer's outputs: begin ──", "// ── One layer's outputs: end ──");
    let (from, to) = (whole.find(begin).expect("the outputs' first mark"), whole.find(end).expect("the outputs' last mark"));
    format!("{}{}{}", &whole[..from], include_str!("far_all.wgsl"), &whole[to..]).replace("camera.", "cameras[layer_now].").replace("@group(0) @binding(0) var<uniform> camera: Camera;", "@group(0) @binding(0) var<uniform> cameras: array<Camera, 3>;")
}

/// The march's audit (`FarEye::audit`): cut-down versions of `far.wgsl`, each with one part taken out, by name. What
/// each says, read off Nsight's Shader Pipelines rows (# Reg, # Warp) and the trace's passes: what that part costs
/// the march in registers, and what a pass holding only the rest would fit. They draw wrong pictures on purpose
/// (into scratch). Each change must find its one place in the source: a variant that can't is a panic at start,
/// under the switch only, and `the_audit_s_variants_are_sound` says so first.
fn audit_variants(whole: &str, batch: u32) -> Vec<(&'static str, String)> {
    let whole = whole.replace("\r\n", "\n");
    // One place changed.
    fn once(text: &str, old: &str, new: &str) -> String {
        assert_eq!(text.matches(old).count(), 1, "the audit can't find its place in far.wgsl: {old:?}");
        text.replace(old, new)
    }
    // From one place up to another, changed.
    fn cut(text: &str, from: &str, to: &str, with: &str) -> String {
        assert_eq!(text.matches(from).count(), 1, "the audit can't find its place in far.wgsl: {from:?}");
        let a = text.find(from).unwrap();
        let b = a + text[a..].find(to).unwrap_or_else(|| panic!("the audit can't find its place in far.wgsl after {from:?}: {to:?}"));
        format!("{}{}{}", &text[..a], with, &text[b..])
    }
    const SKIP: &str = "        // Space with nothing in it is skipped whole.";
    const SAMPLE: &str = "        // Over the last stretch before the next level takes over";
    const AFTER: &str = "    // Down through the world's floor having gathered";
    const FINISH: &str = "    // Its shadow and sky light only where it shows";
    const VIEWS: &str = "    if (!LOADED && camera.reach.z > 1.5 && seen.level >= 0) {\n";
    const PUT: &str = "    // Kept with its alpha (premultiplied)";
    // A ray stops where it would first sample: the hops alone (what a pass that only crosses clear air would be).
    let hops = |t: &str| cut(t, SAMPLE, AFTER, "        first = t;\n        first_level = level;\n        through = 0.5;\n        break;\n    }\n");
    // No skipping: every step samples (what a pass that only samples would be).
    let samples = |t: &str| cut(t, SKIP, SAMPLE, "");
    // Over a hand-over, the level's own blend alone: no second level read and mixed.
    let no_handover = |t: &str| once(t, "        if (mixed > 0.0) {\n", "        if (false) {\n");
    // The ray's answer written as it is: no light, fog, sky's weight or views after the march.
    let no_finish = |t: &str| cut(t, FINISH, PUT, "    if (camera.layer.y > 0.5) {\n        put_worth(id.y * size.x + id.x, u32(round(worth_of(seen.distance, seen.level) * 255.0)));\n    }\n    var colour = seen.colour;\n");
    // Nothing kept of the cells over the world's floor.
    let no_floor = |t: &str| once(t, "                    if (p.z <= size * 1.5) {\n                        bottom = true;\n                        bottom_here = here;\n                        bottom_sun = here_sun;\n                        bottom_t = t;\n                        bottom_level = level;\n                    }\n", "");
    // The skip with two columns, its own and the one two levels coarser: no third row kept.
    let two_rows = |t: &str| {
        let t = once(t, "        for (var k = 0; k < 3 && in_air && !walking; k++) {\n", "        for (var k = 1; k < 3 && in_air && !walking; k++) {\n");
        let t = once(&t, "            keep_row(&row_far, far_level);\n", "");
        once(&t, "            high_far = top_in(row_far, i32(cc_far.x), i32(cc_far.y));\n", "")
    };
    // No rings: a level is its distance's, and nothing fades at a ring's edge.
    let no_rings = |t: &str| {
        let t = once(t, "        level = ring_level(level, p);\n", "");
        let t = once(&t, "    l = ring_level(l, p);\n", "");
        once(&t, "        if (level < FIRST) {\n            mixed = max(mixed, 1.0 - in_ring(level, p) / (RING_FADE * f32(1 << u32(level))));\n        }\n", "")
    };
    // Neither of the views (the steps', the levels' with its table of six tints).
    let no_views = |t: &str| cut(t, VIEWS, PUT, "");
    // Where the bricks can't answer, nothing: no blend of eight cells by the code.
    let no_code_blend = |t: &str| once(t, "    if (!LOADED && spot.kind == 1) {\n        return blend(spot.level, p);\n    }\n", "    if (spot.kind == 1) {\n        return empty_look(true);\n    }\n");
    let lean = |t: &str| no_code_blend(&no_views(&no_rings(&two_rows(&no_floor(&no_handover(t))))));
    // The second batch (`INCUS_FAR_AUDIT=2`), on the march once the textures are in (build 178: 167 registers, nothing
    // put by in memory, 12 groups): what the hand-over's second read holds at its fullest, and what each thing kept
    // the whole way holds on top (the floor's cells, the third row, the rings, what the finish needs), alone and with
    // the hand-over out, to find the fewest that bring it to the 128 at which 16 groups fit. Ceilings: each prices a
    // part's going, not what would stand in for it (build 178's lesson).
    if batch == 2 {
        let loaded = |t: &str| once(t, "const LOADED: bool = false;", "const LOADED: bool = true;");
        let hand = no_handover(&whole);
        return vec![
            ("audit2_whole", loaded(&whole)),
            ("audit2_no_handover", loaded(&hand)),
            ("audit2_no_floor", loaded(&no_floor(&whole))),
            ("audit2_two_rows", loaded(&two_rows(&whole))),
            ("audit2_no_rings", loaded(&no_rings(&whole))),
            ("audit2_no_finish", loaded(&no_finish(&whole))),
            ("audit2_hand_floor", loaded(&no_floor(&hand))),
            ("audit2_hand_rows", loaded(&two_rows(&hand))),
            ("audit2_hand_rings", loaded(&no_rings(&hand))),
            ("audit2_hand_finish", loaded(&no_finish(&hand))),
            ("audit2_hand_floor_rows", loaded(&two_rows(&no_floor(&hand)))),
            ("audit2_hand_floor_rings", loaded(&no_rings(&no_floor(&hand)))),
            ("audit2_lean4", loaded(&no_rings(&two_rows(&no_floor(&hand))))),
            ("audit2_lean5", loaded(&no_finish(&no_rings(&two_rows(&no_floor(&hand)))))),
        ];
    }
    vec![
        ("audit_whole", whole.clone()),
        ("audit_hops", hops(&whole)),
        ("audit_samples", samples(&whole)),
        ("audit_no_handover", no_handover(&whole)),
        ("audit_no_finish", no_finish(&whole)),
        ("audit_no_floor", no_floor(&whole)),
        ("audit_two_rows", two_rows(&whole)),
        ("audit_no_rings", no_rings(&whole)),
        ("audit_no_views", no_views(&whole)),
        ("audit_no_code_blend", no_code_blend(&whole)),
        ("audit_lean", lean(&whole)),
        ("audit_samples_lean", samples(&no_code_blend(&no_views(&no_floor(&no_handover(&whole)))))),
        ("audit_hops_lean", hops(&no_views(&no_floor(&whole)))),
    ]
}

/// For the Oracle's question of 2026-11-14 ("the longest ray's steps by kind"): the march's source with a ray's steps
/// counted by kind, if a picture test asks (`FAR_TEST_KINDS`: three kinds' numbers, drawn as the picture's red, green
/// and blue, a 1,024th a step). The kinds: 0 a region not summed passed whole; 1 a skip over a coarser column
/// (four levels up, or two), 2 over its own level's; 3 a back-up after a first touch; 4 a stride through air; 5 a step walking back in
/// through air; 6 a step in thin haze; 7 a step at a surface; 8 a sample mixed from two levels; 9 a blend fallen back
/// to the code; 10 a sample read from the 8s, 11 from the 16s; 12 every step. Each change must find its one place in
/// the source, or the variant isn't made and the test says so.
#[cfg(test)]
fn kinds_source(source: &str) -> String {
    // Or one of the audit's cut-down versions drawn as the march itself (`FAR_TEST_VARIANT`, its name): to see what a
    // part taken out does to the picture, against the whole march's.
    if let Ok(asked) = std::env::var("FAR_TEST_VARIANT") {
        return (1..=2).flat_map(|batch| audit_variants(source, batch)).find(|(name, _)| *name == asked).unwrap_or_else(|| panic!("FAR_TEST_VARIANT={asked} is none of the audit's versions")).1;
    }
    // Or its steps counted by the level they were taken at (`FAR_TEST_LEVELS`: three levels' numbers, drawn the same
    // way): a level is a doubling of distance, so this is where along a ray its steps lie (for sizing a long ray cut
    // into lengths, or spread over frames).
    if let Ok(asked) = std::env::var("FAR_TEST_LEVELS") {
        let show: Vec<usize> = asked.split(',').filter_map(|n| n.trim().parse().ok()).collect();
        assert_eq!(show.len(), 3, "FAR_TEST_LEVELS wants three levels");
        let changes: [(&str, String); 5] = [
            ("    ended_at: f32,\n}", "    ended_at: f32,\n    kinds: array<f32, 17>,\n}".into()),
            ("    var steps = 0;\n", "    var steps = 0;\n    var kinds: array<f32, 17>;\n".into()),
            ("        var skipped = false;\n", "        kinds[level] += 1.0;\n        var skipped = false;\n".into()),
            ("reached_level, in_air, t);", "reached_level, in_air, t, kinds);".into()),
            ("    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", format!("    colour = vec3<f32>(seen.kinds[{}], seen.kinds[{}], seen.kinds[{}]) / 1024.0;\n    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", show[0], show[1], show[2])),
        ];
        let mut made = source.replace("\r\n", "\n");
        for (old, new) in changes {
            assert_eq!(made.matches(old).count(), 1, "the steps-by-level variant can't find its place in far.wgsl: {old:?}");
            made = made.replace(old, &new);
        }
        return made;
    }
    let Ok(asked) = std::env::var("FAR_TEST_KINDS") else { return source.to_string() };
    let show: Vec<usize> = asked.split(',').filter_map(|n| n.trim().parse().ok()).collect();
    assert_eq!(show.len(), 3, "FAR_TEST_KINDS wants three kinds");
    let kind = |k: usize| if k == 12 { "seen.steps".to_string() } else { format!("seen.kinds[{k}]") };
    let changes: [(&str, String); 14] = [
        ("    ended_at: f32,\n}", "    ended_at: f32,\n    kinds: array<f32, 12>,\n}".into()),
        ("    var steps = 0;\n", "    var steps = 0;\n    var kinds: array<f32, 12>;\n".into()),
        ("                t = max(leave2(o.xy, d.xy, vec2<f32>(place.region), f32(1 << u32(REGION))), t) + 0.01;\n", "                t = max(leave2(o.xy, d.xy, vec2<f32>(place.region), f32(1 << u32(REGION))), t) + 0.01;\n                kinds[0] += 1.0;\n".into()),
        ("                skipped = true;\n", "                skipped = true;\n                kinds[1 + k / 2] += 1.0;\n".into()),
        ("        var over = 0.0;\n", "        if (level == 3) { kinds[10] += 1.0; } else if (level == 4) { kinds[11] += 1.0; }\n        var over = 0.0;\n".into()),
        ("        if (mixed > 0.0) {\n", "        if (mixed > 0.0) {\n            kinds[8] += 1.0;\n".into()),
        ("            } else {\n                look = look_at(here, p);\n", "            } else {\n                kinds[9] += 1.0;\n                look = look_at(here, p);\n".into()),
        ("            look = far_blend_kept(&kept, place, level, p);\n", "            let spot_k = spot_kept(&kept, place, level, p);\n            if (spot_k.kind == 1) { kinds[9] += 1.0; }\n            look = look_at(spot_k, p);\n".into()),
        ("            walk_to = t;\n", "            walk_to = t;\n            kinds[3] += 1.0;\n".into()),
        ("        if (thick > 0.0) {\n            in_air = false;\n            walking = true;\n", "        if (thick > 0.0) {\n            in_air = false;\n            walking = true;\n            if (thick > 0.02) { kinds[7] += 1.0; } else { kinds[6] += 1.0; }\n".into()),
        ("        } else if (walking && t < walk_to) {\n            in_air = false;\n", "        } else if (walking && t < walk_to) {\n            in_air = false;\n            kinds[5] += 1.0;\n".into()),
        ("        } else {\n            last_air = t;\n            walking = false;\n", "        } else {\n            last_air = t;\n            walking = false;\n            kinds[4] += 1.0;\n".into()),
        ("reached_level, in_air, t);", "reached_level, in_air, t, kinds);".into()),
        ("    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", format!("    colour = vec3<f32>({}, {}, {}) / 1024.0;\n    put_picture(vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour,", kind(show[0]), kind(show[1]), kind(show[2]))),
    ];
    let mut made = source.replace("\r\n", "\n");
    for (old, new) in changes {
        assert_eq!(made.matches(old).count(), 1, "the steps-by-kind variant can't find its place in far.wgsl: {old:?}");
        made = made.replace(old, &new);
    }
    made
}

#[cfg(test)]
mod tests {
    /// The audit's variants each find their places in the march and are sound shaders (parsed and checked without the
    /// card), so `INCUS_FAR_AUDIT` can't fail at the game's start.
    #[test]
    fn the_audit_s_variants_are_sound() {
        let variants: Vec<_> = (1..=2).flat_map(|batch| super::audit_variants(include_str!("far.wgsl"), batch)).collect();
        assert!(variants.len() >= 20);
        for (name, text) in variants {
            let source = super::one_source(&text).replace("fn march_all(", &format!("fn {name}("));
            let module = wgpu::naga::front::wgsl::parse_str(&source).unwrap_or_else(|error| panic!("{name} doesn't parse: {}", error.emit_to_string(&source)));
            wgpu::naga::valid::Validator::new(wgpu::naga::valid::ValidationFlags::all(), wgpu::naga::valid::Capabilities::all()).validate(&module).unwrap_or_else(|error| panic!("{name} isn't sound: {error:?}"));
            assert!(module.entry_points.iter().any(|entry| entry.name == name), "{name} has no entry of its name");
        }
    }

    use super::*;
    use crate::doorway::Doorway;
    use crate::far::{FarColours, region, regions_of};

    /// Each shader's text as the card's compiler is given it, kept in a folder (`FAR_TEST_SHADER_TEXT`), a file a
    /// program (a name met again gets a number): for the march's size by its parts, taken off the screen
    /// (`journal/docs/march-by-parts.md`). The game's own way to the same text is `INCUS_SHADER_TEXT`, into the
    /// session's log.
    struct ShaderText {
        folder: std::path::PathBuf,
        seen: std::sync::Mutex<std::collections::HashMap<String, u32>>,
    }

    impl log::Log for ShaderText {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.target().starts_with("wgpu_hal")
        }

        fn log(&self, record: &log::Record) {
            let text = record.args().to_string();
            let Some(rest) = text.strip_prefix("Naga generated shader for \"") else { return };
            let Some((name, rest)) = rest.split_once("\" at ") else { return };
            let Some((stage, source)) = rest.split_once(":\n") else { return };
            let mut seen = self.seen.lock().unwrap();
            let met = seen.entry(name.to_string()).or_insert(0);
            let file = if *met == 0 { format!("{name}.{}.hlsl", stage.to_lowercase()) } else { format!("{name}_{met}.{}.hlsl", stage.to_lowercase()) };
            *met += 1;
            let _ = std::fs::create_dir_all(&self.folder);
            let _ = std::fs::write(self.folder.join(file), source);
        }

        fn flush(&self) {}
    }

    /// The far eye draws (plan 2.2, step 2a): its passes made and run with no error from the card, off the screen, from
    /// the spawn looking north, with the regions round the spawn on the card: the picture has ground below and sky above.
    #[test]
    fn the_far_eye_draws_ground_and_sky() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        // The shaders' text kept, if asked (`FAR_TEST_SHADER_TEXT`: a folder), and made as the game makes it: for
        // the game's own compiler (`card.rs`), which the text depends on.
        if let Some(folder) = std::env::var_os("FAR_TEST_SHADER_TEXT") {
            let kept: &'static ShaderText = Box::leak(Box::new(ShaderText { folder: folder.into(), seen: Default::default() }));
            if log::set_logger(kept).is_ok() {
                log::set_max_level(log::LevelFilter::Debug);
            }
            if let Some(dxc) = crate::card::windows_sdk_dxc() {
                descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::DynamicDxc { dxc_path: dxc };
            }
        }
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).expect("a graphics card");
        let largest = adapter.limits().max_buffer_size.min(adapter.limits().max_storage_buffer_binding_size).min((2 << 30) - 1);
        let required_limits = wgpu::Limits { max_buffer_size: largest, max_storage_buffer_binding_size: largest, ..wgpu::Limits::default() };
        // (The near eye's draws are read from a list: `card.rs` asks the same of the game's card.)
        let required_features = adapter.features() & wgpu::Features::INDIRECT_FIRST_INSTANCE;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("a test"), required_features, required_limits, ..Default::default() })).expect("the card's device");
        let errors = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let marks = crate::marks::load().unwrap();
        // From the spawn, or another mark if asked (`FAR_TEST_MARK`, its name, for a look).
        let mark = std::env::var("FAR_TEST_MARK").ok().and_then(|name| marks.iter().find(|m| m.name == name)).unwrap_or(&marks[0]);
        let mut camera = Camera::new();
        // The field of view, if asked (`FAR_TEST_FOV`, degrees up and down).
        if let Some(fov) = std::env::var("FAR_TEST_FOV").ok().and_then(|fov| fov.parse::<f32>().ok()) {
            camera.fov = fov;
        }
        camera.position = crate::marks::eye(&doorway, mark).unwrap();
        camera.facing = mark.facing;
        camera.pitch = mark.pitch;
        // Or from anywhere, if asked (`FAR_TEST_EYE`: north, east, up, facing, pitch, as the Camera page shows them;
        // the mark still says which regions are loaded), for a look at the very view David sent.
        let eye_asked: Option<Vec<f32>> = std::env::var("FAR_TEST_EYE").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect());
        if let Some(v) = eye_asked.as_ref().filter(|v| v.len() == 5) {
            (camera.position.x, camera.position.y, camera.position.z, camera.facing, camera.pitch) = (v[0], v[1], v[2], v[3], v[4]);
        }
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
        // The rings, as the game starts them (as many as `INCUS_FAR_RING` says: 0 none).
        let rings = crate::far_ring::rings_asked().into_iter().map(|level| crate::far_ring::Ring::start(crate::doorway::test_world(), store.levels[crate::bricks::row(level)], store.kept.side()).unwrap()).collect();
        let mut eye = FarEye::new(&device, format, store, lane, rings);
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
        // The far light made first too (read from the cell), the ring round the eye whole on the card, and the height
        // table's true numbers in.
        while !eye.lit() || !eye.ring_settled() || eye.tops_made.is_none() {
            eye.put(&device, &queue, at);
            queue.submit([]);
            let _ = device.poll(wgpu::PollType::Poll);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        // The ring moved first, if asked (`FAR_TEST_MOVE`: blocks north, east): brought to a place that far from the
        // eye, then back, so the picture is drawn from a ring that has moved. It must be the picture of a ring made here.
        let moved: Vec<f32> = std::env::var("FAR_TEST_MOVE").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        if moved.len() == 2 {
            for place in [[at[0] - moved[0], at[1] - moved[1]], at] {
                loop {
                    eye.put(&device, &queue, place);
                    queue.submit([]);
                    let _ = device.poll(wgpu::PollType::Poll);
                    if eye.ring_settled() && eye.store.behind == 0 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
        // Cover by faces by the look's switch, if asked (`FAR_TEST_FACES`: 0 off, 1 on; 2 on till the last frame,
        // which is drawn off: the switch flipped while it runs): the march's other program where it differs from the
        // launch's.
        let faces_test = std::env::var("FAR_TEST_FACES").ok().and_then(|f| f.parse::<u32>().ok());
        if let Some(asked) = faces_test {
            assert!(eye.one_loaded_other.is_some(), "the march's other program is made");
            eye.look.faces = asked != 0;
        }
        // The mark's own changes (David's build; a scatter at `FAR_TEST_SCATTER`, its count), made as the game makes them
        // on going to the mark, and told to the far eye (and the near eye's, below) as the game tells them; the far eye
        // given them whole, its light and height table and rings too, before a frame is drawn.
        let mut changed = Doorway::open(&crate::doorway::test_world()).unwrap();
        if let Some(name) = &mark.changes {
            let made = crate::marks::place(&mut changed, &std::fs::read_to_string(crate::session::project_folder().join(name)).unwrap());
            eprintln!("The mark's changes: {} whole blocks, {} bars, {} refused", made.blocks, made.bars, made.refused);
        }
        if let (Some(scatter), Some(count)) = (&mark.scatter, std::env::var("FAR_TEST_SCATTER").ok().and_then(|count| count.parse::<u32>().ok())) {
            crate::marks::scatter(&mut changed, mark, scatter, count).unwrap();
        }
        if !changed.changes().is_empty() && std::env::var("FAR_TEST_PLACED").as_deref() != Ok("0") {
            eye.changed(changed.changes());
            let began = std::time::Instant::now();
            loop {
                eye.put(&device, &queue, at);
                queue.submit([]);
                let _ = device.poll(wgpu::PollType::Poll);
                if eye.changes_settled() && eye.lit() && eye.tops_made.is_some() && eye.ring_settled() {
                    break;
                }
                assert!(began.elapsed().as_secs() < 300, "the far eye takes what's placed in five minutes");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            eprintln!("The far eye took what's placed in {:.2} s", began.elapsed().as_secs_f32());
            // Taken back again, if asked (`FAR_TEST_TAKE_BACK`): the picture should be the bare mark's.
            if std::env::var_os("FAR_TEST_TAKE_BACK").is_some() {
                eye.changed(&std::collections::HashMap::new());
                let began = std::time::Instant::now();
                loop {
                    eye.put(&device, &queue, at);
                    queue.submit([]);
                    let _ = device.poll(wgpu::PollType::Poll);
                    if eye.changes_settled() && eye.lit() && eye.tops_made.is_some() && eye.ring_settled() {
                        break;
                    }
                    assert!(began.elapsed().as_secs() < 300, "the far eye takes what's placed back in five minutes");
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                eprintln!("The far eye took what's placed back in {:.2} s", began.elapsed().as_secs_f32());
            }
        }
        // The near eye over the far picture, if asked (`FAR_TEST_NEAR`: its render radius, chunks): every chunk of the
        // radius in before a frame is drawn.
        let mut near = std::env::var("FAR_TEST_NEAR").ok().and_then(|radius| radius.parse::<f32>().ok()).map(|radius| {
            let mut near = crate::near_eye::NearEye::new(&device, &queue, format, crate::doorway::test_world(), &colours.as_light(), &crate::textures::load(&doorway, &colours)).unwrap();
            near.look.radius = radius;
            // The x-fader, if asked (`FAR_TEST_FADER`: its width in blocks, then its curve).
            let fader: Vec<f32> = std::env::var("FAR_TEST_FADER").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
            if fader.len() == 2 {
                (near.look.fader_width, near.look.fader_curve) = (fader[0], fader[1]);
            }
            // The near eye's largest squares, if asked (`FAR_TEST_NEAR_PIXEL`: 1, 2 or 4), and the detail curve's near
            // number (`FAR_TEST_DETAIL`, blocks).
            if let Some(pixel) = std::env::var("FAR_TEST_NEAR_PIXEL").ok().and_then(|pixel| pixel.parse::<u32>().ok()) {
                near.look.pixel = pixel;
            }
            if let Some(detail) = std::env::var("FAR_TEST_DETAIL").ok().and_then(|detail| detail.parse::<f32>().ok()) {
                near.look.detail = detail;
            }
            // (And the stipple's width, `FAR_TEST_STIPPLE`.)
            if let Some(stipple) = std::env::var("FAR_TEST_STIPPLE").ok().and_then(|stipple| stipple.parse::<f32>().ok()) {
                near.look.stipple = stipple;
            }
            // (And the pictures' sharpness in the squares, `FAR_TEST_SHARP`.)
            if let Some(sharp) = std::env::var("FAR_TEST_SHARP").ok().and_then(|sharp| sharp.parse::<f32>().ok()) {
                near.look.sharp = sharp;
            }
            // (And the placed lights' flicker, `FAR_TEST_FLICKER`: 0 for pictures that are to be the same twice.)
            if let Some(flicker) = std::env::var("FAR_TEST_FLICKER").ok().and_then(|flicker| flicker.parse::<f32>().ok()) {
                near.look.flicker = flicker;
            }
            // (And a light's step, `FAR_TEST_STEP`.)
            if let Some(step) = std::env::var("FAR_TEST_STEP").ok().and_then(|step| step.parse::<f32>().ok()) {
                near.look.step = step;
            }
            // (`FAR_TEST_GROUPS=0`: every group of a chunk's faces listed, the card's own dropping of faces turned away
            // left as it is: to see the listing's rule alone change nothing.)
            if std::env::var("FAR_TEST_GROUPS").as_deref() == Ok("0") {
                near.turned = false;
            }
            near.changed(changed.changes());
            let began = std::time::Instant::now();
            loop {
                near.put(&device, &queue, at, &mut eye).unwrap();
                queue.submit([]);
                if near.whole(at) {
                    break;
                }
                assert!(began.elapsed().as_secs() < 600, "the near eye's radius fills in ten minutes");
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            eprintln!("The near eye filled in {:.2} s: {:?}", began.elapsed().as_secs_f32(), near.counts);
            near
        });
        // The sky's weight to draw with, if asked (`FAR_TEST_SKY_WEIGHT`, for a look at the skyline).
        if let Some(weight) = std::env::var("FAR_TEST_SKY_WEIGHT").ok().and_then(|w| w.parse().ok()) {
            eye.look.sky_weight = weight;
        }
        // Where the rays' steps begin, if asked (`FAR_TEST_START`: the pattern, then the weave's bits, the blue noise
        // in the weave and the spread, as many as given).
        let start_asked: Vec<f32> = std::env::var("FAR_TEST_START").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        if let Some(&v) = start_asked.first() {
            eye.look.start_pattern = v as u32;
        }
        if let Some(&v) = start_asked.get(1) {
            eye.look.weave_bits = v as u32;
        }
        if let Some(&v) = start_asked.get(2) {
            eye.look.start_blue = v;
        }
        if let Some(&v) = start_asked.get(3) {
            eye.look.start_spread = v;
        }
        // The picture's size (640 x 400, or `FAR_TEST_SCREEN`: across, down).
        let screen_asked: Vec<u32> = std::env::var("FAR_TEST_SCREEN").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        let screen = if screen_asked.len() == 2 { (screen_asked[0], screen_asked[1]) } else { (640u32, 400u32) };
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
        // (The lantern lit, if asked, `FAR_TEST_LANTERN`: its light waited for from its thread, so the picture has it.)
        if let (Some(near), true) = (&mut near, std::env::var_os("FAR_TEST_LANTERN").is_some()) {
            near.look.lantern = true;
        }
        // A frame, or as many as asked (`FAR_TEST_FRAMES`; by the one dispatch a finer layer's lead is the frame
        // before's, and the first frame makes its own); the last one's picture read back.
        let frames = std::env::var("FAR_TEST_FRAMES").ok().and_then(|n| n.parse::<u32>().ok()).unwrap_or(1).max(1);
        // The near eye before the march, as the game's frame goes: its faces and its water into its own picture, then
        // its cover for the far eye.
        let near_first = |near: &mut crate::near_eye::NearEye, eye: &FarEye, encoder: &mut wgpu::CommandEncoder, camera: &Camera| {
            assert!(near.prepare(&device, &queue, screen, eye, camera), "the near eye has what it reads of the far eye's");
            // (The lantern's light comes from its thread: waited for here, so a picture or a timed frame has it.)
            for _ in 0..5000 {
                if near.lantern_settled() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
                near.prepare(&device, &queue, screen, eye, camera);
            }
            assert!(near.lantern_settled(), "the lantern's light came from its thread");
            near.draw_faces(encoder, None);
            near.draw_water(encoder, None);
            if near.cover(&device, &queue, encoder, eye, None) {
                eye.covered(&queue);
            }
        };
        for _ in 1..frames {
            if let Some(near) = &mut near {
                near.put(&device, &queue, at, &mut eye).unwrap();
            }
            eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
            eye.start(&mut encoder, None);
            if let Some(near) = &mut near {
                near_first(near, &eye, &mut encoder, &camera);
            }
            for k in 0..LAYERS.len() {
                eye.layer(k, &mut encoder, None);
            }
            queue.submit([encoder.finish()]);
        }
        // What the far eye holds, if asked (`FAR_TEST_STATE`): its bricks, its rings, its memory.
        if std::env::var_os("FAR_TEST_STATE").is_some() {
            eprintln!("The far eye holds: {}", eye.state());
        }
        // The rings moved as a walk moves them, if asked (`FAR_TEST_RING_MOVE`: blocks north, each of four moves): how
        // long each takes to be on the card whole; then back where they were.
        if let Some(step) = std::env::var("FAR_TEST_RING_MOVE").ok().and_then(|step| step.parse::<f32>().ok()) {
            for k in [1.0, 2.0, 3.0, 4.0, 0.0] {
                let there = [at[0] + step * k, at[1]];
                let began = std::time::Instant::now();
                loop {
                    eye.put(&device, &queue, there);
                    queue.submit([]);
                    let _ = device.poll(wgpu::PollType::Poll);
                    if eye.ring_settled() {
                        break;
                    }
                    assert!(began.elapsed().as_secs() < 300, "the rings follow a move in five minutes");
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                eprintln!("The rings moved to {:.0} blocks north of the start in {:.2} s", step * k, began.elapsed().as_secs_f32());
            }
        }
        // The near eye's listing timed, if asked (`FAR_TEST_LISTING`): with every group listed, with those turned away
        // left out, and again, in this one run.
        if let (Some(near), true) = (&mut near, std::env::var_os("FAR_TEST_LISTING").is_some()) {
            let as_it_was = near.turned;
            for on in [false, true, false, true, false] {
                near.turned = on;
                let began = std::time::Instant::now();
                for _ in 0..400 {
                    near.prepare(&device, &queue, screen, &eye, &camera);
                }
                let each = began.elapsed().as_secs_f64() * 1e6 / 400.0;
                queue.submit([]);
                // And whole frames (the march and the near eye's passes, as the game's frame goes): the handing of
                // each to the card timed on the processor, and all of them until the card has finished the last.
                device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
                let (frames_timed, mut handing) = (300, 0.0f64);
                let whole = std::time::Instant::now();
                for _ in 0..frames_timed {
                    eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
                    eye.start(&mut encoder, None);
                    near_first(near, &eye, &mut encoder, &camera);
                    for k in 0..LAYERS.len() {
                        eye.layer(k, &mut encoder, None);
                    }
                    let handed = std::time::Instant::now();
                    queue.submit([encoder.finish()]);
                    handing += handed.elapsed().as_secs_f64();
                }
                device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
                // And the near eye's own two passes alone (its faces and its water into its own picture), to the
                // card's finish: at two sizes of picture the difference is its pixels' work, its corners' the same.
                let alone = std::time::Instant::now();
                for _ in 0..frames_timed {
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the near eye alone") });
                    near.prepare(&device, &queue, screen, &eye, &camera);
                    near.draw_faces(&mut encoder, None);
                    near.draw_water(&mut encoder, None);
                    queue.submit([encoder.finish()]);
                }
                device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
                eprintln!("The near eye's passes alone at {} x {}, groups turned away {}: {:.0} millionths of a second a frame", screen.0, screen.1, if on { "left out" } else { "listed" }, alone.elapsed().as_secs_f64() * 1e6 / frames_timed as f64);
                eprintln!("Whole frames, groups turned away {}: handing a frame over {:.0} millionths of a second on the processor; a frame {:.0} millionths, start to the card's finish", if on { "left out" } else { "listed" }, handing * 1e6 / frames_timed as f64, whole.elapsed().as_secs_f64() * 1e6 / frames_timed as f64);
                eprintln!("The listing, groups turned away {}: {each:.0} millionths of a second a frame; {} chunks, {} draws, {} faces sent of {} kept", if on { "left out" } else { "listed" }, near.counts.drawn, near.counts.draws, near.counts.faces_drawn, near.counts.faces);
            }
            near.turned = as_it_was;
        }
        // The near picture's sizes priced in this one run, if asked (`FAR_TEST_PIXELS`): its pixel 1, 2, 1, 4, 1 screen
        // pixels a side; the near eye's passes alone, and whole frames, each to the card's finish.
        if let (Some(near), true) = (&mut near, std::env::var_os("FAR_TEST_PIXELS").is_some()) {
            let as_it_was = near.look.pixel;
            // (`FAR_TEST_PIXELS=stipple`: the stipple's widths priced instead, at the largest squares as set.)
            // (`FAR_TEST_PIXELS=sharp`: the pictures' sharpness in the squares, 0 and 1 by turns, the same way.)
            let stipples = std::env::var("FAR_TEST_PIXELS").as_deref() == Ok("stipple");
            let sharps = std::env::var("FAR_TEST_PIXELS").as_deref() == Ok("sharp");
            // (`FAR_TEST_PIXELS=lantern`: the lantern out and lit by turns, its light waited for each time.)
            let lanterns = std::env::var("FAR_TEST_PIXELS").as_deref() == Ok("lantern");
            let (stipple_was, sharp_was, lantern_was) = (near.look.stipple, near.look.sharp, near.look.lantern);
            let turns: [(u32, f32, f32, bool); 5] = if stipples {
                [0.0, 0.25, 0.0, 0.5, 0.0].map(|width| (as_it_was, width, sharp_was, lantern_was))
            } else if sharps {
                [0.0, 1.0, 0.0, 1.0, 0.0].map(|sharp| (as_it_was, stipple_was, sharp, lantern_was))
            } else if lanterns {
                [false, true, false, true, false].map(|lit| (as_it_was, stipple_was, sharp_was, lit))
            } else {
                [1, 4, 1, 2, 1].map(|pixel| (pixel, stipple_was, sharp_was, lantern_was))
            };
            for (pixel, stipple, sharp, lantern) in turns {
                near.look.pixel = pixel;
                near.look.stipple = stipple;
                near.look.sharp = sharp;
                near.look.lantern = lantern;
                for _ in 0..5000 {
                    near.prepare(&device, &queue, screen, &eye, &camera);
                    if near.lantern_settled() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                let frames_timed = 300;
                device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
                let alone = std::time::Instant::now();
                for _ in 0..frames_timed {
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the near eye alone") });
                    near.prepare(&device, &queue, screen, &eye, &camera);
                    near.draw_faces(&mut encoder, None);
                    near.draw_water(&mut encoder, None);
                    queue.submit([encoder.finish()]);
                }
                device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
                let alone = alone.elapsed().as_secs_f64() * 1e6 / frames_timed as f64;
                let whole = std::time::Instant::now();
                for _ in 0..frames_timed {
                    eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
                    eye.start(&mut encoder, None);
                    near_first(near, &eye, &mut encoder, &camera);
                    for k in 0..LAYERS.len() {
                        eye.layer(k, &mut encoder, None);
                    }
                    queue.submit([encoder.finish()]);
                }
                device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
                eprintln!("Near pixel size {pixel}, stipple {stipple}, sharpness {sharp}, lantern {lantern}: the near passes alone {alone:.0} millionths of a second a frame; a whole frame {:.0}", whole.elapsed().as_secs_f64() * 1e6 / frames_timed as f64);
            }
            near.look.pixel = as_it_was;
            near.look.stipple = stipple_was;
            near.look.sharp = sharp_was;
            near.look.lantern = lantern_was;
        }
        // The last frame drawn with the eye turned, if asked (`FAR_TEST_TURN`: degrees of facing, then of pitch): a
        // moving view's first frame, where every ray is led by another ray's frame before (the cut's lengths begin
        // as the ray at that pixel stood; the finer layers follow the coarser's).
        let turned: Vec<f32> = std::env::var("FAR_TEST_TURN").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        camera.facing += turned.first().copied().unwrap_or(0.0);
        camera.pitch += turned.get(1).copied().unwrap_or(0.0);
        if faces_test == Some(2) {
            eye.look.faces = false;
        }
        if let Some(near) = &mut near {
            near.put(&device, &queue, at, &mut eye).unwrap();
        }
        eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
        eye.start(&mut encoder, None);
        if let Some(near) = &mut near {
            near_first(near, &eye, &mut encoder, &camera);
        }
        for k in 0..LAYERS.len() {
            eye.layer(k, &mut encoder, None);
        }
        eye.bring_up(&mut encoder, &view, None);
        // (`FAR_TEST_NEAR_ALONE`: the near eye's picture over plain magenta, not over the far picture, so a pixel the
        // near eye left out can be counted.)
        if std::env::var_os("FAR_TEST_NEAR_ALONE").is_some() {
            let magenta = wgpu::Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
            let attachment = wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(magenta), store: wgpu::StoreOp::Store } };
            drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("magenta under the near picture"), color_attachments: &[Some(attachment)], depth_stencil_attachment: None, ..Default::default() }));
        }
        if let Some(near) = &mut near {
            near.join(&mut encoder, &view, None);
            eprintln!("The near eye drew: {:?}", near.counts);
        }
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
        // The long rays' list and the cut: two frames more (no ray is listed in the frame the layers were made; the
        // frame after lists its long rays; the frame after that cuts them into lengths and joins them), then every
        // layer's rays counted and the list read back: it holds each long ray, once, and every listed ray was cut.
        let errors = device.push_error_scope(wgpu::ErrorFilter::Validation);
        for last in [false, true] {
            eye.prepare(&device, &queue, screen, &camera, doorway.sea_level as f32);
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far eye's test") });
            eye.start(&mut encoder, None);
            if let Some(near) = &mut near {
                near_first(near, &eye, &mut encoder, &camera);
            }
            for k in 0..LAYERS.len() {
                eye.layer(k, &mut encoder, None);
            }
            if last {
                assert!(eye.tally(&queue, &mut encoder), "the rays are counted");
            }
            queue.submit([encoder.finish()]);
        }
        eye.tally_sent();
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let error = pollster::block_on(errors.pop());
        assert!(error.is_none(), "the card refused the cut's work: {error:?}");
        let counts = eye.tally_read().expect("the rays' counts came back");
        eprintln!("The rays: {}", counts.iter().zip(["8 x 8", "4 x 4", "2 x 2"]).map(|(c, name)| format!("the {name} {} marched, {} steps, the longest {} (the longest chain {}), {} of {} steps or more ({} listed), {} cut, {} in the list", c[6], c[7], c[0], c[8], c[4], c[11], c[5], c[9], c[10])).collect::<Vec<_>>().join("; "));
        if eye.one.is_some() {
            // Threads kept waiting: every step a ray took is some thread's, the layers' or the strip's. (The threads
            // may have taken more: a length that marched behind the ray's cover isn't counted as the ray's.)
            let waiting = counts[LAYERS.len()];
            eprintln!("Threads kept waiting: the layers' {} steps taken, {} held by 32s, {} by 64s, {} threads marched; the strip's {} steps taken, {} held by 32s, {} by 64s, {} threads marched", waiting[0], waiting[1], waiting[2], waiting[3], waiting[4], waiting[5], waiting[6], waiting[7]);
            assert!(waiting[0] + waiting[4] >= counts[..LAYERS.len()].iter().map(|c| c[7]).sum::<u32>(), "the rays' steps are all some thread's");
            assert!(waiting[1] >= waiting[0] && waiting[2] >= waiting[1] && waiting[5] >= waiting[4] && waiting[6] >= waiting[5], "a group is held at least as long as its threads are busy");
        }
        if eye.one.is_some() && eye.cut_at > 0.0 {
            let list = eye.long_list(&device, &queue);
            let layers = &counts[..LAYERS.len()];
            let (long, listed, cut, in_list): (u32, u32, u32, u32) = (layers.iter().map(|c| c[4]).sum(), layers.iter().map(|c| c[5]).sum(), layers.iter().map(|c| c[9]).sum(), layers.iter().map(|c| c[10]).sum());
            assert!(list[0] >= long, "the list counts the long rays (and the cut rays it keeps under that)");
            if list[0] <= LONG_LIST {
                assert_eq!(listed, long, "every long ray carries its place in the list");
                assert_eq!(in_list, list[0], "as many rays carry a place as the list counts");
                let mut entries = list[1..1 + list[0] as usize].to_vec();
                entries.sort_unstable();
                entries.dedup();
                assert_eq!(entries.len() as u32, list[0], "and each is in it once");
                // The cut rays' lengths' steps, for a look (`FAR_TEST_LENGTHS`): the most any ray took in each length,
                // the lengths' steps of the rays with the longest chains, and how the chains are spread.
                if std::env::var_os("FAR_TEST_LENGTHS").is_some() {
                    let lengths = eye.cut_lengths(&device, &queue);
                    let pieces: usize = lengths.iter().map(|ray| ray.2.len() - 1).sum();
                    let mut how_many = [0usize; 17];
                    for ray in &lengths {
                        how_many[(ray.2.len() - 1).min(16)] += 1;
                    }
                    eprintln!("The cut: {} rays listed, {pieces} lengths in the strip; rays by how many lengths beyond their first: {how_many:?}", lengths.len());
                    let mut by_chain: Vec<_> = lengths.iter().collect();
                    by_chain.sort_by_key(|ray| std::cmp::Reverse(*ray.2.iter().max().unwrap_or(&0)));
                    for ray in by_chain.iter().take(8) {
                        eprintln!("  layer {} pixel {:?}: {:?} (in all {})", ray.0, ray.1, ray.2, ray.2.iter().sum::<u32>());
                    }
                    // One pixel's lengths whole, if asked (`FAR_TEST_LENGTHS`: its layer, across, down): what each
                    // gathered, the sun's part, what it let through, its first surface, its first touch.
                    let asked: Vec<u32> = std::env::var("FAR_TEST_LENGTHS").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
                    for asked in asked.chunks_exact(3) {
                        match lengths.iter().find(|ray| ray.0 == asked[0] && ray.1 == (asked[1], asked[2])) {
                            Some(ray) => {
                                eprintln!("  layer {} pixel ({}, {}):", asked[0], asked[1], asked[2]);
                                for (k, (steps, s)) in ray.2.iter().zip(&ray.3).enumerate().filter(|(_, (steps, _))| **steps > 0) {
                                    eprintln!("  length {k}: {steps} steps, gathered {:.3} {:.3} {:.3}, the sun's {:.3} {:.3} {:.3}, ended letting through {:.4}, reached {:.1}, began letting through {:.4}, from {:.0} to {:.0}", s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], s[10], s[11], s[12]);
                                }
                            }
                            None => eprintln!("  layer {} pixel ({}, {}) wasn't cut", asked[0], asked[1], asked[2]),
                        }
                    }
                    let chains: Vec<u32> = by_chain.iter().map(|ray| *ray.2.iter().max().unwrap_or(&0)).collect();
                    let at = |share: f32| chains.get(((chains.len() as f32 * share) as usize).min(chains.len().saturating_sub(1))).copied().unwrap_or(0);
                    eprintln!("  chains: the longest {}, one in a hundred over {}, one in ten over {}, half over {}", at(0.0), at(0.01), at(0.1), at(0.5));
                }
                if eye.cutting() {
                    assert!(long == 0 || cut > 0, "rays long the frame before were cut into lengths");
                }
            }
        }
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
            let _ = std::fs::write(std::path::Path::new(&folder).join(format!("far eye {} {}x{} weight {}{} blend {} detail {}", mark.name, screen.0, screen.1, eye.look.sky_weight, if std::env::var_os("INCUS_SIGHT_NO_SHADOWS").is_some() { " unlit" } else { "" }, eye.look.blend, eye.look.detail) + &std::env::var("FAR_TEST_START").map_or(String::new(), |s| format!(" start {s}")) + &std::env::var("FAR_TEST_KINDS").map_or(String::new(), |s| format!(" kinds {s}")) + &std::env::var("FAR_TEST_LEVELS").map_or(String::new(), |s| format!(" levels {s}")) + &std::env::var("FAR_TEST_VARIANT").map_or(String::new(), |s| format!(" variant {s}")) + if eye_asked.is_some() { " eye" } else { "" } + ".rgba"), raw);
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

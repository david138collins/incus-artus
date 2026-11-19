// The near eye's faces (plan 2.3, build 195): the chunks' merged faces drawn as triangles, lit as the far eye lights.
// `near_eye.rs` puts the far eye's own text in front of this when the program is built (`far.wgsl`'s `Camera`, its
// levels' table, its light's rule and its light store's reading), so both eyes light by one text.

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> levels: array<vec4<i32>, 48>;
@group(0) @binding(2) var t_shadow: texture_2d<f32>;
@group(0) @binding(3) var t_sky: texture_2d<f32>;
@group(0) @binding(4) var soft: sampler;
// Each block's colour from far, as light: its top's, then its sides' (the far colours' table). A sides' fourth
// number: 0, or where the block's bars' own pictures are, the long side's layer and one (the end's is the next).
@group(0) @binding(5) var<storage, read> colours: array<vec4<f32>>;
@group(0) @binding(6) var<uniform> near: Near;
// The blocks' pictures (build 205): a layer a picture, a block's top's then its sides', each with its shrunk sizes;
// read with its own pixels crisp up close, its shrunk sizes blended as it gets small.
@group(0) @binding(7) var t_blocks: texture_2d_array<f32>;
@group(0) @binding(8) var crisp: sampler;
// The near light (build 210): kept apart from the faces, only where it isn't plain daylight (`near_eye.rs`, `Light`;
// `near.rs`, `LightBlock`): a directory, an entry a chunk, then the chunks' bricks of 8 x 8 x 8 cells, 4 bits a cell.
@group(0) @binding(9) var<storage, read> kept_light: array<u32>;
// The lantern's box (build 217): a moving light's light in the 32 x 32 x 32 cells round it, two cells a word from the
// low bits, a cell's red, green and blue in 4 bits each (`near.rs`, `Blocks::carried_light`).
@group(0) @binding(10) var<storage, read> carried_light: array<u32>;
// The band being drawn (build 213), and the pictures of the two bands nearer than it (the one just nearer, then the
// one before that; a clear stand-in where there's none).
@group(1) @binding(0) var<uniform> band: Band;
@group(1) @binding(1) var t_nearer_a: texture_2d<f32>;
@group(1) @binding(2) var t_nearer_b: texture_2d<f32>;
// For the water (its pass alone): the band's own faces' depth, and those of the two bands behind it (the next, then
// the one after; an empty stand-in where there's none).
@group(2) @binding(0) var t_depth: texture_depth_2d;
@group(2) @binding(1) var t_behind_a: texture_depth_2d;
@group(2) @binding(2) var t_behind_b: texture_depth_2d;

// A band of the near eye (build 213; `near_eye.rs`, `bands_of`): a stretch of distance from the eye drawn into a
// picture of its own, in squares of 1, 2 or 4 screen pixels, by the detail curve [David, 2026-11-17: "4x4 *up
// close*, fading to 2x2, fading to 1x1"].
struct Band {
    // x, y: how much of its picture the screen takes, across and down (1, or a little less where a half or a quarter
    // of the screen was rounded up); z: its pixel's side in screen pixels; w: how much a face's hair is grown by (1 at
    // the screen's own size and 100 degrees of view).
    picture: vec4<f32>,
    // x: it draws nothing nearer the eye than this (blocks, true distance: half the reach of the band nearer than it);
    // y: nor anything farther than this (its own reach; the farthest band's is past everything). z, w: how many of
    // its pixels a side a pixel of the band just nearer than it is, and of the one before that (0: no such band).
    span: vec4<f32>,
    // x, y: how many pixels of the next band behind it a side a pixel of its own is, and of the one after (0: none).
    // z: the share of its reach, at its far end, it thins out over by a stipple (build 214; 0: a hard edge).
    // w: how many sizes its blocks' pictures are read off from what the card would choose for its pixels (build 215;
    // 0 or under: finer, up to as many sizes as its pixel is bigger than the screen's).
    behind: vec4<f32>,
}

struct Near {
    // The eye's block (north, east, up), and how far into it the eye is: a face's place is taken from the eye in whole
    // blocks first, so far from the map's middle nothing wobbles. (w's bits: 1 if the faces wear their blocks'
    // pictures, not flat in their far colours; 2 if the near eye has a light of its own; 4 if that light is the one
    // kept apart from the faces, not each face's own.)
    cell: vec4<i32>,
    // (w: the x-fader's curve, 1 a straight fade.)
    part: vec4<f32>,
    // x: how far out the near faces reach now (blocks, level); y: the x-fader's band, how many blocks before that
    // they hand over to the far picture across; z: the nearest kept, blocks ahead of the eye (the depth is that over
    // how far ahead a place is: kept turned round, 1 the nearest and falling towards 0 far out, so it's as fine at
    // 2,000 blocks as at 2 [the common way round, to build 201, couldn't tell faces a block apart out there: a
    // hidden face won a pixel now and then, and which one differed from run to run]); w: how far ahead "nothing
    // behind" is taken to be, for the water.
    reach: vec4<f32>,
    // Placed lights' flicker (build 216): x: the share of a light's brightness that wavers; y: how many times a
    // second; z: the time, seconds; w: a light's step, the share dimmer each block from it is than the last (build 218).
    glow: vec4<f32>,
    // The lantern's box's least corner (north, east, up; blocks); w: 1 if there's a box (the lantern lit).
    carried: vec4<i32>,
}

// How much wider than its cells a face is drawn all round, for each block of its distance from the eye (`near_corner`).
const HAIR: f32 = 0.00002;
// How far inside its face a corner says it is on its block's picture, blocks: so every pixel of a face, the hair's
// too, falls in a cell the face covers, whatever the rounding (the light's looked up by that cell). A hundredth of a
// picture's pixel.
const INSIDE: f32 = 0.000244140625;

struct Corner {
    @builtin(position) clip: vec4<f32>,
    // From the eye (north, east, up).
    @location(0) from_eye: vec3<f32>,
    // The facing (0 north, 1 south, 2 east, 3 west, 4 up, 5 down); the block, with what it's part of over 16 bits
    // up (`near.rs`, `Face::part`); the cell it looks into along the way it looks, in its chunk, in whole blocks and
    // one added (north or east -1 to 32, or up), with its own sky light (`<< 16`, 0 to 15: build 208's way,
    // `INCUS_NEAR_LIGHT=faces`); its chunk, as the face's third word has it.
    @location(1) @interpolate(flat) what: vec4<u32>,
    // How far along the eye's own way.
    @location(2) ahead: f32,
    // Where on its block's picture, in blocks (the picture repeats a block at a time).
    @location(3) on_picture: vec2<f32>,
}

// A face's corner. The face is three words (`near.rs`, `packed`): its place in its chunk, its facing and its width;
// its height and its block; its chunk. Four corners a face, a strip; every face is wound the same way seen from the
// side it looks out of.
@vertex
fn near_corner(@builtin(vertex_index) corner: u32, @location(0) face: vec3<u32>) -> Corner {
    let facing = (face.x >> 22u) & 7u;
    // (A face of a chunk with pieces is all in half blocks: its top bit says so; its place's eleventh bit of up is
    // in the second word.)
    let unit = select(1.0, 0.5, (face.x >> 31u) == 1u);
    let wide = f32(((face.x >> 25u) & 31u) + 1u) * unit;
    let high = f32((face.y & 1023u) + 1u) * unit;
    let chunk = vec2<i32>(i32(face.z & 0xFFFFu) - 32768, i32(face.z >> 16u) - 32768);
    // The face's least corner, from the eye's cell: its chunk's in whole blocks first, then its place in the chunk.
    let in_chunk = vec3<f32>(f32(face.x & 63u), f32((face.x >> 6u) & 63u), f32(((face.x >> 12u) & 1023u) | (((face.y >> 26u) & 1u) << 10u))) * unit;
    let cell = vec3<f32>(vec3<i32>(chunk * 32, 0) - near.cell.xyz) + in_chunk;
    var u = f32(corner & 1u);
    let v = f32(corner >> 1u);
    // The two ways it spreads, and the way it looks: a face that looks up or down spreads north and east; north or
    // south, east and up; east or west, north and up. On the far side of its cell if it looks the greater way.
    var spread_a = vec3<f32>(1.0, 0.0, 0.0);
    var spread_b = vec3<f32>(0.0, 1.0, 0.0);
    var out_of = vec3<f32>(0.0, 0.0, 1.0);
    if (facing < 2u) {
        spread_a = vec3<f32>(0.0, 1.0, 0.0);
        spread_b = vec3<f32>(0.0, 0.0, 1.0);
        out_of = vec3<f32>(1.0, 0.0, 0.0);
    } else if (facing < 4u) {
        spread_b = vec3<f32>(0.0, 0.0, 1.0);
        out_of = vec3<f32>(0.0, 1.0, 0.0);
    }
    // (North, west and up are wound one way as they come; the other three turned round.)
    if (facing == 1u || facing == 2u || facing == 5u) {
        u = 1.0 - u;
    }
    let greater = facing == 0u || facing == 2u || facing == 4u;
    let exact = cell + spread_a * u * wide + spread_b * v * high + out_of * select(0.0, unit, greater) - near.part.xyz;
    // A hair wider than its cells all round, in its own plane, and more the farther it is (a few hundredths of a
    // pixel whatever its distance). Merged faces meet along an edge at different corners, and the card, placing
    // corners on its fine grid, can leave a pixel between them that neither covers: with the far eye's sky behind,
    // white dots that flicker as the eye moves [David, 2026-11-16, build 199: "i notice speckling white dots of sky
    // when looking down and moving"]. So neighbours overlap by that hair.
    // (As much wider again on a coarser picture or a wider view, so the overlap stays the same share of its pixel.)
    let hair = length(exact) * HAIR * band.picture.w;
    let from_eye = exact + spread_a * (u * 2.0 - 1.0) * hair + spread_b * (v * 2.0 - 1.0) * hair;
    // The far eye's own view: a pixel's ray is forward + x right + y up, so a place's x and y are its share of right
    // and up over how far ahead it is.
    let ahead = dot(from_eye, camera.forward.xyz);
    let x = dot(from_eye, camera.right.xyz) / dot(camera.right.xyz, camera.right.xyz);
    let y = dot(from_eye, camera.up.xyz) / dot(camera.up.xyz, camera.up.xyz);
    var corner_out: Corner;
    // (On a picture a little bigger than the screen, the screen is its top left part.)
    corner_out.clip = vec4<f32>(x * band.picture.x + ahead * (band.picture.x - 1.0), y * band.picture.y + ahead * (1.0 - band.picture.y), near.reach.z, ahead);
    corner_out.from_eye = from_eye;
    // The cell it looks into, along the way it looks: the next past its own (in half blocks, the block that holds it).
    let own = vec3<i32>(i32(face.x & 63u), i32((face.x >> 6u) & 63u), i32(((face.x >> 12u) & 1023u) | (((face.y >> 26u) & 1u) << 10u)));
    let along = select(select(own.z, own.y, facing < 4u), own.x, facing < 2u);
    let into = (along + select(-1, 1, greater)) >> (face.x >> 31u);
    corner_out.what = vec4<u32>(facing, ((face.y >> 10u) & 0xFFFu) | (((face.y >> 22u) & 7u) << 16u), u32(into + 1) | (((face.y >> 27u) & 15u) << 16u), face.z);
    // Where the corner is on its block's picture: its place in its chunk along the two ways the face spreads, in
    // blocks. So a merged face wears the picture once a block, in step with every face beside it; a side's picture
    // stands upright (the picture's first row is its top).
    let place = in_chunk + spread_a * (u * wide + (1.0 - 2.0 * u) * INSIDE) + spread_b * (v * high + (1.0 - 2.0 * v) * INSIDE);
    corner_out.on_picture = vec2<f32>(dot(place, spread_a), dot(place, spread_b) * select(1.0, -1.0, facing < 4u));
    corner_out.ahead = ahead;
    return corner_out;
}

fn looks(facing: u32) -> vec3<f32> {
    switch facing {
        case 0u: { return vec3<f32>(1.0, 0.0, 0.0); }
        case 1u: { return vec3<f32>(-1.0, 0.0, 0.0); }
        case 2u: { return vec3<f32>(0.0, 1.0, 0.0); }
        case 3u: { return vec3<f32>(0.0, -1.0, 0.0); }
        case 4u: { return vec3<f32>(0.0, 0.0, 1.0); }
        default: { return vec3<f32>(0.0, 0.0, -1.0); }
    }
}

// The light kept for a cell (north, east, up; blocks, the world's): x, the sky's in its low 4 bits, 15, plain daylight,
// where nothing is kept for it, and over them the voice the placed light there flickers to (build 221; 0 to 15); y, z,
// w, the placed light's red, green and blue (build 216), each 0 to 15, none where no colour brick is kept (`near.rs`,
// `LightBlock::at`, `placed_at` and `voice_at`, are this in Rust, for the tests).
fn kept_at(place: vec3<i32>) -> vec4<u32> {
    let chunk = place.xy >> vec2<u32>(5u);
    let entry = (u32(chunk.x & 255) + u32(chunk.y & 255) * 256u) * 2u;
    let first = kept_light[entry];
    if (first == 0xFFFFFFFFu) {
        return vec4<u32>(15u, 0u, 0u, 0u);
    }
    let span = kept_light[entry + 1u];
    let layers = span >> 16u;
    let layer = (place.z >> 3u) - i32(span & 0xFFFFu);
    if (layer < 0 || layer >= i32(layers)) {
        return vec4<u32>(15u, 0u, 0u, 0u);
    }
    let local = place.xy & vec2<i32>(31);
    let which = u32(layer) * 16u + u32(local.x >> 3u) + u32(local.y >> 3u) * 4u;
    let i = u32(local.x & 7) + u32(local.y & 7) * 8u + u32(place.z & 7) * 64u;
    var sky = 15u;
    let brick = kept_light[first + which];
    if (brick != 0xFFFFFFFFu) {
        sky = (kept_light[first + brick + (i >> 3u)] >> ((i & 7u) * 4u)) & 15u;
    }
    var colour = 0u;
    let tinted = kept_light[first + layers * 16u + which];
    if (tinted != 0xFFFFFFFFu) {
        colour = kept_light[first + tinted + (i >> 1u)] >> ((i & 1u) * 16u);
    }
    return vec4<u32>(sky | (((colour >> 12u) & 15u) << 4u), colour & 15u, (colour >> 4u) & 15u, (colour >> 8u) & 15u);
}

// The lantern's light in a cell (north, east, up; the world's): red, green and blue, each 0 to 15; none outside its
// box, or with the lantern out (build 217).
fn carried_at(place: vec3<i32>) -> vec3<u32> {
    let local = place - near.carried.xyz;
    if (near.carried.w == 0 || any(local < vec3<i32>(0)) || any(local >= vec3<i32>(32))) {
        return vec3<u32>(0u);
    }
    let i = u32(local.x) + u32(local.y) * 32u + u32(local.z) * 1024u;
    let colour = carried_light[i >> 1u] >> ((i & 1u) * 16u);
    return vec3<u32>(colour & 15u, (colour >> 4u) & 15u, (colour >> 8u) & 15u);
}

// A placed light's levels (red, green, blue, each 0 to 15) as light on a surface (build 216): squared, so each step
// from a lamp is a clear one [David, 2026-11-06: "the light from the brazieres, where its very clear steps in
// brightness"]; and its flicker, a wobble here where it's read (every lamp in step: two waves, one the main beat).
// (Build 218: a level under 15 is a fixed share dimmer for each step under, where build 216 squared the level: so
// every step is the same size to the eye, the bright ones beside a lamp as plain as the dim ones at its edge, and
// colours whose levels fall together keep their mix [David, 2026-11-17: "i couldn't really see the steps in the red
// and green light levels"].)
// (Build 221: each lamp flickers on its own [David, 2026-11-17: "can lamps be made to flicker at random at some
// point?"]. `voice`: whose light it is, one of 16 for a lamp (`near.rs`, `voice_of`, kept with the cell's levels) or
// the lantern's own. A voice has its own pace, a quarter slower to a quarter quicker than the flicker's speed, its own
// place in the beat, and three waves that never line up, so it wavers unevenly, as a flame does. Not chance: a
// lamp's beat is fixed by where it stands. Where no light reaches, nothing is worked out.)
fn placed(levels: vec3<u32>, voice: u32) -> vec3<f32> {
    if (all(levels == vec3<u32>(0u))) {
        return vec3<f32>(0.0);
    }
    return steady(levels) * (1.0 - near.glow.x * voice_wave(voice));
}

// A placed light's levels as light on a surface, before its flicker (build 218's steps).
fn steady(levels: vec3<u32>) -> vec3<f32> {
    return select(vec3<f32>(0.0), pow(vec3<f32>(1.0 - near.glow.w), 15.0 - vec3<f32>(levels)), levels > vec3<u32>(0u));
}

// Where a voice is in its beat now, 0 to 1 (build 221): its light is dimmest at 1. (Build 222: the far lamps' dots
// read it too, so a dot pops in as its lamp's light swells.)
fn voice_wave(voice: u32) -> f32 {
    let v = f32(voice);
    let turn = near.glow.z * near.glow.y * 6.2832 * (0.75 + 0.5 * fract(v * 0.618034)) + v * 2.399963;
    return 0.5 + 0.25 * sin(turn) + 0.15 * sin(turn * 2.7 + 1.3 + v) + 0.1 * sin(turn * 5.3 + 0.7 * v);
}

// The lantern's voice (build 221): one of its own, apart from the lamps' 16.
const LANTERN_VOICE: u32 = 16u;

// The light on a face at a pixel.
struct FaceLight {
    // The sky's, 0 to 15.
    sky: u32,
    // The placed light's red, green and blue as light on a surface (`placed`): the lamps' and the lantern's, colour by
    // colour the brighter.
    placed: vec3<f32>,
}

// The light on a face at a pixel: the sky's its own (build 208's way, and no placed light); or, kept apart from the
// faces (build 210), that of the cell the face looks into there: the cell its place on its block's picture falls in,
// along the two ways the face spreads, and the one it looks into along the third.
fn face_light(corner: Corner) -> FaceLight {
    if ((near.cell.w & 4) == 0) {
        return FaceLight((corner.what.z >> 16u) & 15u, vec3<f32>(0.0));
    }
    let facing = corner.what.x;
    let a = i32(floor(corner.on_picture.x));
    let b = i32(floor(corner.on_picture.y * select(1.0, -1.0, facing < 4u)));
    let into = i32(corner.what.z & 0xFFFFu) - 1;
    var cell = vec3<i32>(a, b, into);
    if (facing < 2u) {
        cell = vec3<i32>(into, a, b);
    } else if (facing < 4u) {
        cell = vec3<i32>(a, into, b);
    }
    let chunk = vec2<i32>(i32(corner.what.w & 0xFFFFu) - 32768, i32(corner.what.w >> 16u) - 32768);
    // (The lamps' light and the lantern's, each flickering to its own voice: colour by colour, the brighter.)
    let place = vec3<i32>(chunk * 32, 0) + cell;
    let kept = kept_at(place);
    return FaceLight(kept.x & 15u, max(placed(kept.yzw, kept.x >> 4u), placed(carried_at(place), LANTERN_VOICE)));
}

// A surface's colour lit as the far eye lights a finished ray (`far.wgsl`'s `lit`, `sunlit` and `shaded`): the sun on
// the way it looks, the sky's light, and from the far eye's light store at its place the sun's shadow and how much
// sky it sees.
//
// With the near eye's own sky light (build 208; `near.cell.w`'s second bit), `sky_light` is the face's (0 to 15): the
// sun reaches it only where it's fully open to the sky, and the sky's share is scaled by it (0, a cave past its mouth,
// is black; outdoor shade never under the floor). The far eye's light store still gives the land's large shadows and
// how open a valley is, so the two eyes agree at the hand-over.
fn near_lit(colour: vec3<f32>, light: f32, from_eye: vec3<f32>, sky_light: u32) -> vec3<f32> {
    var whole = lit(colour, light);
    let sunny = sunlit(colour, light);
    let skyish = whole - sunny;
    var sun = 1.0;
    var sky_open = 1.0;
    if (camera.view.w > 0.5) {
        let store = cell_light(camera.position.xyz + from_eye, f32(1 << u32(FIRST)));
        sun = select(1.0, store.x, camera.follow.y > 0.5);
        sky_open = store.y;
    }
    var own = 1.0;
    if ((near.cell.w & 2) != 0) {
        own = f32(sky_light) / 15.0;
        sun *= select(0.0, 1.0, sky_light >= 15u);
    }
    whole -= sunny * (1.0 - sun);
    let sky = select(1.0, mix(1.0, sky_open, camera.sky.y), camera.sky.x > 0.5);
    let tint = mix(vec3<f32>(1.0), SKY_TINT, camera.sky.z);
    return whole + skyish * (sky * tint * own - 1.0);
}

// The x-fader [David, 2026-11-16: "i think a x-fader with curves could hide it almost completely so blocks smoothly
// fade into rays"]. How much of a near surface shows at a place: all of it before the band, none at the near faces'
// reach, and across the band along the curve: straight at 1; higher, an S that holds each side longer and crosses
// quicker in the middle (half and half at the band's middle whatever the curve). A band of no width is a hard edge.
fn near_shows(from_eye: vec3<f32>) -> f32 {
    let across = clamp(1.0 - (near.reach.x - length(from_eye.xy)) / max(near.reach.y, 0.001), 0.0, 1.0);
    let curve = max(near.part.w, 1.0);
    let rays = pow(across, curve);
    let blocks = pow(1.0 - across, curve);
    return blocks / max(blocks + rays, 1e-9);
}

// Whether a place isn't in this band's stretch of distance (build 213): past the band's reach, or well inside the band
// nearer than it. A nearer band is always in front of a farther one along a line of sight (bands are by true
// distance from the eye).
fn out_of_band(corner: Corner) -> bool {
    let off = length(corner.from_eye);
    return off > band.span.y || off < band.span.x;
}

// The stipple (build 214) [David, 2026-11-17: "lets try stippling"]: over the last stretch of a band's reach its
// pixels are drawn or left out by an ordered pattern of 4 x 4 of its own pixels (Bayer's), all of them at the
// stretch's start and none at the band's end, so the band thins out into the one behind it with every pixel crisp.
// Whether this pixel is one left out.
fn stippled_out(corner: Corner) -> bool {
    if (band.behind.z <= 0.0) {
        return false;
    }
    let left = (band.span.y - length(corner.from_eye)) / (band.span.y * band.behind.z);
    if (left >= 1.0) {
        return false;
    }
    // (Bayer's 4 x 4 from his 2 x 2, twice: 0, 2 / 3, 1.)
    let at = vec2<u32>(corner.clip.xy);
    let fine = at & vec2<u32>(1u);
    let coarse = (at >> vec2<u32>(1u)) & vec2<u32>(1u);
    let order = 4u * (((fine.x ^ fine.y) << 1u) | fine.y) + (((coarse.x ^ coarse.y) << 1u) | coarse.y);
    return left <= (f32(order) + 0.5) / 16.0;
}

// Whether a nearer band's picture already has at least `covered` of this pixel (the water's test: any of it, so the
// same water isn't laid on twice; a block's side under a nearer band's water is still drawn, and seen through it:
// `near_mask.wgsl` leaves those out only where a nearer picture has all of the pixel).
fn nearer_has(corner: Corner, covered: f32) -> bool {
    let at = vec2<u32>(corner.clip.xy);
    if (band.span.z > 0.5 && textureLoad(t_nearer_a, vec2<i32>(at / u32(band.span.z)), 0).a >= covered) {
        return true;
    }
    if (band.span.w > 0.5 && textureLoad(t_nearer_b, vec2<i32>(at / u32(band.span.w)), 0).a >= covered) {
        return true;
    }
    return false;
}

// Which picture a face wears, and where on it (build 227): its block's, a block at a time (`on_picture`); or, for a
// piece of a block whose bars have their own pictures (`bars`: the long side's layer and one), old's way
// (`old/src/sight/march.wgsl`, `bar_uv`): a bar's long sides the side's picture, its length up the bar from the
// bar's own end (half a block on where it begins at an odd half block) and a half block across; its ends, and every
// face of a lone eighth, the end's, a half block each way. (The pictures are laid side by side in their layers, so
// a block's measure reads them a half block a time.) By sums only, no branch: the card's reading wants its
// neighbours' places.
fn face_picture(facing: u32, which: u32, part: u32, bars: u32, on_picture: vec2<f32>) -> vec3<f32> {
    // The way the bar lies (0 north, 1 east, 2 up; 3 for no bar) and the way the face looks.
    let kind = part & 3u;
    let axis = select(3u, kind - 1u, kind != 0u);
    let looks_along = facing / 2u;
    let own = bars != 0u && part != 0u;
    let end = kind == 0u || axis == looks_along;
    // The face spreads along `a` then `b` (north and south: east, up; east and west: north, up; up and down: north,
    // east); `on_picture` is its place along them, in blocks, up turned down on a side.
    let along_a = (facing < 2u && axis == 1u) || (facing >= 2u && axis == 0u);
    let start = select(0.0, 0.5, (part & 4u) != 0u);
    let flip = select(1.0, -1.0, facing < 4u);
    let a = on_picture.x;
    let b = on_picture.y * flip;
    let side = select(vec2<f32>(a, -(b - start)), vec2<f32>(b, -(a - start)), along_a);
    let uv = select(on_picture, side, own && !end);
    let layer = select(which, select(bars - 1u, bars, end), own);
    return vec3<f32>(uv, f32(layer));
}

@fragment
fn near_face(corner: Corner) -> @location(0) vec4<f32> {
    let facing = corner.what.x;
    let block = corner.what.y & 0xFFFFu;
    // The block's picture here (its top's looking up, its sides' otherwise; a bar's own, build 227), read first of
    // all: the card finds how small the picture is here from this pixel's neighbours' readings, before any pixel is
    // left out.
    let which = block * 2u + select(1u, 0u, facing == 4u);
    let pictures = textureNumLayers(t_blocks);
    var bars = 0u;
    if (block * 2u + 1u < arrayLength(&colours)) {
        bars = u32(colours[block * 2u + 1u].w);
    }
    let laid = face_picture(facing, which, corner.what.y >> 16u, bars, corner.on_picture);
    // (In a coarse band the card finds the picture small sooner, by its bigger pixels, and blends it: read finer by
    // the sharpness, a coarse square shows one square of the picture for as long as the screen's own pixels would,
    // build 215.)
    let picture = textureSampleBias(t_blocks, crisp, laid.xy, min(u32(laid.z), pictures - 1u), band.behind.w);
    // (What a nearer band's picture covers never gets here: the mask in the band's depth drops it first.)
    if (out_of_band(corner) || stippled_out(corner)) {
        discard;
    }
    let shows = near_shows(corner.from_eye);
    if (shows <= 0.0) {
        discard;
    }
    let way = looks(facing);
    // Its picture's colour there; or flat, its far colour (`INCUS_NEAR_TEXTURES=0`). (A block the tables haven't got,
    // one first named after they were made: mid grey, as the far cells have a block with no colour.)
    var colour = vec3<f32>(0.5);
    if ((near.cell.w & 1) != 0 && which < pictures) {
        colour = picture.rgb;
    } else if (which < arrayLength(&colours)) {
        colour = colours[which].rgb;
    }
    let light = max(dot(way, camera.sun.xyz), 0.0);
    // (Into the near eye's own picture, whole: the nearest face wins a pixel, whatever order the chunks come in.
    // Kept multiplied by how much of it shows, for the x-fader to lay over the far picture.)
    // By placed light (build 216) a surface is its colour times the light there; a pixel shows the brighter, colour
    // by colour, of that and its daylight: so by day a lamp shows only in shade, caves and halls.
    let kept = face_light(corner);
    let seen = max(near_lit(colour, light, corner.from_eye, kept.sky), colour * kept.placed);
    return vec4<f32>(to_srgb(clamp(seen, vec3<f32>(0.0), vec3<f32>(1.0))) * shows, shows);
}

// The water's top, over the faces already drawn: the far eye's water (its colour, and how fast it dims what's seen
// through it), by how much water the eye looks through to the face behind.
@fragment
fn near_water(corner: Corner) -> @location(0) vec4<f32> {
    if (out_of_band(corner) || stippled_out(corner) || nearer_has(corner, 0.001)) {
        discard;
    }
    let shows = near_shows(corner.from_eye);
    if (shows <= 0.0) {
        discard;
    }
    var behind = textureLoad(t_depth, vec2<i32>(corner.clip.xy), 0);
    // (A face nearer than the water has the greater depth: it hides it.)
    if (corner.clip.z < behind) {
        discard;
    }
    // (No face of its own band behind it: the face of the next band behind, or of the one after, at this pixel's
    // middle in their pictures.)
    if (behind <= 0.0 && band.behind.x > 0.5) {
        behind = textureLoad(t_behind_a, vec2<i32>((floor(corner.clip.xy) + 0.5) * band.behind.x), 0);
    }
    if (behind <= 0.0 && band.behind.y > 0.5) {
        behind = textureLoad(t_behind_b, vec2<i32>((floor(corner.clip.xy) + 0.5) * band.behind.y), 0);
    }
    // How far ahead the face behind is, from its depth; nothing behind (the near faces end): deep water.
    let ahead_behind = select(near.reach.z / max(behind, 1e-12), near.reach.w, behind <= 0.0);
    let through = max(ahead_behind - corner.ahead, 0.0) * length(corner.from_eye) / max(corner.ahead, 0.01);
    let dimmed = 1.0 - exp(-camera.water.w * through);
    let covers = dimmed * shows;
    let kept = face_light(corner);
    let seen = max(near_lit(camera.water.rgb, 1.0, corner.from_eye, kept.sky), camera.water.rgb * kept.placed);
    return vec4<f32>(to_srgb(clamp(seen, vec3<f32>(0.0), vec3<f32>(1.0))) * covers, covers);
}

// ── The far lamps (build 222) ──
//
// [David, 2026-11-18: "the single pixels of light don't twinkle quite the way they did in old incus, from a
// distance"; old's far glows, 2026-11-09: "pixels of green that popped in and out as they twinkled, less and less as
// you got farther away"; where it begins: "About a pixel".] After the x-fader, each lamp in the list (`near.rs`,
// `LampsLane`) is a dot one screen pixel square at its block's middle, its light added to the picture: none while
// its block is `begins.x` pixels across or more, whole at `begins.y`; shown only while its light is up its swing past
// a bar that rises with distance, so far off it pops in at its peaks alone (with no flicker, shown steadily); dimmer
// by the share of the pixel its block covers under a pixel; and hidden where the near picture has something nearer
// at its pixel. Across the x-fader (build 223) it's two parts: the near part by how much of the near picture shows
// there; the far part by the rest, hidden where a ray marched to it through the far cells met a surface short of
// it (`far_lamps.wgsl`), and dimmed by the fog; so it goes on past the near reach and fades into the fog, never snapping off. (In a group of its own, the
// fourth: the near eye's other passes' groups take the second and third.)

struct Twinkle {
    // x, y: the screen's size; z: the full-size band's pixel's side in screen pixels; w: how many screen pixels a
    // block is across a block ahead of the eye.
    screen: vec4<f32>,
    // x: a lamp's block across, pixels, at which its dot begins to show; y: at which it's whole; z: the bar it must be
    // over where it begins (a share of its swing); w: how much the bar rises.
    begins: vec4<f32>,
    // x: over how many blocks past where it's whole the bar rises; y: how far behind its block's middle something
    // may be at its pixel in the near picture and not hide it (blocks).
    rest: vec4<f32>,
}

@group(3) @binding(0) var<uniform> twinkle: Twinkle;
// The lamps: each its cell (north, east, up; the world's) and its light, red, green and blue in 4 bits each from the
// low bits, with its voice over them (`<< 12`), and over that (`<< 16`, build 223) whether it's seen past the near
// reach (`far_lamps.wgsl`: a ray marched to it through the far cells).
@group(3) @binding(1) var<storage, read> lamps: array<vec4<i32>>;
// The full-size band's depth (the near eye's farthest picture; where a nearer band covers a pixel its mask is in it,
// the nearest there is).
@group(3) @binding(2) var t_whole_depth: texture_depth_2d;
// How much of a light the fog lets through to the eye over `t` blocks along `d` (`far.wgsl`'s `fogged`, its amount
// turned round).
fn fog_leaves(d: vec3<f32>, t: f32) -> f32 {
    if (camera.fog.x <= 0.0) {
        return 1.0;
    }
    let thins = camera.fog.y;
    let at_eye = camera.fog.x * exp(clamp(-thins * (camera.position.z - camera.fog.z), -80.0, 80.0));
    let climb = thins * d.z;
    var along = t;
    if (abs(climb * t) > 1e-4) {
        along = (1.0 - exp(-clamp(climb * t, -80.0, 80.0))) / climb;
    }
    return exp(-at_eye * along);
}

struct Dot {
    @builtin(position) clip: vec4<f32>,
    @location(0) @interpolate(flat) colour: vec3<f32>,
}

// A lamp's dot's corner: two triangles over the screen pixel its block's middle falls in; off the screen (nothing
// drawn) where it doesn't show.
@vertex
fn lamp_dot(@builtin(vertex_index) k: u32, @builtin(instance_index) which: u32) -> Dot {
    var dot_out: Dot;
    dot_out.clip = vec4<f32>(2.0, 2.0, 0.0, 1.0);
    dot_out.colour = vec3<f32>(0.0);
    let lamp = lamps[which];
    let from_eye = vec3<f32>(lamp.xyz - near.cell.xyz) + 0.5 - near.part.xyz;
    let ahead = dot(from_eye, camera.forward.xyz);
    if (ahead < 1.0) {
        return dot_out;
    }
    // How many pixels across its block is, and how far into showing that puts it.
    let across = twinkle.screen.w / ahead;
    let begun = clamp((twinkle.begins.x - across) / max(twinkle.begins.x - twinkle.begins.y, 0.001), 0.0, 1.0);
    if (begun <= 0.0) {
        return dot_out;
    }
    let x = dot(from_eye, camera.right.xyz) / dot(camera.right.xyz, camera.right.xyz) / ahead;
    let y = dot(from_eye, camera.up.xyz) / dot(camera.up.xyz, camera.up.xyz) / ahead;
    if (abs(x) >= 1.0 || abs(y) >= 1.0) {
        return dot_out;
    }
    let pixel = floor(vec2<f32>((x * 0.5 + 0.5) * twinkle.screen.x, (0.5 - y * 0.5) * twinkle.screen.y));
    // Hidden by something nearer at its pixel in the near picture (its own block's faces are within the slack).
    let depth = textureLoad(t_whole_depth, vec2<i32>(pixel / max(twinkle.screen.z, 1.0)), 0);
    if (depth > 0.0 && near.reach.z / depth < ahead - twinkle.rest.y) {
        return dot_out;
    }
    // The near part, by how much of the near picture shows there; the far part, the rest, unless the far cells stand
    // in the way, through the fog.
    let shows = near_shows(from_eye);
    var far_part = 1.0 - shows;
    if (far_part > 0.0) {
        if (((lamp.w >> 16u) & 1) == 0) {
            far_part = 0.0;
        } else {
            far_part *= fog_leaves(normalize(from_eye), length(from_eye));
        }
    }
    let seen = shows + far_part;
    if (seen <= 0.0) {
        return dot_out;
    }
    // Its twinkle: shown while its light is up its swing past the bar (1 less its wave), the bar rising from where
    // it's whole on.
    let whole_at = twinkle.screen.w / max(twinkle.begins.y, 0.001);
    let past = clamp((ahead - whole_at) / max(twinkle.rest.x, 1.0), 0.0, 1.0);
    let voice = u32(lamp.w >> 12u) & 15u;
    if (near.glow.x > 0.0 && 1.0 - voice_wave(voice) < twinkle.begins.z + twinkle.begins.w * past) {
        return dot_out;
    }
    let levels = vec3<u32>(u32(lamp.w) & 15u, (u32(lamp.w) >> 4u) & 15u, (u32(lamp.w) >> 8u) & 15u);
    let share = min(across * across, 1.0);
    dot_out.colour = to_srgb(clamp(steady(levels) * begun * share * seen, vec3<f32>(0.0), vec3<f32>(1.0)));
    let corners = array<vec2<f32>, 6>(vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0));
    let at = pixel + corners[k % 6u];
    dot_out.clip = vec4<f32>(at.x / twinkle.screen.x * 2.0 - 1.0, 1.0 - at.y / twinkle.screen.y * 2.0, 0.0, 1.0);
    return dot_out;
}

@fragment
fn lamp_glow(dot_in: Dot) -> @location(0) vec4<f32> {
    return vec4<f32>(dot_in.colour, 0.0);
}

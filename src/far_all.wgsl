// ── The far eye's layers marched by one dispatch (the Oracle's step 2, its one-dispatch form) ──
//
// Not a shader on its own: `far_eye.rs` puts this in `far.wgsl`'s place of one layer's outputs (between its two
// marks), and every use of the camera there becomes a read of this invocation's layer's row of `cameras` (in place: a
// copy of the whole row for each invocation cost, build 155). Every invocation works out which layer's pixel it is
// and marches it with `far.wgsl`'s own `march_pixel`. A finer layer's lead (where it's wanted, where its rays start)
// is the coarser layer's of the frame before.

// A picture for each layer (0 the base), and all their distances in one texture: the base, the middle layer and the
// finest side by side from its left (four written textures in a pass, the library's default most).
@group(0) @binding(5) var picture0: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(17) var picture1: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(19) var picture2: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var how_far: texture_storage_2d<rgba32float, write>;
// The frame before's words, the base's then the middle layer's in one run; and this frame's, the same way.
@group(0) @binding(8) var<storage, read> worth_in: array<u32>;
@group(0) @binding(9) var<storage, read_write> worth_out: array<u32>;
// The frame before's distances, laid out the same.
@group(0) @binding(10) var far_prev: texture_2d<f32>;

// The long rays' two counts (the cut): how many rays this frame's list holds, and the frame before's. Words written
// by many rays at once, so only ever touched by the few rays that are long, and never read here. [Build 181 kept the
// tiles' starts in the same buffer, and every ray's read of its tile's start became a read and a write locked
// together, on a word its 63 neighbours were locking too: 4 to 11% at every mark.]
@group(0) @binding(7) var<storage, read_write> long_rays: array<atomic<u32>>;
// The cut's own store, plain words, each written by one thread: the two lists (a listed ray's pixel and its layer,
// a word); then, for each place in each list, how its ray stood where each of its 17 lengths begins (how much it still
// let through; under nothing if it was in haze there), for the frame after to begin its lengths by; and after them
// the lengths' stacks: for each place in a list, its ray's 17 lengths, each `STACK` words
// (what the length gathered, 3; the sun's part of that, 3; how much it let through; where it last gathered with over
// half still let through, which is its own first surface if it let through a half or less; where it first touched
// anything; its steps with the level read where it last gathered, and in the top bit whether it ended in air). What a
// length gathered and let through are of the whole ray's light, as far as the frame before knew: it began letting
// through what the ray did there then (`cut_before`), which is the stack's last word.
@group(0) @binding(25) var<storage, read_write> cut: array<u32>;
const STACK: u32 = 11u;

// Where each tile's rays start, by the one dispatch: under the base layer's part of the frame before's distances
// (nothing else is there: the base's rows are fewer than the finest's). The start pass is given that picture to
// write; the march reads it as it reads the rest.
fn put_start(tile: vec2<u32>, across: u32, v: f32) {
    textureStore(how_far, vec2<i32>(tile) + vec2<i32>(0, i32(cameras[0].view.y)), vec4<f32>(v, 0.0, 0.0, 0.0));
}

fn tile_start(tile: vec2<u32>, across: u32) -> f32 {
    return textureLoad(far_prev, vec2<i32>(tile) + vec2<i32>(0, i32(cameras[0].view.y)), 0).x;
}

// Which layer this invocation marches (0 the base).
var<private> layer_now: u32;

// Where the middle layer's words start in the runs: after the base's.
fn base_pixels() -> u32 {
    return u32(cameras[0].view.x) * u32(cameras[0].view.y);
}

fn put_picture(at: vec2<i32>, v: vec4<f32>) {
    switch layer_now {
        case 0u { textureStore(picture0, at, v); }
        case 1u { textureStore(picture1, at, v); }
        default { textureStore(picture2, at, v); }
    }
}

// Where a layer's distances start in their texture: after the layers before it.
fn far_left(layer: u32) -> i32 {
    var left = 0;
    if (layer > 0u) {
        left += i32(cameras[0].view.x);
    }
    if (layer > 1u) {
        left += i32(cameras[1].view.x);
    }
    return left;
}

fn put_far(at: vec2<i32>, v: vec4<f32>) {
    textureStore(how_far, at + vec2<i32>(far_left(layer_now), 0), v);
}

// This layer's word for the layer below (the finest has none below and never writes).
fn put_worth(i: u32, v: u32) {
    worth_out[i + select(0u, base_pixels(), layer_now == 1u)] = v;
}

// The layer above's word (the middle layer reads the base's, the finest the middle layer's).
fn get_worth(i: u32) -> u32 {
    return worth_in[i + select(0u, base_pixels(), layer_now == 2u)];
}

// The layer above's last pixel, and its distance at one (never asked by the base).
fn far_above_edge() -> vec2<i32> {
    return vec2<i32>(cameras[max(layer_now, 1u) - 1u].view.xy) - 1;
}

fn get_far_above(at: vec2<i32>) -> f32 {
    return textureLoad(far_prev, at + vec2<i32>(far_left(max(layer_now, 1u) - 1u), 0), 0).x;
}

// A long ray listed (the cut): a ray that took `list.y` steps or more adds its pixel to this frame's list, and its
// place in the list, one up, goes beside its distance for the frame after (0: not long, or the list was full). A cut
// ray, joined from its lengths, stays listed down to a quarter of that: its lengths' steps can come to well under
// the whole ray's (a ray in haze goes on in haze, where a length that begins in a gap of it skips on), and a ray
// that left the list on its lengths' count came straight back, marched whole every other frame [build 183's check:
// at the mountain top a ray of 172 steps did]. [Rays long enough to set the far eye's time are a few hundred, in a
// thin band at the horizon, and mostly the same from frame to frame: David, 2026-11-14, "we have to reduce the
// longest rays, one by one in order of length".]
fn list_long(id: vec2<u32>, steps: f32, joined: bool) -> f32 {
    let long = cameras[0].list.y;
    if (steps < select(long, long * 0.25, joined)) {
        return 0.0;
    }
    let holds = u32(cameras[0].list.z);
    let now = u32(cameras[0].list.x);
    let slot = atomicAdd(&long_rays[now], 1u);
    if (slot >= holds) {
        return 0.0;
    }
    cut[now * holds + slot] = (layer_now << 30u) | (id.y << 15u) | id.x;
    listed_now = slot;
    // A ray listed whole: how it stands where its lengths begin isn't known yet. The frame after, each takes up
    // letting through all, and looks before it skips. (A joined ray's are written by its join.)
    if (!joined) {
        let befores = befores_at(now, slot);
        for (var k = 0u; k < 17u; k++) {
            cut[befores + k] = bitcast<u32>(-1.0);
        }
    }
    return f32(slot + 1u);
}

// The place in this frame's list a ray was given by `list_long` (the join asks, to write how the ray stood).
var<private> listed_now: u32;

// Where in `cut` a listed ray's 17 numbers are, of how it stood where each of its lengths begins: `which` list, its
// place.
fn befores_at(which: u32, slot: u32) -> u32 {
    let holds = u32(cameras[0].list.z);
    return 2u * holds + (which * holds + slot) * 17u;
}

// How a ray in the frame before's list stood there, that frame, where its length `length` begins: how much it still
// let through; under nothing if it was in haze.
fn cut_before(slot: u32, length: u32) -> f32 {
    return bitcast<f32>(cut[befores_at(1u - u32(cameras[0].list.x), slot) + length]);
}

// The ray at a place in the frame before's list: its pixel (x, y) and its layer; layer 3 if no ray was listed there
// the frame before (the ray's own word beside its distance says where it was listed, and is asked: the list's word
// may be an older frame's).
fn listed_pixel(slot: u32) -> vec3<u32> {
    let holds = u32(cameras[0].list.z);
    if (slot >= holds) {
        return vec3<u32>(0u, 0u, 3u);
    }
    let entry = cut[(1u - u32(cameras[0].list.x)) * holds + slot];
    let layer = entry >> 30u;
    let pixel = vec2<u32>(entry & 0x7FFFu, (entry >> 15u) & 0x7FFFu);
    if (layer == 3u || any(pixel >= vec2<u32>(cameras[layer].view.xy))) {
        return vec3<u32>(0u, 0u, 3u);
    }
    let listed = textureLoad(far_prev, vec2<i32>(pixel) + vec2<i32>(far_left(layer), 0), 0).w;
    if (u32(listed) != slot + 1u) {
        return vec3<u32>(0u, 0u, 3u);
    }
    return vec3<u32>(pixel, layer);
}

// A pixel's place in the frame before's list, if its ray was listed (it's cut this frame): from beside its distance.
fn cut_slot(id: vec2<u32>) -> u32 {
    if (cameras[0].cut.x <= 0.0) {
        return NOT_CUT;
    }
    let listed = textureLoad(far_prev, vec2<i32>(id) + vec2<i32>(far_left(layer_now), 0), 0).w;
    return select(NOT_CUT, u32(listed) - 1u, listed > 0.5);
}

// Where a listed ray's lengths' stacks start in `cut`.
fn stacks_at(slot: u32) -> u32 {
    return 36u * u32(cameras[0].list.z) + slot * 17u * STACK;
}

// What a length gathered, put by for the join, and what it began letting through.
fn put_stack(slot: u32, length: u32, seen: Seen, began: f32) {
    let gathered = seen.skyish + seen.sunny;
    let at = stacks_at(slot) + length * STACK;
    cut[at] = bitcast<u32>(gathered.r);
    cut[at + 1u] = bitcast<u32>(gathered.g);
    cut[at + 2u] = bitcast<u32>(gathered.b);
    cut[at + 3u] = bitcast<u32>(seen.sunny.r);
    cut[at + 4u] = bitcast<u32>(seen.sunny.g);
    cut[at + 5u] = bitcast<u32>(seen.sunny.b);
    cut[at + 6u] = bitcast<u32>(seen.sky_share);
    cut[at + 7u] = bitcast<u32>(seen.reached);
    cut[at + 8u] = bitcast<u32>(seen.touched);
    cut[at + 9u] = (u32(seen.steps) << 5u) | u32(seen.reached_level + 1) | (u32(seen.ended_in_air) << 31u);
    cut[at + 10u] = bitcast<u32>(began);
}

// The join (the cut): for each ray in the frame before's list, its lengths' stacks joined front to back (each lets
// through to the next what it didn't cover), into what the whole ray would have seen: its first surface is where
// what's let through first falls to a half, as the march has it. Then the ray is finished into its pixel as any ray
// is: lit once, by that surface. A pass of its own after the march, 64 x 64 threads, one a place in the list.
@compute @workgroup_size(8, 8)
fn join_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    if (cameras[0].cut.x <= 0.0) {
        return;
    }
    let listed = id.y * 64u + id.x;
    let pixel = listed_pixel(listed);
    if (pixel.z == 3u) {
        return;
    }
    layer_now = pixel.z;
    let at = pixel.xy;
    // Shown as much as the layer above says, as its own thread found; not at all, and its threads put nothing by.
    var shown = 1.0;
    if (camera.layer.x > 0.0) {
        let parent = parent_of(at);
        let worth = get_worth(parent.y * u32(camera.layer.x) + parent.x) & 0xFFu;
        if (worth == 0u) {
            return;
        }
        shown = f32(worth) / 255.0;
    }
    let d = ray_through(at);
    var gathered = vec3<f32>(0.0);
    var sunny = vec3<f32>(0.0);
    var through = 1.0;
    var first = camera.reach.x;
    var first_level = -1;
    var touched = camera.reach.x;
    var steps = 0.0;
    var chain = 0.0;
    // How the ray stands where each length begins, for the frame after.
    var befores: array<f32, 17>;
    var in_air = true;
    let stacks = stacks_at(listed);
    for (var k = 0u; k <= 16u; k++) {
        befores[k] = select(-through, through, in_air);
        if (through < 0.01) {
            continue;
        }
        let s = stacks + k * STACK;
        // What the length began letting through, and so how its numbers stand to the ray's as it is this frame:
        // scaled by `now`.
        let began = clamp(bitcast<f32>(cut[s + 10u]), 0.05, 1.0);
        let now = through / began;
        let ended = bitcast<f32>(cut[s + 6u]);
        let reached = bitcast<f32>(cut[s + 7u]);
        let touch = bitcast<f32>(cut[s + 8u]);
        let word = cut[s + 9u];
        let took = f32((word >> 5u) & 0x3FFFFFFu);
        steps += took;
        chain = max(chain, took);
        touched = min(touched, touch);
        gathered += now * vec3<f32>(bitcast<f32>(cut[s]), bitcast<f32>(cut[s + 1u]), bitcast<f32>(cut[s + 2u]));
        sunny += now * vec3<f32>(bitcast<f32>(cut[s + 3u]), bitcast<f32>(cut[s + 4u]), bitcast<f32>(cut[s + 5u]));
        let after = now * ended;
        if (through > 0.5 && after <= 0.5 && touch < camera.reach.x) {
            // The ray's first surface is in this length: where the ray has let through a half. If the length began
            // as the ray stands this frame, that's where the length itself last gathered with over a half let
            // through (`reached`), exactly. If the ray stands otherwise now (the view moved, or the ray is new to
            // the list): between the length's first touch and there, as far along as a surface thinning evenly
            // would put it. [Build 183's first pictures: a ray's first surface a tenth out, and the layer below was
            // drawn a quarter more or less over a skyline: it fades in by that distance.]
            if (reached < camera.reach.x) {
                let at_half = 0.5 / through;
                let at_reached = clamp(max(ended, 0.5) / began, 1e-4, 0.9999);
                first = mix(touch, reached, clamp(log(at_half) / log(at_reached), 0.0, 1.0));
                first_level = i32(word & 31u) - 1;
            } else {
                first = touch;
                first_level = level_at(touch);
            }
        }
        through = after;
        in_air = (word >> 31u) == 1u;
    }
    var seen: Seen;
    seen.colour = gathered + through * sky(d);
    seen.distance = first;
    seen.steps = steps;
    seen.level = first_level;
    seen.sunny = sunny;
    seen.at = camera.position.xyz + d * first;
    seen.wide = shadow_wide(first, first_level);
    seen.skyish = gathered - sunny;
    seen.touched = touched;
    seen.sky_share = through;
    seen.reached = first;
    seen.reached_level = first_level;
    listed_now = NOT_CUT;
    finish(at, d, shown, seen, true, chain);
    // Still in the list: how it stood this frame, for its lengths the frame after.
    if (listed_now != NOT_CUT) {
        let next = befores_at(u32(cameras[0].list.x), listed_now);
        for (var k = 0u; k < 17u; k++) {
            cut[next + k] = bitcast<u32>(befores[k]);
        }
    }
}

// The dispatch's grid: the base, the middle layer and the finest side by side across it, each from its top row (each
// rounded up to whole groups of 8), so the base's rows are the middle layer's too and neither goes first [the grid's
// order matters: builds 155 and 156]. `march_pixel` turns away what falls outside its layer.
// Past the layers, a strip of threads for the cut rays' lengths.
@compute @workgroup_size(8, 8)
fn march_all(@builtin(global_invocation_id) id: vec3<u32>) {
    let base_across = (u32(cameras[0].view.x) + 7u) / 8u * 8u;
    let middle_across = (u32(cameras[1].view.x) + 7u) / 8u * 8u;
    let fine_across = (u32(cameras[2].view.x) + 7u) / 8u * 8u;
    var at = id.xy;
    var length = 0u;
    var listed = NOT_CUT;
    if (id.x < base_across) {
        layer_now = 0u;
    } else if (id.x < base_across + middle_across) {
        at.x -= base_across;
        layer_now = 1u;
    } else if (id.x < base_across + middle_across + fine_across) {
        at.x -= base_across + middle_across;
        layer_now = 2u;
    } else {
        // The strip past the layers (the cut): a thread for each of the sixteen lengths beyond the near one, for
        // each place in the frame before's list, row after row from the strip's top.
        if (cameras[0].cut.x <= 0.0) {
            return;
        }
        let n = id.y * u32(cameras[0].list.w) + (id.x - (base_across + middle_across + fine_across));
        listed = n >> 4u;
        length = 1u + (n & 15u);
        let pixel = listed_pixel(listed);
        if (pixel.z == 3u) {
            return;
        }
        layer_now = pixel.z;
        at = pixel.xy;
    }
    march_pixel(at, length, listed);
}

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

// The long rays' lists (the cut): this frame's and the frame before's, each a count and then the pixels. Words
// written by many rays at once, so only ever touched by the few rays that are long. [Build 181 kept the tiles'
// starts in the same buffer, and every ray's read of its tile's start became a read and a write locked together, on
// a word its 63 neighbours were locking too: 4 to 11% at every mark.]
@group(0) @binding(7) var<storage, read_write> long_rays: array<atomic<u32>>;

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

// A long ray listed (the cut, its first stage): a ray that took `list.z` steps or more adds its pixel to this
// frame's list in `long_rays` (a count, then the pixels, the layer in the top two bits),
// and its place in the list, one up, goes beside its distance for the frame after (0: not long, or the list was
// full). [Rays long enough to set the far eye's time are a few hundred, in a thin band at the horizon, and mostly
// the same from frame to frame: David, 2026-11-14, "we have to reduce the longest rays, one by one in order of
// length".]
fn list_long(i: u32, steps: f32) -> f32 {
    if (steps < cameras[0].list.z) {
        return 0.0;
    }
    let at = u32(cameras[0].list.x);
    let slot = atomicAdd(&long_rays[at], 1u);
    if (slot >= u32(cameras[0].list.w)) {
        return 0.0;
    }
    atomicStore(&long_rays[at + 1u + slot], (layer_now << 30u) | i);
    return f32(slot + 1u);
}

// The dispatch's grid: the base, the middle layer and the finest side by side across it, each from its top row (each
// rounded up to whole groups of 8), so the base's rows are the middle layer's too and neither goes first [the grid's
// order matters: builds 155 and 156]. `march_pixel` turns away what falls outside its layer.
@compute @workgroup_size(8, 8)
fn march_all(@builtin(global_invocation_id) id: vec3<u32>) {
    let base_across = (u32(cameras[0].view.x) + 7u) / 8u * 8u;
    let middle_across = (u32(cameras[1].view.x) + 7u) / 8u * 8u;
    var at = id.xy;
    if (id.x < base_across) {
        layer_now = 0u;
    } else if (id.x < base_across + middle_across) {
        at.x -= base_across;
        layer_now = 1u;
    } else {
        at.x -= base_across + middle_across;
        layer_now = 2u;
    }
    march_pixel(at);
}

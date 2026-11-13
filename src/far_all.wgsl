// ── The far eye's layers marched by one dispatch (the Oracle's step 2, its one-dispatch form) ──
//
// Not a shader on its own: `far_eye.rs` puts this in `far.wgsl`'s place of one layer's outputs (between its two
// marks), and every use of the camera there becomes a read of this invocation's layer's row of `cameras` (in place: a
// copy of the whole row for each invocation cost, build 155). Every invocation works out which layer's pixel it is
// and marches it with `far.wgsl`'s own `march_pixel`. A finer layer's lead (where it's wanted, where its rays start,
// its sky light) is the coarser layer's of the frame before.

// A picture and distances for each layer (0 the base).
@group(0) @binding(5) var picture0: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var how_far0: texture_storage_2d<rgba32float, write>;
@group(0) @binding(17) var picture1: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(18) var how_far1: texture_storage_2d<rgba32float, write>;
@group(0) @binding(19) var picture2: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(20) var how_far2: texture_storage_2d<rgba32float, write>;
// The frame before's words, the base's then the middle layer's in one run; and this frame's, the same way.
@group(0) @binding(8) var<storage, read> worth_in: array<u32>;
@group(0) @binding(9) var<storage, read_write> worth_out: array<u32>;
// The frame before's distances: the base's (for the middle layer) and the middle layer's (for the finest).
@group(0) @binding(10) var far_prev0: texture_2d<f32>;
@group(0) @binding(21) var far_prev1: texture_2d<f32>;

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

fn put_far(at: vec2<i32>, v: vec4<f32>) {
    switch layer_now {
        case 0u { textureStore(how_far0, at, v); }
        case 1u { textureStore(how_far1, at, v); }
        default { textureStore(how_far2, at, v); }
    }
}

// This layer's word for the layer below (the finest has none below and never writes).
fn put_worth(i: u32, v: u32) {
    worth_out[i + select(0u, base_pixels(), layer_now == 1u)] = v;
}

// The layer above's word (the middle layer reads the base's, the finest the middle layer's).
fn get_worth(i: u32) -> u32 {
    return worth_in[i + select(0u, base_pixels(), layer_now == 2u)];
}

fn far_above_edge() -> vec2<i32> {
    if (layer_now == 2u) {
        return vec2<i32>(textureDimensions(far_prev1)) - 1;
    }
    return vec2<i32>(textureDimensions(far_prev0)) - 1;
}

fn get_far_above(at: vec2<i32>) -> f32 {
    if (layer_now == 2u) {
        return textureLoad(far_prev1, at, 0).x;
    }
    return textureLoad(far_prev0, at, 0).x;
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

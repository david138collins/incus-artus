// ── Sight's upscale (the regroup, phase 3.2; the screen pyramid, David 2026-11-06) ──
//
// Each screen pixel shows the finest layer of the pyramid marched there (8 × 8 everywhere; 4 × 4, 2 × 2 and 1 × 1 where
// they were worth it), brought up crisp (a layer's pixel a square of screen pixels), smooth (blended between a
// layer's pixels), or smooth within a surface but sharp at a skyline (where the base layer's pixels round it are
// at very different distances). And the depth the objects are drawn into (build 96): each screen pixel's, from the
// finest layer shown there (more than half of it), where its rays first touched anything.

struct Up {
    // x, y: the screen's size; z: the finest layer's factor (8, 4, 2 or 1); w: the coarsest's, the base (David
    // 2026-11-08: a choice; a layer coarser than it isn't made, and its binding stands in for it).
    size: vec4<f32>,
    // x: the mode (0 crisp, 1 smooth, 2 sharp at skylines); y: 1 if there's a layer of 1.5 (between 2 × 2 and
    // 1 × 1, or the finest).
    mode: vec4<f32>,
    // The depth (build 96, `models.rs`): x, y the screen's half width and half height at a block's distance; z the
    // nearest drawn (blocks).
    lens: vec4<f32>,
}

@group(0) @binding(0) var layer8: texture_2d<f32>;
@group(0) @binding(1) var layer4: texture_2d<f32>;
@group(0) @binding(2) var layer2: texture_2d<f32>;
@group(0) @binding(3) var far8: texture_2d<f32>;
@group(0) @binding(4) var blended: sampler;
@group(0) @binding(5) var<uniform> up: Up;
@group(0) @binding(6) var layer1: texture_2d<f32>;
@group(0) @binding(7) var layer15: texture_2d<f32>;
// Each finer layer's distances (where its rays first touched anything): the depth's.
@group(0) @binding(8) var far4: texture_2d<f32>;
@group(0) @binding(9) var far2: texture_2d<f32>;
@group(0) @binding(10) var far1: texture_2d<f32>;
@group(0) @binding(11) var far15: texture_2d<f32>;

struct Shown {
    @location(0) colour: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

// A layer's distance at this screen pixel if more than half of it is shown here (crisp, its own pixel), else `had`.
fn nearer(layer: texture_2d<f32>, far: texture_2d<f32>, factor: f32, at: vec2<f32>, had: f32) -> f32 {
    let size = vec2<i32>(textureDimensions(layer));
    let p = clamp(vec2<i32>(floor(at / factor)), vec2<i32>(0), size - 1);
    if (textureLoad(layer, p, 0).a < 0.5) {
        return had;
    }
    return textureLoad(far, p, 0).x;
}

@vertex
fn whole_screen(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((k << 1u) & 2u), f32(k & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

// A layer of `factor` at this screen pixel: its colour and how much of it is shown (kept premultiplied: a pixel
// not marched shows nothing), crisp, or blended between its pixels.
fn read(layer: texture_2d<f32>, factor: f32, at: vec2<f32>, smooth_it: bool) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(layer));
    let q = at / factor;
    var c = textureLoad(layer, clamp(vec2<i32>(floor(q)), vec2<i32>(0), size - 1), 0);
    if (smooth_it) {
        c = textureSampleLevel(layer, blended, q / vec2<f32>(size), 0.0);
    }
    if (c.a < 0.001) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(c.rgb / c.a, c.a);
}

@fragment
fn brought_up(@builtin(position) at: vec4<f32>) -> Shown {
    let mode = u32(up.mode.x);
    var smooth_it = mode != 0u;
    let finest = up.size.z;
    let coarsest = up.size.w;
    if (mode == 2u) {
        // Sharp at a skyline: the base layer's four pixels round here far apart in distance (`far8` is the base's,
        // whichever layer that is).
        let size = vec2<i32>(textureDimensions(far8));
        let b = vec2<i32>(floor(at.xy / coarsest - 0.5));
        var near = 1e30;
        var far = 0.0;
        for (var k = 0; k < 4; k++) {
            let d = textureLoad(far8, clamp(b + vec2<i32>(k & 1, k >> 1), vec2<i32>(0), size - 1), 0).x;
            near = min(near, d);
            far = max(far, d);
        }
        smooth_it = !(far > near * 1.5 + 8.0);
    }
    // The base, then each finer layer over it as much as it's worth there (the base is shown whole: its alpha 1).
    var colour = vec3<f32>(0.0);
    // The base's distance (`far8` is the base's, whichever layer that is), then each finer layer's where it shows.
    let base = vec2<i32>(textureDimensions(far8));
    var distance = textureLoad(far8, clamp(vec2<i32>(floor(at.xy / coarsest)), vec2<i32>(0), base - 1), 0).x;
    if (coarsest >= 8.0) {
        colour = read(layer8, 8.0, at.xy, smooth_it).rgb;
    }
    if (finest <= 4.0 && coarsest >= 4.0) {
        let c = read(layer4, 4.0, at.xy, smooth_it);
        colour = mix(colour, c.rgb, c.a);
        if (coarsest > 4.0) {
            distance = nearer(layer4, far4, 4.0, at.xy, distance);
        }
    }
    if (finest <= 2.0 && coarsest >= 2.0) {
        let c = read(layer2, 2.0, at.xy, smooth_it);
        colour = mix(colour, c.rgb, c.a);
        if (coarsest > 2.0) {
            distance = nearer(layer2, far2, 2.0, at.xy, distance);
        }
    }
    // A layer of 1.5: its pixels don't line up with the screen's (crisp, one and two screen pixels wide by turns;
    // smooth, a little soft).
    if (up.mode.y > 0.5) {
        let c = read(layer15, 1.5, at.xy, smooth_it);
        colour = mix(colour, c.rgb, c.a);
        distance = nearer(layer15, far15, 1.5, at.xy, distance);
    }
    if (finest <= 1.0) {
        let c = read(layer1, 1.0, at.xy, false);
        colour = mix(colour, c.rgb, c.a);
        if (coarsest > 1.0) {
            distance = nearer(layer1, far1, 1.0, at.xy, distance);
        }
    }
    // The depth: the nearest drawn over how far ahead it is (the distance along this pixel's ray, its slant taken
    // off), as the objects' own (`models.wgsl`); nearer is more.
    let screen = (at.xy / up.size.xy) * 2.0 - 1.0;
    let slant = length(vec3<f32>(1.0, screen.x * up.lens.x, screen.y * up.lens.y));
    var out: Shown;
    out.colour = vec4<f32>(colour, 1.0);
    out.depth = clamp(up.lens.z * slant / max(distance, 1e-3), 0.0, 1.0);
    return out;
}

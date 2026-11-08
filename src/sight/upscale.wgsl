// ── Sight's upscale (the regroup, phase 3.2; the screen pyramid, David 2026-11-06) ──
//
// Each screen pixel shows the finest layer of the pyramid marched there (8 × 8 everywhere; 4 × 4, 2 × 2 and 1 × 1 where
// they were worth it), brought up crisp (a layer's pixel a square of screen pixels), smooth (blended between a
// layer's pixels), or smooth within a surface but sharp at a skyline (where the base layer's pixels round it are
// at very different distances).

struct Up {
    // x, y: the screen's size; z: the finest layer's factor (8, 4, 2 or 1); w: the coarsest's, the base (David
    // 2026-11-08: a choice; a layer coarser than it isn't made, and its binding stands in for it).
    size: vec4<f32>,
    // x: the mode (0 crisp, 1 smooth, 2 sharp at skylines).
    mode: vec4<f32>,
}

@group(0) @binding(0) var layer8: texture_2d<f32>;
@group(0) @binding(1) var layer4: texture_2d<f32>;
@group(0) @binding(2) var layer2: texture_2d<f32>;
@group(0) @binding(3) var far8: texture_2d<f32>;
@group(0) @binding(4) var blended: sampler;
@group(0) @binding(5) var<uniform> up: Up;
@group(0) @binding(6) var layer1: texture_2d<f32>;

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
fn brought_up(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
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
    if (coarsest >= 8.0) {
        colour = read(layer8, 8.0, at.xy, smooth_it).rgb;
    }
    if (finest <= 4.0 && coarsest >= 4.0) {
        let c = read(layer4, 4.0, at.xy, smooth_it);
        colour = mix(colour, c.rgb, c.a);
    }
    if (finest <= 2.0 && coarsest >= 2.0) {
        let c = read(layer2, 2.0, at.xy, smooth_it);
        colour = mix(colour, c.rgb, c.a);
    }
    if (finest <= 1.0) {
        let c = read(layer1, 1.0, at.xy, false);
        colour = mix(colour, c.rgb, c.a);
    }
    return vec4<f32>(colour, 1.0);
}

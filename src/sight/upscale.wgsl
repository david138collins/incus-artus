// ── Sight's upscale (the regroup, phase 3.2): the coarse layer brought up to the screen ──
//
// Crisp (each coarse pixel a square of screen pixels), smooth (blended between coarse pixels), or smooth
// within a surface but sharp at a skyline (where the coarse pixels round it are at very different distances).

struct Up {
    // x, y: the screen's size; z, w: the coarse layer's.
    size: vec4<f32>,
    // x: the mode (0 crisp, 1 smooth, 2 sharp at skylines); y: the grid's factor.
    mode: vec4<f32>,
}

@group(0) @binding(0) var coarse: texture_2d<f32>;
@group(0) @binding(1) var coarse_far: texture_2d<f32>;
@group(0) @binding(2) var blended: sampler;
@group(0) @binding(3) var<uniform> up: Up;

@vertex
fn whole_screen(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((k << 1u) & 2u), f32(k & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn brought_up(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let q = at.xy / up.mode.y;
    let size = vec2<i32>(up.size.zw);
    let crisp = textureLoad(coarse, clamp(vec2<i32>(floor(q)), vec2<i32>(0), size - 1), 0);
    let mode = u32(up.mode.x);
    if (mode == 0u) {
        return crisp;
    }
    let blended_up = textureSampleLevel(coarse, blended, q / up.size.zw, 0.0);
    if (mode == 1u) {
        return blended_up;
    }
    // Sharp at a skyline: the four coarse pixels the blend reads, far apart in distance, the crisp one.
    let b = vec2<i32>(floor(q - 0.5));
    var near = 1e30;
    var far = 0.0;
    for (var k = 0; k < 4; k++) {
        let d = textureLoad(coarse_far, clamp(b + vec2<i32>(k & 1, k >> 1), vec2<i32>(0), size - 1), 0).x;
        near = min(near, d);
        far = max(far, d);
    }
    if (far > near * 1.5 + 8.0) {
        return crisp;
    }
    return blended_up;
}

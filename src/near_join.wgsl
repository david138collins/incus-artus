// The x-fader (plan 2.3, build 199): the near eye's own picture laid over the far picture, a pixel at a time. The near
// picture is kept already multiplied by how much of it shows (`near.wgsl`'s `near_shows`: all of a block's pixel
// before the x-fader's band, falling to none at the near faces' reach along its curve), so this pass only lays it
// over what the far eye drew.

@group(0) @binding(0) var t_near: texture_2d<f32>;
// x: the near picture's pixel's side in screen pixels (1, 2 or 4: build 212). Each of its pixels is laid over as a
// square that size, counted from the screen's top left corner (the far eye's tiles' corner), with no smoothing.
@group(0) @binding(1) var<uniform> pixel: vec4<u32>;

// One triangle over the whole screen.
@vertex
fn near_whole(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(f32(k & 1u) * 4.0 - 1.0, f32(k >> 1u) * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn near_over(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(t_near, vec2<i32>(vec2<u32>(at.xy) / max(pixel.x, 1u)), 0);
}

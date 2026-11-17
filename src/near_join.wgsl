// The x-fader (plan 2.3, build 199): the near eye's own picture laid over the far picture, a pixel at a time. The near
// picture is kept already multiplied by how much of it shows (`near.wgsl`'s `near_shows`: all of a block's pixel
// before the x-fader's band, falling to none at the near faces' reach along its curve), so this pass only lays it
// over what the far eye drew.

@group(0) @binding(0) var t_near: texture_2d<f32>;
// The near eye's bands (build 213): `t_near` the full-size picture (the farthest band's), `t_before` the band just
// nearer than it and `t_first` the one before that. `pixel`: each one's pixel's side in screen pixels, in that order
// (0: no such band). Each picture's pixel is laid over as a square that size, counted from the screen's top left
// corner (the far eye's tiles' corner), with no smoothing; a nearer band over a farther.
@group(0) @binding(1) var<uniform> pixel: vec4<u32>;
@group(0) @binding(2) var t_before: texture_2d<f32>;
@group(0) @binding(3) var t_first: texture_2d<f32>;

// One triangle over the whole screen.
@vertex
fn near_whole(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(f32(k & 1u) * 4.0 - 1.0, f32(k >> 1u) * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn near_over(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let on_screen = vec2<u32>(at.xy);
    // (Each is kept multiplied by how much of it shows: laid over, it leaves that much less of what's under it.)
    var laid = textureLoad(t_near, vec2<i32>(on_screen / max(pixel.x, 1u)), 0);
    if (pixel.y > 0u) {
        let over = textureLoad(t_before, vec2<i32>(on_screen / pixel.y), 0);
        laid = over + laid * (1.0 - over.a);
    }
    if (pixel.z > 0u) {
        let over = textureLoad(t_first, vec2<i32>(on_screen / pixel.z), 0);
        laid = over + laid * (1.0 - over.a);
    }
    return laid;
}

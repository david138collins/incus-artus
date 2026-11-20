// The 95 drawing's sheets on the screen (`on_card.rs`): each a rectangle its own size times its scale at its place, a
// pixel of it a square of the screen's, read whole (no filtering, no blending); where it's clear, what's under it shows.

struct Place {
    // Its top left on the screen, and its size in its own pixels.
    origin: vec2<f32>,
    size: vec2<f32>,
    // The screen's size, and how many of its pixels a sheet's pixel is across.
    screen: vec2<f32>,
    scale: f32,
    unused: f32,
}

@group(0) @binding(0) var sheet: texture_2d<f32>;
@group(0) @binding(1) var<uniform> place: Place;

@vertex
fn corners(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    // Two triangles: corners 0, 1, 2 and 2, 1, 3 of the rectangle.
    var corner = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let pixel = place.origin + corner[index] * place.size * place.scale;
    return vec4(pixel.x / place.screen.x * 2.0 - 1.0, 1.0 - pixel.y / place.screen.y * 2.0, 0.0, 1.0);
}

@fragment
fn pixel(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<i32>(place.size);
    let texel = clamp(vec2<i32>(floor((at.xy - place.origin) / place.scale)), vec2(0), size - vec2(1));
    let colour = textureLoad(sheet, texel, 0);
    if colour.a < 0.5 {
        discard;
    }
    return vec4(colour.rgb, 1.0);
}

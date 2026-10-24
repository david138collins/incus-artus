// Distant terrain shader: draws the far-away landscape (see src/render/distant.rs).
//
// Each corner carries its position and a colour (the block's average colour, already shaded by
// which way the face points). The fragment part:
//   - draws nothing over a real chunk that's on screen (the `shown` texture marks them, one pixel
//     per chunk), so the two never overlap, and where real chunks haven't loaded yet the distant
//     terrain still shows: no holes when you fly fast;
//   - fades into the sky colour with distance, exactly like the near terrain's fog.

import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::view;

struct DistantShaderSettings {
    fog_color: vec4<f32>,
    // x fog start, y fog end, z fog on, w unused
    fog: vec4<f32>,
    // xy = the chunk (game axes) at the middle of the `shown` texture
    shown_centre: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: DistantShaderSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var shown: texture_2d<f32>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) colour: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) colour: vec4<f32>,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let world = get_world_from_local(in.instance_index) * vec4<f32>(in.position, 1.0);
    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world.xyz);
    out.world_position = world.xyz;
    out.colour = in.colour;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Which chunk this is in (game x = -Bevy z, game y = Bevy x), and so which pixel of `shown`.
    let block = vec2<f32>(-in.world_position.z, in.world_position.x);
    let size = vec2<i32>(textureDimensions(shown));
    let pixel = vec2<i32>(floor(block / 32.0)) - vec2<i32>(settings.shown_centre.xy) + size / 2;
    if all(pixel >= vec2<i32>(0)) && all(pixel < size) && textureLoad(shown, pixel, 0).r > 0.5 {
        discard;
    }
    var colour = in.colour.rgb;
    if settings.fog.z > 0.5 {
        let distance = length(in.world_position - view.world_position);
        colour = mix(colour, settings.fog_color.rgb, smoothstep(settings.fog.x, settings.fog.y, distance));
    }
    return vec4<f32>(colour, 1.0);
}

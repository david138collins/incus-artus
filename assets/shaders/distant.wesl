// Distant terrain shader: draws the far-away landscape (see src/render/distant.rs).
//
// Each corner carries its position and a colour (the block's average colour, already shaded by
// which way the face points). The fragment part:
//   - draws nothing close to the camera, inside the circle where real chunks are drawn, so the
//     two never overlap: the hand-off is a clean line that follows you;
//   - fades into the sky colour with distance, exactly like the near terrain's fog.

import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::view;

struct DistantShaderSettings {
    fog_color: vec4<f32>,
    // x fog start, y fog end, z fog on, w unused
    fog: vec4<f32>,
    // x = no distant terrain closer than this (blocks, measured across the ground)
    near: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: DistantShaderSettings;

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
    // Across the ground only (Bevy's x and z): flying high doesn't change where the hand-off is.
    let across = in.world_position.xz - view.world_position.xz;
    if length(across) < settings.near.x {
        discard;
    }
    var colour = in.colour.rgb;
    if settings.fog.z > 0.5 {
        let distance = length(in.world_position - view.world_position);
        colour = mix(colour, settings.fog_color.rgb, smoothstep(settings.fog.x, settings.fog.y, distance));
    }
    return vec4<f32>(colour, 1.0);
}

// Terrain shader: draws block faces.
//
// Runs on the graphics card. Two parts:
//   vertex   — once per corner: unpack the 8-byte vertex, work out where it lands on screen
//   fragment — once per pixel: look up the texture, apply face shading and fog
//
// Positions arrive in GAME axes inside the section (X north, Y east, Z up, 0..32) and are
// converted to Bevy's axes here, matching src/coords.rs: bevy = (game.y, game.z, -game.x).

import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::view;

struct TerrainShaderSettings {
    // x top, y bottom, z north/south, w east/west
    face_shade: vec4<f32>,
    fog_color: vec4<f32>,
    // x fog start, y fog end, z fog on, w debug view
    fog_and_debug: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var block_textures: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var block_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var<uniform> settings: TerrainShaderSettings;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) packed: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    // "flat" = the same value across the whole triangle, no blending between corners.
    @location(2) @interpolate(flat) layer: u32,
    @location(3) @interpolate(flat) face: u32,
    @location(4) @interpolate(flat) quad: u32,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let a = in.packed.x;
    let b = in.packed.y;
    let gx = f32(a & 63u);
    let gy = f32((a >> 6u) & 63u);
    let gz = f32((a >> 12u) & 63u);
    let face = (a >> 18u) & 7u;

    let local = vec3<f32>(gy, gz, -gx);
    let world = get_world_from_local(in.instance_index) * vec4<f32>(local, 1.0);

    // Texture coordinates come straight from the position, so textures tile once per block.
    // Side faces: image top = up (+Z). Top/bottom faces: image top = north (+X).
    // "Right" on the image is chosen so nothing appears mirrored when seen from outside.
    var uv: vec2<f32>;
    switch face {
        case 0u: { uv = vec2<f32>(-gy, -gz); }  // north face: seen from the north, east is on the left
        case 1u: { uv = vec2<f32>(gy, -gz); }   // south face
        case 2u: { uv = vec2<f32>(gx, -gz); }   // east face
        case 3u: { uv = vec2<f32>(-gx, -gz); }  // west face
        case 4u: { uv = vec2<f32>(gy, -gx); }   // top
        default: { uv = vec2<f32>(-gy, -gx); }  // bottom
    }

    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world.xyz);
    out.uv = uv;
    out.world_position = world.xyz;
    out.layer = b & 0xFFFFu;
    out.face = face;
    out.quad = b >> 16u;
    return out;
}

// A quick "random" colour from a number (only for the debug views).
fn debug_colour(n: u32) -> vec3<f32> {
    let h = (n * 2654435761u) >> 8u;
    return vec3<f32>(f32(h & 255u), f32((h >> 8u) & 255u), f32((h >> 16u) & 255u)) / 255.0;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var colour = textureSample(block_textures, block_sampler, in.uv, in.layer).rgb;

    let debug_view = u32(settings.fog_and_debug.w);
    if debug_view == 1u {
        colour = debug_colour(in.quad + 1u);
    } else if debug_view == 2u {
        colour = debug_colour(in.face + 100u);
    }

    // Classic block shading: brightness by which way the face points.
    var shade = settings.face_shade.x;
    if in.face == 5u {
        shade = settings.face_shade.y;
    } else if in.face <= 1u {
        shade = settings.face_shade.z;
    } else if in.face <= 3u {
        shade = settings.face_shade.w;
    }
    colour = colour * shade;

    // Fog: fade into the sky colour with distance.
    if settings.fog_and_debug.z > 0.5 {
        let distance = length(in.world_position - view.world_position);
        let amount = smoothstep(settings.fog_and_debug.x, settings.fog_and_debug.y, distance);
        colour = mix(colour, settings.fog_color.rgb, amount);
    }

    return vec4<f32>(colour, 1.0);
}

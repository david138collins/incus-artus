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
    // x random texture rotation on
    options: vec4<f32>,
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
    // Position inside the section, game axes (0..32).
    @location(0) local: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    // "flat" = the same value across the whole triangle, no blending between corners.
    @location(2) @interpolate(flat) layer: u32,
    @location(3) @interpolate(flat) face: u32,
    @location(4) @interpolate(flat) quad: u32,
    @location(5) @interpolate(flat) random_rotation: u32,
    // The section's south-west bottom corner in the world, game axes.
    @location(6) @interpolate(flat) section_origin: vec3<i32>,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let a = in.packed.x;
    let b = in.packed.y;
    let game_local = vec3<f32>(f32(a & 63u), f32((a >> 6u) & 63u), f32((a >> 12u) & 63u));

    let world_from_local = get_world_from_local(in.instance_index);
    let local = vec3<f32>(game_local.y, game_local.z, -game_local.x);
    let world = world_from_local * vec4<f32>(local, 1.0);
    // The section's placement (Bevy axes), turned back into game axes.
    let origin = world_from_local[3].xyz;

    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world.xyz);
    out.local = game_local;
    out.world_position = world.xyz;
    out.layer = b & 0xFFFFu;
    out.face = (a >> 18u) & 7u;
    out.quad = b >> 16u;
    out.random_rotation = (a >> 21u) & 1u;
    out.section_origin = vec3<i32>(round(vec3<f32>(-origin.z, origin.x, origin.y)));
    return out;
}

// Texture coordinates come straight from the position, so textures tile once per block.
// Side faces: image top = up (+Z). Top/bottom faces: image top = north (+X).
// "Right" on the image is chosen so nothing appears mirrored when seen from outside.
fn face_uv(face: u32, p: vec3<f32>) -> vec2<f32> {
    switch face {
        case 0u: { return vec2<f32>(-p.y, -p.z); }  // north face: seen from the north, east is on the left
        case 1u: { return vec2<f32>(p.y, -p.z); }   // south face
        case 2u: { return vec2<f32>(p.x, -p.z); }   // east face
        case 3u: { return vec2<f32>(-p.x, -p.z); }  // west face
        case 4u: { return vec2<f32>(p.y, -p.x); }   // top
        default: { return vec2<f32>(-p.y, -p.x); }  // bottom
    }
}

// Which way each face points (game axes).
fn face_normal(face: u32) -> vec3<f32> {
    switch face {
        case 0u: { return vec3<f32>(1.0, 0.0, 0.0); }
        case 1u: { return vec3<f32>(-1.0, 0.0, 0.0); }
        case 2u: { return vec3<f32>(0.0, 1.0, 0.0); }
        case 3u: { return vec3<f32>(0.0, -1.0, 0.0); }
        case 4u: { return vec3<f32>(0.0, 0.0, 1.0); }
        default: { return vec3<f32>(0.0, 0.0, -1.0); }
    }
}

// Scramble a block position into a "random" number. The same block always gives the same number.
fn block_hash(p: vec3<i32>, face: u32) -> u32 {
    var h = (bitcast<u32>(p.x) * 73856093u) ^ (bitcast<u32>(p.y) * 19349663u)
        ^ (bitcast<u32>(p.z) * 83492791u) ^ (face * 2654435761u);
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return h;
}

// A quick "random" colour from a number (only for the debug views).
fn debug_colour(n: u32) -> vec3<f32> {
    let h = (n * 2654435761u) >> 8u;
    return vec3<f32>(f32(h & 255u), f32((h >> 8u) & 255u), f32((h >> 16u) & 255u)) / 255.0;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_uv = face_uv(in.face, in.local);
    // How fast the texture position changes across the screen: picks the distant (mip) version.
    // Taken from the unrotated position, so turning a block never changes its blurriness.
    let uv_dx = dpdx(base_uv);
    let uv_dy = dpdy(base_uv);

    var uv = base_uv;
    if in.random_rotation == 1u && settings.options.x > 0.5 {
        // The block this pixel belongs to: step half a block back from the face, into the block.
        let in_block = in.local - face_normal(in.face) * 0.5;
        let block = in.section_origin + vec3<i32>(floor(in_block));
        let turns = (block_hash(block, in.face) >> 8u) & 3u;
        // Turn the image around the block's centre in quarter turns.
        let cell = floor(uv);
        var f = uv - cell;
        switch turns {
            case 1u: { f = vec2<f32>(1.0 - f.y, f.x); }
            case 2u: { f = vec2<f32>(1.0 - f.x, 1.0 - f.y); }
            case 3u: { f = vec2<f32>(f.y, 1.0 - f.x); }
            default: {}
        }
        uv = cell + f;
    }

    let sampled = textureSampleGrad(block_textures, block_sampler, uv, in.layer, uv_dx, uv_dy);
    // See-through blocks (like the grid): pixels less than half opaque are cut out entirely.
    // Ordinary block textures are fully opaque, so this never affects them.
    if sampled.a < 0.5 {
        discard;
    }
    var colour = sampled.rgb;

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

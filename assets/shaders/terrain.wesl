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
    // Survey view: x on, y every how many levels, z darkness, w sea level
    survey: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var block_textures: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var block_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var<uniform> settings: TerrainShaderSettings;

// How the ground is lit (src/render/light.rs).
struct TerrainLight {
    // The world light map: x on, y shadows, z sky, w relief (strengths)
    light: vec4<f32>,
    // x the map's half width, y the light map's width (blocks)
    light_area: vec4<f32>,
    // xyz toward the sun (game axes), w hill shading strength
    sun: vec4<f32>,
    // The window of the ground around you: x north and y east of its north-west corner, z its side
    // (blocks), w on
    near_area: vec4<f32>,
    // x height tint, y the distance slopes are measured over (blocks), z sea level, w the highest a
    // height can be
    hill: vec4<f32>,
    // x how hill shading fades toward its limits (see distant.rs, `soft_limit`), y foot shading, z
    // step shadows (strengths), w unused
    shade: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var light_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(5) var near_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(6) var near_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(7) var<uniform> lighting: TerrainLight;
// The near window's cone map (src/render/light.rs, "The cone map"): x all round, y from the sun's
// side; plain numbers, blended here.
@group(constants::MATERIAL_BIND_GROUP) @binding(8) var near_cones: texture_2d<f32>;

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

// A value stored square-rooted each way around 0.5 (see light.rs), back to -1 to 1.
fn unroot(v: f32) -> f32 {
    let b = v * 2.0 - 1.0;
    return sign(b) * b * b;
}

// Hill shading's light, `raw` (1 = flat ground), eased toward its darkest (0.35) and brightest
// (1.6) instead of stopped dead, more gradually the lower `fade` is. The same sums as
// src/render/distant.rs, `soft_limit`, so near and far match.
fn soft_limit(raw: f32, fade: f32) -> f32 {
    let d = raw - 1.0;
    let limit = select(1.6 - 1.0, 1.0 - 0.35, d < 0.0);
    // Never quite 0: pow(0, p) isn't defined on every graphics card.
    let x = max(abs(d) / limit, 1e-6);
    let p = clamp(fade, 0.5, 16.0);
    return 1.0 + sign(d) * limit * x / pow(1.0 + pow(x, p), 1.0 / p);
}

// The cone map at `uv` (0 to 1 across the window), blended between its four nearest pixels.
fn cones_at(uv: vec2<f32>) -> vec2<f32> {
    let size = vec2<i32>(textureDimensions(near_cones));
    let p = uv * vec2<f32>(size) - 0.5;
    let base = floor(p);
    let t = p - base;
    let low = clamp(vec2<i32>(base), vec2<i32>(0), size - 1);
    let high = clamp(vec2<i32>(base) + 1, vec2<i32>(0), size - 1);
    let a = textureLoad(near_cones, vec2<i32>(low.x, low.y), 0).xy;
    let b = textureLoad(near_cones, vec2<i32>(high.x, low.y), 0).xy;
    let c = textureLoad(near_cones, vec2<i32>(low.x, high.y), 0).xy;
    let d = textureLoad(near_cones, vec2<i32>(high.x, high.y), 0).xy;
    return mix(mix(a, b, t.x), mix(c, d, t.x), t.y);
}

// How much a foot `below` blocks under the cones round it is shaded, 0 to 1: about as much as it
// is below, for small amounts, easing toward 1 by `fade` (as `soft_limit` does).
fn eased(below: f32, fade: f32) -> f32 {
    let x = max(below, 1e-6);
    let p = clamp(fade, 0.5, 16.0);
    return select(x / pow(1.0 + pow(x, p), 1.0 / p), 0.0, below <= 0.0);
}

// How lit this spot is (1 = as painted), the same way the distant terrain is lit, so the two match:
//   - the world light map (shadows, valley darkening, relief), on every face, as distant.wesl does;
//   - hill shading on tops, from the slope of the ground under them (a block world's tops are all
//     flat, so which way the face points says nothing about the lie of the land), with the same
//     sums as the distant tiles (src/render/distant.rs, `Shade::hill`), and the height tint.
// Faces well below the ground (caves, overhangs) keep their plain light, for now.
fn ground_light(world_position: vec3<f32>, face: u32) -> f32 {
    let north = -world_position.z;
    let east = world_position.x;
    let up = world_position.y;
    var light = 1.0;

    if lighting.light.x > 0.5 {
        let uv = vec2<f32>(east + lighting.light_area.x, lighting.light_area.x - north) / max(lighting.light_area.y, 1.0);
        let map = textureSampleLevel(light_map, light_sampler, uv, 0.0);
        let sun = 1.0 - lighting.light.y * (1.0 - map.r);
        let sky = 1.0 - min(lighting.light.z * map.g * map.g, 0.9);
        let relief = max(1.0 + lighting.light.w * unroot(map.b), 0.2);
        light = sun * mix(1.0, sky * relief, map.a);
    }

    if lighting.near_area.w > 0.5 {
        let uv = vec2<f32>(east - lighting.near_area.y, lighting.near_area.x - north) / lighting.near_area.z;
        let near = textureSampleLevel(near_map, near_sampler, uv, 0.0);
        if all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0)) && near.a > 0.5 {
            let sea = lighting.hill.z;
            let surface = sea + near.b * (lighting.hill.w - sea);
            if face == 4u {
                // The rise across one distant cell, as the distant tiles measure it.
                let rise = vec2<f32>(unroot(near.r), unroot(near.g)) * 4.0 * lighting.hill.y;
                let normal = normalize(vec3<f32>(-rise.x * lighting.sun.w, -rise.y * lighting.sun.w, 1.0));
                let hill = soft_limit(dot(normal, lighting.sun.xyz) / lighting.sun.z, lighting.shade.x);
                // The feet of steps: how far this top is below the cones of the ground round it
                // (0 on a top, a rim or open ground), eased like the hill shading.
                let cones = cones_at(uv);
                let fade = lighting.shade.x;
                let feet = (1.0 - lighting.shade.y * eased(cones.x - up, fade)) * (1.0 - lighting.shade.z * eased(cones.y - up, fade));
                let tint = 1.0 + lighting.hill.x * (up - sea) / (lighting.hill.w - sea);
                // The sea's surface stays flat, as on the distant terrain.
                let water = surface <= sea + 0.5 && up <= sea + 0.5;
                light = light * select(hill * tint * feet, 1.0, water);
            }
            // Well below the ground: plain light.
            light = mix(1.0, light, smoothstep(surface - 24.0, surface - 12.0, up));
        }
    }
    return light;
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
    colour = colour * shade * ground_light(in.world_position, in.face);

    // Survey view: every Nth level of blocks above sea level darkened, like a contour map's bands.
    if settings.survey.x > 0.5 {
        let in_block = in.local - face_normal(in.face) * 0.5;
        let z = in.section_origin.z + i32(floor(in_block.z));
        let every = max(i32(settings.survey.y), 1);
        let level = z - i32(settings.survey.w);
        if ((level % every) + every) % every == 0 {
            colour = colour * (1.0 - settings.survey.z);
        }
    }

    // Fog: fade into the sky colour with distance.
    if settings.fog_and_debug.z > 0.5 {
        let distance = length(in.world_position - view.world_position);
        let amount = smoothstep(settings.fog_and_debug.x, settings.fog_and_debug.y, distance);
        colour = mix(colour, settings.fog_color.rgb, amount);
    }

    return vec4<f32>(colour, 1.0);
}

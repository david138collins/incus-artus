// Terrain shader: draws block faces.
//
// Runs on the graphics card. Two parts:
//   vertex   — once per corner: unpack the 8-byte vertex, work out where it lands on screen
//   fragment — once per pixel: look up the texture, apply face shading and fog
//
// Positions arrive in GAME axes inside the section (X north, Y east, Z up, 0..32; in half blocks in the
// vertex, 0..64, halved here) and are
// converted to Bevy's axes here, matching src/coords.rs: bevy = (game.y, game.z, -game.x).

import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::{globals, view};
// The settings, the light and the blocks' shading are lighting.wesl's, shared with the march's whole
// copies (march.wesl).
import super::lighting::{face_normal, face_shade, face_uv, ground_lit, land, light_from_map, to_game, turned, underground_lit, unroot, TerrainLight, TerrainShaderSettings};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var block_textures: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var block_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var<uniform> settings: TerrainShaderSettings;

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
    @location(0) packed: vec3<u32>,
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
    // Daylight at this corner, 0 (black) to 1 (full): src/render/sky_light.rs.
    @location(7) daylight: f32,
    // Placed light (red, green, blue, 0 to 1): src/render/placed_light.rs.
    @location(8) @interpolate(flat) placed: vec3<f32>,
    // How many blocks a cell is: 1, or a far copy's 2, 4 or 8 (src/render/far_copies.rs).
    @location(9) @interpolate(flat) scale: f32,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    let a = in.packed.x;
    let b = in.packed.y;
    // In half blocks (src/render/mesher.rs), so a cell's pieces sit at half-block corners.
    let corner = vec3<f32>(f32(a & 127u), f32((a >> 7u) & 127u), f32((a >> 14u) & 127u)) * 0.5;
    let face = (a >> 21u) & 7u;

    let world_from_local = get_world_from_local(in.instance_index);
    // Nudged a hair out of its rectangle, in the face's plane (src/render/mesher.rs: which way is
    // out at this corner), about a third of a pixel at this distance: neighbouring rectangles
    // overlap, so no pixel-thin cracks open where a big one's edge meets small ones' corners.
    let out_of = vec3<f32>(f32((a >> 25u) & 1u), f32((a >> 26u) & 1u), f32((a >> 27u) & 1u)) * 2.0 - 1.0;
    let in_plane = out_of * (1.0 - abs(face_normal(face)));
    let at = world_from_local * vec4<f32>(vec3<f32>(corner.y, corner.z, -corner.x), 1.0);
    let hair = max(0.0005, 0.0006 * length(at.xyz - view.world_position));
    let game_local = corner + in_plane * hair;
    let local = vec3<f32>(game_local.y, game_local.z, -game_local.x);
    let world = world_from_local * vec4<f32>(local, 1.0);
    // The section's placement (Bevy axes), turned back into game axes.
    let origin = world_from_local[3].xyz;

    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world.xyz);
    out.local = game_local;
    out.world_position = world.xyz;
    out.layer = b & 0xFFFFu;
    out.face = face;
    out.quad = b >> 16u;
    out.random_rotation = (a >> 24u) & 1u;
    // In the world's blocks: measured from the floating origin, plus it (src/coords.rs), so a block's
    // random turn is the same wherever the origin is.
    out.section_origin = vec3<i32>(round(vec3<f32>(-origin.z, origin.x, origin.y))) + vec3<i32>(i32(lighting.origin.x), i32(lighting.origin.y), 0);
    out.daylight = f32((a >> 28u) & 15u) / 15.0;
    let c = in.packed.z;
    out.placed = vec3<f32>(f32(c & 15u), f32((c >> 4u) & 15u), f32((c >> 8u) & 15u)) / 15.0;
    out.scale = length(world_from_local[0].xyz);
    return out;
}

// A quick "random" colour from a number (only for the debug views).
fn debug_colour(n: u32) -> vec3<f32> {
    let h = (n * 2654435761u) >> 8u;
    return vec3<f32>(f32(h & 255u), f32((h >> 8u) & 255u), f32((h >> 16u) & 255u)) / 255.0;
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

// How lit this spot is (1 = as painted), with the sums the distant terrain is lit with too
// (lighting.wesl, `ground_lit`), so the two match: the light map, the window of the ground round
// you and its cone map, read here.
fn ground_light(world_position: vec3<f32>, face: u32) -> f32 {
    let north = -world_position.z + lighting.origin.x;
    let east = world_position.x + lighting.origin.y;
    let up = world_position.y;
    var map_light = 1.0;
    if lighting.light.x > 0.5 {
        let uv = vec2<f32>(east + lighting.light_area.x, lighting.light_area.x - north) / max(lighting.light_area.y, 1.0);
        map_light = light_from_map(textureSampleLevel(light_map, light_sampler, uv, 0.0), lighting.light);
    }
    var near = vec4<f32>(0.0);
    var cones = vec2<f32>(0.0);
    if lighting.near_area.w > 0.5 {
        let uv = vec2<f32>(east - lighting.near_area.y, lighting.near_area.x - north) / lighting.near_area.z;
        near = textureSampleLevel(near_map, near_sampler, uv, 0.0);
        if !(all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0))) {
            near.a = 0.0;
        }
        if near.a > 0.5 && face == 4u {
            cones = cones_at(uv);
        }
    }
    return ground_lit(map_light, near, cones, face, up, lighting.sun, lighting.hill, lighting.shade, lighting.sides);
}

// One of the far terrain's steps' shade tables (`step` if `steps`, else `foot`) at a slope `rise`
// blocks up per block across, blended between its two nearest entries (as distant.wesl's `table_at`).
fn far_table_at(steps: bool, rise: f32) -> f32 {
    let k = min(rise / max(lighting.far.y, 1e-3) * 32.0, 32.0);
    let low = u32(floor(k));
    let high = min(low + 1u, 32u);
    let a = select(lighting.foot[low / 4u][low % 4u], lighting.step[low / 4u][low % 4u], steps);
    let b = select(lighting.foot[high / 4u][high % 4u], lighting.step[high / 4u][high % 4u], steps);
    return mix(a, b, fract(k));
}

// A far copy's plain ground (`far_copies.rs`: natural open land, nothing changed or carved in its
// column), drawn as the far terrain round it draws the land, so the copy doesn't show as a patch
// [David, build 5: "has different shading than the surrounding areas"]: the surface block's top
// and side colours (its textures' smallest versions, as the far terrain's are their averages), lit
// by the far terrain's sums (lighting.wesl, `land`) with the slope from the window round you, and
// the light map. Every face of the cell the same, as the far terrain's smooth surface has no steps.
fn far_ground(top_colour: vec3<f32>, side_colour: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {
    let north = -world_position.z + lighting.origin.x;
    let east = world_position.x + lighting.origin.y;
    var light = 1.0;
    if lighting.light.x > 0.5 {
        let uv = vec2<f32>(east + lighting.light_area.x, lighting.light_area.x - north) / max(lighting.light_area.y, 1.0);
        light = light_from_map(textureSampleLevel(light_map, light_sampler, uv, 0.0), lighting.light);
    }
    var slope = vec2<f32>(0.0);
    if lighting.near_area.w > 0.5 {
        let uv = vec2<f32>(east - lighting.near_area.y, lighting.near_area.x - north) / lighting.near_area.z;
        let near = textureSampleLevel(near_map, near_sampler, uv, 0.0);
        if all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0)) && near.a > 0.5 {
            // Stored as up to 4 blocks up per block across (src/render/light.rs, `STEEPEST`).
            slope = vec2<f32>(unroot(near.r), unroot(near.g)) * 4.0;
        }
    }
    let rise = length(slope);
    let faces = vec4<f32>(settings.face_shade.x, settings.face_shade.z, settings.face_shade.w, lighting.sides.x);
    return light * land(
        top_colour,
        side_colour,
        slope,
        world_position.y,
        to_game(normalize(view.world_position - world_position)),
        lighting.sun,
        vec3<f32>(lighting.hill.x, lighting.hill.z, lighting.shade.w),
        lighting.hill.y,
        lighting.shade.x,
        faces,
        lighting.far.x,
        far_table_at(false, rise),
        far_table_at(true, rise),
    );
}

// One along an axis (0 x, 1 y, 2 z), as a vector: picks a vector's part by a number worked out as the
// shader runs, which DirectX's compiler won't do by indexing (it quit the game at its first launch,
// 2026-11-01: "array reference cannot be used as an l-value").
fn unit(axis: u32) -> vec3<f32> {
    return vec3<f32>(f32(axis == 0u), f32(axis == 1u), f32(axis == 2u));
}

// A bar's own texture, measured from the bar's own corner (src/render/mesher.rs, `bar_tag`): `tag`
// says which octant of its cell this is (3 bits) and what it's part of (src/world/pieces.rs, `Part`:
// its bar's axis, 0 for a lone octant; which end of the bar it is; whether its other half is there).
// A long side's picture runs up the bar's length, whichever way the bar lies, and on across a cell's
// side for a bar placed across two cells; an end's covers the ½ × ½ end (every face of a lone octant).
// Each layer is the bar's picture with its pixels repeated to fill 32 × 32, so 0 to 1 across the bar's
// own width covers it.
fn bar_uv(face: u32, p: vec3<f32>, tag: u32) -> vec2<f32> {
    let o = tag & 7u;
    let part = tag >> 3u;
    let lone = (part & 3u) == 0u;
    let axis = select((part & 3u) - 1u, face / 2u, lone);
    let end = f32((part >> 2u) & 1u);
    let u = (axis + 1u) % 3u;
    let v = (axis + 2u) % 3u;
    // The octant's corner in its cell, and where its bar starts along its axis (its octant's place,
    // less half a block for the high end: before this cell, for half of a bar from the cell behind).
    let corner = vec3<f32>(f32(o & 1u), f32((o >> 1u) & 1u), f32((o >> 2u) & 1u)) * 0.5;
    let start = dot(corner, unit(axis)) - end * 0.5;
    // Where in its cell (0 to 1 each way), from a hair inside the face.
    let inside = p - face_normal(face) * 0.001;
    let c = inside - floor(inside);
    let along = dot(c, unit(axis)) - start;
    let across_u = dot(c - corner, unit(u)) * 2.0;
    let across_v = dot(c - corner, unit(v)) * 2.0;
    let facing = face / 2u;
    if facing == axis {
        return vec2<f32>(across_u, 1.0 - across_v);
    }
    // A long side: across is the bar's other axis lying in this face.
    return vec2<f32>(select(across_u, across_v, facing == u), 1.0 - along);
}

// How lit a face is underground (lighting.wesl, `underground_lit`).
fn underground_light(daylight: f32, placed: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {
    return underground_lit(daylight, placed, length(world_position - view.world_position), settings.underground, settings.placed, globals.time);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let bar = (in.quad & 0x8000u) != 0u;
    var base_uv = face_uv(in.face, in.local);
    let part = (in.quad & 0x7FFu) >> 3u;
    if bar && (part & 16u) != 0u {
        // A whole block on the octant grid (src/world/pieces.rs, `Part::of_block`): its own face,
        // slid by half a block along each way it's off the cells (the part's top 3 bits, from the
        // mesher's `bar_tag`), so its picture lines up with the block, not the cell.
        let off = part >> 5u;
        let slide = vec3<f32>(f32(off & 1u), f32((off >> 1u) & 1u), f32((off >> 2u) & 1u)) * 0.5;
        base_uv = face_uv(in.face, in.local - slide);
    } else if bar {
        // (Kept off the far edge, which the sampler would wrap back to the near one.)
        base_uv = clamp(bar_uv(in.face, in.local, in.quad & 0x7FFu), vec2<f32>(0.0), vec2<f32>(0.9999));
    }
    // How fast the texture position changes across the screen: picks the distant (mip) version.
    // Taken from the unrotated position, so turning a block never changes its blurriness.
    let uv_dx = dpdx(base_uv);
    let uv_dy = dpdy(base_uv);

    var uv = base_uv;
    if in.random_rotation == 1u && settings.options.x > 0.5 && !bar {
        // The block this pixel belongs to: step half a block back from the face, into the block.
        let in_block = in.local - face_normal(in.face) * 0.5;
        let block = in.section_origin + vec3<i32>(floor(in_block));
        // Turn the image around the block's centre in quarter turns.
        uv = turned(uv, block, in.face);
    }

    // A far copy's plain ground (`far_ground`) carries its block's top and side layers instead of
    // one of its own (`far_copies.rs`, `PLAIN`): the top's for its top, the side's for its sides.
    let plain = (in.layer & 0x8000u) != 0u;
    let top_layer = in.layer & 127u;
    let side_layer = (in.layer >> 7u) & 127u;
    let layer = select(in.layer, select(side_layer, top_layer, in.face >= 4u), plain);
    let sampled = textureSampleGrad(block_textures, block_sampler, uv, layer, uv_dx, uv_dy);
    // See-through blocks (like the grid): pixels less than half opaque are cut out entirely.
    // Ordinary block textures are fully opaque, so this never affects them.
    if sampled.a < 0.5 {
        discard;
    }
    var colour = sampled.rgb;
    // Light only (the seam probe): plain grey, the lighting kept.
    if settings.options.y > 0.5 {
        colour = vec3<f32>(0.5);
    }

    let debug_view = u32(settings.fog_and_debug.w);
    if debug_view == 1u {
        colour = debug_colour(in.quad + 1u);
    } else if debug_view == 2u {
        colour = debug_colour(in.face + 100u);
    } else if debug_view == 3u {
        // Who drew this (`docs/caves.md` 13.7): the real chunks blue, the far copies green (darker the
        // coarser), their outer edges magenta (the skirts, and any face there); the texture's light and
        // dark kept faintly and all the lighting after, so shapes still read.
        var who = vec3<f32>(0.25, 0.45, 1.0);
        if settings.options.z > 0.5 {
            who = mix(vec3<f32>(0.45, 1.0, 0.35), vec3<f32>(0.05, 0.45, 0.1), clamp(log2(in.scale) / 3.0, 0.0, 1.0));
            let across = select(in.local.y, in.local.x, in.face <= 1u);
            let outward = select(0.0, 32.0, in.face == 0u || in.face == 2u);
            // Plain ground (drawn as the far terrain draws the land) paler.
            if plain {
                who = mix(who, vec3<f32>(1.0), 0.45);
            }
            if in.face <= 3u && abs(across - outward) < 0.01 {
                who = vec3<f32>(1.0, 0.2, 0.9);
            }
        }
        colour = who * (0.6 + 0.4 * dot(colour, vec3<f32>(0.3, 0.59, 0.11)));
    }

    // Classic block shading: brightness by which way the face points.
    let shade = face_shade(in.face, settings.face_shade);
    if plain && debug_view == 0u {
        // The textures' smallest versions: one colour each, as the far terrain's.
        let smallest = f32(textureNumLevels(block_textures) - 1u);
        var top_colour = textureSampleLevel(block_textures, block_sampler, vec2<f32>(0.5), top_layer, smallest).rgb;
        var side_colour = textureSampleLevel(block_textures, block_sampler, vec2<f32>(0.5), side_layer, smallest).rgb;
        if settings.options.y > 0.5 {
            top_colour = vec3<f32>(0.5);
            side_colour = vec3<f32>(0.5);
        }
        colour = far_ground(top_colour, side_colour, in.world_position) * underground_light(in.daylight, in.placed, in.world_position);
    } else {
        colour = colour * shade * ground_light(in.world_position, in.face) * underground_light(in.daylight, in.placed, in.world_position);
    }

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

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
import super::lighting::{eased, height_tint, hill_light, light_from_map, side_lie, unroot};

struct TerrainShaderSettings {
    // x top, y bottom, z north/south, w east/west
    face_shade: vec4<f32>,
    fog_color: vec4<f32>,
    // x fog start, y fog end, z fog on, w debug view
    fog_and_debug: vec4<f32>,
    // x random texture rotation on, y light only (plain grey)
    options: vec4<f32>,
    // Survey view: x on, y every how many levels, z darkness, w sea level
    survey: vec4<f32>,
    // Underground (src/render/sky_light.rs): x daylight on, y the test lantern's reach (blocks; 0 off),
    // z its brightness
    underground: vec4<f32>,
    // Placed light (src/render/placed_light.rs, in the vertex): x on, y how much it flickers, z how fast
    placed: vec4<f32>,
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
    // step shadows (strengths), w the height the height tint counts up to (as the distant tiles)
    shade: vec4<f32>,
    // x how much of the lie of the land's sun and shade the steps' sides take (0 to 1)
    sides: vec4<f32>,
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
    out.section_origin = vec3<i32>(round(vec3<f32>(-origin.z, origin.x, origin.y)));
    out.daylight = f32((a >> 28u) & 15u) / 15.0;
    let c = in.packed.z;
    out.placed = vec3<f32>(f32(c & 15u), f32((c >> 4u) & 15u), f32((c >> 8u) & 15u)) / 15.0;
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
// (lighting.wesl), so the two match:
//   - the world light map (shadows, valley darkening, relief), on every face;
//   - the lie of the land's sun and shade, from the slope of the ground under them (a block world's
//     tops are all flat, so which way a face points says nothing about it), and the height tint: on
//     tops, and as much as set on the steps' sides;
//   - the feet of the steps, from the cone map, on tops.
// Faces well below the ground (caves, overhangs) keep their plain light, for now.
fn ground_light(world_position: vec3<f32>, face: u32) -> f32 {
    let north = -world_position.z;
    let east = world_position.x;
    let up = world_position.y;
    var light = 1.0;

    if lighting.light.x > 0.5 {
        let uv = vec2<f32>(east + lighting.light_area.x, lighting.light_area.x - north) / max(lighting.light_area.y, 1.0);
        light = light_from_map(textureSampleLevel(light_map, light_sampler, uv, 0.0), lighting.light);
    }

    if lighting.near_area.w > 0.5 {
        let uv = vec2<f32>(east - lighting.near_area.y, lighting.near_area.x - north) / lighting.near_area.z;
        let near = textureSampleLevel(near_map, near_sampler, uv, 0.0);
        if all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0)) && near.a > 0.5 {
            let sea = lighting.hill.z;
            let surface = sea + near.b * (lighting.hill.w - sea);
            if face != 5u {
                // The rise across one distant cell, as the distant terrain measures it.
                let rise = vec2<f32>(unroot(near.r), unroot(near.g)) * 4.0 * lighting.hill.y;
                let lie = hill_light(rise, lighting.sun.w, lighting.sun.xyz, lighting.shade.x) * height_tint(up, sea, lighting.shade.w, lighting.hill.x);
                var lit = side_lie(lie, lighting.sides.x);
                if face == 4u {
                    // The feet of steps: how far this top is below the cones of the ground round it
                    // (0 on a top, a rim or open ground), eased like the hill shading.
                    let cones = cones_at(uv);
                    let fade = lighting.shade.x;
                    let feet = (1.0 - lighting.shade.y * eased(cones.x - up, fade)) * (1.0 - lighting.shade.z * eased(cones.y - up, fade));
                    lit = lie * feet;
                }
                // The sea's surface stays flat, as on the distant terrain.
                let water = surface <= sea + 0.5 && up <= sea + 0.5;
                light = light * select(lit, 1.0, water);
            }
            // Well below the ground: plain light.
            light = mix(1.0, light, smoothstep(surface - 24.0, surface - 12.0, up));
        }
    }
    return light;
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

// How lit a face is underground (src/render/sky_light.rs; docs/caves.md, stop 3): its daylight, eased
// (squared, so it falls away quickly into the dark), or the test lantern round the camera, whichever
// is brighter; and the placed light on top, in its colour (src/render/placed_light.rs; docs/objects.md,
// stop 3), eased the same way and wavering a little with time, as a fire's does. Pitch black where
// none reaches [David: "pitch black unless something is making light down there"].
fn underground_light(daylight: f32, placed: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {
    var lit = 1.0;
    if settings.underground.x > 0.5 {
        lit = daylight * daylight;
    }
    let reach = settings.underground.y;
    if reach > 0.0 {
        let near = 1.0 - clamp(length(world_position - view.world_position) / reach, 0.0, 1.0);
        lit = max(lit, settings.underground.z * near * near);
    }
    var light = vec3<f32>(lit);
    if settings.placed.x > 0.5 {
        // The flicker: two slow waves and a quicker one, so it never quite repeats.
        let t = globals.time * settings.placed.z;
        let waver = 0.5 + 0.25 * sin(t * 6.1) + 0.15 * sin(t * 13.7 + 1.3) + 0.1 * sin(t * 29.3 + 2.9);
        let flicker = 1.0 - settings.placed.y * waver;
        light = light + placed * placed * flicker;
    }
    return min(light, vec3<f32>(1.0));
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
    // Light only (the seam probe): plain grey, the lighting kept.
    if settings.options.y > 0.5 {
        colour = vec3<f32>(0.5);
    }

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
    colour = colour * shade * ground_light(in.world_position, in.face) * underground_light(in.daylight, in.placed, in.world_position);

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

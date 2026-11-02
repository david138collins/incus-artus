// Distant terrain shader: draws the far-away landscape (see src/render/distant.rs).
//
// Each corner carries its position, the surface block's colours (its top's, and its side's for the
// steps' sides), the slope there and its easing into the next ring out (the vertex part). The
// fragment part:
//   - draws nothing over a real chunk that's on screen (the `shown` texture marks them, one pixel
//     per chunk), so the two never overlap, and where real chunks haven't loaded yet the distant
//     terrain still shows: no holes when you fly fast;
//   - lights the ground with the near ground's own sums (lighting.wesl), from the same settings
//     [2026-10-29, Counsel's seam plan]: the lie of the land, the height tint, the steps' feet and
//     sides as the near ground's blocks show them on average;
//   - is lit by the light map (shadows, valleys: see src/render/light.rs);
//   - fades into the sky colour with distance, exactly like the near terrain's fog.

import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::view;
import super::lighting::{as_seen, height_tint, hill_light, light_from_map, side_lie, to_game};

struct DistantShaderSettings {
    fog_color: vec4<f32>,
    // x fog start, y fog end, z fog on, w the seam probe (drawn over the near chunks too)
    fog: vec4<f32>,
    // xy = the chunk (game axes) at the middle of the `shown` texture
    shown_centre: vec4<f32>,
    // Survey view: x on, y every how many levels, z darkness, w sea level
    survey: vec4<f32>,
    // The light map (src/render/light.rs): x on, y shadows, z sky, w relief (strengths)
    light: vec4<f32>,
    // x the map's half width, y the light map's width (blocks)
    light_area: vec4<f32>,
    // xyz toward the sun (game axes), w hill shading strength
    sun: vec4<f32>,
    // Face brightness: x tops, y north/south sides, z east/west sides, w how much of the slope's
    // light the sides take
    faces: vec4<f32>,
    // x height tint, y the finest cell (the slope's measure), z sea level, w the tint's top
    hill: vec4<f32>,
    // x shading fade, y far slopes match the near, z light only, w the steepest slope the tables reach
    shade: vec4<f32>,
    // The steps' feet's shade by steepness (blocks up per block across, from flat to `shade.w` in 32
    // steps, four to a vec4): all round, and from the sun's side
    foot: array<vec4<f32>, 9>,
    step: array<vec4<f32>, 9>,
    // x the "who drew this" view on (`docs/caves.md` 13.7), y where a ring's easing starts, in its
    // own tiles (so a corner's ring can be told from its easing), z how bright the sky is (0 to 1)
    view: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: DistantShaderSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var shown: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var light_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    // rgb the surface block's top colour, a 1 on land, 0 on the sea
    @location(1) colour: vec4<f32>,
    // Easing into the next ring out (src/render/distant.rs, `TileMesh::morph`): x the coarser ring's
    // height here (game Z = Bevy y), y where easing starts and z where it's complete (blocks, the
    // larger of north-south and east-west from the camera).
    @location(2) ease: vec3<f32>,
    // rgb the surface block's side colour
    @location(3) side: vec4<f32>,
    // xy the slope: blocks up per block across, north and east
    @location(4) slope: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) side: vec3<f32>,
    @location(3) slope: vec2<f32>,
    // Which ring (0 the finest), for the "who drew this" view.
    @location(4) @interpolate(flat) ring: f32,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var world = get_world_from_local(in.instance_index) * vec4<f32>(in.position, 1.0);
    // Toward the ring's outer edge each corner slides to the coarser ring's surface, so the two meet
    // without a crack and nothing pops as rings change round you.
    let away = max(abs(world.x - view.world_position.x), abs(world.z - view.world_position.z));
    let eased = clamp((away - in.ease.y) / max(in.ease.z - in.ease.y, 1.0), 0.0, 1.0);
    world.y = mix(world.y, in.ease.x, eased);
    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world.xyz);
    out.world_position = world.xyz;
    out.colour = in.colour;
    out.side = in.side.rgb;
    out.slope = in.slope.xy;
    // A ring's easing starts `view.y` of its tiles out, and its tiles are 16 cells of the finest
    // cell doubled once a ring.
    out.ring = clamp(round(log2(max(in.ease.y / max(settings.view.y * settings.hill.y * 16.0, 1.0), 1.0))), 0.0, 15.0);
    return out;
}

// One of the steps' shade tables (`step` if `steps`, else `foot`) at a slope `rise` blocks up per block across,
// blended between its two nearest entries (as `Shade::steps` read them on the CPU before).
fn table_at(steps: bool, rise: f32) -> f32 {
    let k = min(rise / max(settings.shade.w, 1e-3) * 32.0, 32.0);
    let low = u32(floor(k));
    let high = min(low + 1u, 32u);
    let a = select(settings.foot[low / 4u][low % 4u], settings.step[low / 4u][low % 4u], steps);
    let b = select(settings.foot[high / 4u][high % 4u], settings.step[high / 4u][high % 4u], steps);
    return mix(a, b, fract(k));
}

// The land's colour as the near ground's blocks show it on average, from here: the tops (the lie of
// the land, the height tint, the shade at the feet of the steps, more on the side away from the sun)
// and the steps' sides, each its own colour and brightness, mixed by how much the camera sees of each.
fn land(top_colour: vec3<f32>, side_colour: vec3<f32>, slope: vec2<f32>, world_position: vec3<f32>) -> vec3<f32> {
    let up = world_position.y;
    let lie = hill_light(slope * settings.hill.y, settings.sun.w, settings.sun.xyz, settings.shade.x) * height_tint(up, settings.hill.z, settings.hill.w, settings.hill.x);
    let top = top_colour * settings.faces.x * lie;
    let rise = length(slope);
    let matched = settings.shade.y;
    if matched <= 0.0 || rise <= 1e-4 {
        return top;
    }
    // The step shadows fall where the slope rises toward the sun (the steps' walls face away).
    let sun = settings.sun.xy;
    let toward_sun = select(0.0, clamp(dot(slope / rise, normalize(sun)), 0.0, 1.0), length(sun) > 1e-5);
    let feet = table_at(false, rise) * mix(1.0, table_at(true, rise), toward_sun);
    let sides = side_lie(lie, settings.faces.w);
    return as_seen(
        top * mix(1.0, feet, min(matched, 1.0)),
        side_colour * settings.faces.y * sides,
        side_colour * settings.faces.z * sides,
        slope * matched,
        to_game(normalize(view.world_position - world_position)),
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // The light map here: columns run east, rows south, from the map's north-west corner. Read
    // first, before anything is cut out, as the graphics card needs for smooth sampling.
    let north = -in.world_position.z;
    let east = in.world_position.x;
    let light_uv = vec2<f32>(east + settings.light_area.x, settings.light_area.x - north) / max(settings.light_area.y, 1.0);
    let light_pixel = textureSample(light_map, light_sampler, light_uv);

    // Which chunk this is in (game x = -Bevy z, game y = Bevy x), and so which pixel of `shown`.
    let block = vec2<f32>(-in.world_position.z, in.world_position.x);
    let size = vec2<i32>(textureDimensions(shown));
    let pixel = vec2<i32>(floor(block / 32.0)) - vec2<i32>(settings.shown_centre.xy) + size / 2;
    if settings.fog.w < 0.5 && all(pixel >= vec2<i32>(0)) && all(pixel < size) && textureLoad(shown, pixel, 0).r > 0.5 {
        discard;
    }
    // Light only (the seam probe): every colour plain grey, the lighting kept.
    let grey = settings.shade.z > 0.5;
    var top_colour = select(in.colour.rgb, vec3<f32>(0.5), grey);
    var side_colour = select(in.side, vec3<f32>(0.5), grey);
    // Who drew this (`docs/caves.md` 13.7): the sheet yellow in its finest ring, toward red further
    // out, its lighting kept.
    if settings.view.x > 0.5 {
        top_colour = mix(vec3<f32>(1.0, 0.85, 0.2), vec3<f32>(0.75, 0.15, 0.1), min(in.ring / 6.0, 1.0));
        side_colour = top_colour;
    }
    // The sea stays flat; between a sea corner and a land one, a blend of the two.
    let sea = top_colour * settings.faces.x;
    var colour = mix(sea, land(top_colour, side_colour, in.slope, in.world_position), clamp(in.colour.a, 0.0, 1.0));
    if settings.light.x > 0.5 {
        colour = colour * light_from_map(light_pixel, settings.light);
    }
    // Survey view, the same bands as the near terrain. Which level of blocks this pixel is on: a
    // top sits on the block below it, so step a hair down (Bevy y = game Z).
    if settings.survey.x > 0.5 {
        let z = i32(floor(in.world_position.y - 0.01));
        let every = max(i32(settings.survey.y), 1);
        let level = z - i32(settings.survey.w);
        if ((level % every) + every) % every == 0 {
            colour = colour * (1.0 - settings.survey.z);
        }
    }
    // How bright the sky is: the far land dimmed with the near (a stand-in for night).
    colour = colour * settings.view.z;
    if settings.fog.z > 0.5 {
        let distance = length(in.world_position - view.world_position);
        colour = mix(colour, settings.fog_color.rgb, smoothstep(settings.fog.x, settings.fog.y, distance));
    }
    return vec4<f32>(colour, 1.0);
}

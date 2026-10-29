// Distant terrain shader: draws the far-away landscape (see src/render/distant.rs).
//
// Each corner carries its position, a colour (the surface block's average colour, already lit by the
// slope there) and its easing into the next ring out (the vertex part). The fragment part:
//   - draws nothing over a real chunk that's on screen (the `shown` texture marks them, one pixel
//     per chunk), so the two never overlap, and where real chunks haven't loaded yet the distant
//     terrain still shows: no holes when you fly fast;
//   - is lit by the light map (shadows, valleys: see src/render/light.rs);
//   - fades into the sky colour with distance, exactly like the near terrain's fog.

import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::view;

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
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: DistantShaderSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var shown: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var light_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_sampler: sampler;

// How lit the ground is (1 = as painted), from a light map pixel: red sun, green sky hidden
// (square-rooted), blue relief (0.5 = level with its surroundings, square-rooted each way), alpha
// land. The sea only takes the shadows.
fn light_from(map: vec4<f32>) -> f32 {
    let sun = 1.0 - settings.light.y * (1.0 - map.r);
    let sky = 1.0 - min(settings.light.z * map.g * map.g, 0.9);
    let b = map.b * 2.0 - 1.0;
    let relief = max(1.0 + settings.light.w * sign(b) * b * b, 0.2);
    return sun * mix(1.0, sky * relief, map.a);
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) colour: vec4<f32>,
    // Easing into the next ring out (src/render/distant.rs, `TileMesh::morph`): x the coarser ring's
    // height here (game Z = Bevy y), y where easing starts and z where it's complete (blocks, the
    // larger of north-south and east-west from the camera).
    @location(2) ease: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) colour: vec4<f32>,
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
    return out;
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
    var colour = in.colour.rgb;
    if settings.light.x > 0.5 {
        colour = colour * light_from(light_pixel);
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
    if settings.fog.z > 0.5 {
        let distance = length(in.world_position - view.world_position);
        colour = mix(colour, settings.fog_color.rgb, smoothstep(settings.fog.x, settings.fog.y, distance));
    }
    return vec4<f32>(colour, 1.0);
}

// The far view by ray marching (src/render/march.rs, docs/far-view.md): the far land the sheet
// draws (distant.wesl), found by a ray from every pixel instead of drawn as triangles.
//
// One triangle covers the screen. For each pixel:
//   - the ray starts where it leaves the square of real chunks round you (nothing far shows inside);
//   - each step goes past a tile the ray is wholly above (by the tile's highest land), past a block
//     of 4 x 4 cells likewise, or across one cell, where it meets the cell's two triangles exactly;
//   - the first triangle it comes down onto, outside what the real chunks and the far copies draw
//     (the `shown` texture, as the sheet uses it), is where it meets the land;
//   - a ray that goes deep under the land has gone into the ground the near world draws, and stops;
//   - the land there is lit with the sheet's own sums, from the sheet's settings, and the pixel's
//     depth is written, so the near world, water and the far copies draw in front where they're nearer.
// The land is read from the sheet's own tiles' corners: two triangles a cell, eased into the next
// ring out toward a ring's edge, so the march and the sheet are the same shape. Each step follows
// `MarchState::march_ray` (src/render/march.rs), which the tests check against small steps.

import bevy_pbr::render::mesh_view_bindings::view;
import super::lighting::{land, light_from_map, to_game};

struct DistantShaderSettings {
    fog_color: vec4<f32>,
    fog: vec4<f32>,
    shown_centre: vec4<f32>,
    survey: vec4<f32>,
    light: vec4<f32>,
    light_area: vec4<f32>,
    sun: vec4<f32>,
    faces: vec4<f32>,
    hill: vec4<f32>,
    shade: vec4<f32>,
    foot: array<vec4<f32>, 9>,
    step: array<vec4<f32>, 9>,
    view: vec4<f32>,
}

struct MarchSettings {
    // x rings, y the finest cell (blocks), z the highest land, w the most steps a pixel takes
    grid: vec4<f32>,
    // The square of real chunks round you (game axes, blocks): least x, y, greatest x, y; all 0: none
    inner: vec4<f32>,
}

// One corner of a tile: x its height, y the coarser ring's height there, zw the slope north and
// east; the top's colour (a: 1 on land, 0 on the sea); the side's colour.
struct Corner {
    ground: vec4<f32>,
    top: vec4<f32>,
    side: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: DistantShaderSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var shown: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var light_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var<uniform> march: MarchSettings;
// Every ring's 16 x 16 slots, 17 x 17 corners each
@group(constants::MATERIAL_BIND_GROUP) @binding(5) var<storage, read> corners: array<Corner>;
// Each slot's tile: x, y, then 0 nothing yet, 1 land, 2 wholly past the map
@group(constants::MATERIAL_BIND_GROUP) @binding(6) var<storage, read> slots: array<vec4<i32>>;
// Each slot's highest land: its 16 blocks of 4 x 4 cells (row by row east, north first), then the
// tile's; 5 x 4 numbers a slot
@group(constants::MATERIAL_BIND_GROUP) @binding(7) var<storage, read> tops: array<vec4<f32>>;

const CELLS: f32 = 16.0;
const TILES: i32 = 16;
const SIDE: u32 = 17u;
const BLOCK: f32 = 4.0;
const MISSING: f32 = -1.0e29;

struct Vertex {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position.xy, 0.0, 1.0);
    return out;
}

fn to_bevy(game: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(game.y, game.z, -game.x);
}

// Where a spot's land is kept: its tile's slot, the tile's corner and size.
struct Place {
    found: bool,
    // Past every ring (or wholly past the map): no land here or further on
    beyond: bool,
    ring: u32,
    slot: u32,
    origin: vec2<f32>,
    size: f32,
}

// The finest ring whose tile there is in (as the sheet shows it: each ring's hole is where the ring
// inside it lies). A ring whose tile isn't in yet leaves it to the ring outside.
fn locate(at: vec2<f32>) -> Place {
    var place: Place;
    place.found = false;
    place.beyond = true;
    let camera = to_game(view.world_position).xy;
    let rings = u32(march.grid.x);
    for (var r = 0u; r < rings; r++) {
        let size = march.grid.y * CELLS * exp2(f32(r));
        let tile = vec2<i32>(floor(at / size));
        let start = 2 * vec2<i32>(floor(camera / (2.0 * size))) - vec2<i32>(TILES / 2);
        if any(tile < start) || any(tile >= start + vec2<i32>(TILES)) {
            continue;
        }
        place.beyond = false;
        let wrapped = ((tile % TILES) + TILES) % TILES;
        let slot = r * u32(TILES * TILES) + u32(wrapped.y * TILES + wrapped.x);
        let info = slots[slot];
        if info.z == 0 || info.x != tile.x || info.y != tile.y {
            continue;
        }
        if info.z == 2 {
            place.beyond = true;
            return place;
        }
        place.found = true;
        place.ring = r;
        place.slot = slot;
        place.origin = vec2<f32>(tile) * size;
        place.size = size;
        return place;
    }
    return place;
}

fn corner_at(place: Place, i: u32, j: u32) -> Corner {
    return corners[place.slot * SIDE * SIDE + j * SIDE + i];
}

// Corner (i, j)'s height, eased toward the coarser ring's toward its ring's outer edge (as the
// sheet's vertex part does), or below MISSING past the map.
fn eased_height(place: Place, i: u32, j: u32) -> f32 {
    let c = corner_at(place, i, j);
    if c.ground.x < MISSING {
        return MISSING * 2.0;
    }
    let spot = place.origin + vec2<f32>(f32(i), f32(j)) * (place.size / CELLS);
    let camera = to_game(view.world_position).xy;
    let away = max(abs(spot.x - camera.x), abs(spot.y - camera.y));
    let eased = clamp((away - settings.view.y * place.size) / max(place.size, 1.0), 0.0, 1.0);
    return mix(c.ground.x, c.ground.y, eased);
}

// One of a slot's highest-land numbers (0 to 15 its blocks, 16 the tile).
fn top_of(slot: u32, k: u32) -> f32 {
    return tops[slot * 5u + k / 4u][k % 4u];
}

// Where a ray leaves the square from `low` to `high` (cells across the tile), as a distance along
// it, from `t` where it's at `local` (cells), changing by `rate` cells a block.
fn leave(t: f32, local: vec2<f32>, rate: vec2<f32>, low: vec2<f32>, high: vec2<f32>) -> f32 {
    var out = vec2<f32>(1.0e9);
    if rate.x > 1e-9 {
        out.x = (high.x - local.x) / rate.x;
    } else if rate.x < -1e-9 {
        out.x = (low.x - local.x) / rate.x;
    }
    if rate.y > 1e-9 {
        out.y = (high.y - local.y) / rate.y;
    } else if rate.y < -1e-9 {
        out.y = (low.y - local.y) / rate.y;
    }
    return t + max(min(out.x, out.y), 0.0);
}

// The cell's triangle a spot is in: its three corners (as (di, dj) pairs) and their weights. The
// sheet splits each cell from its first corner to the opposite one.
struct Triangle {
    a: vec2<u32>,
    b: vec2<u32>,
    c: vec2<u32>,
    w: vec3<f32>,
}

fn triangle(uv: vec2<f32>) -> Triangle {
    var t: Triangle;
    t.a = vec2<u32>(0u, 0u);
    t.c = vec2<u32>(1u, 1u);
    if uv.x >= uv.y {
        t.b = vec2<u32>(1u, 0u);
        t.w = vec3<f32>(1.0 - uv.x, uv.x - uv.y, uv.y);
    } else {
        t.b = vec2<u32>(0u, 1u);
        t.w = vec3<f32>(1.0 - uv.y, uv.y - uv.x, uv.x);
    }
    return t;
}

// Whether the real chunks or the far copies draw this spot (the sheet's `shown` texture).
fn drawn_near(at: vec2<f32>) -> bool {
    if settings.fog.w > 0.5 {
        return false;
    }
    let size = vec2<i32>(textureDimensions(shown));
    let pixel = vec2<i32>(floor(at / 32.0)) - vec2<i32>(settings.shown_centre.xy) + size / 2;
    return all(pixel >= vec2<i32>(0)) && all(pixel < size) && textureLoad(shown, pixel, 0).r > 0.5;
}

// One of the steps' shade tables, as distant.wesl reads it.
fn table_at(steps: bool, rise: f32) -> f32 {
    let k = min(rise / max(settings.shade.w, 1e-3) * 32.0, 32.0);
    let low = u32(floor(k));
    let high = min(low + 1u, 32u);
    let a = select(settings.foot[low / 4u][low % 4u], settings.step[low / 4u][low % 4u], steps);
    let b = select(settings.foot[high / 4u][high % 4u], settings.step[high / 4u][high % 4u], steps);
    return mix(a, b, fract(k));
}

struct Output {
    @location(0) colour: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput) -> Output {
    // The pixel's ray, in game axes: worked out from the camera, then turned to the world; never
    // from two points in the world. A point on the near plane is a tenth of a block from the camera,
    // and thousands of blocks out the world's positions are kept only to a few thousandths of a
    // block, so the difference pointed every ray a little wrong, differently each frame [David,
    // build 13: "its jiggly"].
    let ndc = vec2<f32>(
        (in.clip_position.x - view.viewport.x) / view.viewport.z * 2.0 - 1.0,
        1.0 - (in.clip_position.y - view.viewport.y) / view.viewport.w * 2.0,
    );
    let on_near = view.view_from_clip * vec4<f32>(ndc, 1.0, 1.0);
    let toward = normalize(on_near.xyz / on_near.w);
    let origin = to_game(view.world_position);
    let ray = normalize(to_game((view.world_from_view * vec4<f32>(toward, 0.0)).xyz));

    // Start where the ray leaves the square of real chunks (if you're in it).
    var t = 0.5;
    let inner = march.inner;
    if inner.z > inner.x && all(origin.xy >= inner.xy) && all(origin.xy <= inner.zw) {
        let bound = select(inner.xy, inner.zw, ray.xy > vec2<f32>(0.0));
        var out = vec2<f32>(1.0e9);
        if abs(ray.x) > 1e-9 {
            out.x = (bound.x - origin.x) / ray.x;
        }
        if abs(ray.y) > 1e-9 {
            out.y = (bound.y - origin.y) / ray.y;
        }
        t = max(t, min(out.x, out.y));
    }
    var furthest = 1.0e7;
    if settings.fog.z > 0.5 {
        furthest = settings.fog.y;
    }
    let highest = march.grid.z;
    let most = u32(march.grid.w);
    var hit = false;
    var hit_place: Place;
    var hit_cell = vec2<u32>(0u);
    var hit_uv = vec2<f32>(0.0);
    for (var k = 0u; k < most; k++) {
        if t > furthest {
            break;
        }
        let p = origin + ray * t;
        if ray.z >= 0.0 && p.z > highest {
            break;
        }
        let place = locate(p.xy);
        if !place.found {
            if place.beyond {
                break;
            }
            // No tile in yet here: on, at a twentieth of the distance.
            t += max(t * 0.05, 16.0);
            continue;
        }
        let cell = place.size / CELLS;
        let local = (p.xy - place.origin) / cell;
        let rate = ray.xy / cell;
        let nudge = cell * 1e-3;
        // Past the tile, if the ray is above all of it until it leaves.
        let out_of_tile = leave(t, local, rate, vec2<f32>(0.0), vec2<f32>(CELLS));
        if min(p.z, origin.z + ray.z * out_of_tile) > top_of(place.slot, 16u) {
            t = out_of_tile + nudge;
            continue;
        }
        // Past the block of 4 x 4 cells, likewise.
        let block = clamp(floor(local / BLOCK), vec2<f32>(0.0), vec2<f32>(CELLS / BLOCK - 1.0));
        let out_of_block = leave(t, local, rate, block * BLOCK, (block + 1.0) * BLOCK);
        if min(p.z, origin.z + ray.z * out_of_block) > top_of(place.slot, u32(block.y * (CELLS / BLOCK) + block.x)) {
            t = out_of_block + nudge;
            continue;
        }
        // Across one cell: where the ray comes down onto either of its triangles.
        let ij = clamp(floor(local), vec2<f32>(0.0), vec2<f32>(CELLS - 1.0));
        let out_of_cell = leave(t, local, rate, ij, ij + 1.0);
        let i = u32(ij.x);
        let j = u32(ij.y);
        let a = eased_height(place, i, j);
        let b = eased_height(place, i + 1u, j);
        let c = eased_height(place, i + 1u, j + 1u);
        let d = eased_height(place, i, j + 1u);
        if min(min(a, b), min(c, d)) > MISSING {
            let uv = local - ij;
            // Each triangle's plane: height = h0 + gu u + gv v, and which half of the cell it covers.
            var nearest = 1.0e9;
            for (var half = 0; half < 2; half++) {
                let first_half = half == 0;
                let h0 = a;
                let gu = select(c - d, b - a, first_half);
                let gv = select(d - a, c - b, first_half);
                let gap = p.z - (h0 + gu * uv.x + gv * uv.y);
                let closing = ray.z - gu * rate.x - gv * rate.y;
                if gap >= 0.0 && closing < 0.0 {
                    let at = t + gap / -closing;
                    let there = uv + rate * (at - t);
                    if at <= out_of_cell && (there.x >= there.y) == first_half {
                        nearest = min(nearest, at);
                    }
                }
            }
            if nearest < 1.0e9 && !drawn_near((origin + ray * nearest).xy) {
                hit = true;
                hit_place = place;
                hit_cell = vec2<u32>(i, j);
                hit_uv = clamp(uv + rate * (nearest - t), vec2<f32>(0.0), vec2<f32>(1.0));
                t = nearest;
                break;
            }
            // Deep under the land: gone in under the near world, whose ground draws in front of
            // anything further along.
            let w = triangle(uv).w;
            let under = select(a * w.x + d * w.y + c * w.z, a * w.x + b * w.y + c * w.z, uv.x >= uv.y);
            if p.z < under - max(16.0, 2.0 * cell) {
                break;
            }
        }
        t = out_of_cell + nudge;
    }
    if !hit {
        discard;
    }
    let spot = origin + ray * t;
    let tri = triangle(hit_uv);
    let ca = corner_at(hit_place, hit_cell.x + tri.a.x, hit_cell.y + tri.a.y);
    let cb = corner_at(hit_place, hit_cell.x + tri.b.x, hit_cell.y + tri.b.y);
    let cc = corner_at(hit_place, hit_cell.x + tri.c.x, hit_cell.y + tri.c.y);
    let w = tri.w;
    var top_colour = ca.top.rgb * w.x + cb.top.rgb * w.y + cc.top.rgb * w.z;
    var side_colour = ca.side.rgb * w.x + cb.side.rgb * w.y + cc.side.rgb * w.z;
    let land_share = ca.top.a * w.x + cb.top.a * w.y + cc.top.a * w.z;
    let slope = ca.ground.zw * w.x + cb.ground.zw * w.y + cc.ground.zw * w.z;
    let world_position = to_bevy(spot);

    // From here on, as the sheet's fragment part.
    let light_uv = vec2<f32>(spot.y + settings.light_area.x, settings.light_area.x - spot.x) / max(settings.light_area.y, 1.0);
    let light_pixel = textureSampleLevel(light_map, light_sampler, light_uv, 0.0);
    let grey = settings.shade.z > 0.5;
    top_colour = select(top_colour, vec3<f32>(0.5), grey);
    side_colour = select(side_colour, vec3<f32>(0.5), grey);
    // Who drew this: the march blue, toward purple further out, its lighting kept.
    if settings.view.x > 0.5 {
        top_colour = mix(vec3<f32>(0.3, 0.65, 1.0), vec3<f32>(0.55, 0.2, 0.75), min(f32(hit_place.ring) / 6.0, 1.0));
        side_colour = top_colour;
    }
    let sea = top_colour * settings.faces.x;
    let rise = length(slope);
    let lit = land(
        top_colour,
        side_colour,
        slope,
        world_position.y,
        to_game(normalize(view.world_position - world_position)),
        settings.sun,
        vec3<f32>(settings.hill.x, settings.hill.z, settings.hill.w),
        settings.hill.y,
        settings.shade.x,
        settings.faces,
        settings.shade.y,
        table_at(false, rise),
        table_at(true, rise),
    );
    var colour = mix(sea, lit, clamp(land_share, 0.0, 1.0));
    if settings.light.x > 0.5 {
        colour = colour * light_from_map(light_pixel, settings.light);
    }
    if settings.survey.x > 0.5 {
        let z = i32(floor(world_position.y - 0.01));
        let every = max(i32(settings.survey.y), 1);
        let level = z - i32(settings.survey.w);
        if ((level % every) + every) % every == 0 {
            colour = colour * (1.0 - settings.survey.z);
        }
    }
    colour = colour * settings.view.z;
    if settings.fog.z > 0.5 {
        let distance = length(world_position - view.world_position);
        colour = mix(colour, settings.fog_color.rgb, smoothstep(settings.fog.x, settings.fog.y, distance));
    }
    var out: Output;
    out.colour = vec4<f32>(colour, 1.0);
    let clip = view.clip_from_world * vec4<f32>(world_position, 1.0);
    out.depth = clip.z / clip.w;
    return out;
}

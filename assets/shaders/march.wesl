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
    // x rings, y the finest cell (blocks), z the highest land (or far copy), w the most steps a pixel takes
    grid: vec4<f32>,
    // The far copies: x 1 if the march draws them
    copies: vec4<f32>,
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
// The far copies (src/render/far_copies.rs, `CopyCells`; src/render/march.rs, `Copies`): their
// cells in pages of 32 levels of 32 x 32, each packed (look's block in the low 12 bits; 1 solid or
// 2 water above that; 1 << 14 plain ground; an air cell's daylight, 0 to 15, in bits 16 to 19); a
// record of each copy (its first cell's corner x, y, z in blocks and its cell size; how many cells
// tall and how many pages; 2 spare; its pages); a map of which copy draws each chunk, each at its
// place counted round the map's width (0 none, else the record's number + 1); and each block's colours, top then side.
@group(constants::MATERIAL_BIND_GROUP) @binding(8) var<storage, read> copy_cells: array<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(9) var<storage, read> copy_records: array<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(10) var<storage, read> copy_map: array<u32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(11) var<storage, read> block_colours: array<vec4<f32>>;

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

// The land's height across a cell from its corners' heights (a at its first corner, b one cell
// north, c north-east, d east), at `uv` from 0 to 1: the sheet's two triangles.
fn surface(a: f32, b: f32, c: f32, d: f32, uv: vec2<f32>) -> f32 {
    if uv.x >= uv.y {
        return a * (1.0 - uv.x) + b * (uv.x - uv.y) + c * uv.y;
    }
    return a * (1.0 - uv.y) + c * uv.x + d * (uv.y - uv.x);
}

// Whether the real chunks or the far copies draw this spot (the sheet's `shown` texture).
// The real chunks are marked 255 and the far copies' 160 (distant.rs, `COPY_MARK`): while the march
// draws the copies, only the real chunks are someone else's.
fn drawn_near(at: vec2<f32>) -> bool {
    if settings.fog.w > 0.5 {
        return false;
    }
    let size = vec2<i32>(textureDimensions(shown));
    let pixel = vec2<i32>(floor(at / 32.0)) - vec2<i32>(settings.shown_centre.xy) + size / 2;
    let least = select(0.5, 0.9, march.copies.x > 0.5);
    return all(pixel >= vec2<i32>(0)) && all(pixel < size) && textureLoad(shown, pixel, 0).r > least;
}

const MAP: i32 = 256;
const COPY_RECORD: u32 = 24u;
const PAGE: u32 = 32768u;
const SOLID: u32 = 1u;
const PLAIN: u32 = 16384u;

// Which far copy draws the chunk a spot is in: its record's number, or -1.
fn copy_at(at: vec2<f32>) -> i32 {
    if march.copies.x < 0.5 {
        return -1;
    }
    // Laid out by the world: each chunk at its place counted round the map's width (march.rs, why).
    let chunk = vec2<i32>(floor(at / 32.0));
    let wrapped = ((chunk % MAP) + MAP) % MAP;
    let record = i32(copy_map[u32(wrapped.y * MAP + wrapped.x)]) - 1;
    if record < 0 {
        return -1;
    }
    // And that copy covers the spot (not one a map's width away).
    let base = u32(record) * COPY_RECORD;
    let low = vec2<f32>(f32(bitcast<i32>(copy_records[base])), f32(bitcast<i32>(copy_records[base + 1u])));
    let width = 32.0 * f32(copy_records[base + 3u]);
    if any(at < low) || any(at >= low + vec2<f32>(width)) {
        return -1;
    }
    return record;
}

// A copy's cell (x, y across, z up from its bottom), packed as above: solid below its bottom, open
// air in full daylight above its top.
fn copy_cell(record: u32, c: vec3<i32>) -> u32 {
    let base = record * COPY_RECORD;
    let tall = i32(copy_records[base + 4u]);
    if c.z < 0 {
        return SOLID << 12u;
    }
    if c.z >= tall {
        return 15u << 16u;
    }
    let page = copy_records[base + 8u + u32(c.z) / 32u];
    return copy_cells[page * PAGE + (u32(c.z) % 32u) * 1024u + u32(c.y) * 32u + u32(c.x)];
}

// Where a ray leaves the box from `low` to `high` (cells), from `t` where it's at `local`, changing
// by `rate` cells a block: the distance along it, and which way it leaves (0 north-south, 1
// east-west, 2 up-down).
// (Each way written out: DirectX's shader compiler can't turn a loop over them into straight code.
// Build 18 stopped at the start: "unable to unroll loop".)
fn leave_axis(at: f32, rate: f32, low: f32, high: f32) -> f32 {
    if rate > 1e-9 {
        return (high - at) / rate;
    }
    if rate < -1e-9 {
        return (low - at) / rate;
    }
    return 1.0e9;
}

fn leave_box(t: f32, local: vec3<f32>, rate: vec3<f32>, low: vec3<f32>, high: vec3<f32>) -> vec2<f32> {
    let out = vec3<f32>(leave_axis(local.x, rate.x, low.x, high.x), leave_axis(local.y, rate.y, low.y, high.y), leave_axis(local.z, rate.z, low.z, high.z));
    var axis = 0.0;
    var least = out.x;
    if out.y < least {
        least = out.y;
        axis = 1.0;
    }
    if out.z < least {
        least = out.z;
        axis = 2.0;
    }
    return vec2<f32>(t + max(least, 0.0), axis);
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

// The far land's colour where a ray met it (cell `cell` of the tile at `place`, `uv` across it), lit
// with the sheet's own sums.
fn land_colour(place: Place, cell: vec2<u32>, uv: vec2<f32>, spot: vec3<f32>) -> vec3<f32> {
    let tri = triangle(uv);
    let ca = corner_at(place, cell.x + tri.a.x, cell.y + tri.a.y);
    let cb = corner_at(place, cell.x + tri.b.x, cell.y + tri.b.y);
    let cc = corner_at(place, cell.x + tri.c.x, cell.y + tri.c.y);
    let w = tri.w;
    var top_colour = ca.top.rgb * w.x + cb.top.rgb * w.y + cc.top.rgb * w.z;
    var side_colour = ca.side.rgb * w.x + cb.side.rgb * w.y + cc.side.rgb * w.z;
    let land_share = ca.top.a * w.x + cb.top.a * w.y + cc.top.a * w.z;
    let slope = ca.ground.zw * w.x + cb.ground.zw * w.y + cc.ground.zw * w.z;
    let world_position = to_bevy(spot);
    let grey = settings.shade.z > 0.5;
    top_colour = select(top_colour, vec3<f32>(0.5), grey);
    side_colour = select(side_colour, vec3<f32>(0.5), grey);
    // Who drew this: the march blue, toward purple further out, its lighting kept.
    if settings.view.x > 0.5 {
        top_colour = mix(vec3<f32>(0.3, 0.65, 1.0), vec3<f32>(0.55, 0.2, 0.75), min(f32(place.ring) / 6.0, 1.0));
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
    return mix(sea, lit, clamp(land_share, 0.0, 1.0));
}

// How long ago (along the ray) it came in through a cell's face this way, `into` across the cell.
fn since_entering(into: f32, rate: f32) -> f32 {
    if abs(rate) < 1e-9 {
        return 1.0e9;
    }
    return select(1.0 - into, into, rate > 0.0) / abs(rate);
}

// A far copy's cell where a ray met it (`value` packed as above; `local` where the ray was, in its
// cells, and `rate` how fast that changes), lit by the daylight `light` (0 to 15) of the air cell it
// came through: its block's colour, top or side by the face it came in through, shaded by which way
// that faces, and the daylight eased (squared) as the near ground's is underground, so a cave's
// inside is dark.
fn copy_colour(value: u32, local: vec3<f32>, rate: vec3<f32>, light: u32) -> vec3<f32> {
    // The face it came in through: the one it crossed last.
    let into = local - floor(local);
    let since = vec3<f32>(since_entering(into.x, rate.x), since_entering(into.y, rate.y), since_entering(into.z, rate.z));
    var axis = 2;
    if since.x < since.y && since.x < since.z {
        axis = 0;
    } else if since.y < since.z {
        axis = 1;
    }
    let block = value & 0xFFFu;
    let up = axis == 2 && rate.z < 0.0;
    var colour = select(block_colours[block * 2u + 1u].rgb, block_colours[block * 2u].rgb, up);
    if settings.shade.z > 0.5 {
        colour = vec3<f32>(0.5);
    }
    // Who drew this: the march's far copies green.
    if settings.view.x > 0.5 {
        colour = vec3<f32>(0.3, 0.85, 0.4);
    }
    var shade = settings.faces.x;
    if axis == 0 {
        shade = settings.faces.y;
    } else if axis == 1 {
        shade = settings.faces.z;
    }
    let daylight = f32(light) / 15.0;
    return colour * shade * daylight * daylight;
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
    // Whether the ray was above the land where the last step left it (not known at the start: a ray
    // starting under the land went in under the near world). A ray above the land leaving one step
    // and under it entering the next met it in between [build 15: "a field of sky speckles"].
    var was_above = false;
    // A far copy met: its cell, where the ray was in its cells and how fast that changes, and the
    // daylight of the air cell the ray came through (full in the open).
    var hit_copy = false;
    var hit_value = 0u;
    var hit_local = vec3<f32>(0.0);
    var hit_rate = vec3<f32>(0.0);
    var last_light = 15u;
    for (var k = 0u; k < most; k++) {
        if t > furthest {
            break;
        }
        let p = origin + ray * t;
        if ray.z >= 0.0 && p.z > highest {
            break;
        }
        // In a chunk a far copy draws: through its cells, one at a time (step 1 part 5).
        let record = copy_at(p.xy);
        if record >= 0 {
            let base = u32(record) * COPY_RECORD;
            let corner = vec3<f32>(f32(bitcast<i32>(copy_records[base])), f32(bitcast<i32>(copy_records[base + 1u])), f32(bitcast<i32>(copy_records[base + 2u])));
            let scale = f32(copy_records[base + 3u]);
            let tall = f32(copy_records[base + 4u]);
            let local = (p - corner) / scale;
            let rate = ray / scale;
            let nudge = scale * 1e-3;
            let c = vec3<i32>(clamp(floor(local), vec3<f32>(0.0, 0.0, -1.0), vec3<f32>(31.0, 31.0, tall)));
            if f32(c.z) >= tall {
                // Above it: on to where the ray leaves the air over it, or comes down to its top.
                let out = leave_box(t, local, rate, vec3<f32>(0.0, 0.0, tall), vec3<f32>(32.0, 32.0, 1.0e9));
                was_above = true;
                last_light = 15u;
                t = out.x + nudge;
                continue;
            }
            let value = copy_cell(u32(record), c);
            if ((value >> 12u) & 3u) != 0u {
                // Solid, or water: met, coming from the air.
                if was_above && !drawn_near(p.xy) {
                    hit = true;
                    hit_copy = true;
                    hit_value = value;
                    hit_local = local;
                    hit_rate = rate;
                }
                // Else in the ground: gone in under the near world.
                break;
            }
            was_above = true;
            last_light = (value >> 16u) & 15u;
            let out = leave_box(t, local, rate, vec3<f32>(c), vec3<f32>(c) + 1.0);
            t = out.x + nudge;
            continue;
        }
        let place = locate(p.xy);
        if !place.found {
            if place.beyond {
                break;
            }
            // No tile in yet here: on, at a twentieth of the distance.
            t += max(t * 0.05, 16.0);
            was_above = false;
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
            was_above = true;
            continue;
        }
        // Past the block of 4 x 4 cells, likewise.
        let block = clamp(floor(local / BLOCK), vec2<f32>(0.0), vec2<f32>(CELLS / BLOCK - 1.0));
        let out_of_block = leave(t, local, rate, block * BLOCK, (block + 1.0) * BLOCK);
        if min(p.z, origin.z + ray.z * out_of_block) > top_of(place.slot, u32(block.y * (CELLS / BLOCK) + block.x)) {
            t = out_of_block + nudge;
            was_above = true;
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
        if min(min(a, b), min(c, d)) <= MISSING {
            t = out_of_cell + nudge;
            was_above = false;
            continue;
        }
        let uv = local - ij;
        let under = surface(a, b, c, d, uv);
        // Above the land leaving the last step and under it here: it met the land in between.
        if was_above && p.z < under && !drawn_near(p.xy) {
            hit = true;
            hit_place = place;
            hit_cell = vec2<u32>(i, j);
            hit_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
            break;
        }
        // Each triangle's plane: height = h0 + gu u + gv v, and which half of the cell it covers
        // (with a hair either side, where the two planes are the same height anyway).
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
                let across = there.x - there.y;
                let in_half = select(across <= 1e-3, across >= -1e-3, first_half);
                if at <= out_of_cell + nudge && in_half {
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
        if p.z < under - max(16.0, 2.0 * cell) {
            break;
        }
        let leaving = clamp(local + rate * (out_of_cell - t) - ij, vec2<f32>(0.0), vec2<f32>(1.0));
        was_above = origin.z + ray.z * out_of_cell >= surface(a, b, c, d, leaving);
        t = out_of_cell + nudge;
    }
    if !hit {
        discard;
    }
    let spot = origin + ray * t;
    let world_position = to_bevy(spot);
    var colour = vec3<f32>(0.0);
    if hit_copy {
        // Plain ground in a copy takes the far land's look there, so the copy doesn't show as a patch.
        let place = locate(spot.xy);
        if (hit_value & PLAIN) != 0u && place.found {
            let local = (spot.xy - place.origin) / (place.size / CELLS);
            let ij = clamp(floor(local), vec2<f32>(0.0), vec2<f32>(CELLS - 1.0));
            colour = land_colour(place, vec2<u32>(ij), clamp(local - ij, vec2<f32>(0.0), vec2<f32>(1.0)), spot);
        } else {
            colour = copy_colour(hit_value, hit_local, hit_rate, last_light);
        }
    } else {
        colour = land_colour(hit_place, hit_cell, hit_uv, spot);
    }

    // From here on, as the sheet's fragment part.
    let light_uv = vec2<f32>(spot.y + settings.light_area.x, settings.light_area.x - spot.x) / max(settings.light_area.y, 1.0);
    let light_pixel = textureSampleLevel(light_map, light_sampler, light_uv, 0.0);
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

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

import bevy_pbr::render::mesh_view_bindings::{globals, view};
import super::lighting::{block_hash, face_normal, face_shade, face_uv, ground_lit, land, light_from_map, to_game, turned, underground_lit, TerrainLight, TerrainShaderSettings};

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
    // The floating origin (src/coords.rs): x north, y east (blocks), what drawn positions are measured from
    origin: vec4<f32>,
}

struct MarchSettings {
    // x rings, y the finest cell (blocks), z the highest land (or far copy), w the most steps a pixel takes
    grid: vec4<f32>,
    // The far copies: x 1 if the march draws them, y 1 if shaded as the meshes, z the pixels a far cell
    // (0: sizes per ray off), w 1 for contour bands
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
// Each block: its top's far colour, its side's, then its faces' texture layers (north, south, east,
// west; top, bottom, and 2 spare), four sets of four a block.
@group(constants::MATERIAL_BIND_GROUP) @binding(11) var<storage, read> block_colours: array<vec4<f32>>;
// The near world's block textures (the near-march experiment's whole copies).
@group(constants::MATERIAL_BIND_GROUP) @binding(12) var block_textures: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(13) var block_sampler: sampler;
// The near world's settings and light, its window of the ground round you and that window's cone
// map, as the meshes have them (src/render/march.rs, `MarchMaterial::near`): the whole copies shaded
// as the meshes are (`copy_shaded`).
@group(constants::MATERIAL_BIND_GROUP) @binding(14) var<uniform> near_settings: TerrainShaderSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(15) var<uniform> lighting: TerrainLight;
@group(constants::MATERIAL_BIND_GROUP) @binding(16) var near_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(17) var near_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(18) var near_cones: texture_2d<f32>;

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

// The floating origin (src/coords.rs): what drawn positions are measured from, in the world (game
// axes, blocks; whole numbers, so exact).
fn world_origin() -> vec3<f32> {
    return vec3<f32>(settings.origin.xy, 0.0);
}

// The camera in the world's blocks (to a few thousandths far out: for finding tiles and copies).
fn camera_game() -> vec3<f32> {
    return to_game(view.world_position) + world_origin();
}

// A place in the world, as drawn: measured from the floating origin.
fn drawn_at(game: vec3<f32>) -> vec3<f32> {
    return to_bevy(game - world_origin());
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
    let camera = camera_game().xy;
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
    let camera = camera_game().xy;
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

// Whether the near world draws this spot's water (a real chunk past the line, its see-through blocks
// meshed: marked 200, distant.rs `MESHED_WATER_MARK`): the march goes on through it to the bed.
fn water_drawn_near(at: vec2<f32>) -> bool {
    let size = vec2<i32>(textureDimensions(shown));
    let pixel = vec2<i32>(floor(at / 32.0)) - vec2<i32>(settings.shown_centre.xy) + size / 2;
    if !(all(pixel >= vec2<i32>(0)) && all(pixel < size)) {
        return false;
    }
    let mark = textureLoad(shown, pixel, 0).r;
    return mark > 0.7 && mark < 0.9;
}

const MAP: i32 = 256;
// A copy's record: 8 numbers, its 16 pages, its columns' highest (512, two a number) and its 4 x 4
// groups' (32): march.rs, `COPY_RECORD`.
const COPY_RECORD: u32 = 568u;
const COLUMNS_AT: u32 = 24u;
const GROUPS_AT: u32 = 536u;
const PAGE: u32 = 32768u;
const SOLID: u32 = 1u;
const PLAIN: u32 = 16384u;

// How far ahead a ray looks up the copy it's in (blocks; `march.rs`, `LOOKUP_AHEAD`).
const LOOKUP_AHEAD: f32 = 0.03125;

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

// ── Sizes per ray (src/render/march.rs, `FarCopySettings::sized`; docs/far-view.md) ──
// A whole copy keeps its cells 2, 4 and 8 to a side as one too (its levels 1 to 3: far_copies.rs,
// `copy_levels`), straight after its own in its pages. Each spot in it is drawn at the level whose
// cells are about `march.copies.z` pixels across there: a level's cell is used if the level wanted
// at its middle is past the one below it by more than a number of its own (its "random" number, from
// where it is), so the change from one size to the next goes a cell at a time, the same for every
// ray, and each cell changes once as you come near. Fixed by the world, so a ray only ever comes into
// a cell through one of its faces.

// How wide a pixel is, one block from the camera (blocks).
fn pixel_width() -> f32 {
    return 2.0 / (view.clip_from_view[1][1] * view.viewport.w);
}

// A whole copy's cell at a coarser `level` (1 to 3), `c` counted in that level's cells, packed as
// `copy_cell`'s: solid below its bottom, open air in full daylight above its top.
fn level_cell(record: u32, level: u32, c: vec3<i32>) -> u32 {
    let base = record * COPY_RECORD;
    let tall = i32(copy_records[base + 4u]);
    let tall1 = (tall + 1) / 2;
    let tall2 = (tall1 + 1) / 2;
    let tall3 = (tall2 + 1) / 2;
    var high = tall1;
    var side = 16;
    var start = tall * 1024;
    if level == 2u {
        high = tall2;
        side = 8;
        start = start + 256 * tall1;
    } else if level == 3u {
        high = tall3;
        side = 4;
        start = start + 256 * tall1 + 64 * tall2;
    }
    if c.z < 0 {
        return SOLID << 12u;
    }
    if c.z >= high {
        return 15u << 16u;
    }
    let k = u32(start + (c.z * side + c.y) * side + c.x);
    let page = copy_records[base + 8u + k / PAGE];
    return copy_cells[page * PAGE + k % PAGE];
}

// A copy's corner from the camera, as drawn (small numbers: the corner and the floating origin are
// whole numbers, so exact).
fn corner_from_eye(record: u32) -> vec3<f32> {
    let base = record * COPY_RECORD;
    let corner = vec3<f32>(f32(bitcast<i32>(copy_records[base])), f32(bitcast<i32>(copy_records[base + 1u])), f32(bitcast<i32>(copy_records[base + 2u])));
    return (corner - world_origin()) - to_game(view.world_position);
}

// Whether a copy's cell at `level` wants drawing at that size (`c` the copy's own cell the spot is in).
fn wants_level(record: u32, level: u32, c: vec3<i32>, from_eye: vec3<f32>, scale: f32, pixels: f32, pixel: f32) -> bool {
    let base = record * COPY_RECORD;
    let corner = vec3<i32>(bitcast<i32>(copy_records[base]), bitcast<i32>(copy_records[base + 1u]), bitcast<i32>(copy_records[base + 2u]));
    let at = c >> vec3<u32>(level);
    let size = f32(1u << level) * scale;
    let middle = from_eye + (vec3<f32>(at) + 0.5) * size;
    let wanted = log2(pixels * pixel * length(middle) / scale);
    let here = corner + at * i32(size);
    let chance = f32(block_hash(here, level) >> 8u) / 16777216.0;
    return wanted >= f32(level) - 1.0 + chance;
}

// The level a copy's cell `c` is drawn at, the ray `t` from the camera there: 0 (its own size) unless
// sizes per ray are on and it keeps levels.
fn sized_level(record: u32, c: vec3<i32>, t: f32) -> u32 {
    let base = record * COPY_RECORD;
    let pixels = march.copies.z;
    if pixels <= 0.0 || copy_records[base + 7u] != 1u || c.z < 0 {
        return 0u;
    }
    let scale = f32(copy_records[base + 3u]);
    let pixel = pixel_width();
    // No coarser cell's middle within reach wants it (most of what's near): its own size.
    if log2(pixels * pixel * (t + 7.0 * scale) / scale) < 0.0 {
        return 0u;
    }
    let from_eye = corner_from_eye(record);
    if wants_level(record, 3u, c, from_eye, scale, pixels, pixel) {
        return 3u;
    }
    if wants_level(record, 2u, c, from_eye, scale, pixels, pixel) {
        return 2u;
    }
    if wants_level(record, 1u, c, from_eye, scale, pixels, pixel) {
        return 1u;
    }
    return 0u;
}

// Whether any of a copy might be drawn coarser than its own cells (its far corner wants it), the ray
// `t` along.
fn may_be_coarse(record: u32, t: f32) -> bool {
    let base = record * COPY_RECORD;
    let pixels = march.copies.z;
    if pixels <= 0.0 || copy_records[base + 7u] != 1u {
        return false;
    }
    let scale = f32(copy_records[base + 3u]);
    let low = corner_from_eye(record);
    let high = low + vec3<f32>(32.0, 32.0, f32(copy_records[base + 4u])) * scale;
    let far = length(max(abs(low), abs(high)));
    return log2(pixels * pixel_width() * far / scale) >= 0.0;
}

// The highest of a copy's square of 8 x 8 columns (four of its groups), rounded up to a whole
// coarsest cell: nothing solid at any level is above it there.
fn coarse_top(record: u32, square: vec2<i32>) -> f32 {
    let g = square * 2;
    let a = copy_top(record, GROUPS_AT, u32(g.y * 8 + g.x));
    let b = copy_top(record, GROUPS_AT, u32(g.y * 8 + g.x + 1));
    let d = copy_top(record, GROUPS_AT, u32((g.y + 1) * 8 + g.x));
    let e = copy_top(record, GROUPS_AT, u32((g.y + 1) * 8 + g.x + 1));
    return ceil(max(max(a, b), max(d, e)) / 8.0) * 8.0;
}

// One of a copy's columns' or groups' highest (the cell over its highest solid or water one, 0 if
// none), `at` (`COLUMNS_AT` or `GROUPS_AT`) the start of its list, `k` which: two to a number.
fn copy_top(record: u32, at: u32, k: u32) -> f32 {
    let pair = copy_records[record * COPY_RECORD + at + k / 2u];
    return f32((pair >> ((k % 2u) * 16u)) & 0xFFFFu);
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
    let world_position = drawn_at(spot);
    let grey = settings.shade.z > 0.5;
    top_colour = select(top_colour, vec3<f32>(0.5), grey);
    side_colour = select(side_colour, vec3<f32>(0.5), grey);
    // Who drew this: the march blue, toward purple further out, its lighting kept.
    if settings.view.x > 0.5 {
        top_colour = mix(vec3<f32>(0.3, 0.65, 1.0), vec3<f32>(0.55, 0.2, 0.75), min(f32(place.ring) / 6.0, 1.0));
        side_colour = top_colour;
    }
    let sea = top_colour * settings.faces.x;
    return mix(sea, lit_land(top_colour, side_colour, slope, spot), clamp(land_share, 0.0, 1.0));
}

// The slope of the far land at a spot of a ring's tile (blocks up per block across, north and east).
fn land_slope(place: Place, cell: vec2<u32>, uv: vec2<f32>) -> vec2<f32> {
    let tri = triangle(uv);
    let w = tri.w;
    return corner_at(place, cell.x + tri.a.x, cell.y + tri.a.y).ground.zw * w.x
        + corner_at(place, cell.x + tri.b.x, cell.y + tri.b.y).ground.zw * w.y
        + corner_at(place, cell.x + tri.c.x, cell.y + tri.c.y).ground.zw * w.z;
}

// This pixel's ray, its neighbours' (right and down) and how far it went, for the contour bands'
// width; `band_t` below 0: no bands (set by the fragment, read by `band_width`).
var<private> band_ray: vec3<f32>;
var<private> band_ray_x: vec3<f32>;
var<private> band_ray_y: vec3<f32>;
var<private> band_t: f32 = -1.0;

// How many blocks of height this pixel spans on ground with that slope (blocks up per block across,
// north and east): from where the next pixels' rays meet its plane. Below 0: no bands.
fn band_width(slope: vec2<f32>) -> f32 {
    if band_t < 0.0 {
        return -1.0;
    }
    let normal = normalize(vec3<f32>(-slope.x, -slope.y, 1.0));
    let right = across_pixel(normal, band_ray, band_ray_x, band_t);
    let down = across_pixel(normal, band_ray, band_ray_y, band_t);
    return max(abs(right.z), abs(down.z));
}

// Colours lit as the far land is lit at a spot with that slope (lighting.wesl, `land`), its contour
// bands by the spot's height.
fn lit_land(top_colour: vec3<f32>, side_colour: vec3<f32>, slope: vec2<f32>, spot: vec3<f32>) -> vec3<f32> {
    return lit_land_at(top_colour, side_colour, slope, spot, drawn_at(spot).y);
}

// The same, the contour bands by `height` (a far copy's own ground's, for one).
fn lit_land_at(top_colour: vec3<f32>, side_colour: vec3<f32>, slope: vec2<f32>, spot: vec3<f32>, height: f32) -> vec3<f32> {
    let world_position = drawn_at(spot);
    let rise = length(slope);
    return land(
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
        vec2<f32>(height, band_width(slope)),
    );
}

// A far copy's solid cell that isn't plain ground (a mouth's rim, anything built or dug) where a ray
// met it, over the far land's tile at `place` (cell `cell`, `uv` across it): its block's own colours,
// lit as the land there is lit (its slope, the sun, the hill shading); its daylight and glow are the
// caller's (`copy_light`). [Build 21 lit them by their faces alone, so a snowy rim
// came out flat white against the shaded land: David, "the far view march is just white".]
fn copy_lit(value: u32, place: Place, cell: vec2<u32>, uv: vec2<f32>, spot: vec3<f32>) -> vec3<f32> {
    let block = value & 0xFFFu;
    var top_colour = block_colours[block * 4u].rgb;
    var side_colour = block_colours[block * 4u + 1u].rgb;
    if settings.shade.z > 0.5 {
        top_colour = vec3<f32>(0.5);
        side_colour = vec3<f32>(0.5);
    }
    // Who drew this: the march's far copies green.
    if settings.view.x > 0.5 {
        top_colour = vec3<f32>(0.3, 0.85, 0.4);
        side_colour = top_colour;
    }
    // Water: flat, as the sea is.
    if ((value >> 12u) & 3u) == 2u {
        return top_colour * settings.faces.x;
    }
    return lit_land(top_colour, side_colour, land_slope(place, cell, uv), spot);
}

// Where a ray from `origin` along `ray`, at `t`, meets the far land before `until` (both distances
// along it): `t` itself if it's under the land already, else where it comes down onto either of the
// land's triangles there, or -1 if it doesn't. For a far copy's plain cells, which then draw the
// land's own smooth surface through them, not their box [David, build 21: slabs sticking out where
// a coarse copy's peak fell away, by the real chunks' edge]. A copy's cell (8 blocks at most) lies
// within one cell of the land's, so its triangles are all there is.
fn land_through(origin: vec3<f32>, ray: vec3<f32>, t: f32, until: f32) -> f32 {
    let p = origin + ray * t;
    let place = locate(p.xy);
    if !place.found {
        return t;
    }
    let cell = place.size / CELLS;
    let local = ((to_game(view.world_position).xy + (world_origin().xy - place.origin)) + ray.xy * t) / cell;
    let ij = clamp(floor(local), vec2<f32>(0.0), vec2<f32>(CELLS - 1.0));
    let i = u32(ij.x);
    let j = u32(ij.y);
    let a = eased_height(place, i, j);
    let b = eased_height(place, i + 1u, j);
    let c = eased_height(place, i + 1u, j + 1u);
    let d = eased_height(place, i, j + 1u);
    if min(min(a, b), min(c, d)) <= MISSING {
        return t;
    }
    let uv = local - ij;
    if p.z <= surface(a, b, c, d, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))) {
        return t;
    }
    let rate = ray.xy / cell;
    var nearest = 1.0e9;
    for (var half = 0; half < 2; half++) {
        let first_half = half == 0;
        let gu = select(c - d, b - a, first_half);
        let gv = select(d - a, c - b, first_half);
        let gap = p.z - (a + gu * uv.x + gv * uv.y);
        let closing = ray.z - gu * rate.x - gv * rate.y;
        if gap >= 0.0 && closing < 0.0 {
            let at = t + gap / -closing;
            let there = uv + rate * (at - t);
            let across = there.x - there.y;
            if at <= until && select(across <= 1e-3, across >= -1e-3, first_half) {
                nearest = min(nearest, at);
            }
        }
    }
    return select(-1.0, nearest, nearest < 1.0e9);
}

// Whether a ray leaving copy cell `c` the way `axis` says (0 north-south, 1 east-west, 2 up-down)
// comes out into open air (full daylight, never covered), or out of the copy's sides or top.
fn opens_out(record: u32, c: vec3<i32>, axis: f32, rate: vec3<f32>) -> bool {
    var next = c;
    if axis < 0.5 {
        next.x = next.x + select(-1, 1, rate.x > 0.0);
    } else if axis < 1.5 {
        next.y = next.y + select(-1, 1, rate.y > 0.0);
    } else {
        next.z = next.z + select(-1, 1, rate.z > 0.0);
    }
    if next.x < 0 || next.x > 31 || next.y < 0 || next.y > 31 {
        return true;
    }
    let value = copy_cell(record, next);
    return ((value >> 12u) & 3u) == 0u && ((value >> 16u) & 15u) == 15u;
}

// The face of a whole copy's cell a ray met (`value` packed as above; `local`, `rate` as for
// `copy_colour`), at `spot`, `distance` along the ray: its block's texture for that face, at the
// size a block is on screen there (the textures' smaller versions further off), where on the face
// the ray met it, shaded by which way the face points. [The near-march experiment, `docs/far-view.md`:
// David, "lets do it".]
fn copy_textured(value: u32, local: vec3<f32>, rate: vec3<f32>, spot: vec3<f32>, distance: f32) -> vec3<f32> {
    let into = local - floor(local);
    let since = vec3<f32>(since_entering(into.x, rate.x), since_entering(into.y, rate.y), since_entering(into.z, rate.z));
    var axis = 2u;
    var forward = rate.z > 0.0;
    if since.x < since.y && since.x < since.z {
        axis = 0u;
        forward = rate.x > 0.0;
    } else if since.y < since.z {
        axis = 1u;
        forward = rate.y > 0.0;
    }
    // Which face: the side the ray came from (north 0, south 1, east 2, west 3, top 4, bottom 5).
    let face = axis * 2u + select(0u, 1u, forward);
    let block = value & 0xFFFu;
    let first = block_colours[block * 4u + 2u];
    let last = block_colours[block * 4u + 3u];
    var layer = last.x;
    if face == 0u {
        layer = first.x;
    } else if face == 1u {
        layer = first.y;
    } else if face == 2u {
        layer = first.z;
    } else if face == 3u {
        layer = first.w;
    } else if face == 5u {
        layer = last.y;
    }
    let f = spot - floor(spot);
    var uv = vec2<f32>(f.x, f.y);
    if axis == 0u {
        uv = vec2<f32>(f.y, 1.0 - f.z);
    } else if axis == 1u {
        uv = vec2<f32>(f.x, 1.0 - f.z);
    }
    // A block's size on screen (pixels), and so which of the texture's sizes to read.
    let across = f32(textureDimensions(block_textures).x);
    let pixels = view.viewport.w * 0.5 * view.clip_from_view[1][1] / max(distance, 1.0);
    let level = clamp(log2(max(across / max(pixels, 1e-3), 1.0)), 0.0, f32(textureNumLevels(block_textures) - 1u));
    var colour = textureSampleLevel(block_textures, block_sampler, uv, i32(layer), level).rgb;
    if settings.shade.z > 0.5 {
        colour = vec3<f32>(0.5);
    }
    // Who drew this: the march's whole copies a paler green.
    if settings.view.x > 0.5 {
        colour = vec3<f32>(0.55, 0.95, 0.6);
    }
    var shade = settings.faces.x;
    if axis == 0u {
        shade = settings.faces.y;
    } else if axis == 1u {
        shade = settings.faces.z;
    } else if face == 5u {
        shade = settings.faces.x * 0.5;
    }
    return colour * shade;
}

// The near window's cone map at `uv` (0 to 1 across the window), blended between its four nearest
// pixels (as terrain.wesl's `cones_at`).
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

// How lit a spot on the ground is, as the meshes light it (terrain.wesl's `ground_light`, the sums
// lighting.wesl's `ground_lit`): the light map, the window of the ground round you and its cone map,
// read here. `world_position` as drawn (Bevy's axes, from the floating origin). `size`: the cell's, in
// blocks. A coarser cell's top can stand up to its size less a block off the ground's own, and the cone
// map is block-accurate, so a top less than that below the cones isn't darkened (build 36's dark tops:
// David, "i think you're right"), the deeper kept: a hill's shadow carries on past the line (build 38,
// David: with the step shadows and foot shading at 0, "pretty much seamless").
fn near_ground_light(world_position: vec3<f32>, face: u32, size: f32) -> f32 {
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
            cones = cones_at(uv) - (size - 1.0);
        }
    }
    return ground_lit(map_light, near, cones, face, up, lighting.sun, lighting.hill, lighting.shade, lighting.sides);
}

// How far the spot where the next pixel's ray (`other`) meets a face's plane is from where this
// pixel's (`ray`, `t` along it) met it, in the plane: what a block's texture changes by from one pixel
// to the next, as the graphics card measures it for the meshes. From the eye, so small numbers, never
// two places in the world taken apart [build 13's jiggle]. At a grazing angle, where the next ray
// nearly runs along the plane, it's taken no further than 8 times as far.
fn across_pixel(normal: vec3<f32>, ray: vec3<f32>, other: vec3<f32>, t: f32) -> vec3<f32> {
    let facing = dot(normal, ray);
    let next = dot(normal, other);
    var s = 8.0 * t;
    if facing * next > 0.0 {
        s = min(t * facing / next, 8.0 * t);
    }
    let d = other * s - ray * t;
    return d - normal * dot(normal, d);
}

// A whole copy's cell a ray met, shaded as the meshes shade a block (one shading recipe,
// lighting.wesl): the face the ray came in through (`value`, `local`, `rate` as for `copy_colour`)
// on the block it belongs to, found from the copy's corner (`record`) in whole blocks; its texture
// placed and turned as the meshes place and turn it, at the size the meshes would read (from how far
// it moves to the next pixel's ray, `ray_x` and `ray_y`); lit by the face shade, the ground light and
// the underground light (`daylight` and `glow`, 0 to 1, from the air the ray came through, as a mesh
// takes them from the air in front of its face). `world_position` as drawn. A coarser cell (sizes per
// ray: `level` 1 to 3, `local` and `rate` in that level's cells) the same, as a copy's own cells of its
// size are, so a cell of a size looks alike in either [flat colours there, build 40, against the next
// copies' textured cells of the same size: David, "against the side of mountains, there's still a
// mismatch"]; "who drew this" shows its size: yellow, orange, red.
fn copy_shaded(value: u32, local: vec3<f32>, rate: vec3<f32>, level: u32, record: u32, ray: vec3<f32>, ray_x: vec3<f32>, ray_y: vec3<f32>, t: f32, world_position: vec3<f32>, daylight: f32, glow: vec3<f32>) -> vec3<f32> {
    let into = local - floor(local);
    let since = vec3<f32>(since_entering(into.x, rate.x), since_entering(into.y, rate.y), since_entering(into.z, rate.z));
    var axis = 2u;
    var forward = rate.z > 0.0;
    if since.x < since.y && since.x < since.z {
        axis = 0u;
        forward = rate.x > 0.0;
    } else if since.y < since.z {
        axis = 1u;
        forward = rate.y > 0.0;
    }
    // Which face: the side the ray came from (north 0, south 1, east 2, west 3, top 4, bottom 5).
    let face = axis * 2u + select(0u, 1u, forward);
    let normal = face_normal(face);
    // Where it met the face, in blocks from the copy's corner (whole numbers, so a block's place in
    // it is exact), and the block the face is on: half a block in from it, as the meshes find it.
    let base = record * COPY_RECORD;
    let corner = vec3<i32>(bitcast<i32>(copy_records[base]), bitcast<i32>(copy_records[base + 1u]), bitcast<i32>(copy_records[base + 2u]));
    let scale = copy_records[base + 3u] << level;
    let p = local * f32(scale);
    let at_block = corner + vec3<i32>(floor(p - normal * 0.5));
    let block = value & 0xFFFu;
    let first = block_colours[block * 4u + 2u];
    let last = block_colours[block * 4u + 3u];
    var layer = last.x;
    if face == 0u {
        layer = first.x;
    } else if face == 1u {
        layer = first.y;
    } else if face == 2u {
        layer = first.z;
    } else if face == 3u {
        layer = first.w;
    } else if face == 5u {
        layer = last.y;
    }
    // Placed as the meshes place it (the corner is whole, so only whole blocks apart from the mesh's
    // own numbers: the same picture), its size from the unturned place, as theirs.
    var uv = face_uv(face, p);
    let uv_dx = face_uv(face, across_pixel(normal, ray, ray_x, t));
    let uv_dy = face_uv(face, across_pixel(normal, ray, ray_y, t));
    if ((u32(last.z) >> face) & 1u) == 1u && near_settings.options.x > 0.5 {
        uv = turned(uv, at_block, face);
    }
    var colour = textureSampleGrad(block_textures, block_sampler, uv, i32(layer), uv_dx, uv_dy).rgb;
    if near_settings.options.y > 0.5 {
        colour = vec3<f32>(0.5);
    }
    // Who drew this: the march's whole copies a paler green, their coarser cells by size.
    if settings.view.x > 0.5 {
        colour = vec3<f32>(0.55, 0.95, 0.6);
        if level > 0u {
            colour = select(select(vec3<f32>(1.0, 0.35, 0.3), vec3<f32>(1.0, 0.65, 0.3), level == 2u), vec3<f32>(0.95, 0.9, 0.4), level == 1u);
        }
    }
    let lit = face_shade(face, near_settings.face_shade) * near_ground_light(world_position, face, f32(scale));
    return colour * lit * underground_lit(daylight, glow, length(world_position - view.world_position), near_settings.underground, near_settings.placed, globals.time);
}

// A whole copy's solid cell of 2 blocks or more (its own, or a coarser level's: `level`) on the open
// ground, where a ray met it at `spot`, drawn as the far land draws its ground (`lit_land_at`: the
// land's slope and light), in its block's colours, its contour bands by the copy's own ground: its
// columns' tops eased between their middles, so each band falls where a step of real blocks is
// [round 2: a 2- or 4-block cell's steps showed 2 or 4 blocks tall, wide grey and red-brown stripes
// in the middle distance, David: "the bad banding in a middle-band of the rendering"]. Under the
// ground's top (an overhang, a cave's mouth) its blocks stay: w 0, as for no bands.
fn copy_banded(value: u32, record: u32, level: u32, spot: vec3<f32>) -> vec4<f32> {
    if march.copies.w < 0.5 || ((value >> 12u) & 3u) != 1u {
        return vec4<f32>(0.0);
    }
    let base = record * COPY_RECORD;
    let scale = f32(copy_records[base + 3u]);
    let size = scale * f32(1u << level);
    if size < 1.5 {
        return vec4<f32>(0.0);
    }
    let corner = vec3<f32>(f32(bitcast<i32>(copy_records[base])), f32(bitcast<i32>(copy_records[base + 1u])), f32(bitcast<i32>(copy_records[base + 2u])));
    let at = (spot.xy - corner.xy) / scale;
    let own = clamp(floor(at), vec2<f32>(0.0), vec2<f32>(31.0));
    let column = corner.z + scale * copy_top(record, COLUMNS_AT, u32(own.y * 32.0 + own.x));
    if spot.z < column - 1.5 * size {
        return vec4<f32>(0.0);
    }
    let place = locate(spot.xy);
    if !place.found {
        return vec4<f32>(0.0);
    }
    // The ground's height here: the four columns' tops round the spot, eased between their middles
    // (a step's middle, where its side is, half a block below the higher top).
    let q = at - 0.5;
    let low = clamp(floor(q), vec2<f32>(0.0), vec2<f32>(31.0));
    let high = min(low + 1.0, vec2<f32>(31.0));
    let f = clamp(q - low, vec2<f32>(0.0), vec2<f32>(1.0));
    let a = copy_top(record, COLUMNS_AT, u32(low.y * 32.0 + low.x));
    let b = copy_top(record, COLUMNS_AT, u32(low.y * 32.0 + high.x));
    let c = copy_top(record, COLUMNS_AT, u32(high.y * 32.0 + low.x));
    let d = copy_top(record, COLUMNS_AT, u32(high.y * 32.0 + high.x));
    let ground = corner.z + scale * mix(mix(a, b, f.x), mix(c, d, f.x), f.y) - 0.5 * scale;
    let local = (spot.xy - place.origin) / (place.size / CELLS);
    let ij = clamp(floor(local), vec2<f32>(0.0), vec2<f32>(CELLS - 1.0));
    let uv = clamp(local - ij, vec2<f32>(0.0), vec2<f32>(1.0));
    let block = value & 0xFFFu;
    var top_colour = block_colours[block * 4u].rgb;
    var side_colour = block_colours[block * 4u + 1u].rgb;
    if settings.shade.z > 0.5 {
        top_colour = vec3<f32>(0.5);
        side_colour = vec3<f32>(0.5);
    }
    // Who drew this: as the copy's cells of its size (pale green its own, then yellow, orange, red).
    if settings.view.x > 0.5 {
        top_colour = vec3<f32>(0.55, 0.95, 0.6);
        if level > 0u {
            top_colour = select(select(vec3<f32>(1.0, 0.35, 0.3), vec3<f32>(1.0, 0.65, 0.3), level == 2u), vec3<f32>(0.95, 0.9, 0.4), level == 1u);
        }
        side_colour = top_colour;
    }
    return vec4<f32>(lit_land_at(top_colour, side_colour, land_slope(place, vec2<u32>(ij), uv), spot, ground), 1.0);
}

// This pixel's ray (`ndc`: where it is on the screen, -1 to 1 each way), in game axes: worked out
// from the camera, then turned to the world; never from two points in the world.
fn pixel_ray(ndc: vec2<f32>) -> vec3<f32> {
    let on_near = view.view_from_clip * vec4<f32>(ndc, 1.0, 1.0);
    let toward = normalize(on_near.xyz / on_near.w);
    return normalize(to_game((view.world_from_view * vec4<f32>(toward, 0.0)).xyz));
}

// How long ago (along the ray) it came in through a cell's face this way, `into` across the cell.
fn since_entering(into: f32, rate: f32) -> f32 {
    if abs(rate) < 1e-9 {
        return 1.0e9;
    }
    return select(1.0 - into, into, rate > 0.0) / abs(rate);
}

// A far copy's cell where a ray met it (`value` packed as above; `local` where the ray was, in its
// cells, and `rate` how fast that changes), where it's not over a land tile: its block's colour, top
// or side by the face it came in through, shaded by which way that faces (its daylight and glow are
// the caller's, `copy_light`).
fn copy_colour(value: u32, local: vec3<f32>, rate: vec3<f32>) -> vec3<f32> {
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
    var colour = select(block_colours[block * 4u + 1u].rgb, block_colours[block * 4u].rgb, up);
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
    return colour * shade;
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
    // The camera in the world, and from the floating origin (small, so exact: what positions that
    // must be exact are measured from).
    let origin = camera_game();
    let eye = to_game(view.world_position);
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
    var hit_record = 0u;
    // The size of the cell met (sizes per ray): 0 the copy's own, 1 to 3 its coarser levels.
    var hit_level = 0u;
    var last_light = 15u;
    var last_glow = 0u;
    for (var k = 0u; k < most; k++) {
        if t > furthest {
            break;
        }
        let p = origin + ray * t;
        if ray.z >= 0.0 && p.z > highest {
            break;
        }
        // In a chunk a far copy draws: through its cells, one at a time (step 1 part 5).
        // Looked up a 32nd of a block on the way it's going: a ray almost along a copy's side, leaving
        // it, moves across by less than the world's numbers keep far out (a few thousandths), so
        // found in the copy it left, again and again, it ran out of steps [lines of sky straight up the
        // screen, moved only by moving sideways: David, build 41].
        let record = copy_at(p.xy + sign(ray.xy) * LOOKUP_AHEAD);
        if record >= 0 {
            let base = u32(record) * COPY_RECORD;
            let corner = vec3<f32>(f32(bitcast<i32>(copy_records[base])), f32(bitcast<i32>(copy_records[base + 1u])), f32(bitcast<i32>(copy_records[base + 2u])));
            let scale = f32(copy_records[base + 3u]);
            let tall = f32(copy_records[base + 4u]);
            // From the copy's corner, not the world's middle: thousands of blocks out a position is kept
            // only to a few thousandths of a block, coarser than the nudge into the next cell, so a
            // ray could round back into the cell it left and step there till it ran out (sky specks in a
            // far cave, at (-4343, 17800): David, build 23).
            let local = ((eye + (world_origin() - corner)) + ray * t) / scale;
            let rate = ray / scale;
            let nudge = scale * 1e-3;
            let c = vec3<i32>(clamp(floor(local), vec3<f32>(0.0, 0.0, -1.0), vec3<f32>(31.0, 31.0, tall)));
            if f32(c.z) >= tall {
                // Above it: on to where the ray leaves the air over it, or comes down to its top.
                let out = leave_box(t, local, rate, vec3<f32>(0.0, 0.0, tall), vec3<f32>(32.0, 32.0, 1.0e9));
                was_above = true;
                last_light = 15u;
                last_glow = 0u;
                t = out.x + nudge;
                continue;
            }
            // The cell the ray is in, at the size wanted there (sizes per ray: `sized_level`), and
            // where the ray leaves it. Each spot's size is fixed by the world, so a ray only ever comes
            // into a cell through one of its faces.
            let level = sized_level(u32(record), c, t);
            let size = f32(1u << level);
            let at_level = c >> vec3<u32>(level);
            var value = 0u;
            if level == 0u {
                value = copy_cell(u32(record), c);
            } else {
                value = level_cell(u32(record), level, at_level);
            }
            let out = leave_box(t, local, rate, vec3<f32>(at_level) * size, (vec3<f32>(at_level) + 1.0) * size);
            // Water the near world draws itself (see-through, past the line): on through it as
            // through air, the air's light kept, to the bed, which the water's drawn over as inside
            // the line. [Drawn as the march's own water under it, each copy size a shade of its own:
            // David, build 39, "the border chunks ... make squares in the water appear".]
            if ((value >> 12u) & 3u) == 2u && water_drawn_near(p.xy) {
                t = out.x + nudge;
                continue;
            }
            if ((value >> 12u) & 3u) != 0u {
                // In the ground: gone in under the near world.
                if !was_above || drawn_near(p.xy) {
                    break;
                }
                // Plain ground: the land's own surface through the cell, if it passes through it;
                // where it doesn't (the cell stands above it), on as through air.
                var at = t;
                if (value & PLAIN) != 0u && ((value >> 12u) & 3u) == 1u {
                    at = land_through(origin, ray, t, out.x);
                    // Standing above the land: passed through only if the ray comes out of it into
                    // open air, as from a box sticking up out of the ground. Into a cave's air, or
                    // more ground, it's the cell's face after all. [On a sharp peak the smooth land
                    // runs below the blocks, and rays went through the cells round a shallow cave,
                    // through the cave and out to the sky: David, build 22.]
                    if at < 0.0 && opens_out(u32(record), c, out.y, rate) {
                        last_light = 15u;
                        last_glow = 0u;
                        t = out.x + nudge;
                        continue;
                    }
                    at = max(at, t);
                }
                // Solid, or water: met, coming from the air.
                hit = true;
                hit_copy = true;
                hit_value = value;
                // In the cells of its size, for which face it came in through.
                hit_local = local / size;
                hit_rate = rate / size;
                hit_record = u32(record);
                hit_level = level;
                t = at;
                break;
            }
            was_above = true;
            last_light = (value >> 16u) & 15u;
            last_glow = (value >> 20u) & 0xFFFu;
            // Where it might draw coarser cells: past everything solid in its square of 8 x 8
            // columns (four groups), rounded up to a coarsest cell, in one step. A coarser cell can
            // stand above its own columns' highest (it's solid for the columns round it), so the
            // groups' and columns' own steps below could pass through it.
            if may_be_coarse(u32(record), t) {
                let square = c.xy / 8;
                let square_top = coarse_top(u32(record), square);
                if f32(c.z) >= square_top {
                    let past = leave_box(t, local, rate, vec3<f32>(vec2<f32>(square * 8), square_top), vec3<f32>(vec2<f32>(square * 8 + 8), 1.0e9));
                    last_light = 15u;
                    last_glow = 0u;
                    t = past.x + nudge;
                    continue;
                }
                t = out.x + nudge;
                continue;
            }
            // Above every block in its group of 4 x 4 columns, or in its column: past them in one
            // step (open air, full daylight). [Build 30: a ray down past a mountain stepped a cell at a
            // time through the band of whole copies, ran out of steps and showed sky.]
            let group = c.xy / 4;
            let group_top = copy_top(u32(record), GROUPS_AT, u32(group.y * 8 + group.x));
            if f32(c.z) >= group_top {
                let past = leave_box(t, local, rate, vec3<f32>(vec2<f32>(group * 4), group_top), vec3<f32>(vec2<f32>(group * 4 + 4), 1.0e9));
                last_light = 15u;
                last_glow = 0u;
                t = past.x + nudge;
                continue;
            }
            let column_top = copy_top(u32(record), COLUMNS_AT, u32(c.y * 32 + c.x));
            if f32(c.z) >= column_top {
                let past = leave_box(t, local, rate, vec3<f32>(vec2<f32>(c.xy), column_top), vec3<f32>(vec2<f32>(c.xy + 1), 1.0e9));
                last_light = 15u;
                last_glow = 0u;
                t = past.x + nudge;
                continue;
            }
            t = out.x + nudge;
            continue;
        }
        // Out on the land: open daylight, no glow.
        last_light = 15u;
        last_glow = 0u;
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
        // From the tile's corner, as in a copy (above): never the world's big numbers.
        let local = ((eye.xy + (world_origin().xy - place.origin)) + ray.xy * t) / cell;
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
        // Over the real chunks, only far deeper: there the near world says what's solid, and the smooth
        // land can stand far above its blocks (over a cliff's drop), so a ray going on down past a ledge
        // was given up and showed sky [David, build 29: "gaps still present, looking down a ledge"].
        if p.z < under - select(max(16.0, 2.0 * cell), max(64.0, 4.0 * cell), drawn_near(p.xy)) {
            break;
        }
        let leaving = clamp(local + rate * (out_of_cell - t) - ij, vec2<f32>(0.0), vec2<f32>(1.0));
        // Over the real chunks, in the air: the near world draws whatever's solid there, and the
        // smooth land can run a block or two above its blocks, so a ray over them but under it was
        // taken for one in the ground, and turned away by a far copy next door [David, build 28: sky
        // between the real chunks and the band of whole copies].
        was_above = drawn_near(p.xy) || origin.z + ray.z * out_of_cell >= surface(a, b, c, d, leaving);
        t = out_of_cell + nudge;
    }
    if !hit {
        discard;
    }
    let spot = origin + ray * t;
    // As drawn: from the floating origin, from the camera's place there (exact, not the world's big numbers).
    let world_position = to_bevy(eye + ray * t);
    var colour = vec3<f32>(0.0);
    // Lit already (a whole copy shaded as the meshes are, light and all): none of the light below.
    var lit = false;
    let glows = vec3<f32>(f32(last_glow & 15u), f32((last_glow >> 4u) & 15u), f32((last_glow >> 8u) & 15u)) / 15.0;
    // The next pixels' rays, right and down: for a texture's size, and the contour bands' width.
    let ray_x = pixel_ray(ndc + vec2<f32>(2.0 / view.viewport.z, 0.0));
    let ray_y = pixel_ray(ndc - vec2<f32>(0.0, 2.0 / view.viewport.w));
    if march.copies.w > 0.5 {
        band_ray = ray;
        band_ray_x = ray_x;
        band_ray_y = ray_y;
        band_t = t;
    }
    var banded = vec4<f32>(0.0);
    if hit_copy && copy_records[hit_record * COPY_RECORD + 6u] == 1u && march.copies.y > 0.5 {
        banded = copy_banded(hit_value, hit_record, hit_level, spot);
    }
    if banded.w > 0.5 {
        // A copy's cell of 2 blocks or more on the open ground: the far land's look, in contour
        // bands by the copy's own ground (lit below, as the land is).
        colour = banded.rgb;
    } else if hit_copy && copy_records[hit_record * COPY_RECORD + 6u] == 1u && march.copies.y > 0.5 {
        // Copied whole, shaded as the meshes shade the same block (one shading recipe), its coarser
        // cells too (sizes per ray).
        colour = copy_shaded(hit_value, hit_local, hit_rate, hit_level, hit_record, ray, ray_x, ray_y, t, world_position, f32(last_light) / 15.0, glows);
        lit = true;
    } else if hit_copy && hit_level > 0u {
        // A coarser cell, as build 35's march coloured a copy's blocks (lit below).
        colour = copy_colour(hit_value, hit_local, hit_rate);
    } else if hit_copy && copy_records[hit_record * COPY_RECORD + 6u] == 1u {
        // Copied whole (the near-march experiment): the block's own texture on the face the ray
        // came in through, as the near world draws it.
        colour = copy_textured(hit_value, hit_local, hit_rate, spot, t);
    } else if hit_copy {
        // Plain ground in a copy takes the far land's look there, so the copy doesn't show as a patch;
        // the rest keep their blocks' colours, lit as the land there is.
        let place = locate(spot.xy);
        if place.found {
            let local = (spot.xy - place.origin) / (place.size / CELLS);
            let ij = clamp(floor(local), vec2<f32>(0.0), vec2<f32>(CELLS - 1.0));
            let uv = clamp(local - ij, vec2<f32>(0.0), vec2<f32>(1.0));
            if (hit_value & PLAIN) != 0u {
                colour = land_colour(place, vec2<u32>(ij), uv, spot);
            } else {
                colour = copy_lit(hit_value, place, vec2<u32>(ij), uv, spot);
            }
        } else {
            colour = copy_colour(hit_value, hit_local, hit_rate);
        }
    } else {
        colour = land_colour(hit_place, hit_cell, hit_uv, spot);
    }

    // From here on, as the sheet's fragment part.
    let light_uv = vec2<f32>(spot.y + settings.light_area.x, settings.light_area.x - spot.x) / max(settings.light_area.y, 1.0);
    let light_pixel = textureSampleLevel(light_map, light_sampler, light_uv, 0.0);
    // Daylight (dimmed by the light map's shadows and the sky's brightness: the stand-in for night)
    // and, in a far copy, placed light (a brazier's, step 1 part 6), which neither dims. A copy's cell
    // takes both from the air the ray came through, eased (squared) as the near ground's are
    // underground (terrain.wesl, `underground_light`); the land is in the open.
    var day = settings.view.z;
    if settings.light.x > 0.5 {
        day = day * light_from_map(light_pixel, settings.light);
    }
    var glow = vec3<f32>(0.0);
    if hit_copy {
        let daylight = f32(last_light) / 15.0;
        day = day * daylight * daylight;
        glow = glows * glows;
    }
    if !lit {
        colour = colour * min(vec3<f32>(day) + glow, vec3<f32>(1.0));
    }
    if settings.survey.x > 0.5 {
        let z = i32(floor(world_position.y - 0.01));
        let every = max(i32(settings.survey.y), 1);
        let level = z - i32(settings.survey.w);
        if ((level % every) + every) % every == 0 {
            colour = colour * (1.0 - settings.survey.z);
        }
    }
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

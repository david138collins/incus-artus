// ── The far eye (plan 2.2, step 2a): old's far march on bricks ──
//
// Old's far march (`old/src/sight/march.wgsl`, its far part), its logic as it was; only where the cells live is new.
// A ray reads the level whose cells are about its pixel's size at that distance, skips space a coarser cell column
// says is empty (over its band's top) and regions not summed yet, and elsewhere takes half a cell at a time, reading a
// smooth blend of the eight cells round it (no threshold: a part-solid cell covers part of the pixel, gathered front to
// back; colour and facing weighted by how much surface each cell shows), lit as its faces are (each of the six ways a
// block's faces look, lit by the sun as itself, weighted by how much of that way the eye sees). The sky behind, and
// fog between.
//
// The cells (`bricks.rs`): each level's columns' bands (`entries`: the lowest kept cell and how many; solid below,
// empty above), and the bands' cells in bricks (`index` gives a brick's place in `pool`, by arithmetic).
//
// And old's light (2b): the sun's shadow (a widening ray towards the sun through the same cells) and the sky light
// (five wide cones up), only where a pixel shows.

struct Camera {
    // The eye (game axes: x north, y east, z up).
    position: vec4<f32>,
    forward: vec4<f32>,
    // Scaled so a pixel's direction is forward + x·right + y·up for x, y from −1 to 1.
    right: vec4<f32>,
    up: vec4<f32>,
    // x, y: the layer's size; z: a layer pixel's angle (radians); w: rays a pixel.
    view: vec4<f32>,
    // xyz: towards the sun; w: the ambient light.
    sun: vec4<f32>,
    // x: the farthest a ray goes (blocks); y: the level bias (old's: more, coarser; the Render page's detail turned
    // round, so more detail is finer); z: 1 the steps' view, 2 the levels' view; w: the highest
    // anything stands (blocks).
    reach: vec4<f32>,
    // The start pass: x, y the tiles across and down; z a tile's side in this layer's pixels (0: no start pass); w the
    // cone's spread (radians from a tile's middle ray to its corners).
    tiles: vec4<f32>,
    // The screen pyramid: x the layer above's width in its pixels (0: this is the base); y 1 if a finer layer follows;
    // z the rule (0 refine everywhere, 1 only where it's earned); w how the far cells are blended (0 the code, 1 the
    // texture hardware: change 1).
    layer: vec4<f32>,
    // x: 1 if a finer layer's rays start a little short of the nearest touch round their parent pixel; y: 1 for sun
    // shadows; z: how fast a shadow ray widens (blocks across a block of distance).
    follow: vec4<f32>,
    // The pyramid's steps: x this layer's factor over the one above's; y the one below's over this one's.
    pyramid: vec4<f32>,
    // The finer layer's fade in: x where it starts (its pixel so many times its cells), y over how many doublings of
    // distance; z where in a level's stretch its blend with the next begins (the hand-over); w the sky's weight (0: as
    // old's; more: a pixel the land only partly covers gives more of itself to the sky).
    fade: vec4<f32>,
    // Sky light: x 1 for it on, y its strength (how dark a place seeing no sky goes), z the tint (0 white, 1 a sky
    // blue), w its cones' width (blocks across a block of distance).
    sky: vec4<f32>,
    // Fog: x its thickness at sea level (a block's worth; 0 none), y how fast it thins going up, z sea level, w how much
    // warmer it is towards the sun.
    fog: vec4<f32>,
    // The time of day: x the sun's own light, y the sky's light, z the sky's own brightness, w night's light.
    daylight: vec4<f32>,
    // The world's water: its colour as light, and how fast it dims what's seen through it (a block's worth).
    water: vec4<f32>,
    // The far bricks' atlas (change 1): its size in texels (1s while there's none).
    atlas: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
// Each level's numbers (`bricks::table`, `TABLE` each): the level, its least column (north, east), columns across,
// where its columns start in `entries`, where its bricks start in `index`, a brick's width and height (shifts), bricks
// across and up, cells up.
@group(0) @binding(1) var<storage, read> levels: array<i32>;
@group(0) @binding(2) var<storage, read> entries: array<u32>;
@group(0) @binding(3) var<storage, read> index: array<u32>;
@group(0) @binding(4) var<storage, read> pool: array<vec4<u32>>;
@group(0) @binding(5) var picture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var how_far: texture_storage_2d<r32float, write>;
// Where each tile's rays start (the start pass's answers).
@group(0) @binding(7) var<storage, read_write> starts: array<f32>;
// The pyramid: how much each of the layer above's pixels says its children are worth (255ths) in its low 8 bits, its
// sky light handed down over them (2b); and this layer's word for the layer below.
@group(0) @binding(8) var<storage, read> worth_in: array<u32>;
@group(0) @binding(9) var<storage, read_write> worth_out: array<u32>;
// The layer above's distances (the base: a placeholder never read).
@group(0) @binding(10) var far_above: texture_2d<f32>;
// The far bricks as textures (change 1, `atlas.rs`): each brick place's slot in the atlas, the filter, how solid and
// how watery, and the colour and faces spread into empty neighbours (with the surface ÷ 4).
@group(0) @binding(11) var<storage, read> slots: array<u32>;
@group(0) @binding(12) var soft: sampler;
@group(0) @binding(13) var t_solid: texture_3d<f32>;
@group(0) @binding(14) var t_colour: texture_3d<f32>;
@group(0) @binding(15) var t_plus: texture_3d<f32>;
@group(0) @binding(16) var t_minus: texture_3d<f32>;

const FIRST: i32 = 5;
const TOP: i32 = 16;
// The regions are the cell columns of 1,024.
const REGION: i32 = 10;
const TABLE: i32 = 12;
const NOT_SUMMED: u32 = 0xFFFF0000u;
// Outside a level's columns: past the map, nothing at all, not even the world's floor.
const PAST_MAP: u32 = 0xFFFE0000u;
const NO_BRICK: u32 = 0xFFFFFFFFu;
const WORLD_HEIGHT: f32 = 1024.0;
// Rock's colour, where a cell shows no surface (inside).
const ROCK: vec3<f32> = vec3<f32>(0.25, 0.24, 0.22);
const SKY_TINT: vec3<f32> = vec3<f32>(0.84, 1.01, 1.40);
// A pixel's sky light as handed down (2b): none.
const NO_SKY: u32 = 0x70000u;

// ── The cells ──

// A cell as the march reads it.
struct Look {
    known: bool,
    solid: f32,
    water: f32,
    colour: vec3<f32>,
    // Its faces' shares by the way they look (0 to 1, adding to 1): `plus` looking north, east and up, `minus` south,
    // west and down.
    plus: vec3<f32>,
    minus: vec3<f32>,
    // How much surface it shows (the weight of its colour and faces).
    surface: f32,
}

fn empty_look(known: bool) -> Look {
    var look: Look;
    look.known = known;
    return look;
}

fn solid_look() -> Look {
    var look: Look;
    look.known = true;
    look.solid = 1.0;
    return look;
}

fn cell_of(w: vec4<u32>) -> Look {
    var look: Look;
    look.known = true;
    look.solid = f32(w.x & 0xFFu) / 255.0;
    look.water = f32((w.x >> 8u) & 0xFFu) / 255.0;
    // The six ways' shares (bytes 2 to 6: up, down, north, south, east; west what's left).
    let up = f32((w.x >> 16u) & 0xFFu);
    let down = f32(w.x >> 24u);
    let north = f32(w.y & 0xFFu);
    let south = f32((w.y >> 8u) & 0xFFu);
    let east = f32((w.y >> 16u) & 0xFFu);
    let west = max(255.0 - up - down - north - south - east, 0.0);
    look.plus = vec3<f32>(north, east, up) / 255.0;
    look.minus = vec3<f32>(south, west, down) / 255.0;
    look.colour = vec3<f32>(f32((w.z >> 20u) & 1023u), f32((w.z >> 10u) & 1023u), f32(w.z & 1023u)) / 1023.0;
    look.surface = min(f32(w.w >> 16u) / 256.0, 4.0);
    return look;
}

// A level's number `k` in the table.
fn table(level: i32, k: i32) -> i32 {
    return levels[(level - FIRST) * TABLE + k];
}

// A cell column's entry at `level`: its band's lowest cell and how many (`NOT_SUMMED` if it isn't in yet, `PAST_MAP`
// outside the level).
fn entry_at(level: i32, cx: i32, cy: i32) -> u32 {
    let x = cx - table(level, 1);
    let y = cy - table(level, 2);
    let across = table(level, 3);
    if (x < 0 || y < 0 || x >= across || y >= across) {
        return PAST_MAP;
    }
    return entries[u32(table(level, 4) + y * across + x)];
}

// A cell of a column's band, from its brick: the brick's place by arithmetic (shifts: no whole-number division, which
// the card has no instruction for), then the cell's place in it.
fn cell_word(level: i32, cx: i32, cy: i32, cz: i32) -> vec4<u32> {
    let wide = u32(table(level, 6));
    let tall = u32(table(level, 7));
    let x = u32(cx - table(level, 1));
    let y = u32(cy - table(level, 2));
    let z = u32(cz);
    let brick = (i32(y >> wide) * table(level, 8) + i32(x >> wide)) * table(level, 9) + i32(z >> tall);
    let start = index[u32(table(level, 5) + brick)];
    if (start == NO_BRICK) {
        return vec4<u32>(0u);
    }
    let mask = (1u << wide) - 1u;
    return pool[start + (x & mask) + ((y & mask) << wide) + ((z & ((1u << tall) - 1u)) << (2u * wide))];
}

// A band's entry read: its cell `cz` up, or solid below the band, or empty above it.
fn from_band(level: i32, cx: i32, cy: i32, e: u32, cz: i32) -> Look {
    if (e == NOT_SUMMED) {
        return empty_look(false);
    }
    if (e == PAST_MAP) {
        return empty_look(true);
    }
    let low = bitcast<i32>(e << 16u) >> 16u;
    let count = i32(e >> 16u);
    if (cz < 0 || (cz < low && low > 0)) {
        return solid_look();
    }
    if (cz < low || cz >= low + count) {
        return empty_look(true);
    }
    return cell_of(cell_word(level, cx, cy, cz));
}

// A smooth blend of the eight cells round `p` at `level` (trilinear: no threshold, so a slope's edge is a gradient,
// never stairs); `known` false if any of them isn't there. Each of the four cell columns' entries is read once, then
// its two cells.
fn blend(level: i32, p: vec3<f32>) -> Look {
    let size = f32(1 << u32(level));
    let u = p / size - 0.5;
    let b = floor(u);
    let f = u - b;
    let base = vec3<i32>(b);
    var out = empty_look(true);
    var weight = 0.0;
    for (var k = 0; k < 4; k++) {
        let ox = k & 1;
        let oy = k >> 1;
        let cx = base.x + ox;
        let cy = base.y + oy;
        let entry = entry_at(level, cx, cy);
        if (entry == NOT_SUMMED) {
            return empty_look(false);
        }
        let wxy = select(1.0 - f.x, f.x, ox == 1) * select(1.0 - f.y, f.y, oy == 1);
        for (var oz = 0; oz < 2; oz++) {
            let look = from_band(level, cx, cy, entry, base.z + oz);
            let w = wxy * select(1.0 - f.z, f.z, oz == 1);
            out.solid += w * look.solid;
            out.water += w * look.water;
            let shown = w * look.surface;
            out.colour += shown * look.colour;
            out.plus += shown * look.plus;
            out.minus += shown * look.minus;
            weight += shown;
        }
    }
    if (weight > 0.0) {
        out.colour /= weight;
        out.plus /= weight;
        out.minus /= weight;
        out.surface = weight;
    } else {
        // Inside, with no surface to show: rock's grey.
        out.colour = ROCK;
    }
    return out;
}

const NO_SLOT: u32 = 0xFFFFFFFFu;
// A brick's texels a side in the atlas, its border included.
const SIDE: f32 = 9.0;

// The eight cells round `p` at `level` blended by the texture hardware (change 1, the spread way). The four columns'
// bands are read first, as the code's blend reads them (a column not summed: unknown); where the bricks can't answer
// (outside the level, under or over the world, a brick not in the atlas), the code's blend.
fn blend_soft(level: i32, p: vec3<f32>) -> Look {
    let size = f32(1 << u32(level));
    let u = p / size - 0.5;
    let b = floor(u);
    let f = u - b;
    let base = vec3<i32>(b);
    for (var k = 0; k < 4; k++) {
        if (entry_at(level, base.x + (k & 1), base.y + (k >> 1)) == NOT_SUMMED) {
            return empty_look(false);
        }
    }
    let x = base.x - table(level, 1);
    let y = base.y - table(level, 2);
    if (x < 0 || y < 0 || base.z < 0 || x >= table(level, 3) || y >= table(level, 3) || base.z >= table(level, 10)) {
        return blend(level, p);
    }
    let wide = u32(table(level, 6));
    let tall = u32(table(level, 7));
    let place = table(level, 5) + (i32(u32(y) >> wide) * table(level, 8) + i32(u32(x) >> wide)) * table(level, 9) + i32(u32(base.z) >> tall);
    let slot = slots[u32(place)];
    if (slot == NO_SLOT) {
        return blend(level, p);
    }
    let origin = vec3<f32>(f32(slot & 0xFFu), f32((slot >> 8u) & 0xFFu), f32(slot >> 16u)) * SIDE;
    let local = vec3<f32>(f32(u32(x) & ((1u << wide) - 1u)), f32(u32(y) & ((1u << wide) - 1u)), f32(u32(base.z) & ((1u << tall) - 1u)));
    let at = (origin + local + f + 0.5) / camera.atlas.xyz;
    // How solid and how watery, at every step; the colour and faces only where something's there.
    let sw = textureSampleLevel(t_solid, soft, at, 0.0).xy;
    var look = empty_look(true);
    look.solid = sw.x;
    look.water = sw.y;
    look.colour = ROCK;
    if (sw.x + sw.y <= 0.0) {
        return look;
    }
    let c = textureSampleLevel(t_colour, soft, at, 0.0);
    if (c.a > 0.0) {
        look.colour = c.rgb;
        look.plus = textureSampleLevel(t_plus, soft, at, 0.0).rgb;
        look.minus = textureSampleLevel(t_minus, soft, at, 0.0).rgb;
        look.surface = c.a * 4.0;
    }
    return look;
}

// The eight cells round `p` blended: by the code (old's, `blend`) or the texture hardware, as chosen (`camera.layer.w`:
// 0 the code, 1 the hardware).
fn far_blend(level: i32, p: vec3<f32>) -> Look {
    if (camera.layer.w < 0.5) {
        return blend(level, p);
    }
    return blend_soft(level, p);
}

// Two looks mixed: `w` of the second.
fn mix_looks(a: Look, b: Look, w: f32) -> Look {
    let k = clamp(w, 0.0, 1.0);
    var out = a;
    out.solid = mix(a.solid, b.solid, k);
    out.water = mix(a.water, b.water, k);
    out.colour = mix(a.colour, b.colour, k);
    out.plus = mix(a.plus, b.plus, k);
    out.minus = mix(a.minus, b.minus, k);
    out.surface = mix(a.surface, b.surface, k);
    return out;
}

// ── The sky and the fog (old's) ──

const HORIZON: vec3<f32> = vec3<f32>(0.62, 0.72, 0.86);
const ZENITH: vec3<f32> = vec3<f32>(0.16, 0.32, 0.66);
// Night's sky as a share of the day's, at night's light 1.
const NIGHT_SKY: vec3<f32> = vec3<f32>(0.35, 0.45, 0.8);
// The dusk's glow low over the horizon where the sun went down.
const DUSK: vec3<f32> = vec3<f32>(1.0, 0.45, 0.18);

// The sky's day colour seen along `d`, darkened through the dusk to night's, and the dusk's glow where the sun went
// down, while it's near the horizon.
fn sky_tone(day: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    let night = day * NIGHT_SKY * camera.daylight.w;
    var colour = mix(night, day, camera.daylight.z);
    let height = asin(clamp(camera.sun.z, -1.0, 1.0)) * 57.29578;
    let dusk = exp(-pow((height + 2.0) / 4.0, 2.0));
    let flat_sun = normalize(vec3<f32>(camera.sun.xy, 0.0) + vec3<f32>(1e-6, 0.0, 0.0));
    let flat_d = normalize(vec3<f32>(d.xy, 0.0) + vec3<f32>(1e-6, 0.0, 0.0));
    let towards = pow(max(dot(flat_d, flat_sun), 0.0), 3.0);
    let low = pow(1.0 - clamp(abs(d.z), 0.0, 1.0), 6.0);
    colour = mix(colour, DUSK * max(camera.daylight.z, 0.15), dusk * towards * low * 0.6);
    return colour;
}

fn sky(d: vec3<f32>) -> vec3<f32> {
    let up = clamp(d.z, -0.2, 1.0);
    return sky_tone(mix(HORIZON, ZENITH, pow(max(up, 0.0), 0.6)), d);
}

// The air between the eye and what it sees: thickest at sea level, halving every so many blocks up, worked out whole
// along the ray (no steps). Land fades to the sky's horizon colour; a little warmer looking towards the sun.
const FOG_WARM: vec3<f32> = vec3<f32>(1.0, 0.82, 0.62);

fn fogged(colour: vec3<f32>, d: vec3<f32>, t: f32) -> vec3<f32> {
    if (camera.fog.x <= 0.0) {
        return colour;
    }
    let thins = camera.fog.y;
    let at_eye = camera.fog.x * exp(clamp(-thins * (camera.position.z - camera.fog.z), -80.0, 80.0));
    let climb = thins * d.z;
    var along = t;
    if (abs(climb * t) > 1e-4) {
        along = (1.0 - exp(-clamp(climb * t, -80.0, 80.0))) / climb;
    }
    let amount = 1.0 - exp(-at_eye * along);
    let sunward = pow(max(dot(d, camera.sun.xyz), 0.0), 8.0) * camera.fog.w * camera.daylight.x;
    return mix(colour, sky_tone(mix(HORIZON, FOG_WARM, sunward), d), amount);
}

// ── Light (old's 3.4a) ──

// The level a ray reads at distance `t`: its pixel's size there, in whole powers of two, never finer than the finest
// level there is.
fn level_at(t: f32) -> i32 {
    let footprint = max(t * camera.view.z, 1.0);
    let level = i32(floor(log2(footprint))) + i32(camera.reach.y);
    return clamp(level, FIRST, TOP);
}

// The light in two parts: the sky's, always there (`camera.sun.w` of a face's colour, by day), and the sun's, which a
// shadow takes away (2b). `lit` is both; `sunlit` the sun's alone.
fn lit(colour: vec3<f32>, light: f32) -> vec3<f32> {
    return colour * (camera.sun.w * camera.daylight.y + (1.0 - camera.sun.w) * camera.daylight.x * light);
}

fn sunlit(colour: vec3<f32>, light: f32) -> vec3<f32> {
    return colour * (1.0 - camera.sun.w) * camera.daylight.x * light;
}

// How much sun a cell's faces get, seen along `d`: each way's faces lit as themselves, and weighted by how much of
// that way the eye sees. A cell whose faces all look away (the eye inside it) is lit by all of them.
fn faces_light(look: Look, d: vec3<f32>) -> f32 {
    let sun = camera.sun.xyz;
    let seen_plus = look.plus * max(-d, vec3<f32>(0.0));
    let seen_minus = look.minus * max(d, vec3<f32>(0.0));
    let seen = dot(seen_plus + seen_minus, vec3<f32>(1.0));
    let sunny_plus = max(sun, vec3<f32>(0.0));
    let sunny_minus = max(-sun, vec3<f32>(0.0));
    var light = 0.0;
    if (seen > 1e-4) {
        light = (dot(seen_plus, sunny_plus) + dot(seen_minus, sunny_minus)) / seen;
    } else {
        light = dot(look.plus, sunny_plus) + dot(look.minus, sunny_minus);
    }
    return light;
}

// The highest anything stands in what's on the card so far: over it, a ray that isn't coming down meets nothing more.
fn world_top() -> f32 {
    return min(max(camera.reach.w, 0.0), WORLD_HEIGHT);
}

// How far along the ray (in x and y only) it leaves the cell column `c` of `size`.
fn leave2(o: vec2<f32>, d: vec2<f32>, c: vec2<f32>, size: f32) -> f32 {
    let next = (c + select(vec2<f32>(0.0), vec2<f32>(1.0), d > vec2<f32>(0.0))) * size;
    let t = (next - o) / d;
    let big = 1e30;
    let tx = select(big, t.x, abs(d.x) > 1e-6);
    let ty = select(big, t.y, abs(d.y) > 1e-6);
    return min(tx, ty);
}

// A number from 0 to 1 for a pixel, steady from frame to frame (a hash of its place).
fn scatter(at: vec2<u32>) -> f32 {
    var h = at.x * 0x9E3779B1u ^ (at.y * 0x85EBCA77u);
    h = (h ^ (h >> 15u)) * 0x2C1B3C6Du;
    h = h ^ (h >> 12u);
    return f32(h & 0xFFFFu) / 65536.0;
}

// ── The march ──

struct Seen {
    colour: vec3<f32>,
    distance: f32,
    steps: f32,
    level: i32,
    // For its shadow (2b): the sun's part of `colour`, the place, and how wide the shadow ray starts.
    sunny: vec3<f32>,
    at: vec3<f32>,
    wide: f32,
    // The sky's part of `colour` (the light that isn't the sun's: what sky light scales).
    skyish: vec3<f32>,
    // Where it first touched anything.
    touched: f32,
    // How much of it went on to the sky (the sky behind isn't fogged).
    sky_share: f32,
}

// A ray from `o` along `d`, from `begin`. `phase`: a pixel's own number from 0 to 1 (`scatter`), where its fine steps
// fall within a fine step, so their places against the ground don't line up across the screen.
fn march(o: vec3<f32>, d: vec3<f32>, phase: f32, begin: f32) -> Seen {
    var colour = vec3<f32>(0.0);
    var sunny = vec3<f32>(0.0);
    var through = 1.0;
    var first = camera.reach.x;
    var touched = camera.reach.x;
    var steps = 0;
    var t = max(begin, 0.5) + phase;
    // Whether the last step found only air (only then is skipping tried).
    var in_air = true;
    // The last place a step found only air, and whether the ray is walking into a surface in fine steps from there (so
    // what's gathered never depends on where a stride fell: old's rings on flat ground, 2026-11-06).
    var last_air = t;
    var walking = false;
    var walk_to = 0.0;
    var first_level = -1;
    // The last sample, for gathering each step from both its ends (trapezoids).
    var before: Look;
    var before_t = 0.0;
    var chained = false;
    let ceiling = world_top() + 32.0;
    // By a surface, steps sized to how fast the ray crosses it upward or downward.
    let surface_step = 0.25 / clamp(abs(d.z), 0.25, 1.0);
    let water_colour = camera.water.rgb;
    // Whether it went down through the world's floor, and what it last gathered in the cells over the floor.
    var floored = false;
    var bottom = false;
    var bottom_here = vec3<f32>(0.0);
    var bottom_sun = vec3<f32>(0.0);
    var bottom_t = 0.0;
    var bottom_level = -1;
    for (var n = 0; n < 768; n++) {
        if (t > camera.reach.x || through < 0.01) {
            break;
        }
        steps = n + 1;
        let p = o + d * t;
        if (p.z > ceiling && d.z >= 0.0) {
            break;
        }
        if (p.z < -8.0 && d.z <= 0.0) {
            floored = true;
            break;
        }
        var level = level_at(t);
        // A coarse level not built over here yet (it waits for all its regions): the regions' own level. A region not
        // summed yet holds nothing to draw: skipped whole.
        if (level > REGION) {
            let c = floor(p.xy / f32(1 << u32(level)));
            if (entry_at(level, i32(c.x), i32(c.y)) == NOT_SUMMED) {
                level = REGION;
            }
        }
        if (level <= REGION) {
            let r = floor(p.xy / f32(1 << u32(REGION)));
            if (entry_at(REGION, i32(r.x), i32(r.y)) == NOT_SUMMED) {
                t = max(leave2(o.xy, d.xy, r, f32(1 << u32(REGION))), t) + 0.01;
                last_air = t;
                walking = false;
                chained = false;
                continue;
            }
        }
        // Space a coarser cell says is empty is skipped whole: over a coarse cell column's band's top, the ray goes on
        // to where it leaves the column or comes down to the top, whichever is sooner. The column two levels coarser,
        // then its own.
        var skipped = false;
        let fine = f32(1 << u32(level));
        for (var k = 0; k < 2 && in_air && !walking; k++) {
            var up = level;
            if (k == 0) {
                up = min(level + 2, TOP);
            }
            let up_size = f32(1 << u32(up));
            let cc = floor(p.xy / up_size);
            let entry = entry_at(up, i32(cc.x), i32(cc.y));
            if (entry == NOT_SUMMED) {
                continue;
            }
            let low = bitcast<i32>(entry << 16u) >> 16u;
            let count = i32(entry >> 16u);
            let top = f32(low + count) * up_size + fine;
            if (p.z <= top) {
                continue;
            }
            var next = leave2(o.xy, d.xy, cc, up_size);
            if (d.z < 0.0) {
                next = min(next, t + (top - p.z) / d.z);
            }
            if (next > t + 0.01) {
                t = next + 0.01;
                last_air = t;
                walking = false;
                chained = false;
                skipped = true;
                break;
            }
        }
        if (skipped) {
            continue;
        }
        var look = far_blend(level, p);
        // Over the last stretch before the next level takes over, the two blended, so the change is gradual, never a
        // seam (seams drew rings round the eye over flat water that moved as it moved: David, build 69).
        let over = log2(max(t * camera.view.z, 1.0)) + camera.reach.y - f32(level);
        if (look.known && over > camera.fade.z && level < TOP) {
            let next = far_blend(level + 1, p);
            if (next.known) {
                look = mix_looks(look, next, (over - camera.fade.z) / (1.0 - camera.fade.z));
            }
        }
        let size = f32(1 << u32(level));
        let thick = select(0.0, max(look.solid, look.water), look.known);
        // The first touch of a surface after a stride through air: back to where the air was, and into the surface
        // from there in fine steps.
        if (thick > 0.0 && !walking && t - last_air > size * 0.26) {
            walk_to = t;
            t = last_air + phase * size * 0.25;
            walking = true;
            in_air = false;
            chained = false;
            continue;
        }
        // Air within a cell all round: two cells on. By a surface, a quarter of a cell at a time.
        var dt = size * 2.0;
        in_air = true;
        if (thick > 0.0) {
            in_air = false;
            walking = true;
            dt = select(size * 0.5, size * surface_step, thick > 0.02);
        } else if (walking && t < walk_to) {
            in_air = false;
            dt = size * surface_step;
        } else {
            last_air = t;
            walking = false;
            // Going down, never past the world's floor (under it all is rock).
            if (d.z < 0.0) {
                dt = min(dt, max(p.z / -d.z, size * 0.25));
            }
        }
        // The step from the last sample to this one, gathered from both its ends.
        if (look.known && chained && before.known) {
            let length = t - before_t;
            let solid_share = 0.5 * (before.solid + look.solid);
            let water_share = 0.5 * (before.water + look.water);
            let solid = 1.0 - pow(1.0 - min(solid_share, 0.999), length / size);
            let water = 1.0 - exp(-camera.water.w * water_share * length);
            let covered = 1.0 - (1.0 - solid) * (1.0 - water);
            if (covered > 0.0) {
                touched = min(touched, before_t);
                let light_before = faces_light(before, d);
                let light_now = faces_light(look, d);
                let solids = max(before.solid + look.solid, 1e-6);
                let ground = (lit(before.colour, light_before) * before.solid + lit(look.colour, light_now) * look.solid) / solids;
                let ground_sun = (sunlit(before.colour, light_before) * before.solid + sunlit(look.colour, light_now) * look.solid) / solids;
                let both = max(solid + water, 1e-6);
                let here = (ground * solid + lit(water_colour, 1.0) * water) / both;
                let here_sun = (ground_sun * solid + sunlit(water_colour, 1.0) * water) / both;
                colour += through * covered * here;
                sunny += through * covered * here_sun;
                if (p.z <= size * 1.5) {
                    bottom = true;
                    bottom_here = here;
                    bottom_sun = here_sun;
                    bottom_t = t;
                    bottom_level = level;
                }
                if (through > 0.5 && through * (1.0 - covered) <= 0.5) {
                    first = t;
                    first_level = level;
                }
                through *= 1.0 - covered;
            }
        }
        before = look;
        before_t = t;
        chained = look.known;
        t += dt;
    }
    // Down through the world's floor having gathered from the cells just over it: what it didn't gather is that ground
    // too, not the sky (a far cell is an average). Only the ground over the floor: over nothing, it stays sky.
    if (floored && bottom) {
        colour += through * bottom_here;
        sunny += through * bottom_sun;
        through = 0.0;
        if (first >= camera.reach.x) {
            first = bottom_t;
            first_level = bottom_level;
        }
    }
    let gathered = colour;
    colour += through * sky(d);
    // The shadow ray as wide as the cells the surface was read from (or the eye's pixel there, if wider). None: nothing
    // met.
    var wide = 0.0;
    if (first < camera.reach.x) {
        let read = select(1.0, f32(1 << u32(max(first_level, 0))), first_level >= 0);
        wide = max(first * camera.view.z, read);
    }
    return Seen(colour, first, f32(steps), first_level, sunny, o + d * first, wide, gathered - sunny, touched, through);
}


// ── Light (old's 3.4b): the sun's shadow and the sky's light, through the same far cells ──
//
// How open the way `s` is from a far surface at `p`, 0 to 1 (old's `cone`, the far cells' part): a ray widening `spread`
// a block of distance, reading the level as wide as it is. Clear of the surface's own soft edge first: up half a cell
// and along `s` till a cell higher. Air over a column's top is skipped whole (the ray only climbs), regions not summed
// are passed whole, and once it's over the highest anything stands nothing more can shade it.
fn cone(p: vec3<f32>, footprint: f32, s: vec3<f32>, spread: f32, most: i32) -> f32 {
    let wanted = clamp(i32(floor(log2(max(footprint, 1.0)))), FIRST, TOP);
    let start = max(exp2(floor(log2(max(footprint, 1.0)))), f32(1 << u32(wanted)));
    let o = p + vec3<f32>(0.0, 0.0, start * 0.5);
    var t = start / max(s.z, 0.1);
    var light = 1.0;
    let top = world_top() + 1.0;
    for (var n = 0; n < most; n++) {
        let q = o + s * t;
        if (q.z > top || t > camera.reach.x) {
            break;
        }
        let wide = max(footprint, t * spread);
        let level = clamp(i32(floor(log2(max(wide, 1.0)))), FIRST, TOP);
        // Over a column's top: on to where it leaves that column, the biggest it's over the top of, tried from the
        // coarsest level down to two coarser than it reads.
        var skipped = false;
        for (var up = TOP; up >= level + 2; up -= 2) {
            let up_size = f32(1 << u32(up));
            let cc = floor(q.xy / up_size);
            let entry = entry_at(up, i32(cc.x), i32(cc.y));
            if (entry == PAST_MAP) {
                return light;
            }
            if (entry == NOT_SUMMED) {
                continue;
            }
            let low = bitcast<i32>(entry << 16u) >> 16u;
            let count = i32(entry >> 16u);
            if (q.z > f32(low + count) * up_size + f32(1 << u32(level))) {
                t = max(leave2(o.xy, s.xy, cc, up_size), t) + 0.01;
                skipped = true;
                break;
            }
        }
        if (skipped) {
            continue;
        }
        // A region not summed yet holds nothing known to shade with: on past it whole.
        let r = floor(q.xy / f32(1 << u32(REGION)));
        if (entry_at(REGION, i32(r.x), i32(r.y)) == NOT_SUMMED) {
            t = max(leave2(o.xy, s.xy, r, f32(1 << u32(REGION))), t) + 0.01;
            continue;
        }
        let look = far_blend(level, q);
        let size = f32(1 << u32(level));
        // A step as long as the ray is wide (never under its cell): what it crosses dims it by how solid it is, once
        // it's climbed a whole cell of what it reads over where it started.
        let step = max(size, wide);
        if (look.known && q.z - p.z >= size) {
            light *= pow(1.0 - min(look.solid, 0.999), step / size) * exp(-camera.water.w * look.water * step);
            if (light < 0.02) {
                return 0.0;
            }
        }
        t += step;
    }
    return light;
}

// The most steps a shadow ray takes (past them it's taken as sunlit), and a sky cone.
const SHADOW_STEPS: i32 = 160;
const SKY_STEPS: i32 = 40;

// How much of the sun reaches a far surface at `p` (`footprint` the eye's pixel there, blocks).
fn sunlight(p: vec3<f32>, footprint: f32) -> f32 {
    return cone(p, footprint, camera.sun.xyz, camera.follow.z, SHADOW_STEPS);
}

// How much of the sky a far surface at `p` sees: five wide cones, straight up and four leaning 50° out (north, south,
// east, west), each weighted by how high it looks. In one loop, its count read from the levels so the compiler can't
// unroll it into five copies (old's lesson: the instruction cache).
fn sky_seen(p: vec3<f32>, footprint: f32) -> f32 {
    let lean = sin(radians(50.0));
    let rise = cos(radians(50.0));
    let cones = 5 + min(levels[0] - FIRST, 0);
    var seen = 0.0;
    for (var k = 0; k < cones; k++) {
        var way = vec3<f32>(0.0, 0.0, 1.0);
        var weight = 1.0;
        if (k > 0) {
            let s = select(-lean, lean, (k & 1) == 1);
            way = select(vec3<f32>(0.0, s, rise), vec3<f32>(s, 0.0, rise), k <= 2);
            weight = rise;
        }
        seen += weight * cone(p, footprint, way, camera.sky.w, SKY_STEPS);
    }
    return seen / (1.0 + 4.0 * rise);
}

// A pixel's sky light as handed down to the layer below: its kind (the far cells' is 0) over its sky seen in
// 65,535ths.
fn sky_word(open: f32) -> u32 {
    return u32(round(clamp(open, 0.0, 1.0) * 65535.0));
}

// A ray's colour with its shadow and its sky light (`open`: how much of the sky it sees, 0 to 1).
fn shaded(seen: Seen, open: f32) -> vec3<f32> {
    if (seen.wide <= 0.0) {
        return seen.colour;
    }
    var colour = seen.colour;
    if (camera.follow.y > 0.5) {
        colour -= seen.sunny * (1.0 - sunlight(seen.at, seen.wide));
    }
    let sky = select(1.0, mix(1.0, open, camera.sky.y), camera.sky.x > 0.5);
    let tint = mix(vec3<f32>(1.0), SKY_TINT, camera.sky.z);
    return colour + seen.skyish * (sky * tint - 1.0);
}

// ── The start pass (the screen pyramid's first step) ──
//
// One ray down the middle of each 8 × 8 tile of the screen, taken as a cone that holds the whole tile: it only skips
// (over cell columns' tops, the cone's width kept clear of them, and a margin for the blends), never blends, and stops
// at the first place the cone could touch anything. Every ray of the tile starts there.

fn cone_start(o: vec3<f32>, d: vec3<f32>, spread: f32) -> f32 {
    var t = 0.5;
    let margin = 32.0;
    let across = length(d.xy);
    let ceiling = world_top() + margin;
    for (var n = 0; n < 96; n++) {
        if (t >= camera.reach.x) {
            return camera.reach.x;
        }
        let p = o + d * t;
        if (p.z - t * spread > ceiling && d.z >= 0.0) {
            return camera.reach.x;
        }
        let r = t * spread + 2.0;
        // A region not summed yet: from here (the march skips it itself).
        let rg = floor(p.xy / f32(1 << u32(REGION)));
        if (entry_at(REGION, i32(rg.x), i32(rg.y)) == NOT_SUMMED) {
            return t;
        }
        var advanced = false;
        let first_level = clamp(i32(ceil(log2(4.0 * r))), FIRST, TOP);
        for (var k = 0; k < 3; k++) {
            let level = min(first_level + k, TOP);
            let size = f32(1 << u32(level));
            let c = floor(p.xy / size);
            // The cone has to sit inside the column.
            if (any(p.xy - r < c * size) || any(p.xy + r >= (c + 1.0) * size)) {
                continue;
            }
            let entry = entry_at(level, i32(c.x), i32(c.y));
            if (entry == NOT_SUMMED) {
                continue;
            }
            var top = -1e9;
            if (entry != PAST_MAP) {
                let low = bitcast<i32>(entry << 16u) >> 16u;
                let count = i32(entry >> 16u);
                top = f32(low + count) * size;
            }
            if (p.z - r <= top + margin) {
                break;
            }
            // On till the cone's edge reaches the column's side, or its bottom comes down to the top.
            let leaving = leave2(o.xy, d.xy, c, size);
            var next = (leaving * across - 2.0) / max(across + spread, 1e-6);
            if (d.z - spread < 0.0) {
                next = min(next, (top + margin + 2.0 - p.z + d.z * t) / (d.z - spread));
            }
            if (next > t + 1.0) {
                t = next;
                advanced = true;
                break;
            }
        }
        if (!advanced) {
            return t;
        }
    }
    return t;
}

@compute @workgroup_size(8, 8)
fn start_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let tiles = vec2<u32>(camera.tiles.xy);
    if (id.x >= tiles.x || id.y >= tiles.y) {
        return;
    }
    let uv = min((vec2<f32>(id.xy) + 0.5) * camera.tiles.z / camera.view.xy, vec2<f32>(1.0));
    let d = normalize(camera.forward.xyz + (uv.x * 2.0 - 1.0) * camera.right.xyz + (1.0 - uv.y * 2.0) * camera.up.xyz);
    starts[id.y * tiles.x + id.x] = cone_start(camera.position.xyz, d, camera.tiles.w);
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3<f32>(0.0031308));
}

// How much the layer below is worth where this layer's surface is `near` off, read from cells of `level` (0 to 1).
// Nothing while its pixels would be smaller than the cells they'd read (finer than the cells, they only magnify the
// blend into smudge: David's bands, "2 looks worse than 4, up close"); rising to all of it as they grow bigger; never
// for sky; nothing if there's no layer below.
fn worth_of(near: f32, level: i32) -> f32 {
    if (camera.layer.y < 0.5) {
        return 0.0;
    }
    if (camera.layer.z < 0.5) {
        return 1.0;
    }
    if (near >= camera.reach.x || level < 0) {
        return 0.0;
    }
    let cell = f32(1 << u32(level));
    let ratio = near * camera.view.z * camera.pyramid.y / cell;
    return clamp((ratio / camera.fade.x - 1.0) / max(exp2(camera.fade.y) - 1.0, 1e-3), 0.0, 1.0);
}

// One layer of the pyramid: a ray a pixel, where the layer above said it's worth it.
@compute @workgroup_size(8, 8)
fn march_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = vec2<u32>(camera.view.xy);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    var shown = 1.0;
    let parent = min(vec2<u32>((vec2<f32>(id.xy) + 0.5) * camera.pyramid.x), vec2<u32>(u32(max(camera.layer.x, 1.0)) - 1u, 65535u));
    if (camera.layer.x > 0.0) {
        let worth = worth_in[parent.y * u32(camera.layer.x) + parent.x] & 0xFFu;
        shown = f32(worth) / 255.0;
        if (worth == 0u) {
            textureStore(picture, vec2<i32>(id.xy), vec4<f32>(0.0));
            textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(camera.reach.x, 0.0, 0.0, 0.0));
            if (camera.layer.y > 0.5) {
                worth_out[id.y * size.x + id.x] = NO_SKY << 8u;
            }
            return;
        }
    }
    // Where its tile's rays start (from the start pass), or from the eye.
    var begin = 0.5;
    if (camera.tiles.z > 0.0) {
        let tile = vec2<u32>(vec2<f32>(id.xy) / camera.tiles.z);
        begin = starts[tile.y * u32(camera.tiles.x) + tile.x];
    }
    // A finer layer's rays start a little short of the nearest touch among their parent and its neighbours.
    if (camera.layer.x > 0.0 && camera.follow.x > 0.5) {
        let above = vec2<i32>(parent);
        let edge = vec2<i32>(textureDimensions(far_above)) - 1;
        var nearest = camera.reach.x;
        for (var k = 0; k < 9; k++) {
            let at = clamp(above + vec2<i32>(k % 3 - 1, k / 3 - 1), vec2<i32>(0), edge);
            nearest = min(nearest, textureLoad(far_above, at, 0).x);
        }
        begin = max(begin, nearest - 2.0 - nearest * camera.view.z * 4.0);
    }
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
    let d = normalize(camera.forward.xyz + (uv.x * 2.0 - 1.0) * camera.right.xyz + (1.0 - uv.y * 2.0) * camera.up.xyz);
    let seen = march(camera.position.xyz, d, scatter(id.xy * 2u), begin);
    // Its shadow and sky light only where it shows (a finer layer laid over it whole hides it).
    let hidden = round(worth_of(seen.distance, seen.level) * 255.0) >= 255.0;
    // Its sky light, where it shows or the layer below may take it: its parent's, if that was about as far off (sky
    // light changes slowly: most finer pixels take it, and only edges and silhouettes look for themselves).
    var open = 1.0;
    var handed = NO_SKY;
    if (camera.sky.x > 0.5 && seen.wide > 0.0 && (!hidden || camera.layer.y > 0.5)) {
        var have = false;
        if (camera.layer.x > 0.0) {
            let above = worth_in[parent.y * u32(camera.layer.x) + parent.x] >> 8u;
            let parent_touch = textureLoad(far_above, vec2<i32>(parent), 0).x;
            if (above != NO_SKY && (above >> 16u) == 0u && abs(seen.touched - parent_touch) <= 0.1 * seen.touched + 2.0) {
                open = f32(above & 0xFFFFu) / 65535.0;
                have = true;
            }
        }
        if (!have) {
            open = sky_seen(seen.at, seen.wide);
        }
        handed = sky_word(open);
    }
    var here = seen.colour;
    if (!hidden) {
        here = shaded(seen, open);
    }
    // Through the fog to where it first met anything; the sky it went on to past a soft edge stays the sky. The sky's
    // weight [David, 2026-11-12: "if we could just make sky a bit stronger in the math over land, i think that might
    // give a basic natural feather at an early stage that will make a clean chamfer"]: the land's share of a pixel it
    // only partly covers is taken to a power, so a skyline's thinly covered squares lean to the sky. The power fades as
    // the land covers more (1 + the weight × the sky's share squared): a far mountain's own pixels, which its soft cells
    // never quite cover, are left nearly as they were [build 123: a plain power washed the mountains out, David: "love
    // what the weight does to the edges, hate what it does to the mountain *not* on the edges"].
    let met = select(seen.distance, seen.touched, seen.distance >= camera.reach.x);
    if (met < camera.reach.x) {
        let behind = seen.sky_share * sky(d);
        let covered = max(1.0 - seen.sky_share, 1e-4);
        let land = fogged((here - behind) / covered, d, met);
        let shown = pow(covered, 1.0 + camera.fade.w * seen.sky_share * seen.sky_share);
        here = sky(d) * (1.0 - shown) + land * shown;
    }
    if (camera.layer.y > 0.5) {
        worth_out[id.y * size.x + id.x] = u32(round(worth_of(seen.distance, seen.level) * 255.0)) | (handed << 8u);
    }
    var colour = here;
    if (camera.reach.z > 1.5 && seen.level >= 0) {
        // The levels' view: each pixel tinted by the level its surface was read from.
        let tints = array<vec3<f32>, 6>(vec3<f32>(1.0, 0.3, 0.3), vec3<f32>(0.3, 1.0, 0.3), vec3<f32>(0.3, 0.4, 1.0), vec3<f32>(1.0, 1.0, 0.3), vec3<f32>(1.0, 0.3, 1.0), vec3<f32>(0.3, 1.0, 1.0));
        colour = colour * 0.4 + tints[seen.level % 6] * 0.35;
    } else if (camera.reach.z > 0.5 && camera.reach.z < 1.5) {
        // The steps' view: dark blue few, through green, to red 256 or more.
        let k = clamp(seen.steps / 256.0, 0.0, 1.0);
        colour = select(mix(vec3<f32>(0.0, 0.8, 0.2), vec3<f32>(1.0, 0.0, 0.0), (k - 0.5) * 2.0), mix(vec3<f32>(0.0, 0.0, 0.4), vec3<f32>(0.0, 0.8, 0.2), k * 2.0), k < 0.5);
        colour = colour * colour;
    }
    // Kept with its alpha (premultiplied), so the screen can blend it over the layer above.
    textureStore(picture, vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour, vec3<f32>(0.0), vec3<f32>(1.0))) * shown, shown));
    // Kept for the layer below to start its rays from: where it first touched anything.
    textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(seen.touched, 0.0, 0.0, 0.0));
}

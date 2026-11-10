// ── Sight's march (the regroup, phase 3.2): the coarse layer, a ray (or a few) a coarse pixel through the store ──
//
// The store (`card_store.rs`): the live level round the camera (cells of 16), the baked levels from 32 up
// over the whole map. A ray reads the level whose cells are about its pixel's size at that distance (never
// finer than the live level, which is the finest there is in 3.2), skips space a coarser cell says is empty,
// and elsewhere takes half a cell at a time, reading a smooth blend of the eight cells round it (no threshold:
// a part-solid cell covers part of the pixel, gathered front to back; colour and facing weighted by how much
// surface each cell shows), lit as its faces are (3.4a: each of the six ways a block's faces look, lit by the sun
// as itself, weighted by how much of that way the eye sees). The sky behind.

struct Camera {
    // The eye (game axes: x north, y east, z up).
    position: vec4<f32>,
    forward: vec4<f32>,
    // Scaled so a pixel's direction is forward + x·right + y·up for x, y from −1 to 1.
    right: vec4<f32>,
    up: vec4<f32>,
    // x, y: the coarse layer's size; z: a coarse pixel's angle (radians); w: rays a coarse pixel.
    view: vec4<f32>,
    // xyz: towards the sun; w: the ambient light.
    sun: vec4<f32>,
    // x: the farthest a ray goes (blocks); y: the level bias; w: the highest anything stands (blocks); z: 1 to
    // show each ray's steps instead (a
    // measuring view: dark blue few, through green and yellow, to red 256 or more), 2 to tint each pixel by
    // the level its surface was read from (a measuring view: where the levels change).
    reach: vec4<f32>,
    // The start pass (the screen in tiles of 8 × 8 pixels): x, y the tiles across and down; z the coarse pixels a
    // tile side (0: no start pass, every ray from the eye); w the cone's spread (radians from a tile's middle
    // ray to its corners).
    tiles: vec4<f32>,
    // The screen pyramid (David 2026-11-06: "1 - 2 - 4 - 8"): x the layer above's width in its pixels (0: this is
    // the 8 × 8 base, everything marched); y 1 if a finer layer follows (this layer says where it's worth it); z
    // the rule (0 refine everywhere, 1 only where it's earned); w how far the finest live level reaches (blocks;
    // each coarser one reaches twice as far for each level coarser).
    layer: vec4<f32>,
    // x: 1 if a refining layer's rays start a little short of the nearest touch round their parent pixel (0: from
    // the tile's start, for measuring). y: 1 for sun shadows (3.4b). z: how fast a shadow ray widens (blocks
    // across a block of distance). w: 2 for the near walk to skip no air, every block walked (the truth, for the
    // tests: the dyno's first step, 2026-11-09).
    follow: vec4<f32>,
    // The pyramid's steps (David 2026-11-08, 1.5 between 2 and 1, and the fades his to tune): x this layer's factor
    // over the one above's (0.5 halving, 0.75 for 1.5 under 2; 0 the base); y the one below's over this one's; z,
    // w where the one below fades out among the blocks (shares of their reach, from and to).
    pyramid: vec4<f32>,
    // Past the blocks, the one below's fade in: x where it starts (its pixel so many times its cells), y over how
    // many doublings of distance. z: where in a level's stretch its blend with the next begins (the hand-over).
    fade: vec4<f32>,
    // Sky light (3.4b): x 1 for it on, y its strength (how dark a place seeing no sky goes), z the tint (0 white, 1
    // a sky blue), w how wide its cones are (blocks across a block of distance).
    sky: vec4<f32>,
    // Fog (3.4c): x its thickness at sea level (a block's worth; 0 none), y how fast it thins going up (a block's
    // worth), z sea level (blocks), w how much warmer it is towards the sun.
    fog: vec4<f32>,
    // Block textures (3.6): x 1 for them on, y how sharp (a shift in the shrunk versions taken).
    textures: vec4<f32>,
    // The time of day (David 2026-11-09, darkness before lights): x the sun's own light (0 to 1: gone with it
    // under the horizon), y the sky's light (night's at the least), z the sky's own brightness (0 night, 1 day),
    // w night's light.
    daylight: vec4<f32>,
    // Placed lights (build 95): x 1 for them on, y how much they flicker, z the time they flicker by (seconds); w 1 to
    // blend them across a face (0: a block's step at a time, hard-edged, as decided).
    glow: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<storage, read> header: array<i32>;
@group(0) @binding(2) var<storage, read> baked: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> live: array<vec4<u32>>;
@group(0) @binding(4) var<storage, read> pool: array<vec4<u32>>;
@group(0) @binding(5) var picture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var how_far: texture_storage_2d<r32float, write>;
// Where each tile's rays start (the start pass's answers).
@group(0) @binding(7) var<storage, read_write> starts: array<f32>;
// The pyramid: how much each of the layer above's pixels says its children are worth (255ths: 0 not marched, 255
// shown whole) in its low 8 bits, its sky light handed down over them (`sky_word`, 3.4b; in the same word, as a
// shader may have only 8 storage buffers), and this layer's word for the layer below.
@group(0) @binding(8) var<storage, read> worth_in: array<u32>;
@group(0) @binding(9) var<storage, read_write> worth_out: array<u32>;
// The layer above's distances (the base: a placeholder never read).
@group(0) @binding(10) var far_above: texture_2d<f32>;
// The block textures (3.6: each with its shrunk versions, stored as painted, read as light), and each block's faces'
// textures: two rows a block (north, south, east, west; top, bottom, which turn at random, 0).
@group(0) @binding(11) var block_textures: texture_2d_array<f32>;
@group(0) @binding(12) var block_sampler: sampler;
@group(0) @binding(13) var<uniform> block_table: array<vec4<u32>, 512>;
// The cells built of pieces (build 90, `CardStore::put_pieces`): two words each, its place (x, y, z, 1 if used),
// then its eight octants' blocks, two to a word (0 an empty octant).
@group(0) @binding(14) var<storage, read> pieces: array<vec4<u32>>;
// The lit cells of air (build 95, `CardStore::put_lights`): each its place, and its light (red, green, blue 4 bits
// each, the brightest light's beat 8 bits over them, the top bit set: a place in use).
@group(0) @binding(15) var<storage, read> lights: array<vec4<u32>>;
// Each block's medium (build 98: water, glass, ice; `BlockTextures::media_table`): its colour as light, and how fast
// it dims what's seen through it (a block's worth; 0 not a medium). The first is the world's water's.
@group(0) @binding(16) var<uniform> media: array<vec4<f32>, 256>;
// Far lights' glows (build 100, `glow_pass`): each one's place in (xyz; w 0, or −1 for none), and whether the eye
// sees it out (w: 1 seen, 0 hidden). Read straight after as the glows' places (`models.rs`).
@group(0) @binding(17) var<storage, read_write> glows: array<vec4<f32>>;

// The medium of block `block`: its own, or the world's water's (water the cells don't keep the block of).
fn medium_of(block: u32) -> vec4<f32> {
    if (block > 0u && block < 256u && media[block].w > 0.0) {
        return media[block];
    }
    return media[0];
}

const NOT_BAKED: u32 = 0xFFFF0000u;
// A cell of one block that's built of pieces (`BlockId::PIECES`, kept as its block by `Cell::with_block`).
const PIECES_MARK: u32 = 0xFFFFu;
const PIECES_SLOTS: u32 = 16384u;
const PIECES_PROBES: i32 = 32;
const LIGHT_SLOTS: u32 = 65536u;
const LIGHT_PROBES: i32 = 32;
// Past the map: nothing at all, not even the world's floor.
const PAST_MAP: u32 = 0xFFFE0000u;
const WORLD_HEIGHT: f32 = 1024.0;
// The bake's regions are its cell columns of 1,024 (`bake::REGION`).
const REGION: i32 = 10;

// Rock's colour, where a cell shows no surface (inside). (Water's is its medium's since build 98: `media`.)
const ROCK: vec3<f32> = vec3<f32>(0.25, 0.24, 0.22);

// A cell as the march reads it.
struct Look {
    known: bool,
    solid: f32,
    water: f32,
    colour: vec3<f32>,
    // Its faces' shares by the way they look (0 to 1, adding to 1): `plus` looking north, east and up, `minus`
    // south, west and down.
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
    // The six ways' shares (`Cell::faces`, bytes 2 to 6: up, down, north, south, east; west what's left).
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

// Which block a cell of one block is (3.6: `Cell::with_block`, kept where a bigger cell keeps its heights' spread);
// 0 outside the band. Read once a ray, after the march (`wear_texture`), not carried in `Look`.
fn block_in_band(entry_start: u32, low_count: u32, cz: i32) -> u32 {
    let low = bitcast<i32>(low_count << 16u) >> 16u;
    let count = i32(low_count >> 16u);
    if (low_count == NOT_BAKED || low_count == PAST_MAP || cz < low || cz >= low + count) {
        return 0u;
    }
    return pool[entry_start + u32(cz - low)].w & 0xFFFFu;
}

// A band's entry read: its cell `cz` up, or solid below the band, or empty above it.
fn from_band(entry_start: u32, low_count: u32, cz: i32) -> Look {
    if (low_count == NOT_BAKED) {
        return empty_look(false);
    }
    if (low_count == PAST_MAP) {
        return empty_look(true);
    }
    let low = bitcast<i32>(low_count << 16u) >> 16u;
    let count = i32(low_count >> 16u);
    if (cz < 0 || (cz < low && low > 0)) {
        return solid_look();
    }
    if (cz < low || cz >= low + count) {
        return empty_look(true);
    }
    return cell_of(pool[entry_start + u32(cz - low)]);
}

// Whether the live level `level` is kept (`header[5]`, a bit each).
fn is_kept(level: i32) -> bool {
    return ((u32(header[5]) >> u32(level)) & 1u) == 1u;
}

// The level read for `level`: it, or the next coarser one kept (a live level between the kept ones isn't).
fn kept(level: i32) -> i32 {
    var l = level;
    for (var k = 0; k < 8 && l < header[0] && !is_kept(l); k++) {
        l = l + 1;
    }
    return l;
}

// Where the live level `level`'s column (`cx`, `cy`) is in the live index: its window (each `header[4]` squared
// on from the last), then its place in the wrapping window.
fn live_slot(level: i32, cx: i32, cy: i32) -> vec4<u32> {
    let side = header[3];
    let window = u32(level - header[2]) * u32(header[4] * header[4]);
    return live[window + u32(wrapped(cy, side) * side + wrapped(cx, side))];
}

// `c` wrapped into 0 to `side` (as `((c % side) + side) % side`, the same for every `c`), by a float floor, then a
// fix-up either way for the quotient's rounding: no whole-number division, which the card has no instruction for, a
// long chain of steps each [2026-11-10, the outside read's lead 6: four of them for every column read at a live level,
// sixteen a blend; without them the card's sight 11.9% faster on the median, the picture the same to the pixel].
// Exact while `c` and `side` are whole numbers a float holds (under 16 million).
fn wrapped(c: i32, side: i32) -> i32 {
    var m = c - side * i32(floor(f32(c) / f32(side)));
    m = select(m, m - side, m >= side);
    m = select(m, m + side, m < 0);
    return m;
}

// The cell at `c` (cell indices) of `level`: from the live windows, or the baked levels; `known` false if it
// isn't there yet.
fn cell_at(level: i32, c: vec3<i32>) -> Look {
    if (level < header[0]) {
        let slot = live_slot(level, c.x, c.y);
        if (bitcast<i32>(slot.z) != c.x || bitcast<i32>(slot.w) != c.y) {
            return empty_look(false);
        }
        return from_band(slot.x, slot.y, c.z);
    }
    let k = level - header[0];
    let base = 6 + 4 * k;
    let x = c.x - header[base];
    let y = c.y - header[base + 1];
    let across = header[base + 2];
    if (x < 0 || y < 0 || x >= across || y >= across) {
        // Past the map: nothing.
        return empty_look(true);
    }
    let entry = baked[u32(header[base + 3] + y * across + x)];
    return from_band(entry.x, entry.y, c.z);
}

// A cell column's entry at `level`: where its band is in the pool, and its lowest cell and count (`NOT_BAKED`
// if it isn't there yet).
fn entry_at(level: i32, cx: i32, cy: i32) -> vec2<u32> {
    if (level < header[0]) {
        let slot = live_slot(level, cx, cy);
        if (bitcast<i32>(slot.z) != cx || bitcast<i32>(slot.w) != cy) {
            return vec2<u32>(0u, NOT_BAKED);
        }
        return slot.xy;
    }
    let k = level - header[0];
    let base = 6 + 4 * k;
    let x = cx - header[base];
    let y = cy - header[base + 1];
    let across = header[base + 2];
    if (x < 0 || y < 0 || x >= across || y >= across) {
        // Past the map: nothing.
        return vec2<u32>(0u, PAST_MAP);
    }
    return baked[u32(header[base + 3] + y * across + x)];
}

// A smooth blend of the eight cells round `p` at `level` (trilinear: no threshold, so a slope's edge is a
// gradient, never stairs); `known` false if any of them isn't there. Each of the four cell columns' entries
// is read once, then its two cells.
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
        let entry = entry_at(level, base.x + ox, base.y + oy);
        if (entry.y == NOT_BAKED) {
            return empty_look(false);
        }
        let wxy = select(1.0 - f.x, f.x, ox == 1) * select(1.0 - f.y, f.y, oy == 1);
        for (var oz = 0; oz < 2; oz++) {
            let look = from_band(entry.x, entry.y, base.z + oz);
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

// Where in a level's stretch (0 its start, 1 where the next takes over) the blend with the next begins.
// (`camera.fade.z`: 0.8 unless chosen on the Render page (0.6 before build 109) [2026-11-10, the dyno: later, a narrower hand-over, fewer
// steps reading two levels: at 0.8 the whole flight's 95th 6.7% faster; David's eye on rings over flat water].)

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

const HORIZON: vec3<f32> = vec3<f32>(0.62, 0.72, 0.86);
const ZENITH: vec3<f32> = vec3<f32>(0.16, 0.32, 0.66);
// Night's sky as a share of the day's, at night's light 1 (bluer: the land under starlight looks blue, not grey).
const NIGHT_SKY: vec3<f32> = vec3<f32>(0.35, 0.45, 0.8);
// The dusk's glow low over the horizon where the sun went down.
const DUSK: vec3<f32> = vec3<f32>(1.0, 0.45, 0.18);

// The sky's day colour seen along `d`, darkened through the dusk to night's (its brightness `camera.daylight.z`),
// and the dusk's glow where the sun went down, while it's near the horizon.
fn sky_tone(day: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    let night = day * NIGHT_SKY * camera.daylight.w;
    var colour = mix(night, day, camera.daylight.z);
    // The dusk: strongest with the sun just under the horizon, low in the sky on its side.
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

// ── Fog (3.4c, David 2026-11-08: "that sounds alright") ──
//
// The air between the eye and what it sees: thickest at sea level, halving every so many blocks up, so valleys
// fill with haze and peaks stand clear. How much a ray goes through is worked out whole along it (the thickness
// falls off exponentially with height, so its sum along a straight line has a closed form): no steps, nearly
// free. Land fades to the sky's horizon colour, so far mountains melt into the sky rather than stand on it as
// cut-outs; a little warmer looking towards the sun.
const FOG_WARM: vec3<f32> = vec3<f32>(1.0, 0.82, 0.62);

// `colour` seen `t` off along `d` through the fog.
fn fogged(colour: vec3<f32>, d: vec3<f32>, t: f32) -> vec3<f32> {
    if (camera.fog.x <= 0.0) {
        return colour;
    }
    let thins = camera.fog.y;
    // The thickness at the eye, then summed along the ray as it climbs or falls: e^(−thins·z) from 0 to t.
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

// The level a ray reads at distance `t`: its pixel's size there, in whole powers of two, never finer than
// the finest level there is (and where that one isn't kept, the next coarser that is).
fn level_at(t: f32) -> i32 {
    let footprint = max(t * camera.view.z, 1.0);
    let level = i32(floor(log2(footprint))) + i32(camera.reach.y);
    return kept(clamp(level, header[2], header[1]));
}

// The light is kept in two parts (3.4b): the sky's, always there (`camera.sun.w` of a face's colour, by day), and
// the sun's, which a shadow takes away. `lit` is both; `sunlit` the sun's alone. Each as bright as the time of day
// has it: the sun's gone with it under the horizon, the sky's down to night's light.
fn lit(colour: vec3<f32>, light: f32) -> vec3<f32> {
    return colour * (camera.sun.w * camera.daylight.y + (1.0 - camera.sun.w) * camera.daylight.x * light);
}

fn sunlit(colour: vec3<f32>, light: f32) -> vec3<f32> {
    return colour * (1.0 - camera.sun.w) * camera.daylight.x * light;
}

// How much sun a face gets by its facing.
fn face_light(facing: vec3<f32>) -> f32 {
    return max(dot(facing, camera.sun.xyz), 0.0);
}

// How much sun a cell's faces get, seen along `d` (3.4a): each way's faces lit as themselves (the sun on a face
// looking north is the sun's northward part, or none), and weighted by how much of that way the eye sees (a face
// looking north shows as much as the eye looks south), so a far terraced hillside shows the tops and sides it
// really shows, each as bright as it really is. A cell whose faces all look away (the eye inside it) is lit by
// all of them.
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

// The highest anything stands in what's on the card so far (the store keeps it as regions and tiles go up):
// over it, a ray that isn't coming down meets nothing more.
fn world_top() -> f32 {
    return min(max(camera.reach.w, 0.0), WORLD_HEIGHT);
}

// How far along the ray it leaves the cell `c` of `size`.
fn leave(o: vec3<f32>, d: vec3<f32>, c: vec3<f32>, size: f32) -> f32 {
    let next = (c + select(vec3<f32>(0.0), vec3<f32>(1.0), d > vec3<f32>(0.0))) * size;
    let t = (next - o) / d;
    let big = 1e30;
    let tx = select(big, t.x, abs(d.x) > 1e-6);
    let ty = select(big, t.y, abs(d.y) > 1e-6);
    let tz = select(big, t.z, abs(d.z) > 1e-6);
    return min(tx, min(ty, tz));
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

// ── Blocks as blocks (3.3, cells of 1, David 2026-11-06: "i think you're right about 1's next") ──
//
// Near the eye, where a block is bigger than a pixel, a cell's average is a fog, not the truth (David's near look:
// "the blocks up close exist only as clouds"). There the ray walks the blocks one by one (cells of 1, the live
// level 0) and stops on the face it first meets, lit by that face's own facing: a hard edge. Past where a block is
// a pixel, the averages are the truth again and the march reads them; at that distance a block's soft edge is a
// pixel wide, so the two meet.

// ── Block textures (3.6, David 2026-11-08: "so the blocks are mipmapped, right? go ahead") ──
//
// Where the walk meets a face, the face's texture where it met it, at the size the pixel takes there: up close a
// texel is a square as painted, shrinking it steps down through the texture's shrunk versions, and where a block
// is a pixel it's one texel, the block's far colour, which the cells past it are made of.

const NO_TEXTURE: u32 = 0xFFFFu;

// Which of a block's faces looks along `facing` (`data::FACE_NAMES`' order: north, south, east, west, top, bottom).
fn face_of(facing: vec3<f32>) -> u32 {
    let a = abs(facing);
    if (a.z >= a.x && a.z >= a.y) {
        return select(5u, 4u, facing.z > 0.0);
    }
    if (a.x >= a.y) {
        return select(1u, 0u, facing.x > 0.0);
    }
    return select(3u, 2u, facing.y > 0.0);
}

// Where on its face a place on block `c` is: across from the left, down from the top, as seen from outside (the
// sides upright, the top with north up).
fn face_uv(p: vec3<f32>, c: vec3<i32>, face: u32) -> vec2<f32> {
    return face_uv_from(p, vec3<f32>(c), face);
}

// The same for a block whose least corner is `origin` (a whole block on the octant grid, half a block off the cells).
fn face_uv_from(p: vec3<f32>, origin: vec3<f32>, face: u32) -> vec2<f32> {
    let q = clamp(p - origin, vec3<f32>(0.0), vec3<f32>(1.0));
    switch face {
        case 0u: {
            return vec2<f32>(1.0 - q.y, 1.0 - q.z);
        }
        case 1u: {
            return vec2<f32>(q.y, 1.0 - q.z);
        }
        case 2u: {
            return vec2<f32>(q.x, 1.0 - q.z);
        }
        case 3u: {
            return vec2<f32>(1.0 - q.x, 1.0 - q.z);
        }
        case 4u: {
            return vec2<f32>(q.y, 1.0 - q.x);
        }
        default: {
            return vec2<f32>(q.y, q.x);
        }
    }
}

// The colour of block `block`'s face looking along `facing`, met at `p` on block `c`, `t` off: its texture's, or
// `flat` if it has none.
fn textured(flat: vec3<f32>, block: u32, p: vec3<f32>, c: vec3<i32>, facing: vec3<f32>, t: f32) -> vec3<f32> {
    if (camera.textures.x < 0.5 || block == 0u || block >= 256u) {
        return flat;
    }
    let face = face_of(facing);
    let layer = block_table[block * 2u + face / 4u][face % 4u];
    if (layer == NO_TEXTURE) {
        return flat;
    }
    var uv = face_uv(p, c, face);
    // A face that turns at random: a quarter turn or more, by the block's place (never changing).
    if (((block_table[block * 2u + 1u].z >> face) & 1u) != 0u) {
        let h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u);
        let turns = (h >> 7u) & 3u;
        for (var k = 0u; k < turns; k++) {
            uv = vec2<f32>(1.0 - uv.y, uv.x);
        }
    }
    return sample_layer(layer, uv, t);
}

// Texture `layer` at `uv`, `t` off: at the shrunk version for the pixel's size there (32 texels a block).
fn sample_layer(layer: u32, uv: vec2<f32>, t: f32) -> vec3<f32> {
    let texels = 32.0 * t * camera.view.z;
    let lod = log2(max(texels, 1e-4)) - camera.textures.y;
    return textureSampleLevel(block_textures, block_sampler, uv, i32(layer), lod).rgb;
}

// The texture put on a block's face the near walk met, after the march: the face's flat colour in what the ray
// gathered swapped for the texture's, lit the same (its sun's part and sky's part with it). Done here, once a ray,
// not in the walk: there it made every step of every ray heavier on the card, about 5 ms a frame [2026-11-08].
fn wear_texture(seen: Seen) -> Seen {
    if (seen.face_share <= 0.0 || camera.textures.x < 0.5) {
        return seen;
    }
    // The block behind the face (the face looks back out of it).
    let c = vec3<i32>(floor(seen.at - seen.facing * 0.5));
    let entry = entry_at(0, c.x, c.y);
    let look = from_band(entry.x, entry.y, c.z);
    let block = block_in_band(entry.x, entry.y, c.z);
    // A face with no surface of its own in the sums (one seen through a gap in a cell of pieces, which the sums
    // count as whole) was drawn rock grey: it still wears its block's texture.
    if (!look.known || (look.surface <= 0.0 && block == 0u)) {
        return seen;
    }
    let flat = select(look.colour, ROCK, look.surface <= 0.0);
    let t = length(seen.at - camera.position.xyz);
    let change = (textured(flat, block, seen.at, c, seen.facing, t) - flat) * seen.face_share;
    let light = face_light(seen.facing);
    var out = seen;
    out.colour += lit(change, light);
    out.sunny += sunlit(change, light);
    out.skyish += lit(change, light) - sunlit(change, light);
    out.glow += change * seen.glow_light;
    return out;
}

// ── Placed light (build 95, David through Counsel: "yeah, lets put the object in at the same time") ──
//
// The light things give (a brazier's), flooded through the air on the processor (`lights.rs`): a face met by the
// near walk takes the light of the air in front of it, on top of the sun's and the sky's, by its own colour (its
// texture's too). Each light flickers on its own beat. Squared, as the old renderer eased it, so it falls away
// softly towards the edge of its reach.

// The placed light in the air cell `c` (each colour 0 to 1, as it flickers now), none where it's dark.
fn placed_light(c: vec3<i32>) -> vec3<f32> {
    return placed_light_at(0u, c);
}

// The placed light kept for the cell `c` of `level`: at 0 a block's (the air's, and the block under lit ground), at
// the coarser levels the wash's (`lights.rs`: the lit ground's average over the cell) [build 101].
fn placed_light_at(level: u32, c: vec3<i32>) -> vec3<f32> {
    if (camera.glow.x < 0.5) {
        return vec3<f32>(0.0);
    }
    var h = ((bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u) ^ (level * 2654435761u)) & (LIGHT_SLOTS - 1u);
    for (var k = 0; k < LIGHT_PROBES; k++) {
        let e = lights[h];
        if (e.w == 0u) {
            return vec3<f32>(0.0);
        }
        if (bitcast<i32>(e.x) == c.x && bitcast<i32>(e.y) == c.y && bitcast<i32>(e.z) == c.z && ((e.w >> 20u) & 31u) == level) {
            let level = vec3<f32>(f32(e.w & 15u), f32((e.w >> 4u) & 15u), f32((e.w >> 8u) & 15u)) / 15.0;
            // Its beat: two waves at odd speeds, each light's own way along them.
            let beat = f32((e.w >> 12u) & 255u) / 255.0 * 6.2831853;
            let t = camera.glow.z;
            let wave = 0.5 + 0.3 * sin(t * 9.1 + beat) + 0.2 * sin(t * 23.7 + beat * 3.0);
            return level * level * (1.0 - camera.glow.y * wave);
        }
        h = (h + 1u) & (LIGHT_SLOTS - 1u);
    }
    return vec3<f32>(0.0);
}

// The placed light on a face, at `at` on it, looking along `facing`: the air in front of the face. A face takes
// its cell's light whole, so the light falls away a block's step at a time, hard-edged [David 2026-11-09, on build
// 95's blend: "the smooth light looks ok. but i just love the quantized per block lighting with the hard edges";
// `decisions.md`: "very clear steps in brightness"]. Or, chosen, blended between the four cells nearest across the
// face (as the old renderer kept light at each face's corners), falling away smoothly.
fn placed_light_on(at: vec3<f32>, facing: vec3<f32>) -> vec3<f32> {
    if (camera.glow.x < 0.5) {
        return vec3<f32>(0.0);
    }
    if (camera.glow.w < 0.5) {
        return placed_light(vec3<i32>(floor(at + facing * 0.5)));
    }
    let along = abs(round(facing)) > vec3<f32>(0.5);
    // The middle of the layer of air in front of the face, and across it, the cells' middles round the point.
    let q = at + facing * 0.5;
    let low = select(floor(q - 0.5), floor(q), along);
    let f = select(q - 0.5 - floor(q - 0.5), vec3<f32>(0.0), along);
    // The two ways across the face.
    let u = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), along.x);
    let v = select(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), along.z);
    let fu = dot(f, u);
    let fv = dot(f, v);
    let c = vec3<i32>(low);
    let iu = vec3<i32>(u);
    let iv = vec3<i32>(v);
    return placed_light(c) * (1.0 - fu) * (1.0 - fv) + placed_light(c + iu) * fu * (1.0 - fv) + placed_light(c + iv) * (1.0 - fu) * fv + placed_light(c + iu + iv) * fu * fv;
}

// What the walk through the blocks found: whether it met a face (and where), and what it gathered on the way
// (water's surface); not met, the march goes on from `t`.
struct Hard {
    met: bool,
    t: f32,
    first: f32,
    colour: vec3<f32>,
    // The sun's part of `colour` (what a shadow takes away), and the face met's facing.
    sunny: vec3<f32>,
    facing: vec3<f32>,
    through: f32,
    steps: i32,
    // How much of the ray the face met took (what was left of it after any water on the way): its texture is
    // put on after the march, by that much (`wear_texture`).
    face_share: f32,
    // The placed light on the face met (each colour 0 to 1), and what it gave (the face's colour by it, by how
    // much of the ray the face took): kept apart from the sun's and the sky's, which shadows and sky light scale.
    glow_light: vec3<f32>,
    glow: vec3<f32>,
}

// ── Pieces (build 90, David on build 88: "no quarter blocks or grid shifts, they appear as white cubes") ──
//
// A cell built of pieces is eight octants (half a block a side), each empty or a block's. The near walk, meeting
// one, steps through its octants to the first filled one (a ray crosses at most four), and that octant's face is
// what it sees, its block's texture on it where the octant lies on the block's face (half the texture across).

// Where the cell of pieces at `c` is in the table, or −1 if it isn't there.
fn pieces_slot(c: vec3<i32>) -> i32 {
    var h = ((bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u)) & (PIECES_SLOTS - 1u);
    for (var k = 0; k < PIECES_PROBES; k++) {
        let key = pieces[h * 2u];
        if (key.w == 0u) {
            return -1;
        }
        if (bitcast<i32>(key.x) == c.x && bitcast<i32>(key.y) == c.y && bitcast<i32>(key.z) == c.z) {
            return i32(h);
        }
        h = (h + 1u) & (PIECES_SLOTS - 1u);
    }
    return -1;
}

// Octant `o` (`x + 2y + 4z`) as the table has it: its block in the low byte, what it's part of (`Part`) in the
// high one; 0 if it's empty.
fn octant_word(slot: i32, o: i32) -> u32 {
    let w = pieces[u32(slot) * 2u + 1u][o / 2];
    return (w >> (16u * u32(o % 2))) & 0xFFFFu;
}

// One along an axis (0 x, 1 y, 2 z), as a vector.
fn unit(axis: u32) -> vec3<f32> {
    return vec3<f32>(f32(axis == 0u), f32(axis == 1u), f32(axis == 2u));
}

// An octant's corner in its cell (`o`'s bits), in blocks.
fn octant_corner(o: u32) -> vec3<f32> {
    return vec3<f32>(f32(o & 1u), f32((o >> 1u) & 1u), f32((o >> 2u) & 1u)) * 0.5;
}

// A bar's own texture's place, measured from the bar's own corner (the old renderer's `bar_uv`, before the
// regroup): a long side's picture runs up the bar's length, whichever way the bar lies, and on across a cell's
// side for a bar placed across two cells; an end's covers the half-by-half end (every face of a lone octant).
// `part`: what the octant is part of (`Part`: its bar's axis, 0 for a lone octant; which end of the bar it is).
fn bar_uv(face: u32, p: vec3<f32>, facing: vec3<f32>, o: u32, part: u32) -> vec2<f32> {
    let lone = (part & 3u) == 0u;
    let axis = select((part & 3u) - 1u, face / 2u, lone);
    let end = f32((part >> 2u) & 1u);
    let u = (axis + 1u) % 3u;
    let v = (axis + 2u) % 3u;
    let corner = octant_corner(o);
    let start = dot(corner, unit(axis)) - end * 0.5;
    // Where in its cell (0 to 1 each way), from a hair inside the face.
    let inside = p - facing * 0.001;
    let c = inside - floor(inside);
    let along = dot(c, unit(axis)) - start;
    let across_u = dot(c - corner, unit(u)) * 2.0;
    let across_v = dot(c - corner, unit(v)) * 2.0;
    if (face / 2u == axis) {
        return vec2<f32>(across_u, 1.0 - across_v);
    }
    return vec2<f32>(select(across_u, across_v, face / 2u == u), 1.0 - along);
}

// An octant's face's colour, met at `at` in the cell of pieces `c`, `t` off: a bar's own long side or end if its
// block has them (build 91), a whole block on the octant grid its own face lined up with the block, else the
// share of its block's face where the octant lies; with textures off, the texture shrunk to its one texel (the
// block's far colour); rock grey for a block with none.
fn octant_colour(word: u32, o: u32, at: vec3<f32>, c: vec3<i32>, facing: vec3<f32>, t: f32) -> vec3<f32> {
    let block = word & 0xFFu;
    let part = word >> 8u;
    let face = face_of(facing);
    var layer = block_table[block * 2u + face / 4u][face % 4u];
    var uv = face_uv(at, c, face);
    let bars = block_table[block * 2u + 1u].w;
    if ((part & 16u) != 0u) {
        // A whole block on the octant grid (`Part::of_block`): which of its corners this octant is.
        let origin = vec3<f32>(c) + octant_corner(o) - octant_corner(part >> 5u);
        uv = face_uv_from(at, origin, face);
    } else if ((bars & 0xFFFFu) != NO_TEXTURE && (bars >> 16u) != NO_TEXTURE) {
        let lone = (part & 3u) == 0u;
        let axis = select((part & 3u) - 1u, face / 2u, lone);
        layer = select(bars & 0xFFFFu, bars >> 16u, face / 2u == axis);
        // (Kept off the far edge, which the sampler would wrap back to the near one.)
        uv = clamp(bar_uv(face, at, facing, o, part), vec2<f32>(0.0), vec2<f32>(0.9999));
    }
    if (layer == NO_TEXTURE) {
        return ROCK;
    }
    if (camera.textures.x < 0.5) {
        return textureSampleLevel(block_textures, block_sampler, vec2<f32>(0.5), i32(layer), 16.0).rgb;
    }
    return sample_layer(layer, uv, t);
}

struct Octant {
    met: bool,
    t: f32,
    facing: vec3<f32>,
    word: u32,
    o: u32,
}

// The ray through the cell of pieces `c` (in its table place `slot`) from `t0`, where it came in facing
// `facing0`: the first filled octant it meets, if any.
fn octants(o: vec3<f32>, d: vec3<f32>, t0: f32, c: vec3<i32>, facing0: vec3<f32>, slot: i32) -> Octant {
    var out: Octant;
    let s = select(vec3<i32>(-1), vec3<i32>(1), d > vec3<f32>(0.0));
    let across = select(0.5 / abs(d), vec3<f32>(1e30), abs(d) < vec3<f32>(1e-6));
    let base = vec3<f32>(c);
    var q = clamp(vec3<i32>(floor((o + d * t0 - base) * 2.0)), vec3<i32>(0), vec3<i32>(1));
    let next = base + (vec3<f32>(q) + select(vec3<f32>(0.0), vec3<f32>(1.0), d > vec3<f32>(0.0))) * 0.5;
    var ahead = select((next - o) / d, vec3<f32>(1e30), abs(d) < vec3<f32>(1e-6));
    var t = t0;
    var facing = facing0;
    for (var n = 0; n < 4; n++) {
        let o = q.x + 2 * q.y + 4 * q.z;
        let word = octant_word(slot, o);
        if (word != 0u) {
            out.met = true;
            out.t = t;
            out.facing = facing;
            out.word = word;
            out.o = u32(o);
            return out;
        }
        if (ahead.x < ahead.y && ahead.x < ahead.z) {
            t = ahead.x;
            q.x += s.x;
            ahead.x += across.x;
            facing = vec3<f32>(-f32(s.x), 0.0, 0.0);
        } else if (ahead.y < ahead.z) {
            t = ahead.y;
            q.y += s.y;
            ahead.y += across.y;
            facing = vec3<f32>(0.0, -f32(s.y), 0.0);
        } else {
            t = ahead.z;
            q.z += s.z;
            ahead.z += across.z;
            facing = vec3<f32>(0.0, 0.0, -f32(s.z));
        }
        if (any(q < vec3<i32>(0)) || any(q > vec3<i32>(1))) {
            return out;
        }
    }
    return out;
}

// Where the ray leaves block `c` across each axis's next face.
fn faces_ahead(o: vec3<f32>, d: vec3<f32>, c: vec3<i32>) -> vec3<f32> {
    let next = vec3<f32>(c + select(vec3<i32>(0), vec3<i32>(1), d > vec3<f32>(0.0)));
    return select((next - o) / d, vec3<f32>(1e30), abs(d) < vec3<f32>(1e-6));
}

// The blocks from `t0` to `t1` along the ray; the placed light on the face met if `glowing` (the eye's rays, not the
// shadows').
fn hard(o: vec3<f32>, d: vec3<f32>, t0: f32, t1: f32, glowing: bool) -> Hard {
    var out: Hard;
    out.t = t0;
    out.first = camera.reach.x;
    out.through = 1.0;
    let s = select(vec3<i32>(-1), vec3<i32>(1), d > vec3<f32>(0.0));
    let across = select(1.0 / abs(d), vec3<f32>(1e30), abs(d) < vec3<f32>(1e-6));
    var t = t0;
    var c = vec3<i32>(floor(o + d * t));
    var ahead = faces_ahead(o, d, c);
    // The face it came in by (the eye's own block: facing the eye).
    var facing = -d;
    var in_water = false;
    for (var n = 0; n < 512; n++) {
        out.steps = n + 1;
        if (t >= t1) {
            out.t = t1;
            return out;
        }
        // Over the world's highest point going up: nothing more to meet (as in the march).
        let p = o + d * t;
        if (p.z > world_top() + 1.0 && d.z >= 0.0) {
            out.t = t1;
            return out;
        }
        // Air over a coarser cell column's top (the bake's 64 and 32, then the live 16 and 8) is skipped whole,
        // as in the march (not in the tests' truth: every block walked).
        var skipped = false;
        for (var k = 0; k < 4 && !in_water && camera.follow.w < 0.5; k++) {
            let up = 6 - k;
            let size = f32(1 << u32(up));
            let cc = floor(p.xy / size);
            let entry = entry_at(up, i32(cc.x), i32(cc.y));
            if (entry.y == NOT_BAKED) {
                continue;
            }
            let low = bitcast<i32>(entry.y << 16u) >> 16u;
            let count = i32(entry.y >> 16u);
            let top = f32(low + count) * size;
            if (p.z <= top) {
                continue;
            }
            var next = leave2(o.xy, d.xy, cc, size);
            if (d.z < 0.0) {
                next = min(next, t + (top - p.z) / d.z);
            }
            if (next >= t1 || next > t + 0.02) {
                // Never on into a column whose blocks aren't made (past their window, or not made yet): there the walk
                // hands over to the march, and a march started on a surface's top misses the soft edge over it that it
                // gathers coming down to it [2026-11-09: looking down into a lake past the blocks made, up to 24 steps
                // of colour from every block walked]. A finer column's skip, or a block at a time to it, as every
                // block walked does. (A column not made between here and there isn't looked for: the window's made
                // nearest first.)
                let to = select(next - 0.01, t1, next >= t1);
                let lands = floor((o + d * to).xy);
                if (entry_at(0, i32(lands.x), i32(lands.y)).y == NOT_BAKED) {
                    continue;
                }
            }
            if (next >= t1) {
                out.t = t1;
                return out;
            }
            if (next > t + 0.02) {
                // A little short of it, so the walk crosses into the next block itself (and knows by which face).
                t = next - 0.01;
                c = vec3<i32>(floor(o + d * t));
                ahead = faces_ahead(o, d, c);
                facing = -d;
                skipped = true;
                break;
            }
        }
        if (skipped) {
            continue;
        }
        let entry = entry_at(0, c.x, c.y);
        if (entry.y == NOT_BAKED) {
            // Not made yet: the march takes over from here.
            out.t = t;
            return out;
        }
        let look = from_band(entry.x, entry.y, c.z);
        // A cell of pieces: its octants walked; passed through them all, the walk goes on.
        var through_pieces = false;
        if (look.solid >= 0.5 && block_in_band(entry.x, entry.y, c.z) == PIECES_MARK) {
            let slot = pieces_slot(c);
            if (slot >= 0) {
                let hit = octants(o, d, t, c, facing, slot);
                if (hit.met) {
                    out.met = true;
                    out.t = hit.t;
                    let at = o + d * hit.t;
                    let colour = octant_colour(hit.word, hit.o, at, c, hit.facing, hit.t);
                    let light = face_light(hit.facing);
                    out.colour += out.through * lit(colour, light);
                    out.sunny += out.through * sunlit(colour, light);
                    // The air in front of the octant's face (in its own cell, or the next).
                    if (glowing) {
                        out.glow_light = placed_light_on(at, hit.facing);
                    }
                    out.glow += out.through * colour * out.glow_light;
                    out.facing = hit.facing;
                    if (out.through > 0.5) {
                        out.first = hit.t;
                    }
                    // Its texture's on already.
                    out.face_share = 0.0;
                    out.through = 0.0;
                    return out;
                }
                through_pieces = true;
            }
        }
        if (look.solid >= 0.5 && !through_pieces) {
            out.met = true;
            out.t = t;
            let colour = select(look.colour, ROCK, look.surface <= 0.0);
            let light = face_light(facing);
            out.colour += out.through * lit(colour, light);
            out.sunny += out.through * sunlit(colour, light);
            // The air in front of the face: the block it came from.
            if (glowing) {
                out.glow_light = placed_light_on(o + d * t, facing);
            }
            out.glow += out.through * colour * out.glow_light;
            out.facing = facing;
            if (out.through > 0.5) {
                out.first = t;
            }
            out.face_share = out.through;
            out.through = 0.0;
            return out;
        }
        // A medium (water, glass, ice: build 98, David: "as long as transparency is done well and flexible"): what's
        // seen through it dimmed and tinted by how far the ray goes through this block, by the block's own medium (a
        // twentieth left at its clarity). It took the place of water's fixed 85% at its surface.
        in_water = look.water >= 0.5;
        if (in_water) {
            let m = medium_of(block_in_band(entry.x, entry.y, c.z));
            let leaving = min(min(min(ahead.x, ahead.y), ahead.z), t1);
            let keep = exp(-m.w * max(leaving - t, 0.0));
            let taken = out.through * (1.0 - keep);
            out.colour += taken * lit(m.rgb, 1.0);
            out.sunny += taken * sunlit(m.rgb, 1.0);
            if (out.through > 0.5 && out.through * keep <= 0.5) {
                out.first = t;
            }
            out.through *= keep;
            // Seen through no further: nothing behind shows.
            if (out.through < 0.01) {
                out.t = leaving;
                return out;
            }
        }
        // On to the next block, across whichever face comes first.
        if (ahead.x < ahead.y && ahead.x < ahead.z) {
            t = ahead.x;
            c.x += s.x;
            ahead.x += across.x;
            facing = vec3<f32>(-f32(s.x), 0.0, 0.0);
        } else if (ahead.y < ahead.z) {
            t = ahead.y;
            c.y += s.y;
            ahead.y += across.y;
            facing = vec3<f32>(0.0, -f32(s.y), 0.0);
        } else {
            t = ahead.z;
            c.z += s.z;
            ahead.z += across.z;
            facing = vec3<f32>(0.0, 0.0, -f32(s.z));
        }
    }
    out.t = t;
    return out;
}

// One ray: its colour (linear light) and how far it went before it was half covered.
struct Seen {
    colour: vec3<f32>,
    distance: f32,
    steps: f32,
    level: i32,
    // For its shadow (3.4b), looked up only where the pixel is shown: the sun's part of `colour`, the place, how
    // wide the shadow ray starts, and the face's facing if it's a block's (`on_block`).
    sunny: vec3<f32>,
    at: vec3<f32>,
    wide: f32,
    facing: vec3<f32>,
    on_block: bool,
    // The sky's part of `colour` (the light that isn't the sun's, on the surfaces gathered: what sky light scales).
    skyish: vec3<f32>,
    // Where it first touched anything (a finer layer's rays start short of it: before the soft edge of what it
    // read, not inside it).
    touched: f32,
    // How much of it a block's face met by the near walk took (0: none): its texture goes on after (`wear_texture`).
    face_share: f32,
    // The placed light on the face met, and what it gave (`Hard`'s): added after the shadow and sky light.
    glow_light: vec3<f32>,
    glow: vec3<f32>,
    // How much of it went on to the sky (the sky's colour in `colour` that much): the sky behind isn't fogged.
    sky_share: f32,
}

// A ray's colour with its shadow and its sky light (`open`: how much of the sky it sees, 0 to 1).
fn shaded(seen: Seen, open: f32) -> vec3<f32> {
    if (seen.wide <= 0.0) {
        return seen.colour;
    }
    var colour = seen.colour;
    if (camera.follow.y > 0.5) {
        colour -= seen.sunny * (1.0 - sunlight(seen.at, seen.wide, seen.facing, seen.on_block));
    }
    // The sky's part: as much as the place sees of the sky (David's strength), tinted (his tint, white to a sky blue).
    let sky = select(1.0, mix(1.0, open, camera.sky.y), camera.sky.x > 0.5);
    let tint = mix(vec3<f32>(1.0), SKY_TINT, camera.sky.z);
    return colour + seen.skyish * (sky * tint - 1.0);
}

// What kind of surface a ray's sky light was for: 0 the cells, 1 to 6 a block's face by its facing (a finer pixel
// takes its parent's sky only for the same kind).
fn surface_kind(seen: Seen) -> u32 {
    if (!seen.on_block) {
        return 0u;
    }
    let f = seen.facing;
    if (abs(f.x) > 0.5) {
        return select(2u, 1u, f.x > 0.0);
    }
    if (abs(f.y) > 0.5) {
        return select(4u, 3u, f.y > 0.0);
    }
    return select(6u, 5u, f.z > 0.0);
}

// A pixel's sky light as handed down: its kind (3 bits) over its sky seen in 65,535ths; `NO_SKY` none.
const NO_SKY: u32 = 0x70000u;

fn sky_word(kind: u32, open: f32) -> u32 {
    return (kind << 16u) | u32(round(clamp(open, 0.0, 1.0) * 65535.0));
}

// ── Sky light (3.4b's second half, David 2026-11-08: "tint slider is fine") ──
//
// The light from the sky isn't the same everywhere: a ridge sees all of it, a valley floor less, the foot of a cliff
// half, an overhang's underside none. Five wide cones up through the store, one straight up and four leaning 50°
// out (north, east, south, west), each weighted by how high it looks (the sky overhead lights a flat surface most),
// say how much of the sky a place sees. Wide (`camera.sky.w` a block of distance), so they take few steps and see
// the land round about softly, as sky light is.

// The tint at its fullest: a sky blue as bright as white (its brightness 1), so a tinted shade is cooler, not darker.
const SKY_TINT: vec3<f32> = vec3<f32>(0.84, 1.01, 1.40);
// The most steps a sky cone takes.
const SKY_STEPS: i32 = 40;

fn sky_seen(p: vec3<f32>, footprint: f32, facing: vec3<f32>, on_block: bool) -> f32 {
    let lean = sin(radians(50.0));
    let rise = cos(radians(50.0));
    // The five cones in one loop, so the march holds one copy of the cone's code, not five [the dyno, 2026-11-09:
    // Nsight, the march's 30,600 instructions too many for the card's instruction cache; the picture the same to the
    // pixel, and the card 3% faster on the median, 5 to 6% at the 95th and 99th]: straight up, then north, south,
    // east and west, added up in that order as when written out one by one. The count read from the store
    // (`header[0]`, never below 0), so the compiler can't unroll the loop back into five copies.
    let cones = 5 + min(header[0], 0);
    var seen = 0.0;
    for (var k = 0; k < cones; k++) {
        var way = vec3<f32>(0.0, 0.0, 1.0);
        var weight = 1.0;
        if (k > 0) {
            let s = select(-lean, lean, (k & 1) == 1);
            way = select(vec3<f32>(0.0, s, rise), vec3<f32>(s, 0.0, rise), k <= 2);
            weight = rise;
        }
        seen += weight * cone(p, footprint, facing, on_block, way, camera.sky.w, 0.0, 0.0, SKY_STEPS);
    }
    return seen / (1.0 + 4.0 * rise);
}

// A number from 0 to 1 for a pixel, steady from frame to frame (a hash of its place).
fn scatter(at: vec2<u32>) -> f32 {
    var h = at.x * 0x9E3779B1u ^ (at.y * 0x85EBCA77u);
    h = (h ^ (h >> 15u)) * 0x2C1B3C6Du;
    h = h ^ (h >> 12u);
    return f32(h & 0xFFFFu) / 65536.0;
}

// `phase`: a pixel's own number from 0 to 1 (`scatter`), where its fine steps fall within a fine step, so
// their places against the ground don't line up across the screen (lined up, they draw rings round the eye).
fn march(o: vec3<f32>, d: vec3<f32>, phase: f32, begin: f32) -> Seen {
    var colour = vec3<f32>(0.0);
    // The sun's part of it (3.4b: what the shadow at the surface takes away).
    var sunny = vec3<f32>(0.0);
    // The placed light gathered (build 101: the wash, far off), added after the shadow and sky light.
    var glow = vec3<f32>(0.0);
    var through = 1.0;
    var first = camera.reach.x;
    var touched = camera.reach.x;
    var steps = 0;
    // Near the eye, out to where a block is a pixel (and inside the blocks' window), the blocks as blocks.
    var start_at = begin;
    if (header[2] == 0 && is_kept(0)) {
        let blocks_end = min(1.0 / camera.view.z, camera.layer.w);
        if (begin < blocks_end) {
            // From the eye itself when the start is the first half block (the start pass's and a ray's own least):
            // started there, a face nearer than half a block was met inside its block, facing the eye, its shadow
            // and sky rays starting in the rock [David after build 84: "when you put your face right into a block, a
            // circle appears": the disc where the wall is nearer than half a block, dark, cut into wedges by the sky
            // cones each taking it as facing towards or away from them].
            let h = hard(o, d, select(max(begin, 0.0), 0.0, begin <= 0.5), blocks_end, true);
            steps = h.steps;
            if (h.met) {
                // The face met: its shadow from the blocks themselves first.
                return Seen(h.colour, h.first, f32(steps), 0, h.sunny, o + d * h.t, 1.0, h.facing, true, h.colour - h.sunny, min(h.first, h.t), h.face_share, h.glow_light, h.glow, 0.0);
            }
            colour = h.colour;
            sunny = h.sunny;
            glow = h.glow;
            through = h.through;
            first = h.first;
            touched = h.first;
            start_at = h.t;
        }
    }
    var t = max(start_at, 0.5) + phase;
    // Whether the last step found only air (only then is skipping tried: among the ground it never pays).
    var in_air = true;
    // The last place a step found only air, and whether the ray is walking into a surface in fine steps from
    // there [2026-11-06, David's first look: "a severe ringing effect on flat surfaces or water beneath me":
    // the first step to touch the ground after a stride through air landed wherever the stride fell, up to two
    // cells deep, and what it gathered swung with that, drawing contours on a flat plane].
    var last_air = t;
    var walking = false;
    // Where the walk in has to reach (the place it first touched) before strides start again.
    var walk_to = 0.0;
    // (Half covered already by water met among the blocks: read from the blocks.)
    var first_level = select(-1, 0, first < camera.reach.x);
    // The last sample, for gathering each step from both its ends (trapezoids: what's gathered then hardly
    // depends on where the steps fall; one end alone left contours that moved as the eye rose and fell, David
    // build 69: "the oceans still show the rings... so fast"). `chained` false after a skip or a back-up.
    var before: Look;
    var before_t = 0.0;
    var chained = false;
    let ceiling = world_top() + 32.0;
    // By a surface, steps sized to how fast the ray crosses it upward or downward: a quarter of a cell for a ray
    // looking down at it, up to a whole cell for one skimming along it (it crosses the soft edge slowly; the
    // steps' two ends keep the sum true).
    let surface_step = 0.25 / clamp(abs(d.z), 0.25, 1.0);
    // The far cells' see-through share is the world's water's medium (build 98).
    let water_medium = media[0];
    let water_colour = water_medium.rgb;
    let walked = steps;
    // Whether it went down through the world's floor, and what it last gathered in the cells over the floor (their
    // colour as lit, the sun's part, where and from which level).
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
        steps = walked + n + 1;
        let p = o + d * t;
        // Over the world's highest point going up, or under its floor going down: nothing more to meet.
        if (p.z > ceiling && d.z >= 0.0) {
            break;
        }
        if (p.z < -8.0 && d.z <= 0.0) {
            floored = true;
            break;
        }
        var level = level_at(t);
        // A coarse level not built over here yet (it waits for all its regions): the regions' own level. A
        // region not baked yet holds nothing to draw: skipped whole (the raw view shows the sky there).
        if (level > REGION) {
            let c = floor(p.xy / f32(1 << u32(level)));
            if (entry_at(level, i32(c.x), i32(c.y)).y == NOT_BAKED) {
                level = REGION;
            }
        }
        if (level >= header[0] && level <= REGION) {
            let r = floor(p.xy / f32(1 << u32(REGION)));
            if (entry_at(REGION, i32(r.x), i32(r.y)).y == NOT_BAKED) {
                t = max(leave2(o.xy, d.xy, r, f32(1 << u32(REGION))), t) + 0.01;
                last_air = t;
                walking = false;
                chained = false;
                continue;
            }
        }
        // Space a coarser cell says is empty is skipped whole: the biggest such cell round the ray, up to
        // four levels coarser.
        // Over a coarse cell column's top (its band's top: nothing above it, its finer columns' tops all under
        // it), the ray goes on to where it leaves the column or comes down to the top, whichever is sooner. The
        // column two levels coarser round it, else the bake's first (always there, where a live level's tile isn't
        // made yet [2026-11-07: the fine levels' rays walked cell by cell while the 8s and 16s were still being
        // made]), else its own, with a fine cell's room above the top.
        var skipped = false;
        let fine = f32(1 << u32(level));
        for (var k = 0; k < 3 && in_air && !walking; k++) {
            // Two levels coarser, then the bake's first, then its own.
            var up = level;
            if (k == 0) {
                up = kept(min(level + 2, header[1]));
            } else if (k == 1) {
                if (level + 2 >= header[0]) {
                    continue;
                }
                up = header[0];
            }
            let up_size = f32(1 << u32(up));
            let cc = floor(p.xy / up_size);
            let entry = entry_at(up, i32(cc.x), i32(cc.y));
            if (entry.y == NOT_BAKED) {
                continue;
            }
            let low = bitcast<i32>(entry.y << 16u) >> 16u;
            let count = i32(entry.y >> 16u);
            let top = f32(low + count) * up_size + fine;
            if (p.z <= top) {
                continue;
            }
            var next = leave2(o.xy, d.xy, cc, up_size);
            if (d.z < 0.0) {
                // Where it comes down to the top (both negative: a positive way on) [2026-11-07: the sign was
                // turned, so rays coming down never skipped to a top, only strode].
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
        var look = blend(level, p);
        // A live level not there (past its window, or not made yet): the next coarser kept, down to the first
        // baked one.
        for (var up = 0; up < 8 && !look.known && level < header[0]; up++) {
            level = kept(level + 1);
            look = blend(level, p);
        }
        // Over the last stretch before the next level takes over, the two blended, so the change is gradual,
        // never a seam (seams drew rings round the eye over flat water that moved as it moved: David, build 69).
        let over = log2(max(t * camera.view.z, 1.0)) + camera.reach.y - f32(level);
        if (look.known && over > camera.fade.z && level < header[1]) {
            let next = blend(kept(level + 1), p);
            if (next.known) {
                look = mix_looks(look, next, (over - camera.fade.z) / (1.0 - camera.fade.z));
            }
        }
        let size = f32(1 << u32(level));
        let thick = select(0.0, max(look.solid, look.water), look.known);
        // The first touch of a surface after a stride through air: back to where the air was, and into the
        // surface from there in fine steps, so what's gathered never depends on where the stride fell.
        if (thick > 0.0 && !walking && t - last_air > size * 0.26) {
            walk_to = t;
            t = last_air + phase * size * 0.25;
            walking = true;
            // No skipping while walking in (a skip from here only leapt back to where it was).
            in_air = false;
            chained = false;
            continue;
        }
        // Air within a cell all round: two cells on (the next sample still reads everything between). By a
        // surface, a quarter of a cell at a time, where the coverage changes.
        var dt = size * 2.0;
        in_air = true;
        if (thick > 0.0) {
            in_air = false;
            walking = true;
            dt = select(size * 0.5, size * surface_step, thick > 0.02);
        } else if (walking && t < walk_to) {
            // Still walking in to where it first touched: fine steps, air or not.
            in_air = false;
            dt = size * surface_step;
        } else {
            last_air = t;
            walking = false;
            // Going down, never past the world's floor (under it all is rock): where the cells are as tall as all the
            // ground, a stride from over its top landed under the floor and the ray met nothing, a hole of sky (seen
            // from 30,000 up).
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
            // Dimmed by the medium as far as the step goes through it (its share of the cells).
            let water = 1.0 - exp(-water_medium.w * water_share * length);
            let covered = 1.0 - (1.0 - solid) * (1.0 - water);
            if (covered > 0.0) {
                // The first touch: where this step began.
                touched = min(touched, before_t);
                // Each end's colour as lit, weighted by how much solid it has.
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
                // The placed light washed over the ground here, by the ground's own colour, whole cells at a time (no
                // sun or sky takes it away: added after them) [build 101].
                if (camera.glow.x > 0.5 && solid > 0.0) {
                    let washed = placed_light_at(u32(level), vec3<i32>(floor(p / size)));
                    if (any(washed > vec3<f32>(0.0))) {
                        let albedo = (before.colour * before.solid + look.colour * look.solid) / solids;
                        glow += through * covered * albedo * washed * solid / both;
                    }
                }
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
    // Down through the world's floor having gathered from the cells just over it, it has met the ground there
    // whatever the averages said (under the ground is rock): what it didn't gather is that ground too, not the sky
    // [David after build 84: "the circles of light that swallow the world if i fly tens of miles over the surface
    // straight up looking down": a far cell is an average, a cell of 1,024 over 150 blocks of ground about a seventh
    // solid, so a ray looking down through it gathered a seventh of the ground and the rest the sky's colour]. Only
    // the ground over the floor: down through it over nothing (past the map, a region not baked yet), it stays sky
    // [David on 93: "on the peaks at a distance, a band juts out from either side... the band slides up and down the
    // tip with me": rays a little under the horizon brushed a peak's soft edge, went on over regions not baked yet and
    // through the floor, and all of what wasn't gathered was taken as that brush].
    if (floored && bottom) {
        colour += through * bottom_here;
        sunny += through * bottom_sun;
        through = 0.0;
        if (first >= camera.reach.x) {
            first = bottom_t;
            first_level = bottom_level;
        }
    }
    // What the surfaces gave (the sky behind them added after).
    let gathered = colour;
    colour += through * sky(d);
    // The shadow where the ray was half covered (the surface it shows), on everything it gathered; the shadow ray
    // as wide as the cells the surface was read from (or the eye's pixel there, if wider). None: nothing met.
    var wide = 0.0;
    if (first < camera.reach.x) {
        let read = select(1.0, f32(1 << u32(max(first_level, 0))), first_level >= 0);
        wide = max(first * camera.view.z, read);
    }
    return Seen(colour, first, f32(steps), first_level, sunny, o + d * first, wide, vec3<f32>(0.0), false, gathered - sunny, touched, 0.0, vec3<f32>(0.0), glow, through);
}

// ── Sun shadows (3.4b, David 2026-11-08: "proceed") ──
//
// How much of the sun reaches a place: a ray towards the sun through the same store the eye's rays read. By a
// block's face (near the eye) it walks the blocks themselves first, so a block's shadow is a block's, sharp. Past
// them, or from a far surface, it reads the averaged cells, widening as it goes (`camera.follow.z` across a block of
// distance) and reading the level as wide as it is: a far ridge's shadow comes soft-edged, as the sun's own width
// and the haze make it, and the steps grow with the distance. Air over a cell column's top is skipped whole (the
// ray only climbs), and once it's over the highest anything stands, nothing more can shade it.

// The most steps a shadow ray takes (past them it's taken as sunlit).
const SHADOW_STEPS: i32 = 160;
// How far the blocks are walked towards the sun from a face, at most (blocks).
const SHADOW_BLOCKS: f32 = 128.0;

// How much of the sun reaches `p` (`footprint` the eye's pixel there, blocks; `facing` the face's, on a block).
fn sunlight(p: vec3<f32>, footprint: f32, facing: vec3<f32>, on_block: bool) -> f32 {
    // A face turned from the sun has no sun to lose.
    return cone(p, footprint, facing, on_block, camera.sun.xyz, camera.follow.z, SHADOW_BLOCKS, 1.0, SHADOW_STEPS);
}

// How open the way `s` is from `p`, 0 to 1: a ray widening `spread` a block of distance, reading the store as wide
// as it is. On a block's face, the blocks walked first out to `blocks` (0: none), and `away` if the face looks away
// from `s`. The sun's shadow ray and the sky's cones (3.4b) are both this.
fn cone(p: vec3<f32>, footprint: f32, facing: vec3<f32>, on_block: bool, s: vec3<f32>, spread: f32, blocks: f32, away: f32, most: i32) -> f32 {
    var o = p;
    var t = 0.0;
    var light = 1.0;
    if (on_block) {
        if (dot(facing, s) <= 0.0) {
            return away;
        }
        // Off the face a hair, then (for the sun) the blocks walked; dimmed by any medium on the way (build 98).
        o = p + facing * 0.01;
        if (blocks > 0.0 && header[2] == 0 && is_kept(0)) {
            let h = hard(o, s, 0.0, min(camera.layer.w, blocks), false);
            if (h.met) {
                return 0.0;
            }
            t = h.t;
            light = h.through;
        }
    } else {
        // Clear of the surface's own soft edge first (the blend reaches half a cell either side of it): up half a
        // cell, and along the sun till a cell higher. The cell: as wide as `footprint`, or the level read there
        // if that's coarser (a live level not made yet reads the next one kept).
        let wanted = clamp(i32(floor(log2(max(footprint, 1.0)))), header[2], header[1]);
        let size = max(exp2(floor(log2(max(footprint, 1.0)))), f32(1 << u32(kept(wanted))));
        o = p + vec3<f32>(0.0, 0.0, size * 0.5);
        t = size / max(s.z, 0.1);
    }
    let top = world_top() + 1.0;
    for (var n = 0; n < most; n++) {
        let q = o + s * t;
        if (q.z > top || t > camera.reach.x) {
            break;
        }
        let wide = max(footprint, t * spread);
        var level = kept(clamp(i32(floor(log2(max(wide, 1.0)))), header[2], header[1]));
        // Over a column's top: on to where it leaves that column, the biggest it's over the top of, tried from the
        // coarsest level down to two coarser than it reads [2026-11-08: two coarser alone, a sunlit place's ray
        // climbed to the world's top in small hops, about 40 fps against 140 without shadows].
        var skipped = false;
        for (var up = header[1]; up >= level + 2; up -= 2) {
            let at = kept(up);
            let up_size = f32(1 << u32(at));
            let cc = floor(q.xy / up_size);
            let entry = entry_at(at, i32(cc.x), i32(cc.y));
            if (entry.y == PAST_MAP) {
                return light;
            }
            if (entry.y == NOT_BAKED) {
                continue;
            }
            let low = bitcast<i32>(entry.y << 16u) >> 16u;
            let count = i32(entry.y >> 16u);
            if (q.z > f32(low + count) * up_size + f32(1 << u32(level))) {
                t = max(leave2(o.xy, s.xy, cc, up_size), t) + 0.01;
                skipped = true;
                break;
            }
        }
        if (skipped) {
            continue;
        }
        // A region not baked yet holds nothing known to shade with: on past it whole, as the march does
        // [2026-11-08: tried level by level there, each step read up to eight levels and found none].
        let r = floor(q.xy / f32(1 << u32(REGION)));
        if (entry_at(REGION, i32(r.x), i32(r.y)).y == NOT_BAKED) {
            t = max(leave2(o.xy, s.xy, r, f32(1 << u32(REGION))), t) + 0.01;
            continue;
        }
        var look = blend(level, q);
        // A live level not made here yet: the bake's first, there wherever its region is.
        if (!look.known && level < header[0]) {
            level = header[0];
            look = blend(level, q);
        }
        let size = f32(1 << u32(level));
        // A step as long as the ray is wide (never under its cell): what it crosses dims it by how solid it is.
        let step = max(size, wide);
        // Only once it's climbed a whole cell of what it reads over where it started: a coarser level read where a
        // finer one isn't made yet has a soft edge reaching higher than the ray started over its own surface, and
        // shaded the surface itself [David's F12 of build 80: rows of dark triangles down a sunlit mountainside
        // while its tiles filled in, gone once they had].
        let climbed = q.z - p.z;
        if (look.known && climbed >= size) {
            // Through a medium's share, dimmed as the eye's rays are (build 98: darker the deeper under water).
            light *= pow(1.0 - min(look.solid, 0.999), step / size) * exp(-media[0].w * look.water * step);
            if (light < 0.02) {
                return 0.0;
            }
        }
        t += step;
    }
    return light;
}

// ── The start pass (the screen pyramid's first step, David 2026-11-06) ──
//
// One ray down the middle of each 8 × 8 tile of the screen, taken as a cone that holds the whole tile: it only
// skips (over cell columns' tops, the cone's width kept clear of them, and a margin for the blends), never
// blends, and stops at the first place the cone could touch anything. Every ray of the tile starts there: none of
// them can meet anything before it.

// How far the cone from `o` along `d` (spreading `spread` a block of distance) goes before it could touch.
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
        // A region not baked yet: from here (the march skips it itself, and the live level may be there).
        let rg = floor(p.xy / f32(1 << u32(REGION)));
        if (entry_at(REGION, i32(rg.x), i32(rg.y)).y == NOT_BAKED) {
            return t;
        }
        var advanced = false;
        let first_level = clamp(i32(ceil(log2(4.0 * r))), header[0], header[1]);
        for (var k = 0; k < 3; k++) {
            let level = min(first_level + k, header[1]);
            let size = f32(1 << u32(level));
            let c = floor(p.xy / size);
            // The cone has to sit inside the column.
            if (any(p.xy - r < c * size) || any(p.xy + r >= (c + 1.0) * size)) {
                continue;
            }
            let entry = entry_at(level, i32(c.x), i32(c.y));
            if (entry.y == NOT_BAKED) {
                continue;
            }
            var top = -1e9;
            if (entry.y != PAST_MAP) {
                let low = bitcast<i32>(entry.y << 16u) >> 16u;
                let count = i32(entry.y >> 16u);
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

// How much the layer below is worth where this layer's surface is `near` off, read from cells of `level` (0 to 1;
// 1: it's laid over this whole). Nothing while its pixels would be smaller than the cells they'd read (finer than
// the cells, they only magnify the blend into smudge: David's bands, "2 looks worse than 4, up close"); rising to all
// of it as they grow bigger, so the change of pixel size fades in rather than drawing a ring round the eye; never
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
    // The cells this pixel's surface was read from (not the finest that could be there: where a live level's tile
    // isn't made yet, a finer pixel over the coarser cells read instead only smudges, and cost the most
    // [2026-11-07: with the 8s and 16s not made, about 96 fps hovering against 125]).
    let cell = f32(1 << u32(level));
    if (cell <= 1.0) {
        // Blocks as blocks: a finer pixel always shows more of them (their edges; later their faces), fading out
        // towards the edge of the blocks' reach so the change of pixel size isn't a ring (by default over its last
        // quarter; where is David's to set, each layer its own).
        let begins = camera.pyramid.z * camera.layer.w;
        let ends = camera.pyramid.w * camera.layer.w;
        return clamp((ends - near) / max(ends - begins, 1e-3), 0.0, 1.0);
    }
    // Past them: in as the finer pixel grows bigger than the cells (by default from as big as them, over one
    // doubling of distance; where and how long are David's to set).
    let ratio = near * camera.view.z * camera.pyramid.y / cell;
    return clamp((ratio / camera.fade.x - 1.0) / max(exp2(camera.fade.y) - 1.0, 1e-3), 0.0, 1.0);
}

@compute @workgroup_size(8, 8)
fn march_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = vec2<u32>(camera.view.xy);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    // A refining layer marches only where the pixel above said it's worth something; elsewhere the screen shows
    // that one. How much it's worth is how much of it the screen shows over the layer above (its alpha).
    var shown = 1.0;
    // Its pixel in the layer above: by scale (halving, or 1.5's three to the 2 × 2's two).
    let parent = min(vec2<u32>((vec2<f32>(id.xy) + 0.5) * camera.pyramid.x), vec2<u32>(u32(max(camera.layer.x, 1.0)) - 1u, 65535u));
    if (camera.layer.x > 0.0) {
        let above = parent;
        let worth = worth_in[above.y * u32(camera.layer.x) + above.x] & 0xFFu;
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
    let rays = u32(camera.view.w);
    // Where its tile's rays start (from the start pass), or from the eye.
    var begin = 0.5;
    if (camera.tiles.z > 0.0) {
        let tile = vec2<u32>(vec2<f32>(id.xy) / camera.tiles.z);
        begin = starts[tile.y * u32(camera.tiles.x) + tile.x];
    }
    // A refining layer's rays start a little short of the nearest touch among their parent and its neighbours
    // (the parent's pixels are no wider than a block where the blocks are walked, so nothing a block big hides
    // between them; backed off two blocks and two of the parent's pixels, for grazing rays): the 1 × 1 layer's
    // walk then crosses a block or two, not all the air from the tile's start [2026-11-07, the
    // low slow flight uncapped: finest 1 × 1 about 165 fps against 104, and 2 × 2 about 210 against 172].
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
    var near = camera.reach.x;
    var touch = camera.reach.x;
    var steps = 0.0;
    var level = -1;
    var colour = vec3<f32>(0.0);
    var handed = NO_SKY;
    for (var r = 0u; r < rays; r++) {
        // One ray at the middle; four at the middles of the pixel's quarters.
        var offset = vec2<f32>(0.5, 0.5);
        if (rays > 1u) {
            offset = vec2<f32>(0.25 + 0.5 * f32(r & 1u), 0.25 + 0.5 * f32(r >> 1u));
        }
        let uv = (vec2<f32>(id.xy) + offset) / vec2<f32>(size);
        let x = uv.x * 2.0 - 1.0;
        let y = 1.0 - uv.y * 2.0;
        let d = normalize(camera.forward.xyz + x * camera.right.xyz + y * camera.up.xyz);
        let seen = wear_texture(march(camera.position.xyz, d, scatter(id.xy * 2u + vec2<u32>(r & 1u, r >> 1u)), begin));
        // Its shadow and sky light only where it shows (a finer layer laid over it whole hides it), by its own
        // worth [2026-11-08: the rays kept till the pixel's worth was known made each pixel's work heavy on the
        // card, about 2 to 3 ms with the sky's part added].
        let hidden = round(worth_of(seen.distance, seen.level) * 255.0) >= 255.0;
        // Its sky light, where it shows or the layer below may take it: its parent's, if that was for the same kind
        // of surface about as far off (sky light changes slowly: most finer pixels take it, and only edges and
        // silhouettes look for themselves) [2026-11-08: every pixel looking for itself, the blocks near the eye at
        // 2 × 2 cost about 7 ms low over the ground; the far cells about 1].
        var open = 1.0;
        if (camera.sky.x > 0.5 && seen.wide > 0.0 && (!hidden || camera.layer.y > 0.5)) {
            let kind = surface_kind(seen);
            var have = false;
            if (camera.layer.x > 0.0) {
                let above = worth_in[parent.y * u32(camera.layer.x) + parent.x] >> 8u;
                let parent_touch = textureLoad(far_above, vec2<i32>(parent), 0).x;
                if (above != NO_SKY && (above >> 16u) == kind && abs(seen.touched - parent_touch) <= 0.1 * seen.touched + 2.0) {
                    open = f32(above & 0xFFFFu) / 65535.0;
                    have = true;
                }
            }
            if (!have) {
                open = sky_seen(seen.at, seen.wide, seen.facing, seen.on_block);
            }
            if (r == 0u) {
                handed = sky_word(kind, open);
            }
        }
        // Through the fog to where it first met anything (the sky behind nothing isn't fogged: it's the sky). Only
        // what it met is fogged, as much of it as it met; the sky it went on to past a soft edge stays the sky
        // [David on 98: "the way the mountains make a brighter outline against the sky where their haze ends": a ray
        // that brushed a far peak's soft edge and went on had all of it fogged to the horizon's pale colour, the sky
        // it saw too, paler than the sky up there and than the hazed peak: a pale rim round every far skyline].
        var here = seen.colour;
        if (!hidden) {
            here = shaded(seen, open);
        }
        // The placed light on it, which no shadow or sky takes away.
        here += seen.glow;
        let met = select(seen.distance, seen.touched, seen.distance >= camera.reach.x);
        if (met < camera.reach.x) {
            let behind = seen.sky_share * sky(d);
            let covered = max(1.0 - seen.sky_share, 1e-4);
            here = behind + fogged((here - behind) / covered, d, met) * covered;
        }
        colour += here;
        near = min(near, seen.distance);
        touch = min(touch, seen.touched);
        steps += seen.steps;
        level = max(level, seen.level);
    }
    // How much is the layer below worth here? Nothing while its pixels would be smaller than the cells they'd read
    // (finer than the cells, they only magnify the blend into smudge: David's bands, "2 looks worse than 4, up
    // close"); rising to all of it over the next doubling of distance, so the change of pixel size fades in rather
    // than drawing a ring round the eye; never for sky.
    if (camera.layer.y > 0.5) {
        worth_out[id.y * size.x + id.x] = u32(round(worth_of(near, level) * 255.0)) | (handed << 8u);
    }
    colour /= f32(rays);
    if (camera.reach.z > 1.5 && level >= 0) {
        let tints = array<vec3<f32>, 6>(vec3<f32>(1.0, 0.3, 0.3), vec3<f32>(0.3, 1.0, 0.3), vec3<f32>(0.3, 0.4, 1.0), vec3<f32>(1.0, 1.0, 0.3), vec3<f32>(1.0, 0.3, 1.0), vec3<f32>(0.3, 1.0, 1.0));
        colour = colour * 0.4 + tints[level % 6] * 0.35;
    } else if (camera.reach.z > 0.5 && camera.reach.z < 1.5) {
        let k = clamp(steps / f32(rays) / 256.0, 0.0, 1.0);
        colour = select(mix(vec3<f32>(0.0, 0.8, 0.2), vec3<f32>(1.0, 0.0, 0.0), (k - 0.5) * 2.0), mix(vec3<f32>(0.0, 0.0, 0.4), vec3<f32>(0.0, 0.8, 0.2), k * 2.0), k < 0.5);
        colour = colour * colour;
    }
    // Kept with its alpha (premultiplied), so the screen can blend it over the layer above.
    textureStore(picture, vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour, vec3<f32>(0.0), vec3<f32>(1.0))) * shown, shown));
    // Kept for the layer below to start its rays from, and the skyline's test: where it first touched anything
    // [2026-11-08: the half-covered distance was kept, and a finer layer marched everywhere over cells of 32 began
    // inside their soft edge, gathered too little of it, and its shadow rays shaded their own ground].
    textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(touch, 0.0, 0.0, 0.0));
}

// ── The march split from its light (the dyno, 2026-11-09) ──
//
// The march is one big shader, and the card keeps as few of its pixels in flight as the room (registers) its
// biggest path needs allows, used or not [the dyno's second step: the build with sky light kept's code in it, off,
// was 8% slower]. So, with a ray a pixel, it's two passes a layer: `march_hit` marches each pixel's ray, wears its
// texture, and keeps what it met (`hits`: its colour and the sun's and the sky's parts of it, where, how far, which
// way it faces); then `light_hit` lights it as `march_pass` did after its march (the sun's shadow, sky light,
// placed light, fog). Each runs with the room its own work needs. `march_pass` stays for four rays a pixel, and for
// measuring against. Off unless chosen (`Eye::split`, `INCUS_SIGHT_SPLIT`): flown in one pass, it made the worst
// frames 12% better and the median 13% worse (2026-11-09).

// What each pixel's ray met: three words of four a pixel (`march_hit` writes, `light_hit` reads).
@group(0) @binding(19) var<storage, read_write> hits: array<vec4<u32>>;

// A pixel the layer above didn't find worth marching (`march_hit` drew it).
const HIT_NONE: u32 = 4096u;

// Every binding the layers have named, so both passes have the same ones whatever each reaches, and one layer's
// bindings serve both.
fn name_the_bindings() {
    _ = camera.position;
    _ = header[0];
    _ = baked[0];
    _ = live[0];
    _ = pool[0];
    _ = textureDimensions(picture);
    _ = textureDimensions(how_far);
    _ = starts[0];
    _ = worth_in[0];
    _ = worth_out[0];
    _ = textureDimensions(far_above);
    _ = textureSampleLevel(block_textures, block_sampler, vec2<f32>(0.0), 0, 0.0);
    _ = block_table[0];
    _ = pieces[0];
    _ = lights[0];
    _ = media[0];
    _ = hits[0];
}

// The way a block face looks, from `face_of`'s number.
fn face_way(face: u32) -> vec3<f32> {
    let ways = array<vec3<f32>, 6>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(-1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, -1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, -1.0));
    return ways[min(face, 5u)];
}

// A pixel's ray's way (one ray, through its middle), as `march_pass` has it.
fn pixel_way(id: vec2<u32>, size: vec2<u32>) -> vec3<f32> {
    let uv = (vec2<f32>(id) + 0.5) / vec2<f32>(size);
    let x = uv.x * 2.0 - 1.0;
    let y = 1.0 - uv.y * 2.0;
    return normalize(camera.forward.xyz + x * camera.right.xyz + y * camera.up.xyz);
}

// Its pixel in the layer above: by scale (halving, or 1.5's three to the 2 × 2's two).
fn parent_of(id: vec2<u32>) -> vec2<u32> {
    return min(vec2<u32>((vec2<f32>(id) + 0.5) * camera.pyramid.x), vec2<u32>(u32(max(camera.layer.x, 1.0)) - 1u, 65535u));
}

@compute @workgroup_size(8, 8)
fn march_hit(@builtin(global_invocation_id) id: vec3<u32>) {
    name_the_bindings();
    let size = vec2<u32>(camera.view.xy);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let at = (id.y * size.x + id.x) * 3u;
    // A refining layer marches only where the pixel above said it's worth something (as `march_pass`).
    let parent = parent_of(id.xy);
    if (camera.layer.x > 0.0) {
        let worth = worth_in[parent.y * u32(camera.layer.x) + parent.x] & 0xFFu;
        if (worth == 0u) {
            textureStore(picture, vec2<i32>(id.xy), vec4<f32>(0.0));
            textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(camera.reach.x, 0.0, 0.0, 0.0));
            if (camera.layer.y > 0.5) {
                worth_out[id.y * size.x + id.x] = NO_SKY << 8u;
            }
            hits[at + 2u] = vec4<u32>(HIT_NONE, 0u, 0u, 0u);
            return;
        }
    }
    // Where its tile's rays start, or short of its parent's nearest touch (as `march_pass`).
    var begin = 0.5;
    if (camera.tiles.z > 0.0) {
        let tile = vec2<u32>(vec2<f32>(id.xy) / camera.tiles.z);
        begin = starts[tile.y * u32(camera.tiles.x) + tile.x];
    }
    if (camera.layer.x > 0.0 && camera.follow.x > 0.5) {
        let above = vec2<i32>(parent);
        let edge = vec2<i32>(textureDimensions(far_above)) - 1;
        var nearest = camera.reach.x;
        for (var k = 0; k < 9; k++) {
            let near_at = clamp(above + vec2<i32>(k % 3 - 1, k / 3 - 1), vec2<i32>(0), edge);
            nearest = min(nearest, textureLoad(far_above, near_at, 0).x);
        }
        begin = max(begin, nearest - 2.0 - nearest * camera.view.z * 4.0);
    }
    let d = pixel_way(id.xy, size);
    let seen = wear_texture(march(camera.position.xyz, d, scatter(id.xy * 2u), begin));
    // The layer below's worth (the light pass puts its sky light over it), and where it first touched anything.
    if (camera.layer.y > 0.5) {
        worth_out[id.y * size.x + id.x] = u32(round(worth_of(seen.distance, seen.level) * 255.0));
    }
    textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(seen.touched, 0.0, 0.0, 0.0));
    // The placed light goes in with its colour: the sun's and the sky's parts the light takes away aren't it.
    let colour = seen.colour + seen.glow;
    hits[at] = vec4<u32>(pack2x16float(colour.rg), pack2x16float(vec2<f32>(colour.b, seen.sky_share)), pack2x16float(seen.sunny.rg), pack2x16float(vec2<f32>(seen.sunny.b, seen.skyish.r)));
    hits[at + 1u] = vec4<u32>(pack2x16float(seen.skyish.gb), bitcast<u32>(seen.distance), bitcast<u32>(seen.touched), bitcast<u32>(dot(seen.at - camera.position.xyz, d)));
    let kept = (u32(seen.level + 1) & 0xFFu) | (select(0u, 256u, seen.on_block)) | (face_of(seen.facing) << 9u) | (min(u32(seen.steps), 65535u) << 16u);
    hits[at + 2u] = vec4<u32>(kept, 0u, 0u, 0u);
}

@compute @workgroup_size(8, 8)
fn light_hit(@builtin(global_invocation_id) id: vec3<u32>) {
    name_the_bindings();
    let size = vec2<u32>(camera.view.xy);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let at = (id.y * size.x + id.x) * 3u;
    let r2 = hits[at + 2u];
    if ((r2.x & HIT_NONE) != 0u) {
        return;
    }
    var shown = 1.0;
    let parent = parent_of(id.xy);
    if (camera.layer.x > 0.0) {
        shown = f32(worth_in[parent.y * u32(camera.layer.x) + parent.x] & 0xFFu) / 255.0;
    }
    // What the ray met, as the march left it.
    let r0 = hits[at];
    let r1 = hits[at + 1u];
    let d = pixel_way(id.xy, size);
    var seen: Seen;
    let colour_b = unpack2x16float(r0.y);
    let sunny_b = unpack2x16float(r0.w);
    seen.colour = vec3<f32>(unpack2x16float(r0.x), colour_b.x);
    seen.sky_share = colour_b.y;
    seen.sunny = vec3<f32>(unpack2x16float(r0.z), sunny_b.x);
    seen.skyish = vec3<f32>(sunny_b.y, unpack2x16float(r1.x));
    seen.distance = bitcast<f32>(r1.y);
    seen.touched = bitcast<f32>(r1.z);
    seen.at = camera.position.xyz + d * bitcast<f32>(r1.w);
    seen.level = i32(r2.x & 0xFFu) - 1;
    seen.on_block = (r2.x & 256u) != 0u;
    seen.steps = f32(r2.x >> 16u);
    seen.facing = select(vec3<f32>(0.0), face_way((r2.x >> 9u) & 7u), seen.on_block);
    // How wide its shadow ray starts, as the march works it out.
    if (seen.on_block) {
        seen.wide = 1.0;
    } else if (seen.distance < camera.reach.x) {
        let read = select(1.0, f32(1 << u32(max(seen.level, 0))), seen.level >= 0);
        seen.wide = max(seen.distance * camera.view.z, read);
    }
    // From here as `march_pass` lights its ray.
    var handed = NO_SKY;
    let hidden = round(worth_of(seen.distance, seen.level) * 255.0) >= 255.0;
    var open = 1.0;
    if (camera.sky.x > 0.5 && seen.wide > 0.0 && (!hidden || camera.layer.y > 0.5)) {
        let kind = surface_kind(seen);
        var have = false;
        if (camera.layer.x > 0.0) {
            let above = worth_in[parent.y * u32(camera.layer.x) + parent.x] >> 8u;
            let parent_touch = textureLoad(far_above, vec2<i32>(parent), 0).x;
            if (above != NO_SKY && (above >> 16u) == kind && abs(seen.touched - parent_touch) <= 0.1 * seen.touched + 2.0) {
                open = f32(above & 0xFFFFu) / 65535.0;
                have = true;
            }
        }
        if (!have) {
            open = sky_seen(seen.at, seen.wide, seen.facing, seen.on_block);
        }
        handed = sky_word(kind, open);
    }
    var colour = seen.colour;
    if (!hidden) {
        colour = shaded(seen, open);
    }
    let met = select(seen.distance, seen.touched, seen.distance >= camera.reach.x);
    if (met < camera.reach.x) {
        let behind = seen.sky_share * sky(d);
        let covered = max(1.0 - seen.sky_share, 1e-4);
        colour = behind + fogged((colour - behind) / covered, d, met) * covered;
    }
    if (camera.layer.y > 0.5) {
        let k = id.y * size.x + id.x;
        worth_out[k] = (worth_out[k] & 0xFFu) | (handed << 8u);
    }
    if (camera.reach.z > 1.5 && seen.level >= 0) {
        let tints = array<vec3<f32>, 6>(vec3<f32>(1.0, 0.3, 0.3), vec3<f32>(0.3, 1.0, 0.3), vec3<f32>(0.3, 0.4, 1.0), vec3<f32>(1.0, 1.0, 0.3), vec3<f32>(1.0, 0.3, 1.0), vec3<f32>(0.3, 1.0, 1.0));
        colour = colour * 0.4 + tints[seen.level % 6] * 0.35;
    } else if (camera.reach.z > 0.5 && camera.reach.z < 1.5) {
        let k = clamp(seen.steps / 256.0, 0.0, 1.0);
        colour = select(mix(vec3<f32>(0.0, 0.8, 0.2), vec3<f32>(1.0, 0.0, 0.0), (k - 0.5) * 2.0), mix(vec3<f32>(0.0, 0.0, 0.4), vec3<f32>(0.0, 0.8, 0.2), k * 2.0), k < 0.5);
        colour = colour * colour;
    }
    textureStore(picture, vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour, vec3<f32>(0.0), vec3<f32>(1.0))) * shown, shown));
}

// ── The march in stages (the dyno, 2026-11-09; David: "lets atomize the megakernal") ──
//
// The split cut differently: `march_hit` marches each layer's pixels; `listed_staged` lists those it met something
// for (and gathers the faces near the eye, with light per face, below); `face_light_staged` lights the faces once
// each; `shade_staged` shades each listed pixel (its sky light, the sun reaching it, its light), from its face where
// it has one. [2026-11-09: built as a pass each for the shadow, sky and light, then folded into one: apart, they read
// and wrote each pixel's record three times over, for little work on the lighter views.] The split stays as it was,
// to measure against.

// Each layer's pixels its march met something for (not the ones the layer above didn't find worth marching), listed
// by the pass after its march (`listed_staged`), so the stages after it run over them only, not over the whole layer
// [2026-11-09, each stage timed: on the lightest views the near layer's passes, launched over its million pixels
// to find a few to work on, cost a fixed half millisecond]. Its count as the card's indirect dispatch reads it:
// groups of 64 across, 1, 1, then the count.
@group(0) @binding(20) var<storage, read_write> live_list: array<u32>;
@group(0) @binding(21) var<storage, read_write> live_count: array<atomic<u32>, 4>;

// The list, so every pass after the march has the same bindings. Not its count: a pass launched by the count can't
// have it open as well.
fn name_the_lists() {
    _ = live_list[0];
}

// The pixel the `k`th thread of a listed pass works on. The last group's threads past the list's end read what an
// earlier frame left there: a pixel with nothing to do this frame (it leaves at once), or one listed this frame as
// well, worked twice the same way (every listed pass writes the same from the same record), so no harm.
fn listed_pixel(group: vec3<u32>, index: u32, size: vec2<u32>) -> vec2<u32> {
    let at = live_list[group.x * 64u + index];
    return vec2<u32>(at % size.x, at / size.x);
}

// A group's pixels it marched, gathered (counted in the group, a place in the list taken once for them all).
var<workgroup> group_live: atomic<u32>;
var<workgroup> group_first: u32;

// The stages' list of a layer's live pixels, after its march; with light per face, each live pixel showing a block's
// face near the eye finds its face in the table (or puts it in), its slot kept in the record (the fourth word's
// fourth: slot and 1, 0 for none) [2026-11-09: once a pass of its own, folded in here].
@compute @workgroup_size(8, 8)
fn listed_staged(@builtin(global_invocation_id) id: vec3<u32>, @builtin(local_invocation_index) index: u32) {
    name_the_bindings();
    name_the_lists();
    name_the_faces();
    if (index == 0u) {
        atomicStore(&group_live, 0u);
    }
    workgroupBarrier();
    let size = vec2<u32>(camera.view.xy);
    var live = false;
    if (id.x < size.x && id.y < size.y) {
        live = (hits[(id.y * size.x + id.x) * 3u + 2u].x & HIT_NONE) == 0u;
    }
    var mine = 0u;
    if (live) {
        mine = atomicAdd(&group_live, 1u);
    }
    workgroupBarrier();
    if (index == 0u) {
        let n = atomicLoad(&group_live);
        if (n > 0u) {
            group_first = atomicAdd(&live_count[3], n);
            atomicMax(&live_count[0], (group_first + n + 63u) / 64u);
        }
    }
    workgroupBarrier();
    if (live) {
        live_list[group_first + mine] = id.y * size.x + id.x;
        let at = (id.y * size.x + id.x) * 3u;
        let r2 = hits[at + 2u];
        if (atomicLoad(&face_count[4]) != 0u && (r2.x & 256u) != 0u) {
            let d = pixel_way(id.xy, size);
            let block = vec3<i32>(floor(camera.position.xyz + d * (bitcast<f32>(hits[at + 1u].w) + 0.01)));
            let entry = entry_at(0, block.x, block.y);
            if (entry.y != NOT_BAKED && block_in_band(entry.x, entry.y, block.z) != PIECES_MARK) {
                let key = face_key(block, (r2.x >> 9u) & 7u);
                if (key != 0u) {
                    hits[at + 2u] = vec4<u32>(r2.x, r2.y, r2.z, face_slot(key));
                }
            }
        }
    }
}

// What pixel `id`'s ray met, as `march_hit` left it (its record's three words), with how wide its shadow ray starts.
fn hit_of(id: vec2<u32>, size: vec2<u32>, r0: vec4<u32>, r1: vec4<u32>, r2: vec4<u32>) -> Seen {
    let d = pixel_way(id, size);
    var seen: Seen;
    let colour_b = unpack2x16float(r0.y);
    let sunny_b = unpack2x16float(r0.w);
    seen.colour = vec3<f32>(unpack2x16float(r0.x), colour_b.x);
    seen.sky_share = colour_b.y;
    seen.sunny = vec3<f32>(unpack2x16float(r0.z), sunny_b.x);
    seen.skyish = vec3<f32>(sunny_b.y, unpack2x16float(r1.x));
    seen.distance = bitcast<f32>(r1.y);
    seen.touched = bitcast<f32>(r1.z);
    seen.at = camera.position.xyz + d * bitcast<f32>(r1.w);
    seen.level = i32(r2.x & 0xFFu) - 1;
    seen.on_block = (r2.x & 256u) != 0u;
    seen.steps = f32(r2.x >> 16u);
    seen.facing = select(vec3<f32>(0.0), face_way((r2.x >> 9u) & 7u), seen.on_block);
    if (seen.on_block) {
        seen.wide = 1.0;
    } else if (seen.distance < camera.reach.x) {
        let read = select(1.0, f32(1 << u32(max(seen.level, 0))), seen.level >= 0);
        seen.wide = max(seen.distance * camera.view.z, read);
    }
    return seen;
}

// A ray's colour with its shadow and its sky light, as `shaded`, the sun that reaches it (`sun`, 0 to 1) given.
fn shaded_by(seen: Seen, open: f32, sun: f32) -> vec3<f32> {
    if (seen.wide <= 0.0) {
        return seen.colour;
    }
    var colour = seen.colour;
    if (camera.follow.y > 0.5) {
        colour -= seen.sunny * (1.0 - sun);
    }
    let sky = select(1.0, mix(1.0, open, camera.sky.y), camera.sky.x > 0.5);
    let tint = mix(vec3<f32>(1.0), SKY_TINT, camera.sky.z);
    return colour + seen.skyish * (sky * tint - 1.0);
}

// ── Light per face near the eye (the stages, 2026-11-09; David: "blocky and quantized lighting is good all around") ──
//
// A block face near the eye covers many pixels, and each worked out its own shadow ray and five sky cones, all
// coming to about the same answer. With this on (the Render page's switch, in stages), each face the near walk shows
// is lit once, at its middle, and every pixel showing it takes that: the frame's faces gathered in a table on the
// card (by `listed_staged`), lit one a thread (`face_light_staged`), read by `shade_staged`. A face is its block
// (from the eye's block, within 127 every way: the near walk reaches some 64) and the way it looks, in one word.
// Pieces' faces and everything off the blocks stay a pixel at a time.

// The frame's faces: each slot's key (0: empty), and its light (the sun reaching it and the sky it sees, half floats).
@group(1) @binding(0) var<storage, read_write> face_keys: array<atomic<u32>>;
@group(1) @binding(1) var<storage, read_write> face_values: array<u32>;
// The slots taken this frame, listed (so the light per face lights only them), and how many, as the card's indirect
// dispatch reads it: groups of 64 across, 1, 1, then the count; then whether light per face is on this frame (1).
@group(1) @binding(2) var<storage, read_write> face_list: array<u32>;
@group(1) @binding(3) var<storage, read_write> face_count: array<atomic<u32>, 8>;

// The table's slots (a power of two), and how far along it a face is looked for from its first slot.
const FACE_SLOTS: u32 = 262144u;
const FACE_PROBES: u32 = 16u;

// Both the faces' bindings, so every pass that has them has the same ones.
fn name_the_faces() {
    _ = atomicLoad(&face_keys[0]);
    _ = face_values[0];
    _ = face_list[0];
    _ = atomicLoad(&face_count[0]);
}

// A face's key: its block from the eye's (each way +128, a byte), the way it looks, and a high bit (never 0); 0 if
// it's too far off to fit.
fn face_key(block: vec3<i32>, face: u32) -> u32 {
    let from_eye = block - vec3<i32>(floor(camera.position.xyz));
    if (any(abs(from_eye) > vec3<i32>(127))) {
        return 0u;
    }
    let b = vec3<u32>(from_eye + vec3<i32>(128));
    return b.x | (b.y << 8u) | (b.z << 16u) | (face << 24u) | 0x80000000u;
}

// A face's slot, found or taken (its slot and 1; 0 if the table is too full there).
fn face_slot(key: u32) -> u32 {
    let first = (key * 2654435761u) >> 14u;
    for (var k = 0u; k < FACE_PROBES; k++) {
        let slot = (first + k) & (FACE_SLOTS - 1u);
        // Read first: a big face's pixels all find it there, and only the first to come takes it [2026-11-09: every
        // pixel taking the slot in turn to find it taken, the faces pass cost 0.28 ms on the lightest views].
        let there = atomicLoad(&face_keys[slot]);
        if (there == key) {
            return slot + 1u;
        }
        if (there != 0u) {
            continue;
        }
        let found = atomicCompareExchangeWeak(&face_keys[slot], 0u, key);
        if (found.exchanged) {
            // A face new this frame: listed, for the light per face.
            let k = atomicAdd(&face_count[3], 1u);
            face_list[k] = slot;
            atomicMax(&face_count[0], (k + 64u) / 64u);
            return slot + 1u;
        }
        if (found.old_value == key) {
            return slot + 1u;
        }
    }
    return 0u;
}

// The stages' light per face: each face in the table lit once, at its middle: the sun reaching it (its shadow, as a
// pixel's from a block's face) and the sky it sees (its five cones), as wide as a block.
@compute @workgroup_size(64)
fn face_light_staged(@builtin(global_invocation_id) id: vec3<u32>) {
    name_the_bindings();
    // Not the faces' count: it's launched by it. Past the list's end, an earlier frame's slots: empty this frame (it
    // leaves), or listed this frame too, lit twice the same way.
    _ = face_values[0];
    let slot = face_list[id.x] & (FACE_SLOTS - 1u);
    let key = atomicLoad(&face_keys[slot]);
    if (key == 0u) {
        return;
    }
    let from_eye = vec3<i32>(i32(key & 0xFFu), i32((key >> 8u) & 0xFFu), i32((key >> 16u) & 0xFFu)) - vec3<i32>(128);
    let block = vec3<i32>(floor(camera.position.xyz)) + from_eye;
    let facing = face_way((key >> 24u) & 7u);
    let middle = vec3<f32>(block) + vec3<f32>(0.5) + facing * 0.5;
    var sun = 1.0;
    if (camera.follow.y > 0.5) {
        sun = sunlight(middle, 1.0, facing, true);
    }
    var open = 1.0;
    if (camera.sky.x > 0.5) {
        open = sky_seen(middle, 1.0, facing, true);
    }
    face_values[slot] = pack2x16float(vec2<f32>(sun, open));
}

// ── Far lights' glows (build 100; David: "braziers in a cave at night from a distance were just pixels of green that
// popped in and out as they twinkled") ──
//
// Past the loaded chunks a light is drawn as a dot (`models.wgsl`), seen or not as a line walked to it through the
// cells says: a hill or a cave's rock between hides it. Not by the picture's depth, nor by a ray marched as the
// picture's are: far off, those meet the soft edge of the cells the ground is read from (blended smoothly, a third
// solid ground cell is a faint haze a whole cell over the ground), and a line skimming open ground to a flame standing
// on it adds that haze up and counts as covered. The line instead takes each cell as it's kept, as wide as the cells
// read at its distance, and only a cell mostly solid stops it. A light in the open (the sky over it, as a sky cone
// straight up says) has its line aimed a cell over its flame, out of the cell of ground it stands on; one under rock (a
// cave) is aimed at its flame. Seen if nothing stops the line short of where it's aimed by more than a cell.
@compute @workgroup_size(64)
fn glow_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    // The march's bindings named, so this pass has the same ones whatever it reaches (the layers', but for their
    // pictures and the pyramid's).
    _ = header[0];
    _ = baked[0];
    _ = live[0];
    _ = pool[0];
    _ = pieces[0];
    _ = lights[0];
    _ = media[0];
    _ = block_table[0];
    _ = textureSampleLevel(block_textures, block_sampler, vec2<f32>(0.0), 0, 0.0);
    let i = id.x;
    if (i >= arrayLength(&glows)) {
        return;
    }
    let glow = glows[i];
    if (glow.w < 0.0) {
        return;
    }
    let eye = camera.position.xyz;
    let level = level_at(length(glow.xyz - eye));
    let cell = f32(1 << u32(max(level, 0)));
    let up = vec3<f32>(0.0, 0.0, 1.0);
    let open = cone(glow.xyz, cell, up, false, up, camera.sky.w, 0.0, 0.0, SKY_STEPS);
    let r = glow.xyz + up * cell * smoothstep(0.3, 0.7, open) - eye;
    let far = length(r);
    if (far < 1.0) {
        glows[i].w = 0.0;
        return;
    }
    let d = r / far;
    let end = far - cell - 1.5;
    var t = 0.5;
    var seen = 1.0;
    for (var k = 0; k < 4096 && t < end; k++) {
        let p = eye + d * t;
        var l = level_at(t);
        var look = cell_at(l, vec3<i32>(floor(p / f32(1 << u32(l)))));
        // Not there yet (past a live window's reach): the next coarser that is.
        for (var up_one = 0; up_one < 4 && !look.known && l < header[1]; up_one++) {
            l = l + 1;
            look = cell_at(l, vec3<i32>(floor(p / f32(1 << u32(l)))));
        }
        if (look.known && look.solid > 0.5) {
            seen = 0.0;
            break;
        }
        t += max(f32(1 << u32(l)) * 0.5, 0.5);
    }
    glows[i].w = seen;
}

// The stages' shading, one pass a layer after the march [2026-11-09: the shadow, sky and light passes apart read and
// wrote each pixel's record three times over, for little work on the lighter views]: its sky light (its face's, its
// parent's where that's the same surface about as far off, or its five cones; handed down to the layer below), the
// sun reaching it (its face's, or its walk toward the sun), then its light, placed light and fog, as `light_hit`.
@compute @workgroup_size(8, 8)
fn shade_staged(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) index: u32) {
    name_the_bindings();
    name_the_lists();
    name_the_faces();
    let size = vec2<u32>(camera.view.xy);
    let id = listed_pixel(group, index, size);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let at = (id.y * size.x + id.x) * 3u;
    let r2 = hits[at + 2u];
    if ((r2.x & HIT_NONE) != 0u) {
        return;
    }
    var shown = 1.0;
    let parent = parent_of(id.xy);
    if (camera.layer.x > 0.0) {
        shown = f32(worth_in[parent.y * u32(camera.layer.x) + parent.x] & 0xFFu) / 255.0;
    }
    let d = pixel_way(id.xy, size);
    let seen = hit_of(id.xy, size, hits[at], hits[at + 1u], r2);
    let hidden = round(worth_of(seen.distance, seen.level) * 255.0) >= 255.0;
    // Its face's slot and 1, with light per face on (0: none).
    let face = select(0u, r2.w, atomicLoad(&face_count[4]) != 0u);
    var handed = NO_SKY;
    var open = 1.0;
    if (camera.sky.x > 0.5 && seen.wide > 0.0 && (!hidden || camera.layer.y > 0.5)) {
        let kind = surface_kind(seen);
        var have = false;
        if (face > 0u) {
            open = unpack2x16float(face_values[face - 1u]).y;
            have = true;
        }
        if (!have && camera.layer.x > 0.0) {
            let above = worth_in[parent.y * u32(camera.layer.x) + parent.x] >> 8u;
            let parent_touch = textureLoad(far_above, vec2<i32>(parent), 0).x;
            if (above != NO_SKY && (above >> 16u) == kind && abs(seen.touched - parent_touch) <= 0.1 * seen.touched + 2.0) {
                open = f32(above & 0xFFFFu) / 65535.0;
                have = true;
            }
        }
        if (!have) {
            open = sky_seen(seen.at, seen.wide, seen.facing, seen.on_block);
        }
        handed = sky_word(kind, open);
    }
    var sun = 1.0;
    if (!hidden && seen.wide > 0.0 && camera.follow.y > 0.5) {
        if (face > 0u) {
            sun = unpack2x16float(face_values[face - 1u]).x;
        } else {
            sun = sunlight(seen.at, seen.wide, seen.facing, seen.on_block);
        }
    }
    var colour = seen.colour;
    if (!hidden) {
        colour = shaded_by(seen, open, sun);
    }
    let met = select(seen.distance, seen.touched, seen.distance >= camera.reach.x);
    if (met < camera.reach.x) {
        let behind = seen.sky_share * sky(d);
        let covered = max(1.0 - seen.sky_share, 1e-4);
        colour = behind + fogged((colour - behind) / covered, d, met) * covered;
    }
    if (camera.layer.y > 0.5) {
        let k = id.y * size.x + id.x;
        worth_out[k] = (worth_out[k] & 0xFFu) | (handed << 8u);
    }
    if (camera.reach.z > 1.5 && seen.level >= 0) {
        let tints = array<vec3<f32>, 6>(vec3<f32>(1.0, 0.3, 0.3), vec3<f32>(0.3, 1.0, 0.3), vec3<f32>(0.3, 0.4, 1.0), vec3<f32>(1.0, 1.0, 0.3), vec3<f32>(1.0, 0.3, 1.0), vec3<f32>(0.3, 1.0, 1.0));
        colour = colour * 0.4 + tints[seen.level % 6] * 0.35;
    } else if (camera.reach.z > 0.5 && camera.reach.z < 1.5) {
        let k = clamp(seen.steps / 256.0, 0.0, 1.0);
        colour = select(mix(vec3<f32>(0.0, 0.8, 0.2), vec3<f32>(1.0, 0.0, 0.0), (k - 0.5) * 2.0), mix(vec3<f32>(0.0, 0.0, 0.4), vec3<f32>(0.0, 0.8, 0.2), k * 2.0), k < 0.5);
        colour = colour * colour;
    }
    textureStore(picture, vec2<i32>(id.xy), vec4<f32>(to_srgb(clamp(colour, vec3<f32>(0.0), vec3<f32>(1.0))) * shown, shown));
}

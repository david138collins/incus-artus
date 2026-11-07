// ── Sight's march (the regroup, phase 3.2): the coarse layer, a ray (or a few) a coarse pixel through the store ──
//
// The store (`card_store.rs`): the live level round the camera (cells of 16), the baked levels from 32 up
// over the whole map. A ray reads the level whose cells are about its pixel's size at that distance (never
// finer than the live level, which is the finest there is in 3.2), skips space a coarser cell says is empty,
// and elsewhere takes half a cell at a time, reading a smooth blend of the eight cells round it (no threshold:
// a part-solid cell covers part of the pixel, gathered front to back; colour and facing weighted by how much
// surface each cell shows), lit by the blend's average facing under a noon sun. The sky behind.

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
    // the rule (0 refine everywhere, 1 only where it's earned); w how far the live level reaches (blocks).
    layer: vec4<f32>,
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
// shown whole), and this layer's word for the layer below.
@group(0) @binding(8) var<storage, read> worth_in: array<u32>;
@group(0) @binding(9) var<storage, read_write> worth_out: array<u32>;

const NOT_BAKED: u32 = 0xFFFF0000u;
// Past the map: nothing at all, not even the world's floor.
const PAST_MAP: u32 = 0xFFFE0000u;
const WORLD_HEIGHT: f32 = 1024.0;
// The bake's regions are its cell columns of 1,024 (`bake::REGION`).
const REGION: i32 = 10;

// A cell as the march reads it.
struct Look {
    known: bool,
    solid: f32,
    water: f32,
    colour: vec3<f32>,
    facing: vec3<f32>,
    // How much surface it shows (the weight of its colour and facing).
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
    look.facing = vec3<f32>(f32(bitcast<i32>(w.x << 8u) >> 24u), f32(bitcast<i32>(w.x) >> 24u), f32(bitcast<i32>(w.y << 24u) >> 24u)) / 127.0;
    look.colour = vec3<f32>(f32((w.z >> 20u) & 1023u), f32((w.z >> 10u) & 1023u), f32(w.z & 1023u)) / 1023.0;
    look.surface = min(f32(w.w >> 16u) / 256.0, 4.0);
    return look;
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

// The cell at `c` (cell indices) of `level`: from the live window, or the baked levels; `known` false if it
// isn't there yet.
fn cell_at(level: i32, c: vec3<i32>) -> Look {
    if (level == header[2]) {
        let side = header[3];
        let slot = live[(((c.y % side) + side) % side) * side + (((c.x % side) + side) % side)];
        if (bitcast<i32>(slot.z) != c.x || bitcast<i32>(slot.w) != c.y) {
            return empty_look(false);
        }
        return from_band(slot.x, slot.y, c.z);
    }
    let k = level - header[0];
    let base = 4 + 4 * k;
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
    if (level == header[2]) {
        let side = header[3];
        let slot = live[(((cy % side) + side) % side) * side + (((cx % side) + side) % side)];
        if (bitcast<i32>(slot.z) != cx || bitcast<i32>(slot.w) != cy) {
            return vec2<u32>(0u, NOT_BAKED);
        }
        return slot.xy;
    }
    let k = level - header[0];
    let base = 4 + 4 * k;
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
            out.facing += shown * look.facing;
            weight += shown;
        }
    }
    if (weight > 0.0) {
        out.colour /= weight;
        out.facing /= weight;
        out.surface = weight;
    } else {
        // Inside, with no surface to show: rock's grey.
        out.colour = vec3<f32>(0.25, 0.24, 0.22);
    }
    return out;
}

// Where in a level's stretch (0 its start, 1 where the next takes over) the blend with the next begins.
const BLEND_FROM: f32 = 0.6;

// Two looks mixed: `w` of the second.
fn mix_looks(a: Look, b: Look, w: f32) -> Look {
    let k = clamp(w, 0.0, 1.0);
    var out = a;
    out.solid = mix(a.solid, b.solid, k);
    out.water = mix(a.water, b.water, k);
    out.colour = mix(a.colour, b.colour, k);
    out.facing = mix(a.facing, b.facing, k);
    out.surface = mix(a.surface, b.surface, k);
    return out;
}

fn sky(d: vec3<f32>) -> vec3<f32> {
    let up = clamp(d.z, -0.2, 1.0);
    let horizon = vec3<f32>(0.62, 0.72, 0.86);
    let zenith = vec3<f32>(0.16, 0.32, 0.66);
    return mix(horizon, zenith, pow(max(up, 0.0), 0.6));
}

// The level a ray reads at distance `t`: its pixel's size there, in whole powers of two, never finer than
// the finest level there is.
fn level_at(t: f32) -> i32 {
    let footprint = max(t * camera.view.z, 1.0);
    let level = i32(floor(log2(footprint))) + i32(camera.reach.y);
    return clamp(level, header[2], header[1]);
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

// One ray: its colour (linear light) and how far it went before it was half covered.
struct Seen {
    colour: vec3<f32>,
    distance: f32,
    steps: f32,
    level: i32,
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
    var through = 1.0;
    var first = camera.reach.x;
    var t = max(begin, 0.5) + phase;
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
    var first_level = -1;
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
    var steps = 0;
    let water_colour = vec3<f32>(0.05, 0.16, 0.30);
    for (var n = 0; n < 768; n++) {
        if (t > camera.reach.x || through < 0.01) {
            break;
        }
        steps = n + 1;
        let p = o + d * t;
        // Over the world's highest point going up, or under its floor going down: nothing more to meet.
        if ((p.z > ceiling && d.z >= 0.0) || (p.z < -8.0 && d.z <= 0.0)) {
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
        // column two levels coarser round it, else its own, with a fine cell's room above the top.
        var skipped = false;
        let fine = f32(1 << u32(level));
        for (var k = 0; k < 2 && in_air && !walking; k++) {
            // Two levels coarser, then its own.
            let up = select(level, min(level + 2, header[1]), k == 0);
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
                next = min(next, t + (top - p.z) / d.z * -1.0);
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
        // The live level not there (past its window, or not made yet): the first baked one.
        if (!look.known && level == header[2]) {
            level = level + 1;
            look = blend(level, p);
        }
        // Over the last stretch before the next level takes over, the two blended, so the change is gradual,
        // never a seam (seams drew rings round the eye over flat water that moved as it moved: David, build 69).
        let over = log2(max(t * camera.view.z, 1.0)) + camera.reach.y - f32(level);
        if (look.known && over > BLEND_FROM && level < header[1]) {
            let next = blend(level + 1, p);
            if (next.known) {
                look = mix_looks(look, next, (over - BLEND_FROM) / (1.0 - BLEND_FROM));
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
        }
        // The step from the last sample to this one, gathered from both its ends.
        if (look.known && chained && before.known) {
            let length = t - before_t;
            let solid_share = 0.5 * (before.solid + look.solid);
            let water_share = 0.5 * (before.water + look.water);
            let solid = 1.0 - pow(1.0 - min(solid_share, 0.999), length / size);
            let water = (1.0 - pow(1.0 - min(water_share, 0.999), length / size)) * 0.85;
            let covered = 1.0 - (1.0 - solid) * (1.0 - water);
            if (covered > 0.0) {
                // Each end's colour as lit, weighted by how much solid it has.
                let lit_before = before.colour * (camera.sun.w + (1.0 - camera.sun.w) * max(dot(before.facing, camera.sun.xyz), 0.0));
                let lit_now = look.colour * (camera.sun.w + (1.0 - camera.sun.w) * max(dot(look.facing, camera.sun.xyz), 0.0));
                let ground = (lit_before * before.solid + lit_now * look.solid) / max(before.solid + look.solid, 1e-6);
                let here = (ground * solid + water_colour * water) / max(solid + water, 1e-6);
                colour += through * covered * here;
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
    colour += through * sky(d);
    return Seen(colour, first, f32(steps), first_level);
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

@compute @workgroup_size(8, 8)
fn march_pass(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = vec2<u32>(camera.view.xy);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    // A refining layer marches only where the pixel above said it's worth something; elsewhere the screen shows
    // that one. How much it's worth is how much of it the screen shows over the layer above (its alpha).
    var shown = 1.0;
    if (camera.layer.x > 0.0) {
        let above = id.xy / 2u;
        let worth = worth_in[above.y * u32(camera.layer.x) + above.x];
        shown = f32(worth) / 255.0;
        if (worth == 0u) {
            textureStore(picture, vec2<i32>(id.xy), vec4<f32>(0.0));
            textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(camera.reach.x, 0.0, 0.0, 0.0));
            if (camera.layer.y > 0.5) {
                worth_out[id.y * size.x + id.x] = 0u;
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
    var colour = vec3<f32>(0.0);
    var near = camera.reach.x;
    var steps = 0.0;
    var level = -1;
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
        let seen = march(camera.position.xyz, d, scatter(id.xy * 2u + vec2<u32>(r & 1u, r >> 1u)), begin);
        colour += seen.colour;
        near = min(near, seen.distance);
        steps += seen.steps;
        level = max(level, seen.level);
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
    textureStore(how_far, vec2<i32>(id.xy), vec4<f32>(near, 0.0, 0.0, 0.0));
    // How much is the layer below worth here? Nothing while its pixels would be smaller than the cells they'd read
    // (finer than the cells, they only magnify the blend into smudge: David's bands, "2 looks worse than 4, up
    // close"); rising to all of it over the next doubling of distance, so the change of pixel size fades in rather
    // than drawing a ring round the eye; never for sky.
    if (camera.layer.y > 0.5) {
        var worth = select(0.0, 1.0, camera.layer.z < 0.5);
        if (camera.layer.z > 0.5 && near < camera.reach.x) {
            let cell = select(32.0, 16.0, near < camera.layer.w);
            worth = clamp(near * camera.view.z * 0.5 / cell - 1.0, 0.0, 1.0);
        }
        worth_out[id.y * size.x + id.x] = u32(round(worth * 255.0));
    }
}

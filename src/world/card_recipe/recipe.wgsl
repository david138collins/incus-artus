// The recipe on the card (the regroup, phase 2.3): `whole_recipe.rs`'s steps, one for one, in the order
// it does them, reading the world packed into one list of whole numbers (`data`, laid out by
// `card_recipe::Pack`). The kit (`whole/kit.wgsl`) and the noise's arrows come before this. A Fix is an
// i32 in 65536ths; a Pos an i32 in 256ths of a block.

@group(0) @binding(0) var<storage, read> data: array<i32>;

fn word(at: u32) -> i32 {
    return data[at];
}

fn at_u(at: u32) -> u32 {
    return u32(data[at]);
}

// ── The noise (`worldgen/noise.rs`'s whole numbers) ──

fn rotl(x: u32, by: u32) -> u32 {
    return (x << by) | (x >> (32u - by));
}

fn post_hash(key: u32, x: i32, y: i32) -> u32 {
    var h = key;
    for (var i = 0; i < 2; i++) {
        var k = bitcast<u32>(x);
        if (i == 1) {
            k = bitcast<u32>(y);
        }
        h = h ^ (rotl(k * 0xcc9e2d51u, 15u) * 0x1b873593u);
        h = rotl(h, 13u) * 5u + 0xe6546b64u;
    }
    h = h ^ (h >> 16u);
    h = h * 0x85ebca6bu;
    h = h ^ (h >> 13u);
    h = h * 0xc2b2ae35u;
    return h ^ (h >> 16u);
}

fn arrow(key: u32, x: i32, y: i32) -> vec2<i32> {
    return ARROWS[post_hash(key, x, y) >> 26u];
}

// 0 smooth, 1 soft, 2 straight.
fn whole_fade(fade: i32, t_in: i32) -> i32 {
    let t = bitcast<u32>(t_in);
    let t2 = (t * t) >> 16u;
    if (fade == 0) {
        let t3 = (t2 * t) >> 16u;
        let inner = 10u * 65536u + 6u * t2 - 15u * t;
        return bitcast<i32>((t3 * (inner >> 4u)) >> 12u);
    }
    if (fade == 1) {
        return bitcast<i32>((t2 * ((3u * 65536u - 2u * t) >> 2u)) >> 14u);
    }
    return bitcast<i32>(t);
}

fn noise_mix(a: i32, b: i32, t: i32) -> i32 {
    let gap = b - a;
    return a + ((gap * (t >> 4u)) >> 12u) + ((gap * (t & 15)) >> 16u);
}

struct Corners {
    a: array<vec2<i32>, 4>,
    d: array<i32, 4>,
}

fn corners(key: u32, ix: i32, fx: i32, iy: i32, fy: i32) -> Corners {
    var c: Corners;
    c.a[0] = arrow(key, ix, iy);
    c.a[1] = arrow(key, ix + 1, iy);
    c.a[2] = arrow(key, ix, iy + 1);
    c.a[3] = arrow(key, ix + 1, iy + 1);
    let ox = array<i32, 4>(fx, fx - 65536, fx, fx - 65536);
    let oy = array<i32, 4>(fy, fy, fy - 65536, fy - 65536);
    for (var k = 0; k < 4; k++) {
        c.d[k] = (c.a[k].x * ox[k] + c.a[k].y * oy[k]) >> 12u;
    }
    return c;
}

// `whole_perlin_at`: x and y in the noise's units as Fix.
fn perlin_at(key: u32, fade: i32, x: i32, y: i32) -> i32 {
    let c = corners(key, x >> 16u, x & 0xFFFF, y >> 16u, y & 0xFFFF);
    let u = whole_fade(fade, x & 0xFFFF);
    let v = whole_fade(fade, y & 0xFFFF);
    let value = noise_mix(noise_mix(c.d[0], c.d[1], u), noise_mix(c.d[2], c.d[3], u), v);
    return (value * 1434) >> 10u;
}

fn fade_steep(t: i32) -> i32 {
    return 30 * ((((t * t) >> 12u) * (((t - 4096) * (t - 4096)) >> 12u)) >> 12u);
}

// `whole_perlin_with_slope_at`: (value, slope along x, slope along y), all Fix.
fn perlin_with_slope_at(key: u32, x: i32, y: i32) -> vec3<i32> {
    let fx_full = x & 0xFFFF;
    let fy_full = y & 0xFFFF;
    let c = corners(key, x >> 16u, fx_full, y >> 16u, fy_full);
    let fine_u = whole_fade(0, fx_full);
    let fine_v = whole_fade(0, fy_full);
    let value = noise_mix(noise_mix(c.d[0], c.d[1], fine_u), noise_mix(c.d[2], c.d[3], fine_u), fine_v);
    let fx = fx_full >> 4u;
    let fy = fy_full >> 4u;
    let u = fine_u >> 4u;
    let v = fine_v >> 4u;
    let du = fade_steep(fx);
    let dv = fade_steep(fy);
    let a = c.d[0];
    let b = c.d[1];
    let cc = c.d[2];
    let d = c.d[3];
    let k = a - b - cc + d;
    let uv = (u * v) >> 12u;
    let across_x = (b - a) + ((v * k) >> 12u);
    let across_y = (cc - a) + ((u * k) >> 12u);
    let along_x = c.a[0].x + ((u * (c.a[1].x - c.a[0].x)) >> 12u) + ((v * (c.a[2].x - c.a[0].x)) >> 12u) + ((uv * (c.a[0].x - c.a[1].x - c.a[2].x + c.a[3].x)) >> 12u) + ((du * (across_x >> 4u)) >> 12u);
    let along_y = c.a[0].y + ((u * (c.a[1].y - c.a[0].y)) >> 12u) + ((v * (c.a[2].y - c.a[0].y)) >> 12u) + ((uv * (c.a[0].y - c.a[1].y - c.a[2].y + c.a[3].y)) >> 12u) + ((dv * (across_y >> 4u)) >> 12u);
    return vec3<i32>((value * 1434) >> 10u, ((along_x * 1434) >> 10u) << 4u, ((along_y * 1434) >> 10u) << 4u);
}

// ── Positions (`Pos`) ──

fn pos_scaled(p: i32, by: i32) -> i32 {
    return fix_mul(p, by);
}

fn pos_ratio(a: i32, b: i32) -> i32 {
    return fix_div(a, b);
}

fn pos_to_fix(p: i32) -> i32 {
    return p << 8u;
}

// ── Curves (`WholeCurve`): [stretches, single, then per stretch x0 x1 y0 y1 h hm0 hm1 has_power power] ──

const STRETCH: u32 = 9u;

fn curve_at(off: u32, x_in: i32) -> i32 {
    let n = at_u(off);
    if (n == 0u) {
        return word(off + 1u);
    }
    let first = off + 2u;
    let last = first + (n - 1u) * STRETCH;
    let x = clamp(x_in, word(first), word(last + 1u));
    var s = last;
    for (var k = 0u; k < n; k++) {
        if (x <= word(first + k * STRETCH + 1u)) {
            s = first + k * STRETCH;
            break;
        }
    }
    var t = clamp(fix_div(x - word(s), word(s + 4u)), 0, 65536);
    if (word(s + 7u) != 0) {
        t = fix_pow(t, word(s + 8u));
    }
    let t2 = fix_mul(t, t);
    let t3 = fix_mul(t2, t);
    let h00 = 2 * t3 - 3 * t2 + 65536;
    let h10 = t3 - 2 * t2 + t;
    let h01 = 3 * t2 - 2 * t3;
    let h11 = t3 - t2;
    return fix_mul(h00, word(s + 2u)) + fix_mul(h10, word(s + 5u)) + fix_mul(h01, word(s + 3u)) + fix_mul(h11, word(s + 6u));
}

fn curve_steepness(off: u32, x_in: i32) -> i32 {
    let n = at_u(off);
    if (n == 0u) {
        return 0;
    }
    let first = off + 2u;
    let last = first + (n - 1u) * STRETCH;
    let x = clamp(x_in, word(first), word(last + 1u));
    var s = last;
    for (var k = 0u; k < n; k++) {
        if (x <= word(first + k * STRETCH + 1u)) {
            s = first + k * STRETCH;
            break;
        }
    }
    var t = clamp(fix_div(x - word(s), word(s + 4u)), 0, 65536);
    var bowed_by = 65536;
    if (word(s + 7u) != 0) {
        let power = word(s + 8u);
        bowed_by = fix_mul(power, min(fix_pow(t, power - 65536), 64 << 16u));
        t = fix_pow(t, power);
    }
    let t2 = fix_mul(t, t);
    let d00 = 6 * t2 - 6 * t;
    let d10 = 3 * t2 - 4 * t + 65536;
    let d01 = 6 * t - 6 * t2;
    let d11 = 3 * t2 - 2 * t;
    let sum = fix_mul(d00, word(s + 2u)) + fix_mul(d10, word(s + 5u)) + fix_mul(d01, word(s + 3u)) + fix_mul(d11, word(s + 6u));
    return fix_div(fix_mul(sum, bowed_by), word(s + 4u));
}

// ── Noise layers (`WholeNoise`): [offset_north offset_east cos sin over_stretch turn_cos turn_sin octaves
//    solo fade contrast brightness has_power power invert, then per octave key size weight] ──

const LAYER: u32 = 15u;

fn noise_raw(off: u32, north: i32, east: i32) -> i32 {
    let x0 = east + word(off + 1u);
    let y0 = north + word(off);
    let cos = word(off + 2u);
    let sin = word(off + 3u);
    var x = pos_scaled(pos_scaled(x0, cos) + pos_scaled(y0, sin), word(off + 4u));
    var y = pos_scaled(y0, cos) - pos_scaled(x0, sin);
    let turn_cos = word(off + 5u);
    let turn_sin = word(off + 6u);
    let n = at_u(off + 7u);
    let solo = word(off + 8u);
    let fade = word(off + 9u);
    var total = 0;
    for (var k = 0u; k < n; k++) {
        let o = off + LAYER + k * 3u;
        let size = word(o + 1u);
        let value = perlin_at(at_u(o), fade, pos_per(x, size), pos_per(y, size));
        if (solo == i32(k)) {
            return value;
        }
        total = total + fix_mul(value, word(o + 2u));
        let nx = pos_scaled(x, turn_cos) - pos_scaled(y, turn_sin);
        let ny = pos_scaled(x, turn_sin) + pos_scaled(y, turn_cos);
        x = nx;
        y = ny;
    }
    return total;
}

fn shape(off: u32, t_in: i32) -> i32 {
    var t = clamp(fix_mul(t_in - 32768, word(off + 10u)) + 32768 + word(off + 11u), 0, 65536);
    if (word(off + 12u) != 0) {
        t = fix_pow(t, word(off + 13u));
    }
    if (word(off + 14u) != 0) {
        return 65536 - t;
    }
    return t;
}

fn noise_at(off: u32, north: i32, east: i32) -> i32 {
    let shaped = shape(off, fix_mul(noise_raw(off, north, east), 32768) + 32768);
    return shaped * 2 - 65536;
}

// ── The ground (`WholeMap`) ──

fn h(slot: u32) -> i32 {
    return data[slot];
}

fn hu(slot: u32) -> u32 {
    return u32(data[slot]);
}

fn muted(bit: i32) -> bool {
    return (h(H_MUTES) & bit) != 0;
}

const SEA_Z: i32 = 128 << 16u;
const HIGHEST_Z: i32 = 333 << 16u;
const TINY_FIX: i32 = 1;

fn div_floor(a: i32, b: i32) -> i32 {
    let q = a / b;
    if ((a % b != 0) && ((a < 0) != (b < 0))) {
        return q - 1;
    }
    return q;
}

fn smooth_fix(t: i32) -> i32 {
    return fix_smooth(t);
}

fn powered(x: i32, has: i32, power: i32) -> i32 {
    if (has != 0) {
        return fix_pow(x, power);
    }
    return x;
}

struct Here {
    coast: i32,
    height: i32,
    tilt: i32,
    squeeze: i32,
    push_x: i32,
    push_y: i32,
    push_strength: i32,
    coast_distance: i32,
    lift_distance: i32,
}

fn catmull_rom(a: i32, b: i32, c: i32, d: i32, t: i32) -> i32 {
    let t2 = fix_mul(t, t);
    let t3 = fix_mul(t2, t);
    let first = b * 2;
    let second = fix_mul(c - a, t);
    let third = fix_mul(2 * a - 5 * b + 4 * c - d, t2);
    let fourth = fix_mul(3 * (b - c) + d - a, t3);
    return (first + second + third + fourth) >> 1u;
}

fn field_at(field: u32, c: i32, r: i32) -> i32 {
    let w = h(H_WIDTH);
    return data[field + u32(clamp(r, 0, w - 1) * w + clamp(c, 0, w - 1))];
}

fn blend(field: u32, c0: i32, r0: i32, tc: i32, tr: i32) -> i32 {
    var rows: array<i32, 4>;
    for (var k = 0; k < 4; k++) {
        let r = r0 - 1 + k;
        rows[k] = catmull_rom(field_at(field, c0 - 1, r), field_at(field, c0, r), field_at(field, c0 + 1, r), field_at(field, c0 + 2, r), tc);
    }
    let n0 = field_at(field, c0, r0);
    let n1 = field_at(field, c0 + 1, r0);
    let n2 = field_at(field, c0, r0 + 1);
    let n3 = field_at(field, c0 + 1, r0 + 1);
    let low = min(min(n0, n1), min(n2, n3));
    let high = max(max(n0, n1), max(n2, n3));
    return clamp(catmull_rom(rows[0], rows[1], rows[2], rows[3], tr), low, high);
}

// The maps blended to a spot (north, east): the maps' values, and the spot from the map's north-west
// corner (x, y). `ok` false past the map's edge.
struct MapsAt {
    ok: bool,
    here: Here,
    x: i32,
    y: i32,
}

fn maps_at(north: i32, east: i32) -> MapsAt {
    var out: MapsAt;
    let w = h(H_WIDTH);
    let half_pixels = w * 32768;
    let col = pos_ratio(east, h(H_BLOCKS)) + half_pixels - 32768;
    let row = half_pixels - pos_ratio(north, h(H_BLOCKS)) - 32768;
    let last = (w << 16u) - 32768;
    if (col < -32768 || row < -32768 || col > last || row > last) {
        out.ok = false;
        return out;
    }
    let c0 = col >> 16u;
    let r0 = row >> 16u;
    let tc = col & 0xFFFF;
    let tr = row & 0xFFFF;
    out.ok = true;
    out.here.coast = blend(hu(H_COAST), c0, r0, tc, tr);
    out.here.height = blend(hu(H_HEIGHT), c0, r0, tc, tr);
    out.here.tilt = blend(hu(H_TILT), c0, r0, tc, tr);
    out.here.squeeze = blend(hu(H_SQUEEZE), c0, r0, tc, tr);
    out.here.push_x = blend(hu(H_PUSH_X), c0, r0, tc, tr);
    out.here.push_y = blend(hu(H_PUSH_Y), c0, r0, tc, tr);
    out.here.push_strength = blend(hu(H_PUSH_STRENGTH), c0, r0, tc, tr);
    out.here.coast_distance = blend(hu(H_COAST_DISTANCE), c0, r0, tc, tr);
    out.here.lift_distance = blend(hu(H_LIFT_DISTANCE), c0, r0, tc, tr);
    out.x = east + h(H_HALF);
    out.y = h(H_HALF) - north;
    return out;
}

fn mountain_share(here: Here) -> i32 {
    if (here.squeeze <= 0) {
        return 0;
    }
    return clamp(curve_at(hu(H_MOUNTAINS), fix_div(here.squeeze, max(h(H_MOST_CRUSHED), TINY_FIX))), 0, 65536);
}

fn lifted(here: Here) -> i32 {
    if (h(H_LIFT_HEIGHT) <= 0 || here.coast < 0 || muted(1)) {
        return 0;
    }
    let near = 65536 - clamp(pos_ratio(here.lift_distance, h(H_LIFT_REACH)), 0, 65536);
    let pad = powered(smooth_fix(pos_ratio(here.coast_distance, h(H_LIFT_PAD))), h(H_LIFT_PAD_HAS), h(H_LIFT_PAD_POWER));
    return fix_mul(fix_mul(h(H_LIFT_HEIGHT), clamp(curve_at(hu(H_LIFT_CURVE), near), 0, 65536)), pad);
}

fn raised_coast(here: Here) -> i32 {
    if (h(H_COAST_RISE) <= 0 || here.coast < 0 || here.tilt <= 0 || muted(4)) {
        return 0;
    }
    let across = clamp(pos_ratio(here.coast_distance, h(H_COAST_EDGE_WIDTH)), 0, 65536);
    let past = here.coast_distance - h(H_COAST_EDGE_WIDTH);
    var falling = 65536;
    if (h(H_COAST_FALL_BACK) > 0 && past > 0) {
        falling = 65536 - smooth_fix(pos_ratio(past, h(H_COAST_FALL_BACK)));
    }
    return fix_mul(fix_mul(fix_mul(h(H_COAST_RISE), here.tilt), clamp(curve_at(hu(H_COAST_EDGE), across), 0, 65536)), falling);
}

// A ridge pattern from the layers' keys starting at `keys`.
fn ridges(keys: u32, u: i32, v: i32) -> i32 {
    var total = 0;
    let n = hu(H_RIDGE_LAYERS);
    for (var k = 0u; k < n; k++) {
        let scale = data[hu(H_RIDGE_SCALES) + k];
        let crest = 65536 - min(abs(perlin_at(u32(data[keys + k]), 0, fix_mul(u, scale), fix_mul(v, scale))), 65536);
        total = total + fix_mul(fix_pow(crest, h(H_SHARPNESS)), data[hu(H_RIDGE_STRENGTHS) + k]);
    }
    return fix_div(total, h(H_RIDGE_TOTAL));
}

fn ridge_keys(direction: u32) -> u32 {
    return hu(H_RIDGE_KEYS) + direction * hu(H_RIDGE_LAYERS);
}

fn lined(x: i32, y: i32, cos: i32, sin: i32, keys: u32) -> i32 {
    let across = pos_scaled(x, cos) - pos_scaled(y, sin);
    let along = pos_scaled(y, cos) + pos_scaled(x, sin);
    return ridges(keys, pos_per(across, h(H_RIDGE_SPACING)), pos_per(along, h(H_RIDGE_LENGTH)));
}

fn ridge_pattern(here: Here, x_in: i32, y_in: i32) -> i32 {
    let angle = fix_atan2(here.push_y, here.push_x) >> 1u;
    var x = x_in;
    var y = y_in;
    if (h(H_RIDGE_WANDER) > 0) {
        let size = h(H_RIDGE_WANDER_SIZE);
        let nx = pos_scaled(h(H_RIDGE_WANDER), perlin_at(hu(H_WANDER_KEY_0), 0, pos_per(x_in, size), pos_per(y_in, size)));
        let ny = pos_scaled(h(H_RIDGE_WANDER), perlin_at(hu(H_WANDER_KEY_1), 0, pos_per(x_in, size), pos_per(y_in, size)));
        x = x_in + nx;
        y = y_in + ny;
    }
    var lined_up: i32;
    if (h(H_STEADY) != 0) {
        let k = (angle + 16384) * 24;
        let first = ((k >> 16u) % 12 + 12) % 12;
        let t = smooth_fix(k & 0xFFFF);
        let second = (first + 1) % 12;
        let da = hu(H_DIRECTIONS) + u32(first) * 2u;
        let db = hu(H_DIRECTIONS) + u32(second) * 2u;
        let crest_a = lined(x, y, data[da], data[da + 1u], ridge_keys(u32(first) + 1u));
        let crest_b = lined(x, y, data[db], data[db + 1u], ridge_keys(u32(second) + 1u));
        lined_up = crest_a + fix_mul(crest_b - crest_a, t);
    } else {
        lined_up = lined(x, y, fix_cos(angle), fix_sin(angle), ridge_keys(0u));
    }
    let wandering = ridges(ridge_keys(13u), pos_per(x, h(H_RIDGE_SPACING)), pos_per(y, h(H_RIDGE_SPACING)));
    let clear = smooth_fix(fix_mul(here.push_strength, h(H_RIDGE_ALIGNMENT)));
    return wandering + fix_mul(lined_up - wandering, clear);
}

fn pos_hypot(a: i32, b: i32) -> i32 {
    return fix_hypot(a, b);
}

// The peaks' rise here, a block east and a block south (x, y, z), and the crags' room (w).
fn peak_rise(x: i32, y: i32) -> vec4<i32> {
    var rise = vec3<i32>(0, 0, 0);
    var room = 0;
    let across = h(H_PEAK_ACROSS);
    if (across == 0) {
        return vec4<i32>(0, 0, 0, 0);
    }
    let side = max(h(H_PEAK_SIDE), 1);
    let c0 = div_floor(x, side);
    let r0 = div_floor(y, side);
    for (var r = r0 - 1; r <= r0 + 1; r++) {
        for (var c = c0 - 1; c <= c0 + 1; c++) {
            if (c < 0 || c >= across || r < 0 || r >= across) {
                continue;
            }
            let cell = hu(H_PEAK_CELLS) + u32(r * across + c) * 2u;
            let first = u32(data[cell]);
            let count = u32(data[cell + 1u]);
            for (var p = 0u; p < count; p++) {
                let at = hu(H_PEAK_LIST) + (first + p) * 6u;
                let px = data[at];
                let py = data[at + 1u];
                let height = data[at + 2u];
                let peak_width = data[at + 3u];
                let wobble = data[at + 4u];
                let key = u32(data[at + 5u]);
                let dx = x - px;
                let dy = y - py;
                let furthest = peak_width + pos_scaled(peak_width, wobble) + 384;
                if (pos_hypot(dx, dy) >= furthest) {
                    continue;
                }
                var width = peak_width;
                if (wobble > 0) {
                    let size = pos_scaled(width, 39322);
                    width = width + pos_scaled(width, fix_mul(wobble, perlin_at(key, 0, pos_per(dx, size), pos_per(dy, size))));
                }
                let width_fix = pos_to_fix(max(width, 256));
                let here_d = fix_hypot(pos_to_fix(dx), pos_to_fix(dy));
                let t = 65536 - fix_div(here_d, width_fix);
                if (t <= 0) {
                    continue;
                }
                let rise_t = curve_at(hu(H_RISE), t);
                let own = fix_mul(height, clamp(rise_t, 0, 65536));
                var steepness = 0;
                if (rise_t > 0 && rise_t < 65536) {
                    steepness = fix_mul(height, curve_steepness(hu(H_RISE), t));
                }
                let east_d = fix_hypot(pos_to_fix(dx + 256), pos_to_fix(dy));
                let south_d = fix_hypot(pos_to_fix(dx), pos_to_fix(dy + 256));
                let east_v = own + fix_mul(steepness, fix_div(here_d - east_d, width_fix));
                let south_v = own + fix_mul(steepness, fix_div(here_d - south_d, width_fix));
                if (h(H_PEAKS_ADD) != 0) {
                    rise = rise + vec3<i32>(own, east_v, south_v);
                } else {
                    rise = max(rise, vec3<i32>(own, east_v, south_v));
                }
                var share = 65536;
                if (h(H_FOOT_FADE) > 0) {
                    share = smooth_fix(fix_div(t, h(H_FOOT_FADE)));
                }
                room = max(room, fix_mul(height, share));
            }
        }
    }
    return vec4<i32>(rise, room);
}

fn crag_field(x: i32, y: i32, strength: i32, slope: vec2<i32>) -> i32 {
    var total = 0;
    var steep = slope;
    let n = hu(H_CRAG_COUNT);
    for (var k = 0u; k < n; k++) {
        let at = hu(H_CRAG_LAYERS) + k * 3u;
        let size = data[at];
        let kept_share = data[at + 1u];
        let key = u32(data[at + 2u]);
        let noise = perlin_with_slope_at(key, pos_per(x, size), pos_per(y, size));
        let kept = clamp(curve_at(hu(H_SLOPE_RESPONSE), min(fix_div(fix_hypot(steep.x, steep.y), h(H_STEEPEST)), 65536)), 0, 65536);
        let layer = fix_mul(fix_mul(strength, kept_share), kept);
        total = total + fix_mul(layer, noise.x);
        let size_fix = pos_to_fix(size);
        steep = vec2<i32>(steep.x + fix_div(fix_mul(layer, noise.y), size_fix), steep.y + fix_div(fix_mul(layer, noise.z), size_fix));
    }
    return total;
}

fn peaks(here: Here, x: i32, y: i32) -> i32 {
    if (h(H_PEAKS_ON) == 0 || here.coast < 0 || muted(64)) {
        return 0;
    }
    let r = peak_rise(x, y);
    if (r.w <= 0) {
        return 0;
    }
    var total = r.x;
    if (h(H_CRAGS) > 0 && !muted(128)) {
        total = total + crag_field(x, y, fix_mul(r.w, h(H_CRAGS)), vec2<i32>(r.y - r.x, r.z - r.x));
    }
    if (h(H_CARVE_BELOW) == 0) {
        total = max(total, 0);
    }
    let fade = powered(smooth_fix(pos_ratio(here.coast_distance, h(H_PEAK_COAST_FADE))), h(H_PEAK_COAST_HAS), h(H_PEAK_COAST_POWER));
    return fix_mul(total, fade);
}

fn summit_crags(here: Here, x: i32, y: i32, z: i32) -> i32 {
    if (h(H_PEAKS_SWITCHED) == 0 || h(H_SUMMIT) == 0 || muted(256) || z <= h(H_SUMMIT_FROM)) {
        return 0;
    }
    let across = min(fix_div(z - h(H_SUMMIT_FROM), max(h(H_SUMMIT_FULL) - h(H_SUMMIT_FROM), 65536)), 65536);
    let shown = clamp(curve_at(hu(H_SUMMIT_FADE), across), 0, 65536);
    if (shown <= 0) {
        return 0;
    }
    let coast = powered(smooth_fix(pos_ratio(here.coast_distance, h(H_PEAK_COAST_FADE))), h(H_PEAK_COAST_HAS), h(H_PEAK_COAST_POWER));
    return fix_mul(fix_mul(fix_mul(noise_at(hu(H_SUMMIT_NOISE), -y, x), h(H_SUMMIT)), shown), coast);
}

fn torn_coast(here: Here, x: i32, y: i32) -> i32 {
    if (h(H_TORN_AMOUNT) == 0 || muted(16)) {
        return 0;
    }
    let d = here.coast_distance;
    var band: i32;
    if (d >= 0) {
        band = 65536 - pos_ratio(d, h(H_TORN_INLAND));
    } else {
        band = 65536 + pos_ratio(d, h(H_TORN_OUT));
    }
    if (band <= 0) {
        return 0;
    }
    var shown: i32;
    let patches = h(H_TORN_PATCHES);
    if (patches >= 65536) {
        shown = 65536;
    } else if (patches <= 0) {
        shown = 0;
    } else {
        let size = h(H_TORN_PATCH_SIZE);
        let patch_value = fix_mul(perlin_at(hu(H_TORN_PATCH_KEY), 0, pos_per(x, size), pos_per(y, size)), 32768) + 32768;
        shown = smooth_fix((patch_value - (65536 - patches)) * 5 + 32768);
    }
    if (shown <= 0) {
        return 0;
    }
    let push = noise_at(hu(H_TORN_NOISE), -y, x) + h(H_TORN_BALANCE);
    return fix_mul(fix_mul(fix_mul(push, h(H_TORN_AMOUNT)), smooth_fix(band)), shown);
}

// The ground, the mountains on it, and the peaks on both.
fn parts(here: Here, x: i32, y: i32) -> vec3<i32> {
    let steep = max(65536 + fix_mul(h(H_GRADE), here.tilt), 9830);
    var ground: i32;
    if (here.coast >= 0) {
        let t = fix_div(max(here.height - h(H_WATER), 0), max(h(H_HIGHEST) - h(H_WATER), TINY_FIX));
        ground = SEA_Z + fix_mul(fix_mul(max(h(H_HIGHLANDS) - SEA_Z, 0), clamp(curve_at(hu(H_LAND), t), 0, 65536)), steep);
    } else {
        let t = fix_div(max(h(H_WATER) - here.height, 0), max(h(H_WATER) - h(H_LOWEST), TINY_FIX));
        ground = (SEA_Z - 65536) - fix_mul(fix_mul(max(SEA_Z - 65536 - h(H_DEEPEST), 0), clamp(curve_at(hu(H_SEA), t), 0, 65536)), steep);
    }
    ground = ground + raised_coast(here) + lifted(here);
    var mountains = 0;
    let tall = mountain_share(here);
    if (tall > 0 && !muted(2)) {
        let crest = ridge_pattern(here, x, y);
        var carved = 65536;
        if (!muted(8)) {
            carved = max(65536 - fix_mul(h(H_RIDGES), 65536 - crest), 0);
        }
        mountains = fix_mul(fix_mul(max(HIGHEST_Z - ground, 0), tall), carved);
    }
    return vec3<i32>(ground, mountains, peaks(here, x, y));
}

fn put_together(here: Here, x: i32, y: i32) -> i32 {
    return put_together_parts(here, x, y, parts(here, x, y));
}

// The same with the parts already worked out (a column works them out once, for its ground and its blocks).
fn put_together_parts(here: Here, x: i32, y: i32, p: vec3<i32>) -> i32 {
    let below = p.x + p.y;
    var z = min(below + p.z, h(H_CEILING));
    if (below >= SEA_Z) {
        z = max(z, SEA_Z);
        z = clamp(z + summit_crags(here, x, y, z), SEA_Z, h(H_CEILING));
    }
    z = z + torn_coast(here, x, y);
    return clamp(z, 65536, max(h(H_CEILING), 65536));
}

fn detail_shown(here: Here, z: i32) -> i32 {
    var t: i32;
    let source = h(H_MASK_SOURCE);
    if (source == 0) {
        return 65536;
    } else if (source == 1) {
        t = 32768 + fix_mul(32768, clamp(pos_ratio(here.coast_distance, h(H_MASK_REACH)), -65536, 65536));
    } else if (source == 2) {
        t = fix_div(z, HIGHEST_Z);
    } else if (source == 3) {
        t = mountain_share(here);
    } else if (here.squeeze >= 0) {
        t = 32768 + fix_mul(32768, fix_div(here.squeeze, max(h(H_MOST_CRUSHED), TINY_FIX)));
    } else {
        t = 32768 - fix_mul(32768, fix_div(here.squeeze, min(h(H_MOST_STRETCHED), -TINY_FIX)));
    }
    t = clamp(t, 0, 65536);
    if (h(H_MASK_INVERT) != 0) {
        t = 65536 - t;
    }
    return clamp(curve_at(hu(H_MASK_CURVE), t), 0, 65536);
}

// The ground's height (a Z, with its fraction) before the rivers carve it; `ok` false past the map.
struct Height {
    ok: bool,
    z: i32,
}

fn uncarved(north: i32, east: i32) -> Height {
    let m = maps_at(north, east);
    var p = vec3<i32>(0, 0, 0);
    if (m.ok) {
        p = parts(m.here, m.x, m.y);
    }
    return uncarved_from(north, east, m, p);
}

// The same from the maps and the parts already worked out.
fn uncarved_from(north: i32, east: i32, m: MapsAt, p: vec3<i32>) -> Height {
    var out: Height;
    if (!m.ok) {
        out.ok = false;
        return out;
    }
    out.ok = true;
    let ground = put_together_parts(m.here, m.x, m.y, p);
    if (muted(32) || h(H_DETAIL) == 0) {
        out.z = ground;
        return out;
    }
    let shown = detail_shown(m.here, ground);
    var detail = 0;
    if (shown > 0) {
        detail = fix_mul(fix_mul(noise_at(hu(H_DETAIL_NOISE), north, east), h(H_DETAIL)), shown);
    }
    if (h(H_KEEPS_COAST) != 0) {
        let waterline = SEA_Z - 32768;
        let nudge = 655;
        if (ground >= waterline) {
            detail = max(detail, waterline + nudge - ground);
        } else {
            detail = min(detail, waterline - nudge - ground);
        }
    }
    out.z = ground + detail;
    return out;
}

// ── The rivers and lakes (`WholeCarving`) ──

// A point along a river: north east width depth edge flat bowl level bend has_curtain curtain fade.
const POINT_WORDS: u32 = 12u;

struct Point {
    north: i32,
    east: i32,
    width: i32,
    depth: i32,
    edge: i32,
    flat: i32,
    bowl: i32,
    level: i32,
    bend: i32,
    has_curtain: bool,
    curtain: i32,
    fade: i32,
}

fn point_at(reach: u32, k: u32) -> Point {
    let first = u32(data[hu(H_REACHES) + reach * 2u]);
    let at = hu(H_POINTS) + (first + k) * POINT_WORDS;
    var p: Point;
    p.north = data[at];
    p.east = data[at + 1u];
    p.width = data[at + 2u];
    p.depth = data[at + 3u];
    p.edge = data[at + 4u];
    p.flat = data[at + 5u];
    p.bowl = data[at + 6u];
    p.level = data[at + 7u];
    p.bend = data[at + 8u];
    p.has_curtain = data[at + 9u] != 0;
    p.curtain = data[at + 10u];
    p.fade = data[at + 11u];
    return p;
}

fn rounded(x: i32) -> i32 {
    return (x + 32768) & ~0xFFFF;
}

fn whole_number(x: i32) -> i32 {
    return rounded(x) >> 16u;
}

fn under_the_sea(height: i32) -> bool {
    return whole_number(height) < 128;
}

fn depth_across(p: Point, across: i32) -> i32 {
    let flat = clamp(p.flat, 0, 62259);
    let u = min(abs(across), 65536);
    var t = 65536;
    if (u > flat) {
        t = fix_div(65536 - u, 65536 - flat);
    }
    let at_edge = fix_mul(clamp(p.edge, 0, 65536), p.depth);
    return max(rounded(at_edge + fix_mul(p.depth - at_edge, fix_pow(t, max(p.bowl, 3277)))), 65536);
}

// Where a spot is along a piece: distance (x), side (y), how far along (z).
fn along(a: Point, b: Point, north: i32, east: i32) -> vec3<i32> {
    let dn = pos_to_fix(b.north - a.north);
    let de = pos_to_fix(b.east - a.east);
    let mn = pos_to_fix(north - a.north);
    let me = pos_to_fix(east - a.east);
    let length = fix_hypot(dn, de);
    var t = 0;
    var side = 0;
    if (length > TINY_FIX) {
        let un = fix_div(dn, length);
        let ue = fix_div(de, length);
        t = clamp(fix_div(fix_mul(mn, un) + fix_mul(me, ue), length), 0, 65536);
        side = fix_mul(fix_div(de, length), mn) - fix_mul(fix_div(dn, length), me);
    }
    let pn = fix_mul(dn, t);
    let pe = fix_mul(de, t);
    let distance = fix_hypot(mn - pn, me - pe);
    var which = 65536;
    if (side <= 0) {
        which = -65536;
    }
    return vec3<i32>(distance, which, t);
}

fn half_across(a: Point, b: Point, width: i32) -> i32 {
    let half = max(width, 65536) >> 1u;
    if (h(H_ANTI_KITTY) == 0) {
        return half;
    }
    let dn = pos_to_fix(b.north - a.north);
    let de = pos_to_fix(b.east - a.east);
    let length = max(fix_hypot(dn, de), TINY_FIX);
    return max(half, fix_div(abs(dn) + abs(de), length) >> 1u);
}

fn blend_point(a: Point, b: Point, t: i32, bend: i32) -> Point {
    var p = a;
    p.north = a.north + pos_scaled(b.north - a.north, t);
    p.east = a.east + pos_scaled(b.east - a.east, t);
    p.width = a.width + fix_mul(b.width - a.width, t);
    p.depth = a.depth + fix_mul(b.depth - a.depth, t);
    p.edge = a.edge + fix_mul(b.edge - a.edge, t);
    p.flat = a.flat + fix_mul(b.flat - a.flat, t);
    p.bowl = a.bowl + fix_mul(b.bowl - a.bowl, t);
    p.bend = bend;
    p.fade = a.fade + fix_mul(b.fade - a.fade, t);
    return p;
}

struct Piece {
    ok: bool,
    distance: i32,
    side: i32,
    point: Point,
    past: i32,
    half: i32,
}

fn grid_cell(north: i32, east: i32) -> u32 {
    let cell = max(h(H_GRID_CELL), 1);
    let across = h(H_GRID_ACROSS);
    let row = min(max(div_floor(h(H_GRID_NORTH) - north, cell), 0), across - 1);
    let col = min(max(div_floor(east - h(H_GRID_EAST), cell), 0), across - 1);
    return u32(row * across + col);
}

// Lower water, then the deeper floor, first.
fn lower(level: i32, floor: i32, than_level: i32, than_floor: i32) -> bool {
    return level < than_level || (level == than_level && floor < than_floor);
}

fn nearest_piece(north: i32, east: i32) -> Piece {
    var best: Piece;
    best.ok = false;
    if (h(H_GRID_ACROSS) == 0) {
        return best;
    }
    let reach_blocks = h(H_GRID_CELL) * h(H_GRID_ACROSS);
    let inside = north <= h(H_GRID_NORTH) && east >= h(H_GRID_EAST) && north > h(H_GRID_NORTH) - reach_blocks && east < h(H_GRID_EAST) + reach_blocks;
    if (!inside) {
        return best;
    }
    var wet: Piece;
    wet.ok = false;
    var wet_level = 0;
    var wet_floor = 0;
    var stretch_reach = 0xFFFFFFFFu;
    var stretch: Piece;
    stretch.ok = false;
    var stretch_level = 0;
    var stretch_floor = 0;
    let cell = grid_cell(north, east);
    let first = u32(data[hu(H_GRID_CELLS) + cell * 2u]);
    let count = u32(data[hu(H_GRID_CELLS) + cell * 2u + 1u]);
    for (var e = 0u; e < count; e++) {
        let r = u32(data[hu(H_GRID_ENTRIES) + (first + e) * 2u]);
        let k = u32(data[hu(H_GRID_ENTRIES) + (first + e) * 2u + 1u]);
        if (r != stretch_reach) {
            if (stretch.ok && (!wet.ok || lower(stretch_level, stretch_floor, wet_level, wet_floor))) {
                wet = stretch;
                wet_level = stretch_level;
                wet_floor = stretch_floor;
            }
            stretch.ok = false;
            stretch_reach = r;
        }
        let a = point_at(r, k);
        let b = point_at(r, k + 1u);
        let al = along(a, b, north, east);
        let distance = al.x;
        let side = al.y;
        let t = al.z;
        let nearer = !best.ok || distance < best.distance;
        let width = a.width + fix_mul(b.width - a.width, t);
        var bend = b.bend;
        if (t < 32768) {
            bend = a.bend;
        }
        var shift = 0;
        if (whole_number(width) % 2 == 0) {
            shift = fix_mul(bend, 32768);
        }
        let half = half_across(a, b, width);
        let offset = fix_mul(distance, side) - shift;
        let in_water = abs(offset) <= half && (!stretch.ok || distance < stretch.distance);
        if (!nearer && !in_water) {
            continue;
        }
        let span = fix_hypot(pos_to_fix(b.north - a.north), pos_to_fix(b.east - a.east));
        var piece: Piece;
        piece.ok = true;
        piece.distance = distance;
        piece.side = side;
        piece.point = blend_point(a, b, t, bend);
        piece.past = fix_mul(t, span);
        piece.half = half;
        if (nearer) {
            best = piece;
        }
        if (in_water) {
            let level = rounded(piece.point.level);
            stretch_level = level;
            stretch_floor = level - 65536 - depth_across(piece.point, fix_div(offset, half));
            stretch = piece;
        }
    }
    if (stretch.ok && (!wet.ok || lower(stretch_level, stretch_floor, wet_level, wet_floor))) {
        wet = stretch;
    }
    if (wet.ok) {
        return wet;
    }
    return best;
}

// 0 smooth, 1 slope, 2 terraces, 3 ledge then slope.
struct Bank {
    ok: bool,
    z: i32,
}

fn bank_ground(level: i32, fade_in: i32, beyond: i32, ground: i32) -> Bank {
    var out: Bank;
    let way = h(H_BANK_WAY);
    var shore = level - 65536;
    if (way == 3) {
        shore = level;
    }
    let rise = ground - shore;
    var fade = max(fade_in, 65536);
    if (rise > 0) {
        fade = clamp(fix_div(rise, max(h(H_BANK_STEEPEST), 3277)), fade, fix_mul(fade, max(h(H_BANK_WIDEST), 65536)));
    }
    if (beyond >= fade) {
        out.ok = false;
        return out;
    }
    let t = clamp(fix_div(beyond, fade), 0, 65536);
    let eased = fix_smooth(t);
    let terrace = max(h(H_BANK_TERRACE), 65536);
    out.ok = true;
    if (way == 1) {
        out.z = shore + fix_mul(rise, t);
    } else if (way == 2 && rise > 0) {
        out.z = shore + fix_mul((fix_div(fix_mul(rise, t), terrace) >> 16u) << 16u, terrace);
    } else {
        out.z = shore + fix_mul(rise, eased);
    }
    return out;
}

fn bank_compressed(level: i32, fade_in: i32, beyond: i32, ground: i32) -> Bank {
    var out: Bank;
    let way = h(H_BANK_WAY);
    let shore = level - 65536;
    let rise = ground - shore;
    var kept = 0;
    if (rise > 0 && way == 3) {
        kept = min(65536, rise);
    }
    let rest = rise - kept;
    var peak = 65536;
    if (way == 0 || way == 3) {
        peak = 98304;
    }
    var fade = max(fade_in, 65536);
    if (rest > 0) {
        fade = clamp(fix_div(fix_mul(rest, peak), max(h(H_BANK_STEEPEST), 3277)), fade, fix_mul(fade, max(h(H_BANK_WIDEST), 65536)));
    } else if (rise < 0) {
        fade = clamp(fix_div(fix_mul(-rise, peak), max(h(H_BANK_RAISED_STEEPEST), 3277)), fade, fix_mul(fade, max(h(H_BANK_RAISED_WIDEST), 65536)));
    }
    let t = fix_div(max(beyond - 65536, 0), fade);
    if (t >= 65536) {
        out.ok = false;
        return out;
    }
    let eased = fix_smooth(t);
    let terrace = max(h(H_BANK_TERRACE), 65536);
    var add: i32;
    if (way == 1) {
        add = fix_mul(rest, t);
    } else if (way == 2 && rest > 0) {
        add = fix_mul((fix_div(fix_mul(rest, t), terrace) >> 16u) << 16u, terrace);
    } else {
        add = fix_mul(rest, eased);
    }
    out.ok = true;
    out.z = shore + kept + add;
    return out;
}

fn compressed_bank(north: i32, east: i32, ground: i32) -> Bank {
    let widest = h(H_BANK_WIDEST_EITHER);
    var bank = 2147483647;
    var hold = -2147483647 - 1;
    var rim = -2147483647 - 1;
    var s_reach = 0xFFFFFFFFu;
    var s_k = 0u;
    var s_distance = 2147483647;
    var s_has = false;
    var s_target = 0;
    var s_held = 0;
    let cell = grid_cell(north, east);
    let first = u32(data[hu(H_GRID_CELLS) + cell * 2u]);
    let count = u32(data[hu(H_GRID_CELLS) + cell * 2u + 1u]);
    for (var e = 0u; e < count; e++) {
        let r = u32(data[hu(H_GRID_ENTRIES) + (first + e) * 2u]);
        let k = u32(data[hu(H_GRID_ENTRIES) + (first + e) * 2u + 1u]);
        let a = point_at(r, k);
        let c = point_at(r, k + 1u);
        let al = along(a, c, north, east);
        let distance = al.x;
        let t = al.z;
        let width = a.width + fix_mul(c.width - a.width, t);
        let fade = a.fade + fix_mul(c.fade - a.fade, t);
        let half = half_across(a, c, width);
        var bend = c.bend;
        if (t < 32768) {
            bend = a.bend;
        }
        var shift = 0;
        if (whole_number(width) % 2 == 0) {
            shift = fix_mul(bend, 32768);
        }
        let beyond = abs(fix_mul(distance, al.y) - shift) - half;
        let past = fix_mul(t, fix_hypot(pos_to_fix(c.north - a.north), pos_to_fix(c.east - a.east)));
        var level = rounded(a.level);
        if (a.has_curtain && past < 131072) {
            level = max(rounded(a.curtain), rounded(a.level));
        }
        if (beyond > 0 && beyond <= 65536) {
            rim = max(rim, level - 65536);
        }
        var say_has = false;
        var say_target = 0;
        var say_held = 0;
        if (fade > 0 && beyond > 0 && beyond <= fix_mul(fade, widest) + 131072) {
            let b = bank_compressed(level, fade, beyond, ground);
            if (b.ok) {
                say_has = true;
                say_target = b.z;
                say_held = level - 65536 - fix_mul(max(beyond - 65536, 0), max(h(H_BANK_STEEPEST), 3277));
            }
        }
        if (r != s_reach || k > s_k + 2u) {
            if (s_has) {
                bank = min(bank, s_target);
                hold = max(hold, s_held);
            }
            s_reach = r;
            s_k = k;
            s_distance = distance;
            s_has = say_has;
            s_target = say_target;
            s_held = say_held;
        } else if (distance < s_distance) {
            s_reach = r;
            s_k = k;
            s_distance = distance;
            s_has = say_has;
            s_target = say_target;
            s_held = say_held;
        } else {
            s_k = k;
        }
    }
    if (s_has) {
        bank = min(bank, s_target);
        hold = max(hold, s_held);
    }
    var out: Bank;
    out.ok = bank < 2147483647;
    out.z = max(max(bank, min(hold, ground)), rim);
    return out;
}

// The lake water at a spot: x the surface, y 1 if only beside it; `ok` in z (0 none).
fn lake_at(north: i32, east: i32) -> vec3<i32> {
    if (h(H_LAKE_ACROSS_X) == 0) {
        return vec3<i32>(0, 0, 0);
    }
    let cell = max(h(H_LAKE_CELL), 1);
    let col = clamp(div_floor(east - h(H_LAKE_WEST), cell), 0, h(H_LAKE_ACROSS_X) - 1);
    let row = clamp(div_floor(h(H_LAKE_NORTH) - north, cell), 0, h(H_LAKE_ACROSS_Y) - 1);
    let at = hu(H_LAKE_CELLS) + u32(row * h(H_LAKE_ACROSS_X) + col) * 2u;
    let first = u32(data[at]);
    let count = u32(data[at + 1u]);
    var water_has = false;
    var water = 0;
    var beside_has = false;
    var beside = 0;
    for (var e = 0u; e < count; e++) {
        let lake = hu(H_LAKES) + u32(data[hu(H_LAKE_ENTRIES) + first + e]) * 7u;
        let c_raw = east - data[lake + 1u];
        let r_raw = data[lake] - north;
        if (c_raw < 0 || r_raw < 0) {
            continue;
        }
        let lake_cell = max(data[lake + 4u], 1);
        let c = c_raw / lake_cell;
        let r = r_raw / lake_cell;
        if (c >= data[lake + 2u] || r >= data[lake + 3u]) {
            continue;
        }
        let i = u32(r * data[lake + 2u] + c);
        let s = data[u32(data[lake + 5u]) + i];
        let bs = data[u32(data[lake + 6u]) + i];
        if (s != -2147483647 - 1) {
            if (water_has) {
                water = max(water, s);
            } else {
                water = s;
            }
            water_has = true;
        }
        if (bs != -2147483647 - 1) {
            if (beside_has) {
                beside = max(beside, bs);
            } else {
                beside = bs;
            }
            beside_has = true;
        }
    }
    if (water_has) {
        return vec3<i32>(water, 0, 1);
    }
    if (beside_has) {
        return vec3<i32>(beside, 1, 1);
    }
    return vec3<i32>(0, 0, 0);
}

// What the rivers and lakes do: floor (x, has in bit 1 of w), fill (y, bit 2), water (z, bit 4).
fn carve_here(north: i32, east: i32, ground: i32) -> vec4<i32> {
    var floor = 0;
    var fill = 0;
    var water = 0;
    var has = 0;
    if (!under_the_sea(ground)) {
        let piece = nearest_piece(north, east);
        if (piece.ok) {
            let point = piece.point;
            var shift = 0;
            if (whole_number(point.width) % 2 == 0) {
                shift = fix_mul(point.bend, 32768);
            }
            let offset = fix_mul(piece.distance, piece.side) - shift;
            var fall = 0;
            if (point.has_curtain) {
                fall = max(rounded(point.curtain), rounded(point.level));
            }
            if (abs(offset) <= piece.half) {
                let level = rounded(point.level);
                var top = level;
                if (point.has_curtain && h(H_CURTAINS) != 0 && piece.past < 65536) {
                    top = fall;
                }
                floor = level - 65536 - depth_across(point, fix_div(offset, piece.half));
                water = top >> 16u;
                has = has | 1 | 4;
            } else if (h(H_BANK_COMPRESSOR) != 0) {
                let b = compressed_bank(north, east, ground);
                if (b.ok) {
                    if (b.z < ground) {
                        floor = b.z;
                        has = has | 1;
                    } else {
                        fill = b.z;
                        has = has | 2;
                    }
                }
            } else if (point.fade > 0) {
                var level = rounded(point.level);
                if (point.has_curtain && piece.past < 131072) {
                    level = fall;
                }
                let b = bank_ground(level, point.fade, abs(offset) - piece.half, ground);
                if (b.ok) {
                    if (b.z < ground) {
                        floor = b.z;
                        has = has | 1;
                    } else {
                        fill = b.z;
                        has = has | 2;
                    }
                }
            }
        }
    }
    let lake = lake_at(north, east);
    if (lake.z != 0 && lake.x > SEA_Z) {
        let top = rounded(lake.x);
        var cut = ground;
        if ((has & 1) != 0) {
            cut = min(ground, floor);
        }
        var carved = cut;
        if ((has & 2) != 0) {
            carved = max(cut, fill);
        }
        if (lake.y != 0) {
            var against = ground;
            if ((has & 4) == 0) {
                against = carved;
            }
            if (against < top - 65536) {
                fill = top - 65536;
                has = has | 2;
            }
        } else if (carved < top - 32768) {
            if ((has & 4) != 0) {
                water = max(water, top >> 16u);
            } else {
                water = top >> 16u;
            }
            has = (has | 4) & ~2;
        }
    }
    return vec4<i32>(floor, fill, water, has);
}

// The ground carved, and the water over it: z (x), water (y, 0 none); `ok` false past the map.
struct Column {
    ok: bool,
    z: i32,
    water: i32,
}

fn ground_and_water(north: i32, east: i32) -> Column {
    return carved_from(north, east, uncarved(north, east));
}

// The rivers and lakes carved into a ground already worked out.
fn carved_from(north: i32, east: i32, u: Height) -> Column {
    var out: Column;
    out.ok = u.ok;
    out.water = 0;
    if (!u.ok) {
        return out;
    }
    out.z = u.z;
    if (muted(512) || h(H_CARVE_FULL) == 0) {
        return out;
    }
    let c = carve_here(north, east, u.z);
    var z = u.z;
    if ((c.w & 1) != 0) {
        z = min(z, c.x);
    }
    if ((c.w & 2) != 0) {
        z = max(z, c.y);
    }
    out.z = z;
    if ((c.w & 4) != 0 && c.z > whole_number(z) + 1) {
        out.water = c.z;
    }
    return out;
}

// ── Mineralogy (`Mineralogy::whole_layers`, the placer's `WholeProgram` and `WholeReading`) ──

// 64-bit numbers as (low, high) halves, for the planet hash.
fn u64_mul(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let low = wide_mul(a.x, b.x);
    return vec2<u32>(low.x, low.y + a.x * b.y + a.y * b.x);
}

fn u64_shr(a: vec2<u32>, n: u32) -> vec2<u32> {
    return vec2<u32>((a.x >> n) | (a.y << (32u - n)), a.y >> n);
}

fn split_mix(z_in: vec2<u32>) -> vec2<u32> {
    var z = u64_mul(z_in ^ u64_shr(z_in, 30u), vec2<u32>(0x1CE4E5B9u, 0xBF58476Du));
    z = u64_mul(z ^ u64_shr(z, 27u), vec2<u32>(0x133111EBu, 0x94D049BBu));
    return z ^ u64_shr(z, 31u);
}

// `fate::planet_hash`, from its start (the seed and salt mixed, on the processor) and a spot.
fn planet_hash_from(start: vec2<u32>, x: i32, y: i32) -> vec2<u32> {
    let k = vec2<u32>(0x7F4A7C15u, 0x9E3779B9u);
    var h = start;
    for (var i = 0; i < 3; i++) {
        var v = 0;
        if (i == 0) {
            v = x;
        } else if (i == 1) {
            v = y;
        }
        var wide = vec2<u32>(bitcast<u32>(v), 0u);
        if (v < 0) {
            wide.y = 0xFFFFFFFFu;
        }
        h = split_mix(h ^ u64_mul(wide, k));
    }
    return h;
}

fn climate_at(north: i32, east: i32, z: i32) -> vec3<i32> {
    if (h(H_HAS_CLIMATE) == 0) {
        return vec3<i32>(0, 0, 0);
    }
    let w = h(H_WIDTH);
    let half_pixels = w * 32768;
    let col = pos_ratio(east, h(H_BLOCKS)) + half_pixels - 32768;
    let row = half_pixels - pos_ratio(north, h(H_BLOCKS)) - 32768;
    let c0 = col >> 16u;
    let r0 = row >> 16u;
    let tc = col & 0xFFFF;
    let tr = row & 0xFFFF;
    var out: array<i32, 2>;
    for (var k = 0; k < 2; k++) {
        var field = hu(H_TEMPERATURE);
        if (k == 1) {
            field = hu(H_RAIN);
        }
        let a = field_at(field, c0, r0);
        let b = field_at(field, c0 + 1, r0);
        let c = field_at(field, c0, r0 + 1);
        let d = field_at(field, c0 + 1, r0 + 1);
        let top = a + fix_mul(b - a, tc);
        let bottom = c + fix_mul(d - c, tc);
        out[k] = top + fix_mul(bottom - top, tr);
    }
    let above_sea = max(z - SEA_Z, 0);
    return vec3<i32>(out[0] - fix_div(fix_mul(h(H_COOLING), above_sea), 100 << 16u), out[1], 1);
}

fn bare_point(north: i32, east: i32) -> Point {
    var p: Point;
    p.north = north;
    p.east = east;
    return p;
}

// How far a spot is from the nearest river's water (blocks), up to `most` (`WholeFarIndex::distance`).
fn river_distance(north: i32, east: i32, most: i32) -> i32 {
    if (h(H_FAR_ACROSS) == 0) {
        return most;
    }
    let cell = 128 * 256;
    let c = div_floor(east - h(H_FAR_EAST), cell);
    let r = div_floor(h(H_FAR_NORTH) - north, cell);
    var best = most;
    let cell_fix = 128 << 16u;
    let rings = (fix_div(most + h(H_FAR_WIDEST_HALF), cell_fix) >> 16u) + 2;
    for (var k = 0; k <= rings; k++) {
        if (fix_mul((k - 1) << 16u, cell_fix) - h(H_FAR_WIDEST_HALF) >= best) {
            break;
        }
        for (var dr = -k; dr <= k; dr++) {
            var step = max(2 * k, 1);
            if (abs(dr) == k) {
                step = 1;
            }
            for (var dc = -k; dc <= k; dc += step) {
                let cc = c + dc;
                let rr = r + dr;
                if (cc < 0 || rr < 0 || cc >= h(H_FAR_ACROSS) || rr >= h(H_FAR_DOWN)) {
                    continue;
                }
                let at = hu(H_FAR_CELLS) + u32(rr * h(H_FAR_ACROSS) + cc) * 2u;
                let first = u32(data[at]);
                let count = u32(data[at + 1u]);
                for (var e = 0u; e < count; e++) {
                    let piece = hu(H_FAR_ENTRIES) + (first + e) * 5u;
                    let al = along(bare_point(data[piece], data[piece + 1u]), bare_point(data[piece + 2u], data[piece + 3u]), north, east);
                    best = min(best, max(al.x - data[piece + 4u], 0));
                }
            }
        }
    }
    return best;
}

fn water_across(p: Point) -> i32 {
    let edge = clamp(p.edge, 0, 65536);
    let flat = clamp(p.flat, 0, 62259);
    let fullness = flat + fix_mul(65536 - flat, edge + fix_div(65536 - edge, max(p.bowl, 3277) + 65536));
    return fix_mul(max(p.width, 65536), max(fix_mul(p.depth, fullness), 65536));
}

// A column, as mineralogy reads it: with the maps there and the recipe's mountains and peaks, as the
// ground found them (`with_maps`), so they're worked out once a column, not again for its blocks.
struct Col {
    north: i32,
    east: i32,
    ground: i32,
    slope: i32,
    temperature: i32,
    rain: i32,
    water: i32,
    inside: bool,
    here: Here,
    mountains: i32,
    peaks: i32,
}

fn with_maps(col_in: Col) -> Col {
    var col = col_in;
    let m = maps_at(col.north, col.east);
    col.inside = m.ok;
    if (m.ok) {
        col.here = m.here;
        let p = parts(m.here, m.x, m.y);
        col.mountains = p.y;
        col.peaks = p.z;
    }
    return col;
}

fn held(p: i32) -> i32 {
    return clamp(p, -32767 * 256, 32767 * 256) << 8u;
}

fn column_value(field: i32, col: Col) -> i32 {
    if (field == 12) {
        return river_distance(col.north, col.east, 2000 << 16u);
    }
    if (field == 13) {
        let piece = nearest_piece(col.north, col.east);
        if (piece.ok) {
            return water_across(piece.point);
        }
        return 0;
    }
    if (field == 14) {
        var sea = 0;
        if (col.ground < SEA_Z) {
            sea = SEA_Z - 65536 - col.ground;
        }
        var river = 0;
        if (col.water != 0) {
            river = (col.water << 16u) - 65536 - col.ground;
        }
        return max(max(sea, river), 0);
    }
    if (field == 15) {
        return select(0, 65536, col.water != 0);
    }
    if (field == 16) {
        return col.temperature;
    }
    if (field == 17) {
        return col.rain;
    }
    if (field == 18) {
        return col.ground;
    }
    if (field == 19) {
        return col.slope;
    }
    if (!col.inside) {
        return 0;
    }
    let here = col.here;
    switch field {
        case 0: { return here.height; }
        case 1: { return here.tilt; }
        case 2: {
            if (here.squeeze >= 0) {
                return fix_div(here.squeeze, max(h(H_MOST_CRUSHED), TINY_FIX));
            }
            return fix_div(here.squeeze, max(-h(H_MOST_STRETCHED), TINY_FIX));
        }
        case 3: { return here.push_strength; }
        case 4: {
            let degrees = (fix_atan2(here.push_y, here.push_x) >> 1u) * 360;
            let round = 180 * 65536;
            return ((degrees % round) + round) % round;
        }
        case 5: { return held(here.coast_distance); }
        case 6: { return held(here.lift_distance); }
        case 7: { return mountain_share(here); }
        case 8: { return lifted(here); }
        case 9: { return raised_coast(here); }
        case 10: { return col.mountains; }
        case 11: { return col.peaks; }
        default: { return 0; }
    }
}

fn read_field(field: i32, col: Col, z: i32) -> i32 {
    if (field == 20) {
        return z;
    }
    if (field == 21) {
        return col.ground - z;
    }
    if (field == 22) {
        return fix_div(col.ground - z, max(h(H_CRUST), 65536));
    }
    return column_value(field, col);
}

// The made fields worked out so far, and what each distributor chose (its block; 0 nothing).
var<private> MADE: array<i32, 64>;
var<private> CHOSEN: array<i32, 16>;

fn mix_mode(mode: i32, so_far: i32, value: i32, strength: i32) -> i32 {
    switch mode {
        case 0: { return so_far + fix_mul(value - so_far, strength); }
        case 1: { return so_far + fix_mul(value, strength); }
        case 2: { return so_far - fix_mul(value, strength); }
        case 3: { return fix_mul(so_far, 65536 + fix_mul(value - 65536, strength)); }
        case 4: { return so_far + fix_mul(min(so_far, value) - so_far, strength); }
        default: { return so_far + fix_mul(max(so_far, value) - so_far, strength); }
    }
}

fn shaped(at: u32, x: i32) -> i32 {
    let kind = data[at + 8u];
    if (kind == 0) {
        return x;
    }
    if (kind == 1) {
        let mark = data[at + 9u];
        let soft = data[at + 11u];
        var t = 0;
        if (soft > 0) {
            t = fix_smooth(fix_div(x - mark + soft, soft * 2));
        } else if (x >= mark) {
            t = 65536;
        }
        if (data[at + 10u] != 0) {
            return t;
        }
        return 65536 - t;
    }
    if (kind == 2) {
        let middle = data[at + 9u];
        let spread = data[at + 10u];
        if (spread > 0) {
            return 65536 - fix_smooth(fix_div(abs(x - middle), spread));
        }
        return select(0, 65536, x == middle);
    }
    if (kind == 3) {
        let low = data[at + 14u];
        let high = data[at + 15u];
        return curve_at(u32(data[at + 9u]), clamp(fix_div(x - low, max(high - low, 1)), 0, 65536));
    }
    return 0;
}

fn column_noise(col: Col, key: u32, size: i32) -> i32 {
    return perlin_at(key, 0, pos_per(col.north, size), pos_per(col.east, size));
}

// A stack of the program (a made field, or a candidate's) at `z`.
fn stack_value(program: u32, k: u32, col: Col, z: i32) -> i32 {
    let table = u32(data[program + 1u]);
    let steps = u32(data[program + 2u]);
    let first = u32(data[table + k * 2u]);
    let count = u32(data[table + k * 2u + 1u]);
    var so_far = 0;
    for (var n = 0u; n < count; n++) {
        let at = steps + (first + n) * 16u;
        if (data[at] == 0) {
            continue;
        }
        let kind = data[at + 1u];
        let a = data[at + 2u];
        let b = data[at + 3u];
        var value = 0;
        if (kind == 0) {
            value = read_field(a, col, z);
        } else if (kind == 1) {
            value = column_noise(col, u32(b), a);
        } else if (kind == 2) {
            value = MADE[a];
        } else if (kind == 3) {
            value = so_far;
        } else if (kind == 4) {
            value = a;
        } else if (kind == 5) {
            let list = u32(b);
            let blocks = u32(data[list]);
            for (var i = 0u; i < blocks; i++) {
                if (CHOSEN[a] == data[list + 1u + i]) {
                    value = 65536;
                }
            }
        }
        if (data[at + 4u] != 0) {
            value = value + fix_mul(column_noise(col, u32(data[at + 7u]), data[at + 6u]), data[at + 5u]);
        }
        so_far = mix_mode(data[at + 12u], so_far, shaped(at, value), data[at + 13u]);
    }
    return so_far;
}

// What each distributor chose, as (its candidate + 1) in a byte each, the first in the lowest. (Every
// usable candidate's suitability is worked out once, in one place, before its distributor chooses: the
// compiler copies a function into every place it's called, and five calls of the stacks, each with every
// field the placer reads, made the recipe take a minute to compile. The answer is the same: the stacks
// read nothing that changes.)
fn choose_blocks(col: Col) -> i32 {
    let mineralogy = hu(H_MINERALOGY);
    let program = u32(data[mineralogy]);
    let distributors = u32(data[mineralogy + 1u]);
    let table = u32(data[mineralogy + 2u]);
    let fields = u32(data[mineralogy + 4u]);
    for (var j = 0u; j < fields; j++) {
        MADE[j] = stack_value(program, j, col, col.ground);
    }
    var choices = 0;
    var suits: array<i32, 32>;
    for (var d = 0u; d < distributors; d++) {
        let at = table + d * 7u;
        let count = min(u32(data[at + 3u]), 32u);
        let candidates = u32(data[at + 4u]);
        let on = data[at] != 0;
        // Each usable candidate's suitability (below 0: not usable, never chosen).
        for (var c = 0u; c < count; c++) {
            let cand = candidates + c * 4u;
            suits[c] = -2147483647 - 1;
            if (on && data[cand] != 0) {
                if (data[cand + 1u] != 0) {
                    suits[c] = 65536;
                } else {
                    suits[c] = stack_value(program, u32(data[cand + 2u]), col, col.ground);
                }
            }
        }
        var chosen = -1;
        let chooser = data[at + 1u];
        if (on && chooser == 0) {
            for (var c = 0u; c < count; c++) {
                if (suits[c] != -2147483647 - 1 && suits[c] >= data[at + 2u]) {
                    chosen = i32(c);
                    break;
                }
            }
        } else if (on && chooser == 1) {
            var best = 0;
            for (var c = 0u; c < count; c++) {
                if (suits[c] > 0 && (chosen < 0 || suits[c] > best)) {
                    chosen = i32(c);
                    best = suits[c];
                }
            }
        } else if (on && chooser == 2) {
            var total = 0;
            for (var c = 0u; c < count; c++) {
                total = total + max(suits[c], 0);
            }
            let hash = planet_hash_from(vec2<u32>(u32(data[at + 5u]), u32(data[at + 6u])), col.north >> 8u, col.east >> 8u);
            let product = wide_mul(hash.y >> 8u, bitcast<u32>(total));
            let roll = bitcast<i32>((product.x >> 24u) | (product.y << 8u));
            var sum = 0;
            var last = -1;
            for (var c = 0u; c < count; c++) {
                if (suits[c] <= 0) {
                    continue;
                }
                last = i32(c);
                sum = sum + suits[c];
                if (roll < sum && chosen < 0) {
                    chosen = i32(c);
                }
            }
            if (chosen < 0) {
                chosen = last;
            }
        }
        if (chosen >= 0) {
            CHOSEN[d] = data[candidates + u32(chosen) * 4u + 3u];
        } else {
            CHOSEN[d] = 0;
        }
        choices = choices | ((chosen + 1) << (8u * d));
    }
    return choices;
}

// ── A whole column, once (the readback's work: the ground, carved, and the blocks on it) ──

// The ground, carved (x), the water over it (y, 0 none), and what each distributor chose (z); `w` 0 past
// the map. The maps and the recipe's parts are worked out once, for both the ground and the blocks.
fn whole_column(north: i32, east: i32, slope: i32) -> vec4<i32> {
    let m = maps_at(north, east);
    if (!m.ok) {
        return vec4<i32>(0, 0, 0, 0);
    }
    let p = parts(m.here, m.x, m.y);
    let column = carved_from(north, east, uncarved_from(north, east, m, p));
    var col: Col;
    col.north = north;
    col.east = east;
    col.ground = rounded(column.z);
    col.slope = slope;
    col.water = column.water;
    let climate = climate_at(north, east, col.ground);
    col.temperature = 13107;
    col.rain = 39322;
    if (climate.z != 0) {
        col.temperature = climate.x;
        col.rain = climate.y;
    }
    col.inside = true;
    col.here = m.here;
    col.mountains = p.y;
    col.peaks = p.z;
    return vec4<i32>(column.z, column.water, choose_blocks(col), 1);
}

// ── The caves (`caves.rs`: `CavePlan::inside`, whole numbers already) ──

fn hash32_3(seed: u32, x: i32, y: i32, z: i32) -> u32 {
    var h = seed ^ 0x2545f491u;
    for (var i = 0; i < 3; i++) {
        var k = bitcast<u32>(x);
        if (i == 1) {
            k = bitcast<u32>(y);
        } else if (i == 2) {
            k = bitcast<u32>(z);
        }
        h = h ^ (rotl(k * 0xcc9e2d51u, 15u) * 0x1b873593u);
        h = rotl(h, 13u) * 5u + 0xe6546b64u;
    }
    h = h ^ (h >> 16u);
    h = h * 0x85ebca6bu;
    h = h ^ (h >> 13u);
    h = h * 0xc2b2ae35u;
    return h ^ (h >> 16u);
}

fn value3(seed: u32, x: i32, y: i32, z: i32) -> i32 {
    return i32(hash32_3(seed, x, y, z) >> 15u) - 65536;
}

fn rem_floor(a: i32, b: i32) -> i32 {
    return a - div_floor(a, b) * b;
}

// `whole_value3`: value noise at a block, one lump every `size` blocks.
fn whole_value3(seed: u32, bx: i32, by: i32, bz: i32, size_in: i32) -> i32 {
    let size = max(size_in, 1);
    let ix = div_floor(bx, size);
    let iy = div_floor(by, size);
    let iz = div_floor(bz, size);
    let u = whole_fade(0, rem_floor(bx, size) * 65536 / size);
    let v = whole_fade(0, rem_floor(by, size) * 65536 / size);
    let w = whole_fade(0, rem_floor(bz, size) * 65536 / size);
    var planes: array<i32, 2>;
    for (var k = 0; k < 2; k++) {
        let z = iz + k;
        planes[k] = noise_mix(noise_mix(value3(seed, ix, iy, z), value3(seed, ix + 1, iy, z), u), noise_mix(value3(seed, ix, iy + 1, z), value3(seed, ix + 1, iy + 1, z), u), v);
    }
    return noise_mix(planes[0], planes[1], w);
}

const MOST_ACROSS: i32 = 128;

// A tunnel: first point, points, rough, lump, floor, seed. A point: x, y, z (16ths), width, height.
fn inside_piece(tunnel: u32, p: u32, bx: i32, by: i32, bz: i32) -> bool {
    let t = hu(H_TUNNELS) + tunnel * 6u;
    let first = u32(data[t]);
    let rough = data[t + 2u];
    let lump = data[t + 3u];
    let floor_share = data[t + 4u];
    let seed = u32(data[t + 5u]);
    let a = hu(H_CAVE_POINTS) + (first + p) * 5u;
    let b = a + 5u;
    let at = vec3<i32>(bx * 16 + 8, by * 16 + 8, bz * 16 + 8);
    let line = vec3<i32>(data[b] - data[a], data[b + 1u] - data[a + 1u], data[b + 2u] - data[a + 2u]);
    let off_a = at - vec3<i32>(data[a], data[a + 1u], data[a + 2u]);
    let reach = max(max(data[a + 3u], data[a + 4u]), max(data[b + 3u], data[b + 4u])) + abs(rough);
    for (var k = 0; k < 3; k++) {
        if (off_a[k] < min(line[k], 0) - reach || off_a[k] > max(line[k], 0) + reach) {
            return false;
        }
    }
    let length = line.x * line.x + line.y * line.y + line.z * line.z;
    var along = 0;
    if (length != 0) {
        along = clamp((off_a.x * line.x + off_a.y * line.y + off_a.z * line.z) * 256 / length, 0, 256);
    }
    let off = off_a - line * along / 256;
    let bump = rough * whole_value3(seed, bx, by, bz, lump) / 65536;
    let width = clamp(data[a + 3u] + (data[b + 3u] - data[a + 3u]) * along / 256 + bump, 0, MOST_ACROSS);
    let height = clamp(data[a + 4u] + (data[b + 4u] - data[a + 4u]) * along / 256 + bump, 0, MOST_ACROSS);
    if (width == 0 || height == 0 || off.z < -(height * floor_share / 256)) {
        return false;
    }
    let across = off.x * off.x + off.y * off.y;
    let up = off.z * off.z;
    let wide = width * width;
    let tall = height * height;
    return across <= wide && up <= tall && across * tall + up * wide <= wide * tall;
}

// The box's place in the caves' hash table (as the packer's `cave_slot`).
fn cave_slot(cx: i32, cy: i32, level: i32) -> u32 {
    return ((bitcast<u32>(cx) * 73856093u) ^ (bitcast<u32>(cy) * 19349663u) ^ (bitcast<u32>(level) * 83492791u)) & (hu(H_CAVE_SLOTS) - 1u);
}

// Is this block inside a cave? (`CavePlan::inside`)
fn cave_inside(bx: i32, by: i32, bz: i32) -> bool {
    if (h(H_CAVE_SLOTS) == 0) {
        return false;
    }
    let cx = div_floor(bx, 32);
    let cy = div_floor(by, 32);
    let level = div_floor(bz, 32);
    var slot = cave_slot(cx, cy, level);
    let mask = hu(H_CAVE_SLOTS) - 1u;
    for (var probe = 0u; probe <= mask; probe++) {
        let at = hu(H_CAVE_TABLE) + slot * 6u;
        if (data[at] == 0) {
            return false;
        }
        if (data[at + 1u] == cx && data[at + 2u] == cy && data[at + 3u] == level) {
            let first = u32(data[at + 4u]);
            let count = u32(data[at + 5u]);
            for (var e = 0u; e < count; e++) {
                let entry = hu(H_CAVE_PIECES) + (first + e) * 2u;
                if (inside_piece(u32(data[entry]), u32(data[entry + 1u]), bx, by, bz)) {
                    return true;
                }
            }
            return false;
        }
        slot = (slot + 1u) & mask;
    }
    return false;
}

// A column as the world fills it (`terrain::whole_block_column`): its ground in whole blocks (halves up,
// at least 1, at most 1,023; the sea floor at least two under the sea's surface), and the water over it (0
// none, at most 1,024): x ground, y water, z 1 inside the map (0 past it).
fn world_column(north: i32, east: i32) -> vec3<i32> {
    let column = ground_and_water(north, east);
    if (!column.ok) {
        return vec3<i32>(0, 0, 0);
    }
    var z = clamp((column.z + 32768) >> 16u, 1, 1023);
    var water = 0;
    if (column.water > z + 1) {
        water = min(column.water, 1024);
    }
    if (z < 128) {
        z = min(z, 126);
    }
    return vec3<i32>(z, water, 1);
}

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
    let p = parts(here, x, y);
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
    var out: Height;
    let m = maps_at(north, east);
    if (!m.ok) {
        out.ok = false;
        return out;
    }
    out.ok = true;
    let ground = put_together(m.here, m.x, m.y);
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

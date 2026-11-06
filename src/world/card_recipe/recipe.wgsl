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

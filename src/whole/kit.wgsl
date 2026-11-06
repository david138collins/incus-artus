// The whole-number kit, the card's copy (src/whole/mod.rs is the processor's, and says what each step
// does; the twin test runs both and compares them bit for bit). A Fix is an i32 in 65536ths. Shaders
// have no 64-bit numbers, so the wide steps are built from 32-bit pieces, as (low, high) pairs. The
// tables (LOG2, EXP2, SIN, ATAN) are put in front of this by `shader::source`, from the processor's.

fn abs_u(a: i32) -> u32 {
    if (a < 0) {
        return ~bitcast<u32>(a) + 1u;
    }
    return bitcast<u32>(a);
}

// u32 × u32, all 64 bits of it.
fn wide_mul(a: u32, b: u32) -> vec2<u32> {
    let a0 = a & 0xFFFFu;
    let a1 = a >> 16u;
    let b0 = b & 0xFFFFu;
    let b1 = b >> 16u;
    let p00 = a0 * b0;
    let p01 = a0 * b1;
    let p10 = a1 * b0;
    let p11 = a1 * b1;
    let mid = (p00 >> 16u) + (p01 & 0xFFFFu) + (p10 & 0xFFFFu);
    let lo = (p00 & 0xFFFFu) | (mid << 16u);
    let hi = p11 + (p01 >> 16u) + (p10 >> 16u) + (mid >> 16u);
    return vec2<u32>(lo, hi);
}

fn wide_neg(v: vec2<u32>) -> vec2<u32> {
    let lo = ~v.x + 1u;
    return vec2<u32>(lo, ~v.y + select(0u, 1u, lo == 0u));
}

fn wide_add(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x + b.x;
    return vec2<u32>(lo, a.y + b.y + select(0u, 1u, lo < a.x));
}

fn wide_le(a: vec2<u32>, b: vec2<u32>) -> bool {
    return a.y < b.y || (a.y == b.y && a.x <= b.x);
}

// A 64-bit number over a 32-bit one (at most 2^31), rounded down, by long division.
fn wide_div(n: vec2<u32>, d: u32) -> vec2<u32> {
    var q = vec2<u32>(0u, 0u);
    var r = 0u;
    for (var i = 63i; i >= 0; i--) {
        var bit: u32;
        if (i >= 32) {
            bit = (n.y >> u32(i - 32)) & 1u;
        } else {
            bit = (n.x >> u32(i)) & 1u;
        }
        let carry = (r >> 31u) == 1u;
        r = (r << 1u) | bit;
        if (carry || r >= d) {
            r = r - d;
            if (i >= 32) {
                q.y = q.y | (1u << u32(i - 32));
            } else {
                q.x = q.x | (1u << u32(i));
            }
        }
    }
    return q;
}

// The square root of a 64-bit number, rounded down, bit by bit.
fn wide_sqrt(n: vec2<u32>) -> u32 {
    var r = 0u;
    for (var bit = 31i; bit >= 0; bit--) {
        let c = r | (1u << u32(bit));
        if (wide_le(wide_mul(c, c), n)) {
            r = c;
        }
    }
    return r;
}

fn fix_mul(a: i32, b: i32) -> i32 {
    var p = wide_mul(abs_u(a), abs_u(b));
    if ((a < 0) != (b < 0)) {
        p = wide_neg(p);
    }
    return bitcast<i32>((p.x >> 16u) | (p.y << 16u));
}

fn fix_div(a: i32, b: i32) -> i32 {
    let top = abs_u(a);
    let bottom = abs_u(b);
    var size = 0x7FFFFFFFu;
    if (bottom != 0u) {
        let q = wide_div(vec2<u32>(top << 16u, top >> 16u), bottom);
        if (q.y == 0u && q.x <= 0x7FFFFFFFu) {
            size = q.x;
        }
    }
    let s = bitcast<i32>(size);
    if ((a < 0) != (b < 0)) {
        return -s;
    }
    return s;
}

// A position over a size, in Fix units, wrapping (`Pos::per`): the low 32 bits of the quotient.
fn pos_per(a: i32, size: i32) -> i32 {
    if (size == 0) {
        return 0;
    }
    let top = abs_u(a);
    let q = wide_div(vec2<u32>(top << 16u, top >> 16u), abs_u(size));
    let units = bitcast<i32>(q.x);
    if ((a < 0) != (size < 0)) {
        return -units;
    }
    return units;
}

fn fix_sqrt(x: i32) -> i32 {
    if (x <= 0) {
        return 0;
    }
    let raw = bitcast<u32>(x);
    return bitcast<i32>(wide_sqrt(vec2<u32>(raw << 16u, raw >> 16u)));
}

fn fix_hypot(x: i32, y: i32) -> i32 {
    let ax = abs_u(x);
    let ay = abs_u(y);
    let r = wide_sqrt(wide_add(wide_mul(ax, ax), wide_mul(ay, ay)));
    return bitcast<i32>(min(r, 0x7FFFFFFFu));
}

fn between_log2(k: u32, t: i32) -> i32 {
    return LOG2[k] + (((LOG2[k + 1u] - LOG2[k]) * t) >> 16u);
}

fn between_exp2(k: u32, t: i32) -> i32 {
    return EXP2[k] + (((EXP2[k + 1u] - EXP2[k]) * t) >> 16u);
}

fn between_sin(k: u32, t: i32) -> i32 {
    return SIN[k] + (((SIN[k + 1u] - SIN[k]) * t) >> 16u);
}

fn between_atan(k: u32, t: i32) -> i32 {
    return ATAN[k] + (((ATAN[k + 1u] - ATAN[k]) * t) >> 16u);
}

fn fix_log2(x: i32) -> i32 {
    if (x <= 0) {
        return -2147483647 - 1;
    }
    let raw = bitcast<u32>(x);
    let top = 31 - i32(countLeadingZeros(raw));
    let m = raw << u32(31 - top);
    return ((top - 16) << 16u) + between_log2((m >> 23u) & 0xFFu, i32((m >> 7u) & 0xFFFFu));
}

fn fix_exp2(x: i32) -> i32 {
    let whole = x >> 16u;
    let part = x & 0xFFFF;
    let m = between_exp2(u32(part >> 8u), (part & 0xFF) << 8u);
    if (whole >= 15) {
        return 2147483647;
    }
    if (whole >= 0) {
        return m << u32(whole);
    }
    if (whole > -31) {
        return m >> u32(-whole);
    }
    return 0;
}

fn fix_pow(x: i32, e: i32) -> i32 {
    if (x <= 0) {
        return 0;
    }
    return fix_exp2(fix_mul(e, fix_log2(x)));
}

fn fix_sin(angle: i32) -> i32 {
    let a = angle & 0xFFFF;
    return between_sin(u32(a >> 6u), (a & 63) << 10u);
}

fn fix_cos(angle: i32) -> i32 {
    return fix_sin(angle + 16384);
}

fn fix_atan2(y: i32, x: i32) -> i32 {
    let ax = abs_u(x);
    let ay = abs_u(y);
    if (ax == 0u && ay == 0u) {
        return 0;
    }
    var small = ax;
    var large = ay;
    if (ay <= ax) {
        small = ay;
        large = ax;
    }
    let r = bitcast<i32>(wide_div(vec2<u32>(small << 16u, small >> 16u), large).x);
    var a: i32;
    if (r >= 65536) {
        a = ATAN[256];
    } else {
        a = between_atan(u32(r >> 8u), (r & 0xFF) << 8u);
    }
    if (ay > ax) {
        a = 16384 - a;
    }
    if (x < 0) {
        a = 32768 - a;
    }
    if (y < 0) {
        return -a;
    }
    return a;
}

fn fix_smooth(t: i32) -> i32 {
    let c = clamp(t, 0, 65536);
    return fix_mul(fix_mul(c, c), (3 << 16u) - fix_mul(2 << 16u, c));
}

fn fix_mix(a: i32, b: i32, t: i32) -> i32 {
    return a + fix_mul(b - a, t);
}

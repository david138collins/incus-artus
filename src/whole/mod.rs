//! Whole-number maths, the same on every processor and every graphics card (the regroup, phase 2.1;
//! `decisions.md`, "Projected code is GPU-safe": code that makes the world in play "uses whole numbers
//! and fixed point, and no sin, cos, pow or other maths that varies by chip, driver or operating
//! system. Then the CPU and the GPU agree bit for bit").
//!
//! **Why not fractions?** A card may fuse a multiply and an add into one step (rounding once instead of
//! twice), and its division and square root aren't rounded exactly. A difference in the last bit is
//! enough to put the top of a column in a different block. Whole numbers come out the same everywhere.
//!
//! **`Fix`** is a number in 65536ths (16 bits of whole, 16 of fraction: about ±32,768, in steps of
//! 0.000015), in 32 bits. Every step here is defined exactly, so the shader's copy (`shader`, built from
//! 32-bit pieces, since shaders have no 64-bit numbers) gives the same bits; the twin test runs both
//! and compares them.
//! - `+ − ×` wrap round past the range (as a shader's do); `×` rounds down (towards −∞).
//! - `÷` rounds towards zero, and holds at the range's end rather than wrapping (also ÷ 0).
//! - `sqrt` and `hypot` round down.
//! - `log2`, `exp2`, `pow`, `sin`, `cos`, `atan2` read tables (`tables`) and draw a straight line between
//!   their entries: within about 1/10,000 of the true value.
//! - Angles are in 65536ths of a turn.
//! - **`Pos`** is a position (blocks north, east, or across the map) in 256ths of a block: a world is
//!   72,000 blocks across, past `Fix`'s reach, and `Pos` holds about ±8 million. `Pos × Fix` is a `Pos`
//!   (rounded down); `Pos ÷ Pos` is a `Fix` (`ratio`, as `Fix ÷`); `per` is a position over a size, in
//!   `Fix` units, wrapping past ±32,768 units (as the noise's grid already repeats there).
//!
//! The only fractions are the way in (`Fix::from_f32`, for settings, once, on the processor) and the way
//! out (`to_f32`, exact).

#[allow(dead_code)] // the card's copy, read by the card's recipe (phase 2.3) and the twin test
pub mod shader;
mod tables;

use tables::{ATAN, EXP2, LOG2, SIN};

/// A number in 65536ths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fix(pub i32);

/// 1, in 65536ths.
pub const ONE: i32 = 65_536;
/// A whole turn, in angles.
pub const TURN: i32 = 65_536;

#[allow(dead_code)] // the rest are used as the recipe is converted (phase 2.2)
impl Fix {
    pub const ZERO: Fix = Fix(0);
    pub const ONE: Fix = Fix(ONE);
    pub const MAX: Fix = Fix(i32::MAX);
    pub const MIN: Fix = Fix(i32::MIN);

    /// From a fraction (settings, once, on the processor: never in projected code), rounded to the
    /// nearest 65536th, held at the range's ends.
    pub fn from_f32(x: f32) -> Fix {
        Fix((x as f64 * ONE as f64).round().clamp(i32::MIN as f64, i32::MAX as f64) as i32)
    }

    pub const fn from_int(n: i32) -> Fix {
        Fix(n.wrapping_shl(16))
    }

    /// Back to a fraction, exactly (a power of two).
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / ONE as f32
    }

    /// The whole part, rounded down.
    pub const fn floor(self) -> i32 {
        self.0 >> 16
    }

    pub const fn abs(self) -> Fix {
        Fix(self.0.wrapping_abs())
    }

    pub fn min(self, other: Fix) -> Fix {
        if other.0 < self.0 { other } else { self }
    }

    pub fn max(self, other: Fix) -> Fix {
        if other.0 > self.0 { other } else { self }
    }

    pub fn clamp(self, low: Fix, high: Fix) -> Fix {
        self.max(low).min(high)
    }
}

impl std::ops::Add for Fix {
    type Output = Fix;
    fn add(self, other: Fix) -> Fix {
        Fix(self.0.wrapping_add(other.0))
    }
}

impl std::ops::Sub for Fix {
    type Output = Fix;
    fn sub(self, other: Fix) -> Fix {
        Fix(self.0.wrapping_sub(other.0))
    }
}

impl std::ops::Neg for Fix {
    type Output = Fix;
    fn neg(self) -> Fix {
        Fix(self.0.wrapping_neg())
    }
}

impl std::ops::Mul for Fix {
    type Output = Fix;
    /// Rounded down: the 64-bit product's bits 16 to 47.
    fn mul(self, other: Fix) -> Fix {
        Fix(((self.0 as i64 * other.0 as i64) >> 16) as i32)
    }
}

impl std::ops::Div for Fix {
    type Output = Fix;
    /// Rounded towards zero, held at ±(2³¹ − 1) rather than wrapping (and for ÷ 0).
    fn div(self, other: Fix) -> Fix {
        let negative = (self.0 < 0) != (other.0 < 0);
        let (top, bottom) = (self.0.unsigned_abs() as u64, other.0.unsigned_abs() as u64);
        let size = if bottom == 0 { u64::MAX } else { (top << 16) / bottom };
        let size = size.min(i32::MAX as u64) as i32;
        Fix(if negative { -size } else { size })
    }
}

/// The square root, rounded down (0 for 0 and below).
#[allow(dead_code)] // in the kit for the recipe's use; the twin test keeps it honest
pub fn sqrt(x: Fix) -> Fix {
    if x.0 <= 0 {
        return Fix::ZERO;
    }
    Fix(((x.0 as u64) << 16).isqrt() as i32)
}

/// A position, in 256ths of a block.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(pub i32);

/// A block, in positions.
pub const BLOCK: i32 = 256;

#[allow(dead_code)] // used as the recipe is converted (phase 2.2)
impl Pos {
    /// From a fraction (settings and maps, once, on the processor), rounded to the nearest 256th.
    pub fn from_f32(x: f32) -> Pos {
        Pos((x as f64 * BLOCK as f64).round().clamp(i32::MIN as f64, i32::MAX as f64) as i32)
    }

    /// The middle of block `n` (a column's middle, as the world is filled).
    pub const fn middle_of(n: i32) -> Pos {
        Pos(n.wrapping_mul(BLOCK).wrapping_add(BLOCK / 2))
    }

    pub fn to_f32(self) -> f32 {
        self.0 as f32 / BLOCK as f32
    }

    /// As a `Fix` (exact while it's within ±32,768 blocks).
    pub const fn to_fix(self) -> Fix {
        Fix(self.0.wrapping_shl(8))
    }

    /// × a `Fix`, rounded down.
    pub fn scaled(self, by: Fix) -> Pos {
        Pos(((self.0 as i64 * by.0 as i64) >> 16) as i32)
    }

    /// This over `other`, as a `Fix` (held at the range's end).
    pub fn ratio(self, other: Pos) -> Fix {
        Fix(self.0) / Fix(other.0)
    }

    /// This over a size, in `Fix` units, wrapping past ±32,768 (the noise's coordinates); 0 for a size
    /// of 0.
    pub fn per(self, size: Pos) -> Fix {
        if size.0 == 0 {
            return Fix::ZERO;
        }
        let negative = (self.0 < 0) != (size.0 < 0);
        let units = (((self.0.unsigned_abs() as u64) << 16) / size.0.unsigned_abs() as u64) as u32 as i32;
        Fix(if negative { units.wrapping_neg() } else { units })
    }

    /// How far (x, y) is from (0, 0), rounded down.
    pub fn hypot(self, other: Pos) -> Pos {
        Pos(hypot(Fix(self.0), Fix(other.0)).0)
    }
}

#[allow(dead_code)] // used as the recipe is converted (phase 2.2)
impl Fix {
    /// As a position (rounded down).
    pub const fn to_pos(self) -> Pos {
        Pos(self.0 >> 8)
    }
}

impl std::ops::Add for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos(self.0.wrapping_add(other.0))
    }
}

impl std::ops::Sub for Pos {
    type Output = Pos;
    fn sub(self, other: Pos) -> Pos {
        Pos(self.0.wrapping_sub(other.0))
    }
}

impl std::ops::Neg for Pos {
    type Output = Pos;
    fn neg(self) -> Pos {
        Pos(self.0.wrapping_neg())
    }
}

/// √(x² + y²), rounded down, held at the range's end.
pub fn hypot(x: Fix, y: Fix) -> Fix {
    let (x, y) = (x.0.unsigned_abs() as u64, y.0.unsigned_abs() as u64);
    Fix((x * x + y * y).isqrt().min(i32::MAX as u64) as i32)
}

/// A straight line between `table[k]` and `table[k + 1]`, `t` (0 to 65535) of the way.
fn between(table: &[i32], k: usize, t: i32) -> i32 {
    table[k] + (((table[k + 1] - table[k]) * t) >> 16)
}

/// log₂ (for 0 and below, the range's bottom).
pub fn log2(x: Fix) -> Fix {
    if x.0 <= 0 {
        return Fix::MIN;
    }
    let raw = x.0 as u32;
    let top = 31 - raw.leading_zeros() as i32;
    // The mantissa, its top bit at bit 31: the table's entry in the next 8 bits, then 16 between.
    let m = raw << (31 - top);
    let (k, t) = (((m >> 23) & 0xFF) as usize, ((m >> 7) & 0xFFFF) as i32);
    Fix(((top - 16) << 16) + between(&LOG2, k, t))
}

/// 2 to the power `x` (held at the range's end; 0 far below).
pub fn exp2(x: Fix) -> Fix {
    let whole = x.0 >> 16;
    let part = x.0 & 0xFFFF;
    let m = between(&EXP2, (part >> 8) as usize, (part & 0xFF) << 8);
    Fix(if whole >= 15 {
        i32::MAX
    } else if whole >= 0 {
        m << whole
    } else if whole > -31 {
        m >> -whole
    } else {
        0
    })
}

/// `x` to the power `e` (for `x` above 0; 0 otherwise).
pub fn pow(x: Fix, e: Fix) -> Fix {
    if x.0 <= 0 {
        return Fix::ZERO;
    }
    exp2(e * log2(x))
}

/// The sine of an angle (in 65536ths of a turn).
pub fn sin(angle: i32) -> Fix {
    let a = angle & 0xFFFF;
    Fix(between(&SIN, (a >> 6) as usize, (a & 63) << 10))
}

pub fn cos(angle: i32) -> Fix {
    sin(angle.wrapping_add(TURN / 4))
}

/// The angle of (x, y), in 65536ths of a turn, from −½ to ½ (0 for (0, 0)).
pub fn atan2(y: Fix, x: Fix) -> i32 {
    let (ax, ay) = (x.0.unsigned_abs(), y.0.unsigned_abs());
    if ax == 0 && ay == 0 {
        return 0;
    }
    // The angle in the first eighth of a turn, from the smaller over the larger (0 to 1).
    let (small, large) = if ay <= ax { (ay, ax) } else { (ax, ay) };
    let r = (((small as u64) << 16) / large as u64) as i32;
    let mut a = if r >= ONE { ATAN[256] } else { between(&ATAN, (r >> 8) as usize, (r & 0xFF) << 8) };
    if ay > ax {
        a = TURN / 4 - a;
    }
    if x.0 < 0 {
        a = TURN / 2 - a;
    }
    if y.0 < 0 { -a } else { a }
}

/// 3t² − 2t³ for t from 0 to 1 (held there).
pub fn smooth(t: Fix) -> Fix {
    let t = t.clamp(Fix::ZERO, Fix::ONE);
    t * t * (Fix::from_int(3) - Fix::from_int(2) * t)
}

/// `a` to `b` by `t`.
#[allow(dead_code)] // in the kit for the recipe's use; the twin test keeps it honest
pub fn mix(a: Fix, b: Fix, t: Fix) -> Fix {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(got: Fix, want: f64, within: f64, what: &str) {
        assert!((got.to_f32() as f64 - want).abs() <= within, "{what}: {} against {want}", got.to_f32());
    }

    #[test]
    fn the_four_sums_are_exact_where_they_should_be() {
        let (a, b) = (Fix::from_f32(3.25), Fix::from_f32(-1.5));
        assert_eq!((a + b).to_f32(), 1.75);
        assert_eq!((a - b).to_f32(), 4.75);
        assert_eq!((a * b).to_f32(), -4.875);
        assert_eq!((a / b).0, -141_994);
        // × rounds down; ÷ towards zero.
        assert_eq!((Fix(-1) * Fix(1)).0, -1);
        assert_eq!((Fix(-1) / Fix::from_int(3)).0, 0);
        // ÷ holds at the end.
        assert_eq!((Fix::ONE / Fix::ZERO).0, i32::MAX);
        assert_eq!((-Fix::ONE / Fix::ZERO).0, -i32::MAX);
    }

    #[test]
    fn roots_logs_powers_and_angles_are_close_to_the_true_ones() {
        // (Against the true answer for the number as it's held: 0.001 is held as 66/65536.)
        let held = |x: f64| Fix::from_f32(x as f32).to_f32() as f64;
        for x in [0.001f64, 0.3, 1.0, 2.0, 7.5, 1000.0, 30_000.0] {
            close(sqrt(Fix::from_f32(x as f32)), held(x).sqrt(), 0.0001, "sqrt");
            close(log2(Fix::from_f32(x as f32)), held(x).log2(), 0.0003, "log2");
        }
        for x in [-10.0f64, -1.0, -0.3, 0.0, 0.5, 3.0, 14.5] {
            close(exp2(Fix::from_f32(x as f32)), 2f64.powf(x), 2f64.powf(x) * 0.0002 + 0.00002, "exp2");
        }
        for (x, e) in [(0.5f64, 2.0f64), (0.25, 0.5), (0.9, 8.0), (0.1, 0.125), (1.0, 3.0)] {
            close(pow(Fix::from_f32(x as f32), Fix::from_f32(e as f32)), held(x).powf(held(e)), 0.001, "pow");
        }
        for turns in [0.0f64, 0.1, 0.25, 0.4, 0.5, 0.77, 0.999] {
            let angle = (turns * TURN as f64) as i32;
            let radians = turns * std::f64::consts::TAU;
            close(sin(angle), radians.sin(), 0.0001, "sin");
            close(cos(angle), radians.cos(), 0.0001, "cos");
        }
        for (y, x) in [(1.0f64, 1.0f64), (0.0, -1.0), (-2.0, 0.5), (3.0, -4.0), (0.0, 1.0), (-1.0, 0.0)] {
            let got = atan2(Fix::from_f32(y as f32), Fix::from_f32(x as f32)) as f64 / TURN as f64;
            assert!((got - y.atan2(x) / std::f64::consts::TAU).abs() < 0.0001, "atan2({y}, {x}): {got}");
        }
        close(hypot(Fix::from_f32(3.0), Fix::from_f32(4.0)), 5.0, 0.0001, "hypot");
        // (A fraction this big shows only to about 0.002.)
        close(hypot(Fix::from_f32(20_000.0), Fix::from_f32(-20_000.0)), 28_284.271, 0.004, "hypot far");
        close(smooth(Fix::from_f32(0.25)), 0.15625, 0.0001, "smooth");
    }

    #[test]
    fn positions_reach_across_the_world_and_turn_into_the_noise_s_units() {
        let far = Pos::from_f32(-36_000.0);
        assert_eq!(far.to_f32(), -36_000.0);
        assert_eq!(Pos::middle_of(-3).to_f32(), -2.5);
        // 36,000 blocks over a 300-block feature: 120 of the noise's units.
        close(far.per(Pos::from_f32(300.0)), -120.0, 0.0001, "per");
        // Over a tiny size it wraps round rather than holding.
        assert_eq!(Pos::from_f32(36_000.0).per(Pos::from_f32(1.0)).0, (36_000i64 * 65_536) as i32);
        close(Pos::from_f32(30.0).ratio(Pos::from_f32(120.0)), 0.25, 0.0001, "ratio");
        assert_eq!(Pos::from_f32(-1000.0).scaled(Fix::from_f32(0.5)).to_f32(), -500.0);
        assert_eq!(Pos::from_f32(30_000.0).hypot(Pos::from_f32(40_000.0)).to_f32(), 50_000.0);
    }
}

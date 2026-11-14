//! Blue noise, for where a ray's steps begin (plan 2.2, the whorls talk; David, 2026-11-14: "have you heard of blue
//! noise being used for this kind of thing?").
//!
//! A square of numbers, each texel's rank among them all, ordered so the texels under any rank are spread evenly: no
//! clumps and no gaps, and the square tiles without a seam. A pixel takes its texel's number for where along a step
//! its ray begins, so neighbouring rays never begin at nearly the same place by chance, as plain scatter lets them.
//! Made once at load by void and cluster (Ulichney, 1993): a few points scattered and evened out, then every other
//! texel ranked by where the widest gap is.

/// The square's side in texels (it repeats every so many pixels of a layer).
pub const SIDE: usize = 64;
const TEXELS: usize = SIDE * SIDE;
/// How far a point's presence is felt (texels): void and cluster's own 1.5 (at 1.9 a few of the first points touched).
const REACH: f32 = 1.5;

/// How much one point adds at every offset from it, round the square's edges.
fn kernel() -> Vec<f32> {
    let mut kernel = vec![0.0; TEXELS];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let (dx, dy) = (x.min(SIDE - x) as f32, y.min(SIDE - y) as f32);
            kernel[y * SIDE + x] = (-(dx * dx + dy * dy) / (2.0 * REACH * REACH)).exp();
        }
    }
    kernel
}

/// A point put in at `at` (`by` 1) or taken out (−1): what every texel feels of it.
fn feel(felt: &mut [f32], kernel: &[f32], at: usize, by: f32) {
    let (ax, ay) = (at % SIDE, at / SIDE);
    for y in 0..SIDE {
        let row = &kernel[((y + SIDE - ay) % SIDE) * SIDE..][..SIDE];
        let out = &mut felt[y * SIDE..][..SIDE];
        for x in 0..SIDE {
            out[x] += by * row[(x + SIDE - ax) % SIDE];
        }
    }
}

/// The texel that feels the most (the tightest cluster) among those `set` is `want` for; `least`: the least (the
/// widest gap). The first of equals.
fn pick(felt: &[f32], set: &[bool], want: bool, least: bool) -> usize {
    let mut best = usize::MAX;
    for at in 0..TEXELS {
        if set[at] != want {
            continue;
        }
        if best == usize::MAX || (least && felt[at] < felt[best]) || (!least && felt[at] > felt[best]) {
            best = at;
        }
    }
    best
}

/// Each texel's rank, 0 to `SIDE`² − 1, every rank once.
pub fn ranks() -> Vec<u16> {
    let kernel = kernel();
    // A tenth of the texels scattered (a fixed scatter: the same square every launch).
    let mut set = vec![false; TEXELS];
    let mut felt = vec![0.0f32; TEXELS];
    let mut seed = 0x9E3779B9u32;
    let mut points = 0;
    while points < TEXELS / 10 {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let at = seed as usize % TEXELS;
        if !set[at] {
            set[at] = true;
            feel(&mut felt, &kernel, at, 1.0);
            points += 1;
        }
    }
    // Evened out: the tightest point moved to the widest gap, till the gap it leaves is the widest.
    loop {
        let tight = pick(&felt, &set, true, false);
        set[tight] = false;
        feel(&mut felt, &kernel, tight, -1.0);
        let gap = pick(&felt, &set, false, true);
        set[gap] = true;
        feel(&mut felt, &kernel, gap, 1.0);
        if gap == tight {
            break;
        }
    }
    let mut ranks = vec![0u16; TEXELS];
    // The first points' own ranks: the tightest taken out first has the highest of them.
    let (mut left, mut left_felt) = (set.clone(), felt.clone());
    for rank in (0..points).rev() {
        let tight = pick(&left_felt, &left, true, false);
        left[tight] = false;
        feel(&mut left_felt, &kernel, tight, -1.0);
        ranks[tight] = rank as u16;
    }
    // Up to half: each next point in the widest gap.
    for rank in points..TEXELS / 2 {
        let gap = pick(&felt, &set, false, true);
        set[gap] = true;
        feel(&mut felt, &kernel, gap, 1.0);
        ranks[gap] = rank as u16;
    }
    // Past half the empty texels are the few: each next point where they cluster tightest.
    let mut empty_felt = vec![0.0f32; TEXELS];
    for at in 0..TEXELS {
        if !set[at] {
            feel(&mut empty_felt, &kernel, at, 1.0);
        }
    }
    for rank in TEXELS / 2..TEXELS {
        let tight = pick(&empty_felt, &set, false, false);
        set[tight] = true;
        feel(&mut empty_felt, &kernel, tight, -1.0);
        ranks[tight] = rank as u16;
    }
    ranks
}

/// The square as bytes for the card (0 to 255, each value as often as every other).
pub fn bytes() -> Vec<u8> {
    ranks().into_iter().map(|rank| (rank as usize * 256 / TEXELS) as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How near the nearest other point is, for each of `points` (texels, round the square's edges): the least and
    /// the mean.
    fn nearest(points: &[usize]) -> (f32, f32) {
        let (mut least, mut sum) = (f32::MAX, 0.0);
        for &a in points {
            let mut near = f32::MAX;
            for &b in points {
                if a == b {
                    continue;
                }
                let (dx, dy) = ((a % SIDE).abs_diff(b % SIDE), (a / SIDE).abs_diff(b / SIDE));
                let (dx, dy) = (dx.min(SIDE - dx) as f32, dy.min(SIDE - dy) as f32);
                near = near.min((dx * dx + dy * dy).sqrt());
            }
            least = least.min(near);
            sum += near;
        }
        (least, sum / points.len() as f32)
    }

    #[test]
    fn every_rank_once_and_no_clumps() {
        let started = std::time::Instant::now();
        let ranks = ranks();
        eprintln!("Blue noise: {SIDE} x {SIDE} made in {:.3} s", started.elapsed().as_secs_f32());
        let mut seen = vec![false; TEXELS];
        for &rank in &ranks {
            assert!(!seen[rank as usize], "rank {rank} twice");
            seen[rank as usize] = true;
        }
        // Every byte as often as every other.
        let mut counts = [0usize; 256];
        for byte in bytes() {
            counts[byte as usize] += 1;
        }
        assert!(counts.iter().all(|&n| n == TEXELS / 256), "the bytes aren't even");
        // The texels under a sixteenth, an eighth, a quarter and a half: none close together. Evenly spread, 256
        // points on 4,096 texels stand about 4.3 apart; scattered by chance (the control: the same count picked by a
        // hash), some always touch.
        for (share, least_wanted, mean_wanted) in [(16, 2.5, 3.0), (8, 1.9, 2.2), (4, 1.0, 1.35), (2, 1.0, 1.0)] {
            let points: Vec<usize> = (0..TEXELS).filter(|&at| (ranks[at] as usize) < TEXELS / share).collect();
            let (least, mean) = nearest(&points);
            eprintln!("Blue noise: the lowest 1/{share}: nearest neighbours at least {least:.2} apart, {mean:.2} on average");
            assert!(least >= least_wanted && mean >= mean_wanted, "the lowest 1/{share} clump: least {least}, mean {mean}");
        }
        let mut order: Vec<usize> = (0..TEXELS).collect();
        order.sort_by_key(|&at| (at as u32).wrapping_mul(0x9E3779B1).rotate_left(13).wrapping_mul(0x85EBCA77));
        let (least, mean) = nearest(&order[..TEXELS / 16]);
        eprintln!("Blue noise: the control (by chance): at least {least:.2} apart, {mean:.2} on average");
        assert!(least < 2.0 && mean < 3.0, "the control doesn't clump: the check proves nothing");
    }
}

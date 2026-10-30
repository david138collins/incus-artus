//! The river network — Climate step 6: which of the water's paths are rivers [David and Builder,
//! 2026-10-30: "The river rework: growing rivers from the mouths", docs/worldgen.md].
//!
//! Step 5 (`rivers.rs`) already gives every patch of land the patch it drains into, on the filled
//! ground, always downhill or across a lake, ending at the sea. Read upstream that's a tree for each
//! river mouth, and a path climbing only uphill or level from a mouth can never leave it. So growing
//! rivers from the mouths is **choosing branches of that tree**: every river drains, stays in its own
//! catchment, and has a bed that only falls downstream, without having to check. Five passes, on the
//! map's 32-block grid:
//! 1. **Springs first:** where the ground is high (plain height, or high for its surroundings) and wet
//!    enough, spaced apart with some jitter (even, but not too even). Big lakes' outlets count too.
//! 2. **Mouths:** each spring's path down the tree ends at a mouth. Mouths too close together: the one
//!    with fewer springs loses them.
//! 3. **Growing up from each mouth** toward its springs, splitting where they're reached through
//!    different branches; how readily it splits is a slider.
//! 4. **Springs no river took** run down as short streams: they join a river, end in a lake or the sea,
//!    or fizzle out first (sooner the drier they are).
//! 5. **Back down to conclude:** the rank comes from branches joining (stream order: 1 where a
//!    branch begins, one up where two of the same order meet) [David: no need to model the water's
//!    volume]; the bed is laid in flat runs and steps (the waterfalls and rapids).

use super::rivers::Flow;
use crate::world::SEA_LEVEL;
use bevy_egui::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};

/// The river network's settings (the Climate tab's Rivers section).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    /// Springs by plain height (between `spring_from` and `spring_to`, Z), or by height relative to
    /// the surroundings (at least `rise` blocks above the average within `around` blocks).
    pub relative: bool,
    pub spring_from: f32,
    pub spring_to: f32,
    pub rise: f32,
    pub around: f32,
    /// How much of its own water (rain, snowmelt, slope-foot springs; units) a spring needs.
    pub spring_water: f32,
    /// The least distance between springs (blocks), and how much it varies (0 even to 1 loose).
    pub spacing: f32,
    pub jitter: f32,
    /// The least distance between river mouths (blocks).
    pub mouth_spacing: f32,
    /// How readily a river splits (0: one trunk per mouth; 1: every branch that reaches a spring).
    pub branching: f32,
    /// A lake this many patches big or more feeds a river from its outlet; smaller is a pond.
    pub lake_size: f32,
    /// How far a stream no river took runs before fizzling out (blocks, at the spring water needed;
    /// wetter springs run further).
    pub reach: f32,
    /// How many stream orders make one rank (creek, stream, river, great river).
    pub orders_per_rank: u32,
    /// How far up from its mouth a river keeps its full width (a share of its length; past that it
    /// narrows toward its spring) [David: "how far up the water from the ocean it is river"].
    pub full_size: f32,
    /// The bed: how much drop (blocks) gathers into one step, and how tall a step has to be to be a
    /// waterfall rather than rapids.
    pub step: f32,
    pub fall_from: f32,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            relative: false,
            spring_from: 260.0,
            spring_to: 700.0,
            rise: 40.0,
            around: 2_000.0,
            spring_water: 0.5,
            spacing: 2_500.0,
            jitter: 0.5,
            mouth_spacing: 3_000.0,
            branching: 0.5,
            lake_size: 20.0,
            reach: 3_000.0,
            orders_per_rank: 1,
            full_size: 0.5,
            step: 4.0,
            fall_from: 3.0,
        }
    }
}

/// What became of a spring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// The head of a river.
    River,
    /// Its stream joined a river.
    Joined,
    /// Its stream ran into a lake, or into the sea.
    Lake,
    Sea,
    /// Its stream dried away first.
    Fizzled,
}

/// The ranks by width, as David set them (blocks): creek, stream, river, great river.
pub const RANKS: [(&str, f32, f32); 4] = [("creek", 2.0, 4.0), ("stream", 4.0, 8.0), ("river", 8.0, 32.0), ("great river", 32.0, 128.0)];

/// The river network on the map's grid.
#[derive(Clone, Debug)]
pub struct Network {
    pub width: usize,
    /// Each patch's stream order (0: no river there).
    pub order: Vec<u8>,
    /// Each patch's rank: 0 none, then 1 creek, 2 stream, 3 river, 4 great river.
    pub rank: Vec<u8>,
    /// The bed's height on river patches (NaN elsewhere), and the step down just below each (blocks;
    /// 0 for none).
    pub bed: Vec<f32>,
    pub drop: Vec<f32>,
    /// Every spring and what became of it; every river mouth.
    pub springs: Vec<(u32, Fate)>,
    pub mouths: Vec<u32>,
}

/// A fixed number from 0 to 1 for a patch (the same every time for the same seed).
fn hashed(seed: u64, i: usize, salt: u64) -> f32 {
    let mut x = seed ^ (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 31;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 29;
    (x >> 40) as f32 / (1u64 << 24) as f32
}

/// The average of `values` within `radius` patches of each (a square), from a summed table.
fn local_average(values: &[f32], width: usize, radius: usize) -> Vec<f32> {
    let w = width;
    let mut sum = vec![0.0f64; (w + 1) * (w + 1)];
    for r in 0..w {
        for c in 0..w {
            sum[(r + 1) * (w + 1) + c + 1] = values[r * w + c] as f64 + sum[r * (w + 1) + c + 1] + sum[(r + 1) * (w + 1) + c] - sum[r * (w + 1) + c];
        }
    }
    (0..w * w)
        .map(|i| {
            let (c, r) = (i % w, i / w);
            let (c0, r0, c1, r1) = (c.saturating_sub(radius), r.saturating_sub(radius), (c + radius + 1).min(w), (r + radius + 1).min(w));
            let total = sum[r1 * (w + 1) + c1] - sum[r0 * (w + 1) + c1] - sum[r1 * (w + 1) + c0] + sum[r0 * (w + 1) + c0];
            (total / ((c1 - c0) * (r1 - r0)) as f64) as f32
        })
        .collect()
}

/// Works out the network from step 5's flow (`blocks`: blocks per patch), or `None` if stopped.
pub fn make(flow: &Flow, blocks: f32, seed: u64, s: &NetworkSettings, stop: &AtomicBool) -> Option<Network> {
    let w = flow.width;
    let n = w * w;
    let sea = SEA_LEVEL as f32;
    let down = |i: usize| flow.down[i] as usize;
    let land = |i: usize| flow.surface[i].is_finite() && flow.surface[i] >= sea;
    let lake = |i: usize| flow.lake[i] > 0.0;
    let distance = |a: usize, b: usize| {
        let (dc, dr) = ((a % w) as f32 - (b % w) as f32, (a / w) as f32 - (b / w) as f32);
        (dc * dc + dr * dr).sqrt() * blocks
    };
    // Each patch's own water (what it adds, not what passes through).
    let mut own = flow.flow.clone();
    for i in 0..n {
        if down(i) != i {
            own[down(i)] -= flow.flow[i];
        }
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // Lakes: each lake patch's outlet (the first patch past the lake, downstream), and each outlet's
    // lake size.
    let mut outlet = vec![u32::MAX; n];
    let mut lake_size = vec![0u32; n];
    for i in (0..n).filter(|&i| lake(i)) {
        let mut path = Vec::new();
        let mut j = i;
        while lake(j) && outlet[j] == u32::MAX && down(j) != j {
            path.push(j);
            j = down(j);
        }
        let out = if lake(j) && outlet[j] != u32::MAX { outlet[j] as usize } else { j };
        for k in path {
            outlet[k] = out as u32;
        }
        lake_size[out] += 1;
    }

    // 1. Springs: candidates, then spaced.
    let height = |i: usize| flow.surface[i];
    let surroundings = if s.relative {
        let filled: Vec<f32> = (0..n).map(|i| if land(i) { height(i) } else { sea }).collect();
        local_average(&filled, w, (s.around / blocks).round().max(1.0) as usize)
    } else {
        Vec::new()
    };
    let high = |i: usize| {
        if s.relative {
            height(i) - surroundings[i] >= s.rise
        } else {
            (s.spring_from..=s.spring_to).contains(&height(i))
        }
    };
    let mut candidates: Vec<(f32, usize)> = (0..n).filter(|&i| land(i) && !lake(i) && high(i) && own[i] >= s.spring_water).map(|i| (hashed(seed, i, 1), i)).collect();
    // Big lakes' outlets first (they always spill a river's worth), then the rest in a shuffled order.
    let mut first: Vec<usize> = (0..n).filter(|&i| lake_size[i] as f32 >= s.lake_size.max(1.0) && land(i) && !lake(i)).collect();
    first.sort_unstable();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    let order_tried: Vec<usize> = first.iter().copied().chain(candidates.iter().map(|&(_, i)| i)).collect();
    let cell = (s.spacing / blocks / 2.0).max(1.0);
    let cells = (w as f32 / cell).ceil() as usize + 1;
    let mut grid: Vec<Vec<usize>> = vec![Vec::new(); cells * cells];
    let mut springs: Vec<usize> = Vec::new();
    for i in order_tried {
        let room = s.spacing * (1.0 - s.jitter.clamp(0.0, 1.0) * hashed(seed, i, 2));
        let (gc, gr) = (((i % w) as f32 / cell) as i64, ((i / w) as f32 / cell) as i64);
        let reach = (room / blocks / cell).ceil() as i64 + 1;
        let crowded = (-reach..=reach).any(|dr| {
            (-reach..=reach).any(|dc| {
                let (c, r) = (gc + dc, gr + dr);
                c >= 0 && r >= 0 && (c as usize) < cells && (r as usize) < cells && grid[r as usize * cells + c as usize].iter().any(|&j| distance(i, j) < room)
            })
        });
        if !crowded {
            grid[gr as usize * cells + gc as usize].push(i);
            springs.push(i);
        }
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // 2. Mouths: where each spring's path ends; spaced, the busier first.
    let mouth_of = |mut j: usize| {
        while down(j) != j {
            j = down(j);
        }
        j
    };
    let ends: Vec<usize> = springs.iter().map(|&i| mouth_of(i)).collect();
    let mut busy: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &m in &ends {
        *busy.entry(m).or_default() += 1;
    }
    let mut by_busy: Vec<(usize, usize)> = busy.into_iter().collect();
    by_busy.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut mouths: Vec<usize> = Vec::new();
    for (m, _) in by_busy {
        if mouths.iter().all(|&kept| distance(m, kept) >= s.mouth_spacing) {
            mouths.push(m);
        }
    }

    // 3. Grow up from each mouth: the paths of its springs, splitting where they part.
    let kept: Vec<bool> = ends.iter().map(|m| mouths.contains(m)).collect();
    let mut springs_above = vec![0u32; n];
    for (k, &i) in springs.iter().enumerate().filter(|&(k, _)| kept[k]) {
        let _ = k;
        let mut j = i;
        loop {
            springs_above[j] += 1;
            if down(j) == j {
                break;
            }
            j = down(j);
        }
    }
    // Each patch's upstream neighbours on those paths.
    let mut ups: Vec<Vec<u32>> = vec![Vec::new(); n];
    for i in (0..n).filter(|&i| springs_above[i] > 0 && down(i) != i) {
        ups[down(i)].push(i as u32);
    }
    let mut river = vec![false; n];
    let mut queue: VecDeque<usize> = mouths.iter().copied().filter(|&m| springs_above[m] > 0).collect();
    for &m in &queue {
        river[m] = true;
    }
    while let Some(i) = queue.pop_front() {
        let most = ups[i].iter().map(|&u| springs_above[u as usize]).max().unwrap_or(0);
        for &u in &ups[i] {
            let u = u as usize;
            // The main branch always; the others if they carry enough of the springs.
            if springs_above[u] == most || springs_above[u] as f32 >= (1.0 - s.branching.clamp(0.0, 1.0)) * most as f32 {
                river[u] = true;
                queue.push_back(u);
            }
        }
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // 4. The springs no river took: short streams down the tree, until they join a river, reach a lake
    // or the sea, or dry away.
    let mut fates = Vec::with_capacity(springs.len());
    let mut stream = vec![false; n];
    for &i in &springs {
        if river[i] {
            fates.push((i as u32, Fate::River));
            continue;
        }
        let life = s.reach * (own[i] / s.spring_water.max(1e-3)).max(0.0).sqrt();
        let (mut j, mut run, mut path) = (i, 0.0, vec![i]);
        let fate = loop {
            let next = down(j);
            if river[next] {
                break Fate::Joined;
            }
            if lake(next) {
                break Fate::Lake;
            }
            if next == j || !land(next) {
                break Fate::Sea;
            }
            run += distance(j, next);
            if run > life {
                break Fate::Fizzled;
            }
            j = next;
            path.push(j);
        };
        for k in path {
            stream[k] = true;
        }
        fates.push((i as u32, fate));
    }

    // 5. Back down: stream order (upstream first), ranks, and the bed in flats and steps.
    let on = |i: usize| (river[i] || stream[i]) && land(i) && !lake(i);
    let mut waiting = vec![0u32; n];
    for i in (0..n).filter(|&i| on(i) && down(i) != i && on(down(i))) {
        waiting[down(i)] += 1;
    }
    let mut order = vec![0u8; n];
    let (mut best, mut ties) = (vec![0u8; n], vec![0u8; n]);
    let mut ready: Vec<usize> = (0..n).filter(|&i| on(i) && waiting[i] == 0).collect();
    while let Some(i) = ready.pop() {
        order[i] = if best[i] == 0 { 1 } else if ties[i] >= 2 { best[i].saturating_add(1) } else { best[i] };
        let d = down(i);
        if d != i && on(d) {
            if order[i] > best[d] {
                best[d] = order[i];
                ties[d] = 1;
            } else if order[i] == best[d] {
                ties[d] += 1;
            }
            waiting[d] -= 1;
            if waiting[d] == 0 {
                ready.push(d);
            }
        }
    }
    let per = s.orders_per_rank.max(1) as u8;
    let rank: Vec<u8> = order.iter().map(|&o| if o == 0 { 0 } else { (1 + (o - 1) / per).min(4) }).collect();
    // The bed, from each river's lowest patch upward: flat until the ground has risen a step above
    // it, then up to the ground (a step, between that patch and the one below it).
    let mut bed = vec![f32::NAN; n];
    let mut drop = vec![0.0f32; n];
    let mut upstream: Vec<Vec<u32>> = vec![Vec::new(); n];
    for i in (0..n).filter(|&i| on(i) && down(i) != i && on(down(i))) {
        upstream[down(i)].push(i as u32);
    }
    let mut climb: Vec<usize> = (0..n).filter(|&i| on(i) && (down(i) == i || !on(down(i)))).collect();
    for &i in &climb {
        bed[i] = height(i);
    }
    while let Some(i) = climb.pop() {
        for &u in &upstream[i] {
            let u = u as usize;
            if height(u) - bed[i] >= s.step.max(0.0) {
                bed[u] = height(u);
                drop[u] = bed[u] - bed[i];
            } else {
                bed[u] = bed[i];
            }
            climb.push(u);
        }
    }
    let mouths = mouths.into_iter().filter(|&m| springs_above[m] > 0).map(|m| m as u32).collect();
    Some(Network { width: w, order, rank, bed, drop, springs: fates, mouths })
}

impl Network {
    /// The network as a picture on the map's grid: rivers bluer and more solid the higher their rank,
    /// waterfalls white and rapids pale; springs by fate (river green, joined yellow, into a lake or
    /// the sea cyan, fizzled red); mouths magenta.
    pub fn picture(&self, fall_from: f32) -> Vec<Color32> {
        let mut pixels: Vec<Color32> = (0..self.width * self.width)
            .map(|i| match self.rank[i] {
                0 => Color32::TRANSPARENT,
                rank => {
                    if self.drop[i] >= fall_from {
                        Color32::WHITE
                    } else if self.drop[i] > 0.0 {
                        Color32::from_rgb(190, 230, 255)
                    } else {
                        let k = (rank - 1) as f32 / 3.0;
                        Color32::from_rgba_unmultiplied((110.0 - 90.0 * k) as u8, (210.0 - 110.0 * k) as u8, 255, (170.0 + 85.0 * k) as u8)
                    }
                }
            })
            .collect();
        for &(i, fate) in &self.springs {
            pixels[i as usize] = match fate {
                Fate::River => Color32::from_rgb(80, 255, 80),
                Fate::Joined => Color32::from_rgb(255, 230, 60),
                Fate::Lake | Fate::Sea => Color32::from_rgb(60, 255, 255),
                Fate::Fizzled => Color32::from_rgb(255, 70, 70),
            };
        }
        for &m in &self.mouths {
            pixels[m as usize] = Color32::from_rgb(255, 60, 220);
        }
        pixels
    }

    /// How many springs met each fate (river, joined, lake or sea, fizzled), for the panel.
    pub fn fates(&self) -> [usize; 4] {
        let mut counts = [0; 4];
        for &(_, fate) in &self.springs {
            counts[match fate {
                Fate::River => 0,
                Fate::Joined => 1,
                Fate::Lake | Fate::Sea => 2,
                Fate::Fizzled => 3,
            }] += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A valley country: ground rising from the sea in the west to hills in the east, with valleys
    /// running west, and its flow (every patch one unit of water).
    fn country(width: usize) -> Flow {
        let z: Vec<Option<f32>> = (0..width * width)
            .map(|i| {
                let (c, r) = ((i % width) as f32, (i / width) as f32);
                let valleys = (r * 0.35).sin() * 6.0 * (c / width as f32);
                Some(100.0 + c * 6.0 + valleys)
            })
            .collect();
        let water = vec![1.0; width * width];
        super::super::rivers::flow(width, &z, &water, 2.0, 7, &AtomicBool::new(false)).unwrap()
    }

    fn settings() -> NetworkSettings {
        NetworkSettings { spring_from: 300.0, spring_to: 1_000.0, spacing: 200.0, mouth_spacing: 100.0, spring_water: 0.5, ..NetworkSettings::default() }
    }

    #[test]
    fn springs_keep_their_distance_and_rivers_run_only_down_to_the_sea() {
        let flow = country(64);
        let s = settings();
        let network = make(&flow, 32.0, 3, &s, &AtomicBool::new(false)).unwrap();
        assert!(network.springs.len() > 3, "{} springs", network.springs.len());
        // No two springs closer than the spacing allows (with its jitter, at least half of it).
        for (a, &(i, _)) in network.springs.iter().enumerate() {
            for &(j, _) in &network.springs[a + 1..] {
                let (dc, dr) = ((i % 64) as f32 - (j % 64) as f32, (i / 64) as f32 - (j / 64) as f32);
                assert!((dc * dc + dr * dr).sqrt() * 32.0 >= s.spacing * (1.0 - s.jitter) - 1e-3);
            }
        }
        // Every river patch's bed is never below the next one down, and the step there is what
        // the bed drops by.
        for i in (0..64 * 64).filter(|&i| network.rank[i] > 0) {
            let d = flow.down[i] as usize;
            if d != i && network.rank[d] > 0 {
                assert!(network.bed[i] >= network.bed[d] - 1e-3, "the bed climbs going downstream");
                assert!((network.drop[i] - (network.bed[i] - network.bed[d])).abs() < 1e-3 || network.drop[i] == 0.0);
            }
        }
        assert!(!network.mouths.is_empty());
    }

    #[test]
    fn two_branches_of_the_same_order_meeting_make_the_next_order() {
        let flow = country(64);
        let network = make(&flow, 32.0, 3, &NetworkSettings { branching: 1.0, ..settings() }, &AtomicBool::new(false)).unwrap();
        for i in (0..64 * 64).filter(|&i| network.order[i] > 0) {
            let ups: Vec<u8> = (0..64 * 64).filter(|&u| flow.down[u] as usize == i && u != i && network.order[u] > 0).map(|u| network.order[u]).collect();
            let most = ups.iter().copied().max().unwrap_or(0);
            let ties = ups.iter().filter(|&&o| o == most).count();
            let expected = if ups.is_empty() { 1 } else if ties >= 2 { most + 1 } else { most };
            assert_eq!(network.order[i], expected, "at {i}: upstream {ups:?}");
        }
    }

    #[test]
    fn less_branching_keeps_fewer_river_patches() {
        let flow = country(64);
        let count = |branching: f32| {
            let network = make(&flow, 32.0, 3, &NetworkSettings { branching, ..settings() }, &AtomicBool::new(false)).unwrap();
            network.springs.iter().filter(|(_, fate)| *fate == Fate::River).count()
        };
        assert!(count(0.0) <= count(1.0));
        assert!(count(0.0) >= 1);
    }

    #[test]
    fn relative_height_finds_springs_on_hills_lower_than_the_plain_band() {
        let flow = country(64);
        let plain = make(&flow, 32.0, 3, &NetworkSettings { spring_from: 2_000.0, spring_to: 3_000.0, ..settings() }, &AtomicBool::new(false)).unwrap();
        assert!(plain.springs.is_empty(), "nothing that high");
        let relative = make(&flow, 32.0, 3, &NetworkSettings { relative: true, rise: 10.0, around: 320.0, ..settings() }, &AtomicBool::new(false)).unwrap();
        assert!(!relative.springs.is_empty());
    }
}

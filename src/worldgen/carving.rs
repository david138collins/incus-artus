//! Carving's curves — the river stage's last part: each river of the network (`network.rs`) as a
//! smooth line with its width, bank depth and water level all along it, for the Interpreter to carve
//! (`world/carving.rs`) [David and Builder, 2026-10-30: "Carving: the curve and the carve in one go",
//! docs/worldgen.md]. The curve and the carve come from one making:
//! 1. **Routed on the real ground** [David, 2026-10-30: "curving between hills instead of slicing
//!    through them", like a marble; docs/rivers.md section 9]: the network plans each river on the
//!    broad ground, patch to patch (32 blocks a step), with no hills in it. Each reach then finds its
//!    way on the real, detailed ground in a corridor round that plan, by fill and spill from its end
//!    (as step 5, locally): round the hills, through the lowest gaps, along the valley floors.
//! 2. **Smoothed:** corner cutting (Chaikin's method: each pass slices every corner off) over points a
//!    river's width or so apart, so bigger rivers bend wider. (The wander, lean and meanders that
//!    moved it sideways were cut on 2026-10-31: they only moved it onto higher ground.)
//! 3. **The carve** (in the world): each block measures its distance to the line; the odd block of an
//!    even width goes to the outside of the bend.
//!
//! 4. **Ponds behind rises** [David, 2026-10-30: "ride over the terrain like a marble, or cut far
//!    deeper"]: a rise the route couldn't get round is cut through up to a few blocks; a higher one
//!    holds the water back in a pond up to its lowest gap (unless the pond would spread too far).
//!
//! Safety: the water level is laid from the real ground under the line, only falling downstream; and
//! the ends (junctions, lake outlets, mouths) are pinned, so rivers always meet.

use super::network::Network;
use super::noise::{Fade, perlin};
use super::rivers::Flow;
use crate::world::{Banks, Carving, CompiledMap, Lake, Lakes, Reach, RiverPoint, SEA_LEVEL, Shores, under_sea};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, Ordering};

/// Carving's settings (the Climate tab's Carving section).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CarvingSettings {
    /// Corner-cutting passes for a creek (one more for each rank up).
    pub smoothing: u32,
    /// The anti-kitty-cornerer [David, 2026-10-31]: the water's blocks always join by a face, never
    /// only at a corner (`world::Carving::anti_kitty_corner`).
    pub anti_kitty_corner: bool,
    /// The water's level after a hollow [2026-10-31, David: "filling and cutting should be on a
    /// threshold... least change is kind of how i try to get everything"]: cut through the rises
    /// after it (the level only falling from the lowest ground so far), or the least change, rises cut
    /// and hollows filled by turns (`least_change`). With least change, the water stands at most
    /// `fill_up_to` above the lowest ground across a hollow, and before cutting a rise deeper than
    /// `cut_most` it fills the hollow before it more (the fill's limit wins).
    pub level: Level,
    pub fill_up_to: f32,
    /// With least change, how far the water may stand above the lower bank beside it (blocks; 0: only
    /// where the land rises on both sides across it), its raised banks easing it down.
    pub fill_over_banks: f32,
    pub cut_most: f32,
    /// Where a river joins another, its water eases down toward the joined river's over its last
    /// stretch, this many of the joined river's widths, as steeply as rapids go; what's left is a
    /// fall at the shore (0: none) [David, 2026-10-31: "a linear joint fade similar to how we do the
    /// lateral bank fades"].
    pub join_fade: f32,
    /// Rivers running alongside each other merge, the higher falling into the lower (`merge_alongside`)
    /// [David, 2026-10-31: "it should cause a waterfall to merge, shouldnt it?"].
    pub merge_alongside: bool,
    /// The route on the real ground [2026-10-30]: how far either side of its planned path a river may
    /// find its way (blocks), and the grain it's found on (blocks a cell).
    pub corridor: f32,
    pub grain: f32,
    /// Where the way is found [stage 4 of `docs/water-rework.md`, 2026-10-31, David: "it's better for
    /// the rivers to meander around the terrain down low near sea level", gorges "better up high"]:
    /// everywhere (in the corridor round every river's plan), only where needed (round each stretch
    /// where the water would cut deeper than `deep_cut` below the ground: near the sea, and high up,
    /// blended by the water's height between the sea and the highest river), or off (the plan).
    pub route: Route,
    pub deep_cut: [f32; 2],
    /// Lakes, filled on the real ground [2026-10-30]: how far round the planned lakes (step 5's,
    /// on the broad ground) to look for their real basins (blocks), and how deep a pool has to be
    /// at its deepest to be kept (blocks).
    pub lake_reach: f32,
    pub pond_depth: f32,
    /// Ponds behind rises [David, 2026-10-30: "either need to ride over the terrain like a marble, or
    /// cut far deeper in the land"; breach or fill]: a rise across a river's way is cut through up to
    /// this deep (blocks); a deeper one, if `rise_pools`, holds the water back in a pond up to its
    /// lowest gap, as long as the pond stays within `pool_reach` blocks of the river (if not, it's
    /// cut through).
    pub rise_pools: bool,
    pub cut_up_to: f32,
    pub pool_reach: f32,
    /// Banks [David's profiles proposal, step 3, 2026-10-30; David: "rivers that are 32 blocks wide
    /// should be sloping the terrain for at least 32 blocks in either direction"]: how far each
    /// rank's banks fade into the land (in its widths; creek, stream, river, great river), at least
    /// this many blocks; and how they fade (kept with the world).
    pub fade_widths: [f32; 4],
    pub fade_least: f32,
    pub banks: Banks,
    /// Sizes [David's profiles drawing, 2026-10-30: "drawing wins", growing "by ratios we set"]: a
    /// spring's width and depth (blocks); then for each step up a rank (spring to creek, creek to
    /// stream, stream to river, river to great river) how many times wider and deeper; and how much a
    /// river's size wanders along it (a share).
    pub spring_width: f32,
    pub spring_depth: f32,
    pub width_ratio: [f32; 4],
    pub depth_ratio: [f32; 4],
    pub size_varies: f32,
    /// The shape across for each rank (creek, stream, river, great river), and how much it wanders
    /// along a river (0 none to 1 a lot).
    pub shapes: [Shape; 4],
    pub shape_varies: f32,
    /// Sizes by area [stage 5 of `docs/water-rework.md`, 2026-10-31; David: "sizes are area", "make
    /// sure it can be disable with a checkbox"; an experiment beside the ranks' sizes, off to start]:
    /// a river's size is the blocks of water across it, gathered on its way down (`water_by_area`): a
    /// spring's to start; where waters meet, this share of all of theirs; and a block more for every
    /// `grows_every` blocks run (0: never). Its slope lays it out (`size_by_area`): `broadest` blocks
    /// wide for every block deep where the broad ground falls `flat_below` blocks in 100 or less,
    /// `deepest` where it falls `steep_from` or more; its shape across mixed the same way from
    /// `flat_shape` to `steep_shape`; and its banks fading over `fade_over` of its widths.
    pub by_area: bool,
    pub spring_area: f32,
    pub join_keeps: f32,
    pub grows_every: f32,
    pub broadest: f32,
    pub deepest: f32,
    pub flat_below: f32,
    pub steep_from: f32,
    pub flat_shape: Shape,
    pub steep_shape: Shape,
    pub fade_over: f32,
    /// Waterfalls: a curtain of water blocks down the face. Rapids: blocks of river for each block
    /// of drop.
    pub curtains: bool,
    pub rapids_length: f32,
    /// Deltas: at a river's mouth, over ground this flat or flatter (blocks down per block), its last
    /// stretch splits into channels fanned this wide (blocks) for a river and a great river.
    pub deltas: bool,
    pub delta_flat: f32,
    pub delta_width: [f32; 2],
    /// Made on at most this many of the CPU's cores [David, 2026-10-31: the CPU at 92 to 99 °C while
    /// the rivers were made]: fewer runs cooler and takes longer; the rivers come out the same.
    pub cores: usize,
}

/// Where a river finds its way on the real ground (`CarvingSettings::route`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// In the corridor round every river's plan.
    #[default]
    Everywhere,
    /// Only round the stretches where the plan would cut too deep.
    WhereNeeded,
    /// Nowhere: the plan, smoothed.
    Off,
}

/// The water's level after a hollow (`CarvingSettings::level`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    /// Cut through the rises after it.
    #[default]
    Cut,
    /// The level changing the land least: rises cut and hollows filled by turns.
    LeastChange,
}

/// The water's levels along a reach that change the land least [stage 3 of `docs/water-rework.md`,
/// 2026-10-31: after a hollow, the water only falling, a river cut every rise after it down to the
/// hollow's level]: only falling, as near the lowest ground across each point (`low`) as can be in
/// all (least squares: pool adjacent violators, one pass), no higher than `cap` (the water arriving at
/// its start); then no higher than any point's `ceiling` upstream (its fill limit; it wins), and
/// no more than `cut` below the ground at any point downstream.
fn least_change(low: &[f32], ceiling: &[f32], cap: f32, cut: f32) -> Vec<f32> {
    // (Off the map, the ground before it.)
    let mut last = cap.min(low.iter().copied().find(|&l| l < f32::MAX).unwrap_or(SEA_LEVEL as f32));
    let low: Vec<f32> = low
        .iter()
        .map(|&l| {
            if l < f32::MAX {
                last = l;
            }
            last
        })
        .collect();
    // Pools of points at one level (their sum and count), merged while one stands above the one
    // before it (the water would rise).
    let mut pools: Vec<(f64, usize)> = Vec::with_capacity(low.len());
    for &l in &low {
        pools.push((l as f64, 1));
        while pools.len() > 1 {
            let (b, a) = (pools[pools.len() - 1], pools[pools.len() - 2]);
            if b.0 / (b.1 as f64) <= a.0 / (a.1 as f64) {
                break;
            }
            pools.pop();
            *pools.last_mut().unwrap() = (a.0 + b.0, a.1 + b.1);
        }
    }
    let fitted = pools.iter().flat_map(|&(sum, count)| std::iter::repeat_n((sum / count as f64) as f32, count));
    // The bounds, only falling too: no higher than the ceiling of any point so far, no lower than the
    // cut below any point to come.
    let mut lowest = vec![f32::MIN; low.len()];
    let mut most = f32::MIN;
    for k in (0..low.len()).rev() {
        most = most.max(low[k] - cut.max(0.0));
        lowest[k] = most;
    }
    let mut highest = cap;
    fitted
        .zip(ceiling.iter().zip(lowest))
        .map(|(level, (&ceiling, lowest))| {
            highest = highest.min(ceiling);
            level.max(lowest).min(highest)
        })
        .collect()
}

/// The cores the making may use: its setting, from one to as many as the machine has.
fn most_cores(s: &CarvingSettings) -> usize {
    s.cores.clamp(1, std::thread::available_parallelism().map_or(4, |n| n.get()))
}

impl Default for CarvingSettings {
    fn default() -> Self {
        Self {
            smoothing: 2,
            anti_kitty_corner: true,
            level: Level::Cut,
            fill_up_to: 3.0,
            fill_over_banks: 3.0,
            join_fade: 3.0,
            merge_alongside: true,
            cut_most: 40.0,
            corridor: 64.0,
            grain: 4.0,
            route: Route::Everywhere,
            deep_cut: [8.0, 8.0],
            lake_reach: 64.0,
            pond_depth: 1.0,
            rise_pools: true,
            cut_up_to: 4.0,
            pool_reach: 512.0,
            fade_widths: [1.0; 4],
            fade_least: 2.0,
            banks: Banks::default(),
            spring_width: 2.0,
            spring_depth: 1.5,
            width_ratio: [3.0, 2.5, 2.5, 2.5],
            depth_ratio: [2.0, 1.5, 1.25, 1.25],
            size_varies: 0.15,
            shapes: [
                Shape { edge: 0.3, flat: 0.2, bowl: 1.0 },
                Shape { edge: 0.5, flat: 0.5, bowl: 0.6 },
                Shape { edge: 0.2, flat: 0.3, bowl: 1.6 },
                Shape { edge: 0.2, flat: 0.3, bowl: 1.6 },
            ],
            shape_varies: 0.3,
            by_area: false,
            spring_area: 1.0,
            join_keeps: 0.9,
            grows_every: 2_000.0,
            broadest: 16.0,
            deepest: 0.5,
            flat_below: 0.2,
            steep_from: 5.0,
            flat_shape: Shape { edge: 0.25, flat: 0.4, bowl: 1.6 },
            steep_shape: Shape { edge: 0.5, flat: 0.4, bowl: 0.8 },
            fade_over: 1.0,
            curtains: true,
            rapids_length: 3.0,
            deltas: true,
            delta_flat: 0.01,
            delta_width: [64.0, 128.0],
            // Half of them.
            cores: (std::thread::available_parallelism().map_or(4, |n| n.get()) / 2).max(1),
        }
    }
}

/// A river's shape across (see `world::carving::depth_across`): the water's depth at the bank (a
/// share of the deepest), the share of the width at the deepest, and the bowl between.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub edge: f32,
    pub flat: f32,
    pub bowl: f32,
}

impl Shape {
    fn mix(self, other: Shape, t: f32) -> Shape {
        let m = |a: f32, b: f32| a + (b - a) * t;
        Shape { edge: m(self.edge, other.edge), flat: m(self.flat, other.flat), bowl: m(self.bowl, other.bowl) }
    }
}

/// A river's width, depth and shape a share of the way down its rank's band (`rank` 1 creek to 4
/// great river; 0 where it becomes that rank, 1 where it joins a bigger one). Each rank's size is a
/// spring's times the ratios up to it, and its band runs halfway (by ratio) to the ranks either side,
/// so the bands meet; a creek's starts at a spring's size. Its shape is its rank's in the middle of
/// the band, halfway to its neighbours' at the ends.
pub fn size_in_band(s: &CarvingSettings, rank: usize, share: f32) -> (f32, f32, Shape) {
    let rank = rank.clamp(1, 4);
    let ratio = |k: usize| (s.width_ratio[k].max(1.0), s.depth_ratio[k].max(1.0));
    let mut size = (s.spring_width.max(1.0), s.spring_depth.max(1.0));
    for k in 0..rank {
        size = (size.0 * ratio(k).0, size.1 * ratio(k).1);
    }
    let low = if rank == 1 { (s.spring_width.max(1.0), s.spring_depth.max(1.0)) } else { (size.0 / ratio(rank - 1).0.sqrt(), size.1 / ratio(rank - 1).1.sqrt()) };
    let (up_w, up_d) = ratio(rank.min(3));
    let high = (size.0 * up_w.sqrt(), size.1 * up_d.sqrt());
    let t = share.clamp(0.0, 1.0);
    let grow = |a: f32, b: f32| a * (b / a).powf(t);
    let own = s.shapes[rank - 1];
    let shape = if t < 0.5 { s.shapes[rank.saturating_sub(2)].mix(own, 0.5 + t) } else { own.mix(s.shapes[rank.min(3)], t - 0.5) };
    (grow(low.0, high.0), grow(low.1, high.1), shape)
}

/// A river's width, depth and shape by area (`CarvingSettings::by_area`): `water` blocks of water
/// across it, laid out by how steep the land falls along it (`steep`, 0 at the flattest to 1 at the
/// steepest) [David, 2026-10-31: "deeper and narrower through rapid drops, and flatter and broader
/// through flat, gentle land"]: wide for every block deep from the broadest to the deepest, evenly by
/// ratio, its shape mixed the same way; never under a block wide or deep (a single column holds its
/// whole depth; a block-deep river is as wide as its water).
pub fn size_by_area(s: &CarvingSettings, water: f32, steep: f32) -> (f32, f32, Shape) {
    let t = steep.clamp(0.0, 1.0);
    let shape = s.flat_shape.mix(s.steep_shape, t);
    let (broadest, deepest) = (s.broadest.max(0.1), s.deepest.max(0.1));
    let wide = broadest * (deepest / broadest).powf(t);
    let full = crate::world::fullness(shape.edge, shape.flat, shape.bowl);
    let depth = (water / (wide * full)).sqrt();
    if depth * full < 1.0 && wide * depth >= 1.0 {
        (water.max(1.0), 1.0, shape)
    } else if wide * depth < 1.0 {
        (1.0, (water / full).max(1.0), shape)
    } else {
        (wide * depth, depth, shape)
    }
}

/// The name for a size by area, as a rank for the corner cutting and deltas: 1 a spring, 2 a stream,
/// 3 a river (`world::STREAM_FROM`, `world::RIVER_FROM`).
fn rank_by_area(water: f32) -> u32 {
    if water < crate::world::STREAM_FROM {
        1
    } else if water < crate::world::RIVER_FROM {
        2
    } else {
        3
    }
}

impl CarvingSettings {
    /// How many of its widths a river's banks fade over: one for all by area, else its rank's.
    fn fade_for(&self, rank: u32) -> f32 {
        if self.by_area { self.fade_over } else { self.fade_widths[(rank as usize).clamp(1, 4) - 1] }
    }
}

/// Sizes by area (`CarvingSettings::by_area`): each river patch's water across (blocks), gathered
/// upstream first; and how steep the broad ground falls along it, over up to 4 patches down (0 at
/// `flat_below` blocks in 100 or less, 1 at `steep_from` or more, evenly by ratio). A lake passes the
/// water of the rivers into it on to the river from its outlet, as a junction would.
fn water_by_area(network: &Network, flow: &Flow, blocks: f32, s: &CarvingSettings) -> (Vec<f32>, Vec<f32>) {
    let w = network.width;
    let n = w * w;
    let down = |i: usize| flow.down[i] as usize;
    let on = |i: usize| network.rank[i] > 0;
    let wet = |i: usize| on(i) || flow.lake[i] > 0.0;
    let apart = |a: usize, b: usize| blocks * (((a / w) as f32 - (b / w) as f32).powi(2) + ((a % w) as f32 - (b % w) as f32).powi(2)).sqrt();
    // Each wet patch once every wet patch draining into it is done.
    let mut waiting = vec![0u32; n];
    for i in (0..n).filter(|&i| wet(i) && down(i) != i && wet(down(i))) {
        waiting[down(i)] += 1;
    }
    let (mut gathered, mut joining) = (vec![0.0f32; n], vec![0u32; n]);
    let mut water = vec![0.0f32; n];
    let mut ready: Vec<usize> = (0..n).filter(|&i| wet(i) && waiting[i] == 0).collect();
    while let Some(i) = ready.pop() {
        water[i] = match joining[i] {
            0 if on(i) => s.spring_area,
            0 | 1 => gathered[i],
            _ => gathered[i] * s.join_keeps,
        };
        let d = down(i);
        if d != i && wet(d) {
            if water[i] > 0.0 {
                let ran = if on(i) && s.grows_every > 0.0 { apart(i, d) / s.grows_every } else { 0.0 };
                gathered[d] += water[i] + ran;
                joining[d] += 1;
            }
            waiting[d] -= 1;
            if waiting[d] == 0 {
                ready.push(d);
            }
        }
    }
    let sea = SEA_LEVEL as f32;
    let height = |i: usize| if flow.surface[i].is_finite() { flow.surface[i].max(sea) } else { sea };
    let flat = s.flat_below.max(0.001);
    let steep = s.steep_from.max(flat * 1.01);
    let steepness = (0..n)
        .map(|i| {
            if !on(i) {
                return 0.0;
            }
            let (mut j, mut run) = (i, 0.0);
            for _ in 0..4 {
                let d = down(j);
                if d == j {
                    break;
                }
                run += apart(j, d);
                j = d;
            }
            let fall = if run > 0.0 { 100.0 * (height(i) - height(j)).max(0.0) / run } else { 0.0 };
            ((fall.max(1e-4).ln() - flat.ln()) / (steep.ln() - flat.ln())).clamp(0.0, 1.0)
        })
        .collect();
    (water, steepness)
}

/// A point on a river's path (before the curve): where, and what the river is like there.
#[derive(Clone, Copy)]
struct PathPoint {
    north: f32,
    east: f32,
    width: f32,
    depth: f32,
    shape: Shape,
    level: f32,
}

/// Makes the curves from the network; `None` if stopped.
pub fn make(network: &Network, flow: &Flow, ground: &CompiledMap, blocks: f32, seed: u64, step: f32, fall_from: f32, s: &CarvingSettings, stop: &AtomicBool) -> Option<Carving> {
    let started = std::time::Instant::now();
    let w = network.width;
    let n = w * w;
    let half = w as f32 / 2.0;
    let place = |i: usize| ((half - (i / w) as f32 - 0.5) * blocks, ((i % w) as f32 + 0.5 - half) * blocks);
    let down = |i: usize| flow.down[i] as usize;
    let on = |i: usize| network.rank[i] > 0;
    let patch = |a: usize, b: usize| {
        let ((an, ae), (bn, be)) = (place(a), place(b));
        ((an - bn).powi(2) + (ae - be).powi(2)).sqrt()
    };
    // How far each river patch is from its mouth, and how far along its rank's run: the longest way
    // up and the way down while it stays that rank.
    let rank = |i: usize| network.rank[i].clamp(1, 4);
    let mut to_mouth = vec![0.0f32; n];
    let mut ups: Vec<Vec<u32>> = vec![Vec::new(); n];
    for i in (0..n).filter(|&i| on(i) && down(i) != i && on(down(i))) {
        ups[down(i)].push(i as u32);
    }
    let ends: Vec<usize> = (0..n).filter(|&i| on(i) && (down(i) == i || !on(down(i)))).collect();
    let mut walk: Vec<usize> = ends.clone();
    while let Some(i) = walk.pop() {
        for &u in &ups[i] {
            to_mouth[u as usize] = to_mouth[i] + patch(u as usize, i);
            walk.push(u as usize);
        }
    }
    let (mut up_in_rank, mut down_in_rank) = (vec![0.0f32; n], vec![0.0f32; n]);
    // Upstream first: by distance from the mouth, furthest first.
    let mut by_far: Vec<usize> = (0..n).filter(|&i| on(i)).collect();
    by_far.sort_by(|&a, &b| to_mouth[b].total_cmp(&to_mouth[a]));
    for &i in &by_far {
        let d = down(i);
        if d != i && on(d) && rank(d) == rank(i) {
            up_in_rank[d] = up_in_rank[d].max(up_in_rank[i] + patch(i, d));
        }
    }
    for &i in by_far.iter().rev() {
        let d = down(i);
        if d != i && on(d) && rank(d) == rank(i) {
            down_in_rank[i] = down_in_rank[d] + patch(i, d);
        }
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    // A patch's size and shape: its rank's band, as far down it as the patch is along its rank's run
    // (growing downstream) [2026-10-30, replacing "keeps full size for"]; or by area, its water
    // across laid out by its slope; size and shape wandering slowly along the way.
    let (water, steep) = if s.by_area { water_by_area(network, flow, blocks, s) } else { (Vec::new(), Vec::new()) };
    let size_at = |i: usize, north: f32, east: f32| {
        let run = up_in_rank[i] + down_in_rank[i];
        let share = if run > 0.0 { up_in_rank[i] / run } else { 0.5 };
        let (width, depth, shape) = if s.by_area { size_by_area(s, water[i].max(s.spring_area), steep[i]) } else { size_in_band(s, rank(i) as usize, share) };
        let wobble = |salt: u64| perlin(seed, salt, Fade::Smooth, north / 600.0, east / 600.0);
        let size = (1.0 + s.size_varies * wobble(0x512e)).max(0.3);
        let v = s.shape_varies;
        let shape = Shape {
            edge: (shape.edge + v * 0.5 * wobble(0x5ed6)).clamp(0.0, 1.0),
            flat: (shape.flat + v * 0.5 * wobble(0xf1a7)).clamp(0.0, 0.9),
            bowl: shape.bowl * 2f32.powf(v * wobble(0xb0e1)),
        };
        ((width * size).max(1.0), (depth * size).max(1.0), shape)
    };
    let path_point = |i: usize, (north, east): (f32, f32), level: f32| {
        let (width, depth, shape) = size_at(i, north, east);
        PathPoint { north, east, width, depth, shape, level }
    };

    // The reaches: from each spring or junction down to the next junction, lake, or the sea.
    let starts: Vec<usize> = (0..n).filter(|&i| on(i) && ups[i].len() != 1).collect();
    // Each river reach's planned path (patch to patch), its number and rank.
    let mut planned: Vec<(Vec<PathPoint>, u64, u32)> = Vec::new();
    // The lakes first (filled on the real ground), so the rivers into them end at their water.
    let (lakes, pools) = fill_lakes(flow, blocks, ground, s, stop)?;
    // For each river reach: where it starts and ends (patches), the water's level at its end if it's
    // the sea's or a lake's, and how far its start is from the mouth (deltas aren't listed: they keep
    // the sea's level).
    let mut ends_of: Vec<(usize, usize, Option<f32>, f32)> = Vec::new();
    let mut deltas = Vec::new();
    for (k, &start) in starts.iter().enumerate() {
        if k % 1024 == 0 && stop.load(Ordering::Relaxed) {
            return None;
        }
        let mut path = Vec::new();
        let mut i = start;
        let (mut end, mut end_level) = (start, None);
        loop {
            path.push(path_point(i, place(i), network.bed[i].round()));
            let d = down(i);
            if d == i {
                break;
            }
            if !on(d) {
                // Into the sea (or a lake): the line goes on to its edge, at its water's level; a
                // lake's as it's filled on the real ground, at its water nearest the planned lake
                // [2026-10-30: it ended where the lake was planned, and where the real water lay
                // elsewhere the river just stopped in a field (David's screenshot)].
                let last = path.last().map_or(f32::MAX, |p| p.level);
                if flow.lake[d] == 0.0 {
                    (end, end_level) = (d, Some(SEA_LEVEL as f32));
                    path.push(path_point(i, place(d), (SEA_LEVEL as f32).min(last)));
                    break;
                }
                let (north, east) = place(d);
                if let Some((north, east, surface)) = lakes.nearest_water(north, east) {
                    (end, end_level) = (d, Some(surface.round()));
                    path.push(path_point(i, (north, east), surface.round().min(last)));
                    break;
                }
                // A lake with no water on the real ground: on through it the way it drains, to the
                // river leaving it (joining it there), the sea, or the bottom of a basin.
                let mut j = d;
                for _ in 0..n {
                    if on(j) || flow.lake[j] == 0.0 || down(j) == j {
                        break;
                    }
                    path.push(path_point(i, place(j), last));
                    j = down(j);
                }
                end = j;
                if on(j) {
                    path.push(path_point(i, place(j), network.bed[j].round().min(last)));
                } else if flow.lake[j] == 0.0 {
                    end_level = Some(SEA_LEVEL as f32);
                    path.push(path_point(i, place(j), (SEA_LEVEL as f32).min(last)));
                }
                break;
            }
            if ups[d].len() >= 2 {
                // A junction: this reach ends on it (pinned), the next starts from it; its own size
                // up to there (inside the river it joins) [2026-10-30: it took the junction's, a
                // creek flaring to a river's width over its last patch].
                end = d;
                path.push(path_point(i, place(d), network.bed[d].round()));
                break;
            }
            i = d;
        }
        if path.len() < 2 {
            continue;
        }
        let rank = if s.by_area { rank_by_area(water[start].max(s.spring_area)) } else { network.rank[start].max(1) as u32 };
        ends_of.push((start, end, end_level, to_mouth[start]));
        // A delta, where a river or great river meets the sea over flat ground (with its reach's
        // number).
        if s.deltas
            && rank >= 3
            && let Some(delta) = delta(&path, rank, s)
        {
            deltas.extend(delta.into_iter().map(|channel| (planned.len(), channel)));
        }
        planned.push((path, k as u64, rank));
    }
    // Each reach routed on the real ground (everywhere, only where needed, or not at all) and made a
    // curve, on the cores it may use. The deep cut where needed: from near the sea's to high up's,
    // by the water's level between the sea and the highest river.
    let cores = most_cores(s);
    let each = planned.len().div_ceil(cores).max(1);
    let routing = std::time::Instant::now();
    let (sea, top) = (SEA_LEVEL as f32, planned.iter().filter_map(|(path, ..)| path.first()).map(|p| p.level).fold(SEA_LEVEL as f32 + 1.0, f32::max));
    let x = move |level: f32| s.deep_cut[0] + (s.deep_cut[1] - s.deep_cut[0]) * ((level - sea) / (top - sea)).clamp(0.0, 1.0);
    let made: Vec<Option<(Reach, Found)>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = planned
            .chunks(each)
            .map(|part| {
                let x = &x;
                scope.spawn(move || {
                    part.iter()
                        .map(|(path, _, rank)| {
                            let plan = || path.iter().map(|p| (p.north, p.east)).collect::<Vec<_>>();
                            let (line, found) = match s.route {
                                Route::Everywhere => (route(path, ground, s, stop)?, Found::default()),
                                Route::WhereNeeded => route_where_needed(path, ground, s, &x, stop)?,
                                Route::Off => (plan(), Found::default()),
                            };
                            // (The plan's cut and the line's, for the panel, however it was found.)
                            let found = if s.route == Route::WhereNeeded {
                                found
                            } else {
                                Found { plan_cut: deepest_along(&plan(), ground, f32::MAX), line_cut: deepest_along(&line, ground, f32::MAX), ..found }
                            };
                            Some((curve(&line, path, *rank, s), found))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        // (A part that failed counts as stopped: its reaches can't go missing out of order.)
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_else(|_| vec![None])).collect()
    });
    let made: Vec<(Reach, Found)> = made.into_iter().collect::<Option<_>>()?;
    let route_took = routing.elapsed().as_secs_f32();
    let found = made.iter().fold(Found::default(), |all, (_, f)| Found { deep: all.deep + f.deep, rerouted: all.rerouted + f.rerouted, plan_cut: all.plan_cut.max(f.plan_cut), line_cut: all.line_cut.max(f.line_cut) });
    let mut reaches: Vec<Reach> = made.into_iter().map(|(reach, _)| reach).collect();
    // The water levels, from the real ground (never above the banks, but in a pond behind a rise),
    // only falling downstream.
    let mut laid = lay_water(&mut reaches, &ends_of, ground, step, fall_from, s, stop)?;
    let merged = if s.merge_alongside { merge_alongside(&mut reaches, &ends_of, fall_from) } else { vec![false; reaches.len()] };
    for (cut, merged) in laid.cut_short.iter_mut().zip(&merged) {
        *cut |= merged;
    }
    // The deltas, but where their river met the sea before its planned mouth, or where the real
    // ground at their head isn't low by the sea (they'd carve slots at the sea's level).
    let low_by_the_sea = |p: &RiverPoint| ground.height(p.north, p.east).is_none_or(|h| h < SEA_LEVEL as f32 + step.max(1.0));
    reaches.extend(deltas.into_iter().filter(|(r, channel)| !laid.cut_short[*r] && low_by_the_sea(&channel.points[0])).map(|(_, channel)| channel));
    let rise_ponds = laid.ponds.len();
    let ponds = laid.ponds;
    let lakes = if ponds.is_empty() { lakes } else { Lakes::new(lakes.lakes.into_iter().chain(ponds).collect()) };
    let mut carving = Carving::new(reaches, lakes, s.banks);
    (carving.curtains, carving.anti_kitty_corner) = (s.curtains, s.anti_kitty_corner);
    carving.shores = measure_shores(&carving, ground, cores, stop);
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    (carving.took, carving.deepest_cut, carving.deep_cuts, carving.pools) = (started.elapsed().as_secs_f32(), laid.deepest_cut, laid.deep_cuts, pools);
    (carving.deepest_fill, carving.filled) = (laid.deepest_fill, laid.filled);
    (carving.rise_ponds, carving.cut_instead, carving.into_water) = (rise_ponds, laid.cut_instead, laid.into_water);
    (carving.deep_stretches, carving.rerouted, carving.plan_cut, carving.line_cut, carving.route_took) = (found.deep, found.rerouted, found.plan_cut, found.line_cut, route_took);
    carving.merged = merged.iter().filter(|&&m| m).count();
    carving.sizes = crate::world::Sizes::of(&carving.reaches);
    Some(carving)
}

/// Where a river's water passes below the lowest ground across it by more than this (blocks), it's
/// counted as a deep cut (for the panel).
const DEEP_CUT: f32 = 6.0;

/// The shores are measured at every this many points of each river.
const SHORE_EVERY: usize = 8;

/// The rivers' shores and the land beside them (`Shores`), sampled at every `SHORE_EVERY`th point of
/// each river above the sea (mouths and deltas, at the sea's level, are beaches), on both sides:
/// walking out from its line a block at a time, the first dry column is the shore; then its bank,
/// out to as far as its fade could widen. `ground` is the uncarved ground; the columns are read as
/// the world reads them (`CompiledMap::ground_and_water`), on the cores it may use.
fn measure_shores(carving: &Carving, ground: &CompiledMap, cores: usize, stop: &AtomicBool) -> Shores {
    let sea = SEA_LEVEL as f32;
    // A block's column: its uncarved and carved ground, and the water standing in it.
    let column = |north: f32, east: f32| -> Option<(f32, f32, Option<i32>)> {
        let (north, east) = (north.floor() + 0.5, east.floor() + 0.5);
        let uncarved = ground.height(north, east)?;
        let here = carving.here(north, east, uncarved);
        let mut z = uncarved;
        if let Some(floor) = here.floor {
            z = z.min(floor);
        }
        if let Some(fill) = here.fill {
            z = z.max(fill);
        }
        Some((uncarved, z, here.water.filter(|&water| water as f32 > z.round() + 1.0)))
    };
    let widest = carving.banks.widest.max(1.0);
    let each = carving.reaches.len().div_ceil(cores.max(1)).max(1);
    let parts: Vec<Shores> = std::thread::scope(|scope| {
        let jobs: Vec<_> = carving
            .reaches
            .chunks(each)
            .map(|part| {
                let column = &column;
                scope.spawn(move || {
                    let mut m = Shores::default();
                    for reach in part {
                        let p = &reach.points;
                        for k in (0..p.len()).step_by(SHORE_EVERY) {
                            if stop.load(Ordering::Relaxed) {
                                return m;
                            }
                            if p[k].level <= sea + 0.5 {
                                continue;
                            }
                            let (a, b) = (p[k.saturating_sub(1)], p[(k + 1).min(p.len() - 1)]);
                            let (tn, te) = (b.north - a.north, b.east - a.east);
                            let length = (tn * tn + te * te).sqrt().max(1e-3);
                            let (nn, ne) = (te / length, -tn / length);
                            let at = |o: f32| (p[k].north + nn * o, p[k].east + ne * o);
                            for side in [-1.0f32, 1.0] {
                                // Out across the water to the first dry column.
                                let (mut water, mut shore) = (p[k].level.round() as i32, None);
                                let mut o = 0.5;
                                while o <= p[k].width / 2.0 + 4.0 {
                                    let (n, e) = at(o * side);
                                    match column(n, e) {
                                        Some((_, _, Some(w))) => water = w,
                                        Some((_, z, None)) => {
                                            shore = Some((o, z));
                                            break;
                                        }
                                        None => break,
                                    }
                                    o += 1.0;
                                }
                                let Some((from, z)) = shore else { continue };
                                if under_sea(z) {
                                    continue;
                                }
                                match z.round() as i32 + 1 - water {
                                    ..=-1 => m.below += 1,
                                    0 => m.flush += 1,
                                    1 => m.ledge += 1,
                                    _ => m.bank += 1,
                                }
                                // Across its bank: a block at a time near the water, then in eighths.
                                let out = p[k].fade * widest;
                                let mut last: Option<(f32, f32, bool)> = None;
                                let mut o = from;
                                while o <= from + out {
                                    let (n, e) = at(o * side);
                                    let Some((uncarved, z, None)) = column(n, e) else { break };
                                    let change = z - uncarved;
                                    m.columns += 1;
                                    if change < 0.0 {
                                        m.lowered += -change as f64;
                                        m.most_lowered = m.most_lowered.max(-change);
                                    } else {
                                        m.raised += change as f64;
                                        m.most_raised = m.most_raised.max(change);
                                    }
                                    let changed = change.abs() >= 0.5;
                                    if let Some((lo, lz, was)) = last
                                        && (changed || was)
                                    {
                                        m.steepest = m.steepest.max((z - lz).abs() / (o - lo));
                                    }
                                    last = Some((o, z, changed));
                                    o += if o - from < 6.0 { 1.0 } else { (out / 6.0).max(1.0) };
                                }
                            }
                        }
                    }
                    m
                })
            })
            .collect();
        jobs.into_iter().map(|job| job.join().unwrap_or_default()).collect()
    });
    parts.into_iter().fold(Shores::default(), Shores::merged)
}

/// The lakes, each filled on the real ground [David, 2026-10-30: "perfect elevated rectangular
/// swimming pools with a chunk of square perimeter"; they were step 5's surface on each 32-block
/// patch of the broad ground]. Round step 5's lakes (their patches and `lake_reach` blocks round
/// them, lakes that near each other taken together) the real ground is read on cells `grain` blocks
/// across and filled and spilled from the edge, as the rivers' routes are: each dip fills to its
/// lowest gap, whatever its shape. A pool at least `pond_depth` deep at its deepest is kept. With how
/// many pools; `None` if stopped.
fn fill_lakes(flow: &Flow, blocks: f32, ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<(Lakes, usize)> {
    let w = flow.width;
    // The regions: the lake patches grown by the reach, joined where they touch.
    let grow = (s.lake_reach.max(0.0) / blocks).ceil() as i64;
    let mut near = vec![false; w * w];
    for i in (0..w * w).filter(|&i| flow.lake[i] > 0.0) {
        let (c, r) = ((i % w) as i64, (i / w) as i64);
        for dr in -grow..=grow {
            for dc in -grow..=grow {
                let (c, r) = (c + dc, r + dr);
                if c >= 0 && r >= 0 && c < w as i64 && r < w as i64 {
                    near[r as usize * w + c as usize] = true;
                }
            }
        }
    }
    let mut region = vec![u32::MAX; w * w];
    let mut regions: Vec<Vec<usize>> = Vec::new();
    for i in 0..w * w {
        if !near[i] || region[i] != u32::MAX {
            continue;
        }
        let k = regions.len() as u32;
        let mut patches = vec![i];
        region[i] = k;
        let mut j = 0;
        while j < patches.len() {
            let (c, r) = ((patches[j] % w) as i64, (patches[j] / w) as i64);
            for (dc, dr) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (c, r) = (c + dc, r + dr);
                if c >= 0 && r >= 0 && c < w as i64 && r < w as i64 {
                    let next = r as usize * w + c as usize;
                    if near[next] && region[next] == u32::MAX {
                        region[next] = k;
                        patches.push(next);
                    }
                }
            }
            j += 1;
        }
        regions.push(patches);
    }
    // Each region filled, on the cores it may use.
    let cores = most_cores(s);
    let each = regions.len().div_ceil(cores).max(1);
    let region = &region;
    let filled: Vec<Option<(Lake, usize)>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = regions
            .chunks(each)
            .enumerate()
            .map(|(part, chunk)| scope.spawn(move || chunk.iter().enumerate().map(|(k, patches)| fill_region(patches, (part * each + k) as u32, region, w, blocks, ground, s, stop)).collect::<Vec<_>>()))
            .collect();
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_else(|_| vec![None])).collect()
    });
    let filled: Vec<(Lake, usize)> = filled.into_iter().collect::<Option<_>>()?;
    let pools = filled.iter().map(|(_, pools)| pools).sum();
    Some((Lakes::new(filled.into_iter().map(|(lake, _)| lake).filter(|lake| !lake.is_dry()).collect()), pools))
}

/// One lake region (`k`, its patches) filled on the real ground: see `fill_lakes`.
#[allow(clippy::too_many_arguments)]
fn fill_region(patches: &[usize], k: u32, region: &[u32], w: usize, blocks: f32, ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<(Lake, usize)> {
    let (sea, half) = (SEA_LEVEL as f32, w as f32 / 2.0);
    let (c0, c1) = patches.iter().map(|&i| i % w).fold((usize::MAX, 0), |(a, b), c| (a.min(c), b.max(c)));
    let (r0, r1) = patches.iter().map(|&i| i / w).fold((usize::MAX, 0), |(a, b), r| (a.min(r), b.max(r)));
    // Its grid (coarser for a very big lake, to keep it small).
    let mut cell = s.grain.max(1.0);
    let size = |cell: f32| (((c1 - c0 + 1) as f32 * blocks / cell).ceil() as usize, ((r1 - r0 + 1) as f32 * blocks / cell).ceil() as usize);
    while size(cell).0 * size(cell).1 > 4_000_000 {
        cell *= 2.0;
    }
    let (cols, rows) = size(cell);
    let (north, west) = ((half - r0 as f32) * blocks, (c0 as f32 - half) * blocks);
    let centre = |i: usize| (north - ((i / cols) as f32 + 0.5) * cell, west + ((i % cols) as f32 + 0.5) * cell);
    // Its cells' real ground (NaN outside the region).
    let mut height = vec![f32::NAN; cols * rows];
    for (i, h) in height.iter_mut().enumerate() {
        if i % 65_536 == 0 && stop.load(Ordering::Relaxed) {
            return None;
        }
        let (n, e) = centre(i);
        let (pc, pr) = ((e / blocks + half).floor(), (half - n / blocks).floor());
        if pc >= 0.0 && pr >= 0.0 && (pc as usize) < w && (pr as usize) < w && region[pr as usize * w + pc as usize] == k {
            *h = ground.height(n, e).unwrap_or(sea);
        }
    }
    // Fill and spill from its edge.
    let neighbours = |i: usize| {
        let (c, r) = ((i % cols) as i64, (i / cols) as i64);
        [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)].map(|(dc, dr)| {
            let (c, r) = (c + dc, r + dr);
            (c >= 0 && r >= 0 && c < cols as i64 && r < rows as i64).then(|| r as usize * cols + c as usize)
        })
    };
    let mut level = vec![f32::NAN; cols * rows];
    let mut waiting = BinaryHeap::new();
    for i in (0..cols * rows).filter(|&i| height[i].is_finite()) {
        if neighbours(i).iter().any(|n| n.is_none_or(|n| height[n].is_nan())) {
            level[i] = height[i];
            waiting.push(Reverse(Waiting { level: height[i], run: 0.0, at: i as u32 }));
        }
    }
    while let Some(Reverse(Waiting { level: at_level, at, .. })) = waiting.pop() {
        for next in neighbours(at as usize).into_iter().flatten() {
            if height[next].is_finite() && level[next].is_nan() {
                level[next] = height[next].max(at_level);
                waiting.push(Reverse(Waiting { level: level[next], run: 0.0, at: next as u32 }));
            }
        }
    }
    // The pools (water standing half a block or more, above the sea), each kept if deep enough.
    let water = |i: usize| level[i].is_finite() && level[i] - height[i] >= 0.5 && level[i] > sea;
    let mut surface = vec![f32::NAN; cols * rows];
    let mut seen = vec![false; cols * rows];
    let mut pools = 0;
    for i in 0..cols * rows {
        if seen[i] || !water(i) {
            continue;
        }
        let mut pool = vec![i];
        seen[i] = true;
        let mut j = 0;
        while j < pool.len() {
            for next in neighbours(pool[j]).into_iter().flatten() {
                if !seen[next] && water(next) {
                    seen[next] = true;
                    pool.push(next);
                }
            }
            j += 1;
        }
        if pool.iter().map(|&p| level[p] - height[p]).fold(0.0, f32::max) >= s.pond_depth {
            pools += 1;
            for &p in &pool {
                surface[p] = level[p];
            }
        }
    }
    Some((Lake::new(north, west, cols, rows, cell, surface), pools))
}

/// A cell waiting to be reached by the route's fill and spill: lowest first, then the nearer the
/// reach's end (so it runs the shortest way across level ground), then by place.
#[derive(PartialEq)]
struct Waiting {
    level: f32,
    run: f32,
    at: u32,
}

impl Eq for Waiting {}

impl PartialOrd for Waiting {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Waiting {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level.total_cmp(&other.level).then(self.run.total_cmp(&other.run)).then(self.at.cmp(&other.at))
    }
}

/// How far a spot is from a piece of line (blocks).
fn to_piece(north: f32, east: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dn, de) = (b.0 - a.0, b.1 - a.1);
    let t = (((north - a.0) * dn + (east - a.1) * de) / (dn * dn + de * de).max(1e-6)).clamp(0.0, 1.0);
    ((north - a.0 - dn * t).powi(2) + (east - a.1 - de * t).powi(2)).sqrt()
}

/// One reach's route on the real ground [David, 2026-10-30: "it's got to follow the worlds final
/// actual detailed terrain, curving between hills instead of slicing through them", rolling "like a
/// marble between hills"; Counsel: fill and spill, as step 5, run locally]. The cells `grain` blocks
/// across within `corridor` blocks of its planned path are reached from its end, the lowest cell not
/// yet reached always next, so a dip fills to its lowest gap and spills; the route is the way its
/// start's water then runs. Its ends are the planned path's (pinned, so rivers meet). `None` if
/// stopped.
/// How finely a line is read for the cut its water would need (blocks).
const CUT_STEP: f32 = 8.0;

/// What finding the way did on a reach: its stretches deeper than the deep cut on its plan, how
/// many were found a better way round, and the deepest cut its plan and its line would need (blocks;
/// estimated, `cuts_along`).
#[derive(Clone, Copy, Debug, Default)]
struct Found {
    deep: usize,
    rerouted: usize,
    plan_cut: f32,
    line_cut: f32,
}

/// The cut the water at each of a line's points would need: the ground under it against the water
/// only falling from `from` (each spot's level the lowest ground so far): (cut, level).
fn cuts_along(points: &[(f32, f32)], ground: &CompiledMap, from: f32) -> Vec<(f32, f32)> {
    let mut level = from;
    points
        .iter()
        .map(|&(north, east)| {
            let here = ground.height(north, east).unwrap_or(SEA_LEVEL as f32);
            level = level.min(here);
            (here - level, level)
        })
        .collect()
}

/// The deepest cut along a line, read every `CUT_STEP` blocks, the water falling from `from`.
fn deepest_along(line: &[(f32, f32)], ground: &CompiledMap, from: f32) -> f32 {
    cuts_along(&resample(line, CUT_STEP), ground, from).iter().map(|c| c.0).fold(0.0, f32::max)
}

/// The way found only where it's needed [stage 4 of `docs/water-rework.md`]: the plan, read every
/// `CUT_STEP` blocks for the cut its water would need; where that's deeper than the deep cut (`x`,
/// by the water's level), a box round the stretch, out to where the cut is small again, is filled
/// and spilled as `route` does (its corridor half the stretch's length, or 4 times its deepest cut
/// if more, 32 to 256 blocks, at an 8-block grain), and its way kept if it cuts less. Elsewhere the
/// plan stands. `None` if stopped.
fn route_where_needed(path: &[PathPoint], ground: &CompiledMap, s: &CarvingSettings, x: &dyn Fn(f32) -> f32, stop: &AtomicBool) -> Option<(Vec<(f32, f32)>, Found)> {
    let plan = resample(&path.iter().map(|p| (p.north, p.east)).collect::<Vec<_>>(), CUT_STEP);
    let cuts = cuts_along(&plan, ground, f32::MAX);
    let mut found = Found { plan_cut: cuts.iter().map(|c| c.0).fold(0.0, f32::max), ..Found::default() };
    // The deep stretches, each out to where the cut is small again, and a point more either side.
    let small = |k: usize| cuts[k].0 <= (x(cuts[k].1) / 2.0).min(2.0);
    let mut stretches: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < plan.len() {
        if cuts[i].0 <= x(cuts[i].1) {
            i += 1;
            continue;
        }
        let (mut a, mut b) = (i, i);
        while a > 0 && !small(a) {
            a -= 1;
        }
        while b + 1 < cuts.len() && !small(b) {
            b += 1;
        }
        let (a, b) = (a.saturating_sub(1), (b + 1).min(plan.len() - 1));
        match stretches.last_mut() {
            Some(last) if a <= last.1 => last.1 = last.1.max(b),
            _ => stretches.push((a, b)),
        }
        i = b + 1;
    }
    found.deep = stretches.len();
    let mut line = Vec::with_capacity(plan.len());
    let mut next = 0;
    for &(a, b) in &stretches {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        line.extend_from_slice(&plan[next..a]);
        let old = &plan[a..=b];
        let deepest = cuts[a..=b].iter().map(|c| c.0).fold(0.0, f32::max);
        let boxed = CarvingSettings { corridor: ((b - a) as f32 * CUT_STEP / 2.0).max(deepest * 4.0).clamp(32.0, 256.0), grain: 8.0, ..s.clone() };
        let sub: Vec<PathPoint> = old.iter().map(|&(north, east)| PathPoint { north, east, ..path[0] }).collect();
        let new = route(&sub, ground, &boxed, stop)?;
        // (The water arrives at the stretch at the plan's level there.)
        let from = cuts[a].1;
        if deepest_along(&new, ground, from) + 0.5 < deepest_along(old, ground, from) {
            line.extend_from_slice(&new);
            found.rerouted += 1;
        } else {
            line.extend_from_slice(old);
        }
        next = b + 1;
    }
    line.extend_from_slice(&plan[next..]);
    found.line_cut = deepest_along(&line, ground, f32::MAX);
    Some((line, found))
}

fn route(path: &[PathPoint], ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<Vec<(f32, f32)>> {
    let (start, end) = ((path[0].north, path[0].east), (path[path.len() - 1].north, path[path.len() - 1].east));
    let planned = || path.iter().map(|p| (p.north, p.east)).collect::<Vec<_>>();
    let corridor = s.corridor.max(1.0);
    let (north, south) = path.iter().fold((f32::MIN, f32::MAX), |(n, s), p| (n.max(p.north), s.min(p.north)));
    let (west, east) = path.iter().fold((f32::MAX, f32::MIN), |(w, e), p| (w.min(p.east), e.max(p.east)));
    // The grid over the path and its corridor (coarser for a very long reach, to keep it small).
    let mut grain = s.grain.max(1.0);
    let span = |grain: f32| (((east - west + 2.0 * corridor) / grain).ceil() as usize + 2, ((north - south + 2.0 * corridor) / grain).ceil() as usize + 2);
    while span(grain).0 * span(grain).1 > 4_000_000 {
        grain *= 2.0;
    }
    let (cols, rows) = span(grain);
    let (top, left) = (north + corridor + grain, west - corridor - grain);
    let cell = |n: f32, e: f32| (((top - n) / grain).floor().clamp(0.0, rows as f32 - 1.0) as usize) * cols + ((e - left) / grain).floor().clamp(0.0, cols as f32 - 1.0) as usize;
    let centre = |i: usize| (top - ((i / cols) as f32 + 0.5) * grain, left + ((i % cols) as f32 + 0.5) * grain);
    // The corridor: each cell within its distance of a piece of the path, with its real ground.
    let mut height = vec![f32::NAN; cols * rows];
    for pair in path.windows(2) {
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        let (a, b) = ((pair[0].north, pair[0].east), (pair[1].north, pair[1].east));
        let (first, last) = (cell(a.0.max(b.0) + corridor, a.1.min(b.1) - corridor), cell(a.0.min(b.0) - corridor, a.1.max(b.1) + corridor));
        for row in first / cols..=last / cols {
            for col in first % cols..=last % cols {
                let i = row * cols + col;
                let (n, e) = centre(i);
                if height[i].is_nan() && to_piece(n, e, a, b) <= corridor {
                    height[i] = ground.height(n, e).unwrap_or(SEA_LEVEL as f32);
                }
            }
        }
    }
    let (from, to) = (cell(start.0, start.1), cell(end.0, end.1));
    if from == to || height[from].is_nan() || height[to].is_nan() {
        return Some(planned());
    }
    // Fill and spill from the end, until the start is reached.
    let mut down = vec![u32::MAX; cols * rows];
    let mut waiting = BinaryHeap::new();
    down[to] = to as u32;
    waiting.push(Reverse(Waiting { level: height[to], run: 0.0, at: to as u32 }));
    while let Some(Reverse(Waiting { level, run, at })) = waiting.pop() {
        let at = at as usize;
        if at == from {
            break;
        }
        let (col, row) = ((at % cols) as i64, (at / cols) as i64);
        for (dc, dr) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let (c, r) = (col + dc, row + dr);
            if c < 0 || r < 0 || c >= cols as i64 || r >= rows as i64 {
                continue;
            }
            let next = r as usize * cols + c as usize;
            if down[next] != u32::MAX || height[next].is_nan() {
                continue;
            }
            down[next] = at as u32;
            let step = if dc != 0 && dr != 0 { std::f32::consts::SQRT_2 } else { 1.0 };
            waiting.push(Reverse(Waiting { level: height[next].max(level), run: run + step, at: next as u32 }));
        }
    }
    if down[from] == u32::MAX {
        return Some(planned());
    }
    let mut line = vec![start];
    let mut at = down[from] as usize;
    while at != to {
        line.push(centre(at));
        at = down[at] as usize;
    }
    line.push(end);
    Some(line)
}

/// One reach's curve along its route: smoothed (corner cut over points about a width apart), with
/// the width, depth and shape by how far along its planned path. (Its water level is laid after,
/// `lay_water`.) [2026-10-31, David: the wander, the lean to the outside of bends and the meanders
/// "only seem to make it worse at any setting besides off": each moved the line off the way found,
/// onto the higher ground beside it. Cut.]
fn curve(route: &[(f32, f32)], path: &[PathPoint], rank: u32, s: &CarvingSettings) -> Reach {
    // How far along the planned path each of its points is.
    let mut along = vec![0.0f32];
    for pair in path.windows(2) {
        along.push(along.last().unwrap() + ((pair[1].north - pair[0].north).powi(2) + (pair[1].east - pair[0].east).powi(2)).sqrt());
    }
    let total = along.last().copied().unwrap_or(0.0).max(1e-3);
    let widest = path.iter().map(|p| p.width).fold(1.0f32, f32::max);
    // 1. Corner cutting, over the route's points about a width apart (so bigger rivers bend wider),
    // keeping the ends.
    let mut line = resample(route, widest.clamp(2.0 * s.grain.max(1.0), 48.0));
    for _ in 0..(s.smoothing + rank.saturating_sub(1)).min(5) {
        if line.len() < 3 {
            break;
        }
        let mut cut = vec![line[0]];
        for pair in line.windows(2) {
            let ((an, ae), (bn, be)) = (pair[0], pair[1]);
            cut.push((an * 0.75 + bn * 0.25, ae * 0.75 + be * 0.25));
            cut.push((an * 0.25 + bn * 0.75, ae * 0.25 + be * 0.75));
        }
        cut.push(*line.last().unwrap());
        line = cut;
    }
    // Even spacing along the smoothed line, a few blocks apart.
    let spacing = (widest / 2.0).clamp(2.0, 6.0);
    let line = resample(&line, spacing);
    let length = line.windows(2).map(|p| ((p[1].0 - p[0].0).powi(2) + (p[1].1 - p[0].1).powi(2)).sqrt()).sum::<f32>().max(1e-3);
    // The river's attributes at a share of the way along (0 at its start, 1 at its end).
    let at = |share: f32| {
        let s_path = share * total;
        let k = along.iter().rposition(|&a| a <= s_path).unwrap_or(0).min(path.len() - 2);
        let t = ((s_path - along[k]) / (along[k + 1] - along[k]).max(1e-3)).clamp(0.0, 1.0);
        let (a, b) = (&path[k], &path[k + 1]);
        let mix = |x: f32, y: f32| x + (y - x) * t;
        (mix(a.width, b.width), mix(a.depth, b.depth), a.shape.mix(b.shape, t))
    };
    let mut points = Vec::with_capacity(line.len());
    let mut run = 0.0f32;
    for k in 0..line.len() {
        if k > 0 {
            run += ((line[k].0 - line[k - 1].0).powi(2) + (line[k].1 - line[k - 1].1).powi(2)).sqrt();
        }
        let (width, depth, shape) = at(run / length);
        // How sharply it bends here.
        let bend = if k > 0 && k + 1 < line.len() {
            let (p, q, r) = (line[k - 1], line[k], line[k + 1]);
            let cross = (q.0 - p.0) * (r.1 - q.1) - (q.1 - p.1) * (r.0 - q.0);
            // Positive: turning right (looking downstream), its outside to the left.
            -cross / (spacing * spacing)
        } else {
            0.0
        };
        points.push(RiverPoint {
            north: line[k].0,
            east: line[k].1,
            width,
            depth,
            edge: shape.edge,
            flat: shape.flat,
            bowl: shape.bowl,
            level: f32::NAN,
            bend: if bend.abs() < 0.002 { 0.0 } else { bend.signum() },
            curtain: f32::NAN,
            // Its width over the bend's radius (the bend turns `bend` radians a point, `spacing`
            // blocks apart).
            turn: bend * width / spacing,
            fade: (width * s.fade_for(rank)).max(s.fade_least),
        });
    }
    Reach { points }
}

/// The rivers' water levels, from the real ground [2026-10-30: taken from the network's broad
/// ground, the water stood above the real ground wherever hills dipped below it or the line swung
/// off its path, a ribbon of water walls (David's screenshot)]. Each point's level is the lowest real
/// ground across the river's width there, so the water is never above its banks; then only falling
/// downstream (a reach's start no higher than what flows into it, upstream reaches first); then in
/// steps of the bed's step size relative to that ground (flat runs, then drops); a drop as tall as a
/// waterfall stays sheer (its curtain just below), a smaller one is spread upstream as rapids, a
/// block at a time; then each waterfall is dropped at its lip. Never below the sea's surface, and a
/// reach into the sea or a lake ends at its level, and any reach ends where it first meets the sea.
/// Ponds can't stand where they'd drain into another river. `None` if stopped.
fn lay_water(reaches: &mut [Reach], ends_of: &[(usize, usize, Option<f32>, f32)], ground: &CompiledMap, step: f32, fall_from: f32, s: &CarvingSettings, stop: &AtomicBool) -> Option<Laid> {
    let sea = SEA_LEVEL as f32;
    // The lowest real ground across each point's width (its middle and both banks), on the cores it
    // may use.
    let cores = most_cores(s);
    let each = reaches.len().div_ceil(cores).max(1);
    // With least change, also the lower bank's ground, 3 blocks out from the water either side: the
    // water rises over a hollow no higher than that, so it fills only where the land rises on both
    // sides across it [David, 2026-10-31: a river filled over a valley stood on a causeway, and
    // along a hillside on an embankment on its low side].
    let least = s.level == Level::LeastChange;
    let sampled: Vec<Vec<(f32, f32)>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = reaches
            .chunks(each)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|reach| {
                            let p = &reach.points;
                            (0..p.len())
                                .map(|k| {
                                    if stop.load(Ordering::Relaxed) {
                                        return (sea, f32::MAX);
                                    }
                                    let (a, b) = (p[k.saturating_sub(1)], p[(k + 1).min(p.len() - 1)]);
                                    let (tn, te) = (b.north - a.north, b.east - a.east);
                                    let length = (tn * tn + te * te).sqrt().max(1e-3);
                                    let (nn, ne) = (te / length, -tn / length);
                                    let half = p[k].width / 2.0;
                                    let lowest = |offsets: &[f32]| offsets.iter().filter_map(|&o| ground.height(p[k].north + nn * o, p[k].east + ne * o)).fold(f32::MAX, f32::min);
                                    (lowest(&[0.0, half, -half]), if least { lowest(&[half + 3.0, -half - 3.0]) } else { f32::MAX })
                                })
                                .collect()
                        })
                        .collect::<Vec<Vec<(f32, f32)>>>()
                })
            })
            .collect();
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_default()).collect()
    });
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    let mut lows: Vec<Vec<f32>> = sampled.iter().map(|reach| reach.iter().map(|s| s.0).collect()).collect();
    // The most the water may stand at each point: no more than the fill above the lowest ground
    // across, nor above the lower bank.
    let mut ceilings: Vec<Vec<f32>> = sampled.iter().map(|reach| reach.iter().map(|&(low, bank)| (low + s.fill_up_to.max(0.0)).min(bank.max(low) + s.fill_over_banks.max(0.0))).collect()).collect();
    // Downstream: upstream reaches first; each reach no higher than what arrives at its start.
    let mut order: Vec<usize> = (0..reaches.len()).collect();
    order.sort_by(|&a, &b| ends_of[b].3.total_cmp(&ends_of[a].3));
    let mut arriving: std::collections::HashMap<usize, f32> = std::collections::HashMap::new();
    let mut cut_short = vec![false; reaches.len()];
    for &r in &order {
        let (start, end, end_level, _) = ends_of[r];
        let mut cap = arriving.get(&start).copied().unwrap_or(f32::MAX);
        let floor = end_level.unwrap_or(sea).max(sea);
        // (With least change, the levels that change the land least, which the water then follows.)
        let target = (s.level == Level::LeastChange).then(|| least_change(&lows[r], &ceilings[r], cap, s.cut_most));
        let points = &mut reaches[r].points;
        for k in 0..points.len() {
            let low = if lows[r][k] == f32::MAX { cap } else { lows[r][k] };
            let mut level = target.as_ref().map_or(low, |target| target[k]).min(cap).max(floor).floor();
            // A reach ends where it first meets the sea: at its first point with the sea under it,
            // or with its water at the sea's level on ground more than a step above it (a cliff top,
            // the sea or a beach at its foot) [2026-10-31: where the broad ground had land but the
            // real coast came first, a river ran on along a cliff top, dragged down to the sea's
            // level by a bank over the edge: a slot up to 57 blocks deep]. (Across a low plain at the
            // sea's level, it carries on.)
            let meets_sea = k > 0 && k + 1 < points.len() && (low < sea - 0.5 || (level <= sea && ground.height(points[k].north, points[k].east).is_some_and(|h| h > sea + step.max(1.0))));
            if meets_sea {
                level = sea;
            } else if k + 1 == points.len()
                && let Some(at) = end_level
            {
                level = at.max(sea);
            }
            cap = level;
            points[k].level = level;
            points[k].curtain = f32::NAN;
            if meets_sea {
                points.truncate(k + 1);
                lows[r].truncate(k + 1);
                ceilings[r].truncate(k + 1);
                cut_short[r] = true;
                break;
            }
        }
        // (Cut short, it no longer reaches the water at its end.)
        if !cut_short[r] {
            let last = points.last().map_or(sea, |p| p.level);
            arriving.entry(end).and_modify(|l| *l = l.min(last)).or_insert(last);
        }
    }
    // Which rises hold a pond and which are cut through, each reach on its own, on the cores.
    let ponds: Vec<Ponds> = if s.rise_pools {
        let each = reaches.len().div_ceil(cores).max(1);
        let reaches = &*reaches;
        let lows = &lows;
        let water = &Water::new(reaches, ends_of, s.grain.max(1.0));
        let made: Vec<Option<Ponds>> = std::thread::scope(|scope| {
            let jobs: Vec<_> = (0..reaches.len())
                .collect::<Vec<_>>()
                .chunks(each)
                .map(|part| {
                    let part = part.to_vec();
                    scope.spawn(move || part.into_iter().map(|r| ponds_of(&reaches[r].points, &lows[r], water, r, ground, s, stop)).collect::<Vec<_>>())
                })
                .collect();
            jobs.into_iter().flat_map(|job| job.join().unwrap_or_else(|_| vec![None])).collect()
        });
        made.into_iter().collect::<Option<_>>()?
    } else {
        reaches.iter().map(|reach| Ponds { cut: vec![false; reach.points.len()], too_far: 0, into_water: 0, lakes: Vec::new() }).collect()
    };
    // Then downstream reaches first: each ends at the level of the water it joins (at a junction
    // each reach's level comes from the ground across its own width, so a creek could end blocks
    // above the river it meets), then its steps and drops.
    let from: std::collections::HashMap<usize, usize> = ends_of.iter().enumerate().map(|(r, e)| (e.0, r)).collect();
    let held = Lakes::new(ponds.iter().flat_map(|p| p.lakes.iter().cloned()).collect());
    for &r in order.iter().rev() {
        let (_, end, end_level, _) = ends_of[r];
        // (The water it joins, and that river's width there.)
        let joined = if end_level.is_none() && !cut_short[r] { from.get(&end).and_then(|&next| reaches[next].points.first()).map(|p| (p.level, p.width)) } else { None };
        let joins = joined.map(|(level, _)| level);
        let points = &mut reaches[r].points;
        if let (Some(at), Some(last)) = (joins, points.last_mut()) {
            // (Where the water it joins is held up in a pond, it backs up into this reach too.)
            let pond = held.water(last.north, last.east).is_some_and(|w| w >= at);
            last.level = if pond { at } else { last.level.min(at) };
        }
        // Ponds behind rises: the water held up to its rise's height less the cut, up to each rise
        // that keeps its pond, from the end upward.
        if s.rise_pools {
            let mut need = points.last().map_or(f32::MIN, |p| p.level);
            for k in (0..points.len().saturating_sub(1)).rev() {
                if lows[r][k] < f32::MAX && !ponds[r].cut[k] {
                    need = need.max((lows[r][k] - s.cut_up_to.max(0.0)).floor());
                }
                points[k].level = points[k].level.max(need);
            }
        }
        // Steps, from the reach's end upward: flat until the water would have risen a step above the
        // run, then up to it (like the network's bed) [2026-10-30: rounded down to absolute multiples
        // of the step (Z 128, 132, 136…), the water sat up to 3 blocks more below its banks].
        // With least change, each flat run at the level that changes its land least, not its lowest
        // point's: its points' mean, rounded down, no higher than any of their ceilings [2026-10-31:
        // with steps of 18, each run cut its other points down to its lowest, by up to 17 blocks:
        // two thirds of all points cut more than 6 on David's world].
        if step > 1.0 {
            let mut runs: Vec<(usize, usize)> = Vec::new();
            let mut run = f32::NAN;
            for k in (0..points.len()).rev() {
                if run.is_nan() || points[k].level - run >= step {
                    run = points[k].level;
                    runs.push((k, k));
                } else if let Some(last) = runs.last_mut() {
                    last.0 = k;
                }
            }
            for &(first, last) in &runs {
                let lowest = points[last].level;
                let level = if least {
                    let mean = points[first..=last].iter().map(|p| p.level).sum::<f32>() / (last - first + 1) as f32;
                    let ceiling = ceilings[r][first..=last].iter().copied().fold(f32::MAX, f32::min);
                    mean.floor().min(ceiling.floor()).max(lowest)
                } else {
                    lowest
                };
                for point in &mut points[first..=last] {
                    point.level = level;
                }
            }
        }
        // Each drop: a waterfall (sheer) or rapids (spread upstream, a block at a time). Rapids only
        // over their own run, never past the drop above it, squeezed to fit a short run [2026-10-31:
        // spread past the waterfall above, they pulled its water down after its curtain was set; the
        // curtain and its cliff stood up out of the water, and the real drop moved up with none,
        // walls of water on steep ground ("rapids at 1" only hid it, its spread never reaching the
        // next point)].
        let gap = |a: &RiverPoint, b: &RiverPoint| ((a.north - b.north).powi(2) + (a.east - b.east).powi(2)).sqrt();
        for k in 1..points.len() {
            let drop = points[k - 1].level - points[k].level;
            if (1.0..fall_from).contains(&drop) {
                let (lower, upper) = (points[k].level, points[k - 1].level);
                // The run above it: back to the foot of the drop above (its top keeps its level).
                let (mut room, mut top) = (0.0, k - 1);
                while top > 0 && (points[top - 1].level - upper).abs() < 0.5 {
                    room += gap(&points[top], &points[top - 1]);
                    top -= 1;
                }
                let spread = (drop * s.rapids_length).max(1.0).min(room);
                if spread <= 0.0 {
                    continue;
                }
                let mut run = 0.0;
                let mut j = k - 1;
                loop {
                    let into = run / spread;
                    if into >= 1.0 {
                        break;
                    }
                    points[j].level = points[j].level.min(lower + (drop * into).ceil());
                    if j == 0 {
                        break;
                    }
                    run += gap(&points[j], &points[j - 1]);
                    j -= 1;
                }
            }
        }
        // Where it joins another river, it eases down toward that river's water over its last stretch
        // (`join_fade` of that river's widths), as steeply as rapids go (a block of drop for every
        // `rapids_length` of river); what that can't take is a fall at the shore. Only lowering.
        if let Some((at, width)) = joined
            && s.join_fade > 0.0
            && points.len() > 1
        {
            let fade = s.join_fade * width.max(1.0);
            let mut from_end = vec![0.0f32; points.len()];
            for k in (0..points.len() - 1).rev() {
                from_end[k] = from_end[k + 1] + gap(&points[k], &points[k + 1]);
            }
            if let Some(start) = (1..points.len()).find(|&k| from_end[k] <= fade) {
                let top = points[start - 1].level;
                let eased = (top - at).max(0.0).min(fade / s.rapids_length.max(1.0));
                for k in start..points.len() - 1 {
                    let into = 1.0 - from_end[k] / fade;
                    points[k].level = points[k].level.min((top - eased * into).floor().max(at));
                }
            }
        }
        // Then each waterfall keeps the water above it, for its curtain and its banks: every drop as
        // tall as a waterfall, and every drop of 2 or more still standing sheer, where rapids had no
        // room to spread [2026-10-31, David's screenshot: with no steps and falls from 25, rapids at
        // every point had no run to spread over, 4,895 sheer drops, walls of water up to 24 tall].
        for k in 1..points.len() {
            if points[k - 1].level - points[k].level >= fall_from.min(2.0) {
                points[k].curtain = points[k - 1].level;
            }
        }
    }
    // How far the water passes below the lowest ground across it: the deepest, and how many points
    // deeper than a deep cut.
    let cuts = reaches.iter().zip(&lows).flat_map(|(reach, low)| reach.points.iter().zip(low).filter(|&(_, &l)| l < f32::MAX).map(|(p, &l)| l - p.level));
    let (deepest_cut, deep_cuts, deepest_fill, filled) = cuts.fold((0.0f32, 0usize, 0.0f32, 0usize), |(deepest, deep, highest, filled), cut| (deepest.max(cut), deep + (cut > DEEP_CUT) as usize, highest.max(-cut), filled + (cut < -0.5) as usize));
    let (too_far, into_water): (usize, usize) = (ponds.iter().map(|p| p.too_far).sum(), ponds.iter().map(|p| p.into_water).sum());
    // Then each waterfall dropped at its lip (points put in, so after everything taken by point).
    falls_at_lips(reaches, ground);
    Some(Laid { deepest_cut, deep_cuts, deepest_fill, filled, ponds: ponds.into_iter().flat_map(|p| p.lakes).collect(), cut_instead: too_far + into_water, into_water, cut_short })
}

/// Rivers running alongside each other merge [David, 2026-10-31: two rivers side by side, their edges
/// touching only at corners, "it should cause a waterfall to merge, shouldnt it?"]. Walking each river
/// down, at its first point whose water comes within a block of another river's water that's lower
/// (or as low, the other nearer the sea), it ends there, falling to that river's level, and the rest
/// of its way alongside is dropped. Clear of its own ends by its width and the widest river's, where
/// it meets its junctions' rivers. The rivers nearest the sea go first, so none merges into water
/// that's since gone. Which reaches merged.
fn merge_alongside(reaches: &mut [Reach], ends_of: &[(usize, usize, Option<f32>, f32)], fall_from: f32) -> Vec<bool> {
    const CELL: f32 = 32.0;
    let cell = |v: f32| (v / CELL).floor() as i64;
    let mut cells: std::collections::HashMap<(i64, i64), Vec<(u32, u32)>> = std::collections::HashMap::new();
    let mut widest = 1.0f32;
    for (r, reach) in reaches.iter().enumerate() {
        for (k, p) in reach.points.iter().enumerate() {
            cells.entry((cell(p.north), cell(p.east))).or_default().push((r as u32, k as u32));
            widest = widest.max(p.width);
        }
    }
    let mut order: Vec<usize> = (0..reaches.len()).collect();
    order.sort_by(|&a, &b| ends_of[a].3.total_cmp(&ends_of[b].3));
    let mut merged = vec![false; reaches.len()];
    for &r in &order {
        let p = &reaches[r].points;
        if p.len() < 3 {
            continue;
        }
        let mut along = vec![0.0f32; p.len()];
        for k in 1..p.len() {
            along[k] = along[k - 1] + ((p[k].north - p[k - 1].north).powi(2) + (p[k].east - p[k - 1].east).powi(2)).sqrt();
        }
        let total = along[p.len() - 1];
        let mut found = None;
        'walk: for k in 1..p.len() - 1 {
            let a = p[k];
            let clear = a.width + widest;
            if along[k] < clear || total - along[k] < clear {
                continue;
            }
            let span = ((a.width + widest) / 2.0 + 1.0) / CELL;
            let span = span.ceil() as i64;
            let (cn, ce) = (cell(a.north), cell(a.east));
            for dn in -span..=span {
                for de in -span..=span {
                    for &(r2, k2) in cells.get(&(cn + dn, ce + de)).map_or(&[][..], |v| v.as_slice()) {
                        let (r2, k2) = (r2 as usize, k2 as usize);
                        if r2 == r || k2 >= reaches[r2].points.len() {
                            continue;
                        }
                        let b = reaches[r2].points[k2];
                        let lower = b.level < a.level - 0.5 || (b.level <= a.level && ends_of[r2].3 < ends_of[r].3);
                        if lower && ((a.north - b.north).powi(2) + (a.east - b.east).powi(2)).sqrt() <= (a.width + b.width) / 2.0 + 1.0 {
                            found = Some((k, b.level));
                            break 'walk;
                        }
                    }
                }
            }
        }
        if let Some((k, level)) = found {
            let points = &mut reaches[r].points;
            points.truncate(k + 1);
            let above = points[k - 1].level;
            points[k].level = points[k].level.min(level);
            points[k].curtain = if above - points[k].level >= fall_from.min(2.0) { above } else { f32::NAN };
            merged[r] = true;
        }
    }
    merged
}

/// What laying the water found: for the panel, the deepest cut below the lowest ground across a
/// river, how many points cut deeper than `DEEP_CUT`, the highest it stands above it (a hollow
/// filled, or a pond) and at how many points, and how many rises were cut through instead
/// (and how many of those ran into water below them); the ponds behind rises; and which reaches met
/// the sea before their planned mouth (cut short there).
struct Laid {
    deepest_cut: f32,
    deep_cuts: usize,
    deepest_fill: f32,
    filled: usize,
    ponds: Vec<Lake>,
    cut_instead: usize,
    into_water: usize,
    cut_short: Vec<bool>,
}

/// The rivers a pond behind a rise could drain into [2026-10-31: a creek's pond stood 31 blocks
/// above the river beside it, and a pond backed up a tributary past its own notch, walls of water]:
/// every river's points (their water as laid before any pond, the lowest it can be), by 32-block
/// cell, each in the cells its channel reaches; with which reaches each flows into, at any remove.
/// (A lake below a pond's water is taken into it, at one level: the highest water shows where lakes
/// overlap, `Lakes::at`.)
struct Water<'a> {
    reaches: &'a [Reach],
    into: Vec<Vec<usize>>,
    cells: std::collections::HashMap<(i64, i64), Vec<(u32, u32)>>,
    grain: f32,
}

impl<'a> Water<'a> {
    const CELL: f32 = 32.0;

    fn new(reaches: &'a [Reach], ends_of: &[(usize, usize, Option<f32>, f32)], grain: f32) -> Self {
        // A reach ending at a junction flows into the one starting there.
        let from: std::collections::HashMap<usize, usize> = ends_of.iter().enumerate().map(|(r, e)| (e.0, r)).collect();
        let next = |r: usize| if ends_of[r].2.is_none() { from.get(&ends_of[r].1).copied().filter(|&n| n != r) } else { None };
        let into = (0..reaches.len())
            .map(|r| {
                let mut below = Vec::new();
                let mut at = r;
                while let Some(n) = next(at).filter(|n| !below.contains(n)) {
                    below.push(n);
                    at = n;
                }
                below
            })
            .collect();
        let mut cells: std::collections::HashMap<(i64, i64), Vec<(u32, u32)>> = std::collections::HashMap::new();
        let cell = |v: f32| (v / Self::CELL).floor() as i64;
        for (r, reach) in reaches.iter().enumerate() {
            for (k, p) in reach.points.iter().enumerate() {
                let out = p.width / 2.0 + grain;
                for a in cell(p.north - out)..=cell(p.north + out) {
                    for b in cell(p.east - out)..=cell(p.east + out) {
                        cells.entry((a, b)).or_default().push((r as u32, k as u32));
                    }
                }
            }
        }
        Self { reaches, into, cells, grain }
    }

    /// Whether a pond at `level`, held by reach `r`'s rise at its point `rise`, would drain away at a
    /// cell (by its middle). In its water (`wet`): into another river's channel whose water is lower
    /// (but one flowing into the pond's river), or onto its own river's line below the rise. On its
    /// rim (dry): through another river's channel cut below it, one flowing in too (the pond would
    /// drain back up it, to wherever its notch leads).
    fn drains(&self, r: usize, rise: usize, level: f32, north: f32, east: f32, wet: bool) -> bool {
        let Some(near) = self.cells.get(&((north / Self::CELL).floor() as i64, (east / Self::CELL).floor() as i64)) else { return false };
        near.iter().any(|&(o, k)| {
            let (o, k) = (o as usize, k as usize);
            let p = &self.reaches[o].points[k];
            let off = ((p.north - north).powi(2) + (p.east - east).powi(2)).sqrt();
            if !wet {
                o != r && p.level < level - 1.5 && off <= p.width / 2.0 + self.grain / 2.0
            } else if o == r {
                k > rise && p.level < level - 0.5 && off <= self.grain
            } else {
                !self.into[o].contains(&r) && p.level < level - 0.5 && off <= p.width / 2.0 + self.grain
            }
        })
    }
}

/// Each waterfall dropped at its lip [David, 2026-10-31: pillars of water standing in the air]: the
/// water above a fall held its level on to the next point, out past the cliff's edge where the
/// ground fell away under it, and its curtain hung out in front of that. Where the ground across the
/// river first falls below the water above the fall, a point is put in at the water below, so the
/// water above ends at the edge and the curtain hangs down the face.
fn falls_at_lips(reaches: &mut [Reach], ground: &CompiledMap) {
    for reach in reaches.iter_mut() {
        let old = std::mem::take(&mut reach.points);
        let mut points = Vec::with_capacity(old.len());
        for (k, &(mut b)) in old.iter().enumerate() {
            let a = old[k.saturating_sub(1)];
            let (dn, de) = (b.north - a.north, b.east - a.east);
            let length = (dn * dn + de * de).sqrt();
            if k > 0 && b.curtain.is_finite() && length >= 1.0 {
                let (nn, ne) = (de / length, -dn / length);
                // The lowest ground across the river, `along` blocks down the piece.
                let low = |along: f32| {
                    let t = along / length;
                    let half = (a.width + (b.width - a.width) * t) / 2.0;
                    let (n, e) = (a.north + dn * t, a.east + de * t);
                    [0.0, half, -half].iter().filter_map(|&o| ground.height(n + nn * o, e + ne * o)).fold(f32::MAX, f32::min)
                };
                if let Some(j) = (1..=length.floor() as usize).find(|&j| low(j as f32) < a.level) {
                    // Halfway between the last spot whose ground holds the water up and the first
                    // that doesn't.
                    let t = (j as f32 - 0.5) / length;
                    let mix = |x: f32, y: f32| x + (y - x) * t;
                    points.push(RiverPoint {
                        north: mix(a.north, b.north),
                        east: mix(a.east, b.east),
                        width: mix(a.width, b.width),
                        depth: mix(a.depth, b.depth),
                        edge: mix(a.edge, b.edge),
                        flat: mix(a.flat, b.flat),
                        bowl: mix(a.bowl, b.bowl),
                        level: b.level,
                        bend: if t < 0.5 { a.bend } else { b.bend },
                        curtain: b.curtain,
                        turn: mix(a.turn, b.turn),
                        fade: mix(a.fade, b.fade),
                    });
                    b.curtain = f32::NAN;
                }
            }
            points.push(b);
        }
        reach.points = points;
    }
}

/// A reach's rises: which are cut through though deeper than the cut (their pond would spread too
/// far, or run into water below it), by point, and the ponds the others hold.
struct Ponds {
    cut: Vec<bool>,
    too_far: usize,
    into_water: usize,
    lakes: Vec<Lake>,
}

/// Which rises across a reach hold its water back in a pond (David's "marble"). From the end
/// upward, the water has to be no lower than each rise less the cut (`cut_up_to`), so the rise that
/// sets that height holds the water above it; where that leaves the water standing above the ground,
/// it's a pond, filled out over the real ground at that height. A pond that spreads further than
/// `pool_reach` from the river (or off the map), or runs into `water` below it (a lake, another
/// river, or its own below the rise), can't be held: that rise is cut through instead, and the rest
/// looked at again. These are reach `r`'s points; `low` is the lowest ground across each (as
/// `lay_water` reads it), and each point's level is the water's with every rise cut through. `None`
/// if stopped.
fn ponds_of(points: &[RiverPoint], low: &[f32], water: &Water, r: usize, ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<Ponds> {
    let count = points.len();
    let (mut cut, mut too_far, mut into_water) = (vec![false; count], 0, 0);
    // Each rise looked at so far: the points it held standing above their ground, and its pond.
    let mut seen: std::collections::HashMap<usize, (Vec<usize>, Option<Lake>)> = std::collections::HashMap::new();
    loop {
        // The rise holding each point, and the points it holds above their ground.
        let mut held: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        let (mut need, mut rise) = (f32::MIN, usize::MAX);
        for k in (0..count.saturating_sub(1)).rev() {
            if low[k] < f32::MAX && !cut[k] && (low[k] - s.cut_up_to.max(0.0)).floor() > need {
                (need, rise) = ((low[k] - s.cut_up_to.max(0.0)).floor(), k);
            }
            if need > points[k].level && need > low[k] + 0.5 {
                held.entry(rise).or_default().push(k);
            }
        }
        let mut changed = false;
        for (&rise, under) in &held {
            if seen.get(&rise).is_some_and(|(before, _)| before == under) {
                continue;
            }
            let level = (low[rise] - s.cut_up_to.max(0.0)).floor();
            let drains = |north: f32, east: f32, wet: bool| water.drains(r, rise, level, north, east, wet);
            let count_it = match pond(points, under, level, &drains, ground, s, stop)? {
                Pond::Held(lake) => {
                    seen.insert(rise, (under.clone(), Some(lake)));
                    continue;
                }
                Pond::Dry => {
                    seen.insert(rise, (under.clone(), None));
                    continue;
                }
                Pond::TooFar => &mut too_far,
                Pond::IntoWater => &mut into_water,
            };
            // Cut through, with the points beside it up and down the rise whose ponds would be much
            // the same (each a little lower, so as big).
            *count_it += 1;
            let close = |k: usize| low[k] < f32::MAX && (low[k] - s.cut_up_to.max(0.0)).floor() >= level - s.cut_up_to.max(1.0);
            cut[rise] = true;
            for k in (0..rise).rev().take_while(|&k| close(k)).chain((rise + 1..count).take_while(|&k| close(k))).collect::<Vec<_>>() {
                cut[k] = true;
            }
            seen.remove(&rise);
            changed = true;
        }
        if !changed {
            let lakes = held.keys().filter_map(|rise| seen.remove(rise).and_then(|(_, lake)| lake)).collect();
            return Some(Ponds { cut, too_far, into_water, lakes });
        }
    }
}

/// A rise's pond: none (the ground isn't below it after all, or it's the sea's), held, spreading too
/// far to hold, or draining into a river below it (`Water::drains`).
enum Pond {
    Dry,
    Held(Lake),
    TooFar,
    IntoWater,
}

/// The pond held at `level` round some of a reach's points: the real ground below it reached from
/// under them, on cells `grain` blocks across; too far if it spreads more than `pool_reach` from
/// them, or off the map; into water if it reaches a cell (by its middle; in its water or on its rim)
/// where it `drains` into a river below it. `None` if stopped.
fn pond(points: &[RiverPoint], under: &[usize], level: f32, drains: &dyn Fn(f32, f32, bool) -> bool, ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<Pond> {
    if level <= SEA_LEVEL as f32 + 0.5 {
        return Some(Pond::Dry);
    }
    let g = s.grain.max(1.0);
    let reach = s.pool_reach.max(0.0);
    let (mut north, mut south, mut west, mut east) = (f32::MIN, f32::MAX, f32::MAX, f32::MIN);
    for &k in under {
        let p = &points[k];
        (north, south, west, east) = (north.max(p.north), south.min(p.north), west.min(p.east), east.max(p.east));
    }
    let (top, bottom, left, right) = (((north + reach) / g).floor() as i64, ((south - reach) / g).floor() as i64, ((west - reach) / g).floor() as i64, ((east + reach) / g).floor() as i64);
    let wet = |a: i64, b: i64| ground.height((a as f32 + 0.5) * g, (b as f32 + 0.5) * g).map(|h| h < level - 0.5);
    let into_water = |a: i64, b: i64, wet: bool| drains((a as f32 + 0.5) * g, (b as f32 + 0.5) * g, wet);
    let mut cells: std::collections::HashMap<(i64, i64), bool> = std::collections::HashMap::new();
    let mut queue = Vec::new();
    for &k in under {
        let at = ((points[k].north / g).floor() as i64, (points[k].east / g).floor() as i64);
        if let std::collections::hash_map::Entry::Vacant(cell) = cells.entry(at) {
            // (Its cell is wet if the river is under water there, though the cell's middle may not be.)
            let w = wet(at.0, at.1) == Some(true) || ground.height(points[k].north, points[k].east).is_some_and(|h| h < level - 0.5);
            cell.insert(w);
            if w {
                if into_water(at.0, at.1, true) {
                    return Some(Pond::IntoWater);
                }
                queue.push(at);
            }
        }
    }
    let mut popped = 0usize;
    while let Some((a, b)) = queue.pop() {
        popped += 1;
        if popped.is_multiple_of(4096) && stop.load(Ordering::Relaxed) {
            return None;
        }
        for (da, db) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let next = (a + da, b + db);
            if cells.contains_key(&next) {
                continue;
            }
            if next.0 > top || next.0 < bottom || next.1 < left || next.1 > right {
                return Some(Pond::TooFar);
            }
            let Some(w) = wet(next.0, next.1) else { return Some(Pond::TooFar) };
            cells.insert(next, w);
            // (In its water, or on its rim.)
            if into_water(next.0, next.1, w) {
                return Some(Pond::IntoWater);
            }
            if w {
                queue.push(next);
            }
        }
    }
    let water: Vec<(i64, i64)> = cells.into_iter().filter(|&(_, w)| w).map(|(at, _)| at).collect();
    if water.is_empty() {
        return Some(Pond::Dry);
    }
    // Its grid, a cell bigger all round (so its edge is beside it).
    let (a1, a0) = water.iter().fold((i64::MIN, i64::MAX), |(hi, lo), &(a, _)| (hi.max(a), lo.min(a)));
    let (b1, b0) = water.iter().fold((i64::MIN, i64::MAX), |(hi, lo), &(_, b)| (hi.max(b), lo.min(b)));
    let (rows, cols) = ((a1 - a0 + 3) as usize, (b1 - b0 + 3) as usize);
    let mut surface = vec![f32::NAN; rows * cols];
    for (a, b) in water {
        surface[(a1 + 1 - a) as usize * cols + (b - b0 + 1) as usize] = level;
    }
    Some(Pond::Held(Lake::new((a1 + 2) as f32 * g, (b0 - 1) as f32 * g, cols, rows, g, surface)))
}

/// A line's points again, `spacing` blocks apart along it (the ends kept).
fn resample(line: &[(f32, f32)], spacing: f32) -> Vec<(f32, f32)> {
    let mut out = vec![line[0]];
    let mut carried = 0.0f32;
    for pair in line.windows(2) {
        let ((an, ae), (bn, be)) = (pair[0], pair[1]);
        let length = ((bn - an).powi(2) + (be - ae).powi(2)).sqrt();
        let mut at = spacing - carried;
        while at < length {
            let t = at / length;
            out.push((an + (bn - an) * t, ae + (be - ae) * t));
            at += spacing;
        }
        carried = (carried + length) % spacing;
    }
    let last = *line.last().unwrap();
    if out.last() != Some(&last) {
        out.push(last);
    }
    out
}

/// A delta at a river's mouth, if the ground there is flat: its last stretch fanned into three
/// channels, each a third as wide, all at the sea's level.
fn delta(path: &[PathPoint], rank: u32, s: &CarvingSettings) -> Option<Vec<Reach>> {
    let end = path.last()?;
    if end.level > SEA_LEVEL as f32 {
        return None;
    }
    let size = s.delta_width[(rank as usize).saturating_sub(3).min(1)];
    // Where the delta starts: the path point about its size back from the mouth.
    let mut back = 0.0;
    let mut from = path.len() - 1;
    while from > 0 && back < size {
        back += ((path[from].north - path[from - 1].north).powi(2) + (path[from].east - path[from - 1].east).powi(2)).sqrt();
        from -= 1;
    }
    let start = &path[from];
    if (start.level - end.level) / back.max(1.0) > s.delta_flat {
        return None;
    }
    let (dn, de) = (end.north - start.north, end.east - start.east);
    let length = (dn * dn + de * de).sqrt().max(1.0);
    let (un, ue) = (dn / length, de / length);
    // Each channel the river's shape, a third as wide and about √3 times shallower, its banks its
    // rank's.
    let (width, depth, shape) = ((end.width / 3.0).max(2.0), (end.depth / 3f32.sqrt()).max(1.0), end.shape);
    let fade = (width * s.fade_for(rank)).max(s.fade_least);
    let level = SEA_LEVEL as f32;
    let spread = size / 2.0;
    Some(
        [-1.0f32, 0.0, 1.0]
            .iter()
            .map(|&side| {
                // Fanning out sideways (perpendicular to the river) to the delta's width at the coast.
                let (tip_n, tip_e) = (start.north + un * length * 1.1 - ue * side * spread, start.east + ue * length * 1.1 + un * side * spread);
                let point = |north: f32, east: f32| RiverPoint { north, east, width, depth, edge: shape.edge, flat: shape.flat, bowl: shape.bowl, level, bend: 0.0, curtain: f32::NAN, turn: 0.0, fade };
                let mid = ((start.north + tip_n) / 2.0 - ue * side * spread * 0.2, (start.east + tip_e) / 2.0 + un * side * spread * 0.2);
                Reach { points: vec![point(start.north, start.east), point(mid.0, mid.1), point(tip_n, tip_e)] }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::network::NetworkSettings;

    /// Valley country (ground rising from the sea in the west, valleys running west), its flow and
    /// network, at 32 blocks a patch.
    fn country() -> (Flow, Network, CompiledMap) {
        let width = 64;
        let z: Vec<Option<f32>> = (0..width * width)
            .map(|i| {
                let (c, r) = ((i % width) as f32, (i / width) as f32);
                Some(100.0 + c * 6.0 + (r * 0.35).sin() * 6.0 * (c / width as f32))
            })
            .collect();
        let flow = crate::worldgen::rivers::flow(width, &z, &vec![1.0; width * width], 2.0, 7, &AtomicBool::new(false)).unwrap();
        let s = NetworkSettings { spring_from: 300.0, spring_to: 1_000.0, spacing: 200.0, mouth_spacing: 100.0, branching: 1.0, ..NetworkSettings::default() };
        let network = crate::worldgen::network::make(&flow, 32.0, 3, &s, &AtomicBool::new(false)).unwrap();
        let heights: Vec<f32> = z.iter().map(|h| h.unwrap_or(0.0)).collect();
        (flow, network, CompiledMap::of_heights(width, 32.0, &heights))
    }

    fn carve_with(s: &CarvingSettings) -> Carving {
        let (flow, network, ground) = country();
        make(&network, &flow, &ground, 32.0, 3, 4.0, 3.0, s, &AtomicBool::new(false)).unwrap()
    }

    fn carve() -> Carving {
        carve_with(&CarvingSettings::default())
    }

    #[test]
    fn curves_start_pinned_on_their_paths_and_their_water_never_rises_downstream() {
        let carving = carve();
        assert!(carving.reaches.len() > 2, "{} reaches", carving.reaches.len());
        for reach in &carving.reaches {
            // The start is a patch's centre, not moved sideways (pinned).
            let first = reach.points[0];
            let off = |v: f32| ((v / 32.0 + 0.5) - (v / 32.0 + 0.5).round()).abs();
            assert!(off(first.north) < 1e-3 && off(first.east) < 1e-3, "start moved: {first:?}");
            for pair in reach.points.windows(2) {
                assert!(pair[1].level <= pair[0].level + 1e-3, "the water rises downstream: {} then {}", pair[0].level, pair[1].level);
            }
        }
    }

    #[test]
    fn the_ranks_bands_meet_with_no_jumps() {
        let s = CarvingSettings::default();
        for rank in 1..4 {
            let (top, bottom) = (size_in_band(&s, rank, 1.0), size_in_band(&s, rank + 1, 0.0));
            assert!((top.0 - bottom.0).abs() < 1e-3 && (top.1 - bottom.1).abs() < 1e-3 && top.2 == bottom.2, "{top:?} then {bottom:?}");
        }
        // David's drawing: creeks 5-9 wide, streams 11-19, rivers 29-51, all inside their bands.
        let (creek, stream, river) = (size_in_band(&s, 1, 0.75).0, size_in_band(&s, 2, 0.5).0, size_in_band(&s, 3, 0.5).0);
        assert!((5.0..9.5).contains(&creek) && (11.0..19.0).contains(&stream) && (29.0..51.0).contains(&river), "{creek} {stream} {river}");
    }

    #[test]
    fn rivers_grow_downstream_within_their_rank_s_band_from_a_spring_s_size() {
        let s = CarvingSettings { size_varies: 0.0, shape_varies: 0.0, ..CarvingSettings::default() };
        let carving = carve_with(&s);
        for reach in &carving.reaches {
            for pair in reach.points.windows(2) {
                assert!(pair[1].width >= pair[0].width - 1e-3, "narrowing downstream: {} then {}", pair[0].width, pair[1].width);
            }
        }
        let widths = || carving.reaches.iter().flat_map(|r| r.points.iter()).map(|p| p.width);
        assert!((widths().fold(f32::MAX, f32::min) - s.spring_width).abs() < 0.01, "no spring-sized start");
        assert!(widths().fold(0.0, f32::max) <= size_in_band(&s, 4, 1.0).0 + 1e-3);
    }

    #[test]
    fn a_lake_fills_its_real_basin_and_never_stands_against_air() {
        // Ground rising east, draining west, with a round crater 20 deep and 6 patches across the
        // middle at column 40: step 5 makes it a lake on its 32-block patches.
        let width = 64;
        let z: Vec<f32> = (0..width * width)
            .map(|i| {
                let (c, r) = ((i % width) as f32, (i / width) as f32);
                let d = ((c - 40.0).powi(2) + (r - 32.0).powi(2)).sqrt();
                160.0 + c * 0.6 - 20.0 * (1.0 - d / 6.0).max(0.0)
            })
            .collect();
        let broad: Vec<Option<f32>> = z.iter().map(|&h| Some(h)).collect();
        let flow = crate::worldgen::rivers::flow(width, &broad, &vec![1.0; width * width], 2.0, 7, &AtomicBool::new(false)).unwrap();
        assert!(flow.lake.iter().any(|&l| l > 0.0), "step 5 found no lake");
        let ground = CompiledMap::of_heights(width, 32.0, &z);
        let (lakes, pools) = fill_lakes(&flow, 32.0, &ground, &CarvingSettings::default(), &AtomicBool::new(false)).unwrap();
        let (north, east) = ((32.0 - 32.0 - 0.5) * 32.0, (40.0 + 0.5 - 32.0) * 32.0);
        assert!(pools >= 1 && lakes.water(north, east).is_some(), "no water in the crater");
        // Every column beside the water is as high as the water, or filled up to just under it.
        let carving = Carving::new(Vec::new(), lakes, crate::world::Banks::default());
        for row in -60..=60 {
            for col in -60..=60 {
                let (n, e) = (north + row as f32 * 4.0, east + col as f32 * 4.0);
                let Some(surface) = carving.lakes.water(n, e) else { continue };
                for (dn, de) in [(4.0, 0.0), (-4.0, 0.0), (0.0, 4.0), (0.0, -4.0)] {
                    let (n, e) = (n + dn, e + de);
                    let real = ground.height(n, e).unwrap();
                    if carving.lakes.water(n, e).is_none() {
                        let here = carving.here(n, e, real);
                        let top = real.max(here.fill.unwrap_or(f32::MIN));
                        assert!(top >= surface.round() - 1.5, "water at {surface} beside ground at {top}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_route_goes_round_a_ridge_through_its_gap_not_over_it() {
        // Ground falling west, 8 blocks a pixel; a ridge 50 high across columns 30 to 33, with a gap in
        // rows 44 to 47 (about 108 blocks south of the planned path, along row 32).
        let width = 64;
        let z: Vec<f32> = (0..width * width)
            .map(|i| {
                let (c, r) = (i % width, i / width);
                let ridge = (30..=33).contains(&c) && !(44..=47).contains(&r);
                150.0 + c as f32 * 0.5 + if ridge { 50.0 } else { 0.0 }
            })
            .collect();
        let ground = CompiledMap::of_heights(width, 8.0, &z);
        let shape = CarvingSettings::default().shapes[0];
        let at = |c: f32| PathPoint { north: (32.0 - 32.0 - 0.5) * 8.0, east: (c + 0.5 - 32.0) * 8.0, width: 3.0, depth: 2.0, shape, level: 0.0 };
        let path: Vec<PathPoint> = (0..=12).map(|k| at(56.0 - k as f32 * 4.0)).collect();
        let highest = |corridor: f32| {
            let s = CarvingSettings { corridor, ..CarvingSettings::default() };
            let line = route(&path, &ground, &s, &AtomicBool::new(false)).unwrap();
            assert_eq!((line[0], line[line.len() - 1]), ((path[0].north, path[0].east), (path[12].north, path[12].east)), "the ends are pinned");
            line.iter().map(|&(n, e)| ground.height(n, e).unwrap()).fold(f32::MIN, f32::max)
        };
        let (round, over) = (highest(160.0), highest(16.0));
        assert!(over > 195.0, "a narrow corridor has to cross the ridge: {over}");
        assert!(round < 180.0, "a wide one goes round through the gap: {round}");
    }

    #[test]
    fn only_where_needed_a_river_looks_for_a_way_round_where_it_would_cut_deep() {
        // Stage 4 (David: rivers meander round the land down low): the same ridge, 50 high across the
        // plan, with its gap about 108 blocks south. Only where needed, the river keeps its plan up to
        // the ridge, finds the gap in a box round the stretch, and keeps its plan past it.
        let width = 64;
        let z: Vec<f32> = (0..width * width)
            .map(|i| {
                let (c, r) = (i % width, i / width);
                let ridge = (30..=33).contains(&c) && !(44..=47).contains(&r);
                150.0 + c as f32 * 0.5 + if ridge { 50.0 } else { 0.0 }
            })
            .collect();
        let ground = CompiledMap::of_heights(width, 8.0, &z);
        let shape = CarvingSettings::default().shapes[0];
        let at = |c: f32| PathPoint { north: -4.0, east: (c + 0.5 - 32.0) * 8.0, width: 3.0, depth: 2.0, shape, level: 0.0 };
        let path: Vec<PathPoint> = (0..=12).map(|k| at(56.0 - k as f32 * 4.0)).collect();
        let s = CarvingSettings { route: Route::WhereNeeded, ..CarvingSettings::default() };
        let (line, found) = route_where_needed(&path, &ground, &s, &|_| 8.0, &AtomicBool::new(false)).unwrap();
        assert_eq!((found.deep, found.rerouted), (1, 1), "{found:?}");
        assert!(found.plan_cut > 40.0 && found.line_cut < 20.0, "{found:?}");
        assert_eq!((line[0], line[line.len() - 1]), ((path[0].north, path[0].east), (path[12].north, path[12].east)), "the ends are pinned");
        // Away from the ridge, the plan: the first stretch of the line runs along it.
        assert!(line.iter().take(10).all(|&(n, _)| (n - -4.0).abs() < 1e-3), "it left its plan before the ridge");
        // With a deep cut allowed (high up, say), it cuts through on its plan.
        let (line, found) = route_where_needed(&path, &ground, &s, &|_| 60.0, &AtomicBool::new(false)).unwrap();
        assert_eq!((found.deep, found.rerouted), (0, 0));
        assert!(line.iter().all(|&(n, _)| (n - -4.0).abs() < 1e-3));
    }

    #[test]
    fn least_change_fills_a_hollow_and_cuts_the_rise_after_it_by_turns() {
        // After a hollow 6 deep, a rise 6 high, then the ground falling on. Cut through, the water
        // stays at the hollow's 100 until the ground falls below it: the rise is cut 6. The least
        // change fills the hollow some and cuts the rise some, only falling, changing less in all.
        let low = [112.0, 110.0, 106.0, 100.0, 103.0, 106.0, 104.0, 101.0, 98.0, 95.0];
        let cut: Vec<f32> = low.iter().scan(f32::MAX, |level, &l| {
            *level = level.min(l);
            Some(*level)
        }).collect();
        let changed = |levels: &[f32]| levels.iter().zip(&low).map(|(l, g)| (l - g).abs()).sum::<f32>();
        let least = least_change(&low, &low.map(|l| l + 20.0), f32::MAX, 40.0);
        assert!(least.windows(2).all(|w| w[1] <= w[0]), "it rises: {least:?}");
        assert!(changed(&least) < changed(&cut), "{least:?} changes more than {cut:?}");
        assert!(least[3] > 100.0 && least[5] < 106.0, "the hollow filled and the rise cut: {least:?}");
        // No more than 1 above the ground anywhere: the hollow filled 1 at most, the rise cut for the rest.
        let shallow = least_change(&low, &low.map(|l| l + 1.0), f32::MAX, 40.0);
        assert!(shallow.iter().zip(&low).all(|(l, g)| l - g <= 1.0 + 1e-3), "{shallow:?}");
        // Never above the water arriving.
        assert!(least_change(&low, &low.map(|l| l + 20.0), 105.0, 40.0).iter().all(|&l| l <= 105.0));
    }

    #[test]
    fn with_least_change_each_step_s_run_changes_its_land_least_and_fills_only_between_banks() {
        // David's preset, steps of 18: each flat run cut its points down to its lowest, by up to 17.
        // A creek down a valley (its sides rising 16 a pixel either side), steps of 8.
        let width = 64;
        let z: Vec<f32> = (0..width * width).map(|i| 150.0 + (i % width) as f32 * 4.8 + ((i / width) as f32 - 32.0).abs() * 16.0).collect();
        let ground = CompiledMap::of_heights(width, 8.0, &z);
        let laid = |level| {
            let mut reaches = vec![Reach { points: (0..=112).map(|k| creek(-4.0, (60.0 - k as f32 * 0.5 - 32.0) * 8.0)).collect() }];
            let s = CarvingSettings { level, fill_up_to: 3.0, rapids_length: 1.0, rise_pools: false, ..CarvingSettings::default() };
            lay_water(&mut reaches, &[(0, 1, None, 100.0)], &ground, 8.0, 25.0, &s, &AtomicBool::new(false)).unwrap();
            reaches.remove(0).points.iter().map(|p| (ground.height(p.north, p.east).unwrap(), p.level)).collect::<Vec<_>>()
        };
        let deepest = |levels: &[(f32, f32)]| levels.iter().map(|(g, l)| g - l).fold(0.0f32, f32::max);
        let (cut, least) = (laid(Level::Cut), laid(Level::LeastChange));
        assert!(deepest(&least) < deepest(&cut) - 1.0, "cut {} deep, least change {}", deepest(&cut), deepest(&least));
        assert!(least.iter().all(|(g, l)| l - g <= 3.5), "filled more than 3: {least:?}");
        // On ground level across (no banks rising either side), never filled at all.
        let flat: Vec<f32> = (0..width * width).map(|i| 150.0 + (i % width) as f32 * 4.8).collect();
        let ground = CompiledMap::of_heights(width, 8.0, &flat);
        let mut reaches = vec![Reach { points: (0..=112).map(|k| creek(-4.0, (60.0 - k as f32 * 0.5 - 32.0) * 8.0)).collect() }];
        let s = CarvingSettings { level: Level::LeastChange, fill_up_to: 3.0, fill_over_banks: 0.0, rapids_length: 1.0, rise_pools: false, ..CarvingSettings::default() };
        lay_water(&mut reaches, &[(0, 1, None, 100.0)], &ground, 8.0, 25.0, &s, &AtomicBool::new(false)).unwrap();
        assert!(reaches[0].points.iter().all(|p| p.level <= ground.height(p.north, p.east).unwrap() + 0.5), "filled on a slope");
    }

    #[test]
    fn a_river_running_alongside_a_lower_one_merges_into_it_with_a_fall() {
        // David: two rivers side by side "should cause a waterfall to merge". A river at 160 runs
        // alongside one at 150, 4.5 apart (their 4-wide waters a block apart): it ends where they
        // first come close (clear of its start), falling to 150.
        let line = |north: f32, level: f32| Reach { points: (0..=100).map(|k| RiverPoint { width: 4.0, level, ..creek(north, k as f32 * 4.0 - 200.0) }).collect() };
        let mut reaches = vec![line(0.5, 160.0), line(5.0, 150.0)];
        let ends = [(0, 1, None, 100.0), (2, 3, None, 50.0)];
        let merged = merge_alongside(&mut reaches, &ends, 3.0);
        assert_eq!(merged, vec![true, false]);
        let last = *reaches[0].points.last().unwrap();
        assert!(reaches[0].points.len() < 10, "it ran on alongside: {} points", reaches[0].points.len());
        assert_eq!((last.level, last.curtain), (150.0, 160.0), "a fall to the lower river");
        assert_eq!(reaches[1].points.len(), 101, "the lower river carries on");
        // Further apart (their waters 4 blocks apart), each keeps its own way.
        let mut apart = vec![line(0.5, 160.0), line(8.5, 150.0)];
        assert_eq!(merge_alongside(&mut apart, &ends, 3.0), vec![false, false]);
    }

    #[test]
    fn a_tributary_eases_down_into_the_river_it_joins() {
        // David's joint fade: a creek off a plateau (160) joins a river in a trench (150). Without
        // the fade, it drops 10 at the trench's edge; with it, it eases down over its last 18 blocks
        // (3 of the river's 6-block widths) and drops what's left at the edge.
        let width = 256;
        let z: Vec<f32> = (0..width * width).map(|i| if ((128.0 - (i / width) as f32 - 0.5) - 0.5).abs() <= 4.0 { 150.0 } else { 160.0 }).collect();
        let ground = CompiledMap::of_heights(width, 1.0, &z);
        let drops = |join_fade: f32| {
            let tributary = Reach { points: (0..=25).map(|k| creek(100.5 - k as f32 * 4.0, 0.5)).collect() };
            let river = Reach { points: (0..=40).map(|k| RiverPoint { width: 6.0, ..creek(0.5, 0.5 + k as f32 * 4.0) }).collect() };
            let mut reaches = vec![tributary, river];
            let s = CarvingSettings { join_fade, rapids_length: 1.0, rise_pools: false, ..CarvingSettings::default() };
            lay_water(&mut reaches, &[(0, 5, None, 200.0), (5, 9, None, 100.0)], &ground, 1.0, 3.0, &s, &AtomicBool::new(false)).unwrap();
            let levels: Vec<f32> = reaches[0].points.iter().map(|p| p.level).collect();
            (levels.windows(2).map(|w| w[0] - w[1]).fold(0.0f32, f32::max), *levels.last().unwrap(), reaches[1].points[0].level)
        };
        let (hard, end, river) = drops(0.0);
        assert!(hard >= 9.0 && end == river, "{hard} {end} {river}");
        let (eased, end, river) = drops(3.0);
        assert!(eased <= 5.0 && end == river, "{eased} {end} {river}");
    }

    /// A creek's point, 3 wide.
    fn creek(north: f32, east: f32) -> RiverPoint {
        RiverPoint { north, east, width: 3.0, depth: 2.0, edge: 0.3, flat: 0.2, bowl: 1.0, level: f32::NAN, bend: 0.0, curtain: f32::NAN, turn: 0.0, fade: 2.0 }
    }

    /// A creek running west along row 32 over ground rising east, 8 blocks a pixel, across a ridge 20
    /// high (columns 30 to 33), in a valley walled north and south; its water laid with `s`, among
    /// `others` (reaches, none flowing into it).
    fn over_a_ridge(s: &CarvingSettings, others: Vec<Reach>) -> (Vec<RiverPoint>, Laid, CompiledMap) {
        let width = 64;
        let z: Vec<f32> = (0..width * width)
            .map(|i| {
                let (c, r) = (i % width, i / width);
                let ridge = if (30..=33).contains(&c) { 20.0 } else { 0.0 };
                let walls = if !(24..=40).contains(&r) { 60.0 } else { 0.0 };
                150.0 + c as f32 * 0.5 + ridge + walls
            })
            .collect();
        let ground = CompiledMap::of_heights(width, 8.0, &z);
        let mut reaches = vec![Reach { points: (0..=112).map(|k| creek(-4.0, (60.0 - k as f32 * 0.5 + 0.5 - 32.0) * 8.0)).collect() }];
        reaches.extend(others);
        let ends_of: Vec<_> = (0..reaches.len()).map(|r| (2 * r, 2 * r + 1, None, 100.0)).collect();
        let laid = lay_water(&mut reaches, &ends_of, &ground, 1.0, 3.0, s, &AtomicBool::new(false)).unwrap();
        (reaches.remove(0).points, laid, ground)
    }

    #[test]
    fn a_rise_too_deep_to_cut_holds_a_pond_behind_it_and_is_cut_no_deeper_than_the_cut() {
        // David: "these streams either need to ride over the terrain like a marble, or cut far deeper".
        let s = CarvingSettings::default();
        let (points, laid, ground) = over_a_ridge(&s, Vec::new());
        assert_eq!((laid.ponds.len(), laid.cut_instead), (1, 0), "one pond, none cut through");
        let lakes = Lakes::new(laid.ponds);
        let cut = |p: &RiverPoint| ground.height(p.north, p.east).unwrap() - p.level;
        // Over the ridge, cut through no deeper than the cut (and a block for its rounding), but at the
        // lip of a waterfall down its far side (its curtain on), where the water falls.
        let ridge: Vec<&RiverPoint> = points.iter().filter(|p| (-16.0..16.0).contains(&p.east) && p.curtain.is_nan()).collect();
        assert!(ridge.iter().all(|p| cut(p) <= s.cut_up_to + 1.0), "cut {} through the ridge", ridge.iter().map(|p| cut(p)).fold(0.0, f32::max));
        // Behind it, the water held at one level, a pond on the ground all round.
        let behind: Vec<&RiverPoint> = points.iter().filter(|p| (40.0..120.0).contains(&p.east)).collect();
        let level = behind[0].level;
        assert!(behind.iter().all(|p| p.level == level), "the pond's water isn't level");
        assert!(behind.iter().all(|p| cut(p) < 0.0), "the pond's water is below its ground");
        for (n, e) in [(-4.0, 60.0), (20.0, 80.0), (-30.0, 100.0)] {
            assert_eq!(lakes.water(n, e), Some(level), "no pond at {n}, {e}");
        }
        // Off, the ridge is notched its full height.
        let (points, laid, _) = over_a_ridge(&CarvingSettings { rise_pools: false, ..s }, Vec::new());
        assert!(laid.ponds.is_empty());
        assert!(points.iter().map(cut).fold(0.0, f32::max) > 15.0, "no notch with the ponds off");
    }

    #[test]
    fn a_pond_that_would_spread_too_far_is_cut_through_instead() {
        let s = CarvingSettings { pool_reach: 16.0, ..CarvingSettings::default() };
        let (points, laid, ground) = over_a_ridge(&s, Vec::new());
        assert!(laid.ponds.is_empty() && laid.cut_instead >= 1, "{} ponds, {} cut through", laid.ponds.len(), laid.cut_instead);
        assert!(points.iter().map(|p| ground.height(p.north, p.east).unwrap() - p.level).fold(0.0, f32::max) > 15.0, "not cut through");
        // And the water never rises downstream.
        assert!(points.windows(2).all(|w| w[1].level <= w[0].level));
    }

    /// The highest water of some ponds.
    fn highest(ponds: &[Lake]) -> f32 {
        ponds.iter().flat_map(|p| p.surface.iter()).filter(|s| s.is_finite()).fold(f32::MIN, |a, &b| a.max(b))
    }

    #[test]
    fn a_pond_takes_in_a_lake_below_it_at_one_level() {
        // David's screenshot: a pond spread over a lake stood above the lake's water, a raised slab in
        // it (the lake's water showed in some cells, the pond's in others). A lake in the valley
        // behind the ridge, its water (at 175) below the pond's: the pond's water shows over it.
        let (_, laid, _) = over_a_ridge(&CarvingSettings::default(), Vec::new());
        let level = highest(&laid.ponds);
        assert!(level > 176.0, "the pond was at {level}");
        let lake = Lake::new(28.0, 80.0, 8, 8, 4.0, vec![175.0; 64]);
        let both = Lakes::new(std::iter::once(lake).chain(laid.ponds).collect());
        assert_eq!(both.water(12.0, 96.0), Some(level), "the lake's water shows through the pond");
    }

    #[test]
    fn a_pond_that_would_stand_over_another_river_below_it_is_cut_through_instead() {
        // David's world: a creek's pond stood 31 blocks above the river beside it, walls of water.
        // Another creek in the valley behind the ridge, along its north side (its ground 170 to 175),
        // not flowing into this one.
        let other = Reach { points: (0..=16).map(|k| creek(40.0, 140.0 - k as f32 * 5.0)).collect() };
        let (_, laid, _) = over_a_ridge(&CarvingSettings::default(), vec![other]);
        assert!(laid.into_water >= 1, "{} cut through, none into the other creek", laid.cut_instead);
        assert!(highest(&laid.ponds) <= 175.0, "a pond at {} over the other creek", highest(&laid.ponds));
    }

    #[test]
    fn a_river_into_the_sea_ends_where_it_first_meets_it() {
        // David's world: where the broad ground had land but the real coast came first, a river ran on
        // along a cliff top at the sea's level, a slot 57 blocks deep. Ground falling west to an inlet
        // of the sea or a beach just above it (columns 20 to 23), a cliff top at 180 beyond, then the
        // planned mouth.
        let sea = SEA_LEVEL as f32;
        for (dip, what) in [(110.0, "an inlet"), (sea + 0.5, "a beach")] {
            let width = 64;
            let z: Vec<f32> = (0..width * width)
                .map(|i| match i % width {
                    20..=23 => dip,
                    c if c < 20 => 180.0,
                    c => 140.0 + c as f32 * 0.5,
                })
                .collect();
            let ground = CompiledMap::of_heights(width, 8.0, &z);
            let mut reaches = vec![Reach { points: (0..=100).map(|k| creek(-4.0, (60.0 - k as f32 * 0.5 + 0.5 - 32.0) * 8.0)).collect() }];
            let laid = lay_water(&mut reaches, &[(0, 1, Some(sea), 100.0)], &ground, 1.0, 3.0, &CarvingSettings::default(), &AtomicBool::new(false)).unwrap();
            let points = &reaches[0].points;
            let last = points.last().unwrap();
            assert!(laid.cut_short[0] && last.level == sea, "{what}: not cut short, it ends at {last:?}");
            // Nothing of it along the cliff top beyond.
            assert!(points.iter().all(|p| p.east > (19.5 - 32.0) * 8.0 - 4.0), "{what}: it runs on along the cliff top");
        }
    }

    #[test]
    fn a_waterfall_drops_at_its_cliff_s_edge() {
        // David's screenshot: the water above a fall held its level on past the cliff's edge, a pillar
        // of water standing in the air. Down the ridge's steep far side, the water above each fall
        // ends where the ground across the river (3 wide) falls below it.
        let (points, _, ground) = over_a_ridge(&CarvingSettings::default(), Vec::new());
        let mut falls = 0;
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.level - b.level < 3.0 {
                continue;
            }
            falls += 1;
            let length = ((b.north - a.north).powi(2) + (b.east - a.east).powi(2)).sqrt();
            for j in 1..length.ceil() as usize {
                let (n, e) = (a.north + (b.north - a.north) * j as f32 / length, a.east + (b.east - a.east) * j as f32 / length);
                let low = [0.0, 1.5, -1.5].iter().filter_map(|&o| ground.height(n + o, e)).fold(f32::MAX, f32::min);
                assert!(low >= a.level, "water at {} over ground at {low}, {j} blocks on from {}, {}", a.level, a.north, a.east);
            }
        }
        assert!(falls > 0, "no waterfalls");
    }

    #[test]
    fn rapids_never_reach_past_the_waterfall_above_them() {
        // David: "rapids at 1" got rid of walls of water on steep ground. A rapids spread upstream
        // past the waterfall just above it pulled that fall's water down after its curtain was set:
        // the curtain and its cliff stood up out of the water, and the real drop moved up with none.
        // Ground falling 0.6 a block, points 4 apart: drops of 2 (rapids) and 3 (falls) in turn.
        let width = 64;
        let z: Vec<f32> = (0..width * width).map(|i| 150.0 + (i % width) as f32 * 4.8).collect();
        let ground = CompiledMap::of_heights(width, 8.0, &z);
        let mut reaches = vec![Reach { points: (0..=112).map(|k| creek(-4.0, (60.0 - k as f32 * 0.5 - 32.0) * 8.0)).collect() }];
        let s = CarvingSettings { rapids_length: 4.0, rise_pools: false, ..CarvingSettings::default() };
        lay_water(&mut reaches, &[(0, 1, None, 100.0)], &ground, 1.0, 3.0, &s, &AtomicBool::new(false)).unwrap();
        no_sheer_drops(&reaches[0].points);
        // And David's settings of 2026-10-31 (no steps, falls from 25): rapids at every point, with no
        // run to spread over, were left as sheer drops with no curtain, walls of water up to 24 tall.
        let mut reaches = vec![Reach { points: (0..=112).map(|k| creek(-4.0, (60.0 - k as f32 * 0.5 - 32.0) * 8.0)).collect() }];
        lay_water(&mut reaches, &[(0, 1, None, 100.0)], &ground, 0.0, 25.0, &CarvingSettings { rapids_length: 10.0, ..s }, &AtomicBool::new(false)).unwrap();
        no_sheer_drops(&reaches[0].points);
    }

    /// Every drop of 2 or more has a curtain, level with the water above it.
    fn no_sheer_drops(points: &[RiverPoint]) {
        let mut falls = 0;
        for k in 1..points.len() {
            let (above, here) = (points[k - 1], points[k]);
            let drop = above.level - here.level;
            if here.curtain.is_finite() {
                assert_eq!(here.curtain, above.level, "a curtain at {} over water at {}", here.curtain, above.level);
            }
            if drop >= 2.0 {
                assert!(here.curtain.is_finite(), "a drop of {drop} with no curtain at {}, {}", here.north, here.east);
                falls += 1;
            }
        }
        assert!(falls > 0, "no falls");
    }

    #[test]
    fn the_shores_count_flush_where_you_can_walk_out_and_ledges_where_you_cant() {
        // David: flush is level with the water; "1 block ledge you can't get out of the water from".
        let ground = CompiledMap::of_heights(16, 32.0, &[150.0; 256]);
        let shores = |way| {
            let points = (0..=96).map(|k| RiverPoint { level: 150.0, fade: 6.0, ..creek(0.0, k as f32 * 2.0 - 96.0) }).collect();
            let banks = Banks { way, ..Banks::default() };
            measure_shores(&Carving::new(vec![Reach { points }], Lakes::default(), banks), &ground, 1, &AtomicBool::new(false))
        };
        let smooth = shores(crate::world::FadeWay::Smooth);
        assert!(smooth.flush > 10 && smooth.shores() == smooth.flush, "{smooth:?}");
        assert!(smooth.most_lowered > 0.5 && smooth.most_raised == 0.0, "{smooth:?}");
        let ledges = shores(crate::world::FadeWay::LedgeThenSlope);
        assert!(ledges.ledge > 10 && ledges.shores() == ledges.ledge, "{ledges:?}");
    }

    #[test]
    fn the_rivers_come_out_the_same_on_any_number_of_cores() {
        // (As saved: a point with no curtain, NaN, never equals itself.)
        let on = |cores| carve_with(&CarvingSettings { cores, ..CarvingSettings::default() }).to_text();
        assert!(on(1) == on(4), "different on 1 core and 4");
    }

    #[test]
    fn the_water_never_stands_above_the_real_ground_under_it() {
        // David's screenshot: water laid on the broad ground stood above the real ground as walls.
        // (But in a pond behind a rise, whose water is all round it; and held up by one, the water can
        // stand up to half a block over its bed short of a pond, its banks raised to hold it.)
        let (flow, network, ground) = country();
        let carving = make(&network, &flow, &ground, 32.0, 3, 4.0, 3.0, &CarvingSettings::default(), &AtomicBool::new(false)).unwrap();
        for p in carving.reaches.iter().flat_map(|reach| reach.points.iter()) {
            let real = ground.height(p.north, p.east).unwrap();
            let pond = carving.lakes.water(p.north, p.east).is_some_and(|w| w >= p.level);
            assert!(pond || p.level <= real.max(SEA_LEVEL as f32) + 0.5 + 1e-3, "water at {} over ground at {real}", p.level);
        }
    }

    #[test]
    fn the_water_steps_up_to_its_ground_not_to_multiples_of_the_step() {
        // Snapped to Z 128, 132, 136…, the water sat up to 3 blocks more below its banks. Every drop
        // a waterfall (no rapids), so the levels compare point for point (the points both have: each
        // waterfall puts one in at its lip).
        let (flow, network, ground) = country();
        let s = CarvingSettings { join_fade: 0.0, merge_alongside: false, ..CarvingSettings::default() };
        let lay = |step| make(&network, &flow, &ground, 32.0, 3, step, 1.0, &s, &AtomicBool::new(false)).unwrap();
        let (unstepped, stepped) = (lay(1.0), lay(4.0));
        let mut tops = 0;
        for (a, b) in unstepped.reaches.iter().zip(&stepped.reaches) {
            for k in 0..b.points.len() {
                let Some(own) = a.points.iter().find(|q| (q.north, q.east) == (b.points[k].north, b.points[k].east)).map(|q| q.level) else { continue };
                let level = b.points[k].level;
                assert!(level <= own && own - level < 4.0, "water at {level}, its ground's level {own}");
                // The top of each step sits at its own level.
                if k + 1 < b.points.len() && level > b.points[k + 1].level {
                    assert_eq!(level, own, "a step's top below its ground's level");
                    tops += 1;
                }
            }
        }
        assert!(tops > 0, "no steps");
    }

    #[test]
    fn a_reach_ends_at_the_level_of_the_water_it_joins() {
        let carving = carve();
        let mut junctions = 0;
        for reach in &carving.reaches {
            let end = reach.points.last().unwrap();
            for other in &carving.reaches {
                let start = other.points[0];
                if (start.north - end.north).abs() < 1e-3 && (start.east - end.east).abs() < 1e-3 {
                    assert_eq!(end.level, start.level, "a reach ends at {} on water at {}", end.level, start.level);
                    junctions += 1;
                }
            }
        }
        assert!(junctions > 0, "no junctions");
    }

    #[test]
    fn every_river_point_carves_water_into_the_ground() {
        let carving = carve();
        for reach in carving.reaches.iter().take(5) {
            for p in reach.points.iter().step_by(7) {
                let here = carving.here(p.north, p.east, p.level + 5.0);
                assert!(here.water.is_some() && here.floor.is_some(), "no river at {p:?}");
                // Below its own water (exactly on a step, the carve is the lip above it).
                assert!(here.floor.unwrap() < here.water.unwrap() as f32, "the floor is above the water at {p:?}");
            }
        }
    }

    #[test]
    fn sizes_by_area_spread_on_flat_ground_and_stack_on_steep() {
        let s = CarvingSettings::default();
        let across = |(width, depth, shape): (f32, f32, Shape)| width.max(1.0) * (depth * crate::world::fullness(shape.edge, shape.flat, shape.bowl)).max(1.0);
        for water in [16.0, 64.0, 256.0] {
            let (flat, steep) = (size_by_area(&s, water, 0.0), size_by_area(&s, water, 1.0));
            assert!(flat.0 > steep.0 && flat.1 < steep.1, "{water} blocks: flat {flat:?}, steep {steep:?}");
            for size in [flat, steep, size_by_area(&s, water, 0.5)] {
                assert!((across(size) / water - 1.0).abs() < 0.01, "{water} blocks laid out as {size:?}: {} across", across(size));
            }
        }
        // A block of water is about a block wide and deep, flat or steep; two on steep ground are about
        // a block wide and two deep (David's "2 deep spring with steep falls and drops").
        let flat_one = size_by_area(&s, 1.0, 0.0);
        assert_eq!((flat_one.0, flat_one.1), (1.0, 1.0));
        let steep_one = size_by_area(&s, 1.0, 1.0);
        assert!(steep_one.0 == 1.0 && steep_one.1 < 1.5, "{steep_one:?}");
        let steep_two = size_by_area(&s, 2.0, 1.0);
        assert!(steep_two.0 < 1.5 && steep_two.1 >= 2.0, "{steep_two:?}");
    }

    #[test]
    fn by_area_water_gathers_downstream_and_joins_keep_their_share() {
        let (flow, network, _) = country();
        let n = network.rank.len();
        let on = |i: usize| network.rank[i] > 0;
        let down = |i: usize| flow.down[i] as usize;
        let s = CarvingSettings { grows_every: 0.0, join_keeps: 0.9, ..CarvingSettings::default() };
        let (water, steep) = water_by_area(&network, &flow, 32.0, &s);
        let mut joins = 0;
        for i in (0..n).filter(|&i| on(i)) {
            assert!((0.0..=1.0).contains(&steep[i]));
            let ups: Vec<usize> = (0..n).filter(|&u| u != i && down(u) == i && water[u] > 0.0).collect();
            let gathered: f32 = ups.iter().map(|&u| water[u]).sum();
            let expected = match ups.len() {
                0 => 1.0,
                1 => gathered,
                _ => 0.9 * gathered,
            };
            assert!((water[i] - expected).abs() < 1e-3, "at {i}: {} from {ups:?}, expected {expected}", water[i]);
            joins += (ups.len() >= 2) as usize;
        }
        assert!(joins > 0, "no joins in the test country");
    }

    #[test]
    fn a_lake_passes_the_water_of_the_rivers_into_it_on_to_its_outlet() {
        // A 4 by 4 grid of 32-block patches: springs at 1 and 2 run south into a lake (5 and 6, 5
        // draining east into 6); its outlet river runs from 10 to the sea at 14.
        let w = 4;
        let mut down: Vec<u32> = (0..16).collect();
        for (from, to) in [(1, 5), (2, 6), (5, 6), (6, 10), (10, 14)] {
            down[from] = to;
        }
        let mut lake = vec![0.0f32; 16];
        (lake[5], lake[6]) = (3.0, 3.0);
        let surface: Vec<f32> = (0..16).map(|i| 200.0 - (i / w) as f32 * 10.0).collect();
        let flow = Flow { width: w, down, surface, flow: vec![1.0; 16], lake, most: 1.0, lake_share: 0.1 };
        let mut rank = vec![0u8; 16];
        for i in [1, 2, 10] {
            rank[i] = 1;
        }
        let network = Network { width: w, order: rank.clone(), rank, bed: vec![f32::NAN; 16], drop: vec![0.0; 16], springs: Vec::new(), mouths: Vec::new() };
        let (water, _) = water_by_area(&network, &flow, 32.0, &CarvingSettings { grows_every: 0.0, ..CarvingSettings::default() });
        assert!((water[10] - 1.8).abs() < 1e-3, "the outlet's water: {} (two springs joining in the lake, keeping 90%)", water[10]);
        // Growing a block every patch run (32 blocks): each spring is 2 on reaching the lake.
        let (water, _) = water_by_area(&network, &flow, 32.0, &CarvingSettings { grows_every: 32.0, ..CarvingSettings::default() });
        assert!((water[10] - 3.6).abs() < 1e-3, "the outlet's water: {}", water[10]);
    }

    #[test]
    fn rivers_sized_by_area_are_never_under_a_block_and_the_readout_counts_them() {
        let s = CarvingSettings { by_area: true, size_varies: 0.0, shape_varies: 0.0, ..CarvingSettings::default() };
        let carving = carve_with(&s);
        assert!(carving.reaches.len() > 2);
        let points = || carving.reaches.iter().flat_map(|r| r.points.iter());
        assert!(points().all(|p| p.width >= 1.0 && p.depth >= 1.0));
        let length: f32 = carving.reaches.iter().flat_map(|r| r.points.windows(2)).map(|p| ((p[1].north - p[0].north).powi(2) + (p[1].east - p[0].east).powi(2)).sqrt()).sum();
        assert!((carving.sizes.lengths.iter().sum::<f32>() - length).abs() < 1.0);
        assert!(carving.sizes.most > s.spring_area, "nothing grew: {:?}", carving.sizes);
        // Unticked, the ranks size them as before.
        assert_ne!(carve().reaches, carving.reaches);
    }
}

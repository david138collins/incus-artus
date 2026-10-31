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
//!    river's width or so apart, so bigger rivers bend wider; a gentle **wander**.
//! 3. **Lean outward:** each point moves toward the outside of its bend (fast water cuts the outer bank).
//! 4. **Lengthen (David's squiggles):** a side-to-side wave every 10 to 14 widths, its size by width,
//!    strong on flat ground and fading on steep.
//! 5. **The carve** (in the world): each block measures its distance to the line; the odd block of an
//!    even width goes to the outside of the bend.
//!
//! 6. **Ponds behind rises** [David, 2026-10-30: "ride over the terrain like a marble, or cut far
//!    deeper"]: a rise the route couldn't get round is cut through up to a few blocks; a higher one
//!    holds the water back in a pond up to its lowest gap (unless the pond would spread too far).
//!
//! Safety: the water level is laid from the real ground under the line, only falling downstream; the
//! ends (junctions, lake outlets, mouths) are pinned, the sideways moves fading to nothing there, so
//! rivers always meet; and a sideways move that would climb above the valley floor is pulled back.

use super::network::Network;
use super::noise::{Fade, perlin};
use super::rivers::Flow;
use crate::world::{Banks, Carving, CompiledMap, Lake, Lakes, Reach, RiverPoint, SEA_LEVEL};
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
    /// The wander: how far sideways (blocks) and how long its swings are (blocks).
    pub wander: f32,
    pub wander_length: f32,
    /// How far rivers lean to the outside of their bends (0 none; 1 a width at a sharp bend).
    pub lean: f32,
    /// Meanders: how long one swing is (in river widths), how far it swings (in widths), and how flat
    /// the ground must be for full meanders (blocks down per block along: steeper, less).
    pub meander_length: f32,
    pub meander_size: f32,
    pub flat: f32,
    /// The most a river's line moves sideways from its path (blocks).
    pub most_sideways: f32,
    /// The route on the real ground [2026-10-30]: how far either side of its planned path a river may
    /// find its way (blocks), and the grain it's found on (blocks a cell).
    pub corridor: f32,
    pub grain: f32,
    /// How far above the valley floor (the real ground under its route) the wander, lean and meanders
    /// may move it before they're pulled back (blocks) [2026-10-30: "deepest gorge", over the broad
    /// ground].
    pub floor_rise: f32,
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
    /// Waterfalls: a curtain of water blocks down the face. Rapids: blocks of river for each block
    /// of drop.
    pub curtains: bool,
    pub rapids_length: f32,
    /// Deltas: at a river's mouth, over ground this flat or flatter (blocks down per block), its last
    /// stretch splits into channels fanned this wide (blocks) for a river and a great river.
    pub deltas: bool,
    pub delta_flat: f32,
    pub delta_width: [f32; 2],
}

impl Default for CarvingSettings {
    fn default() -> Self {
        Self {
            smoothing: 2,
            wander: 12.0,
            wander_length: 400.0,
            lean: 0.5,
            meander_length: 12.0,
            meander_size: 1.0,
            flat: 0.02,
            most_sideways: 48.0,
            corridor: 64.0,
            grain: 4.0,
            floor_rise: 3.0,
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
            curtains: true,
            rapids_length: 3.0,
            deltas: true,
            delta_flat: 0.01,
            delta_width: [64.0, 128.0],
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

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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
    // (growing downstream) [2026-10-30, replacing "keeps full size for"]; size and shape wandering
    // slowly along the way.
    let size_at = |i: usize, north: f32, east: f32| {
        let run = up_in_rank[i] + down_in_rank[i];
        let share = if run > 0.0 { up_in_rank[i] / run } else { 0.5 };
        let (width, depth, shape) = size_in_band(s, rank(i) as usize, share);
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
        let rank = network.rank[start].max(1) as u32;
        ends_of.push((start, end, end_level, to_mouth[start]));
        // A delta, where a river or great river meets the sea over flat ground.
        if s.deltas
            && rank >= 3
            && let Some(delta) = delta(&path, rank, s)
        {
            deltas.extend(delta);
        }
        planned.push((path, k as u64, rank));
    }
    // Each reach routed on the real ground and made a curve, on all cores.
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let each = planned.len().div_ceil(cores).max(1);
    let made: Vec<Option<Reach>> = std::thread::scope(|scope| {
        let jobs: Vec<_> = planned
            .chunks(each)
            .map(|part| scope.spawn(move || part.iter().map(|(path, id, rank)| route(path, ground, s, stop).map(|line| curve(&line, path, *id, *rank, ground, seed, s))).collect::<Vec<_>>()))
            .collect();
        // (A part that failed counts as stopped: its reaches can't go missing out of order.)
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_else(|_| vec![None])).collect()
    });
    let mut reaches: Vec<Reach> = made.into_iter().collect::<Option<_>>()?;
    // The water levels, from the real ground (never above the banks, but in a pond behind a rise),
    // only falling downstream.
    let (deepest_cut, deep_cuts, ponds, cut_instead) = lay_water(&mut reaches, &ends_of, ground, step, fall_from, s, stop)?;
    reaches.extend(deltas);
    let rise_ponds = ponds.len();
    let lakes = if ponds.is_empty() { lakes } else { Lakes::new(lakes.lakes.into_iter().chain(ponds).collect()) };
    let mut carving = Carving::new(reaches, lakes, s.banks);
    (carving.took, carving.deepest_cut, carving.deep_cuts, carving.pools) = (started.elapsed().as_secs_f32(), deepest_cut, deep_cuts, pools);
    (carving.rise_ponds, carving.cut_instead) = (rise_ponds, cut_instead);
    Some(carving)
}

/// Where a river's water passes below the lowest ground across it by more than this (blocks), it's
/// counted as a deep cut (for the panel).
const DEEP_CUT: f32 = 6.0;

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
    // Each region filled, on all cores.
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
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

/// One reach's curve along its route: smoothed (corner cut over points about a width apart), then
/// each point moved sideways by the wander, the lean and the meander (fading to nothing at the
/// pinned ends, and pulled back where they'd climb above the valley floor), with the width, depth
/// and shape by how far along its planned path. (Its water level is laid after, `lay_water`.)
fn curve(route: &[(f32, f32)], path: &[PathPoint], id: u64, rank: u32, ground: &CompiledMap, seed: u64, s: &CarvingSettings) -> Reach {
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
    // The real ground, and its slope along the river, for how freely it meanders.
    let ground = |north: f32, east: f32| ground.height(north, east).unwrap_or(SEA_LEVEL as f32);
    let (first, last) = (line[0], line[line.len() - 1]);
    let slope = ((ground(first.0, first.1) - ground(last.0, last.1)) / length).max(0.0);
    let flatness = 1.0 - smooth(slope / s.flat.max(1e-4));
    let phase = perlin(seed, 0xa1e0 + id, Fade::Smooth, id as f32 * 0.37, 0.5) * 6.28;
    let mut points = Vec::with_capacity(line.len());
    let mut run = 0.0f32;
    for k in 0..line.len() {
        if k > 0 {
            run += ((line[k].0 - line[k - 1].0).powi(2) + (line[k].1 - line[k - 1].1).powi(2)).sqrt();
        }
        let (width, depth, shape) = at(run / length);
        // Tangent, normal (to the left, looking downstream) and how sharply it bends here.
        let (a, b) = (line[k.saturating_sub(1)], line[(k + 1).min(line.len() - 1)]);
        let (tn, te) = (b.0 - a.0, b.1 - a.1);
        let t_len = (tn * tn + te * te).sqrt().max(1e-3);
        let (nn, ne) = (te / t_len, -tn / t_len);
        let bend = if k > 0 && k + 1 < line.len() {
            let (p, q, r) = (line[k - 1], line[k], line[k + 1]);
            let cross = (q.0 - p.0) * (r.1 - q.1) - (q.1 - p.1) * (r.0 - q.0);
            // Positive: turning right (looking downstream), its outside to the left.
            -cross / (spacing * spacing)
        } else {
            0.0
        };
        // The pins: no sideways moves at the ends.
        let pin = (2.0 * width).max(48.0);
        let fade = smooth(run / pin) * smooth((length - run) / pin);
        let wander = perlin(seed, 0x3a4d + id, Fade::Smooth, run / s.wander_length.max(1.0), id as f32 * 1.7) * s.wander;
        let lean = -s.lean * width * (bend * 32.0).clamp(-1.0, 1.0);
        let meander = (std::f32::consts::TAU * run / (s.meander_length.max(1.0) * width.max(1.0)) + phase).sin() * s.meander_size * width * flatness;
        let mut sideways = ((wander + lean + meander) * fade).clamp(-s.most_sideways, s.most_sideways);
        // Pulled back where it would climb above the valley floor (the ground under its route).
        if sideways != 0.0 {
            let floor = ground(line[k].0, line[k].1);
            for halving in 0..5 {
                if ground(line[k].0 + nn * sideways, line[k].1 + ne * sideways) - floor <= s.floor_rise {
                    break;
                }
                sideways = if halving < 4 { sideways * 0.5 } else { 0.0 };
            }
        }
        points.push(RiverPoint {
            north: line[k].0 + nn * sideways,
            east: line[k].1 + ne * sideways,
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
            fade: (width * s.fade_widths[(rank as usize).clamp(1, 4) - 1]).max(s.fade_least),
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
/// block at a time. Never below the sea's surface, and a reach into the sea or a lake ends at its
/// level. Gives the deepest cut below the lowest ground across a river, and how many points cut
/// deeper than `DEEP_CUT`.
fn lay_water(reaches: &mut [Reach], ends_of: &[(usize, usize, Option<f32>, f32)], ground: &CompiledMap, step: f32, fall_from: f32, s: &CarvingSettings, stop: &AtomicBool) -> Option<(f32, usize, Vec<Lake>, usize)> {
    let sea = SEA_LEVEL as f32;
    // The lowest real ground across each point's width (its middle and both banks), on all cores.
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let each = reaches.len().div_ceil(cores).max(1);
    let lows: Vec<Vec<f32>> = std::thread::scope(|scope| {
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
                                        return sea;
                                    }
                                    let (a, b) = (p[k.saturating_sub(1)], p[(k + 1).min(p.len() - 1)]);
                                    let (tn, te) = (b.north - a.north, b.east - a.east);
                                    let length = (tn * tn + te * te).sqrt().max(1e-3);
                                    let (nn, ne) = (te / length, -tn / length);
                                    let half = p[k].width / 2.0;
                                    [0.0, half, -half]
                                        .iter()
                                        .filter_map(|&o| ground.height(p[k].north + nn * o, p[k].east + ne * o))
                                        .fold(f32::MAX, f32::min)
                                })
                                .collect()
                        })
                        .collect::<Vec<Vec<f32>>>()
                })
            })
            .collect();
        jobs.into_iter().flat_map(|job| job.join().unwrap_or_default()).collect()
    });
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    // Downstream: upstream reaches first; each reach no higher than what arrives at its start.
    let mut order: Vec<usize> = (0..reaches.len()).collect();
    order.sort_by(|&a, &b| ends_of[b].3.total_cmp(&ends_of[a].3));
    let mut arriving: std::collections::HashMap<usize, f32> = std::collections::HashMap::new();
    for &r in &order {
        let (start, end, end_level, _) = ends_of[r];
        let mut cap = arriving.get(&start).copied().unwrap_or(f32::MAX);
        let floor = end_level.unwrap_or(sea).max(sea);
        let points = &mut reaches[r].points;
        let count = points.len();
        for (k, point) in points.iter_mut().enumerate() {
            let low = if lows[r][k] == f32::MAX { cap } else { lows[r][k] };
            let mut level = low.min(cap).max(floor).floor();
            if k + 1 == count
                && let Some(at) = end_level
            {
                level = at.max(sea);
            }
            cap = level;
            point.level = level;
            point.curtain = f32::NAN;
        }
        let last = points.last().map_or(sea, |p| p.level);
        arriving.entry(end).and_modify(|l| *l = l.min(last)).or_insert(last);
    }
    // Which rises hold a pond and which are cut through, each reach on its own, on all cores.
    let ponds: Vec<Ponds> = if s.rise_pools {
        let each = reaches.len().div_ceil(cores).max(1);
        let reaches = &*reaches;
        let lows = &lows;
        let made: Vec<Option<Ponds>> = std::thread::scope(|scope| {
            let jobs: Vec<_> = (0..reaches.len())
                .collect::<Vec<_>>()
                .chunks(each)
                .map(|part| {
                    let part = part.to_vec();
                    scope.spawn(move || part.into_iter().map(|r| ponds_of(&reaches[r].points, &lows[r], ground, s, stop)).collect::<Vec<_>>())
                })
                .collect();
            jobs.into_iter().flat_map(|job| job.join().unwrap_or_else(|_| vec![None])).collect()
        });
        made.into_iter().collect::<Option<_>>()?
    } else {
        reaches.iter().map(|reach| Ponds { cut: vec![false; reach.points.len()], too_far: 0, lakes: Vec::new() }).collect()
    };
    // Then downstream reaches first: each ends at the level of the water it joins (at a junction
    // each reach's level comes from the ground across its own width, so a creek could end blocks
    // above the river it meets), then its steps and drops.
    let from: std::collections::HashMap<usize, usize> = ends_of.iter().enumerate().map(|(r, e)| (e.0, r)).collect();
    let held = Lakes::new(ponds.iter().flat_map(|p| p.lakes.iter().cloned()).collect());
    for &r in order.iter().rev() {
        let (_, end, end_level, _) = ends_of[r];
        let joins = if end_level.is_none() { from.get(&end).and_then(|&next| reaches[next].points.first()).map(|p| p.level) } else { None };
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
        if step > 1.0 {
            let mut run = f32::NAN;
            for point in points.iter_mut().rev() {
                if run.is_nan() || point.level - run >= step {
                    run = point.level;
                } else {
                    point.level = run;
                }
            }
        }
        // Each drop: a waterfall (sheer, its curtain on the next point) or rapids (spread upstream,
        // a block at a time).
        let mut k = 1;
        while k < points.len() {
            let drop = points[k - 1].level - points[k].level;
            if drop >= fall_from {
                if s.curtains {
                    points[k].curtain = points[k - 1].level;
                }
            } else if drop >= 1.0 {
                let lower = points[k].level;
                let spread = (drop * s.rapids_length).max(1.0);
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
                    run += ((points[j].north - points[j - 1].north).powi(2) + (points[j].east - points[j - 1].east).powi(2)).sqrt();
                    j -= 1;
                }
            }
            k += 1;
        }
    }
    // How far the water passes below the lowest ground across it: the deepest, and how many points
    // deeper than a deep cut.
    let cuts = reaches.iter().zip(&lows).flat_map(|(reach, low)| reach.points.iter().zip(low).filter(|&(_, &l)| l < f32::MAX).map(|(p, &l)| l - p.level));
    let (deepest, deep) = cuts.fold((0.0f32, 0usize), |(deepest, deep), cut| (deepest.max(cut), deep + (cut > DEEP_CUT) as usize));
    let cut_instead = ponds.iter().map(|p| p.too_far).sum();
    Some((deepest, deep, ponds.into_iter().flat_map(|p| p.lakes).collect(), cut_instead))
}

/// A reach's rises: which are cut through though deeper than the cut (their pond would spread too
/// far), by point, and the ponds the others hold.
struct Ponds {
    cut: Vec<bool>,
    too_far: usize,
    lakes: Vec<Lake>,
}

/// Which rises across a reach hold its water back in a pond (David's "marble"). From the end
/// upward, the water has to be no lower than each rise less the cut (`cut_up_to`), so the rise that
/// sets that height holds the water above it; where that leaves the water standing above the ground,
/// it's a pond, filled out over the real ground at that height. A pond that spreads further than
/// `pool_reach` from the river (or off the map) can't be held: that rise is cut through instead, and
/// the rest looked at again. `low` is the lowest ground across each point (as `lay_water` reads it),
/// and each point's level is the water's with every rise cut through. `None` if stopped.
fn ponds_of(points: &[RiverPoint], low: &[f32], ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<Ponds> {
    let count = points.len();
    let (mut cut, mut too_far) = (vec![false; count], 0);
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
            match pond(points, under, level, ground, s, stop)? {
                Pond::Held(lake) => {
                    seen.insert(rise, (under.clone(), Some(lake)));
                }
                Pond::Dry => {
                    seen.insert(rise, (under.clone(), None));
                }
                Pond::TooFar => {
                    // Cut through, with the points beside it up and down the rise whose ponds would
                    // be much the same (each a little lower, so as big).
                    let close = |k: usize| low[k] < f32::MAX && (low[k] - s.cut_up_to.max(0.0)).floor() >= level - s.cut_up_to.max(1.0);
                    (cut[rise], too_far) = (true, too_far + 1);
                    for k in (0..rise).rev().take_while(|&k| close(k)).chain((rise + 1..count).take_while(|&k| close(k))).collect::<Vec<_>>() {
                        cut[k] = true;
                    }
                    seen.remove(&rise);
                    changed = true;
                }
            }
        }
        if !changed {
            let lakes = held.keys().filter_map(|rise| seen.remove(rise).and_then(|(_, lake)| lake)).collect();
            return Some(Ponds { cut, too_far, lakes });
        }
    }
}

/// A rise's pond: none (the ground isn't below it after all, or it's the sea's), held, or spreading
/// too far to hold.
enum Pond {
    Dry,
    Held(Lake),
    TooFar,
}

/// The pond held at `level` round some of a reach's points: the real ground below it reached from
/// under them, on cells `grain` blocks across; too far if it spreads more than `pool_reach` from
/// them, or off the map. `None` if stopped.
fn pond(points: &[RiverPoint], under: &[usize], level: f32, ground: &CompiledMap, s: &CarvingSettings, stop: &AtomicBool) -> Option<Pond> {
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
    let mut cells: std::collections::HashMap<(i64, i64), bool> = std::collections::HashMap::new();
    let mut queue = Vec::new();
    for &k in under {
        let at = ((points[k].north / g).floor() as i64, (points[k].east / g).floor() as i64);
        if let std::collections::hash_map::Entry::Vacant(cell) = cells.entry(at) {
            // (Its cell is wet if the river is under water there, though the cell's middle may not be.)
            let w = wet(at.0, at.1) == Some(true) || ground.height(points[k].north, points[k].east).is_some_and(|h| h < level - 0.5);
            cell.insert(w);
            if w {
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
    let fade = (width * s.fade_widths[(rank as usize).clamp(1, 4) - 1]).max(s.fade_least);
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

    /// A creek running west along row 32 over ground rising east, 8 blocks a pixel, across a ridge 20
    /// high (columns 30 to 33), in a valley walled north and south; its water laid with `s`.
    fn over_a_ridge(s: &CarvingSettings) -> (Vec<RiverPoint>, (f32, usize, Vec<Lake>, usize), CompiledMap) {
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
        let point = |c: f32| RiverPoint { north: -4.0, east: (c + 0.5 - 32.0) * 8.0, width: 3.0, depth: 2.0, edge: 0.3, flat: 0.2, bowl: 1.0, level: f32::NAN, bend: 0.0, curtain: f32::NAN, turn: 0.0, fade: 2.0 };
        let mut reaches = vec![Reach { points: (0..=112).map(|k| point(60.0 - k as f32 * 0.5)).collect() }];
        let laid = lay_water(&mut reaches, &[(0, 1, None, 100.0)], &ground, 1.0, 3.0, s, &AtomicBool::new(false)).unwrap();
        (reaches.remove(0).points, laid, ground)
    }

    #[test]
    fn a_rise_too_deep_to_cut_holds_a_pond_behind_it_and_is_cut_no_deeper_than_the_cut() {
        // David: "these streams either need to ride over the terrain like a marble, or cut far deeper".
        let s = CarvingSettings::default();
        let (points, (_, _, ponds, cut_instead), ground) = over_a_ridge(&s);
        assert_eq!((ponds.len(), cut_instead), (1, 0), "one pond, none cut through");
        let lakes = Lakes::new(ponds);
        let cut = |p: &RiverPoint| ground.height(p.north, p.east).unwrap() - p.level;
        // Over the ridge, cut through no deeper than the cut (and a block for its rounding).
        let ridge: Vec<&RiverPoint> = points.iter().filter(|p| (-16.0..16.0).contains(&p.east)).collect();
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
        let (points, (_, _, ponds, _), _) = over_a_ridge(&CarvingSettings { rise_pools: false, ..s });
        assert!(ponds.is_empty());
        assert!(points.iter().map(cut).fold(0.0, f32::max) > 15.0, "no notch with the ponds off");
    }

    #[test]
    fn a_pond_that_would_spread_too_far_is_cut_through_instead() {
        let s = CarvingSettings { pool_reach: 16.0, ..CarvingSettings::default() };
        let (points, (_, _, ponds, cut_instead), ground) = over_a_ridge(&s);
        assert!(ponds.is_empty() && cut_instead >= 1, "{} ponds, {cut_instead} cut through", ponds.len());
        assert!(points.iter().map(|p| ground.height(p.north, p.east).unwrap() - p.level).fold(0.0, f32::max) > 15.0, "not cut through");
        // And the water never rises downstream.
        assert!(points.windows(2).all(|w| w[1].level <= w[0].level));
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
        // a waterfall (no rapids), so the levels compare point for point.
        let (flow, network, ground) = country();
        let lay = |step| make(&network, &flow, &ground, 32.0, 3, step, 1.0, &CarvingSettings::default(), &AtomicBool::new(false)).unwrap();
        let (unstepped, stepped) = (lay(1.0), lay(4.0));
        let mut tops = 0;
        for (a, b) in unstepped.reaches.iter().zip(&stepped.reaches) {
            for k in 0..b.points.len() {
                let (own, level) = (a.points[k].level, b.points[k].level);
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
}

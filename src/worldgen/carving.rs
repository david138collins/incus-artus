//! Carving's curves — the river stage's last part: each river of the network (`network.rs`) as a
//! smooth line with its width, bank depth and water level all along it, for the Interpreter to carve
//! (`world/carving.rs`) [David and Builder, 2026-10-30: "Carving: the curve and the carve in one go",
//! docs/worldgen.md]. The curve and the carve come from one making:
//! 1. **From the grid's steps to a line:** the network runs patch to patch (straight or diagonal, 32
//!    blocks a step). Corner cutting (Chaikin's method: each pass slices every corner off) rounds its
//!    elbows, more passes for bigger rivers; a gentle **wander** breaks the long straight diagonals
//!    corner cutting can't touch.
//! 2. **Lean outward:** each point moves toward the outside of its bend (fast water cuts the outer bank).
//! 3. **Lengthen (David's squiggles):** a side-to-side wave every 10 to 14 widths, its size by width,
//!    strong on flat ground and fading on steep.
//! 4. **The carve** (in the world): each block measures its distance to the line; the odd block of an
//!    even width goes to the outside of the bend.
//!
//! Safety: the water level comes from the network's path (by how far along the river), so it only
//! falls downstream however the line moves; the ends (junctions, lake outlets, mouths) are pinned, the
//! sideways moves fading to nothing there, so rivers always meet; and a line pushed over ground much
//! higher than its water is pulled back.

use super::network::Network;
use super::noise::{Fade, perlin};
use super::rivers::Flow;
use crate::world::{Carving, CompiledMap, Lakes, Reach, RiverPoint, SEA_LEVEL};
use serde::{Deserialize, Serialize};
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
    /// How far a river may cut below the ground beside its path (a little gorge) before its line is
    /// pulled back toward the path (blocks).
    pub deepest_gorge: f32,
    /// Banks: how deep a river is, for its width (1 = David's table: a 2-wide creek 2 deep, a 32-wide
    /// river 8, a 128-wide great river 16).
    pub depth: f32,
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
            deepest_gorge: 6.0,
            depth: 1.0,
            curtains: true,
            rapids_length: 3.0,
            deltas: true,
            delta_flat: 0.01,
            delta_width: [64.0, 128.0],
        }
    }
}

/// The widths of the ranks (David's table, blocks): creek, stream, river, great river; and a spring.
const BANDS: [(f32, f32); 4] = [(2.0, 4.0), (4.0, 8.0), (8.0, 32.0), (32.0, 128.0)];
const SPRING: f32 = 1.0;

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
    level: f32,
}

/// Makes the curves from the network (`full_size`: the share of a river's length, from its mouth,
/// that keeps its full width before narrowing toward its spring [David]); `None` if stopped.
pub fn make(network: &Network, flow: &Flow, ground: &CompiledMap, blocks: f32, seed: u64, full_size: f32, step: f32, fall_from: f32, s: &CarvingSettings, stop: &AtomicBool) -> Option<Carving> {
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
    // How far each river patch is from its mouth, and the longest way up from it to a spring.
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
    let mut to_spring = vec![0.0f32; n];
    // Upstream first: by distance from the mouth, furthest first.
    let mut by_far: Vec<usize> = (0..n).filter(|&i| on(i)).collect();
    by_far.sort_by(|&a, &b| to_mouth[b].total_cmp(&to_mouth[a]));
    for &i in &by_far {
        let d = down(i);
        if d != i && on(d) {
            to_spring[d] = to_spring[d].max(to_spring[i] + patch(i, d));
        }
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    // A patch's width: its rank's (the middle of its band), narrowing toward the spring past the
    // full-size share of the river's length.
    let width_at = |i: usize| {
        let rank = network.rank[i].clamp(1, 4) as usize;
        let (low, high) = BANDS[rank - 1];
        let full = (low * high).sqrt();
        let length = to_mouth[i] + to_spring[i];
        let up = if length > 0.0 { to_mouth[i] / length } else { 0.0 };
        let share = full_size.clamp(0.0, 1.0);
        let taper = if up <= share { 1.0 } else { 1.0 - smooth((up - share) / (1.0 - share).max(1e-3)) };
        SPRING + (full - SPRING) * taper
    };

    // The reaches: from each spring or junction down to the next junction, lake, or the sea.
    let starts: Vec<usize> = (0..n).filter(|&i| on(i) && ups[i].len() != 1).collect();
    let mut reaches = Vec::new();
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
            let (north, east) = place(i);
            path.push(PathPoint { north, east, width: width_at(i), level: network.bed[i].round() });
            let d = down(i);
            if d == i {
                break;
            }
            if !on(d) {
                // Into the sea (or a lake): the line goes on to its edge, at its water's level.
                let (north, east) = place(d);
                let level = if flow.lake[d] > 0.0 { flow.surface[d].round() } else { SEA_LEVEL as f32 };
                (end, end_level) = (d, Some(level));
                path.push(PathPoint { north, east, width: width_at(i), level: level.min(path.last().map_or(level, |p| p.level)) });
                break;
            }
            if ups[d].len() >= 2 {
                // A junction: this reach ends on it (pinned), the next starts from it.
                end = d;
                let (north, east) = place(d);
                path.push(PathPoint { north, east, width: width_at(d), level: network.bed[d].round() });
                break;
            }
            i = d;
        }
        if path.len() < 2 {
            continue;
        }
        let rank = network.rank[start].max(1) as u32;
        reaches.push(curve(&path, k as u64, rank, flow, blocks, seed, fall_from, s));
        ends_of.push((start, end, end_level, to_mouth[start]));
        // A delta, where a river or great river meets the sea over flat ground.
        if s.deltas
            && rank >= 3
            && let Some(delta) = delta(&path, rank, s)
        {
            deltas.extend(delta);
        }
    }
    // The water levels, from the real ground (never above the banks), only falling downstream.
    lay_water(&mut reaches, &ends_of, ground, step, fall_from, s, stop)?;
    reaches.extend(deltas);
    let lakes = Lakes { width: w, blocks, surface: (0..n).map(|i| if flow.lake[i] > 0.0 { flow.surface[i] } else { f32::NAN }).collect() };
    Some(Carving::new(reaches, lakes))
}

/// One reach's curve: corner cut, then each point moved sideways by the wander, the lean and the
/// meander (fading to nothing at the pinned ends, and pulled back over high ground), with the width,
/// depth and level by how far along the river.
fn curve(path: &[PathPoint], id: u64, rank: u32, flow: &Flow, blocks: f32, seed: u64, fall_from: f32, s: &CarvingSettings) -> Reach {
    // How far along the path each of its points is.
    let mut along = vec![0.0f32];
    for pair in path.windows(2) {
        along.push(along.last().unwrap() + ((pair[1].north - pair[0].north).powi(2) + (pair[1].east - pair[0].east).powi(2)).sqrt());
    }
    let total = along.last().copied().unwrap_or(0.0).max(1e-3);
    // 1. Corner cutting, keeping the ends.
    let mut line: Vec<(f32, f32)> = path.iter().map(|p| (p.north, p.east)).collect();
    for _ in 0..(s.smoothing + rank.saturating_sub(1)).min(8) {
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
    let spacing = (path.iter().map(|p| p.width).fold(1.0f32, f32::max) / 2.0).clamp(2.0, 6.0);
    let line = resample(&line, spacing);
    let length = line.windows(2).map(|p| ((p[1].0 - p[0].0).powi(2) + (p[1].1 - p[0].1).powi(2)).sqrt()).sum::<f32>().max(1e-3);
    // The river's attributes at a share of the way along (0 at its start, 1 at its end).
    let at = |share: f32| {
        let s_path = share * total;
        let k = along.iter().rposition(|&a| a <= s_path).unwrap_or(0).min(path.len() - 2);
        let t = ((s_path - along[k]) / (along[k + 1] - along[k]).max(1e-3)).clamp(0.0, 1.0);
        (path[k].width + (path[k + 1].width - path[k].width) * t, k, t)
    };
    // The water level: each path point's, changing halfway to the next; a fall there is sheer, a
    // smaller drop spread out as rapids, a block at a time.
    let level_at = |share: f32| -> (f32, f32) {
        let s_path = share * total;
        let k = along.iter().rposition(|&a| a <= s_path).unwrap_or(0).min(path.len() - 2);
        let (upper, lower) = (path[k].level, path[k + 1].level);
        let middle = (along[k] + along[k + 1]) / 2.0;
        let drop = upper - lower;
        if drop <= 0.0 {
            return (upper, f32::NAN);
        }
        if drop >= fall_from {
            // A waterfall at the middle; just below it, its curtain.
            return if s_path < middle { (upper, f32::NAN) } else if s_path < middle + 1.5 { (lower, upper) } else { (lower, f32::NAN) };
        }
        let spread = (drop * s.rapids_length).max(1.0);
        let into = ((s_path - (middle - spread / 2.0)) / spread).clamp(0.0, 1.0);
        ((upper - (into * drop).floor()).max(lower), f32::NAN)
    };
    // The ground's slope along the river, for how freely it meanders.
    let slope = ((path[0].level - path[path.len() - 1].level) / total).max(0.0);
    let flatness = 1.0 - smooth(slope / s.flat.max(1e-4));
    let ground = |north: f32, east: f32| sample(flow, blocks, north, east);
    let phase = perlin(seed, 0xa1e0 + id, Fade::Smooth, id as f32 * 0.37, 0.5) * 6.28;
    let mut points = Vec::with_capacity(line.len());
    let mut run = 0.0f32;
    for k in 0..line.len() {
        if k > 0 {
            run += ((line[k].0 - line[k - 1].0).powi(2) + (line[k].1 - line[k - 1].1).powi(2)).sqrt();
        }
        let share = run / length;
        let (width, _, _) = at(share);
        let (level, curtain) = level_at(share);
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
        // Pulled back where it would cut deeper than a little gorge beside the path.
        for _ in 0..4 {
            let (north, east) = (line[k].0 + nn * sideways, line[k].1 + ne * sideways);
            if ground(north, east) - level <= s.deepest_gorge {
                break;
            }
            sideways *= 0.5;
        }
        let depth = ((width.sqrt() * std::f32::consts::SQRT_2 * s.depth).round()).clamp(1.0, 16.0);
        points.push(RiverPoint {
            north: line[k].0 + nn * sideways,
            east: line[k].1 + ne * sideways,
            width,
            depth,
            level,
            bend: if bend.abs() < 0.002 { 0.0 } else { bend.signum() },
            curtain: if s.curtains { curtain } else { f32::NAN },
        });
    }
    Reach { points }
}

/// The rivers' water levels, from the real ground [2026-10-30: taken from the network's broad
/// ground, the water stood above the real ground wherever hills dipped below it or the line swung
/// off its path, a ribbon of water walls (David's screenshot)]. Each point's level is the lowest real
/// ground across the river's width there, so the water is never above its banks; then only falling
/// downstream (a reach's start no higher than what flows into it, upstream reaches first); then
/// rounded down into steps of the bed's step size (flat runs, then drops); a drop as tall as a
/// waterfall stays sheer (its curtain just below), a smaller one is spread upstream as rapids, a
/// block at a time. Never below the sea's surface, and a reach into the sea or a lake ends at its
/// level.
fn lay_water(reaches: &mut [Reach], ends_of: &[(usize, usize, Option<f32>, f32)], ground: &CompiledMap, step: f32, fall_from: f32, s: &CarvingSettings, stop: &AtomicBool) -> Option<()> {
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
    for r in order {
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
        // Steps: rounded down to the step size (flat runs, then drops), never below the end's level.
        if step > 1.0 {
            for point in points.iter_mut() {
                point.level = ((point.level / step).floor() * step).max(floor);
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
    Some(())
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

/// The broad ground's height at a spot (from step 5's filled ground on the map's grid, blended).
fn sample(flow: &Flow, blocks: f32, north: f32, east: f32) -> f32 {
    let w = flow.width;
    let half = w as f32 / 2.0;
    let (x, y) = (east / blocks + half - 0.5, half - north / blocks - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let last = w as f32 - 1.0;
    let at = |c: f32, r: f32| {
        let v = flow.surface[r.clamp(0.0, last) as usize * w + c.clamp(0.0, last) as usize];
        if v.is_finite() { v } else { SEA_LEVEL as f32 }
    };
    (at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx) * (1.0 - ty) + (at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx) * ty
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
    let width = (end.width / 3.0).max(2.0);
    let level = SEA_LEVEL as f32;
    let spread = size / 2.0;
    Some(
        [-1.0f32, 0.0, 1.0]
            .iter()
            .map(|&side| {
                // Fanning out sideways (perpendicular to the river) to the delta's width at the coast.
                let (tip_n, tip_e) = (start.north + un * length * 1.1 - ue * side * spread, start.east + ue * length * 1.1 + un * side * spread);
                let point = |north: f32, east: f32| RiverPoint { north, east, width, depth: (width.sqrt() * std::f32::consts::SQRT_2 * s.depth).round().clamp(1.0, 16.0), level, bend: 0.0, curtain: f32::NAN };
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

    fn carve(full_size: f32) -> Carving {
        let (flow, network, ground) = country();
        make(&network, &flow, &ground, 32.0, 3, full_size, 4.0, 3.0, &CarvingSettings::default(), &AtomicBool::new(false)).unwrap()
    }

    #[test]
    fn curves_start_pinned_on_their_paths_and_their_water_never_rises_downstream() {
        let carving = carve(0.5);
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
    fn rivers_narrow_toward_their_springs_past_the_full_size_share() {
        let narrowest = |carving: &Carving| carving.reaches.iter().flat_map(|r| r.points.iter()).map(|p| p.width).fold(f32::MAX, f32::min);
        // Full size all the way up: never narrower than a creek; none of the way: down to a spring.
        assert!(narrowest(&carve(1.0)) >= BANDS[0].0);
        assert!(narrowest(&carve(0.0)) <= SPRING + 0.01);
    }


    #[test]
    fn the_water_never_stands_above_the_real_ground_under_it() {
        // David's screenshot: water laid on the broad ground stood above the real ground as walls.
        let (flow, network, ground) = country();
        let carving = make(&network, &flow, &ground, 32.0, 3, 0.5, 4.0, 3.0, &CarvingSettings::default(), &AtomicBool::new(false)).unwrap();
        for p in carving.reaches.iter().flat_map(|reach| reach.points.iter()) {
            let real = ground.height(p.north, p.east).unwrap();
            assert!(p.level <= real.max(SEA_LEVEL as f32) + 1e-3, "water at {} over ground at {real}", p.level);
        }
    }
    #[test]
    fn every_river_point_carves_water_into_the_ground() {
        let carving = carve(0.5);
        for reach in carving.reaches.iter().take(5) {
            for p in reach.points.iter().step_by(7) {
                let here = carving.here(p.north, p.east, p.level + 5.0);
                assert!(here.water.is_some() && here.floor.is_some(), "no river at {p:?}");
                assert!(here.floor.unwrap() < p.level, "the floor is below the water");
            }
        }
    }
}

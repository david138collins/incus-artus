//! Rivers — the last of the Climate stages, where water runs on the ground. [Plan:
//! "Rivers, the first part of Climate" and "Breaking the water loop", docs/worldgen.md.]
//!
//! **Step 5, the flow** (built 2026-10-28; river lines, ranks, carving come after):
//! - **The broad ground on the map's own grid** (32 blocks a pixel), so valleys show.
//! - **Basins fill and spill** (the priority-flood method): starting from the sea and the world's
//!   edge, the lowest ground not yet reached is always taken next, and whatever it reaches drains into
//!   it. A dip with no way out is reached only over its rim, so it fills to the rim's lowest point
//!   (its spill height) and overflows there: a lake with a flat surface and an outlet, and the water
//!   carries on downhill. Every patch drains somewhere, always downhill or across a lake, always
//!   reaching the sea. Water's hard rules, kept. One pass.
//! - **Ties are broken by a fixed nudge** from the world's seed (on flat ground, which way is
//!   downhill is a toss-up): the paths wander, the same every time, instead of running in the grid's
//!   straight lines.
//! - **The flow adds up downhill:** each patch of land gives a unit of water, plus **snowmelt** on
//!   cold high ground and **springs** where a slope eases off (David's two picks), and passes it all
//!   on to the patch it drains into, upstream first. Not the rain [David, 2026-10-30: rivers aren't
//!   driven by rain; it was each patch's rain, 1 unit = a patch of the wettest].
//!
//! Lakes are the patches standing under their basin's surface (deeper than a setting). Fresh lakes
//! only, for now: every basin fills and spills (salt lakes, which stand lower, come later).

use super::climate::Ground;
use super::temperature::Temperature;
use crate::world::SEA_LEVEL;
use bevy_egui::egui::Color32;
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering as Order, Reverse};
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, Ordering};

/// The Rivers section's settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RiverSettings {
    /// Snowmelt: how much extra water ground gives (units, on its own one) where it's colder than
    /// `snow_below` (−1 frozen to +1 hottest) and higher than `snow_above` (Z).
    pub snowmelt: f32,
    pub snow_below: f32,
    pub snow_above: f32,
    /// Springs where a slope eases off: how much extra water (units for each block of steepness
    /// lost, per block across).
    pub springs: f32,
    /// How deep (blocks) a basin's water has to stand to count as a lake.
    pub lake_depth: f32,
    /// How much flow (units) a patch needs to be drawn as water.
    pub shown_from: f32,
}

impl Default for RiverSettings {
    fn default() -> Self {
        Self { snowmelt: 1.0, snow_below: -0.2, snow_above: 300.0, springs: 2.0, lake_depth: 2.0, shown_from: 40.0 }
    }
}

/// Where the water goes, on the map's own grid.
#[derive(Clone, Debug)]
pub struct Flow {
    pub width: usize,
    /// The patch each drains into (itself for the sea, outlets and past the world's edge).
    pub down: Vec<u32>,
    /// The water's surface: the ground, or a lake's level over it.
    pub surface: Vec<f32>,
    /// How much water flows through each (units), and how deep its lake is (blocks; 0 = none).
    pub flow: Vec<f32>,
    pub lake: Vec<f32>,
    /// The most flow anywhere on land, and the share of land under lakes.
    pub most: f32,
    pub lake_share: f32,
}

/// A patch waiting to be reached, lowest first; ties broken by a fixed nudge, then by place.
#[derive(PartialEq)]
struct Waiting {
    level: f32,
    nudge: u32,
    at: u32,
}

impl Eq for Waiting {}

impl PartialOrd for Waiting {
    fn partial_cmp(&self, other: &Self) -> Option<Order> {
        Some(self.cmp(other))
    }
}

impl Ord for Waiting {
    fn cmp(&self, other: &Self) -> Order {
        self.level.total_cmp(&other.level).then(self.nudge.cmp(&other.nudge)).then(self.at.cmp(&other.at))
    }
}

/// Where the water goes over the ground (`z`, the map's own grid: `None` past the world's edge),
/// with each patch's water given by `water` (units). `seed` nudges ties. `None` if stopped.
pub fn flow(width: usize, z: &[Option<f32>], water: &[f32], lake_depth: f32, seed: u64, stop: &AtomicBool) -> Option<Flow> {
    let n = width * width;
    let sea = SEA_LEVEL as f32;
    let nudge = |i: usize| (crate::fate::planet_hash(seed, 0xf10, (i % width) as i64, (i / width) as i64, 0) >> 32) as u32;
    let mut down: Vec<u32> = (0..n as u32).collect();
    let mut surface = vec![f32::NAN; n];
    let mut reached = vec![false; n];
    // The order patches were reached in (lowest first): upstream is later.
    let mut order: Vec<u32> = Vec::with_capacity(n);
    let mut waiting = BinaryHeap::new();
    // The outlets: the sea, past the world's edge, and the map's own edge.
    for i in 0..n {
        let (c, r) = (i % width, i / width);
        let edge = c == 0 || r == 0 || c + 1 == width || r + 1 == width;
        match z[i] {
            None => {
                reached[i] = true;
                surface[i] = sea;
            }
            Some(h) if h < sea || edge => {
                reached[i] = true;
                surface[i] = h;
                waiting.push(Reverse(Waiting { level: h, nudge: nudge(i), at: i as u32 }));
            }
            Some(_) => {}
        }
    }
    let mut taken = 0usize;
    while let Some(Reverse(Waiting { level, at, .. })) = waiting.pop() {
        let at = at as usize;
        order.push(at as u32);
        taken += 1;
        if taken % 65_536 == 0 && stop.load(Ordering::Relaxed) {
            return None;
        }
        let (c, r) = ((at % width) as i64, (at / width) as i64);
        for (dc, dr) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let (nc, nr) = (c + dc, r + dr);
            if nc < 0 || nr < 0 || nc >= width as i64 || nr >= width as i64 {
                continue;
            }
            let next = nr as usize * width + nc as usize;
            if reached[next] {
                continue;
            }
            reached[next] = true;
            // Reached from here, it drains here; standing lower than here, it's under a lake that
            // fills to this level.
            let h = z[next].unwrap_or(sea);
            down[next] = at as u32;
            surface[next] = h.max(level);
            waiting.push(Reverse(Waiting { level: surface[next], nudge: nudge(next), at: next as u32 }));
        }
    }
    // The flow, upstream first (the reverse of the order reached).
    let mut flow: Vec<f32> = water.to_vec();
    for &i in order.iter().rev() {
        let i = i as usize;
        let d = down[i] as usize;
        if d != i {
            flow[d] += flow[i];
        }
    }
    let lake: Vec<f32> = (0..n).map(|i| z[i].map_or(0.0, |h| if h >= sea && surface[i] - h >= lake_depth { surface[i] - h } else { 0.0 })).collect();
    let land: Vec<usize> = (0..n).filter(|&i| z[i].is_some_and(|h| h >= sea)).collect();
    let most = land.iter().map(|&i| flow[i]).fold(0.0f32, f32::max);
    let lake_share = land.iter().filter(|&&i| lake[i] > 0.0).count() as f32 / land.len().max(1) as f32;
    Some(Flow { width, down, surface, flow, lake, most, lake_share })
}

/// Each patch's water (units): one for each patch of land, plus snowmelt and springs [David,
/// 2026-10-30: rivers aren't driven by rain; "we shouldn't confuse rivers with rain" (rivers are a
/// co-factor of wetness, with rain, for Ecology). It was each patch's rain]. `z` is the broad ground
/// on the map's grid; `ground` (the climate pixels) tells which climate pixel holds each patch.
pub fn water(z: &[Option<f32>], width: usize, ground: &Ground, temperature: &Temperature, s: &RiverSettings, blocks: f32) -> Vec<f32> {
    let sea = SEA_LEVEL as f32;
    let at = |c: i64, r: i64, fallback: f32| if c < 0 || r < 0 || c >= width as i64 || r >= width as i64 { fallback } else { z[r as usize * width + c as usize].unwrap_or(fallback) };
    (0..width * width)
        .map(|i| {
            let Some(h) = z[i] else { return 0.0 };
            if h < sea {
                return 0.0;
            }
            let (c, r) = (i % width, i / width);
            let k = (r / ground.pixels).min(ground.width - 1) * ground.width + (c / ground.pixels).min(ground.width - 1);
            let mut units = 1.0;
            if temperature.t[k].is_some_and(|t| t < s.snow_below) && h > s.snow_above {
                units += s.snowmelt.max(0.0);
            }
            // Springs: steepness coming down to here, against steepness on from here (the steepest
            // drop each way, per block across). Where it eases off, water comes up.
            let (ci, ri) = (c as i64, r as i64);
            let neighbours = [(-1, 0), (1, 0), (0, -1), (0, 1)].map(|(dc, dr)| at(ci + dc, ri + dr, h));
            let (above, below) = (neighbours.iter().cloned().fold(h, f32::max), neighbours.iter().cloned().fold(h, f32::min));
            let eased = ((above - h) - (h - below)) / blocks;
            units += s.springs.max(0.0) * eased.max(0.0);
            units
        })
        .collect()
}

impl Flow {
    /// The water as a picture on the map's grid: lakes filled (deeper, darker), and the flow drawn
    /// brighter and bluer the more it carries; dry land clear.
    pub fn picture(&self, shown_from: f32) -> Vec<Color32> {
        let top = self.most.max(shown_from * 2.0).ln();
        let from = shown_from.max(1e-3).ln();
        (0..self.width * self.width)
            .map(|i| {
                if self.lake[i] > 0.0 {
                    let deep = (self.lake[i] / 40.0).clamp(0.0, 1.0);
                    return Color32::from_rgba_unmultiplied(40, (150.0 - 80.0 * deep) as u8, (230.0 - 60.0 * deep) as u8, 235);
                }
                let f = self.flow[i];
                if f < shown_from || self.down[i] as usize == i {
                    return Color32::TRANSPARENT;
                }
                let k = ((f.ln() - from) / (top - from)).clamp(0.0, 1.0);
                Color32::from_rgba_unmultiplied((120.0 - 90.0 * k) as u8, (200.0 - 60.0 * k) as u8, 255, (140.0 + 115.0 * k) as u8)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_runs_downhill_to_the_sea_and_adds_up_on_the_way() {
        // A slope down to the sea in the west: every patch drains, and the flow grows toward the sea.
        let w = 32;
        let z: Vec<Option<f32>> = (0..w * w).map(|i| Some(if i % w < 4 { 100.0 } else { 130.0 + (i % w) as f32 * 3.0 })).collect();
        let f = flow(w, &z, &vec![1.0; w * w], 2.0, 7, &AtomicBool::new(false)).unwrap();
        for i in 0..w * w {
            let mut at = i;
            for _ in 0..w * w {
                if f.down[at] as usize == at {
                    break;
                }
                // Never uphill.
                assert!(f.surface[f.down[at] as usize] <= f.surface[at] + 1e-4);
                at = f.down[at] as usize;
            }
            assert_eq!(f.down[at] as usize, at, "patch {i} never reaches an outlet");
        }
        assert_eq!(f.lake_share, 0.0);
        // Near the coast it carries more than up the slope.
        let total_near = (0..w).map(|r| f.flow[r * w + 4]).sum::<f32>();
        let total_far = (0..w).map(|r| f.flow[r * w + 28]).sum::<f32>();
        assert!(total_near > total_far * 5.0, "near {total_near}, far {total_far}");
    }

    #[test]
    fn a_basin_fills_to_its_rim_and_spills_over_its_lowest_point() {
        // Flat land at Z 170 with a bowl down to Z 150 in the middle behind a rim at Z 210, and a notch
        // in the rim at Z 180 on the west side; the map's edge drains it all.
        let w = 40;
        let (cx, cy) = (20.0, 20.0);
        let z: Vec<Option<f32>> = (0..w * w)
            .map(|i| {
                let (c, r) = ((i % w) as f32, (i / w) as f32);
                let d = (c - cx).hypot(r - cy);
                let mut h = if d < 8.0 { 150.0 } else if d < 10.0 { 210.0 } else { 170.0 };
                if (8.0..10.0).contains(&d) && c < cx && (r - cy).abs() < 1.0 {
                    h = 180.0;
                }
                Some(h)
            })
            .collect();
        let f = flow(w, &z, &vec![1.0; w * w], 2.0, 7, &AtomicBool::new(false)).unwrap();
        let middle = 20 * w + 20;
        // A lake 30 deep: filled to the notch.
        assert!((f.surface[middle] - 180.0).abs() < 1e-3, "{}", f.surface[middle]);
        assert!((f.lake[middle] - 30.0).abs() < 1e-3);
        // Its water leaves through the notch: the notch carries the whole lake's flow.
        let notch = 20 * w + 11;
        assert!(f.flow[notch] > 150.0, "{}", f.flow[notch]);
    }

    /// How long a full-sized map takes (run by hand: `cargo test --release flow_time -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn flow_time() {
        let w = 2_250;
        let z: Vec<Option<f32>> = (0..w * w).map(|i| Some(120.0 + ((i % w) as f32 * 0.01).sin() * 80.0 + ((i / w) as f32 * 0.013).cos() * 60.0 + ((i * 7919) % 17) as f32)).collect();
        let started = std::time::Instant::now();
        let f = flow(w, &z, &vec![1.0; w * w], 2.0, 3, &AtomicBool::new(false)).unwrap();
        println!("flow over {w} x {w}: {:.2} s, most {:.0}, lakes {:.1}%", started.elapsed().as_secs_f32(), f.most, f.lake_share * 100.0);
    }

    #[test]
    fn the_same_ground_and_seed_give_the_same_flow() {
        let w = 24;
        let z: Vec<Option<f32>> = (0..w * w).map(|i| Some(140.0 + ((i * 7919) % 13) as f32)).collect();
        let a = flow(w, &z, &vec![1.0; w * w], 2.0, 3, &AtomicBool::new(false)).unwrap();
        let b = flow(w, &z, &vec![1.0; w * w], 2.0, 3, &AtomicBool::new(false)).unwrap();
        assert_eq!(a.down, b.down);
        assert_eq!(a.flow, b.flow);
    }
}

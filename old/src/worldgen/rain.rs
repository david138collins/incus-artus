//! Rain — the third of the Climate stages. [Plan: "Mountains' effect on wind and rain, the simplest
//! way", docs/worldgen.md; built 2026-10-28.] Mountains act on rain only (not on the wind).
//!
//! **Each climate pixel looks upwind.** From its middle it steps back against the wind, a climate
//! pixel at a time and following the wind's bends, for a set distance, reading the broad ground on
//! the way. Three things come out of that one walk, each its own setting and view:
//! - **Rain shadow:** how far the highest ground upwind stands above this spot (nearer ground counting
//!   more): high ground upwind took the rain, so it's dry here, through a curve.
//! - **Facing the wind:** ground rising toward this spot from just upwind pushes the air up, and it
//!   rains: wetter on the windward slopes.
//! - **Far from the sea, upwind:** how far back along the wind the sea is; further inland, drier.
//!
//! Warm air carries more water (a setting), and a last curve shapes how the rain looks, keeping the
//! rough "true" sum apart from the look. Every pixel is worked out alone, in any order, the same every
//! time. The fuller way (carrying moisture along the wind, picked up over the sea and spent as rain)
//! is the upgrade if this looks off.

use super::climate::Ground;
use super::runner::every_pixel;
use super::temperature::Temperature;
use super::wind::Wind;
use crate::world::{SEA_LEVEL, Spline};
use egui::Color32;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;

/// The Rain section's settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RainSettings {
    /// How far upwind each spot looks (blocks).
    pub reach: f32,
    /// Rain shadow: how much it dries (0 to 1), how much higher the ground upwind has to stand for
    /// the full shadow (blocks), and the curve from none (left) to that (right).
    pub shadow: f32,
    pub shadow_height: f32,
    pub shadow_curve: Spline,
    /// Facing the wind: how much wetter (0 to 1), and how much the ground has to rise toward the spot
    /// over how far upwind (blocks) for all of it.
    pub windward: f32,
    pub windward_rise: f32,
    pub windward_reach: f32,
    /// Far from the sea: how much it dries (0 to 1), and how far upwind the sea has to be for all of it
    /// (blocks).
    pub inland: f32,
    pub inland_reach: f32,
    /// How much warm air carrying more water matters (0 = not at all): the cold side drier.
    pub warmth: f32,
    /// The look: the rain worked out (left, dry to wet) to the rain shown (up).
    pub look: Spline,
}

impl Default for RainSettings {
    fn default() -> Self {
        Self {
            reach: 20_000.0,
            shadow: 0.7,
            shadow_height: 250.0,
            shadow_curve: Spline::default(),
            windward: 0.4,
            windward_rise: 80.0,
            windward_reach: 2_000.0,
            inland: 0.5,
            inland_reach: 30_000.0,
            warmth: 0.3,
            look: Spline::default(),
        }
    }
}

/// The rain over the map, a climate pixel at a time, with its parts (for the views and readouts).
#[derive(Clone, Debug)]
pub struct Rain {
    pub width: usize,
    /// How wet, 0 (dry) to 1 (wettest); `None` past the edge of the world.
    pub rain: Vec<Option<f32>>,
    /// Each part's say, 0 to 1: how much the shadow dried it, how much facing the wind wetted it, how
    /// much being far from the sea dried it.
    pub shadow: Vec<f32>,
    pub windward: Vec<f32>,
    pub inland: Vec<f32>,
}

/// One spot's walk upwind: the shadow, windward and inland parts (0 to 1 each).
fn walk(ground: &Ground, wind: &Wind, s: &RainSettings, cell: f32, c: usize, r: usize) -> [f32; 3] {
    let w = ground.width;
    let sea = SEA_LEVEL as f32;
    let at = |x: f32, y: f32| -> Option<(usize, f32)> {
        if x < 0.0 || y < 0.0 || x >= w as f32 || y >= w as f32 {
            return None;
        }
        let i = y as usize * w + x as usize;
        Some((i, ground.z[i]?.max(sea)))
    };
    let Some((_, here)) = at(c as f32 + 0.5, r as f32 + 0.5) else { return [0.0; 3] };
    let steps = (s.reach / cell).ceil().max(1.0) as usize;
    let (mut x, mut y) = (c as f32 + 0.5, r as f32 + 0.5);
    let (mut shadow, mut rise, mut sea_at): (f32, f32, Option<f32>) = (0.0, 0.0, None);
    let windward_steps = (s.windward_reach / cell).round().max(1.0) as usize;
    for step in 1..=steps {
        let Some((i, _)) = at(x, y) else { break };
        // A step back against the wind here.
        let [east, south] = wind.way[i];
        x -= east;
        y -= south;
        let Some((j, z)) = at(x, y) else { break };
        let behind = step as f32 / steps as f32;
        // Nearer high ground casts more of a shadow.
        shadow = shadow.max((z - here) * (1.0 - behind));
        if step == windward_steps {
            rise = here - z;
        }
        if sea_at.is_none() && ground.z[j].is_some_and(|z| z < sea) {
            sea_at = Some(step as f32 * cell);
        }
    }
    let shadow = (shadow / s.shadow_height.max(1.0)).clamp(0.0, 1.0);
    let windward = (rise / s.windward_rise.max(1.0)).clamp(0.0, 1.0);
    let inland = (sea_at.unwrap_or(s.reach.max(s.inland_reach)) / s.inland_reach.max(1.0)).clamp(0.0, 1.0);
    [shadow, windward, inland]
}

/// The rain from the ground, temperature and wind (`None` if stopped). `cell` is a climate pixel's
/// size in blocks.
pub fn make(ground: &Ground, temperature: &Temperature, wind: &Wind, s: &RainSettings, cell: f32, stop: &AtomicBool) -> Option<Rain> {
    let w = ground.width;
    let parts = every_pixel(w, stop, |i| walk(ground, wind, s, cell, i % w, i / w))?;
    let (shadow_curve, look) = (s.shadow_curve.curve(), s.look.curve());
    let rain = (0..w * w)
        .map(|i| {
            ground.z[i]?;
            let [shadow, windward, inland] = parts[i];
            let mut wet = 1.0;
            wet *= 1.0 - s.shadow.clamp(0.0, 1.0) * shadow_curve.at(shadow).clamp(0.0, 1.0);
            wet *= 1.0 - s.inland.clamp(0.0, 1.0) * inland;
            // Warm air carries more: the cold side drier (−1 frozen to +1 hottest, at sea level).
            wet *= 1.0 - s.warmth.clamp(0.0, 1.0) * (1.0 - (temperature.at_sea[i] + 1.0) / 2.0);
            wet += s.windward.clamp(0.0, 1.0) * windward * (1.0 - wet);
            Some(look.at(wet.clamp(0.0, 1.0)).clamp(0.0, 1.0))
        })
        .collect();
    Some(Rain { width: w, rain, shadow: parts.iter().map(|p| p[0]).collect(), windward: parts.iter().map(|p| p[1]).collect(), inland: parts.iter().map(|p| p[2]).collect() })
}

/// Which of the rain's views is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Rain,
    Shadow,
    Windward,
    Inland,
}

/// Rain's colour: dry tan, through green, to deep blue (wettest).
pub fn colour(wet: f32) -> [u8; 3] {
    const STOPS: [(f32, [f32; 3]); 4] = [(0.0, [205.0, 170.0, 105.0]), (0.35, [190.0, 195.0, 110.0]), (0.65, [70.0, 165.0, 90.0]), (1.0, [40.0, 80.0, 190.0])];
    let wet = wet.clamp(0.0, 1.0);
    let k = STOPS.windows(2).position(|pair| wet <= pair[1].0).unwrap_or(STOPS.len() - 2);
    let ((a0, a), (b0, b)) = (STOPS[k], STOPS[k + 1]);
    let f = ((wet - a0) / (b0 - a0)).clamp(0.0, 1.0);
    [0, 1, 2].map(|j| (a[j] + (b[j] - a[j]) * f) as u8)
}

impl Rain {
    /// One of the views as a picture: the rain in its colours, or a part in grey (white = all of it);
    /// the sea dimmed.
    pub fn picture(&self, ground: &Ground, view: View) -> Vec<Color32> {
        let sea = SEA_LEVEL as f32;
        (0..self.width * self.width)
            .map(|i| {
                let (Some(wet), Some(z)) = (self.rain[i], ground.z[i]) else { return Color32::TRANSPARENT };
                let [r, g, b] = match view {
                    View::Rain => colour(wet),
                    View::Shadow => [(self.shadow[i] * 255.0) as u8; 3],
                    View::Windward => [(self.windward[i] * 255.0) as u8; 3],
                    View::Inland => [(self.inland[i] * 255.0) as u8; 3],
                };
                if z < sea { Color32::from_rgb(r / 2, g / 2, b / 2) } else { Color32::from_rgb(r, g, b) }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ground: sea in the west quarter, flat land at Z 160, and a ridge up to Z 460 running north to
    /// south at column `ridge`; the wind blowing east everywhere.
    fn world(w: usize, ridge: usize) -> (Ground, Temperature, Wind) {
        let z = (0..w * w)
            .map(|i| {
                let c = i % w;
                Some(if c < w / 4 { 100.0 } else { 160.0 + 300.0 * (1.0 - (c as f32 - ridge as f32).abs() / 3.0).max(0.0) })
            })
            .collect();
        let ground = Ground { width: w, pixels: 4, z };
        let temperature = Temperature { width: w, t: vec![Some(0.5); w * w], at_sea: vec![0.5; w * w], lines: [vec![0.0; w], vec![w as f32 / 2.0; w], vec![w as f32; w]], mean: 0.5, frozen_land: 0.0, hottest_land: 0.0 };
        let wind = Wind { width: w, way: vec![[1.0, 0.0]; w * w], strength: vec![1.0; w * w] };
        (ground, temperature, wind)
    }

    #[test]
    fn behind_a_ridge_is_dry_its_windward_side_wet_and_far_from_the_sea_drier() {
        let w = 64;
        let (ground, temperature, wind) = world(w, 30);
        let s = RainSettings { reach: 128.0 * 30.0, inland_reach: 128.0 * 40.0, ..RainSettings::default() };
        let rain = make(&ground, &temperature, &wind, &s, 128.0, &AtomicBool::new(false)).unwrap();
        let at = |c: usize| rain.rain[20 * w + c].unwrap();
        // The lee of the ridge (east of it, downwind) is drier than the same flat ground before it.
        assert!(at(34) < at(26) - 0.1, "lee {} before {}", at(34), at(26));
        // Its windward slope is wetter than the flat ground before it.
        assert!(at(29) > at(26), "windward {} flat {}", at(29), at(26));
        // Further inland (past the ridge's shadow), drier than near the coast.
        assert!(at(62) < at(18), "far inland {} near the sea {}", at(62), at(18));
        assert!(rain.shadow[20 * w + 34] > 0.3 && rain.shadow[20 * w + 26] == 0.0);
    }

    #[test]
    fn rain_colours_run_dry_tan_to_wet_blue() {
        let (dry, wet) = (colour(0.0), colour(1.0));
        assert!(dry[0] > dry[2] && wet[2] > wet[0]);
    }
}

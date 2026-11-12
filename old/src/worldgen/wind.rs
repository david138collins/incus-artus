//! Wind — the second of the Climate stages. [Plan: "Breaking the water loop", docs/worldgen.md;
//! built 2026-10-28.] No fluid simulation: the wind is made from the temperature, one way.
//!
//! - **The temperature at sea level** (the ice caps, equator and sea's say, without the cooling with
//!   height): if the wind read the cooled temperature, every mountain range would be a cold spot that
//!   air blew away from on all sides, never across, and there'd be no rain shadows.
//! - **Blurred heavily**, so the wind follows the big picture, not every bay.
//! - **From cold toward hot**, along the blurred temperature's slope (as air pressure pushes it),
//!   stronger where it changes faster.
//! - **Turned partly sideways**, the way the Earth's spin bends winds: to the right north of the
//!   equator, to the left south of it, so it swirls round hot and cold spots instead of pointing
//!   straight at them, and the two halves of the world mirror each other, like trade winds.
//!
//! Mountains don't bend it (yet): they act on rain. Curl noise for eddies can come later.

use super::compiler::blur;
use super::temperature::Temperature;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// The Wind section's settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindSettings {
    /// How far the temperature is blurred before the wind reads it (blocks).
    pub blur: f32,
    /// How far the wind is turned sideways from straight cold-to-hot (degrees; right in the north,
    /// left in the south).
    pub turn: f32,
    /// How much the wind's strength evens out: 0 = as the slope says (calm where the temperature is
    /// flat), 1 = the same strength everywhere.
    pub even: f32,
}

impl Default for WindSettings {
    fn default() -> Self {
        Self { blur: 12_000.0, turn: 50.0, even: 0.3 }
    }
}

/// The wind over the map, a climate pixel at a time.
#[derive(Clone, Debug)]
pub struct Wind {
    pub width: usize,
    /// Which way it blows (east, south: a unit-length direction, on the climate pixels' grid), and how
    /// strongly (0 to 1).
    pub way: Vec<[f32; 2]>,
    pub strength: Vec<f32>,
}

/// The wind from the temperature (`None` if stopped). `cell` is a climate pixel's size in blocks.
pub fn make(temperature: &Temperature, s: &WindSettings, cell: f32, stop: &AtomicBool) -> Option<Wind> {
    let w = temperature.width;
    let mut t = temperature.at_sea.clone();
    blur(&mut t, w, (s.blur / cell / 2.0).round() as usize);
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    let at = |c: usize, r: usize| t[r * w + c];
    let (sin, cos) = s.turn.to_radians().sin_cos();
    let mut way = vec![[0.0f32; 2]; w * w];
    let mut slope = vec![0.0f32; w * w];
    for r in 0..w {
        for c in 0..w {
            let i = r * w + c;
            // Toward warmer: the slope east and south.
            let east = at((c + 1).min(w - 1), r) - at(c.saturating_sub(1), r);
            let south = at(c, (r + 1).min(w - 1)) - at(c, r.saturating_sub(1));
            let length = east.hypot(south);
            slope[i] = length;
            if length <= 1e-9 {
                continue;
            }
            let (e, s) = (east / length, south / length);
            // Turned to the right of the way it blows north of the equator, to the left south of it.
            // With east to the right and south down (as on the map), turning right is clockwise on
            // screen: (e, s) becomes (e cos − s sin, e sin + s cos).
            let north = (r as f32 + 0.5) < temperature.lines[1][c];
            let sin = if north { sin } else { -sin };
            way[i] = [e * cos - s * sin, e * sin + s * cos];
        }
    }
    let strongest = slope.iter().cloned().fold(0.0f32, f32::max).max(1e-9);
    let even = s.even.clamp(0.0, 1.0);
    let strength = slope.iter().map(|&k| even + (1.0 - even) * (k / strongest).sqrt()).collect();
    Some(Wind { width: w, way, strength })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temperature that's cold in the north and warms steadily southward to the equator in the
    /// middle, then cools again.
    fn banded(w: usize) -> Temperature {
        let at_sea: Vec<f32> = (0..w * w).map(|i| 1.0 - ((i / w) as f32 + 0.5 - w as f32 / 2.0).abs() / (w as f32 / 2.0) * 2.0).collect();
        Temperature { width: w, t: at_sea.iter().map(|&t| Some(t)).collect(), at_sea, lines: [vec![2.0; w], vec![w as f32 / 2.0; w], vec![w as f32 - 2.0; w]], mean: 0.0, frozen_land: 0.0, hottest_land: 0.0 }
    }

    #[test]
    fn the_wind_blows_from_cold_to_hot_turned_right_in_the_north_and_left_in_the_south() {
        let w = 64;
        let temperature = banded(w);
        // Not turned: straight toward the equator (south in the north half, north in the south).
        let straight = make(&temperature, &WindSettings { blur: 0.0, turn: 0.0, even: 0.0 }, 128.0, &AtomicBool::new(false)).unwrap();
        let (north, south) = (straight.way[16 * w + 30], straight.way[48 * w + 30]);
        assert!(north[1] > 0.99 && south[1] < -0.99, "{north:?} {south:?}");
        // Turned 90°: blowing south turned right is west; blowing north turned left is also west
        // (mirrored halves, like trade winds).
        let turned = make(&temperature, &WindSettings { blur: 0.0, turn: 90.0, even: 0.0 }, 128.0, &AtomicBool::new(false)).unwrap();
        let (north, south) = (turned.way[16 * w + 30], turned.way[48 * w + 30]);
        assert!(north[0] < -0.99 && south[0] < -0.99, "{north:?} {south:?}");
    }

    #[test]
    fn even_makes_the_strength_the_same_everywhere() {
        let temperature = banded(32);
        let even = make(&temperature, &WindSettings { blur: 0.0, turn: 0.0, even: 1.0 }, 128.0, &AtomicBool::new(false)).unwrap();
        assert!(even.strength.iter().all(|&s| (s - 1.0).abs() < 1e-6));
    }
}

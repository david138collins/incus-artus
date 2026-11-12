//! Warp — a filter layer that bends the layer below it (domain warping).
//!
//! A warp layer is a Perlin noise field of its own, but it isn't mixed in like the others. Instead,
//! its values say how far, and which way, to **move where the layer below is read from**. Each pixel
//! shows what the layer below has a little way off, so straight things meander, round things swirl,
//! and features stretch along a flow. Nothing is added or taken away: the same shapes, bent.
//!
//! **How the warp's own values become a push** (the *mode*):
//! - **swirl:** the value turns the push round like a compass needle (0 = east, ½ = all the way
//!   round once, with *turns* at 1). Neighbouring pixels push almost the same way, so the picture
//!   below is dragged in smooth, curling sweeps.
//! - **push:** every pixel pushes the same way (*direction*), forwards where the value is high and
//!   backwards where it's low. Shears the picture into long streaks along that direction.
//! - **flow:** pushes along the warp's own contour lines (as water would run round a hill rather
//!   than down it). Features below are combed out along the warp's shapes.
//!
//! The *amount* is how far the push reaches, in blocks. By default a warp bends only the layer just
//! below it, before that layer mixes in; *everything below* bends the whole stack beneath it instead.
//! *Opacity* fades between unbent (0%) and fully bent (100%).

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarpMode {
    #[default]
    Swirl,
    Push,
    Flow,
}

impl WarpMode {
    pub const ALL: [WarpMode; 3] = [WarpMode::Swirl, WarpMode::Push, WarpMode::Flow];

    pub fn name(self) -> &'static str {
        match self {
            WarpMode::Swirl => "swirl",
            WarpMode::Push => "push",
            WarpMode::Flow => "flow",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            WarpMode::Swirl => "The warp's value turns the push round like a compass needle.\nDrags what's below into smooth, curling sweeps.",
            WarpMode::Push => "Everything pushes one way: forwards where the warp is bright, backwards where it's dark.\nShears what's below into long streaks along that direction.",
            WarpMode::Flow => "Pushes along the warp's contour lines, the way water runs round a hill.\nCombs what's below out along the warp's shapes.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WarpSettings {
    pub mode: WarpMode,
    /// How far the push reaches, in blocks.
    pub amount: f32,
    /// Swirl: how many times round the compass from the darkest value to the brightest.
    pub turns: f32,
    /// Push: which way, in degrees (0 = east, 90 = north). Swirl: where the compass starts.
    pub direction: f32,
    /// Bend everything below, not just the layer just below.
    pub everything_below: bool,
}

impl Default for WarpSettings {
    fn default() -> Self {
        Self { mode: WarpMode::Swirl, amount: 3_000.0, turns: 1.0, direction: 0.0, everything_below: false }
    }
}

/// Bends `values` (with `covered`: which pixels have anything) by the warp layer's values `by`, all
/// fields `width` pixels across. Returns the bent values and coverage, faded by `opacity`.
pub fn bend(values: &[f32], covered: &[bool], by: &[f32], width: usize, blocks_per_pixel: f32, settings: &WarpSettings, opacity: f32) -> (Vec<f32>, Vec<bool>) {
    let reach = settings.amount / blocks_per_pixel.max(1e-6);
    // The field's rows run south, so north is −y.
    let (sin, cos) = settings.direction.to_radians().sin_cos();
    let at = |col: usize, row: usize| by[row.min(width - 1) * width + col.min(width - 1)];
    let push = |col: usize, row: usize| -> (f32, f32) {
        let v = at(col, row);
        match settings.mode {
            WarpMode::Swirl => {
                let angle = settings.direction.to_radians() + v * settings.turns * TAU;
                (angle.cos() * reach, -angle.sin() * reach)
            }
            WarpMode::Push => {
                let along = (v - 0.5) * 2.0 * reach;
                (cos * along, -sin * along)
            }
            WarpMode::Flow => {
                // The slope (which way is uphill), turned a quarter: along the contour.
                let east = at(col + 1, row) - at(col.saturating_sub(1), row);
                let south = at(col, row + 1) - at(col, row.saturating_sub(1));
                let length = east.hypot(south);
                if length < 1e-6 { (0.0, 0.0) } else { (-south / length * reach, east / length * reach) }
            }
        }
    };
    let opacity = opacity.clamp(0.0, 1.0);
    let last = (width - 1) as f32;
    let mut bent = vec![0.0; values.len()];
    let mut bent_covered = vec![false; values.len()];
    for row in 0..width {
        for col in 0..width {
            let i = row * width + col;
            let (dx, dy) = push(col, row);
            // Read the layer below a little way off, blending the four pixels round that spot.
            let (x, y) = ((col as f32 + dx).clamp(0.0, last), (row as f32 + dy).clamp(0.0, last));
            let (x0, y0) = (x.floor() as usize, y.floor() as usize);
            let (x1, y1) = ((x0 + 1).min(width - 1), (y0 + 1).min(width - 1));
            let (fx, fy) = (x - x0 as f32, y - y0 as f32);
            let near = (y.round() as usize) * width + x.round() as usize;
            let top = values[y0 * width + x0] + (values[y0 * width + x1] - values[y0 * width + x0]) * fx;
            let bottom = values[y1 * width + x0] + (values[y1 * width + x1] - values[y1 * width + x0]) * fx;
            let read = top + (bottom - top) * fy;
            bent_covered[i] = covered[near];
            bent[i] = if covered[i] { values[i] + (read - values[i]) * opacity } else { read };
        }
    }
    (bent, bent_covered)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field that rises steadily from west (0) to east (1).
    fn ramp(width: usize) -> Vec<f32> {
        (0..width * width).map(|i| (i % width) as f32 / (width - 1) as f32).collect()
    }

    #[test]
    fn no_amount_leaves_it_alone() {
        let width = 16;
        let (values, covered) = (ramp(width), vec![true; width * width]);
        let by: Vec<f32> = (0..width * width).map(|i| (i as f32 * 0.37).fract()).collect();
        let settings = WarpSettings { amount: 0.0, ..WarpSettings::default() };
        let (bent, _) = bend(&values, &covered, &by, width, 100.0, &settings, 1.0);
        assert!(bent.iter().zip(&values).all(|(a, b)| (a - b).abs() < 1e-6));
    }

    #[test]
    fn a_push_east_reads_from_further_east() {
        let width = 32;
        let (values, covered) = (ramp(width), vec![true; width * width]);
        // The warp at full brightness: push the whole way, east, 4 pixels (400 blocks).
        let by = vec![1.0; width * width];
        let settings = WarpSettings { mode: WarpMode::Push, amount: 400.0, direction: 0.0, ..WarpSettings::default() };
        let (bent, _) = bend(&values, &covered, &by, width, 100.0, &settings, 1.0);
        let i = 10 * width + 10;
        assert!((bent[i] - values[i + 4]).abs() < 1e-5, "{} vs {}", bent[i], values[i + 4]);
        // Half opacity: halfway between.
        let (half, _) = bend(&values, &covered, &by, width, 100.0, &settings, 0.5);
        assert!((half[i] - (values[i] + values[i + 4]) / 2.0).abs() < 1e-5);
    }
}

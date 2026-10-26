//! Terrain lab overlays — the compiler's maps drawn over the map and the close-up, one switch each,
//! so the shapes hidden in them can be seen: where the plates are, how crushed or stretched the rock
//! is, where mountains rise and the crest line of each range, bands of distance from the coast, and
//! which way the rock was squeezed. These are the clean areas and lines ("modalities") the
//! scatterers will later be driven by.
//!
//! Everything but the squeeze direction is painted into one picture, a map pixel (32 blocks) per
//! picture pixel, on all the computer's cores in the background. The squeeze direction is drawn as
//! short strokes on the screen, so it stays readable at any zoom.

use super::compiler::{Bake, blur};
use super::runner::{Job, Poll, every_pixel};
use crate::world::{CompiledMap, Spline};
use bevy_egui::egui::Color32;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Which overlays are switched on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    pub plates: bool,
    pub squeeze: bool,
    pub mountains: bool,
    pub spines: bool,
    pub coast: bool,
    pub direction: bool,
}

impl Shown {
    /// Any of the overlays painted into the picture (the squeeze direction isn't).
    fn painted(&self) -> bool {
        self.plates || self.squeeze || self.mountains || self.spines || self.coast
    }
}

/// What a picture was painted from.
#[derive(Clone, PartialEq)]
struct Key {
    maps: usize,
    mountains: Spline,
    shown: Shown,
    band: f32,
}

/// A finished overlay picture, and the values behind it (for the readouts under the mouse).
pub struct Painted {
    key: Key,
    pub width: usize,
    pub pixels: Vec<Color32>,
    /// The mountain share at each pixel (0 to 1), and the strongest squeeze push on the map.
    pub share: Vec<f32>,
    pub strongest_push: f32,
}

pub struct Overlays {
    pub shown: Shown,
    /// How strongly the overlays cover the map (0 to 1).
    pub opacity: f32,
    /// Blocks between the coast-distance lines.
    pub band: f32,
    job: Option<(Key, Job<Painted>)>,
    pub painted: Option<Painted>,
    /// Goes up with each new picture (so the texture knows to repaint).
    pub version: u64,
}

impl Default for Overlays {
    fn default() -> Self {
        Self { shown: Shown::default(), opacity: 0.7, band: 2_000.0, job: None, painted: None, version: 0 }
    }
}

impl Overlays {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    /// Collects a finished picture, and starts the next one when the maps, the mountains curve or
    /// the switches have changed.
    pub fn drive(&mut self, map: &Arc<CompiledMap>, bake: &Arc<Bake>) {
        if let Some((_, job)) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.painted = Some(finished.made);
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting => {}
            }
        }
        if !self.shown.painted() && !self.shown.direction {
            return;
        }
        let key = Key { maps: Arc::as_ptr(&map.maps) as usize, mountains: map.settings.mountains.clone(), shown: self.shown, band: self.band };
        if self.painted.as_ref().is_some_and(|painted| painted.key == key) {
            return;
        }
        if let Some((making, job)) = &self.job {
            // Already making this one; or let a quick one finish (the next starts straight after).
            if *making == key || job.started.elapsed().as_secs_f32() < 0.15 {
                return;
            }
            job.stop();
            self.job = None;
        }
        let (map, bake, making) = (map.clone(), bake.clone(), key.clone());
        self.job = Some((key, Job::start(move |stop| paint(&map, &bake, making, stop))));
    }
}

/// A plate's colour, from its grey in the plates map: hues spread round the colour wheel.
fn plate_colour(grey: u8) -> [u8; 3] {
    let hue = (grey as f32 * 0.618_034).fract() * 6.0;
    let (saturation, value) = (0.55f32, 0.95f32);
    let k = |n: f32| {
        let k = (n + hue) % 6.0;
        value - value * saturation * k.min(4.0 - k).clamp(0.0, 1.0)
    };
    [k(5.0), k(3.0), k(1.0)].map(|c| (c * 255.0) as u8)
}

/// Whether a pixel is on the crest line of a mountain range: higher than its neighbours both ways
/// along the direction the mountain share bends down most sharply (so along a range's spine, not
/// across a flat top).
pub(super) fn on_spine(share: &[f32], width: usize, col: usize, row: usize) -> bool {
    if col == 0 || row == 0 || col + 1 >= width || row + 1 >= width {
        return false;
    }
    let at = |c: usize, r: usize| share[r * width + c];
    let here = at(col, row);
    if here < 0.12 {
        return false;
    }
    let pairs = [
        (at(col - 1, row), at(col + 1, row)),
        (at(col, row - 1), at(col, row + 1)),
        (at(col - 1, row - 1), at(col + 1, row + 1)),
        (at(col + 1, row - 1), at(col - 1, row + 1)),
    ];
    let (a, b) = pairs.into_iter().max_by(|p, q| (2.0 * here - p.0 - p.1).total_cmp(&(2.0 * here - q.0 - q.1))).unwrap();
    2.0 * here - a - b > 1e-4 && here >= a && here >= b
}

/// Paints the overlays switched on in `key`, on all the computer's cores. `None` if `stop` was set.
fn paint(map: &CompiledMap, bake: &Bake, key: Key, stop: &AtomicBool) -> Option<Painted> {
    let m = &map.maps;
    let w = m.width;
    let share = every_pixel(w, stop, |i| map.mountain_share(&map.pixel(i)))?;
    // The spines follow a lightly smoothed share, so single-pixel wobbles in the squeeze map don't
    // break them into dashes.
    let mut smooth_share = share.clone();
    blur(&mut smooth_share, w, 1);
    let highest_skirt = m.highest_skirt.max(1e-6);
    let strongest_push = m.push_strength.iter().copied().fold(0.0, f32::max);
    let (shown, band) = (key.shown, key.band.max(1.0));
    let distance = &m.coast_distance;
    let plates = bake.plate.len() == w * w;

    let pixels = every_pixel(w, stop, |i| {
        let (col, row) = (i % w, i / w);
        let (right, below) = ((col + 1 < w).then_some(i + 1), (row + 1 < w).then_some(i + w));
        // Laid one over another: colour and how much it covers.
        let mut colour = [0.0f32; 3];
        let mut cover = 0.0f32;
        let mut over = |rgb: [u8; 3], amount: f32| {
            let amount = amount.clamp(0.0, 1.0);
            for (c, new) in colour.iter_mut().zip(rgb) {
                *c = new as f32 * amount + *c * (1.0 - amount);
            }
            cover = amount + cover * (1.0 - amount);
        };
        if shown.plates && plates && bake.plate[i] != 0 {
            over(plate_colour(bake.plate[i]), 0.4);
            let edge = [right, below].into_iter().flatten().any(|j| bake.plate[j] != bake.plate[i]);
            if edge {
                over([20, 20, 28], 0.9);
            }
        }
        if shown.squeeze {
            let s = m.squeeze[i];
            if s > 0.0 {
                over([235, 60, 40], s / m.most_crushed.max(1e-6) * 0.9);
            } else if s < 0.0 {
                over([60, 130, 245], s / m.most_stretched.min(-1e-6) * 0.9);
            }
        }
        if shown.mountains {
            if m.skirt[i] > 0.0 {
                over([235, 215, 95], (m.skirt[i] / highest_skirt).sqrt() * 0.45);
            }
            if share[i] > 0.0 {
                over([245, 145, 35], 0.25 + share[i] * 0.65);
            }
        }
        if shown.coast {
            let here = distance[i];
            let next = [right, below].into_iter().flatten().map(|j| distance[j]);
            let mut line = None;
            for there in next {
                if (here >= 0.0) != (there >= 0.0) {
                    line = Some(([80, 235, 255], 1.0));
                    break;
                }
                if (here / band).floor() != (there / band).floor() {
                    line = Some(if here >= 0.0 { ([255, 255, 255], 0.65) } else { ([150, 200, 255], 0.55) });
                }
            }
            if let Some((rgb, amount)) = line {
                over(rgb, amount);
            }
        }
        if shown.spines && on_spine(&smooth_share, w, col, row) {
            over([255, 60, 220], 1.0);
        }
        if cover <= 0.0 {
            return Color32::TRANSPARENT;
        }
        let [r, g, b] = colour.map(|c| (c / cover).min(255.0) as u8);
        Color32::from_rgba_unmultiplied(r, g, b, (cover * 255.0) as u8)
    })?;
    Some(Painted { key, width: w, pixels, share, strongest_push })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ridge_of_mountains_has_a_one_pixel_spine_and_a_flat_top_has_none() {
        // A ridge running north–south down column 10, falling off either side.
        let width = 21;
        let ridge: Vec<f32> = (0..width * width).map(|i| (1.0 - ((i % width) as f32 - 10.0).abs() / 10.0).max(0.0)).collect();
        let spine: Vec<usize> = (1..width - 1).filter(|&col| on_spine(&ridge, width, col, 10)).collect();
        assert_eq!(spine, vec![10]);
        let flat = vec![0.8; width * width];
        assert!(!on_spine(&flat, width, 10, 10));
    }
}

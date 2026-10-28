//! Climate — the Terrain Compiler's stage between the Additives and the World Interpreter: the
//! weather a world is made with, worked out once over the whole map, and the water it leaves.
//! [David, 2026-10-28: "Breaking the water loop" in docs/worldgen.md.]
//!
//! ```text
//! the ground (read only: the Interpreter's own recipe, its fine layers left out)
//!     │
//!     ▼
//! temperature · wind · rain · rivers (and lakes)
//!     │
//!     ▼  river lines, lake surfaces, and the fields Ecology reads
//! World Interpreter (carves what Climate chose)
//! ```
//!
//! **The ground it reads** (step 1, built first so everything after it stands on something checked):
//! the Interpreter's recipe itself, asked for the ground's height at the middle of each climate
//! pixel, never a copy of it. Its fine layers are left out, using the same switches as the terrain
//! lab's Show row: the crags, summit crags, hills and torn coast. What's left is the broad shape of
//! the land (the curves, ridgelines, rise and peaks), which is what weather and water care about;
//! crag noise would make thousands of false pits for water to pool in. Rivers are carved into the
//! real, craggy ground later, at each block.
//!
//! A climate pixel is a whole number of the compiler's pixels across (32 blocks each), so the two
//! line up exactly.

use super::runner::{Job, Poll, every_pixel};
use crate::world::{CompiledMap, Mutes, SEA_LEVEL};
use bevy_egui::egui::Color32;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// The Climate panel's settings (saved in the compiler's presets, under `[climate]`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClimateSettings {
    /// How many of the compiler's pixels across one climate pixel is (4 = 128 blocks).
    pub pixels: usize,
}

impl Default for ClimateSettings {
    fn default() -> Self {
        Self { pixels: 4 }
    }
}

/// The recipe with its fine layers left out (on top of whatever the Show row leaves out): the broad
/// shape of the land.
pub fn broad(map: &CompiledMap) -> CompiledMap {
    let mutes = Mutes { crags: true, summit_crags: true, detail: true, torn_coast: true, ..map.mutes };
    map.with(map.settings.clone(), mutes)
}

/// The broad ground at the middle of each climate pixel.
#[derive(Clone, Debug)]
pub struct Ground {
    /// Climate pixels across (the map is square), and how many of the compiler's pixels each is.
    pub width: usize,
    pub pixels: usize,
    /// The ground's Z at each (rows run south); `None` past the edge of the world.
    pub z: Vec<Option<f32>>,
}

/// Reads the broad ground over the whole map, a climate pixel at a time, on all the cores (`None`
/// if stopped).
pub fn read_ground(map: &CompiledMap, pixels: usize, stop: &AtomicBool) -> Option<Ground> {
    let pixels = pixels.max(1);
    let broad = broad(map);
    let (map_width, blocks) = (map.maps.width, map.maps.blocks_per_pixel);
    let width = map_width.div_ceil(pixels);
    let half = map_width as f32 / 2.0;
    let z = every_pixel(width, stop, |i| {
        // The middle of the climate pixel, in the compiler's pixels, then blocks from the middle.
        let (col, row) = ((i % width) as f32 + 0.5, (i / width) as f32 + 0.5);
        let (x, y) = (col * pixels as f32, row * pixels as f32);
        broad.height((half - y) * blocks, (x - half) * blocks)
    })?;
    Some(Ground { width, pixels, z })
}

impl Ground {
    /// The climate pixel holding the compiler's pixel (col, row).
    pub fn at_map_pixel(&self, col: usize, row: usize) -> Option<f32> {
        let (c, r) = (col / self.pixels, row / self.pixels);
        if c >= self.width || r >= self.width {
            return None;
        }
        self.z[r * self.width + c]
    }

    /// The ground as a picture: sea dark blue, deeper darker; land grey, lighter higher; lit from
    /// the north-west so its shape shows.
    pub fn picture(&self) -> Vec<Color32> {
        let sea = SEA_LEVEL as f32;
        let top = self.z.iter().flatten().fold(sea + 1.0, |most, &z| most.max(z));
        let w = self.width;
        (0..w * w)
            .map(|i| {
                let Some(z) = self.z[i] else { return Color32::TRANSPARENT };
                if z < sea {
                    let deep = ((sea - z) / sea).clamp(0.0, 1.0);
                    return Color32::from_rgb(20, (60.0 - 40.0 * deep) as u8, (120.0 - 70.0 * deep) as u8);
                }
                let (col, row) = (i % w, i / w);
                let at = |c: usize, r: usize| self.z[r * w + c].unwrap_or(z);
                // Rising toward the north-west: lit; away: shaded.
                let rise = at(col.saturating_sub(1), row.saturating_sub(1)) - at((col + 1).min(w - 1), (row + 1).min(w - 1));
                let light = (1.0 - rise / (self.pixels as f32 * 32.0) * 2.0).clamp(0.5, 1.5);
                let grey = (60.0 + 180.0 * ((z - sea) / (top - sea)).clamp(0.0, 1.0)) * light;
                let v = grey.clamp(0.0, 255.0) as u8;
                Color32::from_rgb(v, v, v)
            })
            .collect()
    }
}

/// What the climate was made from: the recipe (its maps and settings, by address) and the settings.
#[derive(Clone, PartialEq)]
struct Key {
    map: usize,
    settings: ClimateSettings,
}

/// The Climate stage's work, made in the background as the recipe or its settings change.
#[derive(Default)]
pub struct Climate {
    job: Option<(Key, Job<Ground>)>,
    made: Option<(Key, Arc<Ground>)>,
    /// Goes up with each new result (so pictures know to repaint).
    pub version: u64,
    /// Whether the broad ground is drawn over the map.
    pub show_ground: bool,
}

impl Climate {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    pub fn ground(&self) -> Option<&Arc<Ground>> {
        self.made.as_ref().map(|(_, ground)| ground)
    }

    /// Collects finished work, and starts again when the recipe (`map`, as the lab makes it) or the
    /// settings changed.
    pub fn drive(&mut self, map: &Arc<CompiledMap>, settings: &ClimateSettings) {
        if let Some((key, job)) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.made = Some((key.clone(), Arc::new(finished.made)));
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting => {}
            }
        }
        let key = Key { map: Arc::as_ptr(map) as usize, settings: settings.clone() };
        if self.made.as_ref().is_some_and(|(made, _)| *made == key) {
            return;
        }
        if let Some((making, job)) = &self.job {
            if *making == key || job.started.elapsed().as_secs_f32() < 0.15 {
                return;
            }
            job.stop();
            self.job = None;
        }
        let (map, pixels) = (map.clone(), settings.pixels);
        self.job = Some((key, Job::start(move |stop| read_ground(&map, pixels, stop))));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{InterpreterSettings, Peaks};

    #[test]
    fn the_broad_ground_is_the_recipe_without_its_fine_layers() {
        // A world with craggy peaks and hills: the broad ground leaves the crags and hills out, and
        // keeps the peaks.
        let z: Vec<f32> = vec![200.0; 32 * 32];
        let mut map = CompiledMap::of_heights(32, 128.0, &z);
        std::sync::Arc::make_mut(&mut map.maps).mountain_points = vec![[2_048.0, 2_048.0]];
        let peaks = Peaks { on: true, height: 300.0, width: 1_500.0, crags: 0.5, coast_fade: 1.0, ..Peaks::default() };
        let map = map.with(InterpreterSettings { peaks, detail: 10.0, ..map.settings.clone() }, Mutes::default());
        let smooth = map.with(map.settings.clone(), Mutes { crags: true, detail: true, ..Mutes::default() });
        let broad = broad(&map);
        for k in 0..50 {
            let (north, east) = (k as f32 * 13.0 - 300.0, k as f32 * 7.0);
            assert_eq!(broad.height(north, east), smooth.height(north, east));
        }
        // The peak stands in it.
        assert!(broad.height(0.0, 0.0).unwrap() > 400.0);
        // Read a climate pixel at a time: every pixel of a map inside the world has ground.
        let ground = read_ground(&map, 4, &AtomicBool::new(false)).unwrap();
        assert_eq!(ground.width, 8);
        // Its middle is the middle of a 4-pixel climate pixel: 256 blocks each way from the peak's point,
        // partway down its slope, and the same as the broad ground read there.
        let middle = ground.at_map_pixel(16, 16).unwrap();
        assert_eq!(Some(middle), broad.height(-256.0, 256.0));
        assert!(middle > 300.0, "{middle}");
        assert_eq!(ground.at_map_pixel(1, 1), ground.z[0]);
    }
}

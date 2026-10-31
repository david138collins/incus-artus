//! Climate — the World Compiler's stage between the Additives and the World Interpreter: the
//! weather a world is made with, worked out once over the whole map, and the water it leaves.
//! [David, 2026-10-28: "Breaking the water loop" in docs/worldgen.md.]
//!
//! ```text
//! the ground (read only: the Interpreter's own recipe, its fine layers left out)
//!     │
//!     ▼
//! rivers (and lakes) · temperature · wind · rain · mineralogy
//!     │
//!     ▼  river lines, lake surfaces, and the fields Ecology reads
//! World Interpreter (carves what Climate chose)
//! ```
//!
//! **Rivers first** [David, 2026-10-31: "they don't mean anything to rivers, and rivers CAN mean
//! something to them"]: they read the ground alone, so the order runs one way. Nothing after them
//! reads them yet; the climate may later (the lake effect, milder air by big water).
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
use super::temperature::{Temperature, TemperatureSettings};
use super::wind::{Wind, WindSettings};
use super::rain::{Rain, RainSettings};
use super::carving::CarvingSettings;
use super::network::{Network, NetworkSettings};
use super::rivers::{Flow, RiverSettings};
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
    pub temperature: TemperatureSettings,
    pub wind: WindSettings,
    pub rain: RainSettings,
    pub rivers: RiverSettings,
    /// Which of the water's paths are rivers (step 6; apart from `rivers`, so moving these doesn't
    /// work the flow out again).
    pub network: NetworkSettings,
    /// The rivers' curves and how they're carved.
    pub carving: CarvingSettings,
}

impl Default for ClimateSettings {
    fn default() -> Self {
        Self { pixels: 4, temperature: TemperatureSettings::default(), wind: WindSettings::default(), rain: RainSettings::default(), rivers: RiverSettings::default(), network: NetworkSettings::default(), carving: CarvingSettings::default() }
    }
}

/// The recipe as the world has it, without the rivers (they're what's being made): the real ground the
/// rivers' water is laid on. The same Arc while the recipe is, so stages keyed on it don't restart.
fn broad_free(map: &Arc<CompiledMap>) -> Arc<CompiledMap> {
    if map.mutes.rivers || map.rivers.is_empty() {
        return map.clone();
    }
    Arc::new(map.with(map.settings.clone(), Mutes { rivers: true, ..map.mutes }))
}

/// The recipe with its fine layers left out (on top of whatever the Show row leaves out): the broad
/// shape of the land.
pub fn broad(map: &CompiledMap) -> CompiledMap {
    let mutes = Mutes { crags: true, summit_crags: true, detail: true, torn_coast: true, rivers: true, ..map.mutes };
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

/// One Climate stage's work: made in the background whenever what it's made from (its key) changes,
/// the last result kept meanwhile.
struct Stage<K, T> {
    job: Option<(K, Job<T>)>,
    made: Option<(K, Arc<T>)>,
}

impl<K, T> Default for Stage<K, T> {
    fn default() -> Self {
        Self { job: None, made: None }
    }
}

impl<K: Clone + PartialEq, T: Send + 'static> Stage<K, T> {
    /// Collects finished work (true if there's a new result).
    fn collect(&mut self) -> bool {
        let Some((key, job)) = &self.job else { return false };
        match job.poll() {
            Poll::Done(finished) => {
                self.made = Some((key.clone(), Arc::new(finished.made)));
                self.job = None;
                true
            }
            Poll::Over => {
                self.job = None;
                false
            }
            Poll::Waiting => false,
        }
    }

    /// Starts making it again if `key` isn't what the result (or the work under way) was made from.
    fn want(&mut self, key: K, work: impl FnOnce(&AtomicBool) -> Option<T> + Send + 'static) {
        if self.made.as_ref().is_some_and(|(made, _)| *made == key) {
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
        self.job = Some((key, Job::start(work)));
    }

    fn result(&self) -> Option<&Arc<T>> {
        self.made.as_ref().map(|(_, made)| made)
    }
}

/// What's drawn over the map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Show {
    #[default]
    Nothing,
    Ground,
    Temperature,
    Wind,
    Rain,
    Water,
    Rivers,
}

/// The Climate stage's work, each part made in the background as what it's made from changes: the
/// water's flow first (over the ground on the map's own grid, and nothing else: not the rain
/// [2026-10-30], nor the temperature [2026-10-31]), its network and the rivers' curves; then the
/// ground on the climate's grid (from the recipe), the temperature (from the ground and its
/// settings), the wind (from the temperature) and the rain (from the wind).
#[derive(Default)]
pub struct Climate {
    ground: Stage<(Same<CompiledMap>, usize), Ground>,
    temperature: Stage<(Same<Ground>, TemperatureSettings), Temperature>,
    wind: Stage<(Same<Temperature>, WindSettings), Wind>,
    rain: Stage<(Same<Wind>, RainSettings), Rain>,
    fine: Stage<Same<CompiledMap>, Ground>,
    flow: Stage<(Same<Ground>, RiverSettings), Flow>,
    network: Stage<(Same<Flow>, NetworkSettings), Network>,
    carving: Stage<(Same<Network>, Same<CompiledMap>, NetworkSettings, CarvingSettings), crate::world::Carving>,
    /// Goes up with each new result (so pictures know to repaint).
    pub version: u64,
    /// What's drawn over the map.
    pub show: Show,
    /// Which of the rain's views is drawn when showing rain.
    pub rain_view: super::rain::View,
}

impl Climate {
    pub fn is_making(&self) -> bool {
        self.ground.job.is_some() || self.temperature.job.is_some() || self.wind.job.is_some() || self.rain.job.is_some() || self.fine.job.is_some() || self.flow.job.is_some() || self.network.job.is_some() || self.carving.job.is_some()
    }

    pub fn ground(&self) -> Option<&Arc<Ground>> {
        self.ground.result()
    }

    pub fn temperature(&self) -> Option<&Arc<Temperature>> {
        self.temperature.result()
    }

    pub fn wind(&self) -> Option<&Arc<Wind>> {
        self.wind.result()
    }

    pub fn rain(&self) -> Option<&Arc<Rain>> {
        self.rain.result()
    }

    pub fn carving(&self) -> Option<&Arc<crate::world::Carving>> {
        self.carving.result()
    }

    pub fn network(&self) -> Option<&Arc<Network>> {
        self.network.result()
    }

    pub fn flow(&self) -> Option<&Arc<Flow>> {
        self.flow.result()
    }

    /// Collects finished work, and starts each part again when what it's made from changed: the
    /// recipe (`map`, as the lab makes it) or the settings.
    pub fn drive(&mut self, map: &Arc<CompiledMap>, settings: &ClimateSettings) {
        let collected = [self.ground.collect(), self.temperature.collect(), self.wind.collect(), self.rain.collect(), self.fine.collect(), self.flow.collect(), self.network.collect(), self.carving.collect()];
        self.version += collected.iter().filter(|&&new| new).count() as u64;
        let (recipe, pixels) = (map.clone(), settings.pixels);
        self.ground.want((Same(map.clone()), pixels), move |stop| read_ground(&recipe, pixels, stop));
        // The ground on the map's own grid, for the water (read alongside the rest).
        let recipe = map.clone();
        self.fine.want(Same(map.clone()), move |stop| read_ground(&recipe, 1, stop));
        // The rivers first: they read the ground alone, so they wait on nothing after them.
        self.drive_rivers(map, settings);
        // Each part waits for the one before it to be up to date.
        if self.ground.job.is_some() {
            return;
        }
        let Some(ground) = self.ground.result().cloned() else { return };
        let (recipe, temperature, from) = (map.clone(), settings.temperature.clone(), ground.clone());
        self.temperature.want((Same(ground.clone()), temperature.clone()), move |stop| super::temperature::make(&recipe, &from, &temperature, stop));
        if self.temperature.job.is_some() {
            return;
        }
        let Some(temperature) = self.temperature.result().cloned() else { return };
        let (wind, cell) = (settings.wind.clone(), ground.pixels as f32 * map.maps.blocks_per_pixel);
        let from = temperature.clone();
        self.wind.want((Same(temperature.clone()), wind.clone()), move |stop| super::wind::make(&from, &wind, cell, stop));
        if self.wind.job.is_some() {
            return;
        }
        // (A new ground or temperature always makes a new wind, so the wind is all rain needs to key on.)
        let Some(wind) = self.wind.result().cloned() else { return };
        let (rain, from) = (settings.rain.clone(), (ground.clone(), temperature.clone()));
        self.rain.want((Same(wind.clone()), rain.clone()), move |stop| super::rain::make(&from.0, &from.1, &wind, &rain, cell, stop));
    }

    /// The water: its flow (from the ground on the map's own grid alone), the river network and the
    /// rivers' curves.
    fn drive_rivers(&mut self, map: &Arc<CompiledMap>, settings: &ClimateSettings) {
        if self.fine.job.is_some() {
            return;
        }
        let Some(fine) = self.fine.result().cloned() else { return };
        // (Snowmelt and slope-foot springs retired [David, 2026-10-31: "cut them"]: they only added to
        // a spring's own water, which decided nothing that showed. Their code stays in `rivers::water`.)
        let rivers = super::rivers::RiverSettings { snowmelt: 0.0, springs: 0.0, ..settings.rivers.clone() };
        let (seed, blocks) = (map.seed, map.maps.blocks_per_pixel);
        let from = fine.clone();
        self.flow.want((Same(fine), rivers.clone()), move |stop| {
            let water = super::rivers::water(&from.z, from.width, &rivers, blocks);
            super::rivers::flow(from.width, &from.z, &water, rivers.lake_depth, seed, stop)
        });
        // Step 6: which of the water's paths are rivers (grown from the mouths up to the springs).
        if self.flow.job.is_some() {
            return;
        }
        let Some(flow) = self.flow.result().cloned() else { return };
        let network = settings.network.clone();
        let from = flow.clone();
        self.network.want((Same(flow.clone()), network.clone()), move |stop| super::network::make(&from, blocks, seed, &network, stop));
        // The rivers' curves, for carving (keyed on the network, which a new flow always remakes).
        if self.network.job.is_some() {
            return;
        }
        let Some(made) = self.network.result().cloned() else { return };
        let (network, carving) = (settings.network.clone(), settings.carving.clone());
        // Its water reads the real ground (the recipe, without the rivers), so it's keyed on that too.
        let real = broad_free(map);
        let key = (Same(made.clone()), Same(real.clone()), network.clone(), carving.clone());
        self.carving.want(key, move |stop| super::carving::make(&made, &flow, &real, blocks, seed, network.step, network.fall_from, &carving, stop));
    }
}

/// What a stage was made from, the very one (held, so it can't be mistaken for a new one made
/// where it was).
#[derive(Clone)]
struct Same<T>(Arc<T>);

impl<T> PartialEq for Same<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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

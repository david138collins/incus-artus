//! Temperature — the first of the Climate stages. [David, 2026-10-28]
//!
//! **Three lines across the world**, west to east: the north ice line, the equator and the south ice
//! line, each squiggled north and south by its own seed. North of the north line and south of the
//! south line it's evenly as cold as the caps; along the equator (and its band, if it has one) it's
//! evenly hot. The two ice lines share one set of controls (how far from their own edge, squiggle,
//! wave length), each squiggling by its own seed.
//!
//! **Between them, a soap film.** The fade from an ice line to the equator isn't read straight down
//! each column (that would only copy both lines' wiggles all the way across, so squiggling them would
//! hardly matter); it's the shape a soap film settles into stretched between the two lines: every
//! spot the average of the spots round it (Laplace's equation). Its contour lines are the
//! temperature bands, and the paths straight down its slope are David's "strings", one from each
//! point on a line to its partner on the next, never crossing. Near a line the bands follow its every
//! wiggle; further out small wiggles die away and big waves carry through, as ripples do from a stone.
//! It's settled by letting every pixel move toward its neighbours' average until nothing moves,
//! coarse first then finer (each finer level starting from the coarser answer), well under a second.
//!
//! Then **the fade's curve** (David's spline) shapes it, **the coast** pulls temperatures toward the
//! world's average (milder by the sea), and the ground **cools with height**. One scale throughout: −1
//! frozen, +1 the hottest.

use super::climate::Ground;
use super::noise::{Fade, perlin};
use crate::world::{CompiledMap, SEA_LEVEL, Spline};
use bevy_egui::egui::Color32;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// One of the three lines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Line {
    /// Where it runs, as a share of the way from the map's north edge (0) to its south edge (1).
    pub at: f32,
    /// How far it squiggles north and south (blocks; 0 = straight), and how long its biggest waves
    /// are (blocks).
    pub squiggle: f32,
    pub wave: f32,
    /// Its own seed (mixed with the world's, so each world still gets its own line).
    pub seed: u32,
}

impl Line {
    fn new(at: f32, seed: u32) -> Self {
        Self { at, squiggle: 3_000.0, wave: 20_000.0, seed }
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new(0.5, 0)
    }
}

/// The Temperature section's settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemperatureSettings {
    /// Both ice lines, one set of controls [David, 2026-10-28]: `at` is how far each is from its own
    /// edge (the north line from the north edge, the south line from the south), and their squiggle
    /// and wave length are shared. Each still squiggles by its own seed (this one, and the next), so
    /// the caps aren't mirror images.
    pub ice: Line,
    pub equator: Line,
    /// How wide the equator's evenly hot band is (blocks; 0 = just the line).
    pub band: f32,
    /// How cold both caps are (alike), and how hot the equator (−1 frozen to +1 hottest).
    pub cold: f32,
    pub hot: f32,
    /// The fade from an ice line (left) to the equator (right): how far from cold to hot (up).
    pub fade: Spline,
    /// Milder by the sea: how much the coast pulls temperatures toward the world's average (0 to 1),
    /// and how far inland that reaches (blocks).
    pub sea_pull: f32,
    pub sea_reach: f32,
    /// How much colder for every 100 blocks above the sea.
    pub cooling: f32,
}

impl Default for TemperatureSettings {
    fn default() -> Self {
        Self {
            ice: Line::new(0.12, 1),
            equator: Line::new(0.5, 3),
            band: 3_000.0,
            cold: -1.0,
            hot: 1.0,
            fade: Spline::default(),
            sea_pull: 0.3,
            sea_reach: 4_000.0,
            cooling: 0.1,
        }
    }
}

/// Below this is frozen, above this the hottest (for the shares the panel shows).
pub const FROZEN: f32 = -0.5;
pub const HOTTEST: f32 = 0.6;

/// The temperature over the map, a climate pixel at a time.
#[derive(Clone, Debug)]
pub struct Temperature {
    pub width: usize,
    /// −1 frozen to +1 hottest; `None` past the edge of the world.
    pub t: Vec<Option<f32>>,
    /// The same at sea level (no cooling with height), over the whole map, past the world's edge
    /// too: what the wind reads, so mountains don't push air outward as if they were cold spots.
    pub at_sea: Vec<f32>,
    /// Where each line crosses each column (climate pixel rows, from the north edge).
    pub lines: [Vec<f32>; 3],
    /// The world's average, and the shares of the land frozen and hottest.
    pub mean: f32,
    pub frozen_land: f32,
    pub hottest_land: f32,
}

/// Where a line crosses the middle of each column (climate pixel rows from the north edge): its
/// place, plus three octaves of slow noise along it.
fn line_rows(line: &Line, world_seed: u64, width: usize, cell: f32) -> Vec<f32> {
    let extent = width as f32 * cell;
    let seed = world_seed ^ (line.seed as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    (0..width)
        .map(|c| {
            let x = (c as f32 + 0.5) * cell / line.wave.max(1.0);
            let wiggle = perlin(seed, 0x11e, Fade::Smooth, x, 0.5) + 0.5 * perlin(seed, 0x11f, Fade::Smooth, x * 2.0, 1.5) + 0.25 * perlin(seed, 0x120, Fade::Smooth, x * 4.0, 2.5);
            (line.at * extent + line.squiggle * wiggle / 1.75) / cell
        })
        .collect()
}

/// The soap film between the lines, `width` pixels across: 0 on the ice lines and past them, 1 on
/// the equator's band, settled in between (`None` if stopped). `lines` are the rows the lines cross
/// each column at, and `band` the band's half width in rows.
pub fn film(width: usize, lines: &[Vec<f32>; 3], band: f32, stop: &AtomicBool) -> Option<Vec<f32>> {
    // Coarse first: halve until small, settle that, then each finer level from the coarser answer.
    let mut sizes = vec![width];
    while *sizes.last().unwrap() > 24 {
        sizes.push(sizes.last().unwrap().div_ceil(2));
    }
    let mut u: Vec<f32> = Vec::new();
    let mut last = 0;
    for (level, &w) in sizes.iter().enumerate().rev() {
        let scale = w as f32 / width as f32;
        // Which pixels are held (an ice cap, 0; the equator band, 1) at this size.
        let held: Vec<Option<f32>> = (0..w * w)
            .map(|i| {
                let (c, r) = (i % w, i / w);
                let column = (((c as f32 + 0.5) / scale) as usize).min(width - 1);
                let row = (r as f32 + 0.5) / scale;
                let (north, equator, south) = (lines[0][column], lines[1][column], lines[2][column]);
                if row <= north || row >= south {
                    Some(0.0)
                } else if (row - equator).abs() <= band.max(0.5 / scale) {
                    Some(1.0)
                } else {
                    None
                }
            })
            .collect();
        // Start from the coarser answer (or halfway, at the coarsest).
        let start: Vec<f32> = if u.is_empty() {
            vec![0.5; w * w]
        } else {
            (0..w * w).map(|i| u[((i / w) * last / w) * last + (i % w) * last / w]).collect()
        };
        u = start;
        for (i, held) in held.iter().enumerate() {
            if let Some(v) = held {
                u[i] = *v;
            }
        }
        // Over-relaxed sweeps (each pixel pushed a little past its neighbours' average, which
        // settles far faster), more at the finest.
        let sweeps = if level == 0 { 160 } else { 80 };
        for _ in 0..sweeps {
            if stop.load(Ordering::Relaxed) {
                return None;
            }
            for r in 0..w {
                for c in 0..w {
                    let i = r * w + c;
                    if held[i].is_some() {
                        continue;
                    }
                    // The map's edges: as if mirrored (nothing flows out of them).
                    let at = |cc: usize, rr: usize| u[rr * w + cc];
                    let average = (at(c.saturating_sub(1), r) + at((c + 1).min(w - 1), r) + at(c, r.saturating_sub(1)) + at(c, (r + 1).min(w - 1))) / 4.0;
                    u[i] += 1.85 * (average - u[i]);
                }
            }
        }
        last = w;
    }
    Some(u.into_iter().map(|v| v.clamp(0.0, 1.0)).collect())
}

/// The temperature over the map from the ground Climate reads (`None` if stopped).
pub fn make(map: &CompiledMap, ground: &Ground, s: &TemperatureSettings, stop: &AtomicBool) -> Option<Temperature> {
    let (w, cell) = (ground.width, ground.pixels as f32 * map.maps.blocks_per_pixel);
    let south = Line { at: 1.0 - s.ice.at, seed: s.ice.seed.wrapping_add(1), ..s.ice.clone() };
    let mut lines = [line_rows(&s.ice, map.seed, w, cell), line_rows(&s.equator, map.seed, w, cell), line_rows(&south, map.seed, w, cell)];
    let band = s.band.max(0.0) / 2.0 / cell;
    // Kept in order, a pixel apart at least, whatever the squiggles do.
    for c in 0..w {
        lines[1][c] = lines[1][c].clamp(band + 1.0, w as f32 - band - 1.0);
        lines[0][c] = lines[0][c].min(lines[1][c] - band - 1.0);
        lines[2][c] = lines[2][c].max(lines[1][c] + band + 1.0);
    }
    let u = film(w, &lines, band, stop)?;
    let fade = s.fade.curve();
    // The fade read on the film, from the caps' cold to the hot.
    let base: Vec<f32> = u.iter().map(|&u| s.cold + (s.hot - s.cold) * fade.at(u).clamp(0.0, 1.0)).collect();
    let inside: Vec<usize> = (0..w * w).filter(|&i| ground.z[i].is_some()).collect();
    let mean = inside.iter().map(|&i| base[i]).sum::<f32>() / inside.len().max(1) as f32;
    let sea = SEA_LEVEL as f32;
    let map_width = map.maps.width;
    // At sea level first (the caps, equator and sea's say: what the wind reads, everywhere on the
    // map), then the ground cooled by its height (what everything else reads).
    let at_sea: Vec<f32> = (0..w * w)
        .map(|i| {
            let (c, r) = (i % w, i / w);
            // The coast distance at the climate pixel's middle (blocks: + inland, − out to sea).
            let (mc, mr) = ((c * ground.pixels + ground.pixels / 2).min(map_width - 1), (r * ground.pixels + ground.pixels / 2).min(map_width - 1));
            let inland = map.maps.coast_distance[mr * map_width + mc];
            let near_sea = if inland <= 0.0 { 1.0 } else { 1.0 - smooth(inland / s.sea_reach.max(1.0)) };
            (base[i] + (mean - base[i]) * s.sea_pull.clamp(0.0, 1.0) * near_sea).clamp(-1.0, 1.0)
        })
        .collect();
    let t: Vec<Option<f32>> = (0..w * w)
        .map(|i| {
            let z = ground.z[i]?;
            let cooled = if z > sea { s.cooling * (z - sea) / 100.0 } else { 0.0 };
            Some((at_sea[i] - cooled).clamp(-1.0, 1.0))
        })
        .collect();
    let land: Vec<f32> = (0..w * w).filter(|&i| ground.z[i].is_some_and(|z| z >= sea)).filter_map(|i| t[i]).collect();
    let share = |keep: &dyn Fn(f32) -> bool| land.iter().filter(|&&t| keep(t)).count() as f32 / land.len().max(1) as f32;
    let (frozen_land, hottest_land) = (share(&|t| t < FROZEN), share(&|t| t > HOTTEST));
    Some(Temperature { width: w, t, at_sea, lines, mean, frozen_land, hottest_land })
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A temperature's colour: blue (frozen) through teal and green to yellow and red (hottest).
pub fn colour(t: f32) -> [u8; 3] {
    const STOPS: [(f32, [f32; 3]); 5] = [(-1.0, [40.0, 70.0, 210.0]), (-0.4, [60.0, 170.0, 200.0]), (0.1, [80.0, 180.0, 90.0]), (0.55, [230.0, 200.0, 60.0]), (1.0, [215.0, 55.0, 40.0])];
    let t = t.clamp(-1.0, 1.0);
    let k = STOPS.windows(2).position(|pair| t <= pair[1].0).unwrap_or(STOPS.len() - 2);
    let ((t0, a), (t1, b)) = (STOPS[k], STOPS[k + 1]);
    let f = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
    [0, 1, 2].map(|j| (a[j] + (b[j] - a[j]) * f) as u8)
}

impl Temperature {
    /// The temperature as a picture, sea dimmed so the coasts show, and the three lines drawn white.
    pub fn picture(&self, ground: &Ground) -> Vec<Color32> {
        let w = self.width;
        let sea = SEA_LEVEL as f32;
        let mut pixels: Vec<Color32> = (0..w * w)
            .map(|i| match (self.t[i], ground.z[i]) {
                (Some(t), Some(z)) => {
                    let [r, g, b] = colour(t);
                    if z < sea { Color32::from_rgb(r / 2, g / 2, b / 2) } else { Color32::from_rgb(r, g, b) }
                }
                _ => Color32::TRANSPARENT,
            })
            .collect();
        for line in &self.lines {
            for (c, &row) in line.iter().enumerate() {
                let r = row.floor();
                if r >= 0.0 && (r as usize) < w {
                    pixels[r as usize * w + c] = Color32::WHITE;
                }
            }
        }
        pixels
    }

    /// The climate pixel holding the compiler's pixel (col, row).
    pub fn at_map_pixel(&self, ground: &Ground, col: usize, row: usize) -> Option<f32> {
        let (c, r) = (col / ground.pixels, row / ground.pixels);
        if c >= self.width || r >= self.width {
            return None;
        }
        self.t[r * self.width + c]
    }
}

/// A temperature's name, for the readouts.
pub fn describe(t: f32) -> &'static str {
    match t {
        t if t < FROZEN => "frozen",
        t if t < -0.1 => "cold",
        t if t < 0.25 => "mild",
        t if t < HOTTEST => "warm",
        _ => "hottest",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_film_is_held_on_the_lines_and_settles_smoothly_between_them() {
        // Straight lines: the film between an ice line and the equator rises evenly, as a straight
        // fade would.
        let w = 64;
        let lines = [vec![10.0; w], vec![32.0; w], vec![54.0; w]];
        let u = film(w, &lines, 0.5, &AtomicBool::new(false)).unwrap();
        let at = |c: usize, r: usize| u[r * w + c];
        assert_eq!(at(20, 5), 0.0);
        assert_eq!(at(20, 32), 1.0);
        assert_eq!(at(20, 60), 0.0);
        let middle = at(20, 21);
        assert!((middle - 0.5).abs() < 0.08, "halfway between the lines: {middle}");
        // Rising all the way from the ice line to the equator, and the same in every column.
        for r in 11..31 {
            assert!(at(20, r + 1) >= at(20, r) - 1e-3, "row {r}");
            assert!((at(5, r) - at(50, r)).abs() < 0.02);
        }
    }

    #[test]
    fn small_wiggles_die_away_from_their_line_and_big_waves_carry_through() {
        // The north line wiggles fast; the film close to it follows, further away it hardly does.
        let w = 128;
        let wiggly: Vec<f32> = (0..w).map(|c| 20.0 + if (c / 4) % 2 == 0 { 4.0 } else { 0.0 }).collect();
        let lines = [wiggly, vec![80.0; w], vec![120.0; w]];
        let u = film(w, &lines, 0.5, &AtomicBool::new(false)).unwrap();
        let spread = |r: usize| {
            let row: Vec<f32> = (20..108).map(|c| u[r * w + c]).collect();
            row.iter().cloned().fold(f32::MIN, f32::max) - row.iter().cloned().fold(f32::MAX, f32::min)
        };
        assert!(spread(26) > spread(60) * 4.0, "near the line {}, far {}", spread(26), spread(60));
    }

    #[test]
    fn colours_run_blue_to_red() {
        let (cold, hot) = (colour(-1.0), colour(1.0));
        assert!(cold[2] > cold[0] && hot[0] > hot[2]);
        assert_eq!(describe(-0.9), "frozen");
        assert_eq!(describe(0.9), "hottest");
    }
}

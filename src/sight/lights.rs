//! Placed light (build 95; `docs/caves.md` section 11, the second kind of light; David 2026-11-09 through
//! Counsel: "yeah, lets put the object in at the same time"). Ported from the old renderer's `placed_light.rs`
//! (tag `before-regroup`), its meshes left behind: here the lit air goes to a table on the card, and the near
//! walk reads the air in front of each face it meets (`march.wgsl`, `placed_light`).
//!
//! - **From each light:** an object with a light (a brazier, later a torch, glow lichen) floods it out from its
//!   spot through cells that aren't solid, round corners and down tunnels, an even step darker each block to
//!   black at its reach. It stops at walls: rock between you and a brazier keeps its light from you.
//! - **In colour:** each cell keeps red, green and blue, 16 levels each (the light's colour, times its
//!   brightness, times how near). Where two lights meet, each colour is the brighter of the two.
//! - **Flickering** on the card, by time, each light on its own beat (`phase`, from where it stands), so a row of
//!   braziers doesn't flicker in step.
//! - **Worked out when it changes:** a light placed or taken, blocks changed near it, or the chunks round it come
//!   or go (unloaded ones count as solid). Whole numbers throughout: a walk of at most `reach` steps (15 at most)
//!   from each light.

use std::collections::{HashMap, HashSet, VecDeque};

/// The farthest a placed light reaches (blocks).
pub const MOST_REACH: i32 = 15;

/// A light's colour, 0 to 15 for each of red, green and blue, packed (red in the low 4 bits).
pub fn pack(rgb: [u32; 3]) -> u16 {
    (rgb[0].min(15) | rgb[1].min(15) << 4 | rgb[2].min(15) << 8) as u16
}

pub fn unpack(packed: u16) -> [u32; 3] {
    [packed as u32 & 15, (packed as u32 >> 4) & 15, (packed as u32 >> 8) & 15]
}

/// The brighter of two packed lights, colour by colour.
pub fn brighter(a: u16, b: u16) -> u16 {
    let (a, b) = (unpack(a), unpack(b));
    pack([a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])])
}

/// A light's own beat for its flicker (0 to 255), from where its object stands.
pub fn phase(object: [i32; 3]) -> u32 {
    let h = (object[0] as u32).wrapping_mul(73_856_093) ^ (object[1] as u32).wrapping_mul(19_349_663) ^ (object[2] as u32).wrapping_mul(83_492_791);
    (h ^ (h >> 13)).wrapping_mul(0x5bd1_e995) >> 24
}

/// A light (0 to 15 each) as it shows now: eased (squared), flickering on its beat by `flicker` (0 to 1) at `time`
/// (seconds); the same as `march.wgsl`'s `placed_light`, for what's lit on the processor's say (an object's model).
pub fn glow(light: [u32; 3], beat: u32, flicker: f32, time: f32) -> [f32; 3] {
    let wave = wave(beat, time);
    light.map(|level| {
        let k = level as f32 / 15.0;
        k * k * (1.0 - flicker * wave)
    })
}

/// A light's flicker on its beat at `time` (seconds): 0 to 1, about a half on the whole, its peaks short.
pub fn wave(beat: u32, time: f32) -> f32 {
    let beat = beat as f32 / 255.0 * std::f32::consts::TAU;
    0.5 + 0.3 * (time * 9.1 + beat).sin() + 0.2 * (time * 23.7 + beat * 3.0).sin()
}

// ── Far glows (build 100; David 2026-11-09: "braziers in a cave at night from a distance were just pixels of green
// that popped in and out as they twinkled, less and less as you got farther away") ──
//
// Past the loaded chunks a light is a pixel or so of its colour (`models.rs` draws it, `march.wgsl`'s `glow_pass`
// says whether the eye sees it). It fades in over the last stretch before the loaded chunks' edge, where its flame
// is still drawn, so it never snaps; it twinkles, showing only while its flicker is high, the bar rising with
// distance, so far off it shows only at its peaks; it gets smaller and dimmer with distance, and the fog takes it.

/// Where a light's far glow starts (blocks from the eye), and where it's whole: the loaded chunks reach 64, its flame
/// drawn to there.
pub const GLOW_FROM: f32 = 48.0;
pub const GLOW_WHOLE: f32 = 64.0;
/// How far past that its twinkle thins to its peaks alone (blocks).
pub const GLOW_THINS: f32 = 1500.0;
/// How big a flame is seen from far off (blocks).
pub const GLOW_SIZE: f32 = 0.75;
/// The most far glows drawn (the nearest kept).
pub const GLOWS_MOST: usize = 1024;

/// A far light's glow now, `distance` off (its colour 0 to 255 each, and brightness 0 to 1, as its data has them):
/// its colour (linear, 0 to 1 each), or none while its twinkle is down. Without flicker (`flicker` 0) it doesn't
/// twinkle.
pub fn far_glow(colour: [u8; 3], brightness: f32, beat: u32, flicker: f32, time: f32, distance: f32) -> Option<[f32; 3]> {
    if distance < GLOW_FROM {
        return None;
    }
    if flicker > 0.0 {
        // Shown while its flicker is over a bar: a third of its swing near, nearly its top far off.
        let far = ((distance - GLOW_WHOLE) / GLOW_THINS).clamp(0.0, 1.0);
        if wave(beat, time) < 0.3 + 0.62 * far {
            return None;
        }
    }
    let fade_in = ((distance - GLOW_FROM) / (GLOW_WHOLE - GLOW_FROM)).clamp(0.0, 1.0);
    let strength = brightness.clamp(0.0, 1.0) * fade_in;
    Some(colour.map(|c| c as f32 / 255.0 * strength))
}

/// A light as an object gives it: which object (its feet, 16ths of a block), the cell its light comes from, its
/// colour at full (0 to 15 each) and its reach (blocks).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightFrom {
    pub object: [i32; 3],
    pub cell: [i32; 3],
    pub colour: [u32; 3],
    pub reach: i32,
}

/// One light flooded: what it came from, and the cells it lights.
struct Source {
    from: LightFrom,
    cells: HashMap<[i32; 3], u16>,
}

/// A light flooded out from `cell` through cells that aren't `solid`: full (its colour) at its cell, an even step
/// darker each block on, none at `reach`.
pub fn flood(solid: &dyn Fn([i32; 3]) -> bool, cell: [i32; 3], colour: [u32; 3], reach: i32) -> HashMap<[i32; 3], u16> {
    let reach = reach.clamp(1, MOST_REACH);
    let mut cells = HashMap::new();
    let mut queue = VecDeque::from([(cell, 0)]);
    let mut seen = HashSet::from([cell]);
    while let Some((at, steps)) = queue.pop_front() {
        let level = colour.map(|full| full * (reach - steps) as u32 / reach as u32);
        if level == [0; 3] {
            continue;
        }
        cells.insert(at, pack(level));
        for step in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]] {
            let next = [at[0] + step[0], at[1] + step[1], at[2] + step[2]];
            if steps + 1 < reach && !solid(next) && seen.insert(next) {
                queue.push_back((next, steps + 1));
            }
        }
    }
    cells
}

/// Every placed light, by its object, and what it lights.
#[derive(Default)]
pub struct PlacedLights {
    sources: HashMap<[i32; 3], Source>,
}

impl PlacedLights {
    /// The lights as the objects give them now: those gone taken away, new or changed ones flooded out, the same
    /// ones kept. Whether anything changed.
    pub fn set(&mut self, lights: &[LightFrom], solid: &dyn Fn([i32; 3]) -> bool) -> bool {
        let before = self.sources.len();
        self.sources.retain(|object, source| lights.iter().any(|light| light.object == *object && *light == source.from));
        let mut changed = self.sources.len() != before;
        for light in lights {
            if !self.sources.contains_key(&light.object) {
                let cells = flood(solid, light.cell, light.colour, light.reach);
                self.sources.insert(light.object, Source { from: *light, cells });
                changed = true;
            }
        }
        changed
    }

    /// The lights reaching any of `spots` (blocks changed there) flooded out again. Whether any changed.
    pub fn reflood_near(&mut self, spots: &[[i32; 3]], solid: &dyn Fn([i32; 3]) -> bool) -> bool {
        let mut changed = false;
        for source in self.sources.values_mut() {
            let (cell, reach) = (source.from.cell, source.from.reach + 1);
            if !spots.iter().any(|spot| (0..3).all(|k| (spot[k] - cell[k]).abs() <= reach)) {
                continue;
            }
            let cells = flood(solid, cell, source.from.colour, source.from.reach);
            if cells != source.cells {
                source.cells = cells;
                changed = true;
            }
        }
        changed
    }

    /// Every light flooded out again (the chunks round them came or went). Whether any changed.
    pub fn reflood_all(&mut self, solid: &dyn Fn([i32; 3]) -> bool) -> bool {
        let mut changed = false;
        for source in self.sources.values_mut() {
            let cells = flood(solid, source.from.cell, source.from.colour, source.from.reach);
            if cells != source.cells {
                source.cells = cells;
                changed = true;
            }
        }
        changed
    }

    /// How many lights there are.
    pub fn count(&self) -> usize {
        self.sources.len()
    }

    /// The light at `cell` (each colour 0 to 15, the brighter of the lights there), and the beat of the brightest of
    /// them; none if it's dark. For what isn't a block's face (an object's model, build 96).
    pub fn at(&self, cell: [i32; 3]) -> Option<([u32; 3], u32)> {
        let mut best: Option<(u16, u32, u32)> = None;
        for (object, source) in &self.sources {
            let Some(&light) = source.cells.get(&cell) else { continue };
            let strength = unpack(light).iter().sum::<u32>();
            best = Some(match best {
                Some((have, most, beat)) => (brighter(have, light), most.max(strength), if strength > most { phase(*object) } else { beat }),
                None => (light, strength, phase(*object)),
            });
        }
        best.map(|(light, _, beat)| (unpack(light), beat))
    }

    /// Every lit cell, with its light as the card reads it: the brighter of all the lights there, colour by colour
    /// (12 bits), and the beat of the brightest of them (8 bits over it).
    pub fn lit(&self) -> Vec<([i32; 3], u32)> {
        let mut cells: HashMap<[i32; 3], (u16, u32, u32)> = HashMap::new();
        for (object, source) in &self.sources {
            let beat = phase(*object);
            for (&at, &light) in &source.cells {
                let strength = unpack(light).iter().sum::<u32>();
                let cell = cells.entry(at).or_insert((0, 0, beat));
                if strength > cell.1 {
                    cell.1 = strength;
                    cell.2 = beat;
                }
                cell.0 = brighter(cell.0, light);
            }
        }
        let mut lit: Vec<([i32; 3], u32)> = cells.into_iter().map(|(at, (light, _, beat))| (at, light as u32 | beat << 12)).collect();
        // Nearest the middle of things first doesn't matter to the card; a steady order keeps the table the same
        // from one put to the next.
        lit.sort_unstable_by_key(|(at, _)| *at);
        lit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A floor at 0 (everything under 1 solid), and a wall at x = 3 from y −10 to 10, up to 20.
    fn floor_and_wall(at: [i32; 3]) -> bool {
        at[2] < 1 || (at[0] == 3 && (-10..=10).contains(&at[1]) && at[2] < 20)
    }

    #[test]
    fn a_light_fades_a_step_a_block_and_stops_at_its_reach() {
        let cells = flood(&floor_and_wall, [0, 0, 1], [15, 15, 15], 15);
        assert_eq!(unpack(cells[&[0, 0, 1]]), [15, 15, 15]);
        assert_eq!(unpack(cells[&[-1, 0, 1]]), [14, 14, 14]);
        assert_eq!(unpack(cells[&[-5, 0, 1]]), [10, 10, 10]);
        assert!(!cells.contains_key(&[-15, 0, 1]), "none at its reach");
        assert!(!cells.contains_key(&[0, 0, 0]), "none in the floor");
    }

    #[test]
    fn a_light_stops_at_a_wall_and_goes_round_it() {
        let cells = flood(&floor_and_wall, [0, 0, 1], [15, 15, 15], 15);
        assert!(!cells.contains_key(&[3, 0, 1]), "not in the wall");
        // Behind the wall: round its end (11 along, 4 across, 11 back) or over its top (19 up) is past the reach.
        assert!(!cells.contains_key(&[4, 0, 1]), "not straight through it");
        // A low wall's far side is lit, as far as the way over it.
        let low_wall = |at: [i32; 3]| at[2] < 1 || (at[0] == 3 && (-10..=10).contains(&at[1]) && at[2] < 3);
        let round = flood(&low_wall, [0, 0, 1], [15, 15, 15], 15);
        // Up to 3 (2 steps), across to x 4 (4 steps), down to 1 (2 steps): 8 steps, level 7.
        assert_eq!(unpack(round[&[4, 0, 1]]), [7, 7, 7], "round over the wall, as far as the way round");
    }

    #[test]
    fn where_lights_meet_each_colour_is_the_brighter() {
        let open = |at: [i32; 3]| at[2] < 1;
        let mut lights = PlacedLights::default();
        let green = LightFrom { object: [0, 0, 16], cell: [0, 0, 1], colour: [0, 15, 0], reach: 8 };
        let red = LightFrom { object: [64, 0, 16], cell: [4, 0, 1], colour: [12, 0, 0], reach: 8 };
        assert!(lights.set(&[green, red], &open));
        let lit: HashMap<[i32; 3], u32> = lights.lit().into_iter().collect();
        // Between them, 2 from each: green 15 × 6/8 = 11, red 12 × 6/8 = 9.
        assert_eq!(unpack(lit[&[2, 0, 1]] as u16 & 0xFFF), [9, 11, 0]);
        // The same lights again: nothing changed. One taken away: changed, and its light gone.
        assert!(!lights.set(&[green, red], &open));
        assert!(lights.set(&[green], &open));
        let lit: HashMap<[i32; 3], u32> = lights.lit().into_iter().collect();
        assert_eq!(unpack(lit[&[2, 0, 1]] as u16 & 0xFFF), [0, 11, 0]);
        assert_eq!(lights.count(), 1);
    }

    #[test]
    fn a_wall_built_near_a_light_floods_it_again() {
        let mut lights = PlacedLights::default();
        let light = LightFrom { object: [0, 0, 16], cell: [0, 0, 1], colour: [15, 15, 15], reach: 8 };
        let solid = |at: [i32; 3]| at[2] < 1;
        lights.set(&[light], &solid);
        assert!(lights.lit().iter().any(|(at, _)| *at == [3, 0, 1]));
        let walled = |at: [i32; 3]| at[2] < 1 || at[0] == 2;
        assert!(!lights.reflood_near(&[[40, 0, 1]], &walled), "a change far off floods nothing again");
        assert!(lights.reflood_near(&[[2, 0, 1]], &walled), "a change within its reach does");
        assert!(!lights.lit().iter().any(|(at, _)| *at == [3, 0, 1]), "the wall keeps its light from beyond it");
    }

    /// How often a far glow shows over `seconds`, sampled finely.
    fn shown(distance: f32, seconds: f32) -> f32 {
        let samples = (seconds * 240.0) as usize;
        let on = (0..samples).filter(|&k| far_glow([60, 255, 90], 1.0, 77, 0.15, k as f32 / 240.0, distance).is_some()).count();
        on as f32 / samples as f32
    }

    #[test]
    fn a_far_glow_twinkles_less_the_farther_it_is() {
        assert!(far_glow([60, 255, 90], 1.0, 77, 0.15, 0.0, 30.0).is_none(), "near, its flame is drawn instead");
        let (near, mid, far) = (shown(80.0, 20.0), shown(700.0, 20.0), shown(3000.0, 20.0));
        eprintln!("Far glows shown: {near:.2} at 80, {mid:.2} at 700, {far:.2} at 3,000");
        assert!(near > 0.6, "near the edge mostly on ({near:.2})");
        assert!(mid < near && far < mid, "less and less farther off ({near:.2}, {mid:.2}, {far:.2})");
        assert!(far > 0.0 && far < 0.1, "far off, only at its peaks ({far:.2})");
        // It fades in over the last stretch before the loaded chunks' edge, and it's steady without flicker.
        let fading = far_glow([60, 255, 90], 1.0, 77, 0.0, 0.0, 56.0).unwrap();
        assert!((fading[1] - 0.5).abs() < 0.01, "half way in at 56 ({fading:?})");
        assert!((0..1000).all(|k| far_glow([60, 255, 90], 1.0, 77, 0.0, k as f32 / 100.0, 2000.0).is_some()), "no flicker, no twinkle");
    }

    #[test]
    fn each_light_has_its_own_beat() {
        let beats: HashSet<u32> = (0..16).map(|k| phase([k * 16, 0, 16])).collect();
        assert!(beats.len() > 10, "neighbouring lights mostly differ: {beats:?}");
        assert!(beats.iter().all(|&b| b < 256));
    }
}

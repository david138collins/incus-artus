//! The recipe in whole numbers (the regroup, phase 2.2): the same rules as `interpreter.rs`, `carving.rs`,
//! mineralogy and the cave carve, worked in the whole-number kit (`crate::whole`), so the processor and
//! the card make the same world bit for bit [David 2026-11-05: "move to whole numbers. otherwise the
//! hologram breaks. as long as the behavior is qualitatively identical in its procession, it doesnt
//! matter if worlds come out different, so long as the rules never changed"].
//!
//! Everything a fraction is still allowed for (the settings, the maps made at world creation) is turned
//! into whole numbers once, here, as a world loads, and those same numbers go to the card. Reading the
//! recipe at a spot is whole numbers only, in the kit's steps, in the order the card's copy does them.
//!
//! Built a piece at a time beside the fractions, which stay until David has compared them.

use crate::whole::{self, Fix, Pos};
use crate::worldgen::Shaping;
use crate::worldgen::noise::{Fade, PerlinSettings, noise_key, whole_perlin_at, whole_perlin_with_slope_at};

/// One stretch of a curve, between two dots: where it runs, and its Hermite pieces made ready.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stretch {
    pub x0: Fix,
    pub x1: Fix,
    pub y0: Fix,
    pub y1: Fix,
    /// How wide it is, and its steepness at each end times that.
    pub h: Fix,
    pub hm0: Fix,
    pub hm1: Fix,
    /// The bow's power (`interpreter::bowed`), if it's bowed.
    pub power: Option<Fix>,
}

/// A curve (`interpreter::Curve`), read in whole numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WholeCurve {
    pub stretches: Vec<Stretch>,
    /// A curve of one dot is that dot's height everywhere.
    pub single: Fix,
}

impl WholeCurve {
    /// From a curve's dots, their steepness and bends (worked out once, from the settings).
    pub fn new(dots: &[(f32, f32)], slopes: &[f32], bends: &[f32]) -> Self {
        let single = dots.first().map_or(Fix::ZERO, |dot| Fix::from_f32(dot.1));
        let stretches = (0..dots.len().saturating_sub(1))
            .map(|k| {
                let ((x0, y0), (x1, y1)) = (dots[k], dots[k + 1]);
                let h = (x1 - x0).max(1e-6);
                let bend = bends.get(k).copied().unwrap_or(0.0).clamp(-1.0, 1.0) * (y1 - y0).signum();
                // 8 to the power −bend, as 2 to the power −3 × bend, in the kit (no chip's own powf).
                let power = (bend != 0.0).then(|| whole::exp2(Fix::from_f32(-3.0 * bend)));
                Stretch { x0: Fix::from_f32(x0), x1: Fix::from_f32(x1), y0: Fix::from_f32(y0), y1: Fix::from_f32(y1), h: Fix::from_f32(h), hm0: Fix::from_f32(h * slopes[k]), hm1: Fix::from_f32(h * slopes[k + 1]), power }
            })
            .collect();
        Self { stretches, single }
    }

    pub fn at(&self, x: Fix) -> Fix {
        let Some(last) = self.stretches.last() else { return self.single };
        let x = x.clamp(self.stretches[0].x0, last.x1);
        let s = self.stretches.iter().find(|s| x <= s.x1).unwrap_or(last);
        let mut t = ((x - s.x0) / s.h).clamp(Fix::ZERO, Fix::ONE);
        if let Some(power) = s.power {
            t = whole::pow(t, power);
        }
        let t2 = t * t;
        let t3 = t2 * t;
        // Hermite: the two dots, and the steepness at each.
        let h00 = Fix(2 * t3.0 - 3 * t2.0 + whole::ONE);
        let h10 = Fix(t3.0 - 2 * t2.0 + t.0);
        let h01 = Fix(3 * t2.0 - 2 * t3.0);
        let h11 = t3 - t2;
        h00 * s.y0 + h10 * s.hm0 + h01 * s.y1 + h11 * s.hm1
    }
}

/// A noise layer's shaping (`Shaping::shape`: contrast, brightness, curve, invert), in whole numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WholeShaping {
    pub contrast: Fix,
    pub brightness: Fix,
    /// The curve's power, if it isn't 1.
    pub power: Option<Fix>,
    pub invert: bool,
}

impl WholeShaping {
    pub fn new(shaping: &Shaping) -> Self {
        let curve = shaping.curve.max(0.01);
        Self { contrast: Fix::from_f32(shaping.contrast), brightness: Fix::from_f32(shaping.brightness), power: (curve != 1.0).then(|| Fix::from_f32(curve)), invert: shaping.invert }
    }

    /// On a value already from 0 to 1.
    pub fn shape(&self, t: Fix) -> Fix {
        let half = Fix(whole::ONE / 2);
        let mut t = ((t - half) * self.contrast + half + self.brightness).clamp(Fix::ZERO, Fix::ONE);
        if let Some(power) = self.power {
            t = whole::pow(t, power);
        }
        if self.invert { Fix::ONE - t } else { t }
    }
}

/// One octave of a noise layer: its key, its feature size, and its share of the total.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Octave {
    pub key: u32,
    pub size: Pos,
    pub weight: Fix,
}

/// A noise layer (`PerlinSettings::raw_in`, and `interpreter::noise_at` with its shaping), read in whole
/// numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct WholeNoise {
    pub offset_north: Pos,
    pub offset_east: Pos,
    /// The stretch: its direction, and one over how much.
    pub cos: Fix,
    pub sin: Fix,
    pub over_stretch: Fix,
    /// Each octave turned this much from the last.
    pub turn_cos: Fix,
    pub turn_sin: Fix,
    pub octaves: Vec<Octave>,
    /// Only this octave (counting from 0), for looking at one alone.
    pub solo: Option<usize>,
    pub fade: Fade,
    pub shaping: WholeShaping,
}

/// An angle in degrees, in the kit's 65536ths of a turn (a division, which a processor rounds exactly).
fn turns(degrees: f32) -> i32 {
    Fix::from_f32(degrees / 360.0).0
}

impl WholeNoise {
    pub fn new(noise: &PerlinSettings, seed: u64) -> Self {
        let octaves = noise.octaves.max(1);
        let (mut strength, mut strengths, mut size) = (1.0f32, 0.0f32, noise.scale.max(1.0));
        let mut made = Vec::new();
        for octave in 1..=octaves {
            made.push((noise_key(seed, octave as u64), size, strength));
            strengths += strength;
            strength *= noise.persistence;
            size /= noise.lacunarity.max(1.01);
        }
        let (angle, turn) = (turns(noise.stretch_angle), turns(noise.octave_turn));
        Self {
            offset_north: Pos::from_f32(noise.offset_north),
            offset_east: Pos::from_f32(noise.offset_east),
            cos: whole::cos(angle),
            sin: whole::sin(angle),
            over_stretch: Fix::from_f32(1.0 / noise.stretch.max(0.01)),
            turn_cos: whole::cos(turn),
            turn_sin: whole::sin(turn),
            octaves: made.into_iter().map(|(key, size, strength)| Octave { key, size: Pos::from_f32(size), weight: Fix::from_f32(strength / strengths) }).collect(),
            solo: (noise.solo_octave >= 1 && noise.solo_octave <= octaves).then(|| noise.solo_octave as usize - 1),
            fade: noise.fade,
            shaping: WholeShaping::new(&noise.output),
        }
    }

    /// The octaves summed at a spot, about −1 to 1 (`PerlinSettings::raw_in`).
    pub fn raw(&self, north: Pos, east: Pos) -> Fix {
        let (x, y) = (east + self.offset_east, north + self.offset_north);
        // Stretch: turned into the stretch's direction, then squashed along it.
        let (mut x, mut y) = ((x.scaled(self.cos) + y.scaled(self.sin)).scaled(self.over_stretch), y.scaled(self.cos) - x.scaled(self.sin));
        let mut total = Fix::ZERO;
        for (k, octave) in self.octaves.iter().enumerate() {
            let value = whole_perlin_at(octave.key, self.fade, x.per(octave.size), y.per(octave.size));
            if self.solo == Some(k) {
                return value;
            }
            total = total + value * octave.weight;
            (x, y) = (x.scaled(self.turn_cos) - y.scaled(self.turn_sin), x.scaled(self.turn_sin) + y.scaled(self.turn_cos));
        }
        total
    }

    /// Shaped (`interpreter::noise_at`), about −1 to 1.
    pub fn at(&self, north: Pos, east: Pos) -> Fix {
        let half = Fix(whole::ONE / 2);
        let shaped = self.shaping.shape(self.raw(north, east) * half + half);
        Fix(shaped.0.wrapping_mul(2)) - Fix::ONE
    }
}

// ── The maps and the recipe's parts (`interpreter.rs`) ──────────────────────────────────────────────

/// The highest Z mountains reach, and the sea's surface, as values.
const HIGHEST: Fix = Fix::from_int(333);
const SEA: Fix = Fix::from_int(crate::world::SEA_LEVEL);

/// A tiny number, where the fractions used 1e-6 to keep from dividing by nothing.
const TINY: Fix = Fix(1);

/// The most crag layers (as `interpreter`).
const MOST_CRAG_LAYERS: usize = 8;

/// How many fixed directions steady ridges are laid out in (as `interpreter`).
const RIDGE_DIRECTIONS: usize = 12;

/// The maps' values at a spot (`interpreter::Here`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WholeHere {
    pub coast: Fix,
    pub height: Fix,
    pub tilt: Fix,
    pub squeeze: Fix,
    pub push_x: Fix,
    pub push_y: Fix,
    pub push_strength: Fix,
    pub coast_distance: Pos,
    pub lift_distance: Pos,
}

/// One peak, worked out once (`interpreter::Peak`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WholePeak {
    pub x: Pos,
    pub y: Pos,
    pub height: Fix,
    pub width: Pos,
    pub wobble: Fix,
    /// Its wobble's noise key.
    pub key: u32,
}

/// A mask (`interpreter::Mask`) in whole numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct WholeMask {
    pub source: crate::world::MaskSource,
    pub invert: bool,
    pub curve: WholeCurve,
    pub reach: Pos,
}

/// A compiled world's maps and recipe, every number whole (`interpreter::CompiledMap`, before the rivers
/// carve it): what the card is handed.
#[derive(Clone, Debug)]
pub struct WholeMap {
    pub width: i32,
    pub blocks: Pos,
    /// Half the map's width, in blocks (the map's middle is the world's).
    pub half: Pos,
    pub coast: Vec<Fix>,
    pub height: Vec<Fix>,
    pub tilt: Vec<Fix>,
    pub squeeze: Vec<Fix>,
    pub push_x: Vec<Fix>,
    pub push_y: Vec<Fix>,
    pub push_strength: Vec<Fix>,
    pub coast_distance: Vec<Pos>,
    pub lift_distance: Vec<Pos>,
    pub water: Fix,
    pub lowest: Fix,
    pub highest: Fix,
    pub most_crushed: Fix,
    pub most_stretched: Fix,
    pub mutes: crate::world::Mutes,
    // The settings, whole.
    pub deepest: Fix,
    pub highlands: Fix,
    pub ceiling: Fix,
    pub grade: Fix,
    pub coast_rise: Fix,
    pub coast_edge_width: Pos,
    pub coast_fall_back: Pos,
    pub ridges: Fix,
    pub ridge_alignment: Fix,
    pub steady_ridges: bool,
    pub ridge_wander: Pos,
    pub ridge_wander_size: Pos,
    pub ridge_spacing: Pos,
    pub ridge_length: Pos,
    pub sharpness: Fix,
    /// Each ridge layer's scale and strength (`interpreter::ridges`), with the strengths' total.
    pub ridge_scales: Vec<Fix>,
    pub ridge_strengths: Vec<Fix>,
    pub ridge_total: Fix,
    /// The ridge noise's keys: for each of the steady directions (1 to 12), the turning pattern (0),
    /// and the wandering one (13), a key per layer.
    pub ridge_keys: Vec<Vec<u32>>,
    pub wander_keys: [u32; 2],
    /// The steady directions' turns (cos, sin).
    pub directions: Vec<(Fix, Fix)>,
    pub detail: Fix,
    pub detail_noise: WholeNoise,
    pub detail_mask: WholeMask,
    pub detail_keeps_coast: bool,
    pub torn_amount: Fix,
    pub torn_inland: Pos,
    pub torn_out_to_sea: Pos,
    pub torn_noise: WholeNoise,
    pub torn_patches: Fix,
    pub torn_patch_size: Pos,
    pub torn_patch_key: u32,
    pub torn_balance: Fix,
    pub lift_height: Fix,
    pub lift_reach: Pos,
    pub lift_pad: Pos,
    pub lift_pad_power: Option<Fix>,
    pub peaks_on: bool,
    /// The peaks switched on at all (the summit crags ask only that).
    pub peaks_switched_on: bool,
    pub peak_height: Fix,
    pub peaks_add: bool,
    pub crags: Fix,
    /// Each crag layer's size (blocks; layers finer than a block are left out) and how much of the
    /// first layer's strength it keeps, with its noise key.
    pub crag_layers: Vec<(Pos, Fix, u32)>,
    pub foot_fade: Fix,
    pub summit: Fix,
    pub summit_noise: WholeNoise,
    pub summit_from: Fix,
    pub summit_full: Fix,
    pub peak_coast_fade: Pos,
    pub peak_coast_power: Option<Fix>,
    pub steepest: Fix,
    pub carve_below: bool,
    pub peak_side: Pos,
    pub peak_across: i32,
    pub peak_cells: Vec<Vec<WholePeak>>,
    // The curves.
    pub land: WholeCurve,
    pub sea: WholeCurve,
    pub mountains: WholeCurve,
    pub lift: WholeCurve,
    pub coast_edge: WholeCurve,
    pub rise: WholeCurve,
    pub summit_fade: WholeCurve,
    pub slope_response: WholeCurve,
}

/// A power that isn't 1, for `pow` (1 is left as it is: no rounding for nothing).
fn power(p: f32) -> Option<Fix> {
    (p != 1.0).then(|| Fix::from_f32(p))
}

/// `pow`, if there's a power.
fn powered(x: Fix, by: Option<Fix>) -> Fix {
    by.map_or(x, |p| whole::pow(x, p))
}

impl WholeMap {
    /// The map's numbers made whole, once, as a world loads.
    pub fn new(map: &crate::world::CompiledMap) -> Self {
        let (m, s) = (&*map.maps, &map.settings);
        let p = &s.peaks;
        let noise_seed = crate::fate::derive_seed(map.seed, "world compiler");
        let fixes = |v: &[f32]| v.iter().map(|&x| Fix::from_f32(x)).collect::<Vec<_>>();
        let places = |v: &[f32]| v.iter().map(|&x| Pos::from_f32(x)).collect::<Vec<_>>();
        // Ridges: each layer 2.1 times finer and `roughness` times as strong.
        let (layers, roughness) = (s.ridge_layers.clamp(1, 8), s.ridge_roughness.clamp(0.0, 1.0));
        let (mut scales, mut strengths, mut total, mut scale, mut strength) = (Vec::new(), Vec::new(), 0.0f32, 1.0f32, 1.0f32);
        for _ in 0..layers {
            scales.push(Fix::from_f32(scale));
            strengths.push(Fix::from_f32(strength));
            total += strength;
            strength *= roughness;
            scale *= 2.1;
        }
        let ridge_seed = |k: u64| noise_seed ^ k.wrapping_mul(0x9e37_79b9);
        let layer_keys = |seed: u64| (0..layers as u64).map(|octave| noise_key(seed, 0xa11 + octave)).collect::<Vec<u32>>();
        let mut ridge_keys: Vec<Vec<u32>> = (0..=RIDGE_DIRECTIONS as u64).map(|k| layer_keys(ridge_seed(k))).collect();
        ridge_keys.push(layer_keys(noise_seed ^ 0x5eed));
        // The steady directions: −¼ turn and on, every 1/24 of a turn.
        let directions = (0..RIDGE_DIRECTIONS)
            .map(|i| {
                let angle = -whole::TURN / 4 + (i as f32 * whole::TURN as f32 / (2 * RIDGE_DIRECTIONS) as f32).round() as i32;
                (whole::cos(angle), whole::sin(angle))
            })
            .collect();
        // Crags: each layer finer and weaker (`interpreter::crag_steps`).
        let (finer_curve, weaker_curve) = (p.finer_by_layer.curve(), p.weaker_by_layer.curve());
        let last = p.crag_layers.clamp(1, MOST_CRAG_LAYERS as u32).saturating_sub(2).max(1) as f32;
        let (mut size, mut kept) = (p.crag_size.max(1.0), 1.0f32);
        let mut crag_layers = Vec::new();
        for layer in 0..p.crag_layers.clamp(1, MOST_CRAG_LAYERS as u32) {
            if size < 1.0 {
                break;
            }
            crag_layers.push((Pos::from_f32(size), Fix::from_f32(kept), noise_key(noise_seed ^ 0xc4a6, layer as u64)));
            let across = (layer as f32 / last).min(1.0);
            let finer = (1.0 + (p.crag_finer - 1.0) * finer_curve.at(across).clamp(0.0, 1.0)).max(1.01);
            let weaker = p.crag_weaker.max(0.0) * weaker_curve.at(across).clamp(0.0, 1.0);
            kept *= weaker;
            size /= finer;
        }
        // The peaks, in cells (`interpreter::PeakCells`), their spreads by the kit's powers of two.
        let extent = m.width as f32 * m.blocks_per_pixel;
        let wobble_most = p.wobble.max(0.0) * whole::exp2(Fix::from_f32(p.wobble_spread.max(0.0))).to_f32();
        let furthest = p.width * whole::exp2(Fix::from_f32(p.width_spread.max(0.0))).to_f32() * (1.0 + wobble_most);
        let side = furthest.max(extent / 256.0).max(1.0);
        let across = if m.mountain_points.is_empty() { 0 } else { (extent / side).ceil().max(1.0) as usize };
        let mut peak_cells = vec![Vec::new(); across * across];
        for &[x, y] in &m.mountain_points {
            let seed = crate::fate::planet_hash(noise_seed, 0x9eac, x.round() as i64, y.round() as i64, 0);
            let unit = |which: u64| (crate::fate::planet_hash(seed, which, 0, 0, 0) >> 40) as f32 / (1u64 << 23) as f32 - 1.0;
            let spread = |by: f32, which: u64| whole::exp2(Fix::from_f32(by * unit(which))).to_f32();
            let height = p.height * spread(p.height_spread, 1);
            let width = (p.width * spread(p.width_spread, 2)).max(1.0);
            let wobble = p.wobble.max(0.0) * spread(p.wobble_spread, 3);
            let (c, r) = ((x / side).floor().clamp(0.0, (across - 1) as f32) as usize, (y / side).floor().clamp(0.0, (across - 1) as f32) as usize);
            peak_cells[r * across + c].push(WholePeak { x: Pos::from_f32(x), y: Pos::from_f32(y), height: Fix::from_f32(height), width: Pos::from_f32(width), wobble: Fix::from_f32(wobble), key: noise_key(seed, 0x0b1e) });
        }
        let t = &s.torn_coast;
        Self {
            width: m.width as i32,
            blocks: Pos::from_f32(m.blocks_per_pixel),
            half: Pos::from_f32(m.width as f32 / 2.0 * m.blocks_per_pixel),
            coast: fixes(&m.coast),
            height: fixes(&m.height),
            tilt: fixes(&m.tilt),
            squeeze: fixes(&m.squeeze),
            push_x: fixes(&m.push_x),
            push_y: fixes(&m.push_y),
            push_strength: fixes(&m.push_strength),
            coast_distance: places(&m.coast_distance),
            lift_distance: places(&m.lift_distance),
            water: Fix::from_f32(m.water),
            lowest: Fix::from_f32(m.lowest),
            highest: Fix::from_f32(m.highest),
            most_crushed: Fix::from_f32(m.most_crushed),
            most_stretched: Fix::from_f32(m.most_stretched),
            mutes: map.mutes,
            deepest: Fix::from_f32(s.deepest),
            highlands: Fix::from_f32(s.highlands),
            ceiling: Fix::from_f32(s.ceiling),
            grade: Fix::from_f32(s.grade),
            coast_rise: Fix::from_f32(s.coast_rise),
            coast_edge_width: Pos::from_f32(s.coast_edge_width.max(1.0)),
            coast_fall_back: Pos::from_f32(s.coast_fall_back),
            ridges: Fix::from_f32(s.ridges.max(0.0)),
            ridge_alignment: Fix::from_f32(s.ridge_alignment.max(0.0) / 0.1),
            steady_ridges: s.steady_ridges,
            ridge_wander: Pos::from_f32(s.ridge_wander),
            ridge_wander_size: Pos::from_f32(s.ridge_wander_size.max(1.0)),
            ridge_spacing: Pos::from_f32(s.ridge_spacing.max(1.0)),
            ridge_length: Pos::from_f32(s.ridge_length.max(1.0)),
            sharpness: Fix::from_f32(s.sharpness.max(0.1)),
            ridge_scales: scales,
            ridge_strengths: strengths,
            ridge_total: Fix::from_f32(total),
            ridge_keys,
            wander_keys: [noise_key(noise_seed, 0x3a1), noise_key(noise_seed, 0x3a2)],
            directions,
            detail: Fix::from_f32(s.detail),
            detail_noise: WholeNoise::new(&s.detail_noise, map.seed ^ 0xde7a11),
            detail_mask: WholeMask { source: s.detail_mask.source, invert: s.detail_mask.invert, curve: s.detail_mask.curve.curve().whole(), reach: Pos::from_f32(s.detail_mask.reach.max(1.0)) },
            detail_keeps_coast: s.detail_keeps_coast,
            torn_amount: Fix::from_f32(t.amount),
            torn_inland: Pos::from_f32(t.inland.max(1.0)),
            torn_out_to_sea: Pos::from_f32(t.out_to_sea.max(1.0)),
            torn_noise: WholeNoise::new(&t.noise, noise_seed ^ 0x70c0a5),
            torn_patches: Fix::from_f32(t.patches),
            torn_patch_size: Pos::from_f32(t.patch_size.max(1.0)),
            torn_patch_key: noise_key(noise_seed, 0x7042),
            torn_balance: Fix::from_f32(t.balance),
            lift_height: Fix::from_f32(s.lift.height),
            lift_reach: Pos::from_f32(s.lift.reach.max(1.0)),
            lift_pad: Pos::from_f32(s.lift.pad.max(1.0)),
            lift_pad_power: power(s.lift.pad_falloff.max(0.01)),
            peaks_on: p.on && p.height > 0.0,
            peaks_switched_on: p.on,
            peak_height: Fix::from_f32(p.height),
            peaks_add: p.add,
            crags: Fix::from_f32(p.crags),
            crag_layers,
            foot_fade: Fix::from_f32(p.foot_fade),
            summit: Fix::from_f32(p.summit),
            summit_noise: WholeNoise::new(&p.summit_noise, noise_seed ^ 0x5a_c4a6),
            summit_from: Fix::from_f32(p.summit_from),
            summit_full: Fix::from_f32(p.summit_full),
            peak_coast_fade: Pos::from_f32(p.coast_fade.max(1.0)),
            peak_coast_power: power(p.coast_falloff.max(0.01)),
            steepest: Fix::from_f32(p.steepest.max(0.01)),
            carve_below: p.carve_below,
            peak_side: Pos::from_f32(side),
            peak_across: across as i32,
            peak_cells,
            land: s.land.curve().whole(),
            sea: s.sea.curve().whole(),
            mountains: s.mountains.curve().whole(),
            lift: s.lift.curve.curve().whole(),
            coast_edge: s.coast_edge.curve().whole(),
            rise: p.rise.curve().whole(),
            summit_fade: p.summit_fade.curve().whole(),
            slope_response: p.slope_response.curve().whole(),
        }
    }

    /// The maps blended to a spot `north` and `east` from the world's middle, and the spot as the recipe
    /// takes it (blocks east and south of the map's north-west corner); `None` past the map's edge.
    pub fn maps_at(&self, north: Pos, east: Pos) -> Option<(WholeHere, Pos, Pos)> {
        // Which pixel, counting pixel centres (columns run east, rows south).
        let half_pixels = Fix(self.width.wrapping_mul(whole::ONE / 2));
        let half_a_pixel = Fix(whole::ONE / 2);
        let col = east.ratio(self.blocks) + half_pixels - half_a_pixel;
        let row = half_pixels - north.ratio(self.blocks) - half_a_pixel;
        let last = Fix::from_int(self.width) - half_a_pixel;
        if col < -half_a_pixel || row < -half_a_pixel || col > last || row > last {
            return None;
        }
        Some((self.blended(col, row), east + self.half, self.half - north))
    }

    fn blended(&self, col: Fix, row: Fix) -> WholeHere {
        let (c0, r0) = (col.floor(), row.floor());
        let (tc, tr) = (Fix(col.0 & 0xFFFF), Fix(row.0 & 0xFFFF));
        let w = self.width;
        let index = |c: i32, r: i32| (r.clamp(0, w - 1) * w + c.clamp(0, w - 1)) as usize;
        let blend = |field: &dyn Fn(usize) -> i32| {
            let mut rows = [0i32; 4];
            for (k, row) in rows.iter_mut().enumerate() {
                let r = r0 - 1 + k as i32;
                *row = catmull_rom([field(index(c0 - 1, r)), field(index(c0, r)), field(index(c0 + 1, r)), field(index(c0 + 2, r))], tc);
            }
            // Held within the four nearest pixels' values: the curve never swings past them.
            let near = [field(index(c0, r0)), field(index(c0 + 1, r0)), field(index(c0, r0 + 1)), field(index(c0 + 1, r0 + 1))];
            let (low, high) = (near.iter().copied().min().unwrap_or(0), near.iter().copied().max().unwrap_or(0));
            catmull_rom(rows, tr).clamp(low, high)
        };
        WholeHere {
            coast: Fix(blend(&|i| self.coast[i].0)),
            height: Fix(blend(&|i| self.height[i].0)),
            tilt: Fix(blend(&|i| self.tilt[i].0)),
            squeeze: Fix(blend(&|i| self.squeeze[i].0)),
            push_x: Fix(blend(&|i| self.push_x[i].0)),
            push_y: Fix(blend(&|i| self.push_y[i].0)),
            push_strength: Fix(blend(&|i| self.push_strength[i].0)),
            coast_distance: Pos(blend(&|i| self.coast_distance[i].0)),
            lift_distance: Pos(blend(&|i| self.lift_distance[i].0)),
        }
    }

    /// How much of the room above the ground mountains take (`interpreter::mountain_share`).
    pub fn mountain_share(&self, here: &WholeHere) -> Fix {
        if here.squeeze <= Fix::ZERO {
            return Fix::ZERO;
        }
        self.mountains.at(here.squeeze / self.most_crushed.max(TINY)).clamp(Fix::ZERO, Fix::ONE)
    }

    /// The ground with mountains and peaks, before the torn coast and detail (`interpreter::parts` and
    /// `put_together`).
    pub fn put_together(&self, here: &WholeHere, x: Pos, y: Pos) -> Fix {
        let steep = (Fix::ONE + self.grade * here.tilt).max(Fix::from_f32(0.15));
        let ground = if here.coast >= Fix::ZERO {
            let t = (here.height - self.water).max(Fix::ZERO) / (self.highest - self.water).max(TINY);
            SEA + (self.highlands - SEA).max(Fix::ZERO) * self.land.at(t).clamp(Fix::ZERO, Fix::ONE) * steep
        } else {
            let t = (self.water - here.height).max(Fix::ZERO) / (self.water - self.lowest).max(TINY);
            (SEA - Fix::ONE) - (SEA - Fix::ONE - self.deepest).max(Fix::ZERO) * self.sea.at(t).clamp(Fix::ZERO, Fix::ONE) * steep
        };
        let ground = ground + self.raised_coast(here) + self.lifted(here);
        let mut mountains = Fix::ZERO;
        let tall = self.mountain_share(here);
        if tall > Fix::ZERO && !self.mutes.mountains {
            let crest = self.ridge_pattern(here, x, y);
            let carved = if self.mutes.ridges { Fix::ONE } else { (Fix::ONE - self.ridges * (Fix::ONE - crest)).max(Fix::ZERO) };
            mountains = (HIGHEST - ground).max(Fix::ZERO) * tall * carved;
        }
        let peaks = self.peaks(here, x, y);
        let below = ground + mountains;
        let mut z = (below + peaks).min(self.ceiling);
        if below >= SEA {
            z = z.max(SEA);
            z = (z + self.summit_crags(here, x, y, z)).clamp(SEA, self.ceiling);
        }
        z = z + self.torn_coast(here, x, y);
        z.clamp(Fix::ONE, self.ceiling.max(Fix::ONE))
    }

    /// A ridge pattern, 0 to 1 (`interpreter::ridges`), with these layers' keys.
    fn ridges(&self, keys: &[u32], u: Fix, v: Fix) -> Fix {
        let mut total = Fix::ZERO;
        for (k, &key) in keys.iter().enumerate() {
            let scale = self.ridge_scales[k];
            let crest = Fix::ONE - whole_perlin_at(key, Fade::Smooth, u * scale, v * scale).abs().min(Fix::ONE);
            total = total + whole::pow(crest, self.sharpness) * self.ridge_strengths[k];
        }
        total / self.ridge_total
    }

    /// The ridges at a spot (`interpreter::ridge_pattern`).
    fn ridge_pattern(&self, here: &WholeHere, x: Pos, y: Pos) -> Fix {
        // The push's direction, from the doubled-angle arrow (halved: a turn's 65536ths).
        let angle = whole::atan2(here.push_y, here.push_x) >> 1;
        let (x, y) = if self.ridge_wander > Pos(0) {
            let nudge = |key| self.ridge_wander.scaled(whole_perlin_at(key, Fade::Smooth, x.per(self.ridge_wander_size), y.per(self.ridge_wander_size)));
            (x + nudge(self.wander_keys[0]), y + nudge(self.wander_keys[1]))
        } else {
            (x, y)
        };
        // The ridges lined up across one direction.
        let lined = |(cos, sin): (Fix, Fix), keys: &[u32]| {
            let across = x.scaled(cos) - y.scaled(sin);
            let along = y.scaled(cos) + x.scaled(sin);
            self.ridges(keys, across.per(self.ridge_spacing), along.per(self.ridge_length))
        };
        let lined_up = if self.steady_ridges {
            // Which two of the fixed directions the push lies between, and how far.
            let k = Fix((angle + whole::TURN / 4).wrapping_mul(2 * RIDGE_DIRECTIONS as i32));
            let first = k.floor().rem_euclid(RIDGE_DIRECTIONS as i32) as usize;
            let t = whole::smooth(Fix(k.0 & 0xFFFF));
            let (a, b) = (first, (first + 1) % RIDGE_DIRECTIONS);
            let (crest_a, crest_b) = (lined(self.directions[a], &self.ridge_keys[a + 1]), lined(self.directions[b], &self.ridge_keys[b + 1]));
            crest_a + (crest_b - crest_a) * t
        } else {
            lined((whole::cos(angle), whole::sin(angle)), &self.ridge_keys[0])
        };
        let wandering = self.ridges(&self.ridge_keys[RIDGE_DIRECTIONS + 1], x.per(self.ridge_spacing), y.per(self.ridge_spacing));
        let clear = whole::smooth(here.push_strength * self.ridge_alignment);
        wandering + (lined_up - wandering) * clear
    }

    /// The peaks at a spot (`interpreter::peaks`).
    fn peaks(&self, here: &WholeHere, x: Pos, y: Pos) -> Fix {
        if !self.peaks_on || here.coast < Fix::ZERO || self.mutes.peaks {
            return Fix::ZERO;
        }
        let ([rise, east, south], room) = self.peak_rise(x, y);
        if room <= Fix::ZERO {
            return Fix::ZERO;
        }
        let mut total = rise;
        if self.crags > Fix::ZERO && !self.mutes.crags {
            total = total + self.crag_field(x, y, room * self.crags, (east - rise, south - rise));
        }
        if !self.carve_below {
            total = total.max(Fix::ZERO);
        }
        let fade = powered(whole::smooth(here.coast_distance.ratio(self.peak_coast_fade)), self.peak_coast_power);
        total * fade
    }

    /// The peaks' rise here, a block east and a block south, and the crags' room (`interpreter::peak_rise`).
    fn peak_rise(&self, x: Pos, y: Pos) -> ([Fix; 3], Fix) {
        let (mut rise, mut room) = ([Fix::ZERO; 3], Fix::ZERO);
        if self.peak_across == 0 {
            return (rise, room);
        }
        let (c, r) = (x.0.div_euclid(self.peak_side.0.max(1)), y.0.div_euclid(self.peak_side.0.max(1)));
        let block = Pos(whole::BLOCK);
        for r in r - 1..=r + 1 {
            for c in c - 1..=c + 1 {
                if !(0..self.peak_across).contains(&c) || !(0..self.peak_across).contains(&r) {
                    continue;
                }
                for peak in &self.peak_cells[(r * self.peak_across + c) as usize] {
                    let (dx, dy) = (x - peak.x, y - peak.y);
                    // Too far for even its widest wobble (and a block and a half): skipped before any noise.
                    let furthest = peak.width + peak.width.scaled(peak.wobble) + Pos(whole::BLOCK * 3 / 2);
                    if dx.hypot(dy) >= furthest {
                        continue;
                    }
                    let mut width = peak.width;
                    if peak.wobble > Fix::ZERO {
                        let size = width.scaled(Fix::from_f32(0.6));
                        width = width + width.scaled(peak.wobble * whole_perlin_at(peak.key, Fade::Smooth, dx.per(size), dy.per(size)));
                    }
                    let width = width.max(block);
                    for (k, (dx, dy)) in [(dx, dy), (dx + block, dy), (dx, dy + block)].into_iter().enumerate() {
                        let t = Fix::ONE - dx.hypot(dy).ratio(width);
                        if t <= Fix::ZERO {
                            continue;
                        }
                        let own = peak.height * self.rise.at(t).clamp(Fix::ZERO, Fix::ONE);
                        rise[k] = if self.peaks_add { rise[k] + own } else { rise[k].max(own) };
                        if k == 0 {
                            let share = if self.foot_fade <= Fix::ZERO { Fix::ONE } else { whole::smooth(t / self.foot_fade) };
                            room = room.max(peak.height * share);
                        }
                    }
                }
            }
        }
        (rise, room)
    }

    /// The crags (`interpreter::crags`), from the rise's slope (blocks up per block across).
    fn crag_field(&self, x: Pos, y: Pos, strength: Fix, slope: (Fix, Fix)) -> Fix {
        let (mut total, mut steep) = (Fix::ZERO, slope);
        for &(size, kept_share, key) in &self.crag_layers {
            let (value, along_x, along_y) = whole_perlin_with_slope_at(key, x.per(size), y.per(size));
            let kept = self.slope_response.at((whole::hypot(steep.0, steep.1) / self.steepest).min(Fix::ONE)).clamp(Fix::ZERO, Fix::ONE);
            let layer = strength * kept_share * kept;
            total = total + layer * value;
            steep = (steep.0 + layer * along_x / size.to_fix(), steep.1 + layer * along_y / size.to_fix());
        }
        total
    }

    /// The summit crags with the ground at `z` so far (`interpreter::summit_crags`).
    fn summit_crags(&self, here: &WholeHere, x: Pos, y: Pos, z: Fix) -> Fix {
        if !self.peaks_switched_on || self.summit == Fix::ZERO || self.mutes.summit_crags || z <= self.summit_from {
            return Fix::ZERO;
        }
        let across = ((z - self.summit_from) / (self.summit_full - self.summit_from).max(Fix::ONE)).min(Fix::ONE);
        let shown = self.summit_fade.at(across).clamp(Fix::ZERO, Fix::ONE);
        if shown <= Fix::ZERO {
            return Fix::ZERO;
        }
        let coast = powered(whole::smooth(here.coast_distance.ratio(self.peak_coast_fade)), self.peak_coast_power);
        self.summit_noise.at(-y, x) * self.summit * shown * coast
    }

    /// The lift (`interpreter::lift`).
    fn lifted(&self, here: &WholeHere) -> Fix {
        if self.lift_height <= Fix::ZERO || here.coast < Fix::ZERO || self.mutes.lift {
            return Fix::ZERO;
        }
        let near = Fix::ONE - here.lift_distance.ratio(self.lift_reach).clamp(Fix::ZERO, Fix::ONE);
        let pad = powered(whole::smooth(here.coast_distance.ratio(self.lift_pad)), self.lift_pad_power);
        self.lift_height * self.lift.at(near).clamp(Fix::ZERO, Fix::ONE) * pad
    }

    /// The edge rise (`interpreter::raised_coast`).
    fn raised_coast(&self, here: &WholeHere) -> Fix {
        if self.coast_rise <= Fix::ZERO || here.coast < Fix::ZERO || here.tilt <= Fix::ZERO || self.mutes.raised_coast {
            return Fix::ZERO;
        }
        let across = here.coast_distance.ratio(self.coast_edge_width).clamp(Fix::ZERO, Fix::ONE);
        let past = here.coast_distance - self.coast_edge_width;
        let falling = if self.coast_fall_back > Pos(0) && past > Pos(0) { Fix::ONE - whole::smooth(past.ratio(self.coast_fall_back)) } else { Fix::ONE };
        self.coast_rise * here.tilt * self.coast_edge.at(across).clamp(Fix::ZERO, Fix::ONE) * falling
    }

    /// The torn coast (`interpreter::torn_coast`).
    fn torn_coast(&self, here: &WholeHere, x: Pos, y: Pos) -> Fix {
        if self.torn_amount == Fix::ZERO || self.mutes.torn_coast {
            return Fix::ZERO;
        }
        let d = here.coast_distance;
        let band = if d >= Pos(0) { Fix::ONE - d.ratio(self.torn_inland) } else { Fix::ONE + d.ratio(self.torn_out_to_sea) };
        if band <= Fix::ZERO {
            return Fix::ZERO;
        }
        let half = Fix(whole::ONE / 2);
        let shown = if self.torn_patches >= Fix::ONE {
            Fix::ONE
        } else if self.torn_patches <= Fix::ZERO {
            Fix::ZERO
        } else {
            let patch = whole_perlin_at(self.torn_patch_key, Fade::Smooth, x.per(self.torn_patch_size), y.per(self.torn_patch_size)) * half + half;
            whole::smooth(Fix((patch - (Fix::ONE - self.torn_patches)).0.wrapping_mul(5)) + half)
        };
        if shown <= Fix::ZERO {
            return Fix::ZERO;
        }
        let push = self.torn_noise.at(-y, x) + self.torn_balance;
        push * self.torn_amount * whole::smooth(band) * shown
    }

    /// How much of the detail shows (`interpreter::mask`), with the ground at `z`.
    fn detail_shown(&self, here: &WholeHere, z: Fix) -> Fix {
        use crate::world::MaskSource;
        let mask = &self.detail_mask;
        let half = Fix(whole::ONE / 2);
        let t = match mask.source {
            MaskSource::Everywhere => return Fix::ONE,
            MaskSource::Coast => half + half * here.coast_distance.ratio(mask.reach).clamp(-Fix::ONE, Fix::ONE),
            MaskSource::Height => z / HIGHEST,
            MaskSource::Mountains => self.mountain_share(here),
            MaskSource::Squeeze if here.squeeze >= Fix::ZERO => half + half * (here.squeeze / self.most_crushed.max(TINY)),
            MaskSource::Squeeze => half - half * (here.squeeze / self.most_stretched.min(-TINY)),
        };
        let t = t.clamp(Fix::ZERO, Fix::ONE);
        mask.curve.at(if mask.invert { Fix::ONE - t } else { t }).clamp(Fix::ZERO, Fix::ONE)
    }

    /// The ground's height (a Z, with its fraction) before the rivers carve it (`interpreter::uncarved`).
    pub fn uncarved(&self, north: Pos, east: Pos) -> Option<Fix> {
        let (here, x, y) = self.maps_at(north, east)?;
        let ground = self.put_together(&here, x, y);
        if self.mutes.detail || self.detail == Fix::ZERO {
            return Some(ground);
        }
        let shown = self.detail_shown(&here, ground);
        let mut detail = if shown > Fix::ZERO { self.detail_noise.at(north, east) * self.detail * shown } else { Fix::ZERO };
        if self.detail_keeps_coast {
            // Held back from the waterline (a block is land from Z 127.5).
            let waterline = SEA - Fix(whole::ONE / 2);
            let nudge = Fix::from_f32(0.01);
            detail = if ground >= waterline { detail.max(waterline + nudge - ground) } else { detail.min(waterline - nudge - ground) };
        }
        Some(ground + detail)
    }
}

/// Catmull–Rom through four evenly spaced values (any one kind, as raw whole numbers), `t` of the way
/// between the middle two (`interpreter::catmull_rom`).
fn catmull_rom(p: [i32; 4], t: Fix) -> i32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let (a, b, c, d) = (p[0], p[1], p[2], p[3]);
    let first = Fix(b.wrapping_mul(2));
    let second = Fix(c.wrapping_sub(a)) * t;
    let third = Fix(2i32.wrapping_mul(a).wrapping_sub(5i32.wrapping_mul(b)).wrapping_add(4i32.wrapping_mul(c)).wrapping_sub(d)) * t2;
    let fourth = Fix(3i32.wrapping_mul(b.wrapping_sub(c)).wrapping_add(d).wrapping_sub(a)) * t3;
    (first + second + third + fourth).0 >> 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Spline;
    use crate::worldgen::noise::NoiseMaths;


    /// A small world with every part of the recipe in it: land and sea, a tilted coast, crushed rock with
    /// a push that turns, mountains' points with peaks on them, the lift, the torn coast and detail.
    fn rich_map() -> crate::world::CompiledMap {
        use crate::world::{CompiledMap, InterpreterSettings, Maps, point_distance};
        let width = 48;
        let plain = CompiledMap::of_heights(width, 64.0, &(0..width * width).map(|i| if i % width < 30 { 160.0 + (i / width) as f32 } else { 110.0 }).collect::<Vec<f32>>());
        let mut maps: Maps = (*plain.maps).clone();
        let cells = width * width;
        maps.tilt = (0..cells).map(|i| ((i / width) as f32 / width as f32) - 0.3).collect();
        maps.squeeze = (0..cells).map(|i| (((i % width) as f32 - 10.0) / 15.0).clamp(-0.5, 1.0)).collect();
        maps.push_x = (0..cells).map(|i| ((i / width) as f32 * 0.13).cos()).collect();
        maps.push_y = (0..cells).map(|i| ((i / width) as f32 * 0.13).sin()).collect();
        maps.push_strength = (0..cells).map(|i| (i % 7) as f32 / 7.0).collect();
        maps.mountain_points = vec![[640.0, 700.0], [900.0, 1500.0], [1300.0, 2200.0]];
        let points: Vec<usize> = maps.mountain_points.iter().map(|p| (p[1] / 64.0) as usize * width + (p[0] / 64.0) as usize).collect();
        maps.lift_distance = point_distance(&points, width, 64.0);
        let mut settings = InterpreterSettings { coast_rise: 20.0, coast_fall_back: 300.0, detail: 4.0, ..InterpreterSettings::default() };
        settings.peaks.on = true;
        settings.peaks.summit = 20.0;
        settings.peaks.summit_from = 180.0;
        settings.lift.height = 60.0;
        settings.ridge_wander = 40.0;
        CompiledMap::new(std::sync::Arc::new(maps), 7, settings)
    }

    /// The recipe in whole numbers makes the same ground as in fractions (the noise is whole numbers in
    /// both): the same rules, rounded differently on the way. Measured 2026-11-05: 98% of columns the same
    /// block, 0.02 of a block apart on average, under a block at worst.
    #[test]
    fn the_whole_number_recipe_makes_the_same_ground() {
        for steady in [true, false] {
            let mut map = rich_map();
            map.settings.steady_ridges = steady;
            let map = crate::world::CompiledMap::new(map.maps.clone(), map.seed, map.settings.clone());
            let whole = WholeMap::new(&map);
            let (mut same, mut count, mut worst, mut total) = (0, 0, 0.0f32, 0.0f32);
            for north in (-1500..1500).step_by(17) {
                for east in (-1500..1500).step_by(13) {
                    let (north, east) = (north as f32 + 0.5, east as f32 + 0.5);
                    let Some(want) = map.height(north, east) else { continue };
                    let got = whole.uncarved(Pos::from_f32(north), Pos::from_f32(east)).expect("inside the map").to_f32();
                    same += (want.round() == got.round()) as usize;
                    count += 1;
                    worst = worst.max((want - got).abs());
                    total += (want - got).abs();
                }
            }
            let mean = total / count as f32;
            eprintln!("steady {steady}: {same} of {count} the same block, {mean:.3} blocks apart on average, {worst:.2} at worst");
            assert!(same * 100 >= count * 95 && mean < 0.05 && worst < 1.5, "steady {steady}: {same} of {count} the same block, {mean:.3} blocks apart on average, {worst:.2} at worst");
        }
    }

    #[test]
    fn a_curve_in_whole_numbers_follows_the_curve() {
        let mut spline = Spline::through(&[(0.0, 0.0), (0.3, 0.6), (0.7, 0.65), (1.0, 1.0)]);
        spline.bends = vec![0.0, 0.5, -0.4];
        let (curve, whole) = (spline.curve(), spline.curve().whole());
        for k in 0..=200 {
            let x = k as f32 / 200.0;
            let (want, got) = (curve.at(x), whole.at(Fix::from_f32(x)).to_f32());
            assert!((want - got).abs() < 0.002, "at {x}: {got} against {want}");
        }
    }

    #[test]
    fn a_noise_layer_in_whole_numbers_follows_the_layer() {
        let mut noise = PerlinSettings { scale: 900.0, octaves: 4, stretch: 2.0, stretch_angle: 30.0, octave_turn: 17.0, offset_north: 50.0, ..PerlinSettings::default() };
        noise.output.contrast = 1.3;
        noise.output.curve = 1.4;
        let whole = WholeNoise::new(&noise, 99);
        let (mut worst, mut total) = (0.0f32, 0.0f32);
        for k in 0..400 {
            // Column middles across a 72,000-block world.
            let (north, east) = ((k * 7919 % 72_000) as f32 - 36_000.0 + 0.5, (k * 104_729 % 72_000) as f32 - 36_000.0 + 0.5);
            let want = noise.output.shape(noise.raw_in(NoiseMaths::WholeNumbers, 99, north, east) * 0.5 + 0.5) * 2.0 - 1.0;
            let got = whole.at(Pos::from_f32(north), Pos::from_f32(east)).to_f32();
            worst = worst.max((want - got).abs());
            total += (want - got).abs();
        }
        // The same field: the same posts and arrows, only the steps rounded differently on the way. Far
        // out, the turns (the sine and cosine tables, good to about 1/50,000) put a spot up to a block or
        // so from where fractions do, which shows where the field is steep.
        assert!(worst < 0.02 && total / 400.0 < 0.002, "the whole-number layer strays {worst} at worst, {} on average, from the fractions'", total / 400.0);
    }
}

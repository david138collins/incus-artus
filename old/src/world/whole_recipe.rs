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

    /// How steep the curve is at `x` (up per across): its Hermite's own slope, through the bow, over the
    /// stretch's width. (For a slope read off the curve, rather than as the difference of two readings,
    /// which a value's 65536ths make step.)
    pub fn steepness(&self, x: Fix) -> Fix {
        let Some(last) = self.stretches.last() else { return Fix::ZERO };
        let x = x.clamp(self.stretches[0].x0, last.x1);
        let s = self.stretches.iter().find(|s| x <= s.x1).unwrap_or(last);
        let mut t = ((x - s.x0) / s.h).clamp(Fix::ZERO, Fix::ONE);
        // The bow: t to the power p, whose own slope is p × t^(p − 1).
        let mut bowed_by = Fix::ONE;
        if let Some(power) = s.power {
            bowed_by = power * whole::pow(t, power - Fix::ONE).min(Fix::from_int(64));
            t = whole::pow(t, power);
        }
        let t2 = t * t;
        // The Hermite pieces' slopes: 6t² − 6t, 3t² − 4t + 1, 6t − 6t², 3t² − 2t.
        let d00 = Fix(6 * t2.0 - 6 * t.0);
        let d10 = Fix(3 * t2.0 - 4 * t.0 + whole::ONE);
        let d01 = Fix(6 * t.0 - 6 * t2.0);
        let d11 = Fix(3 * t2.0 - 2 * t.0);
        (d00 * s.y0 + d10 * s.hm0 + d01 * s.y1 + d11 * s.hm1) * bowed_by / s.h
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

/// A world's recipe in whole numbers, made once as it loads: its map and its rivers and lakes.
#[derive(Debug)]
pub struct WholeWorld {
    pub map: WholeMap,
    pub carving: WholeCarving,
}

impl WholeWorld {
    pub fn new(map: &crate::world::CompiledMap) -> Self {
        Self { map: WholeMap::new(map), carving: WholeCarving::new(&map.rivers) }
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
    /// The world's seed (mineralogy's chooser rolls by it).
    pub seed: u64,
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
    /// The climate the world was made with (empty: none), and how much colder per 100 blocks up.
    pub temperature: Vec<Fix>,
    pub rain: Vec<Fix>,
    pub cooling: Fix,
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
            seed: map.seed,
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
            temperature: if m.temperature.len() == m.width * m.width && m.rain.len() == m.width * m.width { fixes(&m.temperature) } else { Vec::new() },
            rain: if m.temperature.len() == m.width * m.width && m.rain.len() == m.width * m.width { fixes(&m.rain) } else { Vec::new() },
            cooling: Fix::from_f32(m.cooling),
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

    /// The ground with mountains and peaks, before detail (`interpreter::parts` and `put_together`).
    pub fn put_together(&self, here: &WholeHere, x: Pos, y: Pos) -> Fix {
        let (ground, mountains, peaks) = self.parts(here, x, y);
        let below = ground + mountains;
        let mut z = (below + peaks).min(self.ceiling);
        if below >= SEA {
            z = z.max(SEA);
            z = (z + self.summit_crags(here, x, y, z)).clamp(SEA, self.ceiling);
        }
        z = z + self.torn_coast(here, x, y);
        z.clamp(Fix::ONE, self.ceiling.max(Fix::ONE))
    }

    /// The recipe's pieces at a spot: the ground, the mountains on it, and the peaks on both
    /// (`interpreter::parts`).
    pub fn parts(&self, here: &WholeHere, x: Pos, y: Pos) -> (Fix, Fix, Fix) {
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
        (ground, mountains, self.peaks(here, x, y))
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
                    // Measured in values (65536ths of a block), not positions (256ths): the crags read the
                    // rise's slope from these three a block apart, and a 256th's rounding made it jitter
                    // from column to column [David 2026-11-05: "a noise got way coarser"]. (Within the
                    // peak's reach, well inside a value's ±32,768 blocks.)
                    let width = width.max(block).to_fix();
                    let here = whole::hypot(dx.to_fix(), dy.to_fix());
                    let t = Fix::ONE - here / width;
                    if t <= Fix::ZERO {
                        continue;
                    }
                    let own = peak.height * self.rise.at(t).clamp(Fix::ZERO, Fix::ONE);
                    // A block east and a block south: the rise there from its curve's steepness here and
                    // how much nearer or further the peak's point is, rather than read again (two
                    // readings' difference steps by the curve's rounding, and the crags read this slope).
                    let steepness = if self.rise.at(t) > Fix::ZERO && self.rise.at(t) < Fix::ONE { peak.height * self.rise.steepness(t) } else { Fix::ZERO };
                    let beside = |d: (Pos, Pos)| own + steepness * ((here - whole::hypot(d.0.to_fix(), d.1.to_fix())) / width);
                    for (k, value) in [own, beside((dx + block, dy)), beside((dx, dy + block))].into_iter().enumerate() {
                        rise[k] = if self.peaks_add { rise[k] + value } else { rise[k].max(value) };
                    }
                    let share = if self.foot_fade <= Fix::ZERO { Fix::ONE } else { whole::smooth(t / self.foot_fade) };
                    room = room.max(peak.height * share);
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
    pub fn lifted(&self, here: &WholeHere) -> Fix {
        if self.lift_height <= Fix::ZERO || here.coast < Fix::ZERO || self.mutes.lift {
            return Fix::ZERO;
        }
        let near = Fix::ONE - here.lift_distance.ratio(self.lift_reach).clamp(Fix::ZERO, Fix::ONE);
        let pad = powered(whole::smooth(here.coast_distance.ratio(self.lift_pad)), self.lift_pad_power);
        self.lift_height * self.lift.at(near).clamp(Fix::ZERO, Fix::ONE) * pad
    }

    /// The edge rise (`interpreter::raised_coast`).
    pub fn raised_coast(&self, here: &WholeHere) -> Fix {
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

    /// The climate at a spot `z` high (`interpreter::climate`): its temperature, colder the higher above
    /// the sea, and its rain, blended between the maps' pixels; `None` without a climate.
    pub fn climate(&self, north: Pos, east: Pos, z: Fix) -> Option<(Fix, Fix)> {
        if self.temperature.is_empty() {
            return None;
        }
        let half_pixels = Fix(self.width.wrapping_mul(whole::ONE / 2));
        let half_a_pixel = Fix(whole::ONE / 2);
        let col = east.ratio(self.blocks) + half_pixels - half_a_pixel;
        let row = half_pixels - north.ratio(self.blocks) - half_a_pixel;
        let (c0, r0) = (col.floor(), row.floor());
        let (tc, tr) = (Fix(col.0 & 0xFFFF), Fix(row.0 & 0xFFFF));
        let w = self.width;
        let at = |field: &[Fix], c: i32, r: i32| field[(r.clamp(0, w - 1) * w + c.clamp(0, w - 1)) as usize];
        let blend = |field: &[Fix]| {
            let top = at(field, c0, r0) + (at(field, c0 + 1, r0) - at(field, c0, r0)) * tc;
            let bottom = at(field, c0, r0 + 1) + (at(field, c0 + 1, r0 + 1) - at(field, c0, r0 + 1)) * tc;
            top + (bottom - top) * tr
        };
        let above_sea = (z - SEA).max(Fix::ZERO);
        Some((blend(&self.temperature) - self.cooling * above_sea / Fix::from_int(100), blend(&self.rain)))
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

// ── The rivers and lakes (`carving.rs`) ─────────────────────────────────────────────────────────────

/// One point along a river (`carving::Point`), whole.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WholePoint {
    pub north: Pos,
    pub east: Pos,
    pub width: Fix,
    pub depth: Fix,
    pub edge: Fix,
    pub flat: Fix,
    pub bowl: Fix,
    pub level: Fix,
    pub bend: Fix,
    /// At a waterfall's lip, the water level above the fall.
    pub curtain: Option<Fix>,
    pub fade: Fix,
}

/// How banks fade (`carving::Banks`), whole.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WholeBanks {
    pub way: crate::world::carving::FadeWay,
    pub steepest: Fix,
    pub widest: Fix,
    pub terrace: Fix,
    pub compressor: bool,
    pub raised_steepest: Fix,
    pub raised_widest: Fix,
    /// The most times its fade a bank may widen to, cut or raised.
    pub widest_either: Fix,
}

/// A lake's grid (`carving::Lake`), whole: its water surface and the water beside each dry cell.
#[derive(Clone, Debug, PartialEq)]
pub struct WholeLake {
    pub north: Pos,
    pub west: Pos,
    pub cols: i32,
    pub rows: i32,
    pub cell: Pos,
    pub surface: Vec<Option<Fix>>,
    pub beside: Vec<Option<Fix>>,
}

/// The rivers and lakes, whole (`carving::Carving`): what the card is handed for carving.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WholeCarving {
    pub full: bool,
    pub reaches: Vec<Vec<WholePoint>>,
    pub banks: Option<WholeBanks>,
    pub curtains: bool,
    pub anti_kitty_corner: bool,
    /// The rivers' lookup grid: its cell, cells across, its north-west corner, each cell's pieces.
    pub cell: Pos,
    pub across: i32,
    pub corner: (Pos, Pos),
    pub cells: Vec<Vec<(u32, u32)>>,
    pub lakes: Vec<WholeLake>,
    /// The lakes' lookup grid, likewise (cells across and down).
    pub lake_cell: Pos,
    pub lake_across: (i32, i32),
    pub lake_corner: (Pos, Pos),
    pub lake_cells: Vec<Vec<u32>>,
    /// The rivers on a coarse grid, for how far a spot is from one (the placer's field).
    pub far: WholeFarIndex,
}

/// What the carving does to a column (`carving::Here`), whole.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WholeCarved {
    pub floor: Option<Fix>,
    pub fill: Option<Fix>,
    pub water: Option<i32>,
}

/// A piece as seen from a spot: its distance (blocks), which side (±1), the point there, how far past
/// the piece's start (blocks), and half its water across.
type WholePiece = (Fix, Fix, WholePoint, Fix, Fix);

/// Half a block, a block and two blocks, as values.
const HALF: Fix = Fix(whole::ONE / 2);
const TWO: Fix = Fix::from_int(2);

/// A value rounded to the nearest whole block (halves up, as the fractions' `round` for the heights the
/// world has, all above 0).
fn rounded(x: Fix) -> Fix {
    Fix((x.0 + whole::ONE / 2) & !0xFFFF)
}

fn whole_number(x: Fix) -> i32 {
    rounded(x).floor()
}

/// Whether a height is under the sea, as the world rounds it (`terrain::under_sea`).
fn under_the_sea(height: Fix) -> bool {
    whole_number(height) < crate::world::SEA_LEVEL
}

/// How deep a river's water is at `across` (`carving::depth_across`): whole blocks, at least 1.
fn depth_across(p: &WholePoint, across: Fix) -> Fix {
    let flat = p.flat.clamp(Fix::ZERO, Fix::from_f32(0.95));
    let u = across.abs().min(Fix::ONE);
    let t = if u <= flat { Fix::ONE } else { (Fix::ONE - u) / (Fix::ONE - flat) };
    let at_edge = p.edge.clamp(Fix::ZERO, Fix::ONE) * p.depth;
    rounded(at_edge + (p.depth - at_edge) * whole::pow(t, p.bowl.max(Fix::from_f32(0.05)))).max(Fix::ONE)
}

/// Where a spot is along a piece from `a` to `b` (`carving::along`): how far from its line (blocks), which
/// side (+1 toward the right looking downstream, as the fractions' sign), and how far along (0 to 1). In
/// blocks as values: a piece listed in a spot's cell is within a few hundred blocks of it.
fn along(a: &WholePoint, b: &WholePoint, north: Pos, east: Pos) -> (Fix, Fix, Fix) {
    let (dn, de) = ((b.north - a.north).to_fix(), (b.east - a.east).to_fix());
    let (mn, me) = ((north - a.north).to_fix(), (east - a.east).to_fix());
    let length = whole::hypot(dn, de);
    let t = if length <= TINY {
        Fix::ZERO
    } else {
        let (un, ue) = (dn / length, de / length);
        ((mn * un + me * ue) / length).clamp(Fix::ZERO, Fix::ONE)
    };
    let (pn, pe) = (dn * t, de * t);
    let side = if length <= TINY { Fix::ZERO } else { (de / length) * mn - (dn / length) * me };
    let distance = whole::hypot(mn - pn, me - pe);
    (distance, if side <= Fix::ZERO { -Fix::ONE } else { Fix::ONE }, t)
}

impl WholeBanks {
    /// A bank's ground (`carving::Banks::ground`).
    fn ground(&self, level: Fix, fade: Fix, beyond: Fix, ground: Fix) -> Option<Fix> {
        use crate::world::carving::FadeWay;
        let way = self.way;
        let shore = if way == FadeWay::LedgeThenSlope { level } else { level - Fix::ONE };
        let rise = ground - shore;
        let fade = fade.max(Fix::ONE);
        let fade = if rise > Fix::ZERO { (rise / self.steepest.max(Fix::from_f32(0.05))).clamp(fade, fade * self.widest.max(Fix::ONE)) } else { fade };
        if beyond >= fade {
            return None;
        }
        let t = (beyond / fade).clamp(Fix::ZERO, Fix::ONE);
        let smooth = whole::smooth(t);
        let terrace = self.terrace.max(Fix::ONE);
        Some(match way {
            FadeWay::Smooth | FadeWay::LedgeThenSlope => shore + rise * smooth,
            FadeWay::Slope => shore + rise * t,
            FadeWay::Terraces if rise > Fix::ZERO => shore + Fix::from_int((rise * t / terrace).floor()) * terrace,
            FadeWay::Terraces => shore + rise * smooth,
        })
    }

    /// The compressor's ground (`carving::Banks::compressed`).
    fn compressed(&self, level: Fix, fade: Fix, beyond: Fix, ground: Fix) -> Option<Fix> {
        use crate::world::carving::FadeWay;
        let way = self.way;
        let shore = level - Fix::ONE;
        let rise = ground - shore;
        let kept = if rise > Fix::ZERO && way == FadeWay::LedgeThenSlope { Fix::ONE.min(rise) } else { Fix::ZERO };
        let rest = rise - kept;
        let peak = if matches!(way, FadeWay::Smooth | FadeWay::LedgeThenSlope) { Fix::from_f32(1.5) } else { Fix::ONE };
        let fade = fade.max(Fix::ONE);
        let least = Fix::from_f32(0.05);
        let fade = if rest > Fix::ZERO {
            (rest * peak / self.steepest.max(least)).clamp(fade, fade * self.widest.max(Fix::ONE))
        } else if rise < Fix::ZERO {
            (-rise * peak / self.raised_steepest.max(least)).clamp(fade, fade * self.raised_widest.max(Fix::ONE))
        } else {
            fade
        };
        let t = (beyond - Fix::ONE).max(Fix::ZERO) / fade;
        if t >= Fix::ONE {
            return None;
        }
        let smooth = whole::smooth(t);
        let terrace = self.terrace.max(Fix::ONE);
        Some(
            shore
                + kept
                + match way {
                    FadeWay::Smooth | FadeWay::LedgeThenSlope => rest * smooth,
                    FadeWay::Slope => rest * t,
                    FadeWay::Terraces if rest > Fix::ZERO => Fix::from_int((rest * t / terrace).floor()) * terrace,
                    FadeWay::Terraces => rest * smooth,
                },
        )
    }
}

impl WholeCarving {
    /// The rivers and lakes made whole, once, as a world loads.
    pub fn new(carving: &crate::world::carving::Carving) -> Self {
        let place = Pos::from_f32;
        let value = Fix::from_f32;
        let b = &carving.banks;
        let reaches = carving
            .reaches
            .iter()
            .map(|reach| {
                reach
                    .points
                    .iter()
                    .map(|p| WholePoint { north: place(p.north), east: place(p.east), width: value(p.width), depth: value(p.depth), edge: value(p.edge), flat: value(p.flat), bowl: value(p.bowl), level: value(p.level), bend: value(p.bend), curtain: p.curtain.is_finite().then(|| value(p.curtain)), fade: value(p.fade) })
                    .collect()
            })
            .collect();
        let (cell, across, corner, cells) = carving.grid();
        let lake_parts = carving.lakes.grid();
        let lakes = carving
            .lakes
            .lakes
            .iter()
            .map(|l| {
                let wet = |v: &[f32]| v.iter().map(|&s| s.is_finite().then(|| value(s))).collect::<Vec<_>>();
                WholeLake { north: place(l.north), west: place(l.west), cols: l.cols as i32, rows: l.rows as i32, cell: place(l.cell), surface: wet(&l.surface), beside: wet(l.beside()) }
            })
            .collect();
        Self {
            full: !carving.is_empty(),
            reaches,
            banks: Some(WholeBanks { way: b.way, steepest: value(b.steepest), widest: value(b.widest), terrace: value(b.terrace), compressor: b.compressor, raised_steepest: value(b.raised_steepest), raised_widest: value(b.raised_widest), widest_either: value(if b.compressor { b.widest.max(b.raised_widest).max(1.0) } else { b.widest.max(1.0) }) }),
            curtains: carving.curtains,
            anti_kitty_corner: carving.anti_kitty_corner,
            cell: place(cell),
            across: across as i32,
            corner: (place(corner.0), place(corner.1)),
            cells: cells.to_vec(),
            lakes,
            lake_cell: place(lake_parts.0),
            lake_across: (lake_parts.1.0 as i32, lake_parts.1.1 as i32),
            lake_corner: (place(lake_parts.2.0), place(lake_parts.2.1)),
            lake_cells: lake_parts.3.to_vec(),
            far: WholeFarIndex::new(carving),
        }
    }

    fn row(&self, north: Pos) -> usize {
        ((self.corner.0 - north).0.div_euclid(self.cell.0.max(1)).max(0) as usize).min((self.across as usize).saturating_sub(1))
    }

    fn col(&self, east: Pos) -> usize {
        ((east - self.corner.1).0.div_euclid(self.cell.0.max(1)).max(0) as usize).min((self.across as usize).saturating_sub(1))
    }

    /// Half a river's water across (`carving::half_across`).
    fn half_across(&self, a: &WholePoint, b: &WholePoint, width: Fix) -> Fix {
        let half = Fix(width.max(Fix::ONE).0 >> 1);
        if !self.anti_kitty_corner {
            return half;
        }
        let (dn, de) = ((b.north - a.north).to_fix(), (b.east - a.east).to_fix());
        let length = whole::hypot(dn, de).max(TINY);
        half.max(Fix(((dn.abs() + de.abs()) / length).0 >> 1))
    }

    /// A point between two, `t` of the way (`carving::nearest_piece`'s blend): its level and curtain the
    /// upstream point's.
    fn blend(a: &WholePoint, b: &WholePoint, t: Fix, bend: Fix) -> WholePoint {
        let mix = |x: Fix, y: Fix| x + (y - x) * t;
        WholePoint { north: a.north + (b.north - a.north).scaled(t), east: a.east + (b.east - a.east).scaled(t), width: mix(a.width, b.width), depth: mix(a.depth, b.depth), edge: mix(a.edge, b.edge), flat: mix(a.flat, b.flat), bowl: mix(a.bowl, b.bowl), level: a.level, bend, curtain: a.curtain, fade: mix(a.fade, b.fade) }
    }

    /// The nearest piece, or the lowest water the spot is in (`carving::nearest_piece`).
    fn nearest_piece(&self, north: Pos, east: Pos) -> Option<WholePiece> {
        if self.cells.is_empty() {
            return None;
        }
        let reach_blocks = Pos(self.cell.0.wrapping_mul(self.across));
        let inside = north <= self.corner.0 && east >= self.corner.1 && north > self.corner.0 - reach_blocks && east < self.corner.1 + reach_blocks;
        if !inside {
            return None;
        }
        type Wet = ((Fix, Fix), WholePiece);
        let mut best: Option<WholePiece> = None;
        let mut wet: Option<Wet> = None;
        let mut stretch: (u32, Option<Wet>) = (u32::MAX, None);
        let lowest = |wet: &mut Option<Wet>, found: Option<Wet>| {
            if let Some(found) = found
                && wet.as_ref().is_none_or(|w| found.0 < w.0)
            {
                *wet = Some(found);
            }
        };
        for &(r, k) in &self.cells[self.row(north) * self.across as usize + self.col(east)] {
            if r != stretch.0 {
                lowest(&mut wet, stretch.1.take());
                stretch.0 = r;
            }
            let points = &self.reaches[r as usize];
            let (a, b) = (&points[k as usize], &points[k as usize + 1]);
            let (distance, side, t) = along(a, b, north, east);
            let nearer = best.as_ref().is_none_or(|(d, ..)| distance < *d);
            let width = a.width + (b.width - a.width) * t;
            let bend = if t < HALF { a.bend } else { b.bend };
            let shift = if whole_number(width) % 2 == 0 { bend * HALF } else { Fix::ZERO };
            let half = self.half_across(a, b, width);
            let offset = distance * side - shift;
            let in_water = offset.abs() <= half && stretch.1.as_ref().is_none_or(|(_, (d, ..))| distance < *d);
            if !nearer && !in_water {
                continue;
            }
            let span = whole::hypot((b.north - a.north).to_fix(), (b.east - a.east).to_fix());
            let point = Self::blend(a, b, t, bend);
            let piece = (distance, side, point, t * span, half);
            if nearer {
                best = Some(piece);
            }
            if in_water {
                let level = rounded(point.level);
                let floor = level - Fix::ONE - depth_across(&point, offset / half);
                stretch.1 = Some(((level, floor), piece));
            }
        }
        lowest(&mut wet, stretch.1.take());
        wet.map(|(_, piece)| piece).or(best)
    }

    /// The compressor's ground at a spot outside the nearest river's water (`carving::compressed_bank`).
    fn compressed_bank(&self, banks: &WholeBanks, north: Pos, east: Pos, ground: Fix) -> Option<Fix> {
        let widest = banks.widest_either;
        let (mut bank, mut hold, mut rim) = (Fix::MAX, Fix::MIN, Fix::MIN);
        let mut stretch: (u32, u32, Fix, Option<(Fix, Fix)>) = (u32::MAX, 0, Fix::MAX, None);
        for &(r, k) in &self.cells[self.row(north) * self.across as usize + self.col(east)] {
            let points = &self.reaches[r as usize];
            let (a, c) = (&points[k as usize], &points[k as usize + 1]);
            let (distance, side, t) = along(a, c, north, east);
            let (width, fade) = (a.width + (c.width - a.width) * t, a.fade + (c.fade - a.fade) * t);
            let half = self.half_across(a, c, width);
            let bend = if t < HALF { a.bend } else { c.bend };
            let shift = if whole_number(width) % 2 == 0 { bend * HALF } else { Fix::ZERO };
            let beyond = (distance * side - shift).abs() - half;
            let past = t * whole::hypot((c.north - a.north).to_fix(), (c.east - a.east).to_fix());
            let level = match a.curtain.map(|top| rounded(top).max(rounded(a.level))) {
                Some(top) if past < TWO => top,
                _ => rounded(a.level),
            };
            if beyond > Fix::ZERO && beyond <= Fix::ONE {
                rim = rim.max(level - Fix::ONE);
            }
            let say = (fade > Fix::ZERO && beyond > Fix::ZERO && beyond <= fade * widest + TWO)
                .then(|| banks.compressed(level, fade, beyond, ground).map(|target| (target, level - Fix::ONE - (beyond - Fix::ONE).max(Fix::ZERO) * banks.steepest.max(Fix::from_f32(0.05)))));
            if r != stretch.0 || k > stretch.1 + 2 {
                if let Some((target, held)) = stretch.3 {
                    (bank, hold) = (bank.min(target), hold.max(held));
                }
                stretch = (r, k, distance, say.flatten());
            } else if distance < stretch.2 {
                stretch = (r, k, distance, say.flatten());
            } else {
                stretch.1 = k;
            }
        }
        if let Some((target, held)) = stretch.3 {
            (bank, hold) = (bank.min(target), hold.max(held));
        }
        (bank < Fix::MAX).then(|| bank.max(hold.min(ground)).max(rim))
    }

    /// The lake water at a spot, and whether it's only beside it (`carving::Lakes::at`).
    fn lake_at(&self, north: Pos, east: Pos) -> Option<(Fix, bool)> {
        if self.lake_cells.is_empty() {
            return None;
        }
        let cell = self.lake_cell.0.max(1);
        let col = (east - self.lake_corner.1).0.div_euclid(cell).clamp(0, self.lake_across.0 - 1);
        let row = (self.lake_corner.0 - north).0.div_euclid(cell).clamp(0, self.lake_across.1 - 1);
        let (mut water, mut beside): (Option<Fix>, Option<Fix>) = (None, None);
        for &k in &self.lake_cells[(row * self.lake_across.0 + col) as usize] {
            let lake = &self.lakes[k as usize];
            let (c, r) = ((east - lake.west).0, (lake.north - north).0);
            if c < 0 || r < 0 {
                continue;
            }
            let (c, r) = (c / lake.cell.0.max(1), r / lake.cell.0.max(1));
            if c >= lake.cols || r >= lake.rows {
                continue;
            }
            let i = (r * lake.cols + c) as usize;
            let higher = |kept: Option<Fix>, new: Option<Fix>| match (kept, new) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            (water, beside) = (higher(water, lake.surface[i]), higher(beside, lake.beside[i]));
        }
        match water {
            Some(surface) => Some((surface, false)),
            None => beside.map(|surface| (surface, true)),
        }
    }

    /// What the rivers and lakes do to the column at a spot whose uncarved ground is `ground`
    /// (`carving::Carving::here`).
    pub fn here(&self, north: Pos, east: Pos, ground: Fix) -> WholeCarved {
        let mut here = WholeCarved::default();
        if !under_the_sea(ground)
            && let Some(banks) = &self.banks
            && let Some((distance, side, point, past, half)) = self.nearest_piece(north, east)
        {
            let shift = if whole_number(point.width) % 2 == 0 { point.bend * HALF } else { Fix::ZERO };
            let offset = distance * side - shift;
            let fall = point.curtain.map(|top| rounded(top).max(rounded(point.level)));
            if offset.abs() <= half {
                let level = rounded(point.level);
                let top = match fall {
                    Some(top) if self.curtains && past < Fix::ONE => top,
                    _ => level,
                };
                here.floor = Some(level - Fix::ONE - depth_across(&point, offset / half));
                here.water = Some(top.floor());
            } else if banks.compressor {
                if let Some(bank) = self.compressed_bank(banks, north, east, ground) {
                    if bank < ground {
                        here.floor = Some(bank);
                    } else {
                        here.fill = Some(bank);
                    }
                }
            } else if point.fade > Fix::ZERO {
                let level = match fall {
                    Some(top) if past < TWO => top,
                    _ => rounded(point.level),
                };
                if let Some(bank) = banks.ground(level, point.fade, offset.abs() - half, ground) {
                    if bank < ground {
                        here.floor = Some(bank);
                    } else {
                        here.fill = Some(bank);
                    }
                }
            }
        }
        if let Some((surface, edge)) = self.lake_at(north, east)
            && surface > SEA
        {
            let top = rounded(surface);
            let cut = here.floor.map_or(ground, |f| ground.min(f));
            let carved = here.fill.map_or(cut, |fill| cut.max(fill));
            if edge {
                if (if here.water.is_none() { carved } else { ground }) < top - Fix::ONE {
                    here.fill = Some(top - Fix::ONE);
                }
            } else if carved < top - HALF {
                here.water = Some(here.water.map_or(top.floor(), |w| w.max(top.floor())));
                here.fill = None;
            }
        }
        here
    }
}

/// A river's water across (`carving::water_across`): its width times its depth times how full its bowl is.
fn water_across(p: &WholePoint) -> Fix {
    let (edge, flat) = (p.edge.clamp(Fix::ZERO, Fix::ONE), p.flat.clamp(Fix::ZERO, Fix::from_f32(0.95)));
    let fullness = flat + (Fix::ONE - flat) * (edge + (Fix::ONE - edge) / (p.bowl.max(Fix::from_f32(0.05)) + Fix::ONE));
    p.width.max(Fix::ONE) * (p.depth * fullness).max(Fix::ONE)
}

/// The far index's cell (blocks), and how far apart the points along a river are thinned to for it
/// (as `carving`).
const FAR_CELL: i32 = 128;
const FAR_THIN: f32 = 6.0;

/// The rivers' pieces on a coarse grid (`carving::FarIndex`), whole: for how far a spot is from a river.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WholeFarIndex {
    pub corner: (Pos, Pos),
    pub across: i32,
    pub down: i32,
    pub widest_half: Fix,
    /// Each cell's pieces: their ends and half their width.
    pub cells: Vec<Vec<((Pos, Pos), (Pos, Pos), Fix)>>,
}

impl WholeFarIndex {
    pub fn new(carving: &crate::world::carving::Carving) -> Self {
        let mut pieces: Vec<((f32, f32), (f32, f32), f32)> = Vec::new();
        for reach in &carving.reaches {
            let Some(mut from) = reach.points.first() else { continue };
            for (k, p) in reach.points.iter().enumerate().skip(1) {
                // (Squared, so a processor's own square root has no say.)
                let (dn, de) = (p.north - from.north, p.east - from.east);
                if dn * dn + de * de >= FAR_THIN * FAR_THIN || k + 1 == reach.points.len() {
                    pieces.push(((from.north, from.east), (p.north, p.east), from.width.max(1.0) / 2.0));
                    from = p;
                }
            }
        }
        if pieces.is_empty() {
            return Self::default();
        }
        let place = Pos::from_f32;
        let whole_pieces: Vec<((Pos, Pos), (Pos, Pos), Fix)> = pieces.iter().map(|&((an, ae), (bn, be), half)| ((place(an), place(ae)), (place(bn), place(be)), Fix::from_f32(half))).collect();
        let (mut low, mut high) = ((Pos(i32::MAX), Pos(i32::MAX)), (Pos(i32::MIN), Pos(i32::MIN)));
        for &(a, b, _) in &whole_pieces {
            for (n, e) in [a, b] {
                (low, high) = ((low.0.min(n), low.1.min(e)), (high.0.max(n), high.1.max(e)));
            }
        }
        let cell = FAR_CELL * whole::BLOCK;
        let corner = (high.0, low.1);
        let (across, down) = ((high.1 - low.1).0 / cell + 1, (high.0 - low.0).0 / cell + 1);
        let mut index = Self { corner, across, down, widest_half: Fix::ZERO, cells: vec![Vec::new(); (across * down) as usize] };
        for piece in whole_pieces {
            let ((an, ae), (bn, be), half) = piece;
            index.widest_half = index.widest_half.max(half);
            let (c0, r0) = index.cell_of(an.max(bn), ae.min(be));
            let (c1, r1) = index.cell_of(an.min(bn), ae.max(be));
            for r in r0.max(0)..=r1.min(down - 1) {
                for c in c0.max(0)..=c1.min(across - 1) {
                    index.cells[(r * across + c) as usize].push(piece);
                }
            }
        }
        index
    }

    fn cell_of(&self, north: Pos, east: Pos) -> (i32, i32) {
        let cell = FAR_CELL * whole::BLOCK;
        ((east - self.corner.1).0.div_euclid(cell), (self.corner.0 - north).0.div_euclid(cell))
    }

    /// How far a spot is from the nearest river's water (blocks), up to `most`.
    pub fn distance(&self, north: Pos, east: Pos, most: Fix) -> Fix {
        if self.cells.is_empty() {
            return most;
        }
        let (c, r) = self.cell_of(north, east);
        let mut best = most;
        let cell = Fix::from_int(FAR_CELL);
        let rings = ((most + self.widest_half) / cell).floor() + 2;
        for k in 0..=rings {
            if Fix::from_int(k - 1) * cell - self.widest_half >= best {
                break;
            }
            for dr in -k..=k {
                let step = if dr.abs() == k { 1 } else { (2 * k).max(1) as usize };
                for dc in (-k..=k).step_by(step) {
                    let (cc, rr) = (c + dc, r + dr);
                    if cc < 0 || rr < 0 || cc >= self.across || rr >= self.down {
                        continue;
                    }
                    for &((an, ae), (bn, be), half) in &self.cells[(rr * self.across + cc) as usize] {
                        let a = WholePoint { north: an, east: ae, width: Fix::ZERO, depth: Fix::ZERO, edge: Fix::ZERO, flat: Fix::ZERO, bowl: Fix::ZERO, level: Fix::ZERO, bend: Fix::ZERO, curtain: None, fade: Fix::ZERO };
                        let b = WholePoint { north: bn, east: be, ..a };
                        let (apart, _, _) = along(&a, &b, north, east);
                        best = best.min((apart - half).max(Fix::ZERO));
                    }
                }
            }
        }
        best
    }
}

impl WholeCarving {
    /// The river nearest a spot: how far its line is, its width there, and about how many blocks of water
    /// it has across (`carving::water_near`).
    pub fn water_near(&self, north: Pos, east: Pos) -> Option<(Fix, Fix, Fix)> {
        self.nearest_piece(north, east).map(|(distance, _, p, _, _)| (distance, p.width, water_across(&p)))
    }
}

impl WholeMap {
    /// The ground's height at a spot, carved, and how high water stands over it there
    /// (`interpreter::ground_and_water`).
    pub fn ground_and_water(&self, carving: &WholeCarving, north: Pos, east: Pos) -> Option<(Fix, Option<i32>)> {
        let ground = self.uncarved(north, east)?;
        if self.mutes.rivers || !carving.full {
            return Some((ground, None));
        }
        let here = carving.here(north, east, ground);
        let mut z = ground;
        if let Some(floor) = here.floor {
            z = z.min(floor);
        }
        if let Some(fill) = here.fill {
            z = z.max(fill);
        }
        Some((z, here.water.filter(|&water| water > whole_number(z) + 1)))
    }
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
        // (In fractions: the tests compare them with whole numbers.)
        let mut settings = InterpreterSettings { coast_rise: 20.0, coast_fall_back: 300.0, detail: 4.0, recipe_maths: crate::worldgen::noise::NoiseMaths::Fractions, ..InterpreterSettings::default() };
        settings.peaks.on = true;
        settings.peaks.summit = 20.0;
        settings.peaks.summit_from = 180.0;
        settings.lift.height = 60.0;
        settings.ridge_wander = 40.0;
        CompiledMap::new(std::sync::Arc::new(maps), 7, settings)
    }

    /// The recipe in whole numbers makes the same ground as in fractions (the noise is whole numbers in
    /// both): the same rules, rounded differently on the way. Measured 2026-11-05: 98% of columns the same
    /// block, 0.02 of a block apart on average, about a block and a half at worst.
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
            // (Up to two blocks at worst: the crags read the rise's slope off its curve, where the
            // fractions measured it across a block, so a steep crag can stand a little differently.)
            assert!(same * 100 >= count * 95 && mean < 0.05 && worst < 2.0, "steady {steady}: {same} of {count} the same block, {mean:.3} blocks apart on average, {worst:.2} at worst");
        }
    }


    /// The rivers and lakes carve the same in whole numbers as in fractions: a river with a bend, a fall
    /// and faded banks across the rich world, and a lake beside it, each way of banking.
    #[test]
    fn the_whole_number_carving_carves_the_same() {
        use crate::world::carving::{Banks, Carving, FadeWay, Lake, Lakes, Point, Reach};
        let point = |north: f32, east: f32, width: f32, level: f32, bend: f32, curtain: f32| Point { north, east, width, depth: 3.0, edge: 0.4, flat: 0.3, bowl: 1.3, level, bend, curtain, turn: 0.0, fade: 12.0 };
        let river = Reach { points: vec![point(-300.5, -400.0, 6.0, 200.0, 1.0, f32::NAN), point(-200.0, -250.0, 7.0, 200.0, 1.0, f32::NAN), point(-120.0, -100.0, 8.0, 190.0, -1.0, 200.0), point(-60.0, 80.0, 9.0, 180.0, 0.0, f32::NAN), point(0.0, 300.0, 10.0, 175.0, 0.0, f32::NAN)] };
        let surface: Vec<f32> = (0..400).map(|i| if (i % 20) > 6 && (i / 20) > 5 { 170.0 } else { f32::NAN }).collect();
        let lakes = Lakes::new(vec![Lake::new(200.0, -200.0, 20, 20, 8.0, surface)]);
        for (way, compressor) in [(FadeWay::Smooth, false), (FadeWay::Slope, false), (FadeWay::Terraces, false), (FadeWay::LedgeThenSlope, true), (FadeWay::Smooth, true)] {
            let carving = Carving::new(vec![river.clone()], lakes.clone(), Banks { way, compressor, ..Banks::default() });
            let map = rich_map().with_rivers(std::sync::Arc::new(carving.clone()));
            let (whole, carved) = (WholeMap::new(&map), WholeCarving::new(&carving));
            let (mut same, mut count, mut water_same, mut worst, mut wet) = (0, 0, 0, 0.0f32, 0);
            for north in (-420..260).step_by(3) {
                for east in (-450..360).step_by(5) {
                    let (north, east) = (north as f32 + 0.5, east as f32 + 0.5);
                    let Some((want, want_water)) = map.ground_and_water(north, east) else { continue };
                    let (got, got_water) = whole.ground_and_water(&carved, Pos::from_f32(north), Pos::from_f32(east)).expect("inside the map");
                    same += (want.round() == got.to_f32().round()) as usize;
                    water_same += (want_water == got_water) as usize;
                    wet += want_water.is_some() as usize;
                    count += 1;
                    worst = worst.max((want - got.to_f32()).abs());
                }
            }
            eprintln!("{way:?}, compressor {compressor}: {same} of {count} the same block, {water_same} the same water ({wet} wet), {worst:.2} at worst");
            assert!(wet > 200, "only {wet} columns of water: the test's river and lake missed the land");
            assert!(same * 100 >= count * 95 && water_same * 100 >= count * 97, "{way:?}, compressor {compressor}: {same} of {count} the same block, {water_same} the same water, {worst:.2} at worst");
        }
    }

    /// A curve's steepness in whole numbers is its slope (measured from the fractions across a step).
    #[test]
    fn a_curve_s_steepness_is_its_slope() {
        let mut spline = Spline::through(&[(0.0, 0.0), (0.3, 0.6), (0.7, 0.65), (1.0, 1.0)]);
        spline.bends = vec![0.0, 0.5, -0.4];
        let (curve, whole) = (spline.curve(), spline.curve().whole());
        for k in 1..100 {
            let x = k as f32 / 100.0;
            // (Not right at a dot: a bowed stretch can start as steep as it likes, and a slope measured
            // across the dot straddles two stretches.)
            if [0.3f32, 0.7].iter().any(|dot| (x - dot).abs() < 0.015) {
                continue;
            }
            let measured = (curve.at(x + 0.0005) - curve.at(x - 0.0005)) / 0.001;
            let got = whole.steepness(Fix::from_f32(x)).to_f32();
            assert!((measured - got).abs() < 0.02 * measured.abs().max(1.0), "at {x}: {got} against {measured}");
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



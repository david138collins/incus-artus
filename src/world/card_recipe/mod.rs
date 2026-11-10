//! The recipe on the card (the regroup, phase 2.3): `whole_recipe.rs` in the shader language
//! (`recipe.wgsl`), reading the world packed into one list of whole numbers (`Pack`), the very numbers the
//! processor reads. Proved by the twin test: the card and the processor bit for bit, every time.
//!
//! Built a piece at a time, each with its own twin test: the noise, curves and noise layers first.

use crate::whole::{Fix, Pos};
use crate::world::whole_recipe::{WholeCarving, WholeCurve, WholeMap, WholeNoise};
use crate::worldgen::noise::{ARROWS, Fade};

mod on_card;
pub use on_card::{WorldOnCard, send};

const RECIPE: &str = include_str!("recipe.wgsl");

/// The recipe in the shader language: the kit, the noise's arrows, then the recipe's steps.
pub fn source() -> String {
    let arrows: Vec<String> = ARROWS.iter().map(|(x, y)| format!("vec2<i32>({x}, {y})")).collect();
    [crate::whole::shader::source(), format!("var<private> ARROWS: array<vec2<i32>, 64> = array<vec2<i32>, 64>({});\n", arrows.join(", ")), slot_names(), RECIPE.to_string()].concat()
}

/// The world packed for the card: one list of whole numbers, each part's start noted where it's packed.
#[derive(Clone, Debug, Default)]
pub struct Pack {
    pub words: Vec<i32>,
}

fn fade_number(fade: Fade) -> i32 {
    match fade {
        Fade::Smooth => 0,
        Fade::Soft => 1,
        Fade::Straight => 2,
    }
}

impl Pack {
    fn at(&self) -> u32 {
        self.words.len() as u32
    }

    fn push(&mut self, values: &[i32]) {
        self.words.extend_from_slice(values);
    }

    /// A curve: its stretches, its single value, then each stretch's nine numbers.
    pub fn curve(&mut self, curve: &WholeCurve) -> u32 {
        let at = self.at();
        self.push(&[curve.stretches.len() as i32, curve.single.0]);
        for s in &curve.stretches {
            self.push(&[s.x0.0, s.x1.0, s.y0.0, s.y1.0, s.h.0, s.hm0.0, s.hm1.0, s.power.is_some() as i32, s.power.unwrap_or(Fix::ZERO).0]);
        }
        at
    }

    /// A noise layer: its fifteen numbers, then each octave's key, size and weight.
    pub fn noise(&mut self, noise: &WholeNoise) -> u32 {
        let at = self.at();
        let sh = &noise.shaping;
        self.push(&[
            noise.offset_north.0,
            noise.offset_east.0,
            noise.cos.0,
            noise.sin.0,
            noise.over_stretch.0,
            noise.turn_cos.0,
            noise.turn_sin.0,
            noise.octaves.len() as i32,
            noise.solo.map_or(-1, |k| k as i32),
            fade_number(noise.fade),
            sh.contrast.0,
            sh.brightness.0,
            sh.power.is_some() as i32,
            sh.power.unwrap_or(Fix::ZERO).0,
            sh.invert as i32,
        ]);
        for octave in &noise.octaves {
            self.push(&[octave.key as i32, octave.size.0, octave.weight.0]);
        }
        at
    }
}


/// The header's slots: each a number, or where a part starts in the list. The shader's names for them
/// (`H_…`) are made from this list, so the two can't drift apart.
pub const SLOTS: &[&str] = &[
    "WIDTH", "BLOCKS", "HALF", "COAST", "HEIGHT", "TILT", "SQUEEZE", "PUSH_X", "PUSH_Y", "PUSH_STRENGTH", "COAST_DISTANCE", "LIFT_DISTANCE",
    "WATER", "LOWEST", "HIGHEST", "MOST_CRUSHED", "MOST_STRETCHED", "MUTES", "DEEPEST", "HIGHLANDS", "CEILING", "GRADE", "COAST_RISE",
    "COAST_EDGE_WIDTH", "COAST_FALL_BACK", "RIDGES", "RIDGE_ALIGNMENT", "STEADY", "RIDGE_WANDER", "RIDGE_WANDER_SIZE", "RIDGE_SPACING",
    "RIDGE_LENGTH", "SHARPNESS", "RIDGE_LAYERS", "RIDGE_SCALES", "RIDGE_STRENGTHS", "RIDGE_TOTAL", "RIDGE_KEYS", "WANDER_KEY_0",
    "WANDER_KEY_1", "DIRECTIONS", "DETAIL", "DETAIL_NOISE", "MASK_SOURCE", "MASK_INVERT", "MASK_CURVE", "MASK_REACH", "KEEPS_COAST",
    "TORN_AMOUNT", "TORN_INLAND", "TORN_OUT", "TORN_NOISE", "TORN_PATCHES", "TORN_PATCH_SIZE", "TORN_PATCH_KEY", "TORN_BALANCE",
    "LIFT_HEIGHT", "LIFT_REACH", "LIFT_PAD", "LIFT_PAD_HAS", "LIFT_PAD_POWER", "PEAKS_ON", "PEAKS_SWITCHED", "PEAK_HEIGHT", "PEAKS_ADD",
    "CRAGS", "CRAG_COUNT", "CRAG_LAYERS", "FOOT_FADE", "SUMMIT", "SUMMIT_NOISE", "SUMMIT_FROM", "SUMMIT_FULL", "PEAK_COAST_FADE",
    "PEAK_COAST_HAS", "PEAK_COAST_POWER", "STEEPEST", "CARVE_BELOW", "PEAK_SIDE", "PEAK_ACROSS", "PEAK_CELLS", "PEAK_LIST", "LAND", "SEA",
    "MOUNTAINS", "LIFT_CURVE", "COAST_EDGE", "RISE", "SUMMIT_FADE", "SLOPE_RESPONSE",
    // The rivers and lakes.
    "CARVE_FULL", "REACHES", "POINTS", "BANK_WAY", "BANK_STEEPEST", "BANK_WIDEST", "BANK_TERRACE", "BANK_COMPRESSOR",
    "BANK_RAISED_STEEPEST", "BANK_RAISED_WIDEST", "BANK_WIDEST_EITHER", "CURTAINS", "ANTI_KITTY", "GRID_CELL", "GRID_ACROSS",
    "GRID_NORTH", "GRID_EAST", "GRID_CELLS", "GRID_ENTRIES", "LAKES", "LAKE_CELL", "LAKE_ACROSS_X", "LAKE_ACROSS_Y", "LAKE_NORTH",
    "LAKE_WEST", "LAKE_CELLS", "LAKE_ENTRIES",
    // The climate, the far index of the rivers, and mineralogy.
    "HAS_CLIMATE", "TEMPERATURE", "RAIN", "COOLING", "FAR_NORTH", "FAR_EAST", "FAR_ACROSS", "FAR_DOWN", "FAR_WIDEST_HALF", "FAR_CELLS",
    "FAR_ENTRIES", "MINERALOGY", "CRUST",
    // The caves.
    "TUNNELS", "CAVE_POINTS", "CAVE_SLOTS", "CAVE_TABLE", "CAVE_PIECES",
    // Sight: the blocks' far colours.
    "SIGHT_BLOCKS", "SIGHT_COLOURS",
];

/// A slot's number in the header.
fn slot(name: &str) -> usize {
    SLOTS.iter().position(|&s| s == name).unwrap_or_else(|| panic!("no slot {name}"))
}

/// A box's place in the caves' hash table (`slots` a power of two): as `cave_slot` on the card.
fn cave_slot(cx: i32, cy: i32, level: i32, slots: usize) -> usize {
    (((cx as u32).wrapping_mul(73_856_093)) ^ ((cy as u32).wrapping_mul(19_349_663)) ^ ((level as u32).wrapping_mul(83_492_791))) as usize & (slots - 1)
}

/// The shader's names for the header's slots.
fn slot_names() -> String {
    SLOTS.iter().enumerate().map(|(k, name)| format!("const H_{name}: u32 = {k}u;\n")).collect()
}

/// The parts left out (`Mutes`), as bits: lift 1, mountains 2, the edge rise 4, ridges 8, the torn coast
/// 16, detail 32, peaks 64, crags 128, summit crags 256, rivers 512.
fn mute_bits(m: &crate::world::Mutes) -> i32 {
    [m.lift, m.mountains, m.raised_coast, m.ridges, m.torn_coast, m.detail, m.peaks, m.crags, m.summit_crags, m.rivers].iter().enumerate().map(|(k, &on)| (on as i32) << k).sum()
}

fn mask_number(source: crate::world::MaskSource) -> i32 {
    use crate::world::MaskSource;
    match source {
        MaskSource::Everywhere => 0,
        MaskSource::Coast => 1,
        MaskSource::Height => 2,
        MaskSource::Mountains => 3,
        MaskSource::Squeeze => 4,
    }
}

impl Pack {
    fn set(&mut self, name: &str, value: i32) {
        self.words[slot(name)] = value;
    }

    /// Numbers put on the end of the list, and where they start.
    pub fn list(&mut self, values: &[i32]) -> i32 {
        self.array(values.iter().copied())
    }

    /// The caves (`CavePlan`): every tunnel (first point, how many, rough, lump, floor, seed) and point (x, y,
    /// z in 16ths, width, height), and the boxes near them in a hash table (a power of two of slots, each
    /// used, the box, its first piece and how many; open addressing, as `cave_slot` on the card).
    pub fn caves(&mut self, plan: &crate::world::caves::CavePlan) {
        let (mut tunnels, mut points) = (Vec::new(), Vec::new());
        for t in &plan.tunnels {
            tunnels.extend([(points.len() / 5) as i32, t.points.len() as i32, t.rough, t.lump, t.floor, t.seed as i32]);
            for p in &t.points {
                points.extend([p.at[0], p.at[1], p.at[2], p.width, p.height]);
            }
        }
        let at = self.array(tunnels);
        self.set("TUNNELS", at);
        let at = self.array(points);
        self.set("CAVE_POINTS", at);
        // Sorted, so the same plan packs the same way however its index was made.
        let mut boxes: Vec<(&(i32, i32, i32), &Vec<(u32, u32)>)> = plan.boxes_with_pieces().collect();
        boxes.sort_by_key(|(key, _)| **key);
        let slots = if boxes.is_empty() { 0 } else { (boxes.len() * 2).next_power_of_two() };
        let mut table = vec![0i32; slots * 6];
        let mut pieces = Vec::new();
        for (&(cx, cy, level), list) in boxes {
            let mut slot = cave_slot(cx, cy, level, slots);
            while table[slot * 6] != 0 {
                slot = (slot + 1) & (slots - 1);
            }
            table[slot * 6..slot * 6 + 6].copy_from_slice(&[1, cx, cy, level, (pieces.len() / 2) as i32, list.len() as i32]);
            for &(t, p) in list {
                pieces.extend([t as i32, p as i32]);
            }
        }
        self.set("CAVE_SLOTS", slots as i32);
        let at = self.array(table);
        self.set("CAVE_TABLE", at);
        let at = self.array(pieces);
        self.set("CAVE_PIECES", at);
    }

    /// The blocks' far colours for sight (`sight::cell::FarColours`): how many blocks, then each one's top's
    /// red, green and blue and its sides', in 1,024ths of linear light.
    pub fn sight(&mut self, colours: &crate::sight::cell::FarColours) {
        self.set("SIGHT_BLOCKS", colours.blocks.len() as i32);
        let at = self.array(colours.blocks.iter().flat_map(|&(top, side)| [top[0], top[1], top[2], side[0], side[1], side[2]]).map(|v| v as i32));
        self.set("SIGHT_COLOURS", at);
    }

    /// Mineralogy, its program in whole numbers, for a world of this seed.
    pub fn mineralogy(&mut self, mineralogy: &crate::world::Mineralogy, program: &crate::world::placer::WholeProgram, seed: u64, crust: Fix) {
        let at = mineralogy.pack(program, seed, self);
        self.set("MINERALOGY", at);
        self.set("CRUST", crust.0);
    }

    fn array(&mut self, values: impl IntoIterator<Item = i32>) -> i32 {
        let at = self.at() as i32;
        self.words.extend(values);
        at
    }

    /// A world's map and recipe, and its rivers and lakes: all the card's recipe reads.
    pub fn world(map: &WholeMap, carving: &WholeCarving) -> Self {
        let mut p = Self::map(map);
        p.carving(carving);
        p
    }

    /// The rivers and lakes (`WholeCarving`): every point, the lookup grids flattened (each cell's first
    /// entry and how many), and the lakes' water.
    fn carving(&mut self, c: &WholeCarving) {
        use crate::world::carving::FadeWay;
        const NONE: i32 = i32::MIN;
        self.set("CARVE_FULL", c.full as i32);
        let (mut reaches, mut points) = (Vec::new(), Vec::new());
        for reach in &c.reaches {
            reaches.extend([(points.len() / 12) as i32, reach.len() as i32]);
            for p in reach {
                points.extend([p.north.0, p.east.0, p.width.0, p.depth.0, p.edge.0, p.flat.0, p.bowl.0, p.level.0, p.bend.0, p.curtain.is_some() as i32, p.curtain.unwrap_or(Fix::ZERO).0, p.fade.0]);
            }
        }
        let at = self.array(reaches);
        self.set("REACHES", at);
        let at = self.array(points);
        self.set("POINTS", at);
        if let Some(b) = &c.banks {
            let way = match b.way {
                FadeWay::Smooth => 0,
                FadeWay::Slope => 1,
                FadeWay::Terraces => 2,
                FadeWay::LedgeThenSlope => 3,
            };
            for (name, value) in [("BANK_WAY", way), ("BANK_STEEPEST", b.steepest.0), ("BANK_WIDEST", b.widest.0), ("BANK_TERRACE", b.terrace.0), ("BANK_COMPRESSOR", b.compressor as i32), ("BANK_RAISED_STEEPEST", b.raised_steepest.0), ("BANK_RAISED_WIDEST", b.raised_widest.0), ("BANK_WIDEST_EITHER", b.widest_either.0)] {
                self.set(name, value);
            }
        }
        self.set("CURTAINS", c.curtains as i32);
        self.set("ANTI_KITTY", c.anti_kitty_corner as i32);
        self.set("GRID_CELL", c.cell.0);
        self.set("GRID_ACROSS", if c.cells.is_empty() { 0 } else { c.across });
        self.set("GRID_NORTH", c.corner.0.0);
        self.set("GRID_EAST", c.corner.1.0);
        let (mut cells, mut entries) = (Vec::new(), Vec::new());
        for cell in &c.cells {
            cells.extend([(entries.len() / 2) as i32, cell.len() as i32]);
            for &(r, k) in cell {
                entries.extend([r as i32, k as i32]);
            }
        }
        let at = self.array(cells);
        self.set("GRID_CELLS", at);
        let at = self.array(entries);
        self.set("GRID_ENTRIES", at);
        // The lakes: north, west, cols, rows, cell, where its water starts and where the water beside it does.
        let mut lakes = Vec::new();
        let mut water = Vec::new();
        for lake in &c.lakes {
            lakes.push(lake.clone());
            water.push(lake.surface.iter().map(|s| s.map_or(NONE, |f| f.0)).collect::<Vec<i32>>());
            water.push(lake.beside.iter().map(|s| s.map_or(NONE, |f| f.0)).collect::<Vec<i32>>());
        }
        let mut table = Vec::new();
        let mut starts = Vec::new();
        for v in water {
            starts.push(self.array(v));
        }
        for (k, lake) in lakes.iter().enumerate() {
            table.extend([lake.north.0, lake.west.0, lake.cols, lake.rows, lake.cell.0, starts[2 * k], starts[2 * k + 1]]);
        }
        let at = self.array(table);
        self.set("LAKES", at);
        self.set("LAKE_CELL", c.lake_cell.0);
        self.set("LAKE_ACROSS_X", if c.lake_cells.is_empty() { 0 } else { c.lake_across.0 });
        self.set("LAKE_ACROSS_Y", c.lake_across.1);
        self.set("LAKE_NORTH", c.lake_corner.0.0);
        self.set("LAKE_WEST", c.lake_corner.1.0);
        let (mut cells, mut entries) = (Vec::new(), Vec::new());
        for cell in &c.lake_cells {
            cells.extend([entries.len() as i32, cell.len() as i32]);
            entries.extend(cell.iter().map(|&k| k as i32));
        }
        let at = self.array(cells);
        self.set("LAKE_CELLS", at);
        let at = self.array(entries);
        self.set("LAKE_ENTRIES", at);
        // The rivers on the coarse grid: each cell's first piece and how many; each piece its ends and half
        // its width.
        let far = &c.far;
        self.set("FAR_NORTH", far.corner.0.0);
        self.set("FAR_EAST", far.corner.1.0);
        self.set("FAR_ACROSS", if far.cells.is_empty() { 0 } else { far.across });
        self.set("FAR_DOWN", far.down);
        self.set("FAR_WIDEST_HALF", far.widest_half.0);
        let (mut cells, mut entries) = (Vec::new(), Vec::new());
        for cell in &far.cells {
            cells.extend([(entries.len() / 5) as i32, cell.len() as i32]);
            for &((an, ae), (bn, be), half) in cell {
                entries.extend([an.0, ae.0, bn.0, be.0, half.0]);
            }
        }
        let at = self.array(cells);
        self.set("FAR_CELLS", at);
        let at = self.array(entries);
        self.set("FAR_ENTRIES", at);
    }

    /// A world's map and recipe (`WholeMap`), the header first.
    pub fn map(map: &WholeMap) -> Self {
        let mut p = Pack { words: vec![0; SLOTS.len()] };
        let fix = |v: &[Fix]| v.iter().map(|f| f.0).collect::<Vec<i32>>();
        let pos = |v: &[Pos]| v.iter().map(|f| f.0).collect::<Vec<i32>>();
        p.set("WIDTH", map.width);
        p.set("HAS_CLIMATE", (!map.temperature.is_empty()) as i32);
        p.set("COOLING", map.cooling.0);
        p.set("BLOCKS", map.blocks.0);
        p.set("HALF", map.half.0);
        for (name, field) in [("COAST", &map.coast), ("HEIGHT", &map.height), ("TILT", &map.tilt), ("SQUEEZE", &map.squeeze), ("PUSH_X", &map.push_x), ("PUSH_Y", &map.push_y), ("PUSH_STRENGTH", &map.push_strength)] {
            let at = p.array(fix(field));
            p.set(name, at);
        }
        for (name, field) in [("COAST_DISTANCE", &map.coast_distance), ("LIFT_DISTANCE", &map.lift_distance)] {
            let at = p.array(pos(field));
            p.set(name, at);
        }
        for (name, value) in [
            ("WATER", map.water.0),
            ("LOWEST", map.lowest.0),
            ("HIGHEST", map.highest.0),
            ("MOST_CRUSHED", map.most_crushed.0),
            ("MOST_STRETCHED", map.most_stretched.0),
            ("MUTES", mute_bits(&map.mutes)),
            ("DEEPEST", map.deepest.0),
            ("HIGHLANDS", map.highlands.0),
            ("CEILING", map.ceiling.0),
            ("GRADE", map.grade.0),
            ("COAST_RISE", map.coast_rise.0),
            ("COAST_EDGE_WIDTH", map.coast_edge_width.0),
            ("COAST_FALL_BACK", map.coast_fall_back.0),
            ("RIDGES", map.ridges.0),
            ("RIDGE_ALIGNMENT", map.ridge_alignment.0),
            ("STEADY", map.steady_ridges as i32),
            ("RIDGE_WANDER", map.ridge_wander.0),
            ("RIDGE_WANDER_SIZE", map.ridge_wander_size.0),
            ("RIDGE_SPACING", map.ridge_spacing.0),
            ("RIDGE_LENGTH", map.ridge_length.0),
            ("SHARPNESS", map.sharpness.0),
            ("RIDGE_LAYERS", map.ridge_scales.len() as i32),
            ("RIDGE_TOTAL", map.ridge_total.0),
            ("WANDER_KEY_0", map.wander_keys[0] as i32),
            ("WANDER_KEY_1", map.wander_keys[1] as i32),
            ("DETAIL", map.detail.0),
            ("MASK_SOURCE", mask_number(map.detail_mask.source)),
            ("MASK_INVERT", map.detail_mask.invert as i32),
            ("MASK_REACH", map.detail_mask.reach.0),
            ("KEEPS_COAST", map.detail_keeps_coast as i32),
            ("TORN_AMOUNT", map.torn_amount.0),
            ("TORN_INLAND", map.torn_inland.0),
            ("TORN_OUT", map.torn_out_to_sea.0),
            ("TORN_PATCHES", map.torn_patches.0),
            ("TORN_PATCH_SIZE", map.torn_patch_size.0),
            ("TORN_PATCH_KEY", map.torn_patch_key as i32),
            ("TORN_BALANCE", map.torn_balance.0),
            ("LIFT_HEIGHT", map.lift_height.0),
            ("LIFT_REACH", map.lift_reach.0),
            ("LIFT_PAD", map.lift_pad.0),
            ("LIFT_PAD_HAS", map.lift_pad_power.is_some() as i32),
            ("LIFT_PAD_POWER", map.lift_pad_power.unwrap_or(Fix::ZERO).0),
            ("PEAKS_ON", map.peaks_on as i32),
            ("PEAKS_SWITCHED", map.peaks_switched_on as i32),
            ("PEAK_HEIGHT", map.peak_height.0),
            ("PEAKS_ADD", map.peaks_add as i32),
            ("CRAGS", map.crags.0),
            ("CRAG_COUNT", map.crag_layers.len() as i32),
            ("FOOT_FADE", map.foot_fade.0),
            ("SUMMIT", map.summit.0),
            ("SUMMIT_FROM", map.summit_from.0),
            ("SUMMIT_FULL", map.summit_full.0),
            ("PEAK_COAST_FADE", map.peak_coast_fade.0),
            ("PEAK_COAST_HAS", map.peak_coast_power.is_some() as i32),
            ("PEAK_COAST_POWER", map.peak_coast_power.unwrap_or(Fix::ZERO).0),
            ("STEEPEST", map.steepest.0),
            ("CARVE_BELOW", map.carve_below as i32),
            ("PEAK_SIDE", map.peak_side.0),
            ("PEAK_ACROSS", map.peak_across),
        ] {
            p.set(name, value);
        }
        let at = p.array(fix(&map.temperature));
        p.set("TEMPERATURE", at);
        let at = p.array(fix(&map.rain));
        p.set("RAIN", at);
        let at = p.array(fix(&map.ridge_scales));
        p.set("RIDGE_SCALES", at);
        let at = p.array(fix(&map.ridge_strengths));
        p.set("RIDGE_STRENGTHS", at);
        let at = p.array(map.ridge_keys.iter().flatten().map(|&k| k as i32).collect::<Vec<i32>>());
        p.set("RIDGE_KEYS", at);
        let at = p.array(map.directions.iter().flat_map(|&(c, s)| [c.0, s.0]).collect::<Vec<i32>>());
        p.set("DIRECTIONS", at);
        let at = p.array(map.crag_layers.iter().flat_map(|&(size, kept, key)| [size.0, kept.0, key as i32]).collect::<Vec<i32>>());
        p.set("CRAG_LAYERS", at);
        // The peaks: each cell's first peak and how many, then every peak's six numbers.
        let mut cells = Vec::new();
        let mut list = Vec::new();
        for cell in &map.peak_cells {
            cells.extend([(list.len() / 6) as i32, cell.len() as i32]);
            for peak in cell {
                list.extend([peak.x.0, peak.y.0, peak.height.0, peak.width.0, peak.wobble.0, peak.key as i32]);
            }
        }
        let at = p.array(cells);
        p.set("PEAK_CELLS", at);
        let at = p.array(list);
        p.set("PEAK_LIST", at);
        for (name, noise) in [("DETAIL_NOISE", &map.detail_noise), ("TORN_NOISE", &map.torn_noise), ("SUMMIT_NOISE", &map.summit_noise)] {
            let at = p.noise(noise) as i32;
            p.set(name, at);
        }
        for (name, curve) in [("MASK_CURVE", &map.detail_mask.curve), ("LAND", &map.land), ("SEA", &map.sea), ("MOUNTAINS", &map.mountains), ("LIFT_CURVE", &map.lift), ("COAST_EDGE", &map.coast_edge), ("RISE", &map.rise), ("SUMMIT_FADE", &map.summit_fade), ("SLOPE_RESPONSE", &map.slope_response)] {
            let at = p.curve(curve) as i32;
            p.set(name, at);
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Spline;
    use crate::worldgen::noise::{PerlinSettings, whole_perlin_at, whole_perlin_with_slope_at};

    /// Runs `entry` over `cases` (four numbers each) on the card, with the world packed as `data`; its
    /// answers, one each.
    fn on_the_card(entry: &str, data: &[i32], cases: &[[i32; 4]]) -> Option<Vec<i32>> {
        use wgpu::util::DeviceExt;
        let (device, queue) = crate::whole::shader::card()?;
        let twin = format!(
            "
struct Case {{ a: i32, b: i32, c: i32, d: i32 }}
@group(0) @binding(1) var<storage, read> cases: array<Case>;
@group(0) @binding(2) var<storage, read_write> results: array<i32>;
@compute @workgroup_size(64)
fn twin(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= arrayLength(&cases)) {{
        return;
    }}
    results[i] = {entry}(cases[i]);
}}
"
        );
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the recipe's twin test"), source: wgpu::ShaderSource::Wgsl((source() + TESTS + &twin).into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some("twin"), compilation_options: Default::default(), cache: None });
        let bytes = |v: &[i32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let mut packed = data.to_vec();
        if packed.is_empty() {
            packed.push(0);
        }
        let world = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("the world"), contents: &bytes(&packed), usage: wgpu::BufferUsages::STORAGE });
        let flat: Vec<i32> = cases.iter().flatten().copied().collect();
        let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("cases"), contents: &bytes(&flat), usage: wgpu::BufferUsages::STORAGE });
        let size = cases.len() as u64 * 4;
        let output = device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: world.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: input.as_entire_binding() }, wgpu::BindGroupEntry { binding: 2, resource: output.as_entire_binding() }],
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((cases.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let view = readback.slice(..).get_mapped_range().expect("the results");
        Some(view.chunks_exact(4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect())
    }

    /// The test's entries into the recipe, case by case.
    const TESTS: &str = "
fn noise_and_curves(c: Case) -> i32 {
    switch c.a {
        case 0: { return curve_at(u32(c.b), c.c); }
        case 1: { return curve_steepness(u32(c.b), c.c); }
        case 2: { return noise_at(u32(c.b), c.c, c.d); }
        case 3: { return noise_raw(u32(c.b), c.c, c.d); }
        case 4: { return perlin_at(0x51u, 0, c.c, c.d); }
        case 5: { return perlin_with_slope_at(0x51u, c.c, c.d).x; }
        case 6: { return perlin_with_slope_at(0x51u, c.c, c.d).y; }
        case 7: { return perlin_with_slope_at(0x51u, c.c, c.d).z; }
        default: { return 0; }
    }
}

fn minerals(c: Case) -> i32 {
    var col: Col;
    col.north = c.a;
    col.east = c.b;
    col.ground = (c.c & 0xFFFF) << 16u;
    col.water = c.c >> 16u;
    col.slope = c.d;
    let climate = climate_at(c.a, c.b, col.ground);
    col.temperature = 13107;
    col.rain = 39322;
    if (climate.z != 0) {
        col.temperature = climate.x;
        col.rain = climate.y;
    }
    return choose_blocks(with_maps(col));
}

fn columns(c: Case) -> i32 {
    let column = world_column(c.a, c.b);
    if (column.z == 0) {
        return -1;
    }
    return column.x | (column.y << 16u);
}

fn caves(c: Case) -> i32 {
    return select(0, 1, cave_inside(c.a, c.b, c.c));
}

fn carved(c: Case) -> i32 {
    let column = ground_and_water(c.a, c.b);
    if (!column.ok) {
        return -2147483647 - 1;
    }
    if (c.c == 1) {
        return column.water;
    }
    return column.z;
}

fn ground(c: Case) -> i32 {
    let height = uncarved(c.a, c.b);
    if (!height.ok) {
        return -2147483647 - 1;
    }
    return height.z;
}
";

    /// A small world with every part of the recipe in it (as `whole_recipe`'s tests have), with this
    /// noise maths and ridges steady or not.
    fn rich_map(steady: bool) -> crate::world::CompiledMap {
        use crate::world::{CompiledMap, InterpreterSettings, MaskSource, Maps, Mutes, point_distance};
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
        let mut settings = InterpreterSettings { coast_rise: 20.0, coast_fall_back: 300.0, detail: 12.0, steady_ridges: steady, ..InterpreterSettings::default() };
        settings.peaks.on = true;
        settings.peaks.crags = 1.0;
        settings.peaks.summit = 40.0;
        settings.peaks.summit_from = 180.0;
        settings.peaks.coast_falloff = 1.6;
        settings.lift.height = 60.0;
        settings.ridge_wander = 40.0;
        settings.detail_mask.source = MaskSource::Coast;
        settings.detail_mask.curve.bends = vec![-1.0];
        settings.detail_noise.output.curve = 2.3;
        settings.detail_noise.octaves = 3;
        let mut map = CompiledMap::new(std::sync::Arc::new(maps), 7, settings);
        map.mutes = Mutes::default();
        map
    }

    /// The rivers and lakes carved in: the card bit for bit the processor, every way of banking, the
    /// compressor on and off, a waterfall and a lake.
    #[test]
    fn the_card_carves_the_rivers_and_lakes_as_the_processor_does() {
        use crate::world::carving::{Banks, Carving, FadeWay, Lake, Lakes, Point, Reach};
        let point = |north: f32, east: f32, width: f32, level: f32, bend: f32, curtain: f32| Point { north, east, width, depth: 3.0, edge: 0.4, flat: 0.3, bowl: 1.3, level, bend, curtain, turn: 0.0, fade: 12.0 };
        let river = Reach { points: vec![point(-300.5, -400.0, 6.0, 200.0, 1.0, f32::NAN), point(-200.0, -250.0, 7.0, 200.0, 1.0, f32::NAN), point(-120.0, -100.0, 8.0, 190.0, -1.0, 200.0), point(-60.0, 80.0, 9.0, 180.0, 0.0, f32::NAN), point(0.0, 300.0, 10.0, 175.0, 0.0, f32::NAN)] };
        let surface: Vec<f32> = (0..400).map(|i| if (i % 20) > 6 && (i / 20) > 5 { 170.0 } else { f32::NAN }).collect();
        let lakes = Lakes::new(vec![Lake::new(200.0, -200.0, 20, 20, 8.0, surface)]);
        for (way, compressor) in [(FadeWay::Smooth, false), (FadeWay::Slope, false), (FadeWay::Terraces, false), (FadeWay::LedgeThenSlope, true), (FadeWay::Smooth, true), (FadeWay::Terraces, true)] {
            let carving = Carving::new(vec![river.clone()], lakes.clone(), Banks { way, compressor, ..Banks::default() });
            let map = rich_map(true).with_rivers(std::sync::Arc::new(carving.clone()));
            let (whole, carved) = (WholeMap::new(&map), WholeCarving::new(&carving));
            let pack = Pack::world(&whole, &carved);
            let mut cases = Vec::new();
            for north in (-420..260).step_by(3 * thinned().min(4)) {
                for east in (-450..360).step_by(5) {
                    for kind in [0, 1] {
                        cases.push([Pos::middle_of(north).0, Pos::middle_of(east).0, kind, 0]);
                    }
                }
            }
            let Some(answers) = on_the_card("carved", &pack.words, &cases) else {
                eprintln!("No graphics card: the carving's twin test is skipped");
                return;
            };
            let (mut wrong, mut wet) = (Vec::new(), 0);
            for (c, &card) in cases.iter().zip(&answers) {
                let column = whole.ground_and_water(&carved, Pos(c[0]), Pos(c[1]));
                let processor = column.map_or(i32::MIN, |(z, water)| if c[2] == 1 { water.unwrap_or(0) } else { z.0 });
                wet += (c[2] == 1 && processor > 0) as usize;
                if card != processor && wrong.len() < 12 {
                    wrong.push(format!("{} at ({}, {}): card {card}, processor {processor}", if c[2] == 1 { "water" } else { "ground" }, Pos(c[0]).to_f32(), Pos(c[1]).to_f32()));
                }
            }
            assert!(wet > 200 / thinned().min(4), "only {wet} columns of water");
            assert!(wrong.is_empty(), "{way:?}, compressor {compressor}: the card and the processor disagree:\n{}", wrong.join("\n"));
        }
    }

    /// Mineralogy: what each distributor chooses, the card bit for bit the processor, every kind of chooser,
    /// with a climate, a river near and the slope.
    #[test]
    fn the_card_chooses_the_blocks_as_the_processor_does() {
        use crate::world::carving::{Banks, Carving, Lakes, Point, Reach};
        use crate::world::placer::{Chooser, WholeColumn, WholeReading};
        use crate::world::{BlockId, TerrainBlocks};
        use std::collections::HashMap;
        let names = ["incus:gabbro", "incus:stone", "incus:grass", "incus:water", "incus:dirt", "incus:sand", "incus:dry_grass", "incus:snowy_ground", "incus:gravel"];
        let blocks = TerrainBlocks { gabbro: BlockId(1), stone: BlockId(2), grass: BlockId(3), water: BlockId(4), names: std::sync::Arc::new(names.iter().enumerate().map(|(k, n)| (n.to_string(), BlockId(k as u16 + 1))).collect::<HashMap<String, BlockId>>()) };
        let point = |north: f32, east: f32| Point { north, east, width: 8.0, depth: 3.0, edge: 0.4, flat: 0.3, bowl: 1.3, level: 190.0, bend: 0.0, curtain: f32::NAN, turn: 0.0, fade: 12.0 };
        let carving = Carving::new(vec![Reach { points: vec![point(-300.0, -400.0), point(0.0, 0.0), point(200.0, 300.0)] }], Lakes::default(), Banks::default());
        for chooser in [None, Some(Chooser::Best), Some(Chooser::Chance)] {
            let mut map = rich_map(true);
            let mut maps = (*map.maps).clone();
            let cells = maps.width * maps.width;
            maps.temperature = (0..cells).map(|i| ((i % 48) as f32 / 24.0) - 1.0).collect();
            maps.rain = (0..cells).map(|i| (i / 48) as f32 / 48.0).collect();
            maps.cooling = 0.3;
            let mut settings = map.settings.clone();
            if let Some(chooser) = chooser {
                for distributor in &mut settings.mineralogy.distributors {
                    distributor.chooser = chooser;
                }
            }
            map = crate::world::CompiledMap::new(std::sync::Arc::new(maps), map.seed, settings).with_rivers(std::sync::Arc::new(carving.clone()));
            let ready = map.mineralogy(&blocks);
            let whole = WholeMap::new(&map);
            let carved = WholeCarving::new(&carving);
            let program = ready.program().whole(map.seed);
            let mut pack = Pack::world(&whole, &carved);
            let crust = Fix::from_f32(map.settings.mineralogy.crust);
            pack.mineralogy(&ready, &program, map.seed, crust);
            let mut cases = Vec::new();
            for k in 0..3000 {
                let (north, east) = ((k * 211) % 3000 - 1500, (k * 157) % 3000 - 1500);
                let z = [100, 126, 129, 150, 260, 340, 500][k as usize % 7];
                let water = if k % 5 == 0 { z + 3 } else { 0 };
                cases.push([Pos::middle_of(north).0, Pos::middle_of(east).0, z | (water << 16), (k % 9) * 20_000]);
            }
            let Some(answers) = on_the_card("minerals", &pack.words, &cases) else {
                eprintln!("No graphics card: mineralogy's twin test is skipped");
                return;
            };
            let mut wrong = Vec::new();
            for (c, &card) in cases.iter().zip(&answers) {
                let (north, east, z, water) = (Pos(c[0]), Pos(c[1]), c[2] & 0xFFFF, c[2] >> 16);
                let column = WholeColumn { north, east, ground: Fix::from_int(z), slope: Fix(c[3]), climate: whole.climate(north, east, Fix::from_int(z)), water: (water > 0).then_some(water) };
                let mut reading = WholeReading::new(&whole, &carved, column, crust);
                let processor = ready.whole_chosen(&program, &mut reading).iter().enumerate().fold(0i32, |all, (d, chosen)| all | ((chosen.map_or(0, |c| c as i32 + 1)) << (8 * d)));
                if card != processor && wrong.len() < 12 {
                    wrong.push(format!("at ({}, {}) Z {z}: card {card:#x}, processor {processor:#x}", north.to_f32(), east.to_f32()));
                }
            }
            assert!(wrong.is_empty(), "{chooser:?}: the card and the processor disagree:\n{}", wrong.join("\n"));
        }
    }

    /// The world's columns as chunks are filled (whole blocks, the sea floor's rule, the water): the card bit
    /// for bit the processor's own chunk maker (`CompiledMap::column`).
    #[test]
    fn the_card_makes_the_world_s_columns_as_the_processor_does() {
        use crate::world::carving::{Banks, Carving, Lake, Lakes, Point, Reach};
        let point = |north: f32, east: f32, width: f32, level: f32| Point { north, east, width, depth: 3.0, edge: 0.4, flat: 0.3, bowl: 1.3, level, bend: 0.0, curtain: f32::NAN, turn: 0.0, fade: 12.0 };
        let carving = Carving::new(vec![Reach { points: vec![point(-300.5, -400.0, 6.0, 200.0), point(-60.0, 80.0, 9.0, 180.0), point(0.0, 300.0, 10.0, 175.0)] }], Lakes::new(vec![Lake::new(200.0, -200.0, 20, 20, 8.0, (0..400).map(|i| if i % 3 == 0 { 170.0 } else { f32::NAN }).collect())]), Banks::default());
        // The rich map has no sea (its peaks and lift raise it all), so a plain one sloping from deep sea up
        // to hills as well.
        let width = 48;
        let sloping = crate::world::CompiledMap::of_heights(width, 64.0, &(0..width * width).map(|i| 40.0 + (i % width) as f32 * 5.0).collect::<Vec<f32>>());
        let (mut under, mut wet) = (0, 0);
        for (name, map) in [("rich", rich_map(true)), ("sloping", sloping)] {
            let map = map.with_rivers(std::sync::Arc::new(carving.clone()));
            let whole = map.whole().expect("the recipe in whole numbers");
            let pack = Pack::world(&whole.map, &whole.carving);
            let mut cases = Vec::new();
            for x in (-1600..1600).step_by(9 * thinned()) {
                for y in (-1600..1600).step_by(13) {
                    cases.push([Pos::middle_of(x).0, Pos::middle_of(y).0, 0, 0]);
                }
            }
            let Some(answers) = on_the_card("columns", &pack.words, &cases) else {
                eprintln!("No graphics card: the columns' twin test is skipped");
                return;
            };
            let mut wrong = Vec::new();
            for (c, &card) in cases.iter().zip(&answers) {
                let (x, y) = (Pos(c[0]).0 >> 8, Pos(c[1]).0 >> 8);
                let processor = map.column(x, y).map_or(-1, |(z, water)| z | (water.unwrap_or(0) << 16));
                under += (processor >= 0 && (processor & 0xFFFF) < 128) as usize;
                wet += (processor > 0xFFFF) as usize;
                if card != processor && wrong.len() < 12 {
                    wrong.push(format!("column ({x}, {y}): card {card:#x}, processor {processor:#x}"));
                }
            }
            assert!(wrong.is_empty(), "{name}: the card and the processor disagree:\n{}", wrong.join("\n"));
        }
        assert!(under > 100 && wet > 20, "{under} under the sea, {wet} with water: the test misses them");
    }

    /// Sight's test world: the rich map with a river and a lake, mineralogy, tunnels climbing out of the
    /// ground (cave mouths, roofs), and made-up far colours, on the card.
    #[allow(clippy::type_complexity)]
    fn sight_world() -> Option<(wgpu::Device, wgpu::Queue, crate::world::CompiledMap, crate::world::CavePlan, crate::world::TerrainBlocks, crate::sight::cell::FarColours, super::WorldOnCard)> {
        use crate::sight::cell::FarColours;
        use crate::world::caves::{CavePlan, CavePoint, Tunnel};
        use crate::world::carving::{Banks, Carving, Lake, Lakes, Point, Reach};
        use crate::world::{BlockId, TerrainBlocks};
        use std::collections::HashMap;
        let (device, queue) = crate::whole::shader::card()?;
        let names = ["incus:gabbro", "incus:stone", "incus:grass", "incus:water", "incus:dirt", "incus:sand", "incus:dry_grass", "incus:snowy_ground", "incus:gravel"];
        let blocks = TerrainBlocks { gabbro: BlockId(1), stone: BlockId(2), grass: BlockId(3), water: BlockId(4), names: std::sync::Arc::new(names.iter().enumerate().map(|(k, n)| (n.to_string(), BlockId(k as u16 + 1))).collect::<HashMap<String, BlockId>>()) };
        let colours = FarColours { blocks: (0..=names.len() as u64).map(|k| ([k * 97 % 1024, k * 389 % 1024, k * 211 % 1024], [k * 53 % 1024, k * 701 % 1024, k * 131 % 1024])).collect(), water: blocks.water, media: Vec::new() };
        let point = |north: f32, east: f32, width: f32, level: f32| Point { north, east, width, depth: 3.0, edge: 0.4, flat: 0.3, bowl: 1.3, level, bend: 0.0, curtain: f32::NAN, turn: 0.0, fade: 12.0 };
        let carving = Carving::new(vec![Reach { points: vec![point(-300.5, -400.0, 6.0, 200.0), point(-60.0, 80.0, 9.0, 180.0), point(0.0, 300.0, 10.0, 175.0)] }], Lakes::new(vec![Lake::new(200.0, -200.0, 20, 20, 8.0, (0..400).map(|i| if i % 3 == 0 { 170.0 } else { f32::NAN }).collect())]), Banks::default());
        let map = rich_map(true).with_rivers(std::sync::Arc::new(carving));
        // Tunnels climbing out of the ground: cave mouths and their roofs.
        let ground = |x: i32, y: i32| map.column(x, y).map_or(150, |(z, _)| z);
        let tunnels: Vec<Tunnel> = [(600, 300), (630, 340), (-1000, -700)]
            .iter()
            .enumerate()
            .map(|(t, &(x, y))| {
                let points = (0..6).map(|k| CavePoint { at: [(x + k * 10) * 16, (y + k * 4) * 16, (ground(x + k * 10, y + k * 4) - 10 + k * 3) * 16], width: 64, height: 48 }).collect();
                Tunnel { points, rough: 6 + t as i32, lump: 3, floor: 180, seed: 77 + t as u32 }
            })
            .collect();
        let plan = CavePlan::new(tunnels);
        let (world, _) = super::send(&device, &queue, &map, &plan, &blocks, &colours, (0, 0)).expect("the world on the card");
        Some((device, queue, map, plan, blocks, colours, world))
    }

    /// Sight's live levels (phase 3.1c): every cell exactly its eight children at the level below added, the
    /// cells left out of the bands included (solid below, empty above), at every level, by caves and a river;
    /// and the cells just past each band really are solid all round, or empty, as the card makes them.
    #[test]
    fn sight_s_levels_are_their_children_added() {
        use crate::sight::cell::Sums;
        use crate::sight::store::{TILE, Tile, TileKey, fill_tile, sums_of};
        let Some((device, queue, _map, plan, _blocks, _colours, world)) = sight_world() else {
            eprintln!("No graphics card: sight's levels test is skipped");
            return;
        };
        let (mut cells, mut lowered, mut edges) = (0, 0, 0);
        for level in 1..5usize {
            for (x, y) in [(600i32, 300i32), (-60, 80)] {
                let tile_blocks = TILE << level;
                let parent_key = TileKey { level, x: x.div_euclid(tile_blocks), y: y.div_euclid(tile_blocks) };
                let parent = fill_tile(&device, &queue, &world, &plan, world.odd, parent_key);
                let children: Vec<Tile> = (0..4).map(|k| fill_tile(&device, &queue, &world, &plan, world.odd, TileKey { level: level - 1, x: parent_key.x * 2 + k % 2, y: parent_key.y * 2 + k / 2 })).collect();
                let size = 1 << level;
                for cy in 0..TILE {
                    for cx in 0..TILE {
                        let column = parent.column(cx, cy);
                        let child = |a: i32, b: i32| {
                            let (gx, gy) = (cx * 2 + a, cy * 2 + b);
                            children[((gy / TILE) * 2 + gx / TILE) as usize].column(gx % TILE, gy % TILE)
                        };
                        for z in 0..crate::world::WORLD_HEIGHT / size {
                            let mut added = Sums::default();
                            for k in 0..8 {
                                added.add(&child(k & 1, k >> 1 & 1).at(z * 2 + (k >> 2), size / 2));
                            }
                            assert_eq!(column.at(z, size), added, "level {level}, tile {parent_key:?}, cell column ({cx}, {cy}), cell {z}");
                        }
                        cells += column.sums.len();
                        lowered += (column.low > 0 && column.sums.len() > 6) as usize;
                    }
                }
                // The cells just under and over each band of the parent, made directly.
                let (x0, y0, side) = parent_key.blocks();
                let (mut past, mut expected) = (Vec::new(), Vec::new());
                for cy in 0..TILE {
                    for cx in 0..TILE {
                        let column = parent.column(cx, cy);
                        for z in [column.low - 1, column.low + column.sums.len() as i32] {
                            if z >= 0 && z < crate::world::WORLD_HEIGHT / size {
                                past.push([x0 + cx * size, y0 + cy * size, z * size, size]);
                                expected.push(column.at(z, size));
                            }
                        }
                    }
                }
                let made = world.cells_at(&device, &queue, [x0, y0, side, side], &past);
                for ((cell, made), expected) in past.iter().zip(&made).zip(&expected) {
                    assert_eq!(sums_of(made), *expected, "just past a band: {cell:?}");
                    edges += 1;
                }
            }
        }
        eprintln!("Sight's levels: {cells} cells kept checked against their children, {lowered} bands reaching well below the surface, {edges} cells just past a band checked");
        assert!(cells > 5000 && edges > 1000, "the test misses something: {cells} cells, {edges} band edges");
    }

    /// Sight's bake (phase 3.1d): the whole test map baked region by region; its cells of 32, 64 and 128 the
    /// same as those cells made directly on the card (by the cave mouths and the river, the first cell past
    /// each band included), and every level up to the top complete.
    #[test]
    fn sight_s_bake_is_the_cells_made_directly() {
        use crate::sight::bake::{Bake, FIRST};
        use crate::sight::store::{Maker, sums_of};
        let Some((device, queue, map, plan, _blocks, _colours, world)) = sight_world() else {
            eprintln!("No graphics card: sight's bake test is skipped");
            return;
        };
        let half = (map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0).ceil() as i32;
        let odd = world.odd;
        let maker = Maker { device: device.clone(), queue: queue.clone(), world: std::sync::Arc::new(world), caves: std::sync::Arc::new(plan), card: None };
        let with = crate::sight::bake::Blocks { map: std::sync::Arc::new(map.clone()), blocks: _blocks.clone(), colours: _colours.clone(), changed: Default::default() };
        let (baking, jobs) = crate::sight::bake::start(maker.clone(), with, odd, half, (0, 0), 1, None);
        // Waits for the loops on the loading lane to finish the map.
        let started = std::time::Instant::now();
        loop {
            let done = baking.bake.lock().map(|bake| bake.all_done.is_some()).unwrap_or(true);
            if done || started.elapsed().as_secs() > 300 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        drop(jobs);
        let bake = baking.bake.lock().expect("the bake");
        assert!(bake.all_done.is_some(), "the bake finished: {}", bake.report());
        assert_eq!(bake.regions, 16);
        let mut checked = 0;
        for level in [FIRST, FIRST + 1, FIRST + 2] {
            let size = 1 << level;
            for (x, y) in [(600i32, 300i32), (-60, 80), (-1500, 1400)] {
                let (cx, cy) = (x.div_euclid(size), y.div_euclid(size));
                let (low, cells) = bake.column(level, cx, cy).expect("baked");
                let mut asked = Vec::new();
                for z in (low - 1).max(0)..=low + cells.len() as i32 {
                    asked.push([cx * size, cy * size, z * size, size]);
                }
                let made = maker.world.cells_at(&device, &queue, [cx * size, cy * size, size, size], &asked);
                for (cell, made) in asked.iter().zip(&made) {
                    let z = cell[2] / size;
                    let expected = if z >= low && z < low + cells.len() as i32 {
                        cells[(z - low) as usize]
                    } else if z < low && low > 0 {
                        crate::sight::cell::Sums { solid: (size as u64).pow(3), ..Default::default() }.compact(size as u32)
                    } else {
                        crate::sight::cell::Cell::default()
                    };
                    assert_eq!(sums_of(made).compact(size as u32), expected, "level {level}, cell {cell:?}");
                    checked += 1;
                }
            }
        }
        // Every level up to the top is complete.
        for level in FIRST..=Bake::top(half) {
            let extent = bake.extent(level);
            for cy in extent.from.1..extent.from.1 + extent.across {
                for cx in extent.from.0..extent.from.0 + extent.across {
                    assert!(bake.column(level, cx, cy).is_some(), "level {level}, cell column ({cx}, {cy}) not baked");
                }
            }
        }
        eprintln!("Sight's bake: {}; {checked} cells checked against the card", bake.report());
        assert!(checked > 20);
    }

    /// Sight's bake kept (phase 3.1e): written to its file and read back the same, a bake started with it
    /// already done; with another stamp, baked again; and an edit (a hole dug at a chunk's edge) patched: the
    /// region's top cell exactly as many blocks lighter as were dug, the faces it bares in the column beside it,
    /// and the columns away from it as they were.
    #[test]
    fn sight_s_bake_is_kept_and_patched() {
        use crate::sight::bake::{Blocks, FIRST, bake_region, read, start};
        use crate::sight::store::Maker;
        use crate::world::{BlockId, ChunkPos};
        let Some((device, queue, map, plan, blocks, colours, world)) = sight_world() else {
            eprintln!("No graphics card: sight's kept bake test is skipped");
            return;
        };
        let half = (map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0).ceil() as i32;
        let odd = world.odd;
        let map = std::sync::Arc::new(map);
        let maker = Maker { device, queue, world: std::sync::Arc::new(world), caves: std::sync::Arc::new(plan.clone()), card: None };
        let with = Blocks { map: map.clone(), blocks: blocks.clone(), colours: colours.clone(), changed: Default::default() };
        let wait = |baking: &crate::sight::bake::Baking| {
            let started = std::time::Instant::now();
            while baking.bake.lock().map(|b| b.all_done.is_none()).unwrap_or(false) && started.elapsed().as_secs() < 300 {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        };
        let (baking, jobs) = start(maker.clone(), with.clone(), odd, half, (0, 0), 77, None);
        wait(&baking);
        drop(jobs);
        let folder = std::env::temp_dir().join(format!("incus-sight-bake-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("a folder");
        let note = baking.bake.lock().expect("the bake").keep(&folder).expect("kept");
        eprintln!("Sight's bake kept: {note}");
        let kept = read(&folder).expect("read").expect("a file");
        assert_eq!(kept.stamp, 77);
        assert_eq!(kept.records.len(), 16);
        // Read back the same, region by region.
        for record in &kept.records {
            let baked = crate::sight::bake::bake_region(&maker, &with, odd, record.region);
            assert_eq!(*record, baked, "region {:?} read back", record.region);
        }
        // Started with it, the bake is done before a region's baked: nothing baked, and what was read back put on the
        // card a few regions a frame (`take_kept`), not as it starts; with another stamp, it isn't.
        let (again, jobs) = start(maker.clone(), with.clone(), odd, half, (0, 0), 77, read(&folder).expect("read"));
        assert_eq!(again.bake.lock().expect("the bake").regions_done, 0, "none put on the card as it starts");
        crate::sight::bake::take_kept(&again, maker.card.as_deref(), 10);
        assert_eq!(again.bake.lock().expect("the bake").regions_done, 10, "ten a frame");
        crate::sight::bake::take_kept(&again, maker.card.as_deref(), 10);
        assert_eq!(again.bake.lock().expect("the bake").regions_done, 16, "kept bake taken whole");
        // Its cells as kept count, the same as counted afresh (and as the first bake's).
        let (kept_count, counted) = again.bake.lock().map(|bake| (bake.cells(), bake.cells_counted())).expect("the bake");
        assert_eq!(kept_count, counted, "the cells kept as they come, the same as counted");
        assert_eq!(kept_count, baking.bake.lock().expect("the bake").cells_counted(), "the same as the first bake's");
        drop(jobs);
        crate::sight::bake::stop(&again);
        let (other, jobs) = start(maker.clone(), with.clone(), odd, half, (0, 0), 78, read(&folder).expect("read"));
        assert!(other.bake.lock().expect("the bake").regions_done < 16, "another stamp: baked again");
        crate::sight::bake::stop(&other);
        drop(jobs);
        // Kept on the loading lane instead: the same file.
        let lane = folder.join("lane");
        std::fs::create_dir_all(&lane).expect("a folder");
        crate::sight::bake::keep_later(&baking, lane.clone());
        let waited = std::time::Instant::now();
        while (baking.writing() || !lane.join(crate::sight::bake::FILE).exists()) && waited.elapsed().as_secs() < 60 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(read(&lane).expect("read").expect("a file").records, kept.records, "kept on the lane, the same");
        // Cut short mid-record (the game closed while writing): read back as far as it's whole.
        let bytes = std::fs::read(folder.join(crate::sight::bake::FILE)).expect("the file");
        std::fs::write(folder.join(crate::sight::bake::FILE), &bytes[..bytes.len() - 100]).expect("cut short");
        let cut = read(&folder).expect("read").expect("a file");
        assert!(!cut.whole && cut.records.len() == 15 && cut.records[..] == kept.records[..15], "cut short: the whole records before the cut");
        let _ = std::fs::remove_dir_all(&folder);

        // An edit: a hole 3 by 3 and 6 deep at the north edge of the chunk at (576, 288), in its ground.
        let pos = ChunkPos::new(576 / 32, 288 / 32);
        let mut chunk = map.chunk(pos, &blocks, &plan).expect("the chunk");
        let (x0, y0) = (576 + 29, 288 + 10);
        let ground = (0..crate::world::WORLD_HEIGHT).rev().find(|&z| chunk.block_at(x0, y0, z) != BlockId::AIR && chunk.block_at(x0, y0, z) != blocks.water).expect("ground");
        let mut dug = 0u64;
        for x in x0..x0 + 3 {
            for y in y0..y0 + 3 {
                for z in ground - 5..=ground {
                    let old = chunk.block_at(x, y, z);
                    if old != BlockId::AIR && old != blocks.water {
                        chunk.set_block_at(x, y, z, BlockId::AIR);
                        dug += 1;
                    }
                }
            }
        }
        let region = (pos.x.div_euclid(32), pos.y.div_euclid(32));
        let before = bake_region(&maker, &with, odd, region);
        let edited = Blocks { changed: std::sync::Arc::new([(pos, std::sync::Arc::new(chunk))].into_iter().collect()), ..with.clone() };
        let after = bake_region(&maker, &edited, odd, region);
        let solid = |column: &crate::sight::store::CellColumn, size: i32| (0..crate::world::WORLD_HEIGHT / size).map(|z| column.at(z, size).solid).sum::<u64>();
        let top_size = 1 << crate::sight::bake::REGION;
        assert_eq!(solid(&before.top, top_size) - solid(&after.top, top_size), dug, "the region's top cell {dug} blocks lighter");
        // The cell column beside the hole (north, across the chunk's edge) shows more faces; one far off is as it was.
        let across = 1 << (crate::sight::bake::REGION - FIRST);
        let index = |cx: i32, cy: i32| ((cy - region.1 * across) * across + (cx - region.0 * across)) as usize;
        let faces = |record: &crate::sight::bake::Record, cx: i32, cy: i32| record.levels[0][index(cx, cy)].1.iter().map(|c| c.showing as u64).sum::<u64>();
        assert!(faces(&after, pos.x + 1, pos.y) > faces(&before, pos.x + 1, pos.y), "the hole bares faces in the column beside it");
        let far = (pos.x - 8, pos.y - 8);
        assert_eq!(before.levels[0][index(far.0, far.1)], after.levels[0][index(far.0, far.1)], "a column away from the edit as it was");
        // Patched in the bake itself: its region replaced whole, nothing left behind.
        let cells_before = baking.bake.lock().expect("the bake").cells();
        let job = crate::sight::bake::patch(baking.clone(), maker.clone(), edited.clone(), odd, vec![region]);
        let waited = std::time::Instant::now();
        while matches!(job.ready(), Ok(None)) && waited.elapsed().as_secs() < 120 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let cells_after = baking.bake.lock().expect("the bake").cells();
        let region_cells = |record: &crate::sight::bake::Record| record.levels.iter().flatten().map(|(_, c)| c.len()).sum::<usize>() as i64;
        assert_eq!(cells_after as i64 - cells_before as i64 - (region_cells(&after) - region_cells(&before)), 0, "the patched region replaced, its old cells gone (and the levels above remade in place)");
        eprintln!("Sight's bake patched: {dug} blocks dug, the region's top cell that much lighter; {cells_before} cells before, {cells_after} after");
    }

    /// Edits in the live levels (build 88): a hole 3 by 3 and 6 deep at a chunk's north edge, and a pillar of 3
    /// blocks stood on the ground. At every live level (cells of 1 to 16), each tile the edits reach, made on the
    /// card and patched, holds exactly the cells the edited blocks make (round the edits, above and below them,
    /// the card's cells next to the patched ones included), shows the hole and the pillar where the recipe
    /// alone doesn't, and a single block's cell showing a face knows its block (a buried one's isn't kept).
    #[test]
    fn sight_s_live_levels_show_edits() {
        use crate::sight::bake::Blocks;
        use crate::sight::cell::sums_of_box;
        use crate::sight::edits::{Edits, Reader, patch_tile};
        use crate::sight::store::{TILE, TileKey, fill_tile};
        use crate::world::{BlockId, ChunkPos, WORLD_HEIGHT};
        let Some((device, queue, map, plan, blocks, colours, world)) = sight_world() else {
            eprintln!("No graphics card: sight's live edits test is skipped");
            return;
        };
        let odd = world.odd;
        let pos = ChunkPos::new(576 / 32, 288 / 32);
        let mut chunk = map.chunk(pos, &blocks, &plan).expect("the chunk");
        let ground_at = |chunk: &crate::world::Chunk, x: i32, y: i32| (0..WORLD_HEIGHT).rev().find(|&z| chunk.block_at(x, y, z) != BlockId::AIR && chunk.block_at(x, y, z) != blocks.water).expect("ground");
        let mut spots = Vec::new();
        let (x0, y0) = (576 + 29, 288 + 10);
        let ground = ground_at(&chunk, x0, y0);
        for x in x0..x0 + 3 {
            for y in y0..y0 + 3 {
                for z in ground - 5..=ground {
                    let old = chunk.block_at(x, y, z);
                    if old != BlockId::AIR && old != blocks.water {
                        chunk.set_block_at(x, y, z, BlockId::AIR);
                        spots.push([x, y, z]);
                    }
                }
            }
        }
        let (px, py) = (576 + 6, 288 + 20);
        let top = ground_at(&chunk, px, py);
        for z in top + 1..=top + 3 {
            chunk.set_block_at(px, py, z, blocks.gabbro);
            spots.push([px, py, z]);
        }
        let map = std::sync::Arc::new(map);
        let with = Blocks { map: map.clone(), blocks: blocks.clone(), colours: colours.clone(), changed: std::sync::Arc::new([(pos, std::sync::Arc::new(chunk))].into_iter().collect()) };
        let edits = Edits { with, spots: std::sync::Arc::new([(pos, std::sync::Arc::new(spots.clone()))].into_iter().collect()), caves: std::sync::Arc::new(plan.clone()), recipe: Default::default() };
        let reader = Reader::new(&edits);
        let mut checked = 0;
        let mut took = Vec::new();
        for level in 0..5usize {
            let size = 1 << level;
            let tile_blocks = TILE << level;
            let mut keys: Vec<TileKey> = spots.iter().flat_map(|&[x, y, _]| [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dy)| TileKey { level, x: (x + dx).div_euclid(tile_blocks), y: (y + dy).div_euclid(tile_blocks) })).collect();
            keys.sort_by_key(|k| (k.x, k.y));
            keys.dedup();
            let (mut hole_seen, mut pillar_seen) = (false, false);
            let mut patching = 0.0;
            for key in keys {
                let recipe = fill_tile(&device, &queue, &world, &plan, odd, key);
                let mut tile = recipe.clone();
                let started = std::time::Instant::now();
                let redone = patch_tile(&mut tile, key, &edits);
                patching += started.elapsed().as_secs_f32() * 1000.0;
                let (tx, ty, _) = key.blocks();
                for (i, column) in tile.columns.iter().enumerate() {
                    let (cx, cy) = (tx / size + i as i32 % TILE, ty / size + i as i32 / TILE);
                    let (bx, by) = (cx * size, cy * size);
                    let near = spots.iter().any(|&[x, y, _]| x + 1 >= bx && x - 1 < bx + size && y + 1 >= by && y - 1 < by + size);
                    if !near {
                        assert_eq!(*column, recipe.columns[i], "level {level}: a column the edits don't reach is the card's");
                        continue;
                    }
                    // Every cell from below the hole to above the pillar, against the edited blocks.
                    let (from, to) = ((ground - 8).div_euclid(size), (top.max(ground) + 6) / size);
                    let first: usize = tile.columns[..i].iter().map(|c| c.sums.len()).sum();
                    for z in from..=to {
                        let want = sums_of_box([bx, by, z * size], size, &|p| reader.seen(p), &colours);
                        assert_eq!(column.at(z, size), want, "level {level}, cell ({cx}, {cy}, {z}) in tile ({}, {})", key.x, key.y);
                        if level == 0 && want.showing() > 0 && z >= column.low && z <= column.high() {
                            let block = reader.block([bx, by, z]);
                            let solid = block != BlockId::AIR && block != blocks.water;
                            assert_eq!(tile.blocks[first + (z - column.low) as usize], if solid { block.0 } else { 0 }, "level 0, block ({bx}, {by}, {z}) knows its block");
                        }
                        let was = recipe.columns[i].at(z, size);
                        if want != was {
                            if bx <= x0 + 2 && bx + size > x0 && by <= y0 + 2 && by + size > y0 && want.solid < was.solid {
                                hole_seen = true;
                            }
                            if bx <= px && bx + size > px && by <= py && by + size > py && want.solid > was.solid {
                                pillar_seen = true;
                            }
                        }
                        checked += 1;
                    }
                }
                assert!(redone > 0, "level {level}: tile ({}, {}) has cells redone", key.x, key.y);
            }
            took.push(format!("cells of {size} {patching:.1} ms"));
            assert!(hole_seen && pillar_seen, "level {level}: the hole ({hole_seen}) and the pillar ({pillar_seen}) show where the recipe alone doesn't");
        }
        eprintln!("Sight's live levels show edits: {} blocks edited, {checked} cells checked at five levels; patching took {} (the recipe's chunks made the first time)", spots.len(), took.join(", "));
    }

    /// A made-up world for sight's drawing tests: a plain map of `width` × `width` pixels of 64 blocks whose
    /// ground is exactly `height(pixel x)` (every part of the recipe muted), baked whole onto a store on the
    /// card. The card, the store, and the map.
    fn sight_scene(blocks_per_pixel: f32, height: impl Fn(usize) -> f32) -> Option<(wgpu::Device, wgpu::Queue, std::sync::Arc<crate::sight::card_store::CardStore>, crate::world::CompiledMap)> {
        sight_scene_with(blocks_per_pixel, height, None)
    }

    /// The same, and with `near` (a place, x and y) the blocks themselves round it as well: the live level of
    /// single blocks, 128 a side, made there (as the game makes it round the eye).
    fn sight_scene_with(blocks_per_pixel: f32, height: impl Fn(usize) -> f32, near: Option<(i32, i32)>) -> Option<(wgpu::Device, wgpu::Queue, std::sync::Arc<crate::sight::card_store::CardStore>, crate::world::CompiledMap)> {
        sight_scene_map(blocks_per_pixel, |column, _row| height(column), near)
    }

    /// The same, the ground's height by the map pixel's column (east) and row (north).
    fn sight_scene_map(blocks_per_pixel: f32, height: impl Fn(usize, usize) -> f32, near: Option<(i32, i32)>) -> Option<(wgpu::Device, wgpu::Queue, std::sync::Arc<crate::sight::card_store::CardStore>, crate::world::CompiledMap)> {
        let live: &[usize] = if near.is_some() { &[0, 4] } else { &[4] };
        sight_scene_levels(blocks_per_pixel, height, near, live, &[0])
    }

    /// The same, with the live levels `live` kept and those of them in `fill` made round `near` (as the game makes
    /// them round the eye: the blocks, and the cells of 2, 4, 8 and 16).
    fn sight_scene_levels(blocks_per_pixel: f32, height: impl Fn(usize, usize) -> f32, near: Option<(i32, i32)>, live: &[usize], fill: &[usize]) -> Option<(wgpu::Device, wgpu::Queue, std::sync::Arc<crate::sight::card_store::CardStore>, crate::world::CompiledMap)> {
        use crate::sight::cell::FarColours;
        use crate::sight::store::Maker;
        use crate::world::{BlockId, CavePlan, CompiledMap, Mutes, TerrainBlocks};
        use std::collections::HashMap;
        let (device, queue) = crate::whole::shader::card()?;
        let width = 48;
        let mut map = CompiledMap::of_heights(width, blocks_per_pixel, &(0..width * width).map(|i| height(i % width, i / width)).collect::<Vec<f32>>());
        map.mutes = Mutes { lift: true, mountains: true, raised_coast: true, ridges: true, torn_coast: true, detail: true, peaks: true, crags: true, summit_crags: true, rivers: true, ..Mutes::default() };
        let names = ["incus:gabbro", "incus:stone", "incus:grass", "incus:water", "incus:dirt", "incus:sand", "incus:dry_grass", "incus:snowy_ground", "incus:gravel"];
        let blocks = TerrainBlocks { gabbro: BlockId(1), stone: BlockId(2), grass: BlockId(3), water: BlockId(4), names: std::sync::Arc::new(names.iter().enumerate().map(|(k, n)| (n.to_string(), BlockId(k as u16 + 1))).collect::<HashMap<String, BlockId>>()) };
        // Grass a mid green, stone a grey, water a blue (1,024ths of linear light).
        let colours = FarColours { blocks: (0..=names.len() as u64).map(|k| if k == 3 { ([90, 220, 60], [120, 130, 90]) } else if k == 4 { ([40, 120, 300], [40, 120, 300]) } else { ([300, 290, 270], [260, 250, 235]) }).collect(), water: blocks.water, media: Vec::new() };
        let plan = CavePlan::default();
        let (world, _) = super::send(&device, &queue, &map, &plan, &blocks, &colours, (0, 0)).ok()?;
        let half = (map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0).ceil() as i32;
        let store = std::sync::Arc::new(crate::sight::card_store::CardStore::new(&device, &queue, half, live, if near.is_some() { 128 } else { 64 }));
        let odd = world.odd;
        if let Some((x, y)) = near {
            use crate::sight::store::{TILE, TileKey, compact_tile, fill_tile};
            for &level in fill {
                let (tx, ty) = (x.div_euclid(TILE << level), y.div_euclid(TILE << level));
                let reach = 64 / TILE;
                for kx in tx - reach..tx + reach {
                    for ky in ty - reach..ty + reach {
                        let key = TileKey { level, x: kx, y: ky };
                        let tile = fill_tile(&device, &queue, &world, &plan, odd, key);
                        store.put_tile(level, kx, ky, TILE as u32, &compact_tile(&tile, key.size() as u32));
                    }
                }
            }
        }
        let maker = Maker { device: device.clone(), queue: queue.clone(), world: std::sync::Arc::new(world), caves: std::sync::Arc::new(plan), card: Some(store.clone()) };
        let with = crate::sight::bake::Blocks { map: std::sync::Arc::new(map.clone()), blocks, colours, changed: Default::default() };
        let (baking, jobs) = crate::sight::bake::start(maker, with, odd, half, (0, 0), 1, None);
        let started = std::time::Instant::now();
        while baking.bake.lock().map(|b| b.all_done.is_none()).unwrap_or(false) && started.elapsed().as_secs() < 300 {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        drop(jobs);
        Some((device, queue, store, map))
    }

    /// The march drawn off the screen, `size` pixels, from `eye`: the picture's red, green and blue, row by row.
    fn sight_picture(device: &wgpu::Device, queue: &wgpu::Queue, store: &std::sync::Arc<crate::sight::card_store::CardStore>, size: (u32, u32), eye: crate::sight::draw::Eye) -> Vec<[u8; 3]> {
        sight_picture_with(device, queue, store, size, eye, None)
    }

    /// The same, with block textures.
    fn sight_picture_with(device: &wgpu::Device, queue: &wgpu::Queue, store: &std::sync::Arc<crate::sight::card_store::CardStore>, size: (u32, u32), eye: crate::sight::draw::Eye, textures: Option<&crate::sight::textures::BlockTextures>) -> Vec<[u8; 3]> {
        sight_picture_full(device, queue, store, size, eye, textures, None)
    }

    /// The same, with objects drawn into it (build 96).
    fn sight_picture_full(device: &wgpu::Device, queue: &wgpu::Queue, store: &std::sync::Arc<crate::sight::card_store::CardStore>, size: (u32, u32), eye: crate::sight::draw::Eye, textures: Option<&crate::sight::textures::BlockTextures>, objects: Option<crate::sight::models::ObjectsFrame>) -> Vec<[u8; 3]> {
        let mut draw = crate::sight::draw::SightDraw::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
        if let Some(textures) = textures {
            draw.set_textures(device, queue, textures, 1);
        }
        if let Some(objects) = objects {
            draw.objects.set(objects);
        }
        sight_picture_by(&mut draw, device, queue, store, size, eye)
    }

    /// A picture by a drawer already made (its march, textures and objects as it has them).
    fn sight_picture_by(draw: &mut crate::sight::draw::SightDraw, device: &wgpu::Device, queue: &wgpu::Queue, store: &std::sync::Arc<crate::sight::card_store::CardStore>, size: (u32, u32), eye: crate::sight::draw::Eye) -> Vec<[u8; 3]> {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("a test picture"),
            size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let row = (size.0 * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * size.1) as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&Default::default());
        draw.frame(device, queue, &mut encoder, &view, size, eye, store);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(size.1) } },
            wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
        );
        queue.submit([encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let bytes = readback.slice(..).get_mapped_range().expect("the picture").to_vec();
        (0..size.1).flat_map(|y| (0..size.0).map(move |x| (x, y))).map(|(x, y)| {
            let at = (y * row + x * 4) as usize;
            [bytes[at], bytes[at + 1], bytes[at + 2]]
        }).collect()
    }

    /// Sight over a flat sea doesn't shimmer (David, build 69: "the oceans still show the rings... so fast"):
    /// the same view from places a fraction of a step apart, each pixel's change measured. Over a sea all
    /// alike, a true picture barely changes; rings that move with the eye change it a lot.
    #[test]
    #[ignore]
    fn sight_over_a_flat_sea_holds_still() {
        // 24 km across, so the view crosses where the store's levels change (rings that move with the eye).
        let Some((device, queue, store, _map)) = sight_scene(512.0, |_| 70.0) else {
            eprintln!("No graphics card: sight's sea test is skipped");
            return;
        };
        let size = (640, 400);
        let pitch = -40f32.to_radians();
        // Up by a fraction of a block at a time (the cells' layering is what a ray's steps meet).
        let eye = |z: f32| crate::sight::draw::Eye { position: [0.0, 0.0, z], looking: [pitch.cos(), 0.0, pitch.sin()], field_of_view: 100.0, grid: 2.0, coarsest: 8, rays: 1, upscale: 0, earned: true, sun: crate::sight::draw::NOON, shadows: false, sky: false, fog: false, ..Default::default() };
        let pictures: Vec<Vec<[u8; 3]>> = (0..6).map(|k| sight_picture(&device, &queue, &store, size, eye(400.0 + k as f32 * 1.37))).collect();
        // From just under the horizon (row 32 at 40° down) to the bottom: all sea.
        let first_row = 40u32;
        let from = (first_row * size.0) as usize;
        let mut changes = Vec::new();
        for pair in pictures.windows(2) {
            let mut total = 0u64;
            let mut most = 0u32;
            for (a, b) in pair[0][from..].iter().zip(&pair[1][from..]) {
                let d = (0..3).map(|c| (a[c] as i32 - b[c] as i32).unsigned_abs()).max().unwrap_or(0);
                total += d as u64;
                most = most.max(d);
            }
            changes.push((total as f64 / (pair[0].len() - from) as f64, most));
        }
        // Down the middle, the sea's brightness should change smoothly with distance: its turns counted (rings
        // are wiggles).
        let light = |p: [u8; 3]| p[0] as i32 * 2 + p[1] as i32 * 7 + p[2] as i32;
        let column: Vec<i32> = (first_row..size.1).map(|y| light(pictures[0][(y * size.0 + size.0 / 2) as usize])).collect();
        let mut turns = 0;
        let mut last_sign = 0;
        for pair in column.windows(2) {
            let sign = (pair[1] - pair[0]).signum();
            if sign != 0 {
                if last_sign != 0 && sign != last_sign {
                    turns += 1;
                }
                last_sign = sign;
            }
        }
        // Where the biggest changes are: rows with a pixel changing by more than 4.
        let mut rows = std::collections::BTreeMap::new();
        for (k, (a, b)) in pictures[0].iter().zip(&pictures[1]).enumerate().skip(from) {
            if (0..3).any(|c| (a[c] as i32 - b[c] as i32).abs() > 4) {
                *rows.entry(k as u32 / size.0).or_insert(0) += 1;
            }
        }
        // Down the middle, the biggest steps in brightness from one row to the next (a seam is a step).
        let mut steps: Vec<(i32, u32)> = (first_row..size.1 - 1).map(|y| ((light(pictures[0][((y + 1) * size.0 + size.0 / 2) as usize]) - light(pictures[0][(y * size.0 + size.0 / 2) as usize])).abs(), y)).collect();
        steps.sort();
        eprintln!("Sight over the sea: the biggest steps down the middle (brightness out of 2,550, row): {:?}", &steps[steps.len().saturating_sub(8)..]);
        eprintln!("Sight over the sea: rows with a pixel changing by more than 4 (row: how many): {rows:?}");
        eprintln!("Sight over the sea: {turns} turns in the brightness down the middle; each rise of 1.37 blocks changes a pixel by (mean, most of 255) {:?}", changes);
    }

    /// Sight's sun shadows fall where they should (3.4b): a long ridge on a plain, a low sun beyond it darkens the
    /// plain on this side with shadows on; with the sun on this side, nothing shades the plain (no surface shades
    /// itself: the first try did, the shadow ray stepping clear by a block where the cells read were 32).
    #[test]
    #[ignore]
    fn sight_s_shadows_fall_where_they_should() {
        // The plain at 150, the ridge at 320 across the map's columns 30 to 33: running north and south, 384 to 640
        // blocks east of the middle (a map's columns go east).
        let Some((device, queue, store, _map)) = sight_scene(64.0, |x| if (30..34).contains(&x) { 320.0 } else { 150.0 }) else {
            eprintln!("No graphics card: sight's shadow test is skipped");
            return;
        };
        let size = (320, 200);
        // The sun 15° up, from the east (beyond the ridge: its shadow reaches about 630 west of it, past the middle)
        // or the west.
        let low = 15f32.to_radians();
        let beyond = [0.0, low.cos(), low.sin()];
        let behind = [0.0, -low.cos(), low.sin()];
        let middle = |height: f32, pitch: f32, sun: [f32; 3], shadows: bool| {
            let pitch = pitch.to_radians();
            let eye = crate::sight::draw::Eye { position: [0.0, 0.0, height], looking: [pitch.cos(), 0.0, pitch.sin()], field_of_view: 60.0, grid: 2.0, coarsest: 8, rays: 1, upscale: 0, earned: true, sun, shadows, sky: false, fog: false, ..Default::default() };
            let picture = sight_picture(&device, &queue, &store, size, eye);
            let p = picture[(size.1 / 2 * size.0 + size.0 / 2) as usize];
            p[0] as f32 * 0.2 + p[1] as f32 * 0.7 + p[2] as f32 * 0.1
        };
        // Close by: 100 over the plain looking 60° down (the ground 58 north, about 115 off). Farther: 350 over it
        // looking 45° down (the ground 350 north, about 500 off). Both read the cells (this store's live levels
        // aren't filled: the bake's 32 and up); the blocks' walk towards the sun is for David's eye in play.
        for (name, height, pitch) in [("close by", 250.0, -60.0), ("farther", 500.0, -45.0)] {
            let (lit, shaded) = (middle(height, pitch, beyond, false), middle(height, pitch, beyond, true));
            let (lit_behind, shaded_behind) = (middle(height, pitch, behind, false), middle(height, pitch, behind, true));
            eprintln!("Sight's shadows, {name}: sun beyond the ridge {lit:.1} → {shaded:.1}; on this side {lit_behind:.1} → {shaded_behind:.1}");
            assert!(shaded < lit * 0.9, "{name}: the ridge's shadow should darken the plain ({lit:.1} → {shaded:.1})");
            assert!((shaded_behind - lit_behind).abs() <= 1.0, "{name}: with the sun on this side nothing shades the plain ({lit_behind:.1} → {shaded_behind:.1})");
        }
    }

    /// Sight's sky light (3.4b): the plain at the foot of a ridge sees less of the sky than the open plain, and
    /// is darker for it; the open plain hardly changes; and the tint makes the sky's part bluer, not darker.
    #[test]
    #[ignore]
    fn sight_s_sky_light_darkens_the_foot_of_a_ridge() {
        // The ridge test's scene: the plain at 150, a ridge at 320 from 384 to 640 east.
        let Some((device, queue, store, _map)) = sight_scene(64.0, |x| if (30..34).contains(&x) { 320.0 } else { 150.0 }) else {
            eprintln!("No graphics card: sight's sky light test is skipped");
            return;
        };
        let size = (320, 200);
        let pitch = -60f32.to_radians();
        // The middle pixel from 100 over the plain at `east`, looking north 60° down (the ground 58 north of it).
        let middle = |east: f32, sky: bool, tint: f32| {
            let eye = crate::sight::draw::Eye {
                position: [0.0, east, 250.0],
                looking: [pitch.cos(), 0.0, pitch.sin()],
                field_of_view: 60.0,
                grid: 2.0,
                coarsest: 8,
                shadows: false,
                sky,
                sky_strength: 1.0,
                sky_tint: tint,
                fog: false,
                ..Default::default()
            };
            let picture = sight_picture(&device, &queue, &store, size, eye);
            picture[(size.1 / 2 * size.0 + size.0 / 2) as usize].map(|c| c as f32)
        };
        let light = |p: [f32; 3]| p[0] * 0.2 + p[1] * 0.7 + p[2] * 0.1;
        // Out on the plain, nearly 1,000 from the ridge; at its foot, 54 from its face (it fills the sky's east).
        let (open_off, open_on) = (light(middle(-600.0, false, 0.0)), light(middle(-600.0, true, 0.0)));
        let (foot_off, foot_on) = (light(middle(330.0, false, 0.0)), light(middle(330.0, true, 0.0)));
        eprintln!("Sight's sky light: the open plain {open_off:.1} → {open_on:.1}; the ridge's foot {foot_off:.1} → {foot_on:.1}");
        assert!(open_on > open_off * 0.95, "the open plain sees nearly all the sky ({open_off:.1} → {open_on:.1})");
        assert!(foot_on / foot_off < open_on / open_off - 0.03, "the ridge's foot sees less of it ({foot_off:.1} → {foot_on:.1})");
        // The tint: the sky's part bluer, the brightness much the same.
        let (white, blue) = (middle(330.0, true, 0.0), middle(330.0, true, 1.0));
        eprintln!("Sight's sky light: white {white:?}, tinted {blue:?}");
        assert!(blue[2] / blue[1] > white[2] / white[1], "tinted, the shade is bluer");
        assert!((light(blue) - light(white)).abs() < light(white) * 0.1, "tinted, not much darker or brighter");
    }

    /// Sight's darkness (David 2026-11-09, before lights): with the sun 5° up the plain is lit as it always was;
    /// with it 30° under the horizon, the plain and the sky go to night's light (a twenty-fifth of the day's sky
    /// light: very dark), and with no night light, black.
    #[test]
    #[ignore]
    fn sight_s_night_is_dark() {
        let Some((device, queue, store, _map)) = sight_scene(64.0, |_| 150.0) else {
            eprintln!("No graphics card: sight's night test is skipped");
            return;
        };
        let size = (320, 200);
        let sun = |degrees: f32| {
            let up = degrees.to_radians();
            [up.cos() * -0.6, up.cos() * 0.8, up.sin()]
        };
        // The middle pixel looking 30° down at the plain, and one looking 30° up at the sky.
        let middle = |pitch: f32, height: f32, night_light: f32| {
            let pitch = pitch.to_radians();
            let eye = crate::sight::draw::Eye {
                position: [0.0, 0.0, 250.0],
                looking: [pitch.cos(), 0.0, pitch.sin()],
                field_of_view: 60.0,
                grid: 2.0,
                coarsest: 8,
                sun: sun(height),
                night_light,
                shadows: true,
                fog: false,
                ..Default::default()
            };
            let picture = sight_picture(&device, &queue, &store, size, eye);
            let p = picture[(size.1 / 2 * size.0 + size.0 / 2) as usize];
            p[0] as f32 * 0.2 + p[1] as f32 * 0.7 + p[2] as f32 * 0.1
        };
        let (day, night, black) = (middle(-30.0, 5.0, 0.04), middle(-30.0, -30.0, 0.04), middle(-30.0, -30.0, 0.0));
        let (sky_day, sky_night) = (middle(30.0, 5.0, 0.04), middle(30.0, -30.0, 0.04));
        eprintln!("Sight's night: the plain {day:.1} by day, {night:.1} at night, {black:.1} with no night light; the sky {sky_day:.1} → {sky_night:.1}");
        assert!(day > 40.0, "by day the plain is lit ({day:.1})");
        assert!(night < day * 0.3 && night < 40.0, "at night it's dark ({day:.1} → {night:.1})");
        assert!(black <= 1.0, "with no night light, black ({black:.1})");
        assert!(sky_night < sky_day * 0.3, "the sky dark too ({sky_day:.1} → {sky_night:.1})");
    }

    /// A picture written out as a PNG, for looking at (probes: `INCUS_PROBE_DIR`).
    fn write_picture(path: &std::path::Path, size: (u32, u32), picture: &[[u8; 3]]) {
        let file = std::fs::File::create(path).expect("the file");
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), size.0, size.1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("the header");
        writer.write_image_data(&picture.concat()).expect("the picture");
    }

    /// Sight up against a face (David after build 84: "when you put your face right into a block, a circle appears
    /// as you get very close like this"): the eye a hair over a sunlit plain, straight down and at a slant, sees the
    /// same even grass it sees from two blocks up, with shadows and sky light on. Every ray's walk began half a
    /// block out, so a face nearer than that was met inside its block, facing the eye: a dark disc, cut into wedges.
    #[test]
    #[ignore]
    fn sight_s_a_face_up_close_is_lit_evenly() {
        let Some((device, queue, store, _map)) = sight_scene_with(64.0, |_| 150.0, Some((0, 0))) else {
            eprintln!("No graphics card: sight's face-up-close test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let size = (320, 200);
        let low = 40f32.to_radians();
        let sun = [low.cos() * 0.6, low.cos() * 0.8, low.sin()];
        // The plain's top is at 151.
        let picture = |above: f32, pitch: f32| {
            let down = pitch.to_radians();
            let eye = crate::sight::draw::Eye { position: [0.3, 0.4, 151.0 + above], looking: [down.cos(), 0.0, down.sin()], field_of_view: 90.0, grid: 2.0, coarsest: 8, sun, shadows: true, sky: true, fog: false, ..Default::default() };
            sight_picture(&device, &queue, &store, size, eye)
        };
        let far = picture(2.0, -89.9);
        let lit = far[(size.1 / 2 * size.0 + size.0 / 2) as usize];
        for (above, pitch) in [(0.02, -89.9), (0.1, -89.9), (0.3, -89.9), (0.1, -60.0), (0.3, -60.0)] {
            let near = picture(above, pitch);
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("up_close_{above}_{}.png", -pitch as i32)), size, &near);
            }
            let off = near.iter().map(|p| (0..3).map(|c| (p[c] as i32 - lit[c] as i32).abs()).max().unwrap_or(0)).max().unwrap_or(0);
            eprintln!("Sight up close, {above} over the plain looking {}° down: the most any pixel is off the plain's colour from 2 up {off} (of 255)", -pitch);
            assert!(off <= 3, "{above} over the plain looking {}° down: a pixel {off} off the plain's own colour {lit:?}", -pitch);
        }
    }

    /// Sight from high up (David after build 84: "the circles of light that swallow the world if i fly tens of miles
    /// over the surface straight up looking down"): over hills and sea, from 2,000 to 60,000 up, straight down and
    /// at a slant, every pixel over the map is ground, none the sky's colour. A far cell is an average (a cell of
    /// 1,024 over 150 blocks of ground about a seventh solid): a ray looking down through it gathered a seventh of
    /// the ground and went out through the world's floor, the rest the sky's colour, beyond the distance where the
    /// cells grew that coarse; and where they were as tall as all the ground, a stride from over its top landed
    /// under the floor, a hole of sky.
    #[test]
    #[ignore]
    fn sight_from_high_up_the_ground_stays_ground() {
        let Some((device, queue, store, _map)) = sight_scene(1024.0, |x| 150.0 + 120.0 * (x as f32 * 0.9).sin()) else {
            eprintln!("No graphics card: sight's from-high-up test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let size = (320, 200);
        let low = 40f32.to_radians();
        let sun = [low.cos() * 0.6, low.cos() * 0.8, low.sin()];
        // The sky's colour looking down (what leaked through): from 30,000 up looking 50° down, the top corner looks
        // past the map's far edge, where there's nothing.
        let slant = (-50f32).to_radians();
        let past = crate::sight::draw::Eye { position: [0.3, 0.4, 30000.0], looking: [slant.cos(), 0.0, slant.sin()], field_of_view: 90.0, grid: 2.0, coarsest: 8, sun, fog: false, ..Default::default() };
        let sky_blue = sight_picture(&device, &queue, &store, size, past)[(2 * size.0 + 2) as usize];
        // Over the map (49,152 a side): only the ground's own pixels, not the sky past its edge.
        let half = 24_000.0;
        for (height, pitch) in [(2000.0, -89.9), (10000.0, -89.9), (30000.0, -89.9), (60000.0, -89.9), (20000.0, -50.0), (30000.0, -50.0), (40000.0, -50.0)] {
            let down = f32::to_radians(pitch);
            let eye = crate::sight::draw::Eye { position: [0.3, 0.4, height], looking: [down.cos(), 0.0, down.sin()], field_of_view: 90.0, grid: 2.0, coarsest: 8, sun, shadows: true, sky: true, fog: false, ..Default::default() };
            let picture = sight_picture(&device, &queue, &store, size, eye);
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("high_up_{height}_{}.png", -pitch as i32)), size, &picture);
            }
            // Each pixel's ray, where it comes down to the ground's middle height (150), if that's over the map.
            let forward = [down.cos(), 0.0, down.sin()];
            let (right, upward) = ([0.0, 1.0, 0.0], [-down.sin(), 0.0, down.cos()]);
            let aspect = size.0 as f32 / size.1 as f32;
            let mut sky_like = 0;
            let mut over_map = 0;
            for y in 0..size.1 {
                for x in 0..size.0 {
                    let (sx, sy) = (((x as f32 + 0.5) / size.0 as f32 * 2.0 - 1.0) * aspect, 1.0 - (y as f32 + 0.5) / size.1 as f32 * 2.0);
                    let d: Vec<f32> = (0..3).map(|k| forward[k] + sx * right[k] + sy * upward[k]).collect();
                    if d[2] >= -1e-3 {
                        continue;
                    }
                    let t = (height - 150.0) / -d[2];
                    let (gx, gy) = (0.3 + d[0] * t, 0.4 + d[1] * t);
                    // Well inside the map (a coarse pixel's width from its edge).
                    if gx.abs() > half * 0.9 || gy.abs() > half * 0.9 {
                        continue;
                    }
                    over_map += 1;
                    let p = picture[(y * size.0 + x) as usize];
                    if (0..3).all(|c| (p[c] as i32 - sky_blue[c] as i32).abs() <= 12) {
                        sky_like += 1;
                    }
                }
            }
            eprintln!("Sight from {height} up looking {}° down: {sky_like} of {over_map} pixels over the map the sky's colour {sky_blue:?}", -pitch);
            assert!(over_map > 1000, "the view takes in the map");
            assert_eq!(sky_like, 0, "from {height} up looking {}° down, {sky_like} pixels over the map are the sky's colour", -pitch);
        }
    }

    /// Over nothing, a ray down through the world's floor stays sky (David on 93: "on the peaks at a distance, a band
    /// juts out from either side... the band slides up and down the tip with me"): 93's floor rule took all of what a
    /// ray going down through the floor hadn't gathered as what it had, so rays a little under the horizon that
    /// brushed a peak's soft edge (or the plain's) and went on over regions not baked yet were filled with that brush.
    /// Here the plain ends at the map's edge: from 300 up (the plain at 131) the rays under the horizon that miss the
    /// plain go on past the map, over nothing, and stay the sky's colour; only ground gathered just over the floor
    /// fills a ray.
    #[test]
    #[ignore]
    fn sight_over_nothing_the_floor_stays_sky() {
        let Some((device, queue, store, _map)) = sight_scene_map(64.0, |column, row| if column == 24 && row == 36 { 330.0 } else if (23..26).contains(&column) && (35..38).contains(&row) { 290.0 } else { 131.0 }, None) else {
            eprintln!("No graphics card: sight's over-nothing test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let size = (320, 200);
        let low = 40f32.to_radians();
        let sun = [low.cos() * 0.6, low.cos() * 0.8, low.sin()];
        let eye = crate::sight::draw::Eye { position: [-1500.0, 32.0, 300.0], looking: [1.0, 0.0, 0.0], field_of_view: 40.0, grid: 2.0, coarsest: 8, sun, fog: false, ..Default::default() };
        let picture = sight_picture(&device, &queue, &store, size, eye);
        if let Some(dir) = &probe {
            write_picture(&dir.join("over_nothing.png"), size, &picture);
        }
        // The map's edge is 3,036 ahead: rays down to 0.056 under level (170 down to the plain over that) miss the
        // plain. Rows 2 to 10 under the horizon (row 100; the view's half height is tan 20° = 0.364), away from the
        // peak (the middle third): each pixel against the sky's colour 2 rows over the horizon.
        let mut most = 0;
        for row in 102..110u32 {
            for column in (8..100u32).chain(220..312) {
                let p = picture[(row * size.0 + column) as usize];
                let sky = picture[(98 * size.0 + column) as usize];
                most = most.max((0..3).map(|c| (p[c] as i32 - sky[c] as i32).abs()).max().unwrap_or(0));
            }
        }
        eprintln!("Sight over nothing: just under the horizon, away from the peak, the most a pixel is off the sky's colour over it {most} (of 255)");
        assert!(most <= 24, "rays down through the floor over nothing should stay sky ({most} off)");
    }

    /// Placed light (build 95): at night, a green light a block over a plain lights the ground round it green, the
    /// ground past its reach stays dark, and with placed lights off it's all dark. The light flooded on the processor
    /// (`sight/lights.rs`) and put on the card's table, as the game does.
    #[test]
    #[ignore]
    fn sight_s_placed_light_lights_the_ground_round_it() {
        use crate::sight::lights::{LightFrom, PlacedLights};
        let Some((device, queue, store, _map)) = sight_scene_with(64.0, |_| 150.0, Some((0, 0))) else {
            eprintln!("No graphics card: sight's placed light test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        // The plain's top is at 151: the light in the air just over it, green, reaching 8.
        let mut lights = PlacedLights::default();
        lights.set(&[LightFrom { object: [0, 0, 151 * 16], cell: [0, 0, 151], colour: [0, 15, 0], reach: 8 }], &|at: [i32; 3]| at[2] < 151);
        let lit = lights.lit();
        assert_eq!(store.put_lights(&lit), lit.len());
        let size = (320, 200);
        let night = (-30f32).to_radians();
        let sun = [night.cos() * 0.6, night.cos() * 0.8, night.sin()];
        // From 12 over the plain, 6 south of the light, looking north 60° down: the middle of the picture is the
        // ground about 1 north of the light; the top of it is the ground 15 or more off (past its reach).
        let down = (-60f32).to_radians();
        let picture = |on: bool| {
            let eye = crate::sight::draw::Eye { position: [-6.0, 0.5, 163.0], looking: [down.cos(), 0.0, down.sin()], field_of_view: 60.0, grid: 2.0, coarsest: 8, sun, night_light: 0.04, fog: false, placed_lights: on, flicker: 0.0, ..Default::default() };
            sight_picture(&device, &queue, &store, size, eye)
        };
        let (with, without) = (picture(true), picture(false));
        if let Some(dir) = &probe {
            write_picture(&dir.join("placed_light_on.png"), size, &with);
            write_picture(&dir.join("placed_light_off.png"), size, &without);
        }
        let middle = |p: &Vec<[u8; 3]>| p[(size.1 / 2 * size.0 + size.0 / 2) as usize];
        let top = |p: &Vec<[u8; 3]>| p[(4 * size.0 + size.0 / 2) as usize];
        eprintln!("Sight's placed light: by the light {:?} (off {:?}); past its reach {:?} (off {:?})", middle(&with), middle(&without), top(&with), top(&without));
        let (lit_g, dark_g) = (middle(&with)[1] as i32, middle(&without)[1] as i32);
        assert!(lit_g > dark_g + 60, "the ground by the light is lit ({dark_g} → {lit_g})");
        assert!(middle(&with)[1] as i32 > middle(&with)[0] as i32 + 40, "green, as the light is");
        assert!((0..3).all(|c| (top(&with)[c] as i32 - top(&without)[c] as i32).abs() <= 2), "past its reach the light does nothing");
        assert!((0..3).all(|c| middle(&without)[c] < 40), "with placed lights off, the night's dark");
    }

    /// Objects drawn (build 96): David's brazier on the plain shows (its model and its flame), in front of the
    /// ground; the same brazier sunk under the ground doesn't (the depth the march left keeps it hidden).
    #[test]
    #[ignore]
    fn sight_draws_the_brazier_where_it_stands() {
        use crate::sight::models::{ModelDrawn, ObjectsFrame, SpriteDrawn};
        let Some((device, queue, store, _map)) = sight_scene_with(64.0, |_| 150.0, Some((0, 0))) else {
            eprintln!("No graphics card: sight's objects test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let bytes = std::fs::read(crate::data::models_folder().join("braziere.gltf")).expect("the brazier's model");
        let shape = crate::data::read_shape(&bytes, Some(1.5)).expect("its shape");
        // On the plain (its top at 151), 4 north of the eye; or sunk 6 under it.
        let brazier = |z: f32| ObjectsFrame {
            models: vec![ModelDrawn { model: "braziere.gltf".into(), feet: [4.5, 0.5, z], model_feet: [-shape.feet.z, shape.feet.x, shape.feet.y], scale: shape.scale, facing: 0, sky: 1.0, glow: [0.0; 3] }],
            sprites: vec![SpriteDrawn { strip: "green_flame_animation.png".into(), spot: [4.5, 0.5, z + 1.3], scale: 0.5, anchor: crate::data::Anchor::BottomCentre, frame_ms: 110, phase: 0 }],
            millis: 0,
            ..Default::default()
        };
        let size = (640, 400);
        let down = (-15f32).to_radians();
        let eye = crate::sight::draw::Eye { position: [2.5, 0.5, 152.6], looking: [down.cos(), 0.0, down.sin()], field_of_view: 50.0, grid: 2.0, coarsest: 8, fog: false, ..Default::default() };
        let plain = sight_picture_full(&device, &queue, &store, size, eye, None, None);
        let standing = sight_picture_full(&device, &queue, &store, size, eye, None, Some(brazier(151.0)));
        let sunk = sight_picture_full(&device, &queue, &store, size, eye, None, Some(brazier(144.0)));
        if let Some(dir) = &probe {
            write_picture(&dir.join("brazier_standing.png"), size, &standing);
            write_picture(&dir.join("brazier_sunk.png"), size, &sunk);
        }
        let changed = |a: &[[u8; 3]], b: &[[u8; 3]]| a.iter().zip(b).filter(|(a, b)| (0..3).any(|c| (a[c] as i32 - b[c] as i32).abs() > 8)).count();
        let (shown, hidden) = (changed(&plain, &standing), changed(&plain, &sunk));
        // The flame: bright green pixels.
        let flame = standing.iter().filter(|p| p[1] > 150 && p[1] as i32 > p[0] as i32 + 60).count();
        eprintln!("Sight's objects: the brazier standing changes {shown} pixels (its flame {flame}); sunk under the ground {hidden}");
        assert!(shown > 500, "the brazier shows ({shown} pixels)");
        assert!(flame > 20, "its flame shows ({flame} pixels)");
        assert_eq!(hidden, 0, "under the ground it's hidden");
    }

    /// Water seen through (build 98, generic transparency: David, "16 fade through water is good, just blue water is
    /// fine for now"): looking straight down into the sea, the bottom 3 blocks under the surface shows through,
    /// tinted; 27 under, it's gone, and the pixel is the water's own colour; and further off, where the cells are read,
    /// the same.
    #[test]
    #[ignore]
    fn sight_sees_through_water_as_deep_as_it_is() {
        // The sea's surface is at 128: the ground 125 to the west of the middle (3 under), 101 to the east (27).
        let Some((device, queue, store, _map)) = sight_scene_with(64.0, |x| if x < 24 { 125.0 } else { 101.0 }, Some((0, 0))) else {
            eprintln!("No graphics card: sight's water test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let size = (160, 100);
        let down = (-89.9f32).to_radians();
        let middle = |east: f32, height: f32| {
            let eye = crate::sight::draw::Eye { position: [0.3, east, height], looking: [down.cos(), 0.0, down.sin()], field_of_view: 30.0, grid: 2.0, coarsest: 8, shadows: false, sky: false, fog: false, ..Default::default() };
            let picture = sight_picture(&device, &queue, &store, size, eye);
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("water_{east}_{height}.png")), size, &picture);
            }
            picture[(size.1 / 2 * size.0 + size.0 / 2) as usize]
        };
        let near = |east: f32| middle(east, 140.0);
        let (shallow, deep) = (near(-30.0), near(30.0));
        // From far up: the cells read, not the blocks.
        let (far_shallow, far_deep) = (middle(-400.0, 2500.0), middle(400.0, 2500.0));
        eprintln!("Sight through water: 3 deep {shallow:?}, 27 deep {deep:?}; from 2,500 up, {far_shallow:?} and {far_deep:?}");
        let gap = |a: [u8; 3], b: [u8; 3]| (0..3).map(|c| (a[c] as i32 - b[c] as i32).abs()).max().unwrap_or(0);
        assert!(gap(shallow, deep) > 20, "the shallows show the bottom ({shallow:?} against {deep:?})");
        assert!(deep[2] > deep[0] + 20, "deep water is the water's blue ({deep:?})");
        assert!(gap(far_shallow, far_deep) > 20, "from far up too ({far_shallow:?} against {far_deep:?})");
        assert!(gap(deep, far_deep) <= 24, "deep water much the same near and far (the far cells average the sea floor in a little) ({deep:?} against {far_deep:?})");
    }

    /// Sight's block textures (3.6): up close the near faces wear their texture (here a checker of red and blue
    /// squares, four texels a side), and as a face shrinks towards a pixel it takes the texture's shrunk versions,
    /// so a face far off is the texture shrunk to one texel: the colour the cells past it are made of (a flat
    /// texture of that colour, up close, as the measure).
    #[test]
    #[ignore]
    fn sight_s_blocks_wear_their_textures() {
        use crate::sight::textures::{BlockTextures, NONE, mips};
        let Some((device, queue, store, _map)) = sight_scene_with(64.0, |_| 150.0, Some((0, 0))) else {
            eprintln!("No graphics card: sight's textures test is skipped");
            return;
        };
        let side = crate::look::TEXTURE_SIZE;
        let checker: Vec<u8> = (0..side * side).flat_map(|i| if (i % side / 4 + i / side / 4) % 2 == 0 { [200, 60, 60, 255] } else { [60, 60, 200, 255] }).collect();
        let textures = BlockTextures { layers: vec![mips(checker)], blocks: (0..16).map(|k| if k == 0 { ([NONE; 6], 0) } else { ([0; 6], 0) }).collect(), bars: Vec::new(), problems: Vec::new(), media: Vec::new() };
        let size = (320, 200);
        // From 4 over the ground, looking north `down` degrees down.
        let picture = |down: f32, on: bool, textures: &BlockTextures| {
            let pitch = -down.to_radians();
            let eye = crate::sight::draw::Eye {
                position: [0.5, 0.5, 155.0],
                looking: [pitch.cos(), 0.0, pitch.sin()],
                field_of_view: 60.0,
                grid: 2.0,
                coarsest: 8,
                shadows: false,
                sky: false,
                fog: false,
                textures: on,
                ..Default::default()
            };
            sight_picture_with(&device, &queue, &store, size, eye, Some(textures))
        };
        let linear = |c: u8| {
            let c = c as f32 / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        // The middle 32 × 32 pixels, in linear light.
        let patch = |p: &[[u8; 3]]| -> Vec<[f32; 3]> { (84..116).flat_map(|y| (144..176).map(move |x| (x, y))).map(|(x, y)| p[(y * size.0 + x) as usize].map(linear)).collect() };
        let reddish = |px: &[[f32; 3]]| px.iter().filter(|c| c[0] > c[2] * 1.5).count() as f32 / px.len() as f32;
        let bluish = |px: &[[f32; 3]]| px.iter().filter(|c| c[2] > c[0] * 1.5).count() as f32 / px.len() as f32;
        // Up close, looking 70° down: the checker shows (red squares and blue), and not without textures.
        let (near_on, near_off) = (patch(&picture(70.0, true, &textures)), patch(&picture(70.0, false, &textures)));
        eprintln!("Sight's textures up close: red {:.2}, blue {:.2}; without them red {:.2}, blue {:.2}", reddish(&near_on), bluish(&near_on), reddish(&near_off), bluish(&near_off));
        assert!(reddish(&near_on) > 0.2 && bluish(&near_on) > 0.2, "up close the checker shows");
        assert!(bluish(&near_off) < 0.01, "without textures, no blue squares");
        // About 40 off (6° down), a face is the texture shrunk to one texel: as a flat texture of that colour is
        // up close.
        let one = textures.layers[0].last().expect("its one texel").clone();
        let flat_texture = BlockTextures { layers: vec![mips((0..side * side).flat_map(|_| one.clone()).collect())], ..textures.clone() };
        let flat = patch(&picture(70.0, true, &flat_texture));
        let average = flat.iter().fold([0.0f32; 3], |a, c| [a[0] + c[0], a[1] + c[1], a[2] + c[2]]).map(|v| v / flat.len() as f32);
        let far = picture((4.0f32 / 40.0).atan().to_degrees(), true, &textures)[(size.1 / 2 * size.0 + size.0 / 2) as usize].map(linear);
        let close_up = near_on.iter().fold([0.0f32; 3], |a, c| [a[0] + c[0], a[1] + c[1], a[2] + c[2]]).map(|v| v / near_on.len() as f32);
        eprintln!("Sight's textures: a face 40 off {far:?}; its one texel up close {average:?} (the checker up close, as light, {close_up:?})");
        for c in [0, 2] {
            assert!((far[c] - average[c]).abs() < average[c] * 0.08, "far off, the texture shrunk to one texel ({far:?} against {average:?})");
        }
    }

    /// Pieces in sight (build 90): a cell 3½ blocks ahead of the eye, on the ground, as air, as a whole block, and
    /// as a slab (its lower four octants). Looking straight at it through its lower half, the slab is the whole
    /// block (the same face, texture and light); through its upper half, it's the air (the ray goes on past it).
    #[test]
    fn sight_s_pieces_show_their_octants() {
        use crate::sight::bake::Blocks;
        use crate::sight::cell::FarColours;
        use crate::sight::edits::{Edits, patch_tile, pieces_near};
        use crate::sight::store::{TILE, TileKey, compact_tile, fill_tile};
        use crate::sight::textures::{BlockTextures, mips};
        use crate::world::{BlockId, CavePlan, ChunkPos, Part, Pieces, TerrainBlocks, WORLD_HEIGHT};
        use std::collections::HashMap;
        let Some((device, queue, store, map)) = sight_scene_with(64.0, |_| 150.0, Some((0, 0))) else {
            eprintln!("No graphics card: sight's pieces test is skipped");
            return;
        };
        // The scene's world again (as `sight_scene_with` makes it), to make the tile with the cell edited.
        let names = ["incus:gabbro", "incus:stone", "incus:grass", "incus:water", "incus:dirt", "incus:sand", "incus:dry_grass", "incus:snowy_ground", "incus:gravel"];
        let blocks = TerrainBlocks { gabbro: BlockId(1), stone: BlockId(2), grass: BlockId(3), water: BlockId(4), names: std::sync::Arc::new(names.iter().enumerate().map(|(k, n)| (n.to_string(), BlockId(k as u16 + 1))).collect::<HashMap<String, BlockId>>()) };
        let colours = FarColours { blocks: (0..=names.len() as u64).map(|k| if k == 3 { ([90, 220, 60], [120, 130, 90]) } else if k == 4 { ([40, 120, 300], [40, 120, 300]) } else { ([300, 290, 270], [260, 250, 235]) }).collect(), water: blocks.water, media: Vec::new() };
        let plan = CavePlan::default();
        let (world, _) = super::send(&device, &queue, &map, &plan, &blocks, &colours, (0, 0)).expect("the world on the card");
        let pos = ChunkPos::new(0, 0);
        let chunk = map.chunk(pos, &blocks, &plan).expect("the chunk");
        let ground = (0..WORLD_HEIGHT).rev().find(|&z| chunk.block_at(4, 0, z) != BlockId::AIR).expect("ground");
        let cell = [4, 0, ground + 1];
        // One colour for every block's every face (a warm orange), so the octant's face and the block's match.
        let side = crate::look::TEXTURE_SIZE;
        let textures = BlockTextures { layers: vec![mips((0..side * side).flat_map(|_| [220u8, 120, 40, 255]).collect())], blocks: (0..16).map(|k| if k == 0 { ([crate::sight::textures::NONE; 6], 0) } else { ([0; 6], 0) }).collect(), bars: Vec::new(), problems: Vec::new(), media: Vec::new() };
        let map = std::sync::Arc::new(map);
        let key = TileKey { level: 0, x: 0, y: 0 };
        // The cell as `what` (none: air), the tile made again round it and put on the card, its pieces too.
        let show = |what: Option<crate::world::Chunk>| {
            let mut tile = fill_tile(&device, &queue, &world, &plan, world.odd, key);
            let changed: HashMap<ChunkPos, std::sync::Arc<crate::world::Chunk>> = what.into_iter().map(|c| (pos, std::sync::Arc::new(c))).collect();
            let edits = Edits { with: Blocks { map: map.clone(), blocks: blocks.clone(), colours: colours.clone(), changed: std::sync::Arc::new(changed.clone()) }, spots: std::sync::Arc::new([(pos, std::sync::Arc::new(vec![cell]))].into_iter().collect()), caves: std::sync::Arc::new(plan.clone()), recipe: Default::default() };
            patch_tile(&mut tile, key, &edits);
            store.put_tile(0, 0, 0, TILE as u32, &compact_tile(&tile, 1));
            store.put_pieces(&pieces_near(&changed));
        };
        let size = (320, 200);
        // The middle pixel looking straight north from 3½ blocks south of the cell, `up` into it.
        let middle = |up: f32| {
            let eye = crate::sight::draw::Eye {
                position: [0.5, 0.5, (ground + 1) as f32 + up],
                looking: [1.0, 0.0, 0.0],
                field_of_view: 60.0,
                grid: 2.0,
                coarsest: 8,
                shadows: false,
                sky: false,
                fog: false,
                textures: true,
                ..Default::default()
            };
            sight_picture_with(&device, &queue, &store, size, eye, Some(&textures))[(size.1 / 2 * size.0 + size.0 / 2) as usize]
        };
        let near = |a: [u8; 3], b: [u8; 3]| (0..3).all(|c| (a[c] as i32 - b[c] as i32).abs() <= 3);
        show(None);
        let air = (middle(0.25), middle(0.75));
        let mut whole = chunk.clone();
        whole.set_block_at(cell[0], cell[1], cell[2], blocks.gabbro);
        show(Some(whole));
        let block = (middle(0.25), middle(0.75));
        let mut slab = chunk.clone();
        let mut pieces = Pieces::default();
        for o in 0..4 {
            pieces.set(o, blocks.gabbro, Part::LONE);
        }
        slab.set_pieces_at(cell[0], cell[1], cell[2], pieces);
        show(Some(slab));
        let seen = (middle(0.25), middle(0.75));
        eprintln!("Sight's pieces: through the lower half air {:?}, block {:?}, slab {:?}; the upper half air {:?}, block {:?}, slab {:?}", air.0, block.0, seen.0, air.1, block.1, seen.1);
        assert!(!near(air.0, block.0), "the block shows against the air");
        assert!(near(seen.0, block.0), "through its lower half the slab is the block");
        assert!(near(seen.1, air.1) && !near(seen.1, block.1), "through its upper half the slab is the air");
        // Gabbro given a bar's own textures (build 91), the side the orange, the end a blue: a lone octant wears
        // its end on every face, so the slab's face is blue.
        let mut with_bars = textures.clone();
        with_bars.layers.push(mips((0..side * side).flat_map(|_| [40u8, 80, 220, 255]).collect()));
        with_bars.bars = vec![[crate::sight::textures::NONE; 2], [0, 1]];
        let eye = crate::sight::draw::Eye { position: [0.5, 0.5, (ground + 1) as f32 + 0.25], looking: [1.0, 0.0, 0.0], field_of_view: 60.0, grid: 2.0, coarsest: 8, shadows: false, sky: false, fog: false, textures: true, ..Default::default() };
        let barred = sight_picture_with(&device, &queue, &store, size, eye, Some(&with_bars))[(size.1 / 2 * size.0 + size.0 / 2) as usize];
        eprintln!("Sight's pieces with a bar's own end texture (blue): {barred:?}");
        assert!(barred[2] > barred[0] * 2, "a lone octant wears its bar's end");
    }

    /// Sight's fog (3.4c): near land hardly fogged, far land well on its way to the horizon's colour, and the same
    /// view 150 blocks higher (over a plateau) clearer, the haze lying low (a thick, low haze, to measure it).
    #[test]
    #[ignore]
    fn sight_s_fog_lies_low_and_far() {
        // A plain at 150 to the west, a plateau at 300 to the east.
        let Some((device, queue, store, _map)) = sight_scene(64.0, |x| if x < 24 { 150.0 } else { 300.0 }) else {
            eprintln!("No graphics card: sight's fog test is skipped");
            return;
        };
        let size = (320, 200);
        // The middle pixel from 100 over the ground at `east`, looking north at the ground `off` away.
        let middle = |east: f32, ground: f32, off: f32, fog: bool| {
            let pitch = -(100.0f32 / off).atan();
            let eye = crate::sight::draw::Eye {
                position: [0.0, east, ground + 100.0],
                looking: [pitch.cos(), 0.0, pitch.sin()],
                field_of_view: 60.0,
                grid: 2.0,
                coarsest: 8,
                shadows: false,
                sky: false,
                fog,
                fog_reach: 1600.0,
                fog_height: 100.0,
                fog_warmth: 0.0,
                ..Default::default()
            };
            let picture = sight_picture(&device, &queue, &store, size, eye);
            picture[(size.1 / 2 * size.0 + size.0 / 2) as usize].map(|c| c as f32)
        };
        // In linear light (the picture holds sRGB), where the fog mixes.
        let linear = |c: f32| {
            let c = c / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        let light = |p: [f32; 3]| linear(p[0]) * 0.2 + linear(p[1]) * 0.7 + linear(p[2]) * 0.1;
        let horizon = 0.62 * 0.2 + 0.72 * 0.7 + 0.86 * 0.1;
        // How far towards the horizon's colour the fog took a view (0 none, 1 all the way).
        let amount = |east: f32, ground: f32, off: f32| {
            let (clear, foggy) = (middle(east, ground, off, false), middle(east, ground, off, true));
            eprintln!("Sight's fog from {east} over {ground}, {off} off: clear {clear:?}, fogged {foggy:?}");
            assert!(clear[1] < 160.0, "the view meets the ground, not the sky ({clear:?})");
            (light(foggy) - light(clear)) / (horizon - light(clear))
        };
        let near = amount(-768.0, 150.0, 58.0);
        let far = amount(-768.0, 150.0, 600.0);
        let high = amount(768.0, 300.0, 600.0);
        eprintln!("Sight's fog: near {near:.2}, far {far:.2}, the same far view 150 higher {high:.2}");
        assert!(near < 0.2, "near land hardly fogged ({near:.2})");
        assert!(far > 0.25 && far > near * 2.5, "far land well into the fog ({far:.2})");
        assert!(high < far * 0.6, "higher up, the air clearer ({high:.2} against {far:.2})");
    }

    /// A far ridge against the sky has no bright rim (David on build 98: "the way the mountains make a brighter
    /// outline against the sky where their haze ends, is there a way we can tighten that up"): down a column of
    /// the picture from the sky onto a hazed ridge, no pixel stands brighter than both the sky a little above it and
    /// the ridge a little below.
    #[test]
    #[ignore]
    fn sight_s_far_ridge_has_no_bright_rim() {
        // A plain at 140; ridges at 330 running east to west, 1,000 to 1,250 off either way north and south.
        let Some((device, queue, store, _map)) = sight_scene_map(64.0, |_, row| if (8..12).contains(&row) || (36..40).contains(&row) { 330.0 } else { 140.0 }, None) else {
            eprintln!("No graphics card: sight's skyline test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let size = (320, 200);
        let pitch = (2.0f32).to_radians();
        let eye = crate::sight::draw::Eye {
            position: [0.0, 0.0, 260.0],
            looking: [pitch.cos(), 0.0, pitch.sin()],
            field_of_view: 40.0,
            grid: 2.0,
            coarsest: 8,
            shadows: false,
            sky: false,
            fog: true,
            fog_reach: 1600.0,
            fog_height: 200.0,
            fog_warmth: 0.0,
            ..Default::default()
        };
        let picture = sight_picture(&device, &queue, &store, size, eye);
        if let Some(dir) = &probe {
            write_picture(&dir.join("far_ridge.png"), size, &picture);
        }
        let linear = |c: u8| {
            let c = c as f32 / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        let light = |p: [u8; 3]| linear(p[0]) * 0.2 + linear(p[1]) * 0.7 + linear(p[2]) * 0.1;
        let mut worst = 0.0f32;
        for x in [80u32, 160, 240] {
            let column: Vec<f32> = (0..size.1).map(|y| light(picture[(y * size.0 + x) as usize])).collect();
            // Where the ridge starts: the first pixel down the column clearly off the sky's smooth fall.
            eprintln!("Sight's skyline, column {x}: {:?}", (0..size.1).step_by(8).map(|y| picture[(y * size.0 + x) as usize]).collect::<Vec<_>>());
            let k = 8;
            for y in k..size.1 as usize - k {
                let bump = column[y] - column[y - k].max(column[y + k]);
                worst = worst.max(bump);
            }
        }
        eprintln!("Sight's skyline: the brightest bump over both sky and ridge {worst:.4}");
        assert!(worst < 0.01, "no bright rim where the ridge meets the sky ({worst:.4})");
    }

    /// Far lights' glows (build 100; David: "braziers in a cave at night from a distance were just pixels of green
    /// that popped in and out as they twinkled"): a light 500 off on open ground is a pixel or so of its colour; one
    /// behind a ridge is hidden (its ray marched to it, not the picture's depth, says so).
    #[test]
    #[ignore]
    fn sight_shows_far_lights_as_glows() {
        // A plain at 140; ridges at 330, 770 to 1,020 off north and south.
        let Some((device, queue, store, _map)) = sight_scene_map(64.0, |_, row| if (8..12).contains(&row) || (36..40).contains(&row) { 330.0 } else { 140.0 }, None) else {
            eprintln!("No graphics card: sight's far glows test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let size = (320, 200);
        let pitch = (-10.0f32).to_radians();
        let eye = crate::sight::draw::Eye { position: [0.0, 0.0, 260.0], looking: [pitch.cos(), 0.0, pitch.sin()], field_of_view: 40.0, grid: 2.0, coarsest: 8, shadows: false, sky: false, fog: false, ..Default::default() };
        let glow = |north: f32| crate::sight::models::ObjectsFrame { glows: vec![crate::sight::models::GlowDrawn { at: [north, 0.0, 141.5], colour: [1.0, 1.0, 1.0], size: 0.75 }], ..Default::default() };
        let plain = sight_picture_full(&device, &queue, &store, size, eye, None, None);
        let open = sight_picture_full(&device, &queue, &store, size, eye, None, Some(glow(500.0)));
        let behind = sight_picture_full(&device, &queue, &store, size, eye, None, Some(glow(1300.0)));
        if let Some(dir) = &probe {
            write_picture(&dir.join("glow_open.png"), size, &open);
        }
        let changed = |picture: &[[u8; 3]]| plain.iter().zip(picture).filter(|(a, b)| a != b).map(|(a, b)| (0..3).map(|c| b[c] as i32 - a[c] as i32).max().unwrap_or(0)).collect::<Vec<i32>>();
        let (seen, hidden) = (changed(&open), changed(&behind));
        eprintln!("Sight's far glows: in the open {seen:?}; behind the ridge {hidden:?}");
        // From low over the plain (30 up), one 600 off on it: a ray to the flame itself skims the ground's soft cells
        // (128 to 160, a third solid) for its last 200 blocks; aimed over them, it sees it. (Not lower: this scene keeps
        // only the 16s near the eye, not the game's 1s to 8s, so an eye 10 up is in a soft cell itself.)
        let flat = (-2.5f32).to_radians();
        let low = crate::sight::draw::Eye { position: [0.0, 0.0, 170.0], looking: [flat.cos(), 0.0, flat.sin()], ..eye };
        let low_plain = sight_picture_full(&device, &queue, &store, size, low, None, None);
        let low_glow = sight_picture_full(&device, &queue, &store, size, low, None, Some(glow(600.0)));
        let grazing = low_plain.iter().zip(&low_glow).filter(|(a, b)| a != b).count();
        eprintln!("Sight's far glows: from low over the plain, {grazing} pixels");
        assert!((1..=4).contains(&seen.len()), "a pixel or so in the open ({} changed)", seen.len());
        assert!(seen.iter().all(|&brighter| brighter > 40), "brighter where it shows ({seen:?})");
        assert!(hidden.is_empty(), "hidden behind the ridge ({hidden:?})");
        assert!((1..=4).contains(&grazing), "seen from low over open ground ({grazing} changed)");
    }

    /// The wash (build 101; David: "i'd think some of the green light wash would squish down to a couple brighter
    /// greener coarse pixels for a moment after that in our new engine"): a light's patch on the ground, seen at night
    /// from high over it, where the march reads cells bigger than blocks, is a few coarse pixels of its green; the
    /// ground away from it is as dark as without it.
    #[test]
    #[ignore]
    fn sight_s_wash_shows_far_off() {
        use crate::sight::lights::{LightFrom, PlacedLights, wash};
        let Some((device, queue, store, _map)) = sight_scene_with(64.0, |_| 150.0, Some((0, 0))) else {
            eprintln!("No graphics card: sight's wash test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        // The plain's top is at 151: the light just over it in the middle of a cell of 16 (as most are: one on a corner
        // of four shares its patch four ways), green, reaching 15.
        let floor = |at: [i32; 3]| at[2] < 151;
        let mut lights = PlacedLights::default();
        lights.set(&[LightFrom { object: [8 * 16, 8 * 16, 151 * 16], cell: [8, 8, 151], colour: [0, 15, 0], reach: 15 }], &floor);
        let lit = lights.lit();
        let cells: Vec<([i32; 3], u32)> = lit.iter().chain(&wash(&lit, &floor)).copied().collect();
        assert_eq!(store.put_lights(&cells), cells.len());
        let size = (200, 200);
        let night = (-30f32).to_radians();
        let sun = [night.cos() * 0.6, night.cos() * 0.8, night.sin()];
        let down = (-89.9f32).to_radians();
        let picture = |height: f32, on: bool| {
            let eye = crate::sight::draw::Eye { position: [8.5, 8.5, height], looking: [down.cos(), 0.0, down.sin()], field_of_view: 30.0, grid: 2.0, coarsest: 8, sun, night_light: 0.04, fog: false, placed_lights: on, flicker: 0.0, ..Default::default() };
            sight_picture(&device, &queue, &store, size, eye)
        };
        let green = |p: &[[u8; 3]]| p.iter().filter(|c| c[1] as i32 > c[0] as i32 + 30).count();
        for height in [600.0f32, 2000.0] {
            let (with, without) = (picture(height, true), picture(height, false));
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("wash_{height}.png")), size, &with);
            }
            let middle = |p: &[[u8; 3]]| p[(size.1 / 2 * size.0 + size.0 / 2) as usize];
            let corner = |p: &[[u8; 3]]| p[(4 * size.0 + 4) as usize];
            eprintln!("Sight's wash from {height} up: by the light {:?} (off {:?}); {} green pixels; far from it {:?} (off {:?})", middle(&with), middle(&without), green(&with), corner(&with), corner(&without));
            assert!(middle(&with)[1] as i32 > middle(&without)[1] as i32 + 20, "the light's patch shows from {height} up");
            assert!(green(&with) >= 1, "green pixels from {height} up");
            assert!(corner(&with) == corner(&without), "the ground away from it as dark as without it, from {height} up");
        }
    }

    /// The near walk as it skips air draws what every block walked draws (the truth), over hills: grazing, down and
    /// level, the blocks and the live cells of 2 to 16 made round the eye [2026-11-09: it skipped into the hills near
    /// the eye, differently each run, till the bake's rows were kept inside the map's bounds].
    #[test]
    #[ignore]
    fn sight_s_near_walk_against_every_block() {
        // Hills: 16 blocks a map pixel, rising and falling 40 either way of 150.
        let height = |column: usize, row: usize| 150.0 + 40.0 * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
        let Some((device, queue, store, map)) = sight_scene_levels(16.0, height, Some((0, 0)), &[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]) else {
            eprintln!("No graphics card: sight's near walk test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let ground = |north: f32, east: f32| map.height(north, east).unwrap_or(150.0);
        let size = (320, 200);
        let mut apart = Vec::new();
        // And over water, the eye 6 over the sea's 128 (x north, y east; the map's rows run south): the lake at (−71,
        // 19), its north shore inside the blocks made round (0, 0), down into it from there; and the one at (152,
        // −268), past them, down into it and along it from its west shore. Both some 18 deep.
        for (name, at, pitch, heading) in [("grazing", [0.0f32, 0.0, 8.0], -8.0f32, 0.0f32), ("down", [10.0, -20.0, 60.0], -45.0, 90.0), ("level", [-30.0, 10.0, 3.0], 0.0, 30.0), ("into water, the blocks", [-48.0, 19.0, 0.0], -50.0, 180.0), ("into water", [152.0, -268.0, 0.0], -50.0, 20.0), ("over water", [152.0, -290.0, 0.0], -12.0, 90.0)] {
            let position = if name.contains("water") { [at[0], at[1], crate::world::SEA_LEVEL as f32 + 6.0] } else { [at[0], at[1], ground(at[0], at[1]) + at[2]] };
            let (p, h) = (pitch.to_radians(), heading.to_radians());
            let looking = [p.cos() * h.cos(), p.cos() * h.sin(), p.sin()];
            let picture = |walk_every_block: bool| {
                let eye = crate::sight::draw::Eye { position, looking, field_of_view: 70.0, grid: 2.0, coarsest: 8, walk_every_block, ..Default::default() };
                sight_picture(&device, &queue, &store, size, eye)
            };
            let (truth, walked) = (picture(true), picture(false));
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("near_walk_{name}_truth.png")), size, &truth);
                write_picture(&dir.join(format!("near_walk_{name}.png")), size, &walked);
            }
            let d: Vec<i32> = truth.iter().zip(&walked).filter(|(x, y)| x != y).map(|(x, y)| (0..3).map(|c| (x[c] as i32 - y[c] as i32).abs()).max().unwrap_or(0)).collect();
            eprintln!("Sight's near walk, {name} (the eye at {:?}, the ground and water under it {:?}): against every block walked, {} pixels differ (by {} at most)", position, map.ground_and_water(at[0], at[1]), d.len(), d.iter().copied().max().unwrap_or(0));
            apart.push((name, d.len()));
        }
        for (name, n) in apart {
            assert!(n <= (size.0 * size.1 / 1000) as usize, "the near walk draws what every block walked draws, {name} ({n} pixels differ)");
        }
    }

    /// Another build of the march (`INCUS_SIGHT_AB=<a march.wgsl>`: the dyno's steps that should leave the picture as
    /// it is, such as the march made smaller) draws what the march as built draws: over hills near and far, down into
    /// lakes and along one, from high up, with the sun's shadows, sky light and textures, every pixel set side by
    /// side. Skipped without one.
    #[test]
    #[ignore]
    fn sight_s_other_march_draws_the_same() {
        use crate::sight::textures::{BlockTextures, NONE, mips};
        let Ok(path) = std::env::var("INCUS_SIGHT_AB") else {
            eprintln!("No other march to set beside it (INCUS_SIGHT_AB): sight's A/B picture test is skipped");
            return;
        };
        let other = std::fs::read_to_string(&path).expect("the other march");
        let height = |column: usize, row: usize| 150.0 + 40.0 * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
        let Some((device, queue, store, map)) = sight_scene_levels(16.0, height, Some((0, 0)), &[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]) else {
            eprintln!("No graphics card: sight's A/B picture test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let ground = |north: f32, east: f32| map.height(north, east).unwrap_or(150.0);
        let side = crate::look::TEXTURE_SIZE;
        let checker: Vec<u8> = (0..side * side).flat_map(|i| if (i % side / 4 + i / side / 4) % 2 == 0 { [200, 60, 60, 255] } else { [60, 60, 200, 255] }).collect();
        let textures = BlockTextures { layers: vec![mips(checker)], blocks: (0..16).map(|k| if k == 0 { ([NONE; 6], 0) } else { ([0; 6], 0) }).collect(), bars: Vec::new(), problems: Vec::new(), media: Vec::new() };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let mut built = crate::sight::draw::SightDraw::new(&device, &queue, format);
        let mut theirs = crate::sight::draw::SightDraw::with_march(&device, &queue, format, &other);
        built.set_textures(&device, &queue, &textures, 1);
        theirs.set_textures(&device, &queue, &textures, 1);
        let size = (480, 300);
        let mut worst = (0usize, 0i32);
        // Heights over the ground, or (water) over the sea's 128; x north, y east.
        for (name, at, pitch, heading) in [
            ("grazing", [0.0f32, 0.0, 8.0], -8.0f32, 0.0f32),
            ("down", [10.0, -20.0, 60.0], -45.0, 90.0),
            ("level", [-30.0, 10.0, 3.0], 0.0, 30.0),
            ("into water, the blocks", [-48.0, 19.0, 6.0], -50.0, 180.0),
            ("into water", [152.0, -268.0, 6.0], -50.0, 20.0),
            ("over water", [152.0, -290.0, 6.0], -12.0, 90.0),
            ("high", [0.0, 0.0, 400.0], -30.0, 45.0),
        ] {
            let position = if name.contains("water") { [at[0], at[1], crate::world::SEA_LEVEL as f32 + at[2]] } else { [at[0], at[1], ground(at[0], at[1]) + at[2]] };
            let (p, h) = (pitch.to_radians(), heading.to_radians());
            let looking = [p.cos() * h.cos(), p.cos() * h.sin(), p.sin()];
            let eye = crate::sight::draw::Eye { position, looking, field_of_view: 70.0, grid: 2.0, coarsest: 8, ..Default::default() };
            let ours = sight_picture_by(&mut built, &device, &queue, &store, size, eye);
            let other_picture = sight_picture_by(&mut theirs, &device, &queue, &store, size, eye);
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("ab_{name}_built.png")), size, &ours);
                write_picture(&dir.join(format!("ab_{name}_other.png")), size, &other_picture);
            }
            let d: Vec<i32> = ours.iter().zip(&other_picture).filter(|(x, y)| x != y).map(|(x, y)| (0..3).map(|c| (x[c] as i32 - y[c] as i32).abs()).max().unwrap_or(0)).collect();
            let found = (d.len(), d.iter().copied().max().unwrap_or(0));
            eprintln!("Sight's other march, {name}: {} pixels differ (by {} at most)", found.0, found.1);
            worst = (worst.0.max(found.0), worst.1.max(found.1));
        }
        assert!(worst.0 <= (size.0 * size.1 / 1000) as usize && worst.1 <= 2, "the other march draws what the march as built draws ({} pixels differ at worst, by {} at most)", worst.0, worst.1);
    }

    /// The march split from its light (the dyno's third step, off unless chosen) draws what the one pass draws, to a
    /// step of a colour (half floats between them): over hills, near and far, with the sun's shadows, sky light and
    /// textures [2026-11-09: dark specks along near blocks' edges in some runs, gone with the bake's rows kept inside
    /// the map's bounds].
    #[test]
    #[ignore]
    fn sight_s_march_split_from_its_light_the_same() {
        let height = |column: usize, row: usize| 150.0 + 40.0 * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
        let Some((device, queue, store, map)) = sight_scene_levels(16.0, height, Some((0, 0)), &[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]) else {
            eprintln!("No graphics card: sight's split test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let ground = |north: f32, east: f32| map.height(north, east).unwrap_or(150.0);
        let size = (640, 400);
        let mut worst = (0usize, 0i32);
        for (name, at, pitch, heading) in [("near", [0.0f32, 0.0, 6.0], -20.0f32, 0.0f32), ("far", [0.0, 0.0, 200.0], -25.0, 45.0), ("low", [0.0, 0.0, 40.0], -8.0, 120.0)] {
            let position = [at[0], at[1], ground(at[0], at[1]) + at[2]];
            let (p, h) = (pitch.to_radians(), heading.to_radians());
            let looking = [p.cos() * h.cos(), p.cos() * h.sin(), p.sin()];
            let picture = |split: bool| {
                let eye = crate::sight::draw::Eye { position, looking, field_of_view: 70.0, grid: 2.0, coarsest: 8, split, ..Default::default() };
                sight_picture(&device, &queue, &store, size, eye)
            };
            // Together twice (the scene's own steadiness), split between.
            let (together, split, again) = (picture(false), picture(true), picture(false));
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("split_{name}.png")), size, &split);
                write_picture(&dir.join(format!("split_{name}_together.png")), size, &together);
                write_picture(&dir.join(format!("split_{name}_again.png")), size, &again);
            }
            let apart = |a: &[[u8; 3]], b: &[[u8; 3]]| {
                let gaps: Vec<i32> = a.iter().zip(b).map(|(x, y)| (0..3).map(|c| (x[c] as i32 - y[c] as i32).abs()).max().unwrap_or(0)).collect();
                (gaps.iter().filter(|&&g| g > 1).count(), gaps.iter().copied().max().unwrap_or(0))
            };
            let (first, second, itself) = (apart(&together, &split), apart(&again, &split), apart(&together, &again));
            // Where together drew the same twice (this scene sometimes moves between pictures, the near walk landing
            // differently on its steep hills: on file to find), split against it.
            let steady: Vec<i32> = together.iter().zip(&again).zip(&split).filter(|((a, b), _)| a == b).map(|((a, _), c)| (0..3).map(|k| (a[k] as i32 - c[k] as i32).abs()).max().unwrap_or(0)).collect();
            let off = (steady.iter().filter(|&&g| g > 1).count(), steady.iter().copied().max().unwrap_or(0));
            eprintln!("Sight's march split from its light, {name}: where together drew the same twice ({} pixels), {} more than a step apart ({} at most); against together {} ({}), together again {} ({}); together against itself {} ({})", steady.len(), off.0, off.1, first.0, first.1, second.0, second.1, itself.0, itself.1);
            worst = (worst.0.max(off.0), worst.1.max(off.1));
        }
        eprintln!("Sight's march split from its light: at worst {} pixels more than a step apart, {} at most", worst.0, worst.1);
        assert!(worst.0 <= (size.0 * size.1 / 1000) as usize, "the same picture ({} pixels more than a step apart, {} at most)", worst.0, worst.1);
    }

    /// The march in stages (the dyno, 2026-11-09: the march, the sun's shadow and the light each a pass of their own)
    /// draws what the one pass draws, to a step of a colour (the records' half floats between them), and what the
    /// split draws: over hills near and far, down into a lake, with the sun's shadows, sky light and textures.
    #[test]
    #[ignore]
    fn sight_s_march_in_stages_the_same() {
        let height = |column: usize, row: usize| 150.0 + 40.0 * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
        let Some((device, queue, store, map)) = sight_scene_levels(16.0, height, Some((0, 0)), &[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]) else {
            eprintln!("No graphics card: sight's stages test is skipped");
            return;
        };
        let probe = std::env::var("INCUS_PROBE_DIR").ok().map(std::path::PathBuf::from);
        let ground = |north: f32, east: f32| map.height(north, east).unwrap_or(150.0);
        let size = (640, 400);
        let mut worst = (0usize, 0i32);
        for (name, at, pitch, heading) in [("near", [0.0f32, 0.0, 6.0], -20.0f32, 0.0f32), ("far", [0.0, 0.0, 200.0], -25.0, 45.0), ("low", [0.0, 0.0, 40.0], -8.0, 120.0), ("into water", [152.0, -268.0, 6.0], -50.0, 20.0)] {
            let position = if name.contains("water") { [at[0], at[1], crate::world::SEA_LEVEL as f32 + at[2]] } else { [at[0], at[1], ground(at[0], at[1]) + at[2]] };
            let (p, h) = (pitch.to_radians(), heading.to_radians());
            let looking = [p.cos() * h.cos(), p.cos() * h.sin(), p.sin()];
            let picture = |split: bool, stages: bool| {
                let eye = crate::sight::draw::Eye { position, looking, field_of_view: 70.0, grid: 2.0, coarsest: 8, split, stages, ..Default::default() };
                sight_picture(&device, &queue, &store, size, eye)
            };
            let (together, split, stages) = (picture(false, false), picture(true, false), picture(false, true));
            if let Some(dir) = &probe {
                write_picture(&dir.join(format!("stages_{name}.png")), size, &stages);
                write_picture(&dir.join(format!("stages_{name}_together.png")), size, &together);
            }
            let apart = |a: &[[u8; 3]], b: &[[u8; 3]]| {
                let gaps: Vec<i32> = a.iter().zip(b).map(|(x, y)| (0..3).map(|c| (x[c] as i32 - y[c] as i32).abs()).max().unwrap_or(0)).collect();
                (gaps.iter().filter(|&&g| g > 1).count(), gaps.iter().filter(|&&g| g > 0).count(), gaps.iter().copied().max().unwrap_or(0))
            };
            let (from_together, from_split) = (apart(&together, &stages), apart(&split, &stages));
            eprintln!("Sight's march in stages, {name}: against the one pass {} pixels more than a step apart ({} differ, {} at most); against the split {} ({} differ, {} at most)", from_together.0, from_together.1, from_together.2, from_split.0, from_split.1, from_split.2);
            worst = (worst.0.max(from_together.0), worst.1.max(from_together.2));
        }
        assert!(worst.0 <= (size.0 * size.1 / 1000) as usize, "the same picture as the one pass ({} pixels more than a step apart, {} at most)", worst.0, worst.1);
    }

    /// The hills scene made twice in one go draws the same, skipping air and every block walked [2026-11-09: it didn't,
    /// the bake's rows written past the map's bounds as the bakers happened to finish].
    #[test]
    #[ignore]
    fn sight_s_scene_made_twice_draws_the_same() {
        let height = |column: usize, row: usize| 150.0 + 40.0 * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
        let size = (320, 200);
        let mut pictures = Vec::new();
        for _ in 0..2 {
            let Some((device, queue, store, map)) = sight_scene_levels(16.0, height, Some((0, 0)), &[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]) else { return };
            let ground = map.height(0.0, 0.0).unwrap_or(150.0);
            let pitch = (-8.0f32).to_radians();
            let picture = |walk_every_block: bool| {
                let eye = crate::sight::draw::Eye { position: [0.0, 0.0, ground + 8.0], looking: [pitch.cos(), 0.0, pitch.sin()], field_of_view: 70.0, grid: 2.0, coarsest: 8, walk_every_block, ..Default::default() };
                sight_picture(&device, &queue, &store, size, eye)
            };
            pictures.push((picture(false), picture(true)));
        }
        let apart = |a: &[[u8; 3]], b: &[[u8; 3]]| a.iter().zip(b).filter(|(x, y)| x != y).count();
        let found = (apart(&pictures[0].0, &pictures[1].0), apart(&pictures[0].1, &pictures[1].1), apart(&pictures[0].0, &pictures[0].1), apart(&pictures[1].0, &pictures[1].1));
        eprintln!("Sight's scene made twice: skipping {} pixels apart, every block {} apart; first scene skipping against its truth {}, second {}", found.0, found.1, found.2, found.3);
        assert_eq!(found, (0, 0, 0, 0), "made twice, the same pictures");
    }

    /// The hills scene made twice holds the same on the card, its store read back each time and set side by side, live
    /// slot by live slot and baked entry by baked entry (which levels differ, and how, if any) [2026-11-09: a few
    /// baked columns were empty in one making and not in the other, the bake's rows written past the map's bounds].
    #[test]
    #[ignore]
    fn sight_s_scene_made_twice_holds_the_same() {
        let height = |column: usize, row: usize| 150.0 + 40.0 * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
        let mut reads = Vec::new();
        for _ in 0..2 {
            let Some((device, queue, store, _map)) = sight_scene_levels(16.0, height, Some((0, 0)), &[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]) else { return };
            reads.push(store.read_back(&device, &queue));
        }
        let (a, b) = (&reads[0], &reads[1]);
        let side2 = a.most_side * a.most_side;
        // The live levels, slot by slot.
        for (window, (sa, sb)) in a.live.chunks(side2).zip(b.live.chunks(side2)).enumerate() {
            let level = a.finest_live + window;
            let (mut where_, mut bands, mut cells, mut example) = (0, 0, 0, String::new());
            for (k, (x, y)) in sa.iter().zip(sb).enumerate() {
                if (x[2], x[3]) != (y[2], y[3]) {
                    where_ += 1;
                    if example.is_empty() {
                        example = format!("slot {k}: column ({}, {}) against ({}, {})", x[2] as i32, x[3] as i32, y[2] as i32, y[3] as i32);
                    }
                    continue;
                }
                let (ca, cb) = (a.cells(x[0], x[1]), b.cells(y[0], y[1]));
                if ca.0 != cb.0 || ca.1.len() != cb.1.len() {
                    bands += 1;
                    if example.is_empty() {
                        example = format!("column ({}, {}): band from {} for {} against from {} for {}", x[2] as i32, x[3] as i32, ca.0, ca.1.len(), cb.0, cb.1.len());
                    }
                } else if ca.1 != cb.1 {
                    cells += 1;
                    if example.is_empty() {
                        let k = ca.1.iter().zip(&cb.1).position(|(p, q)| p != q).unwrap_or(0);
                        example = format!("column ({}, {}): cell {} of its band {:?} against {:?}", x[2] as i32, x[3] as i32, k, ca.1[k], cb.1[k]);
                    }
                }
            }
            eprintln!("Sight's store made twice, live level {level} (cells of {}): {where_} slots for other columns, {bands} bands differ, {cells} columns' cells differ. {example}", 1 << level);
            assert_eq!(where_ + bands + cells, 0, "live level {level} the same both times: {example}");
        }
        // The baked levels, entry by entry.
        for (k, &(from, across, at)) in a.extents.iter().enumerate() {
            let level = crate::sight::bake::FIRST + k;
            let n = (across * across) as usize;
            let (mut bands, mut cells, mut example) = (0, 0, String::new());
            for i in 0..n {
                let (x, y) = (a.baked[at as usize + i], b.baked[at as usize + i]);
                let (ca, cb) = (a.cells(x[0], x[1]), b.cells(y[0], y[1]));
                let column = (from.0 + (i as i32 % across), from.1 + (i as i32 / across));
                if ca.0 != cb.0 || ca.1.len() != cb.1.len() {
                    bands += 1;
                    if example.is_empty() {
                        example = format!("column {column:?}: band from {} for {} against from {} for {} (words {:08x} {:08x})", ca.0, ca.1.len(), cb.0, cb.1.len(), x[1], y[1]);
                    }
                } else if ca.1 != cb.1 {
                    cells += 1;
                    if example.is_empty() {
                        let j = ca.1.iter().zip(&cb.1).position(|(p, q)| p != q).unwrap_or(0);
                        example = format!("column {column:?}: cell {j} {:?} against {:?}", ca.1[j], cb.1[j]);
                    }
                }
            }
            if bands + cells > 0 || k < 3 {
                eprintln!("Sight's store made twice, baked level {level} (cells of {}): {bands} bands differ, {cells} columns' cells differ of {n}. {example}", 1 << level);
            }
            assert_eq!(bands + cells, 0, "baked level {level} the same both times: {example}");
        }
    }

    /// A measuring test (ignored; the dyno's first step): the near walk's steps a ray as it skips air, against every
    /// block walked, over flat land (the dyno's land leg: 40 up, 8° down) and over hills, with every live level made
    /// round the eye and with only the blocks and the 16s (as while the camera moves: the near levels never finish),
    /// at David's screen's size. Run with `--test-threads=1` (it sets the steps view's switch). [2026-11-09, the
    /// finest-first skip tried: 80 a ray against 73 over flat land, 69 against 73 over hills: no steps to save.]
    #[test]
    #[ignore]
    fn sight_s_near_walk_steps() {
        let steps_of = |p: [u8; 3]| {
            let (r, g) = ((p[0] as f32 / 255.0).sqrt(), (p[1] as f32 / 255.0).sqrt());
            if r > 0.01 { (0.5 + r * 0.5) * 256.0 } else { g / 0.8 * 0.5 * 256.0 }
        };
        // David's screen's own size: the near walk reaches as far as there (about 336 blocks: past the ground 285 off).
        let size = (2560u32, 1600u32);
        for (ground_name, bpp, hills) in [("flat", 64.0f32, 0.0f32), ("hills", 16.0, 40.0)] {
            let height = move |column: usize, row: usize| 150.0 + hills * ((column as f32 * 0.7).sin() * (row as f32 * 0.45).cos());
            for (fill_name, fill) in [("every level", &[0usize, 1, 2, 3, 4][..]), ("blocks and 16s", &[0, 4][..])] {
                let Some((device, queue, store, map)) = sight_scene_levels(bpp, height, Some((0, 0)), &[0, 1, 2, 3, 4], fill) else { return };
                let ground = map.height(0.0, 0.0).unwrap_or(150.0);
                let pitch = (-8.0f32).to_radians();
                let picture = |walk_every_block: bool| {
                    // SAFETY: run one at a time (`--test-threads=1`).
                    unsafe { std::env::set_var("INCUS_SIGHT_STEPS", "1") };
                    let eye = crate::sight::draw::Eye { position: [0.0, 0.0, ground + 40.0], looking: [pitch.cos(), 0.0, pitch.sin()], field_of_view: 70.0, grid: 2.0, coarsest: 8, shadows: false, sky: false, fog: false, walk_every_block, ..Default::default() };
                    let picture = sight_picture(&device, &queue, &store, size, eye);
                    unsafe { std::env::remove_var("INCUS_SIGHT_STEPS") };
                    picture
                };
                let (skipping, every) = (picture(false), picture(true));
                // The lower half: the ground, where the near walk is.
                let mean = |picture: &[[u8; 3]]| {
                    let rows = size.1 / 2..size.1;
                    rows.clone().flat_map(|y| (0..size.0).map(move |x| (x, y))).map(|(x, y)| steps_of(picture[(y * size.0 + x) as usize])).sum::<f32>() / (rows.len() as u32 * size.0) as f32
                };
                eprintln!("Sight's near walk steps, {ground_name}, {fill_name}: skipping air {:.0} a ray, every block walked {:.0}", mean(&skipping), mean(&every));
            }
        }
    }

    /// A measuring test (ignored): sight's steps a ray, by band of the screen, over the made-up sea, from 120 up
    /// looking 12° down (the soak's view), read from the steps view's colours.
    #[test]
    #[ignore]
    fn sight_s_steps_by_band() {
        let Some((device, queue, store, _map)) = sight_scene(512.0, |_| 70.0) else { return };
        // SAFETY: tests that read it run one at a time (`--test-threads=1` for this measuring test).
        unsafe { std::env::set_var("INCUS_SIGHT_STEPS", "1") };
        let size = (640u32, 400u32);
        let pitch = -12f32.to_radians();
        let eye = crate::sight::draw::Eye { position: [0.0, 0.0, 250.0], looking: [pitch.cos(), 0.0, pitch.sin()], field_of_view: 100.0, grid: 2.0, coarsest: 8, rays: 1, upscale: 0, earned: true, sun: crate::sight::draw::NOON, shadows: false, sky: false, fog: false, ..Default::default() };
        let picture = sight_picture(&device, &queue, &store, size, eye);
        unsafe { std::env::remove_var("INCUS_SIGHT_STEPS") };
        // The steps view: k = steps / 256 shown as dark blue (0) to green (0.5) to red (1), squared.
        let steps_of = |p: [u8; 3]| {
            let (r, g) = ((p[0] as f32 / 255.0).sqrt(), (p[1] as f32 / 255.0).sqrt());
            if r > 0.01 { (0.5 + r * 0.5) * 256.0 } else { g / 0.8 * 0.5 * 256.0 }
        };
        for band in 0..10 {
            let rows = band * size.1 / 10..(band + 1) * size.1 / 10;
            let mean: f32 = rows.clone().flat_map(|y| (0..size.0).map(move |x| (x, y))).map(|(x, y)| steps_of(picture[(y * size.0 + x) as usize])).sum::<f32>() / ((size.1 / 10) * size.0) as f32;
            eprintln!("Sight's steps: rows {rows:?}: about {mean:.0} a ray");
        }
    }

    /// Sight's cells (phase 3.1b): the card's sums of cells of 1 to 64 blocks, straight from the recipe,
    /// bit for bit the sums of the very blocks the processor's chunks hold (`sight::cell::sums_of_box` over
    /// `CompiledMap::chunk`): ground, a river, a lake, mineralogy's layers, tunnels coming out of the ground
    /// (cave mouths, roofs), and the map's edge.
    #[test]
    fn the_card_makes_sight_s_cells_as_the_chunks_hold_them() {
        use crate::sight::cell::{Sums, sums_of_box};
        use crate::world::{BlockId, ChunkPos};
        use std::collections::HashMap;
        let Some((device, queue, map, plan, blocks, colours, world)) = sight_world() else {
            eprintln!("No graphics card: sight's twin test is skipped");
            return;
        };
        let (mut checked, mut wet, mut roofs, mut mixed) = (0, 0, 0, 0);
        for (x0, y0) in [(-90i32, 40i32), (180, -230), (590, 290), (-1560, 100)] {
            let region = [x0, y0, 64, 64];
            // The processor's chunks round the region, read block by block.
            let mut chunks = HashMap::new();
            for cx in (x0 - 1).div_euclid(32)..=(x0 + 64).div_euclid(32) {
                for cy in (y0 - 1).div_euclid(32)..=(y0 + 64).div_euclid(32) {
                    if let Some(chunk) = map.chunk(ChunkPos::new(cx, cy), &blocks, &plan) {
                        chunks.insert((cx, cy), chunk);
                    }
                }
            }
            let block = |[x, y, z]: [i32; 3]| {
                if z < 0 {
                    blocks.stone
                } else if z >= crate::world::WORLD_HEIGHT {
                    BlockId::AIR
                } else {
                    chunks.get(&(x.div_euclid(32), y.div_euclid(32))).map_or(BlockId::AIR, |chunk| chunk.block_at(x, y, z))
                }
            };
            let (mut low, mut high) = (i32::MAX, i32::MIN);
            for x in x0..x0 + 64 {
                for y in y0..y0 + 64 {
                    if let Some((z, water)) = map.column(x, y) {
                        low = low.min(z);
                        high = high.max(z).max(water.unwrap_or(0)).max(crate::world::SEA_LEVEL);
                    }
                }
            }
            if low > high {
                (low, high) = (100, 140);
            }
            let mut cells = Vec::new();
            for size in [1, 2, 4, 8, 16, 32, 64] {
                let across = if size <= 4 { 16 } else { 64 };
                for z in ((low - 12).div_euclid(size) * size..=high + 4).step_by(size as usize) {
                    for cx in (0..across).step_by(size as usize) {
                        for cy in (0..across).step_by(size as usize) {
                            cells.push([x0 + cx, y0 + cy, z, size]);
                        }
                    }
                }
            }
            let card = world.cells_at(&device, &queue, region, &cells);
            assert_eq!(card.len(), cells.len(), "the card answered");
            let mut wrong = Vec::new();
            for (cell, c) in cells.iter().zip(&card) {
                let processor = sums_of_box([cell[0], cell[1], cell[2]], cell[3], &block, &colours);
                let wide = |k: usize| c[k] as u64 | (c[k + 1] as u64) << 32;
                let on_card = Sums {
                    solid: c[0] as u64,
                    water: c[1] as u64,
                    faces: std::array::from_fn(|d| c[2 + d] as u64),
                    colour: std::array::from_fn(|k| c[8 + k] as u64),
                    light: c[11] as u64,
                    light_squared: wide(12),
                    height: c[14] as u64,
                    height_squared: wide(15),
                };
                checked += 1;
                wet += (processor.water > 0) as usize;
                roofs += (processor.faces[1] > 0) as usize;
                mixed += (processor.solid > 0 && processor.solid < (cell[3] as u64).pow(3)) as usize;
                if on_card != processor && wrong.len() < 6 {
                    wrong.push(format!("cell {cell:?}:\n  card      {on_card:?}\n  processor {processor:?}"));
                }
            }
            assert!(wrong.is_empty(), "region ({x0}, {y0}): the card and the chunks disagree:\n{}", wrong.join("\n"));
        }
        eprintln!("Sight's twin test: {checked} cells, {mixed} part solid, {wet} with water, {roofs} with faces looking down");
        assert!(wet > 50 && roofs > 10 && mixed > 500, "the test misses something: {wet} with water, {roofs} with roofs, {mixed} part solid");
    }

    /// The caves: every block round a set of rough, winding tunnels, inside or not, the card bit for bit the
    /// processor.
    #[test]
    fn the_card_carves_the_caves_as_the_processor_does() {
        use crate::world::caves::{CavePlan, CavePoint, Tunnel};
        let mut next = shuffle();
        let mut tunnels = Vec::new();
        for t in 0..6 {
            let mut at = [(next() % 2000) * 16, (next() % 2000) * 16, (100 + (next() % 60).abs()) * 16];
            let mut points = Vec::new();
            for _ in 0..12 {
                points.push(CavePoint { at, width: 24 + (next() % 60).abs(), height: 20 + (next() % 50).abs() });
                for (k, step) in at.iter_mut().enumerate() {
                    *step += if k == 2 { next() % 40 } else { next() % 120 };
                }
            }
            tunnels.push(Tunnel { points, rough: 6 + t, lump: 3 + t, floor: 180, seed: next() as u32 });
        }
        let plan = CavePlan::new(tunnels.clone());
        let mut pack = Pack::default();
        pack.words = vec![0; SLOTS.len()];
        pack.caves(&plan);
        let mut cases = Vec::new();
        for t in &tunnels {
            for p in &t.points {
                for _ in 0..400 {
                    let (dx, dy, dz) = (next() % 10, next() % 10, next() % 8);
                    cases.push([p.at[0] / 16 + dx, p.at[1] / 16 + dy, p.at[2] / 16 + dz, 0]);
                }
            }
        }
        let Some(answers) = on_the_card("caves", &pack.words, &cases) else {
            eprintln!("No graphics card: the caves' twin test is skipped");
            return;
        };
        let (mut wrong, mut inside) = (Vec::new(), 0);
        for (c, &card) in cases.iter().zip(&answers) {
            let processor = plan.inside_at(c[0], c[1], c[2]) as i32;
            inside += processor as usize;
            if card != processor && wrong.len() < 12 {
                wrong.push(format!("block ({}, {}, {}): card {card}, processor {processor}", c[0], c[1], c[2]));
            }
        }
        assert!(inside > 1000 && inside < cases.len() - 1000, "{inside} of {} blocks inside: the test misses the caves", cases.len());
        assert!(wrong.is_empty(), "the card and the processor disagree:\n{}", wrong.join("\n"));
    }

    /// The ground (before the rivers): the card bit for bit the processor, over a whole small world, steady
    /// ridges and turning.
    #[test]
    fn the_card_makes_the_ground_as_the_processor_does() {
        for steady in [true, false] {
            let map = WholeMap::new(&rich_map(steady));
            let pack = Pack::map(&map);
            let mut cases = Vec::new();
            for north in (-1560..1560).step_by(7 * thinned()) {
                for east in (-1560..1560).step_by(11) {
                    cases.push([Pos::middle_of(north).0, Pos::middle_of(east).0, 0, 0]);
                }
            }
            let Some(answers) = on_the_card("ground", &pack.words, &cases) else {
                eprintln!("No graphics card: the ground's twin test is skipped");
                return;
            };
            let mut wrong = Vec::new();
            let mut inside = 0;
            for (c, &card) in cases.iter().zip(&answers) {
                let processor = map.uncarved(Pos(c[0]), Pos(c[1])).map_or(i32::MIN, |z| z.0);
                inside += (processor != i32::MIN) as usize;
                if card != processor && wrong.len() < 12 {
                    wrong.push(format!("at ({}, {}): card {card}, processor {processor}", Pos(c[0]).to_f32(), Pos(c[1]).to_f32()));
                }
            }
            assert!(inside > 50_000 / thinned(), "only {inside} columns inside the map");
            assert!(wrong.is_empty(), "steady {steady}: the card and the processor disagree:\n{}", wrong.join("\n"));
        }
    }

    /// How much thinner a twin test's sample is on Microsoft's software card (`INCUS_TWIN_CARD=software`):
    /// the processor drawing as a card is slow.
    fn thinned() -> usize {
        if std::env::var("INCUS_TWIN_CARD").is_ok_and(|card| card == "software") { 16 } else { 1 }
    }

    fn shuffle() -> impl FnMut() -> i32 {
        let mut state = 0x2545_F491u32;
        move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as i32
        }
    }

    /// The noise, the curves and the noise layers: the card bit for bit the processor.
    #[test]
    fn the_card_reads_the_noise_and_curves_as_the_processor_does() {
        let mut spline = Spline::through(&[(0.0, 0.0), (0.3, 0.6), (0.7, 0.65), (1.0, 1.0)]);
        spline.bends = vec![0.0, 0.5, -0.4];
        let curve = spline.curve().whole();
        let mut settings = PerlinSettings { scale: 410.0, octaves: 5, persistence: 0.45, lacunarity: 2.3, stretch: 1.7, stretch_angle: 33.0, octave_turn: 21.0, offset_north: 90.0, ..PerlinSettings::default() };
        (settings.output.contrast, settings.output.brightness, settings.output.curve) = (2.35, -0.33, 2.3);
        let noise = WholeNoise::new(&settings, 77);
        let mut pack = Pack::default();
        let (curve_at, noise_at) = (pack.curve(&curve), pack.noise(&noise));
        let mut next = shuffle();
        let mut cases = Vec::new();
        for _ in 0..4000 {
            let x = (next() & 0x1_FFFF) - 0x8000;
            cases.push([0, curve_at as i32, x, 0]);
            cases.push([1, curve_at as i32, x, 0]);
            let (north, east) = (next() >> 9, next() >> 9);
            cases.push([2, noise_at as i32, north, east]);
            cases.push([3, noise_at as i32, north, east]);
            for kind in 4..8 {
                cases.push([kind, 0, next(), next()]);
            }
        }
        let Some(answers) = on_the_card("noise_and_curves", &pack.words, &cases) else {
            eprintln!("No graphics card: the recipe's twin test is skipped");
            return;
        };
        let key = 0x51u32;
        let mut wrong = Vec::new();
        for (c, &card) in cases.iter().zip(&answers) {
            let processor = match c[0] {
                0 => curve.at(Fix(c[2])).0,
                1 => curve.steepness(Fix(c[2])).0,
                2 => noise.at(Pos(c[2]), Pos(c[3])).0,
                3 => noise.raw(Pos(c[2]), Pos(c[3])).0,
                4 => whole_perlin_at(key, Fade::Smooth, Fix(c[2]), Fix(c[3])).0,
                5 => whole_perlin_with_slope_at(key, Fix(c[2]), Fix(c[3])).0.0,
                6 => whole_perlin_with_slope_at(key, Fix(c[2]), Fix(c[3])).1.0,
                _ => whole_perlin_with_slope_at(key, Fix(c[2]), Fix(c[3])).2.0,
            };
            if card != processor && wrong.len() < 12 {
                wrong.push(format!("kind {} ({}, {}): card {card}, processor {processor}", c[0], c[2], c[3]));
            }
        }
        assert!(wrong.is_empty(), "the card and the processor disagree:\n{}", wrong.join("\n"));
    }
}


//! The recipe on the card (the regroup, phase 2.3): `whole_recipe.rs` in the shader language
//! (`recipe.wgsl`), reading the world packed into one list of whole numbers (`Pack`), the very numbers the
//! processor reads. Proved by the twin test: the card and the processor bit for bit, every time.
//!
//! Built a piece at a time, each with its own twin test: the noise, curves and noise layers first.

use crate::whole::{Fix, Pos};
use crate::world::whole_recipe::{WholeCarving, WholeCurve, WholeMap, WholeNoise};
use crate::worldgen::noise::{ARROWS, Fade};

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


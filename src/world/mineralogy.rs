//! Mineralogy — which blocks the ground is made of: its own panel in the World Compiler, after Climate,
//! read by the Interpreter at every column [Counsel and David, 2026-10-29].
//!
//! Since stop 3 of `docs/placer.md` it's the placer's first user: no rules in code, only
//! **distributors** (`placer/distribute.rs`), each a list of candidate blocks with their suitability
//! as stacks of layers, run in order, later ones over earlier ones. It starts as three, with David's
//! own numbers (his preset of 2026-10-31), giving his world to the block:
//! 1. **The surface,** the first that fits: snowy ground where it's cold enough (a block cools for its
//!    height, so peaks too); gabbro on the peaks and stone on the mountainsides, by height; sand on
//!    beaches and in hot, dry deserts; dry grass where it's dry; grass. Under the soils, dirt.
//! 2. **The sea floor:** sand in the shallows, gravel deeper.
//! 3. **River and lake beds:** gravel (or sand) on top, over what's under.
//!
//! The lines between zones wander (each step's own noise), so they don't follow the climate's
//! pixels. **A block that isn't registered is skipped** (no sand block: no beaches), so each new block
//! David paints lands in the world as soon as it's registered.
//!
//! **One function for every place that asks:** the near chunks, the distant terrain and the World
//! Compiler's close-up all call `Mineralogy::layers`, made ready once per map and set of blocks
//! (`CompiledMap::mineralogy`).

use super::block::BlockId;
use super::interpreter::CompiledMap;
use super::placer::{Candidate, Chooser, Column, Curve, Distributor, Field, Layer, MadeField, Mode, Program, Puts, Reading, Source, Stack, Under, Wander};
use super::terrain::{SEA_LEVEL, TerrainBlocks};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Mineralogy's settings (in the Interpreter's recipe, so every world keeps its own; set on the
/// Mineralogy panel).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MineralogySettings {
    /// How deep the crust goes (blocks): the placer's depth as a share of it is measured against this
    /// [Counsel's ores cross-reference, 2026-10-31: about 130], so what's placed by it keeps its place
    /// whatever depth the world ends up with.
    pub crust: f32,
    /// David's made fields (`placer/mod.rs`): seen in the World Compiler's field view, and read by
    /// the distributors' candidates.
    pub fields: Vec<MadeField>,
    /// What's placed, in order: later ones go over earlier ones where they choose a block.
    pub distributors: Vec<Distributor>,
}

impl Default for MineralogySettings {
    fn default() -> Self {
        Self { crust: 130.0, fields: Vec::new(), distributors: starting_distributors() }
    }
}

/// A step layer: 1 where the field is below (or, with `above`, at least) `at`, mixed in by `mode`.
fn step(key: &str, at: f32, above: bool, wander: Option<Wander>, mode: Mode) -> Layer {
    Layer { source: Source::Field { key: key.into() }, wander, curve: Curve::Step { at, above, soft: 0.0 }, mode, ..Layer::default() }
}

fn candidate(block: &str, under: &[(&str, i32)], layers: Vec<Layer>) -> Candidate {
    let under = under.iter().map(|&(block, depth)| Under { block: block.into(), depth }).collect();
    Candidate { block: block.into(), puts: Puts::Column { under, top_only: false }, layers, ..Candidate::default() }
}

/// Mineralogy's first rules as distributors, in David's numbers (his preset of 2026-10-31): the
/// surface, the sea floor, and river and lake beds.
pub fn starting_distributors() -> Vec<Distributor> {
    use Mode::{Multiply, Normal};
    // The zones' edges wander 0.3 in temperature and rain, in patches of 512 blocks (the seeds keep
    // the first rules' patterns).
    let warmth = || Some(Wander { amount: 0.3, size: 512.0, seed: 0x5e0f });
    let wet = || Some(Wander { amount: 0.3, size: 512.0, seed: 0x7a1d });
    let soil = [("incus:dirt", 2)];
    let sand = [("incus:sand", 0)];
    let beach_top = (SEA_LEVEL + 1) as f32;
    let surface = Distributor {
        name: "the surface".into(),
        candidates: vec![
            candidate("incus:snowy_ground", &soil, vec![step("climate/temperature", -0.45, false, warmth(), Normal)]),
            candidate("incus:gabbro", &[], vec![step("ground/height", 333.0, true, None, Normal)]),
            candidate("incus:stone", &[], vec![step("ground/height", 256.0, true, None, Normal)]),
            candidate("incus:sand", &sand, vec![step("ground/height", beach_top, false, None, Normal)]),
            candidate("incus:sand", &sand, vec![step("climate/rain", 0.0, false, wet(), Normal), step("climate/temperature", 0.110_000_02, true, warmth(), Multiply)]),
            candidate("incus:dry_grass", &soil, vec![step("climate/rain", 0.210_000_04, false, wet(), Normal)]),
            candidate("incus:grass", &soil, Vec::new()),
        ],
        ..Distributor::default()
    };
    // Under the sea; the shallows are no more than 8 blocks deep (from Z 120).
    let sea = SEA_LEVEL as f32;
    let shallows = sea - 8.0;
    let sea_floor = Distributor {
        name: "the sea floor".into(),
        candidates: vec![
            candidate("incus:sand", &sand, vec![step("ground/height", sea, false, None, Normal), step("ground/height", shallows, true, None, Multiply)]),
            candidate("incus:gravel", &[("incus:gravel", 0)], vec![step("ground/height", sea, false, None, Normal), step("ground/height", shallows, false, None, Multiply)]),
            candidate("incus:stone", &[], vec![step("ground/height", sea, false, None, Normal)]),
        ],
        ..Distributor::default()
    };
    let bed = |block: &str| Candidate { block: block.into(), puts: Puts::Column { under: Vec::new(), top_only: true }, layers: vec![step("rivers/under_water", 0.5, true, None, Normal)], ..Candidate::default() };
    let beds = Distributor { name: "river and lake beds".into(), candidates: vec![bed("incus:gravel"), bed("incus:sand")], ..Distributor::default() };
    vec![surface, sea_floor, beds]
}

/// The climate where there is none (a world from before climate, the test world): mild and
/// middling wet, which is grass, as the ground always was.
pub const NO_CLIMATE: (f32, f32) = (0.2, 0.6);

/// The most blocks a column profile can have under its top.
pub const MOST_UNDER: usize = 4;

/// What a column is made of at its top: the surface block, then the blocks under it, each down to
/// its depth, then stone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layers {
    pub top: BlockId,
    /// The blocks under the top and the depth below the surface each goes down to, deepest last
    /// (`count` of them; the rest are `AIR` at 0).
    under: [(BlockId, i32); MOST_UNDER],
    count: u8,
    /// How deep they go: stone below.
    pub depth: i32,
}

impl Layers {
    /// Only `top`, stone under it.
    pub fn rock(top: BlockId) -> Self {
        Self { top, under: [(BlockId::AIR, 0); MOST_UNDER], count: 0, depth: 0 }
    }

    /// `top`, then each block under it so many blocks deep (none of no depth), then stone.
    pub fn of(top: BlockId, under: impl IntoIterator<Item = (BlockId, i32)>) -> Self {
        let mut layers = Self::rock(top);
        for (block, deep) in under.into_iter().filter(|&(_, deep)| deep > 0).take(MOST_UNDER) {
            layers.depth += deep;
            layers.under[layers.count as usize] = (block, layers.depth);
            layers.count += 1;
        }
        layers
    }

    /// The same, with another block on top (a river's bed over the soil).
    pub fn with_top(self, top: BlockId) -> Self {
        Self { top, ..self }
    }

    /// The blocks under the top, each with the depth below the surface it goes down to.
    pub fn under(&self) -> &[(BlockId, i32)] {
        &self.under[..self.count as usize]
    }

    /// The block `down` blocks below the surface (1 to `depth`).
    #[inline]
    pub fn under_at(&self, down: i32) -> BlockId {
        self.under().iter().find(|&&(_, deepest)| down <= deepest).map_or(self.top, |&(block, _)| block)
    }
}

/// Mineralogy made ready to read at any column: its distributors' blocks found in the library, and
/// every stack of layers in one `Program`.
pub struct Mineralogy {
    settings: MineralogySettings,
    blocks: TerrainBlocks,
    program: Program,
    distributors: Vec<Ready>,
    /// For each candidate's stack (after the made fields'), its distributor's place in the list.
    owners: Vec<usize>,
    /// Whether any layer reads the slope (the near ground works it out only then).
    pub reads_slope: bool,
    /// The program in whole numbers, made the first time it's read (for this map's seed).
    whole_program: std::sync::OnceLock<super::placer::WholeProgram>,
}

struct Ready {
    on: bool,
    chooser: Chooser,
    candidates: Vec<Ready1>,
}

/// A candidate made ready.
struct Ready1 {
    /// Its block, if it's registered; and whether it'll be placed at all (on, a kind this game knows).
    block: Option<BlockId>,
    placed: bool,
    /// What it puts down, and whether only on top (keeping what's under).
    profile: Layers,
    top_only: bool,
    /// Its stack in the program, and whether it has no layers that are on (it fits everywhere).
    stack: usize,
    everywhere: bool,
}

impl Ready1 {
    fn usable(&self) -> bool {
        self.placed && self.block.is_some()
    }
}

/// Why a column is made of what it is (the readout under the mouse): each distributor, each of its
/// candidates' suitability (none where it's skipped: off, or not registered), and which was chosen.
pub struct Why {
    pub distributor: String,
    pub on: bool,
    pub chooser: Chooser,
    pub candidates: Vec<(String, Option<f32>)>,
    pub chosen: Option<usize>,
}

impl Mineralogy {
    pub fn new(settings: &MineralogySettings, blocks: &TerrainBlocks) -> Self {
        let distributors = &settings.distributors;
        let stacks: Vec<Stack> = distributors.iter().enumerate().flat_map(|(d, distributor)| distributor.candidates.iter().map(move |candidate| Stack { layers: &candidate.layers, after: d })).collect();
        let owners = stacks.iter().map(|stack| stack.after).collect();
        let find = |name: &str, names: &[String], after: usize| {
            let Some(d) = distributors[..after.min(distributors.len())].iter().position(|distributor| distributor.name == name) else {
                return Err(if distributors.iter().any(|distributor| distributor.name == name) {
                    format!("\"{name}\" isn't before this one: a candidate reads only what the distributors before its own chose")
                } else {
                    format!("no distributor \"{name}\"")
                });
            };
            Ok((d, names.iter().filter_map(|name| blocks.names.get(name).copied()).collect()))
        };
        let program = Program::with_stacks(&settings.fields, settings.crust, &stacks, &find);
        let mut stack = settings.fields.len();
        let ready = distributors
            .iter()
            .map(|distributor| Ready {
                on: distributor.on,
                chooser: distributor.chooser,
                candidates: distributor
                    .candidates
                    .iter()
                    .map(|candidate| {
                        let block = blocks.names.get(&candidate.block).copied();
                        let (profile, top_only, known) = match &candidate.puts {
                            Puts::Column { under, top_only } => {
                                let under = under.iter().filter_map(|under| Some((*blocks.names.get(&under.block)?, under.depth)));
                                (Layers::of(block.unwrap_or(blocks.stone), under), *top_only, true)
                            }
                            Puts::Unknown => (Layers::rock(blocks.stone), false, false),
                        };
                        stack += 1;
                        Ready1 { block, placed: candidate.on && known, profile, top_only, stack: stack - 1, everywhere: candidate.layers.iter().all(|layer| !layer.on) }
                    })
                    .collect(),
            })
            .collect();
        Self { reads_slope: program.reads(Field::Slope), settings: settings.clone(), blocks: blocks.clone(), program, distributors: ready, owners, whole_program: std::sync::OnceLock::new() }
    }

    /// The crust's depth (blocks).
    #[cfg(test)]
    pub fn crust(&self) -> f32 {
        self.settings.crust
    }

    /// What the column is made of (in whole numbers if the map's recipe is worked in them).
    pub fn layers(&self, map: &CompiledMap, column: Column) -> Layers {
        if let Some(whole) = map.whole() {
            use crate::whole::{Fix, Pos};
            let column = super::placer::WholeColumn {
                north: Pos::from_f32(column.north),
                east: Pos::from_f32(column.east),
                ground: Fix::from_f32(column.ground),
                slope: Fix::from_f32(column.slope),
                climate: column.climate.map(|(t, r)| (Fix::from_f32(t), Fix::from_f32(r))),
                water: column.water,
            };
            let program = self.whole_program.get_or_init(|| self.program.whole(map.seed));
            let mut reading = super::placer::WholeReading::new(&whole.map, &whole.carving, column, Fix::from_f32(self.settings.crust));
            return self.whole_layers(program, &mut reading);
        }
        let mut reading = Reading::new(map, column, self.settings.crust);
        self.fill(&mut reading, self.distributors.len())
    }

    /// The distributors before `upto` run in a column, what each chose kept in `reading`; what they
    /// put down.
    fn fill(&self, reading: &mut Reading, upto: usize) -> Layers {
        let mut layers = Layers::rock(self.blocks.stone);
        reading.chosen.clear();
        for (d, distributor) in self.distributors[..upto].iter().enumerate() {
            let chosen = if distributor.on { self.choose(d, distributor, reading) } else { None };
            let picked = chosen.map(|c| &distributor.candidates[c]);
            reading.chosen.push(picked.and_then(|candidate| candidate.block).unwrap_or(BlockId::AIR));
            if let Some(candidate) = picked {
                layers = if candidate.top_only { layers.with_top(candidate.profile.top) } else { candidate.profile };
            }
        }
        layers
    }

    /// How well a candidate suits the column, at its surface.
    fn suits(&self, candidate: &Ready1, reading: &mut Reading) -> f32 {
        if candidate.everywhere { 1.0 } else { self.program.value(candidate.stack, reading, reading.column.ground) }
    }

    fn choose(&self, d: usize, distributor: &Ready, reading: &mut Reading) -> Option<usize> {
        let usable = distributor.candidates.iter().enumerate().filter(|(_, candidate)| candidate.usable());
        match distributor.chooser {
            Chooser::FirstFit { at } => {
                for (c, candidate) in usable {
                    if self.suits(candidate, reading) >= at {
                        return Some(c);
                    }
                }
                None
            }
            Chooser::Best => usable.map(|(c, candidate)| (c, self.suits(candidate, reading))).filter(|&(_, s)| s > 0.0).fold(None, |best: Option<(usize, f32)>, (c, s)| if best.is_some_and(|(_, b)| b >= s) { best } else { Some((c, s)) }).map(|(c, _)| c),
            Chooser::Chance => {
                let weights: Vec<(usize, f32)> = usable.map(|(c, candidate)| (c, self.suits(candidate, reading).max(0.0))).filter(|&(_, s)| s > 0.0).collect();
                let total: f32 = weights.iter().map(|&(_, s)| s).sum();
                let Column { north, east, .. } = reading.column;
                let roll = (crate::fate::planet_hash(reading.map.seed, 0x91ac_ed15 + d as u64, north.floor() as i64, east.floor() as i64, 0) >> 40) as f32 / (1u64 << 24) as f32 * total;
                let mut sum = 0.0;
                weights.iter().find(|&&(_, s)| {
                    sum += s;
                    roll < sum
                })
                .or(weights.last())
                .map(|&(c, _)| c)
            }
            Chooser::Unknown => None,
        }
    }

    /// Packed for the card (`card_recipe`): the program, then how many distributors and, for each: on, its
    /// chooser (0 first fit, 1 best, 2 chance, 3 unknown), the first fit's mark, how many candidates and
    /// where they start, and its roll's start (the planet hash of the seed and its salt, as two halves);
    /// each candidate: whether it's usable, whether it fits everywhere, its stack, its block. Returns
    /// where it starts.
    pub fn pack(&self, program: &super::placer::WholeProgram, seed: u64, pack: &mut super::card_recipe::Pack) -> i32 {
        let program_at = program.pack(pack);
        let mut table = Vec::new();
        for (d, distributor) in self.distributors.iter().enumerate() {
            let (chooser, at) = match distributor.chooser {
                Chooser::FirstFit { at } => (0, crate::whole::Fix::from_f32(at).0),
                Chooser::Best => (1, 0),
                Chooser::Chance => (2, 0),
                Chooser::Unknown => (3, 0),
            };
            let candidates: Vec<i32> = distributor.candidates.iter().flat_map(|c| [c.usable() as i32, c.everywhere as i32, c.stack as i32, c.block.map_or(0, |b| b.0 as i32)]).collect();
            let start = crate::fate::roll_start(seed, 0x91ac_ed15 + d as u64);
            let candidates_at = pack.list(&candidates);
            table.extend([distributor.on as i32, chooser, at, distributor.candidates.len() as i32, candidates_at, start as u32 as i32, (start >> 32) as u32 as i32]);
        }
        let table_at = pack.list(&table);
        pack.list(&[program_at, self.distributors.len() as i32, table_at, crate::whole::Fix::from_f32(self.settings.crust).0, self.settings.fields.len() as i32])
    }

    // ── In whole numbers (the regroup, phase 2.2) ──

    /// What the column is made of, read in whole numbers (`layers`): `program` is this mineralogy's
    /// `program().whole(seed)`.
    pub fn whole_layers(&self, program: &super::placer::WholeProgram, reading: &mut super::placer::WholeReading) -> Layers {
        let mut layers = Layers::rock(self.blocks.stone);
        for (distributor, chosen) in self.distributors.iter().zip(self.whole_chosen(program, reading)) {
            if let Some(candidate) = chosen.map(|c| &distributor.candidates[c]) {
                layers = if candidate.top_only { layers.with_top(candidate.profile.top) } else { candidate.profile.clone() };
            }
        }
        layers
    }

    /// Which candidate each distributor chose (in whole numbers), in order.
    pub fn whole_chosen(&self, program: &super::placer::WholeProgram, reading: &mut super::placer::WholeReading) -> Vec<Option<usize>> {
        reading.chosen.clear();
        let mut chosen_all = Vec::with_capacity(self.distributors.len());
        for (d, distributor) in self.distributors.iter().enumerate() {
            let chosen = if distributor.on { self.whole_choose(d, distributor, program, reading) } else { None };
            reading.chosen.push(chosen.and_then(|c| distributor.candidates[c].block).unwrap_or(BlockId::AIR));
            chosen_all.push(chosen);
        }
        chosen_all
    }

    fn whole_suits(&self, candidate: &Ready1, program: &super::placer::WholeProgram, reading: &mut super::placer::WholeReading) -> crate::whole::Fix {
        if candidate.everywhere { crate::whole::Fix::ONE } else { program.value(candidate.stack, reading, reading.column.ground) }
    }

    fn whole_choose(&self, d: usize, distributor: &Ready, program: &super::placer::WholeProgram, reading: &mut super::placer::WholeReading) -> Option<usize> {
        use crate::whole::Fix;
        let usable = distributor.candidates.iter().enumerate().filter(|(_, candidate)| candidate.usable());
        match distributor.chooser {
            Chooser::FirstFit { at } => {
                let at = Fix::from_f32(at);
                for (c, candidate) in usable {
                    if self.whole_suits(candidate, program, reading) >= at {
                        return Some(c);
                    }
                }
                None
            }
            Chooser::Best => usable
                .map(|(c, candidate)| (c, self.whole_suits(candidate, program, reading)))
                .filter(|&(_, s)| s > Fix::ZERO)
                .fold(None, |best: Option<(usize, Fix)>, (c, s)| if best.is_some_and(|(_, b)| b >= s) { best } else { Some((c, s)) })
                .map(|(c, _)| c),
            Chooser::Chance => {
                let weights: Vec<(usize, Fix)> = usable.map(|(c, candidate)| (c, self.whole_suits(candidate, program, reading).max(Fix::ZERO))).filter(|&(_, s)| s > Fix::ZERO).collect();
                let total = weights.iter().fold(Fix::ZERO, |sum, &(_, s)| sum + s);
                let (north, east) = (reading.column.north.0 >> 8, reading.column.east.0 >> 8);
                let hash = crate::fate::planet_hash(reading.map_seed(), 0x91ac_ed15 + d as u64, north as i64, east as i64, 0);
                let roll = Fix((((hash >> 40) as i64 * total.0 as i64) >> 24) as i32);
                let mut sum = Fix::ZERO;
                weights
                    .iter()
                    .find(|&&(_, s)| {
                        sum = sum + s;
                        roll < sum
                    })
                    .or(weights.last())
                    .map(|&(c, _)| c)
            }
            Chooser::Unknown => None,
        }
    }

    /// Why the column is made of what it is: every candidate's suitability, in each distributor.
    pub fn why(&self, map: &CompiledMap, column: Column, name: &dyn Fn(&str) -> String) -> Vec<Why> {
        let mut reading = Reading::new(map, column, self.settings.crust);
        let mut whys = Vec::new();
        for (d, (distributor, settings)) in self.distributors.iter().zip(&self.settings.distributors).enumerate() {
            self.fill(&mut reading, d);
            let candidates = distributor.candidates.iter().zip(&settings.candidates).map(|(candidate, set)| (name(&set.block), candidate.usable().then(|| self.suits(candidate, &mut reading)))).collect();
            let chosen = if distributor.on { self.choose(d, distributor, &mut reading) } else { None };
            whys.push(Why { distributor: settings.name.clone(), on: distributor.on, chooser: distributor.chooser, candidates, chosen });
        }
        whys
    }

    /// Every stack of layers, made ready: the made fields (`0..`), then each candidate's in order.
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// A column read for stack `k`: a candidate's, with what the distributors before its own chose
    /// there already.
    pub fn reading<'a>(&self, map: &'a CompiledMap, column: Column, k: usize) -> Reading<'a> {
        let mut reading = Reading::new(map, column, self.settings.crust);
        if let Some(&d) = k.checked_sub(self.settings.fields.len()).and_then(|n| self.owners.get(n)) {
            self.fill(&mut reading, d);
        }
        reading
    }
}

impl MineralogySettings {
    /// Every stack of layers by name, as `Mineralogy::program` has them: the made fields, then each
    /// candidate's ("the surface: incus:sand").
    pub fn stacks(&self) -> Vec<(String, &[Layer])> {
        let fields = self.fields.iter().map(|made| (made.name.clone(), made.layers.as_slice()));
        let candidates = self.distributors.iter().flat_map(|distributor| distributor.candidates.iter().map(move |candidate| (format!("{}: {}", distributor.name, candidate.block), candidate.layers.as_slice())));
        fields.chain(candidates).collect()
    }
}

/// Where a map keeps its mineralogy made ready (made again when its settings or the blocks change).
#[derive(Default)]
pub struct Kept(Mutex<Option<Arc<Mineralogy>>>);

impl std::fmt::Debug for Kept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Kept(..)")
    }
}

impl CompiledMap {
    /// Mineralogy made ready for these blocks: once, then kept, for every column a chunk, a distant
    /// tile or the close-up asks about.
    pub fn mineralogy(&self, blocks: &TerrainBlocks) -> Arc<Mineralogy> {
        let mut kept = self.mineralogy_kept.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(ready) = kept.as_ref()
            && ready.settings == self.settings.mineralogy
            && ready.blocks == *blocks
        {
            return ready.clone();
        }
        let ready = Arc::new(Mineralogy::new(&self.settings.mineralogy, blocks));
        *kept = Some(ready.clone());
        ready
    }

    /// The layers of the column at (`x` north, `y` east) whose ground is at `z`, in the world's own
    /// climate, flat and dry (for tests).
    #[cfg(test)]
    pub fn layers(&self, x: i32, y: i32, z: i32, blocks: &TerrainBlocks) -> Layers {
        let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
        let column = Column { north, east, ground: z as f32, slope: 0.0, climate: self.climate(north, east, z as f32), water: None };
        self.mineralogy(blocks).layers(self, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::interpreter::{InterpreterSettings, Maps};
    use crate::worldgen::noise::{Fade, perlin};
    use std::collections::HashMap;

    const NAMES: [&str; 9] = ["incus:gabbro", "incus:stone", "incus:grass", "incus:water", "incus:dirt", "incus:sand", "incus:dry_grass", "incus:snowy_ground", "incus:gravel"];

    /// Every block registered, numbered by hand (1 gabbro, 2 stone, 3 grass, 4 water, 5 dirt, 6 sand,
    /// 7 dry grass, 8 snowy ground, 9 gravel), but those named in `missing`.
    fn blocks_but(missing: &[&str]) -> TerrainBlocks {
        let names: HashMap<String, BlockId> = NAMES.iter().enumerate().filter(|(_, name)| !missing.contains(name)).map(|(k, name)| (name.to_string(), BlockId(k as u16 + 1))).collect();
        TerrainBlocks { gabbro: BlockId(1), stone: BlockId(2), grass: BlockId(3), water: BlockId(4), names: Arc::new(names) }
    }

    fn all_blocks() -> TerrainBlocks {
        blocks_but(&[])
    }

    /// A flat map with one temperature and one rain everywhere (or no climate), cooling `cooling` for
    /// every 100 blocks up.
    fn cooled(climate: Option<(f32, f32)>, cooling: f32) -> CompiledMap {
        let width = 8;
        let (temperature, rain) = climate.map_or((Vec::new(), Vec::new()), |(t, r)| (vec![t; width * width], vec![r; width * width]));
        let maps = Maps { width, blocks_per_pixel: 32.0, temperature, rain, cooling, coast: vec![1.0; width * width], height: vec![0.0; width * width], tilt: vec![0.0; width * width], squeeze: vec![0.0; width * width], push_x: vec![0.0; width * width], push_y: vec![0.0; width * width], push_strength: vec![0.0; width * width], coast_distance: vec![100.0; width * width], lift_distance: vec![1_000.0; width * width], ..Maps::default() };
        // In fractions: the first rules, kept word for word below, wandered by fractional noise.
        CompiledMap::new(Arc::new(maps), 1, InterpreterSettings { noise_maths: crate::worldgen::noise::NoiseMaths::Fractions, recipe_maths: crate::worldgen::noise::NoiseMaths::Fractions, ..InterpreterSettings::default() })
    }

    fn map(climate: Option<(f32, f32)>) -> CompiledMap {
        cooled(climate, 0.0)
    }

    /// The first rules as they were written before the placer (with David's numbers), word for word,
    /// river beds and all: what the distributors must give, block for block.
    fn first_rules(map: &CompiledMap, x: i32, y: i32, z: i32, blocks: &TerrainBlocks, climate: Option<(f32, f32)>, river: bool) -> Layers {
        let id = |name: &str| blocks.names.get(name).copied();
        let (dirt, sand, dry_grass, snow, gravel) = (id("incus:dirt"), id("incus:sand"), id("incus:dry_grass"), id("incus:snowy_ground"), id("incus:gravel"));
        let (snow_below, dry_below, desert_below, desert_above, beach, dirt_depth, sand_depth, shallows, wander, wander_size) = (-0.45, 0.210_000_04, 0.0, 0.110_000_02, 1, 2, 0, 8, 0.3, 512.0);
        let (stone_from, gabbro_from) = (256, 333);
        let first = || {
            let stone = Layers::rock(blocks.stone);
            if z < SEA_LEVEL {
                let deep = SEA_LEVEL - z;
                let floor = if deep <= shallows { sand } else { gravel };
                return floor.map_or(stone, |floor| Layers::of(floor, [(floor, sand_depth)]));
            }
            let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
            let (temperature, rain) = climate.unwrap_or(NO_CLIMATE);
            let size: f32 = wander_size;
            let wobble = |salt: u64| perlin(map.seed, salt, Fade::Smooth, north / size, east / size) * wander;
            let (temperature, rain) = (temperature + wobble(0x5e0f), rain + wobble(0x7a1d));
            let earth = |top: BlockId| Layers::of(top, dirt.map(|dirt| (dirt, dirt_depth)));
            let sand_on = |sand: BlockId| Layers::of(sand, [(sand, sand_depth)]);
            if let Some(snow) = snow.filter(|_| temperature < snow_below) {
                return earth(snow);
            }
            if z >= gabbro_from {
                return Layers::rock(blocks.gabbro);
            }
            if z >= stone_from {
                return stone;
            }
            if let Some(beach) = sand.filter(|_| z < SEA_LEVEL + beach) {
                return sand_on(beach);
            }
            if let Some(desert) = sand.filter(|_| rain < desert_below && temperature > desert_above) {
                return sand_on(desert);
            }
            if let Some(dry) = dry_grass.filter(|_| rain < dry_below) {
                return earth(dry);
            }
            earth(blocks.grass)
        };
        let layers = first();
        if river { Layers { top: gravel.or(sand).unwrap_or(layers.top), ..layers } } else { layers }
    }

    #[test]
    fn the_distributors_give_the_first_rules_world_block_for_block() {
        // Every climate, height and river, with every block and with each missing, over enough of the
        // map for the wander to swing both ways.
        let missing: [&[&str]; 4] = [&[], &["incus:dirt"], &["incus:sand", "incus:gravel"], &["incus:snowy_ground", "incus:dry_grass", "incus:dirt"]];
        let mut checked = 0;
        let mut kinds = std::collections::HashSet::new();
        for gone in missing {
            let blocks = blocks_but(gone);
            for climate in [None, Some((-0.9, 0.5)), Some((-0.4, 0.4)), Some((0.0, 0.05)), Some((0.2, 0.2)), Some((0.3, 0.6)), Some((0.6, 0.0)), Some((0.15, 0.25))] {
                let map = map(climate);
                let ready = map.mineralogy(&blocks);
                for z in [100, 119, 120, 121, 126, 128, 129, 130, 150, 255, 256, 300, 332, 333, 500, 900] {
                    for (x, y) in (0..24).map(|k| (k * 211 - 2_000, k * 157 - 1_500)) {
                        for river in [false, true] {
                            let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
                            let column = Column { north, east, ground: z as f32, slope: 0.0, climate, water: river.then_some(z + 3) };
                            let new = ready.layers(&map, column);
                            let old = first_rules(&map, x, y, z, &blocks, climate, river);
                            for down in -1..=4 {
                                let at = z - down;
                                assert_eq!(new.block_at(at, z, 0, &blocks), old.block_at(at, z, 0, &blocks), "{gone:?} {climate:?} Z {z} at ({x}, {y}), river {river}, {down} down");
                            }
                            kinds.insert(new.top);
                            checked += 1;
                        }
                    }
                }
            }
        }
        // Gabbro, stone, grass, sand, dry grass, snowy ground and gravel.
        assert_eq!(kinds.len(), 7, "every kind of surface turned up");
        assert!(checked > 20_000);
    }

    /// Mineralogy read in whole numbers chooses as it does in fractions (with the same whole-number noise):
    /// every climate and height, river beds too.
    #[test]
    fn mineralogy_in_whole_numbers_chooses_the_same_blocks() {
        use crate::whole::{Fix, Pos};
        use crate::world::placer::{WholeColumn, WholeReading};
        use crate::world::whole_recipe::{WholeCarving, WholeMap};
        let blocks = all_blocks();
        let (mut same, mut count) = (0, 0);
        for climate in [None, Some((-0.9, 0.5)), Some((-0.4, 0.4)), Some((0.0, 0.05)), Some((0.2, 0.2)), Some((0.3, 0.6)), Some((0.6, 0.0)), Some((0.15, 0.25))] {
            let mut map = cooled(climate, 0.1);
            map.settings.noise_maths = crate::worldgen::noise::NoiseMaths::WholeNumbers;
            map.settings.recipe_maths = crate::worldgen::noise::NoiseMaths::Fractions;
            let map = CompiledMap::new(map.maps.clone(), map.seed, map.settings.clone());
            let ready = map.mineralogy(&blocks);
            let (whole, carving) = (WholeMap::new(&map), WholeCarving::default());
            let program = ready.program().whole(map.seed);
            for z in [100, 119, 120, 121, 126, 128, 129, 130, 150, 255, 256, 300, 332, 333, 500, 900] {
                for (x, y) in (0..40).map(|k| (k * 211 - 2_000, k * 157 - 1_500)) {
                    for river in [false, true] {
                        let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
                        let water = river.then_some(z + 3);
                        let column = Column { north, east, ground: z as f32, slope: 0.0, climate: map.climate(north, east, z as f32), water };
                        let want = ready.layers(&map, column);
                        let whole_column = WholeColumn { north: Pos::middle_of(x), east: Pos::middle_of(y), ground: Fix::from_int(z), slope: Fix::ZERO, climate: whole.climate(Pos::middle_of(x), Pos::middle_of(y), Fix::from_int(z)), water };
                        let mut reading = WholeReading::new(&whole, &carving, whole_column, Fix::from_f32(ready.crust()));
                        let got = ready.whole_layers(&program, &mut reading);
                        same += (got == want) as usize;
                        count += 1;
                    }
                }
            }
        }
        eprintln!("{same} of {count} columns made of the same");
        // (A column right on an edge of the wander can tip either way: it's the same rules.)
        assert!(same * 100 >= count * 98, "{same} of {count} columns made of the same");
    }

    #[test]
    fn the_climate_picks_the_surface_and_higher_is_colder() {
        let blocks = all_blocks();
        let top = |climate, z| map(climate).layers(0, 0, z, &blocks).top;
        // Mild and wet: grass, with dirt under it.
        assert_eq!(map(Some((0.2, 0.9))).layers(0, 0, 150, &blocks), Layers::of(BlockId(3), [(BlockId(5), 2)]));
        assert_eq!(top(Some((-0.9, 0.6)), 150), BlockId(8));
        assert_eq!(top(Some((0.2, 0.9)), 400), blocks.gabbro);
        assert_eq!(top(Some((0.2, 0.9)), 129), BlockId(3), "past the beach");
        assert_eq!(top(Some((0.2, 0.9)), 128), BlockId(6), "the beach");
        assert_eq!(top(Some((0.2, 0.9)), 100), BlockId(9), "the deep sea floor");
        // Mild at sea level, cooling 0.3 per 100 blocks: snowy higher up.
        let mut map = cooled(Some((0.6, 0.9)), 0.3);
        map.settings.mineralogy.distributors[0].candidates.remove(2);
        map.settings.mineralogy.distributors[0].candidates.remove(1);
        assert_eq!(map.layers(0, 0, 150, &blocks).top, blocks.grass);
        assert_eq!(map.layers(0, 0, 600, &blocks).top, BlockId(8));
    }

    #[test]
    fn choosers_pick_the_first_that_fits_the_best_or_by_chance() {
        let blocks = all_blocks();
        let map = map(Some((0.2, 0.5)));
        // Sand a little suited (rain itself: 0.5), gravel well suited (0.8), grass everywhere.
        let number = |value: f32| Layer { source: Source::Number { value }, ..Layer::default() };
        let sand = candidate("incus:sand", &[], vec![number(0.5)]);
        let gravel = candidate("incus:gravel", &[], vec![number(0.8)]);
        let mut settings = MineralogySettings { distributors: vec![Distributor { candidates: vec![sand, gravel, candidate("incus:grass", &[], Vec::new())], ..Distributor::default() }], ..MineralogySettings::default() };
        let column = |k: i32| Column { north: k as f32 * 7.0 + 0.5, east: k as f32 * 13.0 + 0.5, ground: 150.0, slope: 0.0, climate: None, water: None };
        let top = |settings: &MineralogySettings, k: i32| Mineralogy::new(settings, &blocks).layers(&map, column(k)).top;
        assert_eq!(top(&settings, 0), BlockId(6), "the first that fits, at 0.5");
        settings.distributors[0].chooser = Chooser::FirstFit { at: 0.6 };
        assert_eq!(top(&settings, 0), BlockId(9));
        settings.distributors[0].chooser = Chooser::Best;
        assert_eq!(top(&settings, 0), BlockId(3), "grass fits everywhere: 1");
        // By chance: about 0.5 : 0.8 : 1, the same at a spot every time.
        settings.distributors[0].chooser = Chooser::Chance;
        let ready = Mineralogy::new(&settings, &blocks);
        let tops: Vec<BlockId> = (0..2_300).map(|k| ready.layers(&map, column(k)).top).collect();
        let share = |block: u16| tops.iter().filter(|top| top.0 == block).count() as f32 / tops.len() as f32;
        assert!((share(6) - 0.5 / 2.3).abs() < 0.03 && (share(9) - 0.8 / 2.3).abs() < 0.03 && (share(3) - 1.0 / 2.3).abs() < 0.03, "{} {} {}", share(6), share(9), share(3));
        assert_eq!(tops[17], ready.layers(&map, column(17)).top);
    }

    #[test]
    fn a_later_distributor_reads_what_an_earlier_one_chose() {
        let blocks = all_blocks();
        let map = map(Some((0.2, 0.5)));
        // The first puts sand or grass by noise; the second puts gravel on top only where it's sand.
        let noise = Layer { source: Source::Noise { size: 8.0, seed: 3 }, curve: Curve::Step { at: 0.0, above: true, soft: 0.0 }, ..Layer::default() };
        let first = Distributor { name: "rock".into(), candidates: vec![candidate("incus:sand", &[("incus:dirt", 3)], vec![noise]), candidate("incus:grass", &[], Vec::new())], ..Distributor::default() };
        let reads = Layer { source: Source::Chosen { distributor: "rock".into(), blocks: vec!["incus:sand".into()] }, ..Layer::default() };
        let second = Distributor { name: "over sand".into(), candidates: vec![Candidate { puts: Puts::Column { under: Vec::new(), top_only: true }, ..candidate("incus:gravel", &[], vec![reads]) }], ..Distributor::default() };
        let settings = MineralogySettings { distributors: vec![first, second], ..MineralogySettings::default() };
        let ready = Mineralogy::new(&settings, &blocks);
        assert_eq!(ready.program().problem(1, 0), None);
        let (mut gravel, mut grass) = (0, 0);
        for k in 0..200 {
            let layers = ready.layers(&map, Column { north: k as f32 * 3.0, east: 0.5, ground: 150.0, slope: 0.0, climate: None, water: None });
            match layers.top.0 {
                9 => {
                    gravel += 1;
                    assert_eq!(layers.under(), &[(BlockId(5), 3)], "the sand's dirt kept under the gravel");
                }
                3 => grass += 1,
                other => panic!("block {other}"),
            }
        }
        assert!(gravel > 40 && grass > 40, "{gravel} {grass}");
        // Reading a distributor after its own is a problem, not a crash.
        let backwards = MineralogySettings { distributors: settings.distributors.iter().rev().cloned().collect(), ..MineralogySettings::default() };
        assert!(Mineralogy::new(&backwards, &blocks).program().problem(0, 0).is_some_and(|problem| problem.contains("isn't before")));
    }

    #[test]
    fn profiles_go_down_block_by_block_and_unregistered_blocks_are_skipped() {
        let blocks = blocks_but(&["incus:dry_grass"]);
        let layers = Layers::of(BlockId(3), [(BlockId(5), 2), (BlockId(6), 0), (BlockId(9), 3)]);
        assert_eq!(layers.depth, 5);
        let at = |z: i32| layers.block_at(z, 150, 0, &blocks);
        assert_eq!([at(151), at(150), at(149), at(148), at(147), at(145), at(144)], [BlockId::AIR, BlockId(3), BlockId(5), BlockId(5), BlockId(9), BlockId(9), blocks.stone]);
        // No dry grass: the dry land (too cool for desert) is grass.
        assert_eq!(map(Some((-0.3, 0.0))).layers(0, 0, 150, &blocks).top, blocks.grass);
        // A candidate that's off, or of a kind this game doesn't know, is skipped too.
        let mut settings = MineralogySettings::default();
        settings.distributors[0].candidates[6].on = false;
        settings.distributors[0].candidates[5].puts = Puts::Unknown;
        let ready = Mineralogy::new(&settings, &all_blocks());
        let map = map(Some((0.2, 0.0)));
        let column = Column { north: 0.5, east: 0.5, ground: 150.0, slope: 0.0, climate: Some((0.2, 0.9)), water: None };
        assert_eq!(ready.layers(&map, column).top, blocks.stone, "nothing on the surface fits: the rock");
        let why = ready.why(&map, column, &|name| name.to_string());
        assert_eq!((why[0].chosen, why[0].candidates[6].1, why[0].candidates[5].1), (None, None, None));
    }
}

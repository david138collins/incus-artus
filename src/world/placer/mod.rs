//! The placer — what goes where [David, 2026-10-31: "its going to need to be very powerful and
//! flexible"]: the one system every user that places things shares. Mineralogy is the first; trees,
//! Ecology's plants and animals, and towns grown like jigsaw pieces come after. The plan, its options
//! and its stops are in `docs/placer.md`.
//!
//! - **Fields** (`fields.rs`): every number a world has at a spot, in one table that describes itself.
//! - **Made fields** (here): David's own fields, made from others as a stack of layers, run top to
//!   bottom. Each layer is a **source** (a field, noise, another made field above it, the result so
//!   far, or a number) through a **curve** (as it is, a step, a band, or free), mixed into the result
//!   so far by a **mode** (normal, add, subtract, multiply, the lower, the higher) at a **strength**.
//!   So "copper in a band of depth, minus wet ground" is two layers.
//! - **A `Program`** is a user's made fields made ready to read many times: every source looked up,
//!   every curve built, the ranges worked out. The World Compiler's field view reads one at every
//!   sample; from stop 3, mineralogy at every column.
//!
//! - **Sharing** (`share.rs`, stop 4): a distributor or a made field exported to a file with the
//!   fields it reads, and imported into any preset.
//! - **Distributors** (`distribute.rs`, stop 3): what's placed. Each has candidates (blocks, from the
//!   library), and each candidate's suitability is a stack of layers like a made field's; the
//!   distributor's chooser picks one at each spot. Mineralogy's surface, sea floor and river beds are
//!   three of them.
//!
//! **The kinds of source and curve are open** (section 11 of the plan): fields made from lists of
//! points and lines, things already placed, regions and the plates come later, as more kinds. A kind
//! this game doesn't know (a newer game's) is read as `Unknown`: shown greyed and named, never
//! crashed on.
//!
//! **The linear rule:** a made field reads only fields from earlier stages, and only made fields
//! above it in the list; a candidate reads any made field, and what the distributors before its own
//! chose.

mod distribute;
mod fields;
mod share;

pub use distribute::{Candidate, Chooser, Distributor, Puts, Under};
pub use fields::{Column, FIELDS, Field, Reading, Stage, WholeColumn, WholeReading};
pub use share::{Shared, export_distributor, export_field, import};

use super::block::BlockId;
use super::interpreter::{Curve as SplineCurve, Spline};
use serde::{Deserialize, Serialize};

/// A field made from others: a stack of layers, run top to bottom.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MadeField {
    pub name: String,
    pub layers: Vec<Layer>,
}

impl Default for MadeField {
    fn default() -> Self {
        Self { name: "a new field".into(), layers: vec![Layer::default()] }
    }
}

/// One layer of a made field: its source, through its curve, mixed into the result so far.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Layer {
    /// Off: left out, to see what it does.
    pub on: bool,
    pub mode: Mode,
    /// How much of it is mixed in (0 none, 1 all).
    pub strength: f32,
    pub source: Source,
    /// Noise added to the source before its curve, so a step's or a band's edge wanders (as
    /// mineralogy's zones' edges always have) rather than following the climate's pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wander: Option<Wander>,
    pub curve: Curve,
}

impl Default for Layer {
    fn default() -> Self {
        Self { on: true, mode: Mode::Normal, strength: 1.0, source: Source::Field { key: "ground/height".into() }, wander: None, curve: Curve::AsIs }
    }
}

/// Noise added to a layer's source: up to `amount` either way, in patches about `size` blocks across,
/// each `seed` its own pattern. (Its own seeds, not the noise source's: the starting distributors'
/// wander keeps mineralogy's first patterns, seeds 24079 and 31261.)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Wander {
    pub amount: f32,
    pub size: f32,
    pub seed: u32,
}

impl Default for Wander {
    fn default() -> Self {
        Self { amount: 0.05, size: 64.0, seed: 1 }
    }
}

/// What a layer reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// A field from the table, by its key ("climate/rain").
    Field { key: String },
    /// Seeded noise, −1 to 1, in patches about `size` blocks across (each seed its own pattern).
    Noise { size: f32, seed: u32 },
    /// Another made field, above this one in the list.
    Made { name: String },
    /// The result of the layers above this one (for a curve over all of them).
    SoFar,
    /// A number, the same everywhere.
    Number { value: f32 },
    /// Which block an earlier distributor chose here, as a kind in a set: 1 if it's one of `blocks`,
    /// 0 if not ("the rock is limestone or marble"). Only a distributor's candidates read it, and
    /// only the distributors before their own.
    Chosen { distributor: String, blocks: Vec<String> },
    /// A kind this game doesn't know.
    #[serde(other)]
    Unknown,
}

/// How a layer's source is turned into what it mixes in.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum Curve {
    /// The source as it is, in its own units.
    AsIs,
    /// 1 above `at` (or below it), 0 on the other side, fading over `soft` either side of it.
    Step { at: f32, above: bool, soft: f32 },
    /// 1 at `middle`, fading smoothly to 0 at `spread` either side ("around 40 deep, give or take 15").
    Band { middle: f32, spread: f32 },
    /// A free curve: the source across its range (left to right) to 0 to 1 (up).
    Free { spline: Spline },
    /// A kind this game doesn't know.
    #[serde(other)]
    Unknown,
}

/// How a layer is mixed into the result so far, at its strength.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Takes the result's place.
    #[default]
    Normal,
    Add,
    Subtract,
    Multiply,
    /// The lower of the two.
    Lower,
    /// The higher of the two.
    Higher,
}

impl Mode {
    pub const ALL: [Mode; 6] = [Mode::Normal, Mode::Add, Mode::Subtract, Mode::Multiply, Mode::Lower, Mode::Higher];

    pub fn name(self) -> &'static str {
        match self {
            Mode::Normal => "normal",
            Mode::Add => "add",
            Mode::Subtract => "subtract",
            Mode::Multiply => "multiply",
            Mode::Lower => "the lower",
            Mode::Higher => "the higher",
        }
    }

    fn mix(self, so_far: f32, value: f32, strength: f32) -> f32 {
        match self {
            Mode::Normal => so_far + (value - so_far) * strength,
            Mode::Add => so_far + value * strength,
            Mode::Subtract => so_far - value * strength,
            Mode::Multiply => so_far * (1.0 + (value - 1.0) * strength),
            Mode::Lower => so_far + (so_far.min(value) - so_far) * strength,
            Mode::Higher => so_far + (so_far.max(value) - so_far) * strength,
        }
    }

    /// The same for ranges: the least and most the result can be, from the least and most of each.
    fn mix_range(self, (low, high): (f32, f32), (value_low, value_high): (f32, f32), s: f32) -> (f32, f32) {
        match self {
            Mode::Subtract => (low - value_high * s, high - value_low * s),
            Mode::Multiply => {
                let (a, b) = (1.0 + (value_low - 1.0) * s, 1.0 + (value_high - 1.0) * s);
                let corners = [low * a, low * b, high * a, high * b];
                (corners.iter().copied().fold(f32::MAX, f32::min), corners.iter().copied().fold(f32::MIN, f32::max))
            }
            // The rest only ever grow with both.
            _ => (self.mix(low, value_low, s), self.mix(high, value_high, s)),
        }
    }
}

/// Which step of a made field to work out: the whole stack, the result after a layer, or one
/// layer's own value (through its curve, before it's mixed in).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Upto {
    #[default]
    All,
    After(usize),
    Alone(usize),
}

/// A layer as it's read at each spot.
enum Input {
    Field(Field),
    Noise { size: f32, salt: u64 },
    Made(usize),
    SoFar,
    Number(f32),
    /// 1 if the distributor at this place in the list chose one of these blocks here.
    Chosen { distributor: usize, blocks: Vec<BlockId> },
    /// Nothing to read (something's wrong with it: its `problem` says what).
    Missing,
}

enum Shaped {
    AsIs,
    Step { at: f32, above: bool, soft: f32 },
    Band { middle: f32, spread: f32 },
    Free(SplineCurve),
    Zero,
}

struct Step {
    on: bool,
    input: Input,
    /// Noise added to the source: how much, its patches' size and its salt.
    wander: Option<(f32, f32, u64)>,
    shaped: Shaped,
    mode: Mode,
    strength: f32,
    /// The source's range (a free curve's left and right edges), and the result's after this layer.
    range_in: (f32, f32),
    range_out: (f32, f32),
    problem: Option<String>,
}

struct Compiled {
    steps: Vec<Step>,
    per_block: bool,
}

/// A user's made fields, ready to read at any spot.
pub struct Program {
    made: Vec<Compiled>,
}

/// One layer's part at a spot, for the readout under the mouse: its source's value, that through its
/// curve, and the result so far after it's mixed in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traced {
    pub source: f32,
    pub curved: f32,
    pub so_far: f32,
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A stack of layers that isn't a made field: a distributor's candidate's suitability, read after
/// every made field. `after` is its distributor's place in the list: it may read what the
/// distributors before that one chose.
pub struct Stack<'a> {
    pub layers: &'a [Layer],
    pub after: usize,
}

/// What a "what's chosen" source names, found: the distributor's place in the list (before `after`)
/// and the blocks' numbers; or what's wrong.
pub type FindChosen<'a> = dyn Fn(&str, &[String], usize) -> Result<(usize, Vec<BlockId>), String> + 'a;

impl Program {
    /// `crust`: the crust's depth (blocks), the end of depth's range.
    pub fn new(fields: &[MadeField], crust: f32) -> Self {
        Self::with_stacks(fields, crust, &[], &|_, _, _| Err("only a distributor's candidates read what's chosen".into()))
    }

    /// The made fields, then the stacks (read as made field `fields.len() + n`).
    pub fn with_stacks(fields: &[MadeField], crust: f32, stacks: &[Stack], find: &FindChosen) -> Self {
        let mut program = Self { made: Vec::with_capacity(fields.len() + stacks.len()) };
        for (k, made) in fields.iter().enumerate() {
            let compiled = program.compile(&made.layers, crust, fields, k, None, find);
            program.made.push(compiled);
        }
        for stack in stacks {
            let compiled = program.compile(stack.layers, crust, fields, fields.len(), Some(stack.after), find);
            program.made.push(compiled);
        }
        program
    }

    /// One stack, reading the made fields before `k` (all of them for a candidate's, whose
    /// distributor is at `after`).
    fn compile(&self, layers: &[Layer], crust: f32, fields: &[MadeField], k: usize, after: Option<usize>, find: &FindChosen) -> Compiled {
        let mut steps: Vec<Step> = Vec::with_capacity(layers.len());
        let mut so_far = (0.0, 0.0);
        let mut per_block = false;
        for layer in layers {
            let mut problem = None;
            let (input, range_in) = match &layer.source {
                Source::Field { key } => match Field::by_key(key) {
                    Some(field) => {
                        per_block |= layer.on && field.info().per_block;
                        (Input::Field(field), if field == Field::Depth { (0.0, crust.max(1.0)) } else { field.info().range })
                    }
                    None => {
                        problem = Some(format!("no field \"{key}\" (from a newer game?)"));
                        (Input::Missing, (0.0, 1.0))
                    }
                },
                Source::Noise { size, seed } => (Input::Noise { size: size.max(1.0), salt: 0x9_1ace_0000 + *seed as u64 }, (-1.0, 1.0)),
                Source::Made { name } => match fields[..k].iter().position(|other| &other.name == name) {
                    Some(j) => {
                        per_block |= layer.on && self.made[j].per_block;
                        (Input::Made(j), self.range(j))
                    }
                    None => {
                        problem = Some(if fields[k..].iter().any(|other| &other.name == name) {
                            format!("\"{name}\" comes after this one: a made field reads only those above it")
                        } else {
                            format!("no made field \"{name}\"")
                        });
                        (Input::Missing, (0.0, 1.0))
                    }
                },
                Source::SoFar => (Input::SoFar, so_far),
                Source::Number { value } => (Input::Number(*value), (*value, *value)),
                Source::Chosen { distributor, blocks } => match find(distributor, blocks, after.unwrap_or(0)) {
                    Ok((distributor, blocks)) if after.is_some() => (Input::Chosen { distributor, blocks }, (0.0, 1.0)),
                    Ok(_) => {
                        problem = Some("only a distributor's candidates read what's chosen".into());
                        (Input::Missing, (0.0, 1.0))
                    }
                    Err(why) => {
                        problem = Some(why);
                        (Input::Missing, (0.0, 1.0))
                    }
                },
                Source::Unknown => {
                    problem = Some("a kind of source this game doesn't know (from a newer one?)".into());
                    (Input::Missing, (0.0, 1.0))
                }
            };
            let wander = layer.wander.map(|w| (w.amount, w.size.max(1.0), w.seed as u64));
            let (shaped, curved) = match &layer.curve {
                Curve::AsIs => (Shaped::AsIs, range_in),
                Curve::Step { at, above, soft } => (Shaped::Step { at: *at, above: *above, soft: soft.max(0.0) }, (0.0, 1.0)),
                Curve::Band { middle, spread } => (Shaped::Band { middle: *middle, spread: spread.max(0.0) }, (0.0, 1.0)),
                Curve::Free { spline } => {
                    let dots = spline.dots.iter().map(|&(_, up)| up);
                    let (low, high) = dots.fold((f32::MAX, f32::MIN), |(lo, hi), up| (lo.min(up), hi.max(up)));
                    (Shaped::Free(spline.curve()), if low <= high { (low, high) } else { (0.0, 1.0) })
                }
                Curve::Unknown => {
                    problem.get_or_insert_with(|| "a kind of curve this game doesn't know (from a newer one?)".into());
                    (Shaped::Zero, (0.0, 0.0))
                }
            };
            if layer.on {
                so_far = layer.mode.mix_range(so_far, curved, layer.strength);
            }
            steps.push(Step { on: layer.on, input, wander, shaped, mode: layer.mode, strength: layer.strength, range_in, range_out: so_far, problem });
        }
        Compiled { steps, per_block }
    }

    /// Whether any layer that's on reads this field (the near ground works out the slope only if so).
    pub fn reads(&self, field: Field) -> bool {
        self.made.iter().any(|made| made.steps.iter().any(|step| step.on && matches!(step.input, Input::Field(f) if f == field)))
    }

    /// How many made fields it has.
    pub fn count(&self) -> usize {
        self.made.len()
    }

    /// The least and most made field `k` can be (where the field view's black and white are).
    pub fn range(&self, k: usize) -> (f32, f32) {
        self.made.get(k).and_then(|made| made.steps.last()).map_or((0.0, 0.0), |step| step.range_out)
    }

    /// The same after layer `layer` (or of that layer alone: its source's range through its curve).
    pub fn range_upto(&self, k: usize, upto: Upto) -> (f32, f32) {
        let Some(made) = self.made.get(k) else { return (0.0, 0.0) };
        match upto {
            Upto::All => self.range(k),
            Upto::After(layer) => made.steps.get(layer).map_or((0.0, 0.0), |step| step.range_out),
            Upto::Alone(layer) => made.steps.get(layer).map_or((0.0, 0.0), |step| match step.shaped {
                Shaped::AsIs => step.range_in,
                Shaped::Zero => (0.0, 0.0),
                _ => (0.0, 1.0),
            }),
        }
    }

    /// Layer `layer`'s source's range (where a step's or band's sliders, and a free curve's edges, are).
    pub fn source_range(&self, k: usize, layer: usize) -> (f32, f32) {
        self.made.get(k).and_then(|made| made.steps.get(layer)).map_or((0.0, 1.0), |step| step.range_in)
    }

    /// What's wrong with layer `layer`, if anything.
    pub fn problem(&self, k: usize, layer: usize) -> Option<&str> {
        self.made.get(k).and_then(|made| made.steps.get(layer)).and_then(|step| step.problem.as_deref())
    }

    /// Whether made field `k` changes down a column (it reads a field that does).
    pub fn per_block(&self, k: usize) -> bool {
        self.made.get(k).is_some_and(|made| made.per_block)
    }

    /// Made field `k` at the block `z` high in the column being read.
    pub fn value(&self, k: usize, reading: &mut Reading, z: f32) -> f32 {
        self.value_upto(k, reading, z, Upto::All)
    }

    pub fn value_upto(&self, k: usize, reading: &mut Reading, z: f32, upto: Upto) -> f32 {
        let mut so_far = 0.0;
        let Some(made) = self.made.get(k) else { return 0.0 };
        for (n, step) in made.steps.iter().enumerate() {
            if upto == Upto::Alone(n) {
                return self.curved(step, reading, z, so_far);
            }
            if step.on {
                so_far = step.mode.mix(so_far, self.curved(step, reading, z, so_far), step.strength);
            }
            if upto == Upto::After(n) {
                break;
            }
        }
        so_far
    }

    /// Every layer of made field `k` at a spot, for the readout under the mouse (`None` for a layer
    /// that's off).
    pub fn trace(&self, k: usize, reading: &mut Reading, z: f32) -> Vec<Option<Traced>> {
        let mut so_far = 0.0;
        let Some(made) = self.made.get(k) else { return Vec::new() };
        made.steps
            .iter()
            .map(|step| {
                if !step.on {
                    return None;
                }
                let source = self.source(step, reading, z, so_far);
                let curved = self.shape(step, source);
                so_far = step.mode.mix(so_far, curved, step.strength);
                Some(Traced { source, curved, so_far })
            })
            .collect()
    }

    fn curved(&self, step: &Step, reading: &mut Reading, z: f32, so_far: f32) -> f32 {
        let source = self.source(step, reading, z, so_far);
        self.shape(step, source)
    }

    fn source(&self, step: &Step, reading: &mut Reading, z: f32, so_far: f32) -> f32 {
        let value = match &step.input {
            Input::Field(field) => reading.read(*field, z),
            Input::Noise { size, salt } => reading.noise(*salt, *size),
            Input::Made(j) => self.value(*j, reading, z),
            Input::SoFar => so_far,
            Input::Number(value) => *value,
            Input::Chosen { distributor, blocks } => reading.chosen.get(*distributor).is_some_and(|chosen| blocks.contains(chosen)) as u8 as f32,
            Input::Missing => 0.0,
        };
        match step.wander {
            Some((amount, size, salt)) => value + reading.noise(salt, size) * amount,
            None => value,
        }
    }

    fn shape(&self, step: &Step, x: f32) -> f32 {
        match &step.shaped {
            Shaped::AsIs => x,
            Shaped::Step { at, above, soft } => {
                let t = if *soft > 0.0 { smooth((x - at + soft) / (2.0 * soft)) } else if x >= *at { 1.0 } else { 0.0 };
                if *above { t } else { 1.0 - t }
            }
            Shaped::Band { middle, spread } => {
                if *spread > 0.0 {
                    1.0 - smooth((x - middle).abs() / spread)
                } else if x == *middle {
                    1.0
                } else {
                    0.0
                }
            }
            Shaped::Free(curve) => {
                let (low, high) = step.range_in;
                curve.at(((x - low) / (high - low).max(1e-6)).clamp(0.0, 1.0))
            }
            Shaped::Zero => 0.0,
        }
    }
}

// ── In whole numbers (the regroup, phase 2.2) ──────────────────────────────────────────────────────

use crate::whole::{Fix, Pos};
use crate::world::whole_recipe::WholeCurve;

/// A layer as it's read, in whole numbers.
enum WholeInput {
    Field(Field),
    Noise { size: Pos, key: u32 },
    Made(usize),
    SoFar,
    Number(Fix),
    Chosen { distributor: usize, blocks: Vec<BlockId> },
    Missing,
}

enum WholeShaped {
    AsIs,
    Step { at: Fix, above: bool, soft: Fix },
    Band { middle: Fix, spread: Fix },
    Free(WholeCurve),
    Zero,
}

struct WholeStep {
    on: bool,
    input: WholeInput,
    /// Noise added to the source: how much, its patches' size and its key.
    wander: Option<(Fix, Pos, u32)>,
    shaped: WholeShaped,
    mode: Mode,
    strength: Fix,
    range_in: (Fix, Fix),
}

/// A `Program` in whole numbers: the same stacks, every number whole, the noise's keys found from the
/// world's seed.
pub struct WholeProgram {
    made: Vec<Vec<WholeStep>>,
}

impl Mode {
    fn mix_whole(self, so_far: Fix, value: Fix, strength: Fix) -> Fix {
        match self {
            Mode::Normal => so_far + (value - so_far) * strength,
            Mode::Add => so_far + value * strength,
            Mode::Subtract => so_far - value * strength,
            Mode::Multiply => so_far * (Fix::ONE + (value - Fix::ONE) * strength),
            Mode::Lower => so_far + (so_far.min(value) - so_far) * strength,
            Mode::Higher => so_far + (so_far.max(value) - so_far) * strength,
        }
    }
}

impl Program {
    /// The same program in whole numbers, for a world of this seed.
    pub fn whole(&self, seed: u64) -> WholeProgram {
        use crate::worldgen::noise::noise_key;
        let (value, place) = (Fix::from_f32, Pos::from_f32);
        let made = self
            .made
            .iter()
            .map(|compiled| {
                compiled
                    .steps
                    .iter()
                    .map(|step| WholeStep {
                        on: step.on,
                        input: match &step.input {
                            Input::Field(field) => WholeInput::Field(*field),
                            Input::Noise { size, salt } => WholeInput::Noise { size: place(*size), key: noise_key(seed, *salt) },
                            Input::Made(j) => WholeInput::Made(*j),
                            Input::SoFar => WholeInput::SoFar,
                            Input::Number(n) => WholeInput::Number(value(*n)),
                            Input::Chosen { distributor, blocks } => WholeInput::Chosen { distributor: *distributor, blocks: blocks.clone() },
                            Input::Missing => WholeInput::Missing,
                        },
                        wander: step.wander.map(|(amount, size, salt)| (value(amount), place(size), noise_key(seed, salt))),
                        shaped: match &step.shaped {
                            Shaped::AsIs => WholeShaped::AsIs,
                            Shaped::Step { at, above, soft } => WholeShaped::Step { at: value(*at), above: *above, soft: value(*soft) },
                            Shaped::Band { middle, spread } => WholeShaped::Band { middle: value(*middle), spread: value(*spread) },
                            Shaped::Free(curve) => WholeShaped::Free(curve.whole()),
                            Shaped::Zero => WholeShaped::Zero,
                        },
                        mode: step.mode,
                        strength: value(step.strength),
                        range_in: (value(step.range_in.0), value(step.range_in.1)),
                    })
                    .collect()
            })
            .collect();
        WholeProgram { made }
    }
}

impl WholeProgram {
    /// Packed for the card (`card_recipe`): how many stacks, each stack's first step and how many, then
    /// every step's sixteen numbers: on, the input's kind and its two numbers, the wander (has, amount,
    /// size, key), the curve's kind and its three numbers, the mode, the strength, and the source's range.
    /// Returns where it starts.
    pub fn pack(&self, pack: &mut crate::world::card_recipe::Pack) -> i32 {
        let mut steps: Vec<[i32; 16]> = Vec::new();
        let mut stacks = Vec::new();
        for made in &self.made {
            stacks.extend([steps.len() as i32, made.len() as i32]);
            for step in made {
                let (kind, a, b) = match &step.input {
                    WholeInput::Field(field) => (0, *field as i32, 0),
                    WholeInput::Noise { size, key } => (1, size.0, *key as i32),
                    WholeInput::Made(j) => (2, *j as i32, 0),
                    WholeInput::SoFar => (3, 0, 0),
                    WholeInput::Number(value) => (4, value.0, 0),
                    WholeInput::Chosen { distributor, blocks } => (5, *distributor as i32, pack.list(&blocks.iter().map(|b| b.0 as i32).collect::<Vec<i32>>())),
                    WholeInput::Missing => (6, 0, 0),
                };
                let (shape, s0, s1, s2) = match &step.shaped {
                    WholeShaped::AsIs => (0, 0, 0, 0),
                    WholeShaped::Step { at, above, soft } => (1, at.0, *above as i32, soft.0),
                    WholeShaped::Band { middle, spread } => (2, middle.0, spread.0, 0),
                    WholeShaped::Free(curve) => (3, pack.curve(curve) as i32, 0, 0),
                    WholeShaped::Zero => (4, 0, 0, 0),
                };
                let (w_has, w_amount, w_size, w_key) = step.wander.map_or((0, 0, 0, 0), |(amount, size, key)| (1, amount.0, size.0, key as i32));
                let mode = Mode::ALL.iter().position(|&m| m == step.mode).unwrap_or(0) as i32;
                steps.push([step.on as i32, kind, a, b, w_has, w_amount, w_size, w_key, shape, s0, s1, s2, mode, step.strength.0, step.range_in.0.0, step.range_in.1.0]);
            }
        }
        let table = pack.list(&stacks);
        let at = pack.list(&steps.concat());
        pack.list(&[self.made.len() as i32, table, at])
    }

    /// Made field `k` at the block `z` high in the column being read (`Program::value`).
    pub fn value(&self, k: usize, reading: &mut WholeReading, z: Fix) -> Fix {
        let mut so_far = Fix::ZERO;
        let Some(steps) = self.made.get(k) else { return Fix::ZERO };
        for step in steps {
            if step.on {
                let curved = self.shape(step, self.source(step, reading, z, so_far));
                so_far = step.mode.mix_whole(so_far, curved, step.strength);
            }
        }
        so_far
    }

    fn source(&self, step: &WholeStep, reading: &mut WholeReading, z: Fix, so_far: Fix) -> Fix {
        let value = match &step.input {
            WholeInput::Field(field) => reading.read(*field, z),
            WholeInput::Noise { size, key } => reading.noise(*key, *size),
            WholeInput::Made(j) => self.value(*j, reading, z),
            WholeInput::SoFar => so_far,
            WholeInput::Number(value) => *value,
            WholeInput::Chosen { distributor, blocks } => {
                if reading.chosen.get(*distributor).is_some_and(|chosen| blocks.contains(chosen)) {
                    Fix::ONE
                } else {
                    Fix::ZERO
                }
            }
            WholeInput::Missing => Fix::ZERO,
        };
        match step.wander {
            Some((amount, size, key)) => value + reading.noise(key, size) * amount,
            None => value,
        }
    }

    fn shape(&self, step: &WholeStep, x: Fix) -> Fix {
        use crate::whole::smooth;
        match &step.shaped {
            WholeShaped::AsIs => x,
            WholeShaped::Step { at, above, soft } => {
                let t = if *soft > Fix::ZERO {
                    smooth((x - *at + *soft) / Fix(soft.0.wrapping_mul(2)))
                } else if x >= *at {
                    Fix::ONE
                } else {
                    Fix::ZERO
                };
                if *above { t } else { Fix::ONE - t }
            }
            WholeShaped::Band { middle, spread } => {
                if *spread > Fix::ZERO {
                    Fix::ONE - smooth((x - *middle).abs() / *spread)
                } else if x == *middle {
                    Fix::ONE
                } else {
                    Fix::ZERO
                }
            }
            WholeShaped::Free(curve) => {
                let (low, high) = step.range_in;
                curve.at(((x - low) / (high - low).max(Fix(1))).clamp(Fix::ZERO, Fix::ONE))
            }
            WholeShaped::Zero => Fix::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::interpreter::CompiledMap;

    fn layer(source: Source, curve: Curve, mode: Mode, strength: f32) -> Layer {
        Layer { on: true, mode, strength, source, wander: None, curve }
    }

    fn field(key: &str) -> Source {
        Source::Field { key: key.into() }
    }

    /// A flat map and one column on it, the ground at Z 150, with this climate.
    fn at(map: &CompiledMap, climate: (f32, f32)) -> Reading<'_> {
        Reading::new(map, Column { north: 0.5, east: 0.5, ground: 150.0, slope: 0.0, climate: Some(climate), water: None }, 100.0)
    }

    #[test]
    fn steps_and_bands_shape_a_field_and_modes_mix_it_in() {
        let map = CompiledMap::of_heights(8, 32.0, &[150.0; 64]);
        // Rain below 0.3 (a step), with depth in a band round 40 give or take 20.
        let made = MadeField {
            name: "dry, 40 down".into(),
            layers: vec![
                layer(field("climate/rain"), Curve::Step { at: 0.3, above: false, soft: 0.0 }, Mode::Normal, 1.0),
                layer(field("ground/depth"), Curve::Band { middle: 40.0, spread: 20.0 }, Mode::Multiply, 1.0),
            ],
        };
        let program = Program::new(&[made], 100.0);
        assert_eq!(program.problem(0, 0), None);
        assert!(program.per_block(0), "depth changes down the column");
        let (dry, wet) = (at(&map, (0.0, 0.1)), at(&map, (0.0, 0.6)));
        for (mut reading, rain) in [(dry, 1.0), (wet, 0.0)] {
            assert_eq!(program.value(0, &mut reading, 110.0), rain, "40 down");
            assert_eq!(program.value(0, &mut reading, 150.0), 0.0, "at the surface");
            assert_eq!(program.value(0, &mut reading, 100.0), rain * 0.5, "halfway out of the band");
            assert_eq!(program.value_upto(0, &mut reading, 150.0, Upto::After(0)), rain);
            assert_eq!(program.value_upto(0, &mut reading, 150.0, Upto::Alone(1)), 0.0);
        }
        // Each mode at half strength: the result so far 0.6, mixing in 0.2.
        for (mode, wanted) in [(Mode::Normal, 0.4), (Mode::Add, 0.7), (Mode::Subtract, 0.5), (Mode::Multiply, 0.36), (Mode::Lower, 0.4), (Mode::Higher, 0.6)] {
            assert!((mode.mix(0.6, 0.2, 0.5) - wanted).abs() < 1e-6, "{mode:?}");
        }
        // A soft step fades across `soft` either side.
        let soft = Program::new(&[MadeField { name: "warm".into(), layers: vec![layer(field("climate/temperature"), Curve::Step { at: 0.0, above: true, soft: 0.2 }, Mode::Normal, 1.0)] }], 100.0);
        let value = |t: f32| soft.value(0, &mut at(&map, (t, 0.5)), 150.0);
        assert_eq!((value(-0.3), value(0.0), value(0.3)), (0.0, 0.5, 1.0));
    }

    #[test]
    fn a_made_field_reads_those_above_it_and_not_those_below() {
        let map = CompiledMap::of_heights(8, 32.0, &[150.0; 64]);
        let rain = MadeField { name: "rain".into(), layers: vec![layer(field("climate/rain"), Curve::AsIs, Mode::Normal, 1.0)] };
        let reads_rain = MadeField { name: "rain, less 0.1".into(), layers: vec![layer(Source::Made { name: "rain".into() }, Curve::AsIs, Mode::Normal, 1.0), layer(Source::Number { value: 0.1 }, Curve::AsIs, Mode::Subtract, 1.0)] };
        let reads_later = MadeField { name: "early".into(), layers: vec![layer(Source::Made { name: "late".into() }, Curve::AsIs, Mode::Normal, 1.0)] };
        let late = MadeField { name: "late".into(), ..MadeField::default() };
        let program = Program::new(&[rain, reads_rain, reads_later, late], 100.0);
        assert!((program.value(1, &mut at(&map, (0.0, 0.6)), 150.0) - 0.5).abs() < 1e-6);
        assert!(program.problem(2, 0).is_some_and(|problem| problem.contains("comes after")));
        assert_eq!(program.value(2, &mut at(&map, (0.0, 0.6)), 150.0), 0.0);
        // An unknown field is a problem, not a crash.
        let unknown = Program::new(&[MadeField { name: "?".into(), layers: vec![layer(field("climate/snowfall"), Curve::AsIs, Mode::Normal, 1.0)] }], 100.0);
        assert!(unknown.problem(0, 0).is_some());
    }

    #[test]
    fn ranges_follow_the_layers_down_the_stack() {
        // Temperature as it is (−1 to 1), plus noise at 0.1, then a step: 0 to 1.
        let made = MadeField {
            name: "cold".into(),
            layers: vec![
                layer(field("climate/temperature"), Curve::AsIs, Mode::Normal, 1.0),
                layer(Source::Noise { size: 48.0, seed: 1 }, Curve::AsIs, Mode::Add, 0.1),
                layer(Source::SoFar, Curve::Step { at: -0.45, above: false, soft: 0.0 }, Mode::Normal, 1.0),
            ],
        };
        let program = Program::new(&[made], 100.0);
        assert_eq!(program.range_upto(0, Upto::After(0)), (-1.0, 1.0));
        let (low, high) = program.range_upto(0, Upto::After(1));
        assert!((low + 1.1).abs() < 1e-6 && (high - 1.1).abs() < 1e-6);
        assert_eq!(program.source_range(0, 2), (low, high), "the result so far's range");
        assert_eq!(program.range(0), (0.0, 1.0));
        assert!(!program.per_block(0));
    }

    #[test]
    fn made_fields_keep_through_a_preset_and_unknown_kinds_are_kept_out() {
        let made = MadeField {
            name: "copper".into(),
            layers: vec![
                layer(field("ground/depth"), Curve::Band { middle: 40.0, spread: 15.0 }, Mode::Normal, 1.0),
                layer(Source::Noise { size: 30.0, seed: 4 }, Curve::Free { spline: Spline::through(&[(0.0, 0.0), (0.5, 0.2), (1.0, 1.0)]) }, Mode::Subtract, 0.5),
                layer(Source::SoFar, Curve::Step { at: 0.5, above: true, soft: 0.1 }, Mode::Normal, 1.0),
            ],
        };
        let back: MadeField = toml::from_str(&toml::to_string(&made).unwrap()).unwrap();
        assert_eq!(back, made);
        // A newer game's kinds of source and curve.
        let newer = "name = \"x\"\n[[layers]]\nsource = { kind = \"near\", list = \"rivers/mouths\" }\ncurve = { shape = \"kinds\", set = [\"marble\"] }\n";
        let made: MadeField = toml::from_str(newer).unwrap();
        assert_eq!((&made.layers[0].source, &made.layers[0].curve), (&Source::Unknown, &Curve::Unknown));
        assert!(Program::new(&[made], 100.0).problem(0, 0).is_some());
    }
}

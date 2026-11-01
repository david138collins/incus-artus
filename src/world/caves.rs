//! Caves — openings taken out of the ground [`docs/caves.md`; David, 2026-11-01: "i would start with
//! a simple, dead end cave, that worms a bit into the side of a mountain and stops"].
//!
//! A cave is **planned, then carved**, as the rivers are. The plan is a list of tunnels, each a line
//! of points with a width and a height at each (half across, in 16ths of a block). The column recipe
//! makes the ground first (`terrain.rs`); then every block inside a tunnel becomes air. Nothing earlier
//! changes: the ground's height, the light map, the far view and the climate never see caves.
//!
//! **"Am I inside a cave?"** The plan is sorted into section-sized boxes (32 blocks), each listing the
//! pieces of tunnel near it, so most of the underground answers "nothing here" at once, and the
//! uniform-stone shortcut stays wherever no cave is near. Inside a box, each piece is a line from one
//! point to the next:
//! - the nearest place on it, how far along (in 256ths);
//! - its width and height there (blended from its two ends), each pushed in or out by the walls' lumps
//!   (`whole_value3`), so it isn't a pipe;
//! - an oval across: wide `width`, tall `height` (measured flat and up, so a tunnel should lie closer
//!   to flat than upright), with its floor cut flat `floor` of the way down.
//!
//! **GPU-safe** [the Metaphysics Engine, `docs/decisions.md`]: the carve is whole numbers in 32 bits,
//! with no square roots, so a chip doing it itself gets every block the same. That holds for half
//! widths and heights up to `MOST_ACROSS` and pieces up to `LONGEST_PIECE` long, which the plan keeps
//! to. Making the plan (where the tunnel goes) is world creation, which can use fractions.
//!
//! **Stop 1** has one kind: the test cave, placed by a button at the crosshair (the World page). It's
//! saved with the world (`caves.toml`), so chunks made again from the recipe have it too.

use super::block::BlockId;
use super::chunk::Chunk;
use super::position::{CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTIONS_PER_CHUNK, z_in_world};
use crate::worldgen::noise::whole_value3;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 16ths of a block: the plan's positions and sizes.
pub const SIXTEENTH: i32 = 16;
/// The widest and tallest a tunnel may be, half across (16ths: 8 blocks), so the carve fits 32 bits.
pub const MOST_ACROSS: i32 = 8 * SIXTEENTH;
/// The longest one piece of tunnel may be (16ths: 16 blocks), for the same reason.
pub const LONGEST_PIECE: i32 = 16 * SIXTEENTH;

/// One point along a tunnel: where (north, east, up, in 16ths of a block), and half its width and its
/// height there (16ths).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CavePoint {
    pub at: [i32; 3],
    pub width: i32,
    pub height: i32,
}

/// A tunnel: its points, and the look of its walls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tunnel {
    pub points: Vec<CavePoint>,
    /// How far the walls are pushed in or out (16ths of a block), and how big their lumps are (blocks).
    pub rough: i32,
    pub lump: i32,
    /// How far down its floor is cut flat, as a share of its height (256ths: 256 = no flat floor).
    pub floor: i32,
    /// Its walls' own noise.
    pub seed: u32,
}

/// Every cave in the world, and where each is.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CavePlan {
    pub tunnels: Vec<Tunnel>,
    /// Each section-sized box (chunk north, chunk east, level) → the pieces of tunnel near it (tunnel,
    /// and the piece's first point). Made again from the tunnels after loading.
    #[serde(skip)]
    index: HashMap<(i32, i32, i32), Vec<(u32, u32)>>,
}

impl PartialEq for CavePlan {
    fn eq(&self, other: &Self) -> bool {
        self.tunnels == other.tunnels
    }
}

/// The plan the world is using, shared with the chunk-making threads.
#[derive(Resource, Clone, Default)]
pub struct Caves(pub Arc<CavePlan>);

impl CavePlan {
    /// A plan of these tunnels, sorted into boxes.
    pub fn new(tunnels: Vec<Tunnel>) -> Self {
        let mut plan = Self { tunnels, index: HashMap::new() };
        plan.make_index();
        plan
    }

    pub fn is_empty(&self) -> bool {
        self.tunnels.is_empty()
    }

    /// Sorts the pieces into boxes. A piece too long or too wide for the carve's 32 bits (only from a
    /// hand-edited `caves.toml`) is left out, so it's never carved at all.
    fn make_index(&mut self) {
        self.index.clear();
        for (t, tunnel) in self.tunnels.iter().enumerate() {
            let fits = |point: &CavePoint| (0..=MOST_ACROSS).contains(&point.width) && (0..=MOST_ACROSS).contains(&point.height);
            if !(0..=MOST_ACROSS).contains(&tunnel.rough) || tunnel.lump < 1 {
                continue;
            }
            for p in 0..tunnel.points.len().saturating_sub(1) {
                let (a, b) = (tunnel.points[p], tunnel.points[p + 1]);
                if !fits(&a) || !fits(&b) || (0..3).any(|k| (b.at[k] - a.at[k]).abs() > LONGEST_PIECE) {
                    continue;
                }
                for key in piece_boxes(tunnel, p) {
                    self.index.entry(key).or_default().push((t as u32, p as u32));
                }
            }
        }
    }

    /// Does any cave come near section `level` of this chunk?
    pub fn touches(&self, chunk: ChunkPos, level: usize) -> bool {
        self.index.contains_key(&(chunk.x, chunk.y, level as i32))
    }

    /// The chunks a tunnel comes near.
    pub fn chunks_near(tunnel: &Tunnel) -> HashSet<ChunkPos> {
        (0..tunnel.points.len().saturating_sub(1)).flat_map(|p| piece_boxes(tunnel, p)).map(|(x, y, _)| ChunkPos::new(x, y)).collect()
    }

    /// Is this block inside a cave?
    pub fn inside(&self, block: IVec3) -> bool {
        let key = (block.x.div_euclid(CHUNK_SIZE), block.y.div_euclid(CHUNK_SIZE), block.z.div_euclid(SECTION_SIZE));
        self.index.get(&key).is_some_and(|pieces| pieces.iter().any(|&(t, p)| inside_piece(&self.tunnels[t as usize], p as usize, block)))
    }

    /// The plan as the world's `caves.toml`.
    pub fn to_text(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }

    pub fn from_text(text: &str) -> Result<Self, String> {
        let plan: Self = toml::from_str(text).map_err(|error| error.to_string())?;
        Ok(Self::new(plan.tunnels))
    }
}

/// The section-sized boxes a piece of tunnel (from point `p` to the next) reaches into: its line's
/// box, out by its widest, its walls' lumps and a block more (so a block beside a carved one, whose
/// face now shows, is in a box that's made again too).
fn piece_boxes(tunnel: &Tunnel, p: usize) -> Vec<(i32, i32, i32)> {
    let (a, b) = (tunnel.points[p], tunnel.points[p + 1]);
    let reach = a.width.max(a.height).max(b.width).max(b.height) + tunnel.rough.abs() + SIXTEENTH;
    let low: [i32; 3] = std::array::from_fn(|k| (a.at[k].min(b.at[k]) - reach).div_euclid(SIXTEENTH));
    let high: [i32; 3] = std::array::from_fn(|k| (a.at[k].max(b.at[k]) + reach).div_euclid(SIXTEENTH));
    let size = [CHUNK_SIZE, CHUNK_SIZE, SECTION_SIZE];
    let mut boxes = Vec::new();
    for x in low[0].div_euclid(size[0])..=high[0].div_euclid(size[0]) {
        for y in low[1].div_euclid(size[1])..=high[1].div_euclid(size[1]) {
            for z in low[2].div_euclid(size[2])..=high[2].div_euclid(size[2]) {
                boxes.push((x, y, z));
            }
        }
    }
    boxes
}

/// Is the block inside the piece of tunnel from point `p` to the next? Whole numbers in 32 bits
/// throughout (see the top of the file).
fn inside_piece(tunnel: &Tunnel, p: usize, block: IVec3) -> bool {
    let (a, b) = (tunnel.points[p], tunnel.points[p + 1]);
    // The block's middle, in 16ths.
    let at = block.to_array().map(|v| v * SIXTEENTH + SIXTEENTH / 2);
    let line: [i32; 3] = std::array::from_fn(|k| b.at[k] - a.at[k]);
    let from: [i32; 3] = std::array::from_fn(|k| at[k] - a.at[k]);
    // Quickly out if it's far from the line's box (which also keeps the sums below small).
    let reach = a.width.max(a.height).max(b.width).max(b.height) + tunnel.rough.abs();
    if (0..3).any(|k| from[k] < line[k].min(0) - reach || from[k] > line[k].max(0) + reach) {
        return false;
    }
    // How far along the line the nearest place is, in 256ths (0 at `a`, 256 at `b`).
    let length = line[0] * line[0] + line[1] * line[1] + line[2] * line[2];
    let along = if length == 0 { 0 } else { ((from[0] * line[0] + from[1] * line[1] + from[2] * line[2]) * 256 / length).clamp(0, 256) };
    let off: [i32; 3] = std::array::from_fn(|k| from[k] - line[k] * along / 256);
    // Its size there, the walls pushed in or out by their lumps.
    let bump = tunnel.rough * whole_value3(tunnel.seed, block.to_array(), tunnel.lump) / 65_536;
    let width = (a.width + (b.width - a.width) * along / 256 + bump).clamp(0, MOST_ACROSS);
    let height = (a.height + (b.height - a.height) * along / 256 + bump).clamp(0, MOST_ACROSS);
    if width == 0 || height == 0 || off[2] < -(height * tunnel.floor / 256) {
        return false;
    }
    // Inside the oval: across² / width² + up² / height² ≤ 1, multiplied through (each part is checked
    // within its side first, so the products stay under 2³¹).
    let (across, up) = (off[0] * off[0] + off[1] * off[1], off[2] * off[2]);
    let (wide, tall) = (width * width, height * height);
    across <= wide && up <= tall && across * tall + up * wide <= wide * tall
}

/// Takes the caves out of a chunk already made: every solid block inside one becomes air. Water is
/// never carved. Returns how many blocks were taken. (Chunks being made carve as they go, in
/// `CompiledMap::chunk`, by the same `inside`.)
pub fn carve_chunk(chunk: &mut Chunk, plan: &CavePlan, water: BlockId) -> usize {
    let origin = chunk.pos().min_block();
    let mut taken = 0;
    for level in 0..SECTIONS_PER_CHUNK {
        if !plan.touches(chunk.pos(), level) {
            continue;
        }
        for z in level as i32 * SECTION_SIZE..(level as i32 + 1) * SECTION_SIZE {
            if !z_in_world(z) {
                continue;
            }
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = IVec3::new(origin.x + x, origin.y + y, z);
                    let id = chunk.get(block);
                    if !id.is_air() && id != water && plan.inside(block) {
                        chunk.set(block, BlockId::AIR);
                        taken += 1;
                    }
                }
            }
        }
    }
    taken
}

/// How the test cave is made (the World page; saved in "everything" presets).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TestCaveSettings {
    /// How far it goes in (blocks).
    pub length: f32,
    /// Half its width and its height (blocks).
    pub width: f32,
    pub height: f32,
    /// How much it worms: the most it turns, left or right, every 3 blocks (degrees).
    pub wander: f32,
    /// How much it goes up and down as it worms (degrees), and how steeply it heads down (degrees).
    pub rise_and_fall: f32,
    pub dip: f32,
    /// How narrow it gets toward its end, a share of its width (1 = no narrower).
    pub end_size: f32,
    /// The walls: how far they're pushed in or out (blocks), and how big their lumps are (blocks).
    pub rough: f32,
    pub lump: i32,
    /// How far down the floor is cut flat, a share of its height (1 = round, no flat floor).
    pub floor: f32,
}

impl Default for TestCaveSettings {
    fn default() -> Self {
        Self { length: 30.0, width: 2.5, height: 2.0, wander: 20.0, rise_and_fall: 6.0, dip: 4.0, end_size: 0.5, rough: 0.8, lump: 3, floor: 0.6 }
    }
}

/// Every 3 blocks, a point.
const STEP: f32 = 3.0;

impl TestCaveSettings {
    /// A dead-end tunnel worming in from `mouth` (blocks; its middle, just outside the face), heading
    /// `heading` (on the ground, north then east): a point every 3 blocks, turning a little each time
    /// (the turns from `seed`, so the same seed worms the same way), narrowing toward its end, which
    /// is round. Making the plan is world creation, so fractions are fine here.
    pub fn tunnel(&self, mouth: Vec3, heading: Vec2, seed: u32) -> Tunnel {
        let mut random = seed.max(1);
        let mut next = move || {
            // xorshift: −1 to 1.
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            random as f32 / u32::MAX as f32 * 2.0 - 1.0
        };
        let mut turn = heading.y.atan2(heading.x);
        let steps = (self.length / STEP).ceil().max(1.0) as usize;
        let mut at = mouth;
        let mut points = Vec::with_capacity(steps + 2);
        // One point before the mouth too, so the opening is cut clean through the face.
        let back = Vec3::new(turn.cos(), turn.sin(), 0.0) * STEP;
        for k in 0..=steps + 1 {
            let place = if k == 0 { at - back } else { at };
            // Full size for the first two thirds, then narrowing to the end.
            let along = (k as f32 / (steps + 1) as f32 - 0.66).max(0.0) / 0.34;
            let size = 1.0 + (self.end_size - 1.0) * along;
            points.push(CavePoint {
                at: place.to_array().map(|v| (v * SIXTEENTH as f32).round() as i32),
                width: ((self.width * size * SIXTEENTH as f32).round() as i32).clamp(1, MOST_ACROSS),
                height: ((self.height * size * SIXTEENTH as f32).round() as i32).clamp(1, MOST_ACROSS),
            });
            if k > 0 {
                turn += next() * self.wander.to_radians();
                let tilt = (-self.dip.to_radians() + next() * self.rise_and_fall.to_radians()).clamp(-0.6, 0.6);
                at += Vec3::new(turn.cos() * tilt.cos(), turn.sin() * tilt.cos(), tilt.sin()) * STEP;
            }
        }
        Tunnel {
            points,
            rough: ((self.rough * SIXTEENTH as f32).round() as i32).clamp(0, MOST_ACROSS),
            lump: self.lump.max(1),
            floor: (self.floor.clamp(0.0, 1.0) * 256.0).round() as i32,
            seed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::BlockRegistry;
    use crate::world::interpreter::CompiledMap;
    use crate::world::terrain::TerrainBlocks;

    /// A straight tunnel along north, at Z 200, from (`from`, `east`) to (`to`, `east`), 2 blocks
    /// half wide, 2 tall, smooth walls, no flat floor.
    fn straight(from: i32, to: i32, east: i32) -> Tunnel {
        let point = |north: i32| CavePoint { at: [north * SIXTEENTH, east * SIXTEENTH, 200 * SIXTEENTH], width: 2 * SIXTEENTH, height: 2 * SIXTEENTH };
        let points = (0..=(to - from) / 8).map(|k| point(from + k * 8)).collect();
        Tunnel { points, rough: 0, lump: 3, floor: 256, seed: 7 }
    }

    #[test]
    fn a_tunnel_is_air_inside_rock_outside_and_stops_at_its_end() {
        let plan = CavePlan::new(vec![straight(0, 32, 0)]);
        // On the line, and a block off it, inside; 3 blocks off it, outside.
        assert!(plan.inside(IVec3::new(10, 0, 200)));
        assert!(plan.inside(IVec3::new(10, 1, 200)) && plan.inside(IVec3::new(10, 0, 201)));
        assert!(!plan.inside(IVec3::new(10, 3, 200)) && !plan.inside(IVec3::new(10, 0, 203)));
        // A dead end: round, two blocks past its last point and no more.
        assert!(plan.inside(IVec3::new(33, 0, 200)));
        assert!(!plan.inside(IVec3::new(35, 0, 200)));
        // A flat floor halfway down: what was below it is rock.
        let mut flat = straight(0, 32, 0);
        flat.floor = 128;
        let plan = CavePlan::new(vec![flat]);
        assert!(plan.inside(IVec3::new(10, 0, 199)) && !plan.inside(IVec3::new(10, 0, 198)));
    }

    #[test]
    fn rough_walls_wander_but_the_same_seed_carves_the_same() {
        let mut rough = straight(0, 64, 0);
        rough.rough = 2 * SIXTEENTH;
        let plan = CavePlan::new(vec![rough.clone()]);
        // Two blocks off the line is wall in some places and air in others.
        let edge: Vec<bool> = (0..60).map(|north| plan.inside(IVec3::new(north, 2, 200))).collect();
        assert!(edge.contains(&true) && edge.contains(&false));
        assert_eq!(edge, (0..60).map(|north| CavePlan::new(vec![rough.clone()]).inside(IVec3::new(north, 2, 200))).collect::<Vec<_>>());
        // The noise, whole numbers: in range, the same each time, and smooth from block to block.
        for x in -40..40 {
            let value = whole_value3(9, [x, 3 * x, -x], 4);
            assert!((-65_536..65_536).contains(&value));
            assert_eq!(value, whole_value3(9, [x, 3 * x, -x], 4));
            assert!((value - whole_value3(9, [x + 1, 3 * x, -x], 4)).abs() < 65_536);
        }
    }

    #[test]
    fn a_cave_crossing_every_side_of_a_chunk_is_carved_the_same_whichever_chunk_makes_it() {
        // High ground everywhere (Z 300), and two tunnels through the corner where four chunks meet:
        // one along north at east 0, one along east at north 0 (rough walls, a flat floor).
        let width = 8;
        let map = CompiledMap::of_heights(width, 32.0, &vec![300.0; width * width]);
        let blocks = TerrainBlocks::from_registry(&BlockRegistry::default());
        let mut north = straight(-40, 40, 0);
        let mut east = Tunnel { points: north.points.iter().map(|p| CavePoint { at: [0, p.at[0], p.at[2] + SIXTEENTH], ..*p }).collect(), ..north.clone() };
        north.rough = 24;
        north.floor = 160;
        east.rough = 16;
        east.seed = 8;
        let plan = CavePlan::new(vec![north, east]);
        let none = CavePlan::default();
        for (x, y) in [(-1, -1), (-1, 0), (0, -1), (0, 0), (1, 0), (-2, 0)] {
            let pos = ChunkPos::new(x, y);
            let made = map.chunk(pos, &blocks, &plan).unwrap();
            // Made with the caves, or made without and carved after: the same blocks.
            let mut carved = map.chunk(pos, &blocks, &none).unwrap();
            carve_chunk(&mut carved, &plan, blocks.water);
            let origin = pos.min_block();
            for z in 190..215 {
                for local_y in 0..CHUNK_SIZE {
                    for local_x in 0..CHUNK_SIZE {
                        let block = origin + IVec3::new(local_x, local_y, z);
                        assert_eq!(made.get(block).is_air(), plan.inside(block), "at {block}");
                        assert_eq!(made.get(block), carved.get(block), "at {block}");
                    }
                }
            }
        }
        // Both tunnels were really carved, on both sides of each chunk side.
        for block in [IVec3::new(-1, 0, 200), IVec3::new(0, 0, 200), IVec3::new(0, -1, 201), IVec3::new(0, 1, 201), IVec3::new(-33, 0, 200)] {
            assert!(plan.inside(block), "{block}");
        }
        // Deep under the ground, a section with a cave isn't one uniform block any more; one far from
        // it still is.
        let chunk = map.chunk(ChunkPos::new(0, 0), &blocks, &plan).unwrap();
        assert!(!matches!(chunk.sections()[200 / SECTION_SIZE as usize], crate::world::Section::Uniform(_)));
        assert!(matches!(chunk.sections()[100 / SECTION_SIZE as usize], crate::world::Section::Uniform(_)));
    }

    #[test]
    fn the_plan_survives_being_saved_and_the_biggest_tunnel_far_out_fits_32_bits() {
        let settings = TestCaveSettings { width: 20.0, height: 20.0, rough: 20.0, ..TestCaveSettings::default() };
        let far = Vec3::new(-30_000.0, 30_000.0, 900.0);
        let tunnel = settings.tunnel(far, Vec2::new(0.6, 0.8), 42);
        assert!(tunnel.points.iter().all(|p| p.width == MOST_ACROSS && p.height <= MOST_ACROSS));
        assert!(tunnel.points.windows(2).all(|pair| (0..3).all(|k| (pair[1].at[k] - pair[0].at[k]).abs() <= LONGEST_PIECE)));
        let plan = CavePlan::new(vec![tunnel.clone()]);
        let back = CavePlan::from_text(&plan.to_text()).unwrap();
        assert_eq!(back, plan);
        // Every block round it asked (any overflow would stop the test): the same answers from the
        // loaded plan.
        let middle = far.as_ivec3();
        let mut carved = 0;
        for x in -40..40 {
            for y in -40..40 {
                for z in -20..20 {
                    let block = middle + IVec3::new(x, y, z);
                    let inside = plan.inside(block);
                    assert_eq!(inside, back.inside(block));
                    carved += inside as usize;
                }
            }
        }
        assert!(carved > 1_000, "{carved}");
        // The same seed worms the same way; another seed another way.
        assert_eq!(settings.tunnel(far, Vec2::X, 5), settings.tunnel(far, Vec2::X, 5));
        assert_ne!(settings.tunnel(far, Vec2::X, 5), settings.tunnel(far, Vec2::X, 6));
    }
}

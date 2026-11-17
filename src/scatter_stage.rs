//! The scatter stage (plan 2.2; the Oracle's third part, section 7): test detail the doorway applies as it answers a
//! tile, made from a seed and a short list of places, so it costs nothing to keep. An instrument with a switch
//! (`INCUS_SCATTER_STAGE=1`), off by default; not world generation, and nothing here is carried into it.
//!
//! What it's for: the far cells finer than 32 blocks, the far light and the near eye need things to be judged on
//! that the bare test world doesn't have. David's list (2026-11-15): "large arched entranches to mountain halls,
//! castles with walls and towers, towers in the mountain and on the ground, towns on the ground, forests here and
//! there, random cave mouths here and there. its gonna be blurry and just for testing anyways, so go nuts". The
//! Oracle's: posts one block wide in rows, slabs, an arch. The places, blocks and colours are Builder's picks.
//!
//! How: a column the stage changes is answered as its solid blocks, bedrock up, with air between them (`Staged`): the
//! export's own blocks with the stage's added (a wall, a crown of leaves) or cut away (a cave). The far cells are
//! summed from those runs (`far.rs`); the doorway's own answer for a column gives them with its water.

use crate::doorway::{Block, Run};

/// Whether the stage is on (`INCUS_SCATTER_STAGE=1`).
pub fn on() -> bool {
    std::env::var("INCUS_SCATTER_STAGE").as_deref() == Ok("1")
}

/// The stage's own blocks, in the order the doorway learns them (so every doorway on the world numbers them alike).
pub const BLOCKS: [&str; 9] = ["incus:oak", "incus:leaves", "incus:dressed_stone", "incus:roof_tiles", "incus:plaster", "incus:planks", "incus:thatch", "incus:red_lamp", "incus:green_lamp"];
const OAK: usize = 0;
const LEAVES: usize = 1;
const DRESSED: usize = 2;
const ROOF: usize = 3;
const PLASTER: usize = 4;
const PLANKS: usize = 5;
const THATCH: usize = 6;

/// Woods grow within this many blocks of the spawn (the summing of a wood's columns is slower than plain ground's).
const WOODS_REACH: i32 = 10_000;
/// A tree to a square of this many blocks, where a wood is.
const TREE_GRID: i32 = 7;

/// One of the stage's places: what stands there, where (north, east), and the ground's top there (its floor).
#[derive(Clone, Debug, PartialEq)]
pub struct Site {
    pub kind: Kind,
    pub at: [i32; 2],
    pub floor: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// A round tower: its radius and height; a roof or battlements.
    Tower { radius: i32, high: i32, roofed: bool },
    /// A castle: a square of walls (half its side), corner towers, a keep, and a gate through the wall on the side
    /// its `gate` way looks (0 north, 1 south, 2 east, 3 west).
    Castle { half: i32, gate: u8 },
    /// A building of a town, from its least corner: so wide (north), so deep (east), its walls so high under a
    /// pitched roof. Its make: 0 plaster under roof tiles, 1 planks under thatch, 2 dressed stone under roof tiles (a
    /// church's nave), 3 a church's tower (dressed stone, a spire). Each stands on its own ground: a town is a
    /// street of these (`Stage::town`).
    House { wide: i32, deep: i32, high: i32, make: u8 },
    /// An arched entrance to a hall cut into a hillside: the way in (0 north, 1 south, 2 east, 3 west).
    Hall { way: u8 },
    /// A cave mouth: the way in, its radius and how deep it goes.
    Cave { way: u8, radius: i32, deep: i32 },
    /// Rows of posts one block wide, running the way given.
    Posts { way: u8, rows: i32, long: i32 },
    /// A slab in the air: half its side, and how far over the ground.
    Slab { half: i32, over: i32 },
    /// A free-standing arch: half its span (it spans east and west).
    Arch { half: i32 },
}

impl Site {
    /// How far its blocks reach from `at`, north and east alike.
    fn reach(&self) -> i32 {
        match self.kind {
            Kind::Tower { radius, .. } => radius + 3,
            Kind::Castle { half, .. } => half + 14,
            Kind::House { wide, deep, .. } => wide.max(deep) + 2,
            Kind::Hall { .. } => 130,
            Kind::Cave { radius, deep, .. } => deep + radius + 2,
            Kind::Posts { rows, long, .. } => long.max(rows * 12) + 2,
            Kind::Slab { half, .. } => half + 1,
            Kind::Arch { half } => half + 6,
        }
    }
}

/// A step along a way (0 north, 1 south, 2 east, 3 west), and the step to its right.
fn along(way: u8) -> ([i32; 2], [i32; 2]) {
    match way {
        0 => ([1, 0], [0, 1]),
        1 => ([-1, 0], [0, -1]),
        2 => ([0, 1], [-1, 0]),
        _ => ([0, -1], [1, 0]),
    }
}

/// A number from a place and a salt, steady (a hash).
fn hash(a: i32, b: i32, salt: u64) -> u64 {
    let mut h = (a as u32 as u64) << 32 | (b as u32 as u64);
    h ^= salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h = (h ^ (h >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h = (h ^ (h >> 33)).wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^ (h >> 33)
}

/// The same, as a number from 0 up to 1.
fn chance(a: i32, b: i32, salt: u64) -> f32 {
    (hash(a, b, salt) >> 40) as f32 / (1u64 << 24) as f32
}

/// The columns the stage changed in a tile: each one's solid blocks from bedrock up, air between the runs (the
/// export's water isn't in them: it fills what's open over the export's ground, as before).
#[derive(Default)]
pub struct Staged {
    /// For each column of the tile (north fastest), where its runs end in `runs`, one up from the column before's;
    /// a column the stage left alone has none.
    ends: Vec<u32>,
    runs: Vec<Run>,
}

impl Staged {
    /// The runs of the column at `k` of its tile, if the stage changed it.
    pub fn column(&self, k: usize) -> Option<&[Run]> {
        let (from, to) = (if k == 0 { 0 } else { self.ends[k - 1] } as usize, self.ends[k] as usize);
        (to > from).then(|| &self.runs[from..to])
    }

    /// Every cell of a block `wanted` in the columns the stage changed: its column's place in the tile, and its
    /// height (build 216, for finding the lamps).
    pub fn cells_of(&self, wanted: impl Fn(Block) -> bool) -> Vec<(usize, i32)> {
        let mut cells = Vec::new();
        let mut from = 0;
        for (k, end) in self.ends.iter().enumerate() {
            for run in &self.runs[from..*end as usize] {
                if wanted(run.block) {
                    cells.extend((run.from..run.to).map(|up| (k, up)));
                }
            }
            from = *end as usize;
        }
        cells
    }
}

/// A column being changed: its solid runs, bedrock up.
struct Edit {
    runs: Vec<Run>,
}

impl Edit {
    /// The heights `from` up to `to` (not including it) made air.
    fn carve(&mut self, from: i32, to: i32) {
        if to <= from {
            return;
        }
        let mut kept = Vec::with_capacity(self.runs.len() + 1);
        for run in &self.runs {
            if run.to <= from || run.from >= to {
                kept.push(*run);
                continue;
            }
            if run.from < from {
                kept.push(Run { block: run.block, from: run.from, to: from });
            }
            if run.to > to {
                kept.push(Run { block: run.block, from: to, to: run.to });
            }
        }
        self.runs = kept;
    }

    /// The heights `from` up to `to` made `block`, whatever was there.
    fn fill(&mut self, block: Block, from: i32, to: i32) {
        let from = from.max(0);
        if to <= from {
            return;
        }
        self.carve(from, to);
        let at = self.runs.iter().position(|run| run.from >= to).unwrap_or(self.runs.len());
        self.runs.insert(at, Run { block, from, to });
    }

    /// The top of its highest block.
    fn top(&self) -> i32 {
        self.runs.last().map_or(0, |run| run.to)
    }

    /// Runs of the same block that touch made one.
    fn tidy(&mut self) {
        let mut tidy: Vec<Run> = Vec::with_capacity(self.runs.len());
        for run in &self.runs {
            match tidy.last_mut() {
                Some(last) if last.block == run.block && last.to == run.from => last.to = run.to,
                _ => tidy.push(*run),
            }
        }
        self.runs = tidy;
    }
}

/// The stage, made once for a world: its blocks as the doorway numbers them, and its places.
pub struct Stage {
    blocks: [Block; 9],
    sites: Vec<Site>,
    /// Its towns' middles, and a place where a wood stands on even ground, for its marks.
    towns: Vec<[i32; 2]>,
    wood: Option<[i32; 2]>,
    sea_level: i32,
    height: i32,
}

impl Stage {
    /// The stage for a world: its places chosen, each one's floor read from the world's own ground (`ground_at`: a
    /// column's ground as the export has it, none past the map or under water).
    pub fn new(blocks: [Block; 9], sea_level: i32, height: i32, mut ground_at: impl FnMut(i32, i32) -> Option<i32>) -> Self {
        let mut sites: Vec<Site> = Vec::new();
        // A place put at the first of `spots` that is dry land with room over it for `room` blocks.
        let place = |kind: Kind, spots: &[[i32; 2]], room: i32, sites: &mut Vec<Site>, ground_at: &mut dyn FnMut(i32, i32) -> Option<i32>| {
            for &at in spots {
                if let Some(ground) = ground_at(at[0], at[1]).filter(|ground| ground + 1 + room < height - 8) {
                    sites.push(Site { kind, at, floor: ground + 1 });
                    return;
                }
            }
        };
        // On the plain, ahead of the plain's mark (it stands at north -3975, east -1192, looking south): what the
        // Oracle asked for near, then a tower, a castle, a town and a great tower, each farther off.
        let plain = [-3975, -1192];
        let ahead = |south: i32, east: i32| [plain[0] - south, plain[1] + east];
        place(Kind::Posts { way: 1, rows: 3, long: 160 }, &[ahead(40, -14)], 4, &mut sites, &mut ground_at);
        place(Kind::Arch { half: 12 }, &[ahead(140, 34)], 40, &mut sites, &mut ground_at);
        place(Kind::Slab { half: 12, over: 36 }, &[ahead(220, -60)], 40, &mut sites, &mut ground_at);
        place(Kind::Tower { radius: 7, high: 70, roofed: false }, &[ahead(380, 90)], 80, &mut sites, &mut ground_at);
        place(Kind::Castle { half: 55, gate: 0 }, &[ahead(750, -40)], 60, &mut sites, &mut ground_at);
        let mut towns: Vec<[i32; 2]> = Vec::new();
        towns.extend(Self::town(&[ahead(1100, -500), ahead(1100, 400), ahead(900, -700)], 7, &mut sites, &mut ground_at).map(|(at, _)| at));
        place(Kind::Tower { radius: 10, high: 140, roofed: true }, &[ahead(1500, 300), ahead(1400, 100), ahead(1300, -200), ahead(1200, 500), ahead(1000, 300)], 160, &mut sites, &mut ground_at);
        // By the spawn (it looks north): a town on the flattest ground within a few hundred blocks ahead of it, and a
        // tower on ground near the town's own height.
        let mut flattest: Vec<(i32, [i32; 2])> = Vec::new();
        for north in (1..=9).map(|k| k * 96) {
            for east in (-5..=5).map(|k| k * 96) {
                let grounds: Vec<Option<i32>> = (0..9).map(|k| ground_at(north + (k % 3 - 1) * 90, east + (k / 3 - 1) * 90)).collect();
                if let Some(grounds) = grounds.into_iter().collect::<Option<Vec<i32>>>() {
                    let rough = grounds.iter().max().unwrap() - grounds.iter().min().unwrap();
                    flattest.push((rough, [north, east]));
                }
            }
        }
        flattest.sort();
        let spots: Vec<[i32; 2]> = flattest.iter().map(|(_, at)| *at).collect();
        let by_spawn = Self::town(&spots, 10, &mut sites, &mut ground_at);
        towns.extend(by_spawn.map(|(at, _)| at));
        let town_floor = by_spawn.map_or(0, |(_, level)| level);
        let near_town: Vec<[i32; 2]> = [[800, -250], [700, 300], [600, 0], [500, 400], [200, 500], [-300, 300], [100, -400]].into_iter().filter(|at| ground_at(at[0], at[1]).is_some_and(|ground| (ground + 1 - town_floor).abs() < 80)).collect();
        place(Kind::Tower { radius: 8, high: 90, roofed: true }, &near_town, 110, &mut sites, &mut ground_at);
        // In the mountains, south of the mountain top's mark (north -605, east 78, looking south): towers on the
        // highest ground in view, well apart, and a hall's entrance where the ground rises steepest toward the south.
        let mountain = [-605, 78];
        let mut peaks: Vec<(i32, [i32; 2])> = Vec::new();
        let mut steepest: Option<(i32, [i32; 2])> = None;
        for south in (5..40).map(|k| k * 64) {
            for east in (-12..=12).map(|k| k * 64) {
                let at = [mountain[0] - south, mountain[1] + east];
                let Some(ground) = ground_at(at[0], at[1]) else { continue };
                // A peak: nothing higher 128 blocks round it.
                let round = [[128, 0], [-128, 0], [0, 128], [0, -128], [96, 96], [-96, 96], [96, -96], [-96, -96]];
                if round.iter().all(|d| ground_at(at[0] + d[0], at[1] + d[1]).is_some_and(|other| other < ground)) {
                    peaks.push((ground, at));
                }
                // A slope facing the mark: the ground 96 blocks farther south stands well over this.
                if let Some(beyond) = ground_at(at[0] - 96, at[1]) {
                    let rise = beyond - ground;
                    if rise >= 50 && steepest.is_none_or(|(most, _)| rise > most) {
                        steepest = Some((rise, at));
                    }
                }
            }
        }
        peaks.sort_by_key(|(ground, at)| (std::cmp::Reverse(*ground), *at));
        let mut towers: Vec<[i32; 2]> = Vec::new();
        for (_, at) in &peaks {
            if towers.len() < 3 && towers.iter().all(|other| (other[0] - at[0]).abs().max((other[1] - at[1]).abs()) >= 400) {
                let k = towers.len() as i32;
                place(Kind::Tower { radius: 6 + k, high: 60 + 20 * k, roofed: k != 1 }, &[*at], 70 + 20 * k, &mut sites, &mut ground_at);
                towers.push(*at);
            }
        }
        if let Some((_, at)) = steepest {
            place(Kind::Hall { way: 1 }, &[at], 60, &mut sites, &mut ground_at);
        }
        // Cave mouths here and there, within four kilometres of the spawn: where the ground rises steeply one way,
        // about one such place in twelve.
        for north in (-31..=31).map(|k| k * 128) {
            for east in (-31..=31).map(|k| k * 128) {
                let Some(ground) = ground_at(north, east) else { continue };
                if chance(north, east, 11) > 0.085 {
                    continue;
                }
                for way in 0..4u8 {
                    let (step, _) = along(way);
                    let rise = ground_at(north + step[0] * 40, east + step[1] * 40).map_or(0, |beyond| beyond - ground);
                    if rise >= 32 {
                        let radius = 5 + (hash(north, east, 12) % 5) as i32;
                        place(Kind::Cave { way, radius, deep: 40 + (hash(north, east, 13) % 24) as i32 }, &[[north, east]], 0, &mut sites, &mut ground_at);
                        break;
                    }
                }
            }
        }
        // A wood on even ground near the spawn, for a mark: the first wooded place, going out from the spawn, whose
        // ground 48 blocks each way is within 12 of its own, clear of the places.
        let mut wood = None;
        'wood: for ring in 2..24 {
            for k in 0..8 {
                let at = [[1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0], [-1, -1], [0, -1], [1, -1]][k].map(|step: i32| step * ring * 64);
                let Some(ground) = ground_at(at[0], at[1]) else { continue };
                let even = [[48, 0], [-48, 0], [0, 48], [0, -48]].iter().all(|d| ground_at(at[0] + d[0], at[1] + d[1]).is_some_and(|other| (other - ground).abs() <= 12));
                let clear = sites.iter().all(|site| (site.at[0] - at[0]).abs().max((site.at[1] - at[1]).abs()) > site.reach() + 150);
                if even && clear && ground < 600 && Self::wooded(at[0], at[1]) && Self::wooded(at[0] + 60, at[1]) && Self::wooded(at[0], at[1] + 60) {
                    wood = Some(at);
                    break 'wood;
                }
            }
        }
        Self { blocks, sites, towns, wood, sea_level, height }
    }

    /// A town laid out at the first of `spots` that is dry land: two streets crossing at its middle, houses on both
    /// sides of each (so many to a street's side, some left out), and a church by the crossing. Each building stands
    /// on its own ground, and none where the ground is far off the town's own level. The floor at its middle, if
    /// it was laid out.
    fn town(spots: &[[i32; 2]], houses: i32, sites: &mut Vec<Site>, ground_at: &mut dyn FnMut(i32, i32) -> Option<i32>) -> Option<([i32; 2], i32)> {
        let (at, level) = spots.iter().find_map(|at| ground_at(at[0], at[1]).map(|ground| (*at, ground + 1)))?;
        let mut building = |corner: [i32; 2], wide: i32, deep: i32, high: i32, make: u8, sites: &mut Vec<Site>| {
            if let Some(ground) = ground_at(corner[0] + wide / 2, corner[1] + deep / 2).filter(|ground| (ground + 1 - level).abs() <= 24) {
                sites.push(Site { kind: Kind::House { wide, deep, high, make }, at: corner, floor: ground + 1 });
            }
        };
        for k in -houses..houses {
            for (street, side) in [(0, 1), (0, -1), (1, 1), (1, -1)] {
                let salt = 20 + street as u64 * 2 + (side + 1) as u64;
                if k == 0 || chance(at[0] + k, at[1], salt) > 0.85 {
                    continue;
                }
                let (wide, deep) = (7 + (hash(k, street, salt) % 5) as i32, 6 + (hash(k, street, salt + 9) % 4) as i32);
                let high = 5 + (hash(k, street, salt + 18) % 5) as i32;
                let make = (hash(k, street, salt + 27) % 3 == 0) as u8;
                let back = if side > 0 { 6 } else { -6 - deep };
                if street == 0 {
                    building([at[0] + k * 14, at[1] + back], wide, deep, high, make, sites);
                } else {
                    building([at[0] + back, at[1] + k * 14], deep, wide, high, make, sites);
                }
            }
        }
        building([at[0] + 20, at[1] + 30], 26, 12, 14, 2, sites);
        building([at[0] + 46, at[1] + 32], 8, 8, 40, 3, sites);
        Some((at, level))
    }

    /// Its places.
    pub fn sites(&self) -> &[Site] {
        &self.sites
    }

    /// Marks for a look at its places and for flights with it on, one for each kind of thing: each stands a little
    /// way off, over the ground, looking at what's there. (Made here, not kept in `data/marks.toml`: they follow
    /// the places if those move, and exist only while the stage is on.)
    pub fn marks(&self) -> Vec<crate::marks::Mark> {
        let mark = |name: &str, at: [i32; 2], above: f32, facing: f32, pitch: f32| crate::marks::Mark {
            name: format!("the stage: {name}"),
            at: [at[0] as f32 + 0.5, at[1] as f32 + 0.5],
            above,
            facing,
            pitch,
            fog: false,
            eyes: vec!["far".to_string()],
            changes: None,
            scatter: None,
            speed: None,
            nod: None,
        };
        // The way a mark faces to look along a way (0 north, 1 south, 2 east, 3 west).
        let facing = |way: u8| [0.0, 180.0, 90.0, 270.0][way as usize % 4];
        let mut marks = Vec::new();
        if let Some(site) = self.sites.iter().find(|site| matches!(site.kind, Kind::Castle { .. })) {
            marks.push(mark("the castle", [site.at[0] + 230, site.at[1]], 40.0, 180.0, -7.0));
        }
        for (k, town) in self.towns.iter().enumerate() {
            // The town on the plain from its north, the one by the spawn from its south.
            if k == 0 {
                marks.push(mark("the town on the plain", [town[0] + 200, town[1] + 60], 35.0, 180.0, -6.0));
            } else {
                marks.push(mark("the town by the spawn", [town[0] - 200, town[1]], 45.0, 0.0, -8.0));
            }
        }
        if let Some(site) = self.sites.iter().find(|site| matches!(site.kind, Kind::Hall { .. })) {
            marks.push(mark("the hall in the mountain", [site.at[0] + 280, site.at[1]], 30.0, 180.0, 6.0));
        }
        if let Some(wood) = self.wood {
            marks.push(mark("over a wood", wood, 40.0, 0.0, -15.0));
        }
        if let Some(site) = self.sites.iter().find(|site| matches!(site.kind, Kind::Cave { radius, .. } if radius >= 8)) {
            if let Kind::Cave { way, .. } = site.kind {
                let (step, _) = along(way);
                marks.push(mark("a cave mouth", [site.at[0] - step[0] * 90, site.at[1] - step[1] * 90], 12.0, facing(way), -2.0));
            }
        }
        marks
    }

    /// Whether a wood grows over the column: in patches a few hundred blocks across, within reach of the spawn.
    fn wooded(north: i32, east: i32) -> bool {
        if north.abs() > WOODS_REACH || east.abs() > WOODS_REACH {
            return false;
        }
        // Smooth noise from a lattice of 320 blocks, with a finer one over it.
        let noise = |size: i32, salt: u64| {
            let (i, j) = (north.div_euclid(size), east.div_euclid(size));
            let (u, v) = (north.rem_euclid(size) as f32 / size as f32, east.rem_euclid(size) as f32 / size as f32);
            let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
            let corner = |a: i32, b: i32| chance(i + a, j + b, salt);
            (corner(0, 0) * (1.0 - u) + corner(1, 0) * u) * (1.0 - v) + (corner(0, 1) * (1.0 - u) + corner(1, 1) * u) * v
        };
        noise(320, 1) * 0.75 + noise(80, 2) * 0.25 > 0.56
    }

    /// The tree of a square of the tree grid, if one grows there: its trunk's column, its height and its crown's
    /// radius.
    fn tree(i: i32, j: i32) -> Option<([i32; 2], i32, i32)> {
        if chance(i, j, 3) > 0.7 {
            return None;
        }
        let at = [i * TREE_GRID + (hash(i, j, 4) % TREE_GRID as u64) as i32, j * TREE_GRID + (hash(i, j, 5) % TREE_GRID as u64) as i32];
        if !Self::wooded(at[0], at[1]) {
            return None;
        }
        let high = 4 + (hash(i, j, 6) % 17) as i32;
        Some((at, high, 2 + (high / 7).min(2) + (hash(i, j, 7) % 2) as i32))
    }

    /// The stage applied to a tile: `least` its least column (north, east), `side` its columns a side, `grounds` its
    /// columns' grounds as the export has them (north fastest; -1 past the map), `waters` their rivers' or lakes'
    /// tops (0 none), and `solid` a column's own solid runs when asked (by its place in the tile). None if the stage
    /// changes nothing in it.
    pub fn apply(&self, least: [i32; 2], side: usize, grounds: &[i32], waters: &[i32], mut solid: impl FnMut(usize) -> Vec<Run>) -> Option<Staged> {
        let size = side as i32;
        let mut edits: std::collections::BTreeMap<usize, Edit> = std::collections::BTreeMap::new();
        // A column of the tile being changed, if it's on dry land of the map.
        let mut edit = |north: i32, east: i32, edits: &mut std::collections::BTreeMap<usize, Edit>, change: &mut dyn FnMut(&mut Edit, i32)| {
            let (i, j) = (north - least[0], east - least[1]);
            if i < 0 || j < 0 || i >= size || j >= size {
                return;
            }
            let k = i as usize + j as usize * side;
            let ground = grounds[k];
            if ground < 0 || ground + 1 < self.sea_level || waters[k] > 0 {
                return;
            }
            let column = edits.entry(k).or_insert_with(|| Edit { runs: solid(k) });
            change(column, ground);
        };
        let [oak, leaves, dressed, roof, plaster, planks, thatch, red, green] = self.blocks;
        // The lamps (build 216) [David, 2026-11-17: "just use red and green blocks for the glowing blocks"]: two
        // plain blocks that give light (`data/lights.toml`), stood where light can be seen before there are nights:
        // along each cave's floor, at the feet of the hall's pillars, and in the woods' shade by the spawn.
        let lamps = crate::near::lamps_on();
        let _ = (OAK, LEAVES, DRESSED, ROOF, PLASTER, PLANKS, THATCH);
        let sites: Vec<&Site> = self.sites.iter().filter(|site| {
            let reach = site.reach();
            site.at[0] + reach >= least[0] && site.at[0] - reach < least[0] + size && site.at[1] + reach >= least[1] && site.at[1] - reach < least[1] + size
        }).collect();
        // The woods: every column under a tree's crown, or its trunk's own. Not where a place stands.
        let near_woods = least[0] <= WOODS_REACH && least[0] + size >= -WOODS_REACH && least[1] <= WOODS_REACH && least[1] + size >= -WOODS_REACH;
        if near_woods {
            for j in 0..size {
                for i in 0..size {
                    let (north, east) = (least[0] + i, least[1] + j);
                    let (gi, gj) = (north.div_euclid(TREE_GRID), east.div_euclid(TREE_GRID));
                    for b in -1..=1 {
                        for a in -1..=1 {
                            let Some((at, high, radius)) = Self::tree(gi + a, gj + b) else { continue };
                            let (dn, de) = (north - at[0], east - at[1]);
                            let apart = dn * dn + de * de;
                            if apart > radius * radius {
                                continue;
                            }
                            if sites.iter().any(|site| (at[0] - site.at[0]).abs() <= site.reach() + 4 && (at[1] - site.at[1]).abs() <= site.reach() + 4) {
                                continue;
                            }
                            edit(north, east, &mut edits, &mut |column, ground| {
                                if ground > 700 {
                                    return;
                                }
                                // The crown: a ball round the trunk's top, a little flattened; the trunk under it.
                                let reach = (((radius * radius - apart) as f32).sqrt() * 0.9).round() as i32;
                                let top = ground + 1 + high;
                                if apart == 0 {
                                    column.fill(oak, ground + 1, top);
                                    column.fill(leaves, top, top + reach + 1);
                                } else {
                                    column.fill(leaves, top - reach / 2 - 1, top + reach);
                                }
                            });
                        }
                    }
                }
            }
        }
        // The lamps in the woods within 200 blocks of the spawn: here and there on the ground, red or green.
        if lamps {
            const BY_THE_SPAWN: i32 = 200;
            for east in least[1].max(-BY_THE_SPAWN)..(least[1] + size).min(BY_THE_SPAWN) {
                for north in least[0].max(-BY_THE_SPAWN)..(least[0] + size).min(BY_THE_SPAWN) {
                    if north.rem_euclid(12) != 5 || east.rem_euclid(12) != 5 || chance(north, east, 41) > 0.4 || !Self::wooded(north, east) {
                        continue;
                    }
                    if sites.iter().any(|site| (north - site.at[0]).abs() <= site.reach() + 4 && (east - site.at[1]).abs() <= site.reach() + 4) {
                        continue;
                    }
                    edit(north, east, &mut edits, &mut |column, ground| {
                        if ground <= 700 {
                            column.fill(if hash(north, east, 42) % 2 == 0 { red } else { green }, ground + 1, ground + 2);
                        }
                    });
                }
            }
        }
        // The places.
        for site in sites {
            let (at, floor) = (site.at, site.floor);
            let reach = site.reach();
            // A round tower at `centre`: its wall from the ground up, a ring standing out under its top, and
            // battlements or a roof.
            let tower = |centre: [i32; 2], radius: i32, high: i32, roofed: bool, edits: &mut std::collections::BTreeMap<usize, Edit>, edit: &mut dyn FnMut(i32, i32, &mut std::collections::BTreeMap<usize, Edit>, &mut dyn FnMut(&mut Edit, i32))| {
                for de in -radius - 2..=radius + 2 {
                    for dn in -radius - 2..=radius + 2 {
                        let apart = ((dn * dn + de * de) as f32).sqrt();
                        if apart > radius as f32 + 2.2 {
                            continue;
                        }
                        edit(centre[0] + dn, centre[1] + de, edits, &mut |column, ground| {
                            let top = floor + high;
                            if apart <= radius as f32 + 0.2 {
                                column.fill(dressed, ground + 1, top);
                            } else {
                                // The ring that stands out: air under it.
                                column.fill(dressed, top - 6, top);
                            }
                            if roofed {
                                let rise = ((radius as f32 + 2.2 - apart) * 1.6) as i32;
                                column.fill(roof, top, top + rise);
                            } else if apart > radius as f32 + 0.2 && (dn + de).rem_euclid(2) == 0 {
                                column.fill(dressed, top, top + 2);
                            }
                        });
                    }
                }
            };
            match site.kind {
                Kind::Tower { radius, high, roofed } => tower(at, radius, high, roofed, &mut edits, &mut edit),
                Kind::Castle { half, gate } => {
                    let (into, right) = along(gate);
                    let (wall_high, thick) = (16, 5);
                    for de in -half..=half {
                        for dn in -half..=half {
                            let out = dn.abs().max(de.abs());
                            if out <= half - thick {
                                continue;
                            }
                            // Where along the gate's wall this column is: how far out toward the gate, and how far
                            // to its side.
                            let (toward, aside) = (dn * into[0] + de * into[1], dn * right[0] + de * right[1]);
                            edit(at[0] + dn, at[1] + de, &mut edits, &mut |column, ground| {
                                let gatehouse = toward > half - thick && aside.abs() <= 10;
                                let top = floor + if gatehouse { 26 } else { wall_high };
                                column.fill(dressed, ground + 1, top);
                                if out == half && (dn + de).rem_euclid(2) == 0 {
                                    column.fill(dressed, top, top + 2);
                                }
                                // The gate: an arch through the wall, 10 wide, 9 to its spring and round over that.
                                if toward > half - thick && aside.abs() <= 5 {
                                    let arch = 9 + ((25 - aside * aside) as f32).sqrt() as i32;
                                    column.carve(floor, floor + arch);
                                }
                            });
                        }
                    }
                    for corner in [[half, half], [half, -half], [-half, half], [-half, -half]] {
                        tower([at[0] + corner[0], at[1] + corner[1]], 8, 28, true, &mut edits, &mut edit);
                    }
                    // The keep: a block 26 square and 44 high, a pitched roof over it, standing back from the gate.
                    let keep = [at[0] - into[0] * 12, at[1] - into[1] * 12];
                    for de in -14..=14 {
                        for dn in -14..=14 {
                            edit(keep[0] + dn, keep[1] + de, &mut edits, &mut |column, ground| {
                                if dn.abs() <= 13 && de.abs() <= 13 {
                                    column.fill(dressed, ground + 1, floor + 44);
                                }
                                // The roof's eaves stand a block out past the walls: air under them.
                                column.fill(roof, floor + 44, floor + 44 + (15 - de.abs()));
                            });
                        }
                    }
                }
                Kind::House { wide, deep, high, make } => {
                    let (walls, top) = match make {
                        0 => (plaster, roof),
                        1 => (planks, thatch),
                        _ => (dressed, roof),
                    };
                    for de in -1..=deep {
                        for dn in -1..=wide {
                            edit(at[0] + dn, at[1] + de, &mut edits, &mut |column, ground| {
                                // Not where the ground falls far away under it, or stands over it.
                                if floor - (ground + 1) > 6 || ground + 1 > floor + high {
                                    return;
                                }
                                let inside = dn >= 0 && dn < wide && de >= 0 && de < deep;
                                if inside {
                                    column.fill(walls, ground + 1, floor + high);
                                }
                                // A pitched roof along its longer side, its eaves a block out (air under them); a
                                // church's tower has a spire.
                                let rise = if make == 3 {
                                    if inside { (dn + 1).min(wide - dn).min((de + 1).min(deep - de)) * 4 } else { 0 }
                                } else if wide >= deep {
                                    (de + 1).min(deep - de).max(1)
                                } else {
                                    (dn + 1).min(wide - dn).max(1)
                                };
                                column.fill(top, floor + high, floor + high + rise);
                            });
                        }
                    }
                }
                Kind::Hall { way } => {
                    // A portal of dressed stone on the hillside, an arch through it 20 wide and 30 high, a passage
                    // behind it, and the hall itself: 40 wide and 50 high, two rows of pillars left standing.
                    let (into, right) = along(way);
                    for t in -3..=reach - 10 {
                        for x in -22..=22 {
                            let (north, east) = (at[0] + into[0] * t + right[0] * x, at[1] + into[1] * t + right[1] * x);
                            edit(north, east, &mut edits, &mut |column, ground| {
                                if (-3..=3).contains(&t) && x.abs() <= 15 {
                                    column.fill(dressed, (ground + 1).min(floor), floor + 38);
                                }
                                if t < 44 {
                                    if x.abs() <= 10 {
                                        column.carve(floor, floor + 20 + ((100 - x * x) as f32).sqrt() as i32);
                                    }
                                } else if x.abs() <= 20 {
                                    let pillar = (x.abs() - 12).abs() <= 1 && (t - 44).rem_euclid(20) <= 2;
                                    if !pillar {
                                        column.carve(floor, floor + 50 - (x.abs() / 4));
                                    }
                                }
                                // A lamp on the floor inside each pillar, red and green by turns.
                                if lamps && t >= 44 && x.abs() == 9 && (t - 44).rem_euclid(20) == 1 {
                                    column.fill(if ((t - 44) / 20 + (x > 0) as i32) % 2 == 0 { red } else { green }, floor, floor + 1);
                                }
                            });
                        }
                    }
                }
                Kind::Cave { way, radius, deep } => {
                    // A mouth in the slope, narrowing and falling a little as it goes in.
                    let (into, right) = along(way);
                    for t in -2..=deep {
                        let narrow = radius as f32 * (1.0 - 0.5 * t.max(0) as f32 / deep as f32);
                        for x in -radius..=radius {
                            if (x as f32).abs() > narrow {
                                continue;
                            }
                            let (north, east) = (at[0] + into[0] * t + right[0] * x, at[1] + into[1] * t + right[1] * x);
                            edit(north, east, &mut edits, &mut |column, _| {
                                let low = floor - t.max(0) / 5;
                                let rise = ((narrow * narrow - (x * x) as f32).sqrt() * 1.4) as i32 + 2 + (hash(north, east, 30) % 2) as i32;
                                column.carve(low.max(1), low + rise);
                                // A lamp on the floor along its middle every ten blocks in, red and green by turns.
                                if lamps && x == 0 && t >= 4 && t % 10 == 4 {
                                    column.fill(if (t / 10) % 2 == 0 { red } else { green }, low.max(1), low.max(1) + 1);
                                }
                            });
                        }
                    }
                }
                Kind::Posts { way, rows, long } => {
                    let (into, right) = along(way);
                    for row in 0..rows {
                        for k in (0..long).step_by(4) {
                            edit(at[0] + into[0] * k + right[0] * row * 12, at[1] + into[1] * k + right[1] * row * 12, &mut edits, &mut |column, ground| column.fill(planks, ground + 1, ground + 4));
                        }
                    }
                }
                Kind::Slab { half, over } => {
                    for de in -half..half {
                        for dn in -half..half {
                            edit(at[0] + dn, at[1] + de, &mut edits, &mut |column, _| column.fill(dressed, floor + over, floor + over + 2));
                        }
                    }
                }
                Kind::Arch { half } => {
                    // Two piers 4 square and 20 high, and a round arch over the gap between them, 4 thick.
                    for dn in -2..2 {
                        for de in -half - 4..=half + 4 {
                            edit(at[0] + dn, at[1] + de, &mut edits, &mut |column, ground| {
                                if de.abs() > half {
                                    column.fill(dressed, ground + 1, floor + 20 + half + 4);
                                } else {
                                    let under = floor + 20 + ((half * half - de * de) as f32).sqrt() as i32;
                                    column.fill(dressed, under, floor + 20 + half + 4);
                                }
                            });
                        }
                    }
                }
            }
        }
        if edits.is_empty() {
            return None;
        }
        let mut staged = Staged { ends: Vec::with_capacity(side * side), runs: Vec::new() };
        let mut next = edits.into_iter().peekable();
        for k in 0..side * side {
            if next.peek().is_some_and(|(at, _)| *at == k) {
                let (_, mut column) = next.next().unwrap();
                column.tidy();
                for run in &mut column.runs {
                    run.to = run.to.min(self.height);
                }
                column.runs.retain(|run| run.to > run.from);
                debug_assert!(column.top() <= self.height);
                staged.runs.extend(column.runs);
            }
            staged.ends.push(staged.runs.len() as u32);
        }
        Some(staged)
    }
}

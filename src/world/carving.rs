//! Carving — the rivers and lakes cut into the ground [David and Builder, 2026-10-30: "Carving: the
//! curve and the carve in one go", docs/worldgen.md].
//!
//! The World Compiler's river stage makes the curves (`worldgen/carving.rs`); here they're kept with
//! the world and read at each column. A river is a smooth line carrying, all along it, its width, its
//! bank depth and its water level (only ever falling downstream). Each column measures its distance to
//! the nearest line: inside the river, the ground is cut down to the channel floor and water fills up
//! to the level. Lakes fill their basins to their saved surface, and a crag dip at a lake's edge is
//! filled up to the water, so water never stands against air.
//!
//! The carve lowers the recipe's height itself (`CompiledMap::height`), so the far terrain, the light
//! map and the World Compiler's viewer all see the rivers. Climate never reads it (`Mutes::rivers`):
//! the rivers are made from the climate, and mustn't feed back into it.

use super::terrain::SEA_LEVEL;
use serde::{Deserialize, Serialize};

/// One point along a river.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// Blocks from the world's middle.
    pub north: f32,
    pub east: f32,
    /// Blocks across, and blocks of water deep (at its deepest).
    pub width: f32,
    pub depth: f32,
    /// The shape across [David's profiles drawing, 2026-10-30]: how deep the water is at the bank (a
    /// share of the deepest), the share of the width at the deepest (its flat floor), and the bowl
    /// between (below 1, deep quickly then flat; above 1, shallow then dropping). 1, 1, 1 is a box.
    pub edge: f32,
    pub flat: f32,
    pub bowl: f32,
    /// The water's surface (a Z: water fills the blocks below it, as the sea fills those below 128).
    pub level: f32,
    /// Which way the river bends here: +1 its outside is to the left (looking downstream), −1 to the
    /// right, 0 straight. A river's odd block goes to the outside [David].
    pub bend: f32,
    /// Just below a waterfall: the water level above the fall (a curtain of water down its face);
    /// NaN elsewhere.
    pub curtain: f32,
    /// How sharply it bends here (its width over the bend's radius; + its outside to the left, as
    /// `bend`), and how far its banks fade into the land on the inside (blocks; 0 none) [2026-10-30].
    pub turn: f32,
    pub fade: f32,
}

/// A way a river's bank fades into the land.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeWay {
    /// Gentle at the water and at the top (an S-curve).
    #[default]
    Smooth,
    /// A straight ramp.
    Slope,
    /// Flat steps (old floodplains: walkable shelves along the river).
    Terraces,
    /// A 1-block ledge at the water, then smooth (a cut bank).
    LedgeThenSlope,
}

impl FadeWay {
    pub const ALL: [FadeWay; 4] = [FadeWay::Smooth, FadeWay::Slope, FadeWay::Terraces, FadeWay::LedgeThenSlope];

    pub fn name(self) -> &'static str {
        match self {
            FadeWay::Smooth => "smooth",
            FadeWay::Slope => "slope",
            FadeWay::Terraces => "terraces",
            FadeWay::LedgeThenSlope => "ledge then slope",
        }
    }
}

/// How the rivers' banks fade into the land [David's profiles proposal, step 3, 2026-10-30: shores
/// flush by default, ledges by context, "several ways to fade out smoothly"; docs/rivers.md section
/// 8]: the same for every river, kept with the world.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Banks {
    /// The way on the inside of a bend, its outside, and a straight run.
    pub ways: [FadeWay; 3],
    /// How sharp a bend (its width over its radius) before its sides are an inside and an outside.
    pub bend_from: f32,
    /// The outside's fade, as a share of the inside's (shorter: the water cuts that bank).
    pub outside: f32,
    /// The steepest a fade may be (blocks up for each across) before it widens, and the most it
    /// widens (times); past that, it stays steep (a gorge).
    pub steepest: f32,
    pub widest: f32,
    /// Terraces: each step's height (blocks).
    pub terrace: f32,
}

impl Default for Banks {
    fn default() -> Self {
        Self { ways: [FadeWay::Smooth, FadeWay::LedgeThenSlope, FadeWay::Smooth], bend_from: 0.15, outside: 0.5, steepest: 1.0, widest: 3.0, terrace: 4.0 }
    }
}

impl Banks {
    /// The ground at `beyond` blocks out from a river's water (at `level`), whose own ground there is
    /// `ground`: from the shore (flush with the water, or a block above it for a ledge) faded back to
    /// its own height over `fade` blocks, widened where that would be steeper than `steepest`. `None`
    /// past the fade.
    pub fn ground(&self, way: FadeWay, level: f32, fade: f32, beyond: f32, ground: f32) -> Option<f32> {
        let shore = if way == FadeWay::LedgeThenSlope { level } else { level - 1.0 };
        let rise = ground - shore;
        let fade = fade.max(1.0);
        let fade = if rise > 0.0 { (rise / self.steepest.max(0.05)).clamp(fade, fade * self.widest.max(1.0)) } else { fade };
        if beyond >= fade {
            return None;
        }
        let t = (beyond / fade).clamp(0.0, 1.0);
        let smooth = t * t * (3.0 - 2.0 * t);
        Some(match way {
            FadeWay::Smooth | FadeWay::LedgeThenSlope => shore + rise * smooth,
            FadeWay::Slope => shore + rise * t,
            FadeWay::Terraces if rise > 0.0 => shore + (rise * t / self.terrace.max(1.0)).floor() * self.terrace.max(1.0),
            FadeWay::Terraces => shore + rise * smooth,
        })
    }
}

/// A stretch of river, downstream, as points.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reach {
    pub points: Vec<Point>,
}

/// One lake, or a few ponds near each other, on a fine grid of its own [2026-10-30: each filled on the
/// real ground; it was one surface per 32-block patch of the broad ground, raised square pools with a
/// square wall round them (David's screenshot)]: its north-west corner (blocks from the world's
/// middle), cells across and down, the cell's size (blocks), and each cell's water surface (NaN where
/// dry); worked out from that, each dry cell's next-door water surface (NaN if none), so a dip there is
/// filled up to just under it and water never stands against air.
#[derive(Clone, Debug, Default)]
pub struct Lake {
    pub north: f32,
    pub west: f32,
    pub cols: usize,
    pub rows: usize,
    pub cell: f32,
    pub surface: Vec<f32>,
    beside: Vec<f32>,
}

/// The same lake: the same grid and the same water (dry cells, NaN, alike).
impl PartialEq for Lake {
    fn eq(&self, other: &Self) -> bool {
        (self.north, self.west, self.cols, self.rows, self.cell) == (other.north, other.west, other.cols, other.rows, other.cell)
            && self.surface.iter().map(|s| s.to_bits()).eq(other.surface.iter().map(|s| s.to_bits()))
    }
}

impl Lake {
    pub fn new(north: f32, west: f32, cols: usize, rows: usize, cell: f32, surface: Vec<f32>) -> Self {
        let beside = (0..cols * rows)
            .map(|i| {
                if surface[i].is_finite() {
                    return f32::NAN;
                }
                let (c, r) = ((i % cols) as i64, (i / cols) as i64);
                let at = |dc: i64, dr: i64| {
                    let (c, r) = (c + dc, r + dr);
                    if c < 0 || r < 0 || c >= cols as i64 || r >= rows as i64 { f32::NAN } else { surface[r as usize * cols + c as usize] }
                };
                [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)].iter().map(|&(dc, dr)| at(dc, dr)).filter(|s| s.is_finite()).fold(f32::NAN, f32::max)
            })
            .collect();
        Self { north, west, cols, rows, cell, surface, beside }
    }

    /// The cell a spot is in, if it's on this lake's grid.
    fn cell_at(&self, north: f32, east: f32) -> Option<usize> {
        let (col, row) = ((east - self.west) / self.cell, (self.north - north) / self.cell);
        (col >= 0.0 && row >= 0.0 && (col as usize) < self.cols && (row as usize) < self.rows).then(|| row as usize * self.cols + col as usize)
    }

    /// Whether there's any water in it.
    pub fn is_dry(&self) -> bool {
        !self.surface.iter().any(|s| s.is_finite())
    }
}

/// Every lake, and a lookup grid of which are near each spot.
#[derive(Clone, Debug, Default)]
pub struct Lakes {
    pub lakes: Vec<Lake>,
    /// The lookup: its cell size (blocks), cells across and down, its north-west corner, and in each
    /// cell the lakes whose grids reach it.
    cell: f32,
    across: (usize, usize),
    corner: (f32, f32),
    cells: Vec<Vec<u32>>,
}

impl PartialEq for Lakes {
    fn eq(&self, other: &Self) -> bool {
        self.lakes == other.lakes
    }
}

impl Lakes {
    pub fn new(lakes: Vec<Lake>) -> Self {
        let mut made = Self { lakes, cell: 256.0, ..Self::default() };
        if made.lakes.is_empty() {
            return made;
        }
        let (mut north, mut south, mut west, mut east) = (f32::MIN, f32::MAX, f32::MAX, f32::MIN);
        for l in &made.lakes {
            (north, west) = (north.max(l.north), west.min(l.west));
            (south, east) = (south.min(l.north - l.rows as f32 * l.cell), east.max(l.west + l.cols as f32 * l.cell));
        }
        made.corner = (north, west);
        made.across = (((east - west) / made.cell).ceil() as usize + 1, ((north - south) / made.cell).ceil() as usize + 1);
        made.cells = vec![Vec::new(); made.across.0 * made.across.1];
        for (k, l) in made.lakes.iter().enumerate() {
            let (c0, r0) = made.lookup(l.north, l.west);
            let (c1, r1) = made.lookup(l.north - l.rows as f32 * l.cell, l.west + l.cols as f32 * l.cell);
            for r in r0..=r1 {
                for c in c0..=c1 {
                    made.cells[r * made.across.0 + c].push(k as u32);
                }
            }
        }
        made
    }

    fn lookup(&self, north: f32, east: f32) -> (usize, usize) {
        let col = ((east - self.corner.1) / self.cell).floor().clamp(0.0, self.across.0 as f32 - 1.0) as usize;
        let row = ((self.corner.0 - north) / self.cell).floor().clamp(0.0, self.across.1 as f32 - 1.0) as usize;
        (col, row)
    }

    /// The lake water at a spot: its surface, and whether the spot is only beside it (dry, next to
    /// the water).
    pub fn at(&self, north: f32, east: f32) -> Option<(f32, bool)> {
        if self.cells.is_empty() {
            return None;
        }
        let (col, row) = self.lookup(north, east);
        let mut beside = None;
        for &k in &self.cells[row * self.across.0 + col] {
            let lake = &self.lakes[k as usize];
            if let Some(i) = lake.cell_at(north, east) {
                if lake.surface[i].is_finite() {
                    return Some((lake.surface[i], false));
                }
                if lake.beside[i].is_finite() {
                    beside = Some((lake.beside[i], true));
                }
            }
        }
        beside
    }

    /// The lake water's surface at a spot, if there's water there.
    pub fn water(&self, north: f32, east: f32) -> Option<f32> {
        self.at(north, east).filter(|&(_, beside)| !beside).map(|(surface, _)| surface)
    }
}

/// What the carving does to a column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Here {
    /// The ground is cut no higher than this (a Z, with fractions).
    pub floor: Option<f32>,
    /// …or raised at least to this (a lake's edge, filled).
    pub fill: Option<f32>,
    /// Water fills the blocks from the ground up to below this Z.
    pub water: Option<i32>,
}

/// Every river and lake, and a lookup grid of the rivers' pieces.
#[derive(Clone, Debug, Default)]
pub struct Carving {
    pub reaches: Vec<Reach>,
    pub lakes: Lakes,
    pub banks: Banks,
    /// The grid: its cell size (blocks), cells across, the north-west corner, and in each cell the
    /// pieces (reach, first point) passing near it.
    cell: f32,
    across: usize,
    corner: (f32, f32),
    cells: Vec<Vec<(u32, u32)>>,
    /// Whether there's anything to carve (worked out once: the height asks at every column).
    full: bool,
    /// How its making went, for the panel (not saved): how long it took (seconds), the water's
    /// deepest cut below the lowest ground across a river (blocks), how many points cut deep, and
    /// how many lakes and ponds.
    pub took: f32,
    pub deepest_cut: f32,
    pub deep_cuts: usize,
    pub pools: usize,
}

impl PartialEq for Carving {
    fn eq(&self, other: &Self) -> bool {
        self.reaches == other.reaches && self.lakes == other.lakes && self.banks == other.banks
    }
}

/// How deep a river's water is (whole blocks) at `across` (0 its middle, ±1 its banks), `depth` deep
/// at its deepest: full depth over its flat floor, then the bowl's curve up to the edge depth at the
/// bank; so the bed steps a block at a time, the steps wider the wider the river [David's profiles
/// drawing, 2026-10-30, docs/rivers.md section 8].
pub fn depth_across(depth: f32, edge: f32, flat: f32, bowl: f32, across: f32) -> f32 {
    let flat = flat.clamp(0.0, 0.95);
    let u = across.abs().min(1.0);
    let t = if u <= flat { 1.0 } else { (1.0 - u) / (1.0 - flat) };
    let at_edge = edge.clamp(0.0, 1.0) * depth;
    (at_edge + (depth - at_edge) * t.powf(bowl.max(0.05))).round().max(1.0)
}

/// Where a spot is along a river piece: how far from its line (blocks; + toward the left, looking
/// downstream), and how far along it (0 to 1).
fn along(a: &Point, b: &Point, north: f32, east: f32) -> (f32, f32, f32) {
    let (dn, de) = (b.north - a.north, b.east - a.east);
    let length2 = (dn * dn + de * de).max(1e-6);
    let t = (((north - a.north) * dn + (east - a.east) * de) / length2).clamp(0.0, 1.0);
    let (pn, pe) = (a.north + dn * t, a.east + de * t);
    // Left of the way downstream: north is "up", east "right"; going (dn, de), left is (de, -dn)…
    // as a cross product's sign.
    let side = dn * (east - a.east) - de * (north - a.north);
    let distance = ((north - pn).powi(2) + (east - pe).powi(2)).sqrt();
    (distance, if side >= 0.0 { -1.0 } else { 1.0 }, t)
}

impl Carving {
    pub fn new(reaches: Vec<Reach>, lakes: Lakes, banks: Banks) -> Self {
        let full = !reaches.is_empty() || lakes.lakes.iter().any(|lake| !lake.is_dry());
        let mut carving = Self { reaches, lakes, banks, full, ..Self::default() };
        carving.index();
        carving
    }

    pub fn is_empty(&self) -> bool {
        !self.full
    }

    /// How far from its line a river reaches at a point: half its width, its banks' fade at its
    /// widest, and a couple of blocks.
    fn reach_of(&self, p: &Point) -> f32 {
        p.width / 2.0 + p.fade * self.banks.widest.max(1.0) + 2.0
    }

    /// The lookup grid: each piece entered in every cell within its reach.
    fn index(&mut self) {
        let points = self.reaches.iter().flat_map(|reach| reach.points.iter());
        let (mut low, mut high) = ((f32::MAX, f32::MAX), (f32::MIN, f32::MIN));
        let mut furthest = 1.0f32;
        for p in points {
            low = (low.0.min(p.north), low.1.min(p.east));
            high = (high.0.max(p.north), high.1.max(p.east));
            furthest = furthest.max(self.reach_of(p));
        }
        if low.0 > high.0 {
            return;
        }
        // Small cells (each piece entered in every one its banks reach), so a column far from the big
        // rivers looks through few pieces.
        self.cell = 64.0;
        let margin = furthest;
        self.corner = (high.0 + margin, low.1 - margin);
        self.across = (((high.0 - low.0).max(high.1 - low.1) + 2.0 * margin) / self.cell).ceil() as usize + 1;
        self.cells = vec![Vec::new(); self.across * self.across];
        for (r, reach) in self.reaches.iter().enumerate() {
            for (k, pair) in reach.points.windows(2).enumerate() {
                let reach_of = self.reach_of(&pair[0]).max(self.reach_of(&pair[1]));
                let (n0, n1) = (pair[0].north.min(pair[1].north) - reach_of, pair[0].north.max(pair[1].north) + reach_of);
                let (e0, e1) = (pair[0].east.min(pair[1].east) - reach_of, pair[0].east.max(pair[1].east) + reach_of);
                let (r0, r1) = (self.row(n1), self.row(n0));
                let (c0, c1) = (self.col(e0), self.col(e1));
                for row in r0..=r1 {
                    for col in c0..=c1 {
                        self.cells[row * self.across + col].push((r as u32, k as u32));
                    }
                }
            }
        }
    }

    fn row(&self, north: f32) -> usize {
        (((self.corner.0 - north) / self.cell).floor().max(0.0) as usize).min(self.across.saturating_sub(1))
    }

    fn col(&self, east: f32) -> usize {
        (((east - self.corner.1) / self.cell).floor().max(0.0) as usize).min(self.across.saturating_sub(1))
    }

    /// The nearest river piece within its own reach of a spot: (its distance, which side, the point
    /// there blended between its ends, and how far past the piece's start).
    fn nearest_river(&self, north: f32, east: f32) -> Option<(f32, f32, Point, f32)> {
        if self.cells.is_empty() {
            return None;
        }
        let inside = north <= self.corner.0 && east >= self.corner.1 && north > self.corner.0 - self.across as f32 * self.cell && east < self.corner.1 + self.across as f32 * self.cell;
        if !inside {
            return None;
        }
        let mut best: Option<(f32, f32, Point, f32)> = None;
        for &(r, k) in &self.cells[self.row(north) * self.across + self.col(east)] {
            let points = &self.reaches[r as usize].points;
            let (a, b) = (&points[k as usize], &points[k as usize + 1]);
            let (distance, side, t) = along(a, b, north, east);
            if best.as_ref().is_some_and(|(d, ..)| *d <= distance) {
                continue;
            }
            let mix = |x: f32, y: f32| x + (y - x) * t;
            // The level holds the upstream point's until the next (steps happen at points).
            let point = Point { north: mix(a.north, b.north), east: mix(a.east, b.east), width: mix(a.width, b.width), depth: mix(a.depth, b.depth), edge: mix(a.edge, b.edge), flat: mix(a.flat, b.flat), bowl: mix(a.bowl, b.bowl), level: a.level, bend: if t < 0.5 { a.bend } else { b.bend }, curtain: if t < 0.5 { a.curtain } else { f32::NAN }, turn: mix(a.turn, b.turn), fade: mix(a.fade, b.fade) };
            let span = ((b.north - a.north).powi(2) + (b.east - a.east).powi(2)).sqrt();
            best = Some((distance, side, point, t * span));
        }
        best
    }

    /// What the rivers and lakes do to the column at a spot whose uncarved ground is `ground`.
    pub fn here(&self, north: f32, east: f32, ground: f32) -> Here {
        let mut here = Here { floor: None, fill: None, water: None };
        if let Some((distance, side, point, _)) = self.nearest_river(north, east) {
            // Inside the river: within half its width of the line, the odd block of an even width to
            // the outside of the bend (centred where it runs straight) [David].
            let half = point.width.max(1.0) / 2.0;
            let even = (point.width.round() as i32) % 2 == 0;
            let shift = if even { 0.5 * point.bend } else { 0.0 };
            let offset = distance * side - shift;
            if offset.abs() <= half {
                let level = point.level.round();
                let top = if point.curtain.is_finite() { point.curtain.round().max(level) } else { level };
                here.floor = Some(level - 1.0 - depth_across(point.depth, point.edge, point.flat, point.bowl, offset / half));
                here.water = Some(top as i32);
            } else if point.fade > 0.0 {
                // Its bank: the ground faded from the shore back to its own, the way the bank's
                // side of the bend takes (inside, outside or straight).
                let b = &self.banks;
                let bent = point.turn.abs() >= b.bend_from;
                let outside = bent && side == point.turn.signum();
                let way = b.ways[if !bent { 2 } else if outside { 1 } else { 0 }];
                let fade = point.fade * if outside { b.outside.clamp(0.05, 1.0) } else { 1.0 };
                if let Some(bank) = b.ground(way, point.level.round(), fade, offset.abs() - half, ground) {
                    if bank < ground {
                        here.floor = Some(bank);
                    } else {
                        here.fill = Some(bank);
                    }
                }
            }
        }
        // Lakes: water to the surface wherever the ground is below it; beside one, a dip below the
        // surface is filled up to it.
        if let Some((surface, edge)) = self.lakes.at(north, east)
            && surface > SEA_LEVEL as f32
        {
            let top = surface.round();
            if edge {
                if ground < top - 1.0 {
                    here.fill = Some(top - 1.0);
                }
            } else if ground < top - 0.5 {
                here.water = Some(here.water.map_or(top as i32, |w| w.max(top as i32)));
            }
        }
        here
    }

    /// How far a spot is from the nearest river's line, and that river's width there (for the far
    /// terrain, to blend in the water of creeks narrower than its cells).
    pub fn nearest(&self, north: f32, east: f32) -> Option<(f32, f32)> {
        self.nearest_river(north, east).map(|(distance, _, point, _)| (distance, point.width))
    }

    /// As text, for saving with a world: each lake's grid and its water cells, then each reach's
    /// points.
    pub fn to_text(&self) -> String {
        let b = &self.banks;
        let way = |w: FadeWay| FadeWay::ALL.iter().position(|&a| a == w).unwrap_or(0);
        let mut text = format!("banks {} {} {} {} {} {} {} {}\n", way(b.ways[0]), way(b.ways[1]), way(b.ways[2]), b.bend_from, b.outside, b.steepest, b.widest, b.terrace);
        for lake in &self.lakes.lakes {
            text += &format!("lake {} {} {} {} {}\n", lake.north, lake.west, lake.cols, lake.rows, lake.cell);
            for (i, s) in lake.surface.iter().enumerate().filter(|(_, s)| s.is_finite()) {
                text += &format!("w {i} {s}\n");
            }
        }
        for reach in &self.reaches {
            text += "r\n";
            for p in &reach.points {
                text += &format!("p {} {} {} {} {} {} {} {} {} {} {} {}\n", p.north, p.east, p.width, p.depth, p.level, p.bend, p.curtain, p.edge, p.flat, p.bowl, p.turn, p.fade);
            }
        }
        text
    }

    /// Read back from `to_text`.
    pub fn from_text(text: &str) -> Result<Self, String> {
        // Each lake's grid (north, west, cols, rows, cell) and surfaces, made up at the end.
        let mut grids: Vec<(f32, f32, usize, usize, f32, Vec<f32>)> = Vec::new();
        let mut reaches: Vec<Reach> = Vec::new();
        let mut banks = Banks::default();
        for (number, line) in text.lines().enumerate() {
            let words: Vec<&str> = line.split_whitespace().collect();
            let bad = || format!("line {}: {line}", number + 1);
            let num = |k: usize| -> Result<f32, String> { words.get(k).and_then(|w| w.parse::<f32>().ok()).ok_or_else(bad) };
            match words.first().copied() {
                Some("banks") => {
                    let way = |k: usize| num(k).map(|w| FadeWay::ALL[(w as usize).min(3)]);
                    banks = Banks { ways: [way(1)?, way(2)?, way(3)?], bend_from: num(4)?, outside: num(5)?, steepest: num(6)?, widest: num(7)?, terrace: num(8)? };
                }
                Some("lake") => {
                    let (cols, rows) = (num(3)? as usize, num(4)? as usize);
                    grids.push((num(1)?, num(2)?, cols, rows, num(5)?, vec![f32::NAN; cols * rows]));
                }
                // Saved before the lakes were filled on the real ground (2026-10-30): one grid over
                // the whole map, a cell a patch.
                Some("lakes") => {
                    let (width, blocks) = (num(1)? as usize, num(2)?);
                    let half = width as f32 / 2.0 * blocks;
                    grids.push((half, -half, width, width, blocks, vec![f32::NAN; width * width]));
                }
                Some("w" | "l") => {
                    let i = num(1)? as usize;
                    *grids.last_mut().and_then(|grid| grid.5.get_mut(i)).ok_or_else(bad)? = num(2)?;
                }
                Some("r") => reaches.push(Reach::default()),
                // Saved before the shape across (2026-10-30): a box; before the banks: none.
                Some("p") => {
                    let or = |k: usize, missing: f32| if words.len() > k { num(k) } else { Ok(missing) };
                    let point = Point { north: num(1)?, east: num(2)?, width: num(3)?, depth: num(4)?, level: num(5)?, bend: num(6)?, curtain: num(7)?, edge: or(8, 1.0)?, flat: or(9, 1.0)?, bowl: or(10, 1.0)?, turn: or(11, 0.0)?, fade: or(12, 0.0)? };
                    reaches.last_mut().ok_or_else(bad)?.points.push(point);
                }
                None => {}
                Some(_) => return Err(bad()),
            }
        }
        let lakes = grids.into_iter().filter(|grid| grid.2 * grid.3 > 0).map(|(north, west, cols, rows, cell, surface)| Lake::new(north, west, cols, rows, cell, surface)).collect();
        Ok(Self::new(reaches, Lakes::new(lakes), banks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(north: f32, east: f32, width: f32, level: f32, bend: f32) -> Point {
        Point { north, east, width, depth: 2.0, edge: 1.0, flat: 1.0, bowl: 1.0, level, bend, curtain: f32::NAN, turn: 0.0, fade: 0.0 }
    }

    #[test]
    fn the_bed_steps_up_from_the_middle_to_the_banks() {
        // A creek of David's drawing, 5 wide and 3 deep: 1-2-3-2-1.
        let column = |k: f32| depth_across(3.0, 0.3, 0.2, 1.0, k / 2.5);
        assert_eq!([-2.0, -1.0, 0.0, 1.0, 2.0].map(column), [1.0, 2.0, 3.0, 2.0, 1.0]);
        // A box is full depth all across; a shelf (bowl above 1) stays shallow longer than a V.
        assert!([0.0, 0.5, 0.99].iter().all(|&u| depth_across(4.0, 1.0, 1.0, 1.0, u) == 4.0));
        assert!(depth_across(6.0, 0.2, 0.3, 2.0, 0.8) < depth_across(6.0, 0.2, 0.3, 1.0, 0.8));
    }

    #[test]
    fn a_river_saved_before_its_shape_reads_back_as_a_box() {
        let carving = Carving::from_text("lakes 0 32\nr\np 0.5 -50 3 2 150 0 NaN\np 0.5 50 3 2 150 0 NaN\n").unwrap();
        let p = carving.reaches[0].points[0];
        assert_eq!((p.edge, p.flat, p.bowl), (1.0, 1.0, 1.0));
    }

    /// A river running east along north = 0.5, `width` wide, its water at Z 150.
    fn straight(width: f32, bend: f32) -> Carving {
        Carving::new(vec![Reach { points: vec![point(0.5, -50.0, width, 150.0, bend), point(0.5, 50.0, width, 150.0, bend)] }], Lakes::default(), Banks::default())
    }

    /// The same, 4 wide, with banks fading over 8 blocks, bending as `turn` (+ its outside north).
    fn banked(turn: f32) -> Carving {
        let at = |east: f32| Point { turn, fade: 8.0, ..point(0.5, east, 4.0, 150.0, 0.0) };
        Carving::new(vec![Reach { points: vec![at(-50.0), at(50.0)] }], Lakes::default(), Banks::default())
    }

    #[test]
    fn banks_are_flush_with_the_water_and_fade_back_into_the_land() {
        let river = banked(0.0);
        // A block out from the water, higher ground is cut to the shore (its top at the water's 150).
        let cut = river.here(3.5, 0.0, 160.0).floor.unwrap();
        assert_eq!(cut.round(), 149.0, "{cut}");
        // Lower ground is raised to it (a levee), so the water never stands against air.
        let raised = river.here(3.5, 0.0, 140.0).fill.unwrap();
        assert_eq!(raised.round(), 149.0, "{raised}");
        // Past the fade (widened to 11 blocks for ground 11 above the shore), the land is its own.
        assert_eq!(river.here(14.5, 0.0, 160.0), Here { floor: None, fill: None, water: None });
        // Halfway out, halfway back up.
        let half = river.here(2.5 + 5.5, 0.0, 160.0).floor.unwrap();
        assert!((153.0..156.0).contains(&half), "{half}");
    }

    #[test]
    fn a_bend_s_outside_bank_is_a_ledge_and_its_inside_is_flush() {
        let river = banked(1.0);
        // Its outside (north): a block above the water, then the slope.
        assert_eq!(river.here(3.5, 0.0, 160.0).floor.unwrap().round(), 150.0);
        // Its inside (south): flush.
        assert_eq!(river.here(-2.5, 0.0, 160.0).floor.unwrap().round(), 149.0);
    }

    #[test]
    fn banks_read_back_as_saved() {
        let banks = Banks { ways: [FadeWay::Terraces, FadeWay::Slope, FadeWay::LedgeThenSlope], steepest: 2.0, ..Banks::default() };
        let mut river = banked(0.3);
        river = Carving::new(river.reaches.clone(), Lakes::default(), banks);
        let again = Carving::from_text(&river.to_text()).unwrap();
        assert_eq!((again.banks, again.to_text()), (banks, river.to_text()));
        assert_eq!(again.here(3.5, 0.0, 160.0), river.here(3.5, 0.0, 160.0));
    }

    #[test]
    fn a_river_cuts_its_channel_and_fills_it_with_water_to_its_level() {
        let carving = straight(3.0, 0.0);
        let here = carving.here(0.5, 0.0, 160.0);
        // Water up to Z 149 (its level, 150, as the sea's 128), the floor 2 deep below it.
        assert_eq!(here.water, Some(150));
        assert_eq!(here.floor, Some(147.0));
        // Outside the river: nothing.
        assert_eq!(carving.here(5.0, 0.0, 160.0), Here { floor: None, fill: None, water: None });
    }

    #[test]
    fn a_creek_s_odd_block_goes_to_the_outside_of_the_bend() {
        // 2 wide: straight, the blocks either side of the line; bending with the outside to the left
        // (north, here), the line's block and the one north of it.
        let inside = |carving: &Carving, north: f32| carving.here(north, 0.0, 160.0).water.is_some();
        let straight_creek = straight(2.0, 0.0);
        assert!(inside(&straight_creek, 0.5) && inside(&straight_creek, -0.5) || inside(&straight_creek, 1.5));
        let bent = straight(2.0, 1.0);
        assert!(inside(&bent, 0.5) && inside(&bent, 1.5), "the outer block (north) is river");
        assert!(!inside(&bent, -0.5), "the inner block isn't");
        let other_way = straight(2.0, -1.0);
        assert!(inside(&other_way, 0.5) && inside(&other_way, -0.5) && !inside(&other_way, 1.5));
    }

    /// A lake on a 3 x 3 grid of 32-block cells round the world's middle, the middle cell water at
    /// Z 140.
    fn pond() -> Lakes {
        let mut surface = vec![f32::NAN; 9];
        surface[4] = 140.0;
        Lakes::new(vec![Lake::new(48.0, -48.0, 3, 3, 32.0, surface)])
    }

    #[test]
    fn a_lake_fills_below_its_surface_and_the_dips_beside_it_fill_to_the_water() {
        let carving = Carving::new(Vec::new(), pond(), Banks::default());
        assert_eq!(carving.here(0.0, 0.0, 130.0).water, Some(140), "in the lake");
        assert_eq!(carving.here(0.0, 0.0, 145.0).water, None, "an island");
        // Beside it: a dip below the water is filled up to it; higher ground is left; further off,
        // nothing.
        assert_eq!(carving.here(40.0, 0.0, 120.0).fill, Some(139.0));
        assert_eq!(carving.here(40.0, 0.0, 150.0).fill, None);
        assert_eq!(carving.here(200.0, 0.0, 120.0), Here { floor: None, fill: None, water: None });
    }

    #[test]
    fn the_carving_reads_back_as_saved() {
        let carving = Carving::new(vec![Reach { points: vec![point(0.5, -50.0, 3.0, 150.0, 1.0), Point { curtain: 155.0, ..point(0.5, 50.0, 3.0, 148.0, -1.0) }] }], pond(), Banks::default());
        let again = Carving::from_text(&carving.to_text()).unwrap();
        assert_eq!(again.reaches.len(), 1);
        assert_eq!(again.reaches[0].points[1].curtain, 155.0);
        assert_eq!(again.lakes, carving.lakes);
        assert_eq!(again.here(0.5, 0.0, 160.0), carving.here(0.5, 0.0, 160.0));
        assert_eq!(again.here(40.0, 0.0, 120.0), carving.here(40.0, 0.0, 120.0));
    }

    #[test]
    fn lakes_saved_on_the_old_patch_grid_read_back_the_same() {
        let old = Carving::from_text("lakes 3 32\nl 4 140\n").unwrap();
        assert_eq!(old.here(0.0, 0.0, 130.0).water, Some(140));
        assert_eq!(old.here(40.0, 0.0, 120.0).fill, Some(139.0));
    }
}

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

/// One point along a river.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// Blocks from the world's middle.
    pub north: f32,
    pub east: f32,
    /// Blocks across, and blocks of water deep.
    pub width: f32,
    pub depth: f32,
    /// The water's surface (a Z: water fills the blocks below it, as the sea fills those below 128).
    pub level: f32,
    /// Which way the river bends here: +1 its outside is to the left (looking downstream), −1 to the
    /// right, 0 straight. A river's odd block goes to the outside [David].
    pub bend: f32,
    /// Just below a waterfall: the water level above the fall (a curtain of water down its face);
    /// NaN elsewhere.
    pub curtain: f32,
}

/// A stretch of river, downstream, as points.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reach {
    pub points: Vec<Point>,
}

/// The lakes on the map's grid: each patch's lake surface (NaN for none).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lakes {
    pub width: usize,
    pub blocks: f32,
    pub surface: Vec<f32>,
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
    /// The grid: its cell size (blocks), cells across, the north-west corner, and in each cell the
    /// pieces (reach, first point) passing near it.
    cell: f32,
    across: usize,
    corner: (f32, f32),
    cells: Vec<Vec<(u32, u32)>>,
    /// Whether there's anything to carve (worked out once: the height asks at every column).
    full: bool,
}

impl PartialEq for Carving {
    fn eq(&self, other: &Self) -> bool {
        self.reaches == other.reaches && self.lakes == other.lakes
    }
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
    pub fn new(reaches: Vec<Reach>, lakes: Lakes) -> Self {
        let full = !reaches.is_empty() || lakes.surface.iter().any(|s| s.is_finite());
        let mut carving = Self { reaches, lakes, full, ..Self::default() };
        carving.index();
        carving
    }

    pub fn is_empty(&self) -> bool {
        !self.full
    }

    /// The lookup grid: each piece entered in every cell within its reach (half its width, and a block).
    fn index(&mut self) {
        let points = self.reaches.iter().flat_map(|reach| reach.points.iter());
        let (mut low, mut high) = ((f32::MAX, f32::MAX), (f32::MIN, f32::MIN));
        let mut widest = 1.0f32;
        for p in points {
            low = (low.0.min(p.north), low.1.min(p.east));
            high = (high.0.max(p.north), high.1.max(p.east));
            widest = widest.max(p.width);
        }
        if low.0 > high.0 {
            return;
        }
        self.cell = (widest + 2.0).max(64.0);
        let margin = widest / 2.0 + 2.0;
        self.corner = (high.0 + margin, low.1 - margin);
        self.across = (((high.0 - low.0).max(high.1 - low.1) + 2.0 * margin) / self.cell).ceil() as usize + 1;
        self.cells = vec![Vec::new(); self.across * self.across];
        for (r, reach) in self.reaches.iter().enumerate() {
            for (k, pair) in reach.points.windows(2).enumerate() {
                let reach_of = pair[0].width.max(pair[1].width) / 2.0 + 2.0;
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
            let point = Point { north: mix(a.north, b.north), east: mix(a.east, b.east), width: mix(a.width, b.width), depth: mix(a.depth, b.depth), level: a.level, bend: if t < 0.5 { a.bend } else { b.bend }, curtain: if t < 0.5 { a.curtain } else { f32::NAN } };
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
                here.floor = Some(level - 1.0 - point.depth.max(1.0).round());
                here.water = Some(top as i32);
            }
        }
        // Lakes: water to the surface wherever the ground is below it; next to one, a crag dip below
        // the surface is filled up to it.
        if let Some((surface, edge)) = self.lake_near(north, east)
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

    /// The lake at a spot: its surface, and whether the spot is only at its edge (in the ring of
    /// patches round it, not its own).
    fn lake_near(&self, north: f32, east: f32) -> Option<(f32, bool)> {
        let l = &self.lakes;
        if l.width == 0 || l.surface.is_empty() {
            return None;
        }
        let half = l.width as f32 / 2.0;
        let (col, row) = ((east / l.blocks + half).floor() as i64, (half - north / l.blocks).floor() as i64);
        let at = |c: i64, r: i64| if c < 0 || r < 0 || c >= l.width as i64 || r >= l.width as i64 { f32::NAN } else { l.surface[r as usize * l.width + c as usize] };
        let own = at(col, row);
        if own.is_finite() {
            return Some((own, false));
        }
        let ring = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)].iter().map(|&(dc, dr)| at(col + dc, row + dr)).filter(|s| s.is_finite()).fold(f32::NAN, f32::max);
        ring.is_finite().then_some((ring, true))
    }

    /// How far a spot is from the nearest river's line, and that river's width there (for the far
    /// terrain, to blend in the water of creeks narrower than its cells).
    pub fn nearest(&self, north: f32, east: f32) -> Option<(f32, f32)> {
        self.nearest_river(north, east).map(|(distance, _, point, _)| (distance, point.width))
    }

    /// As text, for saving with a world: the lakes' grid, then each reach's points.
    pub fn to_text(&self) -> String {
        let mut text = format!("lakes {} {}\n", self.lakes.width, self.lakes.blocks);
        for (i, s) in self.lakes.surface.iter().enumerate().filter(|(_, s)| s.is_finite()) {
            text += &format!("l {i} {s}\n");
        }
        for reach in &self.reaches {
            text += "r\n";
            for p in &reach.points {
                text += &format!("p {} {} {} {} {} {} {}\n", p.north, p.east, p.width, p.depth, p.level, p.bend, p.curtain);
            }
        }
        text
    }

    /// Read back from `to_text`.
    pub fn from_text(text: &str) -> Result<Self, String> {
        let mut lakes = Lakes::default();
        let mut reaches: Vec<Reach> = Vec::new();
        for (number, line) in text.lines().enumerate() {
            let words: Vec<&str> = line.split_whitespace().collect();
            let bad = || format!("line {}: {line}", number + 1);
            let num = |k: usize| -> Result<f32, String> { words.get(k).and_then(|w| w.parse::<f32>().ok()).ok_or_else(bad) };
            match words.first().copied() {
                Some("lakes") => {
                    lakes.width = num(1)? as usize;
                    lakes.blocks = num(2)?;
                    lakes.surface = vec![f32::NAN; lakes.width * lakes.width];
                }
                Some("l") => {
                    let i = num(1)? as usize;
                    *lakes.surface.get_mut(i).ok_or_else(bad)? = num(2)?;
                }
                Some("r") => reaches.push(Reach::default()),
                Some("p") => reaches.last_mut().ok_or_else(bad)?.points.push(Point { north: num(1)?, east: num(2)?, width: num(3)?, depth: num(4)?, level: num(5)?, bend: num(6)?, curtain: num(7)? }),
                None => {}
                Some(_) => return Err(bad()),
            }
        }
        Ok(Self::new(reaches, lakes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(north: f32, east: f32, width: f32, level: f32, bend: f32) -> Point {
        Point { north, east, width, depth: 2.0, level, bend, curtain: f32::NAN }
    }

    /// A river running east along north = 0.5, `width` wide, its water at Z 150.
    fn straight(width: f32, bend: f32) -> Carving {
        Carving::new(vec![Reach { points: vec![point(0.5, -50.0, width, 150.0, bend), point(0.5, 50.0, width, 150.0, bend)] }], Lakes::default())
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

    #[test]
    fn a_lake_fills_below_its_surface_and_its_crag_dip_edges_fill_to_the_water() {
        // A 3 x 3 grid of 32-block patches, the middle one a lake at Z 140.
        let mut surface = vec![f32::NAN; 9];
        surface[4] = 140.0;
        let carving = Carving::new(Vec::new(), Lakes { width: 3, blocks: 32.0, surface });
        assert_eq!(carving.here(0.0, 0.0, 130.0).water, Some(140), "in the lake");
        assert_eq!(carving.here(0.0, 0.0, 145.0).water, None, "an island");
        // In the ring round it: a dip below the water is filled up to it; higher ground is left.
        assert_eq!(carving.here(40.0, 0.0, 120.0).fill, Some(139.0));
        assert_eq!(carving.here(40.0, 0.0, 150.0).fill, None);
    }

    #[test]
    fn the_carving_reads_back_as_saved() {
        let mut surface = vec![f32::NAN; 9];
        surface[4] = 140.0;
        let carving = Carving::new(vec![Reach { points: vec![point(0.5, -50.0, 3.0, 150.0, 1.0), Point { curtain: 155.0, ..point(0.5, 50.0, 3.0, 148.0, -1.0) }] }], Lakes { width: 3, blocks: 32.0, surface });
        let again = Carving::from_text(&carving.to_text()).unwrap();
        assert_eq!(again.reaches.len(), 1);
        assert_eq!(again.reaches[0].points[1].curtain, 155.0);
        assert_eq!(again.lakes.surface[4], 140.0);
        assert!(again.lakes.surface[0].is_nan());
        assert_eq!(again.here(0.5, 0.0, 160.0), carving.here(0.5, 0.0, 160.0));
    }
}

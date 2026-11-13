//! The marks: the places every flight stands at, the same each time, as data (`data/marks.toml`; plan 2.1).
//!
//! Each eye is placed from the doorway's answer: a player's eye over the ground or the water under it, and higher if
//! the mark says. A mark's changes (an event log's placings: David's build) and its scatter (random blocks round it, at
//! each count of the dial) are made through the doorway before it's flown. Old worked its marks out from the map; the
//! file holds the places old flew.

use crate::doorway::{Axis, Change, Doorway};
use glam::Vec3;
use serde::Deserialize;

/// A player's eye over the ground (old's, the dyno's marks).
pub const PLAYER_EYE: f32 = 1.38;

#[derive(Deserialize)]
struct MarksFile {
    mark: Vec<Mark>,
}

/// A mark (each field is described in `data/marks.toml`).
#[derive(Deserialize, Clone, Debug)]
pub struct Mark {
    pub name: String,
    pub at: [f32; 2],
    pub above: f32,
    pub facing: f32,
    pub pitch: f32,
    pub fog: bool,
    pub eyes: Vec<String>,
    pub changes: Option<String>,
    pub scatter: Option<Scatter>,
    /// On a flight, the camera flies straight ahead along its facing, level, at this many blocks a second (none: it
    /// stands still).
    pub speed: Option<f32>,
    /// On a flight, the camera pitched slowly up and down through this many degrees (none: held still), so a horizon's
    /// worst case is always met (the Oracle's step 0).
    pub nod: Option<f32>,
}

/// Random whole blocks round a mark: `counts` of them for each setting of the dial, within `around` columns of it each
/// way, from the surface to `high` over it, each one of `blocks`, the same every time for the same `seed`.
#[derive(Deserialize, Clone, Debug)]
pub struct Scatter {
    pub counts: Vec<u32>,
    pub around: i32,
    pub high: i32,
    pub blocks: Vec<String>,
    pub seed: u64,
}

/// The marks, in the file's order.
pub fn load() -> Result<Vec<Mark>, String> {
    let path = crate::session::project_folder().join("data").join("marks.toml");
    let text = std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let file: MarksFile = toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(file.mark)
}

/// Where a mark's eye is: north, east, and a player's eye (more if it's higher) over the ground or water under it.
pub fn eye(doorway: &Doorway, mark: &Mark) -> Result<Vec3, String> {
    let column = doorway.column(mark.at[0].floor() as i32, mark.at[1].floor() as i32)?;
    let surface = doorway.surface(&column).ok_or_else(|| format!("{} is past the map", mark.name))?;
    Ok(Vec3::new(mark.at[0], mark.at[1], surface as f32 + PLAYER_EYE + mark.above))
}

/// What a mark's placings made: whole blocks, bars, and those the doorway refused.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Made {
    pub blocks: usize,
    pub bars: usize,
    pub refused: usize,
}

/// An event log's placings (its "edit request" lines: whole blocks and bars; other edits aren't read), made as changes
/// through the doorway, in order.
pub fn place(doorway: &mut Doorway, log: &str) -> Made {
    let three = |text: &str| -> Option<[i32; 3]> {
        let mut numbers = text.split_whitespace().map(|word| word.parse::<i32>().ok());
        Some([numbers.next()??, numbers.next()??, numbers.next()??])
    };
    let mut made = Made::default();
    for line in log.lines().filter(|line| line.contains("edit request")) {
        let Some(what) = line.split_once("asks to place ").map(|(_, what)| what) else { continue };
        // "a north–south bar of incus:stone_brick from -7980 -2381 272 (half blocks) at -3990 -1191 136"
        let change = if let Some(bar) = what.strip_prefix("a ").filter(|rest| rest.contains(" bar of ")) {
            let parsed = (|| {
                let (axis, rest) = bar.split_once(" bar of ")?;
                let axis = match axis {
                    "north–south" => Axis::NorthSouth,
                    "east–west" => Axis::EastWest,
                    "upright" => Axis::Upright,
                    _ => return None,
                };
                let (block, rest) = rest.split_once(" from ")?;
                let (from, cell) = rest.split_once(" (half blocks) at ")?;
                Some((axis, block.to_string(), three(from)?, three(cell)?))
            })();
            let Some((axis, block, from, cell)) = parsed else { continue };
            Change::Bar { cell, from, axis, block: doorway.block(&block) }
        } else {
            // "incus:gabbro at -3992 -1188 137": a whole block (octants and objects say more, and aren't read).
            let Some((block, cell)) = what.split_once(" at ") else { continue };
            if block.contains(' ') {
                continue;
            }
            let Some(cell) = three(cell) else { continue };
            Change::Block { cell, block: doorway.block(block) }
        };
        match doorway.make(change) {
            Ok(()) if matches!(change, Change::Bar { .. }) => made.bars += 1,
            Ok(()) => made.blocks += 1,
            Err(_) => made.refused += 1,
        }
    }
    made
}

/// A mark's scatter at one setting of the dial: `count` random whole blocks, each in a cell of its own, made as changes
/// through the doorway. The same blocks every time for the same seed and count.
pub fn scatter(doorway: &mut Doorway, mark: &Mark, scatter: &Scatter, count: u32) -> Result<Made, String> {
    let middle = [mark.at[0].floor() as i32, mark.at[1].floor() as i32];
    let side = 2 * scatter.around;
    // The surface under each column round the mark, its tiles read once each.
    let mut tiles = std::collections::HashMap::new();
    let mut surfaces = vec![0; (side * side) as usize];
    for i in 0..side {
        for j in 0..side {
            let (north, east) = (middle[0] - scatter.around + i, middle[1] - scatter.around + j);
            let (tile, [ti, tj]) = doorway.place(north, east);
            if let std::collections::hash_map::Entry::Vacant(vacant) = tiles.entry(tile) {
                vacant.insert(doorway.tile(tile[0], tile[1])?);
            }
            let column = tiles[&tile].column(doorway, ti, tj)?;
            surfaces[(i + j * side) as usize] = doorway.surface(&column).ok_or_else(|| format!("{}'s scatter reaches past the map", mark.name))?;
        }
    }
    if count as usize > surfaces.len() * scatter.high as usize {
        return Err(format!("{} blocks don't fit in {}'s scatter", count, mark.name));
    }
    let blocks: Vec<_> = scatter.blocks.iter().map(|name| doorway.block(name)).collect();
    // A small, fixed random sequence (xorshift), so the same seed always scatters the same blocks.
    let mut state = scatter.seed.max(1);
    let mut next = move |below: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % below
    };
    let mut taken = std::collections::HashSet::new();
    let mut made = Made::default();
    while taken.len() < count as usize {
        let (i, j) = (next(side as u64) as i32, next(side as u64) as i32);
        let z = surfaces[(i + j * side) as usize] + next(scatter.high as u64) as i32;
        let cell = [middle[0] - scatter.around + i, middle[1] - scatter.around + j, z];
        let block = blocks[next(blocks.len() as u64) as usize];
        if !taken.insert(cell) {
            continue;
        }
        match doorway.make(Change::Block { cell, block }) {
            Ok(()) => made.blocks += 1,
            Err(_) => made.refused += 1,
        }
    }
    Ok(made)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pass mark (plan 2.1, the marks as data): each eye where old's stood, to a tenth of a block, facing as old's;
    /// the build all 169 placings, none refused; each scatter setting its count.
    #[test]
    fn the_marks_stand_where_old_s_did() {
        let marks = load().unwrap();
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        // Old's, from its reports (kept logs/dyno-1794448108.txt, kept logs/dyno-1794433359.txt): the eye and the facing.
        let old: [(&str, [f32; 3], f32); 7] = [
            ("on the ground at the spawn", [0.5, 0.5, 395.4], 0.0),
            ("the mountain top on a clear day", [-605.5, 78.5, 1025.4], 172.6),
            ("the bare plain", [-3975.0, -1192.0, 137.4], 180.0),
            ("the plain with David's build", [-3975.0, -1192.0, 137.4], 180.0),
            ("the scatter", [-3975.0, -1192.0, 137.4], 180.0),
            ("high over the spawn", [0.5, 0.5, 795.4], 0.0),
            ("low over the sea", [-1905.7, -2137.7, 151.4], 228.3),
        ];
        // Then the window's two moving marks (plan 2.2, step 4: new, not old's), each with its speed.
        assert_eq!(marks.len(), old.len() + 4);
        assert_eq!(marks[old.len()..].iter().map(|m| (m.name.as_str(), m.speed)).collect::<Vec<_>>(), [("a player's run over the plain", Some(200.0)), ("fast across the world", Some(2000.0)), ("the plain, nodding", None), ("the sea, nodding", None)]);
        assert!(marks[old.len() + 2..].iter().all(|m| m.nod.is_some()), "the nodding marks nod");
        assert!(marks[..old.len()].iter().all(|m| m.speed.is_none()), "old's marks stand still");
        for (mark, (name, at, facing)) in marks.iter().zip(old) {
            assert_eq!(mark.name, name);
            let eye = eye(&doorway, mark).unwrap();
            for (axis, (got, wanted)) in eye.to_array().iter().zip(at).enumerate() {
                assert!((got - wanted).abs() < 0.05, "{name}: the eye's {} is {got}, old's {wanted}", ["north", "east", "up"][axis]);
            }
            assert!((mark.facing - facing).abs() < 0.05, "{name}: facing {}, old's {facing}", mark.facing);
        }
        let build = marks.iter().find(|mark| mark.changes.is_some()).unwrap();
        let log = std::fs::read_to_string(crate::session::project_folder().join(build.changes.as_ref().unwrap())).unwrap();
        assert_eq!(place(&mut doorway, &log), Made { blocks: 23, bars: 146, refused: 0 });
        // Each change kept with the tile its cell is in: the build lies in one or two tiles, all 169 there.
        let mut tiles: Vec<[i32; 2]> = Vec::new();
        for change in std::fs::read_to_string(crate::session::project_folder().join(build.changes.as_ref().unwrap())).unwrap().lines().filter(|line| line.contains("edit request")).filter_map(|line| line.rsplit_once(" at ")) {
            let numbers: Vec<i32> = change.1.split_whitespace().filter_map(|word| word.parse().ok()).collect();
            let tile = doorway.place(numbers[0], numbers[1]).0;
            if !tiles.contains(&tile) {
                tiles.push(tile);
            }
        }
        assert_eq!(tiles.iter().map(|tile| doorway.changes_in(tile[0], tile[1]).len()).sum::<usize>(), 169);
        doorway.forget_changes();
        assert!(tiles.iter().all(|tile| doorway.changes_in(tile[0], tile[1]).is_empty()));
        let scattered = marks.iter().find(|mark| mark.scatter.is_some()).unwrap();
        let dial = scattered.scatter.as_ref().unwrap();
        for &count in &dial.counts {
            doorway.forget_changes();
            assert_eq!(scatter(&mut doorway, scattered, dial, count).unwrap(), Made { blocks: count as usize, bars: 0, refused: 0 });
        }
    }
}

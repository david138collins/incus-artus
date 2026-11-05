//! The test world [`docs/far-view.md`, the path to finish the renderer; David, 2026-11-02: "i think
//! your test world makes sense"]: fixed places round where you stand, to judge every change to the
//! renderer at the same spots, each with a viewpoint to go to, so before-and-after pictures match.
//!
//! **The places,** on a ring round the spot it's built from (each about 180 blocks off, the way
//! the compass says):
//! - **north, a forest:** about 900 stand-in trees (a trunk and a ball of leaves) over 180 × 180
//!   blocks, for sheer amount and fine, sparse detail far off;
//! - **north-east, a village:** 12 block houses (walls, a door, windows, a stepped roof) on
//!   levelled footings;
//! - **south-east, water:** a lake dug and filled, and a river running from it;
//! - **south, a glass wall:** 24 long and 10 high, of the see-through outline block, on a stone sill;
//! - **south-west, a staircase of half blocks:** 4 wide, 24 half steps up;
//! - **north-west, caves with braziers:** three tunnels dug into the steepest slope near there, each
//!   with a room at its end, a brazier in the room and one halfway in.
//!
//! The blocks are edits like any others (in loaded chunks, or in the chunk as kept, kept changed), so
//! saving the world keeps them. Where it was built is kept in `TestPlaces` (saved in presets), for
//! the viewpoints. Its blocks are the game's own where David's aren't there: the trunks dirt (else
//! gabbro), the leaves malachite (else grass), the walls stone brick (else stone), the roofs gravel
//! (else gabbro), the windows and the glass the outline block. Building it is world creation, so
//! fractions are fine here.

use super::position::ChunkPos;
use super::terrain::TerrainBlocks;
use super::{BlockId, Caves, CompiledMap, Part, Pieces, PlacedObject, Streaming, Terrain, VoxelWorld};
use crate::data::BlockRegistry;
use crate::ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Where the test world was built (game axes), if it has been: its places and viewpoints are worked
/// out from it.
#[derive(Resource, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TestPlaces {
    pub centre: Option<[f32; 2]>,
}

/// The places: their names, the compass bearing they're at from the centre (degrees, 0 north, 90
/// east) and how far (blocks).
pub const PLACES: [(&str, f32, f32); 6] = [("forest", 0.0, 200.0), ("village", 60.0, 180.0), ("water", 120.0, 180.0), ("glass wall", 180.0, 170.0), ("half-block stairs", 240.0, 170.0), ("caves", 300.0, 180.0)];

/// A place's middle, from the centre.
pub fn place_at(centre: Vec2, bearing: f32, distance: f32) -> Vec2 {
    let angle = bearing.to_radians();
    centre + Vec2::new(angle.cos(), angle.sin()) * distance
}

/// Where a place is looked at from: back toward the centre and up, facing it (position, heading,
/// pitch); the forest from higher, to see over it. `None` past the map.
pub fn viewpoint(map: &CompiledMap, centre: Vec2, place: usize) -> Option<(Vec3, f32, f32)> {
    let (name, bearing, distance) = PLACES[place];
    let target = place_at(centre, bearing, distance);
    let back = if name == "forest" { 110.0 } else { 70.0 };
    let from = place_at(centre, bearing, distance - back);
    let ground = map.column(from.x.floor() as i32, from.y.floor() as i32)?.0 as f32;
    let up = if name == "forest" { 70.0 } else { 25.0 };
    let toward = target - from;
    Some((from.extend(ground + up), toward.y.atan2(toward.x).to_degrees().rem_euclid(360.0), -18.0))
}

/// The whole test world seen from afar: 1,200 blocks south of the centre, 250 up, facing north.
pub fn far_viewpoint(map: &CompiledMap, centre: Vec2) -> Option<(Vec3, f32, f32)> {
    let from = centre - Vec2::X * 1_200.0;
    let ground = map.column(from.x.floor() as i32, from.y.floor() as i32)?.0 as f32;
    Some((from.extend(ground.max(crate::world::SEA_LEVEL as f32) + 250.0), 0.0, -10.0))
}

/// What's to be changed: blocks, cells of half blocks, and objects.
#[derive(Default)]
struct Edits {
    blocks: Vec<(IVec3, BlockId)>,
    pieces: Vec<(IVec3, Pieces)>,
    objects: Vec<PlacedObject>,
}

/// A small hash of a spot, for the trees' scatter and sizes.
fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(73_856_093) ^ (y as u32).wrapping_mul(19_349_663) ^ salt.wrapping_mul(83_492_791);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

/// The ground's top block at a column (`None` past the map, or under water).
fn ground(map: &CompiledMap, x: i32, y: i32) -> Option<i32> {
    match map.column(x, y) {
        Some((z, None)) if !super::under_sea(z as f32) => Some(z),
        _ => None,
    }
}

/// The blocks the test world is built of (see the module notes).
struct Materials {
    stone: BlockId,
    trunk: BlockId,
    leaves: BlockId,
    walls: BlockId,
    roof: BlockId,
    glass: BlockId,
    water: BlockId,
}

impl Materials {
    fn from(registry: &BlockRegistry, blocks: &TerrainBlocks) -> Self {
        let pick = |names: &[&str], fallback: BlockId| names.iter().find_map(|name| registry.id(name)).unwrap_or(fallback);
        Self {
            stone: blocks.stone,
            trunk: pick(&["incus:dirt1"], blocks.gabbro),
            leaves: pick(&["incus:malachite"], blocks.grass),
            walls: pick(&["incus:stone_brick"], blocks.stone),
            roof: pick(&["incus:gravel"], blocks.gabbro),
            glass: pick(&["incus:outline"], blocks.stone),
            water: blocks.water,
        }
    }
}

/// The forest: a tree every 6 blocks or so (scattered by up to 2), on land, over 180 × 180.
fn forest(map: &CompiledMap, at: Vec2, m: &Materials, edits: &mut Edits) -> usize {
    let mut trees = 0;
    for gy in -15..15 {
        for gx in -15..15 {
            let jitter = |salt: u32| (hash(gx, gy, salt) % 5) as i32 - 2;
            let (x, y) = (at.x as i32 + gx * 6 + jitter(1), at.y as i32 + gy * 6 + jitter(2));
            let Some(g) = ground(map, x, y) else { continue };
            let tall = 4 + (hash(gx, gy, 3) % 4) as i32;
            for z in g + 1..=g + tall {
                edits.blocks.push((IVec3::new(x, y, z), m.trunk));
            }
            let top = g + tall;
            for dz in -1..=2 {
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        if dx * dx + dy * dy + dz * dz <= 5 && !(dx == 0 && dy == 0 && dz <= 0) {
                            edits.blocks.push((IVec3::new(x + dx, y + dy, top + dz), m.leaves));
                        }
                    }
                }
            }
            trees += 1;
        }
    }
    trees
}

/// The village: 3 × 4 houses, 14 blocks apart.
fn village(map: &CompiledMap, at: Vec2, m: &Materials, edits: &mut Edits) -> usize {
    let mut houses = 0;
    for row in 0..3 {
        for column in 0..4 {
            let corner = IVec2::new(at.x as i32 - 24 + row * 16, at.y as i32 - 30 + column * 16);
            let Some(floor) = ground(map, corner.x + 3, corner.y + 3) else { continue };
            for dy in -1..8 {
                for dx in -1..8 {
                    let (x, y) = (corner.x + dx, corner.y + dy);
                    let below = ground(map, x, y).unwrap_or(floor);
                    // A footing down to the ground, and the ground cleared above the floor.
                    for z in below.min(floor - 1)..=floor {
                        edits.blocks.push((IVec3::new(x, y, z), m.stone));
                    }
                    for z in floor + 1..=below.max(floor + 10) {
                        edits.blocks.push((IVec3::new(x, y, z), BlockId::AIR));
                    }
                    if !(0..7).contains(&dx) || !(0..7).contains(&dy) {
                        continue;
                    }
                    let wall = dx == 0 || dx == 6 || dy == 0 || dy == 6;
                    for level in 1..=4 {
                        let z = floor + level;
                        let door = dy == 0 && dx == 3 && level <= 2;
                        let window = wall && level == 2 && (dx == 3 || dy == 3) && !(dy == 0 && dx == 3);
                        if wall && !door {
                            edits.blocks.push((IVec3::new(x, y, z), if window { m.glass } else { m.walls }));
                        }
                    }
                    // The roof: stepped in a block each level.
                    for step in 0..4 {
                        if (step..7 - step).contains(&dx) && (step..7 - step).contains(&dy) {
                            edits.blocks.push((IVec3::new(x, y, floor + 5 + step), m.roof));
                        }
                    }
                }
            }
            houses += 1;
        }
    }
    houses
}

/// The water: a lake 16 blocks across its middle, 4 deep, and a river 3 wide running 80 blocks on
/// from it, away from the centre.
fn water(map: &CompiledMap, at: Vec2, away: Vec2, m: &Materials, edits: &mut Edits) -> bool {
    let Some(level) = ground(map, at.x as i32, at.y as i32) else { return false };
    for dy in -16..=16 {
        for dx in -16..=16 {
            if dx * dx + dy * dy > 16 * 16 {
                continue;
            }
            let (x, y) = (at.x as i32 + dx, at.y as i32 + dy);
            let top = ground(map, x, y).unwrap_or(level);
            for z in level - 4..=level {
                edits.blocks.push((IVec3::new(x, y, z), m.water));
            }
            for z in level + 1..=top {
                edits.blocks.push((IVec3::new(x, y, z), BlockId::AIR));
            }
        }
    }
    let across = Vec2::new(-away.y, away.x);
    for step in 14..94 {
        for side in -1..=1 {
            let spot = at + away * step as f32 + across * side as f32;
            let (x, y) = (spot.x.floor() as i32, spot.y.floor() as i32);
            let Some(top) = ground(map, x, y) else { continue };
            edits.blocks.push((IVec3::new(x, y, top), m.water));
            edits.blocks.push((IVec3::new(x, y, top - 1), m.water));
        }
    }
    true
}

/// The glass wall: 24 long across the way from the centre, 10 high, on a stone sill.
fn glass_wall(map: &CompiledMap, at: Vec2, away: Vec2, m: &Materials, edits: &mut Edits) -> bool {
    let Some(base) = ground(map, at.x as i32, at.y as i32) else { return false };
    let across = Vec2::new(-away.y, away.x);
    for along in -12..12 {
        let spot = at + across * along as f32;
        let (x, y) = (spot.x.floor() as i32, spot.y.floor() as i32);
        let below = ground(map, x, y).unwrap_or(base);
        for z in below.min(base)..=base {
            edits.blocks.push((IVec3::new(x, y, z), m.stone));
        }
        for z in base + 1..=base + 10 {
            edits.blocks.push((IVec3::new(x, y, z), m.glass));
        }
    }
    true
}

/// The staircase: 4 wide, 24 half steps up away from the centre, on a stone footing.
fn stairs(map: &CompiledMap, at: Vec2, away: Vec2, m: &Materials, edits: &mut Edits) -> bool {
    let Some(base) = ground(map, at.x as i32, at.y as i32) else { return false };
    // Straight along the nearer of north-south and east-west, so each step is a row of whole cells.
    let way = if away.x.abs() >= away.y.abs() { IVec2::new(away.x.signum() as i32, 0) } else { IVec2::new(0, away.y.signum() as i32) };
    let across = IVec2::new(-way.y, way.x);
    let start = at.floor().as_ivec2();
    let lower_half = {
        let mut pieces = Pieces::default();
        for octant in 0..4 {
            pieces.set(octant, m.stone, Part::LONE);
        }
        pieces
    };
    for step in 0..24 {
        // In half blocks above the footing.
        let height = step + 1;
        for side in -2..2 {
            let column = start + way * step + across * side;
            let below = ground(map, column.x, column.y).unwrap_or(base);
            // The footing and the whole blocks of the step.
            for z in below.min(base)..=base + height / 2 {
                edits.blocks.push((column.extend(z), m.stone));
            }
            for z in base + 1 + height / 2..=below.max(base + 14) {
                edits.blocks.push((column.extend(z), BlockId::AIR));
            }
            if height % 2 == 1 {
                edits.pieces.push((column.extend(base + 1 + height / 2), lower_half.clone()));
            }
        }
    }
    true
}

/// The caves: three tunnels dug into the steepest slope near `at`, 40 blocks apart across the way
/// from the centre, each 3 wide and 3 high, 14 long, with a room 7 × 7 and 5 high at its end, a
/// brazier in the room and one halfway in.
fn caves(map: &CompiledMap, at: Vec2, away: Vec2, edits: &mut Edits) -> usize {
    let across = Vec2::new(-away.y, away.x);
    let mut dug = 0;
    for k in -1..=1 {
        let near = at + across * (40 * k) as f32;
        // The steepest land within 24 blocks, and which way is uphill there.
        let mut best: Option<(f32, IVec2, Vec2)> = None;
        for dy in (-24..=24).step_by(2) {
            for dx in (-24..=24).step_by(2) {
                let (x, y) = (near.x as i32 + dx, near.y as i32 + dy);
                let h = |x: i32, y: i32| ground(map, x, y).map(|z| z as f32);
                let (Some(n), Some(s), Some(e), Some(w)) = (h(x + 3, y), h(x - 3, y), h(x, y + 3), h(x, y - 3)) else { continue };
                let uphill = Vec2::new(n - s, e - w) / 6.0;
                if best.is_none_or(|(steep, ..)| uphill.length() > steep) {
                    best = Some((uphill.length(), IVec2::new(x, y), uphill.normalize_or(away)));
                }
            }
        }
        let Some((_, spot, uphill)) = best else { continue };
        let Some(floor) = ground(map, spot.x, spot.y) else { continue };
        let side = Vec2::new(-uphill.y, uphill.x);
        let dig = |along: f32, wide: i32, high: i32, edits: &mut Edits| {
            for w in -wide..=wide {
                let p = spot.as_vec2() + Vec2::splat(0.5) + uphill * along + side * w as f32;
                for z in floor + 1..=floor + high {
                    edits.blocks.push((IVec3::new(p.x.floor() as i32, p.y.floor() as i32, z), BlockId::AIR));
                }
            }
        };
        for along in -2..14 {
            dig(along as f32, 1, 3, edits);
        }
        for along in 14..21 {
            dig(along as f32, 3, 5, edits);
        }
        let brazier = |along: f32| {
            let p = spot.as_vec2() + Vec2::splat(0.5) + uphill * along;
            PlacedObject { name: "incus:braziere".to_string(), at: IVec3::new((p.x * 16.0).floor() as i32, (p.y * 16.0).floor() as i32, (floor + 1) * 16), facing: 0, stage: 0, since: 0 }
        };
        edits.objects.push(brazier(17.0));
        edits.objects.push(brazier(7.0));
        dug += 1;
    }
    dug
}

/// Builds the test world round the camera (see the module notes), and keeps where. A note of what
/// was built.
pub(super) fn build_test_world(world: &mut World) -> String {
    let Some(map) = world.resource::<Terrain>().compiled().cloned() else { return "The test world needs a compiled world.".to_string() };
    let Some(from) = world.query::<&crate::camera::FlyCamera>().iter(world).next().map(|camera| camera.position.truncate()) else { return String::new() };
    let registry = world.resource::<BlockRegistry>();
    let blocks = TerrainBlocks::from_registry(registry);
    let m = Materials::from(registry, &blocks);
    let has_braziers = world.resource::<crate::data::ObjectRegistry>().by_name("incus:braziere").is_some();
    let mut edits = Edits::default();
    let mut lines = Vec::new();
    for (k, &(name, bearing, distance)) in PLACES.iter().enumerate() {
        let at = place_at(from, bearing, distance);
        let away = (at - from).normalize_or(Vec2::X);
        let what = match k {
            0 => format!("{} trees", forest(&map, at, &m, &mut edits)),
            1 => format!("{} houses", village(&map, at, &m, &mut edits)),
            2 => if water(&map, at, away, &m, &mut edits) { "a lake and a river".to_string() } else { "nothing (under water there)".to_string() },
            3 => if glass_wall(&map, at, away, &m, &mut edits) { "the wall".to_string() } else { "nothing (under water there)".to_string() },
            4 => if stairs(&map, at, away, &m, &mut edits) { "the stairs".to_string() } else { "nothing (under water there)".to_string() },
            _ => format!("{} caves", caves(&map, at, away, &mut edits)),
        };
        lines.push(format!("{name} ({what})"));
    }
    if !has_braziers {
        edits.objects.clear();
        lines.push("no braziers: the brazier isn't among the objects".to_string());
    }

    // Set in loaded chunks; otherwise in the chunk as it's kept (or made), kept changed.
    let plan = world.resource::<Caves>().0.clone();
    let mut by_chunk: HashMap<ChunkPos, Edits> = HashMap::new();
    for (block, id) in edits.blocks {
        by_chunk.entry(ChunkPos::of_block(block)).or_default().blocks.push((block, id));
    }
    for (block, pieces) in edits.pieces {
        by_chunk.entry(ChunkPos::of_block(block)).or_default().pieces.push((block, pieces));
    }
    for object in edits.objects {
        by_chunk.entry(ChunkPos::of_block(object.cell())).or_default().objects.push(object);
    }
    let mut voxel = world.resource_mut::<VoxelWorld>();
    for (&pos, edits) in &mut by_chunk {
        if voxel.chunk(pos).is_some() {
            for &(block, id) in &edits.blocks {
                voxel.set_block(block, id);
            }
            for (block, pieces) in edits.pieces.drain(..) {
                voxel.set_pieces(block, pieces);
            }
            for object in edits.objects.drain(..) {
                voxel.place_object(object);
            }
        } else if let Some(mut chunk) = voxel.take_put_away(pos).or_else(|| map.chunk(pos, &blocks, &plan)) {
            for &(block, id) in &edits.blocks {
                chunk.set(block, id);
            }
            for (block, pieces) in edits.pieces.drain(..) {
                chunk.set_pieces(block, pieces);
            }
            for object in edits.objects.drain(..) {
                chunk.add_object(object);
            }
            voxel.keep_changed(chunk);
        }
    }
    let places: HashSet<ChunkPos> = by_chunk.keys().copied().collect();
    world.resource_mut::<Streaming>().forget(&places);
    world.resource_mut::<TestPlaces>().centre = Some(from.to_array());
    let note = format!(
        "The test world round ({:.0}, {:.0}): {}. Go to each place with the buttons below; save the world to keep it (and the presets, for the buttons).",
        from.x,
        from.y,
        lines.join(", ")
    );
    info!("{note}");
    note
}

/// Puts the camera at a viewpoint, flying (walking, the body would pull it back).
pub(super) fn go_to(world: &mut World, (position, heading, pitch): (Vec3, f32, f32)) {
    *world.resource_mut::<crate::player::ControlMode>() = crate::player::ControlMode::Fly;
    if let Some(mut camera) = world.query::<&mut crate::camera::FlyCamera>().iter_mut(world).next() {
        camera.position = position;
        camera.heading = heading;
        camera.pitch = pitch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_places_go_round_the_compass_and_are_looked_at_from_toward_the_centre() {
        let centre = Vec2::new(1_000.0, -2_000.0);
        // North is +x, east +y.
        assert!((place_at(centre, 0.0, 200.0) - Vec2::new(1_200.0, -2_000.0)).length() < 1e-3);
        assert!((place_at(centre, 90.0, 100.0) - Vec2::new(1_000.0, -1_900.0)).length() < 1e-3);
        let names: HashSet<&str> = PLACES.iter().map(|(name, ..)| *name).collect();
        assert_eq!(names.len(), PLACES.len());
    }

    #[test]
    fn half_steps_rise_half_a_block_each_and_the_village_stands_on_footings() {
        let width = 64;
        let map = CompiledMap::of_heights(width, 32.0, &vec![200.0; width * width]);
        let registry = BlockRegistry::default();
        let blocks = TerrainBlocks::from_registry(&registry);
        let m = Materials::from(&registry, &blocks);
        let mut edits = Edits::default();
        assert!(stairs(&map, Vec2::new(10.5, 10.5), Vec2::X, &m, &mut edits));
        // 24 steps, 4 wide: the odd ones end in a half block.
        assert_eq!(edits.pieces.len(), 12 * 4);
        let g = map.column(10, 10).unwrap().0;
        assert!(edits.pieces.iter().all(|(cell, pieces)| cell.z > g && (0..4).all(|o| pieces.get(o).is_some()) && (4..8).all(|o| pieces.get(o).is_none())));
        let mut edits = Edits::default();
        assert_eq!(village(&map, Vec2::new(0.0, 0.0), &m, &mut edits), 12);
        assert!(edits.blocks.iter().any(|&(_, id)| id == m.glass) && edits.blocks.iter().any(|&(_, id)| id == m.roof));
        let mut edits = Edits::default();
        assert!(forest(&map, Vec2::ZERO, &m, &mut edits) > 500);
    }
}

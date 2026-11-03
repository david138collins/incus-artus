//! The far test sites [`docs/caves.md` 13.9; David, 2026-11-02: "builder may proceed with the present
//! plan"]: things to look at from afar, in a line the way you're facing, at four distances, so the far
//! view can be checked against each, before and after the far copies.
//!
//! **The distances** are counted as the render distance counts them (the larger of north-south and
//! east-west): about 600 blocks (inside the real chunks), the last ring of real chunks (the edge),
//! and 500 and 1,500 past the edge (the far copies' band), whatever the render distance.
//!
//! **At each distance:**
//! - **a mouth in a cliff:** the test cave, worming into the steepest slope near the line;
//! - **a mouth on a chunk's edge:** the same, started a block from a chunk's side;
//! - **a gorge with a pillar:** a 1 × 1 pillar 20 tall at the bottom of the deepest narrow valley
//!   near the line, where the far surface may bridge the valley;
//! - **a thin wall:** 1 block thick, 20 long and 10 high, side on to you, beside a 1 × 1 pillar;
//! - **a pit:** 6 × 6, dug 20 deep, seen from above.
//!
//! The caves go into the cave plan (saved in `caves.toml`); the blocks are edits, in chunks made
//! for the purpose if they aren't loaded (kept as changed chunks, so they're saved with the world).
//! Finding the spots is world creation, so fractions are fine here.

use super::caves::CavePlan;
use super::position::{CHUNK_SIZE, ChunkPos};
use super::terrain::TerrainBlocks;
use super::{Caves, CompiledMap, Streaming, Terrain, TestCaveSettings, VoxelWorld, under_sea};
use crate::data::BlockRegistry;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// How far from the line each site is looked for, and how far apart they're put (blocks).
const CLIFF_SEARCH: i32 = 96;
const GORGE_SEARCH: i32 = 256;
const BESIDE: f32 = 48.0;

/// The middle of the last real chunk along `dir` (a direction on the ground) from `from`: the
/// furthest the line stays in the chunks the render distance (`chunks`) and the streaming circle
/// (`radius`) leave real (`streaming::real_chunk`).
pub(super) fn edge_distance(from: Vec2, dir: Vec2, chunks: i32, radius: i32) -> f32 {
    let home = (from / CHUNK_SIZE as f32).floor().as_ivec2();
    let chunk = |t: f32| ((from + dir * t) / CHUNK_SIZE as f32).floor().as_ivec2() - home;
    let steps = (chunks.min(radius) + 2) * CHUNK_SIZE * 2;
    let Some(last) = (0..steps).rev().map(|t| t as f32).find(|&t| super::real_chunk(chunk(t), chunks, radius)) else {
        return (chunks.min(radius) * CHUNK_SIZE) as f32;
    };
    let first = (0..steps).map(|t| t as f32).find(|&t| chunk(t) == chunk(last)).unwrap_or(last);
    (first + last) / 2.0
}

/// The ground's top block at a column, if it's land (not under the sea or a river).
fn land(map: &CompiledMap, x: i32, y: i32) -> Option<i32> {
    match map.column(x, y) {
        Some((z, None)) if !under_sea(z as f32) => Some(z),
        _ => None,
    }
}

/// The steepest land within `CLIFF_SEARCH` of `at` (every 2 blocks), and which way is uphill there,
/// among the columns `allowed` says yes to.
fn steepest(map: &CompiledMap, at: Vec2, allowed: impl Fn(i32, i32) -> bool) -> Option<(IVec2, Vec2)> {
    let centre = at.floor().as_ivec2();
    let mut best: Option<(f32, IVec2, Vec2)> = None;
    for dx in (-CLIFF_SEARCH..=CLIFF_SEARCH).step_by(2) {
        for dy in (-CLIFF_SEARCH..=CLIFF_SEARCH).step_by(2) {
            let (x, y) = (centre.x + dx, centre.y + dy);
            if !allowed(x, y) || land(map, x, y).is_none() {
                continue;
            }
            let h = |x: i32, y: i32| map.column(x, y).map(|(z, _)| z as f32);
            let (Some(n), Some(s), Some(e), Some(w)) = (h(x + 3, y), h(x - 3, y), h(x, y + 3), h(x, y - 3)) else { continue };
            let uphill = Vec2::new(n - s, e - w) / 6.0;
            if best.is_none_or(|(steep, ..)| uphill.length() > steep) {
                best = Some((uphill.length(), IVec2::new(x, y), uphill.normalize_or(Vec2::X)));
            }
        }
    }
    best.filter(|(steep, ..)| *steep > 0.3).map(|(_, spot, uphill)| (spot, uphill))
}

/// A test cave worming uphill into the slope at `spot`, its mouth just above the ground.
fn mouth_at(map: &CompiledMap, spot: IVec2, uphill: Vec2, settings: &TestCaveSettings, seed: u32) -> super::caves::Tunnel {
    let ground = map.column(spot.x, spot.y).map_or(0, |(z, _)| z) as f32;
    let middle = Vec3::new(spot.x as f32 + 0.5 - uphill.x, spot.y as f32 + 0.5 - uphill.y, ground + 1.0 + settings.height * 0.6);
    settings.tunnel(middle, uphill, seed)
}

/// Adds caves to the plan: the chunks they come near are made again with them, and changed chunks
/// put away there are carved too. Returns those chunks.
fn add_caves(world: &mut World, map: &CompiledMap, tunnels: Vec<super::caves::Tunnel>, blocks: &TerrainBlocks) -> HashSet<ChunkPos> {
    let old = world.resource::<Caves>().0.clone();
    let places: HashSet<ChunkPos> = tunnels.iter().flat_map(CavePlan::chunks_near).collect();
    let plan = std::sync::Arc::new(CavePlan::new(old.tunnels.iter().cloned().chain(tunnels).collect()));
    world.insert_resource(Caves(plan.clone()));
    super::remake_chunks(world, map, &places, &plan, None);
    let mut voxel = world.resource_mut::<VoxelWorld>();
    for &pos in &places {
        if let Some(mut chunk) = voxel.take_put_away(pos) {
            super::caves::carve_chunk(&mut chunk, &plan, blocks.water);
            voxel.keep_changed(chunk);
        }
    }
    places
}

/// Cave mouths in hillsides round you, for measuring what the far copies cost [`docs/caves.md`
/// 13.9; David, 2026-11-02: "thats a good idea"]: `mouths` of them, each the test cave into the
/// steepest slope near a spot picked at random within `mouths_reach` (evenly over the area), at least
/// 64 blocks apart. The same from the same spot.
pub(super) fn place_hillside_mouths(world: &mut World) -> String {
    let Some(map) = world.resource::<Terrain>().compiled().cloned() else { return "Needs a compiled world.".to_string() };
    let Some(from) = world.query::<&crate::camera::FlyCamera>().iter(world).next().map(|camera| camera.position.truncate()) else { return String::new() };
    let settings = world.resource::<TestCaveSettings>().clone();
    let blocks = TerrainBlocks::from_registry(world.resource::<BlockRegistry>());
    let mut random = (from.x as i32).wrapping_mul(73_856_093) as u32 ^ (from.y as i32).wrapping_mul(19_349_663) as u32 | 1;
    let mut next = move || {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        random as f32 / u32::MAX as f32
    };
    let started = std::time::Instant::now();
    let (mut spots, mut tunnels): (Vec<IVec2>, Vec<_>) = (Vec::new(), Vec::new());
    for k in 0..settings.mouths.max(0) * 4 {
        if tunnels.len() >= settings.mouths.max(0) as usize {
            break;
        }
        let (angle, distance) = (next() * std::f32::consts::TAU, next().sqrt() * settings.mouths_reach);
        let at = from + Vec2::new(angle.cos(), angle.sin()) * distance;
        if let Some((spot, uphill)) = steepest(&map, at, |x, y| spots.iter().all(|&other| (IVec2::new(x, y) - other).length_squared() > 64 * 64)) {
            tunnels.push(mouth_at(&map, spot, uphill, &settings, random_seed(k)));
            spots.push(spot);
        }
    }
    let count = tunnels.len();
    let mut places = add_caves(world, &map, tunnels, &blocks);
    places.extend(spots.iter().map(|spot| ChunkPos::of_block(spot.extend(0))));
    world.resource_mut::<Streaming>().forget(&places);
    // How many are where the far copies draw them (past the real chunks), for the measurement.
    let radius = world.resource::<super::StreamSettings>().radius.min(world.resource::<crate::render::RenderSettings>().march_from) * CHUNK_SIZE;
    let far = spots.iter().filter(|spot| spot.as_vec2().distance(from) > radius as f32).count();
    let note = format!(
        "{count} cave mouths in hillsides within {:.0} blocks of ({:.0}, {:.0}), {far} of them past the real chunks (the far copies draw those); found in {:.0} ms. Save the world to keep them.",
        settings.mouths_reach,
        from.x,
        from.y,
        started.elapsed().as_secs_f64() * 1000.0
    );
    info!("{note}");
    note
}

/// A tunnel's seed from its number.
fn random_seed(k: i32) -> u32 {
    (k as u32).wrapping_mul(2_654_435_761) ^ 0x9e37_79b9
}

/// The bottom of the deepest narrow valley within `GORGE_SEARCH` of `at` (every 4 blocks): the
/// column most below the ground 16 blocks either side of it, across any of four directions.
fn gorge(map: &CompiledMap, at: Vec2) -> Option<IVec2> {
    let centre = at.floor().as_ivec2();
    let mut best: Option<(i32, IVec2)> = None;
    for dx in (-GORGE_SEARCH..=GORGE_SEARCH).step_by(4) {
        for dy in (-GORGE_SEARCH..=GORGE_SEARCH).step_by(4) {
            let (x, y) = (centre.x + dx, centre.y + dy);
            let Some((z, _)) = map.column(x, y) else { continue };
            if under_sea(z as f32) {
                continue;
            }
            let side = |sx: i32, sy: i32| map.column(x + sx, y + sy).map_or(z, |(z, _)| z);
            let depth = [(16, 0), (0, 16), (11, 11), (11, -11)].iter().map(|&(sx, sy)| side(sx, sy).min(side(-sx, -sy)) - z).max().unwrap_or(0);
            if best.is_none_or(|(deepest, _)| depth > deepest) {
                best = Some((depth, IVec2::new(x, y)));
            }
        }
    }
    best.filter(|(depth, _)| *depth >= 6).map(|(_, spot)| spot)
}

/// What the sites at one distance need: the caves, and the blocks to set. And what was put where.
#[derive(Default)]
pub(super) struct Sites {
    pub caves: Vec<super::caves::Tunnel>,
    pub blocks: Vec<(IVec3, bool)>,
    pub placed: Vec<String>,
}

/// The sites at `at` (on the line going `dir`), with the test cave's settings.
pub(super) fn sites_at(map: &CompiledMap, at: Vec2, dir: Vec2, settings: &TestCaveSettings, seed: u32) -> Sites {
    let mut sites = Sites::default();
    let side = Vec2::new(-dir.y, dir.x);
    let place = |spot: IVec2| format!("({}, {}, {})", spot.x, spot.y, map.column(spot.x, spot.y).map_or(0, |(z, _)| z));
    // A mouth in a cliff, and one a block from a chunk's side (away from the first).
    let mouth = |spot: IVec2, uphill: Vec2, seed: u32| mouth_at(map, spot, uphill, settings, seed);
    let cliff = steepest(map, at, |_, _| true);
    if let Some((spot, uphill)) = cliff {
        sites.caves.push(mouth(spot, uphill, seed));
        sites.placed.push(format!("cliff mouth {}", place(spot)));
    }
    let on_edge = |v: i32| matches!(v.rem_euclid(CHUNK_SIZE), 1 | 30);
    let away = |x: i32, y: i32| cliff.is_none_or(|(spot, _)| (IVec2::new(x, y) - spot).length_squared() > 40 * 40);
    if let Some((spot, uphill)) = steepest(map, at, |x, y| (on_edge(x) || on_edge(y)) && away(x, y)) {
        sites.caves.push(mouth(spot, uphill, seed ^ 0x5bd1_e995));
        sites.placed.push(format!("chunk-edge mouth {}", place(spot)));
    }
    // A pillar in a gorge: from the ground up through any water, 20 above its surface.
    if let Some(spot) = gorge(map, at)
        && let Some((ground, water)) = map.column(spot.x, spot.y)
    {
        let top = water.map_or(ground, |water| water - 1).max(ground) + 20;
        sites.blocks.extend((ground + 1..=top).map(|z| (spot.extend(z), true)));
        sites.placed.push(format!("gorge pillar {}", place(spot)));
    }
    // A thin wall, side on to you (along whichever of north or east is more across the line), and a
    // pillar beside its end.
    let wall_at = (at + side * BESIDE).floor().as_ivec2();
    let along = if dir.x.abs() > dir.y.abs() { IVec2::Y } else { IVec2::X };
    if land(map, wall_at.x, wall_at.y).is_some() {
        for k in 0..20 {
            let column = wall_at + along * k;
            if let Some(ground) = land(map, column.x, column.y) {
                sites.blocks.extend((ground + 1..=ground + 10).map(|z| (column.extend(z), true)));
            }
        }
        let pillar = wall_at + along * 24;
        if let Some(ground) = land(map, pillar.x, pillar.y) {
            sites.blocks.extend((ground + 1..=ground + 20).map(|z| (pillar.extend(z), true)));
        }
        sites.placed.push(format!("thin wall {}", place(wall_at)));
    }
    // A pit, 6 × 6, dug 20 deep (on land only, so no water pours in).
    let pit_at = (at - side * BESIDE).floor().as_ivec2();
    if (0..6).all(|x| (0..6).all(|y| land(map, pit_at.x + x, pit_at.y + y).is_some())) {
        for x in 0..6 {
            for y in 0..6 {
                let column = pit_at + IVec2::new(x, y);
                let ground = land(map, column.x, column.y).unwrap_or(0);
                sites.blocks.extend((ground - 19..=ground).map(|z| (column.extend(z), false)));
            }
        }
        sites.placed.push(format!("pit {}", place(pit_at)));
    }
    sites
}

/// Places the far test sites in a line the way you're facing. Says what was put where.
pub(super) fn place_far_test_sites(world: &mut World) -> String {
    let Some(map) = world.resource::<Terrain>().compiled().cloned() else { return "The far test sites need a compiled world.".to_string() };
    let Some((from, looking)) = world.query::<&crate::camera::FlyCamera>().iter(world).next().map(|camera| (camera.position, camera.looking())) else { return String::new() };
    let dir = looking.truncate().normalize_or(Vec2::X);
    let (chunks, march_from) = { let render = world.resource::<crate::render::RenderSettings>(); (render.render_distance_chunks, render.march_from) };
    let edge = edge_distance(from.truncate(), dir, chunks, world.resource::<super::StreamSettings>().radius.min(march_from));
    // Counted the larger of north-south and east-west, as the render distance is.
    let along = |d: f32| d / dir.x.abs().max(dir.y.abs());
    let settings = world.resource::<TestCaveSettings>().clone();
    let registry = world.resource::<BlockRegistry>();
    let blocks = TerrainBlocks::from_registry(registry);
    let Some(stone) = registry.id("incus:stone") else { return "No stone block registered.".to_string() };
    let seed = (from.x as i32).wrapping_mul(73_856_093) as u32 ^ (from.y as i32).wrapping_mul(19_349_663) as u32;
    let mut lines = vec![format!(
        "Far test sites from ({:.0}, {:.0}, {:.0}), heading {:.0}°. Stand there to see them as placed; save the world to keep them.",
        from.x,
        from.y,
        from.z,
        dir.y.atan2(dir.x).to_degrees().rem_euclid(360.0)
    )];
    let (mut tunnels, mut edits) = (Vec::new(), Vec::new());
    // Past the edge by 500 and 1,500: in the far copies' band, whatever the render distance.
    for (k, (name, distance)) in [("600", along(600.0)), ("the edge", edge), ("500 past the edge", edge + along(500.0)), ("1,500 past the edge", edge + along(1_500.0))].into_iter().enumerate() {
        let at = from.truncate() + dir * distance;
        let sites = sites_at(&map, at, dir, &settings, seed.wrapping_add(k as u32 * 7_919));
        lines.push(format!("At {name}: {}", if sites.placed.is_empty() { "nothing found (past the map, or under the sea)".to_string() } else { sites.placed.join(", ") }));
        tunnels.extend(sites.caves);
        edits.extend(sites.blocks);
    }

    // The caves first, so chunks made for the blocks are carved by them too.
    let mut places = add_caves(world, &map, tunnels, &blocks);
    let plan = world.resource::<Caves>().0.clone();
    let mut voxel = world.resource_mut::<VoxelWorld>();

    // The blocks: set in loaded chunks; otherwise in the chunk as it's kept (or made), kept changed.
    let mut by_chunk: HashMap<ChunkPos, Vec<(IVec3, bool)>> = HashMap::new();
    for (block, solid) in edits {
        by_chunk.entry(ChunkPos::of_block(block)).or_default().push((block, solid));
    }
    let id = |solid: bool| if solid { stone } else { super::BlockId::AIR };
    for (&pos, edits) in &by_chunk {
        if voxel.chunk(pos).is_some() {
            for &(block, solid) in edits {
                voxel.set_block(block, id(solid));
            }
        } else if let Some(mut chunk) = voxel.take_put_away(pos).or_else(|| map.chunk(pos, &blocks, &plan)) {
            for &(block, solid) in edits {
                chunk.set(block, id(solid));
            }
            voxel.keep_changed(chunk);
        }
    }
    // Chunks being made from the recipe at these places would undo it: they're made again.
    places.extend(by_chunk.keys());
    world.resource_mut::<Streaming>().forget(&places);
    let note = lines.join("\n");
    info!("{note}");
    note
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edge_site_is_in_the_last_ring_of_real_chunks_whichever_way_you_face() {
        for (from, dir) in [(Vec2::new(5.0, 7.0), Vec2::X), (Vec2::new(-40.0, 1_000.0), Vec2::new(0.6, -0.8)), (Vec2::new(31.0, -1.0), Vec2::new(-1.0, 1.0).normalize())] {
            let t = edge_distance(from, dir, 32, 64);
            let ring = ((from + dir * t) / 32.0).floor().as_ivec2() - (from / 32.0).floor().as_ivec2();
            assert_eq!(ring.x.abs().max(ring.y.abs()), 32, "{from} {dir}: {t}");
        }
        // With the streaming circle inside the render distance, the circle's edge.
        for (from, dir) in [(Vec2::new(5.0, 7.0), Vec2::X), (Vec2::new(31.0, -1.0), Vec2::new(-1.0, 1.0).normalize())] {
            let t = edge_distance(from, dir, 64, 32);
            let chunk = |t: f32| ((from + dir * t) / 32.0).floor().as_ivec2() - (from / 32.0).floor().as_ivec2();
            assert!(super::super::real_chunk(chunk(t), 64, 32) && !super::super::real_chunk(chunk(t + 64.0), 64, 32), "{from} {dir}: {t}");
        }
    }

    #[test]
    fn the_sites_find_the_slope_head_into_it_and_dig_and_build_where_they_should() {
        // Ground rising steeply one way (12 blocks a pixel of 32), from well above the sea.
        let width = 64;
        let heights: Vec<f32> = (0..width * width).map(|k| (230.0 + ((k / width) as f32 - 32.0) * 3.0 + ((k % width) as f32 - 32.0) * 12.0).clamp(140.0, 330.0)).collect();
        let map = CompiledMap::of_heights(width, 32.0, &heights);
        let settings = TestCaveSettings::default();
        let sites = sites_at(&map, Vec2::new(0.0, 0.0), Vec2::Y, &settings, 9);
        // Both mouths found, heading uphill, the second one a block from a chunk's side.
        assert_eq!(sites.caves.len(), 2, "{:?}", sites.placed);
        let ground = |at: [i32; 3]| map.column(at[0].div_euclid(16), at[1].div_euclid(16)).unwrap().0;
        for tunnel in &sites.caves {
            let (first, last) = (tunnel.points[0].at, tunnel.points.last().unwrap().at);
            assert!(ground(last) > ground(first) + 5, "heads uphill: {first:?} to {last:?}");
        }
        assert!(sites.placed[1].starts_with("chunk-edge mouth"));
        // A wall, a pillar and a pit: 200 wall blocks and 20 pillar blocks set, 720 dug.
        let (set, dug) = (sites.blocks.iter().filter(|(_, solid)| *solid).count(), sites.blocks.iter().filter(|(_, solid)| !*solid).count());
        assert_eq!((set, dug), (220, 720), "{:?}", sites.placed);
        // Plain sloping ground has no gorge.
        assert!(!sites.placed.iter().any(|site| site.starts_with("gorge")));
    }
}

//! Streaming — a compiled world's chunks made around you as you move, and dropped behind you.
//!
//! **Every frame:**
//! 1. **Where you are.** The camera's chunk, and which way it's facing.
//! 2. **What's wanted.** Every chunk within `radius` chunks (a circle), plus one ring more, that
//!    isn't loaded or being made. They're queued **nearest first**, and chunks **ahead** count as
//!    nearer, so the ground in front of you fills in before the ground behind. (The queue is made
//!    again when you cross into another chunk, and every half second as you turn.)
//! 3. **Make them** on background threads, a few at a time (`most_at_once`), so the game never
//!    waits and the work spreads over time instead of spiking.
//! 4. **Put finished ones in**, within a small time budget per frame.
//! 5. **Draw a chunk only once all four of its neighbours are in.** Its edges then come out right
//!    the first time: no wall of faces at the edge of the loaded area to draw and then undo, so each
//!    chunk is meshed once. (That's why one extra ring is made: it lets the ring inside it be drawn.)
//! 6. **Drop chunks** more than `radius + 3` away (a margin, so wandering back and forth over a
//!    border doesn't load and drop the same chunks over and over). Changed chunks are kept: they're
//!    saved with the world.
//!
//! Only compiled worlds stream; the test cylinder is made whole.

use super::position::ChunkPos;
use super::interpreter::CompiledMap;
use super::terrain::{Terrain, TerrainBlocks};
use super::{Chunk, VoxelWorld};
use crate::coords::bevy_to_game;
use crate::data::BlockRegistry;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// Tunables (on the "World" debug page, saved in "everything" presets).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StreamSettings {
    /// How far around you chunks are made and drawn, in chunks (1 chunk = 32 blocks).
    pub radius: i32,
    /// How much nearer chunks ahead count: 0 = all directions alike, 0.5 = straight ahead counts as
    /// half as far.
    pub ahead: f32,
    /// The most chunks being made at once (0 = one per CPU core).
    pub most_at_once: usize,
    /// The most time per frame spent putting finished chunks into the world, in milliseconds.
    pub insert_budget_ms: f32,
}

impl Default for StreamSettings {
    fn default() -> Self {
        Self { radius: 32, ahead: 0.5, most_at_once: 0, insert_budget_ms: 1.0 }
    }
}

/// How far past the radius chunks are kept before being dropped.
const KEEP_MARGIN: i32 = 3;

#[derive(Resource, Default)]
pub struct Streaming {
    /// The map these chunks were made from (a new world means starting over).
    map: Option<Arc<CompiledMap>>,
    /// Chunks being made, and finished ones not yet put in.
    making: HashMap<ChunkPos, Task<Option<Chunk>>>,
    finished: Vec<(ChunkPos, Option<Chunk>)>,
    /// Chunks wanted, best first (the next one is at the end).
    queue: Vec<ChunkPos>,
    /// Chunks past the edge of the map (they'll never exist, but count as "in" for their
    /// neighbours).
    empty: HashSet<ChunkPos>,
    /// Chunks that have been handed to the renderer.
    drawn: HashSet<ChunkPos>,
    /// Where the queue was made from, and when.
    queued_at: Option<ChunkPos>,
    queued_when: Option<Instant>,
    // ── For the Performance page ──
    pub made: u64,
    pub dropped: u64,
}

impl Streaming {
    pub fn queued(&self) -> usize {
        self.queue.len()
    }

    pub fn being_made(&self) -> usize {
        self.making.len() + self.finished.len()
    }

    /// Chunks that have been handed to the renderer.
    pub fn drawn(&self) -> &HashSet<ChunkPos> {
        &self.drawn
    }

    /// Whether a chunk is in the world or known to be past the map.
    fn settled(&self, world: &VoxelWorld, pos: ChunkPos) -> bool {
        world.chunk(pos).is_some() || self.empty.contains(&pos)
    }

    /// Hands a loaded chunk to the renderer if all four of its neighbours are in.
    fn draw_if_ready(&mut self, world: &mut VoxelWorld, pos: ChunkPos) {
        if self.drawn.contains(&pos) || world.chunk(pos).is_none() {
            return;
        }
        let neighbours = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(x, y)| ChunkPos::new(pos.x + x, pos.y + y));
        if neighbours.iter().all(|&n| self.settled(world, n)) {
            world.mark_chunk_dirty(pos);
            self.drawn.insert(pos);
        }
    }
}

/// The camera's spot (game axes) and which way it faces across the ground (a unit vector, or zero
/// looking straight up or down).
fn focus(camera: &Query<&Transform, With<Camera3d>>) -> Option<(Vec3, Vec2)> {
    let transform = camera.iter().next()?;
    let position = bevy_to_game(transform.translation);
    let ahead = bevy_to_game(transform.translation + *transform.forward()) - position;
    let flat = Vec2::new(ahead.x, ahead.y);
    Some((position, flat.normalize_or_zero()))
}

/// The chunks wanted around `centre`, best first: within `radius + 1`, not in the world, not being
/// made, not past the map. Returned with the best one last (so it can be popped).
fn wanted(streaming: &Streaming, world: &VoxelWorld, centre: ChunkPos, facing: Vec2, settings: &StreamSettings) -> Vec<ChunkPos> {
    let reach = settings.radius + 1;
    let mut wanted: Vec<(f32, ChunkPos)> = Vec::new();
    for x in -reach..=reach {
        for y in -reach..=reach {
            if x * x + y * y > reach * reach {
                continue;
            }
            let pos = ChunkPos::new(centre.x + x, centre.y + y);
            if streaming.settled(world, pos) || streaming.making.contains_key(&pos) || streaming.finished.iter().any(|(p, _)| *p == pos) {
                continue;
            }
            let offset = Vec2::new(x as f32, y as f32);
            let distance = offset.length();
            let towards = if distance > 0.0 { offset.dot(facing) / distance } else { 1.0 };
            wanted.push((distance * (1.0 - settings.ahead * towards.max(0.0)), pos));
        }
    }
    wanted.sort_by(|a, b| b.0.total_cmp(&a.0));
    wanted.into_iter().map(|(_, pos)| pos).collect()
}

/// The streaming system (see the top of this file).
pub fn stream_chunks(
    terrain: Res<Terrain>,
    settings: Res<StreamSettings>,
    registry: Res<BlockRegistry>,
    camera: Query<&Transform, With<Camera3d>>,
    mut world: ResMut<VoxelWorld>,
    mut streaming: ResMut<Streaming>,
) {
    let Some(map) = terrain.compiled() else {
        if streaming.map.is_some() {
            *streaming = Streaming::default();
        }
        return;
    };
    // A new world: start over (chunks already in, such as the first ones around the middle, are
    // drawn as their neighbours arrive).
    if streaming.map.as_ref().is_none_or(|old| !Arc::ptr_eq(old, map)) {
        *streaming = Streaming { map: Some(map.clone()), ..Streaming::default() };
    }
    let Some((position, facing)) = focus(&camera) else { return };
    let centre = ChunkPos::of_block(position.floor().as_ivec3());

    // Finished chunks come in, within the time budget.
    let started = Instant::now();
    let mut done = Vec::new();
    streaming.making.retain(|&pos, task| match check_ready(task) {
        Some(chunk) => {
            done.push((pos, chunk));
            false
        }
        None => true,
    });
    streaming.finished.extend(done);
    while let Some((pos, chunk)) = streaming.finished.pop() {
        match chunk {
            // A chunk already there (one loaded from a save, say) is kept.
            Some(chunk) if world.chunk(pos).is_none() => world.insert_chunk_quiet(chunk),
            Some(_) => {}
            None => {
                streaming.empty.insert(pos);
            }
        }
        for (x, y) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            streaming.draw_if_ready(&mut world, ChunkPos::new(pos.x + x, pos.y + y));
        }
        if started.elapsed().as_secs_f32() * 1000.0 > settings.insert_budget_ms {
            break;
        }
    }

    // A new queue when you've crossed into another chunk, or every half second as you turn.
    let stale = streaming.queued_when.is_none_or(|when| when.elapsed().as_secs_f32() > 0.5);
    if streaming.queued_at != Some(centre) || stale {
        let queue = wanted(&streaming, &world, centre, facing, &settings);
        streaming.queue = queue;
        streaming.queued_at = Some(centre);
        streaming.queued_when = Some(Instant::now());
        // Drop chunks well behind (not changed ones).
        let keep = settings.radius + KEEP_MARGIN;
        let far: Vec<ChunkPos> = world
            .chunk_positions()
            .filter(|pos| {
                let (x, y) = (pos.x - centre.x, pos.y - centre.y);
                x * x + y * y > keep * keep && !world.changed_chunks().contains(pos)
            })
            .collect();
        for pos in far {
            world.remove_chunk(pos);
            streaming.drawn.remove(&pos);
            streaming.dropped += 1;
        }
        streaming.empty.retain(|pos| {
            let (x, y) = (pos.x - centre.x, pos.y - centre.y);
            x * x + y * y <= keep * keep
        });
    }

    // Start making more, up to the limit.
    let most = if settings.most_at_once == 0 { std::thread::available_parallelism().map_or(4, |n| n.get()) } else { settings.most_at_once };
    let pool = AsyncComputeTaskPool::get();
    let blocks = TerrainBlocks::from_registry(&registry);
    while streaming.making.len() < most {
        let Some(pos) = streaming.queue.pop() else { break };
        if streaming.settled(&world, pos) || streaming.making.contains_key(&pos) {
            continue;
        }
        let map = map.clone();
        let task = pool.spawn(async move { map.chunk(pos, &blocks) });
        streaming.making.insert(pos, task);
        streaming.made += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_and_ahead_come_first() {
        let width = 64;
        let map = CompiledMap::of_heights(width, 32.0, &vec![140.0; width * width]);
        let streaming = Streaming { map: Some(Arc::new(map)), ..Streaming::default() };
        let world = VoxelWorld::default();
        let settings = StreamSettings { radius: 4, ahead: 0.5, ..StreamSettings::default() };
        // Facing north (+x).
        let queue = wanted(&streaming, &world, ChunkPos::new(0, 0), Vec2::new(1.0, 0.0), &settings);
        let order: Vec<ChunkPos> = queue.iter().rev().copied().collect();
        assert_eq!(order[0], ChunkPos::new(0, 0));
        let rank = |x: i32, y: i32| order.iter().position(|&p| p == ChunkPos::new(x, y)).unwrap();
        // Three chunks ahead comes before three chunks behind.
        assert!(rank(3, 0) < rank(-3, 0));
        // Everything within the radius plus one ring, nothing beyond.
        assert!(order.iter().all(|p| p.x * p.x + p.y * p.y <= 25));
        assert!(order.contains(&ChunkPos::new(5, 0)));
    }

    #[test]
    fn a_chunk_is_drawn_only_once_its_four_neighbours_are_in() {
        let mut world = VoxelWorld::default();
        let mut streaming = Streaming::default();
        let empty = |pos| Chunk::from_sections(pos, (0..crate::world::SECTIONS_PER_CHUNK).map(|_| crate::world::Section::Uniform(crate::world::BlockId::AIR)).collect());
        world.insert_chunk_quiet(empty(ChunkPos::new(0, 0)));
        world.take_dirty();
        streaming.draw_if_ready(&mut world, ChunkPos::new(0, 0));
        assert!(world.take_dirty().is_empty(), "drawn before its neighbours were in");
        for (x, y) in [(1, 0), (-1, 0), (0, 1)] {
            world.insert_chunk_quiet(empty(ChunkPos::new(x, y)));
        }
        // The last neighbour is past the map: that counts as in.
        streaming.empty.insert(ChunkPos::new(0, -1));
        streaming.draw_if_ready(&mut world, ChunkPos::new(0, 0));
        assert_eq!(world.take_dirty().len(), crate::world::SECTIONS_PER_CHUNK);
        // And only once.
        streaming.draw_if_ready(&mut world, ChunkPos::new(0, 0));
        assert!(world.take_dirty().is_empty());
    }
}

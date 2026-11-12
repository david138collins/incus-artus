//! Streaming — a compiled world's chunks made around you as you move, and dropped behind you.
//!
//! **Every frame:**
//! 1. **Where you are.** The camera's chunk, and which way it's facing.
//! 2. **What's wanted.** Every chunk within `radius` chunks (a circle), plus one ring more, that
//!    isn't loaded or being made. They're queued **nearest first**, and chunks **ahead** count as
//!    nearer, so the ground in front of you fills in before the ground behind. (The queue is made
//!    again when you cross into another chunk, and every half second as you turn.)
//! 3. **Make them** on the loading lane (`loading`), a few at a time (`most_at_once`), so the game
//!    never waits and the work spreads over time instead of spiking.
//! 4. **Put finished ones in**, within a small time budget per frame.
//! 5. **Draw a chunk only once all four of its neighbours are in.** Its edges then come out right
//!    the first time: no wall of faces at the edge of the loaded area to draw and then undo, so each
//!    chunk is meshed once. (That's why one extra ring is made: it lets the ring inside it be drawn.)
//! 6. **Drop chunks** more than `radius + 3` away (a margin, so wandering back and forth over a
//!    border doesn't load and drop the same chunks over and over). Changed chunks are put away
//!    instead (kept, and saved with the world, but not drawn), and come back from there, not made
//!    again, when they're wanted.
//!
//! Only compiled worlds stream; the test cylinder is made whole.

use super::position::ChunkPos;
use super::interpreter::CompiledMap;
use super::terrain::{Terrain, TerrainBlocks};
use super::{Chunk, VoxelWorld};
use crate::data::BlockRegistry;
use crate::ecs::prelude::*;
use crate::loading::{Job, Tier};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// Tunables (on the "World" debug page, saved in "everything" presets).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StreamSettings {
    /// How far around you chunks are made, in chunks (1 chunk = 32 blocks).
    pub radius: i32,
    /// How much nearer chunks ahead count: 0 = all directions alike, 0.5 = straight ahead counts as
    /// half as far.
    pub ahead: f32,
    /// The most chunks being made at once (0 = one per CPU core).
    pub most_at_once: usize,
    /// The most time per frame spent putting finished chunks into the world, in milliseconds.
    pub insert_budget_ms: f32,
    /// How far the real chunks reach at most, whatever `radius` says (chunks): set by the far copies
    /// while the march draws the far land from the recipe (`far_copies::cap_streaming`), so the real
    /// chunks are made only near you [made out to 32 chunks just to be copied into coarse cells, some
    /// 3,400 of them on every core as the game started, it read 102 °C before a step was flown:
    /// David, "we need to address this problem at the root"]. Not saved; no cap by default.
    #[serde(skip, default = "no_cap")]
    pub cap: i32,
}

fn no_cap() -> i32 {
    i32::MAX
}

/// Alike if they're set alike: the cap isn't a setting (a preset's "default" light ignores it).
impl PartialEq for StreamSettings {
    fn eq(&self, other: &Self) -> bool {
        self.radius == other.radius && self.ahead == other.ahead && self.most_at_once == other.most_at_once && self.insert_budget_ms == other.insert_budget_ms
    }
}

impl StreamSettings {
    /// How far the real chunks are made (chunks): the radius, or the cap if that's nearer.
    pub fn reach(&self) -> i32 {
        self.radius.min(self.cap)
    }
}

impl Default for StreamSettings {
    fn default() -> Self {
        // Four chunks (128 blocks): what walking, collision and edits need round the player [the regroup,
        // David 2026-11-06]. Nothing draws chunks since the regroup (sight is the card's, from the recipe);
        // at 32 they kept some 2.4 cores busy all the time while flying.
        Self { radius: 4, ahead: 0.5, most_at_once: 0, insert_budget_ms: 1.0, cap: no_cap() }
    }
}

/// How far past the radius chunks are kept before being dropped.
const KEEP_MARGIN: i32 = 3;

/// Is a chunk this far from you (chunks, north and east) one of the real chunks? Those are made
/// within the streaming circle (`radius`) and drawn within the render distance's square
/// (`render_distance`); past them the far copies and the distant sheet take over.
pub fn real_chunk(offset: IVec2, render_distance: i32, radius: i32) -> bool {
    offset.x.abs().max(offset.y.abs()) <= render_distance && offset.x * offset.x + offset.y * offset.y <= radius * radius
}

#[derive(Resource, Default)]
pub struct Streaming {
    /// The map these chunks were made from (a new world means starting over).
    map: Option<Arc<CompiledMap>>,
    /// Chunks being made, and finished ones not yet put in.
    making: HashMap<ChunkPos, Job<Option<Chunk>>>,
    finished: Vec<(ChunkPos, Option<Chunk>)>,
    /// Chunks wanted, best first (the next one is at the end).
    queue: Vec<ChunkPos>,
    /// Chunks past the edge of the map (they'll never exist, but count as "in" for their
    /// neighbours).
    empty: HashSet<ChunkPos>,
    /// Chunks that have been handed to the renderer (the far copies: marked changed once their
    /// neighbours are in).
    drawn: HashSet<ChunkPos>,
    /// Where the queue was made from, and when.
    queued_at: Option<ChunkPos>,
    queued_when: Option<Instant>,
    // ── For the Performance page ──
    pub made: u64,
    pub dropped: u64,
}

impl Streaming {
    /// Drops chunks being made (or made and not yet in) at these places, made from a plan that's
    /// since changed (a cave placed): they're made again from the new one.
    pub fn forget(&mut self, places: &HashSet<ChunkPos>) {
        self.making.retain(|pos, _| !places.contains(pos));
        self.finished.retain(|(pos, _)| !places.contains(pos));
        self.queued_when = None;
    }

    pub fn queued(&self) -> usize {
        self.queue.len()
    }

    pub fn being_made(&self) -> usize {
        self.making.len() + self.finished.len()
    }

    /// Chunks that have been handed to the renderer.
    #[cfg(test)]
    pub fn drawn(&self) -> &HashSet<ChunkPos> {
        &self.drawn
    }

    /// Whether a chunk is in the world or known to be past the map.
    fn settled(&self, world: &VoxelWorld, pos: ChunkPos) -> bool {
        world.chunk(pos).is_some() || self.empty.contains(&pos)
    }

    /// Starting over for a new world. Chunks already in (the first ones round the spawn, made before
    /// streaming starts) are drawn now if their neighbours are in too, or as their neighbours
    /// arrive, like any other. (Found 2026-10-25: the middle ones used to be on screen but never
    /// counted as drawn, so the distant terrain showed through them round the spawn.)
    fn start_over(map: Arc<CompiledMap>, world: &mut VoxelWorld) -> Self {
        let mut streaming = Self { map: Some(map), ..Self::default() };
        let already: Vec<ChunkPos> = world.chunk_positions().collect();
        for pos in already {
            streaming.draw_if_ready(world, pos);
        }
        streaming
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
fn focus(view: &crate::camera::CameraView) -> Option<(Vec3, Vec2)> {
    let flat = Vec2::new(view.looking.x, view.looking.y);
    Some((view.position, flat.normalize_or_zero()))
}

/// The chunks wanted around `centre`, best first: within `radius + 1`, not in the world, not being
/// made, not past the map. Returned with the best one last (so it can be popped).
fn wanted(streaming: &Streaming, world: &VoxelWorld, centre: ChunkPos, facing: Vec2, settings: &StreamSettings) -> Vec<ChunkPos> {
    let reach = settings.reach() + 1;
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
    caves: Res<super::Caves>,
    settings: Res<StreamSettings>,
    registry: Res<BlockRegistry>,
    view: Res<crate::camera::CameraView>,
    mut world: ResMut<VoxelWorld>,
    mut streaming: ResMut<Streaming>,
) {
    let _timed = crate::perf::Timed::new("stream chunks");
    let Some(map) = terrain.compiled() else {
        if streaming.map.is_some() {
            *streaming = Streaming::default();
        }
        return;
    };
    // A new world: start over (chunks already in, such as the first ones around the middle, are
    // drawn as their neighbours arrive).
    if streaming.map.as_ref().is_none_or(|old| !Arc::ptr_eq(old, map)) {
        *streaming = Streaming::start_over(map.clone(), &mut world);
    }
    let Some((position, facing)) = focus(&view) else { return };
    let centre = ChunkPos::of_block(position.floor().as_ivec3());

    // Finished chunks come in, within the time budget.
    let timed_in = crate::perf::Timed::new("chunks in");
    let started = Instant::now();
    let mut done = Vec::new();
    streaming.making.retain(|&pos, job| match job.ready() {
        Ok(Some(chunk)) => {
            done.push((pos, chunk));
            false
        }
        Ok(None) => true,
        // It failed (the crash log says why): asked for again with the next queue.
        Err(()) => false,
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

    drop(timed_in);
    // A new queue when you've crossed into another chunk, or every half second as you turn.
    let stale = streaming.queued_when.is_none_or(|when| when.elapsed().as_secs_f32() > 0.5);
    if streaming.queued_at != Some(centre) || stale {
        let timed_queue = crate::perf::Timed::new("chunk wish list");
        let queue = wanted(&streaming, &world, centre, facing, &settings);
        drop(timed_queue);
        let _timed_drop = crate::perf::Timed::new("chunks dropped");
        streaming.queue = queue;
        streaming.queued_at = Some(centre);
        streaming.queued_when = Some(Instant::now());
        // Drop chunks well behind (changed ones are put away).
        let keep = settings.reach() + KEEP_MARGIN;
        let far: Vec<ChunkPos> = world
            .chunk_positions()
            .filter(|pos| {
                let (x, y) = (pos.x - centre.x, pos.y - centre.y);
                x * x + y * y > keep * keep
            })
            .collect();
        for pos in far {
            if world.changed_chunks().contains(&pos) {
                world.put_away(pos);
            } else {
                world.remove_chunk(pos);
            }
            streaming.drawn.remove(&pos);
            streaming.dropped += 1;
        }
        streaming.empty.retain(|pos| {
            let (x, y) = (pos.x - centre.x, pos.y - centre.y);
            x * x + y * y <= keep * keep
        });
    }

    // Start making more, up to the limit.
    let _timed_ask = crate::perf::Timed::new("chunks asked for");
    let most = if settings.most_at_once == 0 { crate::loading::workers() } else { settings.most_at_once };
    let blocks = TerrainBlocks::from_registry(&registry);
    while streaming.making.len() < most {
        let Some(pos) = streaming.queue.pop() else { break };
        if streaming.settled(&world, pos) || streaming.making.contains_key(&pos) {
            continue;
        }
        // A changed chunk comes back as it was left, ready to go in next frame.
        if let Some(chunk) = world.take_put_away(pos) {
            streaming.finished.push((pos, Some(chunk)));
            continue;
        }
        let (map, blocks, caves) = (map.clone(), blocks.clone(), caves.0.clone());
        // On the loading lane: soon (they're round you), nearest first.
        let (x, y) = ((pos.x - centre.x) as f32, (pos.y - centre.y) as f32);
        let job = crate::loading::submit(Tier::Soon, (x * x + y * y).sqrt(), move || {
            let _timed = crate::perf::Background::new("chunks made");
            map.chunk(pos, &blocks, &caves)
        });
        streaming.making.insert(pos, job);
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

    #[test]
    fn a_changed_chunk_put_away_is_kept_for_saving_and_comes_back_as_it_was() {
        let mut world = VoxelWorld::default();
        let empty = |pos| Chunk::from_sections(pos, (0..crate::world::SECTIONS_PER_CHUNK).map(|_| crate::world::Section::Uniform(crate::world::BlockId::AIR)).collect());
        let pos = ChunkPos::new(2, 3);
        world.insert_chunk(empty(pos));
        let block = IVec3::new(70, 100, 50);
        world.set_block(block, crate::world::BlockId(1));
        world.take_dirty();
        world.put_away(pos);
        // Gone from the world (its meshes are removed), but still there to save.
        assert!(world.chunk(pos).is_none());
        assert_eq!(world.take_dirty().len(), crate::world::SECTIONS_PER_CHUNK);
        assert!(world.changed_chunks().contains(&pos));
        assert_eq!(world.changed_chunk(pos).unwrap().get(block), crate::world::BlockId(1));
        // Back, edit and all.
        let back = world.take_put_away(pos).unwrap();
        assert_eq!(back.get(block), crate::world::BlockId(1));
        assert!(world.take_put_away(pos).is_none());
    }

    #[test]
    fn chunks_made_before_streaming_starts_count_as_drawn_once_their_neighbours_are_in() {
        // The first chunks round the spawn: a 3 × 3 block of them.
        let mut world = VoxelWorld::default();
        let empty = |pos| Chunk::from_sections(pos, (0..crate::world::SECTIONS_PER_CHUNK).map(|_| crate::world::Section::Uniform(crate::world::BlockId::AIR)).collect());
        for x in -1..=1 {
            for y in -1..=1 {
                world.insert_chunk_quiet(empty(ChunkPos::new(x, y)));
            }
        }
        let map = Arc::new(CompiledMap::of_heights(4, 32.0, &[130.0; 16]));
        let streaming = Streaming::start_over(map, &mut world);
        // The middle one has all four neighbours in; the ones round the edge wait for theirs.
        assert!(streaming.drawn().contains(&ChunkPos::new(0, 0)));
        assert!(!streaming.drawn().contains(&ChunkPos::new(1, 0)));
        assert_eq!(streaming.drawn().len(), 1);
    }
}

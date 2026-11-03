//! Meshing jobs — rebuilding section meshes on background threads.
//!
//! Every frame:
//! 1. **Collect** the sections the world marked as changed ("dirty") into a waiting list. Two
//!    kinds are settled at once instead, without waiting: sections of chunks that are gone (their
//!    meshes are removed), and empty sky with no mesh (nothing to draw, nothing to remove). Most of
//!    a chunk is sky, so this keeps them from using up the job slots.
//! 2. **Start** jobs for the waiting sections nearest the camera, up to a limit: so many per
//!    240th of a second, so terrain fills in as fast at 60 FPS as at 240 (a limit per frame
//!    meant a slow machine also built terrain four times slower, and fell behind as you flew).
//!    Starting a job means copying that section's blocks (plus a one-block border from its
//!    neighbours) and handing the copy to a worker thread. The game never waits for it.
//! 3. **Finish** any jobs that are done: upload the new mesh and show it.
//!
//! If a section changes again while its job is still running, the old job is cancelled and
//! a fresh one started, so a stale mesh never replaces a newer one.

use super::material::{ATTRIBUTE_PACKED_VOXEL, TerrainMaterial};
use super::placed_light::PlacedLights;
use super::mesher::{MeshInput, MeshOutput, face_index, mesh_section, plane_axes};
use super::sky_light::{LightRegion, MOST_REACH, sky_heights};
use super::{RenderSettings, TerrainAssets};

use crate::world::{
    BlockId, CHUNK_SIZE, ChunkPos, SECTION_SIZE, SECTION_VOLUME, Section, SectionKey, VoxelWorld, WORLD_HEIGHT,
};
use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// A drawn section. The entity's `Mesh3d` holds the current mesh.
#[derive(Component)]
pub struct SectionMesh {
    pub key: SectionKey,
    pub quads: u32,
    pub vertices: u32,
    /// Meshed past the line: only what the march can't draw yet (`follow_the_line`).
    pub past: bool,
}

/// Bookkeeping for all section meshes and jobs.
#[derive(Resource, Default)]
pub struct MeshingState {
    /// Sections that need a new mesh, waiting for a free slot.
    pub waiting: HashSet<SectionKey>,
    /// Jobs currently running on worker threads.
    pub running: HashMap<SectionKey, Task<JobResult>>,
    /// Finished meshes waiting their turn to be handed to the graphics side (a few each frame:
    /// see `finish_jobs`), oldest first.
    pub ready: VecDeque<(SectionKey, JobResult)>,
    /// The entity drawing each section (sections with nothing to draw have none).
    pub entities: HashMap<SectionKey, Entity>,
    /// Goes up whenever the lists above may have changed (so the distant terrain's hand-off only
    /// looks them over again then).
    pub changes: u64,
    // ── Statistics for the debug panel ──
    pub jobs_finished: u64,
    pub total_job_micros: u64,
    pub slowest_job_micros: u64,
    /// Each chunk's sky heights (`sky_light::sky_heights`), worked out when first needed and
    /// forgotten when any of its sections changes.
    pub sky: HashMap<ChunkPos, Vec<i32>>,
    /// Chunks whose light may have changed since the objects were drawn (`objects.rs`).
    pub relight: HashSet<ChunkPos>,
    /// Chunks whose blocks changed, came or went since the far copies last looked (a copy made from
    /// the real chunks is made again: `far_copies.rs`).
    pub touched: HashSet<ChunkPos>,
    /// Whether each chunk's sections were last meshed past the line (true: only what the march can't
    /// draw yet) or whole; and the line they were meshed for (`follow_the_line`).
    pub meshed_past: HashMap<ChunkPos, bool>,
    pub line: Option<Line>,
    /// Chunks come inside the line, being meshed whole again: not counted as drawn until they are
    /// (`distant::update_shown_chunks` takes them), so the copy there stays till then.
    pub unshown: HashSet<ChunkPos>,
}

/// Where the line is that the march starts at, for the meshing (`follow_the_line`): your chunk, the
/// render distance and the line (chunks), and whether the march draws the real chunks past it.
#[derive(Clone, Copy, PartialEq)]
pub struct Line {
    home: IVec2,
    render_distance: i32,
    march_from: i32,
    on: bool,
}

impl Line {
    /// Is this chunk past the line, so meshed only for what the march can't draw yet?
    fn past(&self, pos: ChunkPos) -> bool {
        self.on && !super::drawn_near(IVec2::new(pos.x, pos.y) - self.home, self.render_distance, self.march_from)
    }
}

impl MeshingState {
    fn past_line(&self, pos: ChunkPos) -> bool {
        self.line.is_some_and(|line| line.past(pos))
    }
}

pub struct JobResult {
    output: MeshOutput,
    micros: u64,
    /// Meshed past the line (only what the march can't draw yet).
    past: bool,
}

/// Past the line where the march starts (`RenderSettings::march_from`), while the march draws the
/// real chunks there (`far_copies.rs`, R2 in `docs/far-view.md`), a chunk's sections are meshed only
/// for what the march can't draw yet: see-through blocks and pieces. This keeps each chunk meshed the
/// way its place asks as you move, or the line does: one come inside is meshed whole again, one gone
/// past is meshed again without its blocks.
pub fn follow_the_line(
    world: Res<VoxelWorld>,
    (render, streaming): (Res<RenderSettings>, Res<crate::world::StreamSettings>),
    (copy_settings, distant_settings): (Res<super::far_copies::FarCopySettings>, Res<super::distant::DistantSettings>),
    (camera, origin): (Query<&Transform, With<Camera3d>>, Res<crate::coords::RenderOrigin>),
    mut state: ResMut<MeshingState>,
) {
    let Some(camera) = camera.iter().next() else { return };
    let at = origin.to_game(camera.translation);
    let home = IVec2::new((at.x / CHUNK_SIZE as f32).floor() as i32, (at.y / CHUNK_SIZE as f32).floor() as i32);
    let on = super::far_copies::marches_real(&copy_settings, &distant_settings, &render, &streaming);
    let line = Line { home, render_distance: render.render_distance_chunks, march_from: render.march_from, on };
    if state.line == Some(line) {
        return;
    }
    state.line = Some(line);
    let again: Vec<(ChunkPos, bool)> = state.meshed_past.iter().filter(|&(&pos, &past)| past != line.past(pos)).map(|(&pos, &past)| (pos, past)).collect();
    for (pos, was_past) in again {
        if let Some(chunk) = world.chunk(pos) {
            for (level, section) in chunk.sections().iter().enumerate() {
                if !matches!(section, Section::Uniform(id) if id.is_air()) {
                    state.waiting.insert(SectionKey { chunk: pos, level: level as u8 });
                }
            }
        }
        if was_past {
            state.unshown.insert(pos);
        }
    }
    state.changes += 1;
}

pub fn collect_dirty_sections(mut commands: Commands, mut world: ResMut<VoxelWorld>, mut state: ResMut<MeshingState>) {
    let _timed = crate::render::Timed::new("collect dirty sections");
    let dirty = world.take_dirty();
    if dirty.is_empty() {
        return;
    }
    state.changes += 1;
    let mut gone = HashSet::new();
    for key in dirty {
        state.sky.remove(&key.chunk);
        state.relight.insert(key.chunk);
        state.touched.insert(key.chunk);
        let section = world.chunk(key.chunk).map(|chunk| &chunk.sections()[key.level as usize]);
        match section {
            // A chunk that's gone (dropped behind you): just remove its meshes.
            None => {
                state.waiting.remove(&key);
                state.running.remove(&key);
                state.meshed_past.remove(&key.chunk);
                gone.insert(key);
                if let Some(entity) = state.entities.remove(&key) {
                    commands.entity(entity).despawn();
                }
            }
            // Empty sky never draws anything (the faces of the ground below belong to the
            // ground's own section): only one that had blocks a moment ago has a mesh to remove.
            Some(Section::Uniform(id)) if id.is_air() && !state.entities.contains_key(&key) => {
                state.waiting.remove(&key);
                state.running.remove(&key);
            }
            Some(_) => {
                state.waiting.insert(key);
            }
        }
    }
    // Meshes made for them, still waiting their turn to be shown, are no longer wanted.
    if !gone.is_empty() {
        state.ready.retain(|(key, _)| !gone.contains(key));
    }
}

pub fn start_jobs(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    terrain: Res<TerrainAssets>,
    (settings, time, placed): (Res<RenderSettings>, Res<Time<Real>>, Res<PlacedLights>),
    mut state: ResMut<MeshingState>,
    (camera, origin): (Query<&Transform, With<Camera3d>>, Res<crate::coords::RenderOrigin>),
) {
    let _timed = crate::render::Timed::new("start meshing");
    if state.waiting.is_empty() {
        return;
    }
    state.changes += 1;

    // Nearest sections first, so what you're looking at appears first. (Only the nearest few are
    // picked out, not the whole list sorted: there can be tens of thousands waiting.)
    let camera_position = camera
        .iter()
        .next()
        .map_or(Vec3::ZERO, |t| origin.to_game(t.translation));
    let distance = |key: &SectionKey| {
        let centre = section_origin(*key).as_vec3() + Vec3::splat(SECTION_SIZE as f32 / 2.0);
        centre.distance_squared(camera_position) as i64
    };
    let mut keys: Vec<SectionKey> = state.waiting.iter().copied().collect();
    // The setting is per 240th of a second: a slower frame starts more, but at most twice as many
    // (it was 8: a slow frame then started hundreds more, whose meshes made the next frames slower
    // still). None while plenty of finished meshes are still waiting their turn.
    let frames = (time.delta_secs() * 240.0).clamp(1.0, 2.0);
    if state.ready.len() > settings.meshes_per_frame * 8 {
        return;
    }
    let take = ((settings.max_jobs_per_frame as f32 * frames).round() as usize).min(keys.len());
    if take == 0 {
        return;
    }
    if take < keys.len() {
        keys.select_nth_unstable_by_key(take - 1, distance);
        keys.truncate(take);
    }

    let pool = AsyncComputeTaskPool::get();
    for key in keys {
        state.waiting.remove(&key);
        let past = state.past_line(key.chunk);
        if world.chunk(key.chunk).is_some() {
            state.meshed_past.insert(key.chunk, past);
        }
        match gather_input(&world, key, &terrain, &settings, &mut state.sky, &placed, past) {
            Some(input) => {
                let task = pool.spawn(async move {
                    let started = Instant::now();
                    let output = mesh_section(&input);
                    JobResult {
                        output,
                        micros: started.elapsed().as_micros() as u64,
                        past,
                    }
                });
                // Replacing a running job drops (and so cancels) the old one.
                state.running.insert(key, task);
            }
            None => {
                // Nothing to draw (all air, or the chunk is gone): remove any old mesh.
                state.running.remove(&key);
                if let Some(entity) = state.entities.remove(&key) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn finish_jobs(
    mut commands: Commands,
    mut state: ResMut<MeshingState>,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain: Res<TerrainAssets>,
    (settings, origin): (Res<RenderSettings>, Res<crate::coords::RenderOrigin>),
    mut existing: Query<(&mut Mesh3d, &mut SectionMesh)>,
) {
    let _timed = crate::render::Timed::new("finish meshing");
    let mut done = Vec::new();
    for (key, task) in state.running.iter_mut() {
        if let Some(result) = check_ready(task) {
            done.push((*key, result));
        }
    }
    for (key, result) in done {
        state.running.remove(&key);
        state.jobs_finished += 1;
        state.total_job_micros += result.micros;
        state.slowest_job_micros = state.slowest_job_micros.max(result.micros);
        state.ready.push_back((key, result));
    }

    // Only so many new meshes are handed to the graphics side each frame [2026-10-28]: taking in a
    // new mesh costs Bevy far more than making it costs us, and when hundreds arrived at once (flying
    // fast, at a long render distance) frames took 50-200 ms. The rest wait their turn; meanwhile
    // the distant terrain shows under any chunk not yet drawn.
    let (mut meshes_left, mut vertices_left) = (settings.meshes_per_frame.max(1), VERTICES_PER_FRAME);
    let mut handed = false;
    while meshes_left > 0 && vertices_left > 0 {
        let Some((key, result)) = state.ready.pop_front() else { break };
        // A newer mesh for it is on its way: this one's out of date.
        if state.running.contains_key(&key) || state.waiting.contains(&key) {
            continue;
        }
        meshes_left -= 1;
        vertices_left = vertices_left.saturating_sub(result.output.vertices.len());
        handed = true;
        let output = result.output;
        if output.quads == 0 {
            if let Some(entity) = state.entities.remove(&key) {
                commands.entity(entity).despawn();
            }
            continue;
        }

        let (quads, past) = (output.quads, result.past);
        let vertex_count = output.vertices.len() as u32;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(ATTRIBUTE_PACKED_VOXEL, output.vertices);
        mesh.insert_indices(Indices::U32(output.indices));
        let handle = meshes.add(mesh);

        if let Some(&entity) = state.entities.get(&key) {
            if let Ok((mut mesh3d, mut info)) = existing.get_mut(entity) {
                meshes.remove(&mesh3d.0);
                mesh3d.0 = handle;
                info.quads = quads;
                info.vertices = vertex_count;
                info.past = past;
                continue;
            }
        }

        // Bounding box in Bevy's axes, used by frustum culling (game 0..32 → bevy x 0..32, y 0..32, z −32..0).
        let size = SECTION_SIZE as f32;
        let entity = commands
            .spawn((
                Mesh3d(handle),
                MeshMaterial3d::<TerrainMaterial>(terrain.material.clone()),
                Transform::from_translation(origin.to_bevy(section_origin(key).as_vec3())),
                Aabb::from_min_max(Vec3::new(0.0, 0.0, -size), Vec3::new(size, size, 0.0)),
                SectionMesh {
                    key,
                    quads,
                    vertices: vertex_count,
                    past,
                },
            ))
            .id();
        state.entities.insert(key, entity);
    }
    if handed {
        state.changes += 1;
    }
}

/// The most corners of new meshes handed to the graphics side in one frame (a frame of ordinary
/// sections is a few hundred to a few thousand corners each).
const VERTICES_PER_FRAME: usize = 400_000;

/// The block at a section's south-west bottom corner.
pub fn section_origin(key: SectionKey) -> IVec3 {
    IVec3::new(
        key.chunk.x * CHUNK_SIZE,
        key.chunk.y * CHUNK_SIZE,
        key.level as i32 * SECTION_SIZE,
    )
}

/// What's just past one side of a section.
enum Beyond<'a> {
    /// Below the world: counts as solid (nobody can see the underside of the bottom).
    Below,
    /// Above the world, or a chunk that isn't loaded: counts as air.
    Nothing,
    Section(&'a Section),
}

/// The section next to `key` on one side (`axis` 0 = north, 1 = east, 2 = up; `positive` = that
/// way, otherwise the opposite).
fn beyond(world: &VoxelWorld, key: SectionKey, axis: usize, positive: bool) -> Beyond<'_> {
    let step = if positive { 1 } else { -1 };
    let (mut chunk, mut level) = (key.chunk, key.level as i32);
    match axis {
        0 => chunk.x += step,
        1 => chunk.y += step,
        _ => level += step,
    }
    if level < 0 {
        return Beyond::Below;
    }
    match world.chunk(chunk).and_then(|chunk| chunk.sections().get(level as usize)) {
        Some(section) => Beyond::Section(section),
        None => Beyond::Nothing,
    }
}

/// Where a block sits in its section (x fastest, then y, then z), from its position inside it.
fn index_of(local: IVec3) -> usize {
    let size = SECTION_SIZE as usize;
    local.x as usize + local.y as usize * size + local.z as usize * size * size
}

/// Copy everything the mesher needs out of the world. `None` = nothing to draw.
/// `past`: past the line, only what the march can't draw yet (`follow_the_line`).
fn gather_input(world: &VoxelWorld, key: SectionKey, terrain: &TerrainAssets, settings: &RenderSettings, sky: &mut HashMap<ChunkPos, Vec<i32>>, placed: &PlacedLights, past: bool) -> Option<MeshInput> {
    let chunk = world.chunk(key.chunk)?;
    let section = &chunk.sections()[key.level as usize];
    let is_opaque = |id: BlockId| id != BlockId::PIECES && terrain.opaque.get(id.0 as usize).copied().unwrap_or(true);
    let is_see_through = |id: BlockId| terrain.see_through.get(id.0 as usize).copied().unwrap_or(false);
    let is_grid = |id: BlockId| is_see_through(id) && terrain.inner_faces.get(id.0 as usize).copied().unwrap_or(false);
    // Past the line, a section with no see-through blocks or pieces has nothing the march leaves.
    if past && !section.may_hold(|id| id == BlockId::PIECES || is_see_through(id)) {
        return None;
    }
    if let Section::Uniform(id) = section {
        if !is_opaque(*id) && !is_see_through(*id) {
            return None;
        }
        // Buried: solid all through and walled in by solid on all six sides. Nothing shows.
        let walled_in = (0..3).all(|axis| {
            [true, false].into_iter().all(|positive| match beyond(world, key, axis, positive) {
                Beyond::Below => true,
                Beyond::Nothing => false,
                Beyond::Section(Section::Uniform(other)) => is_opaque(*other),
                Beyond::Section(_) => false,
            })
        });
        if is_opaque(*id) && walled_in {
            return None;
        }
    }

    let blocks: Vec<BlockId> = match section {
        Section::Uniform(id) => vec![*id; SECTION_VOLUME],
        _ => (0..SECTION_VOLUME).map(|i| section.get(i)).collect(),
    };

    // The one-block layer just outside each of the six sides, read straight from the section
    // next door.
    let size = SECTION_SIZE;
    let mut outside = [[0u32; 32]; 6];
    let mut outside_outline = [[0u32; 32]; 6];
    let mut outside_grid = [[0u32; 32]; 6];
    for axis in 0..3 {
        let (u_axis, v_axis) = plane_axes(axis);
        for positive in [true, false] {
            let face = face_index(axis, positive);
            let next = beyond(world, key, axis, positive);
            // The layer of the next section that touches this one.
            let depth = if positive { 0 } else { size - 1 };
            for v in 0..size {
                for u in 0..size {
                    let block = match &next {
                        Beyond::Below => {
                            outside[face][v as usize] |= 1 << u;
                            continue;
                        }
                        Beyond::Nothing => continue,
                        Beyond::Section(Section::Uniform(id)) => *id,
                        Beyond::Section(other) => {
                            let mut local = IVec3::ZERO;
                            local[axis] = depth;
                            local[u_axis] = u;
                            local[v_axis] = v;
                            other.get(index_of(local))
                        }
                    };
                    if is_opaque(block) {
                        outside[face][v as usize] |= 1 << u;
                    }
                    if is_grid(block) {
                        outside_grid[face][v as usize] |= 1 << u;
                    } else if is_see_through(block) {
                        outside_outline[face][v as usize] |= 1 << u;
                    }
                }
            }
        }
    }

    // The cells built of pieces, and those just across the section's sides from them (so bars
    // meeting across a side show no seam there).
    let origin = section_origin(key);
    let mut pieces = Vec::new();
    let mut beyond_pieces = std::collections::HashMap::new();
    for (&block, cell) in chunk.pieces_in(key.level as usize) {
        let local = block - origin;
        pieces.push((index_of(local), cell.owners()));
        for axis in 0..3 {
            for step in [-1, 1] {
                let mut next = local;
                next[axis] += step;
                if !(0..size).contains(&next[axis])
                    && let Some(other) = world.get_pieces(origin + next)
                {
                    beyond_pieces.insert(next.to_array(), other.octants());
                }
            }
        }
    }

    // Daylight (`sky_light.rs`), worked out only where there's covered air in or right round the
    // section.
    let reach = settings.sky_reach.clamp(1, MOST_REACH);
    let light = if settings.sky_light { light_region(world, key, &blocks, &outside, reach, &is_opaque, sky) } else { None };

    Some(MeshInput {
        blocks,
        outside,
        outside_outline,
        outside_grid,
        opaque: terrain.opaque.clone(),
        see_through: terrain.see_through.clone(),
        inner_faces: terrain.inner_faces.clone(),
        face_layers: terrain.face_layers.clone(),
        face_random_rotation: terrain.face_random_rotation.clone(),
        bar_layers: terrain.bar_layers.clone(),
        pieces,
        beyond_pieces,
        light,
        reach,
        // Placed light (`placed_light.rs`): only the cells some light reaches.
        placed: placed.around_section(section_origin(key)),
        only_see_through: past,
    })
}

/// The cells round a section daylight is worked out from (`sky_light.rs`: the section and `reach` +
/// 1 blocks round it), if any air in it or just across its sides is covered (something solid above
/// it); `None` if every cell there is solid or open to the sky, so everything's in full daylight.
/// Unloaded chunks count as solid, and as covered.
fn light_region(world: &VoxelWorld, key: SectionKey, blocks: &[BlockId], outside: &[[u32; 32]; 6], reach: i32, is_opaque: &dyn Fn(BlockId) -> bool, sky: &mut HashMap<ChunkPos, Vec<i32>>) -> Option<LightRegion> {
    let origin = section_origin(key);
    let (size, margin) = (SECTION_SIZE, reach + 1);
    let span = size + 2 * margin;
    // The sky height of each column round it (each chunk's worked out once, until it changes).
    let mut heights = vec![i32::MAX; (span * span) as usize];
    let chunks = |low: i32| (low - margin).div_euclid(CHUNK_SIZE)..=(low + size - 1 + margin).div_euclid(CHUNK_SIZE);
    for cy in chunks(origin.y) {
        for cx in chunks(origin.x) {
            let pos = ChunkPos::new(cx, cy);
            let Some(chunk) = world.chunk(pos) else { continue };
            let column = sky.entry(pos).or_insert_with(|| sky_heights(chunk, is_opaque));
            for ly in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let (x, y) = (cx * CHUNK_SIZE + lx - origin.x + margin, cy * CHUNK_SIZE + ly - origin.y + margin);
                    if (0..span).contains(&x) && (0..span).contains(&y) {
                        heights[(x + y * span) as usize] = column[(lx + ly * CHUNK_SIZE) as usize];
                    }
                }
            }
        }
    }
    // (From a place relative to the section.)
    let height = |x: i32, y: i32| heights[((x + margin) + (y + margin) * span) as usize];
    let covered = |x: i32, y: i32, z: i32| origin.z + z < height(x, y);

    // Any covered air in the section, or just across its sides?
    let inside = (0..size).any(|z| (0..size).any(|y| (0..size).any(|x| !is_opaque(blocks[(x + y * size + z * size * size) as usize]) && covered(x, y, z))));
    let beside = || {
        (0..3).any(|axis| {
            let (u_axis, v_axis) = plane_axes(axis);
            [true, false].into_iter().any(|positive| {
                let face = face_index(axis, positive);
                (0..size).any(|v| {
                    (0..size).any(|u| {
                        let mut p = [0; 3];
                        (p[axis], p[u_axis], p[v_axis]) = (if positive { size } else { -1 }, u, v);
                        (outside[face][v as usize] >> u) & 1 == 0 && covered(p[0], p[1], p[2])
                    })
                })
            })
        })
    };
    if !inside && !beside() {
        return None;
    }

    // Each cell round it: solid or not, open to the sky or not, read a section at a time.
    let mut region = LightRegion::new(margin);
    let levels = (origin.z - margin).div_euclid(size)..=(origin.z + size - 1 + margin).div_euclid(size);
    for level in levels {
        for cy in chunks(origin.y) {
            for cx in chunks(origin.x) {
                let section = usize::try_from(level).ok().and_then(|level| world.chunk(ChunkPos::new(cx, cy)).and_then(|chunk| chunk.sections().get(level)));
                let above = level * size >= WORLD_HEIGHT;
                // This section's corner, relative to the one being meshed.
                let low = IVec3::new(cx * CHUNK_SIZE, cy * CHUNK_SIZE, level * size) - origin;
                for z in low.z.max(-margin)..(low.z + size).min(size + margin) {
                    for y in low.y.max(-margin)..(low.y + size).min(size + margin) {
                        for x in low.x.max(-margin)..(low.x + size).min(size + margin) {
                            let id = match section {
                                Some(Section::Uniform(id)) => *id,
                                Some(other) => other.get(index_of(IVec3::new(x, y, z) - low)),
                                // Past the top of the world: air. Below it, or not loaded: solid.
                                None if above => BlockId::AIR,
                                None => continue,
                            };
                            let i = region.index(IVec3::new(x, y, z));
                            let solid = is_opaque(id);
                            region.opaque[i] = solid;
                            region.open[i] = !solid && !covered(x, y, z);
                        }
                    }
                }
            }
        }
    }
    Some(region)
}

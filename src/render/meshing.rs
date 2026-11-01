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
use super::mesher::{MeshInput, MeshOutput, face_index, mesh_section, plane_axes};
use super::{RenderSettings, TerrainAssets};
use crate::coords::{bevy_to_game, game_to_bevy};
use crate::world::{
    BlockId, CHUNK_SIZE, SECTION_SIZE, SECTION_VOLUME, Section, SectionKey, VoxelWorld,
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
}

pub struct JobResult {
    output: MeshOutput,
    micros: u64,
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
        let section = world.chunk(key.chunk).map(|chunk| &chunk.sections()[key.level as usize]);
        match section {
            // A chunk that's gone (dropped behind you): just remove its meshes.
            None => {
                state.waiting.remove(&key);
                state.running.remove(&key);
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
    settings: Res<RenderSettings>,
    time: Res<Time<Real>>,
    mut state: ResMut<MeshingState>,
    camera: Query<&Transform, With<Camera3d>>,
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
        .map_or(Vec3::ZERO, |t| bevy_to_game(t.translation));
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
        match gather_input(&world, key, &terrain) {
            Some(input) => {
                let task = pool.spawn(async move {
                    let started = Instant::now();
                    let output = mesh_section(&input);
                    JobResult {
                        output,
                        micros: started.elapsed().as_micros() as u64,
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
    settings: Res<RenderSettings>,
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

        let quads = output.quads;
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
                continue;
            }
        }

        // Bounding box in Bevy's axes, used by frustum culling (game 0..32 → bevy x 0..32, y 0..32, z −32..0).
        let size = SECTION_SIZE as f32;
        let entity = commands
            .spawn((
                Mesh3d(handle),
                MeshMaterial3d::<TerrainMaterial>(terrain.material.clone()),
                Transform::from_translation(game_to_bevy(section_origin(key).as_vec3())),
                Aabb::from_min_max(Vec3::new(0.0, 0.0, -size), Vec3::new(size, size, 0.0)),
                SectionMesh {
                    key,
                    quads,
                    vertices: vertex_count,
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
fn gather_input(world: &VoxelWorld, key: SectionKey, terrain: &TerrainAssets) -> Option<MeshInput> {
    let chunk = world.chunk(key.chunk)?;
    let section = &chunk.sections()[key.level as usize];
    let is_opaque = |id: BlockId| id != BlockId::PIECES && terrain.opaque.get(id.0 as usize).copied().unwrap_or(true);
    let is_see_through = |id: BlockId| terrain.see_through.get(id.0 as usize).copied().unwrap_or(false);
    let is_grid = |id: BlockId| is_see_through(id) && terrain.inner_faces.get(id.0 as usize).copied().unwrap_or(false);
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
        pieces.push((index_of(local), cell.octants()));
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
        pieces,
        beyond_pieces,
    })
}

//! Meshing jobs — rebuilding section meshes on background threads.
//!
//! Every frame:
//! 1. **Collect** the sections the world marked as changed ("dirty") into a waiting list.
//! 2. **Start** jobs for the waiting sections nearest the camera, up to a per-frame limit.
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
use std::collections::{HashMap, HashSet};
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
    /// The entity drawing each section (sections with nothing to draw have none).
    pub entities: HashMap<SectionKey, Entity>,
    // ── Statistics for the debug panel ──
    pub jobs_finished: u64,
    pub total_job_micros: u64,
    pub slowest_job_micros: u64,
}

pub struct JobResult {
    output: MeshOutput,
    micros: u64,
}

pub fn collect_dirty_sections(mut world: ResMut<VoxelWorld>, mut state: ResMut<MeshingState>) {
    let dirty = world.take_dirty();
    state.waiting.extend(dirty);
}

pub fn start_jobs(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    terrain: Res<TerrainAssets>,
    settings: Res<RenderSettings>,
    mut state: ResMut<MeshingState>,
    camera: Query<&Transform, With<Camera3d>>,
) {
    if state.waiting.is_empty() {
        return;
    }

    // Nearest sections first, so what you're looking at appears first.
    let camera_position = camera
        .iter()
        .next()
        .map_or(Vec3::ZERO, |t| bevy_to_game(t.translation));
    let mut keys: Vec<SectionKey> = state.waiting.iter().copied().collect();
    keys.sort_by_key(|key| {
        let centre = section_origin(*key).as_vec3() + Vec3::splat(SECTION_SIZE as f32 / 2.0);
        centre.distance_squared(camera_position) as i64
    });

    let pool = AsyncComputeTaskPool::get();
    for key in keys.into_iter().take(settings.max_jobs_per_frame) {
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
    mut existing: Query<(&mut Mesh3d, &mut SectionMesh)>,
) {
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
}

/// The block at a section's south-west bottom corner.
pub fn section_origin(key: SectionKey) -> IVec3 {
    IVec3::new(
        key.chunk.x * CHUNK_SIZE,
        key.chunk.y * CHUNK_SIZE,
        key.level as i32 * SECTION_SIZE,
    )
}

/// Copy everything the mesher needs out of the world. `None` = nothing to draw.
fn gather_input(world: &VoxelWorld, key: SectionKey, terrain: &TerrainAssets) -> Option<MeshInput> {
    let chunk = world.chunk(key.chunk)?;
    let section = &chunk.sections()[key.level as usize];
    let is_opaque = |id: BlockId| terrain.opaque.get(id.0 as usize).copied().unwrap_or(true);
    if let Section::Uniform(id) = section {
        if !is_opaque(*id) {
            return None;
        }
    }

    let blocks: Vec<BlockId> = (0..SECTION_VOLUME).map(|i| section.get(i)).collect();

    // The one-block layer just outside each of the six sides.
    let origin = section_origin(key);
    let size = SECTION_SIZE;
    let mut outside = [[0u32; 32]; 6];
    for axis in 0..3 {
        let (u_axis, v_axis) = plane_axes(axis);
        for positive in [true, false] {
            let face = face_index(axis, positive);
            let depth = if positive { size } else { -1 };
            for v in 0..size {
                for u in 0..size {
                    let mut offset = IVec3::ZERO;
                    offset[axis] = depth;
                    offset[u_axis] = u;
                    offset[v_axis] = v;
                    let position = origin + offset;
                    // Below the world counts as solid: nobody can see the underside of bedrock.
                    let solid = position.z < 0 || is_opaque(world.get_block(position));
                    if solid {
                        outside[face][v as usize] |= 1 << u;
                    }
                }
            }
        }
    }

    Some(MeshInput {
        blocks,
        outside,
        opaque: terrain.opaque.clone(),
        face_layers: terrain.face_layers.clone(),
        face_random_rotation: terrain.face_random_rotation.clone(),
    })
}

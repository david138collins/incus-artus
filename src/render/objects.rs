//! Objects drawn — each placed object's model in the world [`docs/objects.md`, stop 1].
//!
//! - **The model** is the object's glTF (Blockbench's export, `assets/models/`). Its meshes are drawn
//!   one by one, each with its texture, as the blocks are drawn: flat, with no realistic lighting (not
//!   Bevy's glTF scene, which would bring it).
//! - **As made, or scaled to its height** (the object's `height`), standing with its feet's middle on
//!   its spot, as its shape was read (`data/objects.rs`), so what's drawn is what bodies bump into.
//! - **Lit like the blocks round it:** the daylight of the cell it stands in (`sky_light.rs`: open sky
//!   full, a short way in from it fading, black beyond), eased as the terrain shader eases it. Worked
//!   out when it's drawn, and again when the sections round it are made again.
//! - **Kept in step:** a chunk's objects are drawn again whenever they change (placed, taken), the
//!   chunk comes or goes, or the light round them changes; and once a model they need has loaded.

use super::RenderSettings;
use super::TerrainAssets;
use super::meshing::MeshingState;
use super::sky_light::{FULL, sky_heights};
use crate::coords::game_to_bevy;
use crate::data::ObjectRegistry;
use crate::world::{BlockId, CHUNK_SIZE, ChunkPos, PlacedObject, VoxelWorld, WORLD_HEIGHT};
use bevy::asset::LoadState;
use bevy::ecs::system::SystemParam;
use bevy::gltf::{Gltf, GltfMaterial, GltfMesh, GltfNode};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// One mesh of a model, where it sits in the model, and its texture.
struct Part {
    mesh: Handle<Mesh>,
    texture: Option<Handle<Image>>,
    place: Mat4,
}

/// A model: loading, ready (its parts, and its lowest and highest corners, in the model's own axes),
/// or not loadable.
enum Model {
    Loading(Handle<Gltf>),
    Ready { parts: Vec<Part>, low: Vec3, high: Vec3 },
    Failed,
}

/// What's drawn for the objects.
#[derive(Resource, Default)]
pub struct ObjectsDrawn {
    /// Each model, by its file name.
    models: HashMap<String, Model>,
    /// The entities drawing each chunk's objects.
    drawn: HashMap<ChunkPos, Vec<Entity>>,
    /// Chunks waiting for a model to load.
    waiting: HashSet<ChunkPos>,
    /// One material for each texture and level of light (16 at most for each texture).
    materials: HashMap<(Option<AssetId<Image>>, u8), Handle<StandardMaterial>>,
    /// The object registry's revision drawn with.
    revision: u64,
}

/// The glTF's own assets, read to take a model apart.
#[derive(SystemParam)]
pub struct GltfAssets<'w> {
    gltfs: Res<'w, Assets<Gltf>>,
    nodes: Res<'w, Assets<GltfNode>>,
    meshes: Res<'w, Assets<GltfMesh>>,
    materials: Res<'w, Assets<GltfMaterial>>,
}

#[allow(clippy::too_many_arguments)]
pub fn draw_objects(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    objects: Res<ObjectRegistry>,
    asset_server: Res<AssetServer>,
    gltf: GltfAssets,
    (meshes, mut materials): (Res<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    (terrain, settings): (Option<Res<TerrainAssets>>, Res<RenderSettings>),
    mut meshing: ResMut<MeshingState>,
    mut drawn: ResMut<ObjectsDrawn>,
) {
    let _timed = crate::render::Timed::new("draw objects");
    let Some(terrain) = terrain else { return };
    let drawn = &mut *drawn;
    // What to draw again: chunks whose objects changed, whose light changed, or that were waiting
    // for a model; and every one, if an object's settings changed.
    let mut again = world.take_objects_changed();
    again.extend(meshing.relight.drain());
    let waiting = std::mem::take(&mut drawn.waiting);
    again.extend(waiting);
    if drawn.revision != objects.revision() {
        drawn.revision = objects.revision();
        let all: Vec<ChunkPos> = drawn.drawn.keys().copied().collect();
        again.extend(all);
    }
    // Models that have finished loading are taken apart.
    for model in drawn.models.values_mut() {
        if let Model::Loading(handle) = model {
            if let Some(loaded) = gltf.gltfs.get(handle.id()) {
                *model = take_apart(loaded, &gltf, &meshes);
            } else if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
                *model = Model::Failed;
            }
        }
    }
    if again.is_empty() {
        return;
    }

    for pos in again {
        for entity in drawn.drawn.remove(&pos).unwrap_or_default() {
            commands.entity(entity).despawn();
        }
        let Some(chunk) = world.chunk(pos) else { continue };
        let placed: Vec<PlacedObject> = chunk.objects().to_vec();
        let mut entities = Vec::new();
        for object in &placed {
            let Some(def) = objects.by_name(&object.name) else { continue };
            let model = drawn.models.entry(def.model.clone()).or_insert_with(|| Model::Loading(asset_server.load(format!("models/{}", def.model))));
            let (parts, low, high) = match model {
                Model::Ready { parts, low, high } => (parts, *low, *high),
                Model::Loading(_) => {
                    drawn.waiting.insert(pos);
                    continue;
                }
                Model::Failed => continue,
            };
            // As made or scaled to its height, its feet's middle on its spot, turned to face its way
            // (a quarter turn each, as `world::turned` turns its shape): as its shape was read, so
            // what's drawn is what's bumped into.
            let (scale, feet) = match &def.shape {
                Some(shape) => (shape.scale, shape.feet),
                None => (def.height.map_or(1.0, |height| height / (high.y - low.y).max(0.001)), Vec3::new((low.x + high.x) / 2.0, low.y, (low.z + high.z) / 2.0)),
            };
            let spot = game_to_bevy(object.at.as_vec3() / 16.0);
            let turn = Quat::from_rotation_y(-((object.facing % 4) as f32) * std::f32::consts::FRAC_PI_2);
            let placing = Mat4::from_scale_rotation_translation(Vec3::splat(scale), turn, spot) * Mat4::from_translation(-feet);
            // Lit by its cell's daylight, eased as the terrain shader eases it.
            let level = if settings.sky_light { daylight_at(&world, &terrain.opaque, &mut meshing.sky, object.cell(), settings.sky_reach) } else { FULL };
            let brightness = (level as f32 / FULL as f32).powi(2);
            let parts: Vec<(Handle<Mesh>, Option<Handle<Image>>, Mat4)> = parts.iter().map(|part| (part.mesh.clone(), part.texture.clone(), part.place)).collect();
            for (mesh, texture, place) in parts {
                let material = drawn
                    .materials
                    .entry((texture.as_ref().map(Handle::id), level))
                    .or_insert_with(|| {
                        materials.add(StandardMaterial {
                            base_color: Color::srgb(brightness, brightness, brightness),
                            base_color_texture: texture,
                            unlit: true,
                            alpha_mode: AlphaMode::Mask(0.5),
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        })
                    })
                    .clone();
                entities.push(commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::from_matrix(placing * place))).id());
            }
        }
        if !entities.is_empty() {
            drawn.drawn.insert(pos, entities);
        }
    }


}

/// A loaded glTF taken apart: every mesh in its node tree, placed as the tree places it, with its
/// texture; and the corners of the box round them all.
fn take_apart(loaded: &Gltf, gltf: &GltfAssets, meshes: &Assets<Mesh>) -> Model {
    // The nodes no other node holds are the roots.
    let children: HashSet<AssetId<GltfNode>> = loaded.nodes.iter().filter_map(|handle| gltf.nodes.get(handle.id())).flat_map(|node| node.children.iter().map(Handle::id)).collect();
    let mut stack: Vec<(Handle<GltfNode>, Mat4)> = loaded.nodes.iter().filter(|handle| !children.contains(&handle.id())).map(|handle| (handle.clone(), Mat4::IDENTITY)).collect();
    let mut parts = Vec::new();
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    while let Some((handle, above)) = stack.pop() {
        let Some(node) = gltf.nodes.get(handle.id()) else { continue };
        let place = above * node.transform.to_matrix();
        stack.extend(node.children.iter().map(|child| (child.clone(), place)));
        let Some(mesh) = node.mesh.as_ref().and_then(|mesh| gltf.meshes.get(mesh.id())) else { continue };
        for primitive in &mesh.primitives {
            let texture = primitive.material.as_ref().and_then(|material| gltf.materials.get(material.id())).and_then(|material| material.base_color_texture.clone());
            if let Some(VertexAttributeValues::Float32x3(corners)) = meshes.get(primitive.mesh.id()).and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION)) {
                for corner in corners {
                    let at = place.transform_point3(Vec3::from_array(*corner));
                    (low, high) = (low.min(at), high.max(at));
                }
            }
            parts.push(Part { mesh: primitive.mesh.clone(), texture, place });
        }
    }
    if parts.is_empty() || low.y > high.y {
        return Model::Failed;
    }
    Model::Ready { parts, low, high }
}

/// The daylight (0 to 15) at a cell, as the blocks round it get it (`sky_light.rs`): the nearest
/// cell open to the sky within `reach` steps through air, full there and an even step darker for each
/// step; black if none.
fn daylight_at(world: &VoxelWorld, opaque: &[bool], sky: &mut HashMap<ChunkPos, Vec<i32>>, cell: IVec3, reach: i32) -> u8 {
    let reach = reach.clamp(1, super::sky_light::MOST_REACH);
    let is_opaque = |id: BlockId| id != BlockId::PIECES && opaque.get(id.0 as usize).copied().unwrap_or(true);
    let mut height = |x: i32, y: i32| -> i32 {
        let pos = ChunkPos::of_block(IVec3::new(x, y, 0));
        let Some(chunk) = world.chunk(pos) else { return i32::MAX };
        let column = sky.entry(pos).or_insert_with(|| sky_heights(chunk, &is_opaque));
        column[(x.rem_euclid(CHUNK_SIZE) + y.rem_euclid(CHUNK_SIZE) * CHUNK_SIZE) as usize]
    };
    let mut queue = VecDeque::from([(cell, 0)]);
    let mut seen = HashSet::from([cell]);
    while let Some((at, steps)) = queue.pop_front() {
        if at.z >= WORLD_HEIGHT || at.z > height(at.x, at.y) {
            return (FULL as i32 * (reach - steps) / reach) as u8;
        }
        if steps == reach {
            continue;
        }
        for step in [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z] {
            let next = at + step;
            if next.z >= 0 && !is_opaque(world.get_block(next)) && seen.insert(next) {
                queue.push_back((next, steps + 1));
            }
        }
    }
    0
}

//! Objects drawn — each placed object's model in the world, its flame, and its glow [`docs/objects.md`,
//! stops 1 and 2].
//!
//! - **The model** is the object's glTF (Blockbench's export, `assets/models/`). Its meshes are drawn
//!   one by one, each with its texture, as the blocks are drawn: flat, with no realistic lighting (not
//!   Bevy's glTF scene, which would bring it). Every object's model is loaded as soon as the game
//!   knows of it, so the first one placed shows at once.
//! - **As made, or scaled to its height** (the object's `height`), standing with its feet's middle on
//!   its spot, as its shape was read (`data/objects.rs`), so what's drawn is what bodies bump into.
//! - **Its sprite** (a brazier's flame): a strip of frames (`read_strip`, the strip marking its own
//!   frames), on its spot (a named point in the model, or on top), played at its milliseconds a frame,
//!   turning about the upright to face you as Doom's flames do, and full bright. Each one starts on its
//!   own frame (from its place), so a row of braziers doesn't flicker in step.
//! - **Its glow:** the light it gives, in its colour, from its sprite's spot. For now the nearest 8
//!   glows to the camera go to the terrain shader (`Glows`), each fading with distance and passing
//!   through walls: a stand-in, until light spreads through the air (stop 3).
//! - **Lit like the blocks round it:** the daylight of the cell it stands in (`sky_light.rs`: open sky
//!   full, a short way in from it fading, black beyond), eased as the terrain shader eases it, and the
//!   glows round it. Worked out when it's drawn, and again when the sections or glows round it change.
//! - **Kept in step:** a chunk's objects are drawn again whenever they change (placed, taken), the
//!   chunk comes or goes, or the light round them changes; and once a model they need has loaded.

use super::RenderSettings;
use super::TerrainAssets;
use super::meshing::MeshingState;
use super::sky_light::{FULL, sky_heights};
use crate::camera::FlyCamera;
use crate::coords::game_to_bevy;
use crate::data::{Anchor, ObjectRegistry, read_strip, sprites_folder};
use crate::world::{BlockId, CHUNK_SIZE, ChunkPos, PlacedObject, VoxelWorld, WORLD_HEIGHT, turned};
use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::ecs::system::SystemParam;
use bevy::gltf::{Gltf, GltfMaterial, GltfMesh, GltfNode};
use bevy::image::ImageSampler;
use bevy::math::Affine2;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
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

/// A sprite strip, ready to draw: one material for each frame (the strip, slid to that frame), and
/// its frames' size in pixels (the first frame's).
struct StripDrawn {
    frames: Vec<Handle<StandardMaterial>>,
    size: UVec2,
}

/// The light an object gives: where (the game's axes, blocks), how far, and its colour times its
/// brightness (linear).
#[derive(Clone, Copy, PartialEq)]
struct Glow {
    at: Vec3,
    reach: f32,
    colour: Vec3,
}

/// What's drawn for the objects.
#[derive(Resource, Default)]
pub struct ObjectsDrawn {
    /// Each model, by its file name.
    models: HashMap<String, Model>,
    /// Each sprite strip, by its file name (none: it couldn't be read).
    strips: HashMap<String, Option<StripDrawn>>,
    /// Sprite quads, by their size (pixels, scaled, as bits) and anchor.
    quads: HashMap<(u32, u32, Anchor), Handle<Mesh>>,
    /// The entities drawing each chunk's objects.
    drawn: HashMap<ChunkPos, Vec<Entity>>,
    /// The glows each chunk's objects give.
    lights: HashMap<ChunkPos, Vec<Glow>>,
    /// Chunks waiting for a model to load, or to be lit again by a glow beside them.
    waiting: HashSet<ChunkPos>,
    /// One material for each texture and colour of light (16 levels each of red, green, blue).
    materials: HashMap<(Option<AssetId<Image>>, [u8; 3]), Handle<StandardMaterial>>,
    /// The object registry's revision drawn with.
    revision: Option<u64>,
}

/// The glows sent to the terrain shader: the nearest 8 to the camera (`underground_light` in
/// `terrain.wesl`).
#[derive(Resource, Default, PartialEq)]
pub struct Glows {
    pub at: [Vec4; 8],
    pub colour: [Vec4; 8],
}

/// A sprite playing: its frames, how long each shows, and which it starts on.
#[derive(Component)]
pub struct Playing {
    frames: Vec<Handle<StandardMaterial>>,
    frame_ms: u32,
    phase: u32,
}

/// The glTF's own assets, read to take a model apart.
#[derive(SystemParam)]
pub struct GltfAssets<'w> {
    gltfs: Res<'w, Assets<Gltf>>,
    nodes: Res<'w, Assets<GltfNode>>,
    meshes: Res<'w, Assets<GltfMesh>>,
    materials: Res<'w, Assets<GltfMaterial>>,
}

/// The light the glows give at a place (linear, each colour up to 1), as the terrain shader works it
/// out.
fn glow_at(lights: &HashMap<ChunkPos, Vec<Glow>>, at: Vec3) -> Vec3 {
    lights
        .values()
        .flatten()
        .map(|glow| {
            let near = 1.0 - (at.distance(glow.at) / glow.reach.max(0.01)).clamp(0.0, 1.0);
            glow.colour * near * near
        })
        .sum()
}

/// A sprite strip read from `assets/textures/sprites/` and made ready to draw: its picture (sharp
/// pixels), and a material for each frame, full bright, its clear pixels cut out.
fn load_strip(file: &str, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> Option<StripDrawn> {
    let strip = std::fs::read(sprites_folder().join(file)).map_err(|error| error.to_string()).and_then(|bytes| read_strip(&bytes));
    let strip = match strip {
        Ok(strip) => strip,
        Err(error) => {
            warn!("Sprite strip {file} couldn't be read: {error}");
            return None;
        }
    };
    let mut image = Image::new(Extent3d { width: strip.width, height: strip.height, depth_or_array_layers: 1 }, TextureDimension::D2, strip.rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.sampler = ImageSampler::nearest();
    let image = images.add(image);
    let width = strip.width as f32;
    let frames = strip
        .frames
        .iter()
        .map(|&(left, across)| {
            materials.add(StandardMaterial {
                base_color_texture: Some(image.clone()),
                uv_transform: Affine2::from_scale_angle_translation(Vec2::new(across as f32 / width, 1.0), 0.0, Vec2::new(left as f32 / width, 0.0)),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                double_sided: true,
                cull_mode: None,
                ..default()
            })
        })
        .collect();
    Some(StripDrawn { frames, size: UVec2::new(strip.frames.first().map_or(strip.height, |frame| frame.1), strip.height) })
}

#[allow(clippy::too_many_arguments)]
pub fn draw_objects(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    objects: Res<ObjectRegistry>,
    asset_server: Res<AssetServer>,
    gltf: GltfAssets,
    (mut meshes, mut materials, mut images): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>, ResMut<Assets<Image>>),
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
    if drawn.revision != Some(objects.revision()) {
        drawn.revision = Some(objects.revision());
        let all: Vec<ChunkPos> = drawn.drawn.keys().chain(drawn.lights.keys()).copied().collect();
        again.extend(all);
        // Every model loaded now, before it's placed; strips read again (one may have changed).
        for def in objects.defs() {
            drawn.models.entry(def.model.clone()).or_insert_with(|| Model::Loading(asset_server.load(format!("models/{}", def.model))));
        }
        drawn.strips.clear();
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
        let before = drawn.lights.remove(&pos).unwrap_or_default();
        let Some(chunk) = world.chunk(pos) else {
            if !before.is_empty() {
                drawn.waiting.extend(beside(pos));
            }
            continue;
        };
        let placed: Vec<PlacedObject> = chunk.objects().to_vec();
        // Each object's feet and its sprite's spot (the game's axes, blocks), and the glows they give,
        // first: an object is lit by its own glow too.
        let spots: Vec<(Vec3, Vec3)> = placed
            .iter()
            .map(|object| {
                let feet = object.at.as_vec3() / 16.0;
                let def = objects.by_name(&object.name);
                let shape = def.map(|def| def.shape_or_block());
                let spot = match (def.and_then(|def| def.sprite.as_ref()), &shape) {
                    (Some(sprite), Some(shape)) => shape.spot(sprite),
                    (None, Some(shape)) => Vec3::new(0.0, 0.0, shape.size.z as f32 / 32.0),
                    _ => Vec3::ZERO,
                };
                (feet, feet + turned(spot, object.facing))
            })
            .collect();
        let lights: Vec<Glow> = placed
            .iter()
            .zip(&spots)
            .filter_map(|(object, &(_, spot))| {
                let light = objects.by_name(&object.name)?.light.as_ref()?;
                let colour = Color::srgb_u8(light.colour[0], light.colour[1], light.colour[2]).to_linear();
                Some(Glow { at: spot, reach: light.reach, colour: Vec3::new(colour.red, colour.green, colour.blue) * light.brightness })
            })
            .collect();
        if lights != before {
            drawn.waiting.extend(beside(pos).filter(|&next| next != pos));
        }
        if !lights.is_empty() {
            drawn.lights.insert(pos, lights);
        }
        let mut entities = Vec::new();
        for (object, &(feet, spot)) in placed.iter().zip(&spots) {
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
            let (scale, model_feet) = match &def.shape {
                Some(shape) => (shape.scale, shape.feet),
                None => (def.height.map_or(1.0, |height| height / (high.y - low.y).max(0.001)), Vec3::new((low.x + high.x) / 2.0, low.y, (low.z + high.z) / 2.0)),
            };
            let turn = Quat::from_rotation_y(-((object.facing % 4) as f32) * std::f32::consts::FRAC_PI_2);
            let placing = Mat4::from_scale_rotation_translation(Vec3::splat(scale), turn, game_to_bevy(feet)) * Mat4::from_translation(-model_feet);
            // Lit by its cell's daylight, eased as the terrain shader eases it, and the glows round
            // it (its own among them), in 16 steps of each colour.
            let level = if settings.sky_light { daylight_at(&world, &terrain.opaque, &mut meshing.sky, object.cell(), settings.sky_reach) } else { FULL };
            let daylight = (level as f32 / FULL as f32).powi(2);
            let middle = feet + Vec3::Z * def.shape_or_block().size.z as f32 / 32.0;
            let light = (Vec3::splat(daylight) + glow_at(&drawn.lights, middle)).min(Vec3::ONE);
            let steps = (light * 15.0).round().as_uvec3();
            let key = [steps.x as u8, steps.y as u8, steps.z as u8];
            let parts: Vec<(Handle<Mesh>, Option<Handle<Image>>, Mat4)> = parts.iter().map(|part| (part.mesh.clone(), part.texture.clone(), part.place)).collect();
            for (mesh, texture, place) in parts {
                let material = drawn
                    .materials
                    .entry((texture.as_ref().map(Handle::id), key))
                    .or_insert_with(|| {
                        materials.add(StandardMaterial {
                            base_color: Color::LinearRgba(LinearRgba::new(steps.x as f32 / 15.0, steps.y as f32 / 15.0, steps.z as f32 / 15.0, 1.0)),
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
            // Its sprite: on its spot, full bright, playing.
            let Some(sprite) = &def.sprite else { continue };
            let strip = drawn.strips.entry(sprite.file.clone()).or_insert_with(|| load_strip(&sprite.file, &mut images, &mut materials));
            let Some(strip) = strip else { continue };
            let size = strip.size.as_vec2() * sprite.scale / 16.0;
            let quad = drawn
                .quads
                .entry((size.x.to_bits(), size.y.to_bits(), sprite.anchor))
                .or_insert_with(|| {
                    let lift = match sprite.anchor {
                        Anchor::BottomCentre => size.y / 2.0,
                        Anchor::Centre => 0.0,
                        Anchor::TopCentre => -size.y / 2.0,
                    };
                    meshes.add(Mesh::from(Rectangle::new(size.x, size.y)).translated_by(Vec3::Y * lift))
                })
                .clone();
            let phase = (object.at.x.wrapping_mul(73_856_093) ^ object.at.y.wrapping_mul(19_349_663) ^ object.at.z.wrapping_mul(83_492_791)).unsigned_abs();
            let playing = Playing { frames: strip.frames.clone(), frame_ms: sprite.frame_ms.max(1), phase };
            let first = strip.frames.first().cloned().unwrap_or_default();
            entities.push(commands.spawn((Mesh3d(quad), MeshMaterial3d(first), Transform::from_translation(game_to_bevy(spot)), playing)).id());
        }
        if !entities.is_empty() {
            drawn.drawn.insert(pos, entities);
        }
    }
}

/// A chunk and the eight round it.
fn beside(pos: ChunkPos) -> impl Iterator<Item = ChunkPos> {
    (-1..=1).flat_map(move |y| (-1..=1).map(move |x| ChunkPos::of_block(IVec3::new((pos.x + x) * CHUNK_SIZE, (pos.y + y) * CHUNK_SIZE, 0))))
}

/// The nearest 8 glows to the camera, for the terrain shader; changed only when they change.
pub fn choose_glows(drawn: Res<ObjectsDrawn>, cameras: Query<&FlyCamera>, mut glows: ResMut<Glows>) {
    let Some(camera) = cameras.iter().next() else { return };
    let mut all: Vec<&Glow> = drawn.lights.values().flatten().collect();
    all.sort_by(|a, b| a.at.distance_squared(camera.position).total_cmp(&b.at.distance_squared(camera.position)));
    let mut chosen = Glows::default();
    for (i, glow) in all.into_iter().take(8).enumerate() {
        chosen.at[i] = game_to_bevy(glow.at).extend(glow.reach);
        chosen.colour[i] = glow.colour.extend(1.0);
    }
    glows.set_if_neq(chosen);
}

/// Sprites play: each shows the frame for the time (from its own frame), and turns about the upright
/// to face the camera.
pub fn play_sprites(time: Res<Time>, cameras: Query<&FlyCamera>, mut sprites: Query<(&Playing, &mut MeshMaterial3d<StandardMaterial>, &mut Transform)>) {
    let Some(camera) = cameras.iter().next() else { return };
    let eye = game_to_bevy(camera.position);
    let now = (time.elapsed_secs_f64() * 1000.0) as u64;
    for (playing, mut material, mut transform) in &mut sprites {
        if !playing.frames.is_empty() {
            let frame = ((now / playing.frame_ms as u64 + playing.phase as u64) % playing.frames.len() as u64) as usize;
            if material.0 != playing.frames[frame] {
                material.0 = playing.frames[frame].clone();
            }
        }
        let toward = eye - transform.translation;
        transform.rotation = Quat::from_rotation_y(toward.x.atan2(toward.z));
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

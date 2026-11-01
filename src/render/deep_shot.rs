//! Deep screenshot — a picture of everything in view, out to the edge of the world, at the detail of
//! the ground up close (Camera debug page, or Shift + F12). [David, 2026-10-28: a way to see the far
//! mountains as they really are, until the graphics work.]
//!
//! Past about a thousand blocks a block is smaller than a pixel, so modelling every block would only
//! fill memory with detail no pixel can show. Instead the distant terrain's tiles are made again for
//! the picture with wider rings, so each ring's cells are about a pixel across at its distance (one
//! block nearest, then 2, 4 …), and only the tiles inside the camera's view. The usual distant tiles
//! are hidden meanwhile, the camera holds still, and a magenta bar shows how far along it is. Once
//! every tile is in and the near chunks are all meshed, the picture is taken with the panels left out
//! of that frame (the PNG is the window's size), then the tiles go and everything is as it was.

use super::RenderSettings;
use super::distant::{self, DistantState, DistantTile, Palette, TileKey, TileMesh};
use super::meshing::MeshingState;
use crate::camera::CameraHold;
use crate::coords::{bevy_to_game, game_to_bevy};
use crate::data::BlockRegistry;
use crate::screen::Screen;
use crate::world::{CHUNK_SIZE, ChunkPos, CompiledMap, Streaming, Terrain, TerrainBlocks, VoxelWorld};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiRenderOutput, egui};
use std::collections::HashMap;
use std::sync::Arc;

/// Cells along a tile's side (as the distant terrain's).
use super::distant::CELLS;
/// The usual rings' finest cell: slopes are measured over this, so the picture is shaded as they are.
const SLOPE_CELL: i32 = 16;
/// The widest rings (tiles along a side).
const MOST_ACROSS: i32 = 256;
/// Frames the panels are left out for, round the one the picture is taken in.
const CLEAN_FRAMES: u32 = 5;
/// The longest to wait for the near chunks to finish meshing (seconds).
const LONGEST_WAIT: f32 = 30.0;

/// Tunables (Camera debug page).
#[derive(Resource, Clone, Debug)]
pub struct DeepShotSettings {
    /// Cells per pixel at each ring's nearest: 1 = about a pixel each; more = finer (more tiles,
    /// slower), less = coarser.
    pub detail: f32,
}

impl Default for DeepShotSettings {
    fn default() -> Self {
        Self { detail: 1.0 }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum Phase {
    #[default]
    Idle,
    /// Making the tiles.
    Making,
    /// Every tile in; waiting for the near chunks to finish meshing (seconds waited).
    Settling(f32),
    /// The panels left out; the picture taken on the second of these frames.
    Capturing(u32),
}

#[derive(Resource, Default)]
pub struct DeepShot {
    phase: Phase,
    /// Asked for (by the Camera page or the key), started on the next frame.
    requested: bool,
    stop: bool,
    map: Option<Arc<CompiledMap>>,
    palette: Option<std::sync::Arc<Palette>>,
    wanted: Vec<TileKey>,
    making: HashMap<TileKey, Task<Option<TileMesh>>>,
    tiles: Vec<Entity>,
    total: usize,
    done: usize,
    quads: u64,
    started: f32,
    /// What happened last (shown on the Camera page).
    pub last: String,
}

impl DeepShot {
    pub fn busy(&self) -> bool {
        self.phase != Phase::Idle
    }

    /// Asks for a deep screenshot (taken from the next frame on).
    pub fn request(&mut self) {
        if !self.busy() {
            self.requested = true;
        }
    }

    pub fn stop(&mut self) {
        self.stop = true;
    }
}

/// How many tiles along a ring's side keep each ring's cells within `detail` cells per pixel at
/// its nearest, for a pixel `pixel` radians across: a ring's hole is a quarter of its tiles out
/// from you each way (less up to one tile, where you stand in it), and its cells must be no bigger
/// than a pixel there.
pub fn rings_across(pixel: f32, detail: f32) -> i32 {
    let tiles_out = (detail.max(0.1) / (CELLS as f32 * pixel.max(1e-6))).ceil() as i32 + 1;
    (4 * tiles_out).clamp(8, MOST_ACROSS)
}

/// The tiles for a deep screenshot from `at` (game axes) looking along `looking`, `half_across`
/// radians either side of it (left to right): rings `across` tiles wide, from one-block cells, out
/// past `reach` blocks; only the tiles in view, and none under `near` (real chunks already drawn,
/// whose finest tiles are only ever hidden under them).
pub fn deep_tiles(at: Vec3, looking: Vec3, half_across: f32, across: i32, reach: f32, near: &dyn Fn(TileKey) -> bool) -> Vec<TileKey> {
    let mut rings = 1;
    while rings < 20 && (across / 2 * CELLS) as f32 * 2f32.powi(rings as i32 - 1) < reach {
        rings += 1;
    }
    let flat = Vec2::new(looking.x, looking.y);
    // Looking almost straight up or down: every way round is in view.
    let every_way = flat.length() < 0.3 || half_across >= std::f32::consts::PI;
    let forward = flat.normalize_or_zero();
    distant::wanted_tiles_across(at, 1, rings, across)
        .into_iter()
        .filter(|&key| {
            if key.ring == 0 && near(key) {
                return false;
            }
            if every_way {
                return true;
            }
            let size = (CELLS << key.ring) as f32;
            let middle = Vec2::new((key.x as f32 + 0.5) * size, (key.y as f32 + 0.5) * size);
            let to = middle - Vec2::new(at.x, at.y);
            let (distance, radius) = (to.length(), size * std::f32::consts::FRAC_1_SQRT_2);
            if distance <= radius {
                return true;
            }
            let off = to.normalize().dot(forward).clamp(-1.0, 1.0).acos();
            off - (radius / distance).min(1.0).asin() <= half_across
        })
        .collect()
}

/// Starts, runs and finishes a deep screenshot (see the module notes).
#[allow(clippy::too_many_arguments)]
pub fn run_deep_shot(
    mut commands: Commands,
    (terrain, registry, screen): (Res<Terrain>, Res<BlockRegistry>, Res<State<Screen>>),
    (render, settings): (Res<RenderSettings>, Res<DeepShotSettings>),
    (meshing, world, streaming, distant_state): (Res<MeshingState>, Res<VoxelWorld>, Res<Streaming>, Res<DistantState>),
    camera: Query<(&Transform, &Projection), With<Camera3d>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut normal: Query<&mut Visibility, With<DistantTile>>,
    mut shot: ResMut<DeepShot>,
    mut hold: ResMut<CameraHold>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time<Real>>,
) {
    let in_play = *screen.get() == Screen::Play;
    if shot.busy() && (shot.stop || !in_play || terrain.compiled().is_none_or(|map| shot.map.as_ref().is_none_or(|old| !Arc::ptr_eq(old, map)))) {
        finish(&mut commands, &mut shot, &mut hold, &mut normal);
        shot.last = "Stopped.".into();
        return;
    }
    shot.stop = false;
    if shot.requested {
        shot.requested = false;
        if !in_play {
            return;
        }
        let Some(map) = terrain.compiled().cloned() else {
            shot.last = "Only in a world made from the World Compiler (Render World and Spawn).".into();
            return;
        };
        let (Some((transform, projection)), Ok(window)) = (camera.iter().next(), windows.single()) else { return };
        let fov = match projection {
            Projection::Perspective(p) => p.fov,
            _ => 1.0,
        };
        let (height, aspect) = (window.physical_height().max(1) as f32, window.width() / window.height().max(1.0));
        let pixel = 2.0 * (fov / 2.0).tan() / height;
        let across = rings_across(pixel, settings.detail);
        // Left to right, half the view (with a little to spare for the Panini projection's edges).
        let half_across = ((fov / 2.0).tan() * aspect).atan() * 1.15;
        let at = bevy_to_game(transform.translation);
        let looking = bevy_to_game(*transform.forward());
        let half_world = map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0;
        let reach = [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)].iter().map(|&(sx, sy)| Vec2::new(at.x - sx * half_world, at.y - sy * half_world).length()).fold(0.0, f32::max);
        // A finest tile is a chunk: none needed where it and the chunks round it are already drawn.
        let drawn = streaming.drawn();
        let centre = IVec2::new((at.x / CHUNK_SIZE as f32).floor() as i32, (at.y / CHUNK_SIZE as f32).floor() as i32);
        let limit = render.render_distance_chunks - 1;
        let near = |key: TileKey| {
            let offset = IVec2::new(key.x, key.y) - centre;
            offset.x.abs().max(offset.y.abs()) <= limit && (-1..=1).all(|dx| (-1..=1).all(|dy| drawn.contains(&ChunkPos::new(key.x + dx, key.y + dy))))
        };
        let mut wanted = deep_tiles(at, looking, half_across, across, reach, &near);
        // Nearest last (made first: the pop is from the end).
        let distance = |key: &TileKey| {
            let size = (CELLS << key.ring) as f32;
            Vec2::new((key.x as f32 + 0.5) * size, (key.y as f32 + 0.5) * size).distance(Vec2::new(at.x, at.y))
        };
        wanted.sort_by(|a, b| distance(b).total_cmp(&distance(a)));
        let blocks = TerrainBlocks::from_registry(&registry);
        shot.palette = Some(std::sync::Arc::new(Palette::from_textures(&registry, &blocks)));
        shot.total = wanted.len();
        shot.done = 0;
        shot.quads = 0;
        shot.wanted = wanted;
        shot.map = Some(map);
        shot.started = time.elapsed_secs();
        shot.phase = Phase::Making;
        shot.last = String::new();
        hold.held = true;
        info!("Deep screenshot: {} tiles in {across}-tile rings (a pixel is {:.5} radians across)", shot.total, pixel);
    }
    if !shot.busy() {
        return;
    }
    // The usual distant tiles stay hidden while the picture's are up (any made meanwhile too).
    for mut visibility in &mut normal {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
    match shot.phase {
        Phase::Idle => {}
        Phase::Making => {
            let mut done = Vec::new();
            shot.making.retain(|&key, task| match check_ready(task) {
                Some(mesh) => {
                    done.push((key, mesh));
                    false
                }
                None => true,
            });
            let material = distant_state.material.clone();
            for (_, mesh) in done {
                shot.done += 1;
                let (Some(tile), Some(material)) = (mesh, material.clone()) else { continue };
                shot.quads += (tile.indices.len() / 6) as u64;
                let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
                bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tile.positions);
                bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, tile.colours);
                bevy_mesh.insert_attribute(super::distant::ATTRIBUTE_EASE, tile.morph);
                bevy_mesh.insert_attribute(super::distant::ATTRIBUTE_SIDE, tile.sides);
                bevy_mesh.insert_attribute(super::distant::ATTRIBUTE_SLOPE, tile.slopes);
                bevy_mesh.insert_indices(Indices::U32(tile.indices));
                let entity = commands.spawn((Mesh3d(meshes.add(bevy_mesh)), MeshMaterial3d(material), Transform::from_translation(game_to_bevy(tile.corner)), DeepTile)).id();
                shot.tiles.push(entity);
            }
            let most = std::thread::available_parallelism().map_or(4, |n| n.get());
            let pool = AsyncComputeTaskPool::get();
            let (Some(map), Some(palette)) = (shot.map.clone(), shot.palette.clone()) else { return };
            let blocks = TerrainBlocks::from_registry(&registry);
            while shot.making.len() < most {
                let Some(key) = shot.wanted.pop() else { break };
                let map = map.clone();
                let (palette, blocks) = (palette.clone(), blocks.clone());
                let task = pool.spawn(async move { distant::tile_mesh_measured(&map, &blocks, &palette, key, 1, SLOPE_CELL, false) });
                shot.making.insert(key, task);
            }
            if shot.wanted.is_empty() && shot.making.is_empty() {
                shot.phase = Phase::Settling(0.0);
            }
        }
        Phase::Settling(waited) => {
            let meshed = meshing.waiting.is_empty() && meshing.running.is_empty() && meshing.ready.is_empty() && world.dirty_sections().next().is_none();
            // A moment for the last meshes to reach the screen, and no longer than the longest wait.
            let waited = waited + time.delta_secs();
            shot.phase = if (meshed && waited > 0.5) || waited > LONGEST_WAIT { Phase::Capturing(0) } else { Phase::Settling(waited) };
        }
        Phase::Capturing(frame) => {
            if frame == 1 {
                let folder = crate::debug::project_folder().join("screenshots");
                let _ = std::fs::create_dir_all(&folder);
                let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
                let path = folder.join(format!("deep-{seconds}.png"));
                shot.last = format!(
                    "Saved {} ({} tiles, {:.1} million quads, {:.0} s).",
                    path.display(),
                    shot.total,
                    shot.quads as f64 / 1e6,
                    time.elapsed_secs() - shot.started
                );
                info!("Deep screenshot: {}", shot.last);
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            }
            if frame + 1 >= CLEAN_FRAMES {
                let last = std::mem::take(&mut shot.last);
                finish(&mut commands, &mut shot, &mut hold, &mut normal);
                shot.last = last;
            } else {
                shot.phase = Phase::Capturing(frame + 1);
            }
        }
    }
}

/// A tile of the picture's.
#[derive(Component)]
pub struct DeepTile;

/// Takes the picture's tiles away and puts everything back as it was.
fn finish(commands: &mut Commands, shot: &mut DeepShot, hold: &mut CameraHold, normal: &mut Query<&mut Visibility, With<DistantTile>>) {
    for entity in shot.tiles.drain(..) {
        commands.entity(entity).despawn();
    }
    shot.making.clear();
    shot.wanted.clear();
    shot.map = None;
    shot.phase = Phase::Idle;
    shot.stop = false;
    hold.held = false;
    for mut visibility in normal.iter_mut() {
        *visibility = Visibility::Inherited;
    }
}

/// Leaves the panels out of the frames round the picture (after egui has worked out what to draw,
/// before it's drawn).
pub fn leave_out_panels(shot: Res<DeepShot>, mut outputs: Query<&mut EguiRenderOutput>) {
    if !matches!(shot.phase, Phase::Capturing(_)) {
        return;
    }
    for mut output in &mut outputs {
        output.paint_jobs.clear();
    }
}

/// Shift + F12 takes a deep screenshot.
pub fn deep_shot_key(keys: Res<ButtonInput<KeyCode>>, mut shot: ResMut<DeepShot>) {
    if keys.just_pressed(KeyCode::F12) && keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        shot.request();
    }
}

/// The magenta bar while a deep screenshot is being made.
pub fn draw_deep_shot_bar(mut contexts: EguiContexts, shot: Res<DeepShot>) -> Result {
    if !shot.busy() {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let (share, text) = match shot.phase {
        Phase::Settling(_) | Phase::Capturing(_) => (1.0, "Deep screenshot: waiting for the near ground, then the picture".to_string()),
        _ => (shot.done as f32 / shot.total.max(1) as f32, format!("Deep screenshot: making the ground, {} of {} tiles", shot.done, shot.total)),
    };
    egui::Area::new(egui::Id::new("deep shot bar")).anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 40.0)).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.set_width(460.0);
            ui.add(egui::ProgressBar::new(share).fill(egui::Color32::from_rgb(255, 0, 255)).text(text));
            ui.small("The camera holds still until it's taken. Stop it on the Camera page.");
        });
    });
    Ok(())
}

/// The Camera page's section.
pub fn deep_shot_section(ui: &mut egui::Ui, world: &mut World) {
    ui.strong("Deep screenshot");
    ui.small(
        "Everything in view out to the edge of the world, as detailed as a pixel can show (the far ground as fine as the \
         near, no ziggurats), saved in screenshots/ at the window's size. Takes a while; the camera holds still meanwhile.",
    );
    {
        let mut settings = world.resource_mut::<DeepShotSettings>();
        ui.add(crate::debug::Slider::new(&mut settings.detail, 0.25..=2.0).text("detail (cells per pixel)")).on_hover_text(
            "1 = each far cell about a pixel across where it's nearest (and finer further in). Higher = finer still,\n\
             with many more tiles (slower, more memory); lower = quicker and coarser.",
        );
    }
    let mut shot = world.resource_mut::<DeepShot>();
    ui.horizontal(|ui| {
        if shot.busy() {
            if ui.button("Stop").clicked() {
                shot.stop();
            }
            ui.label(format!("{} of {} tiles", shot.done, shot.total));
        } else if ui.button("Take a deep screenshot (Shift + F12)").clicked() {
            shot.request();
        }
    });
    if !shot.last.is_empty() {
        ui.small(shot.last.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rings_are_wide_enough_for_a_cell_a_pixel_and_capped() {
        // 100° up-down over 1,080 pixels.
        let pixel = 2.0 * 50f32.to_radians().tan() / 1080.0;
        let across = rings_across(pixel, 1.0);
        assert_eq!(across % 4, 0);
        // A ring's nearest (a quarter of its tiles out, less one) sees its cells no bigger than a pixel.
        let nearest = ((across / 4 - 1) * CELLS) as f32;
        assert!(1.0 / nearest <= pixel, "{across} across: a cell is {} radians at its nearest", 1.0 / nearest);
        assert_eq!(rings_across(1e-6, 2.0), MOST_ACROSS);
    }

    #[test]
    fn a_deep_screenshot_makes_the_tiles_in_view_out_to_the_world_edge() {
        let at = Vec3::new(500.0, -300.0, 200.0);
        let never_near = |_: TileKey| false;
        let tiles = deep_tiles(at, Vec3::X, 0.8, 16, 40_000.0, &never_near);
        let middle = |key: &TileKey| {
            let size = (CELLS << key.ring) as f32;
            Vec2::new((key.x as f32 + 0.5) * size, (key.y as f32 + 0.5) * size) - Vec2::new(at.x, at.y)
        };
        // Far ahead (north) is covered, far behind is not.
        assert!(tiles.iter().any(|key| middle(key).x > 35_000.0));
        assert!(!tiles.iter().any(|key| middle(key).x < -5_000.0));
        // Looking straight down, every way round.
        let down = deep_tiles(at, Vec3::new(0.01, 0.0, -1.0), 0.8, 16, 40_000.0, &never_near);
        assert!(down.iter().any(|key| middle(key).x < -5_000.0));
        // Finest tiles under drawn chunks are left out.
        let all_near = |_: TileKey| true;
        assert!(deep_tiles(at, Vec3::X, 0.8, 16, 40_000.0, &all_near).iter().all(|key| key.ring > 0));
    }
}

//! Bake — the Tectonic Sim's product: a specific world, written as a folder of maps that the World
//! Compiler will build the real elevation map from.
//!
//! **Baking** takes the settings and the world seed and makes that world from scratch, at a fixed
//! detail (1 chunk per pixel by default): plates, their motion, the rock, the terrain; runs the
//! simulation until everything has coasted to a stop (or a step limit); and writes
//! `world bakes/<name>/`:
//! - `height.png` (16-bit grey): terrain + squeeze, before any water. Black = the lowest point,
//!   white = the highest; `map.toml` says what heights those are.
//! - `squeeze.png` (16-bit grey): the deformation field. Mid-grey = unchanged rock, brighter =
//!   crushed thicker, darker = stretched thinner (the range is in `map.toml`). Mountainousness.
//! - `push_direction.png` and `push_strength.png` (16-bit grey): which way the rock was squeezed
//!   (0 to 180 degrees), and how hard. Mountain ridges run across the squeeze.
//! - `plates.png` (8-bit grey): which plate the rock belongs to (black = none). `map.toml` lists
//!   each plate's grey value, and whether it's land or ocean crust.
//! - `map.toml`: everything needed to read the maps and to make the same world again: the seed,
//!   every setting, the size, the steps run, the water level used while baking.
//!
//! 16-bit PNGs hold 65,536 levels (precise enough for heights) and still open in any picture
//! program, or in the Graphics Kit, to look at.
//!
//! The same seed and settings always bake the same world.

use super::heights::{self, HeightSettings};
use super::plates::{self, PlateSettings};
use super::rags::{RagSettings, Sim, Spot};
use super::tectonics::{self, MotionSettings};
use crate::fate::derive_seed;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Each step's own seed, taken from the world seed. (The Tectonic Sim on screen uses these too,
/// so a bake makes the same world as watching the same settings.)
pub struct Seeds {
    pub plates: u64,
    pub motion: u64,
    pub rock: u64,
    pub terrain: u64,
}

impl Seeds {
    pub fn of(world: u64) -> Self {
        Self {
            plates: derive_seed(world, "plates"),
            motion: derive_seed(world, "plate motion"),
            rock: derive_seed(world, "rock stiffness"),
            terrain: derive_seed(world, "terrain"),
        }
    }
}

/// The folder all bakes go in, next to `assets/`.
pub fn bakes_folder() -> PathBuf {
    crate::debug::project_folder().join("world bakes")
}

/// The bakes made so far (folders holding a `map.toml`), in alphabetical order.
pub fn saved_bakes() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(bakes_folder())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().join("map.toml").exists())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Letters, digits, `-` and `_` only (spaces become `_`), so any name makes a safe folder name.
pub fn clean_name(name: &str) -> String {
    name.trim().chars().map(|c| if c == ' ' { '_' } else { c }).filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect()
}

/// Everything a bake is made from.
#[derive(Clone)]
pub struct Order {
    pub name: String,
    pub seed: u64,
    pub size_blocks: f32,
    pub blocks_per_pixel: f32,
    pub plates: PlateSettings,
    pub motion: MotionSettings,
    pub rock: RagSettings,
    pub heights: HeightSettings,
    /// The look settings that change the maps: how smooth the squeeze is, and when a crack shows
    /// as a rift.
    pub smoothing: u32,
    pub tear_open: f32,
    /// Sea narrower than this (blocks) with rock on both sides is filled from the nearest rock.
    pub seal_gaps: f32,
    /// Stop after this many steps even if things are still moving.
    pub most_steps: u32,
    /// Where to write it (normally `bakes_folder()`).
    pub folder: PathBuf,
}

/// What a bake made.
pub struct Baked {
    pub folder: PathBuf,
    pub steps: u64,
    pub settled: bool,
    pub land_share: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Range {
    /// The value at grey 0 (black) and at the brightest grey (white).
    pub black: f32,
    pub white: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlateRow {
    /// Its grey value in `plates.png`.
    pub grey: u8,
    pub land: bool,
    /// Its share of the world when it was made.
    pub share: f32,
}

/// The settings the world was made with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub plates: PlateSettings,
    pub motion: MotionSettings,
    pub rock: RagSettings,
    pub heights: HeightSettings,
    pub squeeze_smoothing: u32,
    pub tears_open_past: f32,
    #[serde(default)]
    pub seal_gaps: f32,
}

/// `map.toml`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    /// The world seed, as hexadecimal (as shown on screen).
    pub seed: String,
    pub field_blocks: f32,
    pub blocks_per_pixel: f32,
    pub pixels_across: usize,
    pub steps: u64,
    /// Whether everything had coasted to a stop (otherwise the step limit ended it).
    pub settled: bool,
    /// The water level while baking, and how much of the world was land at it. (The World
    /// Compiler can choose its own.)
    pub water_level: f32,
    pub land_share: f32,
    pub height: Range,
    pub squeeze: Range,
    /// `push_strength.png`'s range. (`push_direction.png` is always 0 to 180 degrees.)
    pub push_strength: Range,
    /// `tilt.png`'s range (bakes made before tilt was saved have none).
    #[serde(default)]
    pub tilt: Range,
    pub plates: Vec<PlateRow>,
    pub settings: Settings,
}

/// The maps of a baked world, before they're written: per pixel, row by row from the top left.
pub struct Maps {
    pub width: usize,
    pub height: Vec<f32>,
    pub squeeze: Vec<f32>,
    /// Which plate (`None` = no plate).
    pub plate: Vec<Option<u8>>,
    /// Which way the rock was squeezed (degrees, 0 to 180: 0 = east–west, 90 = north–south), and
    /// how strongly (0 = not at all).
    pub push_angle: Vec<f32>,
    pub push_strength: Vec<f32>,
    /// How far up (+) or down (−) its plate's tilt each spot's rock is (see `heights::tilt_field`).
    pub tilt: Vec<f32>,
    /// Each plate: land or ocean, and its share of the world when made.
    pub plates: Vec<(bool, f32)>,
    pub steps: u64,
    pub settled: bool,
}

/// Makes the world (not the files). Returns `None` if `stop` was set part-way.
pub fn make(order: &Order, stop: &AtomicBool) -> Option<Maps> {
    let seeds = Seeds::of(order.seed);
    let made = plates::make(&order.plates, seeds.plates, order.size_blocks, order.blocks_per_pixel, stop)?;
    let motions = tectonics::motions(&order.motion, seeds.motion, &made);
    let terrain = heights::terrain(&made, &order.heights, seeds.terrain);
    let tilt = heights::tilt_field(&made, &order.heights, seeds.terrain);
    let mut sim = Sim::new(&made, &motions, &order.rock, seeds.rock);
    while sim.steps < order.most_steps as u64 {
        sim.step(&order.rock);
        if sim.settled {
            break;
        }
        if sim.steps % 50 == 0 && stop.load(Ordering::Relaxed) {
            return None;
        }
    }
    let painted = sim.paint(order.smoothing, order.tear_open, order.seal_gaps / order.blocks_per_pixel);
    let height_of = |spot: &Spot| terrain[spot.from as usize] + order.heights.squeeze * spot.squeeze;
    // Open sea (no plate) is as deep as the deepest plate rock.
    let deepest = painted.iter().flatten().map(height_of).fold(order.heights.ocean, f32::min);
    Some(Maps {
        width: made.width,
        height: painted.iter().map(|here| here.as_ref().map_or(deepest, height_of)).collect(),
        squeeze: painted.iter().map(|here| here.map_or(0.0, |spot| spot.squeeze)).collect(),
        plate: painted.iter().map(|here| here.map(|spot| spot.plate)).collect(),
        push_angle: painted.iter().map(|here| here.map_or(0.0, |spot| spot.push_angle())).collect(),
        push_strength: painted.iter().map(|here| here.map_or(0.0, |spot| spot.push_strength())).collect(),
        tilt: painted.iter().map(|here| here.map_or(0.0, |spot| tilt[spot.from as usize])).collect(),
        plates: made.kept.iter().map(|plate| (plate.land, plate.share)).collect(),
        steps: sim.steps,
        settled: sim.settled,
    })
}

fn write_grey16(path: &Path, width: usize, values: impl Iterator<Item = u16>) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, width as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Sixteen);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    let bytes: Vec<u8> = values.flat_map(u16::to_be_bytes).collect();
    writer.write_image_data(&bytes).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}

fn write_grey8(path: &Path, width: usize, values: impl Iterator<Item = u8>) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, width as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    let bytes: Vec<u8> = values.collect();
    writer.write_image_data(&bytes).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}

/// A value as a 16-bit grey on a range.
fn to_grey16(value: f32, range: &Range) -> u16 {
    let span = (range.white - range.black).max(1e-9);
    (((value - range.black) / span).clamp(0.0, 1.0) * 65_535.0).round() as u16
}

/// A plate's grey in `plates.png`: spread evenly from dark to white, so they can be told apart by
/// eye (0, black, is "no plate").
fn plate_grey(plate: usize, plates: usize) -> u8 {
    (((plate + 1) * 255) / plates.max(1)).min(255) as u8
}

/// Makes the world and writes its folder. Returns `None` if `stop` was set part-way.
pub fn bake(order: &Order, stop: &AtomicBool) -> Option<Result<Baked, String>> {
    let maps = make(order, stop)?;
    Some(write(order, &maps))
}

fn write(order: &Order, maps: &Maps) -> Result<Baked, String> {
    let folder = order.folder.join(clean_name(&order.name));
    std::fs::create_dir_all(&folder).map_err(|error| format!("{}: {error}", folder.display()))?;

    let (low, high) = maps.height.iter().fold((f32::MAX, f32::MIN), |(low, high), &h| (low.min(h), high.max(h)));
    let height = Range { black: low, white: high.max(low + 1e-6) };
    // Symmetric about "unchanged", so mid-grey always means unchanged rock.
    let reach = maps.squeeze.iter().fold(0.0f32, |most, &s| most.max(s.abs())).max(1e-6);
    let squeeze = Range { black: -reach, white: reach };
    write_grey16(&folder.join("height.png"), maps.width, maps.height.iter().map(|&h| to_grey16(h, &height)))?;
    write_grey16(&folder.join("squeeze.png"), maps.width, maps.squeeze.iter().map(|&s| to_grey16(s, &squeeze)))?;
    let tilt = Range { black: -1.5, white: 1.5 };
    write_grey16(&folder.join("tilt.png"), maps.width, maps.tilt.iter().map(|&t| to_grey16(t, &tilt)))?;

    let count = maps.plates.len();
    write_grey8(&folder.join("plates.png"), maps.width, maps.plate.iter().map(|p| p.map_or(0, |p| plate_grey(p as usize, count))))?;
    // Direction: 0 to 180 degrees over the whole grey range; strength: 0 to the strongest.
    let angles = Range { black: 0.0, white: 180.0 };
    write_grey16(&folder.join("push_direction.png"), maps.width, maps.push_angle.iter().map(|&a| to_grey16(a, &angles)))?;
    let strongest = maps.push_strength.iter().fold(0.0f32, |most, &s| most.max(s)).max(1e-6);
    let push = Range { black: 0.0, white: strongest };
    write_grey16(&folder.join("push_strength.png"), maps.width, maps.push_strength.iter().map(|&s| to_grey16(s, &push)))?;

    let water = order.heights.water;
    let land = maps.height.iter().filter(|&&h| h > water).count();
    let land_share = land as f32 / maps.height.len().max(1) as f32;
    let info = Info {
        name: clean_name(&order.name),
        seed: format!("{:016X}", order.seed),
        field_blocks: order.size_blocks,
        blocks_per_pixel: order.blocks_per_pixel,
        pixels_across: maps.width,
        steps: maps.steps,
        settled: maps.settled,
        water_level: water,
        land_share,
        height,
        squeeze,
        push_strength: push,
        tilt,
        plates: maps.plates.iter().enumerate().map(|(p, &(land, share))| PlateRow { grey: plate_grey(p, count), land, share }).collect(),
        settings: Settings {
            plates: order.plates.clone(),
            motion: order.motion.clone(),
            rock: order.rock.clone(),
            heights: order.heights.clone(),
            squeeze_smoothing: order.smoothing,
            tears_open_past: order.tear_open,
            seal_gaps: order.seal_gaps,
        },
    };
    let header = "# Incus Artus: a world baked by the Tectonic Sim.\n\
                  # height.png, squeeze.png and push_strength.png are 16-bit grey: grey 0 is the [range]'s black, 65535 its white.\n\
                  # push_direction.png is 16-bit grey: which way the rock was squeezed, 0 to 180 degrees\n\
                  # (0 = east-west, 90 = north-south); it only means something where push_strength isn't black.\n\
                  # Mountain ridges run across the squeeze, at 90 degrees to it.\n\
                  # tilt.png is 16-bit grey: how far up (+) or down (-) its plate's tilt the rock is ([tilt] range).\n\
                  # plates.png is 8-bit grey: 0 = no plate; [[plates]] gives each plate's grey.\n\n";
    let text = toml::to_string_pretty(&info).map_err(|error| error.to_string())?;
    std::fs::write(folder.join("map.toml"), format!("{header}{text}")).map_err(|error| error.to_string())?;
    Ok(Baked { folder, steps: maps.steps, settled: maps.settled, land_share })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(folder: &Path) -> Order {
        Order {
            name: "test world".to_string(),
            seed: 42,
            size_blocks: 72_000.0,
            blocks_per_pixel: 512.0,
            plates: PlateSettings::default(),
            motion: MotionSettings::default(),
            rock: RagSettings::default(),
            heights: HeightSettings::default(),
            smoothing: 3,
            tear_open: 2.0,
            seal_gaps: 1_500.0,
            most_steps: 200,
            folder: folder.to_path_buf(),
        }
    }

    #[test]
    fn a_bake_writes_readable_maps_and_the_same_seed_bakes_the_same_world() {
        let folder = std::env::temp_dir().join(format!("incus-artus-bake-test-{}", std::process::id()));
        let order = order(&folder);
        let baked = bake(&order, &AtomicBool::new(false)).unwrap().unwrap();
        assert_eq!(baked.folder, folder.join("test_world"));
        for file in ["height.png", "squeeze.png", "push_direction.png", "push_strength.png", "tilt.png", "plates.png", "map.toml"] {
            assert!(baked.folder.join(file).exists(), "{file} missing");
        }
        let info: Info = toml::from_str(&std::fs::read_to_string(baked.folder.join("map.toml")).unwrap()).unwrap();
        assert_eq!(info.seed, "000000000000002A");
        assert!(info.steps > 0 && info.steps <= 200);
        assert!(info.height.black < info.height.white);
        assert_eq!(info.settings.rock, RagSettings::default());
        assert!(!info.plates.is_empty());
        // The pictures open, at the right size.
        let picture = super::super::image::load_png(&baked.folder.join("height.png")).unwrap();
        assert_eq!(picture.width, info.pixels_across);

        let again = make(&order, &AtomicBool::new(false)).unwrap();
        let first = make(&order, &AtomicBool::new(false)).unwrap();
        assert_eq!(first.height, again.height);
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn a_stopped_bake_writes_nothing() {
        let folder = std::env::temp_dir().join(format!("incus-artus-bake-stop-{}", std::process::id()));
        assert!(bake(&order(&folder), &AtomicBool::new(true)).is_none());
        assert!(!folder.exists());
    }

    #[test]
    fn greys_map_back_to_their_values() {
        let range = Range { black: -0.8, white: 1.6 };
        assert_eq!(to_grey16(-0.8, &range), 0);
        assert_eq!(to_grey16(1.6, &range), 65_535);
        let mid = to_grey16(0.4, &range) as f32 / 65_535.0 * 2.4 - 0.8;
        assert!((mid - 0.4).abs() < 1e-4);
        // Plates get different greys, none of them black.
        let greys: Vec<u8> = (0..13).map(|p| plate_grey(p, 13)).collect();
        assert!(greys.iter().all(|&g| g > 0) && greys.windows(2).all(|w| w[0] < w[1]));
    }
}

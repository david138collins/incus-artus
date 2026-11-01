//! World Compiler — turns a baked world (the Tectonic Sim's product) into smooth maps the
//! Interpreter makes the ground from.
//!
//! The world is Z 0 (bedrock) to Z 399. **Sea level is Z 128**, and **the tallest mountains reach
//! Z 333** [David]. Ground at Z 128 or above is dry land; below it is sea floor.
//!
//! **The compiler does the jobs that need the whole map at once:**
//! 1. **Sea level**: one of the bake's heights is chosen as sea level (the `water` slider; the land
//!    share shows).
//! 2. **Soften the coastline**: the bake's coast is a plate's outline on a grid of pixels (a
//!    staircase); softened a little, it's a smooth curve between pixels. The heights and tilt near
//!    it are softened as much.
//!
//! **The recipe** for the ground (land and sea curves, grading, mountains, ridges, peaks,
//! detail) is the Interpreter's (`crate::world::interpreter`), worked out at every block as
//! the world is made. Here it's worked out at each pixel's centre, to show the world in relief.
//! [Moved 2026-10-24: worked out per pixel and then smoothed between pixels, its
//! sharp curves made a row of humps along every coast and cliff.]
//!
//! Everything is worked out on all the computer's cores, in the background.

use super::additives::{self, AdditiveSettings};
use super::climate::ClimateSettings;
use super::bake::{self, Info};
use super::runner::every_pixel;
pub use crate::world::Spline;
use crate::world::{CompiledMap, InterpreterSettings, Lift, LiftFrom, Maps};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// The world's own numbers (set in `world`), as the whole Zs the compiler's maps are kept in: one
// place to change them.
/// The world's water surface: ground at this Z or above is dry land.
pub const SEA_LEVEL: u16 = crate::world::SEA_LEVEL as u16;
/// No ground goes higher than this.
pub const HIGHEST: u16 = crate::world::HIGHEST as u16;
/// The top of the world.
pub const TOP_OF_THE_WORLD: u16 = (crate::world::WORLD_HEIGHT - 1) as u16;

/// A baked world, read back from its folder.
pub struct Bake {
    pub name: String,
    pub info: Info,
    pub width: usize,
    /// Per pixel, row by row from the top left, in the bake's own units (see `Info`).
    pub height: Vec<f32>,
    pub squeeze: Vec<f32>,
    /// Degrees, anticlockwise from east as on the map (0 = east–west), and how hard.
    pub push_angle: Vec<f32>,
    pub push_strength: Vec<f32>,
    /// How far up (+) or down (−) its plate's tilt each spot is (all 0 for bakes made before tilt
    /// was saved).
    pub tilt: Vec<f32>,
    /// The plate's grey in `plates.png` (0 = no plate).
    pub plate: Vec<u8>,
    /// The most crushed rock (the highest squeeze) and the most stretched (the lowest) in the bake.
    pub most_crushed: f32,
    pub most_stretched: f32,
}

impl Bake {
    /// Fills in the most crushed and most stretched rock from the squeeze map.
    fn measured(mut self) -> Self {
        self.most_crushed = self.squeeze.iter().copied().fold(0.0, f32::max);
        self.most_stretched = self.squeeze.iter().copied().fold(0.0, f32::min);
        self
    }
}

/// Reads a greyscale PNG (8- or 16-bit) as values from 0 to 1, exactly.
fn read_grey(path: &Path) -> Result<(usize, Vec<f32>), String> {
    let file = std::fs::File::open(path).map_err(|error| format!("couldn't open {}: {error}", path.display()))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|error| format!("{}: not a PNG I can read: {error}", path.display()))?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("the picture is too big")?];
    let info = reader.next_frame(&mut buffer).map_err(|error| format!("{}: {error}", path.display()))?;
    if info.color_type != png::ColorType::Grayscale {
        return Err(format!("{} isn't a grey picture", path.display()));
    }
    let bytes = &buffer[..info.buffer_size()];
    let values = match info.bit_depth {
        png::BitDepth::Sixteen => bytes.chunks_exact(2).map(|b| u16::from_be_bytes([b[0], b[1]]) as f32 / 65_535.0).collect(),
        png::BitDepth::Eight => bytes.iter().map(|&b| b as f32 / 255.0).collect(),
        other => return Err(format!("{}: {other:?}-bit grey isn't supported", path.display())),
    };
    Ok((info.width as usize, values))
}

/// Reads a bake's folder.
pub fn load(folder: &Path) -> Result<Bake, String> {
    let text = std::fs::read_to_string(folder.join("map.toml")).map_err(|error| format!("map.toml: {error}"))?;
    let info: Info = toml::from_str(&text).map_err(|error| format!("map.toml: {error}"))?;
    // A grey (0 to 1) back to the value it stands for.
    fn on(range: &bake::Range) -> impl Fn(f32) -> f32 + '_ {
        move |grey| range.black + grey * (range.white - range.black)
    }
    let (width, height) = read_grey(&folder.join("height.png"))?;
    let (_, squeeze) = read_grey(&folder.join("squeeze.png"))?;
    let (_, push_angle) = read_grey(&folder.join("push_direction.png"))?;
    let (_, push_strength) = read_grey(&folder.join("push_strength.png"))?;
    let (_, plate) = read_grey(&folder.join("plates.png"))?;
    // Older bakes have no tilt map: no grading for them.
    let tilt = match read_grey(&folder.join("tilt.png")) {
        Ok((_, tilt)) => tilt.into_iter().map(on(&info.tilt)).collect(),
        Err(_) => vec![0.0; width * width],
    };
    let sizes = [height.len(), squeeze.len(), push_angle.len(), push_strength.len(), plate.len(), tilt.len()];
    if sizes.iter().any(|&n| n != width * width) {
        return Err("the bake's pictures aren't all the same size".to_string());
    }
    let angles = bake::Range { black: 0.0, white: 180.0 };
    Ok(Bake {
        name: folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        width,
        height: height.into_iter().map(on(&info.height)).collect(),
        squeeze: squeeze.into_iter().map(on(&info.squeeze)).collect(),
        push_angle: push_angle.into_iter().map(on(&angles)).collect(),
        push_strength: push_strength.into_iter().map(on(&info.push_strength)).collect(),
        tilt,
        plate: plate.into_iter().map(|g| (g * 255.0).round() as u8).collect(),
        most_crushed: 0.0,
        most_stretched: 0.0,
        info,
    }
    .measured())
}

/// How the bake becomes the compiler's maps. The recipe for the ground (the curves, mountains,
/// ridges, peaks, detail) is the Interpreter's, kept in here too so one preset holds both panels;
/// in the preset file its settings sit alongside these, as before.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompileSettings {
    /// Which of the bake's heights is sea level.
    pub water: f32,
    /// How much the coastline (and the heights and tilt near it) is softened, in blocks: the bake's
    /// coast is a plate's outline on a grid of pixels, a staircase, and softening it this much
    /// makes it a smooth curve (0 = none: the staircase shows).
    pub soften: f32,
    /// The same for the mountains' maps (the squeeze, and the push's direction and strength): each
    /// is the rock traced on the same grid of pixels, and the mountain curve and the ridges laid
    /// across the push showed its 32-block steps, as rows of notches along every range's flanks
    /// (0 = none).
    pub soften_mountains: f32,
    #[serde(flatten)]
    pub interpreter: InterpreterSettings,
    /// The Topography panel's (not part of compiling: changing them doesn't compile again).
    pub additives: AdditiveSettings,
    /// The Climate panel's (worked out after compiling, from the recipe: changing them doesn't
    /// compile again).
    pub climate: ClimateSettings,
}

impl Default for CompileSettings {
    fn default() -> Self {
        Self {
            water: 0.0,
            soften: 64.0,
            soften_mountains: 64.0,
            interpreter: InterpreterSettings::default(),
            additives: AdditiveSettings::default(),
            climate: ClimateSettings::default(),
        }
    }
}

/// A compiled world: the maps and the recipe (what a world is made from), and the ground's Z at
/// each pixel's centre (what the compiler shows).
pub struct Compiled {
    pub width: usize,
    pub z: Vec<u16>,
    /// How much of the world's circle is dry land.
    pub land_share: f32,
    pub map: Arc<CompiledMap>,
    /// The lift's points (pixel indices), to show.
    pub lift_points: Vec<usize>,
}

/// The bake's seed, as a number.
fn seed_of(info: &Info) -> u64 {
    u64::from_str_radix(&info.seed, 16).unwrap_or(0)
}

/// How far from sea level the coast map looks (in the bake's height units): heights further above
/// or below count the same. So softening the coast is even on both sides, and the coastline stays
/// where it was, only smoothed.
const COAST_REACH: f32 = 0.03;

/// Blurs a map with three passes of a box `reach` pixels wide each way (close to a soft, round
/// blur), across and then down.
pub fn blur(values: &mut [f32], width: usize, reach: usize) {
    if reach == 0 {
        return;
    }
    let mut line = vec![0.0f32; width];
    for _ in 0..3 {
        // Across each row, then down each column.
        for across in [true, false] {
            for k in 0..width {
                let at = |n: usize| if across { k * width + n } else { n * width + k };
                let mut sum = 0.0;
                let mut count = 0usize;
                for n in 0..reach.min(width) {
                    sum += values[at(n)];
                    count += 1;
                }
                for n in 0..width {
                    if n + reach < width {
                        sum += values[at(n + reach)];
                        count += 1;
                    }
                    if n > reach {
                        sum -= values[at(n - reach - 1)];
                        count -= 1;
                    }
                    line[n] = sum / count.max(1) as f32;
                }
                for (n, &value) in line.iter().enumerate() {
                    values[at(n)] = value;
                }
            }
        }
    }
}

/// Compiles a bake: the whole-map jobs, then the ground at each pixel's centre to show. Returns
/// `None` if `stop` was set part-way.
pub fn compile(bake: &Bake, settings: &CompileSettings, stop: &AtomicBool) -> Option<Compiled> {
    let width = bake.width;
    let blocks = bake.info.blocks_per_pixel;
    let reach = |blocks_across: f32| (blocks_across / blocks / 2.0).round() as usize;

    // 1. Sea level, and the coastline softened into a curve (with the heights and tilt beside it,
    // so nothing near the coast keeps the staircase).
    let soften = reach(settings.soften);
    let mut coast: Vec<f32> = bake.height.iter().map(|&h| (h - settings.water).clamp(-COAST_REACH, COAST_REACH)).collect();
    let mut height = bake.height.clone();
    let mut tilt = bake.tilt.clone();
    for map in [&mut coast, &mut height, &mut tilt] {
        blur(map, width, soften);
    }
    // The squeeze's direction as a doubled-angle arrow, so it blends between pixels.
    let (mut push_x, mut push_y): (Vec<f32>, Vec<f32>) = bake.push_angle.iter().map(|a| ((2.0 * a.to_radians()).cos(), (2.0 * a.to_radians()).sin())).unzip();
    // The mountains' maps softened too (as the coast is), so ranges don't keep the pixels' steps.
    // (The arrow is softened as its two parts: that averages directions properly.) The most crushed
    // and stretched are measured again after, so the tallest peaks stay as tall.
    let mut squeeze = bake.squeeze.clone();
    let mut push_strength = bake.push_strength.clone();
    let soften_mountains = reach(settings.soften_mountains);
    for map in [&mut squeeze, &mut push_x, &mut push_y, &mut push_strength] {
        blur(map, width, soften_mountains);
    }
    let (most_crushed, most_stretched) = (squeeze.iter().copied().fold(0.0, f32::max), squeeze.iter().copied().fold(0.0, f32::min));
    let coast_distance = crate::world::coast_distance(&coast, width, blocks);
    let seed = seed_of(&bake.info);

    // The mountains' points, from the bake's own crush (so the mountains curve doesn't move them),
    // where they stand (blocks from the north-west corner, at their pixels' centres), and
    // how far every pixel is from the nearest.
    let lift_points = lift_points(bake, &settings.interpreter.lift, seed);
    let lift_distance = crate::world::point_distance(&lift_points, width, blocks);
    let mountain_points = lift_points.iter().map(|&i| [((i % width) as f32 + 0.5) * blocks, ((i / width) as f32 + 0.5) * blocks]).collect();
    let maps = Maps {
        width,
        blocks_per_pixel: blocks,
        coast,
        coast_distance,
        lift_distance,
        mountain_points,
        height,
        tilt,
        squeeze,
        push_x,
        push_y,
        push_strength,
        water: settings.water,
        lowest: bake.info.height.black,
        highest: bake.info.height.white,
        most_crushed,
        most_stretched,
        // The climate is added when the world is spawned (it's worked out from this map).
        ..Maps::default()
    };
    let map = CompiledMap::new(Arc::new(maps), seed, settings.interpreter.clone());
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // 2. The ground at each pixel's centre, as the Interpreter makes it (less the block-sized
    // detail), to show.
    let z = every_pixel(width, stop, |i| map.at_pixel(i))?;
    let z: Vec<u16> = z.into_iter().map(|z| z.round() as u16).collect();

    // Land share, inside the world's circle.
    let half = width as f32 / 2.0;
    let (mut land, mut inside) = (0usize, 0usize);
    for (i, &z) in z.iter().enumerate() {
        let (dx, dy) = ((i % width) as f32 + 0.5 - half, (i / width) as f32 + 0.5 - half);
        if dx * dx + dy * dy <= half * half {
            inside += 1;
            land += (z >= SEA_LEVEL) as usize;
        }
    }
    Some(Compiled { width, z, land_share: land as f32 / inside.max(1) as f32, map: Arc::new(map), lift_points })
}

/// The lift's points (pixel indices): scattered over the crushed rock, or one in the middle of each
/// range of it. The crush is the bake's squeeze against its most crushed, 0 to 1, lightly smoothed.
fn lift_points(bake: &Bake, lift: &Lift, seed: u64) -> Vec<usize> {
    let (width, blocks) = (bake.width, bake.info.blocks_per_pixel);
    let mut crush: Vec<f32> = bake.squeeze.iter().map(|&s| (s / bake.most_crushed.max(1e-6)).clamp(0.0, 1.0)).collect();
    blur(&mut crush, width, 1);
    match lift.from {
        LiftFrom::Points => additives::pick_seeds(&crush, lift.spacing, lift.least, lift.most, seed ^ 0x11f7, blocks),
        LiftFrom::Ranges => additives::find_ranges(&crush, width, lift.range_cut)
            .list
            .iter()
            .filter(|r| r.peak >= lift.least && r.peak <= lift.most && (r.size as f32).sqrt() * blocks >= lift.smallest_range)
            .map(|r| (r.centre.1 as usize).min(width - 1) * width + (r.centre.0 as usize).min(width - 1))
            .collect(),
    }
}

/// Saves a compiled world into its bake's folder: `elevation.png` (16-bit grey, the grey is the
/// ground's Z itself, 0 to 399) and `compile.toml` (the settings it was compiled with).
pub fn save(folder: &Path, compiled: &Compiled, settings: &CompileSettings) -> Result<(), String> {
    let path = folder.join("elevation.png");
    let file = std::fs::File::create(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), compiled.width as u32, compiled.width as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Sixteen);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    let bytes: Vec<u8> = compiled.z.iter().flat_map(|z| z.to_be_bytes()).collect();
    writer.write_image_data(&bytes).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())?;
    let header = "# Incus Artus: how elevation.png was compiled from this bake.\n\
                  # elevation.png is 16-bit grey, and the grey is the ground's Z (0 to 1,023; sea level 128).\n\n";
    let text = toml::to_string_pretty(settings).map_err(|error| error.to_string())?;
    std::fs::write(folder.join("compile.toml"), format!("{header}{text}")).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up bake: land in the middle (height rising to the right), sea round it, and a band
    /// of crushed rock squeezed east–west down the middle.
    fn bake(push_angle: f32) -> Bake {
        let width = 64;
        let mut height = vec![-0.5; width * width];
        let mut squeeze = vec![0.0; width * width];
        for row in 16..48 {
            for col in 16..48 {
                height[row * width + col] = (col - 16) as f32 / 32.0 * 0.4;
                if (28..36).contains(&col) {
                    squeeze[row * width + col] = 0.8;
                }
            }
        }
        let info: Info = toml::from_str(
            "name = 't'\nseed = '00000000000000AB'\nfield_blocks = 8192.0\nblocks_per_pixel = 128.0\npixels_across = 64\nsteps = 1\n\
             settled = true\nwater_level = 0.0\nland_share = 0.2\nplates = []\n[height]\nblack = -0.5\nwhite = 0.4\n\
             [squeeze]\nblack = -0.8\nwhite = 0.8\n[push_strength]\nblack = 0.0\nwhite = 0.3\n[settings]\nsqueeze_smoothing = 3\ntears_open_past = 2.0\n\
             [settings.plates]\n[settings.motion]\n[settings.rock]\n[settings.heights]\n",
        )
        .unwrap();
        Bake {
            name: "t".to_string(),
            info,
            width,
            height,
            squeeze,
            push_angle: vec![push_angle; width * width],
            push_strength: vec![0.3; width * width],
            tilt: vec![0.0; width * width],
            plate: vec![1; width * width],
            most_crushed: 0.0,
            most_stretched: 0.0,
        }
        .measured()
    }

    fn compiled(bake: &Bake, settings: &CompileSettings) -> Compiled {
        compile(bake, settings, &AtomicBool::new(false)).unwrap()
    }

    /// The default recipe (hills, which this once turned off, were cut 2026-10-28).
    fn plain() -> CompileSettings {
        CompileSettings::default()
    }

    #[test]
    fn a_saved_preset_reads_back_exactly_as_it_was() {
        // The "default" light compares settings with the saved preset exactly.
        let mut settings = CompileSettings { water: -0.003, soften: 96.7, ..CompileSettings::default() };
        settings.interpreter.coast_rise = 12.345;
        let back: CompileSettings = toml::from_str(&toml::to_string_pretty(&settings).unwrap()).unwrap();
        assert!(back == settings);
    }

    #[test]
    fn a_preset_holds_both_panels_in_one_flat_list_as_before() {
        // Presets saved before the recipe moved to the Interpreter list everything side by side.
        let text = "water = -0.003\nsoften = 96.0\nfoothills = 6900.0\nridges = 0.89\nhills = 14.0\nsharpness = 1.5\n\
                    [land]\ndots = [[0.0, 0.0], [1.0, 0.98]]\n";
        let settings: CompileSettings = toml::from_str(text).unwrap();
        // (Foothills, cut 2026-10-27, and hills, cut 2026-10-28, are passed over.)
        assert_eq!((settings.water, settings.soften), (-0.003, 96.0));
        assert_eq!((settings.interpreter.ridges, settings.interpreter.sharpness), (0.89, 1.5));
        assert_eq!(settings.interpreter.land.dots, vec![(0.0, 0.0), (1.0, 0.98)]);
        // And written back the same way: one flat list.
        let written = toml::to_string_pretty(&settings).unwrap();
        assert!(written.contains("ridges = 0.89") && !written.contains("[interpreter]"), "{written}");
    }

    #[test]
    fn presets_from_before_hills_and_detail_were_perlin_fields_still_load() {
        // As David's "hope" was saved: sizes side by side, no noise or mask tables.
        let text = "ridges = 1.0\nhills = 6.0\nhill_size = 1500.0\ndetail = 3.0\ndetail_size = 96.0\n";
        let settings: CompileSettings = toml::from_str(text).unwrap();
        let defaults = InterpreterSettings::default();
        assert_eq!(settings.interpreter.ridges, 1.0);
        assert_eq!(settings.interpreter.detail_noise.scale, 96.0);
        assert_eq!(settings.interpreter.detail_mask, defaults.detail_mask);
        // And a preset with them all survives being written and read.
        let mut changed = settings.clone();
        changed.interpreter.detail_mask.source = crate::world::MaskSource::Coast;
        changed.interpreter.detail_noise.octaves = 4;
        changed.additives.sides = super::super::additives::Sides::Inside;
        changed.additives.bend = -2.5;
        let back: CompileSettings = toml::from_str(&toml::to_string_pretty(&changed).unwrap()).unwrap();
        assert_eq!(back, changed);
    }

    /// `plain`, with the Interpreter's recipe changed by `change`.
    fn recipe(change: impl FnOnce(&mut InterpreterSettings)) -> CompileSettings {
        let mut settings = plain();
        change(&mut settings.interpreter);
        settings
    }

    #[test]
    fn land_is_at_or_above_sea_level_and_sea_below_and_nothing_past_333() {
        let bake = bake(0.0);
        let world = compiled(&bake, &CompileSettings::default());
        for (i, &z) in world.z.iter().enumerate() {
            assert!(z <= HIGHEST && z >= 1);
            // (A pixel exactly at sea level is the coastline itself: softening may put it either side.)
            if bake.height[i] > 0.0 {
                assert!(z >= SEA_LEVEL, "land at {z}, pixel {} {} height {}", i % 64, i / 64, bake.height[i]);
            } else if bake.height[i] < 0.0 && bake.squeeze[i] <= 0.0 {
                assert!(z < SEA_LEVEL, "sea at {z}");
            }
        }
        assert!(world.land_share > 0.1 && world.land_share < 0.5, "land {}", world.land_share);
    }

    #[test]
    fn softening_the_mountains_takes_the_pixels_steps_off_a_ranges_flank() {
        // A range whose edge runs diagonally across the pixels: on the grid, a staircase.
        let mut bake = bake(45.0);
        for row in 0..64 {
            for col in 0..64 {
                bake.height[row * 64 + col] = 0.2;
                bake.squeeze[row * 64 + col] = if col > row { 0.8 } else { 0.0 };
            }
        }
        let bake = bake.measured();
        // Along the edge (pixel col − row = 0, as continuous pixel positions), the ground should be
        // the same all the way: any wobble is the staircase showing.
        let wobble = |soften: f32| {
            let settings = CompileSettings { soften_mountains: soften, ..recipe(|i| (i.ridges, i.detail) = (0.0, 0.0)) };
            let map = compiled(&bake, &settings).map;
            let blocks = bake.info.blocks_per_pixel;
            let along: Vec<f32> = (0..160)
                .map(|k| {
                    let t = 20.0 + k as f32 * 0.1;
                    map.height((32.0 - t) * blocks, (t - 32.0) * blocks).unwrap()
                })
                .collect();
            along.iter().copied().fold(f32::MIN, f32::max) - along.iter().copied().fold(f32::MAX, f32::min)
        };
        let (stepped, softened) = (wobble(0.0), wobble(512.0));
        assert!(stepped > 5.0, "the staircase didn't show unsoftened: {stepped}");
        assert!(softened < stepped / 4.0, "softened {softened} vs {stepped}");
    }

    #[test]
    fn crushed_rock_becomes_mountains() {
        let bake = bake(0.0);
        let world = compiled(&bake, &plain());
        let at = |col: usize| world.z[32 * 64 + col] as f32;
        // The crushed band stands well above the plain next to it.
        assert!(at(31) > at(26) + 30.0, "band {} plain {}", at(31), at(26));
        // A flat-zero mountain spline: no band.
        let none = Spline::through(&[(0.0, 0.0), (1.0, 0.0)]);
        let flat = compiled(&bake, &recipe(|i| i.mountains = none));
        assert!((flat.z[32 * 64 + 31] as f32 - flat.z[32 * 64 + 30] as f32).abs() < 10.0);
    }

    #[test]
    fn the_most_crushed_line_of_a_belt_is_its_peak() {
        // A belt whose crush rises to a spine in the middle.
        let mut bake = bake(0.0);
        for row in 0..64 {
            for col in 0..64 {
                bake.height[row * 64 + col] = 0.2;
                bake.squeeze[row * 64 + col] = (1.6 - (col as f32 - 32.0).abs() * 0.1).max(0.0);
            }
        }
        let bake = bake.measured();
        let world = compiled(&bake, &recipe(|i| i.ridges = 0.0));
        let row: Vec<u16> = (0..64).map(|col| world.z[32 * 64 + col]).collect();
        let peak = (0..64).max_by_key(|&col| row[col]).unwrap();
        assert!((31..=33).contains(&peak), "peak at column {peak}: {row:?}");
        // It climbs steadily to the peak rather than levelling into a plateau.
        assert!(row[32] > row[28] && row[28] > row[24] && row[24] > row[20]);
    }

    #[test]
    fn grading_steepens_a_plates_high_side_and_softens_its_low_side() {
        let mut bake = bake(0.0);
        // The right half of the land is up the plate's tilt, the left half down it.
        for row in 0..64 {
            for col in 0..64 {
                bake.tilt[row * 64 + col] = if col >= 32 { 1.0 } else { -1.0 };
            }
        }
        let flat = compiled(&bake, &recipe(|i| i.grade = 0.0));
        let graded = compiled(&bake, &recipe(|i| i.grade = 0.5));
        let at = |world: &Compiled, col: usize| world.z[20 * 64 + col] as i32;
        assert!(at(&graded, 44) > at(&flat, 44), "high side not higher");
        assert!(at(&graded, 20) <= at(&flat, 20), "low side not gentler");
        // And the sea beside the high side is deeper.
        assert!(at(&graded, 50) < at(&flat, 50));
    }

    #[test]
    fn ridges_run_across_the_squeeze() {
        // Squeezed east–west (0°): ridges run north–south, so going north–south along one changes
        // the height less than going east–west across them.
        let mut bake = bake(0.0);
        // Crush a wide area, so there's room to measure in both directions.
        for row in 0..64 {
            for col in 0..64 {
                bake.height[row * 64 + col] = 0.2;
                bake.squeeze[row * 64 + col] = 0.8;
            }
        }
        let bake = bake.measured();
        let settings = recipe(|i| (i.ridges, i.ridge_spacing, i.ridge_length, i.sharpness) = (1.0, 600.0, 20_000.0, 2.0));
        let roughness = |world: &Compiled, step: (usize, usize)| {
            let mut total = 0.0;
            for row in 8..56 {
                for col in 8..56 {
                    let here = world.z[row * 64 + col] as f32;
                    let next = world.z[(row + step.1) * 64 + col + step.0] as f32;
                    total += (here - next).abs();
                }
            }
            total
        };
        let east_west = compiled(&bake, &settings);
        assert!(roughness(&east_west, (1, 0)) > roughness(&east_west, (0, 1)) * 1.5);
        // Squeezed north–south: the other way round.
        let bake = Bake { push_angle: vec![90.0; 64 * 64], ..bake };
        let north_south = compiled(&bake, &settings);
        assert!(roughness(&north_south, (0, 1)) > roughness(&north_south, (1, 0)) * 1.5);
    }

    #[test]
    fn a_bake_loads_back_with_its_heights_intact() {
        use crate::worldgen::bake::{self, Order};
        let folder = std::env::temp_dir().join(format!("incus-artus-compiler-load-{}", std::process::id()));
        let order = Order {
            name: "round trip".to_string(),
            seed: 7,
            size_blocks: 72_000.0,
            blocks_per_pixel: 512.0,
            plates: Default::default(),
            motion: Default::default(),
            rock: Default::default(),
            heights: Default::default(),
            smoothing: 3,
            tear_open: 2.0,
            seal_gaps: 1_500.0,
            fill_holes: 10_000.0,
            most_steps: 100,
            folder: folder.clone(),
        };
        let maps = bake::make(&order, &AtomicBool::new(false)).unwrap();
        bake::bake(&order, &AtomicBool::new(false)).unwrap().unwrap();
        let loaded = load(&folder.join("round_trip")).unwrap();
        assert_eq!(loaded.width, maps.width);
        // 16-bit greys: within a 65,535th of the range.
        let span = loaded.info.height.white - loaded.info.height.black;
        for (a, b) in loaded.height.iter().zip(&maps.height) {
            assert!((a - b).abs() <= span / 60_000.0, "{a} vs {b}");
        }
        for (a, b) in loaded.tilt.iter().zip(&maps.tilt) {
            assert!((a - b).abs() <= 3.0 / 60_000.0, "tilt {a} vs {b}");
        }
        assert!(loaded.tilt.iter().any(|&t| t.abs() > 0.1), "the tilt map is empty");
        assert_eq!(loaded.plate.iter().filter(|&&p| p > 0).count(), maps.plate.iter().flatten().count());
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn a_saved_elevation_reads_back_as_the_same_zs() {
        let bake = bake(0.0);
        let world = compiled(&bake, &CompileSettings::default());
        let folder = std::env::temp_dir().join(format!("incus-artus-compile-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        save(&folder, &world, &CompileSettings::default()).unwrap();
        let (width, greys) = read_grey(&folder.join("elevation.png")).unwrap();
        assert_eq!(width, 64);
        let back: Vec<u16> = greys.iter().map(|g| (g * 65_535.0).round() as u16).collect();
        assert_eq!(back, world.z);
        let _ = std::fs::remove_dir_all(&folder);
    }
}

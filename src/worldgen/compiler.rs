//! World Compiler — turns a baked world (the Tectonic Sim's product) into the ground's height in
//! blocks, everywhere.
//!
//! The world is Z 0 (bedrock) to Z 399. **Sea level is Z 128**, and **the tallest mountains reach
//! Z 333** [David]. Ground at Z 128 or above is dry land; below it is sea floor.
//!
//! **How a bake's heights become blocks** (one Z per pixel, 32 blocks a pixel):
//! 1. **Sea level**: one of the bake's heights is chosen as sea level (the `water` slider; the land
//!    share shows). Heights above it rise from Z 128 toward the **highlands** height, heights below
//!    it fall toward the **deepest** sea floor. Each side has a **curve**: above 1, most land stays
//!    low and only the highest parts climb (and seas stay shallow near the coast); below 1, the
//!    reverse.
//! 2. **Mountains** from the squeeze map: where rock was crushed thicker (past `mountains_start`,
//!    fully by `mountains_full`) ridges rise, lined up **across the squeeze direction** (ridges run
//!    at right angles to the push, as real ranges do). Where the direction is unclear (weak pushes)
//!    the ridges wander every way. Mountains only use the room left below Z 333, so nothing goes
//!    past it.
//! 3. **Rifts**: rock stretched thinner sinks.
//! 4. **Hills**: gentle rolling bumps on land, for now. (Block-level detail comes later, when chunks
//!    are made.)
//!
//! Everything is worked out on all the computer's cores, in the background.

use super::bake::{self, Info};
use super::noise::{Fade, perlin};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// The world's water surface: ground at this Z or above is dry land.
pub const SEA_LEVEL: u16 = 128;
/// No ground goes higher than this.
pub const HIGHEST: u16 = 333;
/// The top of the world.
pub const TOP_OF_THE_WORLD: u16 = 399;

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
    /// The plate's grey in `plates.png` (0 = no plate).
    pub plate: Vec<u8>,
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
    let sizes = [height.len(), squeeze.len(), push_angle.len(), push_strength.len(), plate.len()];
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
        plate: plate.into_iter().map(|g| (g * 255.0).round() as u8).collect(),
        info,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompileSettings {
    /// Which of the bake's heights is sea level.
    pub water: f32,
    /// The Z of the deepest sea floor.
    pub deepest: f32,
    /// The Z the highest land (before mountains) reaches.
    pub highlands: f32,
    /// The shape of the climb from the coast to the highlands, and of the fall to the deepest sea:
    /// 1 = straight; above 1 = stays low (shallow) near the coast, climbing (deepening) late.
    pub land_curve: f32,
    pub sea_curve: f32,
    /// How crushed rock must be for mountains to start, and to be at their fullest.
    pub mountains_start: f32,
    pub mountains_full: f32,
    /// How much of the room up to Z 333 the fullest mountains fill (0 to 1).
    pub mountains: f32,
    /// How far apart ridges are (across them), and how long they run (along them), in blocks.
    pub ridge_spacing: f32,
    pub ridge_length: f32,
    /// Higher = narrower, sharper ridge crests.
    pub sharpness: f32,
    /// How deep the most stretched rock sinks, in blocks.
    pub rift_depth: f32,
    /// Rolling hills on land: how high (blocks either way) and how big (blocks).
    pub hills: f32,
    pub hill_size: f32,
}

impl Default for CompileSettings {
    fn default() -> Self {
        Self {
            water: 0.0,
            deepest: 30.0,
            highlands: 200.0,
            land_curve: 1.6,
            sea_curve: 1.5,
            mountains_start: 0.05,
            mountains_full: 0.3,
            mountains: 0.85,
            ridge_spacing: 1_800.0,
            ridge_length: 7_000.0,
            sharpness: 2.0,
            rift_depth: 25.0,
            hills: 8.0,
            hill_size: 1_500.0,
        }
    }
}

/// A compiled world: the ground's Z at every pixel.
pub struct Compiled {
    pub width: usize,
    pub z: Vec<u16>,
    /// How much of the world's circle is dry land.
    pub land_share: f32,
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A ridge pattern, 0 to 1 (1 = a crest): stacked "folded" noise, whose creases are the crests.
fn ridges(seed: u64, u: f32, v: f32, sharpness: f32) -> f32 {
    let (mut total, mut strength, mut strengths, mut scale) = (0.0, 1.0, 0.0, 1.0);
    for octave in 0..3 {
        let crest = 1.0 - perlin(seed, 0xa11 + octave, Fade::Smooth, u * scale, v * scale).abs().min(1.0);
        total += crest.powf(sharpness.max(0.1)) * strength;
        strengths += strength;
        strength *= 0.45;
        scale *= 2.1;
    }
    total / strengths
}

/// The bake's seed, as a number.
fn seed_of(info: &Info) -> u64 {
    u64::from_str_radix(&info.seed, 16).unwrap_or(0)
}

/// The ground's Z at one pixel.
fn ground(bake: &Bake, settings: &CompileSettings, seed: u64, i: usize) -> f32 {
    let blocks = bake.info.blocks_per_pixel;
    let (x, y) = (((i % bake.width) as f32 + 0.5) * blocks, ((i / bake.width) as f32 + 0.5) * blocks);
    let sea = SEA_LEVEL as f32;
    let h = bake.height[i];

    // 1. Sea level and the two curves.
    let mut z = if h >= settings.water {
        let t = (h - settings.water) / (bake.info.height.white - settings.water).max(1e-6);
        sea + (settings.highlands - sea).max(0.0) * t.clamp(0.0, 1.0).powf(settings.land_curve.max(0.05))
    } else {
        let t = (settings.water - h) / (settings.water - bake.info.height.black).max(1e-6);
        (sea - 1.0) - (sea - 1.0 - settings.deepest).max(0.0) * t.clamp(0.0, 1.0).powf(settings.sea_curve.max(0.05))
    };

    // 2. Mountains, lined up across the squeeze where its direction is clear.
    let squeeze = bake.squeeze[i];
    let mask = smooth((squeeze - settings.mountains_start) / (settings.mountains_full - settings.mountains_start).max(1e-6));
    if mask > 0.0 {
        // The push's direction on the map (north is −y in the field's rows).
        let angle = bake.push_angle[i].to_radians();
        let push = (angle.cos(), -angle.sin());
        let across = x * push.0 + y * push.1;
        let along = -x * push.1 + y * push.0;
        let lined_up = ridges(seed, across / settings.ridge_spacing.max(1.0), along / settings.ridge_length.max(1.0), settings.sharpness);
        let wandering = ridges(seed ^ 0x5eed, x / settings.ridge_spacing.max(1.0), y / settings.ridge_spacing.max(1.0), settings.sharpness);
        let clear = smooth(bake.push_strength[i] / 0.1);
        let crest = wandering + (lined_up - wandering) * clear;
        let room = (HIGHEST as f32 - z).max(0.0);
        z += room * settings.mountains.clamp(0.0, 1.0) * mask * (0.35 + 0.65 * crest);
    }

    // 3. Rifts.
    if squeeze < 0.0 {
        z -= settings.rift_depth * (-squeeze / settings.mountains_full.max(1e-6)).min(1.0);
    }

    // 4. Hills, on land only.
    if z >= sea {
        let size = settings.hill_size.max(1.0);
        let rolling = perlin(seed, 0x4111, Fade::Smooth, x / size, y / size) * 0.7 + perlin(seed, 0x4112, Fade::Smooth, x * 2.3 / size, y * 2.3 / size) * 0.3;
        // Never drowns land or lifts it past the top.
        z = (z + rolling * settings.hills).max(sea);
    }
    z.clamp(1.0, HIGHEST as f32)
}

/// Compiles a bake. Returns `None` if `stop` was set part-way.
pub fn compile(bake: &Bake, settings: &CompileSettings, stop: &AtomicBool) -> Option<Compiled> {
    let width = bake.width;
    let seed = crate::fate::derive_seed(seed_of(&bake.info), "world compiler");
    let mut z = vec![0u16; width * width];
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in z.chunks_mut(rows_each * width).enumerate() {
            scope.spawn(move || {
                for (k, out) in chunk.iter_mut().enumerate() {
                    if k % width == 0 && stop.load(Ordering::Relaxed) {
                        return;
                    }
                    *out = ground(bake, settings, seed, band * rows_each * width + k).round() as u16;
                }
            });
        }
    });
    if stop.load(Ordering::Relaxed) {
        return None;
    }
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
    Some(Compiled { width, z, land_share: land as f32 / inside.max(1) as f32 })
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
                  # elevation.png is 16-bit grey, and the grey is the ground's Z (0 to 399; sea level 128).\n\n";
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
            plate: vec![1; width * width],
        }
    }

    fn compiled(bake: &Bake, settings: &CompileSettings) -> Compiled {
        compile(bake, settings, &AtomicBool::new(false)).unwrap()
    }

    #[test]
    fn land_is_at_or_above_sea_level_and_sea_below_and_nothing_past_333() {
        let bake = bake(0.0);
        let world = compiled(&bake, &CompileSettings::default());
        for (i, &z) in world.z.iter().enumerate() {
            assert!(z <= HIGHEST && z >= 1);
            if bake.height[i] >= 0.0 {
                assert!(z >= SEA_LEVEL, "land at {z}");
            } else if bake.squeeze[i] <= 0.0 {
                assert!(z < SEA_LEVEL, "sea at {z}");
            }
        }
        assert!(world.land_share > 0.1 && world.land_share < 0.5, "land {}", world.land_share);
    }

    #[test]
    fn crushed_rock_becomes_mountains() {
        let bake = bake(0.0);
        let world = compiled(&bake, &CompileSettings { hills: 0.0, ..CompileSettings::default() });
        let at = |col: usize| world.z[32 * 64 + col] as f32;
        // The crushed band stands well above the plain next to it.
        assert!(at(31) > at(26) + 30.0, "band {} plain {}", at(31), at(26));
        // No mountains, no band.
        let flat = compiled(&bake, &CompileSettings { hills: 0.0, mountains: 0.0, ..CompileSettings::default() });
        assert!((flat.z[32 * 64 + 31] as f32 - flat.z[32 * 64 + 30] as f32).abs() < 10.0);
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
        let settings = CompileSettings { hills: 0.0, ridge_spacing: 600.0, ridge_length: 20_000.0, ..CompileSettings::default() };
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

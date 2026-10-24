//! World Compiler — turns a baked world (the Tectonic Sim's product) into the ground's height in
//! blocks, everywhere.
//!
//! The world is Z 0 (bedrock) to Z 399. **Sea level is Z 128**, and **the tallest mountains reach
//! Z 333** [David]. Ground at Z 128 or above is dry land; below it is sea floor.
//!
//! **How a bake's heights become blocks** (one Z per pixel, 32 blocks a pixel):
//! 1. **Sea level**: one of the bake's heights is chosen as sea level (the `water` slider; the land
//!    share shows). The **land spline** says how land climbs from the coast (Z 128) to the
//!    **highlands**, and the **sea spline** how the sea floor falls from the coast to the
//!    **deepest**. Splines are curves through dots you place.
//! 2. **Grade** [David]: each plate's tilt (saved in the bake) scales the land's height above the sea
//!    and the sea's depth below it: on a plate's high side both are steeper (steep coasts, deep water
//!    close in), on its low side gentler (wide plains, shallow shelves).
//! 3. **Mountains** from the squeeze map: the **mountain spline** says how tall mountains are for
//!    each amount of squeeze, from none to the most crushed rock in the bake, as a share of the room
//!    up to Z 333 (so nothing goes past it). A belt's most crushed line is its peak. They're carved
//!    (by `ridges`) into ridges lined up **across the squeeze direction**; where the direction is
//!    unclear the ridges wander every way. **Passes** [David]: the crest height rises and dips along
//!    each ridge. **Foothills**: each range spreads a skirt of lower slopes around it, so it rises
//!    from the plains instead of jumping up.
//! 4. **Rifts**: stretched rock sinks, the most stretched rock in the bake sinking `rift_depth`.
//! 5. **Hills**: gentle rolling bumps on land, for now. (Block-level detail comes later, when chunks
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

/// A curve through dots, from 0 to 1 across. The first dot is always at the left edge and the last
/// at the right; the line flows smoothly through all of them, without overshooting (a flat stretch
/// stays flat, and it never bulges past a dot).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Spline {
    /// (across, up), left to right.
    pub dots: Vec<(f32, f32)>,
}

impl Default for Spline {
    fn default() -> Self {
        Self { dots: vec![(0.0, 0.0), (1.0, 1.0)] }
    }
}

impl Spline {
    pub fn through(dots: &[(f32, f32)]) -> Self {
        Self { dots: dots.to_vec() }
    }

    /// The curve's height at `x` (0 to 1 across).
    pub fn at(&self, x: f32) -> f32 {
        let dots = &self.dots;
        match dots.len() {
            0 => return 0.0,
            1 => return dots[0].1,
            _ => {}
        }
        let x = x.clamp(dots[0].0, dots[dots.len() - 1].0);
        let k = dots.windows(2).position(|pair| x <= pair[1].0).unwrap_or(dots.len() - 2);
        let slopes = self.slopes();
        let ((x0, y0), (x1, y1)) = (dots[k], dots[k + 1]);
        let h = (x1 - x0).max(1e-6);
        let t = (x - x0) / h;
        // Hermite: the two dots and the curve's steepness at each.
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * y0 + (t3 - 2.0 * t2 + t) * h * slopes[k] + (-2.0 * t3 + 3.0 * t2) * y1 + (t3 - t2) * h * slopes[k + 1]
    }

    /// How steep the curve is at each dot: the average of the slopes either side, flattened where
    /// the line turns (so it never overshoots), and kept gentle enough not to bulge (Fritsch and
    /// Carlson's rule).
    fn slopes(&self) -> Vec<f32> {
        let dots = &self.dots;
        let n = dots.len();
        let between: Vec<f32> = dots.windows(2).map(|p| (p[1].1 - p[0].1) / (p[1].0 - p[0].0).max(1e-6)).collect();
        let mut slopes = vec![0.0; n];
        slopes[0] = between[0];
        slopes[n - 1] = between[n - 2];
        for k in 1..n - 1 {
            slopes[k] = if between[k - 1] * between[k] <= 0.0 { 0.0 } else { (between[k - 1] + between[k]) / 2.0 };
        }
        for k in 0..n - 1 {
            if between[k].abs() < 1e-9 {
                (slopes[k], slopes[k + 1]) = (0.0, 0.0);
                continue;
            }
            let (a, b) = (slopes[k] / between[k], slopes[k + 1] / between[k]);
            let size = a * a + b * b;
            if size > 9.0 {
                let shrink = 3.0 / size.sqrt();
                (slopes[k], slopes[k + 1]) = (shrink * a * between[k], shrink * b * between[k]);
            }
        }
        slopes
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompileSettings {
    /// Which of the bake's heights is sea level.
    pub water: f32,
    /// The Z of the deepest sea floor, and of the highest land (before mountains).
    pub deepest: f32,
    pub highlands: f32,
    /// How land climbs from the coast (left) to the bake's highest ground (right): up = a share of
    /// the way from Z 128 to the highlands.
    pub land: Spline,
    /// How the sea floor falls from the coast (left) to the bake's deepest point (right): up = a
    /// share of the way from just below Z 128 down to the deepest.
    pub sea: Spline,
    /// How strongly each plate's tilt grades it: steeper coasts on its high side, gentler on its low
    /// side (0 = no grading).
    pub grade: f32,
    /// How tall mountains are for each amount of squeeze, from none (left) to the most crushed rock
    /// in the bake (right): up = a share of the room up to Z 333.
    pub mountains: Spline,
    /// How much the ridge pattern carves into the mountains: 0 = smooth domes, 1 = all ridges.
    pub ridges: f32,
    /// How far apart ridges are (across them), and how long they run (along them), in blocks.
    pub ridge_spacing: f32,
    pub ridge_length: f32,
    /// Higher = narrower, sharper ridge crests.
    pub sharpness: f32,
    /// How deeply the crest height dips into passes along each ridge (0 = even crests), and how far
    /// apart the dips are, in blocks.
    pub passes: f32,
    pub pass_spacing: f32,
    /// How far a range's skirt of lower slopes spreads around it, in blocks (0 = none).
    pub foothills: f32,
    /// How deep the most stretched rock sinks, in blocks.
    pub rift_depth: f32,
    /// Rolling hills on land: how high (blocks either way) and how big (blocks).
    pub hills: f32,
    pub hill_size: f32,
    /// How the compiled map becomes blocks when a world is rendered (copied into the world).
    pub interpreter: crate::world::InterpreterSettings,
}

impl Default for CompileSettings {
    fn default() -> Self {
        Self {
            water: 0.0,
            deepest: 30.0,
            highlands: 200.0,
            land: Spline::through(&[(0.0, 0.0), (0.4, 0.2), (0.75, 0.6), (1.0, 1.0)]),
            sea: Spline::through(&[(0.0, 0.0), (0.4, 0.25), (1.0, 1.0)]),
            grade: 0.4,
            mountains: Spline::through(&[(0.0, 0.0), (0.08, 0.0), (0.5, 0.35), (1.0, 0.9)]),
            ridges: 0.5,
            ridge_spacing: 3_000.0,
            ridge_length: 15_000.0,
            sharpness: 1.2,
            passes: 0.4,
            pass_spacing: 12_000.0,
            foothills: 2_500.0,
            rift_depth: 30.0,
            hills: 6.0,
            hill_size: 1_500.0,
            interpreter: crate::world::InterpreterSettings::default(),
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

/// One pixel before foothills and hills: the ground from the splines and grading, the mountains to
/// add on top, and how far a rift sinks it.
struct Parts {
    ground: f32,
    mountains: f32,
    rift: f32,
}

fn parts(bake: &Bake, settings: &CompileSettings, seed: u64, i: usize) -> Parts {
    let blocks = bake.info.blocks_per_pixel;
    let (x, y) = (((i % bake.width) as f32 + 0.5) * blocks, ((i / bake.width) as f32 + 0.5) * blocks);
    let sea = SEA_LEVEL as f32;
    let h = bake.height[i];

    // 1 and 2. Sea level, the splines, and grading by the plate's tilt.
    let steep = (1.0 + settings.grade * bake.tilt[i]).max(0.15);
    let ground = if h >= settings.water {
        let t = (h - settings.water) / (bake.info.height.white - settings.water).max(1e-6);
        sea + (settings.highlands - sea).max(0.0) * settings.land.at(t).clamp(0.0, 1.0) * steep
    } else {
        let t = (settings.water - h) / (settings.water - bake.info.height.black).max(1e-6);
        (sea - 1.0) - (sea - 1.0 - settings.deepest).max(0.0) * settings.sea.at(t).clamp(0.0, 1.0) * steep
    };

    // 3. Mountains.
    let squeeze = bake.squeeze[i];
    let mut mountains = 0.0;
    let tall = settings.mountains.at(squeeze / bake.most_crushed.max(1e-6)).clamp(0.0, 1.0);
    if squeeze > 0.0 && tall > 0.0 {
        // The push's direction on the map (north is −y in the field's rows), and the ridges lined
        // up across it, blended into wandering ridges where the direction is unclear.
        let angle = bake.push_angle[i].to_radians();
        let push = (angle.cos(), -angle.sin());
        let across = x * push.0 + y * push.1;
        let along = -x * push.1 + y * push.0;
        let spacing = settings.ridge_spacing.max(1.0);
        let lined_up = ridges(seed, across / spacing, along / settings.ridge_length.max(1.0), settings.sharpness);
        let wandering = ridges(seed ^ 0x5eed, x / spacing, y / spacing, settings.sharpness);
        let clear = smooth(bake.push_strength[i] / 0.1);
        let crest = wandering + (lined_up - wandering) * clear;
        let carved = 1.0 - settings.ridges.clamp(0.0, 1.0) * (1.0 - crest);
        // Passes: the crest height rises and dips along the ridge.
        let pass_size = settings.pass_spacing.max(1.0);
        let along_ridge = perlin(seed, 0x9a55, Fade::Smooth, along / pass_size, across / (pass_size * 3.0));
        let wandering_passes = perlin(seed, 0x9a56, Fade::Smooth, x / pass_size, y / pass_size);
        let wave = wandering_passes + (along_ridge - wandering_passes) * clear;
        let dipped = 1.0 - settings.passes.clamp(0.0, 1.0) * (0.5 - 0.5 * wave);
        mountains = (HIGHEST as f32 - ground).max(0.0) * tall * carved * dipped;
    }

    // 4. Rifts.
    let rift = if squeeze < 0.0 { settings.rift_depth * smooth(squeeze / bake.most_stretched.min(-1e-6)) } else { 0.0 };
    Parts { ground, mountains, rift }
}

/// Blurs a map with three passes of a box `reach` pixels wide each way (close to a soft, round
/// blur), across and then down.
fn blur(values: &mut [f32], width: usize, reach: usize) {
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

/// Compiles a bake. Returns `None` if `stop` was set part-way.
pub fn compile(bake: &Bake, settings: &CompileSettings, stop: &AtomicBool) -> Option<Compiled> {
    let width = bake.width;
    let seed = crate::fate::derive_seed(seed_of(&bake.info), "world compiler");
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = width.div_ceil(cores).max(1);

    // Each pixel's parts, on all cores.
    let mut ground = vec![0.0f32; width * width];
    let mut mountains = vec![0.0f32; width * width];
    let mut rift = vec![0.0f32; width * width];
    std::thread::scope(|scope| {
        let bands = ground.chunks_mut(rows_each * width).zip(mountains.chunks_mut(rows_each * width)).zip(rift.chunks_mut(rows_each * width));
        for (band, ((ground, mountains), rift)) in bands.enumerate() {
            scope.spawn(move || {
                for k in 0..ground.len() {
                    if k % width == 0 && stop.load(Ordering::Relaxed) {
                        return;
                    }
                    let here = parts(bake, settings, seed, band * rows_each * width + k);
                    (ground[k], mountains[k], rift[k]) = (here.ground, here.mountains, here.rift);
                }
            });
        }
    });
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // Foothills: a soft skirt around each range, wherever it's higher than the range itself.
    let reach = (settings.foothills / bake.info.blocks_per_pixel / 2.0).round() as usize;
    if reach > 0 {
        let mut skirt = mountains.clone();
        blur(&mut skirt, width, reach);
        for (mountain, skirt) in mountains.iter_mut().zip(skirt) {
            *mountain = mountain.max(skirt);
        }
    }
    if stop.load(Ordering::Relaxed) {
        return None;
    }

    // Together, with hills on land.
    let sea = SEA_LEVEL as f32;
    let size = settings.hill_size.max(1.0);
    let blocks = bake.info.blocks_per_pixel;
    let z: Vec<u16> = (0..width * width)
        .map(|i| {
            let mut z = (ground[i] + mountains[i]).min(HIGHEST as f32) - rift[i];
            if z >= sea {
                let (x, y) = (((i % width) as f32 + 0.5) * blocks, ((i / width) as f32 + 0.5) * blocks);
                let rolling = perlin(seed, 0x4111, Fade::Smooth, x / size, y / size) * 0.7 + perlin(seed, 0x4112, Fade::Smooth, x * 2.3 / size, y * 2.3 / size) * 0.3;
                // Never drowns land.
                z = (z + rolling * settings.hills).max(sea);
            }
            z.clamp(1.0, HIGHEST as f32).round() as u16
        })
        .collect();

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

    /// No hills, foothills or passes: only what a test is looking at.
    fn plain() -> CompileSettings {
        CompileSettings { hills: 0.0, foothills: 0.0, passes: 0.0, ..CompileSettings::default() }
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
        let world = compiled(&bake, &plain());
        let at = |col: usize| world.z[32 * 64 + col] as f32;
        // The crushed band stands well above the plain next to it.
        assert!(at(31) > at(26) + 30.0, "band {} plain {}", at(31), at(26));
        // A flat-zero mountain spline: no band.
        let none = Spline::through(&[(0.0, 0.0), (1.0, 0.0)]);
        let flat = compiled(&bake, &CompileSettings { mountains: none, ..plain() });
        assert!((flat.z[32 * 64 + 31] as f32 - flat.z[32 * 64 + 30] as f32).abs() < 10.0);
    }

    #[test]
    fn a_spline_passes_through_its_dots_without_overshooting() {
        let spline = Spline::through(&[(0.0, 0.0), (0.3, 0.2), (0.6, 0.2), (1.0, 1.0)]);
        for &(x, y) in &spline.dots {
            assert!((spline.at(x) - y).abs() < 1e-5);
        }
        // Flat between two equal dots, and never below the lower or above the higher dot of a stretch.
        for k in 0..=20 {
            let x = 0.3 + 0.3 * k as f32 / 20.0;
            assert!((spline.at(x) - 0.2).abs() < 1e-5, "{x}: {}", spline.at(x));
        }
        for k in 0..=100 {
            let x = k as f32 / 100.0;
            assert!((-1e-5..=1.0 + 1e-5).contains(&spline.at(x)));
        }
        // Smooth: no sudden jumps.
        for k in 0..100 {
            let (a, b) = (spline.at(k as f32 / 100.0), spline.at((k + 1) as f32 / 100.0));
            assert!((a - b).abs() < 0.05);
        }
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
        let world = compiled(&bake, &CompileSettings { ridges: 0.0, ..plain() });
        let row: Vec<u16> = (0..64).map(|col| world.z[32 * 64 + col]).collect();
        let peak = (0..64).max_by_key(|&col| row[col]).unwrap();
        assert!((31..=33).contains(&peak), "peak at column {peak}: {row:?}");
        // It climbs steadily to the peak rather than levelling into a plateau.
        assert!(row[32] > row[28] && row[28] > row[24] && row[24] > row[20]);
    }

    #[test]
    fn foothills_spread_a_skirt_without_lowering_the_peaks() {
        let bake = bake(0.0);
        let bare = compiled(&bake, &CompileSettings { ridges: 0.0, ..plain() });
        let skirted = compiled(&bake, &CompileSettings { ridges: 0.0, foothills: 2_000.0, ..plain() });
        let at = |world: &Compiled, col: usize| world.z[32 * 64 + col];
        // Beside the band, the ground now rises toward it; on the band, no lower than before.
        assert!(at(&skirted, 26) > at(&bare, 26) + 5, "{} vs {}", at(&skirted, 26), at(&bare, 26));
        assert!(at(&skirted, 31) >= at(&bare, 31));
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
        let flat = compiled(&bake, &CompileSettings { grade: 0.0, ..plain() });
        let graded = compiled(&bake, &CompileSettings { grade: 0.5, ..plain() });
        let at = |world: &Compiled, col: usize| world.z[20 * 64 + col] as i32;
        assert!(at(&graded, 44) > at(&flat, 44), "high side not higher");
        assert!(at(&graded, 20) <= at(&flat, 20), "low side not gentler");
        // And the sea beside the high side is deeper.
        assert!(at(&graded, 50) < at(&flat, 50));
    }

    #[test]
    fn the_most_stretched_rock_sinks_the_full_rift_depth() {
        let mut bake = bake(0.0);
        bake.squeeze[20 * 64 + 20] = -0.2;
        let bake = bake.measured();
        let none = compiled(&bake, &CompileSettings { rift_depth: 0.0, ..plain() });
        let rifted = compiled(&bake, &CompileSettings { rift_depth: 30.0, ..plain() });
        let sunk = none.z[20 * 64 + 20] as i32 - rifted.z[20 * 64 + 20] as i32;
        assert!((29..=31).contains(&sunk), "sank {sunk}");
    }

    #[test]
    fn passes_vary_the_crest_height_along_a_ridge() {
        // Squeezed east–west, so ridges run north–south: walk along one.
        let mut bake = bake(0.0);
        for row in 0..64 {
            for col in 0..64 {
                bake.height[row * 64 + col] = 0.2;
                bake.squeeze[row * 64 + col] = 0.8;
            }
        }
        let bake = bake.measured();
        let settings = CompileSettings { ridges: 0.0, pass_spacing: 2_000.0, ..plain() };
        let along = |world: &Compiled| (4..60).map(|row| world.z[row * 64 + 32]).collect::<Vec<u16>>();
        let even = along(&compiled(&bake, &settings));
        let passes = along(&compiled(&bake, &CompileSettings { passes: 0.8, ..settings.clone() }));
        let spread = |v: &[u16]| v.iter().max().unwrap() - v.iter().min().unwrap();
        assert!(spread(&passes) > spread(&even) + 20, "even {even:?}, with passes {passes:?}");
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
        let settings = CompileSettings { ridges: 1.0, ridge_spacing: 600.0, ridge_length: 20_000.0, sharpness: 2.0, ..plain() };
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

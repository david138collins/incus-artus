//! Image layers — a picture (PNG) laid onto the field, for using David's own textures as fields:
//! a rough burlap weave pressed into the terrain under an ancient temple, or a texture to colour
//! for the game's art.
//!
//! - The picture's **brightness** becomes the layer's values (0 = black, 1 = white), so it can be
//!   shaped and coloured with a palette like any field. Its **own colours are kept** too, so it can
//!   also be shown as it was.
//! - It's **placed in the world**: its centre in blocks, and how many blocks each of its pixels
//!   covers. Outside the picture, and where it's see-through, the layer below shows.
//! - **Repeat** tiles it across the whole field.
//!
//! Pictures come from the `images/` folder next to `assets/`, or are dragged onto the window.

use super::field::Field;
use super::made::{Made, Shaping};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn image_folder() -> PathBuf {
    crate::debug::project_folder().join("images")
}

/// The PNG files in the images folder, by file name, in alphabetical order.
pub fn saved_images() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(image_folder())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("png")))
        .filter_map(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

/// A loaded picture: its pixels as red, green, blue, alpha.
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 4]>,
}

/// Reads a PNG of any kind (greyscale, colour, with or without see-through, 16-bit, palette).
pub fn load_png(path: &Path) -> Result<Picture, String> {
    let file = std::fs::File::open(path).map_err(|error| format!("couldn't open {}: {error}", path.display()))?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    // Turn every kind of PNG into plain 8-bit channels.
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|error| format!("not a PNG I can read: {error}"))?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("the picture is too big")?];
    let info = reader.next_frame(&mut buffer).map_err(|error| format!("couldn't read the picture: {error}"))?;
    let bytes = &buffer[..info.buffer_size()];
    let pixels: Vec<[u8; 4]> = match info.color_type {
        png::ColorType::Grayscale => bytes.iter().map(|&v| [v, v, v, 255]).collect(),
        png::ColorType::GrayscaleAlpha => bytes.chunks_exact(2).map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Rgb => bytes.chunks_exact(3).map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::Rgba => bytes.chunks_exact(4).map(|p| [p[0], p[1], p[2], p[3]]).collect(),
        png::ColorType::Indexed => return Err("palette PNGs should have been expanded".into()),
    };
    Ok(Picture { width: info.width as usize, height: info.height as usize, pixels })
}

/// Where an image layer sits and how it's shaped. Everything that changes what it makes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Placement {
    /// The picture's centre, in blocks from the field's centre.
    pub centre_north: f32,
    pub centre_east: f32,
    /// How many blocks each of the picture's pixels covers.
    pub blocks_per_pixel: f32,
    /// Tile the picture across the whole field.
    pub repeat: bool,
    pub output: Shaping,
}

impl Default for Placement {
    fn default() -> Self {
        // Pictures are already black to white: shown as they are unless asked.
        Self {
            centre_north: 0.0,
            centre_east: 0.0,
            blocks_per_pixel: 256.0,
            repeat: false,
            output: Shaping { fill_range: false, ..Shaping::default() },
        }
    }
}

/// Lays the picture onto a field of the given size.
pub fn make(picture: &Arc<Picture>, placement: &Placement, size_blocks: f32, blocks_per_pixel: f32) -> Made {
    let mut raw = Field::new(size_blocks, blocks_per_pixel);
    let mut covered = vec![false; raw.values.len()];
    let mut original = vec![[0u8; 3]; raw.values.len()];
    let scale = placement.blocks_per_pixel.max(0.01);
    let (w, h) = (picture.width as f32, picture.height as f32);
    for row in 0..raw.height {
        for col in 0..raw.width {
            let (north, east) = raw.blocks_at(col, row);
            // Which picture pixel lands here (the picture's top row is its north edge).
            let mut x = ((east - placement.centre_east) / scale + w / 2.0).floor();
            let mut y = ((placement.centre_north - north) / scale + h / 2.0).floor();
            if placement.repeat {
                (x, y) = (x.rem_euclid(w), y.rem_euclid(h));
            } else if x < 0.0 || y < 0.0 || x >= w || y >= h {
                continue;
            }
            let [r, g, b, a] = picture.pixels[y as usize * picture.width + x as usize];
            if a < 128 {
                continue; // see-through: the layer below shows
            }
            let i = raw.index(col, row);
            covered[i] = true;
            original[i] = [r, g, b];
            // Brightness as the eye sees it (green looks brightest, blue darkest).
            raw.values[i] = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
        }
    }
    Made::finish(raw, Some(covered), Some(original), &placement.output, |v| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker() -> Arc<Picture> {
        // 2 × 2: white, black / black, see-through.
        let pixels = vec![[255, 255, 255, 255], [0, 0, 0, 255], [0, 0, 0, 255], [0, 0, 0, 0]];
        Arc::new(Picture { width: 2, height: 2, pixels })
    }

    #[test]
    fn a_picture_lands_where_it_is_placed_and_nowhere_else() {
        // A 4-pixel field of 10 blocks per pixel; the picture's pixels are 10 blocks too, centred.
        let placement = Placement { blocks_per_pixel: 10.0, ..Placement::default() };
        let made = make(&checker(), &placement, 40.0, 10.0);
        let covered: Vec<bool> = made.covered.clone().unwrap();
        // The picture fills the middle 2 × 2 of the 4 × 4 field, except its see-through corner.
        assert_eq!(covered.iter().filter(|c| **c).count(), 3);
        let top_left_of_picture = made.raw.index(1, 1);
        assert!(covered[top_left_of_picture]);
        assert_eq!(made.raw.values[top_left_of_picture], 1.0);
        assert!(!covered[made.raw.index(2, 2)], "the see-through pixel lets the layer below show");
        assert!(!covered[made.raw.index(0, 0)], "outside the picture");
    }

    #[test]
    fn repeat_tiles_the_picture_everywhere() {
        let placement = Placement { blocks_per_pixel: 10.0, repeat: true, ..Placement::default() };
        let made = make(&checker(), &placement, 40.0, 10.0);
        // Every 2 × 2 tile has one see-through pixel: 12 of 16 covered.
        assert_eq!(made.covered.unwrap().iter().filter(|c| **c).count(), 12);
    }

    #[test]
    fn a_png_reads_back_as_it_was_written() {
        let folder = std::env::temp_dir().join("incus_artus_image_test");
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("tiny.png");
        let file = std::fs::File::create(&path).unwrap();
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), 2, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().unwrap().write_image_data(&[10, 20, 30, 200, 100, 50]).unwrap();
        let picture = load_png(&path).unwrap();
        assert_eq!((picture.width, picture.height), (2, 1));
        assert_eq!(picture.pixels, vec![[10, 20, 30, 255], [200, 100, 50, 255]]);
    }
}

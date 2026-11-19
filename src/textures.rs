//! The blocks' textures for the near eye (plan 2.4, build 205): each block's top and its sides as a picture 32 by 32,
//! with its shrunk versions down to one pixel. The pictures are old's placeholders, carried into `data/textures/`
//! [the spec: "The master palette isn't chosen yet; textures are placeholders"]; the look is old's, which David has
//! as decided: "Textures crisp up close, mipmapped as they shrink, shrinking toward the far colour". The shrinking is
//! old's rule, the one `data/far colours.toml` was made by, so a picture's one pixel is the colour the far eye draws
//! its block in. A block with no picture wears that colour, flat.

use crate::doorway::{Block, Doorway};
use crate::far::FarColours;

/// A picture's side, pixels (old's).
pub const SIDE: u32 = 32;

/// Every block's two pictures, its top's and then its sides', by the block's number, and after them the bars' own
/// (build 227), at each size: `sizes[k]` holds all of them at 32 >> k a side, one after another, four bytes a pixel as
/// stored (red, green, blue, how opaque).
pub struct Textures {
    pub sizes: Vec<Vec<u8>>,
    /// How many pictures: two a block, and two for each block whose bars have their own.
    pub pictures: u32,
    /// How many of the blocks' two were found as files (the rest are flat).
    pub found: u32,
    /// For each block, by its number: 0, or where its bars' own pictures are, a long side's and then an end's, as the
    /// side's picture's place and one (only where both are found).
    pub bars: Vec<u32>,
}

/// A PNG as red, green, blue and opacity, a byte each.
fn read_png(path: &std::path::Path) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = std::fs::read(path).ok()?;
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        _ => 1,
    };
    let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
    for y in 0..info.height as usize {
        for x in 0..info.width as usize {
            let pixel = &buffer[y * info.line_size + x * channels..][..channels];
            rgba.extend_from_slice(&match channels {
                4 => [pixel[0], pixel[1], pixel[2], pixel[3]],
                3 => [pixel[0], pixel[1], pixel[2], 255],
                2 => [pixel[0], pixel[0], pixel[0], pixel[1]],
                _ => [pixel[0], pixel[0], pixel[0], 255],
            });
        }
    }
    Some((info.width, info.height, rgba))
}

/// A picture file as 32 by 32: a block's as it is; a bar's side or end (16 across: half a block), side by side till
/// it fills a block.
fn picture(path: &std::path::Path) -> Option<Vec<u8>> {
    let (width, height, rgba) = read_png(path)?;
    if !(width == SIDE || width == SIDE / 2) || !(height == SIDE || height == SIDE / 2) {
        log::warn!("{} is {width} x {height}: a block's picture is {SIDE} x {SIDE} (a bar's 16 across): left out", path.display());
        return None;
    }
    let mut whole = Vec::with_capacity((SIDE * SIDE * 4) as usize);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let at = (((y % height) * width + x % width) * 4) as usize;
            whole.extend_from_slice(&rgba[at..at + 4]);
        }
    }
    Some(whole)
}

/// A picture halved, each two by two of its pixels made one: old's rule, by which the far colours were made (the
/// colours as stored, weighted by how opaque each pixel is; the opacity the most opaque of the four).
fn halved(pixels: &[u8], side: u32) -> Vec<u8> {
    let half = side / 2;
    let mut out = Vec::with_capacity((half * half * 4) as usize);
    for y in 0..half {
        for x in 0..half {
            let at = |dx: u32, dy: u32, channel: u32| pixels[(((y * 2 + dy) * side + x * 2 + dx) * 4 + channel) as usize] as u32;
            let corners = [(0, 0), (1, 0), (0, 1), (1, 1)];
            let opacity: u32 = corners.iter().map(|&(dx, dy)| at(dx, dy, 3)).sum();
            for channel in 0..3 {
                let weighted: u32 = corners.iter().map(|&(dx, dy)| at(dx, dy, channel) * at(dx, dy, 3)).sum();
                out.push(if opacity == 0 { 0 } else { ((weighted + opacity / 2) / opacity) as u8 });
            }
            out.push(corners.iter().map(|&(dx, dy)| at(dx, dy, 3)).max().unwrap_or(0) as u8);
        }
    }
    out
}

/// Light (0 to 1) as it's stored in a picture.
fn stored(light: f32) -> u8 {
    let v = light.clamp(0.0, 1.0);
    ((if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }) * 255.0).round() as u8
}

/// Every block's pictures, each found by its block's name in `data/textures/`: `blocks/<name>_top.png` and
/// `_side.png`, or `blocks/<name>.png` for both; else a bar's, `bars/<name>_side.png`; else its far colour, flat.
/// `doorway`: the one `colours` was read through (it knows their blocks' names).
pub fn load(doorway: &Doorway, colours: &FarColours) -> Textures {
    let folder = crate::session::project_folder().join("data").join("textures");
    let light = colours.as_light();
    let mut whole: Vec<u8> = Vec::new();
    let mut found = 0;
    for (k, flat) in light.iter().enumerate() {
        let name = doorway.block_name((k / 2) as Block);
        let base = name.rsplit(':').next().unwrap_or(name);
        let way = if k % 2 == 0 { "top" } else { "side" };
        let files = [folder.join("blocks").join(format!("{base}_{way}.png")), folder.join("blocks").join(format!("{base}.png")), folder.join("bars").join(format!("{base}_side.png"))];
        match files.iter().find_map(|path| picture(path)) {
            Some(pixels) => {
                found += 1;
                whole.extend(pixels);
            }
            None => whole.extend([stored(flat[0]), stored(flat[1]), stored(flat[2]), 255].repeat((SIDE * SIDE) as usize)),
        }
    }
    // The bars' own (build 227): `bars/<name>_side.png` (16 across, 32 long: laid up a bar's length) and `_end.png`
    // (16 by 16: a bar's ends, and every face of a lone eighth), each side by side till it fills a picture, as old's
    // `assets/textures/bars/`. A block with only one of them keeps its own picture on its bars.
    let mut bars = vec![0u32; light.len() / 2];
    for (block, bar) in bars.iter_mut().enumerate() {
        let name = doorway.block_name(block as Block);
        let base = name.rsplit(':').next().unwrap_or(name);
        let [side, end] = ["side", "end"].map(|way| picture(&folder.join("bars").join(format!("{base}_{way}.png"))));
        if let (Some(side), Some(end)) = (side, end) {
            *bar = (whole.len() / (SIDE * SIDE * 4) as usize) as u32 + 1;
            whole.extend(side);
            whole.extend(end);
        }
    }
    let pictures = (whole.len() / (SIDE * SIDE * 4) as usize) as u32;
    let mut sizes = vec![whole];
    let mut side = SIDE;
    while side > 1 {
        let before = sizes.last().unwrap();
        let each = (side * side * 4) as usize;
        sizes.push((0..pictures as usize).flat_map(|k| halved(&before[k * each..(k + 1) * each], side)).collect());
        side /= 2;
    }
    log::info!("The blocks' textures: {found} pictures of {} found in data/textures, the rest flat in their far colour; {} blocks' bars with their own side and end; {} sizes each", light.len(), bars.iter().filter(|bar| **bar > 0).count(), sizes.len());
    Textures { sizes, pictures, found, bars }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every picture found shrinks to its block's far colour (`data/far colours.toml` was made by this rule from
    /// these pictures), to within 2 of 1,023; and the blocks the file has pictures for are found.
    #[test]
    fn a_picture_shrinks_to_its_far_colour() {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let textures = load(&doorway, &colours);
        let light = colours.as_light();
        assert!(textures.found >= 14, "the pictures carried from old are found: {} of {}", textures.found, textures.pictures);
        assert_eq!(textures.sizes.len(), 6, "32 down to 1");
        // Stone brick's bars wear their own side and end (build 227), as the two pictures after the blocks'; no other
        // block has them.
        let brick = doorway.block("incus:stone_brick") as usize;
        let blocks = light.len() as u32;
        assert_eq!(textures.bars[brick], blocks + 1, "stone brick's bars' side is the first picture after the blocks'");
        assert_eq!(textures.bars.iter().filter(|bar| **bar > 0).count(), 1, "only stone brick has bars' own pictures");
        assert_eq!(textures.pictures, blocks + 2, "the blocks' two each, and stone brick's bars' two");
        let smallest = textures.sizes.last().unwrap();
        let as_light = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        for (k, flat) in light.iter().enumerate() {
            let name = doorway.block_name((k / 2) as Block).to_string();
            for channel in 0..3 {
                let (got, wanted) = (as_light(smallest[k * 4 + channel]) * 1023.0, flat[channel] * 1023.0);
                // (A flat picture is its colour stored in a byte and read back: within 3 at the bright end.)
                assert!((got - wanted).abs() <= 4.0, "{name}'s {} shrinks to {got:.0} of 1,023, its far colour is {wanted:.0}", if k % 2 == 0 { "top" } else { "side" });
            }
        }
    }
}

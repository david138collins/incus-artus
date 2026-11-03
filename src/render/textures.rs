//! Block textures — loads every block face image into one GPU *texture array*.
//!
//! A texture array is a stack of same-sized images ("layers"). Each quad says which layer it
//! uses, and the shader picks it. Unlike a single big atlas image, a layer can repeat
//! cleanly across a big merged quad.
//!
//! Rules (see docs/decisions.md):
//! - Files live in `assets/textures/blocks/<name>.png` and must be exactly 32 × 32.
//! - Layer 0 is the magenta/black "missing texture" checkerboard. Any texture that's missing
//!   or the wrong size uses it, so problems are obvious on screen.
//!
//! Each layer also gets *mipmaps*: pre-shrunk copies (16×16, 8×8 … 1×1) used for distant blocks.
//! Without them, pixel art sparkles and shimmers in the distance. They're made automatically by
//! averaging pixels, then (with "far textures keep their painted colours", on the Render page) each
//! averaged pixel is swapped for the nearest colour the texture really has, so no colour appears in
//! the distance that wasn't painted [watch-list item 15d, 2026-10-28]. David may hand-touch them later.

use crate::data::{BlockRegistry, block_texture_folder};
use bevy::asset::RenderAssetUsages;
use bevy::image::{
    CompressedImageFormats, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor,
    ImageType,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};
use std::collections::HashMap;

/// Block face textures must be this many pixels square.
pub const TEXTURE_SIZE: u32 = 32;
/// 32 → 16 → 8 → 4 → 2 → 1
const MIP_LEVELS: u32 = 6;

/// The finished texture array plus what was loaded, for the renderer and the debug panel.
pub struct BlockTextures {
    pub image: Image,
    /// Name of each layer, in layer order (layer 0 = "<missing>").
    pub layer_names: Vec<String>,
    /// Texture names that were asked for but couldn't be used, with the reason.
    pub problems: Vec<String>,
    /// Per block number: the layer used by each of its six faces.
    pub face_layers: Vec<[u32; 6]>,
    /// Per block number: its bars' own textures (a long side, an end), if it has them.
    pub bar_layers: Vec<Option<[u32; 2]>>,
}

/// `painted`: the shrunk versions keep the texture's own colours (see `keep_painted`), and the GPU
/// picks one size rather than blending two.
pub fn build_block_textures(registry: &BlockRegistry, painted: bool) -> BlockTextures {
    let folder = block_texture_folder();

    let mut layers: Vec<Vec<u8>> = vec![missing_texture()];
    let mut layer_names = vec!["<missing>".to_string()];
    let mut problems = Vec::new();
    let mut layer_of: HashMap<String, u32> = HashMap::new();

    let mut face_layers = Vec::with_capacity(registry.len());
    let mut bar_layers = Vec::with_capacity(registry.len());
    for def in registry.defs() {
        let mut faces = [0u32; 6];
        for (face, texture) in def.faces.iter().enumerate() {
            if texture.is_empty() {
                continue;
            }
            let layer = *layer_of.entry(texture.clone()).or_insert_with(|| {
                match load_texture(&folder.join(format!("{texture}.png"))) {
                    Ok(pixels) => {
                        layers.push(pixels);
                        layer_names.push(texture.clone());
                        (layers.len() - 1) as u32
                    }
                    Err(reason) => {
                        warn!("Block texture '{texture}': {reason}");
                        problems.push(format!("{texture}: {reason}"));
                        0
                    }
                }
            });
            faces[face] = layer;
        }
        face_layers.push(faces);
        // Its bars' own textures, if it has them: a long side and an end.
        let bar = def.bar.as_ref().filter(|bar| !bar.side.is_empty() && !bar.end.is_empty()).map(|bar| {
            [(&bar.side, (16, 32)), (&bar.end, (16, 16))].map(|(texture, size)| {
                *layer_of.entry(format!("bar:{texture}")).or_insert_with(|| match load_bar_texture(&crate::data::bar_texture_folder().join(format!("{texture}.png")), size) {
                    Ok(pixels) => {
                        layers.push(pixels);
                        layer_names.push(format!("{texture} (a bar's)"));
                        (layers.len() - 1) as u32
                    }
                    Err(reason) => {
                        warn!("Bar texture '{texture}': {reason}");
                        problems.push(format!("{texture} (a bar's): {reason}"));
                        0
                    }
                })
            })
        });
        bar_layers.push(bar);
    }

    BlockTextures {
        image: build_array_image(&layers, painted),
        layer_names,
        problems,
        face_layers,
        bar_layers,
    }
}

/// Load a bar's texture (`assets/textures/bars/`): a long side, `wide` × `tall` 16 × 32, or an end,
/// 16 × 16 (32 pixels a block, as every block's). Each pixel is repeated to fill a layer (32 × 32),
/// so the bar's own corner measures it in the shader and every pixel keeps its size.
pub(crate) fn load_bar_texture(path: &std::path::Path, (wide, tall): (u32, u32)) -> Result<Vec<u8>, String> {
    let (width, height, rgba) = read_png(path)?;
    if (width, height) != (wide, tall) {
        return Err(format!("is {width}×{height}, a bar's must be {wide}×{tall}"));
    }
    let mut pixels = Vec::with_capacity((TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let from = ((y * tall / TEXTURE_SIZE) * wide + x * wide / TEXTURE_SIZE) as usize * 4;
            pixels.extend_from_slice(&rgba[from..from + 4]);
        }
    }
    Ok(pixels)
}

/// A PNG's size and RGBA pixels, or why not.
pub(crate) fn read_png(path: &std::path::Path) -> Result<(u32, u32, Vec<u8>), String> {
    let bytes = std::fs::read(path).map_err(|_| format!("file not found ({})", path.display()))?;
    let image = Image::from_buffer(&bytes, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::MAIN_WORLD)
        .map_err(|e| format!("not a readable PNG ({e})"))?;
    let (width, height) = (image.width(), image.height());
    let rgba = image.convert(TextureFormat::Rgba8UnormSrgb).and_then(|converted| converted.data).ok_or_else(|| "unsupported pixel format".to_string())?;
    Ok((width, height, rgba))
}

/// Load a PNG as 32 × 32 RGBA pixels, or explain why not.
pub(crate) fn load_texture(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let (width, height, rgba) = read_png(path)?;
    if width != TEXTURE_SIZE || height != TEXTURE_SIZE {
        return Err(format!("is {width}×{height}, must be {TEXTURE_SIZE}×{TEXTURE_SIZE}"));
    }
    Ok(rgba)
}

/// Magenta and black 4-pixel checks.
fn missing_texture() -> Vec<u8> {
    let mut pixels = Vec::with_capacity((TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let magenta = ((x / 4) + (y / 4)) % 2 == 0;
            pixels.extend_from_slice(if magenta { &[255, 0, 255, 255] } else { &[0, 0, 0, 255] });
        }
    }
    pixels
}

/// Halve an RGBA image by averaging each 2 × 2 block of pixels.
/// Colours are averaged only over the pixels that can be seen (weighted by opacity), so the
/// clear pixels of a see-through texture don't darken its coloured ones in the distance.
/// Opacity takes the *most* opaque of the four: a thin line stays solid in the shrunk
/// versions instead of fading below the cut-out threshold and vanishing at a distance or a
/// steep angle. (Ordinary blocks are fully opaque everywhere, so nothing changes for them.)
fn shrink(pixels: &[u8], size: u32) -> Vec<u8> {
    let half = size / 2;
    let mut out = Vec::with_capacity((half * half * 4) as usize);
    for y in 0..half {
        for x in 0..half {
            let sample = |dx: u32, dy: u32, channel: u32| {
                pixels[(((y * 2 + dy) * size + (x * 2 + dx)) * 4 + channel) as usize] as u32
            };
            let corners = [(0, 0), (1, 0), (0, 1), (1, 1)];
            let alpha_sum: u32 = corners.iter().map(|&(dx, dy)| sample(dx, dy, 3)).sum();
            for channel in 0..3 {
                let value = if alpha_sum == 0 {
                    0
                } else {
                    let weighted: u32 = corners.iter().map(|&(dx, dy)| sample(dx, dy, channel) * sample(dx, dy, 3)).sum();
                    (weighted + alpha_sum / 2) / alpha_sum
                };
                out.push(value as u8);
            }
            let alpha_max = corners.iter().map(|&(dx, dy)| sample(dx, dy, 3)).max().unwrap_or(0);
            out.push(alpha_max as u8);
        }
    }
    out
}

/// A block texture's colour as the near ground shows it once a block is smaller than a pixel: its
/// smallest shrunk version, made by the same `shrink` as the mipmaps, read as light (linear RGB, as
/// the shaders work). The distant terrain takes its colours from here, so near and far agree,
/// as the averaged mipmaps make it (with "far textures keep their painted colours" on, the shrunk
/// versions are snapped to painted colours, but the far colour stays this average: what a field of
/// them comes to from afar). [2026-10-29: it averaged the texture as light itself, while the
/// mipmaps average the stored values; on gabbro's strong speckle the near ground came out 18%
/// darker on screen, the seam on rocky hills.]
pub fn far_colour(name: &str) -> Option<[f32; 3]> {
    let pixels = load_texture(&block_texture_folder().join(format!("{name}.png"))).ok()?;
    Some(smallest_mip_colour(pixels, TEXTURE_SIZE))
}

/// The 1 × 1 mipmap of a `size` × `size` RGBA texture, as light.
fn smallest_mip_colour(mut pixels: Vec<u8>, mut size: u32) -> [f32; 3] {
    while size > 1 {
        pixels = shrink(&pixels, size);
        size /= 2;
    }
    let linear = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    [linear(pixels[0]), linear(pixels[1]), linear(pixels[2])]
}

/// Each pixel of a shrunk version swapped for the nearest of the colours painted in `original` (the
/// full-size texture): seen from afar, the texture is still made of its own colours. Opacity is kept.
fn keep_painted(level: &[u8], original: &[u8]) -> Vec<u8> {
    let mut painted: Vec<[u8; 3]> = original.chunks_exact(4).filter(|p| p[3] >= 128).map(|p| [p[0], p[1], p[2]]).collect();
    painted.sort_unstable();
    painted.dedup();
    if painted.is_empty() {
        return level.to_vec();
    }
    // Nearest as the eye sees it, roughly: green counts most, blue least.
    let apart = |a: [u8; 3], b: &[u8]| {
        let d = |k: usize| a[k] as i32 - b[k] as i32;
        3 * d(0) * d(0) + 4 * d(1) * d(1) + 2 * d(2) * d(2)
    };
    let mut out = level.to_vec();
    for pixel in out.chunks_exact_mut(4) {
        let nearest = painted.iter().copied().min_by_key(|&colour| apart(colour, pixel)).unwrap();
        pixel[..3].copy_from_slice(&nearest);
    }
    out
}

/// A grey array of one layer, standing in until the block textures are loaded (the march's
/// material, `march.rs`, is made before them).
pub fn placeholder_array() -> Image {
    build_array_image(&[vec![128; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize]], false)
}

/// Stack the layers (each with its mipmaps) into one array image.
fn build_array_image(layers: &[Vec<u8>], painted: bool) -> Image {
    let mut data = Vec::new();
    for layer in layers {
        // Layer-major order: layer 0's full mip chain, then layer 1's, and so on.
        let mut level = layer.clone();
        let mut size = TEXTURE_SIZE;
        data.extend_from_slice(&level);
        for _ in 1..MIP_LEVELS {
            level = shrink(&level, size);
            if painted {
                level = keep_painted(&level, layer);
            }
            size /= 2;
            data.extend_from_slice(&level);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: TEXTURE_SIZE,
            height: TEXTURE_SIZE,
            depth_or_array_layers: layers.len() as u32,
        },
        TextureDimension::D2,
        // Only the full-size level is given here; the mip chain is attached just below.
        layers.concat(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(data);
    image.texture_descriptor.mip_level_count = MIP_LEVELS;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        // Repeat across big merged quads.
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        // Crisp pixels up close…
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        // …and in the distance either one shrunk size (keeping the painted colours) or a smooth
        // hand-over between two (which blends them into new ones).
        mipmap_filter: if painted { ImageFilterMode::Nearest } else { ImageFilterMode::Linear },
        ..default()
    });
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrink_averages_pixels() {
        // A 2 × 2 image of black and white pixels shrinks to one mid-grey pixel.
        let pixels = [0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255];
        assert_eq!(shrink(&pixels, 2), vec![128, 128, 128, 255]);
    }

    #[test]
    fn clear_pixels_dont_darken_coloured_ones() {
        // One solid red pixel and three fully clear (black) ones: the shrunk pixel stays pure,
        // solid red (so thin lines don't vanish in the distance).
        let pixels = [255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(shrink(&pixels, 2), vec![255, 0, 0, 255]);
    }

    #[test]
    fn the_far_colour_is_the_smallest_mipmap_read_as_light() {
        // Black and white checks: the mipmaps average the stored values (mid-grey, 128), which is
        // about 0.22 as light; averaging the light itself would give 0.5, much brighter.
        let checks: Vec<u8> = (0..16).flat_map(|i| if (i % 4 + i / 4) % 2 == 0 { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect();
        let colour = smallest_mip_colour(checks, 4);
        assert!((colour[0] - 0.2158).abs() < 0.01, "{colour:?}");
        assert_eq!(colour[0], colour[2]);
    }

    #[test]
    fn far_versions_keep_the_painted_colours() {
        // Black and white checks shrink to grey, which was never painted: kept painted, it's black or
        // white again.
        let checks: Vec<u8> = (0..4).flat_map(|i| if i % 3 == 0 { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect();
        let shrunk = shrink(&checks, 2);
        assert_eq!(shrunk[..3], [128, 128, 128]);
        let kept = keep_painted(&shrunk, &checks);
        assert!(kept[..3] == [0, 0, 0] || kept[..3] == [255, 255, 255], "{:?}", &kept[..3]);
        assert_eq!(kept[3], 255);
    }

    #[test]
    fn mip_chain_size() {
        let image = build_array_image(&[missing_texture(), missing_texture()], true);
        // Each layer: 32² + 16² + 8² + 4² + 2² + 1² pixels, 4 bytes each.
        let per_layer = (1024 + 256 + 64 + 16 + 4 + 1) * 4;
        assert_eq!(image.data.unwrap().len(), per_layer * 2);
    }
}

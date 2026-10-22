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
//! Without them, pixel art sparkles and shimmers in the distance. For now they're generated
//! automatically by averaging pixels; David may hand-touch them later.

use crate::data::BlockRegistry;
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
use std::path::PathBuf;

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
}

/// Where the game's `assets` folder is. During development (`cargo run`) that's the project
/// folder; in a shipped game it sits next to the program.
pub fn assets_folder() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(dir).join("assets");
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("assets")))
        .unwrap_or_else(|| PathBuf::from("assets"))
}

pub fn build_block_textures(registry: &BlockRegistry) -> BlockTextures {
    let folder = assets_folder().join("textures").join("blocks");

    let mut layers: Vec<Vec<u8>> = vec![missing_texture()];
    let mut layer_names = vec!["<missing>".to_string()];
    let mut problems = Vec::new();
    let mut layer_of: HashMap<String, u32> = HashMap::new();

    let mut face_layers = Vec::with_capacity(registry.len());
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
    }

    BlockTextures {
        image: build_array_image(&layers),
        layer_names,
        problems,
        face_layers,
    }
}

/// Load a PNG as 32 × 32 RGBA pixels, or explain why not.
fn load_texture(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|_| format!("file not found ({})", path.display()))?;
    let image = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::MAIN_WORLD,
    )
    .map_err(|e| format!("not a readable PNG ({e})"))?;
    if image.width() != TEXTURE_SIZE || image.height() != TEXTURE_SIZE {
        return Err(format!(
            "is {}×{}, must be {TEXTURE_SIZE}×{TEXTURE_SIZE}",
            image.width(),
            image.height()
        ));
    }
    let rgba = image
        .convert(TextureFormat::Rgba8UnormSrgb)
        .and_then(|converted| converted.data)
        .ok_or_else(|| "unsupported pixel format".to_string())?;
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
fn shrink(pixels: &[u8], size: u32) -> Vec<u8> {
    let half = size / 2;
    let mut out = Vec::with_capacity((half * half * 4) as usize);
    for y in 0..half {
        for x in 0..half {
            for channel in 0..4 {
                let sample = |dx: u32, dy: u32| {
                    pixels[(((y * 2 + dy) * size + (x * 2 + dx)) * 4 + channel) as usize] as u32
                };
                let sum = sample(0, 0) + sample(1, 0) + sample(0, 1) + sample(1, 1);
                out.push(((sum + 2) / 4) as u8);
            }
        }
    }
    out
}

/// Stack the layers (each with its mipmaps) into one array image.
fn build_array_image(layers: &[Vec<u8>]) -> Image {
    let mut data = Vec::new();
    for layer in layers {
        // Layer-major order: layer 0's full mip chain, then layer 1's, and so on.
        let mut level = layer.clone();
        let mut size = TEXTURE_SIZE;
        data.extend_from_slice(&level);
        for _ in 1..MIP_LEVELS {
            level = shrink(&level, size);
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
        // …and a smooth hand-over between shrunk versions in the distance.
        mipmap_filter: ImageFilterMode::Linear,
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
    fn mip_chain_size() {
        let image = build_array_image(&[missing_texture(), missing_texture()]);
        // Each layer: 32² + 16² + 8² + 4² + 2² + 1² pixels, 4 bytes each.
        let per_layer = (1024 + 256 + 64 + 16 + 4 + 1) * 4;
        assert_eq!(image.data.unwrap().len(), per_layer * 2);
    }
}

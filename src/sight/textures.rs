//! Block textures for sight (3.6, David 2026-11-08: "so the blocks are mipmapped, right? go ahead").
//!
//! Every block texture the blocks name, gathered into one stack for the card, each with its shrunk versions
//! (32, 16, 8, 4, 2, 1 texels a side) made as the far colours are made (`look::shrink`, averaged as stored, read as
//! light on the card): a texture shrunk to one texel is its block's far colour, so where a block is a pixel the
//! textured face and the cell past it agree. And a table: each block's six faces' textures, and which turn at
//! random. The near walk reads it where it meets a face (`march.wgsl`, `hard`).

use crate::data::{BlockRegistry, TextureRotation, block_texture_folder};
use crate::look::{TEXTURE_SIZE, load_texture, shrink};

/// No texture: the face keeps its flat colour.
pub const NONE: u32 = 0xFFFF;
/// The most blocks the table holds (the card's table is a fixed size).
pub const MOST_BLOCKS: usize = 256;
/// Shrunk versions a texture has, itself counted: 32 down to 1.
pub const MIPS: u32 = TEXTURE_SIZE.ilog2() + 1;

/// The textures and the table, ready for the card.
#[derive(Clone, Debug, Default)]
pub struct BlockTextures {
    /// Each texture: its versions from 32 × 32 down to 1 × 1 (RGBA, as stored).
    pub layers: Vec<Vec<Vec<u8>>>,
    /// By block number: each face's texture (`FACE_NAMES`' order: north, south, east, west, top, bottom; `NONE`
    /// for none), and which faces turn at random (a bit each, in that order).
    pub blocks: Vec<([u32; 6], u32)>,
    /// What couldn't be read (a texture's name and why), for the log.
    pub problems: Vec<String>,
}

/// A texture's versions, 32 × 32 down to 1 × 1.
pub fn mips(pixels: Vec<u8>) -> Vec<Vec<u8>> {
    let mut out = vec![pixels];
    let mut size = TEXTURE_SIZE;
    while size > 1 {
        out.push(shrink(out.last().expect("one at least"), size));
        size /= 2;
    }
    out
}

impl BlockTextures {
    /// Every texture the registry's blocks name, read from the texture folder.
    pub fn gather(registry: &BlockRegistry) -> Self {
        Self::from_faces(registry.defs().iter().map(|def| (def.faces.clone(), def.rotations)), |name| load_texture(&block_texture_folder().join(format!("{name}.png"))))
    }

    /// Made from each block's faces' texture names and turns, reading each texture once through `read`.
    pub fn from_faces(blocks: impl Iterator<Item = ([String; 6], [TextureRotation; 6])>, read: impl Fn(&str) -> Result<Vec<u8>, String>) -> Self {
        let mut out = Self::default();
        let mut named: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (faces, rotations) in blocks.take(MOST_BLOCKS) {
            let mut layers = [NONE; 6];
            for (k, name) in faces.iter().enumerate() {
                if name.is_empty() {
                    continue;
                }
                if let Some(&layer) = named.get(name) {
                    layers[k] = layer;
                    continue;
                }
                match read(name) {
                    Ok(pixels) => {
                        let layer = out.layers.len() as u32;
                        out.layers.push(mips(pixels));
                        named.insert(name.clone(), layer);
                        layers[k] = layer;
                    }
                    Err(why) => {
                        out.problems.push(format!("{name}: {why}"));
                        named.insert(name.clone(), NONE);
                    }
                }
            }
            let random = rotations.iter().enumerate().filter(|(_, r)| **r == TextureRotation::Random).fold(0u32, |bits, (k, _)| bits | 1 << k);
            out.blocks.push((layers, random));
        }
        out
    }

    /// The table as the card reads it: two rows of four a block (north, south, east, west; top, bottom, the
    /// random turns, 0), `MOST_BLOCKS` blocks, those past the registry's with no textures.
    pub fn table(&self) -> Vec<u32> {
        let mut words = Vec::with_capacity(MOST_BLOCKS * 8);
        for k in 0..MOST_BLOCKS {
            let (faces, random) = self.blocks.get(k).copied().unwrap_or(([NONE; 6], 0));
            words.extend([faces[0], faces[1], faces[2], faces[3], faces[4], faces[5], random, 0]);
        }
        words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_texture_shrinks_to_its_far_colour() {
        // Half red, half blue (columns): every version the same average, down to one texel.
        let pixels: Vec<u8> = (0..TEXTURE_SIZE * TEXTURE_SIZE).flat_map(|i| if i % TEXTURE_SIZE < TEXTURE_SIZE / 2 { [200, 40, 40, 255] } else { [40, 40, 200, 255] }).collect();
        let versions = mips(pixels);
        assert_eq!(versions.len() as u32, MIPS);
        assert_eq!(versions.last().map(|v| v.len()), Some(4));
        assert_eq!(&versions[MIPS as usize - 1][..3], &[120, 40, 120]);
    }

    #[test]
    fn each_texture_is_read_once_and_the_table_names_them() {
        let read = |name: &str| if name == "missing" { Err("file not found".to_string()) } else { Ok(vec![name.len() as u8; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize]) };
        let grass = (["side".to_string(), "side".to_string(), "side".to_string(), "side".to_string(), "top".to_string(), "dirt".to_string()], [TextureRotation::Fixed, TextureRotation::Fixed, TextureRotation::Fixed, TextureRotation::Fixed, TextureRotation::Random, TextureRotation::Fixed]);
        let air = (Default::default(), Default::default());
        let odd = (std::array::from_fn(|k| if k == 4 { "missing".to_string() } else { "dirt".to_string() }), Default::default());
        let textures = BlockTextures::from_faces([air, grass, odd].into_iter(), read);
        assert_eq!(textures.layers.len(), 3, "side, top, dirt, each once");
        assert_eq!(textures.blocks[0], ([NONE; 6], 0), "air has none");
        assert_eq!(textures.blocks[1], ([0, 0, 0, 0, 1, 2], 1 << 4), "the top turns at random");
        assert_eq!(textures.blocks[2].0, [2, 2, 2, 2, NONE, 2], "a missing texture is none, the rest shared");
        assert_eq!(textures.problems.len(), 1);
        let table = textures.table();
        assert_eq!(table.len(), MOST_BLOCKS * 8);
        assert_eq!(&table[8..16], &[0, 0, 0, 0, 1, 2, 16, 0]);
    }
}

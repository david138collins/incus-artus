//! Block textures for sight (3.6, David 2026-11-08: "so the blocks are mipmapped, right? go ahead").
//!
//! Every block texture the blocks name, gathered into one stack for the card, each with its shrunk versions
//! (32, 16, 8, 4, 2, 1 texels a side) made as the far colours are made (`look::shrink`, averaged as stored, read as
//! light on the card): a texture shrunk to one texel is its block's far colour, so where a block is a pixel the
//! textured face and the cell past it agree. And a table: each block's six faces' textures, and which turn at
//! random. The near walk reads it where it meets a face (`march.wgsl`, `hard`).
//!
//! **Bars' own textures** (build 91, David on 90: "quarterblocks are still greyed"): a block that's a bar (a quarter
//! block) may have a long side and an end of its own (`assets/textures/bars/`, 16 x 32 and 16 x 16, 32 pixels a
//! block as every block's), each pixel repeated to fill a layer, so 0 to 1 across the bar's own width covers it (as
//! the old renderer had them). The table's last word holds them.

use crate::data::{BlockRegistry, TextureRotation, bar_texture_folder, block_texture_folder};
use crate::look::{TEXTURE_SIZE, load_texture, read_png, shrink};

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
    /// By block number: its bars' own long side and end (`NONE` for none: a bar shows a share of the block's).
    pub bars: Vec<[u32; 2]>,
    /// What couldn't be read (a texture's name and why), for the log.
    pub problems: Vec<String>,
    /// By block number, each medium (build 98: water, glass, ice): its colour (linear light) and how fast it dims
    /// what's seen through it (a block's worth; 0: not a medium). The first place is the world's water's, for water
    /// whose block the cells don't keep (`media_table`).
    pub media: Vec<[f32; 4]>,
}

/// A medium as the card reads it: its colour as light, and how fast it dims, so a twentieth is left at its clarity.
pub fn medium_numbers(medium: &crate::data::Medium) -> [f32; 4] {
    let light = |c: u8| {
        let v = c as f32 / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    [light(medium.colour[0]), light(medium.colour[1]), light(medium.colour[2]), 20f32.ln() / medium.clarity.max(0.5)]
}

/// Water's medium when its file gives none (the blue sight drew it before build 98, seen through 16 blocks).
pub const WATER_MEDIUM: crate::data::Medium = crate::data::Medium { colour: [63, 111, 149], clarity: crate::data::CLARITY };

/// A bar's texture (`assets/textures/bars/`): a long side, `wide` x `tall` 16 x 32, or an end, 16 x 16, each
/// pixel repeated to fill a layer (as the old renderer's `load_bar_texture`).
pub fn load_bar_texture(path: &std::path::Path, (wide, tall): (u32, u32)) -> Result<Vec<u8>, String> {
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
        let mut out = Self::from_faces(registry.defs().iter().map(|def| (def.faces.clone(), def.rotations)), |name| load_texture(&block_texture_folder().join(format!("{name}.png"))));
        let bars = registry.defs().iter().map(|def| def.bar.as_ref().filter(|bar| !bar.side.is_empty() && !bar.end.is_empty()).map(|bar| (bar.side.clone(), bar.end.clone())));
        out.add_bars(bars, |name, size| load_bar_texture(&bar_texture_folder().join(format!("{name}.png")), size));
        let water = crate::world::TerrainBlocks::from_registry(registry).water;
        out.media = registry.defs().iter().map(|def| def.medium.as_ref().map_or([0.0; 4], medium_numbers)).collect();
        let water_medium = registry.def(water).and_then(|def| def.medium).unwrap_or(WATER_MEDIUM);
        if let Some(slot) = out.media.get_mut(water.0 as usize) {
            *slot = medium_numbers(&water_medium);
        }
        if let Some(first) = out.media.first_mut() {
            *first = medium_numbers(&water_medium);
        }
        out
    }

    /// The media as the card keeps them (`MOST_BLOCKS` places; the first the world's water's, water's default if
    /// none were gathered).
    pub fn media_table(&self) -> Vec<f32> {
        let mut table = vec![[0.0f32; 4]; MOST_BLOCKS];
        for (slot, medium) in table.iter_mut().zip(&self.media) {
            *slot = *medium;
        }
        if table[0][3] <= 0.0 {
            table[0] = medium_numbers(&WATER_MEDIUM);
        }
        table.into_iter().flatten().collect()
    }

    /// Each block's bars' own textures (a long side and an end, or none), read once each through `read`.
    pub fn add_bars(&mut self, blocks: impl Iterator<Item = Option<(String, String)>>, read: impl Fn(&str, (u32, u32)) -> Result<Vec<u8>, String>) {
        let mut named: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for bar in blocks.take(MOST_BLOCKS) {
            let Some((side, end)) = bar else {
                self.bars.push([NONE; 2]);
                continue;
            };
            let layers = [(side, (16, 32)), (end, (16, 16))].map(|(name, size)| {
                *named.entry(name.clone()).or_insert_with(|| match read(&name, size) {
                    Ok(pixels) => {
                        self.layers.push(mips(pixels));
                        self.layers.len() as u32 - 1
                    }
                    Err(why) => {
                        self.problems.push(format!("{name} (a bar's): {why}"));
                        NONE
                    }
                })
            });
            self.bars.push(layers);
        }
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
    /// random turns, its bars' long side and end, 16 bits each), `MOST_BLOCKS` blocks, those past the registry's
    /// with no textures.
    pub fn table(&self) -> Vec<u32> {
        let mut words = Vec::with_capacity(MOST_BLOCKS * 8);
        for k in 0..MOST_BLOCKS {
            let (faces, random) = self.blocks.get(k).copied().unwrap_or(([NONE; 6], 0));
            let [side, end] = self.bars.get(k).copied().unwrap_or([NONE; 2]);
            words.extend([faces[0], faces[1], faces[2], faces[3], faces[4], faces[5], random, (side & 0xFFFF) | (end & 0xFFFF) << 16]);
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
        assert_eq!(&table[8..16], &[0, 0, 0, 0, 1, 2, 16, NONE | NONE << 16]);
    }

    #[test]
    fn a_bars_own_textures_go_in_the_table() {
        let read = |name: &str, (wide, tall): (u32, u32)| if name == "missing" { Err("file not found".to_string()) } else { Ok(vec![(wide + tall) as u8; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize]) };
        let mut textures = BlockTextures::default();
        textures.add_bars([None, Some(("brick_side".to_string(), "brick_end".to_string())), Some(("brick_side".to_string(), "missing".to_string()))].into_iter(), read);
        assert_eq!(textures.bars, vec![[NONE; 2], [0, 1], [0, NONE]], "each read once, a missing one none");
        assert_eq!(textures.layers[0][0][0], 48, "the long side read as 16 x 32");
        assert_eq!(textures.layers[1][0][0], 32, "the end as 16 x 16");
        assert_eq!(textures.table()[15], 1 << 16, "block 1's bars: side 0, end 1");
    }
}

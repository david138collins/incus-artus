//! The master palette: every colour the game's art uses, each with a permanent number (its slot),
//! kept in `assets/data/palette.toml`. The Registrar's Color Lab is the only thing that writes it;
//! the world, FX and UI read it (docs/worldgen.md, "The Registrar and its Color Lab").
//!
//! - **Slots are permanent.** A colour keeps its number whatever happens to the others: inserting a
//!   shade into a spline, or removing one, never renumbers anything, and a removed number is never
//!   given out again (so nothing old can point at the wrong colour).
//! - **Regulated or not.** A colour on one of the splines is regulated; one that isn't (painted by
//!   hand, or not sorted yet) is unregulated. An asset is regulated when every colour it uses is.
//! - **The art stays what David paints.** Block textures are ordinary PNGs; each pixel's slot is
//!   the slot with exactly its colour ("a new colour means an exact match", so nothing slips in by
//!   a hair). When the Lab changes a colour, it rewrites the pixels with it at the same time.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A colour as stored: red, green, blue, 0-255 (as painted, sRGB).
pub type Rgb = [u8; 3];

/// One shade in the palette.
#[derive(Clone, Debug, PartialEq)]
pub struct Colour {
    pub rgb: Rgb,
}

/// A spline: one hue's ramp of shades, darkest first, as slots. (Shaped by HSL curves later; for
/// now just its shades.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spline {
    pub name: String,
    pub steps: Vec<u32>,
}

/// The whole palette.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MasterPalette {
    /// Every colour, by slot.
    pub colours: BTreeMap<u32, Colour>,
    pub splines: Vec<Spline>,
    /// The next slot to give out: never one used before, even if it's gone since.
    pub next_slot: u32,
}

/// `assets/data/palette.toml`.
pub fn palette_file() -> PathBuf {
    super::data_folder().join("palette.toml")
}

/// The file as written.
#[derive(Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct PaletteFile {
    next_slot: u32,
    #[serde(rename = "spline")]
    splines: Vec<Spline>,
    #[serde(rename = "colour")]
    colours: Vec<ColourEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ColourEntry {
    slot: u32,
    /// "#rrggbb", as Aseprite and most paint programs write it.
    rgb: String,
}

pub fn to_hex([r, g, b]: Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn from_hex(text: &str) -> Option<Rgb> {
    let hex = text.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let part = |k: usize| u8::from_str_radix(&hex[k..k + 2], 16).ok();
    Some([part(0)?, part(2)?, part(4)?])
}

impl MasterPalette {
    /// From the file's text. Mistakes are refused with a reason, never half-read.
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: PaletteFile = toml::from_str(text).map_err(|error| error.to_string())?;
        let mut palette = MasterPalette { next_slot: file.next_slot.max(1), ..Default::default() };
        for entry in file.colours {
            let rgb = from_hex(&entry.rgb).ok_or_else(|| format!("slot {}: '{}' isn't a #rrggbb colour", entry.slot, entry.rgb))?;
            if palette.colours.insert(entry.slot, Colour { rgb }).is_some() {
                return Err(format!("slot {} is listed twice", entry.slot));
            }
            palette.next_slot = palette.next_slot.max(entry.slot + 1);
        }
        for spline in &file.splines {
            if let Some(missing) = spline.steps.iter().find(|slot| !palette.colours.contains_key(slot)) {
                return Err(format!("spline '{}' names slot {missing}, which has no colour", spline.name));
            }
        }
        palette.splines = file.splines;
        Ok(palette)
    }

    /// The file's text: colours in slot order, one to a line, so a change shows as one line.
    pub fn to_text(&self) -> String {
        let mut text = String::from(
            "# The master palette: every colour the game's art uses, each with a permanent number (slot).\n\
             # Written by the Registrar's Color Lab; edit it there rather than here.\n\n",
        );
        text += &format!("next_slot = {}\n", self.next_slot);
        for spline in &self.splines {
            let steps: Vec<String> = spline.steps.iter().map(u32::to_string).collect();
            text += &format!("\n[[spline]]\nname = {:?}\nsteps = [{}]\n", spline.name, steps.join(", "));
        }
        text += "\n";
        for (slot, colour) in &self.colours {
            text += &format!("[[colour]]\nslot = {slot}\nrgb = \"{}\"\n", to_hex(colour.rgb));
        }
        text
    }

    /// The palette in its file, or an empty one if there's no file yet.
    pub fn load_from(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self { next_slot: 1, ..Default::default() }),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.to_text()).map_err(|error| error.to_string())
    }

    /// The slot with exactly this colour (the lowest, if two share it).
    pub fn slot_of(&self, rgb: Rgb) -> Option<u32> {
        self.colours.iter().find(|(_, colour)| colour.rgb == rgb).map(|(&slot, _)| slot)
    }

    /// The spline a slot is on, if any.
    pub fn spline_of(&self, slot: u32) -> Option<&Spline> {
        self.splines.iter().find(|spline| spline.steps.contains(&slot))
    }

    pub fn is_regulated(&self, slot: u32) -> bool {
        self.spline_of(slot).is_some()
    }

    /// A new colour, unregulated, in a slot never used before.
    pub fn add(&mut self, rgb: Rgb) -> u32 {
        let slot = self.next_slot.max(1);
        self.colours.insert(slot, Colour { rgb });
        self.next_slot = slot + 1;
        slot
    }

    /// Every colour in `used` the palette doesn't have yet joins it, unregulated. Returns the new
    /// slots (none: nothing changed).
    pub fn take_in(&mut self, used: impl IntoIterator<Item = Rgb>) -> Vec<u32> {
        let mut added = Vec::new();
        for rgb in used {
            if self.slot_of(rgb).is_none() {
                added.push(self.add(rgb));
            }
        }
        added
    }
}

/// The colours a texture uses, with how many pixels each: only the pixels that show (fully clear
/// ones have no colour), most used first. `pixels`: RGBA.
pub fn colours_in(pixels: &[u8]) -> Vec<(Rgb, u32)> {
    let mut counts: BTreeMap<Rgb, u32> = BTreeMap::new();
    for pixel in pixels.chunks_exact(4).filter(|pixel| pixel[3] > 0) {
        *counts.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
    }
    let mut colours: Vec<(Rgb, u32)> = counts.into_iter().collect();
    colours.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    colours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_never_given_out_twice() {
        let mut palette = MasterPalette::default();
        let (a, b) = (palette.add([10, 20, 30]), palette.add([40, 50, 60]));
        assert_eq!((a, b), (1, 2));
        palette.colours.remove(&b);
        // The removed number isn't used again.
        assert_eq!(palette.add([70, 80, 90]), 3);
    }

    #[test]
    fn new_colours_join_unregulated_and_known_ones_find_their_slot() {
        let mut palette = MasterPalette::default();
        let ochre = palette.add([200, 150, 60]);
        palette.splines.push(Spline { name: "ochre".to_string(), steps: vec![ochre] });
        let added = palette.take_in([[200, 150, 60], [201, 150, 60], [201, 150, 60]]);
        // Only the exact match is known: a hair off is new (and only joins once).
        assert_eq!(added.len(), 1);
        assert_eq!(palette.slot_of([200, 150, 60]), Some(ochre));
        assert!(palette.is_regulated(ochre) && !palette.is_regulated(added[0]));
    }

    #[test]
    fn the_file_reads_back_as_written() {
        let mut palette = MasterPalette::default();
        let (a, b) = (palette.add([255, 0, 128]), palette.add([1, 2, 3]));
        palette.splines.push(Spline { name: "pink".to_string(), steps: vec![b, a] });
        let gone = palette.add([9, 9, 9]);
        palette.colours.remove(&gone);
        let again = MasterPalette::parse(&palette.to_text()).unwrap();
        assert_eq!(again, palette);
        assert_eq!(again.next_slot, 4, "a removed slot's number stays used");
        // Mistakes are refused with a reason.
        assert!(MasterPalette::parse("[[colour]]\nslot = 1\nrgb = \"#12345\"").unwrap_err().contains("slot 1"));
        assert!(MasterPalette::parse("[[spline]]\nname = \"x\"\nsteps = [5]").unwrap_err().contains("slot 5"));
    }

    #[test]
    fn a_texture_s_colours_are_counted_and_clear_pixels_skipped() {
        let pixels = [[9, 9, 9, 255], [1, 1, 1, 255], [9, 9, 9, 255], [5, 5, 5, 0]].concat();
        assert_eq!(colours_in(&pixels), vec![([9, 9, 9], 2), ([1, 1, 1], 1)]);
    }
}

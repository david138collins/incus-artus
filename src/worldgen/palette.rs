//! Palettes — colouring a field with a palette of David's, for art and later for the map.
//!
//! - **Palettes are GIMP palette files (`.gpl`)**, the plain text format Aseprite, GIMP and Krita all
//!   read and write. They live in the `palettes/` folder next to `assets/`.
//! - **Spreading:** the palette's colours are laid over the field from the lowest values (valleys) to
//!   the highest (peaks), in the palette's order. Either each colour gets an equal slice of the
//!   *values* (bands of equal height, like contour lines), or an equal share of the *pixels* (every
//!   colour covers the same amount of the picture).
//! - **Blend:** hard bands (pixel-art posterising) or smooth gradients between the colours.
//! - **The HSL filter** works on each colour's hue, saturation and lightness separately: keep it,
//!   set it to one value for every colour, or take it from the field (the noise's own value). So
//!   "the greens I'm using" with lightness from the field gives the palette's hues shaded by the noise.
//! - **The filtered palette can be exported** as a `.gpl` again.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The `palettes/` folder.
pub fn palette_folder() -> PathBuf {
    crate::debug::project_folder().join("palettes")
}

/// The `.gpl` files in the palettes folder, by name, in alphabetical order.
pub fn saved_palettes() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(palette_folder())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("gpl")))
        .filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

/// Reads a GIMP palette. Lines are "R G B name"; the header, `Name:`, `Columns:` and `#` comments
/// are skipped.
pub fn read_gpl(text: &str) -> Result<Vec<[u8; 3]>, String> {
    let mut lines = text.lines();
    if !lines.next().is_some_and(|first| first.trim().starts_with("GIMP Palette")) {
        return Err("not a GIMP palette (the first line should say \"GIMP Palette\")".into());
    }
    let mut colours = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.contains(':') && !line.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let numbers: Vec<u8> = line.split_whitespace().take(3).filter_map(|part| part.parse().ok()).collect();
        if let [r, g, b] = numbers[..] {
            colours.push([r, g, b]);
        }
    }
    if colours.is_empty() {
        return Err("the palette has no colours in it".into());
    }
    Ok(colours)
}

pub fn write_gpl(name: &str, colours: &[[u8; 3]]) -> String {
    let mut text = format!("GIMP Palette\nName: {name}\nColumns: 0\n# Made in the Incus Artus World Gen suite\n");
    for [r, g, b] in colours {
        text.push_str(&format!("{r:3} {g:3} {b:3}\t#{r:02X}{g:02X}{b:02X}\n"));
    }
    text
}

pub fn load_palette(name: &str) -> Result<Vec<[u8; 3]>, String> {
    let path = palette_folder().join(format!("{name}.gpl"));
    let text = std::fs::read_to_string(&path).map_err(|error| format!("couldn't read {}: {error}", path.display()))?;
    read_gpl(&text)
}

pub fn save_palette(folder: &Path, name: &str, colours: &[[u8; 3]]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let path = folder.join(format!("{name}.gpl"));
    std::fs::write(&path, write_gpl(name, colours)).map_err(|error| error.to_string())?;
    Ok(path)
}

// ── Hue, saturation, lightness ─────────────────────────────────────────────

/// Hue in degrees (0–360), saturation and lightness 0–1.
pub fn to_hsl([r, g, b]: [u8; 3]) -> [f32; 3] {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let (high, low) = (r.max(g).max(b), r.min(g).min(b));
    let lightness = (high + low) / 2.0;
    let span = high - low;
    if span <= 1e-6 {
        return [0.0, 0.0, lightness];
    }
    let saturation = span / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if high == r {
        60.0 * ((g - b) / span).rem_euclid(6.0)
    } else if high == g {
        60.0 * ((b - r) / span + 2.0)
    } else {
        60.0 * ((r - g) / span + 4.0)
    };
    [hue, saturation.min(1.0), lightness]
}

pub fn from_hsl([hue, saturation, lightness]: [f32; 3]) -> [u8; 3] {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let h = hue.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = lightness - chroma / 2.0;
    let byte = |v: f32| ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(r), byte(g), byte(b)]
}

/// What the HSL filter does with one of hue, saturation or lightness.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Channel {
    /// Leave it as the palette has it.
    Keep,
    /// Every colour gets this value (hue in degrees, the others 0–1).
    Set(f32),
    /// Take it from the field's value under each pixel (hue: 0–360° across black to white).
    FromField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Spread {
    /// Each colour gets an equal slice of the values, like contour bands.
    EqualValues,
    /// Each colour covers an equal share of the pixels.
    EqualShares,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colouring {
    /// Colour the field at all (off = plain greyscale).
    pub on: bool,
    /// The loaded palette's name, and its colours as read from the file.
    pub name: String,
    pub colours: Vec<[u8; 3]>,
    pub spread: Spread,
    pub blend: bool,
    /// First colour on the peaks instead of in the valleys.
    pub reverse: bool,
    pub hue: Channel,
    pub saturation: Channel,
    pub lightness: Channel,
}

impl Default for Colouring {
    fn default() -> Self {
        Self {
            on: false,
            name: String::new(),
            colours: Vec::new(),
            spread: Spread::EqualShares,
            blend: false,
            reverse: false,
            hue: Channel::Keep,
            saturation: Channel::Keep,
            lightness: Channel::Keep,
        }
    }
}

impl Colouring {
    /// The palette after the HSL filter's "set to" choices ("from the field" happens per pixel).
    pub fn filtered(&self) -> Vec<[u8; 3]> {
        self.colours
            .iter()
            .map(|&colour| {
                let mut hsl = to_hsl(colour);
                for (value, channel) in hsl.iter_mut().zip([self.hue, self.saturation, self.lightness]) {
                    if let Channel::Set(set) = channel {
                        *value = set;
                    }
                }
                from_hsl(hsl)
            })
            .collect()
    }

    fn any_from_field(&self) -> bool {
        [self.hue, self.saturation, self.lightness].contains(&Channel::FromField)
    }

    /// Colours a whole field (values 0–1). `sorted` is the same values in order, lowest first
    /// (worked out once per field, for "equal shares"). Falls back to grey when off or without a
    /// palette.
    pub fn paint(&self, values: &[f32], sorted: &[f32]) -> Vec<[u8; 3]> {
        let grey = |v: f32| [(v.clamp(0.0, 1.0) * 255.0).round() as u8; 3];
        if !self.on || self.colours.is_empty() {
            return values.iter().map(|&v| grey(v)).collect();
        }
        let mut colours = self.filtered();
        if self.reverse {
            colours.reverse();
        }
        let n = colours.len();

        // Where each colour's slice begins: equal steps of value, or the values that split the
        // pixels into equal shares.
        let starts: Vec<f32> = match self.spread {
            Spread::EqualValues => (0..=n).map(|k| k as f32 / n as f32).collect(),
            Spread::EqualShares => {
                let mut starts: Vec<f32> =
                    (0..=n).map(|k| sorted[(k * (sorted.len() - 1)) / n]).collect();
                starts[0] = starts[0].min(0.0);
                starts[n] = starts[n].max(1.0);
                starts
            }
        };

        let from_field = self.any_from_field();
        values
            .iter()
            .map(|&v| {
                // Which slice, and how far through it.
                let slice = starts[1..n].partition_point(|&start| start <= v);
                let (low, high) = (starts[slice], starts[slice + 1]);
                let through = if high > low { ((v - low) / (high - low)).clamp(0.0, 1.0) } else { 0.5 };
                let mut colour = if self.blend {
                    // Each colour sits at the middle of its slice; blend toward the neighbour.
                    let position = (slice as f32 + through - 0.5).clamp(0.0, (n - 1) as f32);
                    let (a, b) = (position.floor() as usize, (position.floor() as usize + 1).min(n - 1));
                    let f = position - position.floor();
                    let mix = |i: usize| (colours[a][i] as f32 + (colours[b][i] as f32 - colours[a][i] as f32) * f).round() as u8;
                    [mix(0), mix(1), mix(2)]
                } else {
                    colours[slice]
                };
                if from_field {
                    let mut hsl = to_hsl(colour);
                    let field = [v * 360.0, v, v];
                    for ((value, channel), from) in hsl.iter_mut().zip([self.hue, self.saturation, self.lightness]).zip(field) {
                        if channel == Channel::FromField {
                            *value = from;
                        }
                    }
                    colour = from_hsl(hsl);
                }
                colour
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREENS: &str = "GIMP Palette\nName: greens\nColumns: 4\n# a comment\n 10  40  10\tdark\n 60 120  50\tmid\n150 220 110\tlight\n";

    #[test]
    fn a_gimp_palette_reads_and_writes() {
        let colours = read_gpl(GREENS).unwrap();
        assert_eq!(colours, vec![[10, 40, 10], [60, 120, 50], [150, 220, 110]]);
        assert_eq!(read_gpl(&write_gpl("greens", &colours)).unwrap(), colours);
        assert!(read_gpl("not a palette").is_err());
    }

    #[test]
    fn hsl_goes_there_and_back() {
        for colour in [[10, 40, 10], [255, 0, 0], [128, 128, 128], [12, 200, 255], [0, 0, 0], [255, 255, 255]] {
            let back = from_hsl(to_hsl(colour));
            for i in 0..3 {
                assert!((back[i] as i32 - colour[i] as i32).abs() <= 1, "{colour:?} came back as {back:?}");
            }
        }
    }

    #[test]
    fn equal_shares_give_every_colour_the_same_amount_of_picture() {
        let colouring = Colouring { on: true, colours: read_gpl(GREENS).unwrap(), ..Colouring::default() };
        // Lopsided values: mostly dark.
        let values: Vec<f32> = (0..3000).map(|i| (i as f32 / 3000.0).powi(3)).collect();
        let painted = colouring.paint(&values, &values);
        for colour in &colouring.colours {
            let count = painted.iter().filter(|c| *c == colour).count();
            assert!((900..1100).contains(&count), "{colour:?} covers {count} of 3000");
        }
    }

    #[test]
    fn setting_saturation_to_zero_makes_greys() {
        let colouring = Colouring { colours: read_gpl(GREENS).unwrap(), saturation: Channel::Set(0.0), ..Colouring::default() };
        for [r, g, b] in colouring.filtered() {
            assert!(r == g && g == b);
        }
    }
}

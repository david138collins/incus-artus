//! Ironquill, the interface's letters, laid from its letter sheet the way its own file places them (`Ironquill.fnt`,
//! BMFont): that is Windows' own laying of the font, to the pixel (his four pictures checked, 2026-11-20). It's read
//! from `Art/fonts/Ironquill/` beside his other fonts, and never committed: its licence lets a built game carry it but
//! forbids putting its files "in a public repository (GitHub and similar)".

use super::sheet::{Colour, Sheet};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

/// A line's height (the file's `lineHeight`): his sizes all follow it (the shared page, 6).
pub const LINE: i32 = 15;
/// The base line, from a line's top (the file's `base`): an underline goes a pixel under it.
pub const BASE: i32 = 12;

/// One letter: its place on the letter sheet, its size, where it sits from the pen, and how far the pen moves; and the
/// first and last of its columns that have ink (none for a space).
#[derive(Clone, Copy, Debug)]
struct Letter {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    x_offset: i32,
    y_offset: i32,
    advance: i32,
    ink: Option<(i32, i32)>,
}

pub struct Letters {
    letters: HashMap<char, Letter>,
    /// The letter sheet, whether each pixel is ink.
    ink: Vec<bool>,
    across: i32,
}

impl Letters {
    /// The font in `folder`: `Ironquill.fnt` and the letter sheet it names.
    pub fn load(folder: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(folder.join("Ironquill.fnt")).map_err(|error| format!("Ironquill.fnt: {error}"))?;
        let mut letters = HashMap::new();
        for line in text.lines().filter(|line| line.starts_with("char ")) {
            let field = |name: &str| line.split_whitespace().find_map(|part| part.strip_prefix(name).and_then(|rest| rest.strip_prefix('='))).and_then(|value| value.parse::<i32>().ok());
            let (Some(id), Some(x), Some(y), Some(width), Some(height), Some(x_offset), Some(y_offset), Some(advance)) =
                (field("id"), field("x"), field("y"), field("width"), field("height"), field("xoffset"), field("yoffset"), field("xadvance"))
            else {
                return Err(format!("Ironquill.fnt: a letter's line can't be read: {line}"));
            };
            if let Some(c) = char::from_u32(id as u32) {
                letters.insert(c, Letter { x, y, width, height, x_offset, y_offset, advance, ink: None });
            }
        }
        let (across, _, rgba) = crate::textures::read_png(&folder.join("Ironquill.png")).ok_or("Ironquill.png can't be read")?;
        let ink: Vec<bool> = rgba.chunks_exact(4).map(|pixel| pixel[3] > 127).collect();
        let across = across as i32;
        for letter in letters.values_mut() {
            let inked = |i: i32| (0..letter.height).any(|j| ink.get(((letter.y + j) * across + letter.x + i) as usize).copied().unwrap_or(false));
            let columns: Vec<i32> = (0..letter.width).filter(|i| inked(*i)).collect();
            letter.ink = columns.first().zip(columns.last()).map(|(first, last)| (*first, *last));
        }
        Ok(Self { letters, ink, across })
    }

    /// Where `text`'s ink begins and ends, its first and last inked columns counted from the pen's start (none for
    /// nothing but spaces).
    pub fn ink(&self, text: &str) -> Option<(i32, i32)> {
        let mut pen = 0;
        let mut found: Option<(i32, i32)> = None;
        for letter in text.chars().filter_map(|c| self.letter(c)) {
            if let Some((first, last)) = letter.ink {
                let (left, right) = (pen + letter.x_offset + first, pen + letter.x_offset + last);
                found = Some(found.map_or((left, right), |(l, r)| (l.min(left), r.max(right))));
            }
            pen += letter.advance;
        }
        found
    }

    /// A letter, or the question mark for one the font hasn't got.
    fn letter(&self, c: char) -> Option<&Letter> {
        self.letters.get(&c).or_else(|| self.letters.get(&'?'))
    }

    /// How wide `text` is laid: the pen's moves, as Windows measures it.
    pub fn width(&self, text: &str) -> i32 {
        text.chars().filter_map(|c| self.letter(c)).map(|letter| letter.advance).sum()
    }

    /// `text` laid with its line's top left at `x`, `y`, in `colour`: how wide it was.
    pub fn draw(&self, sheet: &mut Sheet, x: i32, y: i32, text: &str, colour: Colour) -> i32 {
        let mut pen = x;
        for letter in text.chars().filter_map(|c| self.letter(c)) {
            for j in 0..letter.height {
                for i in 0..letter.width {
                    if self.ink[((letter.y + j) * self.across + letter.x + i) as usize] {
                        sheet.px(pen + letter.x_offset + i, y + letter.y_offset + j, colour);
                    }
                }
            }
            pen += letter.advance;
        }
        pen - x
    }
}

/// Ironquill, read once (none if its files aren't there: said once in the session's log, and no words are drawn).
pub fn ironquill() -> Option<&'static Letters> {
    static FONT: OnceLock<Option<Letters>> = OnceLock::new();
    FONT.get_or_init(|| {
        let folder = crate::session::project_folder().join("Art").join("fonts").join("Ironquill");
        Letters::load(&folder).map_err(|error| log::error!("Ironquill couldn't be read from {} ({error}): the interface has no words", folder.display())).ok()
    })
    .as_ref()
}

/// How wide `text` is in Ironquill (0 without it).
pub fn width(text: &str) -> i32 {
    ironquill().map_or(0, |letters| letters.width(text))
}

/// `text` in Ironquill at `x`, `y` (its line's top left), in `colour`: how wide it was.
pub fn draw(sheet: &mut Sheet, x: i32, y: i32, text: &str, colour: Colour) -> i32 {
    ironquill().map_or(0, |letters| letters.draw(sheet, x, y, text, colour))
}

/// Where `text`'s ink begins and ends in Ironquill, from the pen's start ((0, -1) without it, or for nothing but spaces).
pub fn ink(text: &str) -> (i32, i32) {
    ironquill().and_then(|letters| letters.ink(text)).unwrap_or((0, -1))
}

/// The width of `text`'s first letter, for its underline.
pub fn first_width(text: &str) -> i32 {
    text.chars().next().map_or(0, |c| width(&c.to_string()))
}

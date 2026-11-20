//! A sheet: a picture of a window drawn on the processor, four bytes a pixel (red, green, blue, and 255 where it's
//! there, 0 where what's under it shows), with the 95 scheme's colours and Windows' edges.

use std::sync::atomic::{AtomicU64, Ordering};

/// A colour: red, green, blue, and 255 (there) or 0 (clear).
pub type Colour = [u8; 4];

// The 95 scheme, by the names the shared page gives them (`journal/docs/editor-look.md`, 3).
/// F, the face of everything.
pub const FACE: Colour = [0xC0, 0xC0, 0xC0, 255];
/// W.
pub const WHITE: Colour = [0xFF, 0xFF, 0xFF, 255];
/// L, the light edge.
pub const LIGHT: Colour = [0xDF, 0xDF, 0xDF, 255];
/// S, the shadow.
pub const SHADOW: Colour = [0x80, 0x80, 0x80, 255];
/// K.
pub const BLACK: Colour = [0x00, 0x00, 0x00, 255];
/// The title bars' blue, and a chosen row's.
pub const NAVY: Colour = [0x00, 0x00, 0x80, 255];
/// The Software map's board.
pub const TEAL: Colour = [0x00, 0x80, 0x80, 255];
/// Nothing: what's under the sheet shows.
pub const CLEAR: Colour = [0, 0, 0, 0];

/// Windows' edges (its DrawEdge), as the shared page measured them (4): each two rings from the outside in, a ring's
/// top and left in its first colour, its bottom and right in its second.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Edge {
    /// A window's frame, the drop-down's button, a scroll arrow's: light and black outside, white and grey inside.
    RaisedWindow,
    /// A title button's, a tab's page: white and black outside, light and grey inside.
    RaisedButton,
    /// A field's, a list's, a view's: grey and white outside, black and light inside.
    Sunken,
    /// A status bar's pane: grey and white, one ring.
    SunkenOneLine,
    /// A button held down: black and white outside, grey and light inside.
    HeldDown,
}

impl Edge {
    pub fn rings(self) -> &'static [(Colour, Colour)] {
        match self {
            Edge::RaisedWindow => &[(LIGHT, BLACK), (WHITE, SHADOW)],
            Edge::RaisedButton => &[(WHITE, BLACK), (LIGHT, SHADOW)],
            Edge::Sunken => &[(SHADOW, WHITE), (BLACK, LIGHT)],
            Edge::SunkenOneLine => &[(SHADOW, WHITE)],
            Edge::HeldDown => &[(BLACK, WHITE), (SHADOW, LIGHT)],
        }
    }
}

/// Each sheet's own number, so the card knows which one it holds.
static NEXT: AtomicU64 = AtomicU64::new(1);

pub struct Sheet {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    id: u64,
    /// How many times it has been drawn on: the card sends it again when this differs from what it last sent.
    version: u64,
}

impl Sheet {
    /// A sheet `width` x `height`, clear.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, pixels: vec![0; (width * height * 4) as usize], id: NEXT.fetch_add(1, Ordering::Relaxed), version: 1 }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Its pixels, a row after another, four bytes each.
    pub fn bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// Made `width` x `height` and cleared, if it isn't that size already: whether it was made afresh.
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        if (width, height) == (self.width, self.height) {
            return false;
        }
        *self = Self { id: self.id, version: self.version + 1, ..Self::new(width, height) };
        true
    }

    /// The colour at `x`, `y` (clear outside the sheet).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn get(&self, x: i32, y: i32) -> Colour {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return CLEAR;
        }
        let at = ((y as u32 * self.width + x as u32) * 4) as usize;
        [self.pixels[at], self.pixels[at + 1], self.pixels[at + 2], self.pixels[at + 3]]
    }

    /// One pixel (none outside the sheet).
    pub fn px(&mut self, x: i32, y: i32, colour: Colour) {
        self.fill(x, y, 1, 1, colour);
    }

    /// A rectangle, the part on the sheet.
    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, colour: Colour) {
        let (x0, y0) = (x.max(0), y.max(0));
        let (x1, y1) = ((x + w).min(self.width as i32), (y + h).min(self.height as i32));
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for row in y0..y1 {
            let start = ((row as u32 * self.width + x0 as u32) * 4) as usize;
            for pixel in self.pixels[start..start + ((x1 - x0) * 4) as usize].chunks_exact_mut(4) {
                pixel.copy_from_slice(&colour);
            }
        }
        self.version += 1;
    }

    /// One ring of an edge round the rectangle: its top and left lines in `top_left`, its bottom and right in
    /// `bottom_right`; the top's last pixel and the left's are the bottom and right's, as Windows' DrawEdge has them.
    pub fn ring(&mut self, x: i32, y: i32, w: i32, h: i32, top_left: Colour, bottom_right: Colour) {
        self.fill(x, y, w - 1, 1, top_left);
        self.fill(x, y, 1, h - 1, top_left);
        self.fill(x, y + h - 1, w, 1, bottom_right);
        self.fill(x + w - 1, y, 1, h, bottom_right);
    }

    /// An edge round the rectangle, its rings from the outside in.
    pub fn edge(&mut self, x: i32, y: i32, w: i32, h: i32, edge: Edge) {
        for (k, (top_left, bottom_right)) in edge.rings().iter().enumerate() {
            let k = k as i32;
            self.ring(x + k, y + k, w - 2 * k, h - 2 * k, *top_left, *bottom_right);
        }
    }

    /// A picture's rows at `x`, `y`, a letter a pixel (`marks.rs`): W white, L light, . face, S grey, K black, n navy;
    /// a space leaves the sheet as it is.
    pub fn mark(&mut self, x: i32, y: i32, rows: &[&str]) {
        for (j, row) in rows.iter().enumerate() {
            for (i, letter) in row.chars().enumerate() {
                let colour = match letter {
                    'W' => WHITE,
                    'L' => LIGHT,
                    '.' => FACE,
                    'S' => SHADOW,
                    'K' => BLACK,
                    'n' => NAVY,
                    _ => continue,
                };
                self.px(x + i as i32, y + j as i32, colour);
            }
        }
    }

    /// A mark's rows at `x`, `y` in one colour, wherever they aren't a space (a menu's tick, in its words' colour).
    pub fn glyph(&mut self, x: i32, y: i32, rows: &[&str], colour: Colour) {
        for (j, row) in rows.iter().enumerate() {
            for (i, letter) in row.chars().enumerate() {
                if letter != ' ' {
                    self.px(x + i as i32, y + j as i32, colour);
                }
            }
        }
    }

    /// A picture (red, green, blue and opacity, a byte each, `width` across) at `x`, `y`, in `colour` wherever it's there.
    pub fn stamp(&mut self, x: i32, y: i32, width: u32, rgba: &[u8], colour: Colour) {
        for (k, pixel) in rgba.chunks_exact(4).enumerate() {
            if pixel[3] > 127 {
                self.px(x + (k as u32 % width) as i32, y + (k as u32 / width) as i32, colour);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring as Windows draws it: the top-right and bottom-left corners are the bottom and right's colour.
    #[test]
    fn a_ring_has_windows_corners() {
        let mut sheet = Sheet::new(6, 5);
        sheet.ring(0, 0, 6, 5, WHITE, BLACK);
        assert_eq!((sheet.get(0, 0), sheet.get(4, 0), sheet.get(5, 0)), (WHITE, WHITE, BLACK));
        assert_eq!((sheet.get(0, 3), sheet.get(0, 4), sheet.get(5, 4)), (WHITE, BLACK, BLACK));
        assert_eq!(sheet.get(2, 2), CLEAR);
    }

    /// Drawing off the sheet's edges draws only the part on it.
    #[test]
    fn drawing_past_the_edges_is_cut() {
        let mut sheet = Sheet::new(4, 4);
        sheet.fill(-2, -2, 4, 4, NAVY);
        sheet.fill(3, 3, 9, 9, TEAL);
        assert_eq!((sheet.get(0, 0), sheet.get(1, 1), sheet.get(2, 2), sheet.get(3, 3)), (NAVY, NAVY, CLEAR, TEAL));
    }
}

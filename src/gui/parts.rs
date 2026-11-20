//! The pieces windows are made of, each drawn once here for the editor and the game: the window's thick frame, the
//! title bar and its buttons, the menu's words, the drop-down and its list, the line cut in, tabs and their page, the
//! greyed scrollbar, the thick sunk frame, the status bar's panes and the grip. Their sizes are the shared page's (6),
//! measured on the control sheet and his pictures; where they differ, his ruling (named where it's used).

use super::letters::{self, BASE, LINE};
use super::marks;
use super::sheet::{BLACK, CLEAR, Colour, Edge, FACE, LIGHT, NAVY, SHADOW, Sheet, WHITE};

/// A title bar's height (CAPH).
pub const CAPTION: i32 = 20;
/// The menu's row (MENUH).
pub const MENU: i32 = 21;
/// A field's height, and a drop-down's (FH).
pub const FIELD: i32 = 23;
/// A tab's height: none rises when chosen (his ruling, 2026-11-20: "make it not get taller").
pub const TAB: i32 = 21;
/// A status bar's pane (SBH).
pub const PANE: i32 = 22;
/// A scrollbar's width, and its arrows' buttons' side.
pub const SCROLL: i32 = 16;
/// A row of a list (his ruling: "go with 15 and 4").
pub const ROW: i32 = 15;
/// A title button's side.
pub const BUTTON: i32 = 16;
/// Between the menu's words.
const WORDS_APART: i32 = 12;
/// Between a name's ink and the lines round it, in a row of tabs or panes: 2, and no more [his word, 2026-11-20: "just
/// go a shade denser were nothing touches within 2px"; "generally all rows should be as dense as possible"].
pub const CLEARANCE: i32 = 2;
/// A tab's edge each side, and a pane's line.
const TAB_EDGE: i32 = 2;
const PANE_LINE: i32 = 1;
/// The grip's corner.
pub const GRIP: i32 = 14;

/// A rectangle on a sheet: its top left and its size.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn holds(self, (x, y): (i32, i32)) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

/// A window `w` x `h` at `x`, `y`: Windows' thick frame (its raised edge, then two rows of face) and the face inside.
/// It may run past the sheet: his pictures are 1,281 x 801 windows cut a pixel short at the right and the bottom.
pub fn window(sheet: &mut Sheet, x: i32, y: i32, w: i32, h: i32) {
    sheet.fill(x, y, w, h, FACE);
    sheet.edge(x, y, w, h, Edge::RaisedWindow);
}

/// A title bar across `w`, its words or lettering laid by the caller.
pub fn title_bar(sheet: &mut Sheet, x: i32, y: i32, w: i32) {
    sheet.fill(x, y, w, CAPTION, NAVY);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TitleButton {
    Minimise,
    Maximise,
    Close,
}

/// A title button at `x`, `y`, 16 x 16, held down or not (held, as Windows 95's: its edge sunk and its mark a pixel
/// down and right).
pub fn title_button(sheet: &mut Sheet, x: i32, y: i32, which: TitleButton, held: bool) {
    sheet.fill(x, y, BUTTON, BUTTON, FACE);
    sheet.edge(x, y, BUTTON, BUTTON, if held { Edge::HeldDown } else { Edge::RaisedButton });
    let (rows, at): (&[&str], (i32, i32)) = match which {
        TitleButton::Minimise => (&marks::MINIMISE, marks::MINIMISE_AT),
        TitleButton::Maximise => (&marks::MAXIMISE, marks::MAXIMISE_AT),
        TitleButton::Close => (&marks::CLOSE, marks::CLOSE_AT),
    };
    let pushed = i32::from(held);
    sheet.mark(x + at.0 + pushed, y + at.1 + pushed, rows);
}

/// The menu's words from `x` along the row at `y` (the row's top), each first letter underlined: where the next would
/// start.
pub fn menu_words(sheet: &mut Sheet, x: i32, y: i32, words: &[&str]) -> i32 {
    let ty = y + (MENU - LINE) / 2;
    let mut at = x;
    for word in words {
        let width = letters::draw(sheet, at, ty, word, BLACK);
        sheet.fill(at, ty + BASE + 1, letters::first_width(word), 1, BLACK);
        at += width + WORDS_APART;
    }
    at
}

/// A field `w` wide at `x`, `y`: sunken, white, its words 5 in.
pub fn field(sheet: &mut Sheet, x: i32, y: i32, w: i32, text: &str) {
    sheet.edge(x, y, w, FIELD, Edge::Sunken);
    sheet.fill(x + 2, y + 2, w - 4, FIELD - 4, WHITE);
    letters::draw(sheet, x + 5, y + (FIELD - LINE) / 2, text, BLACK);
}

/// A drop-down `w` wide at `x`, `y`: a field with its button at the right.
pub fn drop_down(sheet: &mut Sheet, x: i32, y: i32, w: i32, text: &str) {
    field(sheet, x, y, w, text);
    let (bx, by, bh) = (x + w - 18, y + 2, FIELD - 4);
    sheet.fill(bx, by, SCROLL, bh, FACE);
    sheet.edge(bx, by, SCROLL, bh, Edge::RaisedWindow);
    sheet.mark(bx + marks::DROP_DOWN_AT.0, by + marks::DROP_DOWN_AT.1, &marks::DROP_DOWN);
}

/// A drop-down's open list at `x`, `y`, `w` wide: Windows 95's, a black line round it, white, its rows 15 apart and
/// their words 4 in (his ruling on lists); `lit`, the row under the pointer, navy with white words. Its height.
pub fn drop_list(sheet: &mut Sheet, x: i32, y: i32, w: i32, rows: &[&str], lit: Option<usize>) -> i32 {
    let h = rows.len() as i32 * ROW + 2;
    sheet.fill(x, y, w, h, WHITE);
    sheet.ring(x, y, w, h, BLACK, BLACK);
    for (k, row) in rows.iter().enumerate() {
        let ry = y + 1 + k as i32 * ROW;
        let ink = if lit == Some(k) {
            sheet.fill(x + 1, ry, w - 2, ROW, NAVY);
            WHITE
        } else {
            BLACK
        };
        letters::draw(sheet, x + 4, ry, row, ink);
    }
    h
}

/// A line cut in across `w` at `x`, `y`: grey, then white under it.
pub fn line_cut_in(sheet: &mut Sheet, x: i32, y: i32, w: i32) {
    sheet.fill(x, y, w, 1, SHADOW);
    sheet.fill(x, y + 1, w, 1, WHITE);
}

/// One tab at `x`, `y`, `w` wide (the sheet's drawer's recipe), its words laid at `words`; chosen, its foot is open into
/// its page.
fn tab(sheet: &mut Sheet, x: i32, y: i32, w: i32, name: &str, words: (i32, i32), chosen: bool) {
    sheet.fill(x + 2, y, w - 4, 2, FACE);
    sheet.fill(x, y + 2, w, TAB - 2, FACE);
    sheet.fill(x, y + 2, 1, TAB - 2, WHITE);
    sheet.px(x + 1, y + 1, WHITE);
    sheet.fill(x + 2, y, w - 4, 1, WHITE);
    sheet.fill(x + 1, y + 2, 1, TAB - 2, LIGHT);
    sheet.fill(x + 2, y + 1, w - 4, 1, LIGHT);
    sheet.fill(x + w - 1, y + 2, 1, TAB - 2, BLACK);
    sheet.px(x + w - 2, y + 1, BLACK);
    sheet.fill(x + w - 2, y + 2, 1, TAB - 2, SHADOW);
    if chosen {
        sheet.fill(x + 2, y + TAB, w - 4, 2, FACE);
    }
    letters::draw(sheet, words.0, words.1, name, BLACK);
}

/// How wide a name's ink is.
fn ink_width(text: &str) -> i32 {
    let (left, right) = letters::ink(text);
    right - left + 1
}

/// A tab's place and width in a row from `x`, side by side: each its name's ink, 2 clear of its edges each side (his
/// word: "as dense as possible", "nothing touches within 2px"; the sheet's tabs gave their names 7 a side).
pub fn tab_places(x: i32, names: &[&str]) -> Vec<(i32, i32)> {
    let mut at = x;
    names
        .iter()
        .map(|name| {
            let place = (at, ink_width(name) + 2 * (TAB_EDGE + CLEARANCE));
            at += place.1;
            place
        })
        .collect()
}

/// A row of tabs from `x`, `y`, the page's top edge under them at `y + TAB` (drawn first by the caller): the chosen one
/// drawn last, over its neighbours' edges by 2 each side where it has a neighbour, and nowhere past the row's ends (his
/// ruling, 2026-11-20: "just be up front as re the borders between neighbors"), its foot open into the page. Each name's
/// ink starts 2 clear of its tab's left edge, chosen or not.
pub fn tabs(sheet: &mut Sheet, x: i32, y: i32, names: &[&str], chosen: usize) {
    let places = tab_places(x, names);
    let words = |(at, _): (i32, i32), name: &str| (at + TAB_EDGE + CLEARANCE - letters::ink(name).0, y + (TAB - LINE) / 2 + 1);
    for (k, (name, place)) in names.iter().zip(&places).enumerate() {
        if k != chosen {
            tab(sheet, place.0, y, place.1, name, words(*place, name), false);
        }
    }
    if let (Some(name), Some(place)) = (names.get(chosen), places.get(chosen)) {
        let left = if chosen > 0 { 2 } else { 0 };
        let right = if chosen + 1 < names.len() { 2 } else { 0 };
        tab(sheet, place.0 - left, y, place.1 + left + right, name, words(*place, name), true);
    }
}

/// A tab's page: the button's raised edge round the face.
pub fn tab_page(sheet: &mut Sheet, r: Rect) {
    sheet.fill(r.x, r.y, r.w, r.h, FACE);
    sheet.edge(r.x, r.y, r.w, r.h, Edge::RaisedButton);
}

/// A scrollbar `h` tall at `x`, `y` with nothing to scroll: greyed, as he drew the Software map's ("grey out, don't
/// hide"): its two arrows greyed, its track the checker, no thumb.
pub fn scrollbar_greyed(sheet: &mut Sheet, x: i32, y: i32, h: i32) {
    for j in SCROLL..h - SCROLL {
        for i in 0..SCROLL {
            let (px, py) = (x + i, y + j);
            sheet.px(px, py, if (px + py) % 2 == 0 { WHITE } else { FACE });
        }
    }
    for (by, rows) in [(y, &marks::UP_GREYED[..]), (y + h - SCROLL, &marks::DOWN_GREYED[..])] {
        sheet.fill(x, by, SCROLL, SCROLL, FACE);
        sheet.edge(x, by, SCROLL, SCROLL, Edge::RaisedWindow);
        sheet.mark(x + marks::ARROW_AT.0, by + marks::ARROW_AT.1, rows);
    }
}

/// A view's frame, thick and sunk (his ruling, 2026-11-20: "make it thick frame, sunk, on both the software map board,
/// the tectonic viewport, and the world gen viewport"): the sunken edge outside, then two rows of face, and `inside`
/// within. Where what's inside starts.
pub fn sunk_frame(sheet: &mut Sheet, r: Rect, inside: Colour) -> Rect {
    sheet.fill(r.x, r.y, r.w, r.h, FACE);
    sheet.edge(r.x, r.y, r.w, r.h, Edge::Sunken);
    let within = Rect::new(r.x + 4, r.y + 4, r.w - 8, r.h - 8);
    sheet.fill(within.x, within.y, within.w, within.h, inside);
    within
}

/// A status bar's pane at `x`, `y`, `w` wide: sunk by one line, its words' pen at `words` in from its left.
pub fn pane(sheet: &mut Sheet, x: i32, y: i32, w: i32, text: &str, words: i32) {
    sheet.fill(x, y, w, PANE, FACE);
    sheet.edge(x, y, w, PANE, Edge::SunkenOneLine);
    letters::draw(sheet, x + words, y + (PANE - LINE) / 2, text, BLACK);
}

/// A small pane for `text` in a row of them (the status bar's right end): its ink 2 clear of the pane's line each side,
/// and, in the last one, 2 clear of the grip's corner too. Its width, and its words' pen from its left.
pub fn small_pane(text: &str, last: bool) -> (i32, i32) {
    (ink_width(text) + 2 * (PANE_LINE + CLEARANCE) + if last { GRIP } else { 0 }, PANE_LINE + CLEARANCE - letters::ink(text).0)
}

/// The grip, `GRIP` square, its top left at `x`, `y`.
pub fn grip(sheet: &mut Sheet, x: i32, y: i32) {
    sheet.mark(x, y, &marks::GRIP);
}

/// Words with a dark edge under and right of them (the temperatures over the world): how wide they were.
pub fn words_over_the_world(sheet: &mut Sheet, x: i32, y: i32, text: &str, colour: Colour) -> i32 {
    letters::draw(sheet, x + 1, y + 1, text, BLACK);
    letters::draw(sheet, x, y, text, colour)
}

/// A sheet made clear.
pub fn clear(sheet: &mut Sheet) {
    let (w, h) = sheet.size();
    sheet.fill(0, 0, w as i32, h as i32, CLEAR);
}

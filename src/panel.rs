//! The debug panel (F3): a floating window over the game, in the game only [David, 2026-11-20: "the debug panel should
//! not appear in the editor, it should just be in the game"], drawn by us in Windows 95's look (`gui/`).
//!
//! It starts from Counsel's example 4 (`Art/References/editor reference/mock-up example 4 - the F3 debug panel,
//! floating.png`), his "yes, go with your suggestions for it": its place and size, its title "Debug  [F3]" in Ironquill,
//! its close button, and old's three tabs (their names are old's, the ones he knows). Inside, it's laid out by his rules:
//! 2 pixels between things and nothing loose, the tabs none rising and as dense as they can be, the whole a quarter
//! narrower than the example. Build 228 has the window and its tabs, each page blank; the tabs are filled a build each
//! (Overview, Camera, Render, then the preset bars).

use crate::gui::letters::{self, LINE};
use crate::gui::parts::{self, BUTTON, CAPTION, Rect, SCROLL, TAB, TitleButton};
use crate::gui::sheet::{Sheet, WHITE};

/// Its pages, old's names.
const TABS: [&str; 3] = ["Overview", "Camera", "Render"];
/// Its place on the screen (its own pixels from the top left), example 4's; its size, example 4's height, its width
/// a quarter less [David, 2026-11-20: "it doesnt necessarily need to be quite so wide"; "you can narrow the whole by
/// about a quarter"].
pub const AT: (i32, i32) = (16, 16);
const SIZE: (i32, i32) = (303, 432);
/// Its title.
const TITLE: &str = "Debug  [F3]";

pub struct Panel {
    pub visible: bool,
    tab: usize,
    /// Whether the close button is held down (it shuts the panel when it's let go over it, as Windows' buttons do).
    held: bool,
    sheet: Sheet,
    drawn: Option<(usize, bool)>,
}

impl Panel {
    pub fn new() -> Self {
        Self { visible: true, tab: 0, held: false, sheet: Sheet::new(SIZE.0 as u32, SIZE.1 as u32), drawn: None }
    }

    /// Drawn again if anything it shows has changed.
    pub fn lay_out(&mut self) {
        if self.drawn != Some((self.tab, self.held)) {
            draw(&mut self.sheet, self.tab, self.held);
            self.drawn = Some((self.tab, self.held));
        }
    }

    /// Its sheet, as last laid out.
    pub fn sheet(&self) -> &Sheet {
        &self.sheet
    }

    /// Whether `at` (its own pixels from its top left) is on it.
    pub fn holds(&self, at: (i32, i32)) -> bool {
        Rect::new(0, 0, SIZE.0, SIZE.1).holds(at)
    }

    /// The left button pressed at `at`: the close button held, or a tab chosen.
    pub fn pressed(&mut self, at: (i32, i32)) {
        if close_button().holds(at) {
            self.held = true;
            return;
        }
        let tabs = tabs_row();
        if tabs.holds(at)
            && let Some(k) = parts::tab_places(tabs.x, &TABS).iter().position(|(x, w)| at.0 >= *x && at.0 < x + w)
        {
            self.tab = k;
        }
    }

    /// The left button let go at `at`: the panel shut if the close button was held and the pointer's still on it.
    pub fn released(&mut self, at: (i32, i32)) {
        if self.held && close_button().holds(at) {
            self.visible = false;
        }
        self.held = false;
    }
}

/// The close button, as the editor's: 22 in from the window's right, 6 down.
fn close_button() -> Rect {
    Rect::new(SIZE.0 - 22, 4 + 2, BUTTON, BUTTON)
}

/// The tabs' row: 2 under the title bar.
fn tabs_row() -> Rect {
    Rect::new(4, 4 + CAPTION + 2, SIZE.0 - 8, TAB)
}

fn draw(sheet: &mut Sheet, tab: usize, held: bool) {
    parts::window(sheet, 0, 0, SIZE.0, SIZE.1);
    parts::title_bar(sheet, 4, 4, SIZE.0 - 8);
    letters::draw(sheet, 7, 4 + (CAPTION - LINE) / 2, TITLE, WHITE);
    let close = close_button();
    parts::title_button(sheet, close.x, close.y, TitleButton::Close, held);
    let tabs = tabs_row();
    let page = Rect::new(tabs.x, tabs.y + TAB, tabs.w, SIZE.1 - 4 - (tabs.y + TAB));
    parts::tab_page(sheet, page);
    parts::tabs(sheet, tabs.x, tabs.y, &TABS, tab);
    parts::scrollbar_greyed(sheet, page.x + page.w - 2 - SCROLL, page.y + 2, page.h - 4);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Its close button held and let go over it shuts it; let go elsewhere, it doesn't. A tab pressed is chosen.
    #[test]
    fn its_close_button_and_tabs_answer_the_mouse() {
        let mut panel = Panel::new();
        let close = close_button();
        let on = (close.x + 8, close.y + 8);
        panel.pressed(on);
        panel.released((on.0 - 40, on.1));
        assert!(panel.visible);
        panel.pressed(on);
        panel.released(on);
        assert!(!panel.visible);
        let places = parts::tab_places(tabs_row().x, &TABS);
        panel.pressed((places[2].0 + 3, tabs_row().y + 10));
        assert_eq!(panel.tab, 2);
    }
}

//! The editor's frame, drawn: laid out for the window's size the way his four pictures are at 1,280 x 800
//! (`Art/95gui mockups/current/`), with his rulings where they differ from the control sheet (the shared page,
//! `journal/docs/editor-look.md`, "Where we stand").
//!
//! The window is his pictures' 1,281 x 801 for a sheet of 1,280 x 800: he drew it a pixel wider and taller than the
//! picture, so its black outer line at the right and the bottom falls just past the picture's edge, and everything
//! inside is placed from that. The sheet is drawn the same way at any size.

use super::Page;
use crate::gui::letters::{self, LINE};
use crate::gui::parts::{self, CAPTION, FIELD, MENU, PANE, Rect, SCROLL, TAB, TitleButton};
use crate::gui::sheet::{BLACK, FACE, Sheet, TEAL, WHITE};

/// The menu's words (his pictures).
pub const MENU_WORDS: [&str; 5] = ["File", "Edit", "View", "Window", "Help"];
/// The page drop-down's width: 128 on every page (his answer, 2026-11-20: "128").
pub const DROP_DOWN_WIDTH: i32 = 128;
/// The tabbed panel's width on Tectonic Sim and World Gen at his pictures' size (from 4 to 552): kept between a quarter
/// and a half of the window's inside [David, 2026-11-19: "the panel with tabs will be a fixed 1/2 to 1/4 size on the
/// left"].
const PANEL: i32 = 549;
/// The Software map's card panel (from 4 to 305).
const CARD: i32 = 302;
/// Between things: 2 pixels [his rule, 2026-11-20: "keep at least a 2 pixel border between elements"; "we should always
/// squeeze things as tight as possible while maintaining a consistent logic for gaps and borders"].
const GAP: i32 = 2;

/// Where everything is, for a sheet of `size`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Frame {
    pub size: (i32, i32),
    /// The inside's last column and last row (the window's edge and its two rows of face are outside them).
    pub right: i32,
    pub bottom: i32,
    pub drop_down: Rect,
    /// Where the pages go: under the line, over the status bar.
    pub pages: Rect,
    /// The status bar's panes' top.
    pub status: i32,
}

impl Frame {
    pub fn new((w, h): (i32, i32)) -> Self {
        // The window his way: a pixel past the sheet each way (see above), its frame 4 thick.
        let (right, bottom) = (w + 1 - 5, h + 1 - 5);
        let menu_y = 4 + CAPTION + 1;
        // The drop-down 10 after the menu's last word (his pictures: the words are 12 apart, the drop-down 10 after them).
        let drop_x = 4 + 6 + MENU_WORDS.iter().map(|word| letters::width(word) + 12).sum::<i32>() - 2;
        let line_y = menu_y + MENU;
        let status = bottom - (PANE - 1);
        let pages_top = line_y + 2 + GAP;
        let pages_bottom = status - GAP - 1;
        Self { size: (w, h), right, bottom, drop_down: Rect::new(drop_x, menu_y, DROP_DOWN_WIDTH, FIELD), pages: Rect::new(4, pages_top, right - 4 + 1, pages_bottom - pages_top + 1), status }
    }

    /// The menu's row's top.
    fn menu_y(&self) -> i32 {
        4 + CAPTION + 1
    }

    /// The tabbed panel's width at this size.
    fn panel_width(&self) -> i32 {
        PANEL.min(self.pages.w / 2).max(self.pages.w / 4)
    }

    /// Where a tabbed page's tabs start, and its page under them.
    pub fn tabbed(&self) -> (Rect, Rect) {
        let w = self.panel_width();
        let tabs = Rect::new(self.pages.x, self.pages.y, w, TAB);
        let page = Rect::new(self.pages.x, self.pages.y + TAB, w, self.pages.h - TAB);
        (tabs, page)
    }

    /// A tabbed page's view, right of its panel.
    pub fn view(&self) -> Rect {
        let x = self.pages.x + self.panel_width() + GAP;
        Rect::new(x, self.pages.y, self.right - x + 1, self.pages.h)
    }

    /// The Software map's board, right of the card's panel.
    pub fn board(&self) -> Rect {
        let x = self.pages.x + CARD;
        Rect::new(x, self.pages.y, self.right - x + 1, self.pages.h)
    }

    /// The drop-down's list of pages, opened under it.
    pub fn list(&self) -> Rect {
        Rect::new(self.drop_down.x, self.drop_down.y + FIELD, DROP_DOWN_WIDTH, Page::ALL.len() as i32 * parts::ROW + 2)
    }

    /// The row of the drop-down's list under `at`.
    pub fn list_row(&self, at: (i32, i32)) -> Option<usize> {
        let list = self.list();
        list.holds(at).then(|| ((at.1 - list.y - 1).max(0) / parts::ROW) as usize).filter(|row| *row < Page::ALL.len())
    }

    /// The title buttons' places: minimise, maximise, close.
    pub fn title_buttons(&self) -> [(TitleButton, i32); 3] {
        let close = self.right - 17;
        [(TitleButton::Minimise, close - 34), (TitleButton::Maximise, close - 17), (TitleButton::Close, close)]
    }
}

/// What the editor shows: when any of it changes, it's drawn again.
#[derive(Clone, PartialEq, Debug)]
pub struct Shown {
    pub size: (i32, i32),
    pub page: Page,
    /// The chosen tab on the page, if it has tabs.
    pub tab: usize,
    /// The drop-down's list, if it's open, and its row under the pointer.
    pub list: Option<Option<usize>>,
    /// The temperatures, a pane each at the status bar's right end [David, 2026-11-20: "put them at the bottom bar, to
    /// the right"].
    pub panes: Vec<String>,
}

/// The editor drawn whole for `shown`.
pub fn draw(sheet: &mut Sheet, shown: &Shown) {
    let frame = Frame::new(shown.size);
    let (w, h) = shown.size;
    sheet.resize(w as u32, h as u32);
    parts::window(sheet, 0, 0, w + 1, h + 1);
    title_bar(sheet, &frame);
    let menu_y = frame.menu_y();
    parts::menu_words(sheet, 4 + 6, menu_y, &MENU_WORDS);
    parts::line_cut_in(sheet, 4, menu_y + MENU, frame.right - 4 + 1);
    parts::drop_down(sheet, frame.drop_down.x, frame.drop_down.y, frame.drop_down.w, shown.page.name());
    page(sheet, &frame, shown);
    status_bar(sheet, &frame, &shown.panes);
    if let Some(lit) = shown.list {
        let list = frame.list();
        let names: Vec<&str> = Page::ALL.iter().map(|page| page.name()).collect();
        parts::drop_list(sheet, list.x, list.y, list.w, &names, Some(lit.unwrap_or(shown.page as usize)));
    }
}

/// The title bar: his lettering, the build's number, the three buttons.
fn title_bar(sheet: &mut Sheet, frame: &Frame) {
    parts::title_bar(sheet, 4, 4, frame.right - 4 + 1);
    if let Some((width, _, rgba)) = title_lettering() {
        sheet.stamp(7, 8, *width, rgba, WHITE);
    }
    let buttons = frame.title_buttons();
    let build = format!("Build {}", crate::session::BUILD_NUMBER);
    letters::draw(sheet, buttons[0].1 - 9 - letters::width(&build), 4 + (CAPTION - LINE) / 2, &build, WHITE);
    for (which, x) in buttons {
        parts::title_button(sheet, x, 4 + 2, which, false);
    }
}

/// His title lettering, "INCUS ARTUS" in Darinia as he set it (`data/gui/title.png`, lifted from his pictures: closer
/// set than the font sets it, so it's his picture, not the font's laying).
fn title_lettering() -> Option<&'static (u32, u32, Vec<u8>)> {
    static TITLE: std::sync::OnceLock<Option<(u32, u32, Vec<u8>)>> = std::sync::OnceLock::new();
    TITLE
        .get_or_init(|| {
            let path = crate::session::project_folder().join("data").join("gui").join("title.png");
            let read = crate::textures::read_png(&path);
            if read.is_none() {
                log::error!("The title's lettering couldn't be read from {}", path.display());
            }
            read
        })
        .as_ref()
}

/// The page: blank, with its panels, tabs and views.
fn page(sheet: &mut Sheet, frame: &Frame, shown: &Shown) {
    let pages = frame.pages;
    sheet.fill(pages.x, pages.y, pages.w, pages.h, FACE);
    match shown.page {
        Page::TectonicSim | Page::WorldGen => {
            let (tabs, page) = frame.tabbed();
            parts::tab_page(sheet, page);
            parts::tabs(sheet, tabs.x, tabs.y, shown.page.tabs(), shown.tab);
            parts::scrollbar_greyed(sheet, page.x + page.w - 2 - SCROLL, page.y + 2, page.h - 4);
            parts::sunk_frame(sheet, frame.view(), BLACK);
        }
        Page::SoftwareMap => {
            let inside = parts::sunk_frame(sheet, frame.board(), TEAL);
            parts::scrollbar_greyed(sheet, inside.x + inside.w - SCROLL, inside.y, inside.h);
        }
        // Blank: its three panes differ from the sheet in ways to put to him when the page is built (plan 3.3).
        Page::Registrar => {}
    }
}

/// The status bar: "Ready" in its long pane (4 in, as his), the temperatures in small panes at its right end, each as
/// dense as it can be with its words centred (his rulings: "just center anything in those displays"; "as dense as
/// possible"), and the grip in the corner.
fn status_bar(sheet: &mut Sheet, frame: &Frame, panes: &[String]) {
    let y = frame.status;
    let mut right = frame.right;
    for (k, text) in panes.iter().enumerate().rev() {
        let (w, words) = parts::small_pane(text, k + 1 == panes.len());
        parts::pane(sheet, right - w + 1, y, w, text, words);
        right -= w + GAP;
    }
    parts::pane(sheet, 4, y, right - 4 + 1, "Ready", 4);
    parts::grip(sheet, frame.right - parts::GRIP, frame.bottom - parts::GRIP);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At his pictures' size, the frame's places are his pictures'.
    #[test]
    fn the_frame_is_where_his_pictures_have_it() {
        if letters::ironquill().is_none() {
            panic!("Ironquill isn't in Art/fonts/Ironquill/: the frame's places follow its words' widths");
        }
        let frame = Frame::new((1280, 800));
        assert_eq!((frame.right, frame.bottom), (1276, 796));
        assert_eq!(frame.drop_down, Rect::new(192, 25, 128, 23));
        assert_eq!(frame.pages, Rect::new(4, 50, 1273, 723));
        assert_eq!(frame.status, 775);
        let (tabs, page) = frame.tabbed();
        assert_eq!((tabs, page), (Rect::new(4, 50, 549, 21), Rect::new(4, 71, 549, 702)));
        assert_eq!(frame.view(), Rect::new(555, 50, 722, 723));
        assert_eq!(frame.board(), Rect::new(306, 50, 971, 723));
        assert_eq!(frame.title_buttons().map(|(_, x)| x), [1225, 1242, 1259]);
    }
}

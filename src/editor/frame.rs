//! The editor's frame, drawn: laid out for the window's size the way his four pictures are at 1,280 x 800
//! (`Art/95gui mockups/current/`), with his rulings where they differ from the control sheet (the shared page,
//! `journal/docs/editor-look.md`, "Where we stand").
//!
//! The window is laid out whole, its black outer line showing at the right and the bottom [David, at his look at build
//! 228: "yes"]. His pictures are that window a pixel bigger each way, cut short at 1,280 x 800, so a window of 1,281 x
//! 801 is his pictures to the pixel. While it fills the screen its border stays all round and its grip is gone [David,
//! 2026-11-20: "1 border all around 2 grip gone"].

use super::Page;
use super::menus::{self, Does, Showing};
use crate::gui::letters::{self, LINE};
use crate::gui::parts::{self, BUTTON, CAPTION, FIELD, GRIP, MENU, MENU_MARGIN, MenuLine, PANE, Rect, SCROLL, TAB, TitleButton};
use crate::gui::sheet::{BLACK, FACE, Sheet, TEAL, WHITE};

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
/// The window's frame: its raised edge and two rows of face. A press on it sizes the window.
const BORDER: i32 = 4;
/// How far along each side of a corner the corner's diagonal sizing reaches.
const CORNER: i32 = 16;
/// His title lettering's width, if its picture can't be read.
const TITLE_WIDTH: i32 = 139;

/// How the frame is laid out, beside its size: View's two ticks, and whether the window fills the screen (above the
/// taskbar or full screen), when it has no grip and can't be sized.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Look {
    pub status_bar: bool,
    pub panel: bool,
    pub filling: bool,
}

impl Default for Look {
    fn default() -> Self {
        Self { status_bar: true, panel: true, filling: false }
    }
}

/// Which way the window is sized from where the pointer is on its edge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// The menu open: its word on the row, the item lit, and the Editor Scale's menu if it's open (its item lit).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Open {
    pub word: usize,
    pub lit: Option<usize>,
    pub scales: Option<Option<usize>>,
}

/// Where everything is, for a sheet of `size`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Frame {
    pub size: (i32, i32),
    /// The inside's last column and last row (the window's frame is outside them).
    pub right: i32,
    pub bottom: i32,
    pub drop_down: Rect,
    /// Where the pages go: under the line, over the status bar (or to the frame, with it hidden).
    pub pages: Rect,
    /// The status bar's panes' top.
    pub status: i32,
    pub look: Look,
}

impl Frame {
    pub fn new((w, h): (i32, i32), look: Look) -> Self {
        let (right, bottom) = (w - 1 - BORDER, h - 1 - BORDER);
        let menu_y = Self::menu_y();
        // The drop-down 10 after the menu's last word (his pictures: the words are 12 apart, the drop-down 10 after them).
        let drop_x = BORDER + 6 + menus::WORDS.iter().map(|word| letters::width(word) + 12).sum::<i32>() - 2;
        let line_y = menu_y + MENU;
        let status = bottom - (PANE - 1);
        let pages_top = line_y + 2 + GAP;
        let pages_bottom = if look.status_bar { status - GAP - 1 } else { bottom };
        Self { size: (w, h), right, bottom, drop_down: Rect::new(drop_x, menu_y, DROP_DOWN_WIDTH, FIELD), pages: Rect::new(BORDER, pages_top, right - BORDER + 1, pages_bottom - pages_top + 1), status, look }
    }

    /// The smallest the frame can be laid out: the title bar's lettering, the build's number and the buttons; the menu
    /// row to the drop-down's end; the widest row of tabs in a panel at most half the inside; a tab's page with room for
    /// its scrollbar's two arrows, and the status bar.
    pub fn smallest() -> (i32, i32) {
        let lettering = title_lettering().map_or(TITLE_WIDTH, |(width, _, _)| *width as i32);
        let build = letters::width(&format!("Build {}", crate::session::BUILD_NUMBER));
        let title = 7 + lettering + 12 + build + 9 + 3 * 17 + BORDER + 1;
        let menu_row = Self::new((0, 0), Look::default()).drop_down.x + DROP_DOWN_WIDTH + BORDER + 1;
        let tabs = Page::ALL.iter().filter_map(|page| parts::tab_places(0, page.tabs()).last().map(|(x, w)| x + w)).max().unwrap_or(0);
        let width = title.max(menu_row).max(2 * tabs + 2 * BORDER + 1);
        let height = Self::menu_y() + MENU + 2 + GAP + TAB + 2 * SCROLL + 4 + GAP + PANE + BORDER + 1;
        (width, height)
    }

    /// The menu's row's top.
    fn menu_y() -> i32 {
        BORDER + CAPTION + 1
    }

    /// The tabbed panel's width at this size (none while it's hidden).
    fn panel_width(&self) -> i32 {
        if self.look.panel { PANEL.min(self.pages.w / 2).max(self.pages.w / 4) } else { 0 }
    }

    /// Where a tabbed page's tabs start, and its page under them.
    pub fn tabbed(&self) -> (Rect, Rect) {
        let w = self.panel_width();
        let tabs = Rect::new(self.pages.x, self.pages.y, w, TAB);
        let page = Rect::new(self.pages.x, self.pages.y + TAB, w, self.pages.h - TAB);
        (tabs, page)
    }

    /// A tabbed page's view, right of its panel (the whole page, with the panel hidden).
    pub fn view(&self) -> Rect {
        let x = if self.look.panel { self.pages.x + self.panel_width() + GAP } else { self.pages.x };
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

    /// The title bar, where a press moves the window.
    pub fn title_bar(&self) -> Rect {
        Rect::new(BORDER, BORDER, self.right - BORDER + 1, CAPTION)
    }

    /// The title buttons and their places: minimise; maximise, or restore while the window fills the screen [David,
    /// 2026-11-20: "follow windows"]; close.
    pub fn title_buttons(&self) -> [(TitleButton, i32); 3] {
        let close = self.right - 17;
        let middle = if self.look.filling { TitleButton::Restore } else { TitleButton::Maximise };
        [(TitleButton::Minimise, close - 34), (middle, close - 17), (TitleButton::Close, close)]
    }

    /// The title button under `at`.
    pub fn title_button_at(&self, at: (i32, i32)) -> Option<TitleButton> {
        self.title_buttons().into_iter().find(|(_, x)| Rect::new(*x, BORDER + 2, BUTTON, BUTTON).holds(at)).map(|(button, _)| button)
    }

    /// The menu row's words' places.
    pub fn menu_words(&self) -> Vec<Rect> {
        parts::menu_word_places(BORDER + 6, Self::menu_y(), &menus::WORDS)
    }

    /// Where the `word`th word's menu opens with its `lines`: under its word, its left at the word's place's (kept on
    /// the sheet).
    pub fn menu_at(&self, word: usize, lines: &[MenuLine]) -> Rect {
        let place = self.menu_words()[word];
        let (w, h) = parts::menu_size(lines);
        Rect::new(place.x.min(self.size.0 - w).max(0), place.y + place.h, w, h)
    }

    /// Where a submenu with its `lines` opens beside the `item`th line of the menu at `menu`: its left over the menu's
    /// last 4 columns and its first item level with the one that opened it, as Windows places them (on the left if
    /// there's no room on the right).
    pub fn submenu_at(&self, menu: Rect, menu_lines: &[MenuLine], item: usize, lines: &[MenuLine]) -> Rect {
        let (w, h) = parts::menu_size(lines);
        let top = parts::menu_rows(menu_lines).get(item).map_or(MENU_MARGIN, |(top, _)| *top);
        let x = if menu.x + menu.w - 4 + w <= self.size.0 { menu.x + menu.w - 4 } else { (menu.x - w + 4).max(0) };
        Rect::new(x, (menu.y + top - MENU_MARGIN).min(self.size.1 - h).max(0), w, h)
    }

    /// The item under `at` in the menu at `place` with its `lines`: none over its edge or margin, or over a line.
    pub fn item_under(place: Rect, lines: &[MenuLine], at: (i32, i32)) -> Option<usize> {
        let (x, y) = (at.0 - place.x, at.1 - place.y);
        if !place.holds(at) || x < MENU_MARGIN || x >= place.w - MENU_MARGIN {
            return None;
        }
        parts::menu_rows(lines).iter().position(|(top, h)| y >= *top && y < top + h).filter(|k| matches!(lines[*k], MenuLine::Item { .. }))
    }

    /// The grip's square, while there is one.
    fn grip(&self) -> Option<Rect> {
        (self.look.status_bar && !self.look.filling).then(|| Rect::new(self.right - GRIP, self.bottom - GRIP, GRIP, GRIP))
    }

    /// The way the window is sized from `at`: its frame's 4 pixels, each corner's diagonal 16 along either side of it,
    /// and the grip. None while the window fills the screen, as Windows 95 has it.
    pub fn side_at(&self, at: (i32, i32)) -> Option<Side> {
        let (w, h) = self.size;
        let (x, y) = at;
        if self.look.filling || x < 0 || y < 0 || x >= w || y >= h {
            return None;
        }
        if self.grip().is_some_and(|grip| grip.holds(at)) {
            return Some(Side::SouthEast);
        }
        let (west, east, north, south) = (x < BORDER, x >= w - BORDER, y < BORDER, y >= h - BORDER);
        let (by_west, by_east, by_north, by_south) = (x < CORNER, x >= w - CORNER, y < CORNER, y >= h - CORNER);
        Some(if (north && by_west) || (west && by_north) {
            Side::NorthWest
        } else if (north && by_east) || (east && by_north) {
            Side::NorthEast
        } else if (south && by_west) || (west && by_south) {
            Side::SouthWest
        } else if (south && by_east) || (east && by_south) {
            Side::SouthEast
        } else if north {
            Side::North
        } else if south {
            Side::South
        } else if west {
            Side::West
        } else if east {
            Side::East
        } else {
            return None;
        })
    }

    /// The menu open, and its lines (with where its Editor Scale item is, if it has one).
    pub fn opened(&self, open: Open, showing: &Showing) -> (Rect, Vec<(MenuLine, Does)>, Option<usize>) {
        let menu = menus::menu(open.word, showing);
        let place = self.menu_at(open.word, &menus::lines(&menu));
        let opener = menu.iter().position(|(_, does)| *does == Does::OpensScales);
        (place, menu, opener)
    }

    /// The Editor Scale's menu, where it opens beside its item in the menu at `place`.
    pub fn scales_at(&self, place: Rect, menu: &[(MenuLine, Does)], opener: usize, showing: &Showing) -> (Rect, Vec<(MenuLine, Does)>) {
        let scales = menus::scales(showing);
        (self.submenu_at(place, &menus::lines(menu), opener, &menus::lines(&scales)), scales)
    }
}

/// What the editor shows: when any of it changes, it's drawn again.
#[derive(Clone, PartialEq, Debug)]
pub struct Shown {
    pub size: (i32, i32),
    pub look: Look,
    pub page: Page,
    /// The chosen tab on the page, if it has tabs.
    pub tab: usize,
    /// The drop-down's list, if it's open, and its row under the pointer.
    pub list: Option<Option<usize>>,
    /// The temperatures, a pane each at the status bar's right end [David, 2026-11-20: "put them at the bottom bar, to
    /// the right"].
    pub panes: Vec<String>,
    /// The title button held down with the pointer on it.
    pub held: Option<TitleButton>,
    /// The menu open.
    pub menu: Option<Open>,
    /// What the menus' ticks and greying show.
    pub showing: Showing,
}

/// The editor drawn whole for `shown`.
pub fn draw(sheet: &mut Sheet, shown: &Shown) {
    let frame = Frame::new(shown.size, shown.look);
    let (w, h) = shown.size;
    sheet.resize(w as u32, h as u32);
    parts::window(sheet, 0, 0, w, h);
    title_bar(sheet, &frame, shown.held);
    let menu_y = Frame::menu_y();
    parts::menu_words(sheet, BORDER + 6, menu_y, &menus::WORDS);
    parts::line_cut_in(sheet, BORDER, menu_y + MENU, frame.right - BORDER + 1);
    parts::drop_down(sheet, frame.drop_down.x, frame.drop_down.y, frame.drop_down.w, shown.page.name());
    page(sheet, &frame, shown);
    if shown.look.status_bar {
        status_bar(sheet, &frame, &shown.panes);
    }
    if let Some(lit) = shown.list {
        let list = frame.list();
        let names: Vec<&str> = Page::ALL.iter().map(|page| page.name()).collect();
        parts::drop_list(sheet, list.x, list.y, list.w, &names, Some(lit.unwrap_or(shown.page as usize)));
    }
    if let Some(open) = shown.menu {
        parts::menu_word_pressed(sheet, frame.menu_words()[open.word]);
        let (place, menu, opener) = frame.opened(open, &shown.showing);
        parts::menu(sheet, place.x, place.y, &menus::lines(&menu), open.lit);
        if let (Some(lit), Some(opener)) = (open.scales, opener) {
            let (at, scales) = frame.scales_at(place, &menu, opener, &shown.showing);
            parts::menu(sheet, at.x, at.y, &menus::lines(&scales), lit);
        }
    }
}

/// The title bar: his lettering, the build's number, the three buttons (one held down with the pointer on it).
fn title_bar(sheet: &mut Sheet, frame: &Frame, held: Option<TitleButton>) {
    parts::title_bar(sheet, BORDER, BORDER, frame.right - BORDER + 1);
    if let Some((width, _, rgba)) = title_lettering() {
        sheet.stamp(7, 8, *width, rgba, WHITE);
    }
    let buttons = frame.title_buttons();
    let build = format!("Build {}", crate::session::BUILD_NUMBER);
    letters::draw(sheet, buttons[0].1 - 9 - letters::width(&build), BORDER + (CAPTION - LINE) / 2, &build, WHITE);
    for (which, x) in buttons {
        parts::title_button(sheet, x, BORDER + 2, which, held == Some(which));
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
            if shown.look.panel {
                let (tabs, page) = frame.tabbed();
                parts::tab_page(sheet, page);
                parts::tabs(sheet, tabs.x, tabs.y, shown.page.tabs(), shown.tab);
                parts::scrollbar_greyed(sheet, page.x + page.w - 2 - SCROLL, page.y + 2, page.h - 4);
            }
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
/// possible"), and the grip in the corner. While the window fills the screen there's no grip, and the small panes
/// move into its room, "Ready" growing by as much, so nothing is left empty (his "grip gone"; the room's use Builder's
/// reading).
fn status_bar(sheet: &mut Sheet, frame: &Frame, panes: &[String]) {
    let y = frame.status;
    let grip = frame.grip();
    let mut right = frame.right;
    for (k, text) in panes.iter().enumerate().rev() {
        let (w, words) = parts::small_pane(text, grip.is_some() && k + 1 == panes.len());
        parts::pane(sheet, right - w + 1, y, w, text, words);
        right -= w + GAP;
    }
    parts::pane(sheet, BORDER, y, right - BORDER + 1, "Ready", 4);
    if let Some(grip) = grip {
        parts::grip(sheet, grip.x, grip.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At his pictures' size (a window of 1,281 x 801, cut short), the frame's places are his pictures'; at 1,280 x 800
    /// everything at the right and the bottom is a pixel in.
    #[test]
    fn the_frame_is_where_his_pictures_have_it() {
        if letters::ironquill().is_none() {
            panic!("Ironquill isn't in Art/fonts/Ironquill/: the frame's places follow its words' widths");
        }
        let frame = Frame::new((1281, 801), Look::default());
        assert_eq!((frame.right, frame.bottom), (1276, 796));
        assert_eq!(frame.drop_down, Rect::new(192, 25, 128, 23));
        assert_eq!(frame.pages, Rect::new(4, 50, 1273, 723));
        assert_eq!(frame.status, 775);
        let (tabs, page) = frame.tabbed();
        assert_eq!((tabs, page), (Rect::new(4, 50, 549, 21), Rect::new(4, 71, 549, 702)));
        assert_eq!(frame.view(), Rect::new(555, 50, 722, 723));
        assert_eq!(frame.board(), Rect::new(306, 50, 971, 723));
        assert_eq!(frame.title_buttons().map(|(_, x)| x), [1225, 1242, 1259]);
        let whole = Frame::new((1280, 800), Look::default());
        assert_eq!((whole.right, whole.bottom, whole.status), (1275, 795, 774));
        assert_eq!(whole.title_buttons().map(|(_, x)| x), [1224, 1241, 1258]);
    }

    /// The edges size the window, the corners diagonally, the grip from the bottom right; nothing does while it fills
    /// the screen.
    #[test]
    fn the_edges_size_the_window() {
        let frame = Frame::new((1000, 600), Look::default());
        assert_eq!(frame.side_at((0, 300)), Some(Side::West));
        assert_eq!(frame.side_at((999, 300)), Some(Side::East));
        assert_eq!(frame.side_at((500, 3)), Some(Side::North));
        assert_eq!(frame.side_at((500, 596)), Some(Side::South));
        assert_eq!(frame.side_at((2, 10)), Some(Side::NorthWest));
        assert_eq!(frame.side_at((990, 1)), Some(Side::NorthEast));
        assert_eq!(frame.side_at((1, 590)), Some(Side::SouthWest));
        assert_eq!(frame.side_at((frame.right - 3, frame.bottom - 3)), Some(Side::SouthEast));
        assert_eq!(frame.side_at((500, 300)), None);
        assert_eq!(frame.side_at((4, 300)), None);
        let filling = Frame::new((1000, 600), Look { filling: true, ..Look::default() });
        assert_eq!(filling.side_at((0, 300)), None);
        assert_eq!(filling.side_at((filling.right - 3, filling.bottom - 3)), None);
    }
}

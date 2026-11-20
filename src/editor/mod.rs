//! The editor (F2): a window of its own in Windows 95's look, drawn by us (`frame.rs`, from `gui/`), its pages reached by
//! the drop-down in its top bar [David, 2026-11-12: "the new editor should have a drop-down menu at the top that brings
//! you to the desired page (software map, tectonics, world gen, registrar, etc)"].
//!
//! Build 228 began it as he asked [2026-11-20: "we'll start with blank panels in the window, and add buttons and
//! features one at a time"]: the frame, and every page blank with its panels, tabs and views. Build 229 makes the frame
//! work, and the menus (`menus.rs`): the window is ours alone in every way it's shown, with no Windows frame [2026-11-20:
//! "there should be no wrapper at all, so the entire thing is 95gui"]; its title buttons, its title bar and its edges
//! move it, size it, fill the screen with it and quit, as Windows 95's do. The editor asks the window for these
//! (`Ask`); the runner has the window. Where every setting and control goes, and what each is called, is his, asked
//! before it's placed [2026-11-12: "organizing the UI is super important to me"].

mod frame;
mod menus;

use crate::gui::Sheet;
use crate::gui::parts::TitleButton;
use frame::{Frame, Look, Open, Shown};
pub use frame::Side;
use menus::{Act, Does, Showing};
use std::time::{Duration, Instant};

/// Two presses on the title bar this close in time and place are a double click (Windows' own: half a second, and
/// SM_CXDOUBLECLK's 4 pixels, 2 either way).
const DOUBLE_CLICK: Duration = Duration::from_millis(500);
const DOUBLE_CLICK_REACH: i32 = 2;

/// The editor's pages, in the drop-down's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    SoftwareMap,
    TectonicSim,
    WorldGen,
    Registrar,
}

impl Page {
    pub const ALL: [Page; 4] = [Page::SoftwareMap, Page::TectonicSim, Page::WorldGen, Page::Registrar];

    /// Its name, in his words [2026-11-12: "we should probably call them Tectonic Sim, World Gen, and Registrar"].
    pub fn name(self) -> &'static str {
        match self {
            Page::SoftwareMap => "Software map",
            Page::TectonicSim => "Tectonic Sim",
            Page::WorldGen => "World Gen",
            Page::Registrar => "Registrar",
        }
    }

    /// Its tabs, named as in his pictures (none on the Software map and the Registrar).
    pub fn tabs(self) -> &'static [&'static str] {
        match self {
            Page::TectonicSim => &["Main settings", "Plate settings", "Land plates", "Ocean plates", "Simulation"],
            Page::WorldGen => &["Main", "Landmass", "Topography", "Climate", "Mineralogy", "Interpreter"],
            Page::SoftwareMap | Page::Registrar => &[],
        }
    }
}

/// How the window is shown now, from the runner: filling the screen (above the taskbar, or full screen), full screen,
/// and which of the scales 1x to 5x the screen can hold the editor at.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Window {
    pub filling: bool,
    pub full_screen: bool,
    pub fits: [bool; 5],
}

/// What the editor asks of the window, or of the game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ask {
    /// Back to the game (F2, File > Return to Game).
    ReturnToGame,
    /// The game ends (the X, File > Quit).
    Quit,
    Minimise,
    /// The middle button, or a double click on the title bar: fill the screen, or make the window smaller.
    FillOrRestore,
    /// F11, Window > Full Screen.
    FullScreen,
    /// A press on the title bar: the window moves with the pointer.
    Move,
    /// A press on an edge or the grip: the window is sized from that side.
    Size(Side),
    /// The editor's scale was chosen (Window > Editor Scale).
    Scale,
}

pub struct Editor {
    pub open: bool,
    page: Page,
    /// Each page's chosen tab, by the page's place in the drop-down.
    tabs: [usize; 4],
    /// The drop-down's list, if it's open, and its row under the pointer.
    list: Option<Option<usize>>,
    /// The menu open, if one is.
    menu: Option<Open>,
    /// View's two ticks: the status bar and the parameters panel shown.
    status_bar: bool,
    panel: bool,
    /// The editor's scale chosen (Window > Editor Scale): none for Auto.
    pub scale: Option<u32>,
    /// A title button held down, and whether the pointer is still on it (Windows' buttons act only when let go on them).
    held: Option<(TitleButton, bool)>,
    /// The last press on the title bar, when and where, for a double click.
    title_pressed: Option<(Instant, (i32, i32))>,
    window: Window,
    /// The sheet, and what it was last drawn for: it's drawn again only when that changes.
    sheet: Sheet,
    drawn: Option<Shown>,
    /// Where it's laid out now (the window's size in the sheet's pixels).
    frame: Frame,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            open: false,
            page: Page::SoftwareMap,
            tabs: [0; 4],
            list: None,
            menu: None,
            status_bar: true,
            panel: true,
            scale: None,
            held: None,
            title_pressed: None,
            window: Window::default(),
            sheet: Sheet::new(1, 1),
            drawn: None,
            frame: Frame::new((1280, 800), Look::default()),
        }
    }

    /// The smallest the editor can be laid out, in its sheet's pixels.
    pub fn smallest() -> (u32, u32) {
        let (w, h) = Frame::smallest();
        (w as u32, h as u32)
    }

    fn look(&self) -> Look {
        Look { status_bar: self.status_bar, panel: self.panel, filling: self.window.filling }
    }

    fn showing(&self) -> Showing {
        Showing { status_bar: self.status_bar, has_panel: !self.page.tabs().is_empty(), panel: self.panel, scale: self.scale, fits: self.window.fits, full_screen: self.window.full_screen }
    }

    /// The editor laid out for `size` (the window in the sheet's pixels), the temperatures' panes `panes`, the window
    /// shown as `window`: drawn again if anything it shows has changed.
    pub fn lay_out(&mut self, size: (u32, u32), panes: Vec<String>, window: Window) {
        self.window = window;
        let size = (size.0 as i32, size.1 as i32);
        self.frame = Frame::new(size, self.look());
        let held = self.held.filter(|(_, on)| *on).map(|(button, _)| button);
        let shown = Shown { size, look: self.look(), page: self.page, tab: self.tabs[self.page as usize], list: self.list, panes, held, menu: self.menu, showing: self.showing() };
        if self.drawn.as_ref() != Some(&shown) {
            frame::draw(&mut self.sheet, &shown);
            self.drawn = Some(shown);
        }
    }

    /// Its sheet, as last laid out.
    pub fn sheet(&self) -> &Sheet {
        &self.sheet
    }

    /// Opened, or shut (F2): whatever was open in it (a list, a menu, a button held) is shut with it.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.let_go();
    }

    /// Everything open in it shut, nothing held: Esc, or the window put aside.
    pub fn let_go(&mut self) {
        self.list = None;
        self.menu = None;
        self.held = None;
    }

    /// The pointer moved to `at` (the sheet's pixels): the menus and the drop-down's list follow it, a held title button
    /// pops up while the pointer's off it. The side the window would be sized from there, for the pointer's shape.
    pub fn moved(&mut self, at: (i32, i32)) -> Option<Side> {
        if let Some((button, _)) = self.held {
            self.held = Some((button, self.frame.title_button_at(at) == Some(button)));
        }
        if self.list.is_some() {
            self.list = Some(self.frame.list_row(at));
        }
        if let Some(open) = self.menu {
            self.menu = Some(self.hovered(open, at));
        }
        (self.menu.is_none() && self.list.is_none() && self.held.is_none()).then(|| self.frame.side_at(at)).flatten()
    }

    /// The menu open, followed by the pointer at `at`: onto another word of the row, its menu instead; an item lit under
    /// it, the Editor Scale's menu opening beside its item; off them, nothing lit but the item whose menu is open.
    fn hovered(&self, open: Open, at: (i32, i32)) -> Open {
        if let Some(word) = self.frame.menu_words().iter().position(|place| place.holds(at)) {
            return if word == open.word { open } else { Open { word, lit: None, scales: None } };
        }
        let showing = self.showing();
        let (place, menu, opener) = self.frame.opened(open, &showing);
        if let (Some(_), Some(opener)) = (open.scales, opener) {
            let (at_scales, scales) = self.frame.scales_at(place, &menu, opener, &showing);
            if at_scales.holds(at) {
                return Open { lit: Some(opener), scales: Some(Frame::item_under(at_scales, &menus::lines(&scales), at)), ..open };
            }
        }
        if place.holds(at) {
            let lit = Frame::item_under(place, &menus::lines(&menu), at);
            let scales = (lit.is_some() && lit == opener).then(|| open.scales.flatten());
            return Open { lit, scales, ..open };
        }
        Open { lit: open.scales.and(open.lit), ..open }
    }

    /// The left button pressed at `at` (the sheet's pixels), `now`: what it asks of the window, if anything. With a menu
    /// open, the press is the menus' alone. Otherwise, in order: a title button held; a menu opened; the window sized
    /// from its edge, or moved by its title bar (a double click fills the screen or makes it smaller); the drop-down
    /// opened or a page chosen from it; a tab chosen.
    pub fn pressed(&mut self, at: (i32, i32), now: Instant) -> Option<Ask> {
        if let Some(open) = self.menu {
            return self.pressed_in_menus(open, at);
        }
        if self.list.is_some() {
            if let Some(row) = self.frame.list_row(at) {
                self.page = Page::ALL[row];
            }
            self.list = None;
            return None;
        }
        if let Some(button) = self.frame.title_button_at(at) {
            self.held = Some((button, true));
            return None;
        }
        if let Some(word) = self.frame.menu_words().iter().position(|place| place.holds(at)) {
            self.menu = Some(Open { word, lit: None, scales: None });
            return None;
        }
        if let Some(side) = self.frame.side_at(at) {
            return Some(Ask::Size(side));
        }
        if self.frame.title_bar().holds(at) {
            let double = self.title_pressed.is_some_and(|(when, place)| now.duration_since(when) < DOUBLE_CLICK && (place.0 - at.0).abs() <= DOUBLE_CLICK_REACH && (place.1 - at.1).abs() <= DOUBLE_CLICK_REACH);
            if double {
                self.title_pressed = None;
                return Some(Ask::FillOrRestore);
            }
            self.title_pressed = Some((now, at));
            // A window filling the screen stays put, as in Windows 95.
            return (!self.window.filling).then_some(Ask::Move);
        }
        if self.frame.drop_down.holds(at) {
            self.list = Some(self.frame.list_row(at));
            return None;
        }
        let names = self.page.tabs();
        if !names.is_empty() && self.panel {
            let (tabs, _) = self.frame.tabbed();
            if tabs.holds(at)
                && let Some(k) = crate::gui::parts::tab_places(tabs.x, names).iter().position(|(x, w)| at.0 >= *x && at.0 < x + w)
            {
                self.tabs[self.page as usize] = k;
            }
        }
        None
    }

    /// A press while a menu is open: on another word, its menu; on its own word, the menu shut; on an item that works, it
    /// done and the menus shut; on the Editor Scale item, its menu kept open; on a greyed item or a line, nothing;
    /// anywhere else, the menus shut and nothing more, as Windows 95's.
    fn pressed_in_menus(&mut self, open: Open, at: (i32, i32)) -> Option<Ask> {
        if let Some(word) = self.frame.menu_words().iter().position(|place| place.holds(at)) {
            self.menu = (word != open.word).then_some(Open { word, lit: None, scales: None });
            return None;
        }
        let showing = self.showing();
        let (place, menu, opener) = self.frame.opened(open, &showing);
        if let (Some(_), Some(opener)) = (open.scales, opener) {
            let (at_scales, scales) = self.frame.scales_at(place, &menu, opener, &showing);
            if at_scales.holds(at) {
                return match Frame::item_under(at_scales, &menus::lines(&scales), at).map(|k| scales[k].1) {
                    Some(Does::Act(act)) => {
                        self.menu = None;
                        self.act(act)
                    }
                    _ => None,
                };
            }
        }
        if place.holds(at) {
            return match Frame::item_under(place, &menus::lines(&menu), at).map(|k| (k, menu[k].1)) {
                Some((_, Does::Act(act))) => {
                    self.menu = None;
                    self.act(act)
                }
                Some((k, Does::OpensScales)) => {
                    self.menu = Some(Open { lit: Some(k), scales: Some(open.scales.flatten()), ..open });
                    None
                }
                _ => None,
            };
        }
        self.menu = None;
        None
    }

    /// An item done: View's ticks and the scale are the editor's own; the rest are asked of the window or the game.
    fn act(&mut self, act: Act) -> Option<Ask> {
        match act {
            Act::ReturnToGame => Some(Ask::ReturnToGame),
            Act::Quit => Some(Ask::Quit),
            Act::StatusBar => {
                self.status_bar = !self.status_bar;
                None
            }
            Act::ParametersPanel => {
                self.panel = !self.panel;
                None
            }
            Act::Scale(scale) => {
                self.scale = scale;
                Some(Ask::Scale)
            }
            Act::FullScreen => Some(Ask::FullScreen),
        }
    }

    /// The left button let go at `at`: a title button held acts if it's let go on it, as Windows' buttons do.
    pub fn released(&mut self, at: (i32, i32)) -> Option<Ask> {
        let (button, _) = self.held.take()?;
        (self.frame.title_button_at(at) == Some(button)).then_some(match button {
            TitleButton::Minimise => Ask::Minimise,
            TitleButton::Maximise | TitleButton::Restore => Ask::FillOrRestore,
            TitleButton::Close => Ask::Quit,
        })
    }
}

#[cfg(test)]
mod tests;

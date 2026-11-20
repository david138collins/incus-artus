//! The editor (F2): a window of its own in Windows 95's look, drawn by us (`frame.rs`, from `gui/`), its pages reached by
//! the drop-down in its top bar [David, 2026-11-12: "the new editor should have a drop-down menu at the top that brings
//! you to the desired page (software map, tectonics, world gen, registrar, etc)"].
//!
//! Build 228 begins it as he asked [2026-11-20: "we'll start with blank panels in the window, and add buttons and
//! features one at a time"]: the frame, and every page blank with its panels, tabs and views. What works: the drop-down
//! opens and changes the page, and a tab clicked is chosen. Where every setting and control goes, and what each is
//! called, is his, asked before it's placed [2026-11-12: "organizing the UI is super important to me"].

mod frame;

use crate::gui::Sheet;
use frame::{Frame, Shown};

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

pub struct Editor {
    pub open: bool,
    page: Page,
    /// Each page's chosen tab, by the page's place in the drop-down.
    tabs: [usize; 4],
    /// The drop-down's list, if it's open, and its row under the pointer.
    list: Option<Option<usize>>,
    /// The sheet, and what it was last drawn for: it's drawn again only when that changes.
    sheet: Sheet,
    drawn: Option<Shown>,
    /// Where it's laid out now (the window's size in the sheet's pixels).
    frame: Frame,
}

impl Editor {
    pub fn new() -> Self {
        Self { open: false, page: Page::SoftwareMap, tabs: [0; 4], list: None, sheet: Sheet::new(1, 1), drawn: None, frame: Frame::new((1280, 800)) }
    }

    /// The editor laid out for `size` (the window in the sheet's pixels), the temperatures' panes `panes`: drawn again if
    /// anything it shows has changed.
    pub fn lay_out(&mut self, size: (u32, u32), panes: Vec<String>) {
        let size = (size.0 as i32, size.1 as i32);
        self.frame = Frame::new(size);
        let shown = Shown { size, page: self.page, tab: self.tabs[self.page as usize], list: self.list, panes };
        if self.drawn.as_ref() != Some(&shown) {
            frame::draw(&mut self.sheet, &shown);
            self.drawn = Some(shown);
        }
    }

    /// Its sheet, as last laid out.
    pub fn sheet(&self) -> &Sheet {
        &self.sheet
    }

    /// Opened, or shut (F2): a list left open is shut with it.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.list = None;
    }

    /// The pointer moved to `at` (the sheet's pixels): the drop-down's list follows it.
    pub fn moved(&mut self, at: (i32, i32)) {
        if self.list.is_some() {
            self.list = Some(self.frame.list_row(at));
        }
    }

    /// The left button pressed at `at`: the drop-down opened or shut, a page chosen from its list, a tab chosen.
    pub fn pressed(&mut self, at: (i32, i32)) {
        if self.list.is_some() {
            if let Some(row) = self.frame.list_row(at) {
                self.page = Page::ALL[row];
            }
            self.list = None;
            return;
        }
        if self.frame.drop_down.holds(at) {
            self.list = Some(self.frame.list_row(at));
            return;
        }
        let names = self.page.tabs();
        if !names.is_empty() {
            let (tabs, _) = self.frame.tabbed();
            if tabs.holds(at)
                && let Some(k) = crate::gui::parts::tab_places(tabs.x, names).iter().position(|(x, w)| at.0 >= *x && at.0 < x + w)
            {
                self.tabs[self.page as usize] = k;
            }
        }
    }
}

#[cfg(test)]
mod tests;

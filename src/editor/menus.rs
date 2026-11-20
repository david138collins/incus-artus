//! The editor's menus, as he named them and took Builder's additions [David, 2026-11-20: "return to game and quit
//! under file, greyed out 'settings' under edit, the scale and windowed menus under window, and tooltip mode that turns
//! the cursor into a question mark"; her additions (Undo and Redo, View's two ticks, What's This? and Keys...): "3.
//! yes"; the scales, Auto and 1x to 5x with those the screen can't hold greyed, the panel's greyed till it has
//! controls: "4. yes"]. What isn't built yet is greyed, never hidden.

use crate::gui::parts::{MenuLine, Tick};

/// The menu row's words, each opening a menu (his pictures).
pub const WORDS: [&str; 5] = ["File", "Edit", "View", "Window", "Help"];

/// What an item does, when it works.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    ReturnToGame,
    Quit,
    StatusBar,
    ParametersPanel,
    /// The editor's scale chosen: none for Auto.
    Scale(Option<u32>),
    FullScreen,
}

/// What a menu's line does when it's clicked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Does {
    Nothing,
    Act(Act),
    /// It opens the Editor Scale's menu beside it.
    OpensScales,
}

/// What the menus' ticks and greying show.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Showing {
    pub status_bar: bool,
    /// Whether the page has a parameters panel, and whether it's shown.
    pub has_panel: bool,
    pub panel: bool,
    /// The editor's scale: none for Auto.
    pub scale: Option<u32>,
    /// Which of 1x to 5x the screen can hold the editor at.
    pub fits: [bool; 5],
    pub full_screen: bool,
}

/// An item: greyed, it does nothing.
fn item(words: &'static str, keys: &'static str, works: bool, tick: Tick, does: Does) -> (MenuLine, Does) {
    (MenuLine::Item { words, keys, works, tick, submenu: false }, if works { does } else { Does::Nothing })
}

/// An item that opens a menu beside it.
fn opener(words: &'static str, works: bool, does: Does) -> (MenuLine, Does) {
    (MenuLine::Item { words, keys: "", works, tick: Tick::None, submenu: true }, if works { does } else { Does::Nothing })
}

fn ticked(on: bool) -> Tick {
    if on { Tick::Tick } else { Tick::None }
}

/// The menu the row's `word`th word opens.
pub fn menu(word: usize, showing: &Showing) -> Vec<(MenuLine, Does)> {
    let line = (MenuLine::Line, Does::Nothing);
    match word {
        0 => vec![item("&Return to Game", "F2", true, Tick::None, Does::Act(Act::ReturnToGame)), line, item("&Quit", "Alt+F4", true, Tick::None, Does::Act(Act::Quit))],
        1 => vec![item("&Undo", "Ctrl+Z", false, Tick::None, Does::Nothing), item("&Redo", "Ctrl+Y", false, Tick::None, Does::Nothing), line, item("&Settings...", "", false, Tick::None, Does::Nothing)],
        2 => vec![
            item("&Status Bar", "", true, ticked(showing.status_bar), Does::Act(Act::StatusBar)),
            item("&Parameters Panel", "", showing.has_panel, ticked(showing.has_panel && showing.panel), Does::Act(Act::ParametersPanel)),
        ],
        3 => vec![
            opener("&Editor Scale", true, Does::OpensScales),
            opener("&Panel Scale", false, Does::Nothing),
            line,
            item("&Full Screen", "F11", true, ticked(showing.full_screen), Does::Act(Act::FullScreen)),
        ],
        _ => vec![item("&What's This?", "Shift+F1", false, Tick::None, Does::Nothing), item("&Keys...", "", false, Tick::None, Does::Nothing)],
    }
}

/// The Editor Scale's menu: Auto, a line, 1x to 5x; the chosen one's dot; those the screen can't hold greyed.
pub fn scales(showing: &Showing) -> Vec<(MenuLine, Does)> {
    const NAMES: [&str; 5] = ["&1x", "&2x", "&3x", "&4x", "&5x"];
    let dot = |chosen: bool| if chosen { Tick::Dot } else { Tick::None };
    let mut lines = vec![item("&Auto", "", true, dot(showing.scale.is_none()), Does::Act(Act::Scale(None))), (MenuLine::Line, Does::Nothing)];
    for (k, name) in NAMES.iter().enumerate() {
        let scale = k as u32 + 1;
        lines.push(item(name, "", showing.fits[k], dot(showing.scale == Some(scale)), Does::Act(Act::Scale(Some(scale)))));
    }
    lines
}

/// The lines alone, to draw.
pub fn lines(menu: &[(MenuLine, Does)]) -> Vec<MenuLine> {
    menu.iter().map(|(line, _)| line.clone()).collect()
}

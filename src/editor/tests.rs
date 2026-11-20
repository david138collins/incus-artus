//! The editor against his four pictures (`Art/95gui mockups/current/`), pixel for pixel, and its rulings by their
//! recipes; its menus by Windows 95's recipe; its clicks; and its pages written to `screenshots/229/` for a look off the
//! screen. His pictures and the font are on this machine, never committed: these tests read them there.

use super::frame::{self, Frame, Look, Open, Shown};
use super::menus::{self, Showing};
use super::*;
use crate::gui::parts::{self, MENU_ITEM, MENU_LINE, MENU_MARGIN, Rect};
use crate::gui::sheet::{BLACK, Colour, FACE, LIGHT, NAVY, SHADOW, WHITE};

fn top() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// His picture, `name` in `Art/95gui mockups/current/`.
fn his(name: &str) -> (u32, u32, Vec<u8>) {
    let path = top().join("Art").join("95gui mockups").join("current").join(name);
    crate::textures::read_png(&path).unwrap_or_else(|| panic!("his picture isn't there: {}", path.display()))
}

/// What the menus show, everything on and every scale held.
fn showing() -> Showing {
    Showing { status_bar: true, has_panel: true, panel: true, scale: None, fits: [true, true, true, false, false], full_screen: false }
}

fn shown(size: (i32, i32), look: Look, page: Page, tab: usize) -> Shown {
    Shown { size, look, page, tab, list: None, panes: Vec::new(), held: None, menu: None, showing: showing() }
}

fn draw(shown: &Shown) -> Sheet {
    assert!(crate::gui::letters::ironquill().is_some(), "Ironquill isn't in Art/fonts/Ironquill/");
    let mut sheet = Sheet::new(1, 1);
    frame::draw(&mut sheet, shown);
    sheet
}

/// A page as a smaller window the size his pictures cut short: 1,281 x 801.
fn drawn(page: Page, tab: usize) -> Sheet {
    draw(&shown((1281, 801), Look::default(), page, tab))
}

fn at(picture: &(u32, u32, Vec<u8>), x: i32, y: i32) -> Colour {
    let k = ((y as u32 * picture.0 + x as u32) * 4) as usize;
    [picture.2[k], picture.2[k + 1], picture.2[k + 2], 255]
}

/// Where the frame may differ from his pictures, and why: everywhere else it must match to the pixel.
fn differences_allowed(page: Page) -> Vec<(Rect, &'static str)> {
    let mut allowed = vec![
        (Rect::new(1227, 8, 12, 12), "the minimise bar, centred by his word (\"the icons evenly spaced inside the boxes\")"),
        (Rect::new(1160, 5, 64, 16), "the build's number"),
        (Rect::new(4, 48, 1273, 725), "the pages blank, their tabs and frames by his rulings (none rising and as dense as can be, the row up 2, the page's foot level with the views', the thick sunk frame)"),
        (Rect::new(600, 775, 662, 22), "the status bar's small panes, which come with their pages (the temperatures' only here)"),
        (Rect::new(1277, 48, 1, 726), "the views' frames pulled in to 1,276 by his answer (\"pull it in, replace them\")"),
    ];
    match page {
        Page::TectonicSim => allowed.push((Rect::new(320, 25, 957, 23), "the Tectonic Sim's top bar, which comes with its page")),
        Page::WorldGen => {
            allowed.push((Rect::new(5, 776, 300, 20), "the message: \"Ready\" till the page has work to report"));
            allowed.push((Rect::new(320, 25, 212, 23), "the World Gen's top bar (Save, Load, its presets), which comes with its page"));
        }
        Page::Registrar => allowed.push((Rect::new(192, 25, 160, 23), "the drop-down, 128 wide by his answer (his Registrar's is 150)")),
        Page::SoftwareMap => {
            allowed.push((Rect::new(318, 25, 2, 23), "the drop-down's right edge, lost in his Software map's (his answer: 128 everywhere)"));
            allowed.push((Rect::new(5, 776, 400, 20), "the message: the chosen box's, which comes with the map (build 230)"));
        }
    }
    allowed
}

/// His four pictures against the frame drawn as the window they are, 1,281 x 801 cut short: every pixel that differs
/// is in a place named with the ruling or the build that makes it differ (the same places and counts as at build 228).
#[test]
fn his_pictures_and_the_frame_differ_only_where_his_rulings_say() {
    let mut failed = Vec::new();
    for (name, page, tab) in [("tectonics.png", Page::TectonicSim, 3), ("world gen.png", Page::WorldGen, 3), ("software map.png", Page::SoftwareMap, 0), ("registrar.png", Page::Registrar, 0)] {
        let picture = his(name);
        assert_eq!((picture.0, picture.1), (1280, 800), "{name}");
        let sheet = drawn(page, tab);
        let allowed = differences_allowed(page);
        let mut counts = vec![0usize; allowed.len()];
        let mut outside = Vec::new();
        for y in 0..800 {
            for x in 0..1280 {
                let (mine, theirs) = (sheet.get(x, y), at(&picture, x, y));
                if mine == theirs {
                    continue;
                }
                match allowed.iter().position(|(rect, _)| rect.holds((x, y))) {
                    Some(k) => counts[k] += 1,
                    None => outside.push((x, y, mine, theirs)),
                }
            }
        }
        println!("{name}:");
        for ((_, why), count) in allowed.iter().zip(&counts) {
            println!("  {count:>7} differ: {why}");
        }
        println!("  {:>7} differ anywhere else", outside.len());
        for (x, y, mine, theirs) in outside.iter().take(24) {
            println!("    at {x}, {y}: mine {:02X}{:02X}{:02X}, his {:02X}{:02X}{:02X}", mine[0], mine[1], mine[2], theirs[0], theirs[1], theirs[2]);
        }
        if !outside.is_empty() {
            failed.push(format!("{name}: {} pixels differ outside the rulings", outside.len()));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("; "));
}

/// At its own size the window is whole [his "yes" to the black line]: its edge's rings all round, and everything at the
/// right and the bottom a pixel in from the window his pictures cut short; the rest where it was.
#[test]
fn the_window_is_whole_at_its_own_size() {
    for page in Page::ALL {
        let whole = draw(&shown((1280, 800), Look::default(), page, 0));
        let cut = drawn(page, 0);
        assert!((0..800).all(|y| whole.get(1279, y) == BLACK && (1..799).contains(&y).then(|| whole.get(1278, y) == SHADOW).unwrap_or(true)), "{page:?}: the right's black line, grey inside it");
        assert!((0..1280).all(|x| whole.get(x, 799) == BLACK && (1..1279).contains(&x).then(|| whole.get(x, 798) == SHADOW).unwrap_or(true)), "{page:?}: the bottom's");
        for y in 700..799 {
            for x in 1000..1279 {
                assert_eq!(whole.get(x, y), cut.get(x + 1, y + 1), "{page:?}: at {x}, {y}, a pixel in from the cut window's");
            }
        }
        for y in 0..600 {
            for x in 0..500 {
                assert_eq!(whole.get(x, y), cut.get(x, y), "{page:?}: at {x}, {y}, where it was");
            }
        }
    }
}

/// Filling the screen against a smaller window: only the middle button's mark (restore, his "follow windows") and the
/// status bar's right end differ; there the grip is gone and the small panes are 14 further right, "Ready" 14 longer
/// (his "grip gone"; the room's use Builder's reading).
#[test]
fn filling_the_screen_differs_only_in_its_middle_button_and_grip() {
    let panes = vec!["CPU area 92 °C".to_string(), "cores 95 °C".to_string(), "GPU 71 °C".to_string()];
    let size = (1281, 801);
    let smaller = draw(&Shown { panes: panes.clone(), ..shown(size, Look::default(), Page::WorldGen, 3) });
    let filling = draw(&Shown { panes: panes.clone(), ..shown(size, Look { filling: true, ..Look::default() }, Page::WorldGen, 3) });
    let frame = Frame::new(size, Look::default());
    let middle = Rect::new(frame.title_buttons()[1].1, 6, 16, 16);
    let first_pane = frame.right + 1 - panes.iter().enumerate().map(|(k, text)| parts::small_pane(text, k + 1 == panes.len()).0 + 2).sum::<i32>() + 2;
    let status_end = Rect::new(first_pane - 3, frame.status, frame.right - (first_pane - 3) + 1, 22);
    for y in 0..801 {
        for x in 0..1281 {
            if smaller.get(x, y) != filling.get(x, y) {
                assert!(middle.holds((x, y)) || status_end.holds((x, y)), "at {x}, {y}: outside the middle button and the status bar's end");
            }
        }
    }
    // The restore mark: two small windows, their tops two rows thick.
    assert_eq!((filling.get(middle.x + 6, middle.y + 3), filling.get(middle.x + 4, middle.y + 6), filling.get(middle.x + 4, middle.y + 3)), (BLACK, BLACK, FACE));
    // The small panes 14 further right, the grip gone.
    for y in frame.status..frame.status + 22 {
        for x in first_pane..frame.right - 14 {
            assert_eq!(filling.get(x + 14, y), smaller.get(x, y), "at {x}, {y}: 14 further right");
        }
    }
    assert!((frame.bottom - 14..frame.bottom).all(|y| (frame.right - 14..frame.right).all(|x| filling.get(x, y) != SHADOW)), "no grip");
}

/// The menus by Windows 95's recipe: under their words, the word pressed in; the window's raised edge; items 19 tall,
/// lines 10 with their etched line in the middle; the words 20 in; the lit item navy across, its words white; greyed
/// words grey over white a pixel down and right; the Editor Scale's menu beside its item, its first item level with it.
#[test]
fn the_menus_are_windows_95s() {
    let size = (1280, 800);
    let frame = Frame::new(size, Look::default());
    for word in 0..menus::WORDS.len() {
        let menu = menus::menu(word, &showing());
        let lines = menus::lines(&menu);
        let sheet = draw(&Shown { menu: Some(Open { word, lit: Some(0), scales: None }), ..shown(size, Look::default(), Page::WorldGen, 0) });
        let place = frame.menu_words()[word];
        let at = frame.menu_at(word, &lines);
        assert_eq!((at.x, at.y), (place.x, place.y + place.h), "{word}: under its word");
        assert_eq!((sheet.get(place.x, place.y), sheet.get(place.x + place.w - 1, place.y + place.h - 1)), (SHADOW, WHITE), "{word}: its word pressed in");
        let (x0, y0, x1) = (at.x, at.y, at.x + at.w - 1);
        assert_eq!([sheet.get(x0, y0 + 5), sheet.get(x0 + 1, y0 + 5), sheet.get(x1, y0 + 5), sheet.get(x1 - 1, y0 + 5)], [LIGHT, WHITE, BLACK, SHADOW], "{word}: the raised edge");
        let rows = parts::menu_rows(&lines);
        assert_eq!(rows.iter().map(|(_, h)| h).sum::<i32>() + 2 * MENU_MARGIN, at.h);
        for (line, (top, h)) in lines.iter().zip(&rows) {
            match line {
                parts::MenuLine::Line => {
                    assert_eq!(*h, MENU_LINE);
                    let mid = y0 + top + h / 2;
                    assert_eq!((sheet.get(x0 + 4, mid), sheet.get(x0 + 4, mid + 1), sheet.get(x0 + 3, mid)), (SHADOW, WHITE, FACE), "{word}: the etched line, a pixel in");
                }
                parts::MenuLine::Item { .. } => assert_eq!(*h, MENU_ITEM),
            }
        }
        // The first item lit: navy across inside the margin.
        let (top, h) = rows[0];
        assert!((x0 + 3..=x1 - 3).all(|x| (y0 + top..y0 + top + h).any(|y| sheet.get(x, y) == NAVY)), "{word}: lit across");
        assert_eq!((sheet.get(x0 + 2, y0 + top + 1), sheet.get(x1 - 2, y0 + top + 1)), (FACE, FACE), "{word}: the margin");
        // The words 20 in (where no tick is before them): the first item's ink, its first letter underlined, from there.
        if let parts::MenuLine::Item { words, works, tick: parts::Tick::None, .. } = &lines[0] {
            let ink = if *works { WHITE } else { SHADOW };
            let first = (x0 + 3..x1 - 2).find(|x| (y0 + top..y0 + top + h).any(|y| sheet.get(*x, y) == ink));
            let lead = crate::gui::letters::ink(&words.replacen('&', "", 1)).0.min(0);
            assert_eq!(first, Some(x0 + 20 + lead), "{word}: its words 20 in");
        }
    }
    // Edit's Undo, greyed and not lit: grey ink, white a pixel down and right of it.
    let sheet = draw(&Shown { menu: Some(Open { word: 1, lit: None, scales: None }), ..shown(size, Look::default(), Page::WorldGen, 0) });
    let at = frame.menu_at(1, &menus::lines(&menus::menu(1, &showing())));
    let greyed: Vec<(i32, i32)> = (at.y + 3..at.y + 3 + MENU_ITEM).flat_map(|y| (at.x + 20..at.x + 60).map(move |x| (x, y))).filter(|(x, y)| sheet.get(*x, *y) == SHADOW).collect();
    assert!(!greyed.is_empty() && greyed.iter().any(|(x, y)| sheet.get(x + 1, y + 1) == WHITE), "greyed: grey with white under it");
    // Window's Editor Scale menu, open beside its item: its left over the menu's last 4 columns, its first item level.
    let window = menus::menu(3, &showing());
    let lines = menus::lines(&window);
    let at = frame.menu_at(3, &lines);
    let (sub, scales) = frame.scales_at(at, &window, 0, &showing());
    assert_eq!((sub.x, sub.y + MENU_MARGIN), (at.x + at.w - 4, at.y + MENU_MARGIN), "the scales' menu beside its item");
    assert_eq!(menus::lines(&scales).len(), 7, "Auto, a line, 1x to 5x");
    let sheet = draw(&Shown { menu: Some(Open { word: 3, lit: Some(0), scales: Some(Some(3)) }), ..shown(size, Look::default(), Page::WorldGen, 0) });
    assert!((sub.y + 3..sub.y + 3 + MENU_ITEM).any(|y| (sub.x + 3..sub.x + 16).any(|x| sheet.get(x, y) == BLACK)), "Auto's dot");
}

/// The menus answer the mouse: a word opens its menu; the pointer lights items and moves between menus; a greyed
/// item does nothing; an item that works is done and the menu shut; the Editor Scale's menu opens on its item and
/// chooses the scale; a press anywhere else shuts the menu and does nothing more; Esc shuts it.
#[test]
fn the_menus_answer_the_mouse() {
    let size = (1280, 800);
    let window = Window { filling: true, full_screen: false, fits: [true, true, true, false, false] };
    let mut editor = Editor::new();
    editor.open = true;
    editor.page = Page::WorldGen;
    editor.lay_out(size, Vec::new(), window);
    let frame = editor.frame;
    let now = Instant::now();
    let words = frame.menu_words();
    let centre = |r: Rect| (r.x + r.w / 2, r.y + r.h / 2);
    let item = |word: usize, k: usize, editor: &Editor| {
        let lines = menus::lines(&menus::menu(word, &editor.showing()));
        let at = frame.menu_at(word, &lines);
        let (top, h) = parts::menu_rows(&lines)[k];
        (at.x + 30, at.y + top + h / 2)
    };
    assert_eq!(editor.pressed(centre(words[0]), now), None);
    assert_eq!(editor.menu, Some(Open { word: 0, lit: None, scales: None }));
    editor.moved(item(0, 0, &editor));
    assert_eq!(editor.menu.and_then(|open| open.lit), Some(0));
    editor.moved(item(0, 1, &editor));
    assert_eq!(editor.menu.and_then(|open| open.lit), None, "a line isn't lit");
    editor.moved(centre(words[1]));
    assert_eq!(editor.menu.map(|open| open.word), Some(1), "onto Edit: its menu instead");
    assert_eq!(editor.pressed(item(1, 0, &editor), now), None);
    assert!(editor.menu.is_some(), "Undo is greyed: nothing, the menu stays");
    editor.let_go();
    assert_eq!(editor.menu, None, "Esc");
    // View > Status Bar: hidden, the pages to the frame's foot.
    editor.pressed(centre(words[2]), now);
    assert_eq!(editor.pressed(item(2, 0, &editor), now), None);
    assert_eq!((editor.menu, editor.status_bar), (None, false));
    editor.lay_out(size, Vec::new(), window);
    assert_eq!(editor.frame.pages.y + editor.frame.pages.h - 1, editor.frame.bottom);
    // View > Parameters Panel: hidden, the view the whole page.
    editor.pressed(centre(words[2]), now);
    editor.pressed(item(2, 1, &editor), now);
    editor.lay_out(size, Vec::new(), window);
    assert_eq!(editor.frame.view().x, editor.frame.pages.x);
    // Window > Editor Scale > 3x.
    editor.pressed(centre(words[3]), now);
    editor.moved(item(3, 0, &editor));
    assert_eq!(editor.menu.and_then(|open| open.scales), Some(None), "the scales' menu opens on its item");
    let showing = editor.showing();
    let (place, menu, opener) = frame.opened(editor.menu.unwrap(), &showing);
    let (sub, scales) = frame.scales_at(place, &menu, opener.unwrap(), &showing);
    let scale_row = |k: usize| {
        let (top, h) = parts::menu_rows(&menus::lines(&scales))[k];
        (sub.x + 30, sub.y + top + h / 2)
    };
    editor.moved(scale_row(4));
    assert_eq!(editor.menu.and_then(|open| open.scales), Some(Some(4)));
    assert_eq!(editor.menu.and_then(|open| open.lit), Some(0), "its item stays lit");
    assert_eq!(editor.pressed(scale_row(5), now), None, "4x can't be held: greyed");
    assert_eq!(editor.pressed(scale_row(4), now), Some(Ask::Scale));
    assert_eq!((editor.scale, editor.menu), (Some(3), None));
    // File > Return to Game, File > Quit.
    editor.pressed(centre(words[0]), now);
    assert_eq!(editor.pressed(item(0, 0, &editor), now), Some(Ask::ReturnToGame));
    editor.pressed(centre(words[0]), now);
    assert_eq!(editor.pressed(item(0, 2, &editor), now), Some(Ask::Quit));
    // A press elsewhere shuts it and does nothing more: not the drop-down's list.
    editor.pressed(centre(words[4]), now);
    assert_eq!(editor.pressed(centre(editor.frame.drop_down), now), None);
    assert_eq!((editor.menu, editor.list), (None, None));
    // Its own word again shuts it.
    editor.pressed(centre(words[4]), now);
    editor.pressed(centre(words[4]), now);
    assert_eq!(editor.menu, None);
}

/// The title bar answers the mouse as Windows 95's: a button held acts only when let go on it; a press on the bar moves
/// the window, two quick ones fill the screen or make it smaller; the edges size it, with their pointer, but not while
/// it fills the screen.
#[test]
fn the_title_bar_and_the_edges_answer_the_mouse() {
    let size = (1000, 600);
    let mut editor = Editor::new();
    editor.open = true;
    let smaller = Window { filling: false, full_screen: false, fits: [true; 5] };
    editor.lay_out(size, Vec::new(), smaller);
    let buttons = editor.frame.title_buttons();
    let on = |k: usize| (buttons[k].1 + 8, 14);
    let now = Instant::now();
    assert_eq!(editor.pressed(on(2), now), None);
    editor.moved((on(2).0 - 40, 14));
    editor.lay_out(size, Vec::new(), smaller);
    assert_eq!(editor.drawn.as_ref().and_then(|shown| shown.held), None, "off it, it pops up");
    assert_eq!(editor.released((on(2).0 - 40, 14)), None, "let go off it: nothing");
    editor.pressed(on(2), now);
    assert_eq!(editor.released(on(2)), Some(Ask::Quit));
    editor.pressed(on(0), now);
    assert_eq!(editor.released(on(0)), Some(Ask::Minimise));
    editor.pressed(on(1), now);
    assert_eq!(editor.released(on(1)), Some(Ask::FillOrRestore));
    // The title bar: a press moves the window; a second soon after and close by is a double click.
    assert_eq!(editor.pressed((300, 12), now), Some(Ask::Move));
    assert_eq!(editor.pressed((301, 13), now + Duration::from_millis(300)), Some(Ask::FillOrRestore));
    assert_eq!(editor.pressed((300, 12), now + Duration::from_millis(2000)), Some(Ask::Move));
    assert_eq!(editor.pressed((300, 12), now + Duration::from_millis(2700)), Some(Ask::Move), "too slow for a double click");
    // The edges.
    assert_eq!(editor.moved((1, 300)), Some(Side::West));
    assert_eq!(editor.pressed((1, 300), now), Some(Ask::Size(Side::West)));
    assert_eq!(editor.moved((500, 300)), None);
    // Filling the screen: no moving, no sizing, the restore mark on the middle button.
    let filling = Window { filling: true, ..smaller };
    editor.lay_out(size, Vec::new(), filling);
    assert_eq!(editor.frame.title_buttons()[1].0, crate::gui::parts::TitleButton::Restore);
    assert_eq!(editor.pressed((300, 12), now + Duration::from_secs(10)), None);
    assert_eq!(editor.moved((1, 300)), None);
    assert_eq!(editor.pressed((1, 300), now), None);
}

/// On the laptop's screen above the taskbar, the editor's smallest frame can be had at 1x to 3x, not 4x or 5x.
#[test]
fn the_scales_the_laptop_can_hold() {
    let (w, h) = Editor::smallest();
    println!("the editor's smallest frame: {w} x {h}");
    let room = (2560, 1500);
    let held: Vec<bool> = (1..=5).map(|scale| room.0 / scale >= w && room.1 / scale >= h).collect();
    assert_eq!(held, [true, true, true, false, false], "smallest {w} x {h}");
    let lines = menus::lines(&menus::scales(&Showing { fits: [true, true, true, false, false], ..showing() }));
    let works: Vec<bool> = lines.iter().filter_map(|line| if let parts::MenuLine::Item { works, .. } = line { Some(*works) } else { None }).collect();
    assert_eq!(works, [true, true, true, true, false, false], "Auto, then 1x to 5x");
}

/// The tabs by his rulings: none rises; the row starts 2 under the line; each name's ink 2 clear of its tab's edges and
/// no more ("as dense as possible", "nothing touches within 2px"); the chosen one drawn over its neighbours' edges by
/// 2 and nowhere past the row's ends, its foot open.
#[test]
fn the_tabs_follow_his_rulings() {
    let names = Page::WorldGen.tabs();
    let places = parts::tab_places(4, names);
    assert_eq!(places[0].0, 4, "the row starts at the panel's edge");
    assert!(places.windows(2).all(|pair| pair[0].0 + pair[0].1 == pair[1].0), "side by side");
    let sheet = drawn(Page::WorldGen, 3);
    for x in 4..553 {
        assert_eq!((sheet.get(x, 48), sheet.get(x, 49)), (FACE, FACE), "at {x}: the gap under the line, nothing rising");
    }
    for (name, (x, w)) in names.iter().zip(&places) {
        let inked: Vec<i32> = (x + 2..=x + w - 3).filter(|column| (52..71).any(|y| sheet.get(*column, y) == BLACK)).collect();
        assert_eq!((inked.first().copied(), inked.last().copied()), (Some(x + 4), Some(x + w - 5)), "{name}: its ink 2 clear of its edges");
    }
    let (tx, tw) = places[3];
    assert!((tx..tx + tw).all(|x| sheet.get(x, 50) == WHITE && sheet.get(x, 71) == FACE && sheet.get(x, 72) == FACE), "Climate's top, and its foot open");
    assert!((52..71).all(|y| sheet.get(tx - 2, y) == WHITE && sheet.get(tx - 1, y) == LIGHT && sheet.get(tx + tw, y) == SHADOW && sheet.get(tx + tw + 1, y) == BLACK), "Climate over its neighbours' edges by 2");
    let first = drawn(Page::WorldGen, 0);
    let (mx, mw) = places[0];
    assert!((52..71).all(|y| first.get(mx, y) == WHITE && first.get(mx + 1, y) == LIGHT && first.get(mx - 1, y) == FACE && first.get(mx - 2, y) == FACE));
    assert!((52..71).all(|y| first.get(mx + mw + 1, y) == BLACK), "Main over Landmass's edge by 2");
}

/// The status bar's small panes as dense as can be: each word's ink 2 clear of its pane's line, and of the grip.
#[test]
fn the_small_panes_are_dense() {
    let panes = vec!["CPU area 92 °C".to_string(), "GPU 71 °C".to_string()];
    let size = (1281, 801);
    let sheet = draw(&Shown { panes: panes.clone(), ..shown(size, Look::default(), Page::Registrar, 0) });
    let frame = Frame::new(size, Look::default());
    let mut right = frame.right;
    for (k, text) in panes.iter().enumerate().rev() {
        let (w, _) = parts::small_pane(text, k + 1 == panes.len());
        let x = right - w + 1;
        let inked: Vec<i32> = (x + 1..right).filter(|column| (frame.status + 1..frame.status + 21).any(|y| sheet.get(*column, y) == BLACK)).collect();
        let grip = if k + 1 == panes.len() { parts::GRIP } else { 0 };
        assert_eq!((inked.first().copied(), inked.last().copied()), (Some(x + 3), Some(right - 3 - grip)), "{text}: 2 clear of its line (and of the grip)");
        right -= w + 2;
    }
}

/// The view's frame by his ruling, thick and sunk: the sunken edge, then two rows of face, the same on both tabbed pages
/// whatever their pictures; and the Software map's board the same.
#[test]
fn the_frames_are_thick_and_sunk() {
    let frame = Frame::new((1281, 801), Look::default());
    for (page, r) in [(Page::TectonicSim, frame.view()), (Page::WorldGen, frame.view()), (Page::SoftwareMap, frame.board())] {
        let sheet = drawn(page, 0);
        let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w - 1, r.y + r.h - 1);
        assert_eq!((x1, y1), (1276, 772), "{page:?}: it ends where the rest ends, 2 above the status bar");
        let mid = (y0 + y1) / 2;
        let left: Vec<Colour> = (0..4).map(|k| sheet.get(x0 + k, mid)).collect();
        let right: Vec<Colour> = (0..4).map(|k| sheet.get(x1 - k, mid)).collect();
        assert_eq!(left, [SHADOW, BLACK, FACE, FACE], "{page:?}: its left, from the outside in");
        assert_eq!(right, [WHITE, LIGHT, FACE, FACE], "{page:?}: its right, from the outside in");
        let mid = (x0 + x1) / 2;
        let top: Vec<Colour> = (0..4).map(|k| sheet.get(mid, y0 + k)).collect();
        let bottom: Vec<Colour> = (0..4).map(|k| sheet.get(mid, y1 - k)).collect();
        assert_eq!((top, bottom), (vec![SHADOW, BLACK, FACE, FACE], vec![WHITE, LIGHT, FACE, FACE]), "{page:?}: its top and bottom");
    }
}

/// Made-up clicks: the drop-down opens, its list follows the pointer and changes the page; a tab pressed is chosen.
#[test]
fn the_drop_down_and_the_tabs_answer_the_mouse() {
    let mut editor = Editor::new();
    editor.open = true;
    let window = Window { filling: true, ..Window::default() };
    editor.lay_out((1280, 800), Vec::new(), window);
    let frame = editor.frame;
    let now = Instant::now();
    let d = frame.drop_down;
    editor.pressed((d.x + 20, d.y + 10), now);
    assert_eq!(editor.list, Some(None), "open, with nothing under the pointer");
    let list = frame.list();
    let row = |k: i32| (list.x + 20, list.y + 1 + k * parts::ROW + 7);
    editor.moved(row(2));
    assert_eq!(editor.list, Some(Some(2)));
    editor.pressed(row(2), now);
    assert_eq!((editor.page, editor.list), (Page::WorldGen, None));
    editor.pressed((d.x + 20, d.y + 10), now);
    editor.pressed((900, 400), now);
    assert_eq!((editor.page, editor.list), (Page::WorldGen, None));
    editor.lay_out((1280, 800), Vec::new(), window);
    let (tabs, _) = frame.tabbed();
    let places = parts::tab_places(tabs.x, Page::WorldGen.tabs());
    editor.pressed((places[3].0 + 5, tabs.y + 8), now);
    assert_eq!(editor.tabs[Page::WorldGen as usize], 3);
}

/// A sheet written as a PNG, `scale` times over (a pixel a square, as on the screen).
fn write(sheet: &Sheet, path: &std::path::Path, scale: u32) {
    let (w, h) = sheet.size();
    let mut rgba = Vec::with_capacity((w * h * 4 * scale * scale) as usize);
    for y in 0..h * scale {
        for x in 0..w * scale {
            let colour = sheet.get((x / scale) as i32, (y / scale) as i32);
            rgba.extend_from_slice(&if colour[3] == 0 { [0x5C, 0x76, 0x8C, 255] } else { colour });
        }
    }
    let file = std::fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w * scale, h * scale);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&rgba).unwrap();
}

/// The pages as a smaller window and filling the screen, each menu open, the Editor Scale's menu open, a title button
/// held, the status bar and the panel hidden, and the debug panel: written to `screenshots/229/` (outside git) for a
/// look before his.
#[test]
fn pictures_off_the_screen() {
    let folder = top().join("screenshots").join("229");
    std::fs::create_dir_all(&folder).unwrap();
    let panes = vec!["CPU area 92 °C".to_string(), "cores 95 °C".to_string(), "GPU 71 °C".to_string()];
    let size = (1280, 800);
    let filling = Look { filling: true, ..Look::default() };
    let with = |look: Look, page: Page, tab: usize| Shown { panes: panes.clone(), ..shown(size, look, page, tab) };
    for (page, tab) in [(Page::SoftwareMap, 0), (Page::TectonicSim, 3), (Page::WorldGen, 3), (Page::Registrar, 0)] {
        write(&draw(&with(Look::default(), page, tab)), &folder.join(format!("{}, smaller.png", page.name())), 1);
        write(&draw(&with(filling, page, tab)), &folder.join(format!("{}, filling the screen.png", page.name())), 1);
    }
    for (word, name) in menus::WORDS.iter().enumerate() {
        let open = Open { word, lit: Some(0), scales: None };
        write(&draw(&Shown { menu: Some(open), ..with(filling, Page::WorldGen, 3) }), &folder.join(format!("menu {}, {name}.png", word + 1)), 1);
    }
    let scales = Open { word: 3, lit: Some(0), scales: Some(Some(3)) };
    write(&draw(&Shown { menu: Some(scales), ..with(filling, Page::WorldGen, 3) }), &folder.join("menu 4, Window, Editor Scale.png"), 1);
    write(&draw(&Shown { held: Some(parts::TitleButton::Close), ..with(Look::default(), Page::WorldGen, 3) }), &folder.join("the close button held.png"), 1);
    write(&draw(&with(Look { status_bar: false, panel: false, filling: true }, Page::WorldGen, 3)), &folder.join("the status bar and the panel hidden.png"), 1);
    let mut panel = crate::panel::Panel::new();
    panel.lay_out();
    write(panel.sheet(), &folder.join("the debug panel.png"), 1);
}

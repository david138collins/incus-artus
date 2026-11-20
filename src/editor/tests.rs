//! The editor against his four pictures (`Art/95gui mockups/current/`), pixel for pixel, and its rulings by their
//! recipes; its clicks; and its pages written to `screenshots/228/` for a look off the screen. His pictures and the font
//! are on this machine, never committed: these tests read them there.

use super::frame::{self, Frame, Shown};
use super::*;
use crate::gui::parts::Rect;
use crate::gui::sheet::{BLACK, Colour, FACE, LIGHT, SHADOW, WHITE};

fn top() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// His picture, `name` in `Art/95gui mockups/current/`.
fn his(name: &str) -> (u32, u32, Vec<u8>) {
    let path = top().join("Art").join("95gui mockups").join("current").join(name);
    crate::textures::read_png(&path).unwrap_or_else(|| panic!("his picture isn't there: {}", path.display()))
}

fn drawn(page: Page, tab: usize) -> Sheet {
    assert!(crate::gui::letters::ironquill().is_some(), "Ironquill isn't in Art/fonts/Ironquill/");
    let mut sheet = Sheet::new(1, 1);
    frame::draw(&mut sheet, &Shown { size: (1280, 800), page, tab, list: None, panes: Vec::new() });
    sheet
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

/// His four pictures against the frame drawn at 1,280 x 800: every pixel that differs is in a place named with the
/// ruling or the build that makes it differ.
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

/// The tabs by his rulings: none rises; the row starts 2 under the line; each name's ink 2 clear of its tab's edges and
/// no more ("as dense as possible", "nothing touches within 2px"); the chosen one drawn over its neighbours' edges by
/// 2 and nowhere past the row's ends, its foot open.
#[test]
fn the_tabs_follow_his_rulings() {
    let names = Page::WorldGen.tabs();
    let places = crate::gui::parts::tab_places(4, names);
    assert_eq!(places[0].0, 4, "the row starts at the panel's edge");
    assert!(places.windows(2).all(|pair| pair[0].0 + pair[0].1 == pair[1].0), "side by side");
    let sheet = drawn(Page::WorldGen, 3);
    for x in 4..553 {
        assert_eq!((sheet.get(x, 48), sheet.get(x, 49)), (FACE, FACE), "at {x}: the gap under the line, nothing rising");
    }
    // Each name's ink inside its tab: from 4 in (2 of edge, 2 clear) to 4 in from its right.
    for (name, (x, w)) in names.iter().zip(&places) {
        let inked: Vec<i32> = (x + 2..=x + w - 3).filter(|column| (52..71).any(|y| sheet.get(*column, y) == BLACK)).collect();
        assert_eq!((inked.first().copied(), inked.last().copied()), (Some(x + 4), Some(x + w - 5)), "{name}: its ink 2 clear of its edges");
    }
    // Climate, chosen: as tall as the rest, over Topography's and Mineralogy's edges by 2, its foot open.
    let (tx, tw) = places[3];
    assert!((tx..tx + tw).all(|x| sheet.get(x, 50) == WHITE && sheet.get(x, 71) == FACE && sheet.get(x, 72) == FACE), "Climate's top, and its foot open");
    assert!((52..71).all(|y| sheet.get(tx - 2, y) == WHITE && sheet.get(tx - 1, y) == LIGHT && sheet.get(tx + tw, y) == SHADOW && sheet.get(tx + tw + 1, y) == BLACK), "Climate over its neighbours' edges by 2");
    // Main, chosen: its left edge the page's, so nothing past the row's start; over Landmass's edge by 2.
    let first = drawn(Page::WorldGen, 0);
    let (mx, mw) = places[0];
    assert!((52..71).all(|y| first.get(mx, y) == WHITE && first.get(mx + 1, y) == LIGHT && first.get(mx - 1, y) == FACE && first.get(mx - 2, y) == FACE));
    assert!((52..71).all(|y| first.get(mx + mw + 1, y) == BLACK), "Main over Landmass's edge by 2");
}

/// The status bar's small panes as dense as can be: each word's ink 2 clear of its pane's line, and of the grip.
#[test]
fn the_small_panes_are_dense() {
    let panes = vec!["CPU area 92 °C".to_string(), "GPU 71 °C".to_string()];
    let mut sheet = Sheet::new(1, 1);
    frame::draw(&mut sheet, &Shown { size: (1280, 800), page: Page::Registrar, tab: 0, list: None, panes: panes.clone() });
    let frame = Frame::new((1280, 800));
    let mut right = frame.right;
    for (k, text) in panes.iter().enumerate().rev() {
        let (w, _) = crate::gui::parts::small_pane(text, k + 1 == panes.len());
        let x = right - w + 1;
        let inked: Vec<i32> = (x + 1..right).filter(|column| (frame.status + 1..frame.status + 21).any(|y| sheet.get(*column, y) == BLACK)).collect();
        let grip = if k + 1 == panes.len() { crate::gui::parts::GRIP } else { 0 };
        assert_eq!((inked.first().copied(), inked.last().copied()), (Some(x + 3), Some(right - 3 - grip)), "{text}: 2 clear of its line (and of the grip)");
        right -= w + 2;
    }
}

/// The view's frame by his ruling, thick and sunk: the sunken edge, then two rows of face, the same on both tabbed pages
/// whatever their pictures; and the Software map's board the same.
#[test]
fn the_frames_are_thick_and_sunk() {
    let frame = Frame::new((1280, 800));
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
    editor.lay_out((1280, 800), Vec::new());
    let frame = Frame::new((1280, 800));
    let d = frame.drop_down;
    editor.pressed((d.x + 20, d.y + 10));
    assert_eq!(editor.list, Some(None), "open, with nothing under the pointer");
    let list = frame.list();
    let row = |k: i32| (list.x + 20, list.y + 1 + k * crate::gui::parts::ROW + 7);
    editor.moved(row(2));
    assert_eq!(editor.list, Some(Some(2)));
    editor.pressed(row(2));
    assert_eq!((editor.page, editor.list), (Page::WorldGen, None));
    // A press anywhere else shuts it, the page as it was.
    editor.pressed((d.x + 20, d.y + 10));
    editor.pressed((900, 400));
    assert_eq!((editor.page, editor.list), (Page::WorldGen, None));
    // Climate's tab.
    editor.lay_out((1280, 800), Vec::new());
    let (tabs, _) = frame.tabbed();
    let places = crate::gui::parts::tab_places(tabs.x, Page::WorldGen.tabs());
    editor.pressed((places[3].0 + 5, tabs.y + 8));
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

/// The four pages, the list open, and the debug panel and the temperatures, written to `screenshots/228/` (outside git)
/// for a look before his.
#[test]
fn pictures_off_the_screen() {
    let folder = top().join("screenshots").join("228");
    std::fs::create_dir_all(&folder).unwrap();
    let panes = vec!["CPU area 92 °C".to_string(), "cores 95 °C".to_string(), "GPU 71 °C".to_string()];
    for (page, tab) in [(Page::SoftwareMap, 0), (Page::TectonicSim, 3), (Page::WorldGen, 3), (Page::Registrar, 0)] {
        let mut sheet = Sheet::new(1, 1);
        frame::draw(&mut sheet, &Shown { size: (1280, 800), page, tab, list: None, panes: panes.clone() });
        write(&sheet, &folder.join(format!("{}.png", page.name())), 1);
    }
    let mut sheet = Sheet::new(1, 1);
    frame::draw(&mut sheet, &Shown { size: (1280, 800), page: Page::TectonicSim, tab: 0, list: Some(Some(2)), panes });
    write(&sheet, &folder.join("the list open.png"), 1);
    let mut panel = crate::panel::Panel::new();
    panel.lay_out();
    write(panel.sheet(), &folder.join("the debug panel.png"), 1);
}

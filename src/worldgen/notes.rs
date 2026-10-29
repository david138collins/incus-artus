//! Section notes [David, 2026-10-27]: a note button in the corner of each section's heading in the
//! Tectonic Sim and the World Compiler opens a box to write in. Notes are saved as they're
//! typed, to `notes/section notes.toml` (next to `presets/`), where Claude can read them too: a
//! table for each panel, a note for each section.

use bevy_egui::egui;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

/// Panel → section → the note.
type Pages = BTreeMap<String, BTreeMap<String, String>>;

struct Notes {
    pages: Pages,
    /// The note boxes open now: (panel, section).
    open: BTreeSet<(String, String)>,
}

static NOTES: LazyLock<Mutex<Notes>> = LazyLock::new(|| Mutex::new(Notes { pages: load(), open: BTreeSet::new() }));

thread_local! {
    /// The panel being drawn, whose sections get note buttons (none = no buttons).
    static PAGE: Cell<Option<&'static str>> = const { Cell::new(None) };
}

fn file() -> PathBuf {
    crate::debug::project_folder().join("notes").join("section notes.toml")
}

fn load() -> Pages {
    let Ok(text) = std::fs::read_to_string(file()) else { return Pages::new() };
    toml::from_str(&text).unwrap_or_else(|error| {
        bevy::log::warn!("Couldn't read the section notes ({error}); starting with none (the file is left as it is until a note is written).");
        Pages::new()
    })
}

fn save(pages: &Pages) {
    let path = file();
    let written = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| {
        let text = toml::to_string_pretty(pages).map_err(std::io::Error::other)?;
        std::fs::write(&path, text)
    });
    if let Err(error) = written {
        bevy::log::warn!("Couldn't save the section notes to {}: {error}", path.display());
    }
}

/// From here on, sections drawn are on this panel (`None`: no note buttons).
pub fn on_page(page: Option<&'static str>) {
    PAGE.with(|cell| cell.set(page));
}

/// The note button, at the right end of a section's heading (`row`: the heading's rectangle).
pub fn button(ui: &mut egui::Ui, title: &str, row: egui::Rect) {
    let Some(page) = PAGE.with(Cell::get) else { return };
    let key = (page.to_string(), title.to_string());
    let (written, open) = {
        let notes = NOTES.lock().unwrap();
        let written = notes.pages.get(page).and_then(|sections| sections.get(title)).cloned().unwrap_or_default();
        (written, notes.open.contains(&key))
    };
    let size = egui::vec2(22.0, row.height().max(18.0));
    let at = egui::Rect::from_min_size(egui::pos2(ui.max_rect().right() - size.x, row.center().y - size.y / 2.0), size);
    // Lit when there's a note, faint when there isn't.
    let colour = if written.trim().is_empty() { ui.visuals().weak_text_color() } else { egui::Color32::from_rgb(240, 200, 90) };
    let icon = egui::RichText::new("📝").color(colour);
    let hover = if written.trim().is_empty() { "Notes on this section (none yet)".to_string() } else { super::view::short(&written, 400) };
    // In a little area of its own over the heading, which takes no room in the panel: placing it
    // back up there with `ui.put` moved the panel's layout back up too, so the next section was
    // drawn over this one's contents.
    let mut corner = ui.new_child(egui::UiBuilder::new().max_rect(at).layout(egui::Layout::right_to_left(egui::Align::Center)));
    let response = corner.add(egui::Button::new(icon).frame(open).small()).on_hover_text(hover);
    if response.clicked() {
        let mut notes = NOTES.lock().unwrap();
        if !notes.open.remove(&key) {
            notes.open.insert(key);
        }
    }
}

/// The open note boxes. Call once a frame, after the panels.
pub fn windows(ctx: &egui::Context) {
    let mut notes = NOTES.lock().unwrap();
    let mut closed = Vec::new();
    let mut changed = false;
    let Notes { pages, open } = &mut *notes;
    for key in open.iter() {
        let (page, title) = key;
        let mut showing = true;
        let mut text = pages.get(page).and_then(|sections| sections.get(title)).cloned().unwrap_or_default();
        egui::Window::new(format!("Notes: {title}"))
            .id(egui::Id::new(("section notes", page, title)))
            .open(&mut showing)
            .default_size([320.0, 200.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.small(format!("{page} · saved as you type, in notes/section notes.toml"));
                let edit = egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY).desired_rows(8);
                if ui.add(edit).changed() {
                    let sections = pages.entry(page.clone()).or_default();
                    if text.is_empty() {
                        sections.remove(title);
                    } else {
                        sections.insert(title.clone(), text.clone());
                    }
                    changed = true;
                }
            });
        if !showing {
            closed.push(key.clone());
        }
    }
    for key in closed {
        open.remove(&key);
    }
    pages.retain(|_, sections| !sections.is_empty());
    if changed {
        save(pages);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_button_takes_no_room_in_the_panel() {
        // A heading, its open contents, then the button placed back up on the heading: what comes
        // next must still go below the contents, not over them.
        on_page(Some("test page"));
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let heading = ui.label("heading").rect;
            ui.label("contents");
            ui.label("more contents");
            let next = ui.cursor().min.y;
            button(ui, "heading", heading);
            assert_eq!(ui.cursor().min.y, next, "the button moved the panel's layout back up");
        });
        on_page(None);
    }

    #[test]
    fn notes_read_back_as_they_were_written() {
        let mut pages = Pages::new();
        pages.entry("World Interpreter".into()).or_default().insert("Lift".into(), "wide and low, scalloped:\n\"helps\" see far".into());
        pages.entry("Tectonic Sim".into()).or_default().insert("The simulation".into(), "one line".into());
        let text = toml::to_string_pretty(&pages).unwrap();
        assert_eq!(toml::from_str::<Pages>(&text).unwrap(), pages);
    }
}

//! Presets — save a page's settings to a named file and load them back.
//!
//! Any debug page can add a preset bar with one line:
//!
//! ```ignore
//! preset_bar(ui, "player", &mut settings);
//! ```
//!
//! Presets are plain TOML text files in `presets/<category>/<name>.toml` (next to `assets/`),
//! so they can also be read, compared, copied or edited by hand.
//!
//! A preset saved before a setting existed still loads: missing settings keep their
//! default values, and settings that no longer exist are ignored.
//!
//! This is the quick version. Proper "save as", editing and per-actor profiles come with
//! data loading (build step 7).

use super::project_folder;
use bevy_egui::egui;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// The folder a category's presets live in.
fn preset_folder(category: &str) -> PathBuf {
    project_folder().join("presets").join(category)
}

/// Letters, digits, `-` and `_` only (spaces become `_`), so any name makes a safe file name.
fn clean_name(name: &str) -> String {
    name.trim()
        .chars()
        .map(|c| if c == ' ' { '_' } else { c })
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// The names of the presets saved so far, in alphabetical order.
fn saved_presets(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

fn save<T: Serialize>(folder: &Path, name: &str, value: &T) -> Result<PathBuf, String> {
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let text = toml::to_string_pretty(value).map_err(|error| error.to_string())?;
    let path = folder.join(format!("{name}.toml"));
    std::fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(path)
}

fn load<T: DeserializeOwned>(folder: &Path, name: &str) -> Result<T, String> {
    let path = folder.join(format!("{name}.toml"));
    let text = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    toml::from_str(&text).map_err(|error| error.to_string())
}

/// The preset named "default", if one has been saved: what a page starts with, and what its
/// "back to defaults" buttons go back to.
pub fn default_preset<T: DeserializeOwned>(category: &str) -> Option<T> {
    load(&preset_folder(category), DEFAULT).ok()
}

/// A small light that's on while `value` is exactly the category's "default" preset, and off once
/// anything is changed (saving the changes as "default" lights it again). The preset is read again
/// only when its file changes.
pub fn default_light<T: DeserializeOwned + PartialEq + Clone + Send + Sync + 'static>(ui: &mut egui::Ui, category: &str, value: &T) {
    let path = preset_folder(category).join(format!("{DEFAULT}.toml"));
    let modified = std::fs::metadata(&path).and_then(|meta| meta.modified()).ok();
    let id = egui::Id::new(("default preset", category));
    let cached: Option<(Option<std::time::SystemTime>, Option<T>)> = ui.data(|data| data.get_temp(id));
    let default = match cached {
        Some((when, default)) if when == modified => default,
        _ => {
            let default = modified.and_then(|_| load::<T>(&preset_folder(category), DEFAULT).ok());
            ui.data_mut(|data| data.insert_temp(id, (modified, default.clone())));
            default
        }
    };
    let on = default.as_ref() == Some(value);
    let (text, colour) = if on { ("● default", egui::Color32::from_rgb(120, 230, 120)) } else { ("○ default", egui::Color32::from_gray(110)) };
    ui.label(egui::RichText::new(text).color(colour)).on_hover_text(if default.is_none() {
        "No 'default' preset saved for this page yet: name one 'default' (Presets) and save it."
    } else if on {
        "These settings are exactly the 'default' preset."
    } else {
        "Changed from the 'default' preset. Save under the name 'default' (Ctrl+S) to make these the default."
    });
}

/// The preset name that becomes a page's default.
const DEFAULT: &str = "default";

/// The name in a category's preset box (the same box wherever the bar is drawn, so Ctrl+S can save
/// under it while the bar is folded away).
fn name_id(category: &str) -> egui::Id {
    egui::Id::new(("preset", category))
}

/// Ctrl+S saves `value()` as the preset named in the category's preset box (replacing it). Call it
/// every frame while the page is showing, whether or not its preset bar is.
pub fn save_shortcut<T: Serialize>(ctx: &egui::Context, category: &str, value: impl FnOnce() -> T) {
    let toast = egui::Id::new("preset saved toast");
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
        let name: String = ctx.data(|data| data.get_temp(name_id(category))).unwrap_or_default();
        let clean = clean_name(&name);
        let message = if clean.is_empty() {
            "Ctrl+S: type a name in the preset box first (Presets).".to_string()
        } else {
            match save(&preset_folder(category), &clean, &value()) {
                Ok(_) if clean == DEFAULT => "Saved 'default': the page starts with it now.".to_string(),
                Ok(_) => format!("Saved '{clean}'."),
                Err(error) => format!("Couldn't save: {error}"),
            }
        };
        ctx.data_mut(|data| {
            data.insert_temp(toast, (message.clone(), ctx.input(|input| input.time)));
            data.insert_temp(name_id(category).with("status"), message);
        });
    }
    // The last save's message, at the bottom of the screen for a few seconds.
    if let Some((message, when)) = ctx.data(|data| data.get_temp::<(String, f64)>(toast)) {
        let age = ctx.input(|input| input.time) - when;
        if age < 3.0 {
            egui::Area::new(toast).anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -40.0)).order(egui::Order::Foreground).show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(message));
            });
            ctx.request_repaint();
        }
    }
}

/// A row of preset controls: name box, Save, a Load list, and a button to open the folder.
pub fn preset_bar<T: Serialize + DeserializeOwned>(ui: &mut egui::Ui, category: &str, value: &mut T) {
    let folder = preset_folder(category);
    let id = name_id(category);
    let status_id = id.with("status");
    let mut name: String = ui.data(|data| data.get_temp(id)).unwrap_or_default();
    let mut status: Option<String> = ui.data(|data| data.get_temp(status_id));

    // Wraps onto a second line in a narrow panel, rather than forcing the panel wider.
    ui.horizontal_wrapped(|ui| {
        ui.label("Preset:");
        ui.add(egui::TextEdit::singleline(&mut name).desired_width(110.0).hint_text("name")).on_hover_text(
            "Ctrl+S saves under this name. A preset named 'default' is what the page starts with, and what its\n\
             'back to defaults' buttons go back to.",
        );
        let clean = clean_name(&name);
        let exists = !clean.is_empty() && folder.join(format!("{clean}.toml")).exists();
        let save_label = if exists { "Save (replace)" } else { "Save" };
        if ui.add_enabled(!clean.is_empty(), egui::Button::new(save_label)).clicked() {
            status = Some(match save(&folder, &clean, value) {
                Ok(path) => format!("Saved {}", path.display()),
                Err(error) => format!("Couldn't save: {error}"),
            });
        }
        egui::ComboBox::from_id_salt(id.with("load"))
            .selected_text("Load…")
            .show_ui(ui, |ui| {
                let presets = saved_presets(&folder);
                if presets.is_empty() {
                    ui.label("(none saved yet)");
                }
                for preset in presets {
                    if ui.selectable_label(false, &preset).clicked() {
                        status = Some(match load::<T>(&folder, &preset) {
                            Ok(loaded) => {
                                *value = loaded;
                                name = preset.clone();
                                format!("Loaded '{preset}'")
                            }
                            Err(error) => format!("Couldn't load '{preset}': {error}"),
                        });
                    }
                }
            });
        if ui.small_button("Folder").on_hover_text("Open the presets folder").clicked() {
            let _ = std::fs::create_dir_all(&folder);
            if let Err(error) = std::process::Command::new("explorer").arg(&folder).spawn() {
                status = Some(format!("Couldn't open the folder: {error}"));
            }
        }
    });
    if let Some(message) = &status {
        ui.small(message);
    }

    ui.data_mut(|data| {
        data.insert_temp(id, name);
        if let Some(message) = status {
            data.insert_temp(status_id, message);
        }
    });
}

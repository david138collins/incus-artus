//! The Registrar's Objects tab [`docs/objects.md`; David, 2026-11-01: "blocks are blocks, objects are
//! objects, and blueprints are made of both"]: every object, its settings, and bringing a new model in.
//!
//! - **The list** (left): each object, by name.
//! - **The dock** (middle): the object open: its name, its model, how tall it stands, whether it's
//!   solid, how it sits; saved to its data file (`data/objects.rs`), and the game picks it up in a
//!   moment. Attachments (a flame, its light) come with stops 2 and 3, greyed until then.
//! - **"Load a model…":** a glTF from `Art/` (Blockbench's export, its textures packed inside),
//!   copied into `assets/models/` with a data file made for it.
//!
//! To see one: put it in a hotbar slot (the Player page) and place it in the world. A turning preview
//! here comes later.

use crate::data::{ObjectDef, ObjectRegistry, Sits, data_folder, header_of, models_folder, object_file_text};
use bevy::math::IVec3;
use bevy_egui::egui;
use std::path::{Path, PathBuf};

/// The Objects tab's own state.
#[derive(Default)]
pub struct ObjectsTab {
    /// The object open in the dock, and its settings as being edited.
    selected: Option<String>,
    draft: Option<ObjectDef>,
    tags_text: String,
    /// What the last action did.
    message: String,
    /// A model being brought in.
    loading: Option<Loading>,
}

/// A model being brought in: the glTFs in `Art/`, the one picked, and the object's name.
struct Loading {
    files: Vec<PathBuf>,
    picked: Option<PathBuf>,
    name: String,
    as_made: bool,
    height: f32,
}

pub fn draw(root: &mut egui::Ui, tab: &mut ObjectsTab, objects: &ObjectRegistry) {
    egui::Panel::left("registrar objects list").resizable(true).default_size(260.0).size_range(180.0..=600.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| list(ui, tab, objects));
    });
    egui::CentralPanel::default().show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| dock(ui, tab, objects));
    });
}

fn list(ui: &mut egui::Ui, tab: &mut ObjectsTab, objects: &ObjectRegistry) {
    ui.heading("Objects");
    if ui.button("Load a model…").on_hover_text("A glTF from Art/ (Blockbench: File → Export → glTF, textures packed inside), made into an object").clicked() {
        tab.loading = if tab.loading.is_some() { None } else { Some(Loading { files: gltf_files(), picked: None, name: String::new(), as_made: true, height: 1.0 }) };
    }
    ui.separator();
    if objects.defs().is_empty() {
        ui.label("None yet.");
    }
    for def in objects.defs() {
        let open = tab.selected.as_deref() == Some(def.name.as_str());
        if ui.selectable_label(open, &def.display_name).on_hover_text(&def.name).clicked() {
            tab.selected = Some(def.name.clone());
            tab.loading = None;
        }
    }
    for (file, problem) in &objects.problems {
        ui.colored_label(egui::Color32::from_rgb(230, 160, 60), format!("{}: {problem}", Path::new(file).file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned())));
    }
}

fn dock(ui: &mut egui::Ui, tab: &mut ObjectsTab, objects: &ObjectRegistry) {
    if !tab.message.is_empty() {
        ui.label(&tab.message);
        ui.separator();
    }
    if tab.loading.is_some() {
        loading(ui, tab);
        return;
    }
    let Some(def) = tab.selected.as_deref().and_then(|name| objects.by_name(name)) else {
        ui.label("Click an object in the list to open it here, or load a model.");
        return;
    };
    if tab.draft.as_ref().is_none_or(|draft| draft.name != def.name) {
        tab.draft = Some(def.clone());
        tab.tags_text = def.tags.join(", ");
    }
    let Some(draft) = tab.draft.as_mut() else { return };
    // Its shape is read from its model, not edited here: kept as the game has it.
    draft.shape = def.shape.clone();
    ui.heading(&def.display_name);
    ui.small(&def.name);
    ui.horizontal(|ui| {
        ui.label("name:");
        ui.text_edit_singleline(&mut draft.display_name);
    });
    let models = model_files();
    ui.horizontal(|ui| {
        ui.label("model:");
        egui::ComboBox::from_id_salt("object model").selected_text(&draft.model).show_ui(ui, |ui| {
            for model in &models {
                ui.selectable_value(&mut draft.model, model.clone(), model);
            }
        });
    });
    size_settings(ui, draft, def);
    ui.checkbox(&mut draft.solid, "solid").on_hover_text("Bodies bump into it: into its model's own shape, pixel by pixel");
    ui.horizontal(|ui| {
        ui.label("sits:");
        ui.radio_value(&mut draft.sits, Sits::Floor, "on a floor").on_hover_text("Standing in the middle of its cell");
        ui.add_enabled(false, egui::RadioButton::new(draft.sits == Sits::Wall, "on a wall")).on_disabled_hover_text("Later (torches)");
        ui.add_enabled(false, egui::RadioButton::new(draft.sits == Sits::Ceiling, "from a ceiling")).on_disabled_hover_text("Later");
    });
    ui.horizontal(|ui| {
        ui.label("tags:");
        if ui.text_edit_singleline(&mut tab.tags_text).on_hover_text("Separated by commas").changed() {
            draft.tags = tab.tags_text.split(',').map(str::trim).filter(|tag| !tag.is_empty()).map(str::to_string).collect();
        }
    });
    ui.add_enabled_ui(false, |ui| {
        egui::CollapsingHeader::new("Attachments (a flame, its light)").id_salt("object attachments").show(ui, |_| {});
    })
    .response
    .on_disabled_hover_text("Things hung on the model's named points (a Blockbench locator, e.g. \"flame\"): an animated sprite (objects stop 2), a light (stop 3)");

    let changed = draft != def;
    let draft = draft.clone();
    let path = objects.source(&def.name).map(Path::to_path_buf);
    ui.horizontal(|ui| {
        if ui.add_enabled(changed && path.is_some(), egui::Button::new("Save to the object file")).clicked()
            && let Some(path) = &path
        {
            let header = std::fs::read_to_string(path).map(|text| header_of(&text)).unwrap_or_default();
            tab.message = match std::fs::write(path, object_file_text(&draft, &header)) {
                Ok(()) => format!("Saved {}; the game picks it up in a moment.", path.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned())),
                Err(error) => format!("Couldn't save {}: {error}.", path.display()),
            };
        }
        if ui.add_enabled(changed, egui::Button::new("Revert")).clicked() {
            tab.draft = Some(def.clone());
            tab.tags_text = def.tags.join(", ");
        }
    });
    ui.add_space(8.0);
    ui.small("To see it: put it in a hotbar slot (the Player page) and place it in the world. A turning preview here comes later.");
}

/// Its size: as made, or scaled to a height; how far it may stick over; and what that makes it (small,
/// on the octant grid, or a full-block object), with the room it holds.
fn size_settings(ui: &mut egui::Ui, draft: &mut ObjectDef, def: &ObjectDef) {
    let mut as_made = draft.height.is_none();
    if ui.checkbox(&mut as_made, "as made").on_hover_text("At the size it was made: Blockbench's 16 pixels to a block, the blocks' own pixels").changed() {
        let made_tall = def.shape.as_ref().map_or(1.0, |shape| shape.size.z as f32 / 16.0 / shape.scale);
        draft.height = if as_made { None } else { Some((made_tall * 16.0).round() / 16.0) };
    }
    let mut height = draft.height.unwrap_or_else(|| def.shape.as_ref().map_or(1.0, |shape| shape.size.z as f32 / 16.0 / shape.scale));
    let slider = ui.add_enabled(!as_made, crate::debug::Slider::new(&mut height, 0.1..=8.0).text("how tall it stands (blocks)"));
    slider.on_hover_text("The model is scaled to this, evenly, whatever size it was made at").on_disabled_hover_text("Untick \"as made\" to scale it");
    if !as_made {
        draft.height = Some(height);
    }
    let mut over = draft.stick_over as i32;
    ui.add(crate::debug::Slider::new(&mut over, 0..=8).text("may stick over (pixels)")).on_hover_text("How far it may stick out of the room it holds, and just clip, and still count as that size");
    draft.stick_over = over as u8;
    // Its size as it would be with these settings (the saved shape, scaled), and what that makes it.
    let Some(shape) = &def.shape else {
        ui.colored_label(egui::Color32::from_rgb(230, 160, 60), "Its model couldn't be read: it counts as a block across.");
        return;
    };
    let made_tall = shape.size.z as f32 / 16.0 / shape.scale;
    let scale = draft.height.map_or(1.0, |height| height / made_tall.max(1e-3));
    let mut preview = draft.clone();
    let size = (shape.size.as_vec3() * scale / shape.scale).round().as_ivec3().max(IVec3::ONE);
    preview.shape = Some(crate::data::Shape { size, ..shape.clone() });
    ui.label(format!("size: {} × {} × {} pixels (across, along, up)", size.x, size.y, size.z));
    let room = crate::world::room(&preview, 0);
    let held = if crate::world::is_small(&preview) {
        format!("small: on the octant grid or a cell's middle, holding an octant across and {} up", room.z / 8)
    } else {
        format!("full-block: snaps as blocks do, holding {} × {} blocks across and {} octants up", room.x / 16, room.y / 16, room.z / 8)
    };
    ui.label(held).on_hover_text("Half a block across or less (with what it may stick over) is small. Its model's own shape is what bodies bump into, not this room.");
    if draft.height != def.height {
        ui.small("(what bodies bump into is worked out again when it's saved)");
    }
}

/// Bringing a model in: pick a glTF from Art/, name it, register it.
fn loading(ui: &mut egui::Ui, tab: &mut ObjectsTab) {
    let Some(loading) = tab.loading.as_mut() else { return };
    ui.heading("Load a model");
    if loading.files.is_empty() {
        ui.label("No glTF files in Art/. In Blockbench: File → Export → glTF, with the textures packed inside.");
    }
    for file in loading.files.clone() {
        let shown = file.strip_prefix(crate::debug::project_folder().join("Art")).unwrap_or(&file).display().to_string();
        if ui.selectable_label(loading.picked.as_ref() == Some(&file), shown).clicked() {
            loading.name = super::registrar::asset_name(&file);
            loading.picked = Some(file);
        }
    }
    let Some(picked) = loading.picked.clone() else { return };
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("name:");
        ui.text_edit_singleline(&mut loading.name);
    });
    ui.checkbox(&mut loading.as_made, "as made").on_hover_text("At the size it was made: Blockbench's 16 pixels to a block");
    ui.add_enabled(!loading.as_made, crate::debug::Slider::new(&mut loading.height, 0.1..=8.0).text("how tall it stands (blocks)")).on_disabled_hover_text("Untick \"as made\" to scale it");
    let name = loading.name.trim().to_string();
    let ok_name = !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    let file = data_folder().join("incus").join("objects").join(format!("{name}.toml"));
    let taken = file.exists();
    // A glTF that points at files beside it (rather than packing them inside) wouldn't come with them.
    let separate = std::fs::read_to_string(&picked).map(|text| text.contains("\"uri\":\"") && text.split("\"uri\":\"").skip(1).any(|uri| !uri.starts_with("data:"))).unwrap_or(false);
    if separate {
        ui.colored_label(egui::Color32::from_rgb(230, 160, 60), "This glTF keeps its textures in separate files. Export it again with them packed inside (Blockbench does this by default).");
    }
    let enabled = ok_name && !taken && !separate;
    let register = ui.add_enabled(enabled, egui::Button::new(format!("Register incus:{name}"))).on_disabled_hover_text(if !ok_name { "Lowercase letters, digits and _ only" } else if taken { "There's an object of that name already" } else { "Its textures need to be inside it" }).clicked();
    if register {
        let model = format!("{name}.gltf");
        let height = (!loading.as_made).then_some(loading.height);
        let def = ObjectDef { name: format!("incus:{name}"), display_name: name.replace('_', " "), model: model.clone(), height, stick_over: 2, solid: false, sits: Sits::Floor, tags: Vec::new(), shape: None };
        let header = format!("# {}: from {}.\n# See docs/objects.md for every setting.\n\n", def.display_name, picked.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned()));
        let result = std::fs::create_dir_all(models_folder())
            .and_then(|()| std::fs::copy(&picked, models_folder().join(&model)).map(|_| ()))
            .and_then(|()| std::fs::create_dir_all(file.parent().unwrap_or(Path::new("."))))
            .and_then(|()| std::fs::write(&file, object_file_text(&def, &header)));
        tab.message = match result {
            Ok(()) => format!("Registered incus:{name}; the game picks it up in a moment."),
            Err(error) => format!("Couldn't register it: {error}."),
        };
        if tab.message.starts_with("Registered") {
            tab.selected = Some(def.name);
            tab.loading = None;
        }
    }
}

/// Every glTF in Art/ (a few folders deep).
fn gltf_files() -> Vec<PathBuf> {
    fn walk(folder: &Path, depth: u32, found: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() && depth < 6 {
                walk(&path, depth + 1, found);
            } else if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("gltf")) {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&crate::debug::project_folder().join("Art"), 0, &mut found);
    found.sort();
    found
}

/// The models in `assets/models/`.
fn model_files() -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(models_folder())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension.eq_ignore_ascii_case("gltf") || extension.eq_ignore_ascii_case("glb")))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

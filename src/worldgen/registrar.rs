//! The Registrar: the Editor Suite's first tab, the library of every asset the game has, and its
//! Color Lab (docs/worldgen.md, "The Registrar and its Color Lab"). One page, side by side
//! [David, 2026-10-29]:
//!   - the library, blocks grouped by their tags;
//!   - the dock: the open asset, its faces and its own palette;
//!   - the palettes: the master palette (its splines, then the colours not on one yet), and the
//!     sub-palette of whatever's selected.
//!
//! Built up a step at a time ("sneak up on the final configuration"): this first version shows the
//! library and the palette, and splines made by picking colours (shaped by curves later);
//! registering new assets and HSL transforms follow.

use crate::data::master_palette::{self, MasterPalette, Rgb};
use super::palette::Spread;
use crate::data::{BlockDef, BlockRegistry, block_texture_folder};
use bevy_egui::egui::{self, Color32, Stroke};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// A block texture as the Registrar sees it.
struct Texture {
    /// Its colours, most used first, with how many pixels each.
    colours: Vec<(Rgb, u32)>,
    /// For showing it, made when first drawn.
    image: Option<egui::TextureHandle>,
    pixels: Vec<u8>,
    /// It as the sub-palette's transform makes it, and the transform's number it was made for.
    preview: Option<(u64, egui::TextureHandle)>,
    /// Why it couldn't be read, if it couldn't.
    problem: Option<String>,
}

#[derive(Default)]
pub struct Registrar {
    palette: MasterPalette,
    /// Why the palette file couldn't be read (then nothing is written to it, so it isn't lost).
    palette_problem: Option<String>,
    /// The block registry's revision the textures were read at.
    read_at: Option<u64>,
    textures: BTreeMap<String, Texture>,
    /// The selected blocks (names); the first is the one open in the dock.
    selected: Vec<String>,
    /// The block under the mouse in the library, or in the dock: its colours light up.
    hovered: Option<String>,
    /// The last thing that happened, shown under the palette.
    message: String,
    /// Colours picked (clicked) in any palette, to put on a spline or take off one.
    picked: BTreeSet<Rgb>,
    /// The colour last under the mouse, shown large at the top of the palettes.
    focus: Option<Rgb>,
    /// A box being dragged over colour squares.
    band: Option<Band>,
    /// The sub-palette's transform, what it makes of each of the selection's colours, what that was
    /// worked out for (transform, selection, textures read at), and a number that changes with it.
    transform: Transform,
    mapping: BTreeMap<Rgb, Rgb>,
    mapped_for: Option<(Transform, Vec<String>, Option<u64>, u64)>,
    /// Read the palette and textures again (after an undo).
    reread: bool,
    /// The undo steps, oldest first: what each change was.
    undo: Vec<String>,
    /// Per texture, the colours the Registrar replaced in it (to say if they come back).
    replaced: BTreeMap<String, BTreeSet<Rgb>>,
    /// Which of the master palette's colours are shown.
    view: PaletteView,
    /// "Clear out unused colours" was pressed: its preview is showing, waiting for a yes.
    clearing: bool,
    /// Goes up by one whenever the palette is saved.
    palette_changes: u64,
    mapping_version: u64,
    /// Show the blocks as they are, not as the transform would make them.
    before: bool,
}

/// Blocks worth showing: everything drawn (not air).
fn shown(def: &BlockDef) -> bool {
    def.opaque || def.see_through
}

/// A block's texture names, each once, in face order.
fn texture_names(def: &BlockDef) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for face in &def.faces {
        if !face.is_empty() && !names.contains(&face.as_str()) {
            names.push(face);
        }
    }
    names
}

impl Registrar {
    /// Read the palette and every block's textures again if the blocks or textures changed (or on
    /// first opening, or after an undo). Colours not in the palette yet join it, unregulated, and the
    /// file is saved. Whatever changed a texture (the Registrar or Aseprite), what changed is said:
    /// how many colours it has now, and whether colours the Registrar replaced in it have come back.
    fn refresh(&mut self, registry: &BlockRegistry) {
        if self.read_at == Some(registry.revision()) && !self.reread {
            return;
        }
        let first = self.read_at.is_none() || self.reread;
        self.read_at = Some(registry.revision());
        self.reread = false;
        if first {
            self.palette_problem = None;
            match MasterPalette::load_from(&master_palette::palette_file()) {
                Ok(palette) => self.palette = palette,
                Err(problem) => self.palette_problem = Some(problem),
            }
            self.undo = undo_steps(&undo_folder()).into_iter().map(|(_, what)| what).collect();
        }
        let before: BTreeMap<String, Vec<(Rgb, u32)>> = std::mem::take(&mut self.textures).into_iter().map(|(name, texture)| (name, texture.colours)).collect();
        let folder = block_texture_folder();
        let mut used: Vec<Rgb> = Vec::new();
        for def in registry.defs().iter().filter(|def| shown(def)) {
            for name in texture_names(def) {
                if self.textures.contains_key(name) {
                    continue;
                }
                let texture = match crate::render::textures::load_texture(&folder.join(format!("{name}.png"))) {
                    Ok(pixels) => Texture { colours: master_palette::colours_in(&pixels), image: None, preview: None, pixels, problem: None },
                    Err(problem) => Texture { colours: Vec::new(), image: None, preview: None, pixels: Vec::new(), problem: Some(problem) },
                };
                used.extend(texture.colours.iter().map(|&(rgb, _)| rgb));
                self.textures.insert(name.to_string(), texture);
            }
        }
        // What changed, texture by texture.
        let mut said: Vec<String> = Vec::new();
        for (name, texture) in &self.textures {
            let Some(was) = before.get(name).filter(|was| **was != texture.colours) else { continue };
            let mut line = format!("{name}.png changed: {} colours (was {})", texture.colours.len(), was.len());
            if let Some(replaced) = self.replaced.get(name) {
                let back = texture.colours.iter().filter(|(rgb, _)| replaced.contains(rgb)).count();
                if back > 0 {
                    line += &format!(", {back} of the colours the Registrar replaced are back (saved over from Aseprite?)");
                }
            }
            said.push(line);
        }
        if self.palette_problem.is_some() {
            return;
        }
        let added = self.palette.take_in(used);
        if !added.is_empty() {
            said.push(match self.palette.save_to(&master_palette::palette_file()) {
                Ok(()) => format!("{} new colours joined the master palette, not on a spline yet", added.len()),
                Err(error) => format!("{} new colours, but the palette couldn't be saved: {error}", added.len()),
            });
            self.palette_changes += 1;
        }
        if !said.is_empty() {
            self.message = said.join(". ") + ".";
        }
    }

    /// Write the transform into the selected blocks' textures (every block using one of them
    /// changes too), after saving copies for undo. The colours it replaced are remembered, so it can
    /// be said if they come back.
    fn apply(&mut self, defs: &[&BlockDef]) {
        let names: BTreeSet<String> = defs.iter().filter(|def| self.selected.contains(&def.name)).flat_map(|def| texture_names(def)).map(str::to_string).collect();
        let folder = block_texture_folder();
        let paths: Vec<PathBuf> = names.iter().map(|name| folder.join(format!("{name}.png"))).collect();
        let what = format!("Applied to {}", names.iter().map(|name| format!("{name}.png")).collect::<Vec<_>>().join(", "));
        if let Err(error) = save_undo_step(&undo_folder(), &what, &paths) {
            self.message = format!("Nothing written: couldn't save the undo copies ({error}).");
            return;
        }
        let mut written = 0;
        for (name, path) in names.iter().zip(&paths) {
            let Some(texture) = self.textures.get(name).filter(|texture| texture.problem.is_none()) else { continue };
            let mut pixels = texture.pixels.clone();
            for pixel in pixels.chunks_exact_mut(4) {
                if let Some(&to) = self.mapping.get(&[pixel[0], pixel[1], pixel[2]]).filter(|to| **to != [pixel[0], pixel[1], pixel[2]]) {
                    self.replaced.entry(name.clone()).or_default().insert([pixel[0], pixel[1], pixel[2]]);
                    pixel[..3].copy_from_slice(&to);
                }
            }
            match write_png(path, &pixels) {
                Ok(()) => written += 1,
                Err(error) => {
                    self.message = format!("Couldn't write {name}.png: {error}. Undo puts back what was written.");
                    return;
                }
            }
        }
        self.undo.push(what);
        // The blocks now look as the transform made them, so it's done.
        let kept = self.transform;
        self.transform = Transform { spread: kept.spread, snap: kept.snap, ..Transform::default() };
        self.message = format!("Wrote {written} textures; the game picks them up in a moment.");
    }

    /// Colours on no spline that no block uses any more.
    fn unused(&self) -> Vec<u32> {
        let used: BTreeSet<Rgb> = self.textures.values().flat_map(|texture| texture.colours.iter().map(|&(rgb, _)| rgb)).collect();
        self.palette.colours.iter().filter(|&(&slot, colour)| !self.palette.is_regulated(slot) && !used.contains(&colour.rgb)).map(|(&slot, _)| slot).collect()
    }

    /// Take the unused colours out of the palette, after saving a copy of it for undo.
    fn clear_unused(&mut self) {
        let unused = self.unused();
        let what = format!("Cleared out {} unused colours", unused.len());
        if let Err(error) = save_undo_step(&undo_folder(), &what, &[master_palette::palette_file()]) {
            self.message = format!("Nothing cleared: couldn't save the undo copy ({error}).");
            return;
        }
        self.palette.remove(&unused);
        self.picked.retain(|rgb| self.palette.slot_of(*rgb).is_some());
        self.undo.push(what.clone());
        self.save(format!("{what}; their numbers are retired."));
    }

    /// Undo the newest change: its files put back, and everything read again.
    fn undo_last(&mut self) {
        self.message = match undo_last(&undo_folder()) {
            Ok(what) => format!("Undone: {what}."),
            Err(error) => format!("Couldn't undo: {error}."),
        };
        self.reread = true;
        // The old colours come back on purpose: not worth a warning.
        self.replaced.clear();
        self.palette_changes += 1;
    }

    /// A block's colours, combined over its textures, with how many pixels each.
    fn block_colours(&self, def: &BlockDef) -> BTreeMap<Rgb, u32> {
        let mut colours = BTreeMap::new();
        for name in texture_names(def) {
            for &(rgb, count) in self.textures.get(name).map_or(&[][..], |texture| &texture.colours) {
                *colours.entry(rgb).or_default() += count;
            }
        }
        colours
    }

    /// How many of a block's colours aren't on a spline (0: regulated).
    fn unregulated(&self, def: &BlockDef) -> usize {
        self.block_colours(def).keys().filter(|&&rgb| !self.palette.slot_of(rgb).is_some_and(|slot| self.palette.is_regulated(slot))).count()
    }
}

/// Draws the Registrar.
pub fn draw(root: &mut egui::Ui, registrar: &mut Registrar, registry: &BlockRegistry) {
    registrar.refresh(registry);
    // Escape deselects every colour; a box whose squares have gone ends when the mouse is let go.
    let (escape, held) = root.input(|input| (input.key_pressed(egui::Key::Escape), input.pointer.primary_down() || input.pointer.primary_released()));
    if escape && !root.ctx().egui_wants_keyboard_input() {
        registrar.picked.clear();
    }
    if !held {
        registrar.band = None;
    }
    registrar.hovered = None;
    let defs: Vec<&BlockDef> = registry.defs().iter().filter(|def| shown(def)).collect();
    registrar.update_mapping(&defs);
    egui::Panel::left("registrar library").resizable(true).default_size(260.0).size_range(180.0..=600.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| library(ui, registrar, &defs));
    });
    egui::Panel::right("registrar palettes").resizable(true).default_size(420.0).size_range(260.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| palettes(ui, registrar, &defs));
    });
    egui::CentralPanel::default().show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| dock(ui, registrar, &defs));
    });
}

// ── The library ───────────────────────────────────────────────────────────

/// Every block, grouped by tag (a block with several tags is in each group; untagged ones at the
/// end). Click to open a block; Ctrl + click to add it to the selection, or take it out.
fn library(ui: &mut egui::Ui, registrar: &mut Registrar, defs: &[&BlockDef]) {
    ui.heading("Library");
    ui.small(format!("{} blocks. Click to open; Ctrl + click to select more.", defs.len()));
    ui.separator();
    let mut groups: BTreeMap<&str, Vec<&BlockDef>> = BTreeMap::new();
    let mut untagged = Vec::new();
    for &def in defs {
        if def.tags.is_empty() {
            untagged.push(def);
        }
        for tag in &def.tags {
            groups.entry(tag.as_str()).or_default().push(def);
        }
    }
    for (tag, blocks) in groups {
        egui::CollapsingHeader::new(tag).id_salt(("tag", tag)).default_open(true).show(ui, |ui| {
            for def in blocks {
                library_row(ui, registrar, def);
            }
        });
    }
    if !untagged.is_empty() {
        egui::CollapsingHeader::new("(no tags)").id_salt("untagged").default_open(true).show(ui, |ui| {
            for def in untagged {
                library_row(ui, registrar, def);
            }
        });
    }
}

fn library_row(ui: &mut egui::Ui, registrar: &mut Registrar, def: &BlockDef) {
    let unregulated = registrar.unregulated(def);
    let selected = registrar.selected.contains(&def.name);
    let row = ui.horizontal(|ui| {
        if let Some(top) = def.faces.get(4).filter(|top| !top.is_empty()) {
            texture_image(ui, registrar, top, 20.0, false);
        }
        let label = ui.selectable_label(selected, &def.display_name);
        if unregulated > 0 {
            ui.small(format!("{unregulated} unregulated")).on_hover_text("Colours not on any spline yet");
        } else {
            ui.small("regulated").on_hover_text("Every colour is on a spline");
        }
        label
    });
    let label = row.inner;
    if label.hovered() {
        registrar.hovered = Some(def.name.clone());
    }
    if label.clicked() {
        let adding = ui.input(|input| input.modifiers.command);
        if adding {
            if selected {
                registrar.selected.retain(|name| name != &def.name);
            } else {
                registrar.selected.push(def.name.clone());
            }
        } else {
            registrar.selected = vec![def.name.clone()];
        }
    }
    label.on_hover_text(&def.name);
}

/// A texture, `size` points square (made the first time it's drawn).
fn texture_image(ui: &mut egui::Ui, registrar: &mut Registrar, name: &str, size: f32, transformed: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let Some(texture) = registrar.textures.get_mut(name) else {
        return response;
    };
    if texture.problem.is_some() {
        ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 255));
        return response.on_hover_text(texture.problem.clone().unwrap_or_default());
    }
    let side = crate::render::textures::TEXTURE_SIZE as usize;
    // As the sub-palette's transform would make it: each pixel's colour mapped (made again
    // whenever the transform changes).
    let mapping = &registrar.mapping;
    let id = if transformed && !registrar.before && !mapping.is_empty() {
        let version = registrar.mapping_version;
        if texture.preview.as_ref().is_none_or(|(made_for, _)| *made_for != version) {
            let mut pixels = texture.pixels.clone();
            for pixel in pixels.chunks_exact_mut(4) {
                if let Some(to) = mapping.get(&[pixel[0], pixel[1], pixel[2]]) {
                    pixel[..3].copy_from_slice(to);
                }
            }
            let image = egui::ColorImage::from_rgba_unmultiplied([side, side], &pixels);
            match &mut texture.preview {
                Some((made_for, handle)) => {
                    handle.set(image, egui::TextureOptions::NEAREST);
                    *made_for = version;
                }
                None => texture.preview = Some((version, ui.ctx().load_texture(format!("registrar {name} transformed"), image, egui::TextureOptions::NEAREST))),
            }
        }
        texture.preview.as_ref().map(|(_, handle)| handle.id())
    } else {
        None
    };
    let id = id.unwrap_or_else(|| {
        texture.image.get_or_insert_with(|| ui.ctx().load_texture(format!("registrar {name}"), egui::ColorImage::from_rgba_unmultiplied([side, side], &texture.pixels), egui::TextureOptions::NEAREST)).id()
    });
    ui.painter().image(id, rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
    response
}

// ── The dock ──────────────────────────────────────────────────────────────

/// The open block: its name, tags, faces and own palette.
fn dock(ui: &mut egui::Ui, registrar: &mut Registrar, defs: &[&BlockDef]) {
    ui.horizontal(|ui| {
        ui.heading("Dock");
        // Loading a new asset into the library comes next.
        ui.add_enabled(false, egui::Button::new("Load a new asset…")).on_disabled_hover_text("Coming next: bring a PNG in and register it");
    });
    ui.separator();
    let Some(def) = registrar.selected.first().and_then(|name| defs.iter().find(|def| &def.name == name)).copied() else {
        ui.label("Click a block in the library to open it here.");
        return;
    };
    ui.heading(&def.display_name);
    ui.monospace(&def.name);
    ui.label(if def.tags.is_empty() { "No tags".to_string() } else { format!("Tags: {}", def.tags.join(", ")) });
    ui.add_space(8.0);
    // Its faces, as many as differ.
    let faces: [(&str, &[usize]); 3] = [("top", &[4]), ("sides", &[0, 1, 2, 3]), ("bottom", &[5])];
    ui.horizontal(|ui| {
        let mut done: Vec<&str> = Vec::new();
        for (label, which) in faces {
            let name = def.faces[which[0]].as_str();
            if name.is_empty() || done.contains(&name) {
                continue;
            }
            done.push(name);
            ui.vertical(|ui| {
                if texture_image(ui, registrar, name, 128.0, true).hovered() {
                    registrar.hovered = Some(def.name.clone());
                }
                let all_same = def.faces.iter().all(|face| face == &def.faces[0]);
                ui.small(format!("{} ({name})", if all_same { "all faces" } else { label }));
            });
        }
    });
    // The rest of the selection, smaller, so a transform can be seen across the whole group.
    let others: Vec<&BlockDef> = registrar.selected.iter().skip(1).filter_map(|name| defs.iter().find(|def| &def.name == name).copied()).collect();
    if !others.is_empty() {
        ui.add_space(8.0);
        ui.label("The rest of the selection");
        ui.horizontal_wrapped(|ui| {
            for other in others {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        for name in texture_names(other) {
                            if texture_image(ui, registrar, name, 48.0, true).hovered() {
                                registrar.hovered = Some(other.name.clone());
                            }
                        }
                    });
                    ui.small(&other.display_name);
                });
                ui.add_space(8.0);
            }
        });
    }
    ui.add_space(8.0);
    let colours = registrar.block_colours(def);
    ui.label(format!("Its palette: {} colours", colours.len()));
    let list: Vec<(Rgb, u32)> = sorted_by_lightness(colours);
    let event = swatches(ui, registrar, &list, None);
    registrar.take(ui, egui::Id::new("dock colours"), event);
}

// ── The palettes ──────────────────────────────────────────────────────────

/// What happened to a block of colour squares: the one under the mouse and whether it was clicked,
/// and where each square is (for dragging a box over them).
struct Swatch {
    hover: Option<(Rgb, bool)>,
    squares: Vec<(egui::Rect, Rgb)>,
}

/// A box being dragged over a block of colour squares (Shift + drag picks what it covers; Ctrl +
/// Shift + drag unpicks): which block, and where it started.
struct Band {
    block: egui::Id,
    from: egui::Pos2,
    unpicking: bool,
}

impl Registrar {
    /// The colour under the mouse becomes the focus; a click picks it, or unpicks it. Shift + drag
    /// draws a box: rows, columns or any patch of squares it covers are picked when let go (with
    /// Ctrl too, unpicked). `block` names this block of squares, so a box stays in the one it began in.
    fn take(&mut self, ui: &egui::Ui, block: egui::Id, event: Swatch) {
        let (pointer, pressed, released, shift, ctrl) = ui.input(|input| {
            (input.pointer.interact_pos(), input.pointer.primary_pressed(), input.pointer.primary_released(), input.modifiers.shift, input.modifiers.command)
        });
        if let Some((rgb, clicked)) = event.hover {
            self.focus = Some(rgb);
            if clicked && !shift && !self.picked.remove(&rgb) {
                self.picked.insert(rgb);
            }
            if pressed && shift && self.band.is_none()
                && let Some(from) = pointer
            {
                self.band = Some(Band { block, from, unpicking: ctrl });
            }
        }
        let Some(band) = self.band.as_ref().filter(|band| band.block == block) else { return };
        let Some(to) = pointer else { return };
        let area = egui::Rect::from_two_pos(band.from, to);
        let covered = event.squares.iter().filter(|(rect, _)| rect.intersects(area));
        let colour = if band.unpicking { Color32::WHITE } else { PICKED };
        let painter = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("registrar band")));
        painter.rect_stroke(area, 0.0, Stroke::new(1.0, colour), egui::StrokeKind::Outside);
        for (rect, _) in covered.clone() {
            painter.rect_stroke(*rect, 2.0, Stroke::new(1.0, colour), egui::StrokeKind::Inside);
        }
        if released {
            let unpicking = band.unpicking;
            let covered: Vec<Rgb> = covered.map(|&(_, rgb)| rgb).collect();
            for rgb in covered {
                if unpicking {
                    self.picked.remove(&rgb);
                } else {
                    self.picked.insert(rgb);
                }
            }
            self.band = None;
        }
    }

    /// The selection's colours, combined, with how many pixels each.
    fn selection_colours(&self, defs: &[&BlockDef]) -> BTreeMap<Rgb, u32> {
        let mut combined: BTreeMap<Rgb, u32> = BTreeMap::new();
        for def in defs.iter().filter(|def| self.selected.contains(&def.name)) {
            for (rgb, count) in self.block_colours(def) {
                *combined.entry(rgb).or_default() += count;
            }
        }
        combined
    }

    /// What the transform makes of each of the selection's colours, worked out again when the
    /// transform, the selection, the textures or the palette change (empty: no transform).
    fn update_mapping(&mut self, defs: &[&BlockDef]) {
        let wanted = (self.transform, self.selected.clone(), self.read_at, self.palette_changes);
        if self.mapped_for.as_ref() == Some(&wanted) {
            return;
        }
        self.mapping = if self.transform.is_active() {
            let colours: Vec<(Rgb, u32)> = self.selection_colours(defs).into_iter().collect();
            // The spline it goes onto, if any, and whether it still exists.
            let shades: Vec<Rgb> = self.transform.onto.and_then(|index| self.palette.splines.get(index)).map_or_else(Vec::new, |spline| spline.steps.iter().filter_map(|slot| self.palette.colours.get(slot)).map(|colour| colour.rgb).collect());
            self.transform.mapping(&colours, &shades)
        } else {
            BTreeMap::new()
        };
        self.mapped_for = Some(wanted);
        self.mapping_version += 1;
    }

    /// The picked colours' slots.
    fn picked_slots(&self) -> Vec<u32> {
        self.picked.iter().filter_map(|&rgb| self.palette.slot_of(rgb)).collect()
    }

    /// Write the palette file after a change (never over one that couldn't be read).
    fn save(&mut self, done: String) {
        if self.palette_problem.is_some() {
            return;
        }
        self.palette_changes += 1;
        self.message = match self.palette.save_to(&master_palette::palette_file()) {
            Ok(()) => done,
            Err(error) => format!("{done}, but the palette couldn't be saved: {error}"),
        };
    }
}

fn palettes(ui: &mut egui::Ui, registrar: &mut Registrar, defs: &[&BlockDef]) {
    ui.heading("Master palette");
    if let Some(problem) = &registrar.palette_problem {
        ui.colored_label(Color32::from_rgb(255, 80, 200), format!("The palette file couldn't be read, so nothing is written to it: {problem}"));
    }
    // Undo: the newest change to the files (applying, clearing out), put back.
    ui.horizontal(|ui| {
        let last = registrar.undo.last().cloned();
        let hover = last.clone().map_or("Nothing to undo".to_string(), |what| format!("Undo: {what}"));
        if ui.add_enabled(last.is_some(), egui::Button::new("Undo")).on_hover_text(&hover).on_disabled_hover_text(&hover).clicked() {
            registrar.undo_last();
        }
        if !registrar.undo.is_empty() {
            ui.small(format!("{} steps can be undone", registrar.undo.len()));
        }
    });
    if !registrar.message.is_empty() {
        ui.small(&registrar.message);
    }
    section(ui, "Focus", |ui| focus(ui, registrar, defs));

    // The colours the hovered block (or else the selection) uses light up.
    let names: Vec<String> = registrar.hovered.clone().map_or_else(|| registrar.selected.clone(), |name| vec![name]);
    let lit: BTreeSet<Rgb> = defs.iter().filter(|def| names.contains(&def.name)).flat_map(|def| registrar.block_colours(def).into_keys()).collect();
    let lit = if lit.is_empty() { None } else { Some(lit) };

    section(ui, "Colours", |ui| {
        // Which colours: the splines (in order, to arrange them), the rest, or all of it in the
        // master palette's order (the splines, then the colours not on one).
        ui.horizontal(|ui| {
            ui.selectable_value(&mut registrar.view, PaletteView::Everything, "Everything");
            ui.selectable_value(&mut registrar.view, PaletteView::Splines, "Splines");
            ui.selectable_value(&mut registrar.view, PaletteView::Loose, "Not on a spline");
        });
        let loose: Vec<(Rgb, u32)> = sorted_by_lightness(registrar.palette.colours.iter().filter(|&(&slot, _)| !registrar.palette.is_regulated(slot)).map(|(_, colour)| (colour.rgb, 0)));
        let on_splines: Vec<Rgb> = registrar.palette.splines.iter().flat_map(|spline| spline.steps.iter()).filter_map(|slot| registrar.palette.colours.get(slot)).map(|colour| colour.rgb).collect();
        let in_view: Vec<Rgb> = match registrar.view {
            PaletteView::Everything => on_splines.iter().copied().chain(loose.iter().map(|&(rgb, _)| rgb)).collect(),
            PaletteView::Splines => on_splines.clone(),
            PaletteView::Loose => loose.iter().map(|&(rgb, _)| rgb).collect(),
        };

        // Picking, and making a spline of what's picked.
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} picked", registrar.picked.len())).on_hover_text("Click colours in any palette to pick them; click again to unpick.\nShift + drag a box to pick rows, columns or any patch; Ctrl + Shift + drag to unpick.");
            if ui.add_enabled(!in_view.is_empty(), egui::Button::new("Pick all in view")).clicked() {
                registrar.picked.extend(in_view.iter().copied());
            }
            if ui.add_enabled(!registrar.picked.is_empty(), egui::Button::new("Deselect all")).on_hover_text("Or press Escape").clicked() {
                registrar.picked.clear();
            }
            let editable = registrar.palette_problem.is_none();
            let label = if registrar.picked.is_empty() { "New spline" } else { "New spline of the picked" };
            if ui.add_enabled(editable, egui::Button::new(label)).on_hover_text("A hue's ramp of shades, darkest first. Colours already on a spline stay on theirs").clicked() {
                let name = format!("spline {}", registrar.palette.splines.len() + 1);
                let slots = registrar.picked_slots();
                registrar.palette.new_spline(&name, &slots);
                registrar.picked.clear();
                registrar.save(format!("Made {name}."));
            }
        });

        if registrar.view != PaletteView::Loose {
            splines(ui, registrar, lit.as_ref());
        }
        if registrar.view != PaletteView::Splines {
            ui.add_space(6.0);
            ui.label(format!("Not on a spline: {} colours", loose.len()));
            let event = swatches(ui, registrar, &loose, lit.as_ref());
            registrar.take(ui, egui::Id::new("loose colours"), event);
            clear_unused(ui, registrar);
        }
    });
    section(ui, "Sub-palette", |ui| {
        let chosen: Vec<&&BlockDef> = defs.iter().filter(|def| registrar.selected.contains(&def.name)).collect();
        if chosen.is_empty() {
            ui.label("Select blocks in the library to see their combined palette.");
            return;
        }
        let mut combined: BTreeMap<Rgb, u32> = BTreeMap::new();
        for def in &chosen {
            for (rgb, count) in registrar.block_colours(def) {
                *combined.entry(rgb).or_default() += count;
            }
        }
        let names: Vec<&str> = chosen.iter().map(|def| def.display_name.as_str()).collect();
        ui.label(format!("{}: {} colours", names.join(", "), combined.len()));
        let list = sorted_by_lightness(combined);
        let event = swatches(ui, registrar, &list, None);
        registrar.take(ui, egui::Id::new("sub-palette colours"), event);
        ui.horizontal(|ui| {
            if ui.button("Pick all of these").clicked() {
                registrar.picked.extend(list.iter().map(|&(rgb, _)| rgb));
            }
        });
        egui::CollapsingHeader::new("Transform").default_open(true).show(ui, |ui| transform_controls(ui, registrar, &list, defs));
    });
}

/// A part of the palettes column that folds away, open to start with [David: "pick what's in view
/// while working"].
fn section(ui: &mut egui::Ui, name: &str, body: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(name).id_salt(("registrar section", name)).default_open(true).show(ui, body);
}

/// The sub-palette's transform: crushing to fewer new colours or onto a spline's shades, and
/// shifting hue, saturation and lightness, all previewed on the blocks in the dock. Nothing is
/// written to the block files yet; the crushed colours can be made into a spline.
fn transform_controls(ui: &mut egui::Ui, registrar: &mut Registrar, list: &[(Rgb, u32)], defs: &[&BlockDef]) {
    ui.small("Previewed on the blocks in the dock; nothing is written to the block files yet.");
    let splines: Vec<String> = registrar.palette.splines.iter().map(|spline| spline.name.clone()).collect();
    let transform = &mut registrar.transform;
    // Onto: new colours (crushing), or a spline's shades.
    if transform.onto.is_some_and(|index| index >= splines.len()) {
        transform.onto = None;
    }
    let onto_name = transform.onto.map_or("new colours (crush)".to_string(), |index| format!("the shades of {}", splines[index]));
    ui.horizontal(|ui| {
        ui.label("onto:");
        egui::ComboBox::from_id_salt("transform onto").selected_text(onto_name).show_ui(ui, |ui| {
            ui.selectable_value(&mut transform.onto, None, "new colours (crush)");
            for (index, name) in splines.iter().enumerate() {
                ui.selectable_value(&mut transform.onto, Some(index), format!("the shades of {name}"));
            }
        });
    })
    .response
    .on_hover_text("Crushing makes new colours (each group's average). Onto a spline, every colour becomes one of its shades: crushing to the palette");
    let most = (list.len() as u32).saturating_sub(1).clamp(2, 64);
    ui.add_enabled(transform.onto.is_none(), crate::debug::Slider::new(&mut transform.crush, 0..=most).text("crush to (colours; 0 = off)"))
        .on_hover_text("Lined up darkest first and cut into this many groups; each group becomes its pixels' average colour");
    ui.add_enabled_ui(transform.onto.is_some(), |ui| {
        ui.horizontal(|ui| {
            ui.label("each to:");
            ui.radio_value(&mut transform.snap, Snap::ByRank, "the shade of its rank").on_hover_text("Darkest to darkest and so on: keeps the contrast");
            ui.radio_value(&mut transform.snap, Snap::Nearest, "the nearest shade").on_hover_text("The shade that looks closest");
        });
    });
    ui.add_enabled_ui(transform.crush > 0 || (transform.onto.is_some() && transform.snap == Snap::ByRank), |ui| {
        ui.horizontal(|ui| {
            ui.label("groups:");
            ui.radio_value(&mut transform.spread, Spread::EqualValues, "equal slices of lightness");
            ui.radio_value(&mut transform.spread, Spread::EqualShares, "equal shares of the pixels");
        });
    });
    ui.add(crate::debug::Slider::new(&mut transform.hue, -180.0..=180.0).text("hue").suffix("°"));
    let mut percent = |value: &mut f32, name: &str| {
        let mut shown = *value * 100.0;
        if ui.add(crate::debug::Slider::new(&mut shown, -100.0..=100.0).text(name).suffix("%")).changed() {
            *value = shown / 100.0;
        }
    };
    percent(&mut transform.saturation, "saturation");
    percent(&mut transform.lightness, "lightness");
    if transform.onto.is_some() {
        ui.small("Onto a spline, hue, saturation and lightness move the colours before they're put on its shades.");
    }
    ui.horizontal(|ui| {
        if ui.add_enabled(registrar.transform.is_active(), egui::Button::new("Reset")).clicked() {
            let kept = registrar.transform;
            registrar.transform = Transform { spread: kept.spread, snap: kept.snap, ..Transform::default() };
        }
        ui.checkbox(&mut registrar.before, "show the blocks before").on_hover_text("The dock shows the blocks as they are, not as the transform would make them");
    });
    if registrar.mapping.is_empty() {
        return;
    }
    // Writing it into the files: which textures, and any other block that uses one of them (it
    // changes too).
    let names: BTreeSet<&str> = defs.iter().filter(|def| registrar.selected.contains(&def.name)).flat_map(|def| texture_names(def)).collect();
    let others: Vec<&str> = defs.iter().filter(|def| !registrar.selected.contains(&def.name) && texture_names(def).iter().any(|name| names.contains(name))).map(|def| def.display_name.as_str()).collect();
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(registrar.palette_problem.is_none(), egui::Button::new("Apply to the files"))
            .on_hover_text("Writes these colours into the textures; copies are kept first, for Undo")
            .clicked()
        {
            registrar.apply(defs);
        }
        let files: Vec<String> = names.iter().map(|name| format!("{name}.png")).collect();
        ui.small(format!("writes {}", files.join(", ")));
    });
    if !others.is_empty() {
        ui.colored_label(Color32::from_rgb(255, 200, 80), format!("These textures are shared: {} changes too.", others.join(", ")));
    }
    let mut after: BTreeMap<Rgb, u32> = BTreeMap::new();
    for &(rgb, count) in list {
        *after.entry(registrar.mapping.get(&rgb).copied().unwrap_or(rgb)).or_default() += count;
    }
    ui.label(format!("After: {} colours", after.len()));
    let after = sorted_by_lightness(after);
    let event = swatches(ui, registrar, &after, None);
    registrar.take(ui, egui::Id::new("transformed colours"), event);
    // The crushed colours as a new spline: they join the palette, darkest first.
    let new = registrar.transform.onto.is_none();
    if ui
        .add_enabled(new && registrar.palette_problem.is_none(), egui::Button::new("New spline from these"))
        .on_hover_text("These colours join the master palette as a spline, darkest first. The blocks keep their own colours until the crush is applied to the files")
        .on_disabled_hover_text("These are already a spline's shades")
        .clicked()
    {
        let colours: Vec<Rgb> = after.iter().map(|&(rgb, _)| rgb).collect();
        registrar.palette.take_in(colours.iter().copied());
        let slots: Vec<u32> = colours.iter().filter_map(|&rgb| registrar.palette.slot_of(rgb)).collect();
        let open = registrar.selected.first().cloned().unwrap_or_default();
        let name = format!("{} crushed", open.rsplit(':').next().unwrap_or("spline"));
        let index = registrar.palette.new_spline(&name, &slots);
        let made = registrar.palette.splines[index].steps.len();
        let note = if made < slots.len() { format!(" ({} were already on another spline and stayed there)", slots.len() - made) } else { String::new() };
        registrar.save(format!("Made {name}, {made} shades{note}."));
    }
}



/// Which of the master palette's colours are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PaletteView {
    /// The splines in order, then the colours not on one: the master palette's order.
    #[default]
    Everything,
    /// The splines, one to a row, with their controls and curves, to arrange them.
    Splines,
    /// The colours not on a spline yet.
    Loose,
}

/// The splines, in the master palette's order, one to a row. In the Splines view each has its
/// controls (move up or down, rename, add or take off the picked, delete) and its curves; in
/// Everything, just its name and shades.
fn splines(ui: &mut egui::Ui, registrar: &mut Registrar, lit: Option<&BTreeSet<Rgb>>) {
    let full = registrar.view == PaletteView::Splines;
    let mut action = None;
    if registrar.palette.splines.is_empty() {
        ui.small("No splines yet.");
    }
    let count = registrar.palette.splines.len();
    for index in 0..count {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let picked = !registrar.picked.is_empty();
            let spline = &mut registrar.palette.splines[index];
            if !full {
                ui.label(&spline.name);
                ui.small(format!("{} shades", spline.steps.len()));
                return;
            }
            if ui.add_enabled(index > 0, egui::Button::new("▲").small()).on_hover_text("Earlier in the master palette").clicked() {
                action = Some(SplineAction::Move(index, -1));
            }
            if ui.add_enabled(index + 1 < count, egui::Button::new("▼").small()).on_hover_text("Later in the master palette").clicked() {
                action = Some(SplineAction::Move(index, 1));
            }
            let name = ui.add(egui::TextEdit::singleline(&mut spline.name).desired_width(120.0));
            if name.lost_focus() {
                action = Some(SplineAction::Renamed);
            }
            ui.small(format!("{} shades", spline.steps.len()));
            if ui.add_enabled(picked, egui::Button::new("Add the picked")).on_hover_text("Each goes in by lightness, between the shades darker and lighter").clicked() {
                action = Some(SplineAction::Add(index));
            }
            if ui.add_enabled(picked, egui::Button::new("Take off the picked")).on_hover_text("Picked colours on this spline go back to not being on one").clicked() {
                action = Some(SplineAction::TakeOff(index));
            }
            if ui.button("Delete").on_hover_text("The spline goes; its colours stay in the palette, not on a spline").clicked() {
                action = Some(SplineAction::Delete(index));
            }
        });
        let steps: Vec<(Rgb, u32)> = registrar.palette.splines[index].steps.iter().filter_map(|slot| registrar.palette.colours.get(slot)).map(|colour| (colour.rgb, 0)).collect();
        if full {
            hsl_curves(ui, &steps);
        }
        let event = swatches(ui, registrar, &steps, lit);
        registrar.take(ui, egui::Id::new(("spline colours", index)), event);
    }
    match action {
        Some(SplineAction::Renamed) => registrar.save("Renamed.".to_string()),
        Some(SplineAction::Move(index, by)) => {
            registrar.palette.move_spline(index, by);
            registrar.save("Moved.".to_string());
        }
        Some(SplineAction::Add(index)) => {
            let slots = registrar.picked_slots();
            let skipped = registrar.palette.add_to_spline(index, &slots);
            registrar.picked.clear();
            let name = registrar.palette.splines[index].name.clone();
            let note = if skipped.is_empty() { String::new() } else { format!(" ({} already on a spline stayed there)", skipped.len()) };
            registrar.save(format!("Added {} to {name}{note}.", slots.len() - skipped.len()));
        }
        Some(SplineAction::TakeOff(index)) => {
            let on: Vec<u32> = registrar.picked_slots().into_iter().filter(|slot| registrar.palette.splines[index].steps.contains(slot)).collect();
            registrar.palette.take_off_splines(&on);
            registrar.picked.clear();
            registrar.save(format!("Took {} off.", on.len()));
        }
        Some(SplineAction::Delete(index)) => {
            let gone = registrar.palette.splines.remove(index);
            registrar.save(format!("Deleted {}; its {} colours are back among those not on a spline.", gone.name, gone.steps.len()));
        }
        None => {}
    }
}

/// "Clear out unused colours": the colours on no spline that no block uses any more, shown first,
/// then taken out of the palette on a yes (undoable; their numbers are retired).
fn clear_unused(ui: &mut egui::Ui, registrar: &mut Registrar) {
    let unused = registrar.unused();
    if !registrar.clearing {
        let label = format!("Clear out unused colours ({})", unused.len());
        if ui.add_enabled(!unused.is_empty() && registrar.palette_problem.is_none(), egui::Button::new(label)).on_hover_text("Colours on no spline that no block uses any more; shown before anything goes").clicked() {
            registrar.clearing = true;
        }
        return;
    }
    ui.label(format!("These {} colours are on no spline and no block uses them:", unused.len()));
    let colours: Vec<(Rgb, u32)> = sorted_by_lightness(unused.iter().filter_map(|slot| registrar.palette.colours.get(slot)).map(|colour| (colour.rgb, 0)));
    let event = swatches(ui, registrar, &colours, None);
    registrar.take(ui, egui::Id::new("unused colours"), event);
    ui.horizontal(|ui| {
        if ui.add_enabled(!unused.is_empty(), egui::Button::new(format!("Take these {} out", unused.len()))).clicked() {
            registrar.clear_unused();
            registrar.clearing = false;
        }
        if ui.button("Cancel").clicked() {
            registrar.clearing = false;
        }
    });
}
enum SplineAction {
    Renamed,
    Move(usize, isize),
    Add(usize),
    TakeOff(usize),
    Delete(usize),
}

/// The focus: the colour last under the mouse, large, with its slot, hex, HSL, spline, and which
/// blocks use it.
fn focus(ui: &mut egui::Ui, registrar: &Registrar, defs: &[&BlockDef]) {
    let Some(rgb) = registrar.focus else {
        ui.small("Hover over a colour to see it here.");
        return;
    };
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        ui.vertical(|ui| {
            let [h, s, l] = super::palette::to_hsl(rgb);
            let slot = registrar.palette.slot_of(rgb);
            ui.monospace(format!("{}   {}", master_palette::to_hex(rgb), slot.map_or("not in the palette".to_string(), |slot| format!("slot {slot}"))));
            ui.monospace(format!("H {h:>3.0}°   S {:>3.0}%   L {:>3.0}%", s * 100.0, l * 100.0));
            let spline = slot.and_then(|slot| registrar.palette.spline_of(slot)).map_or("not on a spline".to_string(), |spline| format!("on {}", spline.name));
            ui.label(spline);
            let users: Vec<&str> = defs.iter().filter(|def| registrar.block_colours(def).contains_key(&rgb)).map(|def| def.display_name.as_str()).collect();
            ui.small(if users.is_empty() { "used by no block".to_string() } else { format!("used by {}", users.join(", ")) });
        });
    });
}

/// A spline's hue, saturation and lightness, step by step, as three lines (hue across 0-360°, the
/// others 0-100%): its curves, to see how it ramps.
fn hsl_curves(ui: &mut egui::Ui, steps: &[(Rgb, u32)]) {
    if steps.len() < 2 {
        return;
    }
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 48.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, Color32::from_gray(24));
    let hsl: Vec<[f32; 3]> = steps.iter().map(|&(rgb, _)| super::palette::to_hsl(rgb)).collect();
    let lines = [(0, 360.0, Color32::from_rgb(255, 170, 60)), (1, 1.0, Color32::from_rgb(90, 200, 255)), (2, 1.0, Color32::WHITE)];
    for (channel, top, colour) in lines {
        let points: Vec<egui::Pos2> = hsl
            .iter()
            .enumerate()
            .map(|(k, value)| {
                let x = rect.left() + 4.0 + (rect.width() - 8.0) * k as f32 / (hsl.len() - 1) as f32;
                let y = rect.bottom() - 4.0 - (rect.height() - 8.0) * value[channel] / top;
                egui::pos2(x, y)
            })
            .collect();
        painter.add(egui::Shape::line(points, Stroke::new(1.5, colour)));
    }
    painter.text(rect.left_top() + egui::vec2(4.0, 2.0), egui::Align2::LEFT_TOP, "hue", egui::FontId::proportional(10.0), Color32::from_rgb(255, 170, 60));
    painter.text(rect.left_top() + egui::vec2(28.0, 2.0), egui::Align2::LEFT_TOP, "saturation", egui::FontId::proportional(10.0), Color32::from_rgb(90, 200, 255));
    painter.text(rect.left_top() + egui::vec2(84.0, 2.0), egui::Align2::LEFT_TOP, "lightness", egui::FontId::proportional(10.0), Color32::WHITE);
}

/// Colours darkest first (by HSL lightness, then hue).
fn sorted_by_lightness(colours: impl IntoIterator<Item = (Rgb, u32)>) -> Vec<(Rgb, u32)> {
    let mut list: Vec<(Rgb, u32)> = colours.into_iter().collect();
    list.sort_by(|a, b| {
        let (ha, hb) = (super::palette::to_hsl(a.0), super::palette::to_hsl(b.0));
        ha[2].total_cmp(&hb[2]).then(ha[0].total_cmp(&hb[0]))
    });
    list
}

/// A row of colour squares, wrapping. With `lit`, the ones not in it are dimmed; picked ones have a
/// pink edge. Returns the one under the mouse and whether it was clicked, and where each square is.
fn swatches(ui: &mut egui::Ui, registrar: &Registrar, colours: &[(Rgb, u32)], lit: Option<&BTreeSet<Rgb>>) -> Swatch {
    // Squares between these sizes, sized so a row fills the panel's width exactly [David: "a tad
    // smaller", "fit to size within a range"].
    const SMALLEST: f32 = 10.0;
    const LARGEST: f32 = 16.0;
    const GAP: f32 = 2.0;
    let across = ((ui.available_width() + GAP) / (LARGEST + GAP)).ceil().max(1.0);
    let size = ((ui.available_width() + GAP) / across - GAP).clamp(SMALLEST, LARGEST);
    let mut event = Swatch { hover: None, squares: Vec::with_capacity(colours.len()) };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(GAP, GAP);
        for &(rgb, count) in colours {
            let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
            event.squares.push((rect, rgb));
            let dim = lit.is_some_and(|lit| !lit.contains(&rgb));
            let colour = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            ui.painter().rect_filled(rect, 2.0, if dim { colour.gamma_multiply(0.15) } else { colour });
            if registrar.picked.contains(&rgb) {
                ui.painter().rect_stroke(rect, 2.0, Stroke::new(2.0, PICKED), egui::StrokeKind::Inside);
            } else if lit.is_some() && !dim {
                ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Inside);
            }
            if response.hovered() {
                event.hover = Some((rgb, response.clicked()));
                if count > 0 {
                    response.on_hover_text(format!("{count} pixels"));
                }
            }
        }
    });
    event
}

/// The edge of a picked colour.
const PICKED: Color32 = Color32::from_rgb(255, 60, 200);

// ── Transforms: crushing and HSL, previewed ───────────────────────────────

/// How the sub-palette is changed, previewed on the blocks in the dock before anything is written.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Transform {
    /// Crush to this many colours (0: not crushed).
    crush: u32,
    /// How the colours are shared out when crushed (or put onto a spline by rank), as in the Graphics
    /// Kit's palettes: an equal slice of lightness each, or an equal share of the pixels.
    spread: Spread,
    /// Added to hue (degrees), saturation and lightness (0-1).
    hue: f32,
    saturation: f32,
    lightness: f32,
    /// Onto a spline (its place in the list) instead of crushing: every colour becomes one of its
    /// shades, so the result is regulated.
    onto: Option<usize>,
    snap: Snap,
}

/// How colours are put onto a spline's shades.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snap {
    /// Darkest to darkest and so on, cut as crushing cuts them: keeps the texture's contrast.
    ByRank,
    /// Each to the shade that looks closest.
    Nearest,
}

impl Default for Transform {
    fn default() -> Self {
        Self { crush: 0, spread: Spread::EqualShares, hue: 0.0, saturation: 0.0, lightness: 0.0, onto: None, snap: Snap::ByRank }
    }
}

impl Transform {
    fn is_active(&self) -> bool {
        *self != Self { spread: self.spread, snap: self.snap, ..Self::default() }
    }

    /// What each of these colours (with their pixel counts) becomes. Onto a spline (`shades`, darkest
    /// first): shifted, then each put on a shade. Otherwise: crushed, then shifted.
    fn mapping(&self, colours: &[(Rgb, u32)], shades: &[Rgb]) -> BTreeMap<Rgb, Rgb> {
        let shift = |rgb: Rgb| shifted(rgb, self.hue, self.saturation, self.lightness);
        if self.onto.is_some() {
            if shades.is_empty() {
                return BTreeMap::new();
            }
            let moved: Vec<(Rgb, u32)> = colours.iter().map(|&(rgb, n)| (shift(rgb), n)).collect();
            let snapped = onto(&moved, shades, self.snap, self.spread);
            return colours.iter().zip(&moved).map(|(&(rgb, _), &(to, _))| (rgb, snapped[&to])).collect();
        }
        let crushed = crushed(colours, self.crush, self.spread);
        colours.iter().map(|&(rgb, _)| (rgb, shift(crushed.get(&rgb).copied().unwrap_or(rgb)))).collect()
    }
}

/// Colours lined up darkest first and cut into `count` groups, each an equal slice of lightness or
/// an equal share of the pixels (`spread`): each colour (darkest first) with its group, 0 darkest.
fn groups(colours: &[(Rgb, u32)], count: usize, spread: Spread) -> Vec<(Rgb, u32, usize)> {
    let sorted = sorted_by_lightness(colours.iter().copied());
    if sorted.is_empty() || count == 0 {
        return Vec::new();
    }
    let total: f32 = sorted.iter().map(|&(_, n)| n.max(1) as f32).sum();
    let (darkest, lightest) = (master_palette::lightness(sorted[0].0), master_palette::lightness(sorted[sorted.len() - 1].0));
    let mut so_far = 0.0;
    sorted
        .iter()
        .map(|&(rgb, n)| {
            let weight = n.max(1) as f32;
            let at = match spread {
                // Where the middle of its pixels falls along all the pixels.
                Spread::EqualShares => (so_far + weight / 2.0) / total,
                Spread::EqualValues => (master_palette::lightness(rgb) - darkest) / (lightest - darkest).max(1e-6),
            };
            so_far += weight;
            (rgb, n, ((at * count as f32) as usize).min(count - 1))
        })
        .collect()
}

/// Crush colours to `count` (0, or at least as many as there are: unchanged): cut into groups
/// (`groups`), each group becomes its pixels' average colour.
fn crushed(colours: &[(Rgb, u32)], count: u32, spread: Spread) -> BTreeMap<Rgb, Rgb> {
    if count == 0 || colours.len() <= count as usize {
        return colours.iter().map(|&(rgb, _)| (rgb, rgb)).collect();
    }
    let grouped = groups(colours, count as usize, spread);
    let mut sums = vec![([0.0f32; 3], 0.0f32); count as usize];
    for &(rgb, n, g) in &grouped {
        let n = n.max(1) as f32;
        for k in 0..3 {
            sums[g].0[k] += rgb[k] as f32 * n;
        }
        sums[g].1 += n;
    }
    grouped
        .iter()
        .map(|&(rgb, _, g)| {
            let (sum, n) = sums[g];
            (rgb, sum.map(|c| (c / n).round() as u8))
        })
        .collect()
}

/// Colours put onto a spline's `shades` (darkest first): by rank, cut into as many groups as there
/// are shades and the darkest group to the darkest shade and so on; or each to the nearest shade by
/// eye (OKLab, which measures colour differences about as the eye sees them).
fn onto(colours: &[(Rgb, u32)], shades: &[Rgb], snap: Snap, spread: Spread) -> BTreeMap<Rgb, Rgb> {
    match snap {
        Snap::ByRank => groups(colours, shades.len(), spread).into_iter().map(|(rgb, _, g)| (rgb, shades[g])).collect(),
        Snap::Nearest => {
            let lab: Vec<[f32; 3]> = shades.iter().map(|&shade| oklab(shade)).collect();
            colours
                .iter()
                .map(|&(rgb, _)| {
                    let here = oklab(rgb);
                    let distance = |other: &[f32; 3]| (0..3).map(|k| (here[k] - other[k]).powi(2)).sum::<f32>();
                    let nearest = (0..shades.len()).min_by(|&a, &b| distance(&lab[a]).total_cmp(&distance(&lab[b]))).unwrap_or(0);
                    (rgb, shades[nearest])
                })
                .collect()
        }
    }
}

/// A colour in OKLab: lightness, then two colour axes, spaced about as the eye tells colours apart
/// (Björn Ottosson's formulas). Used only underneath, to find nearest colours; the Lab speaks HSL.
fn oklab(rgb: Rgb) -> [f32; 3] {
    let linear = |c: u8| {
        let c = c as f32 / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    let [r, g, b] = rgb.map(linear);
    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// A colour with `hue` degrees added (round the wheel) and `saturation` and `lightness` added
/// (kept within 0-1).
fn shifted(rgb: Rgb, hue: f32, saturation: f32, lightness: f32) -> Rgb {
    if hue == 0.0 && saturation == 0.0 && lightness == 0.0 {
        return rgb;
    }
    let [h, s, l] = super::palette::to_hsl(rgb);
    super::palette::from_hsl([(h + hue).rem_euclid(360.0), (s + saturation).clamp(0.0, 1.0), (l + lightness).clamp(0.0, 1.0)])
}

// ── Undo: copies of the files a change touches, made before it ────────────

/// `registrar undo/` beside the assets: one folder per change, newest last.
fn undo_folder() -> PathBuf {
    crate::data::assets_folder().parent().map_or_else(|| PathBuf::from("registrar undo"), |project| project.join("registrar undo"))
}

/// Before a change: copy `files` (those that exist) into a new step in `folder`, with what the
/// change is. Each step holds `what.txt`, `files.txt` (each copy's number and where it came from)
/// and the copies.
fn save_undo_step(folder: &Path, what: &str, files: &[PathBuf]) -> Result<(), String> {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_millis());
    let step = folder.join(format!("{stamp:015}"));
    std::fs::create_dir_all(&step).map_err(|error| error.to_string())?;
    let mut list = String::new();
    for (k, file) in files.iter().enumerate().filter(|(_, file)| file.exists()) {
        std::fs::copy(file, step.join(format!("{k}.copy"))).map_err(|error| format!("couldn't copy {}: {error}", file.display()))?;
        list += &format!("{k}\t{}\n", file.display());
    }
    std::fs::write(step.join("files.txt"), list).map_err(|error| error.to_string())?;
    std::fs::write(step.join("what.txt"), what).map_err(|error| error.to_string())
}

/// The steps in `folder`, oldest first: each folder and what its change was.
fn undo_steps(folder: &Path) -> Vec<(PathBuf, String)> {
    let mut steps: Vec<PathBuf> = std::fs::read_dir(folder).into_iter().flatten().flatten().map(|entry| entry.path()).filter(|path| path.join("files.txt").exists()).collect();
    steps.sort();
    steps.into_iter().map(|step| (step.clone(), std::fs::read_to_string(step.join("what.txt")).unwrap_or_default())).collect()
}

/// Put back the newest step's files and remove the step. Returns what was undone.
fn undo_last(folder: &Path) -> Result<String, String> {
    let (step, what) = undo_steps(folder).pop().ok_or("nothing to undo")?;
    let list = std::fs::read_to_string(step.join("files.txt")).map_err(|error| error.to_string())?;
    for line in list.lines() {
        let Some((k, path)) = line.split_once('\t') else { continue };
        std::fs::copy(step.join(format!("{k}.copy")), path).map_err(|error| format!("couldn't put back {path}: {error}"))?;
    }
    std::fs::remove_dir_all(&step).map_err(|error| error.to_string())?;
    Ok(what)
}

/// Write 32 x 32 RGBA pixels as a PNG.
fn write_png(path: &Path, pixels: &[u8]) -> Result<(), String> {
    let side = crate::render::textures::TEXTURE_SIZE;
    let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), side, side);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer.write_image_data(pixels).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crushing_groups_colours_darkest_first_into_their_average() {
        let colours = [([0, 0, 0], 1), ([20, 20, 20], 1), ([200, 200, 200], 1), ([220, 220, 220], 1)];
        let two = crushed(&colours, 2, Spread::EqualShares);
        assert_eq!((two[&[0, 0, 0]], two[&[20, 20, 20]]), ([10, 10, 10], [10, 10, 10]));
        assert_eq!(two[&[220, 220, 220]], [210, 210, 210]);
        // By pixels: a colour covering most of them gets most of the groups to itself.
        let heavy = [([0, 0, 0], 10), ([20, 20, 20], 1), ([200, 200, 200], 1)];
        assert_eq!(crushed(&heavy, 2, Spread::EqualShares)[&[20, 20, 20]], [110, 110, 110]);
        // By lightness: the two darks share a group, averaged by their pixels ((0 × 10 + 20) / 11).
        assert_eq!(crushed(&heavy, 2, Spread::EqualValues)[&[20, 20, 20]], [2, 2, 2]);
        // Off, or no fewer than there are: unchanged.
        assert!(crushed(&colours, 0, Spread::EqualShares).iter().all(|(a, b)| a == b));
        assert!(crushed(&colours, 4, Spread::EqualShares).iter().all(|(a, b)| a == b));
    }


    #[test]
    fn colours_go_onto_a_spline_by_rank_or_by_the_nearest_shade() {
        let shades = [[40, 20, 10], [120, 60, 30], [220, 180, 140]];
        // Four greys, darkest first, onto three shades by rank: the contrast kept, every result a shade.
        let greys = [([10, 10, 10], 1), ([30, 30, 30], 1), ([90, 90, 90], 1), ([250, 250, 250], 1)];
        let ranked = onto(&greys, &shades, Snap::ByRank, Spread::EqualShares);
        assert_eq!(ranked[&[10, 10, 10]], shades[0]);
        assert_eq!(ranked[&[250, 250, 250]], shades[2]);
        assert!(ranked.values().all(|to| shades.contains(to)));
        // By eye: a near-white goes to the lightest shade, a dark brown to the darkest.
        let nearest = onto(&[([230, 190, 150], 1), ([45, 25, 12], 1)], &shades, Snap::Nearest, Spread::EqualShares);
        assert_eq!((nearest[&[230, 190, 150]], nearest[&[45, 25, 12]]), (shades[2], shades[0]));
        // Through the transform: shifted first, then put on the shades.
        let transform = Transform { onto: Some(0), snap: Snap::Nearest, lightness: 1.0, ..Transform::default() };
        assert_eq!(transform.mapping(&[([45, 25, 12], 1)], &shades)[&[45, 25, 12]], shades[2]);
    }

    #[test]
    fn undo_puts_back_the_files_a_change_touched_newest_first() {
        let folder = std::env::temp_dir().join(format!("incus registrar undo test {}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let file = folder.join("gabbro.png");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(&file, "original").unwrap();
        let undo = folder.join("undo");
        save_undo_step(&undo, "first", std::slice::from_ref(&file)).unwrap();
        std::fs::write(&file, "crushed").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        save_undo_step(&undo, "second", std::slice::from_ref(&file)).unwrap();
        std::fs::write(&file, "shifted").unwrap();
        assert_eq!(undo_last(&undo).unwrap(), "second");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "crushed");
        assert_eq!(undo_last(&undo).unwrap(), "first");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "original");
        assert!(undo_last(&undo).is_err());
        let _ = std::fs::remove_dir_all(&folder);
    }
    #[test]
    fn an_hsl_shift_goes_round_the_wheel_and_stays_in_range() {
        assert_eq!(shifted([255, 0, 0], 120.0, 0.0, 0.0), [0, 255, 0]);
        assert_eq!(shifted([255, 0, 0], -120.0, 0.0, 0.0), [0, 0, 255]);
        assert_eq!(shifted([200, 100, 50], 0.0, 0.0, 1.0), [255, 255, 255]);
        assert_eq!(shifted([200, 100, 50], 0.0, -1.0, 0.0)[0], shifted([200, 100, 50], 0.0, -1.0, 0.0)[2], "no saturation: grey");
    }
}

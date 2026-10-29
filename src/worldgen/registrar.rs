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
    mapped_for: Option<(Transform, Vec<String>, Option<u64>)>,
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
    /// first opening). Colours not in the palette yet join it, unregulated, and the file is saved.
    fn refresh(&mut self, registry: &BlockRegistry) {
        if self.read_at == Some(registry.revision()) {
            return;
        }
        let first = self.read_at.is_none();
        self.read_at = Some(registry.revision());
        if first {
            match MasterPalette::load_from(&master_palette::palette_file()) {
                Ok(palette) => self.palette = palette,
                Err(problem) => self.palette_problem = Some(problem),
            }
        }
        self.textures.clear();
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
        if self.palette_problem.is_some() {
            return;
        }
        let added = self.palette.take_in(used);
        if !added.is_empty() {
            self.message = match self.palette.save_to(&master_palette::palette_file()) {
                Ok(()) => format!("{} new colours joined the master palette, not on a spline yet.", added.len()),
                Err(error) => format!("{} new colours, but the palette couldn't be saved: {error}", added.len()),
            };
        }
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
    /// transform, the selection or the textures change (empty: no transform).
    fn update_mapping(&mut self, defs: &[&BlockDef]) {
        let wanted = (self.transform, self.selected.clone(), self.read_at);
        if self.mapped_for.as_ref() == Some(&wanted) {
            return;
        }
        self.mapping = if self.transform.is_active() {
            let colours: Vec<(Rgb, u32)> = self.selection_colours(defs).into_iter().collect();
            self.transform.mapping(&colours)
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
    focus(ui, registrar, defs);
    ui.separator();

    // The colours the hovered block (or else the selection) uses light up.
    let names: Vec<String> = registrar.hovered.clone().map_or_else(|| registrar.selected.clone(), |name| vec![name]);
    let lit: BTreeSet<Rgb> = defs.iter().filter(|def| names.contains(&def.name)).flat_map(|def| registrar.block_colours(def).into_keys()).collect();
    let lit = if lit.is_empty() { None } else { Some(lit) };

    // Picking, and making a spline of what's picked.
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{} picked", registrar.picked.len())).on_hover_text("Click colours in any palette to pick them; click again to unpick.
Shift + drag a box to pick rows, columns or any patch; Ctrl + Shift + drag to unpick.");
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

    let mut action = None;
    if registrar.palette.splines.is_empty() {
        ui.small("No splines yet.");
    }
    for index in 0..registrar.palette.splines.len() {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let spline = &mut registrar.palette.splines[index];
            let name = ui.add(egui::TextEdit::singleline(&mut spline.name).desired_width(140.0));
            if name.lost_focus() {
                action = Some(SplineAction::Renamed);
            }
            ui.small(format!("{} shades", spline.steps.len()));
            let picked = !registrar.picked.is_empty();
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
        hsl_curves(ui, &steps);
        let event = swatches(ui, registrar, &steps, lit.as_ref());
        registrar.take(ui, egui::Id::new(("spline colours", index)), event);
    }
    match action {
        Some(SplineAction::Renamed) => registrar.save("Renamed.".to_string()),
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

    ui.add_space(6.0);
    let loose: Vec<(Rgb, u32)> = registrar.palette.colours.iter().filter(|&(&slot, _)| !registrar.palette.is_regulated(slot)).map(|(_, colour)| (colour.rgb, 0)).collect();
    ui.label(format!("Not on a spline yet: {} colours", loose.len()));
    let event = swatches(ui, registrar, &sorted_by_lightness(loose), lit.as_ref());
    registrar.take(ui, egui::Id::new("loose colours"), event);
    if !registrar.message.is_empty() {
        ui.small(&registrar.message);
    }

    ui.separator();
    ui.heading("Sub-palette");
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
    transform_controls(ui, registrar, &list);
}

/// The sub-palette's transform: crushing to fewer colours, and shifting hue, saturation and
/// lightness, all previewed on the blocks in the dock. Nothing is written to the files yet.
fn transform_controls(ui: &mut egui::Ui, registrar: &mut Registrar, list: &[(Rgb, u32)]) {
    ui.add_space(6.0);
    ui.label("Transform (a preview on the blocks in the dock)");
    let transform = &mut registrar.transform;
    let most = (list.len() as u32).saturating_sub(1).clamp(2, 64);
    ui.add(crate::debug::Slider::new(&mut transform.crush, 0..=most).text("crush to (colours; 0 = off)"))
        .on_hover_text("Lined up darkest first and cut into this many groups; each group becomes its pixels' average colour");
    ui.add_enabled_ui(transform.crush > 0, |ui| {
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
    ui.horizontal(|ui| {
        if ui.add_enabled(registrar.transform.is_active(), egui::Button::new("Reset")).clicked() {
            registrar.transform = Transform { spread: registrar.transform.spread, ..Transform::default() };
        }
        ui.checkbox(&mut registrar.before, "show the blocks before").on_hover_text("The dock shows the blocks as they are, not as the transform would make them");
        ui.add_enabled(false, egui::Button::new("Apply to the files")).on_disabled_hover_text("Coming with replacing: writes the PNGs, with undo");
    });
    if registrar.mapping.is_empty() {
        return;
    }
    let mut after: BTreeMap<Rgb, u32> = BTreeMap::new();
    for &(rgb, count) in list {
        *after.entry(registrar.mapping.get(&rgb).copied().unwrap_or(rgb)).or_default() += count;
    }
    ui.label(format!("After: {} colours", after.len()));
    let after = sorted_by_lightness(after);
    let event = swatches(ui, registrar, &after, None);
    registrar.take(ui, egui::Id::new("transformed colours"), event);
}

enum SplineAction {
    Renamed,
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
    /// How the colours are shared out when crushed, as in the Graphics Kit's palettes: an equal
    /// slice of lightness each, or an equal share of the pixels.
    spread: Spread,
    /// Added to hue (degrees), saturation and lightness (0-1).
    hue: f32,
    saturation: f32,
    lightness: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self { crush: 0, spread: Spread::EqualShares, hue: 0.0, saturation: 0.0, lightness: 0.0 }
    }
}

impl Transform {
    fn is_active(&self) -> bool {
        *self != Self { spread: self.spread, ..Self::default() }
    }

    /// What each of these colours (with their pixel counts) becomes: crushed first, then shifted.
    fn mapping(&self, colours: &[(Rgb, u32)]) -> BTreeMap<Rgb, Rgb> {
        let crushed = crushed(colours, self.crush, self.spread);
        colours.iter().map(|&(rgb, _)| (rgb, shifted(crushed.get(&rgb).copied().unwrap_or(rgb), self.hue, self.saturation, self.lightness))).collect()
    }
}

/// Crush colours to `count` (0, or at least as many as there are: unchanged). They're lined up
/// darkest first and cut into `count` groups, each an equal slice of lightness or an equal share of
/// the pixels (`spread`); each group becomes its pixels' average colour.
fn crushed(colours: &[(Rgb, u32)], count: u32, spread: Spread) -> BTreeMap<Rgb, Rgb> {
    if count == 0 || colours.len() <= count as usize {
        return colours.iter().map(|&(rgb, _)| (rgb, rgb)).collect();
    }
    let sorted = sorted_by_lightness(colours.iter().copied());
    let total: f32 = sorted.iter().map(|&(_, n)| n.max(1) as f32).sum();
    let (darkest, lightest) = (master_palette::lightness(sorted[0].0), master_palette::lightness(sorted[sorted.len() - 1].0));
    let mut so_far = 0.0;
    let group: Vec<usize> = sorted
        .iter()
        .map(|&(rgb, n)| {
            let n = n.max(1) as f32;
            let at = match spread {
                // Where the middle of its pixels falls along all the pixels.
                Spread::EqualShares => (so_far + n / 2.0) / total,
                Spread::EqualValues => (master_palette::lightness(rgb) - darkest) / (lightest - darkest).max(1e-6),
            };
            so_far += n;
            ((at * count as f32) as usize).min(count as usize - 1)
        })
        .collect();
    let mut sums = vec![([0.0f32; 3], 0.0f32); count as usize];
    for (&(rgb, n), &g) in sorted.iter().zip(&group) {
        let n = n.max(1) as f32;
        for k in 0..3 {
            sums[g].0[k] += rgb[k] as f32 * n;
        }
        sums[g].1 += n;
    }
    sorted
        .iter()
        .zip(&group)
        .map(|(&(rgb, _), &g)| {
            let (sum, n) = sums[g];
            (rgb, sum.map(|c| (c / n).round() as u8))
        })
        .collect()
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
    fn an_hsl_shift_goes_round_the_wheel_and_stays_in_range() {
        assert_eq!(shifted([255, 0, 0], 120.0, 0.0, 0.0), [0, 255, 0]);
        assert_eq!(shifted([255, 0, 0], -120.0, 0.0, 0.0), [0, 0, 255]);
        assert_eq!(shifted([200, 100, 50], 0.0, 0.0, 1.0), [255, 255, 255]);
        assert_eq!(shifted([200, 100, 50], 0.0, -1.0, 0.0)[0], shifted([200, 100, 50], 0.0, -1.0, 0.0)[2], "no saturation: grey");
    }
}

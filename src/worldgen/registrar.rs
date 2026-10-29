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
    /// What's selected in the library (blocks, or single textures); the first is open in the dock.
    selected: Vec<Item>,
    /// The block under the mouse in the library, or in the dock: its colours light up.
    hovered: Option<Item>,
    /// The composite blocks opened out in the library, to show their textures.
    expanded: BTreeSet<String>,
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
    mapped_for: Option<(Transform, Vec<Item>, Option<u64>, u64)>,
    /// Read the palette and textures again (after an undo).
    reread: bool,
    /// The undo steps, oldest first: what each change was.
    undo: Vec<String>,
    /// Per texture, the colours the Registrar replaced in it (to say if they come back).
    replaced: BTreeMap<String, BTreeSet<Rgb>>,
    /// Which of the master palette's colours are shown.
    view: PaletteView,
    /// The colour under the mouse this frame, and last frame's (the library, drawn first, lights the
    /// blocks using it).
    colour_hover: Option<Rgb>,
    last_colour_hover: Option<Rgb>,
    /// Each block's data file, and every block texture file's name.
    sources: BTreeMap<String, PathBuf>,
    texture_files: Vec<String>,
    /// The open block's settings as being edited in the dock (saved to its file on "Save"), and its
    /// tags as typed.
    draft: Option<BlockDef>,
    tags_text: String,
    /// The textures of blocks that aren't building aids: the art whose colours the palette holds.
    art_textures: BTreeSet<String>,
    /// Loading a new asset: the PNG coming in, the Art/ picker open or not, its files and filter.
    incoming: Option<Incoming>,
    loading: bool,
    art_files: Vec<PathBuf>,
    art_filter: String,
    /// "Clear out unused colours" was pressed: its preview is showing, waiting for a yes.
    clearing: bool,
    /// Goes up by one whenever the palette is saved.
    palette_changes: u64,
    mapping_version: u64,
    /// Show the blocks as they are, not as the transform would make them.
    before: bool,
}

/// Something in the library that can be selected: a block, or one texture on its own (one of a
/// composite block's, or one no block uses yet) [David: "handle individual textures in a composite
/// individually"].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Item {
    Block(String),
    Texture(String),
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
        self.art_textures.clear();
        self.sources = registry.defs().iter().filter_map(|def| registry.source(&def.name).map(|path| (def.name.clone(), path.to_path_buf()))).collect();
        // Every texture file there is, for choosing a face's.
        self.texture_files = std::fs::read_dir(&folder).into_iter().flatten().flatten().filter_map(|entry| entry.path().file_stem().filter(|_| entry.path().extension().is_some_and(|extension| extension == "png")).map(|stem| stem.to_string_lossy().into_owned())).collect();
        self.texture_files.sort();
        for def in registry.defs().iter().filter(|def| shown(def)) {
            for name in texture_names(def) {
                if self.textures.contains_key(name) {
                    continue;
                }
                let texture = match crate::render::textures::load_texture(&folder.join(format!("{name}.png"))) {
                    Ok(pixels) => Texture { colours: master_palette::colours_in(&pixels), image: None, preview: None, pixels, problem: None },
                    Err(problem) => Texture { colours: Vec::new(), image: None, preview: None, pixels: Vec::new(), problem: Some(problem) },
                };
                self.textures.insert(name.to_string(), texture);
            }
            // Building aids' textures stay out of the palette.
            if !def.tool {
                self.art_textures.extend(texture_names(def).into_iter().map(str::to_string));
            }
        }
        let used: Vec<Rgb> = self.art_colours().into_iter().collect();
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

    /// Write the transform into the selected textures (every block using one of them
    /// changes too), after saving copies for undo. The colours it replaced are remembered, so it can
    /// be said if they come back.
    fn apply(&mut self, defs: &[&BlockDef]) {
        let names = self.selected_textures(defs);
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
        let used = self.art_colours();
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

    /// Write a block's settings to its data file (its comments at the top kept), after saving a copy
    /// for undo. The game reads the file again at once.
    fn save_block(&mut self, def: &BlockDef, path: Option<&Path>) {
        let Some(path) = path else { return };
        let what = format!("Saved {}", path.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned()));
        if let Err(error) = save_undo_step(&undo_folder(), &what, &[path.to_path_buf()]) {
            self.message = format!("Not saved: couldn't save the undo copy ({error}).");
            return;
        }
        let header = std::fs::read_to_string(path).map(|text| crate::data::header_of(&text)).unwrap_or_default();
        self.message = match std::fs::write(path, crate::data::block_file_text(def, &header)) {
            Ok(()) => {
                self.undo.push(what.clone());
                format!("{what}; the game picks it up in a moment.")
            }
            Err(error) => format!("Couldn't save {}: {error}.", path.display()),
        };
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

    /// The textures an item stands for: a block's, or the one texture.
    fn textures_of(&self, item: &Item, defs: &[&BlockDef]) -> Vec<String> {
        match item {
            Item::Block(name) => defs.iter().find(|def| &def.name == name).map_or_else(Vec::new, |def| texture_names(def).into_iter().map(str::to_string).collect()),
            Item::Texture(name) => vec![name.clone()],
        }
    }

    /// Every texture the selection stands for, each once.
    fn selected_textures(&self, defs: &[&BlockDef]) -> BTreeSet<String> {
        self.selected.iter().flat_map(|item| self.textures_of(item, defs)).collect()
    }

    /// Colours with how many pixels each, over these textures (each once).
    fn colours_in_textures<'a>(&self, names: impl IntoIterator<Item = &'a String>) -> BTreeMap<Rgb, u32> {
        let mut colours = BTreeMap::new();
        for name in names.into_iter().collect::<BTreeSet<_>>() {
            for &(rgb, count) in self.textures.get(name).map_or(&[][..], |texture| &texture.colours) {
                *colours.entry(rgb).or_default() += count;
            }
        }
        colours
    }

    /// What an item is called: a block's name, or a texture's file.
    fn label(item: &Item, defs: &[&BlockDef]) -> String {
        match item {
            Item::Block(name) => defs.iter().find(|def| &def.name == name).map_or_else(|| name.clone(), |def| def.display_name.clone()),
            Item::Texture(name) => format!("{name}.png"),
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

    /// Every colour the art uses: the textures of every block that isn't a building aid.
    fn art_colours(&self) -> BTreeSet<Rgb> {
        self.art_textures.iter().filter_map(|name| self.textures.get(name)).flat_map(|texture| texture.colours.iter().map(|&(rgb, _)| rgb)).collect()
    }

    /// How many of a block's colours aren't on a spline (0: regulated; a building aid's never
    /// count).
    fn unregulated(&self, def: &BlockDef) -> usize {
        if def.tool {
            return 0;
        }
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
    registrar.last_colour_hover = registrar.colour_hover.take();
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
/// end), a composite block opening out to its textures; then the textures no block uses. Click to
/// open one; Ctrl + click to add it to the selection, or take it out.
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
                library_row(ui, registrar, def, tag);
            }
        });
    }
    if !untagged.is_empty() {
        egui::CollapsingHeader::new("(no tags)").id_salt("untagged").default_open(true).show(ui, |ui| {
            for def in untagged {
                library_row(ui, registrar, def, "");
            }
        });
    }
    // Textures in the folder that no block uses (a new one waiting for a block, say).
    let used: BTreeSet<&str> = defs.iter().flat_map(|def| texture_names(def)).collect();
    let loose: Vec<String> = registrar.texture_files.iter().filter(|name| !used.contains(name.as_str())).cloned().collect();
    if !loose.is_empty() {
        egui::CollapsingHeader::new(format!("Textures on no block ({})", loose.len())).id_salt("loose textures").default_open(true).show(ui, |ui| {
            for name in loose {
                texture_row(ui, registrar, &name, "");
            }
        });
    }
}

/// Select an item: on its own, or with Ctrl, added to (or taken out of) the selection.
fn select(ui: &egui::Ui, registrar: &mut Registrar, item: Item) {
    if ui.input(|input| input.modifiers.command) {
        if registrar.selected.contains(&item) {
            registrar.selected.retain(|other| other != &item);
        } else {
            registrar.selected.push(item);
        }
    } else {
        registrar.selected = vec![item];
    }
}

fn library_row(ui: &mut egui::Ui, registrar: &mut Registrar, def: &BlockDef, group: &str) {
    let unregulated = registrar.unregulated(def);
    let item = Item::Block(def.name.clone());
    let selected = registrar.selected.contains(&item);
    // Lit when it uses the colour under the mouse, or any picked colour.
    let colours = registrar.block_colours(def);
    let lit = registrar.last_colour_hover.is_some_and(|rgb| colours.contains_key(&rgb)) || registrar.picked.iter().any(|rgb| colours.contains_key(rgb));
    let names: Vec<String> = texture_names(def).into_iter().map(str::to_string).collect();
    let composite = names.len() > 1;
    let open = registrar.expanded.contains(&def.name);
    let row = ui.horizontal(|ui| {
        ui.colored_label(if lit { PICKED } else { Color32::TRANSPARENT }, "•").on_hover_text("Uses the colour under the mouse, or a picked one");
        // A composite block opens out to its textures.
        if composite {
            if ui.small_button(if open { "▾" } else { "▸" }).on_hover_text("Show or hide its textures, to work on one at a time").clicked() {
                if open {
                    registrar.expanded.remove(&def.name);
                } else {
                    registrar.expanded.insert(def.name.clone());
                }
            }
        } else {
            ui.add_space(ui.spacing().interact_size.y);
        }
        if let Some(top) = def.faces.get(4).filter(|top| !top.is_empty()) {
            texture_image(ui, registrar, top, 20.0, false);
        }
        let label = ui.selectable_label(selected, &def.display_name);
        if composite {
            ui.small(format!("{} textures", names.len())).on_hover_text("A composite block: its faces have different textures");
        }
        if def.tool {
            ui.small("building aid").on_hover_text("Not world art: its colours stay out of the master palette");
        } else if unregulated > 0 {
            ui.small(format!("{unregulated} unregulated")).on_hover_text("Colours not on any spline yet");
        } else {
            ui.small("regulated").on_hover_text("Every colour is on a spline");
        }
        label
    });
    let label = row.inner;
    if label.hovered() {
        registrar.hovered = Some(item.clone());
    }
    if label.clicked() {
        select(ui, registrar, item);
    }
    label.on_hover_text(&def.name);
    if composite && open {
        ui.indent(("block textures", &def.name, group), |ui| {
            for name in &names {
                // Which faces it's on.
                let faces: Vec<&str> = (0..6).filter(|&k| &def.faces[k] == name).map(|k| crate::data::FACE_NAMES[k]).collect();
                let on = if faces.len() == 4 && faces.iter().all(|face| ["north", "south", "east", "west"].contains(face)) { "sides".to_string() } else { faces.join(", ") };
                texture_row(ui, registrar, name, &on);
            }
        });
    }
}

/// One texture in the library: on its own, or one of a composite block's (`on`: which faces).
fn texture_row(ui: &mut egui::Ui, registrar: &mut Registrar, name: &str, on: &str) {
    let item = Item::Texture(name.to_string());
    let selected = registrar.selected.contains(&item);
    let colours = registrar.colours_in_textures([&name.to_string()]);
    let lit = registrar.last_colour_hover.is_some_and(|rgb| colours.contains_key(&rgb)) || registrar.picked.iter().any(|rgb| colours.contains_key(rgb));
    let unregulated = colours.keys().filter(|&&rgb| !registrar.palette.slot_of(rgb).is_some_and(|slot| registrar.palette.is_regulated(slot))).count();
    let row = ui.horizontal(|ui| {
        ui.colored_label(if lit { PICKED } else { Color32::TRANSPARENT }, "•");
        texture_image(ui, registrar, name, 16.0, false);
        let label = ui.selectable_label(selected, name);
        if !on.is_empty() {
            ui.small(on);
        }
        if registrar.textures.contains_key(name) {
            ui.small(if unregulated > 0 { format!("{unregulated} unregulated") } else { "regulated".to_string() });
        }
        label
    });
    let label = row.inner;
    if label.hovered() {
        registrar.hovered = Some(item.clone());
    }
    if label.clicked() {
        select(ui, registrar, item);
    }
}

/// A texture's image for showing (made the first time it's asked for), as it is or as the
/// sub-palette's transform would make it. Why not, if it can't be shown.
fn texture_id(ctx: &egui::Context, registrar: &mut Registrar, name: &str, transformed: bool) -> Result<egui::TextureId, String> {
    let Some(texture) = registrar.textures.get_mut(name) else {
        return Err(format!("{name}.png isn't read yet"));
    };
    if let Some(problem) = &texture.problem {
        return Err(problem.clone());
    }
    let side = crate::render::textures::TEXTURE_SIZE as usize;
    // As the transform would make it: each pixel's colour mapped (made again whenever the
    // transform changes).
    let mapping = &registrar.mapping;
    if transformed && !registrar.before && !mapping.is_empty() {
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
                None => texture.preview = Some((version, ctx.load_texture(format!("registrar {name} transformed"), image, egui::TextureOptions::NEAREST))),
            }
        }
        if let Some((_, handle)) = &texture.preview {
            return Ok(handle.id());
        }
    }
    Ok(texture.image.get_or_insert_with(|| ctx.load_texture(format!("registrar {name}"), egui::ColorImage::from_rgba_unmultiplied([side, side], &texture.pixels), egui::TextureOptions::NEAREST)).id())
}

/// A texture, `size` points square.
fn texture_image(ui: &mut egui::Ui, registrar: &mut Registrar, name: &str, size: f32, transformed: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    match texture_id(ui.ctx(), registrar, name, transformed) {
        Ok(id) => {
            ui.painter().image(id, rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
            response
        }
        Err(problem) => {
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 255));
            response.on_hover_text(problem)
        }
    }
}

/// The tiling checker: a texture repeated 5 × 5, each turned at random as the game turns them (or
/// not), to see how it repeats [David, 2026-10-29]. As the transform would make it.
fn tiling_checker(ui: &mut egui::Ui, registrar: &mut Registrar, name: &str, random: &mut bool) {
    ui.checkbox(random, "random rotation").on_hover_text("Each tile turned 0°, 90°, 180° or 270°, picked from its place, as the game does");
    const TILES: i32 = 5;
    const TILE: f32 = 40.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(TILE * TILES as f32, TILE * TILES as f32), egui::Sense::hover());
    let id = match texture_id(ui.ctx(), registrar, name, true) {
        Ok(id) => id,
        Err(problem) => {
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 255));
            response.on_hover_text(problem);
            return;
        }
    };
    let mut mesh = egui::Mesh::with_texture(id);
    let corners = [egui::pos2(0.0, 0.0), egui::pos2(1.0, 0.0), egui::pos2(1.0, 1.0), egui::pos2(0.0, 1.0)];
    for y in 0..TILES {
        for x in 0..TILES {
            // A quarter turn picked from its place (the same each time it's drawn).
            let turns = if *random { ((x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)) as u32 % 4) as usize } else { 0 };
            let tile = egui::Rect::from_min_size(rect.min + egui::vec2(x as f32 * TILE, y as f32 * TILE), egui::vec2(TILE, TILE));
            let at = [tile.left_top(), tile.right_top(), tile.right_bottom(), tile.left_bottom()];
            let first = mesh.vertices.len() as u32;
            for k in 0..4 {
                mesh.vertices.push(egui::epaint::Vertex { pos: at[k], uv: corners[(k + turns) % 4], color: Color32::WHITE });
            }
            mesh.indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));
}

// ── The dock ──────────────────────────────────────────────────────────────

/// The open item: a block (its name, tags, status, settings, faces and palette) or one texture (its
/// image, the blocks using it, its palette); its tiling; and the rest of the selection, smaller.
fn dock(ui: &mut egui::Ui, registrar: &mut Registrar, defs: &[&BlockDef]) {
    ui.horizontal(|ui| {
        ui.heading("Dock");
        if ui.button("Load a new asset…").on_hover_text("Bring a PNG in from Art/ (or drag one onto the window) and register it").clicked() {
            registrar.loading = !registrar.loading;
            if registrar.loading {
                registrar.art_files = art_pngs();
            }
        }
    });
    if registrar.loading {
        art_picker(ui, registrar);
    }
    ui.separator();
    // A PNG coming in takes the dock until it's registered or cancelled.
    if registrar.incoming.is_some() {
        incoming_dock(ui, registrar);
        return;
    }
    let Some(first) = registrar.selected.first().cloned() else {
        ui.label("Click a block or texture in the library to open it here.");
        return;
    };
    let tiling: Option<(String, bool)> = match &first {
        Item::Block(name) => {
            let Some(def) = defs.iter().find(|def| &def.name == name).copied() else { return };
            block_dock(ui, registrar, def);
            // The top's tiling, or the first face that has a texture.
            let face = if def.faces[4].is_empty() { def.faces.iter().position(|face| !face.is_empty()) } else { Some(4) };
            face.map(|k| (def.faces[k].clone(), def.rotations[k] == crate::data::TextureRotation::Random))
        }
        Item::Texture(name) => {
            texture_dock(ui, registrar, name, defs);
            // Turned at random if a block turns it on any face.
            let random = defs.iter().any(|def| (0..6).any(|k| &def.faces[k] == name && def.rotations[k] == crate::data::TextureRotation::Random));
            Some((name.clone(), random))
        }
    };
    if let Some((name, random)) = tiling {
        egui::CollapsingHeader::new(format!("Tiling ({name}.png)")).id_salt("registrar tiling").default_open(false).show(ui, |ui| {
            // Starts as the block has it; the switch here only changes the checker.
            let key = egui::Id::new(("registrar tiling random", &name));
            let mut turned = ui.data(|data| data.get_temp::<bool>(key)).unwrap_or(random);
            tiling_checker(ui, registrar, &name, &mut turned);
            ui.data_mut(|data| data.insert_temp(key, turned));
        });
    }

    // The rest of the selection, smaller, so a transform can be seen across the whole group.
    let others: Vec<Item> = registrar.selected.iter().skip(1).cloned().collect();
    if !others.is_empty() {
        ui.add_space(8.0);
        ui.label("The rest of the selection");
        ui.horizontal_wrapped(|ui| {
            for other in others {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        for name in registrar.textures_of(&other, defs) {
                            if texture_image(ui, registrar, &name, 48.0, true).hovered() {
                                registrar.hovered = Some(other.clone());
                            }
                        }
                    });
                    ui.small(Registrar::label(&other, defs));
                });
                ui.add_space(8.0);
            }
        });
    }
    ui.add_space(8.0);
    let names = registrar.textures_of(&first, defs);
    let colours = registrar.colours_in_textures(&names);
    ui.label(format!("Its palette: {} colours", colours.len()));
    let list: Vec<(Rgb, u32)> = sorted_by_lightness(colours);
    let event = swatches(ui, registrar, &list, None);
    registrar.take(ui, egui::Id::new("dock colours"), event);
}

/// A block in the dock: its name, tags, whether it's regulated or composite, its settings and faces.
fn block_dock(ui: &mut egui::Ui, registrar: &mut Registrar, def: &BlockDef) {
    ui.heading(&def.display_name);
    ui.monospace(&def.name);
    ui.label(if def.tags.is_empty() { "No tags".to_string() } else { format!("Tags: {}", def.tags.join(", ")) });
    let unregulated = registrar.unregulated(def);
    let textures = texture_names(def).len();
    ui.horizontal(|ui| {
        if def.tool {
            ui.label("Building aid: its colours stay out of the master palette");
        } else if unregulated == 0 {
            ui.colored_label(Color32::from_rgb(120, 220, 120), "Regulated: every colour is on a spline");
        } else {
            ui.colored_label(Color32::from_rgb(255, 200, 80), format!("{unregulated} colours not on a spline"));
        }
        if textures > 1 {
            ui.separator();
            ui.label(format!("Composite: {textures} textures"));
        }
    });
    egui::CollapsingHeader::new("Block").id_salt("registrar block settings").default_open(false).show(ui, |ui| block_settings(ui, registrar, def));
    ui.add_space(8.0);
    // Its faces, as many as differ; click one to work on that texture on its own.
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
                    registrar.hovered = Some(Item::Block(def.name.clone()));
                }
                let all_same = def.faces.iter().all(|face| face == &def.faces[0]);
                if ui.small_button(format!("{} ({name})", if all_same { "all faces" } else { label })).on_hover_text("Open this texture on its own").clicked() {
                    registrar.selected = vec![Item::Texture(name.to_string())];
                }
            });
        }
    });
}

/// One texture in the dock: its image, which blocks use it on which faces, and whether it's
/// regulated.
fn texture_dock(ui: &mut egui::Ui, registrar: &mut Registrar, name: &str, defs: &[&BlockDef]) {
    ui.heading(format!("{name}.png"));
    let colours = registrar.colours_in_textures([&name.to_string()]);
    let unregulated = colours.keys().filter(|&&rgb| !registrar.palette.slot_of(rgb).is_some_and(|slot| registrar.palette.is_regulated(slot))).count();
    if unregulated == 0 {
        ui.colored_label(Color32::from_rgb(120, 220, 120), "Regulated: every colour is on a spline");
    } else {
        ui.colored_label(Color32::from_rgb(255, 200, 80), format!("{unregulated} colours not on a spline"));
    }
    let users: Vec<String> = defs
        .iter()
        .filter_map(|def| {
            let faces: Vec<&str> = (0..6).filter(|&k| def.faces[k] == name).map(|k| crate::data::FACE_NAMES[k]).collect();
            (!faces.is_empty()).then(|| format!("{} ({})", def.display_name, faces.join(", ")))
        })
        .collect();
    ui.label(if users.is_empty() { "No block uses it yet.".to_string() } else { format!("Used by {}", users.join("; ")) });
    ui.add_space(8.0);
    if texture_image(ui, registrar, name, 128.0, true).hovered() {
        registrar.hovered = Some(Item::Texture(name.to_string()));
    }
}

/// How a block's faces take their textures.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FaceKind {
    /// One texture on every face.
    One,
    /// The top, the four sides and the bottom (like grass).
    TopSidesBottom,
    /// Each of the six faces its own.
    Each,
}

/// The open block's settings, edited on a copy and saved to its data file on "Save" (a copy kept
/// for Undo first; the game reads the file again at once).
fn block_settings(ui: &mut egui::Ui, registrar: &mut Registrar, def: &BlockDef) {
    if registrar.draft.as_ref().is_none_or(|draft| draft.name != def.name) {
        registrar.draft = Some(def.clone());
        registrar.tags_text = def.tags.join(", ");
    }
    let files = registrar.texture_files.clone();
    let Some(draft) = registrar.draft.as_mut() else { return };
    ui.horizontal(|ui| {
        ui.label("name:");
        ui.text_edit_singleline(&mut draft.display_name);
    });

    // Which faces have their own textures.
    let faces = draft.faces.clone();
    let mut kind = if faces.iter().all(|face| face == &faces[0]) {
        FaceKind::One
    } else if faces[..4].iter().all(|face| face == &faces[0]) {
        FaceKind::TopSidesBottom
    } else {
        FaceKind::Each
    };
    let was = kind;
    ui.horizontal(|ui| {
        ui.radio_value(&mut kind, FaceKind::One, "one texture");
        ui.radio_value(&mut kind, FaceKind::TopSidesBottom, "top, sides and bottom");
        ui.radio_value(&mut kind, FaceKind::Each, "each face its own");
    });
    if kind != was {
        match kind {
            // One: the top's (or the first set) everywhere.
            FaceKind::One => {
                let texture = if faces[4].is_empty() { faces.iter().find(|face| !face.is_empty()).cloned().unwrap_or_default() } else { faces[4].clone() };
                draft.faces = std::array::from_fn(|_| texture.clone());
                let rotation = draft.rotations[4];
                draft.rotations = [rotation; 6];
            }
            // Top, sides and bottom: the sides all as the north one.
            FaceKind::TopSidesBottom => {
                for k in 1..4 {
                    draft.faces[k] = draft.faces[0].clone();
                    draft.rotations[k] = draft.rotations[0];
                }
            }
            FaceKind::Each => {}
        }
    }
    // A texture and random rotation for each group of faces.
    let groups: Vec<(&str, Vec<usize>)> = match kind {
        FaceKind::One => vec![("all faces", vec![0, 1, 2, 3, 4, 5])],
        FaceKind::TopSidesBottom => vec![("top", vec![4]), ("sides", vec![0, 1, 2, 3]), ("bottom", vec![5])],
        FaceKind::Each => crate::data::FACE_NAMES.iter().enumerate().map(|(k, name)| (*name, vec![k])).collect(),
    };
    egui::Grid::new("registrar faces").num_columns(3).show(ui, |ui| {
        for (label, which) in groups {
            ui.label(label);
            let mut texture = draft.faces[which[0]].clone();
            egui::ComboBox::from_id_salt(("registrar face", label)).selected_text(if texture.is_empty() { "(none)" } else { &texture }).show_ui(ui, |ui| {
                for file in &files {
                    ui.selectable_value(&mut texture, file.clone(), file);
                }
            });
            if texture != draft.faces[which[0]] {
                for &k in &which {
                    draft.faces[k] = texture.clone();
                }
            }
            let mut random = which.iter().all(|&k| draft.rotations[k] == crate::data::TextureRotation::Random);
            if ui.checkbox(&mut random, "random rotation").on_hover_text("Each block turned at random, to hide repetition (only for textures with no \"up\")").changed() {
                for &k in &which {
                    draft.rotations[k] = if random { crate::data::TextureRotation::Random } else { crate::data::TextureRotation::Fixed };
                }
            }
            ui.end_row();
        }
    });

    // The look, and what it means for bodies and building.
    let mut look = match (draft.see_through, draft.inner_faces) {
        (true, true) => 1,
        (true, false) => 2,
        _ => 0,
    };
    let looked = look;
    ui.horizontal(|ui| {
        ui.label("look:");
        ui.radio_value(&mut look, 0, "solid");
        ui.radio_value(&mut look, 1, "grid").on_hover_text("See-through, every cell's faces drawn");
        ui.radio_value(&mut look, 2, "outline").on_hover_text("See-through, only a volume's outside drawn");
    });
    if look != looked {
        // What each look usually is (as a block file without these settings would be).
        let see_through = look != 0;
        draft.see_through = see_through;
        draft.opaque = !see_through;
        draft.inner_faces = look == 1;
        draft.solid = !see_through;
        draft.replaceable = see_through;
    }
    ui.add(crate::debug::Slider::new(&mut draft.friction, 0.0..=3.0).text("friction (1 = ordinary ground)"));
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut draft.solid, "solid").on_hover_text("Stops bodies");
        ui.checkbox(&mut draft.targetable, "the crosshair can target it");
        ui.checkbox(&mut draft.replaceable, "replaceable").on_hover_text("Placing a block into it just replaces it, like air");
        ui.checkbox(&mut draft.tool, "building aid").on_hover_text("Not world art: its colours stay out of the master palette");
    });
    ui.horizontal(|ui| {
        ui.label("tags:");
        if ui.text_edit_singleline(&mut registrar.tags_text).on_hover_text("Separated by commas; a / nests one in another, as folders do").changed() {
            draft.tags = registrar.tags_text.split(',').map(|tag| tag.split('/').map(str::trim).filter(|part| !part.is_empty()).collect::<Vec<_>>().join("/")).filter(|tag| !tag.is_empty()).collect();
        }
    });

    let changed = draft != def;
    let draft = draft.clone();
    let path = registrar.sources.get(&def.name).cloned();
    ui.horizontal(|ui| {
        if ui.add_enabled(changed && path.is_some(), egui::Button::new("Save to the block file")).on_disabled_hover_text(if path.is_none() { "This block has no data file" } else { "Nothing changed" }).clicked() {
            registrar.save_block(&draft, path.as_deref());
        }
        if ui.add_enabled(changed, egui::Button::new("Revert")).clicked() {
            registrar.draft = Some(def.clone());
            registrar.tags_text = def.tags.join(", ");
        }
        if let Some(path) = &path {
            ui.small(path.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned()));
        }
    });
}

// ── Loading a new asset ───────────────────────────────────────────────────

/// A PNG being brought into the library: where from, its pixels (or why it can't be used), the
/// name it'll have, and whether a block is made for it too.
pub struct Incoming {
    from: PathBuf,
    pixels: Result<Vec<u8>, String>,
    name: String,
    make_block: bool,
    image: Option<egui::TextureHandle>,
}

/// A file's name as a texture or block name: lowercase letters, digits and `_` only
/// ("Red Granite-2.png" → "red_granite_2").
fn asset_name(path: &Path) -> String {
    let stem = path.file_stem().map_or(String::new(), |stem| stem.to_string_lossy().to_lowercase());
    let name: String = stem.chars().map(|c| if c.is_ascii_lowercase() || c.is_ascii_digit() { c } else { '_' }).collect();
    name.split('_').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("_")
}

/// Every PNG under `Art/`, for picking one to load.
fn art_pngs() -> Vec<PathBuf> {
    fn walk(folder: &Path, depth: u32, found: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() && depth < 6 {
                walk(&path, depth + 1, found);
            } else if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("png")) {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&crate::debug::project_folder().join("Art"), 0, &mut found);
    found.sort();
    found
}

impl Registrar {
    /// A PNG to bring in (picked from Art/, or dragged onto the window): shown in the dock.
    pub fn receive(&mut self, path: &Path) {
        self.incoming = Some(Incoming {
            from: path.to_path_buf(),
            pixels: crate::render::textures::load_texture(path),
            name: asset_name(path),
            make_block: true,
            image: None,
        });
        self.loading = false;
    }

    /// Copy the incoming PNG into the block textures, and make its block if asked (a plain block
    /// with it on every face), after an undo step. The game reads them at once.
    fn register(&mut self) {
        let Some(incoming) = self.incoming.take() else { return };
        let Ok(pixels) = &incoming.pixels else { return };
        let name = incoming.name.clone();
        let texture = block_texture_folder().join(format!("{name}.png"));
        let id = format!("incus:{name}");
        let block = crate::data::data_folder().join("incus").join("blocks").join(format!("{name}.toml"));
        let making_block = incoming.make_block && !block.exists();
        let mut files = vec![texture.clone()];
        if making_block {
            files.push(block.clone());
        }
        let what = format!("Registered {name}.png{}", if making_block { format!(" and the block {id}") } else { String::new() });
        if let Err(error) = save_undo_step(&undo_folder(), &what, &files) {
            self.message = format!("Not registered: couldn't save the undo step ({error}).");
            return;
        }
        if let Err(error) = write_png(&texture, pixels) {
            self.message = format!("Couldn't write {}: {error}. Undo takes back anything written.", texture.display());
            return;
        }
        if making_block {
            let from = incoming.from.file_name().map_or(String::new(), |file| file.to_string_lossy().into_owned());
            let header = format!("# {name}: registered in the Registrar from {from}.\n# This file defines the block \"{id}\". See docs/data-files.md for every setting.");
            let text = match crate::data::parse_block(&id, &format!("textures = {{ all = {name:?} }}")) {
                Ok((def, _)) => crate::data::block_file_text(&def, &header),
                Err(error) => {
                    self.message = format!("Couldn't make the block: {error}.");
                    return;
                }
            };
            if let Err(error) = std::fs::write(&block, text) {
                self.message = format!("Couldn't write {}: {error}. Undo takes back anything written.", block.display());
                return;
            }
        }
        self.undo.push(what.clone());
        self.selected = vec![if making_block { Item::Block(id) } else { Item::Texture(name) }];
        self.message = format!("{what}; it shows in the library in a moment.");
    }
}

/// Picking a PNG from Art/ to bring in.
fn art_picker(ui: &mut egui::Ui, registrar: &mut Registrar) {
    ui.horizontal(|ui| {
        ui.label("From Art/:");
        ui.add(egui::TextEdit::singleline(&mut registrar.art_filter).hint_text("filter").desired_width(160.0));
        if ui.small_button("Look again").clicked() {
            registrar.art_files = art_pngs();
        }
        if ui.small_button("Close").clicked() {
            registrar.loading = false;
        }
    });
    ui.small("Or drag a PNG from anywhere onto the window.");
    let art = crate::debug::project_folder().join("Art");
    let filter = registrar.art_filter.to_lowercase();
    let mut chosen = None;
    egui::ScrollArea::vertical().max_height(220.0).id_salt("art picker").show(ui, |ui| {
        for path in &registrar.art_files {
            let shown = path.strip_prefix(&art).unwrap_or(path).display().to_string().replace('\\', "/");
            if !filter.is_empty() && !shown.to_lowercase().contains(&filter) {
                continue;
            }
            if ui.selectable_label(false, &shown).clicked() {
                chosen = Some(path.clone());
            }
        }
    });
    if let Some(path) = chosen {
        registrar.receive(&path);
    }
}

/// The incoming PNG in the dock: its picture and colours, its name, and Register.
fn incoming_dock(ui: &mut egui::Ui, registrar: &mut Registrar) {
    let palette = registrar.palette.clone();
    let existing: Vec<String> = registrar.texture_files.clone();
    let editable = registrar.palette_problem.is_none();
    let Some(incoming) = registrar.incoming.as_mut() else { return };
    ui.heading(format!("New: {}", incoming.from.file_name().map_or(String::new(), |file| file.to_string_lossy().into_owned())));
    ui.small(incoming.from.display().to_string());
    let pixels = match &incoming.pixels {
        Ok(pixels) => pixels.clone(),
        Err(problem) => {
            ui.colored_label(Color32::from_rgb(255, 80, 200), format!("Can't be used: {problem}"));
            if ui.button("Cancel").clicked() {
                registrar.incoming = None;
            }
            return;
        }
    };
    let side = crate::render::textures::TEXTURE_SIZE as usize;
    let image = incoming.image.get_or_insert_with(|| ui.ctx().load_texture("registrar incoming", egui::ColorImage::from_rgba_unmultiplied([side, side], &pixels), egui::TextureOptions::NEAREST));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(128.0, 128.0), egui::Sense::hover());
    ui.painter().image(image.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
    // Its colours against the master palette.
    let colours = master_palette::colours_in(&pixels);
    let known = colours.iter().filter(|(rgb, _)| palette.slot_of(*rgb).is_some()).count();
    let regulated = colours.iter().filter(|(rgb, _)| palette.slot_of(*rgb).is_some_and(|slot| palette.is_regulated(slot))).count();
    ui.label(format!("{} colours: {regulated} on splines, {} in the palette but not on a spline, {} new", colours.len(), known - regulated, colours.len() - known));
    if colours.len() > known {
        ui.small("New colours join the master palette (not on a spline) when it's registered; then crush it onto a spline as usual.");
    }
    ui.horizontal(|ui| {
        ui.label("name:");
        if ui.text_edit_singleline(&mut incoming.name).lost_focus() {
            incoming.name = asset_name(Path::new(&incoming.name));
        }
    });
    let name = incoming.name.clone();
    ui.checkbox(&mut incoming.make_block, format!("and a block, incus:{name}, with it on every face"));
    if existing.contains(&name) {
        ui.colored_label(Color32::from_rgb(255, 200, 80), format!("{name}.png is already a texture: registering replaces it (Undo puts it back), for every block using it."));
    }
    let block = crate::data::data_folder().join("incus").join("blocks").join(format!("{name}.toml"));
    if incoming.make_block && block.exists() {
        ui.small(format!("The block incus:{name} already exists: only its texture is replaced."));
    }
    ui.horizontal(|ui| {
        if ui.add_enabled(!name.is_empty() && editable, egui::Button::new("Register")).clicked() {
            registrar.register();
        }
        if ui.button("Cancel").clicked() {
            registrar.incoming = None;
        }
    });
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
            self.colour_hover = Some(rgb);
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
        self.colours_in_textures(&self.selected_textures(defs))
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

    /// Save colours as a GPL palette in `palettes/`, as "Registrar - <what>.gpl" (the same name each
    /// time, so Aseprite can load it again after a change).
    fn export_gpl(&mut self, what: &str, colours: &[Rgb]) {
        let name = format!("Registrar - {}", what.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], " "));
        self.message = match super::palette::save_palette(&super::palette::palette_folder(), &name, colours) {
            Ok(path) => format!("Exported {} colours to {}.", colours.len(), path.display()),
            Err(error) => format!("Couldn't export {name}: {error}."),
        };
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

    // The colours the hovered item (or else the selection) uses light up.
    let items: Vec<Item> = registrar.hovered.clone().map_or_else(|| registrar.selected.clone(), |item| vec![item]);
    let textures: Vec<String> = items.iter().flat_map(|item| registrar.textures_of(item, defs)).collect();
    let lit: BTreeSet<Rgb> = registrar.colours_in_textures(&textures).into_keys().collect();
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
        // GPL palettes, for Aseprite: what's in view (in the master palette's order), or the picked.
        ui.horizontal_wrapped(|ui| {
            let view = match registrar.view {
                PaletteView::Everything => "master palette",
                PaletteView::Splines => "splines",
                PaletteView::Loose => "not on a spline",
            };
            if ui.add_enabled(!in_view.is_empty(), egui::Button::new("Export GPL of the view")).on_hover_text("Saves what's in view as a palette in palettes/, for Aseprite").clicked() {
                registrar.export_gpl(view, &in_view);
            }
            if ui.add_enabled(!registrar.picked.is_empty(), egui::Button::new("Export GPL of the picked")).clicked() {
                let picked: Vec<Rgb> = sorted_by_lightness(registrar.picked.iter().map(|&rgb| (rgb, 0))).into_iter().map(|(rgb, _)| rgb).collect();
                registrar.export_gpl("picked", &picked);
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
        if registrar.selected.is_empty() {
            ui.label("Select blocks or textures in the library to see their combined palette.");
            return;
        }
        let combined = registrar.selection_colours(defs);
        let names: Vec<String> = registrar.selected.iter().map(|item| Registrar::label(item, defs)).collect();
        ui.label(format!("{}: {} colours", names.join(", "), combined.len()));
        let list = sorted_by_lightness(combined);
        let event = swatches(ui, registrar, &list, None);
        registrar.take(ui, egui::Id::new("sub-palette colours"), event);
        ui.horizontal(|ui| {
            if ui.button("Pick all of these").clicked() {
                registrar.picked.extend(list.iter().map(|&(rgb, _)| rgb));
            }
            if ui.button("Export GPL").on_hover_text("Saves these colours as a palette in palettes/, for Aseprite").clicked() {
                let colours: Vec<Rgb> = list.iter().map(|&(rgb, _)| rgb).collect();
                registrar.export_gpl(&format!("{} sub-palette", names.join(", ")), &colours);
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
    let names = registrar.selected_textures(defs);
    let others: Vec<&str> = defs
        .iter()
        .filter(|def| !registrar.selected.contains(&Item::Block(def.name.clone())) && texture_names(def).iter().any(|name| names.contains(*name)))
        .map(|def| def.display_name.as_str())
        .collect();
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
        let open = registrar.selected.first().map_or_else(
            || "spline".to_string(),
            |item| match item {
                Item::Block(name) => name.rsplit(':').next().unwrap_or(name).to_string(),
                Item::Texture(name) => name.clone(),
            },
        );
        let name = format!("{open} crushed");
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
    let all: Vec<usize> = (0..registrar.palette.splines.len()).collect();
    spline_group(ui, registrar, "", &all, full, lit, &mut action);
    match action {
        Some(SplineAction::Renamed) => registrar.save("Saved.".to_string()),
        Some(SplineAction::PickMany(indices)) => {
            let shades: Vec<Rgb> = indices.iter().flat_map(|&index| registrar.palette.splines[index].steps.clone()).filter_map(|slot| registrar.palette.colours.get(&slot)).map(|colour| colour.rgb).collect();
            registrar.picked.extend(shades);
        }
        Some(SplineAction::Pick(index)) => {
            let shades: Vec<Rgb> = registrar.palette.splines[index].steps.iter().filter_map(|slot| registrar.palette.colours.get(slot)).map(|colour| colour.rgb).collect();
            registrar.picked.extend(shades);
        }
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

/// The splines in a group (`path`; "" for all of them) that `members` names, in the master
/// palette's order: those directly in it, and each group inside it folding away under its name.
fn spline_group(ui: &mut egui::Ui, registrar: &mut Registrar, path: &str, members: &[usize], full: bool, lit: Option<&BTreeSet<Rgb>>, action: &mut Option<SplineAction>) {
    // The groups one level down, in the order their first spline comes.
    let mut inner: Vec<String> = Vec::new();
    for &index in members {
        let group = registrar.palette.splines[index].group.as_str();
        let rest = if path.is_empty() { Some(group) } else { group.strip_prefix(path).and_then(|rest| rest.strip_prefix('/')) };
        if let Some(next) = rest.and_then(|rest| rest.split('/').next()).filter(|next| !next.is_empty())
            && !inner.iter().any(|name| name == next)
        {
            inner.push(next.to_string());
        }
    }
    for name in inner {
        let inside = if path.is_empty() { name.clone() } else { format!("{path}/{name}") };
        let in_it: Vec<usize> = members.iter().copied().filter(|&index| {
            let group = &registrar.palette.splines[index].group;
            group == &inside || group.starts_with(&format!("{inside}/"))
        }).collect();
        egui::CollapsingHeader::new(format!("{name}  ({} splines)", in_it.len())).id_salt(("spline group", &inside)).default_open(true).show(ui, |ui| {
            if ui.small_button("Pick all in this group").clicked() {
                *action = Some(SplineAction::PickMany(in_it.clone()));
            }
            spline_group(ui, registrar, &inside, &in_it, full, lit, action);
        });
    }
    let direct: Vec<usize> = members.iter().copied().filter(|&index| registrar.palette.splines[index].group == path).collect();
    for index in direct {
        spline_row(ui, registrar, index, full, lit, action);
    }
}

/// One spline: in the Splines view its controls (move up or down, name, group, pick, add or take off
/// the picked, delete), in Everything its name (click to pick its shades); then its shades.
fn spline_row(ui: &mut egui::Ui, registrar: &mut Registrar, index: usize, full: bool, lit: Option<&BTreeSet<Rgb>>, action: &mut Option<SplineAction>) {
    let count = registrar.palette.splines.len();
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        let picked = !registrar.picked.is_empty();
        let spline = &mut registrar.palette.splines[index];
        if !full {
            if ui.add(egui::Label::new(&spline.name).sense(egui::Sense::click())).on_hover_text("Click to pick its shades").clicked() {
                *action = Some(SplineAction::Pick(index));
            }
            ui.small(format!("{} shades", spline.steps.len()));
            return;
        }
        if ui.add_enabled(index > 0, egui::Button::new("▲").small()).on_hover_text("Earlier in the master palette").clicked() {
            *action = Some(SplineAction::Move(index, -1));
        }
        if ui.add_enabled(index + 1 < count, egui::Button::new("▼").small()).on_hover_text("Later in the master palette").clicked() {
            *action = Some(SplineAction::Move(index, 1));
        }
        if ui.add(egui::TextEdit::singleline(&mut spline.name).desired_width(110.0)).lost_focus() {
            *action = Some(SplineAction::Renamed);
        }
        ui.label("group:");
        if ui.add(egui::TextEdit::singleline(&mut spline.group).desired_width(80.0).hint_text("none")).on_hover_text("Its group, to fold it away with others; \"/\" nests one group in another").lost_focus() {
            spline.group = spline.group.split('/').map(str::trim).filter(|part| !part.is_empty()).collect::<Vec<_>>().join("/");
            *action = Some(SplineAction::Renamed);
        }
        ui.small(format!("{} shades · #{}", spline.steps.len(), spline.id)).on_hover_text("Its permanent number: blocks and shaders go by it, so its name, group and place can change freely");
        if ui.button("Pick").on_hover_text("Pick its shades").clicked() {
            *action = Some(SplineAction::Pick(index));
        }
        if ui.add_enabled(picked, egui::Button::new("Add the picked")).on_hover_text("Each goes in by lightness, between the shades darker and lighter").on_disabled_hover_text("Pick some colours first (click or Shift + drag them)").clicked() {
            *action = Some(SplineAction::Add(index));
        }
        if ui.add_enabled(picked, egui::Button::new("Take off the picked")).on_hover_text("Picked colours on this spline go back to not being on one").on_disabled_hover_text("Pick some of its colours first").clicked() {
            *action = Some(SplineAction::TakeOff(index));
        }
        if ui.button("Delete").on_hover_text("The spline goes; its colours stay in the palette, not on a spline").clicked() {
            *action = Some(SplineAction::Delete(index));
        }
    });
    let steps: Vec<(Rgb, u32)> = registrar.palette.splines[index].steps.iter().filter_map(|slot| registrar.palette.colours.get(slot)).map(|colour| (colour.rgb, 0)).collect();
    let event = swatches(ui, registrar, &steps, lit);
    registrar.take(ui, egui::Id::new(("spline colours", index)), event);
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
    Pick(usize),
    PickMany(Vec<usize>),
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

/// Before a change: copy `files` into a new step in `folder`, with what the change is. Each step
/// holds `what.txt`, `files.txt` (each copy's number and where it came from, or "new" for a file
/// that isn't there yet, which the change makes) and the copies.
fn save_undo_step(folder: &Path, what: &str, files: &[PathBuf]) -> Result<(), String> {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_millis());
    let step = folder.join(format!("{stamp:015}"));
    std::fs::create_dir_all(&step).map_err(|error| error.to_string())?;
    let mut list = String::new();
    for (k, file) in files.iter().enumerate() {
        if file.exists() {
            std::fs::copy(file, step.join(format!("{k}.copy"))).map_err(|error| format!("couldn't copy {}: {error}", file.display()))?;
            list += &format!("{k}\t{}\n", file.display());
        } else {
            list += &format!("new\t{}\n", file.display());
        }
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

/// Undo the newest step: its copies put back, the files it made taken away, and the step removed.
/// Returns what was undone.
fn undo_last(folder: &Path) -> Result<String, String> {
    let (step, what) = undo_steps(folder).pop().ok_or("nothing to undo")?;
    let list = std::fs::read_to_string(step.join("files.txt")).map_err(|error| error.to_string())?;
    for line in list.lines() {
        let Some((k, path)) = line.split_once('\t') else { continue };
        if k == "new" {
            if Path::new(path).exists() {
                std::fs::remove_file(path).map_err(|error| format!("couldn't take away {path}: {error}"))?;
            }
            continue;
        }
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
        // A file the change makes is taken away again.
        let made = folder.join("limonite.png");
        save_undo_step(&undo, "made", std::slice::from_ref(&made)).unwrap();
        std::fs::write(&made, "new art").unwrap();
        assert_eq!(undo_last(&undo).unwrap(), "made");
        assert!(!made.exists());
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

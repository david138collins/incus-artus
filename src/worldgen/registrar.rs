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
                    Ok(pixels) => Texture { colours: master_palette::colours_in(&pixels), image: None, pixels, problem: None },
                    Err(problem) => Texture { colours: Vec::new(), image: None, pixels: Vec::new(), problem: Some(problem) },
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
    registrar.hovered = None;
    let defs: Vec<&BlockDef> = registry.defs().iter().filter(|def| shown(def)).collect();
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
            texture_image(ui, registrar, top, 20.0);
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
fn texture_image(ui: &mut egui::Ui, registrar: &mut Registrar, name: &str, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let Some(texture) = registrar.textures.get_mut(name) else {
        return response;
    };
    if texture.problem.is_some() {
        ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 255));
        return response.on_hover_text(texture.problem.clone().unwrap_or_default());
    }
    let side = crate::render::textures::TEXTURE_SIZE as usize;
    let image = texture.image.get_or_insert_with(|| {
        ui.ctx().load_texture(format!("registrar {name}"), egui::ColorImage::from_rgba_unmultiplied([side, side], &texture.pixels), egui::TextureOptions::NEAREST)
    });
    ui.painter().image(image.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
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
                if texture_image(ui, registrar, name, 128.0).hovered() {
                    registrar.hovered = Some(def.name.clone());
                }
                let all_same = def.faces.iter().all(|face| face == &def.faces[0]);
                ui.small(format!("{} ({name})", if all_same { "all faces" } else { label }));
            });
        }
    });
    ui.add_space(8.0);
    let colours = registrar.block_colours(def);
    ui.label(format!("Its palette: {} colours", colours.len()));
    let list: Vec<(Rgb, u32)> = sorted_by_lightness(colours);
    let event = swatches(ui, registrar, &list, None);
    registrar.take(event);
}

// ── The palettes ──────────────────────────────────────────────────────────

/// What happened to a row of colour squares: the one under the mouse, and whether it was clicked.
type Swatch = Option<(Rgb, bool)>;

impl Registrar {
    /// The colour under the mouse becomes the focus; a click picks it, or unpicks it.
    fn take(&mut self, event: Swatch) {
        if let Some((rgb, clicked)) = event {
            self.focus = Some(rgb);
            if clicked && !self.picked.remove(&rgb) {
                self.picked.insert(rgb);
            }
        }
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
        ui.label(format!("{} picked", registrar.picked.len())).on_hover_text("Click colours in any palette to pick them; click again to unpick");
        if ui.add_enabled(!registrar.picked.is_empty(), egui::Button::new("Clear")).clicked() {
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
        registrar.take(event);
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
    registrar.take(event);
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
    registrar.take(event);
    ui.horizontal(|ui| {
        if ui.button("Pick all of these").clicked() {
            registrar.picked.extend(list.iter().map(|&(rgb, _)| rgb));
        }
        ui.add_enabled(false, egui::Button::new("HSL transform…")).on_disabled_hover_text("Coming soon: shift these colours' hue, saturation and lightness");
    });
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
/// pink edge. Returns the one under the mouse, and whether it was clicked.
fn swatches(ui: &mut egui::Ui, registrar: &Registrar, colours: &[(Rgb, u32)], lit: Option<&BTreeSet<Rgb>>) -> Swatch {
    // Squares between these sizes, sized so a row fills the panel's width exactly [David: "a tad
    // smaller", "fit to size within a range"].
    const SMALLEST: f32 = 10.0;
    const LARGEST: f32 = 16.0;
    const GAP: f32 = 2.0;
    let across = ((ui.available_width() + GAP) / (LARGEST + GAP)).ceil().max(1.0);
    let size = ((ui.available_width() + GAP) / across - GAP).clamp(SMALLEST, LARGEST);
    let mut event = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(GAP, GAP);
        for &(rgb, count) in colours {
            let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
            let dim = lit.is_some_and(|lit| !lit.contains(&rgb));
            let colour = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            ui.painter().rect_filled(rect, 2.0, if dim { colour.gamma_multiply(0.15) } else { colour });
            if registrar.picked.contains(&rgb) {
                ui.painter().rect_stroke(rect, 2.0, Stroke::new(2.0, PICKED), egui::StrokeKind::Inside);
            } else if lit.is_some() && !dim {
                ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Inside);
            }
            if response.hovered() {
                event = Some((rgb, response.clicked()));
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

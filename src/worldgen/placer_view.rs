//! The placer seen in the World Compiler (stop 1 of `docs/placer.md`): the Mineralogy panel's
//! Fields section, where David makes fields from others, and the **field view**, the close-up
//! switched to black with lines on it and the field being made shaded in it [David, 2026-10-31: "i'll
//! definitely need to be able to switch the screen to black with lines on the map, with shading
//! showing the field i'm *making* from another field"].
//!
//! - **Any step:** the whole stack, the result after any layer, or one layer alone.
//! - **The lines:** the coast, the rivers' and lakes' edges, and contours every so many blocks.
//! - **The level:** fields that change down a column (depth below the surface, the block's Z) are
//!   seen at a level: by depth below the surface or by Z, one level or a range (a range shows the most
//!   anywhere in it). Ctrl + the wheel over the map steps it.
//! - **Under the mouse** (the right-hand panel): each layer's value, through its curve, and the result
//!   so far.

use super::lab::Frame;
use super::view::fold_section;
use crate::world::placer::{Column, Curve, FIELDS, Field, Layer, MadeField, Mode, Program, Reading, Source, Stage, Traced, Upto};
use crate::world::{CompiledMap, MineralogySettings, SEA_LEVEL, Spline, under_sea};
use bevy_egui::egui::{self, Color32, RichText};

/// Past the edge of the world (as the close-up has it).
const BEYOND: [u8; 3] = [10, 10, 14];
/// The lines' colours: the coast, the rivers' and lakes' edges, and the contours.
const COAST: [u8; 3] = [90, 210, 255];
const WATER: [u8; 3] = [40, 100, 255];
const CONTOUR: [u8; 3] = [190, 120, 50];
/// A range of levels is looked at in at most this many of them.
const MOST_LEVELS: usize = 32;

/// How a level is given: by depth below the surface, or by Z.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LevelBy {
    #[default]
    Depth,
    Z,
}

impl LevelBy {
    /// How shallow or low, and how deep or high, a level can be set.
    fn ends(self) -> (f32, f32) {
        match self {
            LevelBy::Depth => (0.0, 200.0),
            LevelBy::Z => (1.0, crate::world::HIGHEST),
        }
    }
}

/// The level the field view looks at: one (`from` = `to`), or a range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Level {
    pub by: LevelBy,
    pub from: f32,
    pub to: f32,
}

impl Default for Level {
    fn default() -> Self {
        Self { by: LevelBy::Depth, from: 0.0, to: 0.0 }
    }
}

impl Level {
    /// The Zs to look at in a column whose ground is at `ground`: the level's, under the ground (at
    /// most `MOST_LEVELS`, evenly spread); none if it's all above.
    fn zs(&self, ground: f32) -> Vec<f32> {
        let (low, high) = match self.by {
            LevelBy::Depth => (ground - self.from.max(self.to), ground - self.from.min(self.to).max(0.0)),
            LevelBy::Z => (self.from.min(self.to), self.from.max(self.to).min(ground)),
        };
        if high < low {
            return Vec::new();
        }
        let levels = (high - low) as usize + 1;
        let taken = levels.min(MOST_LEVELS);
        (0..taken).map(|k| if taken == 1 { high } else { (high - (high - low) * k as f32 / (taken - 1) as f32).round() }).collect()
    }

    fn describe(&self) -> String {
        let (a, b) = (self.from.min(self.to), self.from.max(self.to));
        match (self.by, a == b) {
            (LevelBy::Depth, true) if a == 0.0 => "at the surface".into(),
            (LevelBy::Depth, true) => format!("{a:.0} below the surface"),
            (LevelBy::Depth, false) => format!("the most from {a:.0} to {b:.0} below the surface"),
            (LevelBy::Z, true) => format!("at Z {a:.0}"),
            (LevelBy::Z, false) => format!("the most from Z {a:.0} to {b:.0}"),
        }
    }
}

/// What the field view shows (when any of it changes, it's painted again).
#[derive(Clone, Debug, PartialEq)]
pub struct FieldLook {
    /// Which made field, and which step of it.
    pub chosen: usize,
    pub upto: Upto,
    pub coast: bool,
    pub water: bool,
    pub contours: bool,
    pub contour_every: f32,
    pub level: Level,
}

impl Default for FieldLook {
    fn default() -> Self {
        Self { chosen: 0, upto: Upto::All, coast: true, water: true, contours: true, contour_every: 16.0, level: Level::default() }
    }
}

/// The field view's state in the World Compiler (not saved: the made fields themselves are, with the
/// preset).
#[derive(Default)]
pub struct FieldView {
    /// The close-up switched to black, showing the chosen made field.
    pub on: bool,
    pub look: FieldLook,
    /// The wheel's turn not yet making a whole level.
    wheel: f32,
    /// What's under the mouse (worked out with the close-up, shown in the right-hand panel).
    pub hover: Option<Hover>,
}

/// The field under the mouse: where, at which Z, and each layer there.
pub struct Hover {
    pub z: f32,
    pub value: f32,
    pub layers: Vec<Option<Traced>>,
}

impl FieldView {
    /// Ctrl + the wheel over the map steps the level (up: shallower, or higher), keeping a range's
    /// size; true if it did.
    pub fn wheel(&mut self, ui: &egui::Ui, over_map: bool) -> bool {
        if !self.on || !over_map {
            return false;
        }
        let turned: f32 = ui.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::MouseWheel { unit, delta, modifiers, .. } if modifiers.command => Some(match unit {
                        egui::MouseWheelUnit::Point => delta.y / 40.0,
                        egui::MouseWheelUnit::Line => delta.y,
                        egui::MouseWheelUnit::Page => delta.y * 10.0,
                    }),
                    _ => None,
                })
                .sum()
        });
        self.wheel += turned;
        let steps = self.wheel.trunc();
        if steps == 0.0 {
            return false;
        }
        self.wheel -= steps;
        let level = &mut self.look.level;
        let by = if level.by == LevelBy::Depth { -steps } else { steps };
        let (low, high) = (level.from.min(level.to), level.from.max(level.to));
        let (lowest, highest) = level.by.ends();
        let moved = by.max(lowest - low).min(highest - high);
        level.from += moved;
        level.to += moved;
        true
    }

    /// The made field shown (if there are any), and the program to read it.
    pub fn shown(&self, m: &MineralogySettings) -> Option<(usize, Program)> {
        (self.on && !m.fields.is_empty()).then(|| (self.look.chosen.min(m.fields.len() - 1), Program::new(&m.fields, m.crust)))
    }
}

/// The value at the field view's level in one column: the most of it anywhere in the level (`None`
/// where the level is all above the ground), and the Z it's at.
fn at_level(program: &Program, k: usize, upto: Upto, reading: &mut Reading, level: &Level) -> Option<(f32, f32)> {
    let zs = level.zs(reading.column.ground);
    let first = *zs.first()?;
    if !program.per_block(k) {
        return Some((program.value_upto(k, reading, first, upto), first));
    }
    zs.into_iter().map(|z| (program.value_upto(k, reading, z, upto), z)).max_by(|a, b| a.0.total_cmp(&b.0))
}

/// The column the field view reads at a spot: the ground there (from the close-up), its slope
/// across `step` blocks, the climate, and the water over it.
pub fn column_at(map: &CompiledMap, climate: &(dyn Fn(f32, f32, f32) -> Option<(f32, f32)> + Sync), north: f32, east: f32, ground: f32, slope: f32) -> Column {
    let water = if map.mutes.rivers { None } else { map.rivers.here(north, east, ground).water.filter(|&top| top as f32 > ground + 1.0) };
    Column { north, east, ground, slope, climate: climate(north, east, ground), water }
}

/// The field view's picture of a close-up: the field shaded black (its least) to white (its most),
/// black where the level has no ground, with the lines drawn over it.
pub fn field_picture(frame: &Frame, map: &CompiledMap, climate: &(dyn Fn(f32, f32, f32) -> Option<(f32, f32)> + Sync), crust: f32, program: &Program, look: &FieldLook) -> Vec<Color32> {
    let request = &frame.request;
    let (cols, rows, step) = (request.cols, request.rows, request.step);
    let k = look.chosen.min(program.count().saturating_sub(1));
    let (low, high) = program.range_upto(k, look.upto);
    let (low, high) = if high > low { (low, high) } else { (low - 0.5, low + 0.5) };
    let z = |col: usize, row: usize| frame.heights[row.min(rows - 1) * cols + col.min(cols - 1)];
    let mut pixels = vec![Color32::BLACK; cols * rows];
    // Each sample's water: 0 dry, 1 the sea, 2 a river's or lake's.
    let mut wet = vec![0u8; cols * rows];
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_each = rows.div_ceil(cores).max(1);
    std::thread::scope(|scope| {
        let each = rows_each * cols.max(1);
        for (band, (part, wet)) in pixels.chunks_mut(each).zip(wet.chunks_mut(each)).enumerate() {
            let z = &z;
            scope.spawn(move || {
                for (i, pixel) in part.iter_mut().enumerate() {
                    let (col, row) = (i % cols, band * rows_each + i / cols);
                    let here = z(col, row);
                    if here.is_nan() {
                        let [r, g, b] = BEYOND;
                        *pixel = Color32::from_rgb(r, g, b);
                        continue;
                    }
                    let north = request.north - (row as f32 + 0.5) * step;
                    let east = request.east + (col as f32 + 0.5) * step;
                    let or_here = |v: f32| if v.is_nan() { here } else { v };
                    let east_rise = (or_here(z(col + 1, row)) - or_here(z(col.saturating_sub(1), row))) / (2.0 * step);
                    let north_rise = (or_here(z(col, row.saturating_sub(1))) - or_here(z(col, row + 1))) / (2.0 * step);
                    let column = column_at(map, climate, north, east, here, east_rise.hypot(north_rise));
                    wet[i] = if under_sea(here) { 1 } else if column.water.is_some() { 2 } else { 0 };
                    let mut reading = Reading::new(map, column, crust);
                    if let Some((value, _)) = at_level(program, k, look.upto, &mut reading, &look.level) {
                        let grey = (((value - low) / (high - low)).clamp(0.0, 1.0) * 255.0).round() as u8;
                        *pixel = Color32::from_gray(grey);
                    }
                }
            });
        }
    });
    // The lines, where a sample and the one east or south of it differ.
    let tint = |pixel: &mut Color32, [r, g, b]: [u8; 3]| {
        let mix = |a: u8, b: u8| ((a as f32) * 0.25 + (b as f32) * 0.75) as u8;
        *pixel = Color32::from_rgb(mix(pixel.r(), r), mix(pixel.g(), g), mix(pixel.b(), b));
    };
    let every = look.contour_every.max(1.0);
    for row in 0..rows {
        for col in 0..cols {
            let i = row * cols + col;
            let here = z(col, row);
            if here.is_nan() {
                continue;
            }
            let beside = [(col + 1 < cols).then(|| i + 1), (row + 1 < rows).then(|| i + cols)];
            let differs = |test: &dyn Fn(usize) -> i32| beside.iter().flatten().any(|&j| !frame.heights[j].is_nan() && test(j) != test(i));
            if look.water && differs(&|j| (wet[j] == 2) as i32) {
                tint(&mut pixels[i], WATER);
            } else if look.coast && differs(&|j| (wet[j] == 1) as i32) {
                tint(&mut pixels[i], COAST);
            } else if look.contours && differs(&|j| (frame.heights[j] / every).floor() as i32) {
                tint(&mut pixels[i], CONTOUR);
            }
        }
    }
    pixels
}

/// Works out what's under the mouse at (`north`, `east`), the ground `z` there, for the readout.
pub fn hover(map: &CompiledMap, climate: &(dyn Fn(f32, f32, f32) -> Option<(f32, f32)> + Sync), crust: f32, program: &Program, look: &FieldLook, (north, east, z): (f32, f32, f32), step: f32) -> Option<Hover> {
    let k = look.chosen.min(program.count().checked_sub(1)?);
    let height = |n: f32, e: f32| map.height(n, e).map_or(z, f32::round);
    let slope = ((height(north, east + step) - height(north, east - step)) / (2.0 * step)).hypot((height(north + step, east) - height(north - step, east)) / (2.0 * step));
    let mut reading = Reading::new(map, column_at(map, climate, north, east, z, slope), crust);
    let (value, at) = at_level(program, k, look.upto, &mut reading, &look.level)?;
    Some(Hover { z: at, value, layers: program.trace(k, &mut reading, at) })
}

/// A source's name, as the layer's picker shows it.
fn source_name(source: &Source) -> String {
    match source {
        Source::Field { key } => Field::by_key(key).map_or_else(|| format!("? {key}"), |field| field.info().name.to_string()),
        Source::Noise { size, .. } => format!("noise ({size:.0} blocks)"),
        Source::Made { name } => format!("made: {name}"),
        Source::SoFar => "the result so far".into(),
        Source::Number { value } => format!("the number {value}"),
        Source::Unknown => "? (unknown)".into(),
    }
}

/// What a layer's buttons asked for, done after the layers are drawn.
enum Change {
    Up(usize),
    Down(usize),
    Remove(usize),
}

/// The Mineralogy panel's Fields section: the made fields, each a stack of layers, and the field
/// view's switches.
pub fn fields_section(ui: &mut egui::Ui, m: &mut MineralogySettings, view: &mut FieldView) {
    fold_section(ui, "Fields", |ui| {
        ui.small(
            "Make fields from the world's own (the climate, the ground, the rivers, the bake's maps…), to see them and, later, to \
             place blocks by them. A field is a stack of layers, run top to bottom: each layer reads a source, through a curve, \
             and is mixed into the result so far by a mode. Saved with the preset; nothing in the world changes yet.",
        );
        ui.checkbox(&mut view.on, "field view").on_hover_text(
            "Switch the map to black, with the coast, the rivers and the contours drawn on it, and the chosen field shaded in it:\n\
             black where it's least, white where it's most. The whole stack, or any step of it (each layer's buttons).",
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (k, made) in m.fields.iter().enumerate() {
                if ui.selectable_label(view.look.chosen == k, &made.name).clicked() {
                    (view.look.chosen, view.look.upto) = (k, Upto::All);
                }
            }
            if ui.small_button("+ new field").on_hover_text("A new field, at the end of the list (it can read any above it).").clicked() {
                let mut name = "a new field".to_string();
                for n in 2.. {
                    if !m.fields.iter().any(|made| made.name == name) {
                        break;
                    }
                    name = format!("a new field {n}");
                }
                m.fields.push(MadeField { name, ..MadeField::default() });
                (view.look.chosen, view.look.upto, view.on) = (m.fields.len() - 1, Upto::All, true);
            }
        });
        if m.fields.is_empty() {
            ui.small("No fields yet.");
        } else {
            made_field(ui, m, view);
        }
        ui.add_space(6.0);
        ui.strong("The view");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut view.look.coast, "coast");
            ui.checkbox(&mut view.look.water, "rivers and lakes");
            ui.checkbox(&mut view.look.contours, "contours");
        });
        ui.add_enabled(view.look.contours, crate::debug::Slider::new(&mut view.look.contour_every, 2.0..=64.0).text("contours every (blocks)"));
        ui.add_space(4.0);
        level_ui(ui, &mut view.look.level);
        ui.add(crate::debug::Slider::new(&mut m.crust, 30.0..=400.0).text("the crust's depth (blocks)")).on_hover_text(
            "How deep the crust goes: \"depth, as a share of the crust\" is 0 at the surface and 1 this far down, so what's placed by\n\
             it keeps its place whatever depth the world ends up with (about 130 to start, from the ores' cross-reference).",
        );
    });
}

/// The chosen made field: its name and buttons, then its layers.
fn made_field(ui: &mut egui::Ui, m: &mut MineralogySettings, view: &mut FieldView) {
    let k = view.look.chosen.min(m.fields.len() - 1);
    view.look.chosen = k;
    let program = Program::new(&m.fields, m.crust);
    ui.horizontal(|ui| {
        let old = m.fields[k].name.clone();
        ui.add(egui::TextEdit::singleline(&mut m.fields[k].name).desired_width(140.0)).on_hover_text("Its name (layers that read it follow a new name).");
        let new = m.fields[k].name.clone();
        if new != old {
            for made in m.fields.iter_mut() {
                for layer in &mut made.layers {
                    if let Source::Made { name } = &mut layer.source
                        && *name == old
                    {
                        *name = new.clone();
                    }
                }
            }
        }
        if ui.small_button("copy").clicked() {
            let mut copy = m.fields[k].clone();
            copy.name += " (copy)";
            m.fields.push(copy);
            view.look.chosen = m.fields.len() - 1;
        }
        if ui.small_button("▲").on_hover_text("Up the list (a field reads only those above it).").clicked() && k > 0 {
            m.fields.swap(k, k - 1);
            view.look.chosen = k - 1;
        }
        if ui.small_button("▼").clicked() && k + 1 < m.fields.len() {
            m.fields.swap(k, k + 1);
            view.look.chosen = k + 1;
        }
        if ui.small_button("delete").clicked() {
            m.fields.remove(k);
            view.look.chosen = k.saturating_sub(1);
        }
    });
    let Some(made) = m.fields.get(view.look.chosen) else { return };
    if view.look.chosen != k {
        return;
    }
    let (low, high) = program.range(k);
    ui.horizontal(|ui| {
        ui.radio_value(&mut view.look.upto, Upto::All, "show the whole stack");
        ui.small(format!("from {low:.2} to {high:.2}")).on_hover_text("The least and most it can be: the field view's black and white.");
    });
    let names: Vec<String> = m.fields.iter().map(|made| made.name.clone()).collect();
    let mut change = None;
    for n in 0..made.layers.len() {
        let layer = &mut m.fields[k].layers[n];
        ui.group(|ui| layer_ui(ui, layer, n, &program, k, &names, view, &mut change));
    }
    let layers = &mut m.fields[k].layers;
    match change {
        Some(Change::Up(n)) if n > 0 => layers.swap(n, n - 1),
        Some(Change::Down(n)) if n + 1 < layers.len() => layers.swap(n, n + 1),
        Some(Change::Remove(n)) => {
            layers.remove(n);
            view.look.upto = Upto::All;
        }
        _ => {}
    }
    if ui.small_button("+ add a layer").on_hover_text("A new layer at the bottom of the stack (it mixes into everything above it).").clicked() {
        layers.push(Layer::default());
    }
}

#[allow(clippy::too_many_arguments)]
fn layer_ui(ui: &mut egui::Ui, layer: &mut Layer, n: usize, program: &Program, k: usize, names: &[String], view: &mut FieldView, change: &mut Option<Change>) {
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut layer.on, format!("{}.", n + 1)).on_hover_text("Off: left out, to see what it does.");
        source_picker(ui, layer, n, k, names);
        if ui.small_button("▲").clicked() {
            *change = Some(Change::Up(n));
        }
        if ui.small_button("▼").clicked() {
            *change = Some(Change::Down(n));
        }
        if ui.small_button("✕").on_hover_text("Remove this layer.").clicked() {
            *change = Some(Change::Remove(n));
        }
    });
    let (low, high) = program.source_range(k, n);
    match &mut layer.source {
        Source::Noise { size, seed } => {
            ui.add(crate::debug::Slider::new(size, 4.0..=16_384.0).logarithmic(true).text("patches (blocks)"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(seed).prefix("seed "));
                ui.small("−1 to 1, its own pattern for each seed");
            });
        }
        Source::Number { value } => {
            ui.add(egui::DragValue::new(value).speed(0.01).prefix("the number "));
        }
        _ => {}
    }
    ui.horizontal(|ui| {
        ui.label("through");
        let shapes = [("as it is", 0), ("a step", 1), ("a band", 2), ("a free curve", 3)];
        let now = match layer.curve {
            Curve::AsIs => 0,
            Curve::Step { .. } => 1,
            Curve::Band { .. } => 2,
            Curve::Free { .. } => 3,
            Curve::Unknown => 4,
        };
        let middle = (low + high) / 2.0;
        egui::ComboBox::from_id_salt(("curve", k, n)).selected_text(shapes.get(now).map_or("? (unknown)", |s| s.0)).show_ui(ui, |ui| {
            for (name, shape) in shapes {
                if ui.selectable_label(now == shape, name).clicked() && now != shape {
                    layer.curve = match shape {
                        1 => Curve::Step { at: middle, above: true, soft: 0.0 },
                        2 => Curve::Band { middle, spread: (high - low) / 4.0 },
                        3 => Curve::Free { spline: Spline::default() },
                        _ => Curve::AsIs,
                    };
                }
            }
        });
    });
    let span = (high - low).max(1e-3);
    match &mut layer.curve {
        Curve::Step { at, above, soft } => {
            ui.horizontal(|ui| {
                ui.selectable_value(above, false, "1 below");
                ui.selectable_value(above, true, "1 above");
            });
            ui.add(crate::debug::Slider::new(at, low..=high).text("…this"));
            ui.add(crate::debug::Slider::new(soft, 0.0..=span / 4.0).text("fading over (either side)"));
        }
        Curve::Band { middle, spread } => {
            ui.add(crate::debug::Slider::new(middle, low..=high).text("1 around"));
            ui.add(crate::debug::Slider::new(spread, 0.0..=span / 2.0).text("…0 this far either side"));
        }
        Curve::Free { spline } => {
            let across = format!("{} {low:.1} … {high:.1}", source_name(&layer.source));
            super::compiler_view::spline_box(ui, &format!("placer curve {k} {n}"), spline, &Spline::default(), &across, "0 to 1");
        }
        _ => {}
    }
    ui.horizontal(|ui| {
        ui.label("mixed in by");
        egui::ComboBox::from_id_salt(("mode", k, n)).selected_text(layer.mode.name()).show_ui(ui, |ui| {
            for mode in Mode::ALL {
                ui.selectable_value(&mut layer.mode, mode, mode.name());
            }
        });
    });
    ui.add(crate::debug::Slider::new(&mut layer.strength, 0.0..=1.0).text("strength"));
    ui.horizontal(|ui| {
        ui.small("show:");
        ui.radio_value(&mut view.look.upto, Upto::After(n), "up to here").on_hover_text("The result after this layer.");
        ui.radio_value(&mut view.look.upto, Upto::Alone(n), "alone").on_hover_text("This layer by itself, through its curve, before it's mixed in.");
    });
    if let Some(problem) = program.problem(k, n) {
        ui.colored_label(Color32::from_rgb(230, 120, 90), problem);
    }
}

/// The picker for a layer's source: the table's fields by stage, then what Mineralogy has.
fn source_picker(ui: &mut egui::Ui, layer: &mut Layer, n: usize, k: usize, names: &[String]) {
    egui::ComboBox::from_id_salt(("source", k, n)).width(190.0).selected_text(source_name(&layer.source)).show_ui(ui, |ui| {
        for stage in Stage::ALL.into_iter().filter(|&stage| stage != Stage::Mineralogy) {
            ui.label(RichText::new(stage.name()).small().strong());
            for info in FIELDS.iter().filter(|info| info.stage == stage) {
                let chosen = matches!(&layer.source, Source::Field { key } if key == info.key);
                let unit = if info.unit.is_empty() { String::new() } else { format!(" ({})", info.unit) };
                if ui.selectable_label(chosen, format!("{}{unit}", info.name)).on_hover_text(info.about).clicked() {
                    layer.source = Source::Field { key: info.key.into() };
                }
            }
        }
        ui.label(RichText::new("Mineralogy (made here)").small().strong());
        if ui.selectable_label(matches!(layer.source, Source::Noise { .. }), "noise").on_hover_text("Seeded noise, −1 to 1, in patches of a size.").clicked() && !matches!(layer.source, Source::Noise { .. }) {
            layer.source = Source::Noise { size: 64.0, seed: 1 };
        }
        for (j, name) in names.iter().enumerate() {
            let chosen = matches!(&layer.source, Source::Made { name: shown } if shown == name);
            let response = ui.add_enabled(j < k, egui::Button::selectable(chosen, format!("made: {name}")));
            if j >= k {
                response.on_disabled_hover_text(if j == k { "This field itself." } else { "Comes after this one: a field reads only those above it." });
            } else if response.clicked() {
                layer.source = Source::Made { name: name.clone() };
            }
        }
        if ui.selectable_label(layer.source == Source::SoFar, "the result so far").on_hover_text("The layers above this one, together: for a curve over all of them.").clicked() {
            layer.source = Source::SoFar;
        }
        if ui.selectable_label(matches!(layer.source, Source::Number { .. }), "a number").clicked() && !matches!(layer.source, Source::Number { .. }) {
            layer.source = Source::Number { value: 1.0 };
        }
    });
}

/// The level fields that change down a column are seen at.
fn level_ui(ui: &mut egui::Ui, level: &mut Level) {
    ui.horizontal(|ui| {
        ui.label("Depth fields at:");
        let before = level.by;
        ui.selectable_value(&mut level.by, LevelBy::Depth, "depth below the surface");
        ui.selectable_value(&mut level.by, LevelBy::Z, "Z");
        if level.by != before {
            (level.from, level.to) = if level.by == LevelBy::Z { (SEA_LEVEL as f32, SEA_LEVEL as f32) } else { (0.0, 0.0) };
        }
    })
    .response
    .on_hover_text("Fields that change down a column (the block's Z, its depth below the surface) are seen at this level. Ctrl + the wheel over the map steps it.");
    let mut one = level.from == level.to;
    let (bottom, top) = level.by.ends();
    let name = if level.by == LevelBy::Depth { "blocks down" } else { "Z" };
    ui.add(crate::debug::Slider::new(&mut level.from, bottom..=top).integer().text(if one { name.to_string() } else { format!("from ({name})") }));
    if ui.checkbox(&mut one, "one level").on_hover_text("Off: a range, showing the most anywhere in it.").changed() && one {
        level.to = level.from;
    }
    if one {
        level.to = level.from;
    } else {
        ui.add(crate::debug::Slider::new(&mut level.to, bottom..=top).integer().text(format!("to ({name})")));
    }
    ui.small(level.describe());
}

/// The right-hand panel's part: the field under the mouse, layer by layer.
pub fn readout(ui: &mut egui::Ui, m: &MineralogySettings, view: &FieldView) {
    let Some(hover) = &view.hover else { return };
    let Some(made) = m.fields.get(view.look.chosen) else { return };
    ui.separator();
    ui.strong(format!("Field: {}", made.name));
    ui.label(format!("{:.3}", hover.value)).on_hover_text("What the field view shows here (at the step chosen).");
    ui.small(format!("at Z {:.0} ({})", hover.z, view.look.level.describe()));
    for (n, (layer, traced)) in made.layers.iter().zip(&hover.layers).enumerate() {
        let text = match traced {
            Some(t) => format!("{}. {}: {:.3} → {:.3}, {} → {:.3}", n + 1, source_name(&layer.source), t.source, t.curved, layer.mode.name(), t.so_far),
            None => format!("{}. {} (off)", n + 1, source_name(&layer.source)),
        };
        ui.small(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A close-up, a sample a block, of a map rising from the sea in the west to Z 254 in the east.
    fn close_up() -> (Arc<CompiledMap>, Frame) {
        let width = 8;
        let z: Vec<f32> = (0..width * width).map(|i| 100.0 + (i % width) as f32 * 22.0).collect();
        let map = Arc::new(CompiledMap::of_heights(width, 32.0, &z));
        let mut lab = super::super::lab::Lab::default();
        let recipe = lab.recipe(&map, &map.settings.clone());
        let view = Some(((60.0, -100.0), (-60.0, 100.0), 1.0));
        let started = std::time::Instant::now();
        while lab.frame.is_none() && started.elapsed().as_secs() < 10 {
            lab.drive(&recipe, view);
        }
        (map, lab.frame.take().expect("no close-up made"))
    }

    #[test]
    fn the_field_view_shades_the_field_on_black_with_the_coast_drawn_over_it() {
        let (map, frame) = close_up();
        // White where the ground is at Z 150 or higher, black below.
        let high = MadeField { name: "high".into(), layers: vec![Layer { curve: Curve::Step { at: 150.0, above: true, soft: 0.0 }, ..Layer::default() }] };
        let program = Program::new(&[high], 130.0);
        let no_climate = |_: f32, _: f32, _: f32| None;
        let look = FieldLook { contours: false, ..FieldLook::default() };
        let pixels = field_picture(&frame, &map, &no_climate, 130.0, &program, &look);
        let (mut white, mut black, mut lines) = (0, 0, 0);
        for (pixel, &z) in pixels.iter().zip(&frame.heights) {
            if *pixel == Color32::WHITE {
                assert!(z >= 150.0, "white at Z {z}");
                white += 1;
            } else if *pixel == Color32::BLACK {
                assert!(z < 150.0, "black at Z {z}");
                black += 1;
            } else {
                lines += 1;
            }
        }
        assert!(white > 1_000 && black > 1_000 && lines > 50, "{white} white, {black} black, {lines} on the coast");
        // A level above the ground everywhere: nothing to shade.
        let above = FieldLook { level: Level { by: LevelBy::Z, from: 300.0, to: 300.0 }, coast: false, water: false, contours: false, ..FieldLook::default() };
        assert!(field_picture(&frame, &map, &no_climate, 130.0, &program, &above).iter().all(|&pixel| pixel == Color32::BLACK));
    }

    #[test]
    fn a_range_of_levels_shows_the_most_anywhere_in_it() {
        // Ground at Z 150; a range from 10 to 100 below it is looked at in 32 levels, deepest last.
        let range = Level { by: LevelBy::Depth, from: 10.0, to: 100.0 };
        let zs = range.zs(150.0);
        assert_eq!((zs.len(), zs[0], zs[31]), (32, 140.0, 50.0));
        // By Z, only what's under the ground.
        assert_eq!(Level { by: LevelBy::Z, from: 140.0, to: 160.0 }.zs(150.0).first(), Some(&150.0));
        assert!(Level { by: LevelBy::Z, from: 160.0, to: 170.0 }.zs(150.0).is_empty());
        // Depth in a band round 40 down: the most in a range 0 to 100 down is the band's middle, 1.
        let map = CompiledMap::of_heights(8, 32.0, &[150.0; 64]);
        let band = MadeField { name: "band".into(), layers: vec![Layer { source: Source::Field { key: "ground/depth".into() }, curve: Curve::Band { middle: 40.0, spread: 20.0 }, ..Layer::default() }] };
        let program = Program::new(&[band], 130.0);
        let column = Column { north: 0.5, east: 0.5, ground: 150.0, slope: 0.0, climate: None, water: None };
        let mut reading = Reading::new(&map, column, 130.0);
        let (value, z) = at_level(&program, 0, Upto::All, &mut reading, &Level { by: LevelBy::Depth, from: 0.0, to: 100.0 }).unwrap();
        assert!(value > 0.99 && (z - 110.0).abs() <= 2.0, "{value} at Z {z}");
        assert_eq!(at_level(&program, 0, Upto::All, &mut reading, &Level::default()).unwrap().0, 0.0, "at the surface");
    }

    #[test]
    fn made_fields_are_saved_with_the_compiler_s_preset() {
        let mut settings = super::super::compiler::CompileSettings::default();
        let layers = vec![
            Layer { source: Source::Field { key: "climate/rain".into() }, curve: Curve::Step { at: 0.3, above: false, soft: 0.05 }, ..Layer::default() },
            Layer { source: Source::Noise { size: 64.0, seed: 2 }, curve: Curve::Free { spline: Spline::through(&[(0.0, 0.0), (0.4, 0.1), (1.0, 1.0)]) }, mode: Mode::Subtract, strength: 0.25, on: false },
        ];
        settings.interpreter.mineralogy.fields = vec![MadeField { name: "dry".into(), layers }, MadeField::default()];
        settings.interpreter.mineralogy.crust = 150.0;
        let text = toml::to_string_pretty(&settings).unwrap();
        let back: super::super::compiler::CompileSettings = toml::from_str(&text).unwrap();
        assert_eq!(back, settings);
        // A preset from before made fields reads with none, and the crust at 130.
        let old: super::super::compiler::CompileSettings = toml::from_str("[mineralogy]\nbeach = 1\n").unwrap();
        assert_eq!((old.interpreter.mineralogy.fields.len(), old.interpreter.mineralogy.crust, old.interpreter.mineralogy.beach), (0, 130.0, 1));
    }
}

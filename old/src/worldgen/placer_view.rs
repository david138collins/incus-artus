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
//!
//! And the **depth sweep** (stop 2) [David, 2026-10-31: "scroll through the z layers with the wheel or
//! a scroll bar that can select ranges or individual levels, in order to sweep like lidar and see where
//! the minerals, rocks, and ores are going"]:
//! - **The level bar** along the map's right edge: the level's two ends as handles, one level or a
//!   range, by Z or by depth below the surface; the wheel over it steps it. It sets the field view's
//!   level too.
//! - **The blocks at the level,** across the map, each in its colour: for a range, the first block from
//!   the top that isn't see-through (air and water always are, stone by a switch), so the soils and
//!   sands stand out, and later the rocks and ores. The world's own rule fills the columns
//!   (`Layers::block_at`).
//! - **Under the mouse:** the column's blocks, from its water or its surface down.
//!
//! And **what's placed** (stop 3): the Mineralogy section's distributors, each a list of candidate
//! blocks from the library (with their colours; one not registered greyed and named), each
//! candidate's column profile and its suitability as a stack of layers, and the chooser. Any
//! candidate's suitability can be shown in the field view, as a made field can, and **"why is this
//! sand?"**: under the mouse, every candidate's suitability there and which was chosen.

use super::lab::Frame;
use super::view::fold_section;
use crate::data::BlockRegistry;
use crate::world::placer::{Candidate, Chooser, Column, Curve, Distributor, FIELDS, Field, Layer, MadeField, Mode, Program, Puts, Reading, Source, Stage, Traced, Under, Upto, Wander};
use crate::world::{BlockId, CompiledMap, Layers, Mineralogy, MineralogySettings, SEA_LEVEL, Spline, TerrainBlocks, WORLD_BOTTOM, WORLD_HEIGHT, Why, under_sea, whole_column};
use egui::{Color32, RichText};

/// The climate at a spot (temperature and rain at north, east and a Z), or none.
pub type ClimateAt<'a> = dyn Fn(f32, f32, f32) -> Option<(f32, f32)> + Sync + 'a;

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
    /// How shallow or low, and how deep or high, a level can be set: the world's whole height, from
    /// the top (peaks reach about 900 [David]) to the bottom.
    fn ends(self) -> (f32, f32) {
        match self {
            LevelBy::Depth => (0.0, (WORLD_HEIGHT - 1 - WORLD_BOTTOM) as f32),
            LevelBy::Z => (WORLD_BOTTOM as f32, (WORLD_HEIGHT - 1) as f32),
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

    /// The level's lowest and highest Z, in whole blocks, in a column whose ground is at `ground`.
    fn span(&self, ground: i32) -> (i32, i32) {
        let (a, b) = (self.from.min(self.to).round() as i32, self.from.max(self.to).round() as i32);
        match self.by {
            LevelBy::Depth => (ground - b, ground - a),
            LevelBy::Z => (a, b),
        }
    }

    fn is_range(&self) -> bool {
        self.from != self.to
    }

    /// Where it is, in words.
    fn describe(&self) -> String {
        let (a, b) = (self.from.min(self.to), self.from.max(self.to));
        match (self.by, a == b) {
            (LevelBy::Depth, true) if a == 0.0 => "at the surface".into(),
            (LevelBy::Depth, true) => format!("{a:.0} below the surface"),
            (LevelBy::Depth, false) => format!("from {a:.0} to {b:.0} below the surface"),
            (LevelBy::Z, true) => format!("at Z {a:.0}"),
            (LevelBy::Z, false) => format!("from Z {a:.0} to {b:.0}"),
        }
    }

    /// Switches it to be given by depth or by Z, starting at the surface or at the top of the world
    /// (to sweep down from above the peaks [David]).
    fn set_by(&mut self, by: LevelBy) {
        if by != self.by {
            self.by = by;
            let start = if by == LevelBy::Z { self.by.ends().1 } else { 0.0 };
            (self.from, self.to) = (start, start);
        }
    }

    /// One end on the bar, in words.
    fn end(&self, value: f32) -> String {
        match self.by {
            LevelBy::Depth if value == 0.0 => "surface".into(),
            LevelBy::Depth => format!("{value:.0} down"),
            LevelBy::Z => format!("Z {value:.0}"),
        }
    }
}

/// What the map shows: the world, a made field (the field view), or the blocks at the level (the
/// depth sweep).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shows {
    #[default]
    World,
    Field,
    Blocks,
}

/// What the field view shows (when any of it changes, it's painted again).
#[derive(Clone, Debug, PartialEq)]
pub struct FieldLook {
    /// Which stack of layers (a made field, or after them a candidate's suitability, as
    /// `MineralogySettings::stacks` lists them), and which step of it.
    pub chosen: usize,
    pub upto: Upto,
    pub coast: bool,
    pub water: bool,
    pub contours: bool,
    pub contour_every: f32,
    pub level: Level,
    /// The depth sweep sees through stone and gabbro too (as through air and water always).
    pub see_stone: bool,
}

impl Default for FieldLook {
    fn default() -> Self {
        Self { chosen: 0, upto: Upto::All, coast: true, water: true, contours: true, contour_every: 16.0, level: Level::default(), see_stone: false }
    }
}

/// The field view's and the depth sweep's state in the World Compiler (not saved: the made fields
/// themselves are, with the preset).
#[derive(Default)]
pub struct FieldView {
    pub shows: Shows,
    pub look: FieldLook,
    /// The wheel's turn not yet making a whole level.
    wheel: f32,
    /// Which of the level bar's handles is being dragged.
    grip: Option<Grip>,
    /// What's under the mouse (worked out with the close-up, shown in the right-hand panel): the field,
    /// the column's blocks, and why it's made of what it is.
    pub hover: Option<Hover>,
    pub column: Option<ColumnHover>,
    pub why: Vec<Why>,
    /// The made field open in the Fields section, and the distributor in the Mineralogy section.
    open_field: usize,
    open_distributor: usize,
    /// What the last export or import did.
    share_note: String,
}

/// Which of the level bar's handles is being dragged: the left (the level's `from`), the right
/// (`to`), or both (the range moved, held this far below the level under the pointer).
#[derive(Clone, Copy)]
enum Grip {
    From,
    To,
    Both(f32),
}

/// What the depth sweep sees in a column: a block (and its Z), or only water, or only air.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seen {
    Block(BlockId, i32),
    Water,
    Nothing,
}

/// The column under the mouse, for the depth sweep's readout: its ground, what the sweep sees there,
/// and its blocks from the top down (named, with their highest and lowest Z).
pub struct ColumnHover {
    pub ground: i32,
    pub seen: String,
    pub runs: Vec<(String, i32, i32)>,
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
        if self.shows == Shows::World || !over_map {
            return false;
        }
        let turned = turned(ui, |modifiers| modifiers.command);
        self.turn(turned, 1.0)
    }

    /// Adds a turn of the wheel, and steps the level `each` levels for every whole notch so far.
    fn turn(&mut self, turned: f32, each: f32) -> bool {
        self.wheel += turned;
        let steps = self.wheel.trunc();
        if steps == 0.0 {
            return false;
        }
        self.wheel -= steps;
        self.step(steps * each);
        true
    }

    /// Moves the level `steps` levels (up: shallower, or higher), keeping a range's size, up to the
    /// ends.
    fn step(&mut self, steps: f32) {
        let level = &mut self.look.level;
        let by = if level.by == LevelBy::Depth { -steps } else { steps };
        let (low, high) = (level.from.min(level.to), level.from.max(level.to));
        let (lowest, highest) = level.by.ends();
        let moved = by.max(lowest - low).min(highest - high);
        level.from += moved;
        level.to += moved;
    }

    /// The stack of layers shown (if the map shows one): a made field, or a candidate's suitability.
    pub fn shown(&self, m: &MineralogySettings) -> Option<usize> {
        let count = stack_count(m);
        (self.shows == Shows::Field && count > 0).then(|| self.look.chosen.min(count - 1))
    }

    /// Shows stack `k`, at step `upto`, in the field view.
    fn show(&mut self, k: usize, upto: Upto) {
        (self.look.chosen, self.look.upto, self.shows) = (k, upto, Shows::Field);
    }

    fn showing(&self, k: usize, upto: Upto) -> bool {
        self.shows == Shows::Field && self.look.chosen == k && self.look.upto == upto
    }
}

/// How many stacks of layers there are: the made fields, and every candidate's.
fn stack_count(m: &MineralogySettings) -> usize {
    m.fields.len() + m.distributors.iter().map(|distributor| distributor.candidates.len()).sum::<usize>()
}

/// How far the wheel turned this frame (in notches, up positive), counting only turns with
/// `held` true of the keys held.
fn turned(ui: &egui::Ui, held: impl Fn(&egui::Modifiers) -> bool) -> f32 {
    ui.input(|input| {
        input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::MouseWheel { unit, delta, modifiers, .. } if held(modifiers) => {
                    // (Shift may turn the wheel sideways.)
                    let delta = if delta.y != 0.0 { delta.y } else { delta.x };
                    Some(match unit {
                        egui::MouseWheelUnit::Point => delta / 40.0,
                        egui::MouseWheelUnit::Line => delta,
                        egui::MouseWheelUnit::Page => delta * 10.0,
                    })
                }
                _ => None,
            })
            .sum()
    })
}

/// The level bar along the map's right edge (`map`, the map's area on screen): the level's two ends
/// as handles left and right of a track, the level between them filled. Drag a handle to move its
/// end (apart for a range, together for one level), drag between them to move the range, click to
/// bring the nearer end there; the wheel over it steps a level (Shift: 10). The label above it
/// switches between depth and Z. Only in the depth sweep [David]; the field view's level is set on
/// the panel, or with Ctrl + the wheel.
pub fn level_bar(ui: &mut egui::Ui, map: egui::Rect, view: &mut FieldView) {
    let track = egui::Rect::from_min_max(egui::pos2(map.right() - 30.0, map.top() + 40.0), egui::pos2(map.right() - 22.0, map.bottom() - 44.0));
    if track.height() < 60.0 || view.shows != Shows::Blocks {
        return;
    }
    let (lowest, highest) = view.look.level.by.ends();
    let (by, span) = (view.look.level.by, highest - lowest);
    // A level's place down the bar (0 at the top: the surface, or the highest Z), and back.
    let down = |value: f32| if by == LevelBy::Depth { (value - lowest) / span } else { (highest - value) / span };
    let y_of = |value: f32| track.top() + down(value).clamp(0.0, 1.0) * track.height();
    let at_y = |y: f32| {
        let t = ((y - track.top()) / track.height()).clamp(0.0, 1.0);
        (if by == LevelBy::Depth { lowest + t * span } else { highest - t * span }).round()
    };
    let painter = ui.painter_at(map);
    // What the mouse does to it: the switch above it, then the handles and the wheel.
    {
        let label = egui::Rect::from_center_size(egui::pos2(track.center().x - 6.0, track.top() - 24.0), egui::vec2(52.0, 16.0));
        let switch = ui.interact(label, ui.id().with("level bar by"), egui::Sense::click()).on_hover_text("Click: the level by depth below the surface, or by Z.");
        if switch.clicked() {
            let other = if by == LevelBy::Depth { LevelBy::Z } else { LevelBy::Depth };
            view.look.level.set_by(other);
            return;
        }
        let area = track.expand2(egui::vec2(14.0, 8.0));
        let response = ui.interact(area, ui.id().with("level bar"), egui::Sense::click_and_drag()).on_hover_text(
            "The level the map shows: drag the left or the right handle to move that end (apart for a range, together for one\n\
             level), drag between them to move the range, or click to bring the nearer end there. The wheel here steps it a\n\
             level (Shift: 10). Ctrl + the wheel over the map steps it too.",
        );
        let level = &mut view.look.level;
        if response.drag_started()
            && let Some(origin) = ui.input(|input| input.pointer.press_origin())
        {
            let (top, bottom) = (y_of(level.from).min(y_of(level.to)), y_of(level.from).max(y_of(level.to)));
            let between = (origin.x - track.center().x).abs() <= track.width() / 2.0 + 1.0 && origin.y > top + 4.0 && origin.y < bottom - 4.0;
            view.grip = Some(if between {
                Grip::Both(at_y(origin.y) - level.from)
            } else if origin.x < track.center().x {
                Grip::From
            } else {
                Grip::To
            });
        }
        if let Some(pointer) = response.interact_pointer_pos() {
            let value = at_y(pointer.y);
            match view.grip {
                Some(Grip::From) => level.from = value,
                Some(Grip::To) => level.to = value,
                Some(Grip::Both(held)) => {
                    // Kept whole, within the ends.
                    let size = level.to - level.from;
                    let from = (value - held).clamp(lowest.max(lowest - size), highest.min(highest - size));
                    (level.from, level.to) = (from, from + size);
                }
                None if response.clicked() => {
                    if (value - level.from).abs() <= (value - level.to).abs() {
                        level.from = value;
                    } else {
                        level.to = value;
                    }
                }
                None => {}
            }
        }
        if response.drag_stopped() {
            view.grip = None;
        }
        if response.hovered() {
            let shift = ui.input(|input| input.modifiers.shift);
            view.turn(turned(ui, |_| true), if shift { 10.0 } else { 1.0 });
        }
    }
    let level = view.look.level;
    let faint = Color32::from_gray(150);
    let shadowed = |at: egui::Pos2, align: egui::Align2, text: &str, colour: Color32| {
        let font = egui::FontId::proportional(11.0);
        painter.text(at + egui::vec2(1.0, 1.0), align, text, font.clone(), Color32::BLACK);
        painter.text(at, align, text, font, colour);
    };
    // The switch, the ends' names, the track, and the sea's level on it.
    shadowed(egui::pos2(track.center().x - 6.0, track.top() - 24.0), egui::Align2::CENTER_CENTER, if by == LevelBy::Depth { "by depth" } else { "by Z" }, Color32::WHITE);
    shadowed(egui::pos2(track.center().x - 6.0, track.top() - 8.0), egui::Align2::CENTER_BOTTOM, &level.end(if by == LevelBy::Depth { lowest } else { highest }), faint);
    shadowed(egui::pos2(track.center().x - 6.0, track.bottom() + 8.0), egui::Align2::CENTER_TOP, &level.end(if by == LevelBy::Depth { highest } else { lowest }), faint);
    painter.rect_filled(track, 3.0, Color32::from_gray(35));
    painter.rect_stroke(track, 3.0, egui::Stroke::new(1.0, Color32::from_gray(110)), egui::StrokeKind::Outside);
    if by == LevelBy::Z {
        let y = y_of(SEA_LEVEL as f32);
        let [r, g, b] = COAST;
        painter.line_segment([egui::pos2(track.left(), y), egui::pos2(track.right(), y)], egui::Stroke::new(1.0, Color32::from_rgb(r, g, b)));
    }
    // The level, the handles and where their ends are.
    let (a, b) = (y_of(level.from), y_of(level.to));
    let band = egui::Rect::from_min_max(egui::pos2(track.left(), a.min(b) - 1.0), egui::pos2(track.right(), a.max(b) + 1.0));
    let amber = Color32::from_rgb(240, 175, 60);
    painter.rect_filled(band, 1.0, amber);
    let handle = |y: f32, left: bool| {
        let (x, way) = if left { (track.left() - 2.0, -1.0) } else { (track.right() + 2.0, 1.0) };
        let points = vec![egui::pos2(x, y), egui::pos2(x + 9.0 * way, y - 6.0), egui::pos2(x + 9.0 * way, y + 6.0)];
        painter.add(egui::Shape::convex_polygon(points, amber, egui::Stroke::new(1.0, Color32::BLACK)));
    };
    handle(a, true);
    handle(b, false);
    shadowed(egui::pos2(track.left() - 14.0, a), egui::Align2::RIGHT_CENTER, &level.end(level.from), Color32::WHITE);
    if (a - b).abs() >= 12.0 {
        shadowed(egui::pos2(track.left() - 14.0, b), egui::Align2::RIGHT_CENTER, &level.end(level.to), Color32::WHITE);
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

/// A close-up's sample (the ground's `height` there from the close-up) as the world fills it, as a
/// column for mineralogy and the field view: its whole block, its ground rounded as the world rounds
/// it, the climate given there, the water over it, and its slope as every view measures it (when a
/// layer reads it) [the Metaphysics Engine: "1 always equals 1"; until 2026-11-01 the close-up cut
/// the height down where the world rounds it, and measured the slope across its own samples].
pub fn world_spot(map: &CompiledMap, climate: &ClimateAt, mineralogy: &Mineralogy, north: f32, east: f32, height: f32) -> Column {
    let water = if map.mutes.rivers { None } else { map.rivers.here(north, east, height).water };
    let (ground, water) = whole_column(height, water);
    let (x, y) = (north.floor() as i32, east.floor() as i32);
    let slope = if mineralogy.reads_slope { map.slope(x, y, ground) } else { 0.0 };
    let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
    Column { north, east, ground: ground as f32, slope, climate: climate(north, east, ground as f32), water }
}

/// The same, filled: its ground, what it's made of (mineralogy's choice in the climate given, a
/// river's or lake's bed under its water), and its water's top (0 for none).
pub fn world_column(map: &CompiledMap, climate: &ClimateAt, mineralogy: &Mineralogy, north: f32, east: f32, height: f32) -> (i32, Layers, i32) {
    let column = world_spot(map, climate, mineralogy, north, east, height);
    (column.ground as i32, mineralogy.layers(map, column), column.water.unwrap_or(0))
}

/// What the depth sweep sees in a column at the level: the first block from the top of it that isn't
/// see-through (air and water always are; stone and gabbro too, with `see_stone`); or, if there's
/// none, whether there was water.
fn seen(level: &Level, see_stone: bool, ground: i32, layers: &Layers, water: i32, blocks: &TerrainBlocks) -> Seen {
    let (low, high) = level.span(ground);
    let rock = |block: BlockId| block == blocks.stone || block == blocks.gabbro;
    // At the ground and under it, the column's own blocks (above it, only water and air).
    let mut z = high.min(ground);
    while z >= low {
        let block = layers.block_at(z, ground, water, blocks);
        if !(see_stone && rock(block)) {
            return Seen::Block(block, z);
        }
        if z < ground - layers.depth {
            // Stone from here all the way down.
            break;
        }
        z -= 1;
    }
    // Water fills the column from over the ground up to the sea's surface or a river's or lake's top.
    let wet_top = (SEA_LEVEL - 1).max(water - 1);
    if high.min(wet_top) >= low.max(ground + 1) { Seen::Water } else { Seen::Nothing }
}

/// The column's blocks from the top of its water (or its ground) down to the first stone under its
/// layers, a run of each: (block, highest Z, lowest Z).
fn column_runs(ground: i32, layers: &Layers, water: i32, blocks: &TerrainBlocks) -> Vec<(BlockId, i32, i32)> {
    let top = ground.max((SEA_LEVEL - 1).max(water - 1));
    let mut runs: Vec<(BlockId, i32, i32)> = Vec::new();
    for z in ((ground - layers.depth - 1).max(1)..=top).rev() {
        let block = layers.block_at(z, ground, water, blocks);
        match runs.last_mut() {
            Some(run) if run.0 == block => run.2 = z,
            _ => runs.push((block, z, z)),
        }
    }
    runs
}

/// The heights in a close-up at a column and row (clamped to it; `NaN` past the map).
fn height(frame: &Frame, col: usize, row: usize) -> f32 {
    let (cols, rows) = (frame.request.cols, frame.request.rows);
    frame.heights[row.min(rows - 1) * cols + col.min(cols - 1)]
}

/// A sample's colour, given its north, east and ground; and its water (0 dry, 1 the sea, 2 a river's
/// or lake's).
type Colour<'a> = dyn Fn(f32, f32, f32) -> (Color32, u8) + Sync + 'a;

/// A close-up's picture, on all the computer's cores: each sample coloured by `colour`, with the
/// view's lines drawn over it. Past the map is `BEYOND`.
fn picture(frame: &Frame, look: &FieldLook, colour: &Colour) -> Vec<Color32> {
    let request = &frame.request;
    let (cols, rows, step) = (request.cols, request.rows, request.step);
    let z = |col: usize, row: usize| height(frame, col, row);
    let mut pixels = vec![Color32::BLACK; cols * rows];
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
                    (*pixel, wet[i]) = colour(north, east, here);
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

/// The field view's picture of a close-up: stack `k` (a made field, or a candidate's suitability, with
/// what the distributors before its own chose) shaded black (its least) to white (its most), black
/// where the level has no ground, with the lines drawn over it.
pub fn field_picture(frame: &Frame, map: &CompiledMap, climate: &ClimateAt, mineralogy: &Mineralogy, k: usize, look: &FieldLook) -> Vec<Color32> {
    let program = mineralogy.program();
    let k = k.min(program.count().saturating_sub(1));
    let (low, high) = program.range_upto(k, look.upto);
    let (low, high) = if high > low { (low, high) } else { (low - 0.5, low + 0.5) };
    picture(frame, look, &|north, east, here| {
        let column = world_spot(map, climate, mineralogy, north, east, here);
        let wet = if under_sea(here) { 1 } else if column.water.is_some() { 2 } else { 0 };
        let mut reading = mineralogy.reading(map, column, k);
        let grey = at_level(program, k, look.upto, &mut reading, &look.level).map_or(Color32::BLACK, |(value, _)| Color32::from_gray((((value - low) / (high - low)).clamp(0.0, 1.0) * 255.0).round() as u8));
        (grey, wet)
    })
}

/// The depth sweep's picture of a close-up: at the level, the block each sample's column has there
/// in its colour (`colour`), for a range the first from the top that isn't see-through; dark blue
/// where only water was seen through, black where only air; with the lines drawn over it.
pub fn sweep_picture(frame: &Frame, map: &CompiledMap, climate: &ClimateAt, blocks: &TerrainBlocks, colour: &(dyn Fn(BlockId) -> Color32 + Sync), look: &FieldLook) -> Vec<Color32> {
    let water = colour(blocks.water);
    let deep = Color32::from_rgb(water.r() / 3, water.g() / 3, water.b() / 3);
    let mineralogy = map.mineralogy(blocks);
    picture(frame, look, &|north, east, here| {
        let (ground, layers, top) = world_column(map, climate, &mineralogy, north, east, here);
        let wet = if under_sea(here) { 1 } else if top > 0 { 2 } else { 0 };
        let pixel = match seen(&look.level, look.see_stone, ground, &layers, top, blocks) {
            Seen::Block(block, _) => colour(block),
            Seen::Water => deep,
            Seen::Nothing => Color32::BLACK,
        };
        (pixel, wet)
    })
}

/// The column under the mouse at (`north`, `east`), the ground's height `z` there, for the depth
/// sweep's readout (`name` names a block).
pub fn column_hover(map: &CompiledMap, climate: &ClimateAt, blocks: &TerrainBlocks, name: &dyn Fn(BlockId) -> String, look: &FieldLook, (north, east, z): (f32, f32, f32)) -> ColumnHover {
    let (ground, layers, water) = world_column(map, climate, &map.mineralogy(blocks), north, east, z);
    let seen = match seen(&look.level, look.see_stone, ground, &layers, water, blocks) {
        Seen::Block(block, at) if at == ground => format!("{} at Z {at} (the surface)", name(block)),
        Seen::Block(block, at) => format!("{} at Z {at} ({} down)", name(block), ground - at),
        Seen::Water => "only water".into(),
        Seen::Nothing => "nothing (only air, or only what's seen through)".into(),
    };
    let runs = column_runs(ground, &layers, water, blocks).into_iter().map(|(block, high, low)| (name(block), high, low)).collect();
    ColumnHover { ground, seen, runs }
}

/// Works out stack `k` under the mouse at (`north`, `east`), the ground `z` there, for the readout.
pub fn hover(map: &CompiledMap, climate: &ClimateAt, mineralogy: &Mineralogy, k: usize, look: &FieldLook, (north, east, z): (f32, f32, f32)) -> Option<Hover> {
    let program = mineralogy.program();
    let k = k.min(program.count().checked_sub(1)?);
    let mut reading = mineralogy.reading(map, world_spot(map, climate, mineralogy, north, east, z), k);
    let (value, at) = at_level(program, k, look.upto, &mut reading, &look.level)?;
    Some(Hover { z: at, value, layers: program.trace(k, &mut reading, at) })
}

/// Why the column under the mouse is made of what it is ("why is this sand?"): every distributor's
/// candidates' suitability there (`name` names a block), and which was chosen.
pub fn why(map: &CompiledMap, climate: &ClimateAt, mineralogy: &Mineralogy, name: &dyn Fn(&str) -> String, (north, east, z): (f32, f32, f32)) -> Vec<Why> {
    mineralogy.why(map, world_spot(map, climate, mineralogy, north, east, z), name)
}

/// A source's name, as the layer's picker shows it.
fn source_name(source: &Source) -> String {
    match source {
        Source::Field { key } => Field::by_key(key).map_or_else(|| format!("? {key}"), |field| field.info().name.to_string()),
        Source::Noise { size, .. } => format!("noise ({size:.0} blocks)"),
        Source::Made { name } => format!("made: {name}"),
        Source::SoFar => "the result so far".into(),
        Source::Number { value } => format!("the number {value}"),
        Source::Chosen { distributor, .. } => format!("chosen by {distributor}"),
        Source::Unknown => "? (unknown)".into(),
    }
}

/// What a layer's (or a candidate's, or an under-block's) buttons asked for, done after they're
/// drawn.
enum Change {
    Up(usize),
    Down(usize),
    Remove(usize),
}

impl Change {
    fn apply<T>(self, list: &mut Vec<T>) {
        match self {
            Change::Up(n) if n > 0 && n < list.len() => list.swap(n, n - 1),
            Change::Down(n) if n + 1 < list.len() => list.swap(n, n + 1),
            Change::Remove(n) if n < list.len() => {
                list.remove(n);
            }
            _ => {}
        }
    }
}

/// The ▲ ▼ ✕ buttons of item `n` in a list.
fn order_buttons(ui: &mut egui::Ui, n: usize, what: &str, change: &mut Option<Change>) {
    if ui.small_button("▲").clicked() {
        *change = Some(Change::Up(n));
    }
    if ui.small_button("▼").clicked() {
        *change = Some(Change::Down(n));
    }
    if ui.small_button("✕").on_hover_text(format!("Remove this {what}.")).clicked() {
        *change = Some(Change::Remove(n));
    }
}

/// The block library, for the candidates' pickers: the blocks, and each one's colour (its far
/// colour, once the close-up has made them).
pub struct Library<'a> {
    pub registry: &'a BlockRegistry,
    pub swatch: &'a dyn Fn(BlockId) -> Option<Color32>,
}

impl Library<'_> {
    /// A block's name as David sees it, or its name and "not registered".
    fn name(&self, block: &str) -> String {
        match self.registry.id(block).and_then(|id| self.registry.def(id)) {
            Some(def) => def.display_name.clone(),
            None => format!("{block} (not registered)"),
        }
    }

    /// A picker for a block by its name: every block in the library, with its colour. One that
    /// isn't registered shows greyed and named (and is skipped in the world).
    fn picker(&self, ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, block: &mut String) {
        let registered = self.registry.id(block);
        let shown = self.name(block);
        if let Some(colour) = registered.and_then(self.swatch) {
            ui.label(RichText::new("■").color(colour));
        }
        let text = if registered.is_some() { RichText::new(shown) } else { RichText::new(shown).color(Color32::GRAY).italics() };
        egui::ComboBox::from_id_salt(id).width(150.0).selected_text(text).show_ui(ui, |ui| {
            for (k, def) in self.registry.defs().iter().enumerate().skip(1) {
                ui.horizontal(|ui| {
                    match (self.swatch)(BlockId(k as u16)) {
                        Some(colour) => ui.label(RichText::new("■").color(colour)),
                        None => ui.label("□"),
                    };
                    if ui.selectable_label(*block == def.name, &def.display_name).on_hover_text(&def.name).clicked() {
                        *block = def.name.clone();
                    }
                });
            }
        })
        .response
        .on_hover_text(if registered.is_some() { block.clone() } else { format!("{block}: not in the library (yet), so it's skipped. Pick one that is, or make it in the Registrar.") });
    }
}

/// Where exported distributors and made fields are kept (`kind`: "distributors" or "fields"): plain
/// TOML files, one each, `presets/placer/<kind>/<name>.toml` (stop 4 of `docs/placer.md`).
fn shared_folder(kind: &str) -> std::path::PathBuf {
    crate::debug::project_folder().join("presets").join("placer").join(kind)
}

/// A name as a file's: letters, digits, `-` and `_` (spaces become `_`).
fn file_name(name: &str) -> String {
    let name: String = name.trim().chars().map(|c| if c == ' ' { '_' } else { c }).filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    if name.is_empty() { "unnamed".into() } else { name }
}

/// Writes a shared file (replacing one of the same name); says how it went.
fn export_shared(kind: &str, name: &str, shared: Option<crate::world::placer::Shared>) -> String {
    let Some(shared) = shared else { return "Nothing to export.".into() };
    let (folder, file) = (shared_folder(kind), file_name(name));
    let path = folder.join(format!("{file}.toml"));
    let replaced = path.exists();
    let written = std::fs::create_dir_all(&folder).map_err(|error| error.to_string()).and_then(|()| toml::to_string_pretty(&shared).map_err(|error| error.to_string())).and_then(|text| std::fs::write(&path, text).map_err(|error| error.to_string()));
    let carried = shared.fields.len() - usize::from(shared.distributors.is_empty());
    let with = if carried == 0 { String::new() } else { format!(", with the {carried} field{} it reads", if carried == 1 { "" } else { "s" }) };
    match written {
        Ok(()) => format!("Exported to presets/placer/{kind}/{file}.toml{with}{}.", if replaced { " (the old one of that name replaced)" } else { "" }),
        Err(error) => format!("Couldn't export it: {error}"),
    }
}

/// The section's import list: every file exported so far (picking one imports it, at the end), and
/// the folder they're in.
fn import_row(ui: &mut egui::Ui, kind: &str, what: &str, m: &mut MineralogySettings, view: &mut FieldView) {
    ui.horizontal(|ui| {
        let folder = shared_folder(kind);
        let mut picked = None;
        egui::ComboBox::from_id_salt(("import", kind)).selected_text(format!("import a {what}…")).show_ui(ui, |ui| {
            let mut files: Vec<String> = std::fs::read_dir(&folder).into_iter().flatten().flatten().map(|entry| entry.path()).filter(|path| path.extension().is_some_and(|extension| extension == "toml")).filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned())).collect();
            files.sort();
            if files.is_empty() {
                ui.label(format!("(no {kind} exported yet)"));
            }
            for file in files {
                if ui.selectable_label(false, &file).clicked() {
                    picked = Some(file);
                }
            }
        })
        .response
        .on_hover_text(format!("Brings in a {what} exported before (from presets/placer/{kind}), at the end of the list, with the fields it reads that this preset hasn't got."));
        if let Some(file) = picked {
            let read = std::fs::read_to_string(folder.join(format!("{file}.toml"))).map_err(|error| error.to_string()).and_then(|text| toml::from_str::<crate::world::placer::Shared>(&text).map_err(|error| error.to_string()));
            view.share_note = match read {
                Ok(shared) => {
                    let notes = crate::world::placer::import(m, shared);
                    (view.open_field, view.open_distributor) = (m.fields.len().saturating_sub(1), m.distributors.len().saturating_sub(1));
                    format!("From {file}: {}.", if notes.is_empty() { "nothing new".into() } else { notes.join("; ") })
                }
                Err(error) => format!("Couldn't read {file}: {error}"),
            };
        }
        if ui.small_button("folder").on_hover_text(format!("Opens presets/placer/{kind}, to copy files in or out.")).clicked()
            && let Err(error) = super::export::open_folder(&folder)
        {
            view.share_note = format!("Couldn't open the folder: {error}");
        }
    });
}

/// What a candidate's layers can read besides what a made field can: the block library (for "chosen
/// by"), and the distributors before its own.
struct ForCandidate<'a> {
    library: &'a Library<'a>,
    earlier: &'a [Distributor],
}

/// The Mineralogy panel's Fields section: the made fields, each a stack of layers, and the field
/// view's switches.
pub fn fields_section(ui: &mut egui::Ui, m: &mut MineralogySettings, view: &mut FieldView) {
    fold_section(ui, "Fields", |ui| {
        ui.small(
            "Make fields from the world's own (the climate, the ground, the rivers, the bake's maps…), to see them and to place \
             blocks by them (a candidate's layers below can read any of them). A field is a stack of layers, run top to bottom: \
             each layer reads a source, through a curve, and is mixed into the result so far by a mode. Saved with the preset.",
        );
        ui.horizontal_wrapped(|ui| {
            ui.label("The map shows:");
            ui.selectable_value(&mut view.shows, Shows::World, "the world").on_hover_text("The blocks on top, as the world will have them.");
            let field = ui.add_enabled(stack_count(m) > 0, egui::Button::selectable(view.shows == Shows::Field, "the field view"));
            if field.clicked() {
                view.shows = Shows::Field;
            }
            field
                .on_hover_text(
                    "The map black, with the coast, the rivers and the contours drawn on it, and the chosen field shaded in it:\n\
                     black where it's least, white where it's most. The whole stack, or any step of it (each layer's buttons). A\n\
                     candidate's suitability can be shown the same way (its \"show\" buttons in the Mineralogy section).",
                )
                .on_disabled_hover_text("Make a field first (+ new field).");
            ui.selectable_value(&mut view.shows, Shows::Blocks, "the depth sweep").on_hover_text(
                "The blocks at the level (the bar on the map's right edge), each in its colour, with the lines drawn over them.\n\
                 For a range, the first block from the top that isn't see-through (air and water always are, stone by a switch),\n\
                 so the soils and sands stand out, like a lidar sweep.",
            );
        });
        if let Some(k) = view.shown(m) {
            let stacks = m.stacks();
            ui.small(format!("The field view shows: {}", stacks.get(k).map_or("?", |(name, _)| name.as_str())));
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (k, made) in m.fields.iter().enumerate() {
                if ui.selectable_label(view.open_field == k, &made.name).on_hover_text("Open it (the field view shows it).").clicked() {
                    view.open_field = k;
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
                view.open_field = m.fields.len() - 1;
                view.show(m.fields.len() - 1, Upto::All);
            }
        });
        import_row(ui, "fields", "field", m, view);
        if !view.share_note.is_empty() {
            ui.small(&view.share_note);
        }
        if m.fields.is_empty() {
            ui.small("No fields yet.");
        } else {
            made_field(ui, m, view);
        }
        ui.add_space(6.0);
        ui.strong("The view");
        let on = view.shows != Shows::World;
        ui.add_enabled_ui(on, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut view.look.coast, "coast");
                ui.checkbox(&mut view.look.water, "rivers and lakes");
                ui.checkbox(&mut view.look.contours, "contours");
            });
            ui.add_enabled(view.look.contours, crate::debug::Slider::new(&mut view.look.contour_every, 2.0..=64.0).text("contours every (blocks)"));
            ui.add_enabled(view.shows == Shows::Blocks, egui::Checkbox::new(&mut view.look.see_stone, "see through stone")).on_hover_text(
                "The depth sweep sees through stone and gabbro too (as through air and water always), so a range shows the\n\
                 soils and sands in it.",
            );
            ui.add_space(4.0);
            level_ui(ui, &mut view.look.level);
        })
        .response
        .on_disabled_hover_text("For the field view and the depth sweep.");
        ui.add(crate::debug::Slider::new(&mut m.crust, 30.0..=400.0).text("the crust's depth (blocks)")).on_hover_text(
            "How deep the crust goes: \"depth, as a share of the crust\" is 0 at the surface and 1 this far down, so what's placed by\n\
             it keeps its place whatever depth the world ends up with (about 130 to start, from the ores' cross-reference).",
        );
    });
}

/// The open made field: its name and buttons, then its layers.
fn made_field(ui: &mut egui::Ui, m: &mut MineralogySettings, view: &mut FieldView) {
    let k = view.open_field.min(m.fields.len() - 1);
    view.open_field = k;
    let program = Program::new(&m.fields, m.crust);
    let mut moved = None;
    ui.horizontal(|ui| {
        let old = m.fields[k].name.clone();
        ui.add(egui::TextEdit::singleline(&mut m.fields[k].name).desired_width(140.0)).on_hover_text("Its name (layers that read it follow a new name).");
        let new = m.fields[k].name.clone();
        if new != old {
            let candidates = m.distributors.iter_mut().flat_map(|distributor| distributor.candidates.iter_mut());
            let layers = m.fields.iter_mut().flat_map(|made| made.layers.iter_mut()).chain(candidates.flat_map(|candidate| candidate.layers.iter_mut()));
            for layer in layers {
                if let Source::Made { name } = &mut layer.source
                    && *name == old
                {
                    *name = new.clone();
                }
            }
        }
        if ui.small_button("copy").clicked() {
            let mut copy = m.fields[k].clone();
            copy.name += " (copy)";
            m.fields.push(copy);
            moved = Some(m.fields.len() - 1);
        }
        if ui.small_button("export").on_hover_text("Saves it to a file of its own (presets/placer/fields), with the fields it reads, to share or bring into another preset.").clicked() {
            view.share_note = export_shared("fields", &m.fields[k].name, crate::world::placer::export_field(m, k));
        }
        if ui.small_button("▲").on_hover_text("Up the list (a field reads only those above it).").clicked() && k > 0 {
            m.fields.swap(k, k - 1);
            moved = Some(k - 1);
        }
        if ui.small_button("▼").clicked() && k + 1 < m.fields.len() {
            m.fields.swap(k, k + 1);
            moved = Some(k + 1);
        }
        if ui.small_button("delete").clicked() {
            m.fields.remove(k);
            moved = Some(k.saturating_sub(1));
        }
    });
    if let Some(to) = moved {
        // The list changed: what the field view shows follows the open field.
        view.open_field = to;
        if view.shows == Shows::Field {
            view.show(to, Upto::All);
        }
        return;
    }
    let (low, high) = program.range(k);
    ui.horizontal(|ui| {
        if ui.radio(view.showing(k, Upto::All), "show the whole stack").clicked() {
            view.show(k, Upto::All);
        }
        ui.small(format!("from {low:.2} to {high:.2}")).on_hover_text("The least and most it can be: the field view's black and white.");
    });
    let names: Vec<String> = m.fields.iter().map(|made| made.name.clone()).collect();
    let mut change = None;
    for n in 0..m.fields[k].layers.len() {
        let layer = &mut m.fields[k].layers[n];
        ui.group(|ui| layer_ui(ui, layer, n, &program, k, &names, None, view, &mut change));
    }
    if let Some(change) = change {
        change.apply(&mut m.fields[k].layers);
        view.look.upto = Upto::All;
    }
    if ui.small_button("+ add a layer").on_hover_text("A new layer at the bottom of the stack (it mixes into everything above it).").clicked() {
        m.fields[k].layers.push(Layer::default());
    }
}

/// One layer of stack `k` (a made field's, or after them a candidate's).
#[allow(clippy::too_many_arguments)]
fn layer_ui(ui: &mut egui::Ui, layer: &mut Layer, n: usize, program: &Program, k: usize, names: &[String], candidate: Option<&ForCandidate>, view: &mut FieldView, change: &mut Option<Change>) {
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut layer.on, format!("{}.", n + 1)).on_hover_text("Off: left out, to see what it does.");
        source_picker(ui, layer, n, k, names, candidate);
        order_buttons(ui, n, "layer", change);
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
        Source::Chosen { blocks, .. } => {
            ui.small("1 where it chose one of these, 0 where it didn't:");
            let mut gone = None;
            for (b, block) in blocks.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if let Some(candidate) = candidate {
                        candidate.library.picker(ui, ("chosen block", k, n, b), block);
                    }
                    if ui.small_button("✕").clicked() {
                        gone = Some(b);
                    }
                });
            }
            if let Some(b) = gone {
                blocks.remove(b);
            }
            if ui.small_button("+ a block").clicked() {
                blocks.push("incus:stone".into());
            }
        }
        _ => {}
    }
    // Noise added to the source before its curve: a step's or band's edge wanders.
    let mut wanders = layer.wander.is_some();
    if ui.checkbox(&mut wanders, "wander").on_hover_text("Noise added to the source before its curve, so a step's or a band's edge wanders rather than following the\nclimate's pixels (as mineralogy's zones' edges always have).").changed() {
        layer.wander = wanders.then(Wander::default);
    }
    if let Some(wander) = &mut layer.wander {
        let span = (high - low).abs().max(0.01);
        ui.add(crate::debug::Slider::new(&mut wander.amount, 0.0..=span / 2.0).text("…by up to (either way)"));
        ui.add(crate::debug::Slider::new(&mut wander.size, 4.0..=16_384.0).logarithmic(true).text("…in patches (blocks)"));
        ui.add(egui::DragValue::new(&mut wander.seed).prefix("…seed "));
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
        if ui.radio(view.showing(k, Upto::After(n)), "up to here").on_hover_text("The result after this layer, in the field view.").clicked() {
            view.show(k, Upto::After(n));
        }
        if ui.radio(view.showing(k, Upto::Alone(n)), "alone").on_hover_text("This layer by itself, through its curve, before it's mixed in.").clicked() {
            view.show(k, Upto::Alone(n));
        }
    });
    if let Some(problem) = program.problem(k, n) {
        ui.colored_label(Color32::from_rgb(230, 120, 90), problem);
    }
}

/// The picker for a layer's source: the table's fields by stage, then what Mineralogy has (and, for a
/// candidate, what the distributors before its own chose).
fn source_picker(ui: &mut egui::Ui, layer: &mut Layer, n: usize, k: usize, names: &[String], candidate: Option<&ForCandidate>) {
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
        // What the distributors before this candidate's chose (greyed for a made field: they come
        // before any distributor).
        let earlier = candidate.map_or(&[][..], |candidate| candidate.earlier);
        if earlier.is_empty() {
            ui.add_enabled(false, egui::Button::selectable(false, "chosen by a distributor")).on_disabled_hover_text(if candidate.is_some() {
                "What a distributor before this one chose: this is the first."
            } else {
                "Only a distributor's candidates read what's chosen (made fields come before any distributor)."
            });
        }
        for distributor in earlier {
            let chosen = matches!(&layer.source, Source::Chosen { distributor: shown, .. } if *shown == distributor.name);
            if ui.selectable_label(chosen, format!("chosen by {}", distributor.name)).on_hover_text("1 where it chose one of the blocks you pick, 0 where it didn't (\"the rock is limestone or marble\").").clicked() && !chosen {
                let block = distributor.candidates.first().map_or_else(|| "incus:stone".to_string(), |candidate| candidate.block.clone());
                layer.source = Source::Chosen { distributor: distributor.name.clone(), blocks: vec![block] };
            }
        }
    });
}

/// The Mineralogy panel's Mineralogy section: what's placed, the distributors (stop 3 of
/// `docs/placer.md`), run in order, later ones over earlier ones.
pub fn distributors_section(ui: &mut egui::Ui, m: &mut MineralogySettings, view: &mut FieldView, registry: &BlockRegistry, swatch: &dyn Fn(BlockId) -> Option<Color32>) {
    let library = Library { registry, swatch };
    fold_section(ui, "Mineralogy", |ui| {
        ui.small(
            "What the ground is made of: distributors, run in order, each choosing one of its candidate blocks at every \
             column, later ones over earlier ones where they choose. A candidate's suitability is a stack of layers, as a \
             field is (multiply means \"and\"; no layers: everywhere). It puts down its block on top, then the blocks under \
             it, each so deep, then stone; or only on top, over what's there. A block that isn't registered is skipped.",
        );
        ui.horizontal_wrapped(|ui| {
            for (d, distributor) in m.distributors.iter().enumerate() {
                let text = if distributor.on { RichText::new(&distributor.name) } else { RichText::new(&distributor.name).color(Color32::GRAY) };
                if ui.selectable_label(view.open_distributor == d, text).clicked() {
                    view.open_distributor = d;
                }
            }
            if ui.small_button("+ new distributor").on_hover_text("A new distributor, at the end: over all the others, where it chooses.").clicked() {
                m.distributors.push(Distributor { name: unused_name(m, "a new distributor"), ..Distributor::default() });
                view.open_distributor = m.distributors.len() - 1;
            }
        });
        import_row(ui, "distributors", "distributor", m, view);
        if !view.share_note.is_empty() {
            ui.small(&view.share_note);
        }
        if m.distributors.is_empty() {
            ui.small("No distributors: the ground is all stone.");
        } else {
            distributor_ui(ui, m, view, &library);
        }
        ui.add_space(4.0);
        if ui.small_button("Back to the starting distributors").on_hover_text("The surface, the sea floor and river and lake beds, in David's numbers of 2026-10-31 (the made fields stay).").clicked() {
            m.distributors = crate::world::starting_distributors();
            view.open_distributor = 0;
        }
    });
}

/// A distributor name not yet used.
fn unused_name(m: &MineralogySettings, name: &str) -> String {
    let mut wanted = name.to_string();
    for n in 2.. {
        if !m.distributors.iter().any(|distributor| distributor.name == wanted) {
            break;
        }
        wanted = format!("{name} {n}");
    }
    wanted
}

/// The open distributor: its name and buttons, its chooser, then its candidates.
fn distributor_ui(ui: &mut egui::Ui, m: &mut MineralogySettings, view: &mut FieldView, library: &Library) {
    let d = view.open_distributor.min(m.distributors.len() - 1);
    view.open_distributor = d;
    let blocks = TerrainBlocks::from_registry(library.registry);
    let ready = Mineralogy::new(m, &blocks);
    let program = ready.program();
    // Where its candidates' stacks start (after the made fields and the distributors before it's).
    let first = m.fields.len() + m.distributors[..d].iter().map(|distributor| distributor.candidates.len()).sum::<usize>();
    let names: Vec<String> = m.fields.iter().map(|made| made.name.clone()).collect();
    let mut moved = None;
    ui.horizontal(|ui| {
        let old = m.distributors[d].name.clone();
        ui.add(egui::TextEdit::singleline(&mut m.distributors[d].name).desired_width(140.0)).on_hover_text("Its name (candidates that read what it chose follow a new name).");
        let new = m.distributors[d].name.clone();
        if new != old {
            for layer in m.distributors.iter_mut().flat_map(|distributor| distributor.candidates.iter_mut()).flat_map(|candidate| candidate.layers.iter_mut()) {
                if let Source::Chosen { distributor, .. } = &mut layer.source
                    && *distributor == old
                {
                    *distributor = new.clone();
                }
            }
        }
        ui.checkbox(&mut m.distributors[d].on, "on").on_hover_text("Off: left out, to see what it does.");
        if ui.small_button("copy").clicked() {
            let mut copy = m.distributors[d].clone();
            copy.name = unused_name(m, &format!("{} (copy)", copy.name));
            m.distributors.push(copy);
            moved = Some(m.distributors.len() - 1);
        }
        if ui.small_button("export").on_hover_text("Saves it to a file of its own (presets/placer/distributors), with the made fields its candidates read,\nto share or bring into another preset. (What it reads as \"chosen by\" another distributor isn't carried.)").clicked() {
            view.share_note = export_shared("distributors", &m.distributors[d].name, crate::world::placer::export_distributor(m, d));
        }
        if ui.small_button("▲").on_hover_text("Earlier: under the ones after it (a candidate reads only what the distributors before its own chose).").clicked() && d > 0 {
            m.distributors.swap(d, d - 1);
            moved = Some(d - 1);
        }
        if ui.small_button("▼").on_hover_text("Later: over the ones before it, where it chooses.").clicked() && d + 1 < m.distributors.len() {
            m.distributors.swap(d, d + 1);
            moved = Some(d + 1);
        }
        if ui.small_button("delete").clicked() {
            m.distributors.remove(d);
            moved = Some(d.saturating_sub(1));
        }
    });
    if let Some(to) = moved {
        view.open_distributor = to;
        return;
    }
    let (earlier, rest) = m.distributors.split_at_mut(d);
    let distributor = &mut rest[0];
    ui.horizontal(|ui| {
        ui.label("chooses");
        egui::ComboBox::from_id_salt(("chooser", d)).selected_text(distributor.chooser.name()).show_ui(ui, |ui| {
            let fits = match distributor.chooser {
                Chooser::FirstFit { at } => at,
                _ => 0.5,
            };
            for (chooser, about) in [
                (Chooser::FirstFit { at: fits }, "In order: the first whose suitability is at least so much (the order matters)."),
                (Chooser::Best, "The most suitable (the first of equals); nothing where all are 0."),
                (Chooser::Chance, "At random, weighted by suitability, the same at a spot every time; nothing where all are 0."),
            ] {
                if ui.selectable_label(std::mem::discriminant(&distributor.chooser) == std::mem::discriminant(&chooser), chooser.name()).on_hover_text(about).clicked() {
                    distributor.chooser = chooser;
                }
            }
        });
    });
    match &mut distributor.chooser {
        Chooser::FirstFit { at } => {
            ui.add(crate::debug::Slider::new(at, 0.0..=1.0).text("fits from (suitability)"));
        }
        Chooser::Unknown => {
            ui.colored_label(Color32::from_rgb(230, 120, 90), "A kind of chooser this game doesn't know (from a newer one?): nothing is chosen.");
        }
        _ => {}
    }
    ui.small("Where none is chosen, the column is left as the distributors before it made it.");
    let for_candidate = ForCandidate { library, earlier };
    let mut change = None;
    for (c, candidate) in distributor.candidates.iter_mut().enumerate() {
        let k = first + c;
        ui.group(|ui| candidate_ui(ui, candidate, (d, c, k), program, &names, &for_candidate, view, &mut change));
    }
    if let Some(change) = change {
        change.apply(&mut distributor.candidates);
        if view.shows == Shows::Field && view.look.chosen >= first {
            view.shows = Shows::World;
        }
    }
    if ui.small_button("+ add a candidate").on_hover_text("A new candidate at the end (stone, everywhere: give it a block and its layers).").clicked() {
        distributor.candidates.push(Candidate::default());
    }
}

/// One candidate: its block, what it puts down, and its suitability's layers.
#[allow(clippy::too_many_arguments)]
fn candidate_ui(ui: &mut egui::Ui, candidate: &mut Candidate, (d, c, k): (usize, usize, usize), program: &Program, names: &[String], for_candidate: &ForCandidate, view: &mut FieldView, change: &mut Option<Change>) {
    let library = for_candidate.library;
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut candidate.on, format!("{}.", c + 1)).on_hover_text("Off: skipped, to see what it does.");
        library.picker(ui, ("candidate", d, c), &mut candidate.block);
        order_buttons(ui, c, "candidate", change);
    });
    match &mut candidate.puts {
        Puts::Column { under, top_only } => {
            ui.checkbox(top_only, "only on top").on_hover_text("Puts only its block on top, over what the distributors before it put down (a river's bed over the soil).");
            ui.add_enabled_ui(!*top_only, |ui| {
                let mut gone = None;
                for (u, below) in under.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.small("then");
                        library.picker(ui, ("under", d, c, u), &mut below.block);
                        ui.add(egui::DragValue::new(&mut below.depth).range(0..=64).suffix(" deep"));
                        if ui.small_button("✕").clicked() {
                            gone = Some(u);
                        }
                    });
                }
                if let Some(u) = gone {
                    under.remove(u);
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(under.len() < crate::world::MOST_UNDER, egui::Button::new("+ a block under").small()).on_disabled_hover_text(format!("At most {} under the top, for now.", crate::world::MOST_UNDER)).clicked() {
                        under.push(Under::default());
                    }
                    ui.small("then stone");
                });
            });
        }
        Puts::Unknown => {
            ui.colored_label(Color32::GRAY, "Puts down a kind of thing this game doesn't know (from a newer one?): skipped.");
        }
    }
    let on = candidate.layers.iter().filter(|layer| layer.on).count();
    let summary = if on == 0 { "suits: everywhere (no layers)".to_string() } else { format!("suits: {on} layer{}", if on == 1 { "" } else { "s" }) };
    egui::CollapsingHeader::new(summary).id_salt(("candidate layers", d, c)).show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.radio(view.showing(k, Upto::All), "show its suitability").on_hover_text("In the field view: black where it doesn't suit, white where it does.").clicked() {
                view.show(k, Upto::All);
            }
            let (low, high) = program.range(k);
            if on > 0 {
                ui.small(format!("from {low:.2} to {high:.2}"));
            }
        });
        let mut layer_change = None;
        for (n, layer) in candidate.layers.iter_mut().enumerate() {
            ui.group(|ui| layer_ui(ui, layer, n, program, k, names, Some(for_candidate), view, &mut layer_change));
        }
        if let Some(layer_change) = layer_change {
            layer_change.apply(&mut candidate.layers);
            view.look.upto = Upto::All;
        }
        if ui.small_button("+ add a layer").on_hover_text("A new layer at the bottom of its stack (multiply: \"and\").").clicked() {
            candidate.layers.push(Layer { mode: if candidate.layers.is_empty() { Mode::Normal } else { Mode::Multiply }, ..Layer::default() });
        }
    });
}

/// The level fields that change down a column are seen at.
fn level_ui(ui: &mut egui::Ui, level: &mut Level) {
    ui.horizontal(|ui| {
        ui.label("The level:");
        let mut by = level.by;
        ui.selectable_value(&mut by, LevelBy::Depth, "depth below the surface");
        ui.selectable_value(&mut by, LevelBy::Z, "Z");
        level.set_by(by);
    })
    .response
    .on_hover_text(
        "The depth sweep shows the blocks at this level, and the field view fields that change down a column (the block's Z,\n\
         its depth below the surface) at it. Also set by the bar on the map's right edge (in the depth sweep), or Ctrl + the\n\
         wheel over the map.",
    );
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
    if level.is_range() {
        ui.small(format!("{}: a field shows the most anywhere in it, the depth sweep the first block from the top that isn't see-through", level.describe()));
    } else {
        ui.small(level.describe());
    }
}

/// The right-hand panel's part: the column's blocks (in the depth sweep), or the field under the mouse,
/// layer by layer (in the field view); then why it's made of what it is.
pub fn readout(ui: &mut egui::Ui, m: &MineralogySettings, view: &FieldView) {
    if view.shows == Shows::Blocks
        && let Some(column) = &view.column
    {
        let level = &view.look.level;
        ui.separator();
        ui.strong("The column");
        ui.label(format!("seen: {}", column.seen)).on_hover_text("What the depth sweep shows here.");
        ui.small(if level.is_range() { format!("(the first block from the top {} that isn't see-through)", level.describe()) } else { format!("({})", level.describe()) });
        let last = column.runs.len().saturating_sub(1);
        for (k, (name, high, low)) in column.runs.iter().enumerate() {
            let surface = if *high == column.ground { " (the surface)" } else { "" };
            let text = match (k == last, high == low) {
                (true, _) => format!("{name}: Z {high} and down{surface}"),
                (false, true) => format!("{name}: Z {high}{surface}"),
                (false, false) => format!("{name}: Z {high} to {low}, {} blocks{surface}", high - low + 1),
            };
            ui.small(text);
        }
    } else if let Some(hover) = &view.hover
        && let Some((name, layers)) = m.stacks().get(view.look.chosen).cloned()
    {
        let level = &view.look.level;
        ui.separator();
        ui.strong(if view.look.chosen < m.fields.len() { format!("Field: {name}") } else { format!("Suits: {name}") });
        ui.label(format!("{:.3}", hover.value)).on_hover_text("What the field view shows here (at the step chosen).");
        ui.small(format!("at Z {:.0} ({}{})", hover.z, if level.is_range() { "the most " } else { "" }, level.describe()));
        for (n, (layer, traced)) in layers.iter().zip(&hover.layers).enumerate() {
            let text = match traced {
                Some(t) => format!("{}. {}: {:.3} → {:.3}, {} → {:.3}", n + 1, source_name(&layer.source), t.source, t.curved, layer.mode.name(), t.so_far),
                None => format!("{}. {} (off)", n + 1, source_name(&layer.source)),
            };
            ui.small(text);
        }
    }
    why_readout(ui, &view.why);
}

/// "Why is this sand?": each distributor, what it chose under the mouse, and every candidate's
/// suitability there.
fn why_readout(ui: &mut egui::Ui, whys: &[Why]) {
    if whys.is_empty() {
        return;
    }
    ui.separator();
    ui.strong("Why it's made of this").on_hover_text("Each distributor (Mineralogy), in order: what it chose here, and how well each of its candidates suits here.");
    for why in whys {
        let chosen = why.chosen.and_then(|c| why.candidates.get(c)).map_or_else(|| "nothing (left as it was)".to_string(), |(name, _)| name.clone());
        let off = if why.on { "" } else { " (off)" };
        ui.label(format!("{}{off}: {chosen}", why.distributor));
        let at = match why.chooser {
            Chooser::FirstFit { at } => format!(" from {at:.2}"),
            _ => String::new(),
        };
        ui.small(format!("  {}{at}:", why.chooser.name()));
        for (c, (name, suits)) in why.candidates.iter().enumerate() {
            let mark = if why.chosen == Some(c) { "  ◀ chosen" } else { "" };
            let text = match suits {
                Some(suits) => format!("    {}. {name}: {suits:.2}{mark}", c + 1),
                None => format!("    {}. {name}: skipped", c + 1),
            };
            ui.small(text);
        }
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
        let settings = MineralogySettings { fields: vec![high], ..MineralogySettings::default() };
        let mineralogy = Mineralogy::new(&settings, &blocks());
        let no_climate = |_: f32, _: f32, _: f32| None;
        let look = FieldLook { contours: false, ..FieldLook::default() };
        let pixels = field_picture(&frame, &map, &no_climate, &mineralogy, 0, &look);
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
        assert!(field_picture(&frame, &map, &no_climate, &mineralogy, 0, &above).iter().all(|&pixel| pixel == Color32::BLACK));
        // A candidate's suitability, the same way: the starting surface's gabbro (from Z 333) is
        // nowhere on this low map, its grass (everywhere) all one grey.
        let gabbro = field_picture(&frame, &map, &no_climate, &mineralogy, 2, &above);
        assert!(gabbro.iter().all(|&pixel| pixel == Color32::BLACK));
        let surface = FieldLook { coast: false, water: false, contours: false, ..FieldLook::default() };
        let grass = field_picture(&frame, &map, &no_climate, &mineralogy, 7, &surface);
        assert!(grass.iter().zip(&frame.heights).all(|(&pixel, z)| z.is_nan() || pixel == grass[frame.heights.iter().position(|z| !z.is_nan()).unwrap()]));
    }

    #[test]
    fn why_is_this_sand_lists_every_candidate_and_the_one_chosen() {
        let (map, _) = close_up();
        let mineralogy = map.mineralogy(&blocks());
        let no_climate = |_: f32, _: f32, _: f32| None;
        let name = |block: &str| block.trim_start_matches("incus:").to_string();
        // On the beach (just above the sea), at Z 128: the surface is sand, the sea floor and the
        // beds choose nothing.
        let whys = why(&map, &no_climate, &mineralogy, &name, (0.0, 0.0, 128.0));
        assert_eq!(whys.len(), 3);
        let surface = &whys[0];
        assert_eq!((surface.distributor.as_str(), surface.chosen), ("the surface", Some(3)));
        assert_eq!(surface.candidates[3], ("sand".to_string(), Some(1.0)));
        assert_eq!(surface.candidates[6], ("grass".to_string(), Some(1.0)), "grass fits everywhere");
        assert_eq!((whys[1].chosen, whys[2].chosen), (None, None));
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

    /// Every block registered, numbered by hand (1 gabbro, 2 stone, 3 grass, 4 water, 5 dirt, 6 sand,
    /// 7 dry grass, 8 snowy ground, 9 gravel).
    fn blocks() -> TerrainBlocks {
        let names = ["incus:gabbro", "incus:stone", "incus:grass", "incus:water", "incus:dirt", "incus:sand", "incus:dry_grass", "incus:snowy_ground", "incus:gravel"];
        let names = names.iter().enumerate().map(|(k, name)| (name.to_string(), BlockId(k as u16 + 1))).collect();
        TerrainBlocks { gabbro: BlockId(1), stone: BlockId(2), grass: BlockId(3), water: BlockId(4), names: Arc::new(names) }
    }

    #[test]
    fn the_depth_sweep_sees_the_first_block_from_the_top_that_isn_t_see_through() {
        let b = blocks();
        let (grass, dirt, sand, gravel) = (b.grass, BlockId(5), BlockId(6), BlockId(9));
        let land = Layers::of(grass, [(dirt, 3)]);
        let depth = |from: f32, to: f32| Level { by: LevelBy::Depth, from, to };
        let z = |from: f32, to: f32| Level { by: LevelBy::Z, from, to };
        let on_land = |level: Level, see_stone: bool| seen(&level, see_stone, 150, &land, 0, &b);
        // One level: the surface, the dirt under it, the stone under that; air above.
        assert_eq!(on_land(depth(0.0, 0.0), false), Seen::Block(grass, 150));
        assert_eq!(on_land(depth(2.0, 2.0), false), Seen::Block(dirt, 148));
        assert_eq!(on_land(depth(10.0, 10.0), false), Seen::Block(b.stone, 140));
        assert_eq!(on_land(z(160.0, 160.0), false), Seen::Nothing);
        // A range: the first from its top.
        assert_eq!(on_land(z(140.0, 200.0), false), Seen::Block(grass, 150));
        assert_eq!(on_land(depth(2.0, 40.0), false), Seen::Block(dirt, 148));
        // Through stone: a range wholly in it shows nothing, and so does a mountainside of it.
        assert_eq!(on_land(depth(5.0, 60.0), false), Seen::Block(b.stone, 145));
        assert_eq!(on_land(depth(5.0, 60.0), true), Seen::Nothing);
        assert_eq!(on_land(depth(0.0, 60.0), true), Seen::Block(grass, 150));
        let mountain = Layers::rock(b.stone);
        assert_eq!(seen(&depth(0.0, 100.0), true, 250, &mountain, 0, &b), Seen::Nothing);
        // The sea floor: seen through the water, and only water over it.
        let floor = Layers::of(sand, [(sand, 3)]);
        assert_eq!(seen(&z(90.0, 140.0), false, 100, &floor, 0, &b), Seen::Block(sand, 100));
        assert_eq!(seen(&z(110.0, 140.0), false, 100, &floor, 0, &b), Seen::Water);
        // A river 2 deep over ground at 150 (its water below Z 153), on a bed of gravel.
        let river = land.with_top(gravel);
        assert_eq!(seen(&z(152.0, 152.0), false, 150, &river, 153, &b), Seen::Water);
        assert_eq!(seen(&z(153.0, 153.0), false, 150, &river, 153, &b), Seen::Nothing);
        assert_eq!(seen(&z(140.0, 160.0), false, 150, &river, 153, &b), Seen::Block(gravel, 150));
        // Its column, from the water's top down to the first stone.
        assert_eq!(column_runs(150, &river, 153, &b), vec![(b.water, 152, 151), (gravel, 150, 150), (dirt, 149, 147), (b.stone, 146, 146)]);
    }

    #[test]
    fn the_depth_sweep_fills_columns_as_the_world_does() {
        // The close-up (a sample a block) of a map rising from the sea in the west; no climate.
        let (map, frame) = close_up();
        let b = blocks();
        let no_climate = |_: f32, _: f32, _: f32| None;
        let (request, cols) = (&frame.request, frame.request.cols);
        assert_eq!(request.step, 1.0);
        let mut kinds = std::collections::HashSet::new();
        for (col, row) in [(10, 10), (60, 40), (110, 70), (150, 20), (190, 100), (250, 120)] {
            let (north, east) = (request.north - (row as f32 + 0.5), request.east + (col as f32 + 0.5));
            let (ground, layers, water) = world_column(&map, &no_climate, &map.mineralogy(&b), north, east, frame.heights[row * cols + col]);
            let (x, y) = (north.floor() as i32, east.floor() as i32);
            assert_eq!(map.column(x, y).map(|(z, _)| z), Some(ground));
            let chunk = map.chunk(crate::world::ChunkPos::of_block(crate::ecs::prelude::IVec3::new(x, y, 0)), &b, &crate::world::CavePlan::default()).unwrap();
            for z in ground - 8..=ground.max(SEA_LEVEL) + 4 {
                let block = layers.block_at(z, ground, water, &b);
                assert_eq!(block, chunk.get(crate::ecs::prelude::IVec3::new(x, y, z)), "at ({x}, {y}, Z {z})");
                kinds.insert(block);
            }
        }
        // Sea, sea floor, beach or grass, dirt, stone and air at least.
        assert!(kinds.len() >= 5, "{kinds:?}");
    }

    #[test]
    fn made_fields_are_saved_with_the_compiler_s_preset() {
        let mut settings = super::super::compiler::CompileSettings::default();
        let layers = vec![
            Layer { source: Source::Field { key: "climate/rain".into() }, curve: Curve::Step { at: 0.3, above: false, soft: 0.05 }, ..Layer::default() },
            Layer { source: Source::Noise { size: 64.0, seed: 2 }, curve: Curve::Free { spline: Spline::through(&[(0.0, 0.0), (0.4, 0.1), (1.0, 1.0)]) }, mode: Mode::Subtract, strength: 0.25, on: false, wander: Some(Wander { amount: 0.1, size: 200.0, seed: 7 }) },
        ];
        settings.interpreter.mineralogy.fields = vec![MadeField { name: "dry".into(), layers }, MadeField::default()];
        settings.interpreter.mineralogy.crust = 150.0;
        let text = toml::to_string_pretty(&settings).unwrap();
        let back: super::super::compiler::CompileSettings = toml::from_str(&text).unwrap();
        assert_eq!(back, settings);
        // A preset from before made fields and distributors reads with no fields, the crust at 130, and
        // the starting distributors (its old mineralogy settings let go).
        let old: super::super::compiler::CompileSettings = toml::from_str("[mineralogy]\nbeach = 1\n").unwrap();
        assert_eq!((old.interpreter.mineralogy.fields.len(), old.interpreter.mineralogy.crust), (0, 130.0));
        assert_eq!(old.interpreter.mineralogy.distributors, crate::world::starting_distributors(), "the first rules' settings are let go: the starting distributors");
    }
}

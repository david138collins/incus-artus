//! World Gen — the half of the Editor Suite where pixels mean something (plates, later ocean,
//! mountains, rivers) and processes run over them. The other half, the Graphics Kit, is the art
//! pipeline: numbers into colours and back, with no meaning attached.
//!
//! **One world seed.** Every World Gen step takes its own seed from it, so "New seeds for
//! everything" rolls one number and the whole world changes. Cycle through it to see whether the
//! settings routinely make good shapes.
//!
//! The first tool is **plates** (`plates.rs`); the second, **tectonics** (`tectonics.rs`), gives
//! them motion. Arrows show where each plate is heading and how it spins, and the boundaries are
//! coloured by what the motion would do there.
//!
//! ```text
//! ┌──────────── top: new seeds for everything ───────────────────┐
//! │ seed           │                          │ under the mouse │
//! │ plates         │        the world         │                 │
//! │  settings      │                          │ the plates      │
//! │  readouts      │                          │                 │
//! └────────────────┴──────────────────────────┴─────────────────┘
//! ```

use super::Suite;
use super::canvas::Camera;
use super::heights::{self, HeightSettings};
use super::palette::from_hsl;
use super::plates::{self, LineNoise, LineStyle, PlateSettings, Plates};
use super::runner::{Finished, Job, Poll};
use super::rags::{RagSettings, Sim, Spot};
use super::tectonics::{self, Boundary, Contact, Motion, MotionSettings};
use super::view::{MAGENTA, PINK, compass, seed_row};
use crate::debug::preset_bar;
use bevy_egui::egui::{self, Color32, Pos2, Vec2};
use serde::{Deserialize, Serialize};

/// Ocean inside the world (deep), shallow water, and the dark past its edge.
const OCEAN: [u8; 3] = [18, 38, 82];
const SHALLOW: [u8; 3] = [52, 104, 150];
/// Ocean floor squeezed up above the water: islands.
const ISLAND: [u8; 3] = [168, 160, 118];
const BEYOND: [u8; 3] = [10, 10, 14];
const VERTEX: Color32 = Color32::from_rgb(255, 220, 60);
/// The boundaries: red colliding, blue pulling apart, green sliding.
const COLLIDING: [u8; 3] = [235, 60, 55];
const PULLING_APART: [u8; 3] = [80, 170, 255];
const SLIDING: [u8; 3] = [70, 225, 110];

/// Everything the picture depends on: when any of it changes, it's painted again.
#[derive(Clone, PartialEq)]
struct Look {
    plates: u64,
    motions: u64,
    terrain: u64,
    steps: Option<u64>,
    show_cells: bool,
    show_boundaries: bool,
    height_shown: f32,
    smoothness: u32,
    tear_open: f32,
    squeeze: f32,
    water: f32,
}

pub struct WorldGen {
    /// The world seed. Each step takes its own seed from it.
    pub seed: u64,
    pub plates: PlateSettings,
    /// Show the deleted cells faintly in the ocean.
    pub show_cells: bool,
    /// Draw the vertices, and the straight edges as they were before wobbling, over the picture.
    pub show_vertices: bool,
    pub show_lines: bool,

    /// How the plates move (tectonics).
    pub motion: MotionSettings,
    pub show_arrows: bool,
    pub show_boundaries: bool,
    /// The arrows show where each plate would get to in this many steps.
    pub arrow_steps: f32,
    /// Each plate's motion and what it does at each boundary pixel, worked out again when the
    /// plates or the motion settings change.
    motions: Vec<Motion>,
    contacts: Vec<Option<Contact>>,
    motions_for: Option<(u64, u64, MotionSettings)>,
    motions_version: u64,

    /// The simulation: made when Start or Step is first pressed, thrown away by Reset or by any
    /// change to the plates or their motion.
    pub sim_settings: RagSettings,
    sim: Option<Sim>,
    /// The settings the rags were made with (when they change, the run starts again).
    sim_made_with: [u32; 3],
    running: bool,
    /// How fast to watch it. Only changes how many steps are taken each second, never the result.
    pub steps_per_second: f32,
    /// Steps owed since the last frame (part steps carry over).
    owed: f32,
    /// The simulation as last painted: what's on each pixel.
    moved: Vec<Option<Spot>>,

    /// Heights and the water level.
    pub heights: HeightSettings,
    /// The terrain of every pixel of the plates as made, worked out again when the plates or the
    /// height settings that shape it change.
    terrain: Vec<f32>,
    terrain_for: Option<(u64, u64, [u32; 8])>,
    terrain_version: u64,
    /// How much of the world was above water when last painted.
    land_share: f32,
    /// How bright the most raised ground shows (a height of this = white).
    pub height_shown: f32,
    /// How many times the shading is blended with its neighbours (only changes the look).
    pub shading_smoothness: u32,
    /// A cracked triangle stretched past this many times its size stops being drawn: a rift.
    pub tear_open: f32,
    pub show_bowl: bool,
    /// For automatic testing: start the simulation as soon as the plates are made.
    pub start_when_ready: bool,
    /// Your own settings, kept aside while an A/B button shows the defaults instead.
    ab_plates: Option<PlateSettings>,
    ab_tectonics: Option<(MotionSettings, RagSettings)>,
    ab_heights: Option<HeightSettings>,

    result: Option<Finished<Plates>>,
    job: Option<Job<Plates>>,
    seen: Option<(f32, f32, u64, PlateSettings)>,
    wanted: bool,
    version: u64,

    camera: Camera,
    texture: Option<egui::TextureHandle>,
    painted_for: Option<Look>,
    hover: Option<(usize, usize)>,
    seed_text: String,
}

impl Default for WorldGen {
    fn default() -> Self {
        Self {
            seed: crate::fate::derive_seed(super::fresh_number(), "world seed"),
            plates: PlateSettings::default(),
            show_cells: true,
            show_vertices: true,
            show_lines: false,
            motion: MotionSettings::default(),
            show_arrows: true,
            show_boundaries: true,
            arrow_steps: 100.0,
            motions: Vec::new(),
            contacts: Vec::new(),
            motions_for: None,
            motions_version: 0,
            sim_settings: RagSettings::default(),
            sim: None,
            sim_made_with: [0; 3],
            running: false,
            steps_per_second: 60.0,
            owed: 0.0,
            moved: Vec::new(),
            heights: HeightSettings::default(),
            terrain: Vec::new(),
            terrain_for: None,
            terrain_version: 0,
            land_share: 0.0,
            height_shown: 0.5,
            shading_smoothness: 3,
            tear_open: 2.0,
            show_bowl: true,
            start_when_ready: false,
            ab_plates: None,
            ab_tectonics: None,
            ab_heights: None,
            result: None,
            job: None,
            seen: None,
            wanted: true,
            version: 0,
            camera: Camera::default(),
            texture: None,
            painted_for: None,
            hover: None,
            seed_text: String::new(),
        }
    }
}

impl WorldGen {
    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    /// The plates step's own seed, taken from the world seed.
    fn plate_seed(&self) -> u64 {
        crate::fate::derive_seed(self.seed, "plates")
    }

    /// The tectonics step's own seed, taken from the world seed.
    fn motion_seed(&self) -> u64 {
        crate::fate::derive_seed(self.seed, "plate motion")
    }

    /// Works out the plates' motion and boundaries again if the plates or the motion settings
    /// changed. (Quick, so it's done here rather than in the background.)
    fn update_motions(&mut self) {
        let Some(result) = &self.result else { return };
        let now = (self.version, self.motion_seed(), self.motion.clone());
        if self.motions_for.as_ref() == Some(&now) {
            return;
        }
        self.motions = tectonics::motions(&self.motion, now.1, &result.made);
        self.contacts = tectonics::boundaries(&result.made, &self.motions);
        self.motions_for = Some(now);
        self.motions_version += 1;
        // New plates or new motions: the old run no longer applies.
        self.reset();
    }

    /// Works out the terrain again if the plates or the height settings that shape it changed.
    /// (The squeeze and the water level are applied as it's drawn, so they never need this.)
    fn update_terrain(&mut self) {
        let Some(result) = &self.result else { return };
        let seed = crate::fate::derive_seed(self.seed, "terrain");
        let now = (self.version, seed, self.heights.shaping());
        if self.terrain_for.as_ref() == Some(&now) {
            return;
        }
        self.terrain = heights::terrain(&result.made, &self.heights, seed);
        self.terrain_for = Some(now);
        self.terrain_version += 1;
    }

    /// Back to the start: the plates where they were made, with their starting motion.
    fn reset(&mut self) {
        self.sim = None;
        self.running = false;
        self.owed = 0.0;
    }

    /// The simulation, made from the plates and their motions if it isn't yet.
    fn sim(&mut self) -> Option<&mut Sim> {
        if self.sim.is_none() {
            let result = self.result.as_ref()?;
            let seed = crate::fate::derive_seed(self.seed, "rock stiffness");
            self.sim = Some(Sim::new(&result.made, &self.motions, &self.sim_settings, seed));
            self.sim_made_with = self.sim_settings.making();
        }
        self.sim.as_mut()
    }

    /// Whether the simulation has moved anything yet (if not, the plates show as made).
    fn has_moved(&self) -> bool {
        self.sim.as_ref().is_some_and(|sim| sim.steps > 0)
    }

    /// How plate `p` is moving: now, if the simulation has started, or its starting push.
    fn motion_of(&self, p: usize) -> Option<Motion> {
        match &self.sim {
            Some(sim) => Some(sim.motion(p)),
            None => self.motions.get(p).copied(),
        }
    }

    /// Where plate `p`'s middle is now, in field pixels.
    fn middle_of(&self, p: usize, made: &Plates) -> (f32, f32) {
        self.sim.as_ref().map_or(made.kept[p].middle, |sim| sim.middle(p))
    }

    /// Takes the steps owed for the time that passed, at the chosen speed. Stops when everything
    /// has coasted to a stop. Never spends more than a little time per frame, so the screen stays
    /// smooth (if it can't keep up, it just plays slower; the result is the same).
    fn run(&mut self, seconds: f32) {
        if !self.running {
            return;
        }
        self.owed += seconds * self.steps_per_second;
        let started = std::time::Instant::now();
        let settings = self.sim_settings.clone();
        let mut owed = self.owed;
        let Some(sim) = self.sim() else { return };
        let mut settled = false;
        while owed >= 1.0 {
            sim.step(&settings);
            owed -= 1.0;
            if sim.settled {
                settled = true;
                break;
            }
            if started.elapsed().as_secs_f32() > 0.012 {
                owed = owed.min(1.0);
                break;
            }
        }
        self.owed = owed;
        if settled {
            self.running = false;
        }
    }

    /// Notices changes, collects finished work, and starts the next when wanted (as layers do).
    pub fn drive(&mut self, size_blocks: f32, blocks_per_pixel: f32) {
        self.drive_plates(size_blocks, blocks_per_pixel);
        self.update_motions();
        self.update_terrain();
        // The rags are made differently now: start again.
        if self.sim.is_some() && self.sim_made_with != self.sim_settings.making() {
            self.reset();
        }
        if self.start_when_ready && self.result.is_some() {
            self.start_when_ready = false;
            self.running = self.sim().is_some();
        }
    }

    fn drive_plates(&mut self, size_blocks: f32, blocks_per_pixel: f32) {
        let now = (size_blocks, blocks_per_pixel, self.seed, self.plates.clone());
        if self.seen.as_ref() != Some(&now) {
            self.seen = Some(now);
            self.wanted = true;
        }
        if let Some(job) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.result = Some(*finished);
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting if self.wanted && job.started.elapsed().as_secs_f32() > 0.25 => {
                    job.stop();
                    self.job = None;
                }
                Poll::Waiting => return,
            }
        }
        if self.wanted {
            self.wanted = false;
            let (settings, seed) = (self.plates.clone(), self.plate_seed());
            self.job = Some(Job::start(move |stop| plates::make(&settings, seed, size_blocks, blocks_per_pixel, stop)));
        }
    }
}

/// A plate's colour. Land: hues spread around the wheel so neighbours differ. Ocean: dim blues,
/// each a little different.
fn plate_colour(plate: usize, land: bool) -> [u8; 3] {
    if land {
        from_hsl([(plate as f32 * 137.5) % 360.0, 0.45, 0.55])
    } else {
        let wobble = (plate as u32).wrapping_mul(2_654_435_761) >> 27; // 0 to 31
        from_hsl([200.0 + wobble as f32 * 1.3, 0.40, 0.24 + (wobble % 4) as f32 * 0.025])
    }
}

/// A boundary's colour.
fn boundary_colour(kind: Boundary) -> [u8; 3] {
    match kind {
        Boundary::Colliding => COLLIDING,
        Boundary::PullingApart => PULLING_APART,
        Boundary::Sliding => SLIDING,
    }
}

/// Degrees clockwise from north as a compass word, for example 135 → "SE".
fn bearing(degrees: f32) -> &'static str {
    const NAMES: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    NAMES[((degrees / 45.0).round() as usize) % 8]
}

/// A deleted cell, faintly, in the ocean.
fn cell_shade(cell: u16) -> [u8; 3] {
    let lift = (cell as u32).wrapping_mul(2_654_435_761) >> 29; // 0 to 7
    [OCEAN[0] + lift as u8, OCEAN[1] + lift as u8 * 2, OCEAN[2] + lift as u8 * 3]
}

pub fn draw(root: &mut egui::Ui, suite: &mut Suite) {
    let blocks = suite.blocks_per_pixel();
    let world = &mut suite.world;
    egui::Panel::top("world gen bar").show(root, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui
                .button("New seeds for everything")
                .on_hover_text("Rolls a new world seed. Every World Gen step takes its seed from it.\nPress it a few times to see whether these settings routinely make good shapes.")
                .clicked()
            {
                world.seed = crate::fate::derive_seed(super::fresh_number(), "world seed");
            }
            if let Some(result) = &world.result {
                ui.small(format!("made in {:.0} ms", result.millis));
            }
            ui.separator();
            playback(ui, world);
        });
    });
    world.run(root.input(|input| input.stable_dt).min(0.1));
    if world.running {
        root.ctx().request_repaint();
    }
    egui::Panel::left("world gen settings").resizable(true).default_size(340.0).size_range(220.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| settings(ui, world));
    });
    egui::Panel::right("world gen readouts").resizable(true).default_size(270.0).size_range(180.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| readouts(ui, world, blocks));
    });
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, world));
}

/// Start / Pause, Step, Reset, the step count and the watching speed.
fn playback(ui: &mut egui::Ui, world: &mut WorldGen) {
    ui.strong("Tectonics:");
    let label = if world.running { "Pause" } else if world.has_moved() { "Carry on" } else { "Start" };
    if ui.button(label).on_hover_text("Run the plates: each coasts on its starting push until drag stops it.").clicked() {
        world.running = !world.running && world.sim().is_some();
    }
    if ui.button("Step").on_hover_text("One step, then pause.").clicked() {
        world.running = false;
        let settings = world.sim_settings.clone();
        if let Some(sim) = world.sim() {
            sim.step(&settings);
        }
    }
    if ui.add_enabled(world.sim.is_some(), egui::Button::new("Reset")).on_hover_text("Back to the plates as made, with their starting push.").clicked() {
        world.reset();
    }
    match &world.sim {
        Some(sim) if sim.settled => ui.label(format!("step {}: everything has coasted to a stop", sim.steps)),
        Some(sim) => ui.label(format!("step {}", sim.steps)),
        None => ui.label("not started"),
    };
    ui.add(egui::Slider::new(&mut world.steps_per_second, 1.0..=2_000.0).logarithmic(true).text("steps per second")).on_hover_text(
        "How fast to watch. It only changes how many steps are taken each second:\nthe steps themselves, and so the result, are always the same.",
    );
    ui.separator();
    ui.add(egui::Slider::new(&mut world.heights.water, -1.0..=1.0).text("water level")).on_hover_text(
        "Below this height is sea. Change it at any time, running or paused:\nit only changes what's drawn as land, never the simulation.",
    );
    ui.label(format!("land: {:.0}% of the world", world.land_share * 100.0));
}

/// Everything on World Gen's left-hand panel except the seed, saved and loaded as one preset. (A
/// preset is a recipe: it works with any seed.)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct WorldGenPreset {
    plates: PlateSettings,
    motion: MotionSettings,
    rock: RagSettings,
    heights: HeightSettings,
    show: Show,
}

/// The display choices on the left-hand panel (they only change the look).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Show {
    cells: bool,
    vertices: bool,
    lines: bool,
    arrows: bool,
    boundaries: bool,
    bowl: bool,
    arrow_steps: f32,
    thickness_as_white: f32,
    shading_smoothness: u32,
    tears_open_past: f32,
}

impl Default for Show {
    fn default() -> Self {
        let world = WorldGen::default();
        world.show()
    }
}

impl WorldGen {
    fn show(&self) -> Show {
        Show {
            cells: self.show_cells,
            vertices: self.show_vertices,
            lines: self.show_lines,
            arrows: self.show_arrows,
            boundaries: self.show_boundaries,
            bowl: self.show_bowl,
            arrow_steps: self.arrow_steps,
            thickness_as_white: self.height_shown,
            shading_smoothness: self.shading_smoothness,
            tears_open_past: self.tear_open,
        }
    }

    fn preset(&self) -> WorldGenPreset {
        WorldGenPreset {
            plates: self.plates.clone(),
            motion: self.motion.clone(),
            rock: self.sim_settings.clone(),
            heights: self.heights.clone(),
            show: self.show(),
        }
    }

    fn use_preset(&mut self, preset: WorldGenPreset) {
        let show = preset.show;
        (self.plates, self.motion, self.sim_settings, self.heights) = (preset.plates, preset.motion, preset.rock, preset.heights);
        (self.show_cells, self.show_vertices, self.show_lines, self.show_arrows, self.show_boundaries, self.show_bowl) =
            (show.cells, show.vertices, show.lines, show.arrows, show.boundaries, show.bowl);
        (self.arrow_steps, self.height_shown, self.shading_smoothness, self.tear_open) =
            (show.arrow_steps, show.thickness_as_white, show.shading_smoothness, show.tears_open_past);
        // Anything being compared with the defaults is now replaced by the preset.
        (self.ab_plates, self.ab_tectonics, self.ab_heights) = (None, None, None);
    }
}

/// "Back to defaults", and an A/B button beside it that swaps between the defaults and your own
/// settings: the first press keeps yours aside and shows the defaults, the next puts yours back.
fn defaults_row<T: Default + Clone>(ui: &mut egui::Ui, what: &str, value: &mut T, kept: &mut Option<T>) {
    ui.horizontal_wrapped(|ui| {
        if ui.button(format!("{what} back to defaults")).clicked() {
            *value = T::default();
            *kept = None;
        }
        let comparing = kept.is_some();
        let label = if comparing { "A/B: back to mine" } else { "A/B: try the defaults" };
        if ui
            .selectable_label(comparing, label)
            .on_hover_text(
                "Flip between the defaults and your own settings, to see which differences matter.\n\
                 While the defaults are showing, changes you make to them are thrown away when you flip back.",
            )
            .clicked()
        {
            match kept.take() {
                Some(mine) => *value = mine,
                None => *kept = Some(std::mem::replace(value, T::default())),
            }
        }
    });
    if kept.is_some() {
        ui.colored_label(PINK, format!("Showing the default {}: press A/B again for yours.", what.to_lowercase()));
    }
}

fn settings(ui: &mut egui::Ui, world: &mut WorldGen) {
    ui.heading("Presets");
    ui.small(
        "Save everything on this panel (plates, tectonics, rock, heights and what's shown) as one preset. \
         The seed isn't saved with it, so a preset is a recipe that works with any seed.",
    );
    let mut preset = world.preset();
    let before = preset.clone();
    preset_bar(ui, "worldgen", &mut preset);
    if preset != before {
        world.use_preset(preset);
    }

    ui.add_space(6.0);
    ui.separator();
    ui.heading("World");
    ui.small("The world is the circle. The square around it is only there because pictures are square.");
    seed_row(ui, u32::MAX, &mut world.seed, &mut world.seed_text);

    ui.add_space(6.0);
    ui.separator();
    ui.heading("Plates");
    ui.small(
        "Voronoi cells over the field. Cells that reach past the edge of the world are deleted, and the ones \
         nearest the middle are kept as plates: the big pieces the continents will grow from.",
    );
    let plates = &mut world.plates;
    ui.add(egui::Slider::new(&mut plates.plates, 1..=20).text("plates"));
    let mut percent = plates.coverage * 100.0;
    if ui
        .add(egui::Slider::new(&mut percent, 5.0..=80.0).suffix("%").text("cover this much of the world"))
        .on_hover_text("Sets how big the cells are: 7 plates covering 30% means each is about 30% ÷ 7 of the world.\nWhat comes out is close, not exact (cells vary in size); the readout shows it.")
        .changed()
    {
        plates.coverage = percent / 100.0;
    }
    ui.add(egui::Slider::new(&mut plates.spread, 0..=10).text("spread evenly (rounds)")).on_hover_text(
        "Each round moves every point to the middle of its cell, then splits again.\n0 = as scattered (very uneven). More = rounder, more even plates.",
    );
    ui.small(format!("{} points scattered over the field.", plates.points()));
    ui.add(egui::Slider::new(&mut plates.ocean_rings, 0..=3).text("rings of ocean plates")).on_hover_text(
        "The cells touching the land plates become ocean plates (dim blue). They move too, and where they\nmeet land they make the coast types. More rings = more ocean plates further out.",
    );

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Wobbly edges");
    ui.small(
        "Each line between two cells is pushed sideways by two line mutators, added together. The push is \
         nothing at the vertices (where three cells meet), so the corners stay put and the lines still meet.",
    );
    line_noise(ui, "Large waves", &mut plates.waves);
    line_noise(ui, "Small roughness", &mut plates.rough);

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Show");
    ui.checkbox(&mut world.show_cells, "the deleted cells, faintly in the ocean");
    ui.checkbox(&mut world.show_vertices, "the vertices (yellow dots)");
    ui.checkbox(&mut world.show_lines, "the straight lines, before wobbling");
    ui.add_space(6.0);
    defaults_row(ui, "Plate settings", &mut world.plates, &mut world.ab_plates);

    ui.add_space(6.0);
    ui.separator();
    ui.heading("Tectonics");
    ui.small(
        "Each plate gets a speed, a direction and a spin, at random from the seed. Nothing moves yet: \
         the arrows show what each plate would do, and the boundaries are coloured by what that would \
         do there.",
    );
    let motion = &mut world.motion;
    ui.add(egui::Slider::new(&mut motion.slowest, 0.0..=200.0).text("slowest (blocks per step)"))
        .on_hover_text("Each plate's speed is picked at random between the slowest and the fastest.");
    ui.add(egui::Slider::new(&mut motion.fastest, 0.0..=200.0).text("fastest (blocks per step)"));
    ui.add(egui::Slider::new(&mut motion.ocean_speed, 0.0..=4.0).text("ocean plates' speed ×"))
        .on_hover_text("Ocean plates' speed is multiplied by this. Real ocean plates are often the faster ones.");
    ui.add(egui::Slider::new(&mut motion.spin, 0.0..=1.0).text("fastest spin (degrees per step)")).on_hover_text(
        "Each plate spins at random up to this fast, either way.\nA spinning plate's far side swings one way and its near side the other,\nso one boundary can collide at one end and pull apart at the other.",
    );
    ui.add(egui::Slider::new(&mut motion.inward, 0.0..=1.0).text("lean toward the middle"))
        .on_hover_text("0 = plates head any way at all.\n1 = every plate heads straight for the middle of the world (lots of collisions).");
    ui.add_space(4.0);
    ui.checkbox(&mut world.show_arrows, "arrows: heading and spin").on_hover_text(
        "Straight arrow: where the plate is heading, and how fast.\nCurved arrow: which way it spins. Three quarters of a circle = the fastest spin allowed;\nreal turns are too small to see. The exact numbers are in the table.",
    );
    if world.show_arrows {
        ui.add(egui::Slider::new(&mut world.arrow_steps, 10.0..=1_000.0).logarithmic(true).text("arrow length (steps)"))
            .on_hover_text("Each straight arrow reaches where the plate's middle would be after this many steps.");
    }
    ui.checkbox(&mut world.show_boundaries, "boundaries: red colliding, blue pulling apart, green sliding").on_hover_text(
        "Where two plates touch, how they move against each other there.\nMost boundaries do a bit of everything; each spot shows what it does most.\nPlates against empty ocean aren't boundaries.",
    );

    ui.add_space(6.0);
    ui.strong("The simulation");
    ui.small(
        "Start it in the bar along the top. Each plate is a heavy rag: a grid of points joined by springs. \
         Plates coast on their starting push, squash and bend where they meet, and slow down. Squeezed \
         rock is thicker (brighter); stretched rock is thinner (darker).",
    );
    let sim = &mut world.sim_settings;
    ui.add_space(4.0);
    ui.label("The rock (changing these starts the run again):");
    ui.add(egui::Slider::new(&mut sim.grain, 200.0..=3_000.0).logarithmic(true).text("grain (blocks)"))
        .on_hover_text("How far apart a plate's points are. Smaller = finer wrinkles and tears, but slower.");
    ui.add(egui::Slider::new(&mut sim.patchiness, 0.0..=1.0).text("patchiness")).on_hover_text(
        "How much the rock's stiffness and strength vary from place to place.\n0 = all the same: only the edges crumple. Higher = soft patches give way deep inside plates too.",
    );
    ui.add(egui::Slider::new(&mut sim.patch_size, 1_000.0..=40_000.0).logarithmic(true).text("patch size (blocks)"))
        .on_hover_text("How big the soft and stiff patches are.");
    ui.add_space(4.0);
    ui.label("How it behaves (can change while it runs; Reset replays):");
    ui.add(egui::Slider::new(&mut sim.stiffness, 0.0..=1.0).text("stiffness"))
        .on_hover_text("How firmly the springs pull back each pass.\nLow = soft, floppy rags. High = stiff sheets.");
    ui.add(egui::Slider::new(&mut sim.passes, 1..=30).text("passes per step")).on_hover_text(
        "How far a push travels through a plate each step: about one point per pass.\nFew = the far side lags far behind a hit (the plate crumples). Many = it feels the hit almost at once (stiffer, slower to work out).",
    );
    ui.add(egui::Slider::new(&mut sim.give_way, 0.0..=0.2).text("gives way at")).on_hover_text(
        "How far rock can be squeezed or stretched (0.02 = 2%) before it gives way for good.\nLow = it squashes and stays squashed easily. High = springy.",
    );
    ui.add(egui::Slider::new(&mut sim.creep, 0.0..=1.0).text("creep"))
        .on_hover_text("Once past giving way, how quickly the new shape becomes permanent.");
    ui.add(egui::Slider::new(&mut sim.crack, 0.05..=2.0).logarithmic(true).text("cracks at")).on_hover_text(
        "How far rock can be stretched (0.4 = 40% longer) before it snaps and the plate tears there.",
    );
    ui.add(egui::Slider::new(&mut sim.push, 0.0..=1.0).text("push back"))
        .on_hover_text("How firmly plates are kept from overlapping.\nLow = they crush into each other. High = hard edges.");
    ui.add(egui::Slider::new(&mut sim.friction, 0.0..=1.0).text("friction"))
        .on_hover_text("How much plates sliding past each other drag each other along.\nAt the edge, that shears and bends them.");
    ui.add(egui::Slider::new(&mut sim.drag, 0.0001..=0.1).logarithmic(true).text("drag"))
        .on_hover_text("How much speed every point loses each step. 0.01 = 1%.\nLower = they coast for longer.");
    ui.add(egui::Slider::new(&mut sim.ocean_weight, 0.05..=2.0).text("ocean rock's weight ×"))
        .on_hover_text("How heavy ocean rock is for its size, compared with land rock.\nLight plates get shoved and crumpled; heavy ones shove.");
    let mut bowl_percent = sim.bowl_start * 100.0;
    if ui
        .add(egui::Slider::new(&mut bowl_percent, 0.0..=99.0).suffix("%").text("bowl starts at"))
        .on_hover_text("The bowl keeps the plates from straying far from the middle (like a Beyblade arena).\nPast this share of the world's radius, the ground slopes up and pushes them back inward.")
        .changed()
    {
        sim.bowl_start = bowl_percent / 100.0;
    }
    ui.add(egui::Slider::new(&mut sim.bowl, 0.0..=20.0).text("bowl steepness"))
        .on_hover_text("How hard the bowl pushes back. The further out a plate goes, the harder.");
    ui.add(egui::Slider::new(&mut world.height_shown, 0.05..=3.0).logarithmic(true).text("thickness shown as white"))
        .on_hover_text(
            "Only changes the look. 0.5 = rock half as thick again shows white, and rock half as thin shows black.\nLower = small squeezes show strongly.",
        );
    ui.add(egui::Slider::new(&mut world.shading_smoothness, 0..=20).text("shading smoothness")).on_hover_text(
        "Only changes the look: how many times the shading is blended with its neighbours.\n0 = sharp (you can see the grid's triangles). Higher = smooth fields.",
    );
    ui.add(egui::Slider::new(&mut world.tear_open, 1.1..=6.0).logarithmic(true).text("tears open past")).on_hover_text(
        "Only changes the look. A crack pulled open less than this (× its size) is drawn as thin, stretched rock;\npulled open further, it's a rift and shows the sea. Too high = plates smeared across wide gaps.",
    );
    ui.checkbox(&mut world.show_bowl, "show where the bowl starts (a faint circle)");

    ui.add_space(6.0);
    // Motion and rock together, as one section.
    let mut tectonics = (world.motion.clone(), world.sim_settings.clone());
    defaults_row(ui, "Tectonics settings", &mut tectonics, &mut world.ab_tectonics);
    (world.motion, world.sim_settings) = tectonics;

    ui.add_space(6.0);
    ui.separator();
    ui.heading("Heights and water");
    ui.small(
        "Every spot on a plate has a height: its plate's base height, plus terrain noise, minus sunken edges, \
         plus how much it has been squeezed. Below the water level is sea. None of these change the \
         simulation, only what's land, so they can be changed at any time.",
    );
    let h = &mut world.heights;
    ui.add(egui::Slider::new(&mut h.land, -1.0..=1.0).text("land plates' base height"))
        .on_hover_text("Where land plates sit before the terrain noise. Around 0 (the usual water level) = about half land.");
    ui.add(egui::Slider::new(&mut h.ocean, -2.0..=0.0).text("ocean plates' base height"))
        .on_hover_text("Where ocean plates sit. Deep: only a hard squeeze lifts them above the water as islands.");
    ui.add_space(4.0);
    ui.label("Terrain noise (one for the whole world, riding with the plates):");
    ui.add(egui::Slider::new(&mut h.terrain, 0.0..=1.0).text("amount")).on_hover_text(
        "How high and low the rolling terrain goes. More = land plates broken into more islands and bays.",
    );
    ui.add(egui::Slider::new(&mut h.terrain_size, 1_000.0..=50_000.0).logarithmic(true).text("size (blocks)"))
        .on_hover_text("The size of the biggest hills and hollows. Big = whole regions above or below water.");
    ui.add(egui::Slider::new(&mut h.octaves, 1..=8).text("detail (octaves)")).on_hover_text("Finer, fainter layers on top: more ragged coasts.");
    ui.add(egui::Slider::new(&mut h.roughness, 0.0..=1.0).text("roughness")).on_hover_text("How strong each finer layer is.");
    ui.add_space(4.0);
    ui.label("Sunken edges (continental shelves):");
    ui.add(egui::Slider::new(&mut h.shelf_width, 0.0..=15_000.0).text("reach in (blocks)")).on_hover_text(
        "How far in from a plate's edge its ground starts sloping down.\nCoasts then fall inside plates, and plates touching at the start don't automatically join up.",
    );
    ui.add(egui::Slider::new(&mut h.shelf_depth, 0.0..=1.0).text("depth at the edge")).on_hover_text("How far the ground sinks at the very edge.");
    ui.add_space(4.0);
    ui.add(egui::Slider::new(&mut h.squeeze, 0.0..=3.0).text("squeeze lifts by")).on_hover_text(
        "How much squeezed rock rises (rock twice as thick rises by this much), and stretched rock sinks.\nThis is what turns crumple belts into mountains and ocean floor into islands.",
    );
    ui.add(egui::Slider::new(&mut h.water, -1.0..=1.0).text("water level")).on_hover_text("Below this height is sea. (Also in the bar along the top.)");
    ui.label(format!("land: {:.0}% of the world", world.land_share * 100.0));
    defaults_row(ui, "Height settings", &mut world.heights, &mut world.ab_heights);
}

/// One line mutator's controls.
fn line_noise(ui: &mut egui::Ui, title: &str, noise: &mut LineNoise) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut noise.on, "");
        ui.strong(title);
    });
    if !noise.on {
        return;
    }
    ui.horizontal(|ui| {
        ui.label("style:");
        ui.selectable_value(&mut noise.style, LineStyle::Perlin, "Perlin noise")
            .on_hover_text("Perlin noise running along the line. Smooth and rolling.");
        ui.selectable_value(&mut noise.style, LineStyle::Midpoint, "midpoint displacement").on_hover_text(
            "Halve the line and nudge the middle sideways at random; halve each half and nudge\ntheir middles by less; and so on down to the finest step. Jagged, like a coastline.",
        );
    });
    let midpoint = noise.style == LineStyle::Midpoint;
    ui.add(egui::Slider::new(&mut noise.amount, 0.0..=10_000.0).text("amount (blocks)")).on_hover_text(if midpoint {
        "The first nudge, at the middle of each line. The smaller nudges are fractions of it."
    } else {
        "How far a line may be pushed sideways.\nVery big pushes get cut off where they run into other lines."
    });
    if midpoint {
        ui.add(egui::Slider::new(&mut noise.size, 50.0..=20_000.0).logarithmic(true).text("finest step (blocks)"))
            .on_hover_text("The halving stops when the pieces are this short. Smaller = finer detail.");
        ui.add(egui::Slider::new(&mut noise.roughness, 0.0..=1.0).text("roughness"))
            .on_hover_text("Each smaller nudge is this much of the one before.\n0.5 = natural-looking. Higher = jaggeder, lower = smoother.");
    } else {
        ui.add(egui::Slider::new(&mut noise.size, 300.0..=40_000.0).logarithmic(true).text("wave size (blocks)"))
            .on_hover_text("How long the biggest wobbles are, along the line.");
        ui.add(egui::Slider::new(&mut noise.octaves, 1..=8).text("octaves")).on_hover_text("Finer, fainter wobbles stacked on the big ones.");
        ui.add(egui::Slider::new(&mut noise.roughness, 0.0..=1.0).text("roughness"))
            .on_hover_text("How strong each finer wobble is compared with the one before.");
    }
}

fn readouts(ui: &mut egui::Ui, world: &WorldGen, blocks_per_pixel: f32) {
    let Some(result) = &world.result else {
        ui.small("Being made…");
        return;
    };
    let made = &result.made;
    ui.heading("Under the mouse");
    match world.hover {
        Some((col, row)) if col < made.width && row < made.width => {
            let i = row * made.width + col;
            let half = made.width as f32 / 2.0;
            let north = (half - row as f32 - 0.5) * blocks_per_pixel;
            let east = (col as f32 + 0.5 - half) * blocks_per_pixel;
            ui.label(format!("Pixel {col}, {row}"));
            ui.label(format!("{}, {}", compass(north, "N", "S"), compass(east, "E", "W")));
            ui.label(if made.inside[i] { "inside the world" } else { "past the edge of the world" });
            if !world.has_moved() {
                ui.label(format!("cell {}", made.cell[i]));
            }
            match world.moved.get(i).copied().flatten() {
                Some(spot) => {
                    ui.label(plate_name(made, spot.plate as usize));
                    let squeeze = spot.squeeze;
                    let what = if squeeze > 0.01 { "squeezed: higher" } else if squeeze < -0.01 { "stretched: lower" } else { "as it was" };
                    ui.label(format!("rock {:.2}× as thick ({what})", squeeze + 1.0));
                    let terrain = world.terrain.get(spot.from as usize).copied().unwrap_or(0.0);
                    let height = terrain + world.heights.squeeze * squeeze;
                    let water = world.heights.water;
                    ui.label(if height > water {
                        format!("height {height:.2}: land, {:.2} above the water", height - water)
                    } else {
                        format!("height {height:.2}: sea, {:.2} deep", water - height)
                    });
                }
                None => {
                    ui.label("no plate (empty ocean)");
                }
            }
            if !world.has_moved()
                && let Some(Some(contact)) = world.contacts.get(i)
            {
                let what = match contact.kind {
                    Boundary::Colliding => "colliding",
                    Boundary::PullingApart => "pulling apart",
                    Boundary::Sliding => "sliding past each other",
                };
                ui.colored_label(colour32(boundary_colour(contact.kind)), format!("boundary: {what}"));
                ui.small(format!("between {} and {}", plate_name(made, contact.plates.0 as usize), plate_name(made, contact.plates.1 as usize)));
                ui.small(format!(
                    "{} {:.0} blocks per step, sliding {:.0}",
                    if contact.closing >= 0.0 { "closing" } else { "opening" },
                    contact.closing.abs(),
                    contact.sliding
                ));
            }
        }
        _ => {
            ui.small("Point at the world.");
        }
    }

    ui.separator();
    ui.heading("The plates");
    ui.label(format!("cover {:.1}% of the world (aiming for {:.0}%)", made.coverage * 100.0, world.plates.coverage * 100.0));
    ui.label(format!("{} cells, {} reached past the edge and were deleted", made.cells, made.past_the_edge));
    ui.label(format!("{} vertices and {} lines on the plates", made.vertices.len(), made.edges.len()))
        .on_hover_text("The ones out in the ocean are culled.");
    let land = made.kept.iter().filter(|plate| plate.land).count();
    if land < world.plates.plates as usize {
        ui.colored_label(PINK, format!("Only {land} cells fit inside the world. Fewer plates, or less coverage?"));
    }
    ui.label(format!("{land} land plates, {} ocean plates", made.kept.len() - land)).on_hover_text(
        "Each card: share of the world; blocks across (the side of a square with the same area);\nspeed in blocks per step; heading; spin in degrees per step.",
    );
    // Cards rather than a table, so the panel can stay thin: as many columns as fit, row by row.
    const CARD: f32 = 104.0;
    let gap = ui.spacing().item_spacing.x;
    let columns = (((ui.available_width() + gap) / (CARD + 14.0 + gap)).floor() as usize).max(1);
    let cards: Vec<usize> = (0..made.kept.len()).collect();
    for row in cards.chunks(columns) {
        ui.horizontal(|ui| {
            for &p in row {
                plate_card(ui, world, made, p, blocks_per_pixel, CARD);
            }
        });
    }
}

/// One plate's card in the readouts.
fn plate_card(ui: &mut egui::Ui, world: &WorldGen, made: &Plates, p: usize, blocks_per_pixel: f32, width: f32) {
    let plate = &made.kept[p];
    egui::Frame::group(ui.style()).inner_margin(6.0).show(ui, |ui| {
        ui.set_width(width);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let (swatch, _) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                ui.painter().rect_filled(swatch, 0.0, colour32(plate_colour(p, plate.land)));
                ui.strong(plate_name(made, p));
            });
            ui.small(format!("{:.1}% of the world", plate.share * 100.0));
            ui.small(format!("{} across", compass((plate.pixels as f32).sqrt() * blocks_per_pixel, "", "").trim()));
            if let Some(motion) = world.motion_of(p) {
                ui.small(format!("speed {:.0}", motion.speed()));
                ui.small(format!("heading {:.0}° {}", motion.heading(), bearing(motion.heading())));
                ui.small(format!("spin {:.2}°", motion.spin.abs().to_degrees()));
                ui.small(if motion.spin >= 0.0 { "clockwise" } else { "anticlockwise" });
            }
        });
    });
}

/// "plate 3" for a land plate, "ocean 2" for an ocean plate (numbered separately).
fn plate_name(made: &Plates, p: usize) -> String {
    let land = made.kept.iter().filter(|plate| plate.land).count();
    if p < land { format!("plate {}", p + 1) } else { format!("ocean {}", p + 1 - land) }
}

fn colour32([r, g, b]: [u8; 3]) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn canvas(ui: &mut egui::Ui, world: &mut WorldGen) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    // The plates are always drawn from the simulation: before it starts, it paints them exactly
    // as made.
    if world.result.is_some() && world.sim.is_none() {
        let _ = world.sim();
    }
    let Some(result) = &world.result else { return };
    let made = &result.made;
    let width = made.width;

    // Repaint only when the plates, the simulation or the look changed.
    let moved = world.has_moved();
    let wanted = Look {
        plates: world.version,
        motions: world.motions_version,
        terrain: world.terrain_version,
        steps: world.sim.as_ref().map(|sim| sim.steps),
        show_cells: world.show_cells,
        show_boundaries: world.show_boundaries,
        height_shown: world.height_shown,
        smoothness: world.shading_smoothness,
        tear_open: world.tear_open,
        squeeze: world.heights.squeeze,
        water: world.heights.water,
    };
    if world.painted_for.as_ref() != Some(&wanted) || world.texture.is_none() {
        let heights = &world.heights;
        // Open water: the ocean colour, lighter where it's shallow (`deep` 0 = at the water
        // level, 1 = as deep as ocean plates are, or deeper).
        let sea = |i: usize, deep: f32| {
            let floor = match (made.inside[i], world.show_cells) {
                (false, _) => return BEYOND,
                (true, true) => cell_shade(made.cell[i]),
                (true, false) => OCEAN,
            };
            let shallow = 1.0 - deep.clamp(0.0, 1.0);
            [0, 1, 2].map(|k| (floor[k] as f32 + (SHALLOW[k] as f32 - floor[k] as f32) * shallow) as u8)
        };
        // Lightened toward white (squeezed) or darkened toward black (stretched).
        let shade = |base: [u8; 3], lift: f32| {
            let toward = if lift >= 0.0 { 255.0 } else { 0.0 };
            base.map(|c| (c as f32 + (toward - c as f32) * lift.abs()) as u8)
        };
        let ocean_depth = (heights.water - heights.ocean).max(0.05);
        let (mut land, mut world_pixels) = (0usize, 0usize);
        if let Some(sim) = &world.sim {
            world.moved = sim.paint(world.shading_smoothness, world.tear_open);
        }
        let mut pixels: Vec<Color32> = (0..width * width)
            .map(|i| {
                world_pixels += made.inside[i] as usize;
                let boundary = (!moved && world.show_boundaries).then(|| world.contacts.get(i).copied().flatten()).flatten();
                if let (true, Some(contact)) = (made.inside[i], boundary) {
                    return colour32(boundary_colour(contact.kind));
                }
                let Some(spot) = world.moved.get(i).copied().flatten() else { return colour32(sea(i, 1.0)) };
                let height = world.terrain.get(spot.from as usize).copied().unwrap_or(heights.ocean) + heights.squeeze * spot.squeeze;
                let lift = (spot.squeeze / world.height_shown).clamp(-1.0, 1.0);
                colour32(if height > heights.water {
                    // Land: a land plate's colour, or an island of ocean floor squeezed up.
                    land += made.inside[i] as usize;
                    let plate_land = made.kept[spot.plate as usize].land;
                    shade(if plate_land { plate_colour(spot.plate as usize, true) } else { ISLAND }, lift)
                } else {
                    // Under water: only squeezed rock shows (white).
                    shade(sea(i, (heights.water - height) / ocean_depth), lift.max(0.0))
                })
            })
            .collect();
        world.land_share = land as f32 / world_pixels.max(1) as f32;
        if !moved {
            // Each plate's middle as a white pixel.
            for plate in &made.kept {
                let (col, row) = (plate.middle.0 as usize, plate.middle.1 as usize);
                if col < width && row < width {
                    pixels[row * width + col] = Color32::WHITE;
                }
            }
        }
        let image = egui::ColorImage::new([width, width], pixels);
        // Made tiny first, then given the picture: egui's first-make check assumes 2,048 pixels
        // is the most a graphics card can take, which is too cautious (1 chunk per pixel is 2,250).
        let texture = world.texture.get_or_insert_with(|| {
            ui.ctx().load_texture("world gen field", egui::ColorImage::new([1, 1], vec![Color32::BLACK]), egui::TextureOptions::NEAREST)
        });
        texture.set(image, egui::TextureOptions::NEAREST);
        world.painted_for = Some(wanted);
    }

    world.hover = world.camera.update(ui, rect, &response, (width, width));
    if let Some(texture) = &world.texture {
        let field = world.camera.field_rect(rect, (width, width));
        painter.image(texture.id(), field, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)), Color32::WHITE);
        painter.rect_stroke(field, 0.0, egui::Stroke::new(1.0, MAGENTA), egui::StrokeKind::Outside);
        // The edge of the world, and where the bowl starts.
        painter.circle_stroke(field.center(), field.width() / 2.0, egui::Stroke::new(1.0, MAGENTA));
        if world.show_bowl {
            let bowl = field.width() / 2.0 * world.sim_settings.bowl_start;
            painter.circle_stroke(field.center(), bowl, egui::Stroke::new(1.0, Color32::from_white_alpha(50)));
        }
    }
    let to_screen = |(x, y): (f32, f32)| world.camera.to_screen(rect, Vec2::new(x, y));
    // The lines and vertices are where the plates were made: hidden once they move.
    if world.show_lines && !moved {
        for edge in &made.edges {
            painter.line_segment([to_screen(edge.from), to_screen(edge.to)], egui::Stroke::new(1.0, Color32::from_white_alpha(140)));
        }
    }
    if world.show_vertices && !moved {
        for vertex in &made.vertices {
            painter.circle_filled(to_screen(vertex.at), 2.5, VERTEX);
        }
    }
    if world.show_arrows {
        // The curve shows spin compared with the fastest allowed at the start: that would be three
        // quarters of a circle. (Real turns are too small to see.)
        let fastest = world.motion.spin.to_radians().max(1e-6);
        for (p, plate) in made.kept.iter().enumerate() {
            let (Some(motion), middle) = (world.motion_of(p), world.middle_of(p, made)) else { continue };
            heading_arrow(&painter, &to_screen, middle, &motion, world.arrow_steps, made.blocks_per_pixel);
            spin_arrow(&painter, &to_screen, middle, (plate.pixels as f32).sqrt() * 0.25, motion.spin / fastest * 1.5 * std::f32::consts::PI);
        }
    }
    world.camera.outline(&painter, rect, world.hover, MAGENTA);
    painter.text(
        rect.left_bottom() + Vec2::new(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "drag = move   scroll = zoom   double-click = fit   north is up   white dot = a plate's middle",
        egui::FontId::proportional(12.0),
        PINK,
    );
}

/// A line with a dark edge, so it shows on any colour.
fn outlined(painter: &egui::Painter, points: Vec<Pos2>, width: f32) {
    painter.add(egui::Shape::line(points.clone(), egui::Stroke::new(width + 2.0, Color32::from_black_alpha(200))));
    painter.add(egui::Shape::line(points, egui::Stroke::new(width, Color32::WHITE)));
}

/// Two short strokes making an arrowhead at `tip`, pointing along `direction` (on screen).
fn arrowhead(tip: Pos2, direction: Vec2) -> [Vec<Pos2>; 2] {
    let back = -direction.normalized() * 7.0;
    let side = Vec2::new(-back.y, back.x) * 0.6;
    [vec![tip + back + side, tip], vec![tip + back - side, tip]]
}

/// A straight arrow from a plate's middle to where it would be after `steps` steps.
fn heading_arrow(painter: &egui::Painter, to_screen: &impl Fn((f32, f32)) -> Pos2, middle: (f32, f32), motion: &Motion, steps: f32, blocks_per_pixel: f32) {
    let travel = (motion.velocity.0 * steps / blocks_per_pixel, motion.velocity.1 * steps / blocks_per_pixel);
    let (from, to) = (to_screen(middle), to_screen((middle.0 + travel.0, middle.1 + travel.1)));
    if from.distance(to) < 3.0 {
        return;
    }
    outlined(painter, vec![from, to], 2.0);
    for stroke in arrowhead(to, to - from) {
        outlined(painter, stroke, 2.0);
    }
}

/// A curved arrow around a plate's middle, `turn` radians long (+ = clockwise). Starts at the
/// top.
fn spin_arrow(painter: &egui::Painter, to_screen: &impl Fn((f32, f32)) -> Pos2, middle: (f32, f32), radius: f32, turn: f32) {
    // Never quite a full circle, so the arrow's start and end stay clear.
    let turn = turn.clamp(-5.5, 5.5);
    if turn.abs() < 0.05 {
        return;
    }
    let pieces = ((turn.abs() * 12.0).ceil() as usize).max(2);
    let points: Vec<Pos2> = (0..=pieces)
        .map(|k| {
            // From straight up (north), clockwise for + (south is down on the map).
            let angle = -std::f32::consts::FRAC_PI_2 + turn * k as f32 / pieces as f32;
            to_screen((middle.0 + radius * angle.cos(), middle.1 + radius * angle.sin()))
        })
        .collect();
    let (tip, before) = (points[pieces], points[pieces - 1]);
    outlined(painter, points, 1.5);
    for stroke in arrowhead(tip, tip - before) {
        outlined(painter, stroke, 1.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_world_gen_preset_survives_being_written_and_read() {
        let mut world = WorldGen::default();
        world.plates.plates = 4;
        world.plates.rough.style = LineStyle::Perlin;
        world.motion.inward = 0.9;
        world.sim_settings.passes = 7;
        world.heights.water = -0.25;
        world.shading_smoothness = 9;
        let text = toml::to_string_pretty(&world.preset()).unwrap();
        let back: WorldGenPreset = toml::from_str(&text).unwrap();
        assert_eq!(back, world.preset());
    }

    #[test]
    fn using_a_preset_changes_the_settings_and_ends_any_comparison() {
        let mut world = WorldGen::default();
        let mut preset = world.preset();
        preset.heights.water = 0.4;
        preset.show.arrows = false;
        world.ab_heights = Some(HeightSettings::default());
        world.use_preset(preset.clone());
        assert_eq!(world.preset(), preset);
        assert!(world.ab_heights.is_none());
    }

    #[test]
    fn an_old_preset_missing_settings_still_loads() {
        let back: WorldGenPreset = toml::from_str("[heights]\nwater = 0.3\n").unwrap();
        assert_eq!(back.heights.water, 0.3);
        assert_eq!(back.plates, PlateSettings::default());
    }
}

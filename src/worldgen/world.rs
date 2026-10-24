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
use super::palette::from_hsl;
use super::plates::{self, LineNoise, LineStyle, PlateSettings, Plates};
use super::runner::{Finished, Job, Poll};
use super::tectonics::{self, Boundary, Contact, Motion, MotionSettings, Sim, SimSettings};
use super::view::{MAGENTA, PINK, compass, seed_row};
use bevy_egui::egui::{self, Color32, Pos2, Vec2};

/// Ocean inside the world, and the dark past its edge.
const OCEAN: [u8; 3] = [18, 38, 82];
const BEYOND: [u8; 3] = [10, 10, 14];
const VERTEX: Color32 = Color32::from_rgb(255, 220, 60);
/// The boundaries: red colliding, blue pulling apart, green sliding.
const COLLIDING: [u8; 3] = [235, 60, 55];
const PULLING_APART: [u8; 3] = [80, 170, 255];
const SLIDING: [u8; 3] = [70, 225, 110];

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
    pub sim_settings: SimSettings,
    sim: Option<Sim>,
    running: bool,
    /// How fast to watch it. Only changes how many steps are taken each second, never the result.
    pub steps_per_second: f32,
    /// Steps owed since the last frame (part steps carry over).
    owed: f32,
    /// The simulation as last painted: the plate and ground height on each pixel.
    moved: Vec<Option<(u8, f32)>>,
    /// How bright the most raised ground shows (a height of this = white).
    pub height_shown: f32,
    pub show_bowl: bool,
    /// For automatic testing: start the simulation as soon as the plates are made.
    pub start_when_ready: bool,

    result: Option<Finished<Plates>>,
    job: Option<Job<Plates>>,
    seen: Option<(f32, f32, u64, PlateSettings)>,
    wanted: bool,
    version: u64,

    camera: Camera,
    texture: Option<egui::TextureHandle>,
    painted_for: Option<(u64, bool, u64, bool, Option<u64>, u32)>,
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
            sim_settings: SimSettings::default(),
            sim: None,
            running: false,
            steps_per_second: 60.0,
            owed: 0.0,
            moved: Vec::new(),
            height_shown: 1.0,
            show_bowl: true,
            start_when_ready: false,
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
            self.sim = Some(Sim::new(&result.made, &self.motions));
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
}

fn settings(ui: &mut egui::Ui, world: &mut WorldGen) {
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
    if ui.button("Plate settings back to defaults").clicked() {
        world.plates = PlateSettings::default();
    }

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
        "Start it in the bar along the top. Plates coast on their starting push, collide, grip and turn \
         each other, and slow down. These can be changed while it runs; Reset replays from the start.",
    );
    let sim = &mut world.sim_settings;
    ui.add(egui::Slider::new(&mut sim.drag, 0.0001..=0.1).logarithmic(true).text("drag")).on_hover_text(
        "How much speed and spin every plate loses each step. 0.01 = 1%.\nLower = they coast for longer.",
    );
    ui.add(egui::Slider::new(&mut sim.push, 0.0..=500.0).text("push back")).on_hover_text(
        "How hard overlapping plates are shoved apart.\nLow = they crush deep into each other. High = they hardly overlap.",
    );
    ui.add(egui::Slider::new(&mut sim.give, 0.0..=200.0).text("give")).on_hover_text(
        "How quickly the speed two plates meet with is soaked up.\nLow = they bounce off. High = they stop dead and stay together.",
    );
    ui.add(egui::Slider::new(&mut sim.friction, 0.0..=200.0).text("friction")).on_hover_text(
        "How strongly plates sliding past each other grip and drag one another.\nThe drag acts at the boundary, off to one side of each plate's middle, so it turns them.",
    );
    ui.add(egui::Slider::new(&mut sim.ocean_weight, 0.05..=2.0).text("ocean plates' weight ×"))
        .on_hover_text("How heavy ocean plates are for their size, compared with land plates.\nLight ones get shoved aside; heavy ones shove.");
    let mut bowl_percent = sim.bowl_start * 100.0;
    if ui
        .add(egui::Slider::new(&mut bowl_percent, 0.0..=99.0).suffix("%").text("bowl starts at"))
        .on_hover_text("The bowl keeps the plates from straying far from the middle (like a Beyblade arena).\nPast this share of the world's radius, the ground slopes up and pushes them back inward.")
        .changed()
    {
        sim.bowl_start = bowl_percent / 100.0;
    }
    ui.add(egui::Slider::new(&mut sim.bowl, 0.0..=50.0).text("bowl steepness"))
        .on_hover_text("How hard the bowl pushes back. The further out a plate goes, the harder.");
    ui.add(egui::Slider::new(&mut sim.uplift, 0.0..=5.0).text("uplift")).on_hover_text(
        "How much the ground rises where plates crush into each other (more when they meet faster).\nCrude for now: the real squeezing comes with the rags.",
    );
    ui.add(egui::Slider::new(&mut world.height_shown, 0.05..=10.0).logarithmic(true).text("height shown as white"))
        .on_hover_text("Only changes the look: raised ground this high shows white, lower ground in between.");
    ui.checkbox(&mut world.show_bowl, "show where the bowl starts (a faint circle)");

    ui.add_space(6.0);
    if ui.button("Tectonics settings back to defaults").clicked() {
        world.motion = MotionSettings::default();
        world.sim_settings = SimSettings::default();
    }
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
            if world.has_moved() {
                match world.moved.get(i).copied().flatten() {
                    Some((p, height)) => {
                        ui.label(plate_name(made, p as usize));
                        ui.label(format!("ground raised {height:.2}"));
                    }
                    None => {
                        ui.label("no plate (empty ocean)");
                    }
                }
            } else {
                ui.label(format!("cell {}", made.cell[i]));
                match made.plate[i] {
                    Some(p) => ui.label(plate_name(made, p as usize)),
                    None => ui.label("no plate (empty ocean)"),
                };
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
    let Some(result) = &world.result else { return };
    let made = &result.made;
    let width = made.width;

    // Repaint only when the plates, the simulation or the look changed.
    let moved = world.has_moved();
    let sim_steps = world.sim.as_ref().filter(|_| moved).map(|sim| sim.steps);
    let wanted = (world.version, world.show_cells, world.motions_version, world.show_boundaries, sim_steps, world.height_shown.to_bits());
    if world.painted_for != Some(wanted) || world.texture.is_none() {
        let ocean = |i: usize| match (made.inside[i], world.show_cells) {
            (false, _) => BEYOND,
            (true, true) => cell_shade(made.cell[i]),
            (true, false) => OCEAN,
        };
        let mut pixels: Vec<Color32>;
        if let Some(sim) = world.sim.as_ref().filter(|_| moved) {
            // The plates where they are now, raised ground lightened toward white.
            world.moved = sim.paint();
            pixels = world
                .moved
                .iter()
                .enumerate()
                .map(|(i, here)| {
                    colour32(match *here {
                        Some((p, height)) => {
                            let base = plate_colour(p as usize, made.kept[p as usize].land);
                            let lift = (height / world.height_shown).clamp(0.0, 1.0);
                            base.map(|c| (c as f32 + (255.0 - c as f32) * lift) as u8)
                        }
                        None => ocean(i),
                    })
                })
                .collect();
        } else {
            pixels = (0..width * width)
                .map(|i| {
                    let boundary = world.show_boundaries.then(|| world.contacts.get(i).copied().flatten()).flatten();
                    colour32(match (made.inside[i], made.plate[i], boundary) {
                        (true, _, Some(contact)) => boundary_colour(contact.kind),
                        (true, Some(p), None) => plate_colour(p as usize, made.kept[p as usize].land),
                        _ => ocean(i),
                    })
                })
                .collect();
            // Each plate's middle as a white pixel.
            for plate in &made.kept {
                let (col, row) = (plate.middle.0 as usize, plate.middle.1 as usize);
                if col < width && row < width {
                    pixels[row * width + col] = Color32::WHITE;
                }
            }
        }
        let image = egui::ColorImage::new([width, width], pixels);
        match &mut world.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => world.texture = Some(ui.ctx().load_texture("world gen field", image, egui::TextureOptions::NEAREST)),
        }
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

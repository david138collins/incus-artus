//! World Gen — the half of the Editor Suite where pixels mean something (plates, later ocean,
//! mountains, rivers) and processes run over them. The other half, the Graphics Kit, is the art
//! pipeline: numbers into colours and back, with no meaning attached.
//!
//! **One world seed.** Every World Gen step takes its own seed from it, so "New seeds for
//! everything" rolls one number and the whole world changes. Cycle through it to see whether the
//! settings routinely make good shapes.
//!
//! The first tool is **plates** (`plates.rs`).
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
use super::plates::{self, PlateSettings, Plates};
use super::runner::{Finished, Job, Poll};
use super::view::{MAGENTA, PINK, compass, seed_row};
use bevy_egui::egui::{self, Color32, Vec2};

/// Ocean inside the world, and the dark past its edge.
const OCEAN: [u8; 3] = [18, 38, 82];
const BEYOND: [u8; 3] = [10, 10, 14];
const VERTEX: Color32 = Color32::from_rgb(255, 220, 60);

pub struct WorldGen {
    /// The world seed. Each step takes its own seed from it.
    pub seed: u64,
    pub plates: PlateSettings,
    /// Show the deleted cells faintly in the ocean.
    pub show_cells: bool,
    /// Draw the vertices, and the straight edges as they were before wobbling, over the picture.
    pub show_vertices: bool,
    pub show_lines: bool,

    result: Option<Finished<Plates>>,
    job: Option<Job<Plates>>,
    seen: Option<(f32, f32, u64, PlateSettings)>,
    wanted: bool,
    version: u64,

    camera: Camera,
    texture: Option<egui::TextureHandle>,
    painted_for: Option<(u64, bool)>,
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

    /// Notices changes, collects finished work, and starts the next when wanted (as layers do).
    pub fn drive(&mut self, size_blocks: f32, blocks_per_pixel: f32) {
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

/// A plate's colour: hues spread around the wheel so neighbours differ.
fn plate_colour(plate: usize) -> [u8; 3] {
    from_hsl([(plate as f32 * 137.5) % 360.0, 0.45, 0.55])
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
        });
    });
    egui::Panel::left("world gen settings").resizable(true).default_size(340.0).size_range(220.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| settings(ui, world));
    });
    egui::Panel::right("world gen readouts").resizable(true).default_size(270.0).size_range(180.0..=900.0).show(root, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| readouts(ui, world, blocks));
    });
    egui::CentralPanel::no_frame().show(root, |ui| canvas(ui, world));
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

    ui.add_space(6.0);
    ui.separator();
    ui.strong("Wobbly edges");
    ui.small(
        "Each line between two cells is pushed sideways by Perlin noise running along it. The push fades \
         to nothing at the vertices (where three cells meet), so the corners stay put and the lines still meet.",
    );
    ui.add(egui::Slider::new(&mut plates.wobble, 0.0..=10_000.0).text("wobble (blocks)"))
        .on_hover_text("How far a line may be pushed sideways. 0 = straight lines.\nVery big wobbles get cut off where they run into other lines.");
    ui.add(egui::Slider::new(&mut plates.wobble_size, 300.0..=40_000.0).logarithmic(true).text("wobble size (blocks)"))
        .on_hover_text("How long the biggest wobbles are, along the line.");
    ui.add(egui::Slider::new(&mut plates.wobble_octaves, 1..=8).text("octaves"))
        .on_hover_text("Finer, fainter wobbles stacked on the big ones.");
    ui.add(egui::Slider::new(&mut plates.wobble_roughness, 0.0..=1.0).text("roughness"))
        .on_hover_text("How strong each finer wobble is compared with the one before.\nHigher = more jagged coastlines.");

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
            ui.label(format!("cell {}", made.cell[i]));
            match made.plate[i] {
                Some(p) => ui.label(format!("plate {}", p + 1)),
                None => ui.label("no plate (ocean)"),
            };
        }
        _ => {
            ui.small("Point at the world.");
        }
    }

    ui.separator();
    ui.heading("The plates");
    ui.label(format!("cover {:.1}% of the world (aiming for {:.0}%)", made.coverage * 100.0, world.plates.coverage * 100.0));
    ui.label(format!("{} cells, {} reached past the edge and were deleted", made.cells, made.past_the_edge));
    ui.label(format!("{} vertices, {} lines between them", made.vertices.len(), made.edges.len()));
    if made.kept.len() < world.plates.plates as usize {
        ui.colored_label(PINK, format!("Only {} cells fit inside the world. Fewer plates, or less coverage?", made.kept.len()));
    }
    egui::Grid::new("world gen plates").striped(true).show(ui, |ui| {
        for (p, plate) in made.kept.iter().enumerate() {
            let [r, g, b] = plate_colour(p);
            let (swatch, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
            ui.painter().rect_filled(swatch, 0.0, Color32::from_rgb(r, g, b));
            ui.label(format!("plate {}", p + 1));
            ui.label(format!("{:.1}% of the world", plate.share * 100.0));
            ui.small(format!("about {} blocks across", compass((plate.pixels as f32).sqrt() * blocks_per_pixel, "", "").trim()))
                .on_hover_text("The side of a square with the same area.");
            ui.end_row();
        }
    });
}

fn canvas(ui: &mut egui::Ui, world: &mut WorldGen) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let Some(result) = &world.result else { return };
    let made = &result.made;
    let width = made.width;

    // Repaint only when the plates or the look changed.
    let wanted = (world.version, world.show_cells);
    if world.painted_for != Some(wanted) || world.texture.is_none() {
        let mut pixels: Vec<Color32> = (0..width * width)
            .map(|i| {
                let [r, g, b] = match (made.inside[i], made.plate[i]) {
                    (false, _) => BEYOND,
                    (true, Some(p)) => plate_colour(p as usize),
                    (true, None) if world.show_cells => cell_shade(made.cell[i]),
                    (true, None) => OCEAN,
                };
                Color32::from_rgb(r, g, b)
            })
            .collect();
        // Each plate's middle as a white pixel.
        for plate in &made.kept {
            let (col, row) = (plate.middle.0 as usize, plate.middle.1 as usize);
            if col < width && row < width {
                pixels[row * width + col] = Color32::WHITE;
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
        // The edge of the world.
        painter.circle_stroke(field.center(), field.width() / 2.0, egui::Stroke::new(1.0, MAGENTA));
    }
    let to_screen = |(x, y): (f32, f32)| world.camera.to_screen(rect, Vec2::new(x, y));
    if world.show_lines {
        for edge in &made.edges {
            painter.line_segment([to_screen(edge.from), to_screen(edge.to)], egui::Stroke::new(1.0, Color32::from_white_alpha(140)));
        }
    }
    if world.show_vertices {
        for vertex in &made.vertices {
            painter.circle_filled(to_screen(vertex.at), 2.5, VERTEX);
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

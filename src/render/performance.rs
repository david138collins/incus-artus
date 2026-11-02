//! Performance — the gauge for "are we hitting the wall?" [David: "I need to know when we start
//! hitting that wall so I can start hitting the brakes."]
//!
//! The "Performance" debug page shows:
//! - **frame rate**: the average, the **1% low** (the slowest 1 frame in 100, which is what you
//!   feel as stutter), and the **worst frame** of the last few seconds;
//! - a **graph** of the last few seconds of frame times, with lines at 240 and 60 frames a second;
//! - **the work in flight**: chunks waiting and being made, sections waiting to be meshed;
//! - **what's being drawn**: sections, rectangles ("quads") and corners ("vertices").
//!
//! The budget it reads against: 240 FPS (4.2 ms a frame) on David's laptop, as a stand-in for "plays
//! on a GTX 980 Ti at 1080p on low settings".

use super::meshing::{MeshingState, SectionMesh};
use crate::world::{Streaming, VoxelWorld};
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

/// How many recent frames are kept (about five seconds at 240 FPS).
const KEPT: usize = 1200;
/// The target: 240 frames a second.
const BUDGET_MS: f32 = 1000.0 / 240.0;

/// Recent frame times, in milliseconds, newest last.
#[derive(Resource, Default)]
pub struct FrameTimes(VecDeque<f32>);

/// The stutter breakdown [deferred since 2026-10-26, built 2026-10-28]: the game's own heavy steps
/// on the main thread time themselves (`Timed`), so a slow frame can say where its time went, or
/// that it wasn't ours (drawing, the graphics card, Bevy, Windows).
static STEPS: Mutex<Vec<(&'static str, f32)>> = Mutex::new(Vec::new());

/// Times a step from here to the end of the block it's made in: `let _timed = Timed::new("name");`.
pub struct Timed {
    name: &'static str,
    started: Instant,
}

impl Timed {
    pub fn new(name: &'static str) -> Self {
        Self { name, started: Instant::now() }
    }
}

impl Drop for Timed {
    fn drop(&mut self) {
        let ms = self.started.elapsed().as_secs_f32() * 1000.0;
        if let Ok(mut steps) = STEPS.lock() {
            steps.push((self.name, ms));
        }
    }
}

/// A frame slower than this is logged with its breakdown (ms).
const HITCH_MS: f32 = 25.0;

/// The steps' times: the last frame's (waiting for its length), the slowest of each in the second
/// before last, and the recent slow frames.
#[derive(Resource, Default)]
pub struct StepTimes {
    last_frame: Vec<(&'static str, f32)>,
    this_second: Vec<(&'static str, f32)>,
    second_started: f32,
    /// The slowest time each step took in the last full second, slowest first.
    pub last_second: Vec<(&'static str, f32)>,
    /// Recent slow frames: their length and the steps over a millisecond.
    pub hitches: VecDeque<(f32, Vec<(&'static str, f32)>)>,
}

/// At the very end of a frame: what its steps took (its length is known next frame).
pub fn collect_steps(mut steps_times: ResMut<StepTimes>) {
    let Ok(mut steps) = STEPS.lock() else { return };
    // The same step can run more than once a frame: added up.
    let mut frame: Vec<(&'static str, f32)> = Vec::new();
    for (name, ms) in steps.drain(..) {
        match frame.iter_mut().find(|(n, _)| *n == name) {
            Some(step) => step.1 += ms,
            None => frame.push((name, ms)),
        }
    }
    steps_times.last_frame = frame;
}

pub fn record_frame_time(time: Res<Time<Real>>, mut times: ResMut<FrameTimes>, mut steps: ResMut<StepTimes>) {
    let ms = time.delta_secs() * 1000.0;
    if ms <= 0.0 {
        return;
    }
    times.0.push_back(ms);
    while times.0.len() > KEPT {
        times.0.pop_front();
    }
    // The frame just measured is the one whose steps were collected last.
    let last = std::mem::take(&mut steps.last_frame);
    for &(name, step) in &last {
        match steps.this_second.iter_mut().find(|(n, _)| *n == name) {
            Some(slowest) => slowest.1 = slowest.1.max(step),
            None => steps.this_second.push((name, step)),
        }
    }
    if ms > HITCH_MS {
        let mut ours: Vec<(&'static str, f32)> = last.into_iter().filter(|&(_, step)| step >= 1.0).collect();
        ours.sort_by(|a, b| b.1.total_cmp(&a.1));
        let listed: Vec<String> = ours.iter().map(|(name, step)| format!("{name} {step:.1}")).collect();
        let ours_total: f32 = ours.iter().map(|(_, step)| step).sum();
        info!("Slow frame: {ms:.1} ms; the game's own steps {ours_total:.1} ms{}{}", if listed.is_empty() { "" } else { ": " }, listed.join(", "));
        steps.hitches.push_back((ms, ours));
        while steps.hitches.len() > 8 {
            steps.hitches.pop_front();
        }
    }
    let now = time.elapsed_secs();
    if now - steps.second_started >= 1.0 {
        let mut second = std::mem::take(&mut steps.this_second);
        second.sort_by(|a, b| b.1.total_cmp(&a.1));
        steps.last_second = second;
        steps.second_started = now;
    }
}

/// Bevy's drawing passes in plain words (the ones this game uses; any other shows its own name).
fn pass_name(name: &str) -> &str {
    match name {
        "main_opaque_pass_3d" => "solid things: near blocks, far land, far copies, objects",
        "main_transparent_pass_3d" => "see-through things: water",
        "main_transmissive_pass_3d" => "glass-like things",
        "shadows" => "shadows",
        "per_view_shadow_pass" => "one light's shadows",
        "early_mesh_preprocessing" => "Bevy sorting out what to draw",
        "late_mesh_preprocessing" => "Bevy sorting out what to draw, second look",
        "clustering" => "Bevy sorting the lights",
        "render_sky" => "the sky",
        "tonemapping" => "finishing the colours",
        "upscaling" => "putting the picture on the screen",
        "ui" => "Bevy's on-screen text",
        other => other,
    }
}

/// The graphics card's time for each drawing pass, averaged over the last 20 frames (ms), with how
/// deep it sits inside another pass (0 = not inside one): slowest first among each level's own. Only
/// passes drawn in the last second; Bevy times them (`RenderDiagnosticsPlugin`) on DX12 and Vulkan.
/// Not timed: the Panini pass and the debug panel.
pub fn card_times(world: &World) -> Vec<(String, f64, usize)> {
    let Some(store) = world.get_resource::<bevy::diagnostic::DiagnosticsStore>() else { return Vec::new() };
    let mut times: Vec<(Vec<String>, f64)> = store
        .iter()
        .filter(|diagnostic| diagnostic.path().as_str().starts_with("render/") && diagnostic.path().as_str().ends_with("/elapsed_gpu"))
        .filter(|diagnostic| diagnostic.measurement().is_some_and(|m| m.time.elapsed().as_secs_f32() < 1.0))
        .filter_map(|diagnostic| {
            let parts: Vec<String> = diagnostic.path().components().map(str::to_string).collect();
            Some((parts[1..parts.len() - 1].to_vec(), diagnostic.average()?))
        })
        .collect();
    // Each pass after the one it's inside, the slowest first at every level.
    let total_of = |times: &[(Vec<String>, f64)], parts: &[String]| -> Vec<f64> { (1..=parts.len()).map(|n| times.iter().find(|(p, _)| p[..] == parts[..n]).map_or(0.0, |t| t.1)).collect() };
    let keys: Vec<Vec<f64>> = times.iter().map(|(parts, _)| total_of(&times, parts)).collect();
    let mut order: Vec<usize> = (0..times.len()).collect();
    order.sort_by(|&a, &b| {
        let (ka, kb) = (&keys[a], &keys[b]);
        for (x, y) in ka.iter().zip(kb) {
            if x != y {
                return y.total_cmp(x);
            }
        }
        ka.len().cmp(&kb.len())
    });
    let mut sorted = Vec::new();
    for k in order {
        let (parts, ms) = std::mem::take(&mut times[k]);
        sorted.push((pass_name(parts.last().map_or("", |s| s)).to_string(), ms, parts.len() - 1));
    }
    sorted
}

/// The card's time on one line: the passes not inside another, added up, and the three slowest.
pub fn card_line(world: &World) -> String {
    let times = card_times(world);
    if times.is_empty() {
        return "graphics card: not timed".to_string();
    }
    let total: f64 = times.iter().filter(|t| t.2 == 0).map(|t| t.1).sum();
    let slowest: Vec<String> = times.iter().filter(|t| t.2 == 0).take(3).map(|(name, ms, _)| format!("{name} {ms:.2}")).collect();
    format!("graphics card {total:.2} ms a frame ({})", slowest.join(", "))
}

/// Average FPS, 1%-low FPS and the worst frame (ms) over the frames kept.
fn summary(times: &VecDeque<f32>) -> Option<(f32, f32, f32)> {
    if times.is_empty() {
        return None;
    }
    let total: f32 = times.iter().sum();
    let average = 1000.0 * times.len() as f32 / total;
    let mut sorted: Vec<f32> = times.iter().copied().collect();
    sorted.sort_by(|a, b| b.total_cmp(a));
    // The slowest 1% of frames, averaged.
    let slowest = (sorted.len() / 100).max(1);
    let low = 1000.0 / (sorted[..slowest].iter().sum::<f32>() / slowest as f32);
    Some((average, low, sorted[0]))
}

pub fn performance_debug_page(ui: &mut egui::Ui, world: &mut World) {
    ui.small("Against the budget: 240 FPS (4.2 ms a frame). The 1% low is what you feel as stutter.");
    let times = world.resource::<FrameTimes>().0.clone();
    if let Some((average, low, worst)) = summary(&times) {
        let colour = |fps: f32| if fps >= 240.0 { egui::Color32::from_rgb(120, 220, 120) } else if fps >= 120.0 { egui::Color32::from_rgb(230, 200, 90) } else { egui::Color32::from_rgb(235, 90, 80) };
        ui.horizontal(|ui| {
            ui.colored_label(colour(average), format!("{average:.0} FPS average"));
            ui.colored_label(colour(low), format!("{low:.0} FPS 1% low"));
            ui.colored_label(colour(1000.0 / worst), format!("worst frame {worst:.1} ms"));
        });
        // The graph: one bar per frame, newest on the right.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(420.0), 90.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(20));
        let top_ms = 25.0;
        let y_of = |ms: f32| rect.bottom() - (ms / top_ms).min(1.0) * rect.height();
        for (ms, label) in [(BUDGET_MS, "240"), (1000.0 / 60.0, "60")] {
            painter.line_segment([egui::pos2(rect.left(), y_of(ms)), egui::pos2(rect.right(), y_of(ms))], egui::Stroke::new(1.0, egui::Color32::from_gray(90)));
            painter.text(egui::pos2(rect.left() + 2.0, y_of(ms) - 1.0), egui::Align2::LEFT_BOTTOM, label, egui::FontId::proportional(9.0), egui::Color32::from_gray(140));
        }
        let shown = times.len().min(rect.width() as usize);
        for (k, &ms) in times.iter().skip(times.len() - shown).enumerate() {
            let x = rect.right() - (shown - k) as f32;
            let colour = if ms <= BUDGET_MS { egui::Color32::from_rgb(90, 180, 90) } else if ms <= 1000.0 / 60.0 { egui::Color32::from_rgb(220, 190, 80) } else { egui::Color32::from_rgb(230, 80, 70) };
            painter.line_segment([egui::pos2(x, rect.bottom()), egui::pos2(x, y_of(ms))], egui::Stroke::new(1.0, colour));
        }
    }

    ui.separator();
    ui.strong("Where the time goes");
    ui.small("The game's own heavy steps, the slowest each took in the last second (ms). A slow frame whose steps don't add up to it lost its time elsewhere: drawing, the graphics card, Bevy or Windows.");
    let steps = world.resource::<StepTimes>();
    if steps.last_second.is_empty() {
        ui.small("Nothing timed yet.");
    }
    for (name, ms) in steps.last_second.iter().take(10) {
        ui.label(format!("{ms:6.2}  {name}"));
    }
    if !steps.hitches.is_empty() {
        ui.small(format!("Recent slow frames (over {HITCH_MS:.0} ms; also in the session log):"));
        for (ms, ours) in steps.hitches.iter().rev() {
            let listed: Vec<String> = ours.iter().map(|(name, step)| format!("{name} {step:.1}")).collect();
            ui.small(format!("{ms:.1} ms: {}", if listed.is_empty() { "none of the game's own steps over 1 ms".to_string() } else { listed.join(", ") }));
        }
    }

    ui.separator();
    ui.strong("Graphics card time");
    ui.small("How long the graphics card spends on each part of the drawing, averaged over the last 20 frames (ms). If the total is close to a whole frame, the card is what holds the frame rate back; if it's well under, the time goes elsewhere (the game, Bevy, Windows). The far land and copies are drawn with the near blocks: turn them off to see their share.");
    let times = card_times(world);
    if times.is_empty() {
        ui.small("Not timed yet (the card times its passes on DirectX 12 and Vulkan only).");
    } else {
        let total: f64 = times.iter().filter(|t| t.2 == 0).map(|t| t.1).sum();
        let frame = summary(&world.resource::<FrameTimes>().0).map_or(0.0, |(average, _, _)| 1000.0 / average as f64);
        ui.label(format!("{total:6.2}  in all, of a {frame:.2} ms frame"));
        for (name, ms, depth) in &times {
            ui.label(format!("{ms:6.2}  {}{name}", "    ".repeat(*depth + 1)));
        }
    }

    ui.separator();
    ui.strong("Work in flight");
    let streaming = world.resource::<Streaming>();
    ui.label(format!("Chunks waiting: {}   being made: {}", streaming.queued(), streaming.being_made()));
    ui.label(format!("Chunks in the world: {}", world.resource::<VoxelWorld>().chunk_count()));
    let meshing = world.resource::<MeshingState>();
    let average_job = if meshing.jobs_finished > 0 { meshing.total_job_micros as f64 / meshing.jobs_finished as f64 / 1000.0 } else { 0.0 };
    ui.label(format!("Sections waiting to mesh: {}   meshing: {}   made, waiting to be shown: {}   (a mesh takes {average_job:.2} ms on average)", meshing.waiting.len(), meshing.running.len(), meshing.ready.len()));

    ui.separator();
    ui.strong("Being drawn");
    let (mut sections, mut quads, mut vertices) = (0u64, 0u64, 0u64);
    let mut query = world.query::<(&SectionMesh, &Visibility)>();
    for (mesh, visibility) in query.iter(world) {
        if *visibility != Visibility::Hidden {
            sections += 1;
            quads += mesh.quads as u64;
            vertices += mesh.vertices as u64;
        }
    }
    ui.label(format!("Sections: {sections}   quads: {quads}   vertices: {vertices}"));
    let distant = world.resource::<super::distant::DistantState>();
    ui.label(format!("Distant terrain: {} tiles, {} quads   (being made: {})", distant.tiles_drawn(), distant.quads, distant.being_made()));

    ui.separator();
    ui.strong("Temperatures");
    crate::temperatures::temperatures_settings(ui, world);
}

/// The page's numbers on one line, plus how many of each kind of thing the engine is holding (a
/// count that keeps climbing on a long flight is a leak): for the soak test's log.
/// What's being drawn, in a line (the World page's test caves, `docs/caves.md` stop 2): the sections
/// in view this frame and their rectangles, of every section meshed, and the frame rate.
pub fn drawing_line(world: &mut World) -> String {
    let fps = summary(&world.resource::<FrameTimes>().0).map_or("no frames".to_string(), |(average, low, worst)| format!("{average:.0} fps, 1% low {low:.0}, worst {worst:.1} ms"));
    let (mut sections, mut quads, mut in_view, mut quads_in_view) = (0u64, 0u64, 0u64, 0u64);
    let mut query = world.query::<(&SectionMesh, &ViewVisibility)>();
    for (mesh, visibility) in query.iter(world) {
        sections += 1;
        quads += mesh.quads as u64;
        if visibility.get() {
            in_view += 1;
            quads_in_view += mesh.quads as u64;
        }
    }
    let distant = world.resource::<super::distant::DistantSettings>();
    let far_land = if !distant.enabled { "off" } else if distant.march { "the march" } else { "the sheet" };
    format!("In view: {in_view} of {sections} sections, {quads_in_view} of {quads} rectangles. Far land: {far_land}. {fps}; {}", card_line(world))
}

pub fn soak_line(world: &mut World) -> String {
    let fps = summary(&world.resource::<FrameTimes>().0).map_or("no frames".to_string(), |(average, low, worst)| format!("{average:.0} fps, 1% low {low:.0}, worst {worst:.1} ms"));
    let streaming = world.resource::<Streaming>();
    let (queued, making, made, dropped) = (streaming.queued(), streaming.being_made(), streaming.made, streaming.dropped);
    let chunks = world.resource::<VoxelWorld>().chunk_count();
    let meshing = world.resource::<MeshingState>();
    let (waiting, meshing_now, section_entities) = (meshing.waiting.len(), meshing.running.len(), meshing.entities.len());
    let distant = world.resource::<super::distant::DistantState>();
    let (tiles, tiles_making) = (distant.tiles_drawn(), distant.being_made());
    let entities = world.query::<Entity>().iter(world).count();
    let meshes = world.resource::<Assets<Mesh>>().len();
    let images = world.resource::<Assets<Image>>().len();
    let materials = world.resource::<Assets<super::material::TerrainMaterial>>().len() + world.resource::<Assets<super::distant::DistantMaterial>>().len();
    format!(
        "{fps} | chunks {chunks} (queued {queued}, making {making}, made {made}, dropped {dropped}) | sections drawn {section_entities}, waiting {waiting}, meshing {meshing_now} | distant tiles {tiles} (making {tiles_making}) | entities {entities}, meshes {meshes}, images {images}, materials {materials}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_one_percent_low_is_the_slowest_frames() {
        // 99 frames at 4 ms and one at 20 ms.
        let mut times: VecDeque<f32> = std::iter::repeat_n(4.0, 99).collect();
        times.push_back(20.0);
        let (average, low, worst) = summary(&times).unwrap();
        assert!((average - 1000.0 / 4.16).abs() < 1.0, "{average}");
        assert!((low - 50.0).abs() < 0.01, "{low}");
        assert_eq!(worst, 20.0);
    }
}

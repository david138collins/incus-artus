//! Temperatures — how hot the computer is running, in the top right corner while you play.
//!
//! Read every few seconds on a background thread (each reading takes about a second), so the game
//! never waits for them:
//! - **CPU area:** Windows' thermal zone for the motherboard round the CPU (`typeperf`, no admin
//!   needed). It follows the CPU, but slower and lower than the cores themselves: those need a
//!   driver-level tool (HWiNFO, LibreHardwareMonitor) run as administrator.
//! - **GPU:** the NVIDIA card's own reading (`nvidia-smi`).
//!
//! Either is left out if the computer doesn't offer it. Switched on and off on the Performance page.

use crate::screen::Screen;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// How often the temperatures are read.
const EVERY: Duration = Duration::from_secs(3);

pub struct TemperaturesPlugin;

impl Plugin for TemperaturesPlugin {
    fn build(&self, app: &mut App) {
        let readings = Readings::default();
        let shared = readings.0.clone();
        // `INCUS_TEMPERATURES=0`: not read at all (it starts two programs every few seconds; to
        // measure the game without its own gauge).
        if !std::env::var("INCUS_TEMPERATURES").is_ok_and(|v| v == "0") {
            std::thread::Builder::new()
                .name("temperatures".into())
                .spawn(move || {
                    loop {
                        let now = Temperatures { cpu_area: cpu_area(), gpu: gpu() };
                        if let Ok(mut latest) = shared.lock() {
                            *latest = now;
                        }
                        std::thread::sleep(EVERY);
                    }
                })
                .ok();
        }
        app.insert_resource(readings)
            .insert_resource(ShowTemperatures(true))
            .add_systems(EguiPrimaryContextPass, draw_temperatures.run_if(in_state(Screen::Play)));
    }
}

/// The latest readings, in °C (`None` = not offered, or not read yet).
#[derive(Clone, Copy, Default)]
struct Temperatures {
    cpu_area: Option<f32>,
    gpu: Option<f32>,
}

#[derive(Resource, Default)]
struct Readings(Arc<Mutex<Temperatures>>);

#[derive(Resource)]
struct ShowTemperatures(bool);

/// Runs a command without flashing a console window, and gives back what it printed.
fn output_of(program: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new(program);
    command.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command.output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

/// The hottest thermal zone, from `typeperf` (tenths of a kelvin).
fn cpu_area() -> Option<f32> {
    let text = output_of("typeperf", &[r"\Thermal Zone Information(*)\High Precision Temperature", "-sc", "1"])?;
    // The second line holds the values: "time","zone 1","zone 2"...
    let values = text.lines().find(|line| line.starts_with("\"") && !line.starts_with("\"(PDH"))?;
    values
        .split(',')
        .skip(1)
        .filter_map(|value| value.trim_matches('"').parse::<f32>().ok())
        .map(|tenths| tenths / 10.0 - 273.15)
        .reduce(f32::max)
}

/// The hottest NVIDIA card, from `nvidia-smi`.
fn gpu() -> Option<f32> {
    let text = output_of("nvidia-smi", &["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])?;
    text.lines().filter_map(|line| line.trim().parse::<f32>().ok()).reduce(f32::max)
}

/// Green while comfortable, yellow when warm, red when hot.
fn colour(celsius: f32) -> egui::Color32 {
    if celsius < 70.0 {
        egui::Color32::from_rgb(120, 220, 120)
    } else if celsius < 85.0 {
        egui::Color32::from_rgb(230, 200, 90)
    } else {
        egui::Color32::from_rgb(235, 90, 80)
    }
}

fn draw_temperatures(mut contexts: EguiContexts, readings: Res<Readings>, show: Res<ShowTemperatures>) -> Result {
    if !show.0 {
        return Ok(());
    }
    let Ok(now) = readings.0.lock().map(|latest| *latest) else { return Ok(()) };
    let lines: Vec<(String, f32)> = [("CPU area", now.cpu_area), ("GPU", now.gpu)]
        .into_iter()
        .filter_map(|(name, reading)| reading.map(|c| (format!("{name} {c:.0} °C"), c)))
        .collect();
    if lines.is_empty() {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("temperatures")));
    let top_right = ctx.content_rect().right_top() + egui::vec2(-10.0, 8.0);
    for (k, (text, celsius)) in lines.iter().enumerate() {
        let at = top_right + egui::vec2(0.0, k as f32 * 18.0);
        // A dark edge, so it reads over sky and snow alike.
        painter.text(at + egui::vec2(1.0, 1.0), egui::Align2::RIGHT_TOP, text, egui::FontId::proportional(15.0), egui::Color32::BLACK);
        painter.text(at, egui::Align2::RIGHT_TOP, text, egui::FontId::proportional(15.0), colour(*celsius));
    }
    Ok(())
}

/// The switch and notes, on the Performance page.
pub fn temperatures_settings(ui: &mut egui::Ui, world: &mut World) {
    let mut show = world.resource_mut::<ShowTemperatures>();
    ui.checkbox(&mut show.0, "show temperatures (top right)");
    ui.small(
        "CPU area: the motherboard's thermal zone round the CPU (Windows' own reading). It follows the CPU, \
         but slower and lower than the cores themselves, which need HWiNFO or LibreHardwareMonitor run as \
         administrator. GPU: the NVIDIA card's own reading. Read every 3 seconds. Green under 70 °C, \
         yellow under 85, red above.",
    );
}

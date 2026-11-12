//! The temperatures: how hot the computer runs, in the top right corner while you play, in the session's log every half
//! minute, and in a flight's report [David, 2026-11-12: "i'd like for you to start tracking temperatures again, just as
//! before"; always shown, no switch. His "i dont need to see the temp readout" answered whether he needed a launch to
//! see it; Builder took it off the screen by mistake, and put it back: "why aren't temps shown on the screen". Then:
//! "in the future, can you make sure that the temp displays are always visible while in the world", flights too, since
//! he watches them: "id like to know what the temps are at while watching"].
//!
//! Read beside the watts, the clocks and the frame rate, not alone: the laptop runs Turbo with its fans full, and its
//! CPU area touches 100 °C as a matter of course [David, via Counsel]. A card slowed by heat shows first in its clock.
//!
//! Read every few seconds on a thread of their own (a reading takes about a second), so the frame never waits for them,
//! as old read them:
//! - **The CPU area:** Windows' thermal zone for the motherboard round the processor (`typeperf`, no administrator
//!   needed). It follows the processor, but slower and lower than its cores, which need a driver-level tool (HWiNFO,
//!   LibreHardwareMonitor) run as administrator. On the laptop it's `_TZ.TZ01`; 105 °C is the cutoff David agreed for
//!   long runs.
//! - **The GPU:** the NVIDIA card's own reading, with its clock (`nvidia-smi`): a card slowed by its heat shows in its
//!   clock.
//!
//! - **The processor's cores, its watts and load, and the card's watts and load:** from LibreHardwareMonitor, if it's
//!   running as administrator with its web server on (Options, Remote Web Server; `localhost:8085`) [David installed it
//!   2026-11-12, to see "how much the graphics will use the cpu"]. Windows alone gives only the board's reading.
//!
//! Each is left out if the computer doesn't offer it. `INCUS_TEMPERATURES=0`: not read at all (it starts two programs
//! every few seconds), to measure the game without them.

use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// How often they're read, and every how many readings one goes in the session's log (half a minute).
const EVERY: Duration = Duration::from_secs(3);
const LOGGED: u32 = 10;

/// A reading, in °C (none: not offered, or not read yet), and the card's clock (MHz).
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Temperatures {
    pub cpu_area: Option<f32>,
    pub gpu: Option<f32>,
    pub gpu_clock: Option<f32>,
    /// From LibreHardwareMonitor: the hottest core (°C), the processor's watts and load (%), the card's watts and load.
    pub cpu_core: Option<f32>,
    pub cpu_watts: Option<f32>,
    pub cpu_load: Option<f32>,
    pub gpu_watts: Option<f32>,
    pub gpu_load: Option<f32>,
}

impl Temperatures {
    /// One line of everything read, for a log or a report.
    pub fn line(&self) -> String {
        let read = |value: Option<f32>, unit: &str| value.map_or("not read".to_string(), |v| format!("{v:.0} {unit}"));
        format!(
            "the CPU area {}, its hottest core {}, {} at {} load; the GPU {} at {}, {} at {} load",
            read(self.cpu_area, "°C"),
            read(self.cpu_core, "°C"),
            read(self.cpu_watts, "W"),
            read(self.cpu_load, "%"),
            read(self.gpu, "°C"),
            read(self.gpu_clock, "MHz"),
            read(self.gpu_watts, "W"),
            read(self.gpu_load, "%")
        )
    }
}

/// The temperatures read on a thread of their own, the latest kept.
pub struct Watch {
    latest: Arc<Mutex<Temperatures>>,
}

impl Watch {
    /// The thread started (none if `INCUS_TEMPERATURES=0`).
    pub fn start() -> Option<Self> {
        if std::env::var("INCUS_TEMPERATURES").is_ok_and(|v| v == "0") {
            log::info!("The temperatures: not read (INCUS_TEMPERATURES=0)");
            return None;
        }
        let latest = Arc::new(Mutex::new(Temperatures::default()));
        let shared = latest.clone();
        std::thread::Builder::new()
            .name("reading the temperatures".into())
            .spawn(move || {
                for n in 0u32.. {
                    let (gpu, gpu_clock) = gpu();
                    let mut now = Temperatures { cpu_area: cpu_area(), gpu, gpu_clock, ..Default::default() };
                    if let Some(text) = monitor() {
                        now.cpu_core = sensor(&text, "/intelcpu/0/temperature/0").or_else(|| sensor(&text, "/amdcpu/0/temperature/0"));
                        now.cpu_watts = sensor(&text, "/intelcpu/0/power/0").or_else(|| sensor(&text, "/amdcpu/0/power/0"));
                        now.cpu_load = sensor(&text, "/intelcpu/0/load/0").or_else(|| sensor(&text, "/amdcpu/0/load/0"));
                        now.gpu_watts = sensor(&text, "/gpu-nvidia/0/power/0");
                        now.gpu_load = sensor(&text, "/gpu-nvidia/0/load/0");
                    }
                    if let Ok(mut latest) = shared.lock() {
                        *latest = now;
                    }
                    if n % LOGGED == 0 {
                        log::info!("The temperatures: {}", now.line());
                    }
                    std::thread::sleep(EVERY);
                }
            })
            .ok()?;
        log::info!("The temperatures: read every {} s (the CPU area by typeperf, the GPU by nvidia-smi)", EVERY.as_secs());
        Some(Self { latest })
    }

    /// The latest reading.
    pub fn latest(&self) -> Temperatures {
        self.latest.lock().map(|now| *now).unwrap_or_default()
    }
}

/// A program run without flashing a console window, and what it printed.
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
    cpu_area_of(&text)
}

/// `typeperf`'s answer read: its second line holds the values, "time","zone 1","zone 2"...
fn cpu_area_of(text: &str) -> Option<f32> {
    let values = text.lines().find(|line| line.starts_with('"') && !line.starts_with("\"(PDH"))?;
    values.split(',').skip(1).filter_map(|value| value.trim_matches('"').parse::<f32>().ok()).map(|tenths| tenths / 10.0 - 273.15).reduce(f32::max)
}

/// The hottest NVIDIA card and its graphics clock (MHz), from `nvidia-smi`.
fn gpu() -> (Option<f32>, Option<f32>) {
    output_of("nvidia-smi", &["--query-gpu=temperature.gpu,clocks.gr", "--format=csv,noheader,nounits"]).map_or((None, None), |text| gpu_of(&text))
}

/// `nvidia-smi`'s answer read: a line a card, "celsius, MHz".
fn gpu_of(text: &str) -> (Option<f32>, Option<f32>) {
    let mut hottest: Option<(f32, Option<f32>)> = None;
    for line in text.lines() {
        let mut values = line.split(',').map(|value| value.trim().parse::<f32>().ok());
        let (Some(Some(celsius)), clock) = (values.next(), values.next().flatten()) else { continue };
        if hottest.is_none_or(|(have, _)| celsius > have) {
            hottest = Some((celsius, clock));
        }
    }
    (hottest.map(|(celsius, _)| celsius), hottest.and_then(|(_, clock)| clock))
}

/// LibreHardwareMonitor's sensors as it serves them on this machine (`localhost:8085/data.json`), if it's running.
fn monitor() -> Option<String> {
    use std::io::{Read, Write};
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], 8085));
    let mut stream = std::net::TcpStream::connect_timeout(&address, Duration::from_millis(300)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    stream.write_all(b"GET /data.json HTTP/1.0\r\nHost: localhost\r\n\r\n").ok()?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).to_string())
}

/// A sensor's value by its id in LibreHardwareMonitor's answer: the number its "Value" begins with, just before the id.
fn sensor(text: &str, id: &str) -> Option<f32> {
    let at = text.find(&format!("\"SensorId\":\"{id}\""))?;
    let value = text[..at].rfind("\"Value\":\"")? + 9;
    let number: String = text[value..].chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-').collect();
    number.replace(',', ".").parse().ok()
}

/// Green while comfortable, yellow when warm, red when hot (old's).
fn colour(celsius: f32) -> egui::Color32 {
    if celsius < 70.0 {
        egui::Color32::from_rgb(120, 220, 120)
    } else if celsius < 85.0 {
        egui::Color32::from_rgb(230, 200, 90)
    } else {
        egui::Color32::from_rgb(235, 90, 80)
    }
}

/// The readings drawn in the top right corner, over everything, a dark edge under each so it reads over sky and snow.
pub fn draw(ctx: &egui::Context, now: Temperatures) {
    // Each line its degrees, then its watts and load where LibreHardwareMonitor gives them.
    let more = |watts: Option<f32>, load: Option<f32>| format!("{}{}", watts.map_or(String::new(), |w| format!("  {w:.0} W")), load.map_or(String::new(), |l| format!("  {l:.0}%")));
    let lines: Vec<(String, f32)> = [("CPU area", now.cpu_area, String::new()), ("CPU cores", now.cpu_core, more(now.cpu_watts, now.cpu_load)), ("GPU", now.gpu, more(now.gpu_watts, now.gpu_load))]
        .into_iter()
        .filter_map(|(name, reading, rest)| reading.map(|c| (format!("{name} {c:.0} °C{rest}"), c)))
        .collect();
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("temperatures")));
    let top_right = ctx.content_rect().right_top() + egui::vec2(-10.0, 8.0);
    for (k, (text, celsius)) in lines.iter().enumerate() {
        let at = top_right + egui::vec2(0.0, k as f32 * 18.0);
        painter.text(at + egui::vec2(1.0, 1.0), egui::Align2::RIGHT_TOP, text, egui::FontId::proportional(15.0), egui::Color32::BLACK);
        painter.text(at, egui::Align2::RIGHT_TOP, text, egui::FontId::proportional(15.0), colour(*celsius));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two programs' answers read as they print them on the laptop (2026-11-12), and nothing made of nothing.
    #[test]
    fn the_readings_are_read() {
        let typeperf = "\"(PDH-CSV 4.0)\",\"\\\\DTC\\Thermal Zone Information(\\_TZ.TZ01)\\High Precision Temperature\"\r\n\"11/12/2026 13:39:22.332\",\"3742.000000\"\r\nExiting, please wait...\r\n";
        assert!((cpu_area_of(typeperf).unwrap() - 101.05).abs() < 0.01);
        assert_eq!(gpu_of("56.09, 1987\n"), (Some(56.09), Some(1987.0)));
        let monitor = r#"{"id":31,"Text":"CPU Package","Min":"17.5 W","Value":"34.9 W","Max":"48.7 W","SensorId":"/intelcpu/0/power/0","Type":"Power"},{"id":40,"Text":"Core Max","Min":"50.0 °C","Value":"74.0 °C","Max":"90.0 °C","SensorId":"/intelcpu/0/temperature/0","Type":"Temperature"}"#;
        assert_eq!((sensor(monitor, "/intelcpu/0/power/0"), sensor(monitor, "/intelcpu/0/temperature/0"), sensor(monitor, "/gpu-nvidia/0/power/0")), (Some(34.9), Some(74.0), None));
        assert_eq!(gpu_of("41, 210\n63, 1650\n"), (Some(63.0), Some(1650.0)), "the hottest card");
        assert_eq!(cpu_area_of("Exiting\n"), None);
        assert_eq!(gpu_of(""), (None, None));
    }
}

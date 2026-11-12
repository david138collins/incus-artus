//! The temperatures: how hot the computer runs, in the session's log every half minute and in a flight's report
//! [David, 2026-11-12: "i'd like for you to start tracking temperatures again, just as before"; then, of the corner
//! readout build 135 drew: "i dont need to see the temp readout"].
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
//! Either is left out if the computer doesn't offer it. `INCUS_TEMPERATURES=0`: not read at all (it starts two programs
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
                    let now = Temperatures { cpu_area: cpu_area(), gpu, gpu_clock };
                    if let Ok(mut latest) = shared.lock() {
                        *latest = now;
                    }
                    if n % LOGGED == 0 {
                        let read = |value: Option<f32>, unit: &str| value.map_or("not read".to_string(), |v| format!("{v:.0} {unit}"));
                        log::info!("The temperatures: the CPU area {}, the GPU {} at {}", read(now.cpu_area, "°C"), read(now.gpu, "°C"), read(now.gpu_clock, "MHz"));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The two programs' answers read as they print them on the laptop (2026-11-12), and nothing made of nothing.
    #[test]
    fn the_readings_are_read() {
        let typeperf = "\"(PDH-CSV 4.0)\",\"\\\\DTC\\Thermal Zone Information(\\_TZ.TZ01)\\High Precision Temperature\"\r\n\"11/12/2026 13:39:22.332\",\"3742.000000\"\r\nExiting, please wait...\r\n";
        assert!((cpu_area_of(typeperf).unwrap() - 101.05).abs() < 0.01);
        assert_eq!(gpu_of("56.09, 1987\n"), (Some(56.09), Some(1987.0)));
        assert_eq!(gpu_of("41, 210\n63, 1650\n"), (Some(63.0), Some(1650.0)), "the hottest card");
        assert_eq!(cpu_area_of("Exiting\n"), None);
        assert_eq!(gpu_of(""), (None, None));
    }
}

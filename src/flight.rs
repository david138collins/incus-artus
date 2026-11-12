//! A measured flight (`INCUS_FLIGHT=1`): the camera stands still, the frames settle, then they're measured, a report is
//! written to `logs/flight-<seconds since 1970>.txt`, and the game quits.
//!
//! Uncapped (frames handed over as soon as they're made) and with the panel hidden, so what's measured is the frame
//! alone. Today that's the empty frame (plan 2.1's gate); marks as data come later, and the flight with them.
//!
//! Each pass has a line of its own, "    <its name>: ...", which the software map's test reads for its costs.

use crate::clocks::Spread;
use crate::wiring::{self, PASSES, PassId, Where};
use std::time::{Duration, Instant};

/// How long the frames settle before they're measured.
const SETTLE: Duration = Duration::from_secs(5);
/// How long they're measured (old's standing marks: 20 s).
const MEASURE: Duration = Duration::from_secs(20);

/// Whether this session is a flight.
pub fn wanted() -> bool {
    std::env::var_os("INCUS_FLIGHT").is_some()
}

pub struct Flight {
    started: Instant,
    /// The measured frames' lengths, ms.
    frames: Vec<f32>,
    /// Each pass's measured times on the processor, by its id.
    passes: Vec<Vec<f32>>,
    /// The measured frames' card times: each card slot's ms, the card's whole last.
    card: Vec<Vec<f32>>,
}

/// What the report says about the session.
pub struct About<'a> {
    pub card: &'a str,
    pub picture: (u32, u32),
    pub present: wgpu::PresentMode,
    pub refresh: f32,
    pub camera: &'a crate::camera::Camera,
}

impl Flight {
    pub fn new() -> Self {
        log::info!("A flight: {} s settling, then {} s measured", SETTLE.as_secs(), MEASURE.as_secs());
        Self { started: Instant::now(), frames: Vec::new(), passes: vec![Vec::new(); PASSES.len()], card: Vec::new() }
    }

    fn measuring(&self, now: Instant) -> bool {
        let since = now.duration_since(self.started);
        since >= SETTLE && since < SETTLE + MEASURE
    }

    /// A pass ran for `ms` on the processor.
    pub fn pass(&mut self, now: Instant, pass: PassId, ms: f32) {
        if self.measuring(now) {
            self.passes[pass as usize].push(ms);
        }
    }

    /// A frame shown (its length), and the card's times that came back with it.
    pub fn record(&mut self, now: Instant, frame_ms: Option<f32>, card: Vec<Vec<f32>>) {
        if !self.measuring(now) {
            return;
        }
        self.frames.extend(frame_ms);
        self.card.extend(card);
    }

    /// Whether the measuring is over.
    pub fn finished(&self, now: Instant) -> bool {
        now.duration_since(self.started) >= SETTLE + MEASURE
    }

    /// The report, written to `logs/`.
    pub fn report(&self, about: &About) {
        let mut text = String::new();
        let code = if crate::session::BUILD_CHANGED.is_empty() { "the code committed".to_string() } else { format!("the code with changes: {}", crate::session::BUILD_CHANGED) };
        text += &format!("The flight: Incus Artus, {}; {code}\n", crate::session::build_line());
        text += &format!("The card: {}\n", about.card);
        text += &format!("The picture {} x {}, the screen's refresh {:.0}, uncapped; handed to the screen: {:?}; the panel hidden\n", about.picture.0, about.picture.1, about.refresh, about.present);
        // The switches set for this session. Not the build's own stamp (`INCUS_BUILD...`): `cargo run` hands the build
        // script's values to the game as it starts, and they're in the first line already.
        let mut switches: Vec<String> = std::env::vars()
            .filter(|(name, _)| (name.starts_with("INCUS_") && !name.starts_with("INCUS_BUILD")) || name.starts_with("WGPU_"))
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        switches.sort();
        text += &format!("Switches: {}\n\n", switches.join(", "));
        let camera = about.camera;
        text += &format!(
            "Standing: the camera at north {:.1}, east {:.1}, up {:.1}, facing {:.0}°, pitch {:.1}°; {} s settling, then {} s measured\n",
            camera.position.x,
            camera.position.y,
            camera.position.z,
            camera.facing,
            camera.pitch,
            SETTLE.as_secs(),
            MEASURE.as_secs()
        );
        let frames = Spread::of(self.frames.iter().copied());
        let fps = if frames.median > 0.0 { 1000.0 / frames.median } else { 0.0 };
        text += &format!(
            "  the frame: {} frames, {fps:.0} a second (median); ms median {:.3}, 95th {:.3}, 99th {:.3}, 99.9th {:.3}, worst {:.3}\n",
            frames.count, frames.median, frames.p95, frames.p99, frames.p999, frames.worst
        );
        let spread = |spread: Spread| format!("median {:.3}, 95th {:.3}, 99th {:.3}, worst {:.3}", spread.median, spread.p95, spread.p99, spread.worst);
        text += &format!("  each pass (ms; {} frames timed on the card):\n", self.card.len());
        for pass in &PASSES {
            let times = &self.passes[pass.id as usize];
            let on_processor = if pass.on == Where::Card { "recording it on the processor" } else { "on the processor" };
            let mut line = format!("    {}: {on_processor} {}", pass.name, if times.is_empty() { "not run".to_string() } else { spread(Spread::of(times.iter().copied())) });
            if let Some(slot) = wiring::card_slot(pass.id) {
                let card = Spread::of(self.card.iter().filter_map(|frame| frame.get(slot).copied()));
                line += &if card.count == 0 { "; on the card not timed".to_string() } else { format!("; on the card {}", spread(card)) };
            }
            text += &line;
            text += "\n";
        }
        if !self.card.is_empty() {
            text += &format!("  the card's whole: {}\n", spread(Spread::of(self.card.iter().filter_map(|frame| frame.last().copied()))));
        }
        let path = crate::session::logs_folder().join(format!("flight-{}.txt", crate::session::unix_seconds()));
        match std::fs::write(&path, &text) {
            Ok(()) => log::info!("The flight's report: {}", path.display()),
            Err(error) => log::error!("The flight's report couldn't be written to {}: {error}", path.display()),
        }
        for line in text.lines() {
            log::info!("{line}");
        }
    }
}

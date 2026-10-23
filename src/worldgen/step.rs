//! Steps — the pieces a generator is stacked from.
//!
//! A step is one settings struct that knows two things:
//! - `tune`: how to show its settings as sliders;
//! - `run`: what it does to the map, given those settings.
//!
//! That's all a new step needs. The suite does the rest: saving and loading its settings in
//! presets, noticing when a slider moved and re-running from that step down, running it in the
//! background, timing it, keeping a copy of the map after it so you can look back, and showing its
//! log lines.
//!
//! **Each step rolls its own dice** (from the world seed and the step's name), so changing one
//! step's settings never changes the randomness of another.

use super::map::Map;
use crate::fate::derive_seed;
use bevy_egui::egui;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Write one of these to make a step.
pub trait Rules: Default + Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Shown in the step list, and the step's name in presets. Changing it orphans old presets.
    const NAME: &'static str;
    /// One line about what the step does.
    const ABOUT: &'static str;
    fn tune(&mut self, ui: &mut egui::Ui);
    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped>;
}

/// The step was asked to stop part-way (a newer run replaced it).
#[derive(Debug)]
pub struct Stopped;

/// A line in the generation log. `at` is a cell, so clicking the line can show where.
#[derive(Clone, Debug)]
pub struct LogLine {
    pub step: &'static str,
    pub text: String,
    pub at: Option<(f32, f32)>,
}

/// What a running step is given besides the map: its dice, its log, and the stop signal.
pub struct Work {
    /// This step's own seed (see the module notes).
    pub seed: u64,
    pub step: &'static str,
    pub lines: Vec<LogLine>,
    stop: Arc<AtomicBool>,
}

impl Work {
    pub fn new(world_seed: u64, step: &'static str, stop: Arc<AtomicBool>) -> Self {
        Self { seed: derive_seed(world_seed, step), step, lines: Vec::new(), stop }
    }

    /// Call this now and then in long loops: it bails out if the run was replaced.
    pub fn check(&self) -> Result<(), Stopped> {
        if self.stop.load(Ordering::Relaxed) { Err(Stopped) } else { Ok(()) }
    }

    pub fn log(&mut self, text: impl Into<String>) {
        self.lines.push(LogLine { step: self.step, text: text.into(), at: None });
    }

    pub fn log_at(&mut self, col: f32, row: f32, text: impl Into<String>) {
        self.lines.push(LogLine { step: self.step, text: text.into(), at: Some((col, row)) });
    }
}

/// Any step, whatever its settings look like (what the suite keeps in its list).
pub trait Step: Send + Sync {
    fn name(&self) -> &'static str;
    fn about(&self) -> &'static str;
    fn tune(&mut self, ui: &mut egui::Ui);
    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped>;
    /// The settings as TOML (also how the suite notices that a slider moved).
    fn save(&self) -> toml::Table;
    fn load(&mut self, settings: &toml::Table) -> Result<(), String>;
    fn reset(&mut self);
    fn copy(&self) -> Box<dyn Step>;
}

/// Wraps a `Rules` so it can sit in the suite's list.
pub struct Stage<R: Rules>(pub R);

impl<R: Rules> Step for Stage<R> {
    fn name(&self) -> &'static str {
        R::NAME
    }
    fn about(&self) -> &'static str {
        R::ABOUT
    }
    fn tune(&mut self, ui: &mut egui::Ui) {
        self.0.tune(ui);
    }
    fn run(&self, map: &mut Map, work: &mut Work) -> Result<(), Stopped> {
        self.0.run(map, work)
    }
    fn save(&self) -> toml::Table {
        toml::to_string(&self.0).ok().and_then(|text| toml::from_str(&text).ok()).unwrap_or_default()
    }
    fn load(&mut self, settings: &toml::Table) -> Result<(), String> {
        let text = toml::to_string(settings).map_err(|error| error.to_string())?;
        self.0 = toml::from_str(&text).map_err(|error| error.to_string())?;
        Ok(())
    }
    fn reset(&mut self) {
        self.0 = R::default();
    }
    fn copy(&self) -> Box<dyn Step> {
        Box::new(Stage(self.0.clone()))
    }
}

/// A labelled slider, the usual way a step shows a setting.
pub fn slider<N: egui::emath::Numeric>(ui: &mut egui::Ui, value: &mut N, range: std::ops::RangeInclusive<N>, label: &str) {
    ui.add(egui::Slider::new(value, range).text(label));
}

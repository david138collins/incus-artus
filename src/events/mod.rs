//! Events — the event bus: how the parts of the game tell each other what's happening.
//!
//! Two kinds of message travel on it:
//! - **Requests**: something someone *wants* ("please break this block"). The system that
//!   owns that thing checks it, then either carries it out or refuses it, with a reason.
//! - **Announcements**: facts that already happened ("this block changed", "a footstep",
//!   "landed"). Anything can listen; nobody has to.
//!
//! Systems never reach into each other to change things; they send requests and listen to
//! announcements. That keeps every part separate, and means a player's actions could just
//! as well come from another machine later (multiplayer) or from a recording (replays, tests).
//!
//! Changes to the world happen on simulation **ticks**, so a paused world is truly frozen.
//! Announcements carry the tick they happened on.
//!
//! Every message type registered with `app.log_event::<T>()` is recorded in the **event log**:
//! shown on the "Events" debug page, and (optionally) written to a file in `logs/`, one
//! file per session, ready to attach to a bug report.
//!
//! Each player has a `PlayerId`. The one at this keyboard also has the `LocalPlayer` marker.

use crate::data::BlockRegistry;
use crate::debug::{DebugAppExt, SettingsAppExt, project_folder};
use crate::time::TimeControl;
use crate::ecs::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Which player someone is. Player 0 is the one at this keyboard (for now, the only one).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u8);

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "player {}", self.0)
    }
}

/// Marks the player controlled by this keyboard and mouse.
#[derive(Component)]
pub struct LocalPlayer;

/// A message that goes in the event log.
pub trait LoggedEvent: Message {
    /// A short name for the kind of event, e.g. "block changed". Used for filtering.
    const KIND: &'static str;
    /// One line describing it.
    fn describe(&self, blocks: &BlockRegistry) -> String;
    /// The tick it happened on, if it knows (announcements do). Otherwise the log uses
    /// the tick it was noticed on.
    fn tick(&self) -> Option<u64> {
        None
    }
}

/// Adds `app.log_event::<T>()` to Bevy's `App`.
pub trait EventLogAppExt {
    fn log_event<T: LoggedEvent>(&mut self) -> &mut Self;
}

impl EventLogAppExt for App {
    fn log_event<T: LoggedEvent>(&mut self) -> &mut Self {
        self.world_mut().get_resource_or_init::<EventLog>().kinds.insert(T::KIND);
        // `Last`: after everything else this frame, so nothing sent this frame is missed.
        self.add_systems(Last, record::<T>.before(write_log_file))
    }
}

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventLog>()
            .init_resource::<EventLogSettings>()
            .add_systems(Last, write_log_file)
            .add_debug_page("Events", events_debug_page)
            .add_settings::<EventLogSettings>("event_log");
    }
}

/// Tunables (shown on the "Events" debug page).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EventLogSettings {
    /// Write every event to a file in `logs/` as well.
    pub write_to_file: bool,
    /// How many recent events the debug page keeps.
    pub keep: usize,
}

impl Default for EventLogSettings {
    fn default() -> Self {
        Self {
            write_to_file: true,
            keep: 500,
        }
    }
}

/// How many session log files to keep in `logs/` (the oldest are deleted).
const LOG_FILES_KEPT: usize = 20;

struct LogEntry {
    tick: u64,
    kind: &'static str,
    text: String,
}

/// The recorded events, and the log file.
#[derive(Resource, Default)]
pub struct EventLog {
    entries: VecDeque<LogEntry>,
    /// Every kind registered, and how many of each have happened.
    kinds: BTreeSet<&'static str>,
    counts: BTreeMap<&'static str, u64>,
    /// Kinds hidden on the debug page (still counted and written to the file).
    hidden: BTreeSet<&'static str>,
    /// Freeze the list on the debug page (to read it); the file keeps going.
    frozen: bool,
    /// Lines waiting to be written to the file.
    unwritten: Vec<String>,
    file: Option<BufWriter<File>>,
    file_path: Option<PathBuf>,
    file_problem: Option<String>,
}

fn record<T: LoggedEvent>(
    mut messages: MessageReader<T>,
    mut log: ResMut<EventLog>,
    time: Res<TimeControl>,
    settings: Res<EventLogSettings>,
    blocks: Res<BlockRegistry>,
) {
    for message in messages.read() {
        let entry = LogEntry {
            tick: message.tick().unwrap_or(time.tick()),
            kind: T::KIND,
            text: message.describe(&blocks),
        };
        *log.counts.entry(T::KIND).or_default() += 1;
        if settings.write_to_file {
            let line = format!("tick {:>8}  {:<14}  {}", entry.tick, entry.kind, entry.text);
            log.unwritten.push(line);
        }
        if !log.frozen {
            log.entries.push_front(entry);
            log.entries.truncate(settings.keep.max(1));
        }
    }
}

/// Opens `logs/events-<time>.log` the first time it's needed, and deletes the oldest logs.
fn open_log_file() -> Result<(BufWriter<File>, PathBuf), String> {
    let folder = project_folder().join("logs");
    std::fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

    // Keep the newest few; file names sort by time.
    let mut old: Vec<PathBuf> = std::fs::read_dir(&folder)
        .map_err(|error| error.to_string())?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("events-")))
        .collect();
    old.sort();
    while old.len() >= LOG_FILES_KEPT {
        let _ = std::fs::remove_file(old.remove(0));
    }

    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let path = folder.join(format!("events-{seconds}.log"));
    let mut file = BufWriter::new(File::create(&path).map_err(|error| error.to_string())?);
    let _ = writeln!(file, "Incus Artus event log (started {seconds}, seconds since 1970)");
    Ok((file, path))
}

/// Writes the waiting lines to the file, once per frame.
fn write_log_file(mut log: ResMut<EventLog>, settings: Res<EventLogSettings>) {
    if !settings.write_to_file || log.unwritten.is_empty() {
        log.unwritten.clear();
        return;
    }
    if log.file.is_none() && log.file_problem.is_none() {
        match open_log_file() {
            Ok((file, path)) => {
                info!("Writing the event log to {}", path.display());
                log.file = Some(file);
                log.file_path = Some(path);
            }
            Err(problem) => {
                warn!("Couldn't open the event log file: {problem}");
                log.file_problem = Some(problem);
            }
        }
    }
    let lines = std::mem::take(&mut log.unwritten);
    if let Some(file) = &mut log.file {
        for line in lines {
            let _ = writeln!(file, "{line}");
        }
        let _ = file.flush();
    }
}

// ── Debug page ──────────────────────────────────────────────────────────────

fn events_debug_page(ui: &mut egui::Ui, world: &mut World) {
    {
        let mut settings = world.resource_mut::<EventLogSettings>();
        ui.checkbox(&mut settings.write_to_file, "write events to a log file");
        ui.add(crate::debug::Slider::new(&mut settings.keep, 50..=5000).logarithmic(true).text("events kept on this page"));
    }
    let mut log = world.resource_mut::<EventLog>();
    match (&log.file_path, &log.file_problem) {
        (Some(path), _) => {
            ui.small(format!("Log file: {}", path.display()));
        }
        (None, Some(problem)) => {
            ui.colored_label(egui::Color32::from_rgb(255, 80, 200), format!("Log file problem: {problem}"));
        }
        (None, None) => {
            ui.small("The log file starts with the first event.");
        }
    }
    ui.horizontal(|ui| {
        if ui.button("Open logs folder").clicked() {
            let folder = project_folder().join("logs");
            let _ = std::fs::create_dir_all(&folder);
            let _ = std::process::Command::new("explorer").arg(&folder).spawn();
        }
        let label = if log.frozen { "Unfreeze list" } else { "Freeze list" };
        if ui.button(label).clicked() {
            log.frozen = !log.frozen;
        }
        if ui.button("Clear list").clicked() {
            log.entries.clear();
        }
    });

    // Filters: one switch per kind of event, with how many have happened.
    ui.separator();
    ui.strong("Show");
    let kinds: Vec<&'static str> = log.kinds.iter().copied().collect();
    ui.horizontal_wrapped(|ui| {
        for kind in kinds {
            let count = log.counts.get(kind).copied().unwrap_or(0);
            let mut shown = !log.hidden.contains(kind);
            if ui.checkbox(&mut shown, format!("{kind} ({count})")).changed() {
                if shown {
                    log.hidden.remove(kind);
                } else {
                    log.hidden.insert(kind);
                }
            }
        }
    });

    // The list, newest first.
    ui.separator();
    if log.frozen {
        ui.small("(frozen: new events still go to the file)");
    }
    let mut shown = 0;
    for entry in log.entries.iter().filter(|entry| !log.hidden.contains(entry.kind)) {
        ui.monospace(format!("{:>7}  {:<13} {}", entry.tick, entry.kind, entry.text));
        shown += 1;
        if shown >= 200 {
            ui.small("(older events are in the log file)");
            break;
        }
    }
    if shown == 0 {
        ui.label("No events yet. Walk, jump, break or place something.");
    }
}

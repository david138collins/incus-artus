//! Diagnostics — finding out what went wrong, after the fact. [David, 2026-10-27: "build yourself
//! a toolkit for identifying when something goes wrong"; prompted by a freeze on Ctrl+S that left
//! no trace at all.]
//!
//! Everything goes in `logs/` (next to `assets/`), costs nothing a frame, and is on in a player's
//! copy too:
//! - **The session log** (`session-<time>.log`): every message and warning the game writes, as it
//!   writes it. The console shows them too, but it's gone once its window closes.
//! - **Crash reports** (`crash-<time>.txt`): when the game panics, what the message was, where in
//!   the code, the chain of calls that led there, and what was going on (below).
//! - **Freeze reports** (`freeze-<time>.txt`): a watchdog thread notices when frames stop for
//!   `FROZEN_AFTER`, and writes what was going on; if the game comes back, it says how long it was
//!   stuck (so long hitches are caught too).
//! - **What was going on**, in every report: where you were (screen, debug page, Editor Suite tab),
//!   your last key presses and clicks, the session log's last lines, and the build: which commit,
//!   and whether it had uncommitted changes.
//! - **Did the last session end normally?** A `running.txt` marker is left in `logs/` while the
//!   game runs, and removed when it closes normally. Found at the next start, the last session
//!   ended some other way (a crash, a freeze closed from Windows, the power), and the game says so
//!   for a few seconds, and on the debug panel's Overview page, with the report that goes with it.
//!
//! `start` is called first thing in `main` (so even a crash while the game is being put together
//! is caught), `log_layer` hands Bevy's logging a copy for the session log, and `ended_normally`
//! is called when the game's loop returns. To check the reports: `INCUS_TEST_FAULT=crash` or `=freeze`.

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::log::BoxedLayer;
use bevy::log::tracing::field::{Field, Visit};
use bevy::log::tracing::{Event, Level, Subscriber};
use bevy::log::tracing_subscriber::Layer;
use bevy::log::tracing_subscriber::layer::Context;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

/// No frame for this long counts as a freeze.
const FROZEN_AFTER: Duration = Duration::from_secs(5);
/// How many session logs, and how many crash and freeze reports, are kept (the oldest go).
const SESSIONS_KEPT: usize = 10;
const REPORTS_KEPT: usize = 20;
/// How many of the session log's last lines, and of the last key presses and clicks, a report holds.
const RECENT_LINES: usize = 80;
const RECENT_INPUTS: usize = 30;

/// Which commit the game was built from, "+ changes" if its code had uncommitted changes (see
/// `build.rs`).
pub const BUILD: &str = match option_env!("INCUS_BUILD") {
    Some(build) => build,
    None => "unknown",
};

/// What this session shares between the game, the watchdog and the panic hook.
struct Session {
    folder: PathBuf,
    /// When it started (seconds since 1970), and the moment, for "how far in".
    started: u64,
    clock: Instant,
    log: Mutex<SessionLog>,
    /// Milliseconds into the session at the last frame, and how many frames so far.
    beat: AtomicU64,
    frames: AtomicU64,
    places: Mutex<Places>,
    /// How the last session ended, found at start-up.
    last_session: Option<String>,
}

struct SessionLog {
    file: Option<std::io::BufWriter<std::fs::File>>,
    path: PathBuf,
    recent: VecDeque<String>,
}

/// Where you are, and what you last pressed.
#[derive(Default)]
struct Places {
    screen: String,
    page: String,
    tab: String,
    inputs: VecDeque<String>,
}

static SESSION: OnceLock<Session> = OnceLock::new();

fn now_seconds() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Starts this session's diagnostics: checks how the last one ended, leaves the "running"
/// marker, opens the session log, installs the crash report, and starts the watchdog.
pub fn start() {
    let folder = crate::debug::project_folder().join("logs");
    let _ = std::fs::create_dir_all(&folder);
    let started = now_seconds();
    let marker = folder.join("running.txt");
    let last_session = std::fs::read_to_string(&marker).ok().map(|text| {
        let since = text.lines().find_map(|line| line.strip_prefix("started ")).and_then(|s| s.trim().parse::<u64>().ok()).unwrap_or(0);
        match newest_report_since(&folder, since) {
            Some(report) => format!("The last session didn't end normally. Its report: logs/{report}"),
            None => "The last session didn't end normally (closed from Windows or the power went), and left no report.".to_string(),
        }
    });
    let _ = std::fs::write(&marker, format!("started {started}\nbuild {BUILD}\nThis file is removed when the game closes normally.\n"));
    tidy(&folder, "session-", SESSIONS_KEPT);
    tidy(&folder, "crash-", REPORTS_KEPT);
    tidy(&folder, "freeze-", REPORTS_KEPT);
    let path = folder.join(format!("session-{started}.log"));
    let file = std::fs::File::create(&path).ok().map(std::io::BufWriter::new);
    let session = Session {
        folder,
        started,
        clock: Instant::now(),
        log: Mutex::new(SessionLog { file, path, recent: VecDeque::new() }),
        beat: AtomicU64::new(0),
        frames: AtomicU64::new(0),
        places: Mutex::new(Places::default()),
        last_session,
    };
    if SESSION.set(session).is_err() {
        return;
    }
    write_line(&format!("Incus Artus session, started {started} (seconds since 1970), build {BUILD}"));
    if let Some(last) = &SESSION.get().unwrap().last_session {
        write_line(last);
    }
    install_crash_report();
    std::thread::Builder::new().name("watchdog".into()).spawn(watchdog).ok();
}

/// Called when the game's loop returns: the session ended normally.
pub fn ended_normally() {
    let Some(session) = SESSION.get() else { return };
    write_line("Ended normally.");
    if let Ok(mut log) = session.log.lock()
        && let Some(file) = log.file.as_mut()
    {
        let _ = file.flush();
    }
    let _ = std::fs::remove_file(session.folder.join("running.txt"));
}

/// The newest crash or freeze report written at or after `since` (seconds since 1970).
fn newest_report_since(folder: &Path, since: u64) -> Option<String> {
    let mut reports: Vec<(u64, String)> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let time = name.strip_prefix("crash-").or_else(|| name.strip_prefix("freeze-"))?.strip_suffix(".txt")?.parse().ok()?;
            Some((time, name))
        })
        .filter(|(time, _)| *time >= since)
        .collect();
    reports.sort();
    reports.pop().map(|(_, name)| name)
}

/// Deletes the oldest files starting with `prefix`, keeping `kept`.
fn tidy(folder: &Path, prefix: &str, kept: usize) {
    let mut files: Vec<PathBuf> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(prefix)))
        .collect();
    files.sort();
    let extra = files.len().saturating_sub(kept);
    for path in &files[..extra] {
        let _ = std::fs::remove_file(path);
    }
}

/// Seconds into the session, for the log.
fn into_session() -> f32 {
    SESSION.get().map_or(0.0, |session| session.clock.elapsed().as_secs_f32())
}

/// Adds a line to the session log (and its recent lines). Never waits on the log if it's busy
/// for long: a report written mid-crash mustn't hang on it.
fn write_line(line: &str) {
    let Some(session) = SESSION.get() else { return };
    let line = format!("{:9.2}  {line}", into_session());
    if let Ok(mut log) = session.log.lock() {
        if let Some(file) = log.file.as_mut() {
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
        log.recent.push_back(line);
        while log.recent.len() > RECENT_LINES {
            log.recent.pop_front();
        }
    }
}

// ── The session log: a copy of everything the game logs ────────────────────

/// Bevy's logging hands every message to this too (see `main`).
pub fn log_layer(_app: &mut App) -> Option<BoxedLayer> {
    Some(Box::new(SessionLayer))
}

struct SessionLayer;

/// Pulls the message (and any other fields) out of a log event.
struct Message(String);

impl Visit for Message {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}{}", self.0);
        } else if !field.name().starts_with("log.") {
            // (Messages from libraries using the older `log` crate carry their own bookkeeping as
            // `log.` fields: left out.)
            self.0.push_str(&format!(" {}={value:?}", field.name()));
        }
    }
}

impl<S: Subscriber> Layer<S> for SessionLayer {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let meta = event.metadata();
        let level = *meta.level();
        // Messages and up from the game and Bevy; only warnings and errors from the graphics
        // libraries, which are chatty.
        let target = meta.target();
        let chatty = target.starts_with("wgpu") || target.starts_with("naga");
        if level > Level::INFO || (chatty && level > Level::WARN) {
            return;
        }
        let mut message = Message(String::new());
        event.record(&mut message);
        write_line(&format!("{level:5} {target}: {}", message.0));
    }
}

// ── Where you are, and what you pressed ─────────────────────────────────────

/// Notes where you are now (`what` is "screen", "page" or "tab").
pub fn place(what: &str, name: &str) {
    let Some(session) = SESSION.get() else { return };
    if let Ok(mut places) = session.places.lock() {
        let slot = match what {
            "screen" => &mut places.screen,
            "page" => &mut places.page,
            _ => &mut places.tab,
        };
        if slot != name {
            *slot = name.to_string();
        }
    }
}

/// Notes something you did (a key press, a click).
fn input(text: String) {
    let Some(session) = SESSION.get() else { return };
    if let Ok(mut places) = session.places.lock() {
        places.inputs.push_back(format!("{:9.2}  {text}", into_session()));
        while places.inputs.len() > RECENT_INPUTS {
            places.inputs.pop_front();
        }
    }
}

// ── Reports ─────────────────────────────────────────────────────────────────

/// What was going on, for a report.
fn going_on(session: &Session) -> String {
    let mut text = String::new();
    let seconds = session.clock.elapsed().as_secs_f32();
    text.push_str(&format!("Build: {BUILD}\nSession started {} (seconds since 1970); {seconds:.1} s in, {} frames.\n", session.started, session.frames.load(Ordering::Relaxed)));
    // `try_lock` throughout: whatever was holding these may be what went wrong.
    match session.places.try_lock() {
        Ok(places) => {
            text.push_str(&format!("Screen: {}\nDebug panel page: {}\nEditor Suite tab: {}\n", or_none(&places.screen), or_none(&places.page), or_none(&places.tab)));
            text.push_str("\nLast key presses and clicks:\n");
            for line in &places.inputs {
                text.push_str(&format!("{line}\n"));
            }
        }
        Err(_) => text.push_str("(where you were couldn't be read)\n"),
    }
    match session.log.try_lock() {
        Ok(log) => {
            text.push_str(&format!("\nThe session log's last lines ({}):\n", log.path.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned())));
            for line in &log.recent {
                text.push_str(&format!("{line}\n"));
            }
        }
        Err(_) => text.push_str("\n(the session log was busy and couldn't be read)\n"),
    }
    text
}

fn or_none(text: &str) -> &str {
    if text.is_empty() { "—" } else { text }
}

/// Writes `logs/<kind>-<time>.txt`, returning its name.
fn write_report(session: &Session, kind: &str, body: &str) -> String {
    let name = format!("{kind}-{}.txt", now_seconds());
    let _ = std::fs::write(session.folder.join(&name), body);
    name
}

/// On a panic (on any thread): a crash report, then Rust's usual message in the console.
fn install_crash_report() {
    let usual = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        static WRITING: AtomicBool = AtomicBool::new(false);
        // One report at a time (a panic while writing one would otherwise loop).
        if let Some(session) = SESSION.get()
            && !WRITING.swap(true, Ordering::SeqCst)
        {
            let message = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_else(|| "(no message)".into());
            let place = info.location().map_or("unknown".to_string(), |at| format!("{}:{}:{}", at.file(), at.line(), at.column()));
            let thread = std::thread::current().name().unwrap_or("unnamed").to_string();
            let backtrace = std::backtrace::Backtrace::force_capture();
            let body = format!(
                "Incus Artus crash report\n\nWhat: {message}\nWhere: {place}\nThread: {thread}\n\n{}\nThe chain of calls (newest first):\n{backtrace}\n",
                going_on(session)
            );
            let name = write_report(session, "crash", &body);
            write_line(&format!("CRASH: {message} at {place} (thread {thread}). Report: logs/{name}"));
            WRITING.store(false, Ordering::SeqCst);
        }
        usual(info);
    }));
}

/// Watches the frames: none for `FROZEN_AFTER` writes a freeze report; when they come back, it
/// says for how long.
fn watchdog() {
    let Some(session) = SESSION.get() else { return };
    // The report, the frame count, and when the last frame before it was (ms into the session).
    let mut frozen: Option<(String, u64, u64)> = None;
    loop {
        std::thread::sleep(Duration::from_millis(500));
        let frames = session.frames.load(Ordering::Relaxed);
        if frames == 0 {
            continue; // Still starting up.
        }
        let now = session.clock.elapsed().as_millis() as u64;
        let since = now.saturating_sub(session.beat.load(Ordering::Relaxed));
        match &frozen {
            None if since >= FROZEN_AFTER.as_millis() as u64 => {
                let body = format!(
                    "Incus Artus freeze report\n\nNo frame for {:.1} s (the window stops responding). If the game comes back, the time it\ncame back is added at the end.\n\n{}",
                    since as f32 / 1000.0,
                    going_on(session)
                );
                let name = write_report(session, "freeze", &body);
                write_line(&format!("FROZEN: no frame for {:.1} s. Report: logs/{name}", since as f32 / 1000.0));
                frozen = Some((name, frames, session.beat.load(Ordering::Relaxed)));
            }
            Some((name, at, last)) if frames > *at => {
                let stuck = session.beat.load(Ordering::Relaxed).saturating_sub(*last) as f32 / 1000.0;
                let line = format!("Came back after being stuck for about {stuck:.1} s.");
                if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(session.folder.join(name)) {
                    let _ = writeln!(file, "\n{line}");
                }
                write_line(&line);
                frozen = None;
            }
            _ => {}
        }
    }
}

// ── In the game ─────────────────────────────────────────────────────────────

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastSessionNotice>()
            .add_systems(First, heartbeat)
            .add_systems(Update, (note_inputs, note_screen))
            .add_systems(EguiPrimaryContextPass, show_last_session_notice);
        // For checking the reports themselves (a developer tool).
        #[cfg(feature = "dev")]
        app.add_systems(Update, test_fault);
    }
}

/// `INCUS_TEST_FAULT=crash` crashes the game 5 seconds in, and `=freeze` stops it for 8 seconds
/// (then it carries on), to check the crash and freeze reports. With `INCUS_SELFTEST_SECONDS`, it
/// quits by itself afterwards.
#[cfg(feature = "dev")]
fn test_fault(time: Res<Time<Real>>, mut done: Local<bool>) {
    if *done || time.elapsed_secs() < 5.0 {
        return;
    }
    *done = true;
    match std::env::var("INCUS_TEST_FAULT").as_deref() {
        Ok("crash") => panic!("a test crash (INCUS_TEST_FAULT=crash)"),
        Ok("freeze") => {
            info!("A test freeze (INCUS_TEST_FAULT=freeze): stopping for 8 seconds");
            std::thread::sleep(Duration::from_secs(8));
        }
        _ => {}
    }
}

/// Each frame: the watchdog's heartbeat.
fn heartbeat() {
    let Some(session) = SESSION.get() else { return };
    session.beat.store(session.clock.elapsed().as_millis() as u64, Ordering::Relaxed);
    session.frames.fetch_add(1, Ordering::Relaxed);
}

/// Key presses (with Ctrl, Shift, Alt) and mouse clicks, for the reports.
fn note_inputs(mut keys: MessageReader<KeyboardInput>, mut clicks: MessageReader<MouseButtonInput>, held: Res<ButtonInput<KeyCode>>) {
    for key in keys.read().filter(|key| key.state == ButtonState::Pressed && !key.repeat) {
        let mut with = String::new();
        for (codes, name) in [([KeyCode::ControlLeft, KeyCode::ControlRight], "Ctrl+"), ([KeyCode::ShiftLeft, KeyCode::ShiftRight], "Shift+"), ([KeyCode::AltLeft, KeyCode::AltRight], "Alt+")] {
            if held.any_pressed(codes) && !codes.contains(&key.key_code) {
                with.push_str(name);
            }
        }
        input(format!("key {with}{:?}", key.key_code));
    }
    for click in clicks.read().filter(|click| click.state == ButtonState::Pressed) {
        input(format!("click {:?}", click.button));
    }
}

/// Which screen is showing.
fn note_screen(screen: Option<Res<State<crate::screen::Screen>>>) {
    if let Some(screen) = screen {
        place("screen", &format!("{:?}", screen.get()));
    }
}

/// The on-screen notice that the last session didn't end normally, for its first seconds.
#[derive(Resource, Default)]
struct LastSessionNotice {
    shown_for: f32,
}

const NOTICE_SECONDS: f32 = 12.0;

fn show_last_session_notice(mut contexts: EguiContexts, mut notice: ResMut<LastSessionNotice>, time: Res<Time<Real>>) {
    let Some(last) = SESSION.get().and_then(|session| session.last_session.as_ref()) else { return };
    if notice.shown_for > NOTICE_SECONDS {
        return;
    }
    notice.shown_for += time.delta_secs();
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::Area::new(egui::Id::new("last session notice")).anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 40.0)).order(egui::Order::Foreground).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.label(last);
            ui.small("(More on the debug panel's Overview page, F3.)");
        });
    });
}

/// The Overview page's part: the build, this session's log, how the last session ended.
pub fn overview(ui: &mut egui::Ui) {
    ui.strong("Diagnostics");
    ui.small(format!("Build {BUILD}"));
    let Some(session) = SESSION.get() else {
        ui.small("Not started.");
        return;
    };
    ui.small(match &session.last_session {
        Some(last) => last.clone(),
        None => "The last session ended normally.".to_string(),
    });
    if let Ok(log) = session.log.try_lock() {
        ui.small(format!("This session's log: logs/{}", log.path.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned())));
    }
    ui.small("Crashes and freezes (no frame for 5 s) write a report in logs/ with what was going on.");
    if ui.small_button("Open the logs folder").clicked() {
        let _ = std::process::Command::new("explorer").arg(&session.folder).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_since_a_time_are_found_newest_first() {
        let folder = std::env::temp_dir().join(format!("incus-diagnostics-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        for name in ["crash-100.txt", "freeze-250.txt", "crash-300.txt", "session-90.log"] {
            std::fs::write(folder.join(name), "").unwrap();
        }
        assert_eq!(newest_report_since(&folder, 200).as_deref(), Some("crash-300.txt"));
        assert_eq!(newest_report_since(&folder, 400), None);
        // Tidying keeps the newest.
        tidy(&folder, "crash-", 1);
        assert!(!folder.join("crash-100.txt").exists() && folder.join("crash-300.txt").exists());
        let _ = std::fs::remove_dir_all(&folder);
    }
}

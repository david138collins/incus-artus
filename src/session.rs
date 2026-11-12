//! The session: which build this is, where the project's folder is, and the session's log.
//!
//! Every session writes its own log to `logs/session-<seconds since 1970>.log` (and the same lines to the console):
//! the game's own messages, and its libraries' warnings. A crash is written there too, with the file and line it
//! happened at, so the first look after any crash or freeze is at `logs/` [old's lesson, "diagnostics first"].

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// The commit this build was made from, with "+ changes" and their fingerprint if the code differed (`build.rs`).
pub const BUILD: &str = env!("INCUS_BUILD");
/// The code files that differed from the commit, by name ("" when none did).
pub const BUILD_CHANGED: &str = env!("INCUS_BUILD_CHANGED");
/// The build's number, from the `BUILD` file ("" without one).
pub const BUILD_NUMBER: &str = env!("INCUS_BUILD_NUMBER");

/// The build in a line: "build 1 (abc1234)", or "(abc1234)" without a number.
pub fn build_line() -> String {
    if BUILD_NUMBER.is_empty() { format!("({BUILD})") } else { format!("build {BUILD_NUMBER} ({BUILD})") }
}

/// The project's folder: where `cargo run` was started from, so the logs land beside the code. Run any other way, the
/// game's own folder [old's lesson, 2026-10-26: run as a bare .exe, old found none of its files; launch through cargo].
pub fn project_folder() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_else(|| {
        std::env::current_exe().ok().and_then(|exe| exe.parent().map(PathBuf::from)).unwrap_or_else(|| PathBuf::from("."))
    })
}

/// The logs' folder (`logs/` in the project's folder), made if it isn't there.
pub fn logs_folder() -> PathBuf {
    let folder = project_folder().join("logs");
    let _ = std::fs::create_dir_all(&folder);
    folder
}

/// Seconds since 1970: names the session's files, so they sort in the order they were made.
pub fn unix_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_secs()).unwrap_or(0)
}

/// The session's log: its file, and when the session started (each line says how long since).
struct Log {
    file: Mutex<Option<std::fs::File>>,
    started: Instant,
}

impl log::Log for Log {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // The game's own messages from "info" up; its libraries' only from "warn" up (wgpu says a lot otherwise).
        if metadata.target().starts_with("incus_artus") { metadata.level() <= log::Level::Info } else { metadata.level() <= log::Level::Warn }
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!("{:>9.3} s  {:<5} {}: {}\n", self.started.elapsed().as_secs_f64(), record.level(), record.target(), record.args());
        eprint!("{line}");
        if let Ok(mut file) = self.file.lock()
            && let Some(file) = file.as_mut()
        {
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock()
            && let Some(file) = file.as_mut()
        {
            let _ = file.flush();
        }
    }
}

/// Starts the session's log, and writes a crash to it before the game ends.
pub fn start_log() {
    let path = logs_folder().join(format!("session-{}.log", unix_seconds()));
    let file = std::fs::File::create(&path).ok();
    let log = Box::leak(Box::new(Log { file: Mutex::new(file), started: Instant::now() }));
    if log::set_logger(log).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
    let crashed = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let place = info.location().map(|place| format!("{}:{}", place.file(), place.line())).unwrap_or_default();
        let message = info.payload().downcast_ref::<&str>().map(|text| text.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_default();
        log::error!("Crashed at {place}: {message}");
        log::logger().flush();
        crashed(info);
    }));
    log::info!("Incus Artus, {}", build_line());
    if !BUILD_CHANGED.is_empty() {
        log::info!("Changed from the commit: {BUILD_CHANGED}");
    }
    log::info!("The session's log: {}", path.display());
}

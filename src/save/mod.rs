//! Save — worlds as folders, Minecraft style. There are no separate "save games": a world folder
//! *is* the world, and saving writes it as it is now.
//!
//! ```text
//! worlds/<world name>/
//!   world.toml      format version, name, seed, the build it was made in, when created and saved
//!   clock.toml      ...one file (or folder) per part of the game...
//!   chunks/         changed chunks only
//! ```
//!
//! Each module owns its part of the world, and registers it with one line in its plugin
//! (just like settings groups):
//!
//! ```ignore
//! app.add_world_state::<WorldClock>("clock");              // a resource saved as it is
//! app.add_world_state_with("fate", save, load, fresh);      // its own save / load / fresh start
//! ```
//!
//! Parts are saved and loaded in the order they were registered (the order of the plugins in
//! `main.rs`), so the terrain is in place before the player is put back on it.
//! A part missing from an older world starts fresh for that world's seed, and the Save page says so.
//!
//! Saving, loading and starting a new world are **requests** (`WorldFileRequest`), carried out once
//! per frame. They work while time is paused. What happened is announced (`WorldFileDone`).

use crate::diagnostics::BUILD_NUMBER;
use crate::data::BlockRegistry;
use crate::debug::{DebugAppExt, project_folder};
use crate::events::{EventLogAppExt, LoggedEvent};
use crate::time::TimeControl;
use crate::ecs::prelude::*;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The world folder format. Raise it when old worlds would be read wrongly, and teach `load` to
/// upgrade them.
pub const FORMAT_VERSION: u32 = 1;

pub type SaveFn = fn(&World, &WorldFolder) -> Result<String, String>;
pub type LoadFn = fn(&mut World, &WorldFolder) -> Result<String, String>;
pub type FreshFn = fn(&mut World, u64);

struct WorldPart {
    name: &'static str,
    save: SaveFn,
    load: LoadFn,
    fresh: FreshFn,
}

/// Every registered part of the world.
#[derive(Resource, Default)]
pub struct WorldParts {
    parts: Vec<WorldPart>,
}

/// Adds `app.add_world_state(...)` to Bevy's `App`.
pub trait WorldStateAppExt {
    /// A resource saved as `<name>.toml` as it is. A fresh world gets `T::default()`.
    fn add_world_state<T: Resource + Serialize + DeserializeOwned + Default>(&mut self, name: &'static str)
    -> &mut Self;
    /// A part with its own save, load and fresh start (given the new world's seed).
    fn add_world_state_with(&mut self, name: &'static str, save: SaveFn, load: LoadFn, fresh: FreshFn) -> &mut Self;
}

impl WorldStateAppExt for App {
    fn add_world_state<T: Resource + Serialize + DeserializeOwned + Default>(
        &mut self,
        name: &'static str,
    ) -> &mut Self {
        self.add_world_state_with(name, save_resource::<T>, load_resource::<T>, fresh_resource::<T>)
    }

    fn add_world_state_with(&mut self, name: &'static str, save: SaveFn, load: LoadFn, fresh: FreshFn) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<WorldParts>()
            .parts
            .push(WorldPart { name, save, load, fresh });
        self
    }
}

/// A world folder being saved or loaded, handed to each part.
pub struct WorldFolder {
    pub path: PathBuf,
    pub seed: u64,
    /// Which part is being handled (its file is `<part>.toml`).
    part: &'static str,
}

impl WorldFolder {
    /// Write this part's `<part>.toml`.
    pub fn write_toml<T: Serialize>(&self, value: &T) -> Result<(), String> {
        let text = toml::to_string_pretty(value).map_err(|error| error.to_string())?;
        write_file(&self.path.join(format!("{}.toml", self.part)), text.as_bytes())
    }

    /// Read this part's `<part>.toml`, or `None` if this world doesn't have one.
    pub fn read_toml<T: DeserializeOwned>(&self) -> Result<Option<T>, String> {
        let path = self.path.join(format!("{}.toml", self.part));
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        toml::from_str(&text).map(Some).map_err(|error| error.to_string())
    }

    /// A sub-folder for parts with many files (like chunks), created if needed.
    pub fn sub_folder(&self, name: &str) -> Result<PathBuf, String> {
        let path = self.path.join(name);
        std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
        Ok(path)
    }
}

/// Write a file safely: write a temporary copy first, then swap it in, so a crash halfway
/// never leaves a half-written file behind.
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    std::fs::rename(&temporary, path).map_err(|error| format!("{}: {error}", path.display()))
}

fn save_resource<T: Resource + Serialize>(world: &World, folder: &WorldFolder) -> Result<String, String> {
    folder.write_toml(world.get_resource::<T>().ok_or("not set up")?)?;
    Ok(String::new())
}

fn load_resource<T: Resource + DeserializeOwned + Default>(
    world: &mut World,
    folder: &WorldFolder,
) -> Result<String, String> {
    match folder.read_toml::<T>()? {
        Some(value) => {
            world.insert_resource(value);
            Ok(String::new())
        }
        None => {
            world.insert_resource(T::default());
            Ok("not in this world, started fresh".to_string())
        }
    }
}

fn fresh_resource<T: Resource + Default>(world: &mut World, _seed: u64) {
    world.insert_resource(T::default());
}

/// Serde helper: big numbers (seeds, dice) as hex text, because TOML numbers stop at 2^63.
pub mod hex_u64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{value:016x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        let text = String::deserialize(deserializer)?;
        u64::from_str_radix(text.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
    }
}

// ── The current world ───────────────────────────────────────────────────────

/// Which world is being played.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CurrentWorld {
    pub name: String,
    #[serde(with = "hex_u64")]
    pub seed: u64,
    /// Seconds since 1970.
    pub created: u64,
    /// The build it was made in (`None`: made before build numbers). Untouched land is made again by
    /// whichever build plays it, so in another build its land may have moved (`docs/worldgen.md`,
    /// recipe versions): loading it says so.
    #[serde(default)]
    pub build: Option<u32>,
    /// When it was last saved or loaded (`None` = never saved).
    #[serde(skip)]
    pub saved: Option<u64>,
}

impl Default for CurrentWorld {
    fn default() -> Self {
        let now = unix_seconds();
        Self {
            name: "New World".to_string(),
            seed: crate::fate::seed_from_clock(),
            created: now,
            build: Some(BUILD_NUMBER),
            saved: None,
        }
    }
}

/// The warning for loading a world made in another build, if it was.
fn other_build(name: &str, made_in: Option<u32>) -> Option<String> {
    let made = match made_in {
        Some(build) if build == BUILD_NUMBER => return None,
        Some(build) => format!("was made in build {build}"),
        None => "was made before build numbers".to_string(),
    };
    Some(format!("\"{name}\" {made}; this is build {BUILD_NUMBER}. The land the recipe makes may have moved since. Start a new world to be sure."))
}

/// What `world.toml` holds.
#[derive(Serialize, Deserialize)]
struct WorldInfo {
    format: u32,
    game_version: String,
    #[serde(flatten)]
    world: CurrentWorld,
    saved_at: u64,
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

/// The folder holding every world.
pub fn worlds_folder() -> PathBuf {
    project_folder().join("worlds")
}

/// A world name as a safe folder name: letters, digits, spaces, `-` and `_` only.
pub fn folder_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() { "Unnamed".to_string() } else { trimmed.to_string() }
}

/// Every world folder, newest first.
pub fn list_worlds() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(worlds_folder()) else {
        return Vec::new();
    };
    let mut worlds: Vec<(std::time::SystemTime, String)> = entries
        .flatten()
        .filter(|entry| entry.path().join("world.toml").exists())
        .map(|entry| {
            let modified = entry
                .path()
                .join("world.toml")
                .metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (modified, entry.file_name().to_string_lossy().to_string())
        })
        .collect();
    worlds.sort_by(|a, b| b.0.cmp(&a.0));
    worlds.into_iter().map(|(_, name)| name).collect()
}

// ── Requests and announcements ──────────────────────────────────────────────

/// Please save, load, or start a world.
#[derive(Message, Debug, Clone)]
pub enum WorldFileRequest {
    /// Save the current world into its folder.
    Save,
    /// Load the world in this folder (replacing what's being played, unsaved changes are lost).
    Load { folder: String },
    /// Start a brand-new world (`None` = pick a random seed).
    New { name: String, seed: Option<u64> },
}

/// What happened to a `WorldFileRequest`.
#[derive(Message, Debug, Clone)]
pub struct WorldFileDone {
    /// "saved", "loaded", "started"; or "couldn't save" etc.
    pub what: &'static str,
    pub world: String,
    /// Notes from each part ("chunks: 3 saved"), or the reason it failed.
    pub details: String,
    pub millis: f64,
    pub tick: u64,
}

impl LoggedEvent for WorldFileDone {
    const KIND: &'static str = "world file";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        format!("{} \"{}\" in {:.0} ms. {}", self.what, self.world, self.millis, self.details)
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldParts>()
            .init_resource::<CurrentWorld>()
            .init_resource::<SavePageState>()
            .add_message::<WorldFileRequest>()
            .add_message::<WorldFileDone>()
            .log_event::<WorldFileDone>()
            // Startup: every part starts fresh for the starting world's seed (this makes the terrain).
            .add_systems(Startup, start_first_world)
            .add_systems(Update, handle_world_file_requests)
            .add_debug_page("Save", save_debug_page);
    }
}

fn start_first_world(world: &mut World) {
    let seed = world.resource::<CurrentWorld>().seed;
    fresh_world(world, seed);
}

/// Every part starts fresh for this seed.
fn fresh_world(world: &mut World, seed: u64) {
    world.resource_scope(|world, parts: Mut<WorldParts>| {
        for part in &parts.parts {
            (part.fresh)(world, seed);
        }
    });
}

/// Carries out waiting requests. An exclusive system (it takes the whole World), because loading
/// replaces nearly everything.
fn handle_world_file_requests(world: &mut World) {
    let requests: Vec<WorldFileRequest> = world.resource_mut::<Messages<WorldFileRequest>>().drain().collect();
    for request in requests {
        let started = Instant::now();
        let (what, result) = match request {
            WorldFileRequest::Save => ("saved", save_world(world)),
            WorldFileRequest::Load { folder } => ("loaded", load_world(world, &folder)),
            WorldFileRequest::New { name, seed } => ("started", new_world(world, name, seed)),
        };
        let (what, details) = match result {
            Ok(details) => (what, details),
            Err(reason) => (
                match what {
                    "saved" => "couldn't save",
                    "loaded" => "couldn't load",
                    _ => "couldn't start",
                },
                reason,
            ),
        };
        let done = WorldFileDone {
            what,
            world: world.resource::<CurrentWorld>().name.clone(),
            details,
            millis: started.elapsed().as_secs_f64() * 1000.0,
            tick: world.resource::<TimeControl>().tick(),
        };
        info!("World file: {} \"{}\": {}", done.what, done.world, done.details);
        world.resource_mut::<SavePageState>().last_result = Some(format!(
            "{} \"{}\" ({:.0} ms)\n{}",
            done.what, done.world, done.millis, done.details
        ));
        world.write_message(done);
    }
}

fn save_world(world: &mut World) -> Result<String, String> {
    save_world_in(world, &worlds_folder())
}

fn save_world_in(world: &mut World, worlds: &Path) -> Result<String, String> {
    let current = world.resource::<CurrentWorld>().clone();
    let path = worlds.join(folder_name(&current.name));
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;

    let info = WorldInfo {
        format: FORMAT_VERSION,
        game_version: env!("CARGO_PKG_VERSION").to_string(),
        world: current.clone(),
        saved_at: unix_seconds(),
    };
    let text = toml::to_string_pretty(&info).map_err(|error| error.to_string())?;
    write_file(&path.join("world.toml"), text.as_bytes())?;

    let mut notes = Vec::new();
    let mut problems = Vec::new();
    world.resource_scope(|world, parts: Mut<WorldParts>| {
        for part in &parts.parts {
            let folder = WorldFolder { path: path.clone(), seed: current.seed, part: part.name };
            match (part.save)(world, &folder) {
                Ok(note) if note.is_empty() => notes.push(part.name.to_string()),
                Ok(note) => notes.push(format!("{} ({note})", part.name)),
                Err(error) => problems.push(format!("{}: {error}", part.name)),
            }
        }
    });
    world.resource_mut::<CurrentWorld>().saved = Some(info.saved_at);
    if problems.is_empty() {
        Ok(format!("Saved: {}", notes.join(", ")))
    } else {
        Err(format!("Saved: {}\nProblems: {}", notes.join(", "), problems.join("; ")))
    }
}

fn load_world(world: &mut World, folder_name: &str) -> Result<String, String> {
    load_world_from(world, &worlds_folder().join(folder_name))
}

fn load_world_from(world: &mut World, path: &Path) -> Result<String, String> {
    let path = path.to_path_buf();
    let text = std::fs::read_to_string(path.join("world.toml")).map_err(|error| format!("world.toml: {error}"))?;
    let info: WorldInfo = toml::from_str(&text).map_err(|error| format!("world.toml: {error}"))?;
    if info.format > FORMAT_VERSION {
        return Err(format!(
            "this world was saved by a newer version of the game (format {}, this game reads up to {FORMAT_VERSION})",
            info.format
        ));
    }

    let seed = info.world.seed;
    let warning = other_build(&info.world.name, info.world.build);
    let mut loaded = info.world;
    loaded.saved = Some(info.saved_at);
    world.insert_resource(loaded);

    let mut notes = Vec::new();
    let mut problems = Vec::new();
    world.resource_scope(|world, parts: Mut<WorldParts>| {
        for part in &parts.parts {
            let folder = WorldFolder { path: path.clone(), seed, part: part.name };
            match (part.load)(world, &folder) {
                Ok(note) if note.is_empty() => notes.push(part.name.to_string()),
                Ok(note) => notes.push(format!("{} ({note})", part.name)),
                Err(error) => {
                    // A part that can't be read starts fresh, so the rest of the world still loads.
                    (part.fresh)(world, seed);
                    problems.push(format!("{}: {error} (started fresh)", part.name));
                }
            }
        }
    });
    let mut report = format!("Loaded: {}", notes.join(", "));
    if !problems.is_empty() {
        report.push_str(&format!("\nProblems: {}", problems.join("; ")));
    }
    if let Some(warning) = warning {
        warn!("{warning}");
        report = format!("{warning}\n{report}");
    }
    Ok(report)
}

fn new_world(world: &mut World, name: String, seed: Option<u64>) -> Result<String, String> {
    let seed = seed.unwrap_or_else(crate::fate::seed_from_clock);
    world.insert_resource(CurrentWorld {
        name: if name.trim().is_empty() { "New World".to_string() } else { name },
        seed,
        created: unix_seconds(),
        build: Some(BUILD_NUMBER),
        saved: None,
    });
    fresh_world(world, seed);
    Ok(format!("Seed {seed:016x}. Not saved yet."))
}

// ── Debug page ──────────────────────────────────────────────────────────────

#[derive(Resource)]
struct SavePageState {
    new_name: String,
    seed_text: String,
    last_result: Option<String>,
    /// The world folders, refreshed when the page asks.
    folders: Vec<String>,
    folders_listed: bool,
}

impl Default for SavePageState {
    fn default() -> Self {
        Self {
            new_name: "New World".to_string(),
            seed_text: String::new(),
            last_result: None,
            folders: Vec::new(),
            folders_listed: false,
        }
    }
}

fn save_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let mut requests = Vec::new();

    // ── This world ──
    {
        let mut current = world.resource_mut::<CurrentWorld>();
        ui.strong("This world");
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut current.name);
        });
        ui.label(format!("Folder: worlds/{}", folder_name(&current.name)));
        ui.label(format!("Seed: {:016x}", current.seed));
        match current.saved {
            Some(_) => ui.label("Saved or loaded this session."),
            None => ui.label("Not saved yet."),
        };
        if ui.button("Save").clicked() {
            requests.push(WorldFileRequest::Save);
        }
    }

    // ── Load ──
    ui.separator();
    ui.strong("Load a world");
    ui.small("Loading replaces the world you're in. Anything not saved is lost.");
    {
        let mut state = world.resource_mut::<SavePageState>();
        if !state.folders_listed {
            state.folders = list_worlds();
            state.folders_listed = true;
        }
        ui.horizontal(|ui| {
            if ui.button("Refresh list").clicked() {
                state.folders = list_worlds();
            }
            if ui.button("Open worlds folder").clicked() {
                let folder = worlds_folder();
                let _ = std::fs::create_dir_all(&folder);
                let _ = std::process::Command::new("explorer").arg(&folder).spawn();
            }
        });
        if state.folders.is_empty() {
            ui.label("No saved worlds yet.");
        }
        for folder in &state.folders {
            ui.horizontal(|ui| {
                if ui.button("Load").clicked() {
                    requests.push(WorldFileRequest::Load { folder: folder.clone() });
                }
                ui.label(folder);
            });
        }
    }

    // ── New ──
    ui.separator();
    ui.strong("New world");
    {
        let mut state = world.resource_mut::<SavePageState>();
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut state.new_name);
        });
        ui.horizontal(|ui| {
            ui.label("Seed (hex, blank = random)");
            ui.add(egui::TextEdit::singleline(&mut state.seed_text).desired_width(140.0));
        });
        let seed = u64::from_str_radix(state.seed_text.trim(), 16).ok();
        if !state.seed_text.trim().is_empty() && seed.is_none() {
            ui.colored_label(egui::Color32::from_rgb(255, 80, 200), "That isn't a hex number (0-9, a-f).");
        }
        if ui.button("Start new world").clicked() {
            requests.push(WorldFileRequest::New { name: state.new_name.clone(), seed });
        }
    }

    if let Some(result) = &world.resource::<SavePageState>().last_result {
        ui.separator();
        ui.label(result);
    }

    let refresh = requests.iter().any(|request| matches!(request, WorldFileRequest::Save));
    for request in requests {
        world.write_message(request);
    }
    if refresh {
        // The list is refreshed next frame, after the save has happened.
        world.resource_mut::<SavePageState>().folders_listed = false;
    }

    ui.separator();
    let parts: Vec<&str> = world.resource::<WorldParts>().parts.iter().map(|part| part.name).collect();
    ui.small(format!("Parts saved: {}", parts.join(", ")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_names_are_safe() {
        assert_eq!(folder_name("My World"), "My World");
        assert_eq!(folder_name("a/b\\c:d"), "a_b_c_d");
        assert_eq!(folder_name("   "), "Unnamed");
    }

    /// Save a world with a clock, Fate and changed terrain; scramble everything; load it back.
    #[test]
    fn a_whole_world_survives_saving_and_loading() {
        use crate::fate::{Fate, Twist, Wanted};
        use crate::time::WorldClock;
        use crate::world::{BlockId, VoxelWorld};

        let mut app = App::new();
        app.init_resource::<WorldParts>()
            .init_resource::<CurrentWorld>()
            .init_resource::<BlockRegistry>()
            .init_resource::<VoxelWorld>()
            .init_resource::<crate::world::TestWorldSettings>()
            .init_resource::<WorldClock>()
            .init_resource::<Fate>()
            .add_world_state::<WorldClock>("clock")
            .add_world_state_with(
                "terrain",
                crate::world::save_terrain,
                crate::world::load_terrain,
                crate::world::fresh_terrain,
            )
            .add_world_state_with("fate", crate::fate::save_fate, crate::fate::load_fate, crate::fate::fresh_fate);
        let world = app.world_mut();
        world.resource_mut::<CurrentWorld>().name = "Round Trip".to_string();
        fresh_world(world, 99);

        // Change things.
        world.resource_mut::<WorldClock>().seconds = 12345.0;
        world.resource_mut::<Fate>().add_twist(Twist { label: "kept".to_string(), luck: 2.0, ..Default::default() });
        let hole = IVec3::new(5, 5, 144);
        world.resource_mut::<VoxelWorld>().set_block(hole, BlockId::AIR);
        let next_roll = world.resource::<Fate>().clone().roll("x", 0.5, Wanted::Either);

        let folder = std::env::temp_dir().join(format!("incus-artus-test-{}", std::process::id()));
        save_world_in(world, &folder).expect("saves");

        // Scramble, then load.
        fresh_world(world, 1);
        world.resource_mut::<WorldClock>().seconds = 0.0;
        assert_ne!(world.resource::<VoxelWorld>().get_block(hole), BlockId::AIR);
        let report = load_world_from(world, &folder.join("Round Trip")).expect("loads");
        let _ = std::fs::remove_dir_all(&folder);

        assert!(report.contains("1 changed chunks"), "{report}");
        assert_eq!(world.resource::<WorldClock>().seconds, 12345.0);
        assert_eq!(world.resource::<VoxelWorld>().get_block(hole), BlockId::AIR);
        assert_eq!(world.resource::<Fate>().twists[0].label, "kept");
        assert_eq!(world.resource_mut::<Fate>().roll("x", 0.5, Wanted::Either), next_roll);
        assert_eq!(world.resource::<CurrentWorld>().name, "Round Trip");
    }

    #[test]
    fn world_info_round_trips() {
        let info = WorldInfo {
            format: FORMAT_VERSION,
            game_version: "0.1.0".to_string(),
            world: CurrentWorld { name: "Test".to_string(), seed: u64::MAX - 5, created: 12, build: Some(BUILD_NUMBER + 1), saved: None },
            saved_at: 34,
        };
        let text = toml::to_string_pretty(&info).unwrap();
        assert!(text.contains(&format!("build = {}", BUILD_NUMBER + 1)), "{text}");
        let back: WorldInfo = toml::from_str(&text).unwrap();
        assert_eq!(back.world.seed, u64::MAX - 5);
        assert_eq!(back.world.name, "Test");
        assert_eq!(back.world.build, Some(BUILD_NUMBER + 1));
        // A world from before build numbers reads as one.
        let old: WorldInfo = toml::from_str("format = 1\ngame_version = \"0.1.0\"\nname = \"nice25\"\nseed = \"89e944ac43497f87\"\ncreated = 1\nsaved_at = 2\n").unwrap();
        assert_eq!(old.world.build, None);
    }

    #[test]
    fn a_world_from_another_build_is_warned_about_and_this_builds_isnt() {
        assert_eq!(other_build("Here", Some(BUILD_NUMBER)), None);
        let older = other_build("Old", Some(BUILD_NUMBER + 1)).unwrap();
        assert!(older.contains(&format!("made in build {}", BUILD_NUMBER + 1)) && older.contains(&format!("this is build {BUILD_NUMBER}")), "{older}");
        assert!(other_build("nice25", None).unwrap().contains("made before build numbers"));
        assert_eq!(CurrentWorld::default().build, Some(BUILD_NUMBER));
    }
}

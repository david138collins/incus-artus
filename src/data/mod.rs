//! Data — registries of game content (blocks now; items and creatures later).
//!
//! The `BlockRegistry` is the list of block definitions. Each block has a text ID
//! (`incus:stone`, used by data files and saves) and a number (used by the running game).
//!
//! Blocks come from David's data files: `assets/data/<pack>/blocks/<name>.toml` (see
//! `loading.rs` and docs/data-files.md). Only air is built in.
//!
//! **Live reload**: about once a second the game checks the data files and block textures.
//! When one changes, it's reloaded and the world redraws. It's a separate system
//! (`watch_data_files`) that can simply be removed for a shipped game.

mod loading;
pub mod master_palette;

use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::world::BlockId;
use bevy::prelude::*;
use bevy_egui::egui;
use loading::{find_block_files, parse_block};
pub use loading::{block_file_text, header_of};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::SystemTime;

/// Where the game's `assets` folder is. During development (`cargo run`) that's the project
/// folder; in a shipped game it sits next to the program.
pub fn assets_folder() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(dir).join("assets");
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("assets")))
        .unwrap_or_else(|| PathBuf::from("assets"))
}

/// `assets/data`: the data files.
pub fn data_folder() -> PathBuf {
    assets_folder().join("data")
}

/// `assets/textures/blocks`: the block face images.
pub fn block_texture_folder() -> PathBuf {
    assets_folder().join("textures").join("blocks")
}

/// Blocks the test world is built from. If a file is missing, a placeholder stands in.
const TEST_WORLD_BLOCKS: [&str; 3] = ["incus:gabbro", "incus:stone", "incus:grass"];

/// Face order used everywhere: north (+X), south (−X), east (+Y), west (−Y), top (+Z), bottom (−Z).
pub const FACE_NAMES: [&str; 6] = ["north", "south", "east", "west", "top", "bottom"];

/// How a face's texture is turned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextureRotation {
    /// Always drawn the same way up (needed for anything with a direction, like grass sides).
    #[default]
    Fixed,
    /// Each block turned 0°, 90°, 180° or 270°, picked from its position (so it never
    /// changes or flickers). Hides repetition on directionless textures such as grass tops.
    Random,
}

/// Everything the game knows about one block type.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDef {
    /// Text ID, e.g. "incus:granite".
    pub name: String,
    /// The name players see, e.g. "Granite".
    pub display_name: String,
    /// Opaque blocks hide whatever is behind them. Air is not opaque.
    pub opaque: bool,
    /// See-through blocks (grid, outline) are drawn on both sides with their empty pixels
    /// cut out, and don't hide what's behind them.
    pub see_through: bool,
    /// For see-through blocks: draw the faces between neighbours of this kind too?
    /// On (grid): a full lattice, every cell visible. Off (outline): only the outside skin of
    /// the volume, so it reads as one outlined box.
    pub inner_faces: bool,
    /// Solid blocks stop bodies (collision). The grid is not solid.
    pub solid: bool,
    /// Can the crosshair stop on it (to break it or build against it)?
    pub targetable: bool,
    /// Can placing a block simply replace it (like placing into air)?
    pub replaceable: bool,
    /// Texture name for each face (see `FACE_NAMES` for the order). Empty = no texture.
    pub faces: [String; 6],
    /// Texture rotation for each face (same order).
    pub rotations: [TextureRotation; 6],
    /// How grippy the top is to walk on: 1 = ordinary ground, lower = slipperier (ice),
    /// higher = grippier. 0 = no grip at all (you'd slide forever).
    pub friction: f32,
    /// Tags the Registrar groups and sorts blocks by (a block can have any number, like being in
    /// several folders at once). A `/` nests one tag in another, as folders do: "rock/igneous".
    pub tags: Vec<String>,
    /// A building aid (grid, outline): its colours stay out of the master palette, as the UI's do.
    pub tool: bool,
}

impl BlockDef {
    /// An ordinary solid block with the same texture on all six faces ("" = the magenta
    /// checkerboard). Data files start from this.
    pub fn uniform(name: &str, texture: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: name.to_string(),
            opaque: true,
            see_through: false,
            inner_faces: false,
            solid: true,
            targetable: true,
            replaceable: false,
            faces: std::array::from_fn(|_| texture.to_string()),
            rotations: Default::default(),
            friction: 1.0,
            tags: Vec::new(),
            tool: false,
        }
    }

    /// An invisible, non-solid block (air).
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: "Air".to_string(),
            opaque: false,
            see_through: false,
            inner_faces: false,
            solid: false,
            targetable: false,
            replaceable: true,
            faces: Default::default(),
            rotations: Default::default(),
            friction: 1.0,
            tags: Vec::new(),
            tool: false,
        }
    }
}

#[derive(Resource)]
pub struct BlockRegistry {
    defs: Vec<BlockDef>,
    ids: HashMap<String, BlockId>,
    /// Each data file read so far, and when it was last changed (to spot edits).
    files: HashMap<PathBuf, Option<SystemTime>>,
    /// Which file each block came from (for the debug page).
    sources: HashMap<String, PathBuf>,
    /// What's wrong, per file (or per block for placeholders). Shown on the Blocks page.
    problems: BTreeMap<String, String>,
    /// Goes up by one every time definitions or textures change, so the renderer knows
    /// to rebuild.
    revision: u64,
}

/// The game's blocks, loaded from the data folder.
impl Default for BlockRegistry {
    fn default() -> Self {
        Self::load()
    }
}

impl BlockRegistry {
    /// Just air (block number 0, always).
    fn with_air() -> Self {
        let mut registry = Self {
            defs: Vec::new(),
            ids: HashMap::new(),
            files: HashMap::new(),
            sources: HashMap::new(),
            problems: BTreeMap::new(),
            revision: 0,
        };
        registry.register(BlockDef::empty("incus:air"));
        registry
    }

    /// Air plus every block in the data folder.
    pub fn load() -> Self {
        let mut registry = Self::with_air();
        registry.reload_changed_files();
        registry
    }

    /// Read every data file that's new or changed since last time. Returns true if any
    /// block definition changed. Blocks keep their numbers; a changed file replaces its
    /// block's definition (including any Blocks-page tweaks to that block).
    pub fn reload_changed_files(&mut self) -> bool {
        let mut folder_problems = Vec::new();
        self.problems.remove("data folder");
        let found = find_block_files(&data_folder(), &mut folder_problems);
        let mut changed = false;
        let still_there: Vec<PathBuf> = found.iter().map(|file| file.path.clone()).collect();

        for file in found {
            if self.files.get(&file.path) == Some(&file.modified) {
                continue;
            }
            self.files.insert(file.path.clone(), file.modified);
            let key = file.path.display().to_string();
            let result = std::fs::read_to_string(&file.path)
                .map_err(|error| error.to_string())
                .and_then(|text| parse_block(&file.id, &text));
            match result {
                Ok((def, warnings)) => {
                    if warnings.is_empty() {
                        self.problems.remove(&key);
                    } else {
                        self.problems.insert(key, warnings.join("; "));
                    }
                    self.sources.insert(file.id.clone(), file.path.clone());
                    // A placeholder for this block is now replaced by the real thing.
                    self.problems.remove(&file.id);
                    changed |= self.insert_or_replace(def);
                }
                Err(error) => {
                    let kept = if self.ids.contains_key(&file.id) { " (keeping the last good version)" } else { "" };
                    self.problems.insert(key, format!("couldn't be used{kept}: {error}"));
                }
            }
        }

        // Files that have gone: their blocks stay until restart (blocks can't be renumbered).
        let gone: Vec<PathBuf> = self.files.keys().filter(|path| !still_there.contains(path)).cloned().collect();
        for path in gone {
            self.files.remove(&path);
            self.problems.insert(
                path.display().to_string(),
                "file is gone; its block stays until the game restarts".to_string(),
            );
        }
        for problem in folder_problems {
            self.problems.insert("data folder".to_string(), problem);
        }

        // Never crash for want of a block: stand-ins for anything the test world needs.
        for name in TEST_WORLD_BLOCKS {
            if !self.ids.contains_key(name) {
                self.register(BlockDef::uniform(name, ""));
                self.problems.insert(
                    name.to_string(),
                    "no data file found; using a magenta placeholder (the test world needs it)".to_string(),
                );
                changed = true;
            }
        }

        if changed {
            self.revision += 1;
        }
        changed
    }

    /// Add a new block, or replace an existing block's definition. True if anything changed.
    fn insert_or_replace(&mut self, def: BlockDef) -> bool {
        match self.ids.get(&def.name) {
            Some(&id) => {
                let slot = &mut self.defs[id.0 as usize];
                let changed = *slot != def;
                *slot = def;
                changed
            }
            None => {
                self.register(def);
                true
            }
        }
    }

    /// Tell the renderer that block textures changed on disk.
    pub fn textures_changed(&mut self) {
        self.revision += 1;
    }

    /// Goes up whenever definitions or textures change.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Add a block definition and give it the next number (or return its existing number).
    pub fn register(&mut self, def: BlockDef) -> BlockId {
        if let Some(&id) = self.ids.get(&def.name) {
            return id;
        }
        let id = BlockId(self.defs.len() as u16);
        self.ids.insert(def.name.clone(), id);
        self.defs.push(def);
        id
    }

    /// The data file a block came from (none for air and placeholders).
    pub fn source(&self, name: &str) -> Option<&std::path::Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }

    pub fn id(&self, name: &str) -> Option<BlockId> {
        self.ids.get(name).copied()
    }

    /// The definition for a block number, if there is one.
    pub fn def(&self, id: BlockId) -> Option<&BlockDef> {
        self.defs.get(id.0 as usize)
    }

    /// The definition for a block number, to change it (debug tweaks).
    pub fn def_mut(&mut self, id: BlockId) -> Option<&mut BlockDef> {
        self.defs.get_mut(id.0 as usize)
    }

    /// Does it stop bodies? (Unknown numbers count as solid, to be safe.)
    pub fn is_solid(&self, id: BlockId) -> bool {
        self.def(id).is_none_or(|def| def.solid)
    }

    /// Can the crosshair stop on it?
    pub fn is_targetable(&self, id: BlockId) -> bool {
        self.def(id).is_none_or(|def| def.targetable)
    }

    /// Can a placed block simply take its place?
    pub fn is_replaceable(&self, id: BlockId) -> bool {
        self.def(id).is_some_and(|def| def.replaceable)
    }

    pub fn name(&self, id: BlockId) -> &str {
        self.defs
            .get(id.0 as usize)
            .map_or("<unknown>", |def| def.name.as_str())
    }

    /// All definitions, in number order (index = BlockId).
    pub fn defs(&self) -> &[BlockDef] {
        &self.defs
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }
}

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .init_resource::<DataSettings>()
            .init_resource::<DataWatch>()
            .add_debug_page("Blocks", blocks_debug_page)
            .add_settings::<DataSettings>("data")
            .add_settings_with("blocks", save_block_tweaks, load_block_tweaks);
        // Reloading block files as they're edited is a developer tool (see `dev` in Cargo.toml).
        #[cfg(feature = "dev")]
        app.add_systems(Update, watch_data_files);
    }
}

/// Tunables (shown on the "Blocks" debug page).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DataSettings {
    /// Check data files and block textures for changes, and reload them.
    pub live_reload: bool,
    /// How often to check, in seconds.
    pub check_every: f32,
}

impl Default for DataSettings {
    fn default() -> Self {
        Self {
            live_reload: true,
            check_every: 1.0,
        }
    }
}

/// When we last checked, and when each block texture was last changed.
#[derive(Resource, Default)]
struct DataWatch {
    last_check: f32,
    textures: HashMap<PathBuf, Option<SystemTime>>,
    /// Set by the "Reload now" button.
    check_now: bool,
}

/// When each block texture PNG was last changed.
fn texture_times() -> HashMap<PathBuf, Option<SystemTime>> {
    std::fs::read_dir(block_texture_folder())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "png"))
        .map(|entry| (entry.path(), entry.metadata().and_then(|metadata| metadata.modified()).ok()))
        .collect()
}

/// Live reload: every `check_every` seconds (real time, so it works while paused), reload
/// changed data files, and tell the renderer if any texture changed.
fn watch_data_files(
    time: Res<Time<Real>>,
    settings: Res<DataSettings>,
    mut watch: ResMut<DataWatch>,
    mut registry: ResMut<BlockRegistry>,
) {
    let now = time.elapsed_secs();
    let due = settings.live_reload && now - watch.last_check >= settings.check_every;
    if !due && !watch.check_now {
        return;
    }
    watch.last_check = now;
    watch.check_now = false;

    let textures = texture_times();
    // The very first check only remembers the times (everything was just loaded).
    let textures_changed = !watch.textures.is_empty() && textures != watch.textures;
    watch.textures = textures;

    if registry.reload_changed_files() {
        info!("Block data reloaded");
    }
    if textures_changed {
        info!("Block textures changed on disk: rebuilding");
        registry.textures_changed();
    }
}

/// The parts of a block that can be tweaked on the Blocks page (saved in "everything" presets).
#[derive(Serialize, Deserialize)]
struct BlockTweaks {
    friction: f32,
    rotations: [TextureRotation; 6],
    targetable: bool,
}

pub(crate) fn save_block_tweaks(world: &World) -> Result<toml::Value, String> {
    let registry = world.get_resource::<BlockRegistry>().ok_or("not set up yet")?;
    let tweaks: BTreeMap<&str, BlockTweaks> = registry
        .defs
        .iter()
        .filter(|def| def.opaque || def.see_through)
        .map(|def| {
            let tweaks = BlockTweaks {
                friction: def.friction,
                rotations: def.rotations,
                targetable: def.targetable,
            };
            (def.name.as_str(), tweaks)
        })
        .collect();
    toml::Value::try_from(tweaks).map_err(|error| error.to_string())
}

/// Blocks in the preset that no longer exist are skipped; blocks missing from it stay as they are.
pub(crate) fn load_block_tweaks(world: &mut World, value: toml::Value) -> Result<(), String> {
    let tweaks: BTreeMap<String, BlockTweaks> = value.try_into().map_err(|error| error.to_string())?;
    let mut registry = world.resource_mut::<BlockRegistry>();
    for (name, tweak) in tweaks {
        let Some(def) = registry.id(&name).and_then(|id| registry.def_mut(id)) else {
            continue;
        };
        def.friction = tweak.friction;
        def.rotations = tweak.rotations;
        def.targetable = tweak.targetable;
    }
    Ok(())
}

/// Random texture rotation switches for one block. A block that looks the same all round
/// gets one switch for every face; a block with a distinct top (like grass) gets separate
/// switches for the top, the sides and the bottom, since its sides usually have an "up".
/// The world redraws itself when these change.
fn rotation_checkboxes(ui: &mut egui::Ui, def: &mut BlockDef) {
    let same_all_round = def.faces.iter().all(|face| *face == def.faces[0]);
    let mut group = |ui: &mut egui::Ui, faces: &[usize], label: &str| {
        let mut random = faces.iter().all(|&face| def.rotations[face] == TextureRotation::Random);
        if ui.checkbox(&mut random, label).changed() {
            let rotation = if random { TextureRotation::Random } else { TextureRotation::Fixed };
            for &face in faces {
                def.rotations[face] = rotation;
            }
        }
    };
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.small("random rotation:");
        if same_all_round {
            group(ui, &[0, 1, 2, 3, 4, 5], "all faces");
        } else {
            group(ui, &[4], "top");
            group(ui, &[0, 1, 2, 3], "sides");
            group(ui, &[5], "bottom");
        }
    });
}

fn blocks_debug_page(ui: &mut egui::Ui, world: &mut World) {
    // ── Live reload ──
    {
        let mut settings = world.resource_mut::<DataSettings>();
        ui.checkbox(&mut settings.live_reload, "live reload (data files and block textures)");
        ui.add(crate::debug::Slider::new(&mut settings.check_every, 0.2..=10.0).text("check every (s)"));
    }
    ui.horizontal(|ui| {
        if ui.button("Reload now").clicked() {
            world.resource_mut::<DataWatch>().check_now = true;
        }
        if ui.button("Open data folder").clicked() {
            let _ = std::process::Command::new("explorer").arg(data_folder()).spawn();
        }
    });

    let mut registry = world.resource_mut::<BlockRegistry>();
    ui.label(format!("{} block types registered", registry.len()));
    ui.small("Tweaks made here are temporary: saving a block's file replaces them.");

    // ── Problems ──
    if registry.problems.is_empty() {
        ui.label("No problems with the data files.");
    } else {
        for (place, problem) in &registry.problems {
            ui.colored_label(egui::Color32::from_rgb(255, 80, 200), format!("{place}: {problem}"));
        }
    }

    ui.separator();
    let sources = registry.sources.clone();
    for (number, def) in registry.defs.iter_mut().enumerate() {
        ui.monospace(format!("{number:>5}  {}  \"{}\"", def.name, def.display_name));
        if let Some(path) = sources.get(&def.name) {
            let short = path.strip_prefix(data_folder()).unwrap_or(path);
            ui.small(format!("        from data/{}", short.display().to_string().replace('\\', "/")));
        }
        if def.opaque || def.see_through {
            let [north, _, _, _, top, bottom] = &def.faces;
            ui.small(format!("        top: {top}   sides: {north}   bottom: {bottom}"));
            if def.see_through {
                let kind = if def.inner_faces { "every cell visible" } else { "outside skin only" };
                ui.small(format!("        see-through ({kind}): walk through it, build into it"));
            }
            rotation_checkboxes(ui, def);
            if def.solid {
                ui.add(crate::debug::Slider::new(&mut def.friction, 0.0..=3.0).text("friction (1 = normal ground)"));
            }
            ui.horizontal(|ui| {
                ui.add_space(40.0);
                ui.checkbox(&mut def.targetable, "crosshair can target it");
            });
        }
    }
}

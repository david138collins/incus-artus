//! Data — registries of game content (blocks now; items and creatures later).
//!
//! The `BlockRegistry` is the list of block definitions. Each block has a text name
//! (used by saves and data files) and a number (used by the running game).
//!
//! TEMPORARY: until data loading exists (step 7), the four blocks the test world needs
//! are defined right here in code. Step 7 replaces this with David's TOML files.

use crate::debug::DebugAppExt;
use crate::world::BlockId;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;

/// Face order used everywhere: north (+X), south (−X), east (+Y), west (−Y), top (+Z), bottom (−Z).
pub const FACE_NAMES: [&str; 6] = ["north", "south", "east", "west", "top", "bottom"];

/// Everything the game knows about one block type (more fields arrive with data files).
#[derive(Debug, Clone)]
pub struct BlockDef {
    /// Text ID, e.g. "incus:granite".
    pub name: String,
    /// Opaque blocks hide whatever is behind them. Air is not opaque.
    pub opaque: bool,
    /// Texture name for each face (see `FACE_NAMES` for the order). Empty = no texture.
    pub faces: [String; 6],
}

impl BlockDef {
    /// The same texture on all six faces.
    pub fn uniform(name: &str, texture: &str) -> Self {
        Self {
            name: name.to_string(),
            opaque: true,
            faces: std::array::from_fn(|_| texture.to_string()),
        }
    }

    /// Different textures for the top, the four sides, and the bottom.
    pub fn top_side_bottom(name: &str, top: &str, side: &str, bottom: &str) -> Self {
        Self {
            name: name.to_string(),
            opaque: true,
            faces: [side, side, side, side, top, bottom].map(str::to_string),
        }
    }

    /// An invisible, non-solid block (air).
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            opaque: false,
            faces: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct BlockRegistry {
    defs: Vec<BlockDef>,
    ids: HashMap<String, BlockId>,
}

impl Default for BlockRegistry {
    fn default() -> Self {
        let mut registry = Self {
            defs: Vec::new(),
            ids: HashMap::new(),
        };
        // Air must be number 0.
        registry.register(BlockDef::empty("incus:air"));
        // TEMPORARY built-ins for the test world (replaced by data files in step 7).
        registry.register(BlockDef::uniform("incus:gabbro", "gabbro"));
        registry.register(BlockDef::uniform("incus:stone", "stone"));
        registry.register(BlockDef::top_side_bottom(
            "incus:grass",
            "grass_top",
            "grass_side",
            "dirt",
        ));
        registry
    }
}

impl BlockRegistry {
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

    pub fn id(&self, name: &str) -> Option<BlockId> {
        self.ids.get(name).copied()
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
            .add_debug_page("Blocks", blocks_debug_page);
    }
}

fn blocks_debug_page(ui: &mut egui::Ui, world: &mut World) {
    let registry = world.resource::<BlockRegistry>();
    ui.label(format!("{} block types registered", registry.len()));
    ui.small("(Temporary built-ins until data files arrive in step 7.)");
    ui.separator();
    for (number, def) in registry.defs.iter().enumerate() {
        ui.monospace(format!("{number:>5}  {}", def.name));
        if def.opaque {
            let [north, _, _, _, top, bottom] = &def.faces;
            ui.small(format!("        top: {top}   sides: {north}   bottom: {bottom}"));
        }
    }
}

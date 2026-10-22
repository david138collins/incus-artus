//! Data — registries of game content (blocks now; items and creatures later).
//!
//! The `BlockRegistry` is the list of block names ↔ numbers. Saves and data files use
//! the names; the running game uses the numbers.
//!
//! TEMPORARY: until data loading exists (step 7), the four blocks the test world needs
//! are registered right here in code. Step 7 replaces this with David's TOML files.

use crate::debug::DebugAppExt;
use crate::world::BlockId;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;

#[derive(Resource)]
pub struct BlockRegistry {
    names: Vec<String>,
    ids: HashMap<String, BlockId>,
}

impl Default for BlockRegistry {
    fn default() -> Self {
        let mut registry = Self {
            names: Vec::new(),
            ids: HashMap::new(),
        };
        // Air must be number 0.
        registry.register("incus:air");
        // TEMPORARY built-ins for the test world (replaced by data files in step 7).
        registry.register("incus:gabbro");
        registry.register("incus:stone");
        registry.register("incus:grass");
        registry
    }
}

impl BlockRegistry {
    /// Give a block name a number (or return its existing number).
    pub fn register(&mut self, name: &str) -> BlockId {
        if let Some(&id) = self.ids.get(name) {
            return id;
        }
        let id = BlockId(self.names.len() as u16);
        self.names.push(name.to_string());
        self.ids.insert(name.to_string(), id);
        id
    }

    pub fn id(&self, name: &str) -> Option<BlockId> {
        self.ids.get(name).copied()
    }

    pub fn name(&self, id: BlockId) -> &str {
        self.names
            .get(id.0 as usize)
            .map_or("<unknown>", String::as_str)
    }

    pub fn len(&self) -> usize {
        self.names.len()
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
    for (number, name) in registry.names.iter().enumerate() {
        ui.monospace(format!("{number:>5}  {name}"));
    }
}

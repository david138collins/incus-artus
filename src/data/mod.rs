//! Data — registries of game content (blocks now; items and creatures later).
//!
//! The `BlockRegistry` is the list of block definitions. Each block has a text name
//! (used by saves and data files) and a number (used by the running game).
//!
//! TEMPORARY: until data loading exists (step 7), the four blocks the test world needs
//! are defined right here in code. Step 7 replaces this with David's TOML files.

use crate::debug::{DebugAppExt, SettingsAppExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::world::BlockId;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;

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

/// Everything the game knows about one block type (more fields arrive with data files).
#[derive(Debug, Clone)]
pub struct BlockDef {
    /// Text ID, e.g. "incus:granite".
    pub name: String,
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
}

impl BlockDef {
    /// The same texture on all six faces.
    pub fn uniform(name: &str, texture: &str) -> Self {
        Self {
            name: name.to_string(),
            opaque: true,
            see_through: false,
            inner_faces: false,
            solid: true,
            targetable: true,
            replaceable: false,
            faces: std::array::from_fn(|_| texture.to_string()),
            rotations: Default::default(),
            friction: 1.0,
        }
    }

    /// Different textures for the top, the four sides, and the bottom.
    pub fn top_side_bottom(name: &str, top: &str, side: &str, bottom: &str) -> Self {
        Self {
            name: name.to_string(),
            opaque: true,
            see_through: false,
            inner_faces: false,
            solid: true,
            targetable: true,
            replaceable: false,
            faces: [side, side, side, side, top, bottom].map(str::to_string),
            rotations: Default::default(),
            friction: 1.0,
        }
    }

    /// A see-through block with the same texture on all six faces: walk through it, build
    /// into it, see it from inside and out. `inner_faces`: grid (true) or outline (false).
    pub fn see_through(name: &str, texture: &str, inner_faces: bool) -> Self {
        Self {
            opaque: false,
            see_through: true,
            inner_faces,
            solid: false,
            replaceable: true,
            ..Self::uniform(name, texture)
        }
    }

    /// An invisible, non-solid block (air).
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            opaque: false,
            see_through: false,
            inner_faces: false,
            solid: false,
            targetable: false,
            replaceable: true,
            faces: Default::default(),
            rotations: Default::default(),
            friction: 1.0,
        }
    }

    /// Set the rotation of one face (face number as in `FACE_NAMES`).
    pub fn with_rotation(mut self, face: usize, rotation: TextureRotation) -> Self {
        self.rotations[face] = rotation;
        self
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
        registry.register(
            BlockDef::top_side_bottom("incus:grass", "grass_top", "grass_side", "dirt")
                .with_rotation(4, TextureRotation::Random), // the top
        );
        // For the structure editor and selection. Grid: every cell visible. Outline: only the
        // outside of a volume, for marking exact 3D regions.
        registry.register(BlockDef::see_through("incus:grid", "grid", true));
        registry.register(BlockDef::see_through("incus:outline", "grid", false));
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
            .add_debug_page("Blocks", blocks_debug_page)
            .add_settings_with("blocks", save_block_tweaks, load_block_tweaks);
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
    let mut registry = world.resource_mut::<BlockRegistry>();
    ui.label(format!("{} block types registered", registry.len()));
    ui.small("(Temporary built-ins until data files arrive in step 7.)");
    ui.separator();
    for (number, def) in registry.defs.iter_mut().enumerate() {
        ui.monospace(format!("{number:>5}  {}", def.name));
        if def.opaque || def.see_through {
            let [north, _, _, _, top, bottom] = &def.faces;
            ui.small(format!("        top: {top}   sides: {north}   bottom: {bottom}"));
            if def.see_through {
                let kind = if def.inner_faces { "every cell visible" } else { "outside skin only" };
                ui.small(format!("        see-through ({kind}): walk through it, build into it"));
            }
            rotation_checkboxes(ui, def);
            if def.solid {
                ui.add(egui::Slider::new(&mut def.friction, 0.0..=3.0).text("friction (1 = normal ground)"));
            }
            ui.horizontal(|ui| {
                ui.add_space(40.0);
                ui.checkbox(&mut def.targetable, "crosshair can target it");
            });
        }
    }
}

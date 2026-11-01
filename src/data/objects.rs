//! Objects — real things in the world that aren't blocks [`docs/objects.md`; David, 2026-11-01:
//! "blocks are blocks, objects are objects, and blueprints are made of both"]: a brazier, a torch, a
//! cup, a loaf, a sword, a bed; and, later, plants ("objects can be 2D, like grass, or 3d, like corn or
//! brazieres or beds").
//!
//! Each object is a data file, `assets/data/<pack>/objects/<name>.toml`, its ID `<pack>:<name>`:
//! ```toml
//! name = "Brazier"
//! model = "braziere.gltf"   # a glTF (Blockbench's export) in assets/models/
//! height = 1.5              # how tall it stands, in blocks: the model is scaled to fit
//! solid = true              # bodies bump into it (its cell, up to its height)
//! sits = "floor"            # how it's placed: "floor" (walls and ceilings later)
//! tags = ["light"]
//! ```
//! **Room kept, not built yet:** a 2D form (a sprite, or crossed sprites, with no model: grass, and in
//! meadow numbers drawn into the chunk's mesh); attachments at the model's named points (a flame
//! sprite, a light: stops 2 and 3); objects placed by Ecology from its plan, never saved.

use super::data_folder;
use super::loading::find_files;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// `assets/models`: the objects' models.
pub fn models_folder() -> PathBuf {
    super::assets_folder().join("models")
}

/// An object's number in the running game (its place in the registry). Saves use its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u16);

/// How an object is placed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sits {
    /// Standing on a floor, in the middle of its cell.
    #[default]
    Floor,
    /// On a wall (torches first, Counsel's lean), or hanging from a ceiling: not built yet.
    Wall,
    Ceiling,
}

/// One object.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectDef {
    /// Its ID (`incus:braziere`), and what it's called.
    pub name: String,
    pub display_name: String,
    /// Its model's file in `assets/models/`.
    pub model: String,
    /// How tall it stands, in blocks.
    pub height: f32,
    pub solid: bool,
    pub sits: Sits,
    pub tags: Vec<String>,
}

/// What an object's data file holds.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFile {
    #[serde(default)]
    name: Option<String>,
    model: String,
    #[serde(default = "one")]
    height: f32,
    #[serde(default)]
    solid: bool,
    #[serde(default)]
    sits: Sits,
    #[serde(default)]
    tags: Vec<String>,
}

fn one() -> f32 {
    1.0
}

/// An object from its data file's text.
pub fn parse_object(id: &str, text: &str) -> Result<ObjectDef, String> {
    let file: ObjectFile = toml::from_str(text).map_err(|error| error.to_string())?;
    if !(0.05..=16.0).contains(&file.height) {
        return Err(format!("height {} is out of range (0.05 to 16 blocks)", file.height));
    }
    let display_name = file.name.unwrap_or_else(|| id.split(':').nth(1).unwrap_or(id).replace('_', " "));
    Ok(ObjectDef { name: id.to_string(), display_name, model: file.model, height: file.height, solid: file.solid, sits: file.sits, tags: file.tags })
}

/// An object's data file, as text (kept to the settings it has, with the file's own header).
pub fn object_file_text(def: &ObjectDef, header: &str) -> String {
    let mut text = header.to_string();
    text += &format!("name = {:?}\nmodel = {:?}\nheight = {}\n", def.display_name, def.model, def.height);
    if def.solid {
        text += "solid = true\n";
    }
    if def.sits != Sits::Floor {
        text += &format!("sits = {:?}\n", format!("{:?}", def.sits).to_lowercase());
    }
    if !def.tags.is_empty() {
        text += &format!("tags = {:?}\n", def.tags);
    }
    text
}

/// Every object, from the data folder.
#[derive(bevy::prelude::Resource, Default)]
pub struct ObjectRegistry {
    defs: Vec<ObjectDef>,
    ids: HashMap<String, ObjectId>,
    files: HashMap<PathBuf, Option<SystemTime>>,
    sources: HashMap<String, PathBuf>,
    /// What's wrong, per file.
    pub problems: BTreeMap<String, String>,
    /// Goes up whenever a definition changes.
    revision: u64,
}

impl ObjectRegistry {
    pub fn load() -> Self {
        let mut registry = Self::default();
        registry.reload_changed_files();
        registry
    }

    /// Read every object file that's new or changed. True if any definition changed. Objects keep
    /// their numbers; one whose file has gone stays until the game restarts.
    pub fn reload_changed_files(&mut self) -> bool {
        let mut problems = Vec::new();
        let found = find_files(&data_folder(), "objects", &mut problems);
        let mut changed = false;
        for file in found {
            if self.files.get(&file.path) == Some(&file.modified) {
                continue;
            }
            self.files.insert(file.path.clone(), file.modified);
            let key = file.path.display().to_string();
            match std::fs::read_to_string(&file.path).map_err(|error| error.to_string()).and_then(|text| parse_object(&file.id, &text)) {
                Ok(def) => {
                    self.problems.remove(&key);
                    self.sources.insert(file.id.clone(), file.path.clone());
                    changed |= self.insert_or_replace(def);
                }
                Err(error) => {
                    self.problems.insert(key, format!("couldn't be used: {error}"));
                }
            }
        }
        for problem in problems.into_iter().filter(|problem| !problem.starts_with("no data folder")) {
            self.problems.insert("data folder".to_string(), problem);
        }
        if changed {
            self.revision += 1;
        }
        changed
    }

    fn insert_or_replace(&mut self, def: ObjectDef) -> bool {
        match self.ids.get(&def.name) {
            Some(&id) => {
                let slot = &mut self.defs[id.0 as usize];
                let changed = *slot != def;
                *slot = def;
                changed
            }
            None => {
                self.ids.insert(def.name.clone(), ObjectId(self.defs.len() as u16));
                self.defs.push(def);
                true
            }
        }
    }

    pub fn id(&self, name: &str) -> Option<ObjectId> {
        self.ids.get(name).copied()
    }

    pub fn def(&self, id: ObjectId) -> Option<&ObjectDef> {
        self.defs.get(id.0 as usize)
    }

    pub fn by_name(&self, name: &str) -> Option<&ObjectDef> {
        self.id(name).and_then(|id| self.def(id))
    }

    pub fn defs(&self) -> &[ObjectDef] {
        &self.defs
    }

    /// The data file an object came from.
    pub fn source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// For tests: one object, as if from a file.
    #[cfg(test)]
    pub fn with(defs: Vec<ObjectDef>) -> Self {
        let mut registry = Self::default();
        for def in defs {
            registry.insert_or_replace(def);
        }
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_object_file_reads_and_writes_back_the_same() {
        let text = "name = \"Brazier\"\nmodel = \"braziere.gltf\"\nheight = 1.5\nsolid = true\n";
        let def = parse_object("incus:braziere", text).unwrap();
        assert_eq!((def.display_name.as_str(), def.model.as_str(), def.height, def.solid, def.sits), ("Brazier", "braziere.gltf", 1.5, true, Sits::Floor));
        assert_eq!(object_file_text(&def, ""), text);
        // Defaults: named from its file, a block tall, not solid.
        let cup = parse_object("incus:tin_cup", "model = \"cup.gltf\"").unwrap();
        assert_eq!((cup.display_name.as_str(), cup.height, cup.solid), ("tin cup", 1.0, false));
        assert!(parse_object("incus:x", "model = \"x.gltf\"\nheight = 0").is_err());
        assert!(parse_object("incus:x", "model = \"x.gltf\"\ncolour = 3").is_err(), "unknown settings are refused");
    }
}

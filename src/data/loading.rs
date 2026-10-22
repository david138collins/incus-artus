//! Loading — block definitions from TOML data files.
//!
//! Each block is one file: `assets/data/<pack>/blocks/<name>.toml` defines the block
//! `<pack>:<name>`. The base game's pack is `incus`; a mod would add its own folder next to it.
//! Pack and file names use lowercase letters, digits and `_` only.
//!
//! A block file (every line is optional; see docs/data-files.md for the full list):
//!
//! ```toml
//! name = "Grass"                     # the name players see
//! look = "solid"                     # solid, grid or outline
//! textures = { top = "grass_top", sides = "grass_side", bottom = "dirt" }
//! random_rotation = ["top"]          # faces turned at random (only for textures with no "up")
//! friction = 1.0                     # 1 = ordinary ground, lower = slipperier
//! ```
//!
//! Face words, for `textures` and `random_rotation`: `all`, `sides`, `top`, `bottom`, `north`,
//! `south`, `east`, `west`. More specific words win: `all`, then `sides`, then single faces.
//!
//! Mistakes are reported (with the file and line), never crash the game: a file that can't
//! be read leaves its block as it was.

use super::{BlockDef, TextureRotation};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// How a block is drawn (and its usual physical behaviour).
#[derive(Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Look {
    /// An ordinary block: hides what's behind it, stops bodies.
    #[default]
    Solid,
    /// See-through, every cell's faces drawn (a lattice). Walk-through, replaceable.
    Grid,
    /// See-through, only the outside skin of a volume drawn. Walk-through, replaceable.
    Outline,
}

/// The file's contents, exactly as written. `deny_unknown_fields` turns a misspelt setting
/// into a clear error instead of being silently ignored.
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct BlockFile {
    name: Option<String>,
    look: Look,
    textures: BTreeMap<String, String>,
    random_rotation: Vec<String>,
    friction: Option<f32>,
    solid: Option<bool>,
    targetable: Option<bool>,
    replaceable: Option<bool>,
}

/// Which faces a face word means (numbers as in `FACE_NAMES`), and how specific it is.
fn faces_of(word: &str) -> Option<(&'static [usize], u8)> {
    Some(match word {
        "all" => (&[0, 1, 2, 3, 4, 5], 0),
        "sides" => (&[0, 1, 2, 3], 1),
        "north" => (&[0], 2),
        "south" => (&[1], 2),
        "east" => (&[2], 2),
        "west" => (&[3], 2),
        "top" => (&[4], 2),
        "bottom" => (&[5], 2),
        _ => return None,
    })
}

const FACE_WORDS: &str = "all, sides, top, bottom, north, south, east, west";

/// Turn a block file's text into a definition. Returns the definition plus any warnings
/// (things that were wrong but not fatal), or an error if the file can't be used at all.
pub fn parse_block(id: &str, text: &str) -> Result<(BlockDef, Vec<String>), String> {
    let file: BlockFile = toml::from_str(text).map_err(|error| error.to_string())?;
    let mut warnings = Vec::new();

    let mut def = BlockDef::uniform(id, "");
    def.display_name = file.name.unwrap_or_else(|| default_display_name(id));
    match file.look {
        Look::Solid => {}
        Look::Grid | Look::Outline => {
            def.opaque = false;
            def.see_through = true;
            def.inner_faces = file.look == Look::Grid;
            def.solid = false;
            def.replaceable = true;
        }
    }

    // Textures: general words first, so more specific ones overwrite them.
    let mut textures: Vec<(&String, &String)> = file.textures.iter().collect();
    textures.sort_by_key(|(word, _)| faces_of(word).map_or(0, |(_, rank)| rank));
    for (word, texture) in textures {
        match faces_of(word) {
            Some((faces, _)) => {
                for &face in faces {
                    def.faces[face] = texture.clone();
                }
            }
            None => warnings.push(format!("unknown face '{word}' in textures (use {FACE_WORDS})")),
        }
    }
    if def.faces.iter().any(String::is_empty) {
        warnings.push("some faces have no texture (they show the magenta checkerboard)".to_string());
    }

    for word in &file.random_rotation {
        match faces_of(word) {
            Some((faces, _)) => {
                for &face in faces {
                    def.rotations[face] = TextureRotation::Random;
                }
            }
            None => warnings.push(format!("unknown face '{word}' in random_rotation (use {FACE_WORDS})")),
        }
    }

    if let Some(friction) = file.friction {
        def.friction = friction.max(0.0);
    }
    if let Some(solid) = file.solid {
        def.solid = solid;
    }
    if let Some(targetable) = file.targetable {
        def.targetable = targetable;
    }
    if let Some(replaceable) = file.replaceable {
        def.replaceable = replaceable;
    }
    Ok((def, warnings))
}

/// "incus:red_granite" → "Red granite".
fn default_display_name(id: &str) -> String {
    let name = id.rsplit(':').next().unwrap_or(id).replace('_', " ");
    let mut letters = name.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

/// Lowercase letters, digits and `_` only.
fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// One block file found on disk.
pub struct FoundFile {
    pub id: String,
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
}

/// Every `data/<pack>/blocks/*.toml`, sorted by ID (so block numbers come out the same every
/// run). Badly named files are skipped and reported in `problems`.
pub fn find_block_files(data: &Path, problems: &mut Vec<String>) -> Vec<FoundFile> {
    let mut found = Vec::new();
    let Ok(packs) = std::fs::read_dir(data) else {
        problems.push(format!("no data folder at {}", data.display()));
        return found;
    };
    for pack in packs.flatten().filter(|entry| entry.path().is_dir()) {
        let pack_name = pack.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(pack.path().join("blocks")) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            if path.extension().is_none_or(|extension| extension != "toml") {
                continue;
            }
            let stem = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
            if !is_valid_name(&pack_name) || !is_valid_name(&stem) {
                problems.push(format!(
                    "{}: skipped; pack and file names may only use lowercase letters, digits and _",
                    path.display()
                ));
                continue;
            }
            let modified = file.metadata().and_then(|metadata| metadata.modified()).ok();
            found.push(FoundFile {
                id: format!("{pack_name}:{stem}"),
                path,
                modified,
            });
        }
    }
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_block_file() {
        let text = r#"
            name = "Grass"
            textures = { all = "dirt", sides = "grass_side", top = "grass_top" }
            random_rotation = ["top"]
            friction = 0.8
        "#;
        let (def, warnings) = parse_block("incus:grass", text).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(def.display_name, "Grass");
        assert_eq!(def.faces[4], "grass_top");
        assert_eq!(def.faces[0], "grass_side");
        assert_eq!(def.faces[5], "dirt", "'all' fills the faces nothing more specific named");
        assert_eq!(def.rotations[4], TextureRotation::Random);
        assert_eq!(def.rotations[0], TextureRotation::Fixed);
        assert_eq!(def.friction, 0.8);
        assert!(def.opaque && def.solid);
    }

    #[test]
    fn looks_set_sensible_behaviour() {
        let (grid, _) = parse_block("incus:grid", r#"look = "grid"
textures = { all = "grid" }"#).unwrap();
        assert!(grid.see_through && grid.inner_faces && !grid.solid && grid.replaceable && grid.targetable);
        let (outline, _) = parse_block("incus:outline", r#"look = "outline"
textures = { all = "grid" }"#).unwrap();
        assert!(outline.see_through && !outline.inner_faces);
    }

    #[test]
    fn a_misspelt_setting_is_an_error_with_its_line() {
        let error = parse_block("incus:stone", "textures = { all = \"stone\" }\nfrictoin = 2.0").err().unwrap();
        assert!(error.contains("frictoin"), "{error}");
        assert!(error.contains('2'), "mentions the line: {error}");
    }

    #[test]
    fn unknown_face_words_are_warnings() {
        let (_, warnings) = parse_block("incus:stone", "textures = { all = \"stone\", roof = \"x\" }").unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("roof"));
    }

    #[test]
    fn names_come_from_the_id_when_not_given() {
        assert_eq!(default_display_name("incus:red_granite"), "Red granite");
    }
}

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
//! tags = ["rock/igneous", "ore"]     # for grouping in the Registrar; "/" nests, as folders do
//! tool = false                       # a building aid: its colours stay out of the master palette
//! bar = { side = "x_side", end = "x_end", only = true }  # it can be a bar (quarter block): its
//!                                    # own textures (assets/textures/bars/; none = a share of its
//!                                    # own), and only ever a bar
//! ```
//!
//! Face words, for `textures` and `random_rotation`: `all`, `sides`, `top`, `bottom`, `north`,
//! `south`, `east`, `west`. More specific words win: `all`, then `sides`, then single faces.
//!
//! Mistakes are reported (with the file and line), never crash the game: a file that can't
//! be read leaves its block as it was.

use super::{BarLook, BlockDef, TextureRotation};
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
    tags: Vec<String>,
    tool: bool,
    bar: Option<BarFile>,
}

/// `bar = { side = "…", end = "…", only = true }`: it can be a bar, with these textures of its own
/// (in `assets/textures/bars/`; none: a share of the block's), and perhaps only ever a bar.
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct BarFile {
    side: String,
    end: String,
    only: bool,
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
    def.bar = file.bar.map(|bar| BarLook { side: bar.side, end: bar.end, only: bar.only });
    // (A block that's only ever a bar needs no whole faces.)
    if def.faces.iter().any(String::is_empty) && !def.bar.as_ref().is_some_and(|bar| bar.only) {
        warnings.push("some faces have no texture (they show the magenta checkerboard)".to_string());
    }
    if let Some(bar) = &def.bar
        && bar.side.is_empty() != bar.end.is_empty()
    {
        warnings.push("a bar needs both a side and an end texture of its own, or neither (a share of the block's)".to_string());
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
    def.tool = file.tool;
    // Tags, tidied: spaces round each `/` and at the ends trimmed, empty parts and repeats dropped.
    for tag in &file.tags {
        let parts: Vec<&str> = tag.split('/').map(str::trim).filter(|part| !part.is_empty()).collect();
        if parts.is_empty() {
            warnings.push("an empty tag (ignored)".to_string());
            continue;
        }
        let tag = parts.join("/");
        if !def.tags.contains(&tag) {
            def.tags.push(tag);
        }
    }
    Ok((def, warnings))
}

/// A block's file text, as the Registrar saves it: `header` (the file's own comment lines, kept),
/// then only the settings that differ from what the block would be without them, so the file stays
/// as short as the hand-written ones. Reads back as the same block (`parse_block`).
pub fn block_file_text(def: &BlockDef, header: &str) -> String {
    let mut text = String::from(header.trim_end());
    if !text.is_empty() {
        text += "\n\n";
    }
    let id = def.name.as_str();
    if def.display_name != default_display_name(id) {
        text += &format!("name = {:?}\n", def.display_name);
    }
    let look = match (def.see_through, def.inner_faces) {
        (true, true) => Look::Grid,
        (true, false) => Look::Outline,
        _ => Look::Solid,
    };
    match look {
        Look::Grid => text += "look = \"grid\"\n",
        Look::Outline => text += "look = \"outline\"\n",
        Look::Solid => {}
    }
    // Textures: as few words as say it ("all", then "sides", then single faces).
    let faces = &def.faces;
    let mut words: Vec<(&str, &str)> = Vec::new();
    if faces.iter().all(|face| face == &faces[0]) {
        words.push(("all", &faces[0]));
    } else {
        if faces[..4].iter().all(|face| face == &faces[0]) {
            words.push(("sides", &faces[0]));
        } else {
            for (k, word) in ["north", "south", "east", "west"].into_iter().enumerate() {
                words.push((word, &faces[k]));
            }
        }
        words.push(("top", &faces[4]));
        words.push(("bottom", &faces[5]));
    }
    let words: Vec<String> = words.into_iter().filter(|(_, texture)| !texture.is_empty()).map(|(word, texture)| format!("{word} = {texture:?}")).collect();
    if !words.is_empty() {
        text += &format!("textures = {{ {} }}\n", words.join(", "));
    }
    let random = |k: usize| def.rotations[k] == TextureRotation::Random;
    let mut rotations: Vec<&str> = Vec::new();
    if (0..6).all(random) {
        rotations.push("all");
    } else {
        if (0..4).all(random) {
            rotations.push("sides");
        } else {
            rotations.extend(["north", "south", "east", "west"].into_iter().enumerate().filter(|&(k, _)| random(k)).map(|(_, word)| word));
        }
        if random(4) {
            rotations.push("top");
        }
        if random(5) {
            rotations.push("bottom");
        }
    }
    if !rotations.is_empty() {
        let quoted: Vec<String> = rotations.iter().map(|word| format!("{word:?}")).collect();
        text += &format!("random_rotation = [{}]\n", quoted.join(", "));
    }
    if def.friction != 1.0 {
        text += &format!("friction = {}\n", def.friction);
    }
    // What the look gives, unless set otherwise.
    let see_through = look != Look::Solid;
    if def.solid == see_through {
        text += &format!("solid = {}\n", def.solid);
    }
    if !def.targetable {
        text += "targetable = false\n";
    }
    if def.replaceable != see_through {
        text += &format!("replaceable = {}\n", def.replaceable);
    }
    if !def.tags.is_empty() {
        let quoted: Vec<String> = def.tags.iter().map(|tag| format!("{tag:?}")).collect();
        text += &format!("tags = [{}]\n", quoted.join(", "));
    }
    if def.tool {
        text += "tool = true\n";
    }
    if let Some(bar) = &def.bar {
        let mut parts = Vec::new();
        if !bar.side.is_empty() || !bar.end.is_empty() {
            parts.push(format!("side = {:?}, end = {:?}", bar.side, bar.end));
        }
        if bar.only {
            parts.push("only = true".to_string());
        }
        text += &if parts.is_empty() { "bar = {}\n".to_string() } else { format!("bar = {{ {} }}\n", parts.join(", ")) };
    }
    text
}

/// A block file's comment lines at its top (kept when the Registrar saves it).
pub fn header_of(text: &str) -> String {
    text.lines().take_while(|line| line.trim().is_empty() || line.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n")
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
    find_files(data, "blocks", problems)
}

/// Every `data/<pack>/<kind>/*.toml` (`blocks`, `objects`), sorted by ID. Badly named files are
/// skipped and reported in `problems`.
pub fn find_files(data: &Path, kind: &str, problems: &mut Vec<String>) -> Vec<FoundFile> {
    let mut found = Vec::new();
    let Ok(packs) = std::fs::read_dir(data) else {
        problems.push(format!("no data folder at {}", data.display()));
        return found;
    };
    for pack in packs.flatten().filter(|entry| entry.path().is_dir()) {
        let pack_name = pack.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(pack.path().join(kind)) else {
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
    fn tags_are_tidied_and_nest_with_slashes() {
        let (def, warnings) = parse_block("incus:limonite", "textures = { all = \"limonite\" }\ntags = [\"rock / sedimentary\", \"ore\", \"ore\", \" / \"]").unwrap();
        assert_eq!(def.tags, ["rock/sedimentary", "ore"]);
        assert_eq!(warnings, ["an empty tag (ignored)"]);
    }


    #[test]
    fn a_saved_block_file_reads_back_as_the_same_block_and_keeps_its_comments() {
        let header = "# Grass: the surface of the test world.\n# More about it.";
        let files = [
            "name = \"Grass\"\ntextures = { top = \"grass_top\", sides = \"grass_side\", bottom = \"dirt\" }\nrandom_rotation = [\"top\"]\ntags = [\"ground\"]",
            "look = \"outline\"\ntextures = { all = \"grid\" }\ntool = true",
            "textures = { north = \"a\", south = \"b\", east = \"a\", west = \"a\", top = \"c\", bottom = \"c\" }\nrandom_rotation = [\"sides\", \"bottom\"]\nfriction = 0.25\nsolid = false\ntargetable = false",
            // Bars: a block that can be one (a share of its own textures), and one that's only a bar.
            "textures = { all = \"oak_log\" }\nbar = {}",
            "name = \"Stone brick\"\nbar = { side = \"stone_brick_side\", end = \"stone_brick_end\", only = true }",
        ];
        for file in files {
            let (def, _) = parse_block("incus:grass", file).unwrap();
            let written = block_file_text(&def, header);
            assert!(written.starts_with(header), "{written}");
            assert_eq!(header_of(&written).trim_end(), header);
            let (again, _) = parse_block("incus:grass", &written).unwrap();
            assert_eq!(again, def, "{written}");
        }
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

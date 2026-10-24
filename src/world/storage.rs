//! Storage — chunks as files, and the "terrain" part of a saved world.
//!
//! Only **changed** chunks are saved. Untouched terrain is made again from the world's settings
//! (later: from the seed) when the world loads, then the changed chunks are laid over it.
//!
//! A chunk file (`chunks/<x>.<y>.chunk`) is compact binary:
//! ```text
//! "IACH"  format (u16)  chunk x (i32)  chunk y (i32)
//! block names: count (u16), then each name (length u16 + text)
//! 13 sections, each either:
//!   0, name number (u16)                          every block the same
//!   1, run count (u32), then (name number u16, run length u16) ...   runs of the same block
//! ```
//! Blocks are stored by **name**, not number, so a world still loads after block files are
//! added, removed or renamed. A name that no longer exists loads as air (and is reported).

use super::block::BlockId;
use super::chunk::Chunk;
use super::position::{ChunkPos, SECTION_VOLUME, SECTIONS_PER_CHUNK};
use super::section::Section;
use super::{CompiledMap, InterpreterSettings, NextTerrain, Terrain, TestBlocks, TestWorldSettings, VoxelWorld, generate_test_world};
use crate::data::BlockRegistry;
use crate::save::{WorldFolder, write_file};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

const MAGIC: &[u8; 4] = b"IACH";
const CHUNK_FORMAT: u16 = 1;

/// A chunk as bytes.
pub fn encode_chunk(chunk: &Chunk, registry: &BlockRegistry) -> Vec<u8> {
    let mut names: Vec<BlockId> = Vec::new();
    let mut number: HashMap<BlockId, u16> = HashMap::new();
    let mut number_of = |id: BlockId| {
        *number.entry(id).or_insert_with(|| {
            names.push(id);
            (names.len() - 1) as u16
        })
    };

    // Sections first (into their own buffer), so the name list is complete before it's written.
    let mut body = Vec::new();
    for section in chunk.sections() {
        match section {
            Section::Uniform(id) => {
                body.push(0);
                body.extend(number_of(*id).to_le_bytes());
            }
            Section::Packed(_) => {
                let mut runs: Vec<(u16, u16)> = Vec::new();
                for index in 0..SECTION_VOLUME {
                    let n = number_of(section.get(index));
                    match runs.last_mut() {
                        Some((last, length)) if *last == n && *length < u16::MAX => *length += 1,
                        _ => runs.push((n, 1)),
                    }
                }
                body.push(1);
                body.extend((runs.len() as u32).to_le_bytes());
                for (n, length) in runs {
                    body.extend(n.to_le_bytes());
                    body.extend(length.to_le_bytes());
                }
            }
        }
    }

    let mut bytes = Vec::with_capacity(body.len() + 64);
    bytes.extend(MAGIC);
    bytes.extend(CHUNK_FORMAT.to_le_bytes());
    bytes.extend(chunk.pos().x.to_le_bytes());
    bytes.extend(chunk.pos().y.to_le_bytes());
    bytes.extend((names.len() as u16).to_le_bytes());
    for id in names {
        let name = registry.name(id).as_bytes();
        bytes.extend((name.len() as u16).to_le_bytes());
        bytes.extend(name);
    }
    bytes.extend(body);
    bytes
}

/// Reads bytes one value at a time, failing politely if the file is cut short.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let slice = self.bytes.get(self.at..self.at + count).ok_or("the file ends too early")?;
        self.at += count;
        Ok(slice)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}

/// Bytes back into a chunk. Also returns the names of any blocks that no longer exist.
pub fn decode_chunk(bytes: &[u8], registry: &BlockRegistry) -> Result<(Chunk, Vec<String>), String> {
    let mut reader = Reader { bytes, at: 0 };
    if reader.take(4)? != MAGIC {
        return Err("not a chunk file".to_string());
    }
    let format = reader.u16()?;
    if format > CHUNK_FORMAT {
        return Err(format!("chunk format {format} is newer than this game"));
    }
    let pos = ChunkPos::new(reader.i32()?, reader.i32()?);

    let mut ids = Vec::new();
    let mut missing = Vec::new();
    for _ in 0..reader.u16()? {
        let length = reader.u16()? as usize;
        let name = String::from_utf8_lossy(reader.take(length)?).to_string();
        ids.push(registry.id(&name).unwrap_or_else(|| {
            missing.push(name.clone());
            BlockId::AIR
        }));
    }
    let id = |n: u16| ids.get(n as usize).copied().ok_or("a block number with no name");

    let mut sections = Vec::with_capacity(SECTIONS_PER_CHUNK);
    let mut buffer = vec![BlockId::AIR; SECTION_VOLUME];
    for _ in 0..SECTIONS_PER_CHUNK {
        match reader.u8()? {
            0 => sections.push(Section::Uniform(id(reader.u16()?)?)),
            1 => {
                let mut index = 0;
                for _ in 0..reader.u32()? {
                    let block = id(reader.u16()?)?;
                    let length = reader.u16()? as usize;
                    let end = index + length;
                    if end > SECTION_VOLUME {
                        return Err("a section with too many blocks".to_string());
                    }
                    buffer[index..end].fill(block);
                    index = end;
                }
                if index != SECTION_VOLUME {
                    return Err("a section with too few blocks".to_string());
                }
                sections.push(Section::from_blocks(&buffer));
            }
            other => return Err(format!("unknown section kind {other}")),
        }
    }
    Ok((Chunk::from_sections(pos, sections), missing))
}

// ── The "terrain" part of a saved world ─────────────────────────────────────

/// What `compiled.toml` holds: how to read the world's own copy of its height map.
#[derive(Serialize, Deserialize)]
struct CompiledFile {
    width: usize,
    blocks_per_pixel: f32,
    #[serde(with = "crate::save::hex_u64")]
    seed: u64,
    interpreter: InterpreterSettings,
    /// Where new arrivals start (north, east).
    #[serde(default)]
    spawn: [i32; 2],
}

/// Writes a compiled world's map into its folder: `compiled.toml`, and `elevation.png` (16-bit
/// grey: the grey is the ground's Z).
fn save_compiled(map: &CompiledMap, spawn: IVec2, folder: &Path) -> Result<(), String> {
    let info = CompiledFile { width: map.width, blocks_per_pixel: map.blocks_per_pixel, seed: map.seed, interpreter: map.settings.clone(), spawn: spawn.to_array() };
    let text = toml::to_string_pretty(&info).map_err(|error| error.to_string())?;
    write_file(&folder.join("compiled.toml"), text.as_bytes())?;
    let file = std::fs::File::create(folder.join("elevation.png")).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), map.width as u32, map.width as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Sixteen);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    let bytes: Vec<u8> = map.z.iter().flat_map(|z| z.to_be_bytes()).collect();
    writer.write_image_data(&bytes).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}

/// Reads a compiled world's map and spawn spot back, or `None` if the world is the test cylinder.
fn load_compiled(folder: &Path) -> Result<Option<(CompiledMap, IVec2)>, String> {
    let Ok(text) = std::fs::read_to_string(folder.join("compiled.toml")) else { return Ok(None) };
    let info: CompiledFile = toml::from_str(&text).map_err(|error| format!("compiled.toml: {error}"))?;
    let file = std::fs::File::open(folder.join("elevation.png")).map_err(|error| format!("elevation.png: {error}"))?;
    let mut reader = png::Decoder::new(std::io::BufReader::new(file)).read_info().map_err(|error| format!("elevation.png: {error}"))?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("elevation.png is too big")?];
    let frame = reader.next_frame(&mut buffer).map_err(|error| format!("elevation.png: {error}"))?;
    if frame.bit_depth != png::BitDepth::Sixteen || frame.width as usize != info.width {
        return Err("elevation.png isn't the 16-bit map compiled.toml describes".to_string());
    }
    let z = buffer[..frame.buffer_size()].chunks_exact(2).map(|b| u16::from_be_bytes([b[0], b[1]])).collect();
    let map = CompiledMap { width: info.width, blocks_per_pixel: info.blocks_per_pixel, z, seed: info.seed, settings: info.interpreter };
    Ok(Some((map, IVec2::from_array(info.spawn))))
}

/// Saves the test world's settings (as `terrain.toml`), a compiled world's own copy of its map,
/// and every changed chunk (in `chunks/`).
pub fn save_terrain(world: &World, folder: &WorldFolder) -> Result<String, String> {
    folder.write_toml(world.resource::<TestWorldSettings>())?;
    match world.get_resource::<Terrain>().unwrap_or(&Terrain::Test) {
        Terrain::Compiled(map) => save_compiled(map, world.resource::<VoxelWorld>().spawn(), &folder.path)?,
        Terrain::Test => {
            let _ = std::fs::remove_file(folder.path.join("compiled.toml"));
            let _ = std::fs::remove_file(folder.path.join("elevation.png"));
        }
    }
    let chunks_folder = folder.sub_folder("chunks")?;
    let voxel = world.resource::<VoxelWorld>();
    let registry = world.resource::<BlockRegistry>();

    let mut written = BTreeSet::new();
    for &pos in voxel.changed_chunks() {
        if let Some(chunk) = voxel.chunk(pos) {
            let file_name = format!("{}.{}.chunk", pos.x, pos.y);
            write_file(&chunks_folder.join(&file_name), &encode_chunk(chunk, registry))?;
            written.insert(file_name);
        }
    }
    // Chunks saved before but not changed any more (the world was regenerated) are removed.
    let mut removed = 0;
    if let Ok(entries) = std::fs::read_dir(&chunks_folder) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".chunk") && !written.contains(&name) {
                let _ = std::fs::remove_file(entry.path());
                removed += 1;
            }
        }
    }
    let mut note = format!("{} changed chunks", written.len());
    if removed > 0 {
        note.push_str(&format!(", {removed} old ones removed"));
    }
    Ok(note)
}

/// Remakes the terrain from the saved settings, then lays the changed chunks over it.
pub fn load_terrain(world: &mut World, folder: &WorldFolder) -> Result<String, String> {
    let settings = folder.read_toml::<TestWorldSettings>()?.unwrap_or_default();
    world.insert_resource(settings.clone());
    match load_compiled(&folder.path)? {
        Some((map, spawn)) => {
            let map = Arc::new(map);
            world.insert_resource(Terrain::Compiled(map.clone()));
            super::generate_compiled_world(world, &map, spawn);
        }
        None => {
            world.insert_resource(Terrain::Test);
            let blocks = TestBlocks::from_registry(world.resource::<BlockRegistry>());
            generate_test_world(&mut world.resource_mut::<VoxelWorld>(), &settings, &blocks);
        }
    }

    let chunks_folder = folder.path.join("chunks");
    let mut loaded = 0;
    let mut problems = Vec::new();
    let mut missing_blocks = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(&chunks_folder) {
        for entry in entries.flatten() {
            if entry.path().extension().is_none_or(|extension| extension != "chunk") {
                continue;
            }
            let bytes = std::fs::read(entry.path()).map_err(|error| error.to_string())?;
            match decode_chunk(&bytes, world.resource::<BlockRegistry>()) {
                Ok((chunk, missing)) => {
                    missing_blocks.extend(missing);
                    let pos = chunk.pos();
                    let mut voxel = world.resource_mut::<VoxelWorld>();
                    voxel.insert_chunk(chunk);
                    // It differs from the generated terrain, so it's saved again next time.
                    voxel.mark_changed(pos);
                    loaded += 1;
                }
                Err(error) => problems.push(format!("{}: {error}", entry.file_name().to_string_lossy())),
            }
        }
    }
    let mut note = format!("{loaded} changed chunks");
    if !missing_blocks.is_empty() {
        let names: Vec<String> = missing_blocks.into_iter().collect();
        note.push_str(&format!(", blocks that no longer exist became air: {}", names.join(", ")));
    }
    if !problems.is_empty() {
        note.push_str(&format!(", unreadable: {}", problems.join("; ")));
    }
    Ok(note)
}

/// A fresh world: a compiled world, if the World Compiler asked for one ("Render World and
/// Spawn"), or else the test cylinder from the current settings.
pub fn fresh_terrain(world: &mut World, _seed: u64) {
    match world.get_resource_mut::<NextTerrain>().and_then(|mut next| next.0.take()) {
        Some((map, spawn)) => {
            world.insert_resource(Terrain::Compiled(map.clone()));
            super::generate_compiled_world(world, &map, spawn);
        }
        None => {
            world.insert_resource(Terrain::Test);
            super::regenerate_test_world(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chunk_survives_being_saved() {
        let registry = BlockRegistry::default();
        let blocks = TestBlocks::from_registry(&registry);
        let mut world = VoxelWorld::default();
        generate_test_world(&mut world, &TestWorldSettings::default(), &blocks);
        world.set_block(IVec3::new(3, 4, 144), BlockId::AIR);
        world.set_block(IVec3::new(3, 4, 150), blocks.stone);

        let chunk = world.chunk(ChunkPos::new(0, 0)).unwrap();
        let bytes = encode_chunk(chunk, &registry);
        let (back, missing) = decode_chunk(&bytes, &registry).unwrap();
        assert!(missing.is_empty());
        assert_eq!(back.pos(), ChunkPos::new(0, 0));
        for z in [0, 1, 143, 144, 145, 150, 399] {
            for (x, y) in [(0, 0), (3, 4), (31, 31)] {
                let block = IVec3::new(x, y, z);
                assert_eq!(back.get(block), chunk.get(block), "at {block}");
            }
        }
    }

    #[test]
    fn a_compiled_worlds_own_map_survives_being_saved() {
        let map = CompiledMap {
            width: 16,
            blocks_per_pixel: 32.0,
            z: (0..256).map(|i| 60 + (i % 200) as u16).collect(),
            seed: 0xABCD,
            settings: InterpreterSettings { detail: 5.5, ..InterpreterSettings::default() },
        };
        let folder = std::env::temp_dir().join(format!("incus-artus-compiled-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        save_compiled(&map, IVec2::new(-1234, 5678), &folder).unwrap();
        let (back, spawn) = load_compiled(&folder).unwrap().unwrap();
        assert_eq!(spawn, IVec2::new(-1234, 5678));
        assert_eq!((back.width, back.blocks_per_pixel, back.seed), (map.width, map.blocks_per_pixel, map.seed));
        assert_eq!(back.z, map.z);
        assert_eq!(back.settings, map.settings);
        // A folder without one is the test cylinder.
        let _ = std::fs::remove_dir_all(&folder);
        assert!(load_compiled(&folder).unwrap().is_none());
    }

    #[test]
    fn a_broken_file_is_refused_not_crashed_on() {
        let registry = BlockRegistry::default();
        assert!(decode_chunk(b"IACH\x01\x00", &registry).is_err());
        assert!(decode_chunk(b"nope", &registry).is_err());
    }
}

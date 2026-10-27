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
use super::{CompiledMap, InterpreterSettings, Maps, NextTerrain, Terrain, TestBlocks, TestWorldSettings, VoxelWorld, generate_test_world};
use crate::data::BlockRegistry;
use crate::save::{WorldFolder, write_file};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
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

/// What `compiled.toml` holds: how to read the world's own copy of the Terrain Compiler's maps, and
/// the Interpreter's recipe for them.
#[derive(Serialize, Deserialize)]
struct CompiledFile {
    width: usize,
    blocks_per_pixel: f32,
    #[serde(with = "crate::save::hex_u64")]
    seed: u64,
    /// The bake's height at sea level, its lowest and highest heights, and its most crushed and
    /// most stretched rock.
    water: f32,
    lowest: f32,
    highest: f32,
    most_crushed: f32,
    most_stretched: f32,
    /// Each map's file (`map-<name>.png`, 16-bit grey): the values black and white stand for.
    ranges: BTreeMap<String, [f32; 2]>,
    interpreter: InterpreterSettings,
    /// Parts of the recipe left out (the Terrain Compiler's Show switches when it was spawned), so
    /// the ground streamed in later matches. None, in worlds saved before.
    #[serde(default)]
    left_out: super::Mutes,
    /// Where new arrivals start (north, east).
    #[serde(default)]
    spawn: [i32; 2],
}

/// The compiler's maps, by file name.
fn named_maps(maps: &Maps) -> [(&'static str, &Vec<f32>); 9] {
    [
        ("coast", &maps.coast),
        ("coast_distance", &maps.coast_distance),
        ("lift_distance", &maps.lift_distance),
        ("height", &maps.height),
        ("tilt", &maps.tilt),
        ("squeeze", &maps.squeeze),
        ("push_x", &maps.push_x),
        ("push_y", &maps.push_y),
        ("push_strength", &maps.push_strength),
    ]
}

/// Writes a compiled world's maps into its folder: `compiled.toml`, and one `map-<name>.png` each
/// (16-bit grey, stretched over the map's own range, which `compiled.toml` records).
fn save_compiled(map: &CompiledMap, spawn: IVec2, folder: &Path) -> Result<(), String> {
    let m = &map.maps;
    let mut ranges = BTreeMap::new();
    for (name, values) in named_maps(m) {
        let (low, high) = values.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        let (low, high) = if low > high { (0.0, 0.0) } else { (low, high) };
        ranges.insert(name.to_string(), [low, high]);
        let span = (high - low).max(1e-12);
        let file = std::fs::File::create(folder.join(format!("map-{name}.png"))).map_err(|error| error.to_string())?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), m.width as u32, m.width as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Sixteen);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        let bytes: Vec<u8> = values.iter().flat_map(|&v| (((v - low) / span * 65_535.0).round() as u16).to_be_bytes()).collect();
        writer.write_image_data(&bytes).map_err(|error| error.to_string())?;
        writer.finish().map_err(|error| error.to_string())?;
    }
    let info = CompiledFile {
        width: m.width,
        blocks_per_pixel: m.blocks_per_pixel,
        seed: map.seed,
        water: m.water,
        lowest: m.lowest,
        highest: m.highest,
        most_crushed: m.most_crushed,
        most_stretched: m.most_stretched,
        ranges,
        interpreter: map.settings.clone(),
        left_out: map.mutes,
        spawn: spawn.to_array(),
    };
    let text = toml::to_string_pretty(&info).map_err(|error| error.to_string())?;
    write_file(&folder.join("compiled.toml"), text.as_bytes())
}

/// Reads a compiled world's maps and spawn spot back, or `None` if the world is the test cylinder.
fn load_compiled(folder: &Path) -> Result<Option<(CompiledMap, IVec2)>, String> {
    let Ok(text) = std::fs::read_to_string(folder.join("compiled.toml")) else { return Ok(None) };
    let info: CompiledFile = toml::from_str(&text).map_err(|error| format!("compiled.toml: {error}"))?;
    let read = |name: &str| -> Result<Vec<f32>, String> {
        let file_name = format!("map-{name}.png");
        let [low, high] = *info.ranges.get(name).ok_or(format!("compiled.toml has no range for {file_name}"))?;
        let file = std::fs::File::open(folder.join(&file_name)).map_err(|error| format!("{file_name}: {error}"))?;
        let mut reader = png::Decoder::new(std::io::BufReader::new(file)).read_info().map_err(|error| format!("{file_name}: {error}"))?;
        let mut buffer = vec![0; reader.output_buffer_size().ok_or(format!("{file_name} is too big"))?];
        let frame = reader.next_frame(&mut buffer).map_err(|error| format!("{file_name}: {error}"))?;
        if frame.bit_depth != png::BitDepth::Sixteen || frame.width as usize != info.width {
            return Err(format!("{file_name} isn't the 16-bit map compiled.toml describes"));
        }
        Ok(buffer[..frame.buffer_size()].chunks_exact(2).map(|b| low + u16::from_be_bytes([b[0], b[1]]) as f32 / 65_535.0 * (high - low)).collect())
    };
    let coast = read("coast")?;
    // Worlds saved before the coast distance was kept work it out again.
    let coast_distance = if info.ranges.contains_key("coast_distance") {
        read("coast_distance")?
    } else {
        super::coast_distance(&coast, info.width, info.blocks_per_pixel)
    };
    // Worlds saved before the lift have no lift points: everywhere is as far as can be from one.
    let lift_distance = if info.ranges.contains_key("lift_distance") {
        read("lift_distance")?
    } else {
        super::point_distance(&[], info.width, info.blocks_per_pixel)
    };
    let maps = Maps {
        width: info.width,
        blocks_per_pixel: info.blocks_per_pixel,
        coast,
        coast_distance,
        lift_distance,
        height: read("height")?,
        tilt: read("tilt")?,
        squeeze: read("squeeze")?,
        push_x: read("push_x")?,
        push_y: read("push_y")?,
        push_strength: read("push_strength")?,
        water: info.water,
        lowest: info.lowest,
        highest: info.highest,
        most_crushed: info.most_crushed,
        most_stretched: info.most_stretched,
    };
    let mut map = CompiledMap::new(Arc::new(maps), info.seed, info.interpreter);
    map.mutes = info.left_out;
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
            for (name, _) in named_maps(&Maps::default()) {
                let _ = std::fs::remove_file(folder.path.join(format!("map-{name}.png")));
            }
        }
    }
    let chunks_folder = folder.sub_folder("chunks")?;
    let voxel = world.resource::<VoxelWorld>();
    let registry = world.resource::<BlockRegistry>();

    let mut written = BTreeSet::new();
    for &pos in voxel.changed_chunks() {
        if let Some(chunk) = voxel.changed_chunk(pos) {
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

/// A fresh world: a compiled world, if the Terrain Compiler asked for one ("Render World and
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
        let z: Vec<f32> = (0..256).map(|i| 60.0 + (i % 200) as f32).collect();
        let mut map = CompiledMap::of_heights(16, 32.0, &z);
        // Some squeeze, so every map has something in it.
        for i in 0..256 {
            Arc::make_mut(&mut map.maps).squeeze[i] = (i as f32 * 0.37).sin();
        }
        let mutes = crate::world::Mutes { hills: true, passes: true, ..Default::default() };
        let mut map = CompiledMap::new(map.maps, 0xABCD, InterpreterSettings { detail: 5.5, ..InterpreterSettings::default() });
        map.mutes = mutes;
        let folder = std::env::temp_dir().join(format!("incus-artus-compiled-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        save_compiled(&map, IVec2::new(-1234, 5678), &folder).unwrap();
        let (back, spawn) = load_compiled(&folder).unwrap().unwrap();
        assert_eq!(spawn, IVec2::new(-1234, 5678));
        assert_eq!((back.maps.width, back.maps.blocks_per_pixel, back.seed), (map.maps.width, map.maps.blocks_per_pixel, map.seed));
        assert_eq!(back.settings, map.settings);
        assert_eq!(back.mutes, mutes, "the parts left out are remembered");
        // 16-bit maps: the ground comes back within a small fraction of a block, everywhere.
        for k in 0..64 {
            let (north, east) = (-200.0 + k as f32 * 6.3, 170.0 - k as f32 * 5.1);
            let (a, b) = (map.height(north, east).unwrap(), back.height(north, east).unwrap());
            assert!((a - b).abs() < 0.05, "{a} vs {b} at {north}, {east}");
        }
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

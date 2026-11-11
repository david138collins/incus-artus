//! A world exported as columns (David 2026-11-11: "export nice25 so that we can use it as a 'generated' world to look
//! at with the new graphics engine, before rebuilding world gen after the graphics engine is done"). Old's own world
//! generation runs once, here; the new engine reads the files and never runs it.
//!
//! **The form** (the Oracle's tablet, `vision/oracle-review.md` 5.5, in a plain form any program can read): each column
//! is its ground, its river's or lake's water, and its profile (the top block, up to four layers under it each to a
//! depth, then stone: `Layers`), the same three things old's chunks are filled from (`CompiledMap::chunk`,
//! `Layers::block_at`). The profiles are few, so a column keeps a number into a list of them. A world with no caves and
//! no changes (as the test world) is all plain columns, so this is the whole world.
//!
//! **The files**, in the folder given:
//! - `world.toml`: where the world lies, the tiles, sea level and height, the blocks by name, every profile.
//! - `tiles/<tx> <ty>.bin`: a tile of `TILE` × `TILE` columns, `tx` north and `ty` east from the world's least corner;
//!   deflate-compressed (raw deflate). Inside, three runs one after another, each a column at a time, north fastest:
//!   the ground as the change from the column before (zigzag LEB128; −1 past the map, the first from 0), the water's
//!   top (LEB128; 0 none), the profile's number (LEB128). A tile with no column on the map has no file.
//! - `README.txt`: this, and the rule that makes a column's blocks.

use super::mineralogy::Layers;
use super::terrain::{SLOPE_REACH, TerrainBlocks};
use super::{CompiledMap, SEA_LEVEL, WORLD_HEIGHT};
use crate::data::BlockRegistry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Columns a side of a tile.
pub const TILE: i32 = 512;

/// The profiles found so far, each with its number.
#[derive(Default)]
struct Profiles {
    list: Vec<Layers>,
    numbers: HashMap<Layers, u32>,
}

impl Profiles {
    fn number(&mut self, layers: Layers) -> u32 {
        if let Some(&n) = self.numbers.get(&layers) {
            return n;
        }
        let n = self.list.len() as u32;
        self.list.push(layers);
        self.numbers.insert(layers, n);
        n
    }
}

/// What an export came to.
pub struct Exported {
    pub tiles: usize,
    pub columns: u64,
    pub bytes: u64,
    pub profiles: usize,
    pub seconds: f64,
}

fn put_leb(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn zigzag(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

/// The world `map` exported into `folder` (its tiles from `tiles.0` to `tiles.1`, north and east, counted from the
/// world's least corner; all of them with `None`), on `threads` threads. `name` and `made_by` go in `world.toml`.
pub fn export(map: &CompiledMap, registry: &BlockRegistry, folder: &Path, tiles: Option<([i32; 2], [i32; 2])>, threads: usize, name: &str, made_by: &str) -> Result<Exported, String> {
    let started = std::time::Instant::now();
    let blocks = TerrainBlocks::from_registry(registry);
    let mineralogy = map.mineralogy(&blocks);
    let half = (map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0).ceil() as i32;
    let least = -half;
    let across = (2 * half + TILE - 1) / TILE;
    let (from, to) = tiles.unwrap_or(([0, 0], [across, across]));
    let tile_folder = folder.join("tiles");
    std::fs::create_dir_all(&tile_folder).map_err(|error| format!("{}: {error}", tile_folder.display()))?;
    let jobs: Vec<[i32; 2]> = (from[0]..to[0].min(across)).flat_map(|tx| (from[1]..to[1].min(across)).map(move |ty| [tx, ty])).collect();
    let next = AtomicUsize::new(0);
    let (written, columns, bytes) = (AtomicUsize::new(0), AtomicU64::new(0), AtomicU64::new(0));
    let profiles = Mutex::new(Profiles::default());
    let failed: Mutex<Option<String>> = Mutex::new(None);
    std::thread::scope(|scope| {
        // How far it's got, every half minute.
        scope.spawn(|| {
            let mut waited = 0;
            while next.load(Ordering::Relaxed) < jobs.len() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                waited += 1;
                if waited % 30 == 0 {
                    eprintln!("Export: {} of {} tiles taken, {} written, {:.2} GB so far, {:.0} s", next.load(Ordering::Relaxed).min(jobs.len()), jobs.len(), written.load(Ordering::Relaxed), bytes.load(Ordering::Relaxed) as f64 / 1e9, started.elapsed().as_secs_f64());
                }
            }
        });
        for _ in 0..threads.max(1) {
            scope.spawn(|| {
                // This thread's numbers for the profiles it's met (the shared list asked only for new ones).
                let mut known: HashMap<Layers, u32> = HashMap::new();
                let side = (TILE + 2 * SLOPE_REACH) as usize;
                let mut ground = vec![-1i32; side * side];
                let mut water = vec![0i32; side * side];
                loop {
                    let k = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&[tx, ty]) = jobs.get(k) else { break };
                    let (x0, y0) = (least + tx * TILE, least + ty * TILE);
                    // The tile's columns and a margin round it (the slope reads its neighbours).
                    let margin = if mineralogy.reads_slope { SLOPE_REACH } else { 0 };
                    let mut any = false;
                    for j in -margin..TILE + margin {
                        for i in -margin..TILE + margin {
                            let at = (i + SLOPE_REACH) as usize + (j + SLOPE_REACH) as usize * side;
                            match map.column(x0 + i, y0 + j) {
                                Some((z, over)) => {
                                    ground[at] = z;
                                    water[at] = over.unwrap_or(0);
                                    any |= (0..TILE).contains(&i) && (0..TILE).contains(&j);
                                }
                                None => {
                                    ground[at] = -1;
                                    water[at] = 0;
                                }
                            }
                        }
                    }
                    if !any {
                        continue;
                    }
                    let n = (TILE * TILE) as usize;
                    let (mut grounds, mut waters, mut numbers) = (Vec::with_capacity(n * 2), Vec::with_capacity(n), Vec::with_capacity(n));
                    let mut last = 0i64;
                    for j in 0..TILE {
                        for i in 0..TILE {
                            let at = (i + SLOPE_REACH) as usize + (j + SLOPE_REACH) as usize * side;
                            let z = ground[at];
                            put_leb(&mut grounds, zigzag(z as i64 - last));
                            last = z as i64;
                            if z < 0 {
                                put_leb(&mut waters, 0);
                                put_leb(&mut numbers, 0);
                                continue;
                            }
                            put_leb(&mut waters, water[at] as u64);
                            // Its profile, as the chunks make it (`CompiledMap::chunk`).
                            let slope = if mineralogy.reads_slope {
                                let near = |di: i32, dj: i32| {
                                    let g = ground[(i + di + SLOPE_REACH) as usize + (j + dj + SLOPE_REACH) as usize * side];
                                    if g < 0 { z } else { g }
                                };
                                super::terrain::slope_across(near(SLOPE_REACH, 0) - near(-SLOPE_REACH, 0), near(0, SLOPE_REACH) - near(0, -SLOPE_REACH))
                            } else {
                                0.0
                            };
                            let (north, east) = ((x0 + i) as f32 + 0.5, (y0 + j) as f32 + 0.5);
                            let over = Some(water[at]).filter(|&top| top > 0);
                            let layers = mineralogy.layers(map, super::placer::Column { north, east, ground: z as f32, slope, climate: map.climate(north, east, z as f32), water: over });
                            let number = match known.get(&layers) {
                                Some(&n) => n,
                                None => {
                                    let n = profiles.lock().map_or(0, |mut profiles| profiles.number(layers));
                                    known.insert(layers, n);
                                    n
                                }
                            };
                            put_leb(&mut numbers, number as u64);
                        }
                    }
                    let mut raw = grounds;
                    raw.extend(waters);
                    raw.extend(numbers);
                    let packed = miniz_oxide::deflate::compress_to_vec(&raw, 6);
                    let path = tile_folder.join(format!("{tx} {ty}.bin"));
                    if let Err(error) = std::fs::write(&path, &packed) {
                        if let Ok(mut failed) = failed.lock() {
                            *failed = Some(format!("{}: {error}", path.display()));
                        }
                        break;
                    }
                    written.fetch_add(1, Ordering::Relaxed);
                    columns.fetch_add(n as u64, Ordering::Relaxed);
                    bytes.fetch_add(packed.len() as u64, Ordering::Relaxed);
                }
            });
        }
    });
    if let Some(error) = failed.into_inner().ok().flatten() {
        return Err(error);
    }
    let profiles = profiles.into_inner().map_err(|_| "the profiles' list was poisoned".to_string())?;
    // `world.toml`: where it lies, the tiles, the blocks by name, every profile.
    let mut used: Vec<crate::world::BlockId> = vec![blocks.stone, blocks.water];
    for layers in &profiles.list {
        used.push(layers.top);
        used.extend(layers.under().iter().map(|&(block, _)| block));
    }
    used.sort();
    used.dedup();
    let mut text = format!(
        "# A world exported as columns from old Incus Artus (src/world/export.rs; README.txt has the rule that makes its blocks).\nname = {name:?}\nmade_by = {made_by:?}\n# The least corner's column (north, east), columns a side of a tile, tiles a side.\nleast = [{least}, {least}]\ntile = {TILE}\ntiles = {across}\nsea_level = {SEA_LEVEL}\nworld_height = {WORLD_HEIGHT}\nstone = {:?}\nwater = {:?}\n\n",
        registry.name(blocks.stone),
        registry.name(blocks.water)
    );
    text += "# Every profile, by its number: the top block, then each block under it and the depth below the ground it reaches.\n";
    for (n, layers) in profiles.list.iter().enumerate() {
        let under: Vec<String> = layers.under().iter().map(|&(block, depth)| format!("[{:?}, {depth}]", registry.name(block))).collect();
        text += &format!("[[profile]]\nnumber = {n}\ntop = {:?}\nunder = [{}]\n\n", registry.name(layers.top), under.join(", "));
    }
    text += &format!("# The blocks named above: {}\n", used.iter().map(|&b| registry.name(b).to_string()).collect::<Vec<_>>().join(", "));
    std::fs::write(folder.join("world.toml"), text).map_err(|error| format!("world.toml: {error}"))?;
    std::fs::write(folder.join("README.txt"), README).map_err(|error| format!("README.txt: {error}"))?;
    Ok(Exported { tiles: written.into_inner(), columns: columns.into_inner(), bytes: bytes.into_inner(), profiles: profiles.list.len(), seconds: started.elapsed().as_secs_f64() })
}

/// `INCUS_EXPORT_WORLD=<a saved world's name>`: that world exported into `exports/<name>/`, and the game ends there
/// (`main.rs`, before anything else starts; no window). `INCUS_EXPORT_TILES=<from north>,<from east>,<to north>,<to
/// east>` (tiles, from the least corner) for a part of it. Whether it was asked for.
pub fn export_if_asked() -> bool {
    let Ok(name) = std::env::var("INCUS_EXPORT_WORLD") else { return false };
    let folder = crate::debug::project_folder().join("worlds").join(&name);
    let map = match super::storage::load_compiled(&folder) {
        Ok(Some((map, _))) => map,
        Ok(None) => {
            eprintln!("Export: {} isn't a compiled world", folder.display());
            return true;
        }
        Err(error) => {
            eprintln!("Export: {} couldn't be read: {error}", folder.display());
            return true;
        }
    };
    let tiles = std::env::var("INCUS_EXPORT_TILES").ok().and_then(|v| {
        let n: Vec<i32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        (n.len() == 4).then(|| ([n[0], n[1]], [n[2], n[3]]))
    });
    let out = crate::debug::project_folder().join("exports").join(&name);
    let threads = std::thread::available_parallelism().map_or(8, |n| n.get()).saturating_sub(2).max(1);
    let made_by = format!("old Incus Artus, build {} ({})", crate::diagnostics::BUILD_NUMBER, crate::diagnostics::BUILD);
    eprintln!("Export: {name} into {} on {threads} threads{}", out.display(), tiles.map_or(String::new(), |t| format!(", tiles {t:?}")));
    match export(&map, &BlockRegistry::load(), &out, tiles, threads, &name, &made_by) {
        Ok(done) => eprintln!(
            "Export: done: {} tiles, {} columns, {:.2} GB ({:.2} bytes a column), {} profiles, {:.0} s ({:.0} columns a second)",
            done.tiles,
            done.columns,
            done.bytes as f64 / 1e9,
            done.bytes as f64 / done.columns.max(1) as f64,
            done.profiles,
            done.seconds,
            done.columns as f64 / done.seconds.max(1e-3)
        ),
        Err(error) => eprintln!("Export: failed: {error}"),
    }
    true
}

const README: &str = "A world exported as columns from old Incus Artus (src/world/export.rs).

world.toml says where it lies (the least corner's column, north and east; columns a side of a tile; tiles a side),
sea level, the world's height, and every profile by its number.

tiles/<tx> <ty>.bin is the tile tx tiles north and ty east of the least corner: tile x tile columns, raw deflate.
Inside, three runs one after another, each a column at a time, north fastest (column i north, j east: i + j * tile):
  1. the ground's Z as the change from the column before (zigzag LEB128; the first from 0); -1 is past the map;
  2. the top of a river's or lake's water over it (LEB128; 0 none);
  3. its profile's number (LEB128; 0 past the map).
A tile with no column on the map has no file.

The blocks of a column with ground G, water W and a profile (top, under = [[block, depth], ...], deepest last;
D the last depth, or 0 with none), at height Z from 0 up:
  Z < G - D            stone
  Z < G                the first under-layer whose depth reaches G - Z
  Z = G                the top block
  Z < sea_level or Z < W   water
  otherwise            air
";

#[cfg(test)]
mod tests {
    use super::*;

    /// A tile read back: the three runs undone, each column's ground, water and profile.
    pub fn read_tile(bytes: &[u8]) -> Vec<(i32, i32, u32)> {
        let raw = miniz_oxide::inflate::decompress_to_vec(bytes).expect("a tile");
        let mut at = 0;
        let mut leb = || {
            let (mut v, mut shift) = (0u64, 0);
            loop {
                let byte = raw[at];
                at += 1;
                v |= ((byte & 0x7F) as u64) << shift;
                if byte & 0x80 == 0 {
                    return v;
                }
                shift += 7;
            }
        };
        let n = (TILE * TILE) as usize;
        let mut last = 0i64;
        let grounds: Vec<i32> = (0..n)
            .map(|_| {
                let v = leb();
                last += ((v >> 1) as i64) ^ -((v & 1) as i64);
                last as i32
            })
            .collect();
        let waters: Vec<i32> = (0..n).map(|_| leb() as i32).collect();
        let numbers: Vec<u32> = (0..n).map(|_| leb() as u32).collect();
        (0..n).map(|k| (grounds[k], waters[k], numbers[k])).collect()
    }

    #[test]
    fn zigzag_and_leb_are_undone() {
        let mut out = Vec::new();
        for v in [0i64, 1, -1, 300, -300, 70_000] {
            put_leb(&mut out, zigzag(v));
        }
        let mut at = 0;
        let mut values = Vec::new();
        while at < out.len() {
            let (mut v, mut shift) = (0u64, 0);
            loop {
                let byte = out[at];
                at += 1;
                v |= ((byte & 0x7F) as u64) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            values.push(((v >> 1) as i64) ^ -((v & 1) as i64));
        }
        assert_eq!(values, [0, 1, -1, 300, -300, 70_000]);
    }

    /// The test world (`worlds/nice25 test`) exported over a few tiles round the spawn, read back, every column's
    /// ground and water the same as `CompiledMap::column` and its profile the same as the chunks' (`INCUS_EXPORT_DIR`
    /// for where; the scratch folder otherwise). Skipped without the world.
    #[test]
    #[ignore]
    fn the_test_world_exports_and_reads_back() {
        let folder = crate::debug::project_folder().join("worlds").join("nice25 test");
        let Ok(Some((map, _))) = super::super::storage::load_compiled(&folder) else {
            eprintln!("No test world: the export test is skipped");
            return;
        };
        let registry = BlockRegistry::load();
        let out = std::env::var("INCUS_EXPORT_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir().join("incus-export-test"));
        let _ = std::fs::remove_dir_all(&out);
        let half = (map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0).ceil() as i32;
        let middle = half / TILE;
        let done = export(&map, &registry, &out, Some(([middle - 1, middle - 1], [middle + 1, middle + 1])), 8, "nice25 test", "the export test").expect("the export");
        eprintln!("{} tiles, {} columns, {} bytes ({:.2} a column), {} profiles, {:.1} s ({:.0} columns a second)", done.tiles, done.columns, done.bytes, done.bytes as f64 / done.columns as f64, done.profiles, done.seconds, done.columns as f64 / done.seconds);
        assert_eq!(done.tiles, 4);
        // Read one tile back: every 37th column against the map, and its profile against the chunks' rule.
        let blocks = TerrainBlocks::from_registry(&registry);
        let mineralogy = map.mineralogy(&blocks);
        let text = std::fs::read_to_string(out.join("world.toml")).expect("world.toml");
        let toml: toml::Table = toml::from_str(&text).expect("world.toml reads");
        let listed = toml["profile"].as_array().expect("profiles").len();
        assert_eq!(listed, done.profiles);
        let (tx, ty) = (middle, middle);
        let tile = read_tile(&std::fs::read(out.join("tiles").join(format!("{tx} {ty}.bin"))).expect("the tile"));
        let least = -half;
        let mut checked = 0;
        for k in (0..tile.len()).step_by(37) {
            let (i, j) = (k as i32 % TILE, k as i32 / TILE);
            let (x, y) = (least + tx * TILE + i, least + ty * TILE + j);
            let (ground, water, number) = tile[k];
            match map.column(x, y) {
                Some((z, over)) => {
                    assert_eq!((ground, water), (z, over.unwrap_or(0)), "the column at {x} {y}");
                    // Its profile's top, as the chunks would make it (the slope's neighbours read again here).
                    let at = |di: i32, dj: i32| map.column(x + di, y + dj).map_or(z, |(g, _)| g);
                    let slope = if mineralogy.reads_slope { super::super::terrain::slope_across(at(SLOPE_REACH, 0) - at(-SLOPE_REACH, 0), at(0, SLOPE_REACH) - at(0, -SLOPE_REACH)) } else { 0.0 };
                    let (north, east) = (x as f32 + 0.5, y as f32 + 0.5);
                    let layers = mineralogy.layers(&map, super::super::placer::Column { north, east, ground: z as f32, slope, climate: map.climate(north, east, z as f32), water: over });
                    let profile = &toml["profile"].as_array().expect("profiles")[number as usize];
                    assert_eq!(profile["top"].as_str(), Some(registry.name(layers.top)), "the top at {x} {y}");
                    checked += 1;
                }
                None => assert_eq!(ground, -1),
            }
        }
        assert!(checked > 1000, "{checked} columns checked");
    }
}

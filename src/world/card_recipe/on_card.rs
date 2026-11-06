//! The world handed to the card as it loads (the regroup, phase 2.0): packed once, put on the card, and the
//! recipe's pipeline compiled, all on the loading lane, off the frame. Measured, and checked as it goes: the
//! card makes the columns round the spawn and every one is compared with the processor's own.
//!
//! **The card is shared with the frame**, one queue for both, so its work goes in small pieces, each waited
//! for before the next [found 2026-11-05: the default bake's 178 MB at once held a frame 92 ms, and the
//! pipeline's first use, 4,096 columns, 26 ms]: the pack in slices of a megabyte, a first use of a few
//! columns, then the rest a thousand or so at a time (about a millisecond each once warm).

use super::{Pack, source};
use crate::whole::{Fix, Pos};
use crate::world::caves::CavePlan;
use crate::world::{CompiledMap, TerrainBlocks};
use std::time::Instant;

/// The world on the card: its pack, the pipeline that makes its columns, and sight's three that make
/// cells (`sight/cell.wgsl`: the grounds, mineralogy's choices, the cells).
pub struct WorldOnCard {
    pub data: wgpu::Buffer,
    pub words: usize,
    pub columns: wgpu::ComputePipeline,
    pub grounds: wgpu::ComputePipeline,
    pub choices: wgpu::ComputePipeline,
    pub cells: wgpu::ComputePipeline,
}

const SIGHT: &str = include_str!("../../sight/cell.wgsl");

/// The columns' entry: for each place (north and east, in 256ths of a block), its ground and water as
/// the world fills it (`world_column`), or −1 past the map.
const COLUMNS: &str = "
@group(0) @binding(1) var<storage, read> places: array<vec2<i32>>;
@group(0) @binding(2) var<storage, read_write> made: array<vec2<i32>>;
@compute @workgroup_size(64)
fn columns(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&places)) {
        return;
    }
    let column = world_column(places[i].x, places[i].y);
    if (column.z == 0) {
        made[i] = vec2<i32>(-1, 0);
    } else {
        made[i] = column.xy;
    }
}
";

/// How many columns a side the check makes round the spawn.
const CHECKED: i32 = 64;
/// The pack goes to the card this much at a time (bytes).
const SLICE: usize = 1 << 20;
/// Columns asked of the card at a time.
const BATCH: usize = 1024;

/// Packs the world, puts it on the card and compiles its pipeline, then checks it round `spawn`. Run on
/// the loading lane: it waits on the card. Returns the world on the card and a report of the timings.
pub fn send(device: &wgpu::Device, queue: &wgpu::Queue, map: &CompiledMap, caves: &CavePlan, blocks: &TerrainBlocks, colours: &crate::sight::cell::FarColours, spawn: (i32, i32)) -> Result<(WorldOnCard, String), String> {
    let ms = |since: Instant| since.elapsed().as_secs_f64() * 1000.0;
    let started = Instant::now();
    let whole = map.whole().ok_or("the recipe is in fractions, which the card doesn't have")?;
    let mut pack = Pack::world(&whole.map, &whole.carving);
    pack.caves(caves);
    let mineralogy = map.mineralogy(blocks);
    let program = mineralogy.program().whole(map.seed);
    pack.mineralogy(&mineralogy, &program, map.seed, Fix::from_f32(map.settings.mineralogy.crust));
    pack.sight(colours);
    let packed = ms(started);

    let started = Instant::now();
    let bytes: Vec<u8> = pack.words.iter().flat_map(|x| x.to_le_bytes()).collect();
    let data = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the world"), size: bytes.len().max(4) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    for (k, slice) in bytes.chunks(SLICE).enumerate() {
        queue.write_buffer(&data, (k * SLICE) as u64, slice);
        queue.submit([]);
        device.poll(wgpu::PollType::wait_indefinitely()).map_err(|error| format!("the card stopped taking the world: {error}"))?;
    }
    let put = ms(started);

    let started = Instant::now();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the recipe"), source: wgpu::ShaderSource::Wgsl((source() + COLUMNS + SIGHT).into()) });
    let pipeline = |entry: &str| device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some(entry), layout: None, module: &module, entry_point: Some(entry), compilation_options: Default::default(), cache: None });
    let columns = pipeline("columns");
    let compiled = ms(started);
    let started = Instant::now();
    let (grounds, choices, cells) = (pipeline("sight_grounds_pass"), pipeline("sight_choices_pass"), pipeline("sight_cells_pass"));
    let sight_compiled = ms(started);
    let world = WorldOnCard { data, words: pack.words.len(), columns, grounds, choices, cells };

    // The check: the card's columns round the spawn against the processor's.
    let places: Vec<(i32, i32)> = (0..CHECKED * CHECKED).map(|i| (spawn.0 - CHECKED / 2 + i % CHECKED, spawn.1 - CHECKED / 2 + i / CHECKED)).collect();
    let started = Instant::now();
    let mut card = world.columns_at(device, queue, &places[..64]);
    let first_use = ms(started);
    let started = Instant::now();
    for batch in places[64..].chunks(BATCH) {
        card.extend(world.columns_at(device, queue, batch));
    }
    let on_the_card = ms(started);
    let started = Instant::now();
    let processor: Vec<[i32; 2]> = places.iter().map(|&(x, y)| map.column(x, y).map_or([-1, 0], |(z, water)| [z, water.unwrap_or(0)])).collect();
    let on_the_processor = ms(started);
    let same = card.iter().zip(&processor).filter(|(a, b)| a == b).count();
    let first_wrong = card.iter().zip(&processor).zip(&places).find(|((a, b), _)| a != b).map(|((a, b), at)| format!("; the first that differs, {at:?}: card {a:?}, processor {b:?}")).unwrap_or_default();
    let report = format!(
        "The world on the card: packed in {packed:.1} ms ({} numbers, {:.0} KB), put on the card in {put:.1} ms, its pipeline compiled in {compiled:.0} ms (sight's three {sight_compiled:.0} ms), first used in {first_use:.1} ms; {} columns round the spawn made on the card in {on_the_card:.1} ms (the processor {on_the_processor:.1} ms), {same} the same{first_wrong}",
        world.words,
        world.words as f64 * 4.0 / 1024.0,
        places.len()
    );
    if same != places.len() {
        return Err(report);
    }
    // Sight's cells round the spawn (32 blocks, through the surface), against the chunks' blocks.
    let report = format!("{report}; {}", check_cells(device, queue, &world, map, caves, blocks, colours, spawn, &processor)?);
    // A measurement for the far view's bake, when asked for.
    let report = match std::env::var("INCUS_MEASURE_BAKE").ok().map(|windows| windows.parse().unwrap_or(64)) {
        Some(windows) => format!("{report}\n{}", measure_bake(device, queue, &world, map, windows)),
        None => report,
    };
    Ok((world, report))
}

/// Sight's cells of 32 blocks over the columns round the spawn (the check's 64 × 64), through the band from
/// the lowest ground to the highest ground or water, made on the card and counted again from the processor's
/// own chunks (`sight::cell::sums_of_box`). `Err` if any differ.
#[allow(clippy::too_many_arguments)]
fn check_cells(device: &wgpu::Device, queue: &wgpu::Queue, world: &WorldOnCard, map: &CompiledMap, caves: &CavePlan, blocks: &TerrainBlocks, colours: &crate::sight::cell::FarColours, spawn: (i32, i32), columns: &[[i32; 2]]) -> Result<String, String> {
    use crate::sight::cell::{Sums, sums_of_box};
    let ms = |since: Instant| since.elapsed().as_secs_f64() * 1000.0;
    let (x0, y0) = (spawn.0 - CHECKED / 2, spawn.1 - CHECKED / 2);
    let inside: Vec<&[i32; 2]> = columns.iter().filter(|c| c[0] >= 0).collect();
    let Some(low) = inside.iter().map(|c| c[0]).min() else { return Ok("no cells to check past the map".into()) };
    let high = inside.iter().map(|c| c[0].max(c[1])).max().unwrap_or(low).max(crate::world::SEA_LEVEL);
    let mut cells = Vec::new();
    for z in (low.div_euclid(32) * 32..=high).step_by(32) {
        for (cx, cy) in [(0, 0), (32, 0), (0, 32), (32, 32)] {
            cells.push([x0 + cx, y0 + cy, z, 32]);
        }
    }
    let started = Instant::now();
    let card = world.cells_at(device, queue, [x0, y0, CHECKED, CHECKED], &cells);
    let on_the_card = ms(started);
    let started = Instant::now();
    let mut chunks = std::collections::HashMap::new();
    for cx in (x0 - 1).div_euclid(32)..=(x0 + CHECKED).div_euclid(32) {
        for cy in (y0 - 1).div_euclid(32)..=(y0 + CHECKED).div_euclid(32) {
            if let Some(chunk) = map.chunk(crate::world::ChunkPos::new(cx, cy), blocks, caves) {
                chunks.insert((cx, cy), chunk);
            }
        }
    }
    let block = |[x, y, z]: [i32; 3]| {
        if z < 0 {
            blocks.stone
        } else if z >= crate::world::WORLD_HEIGHT {
            crate::world::BlockId::AIR
        } else {
            chunks.get(&(x.div_euclid(32), y.div_euclid(32))).map_or(crate::world::BlockId::AIR, |chunk| chunk.block_at(x, y, z))
        }
    };
    let mut first_wrong = None;
    let mut same = 0;
    for (cell, c) in cells.iter().zip(&card) {
        let wide = |k: usize| c[k] as u64 | (c[k + 1] as u64) << 32;
        let on_card = Sums { solid: c[0] as u64, water: c[1] as u64, faces: std::array::from_fn(|d| c[2 + d] as u64), colour: std::array::from_fn(|k| c[8 + k] as u64), light: c[11] as u64, light_squared: wide(12), height: c[14] as u64, height_squared: wide(15) };
        if on_card == sums_of_box([cell[0], cell[1], cell[2]], cell[3], &block, colours) {
            same += 1;
        } else if first_wrong.is_none() {
            first_wrong = Some(*cell);
        }
    }
    let report = format!("sight's {} cells of 32 round the spawn made on the card in {on_the_card:.1} ms (the processor, from its chunks, {:.0} ms), {same} the same", cells.len(), ms(started));
    match first_wrong {
        None => Ok(report),
        Some(cell) => Err(format!("{report}; the first that differs: {cell:?}")),
    }
}

/// The far view's bake measured (phase 3.1; `INCUS_MEASURE_BAKE=<windows>`, 64 if no number): the card
/// makes the columns of that many windows of 512 × 512 blocks spread over the map, and they're counted for
/// the cells a bake would keep at each size: those holding a surface (part solid, part air or water), from
/// each cell column's lowest ground to its highest ground or water. Caves aren't counted (the default world
/// has none yet). Also the card's speed at full batches, for the bake's time.
fn measure_bake(device: &wgpu::Device, queue: &wgpu::Queue, world: &WorldOnCard, map: &CompiledMap, windows: usize) -> String {
    const SIDE: i32 = 512;
    const SIZES: [i32; 5] = [16, 32, 64, 128, 256];
    let half = (map.maps.width as f32 * map.maps.blocks_per_pixel / 2.0) as i32;
    let across = 2 * half;
    let sea = crate::world::SEA_LEVEL;
    let (mut mixed, mut cell_columns) = ([0u64; 5], [0u64; 5]);
    let (mut columns, mut on_the_map, mut card_ms) = (0u64, 0u64, 0.0);
    // Sight's cells of 32 through each window's surface band, timed (the bake's real work).
    let (mut cells_made, mut cell_ms, mut cell_columns_made) = (0usize, 0.0, 0u64);
    let mut roll = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = |below: i32| {
        roll = roll.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        ((roll >> 33) % below.max(1) as u64) as i32
    };
    for _ in 0..windows {
        let (wx, wy) = (-half + next(across - SIDE) / 256 * 256, -half + next(across - SIDE) / 256 * 256);
        let places: Vec<(i32, i32)> = (0..SIDE * SIDE).map(|i| (wx + i % SIDE, wy + i / SIDE)).collect();
        let started = Instant::now();
        let mut made = Vec::with_capacity(places.len());
        for batch in places.chunks(16384) {
            made.extend(world.columns_at(device, queue, batch));
        }
        card_ms += started.elapsed().as_secs_f64() * 1000.0;
        columns += places.len() as u64;
        {
            let inside: Vec<&[i32; 2]> = made.iter().filter(|c| c[0] >= 0).collect();
            if let (Some(low), Some(high)) = (inside.iter().map(|c| c[0]).min(), inside.iter().map(|c| c[0].max(c[1]).max(sea)).max()) {
                let mut cells = Vec::new();
                for z in (low.div_euclid(32) * 32..=high).step_by(32) {
                    for cx in (0..SIDE).step_by(32) {
                        for cy in (0..SIDE).step_by(32) {
                            cells.push([wx + cx, wy + cy, z, 32]);
                        }
                    }
                }
                let started = Instant::now();
                let made_cells = world.cells_at(device, queue, [wx, wy, SIDE, SIDE], &cells);
                cell_ms += started.elapsed().as_secs_f64() * 1000.0;
                cells_made += made_cells.len();
                cell_columns_made += (SIDE * SIDE) as u64;
            }
        }
        on_the_map += made.iter().filter(|c| c[0] >= 0).count() as u64;
        for (k, &size) in SIZES.iter().enumerate() {
            for cy in 0..SIDE / size {
                for cx in 0..SIDE / size {
                    let (mut low, mut high) = (i32::MAX, i32::MIN);
                    for y in 0..size {
                        for x in 0..size {
                            let [z, water] = made[((cy * size + y) * SIDE + cx * size + x) as usize];
                            if z >= 0 {
                                low = low.min(z);
                                high = high.max(z.max(water).max(if z < sea { sea } else { 0 }));
                            }
                        }
                    }
                    if low <= high {
                        mixed[k] += (high / size - low / size + 1) as u64;
                        cell_columns[k] += 1;
                    }
                }
            }
        }
    }
    // Scaled from the windows to the whole map; 16 bytes a kept cell, 4 a cell column (where its cells start).
    let scale = (across as f64 * across as f64) / columns.max(1) as f64;
    let mut lines = vec![format!(
        "The bake measured: {windows} windows, {columns} columns ({:.0}% on the map) in {card_ms:.0} ms on the card: {:.1} million columns a second at full batches; the map {across} blocks across, {:.2} billion columns, so a level's recipe pass about {:.0} s",
        on_the_map as f64 * 100.0 / columns.max(1) as f64,
        columns as f64 / card_ms.max(0.001) / 1000.0,
        across as f64 * across as f64 / 1e9,
        across as f64 * across as f64 / (columns as f64 / card_ms.max(0.001) * 1000.0)
    )];
    lines.push(format!(
        "  sight's cells of 32 through the surface: {cells_made} in {cell_ms:.0} ms, {:.1} million columns a second, so the world's cells of 32 about {:.0} s",
        cell_columns_made as f64 / cell_ms.max(0.001) / 1000.0,
        across as f64 * across as f64 / (cell_columns_made as f64 / cell_ms.max(0.001) * 1000.0)
    ));
    let mut from = 0.0;
    for k in (0..SIZES.len()).rev() {
        let cells = mixed[k] as f64 * scale;
        let bytes = cells * 16.0 + cell_columns[k] as f64 * scale * 4.0;
        from += bytes;
        lines.push(format!(
            "  cells of {} blocks: {:.2} surface cells a cell column, {:.1} million kept, {:.0} MB; baked from here up {:.0} MB",
            SIZES[k],
            mixed[k] as f64 / cell_columns[k].max(1) as f64,
            cells / 1e6,
            bytes / 1e6,
            from / 1e6
        ));
    }
    lines.join("\n")
}

impl WorldOnCard {
    /// Sight's cells (`sight::cell::Sums`, as 17 numbers: see `cell.wgsl`), each a least corner and a size,
    /// all with their columns inside `region` (least corner x and y, columns across and along). Waits for
    /// the card.
    pub fn cells_at(&self, device: &wgpu::Device, queue: &wgpu::Queue, region: [i32; 4], cells: &[[i32; 4]]) -> Vec<[u32; 17]> {
        use wgpu::util::DeviceExt;
        if cells.is_empty() {
            return Vec::new();
        }
        let bytes = |v: &[i32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let storage = |label: &str, size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: size.max(16), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let region_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("the region"), contents: &bytes(&region), usage: wgpu::BufferUsages::STORAGE });
        let ground_count = ((region[2] + 6) * (region[3] + 6)) as u32;
        let chosen_count = ((region[2] + 2) * (region[3] + 2)) as u32;
        let grounds = storage("grounds", ground_count as u64 * 8);
        let chosen = storage("chosen", chosen_count as u64 * 16);
        let flat: Vec<i32> = cells.iter().flatten().copied().collect();
        let cells_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("cells"), contents: &bytes(&flat), usage: wgpu::BufferUsages::STORAGE });
        let size = cells.len() as u64 * 17 * 4;
        let sums = storage("sums", size);
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("sums read back"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind = |pipeline: &wgpu::ComputePipeline, entries: &[(u32, &wgpu::Buffer)]| {
            let entries: Vec<wgpu::BindGroupEntry> = entries.iter().map(|&(binding, buffer)| wgpu::BindGroupEntry { binding, resource: buffer.as_entire_binding() }).collect();
            device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &pipeline.get_bind_group_layout(0), entries: &entries })
        };
        let passes = [
            (&self.grounds, bind(&self.grounds, &[(0, &self.data), (1, &region_buffer), (2, &grounds)]), ground_count),
            (&self.choices, bind(&self.choices, &[(0, &self.data), (1, &region_buffer), (2, &grounds), (3, &chosen)]), chosen_count),
            (&self.cells, bind(&self.cells, &[(0, &self.data), (1, &region_buffer), (3, &chosen), (4, &cells_buffer), (5, &sums)]), cells.len() as u32),
        ];
        let mut encoder = device.create_command_encoder(&Default::default());
        for (k, (pipeline, group, count)) in passes.iter().enumerate() {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            // The cells pass gives each cell a group of its own (in rows of at most 65,535 groups).
            let groups = if k == 2 { *count } else { count.div_ceil(64) };
            pass.dispatch_workgroups(groups.min(65_535), groups.div_ceil(65_535), 1);
        }
        encoder.copy_buffer_to_buffer(&sums, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        if device.poll(wgpu::PollType::wait_indefinitely()).is_err() {
            return Vec::new();
        }
        let Ok(view) = readback.slice(..).get_mapped_range() else { return Vec::new() };
        view.chunks_exact(17 * 4).map(|cell| std::array::from_fn(|k| u32::from_le_bytes([cell[4 * k], cell[4 * k + 1], cell[4 * k + 2], cell[4 * k + 3]]))).collect()
    }

    /// The columns at `places` (x and y, whole blocks): each one's ground and water (0 none), [−1, 0] past
    /// the map. Waits for the card.
    pub fn columns_at(&self, device: &wgpu::Device, queue: &wgpu::Queue, places: &[(i32, i32)]) -> Vec<[i32; 2]> {
        use wgpu::util::DeviceExt;
        if places.is_empty() {
            return Vec::new();
        }
        let bytes = |v: &[i32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let flat: Vec<i32> = places.iter().flat_map(|&(x, y)| [Pos::middle_of(x).0, Pos::middle_of(y).0]).collect();
        let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("places"), contents: &bytes(&flat), usage: wgpu::BufferUsages::STORAGE });
        let size = places.len() as u64 * 8;
        let output = device.create_buffer(&wgpu::BufferDescriptor { label: Some("columns"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("columns read back"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.columns.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: self.data.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: input.as_entire_binding() }, wgpu::BindGroupEntry { binding: 2, resource: output.as_entire_binding() }],
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.columns);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((places.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        if device.poll(wgpu::PollType::wait_indefinitely()).is_err() {
            return Vec::new();
        }
        let Ok(view) = readback.slice(..).get_mapped_range() else { return Vec::new() };
        view.chunks_exact(8).map(|b| [i32::from_le_bytes([b[0], b[1], b[2], b[3]]), i32::from_le_bytes([b[4], b[5], b[6], b[7]])]).collect()
    }
}

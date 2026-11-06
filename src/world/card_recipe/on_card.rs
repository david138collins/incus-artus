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

/// The world on the card: its pack, and the pipeline that makes its columns.
pub struct WorldOnCard {
    pub data: wgpu::Buffer,
    pub words: usize,
    pub columns: wgpu::ComputePipeline,
}

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
pub fn send(device: &wgpu::Device, queue: &wgpu::Queue, map: &CompiledMap, caves: &CavePlan, blocks: &TerrainBlocks, spawn: (i32, i32)) -> Result<(WorldOnCard, String), String> {
    let ms = |since: Instant| since.elapsed().as_secs_f64() * 1000.0;
    let started = Instant::now();
    let whole = map.whole().ok_or("the recipe is in fractions, which the card doesn't have")?;
    let mut pack = Pack::world(&whole.map, &whole.carving);
    pack.caves(caves);
    let mineralogy = map.mineralogy(blocks);
    let program = mineralogy.program().whole(map.seed);
    pack.mineralogy(&mineralogy, &program, map.seed, Fix::from_f32(map.settings.mineralogy.crust));
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
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the recipe"), source: wgpu::ShaderSource::Wgsl((source() + COLUMNS).into()) });
    let columns = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("the world's columns"), layout: None, module: &module, entry_point: Some("columns"), compilation_options: Default::default(), cache: None });
    let compiled = ms(started);
    let world = WorldOnCard { data, words: pack.words.len(), columns };

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
        "The world on the card: packed in {packed:.1} ms ({} numbers, {:.0} KB), put on the card in {put:.1} ms, its pipeline compiled in {compiled:.0} ms, first used in {first_use:.1} ms; {} columns round the spawn made on the card in {on_the_card:.1} ms (the processor {on_the_processor:.1} ms), {same} the same{first_wrong}",
        world.words,
        world.words as f64 * 4.0 / 1024.0,
        places.len()
    );
    if same != places.len() {
        return Err(report);
    }
    Ok((world, report))
}

impl WorldOnCard {
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

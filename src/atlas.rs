//! The far bricks as textures (plan 2.2, change 1): soft bricks blended by the texture hardware.
//!
//! Each far brick is kept again as a block of texels in 3D textures (atlases), its cells (8 a side, or 4 or 2: step 3)
//! and a border of one cell on its high sides holding the next cells over, 9 × 9 × 9 at 8, so the hardware's filter
//! blends the eight cells round a point in one fetch, across a brick's edge too. Under a band every cell is solid, over it empty, as the bands say.
//!
//! How solid and how watery (true, unspread, 16-bit floats), and beside them the colour and the six faces' shares the
//! spread way (the Oracle's): each cell that shows no surface takes the colour and faces of the nearest one that does,
//! in the brick, filtered unweighted, in 8 bits, the surface kept beside the colour. Build 128 flew it against the exact
//! way (colour and faces times surface, 16-bit, one divide after the filter: old's blend to the hardware's precision)
//! and David saw no difference ("as far as the far blend, they all look the same"), so spread was kept: a little faster,
//! and 1.1 GB lighter on the card (`journal/docs/experiments.md`, change 1).
//!
//! Built once, after every region is in, on a thread of its own from the cells kept as they came (`Kept`), an atlas layer
//! at a time; till then the far eye blends in code as before. Each brick's place in the atlas is kept beside its place in
//! the pool (`slots`, the same places as the bricks' index).

use crate::bricks::{Level, NOT_SUMMED};
use crate::far::Cell;
use std::collections::VecDeque;

/// Bricks a side of an atlas layer (north and east).
pub const ACROSS: u32 = 64;
/// No brick in the atlas at a place.
pub const NO_SLOT: u32 = u32::MAX;

/// The far cells kept as they came, for the atlas: each level's columns' entries and their bands' cells, in the same order
/// as the card's bands (`Level::entry`).
#[derive(Default)]
pub struct Kept {
    pub levels: Vec<Level>,
    pub columns: Vec<Vec<(u32, Vec<Cell>)>>,
}

impl Kept {
    pub fn new(levels: &[Level]) -> Self {
        Self { levels: levels.to_vec(), columns: levels.iter().map(|l| vec![(NOT_SUMMED, Vec::new()); (l.across * l.across) as usize]).collect() }
    }

    /// A brick's texels a side in the atlas: its cells a side (the most of any level's) and a border.
    pub fn side(&self) -> u32 {
        self.levels.iter().map(|l| l.wide.max(l.tall)).max().unwrap_or(1) as u32 + 1
    }

    /// A column's entry and band kept.
    pub fn keep(&mut self, level: usize, cx: i32, cy: i32, entry: u32, cells: Vec<Cell>) {
        let l = &self.levels[level - crate::far::FIRST];
        if let Some(at) = l.entry(cx, cy) {
            self.columns[level - crate::far::FIRST][(at - l.entries_at) as usize] = (entry, cells);
        }
    }

    /// The cell at a place, as the far eye reads it: none if its column isn't summed; past the level, under the world,
    /// under a band (solid) and over it (empty) as the bands say.
    fn cell(&self, level: &Level, cx: i32, cy: i32, cz: i32) -> Option<Look> {
        let Some(at) = level.entry(cx, cy) else { return Some(Look::default()) };
        let (entry, cells) = &self.columns[level.level - crate::far::FIRST][(at - level.entries_at) as usize];
        if *entry == NOT_SUMMED {
            return None;
        }
        let low = (*entry & 0xFFFF) as u16 as i16 as i32;
        let count = (*entry >> 16) as i32;
        if cz < 0 || (cz < low && low > 0) {
            return Some(Look { solid: 1.0, ..Look::default() });
        }
        if cz < low || cz >= low + count {
            return Some(Look::default());
        }
        Some(Look::of(&cells[(cz - low) as usize]))
    }
}

/// A cell as the far eye reads it (`far.wgsl`'s `cell_of`).
#[derive(Clone, Copy, Default)]
struct Look {
    solid: f32,
    water: f32,
    colour: [f32; 3],
    plus: [f32; 3],
    minus: [f32; 3],
    surface: f32,
}

impl Look {
    fn of(cell: &Cell) -> Self {
        let f = cell.faces.map(|v| v as f32);
        let west = (255.0 - f.iter().sum::<f32>()).max(0.0);
        let c = cell.colour;
        Self {
            solid: cell.solid as f32 / 255.0,
            water: cell.water as f32 / 255.0,
            colour: [((c >> 20) & 1023) as f32 / 1023.0, ((c >> 10) & 1023) as f32 / 1023.0, (c & 1023) as f32 / 1023.0],
            plus: [f[2] / 255.0, f[4] / 255.0, f[0] / 255.0],
            minus: [f[3] / 255.0, west / 255.0, f[1] / 255.0],
            surface: (cell.showing as f32 / 256.0).min(4.0),
        }
    }
}

/// The atlas on the card. It follows the window round the eye (step 4): bricks put in and given back as it moves,
/// their slots used again, and grown (copied into textures twice as deep) only if it's full.
pub struct Atlas {
    /// Its size in texels, and a brick's texels a side.
    pub size: [u32; 3],
    pub side: u32,
    pub solid: wgpu::TextureView,
    /// Spread: colour and surface ÷ 4, plus faces, minus faces.
    pub spread: [wgpu::TextureView; 3],
    /// The textures themselves: how solid, the colour, the faces plus and minus.
    textures: [wgpu::Texture; 4],
    /// Each brick place's slot in the atlas (`NO_SLOT` none), packed: north, east, up in slots, 8 bits each.
    pub slots: wgpu::Buffer,
    /// Each place's slot as the card has it, the slots given back, the slots made, and how many fit.
    slot_of: Vec<u32>,
    free: Vec<u32>,
    made: u32,
    capacity: u32,
    /// Bricks in it.
    pub bricks: usize,
    /// Its texels' bytes on the card.
    pub bytes: u64,
    /// How long it took to build and send at first (seconds).
    pub seconds: f32,
    /// Bumped whenever the textures are made again (grown): bind groups made with the old ones are stale.
    pub generation: u32,
}

/// A 32-bit float as a 16-bit one (round to nearest even), for how solid and how watery.
fn half(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xFF) as i32;
    let mantissa = bits & 0x7F_FFFF;
    if exponent == 0xFF {
        return sign | 0x7C00 | if mantissa != 0 { 0x200 } else { 0 };
    }
    let e = exponent - 127 + 15;
    if e >= 0x1F {
        return sign | 0x7C00;
    }
    if e <= 0 {
        if e < -10 {
            return sign;
        }
        let m = mantissa | 0x80_0000;
        let shift = (14 - e) as u32;
        let half_m = m >> shift;
        let rest = m & ((1 << shift) - 1);
        let midpoint = 1 << (shift - 1);
        let round = if rest > midpoint || (rest == midpoint && half_m & 1 == 1) { 1 } else { 0 };
        return sign | (half_m + round) as u16;
    }
    let half_m = mantissa >> 13;
    let rest = mantissa & 0x1FFF;
    let round = if rest > 0x1000 || (rest == 0x1000 && half_m & 1 == 1) { 1 } else { 0 };
    sign | (((e as u32) << 10) + half_m + round) as u16
}

/// A brick to put in the atlas: its level, its place in the index, and its least cell.
#[derive(Clone, Copy, Debug)]
pub struct Brick {
    pub level: usize,
    pub place: u32,
    pub origin: [i32; 3],
}

/// Slot `n`, packed as the card reads it: north and east in the layer, and the layer.
fn slot(n: u32) -> u32 {
    (n % ACROSS) | (((n / ACROSS) % ACROSS) << 8) | ((n / (ACROSS * ACROSS)) << 16)
}

/// The deepest a 3D texture may be (wgpu's default limit), in texels.
const DEEPEST: u32 = 2048;

/// The atlas's four textures at `size` (how solid and how watery, then the colour and the faces spread).
fn textures(device: &wgpu::Device, size: [u32; 3]) -> [wgpu::Texture; 4] {
    let texture = |label: &str, format: wgpu::TextureFormat| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: size[2] },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    use wgpu::TextureFormat::{Rg16Float, Rgba8Unorm};
    [texture("the far bricks: solid and watery", Rg16Float), texture("the far bricks: colour", Rgba8Unorm), texture("the far bricks: faces plus", Rgba8Unorm), texture("the far bricks: faces minus", Rgba8Unorm)]
}

/// A texel's four words as the textures hold them: how solid and how watery (16-bit floats), and the spread look's
/// colour and surface ÷ 4, faces plus, faces minus (8 bits).
fn texel_bytes(look: &Look, spread: &Look) -> [[u8; 4]; 4] {
    let mut solid = [0u8; 4];
    solid[..2].copy_from_slice(&half(look.solid).to_le_bytes());
    solid[2..].copy_from_slice(&half(look.water).to_le_bytes());
    let l = spread;
    let rows = [[l.colour[0], l.colour[1], l.colour[2], l.surface / 4.0], [l.plus[0], l.plus[1], l.plus[2], 0.0], [l.minus[0], l.minus[1], l.minus[2], 0.0]];
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [solid, rows[0].map(byte), rows[1].map(byte), rows[2].map(byte)]
}

/// The bricks the pool holds (those with some band in their own cells), level by level.
fn bricks(kept: &Kept) -> Vec<Brick> {
    let mut list = Vec::new();
    for l in &kept.levels {
        let mut marked = vec![false; (l.bricks * l.bricks * l.bricks_up) as usize];
        for (at, (entry, _)) in kept.columns[l.level - crate::far::FIRST].iter().enumerate() {
            if *entry == NOT_SUMMED {
                continue;
            }
            let low = (*entry & 0xFFFF) as u16 as i16 as i32;
            let count = (*entry >> 16) as i32;
            let (x, y) = (at as i32 % l.across, at as i32 / l.across);
            for cz in low.max(0)..(low + count).min(l.up) {
                marked[(((y / l.wide) * l.bricks + x / l.wide) * l.bricks_up + cz / l.tall) as usize] = true;
            }
        }
        for (k, _) in marked.iter().enumerate().filter(|(_, m)| **m) {
            let k = k as i32;
            let (bz, bx, by) = (k % l.bricks_up, (k / l.bricks_up) % l.bricks, k / l.bricks_up / l.bricks);
            list.push(Brick { level: l.level, place: l.index_at + k as u32, origin: [l.from[0] + bx * l.wide, l.from[1] + by * l.wide, bz * l.tall] });
        }
    }
    list
}

/// A brick's looks, `side` a side (x fastest, then y, then z), its border the next cells over.
fn texels(kept: &Kept, brick: &Brick, side: u32) -> Vec<Look> {
    let l = &kept.levels[brick.level - crate::far::FIRST];
    let side = side as i32;
    let mut out = Vec::with_capacity((side * side * side) as usize);
    for k in 0..side {
        for j in 0..side {
            for i in 0..side {
                out.push(kept.cell(l, brick.origin[0] + i, brick.origin[1] + j, brick.origin[2] + k).unwrap_or_default());
            }
        }
    }
    out
}

/// The spread way: each texel with no surface takes the colour and faces of the nearest one with surface (six-way steps
/// through the brick), its own surface kept (none).
fn spread(looks: &[Look], side: u32) -> Vec<Look> {
    let side = side as i32;
    let mut out = looks.to_vec();
    let mut from: Vec<Option<usize>> = looks.iter().enumerate().map(|(k, l)| (l.surface > 0.0).then_some(k)).collect();
    let mut queue: VecDeque<usize> = (0..looks.len()).filter(|&k| looks[k].surface > 0.0).collect();
    while let Some(k) = queue.pop_front() {
        let (i, j, z) = (k as i32 % side, (k as i32 / side) % side, k as i32 / side / side);
        for (di, dj, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
            let (a, b, c) = (i + di, j + dj, z + dz);
            if a < 0 || b < 0 || c < 0 || a >= side || b >= side || c >= side {
                continue;
            }
            let n = (a + b * side + c * side * side) as usize;
            if from[n].is_none() {
                from[n] = from[k];
                queue.push_back(n);
            }
        }
    }
    for (k, source) in from.iter().enumerate() {
        if let Some(s) = source
            && looks[k].surface <= 0.0
        {
            out[k].colour = looks[*s].colour;
            out[k].plus = looks[*s].plus;
            out[k].minus = looks[*s].minus;
        }
    }
    out
}

/// Builds the atlas from what's kept and sends it to the card, an atlas layer (64 × 64 bricks) at a time: the bricks
/// `keep` says (with a window, the coarse levels' and the window's as it is), with a quarter again of room and a layer
/// besides for the window's moves.
pub fn build(device: &wgpu::Device, queue: &wgpu::Queue, kept: &Kept, keep: impl Fn(&Brick) -> bool) -> Atlas {
    let started = std::time::Instant::now();
    let list: Vec<Brick> = bricks(kept).into_iter().filter(|b| keep(b)).collect();
    let per_layer = (ACROSS * ACROSS) as usize;
    let side = kept.side();
    let layers = ((list.len() * 5 / 4).div_ceil(per_layer) as u32 + 1).min(DEEPEST / side);
    assert!(list.len() <= layers as usize * per_layer, "the far atlas is deeper than the card allows");
    let size = [ACROSS * side, ACROSS * side, layers * side];
    let made = textures(device, size);
    let texels_a_layer = (size[0] * size[1] * side) as usize;
    let mut slots = vec![NO_SLOT; kept.levels.last().map_or(0, |l| (l.index_at + (l.bricks * l.bricks * l.bricks_up) as u32) as usize)];
    for (layer, chunk) in list.chunks(per_layer).enumerate() {
        let mut bytes = [vec![0u8; texels_a_layer * 4], vec![0u8; texels_a_layer * 4], vec![0u8; texels_a_layer * 4], vec![0u8; texels_a_layer * 4]];
        for (k, brick) in chunk.iter().enumerate() {
            let (sx, sy) = (k as u32 % ACROSS, k as u32 / ACROSS);
            slots[brick.place as usize] = slot((layer * per_layer + k) as u32);
            let looks = texels(kept, brick, side);
            let spread_looks = spread(&looks, side);
            for (t, look) in looks.iter().enumerate() {
                let (i, j, z) = (t as u32 % side, (t as u32 / side) % side, t as u32 / side / side);
                let at = ((sx * side + i) + (sy * side + j) * size[0] + z * size[0] * size[1]) as usize;
                for (b, word) in bytes.iter_mut().zip(texel_bytes(look, &spread_looks[t])) {
                    b[at * 4..at * 4 + 4].copy_from_slice(&word);
                }
            }
        }
        for (texture, b) in made.iter().zip(&bytes) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 * side }, aspect: wgpu::TextureAspect::All },
                b,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size[0] * 4), rows_per_image: Some(size[1]) },
                wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: side },
            );
        }
        queue.submit([]);
    }
    let slots_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the far bricks' slots in the atlas"), size: (slots.len() as u64 * 4).max(16), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    queue.write_buffer(&slots_buffer, 0, &slots.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    queue.submit([]);
    let view = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
    Atlas {
        size,
        side,
        solid: view(&made[0]),
        spread: [view(&made[1]), view(&made[2]), view(&made[3])],
        textures: made,
        slots: slots_buffer,
        slot_of: slots,
        free: Vec::new(),
        made: list.len() as u32,
        capacity: layers * per_layer as u32,
        bricks: list.len(),
        bytes: (size[0] * size[1] * size[2]) as u64 * 16,
        seconds: started.elapsed().as_secs_f32(),
        generation: 0,
    }
}

impl Atlas {
    /// Bricks the window brought, made from the cells kept and put in: each in a slot given back if there is one, a
    /// new one if not (the atlas grown first if it's full).
    pub fn add(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, kept: &Kept, bricks: &[Brick]) {
        let side = self.side;
        for brick in bricks {
            let s = match self.free.pop() {
                Some(s) => s,
                None => {
                    if self.made == self.capacity {
                        self.grow(device, queue);
                    }
                    self.made += 1;
                    slot(self.made - 1)
                }
            };
            let looks = texels(kept, brick, side);
            let spread_looks = spread(&looks, side);
            let mut bytes = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
            for (look, spread_look) in looks.iter().zip(&spread_looks) {
                for (b, word) in bytes.iter_mut().zip(texel_bytes(look, spread_look)) {
                    b.extend_from_slice(&word);
                }
            }
            let origin = wgpu::Origin3d { x: (s & 0xFF) * side, y: ((s >> 8) & 0xFF) * side, z: (s >> 16) * side };
            for (texture, b) in self.textures.iter().zip(&bytes) {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin, aspect: wgpu::TextureAspect::All },
                    b,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(side * 4), rows_per_image: Some(side) },
                    wgpu::Extent3d { width: side, height: side, depth_or_array_layers: side },
                );
            }
            self.slot_of[brick.place as usize] = s;
            queue.write_buffer(&self.slots, brick.place as u64 * 4, &s.to_le_bytes());
            self.bricks += 1;
        }
    }

    /// Bricks the window left, given back: their places' slots emptied and kept for the next to come.
    pub fn remove(&mut self, queue: &wgpu::Queue, places: &[u32]) {
        for &place in places {
            let s = std::mem::replace(&mut self.slot_of[place as usize], NO_SLOT);
            if s != NO_SLOT {
                self.free.push(s);
                queue.write_buffer(&self.slots, place as u64 * 4, &NO_SLOT.to_le_bytes());
                self.bricks -= 1;
            }
        }
    }

    /// The atlas made twice as deep (as deep as the card allows at most): the old textures copied into the new and
    /// handed to the card at once, so the bricks written before land in the old and are carried over.
    fn grow(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let layers = self.size[2] / self.side;
        let deeper = (layers * 2).min(DEEPEST / self.side);
        assert!(deeper > layers, "the far atlas is as deep as the card allows ({layers} layers)");
        let size = [self.size[0], self.size[1], deeper * self.side];
        let made = textures(device, size);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the far atlas, grown") });
        for (old, new) in self.textures.iter().zip(&made) {
            encoder.copy_texture_to_texture(old.as_image_copy(), new.as_image_copy(), wgpu::Extent3d { width: self.size[0], height: self.size[1], depth_or_array_layers: self.size[2] });
        }
        queue.submit([encoder.finish()]);
        let view = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        self.solid = view(&made[0]);
        self.spread = [view(&made[1]), view(&made[2]), view(&made[3])];
        self.textures = made;
        self.size = size;
        self.capacity = deeper * ACROSS * ACROSS;
        self.bytes = (size[0] * size[1] * size[2]) as u64 * 16;
        self.generation += 1;
        log::info!("The far textures grown to {deeper} layers, {:.0} MB", self.bytes as f64 / 1e6);
    }
}

#[cfg(test)]
mod tests {
    use super::half;

    /// The 16-bit floats are the standard's (spot values against known bit patterns).
    #[test]
    fn halves_are_the_standard_s() {
        for (v, bits) in [(0.0f32, 0x0000u16), (1.0, 0x3C00), (-2.0, 0xC000), (0.5, 0x3800), (65504.0, 0x7BFF), (1e-8, 0x0000), (0.333_333_34, 0x3555), (4.0, 0x4400)] {
            assert_eq!(half(v), bits, "{v}");
        }
    }
}

#[cfg(test)]
mod card_tests {
    use super::*;
    use crate::bricks::Store;
    use crate::doorway::Doorway;
    use crate::far::{FarColours, region, regions_of};

    /// The test's shader: the far eye's own, with a kernel blending each point by the code and by the hardware.
    const KERNEL: &str = r#"
@group(0) @binding(20) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(21) var<storage, read_write> answers: array<vec4<f32>>;
fn put(at: u32, look: Look) {
    answers[at] = vec4<f32>(look.solid, look.water, look.surface, select(0.0, 1.0, look.known));
    answers[at + 1u] = vec4<f32>(look.colour, 0.0);
    answers[at + 2u] = vec4<f32>(look.plus, 0.0);
    answers[at + 3u] = vec4<f32>(look.minus, 0.0);
}
@compute @workgroup_size(64)
fn test_blends(@builtin(global_invocation_id) id: vec3<u32>) {
    let n = id.x;
    if (n >= arrayLength(&points)) {
        return;
    }
    let p = points[n];
    let level = i32(p.w);
    put(n * 8u, blend(level, p.xyz));
    put(n * 8u + 4u, blend_soft(level, p.xyz));
}
"#;

    /// Change 1's test: at points through the regions round the spawn, at every level from 32 to 1,024, the hardware's
    /// blend against the code's, value by value. The card's filter weighs the eight cells to about 1/256 (build 128: the
    /// exact way's pass mark of 1/1,000 missed by it), so how solid and how watery are held to 1/200, and known and
    /// unknown must be the code's at every point. The spread way's colour differs by design where little surface shows
    /// (it spreads into empty neighbours where old's weighs by surface): reported by surface, not held to a bar; the
    /// pictures it makes, and David's look, are its test (`journal/docs/experiments.md`). At each brick side (step 3).
    #[test]
    fn the_hardware_blends_as_the_code_does() {
        for side in [2, 4, 8] {
            blends_as_the_code_does(side);
        }
    }

    fn blends_as_the_code_does(side: i32) {
        let mut doorway = Doorway::open(&crate::doorway::test_world()).unwrap();
        let colours = FarColours::load(&mut doorway).unwrap();
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).expect("a graphics card");
        let largest = adapter.limits().max_buffer_size.min(adapter.limits().max_storage_buffer_binding_size).min((2 << 30) - 1);
        let required_limits = wgpu::Limits { max_buffer_size: largest, max_storage_buffer_binding_size: largest, ..wgpu::Limits::default() };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("a test"), required_limits, ..Default::default() })).expect("the card's device");
        let errors = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut store = Store::new(&device, &queue, regions_of(&doorway), doorway.height(), side, 10);
        let mut summed = Vec::new();
        for ry in -1..=1 {
            for rx in -1..=1 {
                let columns = region(&doorway, &colours, [rx, ry]).unwrap();
                store.put_region(&device, &queue, [rx, ry], &columns);
                summed.push(([rx, ry], columns));
            }
        }
        store.follow(&device, &queue, [512.0, 512.0], f32::INFINITY);
        let atlas = build(&device, &queue, &store.kept, |_| true);
        // The points: in the nine regions, near each column's band (where blends mix most), at every level from 32 to
        // 1,024, scattered through each cell.
        let mut points: Vec<[f32; 4]> = Vec::new();
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        for level in crate::far::FIRST..=crate::far::REGION {
            let size = (1 << level) as f64;
            for _ in 0..4000 {
                let (r, columns) = &summed[(next() * summed.len() as f64) as usize % summed.len()];
                let n = 1i32 << (crate::far::REGION - level);
                let k = (next() * (n * n) as f64) as usize % (n * n) as usize;
                let column = &columns[level - crate::far::FIRST][k];
                let (cx, cy) = (r[0] * n + k as i32 % n, r[1] * n + k as i32 / n);
                let cz = column.low as f64 + next() * (column.sums.len() as f64 + 2.0) - 1.0;
                points.push([((cx as f64 + next()) * size) as f32, ((cy as f64 + next()) * size) as f32, (cz.max(0.0) * size) as f32, level as f32]);
            }
        }
        let source = format!("{}\n{}", include_str!("far.wgsl"), KERNEL);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("change 1's test"), source: wgpu::ShaderSource::Wgsl(source.into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("change 1's test"), layout: None, module: &module, entry_point: Some("test_blends"), compilation_options: Default::default(), cache: None });
        let camera = device.create_buffer(&wgpu::BufferDescriptor { label: Some("a test camera"), size: 18 * 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut numbers = vec![0f32; 18 * 4];
        numbers[16 * 4..16 * 4 + 3].copy_from_slice(&atlas.size.map(|v| v as f32));
        numbers[16 * 4 + 3] = atlas.side as f32;
        queue.write_buffer(&camera, 0, &numbers.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let points_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("points"), size: (points.len() * 16) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&points_buffer, 0, &points.iter().flatten().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let size = (points.len() * 8 * 16) as u64;
        let answers = device.create_buffer(&wgpu::BufferDescriptor { label: Some("answers"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("answers read back"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("soft"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        let entries = [
            wgpu::BindGroupEntry { binding: 0, resource: camera.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: store.table.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: store.entries.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: store.index.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: store.pool.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 11, resource: atlas.slots.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 12, resource: wgpu::BindingResource::Sampler(&sampler) },
            wgpu::BindGroupEntry { binding: 13, resource: wgpu::BindingResource::TextureView(&atlas.solid) },
            wgpu::BindGroupEntry { binding: 14, resource: wgpu::BindingResource::TextureView(&atlas.spread[0]) },
            wgpu::BindGroupEntry { binding: 15, resource: wgpu::BindingResource::TextureView(&atlas.spread[1]) },
            wgpu::BindGroupEntry { binding: 16, resource: wgpu::BindingResource::TextureView(&atlas.spread[2]) },
            wgpu::BindGroupEntry { binding: 20, resource: points_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 21, resource: answers.as_entire_binding() },
        ];
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("change 1's test"), layout: &pipeline.get_bind_group_layout(0), entries: &entries });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("change 1's test") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("change 1's test"), timestamp_writes: None });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups((points.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&answers, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);
        let error = pollster::block_on(errors.pop());
        assert!(error.is_none(), "the card refused change 1's test: {error:?}");
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let got: Vec<f32> = readback.slice(..).get_mapped_range().unwrap().as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect();
        // The largest differences from the code's, by value: solid, water, colour, faces (and where).
        let names = ["solid", "water", "colour", "faces"];
        let mut report = String::new();
        let mut worst = [(0.0f32, 0usize); 4];
        let mut unknown = 0;
        for p in 0..points.len() {
            let base = p * 32;
            let code = &got[base..base + 16];
            let soft = &got[base + 16..base + 32];
            if code[3] < 0.5 || soft[3] < 0.5 {
                unknown += (code[3] != soft[3]) as usize;
                continue;
            }
            let differ = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0f32, f32::max);
            // Colour and faces only where there's surface to weigh them (with none, both are rock's and nothing).
            let surfaced = code[2] > 1e-3;
            let d = [(code[0] - soft[0]).abs(), (code[1] - soft[1]).abs(), if surfaced { differ(&code[4..7], &soft[4..7]) } else { 0.0 }, if surfaced { differ(&code[8..15], &soft[8..15]) } else { 0.0 }];
            for k in 0..4 {
                if d[k] > worst[k].0 {
                    worst[k] = (d[k], p);
                }
            }
        }
        // How the colour's differences spread, by how much surface weighs the point.
        for (from, to) in [(1e-3f32, 0.05f32), (0.05, 0.25), (0.25, 10.0)] {
            let mut d: Vec<f32> = (0..points.len())
                .filter(|p| got[p * 32 + 3] > 0.5 && got[p * 32 + 16 + 3] > 0.5 && (from..to).contains(&got[p * 32 + 2]))
                .map(|p| (0..3).map(|c| (got[p * 32 + 4 + c] - got[p * 32 + 16 + 4 + c]).abs()).fold(0.0f32, f32::max))
                .collect();
            d.sort_by(f32::total_cmp);
            if !d.is_empty() {
                report += &format!("  surface {from} to {to}: {} points, the colour's difference median {:.5}, 99th {:.5}, largest {:.5}\n", d.len(), d[d.len() / 2], d[(d.len() * 99 / 100).min(d.len() - 1)], d[d.len() - 1]);
            }
        }
        let parts: Vec<String> = names.iter().zip(worst).map(|(n, (d, p))| format!("{n} {d:.5} (cells of {}, at {:.0} {:.0} {:.0})", 1 << points[p][3] as u32, points[p][0], points[p][1], points[p][2])).collect();
        report += &format!("the largest differences from the code's blend: {}; known where the code's is unknown, or the other way: {unknown}\n", parts.join(", "));
        eprintln!("Change 1's test, bricks of {side}, {} points at levels 32 to 1,024, {} bricks in the atlas ({:.0} MB):\n{report}", points.len(), atlas.bricks, atlas.bytes as f64 / 1e6);
        assert_eq!(unknown, 0, "the hardware's known and unknown are the code's");
        let solid_worst = worst[0].0.max(worst[1].0);
        assert!(solid_worst <= 1.0 / 200.0, "how solid or watery differs from the code's by {solid_worst}");
    }
}

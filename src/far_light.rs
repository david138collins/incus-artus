//! The far light, made on the processor (plan 2.2, "alongside": light read from the cell; the Oracle's change 3).
//!
//! The far eye's light used to be rays: after its march a pixel sent one toward the sun and five toward the sky, up to
//! 360 more steps on the card, 11 to 56% of the far eye's time (build 159). Here the light is worked out ahead for the
//! land itself, and a pixel only reads it where it lands.
//!
//! For light the far land is a surface: each column of each level has a top (how high its solid stands), and from
//! the tops come two numbers a column:
//! - **the height the sun's shadow reaches there**: one sweep across the level along the sun's way, each column taking
//!   the shadow of the columns toward the sun, dropped by the sun's slope (anything at or over it is sunlit);
//! - **how much sky its top sees**: from how high the land stands round it in eight directions (horizon maps: Max,
//!   1988), made once.
//!
//! Both go to the card as pictures of the whole world, every level side by side in each (`layout`): the shadow's
//! heights in 16-bit floats, kept twice so a new sweep is swapped in whole [David, 2026-11-13: "i think we should try
//! snapping light from one of 16 levels every time the sun has moved a 16th of the day, and smooth it only if its bad.
//! that snap may be pretty subtle and act as a little clock for the player"], and the sky seen in 8 bits.
//!
//! What it gives up: nothing in the far view shadows what's under itself (overhangs, arches), and a shadow's edge is
//! as soft as its level's cells.

use crate::atlas::Kept;
use crate::bricks::{Level, NOT_SUMMED};
use std::sync::Arc;
use std::time::Instant;

/// A shadow's height where nothing shades (under any ground).
const NO_SHADOW: f32 = -1000.0;
/// How many columns out the sky's horizon is looked for, each of the eight ways.
const SKY_REACH: i32 = 16;

/// Where each level's columns sit in the light's pictures, and the pictures' size: the first level at the left, the
/// others down a column to its right. A column's texel is its place in its level from there.
pub fn layout(levels: &[Level]) -> (Vec<[u32; 2]>, [u32; 2]) {
    let first = levels.first().map_or(1, |l| l.across as u32);
    let mut at = vec![[0, 0]];
    let (mut down, mut wide) = (0u32, 0u32);
    for level in levels.iter().skip(1) {
        at.push([first, down]);
        down += level.across as u32;
        wide = wide.max(level.across as u32);
    }
    (at, [first + wide, first.max(down)])
}

/// Every level's columns' tops (blocks; north across, then east), and where they sit in the pictures.
pub struct Tops {
    pub levels: Vec<Level>,
    pub tops: Vec<Vec<f32>>,
    pub at: Vec<[u32; 2]>,
    pub size: [u32; 2],
}

/// The tops, from the far cells kept as they came: a column's solid stood up from its band's foot (a column not
/// summed stands at nothing).
pub fn tops(kept: &Kept) -> Tops {
    let (at, size) = layout(&kept.levels);
    let tops = kept
        .levels
        .iter()
        .zip(&kept.columns)
        .map(|(level, columns)| {
            let cell = (1 << level.level) as f32;
            columns
                .iter()
                .map(|(entry, cells)| {
                    if *entry == NOT_SUMMED {
                        return 0.0;
                    }
                    let low = (*entry & 0xFFFF) as u16 as i16 as f32;
                    let solid: f32 = cells.iter().map(|c| c.solid as f32 / 255.0).sum();
                    ((low + solid) * cell).max(0.0)
                })
                .collect()
        })
        .collect();
    Tops { levels: kept.levels.clone(), tops, at, size }
}

/// The way to the sun from where it comes from and how high it stands (degrees), as the far eye has it.
pub fn sun_way(from: f32, height: f32) -> [f32; 3] {
    let (from, height) = (from.to_radians(), height.to_radians());
    [height.cos() * from.cos(), height.cos() * from.sin(), height.sin()]
}

/// One level's shadow heights for the sun at `sun` (the way to it): swept across the level from the sun's side, a line
/// of columns at a time across the way it leans most. Each column takes what stands toward the sun (the higher of
/// that column's top and its own shadow, between the two columns the sun's way passes), dropped by the sun's slope
/// over the step.
fn sweep(tops: &[f32], across: usize, cell: f32, sun: [f32; 3]) -> Vec<f32> {
    let mut heights = vec![NO_SHADOW; across * across];
    let flat = (sun[0] * sun[0] + sun[1] * sun[1]).sqrt();
    if sun[2] <= 0.0 || flat < 1e-4 {
        return heights;
    }
    let north_most = sun[0].abs() >= sun[1].abs();
    let (most, least) = if north_most { (sun[0], sun[1]) } else { (sun[1], sun[0]) };
    let lean = least / most.abs();
    let drop = cell * (1.0 + lean * lean).sqrt() * sun[2] / flat;
    let place = |line: usize, along: usize| if north_most { along * across + line } else { line * across + along };
    let toward: i64 = if most > 0.0 { 1 } else { -1 };
    for k in 1..across {
        let line = if toward > 0 { across - 1 - k } else { k };
        let before = (line as i64 + toward) as usize;
        for along in 0..across {
            let from = along as f32 + lean;
            let low = from.floor();
            let share = from - low;
            let stands = |a: i64| {
                if a < 0 || a >= across as i64 {
                    return NO_SHADOW;
                }
                let at = place(before, a as usize);
                heights[at].max(tops[at])
            };
            let taken = stands(low as i64) * (1.0 - share) + stands(low as i64 + 1) * share;
            heights[place(line, along)] = (taken - drop).max(NO_SHADOW);
        }
    }
    heights
}

/// A 32-bit float as a 16-bit one (heights: whole and half blocks are exact to 2,048).
fn half(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xFF) as i32 - 127 + 15;
    let mantissa = bits & 0x7F_FFFF;
    if exponent >= 0x1F {
        return sign | 0x7BFF;
    }
    if exponent <= 0 {
        return sign;
    }
    sign | (((exponent as u32) << 10) + (mantissa >> 13)) as u16
}

/// The shadow's heights for every level, as the card's picture (16-bit floats, a row after another).
pub fn shadow(tops: &Tops, sun: [f32; 3]) -> Vec<u8> {
    let (wide, tall) = (tops.size[0] as usize, tops.size[1] as usize);
    let mut picture = vec![half(NO_SHADOW); wide * tall];
    for ((level, level_tops), at) in tops.levels.iter().zip(&tops.tops).zip(&tops.at) {
        let across = level.across as usize;
        let heights = sweep(level_tops, across, (1 << level.level) as f32, sun);
        for east in 0..across {
            let row = (at[1] as usize + east) * wide + at[0] as usize;
            for north in 0..across {
                picture[row + north] = half(heights[east * across + north]);
            }
        }
    }
    picture.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// How much sky a column's top sees, 0 to 1: over eight ways round, one less the sine of how high the land stands
/// that way (the most over `SKY_REACH` columns out).
fn sky_seen(tops: &[f32], across: i32, cell: f32, north: i32, east: i32) -> f32 {
    const WAYS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
    let here = tops[(east * across + north) as usize];
    let mut seen = 0.0;
    for (dn, de) in WAYS {
        let step = cell * ((dn * dn + de * de) as f32).sqrt();
        let mut slope = 0.0f32;
        for k in 1..=SKY_REACH {
            let (n, e) = (north + dn * k, east + de * k);
            if n < 0 || e < 0 || n >= across || e >= across {
                break;
            }
            slope = slope.max((tops[(e * across + n) as usize] - here) / (step * k as f32));
        }
        seen += 1.0 - slope / (1.0 + slope * slope).sqrt();
    }
    seen / WAYS.len() as f32
}

/// The sky seen for every level, as the card's picture (a byte a column), on as many threads as are free.
pub fn sky(tops: &Tops) -> Vec<u8> {
    let (wide, tall) = (tops.size[0] as usize, tops.size[1] as usize);
    let mut picture = vec![255u8; wide * tall];
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(2).clamp(1, 16);
    for ((level, level_tops), at) in tops.levels.iter().zip(&tops.tops).zip(&tops.at) {
        let across = level.across as usize;
        let cell = (1 << level.level) as f32;
        let mut seen = vec![0u8; across * across];
        let rows = across.div_ceil(threads).max(1);
        std::thread::scope(|scope| {
            for (k, part) in seen.chunks_mut(rows * across).enumerate() {
                scope.spawn(move || {
                    for (i, out) in part.iter_mut().enumerate() {
                        let (east, north) = (k * rows + i / across, i % across);
                        *out = (sky_seen(level_tops, across as i32, cell, north as i32, east as i32) * 255.0).round() as u8;
                    }
                });
            }
        });
        for east in 0..across {
            let row = (at[1] as usize + east) * wide + at[0] as usize;
            picture[row..row + across].copy_from_slice(&seen[east * across..(east + 1) * across]);
        }
    }
    picture
}

/// The far light on the card: the shadow's heights twice (one shown, one for the next sweep) and the sky seen.
pub struct Light {
    pub tops: Arc<Tops>,
    shadows: [wgpu::Texture; 2],
    pub shadow_views: [wgpu::TextureView; 2],
    pub sky: wgpu::TextureView,
    /// Which of the two shadows is shown, the sun's setting it was swept for (from, height: degrees), and a count
    /// that goes up each time another is shown.
    pub shown: usize,
    pub sun: [f32; 2],
    pub generation: u32,
    /// How long the tops and the sky took to make, and the last sweep with its sending (seconds).
    pub made_in: f32,
    pub swept_in: f32,
    pub bytes: u64,
}

fn texture(device: &wgpu::Device, label: &str, size: [u32; 2], format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn send(queue: &wgpu::Queue, texture: &wgpu::Texture, size: [u32; 2], texel: u32, bytes: &[u8]) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        bytes,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size[0] * texel), rows_per_image: Some(size[1]) },
        wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
    );
    queue.submit([]);
}

/// A texture of one texel, to stand where the light's pictures will be till they're made.
pub fn stand_in(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::TextureView {
    texture(device, "no far light yet", [1, 1], format).create_view(&wgpu::TextureViewDescriptor::default())
}

/// The far light made from the cells kept (every region in), for the sun's setting `sun` (from, height), and sent to
/// the card. Run on a thread of its own.
pub fn make(device: &wgpu::Device, queue: &wgpu::Queue, kept: &Kept, sun: [f32; 2]) -> Light {
    let started = Instant::now();
    let tops = Arc::new(tops(kept));
    let sky_bytes = sky(&tops);
    let made_in = started.elapsed().as_secs_f32();
    let swept = Instant::now();
    let shadow_bytes = shadow(&tops, sun_way(sun[0], sun[1]));
    let shadows = [0, 1].map(|_| texture(device, "the far shadow's heights", tops.size, wgpu::TextureFormat::R16Float));
    let sky_texture = texture(device, "the far sky seen", tops.size, wgpu::TextureFormat::R8Unorm);
    send(queue, &shadows[0], tops.size, 2, &shadow_bytes);
    send(queue, &sky_texture, tops.size, 1, &sky_bytes);
    let view = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
    let bytes = (tops.size[0] * tops.size[1]) as u64 * 5;
    Light { shadow_views: [view(&shadows[0]), view(&shadows[1])], sky: view(&sky_texture), shadows, tops, shown: 0, sun, generation: 0, made_in, swept_in: swept.elapsed().as_secs_f32(), bytes }
}

impl Light {
    /// What a thread needs to sweep the shadow that isn't shown for another setting of the sun: `swept` shows it.
    pub fn sweeper(&self) -> (Arc<Tops>, wgpu::Texture) {
        (self.tops.clone(), self.shadows[1 - self.shown].clone())
    }

    /// The other shadow, swept for `sun` in `seconds`, shown from now: the snap.
    pub fn swept(&mut self, sun: [f32; 2], seconds: f32) {
        self.shown = 1 - self.shown;
        self.sun = sun;
        self.swept_in = seconds;
        self.generation += 1;
    }
}

/// A shadow swept for the sun's setting `sun` and sent into `into` (the one not shown). Run on a thread of its own;
/// how long it took (seconds).
pub fn sweep_into(queue: &wgpu::Queue, tops: &Tops, into: &wgpu::Texture, sun: [f32; 2]) -> f32 {
    let started = Instant::now();
    send(queue, into, tops.size, 2, &shadow(tops, sun_way(sun[0], sun[1])));
    started.elapsed().as_secs_f32()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wall's shadow falls away from the sun and as long as the sun's slope makes it: with the sun in the north at
    /// 45 degrees, a wall 320 blocks over flat ground at 32-block cells shades the ground south of it for about ten
    /// columns, and nothing north of it.
    #[test]
    fn a_walls_shadow_falls_away_from_the_sun() {
        let across = 64;
        let mut tops = vec![0.0f32; across * across];
        for east in 0..across {
            tops[east * across + 40] = 320.0;
        }
        let heights = sweep(&tops, across, 32.0, sun_way(0.0, 45.0));
        let at = |north: usize| heights[20 * across + north];
        assert!(at(45) <= 0.0, "north of the wall, toward the sun, nothing shades: {}", at(45));
        assert!((at(39) - 288.0).abs() < 1.0, "the first column south is shaded to a cell under the wall's top: {}", at(39));
        assert!(at(31) > 0.0 && at(29) <= 0.0, "the shadow ends ten columns south: {} then {}", at(31), at(29));
        // From the east: the shadow falls west instead, and a wall running east to west shades nothing but itself.
        let heights = sweep(&tops, across, 32.0, sun_way(90.0, 45.0));
        assert!(heights[20 * across + 39] <= 0.0 && heights[20 * across + 41] <= 0.0, "a wall along the sun's way shades neither side");
    }

    /// Open flat land sees the whole sky; the floor of a deep pit sees little of it.
    #[test]
    fn a_pit_sees_less_sky_than_a_plain() {
        let across = 32;
        let mut tops = vec![640.0f32; across * across];
        assert!(sky_seen(&tops, across as i32, 32.0, 16, 16) > 0.99);
        tops[16 * across + 16] = 0.0;
        let pit = sky_seen(&tops, across as i32, 32.0, 16, 16);
        assert!(pit < 0.1, "a pit twenty cells deep sees almost no sky: {pit}");
    }

    /// The pictures hold every level without one over another.
    #[test]
    fn the_levels_sit_side_by_side_in_the_pictures() {
        let levels = crate::bricks::levels(([-36, -36], 72), 1024, 8);
        let (at, size) = layout(&levels);
        for (a, la) in levels.iter().enumerate() {
            assert!(at[a][0] + la.across as u32 <= size[0] && at[a][1] + la.across as u32 <= size[1], "level {} is inside the picture", la.level);
            for (b, lb) in levels.iter().enumerate().skip(a + 1) {
                let apart = at[a][0] + la.across as u32 <= at[b][0] || at[b][0] + lb.across as u32 <= at[a][0] || at[a][1] + la.across as u32 <= at[b][1] || at[b][1] + lb.across as u32 <= at[a][1];
                assert!(apart, "levels {} and {} don't overlap", la.level, lb.level);
            }
        }
    }
}

//! Sky light — how much daylight reaches each cell [`docs/caves.md`, stop 3 and section 11; David,
//! 2026-11-01: "i want them to be pitch black unless something is making light down there"].
//!
//! - **Open to the sky:** a cell with nothing solid above it, all the way up, is fully lit (15).
//! - **A short way in:** the light spreads from there through cells that aren't solid (round corners,
//!   down tunnels), one even step darker each block, to black at `reach` blocks. So it reaches a little
//!   way into a cave's mouth, under an overhang or down a hole you dig, and everything further in is
//!   pitch black. Nothing floods through a cave system, as Minecraft's sky light does.
//! - **Whole numbers, and bounded:** a walk of at most `reach` steps from the open cells near a
//!   section.
//!
//! It's worked out when a section is meshed, and each face takes the light of the cell in front of
//! it, stored in 4 bits of the vertex (`mesher.rs`): 16 levels. Most sections need none of this:
//! every cell in and round them is either solid or open to the sky, so everything is full. Only
//! sections with covered air near them (caves, overhangs, digging) get the walk.
//!
//! **The format is settled for colour too** (Counsel, 2026-11-01): this is the sky's channel, its
//! colour coming from the sky (day, the moons). Placed lights (a brazier, lichen) will be their own
//! channel, brightness and colour, in a third word of the vertex, so colour never redoes this.

use crate::world::{BlockId, CHUNK_SIZE, Chunk, SECTION_SIZE, Section};
use bevy::math::IVec3;
use std::collections::VecDeque;

/// Full daylight.
pub const FULL: u8 = 15;
/// The farthest daylight may reach in (blocks).
pub const MOST_REACH: i32 = 15;

/// The cells round a section the light is worked out from: the section and `margin` blocks round it
/// each way (a cube `side` across, x fastest), each solid (it stops light) or not, and open to the sky
/// or not.
pub struct LightRegion {
    pub margin: i32,
    pub side: i32,
    pub opaque: Vec<bool>,
    pub open: Vec<bool>,
}

impl LightRegion {
    /// An empty region (every cell solid) `margin` blocks round a section.
    pub fn new(margin: i32) -> Self {
        let side = SECTION_SIZE + 2 * margin;
        let cells = (side * side * side) as usize;
        Self { margin, side, opaque: vec![true; cells], open: vec![false; cells] }
    }

    /// A cell's place in the lists, from its place relative to the section (−margin to 31 + margin).
    pub fn index(&self, local: IVec3) -> usize {
        let p = local + IVec3::splat(self.margin);
        (p.x + p.y * self.side + p.z * self.side * self.side) as usize
    }
}

/// The light of a section and the one block round it (34³, x fastest, from −1): what the mesher reads.
/// `None`: full daylight everywhere.
#[derive(Default)]
pub struct Daylight(pub Option<Vec<u8>>);

/// Cells across the section and its one-block border.
const AROUND: i32 = SECTION_SIZE + 2;

impl Daylight {
    /// The light (0 to 15) at a cell, from its place relative to the section (−1 to 32).
    pub fn at(&self, x: i32, y: i32, z: i32) -> u32 {
        match &self.0 {
            None => FULL as u32,
            Some(levels) => levels[((x + 1) + (y + 1) * AROUND + (z + 1) * AROUND * AROUND) as usize] as u32,
        }
    }
}

/// Daylight through the region: full where open to the sky, an even step darker for each block away
/// through cells that aren't solid, black from `reach` blocks on. The section and its one-block
/// border, for the mesher.
pub fn flood(region: &LightRegion, reach: i32) -> Daylight {
    let reach = reach.clamp(1, MOST_REACH);
    let side = region.side;
    let mut steps = vec![u8::MAX; region.opaque.len()];
    let mut queue = VecDeque::new();
    for (i, &open) in region.open.iter().enumerate() {
        if open {
            steps[i] = 0;
            queue.push_back(i as i32);
        }
    }
    let strides = [1, side, side * side];
    while let Some(i) = queue.pop_front() {
        let here = steps[i as usize];
        if here as i32 >= reach {
            continue;
        }
        let p = [i % side, (i / side) % side, i / (side * side)];
        for axis in 0..3 {
            for way in [-1, 1] {
                let along = p[axis] + way;
                if !(0..side).contains(&along) {
                    continue;
                }
                let next = (i + way * strides[axis]) as usize;
                if !region.opaque[next] && steps[next] > here + 1 {
                    steps[next] = here + 1;
                    queue.push_back(next as i32);
                }
            }
        }
    }
    let mut levels = vec![0u8; (AROUND * AROUND * AROUND) as usize];
    for z in -1..SECTION_SIZE + 1 {
        for y in -1..SECTION_SIZE + 1 {
            for x in -1..SECTION_SIZE + 1 {
                let s = steps[region.index(IVec3::new(x, y, z))] as i32;
                let level = if s > reach { 0 } else { FULL as i32 * (reach - s) / reach };
                levels[((x + 1) + (y + 1) * AROUND + (z + 1) * AROUND * AROUND) as usize] = level as u8;
            }
        }
    }
    Daylight(Some(levels))
}

/// Each column's sky height in a chunk: the highest solid block in it (below the world's bottom, −1,
/// if none), `x + 32 y` from the chunk's south-west corner. Cells above it are open to the sky.
pub fn sky_heights(chunk: &Chunk, is_opaque: &dyn Fn(BlockId) -> bool) -> Vec<i32> {
    let origin = chunk.pos().min_block();
    let mut heights = vec![-1; (CHUNK_SIZE * CHUNK_SIZE) as usize];
    for (column, height) in heights.iter_mut().enumerate() {
        let (x, y) = (column as i32 % CHUNK_SIZE, column as i32 / CHUNK_SIZE);
        'down: for (level, section) in chunk.sections().iter().enumerate().rev() {
            match section {
                Section::Uniform(id) if !is_opaque(*id) => continue,
                Section::Uniform(_) => {
                    *height = (level as i32 + 1) * SECTION_SIZE - 1;
                    break 'down;
                }
                _ => {
                    for z in (level as i32 * SECTION_SIZE..(level as i32 + 1) * SECTION_SIZE).rev() {
                        if is_opaque(chunk.get(origin + IVec3::new(x, y, z))) {
                            *height = z;
                            break 'down;
                        }
                    }
                }
            }
        }
    }
    heights
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A region `margin` round a section: solid below Z 16, open air above; a tunnel at Z 10 along x
    /// from a shaft at x `shaft` (open to the sky) to x 20, one cell across; and a pocket of air at
    /// (5, 5, 14), walled off from the open air by one block.
    fn tunnel(margin: i32, shaft: i32) -> LightRegion {
        let mut region = LightRegion::new(margin);
        for z in -margin..SECTION_SIZE + margin {
            for y in -margin..SECTION_SIZE + margin {
                for x in -margin..SECTION_SIZE + margin {
                    let i = region.index(IVec3::new(x, y, z));
                    let tunnel = z == 10 && y == 5 && (shaft..=20).contains(&x);
                    let air = z >= 16 || tunnel || (x == shaft && y == 5 && z >= 10) || (x, y, z) == (5, 5, 14);
                    region.opaque[i] = !air;
                    region.open[i] = z >= 16 || (x == shaft && y == 5 && z >= 10);
                }
            }
        }
        region
    }

    #[test]
    fn daylight_reaches_a_little_way_into_a_tunnel_and_it_s_black_beyond() {
        // A shaft 9 blocks before the section: 8 blocks in, at x −1, the light's gone.
        let reach = 8;
        let light = flood(&tunnel(reach + 1, -9), reach);
        assert_eq!(light.at(5, 5, 20), 15, "open air: full");
        for x in -1..=20 {
            assert_eq!(light.at(x, 5, 10), 0, "x {x}: pitch black, right to its end");
        }
        // The light doesn't go through rock: a pocket right below the open air, walled off.
        assert_eq!(light.at(5, 5, 14), 0);
        // A shaft at x −3: full in it, then a step darker each block along the tunnel.
        let light = flood(&tunnel(reach + 1, -3), reach);
        let levels: Vec<u32> = (-1..7).map(|x| light.at(x, 5, 10)).collect();
        assert_eq!(levels, vec![11, 9, 7, 5, 3, 1, 0, 0]);
        // No light worked out: full everywhere.
        assert_eq!(Daylight::default().at(3, 3, 3), 15);
    }
}

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
//! The far copies work it out for their cells (`far_copies.rs`), and the objects for theirs
//! (`objects.rs`), from each column's sky height here. (Before R7 the near mesher walked it for each
//! section with covered air near it, `flood`; that went with the old engine.)
//!
//! **The format is settled for colour too** (Counsel, 2026-11-01): this is the sky's channel, its
//! colour coming from the sky (day, the moons). Placed lights (a brazier, lichen) will be their own
//! channel, brightness and colour, in a third word of the vertex, so colour never redoes this.

use crate::world::{BlockId, CHUNK_SIZE, Chunk, SECTION_SIZE, Section};
use bevy::math::IVec3;

/// Full daylight.
pub const FULL: u8 = 15;
/// The farthest daylight may reach in (blocks).
pub const MOST_REACH: i32 = 15;

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


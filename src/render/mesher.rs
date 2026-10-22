//! Mesher — turns one 32³ section of blocks into rectangles ("quads") to draw.
//!
//! This is *binary greedy meshing*. Three stages:
//!
//! 1. **Solid columns.** For each axis, every line of 32 blocks through the section becomes one
//!    64-bit number: bit N is 1 if the Nth block is opaque. Two extra bits hold the blocks just
//!    outside the section (from neighbouring sections), so faces on the border are judged correctly.
//!
//! 2. **Visible faces.** A face is visible where a solid block sits next to a non-solid one.
//!    With bits, that's one line per column: `column & !(column >> 1)` finds every solid bit
//!    whose next bit is empty: 32 blocks checked in a single CPU instruction.
//!
//! 3. **Greedy merging.** Visible faces are sorted into flat 32 × 32 "planes" (one per depth and
//!    block type). In each plane, runs of set bits are grown into the largest rectangles possible,
//!    so a flat grass floor becomes one quad instead of 1,024.
//!
//! Positions use the game's axes (X north, Y east, Z up), measured inside the section (0 ..= 32).
//! The shader converts them for drawing.
//!
//! ## Vertex format (8 bytes per corner)
//! - word 0: x (6 bits) | y (6 bits) | z (6 bits) | face direction (3 bits)
//! - word 1: texture layer (16 bits) | quad number (16 bits, only used by the debug view)

use crate::coords::game_to_bevy;
use crate::world::BlockId;
use bevy::math::Vec3;
use std::collections::HashMap;
use std::sync::Arc;

const N: usize = 32;

/// Everything the mesher needs, copied out of the world so it can run on another thread.
pub struct MeshInput {
    /// The section's blocks, indexed `x + y * 32 + z * 1024`.
    pub blocks: Vec<BlockId>,
    /// For each face direction, which blocks just outside the section on that side are opaque.
    /// One `u32` per row: row = the face plane's `v` axis, bit = its `u` axis (see `plane_axes`).
    pub outside: [[u32; N]; 6],
    /// Per block number: does it hide what's behind it?
    pub opaque: Arc<Vec<bool>>,
    /// Per block number: texture layer for each of the six faces.
    pub face_layers: Arc<Vec<[u32; 6]>>,
}

#[derive(Default)]
pub struct MeshOutput {
    pub vertices: Vec<[u32; 2]>,
    pub indices: Vec<u32>,
    pub quads: u32,
}

/// The two axes lying in a face plane, for faces perpendicular to `axis` (0 = X, 1 = Y, 2 = Z).
pub fn plane_axes(axis: usize) -> (usize, usize) {
    ((axis + 1) % 3, (axis + 2) % 3)
}

/// Face direction number: 0 north, 1 south, 2 east, 3 west, 4 top, 5 bottom.
pub fn face_index(axis: usize, positive: bool) -> usize {
    axis * 2 + if positive { 0 } else { 1 }
}

pub fn mesh_section(input: &MeshInput) -> MeshOutput {
    let is_opaque = |id: BlockId| input.opaque.get(id.0 as usize).copied().unwrap_or(true);
    let mut out = MeshOutput::default();

    // ── 1. Solid columns, one set per axis ──
    // Bit 0 = the block just before the section, bits 1..=32 = the section, bit 33 = just after.
    let mut columns = vec![[0u64; N * N]; 3];
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                if is_opaque(input.blocks[x + y * N + z * N * N]) {
                    let p = [x, y, z];
                    for (axis, axis_columns) in columns.iter_mut().enumerate() {
                        let (u, v) = plane_axes(axis);
                        axis_columns[p[u] + p[v] * N] |= 1u64 << (p[axis] + 1);
                    }
                }
            }
        }
    }
    for (axis, axis_columns) in columns.iter_mut().enumerate() {
        let before = &input.outside[face_index(axis, false)];
        let after = &input.outside[face_index(axis, true)];
        for v in 0..N {
            for u in 0..N {
                let column = &mut axis_columns[u + v * N];
                if (before[v] >> u) & 1 == 1 {
                    *column |= 1;
                }
                if (after[v] >> u) & 1 == 1 {
                    *column |= 1u64 << (N + 1);
                }
            }
        }
    }

    // ── 2 & 3. Visible faces, sorted into planes, then merged ──
    for axis in 0..3 {
        let (u_axis, v_axis) = plane_axes(axis);
        for positive in [true, false] {
            let face = face_index(axis, positive);

            // (depth, block) → 32 rows of bits
            let mut planes: HashMap<(u32, BlockId), [u32; N]> = HashMap::new();
            for v in 0..N {
                for u in 0..N {
                    let column = columns[axis][u + v * N];
                    let faces = if positive {
                        column & !(column >> 1) // solid here, empty after
                    } else {
                        column & !(column << 1) // solid here, empty before
                    };
                    let mut bits = (faces >> 1) & 0xFFFF_FFFF; // drop the outside bits
                    while bits != 0 {
                        let depth = bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        let mut p = [0; 3];
                        p[axis] = depth;
                        p[u_axis] = u;
                        p[v_axis] = v;
                        let block = input.blocks[p[0] + p[1] * N + p[2] * N * N];
                        planes.entry((depth as u32, block)).or_insert([0; N])[v] |= 1 << u;
                    }
                }
            }

            let flip = needs_flip(axis, positive);
            for ((depth, block), mut plane) in planes {
                let layer = input
                    .face_layers
                    .get(block.0 as usize)
                    .map_or(0, |layers| layers[face]);
                // The face of a block at `depth` lies on its far side when facing the positive way.
                let plane_position = depth + positive as u32;
                merge_plane(&mut plane, |u, v, width, height| {
                    let corners = [(u, v), (u + width, v), (u + width, v + height), (u, v + height)];
                    let base = out.vertices.len() as u32;
                    let quad_tag = out.quads & 0xFFFF;
                    for (cu, cv) in corners {
                        let mut p = [0u32; 3];
                        p[axis] = plane_position;
                        p[u_axis] = cu;
                        p[v_axis] = cv;
                        out.vertices.push([
                            p[0] | p[1] << 6 | p[2] << 12 | (face as u32) << 18,
                            (layer & 0xFFFF) | quad_tag << 16,
                        ]);
                    }
                    let order: [u32; 6] = if flip { [0, 2, 1, 0, 3, 2] } else { [0, 1, 2, 0, 2, 3] };
                    out.indices.extend(order.iter().map(|i| base + i));
                    out.quads += 1;
                });
            }
        }
    }
    out
}

/// Grow runs of set bits into the largest rectangles possible, clearing them as it goes.
/// Calls `emit(u, v, width, height)` for each rectangle.
fn merge_plane(plane: &mut [u32; N], mut emit: impl FnMut(u32, u32, u32, u32)) {
    for v in 0..N {
        while plane[v] != 0 {
            let u = plane[v].trailing_zeros();
            let width = (plane[v] >> u).trailing_ones();
            let mask = if width == 32 {
                u32::MAX
            } else {
                ((1u32 << width) - 1) << u
            };
            plane[v] &= !mask;
            let mut height = 1;
            while v + height < N && plane[v + height] & mask == mask {
                plane[v + height] &= !mask;
                height += 1;
            }
            emit(u, v as u32, width, height as u32);
        }
    }
}

/// The GPU only draws the *front* of a triangle (front = corners go counter-clockwise as you
/// look at it). Because the game's axes are mirrored relative to Bevy's, the natural corner
/// order comes out backwards for some face directions. This works out which ones.
fn needs_flip(axis: usize, positive: bool) -> bool {
    let (u_axis, v_axis) = plane_axes(axis);
    let a = Vec3::ZERO;
    let mut b = Vec3::ZERO;
    b[u_axis] = 1.0;
    let mut c = b;
    c[v_axis] = 1.0;
    let mut outward = Vec3::ZERO;
    outward[axis] = if positive { 1.0 } else { -1.0 };
    let (a, b, c) = (game_to_bevy(a), game_to_bevy(b), game_to_bevy(c));
    (b - a).cross(c - a).dot(game_to_bevy(outward)) < 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const STONE: BlockId = BlockId(1);
    const DIRT: BlockId = BlockId(2);

    fn input(blocks: Vec<BlockId>) -> MeshInput {
        MeshInput {
            blocks,
            outside: [[0; N]; 6],
            opaque: Arc::new(vec![false, true, true]),
            face_layers: Arc::new(vec![[0; 6], [1; 6], [2; 6]]),
        }
    }

    fn empty() -> Vec<BlockId> {
        vec![BlockId::AIR; N * N * N]
    }

    #[test]
    fn single_block_has_six_faces() {
        let mut blocks = empty();
        blocks[5 + 5 * N + 5 * N * N] = STONE;
        let mesh = mesh_section(&input(blocks));
        assert_eq!(mesh.quads, 6);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn solid_section_merges_to_six_quads() {
        let mesh = mesh_section(&input(vec![STONE; N * N * N]));
        assert_eq!(mesh.quads, 6);
    }

    #[test]
    fn buried_section_has_no_faces() {
        let mut data = input(vec![STONE; N * N * N]);
        data.outside = [[u32::MAX; N]; 6];
        assert_eq!(mesh_section(&data).quads, 0);
    }

    #[test]
    fn same_blocks_merge_different_blocks_do_not() {
        let mut same = empty();
        same[0] = STONE;
        same[1] = STONE;
        assert_eq!(mesh_section(&input(same)).quads, 6);

        let mut different = empty();
        different[0] = STONE;
        different[1] = DIRT;
        assert_eq!(mesh_section(&input(different)).quads, 10);
    }

    /// Every triangle must face outward (counter-clockwise seen from outside, in Bevy space).
    #[test]
    fn triangles_face_outward() {
        let mut blocks = empty();
        blocks[5 + 5 * N + 5 * N * N] = STONE;
        let mesh = mesh_section(&input(blocks));
        let position = |i: u32| {
            let w = mesh.vertices[i as usize][0];
            game_to_bevy(Vec3::new(
                (w & 63) as f32,
                ((w >> 6) & 63) as f32,
                ((w >> 12) & 63) as f32,
            ))
        };
        let block_centre = game_to_bevy(Vec3::splat(5.5));
        for triangle in mesh.indices.chunks(3) {
            let (a, b, c) = (position(triangle[0]), position(triangle[1]), position(triangle[2]));
            let normal = (b - a).cross(c - a);
            let middle = (a + b + c) / 3.0;
            assert!(normal.dot(middle - block_centre) > 0.0, "a triangle faces inward");
        }
    }
}

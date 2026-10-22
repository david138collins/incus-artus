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
//! See-through blocks get their own runs of the same stages, with each face drawn twice (once
//! facing each way) so it can be seen from inside too. Opaque faces behind a see-through block
//! are still drawn, because the see-through block doesn't hide them.
//! - **Outline** blocks: faces only where they touch something neither opaque nor see-through,
//!   so a volume of them shows just its outside skin.
//! - **Grid** blocks: faces everywhere except against opaque blocks, so every cell shows. Where
//!   two grid blocks touch, only one of them draws the shared face (never two on top of each other).
//!
//! Positions use the game's axes (X north, Y east, Z up), measured inside the section (0 ..= 32).
//! The shader converts them for drawing.
//!
//! ## Vertex format (8 bytes per corner)
//! - word 0: x (6 bits) | y (6 bits) | z (6 bits) | face direction (3 bits) | random rotation (1 bit)
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
    /// Same layout as `outside`: which of those blocks are outline blocks.
    pub outside_outline: [[u32; N]; 6],
    /// Same layout as `outside`: which of those blocks are grid blocks.
    pub outside_grid: [[u32; N]; 6],
    /// Per block number: does it hide what's behind it?
    pub opaque: Arc<Vec<bool>>,
    /// Per block number: is it see-through (drawn both sides, with holes)?
    pub see_through: Arc<Vec<bool>>,
    /// Per block number: if see-through, does it draw faces between neighbours (grid)?
    pub inner_faces: Arc<Vec<bool>>,
    /// Per block number: texture layer for each of the six faces.
    pub face_layers: Arc<Vec<[u32; 6]>>,
    /// Per block number: is each face's texture randomly rotated?
    pub face_random_rotation: Arc<Vec<[bool; 6]>>,
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

/// One bit column per line of blocks, for each of the three axes.
type Columns = Vec<[u64; N * N]>;

pub fn mesh_section(input: &MeshInput) -> MeshOutput {
    let is_opaque = |id: BlockId| input.opaque.get(id.0 as usize).copied().unwrap_or(true);
    let is_see_through = |id: BlockId| input.see_through.get(id.0 as usize).copied().unwrap_or(false);
    let is_grid = |id: BlockId| is_see_through(id) && input.inner_faces.get(id.0 as usize).copied().unwrap_or(false);
    let is_outline = |id: BlockId| is_see_through(id) && !is_grid(id);
    let mut out = MeshOutput::default();

    // Opaque blocks: faces where they meet anything not opaque.
    let opaque = build_columns(&input.blocks, is_opaque, &input.outside);
    emit_faces(&mut out, input, &opaque, [&opaque, &opaque], false);

    if !input.blocks.iter().any(|&id| is_see_through(id)) {
        return out;
    }
    let grid = build_columns(&input.blocks, is_grid, &input.outside_grid);
    let outline = build_columns(&input.blocks, is_outline, &input.outside_outline);
    let union = |a: &Columns, b: &Columns, c: &Columns| -> Columns {
        (0..3)
            .map(|axis| std::array::from_fn(|i| a[axis][i] | b[axis][i] | c[axis][i]))
            .collect()
    };

    // Outline blocks: faces where they meet anything neither opaque nor see-through.
    let anything = union(&opaque, &grid, &outline);
    emit_faces(&mut out, input, &outline, [&anything, &anything], true);

    // Grid blocks: faces everywhere except against opaque blocks. A face shared by two grid
    // blocks is drawn only once: by the block on its negative side, as that block's positive face.
    let opaque_or_grid = union(&opaque, &grid, &grid);
    emit_faces(&mut out, input, &grid, [&opaque, &opaque_or_grid], true);
    out
}

/// Stage 1: the bit columns of every block that passes `include`.
/// Bit 0 = the block just before the section, bits 1..=32 = the section, bit 33 = just after.
fn build_columns(blocks: &[BlockId], include: impl Fn(BlockId) -> bool, outside: &[[u32; N]; 6]) -> Columns {
    let mut columns = vec![[0u64; N * N]; 3];
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                if include(blocks[x + y * N + z * N * N]) {
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
        let before = &outside[face_index(axis, false)];
        let after = &outside[face_index(axis, true)];
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
    columns
}

/// Stages 2 and 3: a face shows where a block in `columns` has no blocker next to it.
/// `blockers` = [what blocks positive-facing faces, what blocks negative-facing faces].
/// Faces are merged into rectangles and added to `out`. `both_sides` also adds each
/// rectangle facing the other way (for see-through blocks).
fn emit_faces(out: &mut MeshOutput, input: &MeshInput, columns: &Columns, blockers: [&Columns; 2], both_sides: bool) {
    for axis in 0..3 {
        let (u_axis, v_axis) = plane_axes(axis);
        for positive in [true, false] {
            let face = face_index(axis, positive);

            // (depth, block) → 32 rows of bits
            let mut planes: HashMap<(u32, BlockId), [u32; N]> = HashMap::new();
            for v in 0..N {
                for u in 0..N {
                    let column = columns[axis][u + v * N];
                    let blocking = blockers[if positive { 0 } else { 1 }][axis][u + v * N];
                    let faces = if positive {
                        column & !(blocking >> 1) // a block here, nothing blocking after
                    } else {
                        column & !(blocking << 1) // a block here, nothing blocking before
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
                let random_rotation = input
                    .face_random_rotation
                    .get(block.0 as usize)
                    .is_some_and(|faces| faces[face]);
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
                            p[0] | p[1] << 6 | p[2] << 12 | (face as u32) << 18 | (random_rotation as u32) << 21,
                            (layer & 0xFFFF) | quad_tag << 16,
                        ]);
                    }
                    let front: [u32; 6] = [0, 1, 2, 0, 2, 3];
                    let back: [u32; 6] = [0, 2, 1, 0, 3, 2];
                    let (outward, inward) = if flip { (back, front) } else { (front, back) };
                    out.indices.extend(outward.iter().map(|i| base + i));
                    if both_sides {
                        out.indices.extend(inward.iter().map(|i| base + i));
                    }
                    out.quads += 1;
                });
            }
        }
    }
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
            outside_outline: [[0; N]; 6],
            outside_grid: [[0; N]; 6],
            opaque: Arc::new(vec![false, true, true, false, false]),
            see_through: Arc::new(vec![false, false, false, true, true]),
            inner_faces: Arc::new(vec![false, false, false, true, false]),
            face_layers: Arc::new(vec![[0; 6], [1; 6], [2; 6], [3; 6], [3; 6]]),
            face_random_rotation: Arc::new(vec![[false; 6]; 5]),
        }
    }

    const GRID: BlockId = BlockId(3);
    const OUTLINE: BlockId = BlockId(4);

    #[test]
    fn outline_blocks_side_by_side_show_only_their_outside() {
        let mut blocks = empty();
        blocks[0] = OUTLINE;
        blocks[1] = OUTLINE;
        assert_eq!(mesh_section(&input(blocks)).quads, 6, "one merged box, no face between");
    }

    #[test]
    fn grid_blocks_side_by_side_show_the_face_between_once() {
        let mut blocks = empty();
        blocks[0] = GRID;
        blocks[1] = GRID;
        // Two cells in a row along X: 2 end faces, plus the 4 long sides (merged), plus the
        // one shared face in the middle, drawn once.
        assert_eq!(mesh_section(&input(blocks)).quads, 7);
    }

    #[test]
    fn see_through_block_is_drawn_from_both_sides() {
        let mut blocks = empty();
        blocks[5 + 5 * N + 5 * N * N] = GRID;
        let mesh = mesh_section(&input(blocks));
        assert_eq!(mesh.quads, 6);
        assert_eq!(mesh.indices.len(), 6 * 12, "two triangles each way per face");
    }

    #[test]
    fn opaque_block_behind_see_through_still_shows() {
        let mut blocks = empty();
        blocks[0] = STONE;
        blocks[1] = GRID;
        // Stone: all 6 faces (the grid doesn't hide it). Grid: 5 (the side against the stone is hidden).
        assert_eq!(mesh_section(&input(blocks)).quads, 11);
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

    #[test]
    fn only_faces_marked_for_it_get_random_rotation() {
        let mut blocks = empty();
        blocks[5 + 5 * N + 5 * N * N] = STONE;
        let mut data = input(blocks);
        let mut stone = [false; 6];
        stone[4] = true; // the top only
        data.face_random_rotation = Arc::new(vec![[false; 6], stone, [false; 6]]);
        let mesh = mesh_section(&data);
        for vertex in &mesh.vertices {
            let face = (vertex[0] >> 18) & 7;
            let rotates = (vertex[0] >> 21) & 1 == 1;
            assert_eq!(rotates, face == 4);
        }
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

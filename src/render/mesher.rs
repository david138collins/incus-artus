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
//! Positions use the game's axes (X north, Y east, Z up), measured inside the section in half blocks
//! (0 ..= 64), so the pieces of a cell (bars and octants, `world/pieces.rs`) sit at half-block corners.
//! The shader converts them for drawing.
//!
//! ## Vertex format (8 bytes per corner)
//! - word 0: x (7 bits) | y (7 bits) | z (7 bits), in half blocks | face direction (3 bits) | random
//!   rotation (1 bit) | which way is out of the rectangle, along x, y and z (1 bit each)
//! - word 1: texture layer (16 bits) | quad number (15 bits, only used by the debug view; or, with the
//!   top bit set, a bar with textures of its own and its slot)

use crate::coords::game_to_bevy;
use crate::world::{BlockId, Owners};
use bevy::math::Vec3;
use std::collections::HashMap;
use std::sync::Arc;

const N: usize = 32;
/// A face's quad number with this bit set is part of a bar with textures of its own, the low 5 bits
/// its slot (`push_quad`; the shader's `bar_uv`).
const BAR_TAG: u32 = 0x8000;
/// In the pieces' grids: no bar of its own textures.
const NO_BAR: u8 = u8::MAX;

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
    /// The cells built of pieces (their blocks are `BlockId::PIECES`): each one's index, and
    /// the piece filling each of its 8 octants (its slot and block).
    pub pieces: Vec<(usize, Owners)>,
    /// Per block number: its bars' own textures (a long side, an end), if it has them.
    pub bar_layers: Arc<Vec<Option<[u32; 2]>>>,
    /// The cells of pieces just outside the section next to those, and what fills their octants (by
    /// their place, −1 or 32 along one axis), so bars meeting across a section's side show no seam.
    pub beyond_pieces: HashMap<[i32; 3], [BlockId; 8]>,
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
    // A cell built of pieces hides nothing behind it, as glass doesn't: its pieces are drawn on their
    // own (`emit_pieces`).
    let is_opaque = |id: BlockId| id != BlockId::PIECES && input.opaque.get(id.0 as usize).copied().unwrap_or(true);
    let is_see_through = |id: BlockId| input.see_through.get(id.0 as usize).copied().unwrap_or(false);
    let is_grid = |id: BlockId| is_see_through(id) && input.inner_faces.get(id.0 as usize).copied().unwrap_or(false);
    let is_outline = |id: BlockId| is_see_through(id) && !is_grid(id);
    let mut out = MeshOutput::default();

    // Opaque blocks: faces where they meet anything not opaque.
    let opaque = build_columns(&input.blocks, is_opaque, &input.outside);
    emit_faces(&mut out, input, &opaque, [&opaque, &opaque], false);
    emit_pieces(&mut out, input, &is_opaque);

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

            for ((depth, block), mut plane) in planes {
                // The face of a block at `depth` lies on its far side when facing the positive way.
                let plane_position = depth + positive as u32;
                merge_plane(&mut plane, |u, v, width, height| {
                    push_quad(out, input, block, (axis, positive), plane_position * 2, [u * 2, v * 2, width * 2, height * 2], both_sides, None);
                });
            }
        }
    }
}

/// One rectangle of `block`'s face, facing along `axis` (the positive way or not), on the plane
/// `plane` across it, from (`u`, `v`) along the plane's axes, `width` × `height`: all in half blocks.
/// `bar`: the slot of the bar it's part of, if the bar has textures of its own. Then its layer is the
/// bar's side or end, and its quad number is `BAR_TAG` and the slot, for the shader to measure the
/// texture from the bar's own corner.
#[allow(clippy::too_many_arguments)]
fn push_quad(out: &mut MeshOutput, input: &MeshInput, block: BlockId, (axis, positive): (usize, bool), plane: u32, [u, v, width, height]: [u32; 4], both_sides: bool, bar: Option<u8>) {
    let (u_axis, v_axis) = plane_axes(axis);
    let face = face_index(axis, positive);
    let bar_layers = bar.and_then(|slot| input.bar_layers.get(block.0 as usize).copied().flatten().map(|layers| (slot, layers)));
    let (layer, random_rotation, quad_tag) = match bar_layers {
        // The end where the bar's own axis comes out of the face, else a long side.
        Some((slot, [side, end])) => (if (slot / 4) as usize == axis { end } else { side }, false, BAR_TAG | slot as u32),
        None => (
            input.face_layers.get(block.0 as usize).map_or(0, |layers| layers[face]),
            input.face_random_rotation.get(block.0 as usize).is_some_and(|faces| faces[face]),
            out.quads & 0x7FFF,
        ),
    };
    let corners = [(u, v), (u + width, v), (u + width, v + height), (u, v + height)];
    let base = out.vertices.len() as u32;
    for (cu, cv) in corners {
        let mut p = [0u32; 3];
        p[axis] = plane;
        p[u_axis] = cu;
        p[v_axis] = cv;
        // Which way is out of the rectangle at this corner, along each game axis (1 = the positive
        // way): the shader nudges the corner that way by a hair, so neighbouring rectangles overlap
        // instead of leaving pixel-thin cracks where a big one's edge meets small ones' corners (the
        // sky flickered through).
        let mut out_bits = [0u32; 3];
        out_bits[u_axis] = (cu != u) as u32;
        out_bits[v_axis] = (cv != v) as u32;
        out.vertices.push([
            p[0] | p[1] << 7 | p[2] << 14 | (face as u32) << 21 | (random_rotation as u32) << 24 | out_bits[0] << 25 | out_bits[1] << 26 | out_bits[2] << 27,
            (layer & 0xFFFF) | quad_tag << 16,
        ]);
    }
    let front: [u32; 6] = [0, 1, 2, 0, 2, 3];
    let back: [u32; 6] = [0, 2, 1, 0, 3, 2];
    let (outward, inward) = if needs_flip(axis, positive) { (back, front) } else { (front, back) };
    out.indices.extend(outward.iter().map(|i| base + i));
    if both_sides {
        out.indices.extend(inward.iter().map(|i| base + i));
    }
    out.quads += 1;
}

/// The cells built of pieces (`world/pieces.rs`): each filled octant's faces, where nothing opaque
/// covers them (another octant of the cell, a neighbouring cell's octant, or a whole block), merged
/// within the cell, so a bar's long side is one rectangle. Not merged across cells: they're few.
fn emit_pieces(out: &mut MeshOutput, input: &MeshInput, is_opaque: &dyn Fn(BlockId) -> bool) {
    // Each cell's octants as the blocks filling them (for what covers what).
    let blocks_of = |owners: &Owners| owners.map(|owner| owner.map_or(BlockId::AIR, |(_, block)| block));
    let cells: HashMap<usize, [BlockId; 8]> = input.pieces.iter().map(|(index, owners)| (*index, blocks_of(owners))).collect();
    // A bar with textures of its own is measured from its own corner, so it's merged only with itself.
    let own_textures = |block: BlockId| input.bar_layers.get(block.0 as usize).is_some_and(Option::is_some);
    let octant = |h: [usize; 3]| h[0] + 2 * h[1] + 4 * h[2];
    for (index, owners) in &input.pieces {
        let (index, octants) = (*index, blocks_of(owners));
        let cell = [index % N, (index / N) % N, index / (N * N)];
        for axis in 0..3 {
            let (u_axis, v_axis) = plane_axes(axis);
            for positive in [true, false] {
                let face = face_index(axis, positive);
                for depth in 0..2 {
                    // The octants' faces showing at this depth, as a 2 × 2 grid across the face: the
                    // block, and its bar's slot if the bar has textures of its own (else `NO_BAR`).
                    let mut grid = [[(BlockId::AIR, NO_BAR); 2]; 2];
                    for (hv, row) in grid.iter_mut().enumerate() {
                        for (hu, shown) in row.iter_mut().enumerate() {
                            let mut h = [0; 3];
                            (h[axis], h[u_axis], h[v_axis]) = (depth, hu, hv);
                            let Some((slot, block)) = owners[octant(h)] else { continue };
                            let bar = if own_textures(block) { slot.0 } else { NO_BAR };
                            let inside = if positive { depth == 0 } else { depth == 1 };
                            let covered = if inside {
                                h[axis] = 1 - depth;
                                is_opaque(octants[octant(h)])
                            } else {
                                // The neighbouring cell's octant that touches this one.
                                h[axis] = depth ^ 1;
                                let mut next = cell.map(|c| c as i32);
                                next[axis] += if positive { 1 } else { -1 };
                                if (0..N as i32).contains(&next[axis]) {
                                    let at = next[0] as usize + next[1] as usize * N + next[2] as usize * N * N;
                                    match input.blocks[at] {
                                        BlockId::PIECES => cells.get(&at).is_some_and(|other| is_opaque(other[octant(h)])),
                                        whole => is_opaque(whole),
                                    }
                                } else {
                                    match input.beyond_pieces.get(&next) {
                                        Some(other) => is_opaque(other[octant(h)]),
                                        None => (input.outside[face][cell[v_axis]] >> cell[u_axis]) & 1 == 1,
                                    }
                                }
                            };
                            if !covered {
                                *shown = (block, bar);
                            }
                        }
                    }
                    // Merged within the cell: the whole face, rows, columns, then single octants.
                    let plane = (cell[axis] * 2 + depth) as u32 + positive as u32;
                    let (u0, v0) = ((cell[u_axis] * 2) as u32, (cell[v_axis] * 2) as u32);
                    let mut emit = |(block, bar): (BlockId, u8), [u, v, width, height]: [u32; 4]| {
                        push_quad(out, input, block, (axis, positive), plane, [u0 + u, v0 + v, width, height], false, (bar != NO_BAR).then_some(bar));
                    };
                    let same = |a: (BlockId, u8), b: (BlockId, u8)| !a.0.is_air() && a == b;
                    if same(grid[0][0], grid[0][1]) && same(grid[0][0], grid[1][0]) && same(grid[0][0], grid[1][1]) {
                        emit(grid[0][0], [0, 0, 2, 2]);
                        continue;
                    }
                    let mut done = [[false; 2]; 2];
                    for v in 0..2 {
                        if same(grid[v][0], grid[v][1]) {
                            emit(grid[v][0], [0, v as u32, 2, 1]);
                            done[v] = [true, true];
                        }
                    }
                    for u in 0..2 {
                        if !done[0][u] && !done[1][u] && same(grid[0][u], grid[1][u]) {
                            emit(grid[0][u], [u as u32, 0, 1, 2]);
                            (done[0][u], done[1][u]) = (true, true);
                        }
                    }
                    for v in 0..2 {
                        for u in 0..2 {
                            if !done[v][u] && !grid[v][u].0.is_air() {
                                emit(grid[v][u], [u as u32, v as u32, 1, 1]);
                            }
                        }
                    }
                }
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
            pieces: Vec::new(),
            bar_layers: Arc::new(Vec::new()),
            beyond_pieces: HashMap::new(),
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
            let face = (vertex[0] >> 21) & 7;
            let rotates = (vertex[0] >> 24) & 1 == 1;
            assert_eq!(rotates, face == 4);
        }
    }

    #[test]
    fn each_corner_knows_which_way_is_out_of_its_rectangle() {
        // A 3 × 1 × 1 bar: every face's corners say "out" away from the face's middle, along the
        // face's plane (the shader nudges them that way so rectangles overlap a hair).
        let mut blocks = empty();
        for x in 4..7 {
            blocks[x + 5 * N + 5 * N * N] = STONE;
        }
        let mesh = mesh_section(&input(blocks));
        for quad in mesh.vertices.chunks_exact(4) {
            let at = |v: &[u32; 2], k: u32| ((v[0] >> (7 * k)) & 127) as f32 / 2.0;
            let middle: Vec<f32> = (0..3).map(|k| quad.iter().map(|v| at(v, k)).sum::<f32>() / 4.0).collect();
            for v in quad {
                for k in 0..3u32 {
                    let out = (v[0] >> (25 + k)) & 1 == 1;
                    let offset = at(v, k) - middle[k as usize];
                    if offset != 0.0 {
                        assert_eq!(out, offset > 0.0, "axis {k}: corner {} vs middle {}", at(v, k), middle[k as usize]);
                    }
                }
            }
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
                (w & 127) as f32 / 2.0,
                ((w >> 7) & 127) as f32 / 2.0,
                ((w >> 14) & 127) as f32 / 2.0,
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

    /// A section with one cell built of these pieces at (5, 5, 5), and `whole` blocks.
    fn with_pieces(pieces: &crate::world::Pieces, whole: &[(usize, BlockId)]) -> MeshOutput {
        let mut blocks = empty();
        let at = 5 + 5 * N + 5 * N * N;
        blocks[at] = BlockId::PIECES;
        for &(index, block) in whole {
            blocks[index] = block;
        }
        let mut data = input(blocks);
        data.pieces = vec![(at, pieces.owners())];
        mesh_section(&data)
    }

    /// Every corner of the mesh, in blocks (game axes).
    fn corners(mesh: &MeshOutput) -> Vec<Vec3> {
        mesh.vertices.iter().map(|v| Vec3::new((v[0] & 127) as f32, ((v[0] >> 7) & 127) as f32, ((v[0] >> 14) & 127) as f32) / 2.0).collect()
    }

    #[test]
    fn a_bar_is_a_box_of_six_faces_at_half_block_corners() {
        use crate::world::{Pieces, Slot};
        // A low north–south bar on the west: 1 × ½ × ½ in the cell's south-west-low corner.
        let mut pieces = Pieces::default();
        pieces.add(Slot::bar(0, 0, 0), STONE);
        let mesh = with_pieces(&pieces, &[]);
        assert_eq!(mesh.quads, 6, "its long sides each one rectangle");
        let corners = corners(&mesh);
        let (low, high) = corners.iter().fold((Vec3::MAX, Vec3::MIN), |(lo, hi), &c| (lo.min(c), hi.max(c)));
        assert_eq!((low, high), (Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 5.5, 5.5)));
        // Its triangles face outward, as a block's do.
        let centre = game_to_bevy(Vec3::new(5.5, 5.25, 5.25));
        for triangle in mesh.indices.chunks(3) {
            let [a, b, c] = [0, 1, 2].map(|k| game_to_bevy(corners[triangle[k] as usize]));
            assert!((b - a).cross(c - a).dot((a + b + c) / 3.0 - centre) > 0.0, "a triangle faces inward");
        }
    }

    #[test]
    fn a_bar_with_its_own_textures_shows_its_side_and_end_measured_from_its_own_corner() {
        use crate::world::{Pieces, Slot};
        // Stone's bars have textures of their own: a side (layer 7) and an end (layer 8).
        let mut corner = Pieces::default();
        corner.add(Slot::bar(0, 0, 0), STONE);
        corner.add(Slot::bar(1, 0, 0), STONE);
        let mut blocks = empty();
        let at = 5 + 5 * N + 5 * N * N;
        blocks[at] = BlockId::PIECES;
        let mut data = input(blocks);
        data.pieces = vec![(at, corner.owners())];
        data.bar_layers = Arc::new(vec![None, Some([7, 8])]);
        let mesh = mesh_section(&data);
        for quad in mesh.vertices.chunks_exact(4) {
            let (face, layer, tag) = ((quad[0][0] >> 21) & 7, quad[0][1] & 0xFFFF, quad[0][1] >> 16);
            assert!(tag & BAR_TAG != 0, "every face says which bar it's part of");
            let slot = Slot((tag & 31) as u8);
            // The faces along the bar's own axis are its ends.
            assert_eq!(layer, if face / 2 == slot.0 as u32 / 4 { 8 } else { 7 }, "{} face {face}", slot.name());
        }
        // The two bars aren't merged into one face (each is measured from its own corner): 6 faces
        // for the first, and the second's one free octant's 5 showing.
        assert_eq!(mesh.quads, 6 + 5);
    }

    #[test]
    fn bars_meeting_hide_their_faces_between_and_a_whole_block_hides_the_side_against_it() {
        use crate::world::{Pieces, Slot};
        // Two low bars, both on the west (north–south) and the south (east–west), meeting in the
        // corner they share: an L of three octants, with no faces between them.
        let mut corner = Pieces::default();
        corner.add(Slot::bar(0, 0, 0), STONE);
        corner.add(Slot::bar(1, 0, 0), DIRT);
        let mesh = with_pieces(&corner, &[]);
        let area: f32 = mesh.vertices.chunks_exact(4).map(|quad| {
            let c: Vec<Vec3> = quad.iter().map(|v| Vec3::new((v[0] & 127) as f32, ((v[0] >> 7) & 127) as f32, ((v[0] >> 14) & 127) as f32) / 2.0).collect();
            (c[1] - c[0]).length() * (c[3] - c[0]).length()
        }).sum();
        // Three octants, an L: the outside of 3 half-cubes (14 of their 18 faces, a ¼ each).
        assert!((area - 14.0 * 0.25).abs() < 1e-4, "{area}");
        // Stone under the cell: the bar's bottom isn't drawn (and the stone's top still is: the
        // cell hides nothing behind it).
        let mut bar = Pieces::default();
        bar.add(Slot::bar(0, 0, 0), STONE);
        let below = 5 + 5 * N + 4 * N * N;
        let mesh = with_pieces(&bar, &[(below, STONE)]);
        assert_eq!(mesh.quads, 5 + 6);
    }
}

//! Collision — moving a box through the block world without passing through blocks.
//!
//! The body is a box: `width` × `width` across, `height` tall, standing on `feet`
//! (the middle of its bottom face).
//!
//! Movement is done one axis at a time: up/down first, then north/south, then east/west.
//! For each axis we check every block the box's leading face would sweep through, nearest
//! first, and stop flush against the first solid one. Checking the whole swept range (instead
//! of just the end position) means even a fast fall can't skip through a thin floor.
//!
//! Doing the axes separately is what makes you *slide* along a wall when walking into it at
//! an angle: the blocked axis stops, the other keeps going.
//!
//! **Boxes** as well as blocks: a solid object's shape is boxes of any size, off the grid (its
//! model's own parts, pixel by pixel: `world/objects.rs`), and a body stops flush against those too
//! (David, 2026-11-01: "the collision still needs to be from the models geometry, not its volume").

use crate::ecs::math::{IVec3, Vec3};

/// Tiny gap used so a box resting exactly on a block boundary counts as touching it, not
/// overlapping it: at least this, and more far from the world's middle (`margin`).
const EPSILON: f32 = 1e-4;

/// The gap at a place `at` blocks from the middle: a few of the smallest steps a position can take
/// there, never less than `EPSILON`. Far out, positions only change in steps of 0.0002 to 0.004 blocks,
/// so a body stopped flush against a wall could land a hair inside it, the next move's sweep started
/// past the wall's face, and you walked through [2026-10-29: the "walking through block sides" bug,
/// from x = 2,048 out].
fn margin(at: f32) -> f32 {
    (at.abs() * f32::EPSILON * 4.0).max(EPSILON)
}

#[derive(Clone, Copy, Debug)]
pub struct BodySize {
    pub width: f32,
    pub height: f32,
}

/// Which axes were blocked during a move.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Blocked {
    pub x: bool,
    pub y: bool,
    /// Blocked while moving down = standing on something.
    pub down: bool,
    /// Blocked while moving up = hit your head.
    pub up: bool,
}

/// Boxes a body bumps into besides the blocks: each one's two corners, in the grid's units.
pub type Boxes<'a> = &'a [(Vec3, Vec3)];

/// Move the body by `motion` (game axes). Returns the new feet position and what blocked it.
#[cfg(test)]
pub fn move_body(feet: Vec3, size: BodySize, motion: Vec3, solid: impl FnMut(IVec3) -> bool) -> (Vec3, Blocked) {
    move_body_among(feet, size, motion, &[], solid)
}

/// `move_body`, bumping into `boxes` too.
fn move_body_among(
    feet: Vec3,
    size: BodySize,
    motion: Vec3,
    boxes: Boxes,
    mut solid: impl FnMut(IVec3) -> bool,
) -> (Vec3, Blocked) {
    let mut feet = feet;
    let mut blocked = Blocked::default();
    for axis in [2, 0, 1] {
        let (moved, hit) = sweep_axis(feet, size, axis, motion[axis], boxes, &mut solid);
        feet = moved;
        if hit {
            match axis {
                0 => blocked.x = true,
                1 => blocked.y = true,
                _ if motion.z < 0.0 => blocked.down = true,
                _ => blocked.up = true,
            }
        }
    }
    (feet, blocked)
}

// ── On the half-block grid ────────────────────────────────────────────────
//
// A body meets the bars in a cell as they are by stepping through the world in half blocks
// (`VoxelWorld::half_filled`): every position and size doubled, moved as ever, and halved back, which
// is exact (quartile stop 2). `solid_half` says whether a half-block is solid.

impl BodySize {
    fn doubled(self) -> Self {
        Self { width: self.width * 2.0, height: self.height * 2.0 }
    }
}

/// Boxes (blocks) on the half-block grid.
fn doubled(boxes: Boxes) -> Vec<(Vec3, Vec3)> {
    boxes.iter().map(|&(low, high)| (low * 2.0, high * 2.0)).collect()
}

/// `move_body`, on the half-block grid, bumping into `boxes` (blocks) too.
pub fn move_body_halves(feet: Vec3, size: BodySize, motion: Vec3, boxes: Boxes, solid_half: impl FnMut(IVec3) -> bool) -> (Vec3, Blocked) {
    let (feet, blocked) = move_body_among(feet * 2.0, size.doubled(), motion * 2.0, &doubled(boxes), solid_half);
    (feet / 2.0, blocked)
}

/// `overlaps_solid`, on the half-block grid, `boxes` (blocks) too.
pub fn overlaps_solid_halves(feet: Vec3, size: BodySize, boxes: Boxes, solid_half: impl FnMut(IVec3) -> bool) -> bool {
    overlaps_among(feet * 2.0, size.doubled(), &doubled(boxes), solid_half)
}

/// `has_support`, on the half-block grid, `boxes` (blocks) too.
pub fn has_support_halves(feet: Vec3, size: BodySize, boxes: Boxes, solid_half: impl FnMut(IVec3) -> bool) -> bool {
    has_support_among(feet * 2.0, size.doubled(), &doubled(boxes), solid_half)
}

/// How high a body steps up without jumping, walking into it: half a block (a bar, a slab or a stair of
/// bars), if there's room above [David, 2026-11-01: step up half blocks automatically, with head room].
pub const STEP_UP: f32 = 0.5;

/// `move_body_halves`, stepping up: a body `on_ground` walking into something no higher than
/// `STEP_UP`, with room above, goes up onto it. Tried raised, moved across, then set down; kept if it
/// got further. A full block's side still stops it.
pub fn move_body_stepping(feet: Vec3, size: BodySize, motion: Vec3, on_ground: bool, boxes: Boxes, mut solid_half: impl FnMut(IVec3) -> bool) -> (Vec3, Blocked) {
    let (moved, blocked) = move_body_halves(feet, size, motion, boxes, &mut solid_half);
    if !on_ground || !(blocked.x || blocked.y) || motion.z > 0.0 {
        return (moved, blocked);
    }
    let raised = feet + Vec3::Z * STEP_UP;
    if overlaps_solid_halves(raised, size, boxes, &mut solid_half) {
        return (moved, blocked);
    }
    let (across, stepped) = move_body_halves(raised, size, Vec3::new(motion.x, motion.y, 0.0), boxes, &mut solid_half);
    let (settled, _) = move_body_halves(across, size, Vec3::Z * -STEP_UP, boxes, &mut solid_half);
    let gone = |to: Vec3| (to.x - feet.x).hypot(to.y - feet.y);
    if gone(settled) > gone(moved) + 1e-4 {
        (settled, Blocked { x: stepped.x, y: stepped.y, down: true, up: false })
    } else {
        (moved, blocked)
    }
}

/// The box's corners: (min, max).
pub fn body_bounds(feet: Vec3, size: BodySize) -> (Vec3, Vec3) {
    let half = size.width / 2.0;
    (
        Vec3::new(feet.x - half, feet.y - half, feet.z),
        Vec3::new(feet.x + half, feet.y + half, feet.z + size.height),
    )
}

/// Is there anything solid just under the body's footprint (something to stand on)?
#[cfg(test)]
pub fn has_support(feet: Vec3, size: BodySize, solid: impl FnMut(IVec3) -> bool) -> bool {
    has_support_among(feet, size, &[], solid)
}

/// `has_support`, `boxes` too.
fn has_support_among(feet: Vec3, size: BodySize, boxes: Boxes, solid: impl FnMut(IVec3) -> bool) -> bool {
    let sliver = BodySize {
        width: size.width,
        height: 0.1,
    };
    overlaps_among(feet - Vec3::Z * 0.1, sliver, boxes, solid)
}

/// Does the body overlap any solid block where it stands?
#[cfg(test)]
pub fn overlaps_solid(feet: Vec3, size: BodySize, solid: impl FnMut(IVec3) -> bool) -> bool {
    overlaps_among(feet, size, &[], solid)
}

/// Do these two ranges overlap by more than the gap at their ends?
fn ranges_meet(low: f32, high: f32, from: f32, to: f32) -> bool {
    from < high - margin(high) && to > low + margin(low)
}

/// `overlaps_solid`, `boxes` too.
fn overlaps_among(feet: Vec3, size: BodySize, boxes: Boxes, mut solid: impl FnMut(IVec3) -> bool) -> bool {
    let (min, max) = body_bounds(feet, size);
    if boxes.iter().any(|(low, high)| (0..3).all(|axis| ranges_meet(min[axis], max[axis], low[axis], high[axis]))) {
        return true;
    }
    let low = (min + min.map(margin)).floor().as_ivec3();
    let high = (max - max.map(margin)).floor().as_ivec3();
    for z in low.z..=high.z {
        for y in low.y..=high.y {
            for x in low.x..=high.x {
                if solid(IVec3::new(x, y, z)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Move along one axis, stopping at the first solid block, or box, in the way.
fn sweep_axis(
    feet: Vec3,
    size: BodySize,
    axis: usize,
    distance: f32,
    boxes: Boxes,
    solid: &mut impl FnMut(IVec3) -> bool,
) -> (Vec3, bool) {
    if distance == 0.0 {
        return (feet, false);
    }
    let (min, max) = body_bounds(feet, size);
    // The nearest box in the way stops it first, if it's nearer than any block: the move is cut to
    // it, and the blocks are swept only that far.
    let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut distance = distance;
    let mut boxed = false;
    for (low, high) in boxes {
        if !ranges_meet(min[a], max[a], low[a], high[a]) || !ranges_meet(min[b], max[b], low[b], high[b]) {
            continue;
        }
        let gap = if distance > 0.0 { low[axis] - max[axis] } else { high[axis] - min[axis] };
        let ahead = if distance > 0.0 { gap > -margin(max[axis]) } else { gap < margin(min[axis]) };
        if ahead && gap.abs() < distance.abs() {
            distance = if gap.abs() <= margin(max[axis].abs().max(min[axis].abs())) { 0.0 } else { gap };
            boxed = true;
        }
    }
    if distance == 0.0 {
        return (feet, boxed);
    }
    // The block range the box covers on the two other axes.
    let a_range = (min[a] + margin(min[a])).floor() as i32..=(max[a] - margin(max[a])).floor() as i32;
    let b_range = (min[b] + margin(min[b])).floor() as i32..=(max[b] - margin(max[b])).floor() as i32;
    let layer_is_solid = |layer: i32, solid: &mut dyn FnMut(IVec3) -> bool| {
        for i in a_range.clone() {
            for j in b_range.clone() {
                let mut cell = IVec3::ZERO;
                cell[axis] = layer;
                cell[a] = i;
                cell[b] = j;
                if solid(cell) {
                    return true;
                }
            }
        }
        false
    };

    let mut result = feet;
    if distance > 0.0 {
        let front = max[axis];
        let first = (front - margin(front)).floor() as i32 + 1;
        let last = (front + distance - margin(front)).floor() as i32;
        for layer in first..=last {
            if layer_is_solid(layer, solid) {
                // Stop with our front face flush against the block's near face.
                result[axis] += layer as f32 - front;
                return (result, true);
            }
        }
    } else {
        let front = min[axis];
        let first = (front + margin(front)).floor() as i32 - 1;
        let last = (front + distance + margin(front)).floor() as i32;
        for layer in (last..=first).rev() {
            if layer_is_solid(layer, solid) {
                result[axis] += (layer + 1) as f32 - front;
                return (result, true);
            }
        }
    }
    result[axis] += distance;
    (result, boxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DWARF: BodySize = BodySize {
        width: 0.6,
        height: 1.5,
    };

    /// Flat floor: blocks at Z ≤ 9 are solid, so the floor's top surface is Z = 10.
    fn floor(cell: IVec3) -> bool {
        cell.z <= 9
    }

    /// On the half-block grid: the floor (its top at Z 10), with a half-block step from X 5 (top
    /// 10.5), then a whole block from X 8 (top 11).
    fn steps(half: IVec3) -> bool {
        half.z < 20 || (half.x >= 10 && half.z < 21) || (half.x >= 16 && half.z < 22)
    }

    #[test]
    fn a_body_steps_up_half_a_block_but_not_a_whole_one() {
        let feet = Vec3::new(4.0, 0.5, 10.0);
        // Walking into the half step: up onto it, on the ground.
        let (moved, blocked) = move_body_stepping(feet, DWARF, Vec3::new(2.0, 0.0, -0.01), true, &[], steps);
        assert_eq!((moved.x, moved.z, blocked.down), (6.0, 10.5, true));
        // In the air (or rising): no stepping, it stops against the step's side.
        let (moved, _) = move_body_stepping(feet, DWARF, Vec3::new(2.0, 0.0, -0.01), false, &[], steps);
        assert!(moved.x < 4.71 && moved.z < 10.5, "{moved}");
        // From the step, into the whole block (another half up, from 10.5 to 11): up again. But from
        // the floor straight into a whole block's side: stopped.
        let (moved, _) = move_body_stepping(Vec3::new(7.0, 0.5, 10.5), DWARF, Vec3::new(2.0, 0.0, -0.01), true, &[], steps);
        assert_eq!((moved.x, moved.z), (9.0, 11.0));
        let wall = |half: IVec3| half.z < 20 || (half.x >= 10 && half.z < 22);
        let (moved, blocked) = move_body_stepping(feet, DWARF, Vec3::new(2.0, 0.0, -0.01), true, &[], wall);
        assert!(moved.x < 4.71 && moved.z == 10.0 && blocked.x, "{moved}");
        // No room overhead (a ceiling at Z 11.5, just over a body standing on the floor, so not over
        // one on the step): no stepping up.
        let low = |half: IVec3| steps(half) || half.z >= 23;
        let (moved, _) = move_body_stepping(feet, DWARF, Vec3::new(2.0, 0.0, -0.01), true, &[], low);
        assert!(moved.x < 4.71, "{moved}");
    }

    #[test]
    fn a_body_bumps_into_an_objects_boxes_not_the_cell_round_them() {
        // A pole 2 pixels thick in the middle of cell (6, 0), on the floor (Z 10), 1.5 tall.
        let pole = [(Vec3::new(6.4375, 0.4375, 10.0), Vec3::new(6.5625, 0.5625, 11.5))];
        let feet = Vec3::new(4.0, 0.5, 10.0);
        // Walking into it: stopped flush against it (the body is 0.6 across), well inside its cell.
        let (moved, blocked) = move_body_stepping(feet, DWARF, Vec3::new(3.0, 0.0, -0.01), true, &pole, steps_free);
        assert!((moved.x - (6.4375 - 0.3)).abs() < 1e-3 && blocked.x, "{moved}");
        // Beside it, in the same cell: walks on by.
        let (moved, blocked) = move_body_stepping(Vec3::new(4.0, 0.1, 10.0), DWARF, Vec3::new(3.0, 0.0, -0.01), true, &pole, steps_free);
        assert!(moved.x == 7.0 && !blocked.x, "{moved}");
        // A low box (a pixel under half a block): stepped up onto, and stood on.
        let slab = [(Vec3::new(5.0, 0.0, 10.0), Vec3::new(8.0, 1.0, 10.4375))];
        let (moved, _) = move_body_stepping(feet, DWARF, Vec3::new(2.0, 0.0, -0.01), true, &slab, steps_free);
        assert!(moved.x == 6.0 && (moved.z - 10.4375).abs() < 1e-3, "{moved}");
        assert!(has_support_halves(moved, DWARF, &slab, steps_free));
    }

    /// On the half-block grid: just the floor (its top at Z 10).
    fn steps_free(half: IVec3) -> bool {
        half.z < 20
    }

    #[test]
    fn support_ends_at_the_edge_of_a_ledge() {
        // A ledge: floor only where X < 5.
        let ledge = |cell: IVec3| floor(cell) && cell.x < 5;
        assert!(has_support(Vec3::new(4.5, 0.5, 10.0), DWARF, ledge));
        // Hanging part-way over the edge still has support…
        assert!(has_support(Vec3::new(5.2, 0.5, 10.0), DWARF, ledge));
        // …but fully past it there's nothing underneath.
        assert!(!has_support(Vec3::new(5.6, 0.5, 10.0), DWARF, ledge));
    }

    #[test]
    fn falling_lands_on_the_floor() {
        let (feet, blocked) = move_body(Vec3::new(0.5, 0.5, 12.0), DWARF, Vec3::new(0.0, 0.0, -5.0), floor);
        assert!((feet.z - 10.0).abs() < 1e-4);
        assert!(blocked.down);
    }

    #[test]
    fn a_very_fast_fall_cannot_skip_a_thin_floor() {
        // A one-block-thick floor at Z = 5, nothing below it.
        let thin = |cell: IVec3| cell.z == 5;
        let (feet, blocked) = move_body(Vec3::new(0.5, 0.5, 20.0), DWARF, Vec3::new(0.0, 0.0, -50.0), thin);
        assert!((feet.z - 6.0).abs() < 1e-4);
        assert!(blocked.down);
    }

    #[test]
    fn standing_still_on_the_floor_stays_put() {
        let (feet, blocked) = move_body(Vec3::new(0.5, 0.5, 10.0), DWARF, Vec3::new(0.0, 0.0, -0.01), floor);
        assert!((feet.z - 10.0).abs() < 1e-4);
        assert!(blocked.down);
        assert!(!overlaps_solid(feet, DWARF, floor));
    }

    #[test]
    fn walking_into_a_wall_slides_along_it() {
        // Floor plus a wall: blocks at X ≥ 3 (above the floor too).
        let world = |cell: IVec3| floor(cell) || cell.x >= 3;
        let (feet, blocked) = move_body(Vec3::new(1.5, 0.5, 10.0), DWARF, Vec3::new(5.0, 2.0, -0.01), world);
        assert!((feet.x - (3.0 - 0.3)).abs() < 1e-4, "stopped against the wall");
        assert!((feet.y - 2.5).abs() < 1e-4, "still slid east");
        assert!(blocked.x && !blocked.y);
    }

    #[test]
    fn jumping_into_a_ceiling_stops_at_it() {
        // Floor plus a ceiling block at Z = 12 (so the space is 2 blocks high).
        let world = |cell: IVec3| floor(cell) || cell.z == 12;
        let (feet, blocked) = move_body(Vec3::new(0.5, 0.5, 10.0), DWARF, Vec3::new(0.0, 0.0, 3.0), world);
        assert!((feet.z - (12.0 - 1.5)).abs() < 1e-4);
        assert!(blocked.up);
    }


    #[test]
    fn walls_hold_far_from_the_middle_walking_into_them_frame_by_frame() {
        // Far out, a position can only change in steps of about 0.002 to 0.004 blocks: a body
        // stopped flush against a wall can land a hair inside it. Walk into walls (east, west,
        // north, south) at many places out to the world's edge, a tick at a time, sliding along
        // them at an angle, and never come out the far side.
        for k in 0..40 {
            let far = 1_000.0 + k as f32 * 900.0;
            for (dx, dy) in [(1.0f32, 0.3f32), (-1.0, 0.3), (0.3, 1.0), (0.3, -1.0)] {
                let wall = |cell: IVec3| {
                    let at = Vec3::new(cell.x as f32, cell.y as f32, 0.0) - Vec3::new(far, -far, 0.0);
                    floor(cell) || (dx > 0.5 && at.x >= 3.0) || (dx < -0.5 && at.x <= -4.0) || (dy > 0.5 && at.y >= 3.0) || (dy < -0.5 && at.y <= -4.0)
                };
                let mut feet = Vec3::new(far + 0.5, -far + 0.5, 10.0);
                for _ in 0..600 {
                    let motion = Vec3::new(dx, dy, -0.2) * (6.0 / 60.0);
                    feet = move_body(feet, DWARF, motion, wall).0;
                    assert!(!overlaps_solid(feet, DWARF, wall), "inside the wall at {feet} ({k}, {dx}, {dy})");
                }
            }
        }
    }
    #[test]
    fn walls_work_at_negative_coordinates() {
        let world = |cell: IVec3| floor(cell) || cell.y <= -4;
        let (feet, blocked) = move_body(Vec3::new(0.5, -1.5, 10.0), DWARF, Vec3::new(0.0, -5.0, 0.0), world);
        assert!((feet.y - (-3.0 + 0.3)).abs() < 1e-4);
        assert!(blocked.y);
    }
}

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

use bevy::math::{IVec3, Vec3};

/// Tiny gap used so a box resting exactly on a block boundary counts as touching it,
/// not overlapping it.
const EPSILON: f32 = 1e-4;

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

/// Move the body by `motion` (game axes). Returns the new feet position and what blocked it.
pub fn move_body(
    feet: Vec3,
    size: BodySize,
    motion: Vec3,
    mut solid: impl FnMut(IVec3) -> bool,
) -> (Vec3, Blocked) {
    let mut feet = feet;
    let mut blocked = Blocked::default();
    for axis in [2, 0, 1] {
        let (moved, hit) = sweep_axis(feet, size, axis, motion[axis], &mut solid);
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

/// The box's corners: (min, max).
pub fn body_bounds(feet: Vec3, size: BodySize) -> (Vec3, Vec3) {
    let half = size.width / 2.0;
    (
        Vec3::new(feet.x - half, feet.y - half, feet.z),
        Vec3::new(feet.x + half, feet.y + half, feet.z + size.height),
    )
}

/// Is there anything solid just under the body's footprint (something to stand on)?
pub fn has_support(feet: Vec3, size: BodySize, solid: impl FnMut(IVec3) -> bool) -> bool {
    let sliver = BodySize {
        width: size.width,
        height: 0.1,
    };
    overlaps_solid(feet - Vec3::Z * 0.1, sliver, solid)
}

/// Does the body overlap any solid block where it stands?
pub fn overlaps_solid(feet: Vec3, size: BodySize, mut solid: impl FnMut(IVec3) -> bool) -> bool {
    let (min, max) = body_bounds(feet, size);
    let low = (min + Vec3::splat(EPSILON)).floor().as_ivec3();
    let high = (max - Vec3::splat(EPSILON)).floor().as_ivec3();
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

/// Move along one axis, stopping at the first solid block in the way.
fn sweep_axis(
    feet: Vec3,
    size: BodySize,
    axis: usize,
    distance: f32,
    solid: &mut impl FnMut(IVec3) -> bool,
) -> (Vec3, bool) {
    if distance == 0.0 {
        return (feet, false);
    }
    let (min, max) = body_bounds(feet, size);
    // The block range the box covers on the two other axes.
    let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
    let a_range = (min[a] + EPSILON).floor() as i32..=(max[a] - EPSILON).floor() as i32;
    let b_range = (min[b] + EPSILON).floor() as i32..=(max[b] - EPSILON).floor() as i32;
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
        let first = (front - EPSILON).floor() as i32 + 1;
        let last = (front + distance - EPSILON).floor() as i32;
        for layer in first..=last {
            if layer_is_solid(layer, solid) {
                // Stop with our front face flush against the block's near face.
                result[axis] += layer as f32 - front;
                return (result, true);
            }
        }
    } else {
        let front = min[axis];
        let first = (front + EPSILON).floor() as i32 - 1;
        let last = (front + distance + EPSILON).floor() as i32;
        for layer in (last..=first).rev() {
            if layer_is_solid(layer, solid) {
                result[axis] += (layer + 1) as f32 - front;
                return (result, true);
            }
        }
    }
    result[axis] += distance;
    (result, false)
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
    fn walls_work_at_negative_coordinates() {
        let world = |cell: IVec3| floor(cell) || cell.y <= -4;
        let (feet, blocked) = move_body(Vec3::new(0.5, -1.5, 10.0), DWARF, Vec3::new(0.0, -5.0, 0.0), world);
        assert!((feet.y - (-3.0 + 0.3)).abs() < 1e-4);
        assert!(blocked.y);
    }
}

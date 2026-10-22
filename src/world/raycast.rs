//! Raycast — "which block is this line pointing at?"
//!
//! Used to find the block under the crosshair (and in step 5, which block to break,
//! and where a placed block goes).
//!
//! How it works: instead of testing points along the line (which can skip corners),
//! we walk the line one block boundary at a time, always crossing whichever boundary
//! (X, Y or Z) comes next. Every block the line touches is visited exactly once, in order.
//! This is the standard "voxel traversal" method (Amanatides & Woo, 1987).
//!
//! Block (x, y, z) fills the space from x to x+1, y to y+1, z to z+1.

use bevy::math::{IVec3, Vec3};

/// What a ray hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    /// The block that was hit.
    pub block: IVec3,
    /// Which way the hit face points, e.g. (0, 0, 1) = the top face.
    /// (0, 0, 0) means the ray started inside the block, so no face was crossed.
    pub normal: IVec3,
    /// Distance from the ray's start to where it entered the block.
    pub distance: f32,
}

impl RayHit {
    /// The empty cell in front of the hit face: where a placed block would go.
    pub fn in_front(&self) -> Option<IVec3> {
        (self.normal != IVec3::ZERO).then(|| self.block + self.normal)
    }

    /// The hit face as a face number (0 north, 1 south, 2 east, 3 west, 4 top, 5 bottom).
    pub fn face(&self) -> Option<usize> {
        match self.normal.to_array() {
            [1, 0, 0] => Some(0),
            [-1, 0, 0] => Some(1),
            [0, 1, 0] => Some(2),
            [0, -1, 0] => Some(3),
            [0, 0, 1] => Some(4),
            [0, 0, -1] => Some(5),
            _ => None,
        }
    }
}

/// Walk from `origin` along `direction` (game axes) for up to `max_distance` blocks,
/// returning the first block for which `solid` says yes.
pub fn cast_ray(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    mut solid: impl FnMut(IVec3) -> bool,
) -> Option<RayHit> {
    let direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return None;
    }

    let mut block = origin.floor().as_ivec3();
    if solid(block) {
        return Some(RayHit {
            block,
            normal: IVec3::ZERO,
            distance: 0.0,
        });
    }

    // Per axis: which way we step (+1 / −1 / 0), how far along the ray one whole block is,
    // and how far along the ray the next boundary crossing is.
    let mut step = IVec3::ZERO;
    let mut t_delta = Vec3::splat(f32::INFINITY);
    let mut t_next = Vec3::splat(f32::INFINITY);
    for axis in 0..3 {
        let d = direction[axis];
        if d > 0.0 {
            step[axis] = 1;
            t_delta[axis] = 1.0 / d;
            t_next[axis] = (block[axis] as f32 + 1.0 - origin[axis]) / d;
        } else if d < 0.0 {
            step[axis] = -1;
            t_delta[axis] = -1.0 / d;
            t_next[axis] = (origin[axis] - block[axis] as f32) / -d;
        }
    }

    loop {
        // Cross whichever boundary is nearest.
        let axis = if t_next.x <= t_next.y && t_next.x <= t_next.z {
            0
        } else if t_next.y <= t_next.z {
            1
        } else {
            2
        };
        let distance = t_next[axis];
        if distance > max_distance {
            return None;
        }
        block[axis] += step[axis];
        t_next[axis] += t_delta[axis];
        if solid(block) {
            let mut normal = IVec3::ZERO;
            normal[axis] = -step[axis];
            return Some(RayHit {
                block,
                normal,
                distance,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A floor: everything at Z ≤ 10 is solid.
    fn floor(block: IVec3) -> bool {
        block.z <= 10
    }

    #[test]
    fn looking_straight_down_hits_the_top_face() {
        let hit = cast_ray(Vec3::new(0.5, 0.5, 15.5), Vec3::NEG_Z, 100.0, floor).unwrap();
        assert_eq!(hit.block, IVec3::new(0, 0, 10));
        assert_eq!(hit.normal, IVec3::Z);
        assert_eq!(hit.face(), Some(4));
        assert_eq!(hit.in_front(), Some(IVec3::new(0, 0, 11)));
        assert!((hit.distance - 4.5).abs() < 1e-4);
    }

    #[test]
    fn works_at_negative_coordinates() {
        let hit = cast_ray(Vec3::new(-3.2, -7.9, 12.0), Vec3::NEG_Z, 100.0, floor).unwrap();
        assert_eq!(hit.block, IVec3::new(-4, -8, 10));
    }

    #[test]
    fn a_slanted_ray_hits_a_wall_side() {
        // A wall: everything at X ≥ 5 is solid. Ray heads north (+X) and slightly east.
        let hit = cast_ray(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.0, 0.3, 0.0), 100.0, |b| b.x >= 5).unwrap();
        assert_eq!(hit.block.x, 5);
        assert_eq!(hit.normal, IVec3::NEG_X); // the wall's south face
        assert_eq!(hit.face(), Some(1));
    }

    #[test]
    fn too_short_a_ray_misses() {
        assert_eq!(cast_ray(Vec3::new(0.5, 0.5, 15.5), Vec3::NEG_Z, 4.0, floor), None);
        assert_eq!(cast_ray(Vec3::new(0.5, 0.5, 15.5), Vec3::Z, 100.0, floor), None);
    }

    #[test]
    fn starting_inside_a_block_reports_it_with_no_face() {
        let hit = cast_ray(Vec3::new(0.5, 0.5, 5.5), Vec3::NEG_Z, 100.0, floor).unwrap();
        assert_eq!(hit.block, IVec3::new(0, 0, 5));
        assert_eq!(hit.face(), None);
        assert_eq!(hit.in_front(), None);
    }
}

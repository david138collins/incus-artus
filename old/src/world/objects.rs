//! The room objects hold in the world, and where they snap to [`docs/objects.md` section 9; David,
//! 2026-11-01].
//!
//! - **By its model's size** ("basically we just go by the models size"): an object's size is its
//!   model's, in pixels (`data/objects.rs`, `Shape`), turned as it faces.
//! - **Small or full-block:**
//!   - **Half a block or less across** (8 pixels both ways, an octant's width): it sits on the octant
//!     grid, or in the middle of a cell, and holds only the octants it's in ("blocks that are thinner
//!     than 16x16 can sit on octants, or the center, working like the other blocks").
//!   - **Wider:** a full-block object. It snaps to cells as blocks do and holds whole blocks across
//!     ("it needs 4 octants just like a gabbro block"); a bed 64 × 32 holds 4 × 2. With Shift it
//!     goes on the octant grid, as a block does.
//!   - Either way it holds octants up its height, and may stick out of its room by a few pixels (its
//!     `stick_over`) and just clip ("it can still stick a little over probably and just clip").
//! - **What bodies bump into is its model's shape**, not its room ("the collision still needs to be
//!   from the models geometry, not its volume"): the shape's boxes, turned and placed (`solid_boxes`).
//! - **Turning** (B, `facing`): a quarter turn each, anticlockwise seen from above.
//!
//! Everything here is counted in pixels (16ths of a block) and half blocks, as whole numbers, apart
//! from the boxes bodies bump into.

use super::{CHUNK_SIZE, ChunkPos, PlacedObject, VoxelWorld};
use crate::data::{ObjectDef, ObjectRegistry};
use crate::ecs::math::{IVec3, Vec3};

/// A point (blocks, from an object's feet) turned as the object faces: a quarter turn each,
/// anticlockwise seen from above (x to y), as it's drawn (`render/objects.rs`).
pub fn turned(at: Vec3, facing: u8) -> Vec3 {
    (0..facing % 4).fold(at, |at, _| Vec3::new(-at.y, at.x, at.z))
}

/// Its size in pixels, turned as it faces.
fn size(def: &ObjectDef, facing: u8) -> IVec3 {
    let size = def.shape_or_block().size;
    if facing % 2 == 1 { IVec3::new(size.y, size.x, size.z) } else { size }
}

/// Is it small: half a block or less across both ways (with what it may stick over), so it sits on
/// the octant grid?
pub fn is_small(def: &ObjectDef) -> bool {
    let size = def.shape_or_block().size;
    size.x.max(size.y) <= 8 + def.stick_over as i32
}

/// The room it holds, in pixels: an octant across if it's small, else whole blocks across; octants
/// up its height. What it sticks over by (its `stick_over`) doesn't count.
pub fn room(def: &ObjectDef, facing: u8) -> IVec3 {
    let size = size(def, facing);
    let over = def.stick_over as i32;
    let up = |pixels: i32, step: i32| ((pixels - over).max(1) + step - 1) / step * step;
    if is_small(def) { IVec3::new(8, 8, up(size.z, 8)) } else { IVec3::new(up(size.x, 16), up(size.y, 16), up(size.z, 8)) }
}

/// The octants it holds (half blocks): its lowest and its highest. Its room is centred on its feet
/// across, and stands on them.
pub fn held(object: &PlacedObject, def: &ObjectDef) -> (IVec3, IVec3) {
    let room = room(def, object.facing);
    let low = object.at - IVec3::new(room.x / 2, room.y / 2, 0);
    let high = low + room;
    (low.div_euclid(IVec3::splat(8)), (high + IVec3::splat(7)).div_euclid(IVec3::splat(8)) - IVec3::ONE)
}

/// Where its feet go (pixels), aimed at a face: `half` the octant hit, `normal` its face, `point`
/// where the crosshair met it; `shifted` with Shift held.
///
/// - **Small:** in the middle of the cell in front of the face, at the height of the octant in front
///   of it; with Shift, in that octant, against a wall or half a block on.
/// - **Full-block:** in the cell in front of the face (with Shift, the octant), reaching out of the
///   face, and across it centred on that cell, toward the crosshair's side where it's an even number
///   across (a bed's two blocks go the side you aim at), as a block on the octant grid does.
pub fn spot(def: &ObjectDef, facing: u8, half: IVec3, normal: IVec3, point: Vec3, shifted: bool) -> Option<IVec3> {
    if normal == IVec3::ZERO {
        return None;
    }
    let front = half + normal;
    let cell = front.div_euclid(IVec3::splat(2));
    if is_small(def) {
        return Some(if shifted { front * 8 + IVec3::new(4, 4, 0) } else { IVec3::new(cell.x * 16 + 8, cell.y * 16 + 8, front.z * 8) });
    }
    let room = room(def, facing);
    let (step, base) = if shifted { (8, front) } else { (16, cell) };
    let mut at = IVec3::new(0, 0, base.z * step);
    for axis in 0..2 {
        let count = room[axis] / step;
        let start = if normal[axis] > 0 {
            base[axis]
        } else if normal[axis] < 0 {
            base[axis] - (count - 1)
        } else if point[axis] * 16.0 >= (base[axis] * step) as f32 + step as f32 / 2.0 {
            base[axis] - (count - 1) / 2
        } else {
            base[axis] - count / 2
        };
        at[axis] = start * step + room[axis] / 2;
    }
    Some(at)
}

/// The boxes of its model's shape, turned and placed (blocks): what bodies bump into, and what the
/// crosshair meets.
pub fn shape_boxes(object: &PlacedObject, def: &ObjectDef) -> Vec<(Vec3, Vec3)> {
    let feet = object.at.as_vec3() / 16.0;
    def.shape_or_block()
        .boxes
        .iter()
        .map(|&(low, high)| {
            let (a, b) = (turned(low, object.facing), turned(high, object.facing));
            (feet + a.min(b), feet + a.max(b))
        })
        .collect()
}

/// The box round its whole shape (blocks).
pub fn object_box(object: &PlacedObject, objects: &ObjectRegistry) -> (Vec3, Vec3) {
    let Some(def) = objects.by_name(&object.name) else {
        let feet = object.at.as_vec3() / 16.0;
        return (feet - Vec3::new(0.5, 0.5, 0.0), feet + Vec3::new(0.5, 0.5, 1.0));
    };
    shape_boxes(object, def).into_iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(low, high), (a, b)| (low.min(a), high.max(b)))
}

/// The objects whose feet are in the chunks round `cell` (its chunk and the eight beside it): every
/// object that could reach it.
pub fn objects_around(world: &VoxelWorld, cell: IVec3) -> impl Iterator<Item = &PlacedObject> {
    let pos = ChunkPos::of_block(cell);
    (-1..=1).flat_map(move |y| (-1..=1).map(move |x| IVec3::new((pos.x + x) * CHUNK_SIZE, (pos.y + y) * CHUNK_SIZE, 0))).flat_map(|corner| world.objects_near(corner).iter())
}

/// The object holding this octant (half blocks), if one does.
pub fn holder<'a>(world: &'a VoxelWorld, objects: &ObjectRegistry, half: IVec3) -> Option<&'a PlacedObject> {
    objects_around(world, half.div_euclid(IVec3::splat(2))).find(|object| {
        objects.by_name(&object.name).is_some_and(|def| {
            let (low, high) = held(object, def);
            half.cmpge(low).all() && half.cmple(high).all()
        })
    })
}

/// The boxes of the solid objects reaching into the box `low` to `high` (blocks): what a body there
/// can bump into.
pub fn solid_boxes(world: &VoxelWorld, objects: &ObjectRegistry, low: Vec3, high: Vec3) -> Vec<(Vec3, Vec3)> {
    let mut boxes = Vec::new();
    for object in objects_around(world, ((low + high) / 2.0).floor().as_ivec3()) {
        let Some(def) = objects.by_name(&object.name).filter(|def| def.solid) else { continue };
        boxes.extend(shape_boxes(object, def).into_iter().filter(|(a, b)| a.cmplt(high).all() && b.cmpgt(low).all()));
    }
    boxes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Shape, Sits};

    /// An object of this size in pixels, one box.
    fn sized(x: i32, y: i32, z: i32) -> ObjectDef {
        let size = IVec3::new(x, y, z);
        let half = size.as_vec3() / 32.0;
        let shape = Shape { scale: 1.0, feet: Vec3::ZERO, size, boxes: vec![(Vec3::new(-half.x, -half.y, 0.0), Vec3::new(half.x, half.y, z as f32 / 16.0))], points: Vec::new() };
        ObjectDef { name: "incus:thing".into(), display_name: "thing".into(), model: "thing.gltf".into(), height: None, stick_over: 2, solid: true, sits: Sits::Floor, tags: Vec::new(), shape: Some(shape), sprite: None, light: None }
    }

    fn placed(at: IVec3, facing: u8) -> PlacedObject {
        PlacedObject { name: "incus:thing".into(), at, facing, stage: 0, since: 0 }
    }

    #[test]
    fn small_objects_hold_an_octant_and_big_ones_whole_blocks_by_their_models_size() {
        // A cup, 6 across: small, an octant across, an octant tall.
        let cup = sized(6, 6, 7);
        assert!(is_small(&cup));
        assert_eq!(room(&cup, 0), IVec3::new(8, 8, 8));
        // 10 across sticks over by 2, and is still small; 11 isn't.
        assert!(is_small(&sized(10, 10, 8)) && !is_small(&sized(11, 8, 8)));
        // A brazier 12 across, 24 tall: a block across, three octants up.
        assert_eq!(room(&sized(12, 12, 24), 0), IVec3::new(16, 16, 24));
        // A bed 64 × 32 × 32: 4 × 2 blocks, 2 tall; turned a quarter, 2 × 4.
        let bed = sized(64, 32, 32);
        assert_eq!((room(&bed, 0), room(&bed, 1)), (IVec3::new(64, 32, 32), IVec3::new(32, 64, 32)));
        // A cup in the middle of a cell holds the four octants round the middle; on an octant, one.
        assert_eq!(held(&placed(IVec3::new(8, 8, 0), 0), &cup), (IVec3::new(0, 0, 0), IVec3::new(1, 1, 0)));
        assert_eq!(held(&placed(IVec3::new(4, 12, 8), 0), &cup), (IVec3::new(0, 1, 1), IVec3::new(0, 1, 1)));
        // The bed, its feet on a cell's edge: cells -2 to 1 along x, 0 to 1 along y, two up.
        assert_eq!(held(&placed(IVec3::new(0, 16, 0), 0), &bed), (IVec3::new(-4, 0, 0), IVec3::new(3, 3, 3)));
    }

    #[test]
    fn objects_snap_as_blocks_do_small_ones_to_the_middle_or_an_octant() {
        // Aimed at the top of the block under cell (0, 0, 1): its top octant (0, 0, 1) in half blocks,
        // the face up, the crosshair a little toward +x and +y.
        let (half, up, point) = (IVec3::new(0, 0, 1), IVec3::Z, Vec3::new(0.4, 0.4, 1.0));
        let cup = sized(6, 6, 7);
        assert_eq!(spot(&cup, 0, half, up, point, false), Some(IVec3::new(8, 8, 16)), "in the cell's middle");
        assert_eq!(spot(&cup, 0, half, up, point, true), Some(IVec3::new(4, 4, 16)), "on the octant");
        // A brazier: in the cell's middle, with or without Shift (where it lines up, it's the same).
        let brazier = sized(12, 12, 24);
        assert_eq!(spot(&brazier, 0, half, up, point, false), Some(IVec3::new(8, 8, 16)));
        assert_eq!(spot(&brazier, 0, half, up, point, true), Some(IVec3::new(8, 8, 16)));
        // Shifted, aimed at the octant's other half: half a block on.
        assert_eq!(spot(&brazier, 0, half, up, Vec3::new(0.1, 0.4, 1.0), true), Some(IVec3::new(0, 8, 16)));
        // The bed on the floor, aimed at the +x half of the cell: its four blocks from -1 to 2.
        let bed = sized(64, 32, 32);
        let at = spot(&bed, 0, half, up, Vec3::new(0.7, 0.7, 1.0), false).unwrap();
        assert_eq!(held(&placed(at, 0), &bed), (IVec3::new(-2, 0, 2), IVec3::new(5, 3, 5)));
        // Against a wall (aimed at the +x face of the block at x = 1, from the -x side): it reaches
        // out of the wall, toward -x, and never into it.
        let at = spot(&bed, 0, IVec3::new(2, 0, 2), IVec3::NEG_X, Vec3::new(1.0, 0.7, 1.2), false).unwrap();
        assert_eq!(held(&placed(at, 0), &bed), (IVec3::new(-6, 0, 2), IVec3::new(1, 3, 5)));
    }

    #[test]
    fn turning_turns_the_shape_a_quarter_at_a_time() {
        let long = sized(32, 16, 8);
        let boxes = shape_boxes(&placed(IVec3::new(16, 16, 0), 1), &long);
        assert_eq!(boxes, vec![(Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.5, 2.0, 0.5))]);
        assert_eq!(turned(Vec3::X, 1), Vec3::Y);
        assert_eq!(turned(Vec3::X, 4), Vec3::X);
    }
}

//! Edits — the one front door for changing blocks during play.
//!
//! Nobody (except world generation and the debug panel) calls `set_block` directly.
//! Instead they send a **request** ("please break the block at X, Y, Z"; see `events`).
//! On the next simulation tick, `apply_block_edits` checks each request against the rules,
//! then either makes the change and announces it (**BlockChanged**), or refuses it and
//! says why (**BlockEditRefused**). Anything can listen to both (later: item drops, sounds,
//! saving, settlers noticing a wall is gone, ...).
//!
//! Edits happen on ticks, so nothing changes while time is paused. (Plans laid over the
//! world, like dig designations and blueprint ghosts, are a separate layer that will work
//! while paused; see docs/decisions.md.)

use super::objects::{held, holder};
use super::{BlockId, Part, Pieces, PlacedObject, VoxelWorld, bar_halves, block_halves, octant_of};
use crate::data::{BlockRegistry, ObjectId, ObjectRegistry};
use crate::events::{LoggedEvent, PlayerId};
use crate::time::TimeControl;
use crate::ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Who or what asked for a change. (Later: settlers, the structure editor, world events.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditSource {
    Player(PlayerId),
}

impl std::fmt::Display for EditSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditSource::Player(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Break,
    Place(BlockId),
    /// A bar of a block (`pieces.rs`) from its corner `at`, counted in half blocks (its lowest), lying
    /// along `axis` (0 north–south, 1 east–west, 2 upright): in one cell, or across two.
    PlaceBar { at: IVec3, axis: u8, block: BlockId },
    /// The piece at an octant (counted in half blocks): a bar, with what's stuck to it, or a lone
    /// octant.
    BreakPiece { half: IVec3 },
    /// A single octant of a bar block, an octblock (counted in half blocks), part of no bar.
    PlaceOctant { half: IVec3, block: BlockId },
    /// A whole block on the octant grid, from its lowest corner `at` (counted in half blocks): half
    /// a block off the cells along one, two or three axes, its 8 octants across 2, 4 or 8 cells.
    PlaceShifted { at: IVec3, block: BlockId },
    /// An object (`data/objects.rs`), its foot at `at` (16ths of a block), facing `facing` (quarter
    /// turns from north).
    PlaceObject { at: IVec3, object: ObjectId, facing: u8 },
    /// The object whose foot is at `at` (16ths of a block), taken away.
    BreakObject { at: IVec3 },
}

/// A block's name for the log ("pieces" for a cell built of them).
fn name_of(block: BlockId, blocks: &BlockRegistry) -> String {
    if block == BlockId::PIECES { "pieces".into() } else { blocks.name(block).to_string() }
}

/// "Please change this block."
#[derive(Message, Clone, Copy, Debug)]
pub struct BlockEditRequest {
    pub position: IVec3,
    pub kind: EditKind,
    pub source: EditSource,
}

/// "This block changed." Sent after the world has been updated.
#[derive(Message, Clone, Copy, Debug)]
pub struct BlockChanged {
    pub position: IVec3,
    pub old: BlockId,
    pub new: BlockId,
    pub source: EditSource,
    pub tick: u64,
}

/// Why a request was turned down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefuseReason {
    /// Breaking: there's nothing there.
    NothingThere,
    /// Breaking: the bottom layer is protected (a debug rule).
    BottomLayerProtected,
    /// Placing: something that can't be replaced is already there.
    Occupied,
    /// That part of the world isn't loaded (or is outside it).
    NotLoaded,
    /// Time is paused, so the world can't change.
    TimePaused,
    /// A bar of a block that isn't marked as one that can be a bar (its file's `bar`).
    NotABar,
    /// A whole block of one that's only ever a bar.
    OnlyABar,
    /// An object with nothing solid under it to stand on.
    NothingUnder,
}

/// "That request was turned down, and here's why."
#[derive(Message, Clone, Copy, Debug)]
pub struct BlockEditRefused {
    pub request: BlockEditRequest,
    pub reason: RefuseReason,
    pub tick: u64,
}

fn describe_kind(kind: EditKind, blocks: &BlockRegistry) -> String {
    match kind {
        EditKind::Break => "break".to_string(),
        EditKind::BreakPiece { half } => format!("break the piece at {} {} {} (half blocks)", half.x, half.y, half.z),
        EditKind::PlaceOctant { half, block } => format!("place an octant of {} at {} {} {} (half blocks)", blocks.name(block), half.x, half.y, half.z),
        EditKind::Place(block) => format!("place {}", blocks.name(block)),
        EditKind::PlaceBar { at, axis, block } => format!("place a {} bar of {} from {} {} {} (half blocks)", ["north–south", "east–west", "upright"][axis as usize % 3], blocks.name(block), at.x, at.y, at.z),
        EditKind::PlaceShifted { at, block } => format!("place {} on the octant grid from {} {} {} (half blocks)", blocks.name(block), at.x, at.y, at.z),
        EditKind::PlaceObject { at, object, .. } => format!("place object {} at {} {} {} (16ths)", object.0, at.x, at.y, at.z),
        EditKind::BreakObject { at } => format!("take the object at {} {} {} (16ths)", at.x, at.y, at.z),
    }
}

impl LoggedEvent for BlockEditRequest {
    const KIND: &'static str = "edit request";
    fn describe(&self, blocks: &BlockRegistry) -> String {
        let p = self.position;
        format!("{} asks to {} at {} {} {}", self.source, describe_kind(self.kind, blocks), p.x, p.y, p.z)
    }
}

impl LoggedEvent for BlockChanged {
    const KIND: &'static str = "block changed";
    fn describe(&self, blocks: &BlockRegistry) -> String {
        let p = self.position;
        format!(
            "{} {} {}: {} → {} (by {})",
            p.x,
            p.y,
            p.z,
            name_of(self.old, blocks),
            name_of(self.new, blocks),
            self.source
        )
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

impl LoggedEvent for BlockEditRefused {
    const KIND: &'static str = "edit refused";
    fn describe(&self, blocks: &BlockRegistry) -> String {
        let p = self.request.position;
        format!(
            "{} {} {}: {} by {} refused: {:?}",
            p.x,
            p.y,
            p.z,
            describe_kind(self.request.kind, blocks),
            self.request.source,
            self.reason
        )
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// Tunable rules for edits (shown on the "Player" debug page).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditRules {
    /// TEMPORARY rule: the bottom layer (Z = 0) can't be broken, so you can't dig out of the world.
    pub protect_bottom_layer: bool,
}

impl Default for EditRules {
    fn default() -> Self {
        Self {
            protect_bottom_layer: true,
        }
    }
}

/// The most recent changes, newest first (for the debug panel).
#[derive(Resource, Default)]
pub struct RecentBlockChanges {
    pub changes: VecDeque<BlockChanged>,
    pub refused: u64,
}

const RECENT_LIMIT: usize = 12;

/// Runs on every simulation tick: carry out or refuse each waiting request.
pub fn apply_block_edits(
    mut requests: MessageReader<BlockEditRequest>,
    mut changed: MessageWriter<BlockChanged>,
    mut refused: MessageWriter<BlockEditRefused>,
    mut world: ResMut<VoxelWorld>,
    rules: Res<EditRules>,
    (registry, objects): (Res<BlockRegistry>, Res<ObjectRegistry>),
    time: Res<TimeControl>,
    mut recent: ResMut<RecentBlockChanges>,
) {
    let tick = time.tick();
    let mut refuse = |request: &BlockEditRequest, reason: RefuseReason, recent: &mut RecentBlockChanges| {
        recent.refused += 1;
        refused.write(BlockEditRefused {
            request: *request,
            reason,
            tick,
        });
    };
    for request in requests.read() {
        let changes = match request.kind {
            EditKind::Break => break_block(&mut world, &rules, request.position),
            EditKind::Place(block) => place_block(&mut world, &registry, &objects, request.position, block),
            EditKind::PlaceBar { at, axis, block } => place_bar(&mut world, &registry, &objects, at, axis, block),
            EditKind::BreakPiece { half } => break_piece(&mut world, half),
            EditKind::PlaceOctant { half, block } => place_octant(&mut world, &registry, &objects, half, block),
            EditKind::PlaceShifted { at, block } => place_shifted(&mut world, &registry, &objects, at, block),
            EditKind::PlaceObject { at, object, facing } => place_object(&mut world, &registry, &objects, at, object, facing, tick),
            EditKind::BreakObject { at } => world.remove_object(at).map(|_| Vec::new()).ok_or(RefuseReason::NothingThere),
        };
        match changes {
            Err(reason) => refuse(request, reason, &mut recent),
            Ok(changes) => {
                for (position, old, new) in changes {
                    let change = BlockChanged { position, old, new, source: request.source, tick };
                    changed.write(change);
                    recent.changes.push_front(change);
                }
                recent.changes.truncate(RECENT_LIMIT);
            }
        }
    }
}

/// What an edit changed (each cell: its block before and after), or why it was refused.
type Changes = Result<Vec<(IVec3, BlockId, BlockId)>, RefuseReason>;

/// A whole block into air, or into something replaceable (like the grid), but never into the same
/// block that's already there, nor a block that's only ever a bar.
fn place_block(world: &mut VoxelWorld, registry: &BlockRegistry, objects: &ObjectRegistry, at: IVec3, block: BlockId) -> Changes {
    if block_halves(at * 2).iter().any(|&half| holder(world, objects, half).is_some()) {
        return Err(RefuseReason::Occupied);
    }
    if registry.def(block).is_some_and(|def| def.bar.as_ref().is_some_and(|bar| bar.only)) {
        return Err(RefuseReason::OnlyABar);
    }
    let old = world.get_block(at);
    if !registry.is_replaceable(old) || old == block {
        return Err(RefuseReason::Occupied);
    }
    world.set_block(at, block).ok_or(RefuseReason::NotLoaded)?;
    Ok(vec![(at, old, block)])
}

/// A block broken. A cell of pieces is taken piece by piece, each with its bar's other half (in this
/// cell or the next) and the cut pieces stuck to it. (There are no items yet, so nothing drops.)
fn break_block(world: &mut VoxelWorld, rules: &EditRules, at: IVec3) -> Changes {
    let old = world.get_block(at);
    if old.is_air() {
        return Err(RefuseReason::NothingThere);
    }
    if rules.protect_bottom_layer && at.z == 0 {
        return Err(RefuseReason::BottomLayerProtected);
    }
    if old == BlockId::PIECES {
        let halves: Vec<IVec3> = (0..8).map(|o| at * 2 + IVec3::new(o & 1, (o >> 1) & 1, (o >> 2) & 1)).flat_map(|half| world.piece_halves(half)).collect();
        return clear_halves(world, &halves);
    }
    world.set_block(at, BlockId::AIR).ok_or(RefuseReason::NotLoaded)?;
    Ok(vec![(at, old, BlockId::AIR)])
}

/// One piece broken, from any octant of it (in half blocks): a bar, with the cut pieces stuck to it,
/// or a lone octant [David, 2026-11-01: "you can mine individual slabs in minecraft theres no reason
/// you shouldnt be able to target and mine quarter blocks"]. A cut piece can't be got back; there are
/// no items yet, so nothing drops at all.
fn break_piece(world: &mut VoxelWorld, half: IVec3) -> Changes {
    let halves = world.piece_halves(half);
    if halves.is_empty() {
        return Err(RefuseReason::NothingThere);
    }
    clear_halves(world, &halves)
}

/// Empties these octants (in half blocks), each cell once; a cell left with none is air.
fn clear_halves(world: &mut VoxelWorld, halves: &[IVec3]) -> Changes {
    let mut cells: Vec<IVec3> = halves.iter().map(|half| half.div_euclid(IVec3::splat(2))).collect();
    cells.sort_by_key(|cell| cell.to_array());
    cells.dedup();
    let mut changes = Vec::new();
    for cell in cells {
        let Some(mut pieces) = world.get_pieces(cell).cloned() else { continue };
        for half in halves.iter().filter(|half| half.div_euclid(IVec3::splat(2)) == cell) {
            pieces.clear(octant_of(*half));
        }
        let new = if pieces.is_empty() { BlockId::AIR } else { BlockId::PIECES };
        world.set_pieces(cell, pieces).ok_or(RefuseReason::NotLoaded)?;
        changes.push((cell, BlockId::PIECES, new));
    }
    Ok(changes)
}

/// A single octant of a bar block, an octblock (David: "there's no reason octblocks shouldn't be a
/// thing"), at `half` (in half blocks): into air, something replaceable, or a free octant of a cell of
/// pieces.
fn place_octant(world: &mut VoxelWorld, registry: &BlockRegistry, objects: &ObjectRegistry, half: IVec3, block: BlockId) -> Changes {
    if registry.def(block).is_none_or(|def| def.bar.is_none()) {
        return Err(RefuseReason::NotABar);
    }
    let cell = half.div_euclid(IVec3::splat(2));
    if !world.is_loaded(cell) {
        return Err(RefuseReason::NotLoaded);
    }
    if holder(world, objects, half).is_some() {
        return Err(RefuseReason::Occupied);
    }
    let old = world.get_block(cell);
    let mut pieces = if old == BlockId::PIECES {
        world.get_pieces(cell).cloned().unwrap_or_default()
    } else if registry.is_replaceable(old) {
        Pieces::default()
    } else {
        return Err(RefuseReason::Occupied);
    };
    if !pieces.set(octant_of(half), block, Part::LONE) {
        return Err(RefuseReason::Occupied);
    }
    world.set_pieces(cell, pieces).ok_or(RefuseReason::NotLoaded)?;
    Ok(vec![(cell, old, BlockId::PIECES)])
}

/// A bar of `block` from its corner `at` (in half blocks, its lowest), lying along `axis`: its two
/// octants, in one cell or across two. An octant already taken keeps what's there (the first piece
/// keeps it): the bar is cut, its other octant placed as a cut piece stuck to it. Into air,
/// something replaceable, or a cell of pieces; refused if neither octant is free.
fn place_bar(world: &mut VoxelWorld, registry: &BlockRegistry, objects: &ObjectRegistry, at: IVec3, axis: u8, block: BlockId) -> Changes {
    if registry.def(block).is_none_or(|def| def.bar.is_none()) {
        return Err(RefuseReason::NotABar);
    }
    let halves = bar_halves(at, axis);
    // Each octant's cell as it would become, if the octant's free.
    let mut free = Vec::new();
    for (end, half) in halves.into_iter().enumerate() {
        let cell = half.div_euclid(IVec3::splat(2));
        if !world.is_loaded(cell) {
            return Err(RefuseReason::NotLoaded);
        }
        let old = world.get_block(cell);
        let pieces = if old == BlockId::PIECES {
            world.get_pieces(cell).cloned().unwrap_or_default()
        } else if registry.is_replaceable(old) {
            Pieces::default()
        } else {
            continue;
        };
        if pieces.get(octant_of(half)).is_none() && holder(world, objects, half).is_none() {
            free.push((end as u8, half, cell, old));
        }
    }
    if free.is_empty() {
        return Err(RefuseReason::Occupied);
    }
    let whole = free.len() == 2;
    let mut changes = Vec::new();
    for (end, half, cell, old) in free {
        // (Both octants may be in one cell: read it again for the second.)
        let mut pieces = if world.get_block(cell) == BlockId::PIECES { world.get_pieces(cell).cloned().unwrap_or_default() } else { Pieces::default() };
        pieces.set(octant_of(half), block, Part::of_bar(axis as usize, end, whole));
        world.set_pieces(cell, pieces).ok_or(RefuseReason::NotLoaded)?;
        changes.push((cell, old, BlockId::PIECES));
    }
    Ok(changes)
}

/// A whole block of `block` on the octant grid (David, 2026-11-01: "we should be able to place whole
/// blocks on the oct grid, as well"; "shift should free you from the main grid"), from its lowest
/// corner `at` (in half blocks): its 8 octants, each a corner of it (`Part::of_block`), across 2, 4
/// or 8 cells, into air, something replaceable, or free octants of cells of pieces. Refused if any
/// octant is taken. Lined up with the cells, it's an ordinary block.
fn place_shifted(world: &mut VoxelWorld, registry: &BlockRegistry, objects: &ObjectRegistry, at: IVec3, block: BlockId) -> Changes {
    if at.rem_euclid(IVec3::splat(2)) == IVec3::ZERO {
        return place_block(world, registry, objects, at.div_euclid(IVec3::splat(2)), block);
    }
    if registry.def(block).is_some_and(|def| def.bar.as_ref().is_some_and(|bar| bar.only)) {
        return Err(RefuseReason::OnlyABar);
    }
    let halves = block_halves(at);
    for half in halves {
        let cell = half.div_euclid(IVec3::splat(2));
        if !world.is_loaded(cell) {
            return Err(RefuseReason::NotLoaded);
        }
        let old = world.get_block(cell);
        let free = if old == BlockId::PIECES { world.octant(half).is_none() } else { registry.is_replaceable(old) };
        if !free || holder(world, objects, half).is_some() {
            return Err(RefuseReason::Occupied);
        }
    }
    let mut changes = Vec::new();
    let mut cells: Vec<IVec3> = halves.iter().map(|half| half.div_euclid(IVec3::splat(2))).collect();
    cells.sort_by_key(|cell| cell.to_array());
    cells.dedup();
    for cell in cells {
        let old = world.get_block(cell);
        let mut pieces = if old == BlockId::PIECES { world.get_pieces(cell).cloned().unwrap_or_default() } else { Pieces::default() };
        for (corner, half) in halves.iter().enumerate().filter(|(_, half)| half.div_euclid(IVec3::splat(2)) == cell) {
            pieces.set(octant_of(*half), block, Part::of_block(corner));
        }
        world.set_pieces(cell, pieces).ok_or(RefuseReason::NotLoaded)?;
        changes.push((cell, old, BlockId::PIECES));
    }
    Ok(changes)
}

/// An object, its feet at `at` (16ths of a block), turned to `facing`: every octant it holds
/// (`world/objects.rs`) must be free (air or something replaceable, and no other object's), and
/// something solid under at least one of them. When it was placed is kept (`since`), for things that
/// change with time.
fn place_object(world: &mut VoxelWorld, registry: &BlockRegistry, objects: &ObjectRegistry, at: IVec3, object: ObjectId, facing: u8, tick: u64) -> Changes {
    let def = objects.def(object).ok_or(RefuseReason::NothingThere)?;
    let placed = PlacedObject { name: def.name.clone(), at, facing: facing % 4, stage: 0, since: tick };
    let (low, high) = held(&placed, def);
    let mut under = false;
    for y in low.y..=high.y {
        for x in low.x..=high.x {
            for z in low.z..=high.z {
                let half = IVec3::new(x, y, z);
                if !world.is_loaded(half.div_euclid(IVec3::splat(2))) {
                    return Err(RefuseReason::NotLoaded);
                }
                if world.half_filled(half, |block| !registry.is_replaceable(block)) || holder(world, objects, half).is_some() {
                    return Err(RefuseReason::Occupied);
                }
            }
            under |= world.half_filled(IVec3::new(x, y, low.z - 1), |block| registry.is_solid(block));
        }
    }
    if !under {
        return Err(RefuseReason::NothingUnder);
    }
    world.place_object(placed).ok_or(RefuseReason::NotLoaded)?;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{TestBlocks, TestWorldSettings, generate_test_world};
    use crate::ecs::ecs::system::RunSystemOnce;

    const PLAYER: EditSource = EditSource::Player(PlayerId(0));

    /// One object for the tests: a brazier, 1.5 blocks tall.
    /// Two objects for the tests: a brazier 12 pixels across and 24 tall (a full-block object, 1.5
    /// blocks tall), and a cup 6 across (small: it sits on an octant).
    fn test_objects() -> ObjectRegistry {
        let object = |name: &str, size: IVec3| {
            let half = size.as_vec3() / 32.0;
            let shape = crate::data::Shape { scale: 1.0, feet: Vec3::ZERO, size, boxes: vec![(Vec3::new(-half.x, -half.y, 0.0), Vec3::new(half.x, half.y, size.z as f32 / 16.0))], points: Vec::new() };
            crate::data::ObjectDef { name: format!("incus:{name}"), display_name: name.into(), model: format!("{name}.gltf"), height: None, stick_over: 2, solid: true, sits: crate::data::Sits::Floor, tags: Vec::new(), shape: Some(shape), sprite: None, light: None }
        };
        ObjectRegistry::with(vec![object("braziere", IVec3::new(12, 12, 24)), object("cup", IVec3::new(6, 6, 7))])
    }

    #[test]
    fn an_object_stands_on_the_floor_takes_its_cells_and_is_taken_away_whole() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        // The test world's ground is grass at Z 144: the cell above, 145, is the floor's.
        let cell = IVec3::new(2, 3, 145);
        let at = cell * 16 + IVec3::new(8, 8, 0);
        let brazier = BlockEditRequest { position: cell, kind: EditKind::PlaceObject { at, object: ObjectId(0), facing: 0 }, source: PLAYER };
        let floating = BlockEditRequest { position: cell, kind: EditKind::PlaceObject { at: at + IVec3::Z * 16 * 5, object: ObjectId(0), facing: 0 }, source: PLAYER };
        // Placed; the same again (taken); one in the air (nothing under it); a block in its upper cell
        // (it's 1.5 tall, so it holds the lower half of that cell too); a bar along the top of that
        // cell, which it leaves free.
        let block_above = BlockEditRequest { position: cell + IVec3::Z, kind: EditKind::Place(stone), source: PLAYER };
        let bar_on_top = BlockEditRequest { position: cell + IVec3::Z, kind: EditKind::PlaceBar { at: (cell + IVec3::Z) * 2 + IVec3::Z, axis: 0, block: stone }, source: PLAYER };
        let (mut app, _) = run_requests(&[brazier, brazier, floating, block_above, bar_on_top]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied, RefuseReason::NothingUnder, RefuseReason::Occupied]);
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!(world.objects_near(cell).iter().map(|object| (object.name.as_str(), object.at)).collect::<Vec<_>>(), vec![("incus:braziere", at)]);
        // Taken away; then nothing to take.
        let take = BlockEditRequest { position: cell, kind: EditKind::BreakObject { at }, source: PLAYER };
        let (mut app, _) = run_requests(&[brazier, take, take]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::NothingThere]);
        assert!(app.world().resource::<VoxelWorld>().objects_near(cell).is_empty());
    }

    #[test]
    fn small_objects_hold_only_their_octants() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let cell = IVec3::new(2, 3, 145);
        // Two cups on two octants of one cell's floor, against its -x wall.
        let cup = |half: IVec3| BlockEditRequest { position: cell, kind: EditKind::PlaceObject { at: half * 8 + IVec3::new(4, 4, 0), object: ObjectId(1), facing: 0 }, source: PLAYER };
        let (near, far) = (cell * 2, cell * 2 + IVec3::Y);
        // An octant beside them is free; one a cup holds isn't; a cup in the middle of the cell
        // would hold all four floor octants, so it's refused.
        let octant = |half: IVec3| BlockEditRequest { position: cell, kind: EditKind::PlaceOctant { half, block: stone }, source: PLAYER };
        let middle = BlockEditRequest { position: cell, kind: EditKind::PlaceObject { at: cell * 16 + IVec3::new(8, 8, 0), object: ObjectId(1), facing: 0 }, source: PLAYER };
        let (mut app, _) = run_requests(&[cup(near), cup(far), octant(near + IVec3::X), octant(far), middle]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied, RefuseReason::Occupied]);
        assert_eq!(app.world().resource::<VoxelWorld>().objects_near(cell).len(), 2);
    }

    /// A tiny app with just the test world and the edit system, run for one update.
    fn run_requests(requests: &[BlockEditRequest]) -> (App, TestBlocks) {
        // Stone can be a bar here (for the bars' tests).
        let mut registry = BlockRegistry::default();
        let stone = registry.id("incus:stone").unwrap();
        registry.def_mut(stone).unwrap().bar = Some(Default::default());
        let blocks = TestBlocks::from_registry(&registry);
        let mut voxels = VoxelWorld::default();
        generate_test_world(&mut voxels, &TestWorldSettings::default(), &blocks);
        let mut app = App::new();
        app.insert_resource(registry)
            .insert_resource(voxels)
            .insert_resource(test_objects())
            .init_resource::<EditRules>()
            .init_resource::<TimeControl>()
            .init_resource::<RecentBlockChanges>()
            .add_message::<BlockEditRequest>()
            .add_message::<BlockChanged>()
            .add_message::<BlockEditRefused>()
            .add_systems(Update, apply_block_edits);
        for request in requests {
            app.world_mut().write_message(*request);
        }
        app.update();
        (app, blocks)
    }

    fn refusals(app: &mut App) -> Vec<RefuseReason> {
        app.world_mut()
            .run_system_once(|mut refused: MessageReader<BlockEditRefused>| {
                refused.read().map(|refusal| refusal.reason).collect::<Vec<_>>()
            })
            .unwrap()
    }

    #[test]
    fn breaking_announces_the_change() {
        let surface = IVec3::new(0, 0, 144);
        let (mut app, blocks) = run_requests(&[BlockEditRequest { position: surface, kind: EditKind::Break, source: PLAYER }]);
        let changes = app
            .world_mut()
            .run_system_once(|mut changed: MessageReader<BlockChanged>| changed.read().copied().collect::<Vec<_>>())
            .unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old, blocks.grass);
        assert_eq!(changes[0].new, BlockId::AIR);
        assert_eq!(app.world().resource::<VoxelWorld>().get_block(surface), BlockId::AIR);
    }

    #[test]
    fn refusals_say_why() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let (mut app, _) = run_requests(&[
            BlockEditRequest { position: IVec3::new(0, 0, 200), kind: EditKind::Break, source: PLAYER },
            BlockEditRequest { position: IVec3::new(0, 0, 0), kind: EditKind::Break, source: PLAYER },
            BlockEditRequest { position: IVec3::new(0, 0, 144), kind: EditKind::Place(stone), source: PLAYER },
        ]);
        assert_eq!(
            refusals(&mut app),
            vec![RefuseReason::NothingThere, RefuseReason::BottomLayerProtected, RefuseReason::Occupied]
        );
    }

    /// A bar of `block` from its corner (in half blocks), along an axis.
    fn bar(at: IVec3, axis: u8, block: BlockId) -> BlockEditRequest {
        BlockEditRequest { position: at.div_euclid(IVec3::splat(2)), kind: EditKind::PlaceBar { at, axis, block }, source: PLAYER }
    }

    /// The octants filled in a cell, by number.
    fn filled(world: &VoxelWorld, cell: IVec3) -> Vec<usize> {
        world.get_pieces(cell).map_or_else(Vec::new, |pieces| (0..8).filter(|&o| pieces.get(o).is_some()).collect())
    }

    #[test]
    fn a_bar_crossing_one_already_there_is_cut_and_goes_with_it_when_that_s_mined() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let cell = IVec3::new(0, 0, 145);
        let low_west = cell * 2;
        // A: a low bar north–south on the west (octants 0 and 1). B: a low bar east–west on the south
        // (octants 0 and 2): 0 is A's, so B is cut to octant 2. The same A again, and one into the grass
        // below (a whole block): refused. A bar of grass (not a bar block): refused.
        let grass = BlockRegistry::default().id("incus:grass").unwrap();
        let below = bar(low_west - IVec3::Z * 2, 0, stone);
        let (mut app, _) = run_requests(&[bar(low_west, 0, stone), bar(low_west, 1, stone), bar(low_west, 0, stone), below, bar(low_west + IVec3::X * 4, 0, grass)]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied, RefuseReason::Occupied, RefuseReason::NotABar]);
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!(filled(world, cell), [0, 1, 2]);
        let pieces = world.get_pieces(cell).unwrap();
        assert!(pieces.get(2).unwrap().1.is_cut(), "B, cut");
        assert!(pieces.get(0).unwrap().1.whole() && pieces.get(1).unwrap().1.whole(), "A, whole");
        // A's piece: A and B's cut octant, stuck to it. B's: its cut octant alone.
        assert_eq!(world.piece_halves(low_west).len(), 3);
        assert_eq!(world.piece_halves(low_west + IVec3::Y), vec![low_west + IVec3::Y]);
        // Mining B's cut octant takes only it; mining A takes A and what's stuck to it.
        let mine = |half: IVec3| BlockEditRequest { position: cell, kind: EditKind::BreakPiece { half }, source: PLAYER };
        let (mut app, _) = run_requests(&[bar(low_west, 0, stone), bar(low_west, 1, stone), mine(low_west + IVec3::Y)]);
        assert!(refusals(&mut app).is_empty());
        assert_eq!(filled(app.world().resource::<VoxelWorld>(), cell), [0, 1]);
        let (mut app, _) = run_requests(&[bar(low_west, 0, stone), bar(low_west, 1, stone), mine(low_west + IVec3::X), mine(low_west)]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::NothingThere], "the second, A already gone");
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((world.get_block(cell), world.get_pieces(cell).is_none()), (BlockId::AIR, true));
    }

    #[test]
    fn an_octblock_goes_in_alone_cuts_a_bar_across_it_and_takes_that_with_it_when_mined() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let cell = IVec3::new(0, 0, 145);
        let low_west = cell * 2;
        let octant = BlockEditRequest { position: cell, kind: EditKind::PlaceOctant { half: low_west, block: stone }, source: PLAYER };
        let mine = BlockEditRequest { position: cell, kind: EditKind::BreakPiece { half: low_west }, source: PLAYER };
        // The octblock, then the same again (taken), then a bar across it: cut to its other octant.
        let (mut app, _) = run_requests(&[octant, octant, bar(low_west, 0, stone)]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied]);
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!(world.octant(low_west).map(|(_, part)| part), Some(Part::LONE));
        assert!(world.octant(low_west + IVec3::X).is_some_and(|(_, part)| part.is_cut()));
        // Mining the octblock takes the cut bar stuck to it.
        let (mut app, _) = run_requests(&[octant, bar(low_west, 0, stone), mine]);
        assert!(refusals(&mut app).is_empty());
        assert_eq!(app.world().resource::<VoxelWorld>().get_block(cell), BlockId::AIR);
    }

    #[test]
    fn a_whole_block_on_the_octant_grid_fills_eight_cells_an_eighth_each_and_is_mined_whole_from_any() {
        let grass = BlockRegistry::default().id("incus:grass").unwrap();
        let cell = IVec3::new(0, 0, 145);
        // Half a block off the cells every way: an octant in each of 8 cells.
        let at = cell * 2 + IVec3::ONE;
        let shifted = BlockEditRequest { position: cell, kind: EditKind::PlaceShifted { at, block: grass }, source: PLAYER };
        let (mut app, _) = run_requests(&[shifted, shifted]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied], "the same again: taken");
        let world = app.world().resource::<VoxelWorld>();
        for (corner, half) in block_halves(at).into_iter().enumerate() {
            let here = half.div_euclid(IVec3::splat(2));
            assert_eq!(filled(world, here), vec![7 - corner], "corner {corner}");
            assert_eq!(world.octant(half), Some((grass, Part::of_block(corner))));
            assert_eq!(world.piece_halves(half).len(), 8);
        }
        // Mined from its far corner: all of it, every cell air again.
        let mine = BlockEditRequest { position: cell, kind: EditKind::BreakPiece { half: at + IVec3::ONE }, source: PLAYER };
        let (mut app, _) = run_requests(&[shifted, mine]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert!(block_halves(at).iter().all(|half| world.get_block(half.div_euclid(IVec3::splat(2))) == BlockId::AIR));
        // Half a block off along one axis only: two cells, four octants each. Lined up: an ordinary block.
        let along = BlockEditRequest { position: cell, kind: EditKind::PlaceShifted { at: cell * 2 + IVec3::X, block: grass }, source: PLAYER };
        let lined_up = BlockEditRequest { position: cell, kind: EditKind::PlaceShifted { at: (cell + IVec3::Y * 3) * 2, block: grass }, source: PLAYER };
        let (mut app, _) = run_requests(&[along, lined_up]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((filled(world, cell), filled(world, cell + IVec3::X)), (vec![1, 3, 5, 7], vec![0, 2, 4, 6]));
        assert_eq!(world.get_block(cell + IVec3::Y * 3), grass);
    }

    #[test]
    fn a_bar_across_two_cells_is_mined_whole_from_either() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let (first, second) = (IVec3::new(0, 0, 145), IVec3::new(1, 0, 145));
        // From halfway along the first cell, north: an octant in each, whole.
        let at = first * 2 + IVec3::X;
        // A bar besides it in the second cell (east–west, high), which stays.
        let other = bar(second * 2 + IVec3::Z, 1, stone);
        let mine = BlockEditRequest { position: second, kind: EditKind::BreakPiece { half: at + IVec3::X }, source: PLAYER };
        let (mut app, _) = run_requests(&[bar(at, 0, stone), other]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((filled(world, first), filled(world, second)), (vec![1], vec![0, 4, 6]));
        assert_eq!(world.octant(at).map(|(_, part)| (part.end(), part.whole())), Some((0, true)));
        assert_eq!(world.octant(at + IVec3::X).map(|(_, part)| (part.end(), part.whole())), Some((1, true)));
        // Mined from the second cell: both halves go, the other bar stays.
        let (mut app, _) = run_requests(&[bar(at, 0, stone), other, mine]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((world.get_block(first), filled(world, second)), (BlockId::AIR, vec![4, 6]));
        // Breaking a whole cell of pieces takes each piece in it, and the halves they run into.
        let (mut app, _) = run_requests(&[bar(at, 0, stone), BlockEditRequest { position: second, kind: EditKind::Break, source: PLAYER }]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((world.get_block(first), world.get_block(second)), (BlockId::AIR, BlockId::AIR));
    }

    #[test]
    fn placing_into_a_grid_block_replaces_it() {
        let registry = BlockRegistry::default();
        let grid = registry.id("incus:grid").unwrap();
        let stone = registry.id("incus:stone").unwrap();
        let cell = IVec3::new(0, 0, 145);
        let (mut app, _) = run_requests(&[
            BlockEditRequest { position: cell, kind: EditKind::Place(grid), source: PLAYER },
            BlockEditRequest { position: cell, kind: EditKind::Place(stone), source: PLAYER },
        ]);
        assert!(refusals(&mut app).is_empty());
        assert_eq!(app.world().resource::<VoxelWorld>().get_block(cell), stone);
    }
}

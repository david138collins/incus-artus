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

use super::{BlockId, Pieces, Slot, VoxelWorld};
use crate::data::BlockRegistry;
use crate::events::{LoggedEvent, PlayerId};
use crate::time::TimeControl;
use bevy::prelude::*;
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
        EditKind::Place(block) => format!("place {}", blocks.name(block)),
        EditKind::PlaceBar { at, axis, block } => format!("place a {} bar of {} from {} {} {} (half blocks)", ["north–south", "east–west", "upright"][axis as usize % 3], blocks.name(block), at.x, at.y, at.z),
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
    registry: Res<BlockRegistry>,
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
            EditKind::Place(block) => place_block(&mut world, &registry, request.position, block),
            EditKind::PlaceBar { at, axis, block } => place_bar(&mut world, &registry, at, axis, block),
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
fn place_block(world: &mut VoxelWorld, registry: &BlockRegistry, at: IVec3, block: BlockId) -> Changes {
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

/// A block broken, a cell of pieces whole (David: "if you mix bars, it all just becomes nothing when
/// mined"; there are no items yet, so nothing drops). A bar running on into the cell next door goes
/// too: its other half.
fn break_block(world: &mut VoxelWorld, rules: &EditRules, at: IVec3) -> Changes {
    let old = world.get_block(at);
    if old.is_air() {
        return Err(RefuseReason::NothingThere);
    }
    if rules.protect_bottom_layer && at.z == 0 {
        return Err(RefuseReason::BottomLayerProtected);
    }
    let halves: Vec<(Slot, IVec3)> = world.get_pieces(at).map_or_else(Vec::new, |pieces| pieces.list().iter().filter_map(|&(slot, _)| Some((slot, slot.link()?))).collect());
    world.set_block(at, BlockId::AIR).ok_or(RefuseReason::NotLoaded)?;
    let mut changes = vec![(at, old, BlockId::AIR)];
    for (slot, toward) in halves {
        let next = at + toward;
        let Some(mut pieces) = world.get_pieces(next).cloned() else { continue };
        // Its other half: the octant across the side between them, pointing back.
        let partner = (slot.index() - 12) ^ (1 << toward.abs().max_position());
        if pieces.remove_half(partner, -toward) {
            let new = if pieces.is_empty() { BlockId::AIR } else { BlockId::PIECES };
            if world.set_pieces(next, pieces).is_some() {
                changes.push((next, BlockId::PIECES, new));
            }
        }
    }
    Ok(changes)
}

/// A bar of `block` from its corner `at` (in half blocks, its lowest), lying along `axis`. From a
/// whole cell's side it's a bar in that cell; from halfway, half a bar in each of two cells, linked
/// (`docs/quartile.md` section 10). Into air, something replaceable, or a cell of pieces with room;
/// both halves or neither.
fn place_bar(world: &mut VoxelWorld, registry: &BlockRegistry, at: IVec3, axis: u8, block: BlockId) -> Changes {
    if registry.def(block).is_none_or(|def| def.bar.is_none()) {
        return Err(RefuseReason::NotABar);
    }
    let a = axis as usize;
    let mut step = IVec3::ZERO;
    step[a] = 1;
    let two = IVec3::splat(2);
    let part = |half: IVec3, toward: Option<IVec3>| {
        let h = half.rem_euclid(two);
        let slot = match toward {
            None => Slot::bar(axis, h[(a + 1) % 3] as u8, h[(a + 2) % 3] as u8),
            Some(toward) => Slot::half(h.x as u8, h.y as u8, h.z as u8, toward),
        };
        (half.div_euclid(two), slot)
    };
    let parts = if at[a].rem_euclid(2) == 0 { vec![part(at, None)] } else { vec![part(at, Some(step)), part(at + step, Some(-step))] };
    let mut cells = Vec::new();
    for (cell, slot) in parts {
        if !world.is_loaded(cell) {
            return Err(RefuseReason::NotLoaded);
        }
        let old = world.get_block(cell);
        let mut pieces = if old == BlockId::PIECES {
            world.get_pieces(cell).cloned().unwrap_or_default()
        } else if registry.is_replaceable(old) {
            Pieces::default()
        } else {
            return Err(RefuseReason::Occupied);
        };
        if !pieces.add(slot, block) {
            return Err(RefuseReason::Occupied);
        }
        cells.push((cell, old, pieces));
    }
    let mut changes = Vec::new();
    for (cell, old, pieces) in cells {
        world.set_pieces(cell, pieces).ok_or(RefuseReason::NotLoaded)?;
        changes.push((cell, old, BlockId::PIECES));
    }
    Ok(changes)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{TestBlocks, TestWorldSettings, generate_test_world};
    use bevy::ecs::system::RunSystemOnce;

    const PLAYER: EditSource = EditSource::Player(PlayerId(0));

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

    #[test]
    fn bars_go_into_air_and_into_a_cell_of_bars_with_room_and_a_break_takes_the_cell() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let cell = IVec3::new(0, 0, 145);
        // A bar from a corner of the cell (counted in half blocks), along an axis.
        let bar = |at: IVec3, axis: u8, block: BlockId| BlockEditRequest { position: at.div_euclid(IVec3::splat(2)), kind: EditKind::PlaceBar { at, axis, block }, source: PLAYER };
        let low_west = cell * 2;
        // Two bars, then one whose octants are both taken (the same again), then one into the grass
        // below (a whole block): those two refused.
        let below = bar(low_west - IVec3::Z * 2, 0, stone);
        // And a bar of grass, which isn't marked as a block that can be a bar.
        let grass = BlockRegistry::default().id("incus:grass").unwrap();
        let (mut app, _) = run_requests(&[bar(low_west, 0, stone), bar(low_west, 1, stone), bar(low_west, 0, stone), below, bar(low_west + IVec3::X * 4, 0, grass)]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied, RefuseReason::Occupied, RefuseReason::NotABar]);
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!(world.get_block(cell), BlockId::PIECES);
        assert_eq!(world.get_pieces(cell).map(|pieces| pieces.list().len()), Some(2));
        // Breaking it takes the whole cell.
        let (mut app, _) = run_requests(&[bar(low_west, 0, stone), BlockEditRequest { position: cell, kind: EditKind::Break, source: PLAYER }]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((world.get_block(cell), world.get_pieces(cell).is_none()), (BlockId::AIR, true));
    }

    #[test]
    fn a_bar_across_two_cells_is_two_linked_halves_and_goes_whole_when_either_is_broken() {
        let stone = BlockRegistry::default().id("incus:stone").unwrap();
        let (first, second) = (IVec3::new(0, 0, 145), IVec3::new(1, 0, 145));
        // From halfway along the first cell, north: half in each.
        let at = first * 2 + IVec3::X;
        let place = BlockEditRequest { position: first, kind: EditKind::PlaceBar { at, axis: 0, block: stone }, source: PLAYER };
        // A bar in the second cell besides (east–west, high): breaking that cell takes it, and the
        // straddling bar's half in the first cell too.
        let other = BlockEditRequest { position: second, kind: EditKind::PlaceBar { at: second * 2 + IVec3::Z, axis: 1, block: stone }, source: PLAYER };
        let (mut app, _) = run_requests(&[place, other, BlockEditRequest { position: second, kind: EditKind::Break, source: PLAYER }, place]);
        assert!(refusals(&mut app).is_empty());
        // Placed, broken from the second cell (taking the first's half too), placed again.
        let changes = app.world_mut().run_system_once(|mut changed: MessageReader<BlockChanged>| changed.read().map(|change| (change.position, change.new)).collect::<Vec<_>>()).unwrap();
        assert_eq!(changes[..3], [(first, BlockId::PIECES), (second, BlockId::PIECES), (second, BlockId::PIECES)]);
        assert_eq!(changes[3..5], [(second, BlockId::AIR), (first, BlockId::AIR)], "the break took the first cell's half too");
        let world = app.world().resource::<VoxelWorld>();
        let halves = |cell: IVec3| world.get_pieces(cell).map(|pieces| pieces.list().iter().map(|(slot, _)| (slot.index(), slot.link())).collect::<Vec<_>>());
        // The first cell's half (its north, west, low octant: 12 + 1) goes on north; the second's
        // south half goes back south.
        assert_eq!(halves(first), Some(vec![(13, Some(IVec3::X))]));
        assert_eq!(halves(second), Some(vec![(12, Some(IVec3::NEG_X))]));
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

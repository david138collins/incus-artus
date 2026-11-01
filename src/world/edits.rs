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
    /// A bar of a block into a cell (`pieces.rs`): into air, or a cell of pieces with room.
    AddPiece { slot: Slot, block: BlockId },
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
        EditKind::AddPiece { slot, block } => format!("add a {} of {}", slot.name(), blocks.name(block)),
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
        let old = world.get_block(request.position);
        // A bar into a cell: its pieces with the bar added, if there's room for it.
        let mut pieces = None;
        let new = match request.kind {
            EditKind::AddPiece { slot, block } => {
                let mut cell = if old == BlockId::PIECES {
                    world.get_pieces(request.position).cloned().unwrap_or_default()
                } else if registry.is_replaceable(old) {
                    Pieces::default()
                } else {
                    refuse(request, RefuseReason::Occupied, &mut recent);
                    continue;
                };
                if !cell.add(slot, block) {
                    refuse(request, RefuseReason::Occupied, &mut recent);
                    continue;
                }
                pieces = Some(cell);
                BlockId::PIECES
            }
            EditKind::Break => {
                if old.is_air() {
                    refuse(request, RefuseReason::NothingThere, &mut recent);
                    continue;
                }
                if rules.protect_bottom_layer && request.position.z == 0 {
                    refuse(request, RefuseReason::BottomLayerProtected, &mut recent);
                    continue;
                }
                BlockId::AIR
            }
            EditKind::Place(block) => {
                // Into air, or into something replaceable (like the grid), but never into the
                // same block that's already there.
                if !registry.is_replaceable(old) || old == block {
                    refuse(request, RefuseReason::Occupied, &mut recent);
                    continue;
                }
                block
            }
        };
        // `None` = not loaded / outside the world: nothing happened.
        let done = match pieces {
            Some(cell) => world.set_pieces(request.position, cell),
            None => world.set_block(request.position, new),
        };
        if done.is_none() {
            refuse(request, RefuseReason::NotLoaded, &mut recent);
            continue;
        }
        let change = BlockChanged {
            position: request.position,
            old,
            new,
            source: request.source,
            tick,
        };
        changed.write(change);
        recent.changes.push_front(change);
        recent.changes.truncate(RECENT_LIMIT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{TestBlocks, TestWorldSettings, generate_test_world};
    use bevy::ecs::system::RunSystemOnce;

    const PLAYER: EditSource = EditSource::Player(PlayerId(0));

    /// A tiny app with just the test world and the edit system, run for one update.
    fn run_requests(requests: &[BlockEditRequest]) -> (App, TestBlocks) {
        let registry = BlockRegistry::default();
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
        let bar = |s: u8| BlockEditRequest { position: cell, kind: EditKind::AddPiece { slot: Slot(s), block: stone }, source: PLAYER };
        // Two bars, then one whose octants are both taken (the same slot again), then one into the
        // grass below (a whole block): those two refused.
        let below = BlockEditRequest { position: IVec3::new(0, 0, 144), ..bar(0) };
        let (mut app, _) = run_requests(&[bar(0), bar(4), bar(0), below]);
        assert_eq!(refusals(&mut app), vec![RefuseReason::Occupied, RefuseReason::Occupied]);
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!(world.get_block(cell), BlockId::PIECES);
        assert_eq!(world.get_pieces(cell).map(|pieces| pieces.list().len()), Some(2));
        // Breaking it takes the whole cell.
        let (mut app, _) = run_requests(&[bar(0), BlockEditRequest { position: cell, kind: EditKind::Break, source: PLAYER }]);
        assert!(refusals(&mut app).is_empty());
        let world = app.world().resource::<VoxelWorld>();
        assert_eq!((world.get_block(cell), world.get_pieces(cell).is_none()), (BlockId::AIR, true));
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

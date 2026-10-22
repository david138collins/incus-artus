//! Edits — the one front door for changing blocks during play.
//!
//! Nobody (except world generation and the debug panel) calls `set_block` directly.
//! Instead they send a **request** message ("please break the block at X, Y, Z").
//! `apply_block_edits` checks each request against the rules, makes the change, and
//! announces it with a **BlockChanged** message that anything can listen to (later:
//! item drops, sounds, saving, settlers noticing a wall is gone, ...).
//!
//! Step 6 (the event bus) builds on these: logging, a debug viewer, routing through ticks.
//! For now requests are applied in the same frame they're made.

use super::{BlockId, VoxelWorld};
use bevy::prelude::*;
use std::collections::VecDeque;

/// Who or what asked for a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditSource {
    Player,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Break,
    Place(BlockId),
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
}

/// Tunable rules for edits (shown on the "Player" debug page).
#[derive(Resource, Clone)]
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

pub fn apply_block_edits(
    mut requests: MessageReader<BlockEditRequest>,
    mut changed: MessageWriter<BlockChanged>,
    mut world: ResMut<VoxelWorld>,
    rules: Res<EditRules>,
    mut recent: ResMut<RecentBlockChanges>,
) {
    for request in requests.read() {
        let old = world.get_block(request.position);
        let new = match request.kind {
            EditKind::Break => {
                if old.is_air() || (rules.protect_bottom_layer && request.position.z == 0) {
                    recent.refused += 1;
                    continue;
                }
                BlockId::AIR
            }
            EditKind::Place(block) => {
                if !old.is_air() {
                    recent.refused += 1;
                    continue;
                }
                block
            }
        };
        // `None` = not loaded / outside the world: nothing happened.
        if world.set_block(request.position, new).is_none() {
            recent.refused += 1;
            continue;
        }
        let change = BlockChanged {
            position: request.position,
            old,
            new,
            source: request.source,
        };
        changed.write(change);
        recent.changes.push_front(change);
        recent.changes.truncate(RECENT_LIMIT);
    }
}

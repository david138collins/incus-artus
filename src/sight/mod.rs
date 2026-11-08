//! Sight (the regroup, phase 3, docs/regroup.md): the world as the eye takes it in, from the far view.
//!
//! - `cell`: what a cell of the world holds, of any size, as exact sums of its blocks (3.1a); its card
//!   version is `cell.wgsl` (3.1b).
//! - `store`: the live levels round the camera, cells of 1 to 16 blocks (3.1c).
//! - `bake`: the cells of 32 blocks and up, for the whole world, made once (3.1d).
//! - `card_store`: the cells on the card, where the march reads them (3.2).
//! - `draw`: the first sight, the coarse layer marched through the store and brought up to the screen (3.2).
//! - `textures`: the block textures and their shrunk versions, for the near walk's faces (3.6).

pub mod bake;
pub mod card_store;
pub mod cell;
pub mod draw;
pub mod store;
pub mod textures;

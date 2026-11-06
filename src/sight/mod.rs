//! Sight (the regroup, phase 3, docs/regroup.md): the world as the eye takes it in, from the far view.
//!
//! - `cell`: what a cell of the world holds, of any size, as exact sums of its blocks (3.1a); its card
//!   version is `cell.wgsl` (3.1b).
//! - `store`: the live levels round the camera, cells of 1 to 16 blocks (3.1c).
//! - `bake`: the cells of 32 blocks and up, for the whole world, made once (3.1d).

pub mod bake;
pub mod cell;
pub mod store;

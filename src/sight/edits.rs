//! Edits in sight's live levels (3.6, build 88): the cells of 1 to 16 blocks are made from the recipe on the
//! card, which doesn't know what's been dug or placed, so the cells an edit reaches are made again from the
//! blocks themselves, the changed chunks as they are and the rest as the recipe makes them (the bake's rule
//! for its cells of 32, `bake::column_from_blocks`).
//!
//! - **Only the cells an edit reaches are made again** (`patch_tile`): a block counts its own faces, so an
//!   edited block changes its own cell and the cells of the six blocks beside it, nothing else. Each cell
//!   is its blocks' exact sums, as the card makes them, so a cell made again sits among the card's seamlessly.
//! - **Which blocks are edited** (`Edits::spots`): each change as it's announced, and for chunks changed before
//!   sight started (a saved world), every block that differs from the recipe's (`differences`, on the loading
//!   lane). A spot changed back is still made again, to the same cells: harmless.
//! - **A tile an edit reaches is made again** at every live level (`LiveStore::touch`), the cells on the card
//!   kept till the new ones replace them, so nothing blinks.

use super::bake::Blocks;
use super::cell::{DIRECTIONS, Kind, Sums, sums_of_box};
use super::store::{CellColumn, TILE, Tile, TileKey};
use crate::world::{BlockId, CavePlan, Chunk, ChunkPos, WORLD_HEIGHT};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// The recipe's own chunks (none past the map), made once and shared by every tile made again.
pub type RecipeChunks = Arc<Mutex<HashMap<ChunkPos, Option<Arc<Chunk>>>>>;

/// What making cells again from the blocks needs: the blocks (the changed chunks as they are), the edited
/// blocks by chunk, the caves, and the recipe's chunks.
#[derive(Clone)]
pub struct Edits {
    pub with: Blocks,
    pub spots: Arc<HashMap<ChunkPos, Arc<Vec<[i32; 3]>>>>,
    pub caves: Arc<CavePlan>,
    pub recipe: RecipeChunks,
}

impl Edits {
    /// Whether anything's been edited.
    pub fn any(&self) -> bool {
        !self.spots.is_empty()
    }
}

/// Reads blocks for one tile's making: the changed chunks as they are, the rest the recipe's (asked of the
/// shared ones once a chunk, then kept here).
pub struct Reader<'a> {
    edits: &'a Edits,
    near: RefCell<HashMap<ChunkPos, Option<Arc<Chunk>>>>,
}

impl<'a> Reader<'a> {
    pub fn new(edits: &'a Edits) -> Self {
        Self { edits, near: RefCell::new(HashMap::new()) }
    }

    /// The block at `at`: stone under the world, air over it and past the map.
    pub fn block(&self, [x, y, z]: [i32; 3]) -> BlockId {
        if z < 0 {
            return self.edits.with.blocks.stone;
        }
        if z >= WORLD_HEIGHT {
            return BlockId::AIR;
        }
        let pos = ChunkPos::new(x.div_euclid(32), y.div_euclid(32));
        if let Some(chunk) = self.edits.with.changed.get(&pos) {
            return chunk.block_at(x, y, z);
        }
        let mut near = self.near.borrow_mut();
        let chunk = near.entry(pos).or_insert_with(|| recipe_chunk(self.edits, pos));
        chunk.as_ref().map_or(BlockId::AIR, |chunk| chunk.block_at(x, y, z))
    }
}

/// The recipe's chunk at `pos`, made if it hasn't been (outside the lock: two tiles may both make it, once).
fn recipe_chunk(edits: &Edits, pos: ChunkPos) -> Option<Arc<Chunk>> {
    if let Some(chunk) = edits.recipe.lock().ok().and_then(|made| made.get(&pos).cloned()) {
        return chunk;
    }
    let chunk = edits.with.map.chunk(pos, &edits.with.blocks, &edits.caves).map(Arc::new);
    if let Ok(mut made) = edits.recipe.lock() {
        made.insert(pos, chunk.clone());
    }
    chunk
}

/// The cells (in the level's cells, x, y and up) of a tile that the edits reach: each edited block's own and
/// those of the six beside it, inside the tile.
pub fn cells_reached(key: TileKey, edits: &Edits) -> HashSet<[i32; 3]> {
    let size = key.size();
    let (x0, y0, side) = key.blocks();
    let mut cells = HashSet::new();
    let (from, to) = (ChunkPos::new((x0 - 1).div_euclid(32), (y0 - 1).div_euclid(32)), ChunkPos::new((x0 + side).div_euclid(32), (y0 + side).div_euclid(32)));
    for cy in from.y..=to.y {
        for cx in from.x..=to.x {
            let Some(spots) = edits.spots.get(&ChunkPos::new(cx, cy)) else { continue };
            for &[x, y, z] in spots.iter() {
                for [dx, dy, dz] in std::iter::once([0, 0, 0]).chain(DIRECTIONS) {
                    let (bx, by, bz) = (x + dx, y + dy, z + dz);
                    if (x0..x0 + side).contains(&bx) && (y0..y0 + side).contains(&by) && (0..WORLD_HEIGHT).contains(&bz) {
                        cells.insert([bx.div_euclid(size), by.div_euclid(size), bz / size]);
                    }
                }
            }
        }
    }
    cells
}

/// The cells of `tile` the edits reach made again from the blocks, each cell column's band reaching out to
/// hold them (the cells it takes in between as the band had them: solid all round below, empty above). How
/// many cells were made again.
pub fn patch_tile(tile: &mut Tile, key: TileKey, edits: &Edits) -> usize {
    let cells = cells_reached(key, edits);
    if cells.is_empty() {
        return 0;
    }
    let size = key.size();
    let (x0, y0, _) = key.blocks();
    let (first_x, first_y) = (x0 / size, y0 / size);
    let reader = Reader::new(edits);
    let colours = &edits.with.colours;
    let solid_block = |at: [i32; 3]| {
        let block = reader.block(at);
        if colours.kind(block) == Kind::Solid { block.0 } else { 0 }
    };
    // Each column's blocks apart (a single block's cell's block, by the column's cells).
    let mut blocks = tile.blocks.iter().copied();
    let mut by_column: Vec<Vec<u16>> = tile.columns.iter().map(|c| (0..c.sums.len()).map(|_| blocks.next().unwrap_or(0)).collect()).collect();
    for &[cx, cy, cz] in &cells {
        let i = ((cy - first_y) * TILE + (cx - first_x)) as usize;
        let column = &mut tile.columns[i];
        let column_blocks = &mut by_column[i];
        let (low, high) = (column.low, column.high());
        if cz < low || cz > high {
            // The band reaches out to the cell (an empty band's `high` is `low − 1`: below it solid, above empty).
            let (from, to) = (low.min(cz), high.max(cz));
            let sums: Vec<Sums> = (from..=to).map(|z| column.at(z, size)).collect();
            let kept: Vec<u16> = (from..=to).map(|z| if z >= low && z <= high { column_blocks[(z - low) as usize] } else if size == 1 { solid_block([cx, cy, z]) } else { 0 }).collect();
            *column = CellColumn { low: from, sums };
            *column_blocks = kept;
        }
        let at = [cx * size, cy * size, cz * size];
        let k = (cz - column.low) as usize;
        column.sums[k] = sums_of_box(at, size, &|p| reader.block(p), colours);
        column_blocks[k] = if size == 1 { solid_block(at) } else { 0 };
    }
    tile.blocks = by_column.into_iter().flatten().collect();
    cells.len()
}

/// The blocks of `chunk` that differ from the recipe's (for chunks changed before sight started).
pub fn differences(chunk: &Chunk, edits: &Edits) -> Vec<[i32; 3]> {
    let pos = chunk.pos();
    let Some(recipe) = recipe_chunk(edits, pos) else {
        // Past the map: every block there is an edit.
        return every_solid(chunk);
    };
    let origin = pos.min_block();
    let mut spots = Vec::new();
    for (level, (ours, theirs)) in chunk.sections().iter().zip(recipe.sections()).enumerate() {
        if let (crate::world::Section::Uniform(a), crate::world::Section::Uniform(b)) = (ours, theirs)
            && a == b
        {
            continue;
        }
        let z0 = level as i32 * crate::world::SECTION_SIZE;
        for z in z0..(z0 + crate::world::SECTION_SIZE).min(WORLD_HEIGHT) {
            for y in origin.y..origin.y + 32 {
                for x in origin.x..origin.x + 32 {
                    if chunk.block_at(x, y, z) != recipe.block_at(x, y, z) {
                        spots.push([x, y, z]);
                    }
                }
            }
        }
    }
    spots
}

fn every_solid(chunk: &Chunk) -> Vec<[i32; 3]> {
    let origin = chunk.pos().min_block();
    let mut spots = Vec::new();
    for z in 0..WORLD_HEIGHT {
        for y in origin.y..origin.y + 32 {
            for x in origin.x..origin.x + 32 {
                if chunk.block_at(x, y, z) != BlockId::AIR {
                    spots.push([x, y, z]);
                }
            }
        }
    }
    spots
}

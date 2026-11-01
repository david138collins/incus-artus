# Quartile placement: what it touches, options and order (Builder, 2026-10-31, for David to choose)

**Status: stops 1 to 3 decided (section 8, end); stop 1 built (section 9), waiting for David to look.**

**The decided design** (`docs/decisions.md`, with `Art/quartile minecraft illustration.png`):
- Every cell is 8 **octants** (½ × ½ × ½).
- A **quarter block** is a **bar**, 1 × ½ × ½ (2 octants). It lies along any of the 3 axes, in any of
  4 positions: **12 bar slots** a cell. Bars may share an octant, which makes corners.
- Bars replace stairs, slabs and roof pieces. A stair is 3 bars; a half slab is 2 bars side by side.
- **A cell can mix materials.** Where bars overlap, the bar placed first keeps the shared octant.
  Mining a mixed cell takes all of it.
- **Items sit on face quarters** or a face's centre. Objects protect the blocks they stand against.

**David, 2026-10-31:** "what about the quartile/octant system? we need oak quarter size logs." Trees
need it first: quartile oak makes thin trunks and branches, and the quarter log is the fires' small
log.

---

## 1. What it touches

| Part | Today | With bars |
|---|---|---|
| **Storing cells** | one block number per cell (`BlockId`, 16 bits), in sections of 32³ packed by a palette; each chunk also has a sparse table of extras by position (`BlockExtra`, only a test note so far) | a cell can hold bars of several materials |
| **The mesher** | binary greedy meshing of whole cells, by opacity bit-columns; vertices are 8 bytes, with each position 6 bits an axis (whole blocks, 0 to 32) | bars drawn as boxes at half-block corners; a cell with bars hides nothing behind it |
| **The vertex format and shader** | position in whole blocks; textures tile once a block, from the position | positions in half blocks (7 bits an axis, which still fits the 32-bit word); bar textures (32 × 16 long faces, 16 × 16 ends) need the bar's own corner to measure from |
| **Collision** | the body's box swept cell by cell against `solid(cell)` | a cell with bars is tested against each bar's box; half-block steps need **stepping up**, which doesn't exist yet |
| **Aiming** (raycast) | stepping through cells to the first solid one, and its face | through a cell's bars to the first bar hit, then which quarter of which face |
| **Placing and mining** | `Place(block)` or `Break` at a cell | place a bar (slot, axis, material); the first bar keeps shared octants; mining a mixed cell takes all |
| **The Registrar's block format** | one block, six face textures | whether a block can be a bar, and its bar textures (or a share of the block's own) |
| **Saves** | chunk files by block name, runs of blocks per section (format 2) | format 3: each section's cells with bars added; format 2 files still load |
| **World generation** | blocks only | trees' blueprints put bars into cells as they're made |
| **The far view** | a height sheet made from the recipe; built things aren't shown far away yet | nothing for now; trees are sprites far away; built things wait for "far builds" (`vision/watch-list.md` 17) |
| **Items on quarters** | not built (no items in the world yet) | its own system later; it needs the same "which quarter of which face" aiming |

---

## 2. Options for storing a cell with bars

- **A. Every cell as 8 octants:** double the world's resolution. That's 8 times the memory and
  meshing everywhere, for something most cells never use. No.
- **B. A block number for each shape:** one material's 12 slots give 4,096 shapes, times every
  material. That's far past the 65,000 block numbers, and it can't mix materials. No.
- **C. A marker and a side table** (my pick):
  - **In the section,** a cell with bars holds a reserved number, "bars here". Its palette treats it
    as one more kind, so packing is unchanged.
  - **A side table per section** holds, for those cells only, the bars in the order placed: slot and
    material. That's 1 to 12 bars at 3 bytes each.
  - Placing order is kept, so "the first bar keeps the shared octant" falls out of drawing them in
    order.
  - **Sparse:** a cell without bars costs nothing. A forest of quartile oak might hold a few thousand
    such cells a chunk, which a table copes with.
  - It goes per section, not per chunk (the existing `BlockExtra` table is per chunk), so the
    mesher's job copies a section's bars with its blocks.

---

## 3. The mesher

- **Whole blocks are meshed as now** (greedy, unchanged). A cell with bars counts as not opaque, so
  the blocks round it draw their faces against it. It's like glass to its neighbours.
- **Bars are meshed separately:** each bar is a box of 6 faces at half-block corners. A face is
  dropped where another bar in the cell covers it, or where it's flush with the cell's side against
  an opaque neighbour. There's no greedy merging for bars, which is fine while they're a small share
  of cells.
- **The vertex:** the position becomes half blocks, 7 bits an axis (0 to 64): 21 bits, plus the face's
  way (3) and the random rotation (1). That's 25 of the 32 bits, so the vertex stays 8 bytes. The
  shader halves the position. Everything whole-block is unchanged, just doubled.
- **Textures, two ways:**
  - **A share of the block's own texture** (free): the textures already tile from the position, so a
    bar shows the matching half or quarter of its block. An oak bar shows half of the oak block.
  - **Bar textures of its own** (David's 32 × 16 and 16 × 16 art): the texture is measured from the
    bar's own corner. That needs the bar's corner in the vertex: the quad number's 16 bits in the
    second word are only used by the debug view, so they can carry it.
  - **My pick:** start with the share (no art needed to try it), and add bar textures as an option per
    block, when David makes them.

---

## 4. Collision, stepping up and aiming

- **Collision:** the sweep already asks `solid(cell)` cell by cell. For a cell with bars, the body's
  box is tested against each bar's box instead. Whole cells cost the same as now.
- **Stepping up** (to decide): stairs of bars climb half a block at a time. Without stepping up,
  every stair needs a jump.
  - **My pick:** step up anything up to half a block automatically, when walking into it, if there's
    head room. That's how stairs and slabs feel in most block games.
  - It's new: there's no stepping up today.
- **Aiming:** the ray steps through cells as now. In a cell with bars, it's tested against each bar's
  box. Then the hit point gives which quarter of which face, for placing a bar beside it and later for
  items. The decided "floating cone marker" shows that quarter.

---

## 5. Placing and mining

- **Placing a bar:** from the face and quarter aimed at, the slot next to it, and its axis from the
  way you're facing (or a key to turn it).
  - Into air, or into a cell with bars that has room: the new bar loses any octant already taken.
  - Into a whole block: refused.
- **Mining:** a whole block as now. A cell with bars is taken all at once (decided), dropping its
  bars.
- **Edits** become `Place(block)`, `PlaceBar(slot, material)` and `Break`. The "changed" message
  carries the cell's bars, so saving, the mesher and (later) settlers all see them.
- **Items and inventory:** bars are items like blocks ("oak quarter log"). There's no inventory yet;
  until then, the hotbar picks bars the way it picks blocks.

---

## 6. The order, and the sizes

Four stops for David, each one something to see:

| Stop | What | About (lines) |
|---|---|---|
| 1. See them | the side table, the marker, the mesher's bars at half-block corners (a share of the block's texture), and a debug key to put bars in a cell | 450 |
| 2. Live with them | collision with bars, stepping up half blocks, aiming at bars, placing and mining bars, edits | 500 |
| 3. Keep them | the save format 3 (bars by name), format 2 still loading; the Registrar's "can be a bar" and optional bar textures | 300 |
| 4. Grow them | world generation writing bars (for blueprints); bar textures measured from the bar | 200 |
| Tests throughout | | 300 |
| **Total** | | **about 1,750** |

**Risks:**
- **Speed:** bars aren't greedily merged, so a dense forest of quartile oak is many small quads. It
  needs measuring with a real forest, and could use merging bars along a line later.
- **The vertex change** reaches the shader and the debug view. Everything doubles together, and the
  near/far seam work must be checked again.
- **Collision edges:** a body can catch on half-block edges, and stepping up must not climb walls.
  It needs tests at the edges, like the "walking through block sides" ones.
- **Save format:** a version bump. Old chunk files must still load.
- **The far view:** built things with bars aren't drawn far away until "far builds".

---

## 7. Beside the placer, or first?

**They're independent.**
- **The placer** reaches world generation's fields, mineralogy and the panels.
- **Quartile** reaches storage, the mesher, collision, aiming, edits, saves and the Registrar.
- **They meet only at blueprints:** a tree placed by the placer is made of bars. That needs both, plus
  the asset factory and the Blueprints tab. Either can come first.

**My recommendation:**
1. The placer's first version (mineralogy on it), since David named it most critical and mineralogy
   is waiting on it.
2. Quartile stops 1 to 3.
3. The asset factory and Blueprints, where trees need both.

If trees should come sooner, quartile stop 1 can come first: it's the one to see and judge. The two
don't block each other, and with one Builder, "beside" means taking stops by turns.

---

## 8. Questions for David

1. **Stepping up:** climb half-block steps automatically when walking into them (section 4)?
2. **Textures:** start with bars showing a share of their block's own texture, and add 32 × 16 and
   16 × 16 bar art as an option per block later (section 3)?
3. **Which blocks can be bars:** every block, or only those the Registrar marks (cut stone, boards,
   bricks, logs, dirt)?
4. **Turning a bar:** its axis from the way you face, or a key to turn it?
5. **The order:** the placer first, then quartile stops 1 to 3? Or quartile stop 1 first, to see bars
   sooner (section 7)?

*Counsel's suggested answers (2026-11-01; David: "record them in the notes"; not yet his choices):*
1. **Stepping up:** yes, automatically, up to half a block, with head room, like stairs and slabs in
   most block games.
2. **Textures:** start with a share of the block's own texture; add David's 32 × 16 and 16 × 16 bar
   art per block as he makes it.
3. **Which blocks:** only those the Registrar marks (logs, boards, cut stone, bricks to start), so the
   choice stays David's, block by block.
4. **Turning:** from the way you face, plus a key to turn it.
5. **The order:** the placer's first version is done enough to go on (stop 4, 840af55), so quartile
   stops 1 to 3 next. Counsel's lean: quartile before mineralogy's rock layers and ores, since David
   named the placer and quartile the most critical, quartile depends on nothing else, and rock
   layers are best seen with his rock textures.

**[DECIDED 2026-11-01] Quartile next, with Counsel's five answers** (David: "quartiles, i agree"):
step up half blocks automatically; a share of the block's own texture first, bar art per block later;
only blocks the Registrar marks can be bars; turning by facing plus a key; quartile stops 1 to 3 now,
before mineralogy's rock layers and ores.
- **It must work for octant items too** (David: "lets make sure we are careful it works for octant
  items too"). An octant is one eighth of a cell (½ × ½ × ½); a bar is two. So the cell's side table,
  the slot numbering, the mesher, collision and aiming should all take a single octant as well as a
  bar from the start: 12 bar slots plus 8 octant slots fit in one byte. Whether an octant item is a
  block material in an octant, or a small object resting there (see "Items sit on face quarters"
  above), is for Builder to assess and David to choose.
- **The placing control is David's to design** (David: "ill need to come up with a slick physical
  placing control"). Until he has, placing uses a plain stand-in (facing plus a key), kept separate
  from storage and aiming so his control can replace it without touching them.
- **[DECIDED 2026-11-01] Octant items, joined bars and mining** (David): "an octant *item* would be a
  cup or a candle. but if you join two cut stone blocks (not the large full block versions) you would
  have an L shape. that needs to be implemented seamlessly. mixed shapes will just break and drop
  nothing when mined. but all of them will have collision. the simple control is fine to start with."
  - **Octant items are objects** (a cup, a candle) resting in an octant, not a material filling it.
  - **Joined bars are seamless:** two cut stone bars making an L show no line where they meet. No
    faces are drawn between touching bars, in one cell or across neighbouring cells, and textures
    follow the world position, so the pattern runs straight across the join.
  - **Mixed shapes break whole and drop nothing when mined.** *Counsel:* this refines "mining a mixed
    cell takes all of it" above. Open: does a single-material shape, such as an L of cut stone, also
    drop nothing, or only cells mixing materials? For stop 2, where mining comes in.
    **Answered** (David): "yes, no odd shapes have any chance of giving anything." Then, clarifying:
    "for the player, if he makes some stairs out of a single cut stone and then mines them up, he'll
    get it all back. npc's will probably have a chance based on their mining or some other skill."
    So, as Counsel reads the two together:
    - **One material, mined by the player:** everything comes back. Stairs made from one cut stone
      return what went into them; a lone bar returns itself.
    - **Mixed materials ("odd shapes"):** break whole and drop nothing.
    - **Mined by NPCs:** a chance to get the pieces back, by their mining skill or a similar one
      (rolled through the Goddess of Fate, `docs/simulation.md` section 8).
    Builder confirms this reading with David at stop 2.
  - **Everything has collision:** bars, joined shapes and octant items.
  - **The simple placing control is fine to start with.**

---

## 9. Stop 1, see them: [BUILT 2026-11-01]

Waiting for David to look.
- **The pieces** (`world/pieces.rs`):
  - A slot is one byte: 0 to 11 are bars (the axis × 4 + the place across), 12 to 19 octants.
  - Bars are the pieces of a material. The octant slots are kept for objects resting in an octant (a
    cup, a candle). Those come later, on their own, and will claim their octant for collision and
    against bars.
  - A cell's pieces are kept in the order placed, and the first keeps a shared octant. A bar is only
    placed if it fills at least one free octant.
- **Storage:** a cell of pieces holds the marker `BlockId::PIECES` in its section, so packing is
  unchanged. Its pieces sit in its chunk's side table (by position, as the chunk's extras are).
  Changing the block removes them.
- **The mesher:**
  - Whole blocks are meshed as before. A cell of pieces hides nothing behind it, as glass doesn't.
  - Pieces are drawn by octant: a face shows where no opaque octant covers it, whether in the cell,
    in a neighbouring cell of pieces (across a section's side too), or as a whole block.
  - Faces are merged within the cell, so a bar's long side is one rectangle.
  - **Joined bars are seamless:** no faces between bars that touch, and the texture follows the world
    position, so it runs straight across the join.
- **The vertex:** positions are in half blocks, 7 bits an axis. The shader halves them, so whole
  blocks are as before. Bars show their share of their block's texture. The vertex is still 8 bytes.
- **The edit:** `EditKind::AddPiece { slot, block }` goes into air (or anything replaceable), or into a
  cell of pieces with room. Anything else is refused ("occupied"). Breaking a cell of pieces takes it
  all; what it drops comes with stop 2.
- **The stand-in for placing (for now):** B puts a bar of the hotbar's block into the cell in front of
  the face aimed at, or into the cell of bars aimed at. The slot is chosen on the player panel ("Bars
  (for now)"), going on to the next after each by default, so pressing B again and again fills a
  cell bar by bar. It's kept apart from storage and aiming, for David's own control to replace.
- **Not yet, by plan:**
  - collision with each bar: a cell of pieces is solid as a whole block for now;
  - aiming at a bar, stepping up, and drops: stop 2;
  - saves: a cell of pieces is saved as air until stop 3;
  - the Registrar's "can be a bar": stop 3, so any block can be a bar for now.
- **To look at:** the shading at an L's inside corner (the step and foot shading were made for whole
  blocks), and the seams between bars in neighbouring cells.
- **Tests:**
  - the slots' octants, the first piece keeping a shared octant, and bars only;
  - a bar is a box of 6 rectangles at half-block corners, facing outward;
  - an L has no faces between its bars, and a whole block hides the side against it;
  - the edit into air and into a cell of bars, refusals, and a break taking the cell.

# Quartile placement: what it touches, options and order (Builder, 2026-10-31, for David to choose)

**Status: stops 1 to 3 decided (section 8, end); stop 1 built (section 9); stop 2 built (section 11), waiting for David to look.**

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

**[BUILT 2026-11-01] Bar blocks with pictures of their own, brought forward** (David: "can you create a
new type of block in the library for this, and a registrar entry for it as the first test for a quarter
block? then i can register my own oak tree blocks"):
- **Only marked blocks can be bars** (decided): a block file's `bar` setting (`docs/data-files.md`).
  - `bar = {}` is a share of the block's own textures.
  - `bar = { side = "…", end = "…", only = true }` gives pictures of its own, and only ever a bar.
  - B with an unmarked block is refused ("not a bar"), and placing a bar-only block whole is refused.
- **Its pictures** are in `assets/textures/bars/`: a long side 16 × 32 (its length up the picture) and
  an end 16 × 16, 32 pixels to a block like every block. Each is loaded into its own layer with its
  pixels repeated to fill it.
- **Measured from the bar's own corner:**
  - Each face of such a bar carries its slot (the quad number's top bit and low 5 bits), and the
    shader (`bar_uv`) lays the side's picture along the bar's length, whichever way it lies, and the
    end's on its ends.
  - Where a bar that came first took part of it, what's left shows the matching part of its own
    picture.
  - Such bars aren't merged with each other, and never turned at random.
- **The first: "Stone brick"** (`incus:stone_brick`), from David's `quartile_cut_stone_block_small`
  pictures in `assets/new`.
- **The Registrar's "Load a bar block…":** pick a side picture and an end picture (from Art/ or
  `assets/new`, or drag them onto the window: each goes in by its size), name it, and register. It
  copies both to `assets/textures/bars/` and writes the block file, after an undo step.
- **The hotbar:** the player panel picks any block for the selected slot ("can be a bar" marked). It
  isn't kept between runs yet.
- **David's notes for stop 2:** placing will show a transparent ghost of where the piece goes, and
  bars must be placeable across two cells.
- **To check by eye:** which way each picture faces on each side (mirrored or turned), since that
  can only be seen in the game.

## 10. Bars across two cells: linked half-bars  [DECIDED 2026-11-01]

David asked for bars placeable across two cells. Builder proposed **linked half-bars**, and David
answered the question that went with it: "yes, if you mix bars, it all just becomes nothing when
mined." Builder and Counsel take that as a yes to the proposal too. *(Counsel: inferred from his
answer, not asked outright; he can say otherwise at stop 2.)*
- **A bar shifted half a block is two half-bars,** one octant of material in each cell, each pointing
  to the other. A piece stays one byte: its slot in 5 bits (0 to 11 bars, 12 to 19 octants) and its
  link in 3 (none, or which of the 6 ways its other half lies). Octant slots hold material only as
  linked halves; a lone octant of material can't be placed.
- **Objects** (cups, candles) get a list of their own per cell, each claiming its octant for
  collision and against bars.
- **Drawing:** each half shows its half of the bar's picture, measured from the whole bar's corner
  across both cells; the faces between the halves are hidden, so the bar is seamless.
- **Mining:** breaking either half takes both. One material, mined by the player: one bar back. If
  either cell mixes materials, it all becomes nothing (David); the bar's other half goes too, and its
  cell keeps its other pieces.
- **Collision and aiming** treat each half as its octant.
- **Across chunks** (Counsel): the halves can lie in different chunks and save files, so each half
  draws right while its partner's chunk isn't loaded. Bars the world places (trees) that straddle a
  chunk border come from one plan, so both chunks agree.

---

## 11. Stop 2, live with them: [BUILT 2026-11-01]

Waiting for David to look.
- **The placing control, David's** ("the quarter blocks either snap to the face you're aiming at
  sideways, or longways, and you toggle with a button, like B. the selector box should snap to
  quarter blocks as well when you have a quarter block selection, and show the rotation when you hit
  B"):
  - With a bar block in the hotbar, the selector is the bar's box where it would go
    (`world::bar_placement`). It lies flat against the face aimed at, in the half of the face the
    crosshair is in, and is centred along its length on the crosshair, snapped to half blocks. So it
    runs across two cells when the crosshair is near a cell's side.
  - On a floor or a ceiling, longways points the way you're looking and sideways across it. On a
    wall, sideways lies flat and longways stands upright.
  - B turns it, and right click places it. The player panel's "Bars" section shows which way it's
    turned. It's kept apart from storage and aiming (`PieceChoice`, `aim_bar`), so the control can
    change.
- **Bars across two cells: linked half-bars** (section 10). A piece's byte holds its place (5 bits)
  and its link (3 bits: which way its other half lies). `EditKind::PlaceBar { at, axis, block }`
  takes the bar's corner in half blocks: a bar in one cell, or a linked half in each of two, both or
  neither.
- **Mining:** a cell of pieces breaks whole, and a bar running on into the next cell goes with it,
  its other half taken out there. There are no items yet, so nothing drops. When items come, the
  rule is David's: the player gets back a shape of one material, a mixed shape gives nothing, and
  NPCs get a chance by skill.
- **Aiming:** the ray steps through the world in half blocks (`VoxelWorld::half_filled`,
  `cast_ray_halves`). A whole block fills all eight of its halves, a cell of pieces only its octants,
  so the crosshair meets each bar as it is and passes the empty parts of a cell.
- **Collision:** the same half-block grid, every position and size doubled and halved back (exact):
  bars, L shapes and half-bars are all solid as they are.
- **Stepping up:** walking on the ground into something no higher than half a block, with room
  above, the body goes up onto it (`move_body_stepping`). A whole block's side still stops you.
- **Drawing halves:** each half shows its half of the bar's own picture, measured from the whole
  bar's corner across the two cells (the face's tag carries the link; the shader's `bar_uv`).
- **Tests:**
  - the placing rule (longways, sideways, on floors and walls, across two cells);
  - a bar across two cells as linked halves, broken whole from either cell;
  - stepping up half a block twice, but not into a whole block's side, not in the air, not without
    head room;
  - a half-bar's side, ends and tag.
- **To confirm with David:** that "linked half-bars" is what he meant (Counsel inferred it from his
  answer).

## 12. Mining single bars, cut pieces, and three turns  [DECIDED 2026-11-01, David; replaces "breaks whole"]

David, after stop 2: "you can mine individual slabs in minecraft theres no reason you shouldnt be
able to target and mine quarter blocks, so lets change it: if you have 'cut' a block, its no longer
retrievable. the target box would target the whole quarterblock plus any octblocks that are stuck to
it, having been 'cut' when placed. additionally, when adding a octblock to any other block, it should
highlight exactly the oct itd fill. additionally, the B should cycle between sideways, longways, and
upways, that way the crosshair always knows which oct it should start with."
- **Single bars are aimed at and mined one at a time,** like Minecraft's slabs. This replaces "a cell
  with bars breaks whole" (sections 5, 9 to 11).
- **Cut pieces are lost.** A bar placed where an octant is already taken loses that octant (the first
  piece keeps it, as decided): it's been cut, and a cut piece can't be got back.
- **The target box** takes in the whole bar, plus any cut octants stuck to it from when it was placed.
- **Placing shows exactly the octants it will fill,** a single octant included.
- **B cycles three ways: sideways, longways, upways,** so the axis is always chosen, and the crosshair
  always knows which octant the piece starts from.
- *Counsel:* no objections. Each octant belongs to exactly one piece (the first keeps it), so aiming
  can go from the octant under the crosshair to its piece cleanly. What "stuck to it" means exactly,
  and whether mining an intact bar with cut pieces stuck to it returns the bar, are for Builder to
  ask David directly (he'll answer her questions).
- **[IDEA, David 2026-11-01] Every octant a stick-on:** "this may simplify storing block data, as any
  oct can just be a stickon." *Counsel:* the octant becomes the only unit. Each octant of a cell holds
  a material and a link to the octant it's joined to:
  - a bar is two linked octants;
  - a bar across two cells is the same thing over a cell border;
  - a cut piece is an octant stuck on to the piece that cut it;
  - mining follows the links.

  The link's direction along a bar also gives its axis and which end it is, which the bar's picture
  needs. A cut piece, whose link points elsewhere, may need its axis stored too. It would replace bar
  slots plus links with one rule. For Builder to assess before stop 3's save format, and for David to
  choose.
- **[DECIDED 2026-11-01] Stick-ons, go** (David: "if the player has to mine an oct they can cry me a
  procedurally generated river that looks real, send it"). The stick-on format is a go. A leftover
  lone octant that has to be mined by itself is acceptable. *Counsel:* read as no need to engineer
  around the edge case; the target box still takes a bar's cut pieces with it, as he first asked.

---

## 13. Stick-ons, built: [BUILT 2026-11-01]

David's rules after stop 2 (section 12), and the stick-on format he chose ("if the player has to mine an
oct they can cry me a procedurally generated river that looks real, send it"). Waiting for David to
look.
- **The format** (`world/pieces.rs`): a cell of pieces holds, for each of its 8 octants, its material
  and a `Part` byte: its bar's axis (or none, a lone octant), which end of the bar it is, and
  whether its other half is there. That replaces bar slots and links (sections 9 to 11).
  - A bar is two octants pointing at each other, in one cell or across a cell's side.
  - A cut piece is an octant whose other half was taken. It's "stuck to" whatever holds that octant.
  - A lone octant belongs to no bar.
  - Each half knows whether its other half is there, so it draws right even when the other half's
    chunk isn't loaded.
- **Placing** (`EditKind::PlaceBar`): a bar fills its two octants. If one is already taken, the bar
  is cut: its other octant goes in as a cut piece. If both are taken, it's refused.
- **The selector:** placing outlines exactly the octants the bar would fill (one, for a cut bar). The
  piece aimed at in a cell of pieces is outlined, not the whole cell: a bar and the cut pieces stuck
  to it, or a lone octant.
- **Mining** (`EditKind::BreakPiece`, `VoxelWorld::piece_halves`): one piece at a time, with its
  bar's other half (in this cell or the next) and the cut pieces stuck to it. A cut piece mined
  alone is just itself. Breaking a whole cell of pieces (any other way) takes each piece in it.
  There are no items yet, so nothing drops. When items come, a cut piece can't be got back.
- **Drawing:** a face's tag is its octant and part. The shader (`bar_uv`) measures the bar's picture
  from where the bar starts: the octant's place, less half a block for its high end. So one rule
  draws a bar in one cell, a bar across two, and a cut piece (its part of the picture). A lone
  octant shows its end picture on every face. A bar's two octants in one cell share a tag, so its
  sides merge into one face.
- **Edge case, decided:** a cut piece goes with what it's stuck to. So nothing new can take its
  blocker's place while it's there, and "stuck to" never changes to something that didn't cut it.
- **Tests:** a part's axis, end and whole; the first piece keeping an octant; the placing rule; a bar
  crossing one already there, cut, and mined with it or alone; a bar across two cells mined whole
  from either; a broken cell taking its pieces and their halves; tags and pictures for bars, cut
  pieces and halves.
- **B three ways, from the crosshair** (David: "B 3 ways. sideways, longways, upways. i think that'll
  simplify things. when things are upways, you can treat left as its 'bottom' for the time being, we
  may evolve on that?"; Minecraft's way of placing by the way you face):
  - A piece starts at the octant in front of the face at the crosshair (`piece_start`), so the
    crosshair always knows its octant.
  - A bar runs one octant on (`bar_placement`) along a line: upways upright, longways the way you're
    looking, sideways across that.
  - **Which way along the line is the half of the octant the crosshair is on** (David, 2026-11-01:
    "i notice i have to rotate my body to place them going the other direction; can we have the
    octfaces split so the left or right side flips the quarterblocks orientation?"). Aimed at an
    octant's right half, sideways runs right; at its left half, left. The same goes for longways on
    a floor (far half away, near half toward you) and upways on a wall (top half up, bottom half
    down). The ghost flips as the crosshair crosses the middle. This replaced "sideways always to
    your right".
  - A line straight through the face aimed at goes out of it: longways on a wall sticks straight
    out, upways on a floor stands up, and upways under a ceiling hangs down.
  - This replaces centring the bar on the crosshair (section 11). A bar still runs across two cells
    when its start is at a cell's side.
- **Octblocks** (David: "honestly i think the stone cutting action should be done on screen with a
  chisel on the ground, and so there's no reason octblocks shouldn't be a thing"): Shift + right
  click places a single octant of a bar block at the start octant (`EditKind::PlaceOctant`), a
  stand-in until there's a chisel and an inventory (`docs/vision.md`, "Building, resources and
  labor"). A bar across an octblock is cut by it, and mining the octblock takes that cut piece too.

---

## 14. Stop 3, keep them: [BUILT 2026-11-01]

- **Saves keep pieces** (chunk format 3, `world/storage.rs`). A cell built of pieces is air in its
  section, and a list after the sections puts it back: each cell's place, which octants are filled,
  and for each its block (by name, like every block in a save) and its part (bar axis, end, whole).
  - Format 2 files still load, with no pieces (they were saved as air). A game from before this
    can't read format 3 files.
  - An octant of a block that's gone from the game is left empty, and a cell left with none is air
    (the missing name is reported, as for whole blocks).
  - The list is written in order, so the same chunk always saves the same bytes.
- **The Registrar's "can be a quarter block"** (a block's settings in the dock): any block can be
  marked, with "only ever a quarter block" (cut stone), and a bar side and end picture chosen from
  `assets/textures/bars/`, or "(a share of its own)". Greyed out until it's marked. A side without
  an end (or the other way round) is warned about. Saved to the block file as `bar = { … }`
  (`docs/data-files.md`), and the game picks it up without a restart.
- **Tests:** a chunk keeping its pieces through a save (a whole bar, a cut piece, a lone octant, a
  cell at a chunk's corner), saving the same bytes twice, and a lost block's octants going empty.
- **Next, stop 4 (grow them):** world creation writing pieces, for blueprints and trees.

## Whole blocks on the octant grid  [IDEA, David 2026-11-01; for Builder to assess]

David: "we should be able to place whole blocks on the oct grid, as well. we're trying test caves
now, but i'd like to work that in immediately if its not too intensive."
- A whole block shifted by half a block in any direction covers 8 octants across 2, 4 or 8 cells.
- *Counsel:* the stick-on format may take it with one new kind of part: "a corner of a whole block",
  3 bits for which corner (the part byte has 4 spare). Each octant then shows its eighth of the
  block's picture, measured from the block's corner. Mining takes all eight. If some octants are
  taken, either refuse or cut, as bars do; David's call. Saves only meet a new part kind, so format
  3 should take it without a bump.
- Builder assesses how much it costs, and does it straight away if it's light (David's word), once
  he's done trying the test caves.
- **[David 2026-11-01] Shift frees you from the main grid:** "i think for now, shift should free you
  from the main grid?" So holding Shift places a whole block on the octant grid. *Counsel:* Shift +
  right click is today's stand-in for a lone octblock, so that stand-in needs another key.
- **[DECIDED 2026-11-01] Only blueprints and players make sub-block pieces** (David: "note that non of
  the non-living parts of the world are being considered to be sub-block elements. only blueprint
  stuff and player stuff."). The ground, rock, caves and water are whole blocks. Bars and octants come
  only from blueprints (trees, buildings) and from what players and settlers build. So caves carve
  whole blocks, and world creation writes pieces only when it places a blueprint (stop 4).

## 15. Whole blocks on the octant grid, built: [BUILT 2026-11-01]

It's built as Counsel guessed, about 250 lines, with format 3 unchanged.
- **A new part kind** (`Part::of_block`, `Part::corner`): bit 4 set, and bits 5 to 7 say which corner
  of the block this octant is (`x + 2y + 4z`). Its axis bits are 0, so it's no bar, never cut, and
  has no other half.
- **Placing** (`EditKind::PlaceShifted`, `place_shifted`, `pieces::block_placement`):
  - It starts from the octant in front of the face at the crosshair, then goes one octant on along
    every axis: straight out of the face, and across it toward the half of the octant the crosshair
    is on, the same split-face rule bars use.
  - So it lands half a block off the cells along one or two axes (2 or 4 cells), or lines up with
    them, when it's an ordinary block in the cell in front of the face. (Three axes, across 8
    cells, happens only from a placement made in code.)
  - Any block can be placed this way, except one that's only ever a bar.
  - **Refused if any of its 8 octants is taken** (for now; David's call, below).
- **Mining** (`VoxelWorld::piece_halves`): from any of its octants it takes all 8 (found from its
  corner), with any cut pieces stuck to them.
- **Drawing:**
  - Each octant gets a tag saying which ways its block is off the cells (`bar_tag`: the corner XOR the
    octant). That's the same for all of a block's octants in a cell, so their faces merge.
  - The shader slides the block's own face picture by half a block along those ways, so the picture
    lines up with the block, not the cell. It's never turned, and a block's bar pictures aren't used.
- **The controls** (David: "i think for now, shift should free you from the main grid?"):
  - **Shift + right click** places the block in hand whole on the octant grid. The selector outlines
    its 8 octants.
  - **The octblock stand-in moved to Alt + right click.**
  - The controls line along the top and the Player page say so.
  - Shift is also run while walking, so placing while running puts blocks off the grid.
- **Tests:**
  - where it goes, toward the crosshair's half, out of a wall, or lined up;
  - the part knowing its corner;
  - 8 cells an eighth each, refused when taken, and mined whole from its far corner;
  - two cells of four, and lined up as an ordinary block;
  - the mesher drawing its own face, slid, merged in its cell, and never turned.
- **For David:** when some of the 8 octants are taken, refuse (as now) or cut, the free ones going in
  as pieces stuck to what's there, as bars do?
- **Overlap: deferred by David until he's tried shifted blocks** ("i think we can leave the overlap
  question over for now, i might have a question about the way shifting whole blocks works after
  trying it"). Bars stay as they are.
- **[IDEA, David 2026-11-01] L pieces, if they're needed:** with no overlap, an L is a bar plus a lone
  octant, and with bar art the two may not match ("you'd be using a quarter piece on one side of a
  corner, and an oct on the other"). David: "i think maybe we can just make L's into blocks too, if we
  need to... we could just make the stone cutting minigame let you hunk an oct off the corner of a
  slab, maybe." *Counsel:* see it first. A bar plus an octant of a block with no bar art already
  joins seamlessly, since both measure the block's picture by world position. If a bar-art L looks
  wrong in play, an L piece (3 octants, one picture) cut with the chisel is the answer.

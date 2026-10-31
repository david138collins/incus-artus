# Rivers, from beginning to end

How the rivers work as built (through 926e66b, 2026-10-30), written for David and Counsel; with a
review of the shores and banks (every shore a block high), and of Génevaux et al. 2013, "Terrain
Generation Using Procedural Models Based on Hydrology", and how we differ. The plans and decisions
behind each part are in `worldgen.md` ("Breaking the water loop", "The river rework", "Carving").

**Contents:** 1. The pipeline at a glance · 2. Each step in detail · 3. In the world · 4. Why every shore
is a block high (and everything else that sets a shore's height) · 5. The paper · 6. How we differ, and
what's worth borrowing · 7. Open questions for David.

---

## 1. The pipeline at a glance

Rivers live in **Climate** (the World Compiler's stage between the Additives and the World Interpreter)
and are carved by the **Interpreter**. The pipeline is linear: nothing later feeds back into anything
earlier, so the rivers are made *from* the ground but never change the ground Climate reads.

```text
the recipe (the Interpreter's ground)
  │  read twice, without the rivers:
  ├─ broad ground (no crags, summit crags, hills, torn coast) ──► temperature ─► wind ─► rain
  │                                                                  │              (for Ecology,
  ▼                                                                  ▼ snowmelt      not rivers)
  step 5  THE FLOW       basins fill and spill; every 32-block patch drains somewhere; water adds up
  │                      downhill (a unit a patch, plus snowmelt and slope-foot springs; not rain);
  │                      lakes where the filled water stands 2+ blocks deep
  ▼
  step 6  THE NETWORK    which of those paths are rivers: springs, mouths, grown up from the mouths,
  │                      stream order → rank (creek, stream, river, great river), a stepped bed
  ▼
  step 7  THE CURVES     each river as a smooth line with width, depth, water level all along it;
  │                      water levels laid from the REAL ground (hills and crags in), deltas
  ▼
  the World Interpreter  each column: inside a river? cut to the channel floor, water to the level;
                         in or beside a lake? water to its surface, crag dips filled. Saved as rivers.txt
```

Two grids matter: the **patch** (the compiler's pixel, 32 blocks) that steps 5 and 6 work on, and the
**block**, where the curves are carved. A climate pixel (temperature, wind, rain) is 4 patches, 128
blocks.

## 2. Each step in detail

### Step 5: the flow (`src/worldgen/rivers.rs`)

- **The ground it reads** is the broad ground on the 32-block grid: the recipe's big shapes (curves,
  ridgelines, rise, peaks), without the fine layers, because crag noise makes thousands of false pits.
- **Fill and spill (priority flood).** Start from the sea and the map's edge; always take the lowest
  ground not yet reached next; whatever it reaches drains into it. A dip with no way out is only reached
  over its rim, so it fills to the rim's lowest point and spills there. Result: every patch has one
  patch it drains into (8 neighbours, diagonals included), always downhill or across a lake, always
  ending at the sea. Ties on flat ground are broken by a fixed nudge from the seed, so paths wander
  instead of running in grid lines.
- **Each patch's water:** 1 unit for a patch of land, plus **snowmelt** on cold high ground (colder
  than −0.2, above Z 300: +1 unit), plus **springs where a slope eases off** (the steepness coming down
  to it minus the steepness going on). Water adds up downstream. *[2026-10-30, David: rivers aren't
  driven by rain. It was each patch's rain, with snowmelt and springs as shares of it. The flow is
  now keyed on the temperature and the ground, so changing the rain no longer relays the rivers.]*
- **Lakes:** patches whose filled water stands at least 2 blocks deep (`lake depth`). Fresh lakes only;
  every basin spills (salt lakes later).

### Step 6: the network (`src/worldgen/network.rs`)

Step 5's drainage, read upstream, is a tree for every mouth. So choosing rivers means **choosing
branches of that tree**: every river drains, stays in its own catchment, and its bed can only fall
downstream, without having to check.

1. **Springs.** Candidates are land patches, not in a lake, that are high (by plain height, Z 260 to 700;
   or, with the toggle, at least 40 blocks above the average within 2,000 blocks) with water enough
   (their own water at least 1 unit, which is any land; raise it to keep to snowmelt and slope feet). Big lakes' outlets (20+ patches) go first, then the rest in shuffled
   order. Each one is kept only if no kept spring is within its room: 2,500 blocks, less by up to half
   with the jitter (even, but not too even).
2. **Mouths.** Each spring's path ends at a mouth. Mouths closer than 3,000 blocks: the busier one
   keeps its springs; the other's springs lose their river.
3. **Growing up from each mouth.** The main branch (the most springs above it) always grows; a side
   branch grows if it carries at least (1 − branching) of the main branch's springs. Branching 0.5:
   half as many.
4. **Springs no river took** run down as short streams until they join a river, reach a lake or the sea,
   or fizzle out (after 3,000 blocks at the minimum water, further with more snowmelt or slope-foot
   water).
5. **Back down to conclude.**
   - **Stream order (Strahler):** 1 where a branch begins; where two of the same order meet, one up;
     otherwise the bigger carries on. With one order per rank: order 1 is a creek, 2 a stream, 3 a
     river, 4 and up a great river.
   - **The bed**, from each river's lowest patch upward: it stays flat until the ground has risen
     4 blocks (`step`) above it, then steps up to the ground. A step 3 blocks tall or more
     (`fall_from`) is a waterfall; shorter ones are rapids.

Note: the network's bed is no longer the water level (step 7 lays that from the real ground). It still
decides the meanders' flatness, the gorge pull-back and whether a mouth gets a delta, and it's what the
Rivers overlay colours as waterfalls (white) and rapids (pale). **So the overlay's waterfalls aren't
the world's.**

### Step 7: the curves (`src/worldgen/carving.rs`)

- **Size** (section 8, step 1, built 2026-10-30): a spring's size (2 wide, 1½ deep), times a width
  and a depth ratio for each step up a rank. Each rank's band meets its neighbours: creek 2 to 9½
  wide, stream 9½ to 24, river 24 to 59, great river 59 to 148. A river grows down its band as it goes:
  how far along its rank's run a patch is (the longest way up while it stays that rank, against the
  way down) is how far down the band. A reach ending at a junction keeps its own size up to it.
  "Size varies" (0.15) wanders it slowly by place (600-block noise). *[It was the middle of the old
  width table, full for the first half from the mouth ("keeps full size for", now gone), with depth
  √(2 × width).]*
- **Shape across:** each rank's edge depth, flat floor and bowl (creek 0.3, 0.2, 1; stream 0.5, 0.5,
  0.6; river and great river 0.2, 0.3, 1.6), halfway to its neighbours' at the ends of its band, and
  wandering with "shape varies" (0.3). Each column's depth is `depth_across`: full over the flat
  floor, then the bowl's curve up to the edge depth, rounded to whole blocks.
- **Reaches:** each river is cut into stretches from a spring or junction down to the next junction, a
  lake, or the sea. A stretch's ends are **pinned**, so rivers always meet.
- **The route** (`route`, 2026-10-30, section 9): the network's path is a plan on the broad ground,
  with no hills in it. Each reach finds its way on the **real ground** (the recipe without rivers)
  through a corridor of 4-block cells (`grain`) within 64 blocks (`corridor`) of that plan, by fill and
  spill from its end: the lowest cell not yet reached is always taken next (on level ground, the one
  nearer the end), so a dip fills to its lowest gap and spills. The route is the way the start's water
  then runs: round the hills, through the lowest gaps, along the valley floors. Its ends are the
  plan's, pinned. A very long reach uses a coarser grain, to keep its grid under 4 million cells. The
  reaches are routed on all cores. The panel shows how long it took, and the deepest cut below the
  banks (with how many points cut deeper than 6 blocks).
- **The line:**
  1. **Corner cutting.** The route is taken at points about a river's width apart (at least 2 cells,
     at most 48 blocks), then Chaikin's method slices every corner off: a creek 2 passes, one more per
     rank up (at most 5). So bigger rivers bend wider.
  2. **Resampling,** every 2 to 6 blocks.
  3. **Moving each point sideways,** by the sum of three things:
     - a **wander:** 12 blocks, in swings of about 400 blocks;
     - a **lean** to the outside of the bend: half a width at a sharp bend;
     - a **meander:** a wave every 12 widths, 1 width big, full on ground flatter than 2%, fading on
       steeper.

     Together they're faded to nothing within max(2 widths, 48 blocks) of each end, and capped at
     48 blocks. Wherever the moved point's real ground would be more than 3 blocks above the ground
     under the route (`floor_rise`, "keep within"), the move is halved, up to 4 times, then dropped.
     So the squiggles stay on the valley floor. *[It was "deepest gorge", 6 blocks below the broad
     ground, which can't see hills.]* The meanders' flatness is the real ground's slope from end to
     end.
- **The water level** (`lay_water`, the 926e66b fix):
  1. **From the real ground.** At each point, sample the real ground (the recipe with hills and crags,
     without rivers) at three spots: the middle of the line and both edges of the river. The level
     starts as the lowest of the three, rounded down.
  2. **Only falling downstream.** Stretches are done from upstream down; a stretch starts no higher than
     what flows into it.
  3. **Never below the sea**, and a stretch into the sea or a lake ends at that water's level.
  4. **A stretch ends at the water it joins.** Working downstream first, a stretch that ends at a
     junction ends at the level of the river it joins. Each stretch's level comes from the ground
     across its own width, so a creek could otherwise end blocks above its river.
  5. **Steps, relative to the ground** (the step: 4 blocks). From a stretch's end upward, the water
     stays flat until the ground under it has risen a step, then rises to it, as the network's bed
     does. *[Fixed 2026-10-30: the levels were rounded down to absolute multiples of 4, Z 128, 132,
     136…, which put the water up to 3 more blocks below its banks. See section 4.]*
  6. **Drops.** A drop of 3 blocks or more is a waterfall, sheer, with a one-block curtain of water down
     its face if the switch is on. Smaller drops are spread upstream as rapids, 3 blocks of river per
     block of drop, stepping down a block at a time. Every step is at least the step size (4), and
     that's taller than "a waterfall from" (3), so at the defaults every step is a waterfall. Rapids
     only come from smaller drops, such as a creek's last bit down to its river.
- **Deltas:** where a river or great river reaches the sea and its last 64 or 128 blocks fall by less
  than 1%, the last stretch fans into three channels, each a third as wide, all at sea level.
- **Lakes, filled on the real ground** (`fill_lakes`, 2026-10-30; they were step 5's surface on
  each 32-block patch, raised square pools with square walls round them). Step 5's lake patches,
  grown by 64 blocks (`lake_reach`, "look round each lake"), are joined into regions where they touch.
  Each region's real ground is read on 4-block cells (the route's grain) and filled and spilled from
  its edge, as the routes are: each dip fills to its lowest gap, whatever its shape. A pool at least
  1 block deep at its deepest (`pond_depth`, "a pond from") is kept. So a planned lake can come out
  smaller, larger, or as several ponds, and dips near it can become ponds. The panel counts the lakes
  and ponds. A river into a lake ends at its real water level. If the real water doesn't reach the
  river's end, the river just ends there. The network still plans with step 5's lakes (springs at big
  lakes' outlets).

## 3. In the world (`src/world/carving.rs`, `interpreter.rs`, `terrain.rs`)

- **Saved with the world** as `rivers.txt` (every point of every stretch, and the lake surfaces).
- **Each column** (`Carving::here`) finds the nearest piece of river line through a lookup grid.
  - **Inside the river** means within half its width of the line. For an even width, the odd block goes
    to the outside of the bend. There the ground is cut down to the **channel floor**, the water level
    minus 1 minus the depth, and water fills from the floor up to the level. Just below a waterfall,
    the curtain raises the water to the level above the fall.
  - **In a lake's water cell**, water stands to the surface wherever the ground is more than half a
    block below it. Each lake is a fine grid of its own, found through a lookup grid.
  - **In a dry cell next to the water**, a dip below the surface is filled up to one block under it,
    so water never stands against air. (Worlds saved before 2026-10-30 read their lakes back as one
    grid of 32-block cells.)
  - **On a river's bank** (`Banks::ground`, profiles step 3, built 2026-10-30): beyond the water,
    within the point's fade, the ground is pulled to the **shore** and faded back to its own height.
    The shore is flush (the ground block at the level − 1, its top at the water), or a block higher
    for a ledge. Where the land is higher, it's cut. Where it's lower, it's raised (a levee), so the
    water never stands against air.
    - **Fade width:** one width each side for every rank (`fade_widths`), at least 2 blocks
      (`fade_least`), and half that on the outside of a bend (`outside`). Where the fade would be
      steeper than 1 block up per block across (`steepest`), it widens, up to 3 times (`widest`).
      Past that it stays steep, a gorge.
    - **Way, by the bank's side:** a point bends when its width over the bend's radius (`turn`, saved
      with each point) is at least 0.15 (`bend_from`). The way is chosen by side: the inside of a bend
      is **smooth** (an S-curve), the outside is **ledge then slope**, and a straight run is
      **smooth**. **Slope** (a straight ramp) and **terraces** (flat steps 4 blocks high) are the
      other choices.
    - The settings are kept with the world (a `banks` line in rivers.txt). Worlds saved before have
      no fade.
    - Each river piece is entered in every 64-block lookup cell its banks reach.
- **The carve lowers the recipe's height itself** (`ground_and_water`), so the far terrain, the light
  map and the World Compiler's close-up all see the rivers. The far terrain also blends a creek's water
  in by how much of a far cell it covers, so creeks don't pop.
- **Blocks:**
  - **Rounding:** each column's ground is rounded to a whole block.
  - **The sea** fills every block below Z 128, so its surface is at Z 128. Sea floor is kept at least
    one block under it (`under_sea`: rounds below 128 → sea, floor no higher than Z 126).
  - **Rivers and lakes** fill from the ground up to below their level. River and lake beds are gravel
    (or sand).

## 4. Why every shore is a block high

### The main cause: how a block's height is counted

A column's ground rounds to a whole Z, and the **ground block sits at that Z**, so its top face is one
higher. Water fills the blocks **below** its level, so its top face is **at** the level. For a shore
to be flush (the land's top face level with the water's), the shore's ground block must be at **level − 1**.

- **The sea.** Its surface is Z 128. Any column whose ground rounds to 128 or more is land, with its
  top face at 129 or higher. Any column that rounds to 127 or less is sea, pushed down to 126 so there's
  water over it (`under_sea`, made the single rule for near and far on 2026-10-30). **A flush shore,
  a dry block at Z 127, is impossible by definition.** The smallest step from the sea to land is exactly
  one block. The recipe adds to it: on a plate's low side, land meets the sea at Z 128 (the land curve
  starts there), and the coast pad, the peaks' coast fade and "detail keeps the coast" all hold the
  ground near the coastline at about 128. So almost every low coast lands on that minimum: **one block,
  everywhere.** That matches what David sees.
- **Rivers.** The same counting, plus three things that drop the water further below its banks:
  1. **Rounded down from the lowest of three samples.** That's 0 to 1 block more.
  2. **Rounded down to multiples of 4** (the step). That's 0 to 3 blocks more. The levels snap to
     absolute multiples of 4 (Z 128, 132, 136…), not to the ground: a river whose banks sit at 135 has
     its water at 132. On flat ground this alone makes river shores 1 to 4 blocks high, about 2½ on
     average. **I think this is a fault, not a style:** the network's own bed does it properly (flat
     until the ground has risen a step), and the water should do the same. *[Fixed 2026-10-30: steps
     are now relative to the ground. Within a flat run the water still sits up to 3 blocks under the
     ground where the ground rises toward the next step.]*
  3. **Only ever falling.** Where the ground rises along a river, the water stays at the lower level
     and the banks climb into a gorge (up to 6 blocks before the line is pulled back). That's
     intended, but it's another source of tall banks.
- **Lakes are the one place shores can already be flush.** A lake patch's ground that rounds to one
  under the surface stays dry with its top face at the water, and filled dips are filled to one under.
  But a lake's surface comes from the broad ground (no hills, no crags), on 32-block patches, so it can
  sit well above or below the real ground at its edges.

### Everything else that sets a shore's height (to be thorough)

| Where | What | Effect |
| --- | --- | --- |
| All water | Water fills below its level, ground blocks sit at their Z | Minimum shore is 1 block (the main cause) |
| All water | Each column's ground rounded to a whole block | A shore's height jumps a block at a time; no slopes in between |
| All water | Water drawn as full cubes (no partial water levels yet) | Nothing to soften a 1-block lip by eye |
| Sea | `under_sea`: 127 is sea, pushed to 126 | No flush land at 127; the sea is always 1+ deep at the shore |
| Sea | Land curve starts at Z 128 at the coast | Low coasts sit right on the 1-block minimum |
| Sea | Coast pad, peaks' coast fade, detail keeps the coast | Hold the ground at the waterline, so the 1-block step is uniform |
| Sea | Edge rise (coast rise, fall back) on plate high sides | Taller shores and cliffs by design |
| Sea | Torn coast | Moves the coastline in a band; its own heights near the water |
| Sea | "Crags never drown land" flats (127.5 to 128) | Become land at 128 (1 block up); the open sand-flats question |
| River | Level = rounded down, lowest of 3 samples | 0 to 1 block more |
| River | Flat runs between steps (4) | 0 to 3 blocks more, where the ground rises toward the next step (was snapped to multiples of 4, fixed 2026-10-30) |
| River | Water only falls downstream | Banks rise into gorges where the ground climbs |
| River | Gorge pull-back: 6 blocks deep allowed | Rivers may sit up to 6 under their banks by design |
| River | Rapids spread upstream | Lowers the water above each drop |
| River | The cut is vertical; the ground beside it is untouched | No bank profile at all: shore height is whatever the ground is next to the channel |
| River | Only 3 samples across the width | On a side slope, a column just outside can be below the water: a water face against air (rare) |
| River | Meanders and wander move the line up to 48 blocks | Onto ground higher or lower than the path's |
| Lake | Surface from the broad ground, per 32-block patch | Shores can be cliffs or flush depending on hills and crags |
| Lake | Ring fill to one under the surface | Flush aprons where crag dips were |
| Look | Beach sand by Z (to become "by distance from water", decided) | What the shore is made of, not its height |
| Far | Far terrain uses the same `under_sea` and carve | Agrees with near, so any fix must change both together |

### What fixing it might look like (for David to decide, not decided)

- **A waterline.** Allow a dry block whose top is flush with the water (Z 127 for the sea, level − 1 for
  rivers and lakes). For the sea, this means `under_sea` gets a third answer, "shore", and the near and
  far terrain must agree on it.
- **Banks as a cross-section, not a cut.** Each river carries a profile across it: the channel, then
  the bank, which slopes or steps back up to the ground over some distance. The ground beside the river
  is blended toward that profile instead of left untouched. This is what the paper does (section 5),
  and it's the real answer to banks.
- **Cut banks and point bars.** We already know which side is the outside of every bend:
  - the outside gets a **ledge or cliff** (a cut bank);
  - the inside gets a **flush, gently sloping beach** of sand or gravel (a point bar).

  That's how real rivers look, and it gives "a ledge some places" a reason.
- **Steps relative to the ground,** like the network's bed, instead of absolute multiples of 4.
- **Shore height as a control** by context: the land's slope behind the shore, the river's rank, which
  side of the bend, and later the rock (mineralogy).
- **Graphics pass, later:** a water top a little below the block's top, so flush shores read with a
  small natural lip.

## 5. The paper: Génevaux, Galin, Guérin, Peytavie, Beneš (2013)

*ACM Transactions on Graphics 32(4), SIGGRAPH 2013.* The idea: **real terrain is organised around its
rivers, so make the rivers first and the terrain from them.**

- **Input:** the outline of the land, optionally the river mouths and sketches of the main rivers, and
  two maps:
  - a **river slope map**, how steeply rivers climb from their mouths;
  - a **terrain slope map**, where mountains and flatlands are, a slope of 0 to 0.25.
- **Growing the network** (a graph of nodes a fixed distance apart, grown inward from the mouths):
  - **Choosing which node grows next.** Take the lowest candidate; consider those within ζ of its
    height; pick the one with the highest priority, then the lowest. A small ζ grows the lowlands
    first, with many similar basins. A large ζ lets big rivers run long and dominate.
  - **Priority is the Horton-Strahler number,** coded into a little grammar:
    - continue with no branching (probability Pc);
    - symmetric junction: two children of order n−1 (Ps);
    - asymmetric junction: one of order n and one of order m < n (Pa).

    With Ps high, more than 75% of streams are order 1 and there are few main rivers. With Pa high,
    there are long main rivers and basins of similar size.
  - **Checks on each new node:**
    - it stays 3/4 of an edge length from the coast;
    - its edge stays apart from other edges, bigger rivers further apart;
    - it's higher than its parent (by the river slope map);
    - it never climbs too steeply (a Lipschitz limit, so no huge cliffs).
- **Watersheds and ridges.** The terrain is cut into Voronoi cells around the river nodes. Each cell
  edge that doesn't cross a river is a ridge. A ridge point's height is the highest of its three nearest
  river nodes plus λ × distance, where λ comes from the terrain slope map. So every ridge stands above
  its rivers, and water must flow down to them.
  - **Flow** comes from upstream area: Q = 0.42 · A^0.69 m³/s, an empirical law that includes losses.
- **River types (Rosgen's 1994 classification),** by slope and nearness to the coast:
  - A, B: steep, entrenched, step-pool;
  - C, E: meandering on flood plains;
  - D, DA: braided and many-channelled near coasts;
  - F, G: entrenched and gullied.

  Each type has a trajectory style and a **cross-section profile**, with layers for bedrock, water and
  sand. Mouths with enough flow are marked as deltas.
- **Building the geometry:**
  - **Junctions,** per cell, with angles by flow: unequal rivers meet nearly perpendicular, equal ones
    at a small angle.
  - **Paths,** subdivided and randomly bent by type.
  - **Terrain primitives:** about 50 per cell, Poisson spread. Each one's height blends between the
    nearest river point and the nearest ridge, with more noise further from rivers and in high relief.
- **The terrain is a function, not a grid:** a construction tree, Replace(Blend(terrain pieces),
  Blend(river pieces)).
  - Each piece has a soft, limited reach: weight (1 − d²/r²)³.
  - A river piece is its curve plus its profile: height = the curve's height + profile(distance
    across).
  - Replace blends the river into the terrain by the river's weight, so **banks merge smoothly into
    the land around them.**
- **Results:**
  - A few seconds for several hundred km²; about 1.5 kB per km².
  - Hoya Island: 3,368 km², 81,853 pieces, about 8 MB.
  - Evaluated at any resolution.
- **Limitations, by the authors:**
  - it only makes land shaped by water erosion;
  - the network only splits upstream, so **no deltas or oxbow lakes**;
  - greedy growth makes wandering crests and odd hills;
  - rivers can't adapt to big mountains with clear valleys, and with strong slope-map changes, flat
    rivers cut through high mountains;
  - there are many parameters, and odd terrain is easy to make.

## 6. How we differ, and what's worth borrowing

**The big difference is the order.** They make **rivers first, terrain from rivers**: valleys exist
because rivers made them, and ridges are built to stand above the rivers. We make **terrain first**
(plates, Additives, the recipe), then **find the water on it**, choose rivers from its drainage, and
**carve** them in. Our linear pipeline is deliberate: the plates and mountains David has tuned stay in
charge. But it's why our rivers sit in channels cut into whatever ground is there. Every problem so far
comes from that mismatch: water walls, ledges, gorges, lakes off the real ground. In their method a
river can't stand above its banks, because the banks are made from the river.

| | Génevaux 2013 | Incus Artus |
| --- | --- | --- |
| Order | Rivers → terrain | Terrain → water → rivers → carve |
| Network grows | From mouths, a node at a time, by a grammar | From mouths, choosing branches of the drainage tree |
| Downhill guarantee | Each node higher than its parent | Fill-and-spill drainage; water levels only fall |
| Horton-Strahler | Drives growth (priority and junction types) | Read afterwards, for rank |
| Branching control | Pc, Ps, Pa probabilities; ζ | Branching slider; springs' spacing and height |
| Springs | Wherever growth ends | Placed first: high, wet, spaced |
| Flow | Upstream area: 0.42 A^0.69 | Rain, snowmelt, slope springs, summed downhill |
| Valleys | Made by the method (river to ridge blend) | Only where the terrain already had them |
| River types | Rosgen classes, by slope and coast | Ranks by width; meanders by flatness only |
| Cross-section | Profile function per type, layered | A box: vertical cut to a floor; untouched beside |
| Bank blending | Replace operator with a soft weight | None (the shore problem) |
| Junction angles | By flow ratio | Whatever the drainage gives, corner cut |
| Deltas, oxbows | Not possible (a stated limitation) | 3-channel deltas; no oxbows yet |
| Lakes | Not covered | Fill and spill, carved and filled |
| Representation | A continuous function, any resolution | A recipe evaluated per block, plus river lines |

**Worth borrowing** (my ranking, for Counsel and David to weigh):

1. **River profiles and a soft blend into the land**, which is the shore and bank answer.
   - Each rank (later each type) gets a cross-section: channel, bank, then a blend back to the ground
     over some distance.
   - Inside the blend, the ground is pulled toward the profile instead of left as it was.
   - It fits our pipeline: it's still the Interpreter carving, and Climate never reads it.
   - It also makes a modest **valley floor** around bigger rivers, the part of their "valleys made by
     rivers" we can have without reversing the order.
2. **Rosgen-like types by slope and setting**, driving the profile, the meanders and the bed material:
   - steep: narrow, entrenched, stepped;
   - flat: meandering, wide floodplain, low banks and point bars;
   - near the coast: braided or deltas.

   We already have the slope (flatness) and the coast. The bed material ties into mineralogy (bedrock,
   cobble, gravel, sand, silt) and David's decided "shores by distance from water".
3. **Junction angles by flow ratio:** a creek joins a great river nearly at right angles, and two equal
   rivers meet at a narrow angle. That's a change to the curves near junctions.
4. **A slope limit (their Lipschitz condition) for banks:** the profile never steeper than some slope,
   except where we want a cut bank.
5. **Their selection window ζ** is an interesting extra control on how rivers compete for space, but
   our springs-first method already gives David control there.

**Not worth borrowing:** reversing the order (rivers first) would hand the land's shape to the rivers
and undo the plates and mountains work; their flow-from-area law (our rain-driven flow is richer); and
a continuous-function terrain (our per-block recipe already gives any resolution).

## 7. Open questions for David

- **Shores:** a waterline (flush) shore as the default with ledges by context, or ledges as the default
  with flush beaches by context?
- **Banks:** profiles by rank to start, with types (Rosgen-like) later? Or types straight away?
- **The step snapping:** fix it to work relative to the ground. This is a fault, and I'd fix it first
  either way.
- **The overlay's waterfalls** are the network's, not the world's. Should the overlay show the real
  ones?
- **The sand flats just under sea level** (held up by "crags never drown land"): still open, and tied
  to the waterline question.
- **Still to do from the carving plan:** sliders for the bank table, keeping rivers clear of each
  other, and river banks in mineralogy.

### David's answers [2026-10-30]

- **Rivers aren't driven by rain** [DECIDED]. Basing rivers on rain was misguided, to some extent. Rain
  matters mainly for the wetness and dryness of an area, and **rivers are a co-factor of wetness, not on
  the other side of the equation.** "We only want rivers, lakes and ponds to look right. We only want
  the ecosystem to look right. We shouldn't confuse rivers with rain."
  *Counsel:* wetness for ecology becomes rain plus nearness to rivers, lakes and ponds. Rivers are
  placed and sized for the look, by their own controls. A great river through a desert is fine (the
  Nile).
- **Profiles should be much wider, even from springs.** David will draw the profile each rank should
  take, as a cross-section on a grid.
- **Shores: flush by default, ledges by context** [DECIDED]. Banks get **several ways to fade out
  smoothly** (the cross-section blended into the land).
- **River types:** David will write too many rules for types and slopes, then consolidate them.
- **The paper's Voronoi watersheds** (cells filled with a network running from the high side to the
  low): maybe fit their method to our terrain, if needed. *Counsel:* our drainage tree already gives
  catchments; their cells are worth trying if the tree, made from the broad ground, still gives poorly
  spread branches.
- **The step snapping:** fix it first (Builder's offer). *[Built 2026-10-30: steps relative to the
  ground, and stretches end at the level of the water they join.]*

**Later the same day (via Counsel):**

- **Taking rain out of rivers: go ahead, no proposal round.** *[Built 2026-10-30. Each patch of land
  gives 1 unit of water. Snowmelt and slope-foot springs add units of their own, no longer shares of
  the rain. "Water it needs" for springs defaults to 1 (any high land). The flow is keyed on the
  temperature (for snowmelt) and the ground, so the rivers don't wait on or relay with the wind and
  rain. Widths were already by rank, not water.]*
- **Rain, lakes and springs:** David asked whether it's a problem that rain may still affect lakes,
  and lakes can cause springs (a big lake's outlet is a spring). *Counsel:* no. It's still one way
  (rain → lakes → outlets → rivers), so the pipeline stays linear, and drier country having fewer
  lake-fed rivers is plausible. What matters is that a river's size and placement don't come from
  rain amounts. *Builder:* today lakes don't depend on rain at all. Every basin fills to its rim and
  spills, whatever falls on it. Rain would only come into lakes with salt lakes (dry basins standing
  lower), which are still to come.
- **The network:** "not badly spread". From the world map the rivers look decent and very Dwarf
  Fortress-like, but in the world they're "just not right". David thinks fixing the profiles will
  help most.
- **The profiles drawing:** David has started the cross-section drawing (pixels for shore, dirt and
  the rest, and water blocks), labelled but probably not entirely consistent or exhaustive. Read it for
  the general rules (channel width and depth, bank slope, shoulder, fade, cut bank vs point bar), not as
  every case.
- **Sizes: "drawing wins."** The drawing's sizes replace the earlier width table (creek 2–4, stream
  4–8, river 8–32, great river 32–128). The last profile drawn is a river, not a great river: David
  drew up to rivers. Sizes should **grow proportionally, by ratios we set**, each rank from the one
  below, with a reasonable range per rank rather than one number: "in order to achieve a *pleasant*
  webbing of little springs and creeks and rivers that have bridges and so on." The goal to judge by
  is a pleasant, bridgeable web, not exact sizes.
- **Lakes, after the rivers** (David, 2026-10-30, with a screenshot): lakes come out as "perfect
  elevated rectangular swimming pools with a chunk of square perimeter". Their surface is the broad
  ground's fill on 32-block patches, so the water can stand above the real ground, and the ring fill
  makes square walls. To address once the rivers are adjusted. *[Moved ahead of the banks: the
  rivers into a perched lake were held up to its level, standing as walls of water (David's second
  screenshot). Built 2026-10-30: lakes filled on the real ground, section 2's "Lakes".]* David: "there
  should be some lakes and ponds above sea level."
- **White water at falls, seen from above** (David, 2026-10-30, via Counsel): from above, a waterfall
  looks like the river just carries on. Make the water lighter at the drop wherever a fall is seen
  from above: the map overlays and the far terrain. When convenient, after the routing launch.
- **Rivers cut straight through the land** (David, 2026-10-30, with a screenshot): paused before
  tuning. They must follow the world's final, detailed terrain, "curving between hills instead of
  slicing through them", and "roll like a marble between hills, finding natural valleys and deepening
  them, rather than making its own". Banks should slope the land toward the river, at least a width
  each side (a 32-wide river, at least 32 blocks). See "Why the rivers cut through" below.
- **A small stream notched far too deep** (David, 2026-10-30, after the banks, with a screenshot):
  "these streams either need to ride over the terrain like a marble, or cut far deeper in the land, to
  fit in cleanly." The cause: a hill on the real ground across the planned path, with no way round
  inside the 64-block corridor, so the stream kept its level and slotted through, and steep banks
  (his 2.4 up a block across) made it a gash. A preset to try, `rivers-balanced` (his default with:
  corridor 256, step 2, steepest bank 1.0, widens up to 6, at least 4, the creek's fade 2.0). What
  settings can't fix: a stream starting in a hollow still notches its rim (pools behind rises, not
  built). The same day: the mountains' points are hidden at startup (they don't save with presets,
  like the additives' dots), and the greyed "River banks" placeholder is gone now that banks exist
  (rivers in mineralogy, gravel, sand and mud by the water, is still to come).

## 8. Proposal: river profiles (Builder, for David's approval)

**APPROVED by David, 2026-10-30** ("that sounds good to me, go ahead"), with his answers to the four
questions:
1. Keep the default ratios, and try them in the preview first. If great rivers (59–150) feel too big,
   the fallback is a last width ratio of 2 (59–118).
2. Growing within a band replaces "keeps full size for". If David misses that control, a "how fast it
   grows" slider can come back.
3. The fades by context, as proposed.
4. Step by step, with a launch for David after each: sizes and shape, then lopsided bends and ledges,
   then the fades.

*[Step 1, sizes and shape, built 2026-10-30: the ratios, bands, growth and "size varies"; a shape per
rank and "shape varies"; the preview in the Carving section. Rivers saved before it read back as
boxes.]*

*[Then, ahead of steps 2 and 3, the routing (section 9) and lakes on the real ground (section 2),
both David's call. Step 3, the banks' fades, built 2026-10-30 (section 3, "On a river's bank"). It
includes the outside's "ledge then slope" by bend sharpness, so what's left of step 2 is the
lopsided bed: the deep part moving toward the outside, a steep outer edge, and point bars.]*

From David's drawing, [`rivers-profiles-2026-10-30.png`](rivers-profiles-2026-10-30.png) (one pixel a
block; tan ground, blue water). Top to bottom: 9 springs, 7 creeks, 4 streams, 4 rivers.

**Where it lives** (the linear pipeline): the sizes and shapes are worked out in Climate's step 7 (the
curves), as numbers on each river point. The World Interpreter carves them, together with the bank's
fade into the land. Nothing feeds back: Climate still reads the uncarved ground.

### What the drawing says (measured from the pixels)

| Group | Widths | Deepest | Banks | Shape |
| --- | --- | --- | --- | --- |
| Springs (9) | 1–3 | 1–2 | 5 flush both sides; 3 a 1-block ledge on one side; 1 a ledge on both | a slot, or a little V |
| Creeks (7) | 5–9 | 2–3 | 4 flush; 3 a 1-block ledge on one side | a V (1-2-3-2-1); or deepest right against the ledge, stepping up a block at a time to the other side; or wide and shallow (1-1-1-1-2-1-1-1-1) |
| Streams (4) | 11–19 | 4–5 | all flush | steep sides (2–3 deep right at the edge), a broad flat floor; 2 of the 4 deeper toward one side |
| Rivers (4) | 29–51 | 4–7 | 3 flush; the widest a 1-block ledge on one side | a bowl: shallow shelves at the edges (1 deep, about 4 wide), steps 2–7 wide, a deepest part 3–21 wide; the widest has its deep part against the ledge (a third of the way across) and long shallow steps to the other side |

The rules I read from it:
1. **Flush is the norm**, about two thirds of the profiles. **A ledge is always 1 block**, almost
   always on one side only.
2. **Where there's a ledge, the deep water is against it**, and the other side shallows in steps: a
   cut bank and a point bar, as at the outside and inside of a bend.
3. **The bed steps up a block at a time,** and the steps get wider as the river gets bigger: about 1
   block in creeks, 2–4 in streams, 4–7 in rivers.
4. **The edge depth varies by size:** creeks start 1 deep at the edge, streams 2–3 (steep sides),
   rivers 1–2 with a shelf. So the shape isn't just a scaled box or a scaled V.
5. **Proportions:** each rank is about 2½ to 3 times as wide as the one below, but less than twice as
   deep. Bigger rivers are relatively shallower and wider.

### 1. Sizes: a ratio per step up

- **A spring's size** (default 2 wide, 1½ deep: a 1 or 2 block slot).
- **For each step up a rank, a width ratio and a depth ratio** (sliders). Defaults from the drawing:

  | Step up | Width ratio | Depth ratio | Gives |
  | --- | --- | --- | --- |
  | spring → creek | 3 | 2 | creek about 6 wide, 3 deep |
  | creek → stream | 2.5 | 1.5 | stream about 15 wide, 4–5 deep |
  | stream → river | 2.5 | 1.25 | river about 37 wide, 5–6 deep |
  | river → great river | 2.5 | 1.25 | great river about 94 wide, 7 deep |

  Width ratios slide from 1.5 to 5, depth ratios from 1 to 3.
- **A range per rank, not one number:** each rank's band runs halfway (by ratio) to the ranks either
  side, so the bands meet with no jumps. A river **grows within its rank's band** as it goes down: it's
  smallest where it becomes that rank, and nearly the next rank's size where it joins a bigger one.
  Creeks grow from a spring's size at their springs. The bands at the defaults:

  | Rank | Width | Deepest |
  | --- | --- | --- |
  | creek (from its spring) | 2 → 9½ | 1–4 |
  | stream | 9½ → 24 | 4–5 |
  | river | 24 → 59 | 5–6 |
  | great river | 59 → 150 | 6–8 |

  Those match the drawing: creeks 5–9, streams 11–19, rivers 29–51.
- **"Varies":** a slider for how much a river's size wanders along it (slowly, so it doesn't pulse).
- **This replaces** the old width table, the √(2 × width) depth with its depth slider, and "keeps full
  size for": the growth within each band already starts small at the spring.

### 2. The shape across: four numbers per rank

Each rank gets its own **shape**, with defaults read from the drawing. Great rivers start as copies of
rivers.
- **Edge depth:** how deep the water is right at the bank, as a share of the deepest. Creek 0.3 (a V),
  stream 0.5 (steep sides), river 0.25 (a shelf).
- **Floor:** the share of the width at the deepest. Creek 0.2, stream 0.5, river 0.3.
- **Bowl:** how the bed goes from the edge to the floor. Below 1, it drops quickly then flattens
  (streams); 1 is even steps (creeks); above 1, it stays shallow then drops (rivers' shelves). Creek 1,
  stream 0.6, river 1.6.
- **Shape varies:** how much these three wander along the river. The drawing shows as much variety
  within a rank as between ranks.

Each column's depth is that curve, rounded to whole blocks, so the bed steps a block at a time by
itself, and the steps widen with the river.

### 3. Lopsided at bends: cut bank and point bar

The curve already knows the outside of every bend. Today it only stores which way, so it would also
store **how sharp**.
- **The deep part moves toward the outside,** further the sharper the bend: up to "against the bank"
  for creeks, a third of the way across for rivers (as drawn). Default: up to 0.6 of the half width.
- **The outside's edge depth rises to the full depth** (a steep cut), and **the inside gets long,
  shallow steps** (the point bar).
- **On straight runs it stays even,** with only the "shape varies" wander.

### 4. Banks: flush by default, ledges by context

- **Flush:** the ground beside the water is pulled to the water's level, with the block's top level
  with the water. Where the ground beside is *lower* than the water, it's filled up to that level (a
  natural levee), so water never stands against air.
- **A 1-block ledge on the outside of a bend** sharper than a setting ("ledge from bend"). That makes
  ledges about a third of the time on winding rivers, as in the drawing. The inside stays flush.
- **Later, by rock:** ledges in hard rock, once mineralogy exists.

### 5. The fade into the land: several ways, for you to pick

Beyond the bank, the ground blends from the shore's height back to its own over a **fade distance**.
Default: one river width each side, at least 4 blocks; a slider per rank.
- **Smooth:** an S-curve, gentle at the water and at the top. The default.
- **Slope:** a straight ramp.
- **Terraces:** flat steps of N blocks (old floodplains: walkable shelves along big rivers).
- **Ledge then slope:** the 1-block ledge, then smooth.

**By context:** the inside of bends gets the long smooth fade (a beach), the outside gets "ledge then
slope" with a shorter fade, straight runs get smooth. Each is a dropdown with a slider, so you can
change which goes where.

**Steep land behind:** if the ground beside rises so fast that the fade would be steeper than a
setting ("steepest bank", default 1 block up per block across), the fade widens up to a limit. Past
that, the rest stays a cliff. That's the "ledge by context: steep land behind the shore", and it keeps
mountain gorges.

### What you'd see in the panel

- **The Carving section:** the ratio table, a band readout, the four shape numbers per rank, the bend
  sliders, and the fade dropdowns with their sliders.
- **A live preview,** drawn like your picture (tan and blue pixels), with one profile per rank,
  straight and at a sharp bend. You can put it beside your drawing and tune until they match.
- **The cross-section** (Shift + drag) shows the real carved profile and fade.

### Cost and risk

- Each block column looks further for its river: half the width plus the fade, instead of half the
  width. The lookup grid gets more entries, and the carving work grows with the wider rivers. I expect
  it to stay small next to the terrain itself, and I'll measure it with the soak test.
- **The far terrain must use the same fade,** or near and far won't agree at the banks. It goes in
  the same function (`ground_and_water`), so it can't drift.
- **Waterfalls and rapids** keep working: the profile hangs from the water level at each point.
- **Deltas** use the river's profile, scaled to each channel.
- **Not in this step:** lake shores and the sea's shore. Flush sea shores need the "shore" answer at
  Z 127 (section 4). That's a separate step once rivers look right.

### Questions for David

1. **The size defaults:** the ratios above (3, 2.5, 2.5, 2.5 wide; 2, 1.5, 1.25, 1.25 deep). Does a
   great river of 59–150 wide sound right, or should the last step be smaller?
2. **Growing within a band** as it goes downstream, replacing "keeps full size for": agreed?
3. **The fade's default by context** (smooth inside, ledge then slope outside, smooth when straight):
   agreed, or one way everywhere to start?
4. **Build order:** sizes and shape first, then lopsided bends and ledges, then the fades, with a
   launch for you after each. Or all at once?

## 9. Why the rivers cut through (Builder, 2026-10-30, from David's screenshot)

David's cross-section across a meandering river showed vertical slots 15 to 25 blocks deep through
ground at Z 140–157, with the water near Z 128. From the code, four causes, one underneath them all:

1. **The path is chosen on ground with no hills in it.** The flow (step 5) reads the *broad* ground:
   the recipe with the crags, summit crags, hills and torn coast left out, one sample per 32-block
   patch. The hills David sees don't exist for it, so it runs straight across them. The dead-straight
   diagonals are the 32-block grid's steps on a smooth slope, which corner cutting can't bend.
2. **The squiggles ignore the real ground too.** Meanders, the wander and the lean move the line
   sideways by formula. The only check, "deepest gorge", also reads the broad ground, so meanders swing
   into hills it can't see. "Flat enough to meander" is judged on the broad ground as well.
3. **The water only falls, from the lowest real ground across its width.** Once the line touches low
   ground, every hill it crosses after that is cut down to that level: the vertical slots.
4. **No banks.** The cut is vertical and the land beside it is left as it was (step 3's fades, not
   built yet).

Lakes share cause 1: their surface is the broad ground's fill on 32-block patches, so they can stand
above the real ground in square patches.

**The way to deal with it.** *[GO AHEAD from David, 2026-10-30: "i think we can get this... the rivers
are doing some things right". Counsel's suggestion, taken: fill and spill, as step 5, run locally.
Built the same day: the route (step 7, "The route" above); the sideways moves kept on the valley
floor; a readout of the deepest cut. Not built yet: pools behind rises, so a rise the corridor can't
get round is still cut through, and the readout shows how deep. The router is meant to be reused for
lakes.]*
- **Keep the network's plan** (which rivers, springs, mouths, ranks): David is happy with the placement.
- **Route each reach on the real, detailed ground** (the Interpreter's ground without rivers), every
  few blocks, inside a corridor round its planned path, from its pinned start to its pinned end. Take
  the way that stays lowest: the lowest gap between hills first, then the shortest. That's the marble:
  round the hills, through the lowest gaps, along the valley floors.
- **Meanders only where the real valley floor is flat and wide enough**, measured on the real ground.
  Elsewhere the route's own shape is the river's.
- **The water sits on the routed valley floor.** Where even the lowest gap is higher than the water
  coming in, it fills behind it as a pond up to the gap, as real water does, or cuts a short, limited
  notch through.
- **Then the banks** (step 3), at least a width each side, deepen the valley it found instead of
  digging a trench.
- **Then lakes,** with the same idea: filled on the real ground round their patch.

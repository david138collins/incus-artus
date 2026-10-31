# Rivers, from beginning to end

How the rivers work as built (through 926e66b, 2026-10-30), written for David and Counsel; with a
review of the shores and banks (every shore a block high), and of Génevaux et al. 2013, "Terrain
Generation Using Procedural Models Based on Hydrology", and how we differ. The plans and decisions
behind each part are in `worldgen.md` ("Breaking the water loop", "The river rework", "Carving").

**Contents:** 1. The pipeline at a glance · 2. Each step in detail · 3. In the world · 4. Why every shore
is a block high (and everything else that sets a shore's height) · 5. The paper · 6. How we differ, and
what's worth borrowing · 7. Open questions for David · 8. River profiles · 9. Why the rivers cut
through · 10. Water standing in the air, sandbanks in the sea, a slab in a lake.

---

## 1. The pipeline at a glance

Rivers live in **Climate** (the World Compiler's stage between the Additives and the World Interpreter)
and are carved by the **Interpreter**. The pipeline is linear: nothing later feeds back into anything
earlier, so the rivers are made *from* the ground but never change the ground Climate reads.

```text
the recipe (the Interpreter's ground)
  │  read twice, without the rivers: the broad ground (no crags, summit crags, hills, torn coast)
  │
  ▼  RIVERS FIRST: they read the ground alone (2026-10-31); after them, on the climate's grid,
  │  temperature ─► wind ─► rain ─► mineralogy, none of which the rivers read
  step 5  THE FLOW       basins fill and spill; every 32-block patch drains somewhere; water adds up
  │                      downhill (a unit a patch; snowmelt and slope-foot springs retired; not rain);
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

*Retired 2026-10-31 (David: "cut them"): **snowmelt** and **slope-foot springs**. They only added to
each spring's own water, which only decided whether a spot could hold a spring ("water it needs" 1 lets
any land) and how far a lone stream ran, and lone streams already reach other water. Sizes come from
joins, not water. Their code stays in `rivers::water`; Climate passes them as 0, and the panel no longer
shows them. "Drawn from" is labelled as the picture's only. "A lake from" stays.*

- **The ground it reads** is the broad ground on the 32-block grid: the recipe's big shapes (curves,
  ridgelines, rise, peaks), without the fine layers, because crag noise makes thousands of false pits.
- **Fill and spill (priority flood).** Start from the sea and the map's edge; always take the lowest
  ground not yet reached next; whatever it reaches drains into it. A dip with no way out is only reached
  over its rim, so it fills to the rim's lowest point and spills there. Result: every patch has one
  patch it drains into (8 neighbours, diagonals included), always downhill or across a lake, always
  ending at the sea. Ties on flat ground are broken by a fixed nudge from the seed, so paths wander
  instead of running in grid lines.
- **Each patch's water:** 1 unit for a patch of land, plus **snowmelt** on high ground (above Z 300:
  +1 unit), plus **springs where a slope eases off** (the steepness coming down to it minus the
  steepness going on); both retired, passed as 0. Water adds up downstream. *[2026-10-30, David:
  rivers aren't driven by rain. It was each patch's rain, with snowmelt and springs as shares of it.]*
- **Rivers first in Climate** (2026-10-31, David: "they don't mean anything to rivers, and rivers CAN
  mean something to them"): the flow reads the ground alone, and is made before the temperature, wind
  and rain, so changing those never remakes the rivers. *[It was keyed on the temperature too, for
  snowmelt, which now goes by height alone (it was also colder than −0.2). With snowmelt retired, David's
  15:20 preset gives the same rivers to the byte.]* Nothing after them reads them yet; the climate may
  later (the lake effect, milder air by big water).
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
- **Sizes by area** ("sizes by area", **off** by default; 2026-10-31, stage 5 of `docs/water-rework.md`).
  It's an experiment beside the ranks' sizes above. David: "make sure it can be disable with a
  checkbox". Unticked, everything is as before.
  - **Size is water across:** a river's size is its blocks of water across, about its width × its
    depth × how full its bowl is, with every column at least a block (`world::water_across`,
    `fullness`).
  - **Gathered on the way down** (`water_by_area`, upstream first):
    - a spring starts with "a spring's water" (1);
    - where waters join, they keep a share of both ("where waters join, they keep", 0.9);
    - running adds a block of water for every "grow a block of water every" blocks (2,000).
    - A **lake passes on** the water of the rivers into it to the river from its outlet, as a junction
      would. Without that, an outlet would start again as a spring.
  - **Laid out by the slope** (`size_by_area`): the broad ground's fall over up to 4 patches down.
    - Where it falls "flat ground falls at most" (0.2 blocks in 100) or less, the water is at its
      broadest: "on flat ground, blocks wide for each block deep" (16).
    - Where it falls "steep ground falls at least" (5 in 100) or more, it's at its deepest: "on steep
      ground" (0.5, twice as deep as wide; David's "2 deep spring with steep falls and drops").
    - Between the two, it's mixed evenly by ratio, and so is the shape across, from "by area, on flat
      ground" to "on steep ground".
    - It's never under a block wide or deep. A single column holds all its water, and a block-deep
      river is as wide as its water.
  - **Banks** fade over "by area, all water fades over (widths)" (1), plus "…at least".
  - **Names are labels only** (David: "the names dont matter"): under 3 blocks of water a spring, 3 to
    16 a stream, more a river (`world::STREAM_FROM`, `RIVER_FROM`). They act as ranks 1 to 3 for corner
    cutting (one more pass each) and deltas (rivers).
  - **Greyed out when ticked:** the ranks' sizes, shapes and fades, and the network's "stream orders
    per rank".
  - **The readout** (ticked or not): how much of the rivers' length has under 3 blocks of water across,
    3 to 16, and more; the widest, the deepest, and the most water across.
  - **"Sizes in the close-up"** colours every river's water by its water across: yellow at 1, green at
    4, sky blue at 16, violet at 64, and magenta at 256 or more.
  - **The preview** draws 1, 4, 16, 64 and 256 blocks of water on flat ground, halfway and on steep.
  - **The network map's hover** says a river is sized by area, instead of naming its rank's sizes.
- **Reaches:** each river is cut into stretches from a spring or junction down to the next junction, a
  lake, or the sea. A stretch's ends are **pinned**, so rivers always meet.
- **The route** (`route`, 2026-10-30, section 9): the network's path is a plan on the broad ground,
  with no hills in it. Each reach finds its way on the **real ground** (the recipe without rivers)
  through a corridor of 4-block cells (`grain`) within 64 blocks (`corridor`) of that plan, by fill and
  spill from its end: the lowest cell not yet reached is always taken next (on level ground, the one
  nearer the end), so a dip fills to its lowest gap and spills. The route is the way the start's water
  then runs: round the hills, through the lowest gaps, along the valley floors. Its ends are the
  plan's, pinned. A very long reach uses a coarser grain, to keep its grid under 4 million cells. The
  panel shows how long it took, and the deepest cut below the banks (with how many points cut deeper
  than 6 blocks).
- **Where the way is found** (2026-10-31, stage 4 of `docs/water-rework.md`): "found:" **everywhere**,
  **only where needed**, or **off**. Everywhere is the corridor above, the way before.
  - **Off:** each river follows its plan, smoothed.
  - **Only where needed:**
    - Each river follows its plan, read every 8 blocks for the cut its water would need: the ground
      against the water only falling, its level the lowest ground so far.
    - Where the cut is deeper than "a way round where the cut is deeper than", the river looks for a way
      round. That depth runs from "near the sea" at the sea's level to "high up" at the highest river's,
      blended by the water's height; both start at 8.
    - The search is a fill and spill, as the corridor's, in a box round the stretch, out to where the
      cut is small again. The box reaches half the stretch's length, or 4 times its cut if more, 32 to
      256 blocks, at an 8-block grain. The way found is kept if it cuts less.
    - The corridor and grain sliders are greyed out.
  - **The readout:** how many stretches cut too deep, how many found a better way round, the deepest
    cut on the plan and on the way found (read the same way), and how long it took.
- **Sliders that can't act are greyed out** (2026-10-31, with stage 4; David's "grey out, don't hide"):
  - "steepest bank" when "widens up to" is 1 (it never widens);
  - each rank's sizes, shape and fade where no river reaches the ranks that use them. A creek's band
    reaches toward the stream's size and shape, so creeks use the first two steps up and the first two
    shapes.
- **The shores readout and overlay** (2026-10-31, stage 0 of `docs/water-rework.md`).
  - **Shores:** at every 8th point of every river above the sea, both sides, the first dry block beside
    the water is counted as one of:
    - flush (its top level with the water: David's "you can walk out");
    - a 1-block ledge;
    - a taller bank;
    - below the water (water standing against air).
  - **Land beside rivers:** across those banks, out as far as each fade could widen, how much the carve
    lowered and raised the land (on average and at most), and the steepest slope next to land it
    changed.
  - **The overlay:** "shores in the close-up", under the readout, colours the dry block beside every
    river's and lake's water: green flush, amber a ledge, red a taller bank, magenta below the water.
  - **Cost:** about 20 column readings per sampled point. With it, making the rivers took 5.8 s on 4
    cores (default preset).
- **Cores** (2026-10-31, David: the CPU at 92 to 99 °C while the rivers were made): the making shares
  its work over half the CPU's cores (`cores`, "cores used", under the Carving section's report; 12
  of 24 on David's machine). Fewer runs cooler and takes longer; the rivers come out the same on any
  number (a test checks it).
- **The line:**
  1. **Corner cutting.** The route is taken at points about a river's width apart (at least 2 cells,
     at most 48 blocks), then Chaikin's method slices every corner off: a creek 2 passes, one more per
     rank up (at most 5). So bigger rivers bend wider.
  2. **Resampling,** every 2 to 6 blocks.

  *[Cut 2026-10-31, David: "they only seem to make it worse at any setting besides off": each point
  used to be moved sideways by a wander, a lean to the outside of the bend and a meander, capped by
  "most sideways" and pulled back by "keep within". Every move put the line onto the higher ground
  beside the way found.]*
- **The anti-kitty-cornerer** (2026-10-31, David's name; a switch, on by default; kept with the world,
  an `anti-kitty-corner 0` line in rivers.txt when off). The water's blocks always join by a face,
  never only at a corner:
  - **A narrow river on a diagonal** (under about 1.4 blocks wide) steps in a 1-block zigzag: every
    block its line passes through is water. Wider rivers are unchanged.
  - **A spot in any river's water is water,** though another river's line is nearer. *[Where a narrow
    river joined a wide one, the wide one's water stopped where the narrow one's line was nearer, and
    the gaps joined blocks only at corners: in a sample of David's world (every 8th point), 176 places
    before, 115 after.]* (This holds with the switch off too, since 2026-10-31's ridge fix.)
  - The 115 left were two separate rivers running side by side, their edges touching at corners on a
    diagonal. They now merge (next).
- **Rivers running alongside each other merge** ("rivers alongside each other merge", on by default;
  2026-10-31, David: it "should cause a waterfall to merge").
  - **How:** after the water's laid, each river is walked down. At its first point whose water comes
    within a block of another river's lower water (or as low, the other nearer the sea), it ends
    there: its level falls to the other's, with a curtain if that's 2 or more, and the rest of its
    way alongside is dropped.
  - **Where:** only clear of its own ends, by its width plus the widest river's, since that's where it
    meets its junctions' rivers.
  - **Order:** rivers nearest the sea go first, so none merges into water that's since gone.
  - A river that merged loses its delta.
  - The panel counts how many merged.
- **Joins ease down** ("joins ease over", 3 of the joined river's widths; 0 off; 2026-10-31, David: "a
  linear joint fade similar to how we do the lateral bank fades"). Where a river joins another, its
  water over its last stretch is ramped down toward the joined river's, as steeply as rapids go: a
  block of drop for every "rapids" blocks of river. What that can't take is a fall at the shore. It
  only ever lowers, cutting its bed a little deeper there.
- **Where rivers' waters overlap, the lowest water and then the deepest floor win** (2026-10-31, David's
  screenshot). *[A tributary's last stretch ran on to the junction in the middle of the river it
  joined, and inside that river's water it kept its own shallower floor and higher water: a stone ridge
  running out into the river with a ribbon of water on top.]* Now the tributary ends at the river's
  shore, with its drop there. Within one stretch, the nearest piece still decides: the pieces either
  side of a step or a fall overlap where they meet, and the lower water winning there would move each
  fall half a width upstream and notch its cliff.
- **The water level** (`lay_water`, the 926e66b fix):
  1. **From the real ground.** At each point, sample the real ground (the recipe with hills and crags,
     without rivers) at three spots: the middle of the line and both edges of the river. The level
     starts as the lowest of the three, rounded down.
  2. **Only falling downstream.** Stretches are done from upstream down; a stretch starts no higher than
     what flows into it.

     **After a hollow** ("the water's level", 2026-10-31, stage 3 of `docs/water-rework.md`; David:
     "filling and cutting should be on a threshold... least change is kind of how i try to get
     everything"):
     - **Cut through** (the default, the way before): the level is the lowest ground so far, so after
       a hollow the water stays at the hollow's level and cuts through every rise after it.
     - **Least change:** each stretch's levels, only falling, are those nearest the lowest ground
       across each point in all. That's least squares, by "pool adjacent violators" in one pass: rises
       cut and hollows filled by turns.
       - **Fill hollows up to** (3): the water stands at most this far above the ground at any point.
         A deeper hollow is filled this much, and the rise after it cut for the rest.
       - **Cut rises up to** (40): before cutting a rise deeper than this, the hollow before it is
         filled more, but never past the fill's limit, which wins.
       - **A filled hollow:** the water is deeper there (the bed isn't raised). The banks are raised to
         the shore at the water and ease back down to the land over their fade. With fills of a few
         blocks it reads as a low bank along the river, not a causeway.
       - **The readout** counts the water above its ground: at how many points, and the most.
       - **Only between banks** (2026-10-31, David's screenshots: a causeway over a valley at fill 20,
         an embankment on a hillside's low side at fill 10): the water also rises no higher than the
         lower bank, sampled 3 blocks out from the water either side, plus "…and above the lower
         bank up to" (3 to start, David: "yes"). At 0 a hollow fills only where the land rises on both sides across
         the river. That removes the embankments, but rivers along a slope can't fill, so more is cut.
         On David's 13:22 preset with no steps: 552 points filled and 15,961 cut more than 6 at 0;
         6,792 filled and 12,477 cut at 3.
       - **Steps take it too:** with least change, each flat run of the steps takes the level that
         changes its land least. That's its points' mean, rounded down, never above any of their
         ceilings, never below its lowest point. *[Each run was set at its lowest point, cutting its
         others down to it: with steps of 18, up to 17 blocks, two thirds of all points on David's
         world cut more than 6. Now 45,690 of 80,457 at 0 above the bank, and 36,646 at 3.]*
       - **The step and the fall together give a mix.** Drops of "a waterfall from" or more are falls.
         Smaller ones are spread as rapids where their run has room, and are small curtained falls
         where it hasn't. So a step below the fall height gives rapids and little falls on gentle
         ground and big falls where the ground drops further than a fall in one step.
  3. **Never below the sea**, and a stretch into the sea or a lake ends at that water's level. Any
     stretch **ends where it first meets the sea** (2026-10-31): at its first point with the sea
     under any of its three samples, or with its water at the sea's level on ground more than a step
     above it (a cliff top with the sea or a beach at its foot), the rest dropped. Across a low plain
     at the sea's level it carries on. Cut short, it no longer joins the water at its planned end. *[Where the broad ground had land but the real
     coast came first, a river ran on along a cliff top, dragged down to the sea's level by its bank
     over the edge: a slot up to 57 blocks deep, 748 points along 22 stretches on David's world.]*
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
     only come from smaller drops, such as a creek's last bit down to its river, or every step of 2
     with a step size of 2 (`rivers-balanced`).

     **Rapids stay within their own run** (2026-10-31, stage 0 of `docs/water-rework.md`). They spread
     back only to the foot of the drop above, squeezed to fit a short run, and each waterfall's
     curtain is set after all the rapids. *[They used to spread their full length upstream. On steep
     ground they reached past the waterfall above and pulled its water down after its curtain had
     been set. The curtain and its cliff stood up out of the water, and the real drop moved upstream
     with no curtain: walls of water. David found that "rapids at 1" got rid of them: its spread never
     reached the next point (2 or more blocks away), so it only hid the bug.]*

     **Every drop of 2 or more left sheer gets a curtain,** not only those as tall as "a waterfall
     from". That's where rapids had no run to spread over, and it's dropped at its lip like any
     waterfall (`falls_at_lips` now goes by the curtain). *[2026-10-31, David's screenshot: with no
     steps and waterfalls only from 25 blocks, the water changes level at nearly every point. Rapids
     there had no room, and stayed sheer with no curtain: 4,895 drops on David's world, up to 24
     blocks tall, walls of water. Now none.]*

     **Each waterfall drops at its lip** (`falls_at_lips`, 2026-10-31, David's screenshot: pillars of
     water standing in the air). The water above a fall held its level on to the next point, 2 to 6
     blocks on, out past the cliff's edge where the ground had already fallen away, and its curtain
     hung over half the piece after that, a slab out in front of the face. Now, along each fall's
     piece, a block at a time, where the lowest ground across the river first falls below the water
     above the fall, a point is put in at the water below (halfway between that spot and the last one
     whose ground still held the water). So the water above ends at the edge and the curtain hangs
     down the face. This is done last, after everything worked out point by point. The water above
     a fall is kept on its lip point whether the curtains are on or not, for its banks (section 3);
     the switch is kept with the world.
  7. **Ponds behind rises** (2026-10-30, David: "these streams either need to ride over the terrain
     like a marble, or cut far deeper in the land"; Counsel's option B, breach or fill). Laid as above,
     a rise across a river's way that the route couldn't get round is notched to the water's level
     upstream, however deep. Now, from each stretch's end upward, the water has to be no lower than
     each rise less 4 blocks (`cut_up_to`, "cut through rises up to"): a rise is cut through at most
     that deep, and above a higher one the water is held back, level, up to the rise's height less
     the cut. Where that leaves it standing above the ground, it's a **pond**: the real ground below
     that height, reached from under the river on the route's 4-block cells (the river's own cell
     counted wet), kept as a lake with the others (saved with the world the same way). A pond that
     would spread more than 512 blocks from its river (`pool_reach`, "a pond spreads at most"), or
     off the map, can't be held: usually its water would spill into the next valley, the way the
     river ought to have gone. Nor can one that **would drain into another river** (2026-10-31):
     where its water reaches another river's channel whose water is lower, unless that river flows
     into this one (a creek's pond stood 31 blocks above the river beside it); its own river's line
     below the rise; or where a river's channel is cut through its rim more than a block and a half
     below its water, one flowing in too (it would drain back up that notch: a pond had backed up a
     tributary beyond its own water, in walls 37 blocks tall). The other rivers' water is as laid
     before any pond, the lowest it can be; their points are looked up by 32-block cell, each in the
     cells its channel reaches. **A pond that reaches a lake below it takes the lake in**, at one
     level: where lakes overlap, the highest water shows (section 3). *[Built first that morning as
     "cut through instead" (David's go-ahead, for the raised slab of a pond over a lake); once the
     overlap showed the highest water, that only notched more hills, so changed. Builder, for David
     to confirm.]* That rise is cut through instead, with the points beside it up and down the
     rise whose ponds would be much the same (so the rise isn't tried a point at a time, which took
     9.6 s), and the stretch is looked at again. Where a stretch joins water held in a pond, its own
     end backs up to the pond's level (only there: elsewhere a stretch still ends no higher than its
     own ground). Rises are worked out for each stretch on the cores it may use. The panel (the Lakes
     heading): "ponds behind rises" (on/off; off, every rise is cut through as before), "cut through
     rises up to", "a pond spreads at most"; the report adds how many ponds, and how many rises were
     cut through instead (and how many of those ran into water below). In the world, a river's raised
     bank inside a pond's water is left out (it would stand up out of the pond).

     On David's \_ez-bake world (an offline check that rebuilds its rivers from a preset), the
     points cut more than 6 blocks below their banks: David's default 26,538 → 14,113 (207 ponds, 478
     rises cut through instead; made in 2.3 s, was 1.1 s); `rivers-balanced` 24,643 → 7,369 (221
     ponds, 293 cut through instead; the deepest cut 89 → 51 blocks; 3.5 s). Of the rises still cut
     through (at a 512-block reach), about 40% would have needed ponds over 40 blocks deep: whole
     hills across the planned way, which only routing on the real ground at the network's own step
     (option A) can fix.
- **Deltas:** where a river or great river reaches the sea and its last 64 or 128 blocks fall by less
  than 1%, the last stretch fans into three channels, each a third as wide, all at sea level. Not
  where the river met the real sea before its planned mouth, nor where the real ground at the
  delta's head is a step (4 blocks) or more above the sea (2026-10-31: they'd carve slots at the
  sea's level into high ground).
- **Lakes, filled on the real ground** (`fill_lakes`, 2026-10-30; they were step 5's surface on
  each 32-block patch, raised square pools with square walls round them). Step 5's lake patches,
  grown by 64 blocks (`lake_reach`, "look round each lake"), are joined into regions where they touch.
  Each region's real ground is read on 4-block cells (the route's grain) and filled and spilled from
  its edge, as the routes are: each dip fills to its lowest gap, whatever its shape. A pool at least
  1 block deep at its deepest (`pond_depth`, "a pond from") is kept. So a planned lake can come out
  smaller, larger, or as several ponds, and dips near it can become ponds. The panel counts the lakes
  and ponds. A river into a lake ends at that lake's real water nearest the planned lake (routed
  there like any reach), at its level. *[Fixed 2026-10-30: it ended where the lake was planned, so
  where the real water lay elsewhere the river stopped dead in a field (David's screenshot); 20 of
  207 reaches on David's \_ez-bake world, now none.]* If the lake has no real water at all, the river
  carries on through it the way step 5 drained it: to the river leaving it (joining it there), the
  sea, or the bottom of its basin. The network still plans with step 5's lakes (springs at big lakes'
  outlets).

## 3. In the world (`src/world/carving.rs`, `interpreter.rs`, `terrain.rs`)

- **Saved with the world** as `rivers.txt` (every point of every stretch, the lake surfaces, the
  banks' settings, and a `curtains 0` line if the curtains are off).
- **Each column** (`Carving::here`) finds the nearest piece of river line through a lookup grid.
  - **Inside the river** means within half its width of the line. For an even width, the odd block goes
    to the outside of the bend. There the ground is cut down to the **channel floor**, the water level
    minus 1 minus the depth, and water fills from the floor up to the level. In the block below a
    waterfall's lip, the curtain raises the water to the level above the fall. *[2026-10-31: it was
    the first half of the next piece, up to 3 blocks, a slab out in front of the cliff.]*
  - **The sea is left alone** (2026-10-31): where the ground is under the sea (`under_sea`), a river
    cuts no channel, cuts or raises no bank, and hangs no curtain. *[Where the broad ground had land
    but the real ground is sea, rivers ran on into the sea; their trenches showed through shallow
    water, and their banks stood up out of it as long straight sandbanks (David's screenshots).]*
  - **In a lake's water cell**, water stands to the surface wherever the ground is more than half a
    block below it. Each lake is a fine grid of its own, found through a lookup grid. Where lakes
    overlap (a pond backed up over a lower one), the highest water shows, and beside them the highest
    water's edge (2026-10-31: it was the first in the list, so a lower pond showed through a higher
    one, in walls up to 37 blocks tall).
  - **In a dry cell next to the water**, a dip below the surface is filled up to one block under it,
    so water never stands against air. (Worlds saved before 2026-10-30 read their lakes back as one
    grid of 32-block cells.)
  - **Both are judged on the ground as a river's bank has carved it** (2026-10-31, David's
    screenshot at "a lake from" 40). A river's bank cut in a lake's water fills with the lake. One
    cut beside it is held up to one block under the surface. A river's own channel beside a lake is
    left as it was, since its water joins the lake's. *[They were judged on the uncarved ground.
    Where a river ran past a pool, its bank cut the pool's rim below the pool's water, and the pool
    stood as a slab of water with a sheer side.]*
  - **On a river's bank** (`Banks::ground`, profiles step 3, built 2026-10-30): beyond the water,
    within the point's fade, the ground is pulled to the **shore** and faded back to its own height.
    The shore is flush (the ground block at the level − 1, its top at the water), or a block higher
    for a ledge. Where the land is higher, it's cut. Where it's lower, it's raised (a levee), so the
    water never stands against air. Beside the first two blocks past a waterfall's lip, the shore is
    the water above the fall's: the cliff the fall comes down (2026-10-31: cut down to the water
    below, the fall's sides stood in the air).
    - **Fade width:** one width each side for every rank (`fade_widths`), at least 2 blocks
      (`fade_least`). Where the fade would be steeper than 1 block up per block across (`steepest`),
      it widens, up to 3 times (`widest`). Past that it stays steep, a gorge.
    - **One way for every bank** ("banks fade:"): **smooth** (an S-curve, the default), **slope** (a
      straight ramp), **terraces** (flat steps 4 blocks high) or **ledge then slope**. *[Cut
      2026-10-31, David: bends "don't really seem to have a logical way to be used cleanly". A point
      used to count as a bend from a width-over-radius of 0.15 (`bend_from`), with a way for the
      inside, the outside and a straight run, a shorter fade outside (`outside`) and, with the
      compressor, cut banks on the outside ("cut banks keep"). The switch at "a bend from" left
      straight-edged notches where a river straightened out. Presets saved before load with a smooth
      way.]*
    - The settings are kept with the world (a `bank` line in rivers.txt: the way, steepest, widest,
      terrace step, compressor). Worlds saved with the old `banks` line read back with their straight
      run's way; worlds saved before that have no fade.
    - **Compressor banks** (2026-10-31, stage 1 of `docs/water-rework.md`; a switch, "compressor
      banks", off by default; saved with the world). With it on:
      - **Every stretch of river near a spot has its say.** The land comes down to the lowest of their
        banks, each stretch by its nearest piece, so two rivers' banks meet with no seam. (A reach
        coming back past itself, past a meander's neck, counts as another stretch.) *[Each block used
        to take only its nearest river's bank: where two rivers at different heights were close, the
        land went 178, 218, 178, 218 across, in walls.]*
      - **Another stretch's bank cuts no lower than this one's shore,** coming down from its water no
        steeper than the steepest bank. It only stops cuts; it never raises the land. *[Raising land
        too lifted it by up to 91 blocks beside lower rivers.]*
      - **The block right beside any river's water is at least its shore.** *[Where a river stepped
        down at a bend, the water came from the upper piece and the bank from the lower, cut 2 or 3
        blocks below the water beside it.]*
      - **The first dry block is the shore itself:** flush, or raised to it as a levee.
      - **The fade widens so the S-curve's steepest middle stays within the steepest bank.** That's
        half again as wide as before, gentler, and more land lowered.
      - **Raised banks** (where the land beside the water is lower than it) ease back down to the land
        no steeper than "raised banks no steeper than" (0.5), their fade widening with their height up
        to "raised banks widen up to" (3) times. That's separate from the cuts' "steepest" and
        "widens up to". The lookup reaches the further of the two. Both are kept with the world, on
        the `bank` line. *[2026-10-31, David's screenshots: raised banks dropped back over the fade
        alone, which doesn't widen at "widens up to" 1, as steep embankments.]*
      - A "ledge then slope" way keeps its block above the water. (The cut banks on the outsides of
        bends went with the bends, 2026-10-31.)
    - **Measured on David's world** (`rivers-balanced`, smooth banks all round, 4 cores):

      | | off | on |
      |---|---|---|
      | Flush | 93.6% | 92.6% |
      | Ledge | 1.7% | 0.4% |
      | Bank | 2.7% | 6.6% (mostly the outsides' cut banks) |
      | Below the water | 2.0% | 0.4% |
      | Land lowered on average | 2.1 | 2.6 |
      | Land raised on average | 0.04 | 0.05 |
      | Steepest pull-down | 40 | 38 |
      | Carve, per block near a river | 0.6 µs | 1.0 µs |

      What's left: the steepest is a tall waterfall's own cliff. The shores still below the water
      are beside some waterfalls. A few stretches start well above their ground (a pond's water
      backed up into them), and their levees stand up to 50 blocks tall; that's the water's level,
      not the banks.
    - Each river piece is entered in every 64-block lookup cell its banks reach.
- **The carve lowers the recipe's height itself** (`ground_and_water`), so the far terrain, the light
  map and the World Compiler's close-up all see the rivers. The far terrain also blends a creek's water
  in by how much of a far cell it covers, so creeks don't pop.
- **"rivers and lakes" in the Interpreter's Show switches** (2026-10-31, David via Counsel: to tell the
  sea's water from the water Climate makes): off, the close-up shows the ground as it was before the
  carve, and only the sea's water. A world spawned from it leaves them out too (`Mutes::rivers`; the
  top of the screen says so, and its save remembers it). The rivers aren't made again when it's
  switched: Climate reads the ground without them either way, so the recipe it's handed ignores this
  switch, and the view leaves them out after.
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
  (David's 2.4 up a block across) made it a gash. A preset to try, `rivers-balanced` (David's default with:
  corridor 256, step 2, steepest bank 1.0, widens up to 6, at least 4, the creek's fade 2.0). What
  settings can't fix: a stream starting in a hollow still notches its rim (pools behind rises, not
  built). The same day: the mountains' points are hidden at startup (they don't save with presets,
  like the additives' dots), and the greyed "River banks" placeholder is gone now that banks exist
  (rivers in mineralogy, gravel, sand and mud by the water, is still to come).
- **Islands in a river: keep them** (David, 2026-10-30, with a screenshot of a winding river with
  small dry spots in its water: "must be preserved, as it is great, and duplicated if possible").
  They're accidents today. An offline check of David's \_ez-bake world (David's default preset) found 2,679
  dry columns with water on both sides, from three causes: **43%** where one reach's line loops back
  across itself in tight wiggles, leaving a pocket more than half a width from every piece of it;
  **33%** between two reaches running side by side down the same valley floor (each routed to the
  lowest ground, so a creek can run alongside its river before they meet); **24%** where a creek's
  line runs inside a river's water, so the column counts as the creek's (narrow) bank, raised to a
  ledge (mostly 2-wide creeks in 11-wide streams). So: don't change the wiggles, the routing of
  side-by-side reaches, or "nearest piece wins" in the carve without checking the islands still
  come. [LATER IDEA, David via Counsel, 2026-10-30: "sounds nice but im not sure if its critical right
  now"] Islands on purpose: where a river is wide and gentle, bump some noise up through the bed,
  more the gentler and wider. Nothing to build yet.
- **Counsel's go-ahead on the rivers report** (David, 2026-10-30): the dead-end fix and ponds behind
  rises (option B, breach or fill), then stop for David to look. Not now: A (plan the flow on the real
  ground; Counsel advised starting at 32-block patches, given the rejector's budget of 10 worlds in
  under 15 seconds and A redoing the rivers whenever the crags or hills change), C (cheapest detour)
  and D (an erosion stage). David decides on A after seeing these. Built: see Step 7, "Ponds behind
  rises", and Lakes.

## 8. Proposal: river profiles (Builder, for David's approval)

**APPROVED by David, 2026-10-30** ("that sounds good to me, go ahead"), with David's answers to the four
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

## 10. Water standing in the air, sandbanks in the sea, a slab in a lake (Builder, 2026-10-31)

**What David saw** (screenshots of the evening of 2026-10-30, after the ponds behind rises): long
straight channels and sandbanks out in the sea; pillars and walls of water standing in the air; a
raised rectangle of water inside a lake. The island in the river and the ponds beside a winding river
looked right.

**How it was checked:** an offline test rebuilt the rivers of David's \_ez-bake world from David's default
preset and counted every column side where water stands against air (by whose water it is: a river,
a waterfall's curtain, a pond, a lake), every bank raised out of the sea, and every pond over a lower
lake; then, for the tallest walls, what each one stands against.

**What it found** (not the dead-end fix, as first guessed: that never triggered on David's world):
1. **Sandbanks in the sea:** where the broad ground had land but the real ground is sea, rivers and
   deltas ran on into it, and their banks were raised as levees out of the water (92,591 columns
   along 72 stretches). Their trenches showed through shallow water.
2. **Water in the air at waterfalls:** the water above a fall held its level on to the next point,
   past the cliff's edge; the curtain hung over half the piece after, a slab out in front of the
   face; and the banks just below a fall were cut down to the water below, so the fall's sides
   stood in the air. The ponds made all of it worse: they hold the water up to each rise, so it
   comes down the far side as a staircase of falls.
3. **The slab in a lake, and the tallest walls:** where lakes overlapped, the first in the list
   showed, so a lower pond showed through a higher one, or a lake through a pond over it. And a pond
   could back up a tributary beyond its own water, past the tributary's own notch, or stand over
   another river below it.
4. **Slots along cliff tops by the sea:** a river whose bank looked over a cliff edge down to the sea
   or a beach was dragged down to the sea's level, and ran on along the cliff top in a slot up to 57
   blocks deep (748 points along 22 stretches).

**Fixed** (each in sections 2 and 3 above): the sea left alone; each fall dropped at its lip, its
curtain the block down the face, its banks the cliff beside it; overlapping lakes showing the highest
water; ponds that would drain into another river (in their water or through a notch in their rim)
cut through instead; a lake below a pond taken in; any stretch ending where it first meets the sea;
deltas only on low ground by the sea. And making the rivers now uses half the CPU's cores (David:
92 to 99 °C), a slider.

**Before and after**, the same test and preset (blocks of water standing against air, and the tallest
wall):

| Whose water | Before | After |
|---|---|---|
| Rivers | 95,535 blocks, tallest 55 (3,513 walls 8 or more tall) | 7,845, tallest 26 (70) |
| Ponds | 104,453, tallest 55 | 13,549, tallest 3 |
| Waterfall curtains | 149,320, tallest 51 | 61,796, tallest 23 |
| Lakes | 514 | 649 |
| Banks raised out of the sea | 92,591 columns | 0 |

The curtains left are the waterfalls themselves: a one-block sheet at each lip, its back to the
cliff, facing down over the water below (at a step of 4 blocks, every step is a waterfall). The
rivers' walls left are mostly a few blocks tall, at junctions and where a stream meets a river below
it; the 26-block one is a single place, a creek held above a lake it doesn't reach. Ponds' walls are
a block or three, where a pond's rim is thin beside the river below its rise, or a bank cuts its rim.
Made in 3.5 s on 12 of 24 cores.

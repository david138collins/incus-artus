# Water: the code as it stands (Builder, 2026-10-31)

Written for the regroup (David: "we need to regroup from the start with water"), alongside Counsel's
page `vision/water-from-the-start.md`. For each part of the water code:

- what it does;
- what it reads;
- whether it would survive any redesign (**SOUND**), or exists to cover the gap between the plan and
  the real ground (**PATCH**);
- roughly how big it is, in lines of code without the tests;
- its sliders.

Nothing here is a proposal to build.

**The gap.** Steps 5 and 6 plan the water on the *broad ground*. That is the Interpreter's recipe with
the crags, summit crags, hills and torn coast left out, one sample per 32-block patch. Step 7 then lays
the plan on the *real ground*, which has all of those. Wherever the two disagree, a rule was added to
step 7 or to the carve: a hill the plan never saw, a dip, a coast in a different place. Those rules are
the PATCHes.

## Step 5, the flow (`src/worldgen/rivers.rs`, ~200 lines)

- **Each patch's water** (`water`, ~25): SOUND.
  - **Does:** gives one unit for each patch of land, plus snowmelt (on cold, high ground) and a
    slope-foot spring (where a slope eases off). This only feeds the springs' "water it needs" and how
    far a lone stream runs. Rivers' sizes come from stream order, not from this.
  - **Reads:** the broad ground and the temperature.
  - **Sliders (Rivers):** extra water, colder than and higher than (snowmelt); extra water (slope-foot
    springs).
- **The flow** (`flow`, ~75): SOUND as a method. Which ground it reads is the first question below.
  - **Does:** fill and spill (priority flood) from the sea and the map's edge. Every patch gets the patch
    it drains into, always downhill or across a lake, ending at the sea, and the flow adds up downhill.
  - **Reads:** the broad ground, in 32-block patches.
  - **Sliders:** drawn from (the picture only).
- **Lakes in step 5** (a line in `flow`): SOUND.
  - **Does:** a patch standing under its basin's filled surface by at least the setting is a lake.
  - **Slider:** a lake from (blocks deep).

## Step 6, the network (`src/worldgen/network.rs`, ~400 lines with its settings and picture)

Every part of step 6 works on the 32-block grid and reads only step 5.

- **Springs** (~65): SOUND as a step; the rule itself is a tuning question.
  - **Does:** takes candidates on land that isn't a lake, between two heights (or standing high for
    its surroundings), with enough water of their own. They're spaced apart with some jitter, and big
    lakes' outlets go first. Nothing reads how flat or rough the ground is. That's why springs can't be
    added to the flats without adding them everywhere at that height.
  - **Sliders:** plain or relative; from Z and up to Z; higher than surroundings by; surroundings
    within; water it needs; spaced at least; spacing varies; a lake feeds a river from (patches).
- **Mouths** (~20): SOUND.
  - **Does:** each spring's path ends at a mouth. Where two mouths are too close, the one with fewer
    springs loses them.
  - **Slider:** mouths apart at least.
- **Growing** (~30): SOUND.
  - **Does:** grows each river up from its mouth along its springs' paths, splitting where they part.
  - **Slider:** how readily rivers split.
- **Lone streams** (~35): SOUND.
  - **Does:** springs no river took run downhill until they join a river, reach a lake or the sea, or
    dry away.
  - **Slider:** a lone stream runs.
- **Stream order and rank** (~30): SOUND.
  - **Does:** works out each patch's stream order, which sets its rank: creek, stream, river or great
    river.
  - **Slider:** stream orders per rank.
- **The planned bed** (~30): no longer needed.
  - **Does:** lays flat runs and steps on the broad ground. Step 7 lays every level again from the real
    ground, so this now only feeds the deltas' flatness test and the network picture.
  - **Sliders:** its two sliders, "drop gathered into one step" and "a waterfall from", sit in this
    section but act in step 7.

## Step 7, the carving (`src/worldgen/carving.rs`, ~1,300 lines)

- **Sizes and shape** (~70): SOUND.
  - **Does:** sets width and depth by rank, growing downstream within the rank's band (David's
    profiles drawing). Size and shape wander slowly along a river.
  - **Reads:** the rank, and how far along its rank's run the river is.
  - **Sliders:** a spring's width and depth; times wider and times deeper (four each); size varies;
    the shape across for each rank (water at the bank, flat floor, bowl); shape varies.
- **Reaches** (~70): mostly SOUND, with about 30 lines of PATCH.
  - **Does:** cuts the network into stretches, each from a spring or junction to the next junction, lake
    or the sea. Each stretch gets its planned path, patch to patch, and its ends are pinned so rivers
    always meet.
  - **The PATCH:** a reach into a lake ends at the lake's nearest *real* water. Through a planned lake
    with no real water, the reach carries on along the plan, because the planned lake wasn't where the
    real basin is.
- **Lakes on the real ground** (`fill_lakes`, ~140): a PATCH in where it looks; SOUND in how it fills.
  - **Does:** fills and spills the real ground on a fine grid around each step-5 lake, and `lake_reach`
    blocks round it. Every pool deep enough is kept.
  - **Why it exists:** step 5's lakes were 32-block squares, the "swimming pools".
  - **Side effect:** in rough ground, every crag pit a block deep becomes a pond.
  - **Sliders:** look round each lake; a pond from (blocks deep); grain.
- **The route** (`route`, ~110): a PATCH.
  - **Does:** each reach finds its own way on the real ground, in a corridor round its planned path, by
    fill and spill from its end (the "marble").
  - **Why it exists:** the plan doesn't see the real hills.
  - **Sliders:** corridor; grain; cores.
- **The line** (`curve`, ~120): SOUND, with one PATCH.
  - **Does:** cuts corners (bigger rivers bend wider), adds the wander, leans the river to the outside
    of its bends, and meanders it (fully on flat ground). Its points end up a few blocks apart.
  - **The PATCH (~10 lines):** a sideways move that would climb more than a few blocks above the
    valley floor is pulled back.
  - **Sliders:** corner cutting; wander and its swings; lean; meander every; meander size; full
    meanders below; most sideways; keep within.
- **Laying the water** (`lay_water`, ~185). Its rules, in order:
  1. **Lowest ground across each point** (~35): SOUND if water follows the land; replaced if the land
     makes way. It keeps the water from ever standing above its banks.
  2. **Only falling downstream** (~15): SOUND. Upstream reaches go first, and a reach starts no higher
     than what flows into it.
  3. **Ends at the water it meets** (~15): SOUND. The river ends at a lake's level, the sea's, or a
     junction's.
  4. **Ends where it first meets the sea** (~15): a PATCH. It exists because the broad ground had land
     where the real coast is sea.
  5. **Ponds behind rises** (~35 here, plus `ponds_of`, `pond` and `Water`, ~220): the biggest PATCH.
     - **Does:** a rise the route couldn't get round is cut through up to a few blocks. A deeper rise
       holds the water back in a pond up to its lowest gap, unless the pond would spread too far or
       drain into other water.
     - **The catch:** ponds pile up on rough ground along every river, because one cut depth applies
       everywhere.
     - **Sliders:** ponds behind rises; cut through rises up to; a pond spreads at most.
  6. **Steps** (~12): SOUND. Flat runs, then drops, in a world of blocks.
     - **Slider:** drop gathered into one step (in River network).
  7. **Waterfalls and rapids** (~25): SOUND in intent, with a known bug.
     - **The bug:** a rapids drop spread upstream can reach past the waterfall above it after that
       fall's curtain was set. The water above is pulled down, but the curtain and its cliff stay up,
       and the real drop moves up with no curtain. That makes walls of water on steep ground.
     - **Why rapids at 1 avoids it:** its spread never reaches past the next point, and points are at
       least 2 blocks apart. (Found this morning; not fixed, the rapids work is on hold.)
     - **Sliders:** a waterfall from (in River network); waterfalls get a curtain; rapids.
  8. **The cut readout** (~8): a diagnostic for the panel.
- **Falls at their lips** (`falls_at_lips`, ~45): a PATCH.
  - **Does:** moves each waterfall to where the ground falls below the water above it.
  - **Why it exists:** points are a few blocks apart and the water holds level along each piece, so the
    fall landed past the cliff.
- **Deltas** (`delta`, ~45): SOUND as a feature; its filter (~5 lines) is a PATCH.
  - **The filter:** drops a delta where its river met the sea early, or where the real ground at its
    head isn't low.
  - **Sliders:** deltas; only where flatter than; delta widths (river, great river).
- **The rest of `make`** (~60): SOUND. Rank runs, putting it together, the counts for the panel, and
  the cores.

## In the world

- **The carve** (`Carving::here` in `src/world/carving.rs`, ~70): SOUND at its core, with ~25 lines of
  PATCH.
  - **Does:** each column finds the nearest piece of river line. Inside it, it cuts the channel (its
    floor by the shape across) and fills its water. Beside it, it shapes the bank. In a lake, the water
    stands to the surface.
  - **The PATCHes:**
    - the sea is left alone;
    - a waterfall's curtain, and its cliff, are held up beside it;
    - a dry cell beside a lake is filled to one block under the surface;
    - where lakes overlap, the highest water shows;
    - a bank's levee is dropped inside a pond.
- **Banks** (`Banks::ground`, ~75): SOUND (David's profiles, step 3).
  - **Does:** fades from the shore back to the land, widening where it would be steeper than the
    steepest allowed, up to a limit, past which it's a gorge. The way it fades depends on which side of
    a bend it's on.
  - **Sliders:** each rank's fade over (widths); at least; a bend from; outside fades over; steepest
    bank; widens up to; terrace steps; the three ways.
- **Storage and lookup** (`Lake`, `Lakes`, the river lookup grid, `rivers.txt`, ~400): SOUND.
- **The Interpreter** (`ground_and_water`, ~20): SOUND.
  - **Does:** the carve lowers the recipe's own height, so the close-up, the far terrain and the light
    map all see it. Water shows where it stands more than a block above the carved ground.
  - **Show switch:** "rivers and lakes" (added today, 2d77921).
- **Blocks** (`terrain.rs`, ~15): SOUND. Water fills from the ground up to below its level; beds are
  gravel or sand.
- **The far terrain** (`render/distant.rs`, ~15): SOUND. It reads the carve through the recipe. A creek
  narrower than a far cell is blended in by its share of the cell.
- **The sea** (`terrain.rs`, ~20): SOUND, part of the terrain itself.
  - **Does:** fills every block below Z 128. `under_sea` is the same test for the carve, the close-up
    and the far terrain. The sea floor stays at least a block under the surface.

## Around it

- **Climate's glue** (`climate.rs`, ~60): SOUND. Each stage waits for the one before, and the ground is
  read without the rivers.
- **Panels** (`compiler_view.rs`, ~300): SOUND, but they'd change with the parts.
  - Rivers section ~30, River network ~55, Carving ~135.
  - The network picture and the river lines drawn over the map, ~80.
- **Tests:** ~700 lines.

## Totals

About 2,900 lines of water code, plus ~700 of tests. About 650 of the 2,900 are PATCHes, roughly a
fifth:

| Patch | Lines |
|---|---|
| Ponds behind rises | 260 |
| Searching for the real lakes | 140 |
| The route | 110 |
| Falls at their lips | 45 |
| Reaches into lakes | 30 |
| The carve's patches | 25 |
| Ending where it meets the sea | 15 |
| Smaller ones | ~25 |

Nearly all of them are in step 7 and the carve. Everything in steps 5 and 6 is SOUND. What the first
question is really about is the ground they read.

## The first question: does water follow the land, or does the land make way? (Builder's view)

### A. Water follows the land: found on the finished ground

- **How:** steps 5 and 6 run on the real ground, with everything in it, at a finer grain than now (8 or
  16 blocks a cell, not 32). The network is then the real drainage, and the lakes are the real basins.
- **What goes:**
  - the route;
  - the search for the real lakes;
  - ponds behind rises;
  - ending where it meets the sea;
  - most of the carve's patches.

  About 550 lines.
- **What it costs:**
  - **Reading the whole map's real ground.** The map is 2,250 patches across (72,000 blocks), so
    Climate reads about 5 million samples now. At 16 blocks that's about 20 million, and at 8 blocks
    about 80 million: 4 to 16 times the ground-reading Climate does today. The recipe is the slow part,
    so that's seconds to tens of seconds on the cores.
  - **Memory.** The fill and spill's memory grows the same way: about 1 GB at 16 blocks, and several
    GB at 8.
  - **Pits.** Rough ground is full of them, and filled, every crag pit is a lake: David's complaint,
    multiplied. So it needs a new rule: pits shallower or smaller than a setting are cut through, and
    the rest are lakes. That rule is the land making way, in miniature.
  - **No valleys.** The terrain wasn't made by water, and noise hills have no valleys that join up.
    Rivers would follow whatever is there: staircases of waterfalls down crags, lakes in every hollow,
    rivers wandering over flats. Long rivers from the highlands across the flats would appear only where
    the land happens to lead that way.

### B. The land makes way: water planned first, the land shaped round it (Génevaux-style)

- **How:** keep the plan on the broad ground, steps 5 and 6 as they are. The broad shapes (plates,
  ridgelines, peaks, rise) already drain sensibly, since the plan is made from them. Then the carve
  shapes the land along each river into its valley:
  - the floor sits at the planned water level;
  - the sides rise to meet the land;
  - the fine layers (crags, hills, summit crags, torn coast) fade out toward the floor.

  Planned lakes work the same way: their basins are shaped and filled to the planned level, and the
  fine layers fade inside them. The banks are already a small version of this (the land cut to the
  shore over a fade, up to a limit). B grows them into valleys and lets the fine layers yield.
- **What goes:**
  - the route;
  - ponds behind rises;
  - the search for the real lakes;
  - falls at their lips (a fall goes where the plan steps, and the land is shaped to it);
  - ending where it meets the sea;
  - the lowest-ground-across rule.

  About 600 lines.
- **What it costs:**
  - **A valley shape for each column.** About 200 new lines, and cheap to run: the carve already finds
    the nearest river for every column.
  - **A second reading near rivers.** Each column near a river needs the broad height as well as the
    full one, to fade between them. That's one more recipe call per column near a river, for chunks and
    the far terrain.
  - **A new look.** Mountains change near rivers: every river gets a valley or a gorge. How wide and
    steep those are needs sliders, by rank and perhaps by how rough the land is.
  - **Coarse paths.** The plan is in 32-block patches, so each river's path is the plan's, smoothed
    and meandered. A river never threads a fine gap in the hills; the hills open for it.
- **The pipeline stays linear.** The plan reads the broad ground, which comes before the fine layers.
  The valleys are applied along with the fine layers, and nothing feeds back.

### My view: B

The big shapes already drain well. It's the fine layers that break the rivers, and they're noise with
no idea of water. Making them make way removes the gap where it starts, instead of patching it river by
river. It's also what gives David:

- long rivers from the highlands across the flats;
- valleys instead of ponds in rough ground;
- waterfalls where the plan steps.

A is the simpler idea. But on noise terrain it trades today's patches for a pit-cutting rule, which is
B in miniature, and leaves rivers going wherever the noise sends them.

There's also a middle way: B, then a fine fill and spill inside each valley as a check, with a lake
wherever the shaped valley still holds water.

**Survives either way:**

- steps 5 and 6;
- the sizes and profiles;
- steps, waterfalls and curtains (with the rapids bug fixed);
- the banks (they grow into the valleys in B);
- the carve's core;
- storage;
- the Interpreter, the far terrain and the sea.

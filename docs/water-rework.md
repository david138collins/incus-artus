# Water rework: the Builder's assessment (2026-10-31)

This answers Counsel's eight questions on `vision/water-from-the-start.md`, for David and Counsel. It's
an assessment only; nothing here is built. The parts of today's code are named as in
`docs/water-inventory.md`.

## In short

- **The compressor works, and it's mostly what the banks already do, made gentler.** Today's banks are
  a compressor turned all the way up: at the water's edge, every bank is cut down to the shore,
  whatever its height. The compressor leaves low banks flush and only partly lowers high ones, so it
  changes *less* land than today, not more. It's about 150 lines, and costs nothing you'd notice.
- **The water's height still comes from the real ground** (the lowest ground across the river), so
  water never stands in the air. The plan gives the path and the joins, the real ground gives the
  height, and the land gives way where a rise is in the path. That's Counsel's "the water follows the
  land at the large scale, and the land gives way at the small scale".
  - *Correction to my inventory:* in this version of "the land makes way", the lowest-ground rule
    stays.
- **The best follow-the-land piece:** a local re-route only where the cut would be deeper than a
  setting. It reuses the route's fill and spill on a small box, instead of a corridor along every
  river.
- **Lakes picked first:** feasible and cheap. It retires the biggest patch (ponds behind rises) and the
  crag-pit ponds.
- **One water by area** (Shreve growth, shape from slope): a clean swap for the rank tables, about the
  same size.
- **Petering out by distance:** about 20 lines. **Dry beds:** a later step, about 60 lines, once
  rivers come first.
- **The order:** a measure first, then the compressor, lakes, the local re-route, sizes by area,
  petering out, and dry beds. Each goes beside today's rivers behind a switch.

## 1. The compressor carve

*David's answers (section 8) change one thing here: flush, level with the water, is the default
everywhere. The ratio applies only to cut banks on the outside of bends and to some lake shores, so
there's no "flush up to" threshold to tune.*

### What today's banks already do

For each column beside a river, the bank starts at the shore at the water's edge and rises back to the
land's own height over the **fade**:
- the fade is the fade widths times the river's width, and at least the fade least;
- it widens, up to "widest", where it would be steeper than "steepest".

So at the edge everything is brought to the shore: flush, or a 1-block ledge for "ledge then slope".
The change fades to nothing at the fade's end. In compressor terms that's:
- an endless ratio at the edge;
- 1:1 at the fade's end;
- a soft knee (the S-curve);
- no threshold;
- nothing along the river.

*Note:* `rivers-balanced` sets all three bank ways to "ledge then slope", so every shore in it is a
1-block ledge today. The default makes the inside of bends and straight runs flush.

### The compressor version

**The signal** is how high the land stands above the water at the nearest point of the river. The
carve already finds that point, and its water level, for every column.

| Control | What it does | In the code |
|---|---|---|
| **Flush up to** (the threshold) | At the water's edge, land this little above the water is brought flush. It's a small change, and it's the dial for the flush share. | new, in `Banks::ground` |
| **Ratio** | Land higher than that keeps a bank at the edge. At 4:1, a hill 20 blocks above the water keeps a bank of about 4½. | new, in `Banks::ground` |
| **Knee** | Eases between the two, so there's no line where the banks begin. | new, in `Banks::ground` |
| **Feather across** | The ratio eases back to 1:1 (no change) over this distance. | today's fade (fade widths, at least), widened for tall land (steepest, widest) |
| **Attack and release along** | Where a big cut is coming, the feather widens ahead of it and narrows again after it, so a valley opens gradually instead of a notch. | new: per river point, one pass upstream and one downstream along each reach, after the water is laid |
| **The water rises to meet its banks** | The water may sit at most this far below its lower bank. | new, in the steps |

**Why "flush up to" and not "left alone".** Counsel's threshold leaves land within a block or two of
the water alone. Left alone, a bank 1 or 2 blocks above the water stays a 1 or 2-block step, which
isn't flush. So I'd turn it round: land up to the threshold is brought flush, lowered by at most that
much. The land that's truly left alone is everything beyond the feather.

**The look-ahead along the river.** At each river point, I'd take how much the land at its edges
stands above "flush up to": the cut it needs. That's spread upstream and downstream like a cone (one
pass each way along the reach), so it tapers off. The feather at a point is then wide enough that the
sides are no steeper than "steepest" for the tallest cut nearby. That's today's widening, smoothed
along the river.

**The water rising to meet its banks.** The water already sits at the lowest ground across each point.
It only drops further below its banks where the steps (flat runs, then drops) or rapids put it lower.
A limit there ("at most this far below its lower bank") makes more flush shores without touching any
land. It never raises the water above its lower bank, so water never stands against air.

**The channel itself is still cut.** The bed below the water has to be dug; the compressor is about the
banks.

### David's three conditions

1. **Well feathered, pulling the land down gently.** The change fades to nothing across (the feather)
   and along (attack and release). Tall land widens the feather, so there are no notches.
2. **As little change as possible, as gradually as possible.** Low banks change by at most the
   threshold. Tall banks keep a share of their height at the edge (one over the ratio). Nothing
   changes beyond the feather. Today's banks cut everything to the shore, so this changes less.
3. **At least half the inland shore flush.** The threshold is the dial, and a readout gives the
   flush share, so it can be set to half or more.

### Cost

- **Code:** about 150 lines: the bank curve about 40, the pass along the river about 40, and the
  readout about 60.
- **Making the rivers:** the pass along the river reads two more heights for each point (about 100,000
  readings), well under a second.
- **In the world:** a few more sums for each column, nothing measurable.

### On David's world

Not measured yet, since the CPU is running hot. Here's what I'd expect. The water sits at the lowest
ground across each point, so on gentle ground the lower bank is already within a block or two of the
water nearly everywhere. On a slope, the higher bank isn't. So half flush should need only a small
threshold (2 or 3 blocks) in the hills and lowlands. The mountains will be well under half (gorges)
unless the ratio is high there.

**The measure** is stage 0 below, run on today's rivers as the baseline. It's a readout on the Carving
panel, sampled every few points so it costs about a second:
- the flush share of the inland shore;
- the land lowered and raised (average, most and total);
- the steepest pull-down;
- the deepest cut.

### Counsel's cheap look-aheads

- **The sliding window and the envelope:** yes. The pass along the river is exactly this, as a cone
  rather than a box, so it tapers.
- **The distance to the river:** already there.
- **The pyramid of highest points:** not needed for the compressor. Perhaps later, if the re-route
  ever has to look further than its box.
- **The plan itself:** it already looks ahead at the mountains' scale.

## 2. Follow-the-land pieces on top, ranked

1. **A local re-route only where the cut would be deeper than X. Best value.**
   - **How:** once the water is laid along the plan's line, find the stretches where it runs more than
     X below the lowest ground across. The panel already counts these as "deep cuts". Only there, run
     the route's fill and spill in a small box round the stretch, pinned where the cut is small again.
     That finds a lower way round if there is one, and only works where it's needed.
   - **Cost:** a few hundred small boxes (about 50 by 30 cells each), against today's corridor, which
     runs 256 blocks either side along every river.
   - **Retires:** the corridor along every river.
   - **Switch:** route everywhere, only where needed, or off.
2. **A nudge toward lower ground. Cheapest, smaller gain.**
   - **How:** where the cut is smaller than X but still more than a few blocks, slide the line sideways
     toward the lower side, within a width or two. That's like the lean and "keep within", and costs
     about 4 readings per point there. It's smoothed along the river so it doesn't wiggle.
3. **The webbing, near each reach only. The most natural, and the most work.**
   - **How:** a soft map of the passages in a band round the plan, which the route prefers.
   - **Why later:** it overlaps with piece 1, since fill and spill already finds the lowest way
     between two points. It's worth it if David wants rivers threading the crags everywhere, or for
     roads later.
4. **The pyramid of highest points.** A tool rather than a piece, and only needed if piece 1 has to
   look past its box.

## 3. Lakes picked before the water moves

**Feasible as sketched, and cheap.**

- **The list.** Step 5 already has every basin: each lake patch knows its outlet, and the network
  groups them by it. List the basins with their area, depth, height and how rough the ground round them
  is. Roughness needs a few readings of the real ground per basin, and there are hundreds of basins,
  not millions.
- **Picking.**
  - Keep every basin deeper than some depth: passing one over means cutting or filling that deep.
  - Pick the rest by count or by spacing.
  - Set a smallest size for a lake, and for a pond.
  - Weight the choice for rough ground and for height.
  - The panel shows how many basins were picked.
- **Basins not picked.** The water flows straight through. The river's water stays at or below the
  ground, so it cuts the basin's rim, and the compressor feathers that into a valley. No pond.
- **Picked lakes on the real ground.** Today's fill and spill for a lake (`fill_region`, sound) runs
  on the picked basin only, and keeps its main pool rather than every crag pit. Its surface is the
  real rim's lowest gap, so it never leaks against air.
- **Rises along rivers** are always cut and feathered. Ponds behind rises go entirely.
- **Cost:** about 100 new lines (the list, the picking, sliders, an overlay). It removes about 260
  (ponds behind rises) and the lake search's side effects.
- **The catch:** a deep basin passed over means a deep cut, or a causeway. So deep basins should
  always be lakes, and the choice is really about the shallow ones.
- **Stamped lakes,** placed at points like the peaks, with a basin dug and filled, are separate and
  can come later.

## 4. One water by area

- **Count springs instead of orders.** Each river point's magnitude is the number of springs above it
  (Shreve), and a join adds both. That replaces stream order (Strahler) in step 6, at about the same
  size.
- **The area.**
  - A spring's area is 1.
  - A join keeps a share of both areas (David: "most of their area").
  - Running adds a little for each stretch run.
  - The biggest area is capped.
  - With Shreve, a great river can gather thousands of springs. A join that keeps slightly less than
    all (say 90%) holds sizes in check while still growing fast.
- **The names at thresholds:**
  - spring: 1 to 2;
  - stream: 3 to 8;
  - river: from about 16 (8 wide by 2 deep);
  - great river: from about 200 to 250 (64 wide by 3 or 4 deep).

  The river and great river thresholds are David's widths restated as areas, for David to set.
- **The shape from the slope.** Width times depth times how full the bowl is gives the area. How
  broad against how deep comes from the slope along the plan: the broad ground's fall over its run.
  The plan already has that, so the planned bed gets a job again.
  - Flat ground: broad and shallow.
  - Steep ground: narrow and deep.
  - A deepest shape and a broadest shape as limits.
- **The profiles drawing** then gives only the bowl's shape: one shape, or one by how broad.
- **It replaces:** a spring's width and depth; times wider and times deeper (four each); the shapes
  (four); stream orders per rank; and the fade widths by rank, which become a fade by width. That's
  about 70 lines replaced by about 80.
- **What David would tune:**
  - a spring's area;
  - the share kept at a join;
  - growth by running;
  - the slopes for fully broad and fully deep;
  - the broadest and deepest shapes;
  - the biggest area;
  - the name thresholds;
  - "size varies" (kept).

## 5. Petering out, and dry beds

**Petering out by distance to the next water.**
- Lone streams already walk downhill to the next water and measure the way.
- The new rule: a stream ends only if the next water is more than N away, and then it soaks away on
  the first flat stretch past N.
- It's about 20 lines in step 6, and one slider (N), perhaps with a second for how flat ground has to
  be to soak.

**Dry beds, as a later step after Temperature and Rain.** Putting rivers first in the Climate tab
makes this possible.
- **How:** each river point's area is compared with a threshold that rises with heat and dryness.
  Below it, the bed stays but holds no water (sand or gravel, by mineralogy), and its falls get no
  curtain.
- **Cost:** about 60 lines: a wet or dry mark for each point, a check in the carve, and two sliders.
- **It stays linear:** it reads the rivers and the climate, and changes no shapes.
- **The catches:**
  - A dry stretch upstream of a wet one is fine to the eye (water underground).
  - A picked lake fed only by dry beds still holds its water.

## 6. What each piece retires, and what stays

| Piece | Retires |
|---|---|
| Lakes picked first | ponds behind rises (260); the crag-pit pools and most of the lake search (about 100 of 140); most of "reaches into lakes" (30); the no-levee-in-a-pond patch |
| The compressor | nothing outright; it reshapes the banks (sound), and makes cutting through safe everywhere |
| The local re-route | the corridor along every river (110 lines become about 40, reused) |
| Sizes by area | the rank tables (sound for sound) |

**What stays:**
- the flow, springs, mouths, growing and lone streams;
- the line;
- the lowest ground across (the water's height comes from the real ground);
- only falling downstream, and ending at the water it meets;
- ending where it first meets the sea (sound in this design, since the plan's coast and the real one
  differ);
- steps, and waterfalls (with the rapids bug fixed);
- falls at their lips (still needed, since points are a few blocks apart);
- deltas;
- the carve's core, storage, the Interpreter, the far terrain and the sea.

About 450 of the 650 lines of patches retire.

## 7. The order to build in

Each stage goes beside today's rivers behind a switch. Today's rivers (d816b60) stay selectable until
the new way is better.

0. **The measure, and the rapids bug fix.**
   - The readout from section 1, on today's rivers.
   - An overlay of the shore in the close-up: flush in one colour, ledges in another, tall cut banks
     in a third.
1. **The compressor banks.**
   - A switch: banks by today's fade, or by the compressor.
   - The measure is the same readout. David tunes the threshold, ratio, knee and feather against it.
2. **The look-ahead along the river** (attack and release).
   - The overlay: the feather's width along each river.
3. **Lakes picked first,** with ponds behind rises off.
   - The overlay: every basin, with the picked ones filled.
   - The readout: basins, lakes and ponds picked.
4. **The local re-route, only where needed.**
   - The readout: stretches re-routed, the deepest cut before and after, and the time taken.
5. **Sizes by area** (Shreve, shape from slope, names).
   - The overlay: rivers coloured by name.
   - The readout: the length of each name.
6. **Petering out by distance.**
7. **Rivers first in the Climate tab,** with snowmelt by height. It's decided and small; it can come
   at any time, but before dry beds.
8. **Dry beds.**

## 8. Questions for David, and risks

**David's answers** (2026-10-31, via Counsel; also in `vision/water-from-the-start.md`), and what
each changes:

1. **Flush means level with the water.** "1 block ledge you can't get out of the water from." The test
   is that you can walk out of the water.
   - "Ledge then slope" isn't flush. `rivers-balanced` uses it for all three bank ways, so its inside
     and straight ways go back to smooth (or slope, or terraces). All three of those are level with the
     water.
   - The readout counts a shore column as flush when its ground's top is at the water's surface.
2. **No target share: flush is the default everywhere.** "i'd like to achieve this by them just being
   normally flush and some corner banks are jutting up and the lakes and ponds aren't always all
   flush."
   - The exceptions are cut banks on the outside of bends, and some lake and pond shores.
   - **This changes the compressor (section 1).** There's no "flush up to" threshold to tune: the edge
     is brought flush everywhere, as today's flush ways do.
   - The ratio applies only on the outside of bends, where the cut bank juts up. It fits today's
     choice of way by the bend's side: the outside's way becomes "a bank of 1/ratio of the land's
     height".
   - **Lake and pond shores** keep their own height where the land rises steeply from the water (a
     ratio for lakes), and stay flush where it's gentle.
   - **What still makes it gentle:** the feather across and the attack and release along. The
     flush-share readout stays as a check.
3. **X, the cut before a river looks for a way round:** "a sane range to tune with". A slider from 2 to
   40 blocks, about 8 to start.
4. **Gorges are better up high; down low, rivers should meander round the terrain.** "they're better
   up high. its better for the rivers to meander around the terrain down low near sea level."
   - X could grow with height: a small X near the sea (look for a way round early, so rivers meander),
     and a large one high up (cut gorges).
   - That's two values, "near the sea" and "high up", blended by the water's height.
   - The meanders already grow on flat ground, which is mostly low.
5. **Lakes: a count, like the mountain peaks' range.** "a count threshold, like there is for the mountain
   peaks perhaps, where i can select a range from top to bottom."
   - The basins are ranked **by volume** (area times average depth: "how much lake"), and David picks a
     range from the Nth to the Mth, with the count shown.
   - **The catch with "deep basins always kept":** the two agree as long as the range starts at the
     top, since the deepest basins rank near the top by volume. If David's range leaves out the
     biggest basins, those are passed over, and passing one over means cutting its rim (or filling it)
     as deep as it is.
   - **My suggestion:** keep the range, plus an "always a lake if deeper than" slider that can be set
     out of reach to switch it off. The readout shows the deepest cut that passing a basin over makes,
     so David can see what leaving one out costs.
6. **Area is the size, and the slope only nudges the shape.** "we should try to use area as a simple
   metric for size, slightly shifted by being broader or deeper depending on rise-over-run."
   - Each area has a set shape: a broadness, growing gently with area.
   - The slope tips it a little: broader on flat ground, deeper on steep ground. A strength slider
     starts low, so the slope can't dominate.
   - The deepest and broadest limits stay as guards.
7. **Crags beside water, lowered or smoothed away:** not answered yet. Counsel suggested lowered first,
   which is the compressor on its own, with no second reading of the ground. That's what stage 1 would
   build.

**Still open:** the area thresholds for a river and a great river (about 16, and 200 to 250).

**Risks:**
- **Deep basins passed over** make deep cuts. Keeping the deep ones as lakes avoids it.
- **Tall crags keep tall banks at a low ratio,** so the mountains' flush share will be low by design.
- **Two ways side by side** for a while (the switches) means more code until the old way retires.
- **The readout is an estimate.** Measuring the land change exactly means reading every shore column
  (last night's 30-second scan); the readout samples instead (about a second).
- **Shreve sizes can run large** on big basins. The share kept at a join and the biggest-area cap
  control it.

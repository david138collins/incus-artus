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
   - *Folded in (David, 2026-10-31):* the outside of a bend's fade eased by how sharp the bend is,
     instead of switching at "a bend from". The hard switch left straight-edged notches in the banks
     where a river straightened out of a bend (David's screenshot, with "outside fades over" below 1).
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
7. **Crags beside water are lowered** by the compressor (decided with stage 0's go-ahead), with no
   second reading of the ground for smoothing. That's stage 1, when David gives the word.
8. **No great river:** "great rivers aren't really a 'thing', as we have no discrete types, we should
   just make sure that its rare two whole rivers join up."
   - The names stop at river, from an area of about 16 to start.
   - What David wants instead is a behaviour: two whole rivers joining is rare. Section 9 has how
     it could be controlled, shelved until it's seen to be needed.

**Risks:**
- **Deep basins passed over** make deep cuts. Keeping the deep ones as lakes avoids it.
- **Tall crags keep tall banks at a low ratio,** so the mountains' flush share will be low by design.
- **Two ways side by side** for a while (the switches) means more code until the old way retires.
- **The readout is an estimate.** Measuring the land change exactly means reading every shore column
  (last night's 30-second scan); the readout samples instead (about a second).
- **Shreve sizes can run large** on big basins. The share kept at a join and the biggest-area cap
  control it.

## 9. Stage 0, built (2026-10-31)

**What it is:**
- **The rapids fix.** Rapids spread only within their own run, back to the foot of the drop above,
  squeezed to fit a short run. Each waterfall's curtain is set after all the rapids. A test reproduced
  the bug first: a 4-block drop with no curtain, on a slope of falls and rapids in turn. "Rapids at 1"
  is no longer needed.
- **The shores readout** on the Carving panel, measured on every river as it's made:
  - every 8th point, both sides;
  - the first dry block beside the water: flush (its top level with the water), a 1-block ledge, a
    taller bank, or below the water;
  - across the banks: the land lowered and raised (average, most), and the steepest pull-down.
- **The overlay:** "shores in the close-up", under the readout. The block beside the water is green
  where flush, amber for a ledge, red for a taller bank, and magenta below the water.

**Today's rivers on David's world** (measured offline on 4 cores, with the rapids fix):

| | default | rivers-balanced | rivers-balanced, smooth banks |
|---|---|---|---|
| Flush | 0.9% | 1.8% | **94.5%** |
| 1-block ledge | 97.4% | 93.5% | 1.3% |
| Taller bank | 1.5% | 3.0% | 2.3% |
| Below the water | 0.2% | 1.7% | 1.9% |
| Land lowered, average (most) | 2.3 (76) | 1.7 (81) | 2.1 (82) |
| Land raised, average (most) | 0.05 (30) | 0.03 (22) | 0.02 (21) |
| Steepest pull-down (up a block across) | 50 | 56 | 56 |
| Deepest cut; points cut deeper than 6 | 76; 9,810 | 81; 6,038 | 81; 6,038 |

**What it says:**
- **Almost every shore is a ledge today because of one setting.** Both presets set all three bank ways
  to "ledge then slope". Set to smooth, 94.5% of shores are flush, for 0.4 of a block more land
  lowered on average. That alone meets "flush by default": switch the inside and straight ways to
  smooth, and keep "ledge then slope" for the outside of bends if David wants cut banks there.
- **About 2% of shores stand below the water** (water against air) in `rivers-balanced`. They're a
  target for stage 1.
- **The cuts through rises are what's harsh:** up to 81 blocks deep, with sheer sides (the steepest
  pull-down is 50 to 56 blocks up for one across), and 6,000 to 10,000 points cut deeper than 6.
  That's what the compressor's feathering (stages 1 and 2) and the local re-route (stage 4) are for.

### Two whole rivers joining, rarely: how it could be controlled (SHELVED)

**[SHELVED 2026-10-31, David via Counsel:** "i don't think we need to try and fix problems before we
even see them with re things like river-to-river joining." Kept on file in case whole rivers join too
often once the rework is in. It's in no stage.**]**

Today's controls don't aim at it:
- **"Mouths apart at least"** decides how many systems reach the sea. Fewer, bigger systems make
  big-into-big joins more likely, not less.
- **"How readily rivers split"** keeps a side branch only if it carries enough of the main branch's
  springs. At low settings, the branches that survive are exactly the near-equal ones: whole rivers
  meeting.
- **A rule against river-to-river joins can't reroute water.** The drainage is fixed: a branch's water
  has to go through its junction.

**The way that works: limit what a side branch may carry.**
- When the network grows a river up from its mouth, a side branch may carry at most a share of the
  springs the main branch carries where they meet. The slider would be "a tributary carries at most",
  about a third to start.
- Springs past that share, the furthest up the branch first, aren't taken by the river. They run as
  lone streams instead: they join something smaller, reach a lake, or peter out.
- So a river mostly takes in streams, and a junction where both sides are rivers happens only where
  the share allows it.
- It also gives "more collected to one river": the main stem keeps growing while side branches stay
  smaller.
- **Cost:** about 30 lines in the network's growing pass, one slider, and a readout of how many
  junctions join two rivers (both at least river size), against how many join a stream to a river.
- **Where, if it's ever taken up:** with stage 5 (sizes by area), since "river size" is an area threshold there. The readout
  could come earlier, counting ranks.
- **The catch:** fewer springs feed rivers in big basins, so some of those uplands get shorter lone
  streams instead. With petering out by distance (stage 6), they keep running if there's water within
  N to join.


## 10. Stage 1, built (2026-10-31): the compressor banks

David's go-ahead after stage 0. Before building, a probe of what made stage 0's numbers harsh found
two causes:
- **The shores below the water** were where a river steps down at a bend. The water there came from
  the upper piece and the bank from the lower, cut 2 or 3 blocks below the water beside it.
- **The steepest pull-downs** were seams between two rivers' banks: each block took only its nearest
  river's bank, so the land went 178, 218, 178, 218 across.

So stage 1 is the compressor as section 1 describes it, with David's answers, plus fixes for those
two causes. It's a switch, "compressor banks" (off by default, saved with the world). The details are
in `docs/rivers.md` section 3:
- **Every stretch of river near a spot has its say:** the land comes down to the lowest of their banks
  (each stretch by its nearest piece), so there are no seams.
- **Another stretch's bank cuts no lower than this one's shore,** coming down from it no steeper than
  the steepest bank. It never raises land.
- **The block beside any river's water is at least its shore.**
- **The first dry block is the shore itself.**
- **The feather widens so the curve's steepest part stays within the steepest bank.**
- **Cut banks on the outside of bends keep a share of the land's height** ("cut banks keep", 0.25 to
  start).

Two tries were measured and dropped along the way. Holding the land up raised it by up to 91 blocks
beside lower rivers. And letting every piece of one river have its say held up a little ramp below
every step: 80% flush.

**On David's world** (`rivers-balanced`, smooth banks, 4 cores), off against on:

| | off | on |
|---|---|---|
| Flush | 93.6% | 92.6% |
| Ledges | 1.7% | 0.4% |
| Banks (mostly the cut banks) | 2.7% | 6.6% |
| Below the water | 2.0% | 0.4% |
| Land lowered, average | 2.1 | 2.6 (the gentler fade) |
| Land raised, average | 0.04 | 0.05 |
| Steepest pull-down | 40 | 38 (a waterfall's own cliff) |
| Carve, per block near a river | 0.6 µs | 1.0 µs |

**Not in this stage:** the attack and release along the river (stage 2), and "the water rises to meet
its banks" (it changes the water, not the land; for later, with the steps).


## 11. David's preset after stage 1, annotated (2026-10-31)

David saved the best settings with stage 1 as `presets/compiler/default.toml`, at 11:35, writing: "many
settings on the climate page i've put all the way down, because they do nothing, or are harmful."
Here is each water setting with its value, what it does, and whether it acts at these values. "Min"
and "max" mean the slider's end. The changes from `rivers-balanced` are in **bold**.

### Rivers (step 5)

| Setting | Value | What it does here |
|---|---|---|
| snowmelt: extra water | 5 (max) | Almost nothing. "Higher than" is 1,020 (near max), so almost no ground qualifies. |
| …colder than | 1 (max) | Everything counts as cold. |
| …higher than (Z) | 1,020 | Turns snowmelt off in effect. |
| springs (slope feet): extra water | 20 (max) | Only how far lone streams run, since "water it needs" is 1. |
| a lake from (blocks deep) | 24 | Step 5's lakes are basins 24 or more deep. |
| drawn from | 952 | The picture only. |

### River network (step 6)

| Setting | Value | What it does here |
|---|---|---|
| plain or relative | plain | |
| springs from Z | **128 (min)** | Springs anywhere from sea level up to 700: 573 of them. |
| up to Z | 700 | |
| higher than surroundings, within | 40, 2,000 | Inert: relative height is off. |
| water it needs | 1 (min) | Any land qualifies. |
| spaced at least / varies | 2,500 / 0.5 | |
| a lake feeds a river from (patches) | **150** | Only big lakes start rivers. |
| mouths apart at least | 3,000 | 70 mouths. |
| how readily rivers split | **0 (min)** | One trunk per mouth; other springs run as lone streams. |
| a lone stream runs | **7,600** | Lone streams run far: only 9 of 573 springs fizzle. |
| stream orders per rank | **4 (max)** | **Every river is a creek.** The highest stream order on this world is 4, so nothing reaches stream rank. That's why only "creeks fade over" did anything: every stream, river and great-river size, shape and fade is unused. |
| drop gathered into one step | **0 (min)** | No steps: the water follows its ground block by block. |
| a waterfall from | **25** | Only drops of 25 or more were falls. With no steps, every point dropped a little, and rapids had no room. That left 4,895 sheer drops (walls up to 24 tall) until the fix this afternoon: every sheer drop of 2 or more now gets a curtain. |

### Carving (step 7)

| Setting | Value | What it does here |
|---|---|---|
| cores | 12 | |
| corridor / grain | **64 / 16 (max)** | A narrow, coarse way-finding: rivers can't get round much. |
| corner cutting | **5 (max)** | |
| wander / its swings / lean | **48 (max) / 3,000 (max) / 2 (max)** | **Inert:** "most sideways" is 0. |
| meander every / size / full below | **4 (min) / 0 (min) / 0.001 (min)** | Meanders off. |
| most sideways | **0 (min)** | No sideways moves at all: the line is the route, smoothed. |
| keep within | **0 (min)** | Inert (nothing moves sideways). |
| a spring's width / depth | **4 (max) / 2** | |
| times wider (each rank) | **5 (max)** | Only the creek's applies: creeks grow from 4 to 20 wide (3 to 54 with "size varies"). |
| times deeper | about 2 | Likewise. |
| size varies / shape varies | **0.25 / 0.66** | |
| shapes | edges 0.26–0.41, floors 0.36–0.49, bowls 1.15–2.1 | Only the creek's shape applies. |
| fade over (widths) | **creeks 2.05; others 0** | The others are unused (no rank above creek). |
| …at least | **0 (min)** | |
| bank ways | **slope, slope, slope** | Flush. |
| a bend from | **0.02 (min)** | Every point counts as a bend. |
| outside fades over | **1 (max)** | The outside's the same as the inside, so the bend side doesn't matter (and no notches). |
| steepest bank | **0.2 (min)** | The gentlest: banks widen as far as "widens up to" allows. |
| widens up to | 6 (max) | |
| terrace steps | 4 | Inert: no terraces. |
| compressor banks | **on** | |
| cut banks keep | **0 (min)** | Flush on the outsides too. |
| look round each lake / a pond from | 64 / **4** | Pools must be 4 deep: fewer crag-pit ponds. |
| ponds behind rises | **off** | Every rise is cut through. |
| cut through rises up to / a pond spreads at most | 4 / 512 | Inert: ponds are off. |
| curtains | on | |
| rapids | **10 (max)** | Rapids where there's room. With no steps, there's rarely room. |
| deltas | **off** | Their settings are inert. |

### Measured on David's world (today's bake, 6 cores)

- **Making the rivers** took 1.0 s.
- **Springs and mouths:** 573 springs (121 river heads, 157 joined, 286 into a lake or the sea, 9
  fizzled) and 70 mouths.
- **Rivers:** 805 stretches of river with 140,957 points, all creeks. River patches by stream order 1
  to 4: 14,773, 5,139, 1,563, 92.
- **Sizes:** widths 3 to 54 (average 11); fades 7 to 110 (average 22).
- **Lakes and ponds:** 21 lakes (46 pools), no ponds behind rises.
- **Shores (after the curtain fix):** 95.7% flush, 3.6% ledge, 0.5% bank, 0.1% below the water.
- **Land:** lowered 4.1 on average (most 196), raised 0.01 (most 39). Steepest pull-down 51.
- **Cuts: deepest 439 blocks; 25,098 points cut deeper than 6.**

### What it says, and what to prepare next

- **David has switched off most of the patches and the extras:** ponds behind rises, steps,
  meanders, wander and lean, deltas, cut banks, the bend sides, terraces, snowmelt in effect, and every
  rank above creek. What's left is the plan (springs, mouths, one trunk each), the route, smoothing,
  creek sizes, gentle flush compressor banks, lakes, curtains and rapids.
- **The harm now is the cuts.**
  - With ponds off and a narrow, coarse corridor, rivers saw through whatever is in the way: up to
    439 blocks deep, and 25,098 points more than 6.
  - The gentlest banks then spread each cut wide, so the land is lowered by 4 blocks on average
    and up to 196.
  - This is what stage 4 (a local re-route where the cut would be deeper than X) is for, and it now
    looks like the most useful next step, ahead of stage 2.
- **The ranks are off in practice**, so sizes by area (stage 5) would bring sizes back as one
  continuous number, and make the size sliders do something again.
- **Inert sliders could be greyed out** (David's rule: grey out, don't hide):
  - wander, swings and lean when "most sideways" is 0;
  - cut and pond reach when ponds are off;
  - the delta settings when deltas are off;
  - rise and around when relative height is off;
  - terrace steps without terraces;
  - each rank's size, shape and fade when no river reaches that rank (the panel can count it).

  It's a small change, worth doing with whichever stage comes next.
- **Waterfalls:** with "a waterfall from" at 25 and no steps, nearly every fall is now a short curtain
  of 2 to 5 blocks (4,902 of them). "Lots of waterfalls high up" might want a look at the step and the
  fall height together.
- **Snowmelt** is in effect off ("higher than" 1,020). With rivers moving first in the Climate tab, it
  goes by height anyway.

## 12. Stage 4, built (2026-10-31): the way found only where it's needed

David's go-ahead ("got it, let builder know to proceed"). Built as section 2's piece 1. The details
are in `docs/rivers.md` (step 7, "Where the way is found"):
- **The switch:** "found:" everywhere, only where needed, or off.
- **The deep cut:** "near the sea" and "high up", 2 to 40, both 8 to start, blended by the water's height
  between the sea and the highest river.
- **The box:** round each stretch whose plan would cut deeper, out to where the cut is small again.
  It reaches half the stretch's length or 4 times its cut, 32 to 256 blocks, at an 8-block grain.
  A 4-block grain took 6.3 s on David's world for the same result.
- **Greyed out:** the corridor and grain outside "everywhere", and the sliders that can't act.
- **The readout:** the stretches found and re-routed, the deepest cut before and after, and the time.

**On David's saved preset** (today's bake, 6 cores):

| | everywhere (his) | only where needed | off |
|---|---|---|---|
| Stretches too deep, re-routed | — | 326 of 374 | — |
| Deepest cut along the plans, estimated | 139 | 139 | 139 |
| Along the lines found | 117 | **90** | 139 |
| Deepest cut as laid | 439 | 435 | 446 |
| Points cut deeper than 6 | 25,098 | 23,825 | 29,787 |
| Land lowered, average (most) | 4.1 (196) | **3.6** (212) | 4.3 (187) |
| Steepest pull-down | 51 | **30** | 59 |
| Flush | 95.7% | 96.0% | 96.4% |
| Time finding the way | 0.3 s | 1.6 s | 0.1 s |

**What it found: most deep cuts don't come from the path.**
- **Where it works:** where a plan climbs over a ridge, the re-route goes round it. The plans' own
  deepest cut drops from 139 to 90, the steepest pull-down from 51 to 30, and the land lowered by 13%.
- **What it can't reach:** the cuts as laid still go to 435, and there are still 23,825 points more
  than 6. The water's level can only fall, and with ponds off nothing holds it up. So after any dip,
  the river stays at the dip's level and cuts through every rise after it, until the ground drops
  below it again.
  - About 80% of those deep points come from dips earlier along the river's own stretch.
  - About 20% come from a stream upstream, through a junction.
  - Rivers here follow their plans and seldom pass a crag, so the dips are mostly the hills (12 up and
    down).
- **That's a question of the level, not the path:** a dip "passed over" is filled or cut. Today it's
  always cut. The cheapest answer is to fill small dips: the water runs level over a dip shallower
  than a setting, the bed raised under it, instead of dragging the whole river down. That was the
  "passed over: filled" half of stage 3 (lakes picked first).
- **A level that changes the land least,** cutting rises and filling dips by turns, is the fuller
  answer. It's a known method (isotonic regression, "pool adjacent violators") that runs in one pass
  along a reach. Either is for David to choose.

**Not changed:** "everywhere" is still the default, so David's preset keeps its look until he picks
"only where needed".

## 13. After stage 4: David's notes, and the next step prepared (2026-10-31)

**David, after trying stage 4:**
- "everywhere setting on the route works pretty good. it doesn't take any time, either. im updating
  in 2-3 seconds with it."
- "i think the settings wander through keep within under the line may be useless entirely. they only
  seem to make it worse at any setting besides off. if there's nothing you think is useful there it can
  be cut."
- "im questioning the value of bends, either. they don't really seem to have a logical way to be used
  cleanly."
- "it's starting to look good, i think, we're making progress."

**His preset at 12:21** (`presets/compiler/default.toml`), what changed since 11:35:
- **Banks:** "steepest bank" went to 4 (max) and "widens up to" to 1 (min), so every creek's bank fades
  over a fixed 4 widths ("fade over" 4), with no widening. At "widens up to" 1, "steepest bank" does
  nothing.
- **Lakes:** pools must be 10 deep (it was 4), looked for up to 128 blocks round each lake (it was 64).
- **Size varies:** 0.5, the maximum.
- **Bends:** "a bend from" 0.17, and "cut banks keep" 0.05.
- **The line:** wander, lean and meanders have small values, but "most sideways" is effectively 0, so
  none of them act. The route stays "everywhere"; the deep cut is 2 near the sea and 8 high up, which
  only acts "where needed".

**Measured** (today's bake, 6 cores; against 11:35):
- deepest cut **115** (it was 439);
- steepest pull-down **15** (51);
- land lowered 4.2 on average (4.1), at most 121 (196);
- flush 95.3%, below the water 0.2%;
- rivers made in 1.2 s.

Still 24,207 points are cut more than 6 below the ground. About a fifth of those come from a stream
upstream through a junction; the rest from dips earlier on the river's own stretch (section 12).

### What the next step could be

1. **Cut the line's sideways moves** (David's proposal). That's wander, its swings, lean, meander
   every, meander size, full meanders below, most sideways and keep within: 8 sliders, and about 40
   lines of `curve`. Corner cutting stays.
   - *Builder's view: cut them.* Each moves the line off the way found, onto the ground beside it,
     which is higher (it's what "keep within" pulled back). With the land making way, any wander
     should come from the route on the real ground, not from noise.
2. **Cut the bend sides from the banks** (David's question). That's "a bend from", "outside fades
   over", the three ways by side (one way for all) and "cut banks keep": about 30 lines.
   - *Builder's view: cut them,* unless David wants cut banks back later. They were his one exception
     to flush, and a cleaner way to bring them back would be easing by how sharp the bend is, rather
     than switching at a threshold.
   - The odd block of an even width going to the outside of a bend stays. It's invisible and harmless.
3. **The water's level after a dip (the main lever left for the cuts).** A river can only fall, so
   after a hollow it stays at the hollow's level and cuts through every rise after it. Options, each
   behind a switch, with the readout counting land raised as well as lowered:
   - (a) **cut** (today);
   - (b) **fill**: a hollow shallower than a setting is filled, and the river runs level over it, on
     its bed raised a little, with the banks holding it;
   - (c) **least change**: each stretch at the level that cuts and fills the least land, rises cut and
     hollows filled by turns. It's one pass a reach (isotonic regression, "pool adjacent violators"),
     with no slider.
   - *Builder's view:* (c) as the default, with (b)'s depth as a cap on how high it may fill.
     About 60 to 80 lines, in `lay_water` where the levels are set.
4. **Sizes by area (stage 5)** after that, as planned.

### Questions for David

1. Cut the line's sideways moves entirely (wander through keep within)? Corner cutting stays.
2. Cut the bend sides from the banks: one bank way for all, and no "a bend from", "outside fades
   over" or "cut banks keep"? That drops the cut banks on the outsides of bends, the corner banks
   jutting up.
3. When a river dips into a hollow and the land rises again after it, should it cut through the rise
   (today), fill the hollow so it runs level over it, or do whichever changes the land least?
4. Then sizes by area (stage 5)?

## 14. The line and bends cut, least change, the anti-kitty-cornerer: built (2026-10-31)

David's go-ahead for section 13's 1 to 3 ("cut the bends. cut the sideways moves. i think filling and
cutting should be on a threshold probably... least change is kind of how i try to get everything and
put the intelligence into the rivers"), and "keep the rounding, add the anti-kitty-cornerer". The
details are in `docs/rivers.md`.

1. **The line's sideways moves are cut:** the wander, its swings, the lean, the meanders, "most
   sideways" and "keep within". About 40 lines and 8 sliders. Corner cutting (the rounding) stays.
2. **The bend sides are cut from the banks:** one way for every bank, and no "a bend from", "outside
   fades over" or "cut banks keep".
   - Worlds saved with the old `banks` line read back with their straight run's way.
   - **Presets load with a smooth way.** David's was slope, so he'll need to pick slope again once.
   - "Steepest bank" is greyed out when "widens up to" is 1.
3. **"The water's level" after a hollow: cut through, or least change.**
   - Least change is each stretch's falling level that's nearest its ground in all (least squares,
     one pass).
   - **The two thresholds:**
     - "fill hollows up to" (3): the most the water stands above its ground anywhere;
     - "cut rises up to" (40): before cutting deeper than this, the hollow before is filled more.
     - The fill's limit wins.
   - **A filled hollow:** the water is deeper there, and its banks are raised to the shore and ease
     down to the land over their fade. With fills of a few blocks it's a low bank along the river,
     not a causeway.
     - The bed isn't raised; a flat floor across the whole hollow would need the hollow's extent,
       like filling a lake with ground.
4. **The anti-kitty-cornerer** (on by default, kept with the world): the water's blocks join by a face,
   never only at a corner.
   - A river under about 1.4 blocks wide steps in a 1-block zigzag on a diagonal. That's for stage 5's
     1-wide springs; the narrowest river on David's world now is 2.5 wide, so none today.
   - A spot in any river's water is water, though another river's line is nearer.

**On David's 12:21 preset** (its banks set to slope by hand; 6 cores):

| | cut through | least change |
|---|---|---|
| Deepest cut | 115 | 112 |
| Points cut deeper than 6 | 24,207 | **18,127** |
| Water above its ground (points) | 705 | 9,803 (the hollows filled) |
| Land lowered, average (most) | 4.2 (121) | **3.4** (113) |
| Land raised, average | 0.03 | 0.07 |
| Flush | 96.2% | 96.4% |
| Rivers made in | 1.1 s | 1.1 s |

Notes:
- Least change cuts a quarter fewer points more than 6 deep, and lowers a fifth less land.
- The water stands "at most 65" above its ground in both modes. That's not the fill: a few reaches end
  at a lake's water standing well above their last point. It was there before; for later.

**Corner-only joins** (sampled at every 8th point): **176 before, 115 after.**
- **Fixed:** the 61 were at junctions. A narrow river joined a wide one, and the wide one's water
  stopped where the narrow one's line was nearer, leaving gaps.
- **Left:** the 115 are two separate rivers running side by side, their edges touching at corners on a
  diagonal (a 6-wide river right alongside a 14-wide one, say).
  - Their water could merge there, or keep a dry block between them.
  - **A question for David.** It's also odd that the plan sends two rivers side by side at all.

**Tests:** 49 pass. They include:
- least change filling a hollow and cutting the rise after it, within the fill limit and under the
  water arriving;
- the anti-kitty-cornerer on a 1-wide diagonal river (corner joins before, none after) and at a
  narrow-into-wide junction (the same);
- old saved banks reading back.

**Next:** stage 5, sizes by area, after David's word.

## 15. After the batch: David's notes, his 13:22 preset, and the next steps (2026-10-31)

**David, after trying it** (four screenshots):
- "no controls under the rivers tab except 'a lake from blocks deep' seem to do anything."
- **Least change, "fill hollows up to" 0:** at a sharp bend the water steps down against a sheer stone
  face.
- **Fill 20:** the river runs across a valley on a narrow embankment with steep stone sides, a
  causeway.
- **Fill 10:** "pretty close to good, on one side, not the other": the land falling away on one side
  leaves a steep embankment there.
- **His best preset, saved:** a tributary on a raised bank reaches a wide river, and its channel runs
  on into the middle of it: a long stone ridge with a ribbon of water on top.

**His 13:22 preset** (`presets/compiler/default.toml`). What changed:
- **Rivers section:** snowmelt only colder than -1 and above 128, so almost none; slope-foot springs
  0; "drawn from" 5,000.
- **River network:**
  - springs from Z 148, 2,000 apart, varying 0.18;
  - **mouths apart 0**: every mouth kept, 103 of them;
  - **a lake feeds a river from 400 patches**: in effect, no lake starts a river;
  - lone streams run 3,000;
  - **drop gathered into one step 18**, waterfalls from 25.
- **Carving:**
  - least change, fill up to 10, cut up to 40;
  - route everywhere (corridor 72, grain 4);
  - banks smooth, "steepest" 4, "widens up to" 1, compressor on.

**Measured** (today's bake, 6 cores):

| | as saved | step 0 (least change) | step 18, cut through |
|---|---|---|---|
| Points cut more than 6 deep | **53,745 of 81,193 (66%)** | 7,035 (9%) | 56,899 |
| Deepest cut | 90 | 87 | 113 |
| Water above its ground (points) | 1,666 | 8,921 | 624 |
| Land lowered, average (most) | **10.2** (112) | 2.8 (112) | 12.6 (97) |
| Land raised, average | 0.08 | 0.19 | 0.04 |
| Flush | 92.7% | 96.4% | 92.8% |
| Rivers made in | 3.0 s | 3.0 s | 2.9 s |

That's 274 stretches of river from 192 springs.

### What the screenshots and numbers say

1. **The step is now the biggest cutter.** The steps come after the levels are set. Each flat run
   takes the level of its lowest point, so everything else in the run is cut down to it, by up to 17
   blocks with a step of 18. That's two thirds of all points cut more than 6, and 10 blocks of land
   lowered on average. With no steps it's 9%, and 2.8.
   - **In keeping with least change:** each run could take the level that changes its land least
     (its middle, within the fill limit), rather than its lowest point. About 20 lines in the steps.
2. **Causeways and one-sided embankments come from the raised banks.** Where the water stands above
   the land (a filled hollow), the banks are raised to the shore at the water and drop back over the
   bank's fade. That's 4 widths with no widening, since "widens up to" is 1. So where the land falls
   away, the bank is a narrow, steep embankment: one side at fill 10, both sides (a causeway) at fill
   20.
   - **Options:**
     - raised banks ease back down at a gentle slope of their own ("raised banks no steeper than"),
       widening with their height, separate from the cuts' "widens up to" (so a fill becomes a low,
       broad floodplain bank);
     - and/or only fill a hollow that's a valley, with land rising on both sides, never a strip of
       water standing over a slope.
3. **The ridge into the wide river is a junction bug, older than today, and plainer with fills.**
   - A tributary's last stretch runs on to the junction at the main river's middle. Inside the main
     river's water it keeps its own shallower floor and its own water, higher than the main river's.
   - So a stone ridge runs out into the river with a ribbon of water on top.
   - **The fix:** where rivers' waters overlap, the lowest water and the deepest floor win. The
     tributary then ends at the main river's shore, with its drop there. It's small, in the carve.
4. **The sheer face at a bend (fill 0)** is a step falling at a sharp bend. The curtain hangs along
   the lower piece, so beside it the upper water's face shows. It needs a closer look at such a spot,
   with a probe.
5. **The Rivers section does almost nothing, as David found.**
   - Snowmelt and slope-foot springs only add to each spring's own water. That decides only whether a
     spot can hold a spring ("water it needs" 1 lets any land) and how far a lone stream runs, which
     already reaches other water.
   - Sizes come from joins, not water, so they don't use it either.
   - "Drawn from" is the picture's only. "A lake from" is the one that acts.
   - **Proposal:** retire snowmelt and slope-foot springs (or park them until dry beds or sizes want
     the water), keep "a lake from", and label "drawn from" as the picture's.

### Proposed next steps, in order

1. **The ridge fix:** where waters overlap, the lowest water and deepest floor win. Small, and clearly
   a bug.
2. **Steps by least change:** each flat run at the level that changes its land least, within the fill
   limit.
3. **Gentler raised banks:** a slope of their own, widening with their height.
4. **Retire the Rivers section's inert controls.**
5. **Stage 5:** sizes by area.

### Questions for David

1. "Drop gathered into one step" at 18: was that for bigger, fewer waterfalls? It's what cuts most
   now. Shall steps take the least-change level for each run?
2. Raised banks: a gentle slope of their own (a low, broad bank where a hollow's filled)? And fill
   only hollows with land rising on both sides?
3. Retire snowmelt and slope-foot springs?
4. (Still open) Two rivers side by side, touching at corners: merge their water there, or keep a dry
   block between them?

## 16. The ridge, least-change steps, gentler raised banks, the Rivers section: built (2026-10-31)

David's answers to section 15: "1. yes. rapids of lots of small falls are nice, but bigger ones are
also nice. 2 yes 3. yes, cut them. 4, it should cause a waterfall to merge, shouldnt it? ... we can
proceed to the next step". The details are in `docs/rivers.md`.

1. **The ridge fix.** Where rivers' waters overlap, the lowest water and then the deepest floor win, so
   a tributary ends at the shore of the river it joins, with its drop there.
   - Within one stretch, the nearest piece still decides. The pieces either side of a step or fall
     overlap, and the lower water winning there would move each fall half a width upstream.
   - "A spot in any river's water is water" now holds with the anti-kitty-cornerer off too.
2. **Steps by least change.** With least change, each flat run takes its points' mean level (rounded
   down), within their ceilings, instead of its lowest point's.
   - **A mix of falls (David):** the dials already give one. Drops from "a waterfall from" up are
     falls. Smaller ones are rapids where their run has room, and small curtained falls where it
     hasn't. So a step under the fall height gives rapids and little falls on gentle ground and big
     falls on steep ground. Nothing new added.
3. **Gentler raised banks:**
   - **Their own slope:** "raised banks no steeper than" (0.5) and "raised banks widen up to" (3),
     separate from the cuts'.
   - **Fills only between banks:** the water rises no higher than the lower bank, 3 blocks out either
     side, plus "…and above the lower bank up to" (0 to start). I added that slider because at 0
     rivers along a slope can't fill, so more is cut.
   - "Steepest bank" is greyed out at "widens up to" 1.
4. **The Rivers section:** snowmelt and slope-foot springs are retired (Climate passes them as 0; their
   code stays). "Drawn from" is labelled the picture's. "A lake from" stays.

**On David's 13:22 preset** (6 cores):

| | Points cut more than 6 | Water above its ground (points) | Land lowered, average | Flush |
|---|---|---|---|---|
| Before (this morning's build) | 53,745 | 1,666 | 10.2 | 92.7% |
| Step 18, 0 above the bank | 45,690 | 626 | 9.6 | 95.4% |
| Step 18, 3 above the bank | **36,646** | 8,987 | **6.7** | 96.6% |
| Step 0, 0 above the bank | 15,961 | 552 | 4.6 | 96.2% |
| Step 0, 3 above the bank | **12,477** | 6,792 | **3.7** | 96.5% |

The rows have about 80,000 points each. Rivers are made in about 3.5 s.

**The sheer face at a sharp bend (David's fill-0 screenshot).**
- **What the probe did:** it drew the water round the three sharpest bends with a fall on his world
  (each turning more than a radius of its width).
- **What it found:** the water steps cleanly from one level to the next round each, with no stone face
  between.
- **So the face in the screenshot was most likely a junction:** a higher stream's own floor and water
  running into a lower river (its upper water curling round, a stone face where it ran into the
  other's water). The ridge fix is that. To confirm in the game.

### Two rivers side by side, merging with a fall (proposed, not built)

David: "it should cause a waterfall to merge". Two ways:

- **In the carve:** fill the gap between two rivers' waters where they nearly touch, with the lower
  one's water.
  - It's small, but along a long side-by-side stretch it makes one long wall of water: a weir the
    whole way, not one fall.
- **An early join in the plan** (better). After the levels are laid, walk each river down. At the first
  point where its water comes within about a block of another river's water that's lower, which isn't
  its own junction partner, it ends there: its last point at that river's edge, its level dropping to
  that river's, a fall with its curtain. The rest of its path alongside is dropped.
  - It's the "join" David described: one fall where they first touch, then one river.
  - **Cost:** a lookup of every river's points by cell (like the ponds' own), the walk, cutting the
    river short, and the fall. About 80 to 100 lines with its test, so it's reported, not built.
  - It fits with the joint fade below, and with David's later idea of water drawn toward nearby water.

### David's joint fade (an idea, assessed)

David: "regarding the jointing issues, perhaps a linear joint fade similar to how we do the lateral
bank fades."

- **What it is:** over a tributary's last stretch, its water (and floor) eases down toward the level of
  the river it joins, over a length (say a few of the bigger river's widths). Whatever the ease can't
  take becomes a fall at the shore. That's how most tributaries meet: at the main river's level, unless
  they're a hanging valley with a fall.
- **With the ridge fix:** they don't overlap. The ridge fix decides inside the main river's water; the
  joint fade shapes the tributary's last stretch outside it, so the drop at the shore is small or
  nothing.
- **With the side-by-side merge:** the early join makes a new junction, and the joint fade eases the
  river into it, leaving a fall for any rest.
- **Cost:** about 30 lines in `lay_water`, after the steps and rapids, where a reach's end is lowered
  to the water it joins. It's a ramp down over its last stretch, only ever lowering (cutting its bed a
  little deeper), plus a slider ("joins ease over (widths)").
- **Where it belongs:** it's a fade along the river, so it fits stage 2. But it's small, and it stands
  on its own, so it could come before or with stage 5.

### Questions for David

1. **The two-rivers merge:** build it as an early join (one fall where they first touch)? About 80 to
   100 lines.
2. **The joint fade:** build it now (about 30 lines and a slider), or with stage 2?
3. **"…and above the lower bank up to":** try 3 to start? It halves the cuts against 0, with a few
   blocks of gentle raised bank.

## 17. The merge, the joint fade, a lake's rim: built (2026-10-31)

David's answers to section 16, through Counsel and then directly: build the merge as an early join,
build the joint fade now, and start "…and above the lower bank up to" at 3. "Go ahead and launch it, do
those things before the next step, after i close the game." The details are in `docs/rivers.md`.

1. **Rivers alongside each other merge** ("rivers alongside each other merge", on). After the water's
   laid, each river is walked down, those nearest the sea first. At its first point whose water comes
   within a block of another river's lower water, it ends there. Its level falls to the other's, with a
   curtain if the drop is 2 or more, and the rest of its way alongside is dropped. It only looks clear
   of its own ends, where it meets its junctions' rivers anyway.
2. **Joins ease down** ("joins ease over", 3 of the joined river's widths). A river's last stretch is
   ramped down toward the water it joins, a block for every "rapids" blocks. What's left is a fall at
   the shore. It only ever lowers.
3. **"…and above the lower bank up to" starts at 3.**
4. **A lake's rim beside a river** (David's screenshots at "a lake from" 24 and 40: "some odd behavior
   there"). A pool beside a river stood as a slab of water with a sheer side.
   - **Why:** a lake's water and its rim were judged on the uncarved ground. A river's bank had cut the
     pool's rim below the pool's water, so the water stood against air.
   - **Now:** both are judged on the ground as the bank carved it. A bank cut inside the lake's water
     fills with the lake, and one cut beside it is held up to one block under the surface.

**On David's 14:07 preset** (6 cores; a lake from 36, steps of 8, falls from 18, fills 10 and 10 above
the bank, compressor on, widest 1, steepest 4):

| | Merged | Points (all) | Points cut more than 6 | Water above its ground (points) | Deepest cut | Land lowered, average | Flush | Corner-only joins (sample) |
|---|---|---|---|---|---|---|---|---|
| No merge, no joint fade | 0 | 82,938 | 12,880 | 14,527 | 112 | 4.01 | 96.4% | 11 |
| Merge, joints ease over 3 | 21 | 81,688 | 12,931 | 14,218 | 112 | 4.06 | 96.5% | 11 |

Rivers are made in about 3.4 s.

- **The merge:** 21 rivers joined early, and 1,250 points of river alongside others were dropped. The
  corner-only joins in this sample were already 11 without the merge (115 on an earlier preset, before
  the ridge fix), and stay 11 with it, so the sample doesn't show the merge's effect. The game is
  where to look.
- **The joint fade** barely moves the totals (51 more points cut deep, 0.05 more land lowered on
  average). It changes the last few widths of each join, not the whole river.
- **The deepest cut, 112,** is unchanged. It's not at the joins. David's 14:01 screenshot shows the
  kind: a river falling into a deep, narrow gorge with sheer walls, where the route goes through a ridge
  and banks at "widens up to" 1 can't slope its sides. Sizes by area (stage 5) and the cuts' banks are
  where that's addressed. Noted, not acted on.

### Next

Stage 5, sizes by area, as planned, unless David's look at this build turns something up first.

## 18. Stage 5, sizes by area: built as an experiment (2026-10-31)

David's answers, through Counsel: "1. they can. but on my recent check, they looked ok in general. id
give it a try, if it was a setting, but its not broken now. 1 block may be better or worse. 2. thats a
good starting point. but sizes are not a problem now so we are only checking this. make sure it can be
disable with a checkbox. the names dont matter, since we are going by area. 6. agree. the rest, i dont
know." Counsel took the rest: the steep end at 1:2 (deeper than wide), two shapes mixed by the slope,
and one bank fade for all water. The details are in `docs/rivers.md` (step 7, "Sizes by area").

**What's built:**
- **The "sizes by area" box**, off by default, so David's preset keeps today's look. Unticked,
  nothing changes.
- **Ticked:**
  - Each river patch's water across is gathered upstream first: a spring's (1); where waters join, 90%
    of both; a block more every 2,000 blocks run.
  - **A lake passes its rivers' water on** to its outlet's river, as a junction would. This wasn't
    asked; without it, a river leaving a lake would start again as a 1-block spring.
  - The slope lays the water out: 16 blocks wide for each block deep where the broad ground falls 0.2
    blocks in 100 or less, and 0.5 (twice as deep as wide) where it falls 5 or more, evenly by ratio
    between. The shape across is mixed between a flat-ground and a steep-ground shape the same way.
  - The banks fade over one setting, in widths, for all water.
- **The ranks' sliders and the network's "stream orders per rank"** are greyed out when it's ticked.
- **A readout** of the spread of sizes (ticked or not), "sizes in the close-up" (rivers coloured by
  their water across), and the preview drawn by area.

**David's world, slopes** (his 14:33 preset; 9,792 river patches of 32 blocks): the fall per 100 blocks
over 4 patches down is

| Share of patches at or under | 10% | 25% | 50% | 75% | 90% | 99% |
|---|---|---|---|---|---|---|
| Fall per 100 blocks | 0.00 | 0.01 | 0.47 | 1.9 | 7.2 | 40 |

A quarter are flat (lakes' surfaces and plains). The defaults of 0.2 and 5 put about 40% of the patches
at their broadest and about 13% at their deepest.

**On David's 14:33 preset** (6 cores), along the rivers' length:

| | Unticked (ranks) | Ticked, grows every 2,000 | Ticked, grows every 250 |
|---|---|---|---|
| Water across: 10% / median / 90% / most | 7 / 22 / 114 / 336 | 1.1 / 1.8 / 5.6 / 25 | 1.8 / 7 / 29 / 139 |
| Width: 10% / median / 90% / widest | 4.5 / 9.7 / 31 / 57 | 1.0 / 1.6 / 5.0 / 18 | 1.3 / 4.6 / 21 / 54 |
| Depth: 10% / median / 90% / deepest | 2.0 / 2.9 / 4.7 / 7.0 | 1.0 / 1.1 / 1.8 / 5.6 | 1.0 / 1.5 / 3.6 / 13 |
| Length under 3 blocks of water / 3-16 / more | 0% / 40% / 60% | 74% / 26% / 0% | 21% / 55% / 24% |
| Points cut deeper than 6 | 17.6% | 14.0% | 15.3% |
| Flush shores | 95.7% | 92.3% | 91.5% |
| Land lowered, average | 4.3 | 3.7 | 3.7 |

**What it shows:**
- **At the agreed start, rivers by area are much smaller than today's.** Three quarters of their
  length is spring-sized, and the most water anywhere is 25 blocks, against 336.
- **Why: David's network rarely joins.** Branching is 0 (one trunk to each mouth), so a river gathers
  few springs. The water gathered per river patch is at most 15 at the agreed start, and at that rate
  of growth a long river only gains a few blocks.
- **Faster growth brings sizes back:** at a block every 250 blocks, the widest is 54 (57 today), with
  a quarter of the length river-sized, and narrow, deep water on steep ground (up to 13 deep).
- **Fewer flush shores** (92% against 96%): more of the water is narrow, and a narrow river's shores
  meet more steep ground.
- **Points cut deeper than 6 are a smaller share.** Narrow rivers have more points per block, so the
  share is what compares, not the count.

### Questions for David

1. Look at it ticked, with "sizes in the close-up" on. Is spring-sized water over most of the length
   (at the agreed start) what he's after, or should growth by running be faster (250 gives sizes near
   today's)?
2. Would more joining suit "sizes by area" better? "How readily rivers split" (branching, now 0)
   gives rivers more springs, and so more water from joins. That's a network setting, for him to try.

## 19. Rivers settled for now (2026-10-31)

David, after playing stage 5: "at first i was bothered, but looking at that gorge more closely, its
kind of neat and a really big lake way high up, so it wasn't bad the more i thought about it. i'd say
we've pretty much got rivers fixed. any improvements would have to come as part of a bigger rework,
where i have a really specific goal in mind; right now, rivers are about as good as anything else in
the game. the rough edges are minor enough that i think they can be smoothed out later on without a
huge fuss."

- **Where it stands:** stages 0, 1 and 4, then the batches of sections 14 to 17, and stage 5 (sizes by
  area, its box off by default in the code). David's preset (15:20) has it ticked: a spring's water 3,
  joins keeping all, a block more every 50 blocks run, steep ground 1:1.
- **Not built, kept on file:**
  - stage 2 (the fade along the river);
  - stage 3 (lakes picked first);
  - stages 6 to 8 (petering out, rivers first in the Climate tab, dry beds);
  - waterways drawn toward nearby water, and cutting rivers seeking the nearest pond (section 15);
  - the river-to-river join rule (shelved).
- **Rough edges:** to smooth out later, from David's screenshots, without a rework. The known ones:
  - deep cuts (about 18% of points cut more than 6 deep; the deepest about 170);
  - a few corner-only joins;
  - shores about 95% flush.
- **Further changes** come with a bigger rework, when David has a specific goal for one.

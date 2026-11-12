# Water, from the start  [DONE ENOUGH TO GO ON, 2026-10-31; regrouped by David and Counsel the same day]

David: "rivers arent 'done', nothing is done, everything is only done enough to do the next thing,
until we go back and fine tune everything again. but i think we did good work today."

**Enough for now** (David, 2026-10-31, evening): "at first i was bothered, but looking at that gorge more
closely, its kind of neat and a really big lake way high up, so it wasn't bad the more i thought about
it. i'd say we've pretty much got rivers fixed. any improvements would have to come as part of a bigger
rework, where i have a really specific goal in mind; right now, rivers are about as good as anything
else in the game. the rough edges are minor enough that i think they can be smoothed out later on
without a huge fuss." Builder's summary is `docs/water-rework.md` section 19.
- **Built:**
  - stages 0, 1, 4 and 5 (sizes by area as a box, ticked in David's 15:20 preset);
  - the batches in sections 14–17.
- **On file, not built:**
  - stage 2 (the fade along the river);
  - stage 3 (lakes picked first);
  - stages 6–8 (petering out, rivers first in the Climate tab, dry beds);
  - waterways drawn toward nearby water, and cutting rivers seeking a pond;
  - the shelved river-to-river join rule.
- **Rough edges for later,** from David's screenshots:
  - deep cuts;
  - a few corner-only joins;
  - shores about 95% flush.

The rest of this page is the record of how it got here.

David, 2026-10-31: "im wondering now if we need to disconnect some components and get back to basics and
redesign the logic between rivers and interpretation. rethink how we get from A to B, and implement the
parts that work more carefully, one at a time. i kind of lost control somewhere between the first
theoretical rivers and trying to get them carving sane paths in a pleasant way." Then: "we need to
regroup from the start with water."

New river features are paused. The last state, d816b60, stays as the fallback. The record of how
today's rivers work is `docs/rivers.md`, and the decisions behind them are in `docs/worldgen.md`
("Breaking the water loop", "The river rework", "Carving").

**How this page works:** first what water should be (section 1, in David's words), then the one big
choice (section 2), then what's built and what survives (section 3, from Builder's inventory). Only
then the design, one stage at a time (section 4). No method before the goals.

## Where we stand (2026-10-31)

**Settled by David:**
- springs are 1 block; "creek" is only a colloquial word;
- size is area (water blocks in the cross-section), one continuous number, with names at thresholds,
  not types;
- joining makes a stream a river (growing is the exception); growth is fast on purpose; joining adds
  most of both areas; running adds slowly;
- the slope splits area into shape: steep is deep and narrow, flat is broad and shallow, from spring to
  delta;
- pseudoscience on purpose: it only has to pass "yeah, that could be real";
- the rules that stand: the linear pipeline, plain water blocks, no rain in rivers, the world made
  before you enter.

- **Springs: keep the ways we have** ("we have had good ways of starting springs already i think"):
  plain or relative height, with spacing and jitter. Relative height still overpopulates in places,
  which is tuning, not a redesign.
- **Rivers come first in the Climate tab** (decided, recorded in `docs/worldgen.md`): Rivers,
  Temperature, Wind, Rain, Mineralogy, Caves. Snowmelt then goes by height.
- **River sizes should be tunable** through the area logic.
- **Petering out only where water has nothing else to do** (below).
- **Lakes and ponds chosen on purpose,** if it can be done (below): "i really dont want many but
  theyre kind of obligatory."

**Ideas on the table, not decided:** dry beds (a later step takes water out of small beds by heat and
dryness); the webbing.

**David's answers to Builder's questions (2026-10-31):**
1. **A 1-block ledge isn't flush:** "1 block ledge you can't get out of the water from." Flush means
   level with the water, so you can walk out.
2. **Flush is normal, not a target share:** shores are just normally flush, with **some corner banks
   jutting up** (the outside of bends) and **lakes and ponds not always all flush.**
3. **X (how deep a cut gets before a river looks for a way round):** "a sane range to tune with."
4. **Gorges up high, yes:** "they're better up high. its better for the rivers to meander around the
   terrain down low near sea level."
5. **Lakes by a count, chosen as a range,** like the mountain peaks: "a count threshold, like there is
   for the mountain peaks perhaps, where i can select a range from top to bottom." *Counsel:* the
   basins ranked (by size or depth), with a least-to-most range as the peaks have for crush, and the
   count shown.
6. **Area is the simple metric for size,** "slightly shifted by being broader or deeper depending on
   rise-over-run."
7. **Crags beside water are lowered** by the compressor (decided 2026-10-31), not smoothed away.
8. **No great river** [David 2026-10-31]: "great rivers aren't really a 'thing', as we have no discrete
   types, we should just make sure that its rare two whole rivers join up." The names stop at river.
   What's wanted is a behaviour: **two whole rivers joining is rare.**

**Stage 0 built** (f71e8be, 2026-10-31): the rapids bug fixed ("rapids at 1" not needed); a Shores
readout and a "shores in the close-up" overlay on the Carving panel (green flush, amber ledge, red
bank, magenta below the water). Baseline on David's world:
- **97% of shores are ledges** with the default, because both presets set every bank way to "ledge
  then slope";
- **with smooth banks, rivers-balanced is 94.5% flush,** at 0.4 of a block more land lowered on
  average. So "flush by default" is a settings change today;
- **what's harsh is the cuts through rises:** up to 81 deep, and 6,000–10,000 points cut more than 6.
  That's stages 1, 2 and 4.
**Rare river-to-river joins** (Builder, `docs/water-rework.md` section 9, not built): "a tributary
carries at most" a share of the main branch's springs where they meet (about a third). Springs past it
run as streams, so rivers mostly take in streams and the main stem gathers more. About 30 lines, one
slider, a readout of joins; fits with stage 5. **Shelved** (David, 2026-10-31): "i don't think we need
to try and fix problems before we even see them." Kept on file in case whole rivers do join too often
once the rework is in.

**Stage 1 built** (fa8c14c, 2026-10-31, David's go-ahead given to Builder directly): compressor banks
behind a "compressor banks" switch, off by default.
- Every nearby stretch has its say, and the lowest bank wins, so there are no seams between two
  rivers' banks (they had made the steepest pull-downs).
- The first dry block is the shore.
- Cut banks on the outside of bends keep a share of the land ("cut banks keep", 0.25).
On David's world (rivers-balanced, smooth banks), off against on:
- flush 93.6% → 92.6%;
- ledges 1.7% → 0.4%;
- banks 2.7% → 6.6% (mostly the cut banks, on purpose);
- below the water 2.0% → 0.4%;
- the carve near a river 0.6 → 1.0 µs a block.
Left for later: shores below the water beside some waterfalls; tall levees where a pond's water backs
up a reach (a water-level issue); stage 2 (attack and release).

**David's preset after stage 1** (Builder, `docs/water-rework.md` section 11, 2026-10-31). David's best
settings so far are saved as `presets/compiler/default.toml`: "many settings on the climate page i've put all
the way down, because they do nothing, or are harmful."
- **Every river is a creek.** "Stream orders per rank" is at 4, the highest order on the world, so no
  river reaches stream rank. Only the creek's size, shape and fade act. The other ranks' sliders do
  nothing at these values.
- **Inert because of another setting:**
  - wander, swings and lean, and keep within (nothing moves sideways);
  - cut depth and pond reach (ponds behind rises are off);
  - the delta settings (deltas are off);
  - rise and around (plain height);
  - terrace steps (no terraces);
  - snowmelt ("higher than" is 1,020).
- **David's choices that shape the look:**
  - springs from sea level up (573 of them);
  - one trunk per mouth;
  - lone streams run far (only 9 fizzle);
  - no steps, and falls from 25;
  - a narrow, coarse corridor;
  - springs 4 wide;
  - banks all slope, the gentlest and widest, with the compressor on and cut banks flush;
  - ponds behind rises off, and pools from 4 deep.
- **The curtain fix** (cc9e868), from David's screenshot of a wall of water about 10 tall. With no
  steps the water dropped a little almost everywhere, and 4,895 drops stood sheer, up to 24 tall. Now
  every drop of 2 or more gets a curtain, and none stand sheer.
- **Measured:**
  - shores 95.7% flush, 3.6% ledge, 0.5% bank, 0.1% below the water;
  - 21 lakes;
  - widths 3 to 54;
  - land lowered 4.1 on average (196 at most).
- **The harm now is the cuts:** up to 439 deep, and 25,098 points cut more than 6. Rivers saw through
  whatever is in the way (no ponds, and little room to get round), and the gentle banks spread each cut
  wide.

*Counsel's advice to David:*
1. Stage 4 next: the local re-route where a cut would be deeper than X. It already carries David's
   answers 3 and 4, and it replaces the corridor.
2. Grey out the inert sliders alongside it.
3. Stage 5 (sizes by area) after that, since it brings the size sliders back.
4. Decide on stage 2 after seeing stage 4: it smooths whatever cuts are left.
5. Waterfalls and steps are a look by eye, not a build.

**Stage 4 go-ahead** (David, 2026-10-31: "got it, let builder know to proceed"). The local re-route,
with X near the sea and high up, and the inert sliders greyed out alongside. Relayed to Builder.

**Stage 4 built** (bbc2b76, 2026-10-31; `docs/water-rework.md` section 12).
- **What's on the panel:**
  - a "found:" switch: everywhere (the default, so David's preset looks the same), only where needed, or off;
  - the deep cut "near the sea" and "high up" (2–40, both 8);
  - a readout;
  - inert sliders greyed out.
- **On David's preset,** everywhere against only where needed:
  - the plans' deepest cut 117 → 90;
  - the steepest pull-down 51 → 30;
  - land lowered 4.1 → 3.6 on average;
  - flush 95.7% → 96.0%;
  - 0.3 → 1.6 s.
- **The cuts as laid barely move** (439 → 435; 25,098 → 23,825 points deeper than 6), and the reason
  is the level, not the path.
  - The water's level only falls along a river. With ponds off, after any hollow the river stays at
    the hollow's level and trenches every rise after it, until the ground drops lower.
  - About 80% of these come from hollows earlier on the same stretch, and 20% through a junction.
  - On David's world the hollows are mostly the hills.
- **The lever is the level rule:** what a river does when it crosses a hollow. Today it always cuts.
  - **Fill small dips:** the water runs level over a dip shallower than a setting, with the bed
    raised under it. Cheap.
  - **The level that changes the land least:** cut the rises and fill the dips by turns, in one pass
    along each reach (isotonic regression). The fuller answer.
  - Either way, filling raises land, which the rivers have so far never done.

*Counsel's advice:*
- Try "only where needed" in the game.
- Take the least-change level next, with a "deepest fill" limit. Hollows deeper than the limit are for
  stage 3 to make lakes or not.
- Builder should say first how a filled hollow looks beside the river: a flat valley floor raised to
  the water (as sediment fills basins), not a causeway with levees.

**After stage 4** (`docs/water-rework.md` section 13, 59be2eb). David tried it.
- **David's notes:**
  - "everywhere setting on the route works pretty good. it doesn't take any time";
  - wander through keep within "may be useless entirely... it can be cut";
  - "im questioning the value of bends... they don't really seem to have a logical way to be used
    cleanly";
  - "it's starting to look good, i think, we're making progress."
- **His 12:21 preset:**
  - the deepest cut 439 → 115;
  - the steepest pull-down 51 → 15;
  - land lowered at most 196 → 121;
  - flush 95.3%;
  - still 24,207 points cut more than 6.
- **Builder's four questions,** with Counsel's advice:
  1. **Cut the line's sideways moves** (8 sliders; corner cutting stays)? *Yes.* It's David's own
     call. Wandering down low now comes from the route (deep cut 2 near the sea). If plains rivers
     later look too straight, a wiggle that follows the land can come back then.
  2. **Cut the bend sides** (one bank way, no cut banks)? *Yes.* It drops the jutting corner banks
     of answer 2, but David's preset has them at 0.05, nearly flush already. The way back, easing by
     how sharp the bend is, stays on file. Stage 2 then becomes attack and release only.
  3. **After a hollow?** *Least change, capped by the fill depth.* A filled hollow should be a flat
     valley floor raised to the water, not a causeway with banks holding the river up. Builder should
     confirm which it makes. Hollows deeper than the cap stay cut, for stage 3 to make lakes.
  4. **Then sizes by area?** *Yes.* Stage 2 is decided after the level rule; at a steepest pull-down
     of 15, it may not be needed.
- Also: "steepest bank" does nothing when "widens up to" is 1, so it could be greyed out too.
- **David's answers** (2026-10-31): "i think the corner cutting just needs to be a 1 block zigzag
  instead, corner cutting doesnt really work in minecraft physics. cut the bends. cut the sideways
  moves. i think filling and cutting should be on a threshold probably. so yes, sounds good. least
  change is kind of how i try to get everything and put the intelligence into the rivers."
  - Relayed to Builder as a go-ahead:
    - cut the sideways moves;
    - cut the bend sides;
    - least change, with thresholds on filling and cutting (Builder to propose how);
    - stop for David, then sizes by area.
  - **Corner cutting: held.** The slider is the line's smoothing (Chaikin's method), which rounds the
    route's 16-block steps. David's point is about water blocks that touch only at their corners. That
    calls for a rule that water always joins by a face, a 1-block zigzag on diagonals, which matters
    most once springs are 1 wide. Builder was asked whether it happens today; David to confirm.
  - **Decided** (David, 2026-10-31): "keep the rounding, add the anti-kitty-cornerer." The rounding
    stays as it is. The **anti-kitty-cornerer** (David's name) makes water blocks always join by a face:
    a 1-block zigzag on diagonals. It goes in with the current batch. Relayed to Builder.

**The batch built** (8a32441, 2026-10-31; `docs/water-rework.md` section 14).
- **What changed:**
  - the sideways moves are cut;
  - the bend sides are cut. One bank way for all; presets load with "smooth", so David picks slope
    once;
  - the level after a hollow: "cut through" (the default) or "least change". Least change has two
    thresholds, "fill hollows up to" (3) and "cut rises up to" (40), and the fill limit wins;
  - the anti-kitty-cornerer, on by default. It acts on rivers under about 1.4 wide (none yet), and
    fixed 61 gaps at junctions.
- **On David's preset,** cut through against least change:
  - points cut more than 6: 24,207 → 18,127;
  - deepest cut: 115 → 112;
  - land lowered: 4.2 → 3.4 on average;
  - land raised: 0.03 → 0.07;
  - flush: 96.2% → 96.4%;
  - both 1.1 s.
- **A filled hollow isn't a flat floor.** The water is deeper there, and the banks are raised to the
  shore and ease back to the land: a low bank along the river, a few blocks at most. A flat floor
  across the whole hollow needs the hollow's extent, which stage 3's basin list has.
- **Left for later:** a few reaches end at a lake standing up to 65 above their last point. It's in
  both modes, so it was there before.
- **Builder's question:** 115 sampled places where two separate rivers run side by side, touching at
  corners on a diagonal. Merge their water, or keep a dry block between them?
  - *Counsel:* keep a dry block. Two rivers have two levels, and merging them puts water of
    different heights side by side.
  - Two rivers running side by side is a quirk of the plan. If it looks wrong in the game, the fix
    belongs in the plan (join them sooner), not now.

**After the batch** (`docs/water-rework.md` section 15, 8025dde). David tried it, with four
screenshots.
- **David's notes:**
  - "no controls under the rivers tab except 'a lake from blocks deep' seem to do anything";
  - least change at fill 0 left a sheer face at a sharp bend;
  - fill 20 made a causeway;
  - fill 10 was "pretty close to good, on one side, not the other";
  - a tributary ran on into the middle of a wide river as a stone ridge with water on top.
- **Builder's findings:**
  1. **The step is now the biggest cutter.** At step 18, each flat run takes its lowest point's
     level, so the rest of the run is cut down to it. That's 66% of points cut more than 6; with no
     step it's 9%.
  2. **Causeways and one-sided embankments come from raised banks** that fall back over a narrow fade
     ("widens up to" 1).
  3. **The ridge is an old junction bug:** the tributary keeps its own floor and water inside the main
     river.
  4. **The sheer face at a bend** needs a probe.
  5. **The Rivers section's snowmelt and slope-foot springs feed nothing** that matters.
- **Builder's order:**
  1. the ridge fix;
  2. steps by least change;
  3. gentler raised banks;
  4. retire the Rivers section's inert controls;
  5. sizes by area.
- *Counsel's advice:*
  1. **Steps by least change: yes.** Build them with the gentler raised banks and show them together:
     a run at its middle level has its upper half cut and its lower half held up, a pool behind a lip.
     That raised half needs the gentle banks to look right. The step itself is David's dial (18 cuts
     up to 17).
  2. **Raised banks: yes to both.** A gentle slope of their own, widening with height, and fills only
     where land rises on both sides across the river. Water can only stand above the ground where land
     holds it in. Where one side falls away, the water would run that way.
  3. **Retire snowmelt and slope-foot springs: yes.** Sizes will come from joins (stage 5), petering
     from distance (stage 6), dry beds from heat and dryness (stage 8). None reads a spring's water.
     Keep the code on file.
  4. **Side by side: a dry block between.**
  - Also probe the sheer face at the bend, since it's been seen. And David's banks are on "smooth"
    since the bend cut, if slope is wanted back.
- **David's answers** (2026-10-31): "1. yes. rapids of lots of small falls are nice, but bigger ones
  are also nice. 2 yes 3. yes, cut them. 4, it should cause a waterfall to merge, shouldnt it? ... we
  can proceed to the next step." Relayed to Builder as a go-ahead:
  - the ridge fix;
  - steps by least change, together with the gentler raised banks (a slope of their own; fills only
    where land rises on both sides);
  - retire snowmelt and slope-foot springs;
  - **rivers side by side merge with a waterfall,** the higher falling into the lower (not a dry
    block, as Counsel had advised); Builder to propose how;
  - the probe of the sheer face;
  - then stop before stage 5.
  - **Falls:** David likes both rapids made of lots of small falls and bigger falls. That's tuning on
    the step dial for now.

**Waterways drawn to each other** [IDEA, David 2026-10-31, later]: "later i may try to find a way to
not make waterways ignore nearby waterways, and be attracted to them.... if theyre cutting anyways they
should head for the nearest pond or lake if one is available, i think that would look more natural and
less cluttered."
- *Counsel:* it fits the plan stage (step 6), and stays linear.
  - Water already knows its distance to the next water, from the petering-out idea.
  - A river's way could be weighted toward nearby water.
  - With lakes picked first (stage 3), a river cutting anyway could be routed to a picked lake within
    reach.
  - It would also cure two rivers running side by side.
- More joins aren't a worry (David, 2026-10-31): "the rivers arent that big that im seeing, so them
  joing wouldnt be an issue unless there resulted too few. the pull is very tuneable." So the pull acts
  on all water, with its strength as the dial. What to watch is too few rivers left.
- **Only at the tip, and tied to cutting** (David, 2026-10-31): "i suppose it would only work on the tip
  of water as it travelled and if it found an attractor, but im interested i seeing if we can always
  correlate cutting with attraction towards nearer water. cutting isnt always a bad look but having it
  seek a pond would make it more logical."
  - *Counsel:*
    - The trigger already exists. Stage 4 finds where a river would cut deeper than X; that's where it
      would look for nearer water within reach and lower than the cut.
    - The walk at the tip exists too: lone streams already walk downhill to the next water.
    - **It belongs in the plan (step 6), not the carve.** Ending a river in a pond changes which water
      joins which. Sizes by area (stage 5) count joins, so the joins must be settled before the sizes.
      Step 6 can read the real ground for the estimated cut (stage 4 already estimates the plans'
      cuts). That keeps it linear.

**The next batch built** (b3c005f, 2026-10-31; `docs/water-rework.md` section 16).
- **The ridge fix:** where waters overlap, the lowest water and then the deepest floor win, so a
  tributary ends at the shore with its drop.
- **Steps by least change:** each flat run takes its points' mean, rounded down.
  - David's mix of falls needs no new dial: a step below "a waterfall from" gives rapids and little
    falls on gentle ground and big falls on steep ground.
- **Gentler raised banks:** "raised banks no steeper than" 0.5, "widen up to" 3, separate from the
  cuts'.
  - Fills go only up to the lower bank, plus "…and above the lower bank up to" (0 to start).
- **The Rivers section:** snowmelt and slope-foot springs retired, with the code kept.
- **The probe:** the three sharpest bends with falls step cleanly. The stone face was most likely a
  junction, which the ridge fix covers. To confirm in the game.
- **On the 13:22 preset,** points cut more than 6 (of about 80,000):

  | | 0 above the bank | 3 above the bank |
  |---|---|---|
  | Step 18 | 45,690 | 36,646 |
  | Step 0 | 15,961 | 12,477 |

  - This morning was 53,745.
  - At step 18 with 3 above the bank, land lowered is 6.7 on average (it was 10.2), and shores are
    96.6% flush.
- **Not built, for David:**
  - **the side-by-side merge** as an early join: one fall where they first touch, and the rest of
    the path alongside dropped (80–100 lines);
  - **the joint fade** (about 30 lines and a slider);
  - **"above the lower bank" at 3.**
- *Counsel's advice:*
  1. **The early join: yes.** One flaw for stage 5: it's decided after the levels, so the lower
     river doesn't grow until where they'd have joined anyway. Builder should keep it one-way when
     the sizes count joins.
  2. **The joint fade: now, with the merge,** since they work together.
  3. **3 above the bank: yes,** and watch the low side for embankments.
  - On tuning: step 18 still cuts three times what step 0 does. A smaller step, well below "a
    waterfall from", may give David's mix of rapids and big falls for less cutting.

**The merge and the joint fade built** (634804e, 2026-10-31; `docs/water-rework.md` section 17;
David's go-ahead given to Builder directly).
- **Rivers alongside each other merge** (on). A river ends where its water first comes within a block
  of another river's lower water, falling to it, and the rest of its way alongside is dropped. 21
  merged on the 14:07 preset.
- **Joins ease down** over the last 3 of the joined river's widths, as steeply as rapids go. Anything
  left is a fall at the shore.
- **"…and above the lower bank up to"** is 3.
- **David's lake slab fixed** (at "a lake from" 40). The lake's water and rim were judged on the
  uncarved ground, so a river's bank cut the rim below the water. Now both are judged on the ground as
  carved.
- **Totals barely move,** since these shape only the joins: deep points 12,880 → 12,931, flush 96.4% →
  96.5%. The game is where to judge them.
- **Noted for stage 5:** David's 14:01 screenshot of a deep, narrow gorge with sheer walls and a fall
  into it. That's a route through a ridge, with banks at "widens up to" 1 that can't slope.
  - *Counsel:* up high, that may be the gorge David wanted (answer 4). If not, "widens up to" above 1
    lets its sides slope.

**Stage 5 prepared** (Builder, 2026-10-31). David on 634804e: "very nice, very nice. i think mostly
rough edges remain. lets carry on." Builder makes the USB backup first, then stage 5, behind a "sizes:
by rank / by area" switch, about 250 lines. Builder's six questions, with Counsel's advice:
1. **A spring starts at 1 block (1 wide, 1 deep), as a slider?** *Yes:* David's "a spring is 1 block".
2. **Joins keep 90% of both areas; running adds 1 block of area per 2,000 blocks?** *Yes, to start.*
   - Watch: sizes now come from how many springs feed a river. The 13:22 preset has 192 springs for
     103 mouths, about two a mouth, so few waters may reach river size (16) from joins alone. The
     spring count and spacing become a size dial too.
   - Builder should show the spread of names on David's preset with these starting values.
   - The names still to confirm: spring to 2, stream from 3, river from about 16.
3. **Shape guards from 1:1 (steepest) to 16:1 (flattest)?** *Let the steep end go past 1:1, to 1:2.*
   David's "a 2 deep spring with steep falls and drops" is 1 wide by 2 deep, deeper than wide.
4. **Two bowl shapes, steep and flat, mixed by the slope, or one?** *Two.* The same slope that tips the
   size tips the bowl, with nothing left to set by rank.
5. **One "banks fade over (widths)" for all water, plus "at least" in blocks?** *Yes.*
6. **Placement** (David's call): the switch and sliders in the Line section's Sizes, where the rank
   tables are; the rank sliders and "orders per rank" greyed out by area; "rivers by name" beside
   "shores in the close-up". *Counsel:* it's where the sizes live now, so it reads naturally.
- **The rough edges:** a polish pass after stage 5. New sizes change widths, so they change most of
  the edges (the gorge, the cuts).
- **David's answers** (2026-10-31): "1. they can. but on my recent check, they looked ok in general. id
  give it a try, if it was a setting, but its not broken now. 1 block may be better or worse. 2. thats a
  good starting point. but sizes are not a problem now so we are only checking this. make sure it can be
  disable with a checkbox. the names dont matter, since we are going by area. 6. agree. the rest, i dont
  know. you can tell her to proceed. its looking pretty good right now. im almost happy"
  - **Stage 5 is an experiment:** a "sizes by area" checkbox, off by default, so today's sizes stay
    until David ticks it.
  - **Names are labels only.**
  - Questions 3–5 went as Counsel advised (1:2 at the steep end, two bowls, one fade), with
    Builder free to object.
  - Relayed as a go-ahead after the backup.
- **Where the rivers stand:** "its looking pretty good right now. im almost happy." Polish comes after
  stage 5, from David's screenshots.

**Stage 5 built** (f134bc0, 2026-10-31; `docs/water-rework.md` section 18), as an experiment behind
the "sizes by area" box, off by default. The backup through 634804e is on D: (git fsck clean).
- **The settings, all sliders:**
  - a spring starts at 1;
  - joins keep 90%;
  - running adds 1 per 2,000 blocks;
  - the slope lays the water out from 16 wide per block deep (flat) to 0.5 (steep, 1:2);
  - two bowls;
  - one fade for all water.
- **The readout and overlay:** a readout of the length by size, and a "sizes in the close-up"
  overlay.
- **Builder added one thing unasked:** a lake passes its rivers' water on to its outlet, so a river
  leaving a lake doesn't restart as a spring.
- **On the 14:33 preset, ticked at the start values:** rivers come out much smaller. The median is
  1.8 blocks of water across against 22 unticked, 74% of the length is spring-sized, and none is
  river-sized. At 1 per 250 blocks run, it's near today's sizes.
  - **Why:** branching is 0, one trunk a mouth, so rivers gather few springs (15 at most), and sizes
    by area grow mostly by joining. Counsel's warning came true.
- **Builder's questions:**
  1. Is mostly spring-sized water what's wanted, or should running grow faster?
  2. Would more branching suit it better?
- *Counsel:*
  - Joining is David's rule ("joining should be the rule, and growing the exception"), so try
    branching first. If that clutters the map, speed up running instead.
  - The later pull toward nearby water would also add joins, which is what sizes by area feeds on.
- **David tried it** (preset 15:20, box ticked, no comment yet). He tuned by running, not branching:
  - a spring 3;
  - joins keep all (1.0);
  - a block more every 50 blocks run, the slider's fastest end;
  - steep 1:1;
  - flat at 0.24, steep at 6;
  - branching still 0.
  Against unticked:
  - water across: median 32.5 against 22, most 897 against 336;
  - widths: median 14.9 against 9.7, widest 135 against 57;
  - deepest 22 against 7;
  - 73% river-sized;
  - flush 95.1%.
  So sizes by area can match today's and go past it, with the biggest growing most. One screenshot:
  a narrow river along a hillside into a lake, with a stepped bank on its downhill side.
  - *Counsel:*
    - He's at the slider's end, so by his own rule on ranges it could reach further, if he asks.
    - If the biggest get too big, a cap on the most water is the lever, rather than slowing all
      growth.
    - The screenshot is for David to say whether it's a rough edge.

**Joints faded along the river** [IDEA, David 2026-10-31]: "regarding the jointing issues, perhaps a
linear joint fade similar to how we do the lateral bank fades."
- *Counsel:* where a tributary meets a river, its floor and water ease toward the main river's along
  its last stretch, as the banks fade across, instead of a hard step at the shore.
  - Real tributaries mostly meet at the main river's level, unless they're hanging valleys with a fall.
  - A fade could take most of the drop and leave the rest as a fall. That fits the side-by-side merge
    with a waterfall.
- Sent to Builder to weigh against the ridge fix (now building) and stage 2's fade along the river,
  for David to choose. Not a go-ahead.

**What to judge while tuning, and what to leave** (Counsel, for David, 2026-10-31). Each later stage
replaces some sliders, so time spent on those now is lost.
- **Leave for now:**
  - every size and shape (stage 5);
  - how many lakes and ponds there are, and where (stage 3);
  - how a cut starts and ends along the river, and the notches at bends (stage 2);
  - how far lone streams run, and fizzling (stage 6);
  - snowmelt (stage 7);
  - water in dry country (stage 8);
  - the corridor and grain (stage 4 retires them);
  - the bake time (the overhaul after forests).
- **Judge after stage 4:** meanders and wander down low, and the land-lowered numbers.
- **Worth judging now, since it stays:**
  - where springs start, and how many;
  - where mouths are, and the paths rivers take across the map;
  - the banks across the river (the compressor);
  - waterfalls, steps and rapids;
  - smoothing and corner cutting.

**Builder's assessment: `docs/water-rework.md` (c67dabc, 2026-10-31).** Nothing built.
- **The compressor works.** It's today's banks made gentler, so it changes *less* land than now.
  "Flush up to" brings land within the threshold **flush** (rather than leaving it alone, which would
  leave a 1-2 block step); the ratio, knee and feather do the rest. About 150 lines.
- **The water's height stays from the real ground** (the lowest ground across each point), so it never
  stands in the air. The plan gives the path, the real ground gives the height, and the land gives way
  at rises.
- **Follow-the-land pieces, ranked:**
  1. a local re-route only where a cut would be deeper than X, which replaces the corridor;
  2. a nudge toward lower ground;
  3. the webbing near reaches, later;
  4. the max pyramid, not needed yet.
- **Lakes picked first:** feasible, about 100 lines. Deep basins are always kept, and picked lakes keep
  only their main pool. It retires ponds behind rises.
- **One water by area:** Shreve; a join keeps about 90%; about eight sliders, plus the name
  thresholds (river from about 16, great river from about 200-250).
- **Retires about 450 of the 650 patch lines.**
- **Order,** each stage behind a switch:
  0. the measure and overlay, plus the rapids bug fix;
  1. compressor banks;
  2. attack and release;
  3. lakes picked first;
  4. the local re-route;
  5. sizes by area;
  6. petering out;
  7. rivers first in the Climate tab;
  8. dry beds.

**Lakes and ponds, picked before the water moves** [David 2026-10-31: "can we pick lakes and ponds
deliberately before water moves so i can control how many there are, or do they need to happen in one
go?"]. *Counsel:* they can be picked first, and it stays linear.
- **The basins come from the ground alone.** Step 5's fill and spill already finds every dip with no
  way out and the lowest point of its rim, before any water is counted. (This was the "basins first"
  idea of 2026-10-28.)
- **Choose which basins are lakes,** the way springs are chosen:
  - how many, or how far apart;
  - the smallest size for a lake, and for a pond;
  - fewer in rough ground (or in the mountains) and some above sea level, as weights.
  It's a list David can count and steer.
- **Every basin not chosen is passed over:** the water flows on as if the dip were filled to its rim
  (the land makes way: cut through, or filled level), so no pond appears where none was picked.
- **Then the water moves:** springs, the network and the routes run through the chosen lakes. A
  lake's outlet starts a river, as big lakes' outlets do now.
- **The order:** basins (ground only) → lakes chosen → springs → network → water laid. One way.
- **An extra way, if wanted:** stamped lakes and ponds placed at points, as peaks are, for the
  odd deliberate one where no basin falls naturally.

**Petering out only where it has nothing to do** [David 2026-10-31]. *Counsel:* the drainage already
knows where each spring's water would go, and how far it is to the next water (a river, a lake, the
sea). So a spring peters out **only if that next water is too far** (more than N blocks): it has
nothing to join. Anything with somewhere to go keeps going. It soaks away on the first flat ground
past its length, not halfway down a slope. In dry country, dry beds (below) can do the rest.

**Still for David to make explicit:**
1. **The big choice:** the water follows the land (A), the land makes way (B, Builder's pick), the
   webbing, or a mix.
2. ~~The Climate order~~ *(decided: rivers first.)*
3. **Springs:** the ways stay, so this is now tuning how many go in the mountains, hills and flats,
   and whether steepness is read (David's hill springs that wander and peter out).
4. **The thresholds as areas:** springs 1–2 and streams 3–8 are set. Rivers "at least 8 wide" and
   great rivers "64 or more" were said as widths, so they need restating as areas.
5. **Growth numbers:** how fast by running, how much of both areas a join keeps, how hard the slope
   tips the shape, the deepest and broadest shapes, and the biggest any water gets.
6. **Petering out:** the rule is set (only with nothing to join); N, how far is too far.
7. **Lakes and ponds:** whether to pick them first (proposed above), then the numbers: how many, how
   big, how far apart, and the weights for rough ground and height.
8. **Gathering:** how strongly land is "collected to one river", how far apart big rivers are, and
   rivers "through the middle between two mountains".
9. **Waterfalls and rapids:** when a drop is a fall and when rapids ("lots of waterfalls" high up).
   On David's preset (no steps, falls from 25), nearly every fall is a curtain of 2 to 5 blocks: 4,902
   of them. The step and the fall height want a look together, by eye.
10. **Where water meets the sea:** deltas (from what size, on how flat a coast), mouths, the shore at
    Z127, the sand flats.
11. **Dry beds:** whether to have them, and their thresholds by heat and dryness.
12. **The profiles drawing's job now:** the bowl's shape only, since sizes are now by area.
13. **Valleys and banks:** how wide and steep (these mostly follow the big choice).
14. **Not yet mentioned:** marshes and wet ground; whether water ever splits going downstream (beyond
    islands and deltas); how long making the water may take.

---

## 1. What water should be: David's words

### Perfect rivers, as David pictures them (2026-10-31)

David's own description, kept close to the original words, in the order water runs:

- **Springs are 1 block,** from the mountains and the hills.
- **Hill springs** come out of the hill gently, find their way across the meadow, and **can peter
  out, more often than springs from steep regions.** (The relative-height setting helped this, "but
  overpopulated elsewhere i think".)
- **Springs high up** have **lots of waterfalls**, "not very accessible but pretty", and can be very
  shallow: "just a 1 block path of water trickling down the mountain."
- **Creeks form where a few springs meet.**
- **A few ponds and lakes in the mountains, only a few.** They run over into a stream near their path
  toward the sea.
- **Creeks run through the valleys and the hills, snaking around through gentler terrain.**
- **Creeks and streams are one stage:** "really two ends of the same stage in between springs and
  rivers."
- **Where creeks or streams combine, they form rivers, at least 8 blocks wide.** Rivers can happen to
  join and form **great rivers, 64 or more wide.**
- **Big areas of land** still have a few waterways, "but it'd be more collected to one river."
- **The hierarchy:** creeks and streams gather up the springs; rivers gather up the creeks and streams;
  **rivers run through the middle between two mountains, in a valley.**

### Sizes, and how water grows  [David 2026-10-31]

- **A spring is 1 block.** If it combines with another, it becomes 2 blocks. If it combines with a
  third, it's a **stream**. **"Creek" is just a colloquial word,** not a stage.
- **Springs are 1–2, streams are 3–8.** (From the message before: rivers at least 8 wide, great
  rivers 64 or more.)
- **Water grows as it goes,** not only where it joins: a 1-block spring eventually becomes 2 blocks.
- **It chooses its shape by the land:**
  - over **flat ground**, a spring grows **2 blocks wide**;
  - with **steep falls and drops**, it grows **2 blocks deep**.
- **The general rule, from spring to delta:** a stream might be 3–4 blocks of water in profile. As
  blocks are added to that profile, between stream and river, they go to make it **deeper and narrower
  through rapid drops**, and **flatter and broader through flat, gentle land**. "That logic works from
  spring to delta."

*Counsel's reading:*
- **The size is the number of water blocks in the cross-section** (its area), and **the land decides
  how that area is laid out.** Steep land stacks it (deep and narrow: slots, plunge pools, gorges);
  flat land spreads it (broad and shallow: meadow rivers, then deltas). One rule for every rank, so
  no separate rules per type (David's "write too many rules, then consolidate", done).
- **Counting springs is a known measure:** "a spring meets a spring, 2; meets a third, a stream" adds
  up every spring above a point. Hydrology calls that **Shreve magnitude**. Today's ranks use
  **Strahler order**, which only steps up where two equals meet (so a creek joining a river adds
  nothing). Shreve fits David's rule better: every spring that joins makes the water a little bigger.
  "Grows as it goes" can be added on top, by length.
- **The profiles drawing still has a place:** it gives the bowl's shape. This rule gives the size,
  and the width-against-depth split by slope.
**David's answers (2026-10-31):**
- **Sizes are area:** the number of water blocks in the cross-section. **Width against depth comes
  from the slope:** the water "has no speed but it has a run over rise", so the fall over the run
  decides the split.
- **Growth is unrealistically fast, on purpose:** "the speed at which springs-streams-rivers increase
  in size has to be unrealistic to get satisfying proportions."
- **It grows slowly just by running, and quickly by joining:** "two runs of water joining should
  probably add most of their area together."
- **Pseudoscience, on purpose:** "the goal is just being cool and passing the threshold where people
  glance at it and think 'yeah that could be real'. most people don't really think about how flat the
  himalayas actually are and imagine hyperbolic terrain in their head to begin with."
- **What makes a stream a river: joining is the rule, growing the exception.** "But the logic should
  be thresholds; i think maybe treating them as discrete types of water is misguided."

**One water, not types** [David 2026-10-31]. *Counsel:* there's one continuous size, the area, from
spring to delta. "Spring", "stream", "river" and "great river" are names for ranges of it, crossed at
thresholds. They're useful for the hover readout, for mineralogy, for bridges and for what players call
things. They aren't separate kinds with their own rules. Joining is how water usually crosses a
threshold, since it adds most of both areas; growing by running crosses one now and then. Some looks can
still switch on at a size (deltas, wide meanders, islands), as thresholds on the same number. Today's
code has a table per rank (sizes, shapes); that would become smooth curves over area and slope.

**Dry beds** [IDEA, David 2026-10-31: "can we use the temp layer to just remove some water and leave
dry beds?"]. *Counsel:* yes, and it fits the pipeline neatly.
- **The beds are made the same everywhere:** the shape of the channel comes from the water stage.
- **A later step decides which beds hold water.** It reads the beds and the climate after them, so
  heat (and rain too, if rivers come before rain) can dry the small ones. Their carved channels stay,
  sand or gravel in them (mineralogy), the way wadis and arroyos look.
- **Big water stays wet:** a bed stays wet if its area is above a threshold that rises with heat and
  dryness. So a great river crosses a desert (the Nile, already fine by David), while its little
  tributaries are dry beds.
- **It answers the desert catch above:** small streams in dry country don't need a shorter length.
  They keep running as beds, and the water goes out of them.
- **Later, with seasons:** beds wet in spring and dry in summer, from the same threshold moving
  through the year.

**Petering out against growing** (David: "how is this decided? we may not really need petering out if
they work right"). *Counsel:*
- **Today:** only springs no river took peter out, after 3,000 blocks at the least water, further with
  snowmelt or slope-foot water.
- **With growth by running, water never shrinks,** so petering out has to be its own rule. The
  simplest: **a spring that hasn't joined other water within N blocks soaks away** (N, a slider).
- **That alone gives David's earlier wish,** hill springs petering out more often than steep ones,
  with no rule about flatness. A steep spring drops fast into a valley and meets other water soon. A
  hill spring wanders across a meadow and runs out of length first.
- **N could be very long,** which turns petering off in practice, if springs work right.
- **One catch, if rivers move before rain:** shorter lengths in dry country (streams vanishing into
  desert) would then need another way to know where it's dry, such as heat, or the distance from the
  sea.

**Counsel, on the shape rule:** the flatter the water runs, the broader and shallower; the steeper,
the deeper and narrower. The area is what it has gathered, laid out by its slope. In terms of
controls, that's probably:
- how fast water grows by running;
- how much of their area two waters keep when they join (most of it);
- how strongly the slope tips the shape between deep and broad;
- a deepest and a broadest shape for each size, so a river never turns into a sheet or a well.

*Counsel's notes on the description above:*
- **Four stages, not five:** spring (1 block), creek-or-stream, river (8+), great river (64+). Today
  there are four ranks after the spring (creek 2–9½, stream 9½–24, river 24–59, great river 59–148,
  from the profiles drawing), so rivers would start smaller than now (8, not 24) and creeks and streams
  would share one rank. To check with David against "drawing wins".
- **Where a spring is decides how it behaves:** steep ground means trickles, waterfalls and short
  lives that join up; gentle hills mean slow wandering and petering out. Today springs read height alone,
  and nothing reads steepness or flatness (Builder's inventory), so this needs a new input.
- **"Only a few" lakes in the mountains** is the opposite of today, where rough ground gets the most
  ponds.
- **"More collected to one river"** asks for strong gathering: few big rivers draining wide areas,
  not many parallel ones.

### Gathered earlier

From the docs, for David to confirm, cut, reword or add to. With the list above, this is the test every
design has to pass.

**The rivers**
- **A pleasant web:** "a *pleasant* webbing of little springs and creeks and rivers that have bridges
  and so on." The goal is a pleasant, bridgeable web, not exact sizes. Sizes grow by ratios ("drawing
  wins", the profiles drawing of 2026-10-30).
- **Spread:** "rivers scattered evenly around the world, so the network of rivers and smaller ones fans
  out through the terrain. Not too evenly though." From the world map they already look "decent and
  very Dwarf Fortress-like".
- **They follow the land:** "curving between hills instead of slicing through them"; "roll like a
  marble between hills, finding natural valleys and deepening them, rather than making its own"; they
  "should be finding a way around" hills.
- **Banks slope the land toward the river,** at least a width each side (a 32-wide river, at least 32
  blocks). Shores are **flush by default, ledges by context**.
- **Where the springs are:** "more small springs in the flatter areas, or big rivers that cut through
  them from the highlands."
- **Islands and braids,** like the winding river in the screenshot: "must be preserved, as it is
  great, and duplicated if possible."
- **Waterfalls,** and from above a fall reads lighter, not as the river just carrying on.
- **Rivers aren't rain:** "We only want rivers, lakes and ponds to look right. We shouldn't confuse
  rivers with rain." Wetness for ecology is rain plus nearness to water; a great river through a
  desert is fine.

**Lakes and ponds**
- "There should be some lakes and ponds above sea level."
- But "way way way less lakes and ponds in the rugged regions."
- No raised square pools, no water standing against air.

**David's original method** (2026-10-30), for the record. Deltas at the lowest, flattest points, not
too close together. Climb toward the high ground in increments, favouring long flat runs, then steps
(the waterfalls and rapids). Split halfway, and halfway again, up to the springs. Springs no river
found run forward to a lake or pond, or fizzle out.

**Not yet said, worth asking:**
- the sea itself: the coast, beaches, the shore at sea level (Z127), the sand flats just under it;
- marshes and wet ground;
- how big the biggest rivers and lakes should be, against the world's size;
- whether rivers ever split going downstream, away from deltas (braids are a small case of this).

**Rules that still stand** (decided before; the redesign keeps them unless David says otherwise):
- the linear pipeline: nothing later feeds back, and Climate never reads the carved ground;
- water is plain water blocks, with no flow simulated;
- the world is made completely before you enter it;
- rivers don't come from rain.

## 2. The big choice: does water follow the land, or does the land make way?

- **Water follows the land.** Rivers are found where water really runs on the finished ground, hills
  and crags in. They go round hills because the ground says so. The catch: the finished ground is full
  of little hollows (crags and hills are feathered together, David), so lakes and ponds come out
  everywhere and have to be filtered, by size, depth and how rugged the land is.
- **The land makes way.** Rivers are planned first, on the rough map, and the ground round each one is
  shaped into a valley wide enough to look natural (the Génevaux paper's way). No notches, because the
  hill gives way. The catch: the land near rivers changes, so it touches the hill country David has
  tuned. In 2026-10-30's review, `docs/rivers.md` section 6 judged the full reversal "not worth
  borrowing", as it would hand the land's shape to the rivers.
- **Today's system does half of each:** it plans on one ground (the broad ground), lays the plan on
  another (the real ground), and patches the gap: corridors, ponds behind rises, cut-or-pool, falls at
  lips, layers of water-level rules. That's where control was lost.

**Counsel, rereading David's words:** "finding natural valleys and deepening them, rather than making
its own" is a third answer, and a clean one: **the water follows the land at the large scale, and the
land gives way at the small scale.**

**Builder's view (`docs/water-inventory.md`, 2026-10-31): B, the land makes way, and it fits that
sentence.** The broad shapes (plates, ridgelines, peaks, rise) already drain sensibly, because the
plan is made from them. It's the fine layers (crags, hills, summit crags, torn coast) that break the
rivers: they're noise, made with no idea of water, and "noise hills have no valleys that join up". So:
- keep the plan on the broad ground (steps 5 and 6 as they are);
- along each river, shape the land into its valley: the floor at the planned water level, the sides
  rising to meet the land, and **the fine layers fading out toward the floor**;
- planned lakes work the same way: their basins are shaped and filled, with the fine layers faded
  inside them;
- the banks are already a small version of this, so B grows them into valleys.
- *Middle way:* B, then a fine fill and spill inside each valley as a check, with a lake wherever the
  shaped valley still holds water.

**What A would cost instead:** reading the real ground at 8 to 16 blocks (4 to 16 times today's
reading, 1 to several GB of memory). A rule to cut through small pits, "the land making way, in
miniature". And rivers going wherever the noise sends them: staircases of waterfalls down crags, lakes
in every hollow.

**Counsel, corrected:** I said the following was done on the wrong ground. Builder's point is better.
The broad ground is the *right* ground to follow, since it's the only ground with real drainage. What
was missing is the fine ground making way. So "natural valleys" means the broad ground's valleys, and
"deepening them" means the valley shaping. **What B gives up:** a river never threads a small gap
between hills; the hills open for it instead ("curving between hills" happens at the broad scale, not
the crags'). Mountains also change near rivers, since every river gets a valley or a gorge, so the hill
country David tuned is smoothed near water. The fine-layer fade is also a cheap stand-in for an erosion
stage (option D), and could grow into one later.

**Builder's costs for B:** about 200 new lines, and about 600 removed. One more reading of the ground
per column near a river (the broad height as well as the full one). Sliders for the valleys' width and
steepness, by rank, perhaps by how rough the land is.

**For David to decide.** David, 2026-10-31: Builder's visual aid "is quite helpful to explain the
problem. but im not ready to choose a solution."

### If the land makes way: David's conditions, and the compressor  [2026-10-31]

David: "if the land makes way, it really has to be well feathered and gently pull down land. the less
it changes the land, and the more gradually, the better, id like about half the inland shore to be
flush with water at least. this is really the sticking point. what kinds of 'look-ahead' are cheap? how
do compressors do it?"

**The conditions** (the test for B):
- well feathered, pulling the land down gently;
- as little change to the land as possible, and as gradual as possible;
- **at least half the inland shore flush** with the water.

**How a look-ahead compressor works** (audio): it delays the sound by a few milliseconds, so it can see
a loud peak coming and start turning down before it arrives. It reads the loudest point in a short
window ahead, eases the gain down over the attack, and back up over the release. So the peak is
caught with no sudden jump. Its controls:
- **threshold:** below it, nothing changes;
- **ratio:** above it, how much is taken off (4:1 leaves a quarter of the excess);
- **knee:** how softly it starts at the threshold;
- **attack and release:** how early it starts and how slowly it lets go;
- **look-ahead:** how far ahead it reads.
Upward compression is the other half: it raises the quiet parts.

**The same thing for land beside a river** (Counsel). The "signal" is how high the land stands above
the river's water.
- **Threshold:** land within a block or two of the water is **left alone**. That's what keeps shores
  flush, and the threshold is the dial for "half the shore flush".
- **Ratio:** above the threshold, the land is brought down by a share, not cut to the water. A hill 20
  blocks above the threshold at 4:1 stays 5 above it. The ratio eases back to 1:1 (no change) with
  distance from the river: that's the feather, across.
- **Soft knee:** no line where the change starts.
- **Attack, release and look-ahead, along the river:** the river reads the highest ground some way
  ahead, so the lowering starts well before a rise and lets go well after it. A valley opens gradually
  instead of a notch. That's the gradual pull-down, along.
- **Upward compression, the water's half:** where the banks stand only just above the water, the
  water rises to meet them (both banks, never against air), so more shores come out flush without
  touching the land at all.

**Cheap kinds of look-ahead** (all one-dimensional along a river, or a single lookup):
1. **The highest point in a window ahead,** along each reach: one pass, by the sliding-window
   trick limiters use. Very cheap.
2. **An envelope over the cut it needs:** smooth "how much this spot must come down" forward (the
   attack) and backward (the release), as a compressor smooths its gain. Also one pass.
3. **The distance from the river,** which the carve already finds for every column: the falloff
   across.
4. **A pyramid of highest points** (each level keeping the highest of four below it, like the cone map
   the renderer already has): "the highest ground in this 64-block box" becomes one lookup, so a
   route can look far ahead without reading every block.
5. **The plan itself:** the broad-ground network already looked ahead, at the scale of mountains.

The expensive kind is the one to avoid: searching many possible paths ahead (beam search or A* over
the real ground). It's the routing corridor's cost, and more.

**How to judge it** (numbers Builder can report on David's world): the share of inland shore that's
flush, how much land changed (on average and at most), and the steepest pull-down. Then David tunes
the threshold, ratio, attack and release by eye against those.

David: "i wish there was a way to get a block-scale image of the entire map, and use hsl contours to
find every path between every elevation increase of any kind, forming a white feathered trace that
looks like a bunch of bubbles and their borders, softened, and let the water only look at that
webbing."

*Counsel's reading:* contour lines close into loops round every rise, the bubbles. The borders where
neighbouring bubbles meet are the low ways between rises: valleys, saddles, gaps between hills. A soft
white trace along them is a **map of every passage**, and water routed only along the web goes round
every hill by construction. It's a different answer from A or B:
- unlike A, **pits don't matter.** A crag hollow sits inside the web, not on it, so it never becomes a
  lake unless we want it to;
- unlike B, **the land isn't reshaped:** the river threads the real gaps, which is "curving between
  hills" at the crags' scale too.
- **It has a known shape.** The borders between neighbouring rises' areas are the valley lines
  (thalwegs, in hydrology; on the map they're the edges of a Voronoi-like diagram round the high
  points). Image processing finds them with a skeleton of the low ground. A "feathered" trace is a soft
  field, 0 to 1, of how much each spot is a passage. The routing would prefer high values, the cheaper
  detour of option C done properly.
- **The cost is the resolution.** Block-scale for the whole map is 72,000 × 72,000, about 5 billion
  pixels, far too much. At 8 blocks it's about 80 million, the same reading as option A. It could
  also be made only near each planned river (the corridor's idea), at full detail.
- **It's useful beyond water:** roads follow valleys too (the Civilization stage), and so do paths
  and settlements.

Not chosen, not sent to Builder for a build. Worth Builder's assessment when David wants it.

### The Climate tab's order: rivers first?  [IDEA, David 2026-10-31]

David: rivers "are obviously the piece de resistance", which "makes me wonder if rivers should come
before the temp, the wind, the rain. because they don't mean anything to rivers, and rivers CAN mean
something to them."

*Counsel:* it fits the linear pipeline. Since rain was taken out of rivers (2019c03), the rivers read
**only the ground and, for snowmelt, the temperature.**
- **Rivers before wind and rain** costs nothing.
- **Before temperature too,** snowmelt would read height alone (or a quick cold-by-height guess),
  unless Temperature stays first.

What rivers could then give the climate after them, all one way:
- rain or humidity near lakes and rivers: David's **lake effect** (wanted 2026-10-28, left out then
  because it would have looped);
- milder temperature by big water, as by the sea;
- wetness for Ecology as rain plus nearness to water, as already decided;
- **cues that show where water is** [David 2026-10-31]: "lots of visual cues for rivers and streams on
  plains, like tall grass, and frog sounds, and lushness from it in general, it is difficult to see in
  such a big world." Recorded in `docs/worldgen.md` (Climate, "What Ecology will read"), with Ecology.

The current order (Temperature · Wind · Rain · Rivers · Mineralogy · Caves) was David's, 2026-10-29.
Changing it is David's call.

## 3. What's built, and what survives

**Builder's inventory: `docs/water-inventory.md` (5715596, 2026-10-31).** Every part, with what it
does, what it reads, sound or patch, its lines, and its sliders. In short:
- About **2,900 lines** of water code, plus ~700 of tests. About **650 are patches**, nearly all in step
  7 and the carve:

  | Patch | Lines |
  |---|---|
  | ponds behind rises | 260 |
  | searching for the real lakes round step 5's | 140 |
  | the route in a corridor | 110 |
  | falls at their lips | 45 |
  | reaches into lakes | 30 |
  | the carve's patches | 25 |
  | ending where it meets the sea | 15 |
  | smaller ones | ~25 |

- **Steps 5 and 6 are entirely sound:** fill and spill, springs, mouths, growing, stream order. One
  leftover: the network's planned bed, whose two sliders live in River network but act in step 7.
- **Survives either way:** steps 5 and 6; the sizes and profiles; steps, waterfalls and curtains (with
  the rapids bug fixed, below); the banks; the carve's core; storage; the Interpreter, the far terrain
  and the sea.

**Two findings from the paused questions:**
- **"Rapids at 1" works by hiding a bug.** A rapids drop spread upstream can reach past the waterfall
  above it after that fall's curtain was set. The water above gets pulled down, but the curtain and
  cliff stay up, and the real drop moves upstream with no curtain. At 1, rapids never reach past the
  next point. Not fixed.
- **Springs are placed by height alone; nothing reads flatness.** Ponds come along every river, with
  one cut depth for all ground, so rough ground, with more rivers, gets more ponds. That's why David's
  two wishes were tied together.

## 4. How we rebuild

1. **Goals first** (section 1), signed off by David.
2. **The big choice** (section 2), decided by David.
3. **The stages written down before any code:** for each, what it reads, what it hands on, and how
   David judges it (its overlay). It's a short chain, in pipeline order.
4. **One stage at a time, beside the old one:** the current rivers stay behind a switch, the new stage
   gets its overlay and David's sign-off before the next starts, and the old way is retired only once
   the new one is better.
5. **Keep what's sound.** Rebuilding the plan doesn't mean rewriting the parts that already work.

**A tool for it:** the Interpreter's Show switches have **"rivers and lakes"** (2d77921). Off, the
close-up shows the ground before the carve, with only the sea's water; a world spawned with it off
says "rivers and lakes left out". Flipping it no longer remakes Climate (Show switches used to restart
it, about 10 s of full CPU).

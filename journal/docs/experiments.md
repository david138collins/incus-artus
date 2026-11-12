# The experiments in old

The Oracle's first experiments (`vision/oracle-review.md`, part 7), flown in old Incus Artus before anything new is
written (`master/plan.md`, phase 1). Each one's pass mark is written here **before** it flies, then its results under
it, with the logs that hold them. A journal: what's decided goes in `docs/decisions.md`, where we are in
`master/plan.md`.

---

## Experiment 1: the instruments, and real baselines  *(plan 1.2)*

**Why.** The Oracle found the record softer than it read: the worst views were never properly measured, the card's
time left out the work of making new ground, flight headers didn't say what flew, and nothing had ever run on a weaker
card. Every later experiment is measured with these instruments, so they come first.

### What was built (build 110)

- **Headers that say what flew** (`soak.rs`, `dyno_report`):
  - the build and whether its code was committed: the changed files named, the difference saved beside the report as
    `dyno-<time>.patch`, and a fingerprint of it in the build's stamp (`build.rs`);
  - every `INCUS_*` and `WGPU_*` switch;
  - the card, its kind and driver, and how it was chosen;
  - how the picture was handed to the screen, as granted, not as asked;
  - the presets, and what ran beside it, with no fixed text.
- **The lanes' work on the card counted** (`world/card_recipe/on_card.rs`, the lanes' clock). Each piece of a tile's
  or the bake's pass is stamped by the card's own clock and read back when the pass is done. Every line now gives
  sight alone, sight and the lanes together, and the lanes' work by kind (grounds, choices, cells) and as a share of
  the card's. `INCUS_LANE_CLOCK=off` leaves the stamps out, to check that the clock costs nothing.
- **Three standing marks after the route** (`dyno.rs`, `marks`; `soak.rs`, `fly_marks`). The camera is held still at
  each with a player's eye (1.38 over the ground), waited on until the live levels round it are whole (180 s at
  most), then measured standing for 20 s, uncapped:
  1. *On the ground at the spawn*: (0, 0) in the test world, looking north and level.
  2. *The mountain top on a clear day*: the highest block column near the route's peak, looking out from the middle,
     fog off (`dyno.md`'s "clear day").
  3. *Beside the quarter-block build*: David's build of 2026-11-10 (build 108; never saved), replayed from that
     session's event log (`kept logs/events-1794341728.log`: 169 placings, 146 bars of stone brick and 23 blocks of
     gabbro), standing 8 blocks off its north side, looking at its middle.

  Settling and standing are reported apart. `INCUS_DYNO_LAPS=0` flies the marks alone; `INCUS_DYNO_MARKS=0` leaves
  them out.
- **The choice of card restored** (`runner/mod.rs`, `choose_card`): `WGPU_ADAPTER_NAME` (part of a card's name) or
  `WGPU_POWER_PREF=low`; the high-performance card if neither is set. Every card found is listed in the session log.

### The pass mark (written before the flights)

Two flights, both watched by David:
- **A**: the 5070 Ti, the route and the marks (`INCUS_DYNO=baked`, the test world, the dyno's presets).
- **B**: the Intel card (`WGPU_ADAPTER_NAME=intel`), the marks alone (`INCUS_DYNO_LAPS=0`).

It passes if all of these hold:
1. **The headers pin what flew.** Both name the build, say whether it was committed, list every switch set, name the
   card that actually drew, and give the picture's handing-over as granted (A should show `Immediate`).
2. **The lanes are counted, and believably.** Every line carries "the card's whole (sight and the lanes)". Over the
   land legs, the lanes' work comes to about a frame's worth of what Nsight saw at build 104 (1.8 to 4.8 ms a frame
   over land, 0.6 to 1.4 at sea), within a factor of two either way. Far outside that means the clock is wrong, and it
   gets fixed before anything else is measured with it.
3. **The clock costs nothing.** A's route median for sight lands within ±5% of build 107's flight with build 109's
   change applied (4.31 ms less 2.8%: about 4.19 ms). If it's outside that, one more flight with
   `INCUS_LANE_CLOCK=off` tells the clock's cost from the day's drift.
4. **All three marks are measured.** Each reaches whole live levels, or the report says how far it got. The build
   replays as David built it: 169 placings, 277 cells changed and 1 refused, as his session logged. *[Corrected after
   flight A, as Counsel found: the 277 was miscounted before the flight. It took only the lines naming pieces and left
   out the 22 blocks of gabbro he placed (23 asked, 1 refused). His session's event log,
   `kept logs/events-1794341728.log`, has 299 "block changed" lines and 1 "edit refused", so the mark is 299 cells and
   1 refused.]*
5. **The Intel card flies the marks, or fails for a reason that can be named:** the error, the limit or the compiler's
   message, as written.

**What changes the plan** (the Oracle's): if the Intel card fails for any reason other than speed (memory, a limit, a
compiler fault), that shapes the store before anything new is built. The marks' standing numbers replace the record's
soft figures ("8 ms at the spawn", "14 ms beside quarter blocks"), and the gap the rebuild must close is stated again
from them.

### Results

**Flight A**: the 5070 Ti, 2026-11-11, build 110 (090904f, committed), the laptop in "NVIDIA GPU only" mode
(PredatorSense), uncapped, 2560 × 1600; Edge, Aseprite, Discord and Steam running beside it, as for build 107.
Log: `logs/dyno-1794414666.txt` (kept in `kept logs/`).

| | sight on the card, ms (median / 95th / 99th) | sight and the lanes, ms (median / 95th / 99th) | frames a second (median) |
|---|---|---|---|
| The route, baked (2 laps) | 4.19 / 9.02 / 9.79 | 4.53 / 9.64 / 13.93 | 222 |
| **On the ground at the spawn**, standing | 5.94 / 6.22 / 6.79 | the same (the lanes idle) | 167 |
| **The mountain top, clear day**, standing | 6.59 / 6.90 / 7.48 | the same | 150 |
| **Beside the quarter-block build**, standing | 11.74 / 12.11 / 12.25 | the same | 85 |
| *The target: 240 a second* | *4.17 in all* | | |

Against the pass mark:
1. **Headers: pass.** Build, commit and "committed"; every switch; the card ("chosen by the default, high
   performance"); "Immediate (as granted)". One blemish: the switches list also shows the build's own stamp
   (`INCUS_BUILD`, `INCUS_BUILD_CHANGED`, `INCUS_BUILD_NUMBER`), which cargo passes to the game as it runs. They're
   to be left out of the list.
2. **The lanes: pass.** Over the land, 97 ms of lane work a second, about 1.6 ms in a 16 ms frame. The build 104 trace
   showed 1.8 to 4.8 ms in frames of 14 to 18 ms (`kept logs/nsight/queries/f104_land.json`), so the clock agrees
   with Nsight at the low end, as expected after builds 105 to 107 shared the grounds between levels. Across the route
   the lanes are 12% of the card's time (9 to 19% by leg), and the grounds pass is about 80% of them. They add little
   to the median (4.19 to 4.53) but a lot to the tail (the 99th 9.79 to 13.93).
3. **The clock costs nothing: pass.** Sight's route median is 4.19 ms, exactly as foreseen (107's 4.31 less 2.8%).
4. **The marks: pass.**
   - The spawn's live levels were whole in 137 s, the mountain top's in 44 s.
   - At the build, they were not whole after 180 s (8,315 of 8,820 tiles: it's 4,200 blocks from the middle), then
     whole 82 s after the build went in, with the bake patched.
   - The build replayed exactly as David's session logged it: 169 placings, 299 cells changed, 1 refused (his event
     log, `kept logs/events-1794341728.log`: 299 "block changed", 1 "edit refused"; the pass mark's 277 was a
     miscount, corrected above).
   - The card stayed awake (2.7 to 2.8 GHz). The processor area peaked at 103 °C only while loading, then 76 to 98.
5. **The Intel card:** flight B, below. Its first try was held back so the laptop could be switched to Optimus, where
   the Intel card drives the screen itself (in "NVIDIA GPU only" its frames would be copied across to the NVIDIA's
   screen).

**What the marks say.** Standing, the card alone takes 1.4 times the target's whole frame at the spawn, 1.6 times at
the mountain top and 2.8 times beside the quarter-block build. The record's soft figures were near: "8 ms at the
spawn" is 5.9 here, and "14 ms beside quarter blocks" (a frame time with the panel up) is 11.7. Standing still, the
lanes do nothing, so the marks are sight's alone. While a mark settles, the lanes add 4 to 8% of the card.

**Flight B**: the Intel card (Intel Graphics, the Core Ultra 9 275HX's; driver 32.0.101.8132), 2026-11-11, build 110
(078f326: the same code as 090904f, only docs committed since), the laptop switched to Optimus so the Intel card drove
the screen itself, the marks alone, uncapped, 2560 × 1600. Log: `kept logs/dyno-1794416695.txt`.

**It ran, and nothing but speed held it back.**
- The card offers what sight asks: its largest buffer 2,047 MB and 262,144 storage buffers a shader, the same as the
  NVIDIA's.
- The world's 266 MB went on in 0.44 s.
- The recipe and sight's three passes compiled without a fault (6.8 s and 9.4 s, against 5.9 and 8.5 on the NVIDIA).
- The 4,096 columns round the spawn came out the same as the processor's to the bit, and so did the 4 cells of 32
  (41 ms against the NVIDIA's 13).
- There were no warnings or errors in the session, and the card was never lost.
- The build replayed exactly again: 169 placings, 299 cells, 1 refused.

| Standing | sight on the card, ms (median / 99th) | frames a second | against the 5070 Ti | the live levels after 180 s |
|---|---|---|---|---|
| On the ground at the spawn | 22.69 / 22.84 | 41 | 3.8 × | 4,106 of 8,820 tiles |
| The mountain top, clear day | 20.57 / 21.07 | 47 | 3.1 × | 6,481 of 8,820 |
| Beside the quarter-block build | 29.49 / 32.26 | 33 | 2.5 × | 7,000 of 8,820 (after the build) |

What to keep in mind with these:
- **The live levels never got whole** in 180 s on this card: the lanes ran at about the NVIDIA's share of the card
  (8%), but each tile takes about three times as long. Some of the view was drawn from coarser cells than on the
  NVIDIA, so the numbers aren't the same picture's cost. Standing longer would make them so.
- **The enlarging** (the upscale pass) takes 1.7 to 1.9 ms here, against 0.17 on the NVIDIA.
- **The heat watch** reads the NVIDIA's sensors, so it says nothing about this card (its clock reads 0 while the NVIDIA
  idles).
- **Against a 980 Ti** *[estimate, low confidence]*: this is a small integrated card sharing the laptop's memory,
  likely well under a 980 Ti in both arithmetic and memory speed, so these numbers bound a 980 Ti from below, loosely.
  A 980 Ti at 1080p also has about half these pixels.

**Against the pass mark: all five hold.** The Intel card failed for no reason but speed, so nothing here shapes the
store before it's built (the Oracle's "what changes the plan"). The marks' numbers on both cards are the baselines the
rebuild is measured against.

**Fixed after the flights:** the header's switches list no longer repeats the build's own stamp (`INCUS_BUILD*`).

---

## Experiments 3 and 4a: what the library's checks cost, and how much of each mark is light  *(plan 1.3)*

David's go: "go ahead ... and start the tests" (2026-11-11).

### What was built (build 111)

- **Experiment 3:** `INCUS_SHADERS_UNCHECKED` compiles the two big shaders without wgpu's runtime checks
  (`whole::shader::big_module`, `create_shader_module_trusted`): sight's march (all its passes) and the world's recipe
  (the tiles and the bake). Three guards come out: buffer reads clamped to their bounds, loops given a counter, and
  divisions guarded against zero. A measuring switch only, never the default.
- **Experiment 4a:** no new code. `INCUS_SIGHT_NO_SKY` and `INCUS_SIGHT_NO_SHADOWS` switch sky light and sun shadows
  off as the game runs. Their code stays in the shader, so this measures the work skipped, not a smaller program.

### The pass marks (written before the flights)

Three flights on the 5070 Ti in "NVIDIA GPU only", each as flight A (the route twice, then the three marks; the test
world, the dyno's presets), all on build 111:
- **C**: as built (checked): the baseline for both experiments, on the same build and the same day.
- **D**: `INCUS_SHADERS_UNCHECKED=1`.
- **E**: `INCUS_SIGHT_NO_SKY=1 INCUS_SIGHT_NO_SHADOWS=1`. It flies unchecked too if D shows the checks cost 3% or more
  (then measured against D); otherwise checked (measured against C).

**Experiment 3** passes as a measurement if:
1. Unchecked draws and makes the same as checked. Sight's picture tests and the recipe's bit tests pass with the switch
   on, and D's load check says the columns and cells round the spawn are the same as the processor's. If any of these
   differs, that's a finding in itself (a shader relies on a guard) and is written down before anything else.
2. D against C, for sight on the route (median, 95th, 99th), at each standing mark, and for the lanes' work a piece.

**What changes the plan** (the Oracle's): if the checks cost 3% or more anywhere that matters (the route's median or
tail, or a mark), every later experiment in old flies unchecked so the comparisons are fair, and the new engine ships
unchecked with the checks kept in its tests. Under 3% (A/A noise is ±1.5% over a flight), the checks stay.

**Experiment 4a** passes as a measurement if E's route and all three marks are measured against the right baseline,
light's share given for each.

**What changes the plan:** the Oracle expects light to be about half the card's work near the ground. If switching it
off saves less than a fifth at the spawn mark, that expectation is wrong, and experiment 4b (light kept in the cells and
looked up) is weighed again before it's built.

### Results

**Flight C** (checked, the baseline; build 111, e1c0d6e, 2026-11-11; `kept logs/dyno-1794419654.txt`) repeats flight A:
- the marks within half a percent (spawn 5.94 ms, mountain top 6.58, build 11.71);
- the route's sight median 4.06 against A's 4.19 (−3%).

Standing marks repeat far better than the route, so small differences are judged at the marks.

**Experiment 3, flight D** (`INCUS_SHADERS_UNCHECKED=1`, flown straight after C; `kept logs/dyno-1794420516.txt`).

**1. The same picture and the same world: pass.** With the switch on, the card's 26 tests pass (sight's pictures, the
recipe's bit tests), and D's load check made the columns and cells round the spawn the same as the processor's.

**2. D against C:**

| | checked (C) | unchecked (D) | change |
|---|---|---|---|
| The route, sight median / 95th / 99th (ms) | 4.06 / 9.00 / 9.69 | 3.71 / 8.55 / 9.80 | −8.6% / −5.0% / +1.1% |
| Over the land / round the peak / out to sea / home, median | 5.25 / 2.37 / 5.71 / 5.12 | 5.15 / 1.70 / 5.73 / 4.58 | −2% / −28% / 0% / −11% |
| **On the ground at the spawn**, standing | 5.94 | 8.50 | **+43%** |
| **The mountain top**, standing | 6.58 | 9.79 | **+49%** |
| **Beside the quarter-block build**, standing | 11.71 | 15.32 | **+31%** |
| The lanes' card time a piece, over the route | about 0.19 ms | about 0.16 ms | about −15% |
| Compiling at load: the recipe; sight's three | 5.8 s; 7.9 s | 8.0 s; 11.9 s | slower |

Notes on the table:
- **It isn't the heat.** The card held 2.77 to 2.82 GHz through both flights, at alike temperatures.
- **The marks moved the same at the median, 95th and 99th,** so it's the program, not noise.
- **The settling took longer in D** (the spawn 174 s against 136; the build after the build 160 against 83) because
  the slower frames left the lanes less of the card.

**What it means.** Unchecked, sight's march is faster in the light views and much slower in the heavy ones, the views
that matter. The recipe (the lanes) is about 15% quicker a piece. The Oracle guessed the checks cost 3 to 15%. Where
it matters most, taking them off costs 31 to 49% instead. Why is unverified. A likely cause: without its loops counted,
the compiler lays out the march's long loops differently, needing more registers (Nsight's register count, read in its
own window, would show it).

**Against the rule written beforehand:** the checks don't cost 3% anywhere that matters. Removing them costs more than
that, so **the checks stay on**, and every later experiment in old flies checked. For the new engine, "ship unchecked"
isn't a blanket answer: each program is priced both ways, and the world's rules, which gained, are the candidate.

**Experiment 4a, flight E** (`INCUS_SIGHT_NO_SKY=1 INCUS_SIGHT_NO_SHADOWS=1`, checked, so measured against C;
`kept logs/dyno-1794421157.txt`). What's left on is the sun's direct light on each face (no shadow), placed lights and
fog. The light's code stays in the program; only its work is skipped.

| | light on (C) | light off (E) | light's share |
|---|---|---|---|
| **On the ground at the spawn**, standing | 5.94 | 3.46 | **42%** |
| **The mountain top**, standing | 6.58 | 5.45 | 17% |
| **Beside the quarter-block build**, standing | 11.71 | 9.76 | 17% |
| The route, sight median / 95th / 99th | 4.06 / 9.00 / 9.69 | 3.03 / 7.79 / 8.37 | 25% / 13% / 14% |
| Over the land / out to sea / home, median | 5.25 / 5.71 / 5.12 | 2.87 / 4.02 / 4.00 | 45% / 30% / 22% |

Notes:
- Round the peak, the median rose (2.37 to 3.25) while the 95th and 99th fell by over a third. That leg's median
  jumps between flights (A 2.50, C 2.37, D 1.70), so its tails are the reading there.
- With the light off, the frames left the lanes more of the card: the spawn's levels were whole in 79 s against 136.

**Against the pass mark:** measured at every mark and on the route: pass. The Oracle's "light is about half near the
ground" holds near enough at the spawn (42%), so **experiment 4b goes ahead as planned.** Light isn't the whole
story, though:
- With it free, the spawn would be inside the 240 budget (3.46 ms against 4.17).
- The mountain top (5.45) and the quarter-block build (9.76) would still be over it. There the rest of the march is
  the cost: the far view's reach and the near walk through pieces.

---

## Experiment 2: the far cells crisp, against today's blend  *(plan 1.4)*

David's go: "yes go ahead" (2026-11-11).

**Why.** The far march blends the eight cells round each point it reads (trilinear). Read alone, the cell a point is
in took 41% off sight at build 107 (the probe P7, `kept logs/nsight/probes/march_p7_one_cell.wgsl`). The rebuild's
bricks depend on David's eye here:
- **Crisp is fine:** the bricks are plain buffers, 2.6 times the cells: the simplest and fastest form.
- **Soft is needed:** the bricks become 3D textures with a border, 5.5 times the cells. The card's texture hardware
  does the blend, and the memory plan doubles.

### What was built (build 112)

- `ONE_CELL`, an overridable constant in `march.wgsl`. In the crisp build, `blend()` reads only the cell a point is
  in, as P7 did, for the far march and for the sun's and sky's rays that step through the cells. So the far view's
  shape and colour change, and so do the edges of far shadows and sky light. In the game's own build it's false and
  the compiler leaves the branch out.
- The crisp build is compiled on a thread of its own the first time it's chosen (a few seconds, blended till then),
  then kept, so switching back and forth is instant.
- On the Render page: "Far cells crisp", next to the level hand-over slider. `INCUS_SIGHT_CRISP=1` for flights.
- A card test, `sight_s_crisp_far_cells_draw`: the crisp build compiles and draws. Over a test scene, 10 to 16% of the
  pixels differ from the blend, at the same brightness, with no black holes.

### The pass mark (written before David's look)

**David's look decides:** crisp is wrong, acceptable, or good. He looks at the same places both ways, still and moving:
- standing at the spawn;
- on the mountain top;
- low over the sea, at the horizon;
- flying.

**What changes the plan:**
- **Acceptable or good:** the bricks are plain buffers (crisp), and the far march's 8-cell blend goes from the rebuild's
  design.
- **Wrong:** the bricks are textures with borders (soft), and the memory plan is redone for 5.5 times the cells.
- **Wrong in one place only** (only at the horizon, say): that's written down as a mixed answer, for the Oracle at the
  fork.

**Measured after the look**, for the record: a crisp flight against flight C, the route and the three marks.

### Results

**David's look** (build 112, the test world, 2026-11-11, about five minutes): "it doesn't look awful, but, it doesn't
look better. idk how much room there is to tweak/tune the crisp to being a bit less whorly when you move your head
against a shallow hill. sometimes it looked decent, other times a little janky. like i said idk how much it can be
massaged."

**Against the pass mark: a mixed answer.** Acceptable still, a little janky in motion over shallow slopes, never
better than the blend. *Builder's reading:* read crisp, a slope crossing the cells becomes stair-steps of whole cells,
and as the eye moves their edges slide across the hill as bands (David's "whorls"). The blend hides this by grading the
steps.

**Proposed next** (David's call): a third choice on the same switch, *crisp across, smooth in height*. Each read takes
the one cell column a point is in and blends its two cells up and down: 2 cells and 1 column entry, against the
blend's 8 cells and 4 entries. The whorls on shallow hills come from the height steps, so this aims at them while
keeping the bricks plain. If it's still janky, the answer is soft (textures, the hardware's blend, about twice the
plain bricks' memory).

**Build 113** (d2b52ef): the three ways on one choice on the Render page (blended, crisp, crisp across and smooth in
height), each other way compiled the first time it's chosen. Over the test scene, the in-between differs from the blend
in 6% of the pixels, crisp in 10 to 16%.

**David's second look** (build 113, the test world, 2026-11-11, about six minutes), on the in-between: "eh, i don't
prefer it. its better than ziggurat lods but, its not better than blended".

**The answer: blended, so the rebuild's bricks are soft.** The far cells are stored as 3D textures with a border, and
the card's texture hardware does the blend (about 5.5 times the cells, against 2.6 for plain bricks).
- Crisp and the in-between are both acceptable fallbacks, "better than ziggurat lods" (David), if memory ever forces
  one (a weaker card's), but neither is the look.
- Since the blend stays, its cost moves from the far march's code to the texture hardware. The probe P7's 41% is
  roughly what the hardware can win back, measured at build 107.

**For the record** (David's "yes", 2026-11-11: no look needed): flights F (crisp, build 113,
`kept logs/dyno-1794425205.txt`) against C, and G (the in-between, build 114, `kept logs/dyno-1794427769.txt`) against
C2 (build 114's own baseline, below). Sight on the card, median:

| | blended | crisp (F) | the in-between (G) |
|---|---|---|---|
| On the ground at the spawn, standing | 5.94 (C) / 5.88 (C2) | 4.32 (−27%) | 4.44 (−24%) |
| The mountain top, standing | 6.58 / 6.59 | 4.41 (−33%) | 4.57 (−31%) |
| Beside the quarter-block build, standing | 11.71 / 11.68 | 8.50 (−27%) | 8.36 (−28%) |
| The route, median / 99th | 4.06 / 9.69 | 3.12 / 6.46 (−23% / −33%) | 3.27 (−20%) |

So the far march's blend in code is a quarter to a third of the card at every mark. That's roughly the ceiling of what
the rebuild's soft bricks (the texture hardware blending) can win back while keeping the blended look. The hardware's
blend isn't quite free, so it's an upper bound.

*A flight lost:* G was first flown (2026-11-11, 19:28) straight after F by one script. The script ran `cargo run`
afresh, and the half-written 4b code then in the folder was built and died at start, on a binding count. Flown again
on build 114, committed.

---

## Experiment 4b: light kept in the cells, and looked up  *(plan 1.5)*

David's go: "yes" (2026-11-11).

**Why.** Experiment 4a: light (sky light and sun shadows) is 42% of the card at the spawn mark, 17% at the mountain top
and at the build, and a quarter of the route's median. It's found afresh by rays every pixel, every frame, though the
land hasn't moved and the light hasn't changed. David: "the big sun shadows don't need to change smoothly. they can
change every 30 seconds for all i care."

### What was built (build 114)

- **A light word beside every cell of the pool** (`CardStore::cell_light`, 4 bytes a cell, 96 MB beside the pool's
  384): the sun's step it was lit for, the sky the cell sees (12 bits) and the sun reaching it (12 bits). It's cleared
  wherever new cells go in (regions, upper columns, live tiles), so a reused place never shows old light.
- **The sweep** (`light_cells` in `march.wgsl`, a timed pass of its own, "light kept"). Each frame it goes over
  32,768 of the store's cell columns (some 9 million: live windows, then baked levels), lighting each surface cell
  not yet lit for the sun's step: its sun ray and its sky cones from its surface, as wide as the cell. Those are the
  rays a far pixel casts from where it meets that cell. A cell already lit costs one read. A whole sweep takes about a
  second and a half at 200 frames a second.
- **The sun's step:** a new one whenever the sun moves a quarter of a degree, or shadows or sky light are switched.
  Every cell is then lit again by the sweep, and rays light the pixels meanwhile.
- **The lookup** (`kept_light`, in the shading pass): a far surface pixel (not a block face near the eye, which has
  light per face) reads its cell's light in place of its own sun ray and five sky cones, where it's there for this
  step. Otherwise the pixel's own rays, as now.
- The Render page's switch "Light kept in the far cells", or `INCUS_SIGHT_LIGHT_KEPT=1`. Off unless chosen.
- A card test, `sight_s_light_kept_lights_like_the_rays`: after one sweep, the picture is nearly the rays' (it
  differs somewhere, by little on average, at the same brightness, with no black holes).

### The pass mark (written before the flight and the look)

**Measured:** flight H (`INCUS_SIGHT_LIGHT_KEPT=1`) against C, the route and the three marks, with the sweep's own
cost ("light kept") in every line.

**David's look:** far ground lit a cell at a time (light that steps), against rays, standing and flying; and the sun
moved, to see the relighting.

**What changes the plan** (the Oracle's):
- **The lookup wins most of 4a's ceiling for the far pixels:** the spawn mark comes down by a good part of light's
  42% (to 4.5 ms or under, against C's 5.94 and E's 3.46), the sweep costing little. Then the light lane is
  confirmed for the rebuild as the Oracle drew it.
- **It costs about as much as the rays:** the spawn under a tenth faster. Then the light moves into the hit record's
  read, or the brick layout is reconsidered before bricks are built.
- **The near faces are untouched** (light per face, each frame): what's left of light's share at the spawn after 4b
  is theirs, and says what keeping the near faces' light too would be worth.

### Results

**Flight C2** (build 114, 9d996b8, everything off; `kept logs/dyno-1794427178.txt`): the 4b code costs nothing switched
off. Against C (build 111): route 4.07 against 4.06, spawn 5.88 against 5.94, mountain top 6.59 against 6.58, build
11.68 against 11.71.

**Flight H** (`INCUS_SIGHT_LIGHT_KEPT=1`, build 114; `kept logs/dyno-1794428459.txt`), against C2:

| | C2 | H | change | light off (E, against C) |
|---|---|---|---|---|
| On the ground at the spawn, standing | 5.88 | 5.34 | −9% | −42% |
| The mountain top, standing | 6.59 | 6.37 | −3% | −17% |
| Beside the quarter-block build, standing | 11.68 | 11.29 | −3% | −17% |
| The route, median / 95th / 99th | 4.07 / 9.02 / 9.71 | 4.21 / 8.87 / 9.71 | +3% / −2% / 0% | −25% / −13% / −14% |

- **The sweep costs almost nothing:** "light kept" 0.00 to 0.01 ms a frame at its median.
- **The levels settle sooner** with it on: the build's in 59 s after the build went in, against 83 in C.
- **A repeat** (`kept logs/dyno-1794429425.txt`, the marks alone, with the light's view): 5.38, 6.38 and 11.30, the
  same as H.

**Why so little: the light's view** (`INCUS_SIGHT_LIGHT_VIEW`, the three marks' pictures in `kept logs/pictures/`):
- **Green (the light kept)** covers nearly all the far land at every mark, with a little yellow (sky handed down) and
  hardly any red (own rays). The lookup is found where it should be.
- **Blue (the near block faces, lit per face every frame)** covers most of the screen at all three marks: everything
  near the eye, which 4b doesn't touch.
- **The quarter-block build is yellow:** its pieces aren't in the face table, so each of their pixels casts its own
  sun ray.

So at these eye-level marks, most of light's cost (4a's 42% at the spawn) is in the near faces' light, worked out
again every frame.

**Against the pass mark:**
- The spawn only 9% faster, at the line between "wins most" and "costs as much as the rays". Neither reading fits:
  the lookup is cheap and found nearly everywhere far, but far pixels are a minority of these views, and their sky
  was mostly handed down already.
- So the plan isn't changed by "the lookup costs as much as the rays". The far half of the light lane works as the
  Oracle drew it, for the far view.
- **What's left is the near faces.** Keeping their light between frames, so the face table is filled once for the sun
  and reused, is where most of light's share at eye level is. It was tried at build 107 ("light per face kept",
  dropped: no gain over the moving route). Standing marks would judge it better. This goes to the Oracle at the fork.

**David's look at light that steps** (build 115, the test world, 2026-11-11, a couple of minutes: the switch on and off
over far land, and the sun moved): "honestly the difference is moot. it obviously changes a little, but i don't think i
could tell the difference between the two in a blind test".

**Experiment 4b's answer:** light kept a cell at a time is the look for the far view: David can't tell it from the
rays. The sun's relighting over a second and a half, rays meanwhile, didn't show as a problem. Keeping it is nearly
free and found nearly everywhere far. At eye level the gain is small, because the near faces' light is most of what's
left. The Oracle's 4.2 already has the near faces read their light from the air cell in front of them, tested in her
experiment 6; Counsel put the standing-marks question to her as her question 6.

---

## The reading flight: where the bare plain's cost lives  *(plan 1.7)*

David's go: "i agree on all counts. please ask builder to begin the next steps" (2026-11-11, relayed by Counsel), on
the Oracle's second part (`vision/oracle-review.md`, "Before the first line"). The Oracle found that the third mark is
mostly a bare plain: in flights C, E and F the build added about 2.2 ms to a place that already cost 9.5 with nothing
on it. Nobody knows which part of the frame that is.

### What flies (no code: old stays as tagged, `old-final`)

- **The bare plain at the build mark:** `INCUS_DYNO_BUILD="kept logs/events-1794341728 unplaced.log"`, David's build's
  169 placings with every block renamed to a name no block has. The mark stands where it always has (`dyno::marks`
  takes the spot and the facing from the placings) and places nothing. The report still calls the mark "beside the
  quarter-block build", and should say 169 placings replayed, 0 cells changed.
- **Six flights,** build 115's code (HEAD, 7c809d3's code unchanged). They run on the 5070 Ti in "NVIDIA GPU only"
  with the marks alone (`INCUS_DYNO_LAPS=0`), the test world and the dyno's presets:
  - **I:** as the game runs. This gives the plain's clean number.
  - **J:** light off (`INCUS_SIGHT_NO_SKY=1 INCUS_SIGHT_NO_SHADOWS=1`).
  - **K:** the blocks left out (`INCUS_SIGHT_NO_BLOCKS=1`): the near field drawn from cells of 2.
  - **L:** the finest layer 4 × 4 (`INCUS_SIGHT_GRID=4`), so the 2 × 2 layer's work goes.
  - **M:** the coarsest layer 4 × 4 (`INCUS_SIGHT_COARSEST=4`): the 8 × 8 layer's part drawn with four times the rays.
  - **N:** the steps' view (`INCUS_SIGHT_STEPS=1`): each mark's picture coloured by how many steps its rays took
    (blue few, green 128, red 256 or more). Its numbers aren't compared.

### The pass mark (written before the flights)

**The flight is good to read if** I's spawn and mountain top repeat C2's (5.88 and 6.59 ms standing) within 2%. If they
don't, the day's noise is written down first and the comparisons are read with it.

**The Oracle's guess** *[estimate, low]*: the coarse layers' long rays, over the sky and along the horizon. A ray going
up ends only when it's over the world's highest block (1,024). Builder's arithmetic:
- from the plain's eye at 137, a ray 1° up crosses about 50,800 blocks before it's clear;
- from the spawn's eye at 395, about 36,000;
- from the mountain top's eye at 1,025, none.

That ordering fits the marks' costs.

**What each result means:**
- **The guess holds if** N's picture at the plain is red or green over the sky near the horizon and blue on the near
  ground, and M lifts the plain by much more than it lifts the mountain top. The plain's cost is far rays over the sky.
  - In the new eye that is the sky the drawn boxes skip (2.2's second change). The plain's horizon goes in as one of
    the far eye's marks.
- **If K takes most of the plain's cost,** it's the near walk. The bare plain becomes one of the near eye's ground marks
  (2.3) and weighs more there.
- **If J takes most of it,** it's the near faces' light. The near light lane (the Oracle's second part, section 3) is
  where the plain is won.
- **If none of them takes most of it,** the cost is spread out. That is written down as it is, and the plain is flown in
  both eyes' marks.

Nothing in old changes on any result. The reading decides which eye's marks the plain belongs to and what each eye's
pass mark is measured on.

### Results

Six flights, build 115's code committed (f9e79bc), 2026-11-11, 19:37 to 20:29; the reports are in `kept logs/`. Every
header says the code was committed, and every flight stood the plain with "169 placings replayed, 0 cells changed".
Sight on the card, standing, median ms:

| | I, as the game runs | J, light off | K, no blocks | L, finest 4 × 4 | M, coarsest 4 × 4 |
|---|---|---|---|---|---|
| **The bare plain** | **9.66** | 7.63 (−21%) | 6.64 (−31%) | 7.74 (−20%) | 9.31 (−4%) |
| On the ground at the spawn | 5.96 | 3.46 (−42%) | 3.32 (−44%) | 4.43 (−26%) | 5.81 (−3%) |
| The mountain top | 6.62 | 5.47 (−17%) | 4.56 (−31%) | 5.33 (−19%) | 5.59 (−16%) |

The logs, in order: I `dyno-1794448108.txt`, J `dyno-1794448613.txt`, K `dyno-1794449077.txt`, L
`dyno-1794449309.txt`, M `dyno-1794449896.txt`, N `dyno-1794450518.txt`. N (the steps' view) measured the same as I:
9.66, 5.95 and 6.60.

- **The instruments repeat.** I's spawn read 5.96 against C2's 5.88 (+1.4%), and its mountain top 6.62 against 6.59
  (+0.5%), both within the 2%. One caveat: in I, J, M and N the spawn's live levels weren't whole after the 180 s cap
  (I: 7,533 of 8,820 tiles), so the spawn was measured nearly whole. That didn't happen to the plain: after its 180 s
  the dyno waited on until it was whole (68 s more in I).
- **The plain's clean number is 9.66 ms.** The Oracle's 9.51, from the settling lines, was right. The build adds 2.02
  (C2's 11.68), a sixth of the build mark.
- **The light was measured by switching it off (J), not by the split.** Old's clock times the whole compute pass as one
  ("passes: march, upscale"), so the split would have flown a different program shape without timing light alone.

**The steps' view** (`kept logs/pictures/reading N - …`):
- **At the plain** the sky is green, about 128 steps a ray, everywhere. It turns red, 256 or more, in a band above
  the open horizon about a fifth of the screen high. The near and middle ground is dark blue: few steps. The far land
  is a thin strip at the horizon.
- **At the spawn** mountains fill most of the horizon. The red is only where the horizon is open: the right-hand end
  and a strip on the left.

**Against the pass mark:**
- **The Oracle's guess fails its test.** The long rays over the sky are there: the red band. But in M the sky was
  drawn with four times as many rays (sky pixels are only ever marched in the coarsest layer: `worth_of`, "never for
  sky"), and the plain got 4% cheaper, not dearer. They are long but few (the 8 × 8 layer is 64,000 rays) and cheap.
  They aren't the plain's cost.
- **None of the switches takes most of it.** The cost is spread: the blocks 31%, light 21%, the finest layer 20%.
  These overlap: the blocks are drawn in the finest layer and lit per face.
- **Not placed:** with the blocks off, the plain still costs twice the spawn (6.64 against 3.32 ms); with light off, 7.63
  against 3.46. What differs between the two places is neither the sky nor the blocks nor light. Builder's leading
  candidate *[estimate, low]* is the far land seen grazing along a flat, open horizon the whole width of the screen;
  at the spawn, nearer mountains stop the rays. Old's switches can't split it further. They can't time a layer alone:
  one compute pass, one clock.
- **So by the pass mark, the plain is flown in both eyes' marks.** Its horizon is already one of 2.2's far marks, and the
  bare plain joins 2.3's ground marks. The new engine's clocks, a layer and a stage at a time, will place the rest.

**Found on the way:** the coarsest layer at 4 × 4 in place of 8 × 8 was cheaper at all three standing marks: 16% at the
mountain top, 3 to 4% at the plain and the spawn. That's despite marching four times the rays at that level. At these
views the 8 × 8 layer and its hand-over to the 4 × 4 cost more than they save. The route wasn't flown this way. For
the new eye, whose layers stand alone, the coarsest layer's size is a dial to measure, not assume.


## The pack's check flight: the same game from its new folder  *(plan 2.1)*

David's go: "go ahead and do the test, and pack away any intellij stuff with old incus" (2026-11-12). The pack
(daf4e2c) moved old Incus Artus into `old/` with its own build folder, worlds, bakes and logs. Its code changed in two
places only, so it builds from there: `old/Cargo.toml` is its own workspace, and `old/build.rs` finds git from below
the repository's top. Neither touches what the game does.

### What flies

One flight, I again, from `old\`: the marks alone (`INCUS_DYNO=baked INCUS_DYNO_LAPS=0`), the test world and the
dyno's presets, on the 5070 Ti in "NVIDIA GPU only". The bare plain stands at the build mark with
`"kept logs/events-1794341728 unplaced.log"` (old's own copy, `old/kept logs/`).

### The pass mark (written before the flight)

**The pack held if:**
- all three marks are within 2% of I (sight on the card, standing, median): the bare plain 9.66, the spawn 5.96, the
  mountain top 6.62 ms;
- the plain still says "169 placings replayed, 0 cells changed";
- the header names a commit after the pack with no changes; and the log has no "Path not found", so the shaders, the
  world, its bakes and the presets were all read from `old\`.

**If a mark is off by more than 2%,** it flies once more before anything is read into it: a single flight can miss by
that much (the spawn wasn't whole after the 180 s cap in I). **If it's off again,** something the move changed is being
read differently, and it's found before the new project starts.

### Results

One flight, 2026-11-12, 03:18 to 03:29, from `old\`: `kept logs/dyno-1794475773.txt`. The header says build 115
(18a9b5c), the code committed. Sight on the card, standing, median ms:

| | I (1.7, from the root) | The check (from `old\`) |
|---|---|---|
| The bare plain | 9.66 | 9.63 (−0.3%) |
| On the ground at the spawn | 5.96 | 5.94 (−0.3%) |
| The mountain top | 6.62 | 6.61 (−0.2%) |

- **The pack held, on every line of the pass mark.** All three marks are within 0.3% of I. The plain says "169
  placings replayed, 0 cells changed, 0 refused". The log has no "Path not found".
- **The same caveat as I:** the spawn's live levels weren't whole after the 180 s cap (7,527 of 8,820 tiles; I had
  7,533). The mountain top was whole in 102 s. The plain was whole 68 s after the build, as in I.
- Old is the same game in its new folder. The new project starts at the root.


## The empty frame: the new project's floor  *(plan 2.1's gate)*

David's go: "looks fine. go ahead" (2026-11-12, after his look at build 116, the skeleton). The first measure of the
new project: the frame with nothing in it. It's the floor that every later change is measured from.

### What flies

Build 116 (c4d102e), from the top folder: `INCUS_FLIGHT=1 cargo run --release -j 4`. It runs on the 5070 Ti in
"NVIDIA GPU only". The camera stands still, the frames settle for 5 s, then 20 s are measured. The frames are handed
over uncapped (Immediate) with the panel hidden. One pass on the card: the picture cleared, and the panel (empty) over
it. The report goes to `logs/flight-<seconds>.txt`.

### The pass mark (written before the flight)

**The flight is good to read if** the report's header names build 116 with the code committed, and the card's clock
timed the frames (thousands of them, every one with its pass).

**Builder's estimates** *[estimate, low: nothing like it has been measured here]*:
- the card's pass under 0.1 ms. Clearing a 2560 × 1600 picture is 16 MB written, a small fraction of a millisecond on
  this card.
- the frame under 1 ms at the median (above a thousand a second), since only the processor's round trip and the hand-over
  are in it.

**What each result means:**
- **Within the estimates:** the floor is written down, and the skeleton's line in 2.1 is ticked.
- **The card over 0.1 ms, or the frame over 1 ms:** it's still the floor, written down as it is. Before anything is built
  on it, the time is placed: in the hand-over, the panel's library, or the clocks themselves. An empty frame that costs
  something costs it under every frame after.

### Results

One flight, 2026-11-12, from the top folder: `kept logs/flight-1794486371.txt`. The header says build 116 (182d9e2:
c4d102e's code, the commit after it holding only the pass mark), the code committed. 20 s measured:

| | median | 95th | 99th | 99.9th | worst |
|---|---|---|---|---|---|
| The frame (ms) | 0.404 | 0.576 | 0.748 | 0.952 | 2.271 |
| The card's pass, cleared with the panel over it (ms) | 0.019 | 0.023 | 0.042 | | 0.242 |

- **Good to read:** 52,879 frames measured, all 52,879 timed on the card.
- **Within both estimates:** the card's pass at a fifth of the 0.1 ms, and the frame at 0.40 ms (2,473 a second), under the
  1 ms. Nearly all of the frame is the processor's round trip and the hand-over: the card is busy for a twentieth of it.
- **The floor:** 0.02 ms on the card and 0.4 ms a frame, with nothing in the world. Every later change is measured from
  here.
- **Found on the way:** the report named the build's own stamp (`INCUS_BUILD`, `INCUS_BUILD_CHANGED`,
  `INCUS_BUILD_NUMBER`) among the switches: `cargo run` hands the build script's values to the game as it starts. They
  aren't switches, and are left out from the next build on.


## The wiring's first flight: build 117 against the floor  *(plan 2.1)*

David's go: "editor looks about like what i'd expect. you can go ahead with your tests" (2026-11-12, after his look at
build 117). Build 117 runs the frame from the wiring (`src/wiring.rs`): its passes in the list's order, each timed on
the processor as well as on the card. The flight puts each pass's measured cost on the map (`master/map.md`, drawn
from the newest kept flight).

### What flies

Build 117 (8283179), from the top folder, as the empty frame flew: `INCUS_FLIGHT=1`, the camera still, 5 s settling,
20 s measured uncapped with the panel hidden and the editor closed, on the 5070 Ti in "NVIDIA GPU only".

### The pass mark (written before the flight)

**The flight is good to read if** the header names build 117 with the code committed, and every pass in the frame
has its own line with times. "Hearing the window" can read "not run": nothing is pressed during a flight.

**What changed since the floor (build 116: 0.404 ms a frame, 0.019 on the card):**
- the frame's passes run as a list, each timed on the processor (two clock readings a pass);
- the panels' textures go to the card while they're laid out;
- the panels' figures aren't worked out while nothing shows them, so a flight doesn't carry them.

**Builder's estimate** *[estimate, medium]*: the same frame. The card's pass within 0.005 ms of 0.019, and the frame within
10% of 0.404 ms. A frame this short moves by more than its tenth of a millisecond from flight to flight, and one flight
of it hasn't been repeated yet.

**What each result means:**
- **Within it:** the wiring costs the frame nothing that can be measured. The two plan lines (the wiring, and the
  software map) are ticked, with the map's costs filled from this flight.
- **Over it:** the frame's passes now each have a time of their own, so the new cost is placed pass by pass before
  anything more is built on it. If no pass holds it, it's the flight's noise: flown once more, and both written down.

### Results

One flight, 2026-11-12, from the top folder: `kept logs/flight-1794487642.txt`. The header says build 117 (a2a8cdb:
8283179's code, the commit after it holding only the pass mark), the code committed. 20 s measured, 52,207 frames,
every one timed on the card.

| | median | 95th | 99th | worst |
|---|---|---|---|---|
| The frame (ms) | 0.420 | 0.562 | 0.735 | 3.251 |
| The card's pass (ms) | 0.018 | 0.022 | 0.041 | 0.294 |

Each pass on the processor, median ms: flying 0.000, laying out the panel 0.009, getting the picture 0.005, the panel
(recording it) 0.004, **handing over 0.310**, measuring 0.000. Hearing the window: not run, as expected.

- **Good to read, and within the estimate:** the frame 4% over the floor's 0.404 (inside the 10%), the card's pass
  0.018 against 0.019. The wiring costs the frame nothing that can be measured here.
- **Where the empty frame's time is:** handing over, 0.31 of its 0.42 ms. That's the frame's work handed to the card and the
  picture to the screen, waited for on the processor. About 0.09 ms falls outside every pass: the window's own round
  trip between frames, which no pass holds yet.
- **The map's costs** are filled from this flight (`master/map.md`).


## Handing over, split in two: where the empty frame waits  *(plan 2.1)*

David: "go ahead and split it, so later on we can see it and tune it if it ever becomes something that can be
improved" (2026-11-12). Build 117's flight put 0.31 of the empty frame's 0.42 ms in "handing over". Build 118 splits
it into old's two names for its parts: **handing the card its work** (the frame's work finished and handed to the card)
and **showing it** (the picture handed to the screen). The card's times are now read back in "measuring", which
already said it did that.

### What flies

Build 118, from the top folder, as before: `INCUS_FLIGHT=1`, 20 s measured uncapped, the panel hidden and the editor
closed, on the 5070 Ti in "NVIDIA GPU only".

### The pass mark (written before the flight)

**Good to read if** the header names build 118 with the code committed, and both new passes have their lines.

**Builder's guess** *[estimate, medium]*: most of the 0.31 ms is in **showing it**. Windows lets the processor run only a
frame or two ahead of what the card and the screen have taken, and with an empty frame it gets there almost at once.
Handing the card its work should be well under 0.1 ms. The frame should stay within 10% of 0.420 ms.

**What each result means:**
- **Showing it holds most of it:** it's the hand-over's own pace, the speed limit of handing pictures to the screen. Nothing
  to tune while the frame is empty. It's the place to look when the frame's lead over the screen is tuned (smoothness
  against lag), with David's eye, once the world is drawn.
- **Handing the card its work holds most of it:** the processor is working, not waiting, in finishing and handing over
  the card's work. That's worth a look before the eye adds its passes to it.
- **Split between them, or the frame moved by more than 10%:** written down as it is, and flown once more before it's
  read into.

### Results

Two flights, 2026-11-12, build 118 (29a02df), the code committed: `kept logs/flight-1794487928.txt` and
`kept logs/flight-1794487963.txt` (the second flown because the first came out split, as the pass mark said). Each
measured 20 s, about 52,000 frames, every one timed on the card. On the processor, median ms (95th):

| | first | second |
|---|---|---|
| The frame | 0.418 (0.561) | 0.417 (0.557) |
| handing the card its work | 0.078 (0.241) | 0.077 (0.239) |
| showing it | 0.142 (0.271) | 0.142 (0.271) |
| measuring (now with the card's times read back) | 0.009 | 0.009 |
| the panel, on the card | 0.018 | 0.018 |

- **The two flights repeat** to within 1%, and the frame is where build 117 left it (0.420).
- **Builder's guess held only in part.** Showing it is the larger part, about two thirds of the hand-over. But handing the
  card its work holds the other third, 0.08 ms. That's under the 0.1 guessed, but not "well under".
- **Showing it** (0.14 ms) is the hand-over's own pace: the picture waits its turn for the screen. Nothing to tune while
  the frame is empty. It's the place to look when the frame's lead over the screen is set, with David's eye, once the world
  is drawn.
- **Handing the card its work** (0.08 ms) is likely the processor working, not waiting *[estimate, medium]*: the
  library checking and translating the frame's work for DirectX before it goes. It's the one to watch as the eye adds
  its passes, since that work grows with what's handed over. Nothing is changed for it now; the map shows it.
- **Not read into:** a single pass's median can't be added to another's, so these don't say the split saved or cost
  anything; the frame says it didn't. Several passes' 95th sit near 0.16 ms in turn (getting the picture, recording the
  panel, laying it out), as if one interruption of about that length lands in whichever pass is running. That's to watch, not
  to chase on an empty frame.


## The doorway: the test world answered column by column  *(plan 2.1's gate)*

David's go: "go ahead" (2026-11-12). The doorway is the one piece that answers "what's here?" a tile at a time. The
engine never opens a world file itself. Today the export is behind it (`exports/nice25 test`, old's test world as
columns, read-only); the tablet comes at 3.1.

### What's built first

Columns as runs of blocks, a tile at a time: each column's ground, its river's or lake's water, and its blocks from
bedrock up (stone, the layers under the top, the top block, then water to the sea or the river's top), by the export's
own rule (`exports/nice25 test/README.txt`).

Nothing feeds the doorway's other answers yet: pieces, objects and lights, what changed, and the named fields over the
land. They come with what first feeds them; David's build arrives as changes with the marks, next. Nothing in the game
asks the doorway yet either: its tests do, and the far cells' lane will (2.2). So nothing on the screen changes, and no
flight is flown.

### The pass mark (written before the code)

**The gate passes if** the doorway answers all 5,047 sample columns (`references/columns.txt`, old's `CompiledMap::column`)
to the block:
- the ground's Z the same;
- the river's or lake's water top the same (none where old said none);
- "past the map" where old said so (one column).

And each column's runs keep the export's rule: stone from bedrock, the top block at the ground's Z, and water from just
above it to the sea or the water's top, whichever is higher.

**Builder's estimate** *[estimate, low]*: a tile (512 × 512 columns) answered in under 10 ms on the processor, and the
5,047 columns, nearly one tile each, in under a minute. Measured by the test, written down beside the gate.

### Results

Run by its test, 2026-11-12 (`cargo test --release`, `doorway::tests::the_doorway_answers_old_s_sample_columns`):

- **The gate passes:** all 5,047 sample columns answered as old's, the ground and the river's or lake's water to the
  block (18 of them with such water). Every column's runs keep the export's rule.
- **Corrected in the open:** the pass mark said one column was "past the map". There are none: the one line Builder
  counted was the file's own header, which names "past the map" as a possible answer. Nothing past the map was checked.
- **The time** *(against the estimate)*: a tile answered in 2.2 ms at the median, 5.5 to 6.3 ms the slowest (under the
  10 ms guessed), and the 5,047 columns, a tile each, in 11.5 to 11.9 s (under the minute).
- **Found on the way, by the doorway's own small test:** the layers under a column's top block came out shallowest
  first, not from bedrock up. The test world's profiles have no such layers, so old's columns couldn't have caught it.
  Fixed before the commit: each layer is now laid deepest first.
- **On the map:** a third lane, "loading" ("as tiles are asked for"), holds the doorway. It reads "the export" and
  writes "a tile's columns". Nothing in the game asks it yet. These names are Builder's, David's to change.

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


## The marks as data  *(plan 2.1)*

David's go: "go ahead" (2026-11-12). The marks are the places every flight stands at, the same each time. They're now
data, `data/marks.toml`, not code: each has a name, a place, how high over a player's eye, a facing and pitch, whether
fog is on, which eye's marks it is (2.2's far eye, 2.3's near eye), and what's changed there first. Old worked its
marks out from the map; the new file holds the places old flew, from its reports.

Each mark's eye is placed from the doorway's answer, a player's eye (1.38) over the ground or the water, so the marks
also put the doorway to use in the game. The build's placings and the scatter's blocks are made as changes through
the doorway ("what changed"). Nothing draws the world yet, so none of this shows on the screen; the flight stands at
each mark in turn, and the game starts at the first, the spawn.

The marks, in the file's order:
- **on the ground at the spawn** (near);
- **the mountain top on a clear day** (near);
- **the bare plain** (near and far: it's the plan's "plain's horizon" for the far eye);
- **the plain with David's build** (near): the 169 placings of `kept logs/events-1794341728.log`, made as changes;
- **the scatter** (near): random blocks round the bare plain, at three settings of the dial;
- **high over the spawn** and **low over the sea** (far).

### The pass mark (written before the code)

**The marks are right if,** by their test, each eye stands where old's did, to a tenth of a block. That's old's
reports (`kept logs/dyno-1794448108.txt`, `kept logs/dyno-1794433359.txt`):
- the spawn at 395.4 over (0.5, 0.5);
- the mountain top at 1,025.4 over (−605.5, 78.5);
- the plain at 137.4 over (−3,975, −1,192);
- high over the spawn at 795.4;
- low over the sea at 151.4 over (−1,905.7, −2,137.7).

The facings must be old's: 0°, 172.6°, 180°, 0°, 228.3°.

**And** the build is all 169 placings made as changes, none refused: 146 bars and 23 whole blocks. Each scatter setting
makes exactly its count of blocks.

**The flight** stands at every mark in turn, a section of the report each. *Builder's estimate [medium]:* every mark's frame
is within 10% of the empty frame's 0.42 ms. Nothing is drawn yet, so the places can't differ. A mark that differs points
at the marks' own work (going to a mark, the doorway, the changes), which the passes' times will show.

### Results, first flight

Build 119 (b48ddc3), the code committed, 2026-11-12: `kept logs/flight-1794489087.txt`. Nine standings, 25 s each
(5 settling, 20 measured), in the file's order.

- **The marks are right, by their test and in the flight:** every eye stood where old's did (395.4, 1,025.4, 137.4,
  795.4, 151.4), facing as old's. The build was made as 23 whole blocks and 146 bars, none refused, and each scatter
  setting made its count (1,000, 10,000, 100,000), none refused.
- **The estimate missed, and in the other direction.** The spawn, flown first, read 0.419 ms, the floor as before. Every
  later mark read 0.336 to 0.341 ms, about 20% faster:

  | | spawn (first) | mountain top | bare plain | build | scatter 1k / 10k / 100k | high over spawn | low over sea |
  |---|---|---|---|---|---|---|---|
  | the frame, median ms | 0.419 | 0.337 | 0.340 | 0.341 | 0.339 / 0.338 / 0.339 | 0.338 | 0.336 |

- **It isn't the marks.** Nothing is drawn yet. Going to a mark costs 0.000 at the median (its work is done while the
  frames settle, before measuring). The scatter at 100,000 blocks reads the same as the bare plain.
- **It's the time since the game started.** After the first standing, the card's own pass fell too, 0.018 to 0.011 ms,
  and the slow frames thinned on the processor (recording the panel's 95th, 0.158 to 0.007 ms). So for about its first 25
  seconds the whole session runs slower. *[Estimate, low:* the card's clocks coming up from idle under so light a load,
  and Windows settling the new full-screen window.*]*
- **So the floor needs correcting.** Builds 116 to 118 measured only seconds 5 to 25 after starting, inside that warm-up.
  Their 0.40 to 0.42 ms is the warmed-up-too-little frame; the settled empty frame is about 0.34 ms, 0.011 on the card.
  The comparisons between those builds stand (each flew the same way); the floor's number doesn't.
- **Next:** the first standing settles longer, and the report says what going to each mark cost (its changes, and the
  doorway's tiles), which this flight didn't show. Flown again to see the spawn read as the others do.

### The second flight's pass mark (written before it flies)

Build 120: the first mark settles 30 s more (35 s in all), and each standing's report says what going there cost.

- **The warm-up is the cause if** the spawn, still flown first, now reads within 5% of the other marks' median (about
  0.34 ms), and its card pass about 0.011 ms like theirs. If the spawn is still slow, it's the first standing itself,
  not the time since the start, and that's looked into before the marks are ticked.
- **The marks, as before:** within 10% of each other, every eye and every change as the first flight.
- **Going there** *[estimate, low]*: a few milliseconds for a plain mark (one tile, 2 ms); the scatter at 100,000 in
  under a second (its four tiles and a hundred thousand changes). All of it while the frames settle, not measured.

### Results, second flight

Build 120 (2ee6651), the code committed, 2026-11-12: `kept logs/flight-1794489536.txt`. The first mark settled 35 s.

| | spawn (first) | mountain top | bare plain | build | scatter 1k / 10k / 100k | high over spawn | low over sea |
|---|---|---|---|---|---|---|---|
| the frame, median ms | 0.338 | 0.340 | 0.338 | 0.344 | 0.340 / 0.341 / 0.338 | 0.340 | 0.338 |
| going there, ms | 3.2 | 3.0 | 2.6 | 3.2 | 9.4 / 10.1 / 18.7 | 3.7 | 3.2 |

- **The warm-up was the cause.** The spawn, still first, now reads 0.338 ms, within 1% of the rest, and its card pass
  0.012 like theirs. All nine standings fall within 2% of each other (0.338 to 0.344), and the card's pass is 0.012 at each.
- **The marks, as before:** every eye and every change the same as the first flight.
- **Going there,** inside the estimate: about 3 ms for a mark (the doorway reading one tile, 2.5 to 3.4 ms of it). The
  scatter reads three tiles; at 100,000 blocks it takes 18.7 ms, well under the second guessed.
- **The settled floor:** 0.34 ms a frame, 0.012 on the card, with nothing in the world. Corrected in the plan beside
  build 116's line.
- **The marks' line in 2.1 is ticked.** The plain's horizon is the bare plain, marked for both eyes (Builder's reading,
  David's to change). One command flies every mark on one card. Both cards at once waits on the Optimus setting, when
  the Intel card is flown.


## The far cells, summed through the doorway  *(plan 2.2, step 1)*

David's go: "the far colors are not important now. i'll go in and regulate everything again once we rebuild the
registrar" (2026-11-12; carry old's), and "yes, i think we'd be right to use the processor for summing the far cells.
its a 1 time event, and it's quick." The far eye can't draw until its cells exist, so the first step makes them, with no
picture yet.

### What's built

`src/far.rs` sums the world into far cells from the doorway's columns, using old's cell unchanged (`old/src/sight/cell.rs`):
- cubes of 32 blocks, and every level above, each exactly its children added;
- the compact 16-byte form, and only each column's surface band.

The faces follow old's card rule (`cell.wgsl`): below the world is stone, past the map is air.

It's summed on the processor a region (1,024 blocks a side) at a time, on every thread, nearest the spawn first. A
column's solid blocks are runs, so its faces against each neighbour are added a range at a time, where old went block
by block on the card.

The far colours (`data/far colours.toml`) are old's, worked out from old's textures by old's rule. No colour came within
1% of a rounding step, so they match old's to the 1,024th.

### The pass mark (agreed with Counsel before the code)

- **Every far cell at every level** (32 to 1,024) is the same as old's bake (`old/worlds/nice25 test/sight-bake.bin`,
  209 MB), byte for byte in its compact form. That covers the cells left out of either band, over the whole world, and
  each region's 1,024 to its exact sums. If only colours differ, the colours are found first; anything else that
  differs is placed before step 2.
- **The first sum's time,** measured. *Builder's estimate [low]:* the eighth round the spawn under 15 s, the whole world
  under two minutes. Old's bake took 46 to 55 s for its start (the eighth) and about 12 minutes for the whole world, on
  the card (decisions, 2026-11-06).

### Results

Run by its test, 2026-11-12 (`cargo test --release`, `far::tests::the_far_cells_are_old_s_bake`), on all 24 of the
processor's threads:

- **The pass mark holds: every far cell is old's.** All 5,184 regions of old's bake and the 11,231,031 cells kept from
  32 to 1,024 are the same as old's, byte for byte, the cells left out of either band included. Each region's 1,024
  matches old's to its exact sums. The far colours carried from old's textures are old's to the 1,024th.
- **The check bites:** with stone's side colour one step redder (89 in place of 88 of 1,023), 144,659 cells differed
  and the test failed, naming the first. The colour was put back.
- **The first sum's time:**

  | | the eighth round the spawn | the whole world |
  |---|---|---|
  | Builder's estimate | under 15 s | under 2 min |
  | **The new far cells** (processor, 24 threads) | **2.6 s** (2.5 again) | **16.9 s** (17.2 again) |
  | Old's bake (the card, decisions 2026-11-06) | 46 to 55 s | about 12 min |

  The levels above 1,024, up to four cells of 65,536 covering the map, take under a hundredth of a second more.
- **Why it's this much quicker** *[estimate, medium]:* old summed block by block on the card, in small pieces so as
  not to hold the frame. Here each column's solid blocks are runs, and its faces against a neighbour are one range,
  added in one step, on every thread.
- **On the map:** the loading lane holds a second pass, "summing the far cells", reading "a tile's columns" and writing
  "the far cells" (Builder's names, David's to change). Its card says why it's on the processor. Nothing in the game
  asks for the far cells yet: the far eye will, in step 2.
- *Corrected (Counsel, the same day):* the "why" above is only part of it. Old's bake also made the ground itself,
  running the world's recipe on the card for every column ("only the cells of 32 run the recipe, on the card",
  `old/src/sight/bake.rs`). The new sum reads ground the export already made: that took 8 minutes, once. So 46–55 s against
  2.6 s is both, the recipe left out as well as runs against blocks. What matters for the player stands: in the new design
  the world is made once, and its levels are summed at load in seconds.


## The far eye, 2a: old's far march on bricks, the shape  *(plan 2.2, step 2)*

David's go: "Yes, go ahead" (2026-11-12, through Counsel): the two looks (2a the shape, 2b old's light), the two marks
(high over the spawn, low over the sea), and `INCUS_MARK` as the way there.

### What's built

Old's far march (`old/src/sight/march.wgsl`, its far part), moved onto bricks with its logic as it was:
- each ray reads the level whose cells are about its pixel's size there;
- it skips space a coarser cell column says is empty (over its band's top), and regions not summed yet;
- elsewhere it steps through the trilinear blend of the eight cells round each point, gathering front to back from
  both ends of each step, walking in from the last air;
- the last fifth of each level's stretch is blended into the next (old's hand-over from 0.8);
- it's lit by each cell's faces as they look toward the sun (3.4a), under old's sky and fog;
- old's screen pyramid: the start pass over 8 × 8 tiles; layers of 8 × 8, 4 × 4 and 2 × 2, a finer one only where it's
  earned and never for sky; then brought up to the screen crisp.

The settings are old's for the dyno: field of view 100°, the sun from 315° at 40°, noon, fog on.

What moves is where the cells live:
- each level is cut into bricks of 8 × 8 × 8 cells, where the levels are that big; above cells of 128 a brick stays
  within its region;
- a cell is found by arithmetic: its column's band (its lowest cell and how many), then the brick's place in the
  level's index, then the cell's place in the brick.

The bricks are plain buffers, blended in the shader as old did.

The far cells are summed in the game now, on the loading lane, nearest the spawn first. Each region goes onto the card
as it comes, so the view sharpens in live.

Left for 2b: the sun's shadows and the sky light. Without them, 2a looks flatter than old. Left for the near eye: blocks,
pieces, placed light and textures.

### The pass mark (written before the code)

- **A card test:** cells read through the bricks on the card, at every level, equal the far cells (step 1's) in their
  compact form. That includes the cells under a band (solid), over it (empty), and in regions not summed (unknown).
- **David's eye,** at the two marks: the shape of the far land, and the rings where levels hand over, flying forward
  low over the sea.
- **Measured, at the far marks:** each pass on the card (the start, each layer, bringing it up) and the processor's
  passes, and the bricks' memory. No speed bar here; the gate is after the three changes.
- **Builder's estimate** *[medium]:* the bricks for the whole world take 0.9 to 1.3 GB on the card. Mostly the cells of
  32: a brick is 8 cells tall where a band is 1 or 2. The Oracle's 250 to 450 MB was for windows round the eye, not the
  whole world. Both cards allow a buffer of up to 2 GB.

### Results: David's first look, and the flight

**Build 121 crashed** 18 s into David's first look, high over the spawn. At the levels over 1,024, a row's last brick reaches
past the level's columns, and its bands were written into the next row, and at the last row past the end of the
bands. The card refused it. Build 122 stops each row at the level's last column, with a test on every level's edge
bricks (the card test had covered only the middle). Build 122 ran whole: the eighth nearest in 2.7 s, every region in
17.2 s, 127,606 bricks, 911 MB on the card.

**David's look** (high over the spawn, 2026-11-12): "there's a little bit of blotchiness ... it looks right, mostly, and
if performance is keeping up with what we'll need, we can proceed". Two things he named:
- **The blotches.** The flight below shows why. From the far cells alone, the finest layer (2 × 2) is almost never earned:
  old's rule refines only where a pixel would be bigger than the cells it reads, and the far cells are 32 blocks and up.
  So most of the screen is 8 × 8 and 4 × 4 squares shown crisp, each a slightly different average.
- **Old's skyline:** coarse squares that are part mountain and part sky show as darker squares beside the slopes. David
  wants the sky weighted more in the math, a natural feather "that we can forget about"; it's built next, with a slider.

**The flight** (`kept logs/flight-1794495630.txt`, build 122, the code committed). The first mark was held till every
region was in, then settled 35 s; 20 s measured at each mark. The far eye on the card, median ms (99th):

| | the far start | 8 × 8 | 4 × 4 | 2 × 2 | bringing it up | **the card's whole** | the frame |
|---|---|---|---|---|---|---|---|
| on the ground at the spawn | 0.004 | 0.316 | 0.007 | 0.023 | 0.093 | **0.452** (0.464) | 0.564 |
| the mountain top, clear day | 0.006 | 1.323 | 1.198 | 0.023 | 0.094 | **2.792** (3.465) | 2.791 |
| the bare plain (and the build, the scatter: the same) | 0.005 | 1.081 | 0.007 | 0.023 | 0.093 | **1.219** (1.481) | 1.257 |
| high over the spawn | 0.005 | 0.957 | 1.072 | 0.023 | 0.096 | **2.267** (3.147) | 2.308 |
| low over the sea | 0.004 | 1.766 | 1.903 | 0.023 | 0.093 | **3.950** (4.282) | 3.967 |

- **The 2 × 2 is almost never marched:** 0.023 ms at every mark is its pixels finding they're not worth it. The 4 × 4 is
  worth it only looking out from high or along the sea, where it costs as much as the base.
- **Against 2.2's gate** (the far layers under 1 ms typical, 2 worst, at the far marks), old's march as it is reads 1.2 to
  4.0 ms at the far marks: 2 to 4 times it, low over the sea the most. That's the baseline the plan's three changes are
  measured against (soft bricks, rays begun at drawn boxes, light read from the cell). There's no light in it yet, which
  2b adds.
- **The build and the scatter cost nothing yet:** the far cells don't hold changes (the near eye's change path is 2.3).

### David's look, builds 123 to 125: the sky's weight, the crossovers, low over the sea

- **The sky's weight** (build 123, on the Render page with the crossovers, the card time beside them, and Default). A plain
  power washed the far mountains out: "love what the weight does to the edges, hate what it does to the mountain *not*
  on the edges". Build 124 fades the power as the land covers more: "that actually works great". At the slider's top, "i
  maxed it": 4 is now the default (build 125), and the slider runs to 8.
- **The crossovers:** "honestly none of the crossover sliders do much". From far cells alone the picture is soft
  everywhere, and the finer layers are rarely earned, so moving where they fade in changes little. They're kept for when
  the near eye brings small cells in.
- **Low over the sea:** looking up, mostly sky, 1.3 ms; along the horizon, 4.2 ms: "yeah it goes up if i look around. anyways
  just noting it." He named no ring as a seam.
- **Next: 2b,** old's light.


## The far eye, 2b: old's light  *(plan 2.2, step 2)*

David's go for the two looks (2a the shape, 2b old's light), 2026-11-12; then "whats next" after the first.

### What's built

Old's light for the far cells (`old/src/sight/march.wgsl`, 3.4b), as it was:
- **The sun's shadow:** a ray from the surface towards the sun through the same far cells, widening as it goes (0.04 of a
  block for each block), reading the level as wide as it is, skipping air over the columns' tops, dimmed by what it
  crosses. A far ridge's shadow comes soft.
- **Sky light:** five wide cones (straight up, and four leaning 50° out), each weighted by how high it looks, say how much
  of the sky a place sees, so valleys and hollows darken. A finer layer takes its parent pixel's sky light where it's the
  same surface about as far off, as old did.
- Both only where a pixel shows. Old's switches for measuring: `INCUS_SIGHT_NO_SHADOWS`, `INCUS_SIGHT_NO_SKY`.

With 2b, old's far view is whole on bricks: the baseline the plan's three changes are each measured against.

### The pass mark (written before the code)

- **The card refuses nothing,** and an off-screen picture from the spawn shows the land shaded, not flat, before David's
  look.
- **David's eye** on the light, high over the spawn first.
- **The flight:** every far pass on the card at every mark, beside 2a's. *Builder's estimate [low]:* light adds a fifth to a
  half to the far eye at the far marks (old's light was 17 to 42% of its eye at the three standing marks), which puts
  them about 1.5 to 6 ms. That's the baseline, with no bar of its own in this step.

### David's look, and the flights' pass mark

David on build 126, high over the spawn: "light looks normal. hard to tell as it is, but ive never had an issue with your sun
lights looks, so proceed".

**Four flights, written before they fly** (build 126, every mark, the first held till every region is in). Counsel's ask,
for the size of the third change's target (light read from the cell):
- **W:** whole, as the game runs;
- **S:** without the sun's shadows (`INCUS_SIGHT_NO_SHADOWS=1`);
- **K:** without the sky light (`INCUS_SIGHT_NO_SKY=1`);
- **N:** with neither: the shape alone, against 2a's flight (build 122) as a check that 2a's numbers still hold with the sky's
  weight in.

The light's cost is W against N, at each far mark, and its two parts are W against S and W against K. The estimate in the
pass mark above stands: light a fifth to a half of the far eye.

### Results: the four flights

Build 126, 2026-11-12, 09:25 to 09:44, one after another: W `kept logs/flight-1794497402.txt`, S `-1794497681`, K
`-1794497955`, N `-1794498228`. Counsel committed the team kit while they flew, so S, K and N carry a newer commit
(740c882); the code is the same, and every header says the code was committed. The far eye on the card (its whole, all
its passes), median ms:

| | W, whole | S, no shadows | K, no sky light | N, neither | **light, W − N** | shadows, W − S | sky light, W − K |
|---|---|---|---|---|---|---|---|
| on the ground at the spawn | 0.771 | 0.633 | 0.536 | 0.455 | **0.316 (41%)** | 0.138 | 0.235 |
| the bare plain (the build, the scatter: the same) | 1.415 | 1.382 | 1.259 | 1.224 | **0.191 (13%)** | 0.033 | 0.156 |
| high over the spawn | 2.375 | 2.274 | 2.179 | 2.144 | **0.231 (10%)** | 0.101 | 0.196 |
| the mountain top, clear day | 2.737 | 2.727 | 2.688 | 2.665 | **0.072 (3%)** | 0.010 | 0.049 |
| low over the sea | 3.953 | 3.910 | 3.856 | 3.797 | **0.156 (4%)** | 0.043 | 0.097 |

- **The light is small in the far eye:** 3 to 13% at the far marks, 0.07 to 0.23 ms. That's under Builder's guess of a fifth to a
  half, and under old's 17 to 42%. Old's light was largely the near blocks' faces lit at the finest layer. Here light is only
  worked out where a pixel shows, and nearly every far pixel is an 8 × 8 or a 4 × 4 square.
- **The parts overlap:** shadows and sky light each cost more alone than their share of the two together. The sky light is the
  larger at every mark.
- **So the third change (light read from the cell) has little to win in the far eye,** 0.2 ms at most at the far marks. The
  far eye's cost is the march itself, the 8 × 8 and 4 × 4 layers (high over the spawn 0.96 and 1.07 ms, low over the sea
  1.77 and 1.90). The first two changes are aimed at the march: the eight-cell blend done by the texture hardware, and rays
  begun at the bricks' boxes.
- **N, against 2a (build 122):** the same at the spawn and the plain (0.455 against 0.452, 1.224 against 1.219), and 4 to 5%
  faster at the far marks (high over the spawn 2.144 against 2.267, low over the sea 3.797 against 3.950). Nothing in the
  march's own code changed between them. The likeliest cause is two sessions' noise on the long rays *[estimate, low]*.
  2a's numbers hold within 5%.
- **The baseline,** old's far view whole on bricks (W): the far marks at 1.4 to 4.0 ms, 1.4 to 4 times the gate's typical
  1 ms. The map's costs now come from the newest flight flown as the game runs (W), not from N.


## Change 1: soft bricks blended by the texture hardware  *(plan 2.2, experiment 5's first change)*

David: "honestly, i think we should try all these things and gather data about the performance impacts they have, and let
me take a look at how they look while flying and while still/moving my head" (2026-11-12). Memory is a bar too: the
whole game's card memory "well below" 4 GB at worst. Each step is weighed against it, the near eye and light included.

### What's built

- **The bricks also kept as 3D textures** in atlases on the card, each brick with a one-cell border on its high sides,
  holding the next cells over, so the hardware's filter blends across a brick's edge. A brick's cells under its band
  are kept solid and over it empty, as the bands say.
- **When a region comes in,** its bricks are made with borders from whichever neighbours are in, and the bricks of the
  neighbours below it in north, east and both get their borders filled from it. Rays never read a border not filled
  yet: the march reads the four columns' bands first, as old's blend did, and a column not summed is unknown.
- **The blend:** where all four columns are in and the brick is on the card, one fetch for how solid and how watery (at
  every step), then the colour and the faces' fetches where it's covered. Where not (a column outside the bricks, a
  brick not kept), the code's blend as before, so nothing is ever read wrong.
- **Two ways, both on the card at once for David's look, one at a time for the flights** (`INCUS_FAR_BLEND` = `code`,
  `exact`, `spread`, or `both` for the look), switched live on the Render page:
  - **A, exact:** colour and the six faces' shares stored multiplied by the surface, with the surface itself, as 16-bit
    floats. The hardware's filter sums them, and one divide gives old's blend.
  - **B, spread:** colour and faces spread into empty neighbours when the brick is made, filtered unweighted, in 8 bits.
  - Solid and watery are true and unspread in both (16-bit).
- The hand-over between levels stays as old's.

### The pass mark (written before the code)

- **A card test,** at points spread through the regions round the spawn, each of the eight cells' blends against the
  code's. **A** within 16-bit rounding: every value within 1/1,000 of the code's. **B:** the largest difference reported, and
  where, with no bar (it's meant to differ by a hair).
- **Pictures** off the screen, from high over the spawn and low over the sea, A and B beside the code's, before David
  sees them.
- **Flights:** code, A and B, one at a time at every mark, each far pass on the card, and the card's memory.
- **Builder's estimate** *[low]*: the 8 × 8 and 4 × 4 layers 15 to 30% cheaper with A, a little more with B. Memory *[medium]*:
  A about 2.5 GB for the whole world's far view, B about half. Either way that's set against the 4 GB bar, with what it leaves
  room for, before anything is chosen.

### Results: the card test and the pictures

**Built as planned, with one change of when:** the atlas isn't kept up to date region by region. It's made once, on a
thread of its own, after every region is in, from the cells kept as they came. Till then the far eye blends in code, as
before. Borders then never need filling as neighbours arrive.

**The card test** (`atlas::card_tests`, 24,000 points at levels 32 to 1,024 through the nine regions round the spawn):
- **The pass mark missed.** It asked the exact way for 1/1,000 everywhere, and the card can't do that. Its filter weighs the
  eight cells to about 1/256, so how solid and how watery differ by up to 0.0045. And where hardly any surface weighs a point,
  the colour is a tiny sum's quotient and swings:

  | exact, by the point's surface | points | colour difference, median | 99th | largest |
  |---|---|---|---|---|
  | a quarter of a floor or more | 12,433 | 0.00005 | 0.0011 | 0.0063 |
  | 0.05 to 0.25 | 2,523 | 0.00005 | 0.0037 | 0.14 |
  | under 0.05 | 1,508 | 0.00005 | 0.16 | 0.49 |

  The exact way is exact to the hardware's own precision wherever a point shows real surface. The test's bar is now what
  the hardware can do: solid and watery within 1/200, colour within 1/100 where a quarter of a floor shows.
- **Spread** differs more broadly, by its nature: the colour's median 0.002, its 99th 0.08 with real surface and 0.27
  without, the faces by up to 1.0 where empty cells took a neighbour's.
- Both ways are known and unknown exactly where the code's blend is.

**The pictures** (off the screen, high over the spawn, 640 × 400, all three drawn from the same nine regions): the eye can't
tell them apart. Against the code's, pixel by pixel, out of 255: exact a median of 0, a 99th of 1 and at most 6; spread a
median of 0, a 99th of 2 and at most 8.

### Results: the three flights

Build 128 (842f83a), the code committed, 2026-11-12, 10:14 to 10:29, each way alone (its own textures built, so its memory
is its own): C the code's blend `kept logs/flight-1794500370.txt`, A exact `-1794500669`, B spread `-1794500962`. The
first mark was held till every region was in and the atlas built: 7.9 s to make and send A's, 6.8 s B's.

**The far eye on the card, whole, median ms (against C):**

| | C, the code's | A, exact | B, spread |
|---|---|---|---|
| on the ground at the spawn | 0.794 | 0.644 (−19%) | 0.645 (−19%) |
| the bare plain (the build, the scatter: the same) | 1.440 | 1.148 (−20%) | 1.140 (−21%) |
| high over the spawn | 2.564 | 2.210 (−14%) | 2.173 (−15%) |
| the mountain top, clear day | 2.750 | 2.226 (−19%) | 2.146 (−22%) |
| low over the sea | 4.007 | 2.942 (−27%) | 2.864 (−29%) |

The layers low over the sea: the 8 × 8 1.94 → 1.28 (A) and 1.24 (B); the 4 × 4 1.94 → 1.53 and 1.49.

**The card's memory (the library's allocator, in use and held):** C 1,125 and 1,611 MB; A 3,802 and 4,420 MB (its textures
2,592 MB); B 2,656 and 3,139 MB (its textures 1,481 MB). The bricks' pool (911 MB) stays in all three, for the code's blend
wherever the bricks can't answer.

- **Within the estimate:** 14 to 29% off the far eye at every mark, the most low over the sea. Spread is a little
  faster than exact (1 to 4%), with fewer bytes fetched.
- **The code's own blend got 1 to 8% slower** than build 126 (high over the spawn 2.564 against 2.375, the spawn 0.794
  against 0.771). Nothing in its path changed, but the hardware's path now sits beside it in the same program. It's old's
  lesson again: code added costs every pixel even when it's switched off.
- **Against the 4 GB bar** (the whole game's card memory, well under it at worst), with the near eye, light and the rest
  still to come:
  - A as it stands is over it (3.8 GB in use, 4.4 held), leaving no room.
  - B leaves about 1.3 GB in use (0.9 held): room for the near eye's estimate (0.25 to 0.9 GB), just.
  - Both are the whole world's far view on the card. The window round the eye (the plan's fourth step) would keep only the
    32s and 64s near the eye, which hold 94% of both the pool and the textures.
  - Without the pool (if the textures alone answered), A would be about 2.7 GB and B 1.6.
- **Still David's eye,** still and flying, the three switched live.

### David's look at build 128, and build 129's pass mark (written before the code)

David's look at the three blends, all three built (his words, 2026-11-12): "as far as the far blend, they all look the
same. -2 detail looked better, didn't seem to impact the frames. crisp looks better than smooth or sharp skylines." The
fog line he saw is written down under 2.2 to wait ("we may delete fog and try another way to implement it later").

- **The detail slider ran backwards.** It reads "detail (levels finer)", but its number was added straight to the level
  (old's level bias), so his -2 was two levels *finer*, each far cell a quarter as wide at a given distance. What he
  liked is the finer picture.

**Build 129:**
- **Spread kept, exact taken out.** They look the same, and spread is faster and 1.1 GB lighter. The code's blend
  stays where the bricks can't answer, and before the textures are in. The code's way stays on the Render page to
  compare.
- **The slider turned the right way**, more meaning finer, from -3 to +4, with +2 the default (David's pick).
  `INCUS_FAR_DETAIL` sets it at launch, for measuring.

**Pass mark:**
- **The card test, spread against the code's:** how solid and how watery within 1/200 at every point; known and
  unknown the same at every point; the colour's differences reported by surface, as before (spread differs by design
  where little surface shows).
- **The pictures:** high over the spawn at detail 0, the code's and spread's each the same as build 128's, pixel for
  pixel (only exact's path left the program).
- **Two flights,** spread at detail 0 (against build 128's B) and at +2 (the new default), with the card's memory.
  Builder's estimates:
  - At 0 [medium]: the same as B, or a few % faster with the program smaller.
  - At +2 [low]: a step is as long as the ray is wide, so finer cells don't multiply the steps. They cost in fetches
    and cache, and could earn more of the 4 × 4. The spawn about the same (David saw no change); the far marks 10 to
    50% slower.
  - The memory [medium]: B's, 2.66 GB in use, within 50 MB.

### Results: build 129's two flights

Build 129 (4ad4788), 2026-11-12, 10:55 to 11:05, spread at detail 0 `kept logs/flight-1794502826.txt` and at +2
`-1794503138`.

**The far eye on the card, whole, median ms:**

| | build 128 B (spread, 0) | 129 at 0 | 129 at +2 |
|---|---|---|---|
| on the ground at the spawn | 0.645 | 0.641 | 0.653 (+2%) |
| the bare plain | 1.140 | 1.142 | 1.263 (+11%) |
| high over the spawn | 2.173 | 2.180 | 3.854 (+77%) |
| the mountain top, clear day | 2.146 | 2.149 | 3.801 (+77%) |
| low over the sea | 2.864 | 2.862 | 3.098 (+8%) |

The layers at +2 against 0:
- The mountain top: 8 × 8 1.01 → 1.41, 4 × 4 1.00 → 1.16, **2 × 2 0.03 → 1.11**.
- High over the spawn: 8 × 8 0.89 → 1.33, 4 × 4 1.15 → 1.21, **2 × 2 0.03 → 1.20**.
- Low over the sea: 8 × 8 1.23 → 1.51, 4 × 4 1.50 → 1.46, the 2 × 2 not earned.

**The card's memory:** 2,656 MB in use and 3,139 held at both, B's to the megabyte. The atlas built in 14.2 and 20.2 s
(B's flight 6.8 s, David's look 12.7 s with both ways): noted, not chased.

- **At 0, met:** build 128's B within 0.6% at every mark. Taking exact out gave nothing back. The code's blend is still
  in the program as the fallback.
- **The memory, met.**
- **At +2, missed in part:**
  - The spawn (+2%), the plain (+11%) and the sea (+8%) are within or under the estimate.
  - The mountain top and high over the spawn are at +77%, over the 10 to 50% I gave.
  - I'd guessed finer cells might earn more of the 4 × 4. It was the 2 × 2 instead, never earned before at any mark,
    that the finer cells make worth it there: its rule weighs a pixel against the cells it reads, so cells a quarter
    as wide earn it where the eye looks down on near land. That's about 1.1 to 1.2 ms. The rest is the 8 × 8 (+0.3
    to 0.4 ms), reading finer cells.
- **For David's eye:** whether the 2 × 2 is what he liked at +2, or the finer cells alone. The 2 × 2's crossover (the
  Render page's "2 × 2 fades in from") can push it back out with +2 kept, its cost read on the gauge as he moves it.

**David's look from height (build 130, 2026-11-12):** he flew up and compared 0 and +2: "yeah 0 is fine". The default
goes back to 0, old's, which every flight before build 129 measured. At the spawn he'd seen no difference from -1 to +4
("as you have the detail set now, everything from -1 to 4 looks the same"). The fog line waits as written.

## Step 3: bricks of 4 against 8  *(plan 2.2)*

Memory is the reason for the step: the whole game under 4 GB on the card at worst, and the far view alone was 2.66 GB in
use after change 1.

### The count, before any code (2026-11-12)

The whole world's bands counted at four brick sizes, levels 32 to 1,024, where nearly all of it is (`bricks.rs`,
`bricks_of_each_size`, ignored, run when asked). 11.2 million cells in the bands (180 MB at 16 bytes).

| a brick's side | bricks | the pool (whole bricks) | how much of it is band | the atlas (side + 1 a side) | the index |
|---|---|---|---|---|---|
| 2 | 1,830,906 | 234 MB | 77% | 791 MB | 97 MB |
| 4 | 455,311 | 466 MB | 39% | 911 MB | 12 MB |
| **8 (now)** | 111,979 | 917 MB | 20% | 1,306 MB | 1.5 MB |
| 16 | 27,696 | 1,815 MB | 10% | 2,177 MB | 0.2 MB |

- **Bricks of 4:** the pool halves and the atlas loses 30%, about 0.85 GB off the far view.
- **Bricks of 2:** another 0.27 GB, but each brick's border makes the atlas 3.4 texels a cell, and the index grows to
  97 MB. It costs one more flight, so it's flown too.

### The pass mark (written before the code)

The brick's side is chosen at launch (`INCUS_FAR_BRICK`: 2, 4 or 8; 8 till flown), for the pool and the atlas both.

- **Tests:**
  - The far bricks read back as the far cells, at 2, 4 and 8.
  - Change 1's card test holds at each.
  - The edge test holds.
- **The pictures, high over the spawn:**
  - The code's blend at 4 and 2 is the same as at 8, pixel for pixel: the same cells, found another way.
  - Spread's is within 8 of 255 [medium]. Its colours spread through a smaller block, so an empty cell can take its
    colour from a different neighbour.
- **Three flights,** spread at detail 0, bricks of 8, 4 and 2, with the card's memory. Builder's estimates:
  - Memory in use [medium]: 2,656 MB at 8. At 4 about 1,650 MB (the pool grows by doubling, so 512 MB held against
    1,024). At 2 about 1,450.
  - Card time at 4 [low]: within 10% of 8 at every mark. The start and the march skip by the bands, not the bricks,
    and a blend reads the same number of things. Smaller bricks scatter the atlas's texels, so the far marks could be
    up to 10% slower.
  - At 2 [low]: up to 20% slower.

### Results: the three flights

Build 131 (f6958f9), 2026-11-12, 11:55 to 12:10, spread at detail 0: bricks of 8 `kept logs/flight-1794506431.txt`, 4
`-1794506724`, 2 `-1794507025`.

**The far eye on the card, whole, median ms:**

| | bricks of 8 | 4 | 2 |
|---|---|---|---|
| on the ground at the spawn | 0.653 | 0.702 (+7.5%) | 0.772 (+18%) |
| the bare plain | 1.144 | 1.205 (+5.3%) | 1.301 (+14%) |
| the mountain top, clear day | 2.152 | 2.307 (+7.2%) | 2.631 (+22%) |
| high over the spawn | 2.178 | 2.332 (+7.1%) | 2.603 (+20%) |
| low over the sea | 2.865 | 3.233 (+12.8%) | 3.564 (+24%) |

**The card's memory, in use (held):**

| | the pool (used of grown) | the atlas | the whole game |
|---|---|---|---|
| bricks of 8 | 911 of 1,074 MB | 1,481 MB | 2,656 MB (3,139) |
| 4 | 466 of 537 MB | 934 MB | 1,555 MB (2,147) |
| 2 | 234 of 268 MB | 794 MB | 1,315 MB (2,013) |

The atlas was made and sent in 17.0 s at 8, 3.3 s at 4 and 3.2 s at 2.

- **Memory, met and better:** I gave about 1,650 MB at 4 and got 1,555; I gave 1,450 at 2 and got 1,315.
- **Card time at 4, missed at one mark:** within 10% at four of the five marks. Low over the sea, the dearest, is
  12.8% slower, over the 10% I gave.
- **At 2, missed at two marks:** the mountain top (+22%) and low over the sea (+24%) are over the 20% I gave.
- **The trade:**
  - Bricks of 4 take 1.1 GB off the card for 5 to 13% more far time: 0.37 ms at the dearest mark, 0.05 at the spawn.
  - Bricks of 2 take a further 240 MB for 7 to 10% more on top.
  - The memory goes on the near eye and everything else under the 4 GB bar.
- **Builder's recommendation:** bricks of 4. The window round the eye (step 4) is still to come, and it would take
  most of what's left off the card.

**David's pick (2026-11-12):** "4 sounds good". Bricks of 4 are the default from here. Counsel's point, which David
took: the window round the eye (step 4) cuts the same memory another way, so the last pick waits on step 4's flights
at 4 and at 8, and 8 stays a launch switch away (`INCUS_FAR_BRICK=8`).

## Step 4: a window round the eye for the 32s and 64s  *(plan 2.2)*

At bricks of 4, the 32s hold 75% of the far bricks and the 64s 18%. They're only read near the eye: at detail 0 on
David's 2560 × 1600, the 8 × 8 layer reads 32s out to about 9 km and 64s to about 18 km.

### What's built

- **The window.** Only the 32s within 10 regions of the eye (10,240 blocks) are on the card, and the 64s within 20.
  The 128s and up stay whole.
  - The bands (`entries`) stay whole, for skipping.
  - `INCUS_FAR_WINDOW` sets the 32s' radius in regions (the 64s' is twice it); 0 is the whole world, as before.
- **The conveyor.** Each frame, within the putting pass's 3 ms, it makes and sends the regions coming into the window
  (nearest first) from the cells kept on the processor, and gives back the room of those that left.
  - A region leaves one region past the radius, so the edge doesn't flicker in and out.
  - Freed places in the pool and the atlas are used again, so neither grows with play.
- **Usable regions.** A region is usable at a level when it and its eight neighbours are on the card, since a blend
  reaches one cell over. The card keeps a flag for each.
  - Where a 32 isn't usable, the far eye reads the 64; where neither is, the 128.
  - So falling behind shows coarser cells near the window's edge, never a hole.
- **The atlas follows.** It's built once when every region is in, from the coarse levels and the window. After that,
  bricks are made and sent as they come in, and it grows by copying if it must.
- **A flight that moves.** A mark may have a speed: the camera flies straight ahead along its facing, level, through
  its settling and measuring. Two moving marks are added:
  - a player's top speed, 200 blocks a second (the spec), low over the land;
  - fast, 2,000 a second.
  - Each standing reports where the eye began and ended, so a run that didn't move is seen. It also reports the
    least reach of usable 32s and 64s round the eye over the measured frames: how far behind the window fell.

### The pass mark (written before the code)

- **Tests:**
  - The far bricks read back as the far cells in usable regions, after the window has moved and come back.
  - The window, moved back and forth, leaves the pool's and the atlas's room where its first visit left it.
  - Change 1's card test and the edge test hold.
- **The pictures, still:** the same at the ground marks as with the whole world. High over the spawn, the finer
  layers far out may read a 64 where they read a 32 before: within 8 of 255 [medium].
- **Memory [medium]:** the whole game in use from 1,555 MB (bricks of 4, the whole world) to about 450 MB, and at
  bricks of 8 from 2,656 to about 700. It no longer grows with the world's size, except at the 128s and up.
- **Card time, still [medium]:** within 3% of build 131's at each mark. One flag is read per blend.
- **Moving, at 200 blocks a second:** solid. The window is never behind by more than a region (the usable 32s reach at
  least 8 regions round the eye), and the frames are like the still marks'. This is the player's bar ("solid as a
  rock when i stop and act more like a player").
- **Moving, at 2,000 a second:** no bar. Where it starts to creak is reported: how far the window falls behind, and
  the frames' spread. David's look shows what a player would see if it fell behind (David: "if the game creaks at
  10,000mph that's fine. as long as its not aggregious").
- **Flights:** bricks of 4 and 8 with the window, the still marks and the two moving ones; build 131's give the
  whole world.

### Results: the tests and the pictures

- **Tests, met:**
  - The window's test brought 49 regions on and off through 11 moves. Once it had been both ways, more moves left the
    pool's use and room as they were (nothing grows with play).
  - The usable regions' 159,744 cells read back through the card as summed.
  - On a first visit, only the middle region at the 32s is usable, so the usable reach is half a region, as worked out.
  - The read-back test at 2, 4 and 8, change 1's card test and the edge test all hold through the window.
  - The far eye now reads 9 storage buffers, one past wgpu's default of 8. The limit is raised to 16 on purpose
    (`card.rs`, said in the log; DirectX 12 cards allow 64 or more).
- **The pictures, still, missed in part:**
  - Drawn at 640 × 400 from 13 × 13 regions round each mark, the whole world (`INCUS_FAR_WINDOW=0`) against a
    window of 3. At a quarter of David's 1,600 rows, a pixel covers four times the ground, so 3 here is about 12 on
    his screen, a little more than the 10 chosen.
  - On the ground at the spawn: 0.1% of pixels differ, by 4 of 255 at most. Met.
  - Low over the sea: 0.4%, by 22 at most.
  - High over the spawn: 4.5%, by 35.
  - The mountain top, fog off: 6.5%, by 49 (0.6% by more than 8).
  - The last three are over the 8 I gave. The differences are all on distant ridges. The 8 × 8 layer reads 32s only
    out to about 9 km on his screen, but the finer layers read them twice and four times as far, and past the window
    they read 64s instead.
  - A window of 1 here (about 4 on his screen) shows what falling behind looks like: the land coarser and blurrier,
    never a hole.
  - What's left is for David's eye on his own screen: the window against the whole world at the mountain top and
    high over the spawn.

### The first flights, and the camera's step (build 132)

Build 132 flown 12:44 to 13:02: the window at bricks of 4 `kept logs/flight-1794509431.txt` and 8 `-1794509780`, and
the whole world at 4 `-1794510123`.

- **The moving marks didn't move as asked, and the report showed it.** The player's run went 55 to 67 blocks of the
  5,000 asked; the fast run went 2,400 to 2,800 of the 50,000. That's about 1.3% of its speed.
- **The cause is the camera's step.** It was the time from the last frame handed over to this frame's start, the
  sliver between frames, not the frame. So the camera has flown at about 1% of its speed since the rebuild began. That
  was David's "too slow to notice", and why the speed slider seemed weak.
  - Fixed: the step is now from one frame's step to the next (`clocks.rs`).
  - A test holds it. Run against the old step, it fails ("a 50 ms frame steps 0.05 s, not 0.001").
- **The fast run started inside a mountain's reach.** At 300 over its start it flew into snowy peaks, where the far
  eye has little to march (0.24 ms). It now starts at 1,300 over its start, above the highest peaks.
- **The still marks are good:**
  - The card's memory in use: the window at bricks of 4 **470 MB** (872 held); at 8, 916 MB (1,879); the whole world
    at 4, 1,791 MB (2,253).
  - The whole world is now 236 MB more than build 131's, from the atlas's room for the window's moves (a quarter
    again and a layer), which a whole world doesn't need.
  - So at 4 the estimate of about 450 was met; at 8, about 700, missed (916).
  - Card time with the window against the whole world, same build, at bricks of 4: within 3% at every mark (+0.7,
    -2.4, +1.1, -0.6, +1.6%). Met.
  - The usable 32s reached at least 8.1 to 8.3 km round the eye at every still mark, the 64s 18.3 to 18.5 km: the
    window was whole.
- **After the 22 km jump to the fast run's start,** at bricks of 4 the 64s reached only 9.3 km over its measured
  frames (18.3 at 8). The conveyor at 4 hadn't caught up in the 5 s of settling.
- The moving runs are flown again with the step fixed. Their numbers above aren't the window's.

### Results: the window flown again, moving (build 133)

Build 133 (85d876f), 13:05 to 13:23: the window at bricks of 4 `kept logs/flight-1794510700.txt` and 8 `-1794511049`,
the whole world at 4 `-1794511399`. The moving marks moved as asked:
- The player's run went 5,000 blocks each time.
- The fast run went 50.0 km with the whole world and 49.6 at 8, but 45.6 at 4. The 22 km jump to its start gave
  back 40,000 bricks in one frame, written one by one; the stall lost the run 2.2 s, since a step is capped at a tenth
  of a second. Fixed below.

**Memory, in use:** the window at 4 **477 MB**, at 8 916 MB; the whole world at 4, 1,791 MB.

**Still, the card's whole (window at 4 against the whole world at 4):** the mountain top 2.114 against 2.168, high over
the spawn 2.267 against 2.262, low over the sea 3.071 against 3.024. Within 3%: met.

**Moving, a player's 200 blocks a second, bricks of 4:**

| | the window | the whole world |
|---|---|---|
| the usable 32s reached at least (blocks) | 8,070 (8,172 standing on the plain) | (all) |
| the card's whole, median | 3.209 | 3.180 |
| the frame: median, 95th, 99th | 3.206, 3.793, 4.489 | 3.190, 3.718, 3.943 |
| the frame: 99.9th, worst | **6.733, 9.385** | 4.194, 4.400 |
| the putting pass: 99th, worst (processor) | 3.2, 8.0 | 0.001, 0.002 |

- **Behind by about 100 blocks, so in substance met.** The window falls behind by about 100 blocks against standing at
  the same place. The bar as I wrote it (8 regions, 8,192 blocks) was tighter than the grid allows even standing
  still (8,172).
- **The frames' tail missed.** I said the frames would be like the still marks'. The median is the same, but the 99th
  is 0.5 ms worse, the 99.9th 2.5 ms, and the worst frame 9.4 ms against 4.4.
- **The cause is the conveyor's spikes.** The putting pass goes over its 3 ms when one region's bricks and textures
  are made in a frame, since a region is the smallest piece it does, and the textures' share isn't counted in its
  time.
- **Whether that's solid** is David's to feel. If it isn't, the conveyor could do a brick at a time instead of a
  region, and count the textures.

**Moving fast, 2,000 a second, bricks of 4:**
- The window kept up: the usable 32s reached at least 8,073 blocks, the 64s 18,289.
- The frames creaked: median 1.58 against 1.41 for the whole world, 95th 5.2 against 2.3, 99th 9.3 against 2.5,
  worst 22 ms against 4.4.
- No bar here. It's not "aggregious": the slowest frame was 45 a second.

**After these flights (build 134):**
- A region given back no longer writes its bricks' places on the card, one by one. The flags say it isn't usable, so
  nothing reads them, and a region brought back writes them again.
- The usable flags now ride after the last level's bands in the bands' buffer. The far eye reads wgpu's default 8
  storage buffers again, and the raised limit is taken back. Counsel's question: a card that allows only 8 (tier-1
  DirectX 12) would have refused the far eye.
- Flown once more at bricks of 4, to see that both cost nothing and the jump's stall is gone.

**Build 134 flown at bricks of 4 with the window** (6dc9171, 13:29 to 13:35, `kept logs/flight-1794512135.txt`):
- The fast run went its 50.0 km: the jump's stall is gone.
- Memory 471 MB in use.
- The still marks within 1% of build 133's (the spawn 0.706, the mountain top 2.114, the plain 1.179, high over the
  spawn 2.287, low over the sea 3.092), so the flags in the bands' buffer cost nothing.
- At a player's 200 a second, the tail is as before: 99.9th 6.2 ms, worst 7.9, the putting pass's worst 7.2.
- At 2,000 a second: the window kept up (8,072 and 18,303 blocks); frames 99th 8.7, worst 23.6.
- Next, David's look: the window's distant ridges against the whole world's, and moving at a player's pace with the
  camera at its true speed, for whether the conveyor's spikes can be felt.

## Change 2: rays begun at drawn boxes  *(plan 2.2)*

The Oracle's: "The triangle hardware draws the bricks' boxes; a pixel with no box behind it is sky and runs nothing.
Flown against a plain full-screen start."

### Where the far eye's steps go now (before any code, 2026-11-12)

The picture test can now load the whole world as the game does (`FAR_TEST_REGIONS=all`, the loading lane). Its loading
loop hands the card its writes each time round: held, their copies piled up till the card ran out of memory. Each far
mark was drawn at 640 × 400 with the steps' view, bricks of 4 with the window.

- **Low over the sea and the bare plain:**
  - The cost is a thin band at the horizon, and the sky just above it, where rays skim the land for tens of
    kilometres before they're over the highest ground (the Oracle's guess, 2026-11-07).
  - The ground is cheap.
- **High over the spawn:** the cost is along the far horizon behind the ridges.
- **The mountain top:** the sky is nearly free, since rays start above everything. The cost is in the grazing land
  toward the horizon.
- **A rough count:** pixels judged sky by their colour take 82% of the steps low over the sea and 67% at the plain.
  Fogged far land reads as sky to that test, so these are high, and worthless at the mountain top.
- **Most of the sky's cost is in the 8 × 8 layer.** The finer layers aren't earned over the sky (its worth is
  nothing). Low over the sea that layer is 1.23 of the 3.07 ms.
- **What boxes could win:** the sky rays with no box behind them. Rays that skim far land still march, from the box
  they meet.

**The brick size, last pick (2026-11-12):** with the window, memory stopped binding: 916 MB at bricks of 8 against 471
at 4, both far under the 4 GB bar, and 8 is 6 to 11% faster on the card. Counsel recommended 8, and David said "you can
go with 8". The default since 33e48a1.

David's go (2026-11-12), after the plan was put to him: "yes, proceed".

### What's built

- **The far boxes.** Each 256-block column of land (the 256s' bands, as on the card already, so no new data) is
  drawn as a box by the card's triangle hardware, once a frame, at the 8 × 8 layer's size, with the same rays: a
  layer pixel's middle.
  - A box runs from under the world's floor to its band's top.
  - It's swollen by what a blend can reach at its far corner's distance: the level read there, plus the next for
    the hand-over, a whole cell of that every way, its top rounded up to that cell.
  - Each pixel keeps the farthest box surface its ray passes, front or back (a depth test, farthest kept).
- **The march at the 8 × 8 layer stops there.** Past the last box nothing can be met, so a ray with no box behind it
  is sky at once, and a ray skimming the horizon stops once it's past the last land it could touch.
  - Past the map's edge there are no boxes. Rays now step on through nothing to the reach (150 km) or till they're
    over the highest block.
  - The finer layers are unchanged: they aren't earned over the sky.
- **`INCUS_FAR_BOXES=0`** marches as before, for the flights against it.

### The pass mark (written before the code)

- **The picture is the same, pixel for pixel,** with the boxes and without, at the far marks and the spawn over the
  whole world (the test loading it as the game does). They're conservative by construction, so any difference is a
  box too small.
- **The check must bite:** boxes deliberately too small (no swelling) change the picture.
- **The steps' view:** the sky over the horizon goes from teal to dark blue at the plain and over the sea.
- **Flights, bricks of 8 with the window, boxes on and off.** Builder's estimates [low]:
  - Low over the sea and the plain: 10 to 25% off the far eye. Rays just over the horizon still meet swollen far
    boxes for 20 to 30 km, so not all of the sky is won.
  - High over the spawn and the mountain top: under 5% either way.
  - The boxes' own pass: 0.05 to 0.2 ms (83,000 boxes, 36 corners each).
- **Memory:** a depth picture at the 8 × 8 layer's size, about 0.3 MB.

### Results: the pictures and the steps

Drawn at 640 × 400 over the whole world, bricks of 8 with the window.

- **The same picture, met:** with the boxes and without, 0 pixels differ at the spawn, the mountain top, the plain,
  high over the spawn and low over the sea.
- **The check bites:** boxes left unswollen change 2,880 to 3,008 pixels (about 1%, the skylines) by up to 79 of 255.
  The swelling is what keeps the picture exact.
- **The steps' view, met:** the sky over the horizon goes from teal to dark blue. What's left is a band just over the
  horizon, where rays still pass over far land they could touch. Counted over the whole picture: 23% fewer steps low
  over the sea and 24% fewer at the plain.

### Results: the two flights

Build 137 (9548078), 14:37 to 14:49, bricks of 8 with the window: the boxes on `kept logs/flight-1794516237.txt`, off
`-1794516591`.

| | the boxes on | off |
|---|---|---|
| the far eye whole: on the ground at the spawn | 0.834 | 0.670 |
| the mountain top, clear day | 2.020 | 1.999 |
| the bare plain | 1.122 | 1.122 |
| high over the spawn | 2.464 | 2.291 |
| low over the sea | 3.094 | 2.934 |
| the boxes' pass | 0.164 to 0.177 | |
| the 8 × 8 layer: the mountain top, the plain | 0.775, 0.813 | 0.919, 0.979 |
| the 8 × 8 layer: high over the spawn, the sea | 0.840, 1.194 | 0.869, 1.204 |

- **The boxes' pass, met:** 0.16 to 0.18 ms, inside the 0.05 to 0.2 given.
- **10 to 25% off at the horizon marks, missed:** the plain broke even and the sea is 5% slower. The 8 × 8 layer saved
  16 to 17% at the plain and the mountain top, but only 1% over the sea and 3% high over the spawn. The pass costs
  about what the plain and the mountain top save, and more than the rest.
- **Under 5% either way high up, missed** high over the spawn (+7.5%); met at the mountain top (+1%).
- **Why:** the steps' view showed a quarter of the steps gone, but they were the sky's cheap ones. A ray through
  empty sky only reads bands and skips, two reads a step. The time is where rays meet land near the horizon, blending
  cells step by step, and the boxes don't shorten that. The Oracle's guess (2026-11-07) was right about where the
  steps are, not about where the time is.
- **The heat:** the CPU area at 76 to 104 °C through both flights, the GPU at 72 to 85 °C and its clock 2,407 to
  2,797 MHz at the still marks. One fast run read 1,312 MHz at its slowest: the card idling for a moment, or heat;
  watched for in the next.
- **Builder's recommendation:** the boxes off by default (`INCUS_FAR_BOXES`, kept), the code kept behind its switch
  till the Intel card is flown (step 7), then kept or taken out on its numbers there.

### Does the boxes' code cost the march when it's off? (Counsel's question)

Change 1's lesson was that code sitting in the program can slow it even switched off. So build 136 (ecb67ef, the same game
without change 2) was flown against build 137 with the boxes off (`INCUS_FAR_BOXES=0`), alternating, twice each,
15:15 to 15:39.
- Build 136 was flown from a worktree with its own build folder, the world folder linked in, taken down after.
- The reports: 136 `kept logs/flight-1794518484.txt` and `-1794519192`; 137 off `-1794518831` and `-1794519546`.

| the far eye whole, median ms | 136 | 137, boxes off |
|---|---|---|
| on the ground at the spawn | 0.669, 0.673 | 0.679, 0.677 |
| the mountain top | 1.992, 1.991 | 1.994, 1.998 |
| the bare plain | 1.118, 1.120 | 1.122, 1.123 |
| high over the spawn | 2.288, 2.288 | 2.293, 2.294 |
| low over the sea | 2.927, 2.927 | 2.936, 2.940 |
| a player's run | 3.256, 3.252 | 3.258, 3.261 |

- **The price, measured:** the code costs 0.2 to 1% even off (at most 0.01 ms), the same way at every mark in both
  pairs.
- **The 7.5% hint was noise:** build 137 off against build 133's flight suggested +7.5% high over the spawn and over
  the sea, but that was between builds and sessions.
- **How steady flights are:** each build's two flights agree within 0.004 ms. So a 1% difference is real, and
  comparisons between sessions are worth no more than about 5%.
- **The card's clock** dipped once a flight at a still mark (1,125 to 1,350 MHz at the slowest) without moving the
  medians.
- **Builder's recommendation, still:** off by default and kept till the Intel flights. Its price till then is under
  1%, and the Intel card's balance of triangles against marching may differ. If David would rather not carry it, take
  it out now; git keeps it.

## The main memory  *(David, via Counsel: "begin tracking ram use, and start looking for opportunities where using it is almost a free win or takes some heat/load off something else")*

Tracked from build 138: Windows' own count for the game's process, in each flight's report and the session's log.

**The first flight** (build 138, dce3afb, 15:56 to 16:02, `kept logs/flight-1794520968.txt`), bricks of 8 with the window:
- **The game holds 2,826 MB at the end** (2,829 at most), but only 1,036 MB of it is in memory (1,220 at most). The
  machine has 33.8 GB, 16.1 GB free.
- **Through the session's log:**
  - 1,312 MB at 1.6 s, before the far cells come in;
  - 1,851 MB at 31.6 s, as they come in;
  - 2,670 MB at 61.6 s, all in and the atlas built;
  - then up to 2,804 MB as the flight moves through its marks, and 2,826 MB by the end.
- **Unexplained:** what the 1.3 GB at the start is, and why only a third of what's held is in memory. Drivers and
  copies made for the card are likely, but that's unmeasured. A breakdown comes before any free win is priced.
- **Watched:** about 20 MB more over the last four minutes of marks.
- **From build 139, the temperatures are drawn on flights too** [David: "while you are running tests, i am sitting
  here watching, and id like to know what the temps are at while watching"]. A few letters of drawing, so earlier
  flights compare with later ones a hair less exactly; within a session, as before.

## After the Oracle's return: the boxes out, the map's edge, and her two tests  *(plan 2.2)*

The Oracle's reading (2026-11-12): the far eye is waiting, not working. About a thousand long rays skimming the horizon
set each layer's time, each step waiting on the one before, while the card sits nearly idle. David: "we should at least
try 2, 3 and 4", and these first, to answer her with numbers.

### The pass mark (written before the code)

- **The boxes taken out** (git keeps them at 9548078). The march keeps its `end`.
- **Rays end at the map's edge.** Past the map there's nothing, but today a ray goes on toward the 150 km reach till
  it's over the highest block.
  - The end is where the ray leaves the map's square, swollen by two cells of the level after the one read there (a
    blend's reach). Worked out per ray by arithmetic, in every layer.
  - `INCUS_FAR_EDGE=0` marches on as before.
  - The picture the same, pixel for pixel, as today's saved ones at five marks over the whole world.
  - Estimate [low]: 0 to 5% off at the horizon marks. The steps saved are skips, the cheap kind, as the boxes showed.
- **Her two tests, as measuring switches:**
  - `INCUS_TEST_HALF=1`: only the left half of each layer marched. If a layer's time barely moves, it waits on its
    longest rays; if it halves, the card is busy.
  - `INCUS_TEST_STEPS=384`: the step limit halved from 768. What the longest rays cost; the picture may change at the
    horizon, and that's looked at.
  - Builder's guess [low]: half the rays takes 25 to 40% off a layer, between her two readings; 384 steps takes under
    10% off, since few rays reach 768.
- **Flown in one session:** the edge off, the edge on, half the rays, 384 steps.

### Results: the map's edge and the Oracle's two tests

Build 140 (0425213), 16:33 to 16:56, one session: the edge off `kept logs/flight-1794523158.txt`, on `-1794523509`,
half the rays `-1794523855`, 384 steps `-1794524200`.

| median ms | edge off | edge on | half the rays | 384 steps |
|---|---|---|---|---|
| the spawn, whole | 0.676 | 0.677 | 0.485 | 0.678 |
| the mountain top, whole | 1.981 | 1.762 | 1.563 | 1.634 |
| the bare plain, whole | 1.120 | 1.056 | 0.723 | 0.937 |
| high over the spawn, whole | 2.141 | 1.751 | 1.458 | 1.748 |
| low over the sea, whole | 2.770 | 2.582 | 1.108 | 2.054 |
| a player's run, whole | 3.103 | 2.515 | 1.688 | 2.180 |
| the 8 × 8: plain, sea | 0.976, 1.194 | 0.912, 1.195 | 0.584, 0.970 | 0.792, 1.105 |
| the 4 × 4: mountain top, sea | 0.910, 1.440 | 0.838, 1.252 | 0.751, 0.008 | 0.711, 0.814 |

- **The map's edge, better than estimated:** 6 to 19% off at every far mark (0 to 5% given), nothing at the spawn. Most
  of it is in the 4 × 4 layer (high over the spawn 1.137 to 0.779). Rays marching on past the map were dearer than
  skips: past the map every step is a full blend of nothing.
- **Half the rays:** the 8 × 8 layer loses 13 to 36%, not half and not nothing (Builder's guess 25 to 40%). So the
  truth sits between the Oracle's two readings, nearer "waiting": half the rays gone, two thirds or more of the time
  left.
  - The 4 × 4 over the sea reads nothing with the right half blank: its land to refine was all on the right. Not
    a measure there.
- **384 steps:** 20% off the sea whole and 11% off the plain (under 10% guessed: missed). The 4 × 4 over the sea
  loses 35%. Rays past 384 steps are few, and they cost a fifth of the dearest mark.
- **What it points to:** the long rays set much of each layer's time. The lean step (each step cheaper) and lazy rays
  (the long ones spread over frames) both aim at it. Shuffling the rays, which helps a busy card, would win the
  smaller share.

### Watts and load at the marks (build 141, with LibreHardwareMonitor)

One flight, 17:25 to 17:31, `kept logs/flight-1794525954.txt`, uncapped as every flight is. Each standing's most:

| | the GPU's watts | its load | the processor's watts, load | its hottest core |
|---|---|---|---|---|
| on the ground at the spawn | 99 | 98% | 42 W, 11% | 90 °C |
| the mountain top | 89 | 100% | 35 W, 6% | 102 °C |
| the bare plain | 78 | 99% | 37 W, 7% | 100 °C |
| high over the spawn | 83 | 100% | 36 W, 6% | 102 °C |
| low over the sea | 58 | 100% | 35 W, 6% | 92 °C |
| a player's run | 63 | 100% | 34 W, 5% | 98 °C |
| fast across the world | 103 | 86% | 37 W, 10% | 104 °C |

- **The dearest marks draw the least power.** The card reports itself fully loaded everywhere, yet over the sea it
  draws 58 W against 99 W at the spawn, the cheapest mark. "Loaded" means only that it always has work queued; the
  watts say how much of it is working. A card kept waiting on a few long chains of steps looks like this. It's the
  strongest sign yet for the Oracle's first reading.
- **The processor is nearly idle:** 5 to 11% load and 34 to 42 W through the flight. There's room there for work
  that takes load off the card.
- **Its cores still reach 90 to 104 °C** at that light load: the laptop's shared cooling, heated by the card. Degrees
  read beside watts, as Counsel said.

### The Nsight look at the new far march (build 141, low over the sea)

GPU Trace Profiler, 20 frames from 80 s in, launched from the command line as in `dyno.md`'s notes; the trace in
Builder's scratchpad (`nsight/incus_artus_2026_11_12_17_32_41.ngfx-gputrace`). Each pass, averaged over the 20 frames:

| | groups of rays launched | the card's room in flight | instructions run, of its peak |
|---|---|---|---|
| the far start | 1,264 | 29.7% | 29% |
| the 8 × 8 | 1,769 | **6.9%** | 11.5% |
| the 4 × 4 | 7,367 | **1.3%** | 2.2% |
| the 2 × 2 (nearly all "not wanted") | 30,447 | 24.8% | 41% |
| bringing it up (plain pixels) | | 95.6% | 51% |

- **The far march runs with the card nearly empty.** The two passes that take almost all the far eye's time over the
  sea (about 1.2 ms each) keep 6.9% and 1.3% of the card's room in flight. Bringing it up, a plain pass, fills 96%.
- **It's the Oracle's first reading, confirmed:** waiting, not working. The 8 × 8 has only 1,769 groups of rays in
  all; most finish at once, and the layer lasts as long as its few longest. Worse than old's 13 to 17%, and for a
  different reason: old's program was too big to fit more in flight, where here there are too few long rays to fill
  the card at all.
- **So three signs agree:** half the rays (13 to 36% off, not half), the watts (58 W at the dearest mark against 99 at
  the cheapest), and the card's room in flight (1 to 7%).
- **What follows:**
  - More rays would cost almost nothing while the long ones run: the card has 93 to 99% of its room free in those
    passes. The Oracle's "fewer layers waiting on each other" and a finer base layer are nearly free.
  - The time is the longest chain: fewer steps on it (the edge, a column-height skip), cheaper steps (the lean step),
    or the chain spread over frames (lazy rays).
  - Shuffling rays, which helps a full card, has little to win here.

## The Oracle's order, step 0: an instrument, a mark, and one flight  *(plan 2.2)*

David took the Oracle's order for the rest of 2.2 ("Yes, its order", `journal/vision/oracle-review.md`, the third part).

### The pass mark (written before the code)

- **The longest ray, per layer, in every flight's report:** the most steps any ray took, and how many rays took over
  256, 384 and 512.
  - Each layer keeps its rays' steps beside their distances (the distances' picture made four numbers wide).
  - A small pass of its own counts them, once a second on a flight, read back without waiting.
  - The picture must stay the same pixel for pixel, and a still mark's far time within 1% of before.
  - Check that it bites: with the step limit at 384, no layer may report more than 384.
- **Two nodding marks,** at the sea and the plain: the camera pitched slowly up and down through two rows of the
  8 × 8 layer (16 screen rows), so the horizon's worst case is always met. Read at their 99th.
- **One flight on the Intel card,** as the far eye stands, for which kind of card it is: waiting or working. It needs
  the laptop on Optimus, put to David a step ahead.

### Results: the longest rays (build 142)

Two flights, 18:09 to 18:23: as it stands `kept logs/flight-1794528905.txt`, and with the step limit at 384
`-1794529296`.

- **The count bites:** with the limit at 384 no layer reports more than 384, and the rays over 384 and 512 are none.
- **Nothing lost:** the far eye's time is within 1% of build 140's at every still mark (0.677, 1.782, 1.060, 1.757,
  2.594), and the picture byte for byte the same.

**The longest ray, and the rays over 256, 384 and 512 steps:**

| | the 8 × 8 (64,000 rays) | the 4 × 4 (256,000 at most) |
|---|---|---|
| on the ground at the spawn | 137 steps; none | not earned |
| the bare plain | 467; 296, 164, 0 | not earned |
| the mountain top | 322; 92, 0, 0 | 481; 2,356, 440, 0 |
| high over the spawn | 286; 196, 0, 0 | 455; 889, 143, 0 |
| low over the sea | 558; 422, 217, 37 | 764; 231, 175, 88 |
| a player's run | 484; 818, 262, 0 | 767; 1,867, 698, 450 |

- **A few hundred rays set the dearest marks.** Over the sea, 37 of the 8 × 8's 64,000 rays go past 512 steps, and
  88 of the 4 × 4's; its longest reaches the limit of 768. The Oracle's "about a thousand rays" is, if anything,
  generous.
- **Cost follows the longest ray, not the count.** The spawn's longest is 137 steps at 0.68 ms; the plain's 467 at
  1.06; the sea's 558 and 764 at 2.59. The mountain top has ten times the sea's rays over 256 in its 4 × 4 and is
  cheaper.
- **The nodding marks** (99th of the far eye whole): the plain 1.184 against 1.060 standing; the sea 3.147 against
  2.594. The horizon's worst case is a fifth dearer than the still mark at the sea. These are the numbers the gate is
  read against from here.

### Why it runs hot (David: "id really love to know why its running so hot")

Read from LibreHardwareMonitor, 30 s each, 2026-11-12 evening:

| | the processor | its hottest core, mean (highest) | the busiest core | the card |
|---|---|---|---|---|
| no game running | 32 W, 5% load | 77 °C (102) | 37% | 16 W, 44 °C |
| the game, capped at the screen's rate, its own readings off | 26 W, 3% | 73 °C (76) | 26% | 33 W, 49 °C |
| the same, its readings on | 26 W, 4% | 70 °C (78) | 41% | 33 W, 48 °C |
| a flight (uncapped), mid-run | 38 W, 6% | 84 °C, to 104 over a flight | 65%, at 5.15 GHz | 81 W, 70 °C |

- **The game at a player's frame rate isn't the heat:** 26 W and a hottest core in the 70s, no hotter than the desk
  with no game. Its own temperature readings make no difference.
- **The flights are:** uncapped, one thread prepares frames as fast as it can, so one core sits near its top turbo
  speed, with the card's 60 to 100 W in the same cooling.
- **The spikes past 100 °C come without the game too** (102 °C with nothing but the desk's own programs): short bursts
  on one core at turbo speed, this laptop's way in its Turbo mode.

## The Oracle's step 1a: the lean step, kept in hand  *(plan 2.2)*

### The pass mark (written before the code)

What doesn't change from step to step is kept in hand by the march, and read again only when it changes:
- **The level's numbers** (its place in the table): once, till the level changes. Two levels' in hand, the one read
  and the next for the hand-over, and the bands' part for the two levels the skipping reads.
- **The region:** whether it's summed, and whether its 32s and 64s are usable, once a region.
- **The four "is it summed" reads** before each blend: none while the blend's columns sit inside a summed region
  already in hand; as before at a region's edge.
- **The brick's slot:** kept till the ray leaves the brick.
- **The level itself:** kept over its stretch of the ray; the two logarithms a step only in a sliver at each
  stretch's ends and over the hand-over, so the level and the hand-over's mix are the very numbers as before.

The sun's and the sky's cones are left as they are (their own half, if this one pays).

- **The picture pixel for pixel the same** at five marks over the whole world. Exact by construction; any difference
  is a mistake.
- **The longest rays the same** step for step in the reports (the same rays take the same steps).
- **Flown against build 142 in one session, alternating** (142 from a worktree, as change 2's check was). Builder's
  estimate [low]: 15 to 35% off the 8 × 8 and 4 × 4 layers at the far marks; little at the spawn.
- **Its keep-or-drop waits on the Intel flight** (Counsel: on a working card every number held is room lost).

### Results: 1a against build 142

Flown 18:46 to 19:12, alternating twice (142 from a worktree, taken down after): 142 `kept logs/flight-1794531167.txt`
and `-1794531970`; 143 `-1794531569` and `-1794532362`. Each build's two flights agree within 0.01 ms.

| the far eye whole, median ms | 142 | 143, kept in hand | |
|---|---|---|---|
| on the ground at the spawn | 0.671, 0.681 | 0.707, 0.719 | +5% |
| the bare plain | 1.059, 1.059 | 0.950, 0.950 | −10% |
| the mountain top | 1.786, 1.784 | 1.679, 1.677 | −6% |
| high over the spawn | 1.755, 1.749 | 1.655, 1.653 | −6% |
| low over the sea | 2.583, 2.582 | 2.244, 2.240 | −13% |
| a player's run | 2.580, 2.573 | 2.335, 2.335 | −9% |
| the plain, nodding, 99th | 1.179, 1.178 | 1.028, 1.026 | −13% |
| the sea, nodding, 99th | 3.136, 3.133 | 2.721, 2.715 | −13% |

- **The longest rays the same,** step for step (137, 467, 322 and 481, 286 and 455, 558 and 764): met.
- **The layers:** the 8 × 8 over the sea 1.195 to 0.959 (−20%), at the plain −13%, the mountain top −8%; the 4 × 4
  over the sea −9%. Inside the 15 to 35% given at the sea's 8 × 8 only; under it elsewhere: missed in part.
- **The spawn is 5% slower** (0.04 ms), where "little" was given: its rays are short (137 steps at most), and each
  pays for setting up what it keeps without the steps to earn it back. The fast run reads 3% slower too.
- **So it pays where it was aimed:** 6 to 13% off the far marks and the horizon's worst case, for 0.04 ms at the
  cheapest. Kept for now; its keep-or-drop waits on the Intel flight, as agreed.

## Step 0's Intel flight: which kind of card it is  *(2026-11-13)*

Build 143 on the laptop's Intel card (Optimus, `WGPU_ADAPTER_NAME=intel`), 07:42 to 07:48,
`kept logs/flight-1794577731.txt`. LibreHardwareMonitor wasn't running, so no watts.

| median ms | the far eye whole | the 8 × 8 | the 4 × 4 | NVIDIA's whole (build 143) |
|---|---|---|---|---|
| on the ground at the spawn | 5.94 | 4.38 | 0.09 | 0.71 |
| the bare plain | 6.65 | 5.09 | 0.09 | 0.95 |
| low over the sea | 8.58 | 4.26 | 2.85 | 2.24 |
| high over the spawn | 10.10 | 5.31 | 3.32 | 1.65 |
| the mountain top | 10.50 | 3.64 | 5.37 | 1.68 |
| the sea, nodding, 99th | 9.78 | | | 2.72 |

- **The Intel card is working, not waiting.** Its cost follows how much there is to march, not the longest ray:
  - The spawn's 8 × 8, whose longest ray is 137 steps, costs 4.4 ms, as much as the sea's with 558.
  - The dearest marks are the mountain top and high over the spawn, where the most rays run long (2,356 rays over
    256 steps in the mountain top's 4 × 4), not the sea, whose few longest rays set the NVIDIA's time.
  - It's 4 to 8 times the NVIDIA's time, the most where the NVIDIA is cheapest.
- **So the two cards want different things,** as the Oracle warned. What adds rays for nothing on the NVIDIA (a finer
  base layer, layers side by side) costs the Intel in full. What cuts work (cheaper steps, fewer rays earned,
  shuffling so no lane idles) helps the Intel and less so the NVIDIA.
- **Memory:** 933 MB in use, as on the NVIDIA.
- Next: build 142 against 143 on this card, for 1a's keep-or-drop.

### 1a on the Intel card: build 142 against 143

Flown 07:52 to 08:18, alternating twice: 142 `kept logs/flight-1794578282.txt` and `-1794579087`; 143 `-1794578685` and
`-1794579480`.

| the far eye whole, median ms | 142 | 143, kept in hand | on Intel | on the NVIDIA |
|---|---|---|---|---|
| on the ground at the spawn | 5.811, 5.808 | 5.949, 5.961 | +2.5% | +5% |
| the bare plain | 6.322, 6.314 | 6.679, 6.671 | +6% | −10% |
| the mountain top | 10.219, 10.218 | 10.924, 10.933 | +7% | −6% |
| high over the spawn | 9.557, 9.852 | 10.502, 10.501 | +7 to 10% | −6% |
| low over the sea | 8.324, 8.778 | 8.655, 8.975 | +2 to 4% | −13% |
| a player's run | 9.073, 9.074 | 9.615, 9.554 | +6% | −9% |

- **On the Intel card 1a is slower at every mark,** by 2.5 to 10%. On the NVIDIA it's 6 to 13% faster at the far marks.
- **It's the Oracle's warning, measured:** "on a card that's working, every number held is room lost". The march as
  built keeps about forty numbers in hand; a working card fits fewer rays in flight for it, and loses more by that than
  it gains from the reads saved.
- **Not settled by this:** whether a trimmer 1a, keeping only the handful a skimming step needs (the Oracle's own
  words), pays on both. On Optimus both cards can be flown in one session, so that can be tried without a restart.

### A trimmer 1a: the pass mark (written before the code)

David (2026-11-13): "I'll let you try 3, but i lean towards 1. Keep it. i don't expect someone to be able to play this
game on an igpu without turning some settings way down ... so try the trimmed version, but unless its just better than
1, we go 1."

- **Trimmed:** the next level's numbers (the hand-over reads them plainly, as before 1a), the skipping's second row
  (the level's own, taken from the blend's numbers when they're that level's), and the brick's slot (read each step).
  About two dozen numbers in hand where 1a keeps about forty.
- **The picture pixel for pixel the same** at five marks over the whole world.
- **Flown against build 143 on both cards in one session** (Optimus: the NVIDIA's numbers are "NVIDIA on Optimus",
  fair between builds, not set beside earlier NVIDIA flights or the gate).
- **David's rule:** kept only if it's just better than 1a: no slower on the NVIDIA at the far marks, and faster on the
  Intel. Otherwise 1a stays as built.
- Builder's guess [low]: on the NVIDIA it gives back a third of 1a's gain; on the Intel it recovers half of 1a's
  loss. So it likely loses by David's rule.

### Results: the trimmer 1a against 1a, on both cards

Build 143 (1a) against build 144 (trimmed), 08:29 to 08:56 on Optimus: on the NVIDIA `kept logs/flight-1794580581.txt`
(143) and `-1794580977` (144); on the Intel `-1794581377` (143) and `-1794581774` (144). The NVIDIA's numbers here are
"NVIDIA on Optimus": fair between the two builds, not set beside earlier NVIDIA flights or the gate.

| the far eye whole, median ms | NVIDIA on Optimus: 143, 144 | | Intel: 143, 144 | |
|---|---|---|---|---|
| on the ground at the spawn | 0.690, 0.622 | −10% | 5.933, 5.835 | −2% |
| the bare plain | 0.963, 1.143 | +19% | 6.640, 6.509 | −2% |
| the mountain top | 1.838, 2.073 | +13% | 10.500, 10.071 | −4% |
| high over the spawn | 2.017, 1.897 | −6% | 10.108, 9.781 | −3% |
| low over the sea | 2.424, 2.606 | +8% | 8.562, 8.186 | −4% |
| a player's run | 2.505, 2.786 | +11% | 9.427, 9.151 | −3% |

- **The trimmed version isn't just better.** On the Intel it wins back 2 to 4%, about to where build 142 stood. On the
  NVIDIA it's slower at the plain, the mountain top, the sea and a player's run by 8 to 19%, losing 1a's gain there
  and more. One flight a build, but the differences are many times a flight's spread.
- **By David's rule, 1a stays as built.** The trim is taken out (build 145 is 143's march again); git keeps it at
  build 144.
- **What it taught:** the brick's slot and the next level's numbers kept in hand are what the NVIDIA's gain is made
  of, and what the Intel's room is spent on. There's no one version for both; a setting for weaker cards could choose
  the plain march later, if it's ever wanted.

## The Oracle's step 1b: read together  *(plan 2.2)*

### The pass mark (written before the code)

- **What's built,** behind a switch (`INCUS_FAR_TOGETHER`: 1 on, 0 off; off till flown):
  - A blend fetches how solid, the colour and both faces' pictures at once, with no `if` between them; what's used of
    them is chosen after, by the very tests as before.
  - Over a hand-over, both levels' places are found first, then both levels' pictures fetched together.
- **The picture pixel for pixel the same,** on and off, at five marks over the whole world. The longest rays the same.
- **Flown on and off on both cards in one session** (Optimus: direction and the Intel's price). Its keep-or-drop comes
  from an "NVIDIA GPU only" flight after David switches back (Counsel).
- **Estimates.** The Oracle's [low]: a further 15 to 40% off the open-horizon marks, nothing or a small loss at the
  spawn. Builder's [low]: 5 to 15% off the far marks on the NVIDIA, since most of a long ray's steps are skips that
  read no pictures at all; on the Intel 5 to 15% slower.

### Results: read together, off and on, on both cards (on Optimus)

Build 146, 09:36 to 10:03: the NVIDIA on Optimus off `kept logs/flight-1794583011.txt`, on `-1794583397`; the Intel off
`-1794583796`, on `-1794584192`. One flight each.

| the far eye whole, median ms | NVIDIA on Optimus: off, on | | Intel: off, on | |
|---|---|---|---|---|
| on the ground at the spawn | 0.694, 0.686 | −1% | 6.042, 6.007 | −1% |
| the bare plain | 0.985, 0.958 | −3% | 6.812, 6.781 | 0% |
| the mountain top | 1.880, 2.034 | **+8%** | 10.891, 10.697 | −2% |
| high over the spawn | 2.008, 1.816 | −10% | 10.428, 10.169 | −2% |
| low over the sea | 2.494, 2.381 | −5% | 8.854, 8.677 | −2% |
| a player's run | 2.573, 2.447 | −5% | 9.732, 9.628 | −1% |
| the sea, nodding | 2.464, 2.352 | −5% | 8.783, 8.558 | −3% |

- **A small gain, not the large one guessed.** The Oracle gave a further 15 to 40% off the open-horizon marks; it's 3
  to 5% there. Builder gave 5 to 15% on the NVIDIA: met only high over the spawn (−10%).
- **The mountain top is 8% slower on the NVIDIA.** Not explained yet; it's the mark with the fog off and the most
  rays in its 4 × 4.
- **The Intel doesn't pay, against both guesses:** 0 to 3% faster with it on.
- **Why so little, most likely:** a long ray's steps are mostly skips, which read bands and no pictures at all, so
  there's little for reading together to shorten. By the Oracle's own test ("if 1b gives nothing, the card doesn't
  overlap reads as I believe, and 1c won't pay either") this is between: something, not much.
- **Its keep-or-drop** waits on the "NVIDIA GPU only" flights after David switches back.

### Results: read together on "NVIDIA GPU only", and its verdict

Build 146, 10:16 to 10:43, off, on, off, on: off `kept logs/flight-1794584998.txt` and `-1794585787`; on `-1794585390`
and `-1794586177`.

| the far eye whole, median ms | off (1a alone) | on (1a and 1b) | |
|---|---|---|---|
| on the ground at the spawn | 0.703, 0.772 | 0.703, 0.696 | the same |
| the bare plain | 0.974, 0.973 | 0.949, 0.951 | −2.5% |
| the mountain top | 1.753, 1.760 | 1.631, 1.624 | −7% |
| high over the spawn | 1.693, 1.694 | 1.651, 1.653 | −2.5% |
| low over the sea | 2.329, 2.331 | 2.220, 2.211 | −5% |
| a player's run | 2.426, 2.423 | 2.312, 2.300 | −5% |
| the plain, nodding, 99th | 1.054, 1.056 | 1.029, 1.023 | −3% |
| the sea, nodding, 99th | 2.830, 2.853 | 2.728, 2.713 | −4% |

- **Kept:** 2.5 to 7% off every far mark, both pairs agreeing, nothing slower; the Intel lost nothing. Read together
  is on from build 147 (`INCUS_FAR_TOGETHER=0` reads in turn, as before).
- **The mountain top's 8% loss on Optimus didn't hold:** here it's the best gain. Counsel's caution about Optimus
  numbers was right.
- **Against the guesses:** under both (the Oracle's 15 to 40% at the horizon marks, Builder's 5 to 15%).
- **Against the gate** (1 ms typical, 2 worst): the sea stands at 2.21 ms, its nodding 99th at 2.72; the plain 0.95;
  the mountain top and high over the spawn 1.63 and 1.65.
- **Today reads a little slower than last night** for the same code (1a alone: the sea 2.33 against 2.24, the mountain
  top 1.76 against 1.68). And **the fast run reads double** in every pass, off and on alike, with the card's clock down
  to 1,000 to 1,240 MHz there at 52 to 54 °C (last night 2,500 MHz at 71 °C): the card slowed itself, not by heat.
  Not from the code; asked of David whether the laptop's power mode is as it was.

## The Oracle's step 1c: several steps at once  *(plan 2.2)*

### The pass mark (written before the code)

- **What's built,** behind a switch (`INCUS_FAR_AHEAD`: 1 on; off till flown):
  - After a step through open air, the ray works out its next three places as it would reach them (the same strides,
    by the same sums) and fetches all three's pictures together.
  - Each later step that arrives at exactly one of those places, at the same level, uses what was fetched; the march's
    own tests still run every step. If the ray skips, changes level, meets a surface or a region not summed, what was
    fetched ahead is dropped and the step reads as before.
  - So it's the same places, the same sums and the same steps; only when the pictures are asked for changes.
- **The picture pixel for pixel the same,** on and off, at five marks over the whole world. **The longest rays the
  same** step for step.
- **Flown off and on, alternating twice,** on "NVIDIA GPU only". Decided on the NVIDIA; the Intel's price taken at
  its next flight (it will pay: fetches thrown away are work).
- **Estimates.** The Oracle's [low]: a further 20 to 50% off what the long layers still cost. Builder's [low]: 5 to
  20% off the far marks. 1b showed the card overlaps reads only a little, and only the steps that blend (not the
  skips) can be fetched ahead.

### Results: several steps at once, and its verdict

Build 148 on "NVIDIA GPU only", 10:59 to 11:26, off, on, off, on: off `kept logs/flight-1794587590.txt` and
`-1794588369`; on `-1794587981` and `-1794588757`.

| the far eye whole, median ms | off | on | |
|---|---|---|---|
| on the ground at the spawn | 0.866, 0.854 | 0.866, 0.867 | the same |
| the bare plain | 1.051, 1.047 | 1.251, 1.257 | +19% |
| the mountain top | 1.854, 1.854 | 2.225, 2.238 | +20% |
| high over the spawn | 1.904, 1.903 | 2.110, 2.113 | +11% |
| low over the sea | 2.215, 2.211 | 2.682, 2.703 | +22% |
| a player's run | 2.286, 2.287 | 2.840, 2.868 | +25% |

- **Dropped: 11 to 25% slower at every far mark,** both pairs agreeing; the longest rays the same step for step. The
  Oracle's own test said it: "if 1b gives nothing, the card doesn't overlap reads as I believe, and 1c won't pay
  either". 1b gave a little; 1c pays for fetches the card doesn't overlap, and for those thrown away.
- **Its code cost the march even switched off:** against build 147 the same day, the spawn 0.86 against 0.70, the
  mountain top 1.85 against 1.63, the plain 1.05 against 0.95 (the sea the same, 2.21). So it's taken out whole
  (build 149's march is build 147's); git keeps it at build 148.
- **The fast run read normally again** in these flights (the card's clock at 2,540 to 2,590 MHz, 1.60 ms): the
  morning's halved clock was not the code, and passed with nothing changed (Counsel's second guess: Windows finishing
  its update).

### A correction: every flight so far flew with fog on  *(2026-11-13)*

Found while taking 1c out: a mark's `fog` line was printed in each report ("fog on", "fog off") but never applied. The
far eye's own look had fog on, so every flight and every test picture before build 149 was fogged at every mark, the
mountain top "on a clear day" among them. Comparisons between builds stand (both sides were fogged alike); the labels
were wrong.

- **From build 149 fog is off by default** [David, 2026-11-13: "off by default would be better. it looks nice but is
  most likely reserved for final touches"; and the rule: "we are not using fog or haze or any sort of cover as a
  performance tool. it is just for looks"]. The Render page's Fog section turns it on. Reports say in their head
  whether fog was on, from the far eye's look; the per-mark label is gone.
- **The window's picture check, done again with fog truly off** (640 × 400, 13 × 13 regions, the whole world against
  a window of 3): the mountain top 7.2% of pixels differ, by 39 of 255 at most (0.5% by more than 8); high over the
  spawn 5.5%, by 24; low over the sea 1.0%, by 77 at a few. About what the fogged check found (6.5%, 4.5%, 0.4%). So
  the window's softer far ridges are there without fog too, and David's look at them fog-off is still owed: his look
  at build 134 was with fog on.

### The baseline with fog off (build 149)

One flight, 11:42 to 11:49, `kept logs/flight-1794589790.txt`: the march is build 147's (1a and 1b), fog off.

| the far eye whole, ms | median | 99th |
|---|---|---|
| on the ground at the spawn | 0.697 | 0.733 |
| the bare plain | 0.942 | 0.952 |
| the mountain top | 1.592 | 1.620 |
| high over the spawn | 1.637 | 1.680 |
| low over the sea | 2.201 | 2.229 |
| a player's run | 2.275 | 2.463 |
| the plain, nodding | 0.925 | 1.013 |
| the sea, nodding | 2.142 | 2.688 |

- **1c's code out, the march is back** where build 147 stood (the spawn 0.70, the mountain top 1.59 against 1.63).
- **Fog cost 1 to 2%** (the sea 2.21 with it, 2.20 without; the mountain top 1.63 and 1.59).
- **Against the gate** (1 typical, 2 worst): the sea 2.20, its nodding 99th 2.69.
- David on the window's far ridges without fog (via Counsel): "i know what it looks like without fog its fine." His
  look is given.
- From build 150 a standing's heat line says how long the card's clock read under 2,000 MHz, not only its least (the
  Oracle: the least can't tell a blink from a long stretch).

## The Oracle's step 1d: thin haze by whole cells  *(plan 2.2, a look)*

David, asked after "haze" was explained to him (the land's faint fringe, where the blend is 2% solid or less): "sounds
like it might actually look better, lets try it".

### The pass mark (written before the code)

- **What's built:** where a ray is in thin haze, its step is a whole cell, not half (`INCUS_FAR_HAZE_STEP`, in cells:
  0.5 as it was, 1 the new; 0.5 till David has looked).
- **The picture changes,** and that's the point of looking: pictures at the sea, the plain and high over the spawn,
  side by side and their difference counted, then David's eye on the game.
- **The longest rays get shorter.** The Oracle: "it halves the very longest rays". Builder [low]: the longest by 20
  to 40%, the far marks 5 to 15% faster, the most over the sea.
- **Flown at 0.5 and 1, alternating twice.** Kept only if David likes the look.

### Results: thin haze by half and whole cells

Build 151, fog off, 11:59 to 12:25, alternating twice: half a cell `kept logs/flight-1794591158.txt` and
`-1794591938`; a whole cell `-1794591548` and `-1794592330`. Watts and load are back in the reports (the processor 7
to 8% loaded, the card 87 to 98 W: the machine wasn't shared).

| the far eye whole, median ms | half a cell | a whole cell | |
|---|---|---|---|
| on the ground at the spawn | 0.699, 0.703 | 0.694, 0.697 | −1% |
| the bare plain | 0.946, 0.947 | 0.878, 0.879 | −7% |
| the mountain top | 1.599, 1.604 | 1.569, 1.569 | −2% |
| high over the spawn | 1.640, 1.645 | 1.612, 1.612 | −2% |
| low over the sea | 2.222, 2.221 | **1.777, 1.776** | −20% |
| a player's run | 2.293, 2.296 | 1.996, 1.998 | −13% |
| the plain, nodding, 99th | 1.020, 1.025 | 0.928, 0.929 | −9% |
| the sea, nodding, 99th | 2.712, 2.720 | 2.139, 2.133 | −21% |

- **The sea is under the gate's 2 ms worst for the first time** (1.78, from 4.01 on the 12th), and a player's run at
  2.00. The sea's nodding 99th, 2.14, is still over it.
- **The longest rays are shorter, but not halved** (the Oracle's guess): the sea's 558 and 764 steps to 532 and 668;
  the plain's 467 to 447. The time fell more than the steps, since the steps saved are the dear ones, each a blend.
- **Builder's guess** (the far marks 5 to 15% faster, the most over the sea): the sea beat it at 20%; the mountain top
  and high over the spawn came under it at 2%.
- **The pictures:** nearly the same (above). It waits on David's look in the game.

**David's look at build 151 with whole-cell steps (2026-11-13):** "i don't see anything bad about it. Seems like a good
thing." Kept: thin haze is stepped by whole cells from build 152 (`INCUS_FAR_HAZE_STEP=0.5` for old's).

**A flaw he saw in the same look, written down for the polish, not for now:** close to the ground the far eye's
picture is "splotchy": lumps with faceted, ring-like outlines and faint diagonal streaks. His question: "what can be
done about how rays are read that could make the look more 'orderly' in its noise ... if there's some way to make it
'even'. it could also just be the terrain being messy." Builder's reading of what it is, by its parts:
- the straight blend between cells (creases along the cells' grid, faceted lumps);
- the faces' light taken from each cell's stored shares (a patch of its own brightness a cell);
- the steps' places (rings where they line up, streaks from the hash that offsets them);
- the 8 × 8 squares brought up crisp.
The candidates for the polish: a rounder blend (smooth weights), light from the slope of the solid field itself, an even
(blue-noise) offset for the steps, a finer base layer. Much of this distance is the near eye's to draw.

### The polish passes, noted for the end  *(David, 2026-11-13)*

David: "please make a note of those polishing step passes where we can try to work the rays into a prettier shape
without doing much ... the 'card is waiting not working' has made me think that if we go through our whole plan and it
comes off how we hope, we may have a lot of headroom inside the card for making things prettier."

Each is a look step with its own pass mark and David's eye, tried one at a time once the far eye meets its gate:

1. **A rounder blend.** The cells blended with curved weights, not straight ones, so the creases along the cells'
   grid go and lumps come out round. The texture hardware's blend is straight; the curve can be put on the place
   asked for (one fetch still), or made by more fetches. The largest change to the look for the least code.
2. **Light from the shape itself.** A surface's brightness from the slope of the blended solid field at the point,
   not from each cell's stored face shares, so it changes smoothly across cells. A few more fetches a surface step.
3. **An even offset for the steps.** Blue noise in place of the hash that shifts each pixel's steps: the same amount
   of noise, with no streaks or clumps. A small picture and one read a pixel.
4. **A finer base layer.** More rays where the card is idle (93 to 99% of its room free in the long passes). Tried
   first as step 2's probe; here for its look.
5. **The fog line and skyline squares,** if they still show once 4 is in (the plan's own line).

Not to be mixed with the speed steps: each changes the picture, so none is compared pixel for pixel.

## The Oracle's step 2: the layers side by side. First, its probe: a finer base layer  *(plan 2.2)*

David's go (via Counsel, 2026-11-13): "alright, tell builder to proceed as indicated."

### The probe's pass mark (written before the code)

- **What's built:** the base layer's size as a launch setting (`INCUS_FAR_BASE`: 8 as it is, or 4), the layers under
  it each half the one above (8, 4, 2; or 4, 2, 1). Bringing it up reads the sizes it's given. The reports' pass
  names stay "8 x 8, 4 x 4, 2 x 2" and mean the first, second and third layer; the report's head says the sizes.
- **At 8 the picture is pixel for pixel the same** (nothing but where the sizes come from has changed).
- **At 4 the picture changes:** every pixel of the 4 × 4 is marched, not only where the 8 × 8 said it was wanted. A
  picture beside the 8's, for the record; David's look when it matters (it's also the polish list's fourth pass).
- **The question it answers** (the Oracle's): "If a 4 × 4 base alone costs about what the 4 × 4 layer does today, the
  two waits were adding." Then the layers side by side will pay, by about the 8 × 8's time where both are long.
- **Flown at 8 and 4, alternating twice.** Builder's guess [low]:
  - Where both layers run long (the sea, high over the spawn, the mountain top): the far eye 25 to 40% faster at 4.
  - The plain and the spawn, where today only the 8 × 8 marches: slower at 4, by up to half again, since four times
    the rays must fit through the card where today's layer already half fills it at the spawn.
  - The Intel card would pay in full (four times the base's rays); not flown now.

### The probe's result: missed, both ways; the base stays at 8  *(build 153, f20653a; NVIDIA only, fog off)*

Flown 8, 4, 8, 4 (logs/flight-1794595042, -5434, -5828, -6222); the two flights of each agree within 1%. The card's
whole, median ms (the first, second and third layer's card time in brackets):

| mark | base 8 | base 4 | |
|---|---|---|---|
| on the ground at the spawn | 0.69 (0.53, 0.01, 0.04) | 1.90 (1.59, 0.04, 0.14) | 174% slower |
| the bare plain | 0.88 (0.73, 0.01, 0.04) | 1.68 (1.37, 0.04, 0.14) | 91% slower |
| the mountain top | 1.57 (0.66, 0.76, 0.04) | 3.05 (1.44, 1.32, 0.15) | 94% slower |
| high over the spawn | 1.61 (0.81, 0.65, 0.04) | 3.28 (1.98, 1.03, 0.14) | 103% slower |
| low over the sea | 1.78 (0.70, 0.93, 0.04) | 2.11 (1.34, 0.50, 0.14) | 19% slower |
| a player's run over the plain | 2.00 (0.91, 0.91, 0.04) | 2.70 (1.38, 1.04, 0.15) | 35% slower |
| fast across the world | 1.20 (0.41, 0.63, 0.04) | 2.98 (1.40, 1.28, 0.16) | 149% slower |
| the sea, nodding (99th) | 1.79 (2.15) | 2.24 (2.58) | 25% slower |

- **The guess missed in the open.** "25 to 40% faster where both layers run long": slower at every one of them (19 to
  103%). "The plain and the spawn slower by up to half again": slower by 91 and 174%.
- **What it says. A layer's time is not only its longest ray's wait: the number of rays counts.** Four times the
  base's rays cost 1.9 times (the plain, the sea) to 3 times (the spawn) the base's time. So at this size the card
  isn't only waiting; "waiting, not working" holds for the layers as they are, and stops holding somewhere between
  64,000 rays and 256,000.
- **The Oracle's question** ("if a 4 × 4 base alone costs about what the 4 × 4 layer does today, the two waits were
  adding"): it costs more, 1.4 times (the sea) to 3 times (high over the spawn) today's 4 × 4 layer. But it marches
  every pixel where today's marches only where the 8 × 8 said it was wanted, so the probe doesn't answer it cleanly.
  What it does show: the 8 × 8's culling earns its place, and marching fewer rays is worth more than one fewer wait.
- **A layer with nothing to march still costs by its pixels:** the third layer marched no ray in either flight; 0.04 ms
  at 2 × 2 (a million pixels), 0.14 ms at 1 × 1 (four million).
- **Not kept as the base; the switch stays** (`INCUS_FAR_BASE=4`) for the polish list's fourth pass, a look only, and
  now priced: 0.3 to 1.7 ms.
- **For the layers side by side** (step 2 proper): still untested directly. Builder's estimate comes down [low]: the
  layers marched at once will share a card that this probe shows is partly working, so less than the Oracle's 30 to
  45% where both run long.

**[LATER] The thin blue line of sea on the horizon at "low over the sea" is there at 8 and gone at 4** (352 sea-blue
pixels near the horizon against none). David, watching the flights: "i didnt notice anything". Cause unknown. Ruled
out, each by a picture that didn't change: the step cap (3072 for 768), the sky's weight (0 for 4), the rays' end at
the map's edge (off). Not known either which of the two pictures is right. To find before a finer base is ever kept.

## The Oracle's step 2 proper: the layers side by side  *(plan 2.2)*

### The pass mark (written before the code)

- **What's built:** behind a launch setting (`INCUS_FAR_SIDE=1`), the three layers marched in one pass as three
  dispatches, none reading what another writes that frame. A finer layer takes its lead (where it's wanted, where
  its rays start, the sky light handed down) from the coarser layer **of the frame before**: each layer's "worth it"
  and distances are kept twice and swapped each frame. Unset, the layers go one after another as now.
- **Unset, the picture is pixel for pixel build 153's.**
- **Set, standing still, the picture is pixel for pixel the same as unset** from the third frame on (the lead a
  frame old is the same lead). Moving, a finer layer's lead lags one frame: where the finer layers march trails the
  view by a frame at edges. The first frame after the window's size changes is the coarse layer alone. Both for
  David's look, turning fast with the frames capped (a frame's lag is 4 ms at 240, 17 ms at 60).
- **The card's memory:** about 27 MB more at 2560 × 1600 (the second copies), only when set.
- **The reports:** when set, the three layers' time is all under the first layer's line ("the far 8 x 8"); the other
  two read near nothing. The card's whole is what's compared.
- **Flown unset, set, unset, set.** Builder's guess [low], lowered by the probe:
  - Where two layers run long (the sea, high over the spawn, the mountain top, the player's run): the card's whole
    10 to 25% less. Nothing at all is possible: if the card's driver still runs the dispatches one after another.
  - The plain and the spawn (one layer marching): within 2% either way.
- **Kept if** the long marks gain 10% or more, nothing else is more than 2% slower, and David sees no lag. If it's
  kept, the lag gets its own look before anything is built on it.

### The result: nothing gained; the card ran the three dispatches one after another  *(build 154, e5c4232; NVIDIA only, fog off)*

Flown unset, set, unset, set (logs/flight-1794597712, -8104, -8498, -8894); each pair agrees within 1%. The card's
whole, median ms; in brackets the layers' card time (three passes unset, the one pass set):

| mark | one after another | side by side | |
|---|---|---|---|
| on the ground at the spawn | 0.70 (0.54 + 0.01 + 0.04) | 0.70 (0.59) | the same |
| the bare plain | 0.88 (0.72 + 0.01 + 0.04) | 0.88 (0.78) | 0.5% slower |
| the mountain top | 1.57 (0.66 + 0.76 + 0.04) | 1.58 (1.46) | 0.6% slower |
| high over the spawn | 1.61 (0.81 + 0.65 + 0.04) | 1.53 (1.42) | 5% faster |
| low over the sea | 1.78 (0.70 + 0.93 + 0.04) | 1.79 (1.68) | 0.5% slower |
| a player's run over the plain | 1.99 (0.91 + 0.90 + 0.04) | 1.99 (1.88) | the same |
| the sea, nodding (99th) | 1.79 (2.13) | 1.79 (2.13) | the same |

- **Against the pass mark:** the guess was 10 to 25% at the long marks, "nothing at all is possible". It was nothing
  (one mark 5%). Not kept: the mark to keep it was 10%.
- **What it says: the one pass's time is the three layers' times added.** Freed of each other's results, the
  dispatches still ran in turn. So the two waits do add, as the Oracle supposed, but putting the dispatches in one
  pass isn't enough to overlap them.
- **Not known which holds them in turn:** the card's driver finishing one dispatch before the next, or the
  library's own barriers between dispatches (it moves each layer's picture and distances from "read" to "written"
  just before that layer's dispatch). Not separated here.
- **What would overlap them for certain, untried:** one dispatch that marches all three layers' pixels (each
  invocation working out which layer it belongs to). It needs six written textures in one pass, over the library's
  default of four (the cards have more; the limit would be raised, as was once done and undone for the buffers), or
  the layers packed into shared textures. The frame-before lead built here is what it would stand on.
- **The lag was not looked at:** nothing earned David's look. The code stays behind its switch (unset costs nothing,
  the same picture and resources as before) until it's decided whether the one-dispatch form is tried; then out or on.

## Step 2, the one-dispatch form: all three layers' pixels marched by a single dispatch  *(plan 2.2)*

David (2026-11-13), on the choice between this and going straight to the gate: "go on with a".

### The pass mark (written before the code)

- **What's built:** behind the same launch setting at 2 (`INCUS_FAR_SIDE=2`), one dispatch whose every invocation
  works out which layer's pixel it is (the finest at the left of the grid, the middle layer right of it, the base
  under that) and marches it. The lead is the frame before's, as built for build 154. Nothing between the layers'
  work for the card or the library to wait on.
- **A limit raised on purpose, only when set:** six written textures in one pass (a picture and distances a layer)
  against the library's default of four. If it's kept, the layers get packed to come back under the default, or
  the raise is decided on then; not before.
- **Unset, and at 1, the picture is pixel for pixel build 154's;** the march's code is moved into a function both
  forms call, so unset's times are checked against build 154's too (within 1%).
- **At 2, standing still, the picture is pixel for pixel the same from the third frame.** The lag when moving is
  the same one frame as at 1.
- **Flown unset, 2, unset, 2.** Builder's guess [low]: the probe says the card is partly working, the last flight
  says the dispatches ran in turn, and this is the first time the long rays of two layers are on the card at once.
  - Where two layers run long (the sea, the mountain top, high over the spawn, the player's run): the card's
    whole 15 to 35% less. Nothing is still possible: if a layer's time is all work and no wait.
  - The plain and the spawn: within 3% either way (each invocation now copies its layer's numbers first).
- **Kept if** the long marks gain 10% or more, nothing else is more than 3% slower, and David sees no lag when it's
  put to his look. If nothing: step 2 is read, both forms come out, and the gate (step 3) is next.

### The result: the waits do overlap, 33 to 38% at two marks; but it costs 0.05 to 0.2 ms everywhere else  *(build 155, cf3d3ea; NVIDIA only, fog off)*

Flown unset, 2, unset, 2 (logs/flight-1794600304, -0705, -1098, -1493); each pair agrees within 0.5%. The card's
whole, median ms:

| mark | one after another | by one dispatch | |
|---|---|---|---|
| on the ground at the spawn | 0.69 | 0.85 | 23% slower |
| the bare plain | 0.88 | 1.00 | 14% slower |
| the mountain top | 1.57 | 1.78 | 13% slower |
| high over the spawn | 1.61 | 1.66 | 3% slower |
| low over the sea | 1.78 | 1.11 | **38% faster** |
| a player's run over the plain | 1.99 | 1.34 | **33% faster** |
| fast across the world | 1.19 | 1.63 | 37% slower |
| the plain, nodding (99th) | 0.87 (0.93) | 1.00 (1.06) | 14% slower |
| the sea, nodding (99th) | 1.79 (2.14) | 1.12 (1.30) | **37% faster (39%)** |

- **Unset is build 154's** (0.69, 0.88, 1.57, 1.61, 1.78, 1.99): moving the march into a function cost nothing.
- **Against the pass mark:** "15 to 35% where two layers run long": the sea and the player's run, yes and a little
  over (38, 33%); the mountain top and high over the spawn, no (13 and 3% slower). "The plain and the spawn within
  3%": missed, 14 and 23% slower. **By the mark it isn't kept as it is** (something else more than 3% slower).
- **What it says, the first time it's been seen: two layers' waits can be overlapped, and it is worth what the Oracle
  said** (30 to 45%): at the sea and on the run, where each layer's time is a few long rays, the one dispatch costs
  about the longer of the two, not their sum. And build 154's nothing was the dispatches kept in turn, not the waits.
- **Where each layer's time is many rays (the mountain top, high over the spawn), nothing overlaps:** the times
  add as before. With the probe: a layer's time is part wait (few long rays; overlaps) and part work (many rays; adds).
- **And a cost of its own, 0.12 to 0.16 ms at the marks where one layer marches** (the spawn, the plain), about the
  same at the mountain top, 0.45 flying fast. Not yet found which of these it is: each invocation's copy of its
  layer's numbers before anything else (a million and a third of them, most turned away at once); the grid's empty
  part (about a third more invocations than pixels); the choosing between three textures at each store.
- **Against the gate as it stands** (1 typical, 2 worst): one after another, typical met (0.69, 0.88), worst missed
  by a hair (the sea nodding's 99th 2.14). By one dispatch, worst met with room (the most 1.78, its 99th 1.81), typical
  on the line (0.85, 1.00).
- **The lag was not looked at yet.**

### The one-dispatch form trimmed: the pass mark (written before the code)

- **What's changed, in the one-dispatch form only** (unset and 1 untouched, their shader's text the same):
  1. No copy: each use of the layer's numbers reads them in place from that layer's row, where build 155 copied all
     eighteen rows into the invocation's own before anything else, the turned-away ones too.
  2. The grid turned round: the base and under it the middle layer at the left, the finest to their right, so that
     if the card takes the grid in order, the layers with the long rays start first.
- **The picture at 2, standing still, stays pixel for pixel the same from the third frame.**
- **Flown unset, 2, unset, 2;** the trimmed form is read against build 155's own (sessions have agreed within 0.5%
  all day). Builder's guess [low]:
  - The form's own cost at least halves: the spawn 0.77 or less (build 155: 0.85; in turn: 0.69), the plain 0.94 or
    less (1.00; 0.88), the mountain top 1.68 or less (1.78; 1.57).
  - The sea and the player's run keep their gain: within 3% of 1.11 and 1.34.
- **Kept if** nothing is more than 3% slower than one after another; the guess above wouldn't quite make that at the
  spawn, so a second trim (the grid's empty part, the three-way stores) may follow, or David's choice between the
  two ends of the gate.

### The trim's result: its own cost gone; faster or level at every mark, everything under 2 ms  *(build 156, 89bf7b1; NVIDIA only, fog off)*

Flown unset, 2, unset, 2 (logs/flight-1794602542, -2949, -3344, -3740; their headers name 904bd76, Counsel's
docs-only commit at take-off, the same code); each pair agrees within 0.5%. The card's whole, median ms:

| mark | one after another | one dispatch, build 155 | trimmed, build 156 | trimmed against in turn |
|---|---|---|---|---|
| on the ground at the spawn | 0.69 | 0.85 | 0.68 | 2% faster |
| the bare plain | 0.88 | 1.00 | 0.88 | level |
| the mountain top | 1.57 | 1.78 | 1.38 | 12% faster |
| high over the spawn | 1.61 | 1.66 | 1.37 | 15% faster |
| low over the sea | 1.78 | 1.11 | 1.41 | 21% faster |
| a player's run over the plain | 2.00 | 1.34 | 1.44 | 28% faster |
| fast across the world | 1.18 | 1.63 | 1.21 | 2.5% slower |
| the sea, nodding (99th) | 1.79 (2.14) | 1.12 (1.30) | 1.32 (1.61) | 26% faster (25%) |

- **Against the pass mark:** "its own cost at least halves" (the spawn 0.77 or less, the plain 0.94, the mountain top
  1.68): better than that, it's gone (0.68, 0.88, 1.38). "The sea and the run keep their gain within 3%": missed, the
  sea gave back about half of it (1.11 to 1.41), the run a little (1.34 to 1.44). "Kept if nothing is more than 3%
  slower than one after another": met (the most, flying fast, 2.5%).
- **David's bar** (via Counsel, on build 155's numbers: "if everything in the far view is under 2ms we should be
  gold ... the worst frames or irregular frames were more critical than longest frames"): every mark's median and
  99th is under 2 (the most: flying fast's 99th 1.78; at a player's pace the run's 1.62).
- **What it says:** the copy was the form's own cost (two changes went in together, but only the copy touches the
  marks where one layer marches). **And the grid's order matters, both ways:** with the base before the middle
  layer, the mountain top and high over the spawn now overlap (12 and 15%, nothing in build 155) and the sea
  overlaps less. Build 155's order had the middle layer's rows before the base's. Not yet tried: the base and the
  middle layer side by side in the same rows, so neither is first.
- **Kept by the mark, to go on with,** still owing before anything stands on it: David's look at the one-frame
  lag while turning, and the six written textures packed back under the library's default.

### The one dispatch's grid, a third arrangement: the pass mark (written before the code)

David (2026-11-13), on build 156's numbers: "hell to the yeah claude! ... we've really made strides. im not entirely
sure i'll be able to see the lag without the near view present. You can go ahead with finishing up and then cutting
code we won't be using and see if that gives us another 1% just for good measure. then i'll took a look and zoom
around a bit."

- **What's changed, in the one-dispatch form only:** the base, the middle layer and the finest side by side across
  the grid, all starting at its top row, so the base's rows and the middle layer's are the same rows and neither
  goes first (build 155: the middle layer's rows first; build 156: the base's first).
- **The picture at 2, standing still, stays pixel for pixel the same from the third frame.**
- **Flown unset, 2, unset, 2,** read against build 156's trimmed numbers. Builder's guess [low]:
  - The sea gets back some of build 155's: 1.25 or less (156: 1.41; 155: 1.11). The player's run 1.40 or less (1.44).
  - The mountain top and high over the spawn hold: within 3% of 1.38 and 1.37.
  - The spawn and the plain within 2% of 0.68 and 0.88.
- **Kept if** the sea gains 5% or more on build 156 and no mark is more than 2% slower than build 156; else build
  156's order stays.

### The third arrangement's result: a little better everywhere, less than the mark asked  *(build 157, de5d9b5; NVIDIA only, fog off)*

Flown unset, 2, unset, 2 (logs/flight-1794604653, -5048, -5439, -5837). The card's whole, median ms:

| mark | one after another | build 156 (base first) | build 157 (side by side) | 157 against 156 |
|---|---|---|---|---|
| on the ground at the spawn | 0.70 | 0.68 | 0.68 | level |
| the bare plain | 0.88 | 0.88 | 0.88 | level |
| the mountain top | 1.57 | 1.38 | 1.37 | 1% faster |
| high over the spawn | 1.62 | 1.37 | 1.32 | 3% faster |
| low over the sea | 1.78 | 1.41 | 1.38 | 2% faster |
| a player's run over the plain | 2.00 | 1.44 | 1.41 | 2% faster |
| fast across the world | 1.19 | 1.21 | 1.21 | level |
| the sea, nodding (99th) | 1.79 (2.14) | 1.32 (1.61) | 1.27 (1.58) | 3% faster (2%) |

- **Against the pass mark:** "the sea 1.25 or less": missed (1.38). "Kept if the sea gains 5% or more on build 156":
  missed, 2%. The rest held as guessed.
- **By the mark's letter build 156's order would go back in. Builder leaves build 157's in, and says so:** it is
  level or 1 to 3% faster at every mark, slower at none, and no more code. The 5% was mine, set to keep from chasing
  noise; the pairs agree within 0.5%, so 1 to 3% at five marks at once isn't noise. David's or Counsel's to overrule.
- **Not explained:** build 155's sea (1.11) had the middle layer's rows first and the base's after them, and also
  the copy. Neither later order has given it back. Build 155's order without the copy was never flown; by its other
  marks it would have the mountain top and high over the spawn near 1.6, so the most of all the marks would be worse
  than now (1.41). Left, written down.

## The one dispatch finished: packed under the default limit, made the default; build 154's form out  *(plan 2.2)*

### The pass mark (written before the code)

- **What's built:**
  1. The three layers' distances in one texture (the base, the middle layer and the finest side by side in it), so a
     pass writes four textures, the library's default: the raised limit comes out of `card.rs`.
  2. The one dispatch is how the far eye marches unless told otherwise; `INCUS_FAR_SIDE=0` marches the layers one
     after another (kept: every flight is read against it, and it is the picture's reference). Build 154's form
     (three dispatches in one pass) is taken out: it gained nothing.
  3. The first frame after the layers are remade (the window's size changed, the bricks' pool or the far textures
     grew) is drawn whole: the dispatch is run three times in that one frame, so its lead is this frame's and there
     is no frame of the coarse layer alone. A frame's lead after a jump of the eye (going to a mark) is still the
     view before the jump, for that one frame: written down, not built for.
- **The picture:** at `INCUS_FAR_SIDE=0`, pixel for pixel build 157's. Unset, a single frame from new is pixel for
  pixel the same picture (the three runs settle it); so is the third frame of the old test.
- **The card's memory:** up about 15 MB at 2560 × 1600 on build 157's one dispatch (the distances' texture is as
  tall as the finest layer all the way across), less what the layers one after another no longer hold when they
  aren't used; Builder's guess within 20 MB either way of build 157's one-dispatch figure.
- **Flown unset, 0, unset** (one dispatch twice, in turn once to check the session), read against build 157.
  Builder's guess [low]: every mark within 1.5% of build 157's one dispatch, either way. The packing changes where
  the distances are written, not how many.
- **Kept if** no mark is more than 1.5% slower than build 157's. Then the measuring switches are cut (build 159) and
  flown alone, for David's "another 1%".

### The result: the same as build 157, on the default limits  *(build 158, c90ff53; NVIDIA only, fog off)*

Flown the one dispatch, in turn, the one dispatch (logs/flight-1794607032, -7427, -7822). The card's whole, median ms,
build 157's one dispatch then build 158's: the spawn 0.68 / 0.68; the plain 0.88 / 0.88; the mountain top 1.37 /
1.37; high over the spawn 1.32 / 1.32; the sea 1.38 / 1.39; the player's run 1.41 / 1.41 (99th 1.60 / 1.60); fast
across the world 1.21 / 1.22; the sea nodding 1.27 / 1.27 (99th 1.58 / 1.57). In turn, as all day (0.70, 0.88, 1.57,
1.61, 1.78, 2.00).

- **Against the pass mark:** every mark within 1.5% of build 157's: met (the most 0.7%, the plain). The pictures as
  marked. Kept: the one dispatch is the far eye's march, on the library's default limits.
- **The card's memory:** 973 MB by the one dispatch, 935 MB in turn (build 157: 968 and 935). So the one dispatch
  costs 38 MB at 2560 × 1600: its two textures of distances, as tall as the finest layer all the way across. [LATER]
  the finest layer's distances are read by nothing but the longest-ray count; leaving them out of the texture would
  give most of it back.

## The cut: the measuring switches the decisions have closed, out of the march  *(plan 2.2; David: "cutting code we won't be using and see if that gives us another 1% just for good measure")*

### The pass mark (written before the code)

- **What's cut from the march's code,** each a test every ray or every pixel made of a number that is always the same
  now:
  - A blend's pictures read in turn (`INCUS_FAR_TOGETHER=0`): 1b's "read together" was kept, and is now the only way.
  - The Oracle's half-layer test (`INCUS_TEST_HALF`): asked and answered on 2026-11-12.
  With build 158's (build 154's form) that is what we won't be using. **Kept on purpose:** the layers one after
  another (the baseline), the finer base and the haze's step (looks on the polish list), the map's edge and the step
  cap (still measured with), the code's own blend (a choice on David's panel).
- **The picture is pixel for pixel build 158's,** both ways of marching.
- **Flown the one dispatch, in turn, the one dispatch,** read against build 158. Builder's guess [low]: 0 to 1% off
  the far marks; the card's compiler will likely have been making little of these tests already. Under 0.5% can't be
  told from the flights' own spread.
- **Kept either way** (less code to carry), unless a mark is more than 1% slower, which would want explaining.

### The cut's result: about 2% off every mark  *(build 159, e97aed5; NVIDIA only, fog off)*

Flown the one dispatch, in turn, the one dispatch (logs/flight-1794608683, -9075, -9468). The card's whole, median
ms, build 158 then build 159:

| mark | one dispatch, 158 | one dispatch, 159 | | in turn, 158 | in turn, 159 |
|---|---|---|---|---|---|
| on the ground at the spawn | 0.68 | 0.66 | 2.4% | 0.70 | 0.68 |
| the bare plain | 0.88 | 0.87 | 1.9% | 0.88 | 0.86 |
| the mountain top | 1.37 | 1.34 | 1.8% | 1.57 | 1.55 |
| high over the spawn | 1.32 | 1.30 | 1.8% | 1.61 | 1.59 |
| low over the sea | 1.39 | 1.35 | 2.3% | 1.78 | 1.75 |
| a player's run over the plain (99th) | 1.41 (1.60) | 1.38 (1.56) | 2.4% | 2.00 | 1.96 |
| the sea, nodding (99th) | 1.27 (1.57) | 1.25 (1.54) | 1.7% | 1.79 | 1.76 |

- **Against the pass mark:** "0 to 1%, the compiler will likely have been making little of these tests already":
  it was about 2%, at every mark and both ways of marching; the compiler hadn't. David's "another 1%", twice over.
- **Step 2 as it stands, against this morning's build 152** (the layers in turn): the spawn 0.69 to 0.66, the plain
  0.88 to 0.87, the mountain top 1.57 to 1.34, high over the spawn 1.61 to 1.30, the sea 1.78 to 1.35, the player's
  run 2.00 to 1.38, the sea nodding's 99th 2.14 to 1.54. Everything under 2 ms, the most at a player's pace 1.56.

### David's look at build 159

David (2026-11-13), after 110 seconds in it, frames capped (logs/session-1794609556.log: it quit cleanly, no
warnings; the GPU 57 °C at 41 W, the CPU's hottest core 78 °C): **"fantastic"**. He named no lag and no flash on
arriving at a mark. By his own word before the look ("im not entirely sure i'll be able to see the lag without the
near view present") the lag's look is owed again once the near eye draws.

Step 2 is done enough to go on: the one dispatch is the far eye's march.

## Alongside: light read from the cell, made on the processor  *(plan 2.2)*

David (2026-11-13, via Counsel), on what comes next: "why don't we skip 1. we're putting light on the processor,
probably." To Builder: "right, we should be able to offload light to the cpu."

### First, what the light costs the far eye today: the pass mark for a measuring flight (no code changed)

- **What's flown:** build 159 as it is, against itself with the sun's shadows and the sky light left out
  (`INCUS_SIGHT_NO_SHADOWS`, `INCUS_SIGHT_NO_SKY`, old's switches): light on, off, on, off. The difference is the most
  that taking the light's rays out of the march can win; light read from a cell will cost a fetch of its own, so the
  real win is a little under it.
- **Builder's guess [low]:** 5 to 15% of the card's whole at the far marks, the most at the mountain top and high
  over the spawn (the most land in view, so the most pixels asking); the least at the sea. The Oracle's figure from
  old's flights was 3 to 13%.
- **What it decides:** nothing by itself. David has chosen to move the light; this says what to expect from it and
  which marks to watch, and gives the pass mark for the build its numbers.

### What the light costs today: 11 to 56% of the far eye, far more than guessed  *(build 159, e97aed5; NVIDIA only, fog off)*

Flown light on, off, on, off (logs/flight-1794610329, -0724, -1117, -1511); each pair agrees within 1%. The card's
whole, median ms:

| mark | light on | no shadows, no sky light | the light's share |
|---|---|---|---|
| on the ground at the spawn | 0.66 | 0.29 | 56% |
| the bare plain | 0.87 | 0.77 | 11% |
| the mountain top | 1.34 | 1.04 | 22% |
| high over the spawn | 1.30 | 0.82 | 37% |
| low over the sea | 1.35 | 1.14 | 16% |
| a player's run over the plain (99th) | 1.38 (1.56) | 1.13 (1.33) | 18% |
| fast across the world (99th) | 1.19 (1.72) | 0.79 (1.01) | 34% |
| the sea, nodding (99th) | 1.25 (1.54) | 1.04 (1.32) | 17% |

- **Against the pass mark:** guessed 5 to 15%, the most at the mountain top and high over the spawn. It's 11 to 56%;
  the most is the spawn (the valley floor: nearly every pixel's sky rays climb the valley's walls), then high over
  the spawn. The Oracle's 3 to 13% was from old's eye, where the march was most of the cost; the march has since come
  down and the light hasn't.
- **What it says:** with the light's rays out of the frame, every mark stands at 1.14 ms or under, less whatever
  reading a cell's light costs (a fetch or two a pixel). It is now the far eye's biggest single piece.

### David's choices for it (2026-11-13)

Asked how the sun's steps should show and whether shadows should follow his slider live: "i think we should try
snapping light from one of 16 levels every time the sun has moved a 16th of the day, and smooth it only if its bad.
that snap may be pretty subtle and act as a little clock for the player. live while dragging would only be for my
benefit and i dont think itd really benefit me." And, on there being no day in the new engine yet: "yeah, i know we
dont have days yet."

### Light read from the cell: the pass mark (written before the code)

- **What's built,** behind a launch setting at first (`INCUS_FAR_LIGHT=rays` for today's rays; unset, the new way), so
  the two fly against each other and David can look at both:
  - **On the processor** (a new part, `far_light.rs`), from the far cells kept as they came, for every column of
    every level: its top (how high its solid stands); **the height the sun's shadow reaches there** (one sweep across
    the level along the sun's way: each column takes the shadow of the columns toward the sun, dropped by the sun's
    slope); and **how much sky its top sees** (from how high the land stands round it in eight directions; once,
    after every region is in).
  - **On the card,** two pictures of the whole world, every level side by side in each: the shadow's heights
    (16-bit, kept twice so a new sweep is swapped in whole: David's snap) and the sky seen (8-bit). About 40 MB.
  - **In the march,** no light rays: a pixel reads the shadow's height and the sky where it landed, blended by the
    hardware between columns and by the code between the two levels its size falls between.
  - **The sun's steps:** a new sweep is made when the sun's setting has changed and stood still a moment, and shows
    whole at once. No fade. Sixteenths of a day when there are days.
- **Its lag, the mark Counsel asked for:**
  - A change of the sun shows within half a second of the slider coming to rest (the sweep itself Builder guesses at
    under 50 ms for every level; the wait is the settling and sending 16 MB to the card).
  - The land doesn't change yet in the new engine. When it does (a build, a dig), the columns it touched will want
    their tops made again and a sweep: not built now, written down as owed with 2.4's edits.
  - At load the light shows with the far textures, both made after every region is in; till then the far view is
    unshadowed (today it's shadowed from the first regions). Builder's guess: under 2 s more on the wait at load.
- **What it gives up, said to David before the look:** the far land is a surface for light. Nothing in the far view
  shadows what's under itself (overhangs, arches); a shadow's edge is as soft as its level's cells.
- **The picture changes;** it's a look step. Pictures of the rays' and the cell's light side by side at the four
  marks before the flights, for David's look after.
- **The card's memory:** up about 40 MB.
- **Flown rays, cell, rays, cell.** Builder's guess [medium, from the measuring flight]: each mark within 0.06 ms of
  its "light off" figure: the spawn 0.35 or less (rays 0.66), the plain 0.83 (0.87), the mountain top 1.10 (1.34),
  high over the spawn 0.88 (1.30), the sea 1.20 (1.35), the player's run 1.19 (1.38), the sea nodding's 99th 1.38
  (1.54).
- **Kept if** David likes the look (or likes it after tuning its softness and strength) and the marks land as
  guessed. Then the rays' code is cut from the march, and what the code's own blend (a choice on his panel) does for
  light is put to him.

### The column light built (build 160, unflown): its pictures, David's look at them, its lag

- **David on the land treated as a surface** (via Counsel, before the pictures): "far light skipping any small details
  out there may work if the smaller details can carry their own 'shadow', or absence of light, if they're rendered
  differently. everything has to look even, and reveal everything accurately though. not sure how that'll work". So
  three ways, priced for him: (1) the column light alone, as here; (2) light kept a cell at a time in 3D (true under
  arches at any sun; about 72 MB, seconds a sun step, three hours of Builder's); (3) both: each far cell also carrying
  its own cover, how much solid stands over it in its own column, in a channel of the far textures that's empty
  today. Builder's order: 1 as pictures, then 3 on top of it, 2 held as the fallback.
- **The pictures, made off the screen as he asked** ("if she can make me the pictures of those without taking the
  screen, and then fly them later after looking at them and discarding any that are obviously bad"): the rays beside
  the column light at the spawn, the mountain top, the plain, high over the spawn and the sea, the whole world loaded
  (scratchpad `pictures/light rays and cell ….png`). Builder's look at all five: the shadows fall in the same places,
  a little crisper by the column; nothing off in any. The rays' own pictures are pixel for pixel build 159's.
- **David's look** (the first two): "sounds good. those pics look fine to me."
- **Its lag, measured:** every level's tops and the sky seen made in 0.19 s (the whole world, 5,184 regions); a shadow
  swept for every level and sent to the card in 0.05 s. With the quarter second the sun's setting must stand still,
  a change of the sun shows in about 0.3 s: inside the pass mark's half second. The pictures are 40 MB on the card.
- **Flights held:** David, 2026-11-13: "hold on for a bit, i've got to take a break from watching the screen. we can
  try to queue up a bunch of flights, and you can run them while i'm sleeping, maybe."
- **His thought on test detail** (via Counsel): "we may want to consider adding some detail to the world; im sure
  builder would have fun placing some caves, towers, town buildings, and forests of her own design as test objects."
  Builder's view, given to Counsel: with the cells of 2 to 16, not before (a tower is smaller than one of today's
  far cells); woods from the scatter stage first, then a few hand-placed shapes whose right light is known.

### Way 3 (each cell's own cover) shelved till there's something to judge it on

Builder had meant to build it now behind a switch. Not built: on bare land a cell's cover is nothing, so its picture
would be the column light's and say nothing about "even"; the first overhang, slab or arch the far view can show
comes with the cells of 2 to 16 and the test detail. Shelved with its design, to be built then:
- A cell's cover: going up its own column from it, the most of (the air met so far, the solid then met); so a soft
  slope's own thickness covers nothing, and solid over a gap covers what's under the gap.
- Made when the far textures are made, kept in the faces' textures' fourth channel (empty today): no memory, no
  fetch more. The march dims a covered step's sky light by it, and its sun too.
- True for what's straight overhead; approximate for a low sun reaching in from the side.

### The night queue: the pass marks (written before the flights; build 160, one build, switches only)

David: "we can try to queue up a bunch of flights, and you can run them while i'm sleeping, maybe." Five standings of
the same build flown in turn, the round three times over (fifteen flights, about 100 minutes), so a slow hour of
Windows' own night work shows as one round out of step. A standing whose heat lines show it starved (slow-clock time,
processor load well over its usual) is set aside, not read. NVIDIA only, fog off.

1. **The column light** (unset): the far eye as it would ship.
2. **The rays** (`INCUS_FAR_LIGHT=rays`): what it replaces. The guess is the build's pass mark above: each mark
   within 0.06 ms of its "light off" figure (the spawn 0.35 or less, the plain 0.83, the mountain top 1.10, high over
   the spawn 0.88, the sea 1.20, the player's run 1.19, the sea nodding's 99th 1.38).
3. **No light at all** (`INCUS_SIGHT_NO_SHADOWS`, `INCUS_SIGHT_NO_SKY`, with the column light): what reading the
   cell's light costs. Guess [medium]: 0.02 to 0.06 ms at every mark.
4. **The layers one after another, with the column light** (`INCUS_FAR_SIDE=0`): what the one dispatch is worth now
   the light's rays are out of each layer's wait. Guess [low]: still 10 to 25% at the marks where two layers run long,
   a little less than build 159's 15 to 30%; the spawn and the plain level.
5. **The finer base, with the column light** (`INCUS_FAR_BASE=4`): the polish list's fourth pass re-priced. With rays
   it cost 0.3 to 1.7 ms (build 153, before the one dispatch). Guess [low]: 0.2 to 1.0 ms now, the least at the sea;
   still too dear to be the base, but the price David would weigh for the look.

### The night queue flown (build 160, 0f83598; 2026-11-13, 19:41 to 21:20)

David: "ok, go ahead". All fifteen flew and reported; none was stopped. NVIDIA only, fog off, the laptop plugged in.
Reports `logs/flight-1794620882` to `-1794626427`, in the queue's order; each flight's own log `logs/night-*.err`.

**The machine wasn't in the afternoon's state, in three ways. Said first, as it limits what the night can tell.**
- **Round 1's column light and rays flew starved** (the card's clock under 2000 MHz for up to 20 of the 20 s, the
  processor at 98 °C; medians up to double the later rounds'). Both set aside whole.
- **From about 20:12, half an hour after David left, something else took the card's time now and then.** The 99th
  of the card's whole went to 1.5 to 2.5 times its median at every mark, in every standing, no light too, and stayed
  so to the end; each pass's own 99th stayed tight, so the time went between passes. What flew between 19:54 and
  20:12 has 99ths as tight as the afternoon's. Not the screen going off, sleep or a screen saver (all read as "never" on
  mains); what it was isn't known. **So the night's 99ths and worsts are read by no one.**
- **The whole night ran 4 to 18% slower than the afternoon** for the same picture (the rays, pixel for pixel build
  159's: the mountain top 1.48 against 1.34, the run 1.49 against 1.38), the card drawing 65 to 90 W where it drew
  75 to 109. So the night's figures are read against each other, not against the afternoon's.

**What's read:** the medians of rounds 2 and 3, which agree with each other within 2% at nearly every mark and
standing (the spawn in turn 5%, the sea's finer base 3%). Round 1's three clean flights are within 3% of them but at
the spawn, where they're 9% faster. The card's whole, median ms, the mean of the two rounds:

| mark | column light | rays | no light | in turn | finer base |
|---|---|---|---|---|---|
| the spawn | 0.33 | 0.79 | 0.34 | 0.35 | 0.90 |
| the mountain top | 1.17 | 1.48 | 1.18 | 1.45 | 2.50 |
| the bare plain | 0.82 | 0.94 | 0.82 | 0.78 | 1.31 |
| high over the spawn | 0.93 | 1.42 | 0.92 | 1.21 | 1.81 |
| low over the sea | 1.19 | 1.41 | 1.19 | 1.65 | 1.54 |
| a player's run | 1.21 | 1.49 | 1.22 | 1.86 | 1.96 |
| the plain, nodding | 0.82 | 0.94 | 0.83 | 0.78 | 1.38 |
| the sea, nodding | 1.13 | 1.34 | 1.12 | 1.70 | 1.63 |
| fast across the world | 0.99 | 1.41 | 0.99 | 0.98 | 2.23 |

(The spawn's column light is round 3's alone: round 2's stood 4 s on the slow clock. Fast across the world stood on
the slow clock in most flights; shown, not leaned on.)

**Against the pass marks:**
1. **and 2. The column light against the rays:** 0.12 ms less on the plain, 0.2 to 0.3 at the sea, the run and the
   mountain top, 0.46 at the spawn, 0.49 high over the spawn: 13 to 58% of the far eye. Against the night's own "no
   light" it's within 0.02 ms at every mark: *met*, flown side by side. Against the afternoon's figures the mark
   named (0.35, 0.83, 1.10, 0.88, 1.20, 1.19) the spawn, the plain, the sea and the run land inside, high over the
   spawn 0.05 over (inside the 0.06), **the mountain top 0.07 over: a miss by a hundredth**, on a night running 4 to
   18% slow. **The sea nodding's 99th (1.38 guessed) can't be read tonight:** owed.
3. **What reading the cell's light costs:** nothing the flights can see (0.00 to 0.02 ms either way). Builder
   guessed 0.02 to 0.06: *missed low*.
4. **The one dispatch with the column light:** worth 20% at the mountain top, 23% high over the spawn, 28% at the
   sea, 34% at the sea nodding, 35% at the run. Builder guessed 10 to 25%, "a little less than build 159's":
   *missed*, it's worth more with the light's rays gone, not less. The spawn is level; **the plain is 5% dearer by
   the one dispatch** (0.82 against 0.78, nodding the same), where Builder guessed level.
5. **The finer base with the column light:** 0.35 ms more at the sea, 0.5 on the plain and the sea nodding, 0.57 at
   the spawn, 0.75 at the run, 0.88 high over the spawn, 1.33 at the mountain top. Builder guessed 0.2 to 1.0, the
   least at the sea: the least is at the sea; *the top of it missed* (the mountain top). Every median under 2.0
   but the mountain top's (2.50) and fast across the world's (2.23).

**The card's memory:** 1,015 MB in use with the one dispatch, 977 MB with the layers in turn (the 38 MB noted at
build 157); the far light's two pictures 40 MB of it.

**Owed, to read the column light's 99ths against David's bar** ("if everything in the far view is under 2ms"): the
column light and the rays flown again by day, with David at the machine and Turbo seen to be on; two flights,
fourteen minutes. Till then the bar is shown for build 159's rays (the afternoon) and not yet for build 160.

**Builder's reading:** the column light costs what no light costs, and David's look at its pictures was "those pics
look fine to me"; by its pass mark it's kept once the day flight shows the 99ths. The rays' code isn't cut till
David says so. For a night queue again: the first flight a throwaway to warm up, and a look beforehand at what
else is drawing on the card.

**What took the card's time, most likely** (David, 2026-11-14): "i plug the laptop into my TV to play space noise
sound videos. i bet its just the card running another display. all yesterday it outside with no other display".
Also: "i did close the lid and open it again." It fits what the flights show: with NVIDIA GPU only, the second
screen and the video's decoding are the same card's work, slipped in between the far eye's passes (each pass's own
99th tight, the whole's wide), and the card's power shared. Not proven: the day flights, with nothing else plugged
in, are the check. If their 99ths come back as tight as the afternoon's, this is taken as the cause.

### The day flights (build 160, 294e6ca; 2026-11-14, 09:32 to 09:45): the column light under David's bar

David: "Alright, lets get started". The laptop off his TV (one screen connected, its own), on mains, the card idle
at 18 W beforehand. The column light, then the rays; reports `logs/flight-1794670752` and `-1794671143`. Neither
starved. The card's whole, ms, median and 99th, beside build 159's afternoon flights:

| mark | 160 column light | 159 no light | 160 rays | 159 rays |
|---|---|---|---|---|
| the spawn | 0.29, 0.30 | 0.29, 0.30 | 0.66, 0.68 | 0.67, 0.69 |
| the mountain top | 1.04, 1.07 | 1.03, 1.06 | 1.35, 1.39 | 1.34, 1.39 |
| the bare plain | 0.77, 0.79 | 0.77, 0.78 | 0.87, 0.89 | 0.87, 0.89 |
| high over the spawn | 0.83, 0.86 | 0.82, 0.85 | 1.30, 1.34 | 1.30, 1.33 |
| low over the sea | 1.15, 1.19 | 1.14, 1.18 | 1.36, 1.41 | 1.35, 1.40 |
| a player's run | 1.14, 1.34 | 1.13, 1.32 | 1.38, 1.57 | 1.38, 1.56 |
| fast across the world | 0.76, 1.01 | 0.78, 1.01 | 1.22, 1.74 | 1.18, 1.72 |
| the plain, nodding | 0.77, 0.82 | 0.76, 0.81 | 0.87, 0.93 | 0.86, 0.92 |
| the sea, nodding | 1.05, 1.34 | 1.04, 1.32 | 1.26, 1.55 | 1.25, 1.54 |

- **The night's cause:** the rays by day are build 159's afternoon to within about 0.01 ms at every mark, 99ths
  too (fast across the world 0.04 off), and the card draws the afternoon's power again. The TV (a second screen and a video on the same card) is
  taken as what spoiled the night.
- **The column light's pass mark** (each mark within 0.06 ms of its "light off" figure; the sea nodding's 99th
  1.38): *met at every mark.* It stands no more than 0.012 ms over build 159's no light anywhere. The sea nodding's 99th
  1.34.
- **David's bar** ("if everything in the far view is under 2ms"): the column light's medians 0.29 to 1.15 ms, its
  99ths 0.30 to 1.34, and no single frame over 1.56 ms at any mark in the flight.
- **Against the rays:** 0.10 ms less on the plain, 0.2 at the sea and the run, 0.3 at the mountain top, 0.37 at the
  spawn, 0.47 high over the spawn.
- By its pass mark the column light is kept ("those pics look fine to me", and the marks landed). The rays' light
  and its hand-down stay in the march until David's word to cut them.

### Build 161: a live switch between the rays and the column light, for David's last look

David, asked for his word on the cut: "i would like to be able to take a look, and a/b column light vs rays one last
time." Till now the way was set at launch only. Asked where the switch goes and what it's called, he chose a
checkbox in the panel's Light section, under "sky light", called "light by rays" (unticked, the column light). The
far light is now made and kept up while the light is by rays too (40 MB, a sweep a sun step), so the switch shows
at once. Both ways draw build 160's pictures byte for byte (high over the spawn and the mountain top, off the
screen). Not flown: nothing a frame does has changed. The switch goes when the rays' light is cut.

**David's look** (2026-11-14, flipping the two in build 161): "column light actually looks better." To Counsel:
"column light is actually cleaner, i prefer it". The cut waits on his word, which Counsel is asking for.

**What he raised from the same look** (a screenshot close against a mountainside, north -959, east -90, up 499,
the 32-block cells some hundreds of pixels across): "i know that this will be more of a polish thing, but its
significant enough, close enough to the basic functions of the graphics engine, i'd like to have a talk about it; the
way that these whorls work out from the average of the rays, even if its hard to see at a distance, if these were
just more *orderly* , the way they compress into the distance would improve. im interested in ways that we could get
that sort of tuned, ideally without incurring a whole lot of extra costs in the process." Builder's reading of where
the shapes come from, given to him: a cell holds how full of land it is, the march gathers that as mist, and between
cell centres the card blends the eight nearest in straight lines; the outline is where the mist hides what's behind,
and it bows, pinches and kinks on the cells' grid, not the land's. The speckled fringes are each pixel's own random
start along its ray and the layers brought up crisp. Ways put to him, cheapest first: the panel's "brought up"
(free, now); an even pattern for the pixel's start in place of the random one (free on the card); a firmer rule
from fullness to cover (a few sums a step; thin things may suffer); a blend eased at the cell centres with no more
fetches; and the dear ones (a truly smooth blend at eight times the fetches; a cell holding how far the surface is
in place of how full). Nothing built; talked through first.

### The rays' light cut from the march (build 162): the pass mark, written before the cut

David, via Counsel: "Yes, cut them" (the rays' light, its hand-down, the "light by rays" checkbox). What goes: the
shadow ray and the five sky cones after the march, the sky light handed down between layers in the worth words, the
blends only they used, `INCUS_FAR_LIGHT`, and the checkbox. Old's rays stay in git (build 161, b843d87) and in `old\`.
- **The picture:** byte for byte build 161's column light at all five marks, off the screen.
- **The card's time:** one flight against this morning's column light (build 160, the same session). Builder's
  guess [low]: nothing to 2% less at each mark (build 159's cut of dead branches gave 2%); a mark more than 1%
  *dearer* would be a miss to explain, since a shader this size can shift either way when code leaves it.
- **The card's memory:** unchanged (the worth words keep their size).

**The cut made (build 162, unflown).** Out of `far.wgsl`: the shadow ray and the five sky cones (`cone`, `sunlight`,
`sky_seen`), the sky light handed down in the worth words, and the plain blends only they used (`far_blend`,
`blend_soft`, `usable_at`): 206 lines of 1,354. Out of the rest: `INCUS_FAR_LIGHT`, the "light by rays" checkbox, the
rays' two spreads. The worth words now hold a pixel's worth alone.
- **The picture:** all five marks byte for byte the column light's before the cut (the spawn, the mountain top, the
  plain, high over the spawn, the sea; off the screen). *Met.*
- **A check the cut broke, mended:** the test that the hardware's blend matches the code's called `blend_soft`, one
  of the blends cut. It now reads the hardware's blend the way the march itself does (`spot_soft`, `look_at`), and
  passes: the test now covers the march's own path, which it didn't before.
- **The flight:** owed, on David's go (it takes the screen seven minutes).

### Build 163: where a ray's steps begin, on a switch (the whorls talk's first step; unflown)

David on the ways put to him (2026-11-14): of the panel's "brought up", "smooth and sharp skylines i may ask you to
cut at some point. im waiting until i have more things to look at with it"; of the even pattern, "an even pattern with
a fine weave sounds good. i would like for you to put it on a switch for me"; of the firmer rule, "this also sounds
promising but i believe i would need some more finished content to judge it by. it sounds like it could be a slider
of some kind as well"; of the eased blend, "this sounds good, i would like to see this with a switch as well"; of the
dear ones, "that sounds painful, i would avoid this in favor of simpler fixes. have you heard of blue noise being
used for this kind of thing?"; "i'm fine with your proposed order of looking into this." Then: "if there's a reason
the weave and the blue noise can't be used simultaniously, that sounds good to me, if they can work simultaniously
i'd like to see that as well, and if they have parameters worth exposing i'd like to play with those. you could put
them under a ray start section or something. If there's no reason to wait, we could try the blue noise and weave,
and then you can run the tests that you have qeued after i take a look".

- **Built:** a "Ray start" section on the Render page, under "The picture". Pattern: scattered (by chance, as it
  was, the default), woven (Bayer's ordered pattern; its side 2, 4, 8 or 16 pixels), blue noise (a 64 x 64 square
  made at load by void and cluster in 0.09 s, 4 KB on the card, one read a ray), and the weave with blue noise in it
  (a slider: how much). And "spread (of a step)": how far the starts spread, 1 as it was; at 0 every ray begins
  alike and the rings the scatter was for come back.
- **They can work together:** the weave's number plus a share of the blue noise's, wrapped round. A little blue
  noise fills in between the weave's few distinct numbers (sixteen at 4 x 4) and breaks its crosshatch; a lot is
  blue noise.
- **Blue noise's own check** (`blue_noise.rs`'s test): every rank once; the lowest sixteenth of its texels stand at
  least 2.8 apart and 3.3 on average, where the same number picked by chance touch (1.0) and average 1.7. At void
  and cluster's usual wider reach (1.9) a few of the first points touched; at its own 1.5 none do.
- **The default's picture:** byte for byte build 162's at all five marks. *Met.*
- **Builder's look** (the four patterns drawn off the screen at David's own spot and size; a new `FAR_TEST_EYE`,
  `FAR_TEST_START` and `FAR_TEST_SCREEN` on the picture test): the patterns show only in the speckled patches
  where a ray grazes land, 8 pixels a speck (the base layer's). Scattered: clumps and gaps; woven: a plain
  checkerboard; blue noise: specks evenly spread, no pattern; both: a checkerboard broken up. The whorls themselves
  don't change: they're the blend's (way 4).
- **Pass mark for its flight** (written now, flown after David's look): at the default, one flight against this
  morning's column light measures the cut and the switch's standing cost together; Builder's guess [low] as the
  cut's, nothing to 2% less, a mark more than 1% dearer a miss, and then build 162 is flown alone from its own
  folder to tell the two apart. The pattern David picks is flown against the default: guess within 1% (blue noise
  is one read a ray, not a step).

### Build 163 flown at its defaults: the cut and the switch priced together (2026-11-14, 11:08)

David's look came first ("then you can run the tests that you have qeued after i take a look"); he closed the game
after two minutes and the flight took off (report `logs/flight-1794676498`), against this morning's column light
(build 160, `-1794670752`). One screen, on mains, the card idle beforehand. The card's whole, ms:

| mark | 160 median, 99th | 163 median, 99th | median |
|---|---|---|---|
| the spawn | 0.291, 0.300 | 0.289, 0.298 | 0.7% less |
| the mountain top | 1.043, 1.073 | 1.029, **1.234** | 1.3% less |
| the bare plain | 0.772, 0.790 | 0.763, 0.782 | 1.2% less |
| high over the spawn | 0.828, 0.857 | 0.815, 0.841 | 1.6% less |
| low over the sea | 1.151, 1.191 | 1.134, 1.176 | 1.5% less |
| a player's run | 1.140, 1.338 | 1.119, 1.318 | 1.8% less |
| fast across the world | 0.763, 1.013 | **0.794**, 1.059 | 4.1% more |
| the plain, nodding | 0.766, 0.821 | 0.758, 0.811 | 1.0% less |
| the sea, nodding | 1.054, 1.338 | 1.037, 1.314 | 1.6% less |

- **The pass mark** (nothing to 2% less; a mark more than 1% dearer a miss to explain): *met at every standing
  mark*, 0.7 to 1.8% less, the 99ths with them. The cut and the switch together cost nothing and give a little.
- **Fast across the world, 4% dearer: a miss by the mark's letter, not explained.** The mark loads regions as it
  flies and has stood at 0.763, 0.784 and 0.794 in three flights of the same no-light picture since yesterday, so
  one flight can't tell 4% from its own spread. Flagged, not waved away.
- **The mountain top's 99th, 15% wider, and one frame at 2.03 ms** (the only frame over David's bar in the
  flight): taken as disturbed, not as the build's. In that mark's 20 seconds the frame clock has a 45.7 ms hitch,
  the processor reads 100 °C, and the march's own 99th widens where every other mark's is tight; David was typing
  to Builder as it flew (his message arrived mid-flight), and the editor draws on the same card. So that mark's
  99th isn't shown for build 163: owed a flight with hands off.
- **Builder's lapse:** the flight took off before David was told it was up. Told first, from now on.
- The card's memory 1,015 MB in use, as before.

### The sharp blocky sections (David's look at build 163), and what they aren't

David: "well, the stuff we added didn't hurt. they look good together, but its very subtle. its kind of these sharp
blocky sections that bother me a little. idk if there's anything that can be done about that." His screenshot: a
hazy spire close by, flat plates in it with straight edges, every edge in 8-pixel stairs; the panel showing woven +
blue noise, 2 x 2, the spread at 0.00 (at 0 every ray begins alike and the pattern does nothing: the shot is "no
scatter at all") and the 4 x 4 and 2 x 2 layers at 0.00 ms.
- **The 8-pixel stairs** are the base layer brought up crisp: against cells that big the finer layers are told
  they're not worth it (`worth_of`), so the whole view is the 8 x 8.
- **The straight-edged facets, at his first spot** (north -959, east -90, up 499; drawn off the screen at 2,560 x
  1,600, pictures in the scratchpad's `pictures/`):
  - the code's blend and the hardware's draw them alike: *not* seams between bricks in the far textures;
  - every face lit alike (a throwaway change to `faces_light`): they stay: *not* the faces' light;
  - no switch to bare rock where no surface is near (a throwaway change to `look_of`): no change: *not* that switch.
  What's left is the shape of the blend itself: between cell centres the eight cells are blended in straight lines,
  which gives flat-ish panels that meet in creases along the planes through the cell centres. That is what way 4
  (the blend eased at the cell centres) goes after. Both throwaway changes were put back; the shader is as committed.
- **The flat plates in the hazy spire** (his second shot) may be something else again; not drawn, as its place
  isn't known. Builder's suspect, unchecked: skipping over a column the bands say is empty passes by the faint
  fringe a neighbouring column's cells blend into it (up to half a cell), so thin haze is cut off at a column's
  straight edge. To be drawn at his spot when he gives it (the Camera page's numbers).

### A correction: the rock switch does draw the flat plates (2026-11-14)

**The entry above is wrong where it says the rock switch changes nothing.** Builder looked at the throwaway
picture by eye, and only at its right-hand side, where it's true. Measured over the whole view at David's first
spot: without the switch 12% of the pixels change by 4 or more of 255 (the most 18), and what changes is the
straight-edged plates across the dark left and middle of his first screenshot; brightened side by side, they're
there with the switch and mostly gone without (scratchpad `pictures/rock switch base and without.png`). The
star-shaped facets on the right are untouched by it: those are still the blend's creases. Said to David as a
correction.

David, before the correction reached him: "smooth softens them but doesnt change them. i will wait for 2-16 to to
judge them. next time i have the game, remind me to find some blonky area and try the old code. id have thought we
would have cut that though. but, i think we seriously need to take a look at rock switch and texture making.
anything like that is something we dont want to miss the chance to tune up while its cheap or we forget".

- **The switch** (`look_of` in `far.wgsl`, and the code's blend's own): a spot none of whose eight cells shows
  surface is drawn as bare rock, unlit; the change from coloured and lit to rock is all at once, on the straight
  planes through the cell centres.
- **Why it shows, Builder's reading** (not proven): thin haze at the land's edge. A cell with a few blocks in it
  shows so little surface that the textures, which keep surface in 8 bits (a quarter of it, so under 2/256 is
  none), hold none for it; its haze is then rock, unlit, a box with straight sides beside haze that's coloured.
  The code's blend keeps surface finer, and draws its plates in other places (the two blends differ by 4 or more
  of 255 over 1.4% of that view, one clear rectangle among them).
- **Further off too:** without the switch 17% of the pixels change at the sea and at the bare plain (by 4 to 13 of
  255), 3% high over the spawn, 0.1% at the mountain top. At the sea and the plain it's the near ground: with the
  switch a grey-blue cast lies over the green; without it the ground keeps its colour.
- **The texture making** (`atlas.rs`, `spread`): a cell with no surface borrows the colour and facing of the
  nearest cell that has some, looked for inside its own brick of 8 (and its border), so two bricks can borrow
  differently for the cells where they meet; and a brick where nothing shows surface held black. Not yet measured
  on its own: the next step.

### Build 164 in the making: the rock switch on a flag (the pass mark, written before its flight)

Two steps, each with its own look, put to David: (1) the rock switch; (2) the texture making (borrowing reaching
across bricks, so neighbours agree; time at load only).
- **Built for step 1:** `far.wgsl`'s `look_of` takes the borrowed colour and facing wherever there's anything
  there, when the flag says so (the camera's `pyramid.w`, free since the cut); the look's `rock_switch`, on as it
  was by default. In the textures a texel no surface in its brick reaches now holds rock's colour in place of
  black (never read while the switch is on). The picture test takes `FAR_TEST_ROCK=0`. The panel's control waits
  on David's word for its name and place; the build's number isn't bumped till it's in.
- **The default's picture:** byte for byte build 163's at all five marks. *Met.* Without the switch, the real
  build draws the first spot exactly as the throwaway did.
- **Its flight** (after his look, hands off, with the mountain top's 99th that's owed): Builder's guess [medium]:
  nothing either way, within 1% at every standing mark with the switch on or off (one compare a step, no reads
  more).

**Build 164 handed over** (2026-11-14). David, asked whether a checkbox called "rock switch" under "far blend",
ticked as it is today, was the name and place he wanted, and told why the code's blend can't be cut whole (it draws
till the textures are in, and wherever a brick isn't in them): "ah, i see about old codes blend. maybe it could be
called \"initial blend\". i think you know what to do, proceed". So: the checkbox "rock switch" in "The picture"
under "far blend" (ticked by default); the far blend's first choice renamed from "old's (code)" to "initial blend",
on the panel and in a flight's header. The default's picture still byte for byte build 163's. Unflown; his look
first, then the hands-off flight.

**David's look at build 164** (2026-11-14): "unticked rock switch looks a lot better, with the hardware blend."
(The initial blend keeps its own switch: in code there are no borrowed colours to take, and it draws only till the
textures are in.) Builder's next, put to him: unticked becomes the default in the next build, with a launch switch
for the old way so the two can be flown against each other, hands off; the checkbox stays till he says to cut it;
the texture making is the build after.

**Build 165 (unflown, not handed over): the rock switch off by default.** From David's look ("unticked rock switch
looks a lot better, with the hardware blend"). The checkbox stays (ticked: as it was); `INCUS_FAR_ROCK=1` starts it
on, for flying against. The new default draws byte for byte what build 164 drew unticked (the mountain top, the
plain, high over the spawn, the sea), and the old way still draws build 163's (the plain). Its flight (the pass mark
above: within 1% either way) waits for the texture making, which changes no shader, so one pair of hands-off flights
prices the switch after both.

### Build 166 (unflown): the texture making, borrowing that reaches across bricks

The second of the two steps put to David ("i think we seriously need to take a look at rock switch and texture
making"; "i think you know what to do, proceed").
- **What was wrong:** a cell with no surface borrowed the colour and facing of the nearest cell with some, looked
  for inside its own brick of 8 and its border. So the same cell could borrow one thing in its own brick and
  another as the next brick's border: a seam the hardware's blend can't smooth. In a scattered test world, 486 of
  729 shared cells borrowed differently in the two bricks that hold them.
- **Built:** the nearest is looked for in the world: within three six-way steps (three, so all eight cells round
  any point beside a surfaced cell hold a colour), and of equally near cells the first in the world's own order.
  A cell then borrows the same in every brick that holds it. No surface that near: rock's colour, no faces.
- **Its tests** (`atlas.rs`): no cell borrows differently in two bricks, north, east or up, with the old way as
  the control (it does, so the check can fail); and a lone cell lends to three steps and no further.
- **In the pictures it barely shows.** Against build 165: at David's first spot 4% of the pixels differ at all, by
  3 of 255 at most; at the marks at most 3% differ, by 11 at most (the mountain top), 0.1% by 4 or more. The seams
  were real in the textures and next to nothing on the screen at these views. Said to David plainly: this step is
  for soundness, not for a look.
- **At load it came out faster, after a false start.** Reading three cells more on every side made the first try
  take 4.4 s to make and send the textures (2.0 to 2.3 s before, like for like in the picture test), though spread
  over the machine's threads. Timed in three parts: 3.9 s making the bricks, 0.12 s laying them into layers, 0.04
  s sending. Reading the cells a column at a time changed nothing (4.7 s). The hold-up was memory: every brick
  asked for its working room afresh and the threads waited on one another for it. With a bench a thread, used
  again for each brick: **0.22 s making, 0.6 s in all** for the whole world's 32,886 bricks, the picture byte for
  byte the first try's. The log now carries the three parts.
- All 27 tests pass. The shader isn't touched, so the card's time can't have moved: the pair of hands-off flights
  owed (the new default against `INCUS_FAR_ROCK=1`) prices the rock switch for builds 165 and 166 alike.
- **Not looked at yet:** bricks the window brings as the eye moves are made one at a time on the frame's own
  thread, each now reading the cells round it too; "fast across the world" in the flight will show if putting them
  on the card got dearer.

### Build 166 flown, the rock switch off and on (2026-11-14, 12:34 and 12:41, hands off)

David: "go ahead"; told before take-off this time. One screen, on mains, the card idle beforehand; neither flight
starved. Reports `logs/flight-1794681652` (the switch off, the default) and `-1794682043` (`INCUS_FAR_ROCK=1`),
beside build 163's (`-1794676498`, 11:08). The card's whole, ms, median and 99th:

| mark | 163 | 166, switch off | 166, switch on |
|---|---|---|---|
| the spawn | 0.289, 0.298 | 0.295, 0.306 | 0.295, 0.304 |
| the mountain top | 1.029, (1.234) | 1.059, 1.089 | 1.062, 1.096 |
| the bare plain | 0.763, 0.782 | 0.776, 0.795 | 0.776, 0.794 |
| high over the spawn | 0.815, 0.841 | 0.825, 0.856 | 0.823, 0.847 |
| low over the sea | 1.134, 1.176 | 1.163, 1.208 | 1.162, 1.208 |
| a player's run | 1.119, 1.318 | 1.159, 1.356 | 1.159, 1.358 |
| fast across the world | 0.794, 1.059 | 0.802, 1.068 | 0.795, 1.051 |
| the plain, nodding | 0.758, 0.811 | 0.771, 0.827 | 0.771, 0.828 |
| the sea, nodding | 1.037, 1.314 | 1.067, 1.358 | 1.067, 1.356 |

- **Off against on: the same.** Medians within 0.3% at every mark, 99ths within 1.1%. What the switch is set to
  costs nothing. That half of the pass mark is met.
- **But both are 1.2 to 3.6% dearer than build 163, and the pass mark's "nothing either way" missed.** It isn't
  the machine: the bringing up and the start pass stand to the thousandth where they stood in every flight today,
  the card's clock and power are the same. It's the march alone: 1.8% more on the plain, 2.7% at the sea, 3.4% at
  the mountain top, 3.2% at the sea nodding, 4.1% on the run. The one change to the march since build 163 is the
  flag's own compare in `look_of` (a read of the camera and a compare at every fetch), paid ticked or not. The
  lesson was already on file (in this shader, code added costs every pixel even when switched off); Builder
  guessed past it. Not proven to be the flag till a build without it flies. Cutting the switch (the hardware's
  blend always borrows) leaves `look_of` a compare shorter than build 163's: Builder's guess [medium] that it
  gives the 1.2 to 3.6% back; the cut waits on David's word, as the checkbox is his.
- **The mountain top's 99th, clean:** 1.089 with the switch off (the median 1.059), the worst frame 1.53. Build
  163's 1.234 was the disturbance. No frame of either flight over 1.8 ms.
- **Bricks brought as the eye moves** (the worry noted at build 166): not dearer. Fast across the world, putting
  the far cells on the card: 95th 1.3 ms, 99th 4.1 (build 163: 1.5 and 6.7).
- **The textures made and sent in 0.5 s** in both flights (build 163's flight: 3.8 s).

### Build 167 (unflown): the rock switch cut (the pass mark, written before its flight)

David, told the checkbox itself had made the march 1.2 to 3.6% dearer and asked for his word: "yeah, we can cut it
and move on." Gone: the flag and its compare in `look_of` (the hardware's blend takes the borrowed colour and facing
wherever there's anything there, with no test of surface at all: a compare shorter than build 163's), the "rock
switch" checkbox, `INCUS_FAR_ROCK`. The initial blend keeps its own switch (in code there's nothing to borrow).
- **The picture:** byte for byte build 166's at all five marks. *Met.* All 27 tests pass.
- **Its flight** (one, hands off, against build 163's and build 166's): Builder's guess [medium]: the march gives
  the 1.2 to 3.6% back, each standing mark at build 163's median or within 1% of it. If it doesn't, the flag
  wasn't the cause and the two builds between (164's texel change, 166's borrowing) are flown apart.

### Build 167 flown: the cut gives back what the checkbox cost (2026-11-14, 12:58)

David: "alright, lets proceed."; told before take-off. One screen, on mains, the card idle beforehand; not starved.
Report `logs/flight-1794683091`, beside build 163's and build 166's (the switch off). The card's whole, ms, median
and 99th:

| mark | 163 | 166 | 167 | 167 against 163 |
|---|---|---|---|---|
| the spawn | 0.289, 0.298 | 0.295, 0.306 | 0.291, 0.300 | 0.7% more |
| the mountain top | 1.029, (1.234) | 1.059, 1.089 | 1.030, 1.097 | 0.1% more |
| the bare plain | 0.763, 0.782 | 0.776, 0.795 | 0.766, 0.786 | 0.4% more |
| high over the spawn | 0.815, 0.841 | 0.825, 0.856 | 0.816, 0.841 | 0.1% more |
| low over the sea | 1.134, 1.176 | 1.163, 1.208 | 1.138, 1.180 | 0.4% more |
| a player's run | 1.119, 1.318 | 1.159, 1.356 | 1.123, 1.325 | 0.4% more |
| fast across the world | 0.794, 1.059 | 0.802, 1.068 | 0.783, 1.022 | 1.4% less |
| the plain, nodding | 0.758, 0.811 | 0.771, 0.827 | 0.763, 0.817 | 0.7% more |
| the sea, nodding | 1.037, 1.314 | 1.067, 1.358 | 1.043, 1.320 | 0.6% more |

- **The pass mark** (each standing mark at build 163's median or within 1% of it): *met*, 0.1 to 0.7% over it, and
  1.0 to 3.1% under build 166. The flag's compare was the cost; with it gone the march is back where it was, with
  the plates gone and the textures sound. The lesson, a second time today: in this march a switch isn't free
  because it's off; a look that's chosen gets its switch cut.
- **One frame over 2 ms:** at the mountain top, 2.02 ms, the flight's worst; that mark's 99th is 1.10 and the
  card's clock dipped during it (2,512 MHz at its least, 2,737 to 2,775 at the other marks). David typed a message
  to Counsel during the flight (Counsel's note; the desktop app draws on the same card), which mark it fell in
  isn't known. Build 166's two hands-off flights had no frame over 1.8 ms there. Said to David as it is: one frame
  of some 19,000, in a flight that wasn't quite hands off.

### David's look at the rings where levels hand over (build 167, 2026-11-14): none seen

The gate's owed look. Told what to look for (the changeovers are circles round the eye, at about 5,400, 10,700
and 21,500 blocks; "show: levels" tints them; a bad ring is a line or band that keeps its distance from the eye as
the land slides under it), he flew eleven minutes. David: "i can't see any rings. the crossover section looks nice
when i pull it down, but its slower, so i'll save that for last." So the gate's look is met at the default
hand-over (0.80); a longer hand-over ("levels hand over from", pulled down) is his for the polish list, priced
when he takes it up. Still owed for the gate's line: the Intel card's numbers, for the record. He asked for the
game again at once to try the "initial blend", which he'd forgotten.

**The initial blend, David's look** (build 167, a third launch; the first of them he closed at 26 s, before the
textures were in, where the far blend's second choice is still greyed and both draw the initial blend: "i couldn't
tell the difference but noticed i couldn't press back after i hit initial blend, except by default"). With both to
flip: "ah yes. the blocks go away mostly with the hardware blend, after it loads in. you don't need to do anything
with it." So the blocks he saw are mostly the initial blend's (its own rock switch among them), drawn only till the
textures are in; and the greyed second choice while loading stays as it is, on his word (Builder had offered to
let either be picked at any time).

### The cells of 2 to 16, first build: the 16s in a ring round the eye (the pass mark, written before the code)

David, asked the order (the cells of 2 to 16 next, the skyline talk after, since the soft skyline is as wide as the
cells it's read from and near land is read from 32s today) and the first build (the 16s alone, at old's size of 128
cells a side): "yes and yes, you can proceed. hopefully all this work, being much shorter than the longest rays,
goes on pretty much alongside everything else and doesn't cost much in time."
- **The design.** The 16s can't be laid over the whole map as the 32s and up are (their bands alone would be 85 MB,
  the 2s' 5 GB): a ring of 128 x 128 cell columns round the eye, as old kept (cells within about 1,024 blocks). A
  thread of its own (the ring's maker) sums the ring's cells from the doorway and makes its bricks and their
  textures; the frame only sends what it's handed, a little a frame, and moves the ring over at once when the new
  one is whole on the card, so there's never a hole and the frame makes nothing. Past the ring's edge the march
  reads the 32s as now; over the ring's outermost cells the two are blended, so its edge is no seam. The light
  stays the 32s' column light for now (the ring's own light comes with the 8s, 4s and 2s: a shading flaw written
  down, not a hole).
- **Checks before any look:** every 32 of a region is exactly its eight 16s added (the standing test old kept);
  with the ring off (`INCUS_FAR_RING=0`) the picture is byte for byte build 167's; with it on, no hole and no line
  at the ring's edge in pictures drawn off the screen, still and after a move.
- **Builder's guesses for its flight** [low, the first of its kind]: the marks that look far (the mountain top,
  high over the spawn, the sea) within 2% of build 167; the marks on the ground (the spawn, the plain, the run)
  5 to 15% dearer, as near rays step through cells half the size; moving, the frame's 99th no worse than build
  167's by more than a tenth (the ring's work is off the frame). On the card under 30 MB for the ring.

### Build 168 (unflown): the 16s in a ring round the eye, built and checked off the screen

Built as the pass mark above set out (`far_ring.rs`, new): the ring's maker on a thread of its own sums the ring's
cells from the doorway in squares of 8 columns, kept while the ring is over them, and makes each new brick's cells and
textures; the frame sends what it's handed (0.3 ms of a frame at most) to places nothing reads yet and moves the ring
over at once when the new one is whole. The ring's level sits after the others in the store, its bands and bricks'
places after theirs; the summing reads any cell size now (`far::cells`). In the march: the level asked for goes down
to the 16s once the ring is on the card, only inside it, and over its 8 outermost cells the 16s give way to the 32s.
- **Every 32 is exactly its eight 16s added:** 163,840 cells of 32 over five regions (the spawn's, a mountain's, the
  sea's, two at the map's edge). *Met.*
- **With the ring off** (`INCUS_FAR_RING=0`) the picture is byte for byte build 167's at all five marks. *Met.*
- **With it on, Builder's look:** skylines and near slopes crisper at the spawn, the plain and the sea; at David's
  first spot (the mountainside) the big star-shaped facets give way to ones half the size and the sky's edge is
  sharper. No hole anywhere, no line at the ring's edge. (Pairs in the scratchpad's `pictures/ring pair ….png` and
  `ring first spot without and with.png`.)
- **A ring that has moved draws byte for byte what a ring made in place draws** (the spawn and the sea; a short move
  of 300 and 200 blocks, where most bricks stay, and a far one of 1,500 and 900, where none do): the moving, the
  places given back and taken again, and the textures' borrowing across bricks all hold. *Met.*
- **The maker's time:** the first ring 2.3 to 3.6 s while the world is loading (every thread busy summing regions),
  where Builder guessed half a second: *missed*, though off the frame. A short move 0.06 to 0.12 s; a far jump 0.5
  to 0.8 s. The ring holds 340 bricks over the sea to 620 in the hills.
- **Not known yet:** the card's time and memory (its flight, the ring on against off), and how it looks moving
  (David's look). The march has gained code that every ray passes (a row picked for each level, the finest level
  read from the camera, the ring's test), and today's lesson is that such code costs even when it does nothing: the
  ring-off flight against build 167 prices that on its own.
- **Written down, not mended** (a shading flaw, by David's rule): inside the ring the light is still the 32s' column
  light. The ring's own light comes with the 8s, 4s and 2s.

### David's look at build 168, and the square it showed (build 169, unflown)

David, with a screenshot looking straight down from height: "looks good, i think you already know about this in the
pic?"; "referring to the square border with the light bleeding around it". The ring showed as a square on the
ground: greyer inside, a lighter line along its edge. **Builder's "no line at the ring's edge" was wrong:** every
picture checked was from near the ground; none looked down. Builder had flagged the ring's light (the 32s') and
guessed that; it wasn't that either.
- **Drawn off the screen** (straight down from 1,850 over the bare plain): the square is there, and stays with the
  sun's shadows off and with the sky light off too. Not the light. Inside it the blue of the ground is up by 5 of
  255, the cast the initial blend's rock switch gives; and half the pixels inside are, to the byte, the initial
  blend's picture, though the hardware's blend was chosen.
- **The cause:** a blend of the eight cells round a point is read from the brick holding the least of them. Bricks
  are made only where they hold band of their own, so where ground lies at a brick's floor there's no brick to
  read just under it, and the march falls back to the initial blend, which keeps its rock switch. The 16s' bricks
  start every 128 blocks up, and that plain lies at about 130.
- **Mended for the ring:** its maker makes a brick wherever a blend may read from one (band in its own cells, or
  in the cells one past it north, east or up). The square is gone: the ground's colour inside matches the land
  without a ring (the blue 53.0 against 53.6, where it was 58.6), and 436 of 17,485 pixels inside are the initial
  blend's, where 8,644 were. The ring holds about a third more bricks for it (445 over that plain, 694 at the spawn).
- **Found on the way:** the ring's bricks were sent to the card in no fixed order, so two runs put them in
  different places in the textures and a picture could differ by 1 of 255 in a few pixels (the card's filter, a
  last digit). Sent in order of place now: the same ring gives the same picture to the byte, run after run. A ring
  that has moved still stands elsewhere in the textures than one made in place, and draws the same to that last
  digit (1 of 255 in 0.1% of pixels), not to the byte as build 168's check happened to find.
- **The same flaw stands in the levels laid over the map** (the 32s and up), wherever land crosses one of their
  bricks' floors or edges (every 256 blocks up at the 32s): there too the hardware's blend falls back to the
  initial blend, and since build 165 the two no longer look alike. It may be what's left of the flat plates David
  saw on the mountainside near 500 up ("the blocks go away mostly with the hardware blend"). Not mended yet: it
  wants about a third more bricks in the textures (Builder's estimate from the ring: some 160 MB more on the card),
  so it's put to David first.
- All 28 tests pass; with the ring off the picture is still byte for byte build 167's.

### Build 170 to come: a brick wherever a blend may read from one, in the 32s and up (the pass mark, before the code)

David, told the flaw stands in the levels laid over the map too and what mending it costs: "yeah 160mb is no biggie.
go ahead and launch" (build 169 launched for his look at the ring; this is the build after).
- **To build:** the textures get a brick wherever the hardware's blend may read from one: band in its own cells, or
  in the cells one past it north, east or up (the ring's rule, build 169). For the 32s and the 64s, which follow the
  window, the textures' bricks come and go by region with the window, not by the pool's bricks.
- **Checks:** at a mountainside crossing a brick's floor (David's first spot, near 500 up: the 32s' bricks start at
  512), the pixels that are to the byte the initial blend's fall to a few in a hundred, as the ring's did (8,644 of
  17,485 to 436); the far pictures at the five marks change only there. A look step: Builder's look off the screen,
  then David's.
- **Builder's guesses** [low]: about a third more bricks in the textures, 160 MB more on the card; the textures made
  and sent in under a second still; the card's time within 1% at every mark (the march is untouched; more texels
  only); bricks brought as the eye moves a third dearer on the frame's own thread, still under build 163's figures
  (95th 1.5 ms, 99th 6.7 at fast across the world).

**David's look at build 169** (2026-11-14): "there was a visible ring of brighter sections before the world had
finished loading, and it vanished when it loaded. i don't think it's possible for a player to ever see it, so unless
its indicative of a problem that should be addressed, i think you got it." The square is gone. The bright ring while
loading, checked off the screen: by the initial blend the ring's edge shows as a brighter line; by the hardware's
there's none. It's the initial blend's own drawing of the ring's fade (why brighter, not worked out), seen only
till the textures are in, about half a minute from launch. Written down, not mended: if those seconds come to
matter, the cure is the textures made as the regions arrive, not once at the end (they take 0.8 s for the whole
world now).

### Build 170 (unflown): a brick wherever a blend may read from one, in the 32s and up

Built as its pass mark set out: the textures' bricks marked by the ring's rule (`atlas::bricks`), and the 32s' and
64s' brought and given back by region with the window (`add_region`, `remove_region`; the window's moves now tell
regions, not bricks).
- **Builder's estimate of a third more bricks, 160 MB, was wrong, on the high side.** The textures hold 34,049
  bricks where they held about 33,000: 3% more, and nothing more on the card (the textures stand at 573 MB, their
  room as it was). At the 32s a brick is 256 blocks tall and most land lies well inside one; the ring's third more
  came from its bricks being 128 tall, with plains at 130. So the flaw in the 32s and up was real but rare.
- **What it changes, the ring off, against build 167:** at David's first spot (the mountainside near 500 up) 4% of
  the pixels, 0.6% by 4 or more of 255 (the most 15), in one patch of straight-edged plates at the view's left: a
  place the hardware's blend used to fall back. At the five marks next to nothing (at most 0.1% of pixels, by 3 at
  most).
- **The textures made and sent in 0.8 s** for the whole world (0.6 before). All 28 tests pass.
- Not flown: the ring's flights (on against off) price builds 168 to 170 together; the march is build 168's.

### Build 170 flown: the 16s' ring on and off (2026-11-14, 14:54 and 15:01, hands off)

David: "sweet, good work. go ahead with your flights."; told before take-off. One screen, on mains, the card idle
beforehand; neither flight starved. Reports `logs/flight-1794690079` (the ring on, the default) and `-1794690468`
(`INCUS_FAR_RING=0`), beside build 167's (`-1794683091`). The card's whole, ms, median and 99th:

| mark | 167 | 170, ring off | 170, ring on | the ring on, against 167 |
|---|---|---|---|---|
| the spawn | 0.291, 0.300 | 0.289, 0.297 | 0.380, 0.385 | 31% more |
| the mountain top | 1.030, 1.097 | 1.051, 1.078 | 1.086, 1.117 | 5% more |
| the bare plain | 0.766, 0.786 | 0.785, 0.804 | 0.526, 0.538 | 31% less |
| high over the spawn | 0.816, 0.841 | 0.834, 0.859 | 0.914, 0.986 | 12% more |
| low over the sea | 1.138, 1.180 | 1.166, 1.211 | 1.346, 1.401 | 18% more |
| a player's run | 1.123, 1.325 | 1.156, 1.365 | 1.254, 1.388 | 12% more |
| fast across the world | 0.783, 1.022 | 0.800, 1.039 | 0.829, 1.124 | 6% more |
| the plain, nodding | 0.763, 0.817 | 0.782, 0.835 | 0.564, 0.610 | 26% less |
| the sea, nodding | 1.043, 1.320 | 1.065, 1.358 | 1.296, 1.574 | 24% more |

- **The pass mark's guesses, most of them missed.** The far-looking marks "within 2%": the mountain top 5% more,
  high over the spawn 12%, the sea 18%. The ground marks "5 to 15% dearer": the run 12% (in it), the spawn 31%
  (over it), the plain 31% *less* (the other way). "Under 30 MB on the card": met, and better, nothing more at all
  (1,015 MB in use with the ring and without: its bricks fit in room the pool and the textures already held).
- **Why, from the rays' counts.** At the spawn the 4 x 4 layer marches where it had nothing to do (its longest ray
  106 steps, none before): near land is now finer than its pixels, so it's worth it; that's the sharper picture
  being drawn. Over the sea more rays run long (544 of the 8 x 8's over 256 steps against 417; 276 of the 4 x 4's
  over 512 against 88). On the plain fewer do (130 over 256 against 272), the cells' bands hugging flat ground
  twice as close.
- **The ring off costs 2 to 3% over build 167** at every mark but the spawn: the march's new code, paid without
  the ring. The lesson a third time today. It's the ring's own code, so it goes only if the ring does.
- **David's bar:** every median at or under 1.35 ms and every 99th at or under 1.57 with the ring on. One frame over
  2 ms: 2.24 at high over the spawn (that mark's 99th 0.99; the frame clock's worst 4.6 ms there, the hottest core
  at 104 degrees), not explained.
- **A thing to watch:** over the sea with the ring on, the 4 x 4's longest ray reaches the most steps a ray may take
  (768; 668 without the ring). Rays cut short would show as sky at the sea's far edge; the sea's picture drawn at
  the flight's size shows none. Flagged for the 8s, where rays near the eye take smaller steps still.
- **Moving:** the frame's 99th on the run 2.03 ms (build 167: 2.06). Fast across the world 4.87 against 4.08, but
  4.82 with the ring off too: not the ring. Putting the far cells on the card there is a fifth dearer at the 99th
  (4.3 and 4.2 ms against 3.5) in both flights: build 170's bricks by region, most likely (each region the window
  brings now looks round each of its bricks); one flight each, so said with that weight.
- **The maker kept up:** 367 moves in the flight, the last ring made in 0.47 s (after a mark's jump).

### The 8s' ring (build 171 to come): the pass mark, written before the code

David, advised the 8s alone next and flown before the 4s and 2s: "yes".
- **To build:** the ring's code made to hold several rings, each its own level, maker and box (the 16s within about
  1,024 blocks, the 8s within about 512, each 128 cells a side); the march falls from a ring's level to the next
  one up wherever it's outside that ring's box, and fades at each ring's edge. `INCUS_FAR_RING` says how many
  rings (0 none, 1 the 16s alone, as build 170).
- **Checks before any look:** every 16 exactly its eight 8s added; with one ring the picture byte for byte build
  170's; with both, pictures from the ground and from straight above (the check build 168 lacked): no hole, no
  square at either ring's edge; a moved ring the same to a last digit; and the sea's far edge looked at for rays
  out of steps.
- **Builder's guesses for its flight, the 8s on against the 16s alone** [low; the 16s' guesses mostly missed]: the
  spawn and the run 10 to 30% dearer again (the finer layers find more to do near the eye); the sea 5 to 15%
  dearer; the plain within 10% either way; the mountain top and high over the spawn within 5%. Every 99th still
  under 2 ms. On the card under 60 MB more, likely none allocated (the pool and the textures hold spare room). The
  8s' first ring made in under 3 s, a move in under a tenth.

### Build 171 (unflown): the 8s' ring, and the ring's code made to hold several

Built as its pass mark set out: the store, the maker and the march hold a list of rings (the 16s' and the 8s' now,
room in the camera for the 4s' and the 2s'); the march falls from a ring's level to the next one up wherever it's
outside that ring's box (`ring_level` in `far.wgsl`), and each ring fades over its 8 outermost cells. `INCUS_FAR_RING`
says how many rings (0 none, 1 the 16s alone).
- **Every 16 is exactly its eight 8s added:** 163,840 cells of 16 over four squares of the map. *Met.*
- **With one ring the picture is byte for byte build 170's** (the spawn, the plain, the sea). *Met.*
- **From straight above, the check build 168 lacked:** over the plain from 830 up and the spawn from 1,100, both
  rings against none: across each ring's edge the colour changes by under 1 of 255 on average, with no step. No
  square. (Amplified ten times a faint line shows at each edge; at its true size it's a part in 255.)
- **A moved pair of rings draws what a pair made in place does** to a last digit (the spawn: 1 of 255 at most; the
  sea: the same to the byte).
- **The sea's far edge** drawn at the flight's size: whole, no rays out of steps showing.
- **Builder's look:** at David's first spot the mountainside's small facets (the 16s') smooth into a slope with fine
  contour lines; near skylines sharper again.
- **The makers:** the 8s' first ring made in 1.4 to 2.1 s while the world loads (the guess: under 3 s); 395 bricks
  over the sea to 866 at the mountain top, about as many as the 16s' ring.
- All 29 tests pass. Not flown yet, and not looked at by David.

**David's look at build 171** (2026-11-14, about a minute in the game, both rings on the card within five seconds):
"looks pretty good". Its flights (both rings against the 16s alone) wait on his go.

### Build 171 flown: both rings against the 16s alone (2026-11-14, 16:57 and 17:04, hands off)

David: "go ahead"; told before take-off. One screen, on mains, the card idle beforehand; the standing marks not
starved (fast across the world and the plain nodding stood a second or two on the slow clock in one flight or the
other). Reports `logs/flight-1794697458` (both rings, the default) and `-1794697843` (`INCUS_FAR_RING=1`), beside
build 167's (no ring) and build 170's (the 16s). The card's whole, ms, median (and 99th for build 171's):

| mark | 167, no ring | 170, the 16s | 171, the 16s alone | 171, both rings | both against the 16s alone |
|---|---|---|---|---|---|
| the spawn | 0.291 | 0.380 | 0.390, 0.396 | 0.425, 0.431 | 9% more |
| the mountain top | 1.030 | 1.086 | 1.093, 1.125 | 1.051, 1.089 | 4% less |
| the bare plain | 0.766 | 0.526 | 0.535, 0.547 | 0.964, 1.007 | **80% more** |
| high over the spawn | 0.816 | 0.914 | 0.907, 0.944 | 1.007, 1.032 | 11% more |
| low over the sea | 1.138 | 1.346 | 1.366, 1.421 | 1.389, 1.444 | 2% more |
| a player's run | 1.123 | 1.254 | 1.269, 1.406 | 1.303, 1.422 | 3% more |
| fast across the world | 0.783 | 0.829 | 0.848, 1.110 | 0.907, 1.174 | 7% more |
| the plain, nodding | 0.763 | 0.564 | 0.570, 0.618 | 0.966, 1.036 | 70% more |
| the sea, nodding | 1.043 | 1.296 | 1.318, 1.600 | 1.361, 1.631 | 3% more |

- **The pass mark's guesses.** The spawn "10 to 30% dearer": 9%. The run: 3%, under the guess. The sea "5 to 15%":
  2%, under it. The mountain top "within 5%": 4% less. High over the spawn "within 5%": 11% more, *missed*. **The
  plain "within 10% either way": 80% more, missed by far.** Every 99th under 2 ms: met (the most 1.63, the sea
  nodding; no frame of either flight over 1.76). Under 60 MB more on the card: met, nothing more (1,015 MB in use
  with one ring or two). The 8s' first ring under 3 s, a move under a tenth: the ring moved 729 times in the
  flight, the last made in 0.27 s after a mark's jump.
- **The plain.** The eye stands on the ground there, looking along it. With the 8s the 8 x 8 layer's rays that take
  over 256 steps go from 130 (the 16s alone; 272 with no ring) to 921, 308 of them over 384, the longest 536: rays
  grazing the ground walk it in the 8s' small steps for 500 blocks before they reach the 16s. The 16s had made the
  plain a third cheaper; the 8s take that back and a quarter more. It's the eye on the ground looking level, the
  view a player has most.
- **The trend to weigh before the 4s:** each finer ring halves the step of a ray grazing the ground near the eye.
  If the 4s cost the plain what the 8s did, it stands near 1.7 ms, and the 2s would take it past David's 2.
- **The several-rings code** costs the 16s alone 1 to 3% over build 170's single ring (0.390 against 0.380 at the
  spawn, 1.366 against 1.346 at the sea): the lesson again, a small one.
- **Moving:** the frame's 99th on the run 2.05 ms with both rings (2.07 with one); fast across the world 4.69
  against 4.56.

### The deep dive: the finer rays in time with the rest (2026-11-14). Its first measurement, the pass mark

David, asked whether to look at the grazing rays or build the 4s and 2s first: "lets take a deep dive into how we
can get the finer rays working simultaniously in time with the rest of them." And, offering help: "lmk if i need to
start a testing software for you" (not yet: the flights serve this step; Nsight if the cut is built).
- **Builder's reading, given to him:** the rings add no short rays; they lengthen rays that were long already. A
  ray grazing the ground walks 500 blocks of 8s and 500 of 16s in small steps before its far part. The way to have
  the near work beside the far is the Oracle's step 4, shelved when the gate was met: each ray cut into lengths (the
  rings' edge is the natural cut), the lengths marched side by side in the one dispatch and joined front to back.
  The frame then waits for the longest length, not their sum. Its costs: more rays in all; far lengths marched
  behind near land (they end in a step or two under ground; the frame before can tell them not to start); a join.
  Whether it pays turns on whether the card is waiting or working on the plain's frame, which no flight so far says.
- **The first measurement, before anything is built:** two switches for measuring (`INCUS_TEST_REACH`, a ray's near
  length alone; `INCUS_TEST_BEGIN`, its far length alone; build 172, the default's picture unchanged to a last
  digit), and three flights of the same build: whole; near alone (rays ended 1,024 blocks out); far alone (rays begun
  1,024 out).
- **Builder's guesses** [low]: on the bare plain, the near length alone about 0.45 ms and the far alone about 0.6,
  adding to about the whole's 0.96: the card working, not waiting, there. At the sea the far alone within a tenth
  of the whole (its long rays are far ones). If so, the cut should bring the plain to about the larger length's
  time; if a length alone costs about what the whole does, the cut gives little there.
- **Noticed on the way:** with two rings a picture repeats only to a last digit (1 of 255 in a few pixels at the
  spawn; the plain to the byte): the two makers' bricks reach the textures in the order they finish. Written down.

### The deep dive's first measurement flown (build 172; 2026-11-14, 18:04 to 18:23)

David: "you may proceed with the tests." Told before take-off; he wrote to Counsel twice from his phone as they
flew (a reply landing on the laptop draws on the same card: the frame clock's worst is 12 to 14 ms at the sea in
two of the flights; the 99ths and medians stand tight, and the medians are what's read). One screen, on mains, not
starved. Reports `logs/flight-1794701437` (whole), `-1794701828` (the near length alone, rays ended 1,024 blocks
out), `-1794702221` (the far alone, rays begun 1,024 out). The card's whole, median ms:

| mark | whole | near alone | far alone |
|---|---|---|---|
| the spawn | 0.429 | 0.384 | 0.374 |
| the mountain top | 1.042 | 0.401 | 1.062 |
| the bare plain | 0.972 | 0.380 | **1.177** |
| high over the spawn | 1.015 | 0.622 | 0.855 |
| low over the sea | 1.401 | 0.403 | 1.247 |
| a player's run | 1.312 | 0.377 | 1.185 |
| the plain, nodding | 0.971 | 0.362 | 1.181 |
| the sea, nodding | 1.371 | 0.415 | 1.179 |

(The whole is build 171's both-rings flight again, to a hundredth. Of each figure about 0.10 is the two passes that
don't change, the start and the bringing up.)
- **The near length is cheap and short everywhere:** 0.36 to 0.42 ms at the ground marks, and no near ray over 158
  steps at any mark.
- **At the sea, the run and the mountain top the far length alone costs 86 to 102% of the whole.** The rings' near
  work adds a tenth or less there; a cut could give back that tenth at most. Builder's guess for the sea (the far
  alone within a tenth of the whole): 89%, met.
- **On the plain the far length alone costs more than the whole, and Builder's guess (about 0.6) was wrong.** The
  flight doesn't measure what it was meant to there: with the near land gone the picture is another picture, and
  the 4 x 4 layer, which marches nothing on the plain when the ray is whole, marches 728 rays over 256 steps (its
  longest 683). So "waiting or working" on the plain isn't answered by this flight. **A fault in the measurement's
  design, Builder's.**
- **What it does show for the plain:** the whole's 921 base rays over 256 steps are near steps (142 at most) on top
  of far steps (the far alone: 455 rays over 256, the longest 420). Each grazing ray pays both. And it shows what a
  cut must not do: far lengths marched where near land hides them aren't cheap unless they're told not to start.
- **Next in the dive:** the longest ray's steps by kind (the Oracle's question 7), which reads the plain directly
  instead of by taking the ray apart.

### The plain's steps by kind (build 173; the Oracle's question 7): the long rays hop, they don't sum haze

David: "proceed". Two instruments built, neither in the game's shader:
- **Steps by kind:** a variant of the march that only the picture test builds (`kinds_source` in `far_eye.rs`,
  `FAR_TEST_KINDS`: three kinds drawn as a picture's red, green and blue, a 1,024th a step), so it costs the game
  nothing. Its longest ray on the plain reads 533 steps against the flight's 536 (an 8-bit picture's rounding).
- **Watts, typically** (her question 8): each standing's report gives the card's middle reading beside its highest.

**The bare plain, the base layer's 64,000 rays, at the flight's size, drawn off the screen** (steps of each kind):

| kind | no rings, all rays | both rings, all rays | both rings, the 952 rays over 256 | both rings, the longest ray |
|---|---|---|---|---|
| every step | 1,877,000 | 2,844,000 | 331,000 | 533 |
| skips over a column two levels coarser | 850,000 | 1,659,000 | 60,000 | 49 |
| skips over its own level's column | 513,000 | 735,000 | 233,000 | 422 |
| strides through air | 46,000 | 65,000 | 17,000 | 36 |
| steps in thin haze | 24,000 | 39,000 | 7,000 | 13 |
| steps at a surface | 409,000 | 307,000 | 13,000 | 16 |
| samples mixed from two levels | 2,600 | 5,800 | 1,600 | 0 |
| blends fallen back to the code | 580 | 1,000 | 750 | 3 |

(The 16s alone: 2,211,000 steps, 135 rays over 256, the longest 396 with 290 skips over its own column. Back-ups
after a first touch: none on the plain.)
- **Five steps in six are skips, not samples.** With both rings the rays take fewer samples than with none (412,000
  against 480,000); what grew is skipping, from 1.36 to 2.39 million.
- **The long rays are hopping over their own level's columns one at a time.** The longest ray's 533 steps are 422
  such hops and 65 samples. A ray skimming a little over flat ground is under the top the coarser column gives
  (its band's top is rounded up to its own, bigger, cell), so the coarse skip fails and the ray falls back to
  hopping the fine columns: 8 blocks a hop in the 8s, 16 in the 16s, 32 beyond.
- **So the wall isn't haze being summed, and it isn't the samples' steps.** Builder's reading for the Oracle and
  David: a table of each column's true highest ground, level by level (not rounded to the cell), would let such a
  ray skip a column of 128 or 512 blocks in one hop while it's clear of everything in it, with the haze untouched.
  This is the plan's column-height table, which the Oracle's section 4 set aside as paying only with the skyline by
  cover; these counts say it pays with the haze kept. Neither strides sized to the haze nor the cut into lengths
  goes at the hops: both act on samples, which are a sixth of the steps.
- The far light's tops are each column's average height, not its highest, so they can't serve as that table.

### The height table (build 174 to come): the pass mark, written before the code

David, shown the plain's steps by kind and asked whether to build it or take it to the Oracle first: "go ahead and
build the height table, and keep me informed about how you're doing it and how it works."
- **The design, as told to him.** Each column of each level carries one more number on the card: the height in
  blocks of the highest thing (ground or water) in it and in the eight columns round it at its own level (the
  blend reads a little into its neighbours; today's skip doesn't count them, and may be cutting haze at columns'
  edges). The summing records each column's true top and each coarser column takes its four children's highest. The
  numbers sit after the bands in the same buffer (about 28 MB). While the world loads they hold the band's own
  rounded top, so the skip is today's to the letter; once every region is in a thread works the true ones out and
  sends them whole. The rings' makers do their own columns. In the march the skip asks the table in place of the
  band, a coarser column first (four levels up, then two, then its own), and clears it in one hop when the ray is
  over the highest thing there by the blend's reach.
- **Checks:** no column's number under anything in it or beside it (a test against the doorway's own columns);
  the plain's steps by kind off the screen, before and after; pictures at the marks and from straight above; then
  flights against build 172's.
- **The picture** may change by a little where today's skip cut haze leaning in from a taller neighbour column:
  a look step for Builder's eye off the screen first.
- **Builder's guesses** [medium, from the counts]: on the plain with both rings the hops over a ray's own column
  fall from 735,000 to under 150,000 and the rays over 256 steps from 952 to under 100; the card's time there from
  0.97 ms to 0.6 or under (the 16s alone: 0.54). The sea and the run 10 to 30% less (their long rays skim water).
  The spawn and the mountain top within 5% either way. On the card 28 MB more.

### Build 174 (unflown): the height table built, its counts and its pictures off the screen

Built as its pass mark set out. The summing records each column's highest block's top (`far::CellColumn::highest`),
a coarser column taking its four children's highest; the store keeps a number for every column after the bands in
the same buffer (28 MB); while the world loads it's the band's own rounded top; once every region is in a thread
works out each column's highest with the eight round it and sends the whole table (0.09 s). The rings' makers do
their own columns. The march's skip asks the table (`top_in`), the biggest column first: four levels coarser, then
two, then its own. `INCUS_FAR_TOPS=0` leaves the true numbers unmade, for flying against.
- **The table's numbers are true:** over three regions, 4,095 cell columns' highest things are the doorway's own
  columns', block for block, at every level from 32 to 1,024; and no column's number is under anything in the
  eight columns round it. A column not known holds its neighbours back. (A new test.)
- **The plain's steps by kind, both rings, before and after** (the base layer's 64,000 rays, off the screen):

| kind | build 173 | with the table |
|---|---|---|
| every step | 2,844,000 | 1,173,000 |
| skips over a coarser column | 1,659,000 | 601,000 |
| hops over its own level's column | 735,000 | 116,000 |
| samples (air, haze, surface) | 411,000 | 416,000 |
| rays over 256 steps | 952 | 16 |
| the longest ray | 533 | 326 |

  Builder's guesses: the hops under 150,000 (116,000: *met*); the rays over 256 under 100 (16: *met*). The samples,
  which make the picture, stand as they were.
- **The pictures, against build 171's:** no shift overall (the mean change 0.00 to 0.02 of 255 at every mark and at
  David's first spot), a scatter of about a part in 255 on average at his first spot and less elsewhere, and a few
  pixels at the far horizon flipped between sea and sky, both ways. The scatter is from where rays land after a
  skip: they used to land at the same columns' edges, pixel after pixel, and now land where the table lets them;
  each ray's fine steps into a surface begin from there. At his first spot the slope's fine contour lines (banding
  from the columns' grid, Builder thinks) come out a little grainier. David, hearing the picture had changed:
  "let me see the change before discarding it. for all we know it might be nice." Nothing is discarded; his look
  comes first.
- All 30 tests pass. Not flown.

**David's look at the height table's pictures** (2026-11-14, the pair at his first spot, before and after): "from
those pics i cant see anything wrong". Its flights (the table against `INCUS_FAR_TOPS=0`) wait on his go.

### Build 174 flown, the height table on and off (2026-11-14, 19:16 and 19:23, hands off): a mixed result

David, after his look in the game: "it looks great, builder. go ahead". Told before take-off. One screen, on mains,
not starved; no hitch in either flight's frame clock. Reports `logs/flight-1794705815` (the table, the default)
and `-1794706206` (`INCUS_FAR_TOPS=0`), beside build 172's whole flight (the same rings, before the table's code).
The card's whole, median ms:

| mark | 172, before the table | 174, the table's code without its numbers | 174, the table | the table against 172 |
|---|---|---|---|---|
| the spawn | 0.429 | 0.471 | 0.499 | 16% more |
| the mountain top | 1.042 | 1.178 | 1.132 | 9% more |
| the bare plain | 0.972 | 1.152 | 1.010 | 4% more |
| high over the spawn | 1.015 | 1.136 | 1.136 | 12% more |
| low over the sea | 1.401 | 1.676 | 1.158 | 17% less |
| a player's run | 1.312 | 1.558 | 1.061 | 19% less |
| fast across the world | 0.891 | 0.933 | 0.760 | 15% less |
| the plain, nodding | 0.971 | 1.145 | 0.982 | 1% more |
| the sea, nodding | 1.371 | 1.635 | 1.108 | 19% less |

- **The long rays went, as counted:** the plain's rays over 256 steps 910 to 15 (the longest 531 to 324), the sea's
  631 to 71 and its 4 x 4's 551 to 132 (its longest no longer at the step limit: 768 to 409), the run's 1,088 to 4.
- **Where long rays were the cost, the time fell:** the sea, the run and the sea nodding 17 to 19% under build 172
  (31 to 32% under the same build without the table's numbers). The run is under build 167's, before any ring.
- **Builder's guess for the plain (0.97 to 0.6 or under) missed, badly: it's 1.01.** And the spawn, the mountain
  top and high over the spawn, guessed within 5%, are 9 to 16% dearer. Three fifths of the plain's steps went and
  its time didn't move: so its time wasn't in those hops. A hop was cheap; what the frame pays for there is
  something else (the samples are as many as before, 416,000).
- **Why the dearer marks:** the skip's new shape. It tries three columns where it tried two, the biggest first, and
  a ray near the ground fails all three before it samples. The middle column shows the price alone: the table's
  code without its numbers is 10 to 20% dearer than build 172 at every mark.
- **The mend, next:** try the ray's own column first. The table's numbers only grow with the column's size, so a
  ray that isn't clear of its own column is clear of no bigger one: it samples after one look, where build 172
  took two. A ray that is clear looks at the bigger ones in turn and takes the longest hop.
- **On the card:** 1,044 MB in use, 29 MB more (the guess: 28).
- **Watts, typically** (the new line): 66 to 111 W by mark, the two flights within 2 to 7 W of each other.

### Build 175, the skip's own column first: the pass mark (written before anything is counted, drawn or flown)

The mend named above. *In the open: the skip's new loop was typed before this mark was written. Nothing was counted,
drawn or flown with it first.*

**What changes, and only this:** the skip looks at the ray's own level's column first. Not clear of it: it stops
looking and samples (one look; build 174 took three, build 172 two). Clear of it: it looks at the column two levels
coarser, then four, stopping at the first it isn't clear of, and takes the longest hop of those it's clear of. Each
hop is one build 174 could have taken, so nothing new is skipped.

**The marks:**
- **Off the screen, on the plain with both rings:** no more steps than build 174's (1.17 million; 16 rays over 256
  steps), since the hop taken is never shorter than the one build 174 took. The looks counted too, as they're
  what this build changes.
- **The picture:** build 174's, but for a scatter of about a part in 255 where a hop ends somewhere else (as build
  174's was against 173's). Anything an eye could see is a miss.
- **Flown against build 174, near the ground** (the spawn, the mountain top, high over the spawn): under build
  174's 0.499, 1.132 and 1.136 ms. The hope is within 5% of build 172's 0.429, 1.042 and 1.015, and it's only a
  hope: my guesses in ms have missed four times, and the march still carries a third row of numbers that build 172
  didn't.
- **Flown, where the long rays were** (the sea, the run, the sea nodding): no more than 5% over build 174's 1.158,
  1.061 and 1.108. This is where the mend could cost: a clear ray takes three looks for a hop where build 174 took
  one.
- **The plain: no guess.** What its time is spent on isn't known. David, 2026-11-14: "are you using Nsight to
  check on what the plains time is actually spent on?" Not yet, and it's the right tool: a trace standing on the
  plain after this flight (how full the card is while the far eye marches there, and which lines the time sits on).

**If it misses:** near the ground still over 5% dearer than build 172 means the price is the code's size, not the
order of the looks (to try: two columns, its own and one coarser). The sea over 5% dearer than build 174 means the
looks a hop cost (to try: begin at the column that gave the last hop).

### Build 175's first mark, counted: the mend's reason was wrong, and it isn't flown (2026-11-14)

The own-column-first skip, off the screen at the flight's size, with the looks counted (`FAR_TEST_KINDS`, kinds 13 to
15 new: a look that found the ray clear of the column two levels coarser, four coarser, and a look that found it
not clear). Build 174's looks follow from the same counts (it took one look where the biggest column was clear, two
where only the middle one was, three otherwise).

| mark | steps, 174 and 175 | steps in air where even its own column isn't clear | looks, build 174's order | looks, its own column first |
|---|---|---|---|---|
| the bare plain | 1,173,000, the same to 3 | 102,000 | 1,390,000 | 2,137,000 (54% more) |
| on the ground at the spawn | 1,578,000 | 114,000 | 2,448,000 | 2,603,000 (6% more) |
| the mountain top | 2,051,000 | 150,000 | 3,238,000 | 3,505,000 (8% more) |
| low over the sea | 1,309,000 | 117,000 | 1,495,000 | 2,152,000 (44% more) |

- **The steps passed** (the same as build 174's on the plain, 16 rays over 256, the longest 326), as the mark said.
- **The reason for the mend didn't.** "A ray near the ground fails all three before it samples" is true of only
  100,000 to 150,000 steps a mark. Nearly all the looks are at steps that do hop: at the spawn 941,000 steps are
  clear of their own column and only 113,000 of the biggest. Looking at its own column first takes more looks at
  every mark, not fewer.
- **And looks may not be the price either.** On the plain, build 172 took about 3.3 million looks (two orders,
  2.84 million steps) and build 174 takes 1.39 million, and build 174 is 4% dearer. Steps, hops and now looks have
  each been counted and each failed to say where the time goes. *Builder's explanation of the dearer marks in the
  write-up above ("a ray near the ground fails all three") is withdrawn: not shown, and against the count.*
- **So build 175 isn't flown.** The reordered skip and its counter are put by in Builder's scratchpad
  (`far.wgsl.175-own-first`, `far_eye.rs.175-own-first`); the code is build 174's again. No flight was spent on it.
- **Next, David's question:** Nsight on build 174 as it's built: the plain with the table and without its true
  numbers, the spawn, the sea (20 frames each from 100 s in). What's read: how full the card is while the far eye
  marches, what its groups of rays wait on, and the lines the samples fall on.

### The Nsight look at the plain (build 175, 2026-11-14, 19:59 to 20:07, hands off)

David: "are you using Nsight to check on what the plains time is actually spent on?", then "go ahead". Build 175 is
build 174's march untouched, with one switch outside it (`INCUS_SHADER_TEXT`: each shader's text written to the
session's log as the card's compiler is given it). GPU Trace Profiler from the command line, the game standing at a
mark (`INCUS_MARK`), 20 frames from 100 s in, capped at the screen's 240 a second; the traces, listings and scripts
are kept: the tables, the shaders' text and the scripts in `kept logs/nsight/b175/`, the trace files on the laptop in
`logs/nsight/b175/`.

*A fault of mine in the script: the sea's trace stood at the spawn (a stray "; " after the mark's name, and the game
"starts at the first"). So there are two traces of the spawn, which agree to a part in a hundred, and none of the
sea.*

**How full the card is while the far eye marches:**

| | the plain | the plain, the table's numbers off | the spawn |
|---|---|---|---|
| the far march, ms a frame (the flight's on the plain: 0.901) | 0.896 | 1.053 | 0.421 |
| the card's room in flight while it marches | 4.7% | 5.6% | 17% |
| rays' time in flight a frame, against the plain's | 1 | 1.39 | 1.69 |
| the march's samples | 448,000 | 636,000 | 441,000 |

- **The plain's time isn't work.** The spawn keeps two thirds more rays' time in flight than the plain and is done
  in under half the time. On the plain the card is 95% empty for 0.9 ms: the pass lasts as long as its last few
  rays. (The Oracle's "waiting, not working" of 2026-11-12, at the sea; here on the plain, and emptier.)
- **Only the 8 x 8 layer marches on the plain** (the flight's report: the 4 x 4's and the 2 x 2's longest rays 0
  steps), 1,000 groups of rays; the dispatch launches 28,000 groups a frame for the three layers all the same.
- **Memory:** a texture read takes 420 cycles on average and a buffer read 90 to 100; the first cache answers 87 to
  90% of buffer reads and 73 to 75% of texture reads, the second 77 to 87% of what's left. The program's own
  instructions miss their cache 23% of the time on the plain (16% at the spawn).

**The lines the time falls on.** Nsight names a hot instruction by its line in the compiled program. The march's
text from the log, compiled again by the game's own compiler with debug marks, gives the same 4,398 instructions in
the same order, each with the line of the text it came from (Nsight's listing begins 108 lines into the compiler's:
with that, the waits on memory land on memory reads and the last hot line on the program's end). Nsight gives only
a shader's 38 hottest instructions, which hold 38% of the march's samples. On the plain, of the march's samples:

| what | share |
|---|---|
| the skip's looks: waiting for the height table's number | 4.4% |
| the skip's looks: choosing the column, its sums and the test for a hop | 5.3% |
| waiting for the textures' answer at a sample | 4.2% |
| each pixel's first read, whether it's wanted this frame | 3.2% |
| reads of the bands' table (is the region summed; is the column) | 2.3% |
| the program's end | 2.6% |
| no line given | 8.4% |

- **No one line holds the time; the skip's looks are the largest named part** (about a tenth; a sixth with the
  table's numbers off), and the wait for the table's number is the hottest instruction with a name at every mark.

**Builder's reading (a reading, not shown yet):** the far eye's time is its longest chain: one group of rays, its
steps one after another, each step waiting on its memory reads one after another, to places no other ray has
touched this frame. The height table took two fifths of the steps off the plain's longest ray (531 to 324) and put
three reads in a row in a hop where there were two (the biggest column, the middle, its own). Steps down 39%, reads
a hop up 50%: about the same chain, and the plain didn't move. It fits the other marks: the same code with the
numbers off keeps its long rays and reads three in a row, 10 to 20% dearer everywhere; the sea's longest ray lost
47% of its steps and the sea 17% of its time; the spawn has no long rays to lose and got 16% dearer. It would also
explain why counts of steps, hops and looks never said where the time went: they count all the rays, and the time
is one chain's.

**What would test it:** the three looks' numbers read at once, not one after another (three reads issued together
wait about as long as one; the card has the room: it's 95% empty). The same skips, so the same picture. If the
reading is right, the plain and the spawn both fall; if they don't, it's wrong. Not built: put to David first.

### Build 176, the three looks' numbers read at once: the pass mark (written before the code)

The test of the reading above. David was told it's being built off the screen; it flies on his go.

**What changes, and only this:** when the skip's three numbers are read. All three (the column four levels coarser,
two coarser, its own) are asked for before the first is used, so the card waits for them together and not one
after another. The choice among them is build 174's as it stands: the biggest column the ray is clear of. So the
skips and the steps are the same, and so is the picture. No switch: a switch in the march costs.

**The marks:**
- **Off the screen, the plain:** the steps by kind are build 174's (1,173,000 steps, 16 rays over 256, the longest
  326), and the picture is build 174's as far as it repeats (two rings repeat to a last digit).
- **Flown, if the reading is right:**
  - near the ground, under build 172's, which read two in a row: the spawn under 0.429, the mountain top under
    1.042, high over the spawn under 1.015;
  - the plain under 0.90 (build 174's 1.010 less a tenth or more);
  - the sea, the run and the sea nodding at or under build 174's 1.158, 1.061 and 1.108.
- **By how much, a sum and not a promise:** the plain's longest ray takes 326 steps: 137 hops, 89 samples in air
  after three looks, 87 samples with no look. If a look waits one read and a sample two to five, its chain is 27 to
  41% shorter with the looks' reads together. My guesses in ms have missed five times; the pass line above asks a
  tenth.
- **What it costs:** every step in clear air now reads all three numbers where it read one to three (on the plain
  about 2.5 million reads a frame for 1.4 million). The card is 95% empty while it marches there; if the reads'
  number mattered and not their order, this build is dearer and the reading is wrong.

**If it misses:** the plain within 5% of build 174's means the long chain isn't the looks. Then the tail itself,
seen: one frame's rays in flight over time in Nsight's window (a screenshot from David), before anything else is
built on the reading.

### Build 176 built (unflown): the skip's three numbers read at once

The three numbers are read before the skip's loop, each by its own row, and the loop chooses among them as build
174's did (the reads lifted out of it, nothing else moved). Off the screen:
- **The steps by kind on the plain are build 174's to the last count** (1,173,357 steps; 600,940 skips over a
  coarser column, 115,874 over its own; 16 rays over 256, the longest 326).
- **The pictures are build 174's** at the spawn, the mountain top, the plain, high over the spawn and the sea: no
  pixel differs on the plain or the mountain top, and at the other three the most is 1 of 255 (two rings repeat to a
  last digit).
- All 30 tests pass. Waiting on David's go for one flight against build 174's.

### Build 176 flown (2026-11-14, 20:29, hands off): cheaper at eight marks of nine, and the pass mark missed

David: "go". Told before take-off; one screen, on mains, the card idle before (3 to 5%); no hitch in the frame clock.
Report `logs/flight-1794710172`, beside build 174's (`-1794705815`, flown at 19:16) and build 172's. The card's
whole, median ms:

| mark | 172, before the table | 174, the table | 176, its numbers read at once | 176 against 174 | 176 against 172 |
|---|---|---|---|---|---|
| the spawn | 0.429 | 0.499 | 0.467 | 6% less | 9% more |
| the mountain top | 1.042 | 1.132 | 1.071 | 5% less | 3% more |
| the bare plain | 0.972 | 1.010 | 0.934 | 8% less | 4% less |
| high over the spawn | 1.015 | 1.136 | 1.080 | 5% less | 6% more |
| low over the sea | 1.401 | 1.158 | 1.112 | 4% less | 21% less |
| a player's run | 1.312 | 1.061 | 1.007 | 5% less | 23% less |
| fast across the world | 0.891 | 0.760 | 0.764 | the same | 14% less |
| the plain, nodding | 0.971 | 0.982 | 0.908 | 8% less | 6% less |
| the sea, nodding | 1.371 | 1.108 | 1.052 | 5% less | 23% less |

**Against the pass mark:**
- Off the screen: met (the steps and the pictures build 174's).
- The sea, the run, the sea nodding at or under build 174's: **met** (4 to 5% under).
- Near the ground under build 172's: **missed, all three** (the spawn 0.467 for 0.429, the mountain top 1.071 for
  1.042, high over the spawn 1.080 for 1.015).
- The plain under 0.90: **missed** (0.934). It isn't within 5% of build 174's either (8% under): between the two
  lines the mark drew.

**What that says of the reading:**
- **Right, and small:** reads one after another do cost. Build 176 reads about three quarters more numbers on the
  plain than build 174 and is 8% cheaper there: their order mattered, their number didn't, and the card had the
  room.
- **Wrong in size.** The sum said the longest chain would be 27 to 41% shorter; the plain's march fell 8% (0.901
  to 0.825 ms). The looks' waits are a small part of a step.
- **Wrong as the reason the near marks are dearer than build 172.** With one wait where build 172 had two, build
  176 is still 3 to 9% dearer there. *"Three reads in a row where there were two" is withdrawn as the explanation
  of the spawn, the mountain top and high over the spawn.* What costs there isn't known.
- **What Nsight's names for the waits say, read again** (the plain; the 38% of the march's samples it names): 14%
  of the march's samples wait on memory; 21% wait on the program's own sums and branches or for their turn. The
  larger part of a ray's time, where it's named, is the program's own length, not memory.

**A sum that fits, not shown:** each mark's march, over its longest ray's steps, comes to 2.1 to 2.7 microseconds a
step at seven marks of nine (the mountain top 3.1 and high over the spawn 3.7, where two layers both have long
rays). On the plain the longest ray is 137 hops and 189 samples; build 172's was 471 hops and 62 samples. The two
marches (0.87 and 0.90 ms) fit a hop of about 1.3 microseconds and a sample of about 3.8: by that sum the plain's
march now is its longest ray's 189 samples, one after another (0.72 of 0.83 ms), and the table swapped many cheap
hops for the same time in fewer, dearer steps. It's a sum from two flights, not a measurement.

**Kept:** build 176 (the same picture, cheaper than build 174 at eight marks of nine, the ninth the same). With the
table and its numbers read at once, against build 172: the five marks with long rays 4 to 23% cheaper, the three
near the ground 3 to 9% dearer; the dearest mark is the sea at 1.112 ms, where it was 1.401.

**Next, as the mark said, nothing more built on the reading before the chain is seen.** A test that changes no
code: the step limit (`INCUS_TEST_STEPS`, a number the march is handed). With rays stopped at 160 steps, if a
mark's time is its longest ray's chain, the plain's march falls by about half and the spawn's (153 steps) doesn't
move; if it's something else, neither moves. One flight, the picture wrong on purpose (rays cut short), for
measuring only.

### The step limit at 160, build 176 as it's built: the pass mark (written before the flight)

David: "go". No code changes: `INCUS_TEST_STEPS=160` (the most steps a far ray takes, a number the march is handed;
768 unset). The picture is wrong on purpose, rays cut short: for measuring only.

**The question:** is a mark's time its longest ray's chain?

**The marks** (the march alone, "the far 8 x 8" line of the report; build 176's beside it):
- **If it is:** every mark whose longest ray is over 160 steps falls to about what 160 steps take, 0.34 to 0.43 ms
  by the sum of 2.1 to 2.7 microseconds a step: the plain 0.825 to 0.55 or under, the sea 0.999 to 0.55 or under.
  And the spawn, whose longest ray is 153 steps, stays within 3% of its 0.357.
- **If it isn't:** the plain's march stays over 0.74 (less than a tenth off), though its longest ray is cut in half.
- **Between the two:** partly, and said so.
- **What could muddy it:** a ray cut short ends without an answer, and the finer layers follow the coarser one's
  frame before; if cutting rays changes what the finer layers are asked for, their work changes too. The report's
  longest rays and the 4 x 4's and 2 x 2's lines will show it.

### The step limit at 160, flown (2026-11-14, 20:41, hands off): two kinds of mark

Told before take-off; one screen, on mains, the card idle before (5%); no hitch. Report `logs/flight-1794710853`
(build 176, `INCUS_TEST_STEPS=160`), beside build 176's own of 20:29 (`-1794710172`). The march alone ("the far
8 x 8"), median ms:

| mark | its longest rays' steps (the 8 x 8, the 4 x 4) | the march, build 176 | rays stopped at 160 | change | the card's watts, typically |
|---|---|---|---|---|---|
| the spawn | 153, 139 | 0.357 | 0.357 | the same | 92 |
| the bare plain | 324 | 0.825 | 0.494 | 40% less | 73 |
| low over the sea | 302, 409 | 0.999 | 0.554 | 45% less | 79 |
| the sea, nodding | 309, 442 | 0.938 | 0.572 | 39% less | 82 |
| the plain, nodding | 343 | 0.797 | 0.500 | 37% less | 74 |
| a player's run | 274, 337 | 0.890 | 0.634 | 29% less | 93 |
| the mountain top | 294, 302 | 0.950 | 0.869 | 9% less | 114 |
| high over the spawn | 257, 261 | 0.961 | 0.921 | 4% less | 113 |
| fast across the world | 176, 235 | 0.641 | 0.626 | 2% less | 89 |

**Against the pass mark: partly, as its third line allowed.**
- The spawn, with no ray over 160: the same to the last digit. Met.
- The plain: 0.494 for "0.55 or under". Met. Not "over 0.74": its time does follow its longest ray.
- The sea: 0.554 for "0.55 or under": missed by 0.004.
- "Every mark whose longest ray is over 160 falls": **no.** The mountain top, high over the spawn and fast across
  the world had rays of 235 to 302 steps cut to 160 and moved 2 to 9%.

**So there are two kinds of mark:**
- **Waiting marks** (the plain, the sea, the run, both nodding): their time is their longest ray's chain. Shown
  now, not summed: stop the chain at 160 steps and the march is 29 to 45% shorter. The card runs at 73 to 93 W.
- **Working marks** (the mountain top, high over the spawn, fast across the world): their time isn't their longest
  ray's; cutting it changes little. The card runs at 113 to 114 W at the first two: the old sign of work (the watts,
  2026-11-12). What their time is spent on isn't looked at yet; the spawn's kind isn't settled by this flight
  (nothing of it was cut), though Nsight found it three and a half times as full as the plain.
- **A part that isn't the chain:** the plain's 160 steps take 0.494 ms, 3.1 microseconds a step where the whole
  324 took 2.5. About 0.1 ms of the march doesn't shorten with the chain. Not known what it is.

**What it explains of the day:**
- The height table shortens long chains and adds work to every step. So it paid at the waiting marks (the sea and
  the run 21 to 23% under build 172) and cost at the others (the spawn, the mountain top, high over the spawn 3 to
  9% over).
- Counts over all the rays (steps, hops, looks) can't say where a waiting mark's time goes: only its longest
  chain counts there. At a working mark they should; that isn't tested.

**What a step budget a frame is worth, measured:** this flight is the far eye with no ray taking more than 160
steps in a frame. The card's whole: the sea 1.112 to 0.666 ms, the run 1.007 to 0.752, the plain 0.934 to 0.604, the
sea nodding 1.052 to 0.685 (its 99th 1.359 to 0.751); the dearest mark would be high over the spawn at 1.040. That
is the most lazy rays could give at that budget, not what they would: a ray's steps left over still have to be
taken in later frames. David, this morning: "it would seem we have to reduce the longest rays, one by one in order
of length, in order to actually free up warps more rapidly, consistently." The flight agrees with him.

### Nsight at the working marks (build 176): what's expected, written before the traces

David, 2026-11-14, on the two kinds of mark: "well the goal should always be to bring down the high spot. i was
just about to ask if it is time for lazy rays. go ahead and do your test, and send your complete assessement of how
lazy rays could get us that last bit off the plains to counsel."

Three traces as before (20 frames from 100 s in, standing at a mark): the mountain top, high over the spawn, and the
sea (the one that stood at the spawn last time; each run's log is now checked for where it stood).

**Expected, if "working" is the right word for the mountain top and high over the spawn:** the card's room in
flight while the far eye marches is several times the plain's 4.7%: over 20% at both. And the sea is like the plain:
under 10%.

**If the mountain top's is as low as the plain's:** the card isn't working there either, the watts mislead, and
"two kinds of mark" needs another name and another reason for the step limit's result.

### Nsight at the working marks (build 176, 2026-11-14, 20:54 to 21:00, hands off)

Three traces as before, each run's log checked for where it stood (the mountain top, high over the spawn, the sea:
all three at their own marks). The tables Nsight's reader gave are in `kept logs/nsight/b176/`, the trace files on
the laptop in `logs/nsight/b176/`. Beside build 175's two (the march alone):

| | the plain | the sea | the spawn | the mountain top | high over the spawn |
|---|---|---|---|---|---|
| the far march, ms a frame | 0.896 | 1.023 | 0.421 | 0.981 | 0.996 |
| the card's room in flight while it marches | 4.5% | 6.5% | 16.6% | 17.4% | 19.2% |
| rays' time in flight a frame, against the plain's | 1 | 1.65 | 1.74 | 4.27 | 4.74 |
| of the march's samples Nsight names: waiting on memory | 14% | 19% | 11% | 14% | 15% |
| waiting on the program's own sums, branches and turn | 21% | 16% | 25% | 18% | 16% |

**Against what was expected:** the mountain top and high over the spawn are about four times as full as the plain,
and under the "over 20%" I wrote (17.4 and 19.2). The sea is like the plain (6.5%, for "under 10%"). The two kinds
of mark stand.

- **The three fuller marks land within three points of each other** (16.6, 17.4, 19.2%), on very different views.
  That looks like a ceiling: as full as this program gets. Not shown: a shader's registers and how many groups of
  rays fit are in Nsight's window only, David's to read.
- **Their work isn't the base layer's alone.** Their base layers take 1.3 and 1.8 times the plain's steps and they
  keep 4.3 and 4.7 times its rays' time in flight; the 4 x 4 layer marches there and not on the plain. A layer's
  rays and steps aren't in the reports.
- The march's hot lines weren't read against the text this time (build 176's shader text wasn't logged for these
  runs).

### Where along a ray its steps lie (build 176, off the screen, the base layer)

A test-only count (`FAR_TEST_LEVELS`, beside `FAR_TEST_KINDS`): a ray's steps by the level each was read at. For the
base layer at the flight's size the 8s' ring is read to about half a kilometre, the 16s' to about one, the 32s from
there to about 5.4 km, the 64s to about 10.7, the 128s to about 21.

| | all its steps | a ray's mean | rays over 160 | the 8s | the 16s | the 32s | the 64s | the 128s and coarser |
|---|---|---|---|---|---|---|---|---|
| the plain | 1,146,000 | 18 | about 350 | 62% | 10% | 19% | 6% | 3% |
| the sea | 1,285,000 | 20 | about 130 | 68% | 12% | 15% | 3% | 1% |
| the mountain top | 2,027,000 | 32 | about 600 | 54% | 17% | 23% | 5% | 2% |
| high over the spawn | 1,522,000 | 24 | about 130 | 40% | 33% | 22% | 3% | 1% |

| the longest ray | steps | in the 8s | the 16s | the 32s | the 64s | coarser |
|---|---|---|---|---|---|---|
| the plain | 324 | 59 | 43 | 112 | 61 | 49 |
| the sea | 409 | 1 | 28 | 214 | 128 | 38 |
| the mountain top | 300 | 26 | 6 | 100 | 154 | 15 |
| high over the spawn | 254 | 0 | 25 | 163 | 66 | 0 |

- **The long rays are a thin band:** on the plain within 11 of the base layer's 200 rows, over the sea within 3.
  The steps past a 160-step cap are 1% of all the steps.
- **A long ray's steps are mostly in the 32s and the 64s, 1 to 11 km out,** not at the far horizon; and most of
  every mark's steps are within a kilometre.
- **The longest chain if rays were in two parts** (the near part, then the far part): split at about 1 km, 128 and
  222 on the plain, 86 and 380 over the sea, 148 and 268 at the mountain top, 68 and 240 high over the spawn; split
  at about 5 km, 223 and 146, 275 and 166, 181 and 198, 198 and 115. Cut at every change of level, the longest is
  133 on the plain and 244 over the sea (the 32s' stretch).
- (The counts come from pictures of 8 bits a number: good to a step or two in a hundred and sixty.)

### Lazy rays: Builder's assessment (2026-11-14, sent to Counsel at David's word)

David: "well the goal should always be to bring down the high spot. i was just about to ask if it is time for lazy
rays. go ahead and do your test, and send your complete assessement of how lazy rays could get us that last bit off
the plains to counsel."

**The short of it.** Yes for the plains, and not by the far horizon. The plain's and the sea's time can come down
by about half, but only if "lazy" begins at the edge of the 16s' ring, about a kilometre out, and only on top of
the long ray cut into lengths (the Oracle's step 4), which is the half of lazy rays that carries no lag and gives
the plains their prize by itself. And the high spot won't fall with it: the mountain top and high over the spawn sit
3 to 4% under the sea and don't wait on their longest ray. They need their own answer, and two facts for it are
missing.

**1. What's measured (today, build 176; the far eye's whole in ms: the sea 1.112, high over the spawn 1.080, the
mountain top 1.071, the run 1.007, the plain 0.934, the spawn 0.467)**

- **Two kinds of mark.** Rays stopped at 160 steps (a flight, no code changed): the plain's, the sea's and the run's
  marches fall 29 to 45%; the mountain top's, high over the spawn's and fast across the world's 2 to 9%. Nsight:
  while the far eye marches, the card has 4.5% of its room in flight on the plain and 6.5% over the sea; 17% at the
  spawn and the mountain top, 19% high over the spawn.
- **The long rays are few, in a thin band.** Of the base layer's 64,000 rays, about 350 take over 160 steps on the
  plain (11 screen rows of 200), about 130 over the sea (3 rows), about 600 at the mountain top, about 130 high over
  the spawn. The steps
  past a 160-step cap are 1% of all steps.
- **Their steps aren't at the far horizon.** By the level each step is read at (the base layer: the 8s to about half
  a kilometre, the 16s to about one, the 32s from there to about 5 km, the 64s to about 11):

  | | all its steps | in the 8s' ring | the 16s' | the 32s (1 to 5 km) | past 5 km | its longest ray, by the same four |
  |---|---|---|---|---|---|---|
  | the plain | 1,146,000 | 62% | 10% | 19% | 9% | 324: 59, 43, 112, 110 |
  | the sea | 1,285,000 | 68% | 12% | 15% | 5% | 409: 1, 28, 214, 166 |
  | the mountain top | 2,027,000 | 54% | 17% | 23% | 7% | 300: 26, 6, 100, 169 |
  | high over the spawn | 1,522,000 | 40% | 33% | 22% | 5% | 254: 0, 25, 163, 66 |

  Past about 21 km: under half a percent of the steps, and none of the longest rays' to speak of.
- **A step costs about 2 microseconds on a waiting mark, and part of the march doesn't shorten.** From the two
  flights (whole rays, and stopped at 160): the plain 0.825 and 0.494 ms, the sea 0.999 and 0.554. A line through
  each pair: 2.0 and 1.8 microseconds a step, with 0.17 and 0.27 ms left at no steps at all. What that remainder
  is isn't known (the dispatch launches 28,000 groups of rays a frame, 27,000 of which do nothing on the plain).

**2. What "lazy" can mean, and what each form does**

| form | what it is | the plains | the working marks | lag, and David's eye |
|---|---|---|---|---|
| A. A step budget, resumed next frame, on the screen's squares | his first wording | the prize measured above | nothing | not buildable as worded: a ray resumed after the camera turns is no longer its pixel's ray, and a turn takes a tenth of a second. What it keeps has to be kept by direction, which is C |
| B. The long ray cut into lengths, marched side by side in the same frame (the Oracle's step 4) | each length its own thread; joined front to back by a small pass | the longest chain 324 and 409 to about 100 | nothing | none: nothing kept between frames. Each length begins its own stepping, so the picture is equal in kind, not pixel for pixel: one look |
| C. Far lengths kept between frames, by direction (the Oracle's lazy rays) | the near part every frame; the far lengths re-marched as the eye's travel asks | the chain is the near part's: 86 to 148 if the far part begins at the 16s' edge | saves the far part's work when it isn't re-marched: at most 20 to 30% of the base layer's steps there | all nine of the Oracle's, and sharper than it drew them (below) |

- **B, sized.** Cut where the level changes, the longest chain is 133 on the plain and 244 over the sea: the 32s'
  stretch is two and a half doublings long and has to be cut inside itself. Cut by doublings of distance: about 85
  and 100 *[estimate from the levels' counts]*. By the lines above that's a march of about 0.34 ms on the plain and
  0.45 over the sea, the far eye's whole about 0.45 and 0.56, where it's 0.93 and 1.11 *[estimates: two points a
  line, and the join and the extra threads not priced]*.
- **B needs only the long rays cut.** They're a few hundred, and the frame before already writes every ray's steps
  for the finer layer's lead. So: a ray that was long last frame, or whose neighbour was, is marched as lengths; the
  rest as now. The lead has to be turned for the camera's turn (as I read the march, it isn't today), or a nod moves the band off its
  lead and that frame pays the whole chain. A few thousand extra threads on a card 95% empty.
- **C, sized, and where the Oracle's version misses.** Its smallest version was "the farthest one or two lengths of
  every layer". Past 11 km lie 3% of the plain's steps and no chain: kept or not, nothing moves. For the chain,
  the far part has to begin at the 16s' edge:

  | the far part begins at | the near part's longest chain (plain, sea, mountain top, high) | the far part's share of the steps | how often it's re-marched at a run |
  |---|---|---|---|
  | about 1 km (the 16s' edge) | 128, 86, 148, 68 | 28%, 20%, 30%, 27% | every 3rd or 4th frame (a quarter of an 8 x 8 pixel there is 3 blocks of travel) |
  | about 5 km | 223, 275, 181, 198 | 9%, 5%, 7%, 5% | every 19th |
  | about 11 km | 276, 372 | 3%, 1% | every 38th |

  (The Oracle wrote before the rings and the height table; the table took the far hops away, and what's left of a
  long ray is in the 32s.)
- **So C's real test is the hard one.** Land 1 to 5 km off is big on the screen. Kept by direction it holds still on
  the world while the near ground moves every frame, and at a run it's redrawn every third or fourth frame. Whether
  that's "basically invisible" (his words of 2026-11-12) is his eye's to say, and it can't be judged on a far strip
  first, because the far strip has nothing in it to save.

**3. The Oracle's nine, as they stand for C at one kilometre**

- *A ray in flight always finishes* (1), *behind kept warm* (7), *an age view and a check against a full march*:
  as written. The far part's chains (222 to 380 steps) spread over three or four frames at a run are 60 to 100
  steps a frame: no worse than the near part.
- *Kept by direction isn't pixel for pixel* (2), *far squares fixed to the world* (3): David's open question in the
  plan's table, and now it's about land at 1 to 5 km, not a distant backdrop.
- *Only the far end can be lazy* (4): that's B. C can't be built without it.
- *Light and fog put on every frame* (5): the march today gathers colour already lit. For C the lengths have to
  keep colour, faces, cover and distance, and the light go on at the join. The gather has to be reshaped; the sun's
  part is already kept apart for the first surface, which is a start.
- *What's hidden now is wanted next* (6): at 1 km most far land is behind near ground, so the store marches a lot
  nobody sees. Cheap when lazy; not free.
- *Far land moves in steps* (8): the sharpest at this distance. All at once, never as each direction finishes.
- *The marks stop measuring* (9): standing still C's far part costs nothing, so the gate is read at the run and
  the nodding marks, at their 99th, lazy on and off.
- *Memory:* by direction at the 8 x 8's pitch, all round and 30 degrees up and down, about 46,000 squares a length;
  with the 4 x 4's, tens of MB. Not a constraint.

**4. The high spot after the plains come down**

With B alone, by the estimates above: the sea about 0.56, the plain about 0.45, the run somewhere between; the
mountain top 1.07 and high over the spawn 1.08 as they are. The far eye's worst goes from 1.11 to 1.08. So "bring
down the high spot" is as much those two as the plains, and for them:

- **What's missing, first: how much of their work is the 4 x 4 layer.** Their base layers take 1.3 to 1.8 times
  the plain's steps, but they keep 4.3 to 4.7 times its rays' time in flight. On the plain the 4 x 4 marches
  nothing; there it marches (its longest rays 261 and 302). The reports don't count a layer's rays or steps. They
  should, before anything is built for these marks.
- **What's missing, second: whether 17 to 19% in flight is as full as this program gets.** The spawn, the mountain
  top and high over the spawn land within three points of each other. If that's the ceiling, it's the program's
  size (old's march kept 16 groups of 48 at 125 registers), and the answer there is a leaner march, which also
  explains why the height table's extra row made those marks dearer. Nsight's window shows a shader's registers
  and how many groups fit; it's David's to read off (one screen).
- **What C would give them:** the far part's work on the frames it isn't re-marched: at most 20 to 30% of the base
  layer's steps, less at a run. Real, and not the whole answer.

**5. Another way to the same chain, David's and already in the plan: the skyline by cover**

Of the plain's longest ray's 326 steps, 176 are samples close over the ground: 89 strides inside the blend's reach,
63 in thin haze, 24 at a surface. The Oracle said it on 2026-11-12: the rays that set the time are "adding up the
thin haze that makes your soft skyline", and a height table can't skip what a ray is summing. Today's flights agree.
With the skyline made by cover there's no haze to sum and those steps mostly go. That's a look, his to choose (haze
or cover), set after the cells of 2 to 16; it shortens the same chain B does, by a different road, and the two add.

**6. What I'd do, in order**

0. **Two instruments** (small, no look): each layer's rays and steps in every report; and one flight with rays
   stopped at 1 step (no code changed), which reads the part of each mark's march that isn't its rays at all.
   David reads the march's registers off Nsight's window.
1. **B, the long ray cut into lengths, for the long rays only.** Pass mark from the step-limit flight: the plain's
   and the sea's marches at or under what 160 steps took (0.494 and 0.554), aiming at 0.34 and 0.45. The same flight
   says what it costs the working marks (it should be nothing). One look for David: the horizon band, standing and
   nodding. No lag to judge.
2. **Then the high spot is the working marks,** and what step 0 found decides: a leaner march if it's the
   program's size; the 4 x 4's rule for where it's wanted (a look) or C if it's the 4 x 4's work.
3. **C, far lengths kept between frames,** on B's store, when there's work to save that only it can save. Its first
   build is his look at land 1 to 5 km off while running, far squares fixed to the world, before any of the rest.

**7. What I don't know**

- Whether the lines from two flights hold down to a chain of 100 (the estimates in 2 rest on them).
- What the 0.17 to 0.27 ms that doesn't shorten is.
- The 4 x 4 layer's long rays over the sea (409 to 442 steps): counted only for the base layer. Likely the same
  shape with a longer stretch in the 32s; not measured.
- How B's picture differs: each length beginning its own stepping could show at the cuts (old's rings on flat
  ground were a stepping artefact). It needs the look, and the cuts scattered ray to ray as the starts are now.
- My sums today missed their pass marks twice (the skip's own column first; the three numbers read at once). The
  step-limit flight is a measurement; everything in section 2 past it is a sum.

### The march's registers, read by David off Nsight's window (build 176, the mountain top's trace)

David opened `logs/nsight/b176/b176-mountain/incus_artus_2026_11_14_20_56_04.ngfx-gputrace` (Builder gave the
file's name a second out, from memory; he found it) and sent the Shader Pipelines rows:

| shader | its share of the samples | groups of rays that fit (# Warp) | registers (# Reg; live) | Smem | instructions | its three top waits |
|---|---|---|---|---|---|---|
| `march_all` | 85% | 12 | 168 (166) | 4608 | 8.53K | Wait 35%, Long Scoreboard 17%, Selected 16% |
| `brought_up` | 12% | about 48 | 21 (18) | about 36 to 288 | 264 | Long Scoreboard 60%, Short Scoreboard 19%, Wait 6% |
| `fs_main_gamma_framebuffer` | 2% | about 18 | 26 (24) | about 108 to 864 | 136 | Long Scoreboard 29%, Short Scoreboard 19%, Wait 19% |
| `start_pass` | under 1% | 40 | 48 (45) | 256 | 1.56K | Long Scoreboard 41%, Wait 15%, No Instructions 12% |

- **The ceiling is real, and it's the march's size.** 12 groups fit of the 48 a unit of the card has room for: a
  quarter. The traces found 17.4% in flight at the mountain top and 19.2% high over the spawn: seven tenths and
  three quarters of all the march can keep in flight, averaged over a pass that has to fill and empty. Those marks
  are full. This answers the assessment's second missing fact.
- **By these rows and old's** (its march 125 registers, 16 groups; `start_pass` 48, 40): the groups that fit are
  about 2,048 over the registers. 16 groups would want about 128 registers or fewer, 20 about 100. *[A rule read
  off three rows, not from NVIDIA.]*
- **The Oracle named the trade on 2026-11-12:** "On a card that's working, every number held is room lost, which
  was old's lesson. Nsight's registers afterwards, as before." The lean step's numbers kept in hand, and the height
  table's third row, were bought for the marks that wait; at the marks that work they're room lost. That the table
  is why the spawn, the mountain top and high over the spawn got dearer fits, and isn't shown: there's no register
  count from before it.
- **Smem 4608** is 72 bytes a ray in a group of 64, which is the size of the levels' view's table of six tints
  (`far.wgsl`, `march_pixel`): a guess, to be checked. Old's lesson called local arrays hostile to the card.
- **So the high spot is two jobs:** the marks that wait (the sea 1.112 ms, the run, the plain) want their long rays
  cut into lengths; the marks that work (high over the spawn 1.080, the mountain top 1.071) want a leaner march.
  The three dearest are within 4% of each other, so the worst falls far only when both are done.

## Smaller passes, a leaner march, the long rays cut: David's order  *(plan 2.2)*

David, 2026-11-14, after the assessment and his reading of the registers: "seems like that last bit of performance
is behind leaning out the rays, lazying the rays, and breaking the shader into smaller passes. do you think you can
do all that?" Builder: yes to building all three, no promise of what each gives. Then: "i don't think we should
leave anything undone. if atomizing the megakernal a bit with the main shader being broken into passes *can* give
higher performance at no cost, that will translate directly to more headroom for prettying it up. i think we should
probably start with trying to break it into smaller passes to fit more, and then lean and cut the rays."

So the order is his: smaller passes first, then leaning, then the cut.

### Build 177, the march's audit: what's expected (written before the trace)

Not a split yet: the instrument that says which split fits how much more, so a pass isn't built on a guess (two
sums missed their pass marks on 2026-11-14). `INCUS_FAR_AUDIT=1`: thirteen cut-down versions of the march are
compiled beside it, each under its own name, and each marched over the base layer into scratch after the far eye,
in a pass of its own. One Nsight trace then names each one's registers and how many groups of rays fit (David's
reading, one screenshot) and its time (the trace's passes). Without the switch the game builds none of it.

| version | what's taken out | what it says |
|---|---|---|
| `audit_whole` | nothing | the control: it must read as the march does, 168 registers and 12 groups |
| `audit_hops` | all sampling: a ray stops where it would first sample | what a pass that only crosses clear air would fit |
| `audit_samples` | the skip: every step samples | what a pass that only samples would fit |
| `audit_no_handover` | the second level read and mixed over a hand-over | that part's registers |
| `audit_no_code_blend` | the code's blend of eight cells where the bricks can't answer | that part's registers |
| `audit_no_floor` | what's kept of the cells over the world's floor | that part's |
| `audit_two_rows` | the skip's third column (the height table's far row) | that part's |
| `audit_no_rings` | the rings' levels and their edges' fade | that part's |
| `audit_no_finish` | light, fog, the sky's weight and the views after the march | that part's |
| `audit_no_views` | the steps' and the levels' views (the table of six tints) | whether Smem 4608 is that table |
| `audit_lean` | the six trims that leave a whole march, together | how lean one program gets by trimming |
| `audit_samples_lean`, `audit_hops_lean` | the two pieces, trimmed | what the pieces of a split would fit |

Off the screen: all thirteen parse and are sound (a test, without the card), compile on the card, and with the
audit on the plain's and the mountain top's pictures are build 176's (no pixel on the plain, the most 1 of 255 at
the mountain top). The game's start is about a minute longer under the switch (thirteen compiles).

**Expected** *[guesses; of registers Builder has no record to go by]*:
- `audit_whole`: 168 and 12. If not, the audit's programs aren't the game's and nothing else in the batch is read.
- `audit_hops`: under 64 registers, so 32 groups or more.
- `audit_samples`: 150 to 160: the skip isn't where the registers are.
- The dearest single parts: the hand-over's second read and the code's blend, 15 to 30 registers each; the floor,
  the third row and the rings 5 to 10 each; the finish under 10; the views none, and Smem to nothing.
- `audit_lean`: 120 to 140. Under 128 would mean 16 groups fit by trimming alone.

**What it decides:**
- If the hops fit three times the groups and the samples no more than now, a split into "cross the air" and "the
  rest" pays only as far as hops are the working marks' time: the versions' own times over the base layer say how
  far.
- If trimming alone gets under 128, leaning comes before any split.
- If no version but the hops gets well under 168, the registers are in the gather itself, and the split has to be
  inside a step (find the place; read it; gather it), which is the dearer build.

### The audit's traces didn't run (2026-11-15, 06:04): Nsight couldn't have the card's counters

David: "go ahead". Both runs ended within twenty seconds with no trace: "TARGET ERROR: GPU Performance Counters
unavailable. Please close any existing GPU profiling sessions, or enable access to GPU performance counters."

- **It isn't the audit.** The game started each time, compiled its thirteen versions (15 s) and stood at its mark.
  Last night's own command, with no audit, fails the same way this morning (a 45-second check).
- **Nothing of Nsight's is running** (no window, no reader, none of Builder's), and nothing that profiles the card
  has started since last night's last trace at 21:00, which worked from the same shell, not as administrator. The
  driver is the same (32.0.16.2101); the laptop hasn't restarted since 2026-11-13; LibreHardwareMonitor has run
  since then and was running when the traces worked.
- **What's happened since 21:00, as far as the machine says:** David opened the trace in Nsight's window to read the
  registers; the laptop went off and on mains at 21:11; a night idle.
- **Not known which.** To try, cheapest first: the graphics driver reset (Win+Ctrl+Shift+B) and the 45-second check
  again; else a restart. The traces' script is kept at `kept logs/nsight/b177/scripts/nsight-177.sh`; build 177 is
  committed and built, and nothing else is owed before the traces.

**Later the same morning.** David looked at the driver's setting (Counsel's check, his to look at): "it shows all
users". He reset the graphics driver (Win+Ctrl+Shift+B: "ok, i did it") and the 45-second check failed the same
way. Windows isn't waiting for a restart and installed nothing overnight but a Store runtime. What's left is a
restart of the laptop. The traces' script no longer points into Builder's scratchpad: it runs from the project
(`bash "kept logs/nsight/b177/scripts/nsight-177.sh"`) and its traces land in `logs/nsight/b177/`.

### The march's audit, traced and read (build 177, 2026-11-15, 06:21 to 06:28, hands off)

A restart of the laptop gave Nsight its counters back (David: "ok, i restarted"; the 45-second check then passed;
why they went isn't known: it outlasted a driver reset with the setting on all users). Two traces, the mountain top
and the plain, each at its mark with the far eye whole two minutes before the frames:
`logs/nsight/b177/b177-mountain/incus_artus_2026_11_15_06_24_54.ngfx-gputrace` and
`.../b177-plain/incus_artus_2026_11_15_06_28_12.ngfx-gputrace`. David read the mountain top's Shader Pipelines rows
off Nsight's window (three screenshots):

| version | share of the samples | groups that fit (# Warp) | registers (live) | Smem | instructions |
|---|---|---|---|---|---|
| `march_all` (the far eye itself, all three layers) | 17% | 12 | 168 (166) | 4608 | 4.26K |
| `audit_whole` (the same program, the base layer alone) | 7% | 12 | 168 (166) | 4608 | 4.26K |
| `audit_samples` (no skip) | 13% | 12 | 168 (166) | 1024 | 3.93K |
| `audit_no_rings` | 9% | 12 | 168 (166) | 4608 | 4.13K |
| `audit_no_code_blend` | 7% | 12 | 167 (163) | 0 | 3.25K |
| `audit_no_views` | 7% | 12 | 168 (166) | 4608 | 4.23K |
| `audit_no_finish` | 7% | 12 | 168 (166) | 2816 | 3.8K |
| `audit_two_rows` | 7% | 12 | 168 (166) | 3328 | 4.15K |
| `audit_no_handover` | 7% | 12 | 160 (157) | 0 | 2.75K |
| `audit_no_floor` | 7% | 12 | 168 (166) | 2560 | 4.15K |
| `audit_lean` (the six trims together) | 5% | **16** | **128** (126) | 768 | 2.26K |
| `audit_samples_lean` | 4% | 16 | 119 (116) | 0 | 2.12K |
| `audit_hops` (no sampling) | under 1% | 28 | 68 (65) | 0 | 1.42K |
| `audit_hops_lean` | under 1% | 28 | 68 (65) | 0 | 1.38K |
| `start_pass` | under 1% | 40 | 48 (45) | 256 | 1.56K |

Every version's three top waits are the march's own: Wait 34 to 39%, Selected 15 to 19%, Long Scoreboard 11 to 17%.

**Against what was expected:**
- **The control holds:** `audit_whole` reads as the march does (168, 12, 4608, 4.26K). The audit's programs are
  the game's.
- **The hops alone:** 68 registers and 28 groups, for "under 64 and 32 or more". Near.
- **The samples alone:** 168, for "150 to 160". The skip holds none of the registers at the peak.
- **One part out at a time: nearly nothing, and the guesses missed.** The hand-over's second read 8 registers (15
  to 30 guessed); the code's blend 1 (15 to 30); the floor, the third row, the rings, the finish and the views
  none (5 to 10 guessed for three of them).
- **All six trims together: 128 registers, and 16 groups fit.** ("120 to 140"; "under 128 would mean 16 groups by
  trimming alone": it's 128 exactly, and 16.) Together they give 40 where one at a time they give 9. So the march
  has several places that each need about 166 at once, and the count is the highest of them: take one down and
  another holds it up.
- **Smem isn't the views' table of tints** (out, and Smem is still 4608): that guess was wrong. It goes to nothing
  when either the code's blend or the hand-over's second read is taken out, and part of the way with the finish,
  the floor or the third row out: it looks like what the program can't keep in registers at its fullest places
  and puts by in memory instead (72 bytes a ray). Not shown.

**What it says about a split, which was the question:**
- **"Cross the air" and "the rest" as two passes doesn't pay.** The hops would fit 28 groups, but they're under a
  seventh of the base layer's samples at the mountain top (under 1% against 7%), and the rest keeps all 168
  registers.
- **What pays is inside the step:** the fullest places are two paths few steps take. On the plain 0.6% of steps mix
  two levels at a hand-over and 0.1% fall back to the code's blend, and every ray pays for the room they need.
  The pass worth making is the one that takes the rare, heavy work out of the march that every ray runs.
- **Without the skip the base layer costs nearly twice** (13% against 7%): the height table's skip is worth about
  half the base layer's work at the mountain top.
- **The base layer is about two fifths of the far eye's work there** (7% of 17%): the rest is the 4 x 4's. That's
  the assessment's first missing fact, roughly (whole percents; the trace's own counts to come).

**Next:** each version's time from the two traces (Nsight's reader, once David has closed its window); then a
second batch to find the fewest trims that reach 128 and whether 102 (20 groups) is in reach: the hand-over and the
code's blend out together, then the floor, the rings and the third row added one at a time.

### The audit's versions, timed (build 177's two traces)

Each version's pass over the base layer, the middle of 20 frames, with how full the card was during it. From the
tables the trace itself exports (`BASE_UNLOCKED/D3DPERF_EVENTS.xls`, a pass's time a frame; `GPUTRACE_REGIMES.xls`,
its numbers a pass): Nsight's reader ground at the first of these traces for half an hour and was stopped (they're
four times last night's size); the exported tables needed no reader. (Nsight's window opened and closed did not take
the counters: the 45-second check passed after it. Why they went on 2026-11-14 stays unknown.)

| pass | the mountain top, ms | against the whole | in flight | the plain, ms | against the whole | in flight |
|---|---|---|---|---|---|---|
| the far eye itself (three layers) | 0.954 | | 17.5% | 0.833 | | 4.4% |
| `audit_whole` (its base layer alone) | 0.707 | 1 | 7.3% | 0.800 | 1 | 3.1% |
| `audit_hops` (to the first sample, no further) | 0.008 | 0.01 | 25% | 0.008 | 0.01 | 20% |
| `audit_samples` (no skip) | 1.016 | 1.44 | 11.5% | 1.242 | 1.55 | 10.0% |
| `audit_no_handover` | 0.641 | 0.91 | 7.7% | 0.637 | 0.80 | 3.5% |
| `audit_no_code_blend` | 0.634 | 0.90 | 7.6% | 0.685 | 0.86 | 3.4% |
| `audit_two_rows` | 0.644 | 0.91 | 8.1% | 0.770 | 0.96 | 4.3% |
| `audit_no_rings` | 0.678 | 0.96 | 10.3% | 0.667 | 0.83 | 3.4% |
| `audit_no_finish` | 0.681 | 0.96 | 7.4% | 0.795 | 0.99 | 2.9% |
| `audit_no_floor` | 0.680 | 0.96 | 7.5% | 0.790 | 0.99 | 2.9% |
| `audit_no_views` | 0.681 | 0.96 | 7.6% | 0.798 | 1.00 | 2.9% |
| `audit_lean` (the six together) | 0.547 | 0.77 | 8.5% | 0.563 | 0.70 | 5.7% |
| `audit_samples_lean` | 0.465 | 0.66 | 10.4% | 0.653 | 0.82 | 6.7% |

(At the mountain top every version but the first reads about 4% under the whole, the views' too, which changes
nothing a ray does: taken as the batch's noise there. On the plain the three that should change nothing read 0.99
to 1.00.)

- **Crossing the air to the first sample is 0.008 ms: nothing.** A pass for it would save nothing. The hops that
  cost are the ones between samples, inside the march.
- **The base layer alone is a waiting pass at both marks** (3 to 7% in flight). The mountain top's far eye is 0.954
  ms at 17.5% where its base layer alone is 0.707 at 7.3%: the difference is the 4 x 4 layer, and the fullness is
  its.
- **Two rare paths cost every ray.** With the hand-over's second read out the plain is 20% shorter and the mountain
  top's base 9%; with the code's blend out, 14% and 10%. On the plain 0.6% of steps mix two levels and 0.1% take the
  code's blend. Builder's reading: rays march 32 to a group in step, and when one ray of a group takes a slow path
  the other 31 wait for it; so a path one ray in a thousand takes can be in a good share of the groups' steps. It
  fits the size of the two effects; it isn't shown.
- **The march carries the code's blend three times over** (in the plain read, and twice in the hand-over's
  fall-back): 4.26K instructions with it, 3.25K without, 2.75K without the hand-over's read.
- **All six trims: 30% off the plain and 23% off the mountain top's base,** and 16 groups fit where 12 do. So the
  rare, heavy work taken out of the march every ray runs would help both kinds of mark: a shorter step where the
  card waits, more room and less work where it's full.
- **The skip is worth a third of the plain's march and of the mountain top's base** (without it 1.55 and 1.44
  times).
- **The other layers' empty groups cost 0.03 ms on the plain** (0.833 against 0.800): the launching of 27,000 groups
  that do nothing isn't the 0.17 ms that doesn't shorten with the chain.

**Next, off the screen and no time of David's:** each trim's picture against the whole's, to say which of them are
free (the views; the code's blend once the textures are in, if nothing shows where it's read; the floor) and which
change what's seen (the hand-over). Then the free ones out of the march into programs of their own, build 178.

### Each trim's picture against the whole march's (build 177, off the screen, five marks)

Each of the audit's versions drawn as the march itself (`FAR_TEST_VARIANT`, test-only) and set against build 176's
pictures at the spawn, the mountain top, the plain, the sea and high over the spawn:

| part taken out | pixels that differ at all | the most, of 255 | so |
|---|---|---|---|
| the views | none (the plain) | 0 | free, as it must be with no view on |
| the floor | none at any mark | 2 | free, and it gains nothing |
| the code's blend (nothing in its place) | none at the spawn and the plain; 0.9% at the mountain top, 0.1% over the sea, 0.4% high over the spawn | 58 | not free as it stands |
| the height table's far row | up to 0.7% | 4, but 135 over the sea | has to stay: over the sea rays run out of steps without it |
| the hand-over's second read | 3 to 11% | 134 | changes what's seen: it stays, made lighter |

- **Why the code's blend isn't free yet:** the textures can't answer in three cases, and the march falls back to the
  code's blend of eight cells for each (`spot_soft`): a place within half a cell of the world's floor (the cell under
  is rock, and no texture holds it), which at the coarse levels is everything near sea level (half a cell of the
  256s is 128 blocks); a brick with no texture; and outside a level or over the world. They're the far end of the
  long rays.

### Build 178, the march once the textures are in: the pass mark (written before the code)

The first piece broken out of the march (David's order: smaller passes first). The far eye gets two programs from
the one text, the choice fixed when each is built (one line, `LOADED`): **the whole march**, as now, for while the
world loads, with the initial blend chosen or with a view on; and **the march once the textures are in**, which
carries neither the code's blend nor the views. The game marches by the second when the textures are in, every
ring's too, the hardware's blend is chosen and no view is on.

What stands in for the code's blend in the second, so the picture is kept:
- within half a cell of the world's floor: the lowest cells read from their texture as any place is, mixed with rock
  by how far under (the same sum the code's blend makes);
- a brick with no texture, outside a level, over the world: rock or air by the four columns' bands (four small
  reads), which is all such a place can hold (a brick has a texture wherever a blend may read a band's cell: builds
  169 and 170).

**The marks:**
- **The picture, off the screen at the five marks:** build 176's as far as two rings repeat (the most 1 or 2 of
  255). In particular the pixels the bare trim changed (0.9% at the mountain top, 0.4% high over the spawn) don't
  differ. With `INCUS_FAR_LOADED=0` (the whole march kept on, for measuring) exactly as before.
- **Steps by kind on the plain:** the same but for "blends fallen back to the code": none.
- **Nsight's row** (David's reading): about what `audit_no_code_blend` read: 167 registers or fewer, Smem nothing,
  instructions down by a fifth or more. 12 groups still: this piece alone doesn't buy room.
- **Flown, against the whole march in the same session** (`INCUS_FAR_LOADED=0`): the plain and the sea 8% or more
  under; every mark at or under. The audit's bare trim gave 14% on the plain's base layer and 10% at the mountain
  top's; this build reads a texture where that read nothing, so less is expected. Under 5% on the plain is a miss.

**What it doesn't do:** fit more groups. That needs the other full places down too (the hand-over's second level
kept in hand, first), which is the next audit's batch, on this program.

### Build 178 built (unflown): the march once the textures are in

Two programs from the one text, by one line fixed when each is built (`far.wgsl`'s `LOADED`): `march_loading`, the
whole march as it was, and `march_all`, the march once the textures are in, without the code's blend or the views.
The far eye marches by the second when the hardware's blend is chosen, the atlas is in and every ring's bricks have
their textures, and no view is on (`FarEye::loaded`); `INCUS_FAR_LOADED=0` keeps the whole march on, for measuring
one against the other in a session. Where the textures can't answer as they stand, the second program:
- within half a cell of the world's floor, reads the lowest cells' texture and mixes rock in by how far under;
- one column short of a level's south or west side, reads the first column's texture and takes only its share;
- where a brick has no texture, outside a level or over the world, takes rock or nothing from the four columns'
  bands (`rock_share`, four small reads).

Off the screen:
- **The picture is build 176's at all five marks:** no pixel on the plain, the most 1 of 255 at the spawn, the
  mountain top, the sea and high over the spawn (two rings repeat to a last digit). The whole march kept on reads
  the same.
- **A first try missed at the mountain top and was mended before this:** four rays at the horizon came out lighter
  by 18 of 255, where far land meets the map's south edge. There the blend's nearer column is outside the level;
  the code's blend took the inside column's share, the first try took rock or nothing. The second case in the list
  above is the mend.
- **The steps by kind on the plain are build 176's to the last count** (1,173,357), and no step takes the code's
  blend (it isn't in the program).
- All 31 tests pass. Waiting on David's go: one trace for the registers' row, and two flights in one session, the
  second program against the whole march.

### Build 178 flown (2026-11-15, 08:04 and 08:10, hands off): 2 to 7% off every mark but the spawn, short of its mark

David: "go". Told before take-off; one screen, on mains, the card idle before (4%); no hitch in either flight. One
session, the same program file: the march once the textures are in (`logs/flight-1794751834`), then the whole march
kept on (`INCUS_FAR_LOADED=0`, `-1794752223`). Before them one trace at the mountain top for the registers' row
(`logs/nsight/b178/b178-mountain/incus_artus_2026_11_15_08_03_51.ngfx-gputrace`; its log says the second program
took over). Median ms:

| mark | the march: whole | once the textures are in | change | the card's whole: whole march | once the textures are in | change |
|---|---|---|---|---|---|---|
| the spawn | 0.358 | 0.357 | the same | 0.466 | 0.466 | the same |
| the mountain top | 0.948 | 0.920 | 3.0% less | 1.067 | 1.042 | 2.3% less |
| the bare plain | 0.832 | 0.776 | 6.7% less | 0.941 | 0.886 | 5.8% less |
| high over the spawn | 0.984 | 0.942 | 4.3% less | 1.103 | 1.060 | 3.9% less |
| low over the sea | 1.003 | 0.978 | 2.5% less | 1.114 | 1.090 | 2.2% less |
| a player's run | 0.893 | 0.860 | 3.7% less | 1.008 | 0.976 | 3.2% less |
| fast across the world | 0.642 | 0.627 | 2.3% less | 0.766 | 0.751 | 2.0% less |
| the plain, nodding | 0.798 | 0.750 | 6.0% less | 0.907 | 0.860 | 5.2% less |
| the sea, nodding | 0.941 | 0.921 | 2.1% less | 1.054 | 1.035 | 1.8% less |

(The whole march kept on reads as build 176 did, within 1% at seven marks and 2% high over the spawn.)

**Against the pass mark:**
- Every mark at or under the whole march: **met.**
- The plain 8% or more under: **short** (5.8%; over the 5% that would have been a plain miss).
- The sea 8% or more under: **missed** (2.2%).
- The picture and the steps, off the screen: met (above).

**Why less than the audit's 10 to 14%:** the audit's version put nothing where the code's blend had been, so its rays
also met less at the far end and its picture was wrong; this build reads a texture there. The audit prices a part's
going, not its replacement: its numbers are ceilings. (Said in the pass mark as "less is expected"; the mark's 8%
was still too high.)

**Kept:** build 178 (the picture kept, every mark the same or cheaper). The far eye's whole now: the sea 1.090 ms,
high over the spawn 1.060, the mountain top 1.042, the run 0.976, the plain 0.886, the spawn 0.466.

**Waiting:** David's reading of `march_all`'s row in the mountain top's trace (registers, the groups that fit, Smem).

### Build 178's row, read by David off Nsight's window (the mountain top's trace)

`march_all`, the march once the textures are in: 86% of the samples, **12 groups fit, 167 registers (164 live),
Smem 0**; its top waits Wait 35%, Long Scoreboard 16%, Selected 15%. Against what was expected: 167 or fewer, met;
Smem nothing, met; 12 groups still, as said. (The "instructions down by a fifth" can't be read: Nsight's count for
the same program was 8.53K in one trace and 4.26K in another, so it isn't a count of the program's own length.)

**What the rows so far say about how the card's compiler decides** *[read off the rows, not from NVIDIA]*:
- It seems to choose a number of groups and put by in memory what won't fit the registers that leaves: 12 groups
  with up to about 170 registers, 16 with about 128, 28 with about 73, 40 with about 51.
- So a program's need is its registers and what it puts by (Smem is 4 bytes a word for 64 rays a group): the whole
  march 168 and 18 put by, 186; with the floor out 178, the third row out 181, the finish out 179, the rings or the
  views out 186 still; the hand-over out 160; the code's blend out 167; all six out 128 and 3 put by, 131, at which
  it took 16 groups. Build 178's march: 167 and none.
- To fit 16 groups the march's need has to come from 167 to about 130.

### Build 179, the audit's second batch: what's expected (written before the trace)

`INCUS_FAR_AUDIT=2`: fourteen versions of the march once the textures are in, to find the fewest things to change
that bring its need to about 130: the hand-over's second read; and the things kept the whole way (the floor's cells,
the height table's third row, the rings' boxes, what the finish needs), each alone and each with the hand-over out,
then four and five together. Ceilings again: each prices a part's going. The first batch stays on
`INCUS_FAR_AUDIT=1`.

**Expected** *[guesses]*:
- `audit2_whole`: 167, nothing put by, 12 groups: the control.
- `audit2_no_handover`: a need of 150 to 160. With the code's blend gone the hand-over's two reads at once should be
  the fullest place left; if this reads 167 still, the fullest place is somewhere else (the plain read and the
  gather) and the hand-over isn't the first thing to change.
- Alone, the floor about 8 less, the third row about 5, the finish about 7; the rings none (as in the first batch).
- `audit2_lean4` (the hand-over, the floor, the third row and the rings out): about 130, 16 groups, as the first
  batch's six. Which three, or which two, already get there is the question.

**What it decides:** the order of the pieces after this. If the hand-over out with one other reaches 16 groups,
those two are next, each needing something to stand in that keeps the picture. If only all four do, the 16 groups
cost four changes, and the cut of the long rays may be the better next step for the marks that wait.

### The audit's second batch, traced and read (build 179, 2026-11-15, 08:25 to 08:32, hands off)

David: "go". Two traces, the mountain top and the plain, each at its mark with the march once the textures are in
marching (`logs/nsight/b179/b179-mountain/incus_artus_2026_11_15_08_28_34.ngfx-gputrace`, `.../b179-plain/
incus_artus_2026_11_15_08_31_57.ngfx-gputrace`). David read the mountain top's rows off Nsight's window; the times
are each version's pass over the base layer, from the traces' own tables. "Put by" is Smem as words a ray (4 bytes,
64 rays a group); the need is the registers and what's put by.

| version (of the march once the textures are in) | groups that fit | registers | put by | need | the mountain top, against the whole | the plain |
|---|---|---|---|---|---|---|
| `march_all` itself, and `audit2_whole` | 12 | 167 | 0 | 167 | 1 (0.698 ms) | 1 (0.738 ms) |
| the hand-over's second read out | 12 | 153 | 0 | 153 | 0.90 | 0.84 |
| the floor's cells out | 12 | 161 | 0 | 161 | 1.09 | 1.09 |
| the third row out | 12 | 161 | 0 | 161 | 1.05 | 1.06 |
| the finish out | 12 | 161 | 0 | 161 | 1.12 | 1.08 |
| the rings out | 12 | 167 | 0 | 167 | 0.94 | 0.83 |
| the hand-over and the rings | 12 | 153 | 0 | 153 | 0.80 | 0.68 |
| the hand-over, the floor and the rings | 12 | 142 | 0 | 142 | 0.79 | 0.68 |
| the hand-over and the floor | **16** | 128 | 15 | 143 | 0.99 | 0.97 |
| the hand-over and the third row | **16** | 128 | 22 | 150 | 0.97 | 1.02 |
| the hand-over and the finish | **16** | 128 | 17 | 145 | 1.04 | 1.00 |
| the hand-over, the floor and the third row | **16** | 128 | 7 | 135 | 0.91 | 0.96 |
| those and the rings (`lean4`) | **16** | 128 | 7 | 135 | 0.81 | 0.79 |
| those and the finish (`lean5`) | **16** | 127 | 0 | 127 | 0.77 | 0.76 |

**Against what was expected:**
- The control holds (167, nothing put by, 12 groups).
- The hand-over's second read out: a need of 153, for "150 to 160". Met: it's the fullest place left, worth 14.
- Alone: the floor 6 (for "about 8"), the third row 6 ("about 5"), the finish 6 ("about 7"), the rings none (as
  said).
- `lean4`: a need of 135 and 16 groups, for "about 130, 16 groups". Near.
- **"Which two already get there": the hand-over and any one other.** With the hand-over's second read out and one
  of the floor, the third row or the finish, the compiler took 16 groups, putting 15 to 22 words by to do it.

**What wasn't expected:**
- **The compiler's choice isn't a line at 128.** At a need of 150 it took 16 groups and put 22 words by; at 153
  and at 142 it stayed at 12 and put none by. *The rule read off the first rows ("about 170 for 12, about 128 for
  16") was too simple: it weighs something else too.*
- **16 groups bought with words put by don't make the base layer's pass shorter: they make it longer.** The
  hand-over out at 12 groups: 0.90 and 0.84. The hand-over and the floor out at 16 groups with 15 put by: 0.99 and
  0.97. The base layer alone is a waiting pass (3 to 7% in flight), so more room buys it nothing and each word put
  by is a read and a write in its chain. What 16 groups are worth where the card is full, this batch can't say: it
  marches the base layer alone.
- **The floor, the third row or the finish out alone: 5 to 12% longer** at both marks, with 6 fewer registers and
  nothing put by. Not understood (the first batch read them at 0.96 to 0.99 on the whole march).
- The rings' versions aren't fair prices: without the rings' levels the rays read rock or nothing outside the
  rings and do less.
- **One pass's time not understood:** in the mountain top's trace the far eye's own pass read 1.5 ms in 19 frames
  of 20 (0.92 in the other; 0.92 and 0.95 in build 178's flight and trace), at 10.8% in flight for 17.6%: the same
  rays' time spread thinner. The audit's frames aren't the game's (11 ms of passes a frame, the card never idle).
  The versions' own times agree between the two marks; that pass's time isn't used.

**What it decides, and what it can't:**
- The hand-over's second read is the next piece either way: 14 registers, and 10 to 16% of the base layer's march
  as a ceiling.
- Whether fitting 16 groups pays is a question about the marks that work, with the whole far eye marching, and it
  has to be flown: one of these versions marched as the far eye's own program, its picture wrong on purpose,
  against the march as it is. `lean5` (16 groups, nothing put by) is the ceiling of what room is worth; the
  hand-over out at 12 groups says how much of that is the shorter step and not the room.

### Build 180, what 16 groups are worth where the card is full: the pass mark (written before the flights)

No change to the march. A switch for measuring, `INCUS_FAR_AS`: one of the audit's second batch marched as the far
eye's own program once the textures are in, its picture wrong on purpose (as the step-limit flight's was), so a
flight prices it with every layer marching. Four flights in one session:

| flight | the far eye's program | groups that fit | put by |
|---|---|---|---|
| as it is | build 178's march | 12 | 0 |
| the hand-over out | `audit2_no_handover` | 12 | 0 |
| and the floor | `audit2_hand_floor` | 16 | 15 words a ray |
| and the third row | `audit2_hand_floor_rows` | 16 | 7 |

The last three differ from one another only in things that don't change the picture at the marks (the floor's
cells; the third row), so between them the difference is room against words put by. The first against the second
is the hand-over's second read, as a ceiling.

**The question:** where the card is full (the mountain top, high over the spawn), is the far eye shorter with 16
groups than with 12?

**The marks:**
- **If room pays there:** the two 16-group flights are under the hand-over-out flight at the mountain top and high
  over the spawn by a tenth or more (a quarter would be all of it: 12 groups' work in 16's room).
- **If it doesn't:** they're within 5% of it, or over it (the words put by cost more than the room gives).
- **Where the card waits** (the plain, the sea, the run): the 16-group flights no better than the hand-over out at
  12, and a little worse if the audit's base-layer times hold (0.97 to 0.99 against 0.84 to 0.90).
- **The hand-over out against the march as it is:** under at every mark; by the audit's base-layer times up to 16%
  on the plain and 10% at the mountain top, less with the finer layers in.
- *My guess, low:* room pays in part where the card is full, 5 to 15%.

**What it decides:** a tenth or more for room where the card is full, and the hand-over made lighter and one other
piece are next, each with something standing in that keeps the picture. Under 5%, and the registers aren't chased
further: the hand-over is still worth its shorter step, and the long rays cut into lengths comes next.

### Build 180 flown (2026-11-15, 08:43 to 09:09, hands off): 16 groups are worth 2 to 6% where the card is full

David: "go ahead". Told before take-off; one screen, on mains, the card idle before (5%); no hitch in any of the
four. One session, the same program file: `logs/flight-1794754181` (as it is), `-1794754571` (the hand-over out),
`-1794754961` (and the floor: 16 groups, 15 words put by), `-1794755350` (and the third row: 16 groups, 7 put by).
The last three's pictures are wrong on purpose. The march, median ms:

| mark | as it is | the hand-over out, 12 groups | and the floor, 16 groups | and the third row, 16 groups | 16 against 12: the floor | the third row |
|---|---|---|---|---|---|---|
| the spawn | 0.357 | 0.352 (1% less) | 0.337 (6%) | 0.310 (13%) | 4% less | 12% less |
| the mountain top | 0.934 | 0.889 (5%) | 0.872 (7%) | 0.851 (9%) | 2% less | 4% less |
| high over the spawn | 0.975 | 0.937 (4%) | 0.916 (6%) | 0.876 (10%) | 2% less | 7% less |
| fast across the world | 0.642 | 0.599 (7%) | 0.528 (18%) | 0.561 (13%) | 12% less | 6% less |
| the bare plain | 0.776 | 0.664 (14%) | 0.685 (12%) | 0.693 (11%) | 3% more | 4% more |
| the plain, nodding | 0.751 | 0.704 (6%) | 0.731 (3%) | 0.734 (2%) | 4% more | 4% more |
| low over the sea | 0.980 | 0.976 (the same) | 0.977 (the same) | 1.013 (3% more) | the same | 4% more |
| the sea, nodding | 0.920 | 0.909 (1%) | 0.910 (1%) | 0.962 (5% more) | the same | 6% more |
| a player's run | 0.861 | 0.844 (2%) | 0.839 (3%) | 0.922 (7% more) | 1% less | 9% more |

(With the third row out the rays over the sea and on the run take more steps, as the first batch's picture said:
that column is dearer there for its steps, not its room.)

**Against the pass mark:**
- **Does room pay where the card is full? A little: 2 and 4% at the mountain top, 2 and 7% high over the spawn**,
  for "a tenth or more" if it pays and "within 5%" if it doesn't. Three of the four are under 5%. My guess (5 to
  15%) was high; the low end at best.
- **Where the card waits, 16 groups with words put by cost 3 to 4%** (the plain, the plain nodding): as the audit's
  base-layer times said.
- **The hand-over's second read out, as a ceiling:** under at every mark, as said; 14% on the plain, 4 to 7% at the
  mountain top, high over the spawn and fast across, and next to nothing over the sea, on the run and at the spawn.
- The spawn and fast across the world gain most from room (4 to 12%); they're the cheapest marks.

**What it decides, by the mark's own rule:** the registers aren't chased as a goal. At the dearest marks (the far
eye's whole: high over the spawn 1.096 ms, the sea 1.092, the mountain top 1.056) everything this stretch could
still give is small and comes with a wrong picture until something stands in: the hand-over out 4% and 4% and
nothing; 16 groups a further 2 to 7% at two of them and a loss at the third. The measured prize that's left is the
long rays': with rays stopped at 160 steps the sea's march was 45% shorter and the run's 29% (build 176).

**What the stretch gave, kept:** the march once the textures are in (build 178): 2 to 7% off every mark but the
spawn with the picture kept, and the march's need down from 186 to 167 with nothing put by. And what it found: the
hops to the first sample cost nothing; rare, heavy paths cost every ray; the compiler buys groups with words put
by, and that costs where the card waits.

**What isn't answered:** what the mountain top's and high over the spawn's time is, if it's neither their longest
rays (rays stopped at 160: 9% and 4%), nor room (2 to 7%), nor the hand-over (4 to 5%). The 4 x 4 layer is the
largest known part: the mountain top's base layer alone marches in 0.71 ms of its 0.95.

## The long rays cut into lengths  *(plan 2.2; David's order, its third part)*

David, 2026-11-15, on build 180's flights: "go on with the cut". The Oracle's step 4, with one difference: only the
rays that were long the frame before are cut, by a list, where it cut every ray (a few hundred are long, in a thin
band at the horizon; cutting all would march far lengths behind every near hill).

**How, as told to David:** a ray that was long the frame before is cut at set distances; each length is marched by
its own thread in the same dispatch, and a small pass joins them front to back, so the frame waits for the longest
length and not the longest ray. A ray that turns out long without notice keeps the frame before's pixel for one
frame and is cut from the next. Nothing is kept between frames but the list. Three stages, each with its mark:
1. the list of long rays alone, the picture untouched;
2. the lengths and the join for listed rays (a flight, and David's look at the horizon band);
3. the frame's notice for newly long rays (the moving and nodding marks).

And beside it, by David's word to Counsel ("yes"): each layer's rays and steps counted in every flight's report,
the 4 x 4's above all, since the mountain top and high over the spawn become the high spot once the plains come
down, and the 4 x 4 is their largest known part.

### Build 181, the cut's first stage, the list of long rays: the pass mark (written before the code)

**What changes:** a ray that took 128 steps or more adds its pixel to this frame's list and keeps its place in the
list beside its distance, for the frame after. The list rides after the tiles' starts in the buffer the march
already has (two lists, this frame's and the frame before's), so the march keeps to the library's default limits.
Nothing reads the list yet. The count of the longest ray gains four numbers a layer: its rays marched, their steps
summed, its rays of 128 steps or more, and how many of those were listed.

**The marks:**
- **The picture, off the screen at five marks:** build 178's (the most 1 of 255).
- **The list, off the screen:** it holds as many as there are rays of 128 steps or more over the three layers
  (unless that's over the 8,192 it holds), each once, and every layer's "listed" equals its "128 or more".
- **The report:** on the plain the 8 x 8 marches 64,000 rays and about 1,170,000 steps, and the 4 x 4 none.
- **Flown, against build 180's flight as it is:** every mark's march within 1%. The bookkeeping is a compare and,
  for a few hundred rays, a write; it should cost nothing. Over 2% at any mark is a miss (code added to the march
  has cost before with nothing switched on).

### Build 181 built (unflown): the list of long rays, and what each layer marches

As the pass mark said. A ray of 128 steps or more adds its pixel to the frame's list (`far_all.wgsl`'s
`list_long`), which rides after the tiles' starts in the buffer the march already had (now words written by many
rays at once: two lists of 8,192, this frame's and the frame before's); its place in the list goes beside its
distance. No ray is listed in a frame the layers were remade. Nothing reads the list yet. The count of the longest
ray holds eight numbers a layer now, and a flight's report says what each layer marches.

Off the screen:
- **The picture is build 178's at the five marks** (no pixel at three, the most 1 of 255 at two).
- **The list holds each long ray once** (a test: its count is the long rays' over the three layers, each carries
  its place, none twice).
- **What each layer marches, at the flight's size:**

| mark | the 8 x 8: rays, steps, long | the 4 x 4: rays, steps, long | the 2 x 2 |
|---|---|---|---|
| the bare plain | 64,000; 1,178,000; 890 | none | none |
| the mountain top | 64,000; 2,039,000; 983 | 69,700; 4,556,000; 600 | none |
| low over the sea | 64,000; 1,355,000; 268 | 10,000; 490,000; 485 | none |

- **The mountain top's work is the 4 x 4's: 4.56 million steps of its 6.6 million,** from 69,700 rays at 65 steps
  each, where a base ray takes 32. That's the answer to "what is the mountain top's time", as far as counts say: a
  little over a quarter of the 4 x 4's pixels are wanted there, and each marches twice as long as a base ray though
  it starts just short of where the base ray touched. Why they take so many steps isn't looked at yet.
- All 31 tests pass. Waiting on David's go for one flight, against build 180's as it is.

### Build 181 flown (2026-11-15, 09:32, hands off): the list's bookkeeping costs 4 to 11%. A miss

David: "go". Told before take-off; one screen, on mains, the card idle before; no hitch. Report
`logs/flight-1794757116`, beside build 180's as it is (`-1794754181`, 08:43). The march, median ms:

| mark | build 180 | build 181 | change |
|---|---|---|---|
| the spawn | 0.357 | 0.388 | 8.7% more |
| the mountain top | 0.934 | 0.990 | 6.0% more |
| the bare plain | 0.776 | 0.826 | 6.4% more |
| high over the spawn | 0.975 | 1.087 | 11.5% more |
| low over the sea | 0.980 | 1.041 | 6.2% more |
| a player's run | 0.861 | 0.915 | 6.3% more |
| fast across the world | 0.642 | 0.665 | 3.6% more |
| the plain, nodding | 0.751 | 0.797 | 6.1% more |
| the sea, nodding | 0.920 | 1.000 | 8.7% more |

**Against the pass mark:** "every mark within 1%; over 2% at any mark is a miss". **Missed at every mark.** The
picture, the list and the counts were met off the screen; the cost wasn't looked for there, and can't be.

**What costs, found in the translator's source and not yet shown by a flight:** to hold the list, the buffer of the
tiles' starts became words that many rays write at once, and every ray reads its tile's start from it. The
translator (naga, to the card's language) writes such a read as an atomic "or with nothing", a read and a write
locked together, where it wrote a plain read: every ray of every layer, every frame, on a word its 63 neighbours
are locking too. The marks with the most rays marching lost most (high over the spawn, 253,000 rays: 11.5%).
*Builder's mark said "a compare and, for a few hundred rays, a write"; the read was overlooked.*

**The mend, build 182:** the tiles' starts leave that buffer. By the one dispatch the start pass writes them into a
corner of the frame before's distances (a picture the march already reads, under the base layer's part, which is
empty there), and the march reads them from it as any picture; the buffer then holds only the lists, which a few
hundred rays touch. One layer after another (`INCUS_FAR_SIDE=0`) is as it was.

**What the flight did give: each layer's rays and steps at every mark** (the most over the measured frames):

| mark | the 8 x 8: steps, long rays | the 4 x 4: rays, steps, long rays |
|---|---|---|
| the spawn | 1,579,000; 261 | 13,500; 793,000; 18 |
| the mountain top | 2,039,000; 983 | 69,700; 4,556,000; 600 |
| the bare plain | 1,178,000; 890 | none |
| high over the spawn | 2,263,000; 914 | 188,900; 3,764,000; 977 |
| low over the sea | 1,355,000; 268 | 10,000; 490,000; 485 |
| a player's run | 1,592,000; 780 | 22,800; 940,000; 1,639 |
| fast across the world | 1,654,000; 256 (35,800 rays) | 140,100; 3,182,000; 399 |
| the plain, nodding | 1,191,000; 964 | none |
| the sea, nodding | 1,357,000; 312 | 10,000; 498,000; 511 |

The 2 x 2 marches nothing at any mark. Every long ray was listed at every mark (the most at once 2,419, on the run:
well inside the list's 8,192). **At the three dearest marks that aren't the sea, the 4 x 4 marches most of the
steps:** the mountain top 4.56 million of 6.6, high over the spawn 3.76 of 6.0, fast across the world 3.18 of 4.8.

### Build 182 built (unflown), the mend to the list's cost: its pass mark, and what was checked

**What changes:** by the one dispatch the tiles' starts leave the list's buffer. The start pass (its program now
made from the one dispatch's text) writes them under the base layer's part of the frame before's distances, where
nothing else is, and every ray reads its tile's start from there as it reads any picture. The buffer that many rays
write at once holds only the long rays' two lists, touched by the rays that are long. In a frame the layers were
remade the starts go into both copies of the distances (its march reads each in turn). One layer after another
(`INCUS_FAR_SIDE=0`) keeps its plain buffer of starts, as before build 181.

**The mark for the flight** (the same as build 181's, which missed): every mark's march within 1% of build 180's as
it is (the spawn 0.357 ms, the mountain top 0.934, the plain 0.776, high over the spawn 0.975, the sea 0.980, the
run 0.861, fast across 0.642, the plain nodding 0.751, the sea nodding 0.920); over 2% at any mark is a miss. It
also tests the reading of build 181's cost: if the 4 to 11% is still there, the tiles' read wasn't it.

**Off the screen:** the picture is build 178's at the five marks (no pixel at three, the most 1 of 255 at two); the
plain's counts are build 181's to the last step (1,178,297 steps, 890 long rays, all listed); the list's test
passes; one layer after another still draws. All 31 tests pass. *(A test of the main memory's reading failed once
and passed on the next run: it asks that Windows' "in memory now" never be over its "most ever", which two readings
a moment apart needn't keep. Not this build's; written down to mend.)*

### Build 182 flown (2026-11-15, 09:54, hands off): the list costs nothing now. The cut's first stage is through

David: "go". Told before take-off; one screen, on mains, the card idle before (5%); no hitch. Report
`logs/flight-1794758479`, beside build 180's as it is and build 181's. The march, median ms:

| mark | build 180 | build 181 | build 182 | 182 against 180 |
|---|---|---|---|---|
| the spawn | 0.357 | 0.388 | 0.359 | 0.6% more |
| the mountain top | 0.934 | 0.990 | 0.927 | 0.7% less |
| the bare plain | 0.776 | 0.826 | 0.777 | 0.1% more |
| high over the spawn | 0.975 | 1.087 | 0.980 | 0.5% more |
| low over the sea | 0.980 | 1.041 | 0.986 | 0.6% more |
| a player's run | 0.861 | 0.915 | 0.865 | 0.5% more |
| fast across the world | 0.642 | 0.665 | 0.612 | 4.7% less |
| the plain, nodding | 0.751 | 0.797 | 0.752 | 0.1% more |
| the sea, nodding | 0.920 | 1.000 | 0.927 | 0.8% more |

- **Against the pass mark:** within 1% at eight marks: met. Fast across the world is 4.7% under, outside the band
  on the cheap side; that mark has read 0.599 to 0.665 across today's flights of the same march, so it's taken as
  that mark's own spread, not a gain.
- **The reading of build 181's cost is shown:** with every ray's tile start read as a picture again and only the
  long rays touching the buffer many rays write, the 4 to 11% is gone.
- The start pass is a little shorter for writing a picture: 0.007 to 0.005 ms.

### Build 183, the cut's second stage, the lengths and the join: the pass mark (written before the code)

**What changes:** a ray in the frame before's list is marched as lengths. Its own thread marches the near length,
to the first cut; sixteen more threads, in a strip added to the dispatch's grid, each march one length beyond, to
the next cut; all put what they gathered (lit, by their own first surface) into the list's buffer; and a small
pass after the march joins a ray's lengths front to back, finishes it as any ray is finished (the sky behind, fog,
the sky's weight, what the layer below is worth) and writes its pixel, its distance and its steps. The cuts are a
kilometre out and then every half doubling of distance, shifted ray to ray by the ray's own number so they never
line up into a ring. A listed ray stays listed while its lengths' steps sum to 96 or more. A ray that's long and
not yet listed marches whole, as now, and lists itself (the third stage evens that out). The one march is still in
the program once: a pixel's thread and a length's thread come to it by the same road.

**What it may cost every ray:** each pixel reads its own place in the frame before's distances, to know whether
it's cut. One more read of a picture a ray, where the finer layers already read nine.

**The marks:**
- **Off the screen, the picture cut against uncut** (`INCUS_FAR_CUT=0`, for measuring), five marks, the second
  frame on: the same in kind. Only pixels of listed rays differ, by 8 of 255 or less. More, or a ring or a seam at
  a cut, and it's mended before it flies (a length's first step gathers nothing: that's where it could show).
- **The longest chain** (a new count: the most steps any one thread takes): under 140 on the plain and over the
  sea, where the longest rays are 324 and 409.
- **Flown, one session, cut against uncut:**
  - the plain's march from 0.777 to 0.55 or under, the sea's from 0.986 to 0.60 or under (rays stopped at 160
    steps gave 0.494 and 0.554; the lengths are shorter than that, the join and the strip are extra);
  - the run and the two nodding marks a tenth or more under (newly long rays still march whole there);
  - the spawn, the mountain top and high over the spawn within 2% (the one read more).
- **A miss:** the plain over 0.65; or any of the spawn, the mountain top and high over the spawn over 2% dearer.

### Build 183 built (unflown), the cut's second stage: what was built, where it left the pass mark's words, and what was checked off the screen

**What was built.** By the one dispatch, a ray in the frame before's list is marched as up to 17 lengths side by
side: its own thread the near one, to the first cut (1,024 blocks out, shifted ray to ray by up to half a
doubling); sixteen threads in a strip at the right of the dispatch's grid the lengths beyond, each half a doubling
of distance on, the last to the ray's end. Each puts by what it gathered; a pass after the march (64 x 64 threads,
in the 4 x 4's place in the timing, which the one dispatch left empty) joins a ray's lengths front to back into
what the whole ray would have seen and finishes it as any ray is finished (lit once, by its first surface). The
one march is in the program once.

**Where the build left the pass mark's words** (written here before any flight):
- **A length gathers up to its cut and no further, and the next begins its steps exactly there.** The mark said
  only "a length's first step gathers nothing: that's where it could show". A length stopped at its last step short
  of the cut would have left what lies between that step and the next length's first to neither. So the march has
  a place to stop: it goes on until a step has landed at the cut or past it, and gathers the step that crosses the
  cut only as far as the cut.
- **Each length begins as the ray stood there the frame before** (not in the mark at all; found by the picture
  check, below): letting through what the ray still let through at that cut, and looking before it skips if the
  ray was in haze there. The join writes both down for the frame after. With the eye still this is exact from the
  second frame a ray is cut; while the eye moves it's a frame behind.
- **A cut ray stays listed down to a quarter of the line (32 steps), not three quarters (96).** A cut ray's
  lengths' steps can come to well under the whole ray's; at 96, a ray left the list on its lengths' count and came
  straight back, marched whole every other frame (the mountain top: one ray of 172 steps; the sea: one of 143).
- **The picture's check is of the sixth frame**, not "the second frame on": a ray is listed in the second, cut
  from the third, and its lengths begin by the frame before's from the fourth.
- **The levels' table became a uniform** (it was a storage buffer): the lengths need a buffer of their own, and
  the march was at the library's default of eight storage buffers a stage. Every ray reads the table.
- **The list holds 4,096 rays** (it held 8,192; the most long rays at once over build 181's flight: 2,419).
- **For measuring:** `INCUS_FAR_CUT` (0: no ray cut; or the first cut's distance) and `INCUS_FAR_LONG` (the steps
  that make a ray long). The flight's report gains, for each layer, how many rays were cut and the longest chain
  (the most steps any one thread took).

**Three flaws of mine the picture check found, each mended before this was written:**
1. *A cut ray's first surface put too deep.* Where earlier lengths had part covered a ray, the join took the
   first surface as the length's own halfway point: up to 500 blocks behind the true one (high over the spawn),
   so the ray read its light from the wrong place: two rays 36 of 255 lighter.
2. *A cut ray given the wrong cell size for its surface*, so it told the layer below it was worth drawing where
   the uncut ray says it isn't, and finer pixels drew sky over a far skyline: 35 blocks along the sea's horizon, up
   to 73 of 255. (Seen as a broken band of far hills, cut against uncut.)
3. *The first surface still a tenth out* where haze had covered part of a ray before the length that holds it;
   the layer below fades in by that distance, a quarter more or less for a tenth: 22 blocks along the sea's
   skyline, 24 to 45 of 255. This is what "each length begins as the ray stood the frame before" mends: the
   length that holds the ray's first surface now finds it itself, where the whole ray's light falls to a half.

**Off the screen, 2,560 x 1,600, the sixth frame, cut against uncut (`INCUS_FAR_CUT=0`):**

| mark | rays cut | the longest ray, uncut | the longest chain, cut | pixels that differ at all | the most, of 255 |
|---|---|---|---|---|---|
| the spawn | 279 | 153 | 132 | 608 of 4,096,000 | 2 |
| the mountain top | 1,587 | 302 | 147 | 0.5% | 70, one ray; the rest under 8 |
| the bare plain | 890 | 324 | 127 | 0.3% | 8 |
| low over the sea | 753 | 409 | 127 | 0.1% | 10, one 4 x 4 ray; the rest under 8 |
| high over the spawn | 1,891 | 261 | 127 | 0.1% | 17, one ray; the rest 8 or under |

- **Against the mark's picture line** (only listed rays' pixels differ, by 8 of 255 or less; no ring or seam):
  no ring and no seam at any mark; the spawn and the plain within 8. **Not met to the letter at three marks, by
  three rays:** one at the mountain top (70) and one high over the spawn (17), which are the flaw below, not the
  cut's own; and one 4 x 4 ray at the sea's skyline (10). The test's own picture differs run to run by up to 6 of
  255 at a few rays (two runs of the same uncut build at the spawn, drawn to see).
- **The two rays, and a flaw that was there before the cut (written down, not mended: David hasn't seen it):** a
  ray's last stride before its end (the map's edge, with its margin) can go past the end and never be looked at,
  so a strip of sea a blend reaches there is caught or missed by where a ray's steps happen to fall. A cut ray's
  lengths begin at other places than the whole ray's steps fell, so they catch it where the whole ray missed it
  (the mountain top's ray: the sea at 44 km, undimmed by fog because the ray's first surface is the summit 33
  blocks off; uncut it shows sky there). Either is by chance; the mend would be the march's (never stride past the
  end unlooked), which changes the uncut picture at the map's edge and so is its own build.
- **Against the mark's chain line** (under 140 on the plain and over the sea): met, 127 at both, from 324 and 409.
  At the mountain top 147.
- **What sets the chain now:** rays just under the line of 128 steps, which march whole (127); and a cut ray's
  near length, to the first cut: up to 118 steps on the plain, 104 at the sea, 132 at the spawn, 147 at the
  mountain top. The lengths beyond the first cut take up to 93. Both can be brought down without building
  (`INCUS_FAR_LONG`, `INCUS_FAR_CUT`), after this is flown as it is.
- **What the cut costs in steps:** under 1% more in all at every mark (the plain 1,178,297 to 1,186,818).
- **The uncut picture is build 182's** (640 x 400, the references of build 182): no pixel at the spawn and the
  plain, the most 1 of 255 at the other three, as build 182 against 178. The march's own text changed in three
  places: the place to stop; where a ray's first surface is kept as it goes (kept at every step that gathers while
  over half is let through, the last of which is the same step as before); and the table's reads.
- All 31 tests pass; the far eye's test now runs a frame that lists and a frame that cuts, and checks that every
  long ray is listed once and that listed rays were cut. `FAR_TEST_LENGTHS` prints each cut ray's lengths for a
  look (the tool that found the three flaws).

**Known and left, written down:**
- A cut that falls between a ray's last gathering over the world's floor and its going under the floor leaves the
  ground it didn't gather to the sky (the whole ray fills it in). Not seen in any picture.
- `INCUS_FAR_AUDIT` with the cut on prices cut rays' near lengths, not whole rays: fly it with `INCUS_FAR_CUT=0`.
- The far eye's picture test isn't the same run to run at full size (up to 6 of 255 at a few rays): something
  made off the frame arrives at different moments. Not this build's; to find.

**The flights asked for, one session, hands off:** uncut (`INCUS_FAR_CUT=0`, which against build 182's flight
also says what this build's changes cost every ray), then cut as it is; the marks as the pass mark has them. Then,
if David says, two probes for where the chain goes next: the line at 64 steps, and that with the first cut at 384.

### Build 183 flown (2026-11-15, 12:02 to 12:28, hands off): a miss. The cut halves the march where the card waits and makes it a sixth to a third dearer where the card works

David: "go with probes". Told before take-off; one screen, on mains, the card idle before (3 to 9%); no hitch. Four
flights, one session, build 183 at `1c20872`: uncut (`INCUS_FAR_CUT=0`, `logs/flight-1794766116`), cut as it is
(`logs/flight-1794766509`), and two probes (the line at 64 steps, `logs/flight-1794766903`; that with the first cut
at 384 blocks, `logs/flight-1794767298`). Beside build 182's (`logs/flight-1794758479`). The march, median ms:

| mark | build 182 | 183 uncut | 183 cut | cut against uncut | the longest chain, uncut to cut | rays cut |
|---|---|---|---|---|---|---|
| the spawn | 0.359 | 0.359 | 0.370 | 3.1% more | 153 to 132 | 282 |
| the mountain top | 0.927 | 0.919 | 1.082 | 17.7% more | 302 to 147 | 1,902 |
| the bare plain | 0.777 | 0.769 | 0.396 | 48.5% less | 324 to 127 | 1,455 |
| high over the spawn | 0.980 | 0.939 | 1.235 | 31.5% more | 261 to 127 | 2,682 |
| low over the sea | 0.986 | 0.969 | 0.609 | 37.2% less | 409 to 127 | 1,591 |
| a player's run | 0.865 | 0.852 | 0.994 | 16.7% more | 337 to 154 | 4,346 |
| fast across the world | 0.612 | 0.621 | 0.810 | 30.4% more | 227 to 189 | 5,234 |
| the plain, nodding | 0.752 | 0.743 | 0.423 | 43.1% less | 333 to 131 | 1,769 |
| the sea, nodding | 0.927 | 0.914 | 0.726 | 20.6% less | 429 to 128 | 2,789 |

(Rays cut: the most at once in each layer over the measured frames, summed. The join, in the 4 x 4's place: 0.008
to 0.014 ms.)

**Against the pass mark:**
- The plain's march 0.55 or under: **met**, 0.396. The sea's 0.60 or under: **missed by a hair**, 0.609.
- The two nodding marks a tenth or more under: **met** (43% and 21%). The run a tenth or more under: **missed**, it's
  16.7% dearer.
- The spawn, the mountain top and high over the spawn within 2%: **missed at all three** (3.1%, 17.7% and 31.5%
  dearer). By the mark's own words that is **a miss**.
- **The high spot rose.** The far eye's whole, cut: high over the spawn 1.370 ms (1.057 uncut), the mountain top
  1.220 (1.041), the run 1.133 (0.970), fast across 0.948 (0.746), the sea nodding 0.862 (1.029), the sea 0.739
  (1.083), the plain nodding 0.548 (0.855), the plain 0.519 (0.880), the spawn 0.490 (0.469). The dearest mark was
  1.083; it's 1.370.
- **Uncut, the build costs every ray nothing:** within 1.7% of build 182's at eight marks, all but one on the cheap
  side (high over the spawn 4.2% under, fast across 1.5% over: those two marks' own spread). The place to stop, the
  table as a uniform and the first surface kept as it goes are free.

**What the probes say** (both made every mark dearer than the cut as it is):
- The line at 64 steps: more than twice the rays are long, and the list of 4,096 filled at five marks (at the
  mountain top the 8 x 8's rays took it all and none of the 4 x 4's were listed). The plain, where it didn't fill:
  2,574 rays cut for 1,455, the chain 118 for 127, and the march 0.498 for 0.396. More rays cut cost more than the
  shorter chain gave.
- That with the first cut at 384 blocks: the plain's chain 63, half of before, and its march 0.510: no shorter.
  **With the cut on, the plain's time is no longer its longest chain.**

**The reading** (a reading: not yet seen in a trace):
- The steps are the same: high over the spawn 6.03 million uncut, 6.05 million cut (0.4% more), and the march is a
  third longer. So the cost isn't steps taken, it's threads kept waiting. A group of 32 threads runs as long as
  its longest; the rest wait.
- Every cut ray gets sixteen threads in the strip whatever its length, and its lengths' steps are very uneven
  (most under 20, one or two of 60 to 90; past the ray's end, none). Thirty-two threads of the strip hold two
  rays' lengths, and run as long as the longest of them: about sixteen threads times 75 steps a ray, some 1,200
  thread-steps, for lengths that hold 150 to 250 steps between them. Uncut, a long ray holds its 31 neighbours
  for its length, but its neighbours in the horizon band are mostly long too, so it wastes less.
- Where the card waits (the plain, the sea: it's about 5% busy), wasted threads cost nothing and the shorter chain
  is all gain. Where the card works (the mountain top, high over the spawn: it's at its ceiling), wasted threads
  are the cost. And a list kept down to 32 steps holds more rays than are long (the run: 4,346 cut for 2,400
  long), each with its sixteen threads.
- So the measure to bring down is thread-steps in the strip, not the chain alone.

**What stands:** the machinery (the place to stop, the lengths' stacks, the join, each length beginning as the ray
stood the frame before), the picture (cut against uncut within 8 of 255 but for three rays), and the gain where
the card waits. **What doesn't:** sixteen threads a ray in fixed cuts.

**The mend proposed (build 184; its pass mark before its code, once David says):** as many lengths as a ray needs
and no more (its steps over about a hundred), cut where the frame before says a hundred steps had gone by (the
join knows each length's steps and its ends, and re-cuts for the frame after), and only those lengths' threads in
the strip, packed side by side from its top (the join gives each length a place in a list of lengths, as a long ray
gets its place in the list of rays). Then a group of the strip holds 32 lengths of about a hundred steps each, a
cut ray costs about its own steps, and a long ray no longer holds its neighbours for more than a hundred. If the
reading is right, that is a gain at the working marks too; if it's wrong, the flight of build 184 says so.

**Left as it is meanwhile:** build 183 marches with the cut on unless `INCUS_FAR_CUT=0` (every mark still under
David's 2 ms: the dearest at the 99th is fast across the world, 1.60 ms). David's look at the horizon band, standing and nodding, can be had
on it now: whether a length beginning by the frame before shows while the view moves bears on the mend.

### Build 184, the cut off unless asked, and a count of threads kept waiting: the pass mark (written before the code)

David, 2026-11-15, asked whether to switch the cut off by default and go on to the mend: "yes, lets try your idea to
make the cut work smarter, if it doesn't work it doesn't work. its pretty fast in general now." He'd seen the spread
widen himself: "it looks like it increases the range of speeds between the mark points". Builder's word to him: if
the mend's flight doesn't bring the slowest view to uncut or under, the cut is dropped. *(The numbering: this
build, 184, is the switch and the count, flown on build 183's way of cutting to test the reading before anything is
built on it. The mend itself is build 185, with its own pass mark after this count. Where build 183's entry says
"build 184" for the mend, read 185.)*

**What changes:**
1. **The cut is off unless asked.** `INCUS_FAR_CUT` set to a distance of 64 blocks or more turns it on, the first
   cut there (1024: as build 183 flew); unset or 0, no ray is cut. The game as launched marches as build 183 uncut.
2. **A count of threads kept waiting** (Counsel's suggestion, so a flight reads the cause and not only the time),
   on flights, once a second, beside the rays' count and outside the passes' timing. Over the march's grid, for
   every group of 32 threads: the steps its threads took (busy), and its longest thread's steps times 32 (held:
   a group runs as long as its longest thread, and the rest wait). In two parts: the layers' threads, and the
   strip's (the cut rays' lengths). A thread's steps: a whole ray's steps; a cut ray's own thread, its near
   length's; a thread of the strip, its length's. And how many of the strip's threads marched at all.
   - *Taken, not known:* that the card runs an 8 x 8 group as two groups of 32, its first four rows and its last
     four. So the same count by whole groups of 64 is kept beside it; the truth is one or between.
   - *What it leaves out:* a thread's work before its first step (finding where its ray starts: a dozen reads) and
     after its last. The strip's threads that marched say how many times that is paid.

**The reading it tests** (build 183's flight): the steps are the same cut and uncut, and what the cut costs where
the card works is threads kept waiting, mostly in the strip.

**The marks:**
- **The count is sound** (off the screen, before it flies): busy, the layers' and the strip's together, is the
  rays' steps summed, to the step, cut and uncut.
- **Flown, one session, the game as launched (uncut) and with `INCUS_FAR_CUT=1024`:** each mark's march within 2%
  of build 183's same flight (uncut: the plain 0.769 ms, high over the spawn 0.939, the mountain top 0.919; cut:
  0.396, 1.235, 1.082). Over that, the count or the switch changed the march, and that's found first.
- **The reading stands** if, at the mountain top and at high over the spawn, held thread-steps (the layers' and the
  strip's together) are up, cut over uncut, by half or more of what the march's time is up: 9% or more at the
  mountain top (its march 17.7% longer), 16% or more high over the spawn (31.5% longer), with busy within 1%.
- **The reading falls** if held is up by under 5% at both. Then nothing is built on it: the mend isn't begun, and
  the next step is Nsight at those two marks, cut against uncut.
- **Between the two:** said as it is, with the strip's threads that marched beside it (their work before a first
  step is the other place the cost could be).
- No mark for the plain or the sea: the card waits there, and threads kept waiting cost it nothing. Their numbers
  are reported.

### Build 184 built (unflown): the cut off unless asked, and the count of threads kept waiting. What the count says off the screen, before any flight

**Built as the mark says:** `INCUS_FAR_CUT` unset or 0, no ray is cut (a distance turns it on); the count
(`far_waiting.wgsl`), run with the rays' count on a flight: the march's grid gone over group by group, each
thread's steps read back from what the march kept (a whole ray's beside its distance; a cut ray's lengths' from
their stacks). One thing mended on the way: the join summed a cut ray's steps only up to where it was covered; the
lengths behind that marched too, so their steps and the longest of them are now counted (the list's line and the
reports' longest chain read a little higher for it).

**The count is sound:** on the test's own view, busy is the rays' steps summed to the step, cut (195,875) and
uncut (195,804). All 31 tests pass; the far eye's test turns the cut on itself, so the lengths and the join stay
tested with the game's cut off.

**Off the screen, 2,560 x 1,600, the sixth frame** (thread-steps; held by groups of 32; by 64s the shares are the
same within a point):

| mark | | steps taken | held | of it waiting |
|---|---|---|---|---|
| high over the spawn | uncut | 6,027,147 | 7,497,792 | 20% |
| | cut: the layers | 5,761,099 | 7,192,512 | 20% |
| | cut: the strip | 297,049 | 1,465,728 | 80% |
| | cut: in all | 6,058,148 | 8,658,240 (15.5% more) | 30% |
| the mountain top | uncut | 6,594,798 | 7,485,632 | 12% |
| | cut: the layers | 6,401,040 | 7,223,744 | 11% |
| | cut: the strip | 226,852 | 1,240,320 | 82% |
| | cut: in all | 6,627,892 | 8,464,064 (13.1% more) | 22% |
| the bare plain | uncut | 1,178,297 | 1,392,640 | 15% |
| | cut: the layers | 1,083,506 | 1,246,528 | 13% |
| | cut: the strip | 130,502 | 616,416 | 79% |
| | cut: in all | 1,214,008 | 1,862,944 (33.8% more) | 35% |

- **Against the mark, off the screen:** the mountain top's held is up 13.1% where its march flew 17.7% longer (the
  line: 9% or more): the reading stands there. High over the spawn's is up 15.5% where its march flew 31.5% longer
  (the line: 16% or more): a hair under, so "between the two". The flight's own numbers are still owed: flown, more
  rays were cut than stand in the list here (high over the spawn 2,682 for 1,891), and the strip's part grows
  with them.
- **Four fifths of the strip's thread-steps are waiting.** A cut ray's lengths take about 150 steps between them
  in the strip and hold about 780 thread-steps: sixteen threads a ray, held to the longest length in their group.
- **What the count doesn't hold:** each of the strip's threads finds its ray's start before its first step (a
  dozen reads) and puts its stack by after its last: 30,256 threads high over the spawn, 20,969 of which march.
  That work is in the 31.5% and not in the 15.5%.
- **What it says of the mend's ceiling, before the mend is built** (and a correction to Builder's own word to
  David, who was told it might gain at the working marks too): taking the long rays out of the layers frees little
  there: the layers' held falls by 4.1% high over the spawn and 3.5% at the mountain top. A long ray's neighbours
  in its group are long too, so it holds them for less than Builder supposed. If the strip's lengths were packed
  with no waiting at all, the strip would hold about what it takes (297,049 and 226,852: 4.0% and 3.0% of the
  frame's). So **the best the mend can do where the card works is about level with uncut**: 4.1% freed, 4.0%
  spent, high over the spawn; 3.5% and 3.0% at the mountain top; less the threads' work before a first step.
  Where the card waits it keeps the gain (the sea, the dearest mark uncut at 1.083 ms, flew 0.739 cut).

### Build 184 flown (2026-11-15, 13:29 to 13:42, hands off): the reading stands. Four fifths of the strip's thread-steps are waiting

David: "go ahead with the two flights" (he'd asked what they were for: the still count confirmed in flight, and
that nothing else moved; offered folded into build 185's session instead, he chose now). Told before take-off; one
screen, on mains, the card idle before (5%); no hitch. The game as launched, the cut off
(`logs/flight-1794771340`), and with `INCUS_FAR_CUT=1024` (`logs/flight-1794771730`). Thread-steps held by groups
of 32, the layers' and the strip's together:

| mark | the march uncut, ms | cut | | held uncut | held cut | | the strip waiting | rays cut (build 183's) |
|---|---|---|---|---|---|---|---|---|
| the spawn | 0.359 | 0.371 | 3.3% more | 2,806,496 | 2,896,160 | 3.2% more | 84% | 282 (282) |
| the mountain top | 0.916 | 1.062 | 15.9% more | 7,485,664 | 8,743,680 | 16.8% more | 83% | 2,000 (1,902) |
| the bare plain | 0.765 | 0.432 | 43.5% less | 1,392,640 | 2,271,488 | 63.1% more | 81% | 1,681 (1,455) |
| high over the spawn | 0.934 | 1.301 | 39.3% more | 7,497,792 | 9,509,760 | 26.8% more | 84% | 3,669 (2,682) |
| low over the sea | 0.969 | 0.710 | 26.7% less | 2,484,000 | 3,690,464 | 48.6% more | 81% | 2,572 (1,591) |
| a player's run | 0.853 | 1.067 | 25.1% more | 3,120,000 | 6,150,496 | 97.1% more | 81% | 4,358 (4,346) |
| fast across the world | 0.623 | 0.685 | 10.0% more | 6,221,600 | 9,202,368 | 47.9% more | 88% | 4,574 (5,234) |
| the plain, nodding | 0.743 | 0.515 | 30.7% less | 1,433,920 | 2,563,264 | 78.8% more | 79% | 2,057 (1,769) |
| the sea, nodding | 0.916 | 0.861 | 6.0% less | 2,499,104 | 4,636,544 | 85.5% more | 82% | 4,031 (2,789) |

**Against the pass mark:**
- **The count is sound in flight:** at every standing mark busy is the rays' steps summed, to the step (the moving
  marks' differ by under 3%: each number is its own most over the measured frames).
- **The game as launched within 2% of build 183's uncut flight: met**, within 0.5% at every mark. The count and
  the switch cost the march nothing.
- **The reading stands.** The mountain top's held is up 16.8% where its march is 15.9% longer (the line: 9% or
  more). High over the spawn's is up 26.8% (29.8% by groups of 64) where its march is 39.3% longer (the line: 16%
  or more). The spawn the same way: 3.2% and 3.3%. Where the card works, the march lengthens about as held
  thread-steps do; high over the spawn's last third is likely the strip's threads' work before a first step
  (40,783 of them marched there), which the count doesn't hold.
- Where the card waits, held rises as much or more and the march shortens (the plain: 63% more held, 43% less
  time): threads kept waiting cost it nothing there, as supposed.
- **With the cut on, within 2% of build 183's cut flight: missed, and found.** The plain 0.432 ms for 0.396, the
  sea 0.710 for 0.609, high over the spawn 1.301 for 1.235, the nodding marks a fifth dearer. Not the count (it
  runs outside the passes, and the uncut flight is to the hair): build 184's join counts the steps of the lengths
  behind what covers a ray, and the line that keeps a cut ray listed (32 steps) reads that sum, so more rays stay
  cut (the sea 2,572 for 1,591, high over the spawn 3,669 for 2,682). Builder's reading from the counts and the
  one line changed; not flown apart.
- **A waste this shows, not named before:** a cut ray's lengths behind its cover march on (each begins letting
  through a twentieth and goes till it is covered itself, or to its end), where the whole ray stops at the cover.
  Build 183 had it too; it only kept fewer such rays listed.

**What it says for the mend:** the strip holds 640 thread-steps a cut ray high over the spawn for 100 steps
taken; packed with no waiting it would hold 368,519, 4.9% of the frame's, against 4.6% the layers free with the
long rays out (7,497,792 to 7,151,392). So the ceiling where the card works is level with uncut, as the still
count said. Three wastes for the mend to end: sixteen threads a ray whatever its length; lengths behind a ray's
cover; and rays kept cut that aren't long.

### Build 185, the cut packed, a ray given the lengths it needs: the pass mark (written before the code)

David, 2026-11-15: "yes, lets try your idea to make the cut work smarter, if it doesn't work it doesn't work."
Told since, in the open: its ceiling where the card works is level with uncut, not under; the gain is where the
card waits.

**What changes:**
- **A ray's cuts are its own.** The places a ray may be cut stay as they are (sixteen, each half a doubling of
  distance on, shifted ray to ray), but a ray is cut only at those it needs: a word of sixteen bits a listed ray.
  When a ray is first listed, marched whole, it's given as many cuts as its steps over about 80 say, spread
  evenly over the stretch it marched. After that its join re-cuts it each frame from what its lengths took: a
  length of over 120 steps gets a cut put back near the middle of the stretch it marched; two neighbours of under
  80 steps together lose the cut between them; and where the ray was covered, the cuts beyond are taken out, so
  the length that reaches the cover runs on to the ray's end and stops there as a whole ray does. So nothing
  marches behind a ray's cover, and a ray that has grown short becomes one length again, its own thread's, at no
  thread of the strip's.
- **The strip is packed.** A list of lengths beside the list of rays: a join gives its ray's lengths places in
  it, one after another, and the strip's threads are those places, row after row from its top. A group of the
  strip then holds 32 lengths of 40 to 120 steps each.
- **A cut ray stays listed down to half the line** (64 steps, by its lengths' steps, which are now all before its
  cover): listed with no cuts it costs nothing in the strip.
- The first cut's distance (`INCUS_FAR_CUT`) is 384 blocks as flown, so the sixteen places reach from there to
  70 km and the stretch near the eye can be cut too (a cut ray's near length took up to 147 steps).
- The cut stays off unless asked, as build 184.

**The marks:**
- **Off the screen, the picture cut against uncut**, five marks at 2,560 x 1,600, the tenth frame (a ray's cuts
  settle over a few): as build 183's line. Within 8 of 255 but for rays of the map's-edge flaw (build 183's
  entry); no ring, no seam.
- **Off the screen, the count** (sound first: busy is the rays' steps, to the step): at the mountain top and
  high over the spawn the strip's waiting share is 40% or under (it's 80%), and held thread-steps, cut over
  uncut, are within 3% (they're 13 and 16% more off the screen). The longest chain 140 or under at all five
  marks.
- **Flown, one session, uncut then the cut on:**
  - where the card works (the mountain top, high over the spawn, the spawn): the march within 3% of uncut;
  - the dearest mark with the cut on no dearer than the dearest uncut (the far eye's whole: 1.08 ms, the sea);
  - where the card waits: the plain's march 0.40 ms or under, the sea's 0.62 or under (build 183's cut: 0.396
    and 0.609; build 184's: 0.432 and 0.710);
  - the run and fast across the world: within 3% of uncut or under (rays newly long still march whole).
- **Kept or dropped** (Builder's word to David): kept only if the first two flown lines are met. Else the cut is
  dropped, its code taken out, and the far eye stays as build 182 marches it. The other lines missed alone: said,
  and David's call.

### Build 185 built (unflown), the cut packed: what was built and what was checked off the screen

**Built as the mark says.** A listed ray carries its own cuts (sixteen bits, one for each place it may be cut at);
a ray first listed is given as many as its steps over 80 say, spread over the stretch it marched; its join re-cuts
it each frame (a length over 120 steps: a cut put back near the middle of its stretch; two neighbours under 80
together: the cut between them out; where the ray is covered: the cuts beyond out). A list of lengths beside the
list of rays gives each length beyond a ray's first a place, a ray's lengths side by side, and the strip's threads
are those places (16,384 of them; the strip is 24 threads across at 2,560 x 1,600, where it was 88). The first
place a ray may be cut at is 384 blocks out. A cut ray stays listed down to 64 steps. No ray is listed while the
cut is off.

**One mistake of mine on the way, caught before it was committed:** the rewrite called the march from two places
(a pixel's thread, a length's thread), which put the whole march in the program twice: the far eye's test took 82
seconds for 27, and the game's start would have been as much slower. Called from one place again (build 183's
rule: the one march in the program once): 29 seconds.

**Off the screen, 2,560 x 1,600, the tenth frame:**

| mark | rays listed | lengths in the strip (build 183: sixteen a ray) | the strip waiting (it was 80%) | held, cut over uncut (it was) | the longest chain | the picture cut against uncut, the most of 255 |
|---|---|---|---|---|---|---|
| the spawn | 279 | 279 (4,464) | 24% | 0.1% more (3% more) | 127 | 6 |
| the mountain top | 1,583 | 2,259 (25,328) | 30% | 0.1% more (13% more) | 127 | 8 |
| the bare plain | 890 | 1,238 (14,240) | 36% | level (34% more) | 127 | 7 |
| low over the sea | 756 | 1,496 (12,096) | 42% | 4% less | 127 | 6 |
| high over the spawn | 1,896 | 2,737 (30,336) | 36% | 0.8% more (15% more) | 127 | 7 |

- **The picture:** within 8 of 255 at every mark, no ray over, no ring, no seam: met. (The map's-edge rays that
  were 70 and 17 at build 183 are within 8 here: fewer lengths begin in the stretch a whole ray's last stride goes
  past. The flaw itself is as it was.)
- **The count:** sound (busy is the rays' steps, to the step, checked by the test). The strip's waiting share at
  the mountain top 30% and high over the spawn 36% (the line: 40% or under): met. Held thread-steps, cut over
  uncut, 0.1% and 0.8% more (the line: within 3%): met.
- **The longest chain** 127 at all five (the line: 140 or under): met. It's the rays just under the line of 128
  steps, marched whole; a listed ray's longest length is 120.
- **A ray needs few lengths:** most one or two beyond its first, none more than five (the sea).
- **The uncut picture is build 182's** (640 x 400: no pixel at three marks, 1 of 255 at two). All 31 tests pass.

**Known and left:** a length can only be cut at one of its ray's sixteen places, so a stretch nearer than 384
blocks, or one that lies inside a single place's span, can't be cut shorter; and a ray whose cover goes away runs
its last length to the ray's end for a frame or two before its join has cut it again (the frame's notice for
newly long rays, the cut's third stage, is the same matter).

**The flights asked for, one session, hands off:** the game as launched (the cut off), then `INCUS_FAR_CUT=384`.

### Build 185 flown (2026-11-15, 14:38 to 14:51, hands off): the packed cut is level where the card works and a third to a half faster where it waits; one keep-or-drop line missed by a hair, and the build itself 4.5% dearer at the two working marks, cut or not

David: "go". Told before take-off; one screen, on mains, the card idle before (7%); no hitch. The game as launched,
the cut off (`logs/flight-1794775526`), then `INCUS_FAR_CUT=384` (`logs/flight-1794775912`).

| mark | the march uncut, ms | cut | | the far eye's whole uncut | cut | | held, cut over uncut | the strip waiting | the longest chain |
|---|---|---|---|---|---|---|---|---|---|
| the spawn | 0.358 | 0.368 | 2.8% more | 0.467 | 0.486 | 4.1% more | 0.1% more | 24% | 153 to 127 |
| the mountain top | 0.958 | 0.942 | 1.7% less | 1.082 | 1.076 | 0.6% less | level | 28% | 302 to 129 |
| the bare plain | 0.766 | 0.451 | 41.1% less | 0.877 | 0.572 | 34.8% less | 1.1% more | 40% | 324 to 137 |
| high over the spawn | 0.975 | 0.986 | 1.1% more | 1.096 | 1.117 | 1.9% more | 0.8% more | 36% | 261 to 127 |
| low over the sea | 0.973 | 0.530 | 45.5% less | 1.086 | 0.655 | 39.7% less | 3.9% less | 37% | 409 to 132 |
| a player's run | 0.857 | 0.597 | 30.3% less | 0.975 | 0.729 | 25.2% less | 3.5% more | 35% | 339 to 217 |
| fast across the world | 0.640 | 0.617 | 3.6% less | 0.768 | 0.750 | 2.3% less | 5.8% more | 40% | 228 to 177 |
| the plain, nodding | 0.743 | 0.492 | 33.8% less | 0.854 | 0.613 | 28.2% less | 5.6% more | 40% | 347 to 240 |
| the sea, nodding | 0.918 | 0.563 | 38.7% less | 1.033 | 0.690 | 33.2% less | 1.7% less | 48% | 354 to 248 |

**Against the pass mark:**
- **Where the card works, the march within 3% of uncut: met** (the mountain top 1.7% less, high over the spawn
  1.1% more, the spawn 2.8% more). Build 183's cut was 16 to 39% more there. Held thread-steps are level, as off
  the screen, and the march with them: the reading holds to the end.
- **The dearest mark with the cut on no dearer than the dearest uncut (1.08 ms): missed, narrowly.** With the cut
  on the dearest is high over the spawn at 1.117 ms; this session's own uncut dearest is the same mark at 1.096
  (1.9% under it), and the line's 1.08 was the sea's on build 183's day.
- The sea's march 0.62 or under: met, 0.530. The plain's 0.40 or under: **missed**, 0.451 (build 183's cut flew
  0.396). The run and fast across within 3% of uncut or under: met (30.3% and 3.6% less).
- **By the rule as written** (kept only if the first two lines are met): the second is missed, so by its letter
  the cut is dropped. Put to David as it is, with what follows.

**What wasn't expected: the build itself is dearer at the two working marks, cut or not.** Its uncut flight
against build 184's uncut flight of an hour before: the mountain top 0.958 ms for 0.916 (4.6% more), high over the
spawn 0.975 for 0.934 (4.4% more), fast across 0.640 for 0.623 (2.7% more); every other mark within 0.5%. Only the
marks that work moved. That is how a march needing more registers shows (fewer groups of rays fit; the marks that
wait don't feel it: builds 177 to 180), and not how a slow session shows (every mark would move). Builder's
reading, not seen in a trace: build 185's march carries more than build 184's (a ray's first cuts are worked out
at the end of the march's own program; where a length began is kept through the march for its stack).
- If that 4.5% is the program's, it sits under both columns above: with it gone, high over the spawn with the cut
  on would be about 1.07 ms and the mountain top 1.03, and the second line would be met.
- And it sits on the game as launched, so it's to be found and mended whether the cut is kept or dropped (dropped,
  the cut's code goes and the march is build 182's again).
- For scale, the same two marks over the day's uncut flights: the mountain top 0.934, 0.927, 0.919, 0.916, then
  0.958; high over the spawn 0.975, 0.980, 0.939, 0.934, then 0.975 (builds 180, 182, 183, 184, 185). Builds 183
  and 184 were the cheapest there, which may be the same matter the other way (their march carried less).

**Against build 182, the far eye before the cut** (the far eye's whole, ms): the spawn 0.466 to 0.486, the
mountain top 1.046 to 1.076, the plain 0.886 to 0.572, high over the spawn 1.099 to 1.117, the sea 1.098 to
0.655, the run 0.981 to 0.729, fast across 0.733 to 0.750, the plain nodding 0.862 to 0.613, the sea nodding
1.040 to 0.690. Five views a quarter to two fifths faster; four views 2 to 4% slower, the dearest among them.
At the 99th the worst mark uncut is the sea nodding, 1.308 ms; with the cut on it's fast across, 1.274.

**Moving, the longest chain stays long** (the run 217, the nodding marks 240 and 248): rays newly long march
whole for a frame, and a ray is cut finer one cut a frame. The march is a quarter to two fifths shorter there all
the same. That is the cut's third stage, if it's kept.

### Build 186, the march's program trimmed of what the cut put in it: the pass mark (written before the code)

David, 2026-11-15, on build 185's flight: "it looks like the new cut does bring the overall average down, and
doesn't add much to the high over spawn. half as many figures over 1. why don't you look over everything in the
shader that can be trimmed and see how that works. and then we can move on, if it gets a little better overall
than it is now. but if the thing ever gets too big this would be something to look at. not sure exactly where we
are with that, but i'd like to get a move on." So the cut is kept; his bar, "a little better overall than it is
now", stands in place of Builder's keep-or-drop line; and the march's program size is a thing to look at if it
grows. *(Where that is: last measured at build 178, the march needs 167 registers and 12 groups of rays of 48
fit. Not measured since the cut went in; it's read in Nsight's own window.)*

**What's trimmed** (gone over: everything builds 183 to 185 put in the march's own program; the parts the march
had before were priced by the audits of builds 177 to 180 and left):
- **A ray's first cuts aren't worked out at the end of its march.** A ray listed whole is listed uncut; the frame
  after, its join (which already re-cuts every listed ray by what its lengths took) gives it its first cut, and
  the frame after that the next. So the march's program loses the sums for the first cuts, their writes and
  their lock on the count of lengths; and a newly long ray is cut a frame or two later than in build 185.
- **Nothing of the cut is kept through the march but where a length's stack is.** Where a length began and what
  it began letting through are written to its stack before it marches, not carried to the end of it. (A thing a
  march holds from before its steps to after them takes room for all its steps.)
- What stays in the march's program, looked at and left: a pixel's one read of whether its ray is listed (the
  spawn's 2.8% with the cut on is likely this); the place to stop, which every ray's steps test (flown free,
  build 183 uncut); a long ray listing itself (flown free, build 182).

**The marks:**
- **Off the screen:** the uncut picture build 182's; the picture cut against uncut within 8 of 255 at the
  mountain top and the sea (the twelfth frame: a ray's cuts now settle a frame or two later); the count as
  build 185's (the strip's waiting share 40% or under and held within 3% of uncut at the mountain top and high
  over the spawn; the longest chain 140 or under).
- **Flown, one session, the game as launched and the cut on:**
  - **the reading of the 4.5%:** uncut, the mountain top's march within 2% of build 184's 0.916 ms and high over
    the spawn's within 2% of 0.934. If they stay at build 185's 0.958 and 0.975, the reading (the program's
    weight) is wrong, said so, and Nsight's window is where to look next;
  - **David's bar:** the far eye's whole with the cut on, summed over the nine views, under build 185's 6.69 ms
    (the spawn 0.486, the mountain top 1.076, the plain 0.572, high over the spawn 1.117, the sea 0.655, the
    run 0.729, fast across 0.750, the plain nodding 0.613, the sea nodding 0.690);
  - the dearest mark with the cut on at or under build 185's uncut dearest, 1.096 ms.
- **Then:** his bar met, the cut is on as the game is launched (`INCUS_FAR_CUT=0` turns it off), and we move on.
  Not met: the cut stays in, off unless asked, as build 185, and we move on all the same (his word). Still owed
  either way: his look at the horizon while turning his head, when he next plays (a length begins by the frame
  before).

### Build 186 built (unflown), the march's program trimmed: what was built and what was checked off the screen

**Built as the mark says:** a ray listed whole gets no cuts at the end of its march (the sums for its first cuts,
their writes and their lock on the count of lengths are out of the march's program); its join gives it its first
cut the frame after, the next the frame after that. Where a length begins and what it begins letting through go
to its stack before it marches; through the march the cut carries only where the stack is.

**A mistake of mine the check found, mended before this was written** (it was in build 185's join too, and
showed once the join made every first cut): when a join put a cut back inside a length, it guessed how the ray
stood there, evenly between the length's ends. Where a ray's cover is all in one place the guess was far out
(a ray uncovered till 3.9 km was taken as nine tenths covered at 3.2), the new length began thinking the ray
nearly covered and stopped early, the two lengths then came to under 80 steps and were joined again, and the ray
went from cut to whole and back every other frame: two 4 x 4 rays high over the spawn 33 of 255 lighter when
cut, one at the mountain top 15. Now no guess is made: a length that begins at a cut just made takes up letting
through all and looks before it skips, as a ray's lengths did on its first cut frame in build 185, and the join
scales what it gathered by how the ray stands there in truth. The twelfth and thirteenth frames are then the same
to the step.

**Off the screen, 2,560 x 1,600, the twelfth frame:**

| mark | rays listed | lengths in the strip | the strip waiting | held, cut over uncut | a listed ray's longest length | the picture cut against uncut, the most of 255 |
|---|---|---|---|---|---|---|
| the mountain top | 1,581 | 2,171 | 28% | level | 120 | 5 |
| low over the sea | 754 | 1,252 | 36% | 5% less | 120 | 6 |
| high over the spawn | 1,889 | 2,255 | 32% | 0.2% more | 120 | 7 |

- The marks off the screen: met (within 8 of 255; the strip's waiting share 40% or under; held within 3%; the
  longest chain 127, the rays just under the line). The uncut picture is build 182's (no pixel at three marks, 1
  of 255 at two). All 31 tests pass; the far eye's test takes what it took.

**The flights asked for, one session, hands off:** the game as launched (the cut off), then `INCUS_FAR_CUT=384`.

### Build 186 flown (2026-11-15, 15:26 to 15:39, hands off): David's bar met; the 4.5% was the program's weight. The cut is on as the game is launched (build 187), and the cut is done enough to go on

David: "go ahead with the flights". (He asked first whether, if it missed, it would only be staying in because
lazy rays are a different form of it. Builder: yes, its lengths, their stacks and the join are what lazy rays
would stand on, and that's Builder's view, not the plan's.) Told before take-off; one screen, on mains, the card
idle before (5%); no hitch. The game as launched, the cut off (`logs/flight-1794778376`), then `INCUS_FAR_CUT=384`
(`logs/flight-1794778772`).

| mark | the march uncut, ms | cut | | the far eye's whole uncut | cut | | at the 99th, cut | the longest chain, cut |
|---|---|---|---|---|---|---|---|---|
| the spawn | 0.353 | 0.363 | 2.8% more | 0.462 | 0.479 | 3.7% more | 0.488 | 127 |
| the mountain top | 0.917 | 0.914 | 0.3% less | 1.038 | 1.045 | 0.7% more | 1.086 | 127 |
| the bare plain | 0.762 | 0.450 | 40.9% less | 0.872 | 0.568 | 34.9% less | 0.600 | 127 |
| high over the spawn | 0.933 | 0.958 | 2.7% more | 1.052 | 1.086 | 3.2% more | 1.112 | 127 |
| low over the sea | 0.958 | 0.528 | 44.9% less | 1.071 | 0.651 | 39.2% less | 0.687 | 127 |
| a player's run | 0.845 | 0.591 | 30.1% less | 0.962 | 0.721 | 25.1% less | 0.817 | 165 |
| fast across the world | 0.609 | 0.616 | 1.1% more | 0.732 | 0.754 | 3.0% more | 1.245 | 165 |
| the plain, nodding | 0.735 | 0.473 | 35.6% less | 0.846 | 0.592 | 30.0% less | 0.764 | 247 |
| the sea, nodding | 0.909 | 0.561 | 38.3% less | 1.024 | 0.686 | 33.0% less | 0.899 | 238 |

**Against the pass mark:**
- **The reading of the 4.5% stands.** Uncut, the mountain top's march is 0.917 ms (build 184: 0.916; build 185:
  0.958) and high over the spawn's 0.933 (0.934; 0.975): both within 2% of build 184's, to the hair. What build
  185 kept through the march and worked out at its end was the cost, and it showed only where the card works.
  So the march's program is as near its limit as that: a few numbers more held through its steps cost 4.5% there.
- **David's bar, "a little better overall than it is now": met.** The far eye's whole with the cut on, summed
  over the nine views: 6.582 ms, for build 185's 6.688 (uncut, this flight: 8.059).
- **The dearest mark with the cut on at or under 1.096 ms: met**, 1.086 (high over the spawn). The same
  flight's uncut dearest is the sea at 1.071, so the dearest is 1.4% up; the worst 99th is down, 1.290 to 1.245.
  Views over 1 ms: four uncut, two with the cut on.
- Where the card works the cut costs 1 to 4% (the spawn 3.7%, high over the spawn 3.2%, fast across 3.0%, the
  mountain top 0.7%): held thread-steps are level, so this is what the count doesn't hold: a pixel's read of
  whether its ray is listed, and the strip's threads' work before a first step.
- Moving, the longest chain stays long (165 on the run, 240 nodding): a newly long ray marches whole for two
  frames now, and is cut finer one cut a frame. The march is a quarter to two fifths shorter there all the same.

**So, by the mark: the cut is on as the game is launched** (build 187: `INCUS_FAR_CUT` unset means the first
place at 384 blocks; 0 turns the cut off; nothing else changed from the build flown, so build 187 as launched is
build 186's second flight). **The cut is done enough to go on** (David: "and then we can move on, if it gets a
little better overall than it is now").

**The cut from first to last, for the record** (the far eye's whole, ms, build 182 to build 186 with the cut on):
the spawn 0.466 to 0.479; the mountain top 1.046 to 1.045; the plain 0.886 to 0.568; high over the spawn 1.099 to
1.086; the sea 1.098 to 0.651; the run 0.981 to 0.721; fast across 0.733 to 0.754; the plain nodding 0.862 to
0.592; the sea nodding 1.040 to 0.686. Its lessons: the list costs nothing if only long rays touch the buffer
many write (181, 182); where the card works, threads kept waiting are the cost, not steps (183, 184); a ray wants
only the lengths it needs, packed (185); and the march's program has no room to carry more (185, 186).

**Still owed, written down:**
- David's look at the horizon while turning his head (a length begins as its ray stood the frame before).
- The cut's third stage, if wanted: the frame's notice for newly long rays (the moving marks' long chains).
- The march's program size, by Nsight's window, if it grows (David: "if the thing ever gets too big this would
  be something to look at"): last read at build 178, 167 registers and 12 groups of rays of 48.
- The flaw at the map's edge (a ray's last stride past its end, unlooked); the far eye's test not the same run
  to run at full size (up to 6 of 255 at a few rays); the main memory test that failed once.

### Build 188, the scatter stage behind the doorway, and the far cells summed from columns that aren't plain ground: the pass mark (written before the code)

David's look at build 187, the cut on (2026-11-15): "Builder, all of us are doing something wonderful together. it
is alarmingly fast and looks really nice. i think we're going to have room for people with stronger systems to bump
up the resolution of the march and stuff. lets keep going. whats next?" (No flicker at the horizon named: taken as
his look at the cut given.) Asked where the wood should go and what its trees are made of: "go ahead and take the
oak.png if you want, it won't matter much; you did a good job picking blocks for pretend last time. i think the
things we'd want to be checking are large arched entranches to mountain halls, castles with walls and towers,
towers in the mountain and on the ground, towns on the ground, forests here and there, random cave mouths here and
there. its gonna be blurry and just for testing anyways, so go nuts, do whatever you want, have fun." So the
places, blocks and colours are Builder's to choose; "each placed with David" is lifted for the test detail.

**What was found first:** the far cells are summed from a column's ground, water and profile alone: a height and
what lies under it. Nothing the far eye draws can stand over air: no overhang, arch or cave; and what's made as
changes through the doorway (David's build, the scatter's blocks) never reaches the far cells at all (the
scatter's marks fly at the bare plain's times, to the hair). So the stage needs two things, in this order.

**What's built:**
1. **The far cells summed from any column:** a column's solid blocks as runs with air between; every face
   counted against the runs beside it (a solid block's face shows where the block beside it isn't solid, as old's
   rule has it), under an overhang too. Plain ground keeps its own quick road, untouched: only a column the stage
   changed, or one beside such a column, goes the new one.
2. **The scatter stage** (the Oracle's third part, section 7), applied by the doorway as it answers a tile, from a
   seed and a short list of places; off unless `INCUS_SCATTER_STAGE=1` at launch; an instrument, not world
   generation. What it makes, by David's list and the Oracle's: woods in patches (trunks and crowns, 4 to 20
   high), within ten kilometres of the spawn; towers on the plain and in the mountains; a castle with walls,
   corner towers, a keep and an arched gate; a town; an arched entrance to a hall cut into a mountainside; cave
   mouths on steep slopes; rows of posts, a slab in the air and a free-standing arch. New blocks, their far
   colours Builder's picks but the oak's, which is his `oak.png` shrunk to one pixel as old's rule shrinks a
   texture (32, 14, 6 of 1,023).
3. **Marks from it** in `data/marks.toml`, one for each kind of thing.

**The marks:**
- **Off, nothing changes:** the far cells are old's bake, byte for byte (the test that says so); the five
  marks' pictures are build 187's.
- **On, the summing is exact,** three ways: (a) over a square that holds a wood, a tower with an overhang, an
  arch and a carved cave, the cells counted the slow way (every block, every face, one at a time) are the cells
  the summer makes, to the number; (b) every level is its children added (a 16 its eight 8s, a 32 its eight 16s);
  (c) plain ground sent down the new road gives the quick road's cells, to the number.
- **What it costs:** the whole world summed with the stage on in under twice its time off (17 s); the far
  cells' memory on the card within 15% of off.
- **Pictures off the screen** at the new marks, looked at by Builder first (does each thing read as itself, at
  the size the far cells allow), then David's look in the game.
- **Flights with the stage on:** for information (what detail costs the far eye), once his look is given; no
  line set.

### Build 188 built (unflown), the scatter stage: what was built and what was checked off the screen

**Built as the mark says.**
- **The far cells summed from any column** (`far.rs`, `add_column_runs`): a column's solid blocks as runs with
  air between; a face wherever the block beside, over or under it isn't solid. A column the stage changed, or
  one beside such a column, goes this road; plain ground keeps its own quick one, untouched.
- **The scatter stage** (`scatter_stage.rs`), applied by the doorway as it answers a tile; on with
  `INCUS_SCATTER_STAGE=1`. What stands where (its places chosen once for a world, each floor read from the
  export's own ground):
  - *woods* in patches a few hundred blocks across, within 10 km of the spawn: a tree to a square of 7, trunks 4
    to 20 high under a ball of leaves, on dry land under 700;
  - *on the plain, ahead of the plain's mark* (it looks south): three rows of posts one block wide, a
    free-standing arch, a slab in the air, a tower of 70, a castle (walls, corner towers with roofs, a keep, an
    arched gate through a gatehouse), a town (houses under pitched roofs, a church with a spire) and a tower of
    140;
  - *by the spawn:* a town on the flattest ground ahead of it, and a tower of 90;
  - *in the mountains south of the mountain top's mark:* three towers on the highest ground, well apart, and an
    arched entrance through a stone portal into a hall cut into the steepest hillside (a passage 20 wide and 30
    high, then a hall 40 wide and 50 high with two rows of pillars left standing);
  - *cave mouths* where the ground rises steeply, about sixty within 4 km of the spawn.
- **Its blocks' far colours** (`data/far colours.toml`, kept last so the others keep their numbers): the oak's
  from David's `oak.png` (32, 14, 6 of 1,023), the rest Builder's picks.
- **Its marks** are made by the stage itself and follow its places (not kept in `data/marks.toml`; they exist only
  while it's on): the castle, the two towns, the hall in the mountain, over a wood, a cave mouth.

**Off, nothing changes:** all 34 tests pass, old's bake byte for byte among them; the pictures at the plain, high
over the spawn and the mountain top are build 182's (no pixel, 1 of 255, 1 of 255).

**On, the summing is exact**, three ways (three new tests):
- over eight squares (a tower, the arch, the slab, the castle's gate, a church, the hall, a cave, a wood), 8.1
  million blocks and 89,495 faces counted one at a time from the doorway's own answers give the summer's cells,
  to the number (hundreds of those cells hold faces looking down, which plain ground never has);
- every level is its children added: 73,728 cells, each exactly its eight;
- plain ground sent down the new road gives the quick road's cells: 5,376 cell columns, to the number.

**What it costs:** the picture test with the whole world summed takes 80 s with the stage on for 74 s off (the
line: under twice). The far cells' memory on the card: owed, read at the first launch with it on.

**Mistakes of mine on the way, seen in the pictures and mended:** the stage's blocks had no far colours at first
(everything built came out mid grey, the woods a grey speckle); the town by the spawn stood on a steep slope with
one floor for all its houses, which made pillars of them (each house now stands on its own ground, and the town
is laid where the ground is flattest); three mountain towers stood side by side; a tower stood on a peak too high
to hold it; the caves were all in one corner.

**Builder's look at the pictures (1,280 x 800):** from high over the spawn the slopes are wooded, a town by the
lake; the castle reads as grey walls with red-roofed towers and a keep, a wood behind it; the hall's entrance as a
pale frame round a dark opening on the mountainside, a tower on the peak over it; the town on the plain as pale
houses with the great tower behind. Thin things nearly vanish against the sky at the far cells' size, as expected:
the arch's piers (4 blocks square in cells of 8) are a faint ring, the posts a pale band on the ground, and a cave
mouth mostly doesn't read from the front, lit inside as outside (the far light is a height a column, so nothing
is dark under a roof): the things this detail is here to judge.

### Build 188, David's look at the scatter stage (2026-11-15), and the "square light seams" found to be shallow sea over a flat shelf

Launched on his word with the stage on, at the castle; 72 s, closed cleanly, no error in its log. He flew over the
plain's town, tower and castle and sent a picture from high over them. **His words:** "very nice builder. it looks
like it should, i think. but can we do anything about these square light seams?" (in the picture: a broad
grey-olive region of the plain beyond the tower, its edges long straight lines, faint lines nested inside it).

**What the launch says of the pass mark's costs:** the whole world summed in 42.0 s with the stage on (36.6 s off,
his look at build 187): under twice, met. The far cells on the card 36,524 bricks and 165 MB (36,254 and 163 MB
off): within 15%, met. The 16s' ring's first making 2.5 s for 1.35 (its tiles hold woods).

**The seams, looked into before answering:** not light, and not the stage's.
- The same view drawn off the screen three ways (the stage on; on with the sun's shadows off; the stage off) shows
  the same grey region with the same straight edges each time.
- The export's own columns there: sand (profile 1) with its ground at 126, two blocks under the sea's level of
  128, no river or lake; the yellow plain beside it is the same sand at 128, dry. So the grey is two blocks of sea
  over sand, dimmed as the far eye dims what's seen through water.
- Why straight lines: the shelf is nearly a plane, and heights are whole blocks, so the shore (where the ground
  drops under 128) and each block of depth after it run as long straight contours; the faint nested lines are the
  next blocks of depth.
- So it's the test world's own shape and the far eye's water, there before the stage and before the cut. What
  could be done is a look of water's (a shallow edge that fades in, not a step at the first block) and belongs
  with water's own design step; put to David as his call.

### Build 189, the bright bands at the rings' edges over shallow water: Builder's first answer wrong, the cause, and the pass mark (written before the code)

**Builder's answer to David was wrong about what he meant.** The grey region is shallow sea, as said; but his
"square light seams" are the faint brighter lines inside it. David: "definitely not the shores. its two square
rings that move with me, it looks fine on either direction of the seams but there's bright square seams; if you
look in that sea bed for two bands with a right angle turn that are more saturated yellow, you'll see it; it
moves with you as you fly." They are the edges of the 8s' and the 16s' rings round the eye. (His look of build
167, "i can't see any rings", was of the levels' hand-over by distance, before these rings were built.)

**Seen off the screen:** from 300 over the shallow shelf, looking down: a square outline a little more yellow
than the sea bed either side of it (`rings-before.png`, kept with the session's pictures).

**The cause, as Builder reads the march** (to be shown by the mend): where a level hands over to the next, the
step's look is the two levels' looks mixed, and the mixed solid share is gathered as a share of the finer level's
cell (`1 - (1 - share) ^ (length / size)`, `size` the finer cell's). A coarser cell's share is of a cell twice the
size: counted against the finer cell it stops a ray in half the distance it should. So through a hand-over the
ground is too dense from too high, a ray is stopped before it has crossed all the water over the bed, the bed is
dimmed less, and it shows brighter: most at the hand-over's outer end, where the look is all the coarser level's,
and gone the step after, when the steps are counted in the coarser cell. Over dry ground the same is there and
can't be seen. It is the same at the hand-overs by distance, which are rings round the eye too.

**The mend:** a look read from a coarser level than the step is counted in has its solid share turned into the
share of the finer cell that stops a ray as much (`1 - (1 - share) ^ (finer cell / coarser cell)`: for the next
level, a square root), before the two are mixed. Water needs none: it's gathered by its share and the distance
alone.

**The marks:**
- **Off the screen, the same view over the shelf:** the square outline gone (Builder's look, the two pictures
  side by side); then David's look in the game, flying over the shallow sea.
- **The five marks' pictures** against build 188's: changed only through hand-overs, and by how much is
  reported (no line: every hand-over's surface gets a little thinner, as it should have been).
- **Flown** (the mend is a power in the hand-over's steps only): each mark's march within 2% of build 186's cut
  flight; over that at a working mark, said, and a cheaper form looked for.

### Build 189 built (unflown): the rings' bright bands mended, and the reading shown

**Built as the mark says:** a look read from a coarser level has its solid share turned into the finer cell's
(`by_finer_cell`, `far.wgsl`) before the hand-over mixes the two. One place in the march, in the hand-over's steps
only.

**Off the screen, from 300 over the shallow shelf, looking down:** before, a square outline a little more yellow
than the sea bed (the 8s' ring's edge); after, none: the bed is even across where the outline was. So the reading
holds: the bands were the coarser level's ground counted against the finer cell.

**The five marks' pictures against build 182's** (640 x 400, the cut off): they change through the hand-overs, as
expected of a mend to every hand-over, by distance as well as at the rings.

| mark | pixels that differ at all | by 4 of 255 or more | by 16 or more | the most |
|---|---|---|---|---|
| the spawn | 2.6% | 0.6% | 0.06% | 34 |
| the mountain top | 10.6% | 1.3% | 0.03% | 19 |
| the bare plain | 2.8% | 0.4% | none | 15 |
| low over the sea | 2.7% | 0.7% | 0.06% | 24 |
| high over the spawn | 17.7% | 3.7% | 0.17% | 45 |

Where: in bands across the land where a level hands over (a faint speckle, mostly under 4 of 255), over water,
and at a few skyline pixels inside a hand-over, where ground that stood too dense now lets a little more sky
through. So this is a change to the look everywhere a level hands over, small but real: David's eye on it, on
land as well as over the sea.

All 34 tests pass. (The main memory's test failed once more on the first run and passed on the second: the flaw
written down at build 182, still to mend.)

**Owed:** David's look; then the flight for the mark's line (each mark's march within 2% of build 186's cut
flight).

### Build 189, David's look (2026-11-15): the rings' bands gone enough; done enough to go on

Launched on his word with the stage on, by the town on the plain; 88 s, closed cleanly, no error in its log.
**His words:** "after everything loaded in, it is basically invisible, and i don't think anyone will ever have the
chance to pick it out amongst a finished world even if he got high up on a mountain. well done. let us proceed."
Nothing said against the look on land, where every hand-over changed a little. So the mend is kept, and the
scatter stage with it (his look at build 188: "very nice builder. it looks like it should, i think").

**Still owed for builds 188 and 189:** one flight session: the game as launched (the mark's line: each mark's
march within 2% of build 186's cut flight), and once with the stage on, for what detail costs the far eye.

### Builds 188 and 189 flown (2026-11-15, 17:11 to 17:27, hands off): seven marks of nine hold the line; the plain misses by 11.6% and the sea nodding by 3.9%, and the cause found

Two flights in one session, on David's word ("you can go"): the game as launched (`logs/flight-1794784679.txt`: the
cut on, the stage off) and once with the scatter stage on (`logs/flight-1794785233.txt`). Both reports read "the
code committed" (60bcd7e). No hitch.

**The mark's line: each mark's march within 2% of build 186's cut flight** (`logs/flight-1794778772.txt`). The
march (the far 8 x 8 on the card, ms, the median):

| mark | build 186 | build 189 | | the longest chain, 186 to 189 |
|---|---|---|---|---|
| on the ground at the spawn | 0.363 | 0.363 | the same | 127, 127 |
| the mountain top | 0.914 | 0.913 | the same | 127, 127 |
| the bare plain | 0.450 | 0.502 | **11.6% more: a miss** | 127, **205** |
| high over the spawn | 0.958 | 0.963 | 0.5% more | 127, 127 |
| low over the sea | 0.528 | 0.537 | 1.7% more | 127, 153 |
| a player's run over the plain | 0.591 | 0.583 | 1.4% less | 165, 183 |
| fast across the world | 0.616 | 0.622 | 1.0% more | 165, 181 |
| the plain, nodding | 0.473 | 0.478 | 1.1% more | 247, 280 |
| the sea, nodding | 0.561 | 0.583 | **3.9% more: a miss** | 238, 309 |

The card's whole over the nine views: 6.663 ms for 6.582. Two views over 1 ms, as before; the dearest still high
over the spawn, 1.091 ms (1.086). The two marks where the card works hold to the hair, so the hand-over's mend put
no weight on the march's program. Seven of nine within the line; two over it, said here.

**The plain's miss, found off the screen** (the plain's view, 2,560 x 1,600, frames 12, 13 and 14): one ray, the
8 x 8's pixel (250, 99). Its lengths are [56, 205] steps one frame, [56, 103, and a third of 105 marched behind
its cover] the next, and back again, for ever. What its numbers say:

- Where its first length ends (601 blocks out) the ray is nearly covered: 1.3% still let through. Then it skims
  on over the plain, meeting almost nothing, about a hundred steps more before the last of it is covered (at
  about 3,260 blocks). So the whole ray is long in truth: some 160 steps.
- A length that takes up at a cut never begins from under a twentieth (build 183's guard: so that a length the
  ray reached nearly covered still gathers, if the ray stands more open this frame than the last). So the second
  length begins as if 5% were let through, and marches until *that* is under a hundredth: 205 steps, out to 24,000
  blocks, seven times as far as the ray goes in truth.
- The join cuts the 205 in two. The frame after, it finds the ray covered at the new cut (which is true), takes
  the cut away, and the length is 205 again.

The plain is a mark where the card waits on its longest thread: one thread of 205 steps where the longest was 127
holds the frame, 0.502 for 0.450. The steps summed are the same (1,183,054 for 1,180,651).

**Why build 189 and not 186:** the hand-over's mend lets a little more through at every hand-over, so more rays
come to a cut nearly covered and not quite. The longest chains at the sea and at the moving marks are up the
same way (153 for 127; 309 for 238 at the sea nodding, the other miss). The flaw was in the cut from build 183,
waiting for such a ray. Mended next (build 190, its mark below).

**The picture test's own count was wrong for such a frame:** it held that the threads' steps and the rays' steps
are the same number. A length marched behind the ray's cover is a thread's steps and not the ray's (build 184's
mend to the join); the test now holds that the rays' steps are all some thread's.

**With the scatter stage on** (for what detail costs the far eye; no line was set). The march and the card's
whole, ms, against the same session's flight with it off:

| mark | the march, off | on | the card's whole, on | |
|---|---|---|---|---|
| on the ground at the spawn | 0.363 | 0.307 | 0.422 | 15% less: the town and the woods stand before the long view |
| the mountain top | 0.913 | 0.948 | 1.080 | 3.8% more |
| the bare plain | 0.502 | 0.444 | 0.562 | (the flipping ray's view is changed by what's built there) |
| high over the spawn | 0.963 | 1.080 | 1.210 | 12% more: woods under it, 6.87 million steps for 6.04 |
| low over the sea | 0.537 | 0.564 | 0.688 | 5% more |
| a player's run over the plain | 0.583 | 0.812 | 0.943 | **39% more; the 99th 1.370** |
| fast across the world | 0.622 | 0.640 | 0.774 | 3% more |
| the plain, nodding | 0.478 | 0.472 | 0.590 | |
| the sea, nodding | 0.583 | 0.600 | 0.726 | 3% more |

The card's whole over the nine views: 6.995 ms for 6.663 (5% more). The dearest view is high over the spawn at
1.210 ms (99th 1.244).

- **The run is the one that suffers,** and by the list, not the steps alone: 4,686 rays there are long and
  4,317 are listed (the list holds 4,096 places), so some go uncut: the longest chain is 368 steps for 183.
  The run goes through the town on the plain and its woods at walking height, where rays skim roofs and tree
  tops. Written down, not mended: a bigger list is a number (`LONG_LIST`), to try when the world has real detail
  in it.
- **The stage's own marks** (the march; the card's whole): the castle 0.729 (0.863), the town on the plain 0.740
  (0.874), the town by the spawn 0.671 (0.795), the hall in the mountain 0.909 (1.038), over a wood 0.398
  (0.513), a cave mouth 0.158 (0.269). None is dearer than high over the spawn.
- **At launch** (from the first launch with it on): the world summed in 42.0 s for 36.6; the far cells 165 MB on
  the card for 163.

### Build 190, a length begins as the ray stood there: the pass mark (written before the code)

**What's to change** (two places; neither is in the march's loop, so the program's weight is as it was):

1. **A length that takes up at a cut begins letting through what the ray did there the frame before, down to a
   hundredth** (under that the ray is covered and the join takes the cut away anyway). The twentieth goes. A still
   ray's lengths then stop where the whole ray would, and no length marches on behind the ray's true cover.
2. **The guard the twentieth was moves to the join.** Where a length stopped as covered, and the ray stands more
   open this frame than the length took it to (the view moved), the join gives what the ray still lets through
   to the length's own colour, as if the surface that stopped it went on, and counts the ray covered there. [As it
   is, up to a fifth of the sky shows through such a ray for a frame.] In a still view this never happens: the
   join's numbers are then the lengths' own.

**The pass mark.** Off the screen:

- **The plain's ray holds still:** pixel (250, 99) has the same lengths at frames 12, 13 and 14, none over 127
  steps; the plain's longest chain is the same at the three frames and no more than 127; its steps summed no
  more than build 189's (1,182,905).
- **Still pictures:** at the five marks the cut's picture is no farther from the uncut picture than build 189's
  (pixels differing by 8 of 255 or more, and the most), and the uncut picture is build 189's to the pixel but for
  the test's own noise (nothing in the uncut march changes).
- **A moving view** (a new knob for the picture test, `FAR_TEST_TURN`: the last frame drawn with the eye turned
  so far): the first frame after a turn of 2 degrees, cut against uncut, at the plain, low over the sea and high
  over the spawn: no more pixels differ by 8 of 255 or more than build 189's measured the same way, and the most
  is no greater. Build 189's numbers are taken first.
- All the tests pass.

By a flight (on David's word):

- the bare plain's march within 2% of build 186's 0.450 ms; the sea nodding's within 2% of its 0.561;
- no other mark's march over build 189's by more than 2%.

A miss is said, with what the counts show.

### Build 190 built (unflown): a length begins as the ray stood; what was built and what was checked off the screen

**Built as the mark says,** two places, neither in the march's loop:

- `far.wgsl`, where a length is set off: it begins letting through what the ray did at its cut the frame before,
  down to a hundredth (was: never under a twentieth).
- `far_all.wgsl`, the join: where a length stopped as covered, short of where it was to stop, and the ray still
  lets through a hundredth or more by this frame's count (it stands more open than the length took it to), what
  it lets through is given to the length's own colour and the ray is counted covered there. ("Short of where it
  was to stop" is a line the mark didn't spell out: a length that reached its stop left no gap behind it, and the
  next length carries the ray on.)

A knob for the picture test, `FAR_TEST_TURN` (degrees of facing, then of pitch): the last frame is drawn with the
eye turned.

**The plain's ray holds still.** Pixel (250, 99) at frames 12, 13 and 14: lengths of 56 and 77 steps, the same
each frame (its second length begins letting through 0.0132, as the ray stands, and stops at 1,566 blocks where
it falls under a hundredth; it ran to 24,000). The plain's longest chain is 127 at the three frames (the longest
length of any cut ray 120); its steps 1,181,845 each frame, for build 189's 1,182,905. The mark's three lines
met.

**Still pictures, the cut against uncut** (2,560 x 1,600, the tenth frame; pixels of 4,096,000):

| mark | build 189: differ at all, by 4 of 255 or more, the most | build 190 |
|---|---|---|
| on the ground at the spawn | 496, none, 2 | 480, none, 2 |
| the mountain top | 12,208, 64, 5 | 576, none, 2 |
| the bare plain | 6,080, 3,072, 7 | 128, none, 1 |
| low over the sea | 1,472, 128, 5 | 960, none, 2 |
| high over the spawn | 1,712, 352, 7 | 784, 16, 7 |

No pixel differs by 8 or more in either build. The cut's picture is nearer the uncut one at every mark: a length
begun from a twentieth gathered on past where the whole ray stops, a little more ground and less sky than the
whole ray's.

**The uncut picture, build 190 against build 189:** the plain's and the sea's the same to the pixel; the spawn's
208 pixels differ (the most 4), the mountain top's 80 (the most 1), high over the spawn's 672 (the most 6): the
test's own noise from run to run, as it has been (up to 6 of 255 at a few rays).

**A moving view: the first frame after a turn of 2 degrees, the cut against uncut:**

| mark | build 189: by 4 or more, by 8, by 16, by 32, the most | build 190 |
|---|---|---|
| the bare plain | 2,816, none, none, none, 7 | 1,984, none, none, none, 7 |
| low over the sea | 464, 128, none, none, 10 | 288, 64, none, none, 10 |
| high over the spawn | 2,112, 1,008, 656, 256, 51 | 864, 160, none, none, 9 |

The mark's line (no more pixels by 8 or more than build 189's, the most no greater) is met at the three; high
over the spawn is where the guard shows: build 189's worst were rays the sky showed through for a frame, 51 of 255
out; they're filled with the ground's own colour now, the worst 9.

**All 34 tests pass.** No mistake of mine found on the way this time. (One line of the picture test was wrong and
is put right, written under the flight above: the threads' steps can be more than the rays'.)

**Owed:** the flight, on David's word: the bare plain's march within 2% of build 186's 0.450 ms, the sea
nodding's within 2% of 0.561, no other mark over build 189's by more than 2%.

### Build 190 flown (2026-11-15, 18:31 to 18:38, hands off): the mark met; the plain back to 0.448 ms, the nodding marks cheaper than they have been

`logs/flight-1794789487.txt`, "the code committed" (d1fe13c), on David's word ("go"); the game as launched, as
builds 186 and 189 were flown. No hitch. The march (the far 8 x 8 on the card, ms, the median):

| mark | build 186 | build 189 | build 190 | against build 189 |
|---|---|---|---|---|
| on the ground at the spawn | 0.363 | 0.363 | 0.363 | the same |
| the mountain top | 0.914 | 0.913 | 0.923 | 1.1% more |
| the bare plain | 0.450 | 0.502 | **0.448** | 10.8% less |
| high over the spawn | 0.958 | 0.963 | 0.980 | 1.8% more |
| low over the sea | 0.528 | 0.537 | 0.532 | 0.9% less |
| a player's run over the plain | 0.591 | 0.583 | 0.584 | the same |
| fast across the world | 0.616 | 0.622 | 0.622 | the same |
| the plain, nodding | 0.473 | 0.478 | **0.451** | 5.6% less |
| the sea, nodding | 0.561 | 0.583 | **0.518** | 11.1% less |

The card's whole over the nine views: 6.548 ms, the least yet (build 186: 6.582; build 189: 6.663). Two views
over 1 ms, as before.

**The mark's lines:** the bare plain within 2% of build 186's 0.450: 0.448, met. The sea nodding within 2% of
0.561: 0.518, 7.7% under it, met. No other mark over build 189's by more than 2%: the most is high over the
spawn's 1.8%, met. The plain's longest chain is 127 again, the sea's too (it was 153).

**The dearest view reads a little dearer, said plainly:** high over the spawn's march 0.980 for 0.963 (the card's
whole 1.109 ms for 1.091), and the mountain top's 0.923 for 0.913: the two marks where the card works. By the
card's own counts it did the same work at both (high over the spawn: 6,043,136 thread-steps taken and 7,536,160
held, for build 189's 6,043,398 and 7,535,616; the mountain top: 6,604,394 and 7,492,384 for 6,606,472 and
7,496,128), and nothing in the march's program changed but one number. What differs is the machine: the card's
clock at its least was 2,625 MHz at the mountain top for 2,700, the card 69 degrees for 66, the processor's area
up to 87 for 78 (an hour of picture tests ran right before this flight; build 189's flew after a rest). Builder
reads the 1 to 2% as the session's heat, not the build. One flight can't prove that: the next flight flown for
any reason says. If high over the spawn's march reads 0.98 again from a cool start with these counts, it's the
code and is looked into.

**What the mend gave beyond its mark:** the moving marks. The plain nodding is 0.451 for build 186's 0.473 (the
card's whole at its 99th 0.626 ms for 0.764), the sea nodding 0.518 for 0.561. In a moving view a ray's standing
at a cut changes every frame, so lengths begun from a twentieth marched on past the ray's end there all the time,
not only at one ray.

**Kept; the cut is done enough to go on, again.** David's eye on a moving view when he next plays (the guard
changed: on a turn, where sky showed through a cut ray for a frame, the ground's own colour fills now); nothing
is asked of him for it.

### Build 191, the skyline talk's first step, cover by faces: David's words, why thin things vanish, and the pass mark (written before the code)

**The talk (2026-11-15).** Builder's proposal to David, in three steps: (1) cover by faces, below; (2) his weights
by kind on top, if he still wants things pulled up after seeing step 1; (3) the land's own edge by a height table
(the Oracle's cover), the bigger build, weighed after lazy rays. **His answer:** "first send a list of any class
of items on screen that could conceivably recieve weights, and begin. i do like the crisper line of the 'cover' as
it was shown to me, against the skyline, and other things against other things, hopefully; but pay careful
attention to the weight of the shader. i hope it doesn't become too much and start squeezing no matter what we
do." And, a little after: "by weight of the shader i mean its size on the card". The list was sent (26 classes:
by what a thing is made of and by its shape, both known where the far cells are summed and so free in the frame;
by where it sits in the picture, each priced by a flight; by light; and what isn't in the far cells yet).

**Why thin things vanish** (read from the march, `far.wgsl`): a step through a cell is covered by the cell's
solid share, `1 - (1 - share) ^ (length / size)`: crossing a whole cell stops a ray by as much of the cell as is
full. A wall one block thick, face-on in a cell of 8, is an eighth full and stops an eighth of the ray; in truth
it hides everything behind it. A post one block square is a 64th full and stops a 64th; in truth it covers an
eighth of the rays that cross its cell. The bulk rule is right only for a thing as deep along the ray as its cell.

**What the far cells already hold:** each cell's faces over its floor area (`surface`) and their shares by the six
ways they look (`plus`, `minus`), fetched at every step and used for light alone. Their product with the ray's own
way is the share of the cell's cross-section its faces show the ray: the true share of rays the cell stops, on
average, however thin the thing.

**What's to change:** a look's solid share becomes the greater of its bulk and the face it shows the ray (not over
1), where the look is made, each level's own before the hand-over turns it into the finer cell's and mixes it. Not
where the cell holds water (by its water share): a bed under water lies at one height in its cell, and counting
its face through the whole cell would bring it up through the water over it [the bright bands of build 188 came of
ground counted too high through water]. Nothing new is stored, fetched or carried through the march's loop.
A switch, `INCUS_FAR_FACES=0`: the rule compiled out, build 190's program.

**The pass mark.** Off the screen:

- **Off, it's build 190:** the five marks' pictures with `INCUS_FAR_FACES=0` are build 190's but for the test's
  own noise.
- **Thin things hold** (the stage on, 2,560 x 1,600, on against off): a castle wall seen face-on isn't seen
  through (the ground's colour behind its foot gone from it); the arch's piers and the posts stronger than they
  were; a house's roof from above is its own colour. Shown as pictures side by side, with what was measured.
- **The land's own skyline is left alone:** at the plain and low over the sea (level views of gentle ground), the
  skyline band's pixels change by no more than 8 of 255, but for a handful.
- **No bands over shallow water:** from 300 over the shallow shelf looking down (build 189's view), on against
  off: no outline, no bands.
- **The five marks' pictures on against off** given as numbers, and looked at: what changes and where, said.
- **The steps:** each mark's steps summed, on against off, given (more cover should end rays sooner).
- All the tests pass.

**Its size on the card** (David's worry, and the wall of build 185): nothing new carried through the loop, by the
code; and read where it shows, in Nsight's window: one trace at the mountain top with the rule off and one with it
on, David reading `march_all`'s row from each (the groups that fit, the registers, what's put by). The mark: 12
groups still fit, and nothing more is put by than with it off. [Last read at build 178: 167 registers, none put
by, 12 groups; about 170 is where 12 groups stop fitting. Not read since the cut went in, so the off row is news
too.] If it grows past that: said, and the cheaper form tried (the same rule folded into the cells' solid share
where they're summed, by the level ways only: nothing in the program, less exact).

By flights (on David's word, one session, off then on): the two marks where the card works within 2% of off; no
other mark over off by more than 2%. A miss is said, with what the counts show.

**Then David's look** on the stage, which decides it: the crisper line he liked, "against the skyline, and other
things against other things".

### Build 191 built (unflown, unlooked): cover by faces; what was built and what was checked off the screen

**Built as the mark says** (`far.wgsl`, `by_faces`): a look's solid share is the greater of its bulk and the face
it shows the ray (its faces over its floor area, by the ways they look, times the ray's way; not over 1), each
level's own look before a hand-over turns it into the finer cell's and mixes it. Kept out where the cell holds
water, any to speak of (a 64th of it; the mark said "by its water share", and two blocks of sea are only an eighth
of a cell of 16, so the share alone would have left the rule nearly whole over a shallow bed: tightened by
reasoning, no band was seen either way). From what the look already holds: nothing more is stored, fetched or
carried through the march's loop. `INCUS_FAR_FACES=0` compiles it out (one line of the program's text changed,
as `LOADED` is).

**Off, it's build 190:** the five marks' pictures (2,560 x 1,600, the whole world loaded, the tenth frame)
against build 190's: the plain's the same to the pixel; the others 64 to 640 pixels differing, the most 6 (the
test's own noise).

**Thin things hold** (the stage on; pictures on against off kept in the session's scratch, the pairs shown to
David):
- *The castle from above* (330 up, looking 40 degrees down): its walls are solid pale bands round the court where
  they were faint lines the court's ground showed through; the corner towers and the keep firmer.
- *The plain's group from the plain's mark:* the arch reads, both piers and its round top, where it was a hint;
  the slab in the air stronger; the great tower's spire, 1,500 blocks off, shows against the sky where there was
  nothing.
- *The town from above:* the houses' roofs are their own red where they were mostly their walls' pale colour
  (a roof is a thin shell over a hollow house).
- **And the woods become solid masses,** dark, where the ground showed between and through the trees. True to a
  wood, by the same rule; but it is the largest change on the screen, and it hides more of what stands behind a
  wood (from the east at 300 blocks, the wood in front now covers the castle's foot). David's eye.
- Close to (30 blocks from the castle's wall) everything is soft, on or off: near things are the near eye's, and
  the far eye marches few of its finer rays there.

**No bands over shallow water:** from 430 over the shallow shelf looking down, on against off, the sea's part of
the picture is the same but for a few pixels (the most 5 of 255); its mean colour by rows the same to 0.1.

**The land's own skyline.** The mark's line (the plain and the sea: the skyline's pixels change by no more than 8
of 255, but for a handful) **is missed at the plain as written, and said:** 7,168 pixels (0.17%) change by 8 or
more, the most 32. They're the far mountains that stand on the left of the plain's horizon with the whole world
loaded: steep land, whose edge the rule firms up (their tips a little fuller and darker). The level skyline
itself, the right nine sixteenths of the width, has 128 such pixels; with only the regions round the mark loaded
(no far mountains) nothing changes by more than 2. The line was written for gentle ground and the plain's horizon
isn't all gentle. Low over the sea: 39,200 pixels (1%) by 8 or more, the most 109: the land in the foreground,
where the line between grass and sand and between sand and shallows moves a fraction of a cell (a ray is stopped
a little sooner in its surface cell), and wooded land at the right of the horizon; the sea itself and the level
horizon don't change.

**The five marks, on against off** (pixels of 4,096,000):

| mark | differ at all | by 4 of 255 or more | by 8 or more | by 16 or more | the most |
|---|---|---|---|---|---|
| on the ground at the spawn | 29% | 203,168 | 70,144 | 45,520 | 78 |
| the mountain top | 33% | 132,960 | 44,000 | 9,280 | 58 |
| the bare plain | 5% | 21,184 | 7,168 | 384 | 32 |
| low over the sea | 30% | 408,144 | 39,200 | 1,536 | 109 |
| high over the spawn | 61% | 419,376 | 64,928 | 12,576 | 116 |

Where, on bare land: a rim along steep land's edge against the sky or against land behind it (the edge a pixel or
so fuller), thin lines along ridges and terraces, and a faint speckle; side by side at twice the size the two
pictures are hard to tell apart. So bare land changes everywhere a little, and at its steep edges by more: David's
eye on it as well as on the stage.

**The steps** (the 8 x 8's and the 4 x 4's, on against off): the spawn 0.2% fewer and 3.2% more; the mountain top
0.2% fewer and the same; the plain 0.1% fewer; the sea 0.4% more and 0.4% more; high over the spawn 0.2% more and
0.8% more. Level, not fewer as the mark guessed (with the stage on, where there's more to stop a ray, 2 to 10%
fewer and a fifth fewer long rays).

**All 34 tests pass** (the map redrawn for the new switch).

**A mistake of mine on the way:** low over the sea the first pictures showed a coloured band in the foreground,
on against off; I took it for the sea bed showing through shallow water and wrote that into the program's notes
as seen. It was the land in front (grass, sand), and the notes are put right: the water rule rests on reasoning.

**Owed:** its size on the card (two Nsight rows, off and on, David's to read); the flights, off then on in one
session; David's look on the stage.

### Build 191 traced and flown (2026-11-16, 06:07 to 06:31, hands off): cover by faces costs about 2% of the march at every mark; two marks over the line

On David's word ("go"). Two Nsight traces at the mountain top, the rule off and on
(`logs/nsight/b191/b191-off/incus_artus_2026_11_16_06_09_15.ngfx-gputrace`,
`logs/nsight/b191/b191-on/incus_artus_2026_11_16_06_11_13.ngfx-gputrace`; each session's log says it stood at the
mountain top, and which program it ran), then three flights as launched: off (`logs/flight-1794831480.txt`), on
(`-1794831876`), off again (`-1794832272`), all "the code committed". No hitch. **Before take-off the card read
about 20% busy with the desktop's own compositor** (4% before yesterday's flights); flown all the same, since the
off flights are build 190's program and say whether the session is comparable: it is (below).

The march (the far 8 x 8 on the card, ms, the median):

| mark | off | on | off again | on against the two offs' mean |
|---|---|---|---|---|
| on the ground at the spawn | 0.362 | 0.372 | 0.362 | **2.8% more** |
| the mountain top | 0.919 | 0.939 | 0.929 | 1.6% more |
| the bare plain | 0.446 | 0.455 | 0.448 | 1.8% more |
| high over the spawn | 0.973 | 0.996 | 0.982 | 1.9% more |
| low over the sea | 0.530 | 0.543 | 0.538 | 1.7% more |
| a player's run over the plain | 0.591 | 0.606 | 0.591 | **2.5% more** |
| fast across the world | 0.662 | 0.651 | 0.617 | (the two offs 7% apart: says nothing) |
| the plain, nodding | 0.449 | 0.455 | 0.450 | 1.2% more |
| the sea, nodding | 0.518 | 0.529 | 0.519 | 2.0% more |

The card's whole over the nine views: 6.576, 6.671, 6.559 ms: on 1.6% over the offs' mean. The dearest view, high
over the spawn: 1.125 ms on, for 1.102 and 1.111.

**Against the mark:** the two marks where the card works within 2% of off: 1.6% and 1.9%, met (the second by a
hair). No other mark over off by more than 2%: **missed at the spawn (2.8%) and the run (2.5%)**, and the sea
nodding sits on the line.

**Builder's reading:** it costs the same where the card waits as where it works, and the steps are level, so it's
the rule's own arithmetic at every step (it's worked out at every look, in air too, in five places of the program),
not a program grown past what the card holds [build 185's sign was the two working marks alone, 4.5%]. David's
reading of the two rows says whether that's right. A cheaper form is next (build 192, its mark below).

**Yesterday's reading of build 190 corrected.** Its flight read the two working marks 1 to 2% over build 189's,
with the same work counted; Builder put it down to a hot machine and wrote that a cool start would say. Today's
off flights are build 190's program from a cold machine (the card at 26 degrees before take-off): the mountain top
0.919 and 0.929, high over the spawn 0.973 and 0.982, for build 189's 0.913 and 0.963 and build 190's 0.923 and
0.980. So it wasn't the heat. And two flights of the one program in one session differ by 1% at these marks: a
single flight can't tell a 1% change from none. Build 190 may be up to 1% dearer than 189 where the card works,
or not at all; it isn't chased, and Builder stops reading 1% between single flights as anything. (A mark's line
of 2% needs the bracket flown here, off, on, off, to mean something.)

### Build 192, cover by faces where a step is gathered: the pass mark (written before the code)

**What's to change:** the same rule, moved. The march already works out how much face a cell shows the ray, for
its light, at each step that gathers (`faces_light`, for both ends of the step). There the step's cover becomes
the greater of the bulk's share and the faces' (the two ends' mean), with one more power. In a hand-over the
coarser look's faces are turned into the finer cell's as its bulk is (`by_finer_cell`), so the rule is still each
level's own. Steps through air, the most of them, do nothing new; and the stepping itself goes by bulk as build
190's did, so the path of a ray's steps is build 190's until its cover ends it.

**The pass mark.** Off the screen:

- **Off, it's build 190:** the five marks' pictures with `INCUS_FAR_FACES=0` are build 190's but for the test's
  own noise.
- **On, it's build 191's look:** the stage's three views (the castle from above, the plain's group, the town from
  above) show what build 191's did (walls solid, the arch and the far spire read, roofs their colour, woods solid);
  the pictures on against build 191's on, as numbers, and looked at.
- **No bands over shallow water** (the shelf's view, on against off).
- **The five marks on against off**, as numbers; **the steps** no more than off's by over 1% at any mark.
- All the tests pass.

Its size on the card: the off row is build 191's off trace (the same program); one new trace with it on, David's
to read. The mark as build 191's: 12 groups still fit, nothing more put by than off.

By flights (on David's word; off, on, off again in one session): every mark's march within 1% of the two offs'
mean, or at least under build 191's cost at every mark. If it isn't cheaper than build 191, said, and build 191's
form stays the one David looks at.

### Build 191's two rows, read by David off Nsight's window (2026-11-16): the program hasn't grown since build 178; build 191's rule put two values by

`march_all` at the mountain top, from his screenshots of the two traces:

| | the rule off (build 190's program) | on (build 191) | build 178, for comparison |
|---|---|---|---|
| groups that fit (# Warp) | 12 | 12 | 12 |
| registers (# Reg) | 167 | 168 | 167 |
| live registers | 163 | 164 | 164 |
| put by (Smem) | 0 | 512 | 0 |
| instruction mix | 3.94K | 4.06K | |
| share of the samples | 86% | 86% | 86% |
| top waits | WAIT 35%, LGSB 17%, SELECT 16% | WAIT 35%, LGSB 16%, SELECT 16% | Wait 35%, Long Scoreboard 16%, Selected 15% |

(Also in the off trace: `join_pass` 28 groups, 72 registers, 1,024 put by, 1.4% of the samples; `start_pass` 40
groups, 46 registers, 256 put by; `brought_up` 11% of the samples.)

- **The cut added nothing to the march's size on the card:** the row is build 178's, register for register, with
  builds 183 to 190 in it. Three under the line of about 170 where 12 groups stop fitting.
- **Build 191's rule did press on it:** one more register and two values a ray put by in memory (512 bytes is two
  words for 64 rays). By the measure of build 178's rows (registers and what's put by) the need went from 167 to
  170, onto the line. The 12 groups still fit.
- **Build 191's mark said "nothing more is put by than with it off": missed.** And Builder's reading of its flights
  (the rule's own arithmetic at every step, not the program's size) was partly wrong; said to David, whose worry it
  was ("pay careful attention to the weight of the shader ... its size on the card").

### Build 192 built (untraced, unflown, unlooked): cover by faces where a step is gathered; what was built and what was checked off the screen

**Built as the mark says** (`far.wgsl`): `faces_light` gives back how much of a look's faces the ray sees beside
their light; where a step gathers, its solid share is the greater of its bulk and the mean of what its two ends
show (`shown_share`: the faces over the floor area times what's seen, none where the cell holds a 64th of water),
with one more power; `by_finer_cell` turns a coarser look's faces into the finer cell's as it does its bulk. The
five places where build 191 worked the rule out at every look are gone, and a look's solid share is its bulk
again: a ray's steps go by bulk as build 190's.

**Off, it's build 190:** the five marks (2,560 x 1,600, the whole world loaded) against build 190's: the plain's
and the sea's the same to the pixel, the others 128 to 848 pixels differing, the most 6 (the test's own noise).
Against build 191's off pictures of the stage's views: the same to the pixel.

**On, it's build 191's look.** Build 192 on against build 191 on, pixels by 8 of 255 or more (of 4,096,000) and
the most: the spawn 256 and 23; the mountain top 816 and 22; the plain none and 6; the sea 976 and 36; high over
the spawn 4,704 and 83; the castle from above 7,664 and 12; the plain's group 736 and 49; the town from above
11,216 and 161 (a few pixels at a far corner); the shelf 3,712 and 11. Side by side the castle from above can't
be told apart; the arch, the slab and the far spire read as in build 191; the woods are solid as in build 191.

**No bands over shallow water:** the shelf's sea, on against off, is the same to the pixel (2,457,600 pixels,
none differing): the steps go by bulk and the rule keeps out of water.

**The five marks, on against off:**

| mark | differ at all | by 4 of 255 or more | by 8 or more | by 16 or more | the most | build 191's by 8 or more |
|---|---|---|---|---|---|---|
| on the ground at the spawn | 20% | 166,800 | 67,872 | 42,688 | 80 | 70,144 |
| the mountain top | 28% | 123,456 | 40,960 | 8,464 | 57 | 44,000 |
| the bare plain | 3% | 17,408 | 5,824 | 384 | 31 | 7,168 |
| low over the sea | 29% | 362,704 | 18,128 | 1,376 | 108 | 39,200 |
| high over the spawn | 57% | 343,888 | 54,080 | 9,472 | 110 | 64,928 |

The same changes as build 191's, a little less of each.

**The steps, on against off** (both layers summed): the spawn 0.6% more; the mountain top 0.5% fewer; the plain
0.1% fewer; the sea 0.1% fewer; high over the spawn 0.8% fewer. The mark's line (no more than off's by over 1% at
any mark) is met by each mark's sum; by layer the spawn's 4 x 4 is 2.4% more, said: more of its rays are marched
(13,628 for 13,268: more of the screen is worth the finer layer where edges are firmer), not longer ones.

**All 34 tests pass.**

**Owed:** its size on the card (a trace with it on, and one off again to see the off program is the one David
read: its hash was 0x63fca007acf1f0c6); the flights, off, on, off; David's look on the stage.

### Build 192 traced and flown (2026-11-16, 07:03 to 07:27, hands off): under 1% at five marks, both working marks among them; 1.6 to 2.8% where rays gather at every step; the run a miss

On David's word ("go"). The card idle before take-off this time (4%), mains, one screen, Nsight's window closed.
Two traces at the mountain top (`logs/nsight/b192/b192-off/incus_artus_2026_11_16_07_05_44.ngfx-gputrace`,
`logs/nsight/b192/b192-on/incus_artus_2026_11_16_07_07_41.ngfx-gputrace`; each session's log says where it stood
and which program it ran), then three flights: off (`logs/flight-1794834872.txt`), on (`-1794835265`), off again
(`-1794835657`), all "the code committed". No hitch.

The march (the far 8 x 8 on the card, ms, the median):

| mark | off | on | off again | on against the two offs' mean | build 191's |
|---|---|---|---|---|---|
| on the ground at the spawn | 0.364 | 0.367 | 0.364 | 0.8% more | 2.8% |
| the mountain top | 0.932 | 0.932 | 0.931 | the same | 1.6% |
| the bare plain | 0.448 | 0.452 | 0.451 | 0.6% more | 1.8% |
| high over the spawn | 0.971 | 0.981 | 0.977 | 0.7% more | 1.9% |
| low over the sea | 0.532 | 0.541 | 0.533 | 1.6% more | 1.7% |
| a player's run over the plain | 0.583 | 0.600 | 0.584 | **2.8% more** | 2.5% |
| fast across the world | 0.610 | 0.631 | 0.620 | 2.6% more | (unreadable) |
| the plain, nodding | 0.450 | 0.455 | 0.453 | 0.8% more | 1.2% |
| the sea, nodding | 0.517 | 0.526 | 0.518 | 1.6% more | 2.0% |

The card's whole over the nine views: 6.522, 6.610, 6.551 ms: on 1.1% over the offs' mean (build 191: 1.6%). The
dearest view, high over the spawn: 1.110 ms on, for 1.099 and 1.105. The two offs agree to 0.6% or better at every
mark but fast across (1.6%).

**Against the mark** (every mark within 1% of the offs' mean, or at least under build 191's cost): within 1% at
five marks, the two where the card works among them; under build 191's at the sea (barely) and the sea nodding;
**not under it at the run (2.8% for 2.5%): a miss, said.**

**What the counts say.** With the rule on the card's counted work is the same or less at every mark (thread-steps
held, on against the offs: the mountain top 0.4% fewer, high over the spawn 0.7% fewer, the sea the same, the run
0.2% more, the spawn 0.8% more). So what the rule costs is per step, and it's left where rays gather at nearly
every step: through water (the sea marks), and skimming grass at walking height (the run). Builder's reading: at
each step that gathers, build 192 works out a second power (the bulk's cover first, as build 190, to know whether
the step gathers at all; then the cover again with the faces in). Where most steps are air or end in solid ground,
that's few steps; over water and along grass it's most of them. Not proven: the form with one power, flown, says.

**Owed:** David's reading of the on trace's row (does this form fit with nothing put by); his look on the stage,
which decides whether the rule is wanted at all.

### Build 192's row, read by David off Nsight's window (2026-11-16): the rule where a step is gathered fits, for one register

`march_all` at the mountain top, from his screenshots of build 192's on trace, beside the two he read before:

| | the rule off (build 190's program) | build 191's rule | build 192's rule |
|---|---|---|---|
| groups that fit (# Warp) | 12 | 12 | 12 |
| registers (# Reg) | 167 | 168 | 168 |
| live registers | 163 | 164 | 165 |
| put by (Smem) | 0 | 512 | 0 |
| instruction mix | 3.94K | 4.06K | 4.06K |
| share of the samples | 86% | 86% | 86% |
| top waits | WAIT 35%, LGSB 17%, SELECT 16% | WAIT 35%, LGSB 16%, SELECT 16% | WAIT 35%, LGSB 16%, SELECT 16% |

**The mark (12 groups still fit, nothing more put by than off): met.** Build 192's form puts nothing by; it
takes one register, 168 for 167: two under the line of about 170 where it was three. Build 191's form was the one
that spilled (the same register and two values a ray put by). So the rule's size on the card, in this form, is one
register: affordable, not free, and that register of room is gone for the next thing that wants into the march.

(Not read: the off trace of build 192, taken to see by its hash that the off program is the one David read at
build 191, 0x63fca007acf1f0c6. Its pictures are build 190's to the pixel and its flights read as build 191's offs;
the hash is there in `logs/nsight/b192/b192-off/` if it's wanted.)

**Where cover by faces stands, for David's look:** its look checked off the screen (builds 191 and 192: the same);
its size on the card one register; its time under 1% at five marks, both working marks among them, 1.6% at the sea
marks and 2.8% on the run (one power where there are two is the next form, unflown). His look on the stage decides
whether it's wanted.

### Build 193, a live switch for cover by faces: David's words and the pass mark (written before the code)

**David (2026-11-16), before his look:** "do i have an a/b switch?" He didn't: the rule was chosen when the game
starts. Builder's proposal: the game compiles the march's other program beside the one the launch asks for, and a
tick box swaps which of the two marches; neither carries a switch inside it, so nothing is added to either's size
on the card and each is the program whose row he read; it costs a longer start. Where it goes and what it's called
was asked of him: the Render page, "Cover by faces", ticked by default. **His answer:** "yes, that place is fine
for the switch."

**What's to change:** `far_eye.rs` compiles the loaded march a second time with the rule the other way (one more
program); the look gains `faces`, which the Render page's tick box sets and which picks the program once the world
is loaded (till then the whole march draws, by the launch's choice). `INCUS_FAR_FACES=0` still starts the game
with the rule off everywhere, and then the tick box starts unticked. Nothing in `far.wgsl` changes.

**The pass mark.** Off the screen:

- **Each side of the switch is build 192's program:** ticked, the five marks' pictures are build 192's on
  pictures; unticked, build 192's off pictures (build 190's), each but for the test's own noise; and the text
  each is compiled from is, by the code, the text the launch's choice compiles (so the rows David read stand, and
  the flights flown stand: none is asked for this build).
- **Flipped while it runs:** a frame drawn ticked, then unticked: the second frame's picture is the unticked
  picture's (but for what a ray keeps from the frame before); no error from the card.
- **Greyed, not hidden,** where it can do nothing: while the world loads, and where there's no second program
  (the layers marched one after another, or an audit's version marched in its place).
- **The start:** how much longer, measured and said.
- All the tests pass.

**Then David's look**, the switch in his hand.

### Build 193 built (unlooked): the live switch for cover by faces; what was built and what was checked off the screen

**Built as the mark says.** `far_eye.rs` compiles the loaded march a second time from the same text with
`far.wgsl`'s `FACES` the other way (`one_loaded_other`); the look gains `faces`; the dispatch marches the launch's
loaded program or the other by it. The Render page, the picture's section, under the sky weight: a tick box
"Cover by faces", ticked by default, the section's Default returning it; greyed (by the code: `faces_switch`)
while the world loads and where there's no second program. `INCUS_FAR_FACES=0` starts the game with the rule off
in every program and the box unticked. Nothing in `far.wgsl` changed. A knob for the picture test,
`FAR_TEST_FACES` (0, 1, or 2: on till the last frame, which is drawn off).

**Each side of the switch is build 192's program** (2,560 x 1,600, the whole world loaded, the tenth frame):
ticked, against build 192's on pictures: none to 3,408 pixels differing, the most 6; unticked (by the switch, the
other program), against build 192's off pictures: none to 720, the most 6. The test's own noise; the sea's and the
plain's the same to the pixel. By the code each text is the launch's text with the one line changed, as the
launch's own choice changes it: so the rows David read and the flights flown stand, and none is asked for this
build.

**Flipped while it runs** (ticked nine frames, the tenth drawn unticked): at the plain the flipped frame is the
unticked picture (1,280 pixels differing, the most 6); high over the spawn it's the unticked picture but for 2,432
pixels by 8 of 255 or more (the most 97): what a ray keeps from the frame before (a finer layer is led by the
coarser layer of the frame before, and a cut ray's lengths begin as it stood), gone in the frames after. No error
from the card.

**The start:** the other program takes 5.4 to 6.9 s to make (twelve runs), so the game starts about 6 s later.

**All 34 tests pass** (the map redrawn for the switch's words).

**Not checked off the screen:** the tick box itself (its greying, its Default), which needs the panel: David's
eye at his look, and said to him.

### Build 193, David's look at cover by faces (2026-11-16): "looks good"

Launched on his word ("launch") with the stage on, at the castle (`logs/session-1794837292.log`: it stood at the
stage's castle; the march's other program made in 0.6 s there, not the 5 to 7 s the picture test took, so the
switch costs the start next to nothing; 103 s, closed by him, no error in the log). He was asked whether the tick
box worked once the world had loaded and how the castle's walls and the woods looked, ticked against unticked.
**His words:** "looks good".

**Builder's reading** (hers, not his words): the look is wanted, the woods as solid masses with it; cover by faces
is kept, on as the game is launched, and the tick box stays on the Render page. He didn't say whether the box was
greyed while the world loaded: not confirmed. His weights by kind (the talk's second step: "if he still wants
things pulled up after seeing step 1") weren't asked for; the list of what could take a weight is with him and
Counsel.

**So cover by faces is done enough to go on.** What it costs, as flown (build 192's programs): the card's whole
over the nine views 1.1% more; under 1% at five marks, the two working marks among them; 1.6% at the sea marks and
2.8% on the run. Its size on the card: one register (168 for 167), nothing put by, 12 groups.

### Build 194, one power where a step gathers: the pass mark (written before the code)

**What's to change** (`far.wgsl`, with cover by faces on; off stays build 190's program to the letter): build
192 works out a step's cover twice where it gathers (the bulk's first, to know whether the step gathers at all,
then again with the faces in). A step gathers exactly when its two ends hold any solid or any water, which is two
sums: so with the rule on the test is that, and the power and the water's exponential are worked out once, inside,
with the faces already in. A step through air then works out neither.

**The pass mark.** Off the screen:

- **On, it's build 192's picture** at the five marks and the stage's views, to the test's own noise (the same
  numbers are worked out, in another order).
- **Off, it's build 190's**, the same way (nothing of it changes).
- The steps, on, are build 192's on.
- All the tests pass.

Its size on the card: a trace with it on; the mark, no more than build 192's row (168 registers, nothing put by,
12 groups). By flights (off, on, off, in a window David gives): no mark's march over the two offs' mean by more
than 1%; the sea marks and the run, where build 192 cost 1.6 to 2.8%, are the ones it's for. If it's no better
than build 192 there, said, and build 192's form stays.

### Build 194 built (untraced, unflown): one power where a step gathers; what was built and what was checked off the screen

**Built as the mark says** (`far.wgsl`, the step's gathering): with cover by faces on, a step is tried for
gathering if its two ends hold any solid or water (two sums), and its cover (the power, the water's exponential)
is worked out once, inside, with the faces in; a share too faint to cover anything still gathers nothing (the
cover's own test kept inside, so the picture is build 192's). By bulk alone (the rule off) the lines are build
190's, in their order. The audit's "no floor" version matched a piece of this by its text, moved one level in:
its text follows.

**On, it's build 192's picture** (2,560 x 1,600, the whole world loaded): the plain's, the sea's, the castle
from above and the shelf the same to the pixel; the spawn, the mountain top and high over the spawn 592 to 2,208
pixels differing, the most 6 (the test's own noise). **Off, it's build 192's off** (build 190's): none to 576
pixels, the most 6. **The steps, on, are build 192's on** to a step or two at every mark. **All 34 tests pass.**

David, asked whether it should wait for another build's window: "we can do the trim and checks now." So its trace
and flights (off, on, off) go up as soon as this is committed.

### Build 194 traced and flown (2026-11-16, 08:19 to 08:42, hands off): cover by faces now costs about half a percent; the sea marks 0.5 to 0.7% where they were 1.6%

On David's word ("we can do the trim and checks now"). The card idle before take-off (6 to 9%), mains, one screen,
Nsight's window closed. Two traces at the mountain top
(`logs/nsight/b194/b194-off/incus_artus_2026_11_16_08_20_43.ngfx-gputrace`,
`logs/nsight/b194/b194-on/incus_artus_2026_11_16_08_22_41.ngfx-gputrace`), then three flights: off
(`logs/flight-1794839365.txt`), on (`-1794839756`), off again (`-1794840152`), all "the code committed". No hitch.

The march (the far 8 x 8 on the card, ms, the median):

| mark | off | on | off again | on against the two offs' mean | build 192's |
|---|---|---|---|---|---|
| on the ground at the spawn | 0.373 | 0.368 | 0.370 | 0.9% less | 0.8% more |
| the mountain top | 0.926 | 0.931 | 0.936 | the same | the same |
| the bare plain | 0.450 | 0.453 | 0.451 | 0.6% more | 0.6% |
| high over the spawn | 0.979 | 0.988 | 0.977 | 1.0% more | 0.7% |
| low over the sea | 0.533 | 0.539 | 0.538 | 0.7% more | 1.6% |
| a player's run over the plain | 0.586 | 0.605 | 0.617 | (the two offs 5% apart: unreadable) | 2.8% |
| fast across the world | 0.619 | 0.627 | 0.614 | 1.7% more | 2.6% |
| the plain, nodding | 0.453 | 0.456 | 0.454 | 0.6% more | 0.8% |
| the sea, nodding | 0.518 | 0.524 | 0.525 | 0.5% more | 1.6% |

The card's whole over the nine views: 6.564, 6.616, 6.614 ms: on 0.4% over the offs' mean, the offs themselves
0.8% apart. The dearest view, high over the spawn: 1.116 ms on, for 1.108 and 1.106.

**How good the session is:** worse than the morning's two. The machine was hot by the second flight (the
processor's area at 103 to 104 degrees at several marks). In the on flight the card's clock dipped at high over the
spawn (2,445 MHz at its least, for 2,647 in both offs; that mark's 95th 1.122 for 0.999 and 0.997, its worst
4.9 ms) and at fast across (under 2,000 MHz for 1.9 s). So on's median at high over the spawn leans high, and the
run's two offs differ by 5%.

**Against the mark** (no mark's march over the two offs' mean by more than 1%): met at six marks; high over the
spawn sits on the line (1.0%, with the clock's dip in it); fast across is over it (1.7%); the run can't be read.
**What it was for:** the sea marks, 0.5 and 0.7% where build 192 cost 1.6%: done. The run isn't shown either way
by this session.

**So, as flown:** cover by faces costs the march about half a percent, at the edge of what a bracket of three
flights can tell on this machine; nowhere is it clearly over 1% but fast across, a mark whose 99th is twice its
median. Kept as it is. **Owed:** David's reading of the on trace's row (the mark: no more than build 192's 168
registers, nothing put by, 12 groups).

**For the next windows (Builder's note):** three flights and two traces straight after an hour of picture tests
leave the machine hot by the last flight; a few minutes' rest before a bracket, and the card's clock read beside
each mark's median, as here.

### Build 194's row, read by David off Nsight's window (2026-11-16): the trim changed nothing in the march's size; cover by faces closed

`march_all` at the mountain top, from his screenshots of build 194's on trace (its hash 0xad639e0fa55d7005): 86%
of the samples, **12 groups (# Warp), 168 registers (# Reg), 165 live, nothing put by (Smem 0)**, 4.06K in the
instruction mix; top waits WAIT 35%, LGSB 16%, SELECT 16%. Build 192's row to the number. **The mark (no more
than build 192's row): met.**

**Cover by faces, as it closes** (the skyline talk's first step; builds 191 to 194):
- *The look:* David, the live switch in his hand: "looks good". On as the game is launched; the Render page's
  "Cover by faces" tick box swaps the march's two programs.
- *Its size on the card:* one register (168 for 167), nothing put by, 12 groups; two of room under the line of
  about 170, where there were three.
- *Its time:* about half a percent of the march (build 194, flown off, on, off: the card's whole over the nine
  views 0.4% over the offs' mean, at the edge of what the bracket can tell).
- *What it took:* three forms. At every look in five places (build 191: 2% everywhere, and two values put by);
  where a step is gathered, with a second power (build 192: under 1% at five marks, 1.6 to 2.8% where rays gather
  at every step); one power (build 194). David's worry named the thing to watch before the first was flown ("pay
  careful attention to the weight of the shader ... its size on the card"), and his reading of the rows caught
  what Builder's reading of the flights had missed.
- *Left open, his to ask for:* his weights by kind on top (not asked for at his look); the land's own edge by a
  height table, weighed after lazy rays.

### For the lazy rays talk: how much of the two working marks' work lies beyond a distance (2026-11-16, off the screen, no code changed)

David, 2026-11-16: "what do we move on to next?" By the plan: lazy rays, starting with a talk. A measurement for
it, taken with the measuring switch from build 172 (`INCUS_TEST_REACH`: rays ended so many blocks out) in the
picture test (2,560 x 1,600, the whole world loaded, build 194): the thread-steps held (the layers' and the
strip's, by 32s), which is what the card's time follows at these two marks.

| rays ended at | the mountain top: held | of the whole | high over the spawn: held | of the whole |
|---|---|---|---|---|
| (whole) | 7,465,984 | | 7,472,640 | |
| 8,192 blocks | 6,436,128 | 86% | 6,925,568 | 93% |
| 4,096 | 5,513,088 | 74% | 6,351,264 | 85% |
| 2,048 | 4,258,432 | 57% | 5,361,344 | 72% |
| 1,024 | 2,149,408 | 29% | 4,617,504 | 62% |
| 512 | 1,138,816 | 15% | 1,302,048 | 17% |

So the work that lies beyond a distance, at the most: beyond 1 km, 71% of the mountain top's and 38% of high over
the spawn's; beyond 2 km, 43% and 28%; beyond 4 km, 26% and 15%; beyond 8 km, 14% and 7%.

**These are ceilings, not what lazy rays would save,** for two reasons said before the talk: a ray ended short
finds no surface, so the finer layer isn't marched for it at all and its near steps are dropped with its far ones
(the mountain top's 4 x 4 marches 11,512 rays at 1,024 blocks for 69,588 whole): the far share is over-counted;
and a lazy scheme still marches the far part some frames, re-marches it as the eye travels, and keeps books.
[Build 172's flight of the same switch read the mountain top's near kilometre at 32% of its march and high over
the spawn's at 57%: the same shape.]

### Lazy rays shelved by David (2026-11-16), and packed into a document to resume from

The talk was opened with the measurement above and a proposal (the near part of every ray each frame up to a lazy
line; everything beyond kept as a picture all round the eye, fixed to the world, refreshed a share a frame, put
behind the near picture with light and fog fresh; the march's program not grown; the line at about 4 km first,
then brought in a step a build) and two questions for him (far squares fixed to the world or the screen; refreshed
all at once or piece by piece). **His answer:** "lets hold off on lazy rays for now. we can keep that in our back
pocket if we ever get a performance squeeze limited by long rays. i think we can move on. just pack the
instructions and your output there into a lazy rays doc to resume with if we ever need it."

Done: `journal/docs/lazy-rays.md` (the Oracle's rules and tests; the three measurements, with how to take the
last again; the proposal and what it would stand on in the code; his two questions, unanswered; what isn't known;
how to resume). Nothing of it is built. The plan's item is marked shelved, not dropped.

### Cover by height shelved beside lazy rays, and the march scanned by its parts (2026-11-16, David's word)

Builder had asked whether the land's own edge by a height table should go in the back pocket or be built. David
first asked: "would the lands own edge by a height table work with mountains in front of mountains as well as in
front of sky". Builder: yes by its design (the edge is worked out at the near ridge, whatever is behind), where
today's sky weight sharpens only land against sky; with what it wouldn't change (a far ridge's sideways steps,
anything that isn't plain ground) and that none of it is built. **His answer:** "please pack it up next to lazy
rays, along with scanning the march and noting in a file anything that can later be removed from the march to free
up room on it; i would like for a sort of 'windirtree' view of the march by its components, and the size of those
components inside the march, if possible, so that later review could identify ways to further improve it by
removing sections that do little and replace them with something else. if i become dissatissfied with some views
in a fuller game i'll come back to cover by height and lazy rays and there may be a final check pass against the
engine later anyways."

**Done:**

- `journal/docs/cover-by-height.md`: shelved, not dropped; what it is, the Oracle's description, what the march
  already has, what isn't known, how to resume.
- `journal/docs/march-by-parts.md`: the march at build 194 by its parts, with what could later come out.
- **An instrument for it** (no change to the game): the picture test keeps each program's text as the card's
  compiler is given it (`FAR_TEST_SHADER_TEXT`, a folder; made for the game's own compiler), and
  `kept logs/march/march_parts.py` compiles that text as the game does and again with debug marks, checks the two
  are the same program instruction for instruction, and counts every instruction in the part of the text it came
  from (and, for a function drawn into another, where it was called from). Build 194's texts and trees are kept
  in `kept logs/march/b194/`.

**What the scan found** (the whole of it is in the document):

- `march_all` is 4,548 instructions (4,436 with cover by faces off). The march's own steps are 56% of it, the
  finish 10%, the joins 16%.
- **A third of the program is finding where a read lands in the bricks' textures,** three quarters of that its
  edge cases; the hand-over's second read is a quarter of the program; the gather, where the picture is made, a
  twentieth. Fog is in the program though it's off (84 instructions), and all four ray-start patterns (92).
- **The march carries 98 values round its loop** for every ray (97 with cover by faces off: the one more is the
  register David's rows showed, 167 to 168, so this count follows the registers): the numbers of two levels kept
  in hand 30, the height table's three rows 15, the last step's look 13, the floor's cells 9, what's gathered 7,
  and the rest; 2 are a guard naga adds to every loop.
- **Eleven candidates** for making room, ranked, each marked measured or not: first the loop guard (wgpu lets a
  trusted shader be made without it), fog compiled in only when on, one ray-start pattern; then the two levels'
  numbers and the three rows kept in hand (45 of the 98, unmeasured); the edge cases of placing a read; the
  hand-over's second read (14 registers, measured, and a look); the last step's look; the finish in its own pass;
  the floor's cells; the sky worked out at the finish.

**A check on the instrument:** the program with debug marks is the game's, instruction for instruction (4,548 of
4,548 in order); cover by faces reads as 112 instructions and one held value, as David's rows read one register.
What it can't say: a part's time, or its registers (only a trace and his reading of the row do).

**Something noticed on the way, written down, not acted on:** the picture test's own device leaves the choice
of DirectX compiler to wgpu (DXC if it finds one, else the old FXC), where the game names the Windows SDK's DXC
(`card.rs`); the test now names the game's only when it's asked to keep the text. Which compiler the pictures
checked off the screen have been made by isn't known. No difference has ever shown against the game; it isn't
known that there's none.

### The far eye left as it stands, and on to the near eye (2026-11-16, David's word)

Asked when the Intel card's flight should be flown (it needs the laptop switched to "Optimus" and restarted),
**David:** "we can do that later. as long as we're set up to look back into the graphics engine at a later date,
we can proceed."

So the far eye's part of the plan (2.2) is left as it stands, its gate's last line (the Intel card's numbers)
owed, and the work goes on to the near eye (2.3). What "set up to look back" rests on is listed in one place in
`master/plan.md` ("For the later look back at the far eye"): lazy rays and cover by height, each packed in its
own document; the march by its parts, with its candidates; the Intel flight and the checked and unchecked pricing
owed with it; his weights by kind; the long rays' list; the small things written down and not mended.

**Where the far eye stands as it's left** (build 194, the game as launched, cover by faces on; the card's whole,
ms, from its flight): the mountain top 1.06, high over the spawn 1.12, fast across 0.76, the run 0.74, the sea
0.66, the sea nodding 0.65, the plain nodding 0.58, the plain 0.57, the spawn 0.48. Everything under 2 ms, his
bar of 2026-11-13; two views over 1. Its march's size on the card: 168 registers, nothing put by, 12 groups.

### The near eye: David chooses triangles (2026-11-16), after four questions about how near and far would sit together

Before any reading or code for 2.3, David asked, and Builder answered (none of it built or measured; said so each
time):

- *"how would the near rays pack with the far rays in the march? do we have room in the march, or would it run
  concurrently, ideally as a quite short rays?"* Not in the march: a program of its own, with its own size on the
  card. The near pass first, short rays ended at the near field's edge; it writes what each pixel hit and how much
  it covered; the far march then runs only where it's still needed and starts at the near field's edge. One after
  the other, not side by side (two programs can't share a dispatch).
- *"how do you think it will compare with triangles?"* Builder's lean, low to medium: triangles faster and steadier
  for whole blocks (a triangle can't wait on a long ray; skimming rays at eye height are the risk), rays the better
  tool for the pieces inside a block; the picture the same either way.
- *"is there any way that triangles help the far march at all? will screen triangles cancel screen rays since they
  occlude?"* Yes: drawn first, their depth says which of the far eye's squares are covered; covered squares march
  nothing, part-covered ones only their finer rays, and every far ray starts at the near field's edge. A lot at
  ground level (flat ground 128 blocks off is under a degree below the horizon), in towns, woods and indoors;
  little from high up, which is where the two dearest views are.
- *"so in a cave, the march would be totally silent"* As near as makes no difference, closed all round: what's left
  is asking (about 0.03 ms of groups that do nothing, or nothing if the far pass isn't launched) and bringing the
  picture up. Not along a sightline longer than the near field, through a cave's mouth, or behind water or glass.

**Then his decision:** "I believe we should try to accomplish the rest of the graphics engine with triangles.
rays may come in later for different cases, but i don't believe quarter/oct blocks will present any difficulty in
terms of triangle count. it will still have less polygons than half life 1. i'd like to hear how you think we
could gracefully add the triangles in, and make sure they cooperate nicely with the march."

**What it changes:** the plan's rule for 2.3 was his "Yes, fly both" of 2026-11-12 (rays first, then triangles,
both flown on the same marks before the fork). By today's word the near eye is built with triangles, quarter
blocks and pieces too; near rays aren't built now and may come later for particular cases. The fork's comparison
of two near drawers falls away with it (Counsel's and the Oracle's to weigh).

**One thing in it put right to him, the conclusion standing:** a block world's near field is many more polygons
than Half-Life's scenes were (those ran to hundreds or a few thousand; a flat square of 256 blocks is 65,000 faces
before any merging). It's still no load on this card, and the triangle hardware is what a small card is best at.
What to watch with pieces is the meshing and the overdraw, not the count.

### The near eye's goal, in David's words (2026-11-16): a 64-chunk render radius, running smoothly with the far march

Builder had proposed how triangles would come in (a mesher on a lane; drawn first, their depth cancelling far
rays; one set of colours and one light for both drawers at first; an overlap at the near field's edge, not a line;
a step a build) and said she'd start the near field at about 128 blocks. **David:** "in old incus artus, i was
playing with a 64 chunk render radius, which i believe would be 2000 blocks. the overlap can be pretty deep. that
may be excessive render radius but i don't think so for a whole town, so my goal would be 64 chunk render radius
running smoothly along with the far march like we have it now, which is super quick."

**Checked against the record:**
- Old's chunks were 32 blocks across (`old/src/world/position.rs`), so 64 chunks is 2,048 blocks: his figure.
- What old's Render page did with it, by `journal/docs/far-view.md` (builds 27 and 42, early November): the
  render distance was how far real chunks were kept, and a second number, "the march starts at (chunks)", was how
  far they were *drawn*, the far view taking over past it. His preset then is written there as render distance
  64, the line at 8 (256 blocks), fog off. So with that preset real blocks were drawn to 256 blocks and kept to
  2,048. Whether he also played with the line out at 64 the record doesn't say. Told to him as the record's, with
  its date; his goal now is what counts.

**What the goal means in size** (arithmetic, nothing measured): a circle of 2,048 blocks is 13.2 million columns,
about 16,600 chunks. One top face a column is 26 million triangles before any merging, more with the sides of
slopes. A block 1,000 blocks off is about 0.9 of a screen pixel across (2,560 x 1,600, 100 degrees up and down),
at 2,000 under half a pixel. So true blocks all the way out are tens of millions of triangles a frame, most of
them smaller than a pixel: the count has to come down by merging faces, by drawing only what's in view, and, for
plain ground far out, by coarser cells (which is what the far eye's cells already are). And the far march, begun
at the radius' edge, loses most of its work: by today's measurement at most 43% of the mountain top's work and
28% of high over the spawn's lies beyond 2 km.

### The near eye begins with one radius slider (2026-11-16, David's word)

Builder had put a choice to him (every block true out to 2,048, or built things true all the way with plain
ground stepping down in detail). **David:** "well lets just make a radius slider and get started. the render
radius set there will determine where the triangles start and rays fade. if that becomes intensive we can add a
slider that seperates built things from the land render so they can be adjusted seperately until it hits a nice
balance with the march. and, for combining faces, i thought that was called greedy meshing and kind of an
automatic part of any block game triangle mesh geometry".

So: one render radius, a slider, sets where the triangles end and the far rays fade in; the choice between A and
B isn't made now (a second slider, built things apart from the land, only if the first gets heavy: written down,
not built); faces are merged from the first build (greedy meshing: his name for it is the usual one; it's the
mesher's work, not automatic, and it's built in from the start). Asked of him, by his own rule for controls:
the Render page, "Render radius (chunks)", 1 to 64, unless he wants another place or name.

### Build 195, the near eye's first step, whole blocks as triangles inside a render radius: the pass mark (written before the code)

**What it's for** (plan 2.3, by David's words of 2026-11-16: the near eye by triangles; "lets just make a radius
slider and get started. the render radius set there will determine where the triangles start and rays fade"; his
goal, "64 chunk render radius running smoothly along with the far march like we have it now").

**What's to be built.** Nothing in the far march's program changes.

- **The near blocks, a chunk at a time** (`near.rs`, new): a chunk is 32 x 32 columns, the world's whole height
  (old's chunk, so his 64 is 2,048 blocks). Its blocks are the doorway's answers (each column's runs; the scatter
  stage's columns and changed cells as the doorway gives them).
- **The mesher** (the same file): a chunk's faces are every side of a solid block that looks into air or water,
  merged: faces of one block that look the same way, lie in one plane and make a rectangle become one face
  (greedy meshing). Water's top is a face of its own kind (drawn plain for now: its look is the plan's 2.4).
  Pieces (bars and quarter blocks) aren't meshed in this step (step 4): a cell that holds only pieces draws
  nothing yet, said here so it isn't taken for a fault.
- **A lane** that meshes the chunks inside the radius on background threads, nearest the eye first, hands each to
  the card as it's done, drops those that fall outside, and makes a chunk again when a change touches it.
- **The near pass** (`near.wgsl`, new, a program of its own): the chunks' faces drawn as triangles after the far
  picture is brought up, with depth; each face its block's colour (the far colours' table, top or side) lit as the
  far eye lights (the sun on the face's own way, the far eye's shadow and sky from its light store). Over the last
  stretch before the radius the faces fade out onto the far picture behind them.
- **The far rays begin a little inside the radius** while the near eye is on (the overlap), so there's no gap
  between the two. (The far eye isn't yet told which of its squares the near faces cover: step 2.)
- **The Render page: "Render radius (chunks)"**, 1 to 64, asked of David for its place and name. It starts at 4
  (128 blocks). `INCUS_NEAR=0`: no near eye at all, the game as build 194.
- **Its instruments, from this first build:** the lane's and the near pass's clocks in the panel and the flights'
  reports; chunks kept, faces and triangles drawn, the meshes' memory on the card, a chunk's meshing time.

**The pass mark.** Off the screen:

- **The mesher is exact.** Over squares of bare ground, the stage's built things (a tower, the castle's gate, a
  house, the hall, a cave, a wood) and a square at the map's edge: the merged faces cover every open side of
  every solid block once and nothing else, counted block by block from the doorway's own answers (areas by
  facing and block equal; no two faces overlap). A change in one cell makes again only the chunks it touches, and
  the result is the mesh made from scratch.
- **Off, it's build 194:** with `INCUS_NEAR=0` the five marks' pictures are build 194's but for the test's noise.
- **On, the picture:** at the near marks and the stage's, the near faces' colours and light agree with the far
  eye's picture of the same ground (the same view drawn with the radius at 1 and at 8: mean colour of the ground
  between 64 and 256 blocks within 8 of 255); no pixel of sky or of the clear colour where ground should be, at
  the radius' edge or anywhere (the overlap leaves no gap), at radius 1, 4, 16 and 64.
- **The numbers, given not promised,** at radius 1, 4, 16, 32 and 64 from the spawn and from the plain: chunks,
  faces, triangles, the meshes' memory, how long the lane takes to fill the radius, and how many faces merging
  saves against one face a side.
- All the tests pass; the map redrawn.

Its size on the card: the near pass's own row in a trace, David's to read (asked with the flights).

By flights (on David's word, a bracket: near off, on at radius 4, 16 and 64, off again), at the near marks and
the far ones: off is build 194's to within the bracket's own spread; the near pass's time and the far eye's beside
it at each radius, as they are. No line is set for the time yet: this build is the first measurement, and the
plan's line (1.5 ms or under at every ground mark) is what later steps are held to.

**Then David's look:** the radius' edge standing and walking; chunks coming in as he moves; the slider.
A miss is said, with what was seen.

### Build 195, its first piece: the chunk's blocks and the mesher, checked block by block (2026-11-16; no build number yet, nothing drawn)

`src/near.rs`, new: a chunk's blocks read through the doorway (32 x 32 columns and one column of its neighbours
all round, from under the lowest cell that isn't solid to over the highest that's anything; whole blocks placed
since are in; bars aren't yet), and its faces, merged (for each facing and each plane across it, the open sides of
one block that make a rectangle become one face). A block's side is open where the cell it looks into is air or
water; the water's top is a face of its own kind. Nothing uses it yet but its tests: the lane and the pass are the
rest of the build.

**The mark's first line, "the mesher is exact": met,** three tests, each counting every open side of every block
one at a time from the chunk's cells (and the cells against the doorway's own columns) and holding the merged
faces to cover each side once and nothing else:

| where | chunks | open sides | merged faces | sides a face |
|---|---|---|---|---|
| bare ground round the spawn | 9 | 11,942 | 2,802 | 4.3 |
| round the mountain top's mark | 9 | 37,789 | 21,337 | 1.8 |
| round the plain's mark | 9 | 9,796 | 662 | 14.8 |
| the map's corner (columns past the map are air: the world's edge shows its side) | 9 | 16,768 | 16 | 1,048 |
| the scatter stage's places (every kind; every fifth cave; a wood) | 344 | 1,044,002 | 418,695 | 2.5 |

A block placed in the air at a chunk's edge: six more faces in its own chunk, the chunk across the edge and the
seven others round it unchanged, and each still exact.

**What the counts already say, for the goal of 64 chunks** (16,600 chunks; arithmetic on these few chunks, to be
measured properly when the lane fills a radius): merging helps most where the ground is flat (74 faces a chunk on
the plain) and least where it's steep (2,370 a chunk at the mountain top; the stage's chunks 1,217). At the
mountain top's rate the full radius would be some 40 million faces; at the plain's, about 1.2 million. Real
radii lie between, and the numbers will be read at 1, 4, 16, 32 and 64 as the mark says.

All 37 tests pass (the three new ones in; the map redrawn for the new file).

### Build 195, its second piece: the near lane, and what each render radius comes to (2026-11-16; still nothing drawn)

David on the slider's place and name (the Render page, "Render radius (chunks)", 1 to 64): "yes thats fine".

**The lane** (`near.rs`): threads of its own, each with a doorway of its own, making chunks' faces as they're
asked for, nearest the eye first; asking again replaces what's waiting, so a change of place or radius is heard at
the next chunk. A face is handed over as three words (its place, facing and size; its block; its chunk): 12
bytes. A chunk is wanted if any part of it is within the radius, so the whole circle is covered. Checked: a
radius is filled and each chunk is the mesh made directly; a packed face unpacks to what it was.

**The mark's numbers, "given not promised", for faces, memory and filling** (the test world, off the screen, 23
threads; a triangle count is two a face, before anything is left out for being out of view or turned away):

| from | radius | chunks | faces | triangles | faces' memory | filled in |
|---|---|---|---|---|---|---|
| the spawn | 1 (32 blocks) | 8 | 2,509 | 5,018 | | 0.01 s |
| | 4 (128) | 64 | 39,810 | 79,620 | 0.5 MB | 0.02 s |
| | 16 (512) | 860 | 1,180,365 | 2.4 million | 14 MB | 0.20 s |
| | 32 (1,024) | 3,342 | 5,425,611 | 10.9 million | 62 MB | 0.85 s |
| | 64 (2,048) | 13,114 | 14,399,227 | 28.8 million | 165 MB | 3.4 s |
| the plain | 4 | 69 | 5,771 | 11,542 | 0.1 MB | 0.03 s |
| | 16 | 871 | 54,514 | 109,028 | 0.6 MB | 0.12 s |
| | 32 | 3,347 | 143,932 | 287,864 | 1.6 MB | 0.76 s |
| | 64 | 13,123 | 1,358,554 | 2.7 million | 16 MB | 3.0 s |
| the mountain top | 4 | 67 | 192,628 | 385,256 | 2.2 MB | 0.04 s |
| | 16 | 870 | 1,777,317 | 3.6 million | 20 MB | 0.22 s |
| | 32 | 3,342 | 4,771,992 | 9.5 million | 55 MB | 0.81 s |
| | 64 | 13,125 | 13,592,563 | 27.2 million | 156 MB | 3.5 s |
| the stage's castle, the stage on | 4 | 69 | 10,373 | 20,746 | 0.1 MB | 0.15 s |
| | 16 | 877 | 326,665 | 653,330 | 3.7 MB | 0.84 s |
| | 64 | 13,147 | 3,508,614 | 7.0 million | 40 MB | 34.7 s |

- **His 64 chunks are 13,100 chunks,** not the 16,600 Builder's arithmetic said (that was a square's count; a
  circle of 64 is 13,100).
- **The lane isn't the limit:** the whole of 64 chunks is made in about three and a half seconds, 5 to 6 ms of
  one thread a chunk. (With the scatter stage on it's ten times slower, 35 s: each thread applies the stage to
  every tile it reads, and reads them again as it moves on. The stage is a test's fixture, not the game's world;
  written down.)
- **The memory isn't the limit:** 165 MB of faces at the most, where the card has gigabytes to spare.
- **The triangles are the question,** as expected: on hilly ground the full radius is 27 to 29 million triangles
  before anything is left out; on the plain 2.7 million; with a castle, a town and woods on the plain 7 million.
  How many of those the card draws in what time is the next piece's to measure. What's known to cut them without
  changing the picture: chunks out of view left out, and in each chunk the faces turned away from the eye left
  out (about half).

All the near eye's tests pass (5).

### Build 195 built (unlaunched, unflown, unlooked): the near eye draws; what was built and what was checked off the screen

**Built as the mark says,** with the differences said:

- **The near eye on the card** (`near_eye.rs`, new): the chunks' faces kept in pages of one buffer each (a
  million faces, 12 MB, a page), a chunk's faces in one stretch of a page, given back when it falls outside the
  radius by more than two chunks. Each frame the lane is asked for what the eye's chunk and the radius want (when
  either changes), what it has made is put on the card nearest first, 350,000 faces a frame at the most, and
  **the faces' reach** is worked out: to the nearest chunk still to come, or the whole radius. The faces fade out
  over the last eighth of their reach (8 to 48 blocks), and **the far rays begin 24 blocks inside that**
  (`FarEye::near_from`, taken with the measuring switch's own number in the camera's row): so while a radius
  fills, or the eye outruns the lane, the far eye draws whatever the near eye hasn't got yet and there's no hole.
- **The near pass** (`near.wgsl`, new, two programs of its own): a face is four corners made from its three
  words; its place is taken from the eye in whole blocks first; its place on the screen from the far eye's own
  camera (the same forward, right and up the rays are made from, so the two pictures lie on each other). Its
  colour is its block's from the far colours' table (top or side), lit by the far eye's own text (`lit`,
  `sunlit`, the light store's reading: put in from `far.wgsl` when the program is built, one text for both eyes).
  Drawn after the far picture is brought up, with depth. **The water** is a second pass, which the mark had as
  "drawn plain": it's the far eye's water instead (its colour and how fast it dims what's behind, by how much
  water the eye looks through to the face behind, read from the near faces' depth), so the sea doesn't change
  kind at the radius. A chunk's water's tops are kept after its blocks' sides, so each pass draws its own.
- **Only the chunks in view are drawn** (a chunk's box against the view's four sides), each page's by one call
  that reads its draws from a list. (The card is asked for one thing more for that: a draw from a list may begin
  at any face of a page.) Faces turned away from the eye are still drawn: not left out yet.
- **The Render page:** the near eye's time on the card, chunks in and drawn, triangles drawn and kept, the pages'
  memory, how far its faces reach and where the far rays begin, a chunk's meshing time; and the slider, "Render
  radius (chunks)", 1 to 64 (David: "yes thats fine"), greyed where there's no near eye.
- **The wiring:** four passes ("putting the near chunks on the card", "the near faces", "the near water" in the
  frame; "meshing the near chunks" on the loading lane), a store ("the near faces"), two switches (`INCUS_NEAR=0`:
  no near eye; `INCUS_NEAR_RADIUS`: the radius it starts with). The map redrawn.
- **Not built yet, of the mark's list:** the near eye's counts in a flight's report (the panel has them); a
  chunk made again when a change touches it while the game runs (the mesher and the test for it are there; the
  lane isn't yet told of changes).

**Off the screen** (the picture test with `FAR_TEST_NEAR=<radius>`: the spawn, 2,560 x 1,600, the regions round
it loaded; no error from the card at any radius):

| radius | chunks in | chunks drawn | faces drawn (triangles) | faces kept | pages | filled in |
|---|---|---|---|---|---|---|
| 1 | 8 | 5 | 1,610 (3,220) | 2,509 | 12 MB | 0.01 s |
| 4 | 64 | 28 | 16,554 (33,108) | 39,810 | 12 MB | 0.01 s |
| 16 | 860 | 320 | 313,121 (626,242) | 1,180,365 | 24 MB | 0.20 s |
| 64 | 13,114 | 4,631 | 5,154,126 (10.3 million) | 14,399,227 | 168 MB | 4.9 s |

- **The picture:** crisp blocks, steps and terraces where the far eye alone draws smooth blurred ground; the far
  hill behind as before at radius 4, and itself in true blocks with a sharp edge against the sky at radius 64.
- **Colour and light agree with the far eye's** (the mark's line, within 8 of 255): the ground at the eye's feet
  78, 76, 63 for the far eye's 83, 80, 68 (the near picture has the steps' dark sides, which the far cells
  average in); a little out 83, 80, 67 for 81, 79, 67; the hill behind and the sky the same.
- **No gap between near and far** at radius 1, 4, 16 or 64: no black or clear pixel anywhere. Counted, 57,000 to
  86,000 pixels are sky with the near eye where the far eye alone has something; looked at, they're the far eye's
  soft haze over near slopes (a band many pixels deep beside the eye), which crisp blocks don't have. So the
  mark's count ("no pixel of sky where ground should be") can't be read off that number, and is judged by the
  pictures: met, as far as Builder's eye goes.
- **Seen and written down, for David's eye:** at radius 64 the far hill's blocks are smaller than a pixel and its
  steps' dark sides make a fine speckle on it. That's the shimmer expected of faces under a pixel.

**Not checked yet, of the mark's lines:** off against build 194's pictures with the whole world loaded (off is
the game without a near eye, and nothing of the far eye's program changed; the far eye's own tests pass); the
stage's views; the near marks; the plain. All 39 tests pass.

**Its first run in the game itself is the launch for David's look:** everything above is the picture test's.
The panel's slider and readouts haven't been seen by anyone.

**Owed:** David's look (the radius' edge standing and walking, chunks coming in, the slider); then its row in a
trace and its flights (a bracket, near off and on at radius 4, 16 and 64); the rest of the mark's checks.

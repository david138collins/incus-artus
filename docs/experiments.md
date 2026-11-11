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

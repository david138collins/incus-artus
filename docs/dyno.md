# The dyno: the speed checklist

Every idea queued for making sight fast, in one place. Builder ticks each one off as she goes and writes its
measured result beside it. Started 2026-11-09 at David's asking ("i really hope we dont forget any of all the good
ideas we have had for going from zero to sixty").

The why and David's words for each idea are in `docs/decisions.md`, "Look first, then the dyno". The baseline and
build-by-build numbers are in `docs/regroup.md`.

**How to use it:**
- `[ ]` not started, `[~]` tried or in progress, `[x]` kept, `[-]` tried and dropped (say why).
- Write the build number and the measured change beside each item, for example "build 101: sight 12.1 → 9.4 ms on
  the wide flight, baked, defaults".
- **Block** items must leave the picture unchanged: an A/B picture test on the card, the same frame before and
  after, equal to the pixel or within a stated tolerance.
- **Induction** items trade something for speed, so they get David's eye instead.
- One step a build, each measured alone.

**Where we start** (the baseline after build 92, the low slow flight, uncapped):
- **Defaults, baked:** sight 12 to 16.6 ms, 56 to 75 fps.
- **David's lean preset (8 × 8 and 4 × 4, no shadows), baked:** sight 3.4 to 4.7 ms, 197 to 262 fps.
- About 8 ms of the gap is shadows, sky light and the 2 × 2 layer. Builds 93 and 94 added 2 to 3 ms for far
  ground drawn as ground.

## The plan (2026-11-09, at build 101)

David: "prepare an order of all the steps you're going to try, how you're going to evaluate them, which ones could
have compounding effects, positive or negative on one another, our baseline, anything else important."

### The pipeline we have

**The world's data.** Near the player the processor makes chunks of blocks. The far world is made on the card from
the map's recipe and kept in a **cell store**: cells are the world averaged into cubes, each 16 bytes (how solid, how
much water or other see-through medium, which ways its faces look, its colour, its heights' spread). Two kinds of
level:
- **Live levels** round the eye (cells of 1, 2, 4, 8 and 16 blocks), refilled as you move. Level 1 is the blocks
  themselves.
- **Baked levels** (32 and up) for the whole map, made a region at a time: **the bake**, which shares the card with
  the picture (build 84 gave it about half the card while it runs).

Edits are patched into both. Placed lights are flooded on the processor, and a table of lit cells (and since 101 the
wash) goes to the card.

**Each frame, on the card:**
1. **Start pass:** one cone per 8 × 8 tile of the screen finds where its rays can safely start.
2. **The pyramid:** the march runs layer by layer, coarsest first (a ray per 8 × 8 pixels, then 4 × 4, then 2 × 2 by
   default; 1.5 and 1 × 1 optional). Each finer layer marches only where the layer above says it's worth it (near,
   at edges); elsewhere the coarser picture stands. Each ray has two parts:
   - **The near walk** through real blocks out to where a block is a pixel: hard faces, textures, pieces and bars,
     water and other media, and placed light, stepped.
   - **The far march** through the blended cells: strides through air, skips over column tops, fine steps into
     surfaces, water dimming, and the wash.
3. **Per pixel, after its ray:** the sun's shadow (a cone toward the sun), sky light (five cones, or the parent
   pixel's when it's the same kind of surface), then fog.
4. **Far glows:** one line of sight per far light.
5. **Upscale:** the layers composited into the screen, writing a depth.
6. **Objects:** models, flame sprites and far glows drawn into that depth; then the panel.

**Each frame, on the processor:** the game's update, the objects gathered, lights flooded when something changes,
the bake fed, edits patched. Work is kept even from frame to frame. David: "the work we've done so that stuff happens
predictably every duty cycle makes the lag and chop feel more like a problem that can be worked out through clever
gpu origami than a doleful warning".

### Where we start

The last full baseline was after build 92 (above). Since then:
- builds 93 and 94 added 2 to 3 ms (far ground drawn as ground);
- build 98's water added about 1.5 ms;
- placed light (95) and the glows (100) were within noise;
- the wash (101) isn't measured yet.

So the dyno's **first job is a new baseline at build 101**, on the new fixed flight, for each of:
- defaults, baked;
- defaults, baking;
- David's lean preset, baked.

**What's measured each time:**
- the card's own time for sight (the timer on the passes), split by pass once the instruments are in;
- frames per second;
- the spread of frame times (median, 95th and 99th percentile), since even choppiness beats irregular;
- the mouse's input-to-photon time;
- the bake's rate, on the baking runs.

### The baseline at build 101 (2026-11-09)

The dyno's flight on `nice25 test`, David's settings (`presets/*/dyno.toml`), his screen's own 2,560 × 1,600,
uncapped, two laps a flight; Edge (music), Aseprite, Discord and Steam running beside it. Frame times in ms (the
median, then the 95th and 99th percentiles); "card" is sight on the card (the median, then the 95th).

**Baked** (two flights, `logs/dyno-1794249218.txt` and `dyno-1794249400.txt`):

| | Frame median | 95th | 99th | Card median | Card 95th |
|---|---|---|---|---|---|
| **Whole flight** | **5.51 / 5.59 (182 / 179 fps)** | 16.28 / 16.29 | 21.31 / 21.26 | 4.81 / 4.85 | 14.78 / 14.79 |
| Over the land | 11.66 / 11.57 (86 fps) | 16.7 | 18.4 | 10.49 / 10.42 | 14.7 |
| Round the peak | 3.81 / 3.85 (261 fps) | 18.1 / 18.3 | 22.9 / 23.1 | 1.51 / 1.53 | 16.7 |
| Out to sea | 9.06 / 9.09 (110 fps) | 16.9 | 22.2 / 21.5 | 8.24 / 8.24 | 15.3 |
| Quick | 3.33 / 3.56 (290 fps) | 8.9 | 12.2 | 1.22 / 1.25 | 7.5 |
| Home | 7.28 / 7.48 (135 fps) | 12.4 | 15.4 | 6.11 / 6.15 | 10.9 |

- **Flight to flight:** within 1 to 3% (the quick leg 7%). The dyno is steady enough to see a few percent.
- **Where the card's time goes:** over the land and the sea, the coarse layers (8 × 8 and 4 × 4) take about 2.6 to
  2.8 ms each, and over the land the 2 × 2 layer about 4.
- **The processor's floor:** where the card is light (round the peak, quick), the frame is still about 2 ms more
  than the card's work: about 2 ms a frame on the processor's side.
- **Uneven:** even the cheap legs have a 95th percentile of 16 to 18 ms.

**Baking** (a fresh bake of nice25, two laps from the start, `logs/dyno-1794249904.txt`; the start's 729 regions
done in 119 s, at the end of the flight):

| | Frame median | 95th | 99th | Card median | Card 95th |
|---|---|---|---|---|---|
| **Whole flight** | **10.00 (100 fps)** | 29.07 | 33.41 | 4.52 | 15.76 |
| Over the land | 20.47 (49 fps) | 29.7 | 33.5 | 10.29 | 17.0 |
| Round the peak | 7.92 (126 fps) | 33.3 | 33.7 | 2.95 | 21.0 |
| Out to sea | 17.84 (56 fps) | 28.9 | 32.1 | 8.73 | 15.9 |
| Quick | 5.79 (173 fps) | 16.4 | 24.8 | 1.12 | 7.9 |
| Home | 10.83 (92 fps) | 15.7 | 16.5 | 5.18 | 9.6 |

- **The creaking, measured:** while the bake runs, the frame takes about twice sight's own work on the card: the
  bake's card work is the rest (it has about half the card while it runs, build 84). The frame rate halves, and
  the 99th percentile goes from 21 to 33 ms.
- **A saved world** loads with its bake whole (about 25 s from launch to drawn, the bake on the card in moments),
  so playing one is the baked case: about 180 fps, not 100.

**Heat:** the GPU 69 to 76 °C at 2.7 to 2.8 GHz all through (not slowed by its heat); the CPU area's reading
swings between about 70 and 104 °C.

**Against the mark** (David's tiers, baked): the median at 180 fps against the aim of 237 (a third faster wanted);
the 99th percentile at 21 ms (47 fps) against "fine" at about 180 fps (5.5 ms: four times); the floor, a stable
120, not met at the 99th either. The worst frames are the gap.

### How each step is judged

- **One step a build, each measured alone,** on the same fixed flight, before and after. The runs alternate
  (before, after, before, after), so the laptop's heat drifting doesn't decide it.
- **Three verdicts** [Counsel's review: small lossless wins add up]:
  - **Kept:** the gain is clear of the noise (both pairs agree, at least about 5% or 0.5 ms).
  - **Below noise, kept:** lossless (the picture test passes), a gain in both pairs but under the bar, no worse at
    the 99th percentile, and not enough added complexity to worry about. Several of these add up.
  - **Tried and dropped:** no gain, a loss, or a gain not worth what it adds. A fine result.

  A step landing near the bar gets more paired runs before its verdict. Written down with its numbers either way.
- **Block steps** must leave the picture unchanged: an A/B picture test on the card, the same frames before and after,
  equal to the pixel or within a stated tolerance.
- **Induction steps** trade something, so they get David's eye: each behind a switch on the Render page, so he can
  flick it on and off himself.
- **Every kept step keeps its off switch** (an `INCUS_SIGHT_NO_…` for measuring), so a later step can be measured
  with it and without it, and backed out if two steps turn out to clash.
- **Nothing per-card:** portable only, as decided (no vendor tricks; never assume 32 lanes in a warp).
- **Even beats fast:** a step that raises the average but makes frames uneven (a worse 99th percentile) isn't kept
  as it is.
- **Each step's result** goes in this checklist (`[x]` or `[-]`, with the build number and its numbers) and in
  `regroup.md`, and is reported to David plainly: wins and drops alike.

### The order

Cheapest and most certain first, measuring before guessing. The order after the instruments is a first guess: what
the instruments show re-ranks it, and any change is written here.

**A. Instruments, no change to the picture** (section 1):
1. The fixed wide flight: sea, mountains, a cave mouth, the horizon. Baked and baking measured separately, the
   percentiles kept.
2. A card timer on each pass (start, each layer, glows, upscale, objects), not one for all of sight.
3. The breakdown with the switches there are (shadows, sky light, fog, textures, lights and wash, the finer layers,
   the floor rule, water): what each costs today. That answers "builds 93 and 94's 2 to 3 ms".
4. The slowest-ray view: does a warp wait on its longest ray?
5. Nsight on the flight: memory stalls against divergence, occupancy, registers.
6. The subgroup check.
7. **The new baseline at 101, written down.**

**B. The block, cheapest and surest first** (section 2):
1. Rays end where fog is total (the reach is 150,000 blocks; fog is 99% by about 5,000). Expected the cheapest win.
   Stated tolerance: pixels already more than 99% fog.
2. Read-only bindings and small constants as uniforms. Trivial; may be small.
3. The sea as a flat plane: wins back water's 1.5 ms and the horizon's worst rays (610 steps).
4. Empty space skipped smarter: solved jumps, then relaxed cone stepping, for grazing rays over flat land; then an
   empty-space distance per cell in 3D, for caves and overhangs.
5. Where rays start: a two-level start pass (64 × 64 tiles, then 8 × 8), and **the rasteriser starting the rays**
   (boxes drawn round occupied bricks give each pixel its first possible hit) [David: "we need to look into ways to
   exploit texture units and rasterizers"]. Measured against each other: they do the same job.
6. Workgroup shape and Morton order.
7. Tiles by type: sky, far, near, water and seam tiles, each with its own lean shader.
8. Memory, if Nsight says the march waits on memory: hot and cold data apart, the store in Morton order, smaller
   cells, f16 colour sums; and **texture units** (cells as bricks in a 3D texture, blended by the hardware and read
   through its cache) [David, as above]. Texture units are tried even if memory isn't the wall: the hardware blend
   would take over the march's own eight-cell blend, a cost in arithmetic as well as in reads.
9. Horizon maps for far sun shadows and sky light (made at bake time, patched on edits). Potentially the biggest
   single block win, since shadows and sky light are most of the 8 ms gap to the lean preset. Bigger work, so after
   the cheap ones. Then, or instead, **light the hologram**: sun and sky light kept in the cells, re-lit gradually as
   the sun moves (the same cost taken out of the frame, a different way; measured against horizon maps).
10. Warps kept busy (rays sorted by expected length, compaction, then David's ray queue): only if the slowest-ray
    view shows warps waiting.
11. Textures' last 1 ms; placed light under many lights; the bake in the card's idle gaps.

**C. The processor as a partner** (section 4), slotted in when the instruments point there: the objects' frame work
kept between frames, the lights' flood redone only round what changed, one-way handoffs, camera prediction for the
bake, the turn sent ahead, late latching. **The heat watch** from the first flight on: the processor's and the card's
clocks and temperatures logged on every flight, so a step that only moves heat around shows as that.

**D. Induction, by David's eye** (section 3), after the block's cheap and sure steps:
1. Rays start just short of last frame's hits.
2. Temporal reuse (TRAA, David's rules).
3. Far pixels re-marched only once they've moved half a pixel.
4. Fixed budgets each frame.
5. The far panorama for turning, and the checkerboarded coarse layer.

**E. The first impression** (section 5): the bake's share of the card, and the bake hidden in the opening.

### What compounds, for better or worse

- **Gains that overlap, not add up:** rays ending in the fog (B1), the sea plane (B3) and smarter skipping (B4) all
  cut the same long horizon rays. Each is measured after the ones before it, so the first one in looks best. The
  order is written down, so the numbers are read with that in mind.
- **Gains that multiply:**
  - Tiles by type (B7) make sky tiles nearly free, then skipping makes far tiles cheap.
  - Memory layout (B8) speeds every reader of the cells: the march, the shadow and sky cones, the glows.
  - Temporal reuse (D2) divides the work of everything under it.
- **Gains that cancel:**
  - Sorting rays, compaction and tiles by type all fight the same thing: warps waiting on their slowest ray. After
    one, the others gain less. The slowest-ray view decides which, if any.
  - Horizon maps (B9) replace the sky cones and shadow rays that temporal reuse (D2) would otherwise smooth, so D2's
    gain shrinks after B9, and B9 lowers how many rays D2 needs.
- **Hidden costs that touch everything:**
  - **Registers.** `march.wgsl` is one big shader. Any step that adds code to it can raise its registers, lower how
    many rays the card runs at once, and slow every other part. Registers are checked after each step that touches
    it (Nsight).
  - **The laptop's shared heat.** The processor and the card share one heat budget, so processor savings can show
    up as card gains, and a long run warms up and slows down. Hence the alternating runs.
  - **Even frames:** temporal reuse and fixed budgets can make frames uneven if not budgeted. They go together.
- **The bake against the picture:** they share the card. A faster march leaves the bake more room; the baking runs
  are tracked separately, so a win on one isn't a loss on the other unnoticed.

### The save point

- **Git tag `before-dyno`** at the commit that adds this plan: every file as it stands at build 101.
- **A backup** on David's USB disk: `D:\Incus Artus Backups\Incus Artus 2026-11-09 before dyno`.
- **Each dyno step is its own build and commit,** so a step that goes wrong is undone alone (reverting its commit).
  If everything goes sideways, `git checkout before-dyno` brings the whole engine back as it is today, and the USB
  copy is there if the disk itself fails.

### For David

(Counsel is asking him these, with whether the baking case gets its own target and its items move earlier: his felt
complaint, "creaking when you move too quick", is the baking case. The order here moves when he answers.)

- **Which number is the target?** I'd proposed: defaults, baked, on the fixed flight, holding 60 fps at the 95th
  percentile, with the lean preset as the fallback.
  **David's answer** (2026-11-09): "well, i don't really think that way honestly. I'll be 100% satisfied if we can
  get it to a stable 237fps with current settings. but i don't automatically believe we can't go farther. there's
  no finish line for us, we just have to squeeze the blood from the stone until our hands fall off."
  - So the mark is **a stable 237 fps (about 4.2 ms a frame) with his current settings**, read in the tiers
    below. The settings are the ones he plays with today, written into the flight as a
    preset so the mark never moves.
  - From the baseline at build 92, that's about three to four times faster than defaults baked (12 to 16.6 ms).
    His lean preset was already 3.4 to 4.7 ms.
  - It's not a finish line: past 237, the dyno keeps going. Every step is still judged on its own numbers, not
    against the mark.
  - **Dips** [David, the same day: "its not a big deal if 237 dips to a reasonable number like 180 or something
    sometimes, because that's basically not noticeable, and even at 120 stable fps with slightly lower dips, its
    functional. i'm just saying we've got to give her all she's got"]. So the mark is read in three tiers:
    - **the aim:** 237 fps most of the time (the median);
    - **fine:** dips to about 180 now and then (the 99th percentile);
    - **functional:** a stable 120 with slightly lower dips. That's the floor, not the aim.
  - Still open: whether the baking case gets its own mark and its items move earlier.
- **The flights take over the screen** for two or three minutes each, so I'll ask before running them.

## 0. Done before the dyno

- [x] The baseline (after build 92).
- [x] The two flaws: the close-up disc and the circles from altitude (builds 93 and 94).
- [x] More storage slots asked from the card (build 95): David's card reports 262,144, and we use 9.
- [x] Nsight Graphics installed on David's machine (2026-11-09).
- [x] Far light, before the dyno starts [David 2026-11-09: "far light is important to be in there before we go onto
      the dyno"]: a far brazier's glow as a twinkling pixel or so, and its light on the ground averaged into the
      coarse cells. Details in `docs/regroup.md`, "Lights seen from farther". The glow: build 100. The wash: build 101 (its lookup's cost not measured yet: on the instruments' first flight).

## 1. Instruments: measure first

**Flying the dyno** (built 2026-11-09, `src/dyno.rs` for the route and the sums, `src/soak.rs` for the flight):

```
INCUS_DYNO=baked INCUS_LOAD_WORLD="nice25 test" INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno cargo run --release -j 4
```

- **The test world, `worlds/nice25 test`** [David 2026-11-09: "can you render nice25 as our test map, so when we add
  features for experiments and so forth its persistent?"]: his nice25 map rendered by the World Compiler with his
  compiler preset, saved with its whole bake of sight. `INCUS_LOAD_WORLD` starts the game straight into a saved world
  (any world: David can play one without rendering it again). Loaded, its bake reads back in moments, so the flight
  starts baked. What's placed in it for experiments stays.
- **Made, and made again, by** `INCUS_DYNO=baked INCUS_DYNO_SAVE_AS="nice25 test" INCUS_START_SCREEN=worldgen
  INCUS_SELFTEST_SUITE=compiler` with `compiler/dyno,worldgen/dyno` added to the presets: nice25 rendered afresh
  (it's the newest map in `world bakes/`), the route flown while it bakes (about a quarter of an hour: the whole map
  is 5,184 regions), saved once the bake is whole. Again whenever the cells' code changes what a cell holds
  (`SIGHT_RULES`, which every such step must bump) or the world's pack does: its bake no longer reads back.
- **The settings:** `presets/*/dyno.toml`, copies of David's defaults on 2026-11-09 ("current settings": his mark is
  read with these).
- **The route,** found on the map from the middle, the same every time: over the land to the highest ground within
  2,500, half round it, out to the nearest sea and 800 on (the horizon's long rays), quick back at 300 blocks a
  second, slowly up to the nearest cave mouth, home. About two to four minutes a lap.
- **`INCUS_DYNO=baked`:** the laps wait for the whole bake, then `INCUS_DYNO_LAPS` laps (1 if not said), measured
  baked and leg by leg.
- **`INCUS_DYNO=baking`:** nice25 rendered afresh (as above, without saving), `INCUS_DYNO_LAPS` laps from the start
  while it bakes: the "creaking" case.
- **What's kept:** each frame's time, and each pass's on the card (start, each layer, glows, upscale, objects; the
  card's own timestamps), as medians, 95th and 99th percentiles; the GPU's temperature and clock and the CPU area's
  temperature every 5 seconds. Written to `logs/dyno-<time>.txt` and the log; the game quits after.

- [x] A wider fixed flight that takes in sea, mountains and a cave mouth, measured baked and baking separately.
      Flown (the baseline above). No cave mouth yet: the test world has no caves (caves are test caves placed by
      hand); some placed in `nice25 test` would join the route (its bake made again once, saved again by itself).
- [x] A performance gate: every dyno build flies the same route, with its numbers kept in `regroup.md`.
- [x] A card timer on each pass (start, each layer, glows, upscale, objects), not one for all of sight.
- [x] The heat watch: the GPU's temperature and clock, the CPU area's temperature, on every flight (and the
      agreed cutoff: 105 °C on the CPU area twice running stops the flight).
- [ ] An input-to-photon gate: the mouse stays near build 67's 1.05 ms (uncapped).
- [ ] Nsight: card time per pass, stall reasons (memory against divergence), and occupancy.
- [ ] A slowest-ray view: each tile's slowest ray against its average, built from the steps view (`reach.z`).
- [ ] Register pressure in `march.wgsl` (one big shader).
- [ ] The subgroup check: does the adapter offer them? Keep a path that works without them.
- [ ] Break down builds 93 and 94's 2 to 3 ms: shadows, sky cones or the finer layers' extra pixels.

## 2. The block: the same picture, faster

**Skipping empty air**
- [ ] Rays end where fog is total (99% fog by about 5,000 blocks, against a reach of 150,000). Likely the cheapest
      win.
- [ ] Relaxed cone stepping: each column keeps the widest empty cone over its top, so grazing rays take long safe
      steps.
- [ ] An empty-space distance per cell: the same idea in 3D, for caves and overhangs.
- [ ] Solved jumps: a ray drops straight to the height of the tallest column top below it.
- [ ] The sea as a flat plane: its top found in one sum (the 610-step horizon case). Designed with generic
      transparency.
      Build 98's seeing through water added about 1.5 ms on the low flight (rays go on to the bottom).
- [ ] The rasteriser starts the rays: boxes drawn around occupied bricks give each pixel its first possible hit.
- [ ] A two-level start pass: 64 × 64 tiles, then 8 × 8.

**Light computed once, not every frame**
- [ ] Horizon maps: far sun shadows and sky light become lookups (made at bake time, patched on edits). Same picture
      within a stated tolerance.
- [ ] Light the hologram: sun and sky light kept in the cells, re-lit gradually as the sun moves.

**Memory**
- [ ] Hot and cold data apart: solid shares in their own tight buffer, colour and faces fetched only at a hit.
- [ ] Texture units: cells as bricks in a 3D texture, blended by hardware and read through its neighbour cache.
- [ ] The store's layout: smaller cells, and neighbours side by side in memory (Morton order).
- [ ] Read-only bindings where the march only reads, and small constants as uniforms.
- [ ] f16 colour sums to free registers, within a stated rounding tolerance.

**Keeping the card's workers busy**
- [ ] Tiles by type (sky, far, near, water, seam), each with its own lean shader. Today the finer layers launch over
      the whole screen.
- [ ] Warps kept alike: rays sorted by expected length, or compaction (march N steps, pack the ones still going,
      repeat).
- [ ] No idle lanes: persistent threads, a finished lane taking the next ray from a queue (dynamic ray fetch, Aila
      and Laine 2009). David's shape for it (2026-11-09): "a 'global ray buffer' that lets workers pull rays, and if
      a thread hits a block gets marked terminated, its value written to screen, and at the end of the step use
      wave intrinsics to squeeze out all the dead rays and give them fresh ones". Notes:
      - The warp refills in one grab: one atomic add on the queue for as many rays as it has dead lanes.
      - Rays are handed out in screen tiles (Morton order), so a refilled warp still reads nearby cells.
      - Never assume 32 lanes: AMD's waves are 64 and Intel's 8 to 32.
      - The march writes its hit and a separate pass shades it, so shading doesn't diverge too.
      - A path without subgroups is kept (workgroup memory and atomics).
      Measure first: the slowest-ray view tells whether warps really wait on their longest ray.
- [ ] Subgroups: warp votes for shared strides, the warp's nearest start.
- [ ] Workgroup shape and order: 8 × 8 against 16 × 4, Morton order.
- [ ] Background work, such as the bake, in the card's idle gaps between frames.
- [ ] Textures' last ~1 ms (build 87).
- [ ] Placed light under many lights (build 95): one table probe a face met near the eye (four with the blend
      across faces); check a lit cave full of braziers, and the table's fill (65,536 places, half used at most).

## 3. Induction: trades something, judged by David's eye

- [ ] Temporal reuse (TRAA), after the *INSIDE* talk: jitter (Halton 2,3), reprojection, neighbourhood clipping,
      about 90% feedback, motion blur for fast pixels (output only), noise soaked up so fewer sky cones and shadow
      samples are needed. David's rules:
      - movers never take from history;
      - pixels with no history get rays first;
      - edits and light changes drop history;
      - near hard faces stay crisp.
- [ ] Rays start just short of last frame's hits (with a back-off and the nearest of the neighbours).
- [ ] Far pixels re-marched only once they've moved half a pixel, with marching decoupled from shading (hit position
      and facing kept).
- [ ] A slowly refreshed far panorama for turning.
- [ ] A checkerboarded coarse layer, filled in over time (Fork E's remnant).
- [ ] Fixed budgets each frame: holes first, then refreshing old pixels, coarse first and sharpened after.
- [ ] Blue-noise jitter for the temporal phases.

## 4. The processor as a partner

- [~] **The 10-second hitch** (found in the baseline's logs, 2026-11-09, chasing Counsel's question whether the slow
      frames were the screen's tick: they weren't, they're spread from 8 to 20 ms and nearly all waiting on the
      card's own work). One frame every 10 s took 16 to 22 ms in the game's update, growing through the flight: the
      bake's line in the log counted its cells afresh, the whole map's 7 million columns, holding the bake's lock
      meanwhile. Now kept as they come. Lossless (the count the same, tested). To check on the next flight: no
      update spikes on the 10 s beat. The reports now give the 99.9th percentile and the worst frame too (a hitch
      every 10 s is under the 99th).

- [ ] One-way handoffs: the processor prepares frame N + 1's data while the card draws frame N. Never a round trip;
      anything read back is read a few frames late.
- [ ] Camera prediction: the bake fills the ground the camera is heading into first.
- [ ] The turn sent ahead: the strip about to come on screen gets its rays reserved.
- [ ] Late latching and timewarp: the newest mouse read just before the frame goes, and again just before it's
      shown.
- [ ] Watch the laptop's shared heat budget: processor work can lower the card's clocks.
- [ ] The objects' frame work (build 95): every placed object in the loaded chunks gathered each frame, its
      column scanned up to the world's top for cover, its placings sent to the card afresh. Fine for a few
      braziers; check with a settlement's worth (kept between frames, redone only when objects or blocks change).
- [ ] The placed lights' flood (build 95): every light flooded again when the loaded chunks change; with many
      lights, only those near the chunks that came or went.

## 5. The first impression

- [ ] The bake's share of the card revisited (build 84 traded frame rate for bake speed). Measured at the
      baseline: while it runs, the frame rate halves (100 fps median against 180) and the 99th percentile goes from
      21 to 33 ms. A saved world doesn't bake: playing one is the baked case.
- [ ] The bake hidden in the opening (docs/vision.md, The opening): the world forming before your eyes, and the
      first morning's fog following the bake.

## Ruled out

Tensor cores, ray-tracing cores, DLSS, Nvidia's ray sorting (SER) and hardware variable-rate shading, under David's
rule: "im not interested in implementing all the different tricks individual modern cards can use, id have to own 20
gpus".

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

### How each step is judged

- **One step a build, each measured alone,** on the same fixed flight, before and after. The runs alternate
  (before, after, before, after), so the laptop's heat drifting doesn't decide it.
- **Kept only if the gain is clear of the noise:** both pairs agree, and the change is at least about 5% or 0.5 ms.
  Otherwise it's **tried and dropped**, and that's a fine result. It's written down with its numbers either way.
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
4. Empty space skipped smarter: solved jumps, then relaxed cone stepping, for grazing rays over flat land.
5. A two-level start pass (64 × 64 tiles, then 8 × 8).
6. Workgroup shape and Morton order.
7. Tiles by type: sky, far, near, water and seam tiles, each with its own lean shader.
8. Memory, if Nsight says the march waits on memory: hot and cold data apart, the store in Morton order, smaller
   cells, f16 colour sums.
9. Horizon maps for far sun shadows and sky light (made at bake time, patched on edits). Potentially the biggest
   single block win, since shadows and sky light are most of the 8 ms gap to the lean preset. Bigger work, so after
   the cheap ones.
10. Warps kept busy (rays sorted by expected length, compaction, then David's ray queue): only if the slowest-ray
    view shows warps waiting.
11. Textures' last 1 ms; placed light under many lights; the bake in the card's idle gaps.

**C. The processor as a partner** (section 4), slotted in when the instruments point there: the objects' frame work
kept between frames, the lights' flood redone only round what changed, one-way handoffs, camera prediction for the
bake, late latching.

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

- **Which number is the target?** I'd propose: defaults, baked, on the fixed flight, holding 60 fps at the 95th
  percentile, with the lean preset as the fallback. Yours to set.
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

- [ ] A wider fixed flight that takes in sea, mountains and a cave mouth, measured baked and baking separately.
- [ ] A performance gate: every dyno build flies the same route, with its numbers kept in `regroup.md`.
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

- [ ] The bake's share of the card revisited (build 84 traded frame rate for bake speed).
- [ ] The bake hidden in the opening (docs/vision.md, The opening): the world forming before your eyes, and the
      first morning's fog following the bake.

## Ruled out

Tensor cores, ray-tracing cores, DLSS, Nvidia's ray sorting (SER) and hardware variable-rate shading, under David's
rule: "im not interested in implementing all the different tricks individual modern cards can use, id have to own 20
gpus".

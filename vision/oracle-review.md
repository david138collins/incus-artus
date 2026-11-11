# The Oracle's review  [FOR DAVID, 2026-11-11]

An outside read of Incus Artus at build 109, asked for by David before deciding on a rebuild. Nothing here is
decided; it is material for deciding. The Oracle changed no code, ran no flight and committed nothing; this file is
its only edit.

**How to read the marks.** *[checked]* means I or a helper read it in the code or the logs, with the place given.
*[record]* means it is from `docs/` and I did not re-measure it. *[estimate]* is my own guess, with a confidence. The
earlier outside read's guesses were off by ten times both ways, so treat every estimate here as a lead to measure.
Part 7 exists to measure mine before anything is committed.

Sizes are in Builder's hours and David's looks, never days.

---

## For David, in plain words

**The look is right and the store is right. The way the picture is found each frame is what's wrong.** The pyramid
of cells, each the exact sum of its blocks, is the heart of the honest far view and it stays. What I'd change is
that every frame, for every pixel, a long ray goes looking through that pyramid for what's there and then six more
rays go looking for its light. The land hasn't moved since the last frame and the light hasn't changed. Almost all
of that work finds the same answer it found four thousandths of a second ago.

**If you could change only three things, these are the three:**

1. **Keep the light in the world, not in the frame.** Each cell and each block face carries how much sun and sky
   reach it. Rays work that out once when the ground is made, and again when the sun has moved enough (you said
   every thirty seconds would do). The frame then only looks the light up. Near the ground, light is about half of
   what the card does today.
2. **Store the cells so the card can read them directly, and let the card's own hardware start every ray.** Today
   a ray reads through two lists to find each cell and mixes eight cells by hand at every step. Kept in small boxes
   the card can address directly, a cell is one read. The card's triangle hardware, idle today, draws those boxes
   and so tells each pixel where its ray should begin. The ray then walks a few cells, not hundreds. It is still
   your pyramid and still a ray, so thin things still fade and edges still average; nothing is a stand-in.
3. **Make the world once, like a stone tablet, by rules shaped for the card.** The rules run on the card at world
   creation, the result is saved, and the processor and the card both read it. The rules are then written once, not
   three times, and the card no longer has to imitate the processor's arithmetic, which is why a new world's bake
   takes eight minutes today. The same seed still makes the same world on every machine.

**Rebuild?** Yes, with the old project kept beside it as the check. But not yet, and not all at once. First a
short round of experiments, most of them flown in old Incus Artus, each built to prove one of my guesses right or
wrong cheaply (part 7). Then the new eye is built against the old world's own data, so you see the look again, at
speed, before any world generation is rewritten. Then the tablet. Then the rules and the editor, stage by stage,
each checked against old by your eye.

**What I expect, and how sure I am.** On your laptop at the look you play today, about 2 to 3 ms of card time in
ordinary views and under 4.2 in the worst, against 4.3 typical and 12 worst today. A 980 Ti at 1080p well inside
its 16.7 ms. I hold these at medium to low confidence until the experiments are flown. The direction I hold more
firmly than the numbers, because each of the three changes removes a cost your own dyno has already measured.

**What I'd keep exactly as it is:** the cells as exact sums, the layers of 2, 4 and 8, the crisp enlarging, the
blocky light, the far lights, the fog, the way you measure (the dyno, the twin, the picture tests), your decisions
in your own words, and Rust, wgpu and WGSL.

**Three things in your record that turned out softer than they read** (part 1 has the detail): the worst-view
numbers were never properly measured; the card's time on the dyno leaves out the work of making new ground; and
nothing has ever been flown on a weaker card. None of it changes the direction. It changes what the first
experiment is: fix the instruments.

---

## 1. My understanding

### The game

Minecraft and Dwarf Fortress at once: first-person building and exploring in a living world of settlements, a
commercial standalone game. The world is a hand-tuned circle 72,000 blocks across and 1,024 tall, made whole before
play. Its governing idea is the projection: a form (seed, maps, plans, rules) projected into blocks, "so that 1
always equals 1".

The look David signed off: hard, textured blocks near the eye; past where a block is a pixel, cells that are exact
sums of their blocks, drawn in screen squares of 2, 4 and 8 by distance, enlarged crisp; averaged edges that glide;
blocky light in clear steps; very dark nights; far lights that twinkle down to a pixel; on a clear day the whole
world in view.

**Honest**, in David's words of 2026-11-11: near and far agree in colour and shade before any blending, so no seam
needs hiding; everything that exists shows, however small; far changes may arrive late but must be accurate; nothing
is a separately made stand-in. Thin things fade into their background as you walk away; nothing pops.

### What's built

About 62,300 lines of Rust and 5,100 of shader *[checked]*, in 20 days:

- The Tectonic Sim and World Compiler (ridges, peaks, climate, rivers, lakes, first caves) with their editor.
- The Registrar, quarter blocks and octants, objects, placed light.
- "Sight", the team's own renderer on the card (wgpu 30.0.1, DirectX 12), with Bevy reduced to a scheduler reached
  through one doorway (`ecs.rs`, capped by a test at 51 files) *[checked]*.
- The instruments: the dyno's flight, the twin, per-pass card timers, picture tests, bit tests, Nsight traces.

**Sight, as built** *[checked in `src/sight/`]*:

- **The store.** Cells of 16 bytes (how solid, how watery, six face shares, colour, surface) kept as each column's
  band in one pool, found through an index (`card_store.rs`). Cells of 1 to 16 blocks are made live round the eye,
  on the card, from the world's rules. Cells of 32 and up are baked once per world by running the rules under every
  block column of the map, about 5.2 billion.
- **The frame.** One compute pass, 11 or 12 dispatches (`draw.rs:790-845`). For each layer, coarse to fine, a ray
  per pixel: a block-by-block walk near the eye (up to 512 steps, `march.wgsl:838`), then the far march (up to 768
  steps, `:1184`). The far march gathers colour through the cells like light through fog: at each fine step it reads
  four index entries and eight cells and mixes twelve numbers from each (`blend`, `:281-319`), twice in the last
  fifth of every doubling of distance. Then light: a sun ray and five sky rays per pixel, or per block face near the
  eye.
- **The finest layer is 2 × 2** unless chosen otherwise (`look.rs:430-433`; David's presets don't set it). The game
  is full screen at 2,560 × 1,600 and its finest picture is 1,280 × 800 rays, enlarged crisp. David confirms this is
  the look he plays and the look the 240 target is for.

**The world's rules, as built** *[checked in `src/world/`]*:

- Three copies: fractions (`interpreter.rs`), whole numbers (`whole_recipe.rs`), the card's (`recipe.wgsl`). The
  last two are tested equal to the bit. The fractions are still used by the placer's view, overlays and additives.
- The inputs are nine maps at 32 blocks a pixel (2,250²), the mountain points, the river and lake pieces, the cave
  plan and about 140 settings.
- The processor makes chunks only within 4 chunks of the player (`streaming.rs:79`), at about 145,000 columns a
  second a core. The card makes columns at about 23.7 million a second *[record: regroup.md]*.

### What's slow, and why

Measured on the laptop, build 107, the dyno's flight *[checked: `logs/dyno-1794340596.txt`]*: frame median 4.62 ms
(217 fps), the card's sight 4.31 ms median, 10.2 at the 95th percentile, 11.4 at the 99th.

1. **The frame works out again what hasn't changed.** Visibility and light are both found afresh by rays, every
   pixel, every frame.
2. **One general tool does everything, and the card's specialised hardware is idle.** The far march's blend is done
   by hand: one cell read in place of eight took 41% off all of sight *[record: dyno.md:347]*. The programs are large
   enough that the card keeps 9 to 10.5% of its capacity in flight *[checked: `logs/nsight/queries/m104_*.json`]*.
3. **Light is paid per pixel, per frame.** Shading 29 to 35% and light per face 13.5 to 19% of the card
   *[record: traces at 104 and 107]*.
4. **The rules on the card are shaped by having to match the processor.** The arithmetic is 16.16 fixed point with
   64-bit intermediates. WGSL has no 64-bit numbers, so each division is a loop of about 40 turns
   (`whole/kit.wgsl:57-79`), 25 to 30 of them a column *[checked]*. That is the 24,100 instructions and 255
   registers, and most of the eight-minute bake. Meanwhile the maps are only 32 blocks a pixel and most of the rules
   vary over hundreds of blocks, yet all of it is worked out under every block.

### What I found that differs from the record  *[all checked]*

- **The worst-view figures are soft.** "8 ms standing at the spawn" comes from two soaks run at the 237 cap with
  vsync, at 6,000 blocks north of the middle, not at the spawn (`logs/session-1794337013.log`, `soak.rs:38-40`).
  "70 fps, 14 ms beside quarter blocks" is David's spoken report; 14 ms is 1000 ÷ 70, a frame time; the play log
  from that session shows frame medians of 10.5 to 12.1 ms at the cap with the panel up
  (`logs/session-1794341727.log`).
- **The dyno's "sight on the card" leaves out tile making.** That work is separate small submissions from the
  loading threads. In the build 104 traces it is 1.8 to 4.8 ms a frame over the land and 0.6 to 1.4 at sea, outside
  the number (`m104_land.json`, `m104_sea.json`).
- **No flight exists for builds 108 or 109.** The review's flight figures are build 107's.
- **Flight headers don't pin what flew.** Every header says "+ changes"; the `INCUS_*` switches aren't recorded;
  "uncapped" and the list of programs running beside it are fixed text (`soak.rs:553, 563`).
- **At sea the top stall is memory, not "Wait"** (Long Scoreboard 3.45 against Wait 3.11, `m104_sea.json`), which
  sits awkwardly with "the march isn't waiting on memory" (dyno.md:223).
- **Nothing has been measured on a weaker card,** and the low-power switch the docs describe no longer works: the
  runner asks for the high-performance card and reads no setting (`runner/mod.rs:349`). The nearest thing is two
  flights with the card's clock held at 1,050 MHz, cited nowhere: frame median 15.9 to 18.1 ms.
- **The twins have drifted in one place.** Sight's block chooser builds its column without the map fields
  (`sight/cell.wgsl:76-89`; only `with_maps` at `recipe.wgsl:1416` fills them), so any distributor reading those
  fields would choose differently far off than in the chunks. No default distributor reads them, so the tests pass.
- **Shaders are built with the library's runtime checks on** (every index clamped, every loop given a counter, every
  whole-number division guarded): `create_shader_module` at `draw.rs:302` and `on_card.rs:143`. An unchecked form
  exists (`wgpu-30.0.1/src/api/device.rs:181`). Its cost here is unmeasured.

---

## 2. The assumptions I'd challenge

Each with its evidence and what changes if it falls.

| # | The assumption | Evidence against it | If it falls |
|---|---|---|---|
| 1 | **What's visible must be found by a long ray, every pixel, every frame.** | The march and its blend are the largest cost (32 to 62% of the card; the blend alone up to 41%). Air is already cheap to skip (strides twice as long: −3%); the cost is at surfaces, in how a cell is read *[record]*. David has released "everything must be marched". | Cells stored for direct reading; rays started by drawn boxes and walking a few cells (part 4). |
| 2 | **Light belongs to the pixel.** | Shading and light per face are about half the card near the ground. Light depends on the place, the sun and the world, never on the eye. The attempt to keep it (build 107) kept it in a table keyed from the eye and filled on first sight, so in flight it was made every frame anyway. That drop was for its context. | Light kept in the cells, made when the ground is made, refreshed when the sun steps. The frame looks it up. |
| 3 | **"1 always equals 1" means the processor and the card work the same thing out and agree to the bit.** | It costs the 64-bit imitation (40-turn divisions), 255 registers, three copies of every rule, two copies of the maps in memory plus one on the card (178 MB each), and it has already drifted once (above). David's own decision of 2026-10-25: "nothing about the ground's shape is worked out at runtime." | Work it out once, store it, everyone reads it. The guarantee is stronger, not weaker: there is only one answer to disagree with. |
| 4 | **The rules must run under every block column.** | The maps are 32 blocks a pixel; the ridges' finest layer is about 680 blocks; climate is computed at 128 blocks and stored 16 times over; the map blend's sixteen-tap coefficients are constant inside each map pixel *[checked]*. Only the fine noises, the river carve, the rounding and the surface choice are truly per block. | Slow parts on a coarse lattice, fine parts per column, the open sea skipped (part 5). |
| 5 | **Whole numbers on the card need 64-bit sums.** | They do only because the formulas were written for the processor first. With no twin to match, the formula written for 32 bits is the definition. | Card-native whole numbers: same seed, same world, on every card, without the long way round. |
| 6 | **Every level of cells is summed from blocks.** | The live levels each sum from blocks (`store.rs:13-14`); a cell of 16 is 4,096 blocks. The sums are additive, and the bake already makes parents from eight children exactly (`bake.rs:72-89`). | Each level from the one below: about a quarter of the summing, the same cells to the bit. |
| 7 | **The far view must be redrawn at the frame's rate.** | A surface 5,000 blocks off slides across the screen by movement ÷ distance; walking, that is a far pixel every few seconds. | A reserve lever, not the first move: a far picture fixed to the world, read by the screen's squares (part 3, design E). |
| 8 | **The compact store (bands in a pool) is worth its price.** | Every cell read is an index read then a pool read, with wrapping arithmetic; neighbours aren't neighbours in memory; hardware can't address it. Bricks without the filter's border cost 2.6 times the cells *[record: dyno.md:393]*. | Bricks, directly addressed. More memory per cell, far fewer and cheaper reads, and only the rings in use need to be on the card. |
| 9 | **The dyno's flight is the mark.** | It flies 60 to 300 blocks a second at 20 to 40 up. The heaviest views are standing near the ground and beside builds, which it doesn't fly, and whose numbers are soft. | The flight stays (David wants dev flight demanding). Three standing marks are added and every header says exactly what flew. |
| 10 | **A new world can be played while it bakes.** | The "creaking": frames at half rate for minutes *[record]*. David accepts stage magic over a wait. | The whole world is made before play. With rules shaped for the card I estimate one to three minutes on the laptop *[estimate, low to medium]*. |
| 11 | **The weak card can wait until the ten report.** | David's call, and I respect the order: it is not the floor to design to. But half the spec has no measurement, and the laptop has a second, small card built in. | One flight of the marks on the small card whenever the renderer changes shape. It costs minutes. |
| 12 | **Rust, wgpu, WGSL.** | Questioned and kept. Bevy is already behind a door. DirectX 12 through wgpu measured 240 against Vulkan's 6 on this laptop *[record: memory]*. The library's costs are specific and addressable: runtime checks, the compiler's layout lottery on big programs, no early-depth hint on DirectX 12 (`features.rs:1115`, Vulkan only), which the design in part 4 avoids needing. | Nothing changes. Small programs and an unchecked build for shipping. |

---

## 3. The architecture, compared

Five whole designs for the graphics. Costs are for the look David plays (finest 2 × 2): laptop at 2,560 × 1,600,
980 Ti at 1,920 × 1,080. Only design A has measurements; everything else is *[estimate]*.

### A. Today's, tuned on

The staged march as built, working down the dyno's list.

- **Cost.** Laptop: 4.3 ms typical, 10 to 12 worst, more near the ground *[checked and record]*. 980 Ti: perhaps 6
  to 9 typical, 15 to 25 worst *[estimate, low: from the 1,050 MHz flights and half the pixels]*.
- **What's left on its list** are steps of a few percent each, and half of those tried have lost to the compiler's
  layout of the big program.
- **Risk.** Low risk of harm, high risk of never arriving: it needs three times at the worst frames.
- **Gives up.** The target, most likely.

### B. Builder's hybrid (the review's routes A to E)

Near chunks as triangles; light kept in the world; the far march made lean and reused between frames.

- **Cost.** Laptop: perhaps 2.5 to 4 typical, 5 to 7 worst without reuse between frames; 240 leans on that reuse
  *[estimate, low]*. 980 Ti: reachable *[estimate, medium]*.
- **Risk.** Two different drawers (triangles near, long compute rays far) that must agree at the line where they
  meet. Reuse between frames brings its own rules for what may lag, and resampling fights the crisp squares. The far
  march keeps its shape: long rays through the indexed pool.
- **Gives up.** A little of "one system". Possibly crispness under reuse.
- **My view.** Sound, and its first two routes are right. Routes A and B are in my choice too. I part from it on
  the far march: I would not make the long ray leaner; I would make it short.

### C. All triangles

Every level of the pyramid meshed as faces (cells as big blocks, or far ground as smooth sheets), drawn by the
triangle hardware into the 2, 4 and 8 layers.

- **Cost.** The cheapest per pixel of all: perhaps 1 to 2 ms on the laptop, 3 to 5 on a 980 Ti
  *[estimate, medium: this is the well-trodden road]*.
- **Risk.** The honest part. A cell that is a third full of fence, a canopy with gaps, a soft averaged skyline: a
  triangle is there or not there. Partial things need thresholds (things pop) or dithering and sorting bolted on.
  Every level needs a mesher. It is the "bits and pieces of conventional methods" David left.
- **Gives up.** Fading without popping, unless special cases are added for exactly the things he cares about.
- **My view.** No for the world. Yes for what triangles are honest about: models, people, sprites, glows.

### D. The lens: the same cells, read directly, every ray started by the card's own hardware

The choice. Set out in part 4. In one paragraph: the pyramid of exact cells is kept in bricks (small boxes of cells
the card addresses directly) with the light already in them. For each distance band, the triangle hardware draws the
bricks' boxes into that band's layer (2, 4 or 8), which gives every pixel the place its ray should start and skips
the sky altogether. A small program walks the few cells from there, gathering coverage as today's march does, so a
fence is still a faint line and a skyline still averages. Near blocks are the same thing at cell size 1, meeting
hard faces. Light is looked up. Models and people are triangles drawn over it, as now.

- **Cost.** Laptop: about 2 to 3 ms typical, under 4.2 worst. 980 Ti: about 3 to 5 typical, 6 to 8 worst
  *[estimate, medium to low; part 4 has the budget, part 7 the experiments that price it]*.
- **Risk.** Rays that skim along the ground through many bricks; keeping bricks fed in very fast flight; the look at
  the lines between levels; memory if David chooses soft cells over crisp. Each has an experiment and a fallback.
- **Gives up.** The band store's compactness (more memory per cell). The coarse-to-fine chain between layers
  (each layer now stands alone, which also removes the waits between them).
- **Why this one.** It removes, by structure, the three costs the dyno has already measured: the hand-made blend
  and its reads, the light rays, and the big programs. It uses the two idle machines David asked about on 2026-11-09
  (the triangle hardware and, if he chooses soft cells, the texture hardware). It is one drawer at every scale, near
  included, so nothing has to be matched at a seam. It needs no reuse between frames, so nothing ghosts. And if its
  experiments disappoint, the bricks and the light serve design B unchanged: the work isn't lost.

### E. The far picture fixed to the world (a lever, not a whole design)

David's idea of 2026-11-11: "screen pixels that read through the world pixels in a simple grid." The far bands are
drawn into a picture fixed to the world round the eye, finer than the screen's far squares, and redrawn a little at
a time as the eye moves; each screen square averages the world pixels behind it. Turning costs nothing; cost follows
blocks travelled, not frames.

- **Use.** On top of D (or B) if 240 isn't met on the far bands, or later for 400.
- **Risk.** Things hidden and then revealed as you move; fast flight outruns it; one more thing to keep true.
- **My view.** Held in reserve. D may not need it, and D alone is simpler to keep honest.

### The choice

**D, with light kept in the world and the world made once (part 5), E in reserve, C's triangles for models and
people only.** A is where the numbers come from. B is the fallback that shares D's data.

---

## 4. The graphics in depth: the lens

Everything in this part is design, so everything is *[estimate]* unless marked. It is written for Builder to
attack, not to follow.

### 4.1 The data on the card

**Bricks.** At each level (cell size 1, 2, 4 … blocks) the world is cut into bricks of 8 × 8 × 8 cells. Only bricks
that hold some surface exist; solid rock and open air have none. A brick's cells sit together in one run of memory,
so a cell is found by arithmetic: brick slot × 512 + its place in the brick. One read, no index in between.

**The brick index.** Per level, a small 3D grid round the eye saying which brick slot, if any, is at each brick
position, with "all air" and "all solid" as two special values. A power of two a side, wrapping by a mask (no
division, no float fix-up: the lesson of build 105). For today's window of 672 cells that is 128 × 128 bricks a
level and a few megabytes.

**A cell** stays about 16 bytes, repacked into what's read every step and what's read once at a hit:

- every step: how solid, how watery (2 bytes), and a 64-bit mask per brick saying which 2 × 2 × 2 groups of its
  cells are empty, so a ray crosses empty parts of a brick without reading them;
- at a hit: colour, the six face shares, **and the light** (how much sun reaches it, how open it is to the sky,
  placed light in its 16 steps a channel), which the cell doesn't hold today.

At cell size 1 the brick holds blocks: the block's id where there is one, and in the air cells beside faces, the
light arriving there (the way placed light already works: a face takes the light of the cell in front of it).

**Crisp or soft is a choice of how the brick is read, not of the design.** Crisp: the ray reads the one cell it is
in; bricks are plain buffers, 2.6 times the band store's cells *[record]*. Soft: bricks are 3D textures with a
one-cell border and the texture hardware blends eight cells in one fetch; 5.5 times the cells. David is unsure and
wants to see it (experiment 2).

**Memory.** Only the bricks of the bands in use need be on the card. Each band holds about the same number of
columns whatever its distance, so I estimate 20 to 40 MB a band crisp and 250 to 450 MB in all, twice that soft
*[estimate, medium]*. Today's pool is 384 MB allocated *[checked]*. A 980 Ti has 6 GB.

### 4.2 The near field (cells of 1: blocks)

The same drawer at the finest level. The triangle hardware draws the back faces of the near bricks' boxes; each
pixel's program works out where its ray enters the nearest box, then steps block by block (crossing empty 2 × 2 × 2
groups by the mask) to the first hard face. From there it is today's near picture: the face's texture, crisp and
mipmapped as now; a cell of pieces walked by its octants as now; the face's light looked up from the air cell in
front of it.

- **It is the same picture as today's near walk.** One ray through each 2 × 2 pixel's middle meeting the first
  face is exactly what the march does now, so it can be checked against old to the pixel.
- **Why it should be faster.** No coarse-column probes at every block step (four index reads before each block
  today, `march.wgsl:853`), a start at the box and not at the eye, a small program of its own, and no light rays.
  The dyno already found that near stepping is not the main cost; near light is *[record: dyno.md:598-605]*.
- **Quarter blocks** stop being special for speed: their cost is a few octant steps at the hit, with no light rays
  behind them.
- **The fallback** if skimming rays cost too much here: near faces as triangles (a mesher on the loading lane).
  A triangle sampled at a pixel's middle gives the identical picture to a ray through that middle, so David's worry
  about matching the marched 2 × 2 is answered either way; only averaged edges would need more samples.

### 4.3 The far view (cells of 2 and up)

Each distance band belongs to one layer (2 × 2, 4 × 4 or 8 × 8 pixels) by David's crossover curves, as today. For
each layer, the boxes of that band's bricks are drawn into the layer's own picture. Each pixel's program enters the
nearest box and walks cells, gathering coverage exactly as today's march does (how solid × how far through), until
it is covered or leaves the band. If it leaves a brick uncovered it steps through the brick index to the next one.

- **Thin things and edges.** A cell a third full of fence covers a third: a faint line that fades with distance,
  never a pop. A skyline's cells are part full: the averaged, gliding edge. Leaves with gaps cover in part. This is
  why the drawer is still a ray inside the box and not a triangle.
- **Between layers.** Each layer reports how covered each pixel is. The enlarging pass lays fine over coarse by
  that coverage: nearer bands are finer, so the order is always right, like the cels David described. Layers no
  longer wait on each other: today's parent-to-child chain, and its twelve drains a frame, is gone.
- **Between levels.** A brick is one level; which level is drawn where follows the same distance rule as today.
  Today's blend of two levels over the last fifth of each doubling was a cure for rings over flat water, and those
  rings came from how much a soft edge gathers at each cell size. With cells read exactly along the ray (the
  length inside each cell is known, so no fine sub-steps and no walking back), flat ground gathers the same at
  both levels. If rings still show, the cure is a level chosen per pixel by a dither across the hand-over, which
  costs nothing. David's eye decides.
- **Caves, halls, towers.** A cave mouth is bricks like anything else; the lit market inside is light in those
  cells. Nothing needs an identity.
- **The whole world on a clear day.** The coarsest bands are a few thousand bricks. Cost follows the screen.

### 4.4 Light and shadow

**Kept in the cells, made off the frame.** A lane job, on the card, lights bricks: for each cell with surface (and
at cell size 1, each air cell beside a face) a ray toward the sun and the sky's cones, through the same bricks,
written into the cell. The code is today's `cone()` and `sky_seen` moved from the frame to the lane.

- **When.** When a brick is made or changed (its own cells and those it shades). And when the sun has stepped:
  David allows steps as rough as 30 seconds. All the bricks in view are relit over a second or two, nearest first.
- **How much.** The frame today lights about a million pixels 240 times a second. The lane lights each cell once
  when it appears and again on a sun step: about a thousandth of the work in fast flight, far less standing still
  *[estimate, medium]*.
- **What stays continuous for free.** The sun's angle on each of a face's six ways, the sun's and the sky's colour,
  day turning to dusk: these are sums at lookup time, as now. Only what is in shadow steps.
- **Two moons.** The brighter light in the sky casts the shadows; the hand-over at dusk and dawn happens when both
  are dim. A question for David (part 10).
- **Placed light** floods on the processor as now, into the cells' light, with the wash for far levels. A carried
  lantern refloods only when its holder crosses into a new block.
- **People's shadows, real time.** A small shadow picture from the sun's side, covering about 64 blocks round the
  player, drawn each frame with only the moving things in it. A near face reads it once, at its middle, so the
  shadow lands in whole faces: a person darkens two or three blocks, and you see it come round the corner.
- **Far lights** twinkle as now: one line of sight each, now through bricks.

### 4.5 Water, glass, leaves

The walk is front to back along each ray, so see-through things come in the right order with no sorting, as in
today's march: water dims by its depth, glass tints, a leaf face reads its texture and the ray goes on through the
gaps. Animated and palette-cycled textures are a lookup by time at the face. Glass that wraps its texture across
neighbouring glass reads the neighbours in the same brick.

### 4.6 Objects and people

Triangles, as now: models, sprites, flames and glows drawn after the world into its depth. The world's passes write
how far each pixel's ray went, and the enlarging pass turns that into depth, as today (`upscale.wgsl`). 800 figures
of a few hundred triangles each is small work for either card. Each is lit from the light of the cell it stands in.

To fix while porting *[checked]*: `models.rs` asks the disk about every placed object's file every frame
(`:395-399`) and makes new vertex buffers every frame (`:505-519`).

### 4.7 The frame on the card

| Step | Kind | Notes |
|---|---|---|
| The moving things' shadow picture | triangles | small, only near movers |
| Layer 8, layer 4, layer 2 | triangles for the boxes, a small program per pixel | independent of each other; one indirect draw per layer; nearest tiles first so hidden boxes are culled early |
| Enlarging | one fullscreen pass | fine over coarse by coverage, crisp, writes depth |
| Objects, people, sprites, glows | triangles | as now |
| The panel or the game's interface | | |

One submission. The processor's part is the camera's numbers and the order of a few thousand tiles, redone only
when the eye crosses a tile: the floor David asked for.

**The lanes** (never in the frame's path): bricks made from the tablet and uploaded, light jobs, edits re-summed.
Each has a budget of the card's time a frame, and the dyno counts it inside the card's time.

### 4.8 A frame budget for each machine

The look David plays: finest 2 × 2, coarsest 8 × 8, shadows and sky light on, the clear day.

**Measured today (design A), laptop** *[checked: `logs/dyno-1794340596.txt`, build 107; traces as cited]*:

| | ms |
|---|---|
| Sight's compute, flight median / 95th / 99th | 4.31 / 10.18 / 11.39 |
| Enlarging | 0.17 |
| Tile making, outside the number above (traces, build 104) | 0.6 to 4.8 |
| The processor's steps a frame | 1.7 to 2.1 (the debug panel about 1 of it) |
| Near the ground, capped, not at the spawn | about 8 |

**Guessed for the lens** *[estimate; confidence beside each]*:

| | Laptop, 1600p (budget 4.2) | 980 Ti, 1080p (budget 16.7) | Confidence |
|---|---|---|---|
| Near layer (boxes, block walk, lookups) | 0.8 to 1.5 | 1.5 to 3 | low to medium |
| Far layers 2, 4, 8 | 0.5 to 1.0 | 1 to 2 | low to medium |
| Enlarging | 0.2 | 0.3 | high (measured today) |
| Objects, people (ordinary / the 800 battle) | 0.2 / 0.8 | 0.5 / 2 | medium |
| Lanes' share (bricks, light) | 0.3 to 0.5 | 0.5 to 1.5 | low |
| **Typical / worst** | **2 to 3 / under 4.2** | **4 to 6 / 8 to 9** | low to medium |
| The processor (panel closed) | 0.6 to 0.9 | 1 to 2 | medium |

The 980 Ti's column assumes a card about a third as fast drawing half the pixels. It is the softest column in this
review.

**What would make these wrong:** skimming rays through long runs of surface bricks near the ground; forests (many
part-covered cells, so longer walks); the lanes starving in fast flight. Experiments 5 and 6 are aimed at exactly
these.

---

## 5. The world's rules, shaped for the card

### 5.1 One source instead of three

**Each rule lives in one place and runs in one place.**

- **Plans** (tectonics, climate, the river network, lakes, peaks, cave plans) are made on the processor, once, at
  world creation, as now.
- **Projection** (the ground under each column, the carve, the surface, the blocks, the cells) is written once, in
  WGSL, and runs only on the card, only at world creation and when the editor previews.
- **Nobody else works anything out.** The processor's chunks, the renderer's bricks and the editor's views read the
  result.

The fractions go. The whole-number Rust copy goes. The editor previews by running the very stages the game runs, on
the region in view, so it can never show something the game won't make.

**"The same seed makes the same world" is kept** (David, 2026-11-11). Whole-number arithmetic on the card is exact
on every card by the shading language's own rules. What goes is the twin, not the sameness.

**How it's tested without a twin:**

- A world's tablet is hashed tile by tile. The same seed must give the same hashes on the laptop's two cards (the
  NVIDIA one and the small built-in one): a real test of "every card agrees", on hardware David already owns.
- Golden hashes for the default preset are kept in the tests, so a rule changed by accident shows at once.
- One caution *[estimate, unchecked]*: world creation's plans use fractions with sines and powers on the processor.
  Those can round differently between processors, so "same seed, same world" may not be fully true across machines
  even today. The rebuild's plan stages should use the same table-based maths the recipe already has
  (`src/whole/tables.rs`), or keep to plain sums and products in a fixed order.

### 5.2 Cut by how fast each part changes

| Stage | Where | Grain | What |
|---|---|---|---|
| 0. Plans | processor | maps at 32 blocks; climate at 128 (stored at its own size, not 16 times over) | as now |
| 1. Broad ground | card | a lattice of 8 blocks | the map blend, the land and sea curves' inputs, ridges, the peaks' rise, the lift: all the heavy maths (powers, angles, divisions) at a sixty-fourth of the columns |
| 2. Fine ground | card | every column | stage 1 brought down by whole-number interpolation (the spacing is a power of two, so shifts, never division), then the sharp coast curve, crags, detail, torn coast |
| 3. Water's carve | card | every column, only in tiles the river grid marks | rivers, banks, lakes |
| 4. Surface | card | every column | slope, the distributors' choice of top and layers |
| 5. Blocks | card | only inside marked boxes | caves; later ores and strata |
| 6. Sums and light | card | each level from the one below | cells, sky openness, bricks; then the tablet is written |

- **The lesson of the coast is kept:** the sharp land and sea curve is applied per column after interpolating its
  inputs, never interpolated itself (`interpreter.rs:20-25`, the "humps along every coast").
- **The open sea is skipped.** About 70% of the world is water; past the shelf nothing but stage 1 runs and nothing
  is stored.
- **Why staging should win now when it lost on 2026-11-10.** That attempt cut the same work into pieces and paid
  hand-overs through the processor; its heaviest piece still held 255 registers. Here the heavy piece runs at a
  sixty-fourth of the places, and stages hand on through the card's own memory with no trip to the processor in
  between. A different context, by David's own rule.
- **The worlds will differ in detail** from old's (a lattice and new arithmetic). David allowed this on 2026-11-05
  and again for the rebuild; the check is his eye on his default preset, stage by stage, old beside new.

### 5.3 Whole numbers that fit the card

- Heights and positions in 256ths of a block, noise and shares in 65,536ths, with every product arranged to fit 32
  bits (16 × 16 bit products, or split as the Perlin already does).
- Division by a setting becomes multiplication by its reciprocal, worked out once on the processor and handed over
  as a number. With no twin to match, that formula is the definition, so there is nothing to be inexact against.
- Curves become tables made on the processor from David's dots and bends, read with a whole-number blend.
- Powers, logs, sines and angles stay as the tables there are now.
- wgpu 30 does offer true 64-bit whole numbers on DirectX 12 as an optional feature
  *[checked: `wgpu-types-30.0.1/src/features.rs:1125`]*. I'd leave it unused: it isn't on every card, and the aim is
  rules that don't need it.

### 5.4 How a new world's bake gets fast

Today: 23.7 million columns a second, 5.2 billion columns, cells of 32 and up only, about 8 minutes on the laptop
*[record]*.

After *[estimate, low to medium]*: perhaps 200 to 400 million columns a second for the fine stages (no long
divisions, the heavy maths at a sixty-fourth of the places), over about a third of the columns (land and shelf).
That is tens of seconds of card time for every block column of the world, plus the sums, the light and writing the
tablet to disk. **One to three minutes on the laptop; several minutes on a 980 Ti-class machine, inside David's ten.**
Experiment 7 measures the first number, which decides the rest.

### 5.5 The tablet, and the changes on top

**The tablet** is the made world on disk: per column its ground, its water and its surface profile; the boxes where
the world isn't a plain column (caves, overhangs); and the cells of every level. Kept in region files, compressed,
read by the loading lane. My estimate is 1 to 4 GB a world *[low to medium; experiment 8 measures it]*. David
accepts 4.

**The changes** are a second layer of the same shape, with firm bounds (David's fear: "an overgrown hack"):

- A chunk is stored whole, in the tablet's own format, or not at all. Size follows the area touched, never the
  number of actions. The worst case is one more tablet.
- **One doorway** answers "what is here now": the changed chunk if there is one, else the made one. Walking,
  drawing, the simulation and the cells all ask through it. No system handles changes its own way.
- The cells are always the sum of the world as it is. A change re-sums one cell a level. A test re-sums any region
  from scratch and compares.
- A chunk that ends exactly as it was made is dropped from the layer on saving.
- The lite simulation writes the same way: a building far off is a blueprint stamped into its chunks, then summed.
- The thousand years of history at creation are folded in before the tablet is written: the tablet is the world as
  play begins.

---

## 6. Keep, port, rewrite, drop

"Port" means carried over nearly as it is, off Bevy. "Rewrite" means built again from what was learned, with old as
the check. Line counts *[checked]*.

### World generation and the editor (`src/worldgen/`, 23,200)

| Part | Verdict | Notes |
|---|---|---|
| Tectonic Sim (`plates`, `tectonics`, `rags`, `heights`, `bake`; 3,260) | **Port** | The tuned behaviour is the asset. Move its sines and powers to the table maths for sameness across machines. Check: its maps beside old's for the default preset. |
| Climate (`climate`, `temperature`, `wind`, `rain`; 1,075) | **Port** | Store its fields at their own size. |
| Rivers (`rivers`, `network`, `carving`; 3,300) | **Port first, rearrange later** | "Rivers were a beast." Carry the planner over and match old's network before any new arrangement of sea level and shores, which is a design step of its own with David. |
| The compiler and its views (`compiler`, `compiler_view`, `lab`, `overlays`, `additives`, `placer_view`, `view`, `world`; about 9,600) | **Rewrite** | David: "we can do better doing it again." Built on the stage graph, a view per stage, previews from the card's own stages. Keep egui. |
| Graphics Kit (`noise`, `worley`, `warp`, `image`, `layers`, `blend`, `palette` …; about 3,900) | **Keep** | Plain Rust, no Bevy, liked. |
| Registrar and Color Lab, objects tab, notes (3,100) | **Port** | Liked; its data files as they are. |

### The world (`src/world/`, 18,900)

| Part | Verdict | Notes |
|---|---|---|
| `interpreter.rs` (the fractions' recipe) | **Drop** | Its settings and curve definitions move to data first. |
| `whole_recipe.rs`, `card_recipe/recipe.wgsl`, `whole/kit.wgsl` | **Rewrite** | As the staged, 32-bit rules of part 5. Old is the reference for what each rule does. |
| `card_recipe/on_card.rs`, the twin tests | **Drop** | Replaced by the tablet's hashes. Keep the test of same-made-twice. |
| `carving.rs` (river and lake pieces, their save format) | **Port** the data; **rewrite** the carve on the card only. |
| `mineralogy.rs`, `placer/` | **Port** the design (fields and distributors as data); evaluation on the card only. The drift found in part 1 disappears with the twin. |
| `caves.rs` | **Port** the plan; carving at creation into the tablet's boxes. |
| `chunk`, `section`, `pieces`, `objects`, `raycast`, `position`, `block` | **Port** | Small, working, plain. |
| `storage`, `streaming`, `edits`, `terrain`, `mod.rs` | **Rewrite** | Round the tablet, the changes and the one doorway. |
| `generator`, `test_world`, `test_sites` | **Rewrite** as fixed test places, the same ones old's golden pictures were taken at. |

### Sight (`src/sight/`, 7,800)

| Part | Verdict | Notes |
|---|---|---|
| `cell.rs` (the exact sums, the cell's definition) | **Keep** | The promise the far view rests on, with its tests. Add the light. |
| `cell.wgsl` | **Rewrite** | Sums from the level below; columns from the tablet. |
| `store.rs`, `card_store.rs`, `bake.rs` | **Rewrite** | Bricks, the brick index, the tablet's reader. |
| `march.wgsl` | **Rewrite** as small programs. **Port word for word** what defines the look: `faces_light`, `lit` and `sunlit`, the fog and sky, the textures and their mipmaps, the octants, the crossover curves, and `cone` and `sky_seen` for the light lane. Leave the build-by-build history in the journal, not the comments. |
| `draw.rs` | **Rewrite** |
| `upscale.wgsl` | **Port** |
| `models.rs`, `models.wgsl`, `textures.rs` | **Port**, with the two per-frame fixes of 4.6. |
| `lights.rs` (the flood, the wash, far glows) | **Port** |
| `edits.rs` | **Rewrite** on the doorway. |

### The rest

| Part | Verdict | Notes |
|---|---|---|
| Runner, loading lanes, hardware attention, keys (`runner/`, `loading.rs`) | **Port** | Already our own loop. The simulation gets its own lane. |
| Instruments (`perf`, `soak`, `dyno`, `diagnostics`, `temperatures`, `logs/nsight/scripts`) | **Port first**, with part 7's repairs | Before anything they measure. |
| `ecs.rs` and Bevy | **Drop** | A small scheduler of our own, built from the declared graph (part 9). |
| `time/` (the clock David likes) | **Port early** | The runner needs it. |
| `data/`, `player/`, `camera/`, `kinematics/` | **Port** by feel | Drop the camera's unused panini settings. |
| `sim/`, `people/`, `fate/`, `events/` | **Park** | Ported when the spec reaches them. |
| `save/` | **Rewrite** | A world folder: the tablet, the changes, the settings. |
| `debug/`, `look.rs` | **Rewrite** | As the control room and settings-as-data. |
| `assets/shaders/*.wesl` (2,424 lines) | **Drop** | Never loaded *[checked]*; the engine before the regroup. |
| Assets, art, palettes, block and object data | **Keep** as they are |
| Presets | **Keep** the startup default only, re-saved (David's standing rule). |
| Tests (435) | **Port** those that guard promises (a cell is its blocks' sum, made twice the same, the pictures); **drop** the twin tests. |
| `docs/`, `vision/` | **Freeze** as the journal. `master/` is the living truth. |

### Found in passing, for Builder  *[checked; none measured]*

- Finer layers are dispatched over every pixel even where the parent says no (`draw.rs`; early exit, but two
  texels and two words still written).
- The 1 MiB face-key table is cleared every frame (`draw.rs:764`).
- `STORAGE_BUFFERS_NEEDED` says 10 where `listed_staged` binds 16 (`whole/shader.rs:61`).
- Each tile piece is its own submission and wait, with 200 µs polls (`on_card.rs:85-111, 380-404`).
- `fog_enabled` in the render presets is an old key nothing reads.
- `chunk.rs:1-4` and `store.rs:2` carry stale sizes in their comments.

---

## 7. The first experiments

Small prototypes that settle the biggest uncertainties before anything is committed. The first four are flown in old
Incus Artus as it stands (David: "we may need to keep writing in old incus artus for a bit to use it for
experiments"). Each has its pass mark written before it is flown. One Builder, so they run in this order.

| # | Experiment | What it measures | What result changes the plan | Size |
|---|---|---|---|---|
| 1 | **The instruments repaired, and real baselines.** Flight headers record every switch and whether the code was committed; the card's time includes the lanes' work; three standing marks added to the flight: *on the ground at the true spawn*, *beside the quarter-block build*, *a mountain top on the clear day*, all uncapped; the choice of card restored, and the marks flown once on the laptop's small card. | The true size of the gap in the views that matter, and the first weak-card numbers ever. | If the small card fails for a reason other than speed (memory, a limit, a compiler fault), that shapes the store before anything is built. | 3 to 4 h; David watches two flights |
| 2 | **The one-cell look.** The probe exists (`logs/nsight/probes`, p7; remade from today's march). Same places, both ways, stills and in motion. | Whether crisp far cells are wrong, acceptable or good to David's eye. | Crisp: bricks are plain buffers at 2.6 times the cells, the simplest and fastest form. Soft: 3D textures with borders at 5.5 times, the texture hardware does the blend, and the memory plan doubles. | 1 h; one look |
| 3 | **Runtime checks off.** The march and the recipe built with `create_shader_module_trusted`, unchecked. Flown as a pair against the same build checked. | What the library's guards cost in programs this size. My guess is 3 to 15% *[low]*. | If it is large, every later experiment in old flies unchecked so the comparisons are fair, and the new engine ships unchecked with the checks kept in tests. | 1 h |
| 4 | **Light for free, then light looked up.** (a) The marks flown with sky light and shadows switched off: the ceiling, from switches that exist. (b) A minimal real form: sun and sky written into the baked cells when a region is made, and the shading pass reading them for far pixels in place of its rays. | (a) How much of each mark is light. (b) What a lookup really costs, and David's look at light that steps. | If the lookup costs as much as the rays (as the sky map did inside the one big program), the light moves into the hit record's read, or the brick's layout is reconsidered before bricks are built. | (a) 0.5 h; (b) 4 to 6 h; one look |
| 5 | **The brick walk, from black: the far view.** A small new program (the new project's first code). It reads old's `sight-bake.bin`, builds bricks and their index for the cells of 32 and up, and draws the far view only, into the 2, 4 and 8 layers, with the boxes started by the triangle hardware and, as a second switch, without them. Laptop and small card, the dyno's far legs and the mountain mark; pictures beside old's at the same places. | The far layers' cost in the chosen design; what the box start is worth by itself; whether level lines show. | **Pass:** the far layers under 1 ms typical and 2 worst on the laptop. Twice that or more: keep the bricks and the light, drop the box start, and draw far as design B's lean compute march over the same bricks; then weigh design E. | 10 to 14 h; two looks |
| 6 | **The same at cell size 1.** The same program fed near blocks from old's chunks at the spawn mark and the quarter-block build, light looked up. Pictures against old to the pixel. | The near field's cost where today is worst: skimming rays and pieces. | **Pass:** the ground mark under 1.5 ms on the laptop. If skimming rays through bricks are the cost, near faces become triangles (the fallback of 4.2), which is a mesher on the loading lane and no change to anything else. | 8 to 10 h; one look |
| 7 | **The rules in 32 bits, by grain.** Stages 1 and 2 of part 5 for the ground alone, on the card. Columns a second on both cards; a map of the default preset beside old's. | The bake's speed, and whether the reshaped rules still make David's world. | **Pass:** at least 150 million columns a second on the laptop, and David can't tell which map is which in kind. Under 50 million: store the ground coarser and keep a small detail stage when bricks are made. | 8 to 12 h; one look |
| 8 | **The tablet's size.** A tool that takes a wide piece of old's world (columns from old's own card recipe), packs it as the tablet would be, and reports bytes a column for land, shore and sea. | The world folder's size. | Over 4 GB a world: store only the columns and the coarse levels, and sum the fine levels as they load. | 2 to 3 h |
| 9 | *(Reserve.)* **The far picture fixed to the world.** Only if 5 passes narrowly or David wants 400. | Design E's worth and its look when ground is revealed. | | 8 to 12 h |

After 6, David has seen the new eye draw the old world at both scales, with numbers from two cards. That is the
moment to fix the design, or change it.

---

## 8. The roadmap

Each step has a gate that can be measured. From step 4 on, the game runs and the look is intact at every step, with
old Incus Artus as the check.

| Step | What | The gate |
|---|---|---|
| 0 | **Safety.** The private GitHub repository David made on 2026-11-11 connected and pushed (after his word on `Art/`). A USB backup. The tag `old-final`. **Golden references taken from old:** pictures at fixed places, times of day and settings; the marks' flight logs; the default preset's maps and its river network; sample columns. | The backup reads back; the references are listed in `master/`. |
| 1 | **The experiments** of part 7, in order. | Each one's own pass mark. David reads the results; the design is fixed or changed. |
| 2 | **The master documents.** David cuts the spec. The plan is written from this review as he amends it. | He signs both. |
| 3 | **Packing, and the skeleton.** Old moved to `old\` with its own history and its own build folder. The new runner (no Bevy, three lanes), the stage graph declared as data, the map drawn from it, the control-room page, the instruments, the golden-picture harness. | An empty frame measured (the processor under 0.3 ms, input to photon as build 67's); the map generated from the declaration; old still runs from its new place. |
| 4 | **The eye, on old's data.** Experiment 5's and 6's program grown into the renderer: bricks at every level, the light lane, water and glass, pieces, textures, objects, glows, far lights. Its world comes from old's bake file and chunks through a loader (an instrument, not a stand-in for anything the player sees). | Pictures against old's golden references at every fixed place, within a stated tolerance. The marks: the laptop under 4.2 ms at the 99th percentile on the flight and at all three standing marks; the small card's numbers recorded. **David walks the old test world through the new eye.** |
| 5 | **The tablet.** Its format, the doorway (made, plus changes), chunks from it, edits, re-summing, saving. Filled first by old's own rules, so real worlds run before any rule is rewritten. | The same columns as old's, to the block; digging and placing; save and load; the size bound held. |
| 6 | **The rules, stage by stage, with their editor.** Each stage of part 5 written for the card with its view in the new World Compiler, plans first. | Per stage: David's eye, old beside new, on his default preset; the same hashes on both of the laptop's cards; the whole world's bake time. |
| 7 | **Making a world as the player meets it.** The whole bake before play, the stage magic over it, the rejector if wanted. | A new world in the time David sets; no frame drops after entering. |
| 8 | **The game's systems,** in the spec's order. | The spec's. |

Something to see early and often: step 4 gives the look back before a line of world generation is rewritten, and
every step after it is walked in.

---

## 9. How we work

### The map, drawn from the code

One declaration, in the code, as data: every **stage** of world making, every **pass** of the frame, every
**lane**, every **store** (what it holds, how big), and what each reads and writes.

- **The game is wired from it.** The scheduler runs what the declaration lists, in the order its reads and writes
  imply. A pass that isn't declared doesn't run; a pass that reads what nothing writes fails the build. So the map
  can't drift from the wiring: they are the same thing.
- **`master/map.md` is generated from it** on every build, as a picture (Mermaid, which the workspace already
  previews) and a table. A test fails if the committed map is stale.
- **The control room** is the same graph drawn live in the game: each box with its time on the card and on the
  processor, its memory, what it has waiting, its last fault, against its budget: green, amber, red. This is the
  power-plant screen David asked for, and it is where the budget readout he wanted lives ("a budget readout shows
  when the wall is coming").
- Each box carries one line of what it is, a link to its tests, and its budget. That is the whole of the map's
  prose.

### The plan, the spec, the journal

- `master/plan.md`: the roadmap as a nested checklist, each line with its gate and, when ticked, the log that
  proved it. Counsel keeps it with David; Builder ticks it.
- `master/spec.md`: the game. David's to cut.
- The journal (today's `docs/` and `vision/`) is frozen history with David's words. A decision gets one line in the
  living documents and a pointer back.
- Comments in code say what and why in a line or two. The story of how it got there goes in the journal.

### Tests

- **Promises:** a cell is the sum of its blocks; the cells are the sum of the world as it is; made twice, the same;
  the same seed gives the same hashes on two cards; everything is read through the doorway.
- **Pictures:** golden pictures at fixed places, first from old, then the new engine's own once David has signed a
  view.
- **Budgets:** each mark has a budget in the declaration; a flight over budget is reported as a failed gate, not a
  number to interpret.

### Instruments

- The flight and the three standing marks; headers that record every switch, the commit and whether it was clean,
  the card, the present mode granted.
- The card's time counting the lanes. Timings inside a pass without splitting it: wgpu 30 has a feature for this
  (`TIMESTAMP_QUERY_INSIDE_PASSES`, `features.rs:747`) *[checked that it exists; to be tried on DirectX 12]*.
- The twin, made by a script from the program as it is that day.
- The small card flown whenever the renderer changes shape.
- Nsight when a program's shape changes. Small programs make its readings rarer to need.

### The team

David's question: is he using AI agents properly for a large project he means to ship? Mostly yes. The roles are
right, his decisions are kept in his own words, and the measuring discipline of the last week is better than most
teams manage. What I'd change:

1. **A living truth short enough to read whole.** A new session today must read thousands of lines to learn what is
   true. The generated map, the plan and the spec fix that.
2. **No number without its log.** Two of the review's headline figures had no measurement behind them. A figure in a
   living document carries the file it came from.
3. **The pass mark before the flight.** Each experiment says beforehand what result changes the plan. The dyno's
   ledger already records what a step lost to; this is the other half.
4. **One Builder** (David's choice, and right for now). She may send out short-lived helpers that only read. Counsel
   keeps the plan and never writes code. The Oracle is called at forks, with a fresh mind, and reads `master/` first.
5. **A fresh reader before a structural step joins.** A session that didn't write the change reads it against the
   map. Counsel can do this.
6. **Short memories.** Builder's resume note is 32 KB. What is true belongs in `master/`; a memory note is for
   habits and warnings.
7. **One thing asked of David at a time,** as he has asked. A gate is one look.
8. **The work off the laptop:** the GitHub repository, pushed at every build number.

---

## 10. Questions for David

Those his answers of 2026-11-11 left open, and the new ones this review raises.

1. **Crisp or soft far cells,** after experiment 2. It decides the bricks' form and their memory.
2. **Rivers and the sea's level.** In the rebuild, are the river planner and the shore scheme carried over as they
   are first (my advice), and rearranged afterwards as a design step of their own? Or rearranged on the way through?
3. **The moons.** Does the brighter moon cast shadows like a faint sun, or is night one even dimness?
4. **People far off.** Past where blocks are blocks (about 336 blocks at today's look), figures are a few pixels
   tall. Drawn as they are, they would be sharper than the 2 × 2 or 4 × 4 world behind them. Is that welcome (they
   stay easy to find), or should they be drawn into the same squares?
5. **Leaves far off.** Cells of leaves cover in part, so a far forest's edge is a little see-through, as the
   averages make it. Is that the look, or should a canopy read as solid from afar?
6. **The rejector.** If a handful of worlds are made and the best kept, what makes one best? (Not needed until
   step 7.)
7. **`Art/` in the online copy:** all of it, or his own work only?
8. **The control room** as a page in the game's own panel (egui, as the editor is): acceptable as the first form of
   the live map?
9. **The small card.** Which one is in the laptop? It decides how much its numbers say about a 980 Ti.
10. **After experiment 6:** does the new eye's picture of the old world pass his look? That is the gate the rebuild
    hangs on.

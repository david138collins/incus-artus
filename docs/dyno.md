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

### The test world with its rivers (2026-11-09, build 102)

`nice25 test` was made without its rivers and lakes (and its climate): the automatic render rendered as soon as the
map compiled, before the climate had carved them [David: "if you're using the editor, you have to wait for it to
finish with rivers, otherwise you spawn without thwem"]. It waits for them now, and the test world was made again
with them (its bake 689 s, 209 MB). **Everything before this was flown on the world without rivers;** steps are
judged in pairs on one world, so what's kept stands, but the baseline from here is this:

| Build 102, with rivers (`logs/dyno-1794267791.txt`, `-1794268014.txt`) | Median | 95th | 99th | Card median |
|---|---|---|---|---|
| **Whole flight** | **6.48 / 6.63 (153 fps)** | 16.58 / 16.36 | 21.58 / 21.67 | 5.08 / 5.35 |
| Over the land | 12.70 / 12.97 | 17.7 / 16.6 | 19.8 / 19.2 | 11.2 / 11.6 |
| Round the peak (rivers and lakes on the circle) | 6.79 / 6.85 | 19.9 | 25.7 / 24.7 | 4.68 / 5.04 |
| Out to sea | 9.26 / 9.57 | 16.4 / 16.3 | 20.4 / 20.2 | 8.4 / 8.9 |
| Quick | 3.71 / 4.18 | 8.9 / 9.0 | 12.5 / 12.2 | 1.37 / 1.67 |
| Home | 7.53 / 7.39 | 12.4 / 12.6 | 14.8 / 15.2 | 6.03 / 6.17 |

Against the same build without rivers (5.39 ms median, the card 4.67): the median 17% more, **round the peak the
card's time three and a half times** (1.4 to 4.8 ms). Water near the eye is dear: the near walk goes on through water
a block at a time, no skipping in a medium, till it's dimmed out some 25 blocks down (build 98). Water's moved to the
front of the order.

### Step 4, tried and dropped, and how steps are measured from here (2026-11-09)

**Runs of water crossed whole** (the near walk jumping a live cell of 4 or 2 that's all water, the dimming and tint
added up exactly; the same picture as water walked a block at a time). Flown in its own build, on against off: the
median +7% (5.44 to 5.82 ms), the card +7.5%, round the peak and at sea 10 to 18% slower: the jump's lookups cost
more than the blocks they saved. Dropped (reverted, 8994b6a).

**What it showed about measuring:** its build flew differently from the build before it **even with the jump off**:
over the land the card 10.7 to 15.0 ms, round the peak 4.8 to 1.7, at sea 8.2 to 6.4; the median better (6.14 to
5.44 ms) and the 99th percentile much worse (20.7 to 35.6 ms). The flight with the build before's shader, flown
straight after on the same world, gave the build before's numbers again (`logs/dyno-1794270670.txt`): not the
world, the heat or the background, but the shader itself. With the jump off the two shaders do the same; what
changed is how the card's compiler laid out the whole march round the added lines (where it keeps what, how it
orders the work). The march is one big shader, and added code, run or not, can move each view's cost by tens of
percent either way (the second step's 8% was the same thing).
- **So from here, a step is flown against the build before it**, both whole builds; a switch inside one build
  measures only what running the step costs, not what compiling it in costs.
- **And a lead:** the same march, laid out otherwise, did round the peak (rivers and lakes) in a third of the
  time. Finding what the compiler did there (Nsight's view of the compiled shader: its registers, where it waits)
  may be worth more than any one step.
- *On file:* looking down into a lake, the near walk as it skips air draws the water a few steps of colour darker
  than every block walked (41,600 pixels of the test's view, 12 steps at most), in the shader as it was before the
  water step too: to find.
  - **Found and fixed** (2026-11-09). The test's lake views were inside a hill: the map's rows run south, so the
    lake is at (152, −268), not (−160, −272). What they measured in step 4's build isn't known; today's shader
    draws them the same both ways. Placed over the lake, a real difference showed **past the blocks made** (the
    blocks are made round (0, 0) in the test): 62,400 of the view's 64,000 pixels, 24 steps of colour at most.
  - **What it was:** the near walk skips the air over a column's top in one go, coarsest column first. Looking
    steeply down, the skip ran from the eye straight to the lake's top. Where that landed in a column whose blocks
    aren't made, the walk handed over to the march there, on the water's surface. The march reads averaged cells,
    whose surface is a soft edge reaching above the real one, and gathers it coming down to it. Started on the
    surface, it missed the edge's upper half. Every block walked hands over where the blocks end instead, in the
    air, so the march gathers the whole edge, as it does everywhere farther off.
  - **In the game:** a band just past the near blocks, looking down at water or flat ground: a shade off from the
    same surface farther away. Rays meeting the surface between the blocks' window's edge (48 to 64 blocks off,
    the window going by tiles of 16) and 64 blocks along the ray hand over on the surface; farther ones hand over in
    the air. Also wherever the near blocks aren't made yet, flying fast.
  - **The fix:** a skip never lands in a column whose blocks aren't made. A finer column's skip takes it there,
    or a block at a time, as every block walked does. The test has three views over water now: down into a lake
    inside the blocks, down into one past them, and along it from its shore. All draw the same as every block
    walked, to the pixel. **Its cost:** none to be seen, flown with the twin against the build before (the medians
    −2% to +3% by leg, either way; the twin against itself moves as much). Commit 1868703.
  - **And the card tests take turns** (found running the suite for it): run all at once, each with a device of
    its own, six of sight's picture tests failed together that pass one at a time. The suite: 428 passed.

### The first look inside the march (Nsight Graphics, 2026-11-09)

A GPU trace of the flight out at sea (three frames at the card's own clocks; the trace file is kept outside the
project, in Builder's scratchpad), the march as it was after the near walk's handover fix:

- **Few pixels in flight:** during the march the card keeps 13 to 17% of the most it can (some 6 to 8 groups of 32
  a unit, of 48). If the march needs the most working room (registers) a pixel can have, only 8 groups fit: 16.7%.
  The measured share sits just under that, as long-running groups do. **To confirm** in Nsight's window (Shader
  Pipelines, the Registers column for `march_pass`): the trace file keeps it in a form only the window reads.
- **Waiting, not working:** the card's arithmetic runs at 23 to 30% of its peak, its memory at under 1% of its
  bandwidth, its caches at 4 to 6%. The march's top states: **"Wait"** (a group waiting on its own last sum, with
  too few others to switch to), "Selected" (working), and **"No Instructions"** (waiting for the march's own code
  to be fetched: it's too big for the card's instruction cache). Memory waits come well after.
- **Half of each group idle:** on average 14 of a group's 32 pixels take each step (44%): neighbouring rays go
  different ways.
- **No one hot loop:** the biggest single line takes 4.7% of the samples, the twenty biggest about 12% together.
  The compiled march runs to some 13,000 lines of the card's intermediate code.
- **What it explains:** step 4's finding (added lines, run or not, moving each view's cost by tens of percent) is
  the instruction cache and the working room: where the compiler puts code decides how often its fetch misses, and
  every line the march might run counts against the pixels in flight. The split's march passes, 35 to 58% faster
  alone, were the same thing seen from the other side.
- **So the levers, in order:** the march's working room and size first (fewer registers, less code: each a twin
  flight, and a trace to see the pixels in flight move); then the groups' idle halves (rays alike in a group).
  Memory layout matters less than planned: the march isn't waiting on memory.

**Running Nsight from here** (Builder's notes): the command line, `ngfx.exe --activity "GPU Trace Profiler"`,
started from its own folder; **never `--platform`** (Nsight's windows take it as their own setting and fail:
"no Qt platform plugin could be initialized"); `--set-gpu-clocks unaltered`, `--real-time-shader-profiler`,
`--auto-export`. Under it, the game's DXC had picked up Nsight's newer validator ("DXIL container mismatch", the card
lost at the first shader): the game now loads the SDK's validator itself, by its path. The trace file opens in
Nsight's MCP server (`ngfx-mcp.exe`, on 127.0.0.1 only), which reads its shader profiler rows and hotspots.

### The march in stages: the plan (2026-11-09)

David's decision [via Counsel, `decisions.md`]: "the idea of a monolithic renderer was as misguided as the idea of
two renderers and two scales, so lets go the other way. i believe the hologram enables atomized processing, so lets
atomize the megakernal which apparently i was fated to repeat by not knowing history." The field's name for it is
wavefront rendering (Laine, Karras and Aila, "Megakernels Considered Harmful", 2013: one big program with few pixels
in flight, a cold instruction cache and idle lanes, answered with small programs per stage handing work on through
memory, each running only the pixels that need it).

**Today,** for each layer (8 × 8, 4 × 4, 2 × 2), one program does everything for each pixel: the march (the near
blocks, then the far cells), the texture, sky light (five cones), the sun's shadow (a walk toward the sun), placed
light, fog, and the measuring views. 128 registers for all of it, about 30,000 instructions.

**The stages,** each frame:
1. **Hit,** a layer at a time, coarse to fine (each finer layer starts from the one above, as now): the march
   alone. Where each pixel's ray meets something and what it gathered on the way, written to a record per pixel
   (some 64 bytes: the colour and its sun and sky parts in half floats, where, which way the face looks, how far),
   with the layer's distances and "worth it" for the next layer.
2. **Sky,** coarse to fine: only the pixels whose surface needs sky light of its own (a finer pixel still takes its
   parent's where it's the same surface about as far off): the five cones. Writes how open it is to the sky.
3. **Shadow,** all layers in one go: only the pixels whose surface faces the sun: the walk toward it. Writes how
   much sun reaches it.
4. **Light,** all layers: every marched pixel's colour from its record, the shadow and sky light put in, then
   placed light and fog, into the layer's picture. The measuring views live here, never in the march.
5. **Upscale** and objects, as now.

**Why it should be faster:** each program is small, with its own registers and its own code in the instruction
cache (split before, the march passes alone ran 35 to 58% faster). Sky and shadow run only on the pixels that need
them, packed together (a list per stage, run by indirect dispatch), so no lane in a group idles while its neighbours
work. **What it costs:** the records (some 85 MB on the card at David's screen; their traffic about 1% of the card's
memory bandwidth at 150 fps, where the march uses under 1% now) and a few more waits between stages. The split lost
its median to exactly that (+13%, the worst frames −12%), with sky and shadow still in its light pass.

**Built in this order, each its own commit,** flown with the twin, the picture checked (the same as the one program
to a step of a colour, the records' half floats, and every-block truth for the march):
- a. **The twin for structures:** every other timed frame drawn by the stages in place of the one program (the twin
  as built swaps one shader file for another of the same shape; the stages are another shape).
- b. **Hit and light** (the split, refreshed): the stages' baseline.
- c. **Shadow its own stage,** packed.
- d. **Sky its own stage,** packed.
- e. **Fewer waits:** light and shadow for all layers in one dispatch each.
- f. **The measuring views out of the march.**

Nsight after c and d (the registers per stage: David's two-minute look). Then step 4, tiles by type (sky, far, near
blocks, water, seams, each with its own lean march), and step 5, the ray queue, only if lanes still idle.
**Sizes:** about 7 to 9 hours of Builder's, a 4-minute flight after each step, one Nsight look from David.

**As built so far** (2026-11-09), flown against the split, frame by frame:
- [x] a. The twin as another shape (`INCUS_SIGHT_TWIN=split` or `=stages`; with `INCUS_SIGHT_SPLIT` the base is the
  split). The split against the one program: the median +9.0%, 95th −7.0%, 99th −12.3%; over the land −12 to −15%
  at every percentile.
- [x] c, unpacked: the sun's shadow its own pass. Against the split: the median +4.1%, 95th +1.0%, 99th −0.8%.
- [x] d, unpacked: sky light its own pass. Against the split: the median +6.7%, 95th +0.5%, 99th −2.8%; over the
  land −1 to −3% at every percentile; the lightest views pay the extra passes (+29% on the quick leg, under 1 ms).
- **Each stage timed** (`INCUS_DYNO_PASS_TIMES`, medians, ms): over the whole flight the march's coarse layers
  (8 × 8 and 4 × 4, 1.3 each) are the typical frame; over the land, light and shadow on the near blocks' 2 × 2
  layer (1.5 and 1.7) are the heavy frame's cost, 5.9 of its 9.1 ms light and shadow in all. Every 2 × 2 pixel
  walks its own ray to the sun and casts five sky cones, where one block face covers hundreds of pixels that all
  get the same answer. So packing helps less than planned: the cost is repeated work, not idle lanes.

**The order from here** (David, 2026-11-09, via Counsel: "other than that i agree"):
1. [x] Sky its own pass (d above).
2. [x] **Light per face near the eye:** a shadow and a sky value worked out once for each block face the near walk
   shows, at the face's middle, shared by every pixel showing it (his blocky look: "blocky and quantized lighting is
   good all around", "rough shadows for everything"). A table of the frame's faces on the card, keyed by the block
   and its face, filled by the pixels that show them, lit once each. Behind a Render-page switch for his eye
   (induction: it changes the picture). **David's look** (build 103): "merely slightly different, not better or
   worse." Against the stages without it: the 99th −20.8%, over the land −13.5 / −22.6 / −25.2%.
3. [x] **Fewer handovers.** Built in three goes, each flown against the game as it runs today (one pass):
   - Lists: after each layer's march its live pixels listed, the passes after it launched by the count over them
     only; the light per face over the faces found. First flown worse (the median +24.8%): wgpu runs a checking
     program before every dispatch launched by a count (a pass and two hand-offs each, 13 a frame). Taken off for the
     game (our counts are bounded; the tests keep it): the median +10.1%, the 95th −20.1%, the 99th −35.6%.
   - The face lookup reads before it takes a slot (a big face's pixels had each taken it in turn).
   - **Shadow, sky and light in one shading pass a layer** (apart, they read and wrote each record three times for
     little work on the lighter views), the faces gathered by the listing pass: ten passes a frame, not twenty.
     **The median +6.5%, the 95th −18.6%, the 99th −36.0%** (the card's 99th 17.8 down to 11.4 ms); over the land
     −32.0 / −44.0 / −46.7%; round the peak the 95th −43.3%; at sea −11.4 / −13.1 / −33.2%; the lightest views level
     (+1.7%); the home leg's median +14%. **On a clear day** (fog off): +6.8 / −18.3 / −34.0%, the same story.
   - [-] The sun's cone as the sixth in the sky's loop (one copy of a cone's code in the shading pass, not two): the
     same picture, but the median +9.1% (against +6.5%): the compiler's lottery again. Dropped.
   - Each layer timed against the one program (whole flight, ms): the coarse layers' march alone is a little
     cheaper than the one program's march and light together, but their shading adds the difference back; the
     near layer is where the stages win (over the land 5.2 ms in one program, about 1.4 ms in stages).
   - **The game's drawing since build 104.** Asked whether the stages with light per face become the normal
     drawing (the median +6.5%, the 99th −36%), David: "yes" (2026-11-10, via Counsel; `decisions.md`). Both on
     unless chosen; the Render page's two checks stay, to flip back. `INCUS_SIGHT_ONE_PASS` flies the one pass to
     measure against (`INCUS_SIGHT_SPLIT` the split). The tests draw as the game does, in stages with light per
     face; the stages against the one pass keep their own test, without light per face.
4. Then the far: the typical frame's coarse horizon rays need an answer that doesn't lean on fog (below). Texture
   units for the far cells' blend, empty air skipped smarter for grazing rays (solved jumps, relaxed cone
   stepping), the sea as a plane.
   **The first probes** (2026-11-10, at build 104; each a twin flown every other frame against the game, the whole
   flight's median / 95th / 99th). The twin against itself in the stages' shape reads level (+0.0 / −0.8 / −1.1%; the
   legs within 1.5%, the quick leg's 1 ms median ±8%): the instrument works in the game's shape.
   - No hand-over blend (one level read where two are mixed, the last 40% of each level's stretch): −9.7 / −12.5 /
     −10.4%; round the peak and home −11% on the median, over the land −1.3%. Not to ship (the mix is what keeps the
     level changes from drawing rings, David's build 69), but the second read costs a tenth of the sight.
   - Air strides twice as long: −3.2 / −4.3 / −4.8%. The steps through air are a small part.
   - [-] **The deeper air skip** (built: after each skip in a row the next tries a column coarser still, up to eight
     levels over, never past where the level changes; the same picture to the pixel in all seven test views):
     −1.6 / +2.1 / +2.0%, noise. Dropped: skipping the air already costs little.
   - So the far march's time is where rays meet surfaces: the fine steps there, each reading the eight cells round
     it (four columns' entries, eight cells), and both levels in the hand-over stretch.
   - **David's idea** (via Counsel): "oh yeah! maybe we can make the texture units handle blending two pyramid
     levels." Weighed: the shape fits (each level's cells twice the last's every way, a mip chain; the hardware's
     trilinear filter is the blend within a level, with colours kept weighted by their surface; a fractional level
     does the hand-over in the same fetch). The store doesn't, yet: its cells are bands in a pool, a column's cells
     round its surface only and neighbouring columns not neighbours in memory, so nothing the hardware can filter
     across; a level as one texture would be hundreds of millions of cells (windows some 700 columns a side, the
     world 1,024 tall). The way in is bricks (small blocks of cells where there's surface, with a border for the
     filter, found through a table as the columns are now): a rework of the store and every read of it, more memory
     than the bands; the fine levels' windows can't share one mip chain with the bake's, so there two fetches and a
     mix. Its ceiling is measured first: the blend reading only the one cell a point is in.
   - **The second probes** (David: "yes, take a look with nsight, and think about how we can reduce our registers"):
     - **The one-cell blend** (each read takes only the cell a point is in: one column's entry and one cell, not four
       and eight): **−41.4 / −40.0 / −39.0%**, every leg alike (over the land −40%, at sea −38%, home −46%). A
       ceiling (blocky cells end rays a little sooner too), but the eight-cell blend is the biggest single cost there
       is, and not the far march's alone: the sun's and the sky's rays step through it too (over the land, where the
       far-march probes moved little, it moved 40%).
     - Steps by a surface twice as long: −7.8 / −5.7 / −4.7%. No walk back to the last air: −4.3 / −3.4 / −4.2%.
     - [-] The blend's reads issued together (the four entries, then the eight cells; the same picture to the
       pixel): −1.8 / −1.0 / −0.4%, noise. Dropped: the blend doesn't wait on its reads; its cost is the unpacking
       and mixing of twelve values from each of eight cells, and its code.
   - **The Nsight look at build 104** (two GPU traces, over the land and at sea; each pass's share of the shader
     samples, land / sea): `march_hit` 32 / 62%, `shade_staged` (each pixel's shadow and sky rays) 29 / 17%,
     `face_light_staged` (light per face) 19 / 9%, the near ground's tiles made as the camera flies 15 / 4%, the
     listing 3 / 3%, the upscale 2 / 3%. Pixels in flight about 10% of what the card holds (9% at sea); the top
     stall in every big pass **"Wait"** (a group waiting on its own last sum, too few others to switch to: too many
     registers a pixel); the instruction cache's misses, the old one-pass march's top stall, down to 0.5% (the
     stages' programs fit). Arithmetic at 16 to 18% of peak, memory light. And some 2 GB a second written to the
     card's local memory, almost none read back: two small constant tables (a face's way, the levels view's tints)
     kept there and filled out every pixel. The registers themselves show only in Nsight's window (asked of David).
   - **The registers** (David, from Nsight's window, the sea trace's Shader Pipelines; groups of 32 a unit can
     hold, of 48): `march_hit` **125** (16 groups: a third of the card; the old one pass was 128), `shade_staged` 72
     (28), `face_light_staged` 56 (36), the near ground's tiles (`sight_grounds_pass`) **255** (8; 24,100
     instructions: the world's own column worked out on the card, spilling to memory: the local writes above are
     its, 8× more over the land, where it does about 4× the work, not the two tables'), the listing 33 (48), the
     upscale 20, the start 48 (40), the cells' passes 168 (12). So the stages took the shading out but not the march's
     room: the near walk and the far march together hold it at a third. Under 80 it would hold half, under 64 two
     thirds. The ~10% measured in flight, against that third, is the drains and tails of twelve passes a frame
     (each waiting for the last to empty, the slowest rays holding each one's end).
   - [-] Flown, the same picture to the pixel, noise both: the sun's and the sky's rays with a lean blend of their
     own (+0.2 / +0.8 / +0.8%: the compiler was already leaving out the colour and faces they don't use), and no
     tables picked from as the shader runs (+0.2 / +0.2 / −0.4%). Kept on file, not built in: code that doesn't pay
     has cost before.
   - **The one-cell blend's 41% split:** the sun's and the sky's rays alone −11.3 / −8.2 / −9.4% (over the land
     −16%); the far march alone **−28.0 / −31.2 / −26.1%**. Together the whole. So three quarters of what the
     texture units could win is in the far march's own blend: the first bricks worth building hold all it reads
     (how solid, how watery, the colour, the faces), not the rays' two numbers alone.
   - **David on the near ground's pass** (via Counsel): "yes ... as long as it looks right, which is something we
     know now. seems like the whole thing *can* be carved into a gpu shape." It may be cut into stages: what it works
     out unchanged, the card's columns matching the processor's to the bit, each stage only handing forward.
   - The order (Counsel's, unless the numbers say otherwise): the march split (the near walk and the far march each
     a pass, each with only its own room), then the texture units, then the near ground's stages. The split sized
     first: a trace each with a probe as the twin (the far march alone; the near walk alone), their registers read
     beside the whole march's.
   - **The split, sized** (David's readouts, the sea traces): the far march alone **121** registers (16 groups),
     9,270 instructions; the near walk alone **80** (24 groups, half the card), 10,800; both 125. The far march's
     room is its own: the split would free the walk, which out at sea took 3% of the card beside the whole march's
     49%. It waits.
   - **Bricks, counted** (`bake::tests::bricks_the_bake_would_take`, the test world's bake, cells of 32 up: 11.2
     million cells in bands): bricks of 4 a side take 2.6 times the cells, 5.5 with the one-cell border the
     hardware's blend needs; of 8 with it, 7.3 (a brick holds the air and rock round a thin surface; at the coarse
     levels every brick column is one brick tall). How solid and how watery alone (two bytes a cell): 122 to 164 MB.
     The whole look the far march blends (twelve numbers): 735 MB to 1 GB for the bake alone, too much for most
     cards. So the texture units can carry the density (the rays, the far march's coverage), not the look.
   - **The outside read** (`vision/dyno-audit-2026-11-10.md`, at David's ask; its leads priced as twins):
     - [x] **`live_slot` without whole-number remainders** (its lead 6): a column's place in its wrapping window by
       a float floor and a fix-up either way, not `((c % side) + side) % side` twice: four divisions by a number the
       compiler can't know for every column read at a live level, sixteen a blend, four a block of the near walk,
       each a long chain of steps (the shape of the top stall). **−11.9 / −8.0 / −8.8%**; over the land −15.5%,
       round the peak −9.1%, at sea −7.9%, home −7.0%. The same picture to the pixel. **In the game, build 105.**
     - [-] The far march's last sample keeping its light, not its six face shares (lead 7): −1.5 / −1.2 / −2.0%
       alone, nothing on top of the above (−11.5 / −8.6 / −8.7% together). Kept on file.
     - [-] The sun's and the sky's rays remembering a failed skip (lead 4): +1.5 / −0.9 / −1.7%, noise. Dropped.
     - **Light per face kept between frames** (lead 1; David: "i think quarter second light wont really be visible
       with coarse lit blocks anways, especially if near actors work faster. terrain light almost needs zero
       dynamism for gameplay"; light that moves with actors stays prompt): priced by lighting one face in eight a
       frame, −1.4 / −2.2 / −2.8%; over the land −6.1%, round the peak's 95th and 99th −9.7 / −10.4%. A ceiling for
       the lighting part. Behind a Render-page switch if built.
     - [x] **The ground columns shared by the live levels** (its lead 2; David: "her order is right, she can
       proceed without me looking"): the coarsest level's tiles keep the grounds the card worked out for them (each
       block column's ground and water, two 16-bit numbers, the tile and three columns round), and a finer tile
       inside one takes its grounds from there, put on the card in place of the grounds pass (`SharedGrounds`,
       `fill_tile_sharing`, `cells_by_given`); kept only for the coarse tiles the next level's window reaches (208
       of them flying, about 57 MB on the processor's side). The same to the bit (36 finer tiles at every level, at
       a coarse tile's corners and by caves and a river: `sight_s_shared_grounds_make_the_same_tiles`). Flown build
       against build, off and on twice each (`INCUS_SIGHT_NO_SHARED_GROUNDS` turns it off; the offs agree within
       1%): 96% of the finer tiles took their grounds from the coarse ones, and **the tiles made in a flight went
       from 24,900 to 41,500** (2.2 million cells on the card against 1.4): the live levels now keep up with the
       flight, so the march falls back to coarser levels less. **The median −9.5 / −8.6%**; over the land −4.5 /
       −3.9% and the 99th −8.2%; round the peak −28% (the card's own sight −31 to −38%: fuller windows); the
       quick leg −35 / −28%; home's 99th −11 / −15%. But at sea the 95th and 99th +4 to +5% (more tiles made a
       second, more cells made in some frames), and the whole flight's 95th and 99th +1.5 to +2.2%. **In the game,
       build 106.** The lane's pace (how many tiles at once) is the knob for the sea's tails, later.
     - [-] **The near ground's grounds pass in stages** (David: "yes ... as long as it looks right"; each stage
       handing only forward, the processor's columns to the bit). Cut in two (the ground before the rivers and
       lakes, then carved): the first still 255 registers and spilling, with 19,700 of the 24,100 instructions; the
       carve alone 4,480. Cut in four (the maps blended to each column, left as ten numbers a column; the parts: the
       land or sea, the mountains, the peaks; put together with the crags, the torn coast and the detail; then
       carved): David's readout, the parts **255** (8 groups, 12,600 instructions, still spilling), the putting
       together 53 (36 groups), the carve 96 (20 groups). Every way the processor's to the bit (9,612 columns by
       caves, a river, a lake and the map's edge). **Flown against the single pass, alternating twice: lost.** Each
       stage waited for apart (four round trips a piece): the median +5%, the land's 99th +5 to +7%. Every stage of
       a piece sent at once, waited for once: still the median +1.7 to +1.9%, round the peak +8 to +13%, the rest
       noise. The hand-overs (14 numbers a column out and back) and the extra dispatches cost more than the lighter
       stages saved, while the parts still spill. **Dropped**; the patch kept on file (Builder's scratchpad,
       `grounds_stages_2026-11-10.patch`), and the test kept: `sight_s_grounds_are_the_processors_columns`. If it's
       tried again, the parts cut once more (the ridges apart from the peaks) is where the 255 is.
     - [x] **The tile lane's pace** (`MOST_FILLING`; Counsel, for David's "even beats fast": since build 106 the lane
       made two thirds more tiles a flight, and the sea's 95th and 99th rose 4 to 5%). Flown, 8 fine and 2 coarse
       tiles at once (the base, three times, within 1.4%), 8 and 1, 4 and 2, and build 105's way (no shared grounds,
       8 and 2). Against build 105's way:

       | | median | 95th | 99th | sea 95th / 99th | near levels filled | coarse filled |
       |---|---|---|---|---|---|---|
       | build 106 (8 and 2) | −11.7% | +0.9% | +1.9% | +4.7 / +4.8% | about twice | +22 to +33% |
       | 8 and 1 | −18.6% | −2.4% | −3.1% | −0.3 / −0.8% | about 2.5 times | −25 to −29% |
       | **4 and 2** | **−25.0%** | **−7.9%** | **−8.2%** | **−4.5 / −5.2%** | **the same** | **+30 to +46%** |

       (Filled: the tiles made at each level, three moments of the same flight.) **4 and 2 in the game, build 107**:
       the near levels fill as fast as before build 106, the coarse ones fuller, every frame faster. 8 and 1 traded
       the coarse levels for nothing the near ones needed. The lane's pace is a look at loading, not at the picture
       once loaded; David may want a feel of the near ground filling.
     - **David's look at build 107** (2026-11-10): "it looks good, and definitely is becoming more even". First in a
       world without a kept bake (the bake making itself all through his look: the game paced evenly at 76 to 92
       frames a second, as the dyno measured while baking, 100); then in the test world: high up 216, near the ground
       138, looking down 233, looking up 170. The log: after a load the near levels filled from nothing in about 100
       s (the frames about 100 a second), then 156 to 175, the far levels (8s and 16s, two at a time) still filling
       five minutes on, the frames 105 to 170 by how big the tiles being made were. So the first minutes of a world
       are the live levels being made: **the first impression** (section 5) is where the fix goes. And the shared
       grounds kept 555 coarse tiles standing still (about 150 MB on the processor's side, against 57 MB flying):
       to trim.
     - [-] **Light per face kept between frames** (lead 1, built: faces keyed by their own places, the table kept,
       each face lit when it first shows and again a sixteenth of the frames, stale slots taken back; behind a
       Render-page switch; twenty frames the same to the pixel as lit afresh, the eye still and moving; the suite and
       sight's 21 picture tests passed). Flown kept against afresh, alternating twice: the median +10.8% and +0.6%,
       the quick leg +18 to +20% both times (the card's +30 to +36%), over the land level. **Dropped. What it lost to:**
       on the dyno's route the eye flies a hundred blocks a second and more, so the faces near it are mostly new each
       frame and lit anyway; the table's bookkeeping (a pixel's look-up marking the face seen, the places checked)
       cost more than the relighting it saved, and with the switch off the build ran about 6 to 7% slower than build
       107's own flights (single flights, a hint, not paired). It may pay walking or standing, which the dyno doesn't
       fly. The patch on file (Builder's scratchpad, `faces_kept_2026-11-10.patch`).
     - **The first impression, measured** (David's order: "just do those in order": the first impression, the
       shared grounds' memory, then the far march's blend). Standing still at the test world's spawn, two blocks
       over the ground, the flights' settings, three minutes (the soak at speed 0): **113 to 117 fps from start to
       finish**, the card's sight 8.1 ms throughout, no rise when the near levels finish filling. A trace there 40 s
       in: the tile passes under 3% of the card; the march 41%, the shading (each pixel's sun and sky rays) 35%,
       light per face 13.5%. So the first minutes aren't the levels filling: they're **the view near the ground**,
       about 8 ms of the card where a flight's typical frame is 4.5. That view is the first thing a player sees.
     - [-] **The shared grounds trimmed** (a coarse tile let go once every finer tile wanted inside it was made, and
       once a second, not every frame): standing still the cache still grew while the far levels filled (398 tiles at
       150 s), and in flight sharing fell from 96% to 36% (6,000 tiles given, 10,600 working their own out), the
       tiles made from 24,500 to 17,600, the median +11%. **Dropped. What it lost to:** each finer level needs a
       coarse tile's grounds in turn as its window sweeps over it; let go when the level now over it is done, the
       next one comes to nothing. Kept as it was (the next level's window and a coarse tile round it: 57 MB flying,
       up to 150 MB standing still with the windows full). A trim that would keep the sweep: a cap by distance from
       the eye, or the grounds packed smaller (they're smooth heights).
     - **The view near the ground, split** (two traces at the spawn, a probe as the twin): the near walk about 15%
       of the march a frame there, the far march the rest (the coarse layers' rays and all past where a block is a
       pixel go through the cells). **David's idea** ("is it possible to just shoot less rays in the areas we get
       texels instead of pixels?"; Counsel's form: a fine pixel whose parent and its eight neighbours met the same
       block plane intersects it, reading its own block and texel, where a block spans more than their gap) is sound
       and the same picture, but its ceiling there is that 15% (about 6% of the frame, less: only faces' insides
       inherit). After the far march's work.
     - [-] The blend leaving at once in clear air (same picture to the pixel): +9.1% on the median. **Lost to** added
       code in the blend (its checks, the columns read twice where it isn't air).
     - **The narrower hand-over** (`BLEND_FROM` 0.8, the outside read's lead 3; a twin, priced): the whole flight
       −2.8 / −6.7 / −5.5%, at sea −7.1 / −7.1 / −5.8%, home −5.7 / −6.3 / −9.1%, over the land level. A trade for
       David's eye (rings over flat water, build 69). [A first flight read +12.8%: its twin file was made before the
       `live_slot` fix went in, so it carried the old one. **A twin is made from the march as it is the day it
       flies.**]
5. **Rays ending where the fog is total: last, if at all.** David: "i kind of feel like 'rays end where fog is
   total' is such an obvious hit that we should save it for last, and see if it's necessary. i would prefer to tune
   for any latitude of vision, including very clear days, so it might be nice, but nothing should rely on fog in our
   renderer to perform." A rule now (`decisions.md`, "Nothing relies on fog to perform"): **tune for the clearest
   day.** Big steps are flown with fog off (`INCUS_SIGHT_NO_FOG`) as well as with his settings, so the clear case is
   measured too.
**On the side:** the near ground's pass (`sight_grounds_pass`, the tiles made as the camera flies) holds 255
registers, the most there is (likely spilling to memory), and took 1 to 8% of the card in the traces, by how much
ground was being made; a look after the stages.

### The breakdown (2026-11-09, build 101 with the 10-second hitch fixed)

Each flight with one thing switched off (the measuring switches), against a reference flown the same way (a warm-up
lap, then two measured: flights with it agree within about 2% on every leg). Whole-flight frame times in ms; the
card's time over the land and out at sea. `logs/dyno-1794251718.txt` is the reference.

| Switched off | Median | 95th | 99th | Over the land (card) | Out to sea (card) |
|---|---|---|---|---|---|
| (the reference) | 5.46 | 15.53 | 20.78 | 11.06 | 7.93 |
| The blocks near the eye (`NO_BLOCKS`) | 4.81 (−12%) | 10.06 (−35%) | 11.56 (−44%) | 4.21 (−6.9 ms) | 4.74 (−3.2 ms) |
| The 2 × 2 layer (`GRID=4`) | 4.89 (−10%) | 11.44 (−26%) | 13.21 (−36%) | 6.97 (−4.1 ms) | 5.87 (−2.1 ms) |
| Sky light (`NO_SKY`) | 5.19 (−5%) | 12.63 (−19%) | 16.37 (−21%) | 7.22 (−3.8 ms) | 6.88 (−1.1 ms) |
| Sun shadows (`NO_SHADOWS`) | 5.57 (+2%) | 13.28 (−15%) | 14.95 (−28%) | 8.36 (−2.7 ms) | 7.48 (−0.5 ms) |
| Block textures (`NO_TEXTURES`) | 5.29 (−3%) | 15.59 | 21.12 | 11.22 | 7.82 |
| Fog (`NO_FOG`) | 5.38 (−1%) | 15.52 | 20.74 | 11.00 | 7.88 |
| Placed lights and the wash (`NO_LIGHTS`; none in the world) | 5.41 (−1%) | 15.40 | 20.65 | 11.06 | 7.99 |

**What it says:**
- **The worst frames are the heavy views' near work and their light.** Off, the blocks near the eye take 44% off the
  99th percentile, the 2 × 2 layer 36%, shadows 28%, sky light 21%. The median moves far less: the typical frame
  isn't the problem; the heavy views are.
- **The near walk steps through air.** It skips air only over the tops of columns of 64, 32, 16 and 8; in the last 8
  blocks over the ground it walks block by block, trying all four coarser levels at each block first (a ray 8°
  down from 40 up: some 57 blocks so). The live levels' columns of 4 and 2 aren't used to skip.
- **The sun's shadow ray from a block's face is the same walk** (`hard()` toward the sun, up to 128 blocks): a near
  walk that skips air better pays for the eye's rays and the shadow's at once. **The sky cones aren't**: five cones,
  up to 40 steps each, through the cells (never the blocks), as wide as the cone. Their cost is their own.
- **The savings overlap** (Counsel's question): over the land the blocks, sky and shadows save 6.9 + 3.8 + 2.7 =
  13.4 ms of an 11.1 ms whole. Without the blocks, fewer pixels are refined at 2 × 2, and those don't take sky light
  or shadows either. Each step is measured after the one before it, so the order is written down with its numbers.
- **At sea,** the blocks near the eye are 40% of the card: the near walk goes on through water block by block (no
  skipping in a medium) till it's dimmed out, some 25 blocks down (build 98's 1.5 ms). The sea as a plane takes that.
- **Fog, textures and placed lights cost next to nothing** (with no lights in the world: many lights is its own
  check).
- **Round the peak** the card's own time is small (1.3 ms) and moves ±0.4 ms with the switches either way: its worst
  frames are a few heavy views on the circle.

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

**B. The block, re-ranked by the breakdown** (2026-11-09):
1. [-] **The near walk skips air by the live levels too** (step 1, tried and dropped 2026-11-09). The finest column
   first, then up the levels: the same picture as every block walked, but slower. The first try, each skip a climb
   from the 2s to the 64s: the card's median 6% better, the 99th percentile 20.4 to 29.5 ms worse. Refined
   (starting where the last decision ended, the live levels only): 99th 20.4 to 27.9, over the land the card 11.1
   to 14.1 ms. Measured at David's screen's size, the steps a ray didn't drop (over flat land 80 against 73: no
   stride past 16; over hills 69 against 73): **the blocks near the eye don't cost in stepping through air** but in
   what each near pixel does, the 2 × 2 layer's pixels and their shadow rays and sky cones. Kept from it: the tests'
   truth (`Eye::walk_every_block`, every block walked) and the steps measured (`sight_s_near_walk_steps`).
   *On file:* in its test scene the near walk as it is skipped into steep hills near the eye, by different amounts
   from run to run, so the scene may be at fault; to check in the game before it's called a flaw
   (`sight_s_near_walk_against_every_block`). Seen again in the third step's test pictures (the near view's right
   half drawn from far cells where the blocks are): next to find.
   **What's found so far** (2026-11-09; the pictures in `screenshots/dyno/`: "near walk flaw - as drawn" and
   "- every block walked", 8 over a hillside looking a little down, the right half of the near hill drawn from
   blurry far cells):
   - The test scene with every live level made (the blocks and the cells of 2 to 16, as in the game) comes out
     differently from one making to the next, even twice in one process (`sight_s_scene_made_twice_draws_the_same`:
     skipping air, 37,956 pixels apart; every block walked, 12,488): its data isn't the same each time it's made.
   - In one making a view's near tiles look missing altogether (no blocks even walking every block); in another,
     they're there and the skip over the coarse columns goes into the hill. Which views, changes.
   - Not: the card's work failing (none did), the pool's allotting (right), the store's buffers left unset (made
     whole as it's made), the live window's size (set), the `odd` flag (the world's own).
   - Next: read the store's buffers back from two makings and set them side by side, to see which data differ; then
     whether the game's own making (the live levels filled as the camera moves, the bake beside them) does it.
   - **Found and fixed** (2026-11-09, `sight_s_scene_made_twice_holds_the_same`: the store read back from two
     makings): the live levels were the same both times; a few baked columns were empty in one and not the other.
     The bake writes a region's column index a row at a time, the region's 32 columns (at cells of 32) from where its
     first falls. A map needn't be whole regions: a region over the map's east edge wrote its row on past the end of
     the map's row, over the start of the next, marking those columns empty; one over the west edge wasn't written at
     all (its first column off the map). Which spill-over lasted depended on which baker finished last. An empty
     column tells the near walk there's nothing up to the sky there, so it skipped into the hill. Each row's written
     now only where it's inside the map. **In the game:** the map (72,000 blocks) isn't whole regions either, so
     this was along the map's west and east edges only (a strip a region wide, far out at sea); the test's small map
     put the edge by the eye. The saved test world needs no new bake: its regions' cells were right, and the index
     is written from them as it loads.
   - Checks kept: the store and the pictures the same made twice; the near walk the same as every block walked; the
     split the same as the one pass (its specks went with it). **The A/B picture tests can be trusted again.** And a
     check on the Render page for David (Counsel's): "Walk every block near the eye", to flip where the near ground
     looks wrong.
2. [~] **Sky light kept, not marched** (step 2; David's idea, below in section 2): the five cones a pixel become a
   lookup. Sky light is 3.8 ms over the land, the biggest shading cost, and its own (no overlap with the walk).
   [David 2026-11-09: "blocky and quantized lighting is good all around i think. please try to see if there are any
   savings to be had there", and "yeah, blocky lighting is better. and we should find a reason why that can be
   cheaper, since it's mnuch coarser."] Built as a map on the card filled as places come into view (below);
   against the cones every pixel, the same picture to 0.01 of a level on the whole. **Tried and dropped**
   (2026-11-09): flown, the map was slower than the cones it replaced (the card 5.29 to 5.64 ms on the whole, +7%;
   over the land 11.9 to 12.6), and **the build with it in was 8% slower even with it switched off** (the cones
   every pixel: 5.91 ms median against the reference's 5.46, the card 5.29 against 4.88, the GPU's clock the same).
   So the march is one big shader whose every added line costs every pixel: the card keeps fewer pixels in flight
   the more working room (registers) its biggest path needs, used or not. The sky cones' runtime cost is real (off,
   the card's 3.8 ms over the land), but a lookup in a big map, with atomics, cost as much as it saved.
3. [~] **The march made lean** (step 3, 2026-11-09). Two findings, one kept:
   - [x] **Kept: the card's work in one compute pass** (the start, each layer, the far glows as dispatches one
     after another, not a pass each). With the march as it was: the card's median 4.96 to 4.67 ms (−6%), the frame's
     5.61 to 5.39 (185 fps), the 99th percentile 22.3 to 20.8 ms, every leg a little faster. The passes had been
     leaving gaps: the split's eight passes left some 1.4 ms between them. `INCUS_DYNO_PASS_TIMES` gives each its
     own timed pass again, for measuring where the time goes.
   - [-] **Tried, kept off: the march split from its light** (`march_hit` keeps what each ray met, 48 bytes a pixel;
     `light_hit` lights it). Each march pass alone was 35 to 58% faster (the 8 × 8 layer's 1.93 to 1.18 ms, the
     2 × 2's 0.55 to 0.23): the room the shader needs does weigh on the march, as the second step showed. But in
     one pass, against the march as it was: the worst frames better (the 99th 20.8 to 18.2 ms, −12%; the worst 26.4
     to 22.4; over the land the card −7%), the median worse (5.39 to 6.07 ms, +13%; the lighter views 9 to 27%
     slower, the hand-over and the barrier between march and light costing them more than the leaner march saves).
     Kept as a choice (`Eye::split`, `INCUS_SIGHT_SPLIT`), off: to try again with a lighter light pass (sky light
     kept, which in the light pass wouldn't weigh on the march). *On file:* in some runs it draws dark specks along
     the near blocks' edges (724 pixels in the test's near view): to find before it's used.
   The first plan for it, as written:
   **The march made lean.** The finding above says the lever is the shader's size. Measure it first (Nsight:
   registers and pixels in flight for `march_pass`), then split the light out of the march: the march writes where
   each ray met what (place, facing, level, colour), and a pass of its own lights it (sky, shadows, textures,
   placed light). The march's loop then runs with the room it needs and no more, every step of every ray faster;
   the light pass runs once a pixel, lean too. It's the item under section 2's warps ("the march writes its hit and
   a separate pass shades it"), moved to the front. Sky light kept, stepped per face, may come back in the light
   pass, where it doesn't weigh on the march.
4. **The sun's shadow kept** for the far (horizon maps, or the sun's openness per cell re-lit as the sun moves): 2.7
   ms over the land.
5. **The sea as a plane:** the near walk's steps through water (the blocks near the eye are 40% of the card at sea).
6. Rays end where fog is total; the rest of the old order below as it was.

**B, the order as first written** (the cheapest and surest first; now after the above):
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
- [ ] Nsight Graphics installed on David's machine. *Corrected 2026-11-09:* what's installed is Nsight Systems
      2026.6.1, not Nsight Graphics (found looking for its command line); asked David. And the card's performance
      counters opened to all users (NVIDIA Control Panel: Desktop, Enable Developer Settings; then Developer, Manage
      GPU Performance Counters): Nsight Systems' GPU metrics (warps in flight over a flight) refuse without it
      ("Insufficient privilege", ERR_NVGPUCTRPERM), and Nsight Graphics needs it too.
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
- [~] **The twin** (2026-11-09): two builds of the march flown in one flight, frame by frame. `INCUS_SIGHT_TWIN=<a
      march.wgsl>` has the game read that shader as it starts and draw every other timed frame with it, in the
      game's own shape (since build 104 all its passes, each with bindings of its own, the stages being the game's
      drawing; before, the one pass's start, march and glows only). The card's clock on each kept apart, and the
      report sets them side by side, whole and leg by leg: "the twin against the march", median, 95th and 99th.
      The views and the heat are the same for both, and one flight (about 3 minutes) takes the place of four
      alternating builds (about 25). Step 4 showed a step has to be measured against the build before as a whole:
      this is that, in one game. The frames' own times are both's (the card's work for each is what's apart).
      **Checked:** the march against itself (`logs/dyno-1794274012.txt`), the medians within 0.4% on every leg but
      the sea's 2.2%, the 95th within 1.8%, the 99th within 3.3%: that's the twin's noise. First used on the near
      walk's handover fix against the build before (`logs/dyno-1794274234.txt`): within the noise.
- [ ] An input-to-photon gate: the mouse stays near build 67's 1.05 ms (uncapped).
- [ ] Nsight: card time per pass, stall reasons (memory against divergence), and occupancy. *2026-11-09:* Nsight
      Graphics 2026.4.0 installed, and the counters opened; its command line (`ngfx.exe`, GPU Trace Profiler, the
      real-time shader profiler, exported) is in `C:\Program Files\NVIDIA Corporation\Nsight Graphics
      2026.4.0\host\windows-desktop-nomad-x64`.
- [ ] A slowest-ray view: each tile's slowest ray against its average, built from the steps view (`reach.z`).
- [ ] Register pressure in `march.wgsl` (one big shader).
- [ ] The subgroup check: does the adapter offer them? Keep a path that works without them.
- [x] The breakdown by the switches (the table above): the blocks near the eye, the 2 × 2 layer, sky light and
      shadows are the heavy views' cost; fog, textures and placed lights next to nothing. (Builds 93 and 94's 2 to
      3 ms aren't apart in it: no switch for the floor rule.)

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
- [ ] **Sky light kept, not marched** [David 2026-11-09: "is there some way we can take sky light and bake it into an
      indexed map that grows as the player explores the world and updates as he's in it to make it update, leaving
      the other lighting types alone?"]. Builder's design, for his say:
      - **Kept per cell:** what each of the five sky cones sees from the cell's surface (straight up, and the four
        50° ones), 6 bits each, 4 bytes a cell. At draw time a pixel takes them by its surface's facing, as it adds
        its cones up now, times the sky's colour and strength: a lookup in place of five cones of up to 40 steps.
      - **The store:** 4 bytes beside each cell's 16: about 45 MB more on the card for the whole bake (180 MB now), a
        quarter more on the disk with a saved world.
      - **Made** on the card as each region's cells are baked (the same cones, once), and as the live levels' tiles
        are made near the eye. A region's cones reach into its neighbours: made again once they're baked (the "grows
        as the player explores"). Edits: their regions made again, as now ("updates as he's in it").
      - **Near the eye** it steps a block at a time, a value a block face (Minecraft's way; his build 97 choice for
        placed light): a look for his eye, against the cones' smooth shading now.
      - **Against horizon maps:** they'd do the sun's far shadow too (any sun direction), but only for the ground's
        top (not under overhangs or in caves); this does sky light in 3D. Sky first (the bigger cost), measured
        against the cones (the picture within a tolerance, and the cost); horizon maps after, for the sun's part.
      - **As built** (step 2, 2026-11-09), simpler than the store above: a map on the card, 2 million places (16 MB),
        keyed by a block face (the air in front of it, and its way) near the eye or a cell (its level and place) far
        off. The first pixel to show a place works its five cones out and puts them in; every pixel after reads the
        one number. A sliver of the map is emptied each frame (all of it every 256 frames), so places are worked out
        again within a second or so: edits and newly baked regions show (David's "grows as the player explores the
        world and updates as he's in it"), and no frame does it all at once. `INCUS_SIGHT_NO_SKY_KEPT` marches the
        cones every pixel, for measuring. Needs one more storage buffer: 10.
      - **The sun's shadow, per face?** [David 2026-11-09: "idk how it'd look with shadows for close objects. the
        shadows could be quite coarse, or overlaid, and i have a feeling i'll be fine with rough shadows for
        everything. i'd have to see real shadows from actors with the global stepped lighting."] Leaning to rough
        shadows everywhere, decided once things that move cast shadows he can judge. When the sun's step comes: a
        shadow a face, behind a Render-page switch for his look; objects' shadows are on file from build 95.
- [ ] Light the hologram: sun and sky light kept in the cells, re-lit gradually as the sun moves.
- [ ] **Sky light kept, not marched** [David 2026-11-09, after the breakdown found sky light the biggest single cost
      (sky off: p99 -23%, over land the card 12.6 → 7.7 ms): "is there some way we can take sky light and bake it
      into an indexed map that grows as the player explores the world and updates as he's in it to make it update,
      leaving the other lighting types alone?"]. Sky light is how much of the sky a place can see. That depends only
      on the blocks, not on the time of day, so it can be kept:
      - **What's kept:** each cell's openness to the sky, from a few directions so a face still gets the sky
        it faces (up and the four sides, or four numbers in the manner of spherical harmonics). The sky's colour and
        brightness (dawn, dusk, overcast) multiply it at draw time, so the day still changes smoothly.
      - **It grows as you explore:** made by the bake for the far levels, and as chunks are made for the near ones.
      - **It updates while you're in it:** an edit re-figures the openness near it, and the column below for a dug
        shaft. Spread over a few frames so the work stays even.
      - **Left alone:** the sun's shadow (the sun moves) and placed light (its own flood).
      - **Near the eye, one block a step:** like Minecraft's sky light, and like the stepped placed light he chose in
        build 97, so it's a look question: the five cones could stay for near pixels, or the stepped kept light take
        over. Judged by his eye, behind a switch.
      - A cell-format change (bump SIGHT_RULES, a new bake), or its own buffer beside the cells (the hot/cold split).

**The compiler's layout** (step 4's finding: added code, run or not, moves each view's cost by tens of percent;
Nsight's look: the march holds 128 registers a pixel, so 16 groups of 32 a unit at most of 48, and runs to 30,600
instructions, too many for the instruction cache)
- [x] **The march made smaller, 1: the sky's five cones in one loop** (2026-11-09). Written out, the cone's whole
      code was in the march five times. One loop now, its count read from the store so the compiler can't write it
      out again. The picture the same to the pixel (`sight_s_other_march_draws_the_same`, seven views, against the
      march as it was; a 1% nudge to the sky light shows in thousands of pixels, so the test sees). Flown with the
      twin against the march as it was (`logs/dyno-1794276682.txt`): the card's median −3.3%, 95th −6.1%, 99th
      −5.3%; every leg faster (over the land −8.5% on the median, round the peak −7.0% at the 99th).
- [-] **The march made smaller, 2: the measuring views compiled out unless chosen** (an override constant,
      `MEASURING`; the steps' heat and the levels' tints, whose six-colour table takes some 5 KB of the card's shared
      memory a group). Tried and dropped (2026-11-09): the picture the same to the pixel, but flown with the twin
      against the march (`logs/dyno-1794277097.txt`) **slower**: the median +1.8%, 95th +12%, 99th +6.7% (at sea
      the 95th +14%). Taking code out moved the compiler's choices the wrong way: shrinking one big program is a
      lottery on them, which is why the march goes to stages next (above). Both builds traced together in one game
      (`trace_twin`, Builder's scratchpad), to see what the compiler changed.
- [ ] **The settings compiled in** (WGSL `override` constants; Counsel, 2026-11-09). A switch that's off is taken
      out of the march when its pipeline is made, not kept as a branch whose path still claims registers. wgpu 30
      does it on DX12: naga puts the values in (and keeps only the pipeline's own entry point) before writing the
      HLSL, and DXC drops the branches that can't run. First for the measuring switches, so a step's build with it
      off is the build before; then lean variants of the one march (water or not, shadows or not, tile types later).
      To weigh: sight's three pipelines take 8.8 s to compile with DXC, so a Render-page change picking another
      variant compiles in the background, the old one drawing meanwhile.
- [ ] **What the compiler did** (Nsight Graphics, the compiled march: registers, spills, where it waits), the
      slow layout against the fast one round the peak (step 4's builds). Waits on Nsight Graphics (section 0).

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
- [ ] **DXC shipped with the game** (before release; Counsel, 2026-11-09). The game compiles its shaders with DXC
      only where the Windows SDK's `dxcompiler.dll` is installed (`runner::windows_sdk_dxc`). Players won't have the
      SDK, so they'd get the old compiler (FXC): some 50 s to compile the march at the start (found 2026-11-04), and
      a different layout and speed from what the dyno measures. wgpu 30 builds DXC into the game with its
      `static-dxc` feature (or takes the dlls beside the exe), so every player runs what was measured.

## Ruled out

Tensor cores, ray-tracing cores, DLSS, Nvidia's ray sorting (SER) and hardware variable-rate shading, under David's
rule: "im not interested in implementing all the different tricks individual modern cards can use, id have to own 20
gpus".

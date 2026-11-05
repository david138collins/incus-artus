# The regroup: keep the good work, leave Bevy, start the graphics again on the card

**[PROPOSED 2026-11-05]** Builder's plan, for David and Counsel to read before any code. Nothing here is
built yet. Sizes are in Builder's working hours, plus David's **looks** (a launch he plays and judges,
10 to 20 minutes) and **test windows** (the laptop off while Builder measures, 45 to 60 minutes).

## What David decided (2026-11-05)

- **Regroup.** "it sounds like we should do a ruthless cut pass, get it working again, without graphics at
  all, and start inside the gpu. just don't do work that makes the cpu do things it won't be doing in the
  commercial steam build."
- **Keep what's good.** "our editor, world gen, simulation, and the setup you've done to make it easier to work
  on, these are good." And: "as long as we don't lose anything that we are happy with".
- **Our own scheduler.** "as long as you know what you're doing and feel good about having our own scheduler
  for the simulation multiplexing, i'm into it."
- **Build 60 stops here.** "its stuttering and thats a dealbreaker."
- **Hybrid is allowed.** "i am very attached to the idea of marching everything simultaniously but it may
  require maths i just can't access. getting the game done is more important. we should be able to make it look
  right with a hybrid setup. i do still believe in a somewhat exotic far distance rendering solution, that
  naturally pixelizes, and light that naturall is coarse - but i have an open mind. as long as its beautiful to
  look at, fast, and reads perfectly to the player."

## The spec, as measured from now on

Measured at the screen with PresentMon (installed 2026-11-05), not inside the game.

- **David's laptop** (RTX 5070 Ti Laptop, Core Ultra 9 275HX, Turbo, max fans), uncapped: about 250 a second
  most of the time, often enough that he never feels like capping it. A dip now and then is fine.
- **Capped at 120 on the same laptop: not one stutter, ever.** No frame later than 8.3 ms.
- **A 980 Ti at a smaller resolution (1080p): a smooth 60**, with the full experience at standard looks.
- **A stronger desktop with better cooling:** 250 and up, without question.

**Answered after reading the plan (2026-11-05, `decisions.md`, "The regroup"):**
- **Graphics first.** "i dont think graphics and scheduler should impinge on one another and, to be frank i have
  high confidence in her ability to do it right when its time. right now we have nothing to simulate... getting
  the vision in the gpu *the right way* is our mortal threat and the most fundamental issue we have." So the
  column store and the scheduler move to the simulation stage; until then the simulation runs on its own lane
  with the toy rules as they are.
- **The editor isn't shipped for now.** "the editor is for me until the best use of our time is serving eager
  modders. thats a ways off." The shipped build leaves it out; ours keeps it.
- **Start small, keep checking.** "my instinct is to just start small and keep checking performance as we
  assemble vision." On the stutter: "we need to isolate the offender and not repeat it".
- **Spend the headroom slowly.** After build 61 (about 1,650 FPS with nothing on screen), David said: "i can not
  impress upon you enough how gradually i want to spend that wealth when we start constructing eyesight in the
  gpu again. i really want to go step by step and think carefully about how it can be done. we have to be
  clever." Every step that puts something on screen is talked through with him before it's built, then
  measured on its own before the next one goes on.
- **The landmark:** the local tag `before-regroup` (annotated, on ea7f242: build 60 plus the day's docs).
- Audio, gamepads and menus: not decided yet.

## The rules for the new build

1. **The processor never does, in a frame, anything the shipped game wouldn't.** The shipped build leaves
   the editor out; in ours, debug pages, logging and editor tools cost nothing while they're closed. No
   per-frame bookkeeping that's only there for us.
2. **Three lanes that never wait on each other** (`decisions.md`, "A stage that takes whatever is put on it"):
   - **the frame:** input, camera, hand the card its work, show the panel;
   - **the simulation:** its own thread, 30 ticks a second, its own scheduler; the frame reads its latest
     finished snapshot;
   - **loading:** worker threads (world making, saves, the recipe's batches); results arrive when ready.
   Each lane has a budget: too much work arrives late or runs less often; the frame holds.
3. **What's drawn is the card's.** The card makes what it draws from the recipe; the processor only sends
   what exists nowhere else (edits, the simulation's people), when it changes.
4. **Every feature has a price** in ms on the card (and processor ms if any), measured when it goes in, kept by
   a test. Nothing is added without its number.

## What carries over, what's cut, what's rebuilt

| Part | Lines | What happens |
|---|---|---|
| World gen (`worldgen/`): tectonics, climate, rivers, the compiler, the World Gen suite's views | 23,100 | **Kept.** Its views already draw through egui pictures, which carry over. Bevy's systems wrapping it become plain calls. |
| The world (`world/`): chunks, storage, the recipe, carving, caves, placer, mineralogy, edits | 11,100 | **Kept.** Chunk streaming is rewritten for the loading lane. |
| Simulation (`sim/`, `people/`), Fate, the clock (`time/`), events, saving | 5,100 | **Kept.** Moves onto the simulation lane; the clock's tick loop drives it instead of Bevy's `FixedUpdate`. |
| Data files and loading (`data/`), the debug panel, presets, settings, sliders (`debug/`) | 3,100 | **Kept.** The panel moves from `bevy_egui` to egui on wgpu; its pages barely change. |
| Player collision, camera maths, kinematics, crash and freeze reports (`diagnostics.rs`) | 3,900 | **Kept**, rewired to winit's input. |
| The 395 tests | | **Kept**, the check on everything carried over. |
| The renderer (`render/`): march, far copies, distant tiles, light maps, materials | 8,100 | **Cut.** Kept at the tag `before-regroup` for reference; the march's knowledge and shader pieces come back only where they earn their place. |
| Bevy (window, input, app, schedule, renderer, assets) | | **Cut, but for its core** (the World, schedules, states, time, input types, logging), kept for now behind one doorway with an ejection seat (below). winit (window, input), wgpu (the card), egui, the `gltf` crate (models, already used). Audio (kira) and gamepads (gilrs) when they're needed. |

## The ejection seat: Bevy's core, kept for now, ready to pull out

**[DECIDED 2026-11-05]** David: "we're not having any mercy on bevy. anything we keep either has to be because
its the best way of doing it, or replacing it is impossible for claude and david. if we keep anything in place in
the meantime - we need to build explosive fastener ejection seats for it. so we can use the ECS, and later,
modify/improve it for our purpose, or copy what is good in it; we just have to make sure it is ready for rapid
disconnect."

Phase 0 kept only Bevy's core: the World with its resources and messages, the schedules, states, time, input
types and logging. Its window, renderer, assets, UI and the rest were cut. It stays **only because replacing it
right now isn't worth the risk**, not because it's best.

- **One doorway.** Everything Bevy is reached through `src/ecs.rs` (`crate::ecs::prelude::*` and so on). No other
  file names `bevy::`.
- **New code is Bevy-free** unless it has to talk to the World: the runner's card and window side, the card work
  to come, and later the column store and the simulation's scheduler, which are ours from the start.
- **The gauge** (a test in `src/ecs.rs`): it fails if any file but the doorway names `bevy::`, or if the files and
  lines that use the doorway go up. **The reach on 2026-11-05: 51 files, 84 lines.** It may only go down.
- **When it comes out:** when the column store arrives at the simulation stage; or sooner, if phase 0's floor
  shows the core costing more than about 0.5 ms or 30 wake-ups a frame at 120 over no Bevy (0.6 ms and 13 at 120).
- **Anything else from Bevy that stays** gets the same rule: say why it's the best way, or why it can't be
  replaced, and give it the same seat. (Its maths types are glam's, which we'd use directly: not a Bevy matter.)

**The disconnect list** (what pulling the core out would take; Builder's hours):

| What | Where it's used | Replaced by | Hours |
|---|---|---|---|
| The World and its resources | 74 resources, 14 components, queries in about 14 files | our own store of each lane's state | 3 to 5 |
| Systems and schedules | 63 registrations, their order (`after`, `chain`, sets) and run conditions | ordered lists per lane; the simulation's rules scheduler comes with the column store | 2 to 4 |
| Messages | about 20 kinds | our own queues, read once a frame | about 1 |
| States | the Play / World Gen screen, its enter and exit steps | an enum with its own enter and exit | about 1 |
| Time | real time, the fixed ticks (`FixedUpdate`) | the clock's own loop (`src/time/` already holds its logic) | 1 to 2 |
| Input types | keys, mouse buttons, mouse motion | our own, from winit (the key table is already ours, `src/runner/keys.rs`) | about 1 |
| Logging | `info!` and the rest, with the session log's layer | `tracing` directly (what Bevy's logging wraps) | under 1 |
| Worker threads | the loading lane's jobs | `rayon`, or a small pool of our own | about 1 |
| Tests that build an app | 4 files | the same tests on our own store | about 1 |

In all, about 12 to 18 hours of Builder's, plus 1 look (the controls and the clock) and 1 test window.

## The action plan: from black to the whole world on screen

**How it's held** [David 2026-11-05]: "we dont really know whats going to happen, we will be shooting from the
hip again with graphics. we have to stay light on our feet for this path from black to fast full visuals... we
needed to see the world to tune it and now its tuned in close enough that i know perfect is within our
adjustment range. i know it will be there when we have eyes again. that gives us latitude." So the near phases
are planned action by action; the far ones loosely, to be planned again when we reach them. **Forks** are the
points where a measurement or David's look decides the next move.

**In every phase:** nothing on the processor per frame that the shipped game wouldn't do; every feature gets its
price (card ms, processor ms) when it goes in, written down; the slow-frame log is on in every test window.

**When David sees land again:**
- **rough land to the horizon, with its water:** the end of phase 1, about 3 to 4 hours of work after the stage is done;
- **blocks:** the end of phase 3.

### Phase 0: the stage (dark)  [planned in detail]

| # | Builder does | Hours |
|---|---|---|
| 0.1 | Cut `render/` and Bevy. The new core: a winit window, wgpu on DirectX 12 (DXC, as now), the frame cap ported from `frame_cap.rs`, a sky-coloured clear. Our own input state (keys, mouse, mouse capture). | 2 to 3 |
| 0.2 | The three lanes. The **frame** lane on the main thread. The **simulation** lane on its own thread, driven by the clock (`time/`, 30 ticks a second), running the toy rules as they are, handing the frame a finished snapshot. The **loading** lane: worker threads with a job queue and a budget, results arriving when ready. | 1 to 2 |
| 0.3 | The panel on egui-winit and egui-wgpu. Every page rewired: the World Gen suite, debug pages, presets. Its pictures (`load_texture`) work as before. | 1 to 2 |
| 0.4 | The game's own systems run as before in Bevy's core, each schedule on one thread, every Bevy name through one doorway (`src/ecs.rs`, the ejection seat below). Saving and loading, events, the crash and freeze reports. The tests pass. **[BUILT 2026-11-05, aa49270 and the doorway after it: 353 tests pass; the old renderer's 43 went with it.]** | 1 to 2 |
| 0.5 | The slow-frame log: every step on every lane timed. A frame over 8.3 ms (or well over the usual) is written out with what ran in it. Costs nothing in the shipped build. A PresentMon script for test windows. | about 1 |

- **David:** 1 look. The editor, World Gen suite and save/load all work, on a plain sky-coloured window.
- **Measured:** 1 test window. The floor at the screen, uncapped and at 120. Expected near today's no-Bevy floor (0.02 to 0.6 ms of processor a frame), with no slow frames at 120.
- **Fork A:** if the floor isn't there (well over 1 ms of processor a frame, or any slow frame at 120), the log names it, and it's fixed before anything else goes on the stage.
- **[MEASURED 2026-11-05, build 61, PresentMon at the screen, 2 minutes each, editor closed]**
  - **Uncapped:** 1,950 FPS average, 933 FPS 1% low; processor 0.34 ms a frame, card 0.17 ms. The floor is there.
    But 6 frames took 10 to 19 ms and one took 220 ms, all inside the hand-off to the driver (getting the next
    picture, or presenting it), not in our work.
  - **At 120:** 1% low 100 FPS. 336 frames stayed on screen longer than 12.5 ms (3 refreshes instead of 2). That's
    ours: the cap is waited out on a timer of the game's own, after a vsynced present, and the timer wakes late
    (waits of 9 to 19 ms logged for 8.3), so frames miss their refresh. **Fork A is hit:** fixed before phase 1.
  - **A caution on the uncapped stalls:** the game's own log has only 1 of PresentMon's 6 slow frames in that run,
    and not the 220 ms one. PresentMon without administrator rights may drop events at 2,000 FPS, which shows as
    a gap that wasn't there. A stall counts when PresentMon and the game's log agree.
- **The torque spec** [Counsel's, David 2026-11-05: "this device needs to be trick"]: after any change to the frame
  loop, the same runs at the screen (`INCUS_TEST_WINDOW`, `soak.rs`: 2 minutes uncapped, 2 at 120, by itself),
  with their numbers written here. A bolt is tight once its check passes, and again after every change.
  Response before frame rate: input-to-photon latency is a first-class number in it.
- **Build 62, Fork A's first bolts** (not yet measured):
  - **Pacing by the screen's own clock** (`runner::pace`, `runner/hardware.rs`): under a cap, the frame waits for
    the screen's refreshes (DXGI's wait for the vertical blank on the window's screen) until the cap's number
    has gone by since the frame began, so the next frame starts just after a refresh, reads the input then, and
    shows at the refresh after. Counted from the screen every frame, so it can't drift; a frame that ran long
    waits one refresh only, so frames never queue. The timer stays as the fallback. The simulation's clock
    is untouched (real time; tested: the same 10 s at 60, 120, 240 and 2,000 frames a second give the same
    ticks, and a 1 s stall is a quarter-second of simulation and a pause, not a rush).
  - **Input read at the last moment:** already so. winit hands over the input that came during the wait just
    before the next frame begins.
  - **Nothing written on the frame:** the session log and the console are written by a log writer thread
    (`diagnostics.rs`); the frame only queues the line.
  - **The hand-off timed step by step:** getting the next picture, drawing the panel for the card, handing the
    card its work, showing it, waiting for the screen.
  - **Which core the frame is on:** logged every 10 s in a test window, or whenever any frames landed on the
    efficient cores.
  - **Response time:** PresentMon's input-to-photon columns need input, and the test window has none at the
    controls, so a script beside it (not the game) presses F24, a key the game doesn't use, four times a
    second. Its average and worst, uncapped and at 120, go in the torque spec beside the frame times.
  - **The test suite green:** the 2 panel tests that failed in debug test runs (egui's check that font
    pictures are handed on; the tests had no card to hand them to) now let them go. 355 pass, none fail.
- **For lane 0.2 (David 2026-11-05):** priorities on every job (this frame / soon / whenever), the workers below the
  frame thread; and in the simulation, under pressure the farthest and lightest give way first (the World sim,
  then the Intermediator, the Local sim last), as a pause, never lost state: people pick up what they were doing.
  Until then the simulation's only rule is Bevy's, the same for all: a stall over a quarter-second is a pause.

### Phase 1: first sight, rough land to the horizon  [planned in detail]

| # | Builder does | Hours |
|---|---|---|
| 1.1 | At world load, once: the compiled maps (heights at 32-block pixels, colours from climate and palette) go up as two textures, with a min-max pyramid for skipping. Not per frame. | about 1 |
| 1.2 | A full-screen height-field march (one ray a pixel, stepping down the pyramid), the fly camera back, the sky colour. Its own module, one shader and two textures, so it's thrown away cleanly or kept. | 1 to 2 |
| 1.3 | Water as a flat stand-in colour from the start: the sea at its level, and the rivers and lakes from a water map made once at load (their surfaces drawn into a finer texture, about 4 blocks a pixel, so rivers narrower than 32 blocks still show). The real water look stays in phase 6. | about 1 |

- **David:** 1 look. The whole world to the horizon: smooth hills and coasts at 32 blocks a pixel, with the sea, lakes and rivers in a flat water colour, no blocks yet. Flying only, since walking needs chunks (phase 2).
- **Measured:** its price, folded into the next test window. Expected well under 1 ms.
- **Fork B:** it's a tool for seeing the land again early, not the backdrop. One height per column can't show a cave mouth or a hall in a mountainside, which the backdrop must (phase 4). It's a backdrop candidate only if it's grown to carry holes; otherwise it goes when the first band arrives.

### Phase 2: the recipe as one exact source, on the card  [planned in detail]

| # | Builder does | Hours |
|---|---|---|
| 2.1 | A small fixed-point kit (angles, powers, square roots in whole numbers), with tests. | 1 to 2 |
| 2.2 | The 12 to 15 calls that run in play converted to it. A before-and-after comparison over the whole map: how many columns change, and by how much. | 1 to 2 |
| 2.3 | The recipe ported to the card (WGSL compute): the map blend, land and sea curves, mountains and ridges, peaks and crags, torn coast, detail, the river and lake carving (their lookup lists flattened into buffers), mineralogy's layers and the placer's fields they read (two passes, for the slope), and the cave carve (already whole numbers; the cave plan flattened into buffers as its section boxes). The twin test: about a million columns, card against processor, exactly, plus the cave carve block by block. | 4 to 6 |
| 2.4 | Reading back, on the loading lane: the card works out a chunk's columns, they come back through a ring of staging buffers without waiting, and the real chunks near the player are filled from them, caves taken out as today (whole numbers, the same answer). Walking and collision return. | about 2 |

- **David:** 1 look after 2.2: the same spot before and after (Render World and Spawn at the exact same place). 1 look after 2.4: walking.
- **Measured:** the readback's cost, in the slow-frame log.
- **Fork C:** if a saved world's river disagrees with its banks after 2.2, either carve the rivers again on load or accept it. His look decides.
- **Fork D:** if one piece can't be made exact on the card, it stays on the processor (worked out once and uploaded), or its maths is simplified, with his look.

### Phase 3: the first band, the world as blocks, made on the card  [planned, looser]

| # | Builder does | Hours |
|---|---|---|
| 3.1 | Rings of cells round the camera on the card, 1, 2, 4, 8 ... blocks a cell, filled by the recipe on the card. They're solid cells, not heights, so a coarse cell holds what's really there, overhangs and caves included. Only the strips newly uncovered as the camera moves are filled, so the cost follows movement. Each ring keeps coarse "anything here?" bits for skipping empty air. | 3 to 4 |
| 3.2 | The march: one ray a pixel, from the finest ring outwards, the cell size following the pixel's size, out to where a block is smaller than a pixel (at the game's 100° view, a block is one pixel at about 600 blocks in the middle of the screen; the band's edge is priced between about 600 and 1,000 blocks). Block colours and textures from the block files. Pieces of `march.wesl` come back where they earn their place. Phase 1's rough land shows past it meanwhile. | 3 to 4 |
| 3.3 | Edits: the real chunks' changed blocks override the card's cells, a small upload when they change. | about 1 |
| 3.4 | Water in the band: each column's water top from the recipe, in the flat stand-in colour, the same as phase 1's, so the land and its rivers are judged together from the first look. | about 1 |
| 3.5 | Caves at every ring, by the same carve as the near blocks, so far and near agree: a coarse cell is open where most of it is cave (several points in it tested). A 40-block mouth is about 24 pixels tall at 1,000 blocks and still about a pixel at 24,000 [Counsel's sums, 100° view], so it reads at any distance the band reaches [David 2026-11-05: "if a hall in the mountain or a cave is big enough to be seen at a distance, it should be seen at a distance"; `worldgen.md`, `caves.md`: mouths lit and seen from far away]. **Price:** the cave plan's section boxes answer "nothing here" for almost every cell, so the extra cost follows how much cave there is, not how much land; it's paid when strips are filled (as the camera moves), not every frame. Measured in the test window; expected a few per cent of the filling. | 1 to 2 |
| 3.6 | The stand-in forest, for the bar's fourth point (heavy content doesn't change the answer): simple block trees placed on the card by a whole-number spaced hash over a placer field (the "points" kind the placer has on file), stamped into the cells. In the far rings a tree is a few coarse cells; past the band, a forest is a colour and a roughness in the backdrop. Real trees come later through the placer and blueprints, placed and stamped the same way. | 2 to 3 |

- **David:** 2 looks, the land near and then far (with water, caves and the stand-in forest).
- **Measured:** 1 test window. Its price by distance, as today's tally. The aim, on his laptop: the first band at 2 ms or less of card time a frame, standing and flying, with no slow frames at 120.
- **Fork E: if it's too slow,** try these in order, each measured and kept only with its number:
  1. the coarse pass first: one ray per 8 × 8 pixels, so each pixel's ray starts close to what it'll hit (usually the biggest win);
  2. bring the band's edge nearer and let the backdrop take more;
  3. cells a little coarser than the pixel in the middle distance, judged by his eye;
  4. the band past about 500 blocks marched at half resolution and scaled up, keeping edges.
- **Fork F: if it looks wrong** (shimmer where cell sizes change, swimming as the camera moves): blend the sizes over a short distance, or hold each screen tile to one cell size.

### Phase 4: the second band, the backdrop  [loose]

Past the first band, the far view that pixelates naturally. **It must show what's really there, blending seamlessly with the near** [David: "we need to *accurately* represent far terrain in a way that blends seamlessly with the near"]: a cave mouth or a hall in a mountainside included. Candidates:
- a dome (a panorama painted by the march now and then and shifted as you move, `vision/proposal-voxel-renderer.md`): it shows whatever the march sees, mouths included;
- coarse cells, as the first band's further rings;
- phase 1's height field only if it's grown to carry holes (one height per column can't show a mouth).

His look picks. Important things show from their first pixel through a list drawn on top, lights at cave mouths at night among them (the firm rule). One transition to tune, far away. Aim: about 0.5 ms.
- **Builder:** 1 to 2 sessions. **David:** 2 looks. 1 test window.

### Phase 5: light, coarse by nature  [loose]

Sky light from the card's cells (how much sky each cell sees), placed lights in coarse cells, day and night.
- **Builder:** 1 to 2 sessions. **David:** 1 to 2 looks. 1 test window.

### Phase 6: everything else, each with its price  [loose]

The real look of water (the flat stand-in colour has been there since phase 1) and glass, pieces and bars, models (drawn as meshes into the same picture: the hybrid), people and figures (drawn, one instanced pass), and the 800-figure stand-in battle if David puts it in the bar. Sized when we get there.

### The whole path, roughly

| Phases | Builder | David's looks | Test windows |
|---|---|---|---|
| 0 to 3 | about 30 to 40 hours | about 6 | 3 |
| 4 to 5 | about 2 to 4 sessions | 3 to 4 | 2 |

Fork E could add 2 to 4 hours.

Then **the bar** (`decisions.md`): runs smoothly and cool on his own flight, the land reads as one land, built things hold up at a distance, heavy content doesn't change the answer. After that: the blueprint factory, then the simulation, which starts with **its column store and scheduler** ("tables that don't know what they hold"; rules that name what they read and write, run side by side when they don't clash; `simulation.md`, rules as algebra), the toy rules rewritten on it as the first test. *Builder:* 1 to 2 sessions; *David:* 1 look.

## Risks, and what David would feel

- **Phase 0 touches 75 files.** The tests and the soak test check most of it. What they can't check is feel: mouse look, keys, the panel's grip on the mouse, the clock's pause and speed. That's his first look.
- **The first band's price** is the big unknown. The card measured 3.7 ms for build 60's whole scene with no tricks; the first band has to come in well under that, to leave room for light and everything else. Fork E is the answer if it doesn't.
- **The recipe on the card** is the biggest single piece of work (2.3). The twin test makes it safe: the card is never trusted until it agrees with the processor exactly.
- **A dark stretch:** only phase 0, plus 3 to 4 hours. After phase 1 there's always land on screen.

## Open, for David

- The audio, gamepads and the real game's menus aren't in any step yet. When does he want them?
- Phase 0 starts on his go, after he's read this.

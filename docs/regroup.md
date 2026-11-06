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
  lines that use the doorway go up. **The reach on 2026-11-05: 51 files, 84 lines; 83 once chunk making moved to the loading lane.** It may only go down.
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

**Nothing visible is temporary** [David 2026-11-05: "lets just make sure we're not building 'tools for seeing the
world again'. we're now building what the player will see, and nothing is temporary"]. The old phase 1 (a height
field to see the land early) is withdrawn; sight starts from the far view, done right, after the recipe is on the
card ("installing the hologram onto the gpu comes before cracking the eyelids"). Every visible step is talked
through with him before it's built, and measured on its own.

**The order:** phase 0 (the stage, with lane 0.2), phase 2 (the recipe on the card), phase 3 (sight: the twin
layers), then phase 6. **When David sees land again:** phase 3.2, the coarse layer alone: the whole world to the
horizon, squinted. **The milestone** [David 2026-11-05]: "spawning in to the new vision system and seeing it look like
a dos game resolution, *but it's all there, dynamic, honest, and unflawed* would be a huge milestone. after that, we
just build in the other side of the twin engines." **[GO 2026-11-05]** for everything before it: lane 0.2, phase 2,
3.1 ("whatever time it takes us to build a *killer end-game graphics engine*, what good will it do us to waste
time?"). 3.2 is talked through with him before it's built.

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
- **Build 62, Fork A's first bolts** (measured below):
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
- **[MEASURED 2026-11-05, build 62, the test window by itself, PresentMon, nvidia-smi, F24 pressed 945 times]**
  - **Uncapped: better.** 1,930 FPS average; 4 frames in 2 minutes held over 8.4 ms (build 61: 6, and a 220 ms one),
    and this time PresentMon and the game's log mostly agree. The frame stayed on the performance cores (0.0% on
    the efficient ones). The card ran at full clock (2,850 MHz, P4) and 38 W, to draw an empty sky 1,930 times a
    second.
  - **At 120: worse, and the reason is the screen.** 91 FPS average, frames 8 to 21 ms apart. The time between
    frames on the screen isn't a multiple of a refresh (11.8, 12.9, 5.4, 20.9 ms...): **the laptop's screen has a
    variable refresh rate** (G-SYNC or the like). It has no steady clock of its own in a game: it shows a frame
    when the frame arrives. So waiting for its "refresh" waits for the screen to follow us, which it does late
    and unevenly. The screen's clock is the wrong reference on this screen; a steady clock of our own is the
    right one, because the screen then shows each frame the moment it's ready, with no refresh to miss.
    Build 61's stutter at 120 was the same thing seen the other way: the timer woke late (up to 10 ms), and the
    screen showed the lateness faithfully.
  - **The card's clocks at 120:** dropped to P5, 300 to 825 MHz, 15 W, and its work on a frame took up to 7 ms
    (0.3 ms at full clock). Relaxed, but slow to answer: bolt 5 is real, and it matters more as the picture grows.
  - **Response time: not measured.** PresentMon without administrator rights didn't see the presses (every
    input-to-photon value is empty). It needs an elevated run, or a timing of our own.
- **Build 63, the metronome and the attention** (David 2026-11-05, measured below):
  - **The beat is ours:** the processor's own counter, each beat the last plus a frame's length, so it can't
    drift; the alarm is Windows' high-resolution timer, and how late it wakes after every beat is measured and
    logged (`perf::woke`): the "time to get its attention" figure. No holding still before the beat unless
    the figure calls for it (David: "sounds like a catastrophe waiting to happen"; it's what RivaTuner does,
    but only if needed). The screen's-refresh pacing of build 62 is taken out.
  - **Attention while in front** (David: "a full screen only thing that tyrannizes the PC, as long as people
    can at least tab back to discord quickly"): Windows' tick at 1 ms (for this program only, since 2020),
    the frame's thread at the highest priority, Windows told not to slow the game down to save power. All of
    it given back when the game isn't in front.
  - **Freeze when not in front** (David: "i'd rather have the game just freeze (deliberately) and stay
    there"; "i can't think of why someone would want it running blindly"): one last frame under a grey veil
    with "Paused", then nothing drawn, the world stopped, the event loop asleep; a click brings it back, and
    the time away is left out of the world's clock. A setting on the Render page; never while the game
    measures itself.
  - **Borderless fullscreen** by default (David: "yeah, borderless fullscreen is probably the best"); F11
    makes it a window and back.
  - **The frame limit** (the Render page's `frame_limit`): just under the screen's top rate by default (3
    under: 237 on a 240 screen), the screen's rate by vsync (for screens without a variable refresh rate),
    120, or uncapped. David: "if it ever starts failing to keep up at 237 or 240 or w/e, we stop and assess."
    The old `frame_cap` (a frame every so many refreshes) is left in old presets and not read.
  - **The torque spec's runs are now uncapped and just under the top** (the test window).
- **[MEASURED 2026-11-05, build 63, the test window by itself, PresentMon as administrator, nvidia-smi]**
  - **Our beat holds.** At 237: 28,438 frames in 120 s, exactly 237 a second; frames handed over 4.2 ms apart,
    only 48 more than 1 ms late and 2 over 8.4 ms. **The wake-up figure** (how late the alarm woke after its
    beat): 0.26 ms in the middle, 0.58 ms at the 99th in 100, a handful a minute over 1 ms, 8.5 ms once in
    two minutes. The frame stayed on the performance cores throughout.
  - **The card is now the uneven part (bolt 5).** At 237 it dozes: 555 MHz on average (337 to 1,950), 14 W, and
    its work on a frame, 0.2 ms awake, took up to 6 to 12 ms. So frames reach the screen unevenly: 518 of them
    stayed up longer than 8.4 ms (twice a frame) though they were handed over on the beat.
  - **Uncapped:** 1,952 FPS, the card at full clock (2,840 MHz, 39 W). PresentMon shows 16 gaps over 12.5 ms that
    the game's own log didn't see; at almost 2,000 frames a second the record may still drop events.
  - **Response time: still not measured.** PresentMon ignores keys pressed by a script (966 F24 presses, no
    input-to-photon values even as administrator). It needs real hands on the mouse or keys during a run.
- **Build 64, the ballast** (David 2026-11-05: "give it crayons and paper along with a blank screen, just until it has
  enough load that it becomes linear all the way to full use in a steam release"; "good deal, install ballast"; not
  yet measured): made-up work for the card each frame, like sandbags on a test flight (`runner/ballast.rs`).
  **Developer builds only:** the file isn't compiled into a player's copy (checked: the player's build compiles
  without it). **Sized in work, not time:** a unit is timed at full clock in the first moments (best while uncapped),
  then each frame gets that many milliseconds' worth at full clock, less the real work, so a dozing card shows as
  taking longer over the same work instead of the ballast chasing its clock. It reads and writes 256 MB at random
  as well as doing sums, so the card's memory works too. The card's own timestamps measure it and the real work.
  `INCUS_BALLAST=<ms>`, or the test window with `INCUS_TEST_BALLAST`: the run under the top repeated with 0, 1, 2
  and 3.7 ms (3.7: build 60's whole scene standing). As the real picture grows it shrinks, and it's gone once
  real work passes it.
- **Lane 0.2, the loading lane [BUILT 2026-11-05]** (`src/loading.rs`): our own worker pool (all the processor's
  threads but two), its threads below the frame's priority, jobs by tier (`Soon` before `Whenever`) and then by key
  (nearest first), a job dropped before it runs once nobody wants it, a panic in a job caught (and in the crash
  log) without losing the worker. Chunk making moved onto it from Bevy's task pool (the doorway's reach down to 83
  lines). The World Compiler's own thread also stands below the frame. The frame takes results in within its
  budget, as before. Still on the frame: a save's file writes (rare and asked for; they move when saving is
  reworked to report back from the lane).
- **Lane 0.2, the simulation lane [BUILT 2026-11-05, in the form that lasts till the column store]** (David: "i agree
  with... the builders way"): the toy simulation stays on the frame's thread, with **its share of the frame** (the
  ticks timed together each frame, 2 ms at most by default, on the Time page; "the simulation's ticks" in the
  slow-frame log) and **the give-way order** (`time::GiveWay`, `SimPressure`): over its share two frames running, the
  next level gives way: the World sim's thin records first, then the Intermediator's rich ones, the people round you
  last; well under (half) for a second, they come back one at a time in the reverse order. Giving way is a pause:
  what's due waits, then catches up on all the time that passed. The player's body and block edits never give way.
  Tests: the order going and coming back; a thin place that waits is unchanged, then caught up to the minute. **Its
  own thread** comes with the simulation's column store and scheduler, the ejection seat's planned moment, built on
  its own scheduler from the start rather than on Bevy's.
  - **An option, not a plan** (Counsel's): once the simulation has its own thread, a processor-side ballast standing in
    for a heavy simulation, to prove a heavy sim never disturbs the frame.
- **For lane 0.2 (David 2026-11-05):** priorities on every job (this frame / soon / whenever), the workers below the
  frame thread; and in the simulation, under pressure the farthest and lightest give way first (the World sim,
  then the Intermediator, the Local sim last), as a pause, never lost state: people pick up what they were doing.
  Until then the simulation's only rule is Bevy's, the same for all: a stall over a quarter-second is a pause.

### Phase 1: [WITHDRAWN 2026-11-05]

It was a height field to see the land early, "thrown away cleanly or kept": a tool for seeing, which David has ruled
out. Its one piece on the real path, the compiled maps going up to the card once at load, is now 2.0: they're the
recipe's inputs.

### Phase 2: the recipe as one exact source, on the card  [planned in detail]

**Why it comes before sight:** the compiled maps are the recipe's input, not its output. Peaks, crags, ridges,
the torn coast, the detail, the river and lake carving and the caves are all added on top of them. A true average
of what a patch of world holds (phase 3) means running the recipe finer than the patch and averaging what comes
out, which the card does fast and exactly.

| # | Builder does | Hours |
|---|---|---|
| 2.0 | At world load, once: the compiled maps (heights, colours from climate and palette, the water map) go up to the card as textures, the recipe's inputs. Not per frame. Measured only; nothing to see. | about 1 |
| 2.1 | A small fixed-point kit (angles, powers, square roots in whole numbers), with tests. **[BUILT 2026-11-05]** `src/whole/`: `Fix` in 65536ths; + − × ÷ (× rounds down, ÷ towards zero and holds at the ends), sqrt and hypot (rounded down, exact), log2, exp2, pow, sin, cos, atan2 (tables made once by a script, read the same by both, within about 1/10,000), smooth and mix; its copy for the card in `kit.wgsl`, every 64-bit step built from 32-bit pieces. **The twin test passes on David's card:** about 59,000 cases, every step, the card bit for bit the processor. | 2 to 3 |
| 2.2 | **The whole in-play recipe converted** to it, not just the dozen or so chip-dependent calls first counted: plain fractions differ on the card too (a multiply and an add fused into one rounding; division and square root not exactly rounded), so the map blend, land and sea curves, mountains, ridges and peaks, the torn coast, detail, river and lake carving, mineralogy, the placer's fields and the cave carve all go to whole numbers (about 400 fraction uses). **The bar is qualitative** [David 2026-11-05: "move to whole numbers. otherwise the hologram breaks. as long as the behavior is qualitatively identical in its procession, it doesnt matter if worlds come out different, so long as the rules never changed"]: the same rules behaving the same way, his look at his default presets before and after in the World Gen suite's pictures; the count of columns changed is information, not a pass mark. What must be exact is the new twin, processor and card. | 8 to 15 |
| 2.3 | The recipe ported to the card (WGSL compute): the map blend, land and sea curves, mountains and ridges, peaks and crags, torn coast, detail, the river and lake carving (their lookup lists flattened into buffers), mineralogy's layers and the placer's fields they read (two passes, for the slope), and the cave carve (already whole numbers; the cave plan flattened into buffers as its section boxes). The twin test: about a million columns, card against processor, exactly, plus the cave carve block by block. | 4 to 6 |
| 2.4 | Reading back, on the loading lane: the card works out a chunk's columns, they come back through a ring of staging buffers without waiting, and the real chunks near the player are filled from them, caves taken out as today (whole numbers, the same answer). Walking and collision return. | about 2 |

- **2.2's numbers, decided before converting** (so every piece is checked against one plan):
  - **Values** (heights in blocks, shares from 0 to 1, curves' answers, the noise's): `Fix`, 65536ths, about ±32,768.
  - **Positions** (north, east, and the recipe's x and y): a world is 72,000 blocks across by default, past `Fix`'s
    reach, so positions are their own kind, `Pos`: 256ths of a block, about ±8 million blocks. A column's middle is
    exact.
  - **The noise's own coordinates** (a position over a feature size): `Fix` units, wrapping past ±32,768 units, as the
    noise already repeats there.
  - **The maps' fields** (made at world creation, which may use fractions) are turned into whole numbers once as a
    world loads, each in its own kind (heights as `Fix`, distances as `Pos`), and those same numbers go to the card.
  - **Settings** likewise: turned into whole numbers once, on the processor, the same numbers handed to the card.
  - **Curves**: their dots and slopes worked out once from the settings, then read in whole numbers (the bow through
    the kit's `pow`).
  - **The order:** `Pos` and its twin cases; the curves; the noise's entry in whole numbers; the maps' blend; the
    recipe's parts (land and sea, the edge rise and lift, mountains and ridges, peaks and crags, summit crags, the
    torn coast, detail); the river and lake carving; mineralogy and the placer's fields; the cave carve. Each piece
    kept working with its tests as it goes; David's look once the whole recipe is converted.
- **2.2 [BUILT 2026-11-05, build 65]** `world/whole_recipe.rs` (with `placer`'s `WholeProgram` and `WholeReading`, and
  `Mineralogy::whole_layers`): the whole recipe in whole numbers, every number made whole once as a world loads
  (`WholeWorld`: the maps and settings as `WholeMap`, the rivers and lakes as `WholeCarving`), then read with the kit
  alone. The switch is the recipe's `recipe_maths` (whole numbers by default; the Interpreter panel, "Noise maths (for
  now)", "The whole recipe"), saved with the preset and the world: the chunks, the World Compiler's picture, the
  terrain lab and mineralogy all follow it. **Against the fractions** (tests): the ground on a world with every part,
  98% of columns the same block, 0.02 of a block apart on average, under a block at worst; the rivers and lakes, 98%
  the same block and the water the same in every column, every way of banking; mineralogy, all 10,240 columns tested
  made of the same blocks. Two world-creation river tests stood on a knife's edge (heights on exact halves, where a
  hundredth of a block rounds a column to the next block): their test ground was moved a quarter block, and they
  pass in both maths. The cave carve was whole numbers already. **Not yet:** the World Compiler's readouts under the
  mouse (the field view, "why this block") still read in fractions; the maps are held twice while both maths are
  kept (about 180 MB more on a 72,000-block world), until the fractions go.
- **David's first look at 2.2 (build 65, 2026-11-05):** "a noise got way coarser. rivers look ok though, so thats good."
  The whole numbers' ground was grainy on his preset (crags 1.0, peaks 500 high), six times rougher column to column
  than the fractions' (measured, his default preset on a test world). **The cause:** the crags are kept by how steep the
  peak's rise is, read from the rise a block east and south. A value's 65536ths along a 500-block peak's curve make
  the rise step by about a hundredth of a block, and with the crags that strong, the difference of two such readings
  jittered their strength by up to a third of a block a column. **Build 66:** the rise's slope is read off its curve
  (`WholeCurve::steepness`, the Hermite's own slope through the bow) and the distance to the peak in 65536ths of a
  block, not as the difference of two readings: the whole numbers' row now follows the fractions' shape for shape
  (about a third of a block lower: the slope read exactly rather than across a block), the grain down to about 1.3
  times the fractions' where it was 6. The same rule, done more finely. Also in 66: the panel's text no longer
  stays selected when the window loses the focus mid-press (a screenshot taken), the mouse let go of on the way out.
- **David's look at build 66 (2026-11-05): passed.** "looks like its supposed to now, good job. also, it had taken 15
  seconds to compile rivers before, and its back to 10 now." (The rivers' stage reads the ground at every step of
  every river: in build 65 the whole numbers' crags made it slower; 66 is back to the fractions' 10 seconds.)
- **2.3 [BUILT 2026-11-05]** `world/card_recipe/` (`recipe.wgsl`, `Pack`): the whole recipe in the shader language, reading
  the world packed into one list of whole numbers under a header of named slots (the shader's names made from the same
  list), the very numbers the processor reads. **Every piece proven bit for bit against the processor on David's card,
  a twin test each:** the noise, curves and noise layers (32,000 cases); the ground, every part on (about 125,000
  columns, ridges steady and turning); the rivers and lakes, every way of banking (about 37,000 columns each); mineralogy,
  every kind of chooser, the planet hash in 32-bit halves (3,000 columns each); the caves (about 29,000 blocks). On
  Microsoft's software card (WARP, `INCUS_TWIN_CARD=software`) the kit and the noise pass too; the bigger shaders crash
  inside WARP's own driver (parked: the iGPU is the second card to try, after phase 2). **The compile:** the whole column
  (`whole_column`: the ground, carved, and its blocks) compiles in 9.5 seconds, from 57 at first (the compiler copies a
  function into every place it's called: mineralogy's stacks are called from one place, a column's parts worked out
  once). That's at world load, to be done on the loading lane or kept in a pipeline cache. ~~Left for 2.4: the column's
  whole-block rounding and the sea floor's rule on the card~~ (built in 2.0, below).
- **2.0 [BUILT 2026-11-05]** `world/card_recipe/on_card.rs`, with `world::send_world_to_card`: when a compiled world
  starts (or its caves change), a loading-lane job packs it (the maps, recipe, rivers and lakes, caves and mineralogy:
  `Pack`, not the textures first planned, as 2.3 made one list the card reads), puts it on the card, compiles the
  column's pipeline, and checks it: the card makes the 4,096 columns round the spawn and every one is compared with the
  processor's. The runner shares its device for it (`runner::card()`, our own, not through the doorway). **The columns
  as chunks are filled** now round in whole numbers on the processor too when the recipe is whole
  (`terrain::whole_block_column`: halves up; a fraction can't hold a height's 65536ths above Z 256), and the card's
  `world_column` makes them the same: twin test on two maps, one with sea for the floor's rule, bit for bit.
  **Measured, the default bake, David's card (soak runs):**
  - The pack is **178 MB** (45.6 million numbers, nearly all the maps at full size): past wgpu's usual 128 MB a binding,
    so the device now asks for the card's own limit (2,047 MB here; `whole::shader::big_buffers`, the twin test's
    card the same). The first run hit it; the job's panic was caught on the lane and the game flew on.
  - Packed in 243 ms, on the card in 133 ms, the pipeline compiled in **6.0 s**, first used in 15 ms; the 4,096
    columns in 10 ms on the card (the processor 27.5 ms on one core). **All 4,096 the same.**
  - **The card is shared with the frame** (one queue): the pack in one piece held a frame **92 ms**, the pipeline's
    first use with 4,096 columns 26.5 ms. In pieces, each waited for before the next (a megabyte of pack at a time; a
    first use of 64 columns; then 1,024 at a time) they're 6.3 ms and 8.9 ms, once, as the world loads. The first
    use's 15 ms on the card is what's left (a fixed cost: on file).
  - **What the card costs, the recipe as it is.** ~~About 6 million columns a second~~: that was a **dozing card**
    [corrected 2026-11-06, 40 dispatches back to back]. Awake, a whole column (the ground, the rivers and lakes
    carved, the water; the rich test map) takes **about 21 ns, some 47 million a second** (a plain map 12 ns, 83
    million), about 50 times one processor core; dozing, the same work takes about 190 ns. The card wakes after
    about a tenth of a second of steady work and dozes again when left. Mineralogy's blocks on top: about 25 million
    a second awake (estimated; measured dozing only). The shader's tables (`var<private>` or `const`) make no
    difference, tested both ways. **For sight:** the card's work goes in batches the frame can stand beside, and it
    must be kept awake to get these rates (Fork A's dozing again).
  - Noticed, not chased: a compiled world's first 13 chunks took 106 ms on the frame (8.6 ms at build 60), likely the
    maps made whole on the first column asked (`CompiledMap::whole`, once a world), at load.
- **The bake measured for 3.1** [2026-11-06; `INCUS_MEASURE_BAKE=<windows>`, `on_card::measure_bake`]: 64 windows of
  512 × 512 blocks over the default bake (72,000 blocks across, 5.18 billion columns), the card's columns counted for
  the cells holding a surface at each size (caves not counted: the default world has none yet), 16 bytes a kept cell.
  Baked from 256 blocks up 2 MB; 128 up 13 MB; 64 up 56 MB; **32 up 297 MB** (2.7 surface cells a cell column at 32);
  16 up 1,784 MB (4.3 at 16). The card at full batches of 16,384 on the real world: **23.7 million columns a second**
  (ground and water; slower than the test map's 47), so one recipe pass over the world about **3.7 minutes**, with
  mineralogy's colours an estimated 6 to 8. Batches of 16,384 ran beside the frames without a slow frame. The
  processor on the real world: 28 ms for 4,096 columns on one core (about 145,000 a second), so the card carries it.
- **David:** with nothing on screen yet, his looks here are the World Gen suite's pictures: after 2.2, the World Compiler's picture of the same spot before and after (and the count of columns that changed). Walking is felt once there's sight (phase 3.2 on); 2.4's collision is checked by the tests meanwhile.
- **Measured:** the readback's cost, in the slow-frame log.
- **Fork C:** if a saved world's river disagrees with its banks after 2.2, either carve the rivers again on load or accept it. His look decides.
- **Fork D:** if one piece can't be made exact on the card, it stays on the processor (worked out once and uploaded), or its maths is simplified, with his look.

### Phase 3: sight, the twin layers  [planned; each step talked through with David before it's built]

**What David asked for** [2026-11-05]: start from the far view, "blurry up close, but, equally and *correctly* blurry
at a distance"; "a beautiful world, squinted... high fidelity and low resolution"; and not LOD rings: "i would really
like to think of the low-res hi-fi layer and the hi-res high-detail (not exactly hi-fi) layer as two versions of the
same thing; or even as conjoined twins. they should agree from the start like two particles of a decay that are in
a state of quantum entanglement... they're just aimed differently"; "like the cellographs disney studios would use...
the screen pixels are in a grid where each cell is 2x2 or 4x4, and the coarse view paints those cells as pixels, and
the fine view paints the hi-res in the smaller grid"; the fine share "should stay about the same volume of the screen
as the game is built out".

**The design:**
- **One function, two aims.** One shader function answers "what does this patch of world look like, seen at footprint
  s": its solid share, its surfaces' average colour (the blocks' far colours, weighted by how much of each face
  shows), their average facing and how spread out the facings are, its water, and how much it varies (colour and
  depth spread). Both layers call it; the only difference is the pixel's footprint. No separate far code, no
  copies, nothing that switches by distance. Every look setting lives in the one function, so tuning one twin tunes
  both.
- **The cache.** For speed, the function's answers are kept round the camera at a few footprints (cells of 1, 2, 4
  ... blocks, coarser with distance because the footprint is). It's a store of the one function's answers and
  nothing else: **each level exactly the average of the level below** (a standing test). Only the strips the camera
  uncovers are filled, so the cost follows movement. Caves, halls and overhangs are in it from the start, because
  it averages the recipe's real output.
- **Nested layers, like cels.** The coarse layer covers the whole screen at an exact fraction of it (2560 × 1600: 640
  × 400 at a quarter), several rays a coarse pixel, so every coarse pixel holds exactly its 4 × 4 fine pixels. The
  fine layer paints only the cells that need it, at full resolution, each ray starting at its coarse cell's depth.
- **Each cell knows what it needs** (instead of a fixed distance): the coarse pass flags cells by their depth range
  (a near rock against far sky), their spread (a fence, a lone tree, finer than a coarse pixel), and moving things
  (people, models flag their own cells). **The twins' disagreement is the flag.**
- **The fine layer has a budget, with priority** (lane 0.2's flags): the cells that most need it first (nearest,
  most contrast, important, the screen's middle). When the budget runs out the rest stay coarse that frame: honest
  blur under load, never a stutter. Under pressure the coarse layer is what gives way first (redrawn every other
  frame, nudged for the turn): "the farthest and lightest give way first", applied to sight.
- **The coarse factor is a setting** [David 2026-11-05: "i was thinking of basically a 2x2 grid of either 4 or 1 pixel,
  but i suppose larger sizes would be useful since it doesnt really change anything about whats going on"]: 2, 4 (and
  maybe 8) fine pixels a side, chosen by eye at the same spot, the same code at any factor, switchable live if that's
  cheap. Where a block is one coarse pixel moves with it: about 335 blocks at 2, 168 at 4 (2560 × 1600, 100° view).
- **Haze is dealt with when it's seen** [David: "whatever problem haze may make will have to be dealt with when i see
  it"]: the first form keeps a cell's solidity as one number. Solidity by direction stays on file as the known fix,
  built only if his eye finds the problem.
- **Switching without a pop.** A coarse pixel is the average of its fine ones, so a cell going sharp changes
  sharpness, not colour. Hysteresis (once fine, stay fine a while) and a short fade stop it flickering.
- **A progressive blend, valved like a shock** [David 2026-11-05: "putting threshold and blend curves at the exact
  pixels which are flipping from big to small. almost how the more advanced multistage shocks for baja trucks increase
  their damping across their damping range with valves"]: not one threshold and one fade, but a shaped response. A
  cell's flag strength (its twins' disagreement, its spread, its depth range) goes through a curve, soft at the bottom
  and stiffening as it rises, to how much of the fine layer is blended in, with hysteresis. The curve is a few stages,
  each with its own slope, and those are settings he tunes by eye at the same spot, like valving.

| # | Builder does | Hours |
|---|---|---|
| 3.1 | The one function and the cache: the averaged levels filled from the recipe on the card by strips, with the colour and depth spread kept. **Test:** each level exactly the average of the one below. Nothing to see. | 3 to 4 |
| 3.1½ | **The stone** [David 2026-11-05: "we can use this checkpoint we're coming up on as often as necessary until we get a solution that pulls the sword from the stone"]: once 3.1 stands, the commit is tagged `before-sight`, and every attempt at sight starts from there. | — |
| 3.2 | **The first sight: the coarse layer alone** (talked through with David and Counsel before it's built: the first form of the store and the march, and the coarse factor as a setting), over the whole screen at a quarter resolution, several rays a coarse pixel, coarse cache levels only (about 16 to 32 blocks), the sky colour, the fly camera back. The upscale behind a switch (crisp squares, smooth, or smooth within a surface but sharp at a skyline), his call by eye. **Kept for good: it is the final picture's bottom cel.** He sees the whole world to the horizon, squinted, equally soft at every distance, with blobs about 16 blocks wide at his feet. Flying only. | 2 to 3 |
| 3.3 | Finer cache levels, one a build: 8, 4, 2, 1 blocks. Each one a look and a measurement. | about 1 each |
| 3.4 | Light through the cache: sun on each cell's average facing, softened by its spread (a rough far hillside shades as the average of its faces); then sun shadows and sky light by wide rays through the same cache; then fog. One structure for sight and light. One a build, each a look. | 2 to 4 |
| 3.5 | The fine layer: the flags, the list of flagged cells (an indirect dispatch), the budget and priority, the valved blend curve (its stages as settings) with hysteresis and the fade. **Test, the entanglement test:** both layers drawn over the whole screen, each coarse pixel against the average of its fine pixels, at fixed spots and along a flight, within a tolerance set with David by eye (say a mean colour difference under 1%, and the 99th in 100 under a few per cent). | 3 to 4 |
| 3.6 | Block textures in the fine layer where a block is bigger than a pixel; edits override the cache (a small upload when they change); water as a second medium in the cells, a flat colour until phase 6's real look. Walking is back with phase 2.4. | 2 to 3 |

- **David:** a look at every step from 3.2 on; 3.2 is his first sight of the land.
- **Measured:** every step, by the card's own timestamps, with the ballast topping up to the same total so the frame
  loop is tested under a constant load while the real work replaces it (the sandbags come out as the cargo goes
  in). My guess for the whole of phase 3: half to two thirds of one full-resolution march of everything, plus a
  small flagging and sorting pass.
- **The entanglement tolerance can't be exact everywhere,** and that's expected: an average can't say which of two
  things in a cell is in front; light on an average facing isn't the average of the light unless the spread is
  modelled (it will be, approximately); several coarse rays sample differently from sixteen fine ones. Where the
  test reports disagreement is where the method is weakest, and the same cells the flags should send to the fine
  layer.
- **The flagging cautions:** edges inside a coarse cell (caught by the coarse rays' depth range); small things that
  average out faint (caught by the cache's spread); sharpness flickering at the budget's edge (hysteresis and the
  fade); the budget's sort each frame (routine on the card, but it's code).
- **The honest hard part:** averaging how solid a cell is behaves like haze: right for how much of a pixel is covered,
  wrong for a thin wall seen edge-on or a cell full on its bottom half. Known fixes (solidity per direction, the
  surface's height inside the cell) are exactly what the clean stage lets us try side by side, behind a switch, at
  the same spot and along the same flight.
- **Where an elegant method could still come from** (David hasn't given up on "a truly elegant bespoke solution"):
  the recipe answering at a footprint directly. Detail is octaves of noise; at a cell's scale the octaves finer than
  the cell average out, and how much they would have shaken things is known exactly, so the coarse levels could ask
  the recipe for the averaged answer instead of averaging fine samples. "Correctly blurry" straight from the form: the
  projection principle, each footprint a projection of the one recipe. It has to agree with the fine levels, and the
  cache test says whether it does.
- **Fork E, if it's too slow:** the coarse layer every other frame, a smaller fine budget, fewer rays a coarse pixel,
  a coarser coarse layer (a fifth or a sixth); each measured and kept only with its number.
- **Fork F, if it looks wrong** (an edge of sharpness that shows, swimming as the camera moves): a longer fade,
  blending across neighbouring cells, or more rays a coarse pixel.

### Phase 4: [FOLDED INTO PHASE 3, 2026-11-05]

The second band (the backdrop) and its transition are gone: the coarse layer covers the whole screen at every
distance, so there's no band to join. Important things still show from their first pixel through a list drawn on
top, lights at cave mouths at night among them (the firm rule).

### Phase 5: lights and the day  [loose]

Sun, shadows and sky light come with phase 3.4, through the cache. Left here: placed lights in the cells (torches,
lamps, lights at cave mouths), and day and night.
- **Builder:** 1 to 2 sessions. **David:** 1 to 2 looks. 1 test window.

### Phase 6: everything else, each with its price  [loose]

The real look of water (a flat colour since phase 3.6) and glass, pieces and bars, models (drawn as meshes into the same picture: the hybrid), people and figures (drawn, one instanced pass), and the 800-figure stand-in battle if David puts it in the bar. Sized when we get there.

### The whole path, roughly

| Phases | Builder | David's looks | Test windows |
|---|---|---|---|
| 0 and 2 | about 15 to 20 hours | 2 | 2 |
| 3 | about 15 to 25 hours | about 8 to 10 (one a step from 3.2) | 3 |
| 5 | about 1 to 2 sessions | 1 to 2 | 1 |

Fork E could add 2 to 4 hours. (Rewritten 2026-11-05 for the twin layers; the old table counted phase 1's height field
and the backdrop band.)

Then **the bar** (`decisions.md`): runs smoothly and cool on his own flight, the land reads as one land, built things hold up at a distance, heavy content doesn't change the answer. After that: the blueprint factory, then the simulation, which starts with **its column store and scheduler** ("tables that don't know what they hold"; rules that name what they read and write, run side by side when they don't clash; `simulation.md`, rules as algebra), the toy rules rewritten on it as the first test. *Builder:* 1 to 2 sessions; *David:* 1 look.

## Risks, and what David would feel

- **Phase 0 touches 75 files.** The tests and the soak test check most of it. What they can't check is feel: mouse look, keys, the panel's grip on the mouse, the clock's pause and speed. That's his first look.
- **Phase 3's price** is the big unknown. The card measured 3.7 ms for build 60's whole scene with no tricks; the twin layers have to come in well under that, to leave room for light and everything else. Fork E is the answer if they don't. The ballast lets the frame loop be tested at that load before the real picture exists.
- **The recipe on the card** is the biggest single piece of work (2.3). The twin test makes it safe: the card is never trusted until it agrees with the processor exactly.
- **A longer dark stretch:** with phase 1 withdrawn, there's no land on screen until phase 3.2: lane 0.2, all of phase 2 and 3.1 first (about 20 hours of Builder's). What comes up then is kept for good. David chose this: "installing the hologram onto the gpu comes before cracking the eyelids".

## Open, for David

- The audio, gamepads and the real game's menus aren't in any step yet. When does he want them?
- Phase 0 starts on his go, after he's read this.

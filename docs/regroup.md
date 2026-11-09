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
  - **A rule for any card work off the frame thread** [found 2026-11-06, 3.1c]: wait for the card *gently* (wgpu's
    non-blocking poll, a moment's sleep between asks: `on_card::wait_gently`), never with a blocking wait on another
    thread: a worker inside wgpu's blocking wait held the frame's own showing up 5 to 6 ms, whatever the work's size.
    And hand the card its work in small pieces, each waited for before the next (a lump of several milliseconds holds
    the frame behind it). A soak flight with the work going on is the check: the slowest 1% of frames over 200.
  - **Build 67's test window** [2026-11-06, David at the PC; PresentMon, the card's own log beside it; the test window
    loads no compiled world, so the frame loop alone]:
    - **The ballast runs** (90 s each). The game hands over a frame every 4.2 ms like clockwork in every run (between
      presents: p99 about 4.9 ms). What reaches the screen depends on how awake the card is:

      | Run | Frames | Shown later than 5.2 ms | Card |
      |---|---|---|---|
      | uncapped | 1,873 fps | (all under 1 ms apart) | 2,840 MHz, 39 W, 51 °C |
      | 237, no ballast | 237 | 1,525 of 21,309 (7%; p99 8.8 ms) | dozing: 760 MHz (least 345), 16 W, 45 °C |
      | 237, 1 ms | 237 | 1,023 (5%; p99 8.4) | 1,778 MHz, 41 W, 49 °C |
      | 237, 2 ms | 237 | 471 (2%; p99 7.5) | 2,332 MHz, 67 W, 57 °C |
      | 237, 3.7 ms | **226** (the card's time runs out) | 39 (0.2%; p99 4.8) | 2,811 MHz, 105 W (most 119), 67 °C |

      **The dozing card is what's left of the unevenness:** kept busy, the picture is even; at 3.7 ms of work it runs
      out of time. The sweet spot is 2 to 3 ms of card work a frame, which sight's own work is to supply as it comes in
      (the ballast was its stand-in, the sandbags out as the cargo goes in).
    - **The response run, real hands** (60 s each, David moving the mouse; PresentMon with admin): **input to photon**
      uncapped: mean 1.05 ms, p50 1.03, p99 1.85, worst 5.3 (7,304 frames with input); **at 237**: mean 4.65 ms, p50
      4.41, p90 7.15, p99 11.4, worst 13.7 (7,572 frames), about one frame and a bit, the tail the dozing card's.
    - **PresentMon without admin** works when started after the game and aimed at its process ID; started first, it
      can't name the game (scratchpad `run67.ps1`). With admin it follows the game by name, input included.
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

**3.1 in stages** [David 2026-11-06: "go ahead and proceed. we have that checkpoint if we make a mistake. but think
about what parts of what we're doing that might be good and bad and be prepared to telescope back and forth between the
checkpoint as we make progress towards the next one"; the store as decided that morning with Counsel: 3D cells, live
levels up to 16-block cells, baked from 32 up once a world, saved with it and version-stamped, edits patching it; and
"an eighth of the world being immediately present as the rest fill in"]. Each stage is its own commit, a place to step
back to.

| Stage | What | Good if | Bad if (what would send us back) |
|---|---|---|---|
| 3.1a | **The cell, on the processor.** What a cell of any size holds, kept as exact whole-number sums of its blocks: solid blocks, water blocks, the solid faces showing (each a face against air or water, counted once, by its direction: up, down and the four sides), their colours (each face its block's far colour, top or side, from `look::Palette`, in 1,024ths of linear light), and for the spreads the faces' brightness and height, summed and squared. A cell's sums are its eight children's added, exactly. The compact form the march reads is worked out from the sums once (shares, colour, average facing, the spreads: about 16 bytes). **Tests:** the sums of any box are the sums of its parts; the compact form follows its rule. | sums make "each level exactly the average of the one below" true by construction, not by luck | the sums are heavy (about 70 bytes a cell): only ever held while building, never stored at baked levels |
| 3.1b | **The one function on the card:** the sums of a cell of any size straight from the recipe (columns, mineralogy's blocks, the caves), every block inside and the faces at its edges. **Twin test:** the card against 3.1a's, bit for bit, cells of 1 to 64 blocks, on ground, river, lake, sea and a cave. | the card makes cells as fast as the columns (the column is most of the work) | mineralogy for every exposed block makes it slow: then the colours become their own pass |
| 3.1c | **The live levels** (cells of 1, 2, 4, 8 and 16 blocks): round the camera, filled by strips on the loading lane in batches the frame stands beside, nearest and ahead first, each level built from the one below where they overlap, from the recipe where it's new. **Test:** every level's cells exactly the sums of their children. | flying at 150 costs a few per cent of the card | the 3D sparse layout (each cell column's surface band) is fiddly; the clean fallback is a fixed band per cell column |
| 3.1d | **The bake** (cells of 32 blocks and up, doubling till one cell covers the whole world: no far cut-off, fog is weather [David 2026-11-06: "on clear days from a high point you should see across the entire world"]; past the start bake each cell starts as one true sample and eases to its exact average, outward from the player, in view first, "like sliding the ring on a nice lens"): an eighth of the world round the spawn before play (about 30 to 60 seconds on David's card), the rest outward in the background at the lowest priority; saved with the world as it completes, stamped with the rules' version (a mismatch bakes again); an edit makes its 32-block cell again from the recipe and its parents from their children. **Tests:** baked cells equal the same cells made live; a saved bake reads back the same. | the far world costs nothing to fly over once baked | the start-up wait or the file size grows past what's tolerable: the reach at start is a setting |

- **3.1a [BUILT 2026-11-06]** `sight/cell.rs`: `Sums`, `Cell` (16 bytes), `sums_of_box`, `FarColours`.
- **3.1b [BUILT 2026-11-06]** `sight/cell.wgsl`, run by `WorldOnCard::cells_at`: three passes over a region of columns
  (the grounds with three round them for the slope; mineralogy's choices with one round them; then the cells), each
  column looked at block by block only where something changes (below its own and its neighbours' ground it's solid
  all round, above its own ground only water and air), block by block wherever a cave box or a layer of air or water
  comes near. Mineralogy's candidates now pack what they put down (`Layers`), and the pack holds the blocks' far
  colours. **Twin test:** 67,134 cells of 1 to 64 blocks (a river, a lake, tunnels coming out of the ground, the map's
  edge), the card bit for bit the processor's own chunks; and at every world load the cells of 32 round the spawn are
  checked against the chunks (`check_cells`; the default bake: all the same). **Measured:** a cell first went to one
  thread on the card, and four cells of 32 held two frames 26 and 19.5 ms; now each cell is shared by 64 threads, and
  they take 6.6 ms. Cells of 32 through the surface, mineralogy and colours and all, at **30 million columns a
  second** in windows of 512 × 512: the whole default world's in about **3 minutes**, an eighth of it about 20 s. The
  three passes compile in about 9 to 11 s on the lane, on top of the column's 6 (world load ready on the card at about
  17 s: to trim later, or keep in a pipeline cache). Glass and other see-through blocks count as solid for now (the
  terrain has none).
- **3.1c [BUILT 2026-11-06]** `sight/store.rs` (`LiveStore`, `fill_tile`), driven by `world::keep_sight`: five levels
  (cells of 1, 2, 4, 8, 16 blocks), 128 cells a side each round the camera, in tiles of 16 × 16 cell columns made on
  the loading lane, nearest first, three at a time, let go as the camera leaves them; each cell column keeps only its
  surface band (solid all round below, empty above; reaching down where a cave box comes near, from the bottom where a
  layer is air or water or the map ends). Kept on the processor as sums for now: how the store sits on the card for
  the march is 3.2's talk ("the first form of the store and the march"). Every level is made from the recipe (building
  from the level below doesn't help the leading edge, which the finer level never covers). **Test:** every cell is
  exactly its eight children at the level below added, the cells left out of the bands included, all the way up, at
  every level by caves and a river (10,606 cells kept), and 4,096 cells just past a band made directly and found
  solid all round or empty. **Flying the soak (150 blocks a second):** every level kept full or a row behind, about
  105,000 cells (13 MB as sums), about 225 tiles a second. **Found on the way** (the frame first): tiles in one go held
  frames (215 to 226 fps, 900 slow frames in 40 s, "showing it" 4.7 ms); the columns made once instead of twice and
  every pass in pieces (8,192 columns, 512 cells) helped (about 3 slow frames a second); what cured it was **waiting
  gently on the loading lane** (asking the card without blocking, a moment's sleep between asks): wgpu's blocking wait
  on a worker held the frame's own showing up whatever the pieces' size. Now 237 fps, the slowest 1% over 200, 4
  slow frames in 40 s. **On file:** the first use of sight's three pipelines at load holds one frame about 26 ms
  (once a world, with the other load costs); building a level from the one below kept for when the card is short.
  **Where it lives** [for 3.2's talk]: every cell is made on the card; only the results come back, held on the
  processor as sums (which made the exact test possible). The workers only work out the bands from the read-back
  grounds and sleep in the gentle wait. At 2,000 blocks a second the finest levels fall behind (each tile 2 to 30 ms of
  round trips, about 500 tiles a second at three at once, the finest level uncovering about 1,100 a second), so
  coarser cells show till they catch up. **For 3.2:** the store on the card, the cells pass writing straight into it,
  many tiles a pass, the bands worked out on the card too; the processor keeps only which tiles are where, and nothing
  is read back in play (the bake's cells once, to be saved).
- **3.1d split in two** (2026-11-06, as it was built): **3.1d, the bake** itself, and **3.1e, keeping it**: saved with
  the world as it's made, stamped with the rules' version (a mismatch bakes again), and patched for edits. Edited
  chunks aren't in the recipe: a changed chunk is exactly one cell column of 32, made from its blocks instead (the
  processor's `sums_of_box` over the chunk, its neighbours' faces too), and its parents again from their children.
- **3.1d [BUILT 2026-11-06]** `sight/bake.rs`: cells of 32 made on the card a region at a time (1,024 blocks square,
  the live levels' band rule), every level above built exactly from its four children (`parent_of`): within a region
  up to 1,024, and above as soon as all four children are baked, up to the level where four cells cover the map (17
  levels' worth for the default world: 32 to 65,536). Nearest the spawn first in square rings, so the start bake (the
  smallest square of rings holding an eighth of the regions: 729 of 5,184 for the default world, the rounding to whole
  rings making it about a seventh) is done before anything past it; four loops on the loading lane at its lowest
  priority, pieces of 32,768 columns waited for gently. Kept compact (16 bytes a cell, flat arrays by level; exact
  sums only from 1,024 up, to build the levels above). Nothing reads it yet (3.2), and nothing waits for the start
  bake (the loading screen's question). **Test:** the whole test map baked (16 regions); its cells of 32, 64 and 128
  the same as those cells made directly on the card, the first past each band included; every level complete to the
  top. **The default world, flying the soak at 150:** the start bake in **46 s**, about 20 regions a second after it
  (the whole world in about 4 1/2 minutes), 237 fps throughout, the slowest 1% about 208, 11 slow frames in two
  minutes; about 16 bytes a kept cell, some 100 MB at half the world. (Two loops: 65 s to the start bake; the gentle
  waits cost the bake speed, more loops get it back without the frame noticing.)
- **The start bake's wait** [David 2026-11-06: "yes, we can just do a loading screen. but id like to see the
  sharpening myself, realtime and raw, until we are doing finishing touches"]: the finished game shows a loading
  screen while the start bake runs and plays with the start reach sharp; until the finishing touches our builds skip
  the wait and spawn straight in, the bake sharpening the world live and raw (no fade, nothing hidden): an instrument
  for seeing how the store fills, a switch in our builds only. **For 3.2:** the march reads the store as it fills.
- **3.1e [BUILT 2026-11-06]** (`sight/bake.rs`, the world state "sight" in `world/mod.rs`): **the rules' stamp**
  (`on_card::stamp_of`: the world's whole pack, the maps, recipe, rivers and lakes, caves, mineralogy and far colours,
  hashed with the cells' code's version `SIGHT_RULES`); **the file** `sight-bake.bin` in the world's folder: the
  stamp, then a record a region (its compact cell columns from 32 to 1,024 and its 1,024's exact sums), each save
  adding the records made since the last one to the end (a new place, or a fresh start, writes it whole); read back
  at load and taken whole if the stamp and the map's size are the same, a later record for a region over an earlier
  one; a stale one is baked again and written afresh at the next save. **Edits:** a cell column of 32 whose chunk or
  a chunk beside it has changed is made from the blocks themselves on the processor (`column_from_blocks`: the changed
  chunks as they are, the rest as the recipe makes them; every cell of the column, trimmed to its band), and its
  region's levels built up again; edited regions are patched once left alone 2 seconds, and at the bake's start every
  region with changed chunks is made again whatever the file held (a save may catch a patch under way). **Test:** the
  test map baked, kept, read back record for record the same; a bake started with it done before a region is baked,
  with another stamp baked again; a hole of 54 blocks dug at a chunk's edge: the region's top cell exactly 54 blocks
  lighter, faces bared in the column across the edge, a column away from it unchanged. **Then, at Counsel's asking
  before the first sight** (the save on the frame would freeze it, about 300 MB for a whole world's first save; and a
  column made again left its old cells behind): **each region is kept once**, as a shared record the levels read their
  cells from (the levels above 1,024 by cell column), so a region made again replaces its record whole, nothing left
  behind and nothing held twice (it had also kept every region twice, the levels and the records for the file);
  **the save is written on the loading lane** (`keep_later`: what's to be written taken in a moment, the records
  shared not copied, written in turn after any save before it), and **a file cut short** (the game closed while
  writing) reads back as far as it's whole and is written afresh at the next save. Tests: kept on the lane, the same
  file; cut short, the whole records before the cut; a region patched inside the bake, its cells replaced (22,543
  cells before, 22,536 after: just the patch's own difference). **On file:** the live levels don't show edits yet
  (3.6, "edits override the cache"); the rest of a save (the chunks) is still written on the frame, small next to
  this; the start's burst of slow frames, the bake's four loops beginning on a dozing card ("getting the next
  picture", 66 to 141 frames over about 5 s, once a world: with the loading screen).
- **Telescoping:** `before-sight` is tagged on the commit with the save on the lane (moved there from 3.1e's first
  commit, 6312521, which it supersedes): every attempt at sight starts from there.
- **3.2 [FIRST FORM BUILT 2026-11-06, build 68, for David's first look]** `sight/card_store.rs` (the store on the
  card: one pool of cells with a first-fit allocator on the processor; the baked levels' index over the map; the
  live level's wrapping window, each entry naming its column), `sight/march.wgsl` and `sight/upscale.wgsl` driven by
  `sight/draw.rs` from the runner (sight before the panel, in Play). The live level is cells of 16, sized by the rule
  (`store::side_for`: 1,342 ÷ the grid, whole tiles: 672 a side at 2 × 2), the baked levels from 32 up; the switches
  on the Render page (grid 2, 4, 8; rays a coarse pixel 1 or 4; upscale crisp, smooth, or sharp at skylines; a new
  grid sizes the live window again and it fills again, raw). **The march:** the level whose cells are its pixel's size
  at that distance (never finer than 16), no threshold, the eight cells round it blended (each cell column's entry
  read once), coverage gathered front to back, colour and facing weighted by each cell's surface, lit by the blend's
  facing under a noon sun, the sky behind. **Found getting there** (by a measuring view, `INCUS_SIGHT_STEPS`: each
  pixel coloured by its ray's steps): rays crossed regions not baked yet cell by cell (now skipped whole: the raw
  view shows sky there); the sky over distant ridges and the sea's skimming rays took hundreds of steps (now skipped
  by coarse cell columns' tops, the heightfield way, and two cells a step through air, safe for anything a cell
  thick); coarse steps by a surface drew rings round the eye (quarter steps by any surface, and each pixel's first
  step staggered, a steady hash); the world's floor past the map showed as a grey band (past the map is nothing
  now). **Cost, David's screen (2560 × 1600), uncapped, flying the soak:** grid 2 × 2 about 140 fps (sight about
  6 to 7 ms, over the 2 to 3 ms budget; capped at 237 it runs at about 140); 4 × 4 about 280 fps (about 3 ms). What's
  left is rays grazing ridges near the horizon, stepping at their pixel's size. **Not yet:** the 45° and 30° slope
  tests (next), the price in the torque spec by the card's own clock.
  **David's first look** (build 68, 2026-11-06, trying the switches): "8x8 actually looks good for the far distance,
  on this screen." At 8 × 8 a coarse pixel spans about d/84 blocks: it meets the 16-block cells at about 1,340
  blocks and the baked 32 to 128 further out, so with today's store (coarse there anyway) 8 × 8 loses little far
  off, at about a sixteenth of 2 × 2's rays. Its cost to be measured after his look, beside 2 × 2 (about 140 fps
  uncapped, sight 6 to 7 ms) and 4 × 4 (about 280, about 3 ms). Counsel's thought for 3.5, not a change: the twins
  may end up as an 8 × 8 coarse layer with the fine layer where flagged, rather than 2 × 2 everywhere.
  His other notes: "i think the contrast on the mountains colors is more of the reason its hard to read at 8x8, but,
  it looks nice at 8x8, and frankly the farthest mountains may not benefit from more... the mountains at that extreme
  will probably look better with forests on them and stuff. there's a severe ringing effect on flat surfaces or water
  beneath me that has to be taken care of but i'm liking what i'm seeing. i think we may be able to make this work".
  **The ringing** (a hole, fixed at once): the first step to touch a surface after a two-cell stride through air
  landed wherever the stride fell, up to two cells deep, and what it gathered swung with that: contours on a flat
  plane, worst looking down from up high (the first stagger, a few blocks, only covered the fine steps' phase). The
  cure: back to the last air sample, then into the surface in fine steps staggered per pixel (a steady hash), fine
  steps kept up till past where it first touched (two loops found and closed on the way: a skip from the backed-up
  place leapt back to where it was; a first fine step in air started striding again; both left rays out of steps,
  white specks, 50 fps). Checked from 400 blocks up looking 40° down: smooth, no rings, no specks. **Cost after the
  cure, David's screen, uncapped, flying the soak with the start bake on the card:** 2 × 2 about 120 fps (sight about
  7.5 ms), 4 × 4 about 250 (about 3.4 ms), 8 × 8 about 340 to 376 (about 2 ms).
  **Build 69's look** [David: "the oceans still show the rings. but the rings are so fast that they seem like a
  physical artifact of the eye... i think it's a great start. im concerned about performance however... the far view
  is turning out to look exactly how i imagined when i said, 'what if we just use a lower resolution except up close
  and let it be like a dos game far away'"]. **The sea's rings, measured** (`sight_over_a_flat_sea_holds_still`, a
  test that draws a made-up sea off the screen with the real march, from places a fraction of a block apart): the
  per-pixel shimmer was about half a level of 255 (unseen), but where the store's levels change, a step of about 2%
  in the sea's brightness: a ring round the eye that moved with it. **The cure:** over the last stretch of each
  level's distances the two levels blended (from 0.6 of the way to the next), never a seam; the step is now down in
  the grain (about 1%). Each step is also gathered from both its ends now (trapezoids), which helped little here. In
  the game the sea is one even colour to the edge of the bake. **Cost, uncapped, flying:** 2 × 2 about 112 fps, 4 × 4
  about 255, 8 × 8 about 270 to 332. His notes then: "there really is no mismatch at all, which sparks joy", and "the eye
  sees it as dirty glass rather than 'what am i looking at on a computer screen'" (Counsel's rule in decisions.md: fail
  like a lens, never like a computer). **The screen pyramid** [David: "i think 1 - 2 - 4 - 8 may be good... the question
  is, can it be fast"; "2 looks worse than 4, up close. 4 looks worse than 8 up close at a different range. so theres
  definitely a curve of bands at distances where each scale is preferable"]: 8 × 8 over the whole screen, each part
  refined to 4, 2, 1 only where it's earned, each finer ray starting at its parent's nearest touch; the first term of
  where to refine: only while a pixel is bigger than the store's cells under it (finer pixels than the cells magnify
  the blend into smudge: his bands). In order, one a build on his word: (a) the 8 × 8 pass giving every ray of the
  coarse layer where to start; (b) 4 × 4 refining from it; (c) where to refine; (d) 2 and 1, each with the 3.3 store
  level that earns it. **(a) BUILT, build 71:** the start pass (`march.wgsl` `start_pass`: one ray down each 8 × 8 tile
  taken as a cone holding the whole tile, only skipping, stopping where the cone could first touch; every ray of the
  tile starts there). **Measured honestly, it barely helped** (2 × 2 110 fps with or without it, 4 × 4 and 8 × 8 a few
  per cent): the columns' tops had already taken most of the air. What the steps view showed instead: (1) sky rays
  climbing to the 1,024-block ceiling, though nothing stands near that high: now the store keeps the highest thing on
  the card so far (to a cell) and rays over it that aren't coming down stop; (2) rays skimming a surface crossing its
  soft edge in quarter steps for a long way: now steps by a surface are sized to how fast the ray crosses it (a quarter
  of a cell looking down, up to a whole cell skimming; both ends of each step kept the sea test clean). And (3), the
  biggest: **the 40-second measurements were taken mid-bake, in the raw view**, where rays over unbaked land plod in
  air strides (on the fully baked test sea, sky rays take no steps and sea rays about 50). Flying six minutes at
  2 × 2 uncapped, as the bake filled in, the frame rate rose from about 120 to about 180 to 260 (sight about 4 to 5 ms
  once baked round about, not 7.5); and uncapped, the march takes the whole card and starves the bake (1,815 of
  5,184 regions in six minutes, the start's eighth in 149 s): in play, capped, the card has room for both. 40 s
  figures, mid-bake: 2 × 2 about 125, 4 × 4 about 285, 8 × 8 about 345. A measuring test, `sight_s_steps_by_band`
  (ignored), reads steps a ray by band of the screen. **(b) and (c) BUILT, build 72** (David: "you may proceed"):
  `draw.rs` marches a pyramid of layers, 8 × 8 everywhere, then 4 × 4 and 2 × 2 down to the finest chosen
  (the Render page's "Finest layer"), each finer layer marching only where the pixel above said it's worth it
  (`worth_in`/`worth_out`), each finer ray starting at its tile's start; the upscale shows each screen pixel's
  finest layer marched there. **The first rule for "worth it"** (from his bands): only while the finer pixel would
  still be at least as big as the cells it reads at that distance (16 in the live window, 32 past it), never for sky;
  "everywhere" is the switch beside it, for comparing. Near the eye it stays 8 × 8; with distance it steps to 4 and
  2. **Measured, uncapped, David's screen, mid-bake:** finest 2 × 2, only where worth it, about 210 to 250 fps
  (everywhere about 97; the single 2 × 2 layer was about 125); finest 4 × 4, about 240 to 295 (everywhere about 200
  to 225). Not yet: the valved blend at the layers' seams (now a hard change of pixel size), edges and importance as
  further terms of "worth it", the 1 × 1 layer (with 3.3's finer cells). **Build 72 closed on F11** [David: "i hit f11,
  and it closed immediately"]: the screen's picture set up again (any change of the window's size) needs wgpu to find
  the card's queue empty, and sight's workers (the bake, the live levels) were handing it pieces from other threads
  (wgpu: "It can only be non-empty if another thread is submitting at the same time"); the failure was fatal. Now
  **the card's gate** (`runner::CARD_GATE`): the workers hold it shared while they hand the card a piece, the frame
  holds it alone while it sets the picture up again; and a failed set-up is caught (an error scope) and tried again
  next frame, never fatal. Tested: the window resized every 2 s for 80 s with the workers busy
  (`INCUS_SOAK_RESIZE_EVERY`), 41 times, no failure. Build 73. **The layers' seams blended, build 74:** a pixel's
  word for the layer below is now how much it's worth, 0 to 1 (nothing while the finer pixel would be smaller than
  the cells, rising to all of it over the next doubling of distance), the finer layer kept premultiplied by it and
  laid over the coarser one by it on the screen: the change of pixel size fades in with distance instead of drawing a
  ring round the eye (the sea test's worst shimmer back from 16 to 5 or 6 of 255). One smooth stage for now; the
  valved curve's stages as settings can grow from it. Uncapped, finest 2 × 2: about 215 to 257 fps. **The 45° and 30°
  slope tests dropped** [David 2026-11-06: "im not worried about the slope tests at all now that i see it in
  person"]. The test that draws off the screen (`sight_scene`, `sight_picture`) is what the
  45° and 30° slope tests will use.
  **3.3, finer cells near the eye, the first step BUILT, build 75: cells of 8.** The store on the card keeps several
  live levels now, the finest (`world::FINEST_LIVE`, 3) up to 16, each its own wrapping window in the live index, all
  the same cell columns a side (each reaches to where one of its cells is one finest pixel: at 2 × 2, 672 columns, so
  cells of 8 reach 2,688 blocks and 16s 5,376). The header gained the windows' most side (`header[4]`; the baked
  levels' entries now from 5). The march reads the level its pixel asks for and, where that one isn't there (past
  its window, or not made yet), the next coarser, level by level down to the bake. The pyramid's "worth it" knows
  the cell sizes by distance: 8 inside the 8s' reach, 16 inside twice that, the bake's 32 past them; so the 4 × 4
  layer now starts at about 1,340 blocks instead of 2,680. A tile of 8s takes about 34 + 40 ms to make (16s: 88 +
  88). **Measured, uncapped, finest 2 × 2, mid-bake, flying the soak:** about 219 to 248 fps, the same as build 74.
  The level-tint view shows all the near ground read from the 8s. Next, on his word: cells of 4, then 2 and 1 (with
  the pyramid's 1 × 1 layer).
  **On file, David's near look** [2026-11-06, by phone through Counsel: "its very strange to see the blocks up close
  exist only as clouds. there are odd whorling effects as i fly through a mountain... when i flew up it was like a
  white cloud"]. One hole: a cell's average is read wherever the cell is bigger than its pixel, and there it's a fog,
  not the truth (the clouds; the white cloud with the eye inside a part-snow cell on his peak; the whorls flying
  through partly solid cells). Cells of 4 and 2 only thin it. **The cure: the block level read as blocks, hard
  faces** (nearest block, no blend, the ray stopping on the face), the edge softened to one pixel's width so it meets
  the average as a block shrinks to a pixel; with it the worth rule's near term (the 1 × 1 layer worth it on block
  faces bigger than its pixel). Proposed to land with 3.3's last step, perhaps moved ahead of 4 and 2 (his call).
  **His call too: flying inside rock**, a wall of stone, or each ray skipping the solid it starts in (spectator-like).
  [David: "i think you're right about 1's next... don't worry about flying through the rock and the crazy
  average-world, i was just telling council how interesting it was to be in that kind of 3d space"]
  **3.3's second step BUILT, build 76: the blocks themselves, as blocks.** The live levels kept are now a list (cells
  of 1, 8 and 16: `world::LIVE_LEVELS`); a level between them isn't kept (`header[5]`, a bit each; the baked entries
  now from 6), and the march reads the next coarser kept one. **Near the eye, out to where a block is one of the
  layer's pixels (and inside the blocks' window, 336 blocks at 2 × 2), the ray walks the blocks one by one**
  (`march.wgsl` `hard`: block to block across whichever face comes first, skipping air over the columns' tops of 64,
  32, 16 and 8, stopping over the world's highest point) **and stops on the first face it meets, lit by that face's
  own facing: hard edges, real terraces.** Water's surface is met the same way (mostly covered, the rest seen
  through). Not met by there, the march goes on with the averages, which at that distance are a pixel wide. A block
  is drawn in its cell's colour (its shown faces' average: a grass block on an edge is a little earthy on top too;
  each face's own colour comes with the textures, 3.6). **The worth rule:** where the cells are blocks, a finer pixel
  always shows more of them, so the finer layers are marched all through the blocks' reach, fading out over its last
  quarter. **The live tiles' order:** nearest first measured in each level's own cells, so the levels fill side by
  side (by blocks, the blocks' tiles always came first and the coarser levels starved: 14 of 1,764 tiles of 8 made
  in 40 s). **Not yet:** cells of 4 and 2 between, so past the blocks' reach the ground goes to the 8s' soft band at
  once. **Measured** (a low, slow flight, 30 above the ground at 20 a second, looking 20° down, uncapped, finest
  2 × 2, mid-bake, runs alternated): about 135 fps with the blocks, 153 without (`INCUS_SIGHT_NO_BLOCKS`, a
  measuring switch): **the blocks cost about 1 ms.** A tile of blocks takes about 25 + 37 ms to make. **David's look
  at build 76** [2026-11-06: "this honestly looks amazing. its not done, there's no textures, but, the things that
  used to be glaring problems before are silky smooth now... its so accurate and the fade is cromulent. we are really
  on to something"]. On file from his screenshot, not to fix unless he asks: where a slope runs out of the blocks'
  reach, its pixels speckle between hard face and soft cell (each ray either met a face by there or didn't), like a
  dither.
  **3.3's last cache step BUILT, build 77: cells of 4 and 2 between** [David: "go ahead. the only irritation i have
  is it takes forever to load chunks around me... hopefully we can speed it up or make it go to the closest chunk
  first"]. The live levels are now 1, 2, 4, 8 and 16. Found and changed on the way, each measured (hovering 40 up at
  237, finest 2 × 2; and flying the soak at 150 a second):
  - **The tiles made in two lanes:** the fine levels' (1, 2, 4) up to 8 at once, the coarse ones' (8, 16) 2. A tile
    of 16s is about 8 times a tile of 8s' card work and thousands of times a tile of blocks; together, the coarse
    ones held the near blocks up behind them. Now every tile of 1, 2 and 4 round the eye is made in about 40 s
    hovering (the nearest in the first few), the 8s and 16s going on behind.
  - **What made the frames slow was drawing, not making:** with the fine levels made but no 8s and 16s, hovering
    ran about 96 fps with nothing left to make. Two causes. (1) **The worth rule went by the levels kept, not the
    cells read:** finer pixels were marched wherever a fine level could be, even where its tile wasn't made and the
    ray read the bake's 32s (smudge, at the most cost); now a finer layer is worth it by the cells this pixel's
    surface was actually read from. (2) **A sign turned since build 68:** in the march's skip over a column's top,
    where a ray coming down meets the top came out behind it, so rays coming down never skipped to a top, only
    strode through the air; and with the 8s and 16s not made, the fine levels' rays had no coarse tops to skip by
    either (now the bake's 32s are tried between). The sea test is cleaner for it: the biggest step down the middle
    335 → 59 of 2,550, the shimmer as the eye rises 0.68 → 0.18 of 255 a pixel on average.
  - **Measured:** hovering, the levels filling, about 162 to 188 fps (before the two fixes about 100 to 125; the bake
    alone holds 237); flying at 150 a second, about 205 to 225 with 8,872 tiles made in a minute (before: about 111
    and 3,986); the low slow flight about 157 at 237. Measuring switch: `INCUS_SIGHT_FILLING=<fine>,<coarse>`.
  **David's look at build 77** [2026-11-07: "looking good, builder. its definitely better. chunk loading isn't super
  important, we just want to make sure its not inefficient or anything. you may proceed"].
  **(d)'s last step BUILT, build 78: the 1 × 1 layer.** The Render page's "Finest layer" now goes to 1 × 1: the
  pyramid 8, 4, 2, 1, each finer layer marched only where the one above says it's worth it, by the same rule (on
  the blocks, a finer pixel always shows more of them, out to the last quarter of the blocks' reach; on cells of 2
  and up, only while its pixel would be no smaller than the cells). In practice the 1 × 1 layer is the blocks round
  the eye: full screen pixels on them, out to about 336 blocks. The live windows stay sized for 2 × 2 (the 1 × 1
  layer reads the same ones; sizing them for 1 × 1 would be four times the tiles and the card's room for every
  level, and the blocks' walk would reach about 670). **Each finer ray now starts a little short of the nearest
  touch round its parent** (its parent pixel and the eight round it, backed off two blocks and two of the parent's
  pixels, for grazing rays), not back at its tile's start; the parent's pixels are no wider than a block where the
  blocks are walked, so nothing a block big hides between them. Screenshots with and without it match.
  **Measured** (the low slow flight, 30 up at 20 a second, 20° down, uncapped, runs alternated): finest 1 × 1 about
  104 fps without the parent's start, **about 163 to 171 with it**; finest 2 × 2 about 168 to 177 without, **about
  201 to 218 with it**. Measuring switch: `INCUS_SIGHT_NO_FOLLOW`.
  **David's look at build 78** [2026-11-08: "obviously no textures but its looking good"].
  Next: 3.4 (light through the cache), talked through first.
  **3.4a BUILT, build 79: a cell lit as its faces are** [talked through 2026-11-08, David: "that sounds right, sun
  sliders are a good idea"]. A block's faces only ever look six ways, so a cell now keeps each way's share of its
  faces (255ths, five kept and west's what's left, rounded as running totals so the six add to 255 exactly) in place
  of the one average facing and its spread (an average loses faces that cancel: a terraced hillside's tops and sides
  lit as one tilted plane, a smooth slope and a flight of steps lit the same). The march lights each way by the sun
  as itself and weighs it by how much of that way the eye sees (a face looking north shows as much as the eye looks
  south), so a far hillside shows the tops and sides it really shows, as bright as they really are: the same rule
  the blocks near the eye are lit by, one face at a time, so the two agree where they meet. Still 16 bytes a cell;
  the bake's file is version 2 (an older one is baked again, not refused). **The sun on the Render page**: from
  (compass) and height, the `light` group's `sun_from` and `sun_height` (the one sun: the World Compiler's pictures
  take it too; the Render presets hold 315°, 40°, a sun from the north-west, in place of the first sight's fixed
  noon). Cost: a few more sums a sample (unmeasured; the sea test's shimmer 0.12 of 255 a pixel against 0.14 lit the
  old way, same view).
  **On file, not chased** (unseen in play): the sea test's biggest step down the middle is now about 610 of 2,550,
  where build 77's note says 59; it's the horizon (sky to sea, inside one 8 × 8 tile at row 56), the same lit
  either way, and the same without build 78's parent start. The test's own note (the horizon at row 32) is off too.
  **The one colour a cell** stays: from above, a grassy hill's cells mix grass tops and dirt sides as a side view would.
  A colour for the tops apart from the sides needs room in the cell; held till David's eye asks for it.
  **David's look at build 79** [2026-11-08: "sun looks good. updates instantly"]. He tried "finer layers
  everywhere" a while: "it looks nice but is slow. at least the choppiness we have is very even, its not as
  offensive as irregular lag/chops" (the slow frames in that run's log, 6 to 8 ms, were that).
  **A deep dive into speed, later** [David: "once we've got the graphics engine mostly working we should do an
  marianas trench dive into ways to accelerate and optimize"]: once the look is mostly working, a pass given over to
  making it faster, every idea measured (Fork E's list is a start; "finer layers everywhere" is what to win back).
  Next: 3.4b (sun shadows and sky light), talked through first.
  **3.4b's first half BUILT, build 80: sun shadows** [talked through 2026-11-08, sky light the next build, each
  measured alone; David: "proceed"]. The light is kept in two parts, the sky's (always there) and the sun's, which
  a shadow takes away; each ray gathers the sun's part beside its colour, and where it was half covered a second ray
  goes towards the sun through the same store. From a block's face near the eye it walks the blocks themselves first
  (up to 128), so a block's shadow is a block's; past them, or from a far surface, it reads the averaged cells,
  widening 0.04 a block (wider than the sun's own 0.01: fewer steps, and a far ridge's shadow comes soft), each step
  as long as it is wide, dimmed by how solid the cells are. Air over the biggest column top it's over is skipped
  (tried from the coarsest level down), a region not baked yet is passed whole, and over the highest anything
  stands it's done. Only pixels shown are shaded (one a finer layer covers whole isn't). Water is shaded as a face
  in full sun. The Render page's Sun has a "Shadows" switch (on unless chosen); `INCUS_SIGHT_NO_SHADOWS` for
  measuring. **Test** (`sight_s_shadows_fall_where_they_should`, card, ignored like the sea test): a ridge on a
  plain with a 15° sun beyond it darkens the plain about 16%, near and far; the sun on this side changes nothing.
  Its first run caught a surface shading itself (the shadow ray stepped clear by a block where the cells read were
  32: now by the cells read). **Measured** (the soak's low slow flight, 30 up at 20 a second, 20° down, uncapped,
  a fresh test world baking as it flies): shadows off about 140 to 160 fps, on about 105 to 120, roughly 2 to 3 ms.
  The first try was 38: in land not baked yet each shadow step read up to eight levels and found none.
  Also: a soak or test window never freezes from its first frame (one came up behind another window, froze before
  its setting arrived, and waited for a click).
  **David's look at build 80** [2026-11-08: "the shadows appear to be cast correctly, they work. seems like we are
  getting the looks right easily with our mipmarching, but we're gonna need kung fu to make it fast"]. His F12 (a
  mountainside at a 30° view, still filling in): flat smears (cells of 32 read where the finer live tiles weren't
  made yet, gone as they were) and rows of dark triangles down a sunlit slope, also gone once the tiles were in.
  The triangles were the shadows: a shadow ray stepped clear of its surface by the cells read there, then crossed
  into a tile not made yet, fell back to the bake's 32s, whose soft edge reached above where it started, and shaded
  the slope itself. Textures wouldn't have cured it.
  **Build 81**: a shadow ray trusts a cell only once it has climbed a whole cell of it over where it started (the
  normal reads always have; only those fallbacks didn't). **The coarsest layer a choice** [David: "a set of
  selections for coarsest rays, so i can experiment with nixing the 8x8 layer and only using 1, 2, 4, etc."]: the
  Render page's "Coarsest layer" (8, 4, 2 or 1 × 1; 8 unless chosen), the pyramid starting there, everything marched
  in it; finest choices coarser than it greyed. Checked on the card: the ridge scene's middle pixel the same, within a
  point, from each coarsest (the test's bar now 10% darker, not 15: at 4 × 4 the shadow darkened 14.6%).
  **1.5 × pixels** [David: "is 1.5x pixels conceptually possible in this system?"]: yes. A layer is only a grid of
  rays; the store and the march don't care how big it is. What a non-halving layer needs: its parent found by scale,
  not by halves (the worth rule, the parent's start), and the upscale to take a layer whose pixels don't line up
  with the screen's (crisp, they'd come one and two screen pixels wide by turns; smooth, a little soft, as games'
  render scales are). Cost about 0.44 of 1 × 1 where it's shown. Not built: his word first.
  **David's look at build 81** [2026-11-08: "i liked the looks of 2x finest, 4x coarsest. it's worth adding more
  render settings to check out what different combos look like and how quick they are"; on 1.5: "perhaps to help
  blend the 1x1 to 2x2, which is a little obvious sometimes when you can look up a big slope. do you believe we'll be
  able to tune the ways that it pumps between scales?"]. The start bake took 221 s that run (92 in 80, 46 before
  shadows): his 1 × 1 coarsest left the card the least room; on file for the speed dive (the bake's share of the card).
  **Build 82**: **1.5** a finest choice, or a layer between 2 × 2 and 1 × 1 ("with a 1.5 layer between", only with the
  finest at 1 × 1); a layer's parent found by scale (0.75 for 1.5 under 2), the upscale taking a layer whose pixels
  don't line up with the screen's (crisp: one and two screen pixels wide by turns; smooth: a little soft). **The
  fades his to set** (the Render page, "Fades between layers", folded): among the blocks, where each finer layer
  fades out (2 × 2 and coarser, 1.5, 1 × 1 each its own from and to, shares of the blocks' reach; 75% to 100% unless
  chosen, as before); past them, where a finer layer starts fading in (its pixel so many times its cells, 1 unless
  chosen) and over how many doublings (1): the defaults draw what 81 drew. **How quick, on the page**: the frames and
  **the card's own clock on sight** (its timestamps, now asked for wherever the card has them; smoothed over about
  twenty frames; also in the soak's lines). **Coarsest 4 × 4 unless chosen** (his pick); `INCUS_SIGHT_COARSEST` for
  measuring.
  **A hole found on the way, fixed**: with finer layers everywhere, the 1.5 and 1 × 1 layers shaded their own sunlit
  ground (2 to 6% in the ridge test). Build 78's rule starts a finer ray a little short of its parent's half-covered
  distance; over cells far bigger than that margin (32s, everywhere) it began inside their soft edge, gathered too
  little, and its surface (and so its shadow ray) sat too deep. A layer now keeps where its ray *first touched*
  anything for the layer below to start from. Every combination now passes the ridge test, everywhere or where worth
  it. **Measured** (the low slow flight, uncapped, a fresh test world baking): coarsest 8 about 103 to 118 fps, sight
  on the card about 5.4 to 6.8 ms (as 80: the first-touch rule costs nothing to see); coarsest 4 (the default now)
  about 90 to 104 fps, sight about 7 to 8.3 ms.
  **David's look at build 82** [2026-11-08: "the fades are worth working with. not sure about 1.5x, leave it alone
  for the moment i guess. the speed readout is actually very useful"]. 1.5 stays as built, untouched till he asks;
  the fades are his to tune. Start bake 151 s that run (the coarser default's share of the card).
  Next: sky light (3.4b's second half), talked through first.
  **3.4b's second half BUILT, build 83: sky light** [talked through 2026-11-08; David on its colour: "tint slider is
  fine, i can't decide what color the light should be at this stage anyways"]. The light that isn't the sun's (the
  sky's part, `camera.sun.w` of a face's colour) is now scaled by how much of the sky a place sees: five wide cones up
  through the store (one straight up, four leaning 50° out to the compass points, each weighted by how high it looks),
  widening 0.6 a block, at most 40 steps each. The shadow ray is now the same machinery (`cone`: a direction, a
  spread, blocks walked first or not, what a face looking away gets). Ridges see it all, a valley floor less, the foot
  of a cliff about half, an overhang's underside none. **The Render page, under Sun**: "Sky light" (on unless chosen),
  its strength (how dark a place seeing no sky goes: 0.8 unless chosen) and **the sky's tint** (0 white, 1 a sky blue
  as bright as white, so a tinted shade is cooler, not darker; 0.3 unless chosen). `INCUS_SIGHT_NO_SKY` for measuring.
  **Handed down the pyramid**: each pixel passes its sky seen, and the kind of surface it was for (the cells, or a
  block's face by its facing), to the layer below in the same word as its worth (a shader may have only 8 storage
  buffers); a finer pixel takes it for the same kind about as far off, and looks for itself only at edges and
  silhouettes. **Test** (`sight_s_sky_light_darkens_the_foot_of_a_ridge`, card, ignored): the open plain unchanged
  (109.3 either way), the ridge's foot 54 from its face about 3.5% darker under a noon sun, the tint bluer and not
  darker; the same with finer layers everywhere at 1 × 1 (handed down through every layer).
  **Found on the way, fixed** (cost since build 80): each pixel kept a record of each of its up to four rays till it
  knew its worth, and that weighed on the card (about 2 to 3 ms once the sky's part was added). Each ray now judges its
  own worth and is shaded at once. **Measured** (the low slow flight, uncapped, defaults 4 × 4 to 2 × 2, a fresh test
  world baking): sky light off about 104 to 119 fps, sight on the card about 5.8 to 7.3 ms (build 82: 90 to 104, 7 to
  8); on about 75 to 102 fps, sight 7.2 to 12.2 ms (every pixel looking for itself: up to 15.6). What's left is mostly
  the blocks' side faces near the eye, unlike the tops round them.
  **For the speed dive**: sky light is the land's, not the sun's or the view's: kept in the cells (a byte each, worked
  out as they're baked and made, an edit patching its neighbourhood) it would cost almost nothing a frame.
  **David's look at build 83** [2026-11-08: "the sky light helps give depth to farther terrain"]. The start bake took
  281 s that run (46 before shadows, 92 with them, 151 with coarsest 4, now 281 with sky light): every richer frame
  leaves the card less room for the bake, so a new world shows the raw far view for minutes. Proposed next: the bake
  a sure share of the card each frame while it fills (a little frame rate then, nothing after), then fog (3.4c).
  [David's go, through Counsel: "that makes sense".]
  **Build 84: the bake's share of the card.** Why it starved: the card does its work in the order it's handed it, and
  the bake hands it small pieces, each waited for (so frames slot in between: 2026-11-06). Drawing as fast as it can,
  the next frame is always queued, so each piece waited behind a whole frame; the richer the frame, the longer. Now,
  while the bake's pieces are on the card (or were within the last tenth of a second), the frame loop paces itself so
  drawing takes at most (1 − share) of the card's time, by sight's own clock (never under 30 fps); the rest is free
  and the pieces run as they come. Only the bake counts (`runner::baking`, a mark on the baker's thread): the live
  levels round the eye are made all the time as it moves, and pacing for them would hold frames back in all play.
  The frame limit itself (and so vsync) is untouched. The Render page: the share, 0.5 unless chosen, and a line saying
  whether it's baking now. `INCUS_SIGHT_FILL_SHARE` for measuring (0 none). **Measured** (the low slow flight, 80 s, a
  fresh test world): regions baked 168 with no share (75 to 98 fps), 271 at 0.4 (54 to 73), **346 at 0.5 (45 to
  59)**, 536 at 0.7 (30): the card was the bake's bottleneck, the share trades frames for it about evenly.
  **Build 84 in play** (2026-11-08, a fresh world): the start's 729 regions in **108 s** (281 in build 83, 151 in
  82), all 5,184 in 424 s; after it, 192 fps, sight 2.5 ms (coarsest 8, finest 2 × 2, his 2 × 2 fade 0.33 to 0.52).
  **On file, to clean up before the speed dive** [David: "when you put your face right into a block, a circle appears
  as you get very close like this. sort of like the circles of light that swallow the world if i fly tens of miles
  over the surface straight up looking down. i think that may be the kind of thing that we clean up before
  performance."]: his screenshot (screenshot-1794165405) shows, close up against a cliff face, a disc round the
  crosshair cut into eight pie wedges of slightly different shades, the wall smooth outside it. Guessed, not looked
  into: the disc's edge a distance cut (the nearest part of a flat wall is the middle of the screen), the wedges the
  shading choosing a face by which way the ray points. The circles of light seen from tens of miles up, looking
  down, may be the same kind of thing.
  **His preset** (presets/render/test.toml, his, not committed) ["the fade seems to work well, i can feather the edges
  together cleanly"]: coarsest 8 × 8, finest 4 × 4 (two layers only), the blocks' fade 0.33 to 0.52 of their reach
  (default 0.75 to 1), past the blocks the finer layer in from 0.68 over 0.32 doublings (default 1 over 1): earlier
  and tighter. Sky light on, shadows off ["i turned them off for speed"]. [David: "i'd like to get at least 2x2 to
  8x8 working to where i can 'valve' them together. there doesn't seem to be any trouble with the feathering between
  scales like there was with geometry-space LOD filtering. crunchy 2x2 may actually be a good look, since it'll still
  be like a dozen pixels per texel pixel on screen."]
  **Build 85: the crossover** [David: "a track, with a slider for each intersection between layers, and bars that
  come from the side as i drag the slider for each intersection up and down, the band gets wider. sort of like a
  multiband compressor"; "oh yeah, a crossover! exactly"]. Each seam (by its finer layer: 4 × 4, 2 × 2, 1.5, 1 × 1;
  `draw::SEAMS`) has its own fades now, both kinds (till 84 the 4 × 4 and 2 × 2 seams shared one among the blocks, and
  every seam one past them; the old keys still read, so a preset keeps its look). On the Render page, "Fades between
  layers (the crossover)" (`look/crossover.rs`): two tracks, near among the blocks (by distance, shares of their
  reach) and past them (by a cell's size on the screen, in doublings); each layer a coloured zone, finest left, the
  bands mixed as the shader mixes them; each seam on its own lane, a handle dragged left or right moving it, up or
  down widening its band (bars at its edges), double-clicked back to its default; the handles never pass each
  other; seams not in use named, greyed. No shader change: the look moves only with the handles.
  **David's look at build 85** [2026-11-08: "it actually works is all i can say. i can see the seam between sharp
  blocks and coarse blocks and let them fade together. it looks natural."]. The start bake 115 s.
  **3.4c, fog: build 86** [talked through 2026-11-08; David: "that sounds alright. i have no particular feelings
  either way about fog right now"; 2026-10-28: fog out to about 2 miles]. The air between the eye and what it sees,
  thickest at sea level and halving every so many blocks up (valleys hazy, peaks clear), worked out whole along each
  ray (the exponential's closed form: no steps). Land fades to the sky's horizon colour, so far mountains melt into
  the sky; a little warmer looking towards the sun. Each ray fogged once, where it first meets anything (the sky
  behind nothing isn't fogged: it's the sky), so it runs through the layers and the crossover with no seams of its
  own. Defaults (`draw::FOG_*`): at sea level land 3,200 blocks off nineteen parts in twenty fog; halving every 200
  up; warmth 0.3. The Render page, under Sun: Fog, how far you see, how high the haze lies, warmth (sight's own
  keys, `sight_fog*`: the old renderer's `fog_*` untouched). `INCUS_SIGHT_NO_FOG` for measuring. A card test
  (`sight_s_fog_lies_low_and_far`, a thick low haze to measure it, in linear light): near land 0.15 of the way to the
  horizon's colour, 600 off 0.48, the same view 150 higher 0.18. **Measured** (the low slow flight, 80 s, fresh
  worlds, A/B twice): on and off the same, 46 to 52 fps, sight 10.5 to 12.9 ms (the bake's share pacing the
  frames throughout). **For the speed dive**: where the fog is total a ray could stop. **On file, not chased**: in
  the test scene a plateau asked for at 450 or 650 didn't stand that high (the view 600 to 1,000 off at a graze met
  the sky or something far beyond); 300 does. A test-scene matter as far as seen; real mountains draw above it.
  **David's look at build 86** [2026-11-08: "fog looked sick."]. The start bake 119 s. The mountains' contrast at a
  distance stays his to tune with the fog, by eye.
  **3.6, the first step: block textures, build 87** [talked through 2026-11-08; David: "so the blocks are mipmapped,
  right? go ahead"]. Where the near walk meets a block's face, the face wears its texture: each block's six faces'
  textures from its data file, grass tops turned at random (by the block's place, never changing). Up close a texel
  is a square as painted (no smoothing); shrinking, the texture steps down through its shrunk versions (32 to 1, made
  as the far colours are made, blended between so far faces don't shimmer), and where a block is a pixel it's one
  texel, the block's far colour, which the cells past it are made of: the hand-off is seamless by construction
  (`sight/textures.rs`; the card's stack read as light). The finest cells (single blocks) now know which block they
  are (the card's cell sums carry the solid blocks' numbers added, kept in a single block's cell where a bigger cell
  keeps its heights' spread). A texture changed on disk shows at once (the data watch bumps the registry's revision;
  sight gathers the textures again). The Render page, under Sight: Block textures, and their sharpness (a shift in
  the shrunk versions taken). `INCUS_SIGHT_NO_TEXTURES` for measuring. A card test (`sight_s_blocks_wear_their_textures`,
  the test scene now able to make the blocks round the eye): up close a red and blue checker shows (each a third of
  the pixels; none without textures); 40 off, a face is the checker shrunk to one texel, to four figures.
  **Measured** (the low slow flight, A/B twice): textures on and off alike, sight 11.1 to 14.2 ms against build 86's
  10.5 to 12.9 (the code being there costs about 0.6 to 1 ms). Read inside the walk, the texture first cost about
  5 ms on every ray (the walk held more at once on the card, every step heavier); now it's put on after the march,
  once a ray: the walk notes how much of the ray the face took, and the texture's difference from the flat colour is
  lit and added (its sun's and sky's parts with it). **For the speed dive**: the remaining ~1 ms.
  **For David's eye** (a rule from 2026-10-29, kept): the far colours, and so the shrunk versions, average a texture's
  stored values, not its light; a strongly contrasting texture shrinks a little darker than its light up close (the
  test's red and blue checker 28%, gabbro's speckle about 18% then). Averaging as light instead would change every
  far colour, and the bake with them. **On file**: a block's underside wears its bottom texture up close, while the
  cells count a bottom face as its sides' colour (seen only under overhangs).
  **David's look at build 87** [2026-11-08: "they look good."]. The stored-values averaging stays as it is; it's
  on file for him to raise if a contrasting texture ever shrinks too dark.
  **Water waits** [David 2026-11-08: "right now water is transparent blue. it looks fine, and until the graphics are
  done we won't be messing with water behaviors."]: 3.6's water as a second medium is set aside until the graphics
  are done. Next, edits shown in sight (build 88).
  **3.6, edits in the live levels, build 88** [David 2026-11-08: "yes, proceed"]. Before it only the bake (cells of
  32 up) showed an edit, 2 seconds after it was left alone; the live levels (1 to 16) were the recipe's, so a hole
  dug didn't show near at all. Now each change as it's announced is a spot (`sight/edits.rs`), and every live
  tile it reaches, at every level, is made again at once: on the card as before, then the cells the edits reach
  made again from the blocks (the changed chunks as they are, the rest the recipe's, made once on the processor
  and shared): an edited block changes its own cell and the cells of the six beside it, nothing else. A cell
  column's band reaches out to hold them (a pillar over the ground, a shaft under it). The old cells stay on the
  card till the new ones replace them (nothing blinks); a tile being made when an edit lands is made again after.
  Chunks changed before sight started (a saved world) have their differences from the recipe found on the loading
  lane. The bake's changed chunks are now shared, so an edit copies only its own chunk (it had copied every
  changed chunk at each patch). **Test** (`sight_s_live_levels_show_edits`): a hole 3 by 3 and 6 deep at a chunk's
  edge and a pillar of 3 on the ground; at all five levels every cell round them, patched and the card's beside
  them, exactly the edited blocks' sums (1,334 cells), the hole and the pillar showing where the recipe alone
  doesn't, a single block's cell showing a face knowing its block; columns the edits don't reach the card's.
  **Measured** (the test): patching took about 12 ms across the five levels, on the loading lane, never the frame
  (the cells of 16 the most, 7.9 ms; the neighbouring chunk's recipe made once). **On file**: the recipe's chunks
  made for edits are kept for the session (a few hundred KB each); scattered edits over a wide area would make
  many cells of 16 again (each 4,096 blocks): built up from the finer cells instead, if it ever shows.
  **David's look at build 88** [2026-11-08: "i was able to place whole blocks on the grid, and dig blocks,and see
  them. but there is no magenta highlight, and no quarter blocks or grid shifts, they appear as white cubes, in the
  case of a shifted block, a 1.5x1 white cube."]. Two holes the regroup left: the helpers were Bevy's gizmos, cut
  with its renderer; and sight takes a cell of pieces (the marker `BlockId::PIECES`) for one whole block with no
  colour or texture. Agreed [David: "yes"]: the helpers back first (89), then pieces (90): up close the near walk
  steps through a pieces cell's eight octants, each its own block's colour and texture; in cells of 2 and up
  (where a block is about a pixel) a pieces cell counts as a block of its most-used material when half or more is
  filled, air otherwise (counting eighths exactly would change the store's unit everywhere: not worth it now).
  **Build 89, the helpers back**: the targeted block outlined white, the face pointed at in magenta with a cross,
  a bar's or shifted block's room in magenta and the piece aimed at in white instead of the block, an object's
  facing arrow: lines drawn over the picture from `LookTarget` with the camera's own projection (`to_screen`, the
  same as sight's rays), the part of a line behind the eye cut off. The Camera page's switch and brightness work
  as before. Drawn over everything (gizmos hid behind nearer blocks): seen through walls, if it bothers him.
  **David's look at build 89** [2026-11-08: "they shouldn't go into blocks, they should show what they'll place;
  additionally, the quarter blocks and shifted blocks still just go white"]; asked which, he chose to box the empty
  place. **Build 90**: the helpers box what placing would take (magenta: the place in front of the face, or a bar's
  or shifted block's room), the block aimed at no longer boxed; the face's cross stays (what a dig would take), and
  in a cell of pieces the piece aimed at (white). **Pieces in sight**: the card keeps a table of every cell of
  pieces in the changed chunks, its eight octants' blocks (`CardStore::put_pieces`, 16,384 places, at most half
  used, looked up by a hash of the cell's place; written whole when an edit lands), and a cell of one block built
  of pieces keeps the marker as its block; the near walk, meeting one, steps through its octants (a ray crosses at
  most four) to the first filled one, its face wearing its block's texture where the octant lies on the block
  (half of it across), or with textures off the texture's one texel; passed through, the walk goes on. **Changed
  from what was agreed, and why**: in the sums a cell of pieces counts as a block of its most-used material
  whatever it holds, not air under half: the near walk skips the air over a coarser cell column's top, so a lone
  octant counted as air there would be skipped and never met. A face seen through a gap in a cell of pieces (the
  sums count the cell whole, so the face behind has none) was rock grey: it now wears its texture. **Tests**:
  `sight_s_pieces_show_their_octants` (a cell ahead of the eye as air, a whole block and a slab: through the lower
  half the slab is the block, [170, 93, 30] both, through the upper half the air) and
  `a_cell_of_pieces_counts_as_its_main_material`. Suite 396. Not measured: the walk reads one more word where a
  ray meets a block, and walks octants only in a cell of pieces. **On file**: a block with no texture (the tool
  blocks) shows its octants rock grey; shadows and sky light see a cell of pieces whole.
  **David's look at build 90** [2026-11-08: "quarterblocks are still greyed but place right. the helper lines are
  working right. shifting blocks works."]. The quarter blocks were stone brick, a bar block: its textures are a
  bar's own (`bar = { side, end }`, `assets/textures/bars/`), which sight's table didn't have, so it found none.
  **Build 91**: the bars' own textures in the table (its last word: the long side's layer and the end's), read as
  the old renderer read them (16 × 32 and 16 × 16, each pixel repeated to fill a layer); each octant in the card's
  table carries what it's part of (`Part`, the high byte; its block the low one, past 254 as 255); the near walk
  puts a bar's long side up its length and its end on its ends (a lone octant its end on every face), from the
  bar's own corner (the old `bar_uv`, ported), and lines a whole block on the octant grid's texture up with the
  block, not the cell (as the old renderer did). Tests: `a_bars_own_textures_go_in_the_table`, the pieces table's
  part byte, and the pieces picture test with gabbro given a blue bar end (the slab's face blue).
  **David's look at build 91** [2026-11-08: the bars show their brick textures: "yes"]. Edits in sight done enough
  to go on: whole blocks, bars, octants and shifted blocks show as placed, at every live level, with the helpers.
  **The mountains' contrast at a distance** (palette and shading, his to tune by eye: on file under 3.4, light
  through the cache). **Far mountains at 8 × 8** may not need more; forests will carry them later.
- **3.2's march must not draw ziggurats** [David 2026-11-06: "this wouldn't make ziggurats would it? four coarse
  pixels aren't going to be grey when three should be grey because each one is hitting a fourth of a chunk that is 51%
  stone at a 45 degree angle on a mountain side?"; requirements from Counsel, for 3.2's talk]: (1) **no threshold**: a
  cell's solid share is never rounded to solid or empty; partial cells give partial coverage, gathered front to back,
  so a slope's edge across coarse pixels is a gradient, not three on and one off; (2) **interpolated between cells**:
  the ray reads a smooth blend of neighbouring cells (trilinear: solid share, colour, facing), never the one box it's
  in, which would draw cell-sized stairs; (3) **shaded by the cell's average facing** (which the cell keeps), so a 45°
  slope is lit as a slope, not as cube faces. **Its test:** made-up slopes of 45° and 30° drawn at several distances,
  with the staircase steps along the edge and the shading measured. If interpolated coverage still looks lumpy at cell
  scale, the known next step is keeping where the surface sits inside the cell (a plane's offset), held till his eye
  asks for it. If a stage turns out wrong, its commit is undone and the
  stage redone; the stages before it stand.

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
- **The speed deep dive's list** (now kept as a checklist, ticked off as it goes: `docs/dyno.md`) (David 2026-11-08, its order and his ideas in `decisions.md`, "Look first, then the
  dyno": the look first, then inspection measured, then power one measured step at a time). His ideas there:
  temporal reuse with his rules, far pixels re-marched only once they've moved, warps kept alike, subgroups, solved
  jumps and horizon maps, tiles by type. Builder's review (sent to Counsel 2026-11-08) adds: rays ended where the fog
  is total (land at sea level 99% fog by about 5,000 blocks, against a reach of 150,000: likely the cheapest win);
  relaxed cone stepping (each cell column the widest empty cone over its top, so a ray grazing flat land or sea
  takes long safe steps: the case column-top skipping misses); an empty-space distance per cell (the same in 3D,
  for caves); a hierarchical start pass; f16 accumulators; the workgroup's shape and order; the sea as a plane;
  sky light in cells, or better from horizon maps; textures' last ~1 ms (build 87); Fork E's remnant, a
  checkerboarded coarse layer filled in time. Before the dive, two flaws to clean up [David: "i think that may be
  the kind of thing that we clean up before performance"]: the close-up pie-wedge disc, and the circles of light
  from tens of miles up (both fixed in build 93, below).
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
- **Begun 2026-11-09** [David: "lets begin with light."]. The steps, a build each, talked through with him: (1)
  darkness, so there's a night for lights to show in; (2) placed lights as `caves.md` §11 decided (flooded through
  the air on the processor, coloured, reach up to 15, the brighter of two per colour, stopped by walls, flickering in
  the shader; the card keeping the lit air cells, a face taking the light of the air in front of it); then objects
  drawn in sight (the brazier's model and flame: the march keeping a depth picture, models drawn into it); (3) the
  carried light (a 32-block cube round the player, flooded again each block moved). Later, on file: lights seen from
  far off. Objects wait without harm [David: "its ok, we can proceed without it, as long as it shouldn't be a
  problem when we need to"]: they're placed, saved and collided with already, and a light comes from its object's
  data, not its model.
  **Build 92, darkness**: the Render page's sun goes under the horizon (to 30° under; it stopped at 5° up), and a
  new slider, night light (0.04 unless chosen: what's left of the sky's light with the sun well down). The sun's
  own light is full from 4° up and gone 2° under (its disc behind the horizon), with no shadow rays once it's gone;
  the sky's light and the sky's own brightness fade from 4° up to 18° under (the dusk's first part bright, its last
  dim), down to night's light; the sky darkens to a blue night (its day colours times night's light, bluer), with a
  dusk glow low on the sun's side while the sun is near the horizon; fog fades to the same night sky, and water is
  lit like the land. Caves keep only the sky light they see, so at night they're black. The World Compiler's
  pictures keep their sun at 5° up or more. From 5° up the land looks as it did, but for a faint dusk glow on the
  horizon under a sun 5° to 10° up. Tests: `the_light_goes_with_the_sun` and `sight_s_night_is_dark` (the plain 74 by
  day, 8 at night, 0 with no night light; the sky 179 to 22). Suite 399.
  **David on 92** (2026-11-09): "the dusk was beautiful. it only got as dark as a moonlit night, not pitch black, but
  it looks really good." (Night light 0.04 is the moonlit night; the slider goes to 0 for black.) Darkness done
  enough to go on. Then: "our performance levels are fk'd right now ... i think the 'ok now make it fast' part is
  happening today."
- **The dyno's baseline** (2026-11-09, after build 92; David: "yes, please"). The low slow flight (`INCUS_SOAK=540`,
  continuous, uncapped, 30 above the ground, 20 blocks a second, pitch -20), a fresh world each run, so each run
  goes through the bake and past it; defaults, then David's `presets/render/test.toml` (8 × 8 and 4 × 4, tight
  fades, shadows off), started with a new measuring switch `INCUS_START_PRESET=<category>/<name>` (a page starts
  with that preset instead of its "default"; his files untouched). Every 10 s:

  | | while baking | baked |
  |---|---|---|
  | defaults: frames a second | 38 to 43 (1% low 34 to 41) | 56 to 75 (1% low 40 to 53) |
  | defaults: sight on the card | 11.5 to 14.4 ms | 12.1 to 16.6 ms |
  | his preset: frames a second | 80 to 108 (1% low 67 to 95) | 197 to 262 (1% low 84 to 104) |
  | his preset: sight on the card | 4.4 to 6.6 ms | 3.4 to 4.7 ms |
  | the bake: the start's 729 regions / all 5,184 | defaults 108 s / 434 s; his 91 s / 359 s | |

  What it says: (1) builds 88 to 92 cost nothing to see (sight while baking as at build 87, 11.1 to 14.2 ms);
  (2) while baking, the frame is about twice sight's time: the bake's share (build 84) takes about half the card,
  so a fresh world plays at about half speed for its first six or seven minutes, which is what David played 92 in;
  (3) baked, the defaults' march itself is about 12 ms (finest 2 × 2, coarsest 4 × 4, shadows and sky light on),
  at most about 80 frames a second however the rest is arranged; his preset's march is about a third of
  that; (4) the ground matters: the defaults' last 30 s (new ground on the circle) went to 16 ms. The flight sees
  only a fifth of its circle in 540 s, so both runs see the same ground and compare, but a later flight should
  take in more kinds of land (sea, mountains, a cave mouth). Scratchpad `base92.sh`; logs kept there.
- **The last leg before the dyno** [David via Counsel, 2026-11-09: "yes, builder can put that in her gps"]: the two
  flaws; placed lights with the storage-buffer decision (more slots from the card unless packing proves faster or
  cleaner: "probably getting more is better if its more performant or clean"); the objects' depth contract;
  generic transparency as a medium (water, later glass and ice: "if we get generic transparency done right this
  time"; animated or cycling tiles are textures on it, after), the sea plane beside it on the block list; then the
  dyno. Nsight once Builder is off the PC. Each a build and a look.
- **Build 93: the two flaws**, both found in the test scene first (pictures written out and looked at) and each
  now a card test that fails without its fix:
  - **The disc up close** (`sight_s_a_face_up_close_is_lit_evenly`): every ray's walk of the blocks began half a
    block out from the eye (the start pass's and a ray's least start), so a face nearer than that was met inside
    its block: facing the eye (the walk's guess for the block it starts in), its shadow and sky rays starting in
    the rock. The disc was where the wall is nearer than half a block; dark from the shadow ray, and cut into
    wedges by the sky's cones, each taking the made-up facing as towards or away from it. Now the walk starts at
    the eye when the start is that near. The eye 0.02 to 0.3 over a plain, straight down or 60° down: every pixel
    the plain's own colour from 2 up (48 off before).
  - **The circles of light from high up** (`sight_from_high_up_the_ground_stays_ground`): not light at all. A far
    cell is an average: a cell of 1,024 over 150 blocks of ground is about a seventh solid, so a ray looking down
    through it gathered a seventh of the ground and left through the world's floor, the rest the sky's colour; the
    circle was the distance where the cells read grew that coarse. Now a ray that goes down through the world's
    floor has met the ground (under it all is rock): what it gathered is the ground it went through, all of it.
    And where the cells are as tall as all the ground, a stride through air from over its top landed under the
    floor and the ray met nothing (holes of sky a coarse pixel big, 30,000 up): going down, a stride through air
    now stops at the floor. Hills and sea from 2,000 to 60,000 up, straight down and 50° down: no pixel over the
    map the sky's colour (5,416 of 20,880 from 30,000 before).
  - What may look different in play: far land and sea seen steeply from high up are solid now, not washed toward
    the sky's colour; the fog is what hazes the distance.
  - **Measured** (the baseline's low slow flight, its first 200 s, while baking, defaults): sight 12 to 18 ms,
    32 to 35 fps, against the baseline's 11.5 to 14.2 ms and 38 to 43. The floor rule is the cost: with it alone
    switched off, 10.3 to 14.9 ms and 37 to 40 fps, the baseline again. Rays that went through the floor partly
    gathered were drawn partly as sky, never half covered, so they took no shadow or sky light and the finer
    layers took them for sky; now they're ground, and get all of that. About 2 to 3 ms for ground drawn as ground:
    for the dyno to look into (which part it is: the shadows, the sky's cones, or the finer layers' extra pixels).
  - Suite 408, all passing.
- **David on 93** (2026-11-09, a fresh world, baking): "i think this may be an actual flaw. on the peaks at a
  distance, a band juts out from either side, as i transverse up and down the band slides up and down the tip with
  me." His screenshot: far peaks against the sky, each with a dark brim straight out both sides at the eye's height.
  93's own doing: its floor rule took all of what a ray going down through the floor hadn't gathered as what it
  had. Rays a little under the horizon brushed a peak's soft edge (a few parts in a hundred), went on over regions
  not baked yet (skipped whole) and down through the floor, and the brush became the whole pixel: a band at the
  eye's height, beside every peak, moving with the eye.
- **Build 94: the floor rule, only over ground.** A ray down through the floor is filled with the ground it
  gathered in the cells just over the floor (the last it gathered there, lit), and nothing else: down through the
  floor over nothing (past the map, a region not baked yet) it stays sky, as before 93. Looking down from high up
  the bottom cells are the ground, so the circles of light stay gone (that test still passes). Card test
  `sight_over_nothing_the_floor_stays_sky` (a low plain to the map's edge, the eye 170 over it: just under the
  horizon, away from the peak, every pixel within 5 of the sky over it; 176 off under 93's rule). Measured (the
  low flight's first 130 s): sight 11.6 to 16.8 ms, 33 to 50 fps, about as 93: the cost is far ground drawn as
  ground, not the brushed rays. Suite 409.
- **David on 94** (2026-11-09): "looks good. the circles are gone up close, and i don't even mind the circles i see
  from the sky now. its fine, its just the lens. the glitch on the peaks is gone." The two flaws done enough to go
  on. **On file**: from high up a faint circle still shows (the levels changing with distance, read as the lens);
  his word is that it's fine, so it stays unless he raises it.
- **Build 95: placed lights, and the brazier drawn** [David through Counsel, 2026-11-09: placed lights as planned;
  "yeah, lets put the object in at the same time"; the slots: "probably getting more is better if its more
  performant or clean"]. One build, one launch.
  - **The light** (`sight/lights.rs`, ported from the old renderer's `placed_light.rs`): each object with a
    `[light]` in its file (the brazier: green, reach 8, brightness 0.9) floods it from its sprite's spot through air
    on the processor, a step dimmer a block, round corners, never through solid blocks (unloaded chunks count as
    solid), 16 levels a colour, the brighter of two lights per colour. Flooded again when one's placed or taken,
    when blocks change within its reach, and all again when the chunks round them come or go (`keep_placed_lights`,
    `world/mod.rs`). On the card, a table of lit air cells (`CardStore::put_lights`, 65,536 places, binding 15),
    written whole when what's lit changes.
  - **On the faces:** the near walk, meeting a face, reads the air in front of it, blended between the four cells
    nearest across the face (as the old renderer kept light at each face's corners: it falls away smoothly, not a
    block at a time), eased (squared), flickering on each light's own beat (from where it stands). It's added on
    top of the sun's and the sky's light, by the face's colour and its texture's, and kept apart from them, so no
    shadow or sky openness dims it (a brazier lights a pitch-black cave). Shadow rays don't look lights up. The
    shape it lights is a diamond (a step a block, as the old renderer's): rounder is possible if wanted.
  - **The Render page**, under Sun: Placed lights (on), flicker (0.15), how fast it flickers (1.5): the old
    renderer's keys, so its presets keep their say. `INCUS_SIGHT_NO_LIGHTS` for measuring.
  - **The slots:** the march reads 9 storage buffers now; the card is asked for all it has
    (`whole::shader::big_buffers`), and the log says how many (David's card: 262,144). Under 9, the game stops at
    start saying so. No packing fallback: every desktop card wgpu runs on gives far more.
  - **The brazier drawn** (`sight/models.rs`, `models.wgsl`): each placed object's glTF (its meshes and embedded
    pictures, nearest-sampled, a texel see-through or solid), in the game's axes as its shape is read, scaled to its
    height, on its feet, turned its way; lit by the sky and the sun on its faces if nothing stands over its column
    (none under cover), and by the placed light where it stands (the brightest of its cells, flickering in step
    with the faces round it). Its sprite (the flame strip) on its spot, on its own frame, turning about the upright
    to face you, full bright. A model or strip changed on disk is read again.
  - **The depth contract:** the upscale writes a depth picture as it brings the layers up: each screen pixel's,
    from the finest layer shown there (more than half), where its rays first touched anything; the objects are drawn
    into it (reverse depth: the nearest drawn, 0.05, over the distance ahead). No new slot in the march: the layers'
    distance pictures were there already.
  - **Tests:** the flood (a step a block, its reach, a wall, round over a low wall, brighter of two, a wall built
    near a light); the brazier's glTF read (every mesh with its picture, 1.5 tall, its feet its lowest corner);
    data URIs; card tests: a green light at night lights the ground round it and nothing past its reach, nothing
    with lights off (`sight_s_placed_light_lights_the_ground_round_it`); the brazier on the plain shows, model and
    flame, and sunk under the ground it's hidden (`sight_draws_the_brazier_where_it_stands`).
  - **Measured** (the low flight, 130 s, A/B twice, no lights in the world): placed lights on 8.3 to 13.6 ms, off
    8.3 to 13.2: within the noise.
  - **The brazier's file:** David painted a missed face and saved it in Blockbench (`Art/3d braziere/braziere.bbmodel`,
    2026-11-03, its `texture.png` 2026-11-02), but `assets/models/braziere.gltf` is the export of 2026-11-01 with
    the old texture: the fix shows once he exports the glTF again over it (read again at once, in play).
  - **On file for transparency** [Counsel 2026-11-09]: the light's flood should pass through see-through blocks
    (glass, water), perhaps tinted.
  - **David on 95** (2026-11-09): placed several braziers (up to 7 lights, 2,123 lit cells), no problems in the
    log. Open for his word: the light's smooth fall-off against the clear steps decided before ("very clear steps
    in brightness", `decisions.md`, What light needs): a miss of mine, a one-line change or a switch.
- **Build 96: the crossover back to sliders** [David 2026-11-09: "i don't think the silly multiband visual blender
  thing i specced out was that good of an idea. i think it'd be better if we went back to a single strip for each
  successive stage of the pyramid enabled to the next with their own sliders, because as you see i can't figure
  out how to feather this transition here"]. "Fades between layers" on the Render page: a strip for each step of
  the pyramid ("8 × 8 into 4 × 4", ...), each with its own from and to among the blocks and its start and doublings
  past them; a step not in the pyramid greyed. The same keys as build 85's (nothing set is lost); `look/crossover.rs`
  gone. His screenshot (a hard curved edge between blocks and blurry cells, at night) waits on his look with the
  sliders: it may be a layer's own change from blocks to cells (at a block a pixel of that layer, `blocks_end` in
  `march.wgsl`), which no fade between layers reaches.
  **David on 96** (2026-11-09): "i was able to fix the seam with the new sliders." A fade between layers, then, and
  the strips done enough to go on.
- **Build 97: placed light in steps** [David 2026-11-09, on 95's blend: "the smooth light looks ok. but i just love
  the quantized per block lighting with the hard edges"; as `decisions.md` had it: "very clear steps in
  brightness"]. A face takes the light of the air cell in front of it whole: a block's step at a time, hard-edged.
  The blend across faces stays as a choice on the Render page ("blended across faces", off; `placed_light_smooth`).
  Models are lit from their cells as before. Suite 415.
  **David on 97** (2026-11-09): "the light looked dope." Placed light done enough to go on.
- **Generic transparency, talked through** (2026-11-09) [David: "im not really interested in reflections, and i
  think as long as transparency is done well and flexible, we'll be fine when i add stuff lke that. 16 fade through
  water is good, just blue water is fine for now"]. A block's data may give it a medium (a tint and a clarity: how
  many blocks you see through it); cells keep how much of them is a medium and which; a ray through a medium dims
  and tints with the length it goes (the near walk block by block, the cells by their shares), in place of water's
  fixed 85%; sunlight dims under it, and placed light floods through it, tinted. No reflections. Water first (about
  16 blocks to see through, blue); glass and ice take the same when they come. The sea as a plane stays on the
  dyno's list.
- **Build 98: generic transparency.**
  - **The data:** a block's `medium = { colour, clarity }` (`docs/data-files.md`); water's in `water.toml`
    (63, 111, 149: the blue sight drew it before; clarity 16). Water without one takes that by default.
  - **The cells:** any medium counts in a cell's see-through share (the byte that was water's); a single
    see-through block keeps its block number, as solid ones do (`Cell::with_block`), so the near walk knows which
    medium it is. Water the card's own cells make (the recipe's sea) keeps none, and is the world's water. No new
    bake (the cell's 16 bytes are as they were).
  - **The card:** each block's medium in a table (`BlockTextures::media_table`, binding 16, a uniform): its colour
    as light and how fast it dims (a twentieth left at its clarity); the first place the world's water's.
  - **The march:** the near walk, through a see-through block, dims and tints the ray by the length it goes
    through that block, by that block's medium; seen through no further, it stops. The far cells dim by their
    see-through share times the length, by water's medium. Both in place of water's fixed 85% at its surface: shallow
    water shows its bottom, deep water is its own blue. Shadow rays and the sky's cones are dimmed the same way, so
    it's darker the deeper under water. Placed light floods through media (untinted for now).
  - **On file:** the far cells hold one medium (water's): glass or ice far off would be tinted as water until a cell
    keeps which medium it is (a bake change); the far cells' averages put a little of the sea floor into deep water
    (deep water near 66, 113, 153 against 85, 118, 151 from 2,500 up, in the test); placed light tinted by what it
    goes through; animated and cycling water tiles, as textures on it.
  - **Test:** `sight_sees_through_water_as_deep_as_it_is` (3 deep shows the bottom, 27 deep the water's blue, near
    and from 2,500 up).
  - **Measured** (the low flight's first 130 s): sight 9.9 to 14.6 ms against build 95's 8.2 to 12.7: about 1.5 ms,
    rays going on through water to its bottom (or till it hides it) where they stopped mostly at its surface. For
    the dyno: the sea as a plane (`docs/dyno.md`). Suite 416.
  - **Not yet (95):** lights seen from far off (the list drawn on top); lights and objects beyond the loaded chunks
    (radius 4, 64 blocks: a brazier farther off is neither lit nor drawn); sun shadows on and from objects; fog on
    objects (they're near).
- **David on the engine, going into the dyno** (2026-11-09): "overall the new engine looks really nice, and if its
  loaded, its at least smoothish on my laptop right now, at dismal framerates. you can tell it's creaking when you
  move too quick though. i will say that the work we've done so that stuff happens predictably every duty cycle
  makes the lag and chop feel more like a problem that can be worked out through clever gpu origami than a doleful
  warning." The two things he feels: the bake (smooth once the ground is in, creaking when it has to catch up) and
  the frame rate itself. Even work each frame is what makes it read as solvable.
- **Lights seen from farther** [David 2026-11-09, after looking at 98: "i think the lights should be visible from
  farther; they snap off suddenly, when i think the glow and twinkle would still be able to get smaller for a while
  more"]. Today a brazier beyond the loaded chunks (radius 4, 64 blocks) is neither drawn nor lit, so its glow
  vanishes at once. Wanted: the glow and twinkle carry on past that edge, getting smaller with distance and
  fading into the fog, never snapping off. This was the "lights seen from far off" item under build 95's not yet.
  Queued after the flaw Builder is chasing.
  **Before the dyno** [David, the same day: "far light is important to be in there before we go onto the dyno"].
  What he remembers from the old engine and wants here: "braziers in a cave at night from a distance were just
  pixels of green that popped in and out as they twinkled, less and less as you got farther away. and i'd think
  some of the green light wash would squish down to a couple brighter greener coarse pixels for a moment after
  that in our new engine." So two parts:
  - **The glow:** a far brazier is a pixel or so of its light's colour, twinkling in and out, less often with
    distance, fading into the fog.
  - **The wash:** the light a brazier throws on the ground around it carries on past the loaded chunks, averaged
    down into the coarse cells: a couple of brighter, greener coarse pixels for a while past that.
- **David's look at 98:** "it looks good, but there's one thing that bothers me... the way the mountains make a
  brighter outline against the sky where their haze ends, is there a way we can tighten that up, especially if we're
  going to be seperating screen tiles into lean types we'll have to pick where the sky starts and mountain ends
  somewhere along that lining you see."
- **Build 99: no pale rim on far skylines** (a flaw, from 86's fog). A ray that brushed a far peak's soft edge and
  went on to the sky was fogged whole, to where it first touched: the sky it saw as well, turned to the horizon's pale
  colour, paler than the sky up there and than the hazed peak, so a pale rim ran round every far skyline. Now only
  what a ray met is fogged, as much of it as it met (the ray keeps how much went on to the sky); the sky behind stays
  the sky, as the sky behind nothing always was. The edge is now a plain blend from hazed peak to sky: as soft as the
  coarse cells it's read from, but with nothing paler than either side, so a tile sorted as sky or land has a clean
  line to fall on.
  - **Test:** `sight_s_far_ridge_has_no_bright_rim` (a ridge 330 high, 1,000 off, hazed; down three columns, no pixel
    brighter than both the sky 8 above and the ridge 8 below): 0.050 brighter before (light, 0 to 1), 0.007 after.
  - **On file:** a far peak hazed most of the way is paler than the sky behind it, since the fog's colour is the
    horizon's at every height and the sky is bluer higher up. That's 86's look ("far mountains melt into the sky"),
    and David's to change if he wants peaks to melt into the sky behind them instead. Suite 417.
- **David's look at 99:** "looking good, looking good. your far light plan sounds good as well, proceed." (The plan:
  the glow first, then the wash, a build each.)
- **Build 100: far lights' glows** (the first half of "Lights seen from farther").
  - **Which:** every light from 48 blocks off, in the loaded chunks and in the changed chunks put away (every
    object placed far off is kept in memory with its chunk, so all of them are known), nearest first, 1,024 at most.
  - **How it looks:** a square of its light's colour, added to the picture: as wide as its flame (0.75 blocks) in
    whole pixels, 1 to 4, so it doesn't shimmer; under a pixel, one pixel dimmer by as much as it's smaller. It fades
    in from 48 to 64 blocks, where the flame is still drawn, so it never snaps. It twinkles: shown only while its
    flicker (the same beat as its flame's) is over a bar that rises with distance, a third of its swing at 64 to nearly
    its top 1,500 further, so far off it shows only at its peaks; steady with flicker off. Through the fog as the land
    is (`draw::fog_left`, the march's fog worked out the same on the processor).
  - **What hides it:** a line walked on the card from the eye to it through the cells (`march.wgsl`'s `glow_pass`),
    each cell as it's kept, as wide as the cells read at that distance; only a mostly solid one stops it. Not the
    picture's depth (far off, where a ray first touched the soft edge of the ground's cells, short of a flame on the
    ground), nor a ray marched as the picture's are (their smooth blend leaks a faint haze a cell over the ground, and
    a line skimming open ground adds it up to covered). A light in the open (a sky cone straight up) is aimed a cell
    over its flame, out of the ground's cell; one under rock at its flame. So a hill or a cave's rock hides it, open
    ground doesn't.
  - **Test:** `sight_shows_far_lights_as_glows` (one pixel 500 off in the open, none behind a ridge, one from 30 up
    over the plain, its line skimming the ground's soft cells for 200 blocks); `a_far_glow_twinkles_less_the_farther_it_is`.
  - **Cost:** one line a light, a few hundred cells each, and one small draw: nothing measurable with a handful of
    braziers. Suite 419.
  - **On file:** the wash (its light on the ground averaged into the coarse cells) is the next build; a glow under
    water isn't tinted by it.
- **David's look at 100:** "looks great, builder."
- **Build 101: the wash** (the second half of "Lights seen from farther").
  - **Every light floods,** loaded or put away (all the objects placed far off). What's solid: a chunk's blocks,
    loaded or put away; where no chunk is in memory, the map's ground (`CompiledMap::column`, kept by column from
    frame to frame), so a light by the loaded chunks' edge no longer stops dead at it (unloaded chunks counted solid
    before) and a far one floods over its own ground. Caves and what's built in a chunk never changed aren't there
    till it loads.
  - **Averaged into the coarse cells** (`lights::wash`): the lit cells of air with ground under them, each light eased
    (squared, as shown), summed over each cell of 2 to 128 and shared out over its area across, then square-rooted
    back into a block's 16 steps, with the beat of its brightest light. A patch of a light reaching 15 is about a
    cell of 16 at 8 of 15, a cell of 128 at 1 or 2. Kept for the cell over the ground and the one under it (whichever
    the march meets the ground in), and at blocks' size the block under lit ground, for the far march's reads of blocks.
  - **On the card:** the same table as the blocks' light, the cell's level in the key and its light (bits 20 to 24).
  - **In the march:** where the far march gathers ground, the light kept for that cell at that level, by the ground's
    own colour, added after the sun's and sky's (no shadow takes it), whole cells at a time: hard-edged coarse pixels,
    as the blocks' light is stepped. It flickers on its brightest light's beat.
  - **Test:** `sight_s_wash_shows_far_off` (a light in the middle of a cell of 16, at night, from 600 and 2,000 up:
    one green coarse pixel, green 35 against 10, the ground away from it untouched; a light on a corner of four cells
    shares its patch four ways, about half as bright in each); `the_wash_shares_lit_ground_out_over_coarse_cells`.
  - **Cost:** a lookup in the light table each time the far march gathers ground, while placed lights are on. Not
    measured on the flight yet (its world has no lights: the lookup finds an empty place first); the dyno's
    instruments will show it. The table: about 240 cells of wash a light besides its lit air. Suite 421.
  - **On file:** the wash is the ground's: walls and cliffs lit from the side get the cell's share, as the ground
    does; light through water isn't tinted by it; the flood on a chunk change floods every light again (now the far
    ones too: the dyno's "the lights' flood on chunk changes").

- **The dyno begins** (2026-11-09). The plan, the save point (tag `before-dyno`, the USB backup), David's mark
  ("a stable 237fps with current settings", in tiers) and the instruments are in `docs/dyno.md`. The test world,
  `worlds/nice25 test` [David: "can you render nice25 as our test map, so when we add features for experiments and
  so forth its persistent?"], is nice25 saved with its whole bake; `INCUS_LOAD_WORLD` starts the game in a saved
  world, its bake read back (the first read-back of a whole bake ran the card out of memory: 5,184 regions put at
  once; now 32 a frame). **The baseline at build 101:** baked, 5.5 ms median (180 fps), 16.3 at the 95th, 21.3 at the
  99th; baking, 10.0 ms (100 fps), 29 and 33. Leg by leg, and what it means, in `docs/dyno.md`.

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

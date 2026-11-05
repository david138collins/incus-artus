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
| Bevy (window, input, app, schedule, renderer, assets) | | **Cut.** winit (window, input), wgpu (the card), egui, the `gltf` crate (models, already used), rayon (workers), and our own small scheduler. Audio (kira) and gamepads (gilrs) when they're needed. |

## The steps, in order

Each step ends with something David can see or Builder can measure, and nothing goes on until it's checked.

**Step 0: cut.** Build 60 stays at the tag `before-regroup`. Remove `render/` and Bevy.
- *Builder:* 1 to 2 sessions (about 4 to 8 hours): a new small core (window, input, the three lanes, the clock),
  the panel on egui-wgpu, every system rewired as a plain call on its lane. The simulation gets its own lane,
  running the toy rules as they are (no column store or scheduler yet). The 395 tests pass.
- *David:* 1 look: the editor, World Gen suite and save/load all work, on a plain sky-coloured window.
- *Measured:* 1 test window: the new floor, at the screen (PresentMon), uncapped and at 120. Expected near the
  no-Bevy floor measured today (0.02 to 0.6 ms of processor a frame). **Every slow frame is logged with what
  ran in it** (each lane's steps, timed), so the floor is shown free of the stutter, and anything that brings
  it back later is named the day it arrives. This is the stage he puts things on.

**Step 1: the whole-number recipe.** The 12 to 15 calls that run in play converted to fixed point, so the
processor and the card give the same bits (`decisions.md`, projected code is GPU-safe).
- *Builder:* 1 session. *David:* 1 look: the same spot before and after (Render World and Spawn at the exact
  same place). The world moves very slightly; a saved world's river may disagree with its banks by a block.

**Step 2: the first band: the world as blocks, made on the card.** The recipe on the card, with a twin test
against the processor's copy. The card makes the land in cells sized to the pixel (finest near, coarser with
distance, so the size changes are invisible and never tuned), out to where a block is smaller than a pixel
(about 1,500 to 2,000 blocks on David's screen), and marches it. Real chunks only near the player, for edits.
- *Builder:* 2 to 3 sessions. *David:* 2 looks (the land near, then far). 1 test window: its price.

**Step 3: the second band: the backdrop.** Past the first band, the exotic far view David believes in: one
that pixelates naturally (`vision/proposal-voxel-renderer.md`, the dome, and others to try). Important things
show from their first pixel through a list drawn on top. One transition to tune, far away.
- *Builder:* 1 to 2 sessions. *David:* 2 looks (does it read, is it beautiful). 1 test window.

**Step 4: light, coarse by nature, on the card.** Sky light and placed light worked out on the card in coarse
cells, the way the far land is.
- *Builder:* 1 to 2 sessions. *David:* 1 to 2 looks. 1 test window.

**Step 5: everything else, each with its price.** Water and glass, pieces and bars, models (drawn as meshes
into the same picture: the hybrid), people and figures (drawn, one instanced pass), and the 800-figure
stand-in battle if David puts it in the bar.
- *Builder and David:* sized when we get there.

Then **the bar** (`decisions.md`): runs smoothly and cool on his own flight, the land reads as one land, built
things hold up at a distance, heavy content doesn't change the answer. After that: the blueprint factory, then
the simulation, which starts with **its column store and scheduler** ("tables that don't know what they hold";
rules that name what they read and write, run side by side when they don't clash; `simulation.md`, rules as
algebra), the toy rules rewritten on it as the first test. *Builder:* 1 to 2 sessions; *David:* 1 look.

## Risks, and what David would feel

- **Step 0 touches 75 files.** The tests and the soak test check most of it; what they can't check is feel:
  mouse look, keys, the panel's grip on the mouse, the clock's pause and speed. That's his first look.
- **The new scheduler** is a known pattern (what each piece reads and writes decides what runs side by side),
  tested with deliberate clashes before the simulation relies on it.
- **The first band's price** is the big unknown: the card measured 3.7 ms for today's whole scene with no
  tricks; the first band has to come in well under that to leave room for light and everything else. If it
  doesn't, the coarse pass that starts each ray close to what it'll hit comes before step 3.
- **Nothing drawn for a while.** From step 0 to step 2 the world isn't on screen; the World Gen suite still
  shows it from above.

## Open, for David

- The audio, gamepads and the real game's menus aren't in any step yet. When does he want them?
- Step 0 starts on his go, after he's read this.

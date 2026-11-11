# The Master Plan

Where we've decided to go, in order, and where we are on the way. A living document: Counsel keeps it with David,
Builder ticks a line with the commit or log that proved it, and ticked lines stay. A change of order is David's
call, made here and recorded in the journal (`docs/decisions.md`, "The rebuild: after the Oracle").

Started 2026-11-11 from the Oracle's review (`vision/oracle-review.md`, parts 7 and 8), as David amended it.

---

## Where we are

- **Working on:** Phase 1, *Learn, in old*. Experiments 1, 2, 3, 4a and 4b done (2026-11-11, `docs/experiments.md`):
  soft bricks (the blend stays); light kept a cell at a time, which David can't tell from rays ("in a blind test"),
  though at eye level light's cost is the near faces'. The references from old are taken (1.6), and the test world
  is being exported whole for the new eye to read. Then old is frozen (`old-final`), and the restart begins (2.1).
- **Why:** we can't size the rebuild on guesses. The last outside guesses were off by ten times both ways, so a
  handful of cheap experiments in old settle what paper can't before anything new is written.
- **Waiting on:** the export to finish and be checked (Builder); then David's OK for the tag `old-final`.
- **The Oracle comes back** after experiment 6 (step 2.3), with the flight logs and David's look. Counsel gathers
  what we learn for it as we go, in `vision/oracle-return.md`.

---

## In one picture

```mermaid
flowchart LR
  P0["0 · Decide the direction"]:::ticked --> P1["1 · Learn, in old"]:::now
  P1 --> P2["2 · Restart: the eye on old's world"]
  P2 --> F{"The fork<br/>(the Oracle back)"}
  F --> P2b["2.4 · The eye, whole"]
  P2b --> P3["3 · The world"]
  P3 --> P4["4 · The game"]
  classDef ticked fill:#cfe8cf,stroke:#4a7a4a,color:#1d331d
  classDef now fill:#ffe9a8,stroke:#a8841c,color:#3d2f05
```

---

## Phase 0 · Decide the direction

- [x] Builder's review of the software as it stands (`docs/review-2026-11-10.md`)
- [x] The Oracle's review, after David answered its questions (`vision/oracle-review.md`)
- [x] Rebuild from scratch, old Incus Artus as the reference; experiments first, in old
      (`docs/decisions.md`, "The rebuild: after the Oracle", de53bb7)
- [x] VS Code set up for Builder (1ab561e)
- [ ] The spec cut by David (`master/spec.md`), alongside everything below

## Phase 1 · Learn, in old  ← *now*

Each experiment's pass mark is written down before it flies.

- [x] **1.1 The safety copy** (the Oracle's step 0)
  - [x] David's work that wasn't in git: blocks, textures, models, palettes, presets (1e3c21c)
  - [x] `vision/`, `CLAUDE.md`, the dyno-audit skill (0216222)
  - [x] The logs the record cites, in `kept logs/`; `notes/` and `imgui.ini` kept out of git (a203cdd)
  - [x] Pushed to the private GitHub repository
  - [x] USB copy: `D:\Incus Artus Backups\Incus Artus 2026-11-11 before rebuild`
  - [x] Tagged `before-rebuild` (a203cdd)
  - [x] `Art/`'s new files and `For Claude/` kept off GitHub for now (David 2026-11-11: "Not yet"); both are
        on the USB copy
- [x] **1.2 Experiment 1, the instruments** (both flights passed: `docs/experiments.md`)
  - [x] Flight headers say exactly what flew: every switch, the commit and whether it was clean, the card, the
        present mode (build 110, 090904f; flight A, `kept logs/dyno-1794414666.txt`)
  - [x] The card's time counts the work of making new ground (the lanes' clock; agrees with the build 104 trace:
        `docs/experiments.md`, flight A)
  - [x] Three standing marks, uncapped: on the ground at the spawn, beside the quarter-block build, a mountain
        top on a clear day (flight A: 5.9, 11.7 and 6.6 ms on the card standing)
  - [x] The choice of card restored; the marks flown once on the Intel card (restored in 090904f; flight B,
        `kept logs/dyno-1794416695.txt`: it ran, no failure but speed; standing 22.7, 20.6 and 29.5 ms)
  - *Gate:* David watches two flights; real baselines for the views that matter; the Intel card runs, or fails
    for a reason we can name
- [x] **1.3 Experiments 3 and 4a:** the library's safety checks off; light switched off (build 111; flights C, D,
      E in `kept logs/`; `docs/experiments.md`)
  - *Gate:* two numbers: what the checks cost, and how much of each mark is light
  - The checks: unchecked, sight's march is 31 to 49% *slower* at the three marks (faster only on the light views);
    the world's rules about 15% faster a piece. The checks stay on.
  - Light (sky and shadows): 42% of the spawn mark, 17% of the mountain top and of the build, 25% of the route's
    median.
- [x] **1.4 Experiment 2, one-cell far reads against today's blend** (builds 112 and 113; David's two looks,
      `docs/experiments.md`)
  - *Gate:* David's look. Decides the bricks' form (crisp: plain, 2.6 × the cells; soft: textures, 5.5 ×)
  - **Soft.** Blended is the look: crisp "doesn't look better", the in-between (smooth in height) "better than
    ziggurat lods but, its not better than blended" (David). Crisp stays a known fallback for a card short of memory.
- [x] **1.5 Experiment 4b, light stored in the baked cells and looked up** (builds 114 and 115; flights C2 and H,
      the light's view; `docs/experiments.md`)
  - *Gate:* what a lookup costs; David's look at light that steps
  - The lookup is almost free and found nearly everywhere far: the spawn 9% faster, the mountain top and the build 3%
    (at eye level light's cost is the near faces'). David on light that steps: "i don't think i could tell the
    difference between the two in a blind test".
- [ ] **1.6 References from old, then old frozen**
  - [x] Pictures at fixed places, times of day and settings (24: six places, four times of day; `references/pictures/`,
        `master/references.md`)
  - [x] The marks' flights, with the repaired instruments (C and C2, the Intel card's B; `master/references.md`)
  - [x] The default preset's maps and river network; sample columns (the test world's, nice25 with the dyno's
        presets, not the startup default: `references/world/`, `references/columns.txt`, 5,047 columns)
  - [ ] Tagged `old-final`
  - [ ] The test world exported whole as columns, for the new eye to read, so it never runs old's world gen. David:
        "export nice25 so that we can use it as a 'generated' world to look at with the new graphics engine, before
        rebuilding world gen after the graphics engine is done." (`world/export.rs`, a7d3cfb; kept in `exports/`
        and on the USB, not in git)
  - The pictures are kept for reference, not as a yardstick (David: "it'll look good or it won't, idc what the old
    things look like in particular, other than the basic way we're doing things")
  - *Gate:* the references listed here, each with where it's kept

## Phase 2 · Restart: the eye on old's world

- [ ] **2.1 Pack, and the skeleton**
  - [ ] Old moves to `old\`, with its own assets and its own build folder, and still runs
  - [ ] `docs/` and `vision/` become `journal/`, with a contents page; `master/` on top; `CLAUDE.md` points at
        `master/`; memories trimmed to habits and warnings
  - [ ] The new project: a window that runs, the clock, a flying camera, the instruments
  - [ ] Every stage, pass, lane and store declared as data; it wires the game and draws `master/map.md`
  - [ ] The control room: the same map, live, in the game's panel
  - *Gate:* an empty frame measured; the map drawn from the code
- [ ] **2.2 The eye, far** (experiment 5): far cells kept in bricks, made from the exported test world (old's far
      bake checks their sums)
  - *Gate:* the far layers under 1 ms typical and 2 worst on the laptop; David's look; numbers from the Intel card.
    If they're twice that or more: keep the bricks and the light, march far the lean way instead
- [ ] **2.3 The eye, near** (experiment 6): near blocks from the exported test world, by rays, or by triangles if
      rays skim
  - *Gate:* the ground mark under 1.5 ms on the laptop; David's look
- [ ] **The fork. The Oracle comes back:** flight logs from both cards and David's look; it reads `master/`
      fresh, then Counsel's brief, `vision/oracle-return.md`. The design is fixed here, or changed
- [ ] **2.4 The eye, whole:** the light lane, water and glass, quarter blocks, textures, objects, glows, far lights
  - *Gate:* David's eye, on its own terms, the basic way kept; under 4.2 ms at the 99th percentile on the laptop,
    on the flight and at all three marks; the Intel card's numbers recorded. **David walks old's world through
    the new eye**

## Phase 3 · The world

- [ ] **3.1 The tablet** (experiment 8 sizes it first): the made world on disk; one doorway for "what's here
      now"; walking, digging, placing, saving. Filled first from old's world
  - *Gate:* the same blocks as old's; dig, place, save, load; under 4 GB a world
- [ ] **3.2 The Registrar,** with its Color Lab and the way we work with HSL
- [ ] **3.3 The Graphics Kit,** built on the Registrar's colour system
- [ ] **3.4 World generation, stage by stage, with the new editor** (experiment 7 opens it)
  - [ ] Plans: tectonics, climate, rivers and lakes, peaks, cave plans
  - [ ] Broad ground, then fine ground, on the card
  - [ ] Water: sea level, cliffs and beaches, rivers. A design step of its own with David
  - [ ] Surface, then blocks (caves; later ores and strata)
  - [ ] Sums and light; the tablet written
  - *Gate, each stage:* David's eye, old beside new, on his default preset; the same result on both of the
    laptop's cards
- [ ] **3.5 Making a world as the player meets it:** the whole bake before play, the show while it's made, the
      rejector if wanted
  - *Gate:* a new world in the time David sets; no frame drops after entering

## Phase 4 · The game

- [ ] **4.1 The game's systems,** in the order `master/spec.md` gives, once David has cut it

---

## Open, and the step each one waits on

| Question | Waits on |
|---|---|
| `Art/`'s new files and `For Claude/` online? "Not yet" for now | David, any time |
| ~~Far cells crisp or soft (one cell read, or eight blended)?~~ | Answered at 1.4: soft |
| The world's new format, cut so generation can grow: "that lets me expand and refine the generation process" (David, `docs/decisions.md`). Where the eye gets its world is decided: the exported test world (1.6) | The Oracle, at the fork (her brief's question 7) |
| The control room as a page in the game's own panel? | 2.1 |
| Old's new place: `old\` inside this folder (the default), or elsewhere? | 2.1 |
| The moons: does the brighter one cast shadows? | 2.4 |
| People far off: sharper than the squares behind them, or drawn into them? | 2.4 |
| Leaves far off: a little see-through, as the averages make them, or solid? | 2.4 |
| "1 always equals 1" as one stored answer everything reads, in David's words | 3.1 |
| Rivers and the shore: carried over first, or rearranged on the way? | 3.4 |
| The rejector: what makes one world better than another? | 3.5 |

## The targets

- **The laptop** (RTX 5070 Ti, 2,560 × 1,600, 240 Hz): a median of 237 fps, dips to about 180 fine, 120 the floor.
  Its budget: 4.2 ms a frame.
- **A GTX 980 Ti at 1080p:** a smooth 60, loosely. "If you have one, you should be able to play this game."
  Not a floor to design to.
- Room left in the frame for people, animals, battles of up to 800 figures, and the simulation on the processor.
- **Today's gap** (experiment 1, build 110, `docs/experiments.md`): on the laptop, standing, the card takes 1.4
  times the budget at the spawn, 1.6 times on the mountain top and 2.8 times beside the quarter-block build; on
  the route, the worst frames (99th percentile, new ground counted) take 13.9 ms, 3.3 times. The Intel card is 2.5
  to 3.8 times slower than the NVIDIA, standing.
- **With light free** (experiment 4a): the spawn fits the budget (3.46 ms), but the mountain top (5.45) and the
  build (9.76) don't. There the rest is the far reach and the near walk through quarter blocks, so storing light
  alone won't close the gap: the near field (2.3) and the far reads (2.2) have to win too.
- **Light kept, far only** (experiment 4b): 9% off the spawn, 3% off the mountain top and the build. At eye level most
  of light's cost is in the near faces, lit afresh every frame. The new eye looks their light up too (2.3).
- **The blend** (flights F and G): the far reads' blend in code is a quarter to a third of every mark. That's the
  most the soft bricks' texture hardware can win back (2.2).

## What every step keeps

- **The look:** hard textured blocks near; past a pixel, cells that are the exact sums of their blocks, in squares
  of 2, 4 and 8, enlarged crisp; edges that average and glide; blocky light in 16 steps; very dark nights; far
  lights that twinkle down to a pixel; the whole world in view on a clear day.
- **Honest:** near and far agree before any blending; everything that exists shows, however small; nothing pops;
  far changes may arrive late but are accurate; nothing is a stand-in.
- **The way we work:** even beats fast; nothing relies on fog; no tricks for one maker's cards; measure from the
  start; the pass mark before the flight; no number without its log; old is the reference (its columns check the world to the block; the look is David's eye); one thing asked of David
  at a time; a drop is for its context, not for good; what's true lives in `master/`, and the agents' memory
  holds only habits and warnings.

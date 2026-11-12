# A falling-sand sim on the card: a reading  [Counsel, 2026-11-12]

David asked for a look at https://github.com/NivMiz0/GPU-Sand-Sim-Unity, and for anything useful to the life gauges
(`journal/docs/vision.md`, "Life gauges") and the FX Lab (`journal/vision/toy-factory.md`, "Pixel sims") to be kept
here. Nothing here is decided.

**The project:** a small falling-sand toy in Unity 6, made for a YouTube video (https://www.youtube.com/watch?v=HrrJxkRlRfk,
not watched for this reading). It's two files: one compute shader (`SimulateParticles.compute`) and the script that
drives it (`RenderSimulation.cs`). It has sand only: no liquids, gases or reactions.

**No licence:** the repository has none, so its code is all rights reserved. We read it for ideas and copy none of
it, which is the rebuild's way anyway (results, not code).

---

## How it works

1. **The grid is a picture.** One pixel per grain, the pixel's colour *is* the grain (16-bit floats, 8 bytes a cell).
   An empty cell is the background colour, and "is this sand?" is a colour comparison within 0.05.
2. **Two copies.** Each step reads last step's grid and writes the next. After each step the whole grid is copied
   back into the input and out to the screen (two full copies a step).
3. **One thread per cell** (groups of 8 × 8). A grain looks straight down, then one diagonal, then the other. Which
   diagonal it tries first swaps every step, so piles don't lean one way.
4. **Claims settle who moves where.** Two grains can want the same empty cell. Beside the grid is a "claims" list,
   one number per cell. A grain claims its target with an atomic compare-and-swap; the first wins, and the loser
   tries its next choice or stays put. The claims are cleared every step by uploading a zeroed list from the processor.
5. **Two steps a frame,** with straight-down falls only on every other step (gravity slowed); diagonal slides happen on both.
6. **Poking it:** a circular brush at the mouse (left paints, right erases, scroll sizes it); Space pauses; the right
   arrow steps one frame.
7. **Colour at birth:** new sand is either a random blend of two colours, grain by grain, or taken from a gradient
   that cycles every 2,000 steps, so a steady pour lays down bands.

## Worth taking

- **The size fits the gauges.** A sphere is a 64-pixel circle in the 640×480 UI layer: about 3,200 cells. Three
  spheres are under 10,000 cells, one small job for the card each step. That fits "everything drawn is the card's"
  (`journal/docs/decisions.md`, 2026-11-04).
- **Poke, pause, step.** The brush, the pause and the single step are the FX Lab's "Poke it" section. A step button
  belongs in the lab from its first day: effects are tuned a step at a time.
- **Colour chosen at birth and carried.** Each pixel picks its shade when it's made and keeps it as it moves. The
  pool gets grain and texture without shimmering. For the gauges: a shade from the element's palette ramp (the HUD is
  palette-quantised; the world isn't), stored with the pixel.
- **Pour bands.** The time-cycled gradient is how a poured potion could show as bands of colour entering the pool.
- **Read last step, write the next.** No pixel moves twice in a step, and the picture never shows half a step.

## What we'd do differently

- **Store what a cell is, not its colour.** An element number and a shade (a byte each), plus a spare byte for a
  timer or flags: 2 to 4 bytes a cell, not 8, and no colour comparisons. The elements in data files, like blocks, as
  the FX Lab proposes.
- **A fixed tick.** It steps twice per drawn frame, so it runs faster on a faster machine. Ours draws at 240 and more,
  uncapped: the sim steps on its own clock (its own lane on the software map, or a pass that runs the steps owed).
- **Swap, don't copy.** Alternate which copy is read and which is written, instead of copying the whole grid twice a step.
- **The same answer on every card.** With claims, which grain wins a contested cell depends on the card's thread
  timing, so two runs differ, and could differ by card. Its random numbers come from a `sin` trick that can differ
  between cards too. "1 always equals 1" and "no per-card tricks" ask for two changes:
  - *the Margolus neighbourhood* (from general knowledge, not this repository): the grid cut into 2 × 2 blocks, the
    blocks shifted by one cell every other step. Each block rearranges only its own four cells, so no two threads
    touch the same cell, nothing needs claiming or clearing, and a pixel can't be made or lost by the movement itself;
  - random numbers from a whole-number hash of the cell, the step and a seed.
  If claims are kept for something, stamp each claim with the step's number, so last step's claims go stale by
  themselves and nothing is uploaded each step.

## For the life gauges

- **The gauge is the truth; the pixels show it.** "Each fluid pixel is a fraction of that gauge", so the count of
  red pixels must always equal the blood. The game's number lives with the simulation; each tick it tells the card
  what to add or remove and where (the wound's side, the top for regeneration, the mouth for a potion). The card never
  decides the value. A test can hold it to that, as the doorway is held to old's columns: after any run of events,
  pixels counted = the gauge.
- **Liquids need a sideways rule.** After down and the diagonals, a liquid pixel also slides sideways up to a set
  number of cells (its spread). That's what makes blood pool and level instead of piling like sand.
- **The slosh is the open question.** Falling-sand pixels have no momentum, so a pool shaken by position rules alone
  looks stiff, and the slosh is the gauges' main eye candy ("people should stand there shaking the mouse to watch it
  slosh"). Two ways, which can go together:
  - tilt the pool's "down" with the mouse's turn, so the whole pool leans and runs back;
  - a splash: pixels knocked loose become free particles with a speed, fly, land, and rejoin the grid. Noita does
    this (Petri Purho's GDC 2019 talk on Noita's tech, from memory; worth watching before building).
- **The glass is a mask.** Cells outside the circle are walls. A crack opens wall cells at the bottom, and the sim's
  area reaches below the sphere, so blood runs down the HUD in rivulets.
- **Drops on the glass.** Regeneration's drops are born at the top edge, held to the glass for a few ticks (the
  spare byte's timer), then let go to run down into the pool.
- **Reactions** are rules for touching pixels (the FX Lab's "potion + blood → glowing → more blood"; lightning
  turns a pixel into fire and smoke). Inside 2 × 2 blocks, a block checks its own pairs.
- **Energy sublimating:** a blue pixel becomes a gas that rises and fades on its timer, and is gone in the same tick
  the game's energy drops by that pixel.

## Beyond the gauges

- The FX Lab's pixel sims in general: smoke, sparks, fire and dust on 2D layers, for spell icons, the HUD and menus.
- The "world gen toy versions to watch" while a world is made (`journal/docs/decisions.md`, "The world made once,
  like a stone tablet") could use the same engine.
- Any rule run cell by cell on the card later (fire spreading, the temperature field that has Noita as its
  aspiration, `master/spec.md`) faces the same two questions: who wins a contested cell, and is the answer the same
  on every card. Blocks that only touch their own cells answer both.

## Where it would sit

Not in the plan's steps yet. The FX Lab grows out of the Graphics Kit (after 3.3); the gauges are the game's
(Phase 4). On the software map it would be a pass (the gauges' step) with a store (the gauges' cells), on its own
fixed tick. Names and placement are David's.

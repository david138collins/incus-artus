# Incus Artus — World Generation

From David's notes, 2026-10-23. Tags as elsewhere:
**[DECIDED]** David said so · **[PROPOSED]** Claude's suggestion, waiting for a yes · **[BUILT]** · **[QUESTION]**.

---

## 1. The suite, not the generator  [DECIDED, first version BUILT]

**We're not building the world gen, we're building the suite we build the world gen with.** It's the stack
we use to tune the system that generates terrain, world, history, villages and society, one step at a time.

- **A separate screen inside the program** (F2). The physical world is switched off while it's open (time
  paused, no input to the camera or body). Almost a separate program that makes data for the game to use later.
- **Eventually a fun and attractive process for the players.** Until then, the graphics are placeholder,
  high-contrast, graphing-calculator tier.
- **Coloured pixels are units of land**, plus dots and icons. This is a workable, adjustable way to see it now,
  and something David can later reason about when making map tiles:

  | Land | Colour |
  |---|---|
  | ocean | dark blue |
  | rivers | light blue |
  | plains | light green |
  | forests | dark green |
  | mountains | grey |
  | roads | yellow |

### What's built (`src/worldgen/`)

- **Steps** stacked in order. Each step has its settings as sliders, and writes **layers**:
  - numbers (height, moisture...), shown as a ramp: grey, heat or bands;
  - kinds (ocean, plains...), shown in the palette;
  - regions, shown as patches with borders.

  Steps can also put down **marks** (dots with names).
- **Runs in the background** and shows each step as soon as it's done. Moving a slider re-runs from that step
  down, and the steps before it are kept.
- **One step at a time:** turn off "Run by itself" and use "Next step" to watch the world being made.
- **Look back:** the map as it was after any step ("map" button).
- **Inspector:** every layer's value under the mouse, the cell and its position in blocks (north / east).
- **Log:** every step's lines. Clicking one marked ⌖ shows where it happened.
- **Presets** of the whole stack: seed, world size, resolution, colours and every step's settings
  (`presets/worldgen/`).
- **Each step rolls its own dice** (the world seed plus the step's name), so changing one step never changes
  another step's randomness. The same seed and settings always make the same world.
- **Steps work in blocks, not pixels,** so a coarser or finer map shows the same world in less or more detail.
  Resolution: 1 pixel = 256, 128, 64 (the default: 1,125 pixels across the 72,000-block world) or 32 blocks.
- **Four sample steps (TEMPORARY):** height, moisture, land kinds, regions. They're only there to prove the
  suite works. The real steps get designed one by one.

## 2. Biomes, regions and factions  [DECIDED]

- **Biomes are areas of influence** that add up to a biome conclusion based on ecological factors (soil,
  weather...). These areas can be expressed as cells.
- **Those cells divide neatly into several smaller areas.** A Voronoi pass breaks every rough terrain
  combination into appropriately sized **regions**. Chunks are probably the final resolution for regional zoning.
- **Procedural names:** if the soil and weather make a woods biome, it gets named ("Darkenwood"). If it's big
  enough to split into 3 cells, they're named North Darkenwood, South Darkenwood and East Darkenwood. David will
  add more language conventions to name that many things.
- **Regions are the base unit kingdoms and factions operate in.** The player's map can shade a region as
  belonging to the influence of one faction or another. The size of regions has to be tuned carefully.
- **A goblin den prevents building or expanding** in its region.
- **Resource outposts:** a small mining camp in the region next to a town's buildable area and simulation zone.
  Villages in the light simulation can use them, fight over them, or find them contested by randomly spawned
  bandits or highwaymen.
- **The God of Darkness** has a layer of influence at the base of the world's activity, out of view: some areas
  are naturally bad-aligned. Part of a larger woods biome might become a haunted forest, spooky forest or spider
  forest, named accordingly, with phantoms, a haunted graveyard or huge spider nests.

## 3. Races and barbaric factions  [DECIDED]

- **Three "human" races:** humans, dwarves and elves. Their people are full people: urges, jobs, likes,
  friends.
- **The rest (goblins and so on) are barbaric factions.** Their behaviors are simpler. They serve the same
  purpose as Minecraft's monster spawners, writ large: they populate the world with threats and enemies to
  fight for rewards and fun. The difference is that they do it in a way that gives a sense of purpose and
  narrative behind it.
- **Exceptions are possible**, for example a quest to talk to one goblin who's not like the rest, placed in a
  unique neutral goblin village.

## 4. The chronicle  [DECIDED, not built]

- **One chronicle service** records what happens.
- **During world generation** it lets us read how the world is being made, moment by moment and step by step,
  while we watch and tune. (The suite's log is the first, simple version.)
- **Flagged events are exposed to:**
  - the player's event log;
  - couriers travelling the world;
  - conversation: the generic things many NPCs say about current events, so something comes up when you speak
    to them.

## 5. The calendar and history  [DECIDED]

- **The calendar starts at 0 at world generation** and stays there until the world is populated.
- **Then time starts:** factions and other NPC entities begin living in time. They build onto their towns,
  build roads, form larger factions and fight, over **1,000 years of history** before the game starts.
- **You start in year 1000**, as an originless player character somewhere random, with not much to your name.

## Next steps  [PROPOSED]

1. David tries the suite and its sample steps, and says what's missing from the workbench itself.
2. Design the real stack together, one step at a time, starting at the bottom (the planet's shape and
   plates, or straight to elevation).
3. Steps the suite will need along the way:
   - a **timeline** for the history steps (a year slider, play and pause);
   - **icons** for marks (sites, dens, ruins);
   - the **chronicle** (replacing the log);
   - a **stress readout** (how long one simulated year takes).

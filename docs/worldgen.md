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

### Rebuilt one tool at a time  [DECIDED, 2026-10-23]

The first version (a four-step sample stack: height, moisture, land kinds, regions) showed where the suite is
heading, but too much at once to evaluate, adjust and tune. So it was **peeled back to nuts and bolts**:
- **One tool at a time.** David tries it, Claude gives options, and David picks the next one. Every element is
  hand-selected and carefully considered before moving on.
- **Noise first, not maps.** No forests or plains yet: just greyscale fields of different kinds, with many
  controls over the maths that makes them. Then ways to find **edges and boundaries** around lumps of noise
  (parts of a planet, areas of rainfall), saved as generic fields.
- **A palette of field parameters,** saved as presets, which later gets painted with environmental properties
  to derive world information from.
- **Things to experiment with:** types of noise formula, cell creation, partitioning schemes, and ways to
  interpolate between them.

The first version is in git (commit 2513488) if any of it is wanted back. David looked at it with these
settings:
- 1 pixel = 256 blocks;
- height: size 15,000, 6 layers, roughness 0.54, edge from 0.57, sinking 0.25;
- moisture: 10,500, 12 layers, 0.9;
- land: 70% water, 25% mountains, forest above 0.51;
- regions: 1,800 blocks across, smallest patch 30%, 8 rounds.

### Tools so far

| # | Tool | State |
|---|---|---|
| 1 | **Perlin field** | BUILT, being tried |

**Tool 1, the Perlin field** (`src/worldgen/noise.rs`):

- **Pattern:** size (blocks), octaves, persistence, lacunarity.
- **Placement:** slide north/east, stretch and its direction, turn each octave.
- **Maths:** the fade curve (smooth 2002 / soft 1985 / straight), solo one octave.
- **Output:** fill black to white, contrast, brightness, curve, invert.
- **Readouts:**
  - the raw and final value under the mouse, with its position in blocks;
  - the raw range, the average, how much is clipped to pure black or white;
  - a histogram.
- **Noise presets** in `presets/noise/`. The seed isn't saved with a preset, so a preset is a *kind* of noise
  that works with any seed.

**What stays from the first version** (the plumbing, not tools):
- the separate screen (F2), with the world off and the debug panel hidden;
- making the field in the background;
- pan and zoom;
- resolutions of 256, 128, 64 or 32 blocks per pixel;
- noise worked out in blocks, so resolution doesn't change the field.

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

## Next steps

David tries the Perlin field, then picks the next tool.

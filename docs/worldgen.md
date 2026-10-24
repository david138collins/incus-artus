# Incus Artus — World Generation

From David's notes, 2026-10-23. Tags as elsewhere:
**[DECIDED]** David said so · **[PROPOSED]** Claude's suggestion, waiting for a yes · **[BUILT]** · **[QUESTION]**.

---

## 0. The Editor Suite: Graphics Kit and World Gen  [DECIDED 2026-10-23, BUILT]

The suite turned out bigger than first planned. It's now **the Editor Suite**, with a switch at the top:

- **The Graphics Kit** is everything built so far (tools 1–5): noise, pictures, mixing, palettes, export. It's
  the art pipeline: grading and colourising assets, getting a handle on the palettes and the game's colour
  philosophy, and visual effects. David found effects in it that will be handy later with **oscillators and a
  loop recorder** (settings that change over time). Kept as it is.
- **World Gen** is where pixels **mean** something (plate, ocean, mountain, spring, river). It **sorts** values
  into what they become (a water level), and runs **processes in steps** (plates moving, rivers flowing
  downhill).
- **Where a feature goes:**
  - a pure pixel operation (masks, radial gradients, thresholds) belongs to the shared engine, for both;
  - anything that attaches meaning, or runs over time and steps, belongs to World Gen.

  So **rivers belong to World Gen.** World Gen can use the Graphics Kit's noise, presets and palettes as
  ingredients.
- **Shared:** the field and pixel size (in the suite's bar), pan and zoom, palettes and presets.
- Master craftsmen spend much of their time making and maintaining tools; this is that. It may get messy. If it
  does, the editor can be started over later with a more focused logic, now that we know more.

**The world is the circle** inside the square field. The square exists only because image files are square. For
the player, the circle is the boundary (the ice wall is past it; few players will sail that far).

### World Gen tool 1: plates  [BUILT 2026-10-23]
- **"New seeds for everything"** at the top of World Gen. There's one **world seed**, and each step takes its own
  seed from it, so one press changes the whole world. Cycle it to see whether the settings routinely make good
  shapes.
- **How the plates are found** (`plates.rs`):
  1. Scatter points and split the field into Voronoi cells.
  2. Optionally spread them evenly (Lloyd's relaxation, in rounds).
  3. Delete every cell outside the circle or touching its edge.
  4. Keep the cells nearest the middle.
- **Defaults:** 7 plates covering 30% of the world's circle (David: "for now, just make sure we can vary it").
  Both are sliders, and the number of points scattered follows from them. The readout shows the coverage that
  actually came out, and each plate's share.
- **Wobbly edges** [David, 2026-10-23, BUILT]:
  - the vertices (where three cells meet) are found from the pixel map and made exact;
  - each edge between two vertices is pushed sideways (perpendicular to it) by Perlin noise running along it;
  - the push fades to nothing at the vertices, so the corners stay put;
  - **two line mutators added together:** **large waves** and **small roughness**, each switchable on or off;
  - **each has a style switch:**
    - **Perlin noise** along the line, faded to nothing at the ends: smooth and rolling. Its controls are
      amount, wave size, octaves and roughness.
    - **Midpoint displacement:** halve the line and nudge the middle sideways at random, then halve again with
      smaller nudges, down to the finest step. Jagged and coastline-like, and the ends never move. Its
      controls are amount (the first nudge), finest step and roughness.
  - the defaults are Perlin waves (3,000 blocks across 15,000-block waves) plus midpoint roughness (800
    blocks, down to 1,000-block steps);
  - **vertices and lines out in the ocean are culled.** Only those touching a plate are kept and shown;
  - the vertices and the straight lines can be shown over the picture.

  This only reshapes the plates. Behaviours come after.

### World Gen tool 2: tectonics  [DECIDED 2026-10-24, building]
David's picture: **plates are rags on a table being pushed around.** They wrinkle most at the edges, but their
stiffness is irregular, so lesser wrinkles appear further back too. Land pushed up shows **brighter**, land
pushed down **darker**.

- **How:** each plate becomes a mesh of points joined by springs. Stiffness comes from a noise map. Each small
  triangle keeps its amount of rock, so squeezed ground thickens (brighter, mountains) and stretched ground
  thins (darker, rifts). Nobody places mountains; they're where rock got squeezed. Fold ridges are added at
  right angles to the squeeze.
- **Boundaries come from the same rules:** convergent (squeeze: ridges), divergent (stretch: rifts), transform
  (sliding: friction grips and turns the plates).
- **Ocean plates:** the ring of cells touching the land plates becomes ocean plates. They move too, and make
  the coast types: ocean diving under land gives a trench and a mountain line just inland; ocean against
  ocean gives island chains; land against land gives the biggest mountains.
- **Ocean splitting** [David]: two plates pulling apart in the ocean leave a **V-shaped valley** between them,
  widening at the top as they diverge. Some islands formed by plate motion near the continents are welcome,
  but the ocean should become **empty very quickly** sailing out from land toward the edges.
- **Motion** [David]: plates are **given their motion at the start, then coast until they slow down** (like
  flicking a rag). The force, how slowly it dies away and so on are sliders, to find the tasty ranges.
- **The edge** [David]: plates never reach it. If needed, an imaginary **bowl** surrounds them (like a
  Beyblade arena): a radial slope from the border inward that keeps them from straying far from the middle.
- **Watching** [David]: it runs **live**, with a **speed control that changes only how fast it plays**, never
  the result. Same seed and settings, same mountains.
- **Order, one launch each:**
  1. ocean ring and plate properties (direction, speed, spin as arrows; boundaries coloured by what *would*
     happen: red colliding, blue pulling apart, green sliding). Nothing moves yet;
  2. rigid motion: Start / Pause / Step / Reset, the bowl, collisions and friction turning the plates;
  3. rags: springs, irregular stiffness, height from squeezing, fold ridges;
  4. subduction, rifts and new ocean floor (the V-shaped valleys);
  5. bake the result into a height map.

- **After tectonics, one tool at a time:**
  - a water level;
  - springs and rivers: a field of random dots culled to plausible springs, shown as light blue pixels, that
    run downhill and thicken as tributaries join, out to the ocean.

**Open question** [David]: start from Perlin noise (continents cut out with a radial mask), or simulate plates
before history starts? Having both lets them be compared, or combined (plates for the big shapes, noise for
detail). A **mask layer** (radial, linear, ring, with a falloff curve) is wanted either way: for example, lowering
land toward the edge so the continents are surrounded by ocean.

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
| 1 | **Perlin field** | BUILT; David: "looking really great" |
| 2 | **Export, palettes and the HSL filter** | BUILT |
| 3 | **Layers, image layers, chunkier pixels** | BUILT, being tried |
| 4 | **Mix modes and opacity** (layers affect each other) | BUILT, being tried |
| 5 | **Worley noise field** (cellular / Voronoi) | BUILT, being tried |

**Where colour stands** [David, 2026-10-23]: parts of how the suite works, colour particularly, will need
rethinking, but David can't say exactly how yet. We keep tinkering with it as it is. If needed, the suite gets
started over from scratch later.

**Tool 5, Worley noise** (`worley.rs`) [BUILT 2026-10-23]: points scattered over a grid; every spot belongs to its
nearest point, which splits the world into cells.
- **Cells:** cell size (blocks); jitter (0 = a neat grid, 1 = fully random).
- **Shows:**
  - nearest (F1, bubbles);
  - second nearest (F2, puffier);
  - edges (F2 − F1, cracks and borders);
  - cells (one random value per cell, the classic Voronoi patches).
- **Distance:** straight (round cells), grid (diamond), longest side (square).
- Octaves, placement and output as for Perlin. Its own presets (`presets/worley/`).

**Tool 4, mixing** (`blend.rs`) [DECIDED 2026-10-23, BUILT]:
- **Every layer has a mix mode and an opacity:** normal, add, subtract, multiply, screen, overlay, dodge, burn,
  lighten, darken, difference. Each one's hover text says what it's good for in terrain.
- **Layers mix their numbers, not their colours** (David: "ultimately this software is translating numbers to
  colors and colors to numbers"). The stack makes one field of 0–1 values, and **the result** (top of the Layers
  list) has its own palette that colours it. This is what the world file needs.
- **A layer's own colouring shows when it's shown alone** ("show: the open layer alone"). That's also where an
  image's own colours show.
- Where nothing is below a layer (the bottom layer, or outside an image), its own value is used as it is.
- "Under the mouse" shows the result's value at the top. Opening the result shows its histogram.
- **Next, probably:** adjustment layers (David's "interaction / mutation layers") that sit between layers with no
  picture of their own and change everything below: thresholds, curves, "keep only where below is above X".
- **Where it's heading:** a sequential "world type" file that builds the standard world. It will be made of
  layers, mutation layers, groups, spline maps, and signal routing inside some layers to place features.

**Watching and duplicating** [David, 2026-10-23, BUILT]:
- **What's on screen is chosen separately from the open layer** ("watching:" at the top of the Layers tab), so
  one thing can be watched while another is adjusted: see the effect, not the layer causing it. Choices:
  - the result;
  - any layer alone (its own colours);
  - everything up to a layer (the stack as it stands there, before the layers above mix in).
- The canvas says what's being watched, and the watched layer is marked » in the list.
- **Duplicate** copies the open layer exactly (same seed, settings, mix, colours), just above it.

**Side bars** can be dragged between about 220 (left) / 180 (right) and 900 pixels wide. Long layer names are
cut short with "…", and hovering shows the full name.

**Tool 3, layers** (`layers.rs`, `image.rs`) [DECIDED 2026-10-23, BUILT]:
- **Starts blank.** "+ Add layer" in the upper left: a **Perlin noise field**, or an **image from a file** (from
  `images/`, or drag a PNG onto the window).
- **A Layers tab on the right**, like an image editor, top of the stack first. Each layer:
  - can be named ("Perlin noise field 1", or an image's file name);
  - can be shown or hidden, moved up or down, or deleted.

  Under the list: everything under the mouse (each layer's value), then the open layer's numbers and
  histogram.
- **Layers are opaque:** you see the top visible layer, and the one below shows through where it doesn't cover
  (outside an image, or see-through pixels). Below everything is a dark checkerboard. Exports are see-through
  there. Blending layers is a later tool.
- **Each layer has its own** seed, settings, output shaping and colouring.
- **Image layers:**
  - the picture's brightness becomes the values, and its own colours are kept;
  - placed by centre (blocks N/E) and blocks per picture pixel;
  - "1 picture pixel = 1 field pixel", "fit the field", and **repeat** to tile it;
  - **what for:** stamps of David's textures (a rough burlap weave under an "ancient temple" for buried, eroded
    structures), and the game's art workflow.
- **The default palette** is generated: N greys from black to white (128 to start), saved as `.gpl` with
  "Save .gpl". An image can show its own colours or any palette, and the HSL filter works on either.

**Pixel size in chunks** [BUILT; David, 2026-10-23: "i really like 256", wanted chunkier]:
- **1 pixel = 1 to 45 chunks** (32 to 1,440 blocks).
- **These fit a 72,000-block field exactly:** 1, 2, 3, 5, 6, 9, 10, 15, 18, 25, 30 and 45 chunks. The others leave
  a part-pixel at the edge, which the top bar says.
- **Why chunks:** a chunk (32 blocks) is a storage and drawing convenience, not a terrain unit (see
  `decisions.md`). Pixels don't have to match chunks. But whole-chunk pixels line up with the chunk files, which
  will make it simple to hand world data to the terrain later, chunk by chunk. 720 blocks would be 22½ chunks.

**Working horizontally for a while** [DECIDED, 2026-10-23]: elaborate what we've got before adding new kinds of
field. **The next destination is a Layers tab**, like a graphics editor: start with a blank page and pick the
field type in the upper left (for now, the only type is Perlin field).

**Why the suite is also an art tool** [DECIDED]:
- **Using the tools to make the tools.** Truly versatile noise generators that work black to white, can be
  colourised to a palette, and snipped out to fill areas of a UI element are surprisingly rare.
- **Recursive, fun feedback loops** make it fun to make a fun game.
- **Cohesion through David's hand on every step.** Using the world engine to make the shades of the menu's stone
  panels creates connections in the instinctive, feeling part of people's brains.
- **Palettes will tie directly into how the map is rendered**, in several different styles. So tight
  integration now is right: colour must be a delicious, deliberate experience from top to bottom. Bundling the
  work this software does into the work it makes keeps everything touching everything else.

**Tool 2, export, palettes and the HSL filter** (`export.rs`, `palette.rs`):
- **Export what's in view** or **the whole field** as PNG, one field pixel per picture pixel, coloured as on
  screen. Saved in `exports/`.
- **Load a GIMP palette (`.gpl`)** from `palettes/` (Aseprite: Palette → Save Palette). Its colours are spread
  from the valleys to the peaks in the file's order:
  - **equal shares** (each colour covers the same amount of the picture) or **equal steps** of value
    (contour-style bands);
  - hard bands or blended;
  - reversible.
- **HSL filter:** hue, saturation and lightness separately. Each can be kept, set to one value for every
  colour, or **taken from the field** (for example, the palette's hues shaded by the noise).
- **Save the filtered palette** as a `.gpl`.
- The colour under the mouse shows in the inspector, as hex.

**No dithering** [DECIDED, 2026-10-23]: programmatic dithering is kind of cheap and doesn't give useful patterns
for natural terrain, so none of anything for now. It may come back later with the shaders and graphics
refinements, in the 3D world, for things like sunsets, clouds or animated water.

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

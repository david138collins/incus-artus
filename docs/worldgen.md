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

**An asset library page** [IDEA, David 2026-10-28; name and placement David's call]: a block
registration suite and manager in the editor, so David can add and change blocks himself instead of
asking Builder to put each one in. Comes after a round of graphics polishing and probing for easy wins,
so the blocks are seen right when they go in.
Counsel's notes [PROPOSED]:
- **Blocks stay data, not code:** each block a definition file (`docs/block-properties.md`, the
  Definition tier): name, material, textures for top, sides and bottom, colour tint, hardness, sounds.
  The page writes those files. This is also what modding needs.
- **Stable names, never list positions:** saved worlds store which block is where, so a block's id must
  never change when blocks are added, reordered or renamed on the page (a string id like `dry_grass`,
  with the display name free to change).
- **A preview lit like the world:** the block on a small stand in sun and shade, near and far, beside
  its neighbours, so a texture is judged as it will be seen.
- **Shared textures, many colours:** one texture with a tint per material (as `block-properties.md`
  plans), so the page offers "new block from this texture" as well as "new texture".
- **Where the files live:** `Art/` is David's untracked scratch space. A registered block's textures
  should be copied into the tracked game assets, so the game never depends on scratch files and the
  art David registers gets saved to git (watch list item 12a).
- **Reload without restarting:** change a texture, see it in the world.

**The Registrar and its Color Lab** [IDEA, David 2026-10-29]. Names: "Registrar" is David's likely
name for the block register suite ("suite" is a working word). David would rather call the Terrain
Compiler the **World Compiler**, since it covers everything up to non-sentient life (not renamed in
the code yet).
- **The library page:** every asset the game has. A **dock** shows the asset that's open, or loads a
  new one. In the dock you set its properties and register it into the library. The dock also shows
  the palette that asset uses.
- **The Color Lab**, a page of the Registrar, as the World Compiler has a terrain lab. It may later
  live in the Graphics Kit too, or follow David between the two.
  - **The global palette** is added up from every registered block.
  - **Highlighting:** opening or mousing over an asset in the library lights up the colours it uses.
  - **Grading tools:** register a block that uses a colour not yet in the palette, and the Lab points
    it out and lets David replace it. It shows a mini palette of the nearest colours already in the
    palette in each HSL direction (lighter, darker, more or less saturated, the hue either way).
  - **The source of colours for FX and UI:** they get their colours from the Color Lab too, so all
    colours are managed in one place.
  - **Selections** [David 2026-10-29]: select colours from the global palette or from a library
    asset's palette. Select groups of assets and see their combined palette.
  - **Export to GPL:** any of these palettes (global, one asset's, a group's, a selection) can be
    exported as a `.gpl` palette file.
  - **Replace across assets:** update registered assets one at a time or in groups, replacing a colour
    across every asset selected.
  - **In the Graphics Kit**, some of these functions may need to work slightly differently: in the
    animator, and on the widget/toy/FX script page, which is 2D and 3D.
  - **The master palette from spline sets** [David 2026-10-29]: each hue gets HSL ramp splines that
    set its shades. This answers the problem that every hue needs its own curve to the eye, which so
    far has meant hand-adjusting each shade. The master palette could be defined entirely from these
    HSL formulas.
    - Some things will still need more control than splines give.
    - **Colours that don't come from a formula must work fine** until they're all sorted into
      splines.
    - Eventually David, and the engines, work entirely from the master spline sets.
    - **Insert a shade without shifting the others:** duplicate a colour and place it next to itself,
      expanding the spline. Every colour keeps a permanent slot id, whatever its place on the spline,
      so blocks using the old shades don't change. This is how odd tones join a spline, until no
      colour outside a spline is needed.
    - **Starting fresh** [David 2026-10-29]: once the Color Lab works, David starts the palette fresh on
      splines (the current blocks are few and rough). Old blocks' colours show as dots beside a curve
      to shape it by hand; there's no automatic fitting. **The rule: move everything into a spline as
      soon as possible.**
    - **Reshading onto a spline** is part of registering: each pixel is matched to a chosen spline,
      either by nearest colour, or by rank (the darkest colour to the darkest step and so on, which
      keeps the texture's contrast). Then David touches up by hand.
      **It works like the Graphics Kit's palette tools** [David 2026-10-29], with sliders and settings
      for *how* an asset is reshaded, not one fixed method. This is the feature set David spent a week
      trying to buy before starting in Bevy and Rust. What the Kit already has (`src/worldgen/palette.rs`)
      maps over: the spread (an equal slice of the *values* for each colour, like contour bands, or an
      equal share of the *pixels*, the rank method), hard bands or smooth blends, and the HSL filter
      (each of hue, saturation and lightness kept, set to one value, or taken from the source; for an
      asset, the source is its own pixels' lightness). These should be the same code, shared, not a
      second copy.
  - **Materials by swapping splines** [David 2026-10-29, via Builder]: draw a block (say limonite) using
    one hue spline, duplicate it, switch the copy to a different spline, and call it hematite. A block
    can draw from several splines, which is how blocks stay more than palette swaps. Inserting a shade
    must never disturb the existing ones. (This replaces "a palette row per material".)
    *A duplicate is simply a new block* [David 2026-10-29: "linked" was dropped, as it meant nothing
    beyond regulated or unregulated].
  - **Art always has a way in** [David 2026-10-29]: David keeps the ability to make art anywhere and
    work it into the library through the Color Lab. As the library grows this will be used less, but it
    always keeps its value: any file David needs gets into the game, regulated by the Color Lab.
  - **Regulation status:** the only status is **regulated** (every colour on a spline) or
    **unregulated**, so David can see at a glance what's homologated. Full homologation is what will
    later allow kung fu with colour rendering.
  - **[DECIDED 2026-10-29, via Builder] Placement and first version:**
    - **The Registrar is the first tab** of the Editor Suite: Registrar | Graphics Kit | Tectonic Sim |
      World Compiler. It's one page, side by side: the library by tag | the dock | the master palette
      and a sub-palette.
    - **Grouping by tags, not folders:** any number of tags per block, nested with "/" like folders
      ("rock/igneous"). It stays extensible as block properties grow. Built as 353260c (`tags` in block
      files, `docs/data-files.md`).
    - **The first goal:** open the Registrar and see the library's blocks, with all their unregulated
      colours in the master palette, and a button ready to make a new spline and start sorting. A
      sub-palette viewer shows a group, one asset or a new unregulated asset, with HSL transforms.
      David: "sneak up on the final configuration."
    - **Storage (Builder's design):** the master palette is `assets/data/palette.toml`, with permanent
      slots (`next_slot` never reused) and splines as lists of slots. The PNGs stay what David paints.
      A pixel's slot is found by exact colour match. When the Lab changes a colour, it rewrites those
      pixels in the PNGs at the same moment, so there are no sidecar files.
      *Counsel's notes:* with exact match, **each slot's colour must be unique**. A shade duplicated
      next to itself has to differ by at least a hair before any pixel uses it, and two splines can't
      hold the same colour, or the pixels can't tell which slot they mean. And since the Lab rewrites
      PNGs, keep a backup or undo of the files it touches, and watch out for a file that's open in
      Aseprite at the time.
  - **Snapping or free shading stays open as long as possible** [David]. Builder's way of doing slot
    textures (turning the slots into ordinary colours on the CPU, mipmaps made from those colours)
    keeps both open: the slot ids are always there, so a snapping shader can be added later.
  - **Collapsing sections** [David 2026-10-29]: the Color Lab's sections fold, so David can pick what's in
    view while working.
  - **The crusher** [built, David's request]: crushes an asset's colours, live in the texture viewer
    (gabbro.png: 235 colours look almost the same crushed to 8, with the HSL and crush settings). Open:
    how a crush becomes a spline, and how to crush to the palette (onto colours already in the master
    palette, the reshading feature).
  - **[DECIDED 2026-10-29] Palette cycles, done deliberately** (David): "we need a way to do palette
    cycles deliberately". Slots are normally turned into plain colours ahead of time (Builder's bake),
    so a block that animates needs its own path. That means water's ripples and wind over grass, and
    later anything else that cycles.
    - **How (Builder's assessment, 2026-10-29):** every block stays baked. A block marked "cycles" in
      the Registrar also gets an index layer (which spline and which step each pixel is on). The shader
      adds an offset to the step and looks up the spline's row. For water the offset is time × speed
      along the flow; for wind, a travelling wave from Climate's wind map, sampled like the light map.
      Wrap or bounce at a spline's ends is set per block. **Pixels off the cycling spline stay put, so
      David chooses what moves by which spline each pixel is painted from.** Only flagged faces pay for it.
      Lighting is untouched. Far off, it fades to the baked colours. To settle now: **permanent spline
      ids** (`next_spline_id`, never reused), so reordering splines never changes what a shader reads.
      Builder adds them in the load-a-new-asset round unless David objects.
    - **Named splines and spline groups** [David 2026-10-29]: David names splines and collapses them
      into **spline groups** while working toward the total colours to be used. *Counsel:* a name and a
      group are only labels on the permanent id, so renaming or regrouping never changes what blocks or
      shaders read. Groups fold like the Lab's sections, and could nest with "/" like tags.
    - **Wind ripples on grass** [IDEA, David]: wind makes the grass palette-cycle in expanding rings
      in the wind's direction. *Counsel:* a travelling wave (position along the wind − time × speed),
      gusts as rings carried downwind, its direction from Climate's own wind for that area. It works on
      grass block tops as well as sprites, so whole hillsides shimmer from the air.
  - **Fast round trips with Aseprite:** images and palettes bounce rapidly between the editor and
    Aseprite. Palettes go out as GPL, which Aseprite loads; PNGs come back and are picked up without
    restarting.
  - **Material families as transforms**, with the last of the tuning done by hand in the group palette
    view.
  - **HSL stays the working language** (David uses it and Aseprite has no OKLCH). OKLCH may run
    underneath, only for finding nearest neighbours and flagging near-twins, and maybe for an "OK
    lightness" readout beside HSL's.
  - **Seasons and biomes as real blocks, not palette shifts:** snow piling up 1/8 to a whole block,
    as in Minecraft, with drifts from Climate's wind. The same partial-levels system as water's 1-8.
Counsel's notes [PROPOSED]:
- **Splines are the recipe and the palette is its bake,** the same way the world recipe works: the
  spline sets are the source, the stored palette is what they produce, and a colour set by hand sits
  on top. Every palette slot records where it came from (a spline and a step, or by hand), so David can
  see what's still unsorted. A hand colour can later be adopted into a spline without its slot number
  changing, so no texture has to be touched.
- **The palette is a data file, one for the whole game.** The world, FX and UI all read it. The Color
  Lab is the only thing that writes it. Being a shared panel rather than part of one app is what lets
  it follow David between the Registrar and the Graphics Kit.
- **Slot textures make "replace" cheap.** If each texture pixel names a palette slot rather than
  holding a colour (the next structural step, below), changing a palette colour changes every block
  using it at once, with no repainting. Without slot textures, "replace" means rewriting the image
  files. That's another reason to build slot textures before the Registrar fills up.
- **Replacing in some assets but not others** [2026-10-29]: with slot textures, changing a palette colour
  changes it everywhere. Replacing it in only the selected assets means pointing their pixels at a
  different slot instead. So "replace" needs two forms: *change this colour* (everywhere) and *swap
  these assets to another colour* (only the selection). Show a preview before anything is written, and
  keep an undo, since a group replace touches many files.
- **One set of palette tools, used in several places:** selections, combined palettes, GPL export and
  replace are worth building once, as shared tools that the Registrar, the animator and the toy/FX page
  all use, each adding its own small differences on top, rather than three copies.
- **Counts and near-twins:** show how many blocks use each colour. A colour only one block uses is a
  candidate to merge. Two colours too close to tell apart are flagged as near-twins, so the palette's
  colours stay ones that read as meaning.
- **"New colour" means an exact match.** A pixel not exactly in the palette is new, so nothing slips
  in by a hair.
- This is where David's colour session happens (brightness A/B/C, the number of shades). Whether the
  final frame snaps to the palette stays open, to be decided by eye once the world is full.

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
     **[BUILT 2026-10-24]** Each step: lay every plate's pixels on the map; where two overlap, push back,
     give (soak up the meeting speed) and friction act at the middle of the overlap (friction off-centre
     turns the plates); the bowl pushes back any part past its start; drag slows everything; it stops
     when nothing moves a block per step. Crude height: overlapping ground on both plates rises with
     closing speed, and rides with the plate. Every setting is a slider and can change mid-run (Reset
     replays). Watching speed is steps per second only; a frame never spends more than 12 ms stepping,
     so heavy runs play slower but give the same result;
     **David's verdict** (2026-10-24): it feels like billiard balls, "six Australias shaken up in a bag".
     The controls are good (ranges to refine later), but plates must **deform and squash one another**.
     They should feel like **large, heavy objects**: you can't spin something big by hitting it, because
     the rest of the body resists, more the further away it is, and you're likely to **crack it or bend
     it** instead. So the rigid pieces are replaced by the rags;
  3. rags: springs, irregular stiffness, height from squeezing, fold ridges;
     **[BUILT 2026-10-24, all but the fold ridges]** (`rags.rs`). Each plate is a grid of points (`grain`,
     750 blocks) joined by springs, diagonals included. Each point only feels its neighbours, so a hit is
     felt first where it lands and travels through the plate about one point per pass; few passes = a
     heavy plate whose far side keeps coming while the near side crumples. Springs past **give-way** creep
     to a new resting length for good; stretched past **crack** (measured from how they started) they
     snap. Patchy stiffness from noise. Thickness = start area ÷ area now of the grid's triangles:
     thicker shows brighter, thinner darker. Defaults found by measuring: 3 passes, stiffness 0.3, push
     back 0.3 (at 8 passes / 0.5, under 1% of the land thickened at all). Fold ridges wait until David has
     seen whether patchy stiffness already wrinkles enough;
     **Heights and a water level** [DECIDED 2026-10-24, building] — David: the plates are flat floating
     masses and squash into one blob almost every time; he wants **1 to 3 continents with ocean between**,
     so land plates must be partly underwater before they collide. Every spot gets a height: the
     plate's base height (land near sea level, ocean deep) + **one world-wide terrain noise** (read where
     each pixel started, so it rides, bends and squashes with the plate, and flows across plate
     boundaries at the start) − **sunken edges** (continental shelves, so coasts fall inside plates) +
     **squeeze** (thickening raises ground: crumple belts become mountains). A **water level** slider,
     live at any time, decides what's land; a readout shows how much of the world is land. Collisions
     are unchanged (submerged parts still collide). **Ocean plates squeezed above the water level show as
     land: islands** [David];
     **Tilt** [David, 2026-10-24]: each plate also balances on its middle and tips a little, a random
     direction and amount (up to a slider): a straight slope, one side up and the other sunk. A **blend**
     slider mixes between plain tilt and the terrain noise. **The squeeze field (white haze) is a keeper**:
     use it later to shape terrain in a second pass, or as a mountainousness mask. **Cracking is
     switched off** [David, 2026-10-27: "it just doesn't behave or do anything useful after testing"]:
     rock pulled apart stretches thin (darker, lower) and never tears; only welds between plates let go.
     The "cracks at" and "tears open past" sliders are off the panel, their settings kept unused (so
     it could come back). Edge cracks (the edge splitting where it's crushed along its length) were
     tried the same day and taken out;
     **Spread per plate** [David, 2026-10-27: "spread the values each plate gets so that the plates vary
     from one to another"]: a **±** box in front of a slider gives each plate its own value, from its own
     random number (seeded). Base heights (land, ocean) move up or down by up to the spread; terrain noise
     amount, shelf reach and depth, squeeze lift, stiffness and weight (every plate, land or ocean) are
     multiplied, on a doubling scale (1 = half to twice, 2 = a quarter to four times). 0 = all plates
     alike, the default, so presets and bakes are as before. Stiffness and weight can change while it
     runs (a weld between two plates is halfway between their stiffnesses). Possible later: friction and
     push back (between two plates: their average), and blending values across a strip at plate borders
     if big spreads leave steps there;
     **Land and ocean plates separated** [David, 2026-10-27]: two sections above The simulation, **Land
     plates** and **Ocean plates**, each with its own motion (slowest, fastest, spin, lean toward the
     middle), heights (base height, tilt, blend, terrain noise amount, sunken edges, squeeze lift) and
     rock (patchiness, stiffness, give way, creep, drag, weight, how thick rock can get), spreads
     included. Shared and staying where they were: the terrain noise's size, detail and roughness and
     the water level (Heights and water); grain, patch size, passes, push back, friction, stickiness and
     the bowl (The simulation). Where land and ocean rock weld, the weld is halfway between their
     stiffness, give way and creep. Presets and bakes from before read as both kinds having the old
     values; "ocean plates' speed ×" and "ocean rock's weight ×" become the ocean's own speeds and
     weight;
  4. subduction, rifts and new ocean floor (the V-shaped valleys);
  5. bake the result into a height map. **[BUILT 2026-10-24]**, see "The Tectonic Sim's product" below.

### The Tectonic Sim's product, and the World Compiler  [DECIDED 2026-10-24]
- **Renamed:** the World Gen tab is now the **Tectonic Sim**. It's a sophisticated machine; once it's set
  up well it goes "in the closet", opened again only for fine tuning or to make new **world types** for
  players (each a different preset of tectonics settings).
- **Its product is a bake** [David]: Bake makes the world the current settings and seed make, from
  scratch at **1 chunk per pixel** (2,250 × 2,250, 32 blocks a pixel), runs it until everything stops
  (or a step limit), and writes `world bakes/<name>/`: `height.png`, `squeeze.png`,
  `push_direction.png`, `push_strength.png` (16-bit grey) and `plates.png` (8-bit), with `map.toml`
  saying how to read the greys, and holding the seed and every setting so it can be baked again. A
  full bake takes a few seconds. Any of the maps can be sent to the Graphics Kit; so can the live
  squeeze, height, land and squeeze-direction maps.
- **Squeeze direction** [David: "include it so I can see it"]: which way each spot was squashed most,
  and how hard, from each grid triangle's change of shape. Mountain ridges run across it. Shown as
  short lines on the map.
- **Open sea is as deep as the deepest plate rock** in the height map, so it reads as true deep ocean.
- **Next: a World Compiler tab** [DECIDED: its own tab, after the Tectonic Sim]. It reads a bake and
  turns it into the actual elevation map (and later the rest of world gen).

### World Compiler, first version  [DECIDED 2026-10-24, building]
- **Sea level Z = 128; the tallest mountains reach Z = 333** [David] (the world is Z 0 to 399).
- Reads a bake and makes **the ground's height in blocks everywhere** (one Z per pixel, 32 blocks a
  pixel). Sliders: which bake height is sea level (the land share shows), how deep the deepest ocean
  goes, how high highlands go, the shape of the curves between.
- **Mountains from the squeeze map**: crushed rock gets ridges, as tall as the crush is hard (never
  past Z 333), lined up **across the squeeze direction**. Stretched rock sinks into rift valleys.
- Shown as shaded relief coloured by height; pointing shows the ground's Z.
- Saved into the bake's folder as `elevation.png` (16-bit: the grey is Z directly), and can be sent to
  the Graphics Kit. Block-level detail is added later, when chunks are made.
- **[BUILT 2026-10-24]** First version, on the first bake (`first_world`): compiles 2,250 × 2,250 in
  under 50 ms. Mountain defaults lowered after seeing it (start at squeeze 0.05, fullest at 0.3): the
  first bake's crush belts are gentle.
- **Mountain curve [BUILT 2026-10-24]** [David: mountains came out as plateaus]: a curve box (drag the
  ends up and down, the middle to bend it) maps squeeze, from where mountains start to the bake's most
  crushed rock, to height. A belt's most crushed line is its peak. A **ridges** slider sets how much
  the ridge pattern carves in. (A full spline with more points can come later.)
- **Splines, grading, foothills, passes [BUILT 2026-10-24]** [David]: land, sea and mountain curves
  are **splines** (double-click to add a dot, drag, right-click to remove; smooth, no overshoot). The
  mountain spline spans squeeze from none to the bake's most crushed rock (the start/right-end sliders
  were always pinned at their ends, so they went). Bakes now save **tilt.png**; the compiler's
  **grade** uses it to steepen each plate's high side (land and sea depth) and soften its low side.
  **Foothills** blur a skirt around each range (keeping the peaks); **passes** dip the crest height
  along each ridge. **Rift depth** is measured against the bake's most stretched rock (it used to be
  measured against the most crushed, so it did almost nothing). Ranges re-centred so good results
  sit mid-knob.
- **Next for the compiler: an erosion pass** [David] to wear down the rough edges.

### The World Interpreter  [DECIDED and BUILT 2026-10-24]
The end of the chain, now with its own panel beside the World Compiler [David: "it's at the end of the
signal chain... it's now doing more than just repeating what it's told"].
```text
Tectonic Sim → bake (32-block pixels)
World Compiler (whole map, once): sea level · soften the coastline · soften the mountains
      → smooth maps: coast, height, tilt, squeeze, squeeze direction
Interpreter (every block, as the world is made): blend the maps to the block, then
      land & sea curves · grade · raised coast · the mountains' rise · ridgelines · hills
      · torn coast · detail · surface blocks
      → the ground, and the distant terrain (the same recipe, sampled coarsely)
```
- **Why:** the compiler worked the recipe out per pixel and handed over finished heights; smoothing
  between those swung past every sharp step the curves made (one hump per pixel along coasts and
  cliffs). Rule: **blend smooth inputs, then apply sharp curves**, never the other way round.
- **Blending:** Catmull–Rom through the 4 × 4 pixels round the spot, held within the four nearest
  pixels' values (smooth, and it can't swing past them).
- **Soften the coastline** (default 64 blocks, two pixels): the coast map is the bake's height near
  sea level (clamped to ±0.03 so both sides soften evenly and the coast doesn't move), blurred; the
  heights and tilt near it are blurred as much. Turns the pixel staircase into a curve.
- **Soften the mountains** (default 64 blocks; BUILT 2026-10-26): the squeeze and the push's
  direction and strength were handed over raw, so the mountain curve and the ridges laid across the
  push showed the pixels' steps: rows of notches along range flanks, in step with the 32-block grid
  (David's screenshot). Now blurred the same way (the direction as its doubled-angle arrow, so
  directions average properly), and the most crushed and stretched measured again after, so the
  tallest peaks stay as tall. Old presets get the default.
- **Coasts: each control does one job** [BUILT 2026-10-27, from "controls do what they say"]. Before,
  hills and ground detail set the coastline by accident: land starts at exactly Z 128 at the coast,
  hills flattened anything they'd have drowned to Z 128, and detail (±3 blocks everywhere) then decided
  block by block which of those flats was land, so a small move swung whole coastlines. David loves
  what it made (ponds, inlets, tide pools, islets and shoals: "it gives the whole continent a realistic
  feel when the coast in some areas is tore up and spotty"), so it became its own part:
  - **Torn coast** (Interpreter, between hills and detail; name is a placeholder): in a band along the
    coastline (reach inland, reach out to sea), a Perlin field pushes the ground back and forth across
    the waterline, by an amount; a slow pattern picks which stretches of coast are torn (how much is
    torn, stretch size); a balance leans it toward water (ponds, inlets) or land (islets, shoals). Reads
    the distance from the coast directly, so the land curve doesn't change what it does.
  - **Ground detail keeps off the waterline** (switch, on by default): near the waterline its bumps
    are held back so they never cross it. Off is the old way, for A/B. Old presets get it on, so their
    coasts change: the torn coast is how to get that look back.
  - **Raised coast** (Land and sea section) [David: tilt should "raise it above sea level on the ground
    side as well, with a smoothing spline on the exposed edge"]: before, tilt only changed how fast
    land climbed from the coast, and every coast met the sea at Z 128, so tilt could never make a
    cliff. Now, on a plate's high side, the land stands up out of the sea by the raised coast (blocks
    at tilt 1, more where tilted further, none on the low side), across an edge width, shaped by the
    edge curve (start it high for a sheer face at the water; low for a slope up from a beach). The
    coastline itself doesn't move. Off by default (0).
    **Became the edge rise** [2026-10-27]: in use it climbed inland from the shore (a wall set back
    from the coast with a narrow width) and then held the plate's whole high side raised, reaching
    its highest points. David liked what it does with a wide width: "a sort of continuous and gentle
    but noticeable hill that follows the plates' edges". So it was renamed (edge rise, rise width)
    and given **falls back over**: past the rise width, it eases back down to nothing over that far
    inland, making a hill along the edge and leaving the interior alone (0 = held up, as before).
    It follows the compiler's coastline, so where it starts can sit up to ~16 blocks off the final
    shore the torn coast makes. Cliffs at the waterline could be a control of their own later.
  - **Hills:** David is setting them to 0 while the coast is reworked; their future (perhaps the
    scalloped climbs toward the mountains he wants, so a peak can be seen over the ground in front)
    is the next question.
- **Foothills: cut 2026-10-27** [David]. They were a skirt of lower slopes round each range, made in the compiler (they needed the whole map): first the ridged mountains blurred, which tied how far they reached to how high they stood and smeared the ridges into lower, fatter copies; then, the same day, reach, height and a falloff spline as separate controls, with foothill ridges carved live on the Interpreter. Neither gave David anything he wanted ("if i can't get anything out of it after that, we're slashing foothills"), so they went: the skirt map, its Show switch, mask source and overlay with them. Masks that followed the foothills follow the mountains; worlds saved with a skirt map ignore it.
- **Passes: cut 2026-10-27** [David]. They dipped the crest height along each ridge, to make ways through the mountains. David uses the ridges only to shape where the mountains will go ("like a nest for them"), never for passes: ways through come from continents without so much unbroken range. Gone with them: pass spacing and their Show switch; the ridges are as they were with passes at 0.
- **Presets:** one compiler preset still holds both panels, as one flat list (old presets load).
- **Worlds save** the compiler's maps (`map-<name>.png`, 16-bit, each with its range in
  `compiled.toml`) and the recipe, so a world never changes when the panels do.
- **Cost:** about 0.7 ms per chunk's worth of columns on one core (spread over all cores).

### The terrain lab  [DECIDED 2026-10-24; core BUILT]
A true-scale inspection tool, so terrain can be judged without going in and out of worlds [David].
- **Built:** zooming into the World Compiler's map turns it into a **live close-up** of the ground as
  the Interpreter makes it (every block when zoomed right in; every 2nd, 4th… further out). It
  follows the Interpreter's sliders as they move: a new recipe reads the same compiler maps, nothing
  copied. **Layer switches** leave parts of the recipe out (raised coast, rise, ridgelines,
  ridges, hills, torn coast, detail). The close-up follows them, and so does Render World and Spawn: the world says along the top of the screen what it leaves out, and its save remembers them, so chunks streamed in later match. The big map always uses the whole recipe. **Cross-section:** Shift + drag draws a line; its side view (Z along it, with sea level)
  shows under the map, live. **Cost:** each close-up's time, and what that comes to per chunk.
- **[BUILT 2026-10-30, first version] The viewer upgraded for tuning climate** (David and Counsel). Tuning climate needs
  the map to show the blocks accurately enough to judge. David uses the close-up at 1x for everything,
  even the whole map, so the artificial "map" view may be retired once the close-up does everything it
  did (greyed out until then).
  - **What you see is what you get:** the real surface block (mineralogy's choice) in its far colour,
    the palette the distant terrain uses, at any zoom. One column per screen pixel when zoomed out,
    as the far terrain samples. It reads Climate's live result, the same hand-over the spawn uses.
  - **Keep the heights, recompute only the colours:** a climate slider doesn't change the ground's
    height, only which block is on top, so heights and slopes are kept per tile and a climate change
    only re-picks the blocks.
  - **Lit with the shared lighting sums** (`lighting.wesl`'s), with a switch to turn lighting off, so
    the hills read as they will from the air and the viewer can't drift from the game [David: "a good
    idea too"].
  - **Coarse first, then sharper** [David: a smooth feature if it can be done, but **today's way isn't
    offensive either**]. What matters: move a slider and see the world change, then zoom into a small
    area at 1x and dial it in with fast or instant updates while watching.
  - **[DECIDED] No "because" readout:** the terrain lab's hover readout (rainfall, temperature and so
    on) is enough. Point at a spot that should be dry grass and see whether it has too much rain or is too
    cool.
  - **[DECIDED] Blocks only, no simulated terrain shading:** with elevation or sun shading on the
    blocks, there's no need for a "just terrain" view or a transparency overlay, since all terrain is made
    of blocks. The map's simulated "rocky heights" shading goes: stone and the rest come from the block
    ranges David sets.
  - **Built:** the close-up shows each sample's top block (`layers_in`, with the Climate tab's live
    climate read straight from its pixels) in the distant palette's far colour, water by `under_sea`;
    lit with the distant terrain's hill shading and height tint (`distant::Shade`, the game's own
    settings), or flat with "lit" off (Terrain lab section). Mineralogy and the stone and gabbro heights
    only repaint the colours from the kept heights; climate sliders repaint when the climate catches up.
    The close-up's old height colours and relief shading are gone (the map view keeps them until it
    retires). Not yet: coarse-then-sharper; the side faces of steep steps (tops only for now).
  - **Open, David's call:** the flats just under sea level (held up by "crags never drown land") used
    to be invisible grass and are now wide sand flats along coasts. Keep them as tidal flats, or remove
    them in the recipe?
- **The pretense** [David]: "we'll explore various ways to mathematically define areas and lines in
  the output map. Using the clean modalities we can capture, we will drive a variety of scattering and
  particle systems to look for ways that can generate a second layer of mountains that only look
  wrong to a geologist, not a Skyrim fan."
- **Next: the modalities as overlays** on the lab: areas (mountain mask, distance from the coast and
  from a range, plate regions), lines (ridgelines, coastlines, each range's spine), fields (the squeeze
  direction as arrows or flow lines, and any flow field we build), points (what scatterers make).
- **Then the first scatterers, side by side on the same range:** Grok's particle streams (emitters on
  the real ranges, streaming outward, following the squeeze direction, each stamping a sharp short
  ridge rather than a round cone, with uneven spacing, stopping at the sea); a noise-ridge amplifier;
  and **fractal external rays** [David's idea, from his exchange with Grok]: a Mandelbrot set (and
  Julia sets, whose one complex dial morphs the shape smoothly) laid out in each range's own
  coordinates (along its spine, out from it), so every range gets it bent to follow it rather than
  fitting a range to the cardioid. Points spaced evenly by external angle land densely on exposed
  tips and sparsely in crevices (naturally dithered); the rays are a flow field outward; the body has
  its own interior fields. Cautions: self-similar motifs (bulbs on bulbs) are recognisable, so keep
  to one zoom and blend it in; it's costly (hundreds of iterations a sample), so it's worked out at
  compile time and the Interpreter stamps from it.
- **Live expectations:** Interpreter sliders follow as they move; compiler sliders take a moment
  (whole-map work); Tectonic Sim changes need a rerun.

### Terrain lab overlays  [BUILT 2026-10-25]
Phase 2 of the lab: the compiler's maps drawn over the map and the close-up, a switch each, with an
overall strength: **plates** (a colour each, outlined), **squeeze** (crushed red, stretched blue),
**mountains** (where the mountains curve raises them, orange),
**range spines** (the crest line of each range: where the mountain share peaks across it), **coast
distance** (the coastline in cyan, and lines every so many blocks inland and out to sea), and
**squeeze direction** (strokes along the push; ridges line up across them). Under the mouse: blocks
from the coast, mountain share. One value per map pixel, painted on all cores in the
background (about 0.2 s on a 2,250-pixel map; the meter says "drawing overlays"); only the mountains
curve and the switches repaint it. These are the first "modalities" for the scatterers to read.

### Mountains: "hope"  [FOUND 2026-10-25]
[David: "breathtaking results... we've stuck gold."] His preset `hope` had all six mountain sliders
pinned at the ends of their ranges: ridges 1 (most), ridge spacing 400 (least), ridge length 2,000
(least), sharpness 4 (most), passes 1 (most), pass spacing 3,000 (least), with a mountains curve
flat until the most crushed rock (0.84 → 0.125, then up to 1). Small, sharp, short ridges, broken
into peaks every few thousand blocks, only on the most crushed rock. The ranges are now centred on
those values (ridges and passes run to 2: past 1, valleys and passes reach the ground), and the
ridge pattern's hidden numbers are sliders: ridge layers (3), ridge roughness (0.45), ridge
alignment (1; how readily ridges line up across the squeeze).

**The giant fingerprint** [found 2026-10-25]: with hope's settings, some ranges were "perfectly
scalloped and irregular peaks" and others "a giant fingerprint, 20 blocks deep and 3 wide at the
grooves". Cause: the ridge pattern was turned to follow the squeeze direction by rotating the map's
own coordinates (measured from its corner, up to 72,000 blocks out), so wherever the squeeze bent,
the pattern swung round and crushed the ridges together, worse the further from the corner. Fix
("steady ridges", a checkbox for A/B): 12 fixed directions, each with its own pattern laid across
the whole map; each spot blends the two nearest its squeeze. Plus **ridge wander**: a slow noise
nudges the ridges sideways, so they bend and stop lining up with their neighbours.

### The processing meter  [BUILT 2026-10-24]
[David: reassurance whenever the computer has to think.] In the Editor Suite's top bar, in a slot of
its own: a spinner, what it's working out (compiling, EZ-baking, drawing the close-up, baking,
making plates, simulating, making layers) and for how long; "ready" otherwise.

### The pipeline after the landform  [DECIDED 2026-10-25, not built]
Worked out between David, Claude and Grok. The order, and who owns what:

```
Tectonic Sim
  -> TERRAIN COMPILER  (all of it runs at world creation and is saved)
       1. Landform + additive mountains
       2. Climate (rain, wind, temperature): before rivers, which need the rain
       3. Rivers & erosion (whole map)
       4. Streams & springs (every tile, at about 2 blocks, on the real ground)
       5. Sediment, soil, ore zones
  -> ECOLOGY        reads the ground, never changes it: biomes, plant density, species, animal ranges
  -> CIVILIZATION   towns, roads, territories, plus ground edits (road strips, flat building pads)
  -> INTERPRETER    reads the saved results: cuts channels, applies edits, places trees, ore veins
                    and surface blocks. Never simulates water. Same code for the lab and the world.
```

- **The compiler decides, the interpreter draws.** The compiler's output is *final* (every decision about
  the ground is made there) but not *literal*: at one block per column the world is ~4 billion columns,
  and finished heights on coarse pixels is exactly what made the repeating mounds. So it hands over
  smooth maps and crisp lines (rivers with bed heights along them, a carve map, soil and ore zones), and
  the interpreter makes the sharp shape at each block.
- **Every spring and creek is worked out when the world is made, and saved.** [David: the shape of a
  creek bed shouldn't be derived at runtime; pay the cost once, in one place, and never move it again.]
  The fine pass runs tile by tile (512-block tiles, ~15,500 of them) at about 2 blocks, running the
  interpreter's own recipe so the water follows the ground you'll walk on. Each tile is worked out only
  from the whole-map river network around it, so a tile made alone is the same as in a full make.
  Water always runs downhill and always connects: channels are carved to the bed height saved along
  each line, not "the ground minus 2".
- **Names** [David, 2026-10-25; renamed back to the **World Compiler** 2026-10-29, since it covers
  everything from the tectonics file up to non-sentient life]: the World Compiler was then the **Terrain Compiler** (it owns only the
  ground) and the Interpreter the **World Interpreter** (it makes the world from everything before it).
  **Ecology** (paints life onto the compiled terrain: plants, trees, animal habitat zones) and
  **Civilization** (people and hostile NPCs; a few local edits for roads, villages and structures) will
  be tabs before the interpreter, once the current scope is wrapped up. Civilization is tuned in a
  **civilization lab** (`docs/simulation.md`, section 11). "EZ-Bake" stays as the way a
  stage bundles its result for the next one. Ask David before naming or placing any new tab or panel.
  [David, 2026-10-26] The World Compiler page (still called that in the suite's row) has three
  left-hand panels, shown or hidden by a row of buttons along its top: **Landmass** (what was the
  World Compiler panel: sea level, coastline, soften the mountains), **Additives** (geographic features added
  onto the landmass before erosion and hydrology: dots scattered from the maps, then features driven
  by them; first scatterer: ridge streams off the range spines) and the **World Interpreter**.
- **Each stage caches its result.** Changing a slider redoes that stage and the ones after it, never
  the ones before.
- **World creation budget:** about 10 minutes on a slightly below-average PC (a GTX 980 Ti-era machine
  with 4-6 cores), so roughly a couple of minutes on David's laptop. Players get a loading screen with
  the generation shown live, with lore and art. [David: the wait makes a world feel like a committed
  artifact, and discourages rolling ten worlds looking for a perfect one.] Show the timings on the
  processing meter as each stage is built, so we hit that wall knowingly.
- **Three speeds for designing, all the same code:**
  - *Live* (instant): the lab close-up and late-stage sliders (texture, surface blocks, ecology looks).
  - *Fast make* (seconds): the whole world at map scale (landform, climate, big rivers, erosion), no
    streams. This is the inspection camera for hunting the right settings.
  - *Full make* (minutes, loading screen): everything, exactly what a player gets; spawn in to judge
    in person. The lab can full-make just the area it's zoomed into.
- **The cost we accept:** sliders that change where water runs (mountain shape, ridges) stop being
  instant once erosion exists; they need a fast make.

### The stages after Mountains  [IDEA, David 2026-10-28; nothing final until David says]

Relayed by Builder from David, 2026-10-28.

- **Next:** fine-tune the World Compiler as it is, and decide whether hills and ground detail stay.
- **The Additives may become "Mountains".** If they turn out to hold only mountains, the tab is renamed.
- **A new stage, "Climate", between Mountains and the World Interpreter.** It holds hydrology,
  temperature, wind, rainfall, very fine erosion, springs, creeks, streams, rivers and deltas. Water
  follows **pseudoscientific principles, tuned to look good for a game**, not real physics. Climate makes
  the fields that Ecology reads to lay out ranges, habitats, plants and trees.
  - This matches the 2026-10-25 order above, where climate (rain, wind, temperature) came before rivers
    because rivers need the rain; now they share one stage.
  - It replaces the plan of putting hydrology beside Mountains in the Additives.
- **Animals are part of the terrain.** All life that isn't sentient or evil is generated as a
  **likelihood per area**, not as tracked animals. Individual animals exist only in the full (Local)
  simulation, unless one becomes a pet. Their behaviour never reaches past the moment to moment: no
  herds migrating on the books, no animal records in the world sim.

Counsel's notes [PROPOSED, for David]:
- **The line between Climate and the Interpreter** is the same as the peaks': Climate *chooses* (where
  rivers run, how much flows, where springs rise, the rain and temperature maps), and the Interpreter
  *draws* (each block of a creek bed, sized to its flow). The fine streams at ~2 blocks are still worked
  out once at world creation and saved, as decided above.
- **Animals as likelihoods** fit the three levels (`docs/simulation.md`, section 1): World and Intermediator places
  only need "how many deer could live here" as a number for hunting, hides and food, and Local spawns
  real ones from it. If hunting should ever thin the deer, that's one number per area that recovers over
  time, still not tracked animals.
- **Pets are the one exception** and become person-like records, the way followers do. Worth deciding
  later whether livestock (which villages own and trade) counts as animals-as-likelihood or as goods.

#### Rivers, the first part of Climate  [PROPOSED by Builder 2026-10-28; waiting on David for step 1]

David (2026-10-28, after summit crags: "we're golden to get hydrological"): "first thing is working
rivers." His idea: dots in a range of elevations against the mountains as **springs**; dots on the ocean
and lakes that gather up the nearest springs, drawn as lines; springs gather to **creeks**, creeks to
**streams**, streams to **rivers**, rivers to a **delta**. The line snakes, following the path of least
resistance "up to a point before it carves". Five layers of dots in rings from the mountains, heading
for the ocean, or lakes for a river, ponds for a stream. "I think I'm leaving something out."

**Builder's proposal** (the change: dots run downhill and gather where their paths meet, instead of
gathering to the nearest point, which would send rivers over ridges):
- **Springs** are David's dots, scattered in an elevation band against the mountains.
- **Each spring walks downhill**, step by step, along the path of least resistance (like the grain
  drift dots).
- **Where paths meet, they gather.** The rank comes from how much has joined: creek, stream, river. The
  rings come out by themselves; David tunes where each rank starts instead of placing five layers.
- **Lakes and ponds:** a path into a basin fills it until it spills over the lowest edge and goes on.
- **"Up to a point before it carves"** is a setting: how high a barrier the water cuts through instead
  of pooling (low: more lakes; high: gorges).
- **Snaking** is a meander setting, strong on flat land, weak on steep.
- **Deltas** where a river reaches flat coast, splitting into branches.
- **What David left out** (Builder): rain (even everywhere at first; later Climate's rain and wind set
  each spring's share: rain shadows, dry basins with salt lakes); the water surface stepping down evenly
  in blocks (flat runs, rapids, waterfalls, never uphill); the valley (V-shaped in mountains, floodplains
  in lowlands, sized by rank); the 32-block trickle worry (paths are smooth lines; each block measures its
  distance to the nearest line, so a creek can be 1 to 3 blocks wide); everything else reading it (far
  terrain, the map, ecology's distance to water and wetness).
- **Pipeline:** the Climate tab between Additives and the World Interpreter reads the land (peaks and
  ridgelines included) at the map's 32-block pixels, works out springs, paths, lakes and ranks, and hands
  the river lines down; the Interpreter carves them at each block, after the hills. Nothing flows back.
- **Order:** (1) a Rivers section on the Climate tab: springs, downhill paths, gathering, ranks, lakes
  filling and spilling, shown as dots and lines on the map, no carving yet; (2) meanders; (3) carving and
  water in the world; (4) deltas, then rain.

Counsel's notes [PROPOSED, for David and Builder]:
- **Downhill, not nearest, is right** and matches what was decided on 2026-10-25 (above): water always
  runs downhill and always connects. David's rings and five ranks survive as the result, not the input.
- **Rank by flow, not by count of springs.** Give every spring one unit of water now, so the rank is
  "how much water", and rain later just changes each spring's units. Nothing has to be redone.
- **The fine ground isn't in the 32-block map.** Summit crags, the crags and the hills are drawn per block,
  so a line found at 32 blocks can meet a bump the map never saw. Two things keep it honest: the bed
  height is saved along each line and only ever goes down (decided 2026-10-25: "carved to the bed height
  saved along each line, not the ground minus 2"), and carving comes after the hills so it cuts through
  them. Creeks in steep mountains may later want the fine ~2-block tile pass decided above.
- **Lakes need a saved surface height,** flat across the lake, so the Interpreter fills to it.
- **Same seed, same rivers, in any order,** and a tile made alone matches the full make (as decided).

**David's answers for the river lines (Climate step 6)** [DECIDED 2026-10-29]. Mineralogy is roughed in
first; these wait for when rivers come back.
1. **Ranks, by width at the surface:** spring 1 block; creek 2-4; stream 4-8; river 8-32; great river
   32-128. A **delta** is a triangle about 64-128 blocks wide at the mouth. The widths double as rough
   flow figures for each rank, for whatever flow is used for: "we may keep it really simple".
2. **Springs:** scattered somewhat at random in a height band in the mountains. Water also starts from
   any body called a **lake** (as against a **pond**), depending on rain.
3. **The banks they cut,** width × depth: spring 1×1 to 1×2; creek 2×1 to 4×2; stream 4×2 to 8×3;
   river 8×4 to 32×8; great river 32×8 to 128×16.
4. **Smoothing by lengthening:** the line squiggles on its way to the next point, so it grows longer
   rather than just being softened. Whether a river runs **into** a lake and whether one runs **out** of
   it are separate questions.
5. **Ends:** in the sea, though some will end in lakes naturally.
6. **Waterfalls and rapids, of all shapes and sizes, are a big wow factor**: give them real attention
   (wherever the bed has to step down).

#### Breaking the water loop  [PROPOSED by Counsel 2026-10-28, for David]

**David's leaning (2026-10-28): keep everything before flowing water as simple as possible. Flowing
water is the hurdle.** And (later the same day) **set up everything before springs first**, each in its
simplest form, so the rivers are built on the real inputs. The suggested order (Counsel):
1. **Temperature:** the squiggled ice cap and equator lines with fades, milder by the sea, colder higher.
2. **Wind:** from the blurred temperature, turned partly sideways, sliding round tall ground. Curl noise
   later.
3. **Rain:** wind carries moisture from the sea and drops it where the ground rises; rain shadows.
4. **Then rivers** (Builder's plan, step 1 on).
5. ~~Lake effect last~~ **Left out** [David, 2026-10-28]: see whether steps 1-4 are satisfactory
   without it.
Sent to Builder for technical review, 2026-10-28.

**Builder's review (2026-10-28):** sound and cheap; all of climate, rivers included, in seconds.
- **A bug in the plan, fixed:** wind made from the full temperature would treat every big range as a
  cold source, so wind would blow outward from ranges and never cross them: no rain shadows, no lee
  drylands. **Wind reads sea-level temperature** (latitude and sea mildness only); rain, evaporation and
  the rest read the full temperature. Still one way.
- **Sampling the ground:** Climate calls the Interpreter's own recipe with crags, summit crags, hills and
  torn coast muted (shared maps, no copy, no second recipe). That's the broad shape without noise, which
  rivers need: crag noise would make thousands of false pits. Carving later goes into the real ground.
- **Resolutions:** temperature, wind and rain at 128 blocks a pixel; flow, basins and lakes on the 32-block
  grid; rivers as smoothed lines carved per block by distance to the line (a cell lookup, as the peaks
  have). The ~2-block creek pass is deferred until per-block carving shows whether it's needed.
- **Costs:** under a second for the ground samples, a fraction of a second for temperature, wind and rain
  (sliders near-live), 1-3 s for basins and flow (a background job). Saved: coarse fields and the lines, a
  few MB. Made once at compile, then only read.
- **Simplifications:** lakes **fill and spill only** at first (salt lakes and dry flats are their own
  later step, the fiddliest part); flow on the grid gives straight 45° channels, so heights get a small
  fixed seeded nudge before tracing and the lines are smoothed after.
- **Springs change:** with rain first, a spring is where the gathered flow crosses a threshold, not a dot
  in an elevation band.
- **Build steps:** (1) Climate tab skeleton and a broad-ground overlay; (2) temperature; (3) wind, with
  arrows; (4) rain: shadow, windward, upwind sea distance (the walks follow the wind field backward), each
  with a slider, overlay and "because" readout; (5) flow: basins, fill-and-spill lakes, flow in rain
  units; (6) river lines, ranked by flow, smoothed, bed height only falling downstream; (7) saved with the
  world, handed over at spawn; (8) carving and water in the Interpreter and distant terrain; (9) meanders,
  salt lakes, deltas, the desert picker.
- David's call: names and placement (Builder suggests one Climate tab between Additives and the
  Interpreter, with Temperature, Wind, Rain and Rivers sections), and the go-ahead for step 1.

**Candidate behaviours from real waterways** [PROPOSED by Counsel 2026-10-28; David asked for this
list as a menu, nothing chosen yet]. All water is rain or snowmelt: it runs off the surface, or soaks in
and comes back out lower down as springs (where the land dips below the water table: slope feet, valley
sides, along a rock layer's edge).

| Real behaviour | In the game | Cost |
|---|---|---|
| Runoff gathers into creeks | springs where gathered rain crosses a threshold (Builder's plan) | planned |
| Springs where steep meets flat | extra water where a slope eases off, so springs show at mountain feet | small |
| Snowmelt | rain on cold high ground counts extra; mountain rivers run bigger | one slider |
| Tarns (glacier-scooped lakes under peaks) | small high lakes; the broad ground may smooth their dips away, so maybe stamped | later |
| Fill-and-spill lakes | planned | planned |
| Waterfalls and rapids | where the bed has to step down | at carving |
| Meanders, oxbows, floodplains | winding strongest on flat ground | step 9 |
| Marshes | flat, wet, low ground | later, maybe |
| Salt lakes | under-fed basins | step 9 |

**[DECIDED 2026-10-28] David's answers to Builder's questions:**
1. **Placement:** a **Climate panel inside the World Compiler**, between Additives and the World
   Interpreter (not its own Editor Suite tab).
2. **Sections:** **Temperature · Wind · Rain · Rivers**, collapsible, in pipeline order; lakes inside
   Rivers.
3. **Latitude:** ice caps **north and south**, a hot equator band across the middle, squiggled.
4. **Behaviours for steps 4-6:** **springs at slope feet** and **snowmelt**. Not chosen: tarns and
   marshes (later or never). Waterfalls and rapids come with carving; meanders and salt lakes later, as
   planned.
Step 1 waits on David's go-ahead.

Counsel's note for step 8: lakes are found on the broad ground but drawn into the real, craggy ground.
Inside a lake, crag dips just fill and crag tops become islands (fine, maybe nice). The thing to watch is
the rim: where a crag dip cuts the rim below the lake's surface, water could stand against air like a
wall, since the Interpreter places still water and never simulates it. Likely answer: lower the lake's
surface to the real rim, or fill that dip with ground.
Counsel's reason to agree: river ranks tuned on even rain would shift once real rain arrives, so tuning
them on real rain from the start saves a retune. Each stage gets an overlay to judge it by.

**Mountains' effect on wind and rain, the simplest way** [PROPOSED by Counsel 2026-10-28, David asked]:
- **First version: mountains don't bend the wind; they act only on rain.** Rain is what you see.
  Bending the wind round mountains can come later if the rain looks wrong without it.
- **Each rain pixel looks upwind.** From each spot, step back against the wind for some distance (a
  setting) and read the ground along the way. Three things come out of that one walk:
  - **Rain shadow:** how far the highest ground upwind stands above this spot. High ground upwind means
    dry here, through a curve David shapes.
  - **Facing the wind:** if the ground rises toward this spot from upwind, the air is pushed up and drops
    rain: wetter on the windward slopes.
  - **Distance from the sea, upwind:** far inland along the wind is drier (moisture runs out).
- **Why this way:** every pixel is worked out alone, in any order, with no simulation to settle, so it's
  fast, the same every time, and each part is its own slider and overlay.
- The fuller method, carrying moisture along the wind pixel by pixel (picked up over the sea, spent as
  rain), gives the same picture more exactly. It's the upgrade if the simple one looks off.

David: the chicken and the egg of water, rain and lakes above sea level. In the real world it's a loop
(rain makes rivers and lakes; lakes and wet ground make more rain). A game cuts the loop in one place
and runs the rest one way, which is the linear-pipeline rule again. The order:

1. **Ground** (from the stages before; read only).
2. **Temperature**, from "latitude" and height (colder higher up).
3. **Wind**, a fixed direction field, bent round the big mountains. David (2026-10-28) asked whether
   it needs a simulator making swirls out of the hot and cold regions. Counsel [PROPOSED]: no fluid
   simulation needed; make it from temperature, one way:
   - blur the temperature map heavily; the wind blows **from cold toward hot**, along its slope (that's
     what air pressure does);
   - turn it partly sideways (the way the Earth's spin bends winds), so it **swirls round** hot and cold
     spots instead of pointing straight at them;
   - add **curl noise**, a known trick for swirly flow that never piles up, for eddies, with a strength
     setting;
   - weaken the part that runs into tall ground, so wind slides round mountains (and what does climb
     them drops its rain).
   A real fluid simulation (Jos Stam's "stable fluids") would also fit the budget at coarse pixels, but
   it's much harder to steer; keep it as a later option if the cheap one looks too tidy.
4. **Rain**: the wind carries moisture in from the sea and drops it where the ground rises; whatever
   is left after a mountain is the rain shadow behind it. *The loop is cut here:* lakes and rivers add
   no moisture back to the air.
5. **Runoff**: every patch gets its rain as units of water, and the water runs downhill, adding up as it
   goes. That sum (flow) is the rank: creek, stream, river.
6. **Basins**: every dip with no way out has a **spill height**, the lowest point of its rim (the
   priority-flood method finds them all in one pass).
7. **Lakes**: each basin compares what flows in with what its surface would lose to evaporation (hotter
   and drier, more loss).
   - **More in than out:** it fills to its spill height and overflows, a fresh lake with an outlet, and
     the outflow carries on downhill as a river.
   - **Less:** it stands lower, with no outlet. A salt lake, or a dry flat in a desert.
8. **Springs** are simply where the gathered rain first gets big enough to show as water. David's spring
   dots are the stand-in until rain exists (one unit each, see above).

**Lakes above sea level are no puzzle then:** a lake's level belongs to its basin, not the sea. It's the
rim height if the lake is full, or lower if it dries.

**Lake effect** [David, 2026-10-28: wanted, before driving springs to lakes and seas]: the loop's
missing half, lakes adding rain downwind. Two ways, both still one way:
- **Two passes:** run steps 4-7 once without lakes, then run the rain again with the lakes found as
  extra moisture sources, and redo runoff and lakes. It settles fast; a second pass is usually enough.
- **Basins first:** the basins and their spill heights (step 6) come from the ground alone, so where a
  lake *could* be is known before any rain. Rain can count those as moisture sources, weighted by how
  likely each is to fill. One pass, slightly less exact.
Heat comes in through temperature (step 2): warmer air carries more moisture, and warmer lakes lose more
to evaporation.

#### Resources for game-pretty climate  (from Counsel's memory, not checked online)

- **Red Blob Games, mapgen4** (Amit Patel): wind, rain on windward slopes, rain shadows, rivers, all
  interactive with sliders and explained in its blog posts. The closest match. His older *Polygonal Map
  Generation for Games* covers moisture and biomes from distance to water.
- **Here Dragons Abound** (Scott Turner's blog): years of posts building a fantasy map generator (wind,
  rain shadows, rivers, lakes, biomes), always tuned for what looks right over what's correct.
- **Azgaar's Fantasy Map Generator**: free in a browser; temperature from latitude and height, rain from
  wind, rivers, a biome table. Good for quick play.
- **The Whittaker biome diagram**: temperature across, rainfall up, a biome in each patch. A biome core is
  a corner of it pushed to the extreme.
- **Dwarf Fortress world generation**: rainfall, temperature, drainage, rain shadows and savagery as
  fields; biomes from their mix. Tarn Adams has explained it in talks and interviews.
- **Undiscovered Worlds** (Jonathan Hill): a very detailed hobby world generator; how far the
  pseudoscience can go.
- **Artifexian** (YouTube): the basics in plain terms (why winds blow as they do, where deserts fall).
- **Priority-flood** (Barnes and others): the standard fast way to find basins and spill heights.

#### What to remember, software and data  (Counsel, 2026-10-28)

1. **Climate reads the finished ground at map scale** by sampling the Interpreter's own recipe, never a
   copy of it. The Interpreter only draws what Climate chose.
2. **Different things, different resolutions:** wind, temperature and rain are smooth (64-128 blocks a
   pixel will do); rivers are *lines* with flow and bed height along them; springs and creeks get the
   fine ~2-block tile pass, worked out once and saved.
3. **Pits:** decide early which become lakes and which are filled (see the loop above).
4. **Water's hard rules:** always downhill, always connected, reaching the sea or a lake. Everything else
   may cheat.
5. **Separate the calculation from the look:** a rough "true" rain, then a curve David shapes, as with
   the peaks' rise.
6. **What Ecology will read:** temperature, rainfall, ground wetness, distance to water, height, slope,
   biome core strengths. With 4 seasons, store each place's average *and* its seasonal swing.
7. **Latitude** [IDEA, David 2026-10-28]: **ice caps** at the poles and an **equatorial band** (or a line
   with a falloff), with temperature fading between them, and **the lines squiggle**. David tunes the ice
   caps until worlds always get a reasonable amount of ice land, and the equatorial band so there are
   always 1-2 dryland spots. Local temperature variations: unsure, "sounds recursive".
   - **David's answer:** milder by the sea, plus the squiggled ice cap and equator lines with their
     fades, is probably sufficient. (Colder with height is still assumed; say if not.)
   - Counsel: **height cooling isn't recursive.** Temperature reads the ground (colder higher up, maybe
     milder by the sea), and the ground never reads temperature, so it's one way. It would only loop if
     temperature also read lakes or forests, which come after it; that's the cut.
   - Counsel: **the desert picker already guarantees the drylands' deserts.** It picks the 1-2 driest
     spots whatever the climate makes, so the equatorial band only has to make *some* dry land, not
     exactly the right amount.
8. **Climate doesn't change during the 1,000 years of history:** made once, lived in.
9. **Make every field visible:** an overlay and Show switch each, and a readout under the cursor ("rain
   0.3 because: behind a peak, 40 blocks from the sea").
10. **Cost and saves:** within the 10-minute world-creation budget; mountain sliders then need a fast
    make; saved worlds depend on the climate code too (`vision/watch-list.md`, item 20).

#### Caves, cliffs and carving  [IDEA, David 2026-10-29; after forests, a "caves and cliffs" pass]

- **After mineralogy, caves**, inside Climate [DECIDED 2026-10-29, David]: the Climate tab goes
  **Temperature, Wind, Rain, Rivers, Mineralogy, Caves**.
- **Fun first, not science** [David]: "the important thing is they are fun." If rivers and water can
  make fun caves, that's a happy accident, not a requirement.
- **Cliffs carefully.** Cliffs and overhangs could be a sort of aborted cave feature: a cave that broke
  out under a slope gives an overhang, a cave mouth, a rock shelter, an arch.
- **Non-cave carve features here and there**, too.
- **They must be visible from a distance like everything else.** Dwarves will build underground, with
  great hall entrances into mountains.
Counsel's notes [PROPOSED]:
- **One far-3D technique for everything that breaks the surface:** overhangs, arches, cave mouths,
  carved features, dwarven halls, castles. The heightmap stays the base for the far terrain, the
  lighting and Climate. Only the few spots where something breaks the surface get a coarse 3D far copy
  (a shrunk voxel copy at half, quarter or eighth detail), drawn over the far sheet. Natural ones come
  from the recipe, built ones from saved edits. It's the "far builds" idea (`vision/watch-list.md`, item
  17), so it's settled once as a contract, before caves, not three fixes.
- **The light map won't see overhangs' shadows** at first. They can be stamped in later.
- **Fun variety, from Minecraft's Caves & Cliffs** (1.17 blocks and mobs, 1.18 a new generator): huge
  caverns with pillars, long winding tunnels, thin squiggly ones, ravines, underground lakes at their own
  levels (aquifers), ore veins, and cave biomes chosen with depth. Dwarf Fortress sorts depth into cavern
  layers, a magma sea, then the underworld.
- **Carve features, besides caves:** ravines, gorges, sinkholes, sea cliffs and sea caves, rock
  shelters, arches.
- **To decide before release, or before caves, whichever is first:** how deep the world goes
  (Minecraft's painful one: changing world height means converting saved worlds); rock in 3D layers,
  so mineralogy can grow into ore and caves; block light underground (today faces well below the
  surface get plain light).

#### Biomes as cores, and deserts  [IDEA, David 2026-10-28]

- **Biomes aren't regions the world is divided into.** The climate is a large, gradual topography
  (temperature, rain and so on), and a biome is a **special subregion that spawns where the climate is
  most intensely "that"**. Everywhere else is the blend.
- **Fewer biomes than Minecraft**, more like Dwarf Fortress in spirit, but not as broad.
- **Deserts are rare, and placed with care.** Even at this scale David doesn't expect deserts as
  deserty as Minecraft's: **drylands, sure**, but a New Mexico won't blend in super smoothly. Still, there
  must be some desert, because David wants **giant ants and scorpions** there, possibly **led by evil
  druid cults** once those grow bigger.
- **Eroded badlands: probably too much to take on.** Out.
- **A layer that picks the desert** [David]: **one or two per map**, chosen in the Climate stage, that
  **enhances the drylands the way the peaks enhance the ridgelines**: the climate makes drylands, and
  the desert layer pushes the chosen spot further, into real desert.

Counsel's notes [PROPOSED, for David]:
- **Cores fit the gradual climate.** A biome is the peak of a field, the way a peak is a point on the
  crush: the climate is smooth, a core spawns where it goes past a threshold, and the fringe is the climate
  itself fading out. So there's never a hard border to hide, only how sharply the core's edge rises.
- **The Climate stage can give deserts a reason.** A rain shadow behind the big peaks (wind brings rain,
  the mountains take it, the far side stays dry) puts drylands in the lee of mountains, and the driest core
  of those becomes the desert. That's the "placed with care" for free, and it reads as natural because it
  is: the peaks and the dry land explain each other.
- **The desert layer uses the peaks' pattern** (`vision/terrain-pipeline.md`, socket 4, stamps at
  points): a picker chooses the spot from the climate (the driest drylands, say in a rain shadow), and
  a stamp with its own rise curve, ± spreads and an outline wobble deepens the dryness there. The picker
  is whole-map work, so it chooses; the stamp is per area, so it draws. The rule could keep two deserts
  apart and off the coast, like the rules between peaks.
- **Drylands blend; the desert core doesn't have to.** Scrub, then dry grass, then sand gives a gradient
  that hides most of the jump, and a desert that's rare can afford a slightly harder edge, like a
  landmark.
- ~~Giant ants and scorpions are a faction~~ Answered by David below: giant animals are beasts.

**Giant beasts and biome cores** [David, 2026-10-28]:
- **Giant animals are beasts, not evil.** They're animals like any other (a likelihood per area), and
  can **rarely** generate anywhere their animal can (a giant elk wherever elk live).
- **Biome cores are where they're most likely**, and they can **gate rare resources** that draw you into
  the cores.
- **Biome cores are small, distinctive, extreme regions, plotted into the climate the way peaks are
  plotted into the ridgelines.** Each is a unique enhancement to the ecology there: spawn rates, beasts,
  resources, structures and so on. The desert is one of them.
- **Beasts can come with people** [David]: high-level evil druids and rogues can bring beasts with them.
  A wild beast is a likelihood; a beast someone brings belongs to them, like a pet or a follower. (The
  evil druid cults are a faction, and may lead beasts once they grow bigger.)
- **Why there are beasts: animals have become corrupted** [David]. Now and then an overgrown animal
  kills one of your villagers. Beasts are still **not a faction and not evil**: they're a hazard of the
  world, like weather.
- **Evil factions can use them:** raise the beast likelihood around them to raise hell, or keep beasts of
  their own to fight with.
- **Taming, maybe** [IDEA]: with great effort, you might learn to tame a beast.
- **[DECIDED 2026-10-29] The names** (David): a neutrally spawned one is a **beast** ("elk beast"). One
  that comes from an evil faction is a **vile beast** ("vile elk beast"). Vile beasts have their own
  logic, added to or replacing the neutral beastly behaviours, so the name marks a different kind of
  creature, not just a different owner.

### Additives  [DECIDED 2026-10-26, building]

The World Compiler's middle panel: geographic features *added* onto the landmass, before erosion
and hydrology. The broad heading for everything generated rather than read straight off the maps;
mountains first, other kinds of features later, as David needs them.

- **The road** [David, 2026-10-26]:
  1. Throw dots around mountains until it routinely looks right: dot scattering and streaming only,
     nothing drives the ground yet. Expect to try and scrap several methods.
  2. A mountain synthesizer driven by the dots; experiment with it. The aim: pleasant ranges of
     fantasy mountains (taller than they are wide), which mostly run in strings, sometimes bundles.
  3. Rework hills and ground detail in light of what the additives can now add (today they're good
     at times but change the whole map in unintended ways).
  4. Shift work between the compiler (cheap, whole-map) and the additives (expensive synthesis) until
     both sides feel right.
  5. Grow the additives into other features.
- **Ridges as the base, peaks from the dots** [IDEA, David, 2026-10-27]: dialled in, the ridges make
  surprisingly good mountains, scalloped and rough, in the right places, but they don't stand out and
  aren't tall enough for the world's scale (a realistic island 30–40 miles wide). So: keep the ridges
  as the base layer of every range, and use the point-scattering controls to target ranges with steep,
  towering peaks that fade smoothly into the ridge features. His best results still need the ridge
  sliders at extremes. Counsel's notes [PROPOSED]: re-centre the ridge ranges on those values (David's
  rule); ridges *carve into* the mountains, so they can only lower them, and height has to come from
  something that adds; a cheap first try is the Lift's machinery aimed at the mountains (points, reach,
  a falloff spline) *scaling up the ridged mountains* near the points, so the tall peaks keep the
  ridges' scallops instead of being smooth cones set on top; towering peaks will meet the Z 333 cap
  (the build-height item, Next steps).
  - **Today's mountains are the hill country** [David, 2026-10-27]: at the ~400-block scale real
    mountains need, what the Interpreter calls mountains are really **hills**. Their job: a **nest**
    for the mountains to come, and the **ruggedness** that flows with the ridges and fades out into
    the hills and plains. The scalloped ridgelines are worth keeping for their own looks. (A rename
    for later: the "mountains" controls may become the hill country's, and today's "hills" something
    smaller.) Builder is reworking **foothills** now; if it doesn't do something useful, David is
    cutting it. *(Cut, 2f4aafa; passes cut too, ee92cc2.)*
  - **The peaks' technique** [David, 2026-10-27]: Inigo Quilez's "More Noise"
    (iquilezles.org/articles/morenoise): gradient noise gives its slope for free, and each finer
    octave is turned down where the slope so far is steep, so ridges come out sharp and valleys
    smooth, as if eroded. Stamped at the Lift's points, amplified from each centre. David sees it as
    **a combined rise + slope curve**. Counsel's reading [PROPOSED]: two curves, the *rise* (height
    by distance from the centre) and the *slope response* (how much fine detail survives at each
    steepness); if the rise is counted as the first octave, its own slope steers the detail, so
    flanks come out clean and summits and feet craggy.
  - **What the Additives' fields would hold, versus the Interpreter** [PROPOSED, for David]: the
    Additives do whole-map jobs, once: anything that needs neighbours or the whole map. The
    Interpreter does what one block can work out alone, fast and the same answer every time.
  - **David's answers on the method's clashes** [2026-10-27]:
    - *Worn, not towering:* handled with plenty of shaping controls. Long sloping arms can be smaller
      peaks with a narrow double-dip curve; the ridges cut into them. Maybe the noise's dips may also
      carve into the ridges (perhaps unnecessary).
    - *Every peak alike:* each mountain kernel gets its own seed, but **no per-mountain settings**.
      Any difference between peaks is a rule that applies to every mountain ever made: the noise,
      and programmed variation.
    - *No direction:* fine. Ridges give the continental grain; fantasy peaks don't need it. Maybe a
      slight stretch later, only if needed.
    - *Erosion without water:* fine; real water erosion will exist, as another additive.
    - *Scale:* agreed, tune for blocks and the ceiling.
    - *Hidden numbers:* expose them: more controls over the features.
    - Counsel's notes [PROPOSED]: seed each kernel from where it stands (or a stable id), not its
      place in the list, so moving one slider doesn't reshuffle every mountain; programmed
      variation can reuse the Tectonic Sim's ± spread (each kernel its own value within a range);
      decide how overlapping kernels combine (the higher wins, or they add).
  - **[DECIDED 2026-10-27] The peaks plan**, agreed with changes (being built; the full list is in
    `vision/proposal-peaks.md`, "Agreed, with changes"): the Lift dissolves into the Additives'
    **Mountains** heading (its points become the peaks' points, its cloth the bump each peak is cut
    from); the Interpreter's mountains become **Ridgelines**; the ceiling goes first (build height
    1,024, the terrain cut-off its own setting, 333 kept as the ridgelines' measuring stick).
  - **Replaced 2026-10-28:** hydrology now goes in its own **Climate** stage after Mountains (see "The
    stages after Mountains" below), not beside Mountains in the Additives. The 2026-10-27 note, kept
    for its reasoning about river widths:
  - **Hydrology, later** [David, 2026-10-27; to be designed]: a sibling heading to Mountains. The
    combined ground (ridgelines + peaks) is worked out at map scale in the Additives, rivers found
    there, and the Interpreter gets a carve map applied last. David's worry: 32-block pixels giving
    1-block trickles. Builder's answer: the map says where rivers run and how much flows; width, depth
    and banks are shaped per block, as the coast is, sized to their flow.
  - **A hill field, maybe** [IDEA, David, 2026-10-27; wait to see how the peaks go]: small, smooth
    bumps kept to the wide-open land, shown by two falloffs multiplied, distance from the coast (off
    at the shore) and distance from the nearest peak (fading toward it at its own slope). Counsel's
    note: masks would need to stack several sources, each with its own curve; fading hills to
    nothing at the shore also stops them moving the coastline. David's leaning: rather than a
    separate layer, **extend the peaks' own concept, with much less noise, growing the addition out
    toward the plains** so they aren't so flat and open. Forests (ecology, after the land and rivers)
    will carry much of this too, and complement hills and mountains.
  - **Later stages** [David, 2026-10-27]: **mineralogy adds or removes no blocks** (it only decides
    what they're made of); **ecology comes after the whole world engine is done**.
- **Mountains as a field, not lines** [2026-10-26]. A first try seeded dots on the "range spines"
  (one-pixel crest lines) and streamed them off sideways: wrong tool; the spine detector finds
  straight lines through gentle hills, spurs down curving slopes, and has no answer for a bundle.
  Instead, every mountain spot is measured: *how mountainous* (the share), *which way the range
  runs* (its grain), *how string-like or knot-like* (0 to 1), and later *where across the range*
  (core, flank, edge) and *local prominence* (standing above its own surroundings, for summits).
  Dots blend their behaviour by string-or-knot, so odd formations get a gentler treatment rather
  than a wrong one.
- **The grain** (BUILT): a structure tensor over the mountain share (round each spot, do the
  slopes face across one line or every way?) gives the direction and string-or-knot; the squeeze
  gives a second opinion (ranges run across the push, as fold belts do). "From squeeze" blends the
  directions; "grain reach" sets how big a patch decides. Overlays: *range grain* (strokes) and
  *string or knot* (blue / yellow).
- **David's picture of the dots** (For Claude/visionofparticlestreaming.png): seeds along a range,
  dots leaving on one side and carried onward the way the range is heading, scattered (not in rows)
  and thinning out with distance.
- **Grain drift** (BUILT 2026-10-26): seeds (blue) scattered over the mountains, thicker where
  they're stronger, between *least* and *most mountains* (both ends trimmed independently). Dots
  (yellow, fading with distance) follow the grain step by step, spread sideways (*spread*, *lean*)
  and thin out; in knots they scatter round the seed instead, blended by string-or-knot. Distance
  goes to 100,000 blocks. The ridge streams are kept (off by default) for later use.
- **Ranges and their tips** (BUILT): the mountains are cut into separate ranges where weaker than
  the *range cut*; each has a middle (white ring) and a tip (white arrow). David's idea: a carrot's
  middle sits near its fat end, but more of it reaches out the thin end; measured by the skew along
  the range's main axis, so an egg or sausage gets no tip. Dots drift toward their own range's tip
  (or to a bearing, or both ways); *tip pull* steers them that way, so they stream on out past it.
- **Coherence** (BUILT): draws each range's dots in toward its middle line (ring to arrow), over
  about one range length, and turns down spread and knot scatter: one tight stream at 1.
- **Spiral** (BUILT): every dot turns to swirl round the map's middle (60 degrees off straight in),
  harder the further it has gone; *spiral exponent* shapes how the turning grows (David picked the
  map's middle, and distance travelled).
- **The dots are a viewing aid** for now: a 1x / 2x / 4x / 8x size switch, not saved.

### Lift  [DECIDED and BUILT 2026-10-26]
**Moved 2026-10-27** (step 2 of the peaks plan, `vision/proposal-peaks.md`): the Lift is no longer an Interpreter section. Its controls are on the Additives panel under one **Mountains** heading (Hydrology to come beside it): **Where** (the point picker) and **Rise** (the cloth; "max lift" is now "rise"), above Mountain grain, Grain drift and Ridge streams. The settings are still kept in the recipe (`lift`), since the cloth is worked out at every block, so presets and the ground are unchanged. The Interpreter's "Mountains (from the squeeze map)" is now **Ridgelines** (David: the ridgelines raise the land to the highlands; the Additives' mountains put peaks across the same area). The Show switches and overlay say "rise" and "ridgelines".
David: "a tablecloth draped over a series of points floating in the area, with the edges of the
cloth tapering and ending just before the coast lines". Exposed land is raised toward mountain
points: lift = *max lift* x a spline (nearness to the nearest point, over a *reach*; left the coast
side, right the peak) x a *coast pad* brake (from the coast distance, with a width and falloff, so
the coastline never moves). Very fine controls, so the edge can creep toward the coast until it
can't be seen. A switch picks the points: seed-like points or range middles. They're their own set
(not the scatterer's dots), found from the bake's own crush rather than the mountain curve, with
their own least / most / spacing / range cut. Controls in the World Interpreter panel. May tie in
later with reworking hills and ground detail.

### Peaks  [BUILT 2026-10-27: the rise alone; crags, slope response and variation next]
Step 3-4 of the peaks plan (`vision/proposal-peaks.md`). **Additives, Mountains, Peaks** (a switched
section, off by default): a peak stands at each of the mountains' points (Where), on top of the
ground, the rise and the ridgelines. Its shape is a **height** (blocks above the ground under it), a
**width** (how far its foot reaches from its point) and a **rise** spline (from the foot, left, to the
top, right). Where peaks overlap, the higher wins. Braked by the Rise's coast pad, and none at sea, so
the coast never moves. Past Z 333 only with **ground never above** raised (World Interpreter, Land
and sea). A "peaks" Show switch leaves them out (and so does a world spawned with it off).
- **The points travel with the world:** the compiler keeps them (`mountain_points`, blocks from the
  map's north-west corner), worlds save them in `compiled.toml`, and Render World and Spawn hands them
  over with the maps. Worlds saved before have none, so no peaks.
- **Finding the peaks near a block:** the points are sorted into square cells at least a peak's width
  across (at most 256 along a side), so a block looks only at its own cell and the eight round it.
  Exact: no seams where neighbouring peaks meet (the Lift's nearest-point map can't blend two).
- Above Z 400 the near light window's stored height stops climbing; only its sea-surface check reads
  it, so tall peaks shade as other ground does (the height tint keeps brightening with height).
- **Crags and variety [BUILT 2026-10-27]** (David, after the first peaks: "if we can make them
  irregularly fatter, and then crag them up real good so that they sort of make smaller mountains
  around them where they blend with one another and the ridges"):
  - **Each peak its own:** height and width each with a ± spread (the Tectonic Sim's doubling scale),
    from a seed taken from where the peak stands; an **outline wobble** (a slow noise over the peak's
    own ground, as a share of its width) so footprints aren't circles; **where peaks overlap**: higher
    wins, or **add up** (massifs).
  - **Crags**, Inigo Quilez's "More Noise": layers of noise, each finer and weaker (how many, how much
    finer, how much weaker), each kept as much as the **slope response** spline says for how steep
    the peak is there so far. The rise counts as the first layer, so its own steepness steers the
    rest. The slope comes free with the noise (`perlin_with_slope`, checked against measured slopes).
    The response replaces the article's fixed 1 / (1 + slope²); **steepest** sets what its right end
    means. Crags are a share of the peak's height, faded in from its foot; **crags carve below the
    peaks** lets their dips cut into the ridgelines and rise, never under the sea. A "crags" Show
    switch.
  - **Ground never above** now starts at the world's top (Z 1,023): peaks stand on the ridgelines and
    rise, so their tops differ, and a lower cut-off flattened the tallest (David had to set 720).
  - **Cost:** each peak's numbers are worked out once per map, and a block skips peaks out of reach
    before any noise; with 8 crag layers among 40 overlapping peaks, about 1.5 ms per chunk of
    columns (0.25 without peaks).
- **Wobble spread, by-layer curves and the feather [BUILT 2026-10-28]** (David, maxing the wobble,
  "finer" at its minimum and "as strong" at its maximum): the outline wobble has a ± spread (each
  peak its own) and goes to 2 (past 1 the pinches split a foot into lobes). "Each layer finer by"
  goes down to ×1.1 and "as strong" up to 1, and each has a curve over the steps from the first
  layer to the last (how much of the slider each step takes). The **feather**: past a peak's foot
  its crags carry on as hills (feather height, a share of the peak's height) for a distance, fading
  along their own curve, to see how much hilliness the mountains alone can give.
- **Hills from the ranges and the peaks' own coast fade [BUILT 2026-10-28]** (David: the crags
  feathered out from the mountains "form convincing and tasteful hills"; keep their look, but run
  them off the mountain ranges and fade them from the coasts, a layer of detail over more of the
  world than the ranges). The same crags, with room out from all the mountains' points at once
  (range hills height, a share of the peaks' height; a reach and a fade curve), so they run along
  the ranges and soften into the plains; where a peak or its feather has more room, that wins.
  The peaks used to borrow the Rise's coast pad, so the Rise's settings changed them even with
  "rise" off in Show; they have their own coast fade now (distance and falloff, as the pad's).
  David's default preset was given his pad's numbers, so nothing he had changed. **Next idea:**
  summit crags (a sharp-ridged layer working from the tops down, for the king peaks).
- **Hills cut [2026-10-28]** (David: "none of the controls that work on something like 'hills' do
  a single thing right ... slash hills entirely", without changing the mountains). Gone: the
  Interpreter's Hills (its Perlin field, mask and Show switch), and the peaks' feather and hills from
  the ranges. Where they touched the mountains: the feather and range hills kept the crags strong
  down to the peaks' feet (the crags took the strongest of the three there); that is now the peaks'
  own **foot fade** (how far up from the foot the crags come to full strength, a share of the
  width; it was fixed at 0.2). David's preset was given 0.1, close to what he had. The Interpreter's
  hills only reached about 250 blocks in from the coast in his preset (under half a block inland),
  so cutting them can at most move a mountain column by a block. The peaks' own coast fade stays.
- **Ground detail is Hills now [2026-10-28]** (David: "ground detail is great. its basically what
  i wanted from hills"): the section, its Show switch and hover text say hills (up to 40 blocks,
  sizes to 2,000); the settings keep their names in presets (`detail`, `detail_noise`, ...).
- **Summit crags [BUILT 2026-10-28]** (David: "one more pass of jagginess onto the peaks ... pressing
  noise into the peaks, by effecting everything above X elevation", "a second, more focused pass of
  the first level of crags"): a Perlin field (a Graphics Kit Perlin layer's controls) pressed into
  all the ground on land high up, fading in from a start Z to full at a full Z along a curve,
  braked by the peaks' coast fade. In Additives, Mountains, Peaks, with a "summit crags" Show switch.
  After it: hydrology.
- **Climate, step 1 [BUILT 2026-10-28]** (the plan: "Breaking the water loop"; David placed it as a
  panel in the World Compiler between Additives and the World Interpreter, with Temperature, Wind,
  Rain and Rivers sections). The panel, with the stages still to come greyed in order, and **the ground
  it reads**: the Interpreter's own recipe (no copy), asked for the height at the middle of each
  climate pixel with its fine layers left out (crags, summit crags, hills, torn coast, using the Show
  switches' mutes): the broad shape of the land. A climate pixel is a whole number of the map's
  32-block pixels (4 = 128 blocks, a slider). Read in the background on all cores whenever the recipe
  changes; "show the ground it reads" draws it over the map, and the readouts give its height.
- **Climate, step 2: Temperature [BUILT 2026-10-28]** (David: "a sort of natural flow to the temp
  gradient that isn't as simple as 'warmer in the center, colder north'"; "hundreds of strings lined
  up to go between the 3 squiggles", never crossing). Three lines west to east (north ice line,
  equator with an evenly hot band, south ice line), each squiggled by its own seed mixed with the
  world's (where, squiggle, wave length). Past the ice lines evenly cold (each cap its own value);
  between, a **soap film** (Laplace's equation, settled coarse to fine): its contours are the bands
  and its slope lines the strings, so near a line the bands follow its wiggles and further out small
  wiggles die away while big waves carry through. Lines wave without folding back (hooks not wanted).
  Then the fade spline, **milder by the sea** (pulled toward the world's average, with a reach
  inland), and **colder higher up** (per 100 blocks). Scale −1 frozen to +1 hottest. Show over the map:
  nothing / the ground it reads / temperature (blue to red, sea dimmed, the lines white); shares of the
  land frozen and hottest, the world average, and a readout under the mouse. Its own background job
  after the ground, so its sliders don't read the ground again.
  [2026-10-28: the two ice lines share one set of controls (distance from their own edge, squiggle,
  wave length; each its own seed) and one cold value, David: "they're equal".]
- **Climate, step 3: Wind [BUILT 2026-10-28]** (no fluid simulation, as planned): from the
  temperature **at sea level** (Builder's fix: the height-cooled temperature would make every range a
  cold spot air blows away from, so no rain shadows), blurred (a setting), blowing from cold toward
  hot along its slope, **turned sideways** (a setting, degrees) to the right north of the equator and
  to the left south of it, so the halves mirror like trade winds; **even strength** blends from the
  slope's own strength (calm where it's even) to the same everywhere. Mountains don't bend it (rain
  will read them). Show over the map: arrows on the ground it reads; the readout names the wind by
  where it comes from. Climate's stages hold what they were made from, so a new recipe or result is
  never mistaken for the old one.
- **Climate, step 4: Rain [BUILT 2026-10-28]** (the plan's simplest way): each climate pixel walks
  back against the wind, following its bends, for a set distance over the broad ground, and three
  parts come out of the one walk, each with its own settings and its own view: **rain shadow** (how
  much higher the ground upwind stands, nearer counting more, through a curve), **facing the wind**
  (the ground rising toward here over a set distance upwind: wetter), **far from the sea** (how far
  back along the wind the sea is: drier). Plus **warm air carries more** (the cold side drier) and a
  **look** curve, so the rough sum and its look stay apart. Show over the map: rain (dry tan to wet
  blue) or one part in grey; the readout gives the rain and each part's say ("because: shadow 40%,
  facing the wind 0%, far from the sea 20%"). Every pixel alone, any order, the same every time.
- **Climate, step 5: the flow [BUILT 2026-10-28]** (Rivers section): the broad ground read again on
  the map's own grid (32 blocks), then **priority-flood**: from the sea and the world's edge, the
  lowest ground not yet reached is taken next and whatever it reaches drains into it, so every dip
  fills to its rim's lowest point (a lake with a flat surface and an outlet) and spills on; every patch
  drains, always downhill or across a lake, to the sea. Ties on flat ground are broken by a fixed nudge
  from the world's seed (wandering paths, no grid lines, the same every time). Each patch gives its
  rain as units (1 = a patch of the wettest rain), plus **snowmelt** (cold and high: a share extra)
  and **springs at slope feet** (where the slope eases off), and the flow adds up downhill, upstream
  first. Lakes: under the surface deeper than a setting; fresh only (fill and spill). Show over the
  map: water, brighter and bluer the more it carries, from a "drawn from" amount; lakes filled; the
  readout gives the flow and any lake's depth and surface. About 1.1 s for a full 2,250-pixel map, in
  the background. Next: river lines and ranks.

### Render World and Spawn  [DECIDED 2026-10-24, building]
David: the maps are as wholesome as they can get from above; what's missing can only be judged by
flying over the continents in person. So before the compiler grows more stages, build the path from a
compiled world to a walkable one. (Grok's plan, agreed: minimal elevation → blocks first; walk it; then
expand the compiler into Landmass → Climate & Drainage → Geology → Ecology, then Civilization. Caves are
a separate later system.)
- **Distant terrain like Distant Horizons** [David: "do it the right way"]: blocky LOD rings (2×, 4×,
  8×… coarser further out) out to the whole world, made by sampling the same column function directly
  (no full chunks needed far away). Soaring over the entire world in moments must stay smooth.
  **The hand-off** (2026-10-24): distant terrain steps aside only over real chunks actually on screen
  (a small texture marks them), never a fixed circle, so outflying the chunk loading shows distant
  terrain, not a hole [David: "little things do matter"]. It runs one chunk under the edge of the real
  chunks so no sky shows through the seam.
- **Chunks generated on the fly**, in the background, as fast as possible without architectural
  compromises. Only changed chunks are saved (as now).
- **Performance target** [David]: hold today's 240–250 FPS on his RTX 5070 Ti laptop, as a proxy for
  "playable on a GTX 980 Ti at 1080p on low settings". Potato PCs matter.
- **Old saves don't matter** [David]: changing the generator may change old worlds; nothing to keep yet
  but presets and movement settings.
- **Interpreter settings** (detail noise and so on) live in a section at the end of the World
  Compiler, next to the button, and are copied into each world.
- **Blocks, for now** [David]: only the shape of the land matters. Gabbro on the highest peaks, stone on
  moderate mountain heights, grass for the rest, plain **water** blocks (opaque blue, still) filling the
  oceans to Z 128. Bedrock, dirt, sand, gravel come with the climate and geology stages.
- **Spawn in the middle of the world.** A **reworked Camera debug page** for dialling in observer
  flights (speed up to thousands of blocks a second, above the world's top).
- **The test cylinder stays** for experiments with specific systems, with the test settlement and
  villagers. Generated worlds have no settlement for now.
- **Step 1 [BUILT 2026-10-24]**: 797 chunks around the middle in 45 ms, 244 FPS.
- **Streaming decisions** [David, 2026-10-24]: at least **32 chunks of full-detail gameplay** around the
  player (a city seen in the iso camera may span that); shoot high, dial back when a problem shows.
  Every trick in the book, but keep the code understandable. Chunks left behind are dropped (changed
  ones kept and saved). Nearest and **ahead first**: anything that can wait a second or two, waits, so
  work spreads out instead of spiking. **240 FPS isn't holy**: build a way to see when we run out of
  rope (a performance budget readout), so the art and design can be planned around a real limit;
  for now, just see how it runs. Observer flight **keeps collision**.
- **Order:** (1) the column generator, world from a compiled bake, the button, spawn; (2) streaming
  chunks around you in the background; (3) LOD rings and the observer camera; (4) walk and fly it, then
  decide what the compiler needs.
- **Principle** [David, 2026-10-24]: the terrain can't truly be judged until a world can be explored with
  trees and water. So get as close as we can now, and **leave room to adjust the back end** once
  worlds can be walked. Judge settings **across many seeds** (Shift+Space), not on one.
- **Principle: controls do what they say** [David, 2026-10-27]: the editor's controls should do what
  they indicate, and the suite shouldn't rely on unintended consequences to get its results. A system
  that gets good results by accident is missing the parts that would get the most out of what it does
  well. Example: David has been using the hill and ground-detail noise to set how far inland the coast
  runs and how irregular it is (rippled coasts full of shoals and islets, lowlands full of ponds), and a
  small move turns a continent into mountains in an ocean. **Many influences are wanted** [David]: several
  systems shaping the same thing (like the coast) is what makes it morph across circumstances: cliffs in
  places, simple coasts like Africa's west coast in others, shoals, shallows and islets elsewhere. So the
  fix isn't one owner per feature: it's to home in on what each control actually does, rename it, and
  refine it for the purpose it turned out to be handy for, so no factor is useless, mysterious or too
  unpredictable. Good tunings already give fairly repeatable results. **Order controls from broad to
  fine** [IDEA, David]: transformations over a continental area with small changes at the top; as
  influences get smaller in scale and more locally variable, further down. Then tuning can work from top
  to bottom (draft order: `vision/terrain-controls.md`). **The goal for 2026-10-27:** clarify and rework
  the World Interpreter (the part that reads the compiled results). **Additives wait** [David,
  2026-10-27]: the Tectonic Sim and the Interpreter come first, until they cruise, routinely making
  worlds that look natural and memorable, with enough ridges and plains in a pleasant arrangement that
  mountains being almost imperceptible on foot doesn't bother him. Only then does additive terrain
  synthesis begin. He has already made a handful that look as good as he expects; the work is making
  that routine. **Status, end of 2026-10-27** [David]: about **3 in 4** worlds give the desired results
  without touching anything, and the outliers aren't terrible. The ocean plates shaping the outline
  while the land plates crush enough for ridges (without balling the continent up) was handled by
  giving land and ocean plates their own Tectonic Sim settings.
- **Plates stick [BUILT 2026-10-24]** [David: sea got caught between colliding plates, making canyons
  through every range]: points pressed together **weld** (stickiness slider; welds crack if pulled
  hard, so straits can still open), and sea narrower than **seal gaps up to** with rock on both sides
  is filled from the nearest rock. Bakes made before this still have their gaps: bake again. For now worlds are baked one at a time to experiment on; **eventually**
  [David] the Tectonic Sim runs a world preset and hands its maps straight to the World Compiler, and
  concludes itself.
- **The repeating coastal mounds: found and fixed at the root** [David: "over and over, I see extremely
  regular borders of mountains and coastlines... a solid series of the same bump"]. First guess (an even
  grid of even rock buckling at one spacing) was wrong: a grid shaker and uneven rock were built, did
  "little or nothing" [David], and were removed. Found by elimination on David's own world: the bake and
  the compiled map (32 blocks a pixel) are smooth; the mounds are one per compiler pixel. The compiler
  applied its sharp curves (a coast that drops ~90 blocks within a pixel, steep mountain curves) to each
  pixel, and the interpreter then curved between pixels (Catmull-Rom), swinging past every step: one hump
  per pixel along a coastline that is itself a pixel staircase (a plate outline on a grid). Fix (see "The
  interpreter" below): blend smooth inputs first and apply the curves at every block, with the coastline
  softened slightly so the staircase becomes a curve.
- **The crust has no holes [BUILT 2026-10-24]** [David: deep-ocean "bubbles" drifting round collision
  zones]. Measured on midstart over 6 seeds: 77–118 enclosed pockets of nothing per world, the same
  with tearing off, so not rifts: each point keeps other plates' points at arm's length, and where
  three plates meet (or a corner jams into a notch) the points can't pack the space. Seal gaps filled
  the thin ones and rounded the rest into octagons. Rule: any hole with rock all round it (not joined
  to the open sea) no bigger than *fill holes up to* (default 10,000 blocks across) is filled from the
  nearest rock. After it, 0 left on all 6 seeds.

- **Idea to investigate later** [David, 2026-10-24], once more of tectonics is built: instead of one cell
  per plate, use a **finer grid of cells that cluster together into plates**. That gives organic blob shapes
  even before the line noise, and lets the ocean plates bordering the land be smaller. The early
  subdivisions could be kept and reused later if they're useful (for example for regions).

- **After tectonics, one tool at a time:**
  - a water level;
  - springs and rivers: a field of random dots culled to plausible springs, shown as light blue pixels, that
    run downhill and thicken as tributaries join, out to the ocean.

**Open question** [David]: start from Perlin noise (continents cut out with a radial mask), or simulate plates
before history starts? Having both lets them be compared, or combined (plates for the big shapes, noise for
detail). A **mask layer** (radial, linear, ring, with a falloff curve) is wanted either way: for example, lowering
land toward the edge so the continents are surrounded by ocean.

### Graphics: the light map  [DECIDED 2026-10-26; survey view, steps C and D BUILT]
David: the limiting factor in tuning the terrain is being able to *see* it at great distances. The
distant terrain already reads better than real chunks (its hill shading lights each patch by its
slope; chunks only by which way each face points), so he flies at render distance 1.
- **The goal for graphics** [David]: layered, as well-made games do, each method used where it's most
  effective so players never see behind the curtain. The **world is not snapped to the 256-colour palette**: that
  256-colour palette is for the UI and HUD (rendered at a lower resolution). The world uses a carefully
  chosen selection of colours, so tones and kinds of colour can carry information, and the final
  lighting passes must be able to lighten and darken them (whether the finished frame is then held to
  the world's ~600 colours is open until the art-refinement phase, see `docs/decisions.md` Look & UI and
  watch list item 15: the light map only multiplies brightness, so either answer stays possible). FX (magic and so on) may use every colour, to dazzle
  or to add another meaningful layer. Atmosphere and fog matter little (fog is for looks, e.g. mountains
  fading along a coast). *Changed 2026-10-28:* fog is now wanted, about a couple of miles of visibility
  (`docs/decisions.md`, Look & UI, "Sky, air and weather").
- **For now** [David]: only the first tasks, ones that won't cause trouble if we change our minds. The
  cheap methods first, left open for more sophisticated ones later.
- **Survey view** [BUILT]: Render page. Every Nth level of blocks above sea level darkened (default
  every 4), near and far, so the land reads in bands like a contour map. The first band is the first
  level above the sea, so the coast gets one.
- **The light map** [BUILT, step C] (`src/render/light.rs`): the ground is a height field, so its light
  is worked out in 2D, in the background, as a picture over the whole world (32 blocks a pixel by
  default), and the distant shader multiplies its colour by it. A stage after the ground: it reads the
  compiled map's heights and feeds nothing back. Three ingredients, each with a strength (applied in
  the shader, free to move) and, where it has one, a reach (worked out again):
  - **shadows**: from each spot, look toward the sun for ground rising above the line to it (a march
    that stops once the line clears the highest ground), with a soft edge. Moving the sun redoes only
    this part.
  - **valley darkening** (sky-view factor): how much sky a spot can't see, from the horizon's height in
    8 directions out to a reach. Doesn't depend on the sun, so it's worked out once per world.
  - **relief**: height minus a blurred copy (two box blurs), darkening what sits below its
    surroundings and lightening what stands above.
  - The sea takes only the shadows. **One sun** (Render page, Light): the distant hill shading uses it
    too.
- **D. Near chunks lit the same way** [BUILT]: the near shader reads the world light map (shadows,
  valleys, relief) and a window of the ground around you (*near detail* blocks a pixel, as wide as the
  render distance, made again in the background as you move) holding each spot's slope and height.
  Tops get the distant terrain's hill shading from that slope (same sun, same strength, same sums,
  slopes measured over the distant terrain's finest cell), so near and far match at the hand-off.
  Faces more than about 12 blocks under the ground (caves, overhangs) keep their plain light until
  real block light. Shadows near you are only as sharp as the light map (32 blocks a pixel); sharper
  ones nearby are a later layer (near shadow maps, below).
- **Steps: the cone map** [BUILT 2026-10-27, David: "something that we would want to deliberately build to
  use pretty generally ... at a lower cost"]. Hill shading read from the smooth ground spills across a
  one-block step (anything blended does), so a ledge's top darkened with its foot. Now the near slope is
  only the *broad lie of the land* (the smooth slope averaged over 32 blocks), and the steps you can see
  come from a **cone map**: every block top casts a cone down round itself (its height, less a fall per
  block of distance), and each pixel holds the highest cone over it. The shader compares each block top's
  exact height with it: at or above, nothing near stands higher (a top, a rim, open ground), so it's never
  darkened; below, it's a foot, darkest at the face and fading out, further for a taller step. Two maps:
  all round (*foot shading*) and only from the sun's side (*step shadows*); *shade reach* is blocks of
  shade per block of height. Cost: two sweeps when the near window is made, one read per pixel a frame.
  **Meant to be reused**: it answers "how much higher is the ground near me, and how close?", so the same
  map can drive grass or snow on rims and tops, dirt and moss at feet, damp at cliff bases, shore wash,
  and (with the cone's fall set by the sun's height) cheap true cast shadows from terrain near you.
- **Left open, in order** (each a layer on top, nothing to undo):
  - Hill shading moved from the distant tiles' corners into the light map (so moving the sun doesn't
    rebuild tiles, and in shadow, slopes facing the sun stop being lit).
  - Corner darkening on near blocks (classic voxel ambient occlusion, worked out while meshing).
  - Bevy's shadow maps, near you only, for buildings, trees and people.
  - Block light (torches, the Light property) and the day/night cycle (which only turns the sun).
  - Rejected for landscape scale: screen-space ambient occlusion (reach tied to the screen, so far land
    gets none) and per-vertex light in the meshes (breaks the merged rectangles).

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

**[DECIDED 2026-10-28] The road to alpha-wrapped worlds** (David). Climate steps 1-5 are built; before
rivers, in this order:
1. **A graphics run:** assess what can safely be done now, without making more work later, that gets us
   closer to our own "secret recipe" graphics goal (great performance and looks; `vision/watch-list.md`,
   item 17). Probe for easy wins: colour space, crisp pixels and mipmaps (item 15d), what brightness
   means (item 15e), the peaks' shading mismatch, sun and shade balance, a test stand.
   **Builder's assessment (2026-10-28):** (a) colour space is already right (sRGB textures, linear
   lighting); (b) mipmaps blend colours: palette-preserving mips plus a nearest mip filter, behind a
   switch, small; (c) brightness: A free darkening (today), B painted ramps, C both, for David's colour
   session, and whichever he picks, **slot textures** (each pixel a shade slot, each material a palette
   row, looked up in the shader) are the next structural step, needed for one texture many tints and for
   water's palette cycling; (d) the peaks' shading mismatch is likely the height tint's top (400 near, 333
   far, ~6% step at the hand-off): one shared top, small; (e) sun colour and shade colour with strengths,
   the same near and far, switchable; (f) a test stand as a stamp toggled from the Sandbox, later the
   library's preview. Suggested order: d, b, f, e, decide c at the colour session, then slot textures,
   which unlock the block library.
   **David's call (2026-10-28):** first the smooth far surface (no ziggurats), global lighting and
   shading from the recipe (d is part of it), and a performance tune-up. Small fixes come as David
   points them out. Mipmaps (b) are worth looking into (some mip work was done early, around day 1).
   **David makes the test stand (f) himself** once the block register suite exists. Sun and shade
   colours (e) and the sky and fog wait for his word.
2. **A block register and library tool** in the editor, so adding blocks is a breeze, with Counsel's
   suggestions (section 0, "An asset library page").
3. **A rudimentary what-block-goes-where system**, built to grow cleanly into mineralogy and soil
   grading, enough to show a few basic climates (snow, sand, dry grass, gravel, mud and so on).
4. **Then rivers** (Climate step 6 on), which all of the above makes easier.
After that, David: "it's downhill towards created worlds being alpha wrapped."

**[DECIDED 2026-10-29] Where the road stands** (David). The graphics run is done (the near/far seam
closed at 2cd2335), and the Registrar's first phase is built (David took the palette from 490 colours
to 58, every block regulated). Next:
1. **The World Compiler rename** (from Terrain Compiler), in the gap between features. [DONE 2026-10-29]
2. **Load a new asset**, with the 5×5 tiling checker and GPL export.
3. **David makes blocks** for a while (D's art), *before* the what-block-goes-where system is built.
   *Counsel's starter set* [PROPOSED]: the fewest blocks that make each climate readable from the air.
   Grass, dry grass, **dirt** (early: a few layers between grass and stone change every hillside up
   close, where step risers now show stone straight under the grass), mud, sand, gravel, snowy ground;
   3-4 rock types from one stone pattern on different splines (gabbro, plus granite light, basalt dark,
   limestone or sandstone warm); ice and packed snow; maybe wet sand for shores. Later: clay, peat, moss,
   red sand, cracked earth, scree, and snow layers 1-8 (water's levels system).
   **Crossed-sprite grass** comes with ecology (after rain, after rivers), as a "decoration" kind of
   block: not solid, replaceable, drawn as a cross, many drawn at once, fading out with distance.
4. Then what block goes where, climate set up to work on rivers, and the rivers finished. Then it's time
   for trees.
   **Before rivers: rough in a mineralogy section** [David 2026-10-29], somewhere to decide how the new
   blocks get placed, and **retire the old ways they're placed**. Today that's `surface()` in
   `src/world/terrain.rs`: grass below a height, stone from "stone from", gabbro from "gabbro from",
   stone under every surface. **It stays in Climate, after Rivers** [David], so nothing sits between
   Climate and Ecology. *Counsel:* that fits the linear pipeline. It reads temperature, rain and the rivers
   above it (mud and gravel along rivers, sand on dry ground, snow where it's cold). It's roughed in
   before the rivers are finished, and reads their results once they exist. Its name is David's call.
   **[ROUGH FIRST VERSION BUILT 2026-10-29]** (`src/world/mineralogy.rs`; the Climate tab's Mineralogy
   section, after Rivers, with Caves greyed after it). `surface()` is retired. The climate is handed to
   the world at "Render World and Spawn": temperature (as at sea level) and rain join the world's saved
   maps (`map-temperature.png`, `map-rain.png`), and each block cools for its own height. The surface,
   first rule that applies: snowy ground where cold enough (peaks too); gabbro and stone by height (the
   old sliders, moved here); sand on beaches and in hot, dry deserts; dry grass where dry; grass. Under
   grass, dry grass and snow, dirt; under sand, sand; then stone. The sea floor: sand in the shallows,
   gravel deeper. The zones' edges wander a little. **A rule whose block isn't registered is skipped**
   (it uses `incus:snowy_ground`, `incus:sand`, `incus:dry_grass`, `incus:dirt`, `incus:gravel`), so each
   block lands as David registers it; worlds from before climate stay mild and grassy. The distant
   terrain colours every block (each block's own far colour), so near and far agree whatever is placed.
**Reminder: fix the collision bug (walking through block sides) before the asset factory world is added
to the Registrar**, which comes when it's time for trees. **[FIXED 2026-10-29]** It came from the
position's precision: from 2,048 blocks out a position only changes in steps of 0.0002 blocks or more,
larger than the collision's tiny gap, so a body stopped flush against a wall could land a hair inside it
and the next step's sweep started past the wall. The gap now grows with distance
(`src/player/collision.rs`, `margin`); a test walks into walls out to the world's edge.

Earlier (2026-10-27): David is exploring and improving the terrain gen again. **The Interpreter rework** [David,
2026-10-27], as it goes:
- **Eliminate or repurpose controls that mean nothing to David.** Done so far (2026-10-27): rifts
  taken out of the Interpreter ("seem to do nothing except make a couple ocean spots a little
  deeper"; old presets' rift depth is ignored), and the Tectonic Sim's cracking switched off.
- **Rebalance a preset's slider ranges** once the reworks settle: go through a preset David likes and
  move each slider's range so the preset sits near the middle. Several controls now have a sweet spot
  only 1 to 3 mouse-wheel steps wide on the whole bar.
- **The climb to the mountains:** David wants scalloped ramps from the lowlands up to the peaks, since
  a giant mountain is hard to see over a few-block grade right in front of you. Next after the coast.

**Idea for later: a rejector layer** [David, 2026-10-27]: run the tectonics a few times, quickly, for
a few hundred steps each (the Tectonic Sim's run length), and check each run against rules for whether
it should be rejected, before baking. The run length (built 2026-10-27) is the first piece.

**A world is a frozen copy, by design** [checked 2026-10-27, David asked]: changing the editor and
pressing F2 returns to the same world unchanged. Render World and Spawn hands the world its own copy
of the compiled maps and the recipe's settings (saved in its folder), and only that makes a new one.
It has to: saves keep only changed chunks, and the rest is made again from the map, so a world that
followed the editor would reshape around its builds and seam where new ground meets old. Two things
for later:
- **The recipe's code isn't frozen with it.** An old world reads its saved settings with today's
  Interpreter, so after the code changes (as the foothills did, 2026-10-27) its untouched ground
  comes out differently, and newly streamed ground can seam against old changed chunks. Fine for
  throwaway test worlds; before players keep worlds across updates, either version the recipe (an old
  world makes ground the way it was made with) or save every chunk once made.
- **Quick compare** [David: "sometimes, i do want to set my camera up somewhere, make a change, and
  pop back immediately to compare"]: a "respawn here with changes" that rebuilds the world from the
  current settings where you stand, facing the same way. Not needed yet.

**David's notes, 2026-10-27, for after the peaks:**
- **Torn coast** works well and stays.
- **Hills and ground detail:** not yet decided whether they stay (hills are off in his presets).
- **Grain drift and ridge stream dots** look like useful tools without a use yet. There may be a
  clever way to bring them in once the mountains are sorted.
- **Peaks: "we hit it on the head"** (2026-10-27). Add up looks better than higher wins (now the
  default); "each layer finer by" goes to ×6 (he got good results maxed out at ×3).
- **Far mountains read as ziggurats** [David, 2026-10-27]: the distant terrain's blocky LOD rings
  turn tall peaks into stepped pyramids. Needs adaptive / smoother far meshes at some point, and "our
  performance budget has to last": a long way to go. Flying extremely fast stutters every 2-3 s (he
  finds it reasonable; the stutter breakdown is still on the list).

**Soon after, on the to-do list [David 2026-10-26]:**

- **Raise the build height, and free the terrain from Z 333.** [BUILT 2026-10-27, first step of the peaks plan, `vision/proposal-peaks.md`: the world is 1,024 tall (32 sections; chunk format 2 writes its section count, format 1 loads as 13 topped up with air); the ground's cut-off is the "ground never above (Z)" setting on Land and sea, default 333, while 333 stays the ridgelines' measuring stick; the near light window's height byte and height tint, and the elevation picture's white, stay at the old 400 / 399 until peaks need more, so nothing looks different.] Both limits are arbitrary (a Minecraft
  habit); nothing should stop a Tower of Babel, and the Lift's points are cut flat at 333. The
  assessment, so it needn't be redone:
  - *Build height:* cheap, since empty sky sections are already one value each. Suggested top
    Z 4,096 (two miles above the sea is ~3,350); section levels are a `u8`, so up to ~8,160 needs
    nothing more, and truly unlimited would mean cubic chunks (not worth it). With it: a chunk file
    format bump (old saves pad with air); a chunk still marks every section for drawing, but empty
    sky is now settled without a meshing job (2026-10-26), so 10× more of them costs a little
    bookkeeping, not job slots; and the near light window's height byte should
    run sea → highest ground, not sea → world top (too coarse at 4,096, and it matches the
    distant terrain's tint that way).
  - *Terrain ceiling:* mountains are sized as a share of the room up to 333, so raising it rescales
    every tuned world. Gentlest: keep 333 as the mountains' measuring stick and move only the final
    cut-off (which also flattens the Lift and hills) to its own setting. Height tint and survey
    scale to 333 too.
  - *Left for later:* builds don't reach the distant terrain or the light map (tall towers vanish
    past render distance and cast no long shadows); block light / daylight must skip empty sky.
  - Changes `docs/decisions.md` (Z 0 to 399) when built.
  - **How tall mountains need to be** [David, 2026-10-27]: peaks **at least ~400 blocks high**, so that
    from the top you overlook the land (grand vistas), instead of standing on an endless plain with a
    cliff nearby. Today they can't: mountains fill the room up to Z 333, so no peak stands more than
    ~205 blocks above the sea. David's plan for now: the Tectonic Sim and the Interpreter show *where*
    the ranges of the right scale go; the height and shape come later (Additives). Other ways to carry
    scale he named: forests taller than Minecraft's, long roads, towns seen in the distance, dangerous
    areas that take a long time to cross. His framing: "a small world that needs to be big and a big
    world that needs to be small".
    - *Counsel's sums [PROPOSED]:* on a flat world, what you see from a peak depends on the angle
      the land makes below the horizon. A typical screen shows about 15 pixels per degree. From 400
      blocks up, the land 5 to 10 km away fills about 35 pixels of screen height, 10 to 20 km
      about 17, 20 to 36 km about 8. So from 400 up, the ground within ~5 km reads like a map below
      you, and the rest of a continent is a band near the horizon. From 200 up (today's best), every
      one of those is halved.
    - *Counsel's note [PROPOSED]:* games solve "small world, big feel" with **vertical
      exaggeration**: distances are squeezed far more than heights, so mountains stand much taller
      against the width of the world than real ones would. David's
      "taller than wide" fantasy mountains are this already. So the scale can be set as one number:
      how much taller the mountains are than real ones would be for the world's width.

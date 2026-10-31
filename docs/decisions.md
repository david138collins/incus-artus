# Incus Artus — Decisions Log

This file records every design and technical decision we've agreed on, and why.
When something changes, we update it here so there is always one place that
says "this is how the game works right now."

Status tags: **[DECIDED]** agreed · **[OPEN]** still needs an answer · **[LATER]** noted, not built yet

---

## Units & Coordinates

- **[DECIDED]** 1 block = Minecraft scale (1 m).
- **[DECIDED]** Creature heights: dwarf 1.5, human 1.7, elf 2.1 (elves crouch through human/dwarf doorways).
- **[DECIDED]** +X = north, −X = south, +Y = east, −Y = west, +Z = up. Origin (0,0) is the centre of the world circle.
  - Note for code: this is a *left-handed* system. Bevy (the engine) is right-handed and Y-up,
    so we convert only at the very edge where things get drawn. All game logic uses our axes.
- **[DECIDED 2026-10-27]** Vertical range Z = 0 (bedrock) to Z = 1,023. 1,024 layers [David: raised from 400 for mountains that tower over the land].
- **[DECIDED 2026-10-24]** Sea level is Z = 128. The ridgelines reach Z = 333 (their measuring stick). The ground is cut off at a setting ("ground never above"), 333 until peaks need more [2026-10-27].
- **[DECIDED]** World is a circle ~72,000 blocks across (radius ~36,000), ~70% water (tunable later).
  Float precision is fine at this size.

## Chunks & Storage

- **[DECIDED]** A chunk is a 32 × 32 × 1,024 column. Internally split into stacked 32³ *sections*
  (32 sections; 13 when the world was 400 tall, and chunk files from then load topped up with air). Chunks are only a storage detail —
  no terrain or biome logic is based on them.
- **[DECIDED]** Block type IDs are 16-bit. Bulky per-block state lives in a separate,
  optional structure that only exists for blocks that need it.
- **[LATER]** Performance techniques planned: see "Performance plan" below.

## Quartile Placement System

Reference: `Art/quartile minecraft illustration.png`

- **[DECIDED, LATER to build]** Every block cell is split in half along each axis → 8 *octants* (½ × ½ × ½).
- **[DECIDED]** A *quarter block* (cut stone, board, brick, building stone) is a 1 × ½ × ½ bar = 2 octants
  = ¼ of the block's volume. It can lie along any of the 3 axes, in any of 4 positions →
  12 possible bar slots per cell. Bars may overlap (share an octant) to make corners.
  Quarter blocks replace stairs, slabs and roof pieces. Dirt slabs exist.
  - Example: stairs = 3 bars stacked (¾ of the cell filled).
- **[DECIDED]** Item placement: every exposed surface — top, side, any step of a stair — offers
  its ½ × ½ quarters plus a centre spot. Small items (mug, flask, bowl) take one quarter;
  medium/large items take the whole face. Centre and quarters are mutually exclusive.
  A 2×1 table has 10 spots (4 quarters + 1 centre per block). Picture frames hang on wall quarters.
- **[DECIDED]** Item sizes:
  - Small items need a quarter face, and can also go in the centre.
  - Large items need a full flat block face. They can span two blocks if both faces are suitable.
  - Centre spots only exist on flat full faces, so a stair cell has no centre.
- **[DECIDED]** A cell can mix materials. When bars overlap, the bar placed first keeps the shared octant
  and the new bar loses that half. Mining a mixed cell destroys it all (blocks generally drop less than they're worth).
- **[DECIDED]** You can't put anything on a surface that already has an object on it.
  Objects placed against blocks protect those blocks from mining until the object is moved.
  Example: a 2-block spear leaned on an octant face against a 2-tall wall.
- **[DECIDED]** Pallets show stored blocks/stone visually, in miniature and compressed.
- **[LATER — REMIND DAVID]** Should quarter blocks ever spawn naturally in world gen? Decide while experimenting with world gen.
- **[LATER]** Floating cone marker showing which face/quarter you're aiming at.

## Blocks (see also `docs/block-properties.md`)

- **[DECIDED]** A block = **material × inclusion**. Material (granite, rich loam…) gives hardness, sound, colours.
  Inclusions are an optional overlay layer (ores first; the same mechanism can carry other overlays later).
  Blocks also carry a tag allowing true texture *variants* later if a good case appears.
- **[OPEN]** Name for the "what kind of thing" dimension (natural granite vs granite cut stone vs granite brick) — see chat.
- **[DECIDED]** Colour by **palette swap**: textures are drawn in shade *slots*, and each material
  fills the slots from the strict palette. A dedicated colour session with David comes before the real textures.
- **[DECIDED]** Inclusions list several ores with percentage chances (galena → lead, sometimes silver).
  Density and grade are fixed at "planet forming" world gen.
- **[LATER]** Staged mining: rock (small, "pennies") → boulder ("nickels") → boulder with a pick,
  or a full block with hammer and chisel. Speed depends on tool, material, and the miner's RPG skill (NPCs and player alike).
- **[DECIDED 2026-10-30] Digging is physical; a built mine or quarry is a workplace** (David: "that's
  the way"). Digging and tunnelling take real blocks out, finite, the same rules for the player and
  miners. **A quarry or mine structure built around a deposit or motherlode** lets a handful of
  employees continually produce its rock or mineral, more slowly. That gives settlements the steady
  supply settlement games rely on, while the deposit is still really there underneath.
  - **Tree farms work the same way:** workers replant trees in rows, so a settlement has a steady wood
    supply without clearing the forests round it.
  - **Mine sites from mineralogy** [David]: the mineralogy that places ores also places the locations
    viable for a mine. **Prospecting can be a skill.** *Counsel:* a mine site is then a known deposit
    with a size and grade, which is just what a built mine draws from. Prospecting reveals sites, and
    perhaps their size and grade, more accurately with skill, so finding a good site is gameplay. The
    sims' settlements and the player find sites the same way.
  - **A site self-culls** [David]: like buildings past their "% changed", if enough of a site's
    mineable or quarryable zone is dug out (by the player or by digging orders), it stops being a site.
  - **The player has an edge** [David]: a noticeable efficiency, drop and speed advantage over NPC
    miners of the same skill level. *Counsel:* this changes "the same rules for NPCs and the player"
    (staged mining, above; `docs/block-properties.md`, Hardness) to **the same rules, plus a player
    bonus**, ideally one number per activity so it can be tuned.
- **Ore depletion: settlements versus solo mining** (David: "a real question ... it'll require careful
  balancing between miners clearing areas and ore depletion and solo play"; the decision above settles
  the main question; the notes below are for balancing it). Settlement sims usually have an ore vein you just keep mining, which doesn't fit physical,
  finite ore.
  - **David's ideas:**
    - **mining versus digging or tunnelling** as different things;
    - **a quarry or mine structure built around a deposit or motherlode**, which lets a handful of
      employees continually produce its rock or mineral, more slowly.
  - *Counsel's notes [PROPOSED]:*
    - **Ore comes out slowly per block:** staged mining plus grade means one ore-bearing block gives
      several units over several strikes.
    - **Labour limits a settlement, not the vein:** common veins last years of game time; depletion
      is a long arc that drives outposts, exploration and trade.
    - **The sims mine abstractly:** a deposit is a remaining amount drawn down at a rate, which
      becomes real tunnels when the place opens up into the Local sim.
    - **Solo play's edge:** settlements work the common ore near the surface; the player goes for rich
      pockets, deep and exotic ores, dangerous places.
    - **A few slow renewables:** bog iron, river gold topped up by floods, salt pans.
    - **Clearing rock feeds construction**, as in Dwarf Fortress.
    - **Balance it in the civilization lab:** village needs, vein sizes and grades, miner speed, run for
      decades.
    - David's built quarry or mine fits all of this: digging is physical and finite, while a built
      mine is a workplace producing at a rate from its deposit (the Intermediator sim's way of mining,
      made into a building).
- **[DECIDED]** Random face variants (e.g. some logs with knots) are feasible. The variant is picked from
  the block's position, so it needs no storage.
- **[DECIDED]** "Constructed" is a yes/no bit, plus optional richer ownership (pillagers, quest tags) for the few blocks that need it.
- **[LATER]** Chemistry: blocks and regions carry chemical properties (e.g. soil). Irregular weather
  (milder/harsher winters) moderates plant growth through them.
- **[DECIDED]** Fewer than 65,000 block types — a few thousand is plenty.

## Regions (buildings, fields, pastures, rooms, "cells")

- **[DECIDED]** Areas are stored as shapes, not as a flag on every block. Selecting a room, area or cell
  (an in-mountain room) marks it as warehouse, pasture, meeting area, etc.
- **[LATER]** In isometric view, mousing over shows the area's border; it can be resized; assigned workers act on it when in season.
- **[DECIDED]** A solo player can farm without ever defining an area.

## Isometric View

- **[LATER]** The camera is far away, so distant-style simplification applies (no face detail needed),
  but people must stay easy to locate.
- **[LATER]** Height-slice scrolling to see inside buildings (needs cut caps, see the rendering notes).
- **[LATER]** See-through indicators on people (at least when their character sheet is open), visible through walls and trees.

## Multiplayer
- **[LATER, as late as possible]** Alpha-tester link: first an in-game note box that posts to a Discord webhook
  (with screenshot, log, position, settings); later two-way chat; later a dev "drop-in" by joining the tester's game as
  a guest. That last one is the same feature as hosting a friend: LAN-party / D&D-group scale, host-a-friend,
  no server infrastructure. A built-in webhook address can be abused; hide it behind a tiny relay when it matters.
- **[DECIDED]** Keep the refactor easy: every player action is a request (already true for break/place);
  players get IDs and their own input instead of one global player (to do in step 6).

- **[DECIDED]** Not a goal. A basic hookup may come later for testing. For now, player actions go through the
  event bus as requests ("player wants to break X"), so a network layer could inject them later without rewrites.

## Time

- **[DECIDED]** One TimeControl resource. Simulation is fixed-tick; rendering/UI variable.
- **[DECIDED]** Max speed 4× (dev tool; may not ship to players). Pause must be possible
  (dev + manager screen + optional setting).
- **[DECIDED]** Calendar target: year = 4 seasons × 7 days; day = 15 min light + 15 min dark.
- **[DECIDED]** The calendar is 0 at world generation until the world is populated, then runs through 1,000 years
  of history. Play starts in year 1000. (The world clock currently starts at year 1; this changes when history
  generation exists.) See `docs/worldgen.md`.
- **[DECIDED]** World generation is built in a **suite**: a separate screen (F2) with the physical world switched
  off. See `docs/worldgen.md`.
- **[DECIDED]** Pawn movement runs on the fixed tick, input polled every frame,
  position interpolated between ticks for display.
- **[DECIDED]** 30 simulation ticks per second (tunable on the debug panel). Heavy systems
  can run every Nth tick.

## Look & UI

- **[DECIDED]** 640×480 is the *UI art* resolution only (maybe 1280×960 later). UI is integer-upscaled.
  The 3D world renders at native resolution.
- **[DECIDED]** Limited palette; full colour reserved for special effects.
- **[IDEA 2026-10-26] Palette sizes** (David's current plan):
  - **The 3D world:** probably at least **600 colours**, every one chosen on purpose and used carefully so it
    means something, allowing for brightness so there's depth. Special effects in the world have unlimited
    colour, so they stand out.
  - **The UI: 256 colours.** It only has to handle windows, buttons and words. Its toys (the three spheres with
    fluid sims) need several rows of blue, red and green; window borders work with what's left.
  - **Toys in the world** may simply be entities/actors that draw themselves their own way.
  - **[OPEN until the art-refinement phase]** Whether lighting, shade and distance stay free (David's
    leaning: full shading levels, shading with the palette by hand) or the finished frame is held to the
    palette. David will judge it by looking, when refining the art becomes the best use of his time.
    The aim either way is the essence of the games he remembers (dark rooms, black screens, high contrast,
    every colour meaning something), never the costume of "8-bit nostalgia".
- **[IDEA 2026-10-26] Palette cycling** for water blocks, showing which way the water is moving, and for small
  white ripples that cycle along the shore edges.
- **[DECIDED 2026-10-28] Water doesn't simulate flowing** (David). "Flowing" water is a **direction and a
  speed**: which way the palette cycle runs, and which way and how fast it carries you or anything in it.
  Like Dwarf Fortress, it has a direction but isn't really flowing unless you move something yourself.
  - **Levels 1-8:** a water block holds 1 to 8 eighths. A **bucket holds one full block**; dump it in a
    2×2×1 pit and you get four blocks at level 2/8.
  - **Natural water doesn't need levels,** except perhaps for **rapids and falls**: a grade of water
    levels plus speed, and **mist or splash foam** made at a point.
  - **Ponds, lakes and the ocean past a river's mouth are still:** no direction. Only the water blocks
    **at the edge** get the shore ripple palette cycle.
  - **David will make a tile map of the ripples** (asset), so they go round corners seamlessly. The
    renderer then needs to know, for each edge water block, which of its sides and corners touch land, to
    pick the right tile (watch list item 15c).
  - **Water works like Minecraft's snow layers, but it spreads, and possibly evaporates** (David, later
    the same day). So water levels are layers in a block, and water you disturb spreads to even out.
  - **Water has practical uses:** diverting a river or stream to drive a **water wheel** may be required.
    So dug channels must fill from the river, and the water in them needs a direction and speed.
  - Counsel's notes [PROPOSED]:
    - **The generator places still water** at the heights Climate saved (lake surfaces, river beds
      stepping down), so nothing has to settle when a chunk loads.
    - **Spreading only starts when something changes** (you dig, pour, break a bank), and only in the
      Local sim, block by block, then settles and stops. Untouched water never computes anything, which
      keeps it cheap; changed water is saved with its chunk.
    - **Natural rivers, lakes and the sea are endless sources:** a channel cut from a river fills and keeps
      flowing without draining it. The channel's direction comes from downhill along it, and its speed
      from its slope, so a wheel turns where the channel drops.
    - **The lake-rim item** (`docs/worldgen.md`, step 8 note) changes with spreading: a crag dip below a
      lake's surface would let the lake spill once disturbed, so the generator should still lower the
      rim or fill the dip.
    - **[OPEN, David]:** does diverting a river take water from it downstream (a wheel upstream dries a
      village's mill below), or are rivers always endless? Does evaporation empty your own pools and
      ditches over days, and does rain fill them?
- **[OPEN]** The master palette isn't chosen yet (`lorecraft/art/Palettes/masterpal.gpl` is an experiment, not final).
  Current textures are placeholders; colours get coordinated once a visual style is visible. Don't check art against it.
- **[DECIDED]** Block faces are 32×32 pixels.
- **[DECIDED]** Placeholder UI during development: black boxes, magenta outlines, bright pink text.
- **[DECIDED]** Plain lighting for now. Pretty lighting later.
- **[LATER]** Widescreen presentation decided once the shell runs.
- **[LATER]** Bottom bar: equip section · three fluid spheres · quickbar.
  Sphere order, left to right: **blue energy** (next to equip, because equip can hold spells),
  **red blood**, **green stamina**. Can change later.
- **[DECIDED]** Game name is **Incus Artus**.
  - **[DECIDED 2026-10-26] Spelled *Artus* on purpose, not *Artis*.** *Incus* is Latin for anvil.
    Strict Latin for "the anvil of art" would be *Incus Artis*; *artus* on its own means "limbs" or
    "tight". David chose *Artus* because it looks and sounds better. The story told publicly: it carries
    the ring of **Arthur** (König Artus, the Latin and German form of the name), which gives the title a
    medieval, legendary sound. Anyone who points out the Latin gets that answer. Older notes and the inventory mockup that say *Artis* are out of date.
- Reference art: `lorecraft/art/references/inventoryequipsave.png` (personal/party view, newest),
  `lorecraft/art/bad art/three man.png` (manager view), `lorecraft/art/personaltest1.png` (earlier in-world overlay).
- **[DECIDED 2026-10-28] Sky, air and weather** (David):
  - **Fog is wanted now.** It mattered little before, but now the terrain is cool to look at, and fog makes
    it cooler. **A couple of miles of average visibility** (roughly 3,000 blocks) gives a deep sense of
    smallness. David has been setting fog distances in the render panel and judging deep screenshots.
  - **Skies:** simple, a couple of colours **dithered** together, offset. **Sunset** in deep red, orange,
    magenta and blue, like the title-screen backdrop of an old adventure game.
  - **Clouds:** super simple, almost cartoony **sprites**.
  - **Thunderstorms and rain are important to get right.** Lightning for the sky and for magic may be made
    in the Toy Factory (`vision/toy-factory.md`). **Thunderclouds:** a mush of dark sprites that block
    the sun, fading in HSL toward dark and roiling.
  - **Warm sun:** just a slider David blends to taste.
  - **Moving shadows:** not a priority; the light is static for now.
  - **God rays:** the usual kind feel expensive and heavy-handed, and volumetric lighting clumsy given what
    the recipe offers. David's idea [IDEA]: infer **two** rays from the sun's position and the angle of the
    shadow of whatever blocks it, and cast simple bands of lighter shade when the sun is hidden but
    shining.

## Textures & Assets

- **[DECIDED]** `Art/` is David's messy workshop. `assets/` is the delivery folder: finished files go there
  and the game reads only from there. Block textures: `assets/textures/blocks/<name>.png`.
- **[DECIDED]** Block textures are exactly 32×32 PNGs (transparency allowed). Wrong size or missing →
  magenta/black checkerboard, so problems are obvious.
- **[DECIDED]** Faces: `all`, or `top`/`side`/`bottom`, or six faces. Side faces: image top = up.
  Top/bottom faces: image top = north.
- **[DECIDED]** Textures must tile seamlessly (greedy meshing repeats them across big surfaces).
- **[DECIDED]** Each face of a block has a texture rotation: `fixed` (default) or `random` (each block
  turned 0/90/180/270°, picked from its position so it never changes). Grass top is `random`. Use it only
  on textures with no "up" direction. Randomly rotated textures must tile seamlessly in all four turns.
- **[DECIDED]** Claude points out factual, functional problems in delivered art (seams, sizes, readability).
- **[DECIDED]** Each soil type gets its own texture; the grass edge is one shared transparent overlay drawn on top of any soil.
- **[LATER]** Distant (shrunk) textures: Claude generates the small versions for a few block types; David processes and touches them up; hand-made versions can then replace the generated ones.
- **[LATER]** Random texture variants, animated textures.
- **[LATER]** Trees generate with quarter logs.
- **[LATER]** A blueprint / structure suite for David to define trees and vanilla structures.

- **[DECIDED]** Optional **Panini projection** (full-screen pass): wide FOV with a natural-looking centre. Tunable
  distance and crop-to-fit, on the Camera page. The FOV setting is vertical.
- **[DECIDED]** See-through blocks, both using texture `grid.png`: drawn on both sides with hard-edged cut-out
  transparency (pixels fully clear or fully solid, no grading), walk-through, targetable (switch per block),
  replaceable (placing a block into one swaps it out). Two kinds:
  - `incus:grid`: every cell visible (faces between grid neighbours are drawn).
  - `incus:outline`: only the outside skin of a volume, so you see the far side's inner face through it.
    For defining exact 3D regions in the creative/structure editor.
- **[DECIDED]** Generated mip levels average colour only over visible pixels, so clear pixels never darken
  coloured ones in the distance.
- **[LATER]** In-game structure editor ("temple"): an adjustable grid to build vanilla structures in, with data-point
  blocks (village-piece connectors for wave-function-collapse generation, tree structures for world gen and growth).
  - **[DECIDED 2026-10-28] Structures are defined by building them** in the suite, with **imaginary
    blocks as overlays** that act as triggers or hooks for other things (the data-point blocks above).
    **First structures: a bunch of oak trees** (David).
  - **Trees are made of blocks:** whole oak blocks and **quartile oak blocks**, so trees can be thinner,
    with thinner branches, or in-between shapes (something David has looked forward to).
  - **A couple of growth stages**, changing at random when a tree grows up.
  - **Tree felling** is a mechanic David has long wanted to build: a tree-felling mod idea is what set
    the whole project off, a month ago.
  - **Far away, trees become sprites** of themselves.
  - Counsel's notes [PROPOSED]:
    - **The hooks a tree may need:** its base (where it stands and is cut), where it may branch or grow
      next, where leaves go, the pivot it falls around, which stage it is. Hooks are invisible in the game
      and visible in the suite.
    - **A tree is one thing, not loose blocks:** when placed, it remembers which blocks are its own, so
      felling knows what falls (and a player's house next to it doesn't).
    - **Growing swaps one stage's structure for the next**, picked at random from several, checked for
      room first so a tree never grows into a wall.
    - **A falling tree is a moving voxel object:** it leaves the world as blocks, turns and falls as one
      body (the shared kinematics layer above), and lands as logs or items. The renderer has to draw that.
    - **Tree sprites can be baked automatically** from the structure itself, from several angles, lit
      like the world (`vision/watch-list.md`, item 17, far entities as sprites).
    - **What it needs first:** the quartile placement system built (quartile oak), the block register
      (oak blocks), and the structure suite. Then trees, then felling.
  - **[IDEA, David 2026-10-29] The asset factory world and entity assets.**
    - **Where:** a little lab, a world set up as a nice platform. There you build Minecraft-style with
      the library's blocks, insert data (hook) blocks over the build, and register it as an **entity
      asset** (an oak tree, a farmhouse). It's needed before trees, but not before rivers are handled to
      David's satisfaction. Its place in the suite is undecided (David's call).
    - **The test world retires:** the world the game starts in has outlived its use. A comprehensive
      dev lab carries over any useful features from the original bootstrapping of the simulations. The
      civilization and behaviour simulator lab may take the rest.
    - **No world is rendered** until David enters the asset factory world or spawns into a World
      Compiler world.
    - **The Registrar switches between block assets and entity assets** (single blocks, and structures
      of blocks, which are a bit different), and it can view entity assets.
      **[IDEA 2026-10-30] Tabs on the dock by kind** (David): **blocks**, **items**, **decorations**
      (largely plants that aren't 3D) and **entities/buildings** (name undecided: trees, buildings,
      roads, placed features). A button enters the creative world (the asset factory world) to make
      blueprints, trees, roads and so on, and the dock shows a **mini isometric preview** of the tree,
      building or placed feature.
      *Counsel:* names to choose from for the last tab: Structures, Builds, Features, Blueprints.
      "Features" covers trees and roads as well as buildings.
      **[DECIDED 2026-10-30] The fourth tab is Blueprints** (David). Trees are blueprints the world
      places during creation, buildings are blueprints, and players have blueprints too: one kind of
      thing for all of them.
      **Zones that are areas, not blueprints** [IDEA, David 2026-10-30]: a pasture is a zone, which is
      also a kind of building (a building that's an area). It probably has no blueprint, but a script of
      its own: a fence along the border, a gate you place, maybe a barn (how is still open). There may be
      only **4-5 kinds of zone** that aren't a contained blueprint, so **each is made idiosyncratically**,
      its own way, rather than through one general system.
      *Counsel:* candidates for the 4-5: pasture, farm field, tree farm (rows replanted), mine or quarry
      site (placed by mineralogy), graveyard. Rooms inside buildings (bedroom, dining hall, bar) stay Dwarf
      Fortress's painted zones (`docs/simulation.md`, section 13) and need no script: built ad hoc and
      zoned, or placed as a blueprint (a bar blueprint, David).
      **[DECIDED 2026-10-30] Zones are the primitive** (David): zones are part of buildings. Build
      something, zone it if it meets a room's criteria, and save it as a blueprint if you wish.
      Blueprints are simple or composites, and a blueprint carries its zones (`docs/simulation.md`,
      section 13).
      **Zone types live in the Blueprints tab** [David 2026-10-30, Counsel's suggestion]: their own
      section above the blueprints. Selecting one opens the **zone designer**, a form of its settings and
      criteria (seats per diner at once, a counter for the dance floor, storage for its items).
      **[IDEA, much later] Zones moddable without coding** (David): the zone designer may eventually get a
      **scripting page**, where new functionality is added by choosing from a **menu of hooks and their
      behaviour**. David's own zones can have the settings they need built in, but for zones to be
      extensible, anyone has to be able to add to them that way.
      *Counsel:* it's the same shape as the Toy Factory's wired behaviour modules (`vision/toy-factory.md`)
      and spell lego (`vision/watch-list.md`, item 18): pieces with inputs and outputs plugged together,
      so the three may share one system. Cheap now: build the first zones out of named hooks and
      behaviours internally, even before there's a page for them, so the page can later list the same
      pieces and modded zones work just like David's.
      **The model: the StarCraft editor's triggers** [David 2026-10-30], for the zone designer and the
      Toy Factory alike: an intimidating page, but everything it can do combines flexibly, and finding
      or making a script extends it. *Counsel:* triggers as an event, conditions and actions, each line
      read as a sentence with blanks to fill in, saved scripts importable and usable as new pieces
      (`vision/toy-factory.md`). A zone's script can
      still place blueprints (the barn, the gate), so the two kinds meet there.
      **Roads** [David 2026-10-30]: defined and generated in the civilization stage, as a line that
      smooths a bit and lays different kinds of road, with transitions between them. They need special
      treatment, since other things rely on them (the spawn, for one: the spot on a road farthest from
      any settlement).
      **The isometric perspective is built with the blueprint module, all of it** [David]: there's no
      reason to wait, and it isn't terribly complex. Like Minecraft's OrthoCamera mod. *Counsel:* the
      dock's preview is then that camera's render baked to a small picture when registered, which is
      also the tree sprite's first step.
    - **Blueprints in segments:** players can make blueprints too, up to **32×32×32 blocks** each.
      Enormous things are made by stamping segments together and rotating them (90° turns): a 50-block
      wall from a lower section and an upper section.
    Counsel's notes [PROPOSED]:
    - Before the test world goes, Builder lists what it and the Sandbox do now, and sorts each feature
      into the factory world, the civilization lab, or retired.
    - The round trip, like Aseprite's: build, draw hooks as an overlay, select, register (with a lit
      preview). Open it from the Registrar to edit, and register it again.
    - The factory world lights with the real lighting module, has a sun slider, and later a far view to
      check baked sprites.
    - The Color Lab works on entities for free: an entity's palette is its blocks' combined palette.
    - The Registrar's block format leaves room now for partial blocks (quarter blocks, levels 1-8 for
      water and snow), so blocks aren't registered twice when trees come.
  - **[IDEA, David and Counsel 2026-10-29] Baked paths in structures and villages.** For workshops and
    houses, not trees. Precalculate wherever possible, "a cheap network of things that can get prebaked
    if possible that keeps the majority of commotion orderly", handled once in a while when the CPU
    gets to it.
    - **The bake, made at registration:** where a villager can stand and walk; doors as the ways in and
      out; a flow field toward each use-point (hooks double as use-points: bed, anvil, hearth, door);
      and a table of travel times between hooks. The Intermediator and World sims use the table alone,
      with no pathfinding. Only the Local sim follows the fields. Baked once and rotated for each of the
      four facings.
    - **The bake is only a shortcut villagers try first; the blocks are the truth** (David likes this).
      A block change inside a structure marks its bake stale, and until it's re-baked, villagers there
      use ordinary pathfinding in that small box. **Re-bake automatically** once it hasn't changed for
      a while, so a fire costs one re-bake, not hundreds. The bake and the live pathfinder share one
      walking rule (the lighting-module lesson: one copy can't drift).
    - **A question mark pops up over a villager's head** when they couldn't find something and are
      thinking (pathfinding). "Very cute, very demure."
    - **Past a % changed, it stops being a building** and is just blocks. Villagers may still use it,
      but it can't be managed with work orders and so on, since it's no longer defined as a building.
    - **Players' blueprints:** nothing is forbidden. The bake reports what it finds: an unreachable
      hook is flagged on the blueprint and never used (so nobody gets stuck); no doors means sealed;
      anything too big uses live pathfinding only.
    - **Villages:** procedural puzzle-piece villages bake their street network between doors when
      generated, so villagers know where to go in town unless something comes up. Trips beyond the
      town (for trees, rock) use the world's pathfinding.
    - **Measure the exceptions, to watch for runaways:** fallbacks to live pathfinding, re-bakes per
      minute and the longest wait, failed paths and stuck villagers, counted per structure so a bad one
      shows up by name. Shown on the Performance page and checked by the soak test. A structure that
      re-bakes too often switches to live pathfinding only until it calms down.
- **[DECIDED]** Random texture rotation is switched per block on the Blocks page: one switch for blocks that look
  the same all round, separate top / sides / bottom switches for blocks with a distinct top.

## Data & Modding

- **[DECIDED]** Text IDs in data and save files (e.g. `incus:granite`); numbers only at runtime.
- **[DECIDED]** One file per block, readable folder layout like old PC games.
- **[DECIDED]** Live reload of data files, built so it can be removed cleanly.
- **[DECIDED]** Modding is a long-term goal — content is laid out so mods can add folders alongside the base game.
- **[DECIDED]** Data files are **TOML**: plain text, comments allowed, X-COM-style readable.
- **[DECIDED]** Models: Blockbench now, a real 3D modeller later. We'll use a loading path that
  works for both (likely glTF, which both can export).

- **[DECIDED]** (Step 7) Blocks are defined in `assets/data/<pack>/blocks/<name>.toml` → ID `<pack>:<name>`; the
  base game's pack is `incus`. Only air is built in. Format and settings: `docs/data-files.md`.
  `look = solid | grid | outline` sets the usual behaviour; `solid`/`targetable`/`replaceable` can override it.
- **[DECIDED]** Misspelt settings are errors (reported with file and line); unknown face words are warnings.
  Mistakes never crash: a bad file keeps the block's last good version; a missing test-world block gets a magenta
  placeholder; a deleted file's block stays until restart (block numbers never change while running).
- **[DECIDED]** Live reload covers data files *and* block texture PNGs (checked every second, real time).
- **[LATER]** A "write this tweak into the block's file" button on the Blocks page (keeping comments).

## Saving

- **[DECIDED]** No save/load-game. Just worlds, Minecraft-style world folders.
- **[DECIDED]** Only changed chunks are saved; untouched terrain regenerates from the seed.
- **[DECIDED]** Settings in readable text; tuning saved as named presets (save / edit / save as).
- **[DECIDED, built 2026-10-22]** World folders in `worlds/`: each part of the game saves its own file (clock, terrain,
  fate, sim, player), chunks store block *names*, every file carries a format version. Save page on the debug panel.
  Details in `docs/simulation.md` section 9.
- **[DECIDED, built]** Three simulation levels (local / rich / thin), hooks, journeys, Fate. See `docs/simulation.md`.

## Event Bus (step 6)

- **[DECIDED]** Two kinds of message: **requests** (someone wants something; the owner checks it and carries it
  out or refuses it *with a reason*) and **announcements** (facts that happened, stamped with their tick).
  Systems never reach into each other to change things.
- **[DECIDED]** Physical world changes happen on ticks, so a paused world is truly frozen; a player's break/place
  while paused is refused ("TimePaused").
- **[LATER]** **Plans** (dig designations, region outlines, blueprint ghosts snapping under the cursor) are a separate
  layer over the world and work instantly while paused (plan while paused, settlers act when time runs). Ghosts
  are drawing only. The structure editor's edits will be marked as coming from the editor and may apply while paused.
- **[DECIDED]** Every player has a `PlayerId` and their own input; the one at this keyboard is the `LocalPlayer`.
  Every request says who sent it.
- **[DECIDED]** Event log: the Events debug page (filters, freeze, clear) and a file per session in `logs/`
  (newest 20 kept), for bug reports.

## Dev Tools

- **[DECIDED]** Debug panel built with egui. Every tunable exposed the moment it exists.
- **[DECIDED]** Event log on the panel, plus whatever diagnostics I need.
- **[DECIDED]** Debug settings can be saved and loaded as named presets (TOML files in `presets/<category>/`).
  Quick version now (Player page); full save-as / editing / per-actor profiles with data loading (step 7).
- **[DECIDED]** An **everything preset** (Overview page) saves and loads every debug setting at once, so a good
  combination can always be recovered exactly (the first change that made a later one work isn't forgotten).
  One file in `presets/everything/`, one section per group. Every new tunable joins a group.
- **[DECIDED]** Defaults: 100° vertical FOV with Panini on (distance 0.3, crop 1.0).
- **[DECIDED]** Shrunk (mip) versions of textures keep the most opaque pixel of each 2×2, so thin cut-out lines
  never vanish in the distance or at steep angles.
- **[DECIDED]** Design philosophy: play with every setting; odd settings that feel neat might "be something".
  Tools should make experimenting and keeping the results easy.

## Motion & Kinematics

- **[DECIDED]** Motion factors are shaped with ADSR envelopes (attack, decay, sustain, release, each with a
  curve), edited on a draggable graph like a synth or Ableton automation.
- **[DECIDED]** Keys: **Shift** = run, **double-tap Shift** = sprint, **Ctrl** = crouch (flying keeps Shift = fast,
  Ctrl = down). Tuned so a clean jump clears a 2-block gap from a walk, 3 from a run, 4 from a sprint.
- **[LATER]** Landing slide (carry momentum on touchdown so careless landings can slide you off an edge). Tried a
  grip-reducing landing envelope; it didn't achieve much. Probably needs real momentum and mass; revisit with the
  kinematics system.
- **[DECIDED, built 2026-10-22, to tune]** **Momentum**: a charge built while running or sprinting (momentum
  envelope: attack = time to charge, sustain = how much charge). Stopping spends it by sliding: a charged body brakes
  evenly, so a full charge stops a full sprint in **`slide_distance` blocks** (tested: 10 set → 9.9 slid). Slower
  speeds slide less (speed squared), slippery blocks further. The charge is held during the slide and fades (release)
  if you just slow to a walk; it's frozen in the air, so it carries through a jump. No special backpedal braking
  (David: normal controls are enough). The "woah woah woah" stop; meant to make charging out of a sprint a skill.
- **[DECIDED]** Player movement physics and world-gen "geology" stay separate systems (different scales and jobs);
  a shared kinematics layer will grow for bodies that move: players, NPCs, animals, carts, falling trees.
- **[DECIDED]** A `Landed` message (where, and how fast you hit the ground) fires on every touchdown: a hook for
  landing sounds, fall damage, stumbles.
- **[DECIDED]** Anything that changes on ticks and moves the camera is drawn blended between ticks (crouch included).
- **[DECIDED]** Crouch: shorter body (1.2), lower eyes (1.05), slower (× 0.3), blended by a crouch envelope;
  can't stand up without headroom; won't step off edges while crouched on the ground (switchable).
- **[LATER]** Windows Sticky Keys pops up after 5 quick Shift presses; the game should switch that shortcut off
  while running (as many PC games do) and restore it on exit.
- **[DECIDED]** Three gaits: **walk**, **run**, **sprint** (forwards only).
  Each step up has its own speed and envelope (walk → run → sprint). Defaults: run 5.612, sprint 7.5,
  double-tap window 0.3 s.
- **[LATER]** What it takes to start a sprint will be modulated by the Athletics skill and the Agility and Speed attributes.
- **[DECIDED]** Sprinting widens the field of view (default +10°) through its own envelope.
- **[DECIDED]** A `Footstep` message fires each time the walk envelope fires (e.g. tapping W to pace an NPC).
  Hook for step sounds, noise and stealth; a STEP light flashes on screen for now.
- **[DECIDED]** Run ramps up from walk speed through an envelope (default: 3.5 s attack, 1 s release).
- **[DECIDED]** Walking has its own envelope (the legs pushing; default 0.15 s attack and release). While it
  releases, you keep pushing in the last direction.
- **[DECIDED]** Every block has a **friction** (1 = ordinary ground). Real speed eases towards what the legs want
  at a rate of *ground grip × friction*, so you slide a bit when stopping, turning and landing (under a block
  on normal ground by default), and much more on slippery blocks. Landing slide needs no special rule.
- **[LATER]** Jump lift-off envelope (grading in the take-off). The jump arc itself stays as it is.
- **[DECIDED]** Sideways and backwards movement are slower than forwards (ratios; defaults 0.8 and 0.65).
- **[DECIDED]** The sprint envelope freezes in mid-air ("your feet aren't on the ground"); a debug toggle allows otherwise.
- **[DECIDED]** Aim: minimal but not zero air control (visceral parkour feel). You can swing around a bit,
  and move arms and legs to try to mantle. To be tuned.
- **[LATER]** Mantling (maybe).
- **[DECIDED]** Each moving ADSR stage has its own curve control. Sharp drop-offs at the start and end of
  motion give "stickiness": movement that feels decisive and reads clearly (important for combat later).
- **[DECIDED]** Speeds stay at Minecraft defaults for now. Realism comes from self-similarity, not slow walking
  (avoid early-Morrowind drag). We're building the tools to balance with, not balancing yet.
- **[LATER]** Profiles for different actors and situations: on a donkey, driving a two-horse wagon, a mule cart,
  magic running shoes, etc. The kinematics system should unify the physical feel across all of them.

## First Test World

- **[DECIDED]** Flat cylinder 100 blocks across (radius 50), centred on 0,0.
  Gabbro bedrock at Z = 0, stone from Z = 1 to 143, one layer of grass at Z = 144.
  Nothing outside the cylinder. Spawn at 0, 0, standing on the grass.
- **[DECIDED]** Placeholder textures: the old 16×16 grass/stone/gabbro, doubled to 32×32 with no smoothing.
- **[DECIDED]** Controls work like Minecraft for now: WASD, mouse look, Space jump/up, Ctrl down (free-fly),
  left click breaks, right click places.
- **[DECIDED]** Breaking is instant with a 0.25 s cooldown after each click.

## Platform & Workflow

- **[DECIDED]** Windows only for now. Lightweight graphics, CPU budget saved for simulation.
  Long render distance wanted; distant-terrain approach decided later.
- **[DECIDED]** Bevy, latest stable, version locked.
- **[DECIDED]** git for history, GitHub (private) for backup.
- **[DECIDED]** Claude writes all code and explains what and why as it goes; David runs the game and gives feedback.
  Stop after each step, and whenever there's a real design choice.
- **[DECIDED 2026-10-26] Refactor religiously** [David]. The work goes a step forward, then retraces its steps
  to change whatever the new step affects. The more faithfully that's done, the more integrity the software
  keeps, and the less likely the project reaches a stage where refactors become both more expensive *and*
  more necessary. Counsel's companion rule: the more often things are refactored, the more they need
  checks that prove nothing broke (old presets still load, the same seed still bakes the same world).

## World Creation

- **[DECIDED 2026-10-25]** A new world is made completely before you enter it, every spring and creek
  included, and saved; nothing about the ground's shape is worked out at runtime. Budget: about
  10 minutes on a slightly below-average PC, shown as a loading screen with the generation drawn live,
  with lore and art. While designing: live sliders, a fast make (seconds) and a full make (minutes).
  Pipeline and reasons in `docs/worldgen.md` ("The pipeline after the landform").

---

## Performance Plan (for reference)

Techniques modern voxel engines use, roughly in the order we'd adopt them:

1. **Uniform sections** — a 32³ section that is all air or all stone is stored as one value, not 32,768.
2. **Palette compression** — a section only stores the handful of block types it actually contains, a few bits each.
3. **Binary greedy meshing** — builds chunk meshes using bit tricks; roughly 10–50× faster than the classic method.
4. **Packed vertices** — each mesh corner squeezed into ~8 bytes instead of ~32. Less memory, more chunks on screen.
5. **Background meshing** — meshes are built on worker threads; the game never stalls waiting.
6. **Cave/occlusion culling** — skip drawing sections that can't possibly be seen (e.g. caves behind solid rock).
7. **Distant terrain LOD** — far chunks become low-resolution stand-ins kept after full data unloads
   (the approach of the "Distant Horizons" Minecraft mod).
8. **GPU-driven drawing** — many chunks drawn in one command (later, only if needed).

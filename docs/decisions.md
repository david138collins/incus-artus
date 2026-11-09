# Incus Artus — Decisions Log

This file records every design and technical decision we've agreed on, and why.
When something changes, we update it here so there is always one place that
says "this is how the game works right now."

Status tags: **[DECIDED]** agreed · **[OPEN]** still needs an answer · **[LATER]** noted, not built yet

---

## The Metaphysics Engine (the guiding principle)

**[DECIDED 2026-11-01]** (David: "make what we have said about caves, the 'metaphysics engine' we have
created official"). The thinking behind it is in `vision/projection.md`.

David: "what we have actually constructed is a metaphysical world, and a physical world. i think if
we think of every way we implement features in this way, so that 1 always equals 1, that will
actually be what makes the emergent behaviors that fool the human eye into enough orderly commotion
that you can look at and touch and it has meaning and reasons. its still chaos, but it's the strict
reasoning that enables the chaos." And: "*nothing* in my world is truly random, because the 'random'
parts have barcodes on their noise that lets us run it forwards and backwards at any time."

- **The metaphysical world is the form:** the seed and its barcoded noise, the stored maps, the plans
  (the river network, the peaks, the placer's choices, later caves and roads), and the recipe. It
  answers "what's here?" for any point, at any time.
- **The physical world is the form projected into blocks** where they're needed: the near chunks, the
  far view, the sweep. It's made from the form and can always be made again.
- **What can't be projected is stored as differences on top:** edits, the settlers' work, simulations
  with a history.
- **1 always equals 1:** the same question at the same place gets the same answer, near or far, on the
  CPU or the GPU, today or after a reload. Noise is barcoded, built from whole numbers so every chip
  agrees exactly, and tested block for block.
- **Every feature is asked, in order:**
  1. Can it be projected from position, seed and earlier stages? Then it's a recipe or a field.
  2. Does it need a plan? Then the plan is made once in its stage, stored compactly, and blocks ask
     "am I inside it?" Rivers set the pattern; caves follow it.
  3. Does it have a history? Then it's stored as differences on top, kept small.
  4. Never let a later stage feed an earlier one (the linear pipeline, `docs/worldgen.md`).
- **Speed follows from it:** the CPU keeps the small world near you that it needs now (collision,
  aiming, digging, settlers, saves). The GPU takes what's seen out to the horizon, because projected
  work never has to be handed back. Nothing is worked out twice. The less a feature strays from the
  projection, the less it costs. David: "the entire project can fail if we don't find a way to make it
  run like a top."
- **[David 2026-11-02; Builder to assess] The renderer treats everything the same:** "i dont like the
  idea of caves having ids to the renderer rather than the renderer eating them like everything else,
  if if they are formed by the recipe. what if the player forms them?" And: "i think its cleaner for
  the renderer to treat everything the same and send it things in a format and order that produces
  the right result, and concentrate the black magic into an analogy for the interpreter; no knotted
  pipelines that become difficult to manage or improve."
  - **Nothing gets an ID for the renderer.** A cave from the recipe, a hole the player dug, a mine, a
    great hall and a castle all reach it the same way, as the world as it is.
  - **The cleverness lives in one place before the renderer**, as the World Interpreter does for
    world creation: it prepares the world in a format and an order that draws right, at every
    distance.
  - *Counsel:* the far view then asks the chunks' question, "what's here?", only more coarsely: the
    recipe, minus the cave plan, plus the saved edits. The plain column keeps its fast smooth sheet.
    Only where the world differs from it (the cave plan's boxes, the changed chunks, both already
    known) is a cell built from the world itself, coarsened. Openings big enough at a ring's cell size
    stay holes; smaller ones average into a darker cell, so "a few dark pixels" falls out of the
    averaging with no special rule. Far lights come from the lights placed in the world.
- **[David 2026-11-02; Builder to review] The far view becomes a warp-foveated voxel ray marcher, now:**
  "its the hard part, we might as well do it now." His reasons: "i feel disconnected with rendering
  being bits and pieces of conventional methods for an unconventionally engineered piece of
  entertainment software", and "i want freedom from asking how each new thing has to be handled by
  what we have." The aim is "a renderer that we can forget about, that will take everything we can
  throw at it and look good", good enough that "everyone says 'how?'".
  - **Each pixel asks the world "what's here?"** at the size it can show, so holes, caves, changes and
    villages need no special handling: the principle above, carried into drawing.
  - **The CPU keeps a map** of the world from the recipe and the saved changes (plain, interesting, how
    high and low), updated only where something changes; **the GPU marches.**
  - **Resolution falls off with distance** along a curve David tunes, in a **dome** of cells fixed to
    the world around the player (no shimmer when turning), updated by a budget each frame, not redrawn.
  - **It doesn't touch world generation:** it reads the world at the end of the pipeline and never
    feeds back. World gen stays as it is until it can be seen "in all its glory".
  - **Tested with caves and braziers at night first**, then a stand-in forest of simple block trees;
    trees aren't needed to start.
  - The full discussion, options and draft path: `vision/proposal-voxel-renderer.md`, last section.
  - **After step 1 (builds 11 to 31), David chose to finish it** ("lets do it"): the floating origin,
    a test world button, the near world marched straight from the chunks, a check on weak hardware,
    one lighting near and far, quarter blocks and bars, sizes by distance, then the line all the way
    in (Builder's path, `docs/far-view.md`, last section). **The world is marched, things are drawn:**
    models, sprites, creatures, and see-through surfaces (water, glass) are drawn over the march.
    David: "im not attached to the near look", so the march may change it.
- **Content is added across the pipeline** (David, 2026-11-01): "as long as we have a stable
  foundation that i know wont break when i want to add features like a layer that finds swampiness
  centers, and can effect the ecology and topography and the civilization gen can place structures
  and bad guys, we can add content horizontally through the pipeline that fills area that need detail
  or attention."
  *Counsel:* one feature (a swamp, say) touches many stages: flat lowland in Topography, the water
  table in Climate, peat and mud in Mineralogy, reeds and frogs in Ecology, a bog village or witch
  house in Civilization. The rule that keeps it from breaking: **its centre is defined at the
  earliest stage that has what it needs, and every later stage reads it.** A centre found from the
  rivers can still shape the ground, but only as fine detail after the rivers, as the river carving
  does. If it should shape the broad land the rivers flow over, it has to be found in Topography
  from what's known there, and the rivers then react to it. Either way works; the choice is made on
  purpose, per feature.
  - **[IDEA, David 2026-11-01] A second pass:** "you just have to work left to right when thinking
    about creating detail and how it finds its place. i think we can also create a modular re-read
    method that gets to the end, and then goes back again. it could be a discrete 2nd order locale
    generator... the only cost may be running a 25 second world gen twice."
    *Counsel* [PROPOSED]:
    - **It keeps the rule.** A second pass is just more stages to the right: the first pass's
      results become inputs, like the bake is. Two passes, fixed, never "repeat until it settles",
      which may never settle.
    - **What the first pass hands on is small:** the locales (centres, regions and their kinds, such
      as "a swamp here", "badlands here"), never its fine detail. The second pass builds the world
      from them.
    - **The catch:** the second pass can undo a first-pass reason. A swamp centre found beside a
      river shapes the land, the river moves, and the swamp is no longer beside it. So the locale
      must be checked against the finished world, or chosen from causes the second pass keeps.
    - **Cheaper version:** the first pass can be a quick, coarse scout (the rivers already run on the
      broad ground), so it costs well under double. Stages before the re-entry point don't rerun.
    - **The costs to watch:** tuning (a first-pass slider moves second-pass results in ways that are
      hard to follow) and "why is this here?", which then has to show both passes.
    - **When:** when a feature first needs it, not before.
    - **David, later the same day:** "we can also just keep adding tabs as far to the right in the
      order as we find it necessary. we may not need a 2nd pass if it is arranged as a boutique world
      engine. but it could conceivably be used to good end, so keep it to think about." So the first
      answer is a new stage further right; the second pass stays on file.
- **[DECIDED 2026-11-01] Projected code is GPU-safe** (David agreed to Builder's assessment and
  Counsel's advice: "i agree with all if its like i described"). Code that projects blocks during
  play (the ground's height per column, river carving, the noise, the placer and mineralogy, and
  later caves, ores, tree placement and roads) uses whole numbers and fixed point, and no sin, cos,
  pow or other maths that varies by chip, driver or operating system. Then the CPU and the GPU agree
  bit for bit.
  - **World creation is exempt:** its results are stored (the bake, the climate, the rivers, the
    peaks). David: "its totally fine to roast the crap out of peoples cpu when they generate a new
    world... i'm only concerned with the performance when the player is playing."
  - **From now on,** new projected code is written GPU-safe, with a small kit of GPU-safe maths so
    it's no harder to write. Today's roughly 5,500 lines are converted with the graphics and
    performance pass.
  - **First step now:** a whole-number noise, with an A/B switch on the World Compiler ("noise maths:
    fractions / whole numbers") and a timing readout.
    - **[BUILT 2026-11-01]** `worldgen/noise.rs`: `whole_perlin` and `whole_perlin_with_slope`.
      - It uses 32-bit whole numbers only: a murmur3-style 32-bit barcode, and 64 fixed arrows from a
        table. Positions, values and fades are in 65536ths (4096ths until the fix below), the arrows
        in 4096ths, with every step checked to stay within 32 bits.
      - The only fractions are the way in (the position) and the way out (a division by a power of
        two, which is exact).
      - The switch is the recipe's `noise_maths` (whole numbers by default since David's word below), saved with the preset and
        the world. It covers every noise the recipe uses in play: hills, crags, summit crags, the
        ridges and their wander, the peaks' wobble, the torn coast, and mineralogy's wander. World
        creation's noise is untouched.
      - It sits in the World Interpreter panel's "Noise maths (for now)" section, with "Time both"
        (the noise alone, and the recipe at 5,000 columns of the world compiled, both ways).
      - **David's first look (2026-11-01):** "on first glance, nothing got worse or that different but
        rivers became more... pleasant somehow." **The switch stays** "until it becomes expedient to
        delete any of the fractional maths coding. that'll give me time to get a better impression."
        *Counsel's guesses, unchecked:* the ridges' wander is in the ground the rivers are planned on,
        so new barcodes give a new river network; and 64 evenly spread arrows give evener texture
        than arrows at any angle (as in Perlin's improved noise of 2002).
        **Checked later the same day:** the coarse first version's flaw wasn't the cause. David,
        comparing it with the finer fix: "fine noise doesn't change the rivers at all... whole numbers
        is the same. no worries." Both versions share the new barcodes and the 64 arrows, so which of
        those two it is stays open, and David doesn't need it separated.
      - **[DECIDED 2026-11-01] Whole numbers are the default** (David: "i'd like for whole math to be
        the default setting now; when we gotta can the fractional code entirely just for
        housekeeping or software reasons, i'll make the final call."). The fractional noise stays
        behind the switch until he decides to remove it.
      - **[BUILT 2026-11-01] Made the default, and made finer.** `noise_maths` is now whole numbers by
        default. None of David's presets saved a choice, so they all come up in whole numbers, his
        startup default included, with nothing re-saved.
        - Making it the default turned up a flaw. A ridges test failed: the first whole-number noise
          had too coarse a grain. Positions, values and fades were in 4096ths, and the rounding at
          each step made faint wiggles, about a thousandth of the noise. Folded into ridges, those
          made false crests: 71 along a test line where fractions made 2. On a 200-block ridge that's
          a fifth of a block, enough for stray one-block bumps.
        - Positions, values and the fade are now in 65536ths, still within 32 bits (the larger
          products split in two).
        - Along lines it now makes as many crests as fractions do (11 against 10, 12 against 10).
          Its biggest tick back is 0.0002 of the noise, 0.04 of a block on a 200-block ridge.
        - The speed is unchanged: about 14 ns against 67.
        - A test now checks the crests and the ticks.
        - The ridges test was a single line, a thin test that its pattern decided. It now sums five
          lines. Both maths pass it alike: steady ridges stay apart far out, about 10 crests a line,
          while the old way's crush together, about 20.
        - The rivers David found "more pleasant" were made with the coarse version. A look at the
          finer one would tell whether that holds.
      - **Speed:** in a release build the whole-number noise took about 14 ns a call against about
        70 ns for fractions, roughly 5 times faster (no sines or cosines, and a cheaper hash). With
        its slope, it was about 20 ns against 80.
      - **Tests:** it's smooth, zero on its posts, and its slope matches its measured slope. Its spread
        of values is within 15% of the old noise's, and nothing overflows far out. A map made both
        ways is as rough either way, in a new pattern.
      - **Not yet:** the recipe's own fractions (curves, blends, pow), and the GPU check (running it
        in a shader and comparing block for block), both with the graphics pass.
  - **The test is David's eye, not the bits** (David: "if i cant look at ten worlds and tell you which
    one was 64 bit seeds and which was 32 bit, it doesnt matter. i dont care about anything saved in
    the world gen stack except for the default presets i have and their apparent outcomes. as long as
    it all works the same. if it comes out drastically different we have to assess whether its
    possible to find out why and fix it."). So a new 32-bit hash, with new barcodes, is fine if his
    default presets still look and work the same.
  - **Also fixed with it:** saved maps reload exactly as played (the maps are rounded at spawn as
    they'll be saved), and slope is one measurement in every view, from whole blocks, with a
    whole-number square root and its span named in the field.

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
- What building it touches, the options, the order and the sizes: `docs/quartile.md` (Builder,
  2026-10-31, an assessment for David; nothing decided beyond the above).

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
- **[IDEA, David 2026-11-01] Minerals are rock, and three ways to get ore.** From David's Obsidian
  notes (`lists/minerals.md`, `lists/MATERIALS - ore.md` in his Incus Artis vault, which stay his) and
  chat.
  - **Minerals are called rock:** "hematite is a rock, you can make blocks with it, and it may drop
    iron ore, along with rocks and boulders, or a whole block if quarried or player mines it." So an
    ore rock is a building stone as well as an ore, mined by the staged mining above.
  - **Inclusions too:** "you can find iron inclusions like in minecraft overlaid. and oerhaps those are
    where mines can go." Native ores are an overlay, like moss (`docs/objects.md` section 11).
  - **Tiers:** "common minerals, like rocks, that may have inclusion ores. there are less common
    minerals, like chromite, that may have inclusions, and also drop an ore. there are more useful
    metals, and metals that are mostly economic or decorative, or related to the magical/alchemical
    system."
  - **Colour first:** "the common minerals are important to get the colors right on so the mountains
    look good and so the normal stone building materials look good."
  - **The ladder of getting ore:** "with small chance of ore from inclusion rocks, and also other
    scatters of iron ore here and there, you could get a bit by mining minecraft style, huge amounts
    with big mines and many workers, reasonable amount quicker if its just strip mined like gems."
    1. **A bit, Minecraft style:** a lone player tunnelling gets ore now and then from inclusions and
       small scattered pockets.
    2. **A reasonable amount, quicker:** strip mining a seam near the surface, like gems.
    3. **Huge amounts:** a big mine with many workers on an ore body (the built mine above).
  - *Counsel* [PROPOSED]:
    - **Three sizes, left to right in Mineralogy:**
      1. **Rock layers** (granite, basalt, gabbro, limestone, sandstone, gneiss, slate, marble), the
         big layers that colour whole mountains;
      2. **ore rock bodies** inside them (hematite, magnetite, malachite, galena, garnierite, bauxite,
         chromite): pods, veins and beds; seams near the surface are strip-mined, and deep bodies are
         the mine sites;
      3. **inclusions** overlaid on ordinary rock around and above the bodies.
    - **Inclusions are the clue that leads to a mine**, as real prospectors read green malachite
      staining over copper ("malachite - weathered oxidized" in David's list). This fits prospecting
      as a skill (above).
    - **It joins the two games:** iron is never out of reach for a lone player, but it's plentiful only
      for those who build something. A big mine needs timber for supports (felling feeds mining), carts,
      haulers and smelters. A strip mine leaves a scar you can read from the sky.
    - **Small gaps in the lists:**
      - chromite is in the notes but not in the list;
      - cassiterite (tin) and sphalerite (zinc) are in the ore list but not the minerals list;
      - peridot is olivine's gem;
      - onyx is a decorative stone;
      - granite, basalt and marble aren't listed yet, though the mountains likely need them for colour.
    - **How often each rung gives ore** is a set of numbers for David to tune.
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

- **[David 2026-11-04] Two views, not one; the names aren't final** ("i don't really know for sure"):
  - **The manager:** the full-screen window of lists, sheets and settlement info, for broad overviews.
    (`docs/vision.md`, "Full-screen manager".)
  - **The orthographic view, or orthographic manager:** "more like an RTS or strategy game mode", for
    directing his settlement and soldiers. Like Dwarf Fortress, it uses overlays for planning and control
    in specifics: blueprints, zones, routes (`docs/simulation.md`, section 14). The same view as "the
    foreman view" elsewhere in these docs.
  - **Away from a settlement it's still there** (David): travelling, with no settlement to direct, the
    orthographic view stays available for looking around, mostly without controls. "maybe later there
    could be more".
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
- **[DECIDED 2026-11-01] Light is precious** (David): "the world will be very dark at night. i want
  light to be precious. wouldnt caves just be pitch black unless lit by things just like the world
  above at night, depending on the moons?" So nights are very dark, and **caves are pitch black**
  unless something lights them (torches, fire, glowing things), or near a mouth by day. It fits the
  aim in Look & UI: "dark rooms, black screens, high contrast".
  - **[IDEA] Moonlight from the two moons** (David: "moon stuff is gonna be big, since theres two
    goddesses and one god that are not inimical"; `docs/vision.md`, Story and setting: both goddesses
    live inside a moon, Fire at the centre of the earth, Darkness in the wandering planet X).
    *Counsel* [PROPOSED]:
    - **Two phases make many kinds of night:** both moons full (bright), one, or neither (black).
      Periods that fit the 28-day year (one of 7 days and one of 28, say) make a calendar of nights
      players learn and plan by.
    - **Each goddess's moon may carry her nature:** Fate's moon full, the odds lean kindly; Life and
      Death's, healing and the dead stir. Planet X near might darken nights on its own.
    - **Fire's light from below:** the deeper you go, the nearer Fire, so the deep glows (magma,
      warm crystals) while the middle depths are blackest.
  - **[PROPOSED, Counsel] Darkness is also cheaper.** Sky and moon light are projected (the sun's or
    moons' place and the ground, the same light canvas in other colours and strengths). Underground,
    sky light only reaches in a little way at mouths, and the cave plan knows where those are. Every
    other light is a placed source (torches, fires), stored as edits and spread a short way. With
    few sources, that stays small. Much of Minecraft's light cost is sky light spreading through
    every cave, which this skips.
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
  - **Mopping and drying** (David, 2026-11-02): "water [should] spread out to 1/8 layers of water
    wherever they spill to if you dump a bucket, and to be able to soak it up with a mop, or it
    evaporates, at 1/8. dumping the bucket in a 1x2 block hole would result in two blocks with 4/8
    water in them." The water is kept count of in eighths: a bucket is 8, spread is shared out, a mop
    takes it back, drying takes it an eighth at a time.
  - **How water looks is open** (David, 2026-11-02): "that would need to be transparent, i think, im
    not totally sure ... its possible i leave flowing water opaque and have transparent still water?
    i really don't know. as long as we have options, when details like waters appearance really
    matter." *Counsel:* the renderer keeps them open. Water is drawn as a surface over the march at any
    level of eighths, see-through or not per kind of water; deep-water darkening, reflections and
    refraction can be added to the surface later without touching the march; opaque water could even
    be a solid cell in the march. Decide by looking, once the test world has a lake.
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
    in the FX Lab (`vision/toy-factory.md`). **Thunderclouds:** a mush of dark sprites that block
    the sun, fading in HSL toward dark and roiling.
  - **Warm sun:** just a slider David blends to taste.
  - **Moving shadows:** not a priority; the light is static for now.
  - **God rays:** the usual kind feel expensive and heavy-handed, and volumetric lighting clumsy given what
    the recipe offers. David's idea [IDEA]: infer **two** rays from the sun's position and the angle of the
    shadow of whatever blocks it, and cast simple bands of lighter shade when the sun is hidden but
    shining.
- **[DECIDED 2026-11-01] Every element owns its compromise artistically** (David): "we cant think of
  any element of graphics as plumbing, it has to be like the gilded external plumbing and electrical
  conduits of blued brass with filigree moulding at the ends found only in napoleonic french chateus.
  every element has to own its compromise artistically." Every shortcut taken for speed (lower
  resolution, fewer faces, sprites far away, fog, dithering) is shown as a chosen style, never hidden
  as a defect.
  - **An artistic upscaler for the distance** (David): "we need an artistic upscaler in order to
    render the distance in a way that is simultaniously fast and a joy to look at." The far view is
    drawn at lower resolution, and the upscaler turns that into a look of its own.
    *Counsel* [PROPOSED; the look is David's call], candidates to try side by side:
    - **Edge-keeping upscaling,** as pixel-art upscalers do (the xBR and Scale2x family): it keeps
      silhouettes crisp (ridgelines, the horizon, coasts) and fills inside them. The eye reads the
      distance by its edges.
    - **Painterly smoothing** (a Kuwahara filter): flat patches of colour with sharp borders, like
      brushwork. It's cheap on the GPU.
    - **Air, as chosen colour:** the fog of a couple of miles, tinted in steps toward the sky's
      colours, with dithered bands between steps rather than smooth blending, which matches the skies.
    - **Pixel size growing with distance in steps,** like a mosaic, with dithered seams, so the drop
      in resolution reads as a deliberate scale.
    **They'll probably need to be layered** (David, 2026-11-01: "all those artistic upscaling options
    will probably need to be layered"), for example crisp silhouettes, painterly inside them, and air
    in dithered steps over all. It's measured and judged in the graphics pass (`docs/worldgen.md`, path
    step 8; `vision/proposal-voxel-renderer.md`, "Temporal reuse"). Hue still carries meaning: no
    upscaler may shift a block's colour into another's.

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
  - **[IDEA, David 2026-11-01] How a felled tree lands, and chopping it up:**
    - **Logs stay together as much as they can:** "once you chop it down, you have to chop it up
      again."
    - **Chopping up is a minigame:** logs into quarter logs, for fires or building, "just like hammer
      and chiseling full blocks apart". Once a piece is cut, "it has to pop off and be pickupable".
    - **The tree turns over whole:** "most blocks should do a 90 degree rotation", so the tree lies
      down keeping its shape. "not all leaves vanish. only ones that have to": only leaves with no room
      where they'd land. Branches on the side the tree falls toward break off.
    - **Leaves piled on half the logs:** "there should be s grass piled up on half the logs", and
      then "i mean leaves, not grass".
    - **The fall itself is simple:** it tips over an edge, slowly getting faster until it's at 90
      degrees, then drops straight down, and may bounce once. It can injure and knock back players.
    - **The hard part is the branch underneath:** unless it snaps off whole and finds a place to one
      side of the main log and the other branches, it would land on top of them. David's words:
      "except for the branch on the bottom, unless it snaps off whole, and somehow finds a place to
      either side of the main log and other branches, that would land on top."
    - *Counsel* [PROPOSED], for the branch underneath, in order of preference:
      1. **It folds aside:** it turns another quarter turn at its joint and lies flat beside the
         trunk, still attached, so the logs stay together.
      2. **It snaps off whole** and skids out to the nearest free place on either side.
      3. **Neither fits:** it breaks into pieces you can pick up.
      - **Who gets hit:** the fall is planned before it starts, so the ground it sweeps is known. Anyone
        under it is hit and knocked back, and there's room for a warning first (a creak, a shadow).
    - **[David 2026-11-01] Folding first, and leaves like a pillow:**
      - **The branch folds before the tree hits 90 degrees,** toward the end of the tip, so the tree
        lands as a flat stack and bounces as one.
      - **Leaves end up on top,** because "any collision breaks them like shooting a goosedown pillow":
        leaves that hit anything burst.
      - **The direction is slightly random,** so slow or clumsy villagers and players may be injured.
      - **It's a roll at the cut, with a range** (David, correcting Counsel): "it wouldnt fall the same
        way though if it was rolling when you cut it. itd just have a range, if you roll below a
        theshold it may fall to the back or side." Usually it falls about the way it was cut. A roll
        below a threshold sends it backward or to the side.
      - *Counsel:* a cut is history, not projection, so a real roll is fine (`docs/decisions.md`, the
        Metaphysics Engine). The game rolls once at the cut, and the result is saved with the fall.
        A villager's skill could move the threshold.
    - **The fall looks bouncy but snaps:** a path that looks like physics but only takes each piece to
      its place on the octant grid. It has to be fun to watch.
    - **Dropped items and breaking:** "if i cant think of a better way, minecraft isnt broken."
    - *Counsel* [PROPOSED]:
      - **The whole fall is worked out when the last cut lands:** the direction, where the trunk rests
        (downhill wins, kept in one piece), and which branches break. The fall is then only an
        animation toward those places (a tipping arc, a bounce or two that get smaller, a settle),
        tuned like a cartoon. It's the same every time, and only where things ended up is saved.
      - **Breaking:** crack stages drawn by David over the block, and chips coloured from the block's
        own texture.
      - **Dropped pieces could settle onto an octant as small objects**, so a pile is a real pile.
        Minecraft's floating, spinning drop is the fallback. Whether drops fade away after a while is
        David's call, later.
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
      **[DECIDED 2026-11-01] Three kinds of thing: blocks, objects, blueprints** (David: "these are all
      the same thing, swords, armor, brazieres, torches, bread. blocks are blocks, objects are objects,
      and blueprints are made of both."). An **object** is a model plus attachments (a sprite, a light,
      later particles and sound), whether it's held, worn or placed (`docs/objects.md`). So the decided
      **items** tab is the Objects tab. **Plants are objects too** (David: "objects can be 2D, like
      grass, or 3d, like corn or brazieres or beds"), so the decorations tab folds in as well. The
      Registrar has three: Blocks, Objects, Blueprints.
      *Counsel:* two things keep meadows cheap. Plants Ecology places are projected (worked out from
      the plan, never saved); only ones a player changes are stored. And 2D objects in bulk can be
      drawn into their chunk's mesh as crossed sprites, as Minecraft draws grass, while staying
      objects in the data.
      **Plants change over time** (David: "the plants may need to change from time to time").
      *Counsel* [PROPOSED]: time is one more input to the projection. A plant's stage (sprouting,
      grown, flowering, seeding, dead in winter) is worked out from its place, the seed, the calendar
      and the season, so untouched plants change with nothing saved and 1 still equals 1. A
      disturbance (eaten, cut, trampled, burned) is stored as a difference with its time, and the
      regrowth after it is worked out from the time since. Slow changes across whole regions (a
      forest spreading, a pasture overgrazed) are kept as coarse maps that the plants are then
      projected from.
      **Sprites for effects and spells** [IDEA, David 2026-11-01, after seeing his flame in the game:
      "i did the most haphazard bs in aseprite real quick and that flame sprite animation looks
      terrific"]: "a lot of spells can throw sprites or cause sprites. ive seen really good bouncing
      particles that are sprites."
      *Counsel* [PROPOSED]: one sprite-particle system for all of it: the flame, the leaf burst from a
      felled tree, chips from mining, brazier sparks, splashes, spells. David draws the strips, and each
      effect is a recipe: how many, how fast, how bouncy, how long they last, the light they carry.
      Bouncing is a whole-number check of the cell a particle moves into, so thousands can run on the
      graphics card (Noita and Terraria get a lot from this). A spell sprite carries a moving light, the
      third kind already planned, so a fireball in a pitch-black cave lights the walls as it flies. It's
      the FX Lab's particles (`vision/toy-factory.md`).
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
      *Counsel:* it's the same shape as the FX Lab's wired behaviour modules (`vision/toy-factory.md`)
      and spell lego (`vision/watch-list.md`, item 18): pieces with inputs and outputs plugged together,
      so the three may share one system. Cheap now: build the first zones out of named hooks and
      behaviours internally, even before there's a page for them, so the page can later list the same
      pieces and modded zones work just like David's.
      **The model: the StarCraft editor's triggers** [David 2026-10-30], for the zone designer and the
      FX Lab alike: an intimidating page, but everything it can do combines flexibly, and finding
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
      **[David 2026-10-31] Roughed in with the Registrar upgrade, and measurable:** the orthographic
      camera mode should be "present enough to be relevantly measured during the graphics and
      performance pass" (`docs/worldgen.md`, "Where the road stands", path steps 5 and 8).
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
- **[DECIDED 2026-10-30] Plan for a full revision pass** [David]: once the game is "done", everything
  gets gone through and adjusted again, so build for that from now on. Keep everything **formulaic,
  linear, hierarchical and modular** (the world gen's linear pipeline, extended to every system), which
  makes the second pass easier. **Nothing is ever "done"** [David 2026-10-31]: "nothing is done,
  everything is only done enough to do the next thing, until we go back and fine tune everything again."
  *Counsel:* in practice that means
  - **numbers in data, not code** (sliders, presets, data files), so a revision is retuning, not
    rewriting;
  - **each system reads only from what comes before it**, so changing one place has a known reach;
  - **one formula per behaviour**, shared by everything that does it (the player's mining edge as one
    number per activity, the same zone rules for built and placed rooms);
  - **pieces with named seams** (hooks, stages, modules), so one can be swapped without touching its
    neighbours.

## World Creation

- **[DECIDED 2026-10-25]** A new world is made completely before you enter it, every spring and creek
  included, and saved; nothing about the ground's shape is worked out at runtime. Budget: about
  10 minutes on a slightly below-average PC, shown as a loading screen with the generation drawn live,
  with lore and art. While designing: live sliders, a fast make (seconds) and a full make (minutes).
  Pipeline and reasons in `docs/worldgen.md` ("The pipeline after the landform").

---

## The regroup: leave Bevy, start the graphics again on the card

**[DECIDED 2026-11-05]** From David's talk with Builder after chain 7's measurements. Builder's plan, its
steps and what's kept are in `docs/regroup.md` (PROPOSED until David reads it). Where the renderer sections
below disagree with this one, this one stands.

- **Regroup:** "it sounds like we should do a ruthless cut pass, get it working again, without graphics at all,
  and start inside the gpu. just don't do work that makes the cpu do things it won't be doing in the commercial
  steam build."
- **Kept:** "our editor, world gen, simulation, and the setup you've done to make it easier to work on, these
  are good." And: "as long as we don't lose anything that we are happy with".
- **Bevy goes, with our own scheduler:** "im leaning towards just taking anything good from bevy (like the
  tetris core scheduling...) and starting over." Then: "as long as you know what you're doing and feel good
  about having our own scheduler for the simulation multiplexing, i'm into it."
- **Build 60 stops:** "its stuttering and thats a dealbreaker." The measurements: Bevy's DefaultPlugins cost
  about 6 ms of processor a frame before anything is drawn (12 W more at 120, 27 W at 240); the stutter is in
  our own code, not Bevy's, so leaving Bevy alone doesn't fix it.
- **Hybrid allowed:** "i am very attached to the idea of marching everything simultaniously but it may require
  maths i just can't access. getting the game done is more important. we should be able to make it look right
  with a hybrid setup. i do still believe in a somewhat exotic far distance rendering solution, that naturally
  pixelizes, and light that naturall is coarse - but i have an open mind. as long as its beautiful to look at,
  fast, and reads perfectly to the player."
- **The spec, restated** (replaces "unbreakable 120 or smooth 240"): "if this laptop can't always hold 250,
  thats ok, as long as it holds it enough that i don't really feel the need to set it to 120, but if i do set
  it to 120, i don't want to ever see it stutter once. someone with a smaller resolution and a 980ti should be
  able to get the full experience, 60fps that are smooth are enough for anyone. someone with a stronger PC than
  mine, with better cooling and not a laptop, shouldn't even be a question at 250fps+." Measured at the screen
  with PresentMon, not inside the game.
- **Two bands** (his "powerglide"): blocks out to where a block is smaller than a pixel, then one far band done
  another way, so there's only one transition to tune, and it's far away. Builder: the world as blocks made on
  the card in pixel-sized cells to about 1,500 to 2,000 blocks, then the backdrop.
- **Graphics first; the simulation's scheduler when there's something to simulate:** "i dont think graphics and
  scheduler should impinge on one another and, to be frank i have high confidence in her ability to do it right
  when its time. right now we have nothing to simulate... its small enough that it wont make a difference being
  unoptimized in the meantime." And: "getting the vision in the gpu *the right way* is our mortal threat and the
  most fundamental issue we have... graphics are spooky and hard." So step 0 gives the simulation its own lane
  running the toy rules as they are; the column store and scheduler wait for the simulation stage.
- **Start small, check as we go:** "my instinct is to just start small and keep checking performance as we
  assemble vision." On the stutter: "we need to isolate the offender and not repeat it". Step 0's test window
  logs what ran in each slow frame, so the new floor is shown to be free of it.
- **The editor isn't shipped for now:** "the editor is for me until the best use of our time is serving eager
  modders. thats a ways off. anyone who has it in the meantime is someone im talking to and showing them it
  personally." Maybe later as a paid release or bundle ("not super important right now"). So the shipped build
  leaves the editor out; ours keeps it.
- **Light on our feet:** "we dont really know whats going to happen, we will be shooting from the hip again with
  graphics. we have to stay light on our feet for this path from black to fast full visuals. i dont regret our
  process and i dont see it as wasting time. we needed to see the world to tune it and now its tuned in close
  enough that i know perfect is within our adjustment range. i know it will be there when we have eyes again.
  that gives us latitude." So the plan from black to the whole world is detailed near and loose far, with
  forks where a measurement or his look decides the next move.
- **The go, phase 0 started:** "if you think you understand, i believe we're ready to begin, do you?" Builder
  said yes. The plan is `docs/regroup.md` (through 1167834). Before it, on what waits:
  - Water: "i think we know that our river gen works, and i also am aware that there are two kinds of water in
    the world right now. i don't think designing water into its sophisticated end state is impinged upon by the
    graphics work". So the sea, rivers and lakes are a flat stand-in until phase 6.
  - Caves: "caves require a solution we've hit a brick wall trying to solve the old way; i think we know how
    we'll make them, the hard part is how do we see them, and the work there is just experimenting with the gpu
    and finding a way that is fast, accurate, and pretty."
  - Trees: "trees i don't think are a problem because you can make them with dirt and malachite blocks for now.
    the way they fall is kinematics, not graphics."
- **What stays of Bevy, and its ejection seat:** phase 0 found our modules built on Bevy's ECS throughout, so it
  cuts Bevy's renderer, window, assets and the rest, and keeps the ECS core for now. David: "we're not having any
  mercy on bevy. anything we keep either has to be because its the best way of doing it, or replacing it is
  impossible for claude and david. if we keep anything in place in the meantime - we need to build explosive
  fastener ejection seats for it. so we can use the ECS, and later, modify/improve it for our purpose, or copy
  what is good in it; we just have to make sure it is ready for rapid disconnect." So: one doorway module for
  everything Bevy that's kept; new code never touches Bevy; a test counts Bevy's footprint outside the doorway
  and fails if it grows; a disconnect list kept current. It comes out when the column store arrives, or sooner
  if the phase-0 floor shows it costing more than about 0.5 ms or 30 wake-ups a frame at 120 over no Bevy.
- **Timing, priority and the hardware's attention** [David 2026-11-06]: "timing and sequencing is absolutely
  critical for ongoing stability. perhaps the monitors hardware refresh clock is a better universal reference for
  the rendering to time itself by, rather than a windows process meant for mspaint and edge, as long as it doesnt
  impinge on our simulation clock at all. the sim clock seems to be working very well. we should probably a
  criticality flag system almost like df has job priority so that the most important things get done first, even
  if thats invisible i think it could spare us some grief later. first order of business is finding a way to keep
  the hardware at full attention." So:
  - the frame is paced by the screen's refresh, and the simulation's clock stays its own, from real time;
  - every job carries a priority, and each lane does the most important first within its budget, so the least
    important arrives late;
  - the hardware is kept steady, with no clock drops or core moves that cause stalls, measured first, without
    giving up running cool.
- **Relaxed but ready, skips as pauses, response rate first** [David 2026-11-06]:
  - "i dont want the hardware stressed out, i just want zero latency from having to get its attention. it should
    be relaxed but ready to act instantly without any distraction." So the measure is wake-up latency, not
    clocks, and the cure is never busy-spinning.
  - "skipping a step in a simulation thread is fine as long as A. it happens to the farthest, litest thread first
    and B. its just a pause. theres nothing wrong with npcs drawing a blank, real people do that too, as long as
    they recall what they are doing once they 'snap out of it' once the hardware has a second." So under
    pressure the World sim gives way first, then the Intermediator, the Local sim last, and a skip never loses
    anyone's state. This replaces the global "drop anything over 250 ms" rule inherited from Bevy.
  - "we should definitely be sequencing with priorities as we establish our grip on the hardware, almost
    focusing on our 'response rate' instead of framerate first. it may not matter now but it can never be a
    hinderance and may save our bacon down the line. its also just orderly." So input-to-photon latency is a
    first-class number, next to frame times.
- **Freeze when not in front; borderless fullscreen; the cap just under the top; the PC's attention while in
  front** [David 2026-11-05, with Builder, after build 62's window found the laptop's screen is variable-refresh,
  so it follows the game and can't be the clock]:
  - "i'd rather have the game just freeze (deliberately) and stay there". One greyed "Paused" frame, nothing
    drawn after it, the simulation stopped, a click to resume. "i can't think of why someone would want it
    running blindly." *Counsel:* that's for single play; co-op, if it comes, will need its own rule.
  - "yeah, borderless fullscreen is probably the best." The default, with F11 for a window.
  - The cap just under the screen's top rate (237 on his 240), with uncapped as the headroom reference: "if it
    ever starts failing to keep up at 237 or 240 or w/e, we stop and assess."
  - "a full screen only thing that tyrannizes the PC, as long as people can at least tab back to discord
    quickly". While in front: a 1 ms timer, the frame thread at high priority, Windows' power throttling off,
    all given back when the game isn't in front. Holding the card's clocks up stays out unless measurements call
    for it ("sounds like a catastrophe waiting to happen").
  - The frame's clock: the processor's own counter, with fixed beats, a high-resolution timer, and every
    wake's lateness measured.
- **Nothing temporary; start from the far view** [David 2026-11-05]: "lets just make sure we're not building
  'tools for seeing the world again'. we're now building what the player will see, and nothing is temporary.
  we'll have to discuss carefully what we start with, thinking about rendering and shading all over again, and
  where to begin. i was thinking almost find an adaptive resolution method that lets us start straight from the
  far view - we'd be blurry up close, but, equally and *correctly* blurry at a distance. rendering the near view
  isn't a hard problem, the hard problem is the far view, and how it blends into the near view - i think once we
  have a really clean, synced, controlled, sensorized, stage, we'll be able to experiment with some rudimentary
  methods, and there may yet be a way to produce a truly elegant bespoke solution. i haven't given up on that
  hope. and i think we did not invalidate it as a possible route last time, because we had so many conflicts and
  such a messy program even in spite of our dedication to rigor, diligence, and non spaghetti code." So
  `regroup.md`'s phase 1 (a height field to be thrown away or kept) is withdrawn. Nothing visible is built until
  it's talked through with him. The stage's instruments (the ballast, the logs) aren't graphics and carry on.
  *Counsel:* the record backs him. Build 60's card time was within reach (3.7 ms standing); its stutter was the
  processor's, and its costliest band was the far copies marched cell by cell. The unified method wasn't shown
  wrong; the program around it was.
- **The recipe on the card before the first sight** [David 2026-11-05]: "yes, installing the hologram onto the gpu
  comes before cracking the eyelids, agreed... i am as interested in getting this engine tuned on the dyno as i
  am in the content for the time being. this is a Koenigsegg style game." So: lanes, then the recipe on the card
  (phase 2), then the first sight.
- **A beautiful world, squinted** [IDEA, David 2026-11-05]: "imagine if we had a slick AAA engine doing all the
  tricks in a world like this at 4k and it was really polished: now imagine that we set the resolution to
  640x480. or maybe a bit higher. i believe *that* would be the correct look for the distant terrain; a beautiful
  world, squinted... can we render a complete view of the world, that is high fidelity and low resolution?...
  could we draw the screen normally at a very low resolution, showing everything, and then just draw everything
  bigger than a pixel with some other method over top of it?" *Counsel:* two passes of the same march, split at
  a distance: the far pass at low resolution with its cells sized to its own pixel, the near pass at full
  resolution stopping where the far one starts. At quarter resolution each far pixel costs about a sixteenth, so
  the far view can afford far more care per pixel. It's his "resolution curve warped by the distance" in its
  simplest form. At 2560×1600 and 100°, a block fills one far pixel at about 335 blocks at half resolution and
  about 168 at a quarter. Builder to assess: the seam on the ground, crawl in motion, and crisp or smoothed
  pixels.
- **Twin layers, not LOD rings** [David 2026-11-05]: "when we talk about far pass and near pass, i get scared we're
  talking about LOD rings again, which i've come to have a distaste for. i would really like to think of the
  low-res hi-fi layer and the hi-res high-detail (not exactly hi-fi) layer as two versions of the same thing; or
  even as conjoined twins. they should agree from the start like two particles of a decay that are in a state of
  quantum entanglement - no matter the distance, the twin engines will always do the same thing; they're just
  aimed differently." The grids nest "like the cellographs disney studios would use", each coarse cell exactly
  2×2 or 4×4 fine pixels, and "as long as each cell knows what it needs to be at all times... the far being
  cheap, and the near being dynamically allocated - past a certain point though, it should stay about the same
  volume of the screen as the game is built out." So:
  - one function answers what a patch of world looks like at a pixel's footprint, and both layers call it;
  - the coarse layer is an exact fraction of the screen;
  - a standing test proves that each coarse pixel equals the average of its fine pixels;
  - each coarse cell is flagged for fine pixels by what it holds, not by a fixed distance;
  - the fine layer has a budget with priority, and what doesn't fit stays coarse for that frame.
- **The go, up to the first sight; progressive blend curves** [David 2026-11-05]: "i've read builders notes, and
  agree. i think we can get to work on the stuff we need to do before we first take a look at low-resolution
  hi-fi bottom layer." And: "whatever time it takes us to build a *killer end-game graphics engine*, what good will
  it do us to waste time?" So Builder works through lane 0.2, phase 2 and 3.1 in `regroup.md`, and stops before
  3.2, the first sight, to talk it through with him. The milestone: "spawning in to the new vision system and
  seeing it look like a dos game resolution, *but it's all there, dynamic, honest, and unflawed* would be a huge
  milestone. after that, we just build in the other side of the twin engines." On the switch between twins: "i
  think we could do good work by putting threshold and blend curves at the exact pixels which are flipping from big
  to small. almost how the more advanced multistage shocks for baja trucks increase their damping across their
  damping range with valves." So the blend is a shaped curve in a few tunable stages, not one threshold.
- **The whole recipe in whole numbers** [David 2026-11-05], after Builder found the conversion bigger than planned:
  plain fractions also differ between card and processor, so about 400 fraction uses, and phase 2 at 15 to 25
  of her hours. "move to whole numbers. otherwise the hologram breaks. as long as the behavior is qualitatively
  identical in its procession, it doesnt matter if worlds come out different, so long as the rules never changed.
  its going to be fine tuned again anyways. i will put my faith in builders assessment on this. we arent counting
  cost in time, only in the finished products capability." So the bar for the conversion is the same rules
  behaving the same way, checked by his look at his default presets; worlds may differ in detail. Processor and
  card must agree exactly. The aim, in his words: "a 1 size adaptive solution that we can stay coherent with as we
  fill it with stuff."
  - **His look passed, build 66** [2026-11-06]: "got it fixed, whole numbers look the same now, and it went back to
    10 seconds for rivers." Build 65 had shown a grainy roughness: the crags' strength read from a slope taken as
    the difference of two rounded heights. Build 66 reads the slope from the curve directly. Builder's lesson:
    wherever the recipe takes a slope, whole numbers compute it directly, never by subtracting.
- **The far levels baked exactly, once per world** [David 2026-11-06]: "i think brute forcing things into cache in
  the beginning is an acceptable cost for getting fast honest good graphics." Builder measured the card awake at
  about 47 million columns a second (25 million with blocks). So the store's near levels (cells of 1 to 16 blocks)
  follow the camera, filled live from the recipe. The levels from 32-block cells up are baked once per world by
  running the real recipe under every column (about 2 to 3.5 minutes for a 72,000-block world on his card), patched
  by edits, and saved with the world. The cells are 3D, holding caves and overhangs. Only mixed cells are stored, at
  about 16 bytes each: about 300 MB for that world, more with mountains. Builder's estimates, to be measured.
  - **Refinement in sight is fine, if predictable** [David 2026-11-06]: "i don't really mind the idea of massively
    overpixelated looks as it renders in; i think players seeing the software work in a readable way that slides
    right towards what he sees as 'right' is the kind of thing that doesn't irritate people, as long as its very
    temporary and predictable. if they don't know what to expect it'd become a constant annoyance. i think it can
    be done gracefully if necessary." So a fast first pass, with one true recipe sample per 32-block cell over the
    whole world (about 0.2 s), can be replaced by the exact bake, outward from the player: very temporary, always
    in the same order, easing into the true average rather than snapping.
  - **The start reach, and the go on 3.1** [David 2026-11-06, to Builder]: "an eighth of the world being
    immediately present as the rest fill in would probably be beyond visual range for most circumstances." So the
    start bake covers about an eighth of the world round the spawn (about 25,000 blocks square). On his card that's
    about 55 s with colours before play, and the rest refines in the background in about 12 minutes. On slower
    cards the start reach is a setting that scales with the card. And: "go ahead and proceed. we have that
    checkpoint if we make a mistake. but think about what parts... might be good and bad and be prepared to
    telescope back and forth between the checkpoint".
  - **Clear days see the whole world** [David 2026-11-06]: "fog really would bring the world in, but on clear days
    from a high point you should see across the entire world. if it takes a second for the resolution to sharpen
    and it looks right, almost like sliding the ring on a nice lens for a camera, i don't think that's a big deal.
    i said 8 miles, or 1/8 of the world around you, to snap into when you spawn". So fog comes with the weather,
    not as a fixed wall, and the whole world must be drawable.
  - **A loading screen for players; the raw sharpening for David** [2026-11-06]: "yes, we can just do a loading
    screen. but id like to see the sharpening myself, realtime and raw, until we are doing finishing touches".
    The start bake takes about 46 s on his card, once per world. Our builds skip the wait and show the bake
    filling live.
  - **The first sight, first look** [David 2026-11-06]: "8x8 actually looks good for the far distance, on this
    screen." Then: "i think the contrast on the mountains colors is more of the reason its hard to read at 8x8,
    but, it looks nice at 8x8, and frankly the farthest mountains may not benefit from more... the mountains at
    that extreme will probably look better with forests on them and stuff. there's a severe ringing effect on flat
    surfaces or water beneath me that has to be taken care of but i'm liking what i'm seeing. i think we may be
    able to make this work". The ringing is a flaw to fix; the colours' contrast is noted for tuning.
  - **Bands where each scale looks best** [David 2026-11-06]: "i think 1 - 2 - 4 - 8 may be good. since what im
    seeing is good, the question is, can it be fast." And: "2 looks worse than 4, up close. 4 looks worse than 8 up
    close at a different range. so theres definitely a curve of bands at distances where each scale is
    preferable". *Counsel:* a pixel looks best when it covers about one store cell; pixels finer than the cells
    only magnify the blend. So the screen pyramid (8, 4, 2, 1) splits a cell only while its pixels are bigger
    than the store's cells under it, and the finer store levels earn the near view its smaller pixels. Builder to
    assess.
  - **No mismatch at all** [David 2026-11-06]: "im not sure the change is very obvious. 2x2 is still quite fine on
    screen, and there really is no mismatch at all, which sparks joy i think pixels pump back and forth between the
    scales will seem much more like film grain than anything like the jarring contradictions the previous engine
    had".
  - **Fail like a lens, never like a computer** [David 2026-11-06]: "the artifacts in this view seem
    verasimilitudinous, almost like the eye sees it as dirty glass rather than 'what am i looking at on a computer
    screen'. an organic kind of jacked up." *Counsel's reading, as a rule for every compromise:* where something
    must be imperfect, let it look optical (blur, grain, haze, a lens slightly off focus), never digital (steps,
    pops, seams, holes, shimmer, contradictions).
  - **Gliding edges, kept** [David 2026-11-06]: coarse pixels' edges move "like ps2 era antialiasing. i recall it
    from original half life on a crt." Kept as character: "i like it, its organic and like you said, the human eye
    infers detail where there is none, for that reason". Each coarse pixel's shade follows how much an edge covers
    it, so edges glide through the fixed grid in tone, without stair-stepping.
  - **The slope test, by eye** [David 2026-11-06]: "after having seen what it actually looks like, i'm not
    concerned about the slope test." The ziggurat worry is answered by his look. The test stays a guard, but isn't
    something to chase.
  - **Blocks as blocks near the eye** [David 2026-11-06, build 76]: up close the averages were a fog ("the blocks
    up close exist only as clouds"). Out to where a block is one pixel, rays now walk block by block and stop on
    the first face, hard and lit by its own facing; past that the averages are the truth and are read as before.
    His look: "this honestly looks amazing... the things that used to be glaring problems before are silky smooth
    now... its so accurate and the fade is cromulent. we are really on to something". About 1 ms a frame.
  - **Inside rock, not a concern** [David 2026-11-06]: "don't worry about flying through the rock and the crazy
    average-world, i was just telling council how interesting it was." No rule is made for what the eye sees
    inside solid ground.
  - **Look first, then the dyno** [David 2026-11-08, after build 83]: "we are still installing the crankshaft on
    the holographics engine. i want to get it looking rightish and as fleshed out as possible before we put it on
    the engine dyno and begin to, in a scientific way, first thoroughly inspect our renderer and how it uses the
    gpu - look for ways to sort rays by length and match them to warps of similar length to avoid workers waiting
    for example - and then begin adding power through forced induction until we hit our goal and it doesnt blow a
    headgasket. things like taa i think can be applied artfully to great and even mission critical advantage";
    "the more fully featured it is before tuning the better we can tune and then sculpt the power curve with
    induction". So
    the speed deep dive has three parts, in order:
    1. **The look**, as fleshed out as it can get first.
    2. **Inspection**, done scientifically: how the renderer uses the card, measured before anything changes.
       This covers warps waiting on their slowest ray (divergence) and on memory, Nvidia's Nsight Graphics, and
       a view of each tile's slowest ray against its average. One idea to try is sorting rays by length so a warp
       holds rays of similar length.
    3. **Adding power**, one measured step at a time, until the goal is reached without breaking anything.
       Temporal reuse (TAA/TRAA) is a main candidate: last frame's colours, and rays starting just short of where
       last frame's rays hit. David's rules for it:
       - Anything that moves fast or unpredictably (animals, monsters, people, effects) never takes from history:
         "for everything else it would look fine or good".
       - Pixels with no history are known before any ray is shot, and get rays first from a fixed budget each
         frame: coarse at first, sharpened over the next frames ("fast is slow, smooth is fast").
       - History is dropped wherever a block is edited or the light on a place changes.
       - The hard block faces near the eye stay crisp.

       **Reference, David's pick** [2026-11-08]: Lasse Jon Fuglsang Pedersen, "Temporal Reprojection Anti-Aliasing
       in INSIDE" (Playdead, GDC 2016, youtu.be/2XXS5UyNjjU). He said Playdead would release its source on GitHub.
       What carries over to us:
       - **Jitter.** Each frame shifts the view by a sub-pixel offset: the first 16 points of the Halton (2, 3)
         sequence, chosen by eye with a magnifier. For us that means the pixel's ray offset and our step phase
         (`scatter`).
       - **Reprojection.** The hit position is reprojected with last frame's camera. The still world needs no
         per-object motion, and movers are tagged. Each pixel takes its motion from the nearest-depth neighbour in
         its 3 × 3, so edges travel with the thing in front.
       - **Neighbourhood clipping** (after Karis). History is clipped to the box of colours in the current 3 × 3,
         towards its centre. This was their cure for ghosting, where depth and velocity tests proved fragile, and
         it also works through transparent layers. For us it catches most light changes and fog without special
         rules.
       - **A high feedback.** About 90% history, so old information fades within a few frames.
       - **Motion blur as the fallback for fast pixels.** It starts at 2 pixels of motion a frame and is full at
         15. It goes into the output only, never into the history, which fits "fast is slow, smooth is fast".
       - **Noise soaked up.** Sparse random sampling (their occlusion, reflections, volumetrics) came out smooth.
         For us that means fewer sky cones and shadow samples with rotating jitter.
       - **Cost.** 1.7 ms at 1080p on an Xbox One, not optimised, built over about three months of a production.

       More for the list, scouted with David the same day ("we are not holding anything back"). Each is measured
       before it is kept:
       - **Far pixels refreshed only when they've moved** (David: "distant terrain moves incredibly slowly and i
         believe we can mathematically induce how often it actually needs to be checked"). A surface slides about
         movement ÷ distance across the screen, so a far pixel is re-marched only once its last ray would be off
         by half a pixel.
         - Turning is pure reprojection with no parallax, for example a slowly refreshed panorama of the far field.
         - Far changes come from the simulation, which tells the renderer which places to refresh.
         - Each pixel keeps where it hit and its facing, so a light change re-shades it without re-marching:
           marching decoupled from shading.
       - **Warps kept alike.** Sorting rays by an estimated length (from last frame's depth or the coarse layer),
         or compaction: march a fixed number of steps, pack the rays still going into a dense list, repeat.
       - **Warp intrinsics** (WGSL subgroups, an optional wgpu feature): a warp votes to take a big stride
         together, shares a fetched cell, or takes its nearest start in one instruction.
       - **Jumps solved, not marched.** A ray drops straight to the height of the tallest column below it; it
         meets the sea's flat level in one sum. **Horizon maps** store each column's horizon angle in a few
         directions, once, which turns far sun shadows into a lookup and nearly gives sky light too. That could
         replace the shadow march and the sky cones over far terrain.
       - **Tiles sorted by type** (David: "classify screen tiles into different types that dont access parts of the
         hologram they dont need"): sky only, far only (baked levels only), near (block walk), water, seam. Each
         type gets its own lean shader. That frees registers, so more warps fit on the card, and rays on different
         paths no longer share a warp.

       Builder reviewed the list on 2026-11-08 (after build 86) and found every idea fits. Some exist already in
       spatial form: the pyramid starting finer rays from their parent's touch, the per-tile cone start, and
       column-top skipping. Nothing temporal, sorted or analytic is built yet. Her catches:
       - Compaction writes a lot of ray state between passes.
       - We're at wgpu's default of 8 storage buffers a stage.
       - Subgroups need one adapter check; our DXC path supports them.
       - Temporal starts need a back-off and a minimum over neighbours, because of disocclusion.
       - Horizon maps are 2.5D, so caves and overhangs keep their cone rays near the eye.

       Added by Builder:
       - Rays end where fog is total: 99% by about 5,000 blocks at sea level, against a reach of 150,000.
       - Relaxed cone stepping: each column keeps the widest empty cone over its top, for grazing rays over flat
         land.
       - An empty-space distance per cell, for caves.
       - The sea as an analytic plane: the worst case left is 610 steps at the horizon.
       - A two-level start pass (64 × 64, then 8 × 8).
       - f16 accumulators.
       - Workgroup shape and order (8 × 8 against 16 × 4, Morton order).
       - Fork E's remnant: a checkerboarded coarse layer with temporal fill.

       Added by Counsel:
       - Light the hologram, not the screen: sun and sky light kept in the cells, re-lit gradually as the sun
         moves, so rays read lit colour and shading is shared across pixels and frames.
       - Timewarp, as VR does it: the last picture re-aimed at the newest mouse position just before it's shown,
         so turning feels immediate whatever the frame rate (David's "pleasurable just to move the mouse").
       - The store's layout: smaller cells and cells ordered so neighbours sit together in memory, for fewer
         memory waits.
       - Blue-noise jitter for the temporal phases.
       - Ray-tracing cores as an empty-space skipper (boxes around surface bricks), if wgpu's ray queries ever
         allow it.

       **Idle hardware, and the processor as a partner** [David 2026-11-09, reading about Nsight: "i wonder if
       we'll be able to find a way to use any hardware bits that aren't utilized to assist others"; and "we should
       look at opportunities for cpu acceleration of the graphics... we may be able to hand off solutions to the
       card where it's expedient, without the cost of a 2 way trip"]. The candidates:
       - **Texture units.** The march blends 8 cells by hand. Stored as bricks in a 3D texture with an index,
         cells would be blended by the texture hardware for free, and read through its cache, which is built
         for neighbours. This is portable, and it touches the store's layout, so it's block work.
       - **The rasteriser**, idle in a compute renderer. Drawing boxes around the occupied bricks gives each
         pixel where its ray can first hit anything, so empty air is skipped by hardware that is otherwise
         asleep.
       - **The gaps between frames**, the card's idle time, filled with background work such as the bake.
       - Tensor and ray-tracing cores stay out, under the rule below.
       - **The processor runs one way, never round trips.** While the card draws frame N, the processor
         prepares frame N + 1 and uploads it, so the card never waits:
         - light floods, edit patches and lists of what changed;
         - the regions the camera is heading into, predicted from its speed, so their bake comes first.
         Anything the card reports back is read a few frames late, never waited for.
       - **The laptop's catch.** Processor and card share one power and heat budget, so heavy processor work
         can lower the card's clocks. Every handoff is measured on the dyno.
       - **Texture units and the rasteriser are to be looked into first** [David 2026-11-09: "we need to look
         into ways to exploit texture units and rasterizers to get all the work done more instantly each
         frame"].
       - **No idle lanes** [David 2026-11-09: "you know how a limiter pumps a line between two points according
         to load? we should try to make all 32 workers in a warp... immediately do whatever else we can get them
         to do when they start waiting"]. This works at two levels:
         - **Lanes:** with persistent threads, a lane whose ray has finished takes the next ray from a shared
           queue, so the warp stays full until the queue is empty ("dynamic ray fetch", Aila and Laine 2009,
           *Understanding the Efficiency of Ray Traversal on GPUs*).
         - **Warps:** while one warp waits on memory the scheduler runs another, which needs enough warps
           resident (occupancy, so lean shaders).
       - **The processor knows the mouse first** [David 2026-11-09: "i bet the cpu gets the mouse way before the
         card and can send the anticipation ahead to prioritize work in taa as well for the clip masking"]. The
         card learns where the camera is only when the processor tells it, so the processor can send ahead:
         - the camera's turn and speed, so the strip coming on screen (pixels with no history) gets its rays
           reserved first;
         - the latest mouse at the last moment before the frame goes (late latching), and again for timewarp
           just before it's shown.
       - **The mouse by the short hairs** [David 2026-11-09: "we really should make sure we have human input
         hardware by the short hairs, theres nothing worse than a vague mousefeel, and if we have it tightly, we can
         mess with it for fun, like when you get hit or sick"].
         - Already in place: the runner reads the mouse's raw movement (winit's device motion, so no Windows
           acceleration), and build 67 measured input to photon at 1.05 ms uncapped and 4.65 ms at the 237 cap.
         - To keep it tight: input to photon is a gate on every dyno build, with no smoothing unless chosen, and
           the mouse read as late as possible (above).
         - Once the feel is owned exactly, it becomes an instrument. Deliberate changes to it, made on purpose and
           never by accident:
           - a jolt when hit;
           - drift and sway when sick or drunk;
           - weight when exhausted (the stamina sphere);
           - nodding off when awake too long [David 2026-11-09: "having the mouse drift downwards when youve been
             awake too long would be fun, youre nodding off"]: a slow sag of the view, perhaps snapping back up
             (the energy sphere). A yawn tells a confused player what's happening the first time
             [David: "a yawn would clear it up for any players who get confused the first time it happens"];
           - a tremor with fear;
           - the inertia of a heavy weapon.
           Each has a strength the player can turn down. *Counsel.*

       **No per-card tricks** [David 2026-11-08: "im not interested in implementing all the different tricks
       individual modern cards can use, id have to own 20 gpus"]. Speed comes from the method, in portable
       software that runs on any card wgpu supports (the 980 Ti, AMD, the Steam Deck). One maker's hardware
       features (ray-tracing cores, DLSS, SER, variable-rate shading) are not pursued. Optional wgpu features
       that most cards share, such as subgroups, are fine as long as there's a path that works without them.
  - **What light needs** [David 2026-11-06]: "i dont think i care about reflection and bounce. moving torches are
    obviously necessary as well as light from fx. accurate shadows are only necessary up close and i can imagine
    them being that sharp if theres only 16 light levels per block. sunrays may be cooler than i had previously
    thought, if they feel organic. theres a sense you get with software where it feels springy and it becomes
    pleasurable just to move the mouse". So:
    - reflections and bounce light are out;
    - moving torches and light from effects are in;
    - sharp shadows matter only up close;
    - sun shafts are wanted if they feel organic;
    - the goal for feel is software so responsive that moving the mouse is a pleasure.
    - "16 light levels per block" means brightness steps, as he clarified: "i mean like the light from the
      brazieres, where its very clear steps in brightness". So placed light falls off in clear steps from block
      to block, about 16 of them, as in Minecraft. Far away, the averaged cells soften the steps. The recipe answering at a cell's size stays on file as an
  experiment, if the bake's time on slower cards or bigger worlds asks for it.
- **The whole, in one paragraph** [David 2026-11-05]: "the hologram is twinned between the cpu and the gpu, and
  the screen is twinned between a gross and a fine screen in real time. and the hard work is done ahead of time
  and can be derived by any part of the whole, in isolation, without reaching out; the entities are all
  entangled. only the actual outcome of fate and chaos has to be communicated across the network, to those who
  need it." *Counsel:*
  - The recipe's twins are exact; the screen's twins agree within a tolerance.
  - "Derived in isolation" includes the plans made once at world creation (rivers, caves, peaks), which are part
    of the hologram's data.
  - What crosses the network is the seed, the rules' version, edits, players' actions and Fate's outcomes, and
    every machine must run the same rules.
- **Caves are seen at a distance** (correcting a line in the first draft of `regroup.md`): "if a hall in the
  mountain or a cave is big enough to be seen at a distance, it should be seen at a distance... we need to
  *accurately* represent far terrain in a way that blends seamlessly with the near." At the default 100°
  field of view, a 40-block mouth is about 24 pixels tall at 1,000 blocks and still about one at 24,000.

## The renderer: the old engine dropped now, not last

**[DECIDED 2026-11-04]** David: "i feel as a mechanic the bilateral engine transfer we are doing would
create a real mess in my lift bay. we got rivers working by ripping them out first." Then: "we culd just
remove out eyes entirely and build them again more carefully, the right way. meaning vrr+vlr. i dont think
that code is wrong or should be discarded. i mean lets just drop the old engine and see what happens. go
from there."

So R7 (`docs/far-view.md`) moves from last to now, with no switch kept: the line goes all the way in, the
near meshes and the far sheet come out, and the old engine stays in git history only. Kept: the march and
copies, the real chunks for the game, world gen and the default preset. Water and glass up close, pieces and
bars, models and near lighting may be missing for a while, and come back built for the march, not matched
to the old engine. How they come back is planned after he has seen the result. He won't be near the USB
backup for a week and chose not to wait for it.

**The spec, and going to black [David 2026-11-04]:** "i should be able to run this at 250fps stable with my
cpu at 90c and my gpu at 85c. thats the truth, there are very few detailed blocks that ever need to be shown
and we know almost everything about every block ahead of time in the game. we should go to black and try to
project the cheapest ray that lets us see a block in front of us and the farthest mounting for the cost of
the space ot takes on the screen"

So, on his laptop in Turbo with max fans: 250 frames a second, judged by the 1% low and worst frames too (a
4 ms frame), at 90 °C and 85 °C on the game's corner readout. A lower frame cap isn't an answer to it. This
puts numbers on the first point of the bar below. The method: from a black screen, measure the bare frame;
then the cheapest ray that shows the block in front of you and the farthest mountain, its cost following
the space a thing takes on screen; then everything else added back one at a time, each with its price in ms
and watts. Existing march code is reused where it earns its place, not assumed.

**Bevy, and flight [David 2026-11-04]:** "we need to carefully assess how much bevy does for us and how much
it could impInge on us. i do not expect the computer to keep up with the super fast flight but i dont see
what difference it *must* make... it should be doing almost nothing with nothing on the screen." So Builder
measures the floor (a bare Bevy app, DefaultPlugins, our game with nothing on screen) and lists what Bevy
does for us against what it costs; what follows from that is his call. And the rule on flight: flying fast
may delay when detail arrives, but the frame doesn't wait on the world.

**The spec restated [David 2026-11-05]:** "i should get an unbreakable 120 or a smooth 240 on a nice gaming
laptop", in place of 250 stable: no frame over 8.3 ms in ordinary flight, or 240 (his screen's refresh)
judged by the 1% low. By "a resolution. curve warped by the distance. simple transform applied to the gpus
workload then rendered as screen resolution through a mask using the same warping distance curve the player
views". That is option B in `vision/proposal-voxel-renderer.md`, the resolution curve, not built yet: as far
as Counsel can read, every pixel still gets a ray. Builder to assess it after the floor.

**Far land as a backdrop, and importance [David 2026-11-05]:** "if 4.2 wavers during the boss fight in the
first play through of the game ever while directing two generals of spearmen, im not gonna complain. if the
terrain in the distance looks good but its a Strife or Hexen backdrop, thats enough. as long as he can see
something important from the moment its a pixel to two pixels and it can all be painted right." So the
heaviest moments may dip below 240 (taken as: 120 still holds), and distant terrain only has to look good. The
firm rule: an important thing shows correctly from its first pixel, never dropped or averaged away. Which
things count is his call. Builder to assess a backdrop (the dome in `vision/proposal-voxel-renderer.md`),
how important things survive averaged cells, and many moving figures on the card.

**The low end [David 2026-11-05]:** "980ti should be the price of admission at standard looks". Taken as a
smooth 60 at 1080p on a clean machine ("i am not trying to account for people downloading bonzai buddy"). He
named a 2011 desktop first, then took it back: "oh i didnt know 2011 was the 680 days im sorry no". So there's
a standard look every card from there up gets, and anything past it is extra; what's in it is his call.

**If we part from Bevy, what must survive [David 2026-11-05]:** "it may just be time to go our separate ways
if its doing stuff we dont understand or appreciate". Not decided; the floor runs come first. Three things
he wants whichever way it goes:
- Parallel work: "smart fast parallelization of behaviors is good. if bevy does that *well* we may keep, or
  copy that." Whether it does it well for us is to be assessed.
- Tables that don't know what they hold: "i think hardcoding types of tables is a bad idea, it should be
  completely agnostic so that we can add new stuff without modifications". Whether "new stuff" stops at new
  columns or reaches new rules written in data, without a rebuild, is open.
- The clock: "the timeclock we have works good. we dont want to lose that." That's `src/time/`: fixed ticks,
  pause and speed, and the saved world clock. Its tick loop currently runs on Bevy's `Time` and `FixedUpdate`.

**A stage that takes whatever is put on it [David 2026-11-05]:** "theres no freedom in software except
performance. i cant make a game if the pc starts choking. i need to forget performance is a thing at all and
have a stage that takes whatever is put into it with equanimity. so the real question is, how to arrange the
work so that any relatively modern computer just has a breezy time. like an eating contest pro doing his prep
and using his methods. we have to dip the hot dog buns in water". *Counsel's reading, for him to correct:*
prepare everything possible before play; give drawing, simulation and loading separate lanes that never wait
on each other; give each lane a budget, so too much work arrives late or runs less often and the frame holds;
make cost follow the screen, not the world; and have tests and a price on every feature do the remembering,
so he can forget. "there are [limits] to the world": overload shows as things arriving late or looking simpler.

**How Bevy is judged [David 2026-11-05]:** "i think the method and the results we judge bevy on have to be
considered inseperable". So one judgement, no pass mark on the floor numbers by themselves: for each option,
the numbers together with how this arrangement of the work would be built in it, and where it gets in the way.

**The size of a huge battle [David 2026-11-05]:** "attacking with 100 soldiers against a city of 400 people and
100 soldiers would be the 'huge battle' end of the scale. and that would be a great effort from the player,
after hes good and has been playing for a while." About 600 figures. The ceiling, from a later message: "i think
a max case would be like 800 npcs doing stuff on screen for some epic battle some dedicated player got to
through extremes. not trying to make Total Annihilation". The figures pass and the battle's simulation are sized
against 800.

## The renderer: the processor stays at the floor; everything drawn is the card's

**[DECIDED 2026-11-04]** David, after the watt ledger (the processor's cost was mostly per-frame
bookkeeping, the card's rays the cheap part): "can we start at the point far enough back enough with
visual access to the screen at all, and from there do nothing that increases the cpu's effort, and ONLY
write code that increases the gpu's effort, as we go from the ray's i see you testing right now ... to
an exquisitely efficient rendering engine?" And: "the processor is going to have its work cut out for it
with the simulation ... the cpu sprinting through world gen and feeding it directly into the cards
buffer, and then working out a way to shoot rays that show everything equally and costs the same at
any distance in a mostly empty world would be a good baseline to A. add detail in placer blueprints and
B. operate the game simulation every actor relies on."

- **The floor:** a window, its clear colour and the frame loop, nothing drawn (measured first). The
  processor's work per frame for drawing stays there, whatever is on screen.
- **World gen sprints once** (about 18 s, the processor 75 to 95 °C, accepted: a transition screen
  covers it) and hands the card the recipe once. Nothing per frame.
- **The card makes what it draws from the recipe** (the far land, its cells sized per pixel), so a ray
  costs about the same at any distance and the cost follows the screen, not the world.
- **The one exception, by event, not by frame:** what exists only on the processor's side (blocks dug
  or placed, caves) goes to the card when it changes, a small upload.
- **Every look feature is added as card work**, each with its price in ms on the card (its own 4 ms at
  250 fps, and its heat in the shared cooler counts too).
- **The processor is for the simulation** (DF-deep, written and reworked by Claude across many files
  at once: "the rules that a basically infinitely efficient claude agent can write laterally"), paced
  on its own, not per frame. Detail then comes in through the placer's blueprints (A) on this baseline.

## The renderer: the bar for moving on to content

**[David 2026-11-04]** Counsel's bar, David: "i think you are right with that bar." Done enough to build on:
1. It runs smoothly and stays cool, confirmed by David's own flight.
2. The land looks like one land: far land cells and the seam judged, a reach picked.
3. Built things hold up at a distance: pieces and bars as cells, and an answer for models past the real chunks.
4. Heavy content doesn't change the answer: a stand-in forest confirms the cost follows the screen.

And David's goal past it: "ill feel much better once we have vrr+vlr built and performant so we can move on
and the remaining graphics work is taste and FX". VLR is assessed and planned again before it starts
(`vision/proposal-voxel-renderer.md`); whether it comes before trees or alongside them is decided then.

## The renderer: right and fast together, erring on the side of right

**[DECIDED 2026-11-03]** David, after R2 measured at 348 FPS (`docs/far-view.md`): "my gut feeling
is that we're exceeding our end specs at this point, and i'd almost rather make it perfect at any
cost and then work backwards at the end to make it faster. we don't need to do that, because we're
building it the right way. i think we can defer the weak hardware test for now. the best test will be
having a pre-alpha build fun enough that 10 people can play and send us the report file back on their
hardware and all its performance across several machines, so the sooner we get to that point the
better."

David, the same day: "we *can* get it looking perfect *and* being fast at the same time. we're
already doing it, its just a 2 steps forward 1 step back process. don't say that 'the look comes
first and speed is won back at the end', because we are perfectly capable of achieving both
simultaniously, i'm just saying i'd err on the side of getting it right."

- Looking right and running fast are both aims at every step, together. When a step has to choose,
  err on the side of getting it right.
- **[LATER]** The weak-hardware checkpoint is put off (David travelling this week). The real test of
  hardware is a pre-alpha build: about 10 players on their own machines, each sending back a report
  file with how it ran (the session log already holds most of it).
- **[OPEN]** What the report file holds and how players send it.

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

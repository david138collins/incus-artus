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
  - **[DECIDED 2026-10-26, corrected 2026-11-10] Spelled *Artus* on purpose, not *Artis*.** *Incus* is
    Latin for anvil, and *Artus* means craft: the anvil of the craft. Strict Latin would be *Incus Artis*;
    David prefers the u. David 2026-11-10: "note that artus is craft, not arthur. i do draw from arthurian
    legend but its incus artus and not incus artis because i just prefer the u there." (An earlier note here
    said the name was told publicly as an echo of Arthur; that was wrong. The Arthurian legend is one of his
    sources for the lore, not the meaning of the name.) Older notes and the inventory mockup that say *Artis*
    are out of date.
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
    - **[IDEA, David 2026-11-16] The falling side's branches stack to the heavier side:** "i think the
      branches on the side of the tree it falls on can simply stack off to whatever side it's heavier
      towards, onto its side. the question is how to precedurally animate their reverse-bounce to their
      resting locations when they snap." (Newer than his 2026-11-01 "the branch folds before the tree hits
      90 degrees"; old words weigh less.)
      - *Counsel:* the end decided first, as above (each branch's resting place on the heavier side, on its
        side, on the octant grid, worked out at the last cut), then a path toward it with no physics engine:
        a hop as a parabola from where it snaps to where it rests, then one or two bounces each a fraction
        of the last (a bounciness number); its turn onto its side eased by a damped spring, which overshoots
        and swings back once, the "reverse bounce" (a stiffness and a damping number), rocking about the edge
        it lands on. Snaps staggered a few frames apart, trunk outward, for a crackle. With his Quake note, the
        same path could also be shown as a few held poses, snapping. It belongs to the shared kinematics layer
        (spec), and the branches are rigid pieces, as the figures are.
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
- **[DECIDED, 2026-11-15]** Figures and objects are built at **32 units to a block** in Blockbench, one unit a
  pixel. David asked "is there a way to set blockbench to do 32 pixels instead of 16", and on Counsel's
  suggestion: "Yes, 32 a block".
  - Why: the block faces are 32 × 32, so models match the ground they stand on; and Blockbench's defaults
    all line up (a step of one unit moves a piece one pixel, and the paint is a pixel a unit with no setting
    changed). A person a block and a half tall is 48 units.
  - Blockbench's glTF export counts 16 units to its unit, so the game halves what it reads: one number,
    set once, for figures and objects alike. This replaces "as made" at Blockbench's 16 to a block
    (objects.md) for new models; the brazier, made at 16, is scaled up or read at its own scale.
  - The other way, 16 units with textures twice as fine as the shapes, was turned down: set for every
    texture, and shapes snapping to every other pixel.
  - To check before relying on it: a 32-unit cube, painted, shows 32 pixels a side, like a block face.

- **[DECIDED]** (Step 7) Blocks are defined in `assets/data/<pack>/blocks/<name>.toml` → ID `<pack>:<name>`; the
  base game's pack is `incus`. Only air is built in. Format and settings: `docs/data-files.md`.
  `look = solid | grid | outline` sets the usual behaviour; `solid`/`targetable`/`replaceable` can override it.
- **[DECIDED]** Misspelt settings are errors (reported with file and line); unknown face words are warnings.
  Mistakes never crash: a bad file keeps the block's last good version; a missing test-world block gets a magenta
  placeholder; a deleted file's block stays until restart (block numbers never change while running).
- **[DECIDED]** Live reload covers data files *and* block texture PNGs (checked every second, real time).
- **[LATER]** A "write this tweak into the block's file" button on the Blocks page (keeping comments).
- **[DECIDED 2026-11-14] Ship the engine, not only the game; nothing about the content hardcoded** (David, to
  Counsel): "i have some thoughts about how the editor should work; i really think it should be fully featured
  enough to be extended without touching the code itself. that means nothing about the game content is hardcoded,
  like npc ai's and vfx. it gets complicated to make it extensible with a gui, and im not sure if there's some
  specific type of scripting we should have the editor take for some sections. Not for me, because i'd just have
  Builder extend the editor to give me the ability to use the gui to add behaviors or items or fx, but in the end,
  i'd like to deliver not only a game but the full game engine for people to do things of their own with, and having
  the ability to extend the given game mechanics with their own scripts could give it an order of more value to the
  world over the years".
  - *What it moves on from:* 2026-10-30's "scripting without a scripting language, at first" (`vision/toy-factory.md`:
    the editor's wiring is the script, a real language only if wiring stops being enough). Shipping the engine for
    others to extend the mechanics is that case, so a real language is now planned, at the edges.
  - *Counsel's recommendation,* for the content's first systems: (1) **the game is the first mod**: everything of
    David's (items, people, behaviours, effects) comes in through the same door a modder's pack does (old already
    had the base game as the pack `incus`); (2) **three layers over one store**: data files for things, the
    editor's wiring for behaviour made of pieces, scripts for new pieces, which then show in the editor's menus like
    built-in ones; the editor and the files are two views of the same thing; (3) **scripts decide, the engine does
    the heavy work**: the simulation and the drawing stay Rust and WGSL, scripts run at events and choices, never in
    the per-block, per-frame loops; (4) **sandboxed**: a stranger's pack can't read files or reach the net;
    (5) effects as pieces with settings first, the card's own code from strangers much later if ever.
  - *The language, open:* Counsel's pick is **Luau** (Roblox's open, sandboxed, optionally typed Lua; the Lua family
    is the most widely known modding language: Factorio, World of Warcraft, Don't Starve, Garry's Mod), with
    WebAssembly as a later door for heavy mods in any language. Chosen when the first behaviour needs it.
  - *Beside it:* at the regroup he said "the editor is for me until the best use of our time is serving eager
    modders. thats a ways off." That still holds for now; this is where it ends up.
  - **[DECIDED 2026-11-14, later the same day] His recipe first; room for modders, not modders' tools** (David, to
    Counsel, on the game being the first mod): "i like how you think with your point in 1. although, it will take
    some scaffolding to get there in the end. i do think we should make the world gen work this way, as a modular
    thing; it does not need to work that way at first. priority 1 is shipping incus artus, *my* recipe; but i think
    we should assemble it so later we can expose parts of it for modification - even if the steam release doesn't
    expose any of our own code or is 100% modular. we can wait on that, so i just want to architecture that later on
    can be set up for modders at a later date, once many people are playing it."
    - So the entry above is the end, not the next step: nothing is built for modders now (no scripting language,
      no modders' documents, no open stages); each system is assembled so its parts *can* be opened later. Put in
      the plan's "What every step keeps" as "Room for modders, later".
  - **[IDEA 2026-11-14] Modders' own layers in world gen** (David, straight after): "like, what would we need to do
    to let people create their own topographical layers in the world gen". *Counsel's sketch:* the stages listed in
    data, not in code, so a pack can put its own between two of David's; each stage declaring the named fields it
    reads and writes (height, rock, water, and new ones of its own), only ever from earlier stages (the linear
    pipeline); a stage built from the editor's pieces without code (noise, masks, curves, stamps painted as
    pictures), or a script that decides where and what while the engine's pieces do the per-column work; seeded,
    so a seed always makes the same world; each stage's result shown on the World Gen page. David's own stages go
    through the same door. Cheapest if world gen is rebuilt that way from the start.
  - **[OPEN 2026-11-14] Will others be able to read it?** (David): "i have high confidence i can ship software with you
    guys that works exactly how i want it to. i have less confidence that the code will be something someone could
    actually look at and use like we do with minecraft, or if it'll be alien technology. i just don't know".
    *Counsel:* most modders should never need the engine's code: Minecraft's own code was scrambled for most of
    its life, and its modding came from doors the community built; Factorio's modders never see its engine, only a
    well-documented door. So the door (data, the editor, scripts, their documents, the base game as the example)
    comes first, and the engine is kept readable for the few who go deeper. Its known rough edges for an outsider:
    the far eye's speed tricks (dense in any engine) and the hand style no formatter knows (a formatter config
    matching it, or one reformat at a quiet moment, to settle before outsiders read it). A way to know rather than
    guess: **a newcomer test**, a reader with no history (a fresh session now, a human Rust programmer later) asked to
    find and change one thing from the documents alone, its stumbles fixed. A cousin of the fresh-eyes audit after
    the dyno.

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
- **[DECIDED]** Bevy, latest stable, version locked. *[Replaced: the regroup of 2026-11-05 cut it to a scheduler;
  on 2026-11-11 David: "we're leaving bevy entirely, it just helped bootstrap us."]*
- **[DECIDED]** git for history, GitHub (private) for backup. *[2026-11-11: David made the private repository
  `incus-artus` on GitHub; not connected yet.]*
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
- **[DECIDED 2026-11-10] Four roles, the truth and the journals** [David, stepping back from the dyno to
  "build a better way of working"]:
  - **The roles:** "ok, so Builder, David, Counsel, Oracle." David holds the vision and makes the calls. Builder
    builds, free to focus on the technical constraints. Counsel doesn't hold the master plan in its head but
    "only observe it, protect it, inform builder on it, and work with me to conceive of updates to the master
    plan." The Oracle is turned to for guidance: "to look at the structure, and find a more elegant form that does
    it with less, and question the assumptions and axioms."
  - **The truth, apart from the journals:** "we need a true, live architectural viewer of the software. like the
    view that power plant generator operators see on their screens. this is seperate from the logs you have been
    keeping in docs, which are more like dev logs or journals. so we need living documents that are the truth, and
    reflect that instantly and reflexively, so everyone is on the same page. this could take the form of the
    master map, along with a master plan, as a map for where we have decided to work towards." And: "we need a
    ship list, or the objective spec for the game." So three living documents: **the master map** (what exists
    and how it's wired, kept true by the build itself), **the master plan** (where we've decided to go) and **the
    spec** (the game we're shipping). The journals (`decisions.md`, `dyno.md`, `regroup.md` and the rest) stay as
    the record of how we got here.
  - **The plan lives in files, not in a session:** sessions are summarised and forget; files don't.
  - **On a rewrite** (being weighed, with the Oracle): "nothing that isnt ultimately part of the players joy is
    sacred to me." If it comes, "we could keep 'old incus artus' aside, and only check the rewrite against it":
    the old project as reference and test oracle, each new stage checked against its output.
- **[DECIDED 2026-11-11] Where things are kept** [Counsel's proposal, after David asked "did we establish a working
  principle of memory storage for agents between sessions along with the 'master' documents?"; David: "Yes, record
  it"]:
  - **`master/`** (map, plan, spec): what's true now. Every session reads it first, starting with "Where we are" in
    `master/plan.md`. Kept short enough to read whole.
  - **The journal** (`docs/` and `vision/` today, `journal/` after the packing): how we got here, and David's
    decisions in his own words. Added to, never rewritten.
  - **The agents' memory** (one folder that Builder and Counsel share; its index is loaded into every session): only
    habits and warnings about working with David and this project ("never cargo fmt", "launch with --release",
    "he/him"). Never the plan, the project's state or numbers. A note that says what's true about the project moves
    to `master/`.
  - **`vision/handoff.md`:** short notes between sessions, cleared once acted on.
  - **Session transcripts** aren't a store: they're summarised and forget.

## World Creation

- **[DECIDED 2026-10-25]** A new world is made completely before you enter it, every spring and creek
  included, and saved; nothing about the ground's shape is worked out at runtime. Budget: about
  10 minutes on a slightly below-average PC, shown as a loading screen with the generation drawn live,
  with lore and art. While designing: live sliders, a fast make (seconds) and a full make (minutes).
  Pipeline and reasons in `docs/worldgen.md` ("The pipeline after the landform").
- **[IDEA 2026-11-14] Water after the land's shape, with an easing stage before rivers** (David, to Counsel, for when
  world gen is rebuilt): "thinking about how we do world gen again, we should be able to construct it to get the
  right results in a more streamlined way. perhaps by water coming after topography altogether so rivers could
  generate from natural lakes and ponds as well as from springs, or perhaps, even a topography layer between water
  and rivers that eases/erodes for logical waterways to follow. just a thought".
  - *Counsel's reading, as a straight line of stages:* the land's shape; standing water, every natural hollow filled
    to its brim so lakes and ponds come where the land makes them, each knowing its spill point (priority flood, a
    known fast method); an easing stage that finds where water would gather and gently deepens and smooths those
    paths; then rivers traced from springs and lake overflows along them. Water runs downhill and connects by the
    spill points; the easing does the changing gently in one place, so the rivers barely carve ("the less it changes
    the land, and the more gradually, the better").
  - *Old threads it gathers:* "lakes can feed rivers; ponds only take them in"; springs that reach no river find a
    lake or pond; "pick lakes and ponds before the water moves"; an erosion pass, real water erosion later; the
    webbing for water to follow. Old's rivers were done enough to go on; this is for the rebuild.

---

## The rebuild: after the Oracle

**[DECIDED 2026-11-11]** From David's answers to the Oracle (its session, 2026-11-11) and his reply to its review
(`vision/oracle-review.md`) the same day.

- **Start from scratch, with old Incus Artus as the reference.** David: "i think our plan for the rebuild should
  basically be to use the old incus artus as a reference and go through it step by step. this time i can look at
  the old one and point out the parts that are important, and we can assemble them more neatly this time. some
  structural changes are also going in: so i think its better to start from scratch. even though in most ways it
  will be very much the same." And: "we don't have to cling to the world gen stack we have, because we now know
  the way back; to create is human. to duplicate is divine. you don't know how to do something unless you can do
  it again from memory." *Counsel:* nothing is carried over whole. The Oracle's "port" verdicts become "rebuilt
  with old open beside it"; old's code is copied in pieces only where David or the look says it's right.
- **Experiments first, in old:** "the first order of business is to run some experiments on old incus artus to
  further inform our plans to rebuild it, better, faster, stronger."
- **The eye first, and rigour throughout** (2026-11-11, after experiment 3's surprise): "it illustrates why we need
  to be rigorous and scientific while rebuilding incus artus. and i've been thinking about how important it is to
  start with the graphics solution knocked out of the park. the rest of it is mostly fun and frolicking. getting the
  computationally restrictive and demanding part, that makes such a difference to the game experience, should have
  come first. but, its always a process of iterating on processes. sometimes there's just no way to make a thing
  right the first time and you have to go through prototypes." *Counsel:* the plan already has this order. Phase 2
  is the eye, and it's whole and walked by David before the world is rebuilt. Old was the prototype that found the
  look and made the experiments possible.
- **[OPEN 2026-11-11] The world's format, and old's world gen kept out of the new eye.** A concern raised now so we
  can plan for it:
  - "when we build a new graphics engine from our research, i don't think we should necessarily use the old world
    gen as the way we're going to store and access the world will fundamentally change. i think it would be better
    if we could somehow just take nice25 and convert it the new format we're going to use, and use that for tuning
    the graphics engine, before going back and rebuilding the world gen stack with a finished graphics solution.
    just an idea and a concern."
  - "the new world format is critical to performance. so, we don't really want the new graphics engine touching it.
    we'd need to establish exactly how the format is going to work in a way that lets me expand and refine the
    generation process. we're not quite at that point, but im bringing up the concern now so we can plan for it."
  - *Counsel:* old keeps no blocks (nice25's chunks folder is empty; old's world gen makes every block again when
    it's near), so the plan's "near blocks from old's world" would run old's world gen inside the new eye.
  - The proposal: old exports nice25's blocks where the eye will work before it's frozen. A converter in the new
    project writes them into the new format, and the eye reads only that. World generation is rebuilt later to write
    the same format.
  - In the Oracle's design the eye reads bricks and only the lanes read the tablet, so the format the frame rests on
    may be separable from the one generation grows in.
  - Taken to the Oracle at the fork (`vision/oracle-return.md`, question 7). The export must be decided before
    `old-final`.
  - **[DECIDED 2026-11-11] The export** (David to Builder): "export nice25 so that we can use it as a 'generated'
    world to look at with the new graphics engine, before rebuilding world gen after the graphics engine is done."
    - Built as `world/export.rs` (a7d3cfb): the whole test world as columns (ground, water and a profile number per
      column) plus the profiles, with a README giving the rule that makes a column's blocks. It's the Oracle's tablet
      in a plain form.
    - The test world has no caves and no edits, so the columns are the whole world.
    - Trial over four tiles round the spawn, all four written: every 37th column of one of them checked, all the same
      as old's own; 0.22 bytes a column, about 1 GB for the whole world if all of it were like the land at the spawn.
      The full run writes much less, since much of the square is sea or past the map.
    - Written whole (61f0d4d): 5.2 billion columns in 0.11 GB, 7 profiles, in 8 minutes. Read back one tile in 50:
      104.6 million columns and 1,029,963 whole profiles, none differing from old's.
    - Kept in `exports/` (not in git) and on the USB.
  - Still open, for the Oracle: the new format itself, cut so generation can grow.
- **[DECIDED 2026-11-11] Old's pictures are a reference, not a yardstick** (David to Builder): "still not sure what
  good the pictures are. it'll look good or it won't, idc what the old things look like in particular, other than the
  basic way we're doing things, like pixelating the far horizon." *Counsel:* the new eye is judged by David's eye on
  its own, keeping the basic way (exact cells enlarged crisp, blended far reads, blocky light, honest). Old's pictures
  answer "was it like this before?" when that comes up. The Oracle's roadmap step 4 (the eye whole) set pictures
  against old's within a stated tolerance; that gate is dropped. Any pixel comparison with old is Builder's
  instrument for catching mistakes, not a test of the look.
- **[DECIDED 2026-11-11] Old frozen** (David to Counsel): "so, we'll pack up the old incus artus into old, and just
  use it for looking at and reference." Tagged `old-final` on 7c809d3 and pushed (Builder), after the event log of
  David's quarter-block build went into git (7c809d3; the third standing mark replays it, and it had never been
  committed). The export's USB copy waits for the stick.
  - On the USB the same day (Builder, a78f1f2): `D:\Incus Artus Backups\exports\nice25 test`, all 19,883 files
    compared with the laptop's by SHA-256, none differing. Phase 1 done enough to go on.
- **[DECIDED 2026-11-11] Before the first line: the Oracle called early.** David called the Oracle back after Phase 1,
  before any new code, with Counsel's brief and a prompt of his own on how to start over and what form the engine
  should take first. Its answer is the second part of `vision/oracle-review.md` (63b0eca). Builder read it too, and
  Counsel put the points to David, with Builder's read folded in. David: "i agree on all counts. please ask builder to
  begin the next steps."
  - **A reading flight in old first**, with switches only, so old stays as tagged. It finds where the bare plain's
    cost lives. *Counsel:* the third mark turned out to be mostly a bare plain: 9.51 ms before the build goes in,
    which adds about 2 (flights C, E and F, the settling medians, checked).
  - **The doorway first**, at 2.1, moved up from 3.1. One small piece answers "what's here?" a tile at a time, and the
    export sits behind it. It stays read-only until the fork: looking and flying are enough until then.
  - **The summed levels are the engine's, not the world's.** The far eye's cells come through the doorway from the
    start, summed by a lane. Old's far bake is the check, not the source. David's principle: the engine never opens a
    world file. (Builder had proposed reading old's bake first; Counsel differed, and David agreed with Counsel.)
  - **Test places are data in the new project.** David's build is its 169 placings from the event log; old's chunk
    format is never read.
  - **One change at a time.** Experiments 5 and 6 fly each change behind its own switch, on both cards, with each
    program priced checked and unchecked. Soft bricks are flown two ways: Builder's exact blend (premultiplied, 16-bit)
    and the Oracle's spread. The open-sea shortcut is held until a sea mark shows it's needed.
  - **Rays first for the near eye,** by a rule written before the flight. Triangles are built only if rays miss a mark.
  - **Marks are data,** flown on both cards by one command. The third mark is split into the bare plain and the
    build; the far eye gets marks of its own; a scatter dial gives cost against clutter.
- **[DECIDED 2026-11-11] Old's place, and the control room** (David, to Counsel's two picks: "Yes to both").
  - Old lives in `old\` inside this folder, with its own assets and its own build folder, and still runs.
  - The control room, the live map, is a page in the game's own panel (egui, as the editor is).
- **[DECIDED 2026-11-12] IntelliJ's files go with old** (David, to Builder: "go ahead and do the test, and pack away
  any intellij stuff with old incus"). `.idea` and `Incus Artus.iml` moved into `old\` (c0def46), so old opens in
  IntelliJ as its own project; the new project uses VS Code. The test was the pack's check flight, which held
  (`journal/docs/experiments.md`). The journal moved the same day: `docs/` and `vision/` into `journal/` as they
  were, with `journal/contents.md` (plan 2.1, as the fresh-start proposal set out).
- **[DECIDED 2026-11-12] The new project's build numbers carry on from old's** (David, to Builder: "carry on from
  116."). The skeleton's first launch is build 116 (c4d102e); old's last was 115. One count for both, so a number
  never names two programs.
- **[DECIDED 2026-11-12] The editor's windows arranged on purpose** (David, to Builder): "note that while we're
  re-constructing the editor, which the control panel will be a page in, we should carefully reconsider how we
  arrange and shape windows, because it got a little messy last time, with some settings and lines in the top bars
  of the window becoming pointlessly shared between windows or left after depracated. and organizing the UI is super
  important to me. ... make sure you know as a working principle to ask where settings and controls should go as we
  start filling up the editor screen and stuff."
  - A working principle: before a setting, control or window is placed, Builder asks David where it goes and how
    it's shaped, as with names (her memory, editor-suite-naming). A line in a shared top bar is there only if every
    window under it needs it; a control goes when what it controls goes.
  - *Counsel:* "the control panel will be a page in" the editor. On 2026-11-11 the control room (the live map) was
    to be a page in the game's own panel (above, "Old's place, and the control room"). **[OPEN]** Whether "the
    control panel" is the control room, and so whether its page is in the editor rather than the game's panel:
    David to say.
- **[DECIDED 2026-11-12] The skeleton, looked at** (David, to Builder, on build 116: "looks fine. go ahead"). The
  empty frame was then flown: 0.40 ms a frame at the median and 0.019 ms on the card, with nothing in the world
  (`kept logs/flight-1794486371.txt`; `journal/docs/experiments.md`). Every later change is measured from there.
- **[DECIDED 2026-11-12] The software map, a page of the new editor, and the four words** (David, to Builder, on
  her declaration's shape): "i think the new editor should have a drop-down menu at the top that brings you to the
  desired page (software map, tectonics, world gen, registrar, etc). the software map should just be a black screen
  with magenta boxes that show a tooltip when you hover, connected by lines, and inside of larger boxes for sections
  if need be. the words you're using are fine."
  - **The words** (Builder's, now his): a *lane* is a line of work with its own pace (the frame is one); a *pass* is
    one piece of the frame's work with its own clock, on the processor or the card; a *stage* is one step of the
    world's recipe only; a *store* is something held, with its size and limits. Old's other use of "stage" (the
    march's) isn't carried over, so the plan's "a layer and a stage at a time" reads "a layer and a pass at a time".
    The names in her sample stand (flying, laying out the panel, the panel, the keys held, the camera, the clocks,
    the panel's shapes, the screen's picture).
  - **The control room is the software map**, a page of the new editor, not of the F3 panel. That answers the
    question above ("the control panel will be a page in" the editor): the control panel is the control room.
  - **The editor:** its own screen, its pages reached by the drop-down at the top. *Builder's reading, his to
    change:* old's F2 opens it (the spec's "on its own screen (F2)"); the drop-down lists his four in his words,
    with only the software map working and the rest greyed until they exist.
  - **The software map:** black, with magenta boxes that show a tooltip on hover, joined by lines, inside larger
    boxes for sections. *Builder's reading:* the lanes are the larger boxes with their passes inside; the stores sit
    outside, joined by lines to what reads and writes them; the tooltips carry each box's card, with live times.
    `master/map.md` is still drawn from the same list by the build's test, for the sessions.
- **[DECIDED 2026-11-12] Two desktop launchers** (David, to Counsel's pick: "Yes, add old's"). "Play Incus Artus.bat"
  starts the top of the folder, now the new game, and stays so. "Play old Incus Artus.bat" (Counsel's, the same
  file but for its folder) starts `old\`, with old's own build folder.
- **[DECIDED 2026-11-12] The editor's pages: Tectonic Sim, World Gen, Registrar** (David, to Builder, giving the go
  for build 117): "yes, launch it. we should probably call them Tectonic Sim, World Gen, and Registrar, since
  'compiler/interpreter' would go inside the world gen page. if it even needs an explicit window. we can definitely
  organize the world gen stack better this time."
  - The drop-down's greyed pages spelt so (Builder, 8283179), beside the software map; still build 117, since it
    hadn't launched before.
  - Old's World Compiler and World Interpreter go inside the World Gen page, as a window of their own only if one is
    needed. The world gen stack is laid out afresh at 3.4: its stages in one linear chain, each control placed with
    David.
- **[DECIDED 2026-11-12] The editor and the software map, looked at** (David, to Builder, on build 117: "editor
  looks about like what i'd expect. you can go ahead with your tests"). The wiring's first flight then flew within
  its pass mark: 0.42 ms a frame (the floor's 0.40, +4%), 0.018 ms on the card (`kept logs/flight-1794487642.txt`).
  With each pass timed, handing the frame to the card and the screen is 0.31 ms of it. The names Builder added to the
  map (the window, hearing the window, getting the picture, handing over, measuring, the wiring, the flight's
  report, the session's log) stand as hers, his to change whenever he likes.
- **[DECIDED 2026-11-12] Handing over split in two** (David, to Builder, after asking how it works and whether it can
  be streamlined: "go ahead and split it, so later on we can see it and tune it if it ever becomes something that can
  be improved"). Build 118 (29a02df) times it as two passes under old's names, his to change: *handing the card its
  work* (0.08 ms, likely the processor checking and translating the frame's work for DirectX; the one to watch as the
  eye adds passes) and *showing it* (0.14 ms, the picture waiting its turn for the screen; where the frame's lead
  over the screen is set later, with his eye). Two flights repeat within 1%, the frame as 117's
  (`kept logs/flight-1794487928.txt`, `-1794487963.txt`; `journal/docs/experiments.md`). Nothing is tuned now.
- **[DECIDED 2026-11-12] The doorway's columns** (David, to Builder: "go ahead"). `src/doorway.rs` answers "what's
  here?" a tile at a time from old's export, read-only: all 5,047 reference columns as old's, to the block (ead30eb;
  2.1's gate on the doorway). On the map, a third lane, "loading", holds the pass "the doorway", reading "the export"
  and writing "a tile's columns": Builder's names, his to change. What changed, pieces and lights, and named fields
  come with what first feeds them.
- **[DECIDED 2026-11-12] The marks as data** (David, to Builder: "go ahead"). `data/marks.toml` holds the places
  every flight stands at, as old flew them, each eye checked against old's to a tenth of a block: the spawn, the
  mountain top on a clear day, the bare plain near and far, the plain with David's build (its 169 placings, 146 bars
  and 23 blocks, made as changes through the doorway, none refused), the scatter at 1,000, 10,000 and 100,000, high
  over the spawn, low over the sea (builds 119 and 120). The game starts at the first mark. *Builder's reading, his
  to change:* the plan's "plain's horizon" is the bare plain's far eye. Her names on the map: the pass "going to a
  mark", the stores "the marks" and "what changed".
  - **Corrected in the open: the floor.** Build 119's flight found the session's first 25 seconds or so run slower,
    the card's pass too. The empty frame's floor recorded above (build 116: 0.40 ms, 0.019 on the card) was measured
    inside that warm-up. Settled, it's **0.34 ms a frame and 0.012 on the card** (build 120, every mark 0.338 to
    0.344; `kept logs/flight-1794489536.txt`). 116, 117 and 118 flew the same way, so their comparisons stand; only
    the floor's number moves.
- **[DECIDED 2026-11-12] 2.1 closed, done enough to go on** (David, to Counsel's pick: "yes, close 2.1. let us
  proceed."). Its gate met: the empty frame measured, the map drawn from the code, the doorway answering the 5,047
  reference columns to the block. What's left is carried to where it's first fed: the doorway's pieces, objects and
  lights to 2.4; its named fields to 3.1; every mark flown on both cards by one command to 2.2, whose gate needs
  the Intel card (the laptop on "Optimus" for that flight). Next: 2.2, the far eye.
- **[DECIDED 2026-11-12] The far cells summed on the processor** (David, after Counsel set it beside his rule of
  2026-11-04, "ONLY write code that increases the gpu's effort"): "yes, i think we'd be right to use the processor
  for summing the far cells. its a 1 time event, and it's quick." 2.2's first step sums them on the loading lane
  from the doorway's columns; old's bake ran the recipe on the card. If the first sum's time comes in well over
  Builder's estimate (two minutes), it's looked at again.
- **[DECIDED 2026-11-12] Old words weigh less** (David, to Counsel): "you should probably decay the
  authoritativeness of my statements about how things should be, by the time since i've said them and how much has
  changed. some things may never change, but really, everything everywhere is always moving, and so are the ideas
  we have, as we try and discard them. reminding me of them is great, because it brings up how far we've come, just
  don't be too attached to the things i've said unless its about current or recent work. you guys are doing a great
  job. just remember that i also have a limited context window and may have no clue whatsoever why i said something
  should be this way a week ago!" So an old word is quoted with its date and what's changed since, as a reminder,
  and asked about, not held as binding. Words about current or recent work keep their weight. This file is read
  that way too: its older entries are how we got here, and `master/` is what holds now.
- **[DECIDED 2026-11-12] The far colours carried from old** (David, to Counsel's pick: "Yes, carry old's"). Each of
  the eight blocks' far colours (top and sides) as old worked them out, its texture shrunk to one pixel as light,
  kept as numbers in a data file until the Registrar (3.2) sets colour properly. Why: the far cells' check against
  old's bake needs old's colours, and choosing them now would mean choosing twice. David, to Builder, the same
  hour: "the far colors are not important now. i'll go in and regulate everything again once we rebuild the
  registrar. until the graphics engine is ready to race, everything is just a testing device anyways."
- **[DECIDED 2026-11-12] The far eye's first picture, in two looks** (David, to Builder's plan through Counsel: "Yes,
  go ahead"). 2.2 step 2 moves old's far march onto bricks as it is. 2a is the shape: the march, faces lit by the
  sun, fog and sky, the screen pyramid, with no shadows or sky light yet. 2b adds old's light (sun shadows, sky
  light) and is then the baseline the three changes are measured against. He looks from two marks, high over the
  spawn and low over the sea, reached for now by a launch switch (`INCUS_MARK`), so no key or control is placed. Why
  two looks: the step is big, and the shape and the rings are best judged before the light is added over them.
- **[DECIDED 2026-11-12] The far eye's first look, and its tuning on the Render page** (David, to Builder, launching
  it: "ok, lets see"). Build 121 crashed 18 s in (a brick at the edge of the levels over 1,024 wrote past its row;
  Builder's build 122 fixed it, with a test on every edge, `journal/docs/experiments.md`). High over the spawn: "there's
  a little bit of blotchiness - idk if thats just the terrain being messy, or if it will be covered by near views ...
  one thing i had not been able to work in as the last graphics engine was coming apart around us, was the
  sky-mountain blending. sometimes squares would be kitty corner to the mountain in the sky a bit, darker ... weight
  the sky a bit more ... it looks right, mostly, and if performance is keeping up with what we'll need, we can
  proceed". Then: "if we could just make sky a bit stronger in the math over land, i think that might give a basic
  natural feather ... a very simple crossover slider set for the transitions ... watch the card time as i adjust it
  ... a default button ... we don't need presets yet ... go ahead and do your tests, and yes, we should keep these
  things on the render page of the debug panel, with a relevant speed gauge visible nearby."
  - **Where it goes, his placement:** the Render page of the debug panel (F3), the far eye's card time at its top.
    Sliders for where the levels hand over (where the 4 × 4 and the 2 × 2 fade in) and for the sky's weight, and a
    Default button; no presets. The sky's weight starts at old's (none) until he picks a value, which then becomes the
    default (build 123). The slider labels and the store "the far eye's look" are Builder's names, his to change.
  - **The blotches,** from the flight: the finest screen layer (2 × 2) is almost never earned from far cells alone,
    so most of the screen is 8 × 8 and 4 × 4 squares shown crisp, each a slightly different average.
  - **His condition, "if performance is keeping up":** the far eye as old had it costs 1.2 to 4.0 ms on the card at
    the far marks, 2 to 4 times 2.2's gate, before light. That's the baseline; by the plan, the three changes come
    next and the gate is read after them, with the Intel card.
- **[DECIDED 2026-11-12] The sky's feather at skylines** (David, to Builder). On build 123's plain weight: "love what
  the weight does to the edges, hate what it does to the mountain *not* on the edges. i suppose sky is everywhere
  behind everything... is there anything that can be done about that that isn't expensive?" Build 124 makes the push
  fade as the land covers more (the power is 1 + weight × the sky's share squared): thin skyline squares lean to the
  sky, and a mountain's own pixels are left nearly as they were, for one power a pixel. David: "that actually works
  great". The value he settles on becomes the default, and Default returns to it. Why: the skyline's darker squares
  were old's unsolved edge; a feather that costs nothing and leaves the land's body alone answers it.
  - **His value: 4** ("i maxed it"): the default from build 125, the slider now running to 8 so his value sits in
    the middle of the knob. The rest of 2a's look (`journal/docs/experiments.md`): "honestly none of the crossover
    sliders do much" (from far cells alone the finer layers are rarely earned; the sliders are kept for when the
    near eye brings small cells in); low over the sea, "yeah it goes up if i look around. anyways just noting it"
    (1.3 ms looking up, 4.2 along the horizon). He named no ring as a seam.
- **[DECIDED 2026-11-12] Old's light on the far view, and the baseline** (David, to Builder, on build 126: "light
  looks normal. hard to tell as it is, but ive never had an issue with your sun lights looks, so proceed"). Old's far
  march on bricks with old's light (the sun's shadow ray, the sky's five cones) is the baseline the three changes are
  measured against: on the card, the spawn 0.77, the bare plain 1.42, high over the spawn 2.38, the mountain top
  2.74, low over the sea 3.95 ms (`kept logs/flight-1794497402.txt`). Light is only 3 to 13% of it at the far marks
  (old's eye: 17 to 42%), because it's worked out once a pixel and nearly every far pixel is an 8 × 8 or 4 × 4 square;
  the cost is the march itself.
  - **[OPEN] The changes' order:** Builder proposes changes 1 and 2 (soft bricks blended by the texture hardware;
    rays begun at the drawn boxes), aimed at the march, first; then light's share measured again before deciding
    whether change 3 (light read from the cell) is worth building for the far eye, which can win at most about
    0.2 ms there. Light kept a cell at a time was decided at 4b (2026-11-11) for the rebuild; this asks only when, and
    whether the far eye needs it before the near eye (2.3) does. David to say.
- **[DECIDED 2026-11-12] Try everything, measure it, and let him look** (David, to Builder, answering the order):
  "honestly, i think we should try all these things and gather data about the performance impacts they have, and let
  me take a look at how they look while flying and while still/moving my head." Builder's order from it, each
  measured, then his eye still and flying:
  1. the panel fix and the live settings;
  2. change 1 both ways (A, her exact blend; B, the Oracle's spread), switchable live;
  3. bricks of 4 against 8 (chosen at launch);
  4. a window round the eye for the 32- and 64-block levels, with a fast flight;
  5. change 2 (rays begun at the drawn boxes);
  6. change 3 (light read from the cell), after light is measured again;
  7. the Intel card's flights last, with Optimus put to him a step ahead.
  Why: the costs and the looks are both his to weigh, and data beats guessing which change wins. David, to Counsel,
  on the memory each costs: "yes, we'll need to assess the costs scientifically and weigh their effects
  architecturally." So each step reports its time and the game's whole card memory, and a choice is weighed against
  the whole eye's budget (near eye, light and the rest included), not by the far view alone.
- **[DECIDED 2026-11-12] Ask idle hardware what it can do** (David, to Counsel): "supposing that we keep the games
  running graphics memory consumption under a safe 4gb, or even a sleek 2.5gb, what other uses do we have for
  graphics memory? we should have an exploitative mindset: if something in the generic hardware resources we are
  using is being underutilized, we should ask, 'what can *you* do for me?'" A principle, as with the idle triangle
  and texture hardware before (changes 1 and 2). *Counsel's reading, for the fork:* what the game needs to run stays
  under the memory bar on every card; what only makes it faster or finer is kept as caches that grow into a card's
  free memory and drop away on a small one, bounded by the card and never growing with play, the picture the same
  either way. Put to the Oracle (`journal/vision/oracle-return.md`, question 9).
- **[DECIDED 2026-11-12] Change 1's precision: the look, not the last digit** (David, to Builder, after the card's
  filter missed Builder's 1/1,000 bar by weighing to about 1/256): "the exact colors at any point or few points isnt a
  big deal. itll all be tuned again anyways. as long as it fundamentally works right and meets our goal for the far
  views function and appearance." So the texture hardware's precision is the bar, and a slight difference between
  the NVIDIA and Intel cards' filters matters only if it shows; Builder compares one mark's picture on both cards,
  by the code's blend (the control), exact and spread, at the Intel flights, so it's known rather than assumed.
- **[DECIDED 2026-11-12] David's look at change 1** (build 128, the three blends switched live, to Builder):
  - **The blends:** "as far as the far blend, they all look the same." So the numbers decide: Builder recommends the
    spread (faster, 1.1 GB lighter) and taking the exact way out, to put to him.
  - **Detail:** "-2 detail looked better, didn't seem to impact the frames." -2 is the slider's coarse end: Builder
    offers it as the default, the slider widened so it sits mid-knob, and re-flown.
  - **Skylines:** "crisp looks better than smooth or sharp skylines." Crisp stays the default.
  - **[OPEN] The fog line:** "the fog kind of makes an ugly line, pics are fog on, fog off. not sure if thats a
    problem or will resolve itself as we finish out the engine." Builder's reading: a near ridge's outline against a
    farther, fogged mountain shows the 8 × 8 layer's squares, because the finer layers are earned by distance only,
    never at edges (with fog off, near and far are one colour; against the sky the feather hides them). It won't
    resolve itself: the near eye won't reach ridges kilometres off, and nothing in the order changes it. Her fix:
    earn the finer layers along edges where neighbouring squares differ, priced by flight. A look step; David says
    when.
  - **[OPEN] No flying for players:** "oh and btw you know there's no flying at all in the game right?" (The spec
    had it as an early word, 2026-10: "i don't forsee players being able to fly".) The developer's camera flies; the
    player doesn't. Builder is asking which he meant. It bears on the far eye's gate, measured at marks a player
    can't reach (high over the spawn, low over the sea).
    - *Corrected in the open (Counsel):* he meant the camera, not the design. David: "i just don't want you to think
      i can move in the game, or that you ARE moving in the game during your tests... the camera rotates but does not
      move, so make sure thats what you thought". Builder's flights stand still at each mark and jump between them,
      so nothing has been measured moving yet; the developer's camera is meant to fly (WASD, Space, Ctrl, Shift five
      times, 20 blocks a second), so for him it's a bug or too slow to see against the far cells' blur, and Builder
      is finding which. The question built on the misreading (the far gate at player views) is withdrawn; the early
      word on players flying stands as it was.
  - **Corrected in the open: the detail slider ran backwards.** Labelled "detail (levels finer)", but the shader added
    its value to the level, so his -2 was two levels *finer*, not coarser: what he liked was the finer picture.
    Builder flips the slider so right means finer; his pick becomes +2, offered as the default and re-measured.
  - **His go, and fog** (David, to Counsel): "i was just pointing out to the builder that the camera was not
    flyable in my tests and wasn't flying in her tests to make sure that was as it is meant to be. i could be wrong,
    i'll try again next time and see if im actually moving. tell builder to proceed. note that we may delete fog and
    try another way to implement it later, since it's only for looking nice. i think there are different ways and
    times to add fog, aren't there? it looked good in old incus artus, so if it's the same, maybe it'll look
    different once we're back to where we were." So Builder proceeds (the spread and two levels finer as her picks,
    then bricks of 4 against 8). The fog line waits: fog is for looks only, and may be taken out and done another way
    later; the finer detail and the near eye may change how it reads first. He checks the camera's "up" (F3, Camera)
    at his next launch.
  - **His look at build 129** (31 minutes, to Builder): "as you have the detail set now, everything from -1 to 4
    looks the same." Builder's reading: with the camera not moving for him he saw only the spawn, where every setting
    reads the finest 32-block cells nearly everywhere (+2 costs 2% there); detail shows looking down from height,
    where +2 costs 77%, which he hasn't seen. **[OPEN]** Builder recommends the default go back to 0 (old's, what
    everything has been measured at) unless a look from the mountain top says otherwise, and offers a launch there.
    Why it matters: a 77% cost should be bought by something he can see.
  - **The camera moves, slowly** (David, to Builder): "yes, i did notice i could actually move. its just too slow to
    notice at first." No bug: old's 20 blocks a second (100 with Shift) is too slow to see against far cells. Builder
    offers a faster one, asking him where its control goes before anything is built.
    - *Corrected in the open (Counsel): it was a bug.* Step 4's first moving flight went 1.3% of its asked speed (63
      blocks of 5,000), and its report showed it. Since the rebuild began the camera's step had been the sliver
      between frames, not the whole frame, so it flew at about 1% of its speed: his "too slow to notice", and why the
      speed slider seemed weak. Fixed in build 133 (85d876f) with a test that fails on the old step. At the same
      settings the camera now feels about 75 times faster.
  - **[DECIDED 2026-11-12] The camera's speed, his placement** (David, to Builder): "yes, i should have a speed and
    multiplier slider in the camera tab please". Build 130: on the panel's Camera page (F3), "speed (blocks a
    second)" from 1 to 2,000 on a logarithmic slider, and "multiplier (Shift held)" from 1 to 20, defaulting to old's
    20 and 5, with a Default button. The labels are Builder's, his to change.
  - **Detail back to 0** (David, to Builder, on build 130 from height: "yeah 0 is fine"). The default is old's again
    (2f756a4); +2's 77% from height wasn't bought by anything he could see. Change 1 is done enough to go on: the
    spread kept, the exact way out, detail 0.
- **[DECIDED 2026-11-12] Bricks of 4, for now** (David, to Builder: "4 sounds good"). Step 3 (build 131): bricks of 4
  take the game's card memory from 2,656 to 1,555 MB for 5 to 13% more card time, the picture the same. The default
  since 18d2537; `INCUS_FAR_BRICK=8` keeps 8 a launch away. Why "for now": the window round the eye (step 4) cuts the
  same memory another way, so the final size is picked after step 4 is flown at both 4 and 8 (Counsel's order, put to
  him by Builder).
- **[DECIDED 2026-11-12] The window round the eye, and the card's default limits** (step 4, builds 132 to 134). Only
  the 32s within about 8 km of the eye and the 64s within about 18 km are kept on the card; the coarser levels stay
  whole. The game's card memory in use at bricks of 4: **471 MB** with the window, against 1,791 for the whole world;
  still card time within 3% of the whole world's. Builder had raised the card's storage buffers a pass from 8 to 16
  for it; Counsel asked whether a 980 Ti and the Intel card allow 16, and Builder folded the extra flags into an
  existing buffer instead (build 134), so the far eye keeps the default every card allows, at no cost.
  - **[LATER] What 16 buffers could do** (David, to Counsel): "we will revisit that and ask, what can 16 buffers do
    for us, if there's anything to gain". On the window's card in the map, the 8-buffer way kept as the fallback.
    Why: his "what can *you* do for me?", applied to limits, without making the floor depend on them.
  - **Moving, the first time measured:** at a player's 200 blocks a second the window keeps up (about 100 blocks
    behind) and the typical frame matches the whole world's, but the tail is worse (1 frame in 1,000 at 6.2 to 6.7
    ms, the worst 7.9 to 9.4, against 4.2 and 4.4): spikes as a whole region of the window is swapped in. At 2,000
    it keeps up and creaks (worst 22 ms). **[OPEN]** Builder's fix, a brick at a time instead of a region, waits on
    David feeling the spikes at a player's pace; Counsel would build it anyway against his "solid as a rock".
  - **[OPEN] The distant ridges:** from height, past the window the finer layers now read 64s where they read 32s,
    up to 35 to 49 of 255 apart on far ridges. For his eye, window against whole world.
  - **His look at build 134** (to Builder, moving at a player's 200 blocks a second and at the ridges): "it looks
    good". He named no hitch and no ridge, so the window stands as it is, and the brick-at-a-time conveyor waits
    unless he feels the spikes.
- **[DECIDED 2026-11-12] The temperatures tracked again, always on** (David, to Builder: "i'd like for you to start
  tracking temperatures again, just as before"; asked where its switch goes: "No switch, always on"). Build 135: old's
  readings every 3 s on their own lane (the processor's area, the card and its clock), drawn top right while playing
  but not on flights; each flight standing reports its hottest readings and the card's slowest clock;
  `INCUS_TEMPERATURES=0` turns them off for measuring. Why: heat was how old's slowdowns first showed, and a flight's
  numbers mean more beside the clocks they ran at.
  - *Corrected in the open (Builder):* his "i dont need to see the temp readout" answered whether he needed a launch
    to see it, not whether to show it; Builder took it off the screen (33e48a1), and David asked "why aren't temps
    shown on the screen". Back on screen, top right, in build 136 (ecb67ef).
  - **Always, flights included** (David, to Builder: "in the future, can you make sure that the temp displays are
    always visible while in the world"; then, when Counsel suggested keeping flights clean: "well, while you are
    running tests, i am sitting here watching, and id like to know what the temps are at while watching. believe it
    or not i sit here all day just watching you work through hours of tests."). Build 139 (38e0dd1) draws the
    readout on flights too, a few letters' cost, noted in the experiments. Why: he watches the flights.
- **[DECIDED 2026-11-12] Bricks of 8, with the window** (David, to Builder, after Counsel recommended 8: "you can go
  with 8"). With the window, memory stopped binding (916 MB at 8 against 471 at 4, far under the 4 GB bar) and speed
  still does (the far marks 2 to 3 times the gate), so 8's 6 to 11% is worth more than 445 MB. The default since
  33e48a1; `INCUS_FAR_BRICK` keeps 4 and 2 a launch away.
- **[DECIDED 2026-11-12] Change 2 tried: rays begun at drawn boxes** (David, to Builder's plan: "yes, proceed").
  Build 137: each 256-block column of land drawn as a box at the 8 × 8 layer, swollen by what a blend reaches; rays
  end at the farthest box, and no box means sky at once. The picture the same (0 pixels at five marks). The boxes
  cut a quarter of the steps, but they were the sky's cheap ones: their pass costs 0.16 to 0.18 ms, the 8 × 8 saves
  16 to 17% at the plain and the mountain top but 1 to 3% over the sea and high over the spawn, so the far eye whole
  breaks even at the plain and is 1 to 7.5% *slower* elsewhere (24% at the spawn). The time is where rays meet land
  near the horizon, which the boxes don't shorten.
  - **[OPEN] Builder recommends:** off by default, the code kept behind `INCUS_FAR_BOXES` until the Intel flights,
    then kept or taken out on their numbers. David to say.
  - **The price of keeping it, measured** (Counsel asked, from change 1's lesson): build 136 against build 137 with
    the boxes off, alternating twice in one session: the unused code costs the march 0.2 to 1%, at most 0.01 ms, the
    same at every mark. A first 7.5% hint had been noise between sessions. So "kept till the Intel flights" costs
    under 1%; out now is the other way, since git keeps it.
- **[DECIDED 2026-11-12] The Oracle called midway through 2.2, and its first reply** (David sent Counsel's prompt,
  `journal/vision/oracle-prompt-2026-11-12.md`, with his own: "I beg that if you have access to the knowledge of
  sophisticated algorithms and architectures that could radically change our performance picture, you mention
  them"). The Oracle's reading, in its session, before its third part: **the far eye is waiting, not working.** A
  layer costs about the same however many rays it runs (the 8 × 8 and the 4 × 4, four times the pixels, 0.92 and 0.91
  ms at the mountain top); about a thousand rays skimming the horizon, each up to 768 steps that wait on the one
  before, at forty to sixty small reads a step, set each layer's time while the card sits nearly idle. Medium
  confidence; a second reading (the card busy but holding few rays in flight, the program being large) fits most of
  the same facts. What follows if it holds: make each step lean (what doesn't change kept in hand); make the longest
  rays shorter (the open sea as a sum, rays ended at the map's edge, a gentler rule for long skims); fewer layers
  waiting on each other (the 4 × 4 base). On its own design: the boxes "were aimed at the wrong thing"; the rest held
  (bricks, the hardware's blend, light in the cell); the far eye becomes the lean march on bricks; for the near eye,
  rays from the eye with better skipping, triangles the fallback. (Builder's correction to David: the 4 × 4 runs only
  where the 8 × 8 says it's worth it, so "four times the rays" overstates; the half-rays test settles it.)
  - **Its questions, and David's answer** (to Builder): "we should at least try 2, 3 and 4. Do you think we should do
    your three steps before answering her, to gather some information? if so, please proceed with them". So:
    - **First, before he answers it, Builder's three steps:** the boxes taken out (the Oracle no longer wants them for
      the near eye; git keeps them) and rays ended at the map's edge, exact, behind a switch; the Oracle's two tests
      (half the rays marched; the step limit halved, 768 to 384) and Nsight on the new march; then the lean step, the
      same picture proved pixel for pixel. Each with its pass mark first.
    - **Then, to be tried, his "2, 3 and 4":** the skyline made by how much of each square the land covers, instead
      of rays through a haze of part-solid cells; plain land drawn as a surface from the same cells, rays kept for
      what isn't plain columns (caves, towers, bridges, canopy); the horizon band alone refreshed over a few frames
      (the slowest thing in view, turning still exact). Each a look step; the order set once the tests say which
      reading holds.
    - **[OPEN] Its question 5:** the cells of 2 to 16 blocks aren't in the new eye yet, between the block field and
      the 32s, where forests and towns will mostly be seen. The Oracle advises bringing them into the far eye's march
      before the fork, so the fork judges the whole distance. 2.3's, or 2.2's before the fork: David to say.
  - **Temporal reuse:** Builder's view, keep whole-screen TAA where his 2026-11-08 rules put it, after the eye is
    whole; the Oracle's question 4 is its narrow, safe form (the horizon band only), which tells cheaply whether
    history suits his eye.
  - **The tests' answer** (build 140, one session, `journal/docs/experiments.md`): rays ended at the map's edge take 6
    to 19% off every far mark (kept on; low over the sea 2.77 → 2.58 ms, a player's run 3.10 → 2.52); half the rays
    take only 13 to 36% off the 8 × 8, so the truth sits between the Oracle's readings, nearer "waiting"; the step
    limit halved takes 20% off the sea and 11% off the plain, so the few rays past 384 steps cost a fifth of the
    dearest mark. The long rays' chain sets most of each layer's time.
- **[IDEA 2026-11-12] Distance fields and shuffled rays** (David, to Counsel): "could we use signed distance fields to
  significantly decrease the spread in ray length? is there a way to shuffle rays automatically so that warps fill and
  empty one at a time instead of collectively?" *Counsel and Builder:* a plain distance to the nearest surface shrinks
  just where grazing rays live, so the height forms fit better: a per-column height (a max-height pyramid finer than
  the 32s' band tops the march already skips by) would let a ray skimming a plain skip, cheap in memory and exact for
  plain columns; cone-step maps are its cousin. Shuffling (persistent threads, rounds with packing, rays sorted by last
  frame's length, his own idea of 2026-11-08) helps the share that scales with ray count, 13 to 36% by the test; it
  can't shorten the single longest ray. Both to be priced.
- **[DECIDED 2026-11-12] Lazy rays, a lead** (David, to Counsel): "is there a way to make the long ray step again...
  in pieces across frames? instead of doing it once every ten frames... just take a leisurely time getting there, a
  little at a time, each frame? ... i feel like this could work at several levels by distance/resolution. spread out
  any rays that can be lazy." Before it: "im ok with distant terrain updating less often, you know; at a proportionate
  distance, as long as its real ... increasing duty cycle to meet demand when necessary (probably flying)". Then: "i
  think this lazy-ray idea might be a winner. if its basically invisible and can ramp up duty cycle when necessary, it
  sounds like a good lead."
  - **The idea:** any ray whose part of the picture changes little per frame marches a share of its steps each frame,
    resuming where it left off; its result stands until it finishes. Laziness follows level and distance (the near
    field full every frame, coarser and farther layers lazier) and shortens with speed. It answers the tests'
    "waiting": the long chain is cut into short pieces a frame, and the total work falls too.
  - *What it needs (Counsel, Builder to price):* each lazy ray's state kept (a few MB for the far layers, spare
    memory's free win); a store indexed by direction round the eye, so turning costs nothing; rays restarted when the
    eye has moved past a level's tolerance, and where edits or light changes touch (his 2026-11-08 rules); directions
    new to view get coarse rays first. At 10 km, eight blocks of travel is about a pixel and a third at 2,560 wide,
    under one 8 × 8 square.
  - *Its pass mark, suggested:* the settled picture equal to the full march's, pixel for pixel; the lag measured
    (frames to settle at each level, walking and flying); his eye judging "basically invisible".
  - Builder drafts a writeup of the tests and lazy rays for the Oracle with David.
  - **[IDEA 2026-11-14] How rays know to be lazy** (David, to Builder): "ive been thinking about how rays know to be
    lazy because it would seem we have to reduce the longest rays, one by one in order of length, in order to actually
    free up warps more rapidly, consistently." Builder: right, since the card runs rays in groups of 32 and a group
    ends with its longest, only the longest matters. A **step budget** does it without sorting: every ray stops at the
    budget, which trims the longest first, down to one even length; a ray that ran out carries on next frame from where
    it stopped. Before building it, an instrument measuring how much waiting there is inside groups today, since that's
    the most lazy rays can win. It keeps its place in the order, after the cells of 2 to 16.
- **[DECIDED 2026-11-12] No megakernels: small passes, as an architectural goal** (David, to Builder): "i hope we
  haven't built another megakernal ... if we've made a megakernal, it seems obvious we should start splitting it up
  now and establish an architectural goal that supports the microkernalization of it." And: "lets begin the research
  we need before going back to answer the oracles questions". Why: old's one giant program was register-bound, so
  code added cost every pixel even switched off; the near eye is still to be built.
  - **The goal:** the eye as separate small passes over a per-pixel store (march, light, finish), each measured
    alone. Lazy rays fit it, since a ray's state is kept between frames anyway.
  - **Builder's risks:** a split may not be faster (each measured alone); the soft skyline's gathered sum must be
    carried whole across the handover; more passes add fixed cost at cheap views; if the march itself is the
    megakernel, only the lean step fixes it.
  - **The order (Builder's):** the Nsight look; the lean step; light split out; lazy rays on top. The Oracle is
    answered after this research.
  - **The watts say "waiting"** (build 141, LibreHardwareMonitor, which David installed with PawnIO): the card reports
    98 to 100% load at every mark, but draws less the dearer the mark: 99 W at the spawn, 78 to 89 at the plain, the
    mountain top and high over the spawn, 58 over the sea and 63 on a player's run (about 100 W is the card fully at
    work). Queued, not working. The processor sits at 5 to 11% load, 34 to 42 W: room there for work that takes
    load off the card.
  - **Nsight confirms it** (Builder read the exported tables, so David needn't read anything out): low over the sea,
    the 8 × 8 and 4 × 4 passes keep **6.9% and 1.3%** of the card's room in flight, against 96% for a plain pass; the
    8 × 8 launches only 1,769 groups, most finishing at once, the layer lasting as long as its few longest. Three signs
    agree (half the rays, the watts, the room in flight). Unlike old, whose program was too big to fit more in
    flight, here there are too few long rays to fill the card. So: more rays are nearly free while the long ones run
    (a finer base layer, fewer layers waiting on each other); the time is the longest chain (fewer steps, cheaper
    steps, or lazy rays); shuffling has little to win. *For the goal above:* the program's size isn't what holds this
    march back, so splitting alone won't fix the far eye; it stays the right shape for lazy rays and the near eye.
- **[DECIDED 2026-11-12] The Oracle's third part, "waiting, not working"** (`journal/vision/oracle-review.md`, written
  with the tests', the watts' and Nsight's results). In short:
  - **On a waiting card, work is nearly free and waiting is dear:** shorten the wait, even where it adds work.
  - **Its order for 2.2:** step 0, instruments (the longest ray in every report, a nodding mark, and one flight on the
    Intel card now, to know which kind of card it is); step 1, the lean step in halves flown apart (kept in hand;
    read together; several steps at once; thin haze by whole cells, a look); step 2, the layers side by side, each
    taking its lead from the coarser layer of the frame before, the finer base layer its probe; alongside, light read
    from the cell, made on the processor by sweeps (change 3 and "light split out" as one); step 3, read the gate;
    step 4, only if still missed, the long ray cut into lengths marched side by side and joined front to back.
  - **Lazy rays:** sound (Regan and Pose's "priority rendering", 1994, is its forebear), but after those, by lengths
    of distance, under nine rules (a ray in flight always finishes; the store, once whole, equals a full march of its
    own directions; light and fog put on every frame, not kept lazy; the gate read moving).
  - **The column-height skip** pays only with the skyline made by cover (the long rays are summing haze inside the
    last cell, which a height table can't skip); in the near eye it pays alone, from the first build.
  - **Design D:** its store stands (doorway, cells by a lane, bricks, the card's blend, the window); its lens goes
    (the boxes). The far eye is the lean march on D's store. Small passes are right ("Megakernels considered harmful",
    Laine, Karras and Aila, 2013), but they don't shorten a chain.
  - **The cells of 2 to 16:** before the fork, after steps 1 and 2, with a scatter stage behind the doorway to look at
    (David had left this call to the Oracle).
  - **The larger alternatives,** named with costs; the one it would fly, if David takes the cover look, is "the sheet"
    (plain far land as rings of mesh, geometry clipmaps), as the far levels' rival.
  - **[DECIDED] Both near drawers before the fork** (David, to the Oracle: "Yes, fly both"). It changes the rule he
    agreed in the second part (triangles only if rays missed a mark): 2.3 builds rays, then faces as triangles, and
    flies both on the same marks. The judging stays: 1.5 ms at every ground mark; over 1 ms apart at the worst mark
    is his call; his eye for pops and seams.
  - **[DECIDED] Its order for the rest of 2.2** (David, after Builder weighed it and agreed: "Yes, its order"): step
    0's instruments and an early Intel flight; the lean step in halves; the layers side by side; light read from the
    cell, made on the processor; the gate; then the cells of 2 to 16 with scattered detail; lazy rays after, by
    lengths. Why lazy rays wait (Builder, put to him): the lean step and the layers keep the same picture and are
    quicker to prove; the cut-into-lengths fallback shares lazy rays' machinery and prepares it; if the gate is met
    without them, they become headroom, not a need. Builder's cautions: 1c (several steps at once) flown alone and
    checked pixel for pixel; the processor's light gets a pass mark on lag.
  - **Why it runs hot** (David: "id really love to know why its running so hot. it ran like this with the graphics on
    the cpu"). Builder's readings: the game capped at the screen's rate draws 26 W with its hottest core in the 70s,
    no hotter than the desk with no game; the uncapped flights are the heat (one thread pins a core near top turbo
    speed, beside the card's 60 to 100 W in the same cooling); spikes past 100 °C come with no game at all (102 °C at
    the desk), the laptop's Turbo bursts.
  - **The lean step before the Intel flight** (David's choice, to Builder; the Intel flight when it suits him). Step
    1a, kept in hand (build 143), against build 142, alternating twice: 6 to 13% off the far marks (the sea 2.58 →
    2.24 ms, its nodding 99th 3.13 → 2.72; the plain 1.06 → 0.95; a player's run 2.58 → 2.34), the spawn 5% slower
    (short rays pay for setting up what they keep), the longest rays step for step the same and the picture pixel for
    pixel. Kept; its keep-or-drop after the Intel flight, since a working card pays for what's held.
  - **The Intel card is working, not waiting** (2026-11-13, on Optimus after David's restart; build 143): its cost
    follows how much is marched, not the longest ray (the spawn's 8 × 8 costs 4.4 ms, as much as the sea's); its
    dearest marks are the mountain top (10.5 ms) and high over the spawn (10.1), not the sea (8.6); 4 to 8 times the
    NVIDIA's time; memory 933 MB as on the NVIDIA. So the two cards want different things, as the Oracle warned: what
    adds rays for nothing on the NVIDIA (a finer base, the layers side by side) costs the Intel in full; what cuts
    work helps it more (cheaper steps, fewer rays earned, shuffling, lazy rays).
  - **1a on the Intel card is slower at every mark,** 2.5 to 10% (the NVIDIA 6 to 13% faster at the far marks): the
    march as built keeps about forty numbers in hand, and a working card loses more room than it gains in reads.
    **[OPEN]** Builder's three choices for David: keep it as built (the NVIDIA wins, the Intel loses); drop it; or a
    trimmer 1a first, keeping only the handful a skimming step needs (the Oracle's own words), flown on both cards in
    one session on Optimus. Builder and Counsel recommend the trim.
  - **[DECIDED 2026-11-13] 1a kept as built; the Intel card informs, it doesn't decide** (David, to Builder): "I'll let
    you try 3, but i lean towards 1. Keep it. i don't expect someone to be able to play this game on an igpu without
    turning some settings way down. the 980ti is a different story and should perform much differently than an Igpu,
    and i'm not getting a deep sense of concern over performance on less than current GPU's based on the intel gpu.
    its valuable information to gather nonetheless. so try the trimmed version, but unless its just better than 1, we
    go 1." The trim (build 144, about two dozen numbers held where 1a holds forty) won back 2 to 4% on the Intel but was
    8 to 19% slower on the NVIDIA (flown on Optimus, labelled so) at the plain, the mountain top, the sea and a
    player's run. Not just better, so 1a stays as built (build 145, c89a040's march; git keeps the trim). What it
    taught: the brick's slot and the next level's numbers held in hand are the NVIDIA's gain and the Intel's lost room;
    no one version suits both. Still owed: the NVIDIA side re-flown on "NVIDIA GPU only", a record for the gate.
    Why: the Intel numbers are worth gathering, but integrated graphics are expected to need settings turned well
    down; the 980 Ti, not the Intel card, is the floor he cares about.
  - **1b, read together, its direction** (build 146, behind `INCUS_FAR_TOGETHER`, the picture the same): a small gain,
    far under both guesses. On the NVIDIA on Optimus 3 to 10% off most far marks (the sea 2.49 → 2.38 ms, high over the
    spawn 2.01 → 1.82) but the mountain top 8% slower, not yet explained; on the Intel 0 to 3% faster. The Oracle had
    given 15 to 40% off the open-horizon marks: likely a long ray's steps are mostly skips, which read no pictures. Its
    keep-or-drop waits on the "NVIDIA GPU only" flight after David switches back, with 1a's numbers for the gate.
  - **1b kept, on from build 147** (on "NVIDIA GPU only" after David switched back, off and on alternating twice): 2.5
    to 7% off every far mark, nothing slower (the sea 2.33 → 2.21 ms, the mountain top 1.76 → 1.63, a player's run 2.42
    → 2.31); the Intel lost nothing. The mountain top's 8% loss on Optimus didn't hold: Optimus numbers give a
    direction only. The far eye against the gate now: the sea 2.21 ms (its nodding 99th 2.72), the plain 0.95, the
    mountain top 1.63, high over the spawn 1.65.
  - **A watch:** today reads about 4% slower than last night for the same code, and the fast run reads double in every
    pass with the card's clock down near 1,100 MHz at 53 °C (last night 2,500 MHz at 71). The card slowed itself, not
    from heat; the laptop's power mode after the restarts is suspected. Builder asks David to check it; within-session
    comparisons stand, but numbers against the gate want a re-read once the power mode is known.
    - *Settled:* David confirmed Turbo and full fans on "NVIDIA GPU only" after a fresh restart, with a Windows update
      overnight; the NVIDIA driver was unchanged (32.0.16.2101, read from the reports). Later flights ran at full clock
      with nothing changed: Windows finishing its update in the background, starving the card of frames (the
      Oracle's reading: the card slows itself when it has little to do). A standing whose processor load reads well
      over its usual 5 to 11% is now treated as suspect, and each standing's heat line says how long the card's clock
      read under 2,000 MHz (build 150).
  - **1c, several steps at once, dropped** (build 148, off and on alternating twice): 11 to 25% slower at every far
    mark (the sea 2.21 → 2.69 ms), and its code cost the march even switched off, so it's taken out whole (git keeps
    it at build 148). The Oracle's own test had predicted it: 1b gave little, so the card doesn't overlap reads much.
  - **1d, thin haze by whole cells, tried** (David, told by Builder that haze is the land's own faint fringe, a blend
    of 2% solid or less, not fog: "sounds like it might actually look better, lets try it"). Build 151, a setting
    (`INCUS_FAR_HAZE_STEP`, a half cell as it was, or a whole one), alternating twice: the sea 2.22 → **1.78 ms**
    (−20%), under the gate's 2 worst for the first time, its nodding 99th 2.72 → 2.14; a player's run 2.29 → 2.00
    (−13%); the plain 0.95 → 0.88; the mountain top, high over the spawn and the spawn 1 to 2%. The longest rays
    shorter, not halved (the sea's 558 and 764 steps to 532 and 668). The pictures nearly the same (the sea within
    13/255; high over the spawn a fine speckle, 5% of pixels by more than 4/255). Still a half cell by default until
    David looks in the game.
  - **1d kept** (David, on build 151 in the game: "i don't see anything bad about it. Seems like a good thing."), on
    from build 152. **Step 1 is done enough to go on:** 1a, 1b and 1d kept, 1c dropped. The far eye now: the sea 1.78
    ms (nodding 99th 2.14), a player's run 2.00, the plain 0.88, the mountain top 1.57, high over the spawn 1.61.
  - **[LATER] The far view's splotchy noise, for the polish** (David, in the same look): "you can see how splotchy the
    rendering is; im not sure what would cause this, and i'm also not sure it'll look bad at a distance with near
    field rendered, but im wondering what can be done about how rays are read that could make the look more
    'orderly' in its noise. something for you to ponder, not to change our immediate work for right now; but its
    something that will become more relevant at the end when we're polishing everything". Builder's reading
    (`journal/docs/experiments.md`): the straight blend between cells (creases, faceted lumps), light from each cell's
    stored face shares (a patch a cell), where the steps fall (rings and streaks), and the 8 × 8 squares brought up
    crisp. Candidates for the polish: a rounder blend, light from the slope of the solid field, an even (blue-noise)
    step offset, a finer base layer. Nothing changes now.
  - **The polish passes, noted for the plan's end** (David, to Builder): "please make a note of those polishing step
    passes where we can try to work the rays into a prettier shape without doing much ... we may have a lot of
    headroom inside the card for making things prettier". Each a look step with its own pass mark, once the gate is
    met: (1) a rounder blend, curved weights so the cells' creases go; (2) light from the shape itself, the slope of
    the solid field; (3) an even offset for the steps (blue noise); (4) a finer base layer (also step 2's probe); (5)
    the skyline's squares, if they still show. Why it may be cheap: on this card the far passes leave 93 to 99% of the
    card idle while they wait, so more arithmetic per ray costs little there.
  - **Step 2's probe: a 4 × 4 base is slower at every mark** (David: "alright, tell builder to proceed as indicated";
    build 153, `INCUS_FAR_BASE`, 8 and 4 alternating, NVIDIA only, fog off). Base 8 then 4, median ms: the spawn 0.69
    / 1.90, the plain 0.88 / 1.68, the mountain top 1.57 / 3.05, high over the spawn 1.61 / 3.28, the sea 1.78 / 2.11
    (nodding 99th 2.15 / 2.58), a player's run 2.00 / 2.70. Four times the base's rays cost 1.9 to 3 times its time:
    a layer's time isn't only its longest ray's wait, and "waiting, not working" holds for today's layers but stops
    somewhere between 64,000 and 256,000 rays. The Oracle's probe question isn't answered cleanly (the 4 base marches
    every pixel, today's 4 × 4 only where wanted), but the 8 × 8's culling earns its place. The base stays at 8; the
    switch stays for the polish list's finer-base look, now priced at 0.3 to 1.7 ms.
    - **For step 2 proper (the layers side by side):** still untested directly, and Builder's estimate comes down,
      since layers marched at once share a card this probe shows is partly working. Builder recommends building it
      anyway, as the order David took says, unless the Oracle reads this as a reason to go to step 3 or 4; she
      suggests showing the Oracle the table. She designs it and writes its pass mark meanwhile; no flights yet.
    - **Step 2 proper flown: nothing gained** (build 154, the layers one after another against side by side, each
      taking its lead from the frame before; standing still the same picture from the third frame). Median ms: the
      spawn 0.70 / 0.70, the plain 0.88 / 0.88, the mountain top 1.57 / 1.58, high over the spawn 1.61 / 1.53, the sea
      1.78 / 1.79, a player's run 1.99 / 1.99, the sea's nodding 99th 2.13 / 2.13. The two waits do add (the Oracle's
      supposition), but three dispatches in one pass still ran in turn, whether by the driver or the library's own
      barriers isn't known. Not kept; its code behind a switch meanwhile (unset, the same picture and resources).
    - **[OPEN] Two ways on** (Builder, for David): (a) the form that overlaps them for certain, one dispatch marching
      all three layers' pixels, about an hour of hers and four flights; it needs six written textures in one pass
      against the library's default of four, so either that limit is raised or the layers are packed into shared
      textures; it carries the one-frame lag, for his look if it pays. Or (b) call step 2 read and go to step 3, the
      gate: typical met (the spawn 0.70, the plain 0.88), worst not quite (the sea 1.78, its nodding 99th 2.13, a
      player's run 1.99, against 2), so step 4, the long ray cut into lengths, would follow. Builder recommends (a)
      first: the one move that could take 30 to 40% off the worst marks, the groundwork in; if it gives nothing too,
      step 2 is closed with a clear answer. *Counsel:* (a), with the layers packed into shared textures rather than
      the limit raised, by the rule that what must run stays at the default limits.
    - **(a) taken** (David: "go on with a"). **One dispatch over all three layers** (build 155, led by the frame
      before; standing still the same picture from the third frame; six written textures in a pass, a limit raised
      only while its switch is set, to be packed away if kept). One after another, then one dispatch, median ms: the
      spawn 0.69 / 0.85, the plain 0.88 / 1.00, the mountain top 1.57 / 1.78, high over the spawn 1.61 / 1.66, the sea
      1.78 / **1.11**, a player's run 1.99 / **1.34**, the sea's nodding 99th 2.14 / **1.30**. The worst marks 33 to
      38% faster, the light marks 13 to 23% slower. The waits can be overlapped, worth what the Oracle said where a
      layer's time is a few long rays (the sea, the run), nothing where it's many (the mountain top, high over the
      spawn); build 154's nothing was the dispatches kept in turn. The one dispatch has a cost of its own, 0.12 to
      0.2 ms, not yet placed (Builder's suspects: each invocation copying its layer's numbers before it's turned away;
      the grid's empty third; choosing between three textures at each store). Not kept as it is, by its pass mark.
      Against the gate: one after another, typical met and worst missed by a hair; one dispatch, worst met with room
      (1.81 at most), typical on the line (0.85, 1.00).
    - **[OPEN] Builder recommends one more round** to find and trim the one dispatch's own cost (each suspect off in
      turn; about 40 minutes and one or two sets of flights). If it comes off, this form meets the gate at both ends
      and step 4 isn't needed before going on; if not, David chooses a better worst or a better typical, or it comes
      out and step 4 follows. The lag when turning is his to look at once the numbers settle. David: "Yes, trim it."
    - **[DECIDED 2026-11-13] The far gate, as David reads it now** (to Counsel, on the one dispatch's numbers): "looking
      at the numbers it seems like a great trade overall. if everything in the far view is under 2ms we should be
      gold, i think we crossed the point where the worst frames or irregular frames were more critical than longest
      frames a bit ago. as mind numbing as this entire process is, i can not deny it is working." So the far view's bar
      is everything under 2 ms, worst and irregular frames weighed above the typical's 1 ms. *Counsel's reading:* the
      one dispatch is worth keeping even if the trim takes off only part of its own cost, since it buys the worst
      marks room; the trim still runs, and the six-texture limit is packed away if it's kept.
    - **The trim: its own cost gone, everything under 2 ms** (build 156: the per-invocation copy read in place, the
      grid's order with the base and middle layer first). Median ms, one after another / build 155 / build 156: the
      spawn 0.69 / 0.85 / 0.68, the plain 0.88 / 1.00 / 0.88, the mountain top 1.57 / 1.78 / **1.38**, high over the
      spawn 1.61 / 1.66 / **1.37**, the sea 1.78 / 1.11 / 1.41, a player's run 2.00 / 1.34 / 1.44, fast across 1.18
      / 1.63 / 1.21; the sea's nodding 99th 2.14 / 1.30 / 1.61. Faster or level at every mark against one after
      another; by David's bar, under 2 ms at every mark, medians and 99ths (the most at a player's pace, the run's
      99th, 1.62). The copy was the cost; the grid's order matters both ways (the base ahead lets the mountain top
      and high over the spawn overlap for the first time, and the sea overlap less). Kept by its pass mark, to go on
      with. Owed before anything stands on it: David's look at the one-frame lag while turning, and the six written
      textures packed under the default limit.
    - **[OPEN] Next, Builder's order for David:** (1) one more arrangement, the base and middle layer side by side in
      the same rows, which may give the sea back its 1.11 while keeping the mountain's gain (four flights); (2) his
      look at the lag, turning fast at the mountain top and the sea, frames capped; (3) the packing, then the one
      dispatch the default. Or the look first: if he dislikes the lag, (1) is moot. *Counsel:* the look first, for
      that reason.
    - **[DECIDED 2026-11-13] Finish it, cut what's unused, then his look** (David, to Builder): "hell to the yeah
      claude! you're a beast. i knew we could do it. we've really made strides. im not entirely sure i'll be able to
      see the lag without the near view present. You can go ahead with finishing up and then cutting code we won't be
      using and see if that gives us another 1% just for good measure. then i'll took a look and zoom around a bit.
      after that, what is next?" And to Counsel, of a frame's rough edges while turning: "i think squares getting
      rougher for a frame as you whip your head around will probably look organic and be almost impossible to
      discern. that kind of thing can even have a satisfying feel to it, for some reason. but i'll look for it." The
      order: the arrangement round; the textures packed under the default limit and the one dispatch the default;
      build 154's form cut, the in-turn order and the finer base kept as switches; the closed measuring switches
      taken out of the shader and flown alone for his "another 1%"; then his look. **The lag look is owed again once
      the near eye draws**, by his doubt it can be judged before.
    - **The arrangement round** (build 157, the base and middle layer side by side in the same rows): level or 1 to 3%
      faster at every mark against build 156, slower at none (the sea 1.38 ms, its nodding 99th 1.58; high over the
      spawn 1.32; a player's run 1.41). Builder's pass mark asked 5% at the sea and got 2%; she kept build 157's
      order anyway, in the open, since it's better or level everywhere for no more code and her 5% was set too
      coarse. *Counsel:* agreed; the mark guarded against complexity for a small gain, and there's none added.
    - **Finished and cut** (builds 158 and 159). Build 158: the layers' distances in one texture, so a pass writes
      four textures and the game is back on the library's default limits; the one dispatch is the far eye's march
      (`INCUS_FAR_SIDE=0` marches in turn); build 154's form out; the frame the layers are remade in runs the dispatch
      three times, so there's no coarse frame. The same as build 157 within 0.7%; card memory 973 MB against 935 in
      turn (two shared textures of distances; a [LATER] to win most of it back). Build 159 cut the closed measuring
      switches from the march: about **2% at every mark** (Builder guessed 0 to 1%). David: "thats fine, i didnt expect
      much, just hygiene". **Step 2 done enough to go on**, against this morning's build 152: the spawn 0.69 → 0.66,
      the plain 0.88 → 0.87, the mountain top 1.57 → 1.34, high over the spawn 1.61 → 1.30, the sea 1.78 → 1.35, a
      player's run 2.00 → 1.38, the sea's nodding 99th 2.14 → 1.54. Everything under 2 ms; the most at a player's pace
      1.56.
    - **His words while it flew** (to Builder): on the one dispatch, "dispatching everything at once seems to have
      worked out very well, a wholly tonic effect, but isnt that megakernally, i wonder, whats the difference". Builder:
      one job launched once, not different jobs folded together, so not the megakernel; but the march still marches,
      lights and shades in one function, which stays open. Today priced splitting: launches in a row add their waits,
      so "small passes" means small in code, not many launches a frame. On fog: "fog will probably become some kind of
      post processing with weather" (each pixel's distance is already kept beside its colour, so it fits the finish).
      On light: "right, we should be able to offload light to the cpu" (the plan's "alongside" item).
    - **His look at build 159** (on his "hell yeah lets see"): "fantastic". 110 seconds, frames capped, a clean quit,
      the card at 57 °C and 41 W; no lag and no flash named on arriving at a mark. The lag's look stays owed once the
      near eye draws.
    - **The gate's owed items wait for light on the processor** (David, to Counsel: "why don't we skip 1. we're
      putting light on the processor, probably."). His look at the rings and the Intel card's numbers would be read
      on a far eye that's about to change, so they come after light read from the cell, made on the processor, which
      is next.
      - **His look at the rings, met** (2026-11-14, build 167, to Builder): "i can't see any rings. the crossover
        section looks nice when i pull it down, but its slower, so i'll save that for last." The gate's line now owes
        only the Intel card's numbers (next time Optimus is on); a longer hand-over goes on his polish list, priced
        when he takes it up.
      - **His look at the initial blend** (to Builder): first, "i couldn't tell the difference but noticed i couldn't
        press back after i hit initial blend, except by default" (the far blend's choice sticks while the textures
        load); then "ah yes. the blocks go away mostly with the hardware blend, after it loads in. you don't need to
        do anything with it." So the sticking stays as it is, on his word.
- **[DECIDED 2026-11-14] The cells of 2 to 16 before the skyline look; the 16s' ring alone first** (David, to Builder,
  asked whether to swap the plan's order and whether the first build is one ring alone): "yes and yes, you can
  proceed. hopefully all this work, being much shorter than the longest rays, goes on pretty much alongside everything
  else and doesn't cost much in time." Why (Builder, Counsel agreeing): the soft skyline is as wide as the cells it's
  read from, and near land is read from 32-block cells today, so the finer cells change the very picture the skyline
  talk would judge; and he'd said he'd wait for them to judge the blocky shapes. The first build: a ring of 128 × 128
  columns of 16 round the eye (old's size), made on a thread of its own, the frame only sending what it's handed and
  moving the ring when it's whole; the 32s past its edge with a blended edge; the 32s' column light for now. Builder's
  guesses (written before code): far-looking marks within 2% of build 167, ground marks 5 to 15% dearer, under 30 MB
  on the card. Then the scatter stage's woods, then shapes placed by hand, each placed with him. The plan's order
  changed to match.
  - *Built:* build 168 (72653a9), unflown, checked off the screen: every 32 exactly its eight 16s added (163,840
    cells); the ring off, byte for byte build 167's; on, no hole or line at its edge, skylines and near slopes
    crisper, and at his first spot the star facets half the size; a moved ring draws byte for byte what one made in
    place does. **A miss, in the open:** the first ring takes 2.3 to 3.6 s while the world loads (guessed half a
    second), off the frame; a short move 0.06 to 0.12 s. A shading flaw written down: inside the ring the light is
    still the 32s'. Owed: his look, then fourteen minutes hands off for the ring on against off (the card's time
    and memory).
  - *His look at build 168* (to Builder, with a screenshot looking straight down): "looks good, i think you already
    know about this in the pic?" and "referring to the square border with the light bleeding around it". **A
    correction, in the open:** Builder's "no line at the ring's edge" was wrong; none of her pictures looked down.
    Not the ring's light either. The cause: the hardware's blend reads from the brick holding the least of the eight
    cells round a point, and bricks were made only where they hold land of their own, so just under ground lying at a
    brick's floor there was none, and the march fell back to the initial blend with the rock switch's cast. Build 169
    (e97068f), unflown: the ring's maker makes those bricks too; the square gone off the screen. Also corrected: a
    moved ring draws "to a last digit" (1 of 255 in 0.1% of pixels), not byte for byte.
  - **[OPEN] The same flaw in the 32s and up**, wherever land crosses a brick's floor (every 256 blocks up at the
    32s): mending it wants about a third more bricks in the textures, some 160 MB by Builder's estimate, put to
    David as a memory call.
    - **[DECIDED 2026-11-14] Mend it** (David, to Builder): "yeah 160mb is no biggie. go ahead and launch". Builder's
      next build once he's closed the game, the real figure measured and its pass mark written first. Build 169
      launched for his look meanwhile.
    - *Built:* build 170, unflown. **A miss, in the open, the good way:** 3% more bricks and nothing more on the card
      (the textures stay 573 MB), not a third and 160 MB; at the 32s a brick is 256 blocks tall and most land lies
      inside one (the ring's third came from its bricks being 128 tall, with plains at 130). So he agreed a cost
      that didn't come. A patch of straight-edged plates at his first spot goes; next to nothing at the marks.
  - *His look at build 169* (to Builder): "there was a visible ring of brighter sections before the world had
    finished loading, and it vanished when it loaded. i don't think it's possible for a player to ever see it, so
    unless its indicative of a problem that should be addressed, i think you got it." Builder, off the screen: the
    initial blend's own (a brighter line at the ring's edge by it, none by the hardware's), seen only till the
    textures are in. Written down, not mended.
  - *Flown* (build 170, on "sweet, good work. go ahead with your flights."; ring on `logs/flight-1794690079`, off
    `-1794690468`; checked by Counsel). The 16s' ring on, against build 167, the card's whole at the median: the spawn
    31% more, the sea 18%, the sea nodding 24%, a player's run and high over the spawn 12%, the mountain top 5%; the
    bare plain 31% *less*. Every median at or under 1.35 ms, every 99th at or under 1.57; nothing more on the card
    (1,015 MB in use, ring or no ring). **Misses, in the open:** far-looking marks guessed within 2% (5 to 18%
    dearer), the spawn 5 to 15% (31%), the plain dearer (a third cheaper); the ring's code costs 2 to 3% with the
    ring off. Flags: one frame at 2.24 ms high over the spawn (its 99th 0.99, not explained); over the sea the
    4 × 4's longest ray reaches the step limit (768), no hole in the picture, to watch at the 8s; fast across the
    world, putting the far cells on the card a fifth dearer at the 99th in both flights (build 170's bricks, most
    likely). Builder advises the 8s alone next, flown before the 4s and 2s; his word asked.
    - *Counsel, to watch:* the sea nodding's 99th at 1.57 leaves about 0.4 ms under his 2 ms bar for the 8s, 4s and
      2s together, if each costs what the 16s did. Lazy rays come after them and aim at exactly that time.
  - **The 8s alone next** (David, to Builder, asked whether the 8s come alone and are flown before the 4s and 2s):
    "yes". Build 171, unflown: the 8s' ring beside the 16s', the ring's code now holding several. Checked off the
    screen: every 16 exactly its eight 8s; with one ring, byte for byte build 170's; from straight above, no square
    at either ring's edge (under 1 of 255); a moved pair of rings matches one made in place to a last digit; the
    sea's far edge whole; at his first spot the small facets smooth into a slope. The 8s' first ring 1.4 to 2.1 s at
    load. Owed: his look, then fourteen minutes hands off, both rings against the 16s alone.
    - *His look* (to Builder): "looks pretty good". The flights next, on his go.
    - *Flown* (on "go ahead"; both rings `logs/flight-1794697458`, the 16s alone `-1794697843`; checked by Counsel).
      Both rings against the 16s alone, the card's whole at the median: the spawn 9% more, high over the spawn 11%,
      fast across the world 7%, the sea 2%, the run 3%, the mountain top 4% less; **the bare plain 80% more** (0.535
      → 0.964 ms; nodding 70%), where Builder guessed within 10%: a miss by far, said in the open. The eye stands on
      the ground there, and rays grazing it walk the 8s' small steps for 500 blocks (921 of the base layer's rays
      over 256 steps, where none were). Every 99th still under 2 ms (the most 1.63); nothing more on the card.
    - *The trend, put to him before the 4s* (Builder): each finer ring halves a grazing ray's step near the eye; if
      the 4s cost the plain what the 8s did it stands near 1.7 ms, and the 2s would take it past his 2. Her advice:
      look at the grazing rays' steps before building the 4s. Counsel agrees: the per-column height skip for
      skimming rays has been on the list to price since build 140.
    - *A correction, in the open (Counsel, the same day):* the line above is wrong about the height skip, and Counsel
      said it to David too. The Oracle's third part (section 4) moved it to go with the skyline by cover: a ray summing
      the haze one cell deep over the ground can't skip the haze it sums, and inside the rings that haze is the 8s'
      and 16s'. So the skyline look (haze or cover) and the grazing rays' cost are tied; the cut (her step 4) is the
      lever that keeps the haze. Put to the Oracle in the prompt of 2026-11-14.
    - **[DECIDED 2026-11-14] A deep dive before the 4s and 2s** (David, to Builder): "lets take a deep dive into how we
      can get the finer rays working simultaniously in time with the rest of them." And: "lmk if i need to start a
      testing software for you" (not yet; Nsight if the cut is built). Builder's reading: the rings add no short rays,
      they lengthen rays already long; the way to have the near work beside the far is the Oracle's step 4, shelved
      when the gate was met, the ray cut into lengths at the rings' edge, marched side by side in the one dispatch,
      joined front to back; whether it pays turns on whether the card is waiting or working on the plain. Build 172:
      two measuring switches, three flights (whole, the near length alone to 1,024 blocks, the far alone from 1,024),
      its pass mark written first.
- **[2026-11-14] The Oracle called at the finer cells, and its first reply** (David sent Counsel's prompt,
  `journal/vision/oracle-prompt-2026-11-14.md`; the reply, verbatim, is kept at its end). Its reading, held loosely:
  the plain's time grew more than its longest ray (a third more steps, 80% more time), so something besides length
  changed; the 4s and 2s at 128 cells a side would lie under the near eye's ground; **ring width, not ring count, sets
  how big the facets look** (a cell about 10 screen pixels at a ring's edge, 21 where the next takes over); and over
  flat ground long strides through the haze are exact, so a height table *can* help while the haze stays, by telling a
  ray the haze ahead isn't changing (correcting its own section 4, and so Counsel's correction above). What it expects
  to advise: strides sized by how fast the haze changes, before the cut; the creases from storing "how full" with a
  hard stop at empty and full, to be tried without the stop on his mountainside; the 4s and 2s decided with the near
  eye; switches fixed when the program is built. Its five questions for David and three for Builder are in the file.
  - *Builder's reading, to David* ("what do you think, Builder?"): she agrees on ring width (her "about a pixel" for
    near land was wrong at these ring sizes), on deciding the 4s and 2s with the near eye (the near eye's 336 not
    confirmed from old's code), and on trying the creases' storage off the screen. She adds: the plain's extra cost is
    likely the number of long rays (130 to 921 over 256 steps), which the steps-by-kind counter will test; strides put
    logic in the hottest loop, to be priced by flight; switches fixed at build, strongly. Her order: build 172's
    flights, the counter and a steady watts reading, the 4s and 2s held for his answers to questions 1 and 5, then the
    no-hard-stop storage off the screen.
  - *Build 172's three flights* (on "you may proceed with the tests"; whole `logs/flight-1794701437`, the near length
    alone to 1,024 blocks `-1794701828`, the far alone from 1,024 `-1794702221`; checked by Counsel). The near length
    alone is cheap and short everywhere (0.36 to 0.42 ms at the ground marks, no near ray over 158 steps). At the sea,
    the run and the mountain top the far length alone costs 86 to 102% of the whole, so the rings' near work adds a
    tenth or less there, and a cut could give back that tenth at most. **A fault in the measurement's design, in the
    open (Builder):** on the plain the far length alone costs *more* than the whole (1.18 against 0.97 ms; guessed
    0.6): with the near land gone the picture changes, and the 4 × 4 layer, which marches nothing on the plain when the
    ray is whole, marches 728 long rays. So it doesn't answer "waiting or working" on the plain; it does show that far
    lengths behind near land aren't cheap unless told not to start, which any cut must respect. David wrote from his
    phone during two of them: the frame clock's worst 12 to 14 ms at the sea; medians and 99ths tight, only medians
    read. Next: the longest ray's steps by kind (the Oracle's question 7), a variant that exists only when asked for;
    then a steady watts reading.
  - *The plain's steps by kind* (build 173, 75e042f: two instruments, the game's shader untouched; drawn off the
    screen at the flight's size). **The wall is hopping, not haze.** The base layer's 64,000 rays: with both rings
    2.84 million steps, 2.39 million of them skips and 0.41 million samples; with no rings 1.88 million, 1.36 skips,
    0.48 samples. So with the rings the rays take *fewer* samples; what grew is skipping. The longest ray's 533 steps
    are 422 hops over its own level's columns, 49 coarse skips and 65 samples (13 in haze); the 952 rays over 256
    steps spend 233,000 of their 331,000 steps on those hops. Blends fallen back to the code: about 1,000 of 2.84
    million. Why: a ray skimming a little over flat ground is under the top its coarser column gives (rounded up to
    the coarse cell), so the coarse skip fails and it hops the fine columns one at a time, 8 blocks a hop in the 8s.
    - *Builder's reading:* a table of each column's true highest ground, level by level, lets such a ray clear a
      column of 128 or 512 blocks in one hop, the haze untouched. That's the plan's column-height table, which the
      Oracle's section 4 had set aside as paying only with the skyline by cover; these counts say it pays with the
      haze kept. Strides sized to the haze and the cut both act on samples, a sixth of the steps. The far light's
      column tops are averages, so they can't serve. The record went round on this today (Counsel named the table,
      corrected it by her section 4, she loosened section 4 herself); the counts settle it. Also: every standing now
      reports the card's watts typically beside the highest (her question 8).
  - **[DECIDED 2026-11-14] Build the height table** (David, to Builder, shown the steps by kind): "go ahead and build
    the height table, and keep me informed about how you're doing it and how it works." Build 174 (819f422), unflown:
    every column of every level carries the height of the highest thing in it and its eight neighbours (the blend
    reads half a cell into the next column), kept after the bands in the same list, a column's number a fixed offset
    from its band's (28 MB; no new buffer, so the default limits hold); the bands' rounded tops while the world loads,
    the true ones worked out on a thread and sent whole in 0.09 s; the rings' makers do theirs. The march's skip asks
    it, the biggest column first. Off the screen, the plain with both rings: 2.84 million steps to 1.17, hops over a
    ray's own column 735,000 to 116,000, rays over 256 steps 952 to 16, the longest ray 533 to 326; samples as they
    were; both count guesses met. A test holds the numbers true against the doorway's columns, block for block.
    - *The picture:* no shift overall, a scatter of about a part in 255, a few horizon pixels flipped between sea and
      sky; at his first spot the slope's fine contour lines a little grainier. David: "let me see the change before
      discarding it. for all we know it might be nice." His look first, then fourteen minutes hands off, the table
      against `INCUS_FAR_TOPS=0`. Pass mark: the plain from 0.97 ms to 0.6 or under.
    - *His look* (to Builder): "it looks great, builder. go ahead"; of the pictures, "from those pics i cant see anything
      wrong".
    - *Flown* (table on `logs/flight-1794705815`, its numbers off `-1794706206`; checked by Counsel). **Mixed, and a bad
      miss on the plain, in the open.** Against build 172 (the same rings, before the table's code), the card's whole
      at the median: the sea 17% less, a player's run 19% less (now under build 167's, before any ring), the sea
      nodding 19% less, fast across the world 15% less; but **the bare plain 4% more** (1.01 ms, guessed 0.6 or
      under), the spawn 16% more, high over the spawn 12%, the mountain top 9%. Every 99th under 1.5 ms. The long
      rays went as counted (the plain's over 256 steps 910 to 15, the sea's 631 to 71, the run's 1,088 to 4), so three
      fifths of the plain's steps went and its time didn't move: **the hops were cheap; the counts weren't the cost
      there** (the Oracle's third part had said it: stop counting steps as cost). Counsel had told David each hop was a
      wait worth saving; on the plain it wasn't. The dearer marks: the skip now tries three columns where it tried two,
      biggest first, and a ray near the ground fails all three before it samples; the table's code without its
      numbers is 10 to 20% dearer than build 172 everywhere, that price alone.
    - *The mend next* (build 175, Builder): the ray's own column first. A column's number only grows with its size, so
      a ray not clear of its own column is clear of none, and samples after one look (build 172 took two); a ray
      that's clear then asks the bigger ones and takes the longest hop. The table on the card: 29 MB.
    - *Build 175, counted and not flown* (its pass mark first, 4ed7ede; the count off the screen, 9e518b7): the same
      steps as build 174's, but the reordered skip takes **more** looks at every mark (the plain 54%, the sea 44%, the
      spawn 6%, the mountain top 8%), so the code is build 174's again. The reason given for the dearer marks ("a ray
      near the ground fails all three looks") is withdrawn in the open: only 100,000 to 150,000 steps a mark do that;
      Counsel had passed it on to David too. And looks may not be the price either: on the plain build 172 took about
      3.3 million looks and build 174 1.39 million, and 174 is 4% dearer. **Steps, hops and looks have each been
      counted and each failed to say where the time goes.**
    - **Nsight on the plain** (David, to Builder: "are you using Nsight to check on what the plains time is actually
      spent on?"; she wasn't, and agrees it's the right tool): four traces of build 174 as it's built (the plain with
      the table and without its true numbers, the spawn, the sea), about eight minutes hands off; no flight, compile
      or code change.
    - *Read* (build 175, 467bc69: build 174's march plus a switch writing the shader's text to the log, so Nsight's
      hot lines read against our code; traces on "go ahead", ended 20:07; write-up 42dbc9a). On the plain the far march
      takes 0.896 ms with the card **95% empty** (4.7% of its room in flight); at the spawn 17% in flight for 0.421 ms,
      two thirds more rays' time in under half the time. So the plain's time isn't work: the pass lasts as long as its
      last few rays, and only the 8 × 8 layer marches there. The skip's looks are the biggest named part (a tenth of
      the march on the plain, a sixth with the table's numbers off); the wait for the height table's number is the
      hottest named line at every mark; a texture read about 420 cycles, a buffer read about 100. **Builder's reading,
      not yet shown:** the time is the longest chain, its steps in a row and each step's reads in a row; the table took
      39% of the steps off the plain's longest ray but put three reads in a row in a hop where there were two, so the
      chain stayed as long. It fits the plain, the spawn and the sea, and why counts over all rays never placed it.
      **Next:** the three looks' numbers read at once (the same skips and picture), built off the screen with its pass
      mark first, one flight on his go. A fault in the open: the sea's trace stood at the spawn (a stray "; " in the
      script), so two spawn traces (they agree) and none of the sea.
    - *Build 176, the three looks read at once* (325b42f; pass mark first, 2de71c4; flown on "go", ended 20:36,
      `logs/flight-1794710172`, checked by Counsel): 4 to 8% cheaper than build 174 at eight marks of nine (fast across
      the same); **its pass mark missed**: the plain 0.934 ms, not under 0.90, and the three near-ground marks still 3
      to 9% over build 172's. Kept, being the same picture and cheaper. The height table as it stands, against build
      172: the long-ray marks 4 to 23% cheaper (the sea 1.401 → 1.112, the run 1.312 → 1.007), the near marks 3 to 9%
      dearer. **Builder's reading withdrawn as the reason** for the dearer near marks: its sum said 27 to 41% off the
      longest chain, the plain's march fell 8%. What holds, small: the reads' order mattered, their number didn't
      (three quarters more reads on the plain, 8% cheaper). Nsight's own names for the waits: 14% of the march's
      samples (of the 38% it names) on memory, 21% on the program's own sums and branches; each mark's march over its
      longest ray's steps 2.1 to 2.7 microseconds a step at seven marks of nine.
    - **Next, put to him:** one flight with no code changed, the step limit at 160 (`INCUS_TEST_STEPS`), to see
      whether a mark's time is its longest ray's chain, before anything more is built on that idea.
    - *Flown on "go"* (pass mark first, fd1a0ca; ended 20:47, `logs/flight-1794710853`, checked by Counsel; the picture
      wrong on purpose, rays cut short). **Two kinds of mark.** *Waiting marks* (the plain, the sea, the run, both
      nodding): with rays stopped at 160 steps the march is 29 to 45% shorter (the plain 0.825 → 0.494 ms, the sea
      0.999 → 0.554): their time is their longest rays' chain, shown now, not summed. *Working marks* (the mountain
      top, high over the spawn, fast across): rays of 235 to 302 steps cut to 160 and the march moves 2 to 9%; the card
      draws 110 to 114 W there against 73 to 96 on the plain and the sea; what their time is spent on isn't looked at
      yet. The spawn (no ray over 160) the same to the last digit. Pass mark met in part, as its third line allowed.
      What it explains: the height table shortens long chains and adds work to every step, so it paid at the waiting
      marks and cost at the working ones.
    - **The prize of lazy rays, measured** (the most a step budget of 160 a frame could give): the far eye's whole at
      the sea 1.112 → 0.666 ms, the run 1.007 → 0.752, the plain 0.934 → 0.604; the dearest mark would be high over the
      spawn at 1.040. David's morning thought, borne out: "it would seem we have to reduce the longest rays, one by
      one in order of length, in order to actually free up warps more rapidly, consistently." Builder puts to him
      Nsight at a working mark, and the lazy rays' talk with its prize known; the plan's order (the cells of 2 to 16,
      the skyline talk, then lazy rays) is his and Counsel's to keep or change.
- **[DECIDED 2026-11-14] Bring down the high spot** (David, to Builder, after the step-limit flight): "well the goal
  should always be to bring down the high spot. i was just about to ask if it is time for lazy rays. go ahead and do
  your test, and send your complete assessement of how lazy rays could get us that last bit off the plains to
  counsel." So speed work is ranked by what it takes off the dearest mark (now the sea 1.112 ms, high over the spawn
  1.080, the mountain top 1.071, the run 1.007, the plain 0.934), reported dearest first; kept in the shared memory as
  a standing habit. Next: three Nsight traces at the mountain top, high over the spawn and the sea (what's expected
  committed first, 9bacd38; taken, ended 21:00, each at its own mark), two counts of where along a long ray its steps
  lie, then Builder's complete assessment of lazy rays, to Counsel.
  - *Builder's assessment of lazy rays* (in full in `journal/docs/experiments.md`, "Lazy rays: Builder's assessment",
    d9a9682). In short: yes for the plains, about half off, but "lazy" must begin at the 16s' edge (about 1 km), since
    the long rays' steps lie in the 8s and the 32s (1 to 5 km), not at the far horizon (past 11 km, 3% of the plain's
    steps and no chain); and only on top of the long ray cut into lengths (the Oracle's step 4), which carries no lag
    and gives the plains their prize by itself (estimates: the far eye's whole on the plain 0.93 → about 0.45 ms, the
    sea 1.11 → about 0.56). A step budget on the screen's squares, as first worded, can't be built: a ray resumed
    after a turn isn't its pixel's any more, so kept rays are kept by direction. **The high spot doesn't fall with
    it:** the mountain top (1.07) and high over the spawn (1.08) work, not wait; the far eye's worst would go 1.11 →
    1.08. For them two facts are missing: how much is the 4 × 4 layer's work, and whether 17 to 19% in flight is the
    program's ceiling (registers, read off Nsight's window by David). Her order: (0) per-layer counts in every report
    and one flight at 1 step, David reading the registers; (1) the cut, for the long rays only, its pass mark the
    step-limit flight's 0.494 and 0.554 ms; (2) the working marks, by what step 0 finds; (3) far lengths kept by
    direction, when only they can save something, its first build David's look at land 1 to 5 km off while running.
    The skyline by cover shortens the same chain by another road (176 of the plain's longest ray's 326 steps are
    samples close over the ground) and adds to the cut. Her sums missed twice today; past the step-limit flight it's
    sums.
    - *Counsel's note to weigh with David:* the Oracle's first reply expected to advise strides sized by how fast the
      haze changes, before the cut, since it keeps the look and does less work where the cut adds work; those strides
      go at the same samples close over the ground that cover would. The assessment doesn't price them; they belong
      beside the cut and cover. And the Oracle's five questions to David are still unanswered; her fourth part, with
      today's measurements (the height table, Nsight, the step limit, the levels' table), would order these.
  - **The working marks are full, and it's the march's size** (2026-11-15, just past midnight; David read Nsight's
    window at the mountain top's trace; Builder's write-up 98ccfbd). The march: 168 registers (166 live), 12 groups of
    rays fit of the 48 a unit holds (a quarter), 8,530 instructions, 85% of the samples; its top waits 35% "Wait", 17%
    "Long Scoreboard". The traces' 17 to 19% in flight is seven tenths to three quarters of that ceiling, averaged over
    a pass that fills and empties: full. Old's march had 125 registers and 16 groups; by the three rows, 16 groups
    would want about 128 registers, 20 about 100 (a rule read off three rows, not NVIDIA's). The Oracle named the
    trade on 2026-11-12 ("On a card that's working, every number held is room lost"): the numbers kept in hand and the
    height table's third row were bought for the marks that wait, and at the marks that work they're room lost (fits,
    not shown). A guess to check by taking it out: the march's 4,608 bytes of local memory, 72 a ray, the levels'
    view's six-tint table.
    - *So the high spot is two jobs* (Builder): the marks that wait (the sea 1.112, the run, the plain) want the long
      rays cut into lengths; the marks that work (high over the spawn 1.080, the mountain top 1.071) want a **leaner
      march**. The three dearest are within 4%, so the worst falls far only when both are done. A leaner march changes
      no picture and is priced by a trace and David's reading each time; Builder wants old's twin, several variants
      in one trace, so one screenshot prices a batch. A fault in the open: she gave him the trace file's name from
      memory, a second out; he found the right one.
    - *Counsel:* this is David's "no megakernels" goal of 2026-11-12, now measured as the working marks' cure: a march
      split into smaller passes, each holding fewer numbers, fits more rays on the card.
  - **[IDEA 2026-11-15] All three** (David, to Builder): "seems like that last bit of performance is behind leaning out
    the rays, lazying the rays, and breaking the shader into smaller passes. do you think you can do all that?"
    Builder: yes, all three can be built, and none promised (her sums missed twice on 2026-11-14; the measurements
    held), so each gets its pass mark and a flight and may come out again. Leaning the march: the gain known only after
    a register audit, no look, his Nsight screenshot each batch. Lazy rays: the cut into lengths, fairly sure, one
    look; far land kept between frames from about 1 km, the big one, his eye decides. Smaller passes: the cut is the
    first (march the lengths, join, finish); further is least sure (old's mixed results; passes in a row add waits on
    a waiting card, so side by side). She reminded him the far eye is already under his 2 ms (worst 1.11), so this is
    headroom for the cells of 2 to 16 and the near eye; whether to take it now or after them is his and Counsel's.
    Nothing ordered or built on it yet.
  - **[DECIDED 2026-11-15] Now, and smaller passes first** (David, to Builder, asked now or after the near eye, and
    the cut first or not): "i don't think we should leave anything undone. if atomizing the megakernal a bit with the
    main shader being broken into passes *can* give higher performance at no cost, that will translate directly to
    more headroom for prettying it up. i think we should probably start with trying to break it into smaller passes
    to fit more, and then lean and cut the rays." So: (1) the march broken into smaller passes, to fit more groups of
    rays; (2) the march leaned; (3) the long rays cut into lengths. Builder had proposed the cut first; his order
    stands. Counsel had recommended leaning and the cut now and far land kept between frames after the near eye;
    that last stays as it was. Builder starts with an instrument, not a guess: variants of the march (hops only,
    samples only, without the finish, without the hand-over's second read, and so on) compiled and run on a token
    dispatch that touches no picture, so Nsight lists each one's registers and how many groups fit, one screenshot
    from David pricing the batch; it's the register audit the leaning needs too. The plan's 2.2 gains the line.
  - *The audit, traced and read* (build 177; Nsight's counters went overnight and came back only with a restart,
    cause unknown, the setting on all users throughout; traces at the mountain top and the plain, ended 06:28; David
    read the rows off Nsight's window, three screenshots). **The split guessed at doesn't pay; trimming does.** The
    control as the game's (168 registers, 12 groups). A hop-only pass would fit 28 groups (68 registers), but it's
    under a seventh of the base layer's work at the mountain top and the sampling half keeps all 168: off. One part
    out at a time gives next to nothing (the hand-over's second read 8 registers, the code's blend 1, the rest none);
    **all six together give 40: 128 registers, 16 groups.** So several places in the march each need about 166 at
    once, and the count is the highest of them. Two are paths few steps take (on the plain 0.6% of steps mix two
    levels, 0.1% fall back to the code's blend), and every ray pays for their room: the pass worth making takes the
    rare, heavy work out of the march every ray runs. Builder's guesses missed again on the single parts (15 to 30
    for two that gave 8 and 1) and on the local memory (not the views' tints). On the side: without the skip the base
    layer costs nearly twice, so the height table's skip is worth about half its work at the mountain top; the base
    layer is about two fifths of the far eye's work there, the 4 × 4 the rest. Next: a second batch, the fewest trims
    that reach 128, and whether 102 (20 groups) is in reach.
  - *The audit's versions, timed* (0e5f8aa, from the traces' own tables). Crossing the air to the first sample costs
    0.008 ms at both marks: no pass for it, so the hop/sample split is closed for good. **Two rare paths cost every
    ray:** the hand-over's second read out makes the plain's march 20% shorter and the mountain top's base layer 9%;
    the code's blend out, 14% and 10%; yet on the plain only 0.6% of steps mix two levels and 0.1% take the code's
    blend. Builder's reading, not shown: rays march 32 to a group in step, and when one takes a slow path the other 31
    wait. All six trims together: 30% off the plain's march and 23% off the mountain top's base layer, and 16 groups
    fit where 12 do, so moving the rare, heavy work out of the march helps both kinds of mark. The base layer alone
    waits even at the mountain top (7% in flight); the fullness there is the 4 × 4's. The height table's skip is worth
    a third of the base layer's march at both marks. Nsight's window opened and closed didn't take the counters; why
    they went stays unknown. Next, with no time of David's: each trim's picture against the whole's off the screen,
    to sort the free ones (the views; the code's blend once the textures are in; the floor) from those that change
    what's seen (the hand-over, his look); then the free ones out of the march into programs of their own, build 178.
  - *Build 178, the first piece broken out* (9e4c4c6, unflown; pass mark first): two programs from one text by one
    line fixed when each is built. The whole march stays for while the world loads, the initial blend and the views;
    the far eye's own program, once the textures are in, carries neither the code's blend nor the views, and where the
    textures can't answer as they stand it reads the lowest cells and mixes rock in, takes the first column's share,
    or takes rock or nothing from the bands. `INCUS_FAR_LOADED=0` keeps the whole march on to fly against. The code's
    blend wasn't free to drop bare: it answered for everything within half a cell of the world's floor, at the coarse
    levels everything near sea level, the far end of the long rays. Off the screen: build 176's picture at five marks
    (the most 1 of 255), the plain's steps the same, 31 tests pass; a first try four rays lighter at the mountain
    top's horizon (far land at the map's south edge) mended before the build was cut, in the journal. Pass mark: the
    plain and the sea 8% or more under the whole march in one session, every mark at or under, under 5% on the plain
    a miss; Nsight about 167 registers and 12 groups still (this piece alone buys no room; the hand-over's next).
    About 15 minutes hands off asked (one trace, two flights) and one screenshot.
    - *Flown on "go"* (ended 08:17; the new program `logs/flight-1794751834`, the whole march kept on `-1794752223`,
      one session; checked by Counsel): 2 to 7% cheaper at every mark but the spawn (the same). **Short of its pass
      mark:** the plain 5.8% (8% wanted; over the 5% that would have been a plain miss), the sea 2.2% (missed); every
      mark at or under, met. Why less than the audit's 10 to 14%: the audit's version put nothing where the code's
      blend had been (a wrong picture, rays meeting less); this build reads a texture there. **An audit by removal
      gives ceilings, not prices.** Kept. The far eye's whole now: the sea 1.090, high over the spawn 1.060, the
      mountain top 1.042, the run 0.976, the plain 0.886, the spawn 0.466. Waiting on his reading of the march's row
      from the mountain top's trace; then the hand-over, and a second audit on this program for the fewest changes
      that fit 16 groups.
    - *His reading of build 178's row* (10e13d2): 167 registers (was 168), nothing put by in memory (was 18 words a
      ray), 12 groups still, as its pass mark said. Read off the rows, not from NVIDIA: the compiler seems to choose a
      number of groups and put by in memory what won't fit the registers that leaves (12 groups up to about 170, 16 at
      about 128), so a program's need is its registers plus what it puts by: the whole march 186, build 178's 167; to
      fit 16 groups it must come to about 130. **Build 179**, the audit's second batch (`INCUS_FAR_AUDIT=2`): fourteen
      versions of the new program, the hand-over's second read and the things kept the whole way (the floor's cells,
      the third row, the rings' boxes, what the finish needs), alone and together; ceilings, as build 178 taught.
      About 7 minutes hands off (two traces) and one screenshot asked.
    - *Traced on "go" (ended 08:32) and read by David* (33f92dd). The hand-over's second read is the fullest place left:
      14 registers (167 → 153) and 10 to 16% of the base layer's march; the floor's cells, the third row and the finish
      6 each; the rings none. With the hand-over out and any one of the three, the compiler takes 16 groups but puts 15
      to 22 words a ray by in memory; all of them out reaches 127 with nothing put by. The compiler's choice isn't a
      line at 128 (at a need of 150 it took 16 groups, at 153 and 142 it kept 12): the rule read off the first rows was
      too simple. **16 groups bought with words put by don't shorten the base layer** (0.97 to 0.99 of its time, where
      the hand-over out at 12 groups reads 0.84 to 0.90): the base layer alone waits; what 16 groups are worth where
      the card is full the audit can't say. Not understood, written down: the floor, the third row or the finish out
      alone 5 to 12% *longer*; how the compiler chooses; the far eye's pass at 1.5 ms in the mountain top's audit trace.
    - **Build 180** (no change to the march): a measuring switch (`INCUS_FAR_AS`) marches one of the audit's versions
      as the far eye's own program, the picture wrong on purpose. Four flights in one session: as it is; the hand-over
      out (12 groups); the hand-over and the floor out (16 groups, 15 put by); those and the third row out (16, 7 put
      by). Pass mark: a tenth or more for 16 groups at the mountain top and high over the spawn means the hand-over and
      one other piece are next; under 5% means the registers aren't chased further and the cut of the long rays comes
      next (the hand-over still worth its shorter step). About 26 minutes hands off asked.
    - *Flown on "go ahead"* (ended 09:09; `logs/flight-1794754181` to `-1794755350`, checked by Counsel). **Fitting
      more groups is worth little.** Where the card is full, 16 groups against 12 with the same hand-over out: the
      mountain top 2 and 4% less, high over the spawn 2 and 5.5% less by the card's whole (Builder's 2 and 7%, likely
      the march's own); three of four under 5%; her guess 5 to 15% was high. Where the card waits, 16 groups bought by
      spilling cost 3 to 4% (the plain, the plain nodding). The hand-over's second read out, as a ceiling: 13 to 14% on
      the plain, 4 to 6% at the mountain top, high over the spawn and fast across, next to nothing over the sea, the run
      and the spawn. So by the mark's own rule **the registers aren't chased further as a goal.** Kept from the stretch:
      build 178 (the picture kept, 2 to 7% off, the march's need 186 → 167). The far eye's whole now: high over the
      spawn 1.096 ms, the sea 1.092, the mountain top 1.056, the run 0.977, the plain 0.886, the spawn 0.466. **Not
      answered:** what the mountain top's and high over the spawn's time is (not their longest rays, not room, not the
      hand-over); the 4 × 4 layer is its largest known part (the mountain top's base layer alone 0.71 ms of its 0.95).
      Builder puts to him the long rays' cut next (rays stopped at 160: the sea's march 45% shorter, the run's 29%),
      the third item of his order, with the hand-over as a smaller follow-up.
      - *Counsel's note:* the cut goes at the sea and the run, which wait; the high spot is now high over the spawn
        (1.096), a working mark the cut doesn't touch, a hair over the sea. By his rule the worst only falls once the
        working marks' time is placed, and its largest known part is the 4 × 4 layer: how much of it is earned, and
        where (a look rule), not how big the march is.
  - **[DECIDED 2026-11-15] Go on with the cut** (David, to Builder, on the four flights and what's next): "go on with
    the cut". The smaller passes and the leaning rest where build 178 left them; the long rays cut into lengths is
    built now. Builder's design: a ray that was long the frame before is cut at set distances (from about a kilometre,
    every half doubling), each length marched by its own thread in the same dispatch, a small pass joining them front
    to back; the frame waits for the longest length, not the longest ray. Nothing kept between frames but the list of
    which rays were long, riding in the words the layers already pass on (no new buffers, the default limits kept). A
    ray that turns out long without notice keeps the frame before's pixel for one frame and is cut from the next: no
    holes, no long frames (a one-frame lag at newly long rays, for his look). Three stages, each with its pass mark:
    the list alone, the picture untouched; the lengths and the join, one flight and his look at the horizon band; the
    frame's notice for newly long rays, for the moving and nodding marks. Her sum, not a promise: the sea, the run and
    the plain toward the step-limit flight's (the sea's whole about 0.67 ms from 1.09); high over the spawn and the
    mountain top hardly at all, so they'd be the high spot at about 1.05 to 1.10. The Oracle's step 4 with one
    difference for her eye: only the rays that were long are cut, by a list, where hers cut every ray.
    - **The 4 × 4 counted alongside** (David, to Counsel, asked whether Builder adds each layer's rays and steps to
      the reports now, beside the cut, or after it): "yes". So the next high spot (the working marks, whose largest
      known part is the 4 × 4 layer) is aimed by measurement while the cut is built. An instrument, no look.
    - *Build 181* (6657e17, unflown): the cut's first stage (the list of long rays, nothing reading it yet; the picture
      build 178's at five marks, each long ray listed once) and each layer's rays and steps in every report. **The
      counts place the working marks' time:** the mountain top's 4 × 4 marches 69,700 rays and 4,556,000 steps, over
      two thirds of the far eye's steps there, at 65 steps a ray where a base ray takes 32 (its 8 × 8: 2,039,000). The
      plain's 4 × 4 marches none (the 8 × 8: 64,000 rays, 1,178,000 steps, 890 long); the sea's 10,000 rays and
      490,000 steps. Why a 4 × 4 ray takes twice a base ray's steps, though it starts just short of where the base ray
      touched, isn't looked at yet: the next aim for the marks that work, once the cut is through. One flight asked to
      show the bookkeeping costs nothing, then the second stage.
    - *Flown on "go"* (ended 09:38, `logs/flight-1794757116`, checked by Counsel). **A miss at every mark:** the list's
      bookkeeping costs 4 to 11% by Builder's reading (by the card's whole, 5 to 10% over build 180's own flight; high
      over the spawn the most), where the mark said within 1%. Found in the translator's source, not yet shown by a
      flight: to hold the list, the buffer of the tiles' starts became words many rays write at once, and naga writes
      every ray's read of its tile's start as a locked read-and-write, where it wrote a plain read; the mark counted a
      compare and a few hundred writes, and overlooked the read. The mend, build 182: the tiles' starts move into an
      empty corner of a picture the march already reads, the buffer holding only the lists; the first stage isn't
      through till that flies within 1%. What the flight did give: the 2 × 2 marches nothing anywhere; at the three
      dearest marks that aren't the sea the 4 × 4 marches most of the steps (the mountain top 4.56 million of 6.6, high
      over the spawn 3.76 of 6.0 with 188,900 rays, fast across 3.18 of 4.8); every long ray listed (2,419 at most of
      the list's 8,192).
    - *Build 182, the mend, flown on "go"* (85180a9; ended 10:01, `logs/flight-1794758479`, checked by Counsel): the
      tiles' starts moved into a corner of the distances' picture; every mark within 1% of build 180's (fast across
      4.7% under, that mark's own spread). **The cut's first stage is through**, and the reading of build 181's cost is
      shown. Build 183's pass mark is written before its code: the second stage, a listed ray's own thread marching its
      near length and sixteen more threads in a strip of the dispatch's grid marching the lengths beyond, a small pass
      joining them; off the screen the same in kind (8 of 255 or less on listed rays' pixels, no ring or seam at a cut)
      and the longest chain under 140 on the plain and the sea; flown cut against uncut, the plain's march 0.777 → 0.55
      or under, the sea's 0.986 → 0.60 or under, the working marks within 2%. The largest build of the stretch; nothing
      from David till its flight and his look at the horizon band.
    - *Build 183, the lengths and the join* (47cfcca, flown at 1c20872 on "go with probes"; 12:02 to 12:28,
      `logs/flight-1794766116` to `-1794767298`, checked by Counsel). **A miss by its own mark.** Off the screen first:
      the longest chain 324 → 127 steps on the plain and 409 → 127 over the sea (met); the picture within 8 of 255 but
      for three rays (the mountain top 70 and high over the spawn 17, both the map's-edge flaw from before the cut; low
      over the sea 10, a 4 × 4 ray at the skyline whose base ray's first surface sits a little off). Flown, the march
      cut against uncut in one session: the plain 0.769 → 0.396 ms (met), the sea 0.969 → 0.609 (the mark 0.60, a hair
      over), the plain nodding 43% less, the sea nodding 21% less; where the card works, dearer where the mark allowed
      2%: the mountain top +17.7%, high over the spawn +31.5%, the run +16.7%, fast across +30%, the spawn +3%. By the
      card's whole **the dearest mark rose from 1.083 ms (the sea) to 1.370 (high over the spawn)**, against "bring
      down the high spot". Uncut the build costs nothing (within 1.7% of build 182's). The steps barely change (high
      over the spawn 2,263,000 → 2,274,000) and the slowest chain halves there (257 → 127), so the cost isn't steps.
      Builder's reading, not yet seen in a trace: threads kept waiting (sixteen a cut ray whatever its length, and
      uneven lengths sharing a group of 32, which runs as long as its longest). The two probes (the list's line at 64
      steps; that with the first cut at 384) were dearer than the cut as it is at nearly every mark. The cut stays on
      unless `INCUS_FAR_CUT=0`; every mark is still under 2 ms. Builder proposes the mend, build 184: as many lengths as
      a ray needs, about a hundred steps each, cut where the frame before says, and only those lengths' threads,
      packed. David's look at the horizon band on build 183, standing and nodding, asked first: whether the one-frame
      lag shows bears on the mend.
      - *Counsel's note:* the mark counted the longest chain and the steps, and both held; the time went somewhere
        neither counts. If the reading holds, what a full card pays for is threads held, busy or idle, so build 184's
        reports would do well to count them (the lesson Builder draws for the Oracle: the measure to bring down is
        thread-steps in the strip, not the longest chain alone).
  - **[DECIDED 2026-11-15] The cut tried smarter, or dropped** (David, to Builder, asked "Shall I switch it off by
    default now and go on to build 184?", after she said she'd drop the cut if the mend's flight doesn't bring the
    slowest view to uncut or under): "yes, lets try your idea to make the cut work smarter, if it doesn't work it
    doesn't work. its pretty fast in general now."
    - Before it he asked "did the average of frames go up or down?": about level (the far eye on the card 3.7% down
      over the nine views, the whole frame 1.3% up, within the spread; checked by Counsel from the four reports). And
      "are you saying we should keep the cut rays, or that we should keep them? it looks like it increases the range
      of speeds between the mark points, or are you proposing a way to make them work better?": he saw the wider
      spread himself (0.47 to 1.08 ms uncut, 0.49 to 1.37 cut).
    - So the cut goes off by default. **The numbering, corrected:** build 184 is the cut off by default plus the
      count of threads kept waiting (busy and idle, beside the steps), flown on build 183's way of cutting to test
      Builder's reading before anything is rebuilt. The mend itself (as many lengths as a ray needs, packed) is build
      185, with its own pass mark after 184's count. Where build 183's entry above says "build 184" for the mend, read
      185. The bar for keeping the cut: the slowest view at uncut or under.
    - *Build 184* (2954de5, flown on "go ahead with the two flights"; 13:29 to 13:42, `logs/flight-1794771340` uncut
      and `-1794771730` cut at 1,024, checked by Counsel). **Builder's reading stands by its mark's lines:** held
      thread-steps (a group held as long as its longest thread), cut over uncut, the mountain top +16.8% (its march
      +15.9%; the line 9%), high over the spawn +26.8% (its march +39.3%; the line 16%); the strip's threads wait 79
      to 88% of their time (Builder's "81 to 88% at every mark": the plain nodding is 79). Uncut, every mark within
      0.5% of build 183's uncut flight: the count and the switch cost nothing. **A miss on one line:** cut, the march
      isn't within 2% of build 183's cut flight (the plain 0.432 for 0.396 ms, the sea 0.710 for 0.609, high over the
      spawn 1.301 for 1.235; by the card's whole high over the spawn 1.436). Builder's cause: build 184's join counts
      the steps of lengths behind what covers a ray, so the line that keeps a ray listed keeps more of them (rays cut:
      the sea 2,572 for 1,591, high over the spawn 3,669 for 2,682). It shows a waste not named before: a cut ray's
      lengths behind its cover march on, where the whole ray stops. Before the flights, off the screen, Builder
      corrected her own case for the mend (told David): where the card works its ceiling is about level with uncut,
      not under (taking the long rays out of the layers frees only 3.5 to 4.1% of what's held, a long ray's
      neighbours being long too; a perfectly packed strip would spend about 3 to 4%); where it waits the gain stays.
      So build 185's honest case: the working marks level, the sea and the plain much faster, the dearest mark 1.08 →
      about 1.06 ms. Next, build 185's pass mark before its code.
    - *Build 185, the cut packed* (6cbabaf, flown on "go"; 14:38 to 14:51, `logs/flight-1794775526` as launched, the
      cut off, and `-1794775912` the cut on at 384 blocks, checked by Counsel). A ray given only the lengths it
      needs (one or two beyond its first, none over five), packed; nothing marched behind a ray's cover. Off the
      screen every line met (the strip waiting 24 to 42%, from 80; held thread-steps within 1% of uncut where the card
      works; the longest chain 127 at all five marks; the picture within 8 of 255, no ray over). Flown, cut against
      uncut in one session, the march: where the card works **level** (the mountain top −1.7%, high over the spawn
      +1.1%, the spawn +2.8%; the line 3%: met); where it waits **much faster** (the plain −41%, the sea −46%, the run
      −30%, the nodding marks −34% and −39%; fast across −3.6%). **The second keep-or-drop line missed, narrowly:** by
      the card's whole the dearest mark cut is high over the spawn at 1.117 ms, against the line's 1.08 and this
      session's uncut dearest, 1.096 (the same mark).
      - **Unexpected:** the build's uncut flight is about 4.5% dearer at the two working marks than build 184's uncut
        (the march: the mountain top 0.958 for 0.916, high over the spawn 0.975 for 0.934; fast across, which also
        works, 2.7%), the waiting marks within 0.5%. Across sessions, which can differ by up to 5%, but only the
        working marks moved: Builder reads it as the march's program needing more registers for the cut's new code,
        on or off (the team kit's "code added costs every pixel even switched off"). Not yet seen in a trace (Nsight
        would show it). Builder recommends one more round: the cut's new code out of the march's program, re-flown,
        since that 4.5% sits on the game as launched too. Counsel agrees: 4.5% off the working marks' 0.99 ms march
        would bring the cut's dearest to about 1.07, under the line.
  - **[DECIDED 2026-11-15] The cut kept; one round of trimming, then on** (David, to Builder, after build 185's
    flight): "it looks like the new cut does bring the overall average down, and doesn't add much to the high over
    spawn. half as many figures over 1. why don't you look over everything in the shader that can be trimmed and see
    how that works. and then we can move on, if it gets a little better overall than it is now. but if the thing
    ever gets too big this would be something to look at. not sure exactly where we are with that, but i'd like to
    get a move on."
    - His reading checked by Counsel from the two reports (the card's whole): the average over the nine views 0.915
      ms uncut, 0.743 cut (19% down); views over 1 ms, four uncut, two cut; high over the spawn 1.096 → 1.117 (+1.9%).
    - How Builder reads it (hers, not his): the cut is kept; one round trimming the march's program (build 186),
      flown in one session; if it's a little better overall than build 185, on to what's next in the plan. His bar
      replaces the keep-or-drop rule's second line. "If the thing ever gets too big" she takes as the march's program
      size, to look at later if it grows (last measured at build 178: 167 registers, 12 groups of rays fit; not
      measured since the cut went in).
    - *Counsel's note:* this time he weighs the average and the count over 1 ms, the high spot allowed a little
      dearer, where his rule of 2026-11-14 was "the goal should always be to bring down the high spot". His newer
      word decides; written so the change is seen.
    - *Build 186, the march's program trimmed* (03f0376, its pass mark first in 0db6d8d; flown on "it would only be
      staying in because lazy-rays are a different form of it, as i understand it? go ahead with the flights";
      15:26 to 15:39, `logs/flight-1794778376` as launched and `-1794778772` the cut on at 384, checked by Counsel).
      Builder's answer to his question: yes; had it not beaten build 185 it would have stayed in, off, only because
      lazy rays would reuse its parts (her view, not the plan's), and come out if lazy rays doesn't use it. The trim:
      a ray listed whole gets its first cut from its join the frame after; where a length begins and what it lets
      through go to its stack before it marches. On the way she mended a flaw that was in build 185's join too: a cut
      put back inside a length guessed how the ray stood there, and where the cover was all in one place two small
      rays went cut and whole every other frame (33 of 255 lighter); no guess now, frames 12 and 13 the same.
      - **His bar met:** the far eye's whole with the cut on, summed over the nine views, 6.582 ms against build 185's
        6.688 (uncut 8.059); views over 1 ms, 4 uncut and 2 cut. The dearest with the cut on high over the spawn,
        1.086 ms (185: 1.117; this session's uncut dearest the sea, 1.071). The worst 99th 1.290 uncut, 1.245 cut.
      - **The 4.5% explained:** uncut, the mountain top's march 0.917 ms (184: 0.916; 185: 0.958) and high over the
        spawn's 0.933 (0.934; 0.975); the trim took it out, so the cut's weight in the march's program was the cause.
      - Cut on against off, by the card's whole: the plain −35%, the sea −39%, the run −25%, the nodding marks −30%
        and −33%; the mountain top +0.7%, high over the spawn +3.2%, the spawn +3.7%, fast across +3.0%.
      - By build 186's mark the cut goes on as the game is launched (`INCUS_FAR_CUT=0` turns it off), as build 187,
        and on to what's next in the plan. Still owed: his look at the horizon while turning his head, when he next
        plays.
    - *Build 187, played* (1196569, the cut on as launched; about two minutes, launched on "go ahead and launch and
      i'll look while you think about architecutre as we move into the near field", closed cleanly): "Builder, all
      of us are doing something wonderful together. it is alarmingly fast and looks really nice. i think we're going
      to have room for people with stronger systems to bump up the resolution of the march and stuff. lets keep
      going. whats next?" He named no flicker or speckle at the horizon; Builder takes the look owed for the cut as
      given, and has told him so he can correct her. His thought on stronger systems is placed in the spec as an idea
      (The targets). Next by the plan: the scatter stage.
- **[DECIDED 2026-11-15] The scatter stage: Builder's to place, his list to check** (David, to Builder, asked where
  the wood should go and what its trees are made of): "go ahead and take the oak.png if you want, it won't matter
  much; you did a good job picking blocks for pretend last time. i think the things we'd want to be checking are large
  arched entranches to mountain halls, castles with walls and towers, towers in the mountain and on the ground, towns
  on the ground, forests here and there, random cave mouths here and there. its gonna be blurry and just for testing
  anyways, so go nuts, do whatever you want, have fun."
  - How Builder reads it (hers): for the test detail he has lifted "each placed with David" (the plan's line): she
    chooses the blocks, the colours and the places. It stays an instrument behind the doorway, off by default, not
    world generation. What to check, his list: arched entrances to mountain halls; castles with walls and towers;
    towers in the mountains and on the ground; towns on the ground; forests here and there; cave mouths here and
    there. Next from her: reading the doorway, then the stage's pass mark before its code.
  - *Build 188, the scatter stage* (pass mark d69a031; launched on "launch", started at the castle, 72 s, closed
    cleanly). Found first: the far cells had been summed from a column's ground, water and profile alone, so nothing
    far could stand over air and changes through the doorway never reached the far cells; now any column is summed
    from its solid runs (plain ground keeps its quick road, the same sums by test). The stage, on with
    `INCUS_SCATTER_STAGE=1`: woods, posts, an arch, a slab in the air, towers, a castle with an arched gate, towns, a
    hall entrance in the mountains, about sixty cave mouths. Its costs met its mark (the world summed in 42.0 s for
    36.6 off; the far cells 165 MB for 163). First pictures: thin things nearly vanish against the sky at the far
    cells' size, and a cave mouth mostly doesn't read from the front (lit inside as outside, the far light a height
    a column).
    - **His look:** "very nice builder. it looks like it should, i think. but can we do anything about these square
      light seams?" (a picture from high over the plain's town, tower and castle: a broad grey-olive region beyond the
      tower with long straight edges and faint lines nested inside).
    - Builder's finding: not light and not the stage's (the same with the stage off, and with shadows off). The
      export's columns there are sand with the ground at 126, two blocks under the sea's 128, beside the same sand at
      128: two blocks of sea over a nearly flat shelf, dimmed as the far eye dims what's seen through water, and with
      heights in whole blocks on a near plane, the shore and each block of depth run as long straight contours.
      Softening it is a look of water's (a shallow edge that fades in), which belongs with water's own design step;
      his call whether to try it now.
    - Asked by Counsel whether to leave it for the water step (his rule: look flaws written under their step), he
      answered "yeah buddy. we're killing it."; taken as yes, said so to him so he can correct it, and written under
      2.4 in the plan.
    - **Corrected, in the open:** Builder answered about the wrong thing, and Counsel relayed and recorded it. His
      words to her since: "definitely not the shores. its two square rings that move with me, it looks fine on either
      direction of the seams but there's bright square seams; if you look in that sea bed for two bands with a right
      angle turn that are more saturated yellow, you'll see it; it moves with you as you fly." What he asked about is
      the faint brighter lines inside the shallow sea: the edges of the 8s' and the 16s' rings round the eye, seen by
      Builder off the screen since. Her reading of the march: through a hand-over between levels the coarser level's
      solid share is counted against the finer level's cell, so the ground stops a ray too soon, less of the water
      over the sea bed is crossed, and the bed shows brighter (over dry ground it's there and can't be seen; the same
      at the hand-overs by distance). A flaw of the march, mended now as build 189 (pass mark 894306c, her wrong first
      answer written there). Only the shallow sea's own square contours stay for water's look in 2.4.
  - *Build 189, the rings' edges mended* (050f902; launched on "launch" by the plain's town with the stage on, 88 s,
    closed cleanly): through a hand-over, a look read from the coarser level has its solid share turned into the
    finer cell's before the two are mixed. Off the screen the square outline over the shallow shelf gone; the look
    changed a little wherever a level hands over (the five marks against build 182's: 0.4 to 3.7% of pixels by 4 of
    255 or more; a little more sky where a hand-over crosses a skyline). **His look:** "after everything loaded in, it
    is basically invisible, and i don't think anyone will ever have the chance to pick it out amongst a finished world
    even if he got high up on a mountain. well done. let us proceed." The mend kept, and the scatter stage with it;
    nothing said against the look on land. Owed for builds 188 and 189: one flight session (as launched, for the
    mark's 2% line; and with the stage on, for what detail costs). Then by the plan the skyline talk, which the
    stage's pictures now feed.
- **[OPEN, his lean 2026-11-15] The skyline: cover, with weights by kind** (David, to Counsel, after haze and cover
  were explained again with a picture of one ridge each way): "i like the way cover works. remember how we fixed the
  sky glowing too much into the mountain by weighting those rays? perhaps the same approach can be applied to
  balance/tune the strength of different types of hits in the distance and pull them up in the picture a bit."
  - A lean, not the choice: the plan has the skyline talk with Builder first, then his look. His idea goes to it: the
    sky's feather (2026-11-12, a power on each skyline pixel's share) turned the other way, a weight for each kind of
    far thing, so thin things (posts, towers, trunks, an arch's piers) hold their place at a distance.
  - *Counsel's note:* the trick has a known cousin, used for far foliage (alpha kept to the same coverage at every
    coarser level, Castaño 2010). It could be done where the far cells are summed, a weight a level and a kind,
    the coarser the stronger, which costs the frame nothing; the march, full at the marks that work, would carry
    nothing new. Pulling far things up is a chosen exaggeration, owned as style ("every element has to own its
    compromise artistically"), on a slider of his.
- *Builds 188 and 189 flown* (on "you can go"; 17:11 to 17:27, `logs/flight-1794784679` as launched and
  `-1794785233` with the scatter stage on, checked by Counsel; both "the code committed"). As launched, against build
  186's cut flight: seven of nine marks within 2% (the mark's line), the working marks to the hair (the mountain top's
  march 0.913 for 0.914 ms, high over the spawn 0.963 for 0.958). **Two miss:** the plain 0.502 for 0.450 (+11.6%),
  its longest chain 205 where it was 127 (a ray marching a length of 205 steps that the join isn't cutting; Builder
  finding it), and the sea nodding +3.9%. With the stage on, for information: the far eye's whole over the nine
  views 6.995 ms for 6.663; high over the spawn **1.210** (+11%, woods under it: 6.87 million steps for 6.04), the
  run +32%; the stage's own six marks 0.27 to 1.04 ms. So detail raises the dearest view, a world with woods
  standing about a tenth over a bare one, still well under 2 ms.
  - *The plain's miss explained, and build 190* (mark 63a735f, code d1fe13c): one ray nearly covered where its first
    length ended (1.3% let through) was barred from a new length by build 183's guard (none begun under a
    twentieth), so its next length marched 205 steps past the ray's true end, cut and uncut every other frame; it
    showed after build 189 because the hand-over's mend lets a little more through. Build 190: a length begins as the
    ray truly stood (down to a hundredth), and the moving view's guard moves into the join (nothing in the march's
    loop changed). Off the screen: that ray steady (56 and 77 steps), the plain's longest thread 127; the cut's
    still picture nearer the uncut one everywhere (the plain's 3,072 pixels off by 4 of 255 or more, now none); the
    first frame after a 2° turn high over the spawn 9 of 255 out at worst, for 51 (the ground's own colour where
    sky showed through for a frame). Also written down, not mended: with the stage on, the run's march is 39%
    dearer because more rays turn long than the list holds (4,686 for 4,096 places).
  - *Build 190 flown* (on "go"; 18:31 to 18:38, `logs/flight-1794789487`, as launched, checked by Counsel). **The
    mark met:** the bare plain 0.448 ms (186: 0.450; 189: 0.502), the sea nodding 0.518 (0.561; 0.583), the plain
    nodding 0.451; the card's whole over the nine views 6.548 ms, the least yet (186: 6.582; 189: 6.663); no mark
    over build 189's by more than 2%. The two working marks a little dearer (the mountain top's march 0.923 for 0.913,
    high over the spawn 0.980 for 0.963; the dearest view 1.109 ms for 1.091) with the same steps to the thousand;
    Builder reads it as the session (the card's least clock 2,625 MHz at the mountain top for 2,700, after an hour of
    picture tests), not proven by one flight; the next flight flown for any reason will say. Next: the skyline talk.
- **[DECIDED 2026-11-15] The skyline talk: cover by faces first, the shader's weight watched** (David, to Builder,
  on her three steps): "first send a list of any class of items on screen that could conceivably recieve weights,
  and begin. i do like the crisper line of the 'cover' as it was shown to me, against the skyline, and other things
  against other things, hopefully; but pay careful attention to the weight of the shader. i hope it doesn't become
  too much and start squeezing no matter what we do."
  - What "begin" answers (Builder's step 1, cover by faces): thin things vanish because a far cell stops a ray by how
    full it is (a wall a block thick, face-on in an 8-block cell, stops an eighth), and every far cell already
    carries how much face it shows each way (used for light only). Step 1 lets a ray be stopped by the face a cell
    shows it as well as by its bulk: walls solid, towers holding, a post a true eighth-strength band; gentle ground
    at a skyline shows a skimming ray almost no face, so his crisp skyline and sky weight of 4 stay. Nothing new
    stored. Step 2, his weights on top, if he still wants things pulled up after step 1 (where the slider goes and
    its name asked then). Step 3, the land's own edge by a height table (the Oracle's cover), the bigger build,
    weighed after lazy rays.
  - The list he asked for, sent first: 26 classes in five groups. By what it's made of (ground blocks, water,
    trunks, leaves, built stone, built wood and plaster, roofs, blocks to come) and by shape (thin standing things,
    thin sheets, thin flat things in the air, masses, porous masses, holes): both free per frame, known when the
    cells are summed. By where it sits in the picture (against the sky, against other things, by distance, by size
    on screen, the horizon band): each priced by a flight. By light (sunlit or turned away, in cast shadow, things
    that give light at night, glint). Not in the far cells yet (creatures and people, smoke and weather, the sky's
    bodies).
  - For the shader's weight: nothing new carried through the march's loop, and a switch that gives build 190's exact
    program, so on and off fly in one session. His concern is kept in the plan's "What every step keeps".
  - He added: "by weight of the shader i mean its size on the card". Builder: that's the registers the march needs
    (167 at build 178, 12 groups fitting, about 170 the line; not read since the cut), shown only in Nsight's window;
    two traces (the rule off and on), a row read from each.
  - *Build 191, cover by faces* (mark bdc4451, code 891b2ad; unflown). Off the screen: off (`INCUS_FAR_FACES=0`)
    build 190 to the pixel. On: a castle's walls solid from above where they were faint lines, roofs their own
    colour, the arch and a far tower's spire read against the sky; woods solid dark masses (true to a wood, the
    largest change on the screen, hiding more behind them: his eye); bare land's steep edges a pixel or so fuller,
    level skylines and shallow water as they were; the steps level. One line of the mark missed as written: the
    plain's horizon with the whole world loaded has far mountains whose tips firm up (7,168 pixels by 8 of 255 or
    more), the level skyline itself unmoved. Owed: one hands-off window of about 30 minutes (two Nsight traces at
    the mountain top; three flights, off, on, off again, so the session's drift is known); then his two rows, and
    his look on the stage.
  - *Build 191's window* (on "go", 2026-11-16, 06:07 to 06:31; `logs/nsight/b191/`, flights `logs/flight-1794831480`
    off, `-1794831876` on, `-1794832272` off again; before take-off the card about 20% busy with the desktop's own
    compositor, 4% the day before). The flights, by Counsel's reading: the two off flights agree with each other and
    with build 190 (the card's whole over the nine views 6.576 and 6.559 ms, build 190's 6.548), so the session
    compares; on, 6.671, **about 2% dearer everywhere** (against the two offs' mean: the mountain top +1.6%, high over
    the spawn +1.9%, the plain +1.8%, the sea +1.7%, the spawn and the run +2.5 to 2.8%; the dearest view high over
    the spawn 1.125 ms). **His two Nsight rows** at the mountain top: off, 167 registers, 163 live, nothing put by,
    12 groups, 3.94K in the instruction mix (as build 178's: the cut and all since added nothing to the program's
    size; the join 72 registers, the start 46); on, 168 registers, 164 live, **512 put by**, 12 groups, 4.06K. So
    build 191's form pressed on the program's size, from three under the line of about 170 to on it: its mark's "nothing
    more put by" missed, and Builder's word to him that the 2% was arithmetic alone was partly wrong (said to him).
  - *Build 192* (mark 0596ce6, code 5ca541f): the same rule where a step is gathered, reusing the number worked out
    for light (five places gone). Off the screen: off build 190 to the pixel; on, build 191's look (a few thousand
    pixels by 8 of 255 at most); the shallow shelf's sea the same on and off; the steps within 1% of off's. Owed: one
    window (two traces, off and on; three flights, off, on, off), his row, and his look on the stage.
  - *Build 192's window* (on "go", 07:03 to 07:27, the card idle at 4%; `logs/nsight/b192/`, flights
    `logs/flight-1794834872` off, `-1794835265` on, `-1794835657` off again; checked by Counsel). The march, on
    against the two offs' mean: **under 1% at five marks, both working marks among them** (the mountain top the same,
    0.932 ms; high over the spawn +0.7%; the spawn +0.8%, the plain +0.6%, the plain nodding +0.8%); the sea and the
    sea nodding +1.6%, the run +2.8% (its mark's line, under build 191's, missed there), fast across +2.6%. The card's
    whole over the nine views 6.610 ms on, 6.522 and 6.551 off: **+1.1%** (build 191: about 2%). Builder's reading:
    the work counted is the same or less with the rule on at every mark, so what's left is per step, where rays
    gather at nearly every step (through water; skimming grass at walking height), which work a second power out at
    each such step; one power instead of two is the next form, if David wants the rule after his look. Owed: his row
    (the march in the on trace) and his look on the stage.
  - **His row, build 192 on** (the mountain top): 12 groups, 168 registers (167 off), 165 live, **nothing put by**
    (191's form: 512), 4.06K in the instruction mix. Its mark for size met, at one register: two of room left under
    the line of about 170 where there were three (affordable, not free, said to him).
  - **[DECIDED 2026-11-16] An A/B switch: "Cover by faces" on the Render page** (David, to Builder, asked before his
    look "do i have an a/b switch?"; there was none, the rule chosen at launch). Builder proposed a tick box named
    "Cover by faces" on the Render page, ticked by default, swapping between two compiled copies of the march, each
    the program whose row he read. His word: "yes, that place is fine for the switch." Build 193 (mark 5dd6f0c):
    ticked and unticked match build 192's on and off to the test's noise; flipped mid-run, the next frame is the other
    picture but for what a ray keeps from the frame before. The cost: the game starts about 6 s later (two copies of
    the march compiled), until the rule is decided and one copy goes. No flight: the programs are build 192's. Owed:
    his look, on "launch".
  - **His look, build 193** (on "launch", the stage on at the castle, the tick box in his hand; 103 s, closed by him,
    no error): "looks good". Asked whether the box worked once loaded and how walls and woods looked ticked against
    unticked, he said no more. Builder's reading (hers): the look is wanted, woods as solid masses included; cover by
    faces kept and on as launched; step 1 of the skyline talk done enough to go on. Not confirmed by him: the box's
    greying while loading. His weights by kind (step 2) not asked for. Next: build 194, one power where a step
    gathers (for the 1.6 to 2.8% build 192's form costs at the sea marks and the run), a trace and three flights in a
    window of his; then lazy rays by the plan. The tick box and its two copies of the march (about 6 s at the
    start) stay until he says otherwise.
  - *Build 194, one power where a step gathers* (182104f; on "we can do the trim and checks now"; 08:19 to 08:42,
    `logs/nsight/b194/`, flights `logs/flight-1794839365` off, `-1794839756` on, `-1794840152` off again; checked by
    Counsel). The march, on against the two offs' mean: the spawn −0.9%, the mountain top the same, the plain +0.6%,
    **the sea +0.7% (192: 1.6%), the sea nodding +0.5% (1.6%)**, the plain nodding +0.6%, high over the spawn +1.0%
    (the card's clock dipped there in the on flight, its median leaning high), fast across +1.7%. The run unreadable
    this session: its two offs 5% apart (0.586 and 0.617 ms), the processor's area at 103 °C by then. The card's
    whole over the nine views 6.616 ms on, 6.564 and 6.614 off: +0.4% over their mean, the offs themselves 0.8%
    apart. So cover by faces now costs about half a percent, at the edge of what a bracket can tell; the trim did its
    work at the sea marks. Against the mark's 1% line high over the spawn sits on it and fast across is over (said).
    Owed: his row (the march in the on trace). He asked "what do we move on to next?"; Builder, by the plan: lazy
    rays (a talk first, his choice of far squares fixed to the world or the screen, with a measurement of how much
    of the two working marks' work is far enough to be lazy), the land's edge by a height table weighed, the Intel
    card's flight, then the far eye's gate and the near eye.
  - **His row, build 194 on:** 12 groups, 168 registers, 165 live, nothing put by: build 192's to the number (the
    trim changed nothing in size). Cover by faces closed in the journal: "looks good"; one register; about half a
    percent of the march.
- **[OPEN 2026-11-16] Lazy rays: the talk opened** (Builder to David, on his "what do we move on to next?"). Her
  measurement, off the screen, no code changed: thread-steps held beyond a distance, at most (ceilings), at the
  mountain top and high over the spawn: beyond 1 km 71% and 38%; 2 km 43% and 28%; 4 km 26% and 15%; 8 km 14% and
  7%. Her proposal, after the Oracle's section 3: rays marched every frame up to a lazy line; beyond it a picture all
  round the eye, fixed to the world, refreshed a share a frame and re-made as the eye travels, laid behind the near
  picture each frame with light and fog fresh; the march's program not grown (near rays end at the line; a separate
  compiled copy makes the far picture). One step a build: the line at about 4 km first, then 2 km and 1 km, each with
  his three looks (turning, a ridge sliding across far land, the sun's step) and a flight on the moving marks. Asked
  of him: far squares fixed to the world or the screen (she recommends the world), refreshed all at once or piece by
  piece (she recommends all at once, by his "even beats irregular").
  - *Counsel:* agrees on both. Fixed to the world, turning costs nothing to keep right, only travel and time age the
    picture; and it's his own idea in the spec, "a world-anchored dome for far pixels". All at once keeps every far
    square the same age, so a sun step or a stretch of travel never shows as a seam between fresh and stale pieces.
    It's the first lever aimed at the working marks themselves, the dearest views.
  - **[DECIDED 2026-11-16] Lazy rays shelved, kept in the back pocket** (David, to Builder): "lets hold off on lazy
    rays for now. we can keep that in our back pocket if we ever get a performance squeeze limited by long rays. i
    think we can move on. just pack the instructions and your output there into a lazy rays doc to resume with if we
    ever need it." Shelved, not dropped; his two questions (the world or the screen; all at once or piece by piece)
    stay open. Packed in `journal/docs/lazy-rays.md` (the Oracle's rules and tests, the three measurements and how
    to take the last again, Builder's proposal and what it stands on in the code, his open questions, what isn't
    known, how to resume). Next asked of him: whether the land's own edge by a height table goes in the same pocket
    or is built now, and when the Intel card's flight is flown (Optimus and a restart, which ends the sessions: notes
    written first); then the far eye's gate and the near eye.
  - **[DECIDED 2026-11-16] Cover by height shelved beside lazy rays; the march mapped by its parts** (David, to
    Builder). Asked first: "would the lands own edge by a height table work with mountains in front of mountains as
    well as in front of sky"; Builder: yes by its design (the edge worked out at the near ridge, whatever is behind),
    though a far ridge's sideways steps stay as wide as its columns, anything not plain ground keeps the march, and
    none of it is built. Then his word: "please pack it up next to lazy rays, along with scanning the march and
    noting in a file anything that can later be removed from the march to free up room on it; i would like for a
    sort of 'windirtree' view of the march by its components, and the size of those components inside the march, if
    possible, so that later review could identify ways to further improve it by removing sections that do little and
    replace them with something else. if i become dissatissfied with some views in a fuller game i'll come back to
    cover by height and lazy rays and there may be a final check pass against the engine later anyways."
    - Shelved, not dropped: the land's own edge by a height table and the sheet that hung on it
      (`journal/docs/cover-by-height.md`). "A final check pass against the engine later" is when these documents and
      `lazy-rays.md` get read again.
    - The scan (`journal/docs/march-by-parts.md`, the script and build 194's program texts in `kept logs/march/`): the
      march is 4,548 instructions; about a third finding where a read lands in the bricks' textures, mostly edge
      cases; the hand-over's second read a quarter; the gather a twentieth. It carries 98 values round its loop (97
      with cover by faces off, his rows' one register), 45 of them the two levels' numbers and the height table's
      rows kept in hand. Eleven candidates for making room; first, a guard naga adds to every loop that wgpu lets a
      trusted shader leave out. Only test code changed (34 tests pass); nothing to fly.
    - *Counsel's note:* the first candidate is of the kind experiment 3 tried (the library's checks taken off), which
      made the heavy views a third to a half *slower* (team kit: "price each program checked and unchecked; don't
      assume either"); it would want its own flight, taken alone.
    - Builder agreed and corrected the row in the open (9c57e77: it may well make the march slower; experiment 3's
      result given; its own trace and bracket of flights, each of the three checks priced by itself). She also found
      a standing rule that slipped: Phase 2's "each program is priced checked and unchecked the first time it flies"
      hasn't been done for the new march, as far as the journal shows. Written in the plan as owed, to ride with the
      Intel card's flight if he agrees.
- **[DECIDED 2026-11-16] The far eye left as it stands; on to the near eye** (David, to Builder, asked when the Intel
  card's flight is flown): "we can do that later. as long as we're set up to look back into the graphics engine at a
  later date, we can proceed." No restart now. 2.2 isn't closed by a word of his as 2.1 was; "we can proceed" is what
  he said. Its gate's last line stays owed (the Intel card's numbers, with the new march priced checked and
  unchecked). Where the far eye stands as left: the card's whole 0.48 to 1.12 ms over the nine views, everything
  under his 2 ms, two views over 1; the march 168 registers, nothing put by, 12 groups. Everything set aside is
  listed in the plan, "For the later look back at the far eye", each with where it's packed (lazy rays, cover by
  height and the sheet, room in the march, the Intel flight with the checked and unchecked pricing, his weights by
  kind, the long rays' list, the small flaws). Next: 2.3, the near eye, its first step and pass mark brought to him
  before any code.
- **[DECIDED 2026-11-16] The near eye in triangles** (David, to Builder): "I believe we should try to accomplish the
  rest of the graphics engine with triangles. rays may come in later for different cases, but i don't believe
  quarter/oct blocks will present any difficulty in terms of triangle count. it will still have less polygons than
  half life 1. i'd like to hear how you think we could gracefully add the triangles in, and make sure they cooperate
  nicely with the march."
  - Before it, his four questions and Builder's answers (all unbuilt and unmeasured, said so): "how would the near
    rays pack with the far rays in the march? do we have room in the march, or would it run concurrently, ideally as
    a quite short rays?" (a program of its own, the near pass first, the far march only where still needed); "how do
    you think it will compare with triangles?" (her lean: triangles faster and steadier for whole blocks, rays better
    inside a block's pieces); "is there any way that triangles help the far march at all? will screen triangles
    cancel screen rays since they occlude?" (yes: their depth says which far squares are covered, a lot at ground
    level and indoors, little from high up); "so in a cave, the march would be totally silent" (as near as makes no
    difference).
  - **What it changes:** his "Yes, fly both" (2026-11-12: near rays first, then triangles, both flown before the
    fork) gives way: the near eye is built with triangles, the pieces too; near rays not now. The fork's comparison
    of two near drawers falls away (put to the Oracle in her brief). Builder's correction to him, his conclusion
    standing: a block world's near field is far more polygons than Half-Life's scenes, but still no load on the card.
  - Her "how", being answered to him: triangles drawn first, their depth cancelling far rays; the hand-over at the
    near field's edge the place to get right; one light and one set of colours for both drawers at first; a step a
    build. *Counsel:* this matches Counsel's lean when he asked (triangles for everything near, rays far), and
    David's "it seems wasteful to hit a single triangle a hundred times".
- **[DECIDED 2026-11-16] The near eye's goal: a 64-chunk radius, smooth beside the far march** (David, to Builder,
  answering her proposal to start the near field at about 128 blocks): "in old incus artus, i was playing with a 64
  chunk render radius, which i believe would be 2000 blocks. the overlap can be pretty deep. that may be excessive
  render radius but i don't think so for a whole town, so my goal would be 64 chunk render radius running smoothly
  along with the far march like we have it now, which is super quick."
  - Checked by Builder: old's chunks were 32 blocks, so 64 is 2,048, his figure; the record (`journal/docs/far-view.md`,
    builds 27 and 42) has his old preset as render distance 64 with "the march starts at" 8, so old kept real blocks
    to 2,048 and drew them to 256 (told him as the record's, with its date, not as a correction of his goal).
  - Sized by arithmetic only: 13.2 million columns, about 16,600 chunks, 26 million triangles unmerged, a block under
    a pixel past about a kilometre; so it wants merged faces, drawing only what's in view, and coarser cells for
    plain ground far out; the far march begun at the radius' edge loses most of its work. Put to him: every block
    true out to 2,048, or built things true all the way with plain ground stepping down in detail as the far eye
    draws it now.
  - *Counsel's note:* the plan's "What every step keeps" already says "hard textured blocks near; past a pixel,
    cells that are the exact sums of their blocks". Past about a kilometre a block is under a pixel, so blocks
    "true" there would be an exception to that rule (his to make), and triangles smaller than a pixel shimmer as the
    eye moves and cost the card more per pixel than larger ones. The second choice keeps the rule for ground; for
    built things past a pixel, the far eye's cover by faces already holds walls and towers.
  - **[DECIDED 2026-11-16] One radius slider, and begin** (David, to Builder, on that choice): "well lets just make a
    radius slider and get started. the render radius set there will determine where the triangles start and rays
    fade. if that becomes intensive we can add a slider that seperates built things from the land render so they can
    be adjusted seperately until it hits a nice balance with the march. and, for combining faces, i thought that was
    called greedy meshing and kind of an automatic part of any block game triangle mesh geometry". So: one radius
    slider now; a second (built things apart from the land) only if it gets heavy, written down, not built; faces
    merged (greedy meshing, his word for it and the usual one) from the first build. Its place and name asked by his
    rule for controls (the Render page, "Render radius (chunks)", 1 to 64): not answered yet.
  - *Build 195's pass mark* (in `journal/docs/experiments.md`, before its code): the near eye's first step, whole
    blocks as merged faces a chunk of 32 at a time on a lane, drawn after the far picture with depth, lit from the
    far eye's light, fading at the radius; the far march's program untouched; `INCUS_NEAR=0` is build 194;
    instruments from the first build; no line set for its time yet, this being the first measurement.
  - The slider's place and name (the Render page, "Render radius (chunks)", 1 to 64): "yes thats fine".
  - *Build 195, built* (the mesher b48b804, the lane 026c581, the pass 399f59d; unflown, unlaunched). Off the screen:
    blocks inside the radius as merged faces, a chunk of 32 at a time on a lane of its own, drawn after the far picture
    with depth, each face its block's far colour lit by the far eye's own text; the far eye's water over them; faces
    to where every chunk is in, fading over the last stretch, the far rays beginning a little inside it (no hole while
    a radius fills). The far march's program untouched. From the spawn at 64 chunks: 13,100 chunks, 14.4 million
    faces kept (168 MB), 10.3 million triangles drawn in view, filled in 5 s; the whole radius 2.7 million triangles
    on the plain, 27 to 29 million on hilly ground, 7 million with the stage's castle and town. Colour and light
    within 5 of 255 of the far eye's; no gap at radius 1, 4, 16 or 64. 39 tests pass. At 64 chunks far hills' blocks
    are under a pixel and speckle: written down for his eye. Not yet: the near eye's counts in a flight's report,
    changes reaching the lane while the game runs, off against build 194 with the whole world, the stage's views and
    the near marks. Owed: his word to launch, then a trace's row and a bracket of flights.
  - **His first look at the near eye** (build 195 on "launch", at the spawn, the radius starting at 4 chunks; 372 s,
    closed by him, no error or warning; its first run in the game): "holds 240fps at 64 chunks, once its loaded in."
    So today's goal, "64 chunk render radius running smoothly along with the far march", is met at the spawn by his
    own look. 240 is the screen's refresh the game is held to outside a flight: it says "all it's asked for", not
    what's left over (the flights say that). He didn't speak of the filling, the edge, chunks arriving or far hills'
    speckle; Builder asked lightly, not taking silence for "fine". Seen in the log: the processor's area at 102 °C at
    320 s, the near lane on 12 threads. Build 196 for the next launch: the near lane on 2 to 6 threads with a switch,
    and a slow place in the frame's asking found by reading (looked up in a set now). Owed: a trace's row; flights
    (near off, on at 4, 16 and 64); the near eye's counts in a flight's report; the lane told of changes; the mark's
    remaining checks off the screen.
  - **His words after it** (to Builder): "if i fly very fast the FPS go down to 90-120 while its generating chunks. i
    don't think a player could experience a hiccup. off to a good start. go ahead with the tests. there is a seam,
    but, its not very bad, and i think a x-fader with curves could hide it almost completely so blocks smoothly fade
    into rays." Taken as: his go for the near eye's hands-off window (two traces; five flights, near off, 4, 16, 64,
    off); the filling fine by his judgement, written down and left; the seam's x-fader with curves drawn up after the
    tests, its controls' place and names asked before anything is placed.
  - *Found by Builder, reading for build 197* (the flights' fittings): a mark's changes (his build, the scatter
    densities) have never been drawn by either eye in a flight, only put in the game's own doorway. The reports bear
    it out: in every flight "the plain with David's build" and the three scatter marks time as the bare plain
    (build 183's uncut: 0.910, 0.909, 0.910, 0.909, 0.909 ms). Build 197 tells the near lane of them, waits for the
    near eye's chunks at standing marks, and gives the near eye a line in the report.
  - *The near eye's window* (build 197, 64eeb0e, on "go ahead with the tests"; down at 12:05; two traces at the
    mountain top, flights `logs/flight-1794850683` off, `-1794851080` at 4 chunks, `-1794851486` at 16,
    `-1794851956` at 64, `-1794852349` off again; checked by Counsel; the processor's area 103 to 105 °C all window,
    Builder's own tests having run to take-off, said in the journal). **240 a second holds at 64 chunks at every
    standing mark:** the slowest, high over the spawn, 3.39 ms a frame (295 a second), its 99th 4.27; the spawn 3.01
    (99th 4.03). At 64 chunks the near faces are the dearest thing on the card (the card's whole 2.98 ms at the
    spawn and 3.36 high over the spawn, for 0.48 and 1.12 off: 2.4 to 2.6 ms of near faces for 10 million triangles),
    and the processor spends about 1 ms a frame listing the chunks in view. At 4 and 16 chunks everything stays under
    1.5 ms on the card. **One miss, its cause found:** at ground views the far march is 16 to 23% dearer with the near
    eye on (the plain 0.452 → 0.549 ms at 4 chunks, the mountain top 0.948 → 1.118), though about a third cheaper at
    64 from high ground (the mountain top 0.636, high over the spawn 0.599): far rays begin a set distance out
    whatever lies nearer, so under the near faces they begin inside the ground and make a false edge that calls in
    the finer layer. The mend is the step next on 2.3's list, near cover cancelling far rays. With the near eye on,
    his build and the scatter now cost what they hold (the densest scatter 1.24 ms at 4 chunks).
  - **[DECIDED 2026-11-16] The debug panel folded into sections, scrolling** (David to Builder, mid-window: "hey btw,
    we gotta do something about that debug panel, it runs off the screen its so long. we should add collapsing
    sections to it and a scrollbar."; on her list of sections and names: "your way of folding the content of the
    debug pages is fine"). The Render page: "The far eye", "The near eye" (with the radius slider), "The picture",
    "Ray start", "Crossovers", "Light", "Fog", the first three open at the start; the Overview: "The frame", "Each
    pass"; the whole page scrolling, never taller than the screen. Build 198.
  - Next put to him by Builder, his call: the near eye drawn first into a picture and depth of its own, which serves
    the cancelling, his x-fader at the seam, and shading each near pixel once. *Counsel:* it suits "The shape" in
    the plan (small passes over a per-pixel store) and mends the miss at its cause.
  - **[DECIDED 2026-11-16] The next step, and the near trace read** (David, to Builder, with three pictures of the
    near-64 trace's Shader Pipelines tab): "here is the trace. go ahead with the next step and launch". The rows: the
    near faces' cost on the card is their corners' program (49% of samples) against 15% for their pixels, so it goes
    with how many faces are sent, not how many pixels they cover (Builder corrected her own emphasis: "each pixel
    shaded once" is the small part); the march as it was (168 registers, nothing put by, 12 groups). His go taken to
    cover the step as put (near blocks drawn into a picture and depth of their own, joined to the far picture by his
    x-fader; far rays skipped under blocks the build after) and the x-fader's two sliders by the place and names asked
    in the same question ("x-fader width (blocks)" and "x-fader curve", in "The near eye" under the render radius);
    he answered with the go only, so the names are said back to him at the look. Build 199's pass mark in 230f4d5.
    Build 198 (the panel folded) launched for his look.
    - *Counsel's note:* with the corners the cost, the lever at 64 chunks is fewer faces sent far out, which is his
      second slider's road ("a slider that seperates built things from the land render"), plain ground stepping down.
  - *Builds 198 and 199, launched for his look on that go* (Builder and Counsel both read "go ahead with the next step
    and launch" as covering the step built and launched). Build 198, the panel folded: looked at 134 s, closed with
    no word on it yet. Build 199, the x-fader (46b8606): the near eye draws into a picture and depth of its own (the
    nearest face winning a pixel), and a new pass, "the x-fader", lays it over the far picture; "x-fader width
    (blocks)" (64 to start) and "x-fader curve" (2 to start) in "The near eye" with a Default beside the heading. Off
    the screen: the near picture the same from run to run to the pixel (build 197's wasn't, the fading faces' draw
    order); no gap at 1, 4, 16 or 64 chunks; a hard edge at no width, half and half at the band's middle on every
    curve. Looked at 195 s (`logs/session-1794854665.log`, no error or warning), no word yet on it or the names. Its
    cost owed a bracket on his word. Next, under his go: build 200, far rays skipped under blocks (the near eye drawn
    before the far march, a small pass saying which far pixels are wholly covered, the march not marching those);
    it touches the march, so an off and an on trace for him to read and a bracket of flights, on his word.
  - **His look at builds 198 and 199** (with a picture): "the fade looks good, and the panel folding is good. i notice
    speckling white dots of sky when looking down and moving." The x-fader and the folded panel kept. In his picture
    the x-fader stood at 128 blocks wide and curve 1 (they start at 64 and 2); the starting numbers left until he
    says. The dots are holes, mended first: build 200 the dots, build 201 the far rays skipped under blocks. Found off
    the screen, looking down from above: 38 to 117 lone pixels of sky in solid ground, from two causes together:
    merged faces meeting at different corners can leave a pixel neither covers (greedy meshing's known price, the
    "T-junction"), and behind a block the far rays begin a set distance along the ray, which looking down is under
    the world, so the far eye draws sky there. Build 200: each face a hair wider (a few hundredths of a pixel) so
    neighbours overlap.
  - **[DECIDED 2026-11-16] His starting numbers** (David, to Counsel, asked "Make your crossfade settings, 128 wide
    and curve 1, the defaults?"): "yes, and 32 chunk radius." The defaults become x-fader width 128 blocks, x-fader
    curve 1, render radius 32 chunks (1,024 blocks), Default returning to them as with the sky's feather. Passed to
    Builder whole. ("i like that because now your render radius is the same as the world height.")
- **[DECIDED 2026-11-16] The Oracle is any outside advisor, a fresh agent each time; models by the work** (David, to
  Counsel): "i think we can have the 'oracle' be any outside advisor for our team considerations, and get a new agent
  each time for a fresh look. i've had builder in fable 5.2 for the graphics engine, and we're almost done with it for
  now, so i'll probably just switch you to fable and put builder back on opus as the upcoming stuff is mostly just
  rearranging windows and implementing features out of old incus and its not quite so intense or mysterious. if i
  want an outside look, i'll construct a prompt for a mr meeseeks type consultation."
  - So the fork's "Oracle comes back" becomes an outside look by a fresh agent on his prompt; Counsel's brief
    (`journal/vision/oracle-return.md`) serves any such reader, written for one with no memory of earlier reviews.
    The kept Oracle session's fourth part no longer waits; her five questions to him still stand as questions, two
    bearing on the near eye.
  - Counsel to Fable, Builder back to Opus, at his switch. Everything each role needs is in the files.
  - CLAUDE.md's line for the Oracle ("it's called at the big forks") is his to change; asked. His answer: "Yes,
    change it" (changed in 8f9b248).
- **[DECIDED 2026-11-16] The graphics trusted; hanging questions dropped; the road after it** (David, to Counsel,
  having switched Counsel to Fable: "you just click on the tab and switch models and keep going"; Builder stays on
  Fable until graphics are finished: "i'll wait until we're done on graphics"):
  - "we can drop any homework or questions that are hanging on, and kind of reorganize a little going forward, since
    i'm pretty happy and confident with our graphics engine now. i think it'll be able to handle anything we throw at
    it at this point and i've had builder set up documents for when it gets reviewed again in the polishing pass, so it
    should be easy to work on when we need to."
  - "the next stages of programming incus artus, after we finish up on graphics, will be getting the tectonics, world
    gen, registrar, put back together in a more streamlined way, because it gets pretty crowded, so we'll use space a
    little better. i'll go through old incus tectonics and identify the important parameters and try to make it a
    little more flexible."
  - "then, we'll construct the world gen more deliberately, since we know the road now. i would like a modular stack
    that works much like it did before, but we can do a better job with topography and water layers i think this time.
    and, we should consider the prerequisites for making it extensible so that you could invent new topographies for
    it. one thing i want this time is a much more deliberate way water is added, after topography, so that we can get
    flush beaches and cliffs that make more sense."
  - "once the terrain and waterways are dialed in again, ecology will require the placer and registrar. the registrar
    will require a more deliberately gui'd master palette than last time, it got a bit confusing to navigate. i'll
    give more explicit input on shape and boxes in it."
  - "Builder can proceed with the tests and basic fixes, i'll bring it up if i see undesirable graphics behaviors."
  - **What's dropped** (Counsel's reading, said to him): the kept Oracle's five questions and her fourth part; the
    fork as a gate (an outside look is his to call, on his prompt); Counsel's open question on Phase 3's order, which
    this answers; the one budget and temporal reuse "at the fork", which go to the later look back.
  - **Phase 3 reordered on it** (the plan): tectonics; world generation as a modular stack, topography then water;
    the Registrar with its master palette, and the placer; ecology. Earlier the same day: "then i think we can add
    players, items, and animals, ideally through the editor gui entirely." The tablet (the world on disk), the
    Graphics Kit and making a world as the player meets it aren't in his order; kept in the phase, fitted in where the
    work first needs them.
  - **The palette in the Graphics Kit too** (David, to Counsel, on the Registrar's master palette, the same day):
    "yes, i will want the palette to be able to be copied to the graphics kit for the image processor and do most of
    the same things". *Counsel:* so the master palette is built once as a shared piece (its data and its tools) that
    the Registrar and the Graphics Kit both show, not two palettes kept in step by hand (the rebuild's "shared pieces
    once"; the spec already has reshading "as shared code"). Whether a copy in the Kit stays linked to the Registrar's
    or goes its own way once copied is his to say when its page is laid out.
  - **Answered, and the Graphics Kit's shape** (David, the same day): "i think both sides of the palette would be
    able to both create splines and quantize images into less colors and derive splines. they'd have the same
    palette. each sides unregulated colors would be temporary, as it gathers the colors the image has to be processed
    or added or reduced or whatever. that's useful enough to have the graphics kit have the image processor first
    before the animator and the fx engine. bouncing from the world gen is kind of pointless, but it won't be pointless
    bouncing between those three inside the graphics kit; i don't foresee needing creative editing like a pen tool or
    anything, but filters, layers, curves, colors, etc. yes, in a way where i can macro/automate them on a timeline in
    the animator".
    - So: one palette, the same on both sides (linked, not a copy that drifts); both sides make splines, quantize
      images into fewer colours and derive splines; unregulated colours are a working set, temporary. The Graphics
      Kit's three in building order: the image processor, the animator, the fx engine, made to be moved between. No
      pen tool; filters, layers, curves and colours, automatable on a timeline in the animator.
    - *Counsel:* it carries his earlier ideas forward (the spec: "Not a drawing program ... every slider moves by
      itself"; three devices as pages in one window, then named Graphics Kit, Sequencer and FX Lab; today's "animator"
      and "fx engine" are his newer words, and the names are his to settle when the pages are laid out). For
      building: every filter, layer, curve and colour control made as a numbered setting from its first day, so the
      animator's timeline can drive any of them without each being rebuilt for it.
    - **The names, and what each does** (David, the same day, correcting "animator"): "sequencer and fx engine i
      think. image processor, sequencer in which you can bounce images into frames, or across frames, or back to the
      image processor, fx engine, which im not even sure the exact scope of it, but i know it'll need particle fx to
      bounce to the image processor or sequencer, and be able to step through the simulations and probably fx on
      images as well." So the three are the **image processor**, the **sequencer** and the **fx engine** (where
      Counsel wrote "animator" above, read sequencer). The sequencer: images bounced into frames, across frames, or
      back to the image processor. The fx engine: its scope open; known so far, particle fx that bounce to the image
      processor or the sequencer, simulations stepped through, probably fx on images too.
    - **A shared clipboard for bouncing** (asked by Counsel whether bouncing could be one shared operation): "yes, a
      shared clipboard for bouncing". All three send to it and take from it. *Counsel, for building:* it holds an
      image or a run of frames; what else it may hold (a palette, a spline) is his to say.
    - (Written after build 202's window landed; his words were held in a note outside the project while it flew.)
  - Graphics still ahead by the plan: 2.3's quarter and eighth blocks (scattered at random for testing, his idea),
    then 2.4 (textures, the light lane, water and glass, objects, glows, far lights), his eye raising anything
    undesirable.
- *Builds 200 and 201* (unlaunched, unflown; 201 at db29d24, with his starting numbers: 32 chunks, x-fader 128 wide,
  curve 1, Default returning to them). Build 200 (1affa62), the white dots: looking down from three places, 38, 117
  and 107 lone sky pixels, now none (each face drawn a hair wider, so merged faces overlap). Build 201, far rays
  skipped under blocks: the near eye drawn before the march, a new pass, "the near cover", saying which far pixels
  blocks wholly cover, and the march marching none of those; off the screen the far eye holds 7% (4 chunks) to 87%
  (64 chunks, looking down) fewer thread-steps, the far picture unchanged. Owed on his word: an off and an on trace
  for the march's row, and a bracket of flights after a cool-down. Found and put to him with the launch, not built:
  the near eye's depth is too coarse far out (at 2,000 blocks it can't tell faces a block apart), so a few hundred
  single pixels differ run to run there, and moving would show far faces flickering between a top and a side; the
  standard mend is small.
- *Build 202's window* (12b3737: build 201 with the near depth's mend, the depth kept turned round, the same view
  drawn twice differing at 16 pixels for 422; flown on his "Builder can proceed with the tests and basic fixes";
  down at 14:50; a cool-down before each flight, each begun at 66 to 77 °C; two traces at the mountain top at 64
  chunks; seven flights, `logs/flight-1794859356` near off, then at 4, 16 and 64 chunks the skipping rule off and on,
  to `-1794862234`; checked by Counsel). **Mixed.** The session compares (near off, the far eye within 1.5 to 2% of
  builds 194 and 197). Skipping far rays under blocks saves the far march a quarter to two thirds where blocks fill
  the view: the mountain top 1.180 → 0.836 ms at 4 chunks and 1.148 → 0.756 at 16, the spawn 0.411 → 0.149 at 64,
  the densest scatter 0.570 → 0.154. But its own pass, the near cover, costs 0.142 to 0.146 ms a frame (three times
  Builder's estimate), so at most open standing marks the card's whole is dearer with the rule on (0.06 to 0.19 ms:
  the spawn 0.584 → 0.775 at 4 chunks, the plain 0.756 → 0.929). 240 a second still holds at 64 chunks everywhere
  (the slowest high over the spawn, 3.52 ms a frame, 284 a second). **Missed:** "the far eye no dearer with the near
  eye on than off" on the plain (the march 0.457 ms near off; 0.573 to 0.616 on, 25 to 35% by Counsel's reading of
  the march, Builder's 22 to 30%) and the sea: not waste under blocks; with its rays begun beyond the near blocks
  the far eye no longer draws its own near haze over the horizon, and its finer layer is called in along it. A
  crisper horizon at a cost, governed by the Crossovers' sliders, his business if he wants it. Two faults of
  Builder's in build 201's code, found by reading: the march asks the cover before asking whether a pixel is wanted
  (eleven million needless reads a frame in the finer layers), and the cover's pass reads the near picture a pixel at
  a time where it could read four. Build 203 mends both under his "basic fixes", then a short flight and the on
  trace again.
- *Build 203's window* (1c5753e; down at 15:10; the on trace at the mountain top at 64 chunks, and one flight at 16
  chunks, `logs/flight-1794863432`, read against build 202's two at 16; checked by Counsel). The cover's pass halved,
  0.062 to 0.079 ms (it was 0.137 to 0.162; the mark's "under 0.07" met at some standings, 0.07 to 0.08 at the rest).
  On open ground the far eye is no longer dearer with the rule on than off (the mark met; Builder's figures the plain
  0.705 against 0.709, the sea 0.679 against 0.684). The card's whole, the rule on in build 203 against off in build
  202: the mountain top 1.172 for 1.520 ms, the densest scatter 0.729 for 1.273, the plain 0.882 for 0.814, the sea
  0.876 for 0.811: cheaper at the working marks (0.35 ms at the mountain top, up to 0.54 where blocks fill the
  view), 0.05 to 0.08 dearer on open flat ground, which is the cover's pass itself. Left there, done enough to go on,
  the two ways to take the rest off written down for the later look back. Open, his when he likes: the march's row
  in the two traces (`logs/nsight/b203/b203-cover-on/`, `logs/nsight/b202/b202-cover-off/`; the mark no more than
  168 registers). Next: 2.3's quarter and eighth blocks, scattered at random to test them.
- *Build 204, quarter and eighth blocks in the near eye* (e2949a3; one flight as the game starts, 32 chunks, his
  x-fader numbers, the skipping rule on, down at 15:41, `logs/flight-1794865272`, sixteen standings, checked by
  Counsel). The doorway takes an eighth; a chunk with pieces is meshed in half blocks; the marks' scatter can make
  quarters and eighths, with a new last mark, "the scatter of pieces" (1,000, 10,000, 100,000). **His old build is
  drawn whole for the first time in the new game** (its 146 stone-brick bars with its 23 blocks), 0.006 ms over the
  bare plain by Builder's reading. Pieces cost the card about what whole blocks do, count for count: the near faces
  0.085, 0.162 and 0.642 ms at the three densities of pieces, 0.099, 0.238 and 0.710 for whole blocks (Builder's
  figures for the near eye whole: 0.19, 0.28, 0.77 beside 0.21, 0.35, 0.83). The card's whole under his 2 ms at
  every mark (the dearest high over the spawn, 1.80); every standing frame 0.96 to 1.84 ms. Where blocks or pieces
  fill the view the far march is nearly silent (0.09 to 0.10 ms). 41 tests pass.
  - Found and mended in it: a block's number wasn't the same through every doorway (a block the world's list hasn't
    got was numbered by each doorway as first asked there; the bars first came out oak's brown), now one list for the
    process. Stone brick had no far colour: added to `data/far colours.toml` by the file's own rule from old's bar
    textures (59, 61, 55); the file is David's ("i'll go in and regulate everything again once we rebuild the
    registrar"), told him plainly.
  - Written down and left: a chunk in half blocks costs the lane nine times a whole one (0.3 ms to 2.8), only chunks
    with pieces paying it.
  - Nothing waits on David but his looks when he wants them (the build, the pieces) and the march's row in the two
    traces. What comes next in 2.3, or on to 2.4, is his to say.
- **His reading of the two traces** (David to Builder, with six pictures: "here are the off traces, followed by the
  on traces."): with far rays skipped under blocks the march is 167 registers (168 without the rule), nothing put
  by, 12 groups: the mark met. The near cover's pass is 3% of the card at that view, 93% of its stalls waiting on
  its reads.
- *Build 205, textures on the near faces* (mark 73fcbef, code fd17fda; 2.4's first step, taken on his "so textures
  and quarter/octs next, right?" of the same day; one flight at 32 chunks, down at 16:04, `logs/flight-1794866685`,
  read beside build 204's; checked by Counsel). Old's placeholder pictures carried into `data/textures` (the version
  with the least of his taste in it, said to him), crisp up close, shrunk by old's rule to the far colour (the far
  hill's colour the flat one's to a fifth of a level); a switch leaves them off; the PNG reader old used added. Off
  the screen his build shows its stone brick, gabbro and grass in their own pixels. 42 tests pass. On the card the
  pictures cost 0.01 to 0.04 ms on land and about 0.1 where blocks fill the screen (the densest scatter's whole
  1.073 → 1.197 ms, its faces 0.710 → 0.799); everything still under his 2 ms (the dearest high over the spawn,
  1.82); standing frames 1.09 to 1.86 ms.
  - **Not explained yet, said plainly by Builder:** on the processor, recording the near faces is 0.04 to 0.09 ms a
    frame dearer than in build 204's flight (the plain 0.195 → 0.280) and handing the card its work about 0.06
    dearer (the plain 0.504 → 0.562), every other processor pass the same; read from two single flights half an hour
    apart, so it wants a bracket in one session (pictures off, on, off) before anything is changed; flown after his
    look.
  - Waiting on him: his look at the textures ("launch"), then what of 2.4 comes next.
  - **His look** (asked of Builder: "can you launch with your scattered details like castles and forests?"; build
    205 at the spawn with the scatter stage on, 466 s, `logs/session-1794866918.log`, no error or warning; the near
    eye's first run in the game with the stage on and with textures): "its looking really good." In view near the
    spawn: a village (houses 290 to 340 blocks west), a tower about 540 blocks off, the woods; the castle stands on
    the plain about 4,900 blocks off, past his 32 chunks (Builder's slip about its place, said to him; a launch at
    the plain mark offered). The stage's own blocks have no pictures yet and stay flat in their far colours.
  - **Corrected, in the open: the pictures cost nothing that can be shown** (build 206, 581fb86, an instrument only:
    listing the near chunks in view timed as a pass of its own; three flights in one session, each begun cooled,
    pictures off, on, off, `logs/flight-1794868466`, `-1794869099`, `-1794869687`, down at 16:54; checked by
    Counsel). The card's whole summed over all sixteen marks: 16.99 ms off, 16.66 on, 17.16 off again. Listing the
    near chunks over the standing marks, by Builder's sum, 2.78, 2.99 and 2.47 ms: the two offs further apart than on
    is from the first. So build 205's "0.04 to 0.09 ms dearer on the processor" and the card's "0.01 to 0.1 ms"
    above were the difference between two sessions, read from two single flights against the standing habit
    (the team kit: "compare within one session"). Builder's, said by her with what it cost (a build and an hour of
    his hands-off time); **Counsel's too:** the card's figures above were written down as checked, and told to David
    as the textures' cost, when the check was only that the two logs said so. Checking a number against its log
    doesn't check what the number is compared with.
  - What the instrument did find: all of the near eye's processor time is listing the chunks in view (0.15 to 0.27
    ms a frame at 32 chunks; handing the pass over 0.004), and the processor's passes swing with its heat far more
    than with anything built that day (handing the card its work 0.29 ms at 84 °C, about 0.5 at 104, one program).
  - Nothing waits on him but his word for what of 2.4 comes next.
- **[IDEA, David 2026-11-16] Finer far detail just past the triangles** (to Builder): "i have a question. once the
  triangles end, is it 2x2 rays?" (Builder: no; mostly 8 × 8 rays, 4 × 4 in places, the 2 × 2 layer marching no ray
  at any mark that day.) Then: "those seem to do something at a great distance, but none of those controls seem to
  have an effect on the detail of the nearest of the far view rendering. i ask because it seems like a a ring of 2x2
  fading from the triangle radius might look better, when you have a fence half in and half out of the render
  radius."
  - Builder's answer to him: the controls don't reach it, and the limit there isn't the rays but the far cells: past
    the triangles the far view is 32-block cells (each about 21 screen pixels wide at 1,024 blocks), and finer rays
    over them would only magnify the blur. What would give his ring is the fine rings (the 8s and 16s) sitting just
    past the triangle radius, the finer rays then coming in by the rule already in; a real step, his call. Also: the
    far eye doesn't yet see anything placed in play, so a player's fence past the radius isn't drawn at all, its own
    step.
  - *Counsel, a figure to check before he weighs it* (put to Builder): she told him the fine rings lie "within 128
    blocks of the eye"; `src/far_ring.rs` says 128 cell columns, "so the 16s reach about 1,024 blocks from the eye
    and the 8s about 512". If so, at his 32 chunks (1,024 blocks) the 16s end just where the triangles do, a
    coincidence of the two numbers; a free look is any smaller radius (at 16 chunks the far view past the triangles
    is the 16s out to 1,024); and the step may be only a ring a size wider, not a ring with a hole. The Oracle's
    first reply said the same of the rings ("the rings as sized sit mostly inside the near eye's ground"; "ring
    width, not ring count, sets how big the star facets look").
  - **Confirmed by Builder, her first figures withdrawn in the open** (read in the code: `bricks.rs` has RING = 128
    cells; `far_ring.rs`'s reach spans RING × the cell's size): the 16s reach about 1,024 blocks from the eye and the
    8s about 512, each fading into the next level over its outermost 8 cells (128 and 64 blocks). She'd taken the
    flight header's "a ring of 128 round the eye" for blocks. So: at his 32 chunks the 16s' ring ends about where the
    triangles do, which is why the nearest far view is 32s there; the free look is a radius of 16 chunks against 32
    (at 16 the far view past the triangles is 16s out to about 1,024), not a radius of 4; and the step is the rings
    a size wider, the march's rule as it is, the hole only a saving of memory. Her estimate, to be measured: four
    times the bricks in each ring by area (about 1,800 to 1,950 each for the day's 445 and 487). Still his idea,
    open; corrected to him by her.
- **[DECIDED 2026-11-16] His checklist for finishing graphics** (David, to Builder; in `master/plan.md` under 2.4,
  be2328b): "we need placed things, near light, glows and far light, textures, an assessement of a fine ring after
  the triangles to transition more smoothly, listing chunks, faces turned away, a chunk with pieces taking too long,
  chunk optimizing if theres a visible way, dont worry about very fast flight." On what's set aside: "the lazy rays,
  cover by height, weights by kind, and i dont know what toom in the march is, and tbh idk how to make tays prettier
  personally. ive been learning about shaders but am not sure how to pretty it up rn."
  - As written in the plan, in his order, each its own build with its pass mark first:
    1. Placed things in the far view (he called it necessary before graphics are done enough to move on).
    2. Near light: per-face blocky light, placed lights in 16 steps, coloured, flickering, stopped by walls; moving
       lights; caves and interiors dark unless lit.
    3. Glows and far lights, fading into fog, never snapping off.
    4. Textures: bars' own side and end pictures, the grass edge, pictures for the stage's seven blocks
       (placeholders or his art).
    5. An assessment of a fine ring past the triangles: the 16s' and 8s' rings a size wider behind a switch, A/B at
       the same radius, cost measured; his eye decides.
    6. Listing chunks (about 1 ms a frame at 64 chunks).
    7. Faces turned away left out.
    8. A chunk with pieces meshing nine times slower.
    9. Chunk optimizing where the gain would show (filling time; the corners' program).
    Not now: very fast flight.
  - Not in his list, left in 2.4 as they were: water's look, objects, glass, the doorway's pieces and lights, the
    Intel flight and his walk-through gate.
  - The set-aside list stays set aside (lazy rays, cover by height, weights by kind, room in the march, the
    prettier-rays passes). Builder explains "room in the march" to him plainly (the march holds 167 or 168 registers
    of about 170, so anything new carried through its loop needs something taken out first; the candidates are in
    `journal/docs/march-by-parts.md`), and that the prettier-rays passes are hers to propose and his to judge by eye,
    no shader knowledge needed of him.
  - *Counsel:* on water's look, no question put to him (his "drop any homework or questions"): his new order has
    water designed afresh after topography in world generation (3.2), which will change the shores this look was
    about; it can wait there unless he sees something he dislikes sooner.
- **[DECIDED 2026-11-16] For the march's trimming: the weave and the blue noise can come out** (David, to Builder; a
  note for the later trimming, not a build now): "ok, note for the trimming the march, the blue noise and weave
  section can come out. it really does almost nothing." (His look of 2026-11-13 on them: "very subtle".) When the
  trimming comes: the Render page's Ray start keeps "scattered" (the default) and its "spread" slider; "woven",
  "blue noise" and "woven + blue noise" go with their controls, and the blue-noise picture if nothing else reads it;
  up to 92 instructions out of the march. In `journal/docs/march-by-parts.md` (candidate 3) and the plan's later
  look back.
- *Build 207, item 1 of his checklist: placed things in the far view* (mark d04b382, code 3719fad; one flight down at
  18:00, `logs/flight-1794873606`, `logs/session-1794873113.log`; checked by Counsel). A mark's whole blocks and
  pieces are summed into the far cells (a cell of pieces as its main material, never air, old's rule), the regions
  they touch summed again on a thread and put over what's on the card, the far textures, height table, light and the
  16s' and 8s' rings made again. Off the screen: his build stands in the far picture (a small dark patch in the 8s'
  cells from 290 blocks; mostly air, smaller than a cell there); the scatter of 10,000 a dark field; taken back, the
  bare plain's picture to the pixel. 44 tests pass. In the game, at each of the eight standings with something placed,
  the session log has "What's placed: 1 far regions summed again and put on the card" in 10.8 to 33.2 ms, 0.09 to 0.16
  s after it was made; frames as before (medians 1.02 to 1.24 ms at those standings). Done enough to go on; his look
  when he likes. Next: item 2, near light, its plan brought to him before code (how dark caves and interiors go is a
  look of his).
- **[DECIDED 2026-11-16] Near light: a very dark floor outdoors, caves black past about their mouth's width** (David,
  to Builder, on her plan for near light): "i think you understand what i want for light. i think outdoors should have
  very very dark as the floor, but caves should be pitch black past a distance into their mouth that would equal i
  think the diameter of their mouth or something of that sort."
  - Taken as: step 1, sky light per block, goes ahead with his two rules: outdoor shade never below a very, very dark
    floor; light reaches into a cave about as far as its mouth is wide, then black. Minecraft's flood can't do the
    second (it always reaches 15 blocks), so Builder's light spreads by averaging, walls counting dark, which fades
    over about an opening's width. Build 208's mark written.
  - The test light's colour for step 2 (placed lights) wasn't answered: warm torch orange unless he says.
  - *Counsel:* it keeps his older words: "pitch-black darkness" in caves (the caves stops of 2026-11-01, built in old)
    and "very dark nights" (the plan's "What every step keeps").
  - **No absolute flights tonight** (David, to Builder): "note that the tv is plugged in now, so we'll have to consider
    whether a flight is really critical for the rest of the night and get as much as we can done that only needs
    relative measuring... anyways, i don't mind putting my finger on the scale and seeing if it stays steady. there's
    a little bit of performance to be squeezed out yet." So: only relative measures tonight (counts off the screen, or
    a bracket in one session if he's fine with the TV's noise). The team kit's warning, met before (a night of
    flights 4 to 18% slow with the TV on the same card).
  - *Build 208, near light step 1, the near eye's own sky light* (cc3a824; unlaunched, unflown). Each face lit by the
    cell it looks into; light spreads into openings by averaging, reaching about as far in as an opening is wide
    (tested: a tunnel 2 wide lit 3 blocks in, 4 wide 5 in), black past that; outdoor shade never under a very dark
    floor. Off the screen with the stage on, the cave near the spawn goes black inside and trees throw shade. The
    cost, said plainly by Builder: merged faces split where light changes, so the spawn at 4 chunks keeps 40% more
    faces (190 thousand for 135 thousand) and a chunk takes about 20% longer to make; at 64 chunks the near faces'
    cost goes with faces sent, so it wants pricing by a bracket, and may move "faces turned away" up his list. Next:
    his look, then step 2, placed lights.
- **[DECIDED 2026-11-17] Light kept apart from the faces, and faces turned away, before placed lights; David the
  go-between** (to Counsel). He asked "why do the face meshes have to split when the light changes? how much would
  faces turned away help", then "lets talk about light in its own picture. how does this work for all nonsky light?
  at some point we said rays through dust would be cheap, but that was based on gpu ray light right? and not drawing
  faces turned away does seem obvious." Then: "please all of this about light into a prompt for builder to boil down
  between the pros and cons of a light picture only around you or a small strip next to all merged faces, i agree
  with your suggestions about the order. ill be doing most of the go-between you and builder for now, builder and
  you will check in on the master map and plan at each committed build".
  - Counsel's answers, unmeasured and said so: a merged face carries one light value, so merges stop wherever light
    changes (build 208's 40% more faces). Kept apart (a value a cell: sky, and red, green and blue from placed
    lights), faces merge whatever the light; a torch placed or moved re-spreads a small patch and rebuilds no chunk;
    flicker is a wobble at the read. A full light picture of every near chunk is too much memory at 1,024 blocks
    tall, hence the two forms for Builder to weigh: a light picture only round the eye, or a small strip of light
    values beside each merged face. Rays through dust were cheap because far rays already marched the air; near,
    they'd be a pass of their own reading the light picture, outside the march. Faces turned away: the card drops
    them only after their corners' work (49% of the near faces' cost by his trace); a third to a half of faces sent
    is Counsel's rough guess, to be counted off the screen.
  - **The order, agreed:** faces turned away and where light is kept, before step 2 (placed lights), so placed
    lights are built once and later prices are read against the right baseline.
  - **The way of working, for now:** David carries most messages between Counsel and Builder; each of them reads
    `master/map.md` and `master/plan.md` at each committed build.
- **[DECIDED 2026-11-17] The graphics engine is never welded shut** (David, to Counsel, after talking through figures
  drawn as live pixelized 3D, the Dead Cells and Guilty Gear way): "i see that it would be a development process that
  impinges on the graphics and kinematics and requires a lot of content to work on correctly; we have to make sure
  the graphics engine is left in a way that its not welded shut, so that we can come back to it often. i don't think
  we'll really ever be finished with it until the end, and we're going to probably have to try and throw away some
  styles once we have enough content and finishedness that i can actually practice with combat in it." Kept in the
  plan's "What every step keeps".
  - **[IDEA, open] Figures as live 3D drawn chunky.** His questions: "is it possible to pixelize and crush live 3d
    triangle models so they look like a hi def sprite, maybe even deinterpolate steps of motion like antismoothing";
    whether, in the world, "it would be able to have depth", how villagers at a bakery or a carpenter's bench would
    look, and whether in combat "you [would] be able to target and collide the 3d models and weapons correctly and
    'collimate' with screenspace". And his doubt, unresolved: "i find it hard to imagine if the 3d table and quantized
    carpenter would look right, or look like looking at a fish through water and trying to grab it."
  - *Counsel's answers, unbuilt:* the figure stays real 3D in the world (depth, hiding and hits on its true shapes),
    only its pixels drawn chunky and its colours crushed, poses held (Quake's snap); a flat sprite only far off. Two
    risks named: hits worked out between drawn poses (use the held pose for both), and chunkiness close up in first
    person (ease it off near). His fish-through-water worry is the real one: a chunky figure against a sharp table
    misreads where the two meet unless both share one pixel grid and one depth; only a test with real content says.
    References he's looking at: Dead Cells (Vasseur's pipeline article), Guilty Gear Xrd (Motomura, GDC 2015),
    t3ssel8r's 3D pixel art.
  - He found Dylan Ebert's texel splatting (https://dylanebert.com/texel-splatting/) and asked Counsel's view; on
    "add a pointer to it in the lazy-rays document and the Figures notes?" he said "yes" (done:
    `journal/docs/lazy-rays.md` section 10, `journal/docs/vision.md` Figures).
- **[DECIDED 2026-11-17] A graphics style pass, once there's a slice of gameplay** (David, to Counsel): "it seems we
  will need to do a whole graphics *style* pass once we have a healthy amount of content for a small slice of
  gameplay." In the plan under Phase 4.
  - **[IDEA, his] The near view on the far eye's own 2 × 2 tiles:** "is there a way to make everything in the near
    view quantized from triangles to a 2x2 grid that uses the same screentiles as the raymarcher? like slipping the
    triangles like a pack of compound butter under a turkeys skin?"
  - *Counsel, unbuilt:* yes, and cheaply, since the near eye already draws into a picture of its own (build 199)
    that the x-fader lays over the far one: draw that picture at half size on the far eye's 2 × 2 tiles (the same
    corner, so the squares coincide) and lay it on without smoothing. Edges either hard, or averaged from several
    samples a square ("edges that average and glide", a look the plan keeps). It saves the faces' pixels' work, not
    their corners' (15% against 49% by his trace). The known flaw of any screen-fixed grid: squares crawl over
    surfaces as the eye moves, worst near; what texel splatting is for. A switch on the near picture's size (1, 2,
    4) would let him see it; for the style pass, or sooner as a look if he asks.
  - **[HIS LEAN, 2026-11-17] One fixed 2 × 2 screen that everything feeds** (David, the same talk): "i imagine the
    raymarcher as an adaptive grid of cells in the bottom, masked and subtracted by the triangle field, both feeding
    the same simple 2x2 screenspace that would be the fixed monitor. i think its already setup so the march doesnt do
    work that triangles mask. UI elements can also draw on this grid or even use a 4x4 subgrid on top with
    transparency rather than masking. i think that may be a good artistic interface i could work around, from there i
    could try different 3d styles in between the triangles and the 2x2 screenspace. halving the actual resolution the
    game runs at is probably good. it would still be a borderless window of 2160x1600 but we would actually be
    drawing 1080x800. that may actually be enough to make it all mesh nicely, once its polished. i see propixelizer
    calls them macropixels. not sure what it does that i havent specified other than color crush which i do at an
    asset level. a lot of it seems unnecessary".
    - *Counsel:* the laptop's screen is 2,560 × 1,600 (every flight's header), so half is 1,280 × 800. The march
      already skips what triangles cover (builds 201 to 203), as he thought. Drawn at half size, the far eye's
      layers of 8, 4 and 2 screen pixels are 4, 2 and 1 of the picture's own, the same squares on the glass, and
      every pass has a quarter of the pixels (the far eye's cost is its rays, the near eye's its corners, so the
      saving is a part, to be measured, not a quarter). It's his older word come round: "a beautiful world,
      squinted" at about 640 × 480, and the milestone "a DOS-game resolution where everything is there"; it changes
      the spec's "the 3D world at native resolution" if he settles on it. What ProPixelizer adds beyond what he's
      specified: snapping things and the camera to the macropixel grid against crawl (for a flat, no-perspective
      camera; it doesn't hold in perspective), outlines from depth and normals, and a pixel size per object; the
      rest is the colour work he does at the asset.
    - The whole style, with where each part sits in the frame's passes, is noted at his asking in
      `journal/docs/graphics-style.md`.
    - **When it's taken up** (David, asking for the plan's place to be put in the note too): "we'll probably have to
      keep moving after step six there, and come back to this when we have enough content to work on style." So
      after his checklist's sixth item graphics are left, Phase 3 begins, and the style waits for the style pass.
- *Read from the plan and the journal at builds 209's commits* (David now the go-between; Builder's records):
  **where light is kept, his word** "proceed with C", of three ways put to him with counts: light kept only where it
  isn't plain daylight, in one table on the card a face's pixel looks its cell up in (no entry: plain daylight), so
  faces carry no light and merge whatever it is; a small light picture round the eye later, for moving lights only
  (734acf1). Build 209, faces turned away (ff0c6b2, unflown): the near faces' pass had never culled at all; now the
  card drops turned-away faces and a chunk's groups that all look away aren't listed: half as many faces sent at 32
  chunks off the screen, the listing 0.02 to 0.03 ms dearer; dark dots on roofs and slopes gone. Build 210 is the
  light's table.
- *Read from the plan and the journal at d3f647e* (Builder's records, checked against the logs): **build 209 flown**
  in a bracket of five (the clean three: off, on, off; he'd been at the laptop for the first): the near faces' time
  on the card a third less over the marks, half at the spawn (0.871 / 0.411 / 0.876 ms), the dearest frame 1.89 to
  1.62 ms. **Build 210 flown** in a bracket with the stage on (`flight-1794926555`, `-1794927683`, `-1794928862`:
  build 208's way, the light kept, 208's way): the near faces 15% less over 20 marks (10.509 / 8.840 / 10.337 ms
  summed) and at no mark dearer; even over a wood, where most pixels find a brick; the dearest frame 2.30 to
  2.17 ms (the stage on and the TV in, so read only against each other). The light 20.0 MB for 1,651 chunks at the
  spawn. Both done enough to go on.
- **[DECIDED 2026-11-17] A finer region past the triangles, next; the light looks really good** (David, to Builder,
  at build 210 launched with the stage; relayed by her whole): "this looks really good, builder. the only rough part
  is this. i feel like we should have a finer region between the triangles and the march." (with a picture of trees
  smearing just past the render radius), then: "yeah, i think we need to push the fineness-grossness gradient out to
  the triangle radius; please check with counsel on the things counsel and i have discussed about the end state of
  graphics re. a 2x2 screen grid everything snaps to etc".
  - So his look at builds 208 to 210 is had: the light's cut and floor stand as they are. And the checklist's item
    5 (the assessment of a fine ring, 2026-11-16) moves ahead of placed lights, and grows: no longer the 16s' and
    8s' rings "a size wider" but rings that begin at the render radius and follow it. The switch and the A/B at the
    same radius stay; his eye decides whether it stays.
  - Builder's reading, put to Counsel: the screen's own grain is a 2 x 2 square, so a far cell is fine enough when
    it's about one square wide on the glass: about 3 blocks at 1,024 blocks out, 6 at 2,048, 12 at 4,096. Today
    past the triangles at 32 chunks the cells are 32 blocks, ten times the grain; the 16s and 8s sit wholly under
    the triangles. A far brick with its textures is about 20 KB, so memory and making time are measured before any
    size is promised.
  - *Counsel's answer to her:* the reading holds as a target, but the fixed 2 x 2 screen and half size are his lean
    for the style pass, nothing decided, so nothing is built to it; the march's finest layer is 2 x 2 pixels
    already. The squares line up by themselves (the layers share a corner); only keep the picture's size out of
    the ring's rule ("never welded shut"). Outside the style note and bearing on it: his first words for the ring
    ("a ring of 2x2 fading from the triangle radius ... a fence half in and half out of the render radius"); a
    gradient wants a fade at every step; the radius is a slider and his goal is 64 chunks, where the grain is twice
    as coarse, so the ring is priced there too; the ring lies under the whole x-fader band (128 blocks), so it
    begins inside the radius; a ring with a hole shows the far view only where near chunks aren't in yet, to be
    said to him plainly; the march has no registers put by; remaking is measured with a placing and a walk; and
    what a tree looks like at the cell size memory allows is told to him before he looks.
- **[DECIDED 2026-11-13] Light read from the cell: the sun's steps snap** (David, to Builder, asked whether the sun's
  steps should fade or snap, and whether shadows should be live while he drags the sun's slider): "i think we should
  try snapping light from one of 16 levels every time the sun has moved a 16th of the day, and smooth it only if its
  bad. that snap may be pretty subtle and act as a little clock for the player. live while dragging would only be for
  my benefit and i dont think itd really benefit me." So the far shadows are swept for the sun's place at each
  sixteenth of a day and snap to the new one, no fade unless he finds the snap bad; the next sixteenth is made ready
  before it's shown. While he drags the sun's slider the shadows catch up after he lets go. With no day in the new
  engine yet (the sun is two sliders), a sweep runs when the sun's setting changes; the sixteenths apply once time
  runs. (Inside his word of 2026-11-11: "they can change every 30 seconds for all i care"; a sixteenth is about a
  minute of a 15-minute day.)
  - **What light costs the far eye now** (build 159, light on and off alternating): the spawn 0.66 / 0.29, the plain
    0.87 / 0.77, the mountain top 1.34 / 1.04, high over the spawn 1.30 / 0.82, the sea 1.35 / 1.14, a player's run
    1.38 / 1.13, the sea's nodding 99th 1.54 / 1.32. **11 to 56%** of the card's whole (Builder guessed 5 to 15%; at
    2b it was 3 to 13%, before the march got faster), now the far eye's biggest single piece. With the light's rays out
    of the frame every mark would be 1.14 ms or under, less the cost of reading a cell's light.
  - **Builder's design, its pass mark next:** on the processor, each far level's columns get two numbers: the height
    the sun's shadow reaches there (one sweep across the level along the sun's way, a few milliseconds a level), and
    how much sky the column's top sees (from the horizon in eight directions, once at load). They go to the card as
    two small pictures (about 40 MB for the whole world), read where a pixel lands: no light rays in the frame. Behind
    a switch at first, flown against the rays, then his look, since the picture changes.
  - **[OPEN] What it gives up, for David:** the far cells are treated as a surface for light, so shadows under
    overhangs and arches aren't made in the far view.
    - *His condition* (to Counsel): "far light skipping any small details out there may work if the smaller details
      can carry their own 'shadow', or absence of light, if they're rendered differently. everything has to look
      even, and reveal everything accurately though. not sure how that'll work". *Counsel's reading of the ways it
      could work, for Builder to price:* (1) the column's light as designed, where an overhang's underside still
      darkens itself, since a cell's faces are counted by the way they look and downward faces see no sun; (2) light
      kept a cell at a time in 3D, as experiment 4b did (David couldn't tell it from rays), which shadows arches
      truly at more memory and sweep time; (3) both: the column's light for plain columns, a cell's own light for the
      few that aren't (overhangs, arches, slabs in the air), the same split the Oracle drew for plain land and the
      rest.
    - *Builder's prices and order, before any flights:* way 1 first as pictures (about 40 MB on the card, a sweep of a
      few milliseconds; it's also way 3's base); then way 3 on top, cheaper than it sounds: each far cell carries its
      own "cover", how much solid stands over it in its column, made once with the far textures and riding in a
      channel that's empty today, so no memory and no extra fetch; a cell under an overhang, arch or slab dims its own
      sky and sun, true for what's overhead and approximate for a low sun from the side (about 40 minutes of hers).
      Way 2 held as the fallback (36 MB more a copy, 72 for the snap; a sweep over about 36 million cells each sun
      step, seconds across threads; about three hours), only if David finds way 3 uneven. Side-by-side pictures,
      rays against each way, at the four marks and at slabs and an arch, brought to him before anything is flown.
    - **Pictures first, off the screen, and detail to judge them by** (David, to Counsel): "if she can make me the
      pictures of those without taking the screen, and then fly them later after looking at them and discarding any
      that are obviously bad, that'd be good. i suspect that they'll all look OK from just the land. after adding
      forests, buildings, and caves, there'll be something to look at, particularly in the transition. we may want
      to consider adding some detail to the world; im sure builder would have fun placing some caves, towers, town
      buildings, and forests of her own design as test objects. Shouldn't a lot of those exist in the 'edit' layer of
      the worlds 'tablet', though? tricky to sequence everything to not waste time from being too early or too
      late."
      - *Counsel:* the pictures are made off the screen by the picture tests, so his screen stays free; flights only
        for the ways that survive his look.
      - *On where test detail lives:* two homes already in the plan, both fed through the doorway, neither world
        generation. The changes layer ("what changed", already carrying his 169 placings) suits hand-placed objects,
        kept like a player's builds: Builder's towers, town buildings and carved caves. The scatter stage behind the
        doorway (the Oracle's third part, section 7) suits wide repeating detail, made from a seed as tiles are
        answered and costing nothing to keep: woods, rows of posts. Both serve the cells of 2 to 16, the far light's
        evenness and the near eye's marks (2.3), so made now they're not too early. *Counsel's proposed order:* the
        light pictures on bare land now; then the test detail; then the light pictures again with detail, and
        flights only for the ways that survive.
      - *Builder's refinement, which Counsel takes:* the test detail belongs **with** the cells of 2 to 16, not
        before. At today's far cells (32 blocks and up) a tower, a house or a cave mouth is smaller than one cell, so
        it can't show as a shape or cast a shadow of its own whatever the light does; building it first would be
        early by exactly that step. So: the light pictures on bare land now, his look, flights for what survives (the
        column light's win, up to 0.4 ms at the spawn, in hand); the skyline look where it stands; then the cells of 2
        to 16 with the scatter stage and the test detail in the same step; then the light pictures again with detail,
        where way 3 (each cell's own cover) is judged. What she'd build first: woods from the scatter stage, then a
        few hand-placed objects in a test world's changes (a tower, a free-standing arch, a slab in the air, a cliff
        with a cave mouth, a short street of buildings), shapes whose right light is known; she asks David where and
        in which world before placing any. Her first picture, high over the spawn: the column light close to the
        rays', the same shadows in the same places, a little crisper.
      - **His look at the first two pictures** (rays beside the column light, high over the spawn and at the spawn,
        to Builder): "sounds good. those pics look fine to me." Then, on flights: "yeah hold on for a bit, i've got
        to take a break from watching the screen. we can try to queue up a bunch of flights, and you can run them
        while i'm sleeping, maybe." So no flights while he rests: Builder works off the screen (the pictures; the
        column light committed as build 160 unflown; way 3, each cell's own cover, behind a switch with pictures),
        and writes a night queue as one build with switches (rays against the column light; with and without the
        cover; the finer base re-priced now light is cheap), shown to him before he goes and started only on his
        word.
      - **Build 160, the column light, committed unflown.** All five marks drawn off the screen beside the rays: the
        shadows in the same places, a little crisper, nothing off. Its lag: the whole world's tops and sky made in
        0.19 s, a shadow swept and sent in 0.05 s, a change of the sun showing in about 0.3 s, inside the pass mark's
        half second; 40 MB on the card. **Way 3 shelved** with its design until the cells of 2 to 16 and the test
        detail: on bare land a cell's cover is nothing, so its picture couldn't be judged.
      - **The night queue** (pass marks ffbd581): build 160, five standings in turn, three rounds, fifteen flights,
        about 100 minutes: the column light; the rays; no light at all (what reading the cell costs); the layers one
        after another with the column light (what the one dispatch is worth now); the finer base with the column light
        (re-priced for the polish list). Each flight stopped if not ended in 15 minutes; rounds repeated so a starved
        hour shows; reports name the commit. His list: plugged in, Turbo, kept awake with the screen on. Starts on his
        word.
      - **The night queue, flown on his word** (2026-11-13, 19:41 to 21:20; Builder's write-up in
        `journal/docs/experiments.md`). Medians: the column light costs what no light costs (within 0.02 ms at every
        mark), 0.12 to 0.49 ms under the rays, 0.33 to 1.21 ms in all; the one dispatch is worth 20 to 35% with the
        light's rays gone (the plain 5% dearer by it); the finer base costs 0.35 to 1.33 ms. Not readable: the 99ths.
        From half an hour in every standing's 99th ran 1.5 to 2.5 times its median, no light included; the night ran
        4 to 18% slower than the afternoon for the same picture; the first two flights flew starved. So his 2 ms bar
        is shown for build 159's rays, not yet for build 160.
      - **The likely cause** (David, 2026-11-14 morning, to Builder): "only thing i can think is either the nightlight
        mode, or some awful parasitic nonsense hidden in my pc, or a considerate bitcoin miner virus that runs when i
        sleep. i did close the lid and open it again. oh wait, i plug the laptop into my TV to play space noise sound
        videos. i bet its just the card running another display. all yesterday it outside with no other display".
        Builder's reading: it fits (on "NVIDIA GPU only" a second screen and a video's decoding are the same card's
        work), not proven. **Owed:** two day flights with him at the machine, nothing else plugged in, Turbo on (the
        column light and the rays, fourteen minutes), to read the 99ths; the rays' light code stays in the march until
        his word.
      - **The day flights, on his word** (2026-11-14, 09:32 to 09:45, "Alright, lets get started"; Builder's write-up
        in `journal/docs/experiments.md`, 492c2f3; reports `logs/flight-1794670752` and `-1794671143`, checked by
        Counsel). The laptop off his TV, one screen, on mains, the card idle at 18 W beforehand; Turbo not read. The
        column light: medians **0.29 to 1.15 ms**, 99ths 0.30 to 1.34, no single frame over 1.56 ms at any mark; no
        more than 0.012 ms over build 159's no light, every pass mark met. **His 2 ms bar is shown for build 160.**
        The rays matched build 159's afternoon to about 0.01 ms, 99ths too, so the TV is taken as what spoiled the
        night. The column light is kept by its pass mark; the rays' light and its hand-down stay in the march until
        his word to cut them (Builder asking him).
      - **One last look before the cut** (David, to Builder, asked his word on cutting the rays' light): "i would like
        to be able to take a look, and a/b column light vs rays one last time." Build 161 (b843d87), launched for
        him: a live switch, placed and named by him, a checkbox in the panel's Light section under "sky light",
        called **"light by rays"**. Both ways draw build 160's pictures byte for byte; not flown, as nothing a frame
        does changed. The switch goes when the rays' light is cut, on his word after the look.
      - **His look** (2026-11-14, to Counsel, build 161): "column light is actually cleaner, i prefer it". The column
        light is kept on his eye as well as its pass mark. The cut itself (the rays' light, its hand-down, the
        checkbox) asked of him in so many words before anything goes.
      - **[DECIDED 2026-11-14] Cut the rays' light** (David, to Counsel, asked whether Builder cuts the rays' light
        and the "light by rays" checkbox): "Yes, cut them". The rays' light, its hand-down and the checkbox go; the
        far view's light is the column light. Relayed to Builder, the cut built once he's done playing build 161.
        To Builder directly, the same look: "column light actually looks better."
        *Built:* build 162 (4effc9e), unflown: the shadow ray, the five sky cones, the sky light handed down, the blends
        only they used, the switch and the checkbox gone, 206 lines of the far shader's 1,354; all five marks' pictures
        byte for byte as before. One check broke on the cut (the blend test called a cut blend) and was mended in the
        open. One flight owed, on his go (pass mark: nothing to 2% less on the card).
    - **[OPEN 2026-11-14] The whorls close up** (David, to Builder, from the same look, a screenshot close against a
      mountainside, the 32-block cells hundreds of pixels across): "i know that this will be more of a polish thing,
      but its significant enough, close enough to the basic functions of the graphics engine, i'd like to have a talk
      about it; the way that these whorls work out from the average of the rays, even if its hard to see at a
      distance, if these were just more *orderly* , the way they compress into the distance would improve. im
      interested in ways that we could get that sort of tuned, ideally without incurring a whole lot of extra costs in
      the process." Builder is answering him with where the shapes come from and the ways to order them, cheapest
      first; nothing built. Her reading is in `journal/docs/experiments.md`.
      - **[DECIDED 2026-11-14] The ways, and the order** (David, to Builder, on each): the panel's "brought up",
        "smooth and sharp skylines i may ask you to cut at some point. im waiting until i have more things to look at
        with it"; an even pattern for a ray's start, "an even pattern with a fine weave sounds good. i would like for
        you to put it on a switch for me"; a firmer rule from fullness to cover, "this also sounds promising but i
        believe i would need some more finished content to judge it by. it sounds like it could be a slider of some
        kind as well"; a blend eased at the cell centres, "this sounds good, i would like to see this with a switch
        as well"; the dear ones (eight times the fetches, or cells holding distance), "that sounds painful, i would
        avoid this in favor of simpler fixes. have you heard of blue noise being used for this kind of thing?"; and
        "i'm fine with your proposed order of looking into this." On placing and timing: "if they can work
        simultaniously i'd like to see that as well, and if they have parameters worth exposing i'd like to play with
        those. you could put them under a ray start section or something. If there's no reason to wait, we could try
        the blue noise and weave, and then you can run the tests that you have qeued after i take a look".
        - *So:* the dear ones are out; simpler fixes first. The flights queued (build 162's cut, the switch's
          standing cost) wait for his look at build 163.
        - *Built:* build 163 (c608853), unflown: a "Ray start" section on the Render page, under "The picture", as he
          placed it. Pattern: scattered (as it was, the default, byte for byte build 162's picture), woven (2 to 16
          pixels a side), blue noise (made at load in 0.09 s, 4 KB), or the weave with blue noise in it (a slider);
          and a spread slider. Builder's own look off the screen: the patterns change only the speckled patches
          where rays graze land; **the whorls themselves don't change**, being the blend's, which the eased blend
          (next) is for.
        - *His look at build 163* (to Builder): "well, the stuff we added didn't hurt. they look good together, but
          its very subtle. its kind of these sharp blocky sections that bother me a little. idk if there's anything
          that can be done about that." Builder is finding out off the screen what makes the sharp blocky sections
          (the code's and the hardware's blends show them alike, so not seams between bricks).
        - *Flown at its defaults* (on his standing word, "you can run the tests that you have qeued after i take a
          look"; `logs/flight-1794676498`, checked by Counsel): against build 160's day flight, the card's whole 0.7
          to 1.8% less at every standing mark, inside the pass mark. Said in the open: fast across the world 4%
          dearer (a moving mark that varies about that much between flights of the same code); the mountain top's
          99th 15% wider, one frame at 2.03 ms, in a stretch with a 45 ms hitch while David was typing, taken as
          disturbed, not the build's; a hands-off flight owed for it. The flight took off before Builder told him
          it was up (her lapse, said in the open in the journal and the shared memory).
        - *The sharp blocky sections, narrowed* (Builder, off the screen at his first spot, 904aa65): the code's and
          the hardware's blends show them alike (not seams between bricks); with every face lit alike they stay (not
          the light); with no switch to bare rock nothing changes. What's left is the blend's own creases, which the
          eased blend (way 4) goes after. She asks him whether to build it next, on a switch, and where its control
          goes. His second screenshot (flat plates in a hazy spire) may have another cause; she needs its place.
        - **A correction, in the open** (Builder, 660180a): the line above is wrong where it says that with no
          switch to bare rock nothing changes; she had judged it by eye on one side of the view. Measured over the
          whole of his first view, removing the switch changes 12% of the pixels, and what changes is the flat
          straight-edged plates; at the sea and the plain marks 17% (the near ground loses a grey cast). The
          star-shaped facets are still the blend's creases. Counsel had passed the wrong reading on to David too.
        - *His words, before the correction reached him* (to Builder): "smooth softens them but doesnt change them. i
          will wait for 2-16 to to judge them. next time i have the game, remind me to find some blonky area and try
          the old code. id have thought we would have cut that though. but, i think we seriously need to take a look
          at rock switch and texture making. anything like that is something we dont want to miss the chance to tune
          up while its cheap or we forget". So: the creases judged at the cells of 2 to 16; the rock switch (build
          164, on a flag, its control's name and place his) and then the texture making (borrowing across bricks),
          each with its own look; at his next launch, the reminder: a blocky area, "far blend" on "old's (code)".
        - **Named and placed by him** (to Builder): "ah, i see about old codes blend. maybe it could be called
          "initial blend". i think you know what to do, proceed". Build 164 (7f852ab), unflown: the checkbox
          **"rock switch"** in "The picture" under "far blend", ticked as it was by default (the picture byte for
          byte build 163's); the far blend's first choice renamed from "old's (code)" to **"initial blend"**, on
          the panel and in a flight's header. His look first, then a hands-off flight announced to him before it
          takes off; the texture making after, as its own build.
        - *His look at build 164* (to Builder): "unticked rock switch looks a lot better, with the hardware blend."
          Builder puts to him next: unticked as the default in build 165, a launch switch for the old way so the two
          fly against each other hands off; the checkbox kept until he says to cut it; the texture making the build
          after.
        - *Built on his standing "i think you know what to do, proceed"* (Builder; both unflown): **build 165**
          (ab05594), the rock switch off by default, byte for byte what he saw unticked; the checkbox stays, and
          `INCUS_FAR_ROCK=1` starts it on to fly against. **Build 166** (7aa8518), the texture making: a cell with no
          surface borrows from the nearest surfaced cell in the world, not only in its own brick, so bricks agree
          where they meet; in the pictures it barely shows (3 of 255 at most at his first spot), so it's for
          soundness, not a look. At load 0.6 s for the whole world, after a first try that took 4.4 s (written up
          in the open). Owed: fourteen minutes hands off for two flights, the new default against the old way, which
          also gives the mountain top its clean 99th.
        - *Flown on his "go ahead", hands off* (`logs/flight-1794681652` off, `-1794682043` on; checked by Counsel):
          the rock switch off against on, the same (medians within 0.3%). **A miss, said in the open:** both 1.2 to
          3.6% dearer than build 163 at the standing marks, in the march alone; the one change to the march since is
          the flag's own compare, paid ticked or not (Builder had guessed nothing either way); not proven until a
          build without it flies. Good in the same flights: the mountain top's 99th clean, 1.09 (1.23 disturbed),
          no frame over 1.8 ms; bricks brought as the eye moves no dearer with the wider borrowing; the textures made
          and sent in 0.5 s (3.8 s in build 163's flight). Builder asks him next for his word to cut the checkbox
          (his control), then one hands-off flight to show the march gets it back.
      - **[DECIDED 2026-11-14] Cut the rock switch** (David, to Builder): "yeah, we can cut it and move on". Build 167
        (9868d0d), unflown: the flag and its compare in the march, the "rock switch" checkbox and `INCUS_FAR_ROCK`
        gone; the far view always borrows, as he preferred; the picture byte for byte build 166's at all five marks.
        The initial blend keeps its own switch. Pass mark: the march gives back the 1.2 to 3.6%, each standing mark
        at build 163's median or within 1% of it; if not, the builds between are flown apart. One hands-off flight,
        seven minutes, on his go.
        - *Flown* (on "alright, lets proceed."; `logs/flight-1794683091`, checked by Counsel): **the pass mark met**,
          every standing mark within 0.7% of build 163's median and 1.0 to 3.1% under build 166's, so the
          checkbox's compare was the cost and the cut gave it back. The plates gone, the textures sound and made in
          0.5 s, the march where it was. Not quite hands off (David wrote to Counsel during it); no big hitch on the
          frame clock; the 99ths match build 163's but at the mountain top, where the card's clock dipped and one
          frame of some 19,000 read 2.02 ms (build 166's two hands-off flights: nothing over 1.8 ms there).
    - **[LATER] The vanishing sea line:** at a 4 base, the thin blue line of sea on the horizon low over the sea isn't
      there. David watched: "i didnt notice anything". Cause unknown after three tries (not the step limit, not the
      sky's weight, not the map's edge); to be found before a finer base is ever kept.
- **[DECIDED 2026-11-13] Fog: for looks only, off by default** (David, to Counsel, at "the fog line" in a summary: "we
  are not using fog."; asked whether to take it out or switch it off: "off by default would be better. it looks nice
  but is most likely reserved for final touches"; and to Builder: "what i mean is we are not using fog or haze or any
  sort of cover as a performance tool. it is just for looks.").
  - **A rule, not only a default:** the far eye must look right and meet its gate with fog off; fog never hides a flaw
    or saves work, and no ray stops early in it (the plan's "nothing relies on fog", in his words). The fog-line step
    is dropped. Builder retires one argument of hers that leaned on far land being fogged (the window's reach).
  - **Off by default from build 149,** its code kept, the Render page's Fog section turning it on; every report says
    whether fog was on, read from the look itself. The fog-off baseline (build 149): the spawn 0.70, the plain 0.94,
    the mountain top 1.59, high over the spawn 1.64, the sea 2.20 ms (nodding 99th 2.69), a player's run 2.28. Fog
    had cost 1 to 2%.
  - *Corrected in the open (Builder):* a mark's fog setting was printed in every report but never applied, so every
    flight and test picture before build 149 was fogged at every mark, the "clear day" mountain top included.
    Comparisons between builds stand; the labels were wrong. The window's picture check redone with fog truly off:
    about the same (the mountain top 7.2% of pixels differ, 0.5% by more than 8 of 255). David on the window without
    fog: "i know what it looks like without fog its fine."
  - *Note, so the words don't cross:* the Oracle's "haze or cover" for the skyline's soft edge isn't fog. "Haze" is
    today's way (rays gathering through part-solid cells), "cover" the alternative (how much of each square the land
    covers); neither hides anything.
  - **[OPEN], in the order he'll meet them:** one budget at the fork (its proposal: far 1.2 typical and 2 worst, near 1.5, finishing and the panel
    0.4, at least 0.9 left; read moving and nodding at the 99th); at the skyline's look, haze or cover; when lazy
    rays are built, far squares fixed to the world or to the screen.
- **[DECIDED 2026-11-12] Main memory tracked, and searched for free wins** (David, to Counsel, asking "do you think
  we are under-utilizing system ram? i think basically everyone will have at least 8gb", then: "tell builder to begin
  tracking ram use, and start looking for opportunities where using it is almost a free win or takes some heat/load
  off something else."). Every report and the session's log give the game's main memory beside its card memory, and
  Builder looks for uses of spare main memory that cost next to nothing or take load off the card or the processor
  (Counsel's first candidates: prepared far bricks kept ready so refilling the window is a plain copy, which would
  also spread its hitches; the tiles round the eye kept unpacked; room kept for the simulation). The bar, Counsel
  having proposed about 4 GB required: David, "i think we will be ok if we just keep our utilization somewhere
  between 4 and 8gb in the end." *Counsel's reading, his to correct:* the game's main memory ends up between 4 and
  8 GB, what it must have to run toward the low end, so an 8 GB machine still runs it. Why: his "what can *you* do
  for me?", for the laptop's 31 GB.
  - **The first picture** (build 138, Builder's `src/memory.rs`, Windows' own count): the game holds 2,826 MB
    private at the end, 1,036 MB of it in memory (the working set); 1,312 MB at 1.6 s before any far cells, 2,670 at
    61 s with all in, then about 20 MB more over the marks. The 1.3 GB at the start isn't placed yet (drivers and the
    card's staging copies are likely); Builder breaks it down stage by stage before pricing any free win.
- **[OPEN] Temporal reuse (TAA), and where it goes** (David, to Counsel: "what about TAA, is that in the
  testing/performance pipeline, towards the end, probably?"). Decided for old on 2026-11-08 with his rules (anything
  fast never takes from history; pixels with no history get rays first; edits and light changes drop history; near
  block faces stay crisp; Playdead's INSIDE talk as the reference), as a main candidate for "adding power" after the
  look is whole and inspected. The new plan doesn't name it yet, and the Oracle's design needs no reuse between
  frames. Put to the Oracle (question 13): after the eye is whole, or earlier as a lever if the far or near eye misses
  its gate.
- **[DECIDED 2026-11-12] Memory: a bar, and no growth with play** (David, to Builder): "if i wanted to create a larger
  landmass of continents, that it would scale linearly? a world twice as large would be twice as heavy on the card. i
  think that'd be okay, because anyone messing around with worldgen mods to get giant worlds probably has enthusiast
  hardware and can take some responsibility. but if you mean that, as the player plays the game for days, and weeks,
  and months, that the burden of memory continues to increase without limit, that's a serious concern." Builder: it's
  set by the world's size at load; only changes accumulate, and they're tiny (his build is a few KB). And: "i think a
  980ti is 4gb, so if we can keep the running frames well below that in memory in worst cases, i'd say we're fine."
  (Corrected in the open by Builder: the 980 Ti has 6 GB, the GTX 980 4 GB.) So the bar is his number: the game's
  card memory well below 4 GB in the worst case, on a world of the standard size. Why: a card of that class should
  still play; memory that grows with play would break that over time, so it mustn't.
- **[DECIDED 2026-11-12] Fast flight may creak; play must be solid** (David, to Builder): "if the game creaks at
  10,000mph that's fine. as long as its not aggregious and it's solid as a rock when i stop and act more like a
  player." Why: flying at speed is a developer's view; the player's pace is what has to hold.
- **[DECIDED 2026-11-12] Live settings, with a way back** (David, to Builder): "yes, please expose any interesting or
  valuable parameters that may be worth adjusting live, with default switches so i can get back to normal." The
  blend choice is one, on the Render page. And the panel: "the gui in the game still does that thing where i can start
  highlighting text in the panel, and it never lets go ... I don't think i need to highlight anything to copy it": text
  selection is turned off in the panels, and they always hear the mouse button let go.
  - **[LATER] Weighting kinds of pixels, for the polish** (David, the same day): "i think being able to weight
    different kinds of pixels may be a very cheap and useful tool for a lot of things, once we've got performance
    where we want it and can spend some time just polishing the look a bit." Nothing built for it now; it waits for
    the polish, after the speed work.
- **The Graphics Kit comes after the Registrar:** "we don't need the graphics kit to start. that can come later,
  and SHOULD come after the registrar, so we can copy the color control system over for it, and the way we work
  with HSL." **The editor is made again:** "i think we can do better doing it again, and make it more streamlined."
- **The near field: whichever is cheap, compact and looks right.** "im comfortable with nearfield rays if they're
  cheap, and if that writes elegant compact code that is easier to manage. im also comfortable with triangles for
  actors and other things, or, triangles for everything in the near field *if* it can look right and there are no
  jarring pops or seams." To the Oracle: "I had an idea for a while we had to raymarch everything, but i try to be
  open minded and be ready to part with brief technical convictions."
- **Honest**, David's definition: "honest means that it doesn't need to be blended into the near seam. the colors
  and shading are identical a priori. honest means that you can see everything that actually exists, even if its
  tiny." Far changes "don't need to be absolutely instant real time updates, they just need to be accurate ... we
  don't want any fakery." What must show far off: "lights from a distance, caves from a distance and the stuff
  inside of them." Thin things: "i just don't want things popping in and out. that's jarring. ideally the fence ...
  would just average into its background gradually as you walk backwards from it, until its just a faint
  brown/dark line in the distance." A felled tree shows far off "within a reasonable timeframe ... say a few
  seconds."
- **Light:** "the big sun shadows don't need to change smoothly. they can change every 30 seconds for all i care.
  and even near shadows may be fine as just coarse lighting; a person might only darken 2-3 blocks significantly
  and the adjacent blocks a little bit. as long as its realtime, in the local near field, so that if you're hiding
  behind a wall you will see a shadow approach as someone rounds that corner." The day's length doesn't shape it:
  "10, 15, 30 minute days won't make an appreciable difference to how we implement it." *Counsel:* the relight
  follows the sun's steps whatever the day; the step's size is a tuning number.
  - **[DECIDED 2026-11-11, experiment 4b] Light kept a cell at a time, for the far view.** On the far land lit from
    each cell's kept light against its own rays, with the sun moved (build 115): "honestly the difference is moot. it
    obviously changes a little, but i don't think i could tell the difference between the two in a blind test". So
    the rebuild keeps far light in the cells, relit as the sun steps. At eye level it saves little (the spawn 9%):
    most of light's cost there is the near faces', which the new eye looks up too, at experiment 6
    (`docs/experiments.md`, experiment 4b).
- **The enlarging stays crisp:** "crisp was always my preference. 'crisp' is in fact still 'smooth', and 'smooth'
  mostly just spread it out, which didn't seem as honest." That is the enlarging box (`look.rs:216`: crisp, smooth,
  smooth sharp at skylines). **[OPEN]** The Oracle's crisp or soft *cells* is another thing: whether the far march
  reads the one cell it's in or blends eight (today it blends). David to the Oracle: "I am unsure about crisp,
  we'll have to work it out, and probably just do a test of it." Its experiment 2 is that test.
  - **[DECIDED 2026-11-11, experiment 2] Blended.** On crisp (build 112): "it doesn't look awful, but, it doesn't
    look better ... sometimes it looked decent, other times a little janky." On crisp across and smooth in height
    (build 113): "eh, i don't prefer it. its better than ziggurat lods but, its not better than blended". So the
    rebuild's far cells are soft: 3D textures with a border, blended by the card's texture hardware. Crisp and the
    in-between stay as fallbacks for a card short of memory (`docs/experiments.md`, experiment 2).
- **The world made once, like a stone tablet:** "yeah, 4gb world file is no biggie. generating the world like a
  stone tablet seems like a reasonable idea." Stage magic while it's made: "we're willing to do a bit of
  distraction and stage magic with character/world creation, lore exposition, cool onscreen world gen toy versions
  to watch." The same seed making the same world: "its not really critical, i think" (the Oracle keeps it
  anyway). *Counsel:* "1 always equals 1" is then kept by one stored answer that everything reads, in place of
  the processor and the card agreeing to the bit. **[OPEN]** David to confirm that reading in his own words.
- **The dyno and the player:** "the dyno is meant to be demanding ... if we can make our dev inspection of the
  world smooth and fast, we'll be golden when it comes to the player experience."
- **Weaker cards:** "don't want to treat a 980ti as a floor, i just think that if you have one, you should be able
  to play this game, loosely speaking." And: "once we get back from zero to the dyno i'll set us up to use the
  igpu at least for a check." The laptop's second card is Intel's built-in graphics (Core Ultra 9 275HX). The
  laptop is "always plugged in, turbo, max fans."
- **Heights and water:** "0-1024 should be fine." And: "if we rebuild world gen we need to come up with a better
  way to manage water levels, i think, i had a hard time getting a mix of cliff and beach along water. and rivers
  were a beast to get right, so, may be a better way to do them, or arrange the world gen pipeline."
- **Rust stays:** "i chose rust because it seems like a powerful safe language with a lot of particularly good
  code and very little bad code written in it." **Backed up online:** "no, we should have it backed up online."

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
- **The laptop's target is at its own resolution** [David 2026-11-11]: 2560 × 1600, not 2160p ("yes i meant
  1600p, sorry about that"). The 980 Ti's stays 1080p.
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
    the speed deep dive has three parts, in order (every idea, ticked with its measured result, is in the
    checklist `docs/dyno.md`):
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
       - **Slots specialised by use** [David 2026-11-09, after the card reported 262,144 slots: "are there any
         ways to optimize by specializing slots into multiple channels?"]. This is block work: the same picture
         from a different layout.
         - **Hot and cold data apart.** Every step of the march needs only how solid a cell is; colour and the
           six face shares are needed only at a hit. Keeping the solid shares in their own tightly packed buffer
           means each step pulls one byte through the cache instead of a whole 16-byte cell, so a cache line
           holds 16 cells' worth instead of one.
         - **Read-only where possible.** Buffers the march only reads are bound read-only, which lets the driver
           use faster read paths.
         - **Small constants as uniforms**, not storage.
         - Combined with the texture-unit idea above.
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

       **Atomize the megakernel** [David 2026-11-09, after Nsight showed the march as one huge program the card
       can barely fill: 128 registers a pixel, about 14% of the card's pixels in flight, arithmetic at a quarter
       of peak, half of each group's lanes idle]: "the idea of a monolithic renderer was as misguided as the idea
       of two renderers and two scales, so lets go the other way. i believe the hologram enables atomized
       processing, so lets atomize the megakernal which apparently i was fated to repeat by not knowing history."
       So sight becomes stages, not one program: wavefront-style small kernels passing results through
       buffers (Laine, Karras and Aila, "Megakernels Considered Harmful", HPG 2013).
       - The march finds what each ray hit.
       - Light, shadow and sky run as passes of their own, over only the pixels that need them.
       - Then tiles sorted by type, each type run by its own lean program, so a batch's rays are alike.
       - Then the ray queue, if lanes are still idle.

       The store of averaged cells is what makes this possible: every stage reads the same store, independently.
       Each stage is flown with the twin and checked against the every-block truth, so the picture can't drift.
       Shrinking the one program further was found to be a lottery on the compiler's choices: the sky cones as
       one loop won, and compiling out the measuring views lost. Small kernels are far less exposed to that.
       *On the look-first order:* "working backwards from the result we want made the goal crystal clear."

       **Nothing relies on fog to perform** [David 2026-11-09]: "i kind of feel like 'rays end where fog is total'
       is such an obvious hit that we should save it for last, and see if it's necessary. i would prefer to tune for
       any latitude of vision, including very clear days, so it might be nice, but nothing should rely on fog in our
       renderer to perform." So the renderer is tuned for the clearest day. Ending rays in total fog comes last, if
       at all: a bonus on foggy days, never what the frame rate depends on.

       **The stages, with light per face, are the normal drawing** [David 2026-11-10, asked whether they should
       become the game's normal drawing now: "yes"]. His look at light per face near the eye was "merely slightly
       different, not better or worse" (build 103), and he had asked for it: "blocky and quantized lighting is
       good all around". Against the one-pass march as it ran, the typical frame is 6.5% slower and the worst
       frames much faster: the 95th -18.6%, the 99th -36.0% (the card's 17.8 ms down to 11.4), over the land
       -32/-44/-47%, the same on a clear day. Even beats fast. Both switches ("Draw in stages" and "Light per
       face near the eye") stay on the Render page as checks, on by default [David: "eh just leave em there until
       we need to clean up"]. The typical frame is won back in the far march, which both shapes share (dyno
       item 4).

       **The near ground's generator cut into stages too** [David 2026-11-10, asked whether the near-ground tile
       pass may be cut into stages with the same result to the bit: "yes ... as long as it looks right, which is
       something we know now. seems like the whole thing *can* be carved into a gpu shape."]. Nsight at build 104
       put `sight_grounds_pass` (the terrain generator on the card, working out the near ground's columns as the
       eye flies) at 255 registers, spilling to memory, holding a sixth of the card, and 15% of the card's time
       over the land. It's cut along the terrain pipeline's own stages, each handing only forward, as the march's
       are, or at least brought under 128 registers. What it works out doesn't change: the card's columns still
       match the processor's to the bit. In Builder's order of the register work: the texture units for the blend
       first, then the near walk and the far march as separate passes, then this. (The split was first until
       Nsight, the same day, read each half alone: the far march 121 registers of the whole march's 125, the
       near walk 80. Splitting frees the walk, but the far march's room is its own, mostly the eight-cell
       blend's unpacking and sums, which the texture units take out of the program.)
       *Tried the same day and dropped:* cut in two, then four (the maps, the parts, the uncarved ground, the
       carve), every stage matching the processor's columns to the bit, the stages lost to the single pass even
       sent to the card at once (the median +1.7 to +1.9%, round the peak +8 to +13%): the hand-overs, 14 numbers
       a column out and back, and the extra dispatches cost more than the lighter stages saved, while the parts
       stage still held 255. The game stays on the single pass; the patch and the to-the-bit test are kept. His
       yes stands if it's tried again, and the ridges apart from the peaks is where the room is (dyno.md, 7622168).

       **A drop is for its context, not for good** [David 2026-11-10]: "just note that some fails may have been
       contextual and that context may change as we work through the graphics engine trying to get it into the card
       properly, and that we will be better equipped going forward if we dont get it this time". So a step that
       lost is written down with what it lost to (the sky-light map to the one big program's room, which is gone
       now; the near ground's stages to their hand-overs), and tried again when that changes. And if this engine
       doesn't reach the card's speed, what the dyno taught goes into the next one from its first build.

       **Terrain light may lag; actors' light may not** [David 2026-11-10, asked whether a block face's light
       could be kept from frame to frame and lag up to about a quarter-second when the sun moves or new ground
       lands (an outside audit's lead: light per face kept, not worked out afresh each frame): "i think quarter
       second light wont really be visible with coarse lit blocks anways, especially if near actors work faster.
       terrain light almost needs zero dynamism for gameplay"]. So the sun's and the sky's light on the terrain
       may be kept and brought up to date a little at a time. Light that moves with actors (moving torches,
       light from effects) stays prompt, every frame. Priced with a twin first, then behind a Render-page switch
       for his look.
       *Built the same day and dropped* (the rule stands): light per face kept, the same to the pixel as lit
       afresh, lost in flight (the median +10.8% and +0.6%, the quick leg +18 to +20%, the land level). What it
       lost to: the dyno flies a hundred blocks a second and more, so the near faces are mostly new each frame and
       lit anyway, and the table's bookkeeping cost more than the relighting saved. It may pay walking or standing,
       which the dyno doesn't fly; the patch is kept (dyno.md, 3826eb8).
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
- **[DECIDED 2026-11-14] No 980 Ti machine bought; players' logs check it** (David, to Counsel, after looking at
  workstations with a 980 Ti, one a Precision 5820 at about $650): "honestly i think if we do our very best on the
  graphics engine, it doesn't really matter does it? it has to look at least so good, and we can make it as fast as
  we can, and that's really all we can do, beyond adding lower settings in the end for people to turn on if they
  need to." Counsel: it already isn't a floor to design to; what only the real card shows is whether it runs at all,
  surprises of an older card, and which lower settings help. David: "i think i'll be able to find some people to try
  it out and send me a performance log by the time its important". So the pre-alpha's players (above) are the
  980 Ti's check; till then it's weighed by reasoning and the library's default limits. The plan's targets say so.

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

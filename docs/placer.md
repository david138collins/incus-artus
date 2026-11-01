# The placer (Builder, 2026-10-31)

**Status: go-ahead given** (David, 2026-10-31: "begin on it, and just proceed cautiously").
- His answers to the options are in section 10.
- What the placer has to grow into is in section 11.
- The first version's stops are in section 12.
  - **Stop 1 (the field view) is built.** David looked: "it looks ok so far".
  - **Stop 2 (the depth sweep) is built**, waiting for David to look.

David asked for these through Counsel: "the placer and quartility seem the most critical. but im not
sure how to observe the placer without minerology. i cant really think why it would need its own
panel, its pretty infra but id like to see some options about how it is done and implemented. it
would probably only be seen in the multiple systems that use it. and there will be quite a few i can
foresee."

**His requirements** (`docs/worldgen.md`, "Mineralogy needs a system for placing things"):
- add blocks from the library;
- make new distributors or layers;
- project them from a wide selection of fields;
- not hard-coded: "a wrapper to register distributions and locations or fields... able to call up
  almost any previously defined feature";
- shared by mineralogy, trees (the Registrar's blueprints) and Ecology's plants and animals, with more
  users to come.

**In one line:** **fields** (every number a stage makes, registered by name) are read by
**distributors** (David's layers, each choosing what goes where), through **one shared panel widget**
that every user embeds. Mineralogy is the first user, so it's where the placer is seen.

Each section below gives the options and my pick. The questions for David are at the end.

---

## 1. What exists today

**Today's mineralogy** (`src/world/mineralogy.rs`, about 120 lines of rules) is hand-written code:
- **It reads:** the temperature and rain (blended between the climate map's 32-block pixels, the
  temperature cooled by height) and the ground's height. A little noise makes zone edges wander.
- **It decides:** first rule wins.
  1. snowy ground where it's cold enough;
  2. gabbro above a height, stone above a lower one;
  3. sand on beaches, and in hot, dry places;
  4. dry grass where it's dry;
  5. grass.
  Each has a fixed layer under it (dirt, or sand) for so many blocks, then stone. The sea floor is
  sand in the shallows and gravel deeper. River and lake beds get gravel (in `terrain.rs`, apart from
  mineralogy).
- **Its blocks** are nine named slots in code (`TerrainBlocks`: incus:stone, incus:sand...). A rule
  whose block isn't registered is skipped.
- **It runs** per column, in three places that must agree: the near chunks, the far view's tiles, and
  the World Compiler's close-up.

**The numbers a world already has**, the fields the placer could offer on day one:

| Made by | Field | Kind | Handed to the world today? |
|---|---|---|---|
| Landmass | the bake's height, coast (land or sea), tilt, squeeze (crushed or stretched), push (way, strength), distance to the coast | map, 32-block pixels | yes |
| Topography | distance to the mountains' points | map | yes |
| Topography | the mountains' points | points | yes |
| Topography (the Interpreter's recipe) | the ground's height at a block, and how much each layer added there (rise, ridgelines, peaks, summit crags, crags, hills) | per block | worked out at each block |
| Climate, Rivers | the flow, lake depth, river rank | map, 32-block patches | no (compiler only) |
| Climate, Rivers | distance to the nearest river, its width and water across, under a river's or lake's water | per block | yes (rivers.txt) |
| Climate, Rivers | springs, mouths | points | no |
| Climate, weather | temperature (cooled by height), rain | map | yes |
| Climate, weather | wind, rain shadow, facing the wind, far from the sea | map | no (compiler only) |
| Any | slope, height above the sea, depth below the surface, noise (seeded, any size) | per block, worked out | can be |

---

## 2. Fields: how a stage registers what it makes

Every field needs:
- a **name** (`climate/temperature`), and the name David sees ("temperature");
- a **range and unit** (−1 frozen to +1 hottest; blocks), so pickers and curves know their ends;
- the **stage that makes it** (Landmass, Topography, Climate: Rivers, Climate: weather, Mineralogy,
  Ecology...), so the linear rule can be checked;
- its **kind**: a map (one value per 32-block pixel, blended), per block (worked out at each block or
  column), 3D (depth, or noise in three dimensions), points (peaks, springs), or lines (rivers).

**A distributor only ever reads one number at a spot.** Points and lines become numbers through
**derived fields**: distance to the nearest one, its size there, or how many lie within a reach. So
"distance to a spring" and "a river's water across" are ordinary fields.

**Options:**
- **A. A fixed list in code**, one `match` arm per field. Fast and simple, but every field is code,
  and the list doesn't describe itself: no names, ranges or stages for the panel.
- **B. A live registry:** each stage registers its fields as it runs, as name, information and a
  function to read it. Very open, but each read is an indirect call, at every column, in the near
  chunks and the far tiles alike, which costs speed. And a field could appear or vanish with the
  settings.
- **C. A table in code that describes itself** (my pick): one row per field (name, the name David
  sees, range, unit, stage, kind), and one function that reads any of them by number.
  - **Distributors name fields in data**, so they're data, not code. An unknown name is reported when
    loading, not crashed on.
  - **The linear rule is a check against the table:** a field from a stage after the user's is shown
    greyed out ("made after Mineralogy"), never hidden, and can't be picked.
  - **Adding a field** is one row and one line that reads it. New fields come from new stages,
    which are code anyway.

**Within one user, earlier layers are fields too.** For example, mineralogy's "rock type" layer could
be read by its "soil" layer below it. That's "layers mix their numbers, not their colours", and it's
still linear: a layer reads the ones above it, never below.

**Fields made only in the compiler** (the wind, the flow, springs) can be read by things decided
over the whole map (section 5). To be read at a block, a field has to be handed to the world with its
other maps, as temperature and rain are at "Render World and Spawn". That's one more map in the
world's save for each field handed over, so they're handed over only when a distributor uses them.

---

## 3. Deciding: how a distributor chooses

**Options:**
- **A. First rule wins**, as today's mineralogy. Easy to read and predictable, with hard edges (the
  wander noise softens them).
- **B. Weights or likelihoods:** every candidate gets a score at each spot, the best wins, or one is
  drawn at random by its score. This suits Ecology: an oak likes this much rain and that much warmth,
  and borders blend.
- **C. Stacked layers that mix numbers:** each layer takes a field, puts it through a curve, and mixes
  it into a running number by a mode (multiply, add, the lower, the higher, mask). This is the
  Interpreter's own way, and David's "layers mix their numbers".

**My pick: C as the frame, with A and B as the chooser's modes, all in one model.**
- **A distributor** has a list of **candidates** (blocks, or later blueprints).
- **Each candidate's suitability** is a small stack of terms, mixed by a mode. Multiply means "and":
  every term must hold. There are two kinds of term:
  - **a number through a curve** (0 to 1). A term can add noise to its field before the curve:
    that's today's wander, for any field;
  - **a kind in a set:** for fields that name a kind rather than a number, such as rock type, or
    which candidate an earlier layer chose. For example, "the host rock is limestone or marble".
- **Curves come in three shapes:**
  - **a step** ("below 0.3"): today's rules;
  - **a band**, by its middle and spread ("around 40 deep, give or take 15"), which suits ores by
    depth;
  - **a free curve** (the spline box).
- **The distributor's chooser** then picks:
  - **first that fits:** in order, the first candidate whose suitability passes a threshold (today's
    mineralogy, exactly);
  - **the best fit:** the highest suitability;
  - **by chance:** at random, weighted by suitability, seeded by the spot, so it's the same every
    time (for Ecology's scattering).
- **One model with three chooser modes**, not three systems. Today's rules convert to "first that
  fits" with step curves.

---

## 4. What's placed

1. **Column profiles** (the first version): a surface block, then the blocks under it, each to a
   depth, then what's below. That's today's mineralogy: grass, 3 of dirt, stone. Soils come here.
2. **3D blocks:** rock in bands by depth or by height (strata), and inclusions (ores, by noise in 3D
   and chance). This is the "material × inclusion" decision, and caves need it. It comes with the
   strata work.
3. **Blueprints at points, with spacing:** trees, boulders, ruins.
   - Density comes from suitability.
   - Spacing is a least distance between them, scattered so the gap is kept even across chunk
     edges.
   - A check of the ground under each blueprint's footprint (slope, water).
   - It comes with the asset factory and Ecology.
4. **Decorations:** small things on the surface (tufts, flowers, pebbles). These are blocks or items,
   and wait for their art.
5. **Ranges and areas:** a distributor whose output is a new field for later stages, such as forest
   cover, an animal's range, or a mine site. This is how Ecology's ranges and mineralogy's mine sites
   (`docs/decisions.md`) would come out.

**My pick:** design the format for all five now, so nothing has to be redone, and build only 1 (column
profiles) for the first version, with mineralogy. The others arrive with their first users.
- **What a candidate puts down** is one of three kinds in the format from the start: a block (now),
  an inclusion over the block it's in (ores, with the "material × inclusion" decision), or points
  (pipes, craters, trees). Only blocks are built first.
- **A distributor's result can be read by later layers:** which candidate it chose (a kind), and,
  for points, the distance to the nearest and how many lie within a reach (numbers).

---

## 5. Where it runs

- **A. Over the whole map in the compiler,** cached. That's cheap to preview, but only at the map's
  resolution (32-block pixels). A world is far too big to decide every block ahead of time.
- **B. At each block, in the Interpreter,** as mineralogy does now. Exact, made only where chunks are
  made, nothing stored, and the near and far views agree because they call the same thing. It costs
  time at every column, near and far.
- **C. Each user's kind decides** (my pick):
  - **Column profiles and 3D blocks** run at each block (B), as mineralogy does now.
  - **Blueprints** are scattered region by region, from the seed, with their density read from
    fields. Spacing needs neighbours, so each region looks a little past its edges.
  - **Ranges and areas** are worked out over the whole map in the compiler (A), and handed to the
    world as maps.
  - **Previews** work at both scales. Over the map, the winner at each pixel. In the close-up,
    exactly, since the close-up already asks mineralogy at every pixel.

**Speed:** each distributor is turned into a flat list of steps when it's loaded or changed, not
worked out from its description at every column. Each field is read once per column, and "first that
fits" stops at the first fit. The soak test measures it against today's mineralogy before it's
accepted.

---

## 6. Defined as data

**Options:**
- **A. In the compiler's presets and the world's recipe**, as all of mineralogy's settings are now.
  Ctrl+S saves them with everything else, and a world keeps the look it was made with. But they're
  shared only as part of a whole preset.
- **B. Data files**, like blocks (`assets/data/<mod>/placers/*.toml`). Moddable on their own, but
  separate from the presets, so saving them is another step, and a world needs its own copy to keep
  its look.
- **C. A with import and export** (my pick): distributors live in the preset and the world's recipe,
  as now, and any distributor can be exported to a file and imported from one, for sharing and mods.
  The format is the same either way, so moving to B later is cheap if mods need it.

**Adding things:**
- **A distributor:** in the panel (add, then pick candidates from the block library).
- **A block:** in the Registrar, as now. It then appears in the library picker.
- **A field:** code, one row in the table (section 2).
- **A curve shape or chooser mode:** code, rarely.

---

## 7. How David sees and steers it: one shared widget

**The placer has no panel of its own.** It's one widget that each user's panel embeds: Mineralogy's
first, then the Blueprints tab's and Ecology's.

- **Candidates:** a list, dragged to reorder, added from the block library with their pictures. A
  block that isn't registered shows greyed, named, and is skipped.
- **Each candidate's terms:** a field picker (grouped by stage, later stages greyed out as "made
  after Mineralogy"), the curve (the spline box the Climate tab already uses), and the mode.
- **The chooser:** first that fits, the best fit, or by chance.
- **Previews:**
  - **Over the map:** the winner at each pixel, in its block's colour.
  - **One term:** a single term's result in grey, to see what it's doing.
  - **In the close-up:** the real blocks, as now.
- **"Why is this sand?"**: Under the mouse lists the fields' values there and each candidate's
  suitability. This is how the placer is observed. It shows why every block is what it is.

**Mineralogy's first version on it** would look like today, rebuilt as three distributors:
- **"The surface"** (first that fits): snowy ground, gabbro, stone, beach sand, desert sand, dry grass,
  grass, each with today's conditions as terms and its layer under it (dirt or sand, to its depth);
- **"The sea floor":** sand in the shallows, gravel deeper;
- **"River and lake beds":** gravel (moved in from `terrain.rs`).

**It should give the same world to the byte**, checked by comparing chunks made both ways (as the
rivers were today). From there David adds blocks, layers and fields of his own.

---

## 8. Sizes and risks

**First version** (fields, distributors, column profiles, the widget, mineralogy rebuilt on it):

| Part | About (lines) |
|---|---|
| The field table and reading fields at a column | 250 |
| Distributors, candidates, terms, choosers, saving them, turning them into steps | 300 |
| Room for ores in the format (term kinds, curve shapes, what's put down, results read later: section 9) | 100 |
| The shared widget (candidates, block picker, field picker, curves, chooser) | 400 |
| Previews (over the map, one term, why under the mouse) | 200 |
| Mineralogy rebuilt on it, with the same-world test | 200 |
| **Total** | **about 1,450**, in 3 or 4 stops for David |

**Later, with their first users:**
- blueprints with spacing: about 500;
- 3D strata and inclusions: about 300;
- areas handed on as maps: about 150.

**Risks:**
- **Speed:** every column, near and far, runs the distributors. The flat steps and reading each field
  once keep that close to today's hand-written rules, and the soak test checks it.
- **The near and far views must agree:** they call the same function, as now. Never two copies.
- **Saved worlds:** a world made before the change keeps today's mineralogy settings in its recipe.
  Unchanged chunks are made again from the recipe, so after the change they'd come out in the new
  defaults. Two choices for David:
  - convert old settings into the three distributors once, when loading (about 40 lines);
  - or accept that old worlds' untouched ground changes. Old presets get no shims either way, by
    his rule.
- **Field names are a promise:** a mod's distributor names fields, so once named, a field keeps its
  name.
- **Fields handed to the world** add a map each to the save, and only when used.

---

## 9. Room for ores (Counsel's input, 2026-10-31)

**Where the ores come from:** David asked for a cross-reference of the ores in Minecraft, Vintage
Story and Dwarf Fortress, ranked by a reasonable mean depth (`vision/ores-by-depth.md`; about 130
blocks of crust). Ores are the "3D blocks" kind (section 4), which comes after column profiles. The
first version doesn't build them, but its format must not need redoing when they come.

| Ores will ask for | Where the format has room |
|---|---|
| **Depth below the surface, with a mean and a spread** | a per-block field, "depth below the surface", and a band curve (section 3) |
| **…as a share of the crust**, since how deep the world goes is open | the same field also offered as a share of the crust (0 at the surface, 1 at its bottom), measured against one world setting, "the crust's depth" (130 to start). When the world's depth is decided, ores keep their places in it. |
| **Rock type** (most ores follow their host rock) | rock type is a mineralogy layer; later layers read which rock it chose, as a kind in a set ("in limestone or marble") |
| **Child deposits** (gold inside quartz, silver inside galena, sylvite inside salt) | a later layer whose term is "the earlier layer chose quartz", placing gold there by chance. It's linear, since the child comes after its parent. |
| **Point fields** (kimberlite pipes, impact craters) | a distributor that puts down points; later layers read the distance to the nearest (a pipe is a narrow distance across, over a range of depth) |
| **Ties to the land:** rivers (placers), deserts (evaporites), wetlands (bog iron), the tropics (bauxite), crush (lode gold) | fields in the table already: under a river's water, its water across, distance to a river, lake depth, rain, temperature, slope, height, squeeze (crush) |
| **Ores as inclusions in their rock** ("material × inclusion") | a candidate that puts down an inclusion over the block it's in, rather than a block of its own |

**Four additions to the first version's format,** all small, all built in from the start:
1. terms of two kinds (a number through a curve, a kind in a set);
2. curves of three shapes (step, band, free);
3. what a candidate puts down (block, inclusion, points), with only blocks working at first;
4. a distributor's result readable by later layers, as a kind, or as distances for points.

Together they add about 100 lines to the first version.

**Speed for ores:** they're checked at every block underground, not only at the surface, so they
must stay cheap. A depth term tells the evaluator the depth window it can hold in, so it skips the
column outside it. They're made only in near chunks: the far view sees only the surface.

**Balance** (David: "making sure that areas are diverse, but viable"): a map readout, with the ores,
of the distance from each spot to the nearest fuel, copper, tin, iron, flux and salt. This needs no
format change: the Registrar's block tags already exist ("resource/copper" and so on), and the readout
groups the winners by tag. Gaps then show on the map as places that aren't viable to start from.

---

## 10. Questions for David, and his answers (2026-10-31, through Counsel)

1. **Deciding:** one model, with stacked number layers making each candidate's suitability, and the
   chooser as first that fits, the best fit, or by chance (section 3)?
   - **Yes.**
2. **What's built first:** column profiles only, with the format ready for blueprints, 3D and areas
   (section 4)?
   - **Yes, column profiles first.**
3. **Where it lives:** in the presets and the world's recipe, with import and export of single
   distributors (section 6)? Or data files like blocks?
   - **Presets, with import and export.**
4. **Old worlds:** convert their mineralogy settings once when loading, or let their untouched ground
   change (section 8)?
   - **Let it change.** No converter (his standing rule on old saves).
5. **Seeing it:** is the shared widget, with "why is this sand?" under the mouse, the way you'd want to
   observe it (section 7)?
   - **Yes, and two views besides:**
     - **the field view:** "switch the screen to black with lines on the map, with shading showing
       the field i'm *making* from another field";
     - **the depth sweep:** "scroll through the z layers with the wheel or a scroll bar that can
       select ranges or individual levels, in order to sweep like lidar and see where the minerals,
       rocks, and ores are going. same for soils and sands".
6. **The first fields to offer:** everything in the table in section 1, or a smaller set to start?
   - **The table**, "especially if i can add or subtract them with blend modes".

---

## 11. What it has to grow into (David's go-ahead)

David: "the placer needs to be able to access more than just fields. it'll place towns, and presumably
also place the buildings in the town, which has to grow procedurally like jigsaw pieces and also
through decisions by the npc. it'll need to be able to access fields from tectonics, it'll need to be
able to place things based on other placed things, it'll need to be able to select entire sections of
the world, generate its own fields from any points we already have like 'river end' or 'peak'. its
going to need to be very powerful and flexible, and be able to handle things both at world creation
and going forward possibly as the game will be able to place things for little reason at any time if
necessary. as long as we all understand that we might be forgetting something it has to do, i think
we are good to begin on it, and just proceed cautiously."

**The first version builds little of this, but its format has a place for each,** and every list
below is open to kinds nobody has thought of yet. In the format, each is a kind with a name. A kind an
older game doesn't know is shown greyed and named, never crashed on.

**What a layer reads** (its source):
- **Built in stop 1:**
  - a field from the table, the Tectonic Sim's included (the bake's height, tilt, squeeze and push);
  - seeded noise;
  - another made field above it;
  - the result so far (for a curve over everything below it);
  - a number.
- **Built in stop 3:** which candidate an earlier distributor chose (a kind, such as rock type).
- **Later:**
  - **Fields made from lists of points or lines:** the distance to the nearest, how many lie within
    a reach, or the nearest one's size. The lists are any we have: river ends, peaks, springs,
    mouths, junctions, falls, craters, and **things already placed** (a distributor's own points, or,
    in play, whatever has been placed since). Two of these are table fields from stop 1: the distance
    to a river and to the mountains' points.
  - **Regions:** a whole section of the world, chosen by a field (every spot where it's above a
    level), by a kind (one plate, one basin), or drawn by hand.
  - **The Tectonic Sim's kinds:** which plate a spot is on.

**What a candidate puts down:**
- **Built in stop 3:** a block, with the blocks under it (a column profile).
- **Later:**
  - an inclusion in the rock already there (ores);
  - points with spacing (trees, boulders, craters, towns);
  - **pieces joined at hooks**, for towns and buildings that grow like jigsaw pieces. The placer
    answers "where does this piece fit, at which hook". Civilization's NPCs decide what grows next.
  - an area handed on to later stages as a field (forest cover, an animal's range, a mine site).

**When it runs:**
- **Built:** at world creation, in the compiler (over the map) and the Interpreter (at each block).
- **Later, at any time in play.** A placement in play comes after everything, so it can read
  everything that exists, placed things included: the linear rule still holds. It's saved as an edit,
  like a build, and seeded, so it can be repeated.

---

## 12. The first version's stops

Mineralogy is the first user. **Each stop ends with something for David to look at, and I stop there
and wait for his word.** About 2,400 lines in all. That's more than section 8's 1,450, for the two
views and the wider format.

### Stop 1: the field view (about 900 lines)

**Built:**
- **The field table:** the fields a world has, each with its name, stage, unit and range (section 1's
  table):
  - **Tectonics:** the bake's height, tilt, squeeze, push strength and push way;
  - **Landmass:** distance to the coast;
  - **Topography:** distance to the mountains' points, how much of it is mountains, and the rise,
    edge rise, ridgelines and peaks there;
  - **the ground:** its height, its slope, and, per block, the block's Z, its depth below the
    surface, and that depth as a share of the crust;
  - **Rivers:** distance to a river, a river's water across, and water over the ground;
  - **Climate:** temperature and rain.
- **Made fields:** a stack of layers, run top to bottom.
  - **Each layer** is a source through a curve, mixed into the result so far by a mode, at a strength.
  - **The curve** is one of: as it is, a step, a band, or free (the spline box).
  - **The mode** is one of: normal, add, subtract, multiply, the lower, the higher.
  - So "copper by a band of depth, minus wet ground" is two layers.

**Where:** a new **Fields** section at the top of the Mineralogy panel holds the made fields and the
field view's switches. The field under the mouse shows in the right-hand panel. (Easy to move if
David wants it elsewhere.)

**What David sees:**
- **The map turns black,** with lines on it (the coast, the rivers and lakes, and contours every so
  many blocks), and the field being made shaded in it, white where it's highest.
- **Any step:** the result at any layer, or one layer alone.
- **Under the mouse:** each layer's value and the result so far.
- **Depth fields are seen at a level:**
  - the surface, a Z, or a depth below the surface;
  - one level, or a range (a range shows the most anywhere in it);
  - set by a slider, or Ctrl + wheel over the map.

**Not yet:** nothing in the world changes. Made fields are saved with the preset, and used from stop 3.

**I stop here,** for David to make a few fields and say what's missing.

**[BUILT 2026-10-31]** As planned, with these details:
- **The code:**
  - `src/world/placer/fields.rs`: the table of 22 fields;
  - `src/world/placer/mod.rs`: made fields and the `Program` that reads them;
  - `src/worldgen/placer_view.rs`: the Fields section, the field view and the readout.
- **The settings:** made fields are in the preset as Mineralogy's `fields`, next to the crust's depth
  (`crust`, 130), which is on the panel. Presets from before read with no made fields.
- **Distance to a river** is measured from the water's edge (0 in it), out to 2,000 blocks, through
  a coarser grid of the rivers made the first time it's asked. Lakes aren't counted yet.
- **A river's water across** is known only within its banks' reach. It's 0 further away.
- **Slope** is measured between neighbouring samples, so zoomed out it's measured over more blocks.
  (From stop 3, the near ground measures it block to block.)
- **Black and white** are the least and most the field can be, worked out down the stack from each
  field's range, not from what's on screen. So panning doesn't change the shading.
- **A range of levels** is looked at in at most 32 levels, spread evenly through it.
- **Lines:** the coast is pale blue, the rivers' and lakes' edges are blue, and the contours are amber.
  A contour is drawn every 16 blocks to start, and each kind of line has its own switch.
- **Tests:**
  - curves, modes and ranges;
  - made fields reading those above them, and not those below;
  - unknown fields and kinds;
  - saving with a preset;
  - the distance to a river;
  - the picture (black, white and the coast);
  - levels and ranges.

### Stop 2: the depth sweep (about 350 lines)

- **A bar along the map's right edge sets the level:**
  - drag its two handles apart for a range, or together for one level;
  - the wheel over it steps a level (Shift: 10 at a time);
  - by Z, or by depth below the surface.
  - The same bar sets the field view's level.
- **The blocks view:** at that level, which block is there across the map, in its colour. For now
  that's today's mineralogy: the surface, the dirt and sand under it, and stone.
- **For a range,** each spot shows the first block from the top that isn't see-through:
  - air and water always are;
  - stone is too, by a switch ("see through stone").
  - So soils and sands stand out like a lidar sweep, and later the rocks and ores.
- **Under the mouse:** the column's blocks from the surface down.

**I stop here.**

**[BUILT 2026-10-31]** As planned, with these details:
- **What the map shows** is now a row at the top of the Fields section: "the world", "the field view"
  (greyed until there's a field) or "the depth sweep". It replaces stop 1's "field view" checkbox.
- **The bar:**
  - two handles, the left for the level's one end and the right for its other, so they can be pulled
    apart from one level;
  - dragging between them moves the range;
  - a click brings the nearer end there;
  - the wheel over the bar steps it, by 10 with Shift; Ctrl + the wheel over the map still steps it
    too;
  - the label above it ("by depth" or "by Z") switches between the two;
  - the sea's level is marked on the Z bar;
  - it's greyed while the map shows the world.
  - The panel's level sliders are the same level, and stay.
- **The blocks** are drawn in their far colours, flat (unlit), with the same lines as the field view.
  Where a range sees only water it's dark blue; where it sees only air (or what's seen through),
  it's black.
- **"See through stone"** sees through stone and gabbro, the two rocks so far. It's on the panel
  under the lines' switches, greyed outside the sweep.
- **Under the mouse:** what the sweep sees there, then the column's blocks, a line for each run:
  water, the surface, the layers under it, then "stone: Z … and down".
- **One rule fills a column:** the world's chunks and the sweep both use `Layers::block_at` (with the
  same ground rounding, `whole_column`, and the same river beds, `Layers::bedded`, now shared from
  `world/terrain.rs`).
  - So when stop 3 changes what's placed, the sweep shows it with no change of its own.
- **Tests:**
  - what the sweep sees at a level, in a range, through stone, through the sea and in a river;
  - the column's runs;
  - six columns of a test map compared block for block with the world's own chunks.

**[BUILT 2026-10-31]** Also from David's look at stop 1: a field change no longer makes the climate
again. The Climate stage now reads the recipe without mineralogy, so it remakes only when the ground's
shape changes (502a3a7).

### Stop 3: mineralogy on the placer (about 900 lines)

- **Distributors:**
  - **candidates** from the block library, with their pictures. A block that isn't registered shows
    greyed and named, and is skipped.
  - **Each candidate** has its suitability (a stack, like a made field) and its column profile: the
    top block, then blocks under it, each to a depth, then what's below.
  - **The chooser:** first that fits, the best fit, or by chance (seeded by the spot).
  - **Later layers can read what's chosen,** as a kind in a set ("the rock is limestone or marble").
- **Today's mineralogy rebuilt as three distributors:** the surface, the sea floor, and river and
  lake beds.
  - Their starting numbers are David's own (his 15:20 preset: dry grass below rain 0.21, wander 0.3
    in patches of 512 and so on), and they must give his world to the block. A test compares chunks
    made both ways.
  - Today's Mineralogy sliders go, and their numbers become the distributors' curves.
  - Other presets' and old worlds' mineralogy goes back to these starting numbers (no converter).
- **One function** for the near ground, the far view and the close-up, as now.
- **"Why is this sand?":** under the mouse, each candidate's suitability and the chooser's pick.
- **Speed:** the soak test against today's mineralogy.

**I stop here,** for David to add blocks and layers of his own.

### Stop 4: sharing, and the docs (about 250 lines)

- **Export** a distributor or a made field to a file, and **import** one.
- The docs brought up to date, with what's next.

**I stop here.** The placer's first version is then done enough to go on to the next thing.

# The placer: options (Builder, 2026-10-31, for David to choose)

**Status: options only. Nothing is built or decided.**

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

## 10. Questions for David

1. **Deciding:** one model, with stacked number layers making each candidate's suitability, and the
   chooser as first that fits, the best fit, or by chance (section 3)?
2. **What's built first:** column profiles only, with the format ready for blueprints, 3D and areas
   (section 4)?
3. **Where it lives:** in the presets and the world's recipe, with import and export of single
   distributors (section 6)? Or data files like blocks?
4. **Old worlds:** convert their mineralogy settings once when loading, or let their untouched ground
   change (section 8)?
5. **Seeing it:** is the shared widget, with "why is this sand?" under the mouse, the way you'd want to
   observe it (section 7)?
6. **The first fields to offer:** everything in the table in section 1, or a smaller set to start?

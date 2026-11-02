# Caves: what they touch, options and order (Builder, 2026-11-01, for David to choose)

David chose caves next, before trees (through Counsel, 2026-11-01): "i think that the caves are the
scariest thing coming up and we should think about how we start going about it. trees will be fun
and easy, so we should get on to caves, somehow, and then we can add the blueprint factory and rough
out an ecology tab."

This is an assessment, written in the same shape as `docs/placer.md` and `docs/quartile.md`. **Nothing
here is built.** David reads it with Counsel first.

**What's decided and on file** (not repeated here, only used):
- `docs/decisions.md`:
  - the Metaphysics Engine: projected code uses whole numbers, with no sin, cos or pow;
  - "Light is precious": caves are pitch black unless lit, and daylight reaches only a little way
    in at the mouths.
- `docs/worldgen.md`, "Caves, cliffs and carving":
  - caves are formed deliberately, plan then carve, as the rivers are;
  - the column recipe stays the base, and caves are taken from it;
  - cave biomes are kits chosen by cause;
  - the waterfall cave is the first kind;
  - cave mouths are lit and seen from far away;
  - what lives down there;
  - holes in the canvas;
  - respawning at the same spot, for comparing screenshots.
- `vision/watch-list.md`, items 17 and 17b: the far view's one contract for things that break the
  surface, and the probe with two candidates tried on one cave mouth.

**The short version:**
- **First, four small probes, each something David can see:**
  1. a cave made by hand that you can walk into;
  2. what it costs to draw;
  3. darkness;
  4. the mouth seen from far away.
- **Then the first real kind: the waterfall cave.** It needs a river that ends with nowhere to go,
  and that isn't built yet (water rework stage 6, petering out). So petering out comes first, inside
  that stop.
- **What scares me most** (section 6) is the look, and light. The look is a tuning problem like the
  rivers'. Light is the most tangled technical part, because digging and torches will need it too.

---

## 1. What exists today

- **The column recipe** (`world/terrain.rs`, `CompiledMap::chunk`):
  - Every column has one ground height. Below it are the surface's layers (mineralogy), then stone.
  - Above it is water up to sea level or a river's level, then air.
  - Each chunk is made on its own, on any thread, from the map.
- **Two shortcuts make the solid underground free:**
  - **When making chunks:** a section wholly below the chunk's lowest ground and its layers is one
    `Uniform(stone)`, with no per-block work.
  - **When meshing:** a uniform solid section walled in by solid neighbours gets no mesh at all.
  - Caves break both shortcuts, but only in the sections they pass through.
- **Light underground** (`terrain.wesl`, `ground_light`): faces 12 to 24 blocks below the surface fade
  to *plain light*, fully lit. So a cave dug today is as bright as a meadow. "Light is precious"
  reverses that.
- **There is no light per block:**
  - Light comes from the light map (sun shadows, valleys, relief) and the near window (the lie of the
    land, the feet of steps). Both are worked out from the ground's heights, not from blocks.
  - The vertex has **4 spare bits** (word 0 uses 28 of 32), enough for 16 levels of light at each
    corner.
- **Culling:** Bevy skips sections outside the camera's view, and sections past the render distance
  are hidden. **Nothing hides what's behind rock.** A cave inside a mountain you're looking at is
  drawn in full.
- **The far view** (`render/distant.rs`): smooth sheets with one height per column. It has no idea
  of anything under an overhang. Past the real chunks, a cave mouth doesn't exist.
- **The rivers are built** (water rework, through stage 7). **Petering out isn't** (stage 6, about
  20 lines and a slider). So every river reaches a lake or the sea, and none ends with "nowhere to
  go", which is where the waterfall cave starts.
- **Rock:**
  - Mineralogy gives each column a stack under its surface, with stone below that.
  - **There are no rock layers in 3D yet.** A cave's walls would be plain stone, with the surface's
    layers where a cave comes near the top.
- **Noise:** the whole-number noise is 2D only (`worldgen/noise.rs`, `whole_perlin`). Walls that aren't
  pipes need a 3D version.
- **Depth:**
  - The world runs from `WORLD_BOTTOM` (0) to 1,024.
  - Sea level is 128, so under the lowlands there are about 128 blocks of rock.
  - Under the mountains there are up to 900.

## 2. What it touches

| Part | What changes |
|---|---|
| The plan (new, `world/caves.rs`) | Cave shapes and a look-up for "am I inside a cave?" |
| Chunk making (`world/terrain.rs`) | Taking caves from the column. The stone shortcut is kept wherever no cave is near. |
| Noise (`worldgen/noise.rs`) | Whole-number 3D noise, for the walls |
| Meshing (`render/meshing.rs`, `mesher.rs`) | Light at each corner (stop 3), and maybe which sections can see each other (stop 2) |
| Shader (`terrain.wesl`) | Darkness replaces plain light; mouth light; a test lantern |
| The far view (`render/distant.rs`) | Mouths far away (stop 4, the 17b probe) |
| World Compiler | The Caves section (greyed today, on the Mineralogy panel) comes alive. The depth sweep doubles as the cave viewer: a slice at each depth shows the caves as holes. |
| Saving (`world/storage.rs`) | The plan is saved with the world, as the rivers are, so unchanged chunks are remade with the same caves |
| Rivers (`worldgen/carving.rs`, `world/carving.rs`) | Petering out, and a second ending: falling into a cave |
| Collision, raycasts, streaming | Nothing: they read blocks |

## 3. Options

### 3.1 "Am I inside a cave?"

**A. A plan of shapes, with a look-up by area** (recommended; it's Counsel's proposal on file):
- **The shapes:** tunnels are 3D lines with a width all along them. Chambers are blobs (a few
  overlapping round shapes). A shaft is a line standing up.
- **The look-up:** the plan is sorted into section-sized boxes (32 blocks), each listing the shapes that
  touch it. So most rock answers "none" at once, and the cost grows with the caves, not the world.
- **The carve** (GPU-safe):
  - The distance from a block to a line, squared, in whole numbers (fixed point), compared with the
    width squared. There's no square root, and the width along the line is a share worked out by one
    division.
  - Then the 3D whole-number noise pushes the wall in or out by a few blocks.
- **Making the plan** is world creation, so it's exempt from the GPU rules.

**B. Noise fields everywhere** (Minecraft's way). Ruled out by "there are no 'random' caves", and it
costs across the whole underground.

**C. Shapes stamped as blocks, made once and saved.** It's exact and lets you hand-build anything, but
it costs disk and can't be remade from the recipe. It's right for rare set pieces made in the asset
factory (a great hall, a geode), placed like peaks, so it comes later.

**The column's ground doesn't change:**
- A cave is taken out after the column is made. `CompiledMap::height` stays the top of the ground, so
  the light map, the far view, the climate and mineralogy are all as before.
- So it's linear: caves read the rivers, the rock and the climate, and nothing before them reads
  caves.
- **What it means:**
  - a cave's floor isn't a surface, so it gets no grass (its floor is the kit's: rock, gravel, pools);
  - a cave that comes near the top shows the surface's dirt in its ceiling, as it should;
  - a mouth is a hole in the near chunks only, until stop 4.

**Water:**
- A cave mustn't open beside a river, a lake or the sea unless it means to (the lakes' rule: water
  never stands against air).
- The carve leaves a skin of rock wherever the column has water nearby. The plan places its own water
  (a plunge pool, a cave lake at its own level).
- Water blocks don't flow, so the waterfall's falling water is the falls' curtain, which the rivers
  already have for their falls.

### 3.2 Seeing it cheaply

Caves break "walled-in sections are free". Every section a cave passes through has faces, and from the
surface you look at mountains full of them.
- **A. Measure first** (recommended). Caves are sparse by design: a waterfall cave is one passage, not a
  sponge. Stop 2 builds a stress test: a slider that fills the area round you with N test tunnels.
  It measures draws and frame times from the surface and from inside.
- **B. Which sections can see each other** (Minecraft's cave culling):
  - When a section is meshed, a flood fill through its open cells records which of its 6 sides
    connect to which (15 bits).
  - Each frame, a walk out from the camera's section reaches only what it could see through.
  - It's on the CPU, cheap and well known: about 350 lines.
  - It hides a cave behind rock from someone on the surface. From inside a cave, it hides the surface's
    sections too.
- **C. GPU Hi-Z culling** (the renderer plan's step 6, M to L). It's bigger, and it works with B
  rather than replacing it.
- **My recommendation:** build B only if A's numbers ask for it, before C. B doesn't depend on the
  renderer work and stays useful after it.

### 3.3 Dark underground

"Plain light" goes, and caves are pitch black except near a mouth. The ways to decide what's dark:
- **A. Depth below the recipe's surface** (the near window already has it). It's the cheapest, but
  wrong for a hole you dig: its floor sees the sky, yet sits below the old surface, so it would go dark.
- **B. Distance from a mouth, from the plan.** Each tunnel point carries its steps from daylight, as a
  river's level falls downstream. It's projected and cheap, but only natural caves have a plan. Mines
  and dug holes don't.
- **C. Short sky light from the blocks** (recommended):
  - A cell is lit if nothing solid stands over it. Light then spreads a short way (8 blocks, say) into
    covered cells, and everything further in is dark.
  - It's whole numbers and bounded: the spread stops after 8 steps, so it never floods a cave system.
  - It works the same for a cave mouth, an overhang, a hole you dig and a mine, so the plan needn't
    know its mouths.
  - It's worked out when a section is meshed, needing its neighbours' columns within 8 blocks.
  - **The catch:** an edit at a section's side changes its neighbours' light, so those are re-meshed
    too.
- **D. Minecraft's full sky light flood.** It's what Counsel's note says to skip, and the cost Minecraft
  pays in every cave.

**Where the light goes:**
- 16 levels at each corner, in the vertex's 4 spare bits, multiplied with the light map as now.
- Faces with different light don't merge, so a tunnel's mouth splits into a few more rectangles. Deep
  inside, everything is level 0 and merges as before.
- **A test lantern:** a light round the camera, one setting in the shader, so David can see a black
  cave while tuning, until torches exist.
- **Torches and fires (later):** placed lights, spread a short way, in the same 16 levels. C's spread
  is the same code.

### 3.4 Mouths far away (the 17b probe)

There are two candidates, tried on one mouth (`vision/watch-list.md`, item 17b):
- **holes in the canvas:** a second sheet behind a hole cut in the far sheet;
- **coarse voxel copies** at half, quarter or an eighth of the detail.

My guess: holes in the canvas, since it reuses the far sheets, their rings and the light. It has to be
measured, though. The probe needs a mouth to look at, so it comes after stop 1.

**[2026-11-02] Replaced:** coarse copies won, as one path for caves, builds and objects alike (section
13). The far view treats everything the same, with no IDs (David).

How urgent it is depends on the first kind of cave:
- A waterfall cave's mouth is a shaft, seen from above: the river vanishes into a dark spot a few
  blocks wide. In the far view that's a few dark pixels at most, which a darker cell on the sheet can
  already give.
- Mouths in mountainsides (sea caves, lava tubes, great halls) need it for real.

So it could move after stop 5. That's David's call (section 7).

### 3.5 Where the plan comes from

- **Its place in the chain:** caves come after Mineralogy (decided 2026-10-29), in the Caves section
  greyed on the Mineralogy panel today.
- **Picking where caves go is the placer's job:** fields (river ends, rock, height, slope) and a
  distributor choosing sites, as for the ores.
- **Growing a shape from a site is the kit's job,** which is new code. For the waterfall cave:
  1. a swallow hole where the river ends;
  2. a shaft;
  3. a plunge pool at the bottom;
  4. a passage running on downhill, under the valley, which may come out as a spring further down.
- **Rock layers aren't needed** for the waterfall cave: its walls are stone. Limestone caves in levels
  need them, which is a later kind.

## 4. The stops

Each stop ends with something for David to see. Stops 1 to 4 are probes: small, and in the order of
what's most unknown.

### Stop 1: a cave you can walk into (about 600 lines)

- The plan's format (tunnels, chambers, shafts), the look-up by area, and the carve in chunk making,
  GPU-safe.
- Whole-number 3D noise for the walls, with sliders: how much the walls wander, and how big their
  lumps are.
- **A test cave:** a button places one from the hillside at your crosshair: a mouth, a winding
  tunnel down 40 blocks, and a chamber. It's saved with the world.
- **Measured:** the time to make and mesh a chunk, with and without it.
- It's lit by plain light, as today, until stop 3.

### Stop 2: what it costs to draw (about 150 lines to measure, plus 350 if section visibility is needed)

- A stress slider: N test tunnels round you, and a flight over and through them, from the soak test.
- Draws, faces and frame times, from above and inside.
- Section visibility (3.2 B), behind an A/B switch, only if the numbers call for it.

### Stop 3: darkness (about 450 lines)

- Short sky light (3.3 C) at each corner, plain light removed, the test lantern, and sliders: how far
  daylight reaches in, and how dark is black.
- Edits re-mesh their neighbours' light.

### Stop 4: the mouth far away (about 500 to 800 lines; both candidates)

- The 17b probe, on stop 1's test mouth. Both candidates behind a switch, measured, and one chosen
  with David.
- **Replanned 2026-11-02:** one path, far copies, for caves, builds and objects alike, with build
  numbers first. See section 13. The two-candidate probe is gone.

### Stop 5: the waterfall cave (about 900 lines, petering out included)

- **Petering out** (water rework stage 6): a stream that's more than N from the next water ends.
  Its second ending is falling into a cave: a swallow hole instead of soaking away (a slider for which
  share do which).
- **The cave kit:** the swallow hole, the shaft with the falls' curtain, the plunge pool, and the
  passage on downhill, maybe out as a spring.
- **The Caves section** on the Mineralogy panel comes alive. The depth sweep shows the caves, and
  the map shows where the swallow holes are.
- **Same-spot respawn**, if tuning gets hard (Counsel reminds David).

### Sizes

| Stop | What | About (lines) |
|---|---|---|
| 1 | A cave you can walk into | 600 |
| 2 | What it costs to draw | 150 (+350) |
| 3 | Darkness | 450 |
| 4 | The mouth far away | 500 to 800 |
| 5 | The waterfall cave | 900 |
| Tests throughout | | 400 |
| **Total** | | **about 3,000 to 3,650** |

Then come other kinds of cave, each a kit: limestone in levels (after rock layers in 3D), lava
tubes, sea caves and fault caves. Then what lives down there, and set pieces from the asset factory.

## 5. Risks

- **Seams between chunks:**
  - A cave crosses chunks, and each chunk is made alone. The look-up reads only the plan, so both sides
    agree, but this is where the rivers' bugs lived.
  - Tests should cut a cave at every chunk side, as the rivers' tests do.
- **"One ground per column" is assumed in many places** (spawning, the near window, slopes, mineralogy's
  depth). Keeping the column's top as the ground keeps them right. But anything that asks "what's at
  this height" in the recipe, rather than the blocks, won't see caves.
- **Light ties into the renderer plan:**
  - Light at each corner splits merged faces near mouths.
  - The renderer plan's GPU face finding (step 5) has to carry it too, so the light format is settled
    once in stop 3.
- **Water against air:** a cave opening under a river, a lake or the sea would leave water standing
  against air. The skin of rock (3.1) prevents it, and has to be tested at every water edge.
- **Saved worlds:**
  - Chunks you've changed keep their caves.
  - Changing the recipe moves the caves in chunks you haven't changed (watch list item 20, as for the
    rivers). That's fine now, but matters once players keep worlds.
- **Depth:** under the lowlands there are only 128 blocks of rock. A cave system under a plain is
  shallow, unless the world goes below 0 (`WORLD_BOTTOM` is read everywhere, so that stays possible).

## 6. What scares me most

1. **The look.** As with the rivers: a tunnel that reads as a real cave and not a pipe is subtle, and
   every setting pulls on the others. The tools that helped with the rivers will help here: the depth
   sweep as a cave viewer, same-spot respawns, and sliders for everything. It will take as long as
   it takes.
2. **Light.** It's the one part that reaches past caves: digging, mines, torches, fires, the night,
   the renderer's future. Short sky light (3.3 C) is chosen so it doesn't box us in, because it works
   from blocks, not the plan. If it's wrong, it's wrong everywhere.
3. **Mouths far away.** It's the least known: the far view has never drawn anything that isn't one
   sheet. That's why it's a probe, on one mouth, before mouths are everywhere.

Performance scares me least: caves are few by design, and section visibility is a well-known answer if
the numbers ask for it.

## 7. Questions for David

1. **Where the waterfall cave's water comes from:** petering out first, so a river that ends with nowhere
   to go falls into a cave (my suggestion)? Or should the cave stage pick rivers to swallow, even ones
   that would have reached a lake?
2. **How black is pitch black:** true black, or a faint floor so shapes just read? And is the test
   lantern round the camera all right for tuning, until torches?
3. **The test cave in stop 1:** a button that places one at your crosshair (my suggestion), or one fixed
   place?
4. **Where the cave controls go:** the greyed Caves section on the Mineralogy panel, or a panel of its
   own ("caves might go in there after or have its own page because it might be a pain")? The name
   and the place are yours.
5. **The far mouth (stop 4):** before the waterfall cave, as Counsel suggests, or after it, since a
   waterfall cave's mouth is a shaft seen from above?
6. **The first waterfall cave's size:** one passage from the swallow hole to a spring, or a small system
   with a chamber or two?

## 8. David's answers  [DECIDED 2026-11-01]

David: "i would start with a simple, dead end cave, that worms a bit into the side of a mountain and
stops, so thats a perfect plan, we can worry about a waterfall cave after we have some more basic
ones functioning. i like those steps in the plan. caves panel of its own, but we may experiment with
where it goes in the pipeline. earlier is better if it doesnt clash"
- **The steps stand,** stops 1 to 4 in order. **Go ahead with stop 1.**
- **The first cave is a simple dead end:** a mouth in a mountainside, a tunnel that worms a little way
  in, and stops. The test button places it at the crosshair (question 3).
- **The waterfall cave waits** until more basic caves work. Petering out waits with it (question 1,
  answered then). Stop 5 becomes "the first planned kind" when its turn comes.
- **Caves get a panel of their own** (question 4). Its place in the pipeline is to be tried: earlier
  is better if it doesn't clash. *Counsel:* the rule from "content is added across the pipeline"
  (`docs/decisions.md`) applies. A kind of cave sits at the earliest stage that has what it needs. A
  dead-end mountain cave needs only the ground, so it could come right after Topography. Kinds that
  read the rivers or the rock come after those stages. Caves never change the ground the climate
  reads.
- **Not yet answered:** how black is pitch black (question 2; Counsel's lean: true black by default,
  with a slider for a faint floor), and the first waterfall cave's size (question 6). Ask him at
  stop 3 and stop 5.
- **The far mouth (question 5)** stays at stop 4, before any planned kind, as the steps have it.

---

## 9. Stop 1, a cave you can walk into: [BUILT 2026-11-01]

David: "i would start with a simple, dead end cave, that worms a bit into the side of a mountain and
stops". No chamber.

- **The plan** (`world/caves.rs`, `CavePlan`):
  - A list of tunnels. Each tunnel is a line of points (north, east, up, in 16ths of a block) with a
    half width and a half height at each, plus its walls' look (rough, lump, flat floor) and seed.
  - It's sorted into section-sized boxes, each listing the pieces of tunnel near it. Most of the
    underground answers "nothing here" at once.
  - The plan is the `Caves` resource, shared with the chunk-making threads.
- **The carve** (`inside_piece`): GPU-safe, whole numbers in 32 bits, with no square roots.
  - The nearest place on the piece is worked out in 256ths along it, and the size there is blended
    from the piece's two ends.
  - The walls are pushed in or out by `whole_value3`, the new whole-number 3D noise (`worldgen/noise.rs`:
    value noise, lumps every `lump` blocks).
  - The cross-section is an oval, wide `width` and tall `height`, with its floor cut flat `floor` of the
    way down. It's measured flat and up, so a tunnel should lie nearer flat than upright.
  - To fit 32 bits, a half width or height is at most 8 blocks and a piece at most 16 blocks long. A
    piece past that (only from a hand-edited `caves.toml`) is left out, so it's never carved.
- **In chunk making** (`CompiledMap::chunk`, which takes the plan now): the column first, then every
  solid block inside a cave becomes air. Water is never carved.
  - The uniform-stone shortcut stays for every section no cave comes near.
  - A chunk already made can be carved the same way (`carve_chunk`). A test checks that a chunk made
    with the caves and one made without and carved after are the same, block for block.
- **The ground doesn't change:** `CompiledMap::height`, the light map, the far view, the climate and
  mineralogy never see caves.
- **The test cave** (the World page, "Test cave"):
  - "Place a test cave at the crosshair" worms a dead end into the face aimed at. Aimed at a wall, it
    heads straight in; aimed at the ground, it heads the way you're looking.
  - It starts a step outside the face, so the mouth is cut clean. It has a point every 3 blocks,
    turning a little each time, and narrows over its last third to a round end.
  - **Sliders:** how far in, half width and height, how much it worms, rise and fall, heading down,
    size at its end, rough walls, lumps, and the flat floor. They're saved in "everything" presets as
    `test_cave`.
  - Its chunks are made again with it and timed, with and without it, and the World page says how
    long both took.
  - Chunks you've changed are carved instead, so your changes stay. "Remove the test caves" takes them
    all out, but changed chunks keep their holes, since they're saved as they are.
  - Chunks being made when a cave is placed are dropped and made again (`Streaming::forget`).
- **Saved with the world** as `caves.toml`, read before the chunks are made. A new world has none.
- **Lit by plain light** as before, until stop 3.
- **Not done yet, on file:**
  - a cave opening beside water leaves water standing against air (the skin of rock in 3.1). Not seen
    yet; to do if it shows.
  - the far view doesn't show the mouth (stop 4).
- **Tests:**
  - a tunnel is air inside and rock outside, stops at a round end, and has its flat floor;
  - rough walls wander, but the same seed carves the same;
  - the noise stays in range, is the same each time, and is smooth;
  - two tunnels cross the corner of four chunks: every block in six chunks is air exactly where the
    plan says, and made-with matches carved-after;
  - a cave's section isn't uniform any more, while one far from it is;
  - the plan survives `caves.toml`;
  - the biggest tunnel far out (30,000 blocks) is carved with no overflow;
  - the same seed worms the same way.

## 10. Stop 2, what caves cost to draw: the measuring [BUILT 2026-11-01]

David on stop 1's test caves: "there's nothing wrong with how the test caves came out, they were
quite nice actually, i played with the settings and tried making different shapes". He also saw
them vanish where the far view takes over, about 1,000 blocks out (stop 4).
- **The fill** (the World page, "What caves cost to draw"): "Fill the ground round me with test caves"
  places `how many` dead ends (10 to 2,000; 200 to start) within `how far round you` (32 to 1,024
  blocks; 400 to start).
  - Each is like the test cave, with the sliders above, wholly underground (8 to 100 blocks under
    the surface), heading every which way.
  - The worst case for culling is caves you can't see, which still have faces.
  - The fill is the same from the same spot: its dice are seeded by where you are.
  - "Remove the test caves" takes them out again, to compare.
- **Remaking chunks runs on every core** (`terrain::make_chunks`, shared with starting a world), so a
  fill of hundreds doesn't freeze the game for long. The World page says how long it took.
- **The numbers, live, under the buttons** (`render::drawing_line`):
  - the sections in view this frame (after Bevy's view culling) out of every section meshed;
  - their rectangles;
  - the frame rate (average, 1% low, worst frame).
  - Compare them with and without the fill, from above and from inside a tunnel.
- **What decides the next part:** if the fill adds many rectangles in view from above (caves behind
  rock being drawn) and costs frames, section visibility (3.2 B) comes next, behind an A/B switch. If
  not, it waits, and stop 3 (darkness) is next.
- **David's numbers (2026-11-01, uncapped, one spot, read 10 seconds after each fill):**

  | | No caves | 200 caves (410 round) | 2,000 caves (1,025 round) |
  |---|---|---|---|
  | Sections in view | 1,932 | 2,118 | 3,659 (+89%) |
  | Rectangles in view | 628,774 | 657,251 | 863,619 (+37%) |
  | fps | 253 | 260 | 252 |
  | 1% low | 166 | 157 | 163 |
  | Worst frame | 8.0 ms | 11.5 ms | 10.3 ms |

  - The fill of 2,000 remade 2,850 chunks in 1,316 ms.
  - Hidden caves are drawn (the rectangles in view go up), but even 2,000 of them cost no frame
    rate on David's machine.
  - **So section visibility (3.2 B) waits.** It comes back if forests, cities or denser cave kinds
    push the count much higher. The fill is the way to measure it again. **Stop 3 (darkness) is
    next.**

## 11. Pitch black, and the three kinds of light  [DECIDED 2026-11-01, David; the plan is Builder's]

David: "gameplay wise, i want them to be pitch black unless something is making light down there,
like a magic braziere from a cult lair or glow-lichen. how would you implement a lantern for me?"
- **Pitch black:** no faint floor. Underground is lit only by daylight near openings and by things that
  make light.
- **Three kinds of light, one brightness (16 levels):**
  1. **Sky light** (stop 3): a cell open to the sky is lit, and the light spreads a short way into
     covered cells (mouths, overhangs, dug holes); further in it's black. It's worked out when a
     section is meshed and stored at each face's corners. It changes only when blocks change.
  2. **Placed lights** (with the first glowing block, glow lichen perhaps): a block file's
     `light = 12`, spread a short way through air, stored the same way. A lair's braziers cost
     only the faces near them.
  3. **Moving lights** (a held lantern, a settler's torch, a fire arrow; when there are items to
     hold):
     - a small cube of light round the light, about 32 blocks across, flooded through the air on
       the CPU whenever the light moves a block (32,000 cells, a fraction of a millisecond);
     - sent to the GPU as a 3D texture, and read by each pixel;
     - it curls round corners and doesn't go through walls.
- **Stop 3's test lantern:** a light round the camera, in the shader, for tuning. It goes through walls;
  it's only for seeing a black cave.
- **Colour later:** brightness first. A brazier's orange and lichen's green need more than the
  vertex's 4 spare bits, so colour waits for a glowing block to look at.
- **[2026-11-01, David through Counsel] Colour is planned in from the first glowing block** ("i agree
  about colors"): in a world where light is precious, colour carries meaning (a warm torch, a cult
  brazier's green, the lichen's cold blue).
  - **The format, settled so colour never redoes it:** the vertex's 4 spare bits are the sky's
    channel only; its colour comes from the sky (day, the moons).
  - Placed lights get their own channel, brightness and colour (4 bits each for red, green and
    blue), in a third word of the vertex, when the first glowing block comes.
  - Moving lights' cube of light holds colour from the start.
- **[2026-11-01] The brazier:** David is modelling a Doom-style brazier in Blockbench.
  *Builder, for when he brings it:*
  - **Coming in:** Blockbench exports glTF, which Bevy loads as is. It's an object, not a block: the
    Registrar's tabs by kind (blocks, items, decorations, Blueprints) would take it as a
    decoration, with its model, the light it gives, and where its flame is.
  - **The flame, ways to animate it:**
    1. sprite frames, as Doom's own braziers are: a strip of frames drawn in Aseprite, on a flat
       flame that faces the camera, the shader stepping through them by time (cheap, and true to
       Doom);
    2. palette cycling (the water's plan, `docs/decisions.md`): one frame whose colours roll;
    3. the FX Lab's particles, later.
    - My lean: frames first. It's the look he's after, and it's the least to build.
  - **Its light:** a placed light, coloured, with a flicker in the shader (brightness wavering by
    time), so it costs nothing to animate.

## 12. Stop 3, darkness: [BUILT 2026-11-01]

- **Daylight** (`render/sky_light.rs`):
  - **The sky height:** each column's sky height is its highest solid block, worked out per chunk
    when a section first needs it and forgotten when the chunk changes (`MeshingState::sky`).
  - **The light:** a cell above that is open to the sky, full daylight (15). From there light walks
    through cells that aren't solid, one even step darker each block, to black at "how far daylight
    reaches in" (8 to start, up to 15). It goes round corners, never through rock, and further in
    it's pitch black. The walk is whole numbers and bounded.
  - **Most sections skip it:** when every cell in a section and just round it is solid or open to the
    sky, it's all full daylight. Only sections with covered air near them (caves, overhangs, holes
    dug, the inside of a roofed building) get the walk, from the cells `reach` + 1 blocks round them.
  - **Unloaded neighbours** count as solid and covered.
- **Stored with the faces:** each face takes the light of the cell in front of it, in the vertex's 4
  spare bits. Faces merge only with the same light. Pieces take their own cell's light.
- **The shader:**
  - daylight is squared, so it falls away quickly into the dark;
  - "plain light" for deep faces is gone in effect: deep faces are black;
  - the light map and hill shading are as before above ground.
- **Edits** make every section within 16 blocks again (`LIGHT_MARGIN`), for the light they change.
  - *Known gap:* opening a column to the sky changes the light all the way down it. A cave more than
    16 blocks below a hole dug down to it is lit only when its section is next made.
- **The Render page, "Light underground":**
  - "daylight underground" switches it on or off, to compare;
  - "how far daylight reaches in" (changing either makes every mesh again);
  - the **test lantern**: a light round the camera, 12 blocks by default, with its brightness. It
    goes through walls; it's for tuning only (section 11).
- **Tests:**
  - daylight reaching a tunnel from a shaft, a step darker each block, black beyond its reach;
  - none through rock to a pocket beside open air;
  - full where nothing is worked out;
  - an edit marking the sections within 16 blocks.

## 13. Stop 4, far away: one path for everything that isn't the plain recipe  [PLANNED 2026-11-02, Builder; waiting for David's go-ahead]

David's principle (`docs/decisions.md`, the Metaphysics Engine): the renderer treats everything the
same, and nothing gets an ID for it. The cleverness lives in one layer before it. Builder's assessment
(2026-11-02, through Counsel) chose coarse copies over a second sheet, so stop 4 is one path, not a
probe of two candidates.

### 13.1 The bands

| Distance | What draws the plain land | What draws where the world differs from the recipe |
|---|---|---|
| 0 to 1,024 blocks (render distance) | real chunks | real chunks |
| 1,024 to about 4,000 | the smooth sheet | **far copies** (this stop) |
| past about 4,000 | the sheet | nothing yet (big builds later: section 13.10) |

"Differs from the recipe" covers caves from the plan, holes, mines, builds, felled trees and placed
objects, all in the same way.

### 13.2 Build numbers (first, on their own)

David: "we should start labelling builds sequentially". It's the cheap half of recipe versions
(`docs/worldgen.md`: during pre-alpha a new build means new worlds; an old world only has to be
recognised and warned about, not converted).
- **The number:** a small file in the repository, `BUILD`, holding one whole number. Builder adds one
  each time a build with code changes is handed to David to play, in the commit before the launch.
  The first is build 1. A launch with changes not yet committed shows "build 12 + changes", as the
  logs already show "5d9239c + changes".
- **Where it shows:**
  - the first line of every session, crash and freeze log ("build 12, 5d9239c");
  - on screen (where is David's call: 13.11);
  - written into each saved world's `world.toml` (`build = 12`, beside the `game_version` that's
    already there).
- **Loading an older world:** it loads, with a warning: "nice25 was made in build 11; this is build
  12. Land the recipe makes may have moved since. Start a new world to be sure." A world from before
  build numbers says "made before build numbers". No conversion.
- About 80 lines.

### 13.3 Which chunks get a copy

The layer before the renderer asks the chunks' question, "what's here?", coarsely, and only where
the answer can differ from the sheet:
- **Changed chunks:** the list the save already keeps (all of them are in memory).
- **The cave plan's boxes** (section-sized, already sorted for the carve), only those whose height
  range meets the ground's in that column. A tunnel deep underground changes nothing seen from
  outside.
- **Then, once a chunk is made, the test that needs no IDs:** keep a copy only if the chunk has
  covered air that daylight reaches (daylight's own walk), or if it's a changed chunk with anything
  above the ground. Mouths, open shafts, pits and builds pass. Deep tunnels and deep mines don't.
- **Round each one:** its 8 neighbouring chunks are copied too (plain ground, cheap), so a copy's
  edge always meets plain ground, never a cliff or a gorge.
- **Range:** from render distance out to "how far copies reach" (up to 4,096 blocks, where the hand-off
  mask ends).

### 13.4 Making a copy (in the background, nearest first)

1. **The chunk at full detail,** the same way streaming makes it: the recipe carved by the plan, or
   the changed chunk itself.
2. **Daylight at full detail:** the same walk the near chunks use.
3. **Coarsened** into copy blocks 1, 2, 4 or 8 blocks across (the sharpness setting, 13.6, chooses
   which, by distance):
   - a copy block is solid if at least "how much must be solid" of it is (a half to start);
   - it takes the colour of the block most of it is made of;
   - each face takes the **average daylight** of the cells it faces into. So an opening bigger
     than a copy block stays a hole, dark inside, and a smaller one leaves the face solid but
     darker by its share. It's one rule, like a texture's mipmap. The darkness is carried as daylight,
     not baked into the colour, so day and night still work later.
4. **Meshed by the near mesher and drawn with the near shader,** scaled up: the same faces, the same
   light, the same fog. Teaching the mesher a coarse grid is the main new piece of code.
5. **Skirts:** short walls hung down from the copy's outer edge, below the ground, so no sliver of sky
   shows where its steps meet the smooth sheet.

### 13.5 Drawing it, and the hand-offs

- **The cut:** a copy marks its chunks in the hand-off mask the real chunks already use, so the sheet
  isn't drawn there. A lone copy marks itself (today a chunk is marked only if all 8 round it are
  drawn, which is changed for copies).
- **In order:** a copy's chunks are marked only once it's drawn, and an old copy, or the sheet, stays
  until its replacement is in, as tiles and real chunks already do. So nothing opens to the sky and
  nothing covers a mouth for a frame.
- **Coarser with distance:** a copy is made again at the next size when it crosses a band. It switches
  a little past the line, so walking back and forth on the line doesn't flip it. The old copy stays
  until the new one is in.
- **Real chunks take over at render distance:** a copy is dropped once its real chunk is drawn.
- **Made again when the chunk changes** (an edit near the edge, later villagers building far away).

### 13.6 Controls (the Render page; greyed when copies are off)

- **far copies on / off** (to compare, and the A/B for every test);
- **sharpness** (David's slider): how many pixels a copy block should take on screen, 1 to 4 (2 to
  start). Higher is crisper and costs more faces. Below about a pixel a block only shimmers, so the
  slider stops there. The copy block's size, 1 to 8 blocks, follows from this and the distance;
- **how much must be solid:** an eighth to a half. Lower keeps thin walls and pillars; higher keeps
  small holes open. It's the thin-wall test's slider;
- **how far copies reach** (blocks, up to 4,096).

### 13.7 The "who drew this" view

A debug view (on the Render page, beside the survey view) that colours every pixel by what drew it,
with the lighting kept so shapes still read:
- real chunks one colour;
- far copies another, a shade for each copy block size;
- skirts another;
- the sheet another, a shade for each ring.

Wrong hiding shows at a glance: a mouth painted in the sheet's colour, or a copy with sheet showing
over its edge. With same-spot respawns, it gives before-and-after screenshots. About 80 lines.

### 13.8 Objects (the brazier, later furniture and statues)

- **Through the same path, as blocks.** Objects are saved in their chunks, so a chunk with objects is
  a changed chunk and gets a copy. When it's coarsened, each object counts by its **shape**, the
  1/16-block pixels already worked out from its model for collision, in its model's average colour.
  It adds to the solid share of the copy blocks it's in, like any block.
- **So they drop out by size, not by a distance setting:**
  - a brazier (6 × 6 × 24 pixels, under half a block wide) is under half a pixel at 1,024 blocks, so
    it's gone from the copies at once, as it should be;
  - a 3-block statue lasts until its copy blocks are bigger than it;
  - a big object (a wagon, a ship later) lasts like a small building.
- **Not as models far away:** a model past render distance is a few pixels, and drawing each one costs
  a draw. Within render distance they stay real models, as now.
- **Sprites and light:** a flame sprite stops at render distance. An object's light far away is a far
  light, which waits for night (13.10).

### 13.9 Test sites, and what's measured

A button places the test sites in a line from you, each at three distances: inside render distance
(about 600 blocks), on the 1,024 edge, and in the copies' band (about 1,500 and 3,000 blocks). They're
saved with the world, so a same-spot respawn sees them again. The sites:
- **a mouth in a cliff:** the test cave, aimed into a slope;
- **a mouth on a copy's chunk edge:** the test cave started a block from a chunk's side;
- **a gorge with a pillar:** a 1×1 pillar 20 tall at the bottom of the narrowest river valley near the
  line (found by hand), where the sheet may bridge the gorge;
- **a thin wall:** a 1-block-thick wall, 20 long and 10 high, beside a 1×1 pillar;
- **a pit:** a 6 × 6 hole dug 20 deep, seen from above (the darker-cell case).

**The fast flight:** the soak test's flight, along the line of sites at top speed, watching for sky
showing through or a mouth covered as copies come in and change size.

**Measured** (on the Performance page):
- copies kept, and being made;
- the time to make one (least, average, most);
- their faces, and their memory;
- the frame rate, 1% low and worst frame, with copies on and off, at sharpness 1, 2 and 4, from the
  first site looking down the line.

### 13.10 Not in stop 4

- **Far lights** (a brazier glowing in a cave mouth at night, a castle's windows): they wait for night.
  When they come, a far light is shown only where its building is drawn, so a window light never
  floats over a castle that isn't there.
- **Big builds past 4,000 blocks** (a keep is still a dozen pixels tall at 8,000): coarser copies, or
  the sheet taking a build's height and colour. Not designed yet.
- **Trees** (none exist yet): far trees as sprites from the recipe's list, and a felled tree's sprite
  skipped by its own blocks, not by its whole chunk.
- **Far changed chunks kept on disk** with only their copies in memory: when towns make the count grow.
- **The same gap at the 1,024 edge today:** the sheet is drawn one chunk deep under the real chunks'
  edge, and its cliff triangle can fill a mouth in that last chunk. The test sites check it; if David
  sees it, the copies' cut covers it too.

### 13.11 The order, the size, and questions for David

1. Build numbers (13.2), about 80 lines. A launch, so David sees the number.
2. The "who drew this" view and the test sites, about 200 lines. A launch, to see today's gap at the
   1,024 edge and that every site is in place, before any copy exists.
3. The far copies (13.3 to 13.8), about 600 to 700 lines. A launch, the sites flown, then the
   measurements with David.

About 900 to 1,000 lines in all, a little over the first guess, since build numbers and the test sites
have been added.

**Questions for David:**
- Where should the build number show on screen? There's no title screen yet. It could go in a corner
  while the debug panel is open, at the top of the Performance page, or beside each world in the World
  page's list.
- Should the far copies' controls be a section of their own on the Render page, called "Far copies"
  beside the distant terrain's, or somewhere else?
- Build 1 is the first build carrying the number. Is counting from 1 right?

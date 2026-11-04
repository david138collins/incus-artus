# The far view by ray marching

David decided on 2026-11-02 that the far view becomes a ray marcher, now (`docs/decisions.md`;
the discussion and Builder's assessment are in `vision/proposal-voxel-renderer.md`, last section).
Each step goes to David before the next.

## Step 1: march the far view, past the real chunks

1. **Baseline** (build 11): `docs/caves.md` §13.24. Standing still, the far copies cost nothing;
   remaking them after a fast flight hitches frames up to 117 ms. The march must match the still
   frame rate and not hitch after moving.
2. **Graphics card time** (build 12): below.
3. The march over the far rings: plain ground and sea, starting where the real chunks end, writing
   depth; a switch "far view: sheet / march". Check the line where real chunks end on a snowy ridge
   (David: it's a wiggly ring on the land, and the ridge jitter is this line today).
4. The highest/lowest pyramid, for long steps over plain land.
5. Caves, changes and objects from the copy cells, uploaded as sparse bricks.
6. Light at night: braziers seen in far mouths.
7. The sheet and copy meshes retired behind the switch.

## Graphics card time (build 12)

Performance page, "Graphics card time": how long the graphics card spends on each drawing pass,
averaged over the last 20 frames, slowest first, with passes inside others indented. The total is
read against the frame: close to a whole frame means the card holds the frame rate back; well under
means the time goes elsewhere (the game, Bevy, Windows).

- Bevy times its own passes (`RenderDiagnosticsPlugin`, timestamp queries, DX12 and Vulkan only).
  Names are put in plain words; one the game doesn't know shows Bevy's own name.
- The near blocks, the far sheet, the far copies and objects are all one pass ("solid things"), so
  their shares are found by switching the sheet ("enabled", Distant terrain) or the copies off. The
  march will get its own line.
- Not timed: the Panini pass and the debug panel.
- "Write the frame rate to the log" (Far copies) now adds the card's total and its three slowest
  passes.

### Build 12 measured (David, 2026-11-02)
50 new mouths round (-3441, 17980), 45 past the real chunks; frame cap off; RTX 5070 Ti laptop.

| Reading | Sections in view | fps | 1% low | Card's time a frame |
|---|---|---|---|---|
| Copies on (56 drawn) | 2,522 | 133 | 74 | 1.25 ms |
| Copies off | 1,982 | 124 | 50 | 0.82 ms |
| Copies off (the third reading) | 2,129 | 145 | 65 | 0.88 ms |

David: "with everything on and caves, gpu time stays under 4ms."
- **The card isn't what holds the frame rate.** A frame takes about 7 ms; the card is busy for 1 to
  4 of them. The rest is on the processor's side, most likely Bevy preparing some 2,000 separate
  section meshes each frame (not yet measured).
- **For the march:** the card has plenty of room. A 980 Ti is several times slower than this card,
  and that still leaves room.
- The log line didn't say whether the far land was on; from build 13 it does.

## The march over the far land (build 13, step 1 part 3)

Render page, Distant terrain: **"drawn by the march (new)"** flips the far land between the sheet
and the march, to compare. Its three sliders (greyed while it's off): "march: step" (0.4, a step's
share of the height above the land), "march: shortest step" (0.002 of the distance) and "march: most
steps" (300). "Who drew this" shows the march blue, toward purple further out (the sheet is yellow
toward red). Code: `src/render/march.rs`, `assets/shaders/march.wesl`.

- **The same land.** As each sheet tile is made, its corners (height, the coarser ring's height for
  easing, slope, colours) are also copied into one list on the graphics card: a slot for each of a
  ring's 16 × 16 tiles, the tile at (x, y) in slot (x, y) counted round 16, and a second list of which
  tile each slot holds, so a ray never reads a tile that isn't in yet (it reads the ring outside).
  Between corners the land is the sheet's two triangles a cell, eased into the next ring out the
  same way; a test checks the march's land against the sheet's triangles, at corners and between.
- **Each pixel:** one triangle covers the screen. The ray starts where it leaves the square of
  chunks round you wholly drawn near; steps on by a share of its height above the land; where it
  goes under the land outside what the real chunks and far copies draw (the sheet's `shown`
  texture), narrows in on the spot over 8 halvings; lights it with the sheet's own sums and settings;
  and writes its depth, so the near world, water and far copies draw in front where they're nearer.
  A ray that goes more than 16 blocks (or 2 cells) under the land has gone into the near world's
  ground, and stops. Rays that miss show the sky.
- **Still the sheet's way, for now:** caves and changes are the far copies' (part 5); the whole list
  goes to the card again when tiles come in, at most ten times a second (about 25 MB with 7 rings;
  only while the march is on); its time is part of "solid things" on the Performance page.
- The frame-rate log line now says whether the far land was off, the sheet or the march.
- **To check on a ridge** (Counsel, from David): the line where the real chunks end is a wiggly ring
  on the land about 1,000 blocks out; the snowy ridge's jitter is this line today. Compare the two
  drawers there.

### Build 13 seen, and build 14 (David, 2026-11-02)
David: "ha, it works, and it does not look bad, but its jiggly, its bouncing back and forth real
fast... almost like the vibrating blocks"; it shakes standing still too, and "march: step" changes
nothing.
- **The shaking:** each ray's direction came from the difference between the camera and a point on
  the near plane, a tenth of a block away. Out at (-3441, 17980) the world's positions are kept only
  to a few thousandths of a block, so every ray pointed a little wrong, differently with every hair
  the camera moved. Build 14 works the direction out from the camera's own view and turns it to the
  world, so the big positions never enter it.
- **Hitches while tiles came in:** build 13 sent both lists whole (about 25 MB) ten times a second,
  8 to 12 ms of the frame each time. Build 14 sends only each tile that comes in (about 14 KB),
  written straight into the list on the graphics card on Bevy's drawing side (`write_march`); the
  lists go whole only when the far land starts again (a new world, other rings).

### Build 14 seen (David, 2026-11-02)
David: "i almost couldn't tell anything had changed when i turned on marching." No shaking.

| Reading (no cave mouths placed) | Sections in view | fps | 1% low | Card's time a frame |
|---|---|---|---|---|
| The march, copies on | 2,609 | 120 | 84 | 3.48 ms |
| The march, copies off | 2,301 | 107 | 25 | 2.98 ms |
| The sheet, copies off | 1,975 | 109 | 25 | 1.31 ms |

- **The march costs the card about 2 ms more than the sheet** on this card (the views weren't the
  same, so roughly). A 980 Ti is several times slower, so this is what part 4's long steps are for.
- **No more hitches from the march:** it no longer shows among the slow frames' steps.
- **Stalls of 110 to 140 ms every few seconds, with either drawer,** with none of the game's own
  steps slow (build 11's 117 ms worst was one). Not the march; not yet looked into.

## Long steps over plain land (build 15, step 1 part 4)

David, on the first look at build 13: "it was very pixelized. but it didn't look 'bad', it just
needed some tuning" (that was the shaking rays; a chunky far view on purpose is step 2's to offer).

- **A third list on the graphics card:** each tile's highest land, and each block of 4 × 4 cells'
  (the highest its corners reach either way their easing goes, so the eased land is never higher).
- **Each step is now one of three:** past a tile the ray is wholly above until it leaves it; past a
  block of cells likewise; or across one cell, meeting its two triangles exactly (no stepping into
  the land and narrowing in). A ray over plain land crosses a tile in one step.
- **"march: step" and "march: shortest step" are gone** (there's no step size left to tune); "march:
  most steps" stays, now 200, counting steps of all three kinds.
- **Tested:** `MarchState::march_ray` is the shader's steps in Rust; a test marches 360 rays over
  hilly land, two rings, through their easing, against a slow search in quarter-block steps. Every
  hit is on the land, never later than the slow search's, and nothing it finds is missed. (The slow
  search can step over a ray grazing the land for a few hundredths of a block; the long steps
  can't.)

### Build 15 seen, and build 16 (David, 2026-11-02)
David: "without having moved my camera, after turning on the march, there is a field of sky
speckles", and "when i move, the speckles sparkle, but i can see bands that they sparkle in lines".
- **Rays slipping through:** a ray meeting the land right on the line between a cell's two
  triangles, or right at a cell's edge, could be turned away by both triangles, or both cells, by a
  rounding hair (the lines David saw are the cells' edges and diagonals). Build 16: a triangle takes
  a meeting a hair past its edge, and a ray above the land leaving one step and under it entering
  the next met it in between.
- **Tested:** 20,000 rays steeply down at hilly land must all meet it; with build 15's steps 27 of
  them slipped through, with build 16's none.

### Build 16 seen; the stalls found (David, 2026-11-02)
David: "yes, the speckles were gone." On the stalls of 110 to 140 ms every few seconds while
flying: "the spikes arent the gui. it seems to be usually big hits of near light", and with "light
the near chunks like the distant terrain" off, "there are no 'near light' spikes, its always start
meshing. i can fly smoothly and, once the rendered chunks around me catch up, its done."
- The log agrees: with it off, one frame over 50 ms in 70 seconds of flying; with it on, stalls again.
- **Why (likely):** the near light window (`light.rs`) is made again each time you've moved about
  500 blocks, and each new one is handed to the graphics card as two new 2,048 × 2,048 pictures;
  making them on the card happens on Bevy's drawing side, which the game's own step timings don't
  see. Not yet fixed.

### Build 17: the near light window without stalls; the march on by default (David, 2026-11-02)
David: "yes, fix the near light stall, and lets turn march on by default."
- **The near light window's two pictures stay on the graphics card.** A new window the same size is
  written over them on Bevy's drawing side (`light::write_near_light`), handed over with the
  frame's other changes so it lands the same frame the near ground moves to it. New pictures are
  made only for the first window or one of another size (the render distance changed). Turning
  the cone map into bytes (32 MB) moved off the main thread, into the background job that makes it.
- If a stall is left, it's the writing itself (48 MB a window); the next step would be writing only
  the strip that's new, as the window slides.
- **"drawn by the march (new)" is on by default** (presets that don't say otherwise take it); the
  switch stays, to go back to the sheet.

### Build 17 seen (David, 2026-11-02)
David: "overall, it looks and runs good", and of the jitter left, "very very subtle, i have to stop
and watch very closely".
- **The flying stalls are gone:** after the world loaded, no frame of 110 to 140 ms; a few of 50 to
  70 ms, minutes apart.
- **The jitter left is the camera:** the Camera page's wobble line showed it moving up to 0.0039
  blocks every frame while standing, at (-1438, 17719). That far out a position is kept to 0.002
  blocks, so the pawn settling by one or two of those steps moves the camera, and the ground at your
  feet shifts by about a pixel. (Its "turned 0.04°" is the readout's own rounding: two identical
  rotations measure that far apart.) The cure for every case is drawing relative to the camera
  (for the renderer work); a standing pawn holding exactly still would cure it standing only.

### Set aside, not forgotten: positions far from the world's middle (David, 2026-11-02)
David: "it seems insignificant but also like it could cause problems later and we'd forget what it
was." Builder's advice, taken: set it aside, written down here, and fix it with the renderer work.
- **What it is:** positions are kept as 32-bit numbers, exact to about 1/500 of a block 18,000
  blocks out and 1/250 at the world's edge (36,000). Two things follow, both growing toward the edge:
  1. **The camera wobbles while you stand** (the pawn settling by one step of that), so near blocks
     shimmer by about a pixel. Cure for standing: a standing pawn holding exactly still.
  2. **Everything drawn is placed from the world's middle**, so blocks' corners round to those
     steps: the "vibrating blocks" while moving. Cure: draw relative to the camera (a "floating
     origin": what's drawn is placed from a point near you, moved along as you go). It touches
     every drawing part, the march included, so it belongs with the renderer work.
  - Physics, aiming and building far out keep to the same steps; not seen to matter yet.
- **When:** before or with moving the march's line inward, or sooner if it's seen to matter.

## The far copies drawn by the march (build 18, step 1 part 5)

David: "go ahead." Render page, Far copies: **"drawn by the march (new)"** (on by default; it works
only while the far land is drawn by the march too). Off, the copies are drawn as their own blocks,
as before, to compare. "Who drew this" shows the march's copies green.
- **What the march reads:** each copy's cells as `far_copies.rs` already works them out (step 3 of
  making one): solid, water or air; the block it looks like; whether it's plain ground; an air cell's
  daylight. Packed one number a cell (`CopyCells`), they go to the graphics card in pages of 32
  levels (512 pages, 64 MB), with a record of each copy (where, its cell size, how tall, its pages)
  and a map of which copy draws each chunk round you. A copy that doesn't fit is left to the land
  (and said once in the log).
- **Each ray:** in a chunk a copy draws, it steps through its cells one at a time; over a copy's top
  it goes straight on. It meets the copy coming from the air into anything solid or water, from
  above or the side, and meets the land the same way leaving a copy, so where a copy meets the land
  there's no seam to hide (the copies' skirt isn't needed).
- **Its look:** plain cells take the far land's look there (as the copies' plain ground does);
  the others their block's far colour, top or side by the face the ray came in through, that face's
  brightness, and the daylight of the air the ray came through, eased (squared) as the near ground's
  is underground, so a cave's inside is dark.
- **Long steps never go over a copy:** each ring tile's highest land is raised over any copy on it
  (a tower stands above the land), and stays raised while the copy's there.
- The hand-off mask now marks the copies' chunks 160 and the real chunks 255: the sheet isn't drawn
  under either; the march, drawing the copies itself, leaves only the real chunks to the near world.
- **Not yet:** the braziers' own light far off (part 6); the copies' cells still come from the
  copies' own making, as meshes too (retiring those is part 7).
- **Tested:** a copy's cells carry plain ground and not and dark cave air; a copy taken in raises
  the land's highest under it, keeps it raised when the land's tile comes in again, one too tall is
  left out, and all are let go when off.

### Build 18 stopped at the start; build 19
DirectX 12's shader compiler (FXC) refused the march's shader: "unable to unroll loop" (log,
2026-11-02). The new loops over the three ways (north-south, east-west, up-down) indexed a vector
by the loop's count, which it can only do by writing the loop out, and it wouldn't. Build 19 writes
each way out by hand (`leave_axis`, `since_entering`).

### Build 19 seen, and build 20 (David, 2026-11-02)
David, of a far cave mouth: "its the first look at this, so it can improve, but at first glance, this
is a more, 'verasimilitudinous' look." But backing away, "shards of sky appeared in several layers,
stopping when i stopped moving", then went away further out.
- **Why:** the copies' map (which copy draws each chunk) was laid out round your chunk and made again
  at every chunk you crossed; the map and its middle reached the graphics card by different paths,
  and for a frame could disagree, so rays looked up the wrong chunks and found air in a mountain.
- **Build 20:** the map is laid out by the world, each chunk at its place counted round the map's
  width (256 chunks), so moving never changes it; a ray also checks the copy it finds covers the
  spot. The copies lie within the hand-off mask, less than half the map's width from you, so no two
  share a place.

### Build 20 seen, and build 21 (David, 2026-11-02)
The shards while moving were gone, but a peak by a cave mouth about 1,000 blocks off lost its top,
sky in its place, once David had moved back across the real chunks' edge ("Who drew this": the
march's land round it, nothing in the hole). Not too few steps (2,000 changed nothing), not the
world's ceiling (the peak is 988).
- **Why:** where a far copy covers a chunk, the near world hides its sections
  (`apply_render_distance`), but the hand-off mask kept a chunk still in marked as real (255, the
  larger of the two marks), so the march left it to the near world, which wasn't drawing it.
- **Build 21:** a chunk a copy covers is always marked as the copy's.

### Build 21 seen, and build 22 (David, 2026-11-02)
The peak kept its top. Left: at the real chunks' edge, slabs of a coarse copy sticking out where the
peak fell away, with a sliver of sky; backing away, "the caves dissapear, and with each subsequent
redraw, a piece dissapears, until the far view march is just white"; sharpness 4 brings them back,
"but there always seems to be a square in the middle of them".
- **The caves fading out with distance** is the copies' coarsening (a mouth smaller than half a
  cell is filled), and the square is the mouth at its cell's size: as meant; sharpness keeps them.
- **"Just white":** a copy's cells that aren't plain ground were lit by their faces alone, so a
  snowy rim was flat white against the shaded land. Build 22 lights them as the land is lit there
  (its slope, the sun, the hill shading), in their own blocks' colours, then by their daylight.
- **The slabs:** a ray meeting a copy's plain cell now meets the land's own smooth surface through
  it (`land_through`), or goes on as through air where the cell stands above it; only cells that
  aren't plain (a mouth's rim, what's built or dug) are drawn as boxes. So the copy's plain ground
  is the far land's, and meets it round the copy with no step.

### Build 22 seen, and build 23 (David, 2026-11-02)
No slabs; "the two different views side by side are unsigthly" (the real blocks' textures, speckled
that far off, against the march's averaged colours, at the real chunks' edge: the line-matching
work, with the slider for where the march starts), "but, sky also comes through the shallow cave"
(fog off throughout).
- **Why:** build 22 let a ray through a copy's plain cell wherever the smooth land runs below it;
  on a sharp peak it runs below the blocks, so rays went through the cells round a shallow cave,
  through the cave and out to the sky.
- **Build 23:** a plain cell standing above the land is passed through only if the ray comes out of
  it into open air (a box sticking up out of the ground); into a cave's air or more ground, it's the
  cell's face (`opens_out`).

### Build 23 seen, and build 24 (David, 2026-11-02)
The light through the shallow cave was smaller, but still there in the march, while the real cave
is "pitch black". Not the far land's smoothing ("finest cell" 4 kept it).
- **A test first:** `no_ray_goes_through_the_rock_round_a_copied_cave` makes the far copies' own
  test cave in a steep hillside, copied at 1, 2 and 4 blocks a cell and with a fine and a coarse copy
  over the same chunk, and fires a grid of rays at the mouth from above it and from below (as David
  was); the shader's copy steps are in Rust beside the land's (`march_ray`). None went through the
  rock: the steps are right.
- **Why, then:** the test's cave is near the world's middle; David's is at (-4343, 17800), where a
  position is kept only to about 1/500 of a block, coarser than the nudge (1/1000 of a block at
  full detail) that takes a ray into a copy's next cell. A ray could round back into the cell it
  was leaving and step there until it ran out: sky. The same limit as the camera's wobble.
- **Build 24:** where a ray is in a copy, or a land tile, is worked out from that copy's or tile's
  corner (`(origin - corner) + ray × t`), so the world's big numbers never come into it.

### Build 24 seen; part 6, the braziers' light far off (build 25)
David: the march and the old copies now look the same ("drawn by the march doesn't make a
difference"); the copy's mouth is smaller than the real one (its coarsening) and keeps a speck of
daylight near the opening. "go ahead" to part 6.
- **The light in the cells:** each far copy floods its chunks' objects' lights (a brazier's, from
  `objects::lights_of`, as the near world's) through its own full-detail blocks, as
  `placed_light.rs` floods them through the near world's: a step dimmer each block, stopped by walls.
  Each air cell keeps the brightest that reaches its air, and that hidden in the solid cells beside
  it (as its daylight), packed in bits 20 to 31.
- **In the march:** a ray in a copy remembers the daylight and the glow of the air it came through;
  where it meets the copy, the daylight (eased, squared) is dimmed by the light map's shadows and
  the sky's brightness, and the glow (eased the same) isn't, so a brazier lights its cave at night
  as up close. The near ground's flicker isn't there yet.
- **Tested:** a green light just inside the test cave's mouth makes the air round it glow green,
  fading, and nothing else; with no light nothing glows.

### Build 25 seen; part 7, the copies' blocks retired (build 26)
David, at night with braziers in a far cave: "it is as you said ... could be finer, but we'll tweak
it later" (the green glow shows in the far mouth).
- **While the march draws the copies,** a copy is made without meshing its blocks
  (`CopyJob::mesh`): its cells are all the march needs, and meshing was most of a copy's making
  (200 to 320 ms at 8 blocks a cell), which hitched the frames after a flight (`docs/caves.md`
  13.24). A copy with cells and no blocks counts as drawn.
- **Flipping "drawn by the march (new)"** (or the far land's) makes every copy again, with or
  without its blocks, so the old way stays there to compare.
- Tested: made for the march alone, a copy has the same cells and no blocks.

### Build 26 measured (David, 2026-11-02)
50 mouths, a fast flight, then settled: 109 fps, 1% low 47, worst 31 ms, the card 2.0 ms a frame
(baseline, build 11, settled after a flight: 126 fps, 1% low 29, worst 117 ms; the views differ).
- **Copies:** 121 made, 12 ms the middle one, 152 ms the slowest (baseline: 95 ms average, 200 to
  320 ms at 8 blocks a cell).
- **Slow frames left** come from the near world's meshing as chunks stream in after the flight
  ("start meshing" up to 56 ms in one 174 ms frame), not from the far view.
- David: "it seems smooth", the Performance page near 110 to 119 fps (likely the frame cap at half
  the 240 refresh rate, 120, rather than the game).

## The line, movable (build 27)
David (by Counsel): after parts 6 and 7, "the march starts at N chunks" on the Render page, to drag
the line inward (32, 16, 4) and judge by eye and by the graphics card's time; then Builder's
assessment of marching the near world too. David: "proceed."
- **Render page, under the render distance:** "the march starts at (chunks)", 2 to 64 (64: wherever
  the real chunks end anyway). The real chunks are drawn only within a circle that wide
  (`RenderSettings::drawn_near`, which hides the rest, `apply_render_distance`); past it the far
  view draws everything, the far copies taking over what differs (they're made past the nearer of
  the streaming circle and this line). The hand-off mask and the far test sites follow it.
- The real chunks are still made and kept (and meshed) round you: only their drawing moves. So the
  CPU's work doesn't drop yet; the graphics card's time and the look are what it shows.
- Close in, the far copies' cells are a block each but drawn in their blocks' average colours, with
  no textures: how the far view looks up close, as it stands.

### Build 27 seen (David, 2026-11-02)
"i don't think we've ever had the border *more* seamless ... it looks nice at 64. quote hard to see
the seam, the lighting is very even. i get what you mean, pulling the march distance in did not
show actual regular detail like i thought it would." Up to 26 ms of chunk meshing flying fast.
- **Builder's assessment of marching the near world** (given to David, for Counsel's record): inside
  the line the march has only the smoothed land and the copies where the world differs, not the
  blocks. Marching near would need every near chunk's blocks on the graphics card (about 300 MB at
  full detail to 32 chunks, so coarser with distance, as the copies are), their textures, daylight
  and placed light per block, and the near world's pieces, bars and water; a few ms a frame on
  David's card, perhaps 5 to 15 on a 980 Ti, against 1 to 2 for the near meshes now; it would end
  the CPU's meshing (26 ms spikes) but send blocks instead; large, and the floating origin first.
- **Recommended:** an experiment first: with the line pulled in, every chunk inside it copied at
  full detail (not only where the world differs) and the copies drawn with their blocks' textures,
  to see block detail drawn by the march and its cost on the card. If it holds up, plan the whole
  change; if not, keep the line at 64 and spend the effort on the near world's meshing.

## The near-march experiment (build 28)
David: "ok. lets do it." To see block detail drawn by the march close in, and what it costs.
- **Whole copies:** with "the march starts at" pulled in past where the real chunks would end
  (below the streaming circle), every chunk in a band `FULL_BAND` (6) chunks wide just past the
  line is copied whole: forced, with no plain ground (`CopyJob::full`), so all its blocks are cells.
  Their sizes follow the distance as the copies' always do (1 block a cell close in). A tile
  becoming whole, or no longer, is made again.
- **Textured:** the march draws a whole copy's cells with the near world's block textures (bound to
  its material): the texture of the face the ray came in through, where on it the ray met it, at
  the size a block is on screen there (the smaller versions further off), shaded by the face's way.
  Its daylight and braziers' light as the other copies'. Who drew this: a paler green.
- **Room:** the copies' pages doubled to 1,024 (128 MB, the most one list may hold by default);
  the blocks' colour list now also carries each block's six face layers (four sets of four a block),
  so no list is added.
- Past the band, the land and the copies of what differs as before, untextured.
- Not there: random texture turning, pieces, bars, the near ground's hill and step shading, the
  flicker; and the real chunks are still made and meshed underneath.

### Build 28 seen, and build 29 (David, 2026-11-02)
- **The cost:** "with 'the march starts at 2', the graphics card is much faster when i'm somewhere
  i can't see anything, its at 1.42 in 8.25. at 64 in the same spot, its 2.5 in all. while floating
  over lots of distant land, its about 3.2-4 for both." The card has room; the frame (8.25 ms) is
  held on the processor's side, where the real chunks are still made and meshed underneath.
- **The look:** the band of whole copies reads as blocks, textured: "there's a mismatch but they
  both read fine" (the march's blocks brighter and flatter: no hill shading or step shadows yet).
- **A seam:** sky between the real chunks and the band ("Who drew this": neither). A ray over the
  real chunks was judged against the smooth land, which can run a block or two above their blocks,
  so one in the air over them was taken for one in the ground and turned away by the band's first
  solid cell. Build 29: over the real chunks a ray always counts as in the air (the near world draws
  whatever's solid there).

### Build 29 seen, and build 30 (David, 2026-11-02)
"gaps still present, looking down a ledge": sky between a ledge's real blocks and the band below.
- **Why:** a ray going on down past a ledge's edge, over the real chunks, fell more than 16 blocks
  under the smooth land (a gentle slope over the cliff's drop) and was given up as gone into the
  ground.
- **Build 30:** over the real chunks a ray is given up only far deeper (64 blocks, or 4 cells),
  past what the smoothing can be off by over a cliff; elsewhere 16 (or 2 cells) as before. (Never
  giving up there would march every ray under the near ground to the real chunks' edge.)

### Build 30 seen, and build 31 (David, 2026-11-02)
The gaps moved further down, but stayed: "around the base of the mountain", as far below him at
Z 1025 as at Z 630.
- **Why:** as far below wherever he was, so the rays' length, not the land: a ray down past a
  mountain went through the band of whole copies a cell (a block) at a time, through the air over
  its slopes, and ran out of its 200 steps.
- **Build 31:** each copy's record now holds its columns' highest and its 4 × 4 groups' (the cell
  over the highest solid or water one); in the air above them a ray goes past the group, or the
  column, in one step. The record grows to 568 numbers (about 2 MB for 1,024 copies).
- **Tested:** the cave test now runs at the game's 200 steps and over whole copies too; none
  through the rock. (The braziers' glow on open ground right under a long step isn't kept yet.)

### Build 31 seen (David, 2026-11-02)
"the gaps are... 99% gone": a thin straight line now and then while moving (likely a frame where
a copy is swapped for its replacement), and "some seams but they're just lighting": the whole
copies show as darker squares against the land (their blocks lit by face alone, the land by its
hill shading).

## Assessment after step 1 and the near-march experiment (Builder, for David and Counsel)
David: "i lean towards finishing the custom renderer ... having that done, and a graphics solution
set up that we can rest on, would be the best thing going forward."

**Built (builds 12 to 31):** the far view is a ray march. The far land in rings, long steps over
it by its highest, exact meeting of its triangles; the far copies as cells (daylight, the braziers'
glow, plain ground as the land's surface), no longer meshed; all of it worked out from nearby
corners, not the world's middle; a timer for the graphics card; "the march starts at" to move the
line; and, pulled in, a band of whole copies drawn with the blocks' textures. Fixed on the way: the
near light window's stalls, rays slipping through edges, the copies' map racing the camera, the
mask leaving chunks to nobody, rays running out of steps.

**Measured (David's RTX 5070 Ti laptop; the target is a 980 Ti, several times slower):**
- The graphics card: 1.4 ms a frame with the march drawing nearly everything where the view is
  short, 2.5 ms with the real chunks; 3.2 to 4 ms either way over a big view.
- The frame: about 8 ms, held on the processor's side (the real chunks made and meshed, Bevy).
- After a fast flight: worst frame 31 ms (117 before), 1% low 47 (29); a copy in 12 ms (95).

**What's open:**
1. **One look near and far:** the march's blocks and the land lit by different sums (the darker
   squares); the real blocks' textures against the far averages at the line; the flicker.
2. **What the meshes do that the march doesn't yet:** pieces (quarter blocks), bars, see-through
   blocks (water above all), random texture turning.
3. **The processor's time:** the march draws close in only from copies, made on top of the real
   chunks, which are still meshed underneath. No saving yet.
4. **The floating origin** (watch list 22): the meshes still place everything from the world's
   middle.
5. Small: the thin line while moving; the glow under long steps on open ground; the copies' cells
   needing a ring cell at least as big (finest cell 8 or more); 128 MB of copy pages at most.

**To finish the renderer (each part to David before the next; S small, M medium, L large):**
- **R1, the floating origin (M):** everything drawn placed from a point near the camera; a standing
  pawn holding still. First, since everything after relies on it.
- **R2, the near world straight from the chunks (M-L):** the real chunks' blocks, daylight and
  placed light sent to the graphics card as cells (the copies' format, a block a cell), as they come
  in or change, instead of copies; behind the line, their meshing stops. Measured: this is where the
  processor's time comes back (experiment C).
- **R3, one lighting (M):** the near ground's sums (face shade, hill shading, step shadows, daylight,
  placed light with its flicker) in one place both use, and random texture turning; the line
  disappears in light as well as shape.
- **R4, see-through blocks (L):** water and glass marched through, not stopped at: the ray keeps
  going and blends. Water is everywhere, so early.
- **R5, pieces and bars (M-L):** quarter blocks and bars as finer cells inside a block.
- **R6, sizes by distance (M):** full detail near, coarser cells further (the copies' sizes), by a
  memory budget, sent a piece at a time.
- **R7, the line all the way in (S-M):** the near meshes retired (kept behind a switch as the
  reference); models, sprites and the panel still drawn as now.
- **Then:** the resolution curve (A, B, C, the snap filter), and reuse of last frame if the 980 Ti
  needs it.

**Risks:** the 980 Ti (R2's measurement answers it early, on David's card scaled); see-through
marching is the hardest piece; DirectX's shader compiler (no loops indexing by count; long shaders
compile slowly); bugs that only show far from the world's middle (the rule: always measure from
nearby). Kept against them: the Rust twin of every step, tested against slow searches, and the
switches back to the old way.

**Decisions for David:** the order (R1 and R2 first is Builder's advice; R4 early for water);
whether the near look should match today's exactly or may change where the march makes it
easier; and what test content to build along the way (a stand-in forest for sheer amount).

**Counsel's advice, and Builder's answer (2026-11-02):** R1 then R2 first; measure on weak hardware
at R2, with a checkpoint before R3; the near look needn't match today's exactly (David's call); a
saved test world with fixed viewpoints (stand-in forest and village, a lake and river, a glass wall,
a quarter-block staircase, caves with braziers); the old way behind the switch at every step. And:
is see-through marching (R4) needed at all? **Builder: no, not now.** The march writes depth, so
water and glass can stay drawn surfaces blended over the marched world: the march treats them as
air where it draws close in (marching the riverbed), and the mesher builds only the see-through
blocks of those chunks. Far water stays the march's plane and river cells. It folds into R2 (small
to medium, not large). Lost: nothing today's water has; depth-tinted or refracting water would bring
it back. The path: R1, the test world, R2 (see-through still drawn), the weak-hardware checkpoint,
R3, R5, R6, R7. Waiting on David's word.

**The weak-hardware checkpoint (after R2):** David's laptop has integrated graphics, the pessimistic
floor; a friend's machine may be another. Picking it needs no code: Bevy reads `WGPU_POWER_PREF=low`
(the low-power adapter) or `WGPU_ADAPTER_NAME` (part of its name) at start, and the session log's
first lines name the adapter used. Windows' per-app graphics setting works too. A choice inside the
game would have to be made before the window opens (a settings file, not the debug panel).
**Put off (David, 2026-11-03):** the hardware test becomes a pre-alpha played by about 10 people on
their own machines; right and fast together, erring on the side of right (`docs/decisions.md`, "The
renderer: right and fast together, erring on the side of right").

## R1, the floating origin (build 32)
David (by Counsel): "lets do it", for the path R1, the test world, R2 and on; and "im not attached to
the near look".
- **What's drawn is placed from a point near you** (`coords::RenderOrigin`), on a 1,024-block
  grid, moved to the grid's place nearest the camera when it's gone further than that, first thing
  in the frame (`render::follow_origin`); everything drawn that isn't inside something else (the
  camera too) is shifted by the same, so nothing moves on screen. Game positions stay the world's;
  `RenderOrigin::to_bevy` and `to_game` turn one into the other, and every place that made or read a
  drawn position goes through them (the camera and its helpers, sections, far tiles and copies,
  objects and sprites, stand-ins, the sandbox, streaming, the light window, the deep shot).
- **The shaders get the origin** (the near ground's light settings, the far land's settings, which
  the march shares) where they turn a drawn position back into the world's: the light map, the
  near light window, the hand-off mask, a block's random turn (so it doesn't change when the origin
  moves), the far land's and copies' lookups. The march measures what must be exact (its place in a
  tile or a copy, its hits' depth) from the camera's place from the origin, never the world's big
  numbers.
- **A standing pawn holds exactly still:** on the ground before and after a tick and moved less
  than a thousandth of a block, it isn't moved (gravity pulled it down a hair and the ground pushed
  it back each tick, coming to rest a rounding step off far from the world's middle).
- Still 32-bit: the camera's own place far out (so a moving camera steps by about 1/500 of a block
  there, everything together, not block against block); the physics and the world's own positions.

### Build 32 seen, and build 33 (David, 2026-11-02)
"unfortunately, it is jittering. i can see it in the gabbro blocks": standing, walking mode, the
pawn still (speed 0, on the ground), yet the camera moved up to 0.00138 blocks every frame at
(-9305, 7165): one rounding step there (0.00098), north and east at once.
- **Why:** the camera follows the pawn's place smoothed between ticks, `previous.lerp(current, f)`,
  and glam sums that as `previous × (1 − f) + current × f`, which rounds a step off the place
  itself, differently each frame, even when the two are the same.
- **Build 33:** a place that hasn't changed is shown as it is; otherwise the last tick's place plus
  the share of the way to this one's (`SimPosition::display`).

### Build 33 seen (David, 2026-11-02)
"i think you got it, all zeros": standing, the camera's movement readout reads nothing. R1 done
enough to go on.

## The test world (build 34)
David: "proceed with the plan, i think your test world makes sense." World page, under "Far away":
**"Build the test world here"**, and **"Go to:"** forest, village, water, glass wall, half-block
stairs, caves, and "all, from afar" (`world/test_world.rs`).
- **The places,** on a ring round where it's built, 170 to 200 blocks off by the compass: a forest
  of about 900 stand-in trees over 180 × 180 (north); a village of 12 houses on levelled footings,
  with doors, windows and stepped roofs (north-east); a lake 32 across and 4 deep with a river 80
  long from it (south-east); a glass wall 24 × 10 of the outline block (south); a staircase of half
  blocks, 4 wide and 24 half steps up (south-west); three tunnels dug into the steepest slope near
  there, each with a room and two braziers (north-west).
- **Built of** the game's blocks, David's where they're there (dirt trunks, malachite leaves, stone
  brick walls, gravel roofs), else gabbro, grass and stone.
- **Edits like any others,** in loaded chunks or the chunk as kept, so saving the world keeps them.
  Where it was built is kept (`TestPlaces`, saved in presets) for the "Go to" viewpoints: back
  toward the middle and up, facing the place (the forest higher), flying.

### Build 34 seen (David, 2026-11-02)
"ok, trees, village, water, wall, very nice. tried the views", and the caves and stairs "looked pretty
much like you'd expect". He'd first pressed the older "Regenerate the test world" (the people
simulation's flat test ground, `regenerate_test_world`), which cut a hole into the real land: two
buttons named alike, a rename left for David to choose.

## R2, the real chunks past the line drawn by the march (build 35)
David: "proceed, brave builder". With "the march starts at" pulled in, the real chunks past the line
are no longer meshed; the march draws them, from copies made of the chunks themselves.
- **The band reaches where the real chunks end:** every real chunk past the line is in a whole copy
  (`far_copies.rs`, the near-march experiment's band, no longer 6 chunks wide), sized by distance as
  the copies always are (a block a cell close in).
- **Copied from themselves,** as streaming brought them in (a tile waits for its chunks; one past the
  streaming circle at the edge of a coarser tile still comes from the recipe): no second making from
  the recipe. Their see-through blocks and pieces are air in the copy (the near meshes draw them),
  their objects left to their models, and their placed light taken as the near world has it
  (`PlacedLights::lit_in`), lights beyond the tile's edge too. Copied only as deep as daylight
  reaches under the lowest ground (the recipe's copies go 24 down).
- **Made again** when a chunk in one changes, comes or goes (`MeshingState::touched`, from the
  dirty sections). Four made at once instead of two, the meshing they replace having left that time.
- **Meshing stops past the line** (`meshing::follow_the_line`): there a section is meshed only if it
  may hold see-through blocks or pieces (its palette says), and then only those
  (`MeshInput::only_see_through`); most give nothing at all. Drawn wherever the render distance
  reaches. As you move or the line does, a chunk come inside is meshed whole again (and counted as
  drawn only once it is, so the copy there stays till then: `MeshingState::unshown`), and one gone
  past is meshed again without its blocks.
- **Room:** the copies' cells are no longer held to 128 MB. That was taken for the most one list may
  hold; Bevy asks for the graphics card's own limits, and DirectX 12's is 2 GB. Now 512 MB (4,096
  pages; fewer if a card allows less), made empty on the card, not as 512 MB of zeros sent; 4,096
  copies at once. The Render page shows the pages used and any copies left out for want of room.
- **The switch:** Render page, Far copies, "the real chunks past the line too (new)", on. Off: build
  34's way (a 6-chunk band made from the recipe, the real chunks meshed underneath), to compare. With
  the line at 64 (wherever the real chunks end anyway) nothing changes either way.
- Not yet: glass and water marched through (they stay meshes; the near water is an opaque block, so
  the march draws it); pieces as cells (R5); one lighting near and far (R3); sizes by a memory budget
  (R6). The real chunks are still made and kept round you for the game; only their drawing moved.

### R2's pass criteria (David, by Counsel, 2026-11-02: "i think those are reasonable criteria")
Measured on the test world from the same spots, the line at 64 against the line at 8, the switch on
and off:
1. the processor's frame clearly below today's ~8 ms (the meshing gone past the line);
2. the graphics card no more than a few ms over the big view on the 5070 Ti;
3. a steady 60 frames a second at the default view on the weak machine (the laptop's built-in
   graphics, `WGPU_POWER_PREF=low`; a friend's PC if it's slower still);
4. the worst frame after a fast flight no worse than build 26's 31 ms;
5. David says it looks right.

If one fails, re-plan before R3.

### R2 measured (build 35, 2026-11-03, 5070 Ti laptop, fans on full)
In "nice25" (the big mountains, a harder view than the test world), standing still, the frame cap
off (`uncapped`: with the default cap of 1/2 the game shows 120 and the card's time counts its
waiting, 8 to 14 ms, so those readings don't measure it):

| | line 64 | line 8, the switch off | line 8, the switch on (R2) |
|---|---|---|---|
| frames a second, average | 310 | 344 | 348 |
| 1% low | 232 | 224 | 232 |
| worst frame | 4.8 ms | 5.1 ms | 6.4 ms |
| graphics card | 2.90 ms | 1.42 ms | 1.76 ms |
| frame | 3.22 ms | 2.91 ms | 2.87 ms |

Capped at 120: line 64 gave a 1% low of 70 and a worst frame of 23 to 27 ms; line 8 gave 90 and
15 ms. A fast flight at line 8 with the switch on: slow frames mostly 25 to 29 ms, one of 37 ms
(the near meshing), one of 97 ms just after it (none of the game's own steps). Flying over 2,000
mph outruns the chunks coming in: the copies past the line are made and let go as you pass (about
110 a second), and catch up within a second or so of stopping (David: "i can't fault it for that").
A first run with the fans turned down read slower (91 frames a second; the processor at 103 °C):
measure with the fans on full.

Criterion 1 was written on a wrong number: the ~8 ms "held on the processor" was the frame cap
(and, that night, the fans), not the game's work. Uncapped, line 64 is already 3.22 ms, the card
2.90 of it, so standing still R2 gains about 0.35 ms. Speed is not yet the case for marching the
near world; the case is that its cost follows the screen, not how much is in the world, which a
content-heavy comparison (line 64 against line 8 among tens of thousands of trees) would test
(Counsel, 2026-11-03).

**Why line 8 looks wrong (David's tests on build 35, 2026-11-03, no new code):**
- "light only" at line 8: the dark blotches go, so they're the textures (the march has no random
  turn, and picks the texture's size by distance alone). The contour lines stay, thin and faint: the
  steps are real; their thick red look was the dirt sides' colour.
- sharpness 2 → 4: the far lines get finer (the copies' 2-block cells further off were part of
  it); the saw-tooth edge doesn't move.
- "who drew this": the saw-tooth is the line itself, the real chunks' meshes (blue) against the
  march's copies (green), shaded differently (the march has no `ground_light`: no lie, height tint or
  feet of steps).
So the look gap is shading and cell size: one shading recipe for both (with the random turn), then
sizes per ray. Counsel's order: these tests, the weak-hardware check, one shading function, sizes per
ray (dithered), edge smoothing last.

Against the criteria: 1 passes (2.9 ms, but see above); 2 passes (0.34 ms for the real chunks); 3 is the
weak-hardware checkpoint, still to do; 4 is close (one 37 ms frame); 5 is David's look.

## One shading recipe (build 36)
David: "sounds good, proceed" (the plan sent to Counsel, with Counsel's seven points taken in).
- **The recipe, once** (`assets/shaders/lighting.wesl`): the near world's settings and light, a
  face's texture placement (`face_uv`), its random turn (`block_hash`, `turned`), the face shade,
  the ground light's sums (`ground_lit`: the light map, the lie of the land, the height tint, the
  feet of steps) and the light underground (`underground_lit`: daylight, the test lantern, placed
  light and its flicker). Each shader only reads its own inputs and draws with these. The meshes
  (`terrain.wesl`) draw exactly as before (a commit of its own, the move alone).
- **The march's whole copies** (the real chunks past the line, and build 34's band) drawn with it
  (`copy_shaded`): the block a face is on found in whole blocks from the copy's corner; its texture
  placed and turned as the meshes do (it had the tops a quarter turn off and the north faces
  mirrored); the texture's size from where the next pixels' rays meet the face's plane, measured
  from the eye (as the graphics card measures it for the meshes; no sharper at a slant than theirs:
  the block textures have no anisotropic filtering); lit by the face shade, the ground light (the
  window of the ground round you reaches the render distance, so it covers them) and the light
  underground, from the air the ray came through as a mesh's from the air in front of its face. Not
  lit a second time by the march's own light.
- The march gets the meshes' settings, light, window and cone map every frame, after they're set
  (`update_march` runs after `update_shader_settings` and the light's), so the shade doesn't hop
  at the line when the window or origin moves; and each block's faces' random-turn flags.
- The switch: "the march shaded as the meshes (new)", on the Render page under "the real chunks past
  the line too" (David's place and name). Off: build 35's own shading.
- Tests: neither shader keeps its own placement, turn or light, and both draw with lighting.wesl's;
  the march numbers the face a ray meets as the meshes number it.
- **Expected:** the blotches and the shading jump at the line gone near in. Further out the copies'
  2- and 4-block cells may show faint bands (the shading is block-accurate; the cells aren't): that's
  the next step, sizes per ray. Judge this one on the near range.

### Build 36 seen, and build 37 (David, 2026-11-03)
- "its improving": the blotches and the shading jump at the line are gone near in.
- Whole tops came out dark further off. Tests from one spot: the old shading (the switch off) had
  none, nor did the meshes at line 64; "foot shading" and "step shadows" at 0 took them away, step
  shadows at 1 made them stronger, sharpness 4 pushed them out to the far edge. So it's the coarse
  cells: a 2- or 4-block cell's top can stand a block or more off the ground's own, and the cone map
  is block-accurate (Counsel's point 7). Build 37: no shade at the feet of steps on cells coarser than
  a block (the lie of the land and the height tint stay). Sizes per ray replaces this.
- Still there: dark lines on the steps' sides in the middle distance, ending where the copies end
  and the smooth far land starts. Up close the march's steps look like the meshes'. A step's side a
  fraction of a pixel tall is hit or missed whole by one ray a pixel, where the meshes' edge
  smoothing (4 samples a pixel) blends it to a faint line: the next step's work (sizes per ray, then
  edge smoothing if it's still needed).

**On file, not seen (Counsel, 2026-11-03; checked: `light.rs`, `make_near` reads `map.height`):** the
window of the ground round you takes its slope and height from the recipe, not the real chunks. Over
land that's been dug or built on, the meshes' and the march's ground light (the lie of the land, the
height tint, the feet of steps) would follow the ground as it was made, not as it is. Not acted on
until David sees it; sizes per ray should take its slope from the copy's own column tops instead.

## Sizes per ray, round 1 (build 38)
David (by Counsel, 2026-11-04): "i like sizes by ray idea ... sizing rays fits our philosophy. yes,
choosing when cheap is always a good idea. a slider for the pixelization rate of terrain as it
becomes more distant is a good idea too", and "go ahead with round one". Counsel's seven points
taken in (`vision/proposal-voxel-renderer.md`). Round 1 is the sizes and their picking, in flat
colours; round 2 the land's look toward a pixel a block.
- **The levels** (`far_copies.rs`, `copy_levels`): each whole copy keeps its cells 2, 4 and 8 to a
  side as one, each made from the eight finer below it: solid if at least the share of them is (the
  block of its highest: the surface), else water if as much is water as air, else air, its daylight
  as bright as its air's on average (averaged eased, so squaring it again doesn't darken it) and the
  brightest placed light in it (a far brazier isn't averaged away). About a seventh more room, in the
  copy's own pages after its cells (a copy too tall for 16 pages is drawn at its own size). A whole
  copy's bottom is a whole number of its coarsest cells up, so its coarser cells line up with its
  neighbours'.
- **The picking** (`march.wesl`, `sized_level`): each spot in a whole copy is drawn at the level whose
  cells are about "pixelization" pixels across there. A level's cell is used if the level wanted at
  its middle is past the one below by more than its own "random" number (from where it is): the
  change from one size to the next goes a cell at a time, the same for every ray, each cell changing
  once as you come near; fixed by the world, so a ray comes into a cell only through one of its faces.
  Never finer than the copy keeps. Where a copy might be drawn coarser, rays go past open air a square
  of 8 × 8 columns at a time, over everything solid in it rounded up to a coarsest cell (a coarser
  cell can stand above its own columns' highest, so the columns' own steps could pass through it).
- **The look**: a block a cell as build 37 (textured, the one recipe); coarser cells in their block's
  flat colours (the top's on the top, the side's on the rest), by the same recipe without the feet of
  steps (`copy_flat`). "Who drew this": green a block a cell, then yellow, orange, red.
- **Controls** (Render page, under "the march shaded as the meshes"): "far cells sized per pixel
  (new)", off for build 37's way; "pixelization (pixels a far cell)", 1 to 8, default 2 (higher is
  blockier further off, and quicker). Names and places David's to change.
- **Tests**: a coarser cell from its eight (the share, the highest block, the averaged daylight, the
  brightest glow); the copied cave's slope from 250 blocks, copied whole at 1 and 2 blocks a cell, at
  sizes from a block to 16: no ray through the rock of the size drawn, every ray on the face it chose
  (a hair past it only by the nudge into the next step), every size drawn somewhere.
- **To see**: the squares where one copy size met the next gone? Faster further off? The thin
  red-brown lines on the far steps' sides should mostly go once cells are coarser than a pixel's
  worth of steps; what's left is round 2's. Play capped; uncapped only to measure (the laptop's heat).

### Build 38 seen (David, 2026-11-04, a partner test)
- **The levels**: green, yellow and orange, changing a cell at a time ("who drew this", line at 8);
  the look sharp ("overall it looks sharp"). The switch off and on from one spot: no change seen
  (the coarser cells cost nothing in the look).
- **The flaw: dark patches behind hills, inside the line** ("darkening more than usual within the
  8"), ending in a sawtooth along the line's chunk edges. Not build 38's: the meshes and a block a
  cell copies darken a top below the ground between it and the sun (step shadows) and a step's foot
  (foot shading), both by the cone map; build 37 turned both off on cells 2 and 4 blocks a cell, and
  most copies past the line are those (238 at 2, 146 at 4 in his view), so the switch can't change it.
  Step shadows at 1.00: big dark patches stopping dead at the line; at 0, "better, but not perfect";
  foot shading at 0 too: "makes the entire world pretty much seamless ... the bands fade almost
  imperceptibly through the march border ... the bands get coarser without changing shade. this
  looks good right now".
- **Proposed fix, not built**: both shadings back on the coarser cells, leaving out any darkening less
  than the cell's own size (how far its top can stand off the true ground: build 37's dark tops),
  keeping the deeper (a hill's shadow), so the shadow carries on past the line.

### Build 39: the shadows carry on past the line (David: "go ahead")
Both shadings back on every march cell: a top is darkened only by how far it's below the cones less
its cell's size less a block (`near_ground_light`'s `size`: 1 a block a cell, the copy's size times
2, 4 or 8 for the coarser), so a block a cell is shaded exactly as the meshes, and a coarser cell
keeps a hill's shadow but not the block or so its top can stand off the ground. To see: the step
shadows and foot shading back up, the patches behind hills going on past the line without the
sawtooth, and no dark tops on the far cells (build 36's).

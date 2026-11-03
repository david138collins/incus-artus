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

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

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

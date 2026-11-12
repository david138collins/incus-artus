# The references from old Incus Artus

The yardsticks the rebuild is checked against (`master/plan.md`, 1.6): what old drew and made, kept so the new engine
can be set beside it, picture by picture and column by column. Taken 2026-11-11 from build 115 (e99dcc1), committed,
on the 5070 Ti in "NVIDIA GPU only", 2560 × 1600, the dyno's presets (`presets/render/dyno.toml`, `camera/dyno`,
`player/dyno`). How they were taken: `INCUS_DYNO=baked INCUS_DYNO_LAPS=0 INCUS_DYNO_REFERENCES=1 INCUS_LOAD_WORLD="nice25
test" INCUS_START_PRESET=render/dyno,camera/dyno,player/dyno cargo run --release -j 4` (`soak.rs`, the references
flight, with `INCUS_DYNO_SETTLE_MOST=600` so every place settled first). Its log: `kept logs/dyno-1794433359.txt`. (A first flight, `kept logs/dyno-1794431890.txt`, pictured the spawn and the sea before their live levels were whole at the 180-second cap; its pictures were replaced.)

## The world

All of it is the test world, **"nice25 test"**: David's nice25, rendered with the dyno's presets and saved with its bake.
Not the startup default world.

- **Its definition:** `references/world/`.
  - The recipe's settings: `compiled.toml`.
  - Its eleven maps: `map-*.png` (height, coast, coast distance, lift distance, push strength, push x and y,
    squeeze, rain, temperature, tilt).
  - Its river network: `rivers.txt`.
  - Its terrain and world files.
- **Its bake** (the cells of 32 and up, 200 MB): only in `worlds/nice25 test/sight-bake.bin` and the USB copy. It's
  made again from the files above.
- **The sample columns:** `references/columns.txt`. 5,047 columns:
  - every 1,000 blocks over the map, from −35,000 to 35,000 each way;
  - the column at each pictured place.

  Each gives north, east, the ground's Z and the water's top, to the block: `CompiledMap::column`, what the chunks are
  filled from and the card's columns are checked against.

## The pictures

`references/pictures/`: 24 pictures of the world alone (no panels, `runner::WorldPicture`), six places at four times of
day. Each place was pictured once its live levels were whole: the spawn after 288 s, the mountain top 43, the build 251 (and 2 more after it went in), high over the spawn 179, the sea 216, the mountain top with fog 149. The times of day
are the sun's height, from 315° (the dyno's): noon 40°, low sun 10°, dusk −4°, night −25°.

| | place | how it looks | settings |
|---|---|---|---|
| 01 | On the ground at the spawn (0, 0) | a player's eye (1.38), north, level | fog on |
| 02 | The mountain top (north −606, east 78) | a player's eye, out from the middle, 5° down | fog off: the clear day |
| 03 | Beside the quarter-block build | 8 blocks off its north side, at its middle; the build replayed from `kept logs/events-1794341728.log` | fog on |
| 04 | High over the spawn | 400 over a player's eye, north, 30° down | fog on |
| 05 | Low over the sea | 20 over the sea at the route's far point, away from the middle, 2° down | fog on |
| 06 | The mountain top again | as 02 | fog on |

Every other setting is the dyno's preset: the finest layer 2 × 2, the coarsest 8 × 8, shadows and sky light on,
textures on, the far cells blended, the light not kept.

## The marks' flights

With the repaired instruments (experiment 1), sight on the card in ms, median, standing:

| flight | build | the route (median / 99th) | spawn | mountain top | build |
|---|---|---|---|---|---|
| C, `kept logs/dyno-1794419654.txt` | 111 | 4.06 / 9.69 | 5.94 | 6.58 | 11.71 |
| C2, `kept logs/dyno-1794427178.txt` | 114 | 4.07 / 9.71 | 5.88 | 6.59 | 11.68 |
| B, the Intel card, `kept logs/dyno-1794416695.txt` | 110 | (marks only) | 22.69 | 20.57 | 29.49 |

The rest of the record (light off, unchecked, crisp, the in-between, the light kept) is in `journal/docs/experiments.md`, with
every log in `kept logs/`.

## How the new engine is checked against them

- **Pictures:** kept for reference, not a yardstick. David (2026-11-11): "still not sure what good the pictures are.
  it'll look good or it won't, idc what the old things look like in particular, other than the basic way we're doing
  things, like pixelating the far horizon." The new engine is judged by his eye on its own. These are there if a
  question comes up ("was it like this before?"), not as a test to pass.
- **Columns:** the new world's ground and water at the same columns, to the block, for the same world. A different
  recipe on purpose (a stage rewritten) is judged by David's eye on the maps, not by the columns.
- **The marks:** the same three, flown the same way. The new engine's numbers go beside C2's.

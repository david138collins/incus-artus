# The far march by its parts: its size, what it carries, and what could later come out

Written 2026-11-16 by Builder, at David's word: "scanning the march and noting in a file anything that can later
be removed from the march to free up room on it; i would like for a sort of 'windirtree' view of the march by its
components, and the size of those components inside the march, if possible, so that later review could identify
ways to further improve it by removing sections that do little and replace them with something else."

It describes the march as it stood at **build 194** (`src/far.wgsl`, `src/far_all.wgsl`), the program
`march_all` with cover by faces on. It's a snapshot: section 6 says how to take it again. Nothing here is a
change to the march; every candidate in section 5 is a thing to try, with its pass mark first.

## 1. Three sizes, and which one is "room"

A part of the march can be big in three different ways, and they don't go together:

- **How much program it is** (instructions). Measured here for every part, and it adds up (section 2). It says how
  long the program is, not how much time a part takes: a part most steps never reach can be large.
- **How much it holds in hand** (the values it keeps for every ray). **This is the march's size on the card**, the
  thing David asked to be watched ("the weight of the shader ... its size on the card"). The card fits fewer rays
  at once the more each one holds: 12 groups of rays at 168 registers today, where a program of about 128 or
  under fits 16. Section 3 lists what's held, by name.
- **How much time it takes.** Known only for the parts the audits priced (section 4).

The march's size on the card is **its fullest moment, not the sum of its parts.** Take one part out and another
moment holds the count up: the audits found parts that freed nothing alone and 40 registers together.

## 2. The tree: the program by its parts

`march_all` is 4,548 instructions as the game's compiler makes it (4,436 with cover by faces off). Each
instruction is counted once, in the part of the program's text it came from, so the parts add up to the whole.
The bars are shares of the whole program. (The full tree, down to parts of 12 instructions, is
`kept logs/march/b194/parts.faces-on.txt`.)

```
march_all, the whole program                                             4,548  100%  ########################################
  one ray, start to finish (march_pixel)                                 3,658   80%  ################################
    the march itself: a ray's steps                                      2,537   56%  ######################
      the look: the hand-over's two reads and their mix                  1,138   25%  ##########
        finding where the two reads land in the textures (spot_kept)       792   17%  #######
          of which the edge cases (spot_soft, twice)                       613   13%  #####
        the fall-back reads, when a texture can't answer (look_at)         109    2%  #
        the coarser look turned into the finer cell's (by_finer_cell)       82    2%  #
        mixing the two (mix_looks)                                          76    2%  #
        the two reads themselves (fetch_together)                           58    1%  #
      the look: its own level's read                                       448   10%  ####
        finding where it lands (spot_kept)                                 399    9%  ####
          of which the edge cases (spot_soft)                              307    7%  ###
        the read and its fall-back                                          46    1%
      the skip over clear air (the height table's three rows)              234    5%  ##
      each step: its region, summed or not                                 171    4%  ##
      after the last step: the floor's ground, the sky behind              138    3%  #
        the sky behind the ray (sky)                                        84    2%  #
      the gather: what covers the step (bulk, water, faces)                105    2%  #
      the gather: its light and colour                                      84    2%  #
      each step: the level for this distance                                68    1%  #
      the hand-over: how much of the next level to mix in                   43    1%
      the step's length: surface, thin haze or air; the walk back in        40    1%
      each step: its stops (the ray's end, covered, a cut's stop)           32    1%
      its start: what a ray sets out with                                   26    1%
      the floor's cells kept, the first surface, the step's end             10   0.2%
    the finish: light, fog, the sky's weight, what the next frame needs    475   10%  ####
      light read from the cell (shaded)                                    162    4%  #
      fog (fogged; off as the game is launched, compiled in all the same)   84    2%  #
      what the layer below is worth here (worth_of)                         60    1%  #
      the long rays' list (list_long)                                       35    1%
      the rest (the picture's colour, what's written for the next frame)   134    3%  #
    the ray's set-up, its own lines                                        124    3%  #
    where its steps begin: four patterns (ray_start)                        92    2%  #
    where it ends at the map's edge (map_end)                               76    2%  #
    the cut: a length's stack written, its places read                     146    3%  #
    the rest (the ray's way, the layer above's worth and touch)            208    5%  ##
  joins, where values from different paths meet                            717   16%  ######
  the dispatch: which layer, which thread, the strip                       173    4%  ##
```

What the tree shows that wasn't written down before:

- **A third of the program is finding where a read lands in the bricks' textures** (`spot_kept`, three copies:
  1,191 instructions), and three quarters of that is its edge cases (`spot_soft`, 920): a place within half a cell
  of the world's floor (the cell under is rock, and no texture holds it), a brick with no texture, the first column
  of a level. The reads themselves are a fiftieth.
- **The hand-over's second read is a quarter of the program** for the few steps that mix two levels.
- **The gather, where the picture is made, is a twentieth.**
- **Fog is in the program though it's off** (84 instructions in the finish), and all four of the ray start's
  patterns (92).

## 3. What the march holds in hand: 98 values round its loop

The march's loop carries 98 numbers from one step to the next for every ray, whether a step uses them or not.
These are its permanent residents: of the 168 registers at its fullest moment, 98 are these and about 70 are what
the fullest step is working with. (With cover by faces off it carries 97, and the row David read went from 167 to
168: the one more is the last step's face weight, kept for the next. So this count follows the registers.)

| what's held | values | what it's for |
|---|---|---|
| the numbers of the level being read and of the next one (`kept`, `kept_next`) | 30 | so the levels' table isn't read again at every step (the lean step, build 142 on) |
| the height table's three rows (`row_own`, `row_up`, `row_far`) | 15 | so the skip reads a column's numbers once while the ray stays over it |
| the last step's look, whole (`before`) | 13 | a step is gathered from both its ends, so the look at the end before is kept |
| the floor's cells (`bottom_here`, `bottom_sun`, and three more) | 9 | a ray that goes down through the world's floor gives what's left to the ground it last gathered, not the sky |
| what's gathered so far (colour, the sun's part, what's still let through) | 7 | the ray's answer |
| where the ray is and how it's stepping (its distance, in air or not, walking back in, the step's length) | 8 | |
| the region's place (`place`) | 5 | whether the region under the ray is summed |
| the level for this distance and its stretch (`lv`, where it began, ends, hands over) | 4 | |
| the first surface and the first touch | 3 | lit once at the finish; the finer layers' lead |
| the step counts | 2 | |
| naga's loop guard (`loop_bound`) | 2 | a counter naga adds to every loop so none can run for ever |

## 4. What the audits measured, part by part (builds 177 to 180)

Each part was taken out of a copy of the march and the copy traced under Nsight, David reading the rows; the
copies were then timed and three were flown. (`journal/docs/experiments.md`, 2026-11-15; the switches still work:
section 6.) "Need" is registers and what's put by in memory.

**By room** (the march once the textures are in, build 179; the whole was 167, 12 groups):

| taken out | need | groups that fit |
|---|---|---|
| nothing | 167 | 12 |
| the hand-over's second read | 153 (14 fewer) | 12 |
| the floor's cells | 161 (6 fewer) | 12 |
| the height table's third row | 161 (6 fewer) | 12 |
| the finish | 161 (6 fewer) | 12 |
| the rings | 167 (none) | 12 |
| the hand-over and one of the floor, the third row or the finish | 143 to 150, with 15 to 22 words put by | 16 |
| the hand-over, the floor, the third row, the rings and the finish | 127, nothing put by | 16 |

Earlier, on the whole march (build 177): the skip holds none of the registers at the fullest moment; crossing the
air to the first sample alone is a program of 68 registers that fits 28 groups, and costs 0.008 ms.

**By time:**

- *The skip is worth a third* of the plain's march and of the mountain top's base layer (without it 1.55 and 1.44
  times as long).
- *The hand-over's second read, out* (a ceiling: the picture is wrong without something in its place), flown with
  every layer marching (build 180): 14% off the plain, 4 to 7% off the mountain top, high over the spawn and fast
  across, next to nothing over the sea, on the run and at the spawn.
- *16 groups where there were 12,* flown (build 180): 2 to 7% at the two marks where the card works; 3 to 4% dearer
  where it waits, when the groups were bought with words put by. David's rule since: the registers aren't chased
  as a goal.
- *The floor, the third row or the finish out alone:* 5 to 12% longer on the base layer alone, with 6 fewer
  registers. Not understood.
- *Rare, heavy paths cost every ray:* on the plain 0.6% of steps mix two levels, and with that read out the march
  was 14 to 20% shorter. Rays march 32 to a group in step, and when one takes a slow path the other 31 wait.
- *Cover by faces* (builds 191 to 194, flown off, on, off): one register, 112 instructions, about half a percent.
- *The cut* (builds 183 to 190): no registers (the row was 167 before it and after).

**By their pictures** (build 177, each part out against the whole): the views and the floor's cells change
nothing at the five marks; the third row has to stay (over the sea rays run out of steps without it); the
hand-over's second read changes 3 to 11% of pixels.

## 5. What could later come out, or be made lighter

Each is a candidate, not a recommendation. "Measured" means a trace or a flight said it; "unmeasured" is
Builder's reading of the program as compiled. Ranked by what Builder would try first: cheapest and surest at the
top.

| # | what | what it would free | what it would cost | how sure |
|---|---|---|---|---|
| 1 | **naga's loop guard.** A counter added to all 12 loops of the program so none can run for ever. wgpu lets a shader the game trusts be made without it (`create_shader_module_trusted`, `ShaderRuntimeChecks`, with the division and bounds checks as two more switches) | 2 of the 98 held values, and a few instructions a step in every loop | nothing in the picture; the game must then promise its loops end (the march's stops at its step limit) | unmeasured; small and nearly free to try |
| 2 | **Fog compiled in only when it's on.** It's off as the game is launched and in the finish all the same | 84 instructions; perhaps some of the finish's 6 registers | a second copy of the program for fog on, as cover by faces has | unmeasured |
| 3 | **One ray-start pattern, not four.** The Render page's "Ray start" keeps all four in the program | up to 92 instructions | the three patterns David doesn't pick; his to say which stays | unmeasured |
| 4 | **The numbers of two levels kept in hand** | up to 30 of the 98 held values | the levels' table read at each step again; it was kept in hand for the marks where the card waits, and the table has since become cheap to read (a uniform) | unmeasured; the biggest single holding, and the one most worth an audit copy |
| 5 | **The height table's three rows kept in hand** | up to 15 held values | the rows read again as the ray moves | unmeasured; the third row alone was 6 registers |
| 6 | **The edge cases of placing a read** (`spot_soft`, three copies) | up to 920 instructions, and working values at the fullest step | the textures made to answer those cases themselves (rock baked under the floor's cells, every brick textured, a level's first column padded), so the march keeps one plain path | unmeasured; a change to how the textures are made |
| 7 | **The hand-over's second read** | 14 registers, 1,138 instructions, 4 to 14% of the march's time (as a ceiling) | something has to stand in for the blend between levels, and it's a look: without it 3 to 11% of pixels change. David sees rings where levels hand over before anything else | measured (room, time, picture); what stands in isn't designed |
| 8 | **The last step's look kept whole** | 13 held values | a step gathered from one of its ends instead of both: a look (gathering from both ends is what made the picture stop depending on where a stride fell) | unmeasured |
| 9 | **The finish in a pass of its own** | 6 registers, 475 instructions | every ray's answer written to memory and read back by the new pass (the cut's join already finishes its rays this way); alone it read slower in the audit | measured for room; time not understood |
| 10 | **The floor's cells** | 6 registers (9 held values) | nothing at the five marks' pictures; alone it read 5 to 12% slower in the audit, not understood | measured; needs flying again before it's trusted |
| 11 | **The sky behind the ray worked out at the finish, not inside the march** | 84 instructions | none known | unmeasured |

Not candidates: the skip (a third of the time, none of the room); the third row (the sea needs it); the cut
(no room, and it took a quarter to two fifths off the waiting marks); cover by faces (one register, David's
look).

**What the audits say about chasing room at all:** fitting 16 groups for 12 was worth 2 to 7% where the card
works and cost 3 to 4% where it waits. So room is worth having to put something new in, more than for its own
sake. The march sat two registers under the line where 12 groups stop fitting when this was written; lazy rays
and cover by height (both shelved, `lazy-rays.md` and `cover-by-height.md`) would each want some.

## 6. How to take all this again

**The tree and the held values** (no window, no game; about a minute):

```
FAR_TEST_SHADER_TEXT="<a folder>" FAR_TEST_BLEND=1 cargo test --release -j 4 the_far_eye_draws_ground_and_sky
python "kept logs/march/march_parts.py" "<the folder>" march_all.compute.hlsl
```

The test keeps each program's text as the card's compiler is given it (`march_all.compute.hlsl` is the march once
the textures are in; `march_all_1` the same with cover by faces the other way; `march_loading` the whole march).
The script compiles the text as the game's compiler does, checks that the copy with debug marks is the same
program instruction for instruction, and prints the tree and the held values. It names the march's own blocks by
patterns in the text (the list at its top): if the march is rewritten, look at those first. Build 194's texts and
trees are kept in `kept logs/march/b194/`.

**A part's room and time** (needs David and Nsight): `INCUS_FAR_AUDIT=1` or `2` compiles cut-down copies of the
march beside it and marches each over the base layer, so one trace names each one's registers and time
(`far_eye.rs`, `audit_variants`); `INCUS_FAR_AS=<a copy's name>` marches one as the far eye's own program, its
picture wrong on purpose, so a flight prices it; `FAR_TEST_VARIANT=<name>` draws one off the screen against the
whole march's picture. A new candidate gets a copy there first.

**The march's row** (its size on the card): a trace at the mountain top and David's reading of `march_all` in
Nsight's window, the Shader Pipelines tab: # Warp, # Reg, Smem, Live Registers. An off and an on trace for any
new rule in the march.

## 7. What to remember when reading it

- Size on the card is the fullest moment, not a sum. One part out can free nothing; several together, a lot.
- A heavy path few steps take costs every ray in its group.
- Sixteen groups bought with words put by in memory cost where the card waits.
- Two flights of one program differ by 1%: price a change with a bracket (off, on, off), not one flight.
- A count of steps or instructions has predicted a flight wrongly more than once. Each candidate here needs its
  own trace and flight before it's believed.

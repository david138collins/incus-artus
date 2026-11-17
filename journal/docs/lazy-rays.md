# Lazy rays: shelved, and how to pick them up

Written 2026-11-16 by Builder, at David's word, so that a later session can take lazy rays up cold. It packs the
instructions (the Oracle's), what has been measured, the design Builder proposed, and the decisions left with
David. Nothing here is built. Where the far eye stands today is in `master/plan.md`; the flights and pictures
behind every number here are in `journal/docs/experiments.md` under the dates given.

## 1. Why it's shelved, and when to take it up

**David, 2026-11-16,** after the lazy rays talk was opened with a measurement and a proposal: "lets hold off on
lazy rays for now. we can keep that in our back pocket if we ever get a performance squeeze limited by long rays.
i think we can move on. just pack the instructions and your output there into a lazy rays doc to resume with if
we ever need it."

So: not built, not refused. **Take it up when the far eye's time is the thing in the way and the far part of its
rays is where that time goes.** Before taking it up, find which kind of squeeze it is, from a flight's report:

- **A mark where the card waits** (the plain, the sea: its time follows its longest chain of steps). That was what
  the cut was for (builds 183 to 190). If long chains are back, look at the cut first: the list of long rays
  filling up (4,096 places; it filled on the run with the scatter stage on, build 189's flight), or a length
  nothing cuts.
- **A mark where the card works** (the mountain top, high over the spawn: its time follows the thread-steps held,
  the "threads kept waiting" line). Lazy rays are the answer on file for these: they take away far work on the
  frames it isn't redone.

## 2. What "lazy" means here

Far land changes slowly on the screen as the player moves, so its rays needn't all be marched every frame. Three
forms were told apart on 2026-11-14 (Builder's assessment, sent to Counsel at David's word):

| form | what it is | where it stands |
|---|---|---|
| A. A step budget, resumed next frame, on the screen's squares | David's first wording | Not buildable as worded: a ray resumed after the camera turns is no longer its pixel's ray. What's kept has to be kept by direction, which is C |
| B. The long ray cut into lengths, marched side by side in the same frame, joined front to back | the Oracle's step 4; no lag, nothing kept between frames | **Built: the cut** (builds 183 to 190, on as the game is launched). It took the waiting marks' time down by a quarter to two fifths |
| C. Far lengths kept between frames, by direction | the near part of every ray each frame; the far part kept, redone as the eye travels | **This document.** Shelved |

**David's own words for it, over time:** "spread out any rays that can be lazy" (his lead, in the plan);
"well the goal should always be to bring down the high spot. i was just about to ask if it is time for lazy rays"
(2026-11-14); and of the cut, 2026-11-15: "it would only be staying in because lazy-rays are a different form of
it, as i understand it?" (Builder: yes; the cut's lengths, their stacks and the join are what lazy rays stand on.)

## 3. The instructions: the Oracle's rules

From `journal/vision/oracle-review.md`, its third part, section 3 ("Lazy rays"). Read it whole before building;
this is its skeleton.

**The idea is sound** and has a forebear: Regan and Pose's "priority rendering" (1994) kept the surroundings as
pictures by direction round the head, redrew the far ones less often than the near, and read them through the
head's turning every frame. (Also layered depth images, Shade and others, 1998.)

**What it buys** (the Oracle's arithmetic): a quarter of an 8 x 8 pixel is 6 blocks of travel for land 2 km off,
30 blocks at 10 km. At a player's top speed (200 blocks a second, 240 frames a second) that's a redraw every 7th
frame at 2 km and every 36th at 10 km; at a walk, some thirty times rarer.

**Nine ways it goes wrong, and the rule each gives:**

1. *Restarting starves.* A ray in flight always finishes, and the next begins from where the eye is then. Two
   copies of the far picture: the one shown and the one being made.
2. *"Kept by direction" and "pixel for pixel" can't both hold.* The test becomes: the store, once whole, equals a
   full march of the store's own directions.
3. *The far squares behave differently when turning.* Kept by direction they're fixed to the world and turn with
   it. Two ways to show it: the store finer than the screen's squares and summed into them, so edges glide as now;
   or its squares shown as they are, riding with the world. **David's to choose** (section 7).
4. *Only a ray's far end can be lazy.* Laziness goes by stretch of distance, not by ray: near lengths every frame,
   far ones in turn, joined front to back each frame. That's the cut, which is built.
5. *Light kept in a lazy picture crawls.* Keep what's there (colour, faces, cover, distance) and put light and fog
   on every frame.
6. *What's hidden now is wanted next.* Far land behind a near ridge is marched anyway, or it's a hole when the
   ridge slides aside.
7. *Behind you is kept warm.* A full turn takes a tenth of a second: the whole ring of directions is kept, the part
   out of view at a lower rate.
8. *Far land moves in steps.* Shown all at once, it's even; shown as each direction finishes, it boils along
   edges. David's rule "even beats irregular" says all at once, the work spread evenly through the frames between.
9. *The marks stop measuring.* Standing still, a lazy far eye costs almost nothing. The gate is read moving (a
   walk, a run, turning, nodding) at the 99th, and once with laziness switched off.

**Two more to carry:** things that move far off (lights that twinkle, figures, weather) can't live in a lazy
picture: they're drawn each frame over it, against its kept distance. And a stale picture hides bugs: it needs an
age view (each pixel tinted by how long since it was made) and a flight that checks the store against a full march
at random frames.

**Its four tests:**

1. The store, once whole, equals a full march of its own directions.
2. Lag: the longest any shown result has waited, per length, at a walk, a run, turning and flying; and a count of
   rays restarted before finishing, which should be none.
3. Card time on moving marks at the 99th, against today's in one session, laziness on and off; and the frame's
   evenness while speeding up and slowing down.
4. David's eye, three looks: turning on the spot; walking with a near ridge sliding across far land; the sun's
   step arriving.

**Its smallest version:** "the farthest one or two lengths of every layer, kept by direction": small, where the
tolerance is widest, and it shows David the one thing only his eye can judge, far squares that hold still on the
world. (Builder's note of 2026-11-14 on this, with numbers: the farthest lengths hold almost none of the work, so
that version proves the look and the mechanism and saves next to nothing. See section 4.)

## 4. What has been measured

Three measurements, each with no change to the picture's code. All on the laptop's NVIDIA card, 2,560 x 1,600.

**a. Where a mark's steps lie, by level** (2026-11-14, build 176, the base layer; the 8s reach to about half a
kilometre, the 16s to about one, the 32s to about 5 km, the 64s to about 11):

| | all its steps | in the 8s' ring | the 16s' | the 32s (1 to 5 km) | past 5 km |
|---|---|---|---|---|---|
| the plain | 1,146,000 | 62% | 10% | 19% | 9% |
| the sea | 1,285,000 | 68% | 12% | 15% | 5% |
| the mountain top | 2,027,000 | 54% | 17% | 23% | 7% |
| high over the spawn | 1,522,000 | 40% | 33% | 22% | 5% |

Past about 21 km: under half a percent of the steps. A long ray's steps are mostly in the 32s and the 64s, 1 to
11 km out, not at the far horizon.

**b. The near kilometre alone and the rest alone, flown** (2026-11-14, build 172; the card's whole, ms):

| mark | whole | rays ended 1,024 blocks out | rays begun 1,024 out |
|---|---|---|---|
| the mountain top | 1.042 | 0.401 | 1.062 |
| high over the spawn | 1.015 | 0.622 | 0.855 |
| low over the sea | 1.401 | 0.403 | 1.247 |
| the bare plain | 0.972 | 0.380 | 1.177 |

(About 0.10 of each figure is the two passes that don't change. The "begun 1,024 out" flight draws another
picture, since the near land is gone and the finer layers march where they didn't: it over-reads, and was written
down as a fault in the measurement's design.)

**c. The work that lies beyond a distance, at the two marks where the card works** (2026-11-16, build 194, off the
screen: the thread-steps held, which is what the card's time follows there):

| rays ended at | the mountain top: held | of the whole | high over the spawn: held | of the whole |
|---|---|---|---|---|
| (whole) | 7,465,984 | | 7,472,640 | |
| 8,192 blocks | 6,436,128 | 86% | 6,925,568 | 93% |
| 4,096 | 5,513,088 | 74% | 6,351,264 | 85% |
| 2,048 | 4,258,432 | 57% | 5,361,344 | 72% |
| 1,024 | 2,149,408 | 29% | 4,617,504 | 62% |
| 512 | 1,138,816 | 15% | 1,302,048 | 17% |

So the work beyond a distance, **at the most:** beyond 1 km, 71% of the mountain top's and 38% of high over the
spawn's; beyond 2 km, 43% and 28%; beyond 4 km, 26% and 15%; beyond 8 km, 14% and 7%.

**These are ceilings, not savings.** A ray ended short finds no surface, so the finer layer isn't marched for it
at all and its near steps are dropped with its far ones (the far share is over-counted); and a lazy scheme still
marches the far part on some frames, redoes it as the eye travels, and keeps books.

**How to take (c) again** (the switch is `INCUS_TEST_REACH`, declared in `src/wiring.rs`; `INCUS_TEST_BEGIN` is
its twin, a ray's far part alone). For each mark and each distance, and once without the switch:

```
INCUS_TEST_REACH=2048 FAR_TEST_FRAMES=10 FAR_TEST_SCREEN="2560,1600" FAR_TEST_BLEND=1 FAR_TEST_REGIONS=all \
  FAR_TEST_MARK="the mountain top on a clear day" \
  cargo test --release -j 4 the_far_eye_draws_ground_and_sky -- --nocapture
```

and read the two lines it prints: "The rays: ..." (each layer's rays and steps) and "Threads kept waiting: ..."
(the layers' and the strip's steps held by 32s; add the two). As a flight (the card's time itself), the same
switch with `INCUS_FLIGHT=1` and `cargo run --release -j 4`, flown as a bracket (whole, ended short, whole).

**Where the far eye's time stood when this was shelved** (build 194's flight with cover by faces on, the card's
whole, ms): the mountain top 1.06, high over the spawn 1.12 (the two where the card works, and the dearest); fast
across 0.76, the run 0.74, the sea 0.66, the sea nodding 0.65, the plain nodding 0.58, the plain 0.57, the spawn
0.48. David's reading of the far eye's gate
(2026-11-13): "if everything in the far view is under 2ms we should be gold".

## 5. The design Builder proposed (2026-11-16)

Put to David in plain terms at the talk; he shelved it before answering its two questions.

- **The near part of every ray is marched every frame, as now,** and stops at a set distance: the lazy line.
- **Everything beyond the line is kept as a picture all round the eye, fixed to the world** (by direction), with
  what's there and not its light: colour, faces, cover, distance. It's made from where the eye stood, refreshed a
  share each frame, and made again as the eye travels far enough to shift it (the Oracle's quarter of a pixel).
- **Each frame the far picture is put behind the near one,** read along each pixel's direction, with light and fog
  put on fresh.
- **The march's program doesn't grow.** Near rays simply end at the line; the far picture is made by a separate
  compiled copy of the march; it's laid in by the pass that already brings the picture up to the screen.

**One step a build:** the lazy line far out first (about 4 km: it saves little, but it's the safest place to
judge the look, and it's the whole mechanism); then brought in (2 km, 1 km), each with David's three looks and a
flight, for as long as it looks right. The saving is in the last steps, not the first: by section 4, what lies
past 4 km is at most a quarter of the mountain top's work and a seventh of high over the spawn's.

**What it would stand on, in the code as it is** (`src/`; check each still exists):

- *Ending rays at a line, and beginning them there:* the two measuring switches do exactly this today
  (`INCUS_TEST_REACH`, `INCUS_TEST_BEGIN`; `far_eye.rs` reads them into the camera).
- *A second program from the one text:* `far_eye.rs` already compiles the march several times with one line of
  `far.wgsl` changed (`LOADED`, the march once the textures are in; `FACES`, cover by faces, which the Render
  page's tick box swaps live). The far picture's maker is one more, its outputs changed as `far_all.wgsl` changes
  them for the one dispatch.
- *What a far length keeps, and joining front to back:* the cut's stacks (`far_all.wgsl`: what a length gathered,
  the sun's part kept apart, what it let through, its first surface and that surface's level) and its join pass.
  For lazy rays the stack has to keep colour, faces, cover and distance without the light: the gather lights as it
  goes today (`faces_light`, the sun's way), with only the sun's part kept apart for the first surface's shadow.
  That reshaping is the first real piece of work.
- *Laying it in behind:* the far eye's layers are kept with their alpha (premultiplied), and `far_up.wgsl` brings
  them up; the far picture goes behind there.
- *Light every frame:* a finished ray is lit once, by its first surface (`finish`, `shaded`), from the far light
  read from the cell; the far picture's pixels would be lit the same way, from their kept surface.
- *Even refresh, two copies:* nothing exists; the far light's sweeps (made over frames on the processor, swapped
  whole) are the nearest pattern.

**What must not happen:** anything added to the march's own program. Its size on the card, read by David off
Nsight's window (the `march_all` row: # Warp, # Reg, Smem): 168 registers, nothing put by, 12 groups, two under
the line of about 170 where 12 groups stop fitting (builds 192 and 194). A few values carried through its loop
cost 4.5% at the working marks once (build 185).

## 6. Sizing notes from the 2026-11-14 assessment that still bear

- **The far part begins where the work is, and that's near.** If the far part begins at about 1 km (the 16s'
  edge), it holds 20 to 30% of the base layer's steps and is redone every 3rd or 4th frame at a run; at about
  5 km, 5 to 9% and every 19th; at about 11 km, 1 to 3% and every 38th. So the version that saves something is the
  hard one for the eye: land 1 to 5 km off is big on the screen, holds still on the world while the near ground
  moves every frame, and at a run is redrawn every third or fourth frame.
- **What's hidden now is marched anyway:** at 1 km most far land is behind near ground. Cheap when lazy; not free.
- **Memory:** by direction at the 8 x 8's pitch, all round and 30 degrees up and down, about 46,000 squares a
  length; with the 4 x 4's, tens of MB. Not a constraint.
- **The cut's lead isn't turned for the camera's turn** (as Builder read the march then): a nod or a turn moves
  the band of long rays off its lead for a frame. Worth checking before building on it.

## 7. What's left with David

Asked at the talk, not answered (he shelved the whole):

1. **Far squares fixed to the world, or to the screen as now?** Builder recommended the world: it's what keeping
   the far land as a picture round the eye gives, and turning the head then shows no lag, since the far land is
   already there in every direction. The visible difference: far squares ride with the land instead of the land
   sliding through them. (This is also the plan's open question, "Lazy rays' far squares fixed to the world or to
   the screen?", and the Oracle's item 3 above, with its two ways of showing it.)
2. **Refreshed all at once, or piece by piece?** Builder recommended all at once, the work spread evenly over the
   frames between, by his rule that even beats irregular.

And his looks decide each step: turning on the spot; a near ridge sliding across far land at a walk and a run;
the sun's step.

## 8. What Builder doesn't know

- What it would really save. Section 4's figures are ceilings; nothing of C has been flown.
- Whether land 1 to 5 km off, held still on the world and redrawn every few frames at a run, passes David's eye.
  It can't be judged on a far strip first, because the far strip has nothing in it to save.
- What reshaping the gather (colour, faces, cover and distance kept; light put on after) does to the march's size
  on the card and to its picture. It touches the march's loop.
- How the far picture is read when the eye has moved from where it was made: by the kept distance, with what
  error at the lazy line, where far meets near.
- Whether the near eye (plan 2.3) changes the question. It hadn't been built when this was shelved.

## 9. To resume

1. Read `master/plan.md` ("Where we are"), this file, and the Oracle's section 3.
2. Say which squeeze it is (section 1), from a flight flown that day.
3. Take measurement (c) again at the marks in question: the far eye will have changed.
4. Open the talk with David with those numbers; ask him section 7's two questions.
5. Write the first step's pass mark before any code (the lazy line far out; the store equal to a full march of
   its own directions; an age view; nothing added to the march's program, shown by an off and an on trace; flights
   read on the moving marks, bracketed, laziness off and on).
6. One step a build, his three looks at each.

## 10. A reference found since (Counsel, 2026-11-17; David: "yes", add the pointer)

Dylan Ebert's "texel splatting" (https://dylanebert.com/texel-splatting/; a paper on arXiv, 2603.14587): the
scene drawn from a fixed point near the eye into low-resolution cube pictures, lit there, then each texel drawn as
a small square fixed in the world. It is this document's "picture all round the eye, fixed to the world", built
for 3D pixel art, and it answers two of section 7's open matters in its own way:
- **Crossing into a new cell:** the probe's origin is the eye snapped to a world grid, so it changes only at a
  cell's edge, and the old and new pictures are crossfaded by a 4 × 4 dither.
- **What the fixed point couldn't see:** a second picture taken at the eye fills the holes; those patches
  shimmer, and he names it the main limitation.
Not solved there: moving and animated things. Its cost as he gives it: under 4 ms a frame on an RTX 4090 at 384
texels a face. A reference for the mechanism, not a design to adopt.

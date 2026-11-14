# A prompt for the Oracle, at the finer cells  [Counsel's draft for David, 2026-11-14]

David asked for this: "i believe we should summarize our progress and discoveries since our last consultation with
the oracle, and ask about how we could handle precisely the next few steps, and to look for any clever or novel ways
we could get something for nothing." It's his to edit and send. Everything below the line is the message.

---

Oracle, it's David. Your third part ("waiting, not working") set our order two days ago, and we've followed it. The
far eye met my bar: everything under 2 ms, typical and worst. Light moved to the processor and costs almost nothing
on the card. Then we began your section 7, the cells of 2 to 16, and the second ring found the wall you named in your
first part: rays that skim the ground. I'd like your read before we build the last two rings, and I'd like you to
hunt for anything that gets us something for nothing.

## Please read, in this order

1. `master/plan.md`: "Where we are", step 2.2's lines (each ticked with its proof), "Open", "The targets", and "What
   every step keeps". Then `master/map.md`.
2. `journal/vision/oracle-return.md`, section 2: one row per step since your review, with what you expected, what we
   measured, and what it changed. The rows from "your step 0" on are new since your third part.
3. `journal/docs/experiments.md`, from build 142 on: each pass mark, written before its code, then its results, misses
   included.
4. `journal/docs/decisions.md`, the entries dated 2026-11-13 and 2026-11-14: my words.
5. `src/far.wgsl`, `src/far_all.wgsl`, `src/far_eye.rs`, `src/far_ring.rs` and `src/atlas.rs` as they stand at build
   172, and the flight logs the rows cite (`logs/`).

## What we did with your order

| Your step | What happened |
|---|---|
| 0: the longest ray, the nodding marks, the Intel card | Built (142). Cost follows the longest ray, as you read. The Intel card is **working, not waiting**: cost follows how much is marched, 4 to 8 times the NVIDIA's. Your warning held: 1a made the NVIDIA 6 to 13% faster and the Intel 2.5 to 10% slower |
| 1a, kept in hand | Kept (NVIDIA decides, the Intel informs: my rule now) |
| 1b, read together | 2.5 to 7% off, not your 15 to 40%. Kept |
| 1c, several steps at once | 11 to 25% *slower*, its code costing even off. Dropped. Your own test called it: the card doesn't overlap reads as you believed |
| 1d, thin haze by whole cells | The best half: the sea 2.22 → 1.78 ms. Nothing visible, as you said |
| The 4 × 4 base, your probe | Not clean: four times the rays cost 1.9 to 3 times the time. "Waiting" holds for today's layers but ends somewhere between 64,000 and 256,000 rays |
| 2: the layers side by side | Three dispatches in one pass gained nothing (they ran in turn). Your second form, **one dispatch over all three layers**, took 33 to 38% off the worst marks; trimmed, arranged and packed back under the library's default limits |
| Light read from the cell, made on the processor | Built as you drew it (build 160): each column's sun-shadow height by a sweep along the sun's way, sky openness from horizons in eight directions, 40 MB. The rays' light had grown to 11 to 56% of the far eye; the column light costs what no light costs. A sun change shows in about 0.3 s; the sun snaps at sixteenths of a day ("a little clock for the player"). I A/B'd them: "column light is actually cleaner, i prefer it". The rays' light is cut (206 of 1,354 lines) |
| 3: the gate | Met by my bar (build 159). The rings where levels hand over: "i can't see any rings". Owed only the Intel card's numbers |
| 4: the long ray cut into lengths | Shelved when the gate was met. Back now (below) |
| Fog | Off by default, for looks only: "we are not using fog or haze or any sort of cover as a performance tool" |

## What we found on the way

1. **Waiting, measured every way.** The card at 93 to 99% idle in the long passes; the watts low at full reported load;
   the longest ray setting each layer's time. Your reading held. Your step estimates mostly didn't (1b, 1c), and
   neither did ours (below).
2. **Code costs even switched off,** three times in two days: 1c's, a one-line flag's compare (1.2 to 3.6% of the
   march, given back when the flag was cut), and the ring's code (2 to 3% with the ring off).
3. **The look up close is now set by the blend.** With light settled, what I see near a mountainside is the blend's
   creases: star-shaped facets and "whorls". Where rays begin (a weave, blue noise made at load in 0.09 s) changes
   only the speckle. An eased blend "softens them but doesnt change them". The flat straight-edged plates I saw were a
   switch to bare unlit rock where a cell held almost no surface: cut. Empty cells now borrow colour from the nearest
   surfaced cell in the world, not only their own brick.
4. **Pictures must look every way.** Builder's checks had never looked straight down, and I found a square round the
   eye that way: no brick just under ground lying at a brick's floor, so the blend fell back. Mended in every level
   for 3% more bricks and no more memory.
5. **Night flights lie** when the laptop drives a TV and a video on the same card (4 to 18% slow, the 99ths
   unreadable). Day flights with nothing else on the card matched the afternoon's to 0.01 ms.

## Where we stand: the cells of 2 to 16, two rings in

The 16s in a ring of 128 × 128 columns round the eye (old's size), then the 8s inside it, each made on a thread of
its own and moved as the eye moves (a ring remade in 0.06 to 0.5 s, off the frame; the first at load 1.4 to 3.6 s). Every coarser cell is exactly its
eight finer ones. Near slopes and skylines are crisper, and the star facets shrink with each ring. "looks pretty
good."

The card's whole, median and 99th, ms (build 171, both rings; NVIDIA only, one screen, hands off):

| Mark | median | 99th | against build 167 (no rings) |
|---|---|---|---|
| on the ground at the spawn | 0.43 | 0.43 | +46% |
| the bare plain | 0.96 | 1.01 | **+26%** (the 16s alone: −31%) |
| the plain, nodding | 0.97 | 1.04 | +27% |
| high over the spawn | 1.01 | 1.03 | +23% |
| the mountain top | 1.05 | 1.09 | +2% |
| low over the sea | 1.39 | 1.44 | +22% |
| the sea, nodding | 1.36 | **1.63** | +31% |
| a player's run | 1.30 | 1.42 | +16% |
| fast across the world | 0.91 | 1.17 | +16% |

Card memory 1,015 MB in use, unchanged by the rings. Main memory about 3.0 GB held, 1.2 GB of it in memory.

**The wall.** The 8s took the bare plain from 0.54 to 0.96 ms, where Builder guessed within 10%. Standing on the
ground, the base layer's grazing rays walk the 8s' small steps for some 500 blocks before their far part: 921 rays over
256 steps, where none were. Each finer ring halves a grazing ray's step near the eye. If the 4s cost the plain what the
8s did it stands near 1.7 ms, and the 2s pass my bar. Over the sea the 4 × 4's longest ray already reaches the step
limit (768).

**Where we are this hour.** I asked for "a deep dive into how we can get the finer rays working simultaniously in time
with the rest of them." Builder's reading: the rings add no short rays, they lengthen rays already long, so the near
work wants to run beside the far: your step 4, each ray cut at the rings' edge (1,024 blocks), the lengths marched side
by side in the one dispatch, joined front to back. Whether it pays turns on whether the card is waiting or working on
the plain's frame. Build 172 measures that first, nothing built: the whole, the near length alone, the far alone.

One correction of ours: Counsel first named your column-height skip for this. Your section 4 says it pays only with
the skyline by cover, since these rays are summing the rings' haze. So my skyline choice (haze or cover), which we
moved to after the rings so I'd judge it on them, is tied to this cost.

## My bars and principles since your third part

- **Speed:** "if everything in the far view is under 2ms we should be gold"; worst and irregular frames weigh more
  than typical ones now.
- **The shape:** no megakernels; small passes over a per-pixel store, each measured alone ("establish an
  architectural goal that supports the microkernalization of it"). The one dispatch is one job launched once, and
  light and fog have moved out of it.
- **Memory:** card memory well below 4 GB at worst; main memory "somewhere between 4 and 8gb in the end"; the
  library's default limits for anything required to run.
- **Cards:** the NVIDIA and the 980 Ti decide, the Intel card informs ("i don't expect someone to be able to play
  this game on an igpu without turning some settings way down"). No 980 Ti machine bought: players with one will
  send their logs "by the time its important".
- **Room for modders, later:** "priority 1 is shipping incus artus, *my* recipe", but assembled "so later we can
  expose parts of it for modification". Nothing built for modders now.
- **Effects later, wanted now in the shape:** I'd like to bend the lens for magic (glows by ray length, a smooth
  datamosh, one thing pixelating, a wraith shattering from triangles into cells). A per-pixel tag for what each pixel
  shows would serve those and my pixel weights.
- **Old words weigh less:** what I said a week ago is a reminder, not a rule, unless it's about current work.

## The next few steps, as planned

1. Build 172's measurement, then the cut if it pays.
2. The 4s, then the 2s, each alone, each flown.
3. The scatter stage behind the doorway (your wood, posts, towers, slabs, arch), its marks, each shape placed with me.
4. The skyline talk, haze or cover, on the rings.
5. Lazy rays. Builder and I have a newer form of my idea: a step budget a frame, which trims the longest rays first
   without sorting (a group of 32 ends with its longest), a ray that runs out carrying on next frame. An instrument
   for the waiting inside groups comes first.
6. 2.3, the near eye: rays first (no boxes, both tables from the first build, light from the air cell), then faces as
   triangles, both flown on the same marks ("Yes, fly both").
7. The fork: one budget, read moving, at the 99th; you come back.

## Our questions

**The grazing rays and the rings**
1. Is your step 4's cut the right lever for rays lengthened by the rings, or is there a better one: steps sized by
   the pixel's footprint along a grazing ray (coarser as it goes out, since a skimming pixel covers a long strip of
   ground), the rings sized by distance rather than by count, a level chosen per step from the footprint, or
   something we haven't named? Which of these stay honest by your design (every coarse cell an exact sum, nothing
   popping)?
2. Does the skyline by cover now belong *before* the 4s, since with it your height table pays and skimming rays take
   tens of steps instead of hundreds? Or does the cut make the haze affordable, so the choice stays a pure look?
3. The 4s and 2s: precisely how would you build and fly them, given the plain's trend? Ring sizes, whether the 2s are
   worth it in the far eye at all or belong to the near eye's handover, and what each should cost.
4. If the cut is built: where to cut (the rings' edges, doublings of distance), how the joins stay exact, and how it
   meets lazy rays later (your section 3 said the cut comes first).

**Something for nothing**
5. The card sits mostly idle while the long rays wait. What work could ride in those waits at no cost: the near eye's
   march in the same dispatch, the finish, the rings' next contents, light, the per-pixel tag? Is "work is nearly
   free, waiting is dear" a design rule for the whole frame?
6. What else is idle: the triangle hardware, the processor's threads beyond the rings and the light (the simulation
   will want most of them), main memory up to my 4 to 8 GB, the copy engine, the frame before. For each: what could it
   do for us, honestly and portably?
7. Switches that cost when off, three times: is there a way to build the far eye so a switched-off path truly costs
   nothing (variants compiled apart, constants fixed when the program is built), and is it worth the extra programs?
8. Any cheap trick you know from the literature or from games that we're leaving on the table, for a far view this
   wide.

**The look**
9. The blend's creases (the star facets and whorls): is there a cheap blend that keeps the texture hardware's single
   fetch and loses the creases, or do the finer rings answer it by themselves?
10. Inside the rings the light is still the 32s' column light. Per-ring sweeps, or is the 32s' light honest enough
    there?

**Carried**
11. Your proposed split of one budget at the fork (far 1.2 typical, 2 worst; near 1.5; finishing 0.4; at least 0.9 for
    the rest): does it still stand, given the far eye's real numbers?
12. The near eye: what should it be built to carry from the start, given the per-pixel tag I want, room for modders
    later, and that dungeons will grow underground as the world ages (rooms written into the world as changes)?

## What I'd like back

First, reply with your understanding and your questions. When I've answered, write your review as a fourth part of
`journal/vision/oracle-review.md`, covering:
- whether the cut, the skyline by cover, footprint stepping or something else is the lever for grazing rays, and in
  what order with the 4s and 2s;
- precisely how you'd take the next few steps, each with a pass mark;
- every "something for nothing" you can find, with what it buys, what it costs and how sure you are;
- your answers to the questions above;
- anything we're doing that you'd stop, and anything we're not doing that you'd start.

# A prompt for the Oracle, midway through the far eye  [Counsel's draft for David, 2026-11-12]

David asked for this: "please summarize our work on the new engine, and the things we have left to try, and all our
research and findings into a prompt for the oracle, with our recent discussions and questions involved." It's his to
edit and send. Everything below the line is the message.

---

Oracle, it's David. We're calling you a second time before the fork you named. The new project is built through its
skeleton, the doorway and the far eye's first form, and we've tried four of the far eye's changes one at a time. One
paid in time, two in memory, and one, the heart of your design, didn't pay on this card. The far eye still costs up
to about three times your gate at the far marks, before the near eye exists. I'd like your read before we go further.

## Please read, in this order

1. `master/plan.md`: "Where we are", then Phase 2 (step 2.2's lines show each change with its proof), "Open", and
   "The targets". Then `master/map.md`, drawn from the code's own list of its parts.
2. `journal/vision/oracle-return.md`: Counsel's running brief since your review. Section 2's table has one row per
   experiment and step, with what you expected, what we measured, and what it changed.
3. `journal/docs/experiments.md`, from "Experiment 5" on: every step's pass mark, written before its code, then its
   results, misses included.
4. `journal/docs/decisions.md`, the entries dated 2026-11-12: my words.
5. The logs those cite, in `kept logs/`, if you want the raw reports.

## What exists now

Written fresh, old open beside it, nothing carried but results:

- **The skeleton:** a window, a flying camera, the developer panel (F3), and an editor (F2) whose first page is the
  software map, drawn live from the code's own list of its lanes, passes and stores. A test fails the build if the
  code and `master/map.md` disagree.
- **Clocks on every pass from the first build,** on the processor and on the card. The empty frame, settled: 0.34 ms,
  0.012 on the card.
- **The doorway:** answers "what's here?" a tile at a time from the exported test world, read-only. All 5,047 reference
  columns match old's to the block, about 2 ms a tile.
- **The marks as data,** each eye within a tenth of a block of old's; my build arrives as 169 changes through the
  doorway; a scatter dial. Flights stand at each mark and now also move, at a player's 200 blocks a second and at
  2,000.
- **The far cells,** summed on the processor from the doorway's columns: every one of 11.2 million the same as old's
  bake, byte for byte. The whole world in 17 s on 24 threads; old's bake took about 12 minutes on the card, though it
  also ran the world's recipe.
- **The far eye:** old's far march moved onto your bricks, with old's light (the sun's shadow ray and five sky cones),
  fog, sky, and the screen pyramid (an 8 × 8 base, finer layers where earned). It now has:
  - soft bricks blended by the texture hardware, your spread;
  - bricks of 8;
  - a window round the eye for the 32- and 64-block levels (the coarser levels whole);
  - the sky weighted more at skylines, fading as the land covers more (my pick, at no measurable cost);
  - its settings live on the Render page, each with a Default.

## What we measured

The far eye on the laptop's NVIDIA card, its median in ms. The baseline is old's march on bricks with old's light
(build 126, the code's blend, the whole world on the card). "Now" is build 136: bricks of 8, the spread, the window,
detail 0, your boxes off.

| Mark | baseline | now | change |
|---|---|---|---|
| on the ground at the spawn | 0.77 | 0.67 | −13% |
| the bare plain | 1.42 | 1.12 | −21% |
| high over the spawn | 2.38 | 2.29 | −4% |
| the mountain top, clear day | 2.74 | 1.99 | −27% |
| low over the sea | 3.95 | 2.93 | −26% |
| a player's run, 200 blocks a second | — | 3.26 | |

Your gate for 2.2 is the far layers under 1 ms typical and 2 worst, at the far marks (the plain's horizon, high over
the spawn, low over the sea). We're at 1.1, 2.3 and 2.9 ms there, standing.

**Card memory, the whole game:** 2.7 GB with the whole world's far view as soft bricks; **916 MB now**, with the window
and bricks of 8 (471 MB at bricks of 4). **Main memory,** measured for the first time (build 138): the game holds
2.8 GB private at the end, of which 1.0 GB is in memory; 1.3 GB is held at 1.6 s, before any far cells, which isn't
placed yet (drivers and the card's staging copies are likely). Builder is breaking it down stage by stage.

**Each change alone:**

| Change | What it did |
|---|---|
| 1, soft bricks by the texture hardware | 14 to 29% off at every mark. Your spread and Builder's exact blend looked the same to me, so the spread is kept: faster, and 1.1 GB lighter. The card's filter weighs the eight cells to about 1/256 |
| Bricks of 4 or 2 against 8 | 4 took the whole world's memory from 2.7 to 1.6 GB for 5 to 13% more time. Once the window was in, memory stopped binding, so 8 |
| The window round the eye | Memory from 1.8 GB to 471 MB (bricks of 4), standing time within 3%. Moving at 200 blocks a second it keeps up, but swapping a whole region in at once spikes the worst frames to 8 to 9 ms. At 2,000 it creaks (worst 22 ms), which I accept |
| 2, rays begun at drawn boxes | The picture the same; a quarter of the steps cut. But they were the sky's cheap ones: the boxes' pass costs 0.17 ms and saves 16 to 17% of the 8 × 8 at the plain and the mountain top, 1 to 3% over the sea and high over the spawn. So the plain breaks even, the other marks are 1 to 7.5% slower, and the spawn 24% (the pass alone). Off by default; its unused code costs under 1% |

## What we learned

1. **The far eye's time is the march where rays graze land near the horizon.** The steps' view shows a thin band at the
   horizon and the sky just above it, where rays skim land for tens of kilometres; the ground is cheap. Most of it is
   in the 8 × 8 layer: low over the sea, 1.2 of 2.9 ms. Your guess of 2026-11-07 was right about where the steps are,
   not about where the time is.
2. **Light is cheap in the far eye:** 3 to 13% of it at the far marks (old's whole eye: 17 to 42%). It's worked out
   once a pixel, and nearly every far pixel is an 8 × 8 or 4 × 4 square. Change 3 can win at most about 0.2 ms there.
3. **The finest layer (2 × 2) is almost never earned from far cells alone.** Two levels finer looked better to me at
   first, but from height it cost up to 77% more, as the 2 × 2 switched on. Back to 0.
4. **Code in the program costs even when switched off,** sometimes: change 1's hardware path made the code's blend 1 to
   8% slower; the boxes' unused code costs under 1%. Builder now prices it.
5. **The method:** flights within one session, alternating, agree to about 1%; between sessions only to about 5%. A
   session's first 25 seconds run slower, the card too. Each check is made to bite by breaking one input on purpose.
6. **Card limits:** Builder raised the storage buffers a pass from 8 to 16 for the window, then folded the extra
   flags into an existing buffer, so the far eye runs at the default every card allows.
7. **Two misses caught by our own instruments:** the camera had flown at 1% of its speed since the rebuild began (the
   moving flight's report showed it), and an edge brick wrote past its row (an edge test now guards it).

## My bars and principles since your second part

- "try all these things and gather data about the performance impacts they have, and let me take a look at how they
  look while flying and while still/moving my head." And: "we'll need to assess the costs scientifically and weigh
  their effects architecturally."
- **Card memory:** "keep the running frames well below [4 GB] in memory in worst cases". It may grow with the world's
  size, never with play.
- **Main memory:** "somewhere between 4 and 8gb in the end", with everyone having at least 8 GB.
- **Speed:** "if the game creaks at 10,000mph that's fine. as long as its not aggregious and it's solid as a rock when
  i stop and act more like a player." The laptop's budget at 240 is 4.2 ms a frame, the whole eye.
- **Idle hardware:** "if something in the generic hardware resources we are using is being underutilized, we should
  ask, 'what can *you* do for me?'" Counsel's reading: what the game must have to run stays within the smallest
  machine's budget; what only makes it faster or finer grows into a bigger machine's free memory and drops away on a
  small one, with the same picture either way. Portable features only.
- **Precision:** "the exact colors at any point or few points isnt a big deal. itll all be tuned again anyways. as long
  as it fundamentally works right and meets our goal for the far views function and appearance."
- **Fog:** "we may delete fog and try another way to implement it later, since it's only for looking nice."
- **Old words weigh less:** what I said a week ago is a reminder, not a rule, unless it's about current work.

## What's left to try

The candidates as we see them, none built. Your view on which to fly, in what order, is the main thing I'm asking.

- **Change 3, light read from the cell.** At most about 0.2 ms far; likely worth more in the near eye.
- **The open-sea shortcut,** held until a sea mark showed it was needed. Low over the sea is now the worst far mark.
- **The base layer's size.** Old's reading flight (1.7) found its coarsest layer cheaper at 4 × 4 than 8 × 8 at every
  standing mark (16% at the mountain top). The new eye's 8 × 8 is where the horizon's time is. Not yet tried.
- **Temporal reuse (TAA),** by my rules of 2026-11-08: anything fast never takes from history; pixels with no history
  get rays first; edits and light changes drop history; near block faces stay crisp; Playdead's INSIDE talk is the
  reference. Your design needs no reuse between frames. It might be the lever for grazing rays, or it stays for the
  speed pass after the eye is whole.
- **Your E, the far picture fixed to the world,** held in reserve: turning costs nothing, and the cost follows blocks
  travelled.
- **Finer layers earned along edges** where neighbouring squares differ. Near ridges against fogged far mountains
  show the 8 × 8's squares (my "ugly line"). A look fix, with a cost.
- **The window refilled a brick at a time,** not a region, to remove the spikes at a player's pace.
- **Spare main memory:** prepared bricks kept ready, so refilling the window is a plain copy; the tiles round the eye
  kept unpacked.
- **What a raised limit could buy** (16 storage buffers) on cards that allow it, with the default kept as the
  fallback.
- **Inspection:** the far march's registers and occupancy in Nsight; sorting rays by length into warps; each program
  priced with the library's checks on and off (in old, checks off made the heavy views 31 to 49% *slower*).
- **The Intel card's flights,** last in 2.2.
- **The gate's fallback,** if we're still at twice it: keep the bricks and the light, and march far the lean way
  (your design B).

## Our questions

**The far eye's time**
1. Where would you look first for rays grazing land near the horizon: a 4 × 4 base, temporal reuse, the open-sea
   shortcut, your E, or something we haven't named?
2. Change 2 didn't pay on this card. Does that change design D, or only how the far eye starts its rays? Is the
   triangle hardware still the near eye's way in?
3. Is 2.2's gate still the right shape (1 ms typical, 2 worst for the far layers), given the far eye's cost sits where
   it does and the whole eye's budget is 4.2 ms? Or should the far and near eyes share one budget, read at the fork?
4. Should temporal reuse come in now as a design choice, or stay a lever for the speed pass?

**Memory**
5. Required memory against spare: which of your levers trade memory for time best on a big card, and is a cache that
   changes size with the card honest under "1 always equals 1" and "no per-card tricks", if only the time differs?
6. Main memory, with my 4 to 8 GB (2.8 GB held today, 1.0 GB of it in memory): which parts of your design should
   lean on it to take load off the card?
7. The texture filter's precision is the card maker's choice. Does a hair's difference between cards, with the same
   data and method, fit "no per-card tricks" as you meant it? Builder compares one mark on both cards at the Intel
   flights.

**The look**
8. The fog line: are finer layers earned at edges the lens's way, or is there a better one?
9. Distant ridges past the window now read 64s where they read 32s, up to 35 to 49 of 255 apart on far ridges. I
   didn't notice it. Is that within your design's honesty, or should the window reach further?

**The near eye (2.3) and beyond**
10. What should the near eye do differently, knowing what the far eye taught us: light is cheap per pixel when pixels
    are big, boxes didn't pay at a distance, the time is at grazing angles?
11. The new steps' view puts the plain's far cost in the horizon band, as Builder guessed for old's bare plain (9.66 ms
    there, near and far together, and never placed by old's one clock). Does that change what the near eye's marks
    should be?
12. Your earlier questions still open in the brief (section 4): the near walk beside builds, near light from the air
    cell in front, the small card's full-screen passes, the world's format and the tablet. Any of them changed by
    what you've read?

## What I'd like back

First, reply with your understanding and your questions. When I've answered, write your review as a third part of
`journal/vision/oracle-review.md`, covering:
- where the far eye's time is, and what you'd do about it, in order;
- whether design D holds, changes, or gives way to B or E, and why;
- what the near eye should be, given this;
- your answers to the questions above;
- anything we're doing that you'd stop, and anything we're not doing that you'd start.

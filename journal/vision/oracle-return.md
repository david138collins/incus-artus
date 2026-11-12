# For the Oracle's return  [KEPT BY COUNSEL, from 2026-11-11]

The Oracle asked to be called back after experiment 6 (`master/plan.md`, step 2.3: the fork), "with the flight logs
and your own look". This file gathers what we learn from its review (`journal/vision/oracle-review.md`) to that day.

Counsel adds to it after each experiment and each decision of David's. It is sent as it stands at the fork, then left
as it was, as part of the journal.

**For the Oracle, when you read this:** read `master/` first, then this, then `journal/docs/experiments.md` (each experiment's
pass mark, written before it flew, and its results), then the logs it cites in `kept logs/`.

**Called early, 2026-11-11.** David called the Oracle back before 2.1, with this file as it stood after Phase 1. Its
answer is the second part of `journal/vision/oracle-review.md`, "before the first line". It answered questions 1, 3, 4, 6 and
7 below and corrected question 2. This file goes on gathering for the fork after experiment 6, as before.

---

## 1. What David decided after your review

All in his words in `journal/docs/decisions.md`: "The rebuild: after the Oracle", and "Where things are kept" under Platform
& Workflow.

- **A rebuild from scratch, with old as the reference, step by step.** "to create is human. to duplicate is divine.
  you don't know how to do something unless you can do it again from memory." Your "port" verdicts become "rebuilt
  with old open beside it". Old's code is copied in pieces only where David or the look says it's right.
- **Experiments first, in old,** as you advised.
- **The eye first, and rigour throughout.** After experiment 3's surprise: "it illustrates why we need to be rigorous
  and scientific while rebuilding incus artus ... how important it is to start with the graphics solution knocked out
  of the park. the rest of it is mostly fun and frolicking." He sees old as the prototype it took to get here: "sometimes
  there's just no way to make a thing right the first time and you have to go through prototypes."
- **The world's format, and old's world gen kept out of the new eye** (raised for planning; the export since
  decided, the format still open):
  - "when we build a new graphics engine from our research, i don't think we should necessarily use the old world
    gen as the way we're going to store and access the world will fundamentally change. i think it would be better if
    we could somehow just take nice25 and convert it the new format we're going to use, and use that for tuning the
    graphics engine, before going back and rebuilding the world gen stack with a finished graphics solution."
  - "the new world format is critical to performance. so, we don't really want the new graphics engine touching it.
    we'd need to establish exactly how the format is going to work in a way that lets me expand and refine the
    generation process. we're not quite at that point, but im bringing up the concern now so we can plan for it."
  - Then, deciding the export: "export nice25 so that we can use it as a 'generated' world to look at with the new
    graphics engine, before rebuilding world gen after the graphics engine is done."
  - See question 7.
- **Old's pictures are a reference, not a yardstick:** "still not sure what good the pictures are. it'll look good or
  it won't, idc what the old things look like in particular, other than the basic way we're doing things, like
  pixelating the far horizon." The 24 pictures taken from old (`master/references.md`) are kept to answer "was it
  like this before?". The new eye is judged by his eye on its own.
- **The near field:** rays "if they're cheap, and if that writes elegant compact code that is easier to manage";
  triangles for actors; "triangles for everything in the near field *if* it can look right and there are no
  jarring pops or seams."
- **Crisp:** the enlarging stays crisp (that was the box he'd always set to crisp). Your crisp-or-soft *cells* was
  another question: experiment 2 answered it **soft** (section 2).
- **Light:** sun shadows in steps of up to 30 seconds are fine. The day's length doesn't change how it's built. And
  after experiment 4b, on far land lit from its cell's kept light against its own rays: "honestly the difference is
  moot. it obviously changes a little, but i don't think i could tell the difference between the two in a blind
  test".
- **Order:**
  - the Registrar (and its Color Lab and HSL handling) before the Graphics Kit;
  - the editor made again, "more streamlined", its windows arranged on purpose and each setting and control placed
    with David first (2.1): "organizing the UI is super important to me";
  - world generation "we don't have to cling to"; water (sea level, cliffs and beaches, rivers) gets a design step
    of its own.
- **Where things are kept:**
  - `master/` (map, plan, spec) is what's true now, read first;
  - `docs/` and `vision/` are the journal (since 2.1, under `journal/`);
  - the agents' memory holds only habits and warnings.
- **Online:** your step 0 is ticked. The private GitHub repository is connected and pushed, and there's a USB copy
  tagged `before-rebuild`. `Art/`'s new files and `For Claude/` stay off GitHub for now.

### How your roadmap was changed

The plan is `master/plan.md`.

- **Experiments 1 to 4 fly in old.** Then old is packed away, and the new project's skeleton comes first.
- **Experiments 5 to 8 are the new project's first steps, not a side prototype**, so nothing is written twice.
  - Experiment 5 is step 2.2, the eye far.
  - Experiment 6 is step 2.3, the eye near. That is the fork.
  - Experiment 8 opens the tablet (3.1), and experiment 7 opens world generation (3.4).
- **After the eye:** the tablet, then the Registrar, then the Graphics Kit, then world generation stage by stage, then
  making a world, then the game's systems.
- **The eye reads an exported world, not old's world gen** (David, deciding on his concern above):
  - Before old is frozen, it exports the whole test world once, as columns: each column's ground, water and a
    profile number, plus the profiles and the rule that makes a column's blocks (`world/export.rs`). It's your
    tablet in a plain form. The test world has no caves or edits, so the columns are all of it.
  - The new project converts that into its own format, again whenever the format changes; old is never run for it
    again.
  - Far cells are summed from it in the new project, and old's far bake checks the sums.
  - World generation is rebuilt later to write the same format, with the export as its check, column by column.
- **Your step 4's picture gate is dropped.** "Pictures against old's golden references ... within a stated
  tolerance" becomes David's eye on its own terms, the basic way kept (above).
- **Your second part's changes, taken (2026-11-11).** David: "i agree on all counts" (`journal/docs/decisions.md`, "Before
  the first line"). The changes:
  - the reading flight in old (1.7, switches only);
  - the doorway at 2.1, read-only until the fork;
  - marks as data, with the third mark split and far marks and the scatter dial added;
  - 2.2 and 2.3 one change at a time on both cards, rays first by your rule.

  Builder and Counsel amended three points:
  - The far cells come through the doorway from the start; old's far bake is the check, not the source.
  - Soft bricks are flown two ways. One is your spread into neighbours. The other is Builder's exact blend: colour
    and faces stored premultiplied by surface, plus surface itself, in 16-bit, so the hardware's filter sums them
    and one divide gives old's blend exactly.
  - The test places are data in the new project: David's build is its 169 placings, and old's chunk format is never
    read.

### Your ten questions, where they stand

| # | Question | Where it stands |
|---|---|---|
| 1 | Crisp or soft far cells | **Soft**, after two looks (experiment 2) |
| 2 | Rivers and the sea's level | Not answered directly. David: "we don't have to cling to the world gen stack"; water is to be a design step of its own |
| 3 | The moons' shadows | Open (plan step 2.4) |
| 4 | People far off | Open (2.4) |
| 5 | Leaves far off | Open (2.4). David earlier: leaves partly see-through, "like minecraft" |
| 6 | The rejector | Open, not needed till 3.5 |
| 7 | `Art/` online | "Not yet" |
| 8 | The control room in the game's panel | Yes (2026-11-11), then moved at 2.1: it's the "software map", a page of the new editor ("a black screen with magenta boxes that show a tooltip when you hover, connected by lines"), drawn from the code's own list of its lanes, passes, stages and stores |
| 9 | The small card | The Core Ultra 9 275HX's integrated Intel graphics; flown in experiment 1 |
| 10 | The new eye's picture | At this fork |

---

## 2. What the experiments found

| # | What you expected | What we measured | What it changed |
|---|---|---|---|
| 1 | The worst-view figures were soft. The card's number left out tile making. Nothing had been flown on a weaker card | Soft, a little high: "8 ms at the spawn" is 5.94 standing; "14 ms beside quarter blocks" is 11.74. The lanes are 12% of the card on the route and lift its 99th from 9.79 to 13.93 ms. The Intel card ran everything correctly, 2.5 to 3.8 times slower | The baselines. Nothing fails but speed, so nothing shapes the store yet |
| 2 | Crisp: plain bricks at 2.6 times the cells, the simplest and fastest. Soft: 3D textures at 5.5 times, the texture hardware blending, the memory plan doubled | Crisp (build 112), David: "it doesn't look better ... sometimes it looked decent, other times a little janky". Shallow hills band as you move (stair-steps of whole cells sliding). A third way, crisp across and smooth in height (build 113): "its better than ziggurat lods but, its not better than blended" | **Soft bricks.** Crisp and the in-between stay as fallbacks for a card short of memory |
| 3 | The library's checks cost 3 to 15%; ship unchecked | Unchecked, sight is **31 to 49% slower** at the three marks and 8.6% faster on the route's median. The world's rules are about 15% faster a piece | Checks stay on in old. In the new engine each program is priced both ways; the world's rules are the candidate for unchecked |
| 4a | Light is about half the card near the ground | 42% at the spawn, 17% at the mountain top, 17% beside the build, 25% of the route's median | 4b goes ahead. But light alone won't close the gap where it's widest |
| 8, early | The tablet at 1 to 4 GB a world | The whole test world exported as columns: each column's ground, water and profile number, plus the world's 7 profiles. There are no caves or edits in this world. **5.2 billion columns in 0.11 GB**, 0.02 bytes a column (a sea tile's median 780 bytes, the largest land tile 74 KB, 0.28 bytes a column), written in 8 minutes on 22 threads. Read back one tile in 50: 104.6 million columns and 1,029,963 whole profiles, none differing from old's | The columns are a small part of your 1 to 4 GB. What's left to size is the summed levels, which this doesn't hold, and caves' boxes, which this world doesn't have (question 7) |
| 1.7, reading | Your second part: the bare plain's cost is the coarse layers' long rays over the sky and along the horizon | Flights I to N (switches only, old as tagged). The plain is **9.66 ms**, and the build adds 2.02. Each switch alone: the blocks off −31%, light off −21%, the finest layer at 4 × 4 −20%, the coarsest at 4 × 4 −4% (all overlapping). The steps' view shows long rays over the open horizon, but sky is marched only in the coarsest layer, and four times its rays made the plain cheaper. Found on the way: the coarsest layer at 4 × 4 was cheaper than 8 × 8 at all three standing marks (16% at the mountain top) | Your guess ruled out. The cost is spread, and the rest isn't placed: old has one clock for the whole pass. The plain goes into both eyes' marks, and the new engine times a layer and a stage at a time (question 8). The coarsest layer's size becomes a dial to measure |
| 2.1, the pack's check | (Ours, not yours: old moved into `old\` should fly as it did) | Flight I again from `old\`, a day later: the plain 9.63, the spawn 5.94, the mountain top 6.61 ms (I: 9.66, 5.96, 6.62), all within 0.3%; the build's 169 placings changed 0 cells | The pack held. And on this one repeat, a standing mark came back within 0.3%, so a difference of a few percent between switches is likely real, not noise |
| 2.1, the empty frame | (Ours: Builder's estimates beforehand, the card's pass under 0.1 ms, the frame under 1 ms) | Build 116, the new project's first, nothing in the world, 20 s uncapped: the frame **0.404 ms** at the median (99.9th 0.952), the card's one pass **0.019 ms**; every frame timed on the card, pass by pass | The floor every later change is measured from. Nearly all of the frame is the processor's round trip and the hand-over: the card is busy for a twentieth of it |
| 2.1, the wiring | (Ours: build 117 within 10% of the empty frame) | The frame 0.420 ms (+4%), the card 0.018 ms. With each pass now timed on its own: handing the frame to the card and the screen is 0.31 ms of the 0.42, and about 0.09 ms is outside every pass, the window's round trip between frames | The per-pass clocks work from the first build: the frame's time is placed, pass by pass, which old's one clock never could. At David's word (build 118) handing over is timed in two: handing the card its work 0.08 ms (the processor, likely the library translating for DirectX; to watch as the eye adds passes) and showing it 0.14 ms (waiting for the screen); two flights repeat within 1% |
| 2.1, the doorway | Your doorway: one small piece answers "what's here?" a tile at a time, the export behind it, read-only until the fork | Columns as runs: all 5,047 reference columns answered as old's, ground and river or lake water to the block. A tile in 2.2 ms at the median (5.5 to 6.3 the slowest), from the export's compressed tiles | The doorway's gate passes for columns. What changed (David's build), pieces and lights, and named fields come with what first feeds them |
| 2.1, the marks | (Ours: every mark where old's stood, made through the doorway) | The marks as data, each eye within a tenth of a block of old's; David's build (146 bars, 23 blocks) and the scatter (up to 100,000) made as changes through the doorway, none refused. All nine marks 0.338 to 0.344 ms a frame, 0.012 on the card. **A correction:** the session's first 25 s or so run slower, the card too, so the empty frame's 0.404 and 0.019 (the row above) were measured warming up; settled, the floor is 0.34 and 0.012. The rows above compare like with like and stand | Every flight now settles its first mark 35 s. Old's standing marks weren't touched by it: each waited 43 to 180 s for its live levels before measuring. Only old's route medians (C 4.06, C2 4.07 ms) hold the first ~25 s, about a tenth of lap 1, the same in every route flight, so the comparisons stand |
| 2.2, step 1: the far cells | Yours: the far cells summed by a lane from what the doorway answers, old's bake the check, the first sum's time measured | Summed on the processor (David: "its a 1 time event, and it's quick"), on 24 threads, from the columns as runs: all 5,184 regions and 11,231,031 cells from 32 to 1,024 the same as old's bake, byte for byte (a colour one step off fails 144,659 of them). The eighth round the spawn **2.6 s**, the whole world **16.9 s**; old's bake on the card took 46 to 55 s and about 12 min, though it also made the ground from the recipe, which the export did once in 8 minutes | Your order held: the engine's levels made from the doorway, the world file never opened. At this size the far levels cost a loading pause of seconds, not a background job of minutes |
| 2.2, step 2a: old's far march on bricks | Yours: bricks of 8 × 8 × 8 cells found by arithmetic; 250 to 450 MB (for windows round the eye) | The whole world's bricks on the card: **911 MB** (old's pool 384 MB; Builder's pass mark 0.9 to 1.3 GB, since a brick is 8 cells tall where a band was 1 or 2). Old's march as it was, no light yet, the card's whole: the spawn 0.45, the bare plain 1.22, high over the spawn **2.27**, the mountain top **2.79**, low over the sea **3.95** ms. The finest screen layer (2 × 2) is almost never earned from far cells (0.023 ms everywhere); the 4 × 4 costs as much as the base looking out. One crash on the way: edge bricks at the levels over 1,024 wrote past their rows, fixed with a test on every edge | The baseline for your three changes: 2 to 4 times the gate before light, low over the sea the worst. The whole world on the card is a choice to weigh for the Intel card, which shares the laptop's memory. David: "it looks right, mostly"; he asked for the sky weighted more over land at skylines, and a weight that fades as the land covers more "works great" (one power a pixel). Low over the sea he named no ring as a seam; the hand-over sliders "don't do much" from far cells alone, since the finer layers are rarely earned. Not like for like, but for scale: old never timed its far part alone; its whole eye (near, far, shadows and sky light), flown along the leg out to sea, was 5.75 ms (99th 9.88; C2, `kept logs/dyno-1794427178.txt`) |
| 2.2, David's bars for what follows | (His, 2026-11-12) | "try all these things and gather data ... let me take a look at how they look while flying and while still": every change measured and looked at, plus two of yours added as steps (bricks of 4 against 8; a window round the eye for the 32- and 64-block levels). Card memory well below 4 GB at worst; growing with the world's size is fine, with play never. Fast flight may creak; a player's pace must be "solid as a rock" | Memory is now a bar, not just a cost: change 1's exact blend would take the far bricks to about 2.5 GB for the whole world, so bricks of 4 or a window round the eye may be needed for room, not only speed |
| 2.2, step 2b: old's light on the far view, the baseline | Yours: light looked up from the cell, not traced (change 3) | Old's sun shadow ray and five sky cones on the far cells. The far eye on the card, whole (and without light): the spawn 0.77 (0.46), the bare plain 1.42 (1.22), high over the spawn 2.38 (2.14), the mountain top 2.74 (2.67), low over the sea 3.95 (3.80) ms. **Light is 3 to 13% at the far marks** (old's whole eye: 17 to 42%): it's worked out once a pixel, and nearly every far pixel is an 8 × 8 or 4 × 4 square. David: "light looks normal ... so proceed" | The far eye's cost is the march, in the 8 × 8 and 4 × 4 layers. Change 3 can win at most about 0.2 ms in the far eye; changes 1 and 2 aim at where the cost is. Light kept a cell at a time may pay more in the near eye (2.3) |
| 4b | The lane lights each cell once, "about a thousandth of the work". If the lookup costs as much as the rays, light moves into the hit record's read or the bricks' layout is reconsidered | The lookup works and is cheap: the sweep takes 0.00 to 0.01 ms a frame, and the kept light is read across nearly all the far land. But the marks come down only **9%, 3% and 3%** (4a's ceiling: 42, 17, 17), and the route's median is 3% slower. The light's view shows why: the near block faces, lit per face every frame, fill most of every eye-level view | Neither of your two readings. The far half of the light lane holds as drawn, for the far view; David can't tell it from the rays "in a blind test". At eye level light's cost is the near faces'. Your 4.2 has them read light from the air cell in front, which 4b didn't try; experiment 6 does ("light looked up") |

### Today's gap

The laptop at 2,560 × 1,600, a 240 budget of 4.17 ms a frame. Builds 110 and 111; flights A, C, D, E.

| Standing, sight on the card | ms | × budget | With light switched off (4a) | Intel card (flight B) |
|---|---|---|---|---|
| On the ground at the spawn | 5.94 | 1.4 | 3.46 (fits) | 22.7 |
| A mountain top, clear day, fog off | 6.59 | 1.6 | 5.45 | 20.6 |
| Beside the quarter-block build | 11.74 | 2.8 | 9.76 | 29.5 |
| The route: median, and 99th with the lanes | 4.19, 13.93 | 1.0, 3.3 | 3.03 (median) | — |

### What each piece is worth

How much faster sight on the card is with the piece taken away, against the same build with it in. Each was flown
alone; they don't add up.

| | Spawn | Mountain top | Beside the build | Route median |
|---|---|---|---|---|
| All light: sky light and sun shadows off (4a: E against C) | 42% | 17% | 17% | 25% |
| Light kept, for the far pixels only (4b: H against C2) | 9% | 3% | 3% | 3% slower |
| The far reads' blend in code (crisp, F against C) | 27% | 33% | 27% | 23% |
| The same, smooth in height kept (the in-between, G against C2) | 24% | 31% | 28% | 20% |

C2 is build 114's own baseline, everything off; it matches C within 1%, so 4b's code costs nothing when it's off.

### Details worth your attention

- **The standing marks repeat within half a percent** from flight to flight; the route's median drifts about 3%.
  Small differences are judged at the marks.
- **4a skipped light's work but kept its code** in the program. In your design light leaves the frame's program
  altogether, so the gain could be larger than 4a shows (fewer registers, more of the card in flight). Unmeasured.
- **4a left some light on:** the sun's direct light on each face (without shadow), placed lights and fog.
- **On the Intel card the enlarging pass takes 1.7 to 1.9 ms,** against 0.17 on the NVIDIA. It's a card that shares
  the laptop's memory. Its live levels weren't whole after 180 s (each tile takes about 3 times as long), so its
  numbers drew a slightly coarser picture.
- **The twin drift** you found (`sight/cell.wgsl:76-89`) was left alone; it goes with the twin.
- **What the hardware blend could win** (flights F and G, David's "yes", no look): the blend in code is a quarter to a
  third of the card at every mark, and a third of the route's 99th (9.69 to 6.46 ms crisp). That's the ceiling of what
  the soft bricks' texture hardware can win back while keeping the blended look. The hardware's blend isn't quite
  free, so it's an upper bound.
- **4b's light view** (`INCUS_SIGHT_LIGHT_VIEW`; the three marks' pictures in `kept logs/pictures/`) tints each pixel
  by where its light came from:
  - green, the light kept in its cell: nearly all the far land at every mark;
  - yellow, its sky handed down: a scatter through the far land;
  - red, its own rays: hardly any;
  - blue, a near block face lit per face every frame: the ground and the water near the eye, most of every picture;
  - yellow throughout: the quarter-block build. Its pieces aren't in the face table, so each pixel casts its own sun
    ray.
- **At the mountain top the kept light covers about half the picture, yet wins 3% of 4a's 17%.** Builder's reading:
  the far pixels' light was already cheap, their sky mostly handed down. So the far light lane's gain in the frame is
  small even where it's found. Its other gain: with it on, the build's levels settled in 59 s against 83.
- **4b kept a light word of 4 bytes beside every cell** of old's pool: 96 MB beside the pool's 384.
- **4b's lost flight:** G first died at start, because the script rebuilt the program with 4b half-written in the
  folder. It was flown again on build 114. No numbers came from the broken run.

---

## 3. Where your review held, and where it didn't (so far)

- **Held:**
  - light is a large share near the ground (42% at the spawn);
  - the record's worst figures were soft;
  - the lanes' work was outside the number and fattens the tail;
  - the small card fails only on speed;
  - big programs are fragile under the compiler (experiment 3 is a stark case of it);
  - light belongs to the place, not the eye: kept in the far cells, it's found nearly everywhere and the sweep costs
    almost nothing (4b);
  - the blend is a large cost: a quarter to a third of every mark (flights F and G).
- **Didn't:**
  - the checks' cost had the wrong sign where it matters;
  - "near stepping is not the main cost; near light is" (your 4.2) doesn't hold beside builds. There light is 17%,
    and with it free the build mark is still 9.76 ms, 2.3 times the budget: the walk through quarter blocks is most
    of it;
    - *corrected (your second part, then the reading flight):* the build adds only 2.02 ms to a bare plain that
      costs 9.66. Your 4.2 isn't contradicted by the build; the plain is the open question (section 4, question 8);
  - light kept for the far pixels is a small win at eye level (9%, 3%, 3%): far pixels are a minority of those views,
    and their light was already cheap. Light's share there belongs to the near faces;
  - your second part's guess for the bare plain, the long rays over the sky, failed its test (1.7): the sky drawn with
    four times the rays made the plain 4% cheaper.
- **Settled by David's eye, as you left it:**
  - crisp or soft: soft, so the texture hardware does the blend (2);
  - light that steps: he can't tell the far cells' kept light from the rays (4b).
- **Not yet tested:**
  - near faces' light read from the air cell in front (6);
  - the brick walk far (5) and near (6);
  - the rules in 32 bits (7);
  - the tablet's size (8): its columns are measured (0.11 GB), but not its levels or its boxes.

---

## 4. Questions we're bringing

1. **Experiment 3:** why would taking the guards out make the heavy views slower? Builder's unverified guess is the
   compiler laying out the march's long loops worse without their counters (more registers). Should the new engine
   assume checked by default, and price unchecked program by program?
2. **Experiment 4a, beside builds:** most of the cost is the near walk through quarter blocks, not light.
   - Does the lens's near field (a ray into the box, octant steps at the hit) still answer that?
   - Or does it argue for near faces as triangles from the start?
   - What should step 2.3 show to decide it?
   - *Corrected (the Oracle, early call; checked by Counsel in flights C, E and F):* the premise was wrong. Before
     the build goes in, the same spot already costs 9.51 ms (C), 7.60 with light off (E) and 6.41 crisp (F), so the
     build adds only about 2 ms. Those figures are the settling medians, taken while the live levels were still
     being made, mostly before the build. The mark is mostly a bare plain; why a plain is dear isn't known yet.
3. **Light out of the program, not just skipped:** does that change your frame budget's guesses?
4. **The small card's enlarging pass at ten times the NVIDIA's cost:** what does that say about full-screen passes on
   cards that share memory, and about the 980 Ti's column?

5. **Soft bricks** (experiment 2):
   - The memory plan roughly doubles: your 250 to 450 MB crisp becomes perhaps 500 to 900 MB. Does anything else in
     the lens change with 3D textures, such as borders at brick edges, filtering across a level's hand-over, or the
     empty-group mask under blended reads?
   - Your case that flat ground gathers the same at both levels used exact lengths inside crisp cells. Does it hold
     when every read is blended?
   - Flights F and G put the blend in code at 24 to 33% of the marks. How much of that do you expect the texture
     hardware to keep, and on the small card, whose full-screen passes cost so much more?

6. **Experiment 4b, light at eye level:** kept far, light wins 9%, 3% and 3%. The near faces (lit per face every
   frame) fill most of each view, and 4a's ceiling minus 4b's far gain leaves up to about a third of the spawn mark
   in near light.
   - In the lens the near face reads its light from the air cell in front. Is that a word like 4b's, kept per sun
     step for every air cell near the eye? What does it cost in memory near, and in the lane's work when blocks
     change?
   - Quarter blocks today cast a sun ray per pixel. In the lens, where do pieces read their light?
   - Builder's alternative is old's light per face kept between frames (tried at build 107, dropped on the moving
     route), judged on the standing marks. Is that worth a flight in old, or does experiment 6 answer it better?
   - Far light was already cheap (the mountain top: half the picture, 3%). Does the far light lane still earn its
     place, at 4 bytes a cell (96 MB in old), or does light go inside the soft bricks?

7. **The world's format, and where the eye gets its world** (David's concern, section 1):
   - Your step 4 fed the eye "old's bake file and chunks through a loader". But old keeps no blocks: nice25's chunks
     folder is empty, and every block is made again by old's world gen whenever it's near. So that loader would
     have run old's world gen inside the new engine. David decided the eye reads the exported world instead
     (section 1). Does that change anything in experiments 5 and 6, or in what the fork should show?
   - David needs a format "that lets me expand and refine the generation process": later stages, strata and ores,
     water's redesign, things not yet imagined. How is the format cut so that generation can grow without the eye
     being rebuilt or slowed?
   - Counsel's reading of your design, to confirm or correct: there are two formats. The bricks are what the eye
     reads in the frame, so its speed rests on them. The tablet is what generation writes, read only by the lanes
     that make bricks from it. If so, the bricks could be fixed with the eye, and the tablet left open for
     generation, as long as bricks can still be made from it. Does the tablet's layout matter to the frame
     anywhere, through loading in fast flight, edits or light?
   - The export holds the whole world's columns in 0.11 GB (0.28 bytes a column at most, on land). Is that a good
     start for the tablet's columns, or does the tablet want another layout, for loading in fast flight or for the
     boxes?
   - With the columns this small, the tablet's size is mostly its summed levels (old's far bake, cells of 32 and up
     only, is 200 MB). Should the fine levels be summed as they load, your experiment 8 fallback, even though the
     size limit isn't pressing?

*(More as experiments 5 and 6 land.)*

---

8. **The bare plain** (the reading flight, 1.7). It costs 9.66 ms with nothing on it, against 5.96 at the spawn.
   With the blocks off it still costs twice the spawn (6.64 against 3.32), and with light off 7.63 against 3.46.
   Builder's leading candidate *[estimate, low]* is far land seen grazing along a flat, open horizon the whole width
   of the screen; at the spawn, nearer mountains stop the rays.
   - Is that your "rays that skim along the ground" (your 4.8, what would make the budget wrong), met in old already?
   - Does starting rays at drawn boxes answer grazing rays over flat land, or do they still walk many bricks?
   - What should the new engine's clocks (a layer and a pass at a time, from 2.1) look at first, at this mark?
9. **Required memory and spare memory** (David, 2026-11-12: "if something in the generic hardware resources we are
   using is being underutilized, we should ask, 'what can *you* do for me?'"). His bar is the game's card memory well
   below 4 GB at worst; the laptop's card has 12. Counsel's split: what the game needs to run stays under the bar on
   every card, and what only makes it faster or finer is kept as caches that grow into a card's free memory and drop
   away on a small one, bounded by the card, never growing with play.
   - Which of your levers trade memory for time best on a big card: the far picture fixed to the world (your E),
     light kept by place, the next sun step's light prepared before it's shown, a wider ring of near bricks kept
     loaded, maps that let rays skip air faster?
   - Is a cache that changes size with the card honest under "1 always equals 1" and "no per-card tricks", if the
     picture it gives is the same and only the time differs?
   - What else sits idle that the design should ask the same of: the card's time between frames, its copy engine,
     the processor's spare threads, the laptop's 31 GB of memory?
10. **The texture hardware's precision, card by card** (change 1). The card's filter weighs the eight cells to about
    1/256 on the NVIDIA, and how finely is the card maker's choice, so soft bricks blended by the hardware may differ
    by a hair between cards while the world's data doesn't. David: "the exact colors at any point or few points isnt
    a big deal ... as long as it fundamentally works right". Builder compares one mark on both cards at the Intel
    flights, the code's blend as the control. Does this sit inside "no per-card tricks" as you meant it (the same
    data and method, the picture within the hardware's rounding)?

## 5. Still to come

- [x] Experiment 2: one-cell far reads against the blend (David's looks): soft
- [x] Optional: the crisp and in-between flights at the marks: the blend in code is 24 to 33% of every mark
- [x] Experiment 4b flown: light stored in the baked cells, looked up for the far pixels (flight H, build 114)
- [x] Experiment 4b: David's look at light that steps (build 115): he can't tell it from the rays
- [x] References from old (1.6), listed in `master/references.md`: 24 pictures (six places, four times of day), the
      test world's maps and river network, 5,047 sample columns, and the marks' flights
- [x] The export decided (David): the whole test world as columns, for the new eye (`world/export.rs`)
- [x] The full export written and checked: 0.11 GB, none differing (experiment 8's row)
- [x] Old frozen (`old-final`, 7c809d3; David: "so, we'll pack up the old incus artus into old, and just use it
      for looking at and reference")
- [x] Called early: your second part, "before the first line"; its changes taken (David: "i agree on all counts")
- [x] The reading flight in old (1.7): the bare plain is 9.66 ms, spread across the blocks, light and the finest
      layer; your sky guess ruled out; the rest unplaced by old's one clock (question 8)
- [x] Old packed into `old\` (2.1): its tests pass there, and the check flight matched I within 0.3%. The journal
      moved into `journal/` (once `docs/` and `vision/`)
- [x] The skeleton (2.1), build 116: David "looks fine. go ahead"; the empty frame 0.40 ms a frame, 0.019 ms on
      the card (`kept logs/flight-1794486371.txt`), every pass timed on its own from the first build
- [x] Everything declared as data, the map drawn from it, and the control room as the software map, a page of the
      new editor (2.1, build 117; David: "editor looks about like what i'd expect")
- [x] The doorway's columns (2.1): the 5,047 reference columns answered to the block
- [x] The marks as data, and what changed through the doorway (2.1): 2.1's gate met
- [x] 2.1 closed, done enough to go on (David: "yes, close 2.1. let us proceed."). Carried: both cards by one
      command to 2.2; the doorway's pieces and lights to 2.4, its named fields to 3.1
- [ ] Experiment 5: the eye, far (2.2)
- [ ] Experiment 6: the eye, near (2.3). Then this file goes to the Oracle

## 6. The message to send at the fork

A draft for David to edit when the time comes:

> Oracle, we're back at the fork you named: the new eye has drawn the old world near and far, with numbers from both
> of the laptop's cards. Please read `master/` first, then `journal/vision/oracle-return.md` (what we learned since your
> review, and our questions), then `journal/docs/experiments.md` and the logs it cites. Then tell me: is the design right, or
> should it change, and how? First reply with your understanding and your questions; when I've answered, write your
> review to `[a new file in journal/vision/, named by David]`.

Whether to send it to the same Oracle session or a fresh one is David's call at the time. A fresh one reads with no
memory of the first review except through these files, which is what the Oracle asked for ("I'll read `master/` and
the results fresh").

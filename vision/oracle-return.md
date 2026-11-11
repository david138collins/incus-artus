# For the Oracle's return  [KEPT BY COUNSEL, from 2026-11-11]

The Oracle asked to be called back after experiment 6 (`master/plan.md`, step 2.3: the fork), "with the flight logs
and your own look". This file gathers what we learn from its review (`vision/oracle-review.md`) to that day.

Counsel adds to it after each experiment and each decision of David's. It is sent as it stands at the fork, then left
as it was, as part of the journal.

**For the Oracle, when you read this:** read `master/` first, then this, then `docs/experiments.md` (each experiment's
pass mark, written before it flew, and its results), then the logs it cites in `kept logs/`.

---

## 1. What David decided after your review

All in his words in `docs/decisions.md`: "The rebuild: after the Oracle", and "Where things are kept" under Platform
& Workflow.

- **A rebuild from scratch, with old as the reference, step by step.** "to create is human. to duplicate is divine.
  you don't know how to do something unless you can do it again from memory." Your "port" verdicts become "rebuilt
  with old open beside it". Old's code is copied in pieces only where David or the look says it's right.
- **Experiments first, in old,** as you advised.
- **The near field:** rays "if they're cheap, and if that writes elegant compact code that is easier to manage";
  triangles for actors; "triangles for everything in the near field *if* it can look right and there are no
  jarring pops or seams."
- **Crisp:** the enlarging stays crisp (that was the box he'd always set to crisp). Your crisp-or-soft *cells* was
  another question: experiment 2 answered it **soft** (section 2).
- **Light:** sun shadows in steps of up to 30 seconds are fine. The day's length doesn't change how it's built.
- **Order:**
  - the Registrar (and its Color Lab and HSL handling) before the Graphics Kit;
  - the editor made again, "more streamlined";
  - world generation "we don't have to cling to"; water (sea level, cliffs and beaches, rivers) gets a design step
    of its own.
- **Where things are kept:**
  - `master/` (map, plan, spec) is what's true now, read first;
  - `docs/` and `vision/` are the journal;
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
| 8 | The control room in the game's panel | Open (2.1); Counsel leans yes |
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

### Today's gap

The laptop at 2,560 × 1,600, a 240 budget of 4.17 ms a frame. Builds 110 and 111; flights A, C, D, E.

| Standing, sight on the card | ms | × budget | With light switched off (4a) | Intel card (flight B) |
|---|---|---|---|---|
| On the ground at the spawn | 5.94 | 1.4 | 3.46 (fits) | 22.7 |
| A mountain top, clear day, fog off | 6.59 | 1.6 | 5.45 | 20.6 |
| Beside the quarter-block build | 11.74 | 2.8 | 9.76 | 29.5 |
| The route: median, and 99th with the lanes | 4.19, 13.93 | 1.0, 3.3 | 3.03 (median) | — |

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
- **What the hardware blend could win** is measured only on the route: probe P7's 41% (build 107). At the standing
  marks it isn't measured yet. Builder offered the flights (crisp and the in-between against C, about 25 minutes, no
  look); see section 5.

---

## 3. Where your review held, and where it didn't (so far)

- **Held:**
  - light is a large share near the ground (42% at the spawn);
  - the record's worst figures were soft;
  - the lanes' work was outside the number and fattens the tail;
  - the small card fails only on speed;
  - big programs are fragile under the compiler (experiment 3 is a stark case of it).
- **Didn't:**
  - the checks' cost had the wrong sign where it matters;
  - "near stepping is not the main cost; near light is" (your 4.2) doesn't hold beside builds. There light is 17%,
    and with it free the build mark is still 9.76 ms, 2.3 times the budget: the walk through quarter blocks is most
    of it.
- **Settled by David's eye, as you left it:** crisp or soft. Soft, so the texture hardware does the blend (2).
- **Not yet tested:**
  - light looked up (4b);
  - the brick walk far (5) and near (6);
  - the rules in 32 bits (7);
  - the tablet's size (8).

---

## 4. Questions we're bringing

1. **Experiment 3:** why would taking the guards out make the heavy views slower? Builder's unverified guess is the
   compiler laying out the march's long loops worse without their counters (more registers). Should the new engine
   assume checked by default, and price unchecked program by program?
2. **Experiment 4a, beside builds:** most of the cost is the near walk through quarter blocks, not light.
   - Does the lens's near field (a ray into the box, octant steps at the hit) still answer that?
   - Or does it argue for near faces as triangles from the start?
   - What should step 2.3 show to decide it?
3. **Light out of the program, not just skipped:** does that change your frame budget's guesses?
4. **The small card's enlarging pass at ten times the NVIDIA's cost:** what does that say about full-screen passes on
   cards that share memory, and about the 980 Ti's column?

5. **Soft bricks** (experiment 2):
   - The memory plan roughly doubles: your 250 to 450 MB crisp becomes perhaps 500 to 900 MB. Does anything else in
     the lens change with 3D textures, such as borders at brick edges, filtering across a level's hand-over, or the
     empty-group mask under blended reads?
   - Your case that flat ground gathers the same at both levels used exact lengths inside crisp cells. Does it hold
     when every read is blended?

*(More as experiments 4b, 5 and 6 land.)*

---

## 5. Still to come

- [x] Experiment 2: one-cell far reads against the blend (David's looks): soft
- [ ] Optional: the crisp and in-between flights at the marks (what the hardware blend could win there)
- [ ] Experiment 4b: light stored in the baked cells, looked up
- [ ] References from old; old frozen
- [ ] The packing and the skeleton (2.1)
- [ ] Experiment 5: the eye, far (2.2)
- [ ] Experiment 6: the eye, near (2.3). Then this file goes to the Oracle

## 6. The message to send at the fork

A draft for David to edit when the time comes:

> Oracle, we're back at the fork you named: the new eye has drawn the old world near and far, with numbers from both
> of the laptop's cards. Please read `master/` first, then `vision/oracle-return.md` (what we learned since your
> review, and our questions), then `docs/experiments.md` and the logs it cites. Then tell me: is the design right, or
> should it change, and how? First reply with your understanding and your questions; when I've answered, write your
> review to `[a new file in vision/, named by David]`.

Whether to send it to the same Oracle session or a fresh one is David's call at the time. A fresh one reads with no
memory of the first review except through these files, which is what the Oracle asked for ("I'll read `master/` and
the results fresh").

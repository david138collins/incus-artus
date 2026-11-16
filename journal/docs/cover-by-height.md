# Cover by height (the land's own edge by a height table): shelved, and how to pick it up

Written 2026-11-16 by Builder, at David's word, beside `lazy-rays.md`. It packs what the idea is, the Oracle's
description, what's already in the march that it would build on, what's been measured, and what isn't known.
Nothing of it is built. Where the far eye stands is in `master/plan.md`.

## 1. Why it's shelved, and when to take it up

**David, 2026-11-16:** "please pack it up next to lazy rays ... if i become dissatissfied with some views in a
fuller game i'll come back to cover by height and lazy rays and there may be a final check pass against the engine
later anyways."

So: take it up if far land's edges look wrong to him in a fuller game, or at the final check pass. The edges in
question are **land against land** (a ridge in front of a mountain) and **land against sky**; things that stand
(walls, towers, trunks, an arch's piers) were answered by cover by faces (builds 191 to 194, kept).

**His words on the look, earlier:** "i like the way cover works" (2026-11-15, to Counsel, after haze and cover
were shown him as one ridge each way); "i do like the crisper line of the 'cover' as it was shown to me, against
the skyline, and other things against other things, hopefully; but pay careful attention to the weight of the
shader. i hope it doesn't become too much and start squeezing no matter what we do" (2026-11-16), and of the
weight: "i mean its size on the card". And on skylines as brought up to the screen (2026-11-12): "crisp looks
better than smooth or sharp skylines."

## 2. What it is

**Today (haze).** A far cell stops a ray by how much of it is solid, so the top cell of a hill, part full, is like
thin fog a cell deep: a ray through it gathers some ground and lets some of what's behind through. The land's edge
is soft, about a cell wide, wherever it is.

- *Against the sky* that soft edge is sharpened by the sky's weight (David's 4, build 124): thinly covered skyline
  pixels lean to the sky.
- *Against other land* nothing sharpens it: the sky's weight acts only on the part of a pixel the sky shows
  through. A ridge in front of a mountain has the softest edge in the picture.

**Cover by height.** The land is taken as a surface with a true height in every column (the height table). A ray
meets it or passes it. A passing ray keeps how close it came, measured in pixels, which is the share of that pixel
the land covers; then it goes on and takes whatever is behind for the rest. The edge is worked out at the near
ridge alone, so it's the same against a mountain as against the sky, and crisp to a fraction of a pixel up and
down.

**The Oracle's description** (`journal/vision/oracle-review.md`, its third part, sections 4 and 9; read them
whole before building):

- "With the skyline made by cover there's no haze. The land is a surface, where the blend is half solid. A ray
  meets it or passes it, and a passing ray keeps only its closest miss, measured in pixels, which is the square's
  cover."
- "Then the table pays. Each column's lowest and highest top, kept at every level, lets a skimming ray stride
  across any stretch that's flat to within a share of a pixel, and step finely only where the ground near it is
  rough (Tevs, Ihrke and Seidel, 'maximum mipmaps', 2008; Dick, Krüger and Westermann, 2009). Hundreds of steps
  become tens" (its estimate, marked medium).
- "It answers plain columns. A place that isn't one (an overhang, an arch, a slab in the air) keeps the march it
  has."
- What it buys, in its table: "Skimming rays in tens of steps. Far mountains keep their body, with no washing to
  correct." What it costs: "The haze look. Plain columns only." (Also named there: cone-step maps, Policarpo and
  Oliveira, 2007. The Oracle named its papers from memory: check each before leaning on it.)
- The three things it asked David's eye for: does a skyline still average and glide; do far mountains keep their
  body; does a wood's edge or a row of posts still fade and never pop.
- What follows if cover is taken: "the sheet", plain far land of the levels of 32 and up drawn by the triangle
  hardware as rings of mesh round the eye, flown as the far levels' rival. It waits with this.

## 3. David's question, and Builder's answer (2026-11-16)

"would the lands own edge by a height table work with mountains in front of mountains as well as in front of
sky". Builder: yes, by its design, and that's a difference from today (section 2). What it wouldn't change, as
said to him:

- *Sideways detail.* The table has one height a column, and far columns are wide (32 or 64 blocks). A far ridge's
  outline would be sharp and still drawn in steps as wide as its columns.
- *Anything that isn't plain ground:* overhangs, arches, towers, cave mouths. They stay with bulk and faces.
- *Two ridgelines crossing inside one pixel:* a version that keeps only the nearer miss gets that pixel's colour
  a little wrong.
- And: it's from the Oracle's description and Builder's reading of the march. None of it is built, so the look is
  unproven until he sees it.

## 4. What the march already has that this would stand on

- **A height table, and a skip that uses it.** Each cell column's top, at every level, kept in three rows the
  march reads (its own level's, one two levels up, one four up); a ray over a column's top hops to where it leaves
  the column (`far.wgsl`, the skip; `far_eye.rs` makes the tops, "the height table's true numbers"). The skip
  stops one cell above the land. It's worth a third of the march's time (build 177's audit).
- **What's missing:** the surface test itself (land as a surface met or passed, with the closest miss kept as
  cover) in place of gathering haze through part-solid cells of plain ground; and each column's lowest top beside
  its highest, so a skimming ray can stride over a stretch that's flat to within a share of a pixel.
- **Cover by faces** (`shown_share`, where a step is gathered): the cover of things that show a ray a face. It
  stays for everything that isn't plain ground, as the bulk's rule does.
- **The sky's weight** (`finish`): would have nothing left to correct on plain land.

## 5. What's been measured that bears on it

- **A long ray's steps are mostly close over the ground.** Of the plain's longest ray's 326 steps, 176 were
  samples close over the ground: 89 strides inside the blend's reach, 63 in thin haze, 24 at a surface (2026-11-14,
  build 176). Those are the steps this would take away.
- **The long rays are already cut** (builds 183 to 190): the marks where the card waits on its longest chain came
  down by a quarter to two fifths. So what cover by height adds in speed is less than when the Oracle wrote, and
  it's David's reason for shelving it: it's for a look, or a squeeze that comes later.
- **The march has two registers of room** (168, with 12 groups fitting to about 170; David's reading of builds 192
  and 194's rows). This goes into the march's own loop, in the gather and the skip. What the march holds and what
  could come out to make room is in `march-by-parts.md`.
- **Its edges today, for comparing later:** the five marks' pictures and the stage's (the castle, the plain's
  group, the town) are in the journal under builds 188 to 194.

## 6. What isn't known

- **The look.** Crisp land edges have been shown to David only as a drawing of one ridge. Whether a far skyline
  still glides as he moves (the haze averages; a surface test has to average by cover alone), and whether hills
  keep their body, is his eye's on the real thing.
- **Two rules in one march.** Plain ground by a surface, everything else by bulk and faces, and a ray has to know
  which it's in at each step. How that's told (a column that's plain ground or not is known where the cells are
  summed, since build 188), and what it adds to the program, isn't designed.
- **Where levels hand over.** The surface from two levels' height tables has to agree through a hand-over, or
  there's a ring. David sees rings before anything else.
- **Water.** The sea's surface and a bed under shallow water are surfaces too; ground counted wrongly through
  water has made bright bands twice (builds 188 and 191's first form).
- **The time.** "Hundreds of steps become tens" is the Oracle's estimate. Its numbers were wrong five times in
  that review by its own count, and Builder's sums as often.

## 7. To resume

1. Read `master/plan.md` ("Where we are"), this file, the Oracle's sections 4 and 9, and `march-by-parts.md`.
2. Say what David is dissatisfied with, at which views, and draw those views off the screen as they are.
3. Make room first if the march has none: a trace of `march_all` (its row), and the candidates in
   `march-by-parts.md`.
4. Talk it through with him before building (land against land, the sideways steps, what stays haze).
5. Its pass mark before any code: the picture's edges measured before and after at the views he named; nothing
   put by and 12 groups still, by an off and an on trace; flights as a bracket (off, on, off); his look on the
   stage and on bare mountains, standing and moving.
6. One step a build: the surface test with the table as it is first (the look); the strides over flat stretches
   after (the speed).

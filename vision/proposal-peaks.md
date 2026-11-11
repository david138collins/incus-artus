# Proposal: peaks at the points

From Counsel to Builder, 2026-10-27, written from David's conversation with Counsel. **For assessment
first:** reply with costs, risks and anything you'd change, and don't build until David says. Names
and panel placement are David's call (ask before naming or placing anything). Background is in
`docs/worldgen.md` (Additives, "Ridges as the base, peaks from the dots" and the notes under it) and
`vision/terrain-pipeline.md` (the order of the sums, and the "sockets").

**BUILT 2026-10-27** (08a5860 through 262d1d0). David: "we hit it on the head"; peaks tower at ~512.
`docs/worldgen.md` has the record; `vision/terrain-pipeline.md` shows where they sit in the sums. This
file is kept as the history of the plan.

## Agreed, with changes [David, 2026-10-27]

David agreed the plan below with these changes, which win wherever they differ from it:

- **The Lift dissolves into the mountains.** Its point picker becomes the peaks' points, and its cloth
  (height, reach, spline, coast pad) becomes the bump each peak is cut down from.
- **Additives panel: one "Mountains" heading** holding Where (the picker), Rise (the cloth), Crags
  (layers, a slope response spline, a steepness scale), Variation (± spreads), and the existing Mountain
  grain, Grain drift and Ridge streams. **"Hydrology"** comes later as a sibling heading.
- **Interpreter: "Mountains (from the squeeze map)" becomes "Ridgelines"**, today's ridges unchanged.
  Ridgelines raise the land toward the highlands; the Additive mountains put peaks across the same
  area: two things from the same data. Hills stay (David has them off).
- **The ceiling first:** build height 1,024; the terrain cut-off becomes its own setting, and 333 stays
  the ridgelines' measuring stick, so tuned worlds don't change.
- **Finding peaks per block:** a grid of cells, each listing the peaks that reach into it, so
  neighbours blend without seams. Crag size is a share of each peak's height. The slope response is a
  spline with one "steepest" scale.
- **Later:** a unified ridgeline (one crag pass over ridges and peaks together) as a switch to compare;
  hydrology (see `docs/worldgen.md`).
- **Build order:** (1) build height and the cut-off setting, no visible change; (2) the Mountains group,
  with the Lift's picker and cloth moved in, ground unchanged, and the Interpreter's section renamed
  Ridgelines; (3) the Additives connected to the world (saved, handed over at spawn, a Show switch);
  (4) peaks: rise, crags, slope response, then variation and switches.

The rest of this file is the proposal as first written.

## The aim

Real mountains, standing at least ~400 blocks high, so that from the top you overlook the land. They
rise at points the Additives choose, nested in today's ridged "mountains", which David now treats as
**hill country**: the nest, and the ruggedness that fades out into the plains. Peaks should be
towering and fantasy-like (taller than wide), craggy, varied, and never obviously repeated.

The technique is Inigo Quilez's "More Noise" (iquilezles.org/articles/morenoise): gradient noise gives
its slope for free, and each finer octave is weighted by the slope so far. **Take the mechanism, not
the look.** The article's default makes worn, eroded mountains, smoothest where steepest, which is the
opposite of towering. Here, the weighting is a curve David shapes.

## The rule that splits the work

**Choosing goes in the Additives; drawing goes in the Interpreter.**

- **The Additives** do whole-map jobs, once per compile: anything that needs the whole map or a
  point's neighbours.
- **The Interpreter** works out each block alone, fast, with the same answer every time. It reads
  what the Additives made and never makes choices.

This matches David's linear-pipeline rule: the Additives run upstream of the Interpreter, and nothing
feeds back.

## 1. The Additives side: the peak list

- **Move the Lift's point choosing here.** Today `lift_points` (compiler.rs) already does whole-map
  work with the Additives' own dot picker, but its settings sit in `InterpreterSettings.lift`. The
  choosing settings (points from, spacing, least / most crush, range cut, smallest range) belong
  with the Additives. The cloth's shape (height, reach, curve, coast pad) stays in the Interpreter.
  Old presets must still load. Confirm with David before moving anything.
- **The peaks share those points**, or have a scatterer of their own beside the Lift's. Your call to
  propose; David's to decide.
- **Each peak in the list carries:** where it stands, its own seed, and its values for whatever varies
  (height, width, cragginess, and so on).
- **Seeds from where a peak stands** (or a stable id), never from its place in the list, so moving one
  slider doesn't reshuffle every mountain.
- **No per-mountain settings** [David]. Every difference between peaks is a rule that applies to every
  mountain ever made:
  - the noise, from each peak's own seed;
  - **programmed variation**: reuse the Tectonic Sim's ± spread, so each peak draws its own value
    within a range David sets;
  - variation from the data, if useful: e.g. taller where the crush is strongest, wider where points
    are sparse.
- **Rules between peaks**, if wanted later: keep them apart, the tallest of a range, keep off the
  coast. The list is where those live.
- **Maps for the Interpreter:** whatever lets one block find the peaks near it cheaply (a nearest-peak
  and distance map, as the Lift has, or a grid of peak ids). Blending between neighbours needs more
  than the single nearest.

## 2. The Interpreter side: drawing a peak

For each block, for the peaks near it:

1. **The rise:** height by distance from the peak's centre, a curve David shapes (with the usual
   dots and bends), times the peak's height, out to its width. Long sloping arms can be smaller peaks
   with a narrow double-dip curve [David], so the curve editor must allow that shape.
2. **The crags:** derivative-weighted fbm, the article's method, with **the rise counted as the first
   octave**, so the rise's own slope steers where detail goes.
3. **The slope response:** a curve, how much each finer octave survives at each steepness. This
   replaces the article's fixed `1 / (1 + k·slope²)`. It's what makes towering rather than worn: David
   can keep or add detail on the steep flanks.
   David sees the rise and the slope response as one combined rise + slope pair; the panel should
   show them side by side.
4. **Below zero or not:** a switch for whether the crags' dips may carve into the hill country's
   ridges, or only the peak itself. David thinks it may be unnecessary, so keep it cheap.
5. **Overlaps:** a switch, **higher wins** (separate peaks) or **add** (they merge into massifs).
6. **Where it adds:** onto the ground after the hill country, so the ridges cut into the peaks' feet
   [David]. Check the order against the hill country's "share of the room up to 333", which shrinks
   mountains under raised ground (`vision/terrain-pipeline.md`, first note).

## 3. Controls

David wants more controls over the features: **expose the method's hidden numbers**, each with a
plain name, a tooltip saying what it does, and a narrow, useful range (his standing rule). At least:

- how many octaves; how much finer and how much weaker each one is;
- the rise curve and the slope response curve;
- the peaks' height, width and cragginess, each with a ± spread;
- the overlap switch, the below-zero switch;
- a Show switch for the peaks (spawning already follows the switches).

Tune for blocks: octaves finer than a block are wasted, and on a 400-block peak the slopes are large,
so the article's numbers won't carry over.

## 4. The ceiling

400-block peaks don't fit under Z 333; today no mountain stands more than ~205 blocks above the sea.
The terrain-ceiling item in `docs/worldgen.md` (Next steps) is a prerequisite or a companion. Your
earlier assessment suggested keeping 333 as the hill country's measuring stick and moving only the
final cut-off, which would leave David's tuned worlds alone. Please say which order you'd do these in.

## Not wanted (David, 2026-10-27)

- **Per-mountain settings.**
- **Grain alignment:** the ridges give the continental grain; fantasy peaks don't need it. A slight
  stretch along the grain, maybe later, only if needed.
- **Drainage:** the fake erosion's valleys may end in pits. Fine, real water erosion comes later as
  another additive.

## Checks worth having

- The same seed gives the same peaks; moving one point's slider leaves distant peaks unchanged.
- Old presets load, and with the peaks off the world is exactly as before.
- A peak never moves the coastline (as the Lift's coast pad ensures).
- How long a chunk takes with peaks on, since every block may sample several peaks × several
  octaves.

## Order, suggested

1. Assessment back to David: costs, the ceiling order, anything here that's wrong.
2. Move the point choosing into the Additives (no change to the ground).
3. Raise the ceiling (or set its own cut-off).
4. One peak shape at the points: the rise only, then the crags, then the slope response.
5. Variation (± spreads), the overlap and below-zero switches.

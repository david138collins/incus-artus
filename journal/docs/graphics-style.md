# The graphics style: one fixed screen of squares

A note of the style David and Counsel talked through on 2026-11-17, kept for the graphics style pass ("it seems we
will need to do a whole graphics *style* pass once we have a healthy amount of content for a small slice of
gameplay"). Nothing here is built or decided beyond what's marked; his words are in `journal/docs/decisions.md`
under that date. Where it would sit in the engine is in section 3.

## 1. The picture, in his words

"i imagine the raymarcher as an adaptive grid of cells in the bottom, masked and subtracted by the triangle field,
both feeding the same simple 2x2 screenspace that would be the fixed monitor ... UI elements can also draw on this
grid or even use a 4x4 subgrid on top with transparency rather than masking. i think that may be a good artistic
interface i could work around, from there i could try different 3d styles in between the triangles and the 2x2
screenspace. halving the actual resolution the game runs at is probably good."

His image for it: "slipping the triangles like a pack of compound butter under a turkeys skin."

So, three layers and one screen:

1. **Underneath, the far march:** an adaptive grid of squares (today 8, 4 and 2 screen pixels wide).
2. **Over it, the triangle field** (the near eye), which masks the march: the march does no work under it (in,
   builds 201 to 203).
3. **Between the triangles and the screen, a place for styles:** whatever is done to the near picture before it
   is laid on.
4. **The fixed monitor:** one screen of 2 × 2 squares ("macropixels", ProPixelizer's word) that both feed. The
   world drawn at half the window's size (1,280 × 800 on the laptop's 2,560 × 1,600), shown without smoothing.
5. **The UI** on the same grid, or a finer one over it, with transparency.

## 2. The styles to try in that place (all open)

- **The near view on the same squares:** the near picture drawn at half size on the far eye's own tiles (the same
  corner, so squares coincide). Edges hard, or averaged from several samples a square ("edges that average and
  glide", which the plan keeps).
- **Figures as live 3D drawn chunky** (Dead Cells, Guilty Gear Xrd): the figure stays real 3D in the world (depth,
  hiding, hits on its true shapes); only its pixels are drawn chunky, its poses held and snapped (the Quake note).
  A flat sprite only far off. "the actual model doesn't have to look that great".
- **Colour crush** stays at the asset (the palette and its splines), not as a screen filter.
- **One thing bent or pixelated** as an effect (the wraith's end; `journal/docs/vision.md`, "Bending the lens").
- **Outlines** from depth and surface direction: not asked for; listed because the references lean on them.
- **Silhouettes in the fog** (his idea, 2026-11-17): with real fog, edges found behind it draw terrain, buildings
  and actors as silhouettes; `journal/docs/vision.md`, "Silhouettes in the fog".

## 3. Where it sits in the engine as it is (build 209; `master/map.md`)

The frame's passes on the card, in order, and where each part of the style would go:

| The frame today (`master/map.md`, "The passes") | The style |
|---|---|
| the far start | |
| listing the near chunks; the near faces; the near water → **the near picture** (a store of its own since build 199) | **The styles' place:** a pass, or several, each its own switch, reading and writing the near picture (chunky figures, effects, an outline). Drawn at half size here, the near view is on the squares |
| the near cover (which far pixels blocks wholly hide) | already the mask: "masked and subtracted by the triangle field" |
| the far 8 × 8, 4 × 4, 2 × 2 (the march) | unchanged; at half size its layers are 4, 2 and 1 of the picture's own pixels, the same squares on the glass |
| bringing it up (the far layers onto the picture) | writes the half-size world picture |
| the x-fader (the near picture laid over the far) | writes the half-size world picture: where "both feed the same screenspace" |
| *(new)* | **The fixed monitor:** one small pass enlarging the world picture onto the screen, crisp |
| the panel | the UI: on the screen as now, or on the squares, or a finer grid over them |
| showing it | |

What makes this cheap to try: the near eye already draws into a picture of its own, and the march, the near eye and
the panel are separate passes, each measured alone ("the graphics engine is never welded shut"). The march's
program isn't touched by any of it.

## 4. What isn't known

- **Crawl:** squares fixed to the screen swim over near surfaces as the eye moves, worst close up. Whether it reads
  as style or as a flaw is his eye's. His doubt, in his words: whether a quantized carpenter at a sharp table would
  "look right, or look like looking at a fish through water and trying to grab it." Snapping to the grid
  (ProPixelizer) only holds for a camera without perspective. Dylan Ebert's texel splatting
  (https://dylanebert.com/texel-splatting/) fixes squares to the world instead; moving figures are unsolved there.
- **Hits and held poses:** if poses are held, hits are worked out from the held pose too.
- **What half size saves:** a quarter of the pixels in every pass, but the march's cost is its rays and the near
  eye's its faces' corners; a part, to be measured in a bracket.
- **The spec's "the 3D world at native resolution"** changes if he settles on half size. His older words point the
  same way: "a beautiful world, squinted" at about 640 × 480; "a DOS-game resolution where everything is there".

## 5. Where it sits in the plan (as of 2026-11-17, build 209)

David: "we'll probably have to keep moving after step six there, and come back to this when we have enough content
to work on style."

So the style changes nothing in the work under way. Graphics finish by his checklist of 2026-11-16 (`master/plan.md`,
under 2.4), which stands as it is:

1. Placed things in the far view: in (build 207).
2. Near light: the sky's light in (build 208); faces turned away built, unflown (build 209: half as many faces sent
   at 32 chunks); next, light kept only where it isn't plain daylight (build 210, his "proceed with C"); then placed
   and moving lights.
3. Glows and far lights.
4. More textures (the bars' own pictures, the grass's edge, the stage's seven blocks).
5. An assessment of a fine ring past the triangles.
6. The near eye's own costs: listing chunks, a chunk with pieces, chunk optimizing where it would show.

After step six we keep moving: Phase 3 in his order (tectonics; world generation, topography then water; the
Registrar and the placer; ecology), then players, items and animals through the editor. The style pass comes after
that, in Phase 4, once there's a healthy amount of content for a small slice of gameplay. Until then this note
waits; the engine is kept so it can be come back to ("never welded shut", the plan's "What every step keeps").

## 6. To take it up

1. The smallest look, any time he asks: one switch, the world drawn at half size and enlarged crisp (the new last
   pass above), A/B on the Render page; its place and name asked first.
2. The style pass proper (the plan, Phase 4): with figures, a workshop and combat to judge by; each style a pass
   with its own switch, measured alone, thrown away if it doesn't earn its place.

## 7. Since: the detail curve (builds 212 to 215, the same day)

The "smallest look" of section 6 was built that afternoon (build 212, "Near pixel size": the near picture at half
or quarter size on the far eye's grid), and his look at it moved the picture of section 1 on. Not one fixed screen
of 2 x 2 squares, but squares sized by distance on one curve: "4x4 feels beautiful up close, but it quickly becomes
just blurry pixel art farther away"; "a sort of linear equivalence of detail from the back if your hand towards
infinity ... a formula with a single flexture. triangles go 4x to 2x to 1x and then march 2x - 32x on that golden
curve."

- **The curve:** the smallest thing drawn at a distance d is sqrt(a² + (g × d / P)²) blocks (`master/plan.md`,
  under 2.4; his words in `decisions.md`, 2026-11-17).
- **Its near half is built** (builds 213 to 215): up to three near pictures, in squares of 4, 2 and 1 screen
  pixels, by bands of true distance, fading into each other by a stipple; all on the screen's own grid, so the
  squares still coincide with the far eye's. Controls in The near eye: "Largest near squares", "Detail up close
  (blocks)", "Stipple width (share of the band)", "Picture sharpness in the squares". It launches off.
- **Its far half** (rings that reach where the curve says, a band of 4s at the seam) comes after lights and
  nights, priced first.
- **What stands from this note:** the styles' place (section 3: the near pictures are still stores of their own),
  crawl as the thing his eye judges (section 4), half size for the whole world as one setting of the same
  machinery, and the style pass in Phase 4 (section 5).

## 8. Since: the near half left as it is, and squares fixed to the world (2026-11-18)

His look at the curve's near half (build 218): "i like the look with 4x4 base up close"; what he doesn't want is
the blur where a 4x square holds several of a block picture's squares, nor the speckle that sharpening trades it
for. "there shouldn't be any blur beneath the march, and the march should be all averages." He left it: "make a
note that this could be reworked later and how that could be done and leave it for now." The note is in
`journal/docs/experiments.md` (his look at build 218): the slider as a plain distance; and three ways to no blur,
the last of them section 4's squares fixed to the world.

On that last: "i guess right now the work is doing the little things until whats left is really the last big
change and we can proceed. but we may have to leave it and work towards world gen/content for a bit to be able to
judge it properly." So it waits with the style pass (section 5), and is the first thing to take up there. The
cheaper middle way, to weigh against it then: block pictures that carry their own coarser versions (16, 8 and 4
squares a side, drawn or picked, never averaged).

## 9. Since: a fourth way to steady the chunky look, and Builder's recommendation (2026-11-18)

Sent by Builder at David's asking, with her account of the graphics so far, as the assess bracket flew. All of it
is parked by his word (section 8); nothing built, nothing priced.

**Why the 4x band shimmers as he turns:** the band is drawn at a quarter size, and each 4x4 square takes its
colour from one point of a block's picture at the square's middle. The squares are fixed to the screen, so as the
world slides under the grid each square reads a slightly different spot and changes colour.

**The cure, either way, is to make the chunky squares belong to the faces and not the screen:**

1. **Chunky by the picture, drawn at the screen's own size** (new here; section 8's "pictures that carry their own
   coarser versions" taken a step further). The near band is drawn full size, but each face wears a coarse version
   of its picture (8 squares a block, each a 4x "pixel"), read without blending. Every chunky square is then a
   fixed patch of the block's face and moves with it: no crawl on turning. The coarse versions are drawn by him or
   made by keeping each patch's commonest colour, never an average. **The trade: silhouettes and the edges between
   blocks come out crisp at full resolution, not in 4x stairs.** The cost is the near band drawn full size in
   place of a quarter (to measure). Fairly small to build.
2. **Texel splatting** (section 4): each square of a face's picture drawn as its own small square, snapped to
   whole screen pixels. The chunky look then takes in the silhouettes too, still fixed to the world. The largest
   change of the lot, the card's cost unmeasured, care at seams and steep angles, moving figures unsolved.

**Builder's recommendation, for when he un-parks it:** try the first on one patch of ground, drawn both ways and
priced, because it takes the crawl away for little; go to the second only if he wants the edges chunky too.

*Counsel:* the fork between the two is one question for his eye, and it can't be answered from here: when he said
"i like the look with 4x4 base up close" (2026-11-18), was it the chunky squares on the faces, or the stairs on
the edges as well? The first way keeps the one and loses the other. A patch drawn both ways answers it for the
price of a look. It also bears on figures: a figure's edge in the first way is crisp, in the second chunky.

## 10. Since: his first answer on the edges, and letting it lie (2026-11-18, the same evening)

He read a piece on texel splatting with Builder (one he pasted; her reading: it looks like an AI's answer to a
description of this engine; its sound part is section 9's first way; its speeds have no source, and it takes
4x, 2x and 1x for squares a block, where here they are screen pixels a square). His words after, relayed by her:

- "i like the stairstepped edges i think. its hard to say without seeing it."
- "im just not sure what the two ways would look like. hence thinking maybe i should let it lie and move on for a
  while, maybe."

So section 9's question has a first, unsure answer: the stairs on the edges are part of what he likes. **Nothing
is decided.**

**What that answer would rule in and out, by Builder's reading:** on blocks, squares fixed to a face give
straight, crisp edges (section 9's first way, and plain splatting too, since a block's edge lies along its
squares' edges). Stair-stepped edges that also hold still as he turns come only from a grid fixed to directions
round the eye (the probe grid in Ebert's own write-up, section 4), the part the pasted piece says to skip. So if
the stairs stay wanted, the cheap way doesn't give them, and the way that does is the dearest.

**Where it rests:** Builder recommends letting it lie, as he leans, with a look at Ebert's own demo as the free
way to see the second look. It stays parked with section 8, to be taken up with world and content to judge by.

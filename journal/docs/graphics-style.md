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

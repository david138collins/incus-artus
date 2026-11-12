# Proposal: the voxel renderer

From Counsel, 2026-10-28, written from David's conversation with Counsel. **For Builder's assessment
first:** costs, risks, what's wrong or missing, and a build order. Don't build until David says. Names
and panel placement are David's call.

**PARKED 2026-10-28** (David, after the stutter fix): "we don't need to build a custom renderer at this
point since we have found we were leaving a LOT of performance on the table ... we can keep working on
the plan for a voxel-monster later. right now i just want to enjoy the flight." The plan and Builder's
assessment are kept for later. **When to unpark** (counsel), whichever comes first: trees and forests
arriving at scale (measure on the stress forest first); the frame rate on the target machine falling
short of a smooth 60; the CPU roasting again. Two pieces don't wait for the renderer, because waiting
would cost: **slot textures** before David paints real blocks (Builder's step 2, independent of the
renderer), and keeping **quarter blocks and moving block objects** in mind whenever the mesher is touched.
**2026-11-02:** David wants the far view's root fix discussed here, foveated ray marching (the last
section). **Unparked the same day for the far view** (David: "its the hard part, we might as well do
it now"); decision in `docs/decisions.md`. Builder to review before any building.

## The aim

David: "we're making a voxel game ... rebuild bevy into a voxel monster that eats 9999999 voxels for
breakfast, and we should do it before we start demanding even more of it with 99999 trees and what
not. we need to think very carefully about what it will need to do, so we can get it done right, and
done once." He agrees on level 3 of the options Counsel laid out:

- **Not a fork of Bevy.** Bevy keeps the window, input, ECS, assets, UI and the egui panels.
- **Our own way of getting the world to the GPU**, inside Bevy's render world: large shared GPU buffers
  holding every section's vertices (a new chunk is written into a free slot: no asset, no entity, no
  per-mesh bookkeeping), one draw command for everything (multi-draw indirect), and the GPU deciding what's
  visible (culling in a compute shader).
- **The rest of the game talks to it through a small interface** ("show this section", "update it",
  "hide it", "draw these instances"), so the internals can be improved later without touching the game.

Whether or not the fast-flight stutter turns out to be Bevy's mesh path, David considers this the right
call, and wants it before trees and the rest arrive. It's item 8 of the Performance Plan
(`docs/decisions.md`, "GPU-driven drawing"), moved up.

## "Done once" means: get the contracts right

No renderer is finished in one pass. What can be made right once is what everything else plugs into, so
improvements later never ripple outward:

1. **The interface** the game uses (sections, instances, far surface, overlays).
2. **The vertex format**, with room for everything listed below (it's the costliest thing to change).
3. **The texture system**: block textures in GPU texture arrays built from the block register, **slot
   textures** (each pixel a shade slot, each material a palette row; `docs/worldgen.md`, Next steps), and
   palette-preserving mips. Painted blocks depend on this, so it comes before David paints real ones.
4. **The shared lighting module** (step 2 of the far-surface plan): one place for light, shade, fog, sky.
5. **The GPU memory allocator**: slots that are reused and compacted, within a budget for the target
   machine.

Inside those, everything can be swapped: meshing method, culling method, how far blocks reach.

## What it will need to draw (the inventory)

Gathered from `docs/`; each is something the contracts must leave room for, not something to build now.

**Terrain blocks**
- Full blocks, 32×32 faces, packed vertices (already ~8 bytes), meshed in the background with a budget
  per frame, player edits jumping the queue (a dug block must vanish within a frame or two).
- **Quarter blocks and octants** (`docs/decisions.md`, Quartile Placement): 12 bar slots per cell, mixed
  materials in one cell, stairs from stacked bars. The mesher must handle sub-block shapes, and the
  vertex format must address half-block positions.
- **Water levels 1-8** (like Minecraft's snow layers), see-through, with a flow direction per face (the
  palette cycle's direction), shore ripples from David's ripple tile map (each edge block knows which
  sides and corners touch land), and still water for lakes and the sea.
- See-through and cut-out blocks: water, glass, leaves. Sorting or a method that doesn't need sorting.
- Lighting today: the near light window, the cone map, the height tint, all moving into the shared module.

**The far view**
- The smooth far surface (built, 5c1284c), with the lighting moving to the shader.
- **Blocks far out only where something was built or changed** (`vision/watch-list.md`, item 17), so
  untouched ground past ~1,000 blocks is the far surface alone.
- Later: far stand-ins for cities, the traced or pixel horizon. Keep them possible, don't build them.

**Lots of the same thing (instancing)**
- **Trees**, thousands to hundreds of thousands, **made of blocks** [David, 2026-10-28]: whole oak and
  **quartile oak** blocks (thin trunks and branches), defined by building them in the structure suite,
  with a couple of growth stages. So near trees are terrain voxels with sub-block shapes, not instanced
  models; far away they become **sprites of themselves** (impostors baked from the structure).
- **Moving voxel objects:** a **felled tree** leaves the world as blocks and falls as one body before
  landing as logs (David's long-wanted tree felling, `docs/decisions.md`, structure editor). The renderer
  must draw a meshed block object that moves and turns. Carts, boats or doors might use the same.
- **The structure suite's imaginary blocks** (hooks and triggers): visible only in the suite.
- Grass, flowers, small plants: cross-quads or similar, instanced, faded out with distance.
- **Items on faces and in quarters** (mugs, frames, spears), pallets showing stored blocks in miniature.
- People and animals, then sprites (impostors) for crowds past ~60-100 blocks (item 17).

**Effects**
- Palette cycling (water, and anything else animated by palette).
- Particles: rain, snow, mist and splash foam at falls, lightning, magic (the FX Lab,
  `vision/toy-factory.md`). Probably GPU particles, any colour.
- Sky and clouds (dithered sky, cartoony cloud sprites, thunderclouds fading dark), fog with a couple of
  miles of visibility (`docs/decisions.md`, "Sky, air and weather").

**Other views and tools**
- **The isometric foreman view** (`docs/decisions.md`, Isometric View): an orthographic camera,
  **height-slice scrolling** (cut everything above a Z, with caps on the cut), see-through indicators on
  people. Cutting at a Z should be cheap (a shader test), not a re-mesh.
  **David, 2026-10-31:** the orthographic camera is roughed in with the Registrar upgrade, "present
  enough to be relevantly measured during the graphics and performance pass". From above, it shows far
  more of the world at middle distance than first person, so the renderer has to be measured in both.
- The deep screenshot, the Show switches, debug overlays, the Sandbox's stand-ins.
- Live reload of textures from the block register.

## What it must be fast at

- **Streaming:** flying fast without stalls (David tests at 10,800 mph; normal play is far slower).
- **Edits:** a block placed or dug shows at once.
- **Scale:** many sections on screen in few draw commands; memory within the target machine's budget.
- **Target machine:** a GTX 980 Ti-era PC (6 GB), DX12 by default. Compute shaders and indirect drawing
  are available there; check multi-draw indirect support through wgpu.
- **Measured, not guessed:** the soak test before and after each step, Tracy for where the time goes, the
  old path kept behind a switch until the new one has proven itself.

## Questions for David

1. ~~Trees: blocks or models?~~ **Blocks**, quartile oak included (David, 2026-10-28).
2. ~~Frame-rate target~~ **A smooth 60 is enough for anyone; nice rigs should get at least 120**, and
   David believes higher is possible (David, 2026-10-28).
3. ~~See-through blocks~~ **Not a big deal yet**, but in the end it must do better than Minecraft's glass,
   "so painful to look at when there's a lot of it" (David, 2026-10-28).
4. Anything on the list that's wrong, missing or never wanted.

## The kung fu: what it does differently  [IDEA, David and Counsel, 2026-10-28]

David: the CPU is roasting while the GPU is fine, "we gotta put that thing to work". What it needs is
**a high-bandwidth but shallow pipeline**, shallow because almost anything it needs about the world is
**rapidly precalculated and stored, easy to access and compact**.

Today, roughly: the CPU works out the recipe, builds every mesh, hands each one to Bevy as its own asset,
and decides what to draw; the GPU just draws what it's given. The kung fu turns that round.

1. **Upload voxels, not meshes.** Sections are already compact (uniform sections, palette compression).
   Send *those* to the GPU and keep them there. A dug block is a few bytes uploaded, not a section
   re-meshed on the CPU.
2. **The GPU finds the faces.** Binary greedy meshing is bit tricks, which is exactly what a GPU is fast
   at. A compute shader turns a section's voxels into a list of visible faces, in microseconds.
3. **A face, not four corners** ("vertex pulling"). Each visible face is one small record (where, which
   way, how big, which block and shade), and the vertex shader makes the corners itself. Several times
   less memory and bandwidth than today's packed vertices, and quarter blocks are just smaller faces.
4. **The GPU decides what's seen.** Per-section occupancy masks on the GPU (32³ bits, 4 KB each) feed a
   compute pass that throws out sections outside the view and **behind mountains** (testing against last
   frame's depth, "Hi-Z"). With 500-block peaks, whole valleys behind them are skipped.
5. **One draw for everything.** The GPU writes its own list of what to draw (multi-draw indirect); the
   CPU never touches per-section draw calls.
6. **Draw once, shade once** (a "visibility buffer"). The geometry pass writes only which face each pixel
   shows and how far away it is. One full-screen pass then does all the shading: textures, slot palettes,
   light, the cone map, fog, sky, palette cycling. Shading costs per pixel, not per face, and changing the
   light or the fog never re-meshes anything.
7. **Light stored compactly beside the voxels**, updated only where an edit happened, read by the shading
   pass.
8. **The far view on the same path.** The far surface from a height texture the game makes from the
   recipe (data, not a copy of the formula); built things far away as coarser voxel versions of their
   sections ("voxel mips"), which is what "blocks far only where changed" becomes.
9. **Never wait.** The CPU only generates voxels from the recipe, in parallel, on its own threads, with a
   budget; the frame never waits for it. What the CPU does is small and easy to measure.
10. **Glass done right.** Minecraft's glass hurts because every block is a framed pane, the faces between
    neighbouring glass blocks are drawn, and layers stack up and flicker. Instead: glass joins into one
    surface (only the outer skin is drawn, as `incus:outline` already does) with connected textures, and
    see-through layers blended without sorting (weighted blended transparency, cheap). **Connected
    textures from a tile map** [David]: edge and corner pixels don't repeat on every block; a tile map
    fills any shape, so a glass wall has one frame round its outside (the same idea as David's shore
    ripple tile map).

**Frontier option, not core:** run the recipe itself on the GPU from the *same* Rust code (rust-gpu
compiles Rust to GPU shaders), which would keep one recipe and take its cost off the CPU too.
Experimental; worth a look only if the recipe turns out to be what roasts the CPU.

**The principle** (David): rewrite every trick in the book for our specific cases, use the compromises
as aesthetic bonuses, and gouge our advantages. That's enough for Incus Artus 1; "for Incus Artus XL we
have to discover new math."

**Temporal reuse** [IDEA, David and Counsel, 2026-11-01]. David asked about the GPU "temporal" tricks
that "just pretend it knows what its doing sometimes and its pretty seamless". They work because a frame
is mostly the last frame. The family:
- **Upscaling:** draw fewer pixels and rebuild the rest from earlier frames. FSR runs on any GPU,
  DLSS needs an NVIDIA RTX card, XeSS is Intel's.
- **Temporal anti-aliasing:** smooths edges using past frames.
- **Frame generation:** makes in-between frames. It adds delay, which is a poor fit for first person.

*Counsel:* the deep idea fits the projection exactly (`vision/projection.md`): an answer stays true
until its stage changes, so reuse it.
- **Spread the slow work over frames:** the far view, light and shadows updated a slice per frame,
  never all at once.
- **Upscaling only where it doesn't hurt:** temporal methods soften crisp pixel textures, so the near
  world stays sharp, while the far view (or the far view's light) could be drawn at lower resolution
  and rebuilt. **David, 2026-11-01:** it must be an **artistic upscaler**, "simultaniously fast and a
  joy to look at", because "every element has to own its compromise artistically"
  (`docs/decisions.md`, Look & UI, with the candidates to try).
- **Impostors:** far trees as sprites, already planned.
Decide with the graphics pass, measured.

**David's old PC versus a weaker new laptop** (a Titan X Pascal with a delidded 8086K ran a small
Steam game badly, where a laptop's Intel graphics ran it well): *Counsel:* most likely the drivers or
the graphics API the game used on that card, not a temporal trick. This game showed the same thing on
2026-10-24: 6 FPS on Vulkan against 240 on DX12, on the same machine. The path the work takes can
matter more than the chip.

**Honest about "best on earth":** each of these is known (GPU meshing, vertex pulling, GPU culling,
visibility buffers, as in today's best engines and the fastest Minecraft renderers). The edge is the
combination, fitted to a world that's a recipe with known content, and David's eye for the flaws.

**First, measure:** Builder is finding what roasts the CPU now (the recipe, meshing, or Bevy's per-mesh
path). The order below should follow what that shows. **First finding (2026-10-28):** the fast-flight
stutter was the **near light window** (the square map of the ground's height, slope and cone shading
around you, remade from the recipe as you move) running on every core at once, so Windows paused the
game's own threads; limited to a few cores, the flight test ran smooth. It's the recipe being worked
out again and again at runtime: the case for "precalculated, stored, compact", and a candidate for the
GPU.

## Builder's assessment (2026-10-28)

**The stutter, measured and fixed (365bc28).** Automatic flights at 2,000 blocks a second, render
distance 64: 247 slow frames before (1% low ~9 fps); the near light window read the recipe ~4 million
times per window on all 24 cores, about twice a second, so Windows pre-empted Bevy's threads. Now the
window slides (the overlap is copied, only the new strip is read), on a quarter of the cores: 14 slow
frames (load only), 1% low 74-88, steady 120 at the cap, chunk-making 60% faster. **Bevy's mesh path is
not the bottleneck today.**

**Right in the proposal:** contracts first, the interface, vertex pulling, GPU face finding, Hi-Z, far
blocks only where changed.

**Wrong or missing:**
1. **The recipe is the CPU's real load** (chunks, far tiles and light all read it): add a recipe item,
   shared column heights worked out once and used by all three; rust-gpu later.
2. **The CPU still needs voxels** (collision, raycasts, the sims, saves): two copies, synced on every
   edit, and the memory to match.
3. **The visibility buffer** is the biggest and riskiest part, and transparency needs a normal pass
   anyway: defer until shading cost is measured.
4. **GPU face finding** fits (32-wide rows as one number each) but is hard to debug: keep the CPU mesher
   as the reference behind a switch, and test against it.
5. **Hi-Z:** high payoff with tall peaks, medium difficulty.
6. **Multi-draw indirect** on the 980 Ti through wgpu on DX12: low risk, with a fallback; buffers paged to
   fit the default binding limits.
7. **Bevy's render internals change between versions:** stay pinned on 0.20.
8. **Missing: a stress scene** (a quarter-oak forest stand-in) to measure against before trees exist.

**Builder's order** (S small, M medium, L large): (1) the interface over today's path, S; (2) slot
textures and the texture system before David paints, M, independent of the renderer; (3) the shared
lighting module, S (also fixes near/far brightness); (4) face records, shared buffers and multi-draw
indirect behind a switch, measured on the stress forest, M; (5) GPU face finding, L; (6) GPU and Hi-Z
culling, M-L; (7) far blocks only where changed, M; (8) instancing, moving objects and transparency
when trees arrive; (9) the visibility buffer only if measurement calls for it.

Counsel's notes: agreed throughout. The shared column heights are David's "precalculated and stored"
in its first, cheapest form. Step 2 needs David's colour session first (brightness A, B or C, and how
many shades a material gets), since slot textures are built around those answers.

## Suggested order (first draft, superseded by Builder's above)

1. Find the stutter's real cause (in progress), so the case for each part is measured.
2. Settle the contracts: interface, vertex format with room for slot textures, quarter blocks and water
   levels, the texture system, the lighting module.
3. Shared buffers and one draw for the near terrain, the old path behind a switch.
4. GPU culling.
5. Blocks far only where changed.
6. Slot textures, then the block register suite on top of them.
7. Instancing (trees, plants, items) when the trees arrive.

Risks: Bevy's render internals change between versions more than the rest of Bevy, so stay pinned on
0.20 and upgrade deliberately; bugs that are ours (flicker, missing sections, one GPU brand misbehaving),
which is what the switch and the soak test are for.

## The far view by foveated ray marching  [IDEA, David and Counsel, 2026-11-02]

**Working name, David 2026-11-02: the moxel-vonster**, his own "voxel monster" from 2026-10-28 with
its first letters swapped. Counsel's other offers were Fovea, Querent, Speculum, Augur and Wherenot.
After build 19, David: "other than a transition glitch, the far view on raymarching actually looks
more 'germaine' than the previous method did. i'd say we're doing the right thing."

Written from David's conversation with Counsel during caves stop 4 (builds 5 to 10, the far copies).
**Not decided and not for building.** David: "im ok with some discomfort with rendering now if i
understand theres a better way to do it once the world is full enough i really need to see it better."
He wants it discussed as the way to "solve the actual problem at its root". For Builder's assessment
when David asks for it.

**The framing** (David: "the stuff we are doing with graphics now is figuring out how to draw nonrecipe
content right"). The far surface can draw only one thing: the recipe's open ground, smooth, with a
colour. Everything it can't draw goes through the far copies: caves and overhangs (even though the
recipe makes them), David's changes, and later felled trees, roads, mines and whole settlements seen
from a hill. The renderer doesn't care who made a thing, only whether the smooth surface can show it.
The hard part is making the copies and the far surface agree where they meet. Every build in stop 4
fixed one disagreement: where one ends (5), shading (6), colour (8, the far surface sampled too
sparsely), slopes and seams (9, 10). Still open: the skyline flicker, fog turning a dark mouth sky
coloured. "1 always equals 1" applied to distance: the far view must be the same world, coarser. The
payoff: once this is right, anything that changes the world shows far off by the same path, so a
village shows from 2 km without code for villages.

**Why a layer on top can't do it.** David wished for "a second layer at full resolution" drawn on top
of the far view, only where pixels differ. A layer on top can *add* (a tower, a tree) but not *take
away*: a cave mouth, a pit, dug land or a felled tree is a hole, and the far surface in front of it is
solid. To show a hole something must cut the surface away first (today's mask), and the seams come
from the cut.

**Ray marching.** Each pixel sends a ray into the world and asks what it hits first, in a grid read
at the size of that pixel. Holes just work: the ray goes into the mouth and finds the dark inside. No
mask, no layers, no seam. A 1080p screen is about 2 million pixels; GPUs march millions of rays a
frame, so even marching every far pixel is affordable.

**The recipe as a mask; the CPU guides the GPU.** David: leverage the recipe "to efficiently use the
cpu to guide the gpu". The CPU builds, once per area and again only where something changes, a map of
the world: where it's plain recipe ground, where it's interesting (caves from the cave plan, changes),
and how high and low each area reaches. Rays use it to take long steps over plain ground and to start
where an interesting area begins, not at the camera. The GPU works out each frame's distances itself
with a cheap first pass over that map: it's faster, and distances sent by the CPU each frame would be
a frame late. The CPU stays free for the settlements, which will be its real work. **What carries
over:** the far copies already work out what each coarse cell holds (its top block, light, how full
it is). That is the data a march reads; today it's meshed, later it would be uploaded as a grid.

**Resolution that falls off with distance** (David: "warp resolution as rendered so the screens
resolution is a continuous curve into the distance ... the screen resolution is what anything nearby
would render at, and looking up a long hill, the resolution shrinks, pixel by pixel"). Fewer rays far
off is real performance, in proportion. Three ways, for David's eye to judge side by side:
- **A. Steps by screen tile:** 1×1 near, 2×2 up the hill, 4×4 on the far ridge. Crisp square pixels;
  a visible line where sizes meet. NVIDIA calls the hardware form variable rate shading (whether
  wgpu exposes it is unknown); a ray marcher can do it alone, since it chooses where rays go.
- **B. A continuous warp:** render into a squeezed image (far rows get fewer pixels) and unsqueeze it.
  VR headsets do this as *foveated rendering*, sharpness falling off from where the eye looks; David's
  version falls off by distance. No steps, no blending; the pixels come out as gradually stretched
  shapes. David: "the messy tetris from the warping resolution probably isnt messy enough to make it
  look wrong."
- **C. Cells anchored to the world:** each ray reads a world cell sized by distance and draws it flat.
  The pixels belong to the land, so they don't shimmer as you turn; sizes step 1, 2, 4, 8 blocks, but
  on the land, not the screen.

**David's snap filter:** once a far pixel stands for 4 screen pixels, run "a literal photoshop filter"
over the masked area that snaps it to a grid (a mosaic). Cheap, and it gives the pixel look on top of
B. *Counsel's catches:* (1) a grid fixed to the screen shimmers when the camera moves, because each
frame the squares sample different land; the cure is to anchor it (to the world, as in C, or by
snapping the camera the way pixel-art 3D games do), and to average each square, not pick one sample;
(2) a filter alone saves no work, only the warp or fewer rays does; (3) squares at the mask's edge,
half near world and half far, need a rule (near wins).

**The dome** (David: "anchored world grid that revolves like a dome around you for anti shimmer").
The far view is marched into a grid of directions fixed to the world, centred on the player, not to
the screen, and the screen reads from it. Turning your head doesn't move the grid, so turning can't
shimmer; walking shifts far things only by a hair, since far things barely move as you walk. Its
cells hold the warp: each sized by how far the land is in that direction, a cell size David sets to
taste until it looks and performs right. **It updates rather than redraws** (David: "it could come
last in the render que, and work by update; if a dragon in the sky is far enough away that he is
pixelized, it does not matter if he slightly lags behind"): each frame refreshes a budget of cells,
first those where something moved or changed, then those near the threshold, then the rest, behind
you least. *Counsel's notes:*
- **A cube-shaped dome**, not latitude and longitude: lines of longitude pinch together straight up
  and straight down, where the dragons are.
- **It stores depth as well as colour**, so the near world and nearer things draw in front correctly.
- **Cells near the threshold** need refreshing most often, or the far land will slip against the near
  world where they meet; the faster you fly, the more must be refreshed (David tests at 10,800 mph).
- **Cells change size with hysteresis** (a little resistance before changing), or a cell on the
  boundary would flip between sizes and flicker.
- **Zooming** (a spyglass) would show the dome's coarseness and need finer cells where you look.
- **A dragon crossing from far to near** passes from the dome to normal drawing; the style should
  match on both sides of the line.
Games have cached far distance in a panorama fixed to the world before; the warp, the recipe map and
the budgeted update are what make this one ours.

**David's stance, 2026-11-02:** "i absolutely think an exotic solution to rendering a big world is
worth it. i just think its the biggest obstacle to success and cleanly solves all our problems in one
stroke rather than balancing multiple systems with overlapping jobs. a AAA studio can afford to do
that, but its just me you and builder here."

**The goal, David 2026-11-02:** "a renderer that we can forget about, that will take everything we can
throw at it and look good. it should look and act good enough that everyone says 'how?'. for a left
field indie game, that will make people who only see AAA stuff look twice." On speed: "i dont expect
the player to move 200+ blocks a second, so im tolerant of the edge case that is development" (fast
test flights may show the dome lagging; normal play must not).

*Counsel:* "forget about" means its cost follows the screen, not the world. A march costs per ray,
and the rays are set by the screen and the budget, so a forest, a city or a dragon flock adds data to
the map but barely adds frame time. Past its budget it gets coarser for a moment, never slower. That,
the stable contracts above, and no pop-in or shimmer are what make it something we stop thinking
about. David: "i want freedom from asking how each new thing has to be handled by what we have."

**Testing it doesn't need trees** (David: "i can make a bunch of caves with brazieres in them and that
should be about the same to it, right?"). *Counsel:* the "trees at scale" unpark trigger above was for
the drawing path, whose cost grows with content. For the marcher's first step, caves with braziers
are the harder test: holes, dark insides, light seen from outside at night (the stand-in night slider
exists), all at the seam where today's copies struggle. What caves don't test: **fine, sparse
detail** (leaves and thin branches only partly fill a far cell, so a far forest must read as soft
canopy, not solid blobs or nothing, and fine detail shimmers most) and **sheer amount** (memory, and
the claim that cost follows the screen). Both can be tested with a stand-in forest of thousands of
simple block trees scattered by a test tool, which is the stress scene Builder asked for, without
the real tree system.

**Layers, back to front:** sky; a horizon ring of far mountains past the data's edge, repainted now
and then (a ray that leaves the data reads it, then the sky); the far view, marched; the near world,
drawn as now. **End state:** march the near world too, so there's one pass, full resolution up close
falling off along the curve, nothing to blend. Biggest version of the change: water, leaves and glass
need see-through marching, moving things (animals, a falling tree) stay drawn the usual way, and much
of Bevy's drawing is replaced.

**Why chunks at all** (David asked). Chunks are filing cabinets, not paintbrushes: memory and saving
(only changed chunks are saved), generation in parallel, editing (one chunk re-meshed per placed
block), skipping what's off screen, and later the simulation's active areas. The seams of stop 4 come
from using the filing unit as the drawing unit. With marching, a chunk is where the data is filed and
stops being something you see; the unit for saving and simulation needn't match the one for drawing.

**The same grid gives more:** far lights at night (a ray finds a lit brazier inside a distant mouth,
`docs/worldgen.md`), long sun shadows over far land, clicking far things (orders to a distant tower),
and later light bouncing.

**David's joke** ("i wouldnt really try for to do it"): a "dynamic filing cabinet" in higher-dimensional
maths, always one small movement away from any shape the terrain must show, solved each frame by
trying a step up and a step down and dropping the wrong one, in under a millisecond. *Counsel:* it has
real cousins. Clipmaps keep a ring of detail centred on the player and update only the strip that
moved; temporal reuse starts each frame from the last one; a solver warm-started from last frame's
answer is the step-up, step-down idea. "A big formula that renders any view" is what neural scene
methods and Gaussian splats are, though they suit fixed scenes, not an editable world. The serious
part: a frame is mostly the last frame, so the cabinet should move, not be rebuilt.

**Counsel's draft path** (each step seen by David before the next):
1. **March the far view only**, past the real chunks, replacing the far surface and the copies there:
   one ray per pixel, writing depth so Bevy's near world draws in front. Data: the height map with
   each area's highest and lowest point, plus coarse grids for the interesting areas (today's copy
   cells).
2. **The resolution curve:** tile steps first (A), then the warp (B) and anchored cells (C) with and
   without the snap filter, for David to choose by eye.
3. **Temporal reuse:** carry last frame's hits forward and march again only where they don't hold.
4. **Far lights and sun shadows** from the same grid, when real night arrives.
5. **Move the line inward** toward the end state, if measurement and David's eye call for it.

This would replace Builder's step 7 ("far blocks only where changed") in the order above, and leaves
steps 1 to 6 as they are.

**Open:** which of A, B, C David likes (needs a test); whether the near world is ever marched; when
(David: "once the world is full enough i really need to see it better"); Builder's assessment of costs,
the wgpu side (compute shaders and depth output on the 980 Ti), and what's wrong or missing.

### Builder's assessment (2026-11-02)

**Verdict:** sound, and it fits the code we have. Most of the CPU side carries over: the far rings'
heights and colours (`distant.rs`) and the copy cells' tally (`far_copies.rs`). What goes is the
drawing side: the sheet mesh, the copy meshes, the plain-ground stand-ins and the mask. Step 1 needs
no compute shaders and no dome.

**Wrong or overstated:**
1. **One line stays:** where the real chunks end and the march begins (the shown-chunks map already
   exists). Both sides show the same blocks at full size there, so it can be invisible if the march
   textures its hits with the same block textures. One line we control replaces many.
2. **Marching alone doesn't fix two open problems:** the dark mouth turning sky-coloured is the fog
   formula; skyline flicker is aliasing, cured by cells no smaller than a pixel, colours averaged per
   cell, and a few extra rays at silhouettes. The march puts the fix in one place, but it's still work.
3. **Missing: water** from day one, as today's far view has it: the sea as a flat plane, rivers as cells.
4. **Missing: partial coverage.** The cell format should carry how full each cell is from the start
   (the copies' "must be solid" tally already works it out), even if step 1 stops at the first solid hit.
5. **Missing: a GPU timer.** Frame rate alone won't show the march's cost. Bevy 0.20 can time GPU
   passes, and the 980 Ti on DX12 supports it.
6. **Not a conflict:** the march is drawing, not projection, so ordinary floating point is fine. It
   reads only data the game has worked out, never the recipe formula.

**How A, B, C, the dome and the warp really behave:**
- **C sits under A and B, not beside them.** A coarse march is C by nature: a ray reads a world cell
  sized by distance and draws its averaged colour, the same for the same cell however you turn, which
  removes most turning shimmer without a dome. A and B only decide how many rays are cast.
- **B assumes distance follows screen height,** which fails looking down from a peak or at a near
  ridge. A, with tile sizes from a cheap distance pass, adapts to what's there. Still worth showing B.
- **Cells near the line are already pixel-sized:** the far view starts about 1,000 blocks out, where a
  block is about 0.8 of a pixel (1280×960, 90°). The far view is a modest part of the screen (a band
  toward the horizon, plus mountains), so per-pixel marching should cost a few milliseconds at most on
  the 980 Ti. Estimate, to be measured.
- **The dome is a cache: turning is free, moving isn't.** At the line, walking one block shifts the
  land about a pixel; at 100 blocks a second the band near the line must be redone every frame, and
  that band has the most detail, so the dome saves least where the pixels are. Moving uncovers spots
  the cache never saw; a spyglass needs finer cells. It's step 3's temporal reuse in another shape, and
  the most complex part of the plan. **Recommendation: no dome in step 1;** decide at step 3 with the
  timer's numbers, dome against ordinary reuse of last frame.

**wgpu, Bevy 0.20 and the 980 Ti** (checked in the 0.20 and wgpu 30 sources):
- **Depth from a shader works** (Bevy's meshlet pass does it). Simplest route: a full-screen triangle
  as an ordinary material in the opaque pass, writing colour and depth, so the near world, water and
  objects sort correctly. Cost: depth-writing shaders lose the early skip, so pixels the near world
  covers still get marched. If the timer says it matters, a pass after the near world that reads depth
  and skips covered pixels: a custom render node, medium risk (Bevy internals; stay pinned on 0.20).
  Panini's FullscreenMaterial can't do it (no depth attachment).
- **Data:** materials take storage buffers (ShaderBuffer, `#[storage]`), 128 MB per binding and 256 MB
  per buffer by default. Step 1 fits: far rings about 4 MB with the highest/lowest pyramid, copy cells
  as sparse bricks tens of MB at most, the chunk mask kilobytes. Updating re-uploads the whole buffer,
  fine for step 1; small partial updates later need render-world code (medium).
- **Compute shaders** work on DX12 and Maxwell, but aren't needed until temporal reuse or the dome.
- **The 980 Ti** (Maxwell, 6 GB) handles storage buffers in fragment shaders, depth writing, 3D
  textures and timestamps. No hardware variable-rate shading (Turing onward, and wgpu doesn't expose
  it), so A is done inside the march. Async compute is weak on Maxwell; don't rely on it.
- Panini still works: it re-maps the finished image.

**Risks:** GPU code is hard to debug, so keep a CPU twin of the march in Rust for tests (one ray, the
same data, the expected hit), plus "who drew this" colours. The worst case is rays skimming flat land
toward the horizon, which the highest/lowest pyramid is for. Thin things far off (a 3-block mouth at
3 km) average away by the same share rule as now.

**Stop 4's measurements:** a short baseline now, from build 11, as the yardstick: the march should at
least match "copies off" while showing what "copies on" shows. Copies off, copies on at default, one
fast flight; the sharpness sweep matters less now. Frame cap to 0 first (at the preset's half-refresh
cap every reading would be 120). Keep the 50 cave mouths as the march's first test scene, with braziers
in some for the night test.

**Builder's order for step 1** (S small, M medium, L large; each seen by David before the next):
1. Baseline from build 11. David, about 10 minutes.
2. A GPU timer on the debug panel (sheet, copies, near world, march). S.
3. The march over the far rings only: plain ground and sea, shaded with the shared `land()` and fog,
   starting where the real chunks end, writing depth; a "far view: sheet / march" switch for A/B, with
   a who-drew colour. M.
4. The highest/lowest pyramid for long steps over plain land; timer readings. S.
5. Interesting areas: copy cells uploaded as sparse bricks, rays dropping into them for caves, changes
   and objects; copy meshing off while the march is on. M-L.
6. Night: cell light, so braziers glow from far mouths. S-M.
7. Retire the sheet and copy meshes once David prefers the march, kept behind the switch as the
   reference for a while. S.
Then Counsel's step 2, the resolution curve (A, B and C side by side, the snap filter).

*Counsel's notes:* agreed throughout. Water was my gap. Leaving the dome to step 3 is right: it's an
optimisation, and the timer will say whether it's needed, while C gives most of its shimmer-proofing
for free. The single line at the real chunks' edge, with matching textures, is the honest version of
"no seam".

**The line is a squiggle** (David: "that line though would be the shape of the horizon on the screen
though; its not a flat line around us, its a line squiggled across the screen"). It's a stepped ring
of chunk edges on the ground about 1,000 blocks out, so on screen it runs up and down every hill it
crosses, can cut a far cliff in two, and hides behind nearer hills. David's "jitter" along the snowy
ridge in stop 4 is this line today. *Counsel:* nothing has to know its shape, because each pixel's
ray starts where it crosses the edge in the world; but any mismatch shows as a wiggle through the
land, which is worse than a straight line. So: (1) both sides must match to the pixel there (the
same textures averaged the same way, the same light and fog), which is possible because a block is
about a pixel at that distance; (2) if a trace still shows, a **dithered band** about a chunk wide,
where pixels choose near or march in a fine fixed pattern, so the line has no edge to see (the
real chunks streamed just past the circle are there to draw it); (3) the end state, marching the
near world too, removes it. Check it in step 3 with the switch, on a ridge.

**Camera-relative maths** (Builder, build 14, 2026-11-02): build 13's march shook even standing
still ("ha, it works, and it does not look bad, but its jiggly"). Each ray's direction came from two
world positions a tenth of a block apart, and at ~18,000 blocks out floating-point precision made
the difference noisy; build 14 works it out in the camera's own view. The same limit is probably
behind David's "vibrating blocks" up close. **For the contracts:** everything the renderer draws
should be placed relative to the camera, not in raw world positions, so precision never runs out
however far the player travels.

**The DX12 shader compiler** (Builder, build 18, 2026-11-02): DX12's old shader compiler, FXC, refused
the march shader ("unable to unroll loop") because loops over the three axes indexed a vector by the
loop counter; build 19 writes each axis out by hand. For the renderer work: FXC won't take loops that
index vectors by the loop counter. *Counsel:* wgpu can also use DX12's newer compiler, DXC, which
lacks this limit; worth Builder checking whether Bevy 0.20 can switch to it, and what it costs to ship.

**Why a hybrid at all?** (David, after build 20: "why do we need it to be a hybrid, isn't raymarching
the immediate surroundings the point"). *Counsel:* the hybrid is the order of work, not the goal. The
far view came first because it was where the old method failed, it covers a small share of the
screen, and nothing broke on the way. Near up close, a march must handle crisp full-size textures,
models (the brazier), see-through things and moving things; it wins no line at all, edits as data
updates with no meshes, and cheap fine detail. A cleaner split than near and far: **the world is
marched, things are drawn** (terrain, caves and buildings in the grid at every distance; models,
creatures and effects drawn on top). Build 21's bug (a peak's top drawn by nobody, each side thinking
the other had it) is the hybrid's cost in small. **The line as a slider:** after parts 6 and 7, "the
march starts at N chunks" on the Render page, dragged inward 32 → 16 → 4 while David looks and the GPU
timer reads; his eye and the timer decide how far in it comes. If it reaches all the way, the hybrid is
gone, and the floating origin (watch list 22) comes with it. David asked for it to be noted to Builder.

**Step 1 through, 2026-11-02 (builds 11 to 26), done enough to go on.** The march draws the far land,
the sea, the copies (caves, changes, objects) and braziers' glow at night; copy meshes are no longer
made while it's on (the old way stays behind the switch). David: "the far view on raymarching
actually looks more 'germaine'", "a more, 'verasimilitudinous' look", "it seems smooth". Settled after
a fast flight, against the build 11 baseline: worst frame 117 → 31 ms, slowest 1% 29 → 47 fps, a far
copy 95 → 12 ms to make. What's left of the hitch is the near world's meshing as chunks stream in.
Record: `docs/far-view.md`. Next: the "march starts at N chunks" slider, the near/far look match, then
Builder's assessment of marching inward.

**The line as a slider, build 27, and Builder's assessment of marching near** (2026-11-02, full text
in `docs/far-view.md`). David at 64: "i don't think we've ever had the border *more* seamless ... quote
hard to see the seam, the lighting is very even", and pulled in: "did not show actual regular detail
like i thought it would" (inside the line the march only has smoothed land and copies where the world
differs). Builder: marching near needs every near chunk's blocks on the card (~300 MB at full detail
to 32 chunks, so coarser with distance), their textures, daylight and placed light, pieces, bars and
water; a few ms a frame on David's card, perhaps 5 to 15 on a 980 Ti, against 1 to 2 for the near
meshes now; it would end the CPU's meshing (26 ms spikes in fast flight) but send blocks instead;
large, floating origin first. **Her recommendation, taken by David ("ok. lets do it"):** an experiment
first, build 28: a band 6 chunks wide past the pulled-in line copied whole and drawn with the near
world's block textures, to see block detail by the march and its cost. If it holds up, plan the whole
change; if not, keep the line at 64 and spend the effort on the near world's meshing.
**Build 28's result:** with the march at 2 chunks the card took 1.42 ms in an enclosed spot (2.5 ms
at 64); over a big view both about 3.2 to 4 ms, on David's 5070 Ti (the 980 Ti untested). The 8.25 ms
frame is held by the processor, since real chunks are still made and meshed underneath. David: "there's
a mismatch but they both read fine": the textured band reads as blocks, brighter and flatter (no hill
or step shading yet). A gap of sky between the real chunks and the band was fixed in build 29.
*Counsel:* on these numbers the experiment holds up, so far, on David's card.

**Finishing the renderer** (2026-11-02, after build 31; Builder's path R1 to R7 is in
`docs/far-view.md`, last section). David: "i lean towards finishing the custom renderer ... having
that done, and a graphics solution set up that we can rest on, would be the best thing going forward.
that would be the last big difficult technical piece, i think, except for the behavior engine."
*Counsel's advice:*
- **Agreed, and the order makes sense:** R2 gives the processor back, and that processor time is what
  the behavior engine will spend. Renderer first, then behavior, is the right way round.
- **R1 then R2, as Builder says.** Three precision bugs in one day earn the floating origin its place
  first.
- **Measure on weak hardware at R2, not at the end.** The 980 Ti numbers are guesses ("several times
  slower"). If David has no 980 Ti-class machine, the nearest stand-ins are an older PC, or the
  laptop's own integrated graphics, if it has them, as a pessimistic floor. **A checkpoint after R2:**
  if the weak-hardware numbers are bad, re-plan before R3 to R7.
- **Water and glass may not need see-through marching (R4).** The march already writes depth, and
  today's near water is drawn as a surface over it. Keep water and glass as drawn surfaces, blended
  over the marched world, the "things are drawn" side of the split. The ray marches the riverbed; the
  surface is drawn on top. That takes the hardest piece off the path, at least near; march it later
  only if the surfaces fall short. For Builder to assess.
- **The near look needn't match today's.** Today's near rendering is a placeholder until the graphics
  pass; let the march set the look where that's easier, keeping what's David's (crisp pixel textures,
  the light), and tune it in the graphics pass.
- **A test world, kept as a saved preset with fixed viewpoints,** and the same spawn spot every time
  for A/B (the "Render World and Spawn" reminder): a stand-in forest, a stand-in village of block
  buildings, a lake and a river, a glass wall, a quarter-block staircase, caves with braziers for
  night.
- **Keep it playable and keep the switch** at every step until R7, as step 1 did. After R7: stop and
  play in the world for a while before the resolution curve.

*Builder's reply:* agreed on all six. **R4 reassessed:** inside the line the march treats water and
glass as "pass through", marching the riverbed and far bank, and the mesher builds only the
see-through blocks' faces (a few per section, so R2's saving mostly stands), drawn over the march as
now. Small to medium instead of large, folded into R2. Far water stays the march's plane and river
cells. It can't tint water by depth (today's can't either); depth-tinted or refracting water later
would bring see-through marching, or a depth copy, back. One seam to watch: drawn near water against
marched far water at the line, both flat at one level, so a colour match. **The path now:** R1 floating
origin → a "test world" button building the test content at fixed spots (M) → R2 the near world from
the chunks, see-through blocks still drawn → checkpoint on weak hardware → R3 one lighting → R5 pieces
and bars → R6 sizes by distance → R7 the line all the way in. True see-through marching parked.
**David on the near look:** "im not attached to the near look", so the march may change it (Counsel's
advice keeps the pixel textures and the light). Weak hardware for the checkpoint: the laptop's
integrated graphics, and perhaps a friend's PC (David is asking for its specs). **Go on the path,
2026-11-02:** "lets do it". R1 first.

**R2 measured, and "every single thing marched"** (2026-11-03; Builder's numbers in `docs/far-view.md`).
Line 64: 310 fps, card 2.90 ms, frame 3.22 ms, "it does look beautiful". Line 8 with R2: 348 fps, card
1.76 ms, frame 2.87 ms; the look fails (thick dark-and-red contour lines, dark blotches, squares at
tile edges). David: "if we can have every single thing marched, i think we can make that work really
well and look unbelievably good, as well as being flexible enough to basically forget about as we
fill the world with content and behavior."

*Counsel, from reading `march.wesl`, `terrain.wesl`, `far_copies.rs` and `mesher.rs` (none of this
tested yet):*
- **The earlier "8 ms, held by the processor" was mostly the 120 cap and hot fans.** Uncapped, the
  frame at line 64 is 3.22 ms and the card is 2.90 of it. So R2 gains little speed standing still
  (0.35 ms), and **speed is not yet the case for marching near.** The case is the claim that cost
  follows the screen as content grows, and that is unproven: nice25 has no content. It needs a
  content-heavy comparison, line 64 against line 8.
- **Why the lines are thick: likely the cell size, not only missing anti-aliasing.** `scale_for` at
  the default sharpness 2 gives 2-block cells from about 500 blocks and 4-block cells from about
  1,000. So at line 8 much of the view isn't blocks at all: its steps are 2 to 4 blocks tall, where
  meshes have 1. A sliver at one ray a pixel would alias, but wouldn't be thicker. The same cause
  gives the squares where tile sizes meet.
- **Why it's flat: `copy_textured` leaves out `ground_light`.** Near: colour × face shade ×
  `ground_light` (the lie of the land, height tint, the feet of steps) × underground light. The march's
  whole copies: colour × face shade × light map × daylight². This is what David's "brighter and flatter"
  is.
- **Builder's cause (c) doesn't hold as far as the code shows:** the mesher takes one flat "light in
  front" per face, the same as the march's air cell. Smooth daylight isn't the gap; `ground_light` is.
- **Blotches, two candidates:** moire, since the level comes from distance alone *and* the march has
  no random texture turn (all tops the same way, which is what makes whorls), against the near
  shader's `textureSampleGrad` with the turn; or daylight, since an air cell averages in the hidden
  air of solid cells beside it, squared.
- **The fix for all of it starts with one shading function:** near and march call the same sums,
  as lighting.wesl began to do, so they match by construction, not by tuning.

*What "every single thing" needs that the list so far missed:*
- **Moving things needn't be re-voxelised, or drawn as meshes.** Each thing keeps its own small grid in
  its own coordinates, and moving it only changes where that grid sits. The graphics card draws the
  thing's box, and each pixel of the box marches inside its grid and writes depth (Teardown's way, as
  far as Counsel knows). So figurines are rigid parts, a felled tree is one part, and a sprite is a ray
  meeting a flat quad. This corrects Counsel's earlier "models and sprites are drawn": they can be
  marched too. Objects already read their shape from the model pixel by pixel (objects stop 1b).
  Particles, the hands and the HUD can stay drawn.
- **Moving lights** (a torch in hand, spells, dragon fire) can't be baked into cells. They need a
  short list of lights checked at each hit, with a ray toward each for shadow, which the march makes
  possible.
- **A moving sun:** hill shading and step cones are baked for one sun; real day and night isn't tested.
- **See-through marching comes back, and it's better than feared:** a ray walks front to back, so
  glass and water need no sorting, and it adds depth-tinted water and an underwater view for free. An
  eighths water level is a flat plane inside its cell. This reverses the parking of R4 only if David
  holds to everything marched.
- **Sizes chosen per ray, not per tile** (Builder's voxel mips): no tile squares at all, with the
  change of size dithered, which suits the art.
- **Digging:** a touched tile takes ~12 ms in the background. A dug block must vanish in a frame or
  two, so patch the one cell at once and relight behind it.
- **The foreman view:** orthographic rays and the height slice aren't checked in the march. The slice
  is a gift: rays just start below the cut, with caps for free.
- **The cost of a miss:** with no near meshes left, every pixel is a ray, so the iGPU check comes
  before more building, not after.

*Order, Counsel's advice:* a new step before R3, "line 8 looks like line 64": first the two tests on
build 35 with no new code ("Light only", then sharpness 4), then the iGPU check; then one shading
function with the texture turn; then sizes per ray; and anti-aliasing last, since it's the costliest
and the others may remove most of what it would hide (temporal methods soften crisp pixels, so edge
rays or the deliberate pixel look before those). Compare screenshots from one spawn spot.

*The tests, build 35* (`docs/far-view.md`, "Why line 8 looks wrong"): "light only" removes the blotches,
so they're texture (moire, no random turn); the lines stay but thin and faint, their thick red being the
dirt-side colour. Sharpness 2 → 4 makes far lines finer, so cell size was part of it. The saw-tooth
edge is the line itself, meshes against copies, shaded differently (no `ground_light` in the march).
Anti-aliasing isn't yet shown to be needed. Builder confirmed the mesher's flat per-face light. Next:
the iGPU check (since put off by David: `docs/decisions.md`, "The renderer: right and fast together,
erring on the side of right").

**One shading recipe, builds 36 and 37** (2026-11-03; `docs/far-view.md`). David: "its improving"; the
blotches and the shading jump at the line are gone near in. Dark tops further off were the coarse
cells against the block-accurate cones; build 37 takes the feet shade off coarse cells, and the land
reads as one surface across the line. David: "i'm really optimistic that we're going to achieve
exactly what we aim for: a bespoke renderer that is both fast, and beautiful, and, the holographic
nature of our worlds makes it possible." Still there: thin red-brown contour lines in the middle
distance (sub-pixel dirt sides, hit or missed whole by one ray a pixel).

**Sizes per ray, Builder's proposal** (for David's word, nothing built): each copy keeps coarser levels
(2, 4, 8 blocks, ~1/7 more room); each pixel picks its level by its footprint, dithered between two;
near a pixel a block, cells blend into the land's look (`land()`, top and side colours mixed by how
much of each the camera sees), so slivers become the right share of side colour; edge smoothing
last, only if needed. *Counsel's review:*
- **Blending must not eat what differs.** Whole copies have no plain-ground mark (`far_copies.rs`,
  `!job.full`), and the slope comes from the near window, which is read from the recipe. So a quarry,
  a village or a cave mouth drawn in the land's look would show the recipe's slope over them: David's
  mouths and crisp braziers would fade. Take the slope from the copy's own column tops (the world as it
  is, nothing asks who made it), and keep real faces where a cell's neighbours break the slope.
- **Glow takes the brightest down the levels, not the average,** or far braziers go out.
- **A ray changing level mid-flight** can find itself inside a coarser solid cell. That needs a rule
  (count it as meeting the top there), or wrong faces and specks.
- **"Fixed to the world" dither:** a ray has no hit point when it chooses, so a per-pixel world dither
  has nothing to key on. Key it on the coarser cell's own hash: each cell decides, the same for every
  pixel that sees it, world-fixed by construction, flipping once as you approach.
- **A reference to judge against:** a debug switch with several rays per pixel (slow, for looking
  only), so every look step has a target, instead of comparing with meshes that won't be the end state.
- **The dial:** the level's bias is the "size to taste" David asked for (the dome, the resolution
  curve), so sharpness becomes per ray; and flat cell colours against the land's look is the artistic
  choice ("every element has to own its compromise"). Show both.
- *Builder took all of these into the plan.* **David, 2026-11-04:** "i like sizes by ray idea ... the ray
  has to pick between two bad choices for the case at the moment. sizing rays fits our philosophy.
  yes, choosing when cheap is always a good idea. a slider for the pixelization rate of terrain as it
  becomes more distant is a good idea too. even with a snapped grid overlay (optionally) id think id
  still want that." Then he asked to think through the light map: "explain what it is and *why* it has to
  be that way, so i can locate the 'but if we...'"

**The light map: replacing it by asking the world** [IDEA, David and Counsel, 2026-11-04; nothing built].
*Why it's the way it is:* a mesh's pixel can only see its own face, so anything about its surroundings
must be worked out beforehand and handed over as a picture (`src/render/light.rs`: the world map at
32 blocks a pixel, the near window at 2, the cone maps). A ray stands in the world's grid, so that
reason no longer holds for what the march draws. David: "we can sneak up on the idea of replacing a
static light mesh since we have preliminary targets either way. i like the idea of just asking the sun
about its shading per ray, if it is efficient it seems like common sense ... i wonder if we can just
infer the suns angle somehow per ray without actually doing the work. but i recall the cone map worked
well; so feet and slope could work very well too."
- **Cheap reads** (Builder): slope from column tops, foot shade from neighbouring cells.
- **Sun rays:** Builder's first read is about double the march where used. *Counsel:* maybe far less,
  since a sun ray climbs out above the column tops in a few long steps while a view ray skims; a low
  sun is the bad case. To be measured.
- **Inferring, not asking** (David's question): the sun's angle is already known, one number for
  the whole world, and how a face turns to it costs nothing. What takes work is "is anything in the
  way?". Two ways to infer that: keep the sky pass's 8-direction horizon heights, so a mountain's shadow
  is "is the sun above the horizon that way?" with no ray and a free moving sun (~40 MB, soft edges);
  and have a cell remember "the sun sees me", refreshed by a budget each frame since the sun moves slowly.
  A short real ray covers buildings and trees.
- **What stays:** light only lightens and darkens David's colours. That's his art rule.
- **Order:** after sizes per ray, which makes all of these cheaper. Builder gives cost and risk for each.
- **Blocked rays are short** (David: "every ray that doesnt find the sun will be significantly shorter.
  that may be useful? and since the sun moves slowly, it only has to do that work slowly"). *Counsel:*
  yes, twice over. A sun ray stops at the first thing in its way, and needs no nearest hit, texture or
  shading. And how far it went is itself an answer: a short blocked ray is a contact shadow (crisp, and
  the feet of steps), a long one a far shadow (soft), so one ray could do the work of both the cone maps
  and the shadows. Doing it slowly holds for land, with limits: digging must re-ask along the sun's
  line, newly seen cells need an answer at once (the horizon map as the stand-in), and moving things
  can't be remembered.
- **Coarse shade levels everywhere** [IDEA, David, 2026-11-04]: "i love the way the brazieres have obvious
  coarse shade levels. that may look good everywhere, any benefit?" *Counsel:* several, beyond the look
  (hand-shaded bands, which suits his "shading with the palette by hand" leaning, `docs/decisions.md`):
  - **It makes "slowly" far cheaper:** with few levels, a slowly moving sun changes a cell's answer
    rarely, so little work and little to store.
  - **Small near/far differences round to the same level,** so they don't show.
  - **It's steady:** a level per face doesn't shimmer, and shadow edges follow the blocks.
  - **It can follow the size per ray:** coarser cells, coarser bands, under the same slider.
  Risks: on broad slopes, bands read as contour lines, the thing just removed from line 8; a moving
  sun moves them in jumps; a flickering light steps through them. Dithering between levels (as the sky's)
  may answer the first two. Two kinds to tell apart: **per face** (what the braziers do, flat per block)
  and per pixel (smooth curved bands). Cheap to try, as one slider of levels in the shared recipe.
  **David, the same day:** "coarse shade has a real look and it does hide blemishes and add some blur and
  consistency ... we should check out coarse lighting once we have regular lighting solved though." So:
  on file, after regular lighting.
- **Contour bands aren't the enemy** (David): "contour bands are how we see shape and depth in land, we
  just need them to all look the same, almost all our graphics fixes are already just making things
  agree with each other or stop conflicting." This corrects Counsel, who counted bands as a risk and
  the lines at line 8 as something to remove. The aim is one kind of band at every distance. It bears
  on sizes per ray's round 2: blending slivers into the land's look could lose the bands in the middle
  distance and leave them only up close. A third look worth showing beside flat colours and the land's
  look: the far land keeping thin bands of its own, at each whole block of height, in the side colour,
  as wide as they'd really be on screen. *Builder's read:* cheap, and steady if filtered: the share of
  the pixel that is band comes from the pixel's height span (from the next-pixel rays) overlapping the
  slab at each whole block of height, so a thin band is a faint line, never hit or miss, and with many
  steps in a pixel it tends to `land()`'s average by itself: near blocks, thin bands and the far average
  as one curve. Height from the copy's own column tops, so it follows dug land and joins the blocks
  exactly (about 4 reads a pixel). Open surface only. She'd make it round 2's main look, with flat colours
  and the plain average to compare. *Counsel:* agreed; check the join where copies end and the far land's
  own height takes over.
- **The principle** (David): "i like the idea of continuous difficulty curves of a single rendering
  method, rays, being used to do work on demand." And: **"variable length rayshading for variable
  resolution raymarching."** *Counsel:* the same dial for both. A shading ray starts at the size of the
  cell it left, and grows coarser as it goes (a cone, not a line), so it reads the size-per-ray levels
  already there. Its length follows what can still change the pixel: up close a long ray, crisp at the
  start and soft far along; far off a short coarse one, with the horizon map answering for mountains. It
  stops as soon as the answer is settled, which coarse shade levels would make sooner. To assess.

**Go, 2026-11-02:** David approved Builder's order for step 1, baseline first: "yes. lets begin. i
think most of the things we need to add the basic game assets are in place, and ive kinda gotta use
them hard to refine them. cant so that wondering what anything really looks like."

### Where the path stands, and one answer for a far slope (2026-11-04, builds 38 to 46)

David: "im finding it hard to tell right now if we're making progress towards the completion of unified
raymarching theory".

*Counsel's picture:* R1 and R2 done enough (R2b, chunks written straight into cells, on file). R3 one
lighting and R6 sizes partway: blocks a cell share the near recipe, while the far land and the banded
coarse cells light their own way; sizes reach the copies, not the far land. R4 see-through, R5 pieces and
bars, and R7 the line all the way in not started. Past those, moving things, not costed, and his yes or
no still open. A plain measure: how many ways the world is drawn, and how many lighting recipes.

*Counsel's question to Builder, not assessed:* builds 43 to 45 tried to make the far land's recipe and
the copies agree on a far grassy slope (brown against green). Could the far land sized per pixel answer
as blocks do, from the same data at a coarser size, so the two can't disagree, instead of being matched
by hand? If so, more matching now would be thrown away.

*Builder's answer:* no, not by construction. "Far land sized per pixel" only sizes the far land's own
detail; its look stays the recipe's. Copies made from the recipe, with cells of 8 or 16 blocks to the
horizon, are possible, but such a cell draws one tall step where the world has 8 or 16 small ones (the
stripes build 43 banded away), so it must still be drawn as the average of one-block steps: the recipe's
job. One way of drawing the shape, but the colour of a far slope stays one question: how much side, in
what colour. So matching to the reference isn't wasted; hand-tuning shades would be. If the reference shows
those slopes brown, the recipe is right and the textured coarse copies are off; if green, the recipe
overstates the sides. Builder guesses brown (a grass riser is mostly dirt). The reference is true only where
copies are a block a cell.

### VLR, variable-length rayshading: plan of attack (Counsel, 2026-11-04; a proposal, not assessed by Builder)

David asked for the plan and its fallbacks. After far land sized per pixel, as shading rays walk that loop.

**Ladder, each rung behind a switch and usable alone:** (0) a gauge first: slow, long sun rays, several a
pixel, to measure the rest against; (1) slope from column tops, foot shade from neighbouring cells;
(2) the horizon map for mountains (8 directions, about 40 MB); (3) short sun rays, a cone growing coarser,
length by what can still change the pixel, stopping once settled; (4) cells remembering "the sun sees me",
only if 3 costs too much; (5) coarse light levels, after regular lighting is solved (David). Placed light stays
in the cells. The light map retires only when these pass.

**Measure at each rung:** card time, low sun as the bad case, against the gauge, and in flight.

**Fallbacks:** rays only near, horizon map and today's light map past that; shorter rays by a dial; today's
light map behind its switch, which looks consistent now (build 47), so nothing is lost if this fails.

**Alternate routes:** light per cell, on a budget, instead of per pixel; a few rays a pixel blended over frames
(David's rayblending, suiting a slow sun, with ghosting on moving things); shading at lower resolution;
horizon map plus local shade with no shadow rays; the light map fed the real dug ground (smallest, no rays).

**Risks:** shadows popping as a cone's size changes; remembered light going stale when a block's dug.

**David, 2026-11-04, before VLR is assessed and planned again:** "we will assess and plan again before we engage with VLR." And on what shadows have to be: "shadows are one thing where 'more stuff on screen' isnt just a sword with its edge pointed towards us; its double edged, because as screen content increases, small flaws also become less apparent to the human eye; i think shadows not right up next to you close enough to know a bad guy is rounding the corner you are hiding behind will be very tolerant to the player. the goal is just to drive the perceived complexity past the resolution of a players ability to fully take in at a realtime rate. this makes it feel like life, and keeps you alert and engaged."

*Counsel, for that planning:* so the gauge isn't a pass mark everywhere: exact close enough to matter in play, tolerant further off and in busy scenes. One limit, from what's generally known about vision and not tested here: the eye forgives wrong far better than it forgives moving, so busy scenes hide a shadow in the wrong place more readily than one that pops or crawls. Far off, steady may count for more than exact.

**David, 2026-11-04:** "for small things in the distance, i think they simply don't need shadows. big things need shadows from a distance, but if you're watching a caravan make its way through a thin forest from half a mile away, i don't think anyone would immediately notice an *absence* of shadow. the gradations of brightness across the entire world from you to the thing you're looking swings so much more, i don't think someone in the distance not having a shadow could be noticed without carefully looking for shadows."

*Counsel, for that planning:* the same dial may give this for free: a shading ray grows coarser with distance, so a small thing in a coarse cell stops casting by itself, and big things stay. To check: a small thing's shadow should fade out, not pop (the motion point above); a thick forest should still darken the ground as a mass, though no one tree casts; and how a coarse cell counts a part-filled cell decides that. Moving things aren't in the cells yet, so they'd need their own rule.

**David, 2026-11-04:** "as long as we're working off one shader and one renderer, the rules we write inside of them to get it to work, are art."

**David, 2026-11-04:** "have we really handed the work to the gpu yet though". *Counsel:* for drawing, yes; for preparing the world, no. Build 59: card about 1.9 ms a frame, main and render threads about 3.8 and 3.6 ms, background jobs 13 to 24% of a core. No compute shaders yet: copies, the recipe, meshing and light maps all run on the processor, and copies are sent over at 128 KB a page. The next handoff, to assess: the recipe asked on the card, copies made there with nothing to send. The whole-number, GPU-safe projected code (decisions.md, the Metaphysics Engine) was meant to allow it. Not costed.

**David, 2026-11-04:** "part of the point of the entire graphics pass we chose to undertake was to offload most everything from the cpu to the gpu. we need to identify why its getting hot." *Counsel:* we had one thermometer (a motherboard zone) and had never read watts, though the laptop offers them without admin. Next: a ledger of watts by system (docs/far-view.md, Open concerns), then the slow frames, then the recipe on the card costed in watts as well as ms. Caution for the goal: on a laptop the processor and card share one cooler, so moving work to the card helps heat only if the card does it for fewer watts. It may well (wide and slow beats one core at top boost), but that's to be measured.

**David, 2026-11-04 (not decided):** "i feel as a mechanic the bilateral engine transfer we are doing would create a real mess in my lift bay. we got rivers working by ripping them out first. im confident in my default world gen preset and none of that is graphics. if we nuke all graphics and start from scratch we may get back to the blueprint factory faster. what do you think?" *Counsel:* pull the old engine, keep the new one. Old, about 3,200 lines (mesher, meshing, distant, terrain and distant shaders, material); new, about 5,500 (march, far copies, march shader), and it isn't what's failing. That is R7 moved from last to next, with no switch kept, so the bridge work (hand-off, copies of real chunks, matching at the line) stops. What it doesn't skip: water and glass, pieces and bars, one lighting, block edits showing at once. First: find the menu's 36 W, which is present with no world drawn; a new renderer could inherit it. Backup and a git tag before pulling anything.

**David, 2026-11-04 (not decided):** "what part of the editor suite cant be safely extracted from bevy? what part of the rudimentary but accurate quartility and behavior and npc mockups we have rely on something specific about bevies entity component system that we can not rebuild 'naively' ... at hardware idle, and think about how to maintain headroom on a current but not super powerful pc?" *Counsel, from a text count of src (55,038 lines):* almost nothing. About 47 Query uses and 14 Components in the whole game; about 322 Res uses, Bevy as a bag of globals. Editor: 5,710 of 23,108 lines touch Bevy proper, the rest plain or egui, which runs without Bevy. Sim, people, fate logic: plain records. Real rebuilding: the march's render path, models, window and frame loop, egui's host, background tasks. Unused so far: audio, animation, bevy_ui. I had told David and Builder most of the game leans on Bevy; that was wrong. Decided on the floor test, with a no-Bevy row added. Headroom: 250 fps on his card is the headroom: a card a quarter as strong still makes about 60, with the ray cost following the screen and resolution left to scale; rough, unmeasured.

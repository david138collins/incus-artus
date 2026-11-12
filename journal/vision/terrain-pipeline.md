# What's calculated where: the terrain pipeline

Written by Counsel, redrawn 2026-10-27 after the peaks were built (through 262d1d0), from the code
(`src/worldgen/compiler.rs` `compile()`, `src/world/interpreter.rs` `parts()`, `peaks()`,
`put_together()`, `height()`). For David: where each control acts, in the order it's applied, and where
a new layer could go. If the code changes, this goes stale; `docs/worldgen.md` stays the record.

---

## Stage 1. Tectonic Sim → the bake

Plates move and collide (land plates and ocean plates each with their own motion, heights and rock;
± spreads give each plate its own value). The bake saves five maps, one value per 32-block pixel:

| Map | What it is |
|---|---|
| **height** | the plate's surface height (land high, sea low) |
| **tilt** | which way each plate tips (its high side and low side) |
| **squeeze** | how crushed (+) or stretched (−) the rock is |
| **push direction** | which way the crush pushed |
| **push strength** | how clear that direction is |

Everything after this only *reads* these five maps. Nothing downstream changes the plates.

## Stage 2. The compile: maps prepared, once per compile

**Landmass panel**
1. **Sea level** (`water`) picks which bake height is the coastline. From it: the **coast map** (land +,
   sea −) and the **distance from the coast** for every pixel.
2. **Soften coast** blurs the coast, height and tilt. **Soften mountains** blurs squeeze and push.
   These only take the pixels' staircase off.

**Topography panel (was Additives), Mountains heading**
3. **Where:** the mountain points are picked from the crush (scattered, or range middles). They are
   saved with the world (`mountain_points`). From them: the **distance to the nearest point**, and a
   grid of cells, each listing the peaks that reach into it.
4. **Mountain grain, Grain drift, Ridge streams** still only draw dots. Nothing they make reaches the
   ground (a use for them is to be found, David).

(Foothills, rifts and passes are cut.)

**[IDEA, David 2026-10-28; the tab became Topography instead, 2026-10-31]:** if the Additives end up holding only mountains, the tab
becomes **Mountains**. After it comes a new stage, **Climate** (hydrology, temperature, wind, rain, fine
erosion, springs, creeks, rivers, deltas), whose fields Ecology reads. Climate chooses where water goes;
the Interpreter draws it per block. Details in `docs/worldgen.md`, "The stages after Mountains".
Climate gets its own stamps at points too: **biome cores** (the desert among them), small extreme
regions plotted into the climate as peaks are into the ridgelines, which enhance the ecology there.
**Decided 2026-10-28:** Climate is a **panel** in the World Compiler, between Additives (now Topography) and the World
Interpreter, with sections Temperature · Wind · Rain · Rivers. It reads the broad ground (the recipe with
crags, summit crags, hills and torn coast muted); wind reads sea-level temperature, so it crosses ranges.
**2026-10-31:** Rivers now come first, then Temperature, Wind, Rain (8f74856). The panels by stage
(ad79fec) are Landmass · Topography · Climate · Mineralogy · World Interpreter; Mineralogy has its own
panel, with the greyed Caves after it. Topography holds the ground's shape in David's order (8406459):
Land and sea · Ridgelines · Mountains · Hills · Torn coast, moved from the Interpreter panel, which now
shows only the chain. The recipe still works them out in its own order.
**Built 2026-10-28** (049c941 through e3cae58), as map overlays only: the broad ground (128-block
climate pixel), Temperature (squiggled ice lines and equator band, a soap film between them, milder by
the sea, cooler with height), Wind, Rain (upwind walk, with a "because" readout) and the flow (basins
filled and spilled on the 32-block grid, rain plus snowmelt and slope-feet springs, ~1.1 s). Not yet:
river lines and ranks, carving, water in the world.
**Sections since 2026-10-29:** Temperature · Wind · Rain · Rivers · **Mineralogy** · Caves (greyed).
**The river network, built 2026-10-30 (8e87946):** David's rework, growing rivers from the mouths by
choosing branches of step 5's drainage tree (`src/worldgen/network.rs`, a climate stage after the
flow; a "River network" section; a "rivers" overlay in Show; a readout under the mouse).
- Springs are placed by plain or relative height, switched by a toggle.
- Rank comes from stream order (branches joining), not water volume. Rain still decides which springs
  exist, since a spring needs wet enough ground.
- Next greyed step: **Carving**, by the shared contract that lowers the recipe's height.
**Carving, first version, built 2026-10-30 (6b16145).**
- The curves are worked out in a climate stage after the network. The carve lowers the compiled
  height, and a "rivers" mute keeps climate reading the uncarved ground, so nothing feeds back.
- Chunks get water with gravel or sand beds, and lakes are filled at crag-dip edges. Rivers are saved
  with the world (rivers.txt).
- The far terrain shows river and lake surfaces, and creeks fade in by how much of a cell they cover.
  The close-up shows rivers carved. Waterfall curtains are on a switch, on to start.
- "Keeps full size for" (David's control) sets the share of the length, from the mouth, at full width.
- Not yet: the bank table as sliders, keeping clear of other rivers, and coarse-then-sharper in the
  viewer.
**Since then, 2026-10-30:**
- Rain no longer feeds rivers (2019c03).
- Profiles step 1 (7963224): sizes by ratio from David's drawing (creek, stream, river, great river)
  and bowl-shaped beds. This replaces "keeps full size for".
- Routing (43403ae): each reach is filled and spilled on the real ground, in 4-block cells within 64
  blocks of the plan.
- Lakes (af8c8a0): step 5's lake patches and 64 blocks round them are grouped into regions, then each
  region is filled and spilled on the real ground with the router's method. Lakes take their basins'
  real shapes, nearby dips become ponds (from 1 block deep), water never stands against air, and
  rivers end at a lake's real level. The network still plans with step 5's lakes, so nothing feeds
  back. This replaces the crag-dip-edge filling and the raised square pools.
- Banks (e9b26b9): shores are flush, with levees where the land beside the water is lower. Each rank
  fades into the land over its own width each side (at least 2 blocks, half that on the outside of a
  bend), widening up to 3 times where it would be steeper than 1 in 1; beyond that it stays a gorge.
  Inside of a bend is smooth, outside is a ledge then a slope, straights are smooth (a bend counts
  from width/radius 0.15). A Banks heading on the panel; old saves have no banks.
- Dead ends and ponds behind rises (864350f): a reach into a lake ends at its nearest real water (or
  follows the lake's drain if it has none). A rise deeper than "cut through rises up to" holds a level
  pond behind it, and the river spills over the cut, unless the pond would spread past "a pond spreads
  at most". The islands David wants kept come from three accidents (loops in tight wiggles, a creek
  beside its river, a creek inside a river), recorded in rivers.md as not to be changed unchecked.
- Water walls (d816b60, 2026-10-31): rivers leave the sea alone and end where they first meet it;
  falls drop at the cliff edge as one-block sheets; a pond can't hold where it would drain into a
  lower river, and where water overlaps the highest shows; a pond reaching a lower lake takes it in at
  one level. River walls 95,535 → 7,845 blocks. A "cores used" slider (12 of 24 to start).
**The water rework, 2026-10-31** (`vision/water-from-the-start.md`, `docs/water-rework.md`).
**Done enough to go on** (David: "i'd say we've pretty much got rivers fixed"; "nothing is done,
everything is only done enough to do the next thing").
- **Built:**
  - a shores readout and overlay;
  - compressor banks;
  - a re-route only where a cut is deeper than X (near the sea and high up);
  - the sideways moves and bend sides cut;
  - least change across hollows and in steps;
  - gentler raised banks;
  - the anti-kitty-cornerer;
  - the ridge fix;
  - rivers alongside each other merging with a fall;
  - joins easing down;
  - the lake slab fix;
  - snowmelt and slope-foot springs retired;
  - sizes by area as a box (Shreve counts, the slope sets the shape).
- **On file:**
  - the fade along the river;
  - lakes picked first;
  - petering out;
  - rivers first in Climate (decided);
  - dry beds;
  - water drawn toward water.
- **Later:** the lopsided bed, white water.
**Mineralogy, rough first version, built 2026-10-29 (ea9c654):**
- The world gets temperature and rain as saved maps at spawn.
- The first rule that applies wins: snowy ground if cold, gabbro and stone by height, sand on beaches
  and in hot dry deserts, dry grass if dry, otherwise grass.
- Underneath: dirt under grass, dry grass and snow; sand under sand; then stone. The sea floor is sand
  in the shallows and gravel deeper. The zone edges wander.
- A rule whose block isn't registered yet is skipped, so David's blocks land as they're registered.
- `surface()`'s height bands are retired. River-reading parts (sediment, lake beds) come with rivers.

## Stage 3. World Interpreter: every block, in this order

The maps are blended smoothly between pixels to the block's spot. Then:

**a. The base ground** (land or sea)
- On land: sea level + (highlands − sea) × **land curve**(how high the bake is here) × **grade**(tilt).
- At sea: the same downward, with the **sea curve**, **deepest** and **grade**.

**b. Added onto the ground**
- \+ **Edge rise** (raised coast): on a plate's high side, a rise along the coast that eases back.
- \+ **Rise** (the Lift's cloth, now under Topography → Mountains): the tablecloth from the distance to
  the nearest point, braked to nothing near the coast. *Its settings are still stored in the recipe
  (`lift`), because it's drawn per block.*

**c. Ridgelines** (were "Mountains (from the squeeze map)")
- ridgelines = (Z 333 − ground) × **ridgeline curve**(squeeze) × **ridge carving**.
- They fill a *share of the room between the ground and Z 333*. 333 stays their measuring stick, so
  tuned worlds didn't change when the world grew taller.
- Ridge shape: **spacing, length, sharpness, layers, roughness**; direction from the push
  (**alignment, steady, wander**).

**d. Peaks** (Topography → Mountains; new)
- For each peak whose reach covers the block: its **rise curve** at how near its point is, times its
  height, out to its width. Each peak's height and width come from **± spreads** seeded from where it
  stands, with an **outline wobble**.
- Overlapping peaks **add up** (default, David's preference) or **the higher wins**.
- \+ **Crags:** Quilez's "More Noise". The rise is the first layer; its slope steers each finer layer
  through the **slope response** curve and the **steepest** scale. Crags are sized as a share of the
  peak's height, faded in from the foot; **carve below the peaks** lets them dig under it.
- Braked by the peaks' own **coast fade** (was the Rise's coast pad), so peaks never move the coast.
  The crags' **foot fade** says how far up from the foot they reach full strength. The outline wobble
  has its own ± spread; each crag layer's "finer" and "as strong" follow a curve. (4b7e098, b3cde44)
- \+ **Summit crags** (9bc56f8): a Perlin field pressed into all land high up, fading in from a start Z
  to full at a full Z along a curve, so the higher, the jaggier. Its own Show switch.

**e. Put together**
- Z = ground + ridgelines + peaks, cut at **Ground never above** (defaults to the world's top, Z 1,023).
- Never drowns land that was above the sea before the crags.
- \+ **Torn coast** (Perlin, in a band along the coast, in patches; staying).
- (The old Hills were cut, 77eed95: "none of the controls ... do a single thing right".)

**f. Hills** (was ground detail; renamed 9bc56f8, David: "basically what i wanted from hills")
- \+ **Hills** (Perlin, up to 40 blocks, sizes to 2,000, faded by its **mask**), kept off the
  waterline. Its settings keep their old names in presets.

**g. Surface blocks:** **stone from** and **gabbro from** colour by height. Shape is untouched.

---

## Things worth knowing

- **Peaks add on top; ridgelines share the room.** The ridgelines still shrink under raised ground
  (they fill a share of the room up to 333), but peaks are added after them at their own height, so
  raising the ground lifts the peaks with it.
- **The ridgelines are the nest, the peaks the mountains** (David): two things made from the same
  data, the crush.
- **The world is 1,024 tall.** Peaks tower at ~512 in the distance. The height tint, the near light
  window's height byte and the elevation picture still count to ~400.
- **Only the peaks vary from place to place by rule**; torn coast and hills are still the same
  size everywhere unless a mask fades them.
- **The panels' order isn't quite the sums' order:** the Rise sits under Topography but is drawn in 3b,
  before the ridgelines.

## Where a new layer can go (the sockets)

| Socket | What goes there | Its effect | Already there |
|---|---|---|---|
| **1. A new map** (stage 2) | a map or list made once from the bake | every later step can read it; the cleanest place for anything that should steer several things at once | coast distance, mountain points and their cells |
| **2. Added to the ground** (3b) | broad lifts and dips | moves the base; the ridgelines adjust to fit the room | edge rise, the Rise |
| **3. A multiplier on the ridgelines** (3c) | something that scales their share or carving | how tall or rugged the nest is, not where | ridgeline curve, ridge carving |
| **4. Stamps at points** (3d) | shapes drawn round a list of points | placed features with their own outline and texture | the peaks |
| **5. Added at the end** (3e, 3f) | free shapes on top of everything | easiest to add, hardest to control | summit crags, torn coast, hills |
| **6. Carved last** (after 3e; planned) | the Climate stage's carve map | rivers cut through everything above | nothing yet |

## A clean starting point

The Show switches turn off edge rise, the Rise, ridgelines, ridge carving, peaks, crags, summit crags,
torn coast and hills. With all of them off, what's left is stages 1–3a: **the bare continent**. Saved as a
preset (say `bare`), that's a fixed place to start from.

**Spawning follows the switches** (cdbf751): Render World and Spawn leaves out whatever is switched
off, a pink label in the world names what's missing, and the world remembers it (`left_out` in
compiled.toml). The big map still uses the whole recipe.
